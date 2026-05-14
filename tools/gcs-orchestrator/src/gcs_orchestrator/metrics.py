"""Per-turn latency metrics for regression detection + viewer footer.

Schema v=1 — all durations are integer milliseconds (no unit drift).

Privacy contract (hard rule, do not violate):
- No `content`, no `assistant_text`, no `cwd`, no env vars.
- Only counts, durations, status, taxonomy codes.
- `content_sha256` is the join key to receipts and is non-reversible.

Storage: %LOCALAPPDATA%/gcs-app-relay/metrics.jsonl with size-based
rotation (10 MB × 5 backups ≈ 30+ months of headroom at 20 KB/hour).
"""
from __future__ import annotations

import json
import statistics
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Optional

from .config import load as load_config
from .platform import FileLock


SCHEMA_VERSION = 1

# Field taxonomy — keep stable so consumers don't break on additions
_REQUIRED_FIELDS = ("ts", "schema", "thread_id", "status", "total_ms", "relay_version")
_DURATION_FIELDS = (
    "ws_connect_ms", "initialize_ms", "resume_ms", "lock_wait_ms",
    "ttft_ms", "turn_ms", "total_ms",
)
_COUNT_FIELDS = ("input_tokens", "cached_input_tokens", "output_tokens", "retry_count")
_STATUS_VALUES = ("ok", "error", "aborted", "timeout", "duplicate")

_ROTATE_MAX_BYTES = 10 * 1024 * 1024
_ROTATE_BACKUPS = 5


def _metrics_path(base: Optional[Path] = None) -> Path:
    if base is None:
        base = load_config().relay_base_dir()
    return base / "metrics.jsonl"


def _lock_path(metrics_path: Path) -> Path:
    return metrics_path.with_suffix(".lock")


def _rotate_if_needed(path: Path) -> None:
    if not path.exists():
        return
    try:
        if path.stat().st_size < _ROTATE_MAX_BYTES:
            return
        # shift .4 -> .5, .3 -> .4, ... .1 -> .2, base -> .1
        for i in range(_ROTATE_BACKUPS, 0, -1):
            src = path.with_suffix(f".jsonl.{i-1}") if i > 1 else path
            dst = path.with_suffix(f".jsonl.{i}")
            if src.exists():
                if dst.exists():
                    dst.unlink()
                src.replace(dst)
    except OSError:
        # rotation is best-effort; never block a metric write
        pass


def append(record: dict, *, base: Optional[Path] = None) -> None:
    """Append one metric record. Holds a FileLock for the write.

    Best-effort: swallows all errors so a metric write never breaks a
    turn. The receipt is still the source of truth.
    """
    try:
        record = dict(record)  # don't mutate caller's dict
        record.setdefault("schema", SCHEMA_VERSION)
        record.setdefault("ts", datetime.now(timezone.utc).isoformat(timespec="milliseconds").replace("+00:00", "Z"))
        path = _metrics_path(base)
        path.parent.mkdir(parents=True, exist_ok=True)
        with FileLock(_lock_path(path), timeout_s=2.0):
            _rotate_if_needed(path)
            with path.open("a", encoding="utf-8") as fh:
                fh.write(json.dumps(record, ensure_ascii=False) + "\n")
    except Exception:
        pass


def read_recent(n: int = 20, *, base: Optional[Path] = None) -> list[dict]:
    """Tail-read the metrics file plus rotated .1 if needed. Newest-first."""
    path = _metrics_path(base)
    out: list[dict] = []
    files = [path]
    rotated = path.with_suffix(".jsonl.1")
    if rotated.exists():
        files.append(rotated)
    for p in files:
        if not p.exists():
            continue
        try:
            # read entire file (small enough — capped at 10 MB rotation)
            lines = p.read_text(encoding="utf-8", errors="replace").splitlines()
        except OSError:
            continue
        for line in reversed(lines):
            line = line.strip()
            if not line:
                continue
            try:
                out.append(json.loads(line))
            except json.JSONDecodeError:
                continue
            if len(out) >= n:
                return out
    return out


def percentiles(field: str, n: int = 20, ps: tuple[int, ...] = (50, 95),
                *, base: Optional[Path] = None,
                status_filter: str = "ok") -> dict[int, Optional[float]]:
    """Compute percentiles for `field` over the last `n` records.

    Filters to status == status_filter (default "ok") and skips records
    where field is missing or non-numeric. Returns {p: value_or_None}.
    """
    records = read_recent(n=max(n, 1), base=base)
    values: list[float] = []
    for r in records:
        if status_filter and r.get("status") != status_filter:
            continue
        v = r.get(field)
        if isinstance(v, (int, float)):
            values.append(float(v))
    if not values:
        return {p: None for p in ps}
    out: dict[int, Optional[float]] = {}
    for p in ps:
        try:
            # statistics.quantiles requires n >= 2; for small samples fall back
            if len(values) == 1:
                out[p] = values[0]
            else:
                # method='inclusive' so p50 of [1,2,3] gives 2 exactly
                qs = statistics.quantiles(sorted(values), n=100, method="inclusive")
                out[p] = qs[p - 1] if 1 <= p <= 99 else max(values)
        except (statistics.StatisticsError, IndexError):
            out[p] = None
    return out


def summary_line(n: int = 20, *, base: Optional[Path] = None) -> str:
    """One-line p50/p95 summary for footer display."""
    pcts = percentiles("total_ms", n=n, ps=(50, 95), base=base)
    p50 = pcts.get(50)
    p95 = pcts.get(95)
    if p50 is None:
        return f"metrics: (no data yet)"
    def fmt(v: Optional[float]) -> str:
        if v is None:
            return "-"
        if v >= 1000:
            return f"{v/1000:.1f}s"
        return f"{int(v)}ms"
    return f"last{n} p50={fmt(p50)} p95={fmt(p95)}"
