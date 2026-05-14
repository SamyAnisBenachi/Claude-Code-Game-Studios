"""Full-text search across the Codex rollout via SQLite FTS5 index.

Day-2 ops feature: "when did the orchestrator last discuss X?" Without
this, finding anything in a 90 MB JSONL means scrolling an editor that
chokes on the size.

The index is rebuilt incrementally — at each run, we resume from the
byte offset stored in the index metadata and append new lines. First
build on a 90 MB rollout: ~30s. Subsequent runs: sub-second.

Usage:
    python -m gcs_orchestrator.rollout_grep <regex>
    python -m gcs_orchestrator.rollout_grep --reindex <regex>
    python -m gcs_orchestrator.rollout_grep --role user <regex>
"""
from __future__ import annotations

import argparse
import json
import re
import sqlite3
import sys
import time
from pathlib import Path
from typing import Iterator, Optional

from .config import load as load_config
from .backup import _resolve_rollout_path


SCHEMA_SQL = """
CREATE VIRTUAL TABLE IF NOT EXISTS turns USING fts5(
    timestamp,
    role,
    type,
    text,
    line_offset UNINDEXED,
    tokenize = 'porter unicode61'
);
CREATE TABLE IF NOT EXISTS meta (
    key TEXT PRIMARY KEY,
    value TEXT
);
"""


def _index_path_for(rollout: Path, cfg_codex_home: str) -> Path:
    """Index lives next to rollout; one .fts5.db per session UUID."""
    sid = rollout.stem.rsplit("-", 5)[-1] if "-" in rollout.stem else "unknown"
    index_dir = Path(cfg_codex_home) / "indexes"
    index_dir.mkdir(parents=True, exist_ok=True)
    return index_dir / f"{sid}.fts5.db"


def _iter_lines_from(rollout: Path, start_offset: int) -> Iterator[tuple[int, dict]]:
    """Yield (byte_offset_at_line_start, parsed_dict) for each JSONL line
    starting at start_offset.

    On JSON parse failure, skip the line silently (corruption is the
    integrity validator's concern, not the indexer's).
    """
    with rollout.open("rb") as f:
        f.seek(start_offset)
        while True:
            line_offset = f.tell()
            line = f.readline()
            if not line:
                return
            if not line.endswith(b"\n"):
                # Incomplete final line — rollout is being written;
                # bail out and let the next run pick it up.
                return
            try:
                yield line_offset, json.loads(line.decode("utf-8"))
            except (json.JSONDecodeError, UnicodeDecodeError):
                continue


def _extract_searchable_text(d: dict) -> tuple[str, str, str]:
    """From a rollout JSONL entry, return (role, type, text) for indexing."""
    payload = d.get("payload") if isinstance(d.get("payload"), dict) else d
    typ = d.get("type", "") or payload.get("type", "")
    role = payload.get("role", "") if isinstance(payload, dict) else ""
    text = ""
    content = payload.get("content") if isinstance(payload, dict) else None
    if isinstance(content, list):
        for c in content:
            if isinstance(c, dict):
                text += " " + (c.get("text") or c.get("input_text") or c.get("output_text") or "")
            elif isinstance(c, str):
                text += " " + c
    elif isinstance(content, str):
        text = content
    return role[:32], typ[:32], text.strip()[:50_000]  # cap text per entry


