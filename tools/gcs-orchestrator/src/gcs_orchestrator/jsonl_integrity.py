"""JSONL rollout integrity check + auto-repair.

Validates a Codex rollout file is well-formed JSONL (one JSON object per
line). On detection of a torn final line (write interrupted mid-line by
a power loss / OS reboot), can auto-truncate to the last valid newline
and emit a `.repaired-<ts>.jsonl` sidecar with the chopped bytes.

Used by:
- the daily backup cron (validate before copying)
- the supervisor at app-server startup (refuse to start a corrupt rollout)
- ad-hoc operator check via `gcs-rollout-check` CLI

Public API:
    validate(path) → ValidationResult
    repair(path) → RepairResult

CLI:
    python -m gcs_orchestrator.jsonl_integrity <rollout-path> [--repair]
"""
from __future__ import annotations

import argparse
import json
import sys
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Optional


@dataclass
class ValidationResult:
    ok: bool
    line_count: int
    bytes_total: int
    last_line_complete: bool
    last_line_parses: bool
    last_line_preview: str = ""
    error: str = ""


@dataclass
class RepairResult:
    repaired: bool
    truncated_bytes: int
    backup_path: Optional[Path]
    reason: str = ""


# Tail bytes to scan when looking for last newline / parse-validating
TAIL_SCAN_BYTES = 4096


def validate(path: Path) -> ValidationResult:
    """Cheap integrity check: confirm file is non-empty, line-terminated,
    and the last line parses as JSON. Does NOT parse every line —
    use validate_full() for that (slower)."""
    try:
        size = path.stat().st_size
    except OSError as exc:
        return ValidationResult(False, 0, 0, False, False, error=str(exc))

    if size == 0:
        return ValidationResult(False, 0, 0, False, False, error="file is empty")

    # Read tail
    with path.open("rb") as f:
        f.seek(max(0, size - TAIL_SCAN_BYTES), 0)
        tail = f.read()

    last_complete = tail.endswith(b"\n")
    last_line_bytes = tail.rsplit(b"\n", 2)[-2 if last_complete else -1]
    last_preview = last_line_bytes[:200].decode("utf-8", errors="replace")

    last_parses = False
    if last_line_bytes:
        try:
            json.loads(last_line_bytes.decode("utf-8"))
            last_parses = True
        except (json.JSONDecodeError, UnicodeDecodeError):
            last_parses = False

    # Line count: cheap approximation by counting newlines in the tail proportionally
    # (don't read the whole file). For exact count, use validate_full().
    line_count = -1  # unknown without full scan

    ok = last_complete and last_parses
    return ValidationResult(
        ok=ok,
        line_count=line_count,
        bytes_total=size,
        last_line_complete=last_complete,
        last_line_parses=last_parses,
        last_line_preview=last_preview,
    )


def validate_full(path: Path) -> ValidationResult:
    """Strict: parse every line. Returns first-failure detail."""
    try:
        size = path.stat().st_size
    except OSError as exc:
        return ValidationResult(False, 0, 0, False, False, error=str(exc))

    count = 0
    with path.open("r", encoding="utf-8", errors="strict") as f:
        for n, line in enumerate(f, start=1):
            count = n
            stripped = line.rstrip("\n")
            if not stripped:
                continue
            try:
                json.loads(stripped)
            except json.JSONDecodeError as exc:
                return ValidationResult(
                    ok=False, line_count=n, bytes_total=size,
                    last_line_complete=line.endswith("\n"),
                    last_line_parses=False,
                    last_line_preview=stripped[:200],
                    error=f"line {n}: {exc}",
                )
    return ValidationResult(
        ok=True, line_count=count, bytes_total=size,
        last_line_complete=True, last_line_parses=True,
    )


def repair(path: Path) -> RepairResult:
    """Truncate file to last valid newline. Writes the chopped bytes to
    a `.repaired-<ts>` sidecar so nothing is lost permanently.

    Only safe on a quiesced file — caller must ensure no writer is open.
    """
    result = validate(path)
    if result.ok:
        return RepairResult(repaired=False, truncated_bytes=0, backup_path=None,
                            reason="already valid; nothing to repair")

    try:
        size = path.stat().st_size
    except OSError as exc:
        return RepairResult(False, 0, None, reason=f"stat failed: {exc}")

    # Find last newline by scanning backwards
    with path.open("rb") as f:
        f.seek(max(0, size - TAIL_SCAN_BYTES), 0)
        tail = f.read()
    last_nl_in_tail = tail.rfind(b"\n")
    if last_nl_in_tail == -1:
        # No newline in last 4KB — file is structurally broken at the end
        return RepairResult(False, 0, None,
                            reason="no newline found in last 4KB; file too corrupted for auto-repair")

    # Compute absolute offset of the last good byte (one past the last \n)
    tail_start_offset = max(0, size - TAIL_SCAN_BYTES)
    cut_offset = tail_start_offset + last_nl_in_tail + 1
    if cut_offset >= size:
        return RepairResult(False, 0, None,
                            reason="file already ends in newline but didn't parse — last line corrupt mid-content")

    # Save chopped bytes to sidecar
    ts = time.strftime("%Y%m%dT%H%M%S")
    sidecar = path.with_suffix(path.suffix + f".repaired-{ts}.dropped")
    with path.open("rb") as fin, sidecar.open("wb") as fout:
        fin.seek(cut_offset)
        fout.write(fin.read())

    # Truncate the rollout
    with path.open("rb+") as f:
        f.truncate(cut_offset)

    return RepairResult(
        repaired=True,
        truncated_bytes=size - cut_offset,
        backup_path=sidecar,
        reason="truncated to last valid newline",
    )


def main(argv: Optional[list[str]] = None) -> int:
    parser = argparse.ArgumentParser(prog="gcs-rollout-check")
    parser.add_argument("path", help="rollout JSONL file to validate")
    parser.add_argument("--full", action="store_true", help="parse every line (slow but thorough)")
    parser.add_argument("--repair", action="store_true",
                        help="auto-truncate to last valid newline; backup goes to .repaired-<ts>.dropped")
    args = parser.parse_args(argv)

    path = Path(args.path)
    if not path.exists():
        sys.stderr.write(f"not found: {path}\n")
        return 2

    if args.full:
        result = validate_full(path)
    else:
        result = validate(path)

    print(f"path:                {path}")
    print(f"size:                {result.bytes_total:,} bytes")
    print(f"last_line_complete:  {result.last_line_complete}")
    print(f"last_line_parses:    {result.last_line_parses}")
    if result.last_line_preview:
        print(f"last_line_preview:   {result.last_line_preview!r}")
    if result.error:
        print(f"error:               {result.error}")
    print(f"ok:                  {result.ok}")

    if not result.ok and args.repair:
        rep = repair(path)
        print(f"\nrepair attempt:")
        print(f"  repaired:        {rep.repaired}")
        print(f"  truncated_bytes: {rep.truncated_bytes}")
        print(f"  backup_path:     {rep.backup_path}")
        print(f"  reason:          {rep.reason}")
        return 0 if rep.repaired else 3

    return 0 if result.ok else 1


if __name__ == "__main__":
    sys.exit(main())
