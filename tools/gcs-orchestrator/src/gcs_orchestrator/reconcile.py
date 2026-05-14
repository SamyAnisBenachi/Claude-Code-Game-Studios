"""Reconcile worker reports/ vs orchestrator-known turns.

Catches silent loss in either direction:
- orphan-report: report file exists on disk but no DONE turn ever reached
  the orchestrator → worker→orch path lost the message
- silent-loss: orchestrator's assistant text mentions spawning PROMPT-N
  (via SPAWN/NEW disposition) but no report file exists and no DONE
  turn was injected → worker died before writing Step 2

Usage:
    gcs-reconcile [--since 24h] [--json]

Matching strategy:
- Reports keyed by `PROMPT-N` from filename
- Turns scanned via FTS5 index (rollout_grep). Looks for "PROMPT-N" in
  user-role turns (DONE injection) and assistant-role turns (spawn or
  reference). A pair = report file + at least one matching user turn.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from datetime import datetime, timedelta
from pathlib import Path
from typing import Optional

from .config import load as load_config
from .backup import _resolve_rollout_path
from .rollout_grep import build_or_update_index, search, _index_path_for


REPORTS_DIR = Path("D:/_DEV/Work/Claude-Code-Game-Studios/reports")
PROMPT_PATTERN = re.compile(r"PROMPT[-_]?(\d+)", re.IGNORECASE)


def _parse_since(since: str) -> Optional[float]:
    if not since:
        return None
    s = since.strip().lower()
    now = datetime.now()
    try:
        if s.endswith("h"):
            return (now - timedelta(hours=int(s[:-1]))).timestamp()
        if s.endswith("d"):
            return (now - timedelta(days=int(s[:-1]))).timestamp()
        if s.endswith("m"):
            return (now - timedelta(minutes=int(s[:-1]))).timestamp()
        return datetime.fromisoformat(s).timestamp()
    except ValueError:
        return None


def _scan_reports(reports_dir: Path, since_ts: Optional[float]) -> dict[int, list[Path]]:
    """Map PROMPT-N -> list of report file paths (may have multiple per N)."""
    out: dict[int, list[Path]] = {}
    if not reports_dir.exists():
        return out
    for p in reports_dir.glob("*.md"):
        if since_ts is not None and p.stat().st_mtime < since_ts:
            continue
        m = PROMPT_PATTERN.search(p.name)
        if not m:
            continue
        n = int(m.group(1))
        out.setdefault(n, []).append(p)
    return out


def _scan_rollout_for_prompts(index_path: Path, since_ts: Optional[float]) -> dict[int, dict]:
    """For each PROMPT-N mentioned in the rollout, return:
       {n: {"user_turns": [...], "assistant_turns": [...]}}

    Uses FTS5 to find every line containing PROMPT- pattern; parses N.
    """
    out: dict[int, dict] = {}
    # FTS5 doesn't do regex; query for the substring "PROMPT-" then filter
    hits = search(index_path, '"PROMPT-"', role=None, limit=10_000)
    for h in hits:
        # Re-derive N from the snippet text
        snip = h.get("snippet") or ""
        ts = h.get("timestamp", "") or ""
        if since_ts is not None and ts:
            try:
                if datetime.fromisoformat(ts.replace("Z", "+00:00")).timestamp() < since_ts:
                    continue
            except (ValueError, AttributeError):
                pass
        # Extract every PROMPT-N from this hit
        for m in PROMPT_PATTERN.finditer(snip):
            n = int(m.group(1))
            bucket = out.setdefault(n, {"user_turns": [], "assistant_turns": []})
            role = h.get("role", "")
            entry = {"ts": ts, "snippet": snip[:200]}
            if role == "user":
                bucket["user_turns"].append(entry)
            elif role == "assistant":
                bucket["assistant_turns"].append(entry)
    return out


def reconcile(reports_dir: Path, *, since: str = "",
              rebuild_index: bool = False) -> list[dict]:
    """Return list of reconciliation rows, one per distinct PROMPT-N seen."""
    since_ts = _parse_since(since)
    cfg = load_config()

    # Make sure the FTS5 index is current
    rollout = _resolve_rollout_path(cfg.orchestrator.session_id)
    if rollout is None or not rollout.exists():
        return [{"error": "rollout not found",
                 "session_id": cfg.orchestrator.session_id}]
    index_path = _index_path_for(rollout, cfg.paths.codex_home)
    build_or_update_index(rollout, index_path, reindex=rebuild_index)

    reports_by_n = _scan_reports(reports_dir, since_ts)
    turns_by_n = _scan_rollout_for_prompts(index_path, since_ts)

    all_ns = sorted(set(reports_by_n.keys()) | set(turns_by_n.keys()))

    rows: list[dict] = []
    for n in all_ns:
        files = reports_by_n.get(n, [])
        bucket = turns_by_n.get(n, {"user_turns": [], "assistant_turns": []})
        user_turns = bucket["user_turns"]
        assistant_turns = bucket["assistant_turns"]
        has_report = bool(files)
        has_done = bool(user_turns)
        has_mention = bool(assistant_turns)

        if has_report and has_done:
            status = "matched"
        elif has_report and not has_done:
            status = "orphan-report"  # file on disk, no DONE reached orchestrator
        elif has_done and not has_report:
            status = "done-no-report"  # DONE reached orch but no file on disk
        elif has_mention and not has_report and not has_done:
            status = "silent-loss"  # orchestrator referenced it but worker never reported
        else:
            status = "unknown"

        rows.append({
            "prompt_id": f"PROMPT-{n}",
            "status": status,
            "report_files": [str(p) for p in files],
            "user_turn_count": len(user_turns),
            "assistant_mention_count": len(assistant_turns),
            "latest_user_ts": user_turns[0]["ts"] if user_turns else None,
            "latest_assistant_ts": assistant_turns[0]["ts"] if assistant_turns else None,
            "report_mtimes": [datetime.fromtimestamp(p.stat().st_mtime).isoformat(timespec="seconds")
                              for p in files],
        })
    return rows


def main(argv: Optional[list[str]] = None) -> int:
    parser = argparse.ArgumentParser(prog="gcs-reconcile")
    parser.add_argument("--since", default="",
                        help="time window: 24h, 7d, 30m, or ISO datetime")
    parser.add_argument("--json", action="store_true",
                        help="raw JSON output instead of table")
    parser.add_argument("--reports-dir", default=str(REPORTS_DIR),
                        help="directory containing PROMPT-*.md files")
    parser.add_argument("--reindex", action="store_true",
                        help="rebuild FTS5 index from scratch before reconciling")
    parser.add_argument("--status", default="",
                        choices=["", "matched", "orphan-report", "done-no-report",
                                 "silent-loss", "unknown"],
                        help="filter rows by status")
    args = parser.parse_args(argv)

    rows = reconcile(Path(args.reports_dir), since=args.since,
                     rebuild_index=args.reindex)
    if args.status:
        rows = [r for r in rows if r.get("status") == args.status]

    if args.json:
        print(json.dumps(rows, indent=2, default=str))
        return 0

    if not rows:
        print("(no rows)")
        return 0

    if rows and rows[0].get("error"):
        print(f"error: {rows[0]['error']}")
        return 2

    print(f"{'PROMPT-N':<12} {'status':<18} {'files':<6} {'user_turns':<11} {'assistant_mentions':<19} latest_user_ts")
    print("-" * 110)
    for r in rows:
        print(f"{r['prompt_id']:<12} {r['status']:<18} {len(r['report_files']):<6} "
              f"{r['user_turn_count']:<11} {r['assistant_mention_count']:<19} "
              f"{(r['latest_user_ts'] or '-')[:19]}")
    print(f"\n{len(rows)} rows")
    # Summary
    by_status: dict[str, int] = {}
    for r in rows:
        by_status[r["status"]] = by_status.get(r["status"], 0) + 1
    print(f"summary: {by_status}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
