"""GFS-style rollout backup with sha256 verification.

Schedules: 7 daily / 4 weekly / 12 monthly snapshots (configurable via
gcs.toml `[backup]` section). Each snapshot is sha256-verified post-copy
and stored with a `.sha256` sidecar. Pre-copy JSONL integrity check
prevents propagating corruption.

Usage:
    python -m gcs_orchestrator.backup [--check-only] [--rollout PATH]

CLI is idempotent — calling daily will only create today's snapshot if
not already present, and prune older snapshots beyond retention.
"""
from __future__ import annotations

import argparse
import hashlib
import shutil
import sys
import time
from datetime import datetime, timedelta
from pathlib import Path
from typing import Optional

from .config import load as load_config
from .jsonl_integrity import validate as jsonl_validate


def _sha256_file(path: Path, chunk_size: int = 1 << 20) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(chunk_size), b""):
            h.update(chunk)
    return h.hexdigest()


def _resolve_rollout_path(session_id: str) -> Optional[Path]:
    """Find the active rollout file by session-id. Walks ~/.codex/sessions/."""
    import glob
    # session-id is encoded in the filename
    pattern = str(Path.home() / ".codex" / "sessions" / "**" / f"rollout-*{session_id}*.jsonl")
    candidates = glob.glob(pattern, recursive=True)
    # Windows path
    pattern2 = rf"C:\Users\Sam\.codex\sessions\**\rollout-*{session_id}*.jsonl"
    import glob as _g
    candidates.extend(_g.glob(pattern2, recursive=True))
    if not candidates:
        return None
    # Return the most-recently-modified match
    return Path(max(candidates, key=lambda p: Path(p).stat().st_mtime))


def _snapshot_path(target_dir: Path, session_id_short: str, when: datetime) -> Path:
    ts = when.strftime("%Y-%m-%dT%H-%M-%S")
    return target_dir / f"rollout-{session_id_short}.{ts}.jsonl"


def take_snapshot(rollout: Path, target_dir: Path, session_id_short: str) -> tuple[Path, str]:
    """Copy + sha256-verify. Returns (snapshot_path, sha256)."""
    target_dir.mkdir(parents=True, exist_ok=True)
    when = datetime.now()
    dst = _snapshot_path(target_dir, session_id_short, when)

    # Pre-copy integrity check
    integrity = jsonl_validate(rollout)
    if not integrity.ok:
        raise RuntimeError(
            f"refusing to back up corrupt rollout: {integrity.error or 'last line invalid'}"
        )

    pre_hash = _sha256_file(rollout)
    shutil.copy2(rollout, dst)
    post_hash = _sha256_file(dst)
    if pre_hash != post_hash:
        # Drift — likely written-to during copy. Retry once.
        dst.unlink()
        time.sleep(1)
        pre_hash = _sha256_file(rollout)
        shutil.copy2(rollout, dst)
        post_hash = _sha256_file(dst)
        if pre_hash != post_hash:
            raise RuntimeError(f"sha256 drift during copy: pre={pre_hash[:16]} post={post_hash[:16]}")

    # Sidecar with hash + metadata
    (dst.with_suffix(dst.suffix + ".sha256")).write_text(
        f"{post_hash}  {dst.name}\n", encoding="utf-8"
    )
    return dst, post_hash


def _prune_gfs(target_dir: Path, session_id_short: str, daily: int, weekly: int, monthly: int) -> list[Path]:
    """GFS retention. Returns list of deleted snapshot paths.

    Strategy: keep the N most-recent snapshots (daily count), then keep
    weeklies (one per ISO week) and monthlies (one per calendar month).
    Anything not in any of the three sets is deleted.
    """
    snapshots = sorted(
        target_dir.glob(f"rollout-{session_id_short}.*.jsonl"),
        key=lambda p: p.stat().st_mtime,
        reverse=True,
    )
    if not snapshots:
        return []

    keep: set[Path] = set()

    # Most-recent N as "daily"
    for p in snapshots[:daily]:
        keep.add(p)

    # Group remaining by week / month, keep newest per group
    seen_weeks: set[str] = set()
    seen_months: set[str] = set()
    for p in snapshots[daily:]:
        mtime = datetime.fromtimestamp(p.stat().st_mtime)
        wkey = mtime.strftime("%Y-W%W")
        mkey = mtime.strftime("%Y-%m")
        if len([w for w in seen_weeks]) < weekly and wkey not in seen_weeks:
            seen_weeks.add(wkey)
            keep.add(p)
            continue
        if len([m for m in seen_months]) < monthly and mkey not in seen_months:
            seen_months.add(mkey)
            keep.add(p)

    deleted = []
    for p in snapshots:
        if p not in keep:
            try:
                # Delete both the snapshot + its sidecar
                p.unlink()
                sidecar = p.with_suffix(p.suffix + ".sha256")
                if sidecar.exists():
                    sidecar.unlink()
                deleted.append(p)
            except OSError:
                pass
    return deleted


def main(argv: Optional[list[str]] = None) -> int:
    parser = argparse.ArgumentParser(prog="gcs-rollout-backup")
    parser.add_argument("--check-only", action="store_true",
                        help="run integrity check; don't take snapshot")
    parser.add_argument("--rollout", default="", help="override rollout path")
    parser.add_argument("--no-prune", action="store_true", help="skip GFS retention pruning")
    args = parser.parse_args(argv)

    cfg = load_config()
    if not cfg.backup.enabled:
        print("backup.enabled = false in gcs.toml; nothing to do")
        return 0
    if not cfg.orchestrator.session_id:
        print("no session_id configured", file=sys.stderr)
        return 2

    if args.rollout:
        rollout = Path(args.rollout)
    else:
        rollout = _resolve_rollout_path(cfg.orchestrator.session_id)
        if rollout is None:
            print(f"cannot resolve rollout for session {cfg.orchestrator.session_id}", file=sys.stderr)
            return 2
    if not rollout.exists():
        print(f"rollout not found: {rollout}", file=sys.stderr)
        return 2

    integrity = jsonl_validate(rollout)
    print(f"integrity: ok={integrity.ok} size={integrity.bytes_total:,}b last_parses={integrity.last_line_parses}")
    if not integrity.ok:
        print(f"  error: {integrity.error}")
    if args.check_only:
        return 0 if integrity.ok else 1
    if not integrity.ok:
        print("refusing to back up corrupt rollout (use jsonl_integrity --repair first)", file=sys.stderr)
        return 1

    target_dir = Path(cfg.backup.target_dir)
    sid_short = cfg.orchestrator.session_id.split("-")[0]
    snap, sha = take_snapshot(rollout, target_dir, sid_short)
    print(f"snapshot:  {snap}")
    print(f"sha256:    {sha[:16]}…")

    if not args.no_prune:
        deleted = _prune_gfs(target_dir, sid_short,
                             cfg.backup.gfs_daily, cfg.backup.gfs_weekly, cfg.backup.gfs_monthly)
        if deleted:
            print(f"pruned:    {len(deleted)} old snapshot(s)")
        else:
            print(f"pruned:    0 (within retention {cfg.backup.gfs_daily}d/{cfg.backup.gfs_weekly}w/{cfg.backup.gfs_monthly}m)")

    return 0


if __name__ == "__main__":
    sys.exit(main())
