"""Structured audit log for the dispatcher (~/.codex/dispatch-audit.jsonl).

Lives next to the human-readable gcs-dispatch.log; this one is the
machine-readable trail. One JSON object per decision the dispatcher
makes (SPAWN/CLEAR/REPONDRE/RELANCER/NEW marker noted/AUTO_PROMOTE/
DEDUP_SKIP/FALLBACK_SPAWN).

Consumer use cases:
- "Show me every FAILED spawn in the last 24h":
    grep '"success":false' dispatch-audit.jsonl | recent
- "Which workers were CLEAR'd that I didn't expect":
    jq 'select(.decision_type == "CLEAR")' dispatch-audit.jsonl
- "What's the dispatcher's p95 latency right now":
    feed into metrics.percentiles or use jq

Rotation: size-based, 5 MB × 5 backups (~20 weeks of headroom at usual rate).
"""
from __future__ import annotations

import json
import os
import threading
import time
import uuid
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Optional


_AUDIT_PATH = Path(os.path.expanduser("~/.codex/dispatch-audit.jsonl"))
_LOCK = threading.Lock()
_ROTATE_MAX_BYTES = 5 * 1024 * 1024
_ROTATE_BACKUPS = 5


def _rotate_if_needed(path: Path) -> None:
    try:
        if not path.exists() or path.stat().st_size < _ROTATE_MAX_BYTES:
            return
        for i in range(_ROTATE_BACKUPS, 0, -1):
            src = path.with_suffix(f".jsonl.{i-1}") if i > 1 else path
            dst = path.with_suffix(f".jsonl.{i}")
            if src.exists():
                if dst.exists():
                    dst.unlink()
                src.replace(dst)
    except OSError:
        pass


def new_dispatch_id() -> str:
    """One id per dispatcher main() invocation — correlates decisions."""
    return uuid.uuid4().hex[:12]


def record(*, dispatch_id: str, decision_type: str,
           prompt_id: Optional[str] = None,
           action: Optional[str] = None,
           octogent_status: Optional[int] = None,
           success: bool = True,
           latency_ms: Optional[int] = None,
           note: Optional[str] = None,
           extra: Optional[dict] = None,
           path: Optional[Path] = None) -> None:
    """Append one audit record. Best-effort, swallows all errors."""
    try:
        target = path or _AUDIT_PATH
        target.parent.mkdir(parents=True, exist_ok=True)
        rec: dict[str, Any] = {
            "ts": datetime.now(timezone.utc).isoformat(timespec="milliseconds").replace("+00:00", "Z"),
            "dispatch_id": dispatch_id,
            "decision_type": decision_type,
            "success": success,
        }
        if prompt_id is not None:
            rec["prompt_id"] = prompt_id
        if action is not None:
            rec["action"] = action
        if octogent_status is not None:
            rec["octogent_status"] = octogent_status
        if latency_ms is not None:
            rec["latency_ms"] = latency_ms
        if note is not None:
            rec["note"] = note
        if extra:
            rec.update(extra)
        with _LOCK:
            _rotate_if_needed(target)
            with target.open("a", encoding="utf-8") as fh:
                fh.write(json.dumps(rec, ensure_ascii=False) + "\n")
    except Exception:
        pass


def tail(n: int = 50, *, path: Optional[Path] = None,
         dispatch_id: Optional[str] = None) -> list[dict]:
    """Read the last n audit records, optionally filtered to one dispatch_id."""
    target = path or _AUDIT_PATH
    if not target.exists():
        return []
    try:
        lines = target.read_text(encoding="utf-8", errors="replace").splitlines()
    except OSError:
        return []
    out: list[dict] = []
    for line in reversed(lines):
        line = line.strip()
        if not line:
            continue
        try:
            d = json.loads(line)
        except json.JSONDecodeError:
            continue
        if dispatch_id is not None and d.get("dispatch_id") != dispatch_id:
            continue
        out.append(d)
        if len(out) >= n:
            break
    return out
