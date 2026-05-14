"""Idempotency receipts — sha256(session_id||content) keyed JSON files.

Shared between the one-shot relay and the persistent daemon. Both write
identical receipts so:
- `gcs-relay-history` works regardless of which path delivered the turn
- A daemon crash mid-turn doesn't permanently block retry (pending
  receipts are overwritten / deleted on completion)
"""
from __future__ import annotations

import hashlib
import json
import os
import time
from pathlib import Path


def key_for(session_id: str, content: str) -> str:
    """Stable receipt key — sha256 of session_id + NUL + content, truncated to 32 hex."""
    return hashlib.sha256(
        f"{session_id}\0{content}".encode("utf-8")
    ).hexdigest()[:32]


def path_for(base: Path, key: str) -> Path:
    return base / "receipts" / f"{key}.receipt"


def is_success(base: Path, key: str) -> bool:
    """True only if a SUCCESS receipt exists. Pending receipts don't block retry."""
    p = path_for(base, key)
    if not p.exists():
        return False
    try:
        d = json.loads(p.read_text(encoding="utf-8"))
        return d.get("status") == "success"
    except (OSError, json.JSONDecodeError):
        return False


def write_atomic(base: Path, key: str, payload: dict) -> None:
    """Atomic write via tmp + os.replace (Windows-safe)."""
    receipts_dir = base / "receipts"
    receipts_dir.mkdir(parents=True, exist_ok=True)
    p = path_for(base, key)
    tmp = p.with_suffix(".receipt.tmp")
    try:
        tmp.write_text(json.dumps(payload, indent=2), encoding="utf-8")
        os.replace(tmp, p)
    except OSError:
        pass


def delete(base: Path, key: str) -> None:
    """Drop a (pending) receipt so retry isn't blocked."""
    try:
        path_for(base, key).unlink(missing_ok=True)
    except OSError:
        pass


def prune_if_stale(base: Path, ttl_days: int, max_count: int,
                   sentinel_age_s: float = 3600.0) -> None:
    """Lazy prune. Only fires if last prune was older than sentinel_age_s.

    Why lazy: pruning on every call is O(n log n) glob+sort on the hot
    path. Pruning once per hour bounds amortized cost to negligible.
    """
    receipts_dir = base / "receipts"
    if not receipts_dir.exists():
        return
    sentinel = base / ".last_prune_at"
    try:
        if sentinel.exists():
            if time.time() - sentinel.stat().st_mtime < sentinel_age_s:
                return
        receipts_dir.mkdir(parents=True, exist_ok=True)
        files = sorted(receipts_dir.glob("*.receipt"),
                       key=lambda p: p.stat().st_mtime, reverse=True)
        cutoff = time.time() - ttl_days * 86400
        for i, f in enumerate(files):
            try:
                if i >= max_count or f.stat().st_mtime < cutoff:
                    f.unlink(missing_ok=True)
            except OSError:
                pass
        sentinel.touch()
    except OSError:
        pass