def build_or_update_index(rollout: Path, index_path: Path, reindex: bool = False) -> dict:
    """Build (full) or update (incremental) the FTS5 index for a rollout.

    Returns stats dict: {indexed, skipped, elapsed_s, rollout_size}.
    """
    rollout_size = rollout.stat().st_size

    if reindex and index_path.exists():
        index_path.unlink()

    conn = sqlite3.connect(str(index_path))
    conn.executescript(SCHEMA_SQL)

    cur = conn.execute("SELECT value FROM meta WHERE key = 'last_offset'")
    row = cur.fetchone()
    start_offset = int(row[0]) if row else 0

    t0 = time.time()
    indexed = 0
    skipped = 0
    last_offset = start_offset

    conn.execute("BEGIN")
    try:
        for line_offset, d in _iter_lines_from(rollout, start_offset):
            role, typ, text = _extract_searchable_text(d)
            ts = d.get("timestamp", "") or ""
            if not text:
                skipped += 1
                last_offset = line_offset + len(json.dumps(d).encode("utf-8")) + 1
                continue
            conn.execute(
                "INSERT INTO turns(timestamp, role, type, text, line_offset) VALUES (?, ?, ?, ?, ?)",
                (ts, role, typ, text, line_offset),
            )
            indexed += 1
            last_offset = line_offset + len(json.dumps(d).encode("utf-8")) + 1

        # Persist the last_offset as our next start point (best-effort)
        # We use the file's actual size as the conservative high-water mark.
        new_high = rollout_size
        conn.execute(
            "INSERT OR REPLACE INTO meta(key, value) VALUES (?, ?)",
            ("last_offset", str(new_high)),
        )
        conn.execute("COMMIT")
    except Exception:
        conn.execute("ROLLBACK")
        raise
    finally:
        conn.close()

    return {
        "indexed": indexed,
        "skipped": skipped,
        "elapsed_s": time.time() - t0,
        "rollout_size": rollout_size,
        "from_offset": start_offset,
        "to_offset": last_offset,
    }


def search(index_path: Path, query: str, role: Optional[str] = None,
           limit: int = 50) -> list[dict]:
    """Run FTS5 query, optionally filtered by role. Returns hit dicts."""
    conn = sqlite3.connect(str(index_path))
    try:
        sql = (
            "SELECT timestamp, role, type, "
            "snippet(turns, 3, '[match]', '[/match]', '…', 12) AS snip, "
            "line_offset "
            "FROM turns "
            "WHERE turns MATCH ? "
        )
        params: list = [query]
        if role:
            sql += "AND role = ? "
            params.append(role)
        sql += "ORDER BY timestamp DESC LIMIT ?"
        params.append(limit)
        cur = conn.execute(sql, params)
        return [
            {"timestamp": ts, "role": r, "type": t, "snippet": snip, "offset": off}
            for (ts, r, t, snip, off) in cur.fetchall()
        ]
    finally:
        conn.close()


def main(argv: Optional[list[str]] = None) -> int:
    parser = argparse.ArgumentParser(prog="gcs-rollout-grep")
    parser.add_argument("query", help="FTS5 query (supports AND/OR/NEAR, quotes for phrases)")
    parser.add_argument("--reindex", action="store_true", help="rebuild index from scratch")
    parser.add_argument("--role", choices=["user", "assistant", "developer", ""], default="",
                        help="filter by JSONL payload.role")
    parser.add_argument("--limit", type=int, default=50, help="max hits (default 50)")
    parser.add_argument("--rollout", default="", help="override rollout path")
    parser.add_argument("--build-only", action="store_true",
                        help="just build the index; don't search")
    args = parser.parse_args(argv)

    cfg = load_config()
    if args.rollout:
        rollout = Path(args.rollout)
    else:
        rollout = _resolve_rollout_path(cfg.orchestrator.session_id)
        if rollout is None:
            sys.stderr.write(f"cannot resolve rollout for session {cfg.orchestrator.session_id}\n")
            return 2
    if not rollout.exists():
        sys.stderr.write(f"rollout not found: {rollout}\n")
        return 2

    idx = _index_path_for(rollout, cfg.paths.codex_home)
    needs_index = args.reindex or not idx.exists() or idx.stat().st_size == 0
    if needs_index or args.build_only:
        stats = build_or_update_index(rollout, idx, reindex=args.reindex)
        print(f"[index] indexed={stats['indexed']} skipped={stats['skipped']} "
              f"elapsed={stats['elapsed_s']:.1f}s offset={stats['from_offset']:,}->{stats['to_offset']:,}")
        if args.build_only:
            return 0
    else:
        # Catch up incrementally
        stats = build_or_update_index(rollout, idx, reindex=False)
        if stats["indexed"] > 0:
            print(f"[index] caught up {stats['indexed']} new entries in {stats['elapsed_s']:.2f}s")

    hits = search(idx, args.query, role=args.role or None, limit=args.limit)
    if not hits:
        print("(no hits)")
        return 0

    for h in hits:
        ts = h["timestamp"][:19] if h["timestamp"] else "?"
        snip = h["snippet"].replace("\n", " ")
        print(f"[{ts}] {h['role']:<10} {h['type']:<20} {snip}")
    print(f"\n{len(hits)} hits (index: {idx})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
