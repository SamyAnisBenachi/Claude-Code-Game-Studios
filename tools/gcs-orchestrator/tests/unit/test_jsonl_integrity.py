"""Tests for the JSONL integrity validator + auto-repair."""
from __future__ import annotations

import json

import pytest

from gcs_orchestrator.jsonl_integrity import validate, validate_full, repair


def _make_jsonl(path, lines):
    """Helper: write a JSONL file from a list of dicts."""
    with path.open("w", encoding="utf-8") as f:
        for d in lines:
            f.write(json.dumps(d) + "\n")


def test_jsonl_integrity_validate_well_formed_file_is_ok(tmp_path):
    p = tmp_path / "rollout.jsonl"
    _make_jsonl(p, [{"type": "session_meta"}, {"type": "user", "id": 1}])
    result = validate(p)
    assert result.ok is True
    assert result.last_line_parses is True
    assert result.last_line_complete is True


def test_jsonl_integrity_validate_empty_file_is_not_ok(tmp_path):
    p = tmp_path / "empty.jsonl"
    p.write_text("", encoding="utf-8")
    result = validate(p)
    assert result.ok is False
    assert "empty" in result.error.lower()


def test_jsonl_integrity_validate_missing_trailing_newline_is_not_ok(tmp_path):
    """Torn final write: content but no trailing \\n."""
    p = tmp_path / "torn.jsonl"
    p.write_text('{"type":"a"}\n{"type":"b"', encoding="utf-8")
    result = validate(p)
    assert result.last_line_complete is False
    assert result.ok is False


def test_jsonl_integrity_validate_corrupt_final_line_is_not_ok(tmp_path):
    p = tmp_path / "corrupt.jsonl"
    p.write_text('{"type":"a"}\nnot-json-not-parseable\n', encoding="utf-8")
    result = validate(p)
    # Last line is complete (ends in \n) but doesn't parse
    assert result.last_line_complete is True
    assert result.last_line_parses is False
    assert result.ok is False


def test_jsonl_integrity_validate_full_parses_every_line(tmp_path):
    p = tmp_path / "full-ok.jsonl"
    _make_jsonl(p, [{"id": i} for i in range(20)])
    result = validate_full(p)
    assert result.ok is True
    assert result.line_count == 20


def test_jsonl_integrity_validate_full_detects_mid_file_corruption(tmp_path):
    """validate_full must catch corruption that validate() misses."""
    p = tmp_path / "mid-corrupt.jsonl"
    with p.open("w", encoding="utf-8") as f:
        f.write('{"id":1}\n')
        f.write('{badjson}\n')
        f.write('{"id":3}\n')
    result = validate_full(p)
    assert result.ok is False
    assert result.line_count == 2  # failure on line 2


def test_jsonl_integrity_repair_truncates_to_last_valid_newline(tmp_path):
    """A torn trailing fragment is moved to a .dropped sidecar; file truncated."""
    p = tmp_path / "torn-repair.jsonl"
    p.write_text('{"type":"a"}\n{"type":"b"}\n{"torn fragment without newline', encoding="utf-8")
    pre_size = p.stat().st_size

    rep = repair(p)

    assert rep.repaired is True
    assert rep.truncated_bytes > 0
    assert rep.backup_path is not None
    assert rep.backup_path.exists()
    # File now ends at the last valid newline
    new_size = p.stat().st_size
    assert new_size == pre_size - rep.truncated_bytes
    # Content remaining must validate
    result = validate(p)
    assert result.ok is True


def test_jsonl_integrity_repair_no_op_on_valid_file(tmp_path):
    p = tmp_path / "fine.jsonl"
    _make_jsonl(p, [{"id": 1}])
    rep = repair(p)
    assert rep.repaired is False
    assert rep.truncated_bytes == 0
