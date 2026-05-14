"""Relay-receipts timeline CLI.

Aggregates the receipt JSON files into a chronological table so operators
can answer "which workers reported DONE in the last 24h, with what
outcomes" without grepping the receipts dir by hand.

Usage:
    python -m gcs_orchestrator.history [--since 24h] [--status success|pending] [--limit 100]
"""
from __future__ import annotations

import argparse
import json
import sys
import time
from datetime import datetime, timedelta
from pathlib import Path
from typing import Optional

from .config import load as load_config


def _parse_since(since: str) -> Optional[datetime]:
    """Accepts '24h', '7d', '30m', or an ISO datetime."""
    if not since:
        return None
    s = since.strip().lower()
    if s.endswith("h"):
        return datetime.now() - timedelta(hours=int(s[:-1]))
    if s.endswith("d"):
        return datetime.now() - timedelta(days=int(s[:-1]))
    if s.endswith("m"):
        return datetime.now() - timedelta(minutes=int(s[:-1]))
    try:
        return datetime.fromisoformat(s)
    except ValueError:
        return None


def _load_receipts(base: Path) -> list[dict]:
    receipts_dir = base / "receipts"
    if not receipts_dir.exists():
        return []
    out = []
    for p in receipts_dir.glob("*.receipt"):
        try:
            d = json.loads(p.read_text(encoding="utf-8"))
            d["_path"] = str(p)
            d["_mtime"] = p.stat().st_mtime
            out.append(d)
        except (OSError, json.JSONDecodeError):
            continue
    out.sort(key=lambda d: d.get("_mtime", 0), reverse=True)
    return out


def main(argv: Optional[list[str]] = None) -> int:
    parser = argparse.ArgumentParser(prog="gcs-relay-history")
    parser.add_argument("--since", default="", help="time window: 24h, 7d, 30m, or ISO datetime")
    parser.add_argument("--status", default="", choices=["", "success", "pending", "failed"],
                        help="filter by receipt status")
    parser.add_argument("--limit", type=int, default=100, help="max rows (default 100)")
    parser.add_argument("--json", action="store_true", help="raw JSON output instead of table")
    args = parser.parse_args(argv)

    cfg = load_config()
    base = cfg.relay_base_dir()
    receipts = _load_receipts(base)

    cutoff = _parse_since(args.since)
    if cutoff is not None:
        cutoff_ts = cutoff.timestamp()
        receipts = [r for r in receipts if r.get("_mtime", 0) >= cutoff_ts]

    if args.status:
        receipts = [r for r in receipts if r.get("status") == args.status]

    receipts = receipts[: args.limit]

    if args.json:
        print(json.dumps(receipts, indent=2, default=str))
        return 0

    if not receipts:
        print("(no matching receipts)")
        return 0

    # Pretty table
    print(f"{'completed_at':<22} {'status':<10} {'elapsed':<9} {'in':<7} {'cached':<7} {'out':<6} preview")
    print("-" * 110)
    for r in receipts:
        ts = r.get("completed_at") or r.get("started_at") or "-"
        ts = ts[:19] if isinstance(ts, str) else "-"
        status = r.get("status", "-")
        elapsed = r.get("elapsed_s")
        elapsed_s = f"{elapsed:.1f}s" if isinstance(elapsed, (int, float)) else "-"
        in_t = r.get("input_tokens") or 0
        cached = r.get("cached_input_tokens") or 0
        out_t = r.get("output_tokens") or 0
        preview = (r.get("assistant_text_preview") or "").replace("\n", " ")[:50]
        print(f"{ts:<22} {status:<10} {elapsed_s:<9} {in_t:<7} {cached:<7} {out_t:<6} {preview}")

    print(f"\n{len(receipts)} receipts shown (base={base})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
