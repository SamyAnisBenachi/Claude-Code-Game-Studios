"""Tests for the HTTP status sidecar."""
from __future__ import annotations

import json
import socket
import time
import urllib.request

import pytest

from gcs_orchestrator import sidecar


def _free_port() -> int:
    with socket.socket() as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


@pytest.fixture
def running_sidecar():
    port = _free_port()
    server, thread = sidecar.start(supervisor_ref=None, port=port)
    yield port
    server.shutdown()
    server.server_close()


def test_sidecar_version_endpoint_returns_package_version(running_sidecar):
    port = running_sidecar
    with urllib.request.urlopen(f"http://127.0.0.1:{port}/version", timeout=3) as r:
        d = json.loads(r.read())
    assert "package" in d
    assert "python" in d


def test_sidecar_status_endpoint_standalone_mode(running_sidecar):
    port = running_sidecar
    with urllib.request.urlopen(f"http://127.0.0.1:{port}/status", timeout=3) as r:
        d = json.loads(r.read())
    assert d["mode"] == "standalone"
    assert d["uptime_s"] >= 0


def test_sidecar_unknown_path_404(running_sidecar):
    port = running_sidecar
    import urllib.error
    with pytest.raises(urllib.error.HTTPError) as exc_info:
        urllib.request.urlopen(f"http://127.0.0.1:{port}/nope", timeout=3)
    assert exc_info.value.code == 404


def test_sidecar_help_lists_endpoints(running_sidecar):
    port = running_sidecar
    with urllib.request.urlopen(f"http://127.0.0.1:{port}/", timeout=3) as r:
        d = json.loads(r.read())
    assert "/status" in d["endpoints"]
    assert "/metrics" in d["endpoints"]


def test_sidecar_metrics_handles_no_data(running_sidecar):
    """When metrics.jsonl is missing, /metrics should still return 200 with nulls."""
    port = running_sidecar
    with urllib.request.urlopen(f"http://127.0.0.1:{port}/metrics", timeout=3) as r:
        d = json.loads(r.read())
    # Either real data or nulls — never a crash
    assert "p50_total_ms" in d
