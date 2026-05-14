"""Tests for relay receipt key + storage."""
from __future__ import annotations

import json

from gcs_orchestrator.relay import (
    _check_receipt,
    _receipt_key,
    _receipt_path,
    _write_receipt_atomic,
)


def test_receipt_key_is_stable_and_unique():
    a = _receipt_key("session-1", "DONE A")
    b = _receipt_key("session-1", "DONE A")
    c = _receipt_key("session-1", "DONE B")
    d = _receipt_key("session-2", "DONE A")
    assert a == b
    assert a != c
    assert a != d
    assert len(a) == 32  # truncated sha256 hex


def test_pending_receipt_does_not_block_retry(tmp_path):
    key = "abc123"
    _write_receipt_atomic(tmp_path, key, {"status": "pending"})
    assert _check_receipt(tmp_path, key) is False  # pending = not success


def test_success_receipt_blocks_retry(tmp_path):
    key = "def456"
    _write_receipt_atomic(tmp_path, key, {"status": "success", "turn_id": "t-1"})
    assert _check_receipt(tmp_path, key) is True


def test_corrupted_receipt_treated_as_missing(tmp_path):
    key = "ghi789"
    p = _receipt_path(tmp_path, key)
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text("{ corrupted JSON", encoding="utf-8")
    assert _check_receipt(tmp_path, key) is False


def test_receipt_atomic_write(tmp_path):
    """Atomic write should leave no .tmp file behind on success."""
    key = "atomic-test"
    _write_receipt_atomic(tmp_path, key, {"status": "success", "data": 42})
    p = _receipt_path(tmp_path, key)
    assert p.exists()
    assert not p.with_suffix(".receipt.tmp").exists()
    d = json.loads(p.read_text(encoding="utf-8"))
    assert d["data"] == 42
