"""Tests for relay receipt key + storage."""
from __future__ import annotations

import json

from gcs_orchestrator.receipts import (
    is_success,
    key_for,
    path_for,
    write_atomic,
)


def test_receipt_key_is_stable_and_unique():
    a = key_for("session-1", "DONE A")
    b = key_for("session-1", "DONE A")
    c = key_for("session-1", "DONE B")
    d = key_for("session-2", "DONE A")
    assert a == b
    assert a != c
    assert a != d
    assert len(a) == 32  # truncated sha256 hex


def test_pending_receipt_does_not_block_retry(tmp_path):
    key = "abc123"
    write_atomic(tmp_path, key, {"status": "pending"})
    assert is_success(tmp_path, key) is False  # pending = not success


def test_success_receipt_blocks_retry(tmp_path):
    key = "def456"
    write_atomic(tmp_path, key, {"status": "success", "turn_id": "t-1"})
    assert is_success(tmp_path, key) is True


def test_corrupted_receipt_treated_as_missing(tmp_path):
    key = "ghi789"
    p = path_for(tmp_path, key)
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text("{ corrupted JSON", encoding="utf-8")
    assert is_success(tmp_path, key) is False


def test_receipt_atomic_write(tmp_path):
    """Atomic write should leave no .tmp file behind on success."""
    key = "atomic-test"
    write_atomic(tmp_path, key, {"status": "success", "data": 42})
    p = path_for(tmp_path, key)
    assert p.exists()
    assert not p.with_suffix(".receipt.tmp").exists()
    d = json.loads(p.read_text(encoding="utf-8"))
    assert d["data"] == 42
