"""Tests for the metrics.jsonl module."""
from __future__ import annotations

import json

from gcs_orchestrator import metrics


def test_metrics_append_writes_jsonl(tmp_path):
    metrics.append({"thread_id": "t1", "status": "ok", "total_ms": 1234,
                    "relay_version": "0.0.0-test"}, base=tmp_path)
    p = tmp_path / "metrics.jsonl"
    assert p.exists()
    lines = p.read_text(encoding="utf-8").splitlines()
    assert len(lines) == 1
    d = json.loads(lines[0])
    assert d["thread_id"] == "t1"
    assert d["total_ms"] == 1234
    assert d["schema"] == 1
    assert "ts" in d  # auto-injected


def test_metrics_read_recent_newest_first(tmp_path):
    for i in range(5):
        metrics.append({"thread_id": "t", "status": "ok",
                        "total_ms": i * 100, "relay_version": "0"},
                       base=tmp_path)
    recs = metrics.read_recent(n=3, base=tmp_path)
    assert len(recs) == 3
    # Newest first → last appended first
    assert recs[0]["total_ms"] == 400
    assert recs[1]["total_ms"] == 300
    assert recs[2]["total_ms"] == 200


def test_metrics_percentiles_filters_errors(tmp_path):
    for i, status in enumerate(["ok", "ok", "ok", "ok", "error"]):
        metrics.append({"thread_id": "t", "status": status,
                        "total_ms": 100 + i * 100, "relay_version": "0"},
                       base=tmp_path)
    # Only the 4 "ok" records count
    pcts = metrics.percentiles("total_ms", n=10, ps=(50, 95), base=tmp_path)
    assert pcts[50] is not None
    # p50 of [100,200,300,400] is somewhere around 250
    assert 200 <= pcts[50] <= 300


def test_metrics_percentiles_no_data_returns_none(tmp_path):
    pcts = metrics.percentiles("total_ms", base=tmp_path)
    assert pcts == {50: None, 95: None}


def test_metrics_summary_line_handles_empty(tmp_path):
    assert "no data" in metrics.summary_line(base=tmp_path).lower()


def test_metrics_summary_line_with_data(tmp_path):
    for i in range(5):
        metrics.append({"thread_id": "t", "status": "ok",
                        "total_ms": (i + 1) * 1000, "relay_version": "0"},
                       base=tmp_path)
    line = metrics.summary_line(base=tmp_path)
    assert "p50=" in line
    assert "p95=" in line


def test_metrics_corrupt_line_is_skipped(tmp_path):
    metrics.append({"thread_id": "ok", "status": "ok",
                    "total_ms": 100, "relay_version": "0"}, base=tmp_path)
    p = tmp_path / "metrics.jsonl"
    # Inject a corrupt line
    with p.open("a", encoding="utf-8") as f:
        f.write("{not json\n")
    metrics.append({"thread_id": "ok2", "status": "ok",
                    "total_ms": 200, "relay_version": "0"}, base=tmp_path)
    recs = metrics.read_recent(n=10, base=tmp_path)
    # Corrupt line skipped, valid lines preserved
    assert len(recs) == 2
