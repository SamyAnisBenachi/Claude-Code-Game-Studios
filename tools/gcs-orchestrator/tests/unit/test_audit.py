"""Tests for the dispatcher audit log."""
from __future__ import annotations

import json

from gcs_orchestrator import audit


def test_audit_record_appends_jsonl(tmp_path):
    p = tmp_path / "audit.jsonl"
    did = audit.new_dispatch_id()
    audit.record(dispatch_id=did, decision_type="SPAWN", prompt_id="PROMPT-100",
                 action="POST /api/terminals", octogent_status=201,
                 success=True, latency_ms=143, path=p)
    assert p.exists()
    lines = p.read_text(encoding="utf-8").splitlines()
    assert len(lines) == 1
    d = json.loads(lines[0])
    assert d["dispatch_id"] == did
    assert d["decision_type"] == "SPAWN"
    assert d["prompt_id"] == "PROMPT-100"
    assert d["success"] is True
    assert d["latency_ms"] == 143
    assert "ts" in d


def test_audit_tail_filters_by_dispatch_id(tmp_path):
    p = tmp_path / "audit.jsonl"
    d1 = audit.new_dispatch_id()
    d2 = audit.new_dispatch_id()
    audit.record(dispatch_id=d1, decision_type="SPAWN", prompt_id="A", path=p)
    audit.record(dispatch_id=d2, decision_type="CLEAR", prompt_id="B", path=p)
    audit.record(dispatch_id=d1, decision_type="CLEAR", prompt_id="C", path=p)
    only_d1 = audit.tail(n=10, path=p, dispatch_id=d1)
    assert len(only_d1) == 2
    assert all(r["dispatch_id"] == d1 for r in only_d1)


def test_audit_new_dispatch_id_is_unique():
    a = audit.new_dispatch_id()
    b = audit.new_dispatch_id()
    assert a != b
    assert len(a) == 12


def test_audit_failure_records_with_status(tmp_path):
    p = tmp_path / "audit.jsonl"
    audit.record(dispatch_id="d0", decision_type="SPAWN", prompt_id="X",
                 octogent_status=500, success=False, latency_ms=2000,
                 note="connection refused", path=p)
    recs = audit.tail(n=1, path=p)
    assert recs[0]["success"] is False
    assert recs[0]["octogent_status"] == 500
    assert recs[0]["note"] == "connection refused"
