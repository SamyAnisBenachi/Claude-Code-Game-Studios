"""HTTP status sidecar — read-only operational endpoints on 127.0.0.1:9788.

Co-located with `gcs-app-supervisor` as a sibling thread so it shares
process state (PID, last probe ts, backoff counter) without IPC. Stdlib
http.server only — no FastAPI/uvicorn footprint.

Endpoints:
    GET /status      → orchestrator health summary (JSON)
    GET /last-turn   → latest assistant message preview (JSON)
    GET /queue       → Octogent terminals filtered to gcs-orchestrator (JSON)
    GET /version     → package + codex CLI versions (JSON)
    GET /metrics     → p50/p95 over last 20 turns (JSON)

Security: bind 127.0.0.1 only. No content body exposure — only previews
and counts. No /receipts /rollout /backup paths.
"""
from __future__ import annotations

import json
import threading
import time
import urllib.error
import urllib.request
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Any, Callable, Optional


# Module-global injected by start(): the supervisor reference so /status
# can read in-proc state. None when sidecar runs standalone.
_SUPERVISOR_REF: Any = None


# Small TTL cache for endpoints that hit external resources
class _TTLCache:
    def __init__(self, ttl_s: float) -> None:
        self.ttl_s = ttl_s
        self._value: Any = None
        self._fetched_at: float = 0
        self._lock = threading.Lock()

    def get_or_fetch(self, fetch: Callable[[], Any]) -> Any:
        with self._lock:
            now = time.time()
            if self._value is not None and now - self._fetched_at < self.ttl_s:
                return self._value
            try:
                self._value = fetch()
                self._fetched_at = now
            except Exception as exc:
                self._value = {"error": repr(exc)}
                self._fetched_at = now
            return self._value


_LAST_TURN_CACHE = _TTLCache(ttl_s=2.0)
_QUEUE_CACHE = _TTLCache(ttl_s=5.0)
_VERSION_CACHE = _TTLCache(ttl_s=60.0)


def _get_version() -> dict:
    out: dict = {"package": "0.0.0", "python": ""}
    try:
        from importlib.metadata import version
        out["package"] = version("gcs-orchestrator")
    except Exception:
        pass
    import sys
    out["python"] = sys.version.split()[0]
    # codex CLI — try `codex --version` (best effort)
    try:
        import subprocess
        proc = subprocess.run(["codex", "--version"], capture_output=True, text=True,
                              timeout=3, creationflags=getattr(subprocess, "CREATE_NO_WINDOW", 0))
        if proc.returncode == 0:
            out["codex_cli"] = proc.stdout.strip()
    except Exception:
        pass
    return out


def _get_last_turn(cfg) -> dict:
    """Read latest assistant text from the rollout JSONL.

    Returns a small dict — never the full agent text (privacy + size).
    """
    from .backup import _resolve_rollout_path
    from .jsonl_integrity import validate

    rollout = _resolve_rollout_path(cfg.orchestrator.session_id)
    if rollout is None or not rollout.exists():
        return {"error": "rollout not found"}

    val = validate(rollout)
    if not val.ok:
        return {"error": f"rollout integrity: {val.error}",
                "rollout_size": rollout.stat().st_size}

    # Tail-scan up to last 200 KB for the last assistant message
    try:
        size = rollout.stat().st_size
        with rollout.open("rb") as f:
            f.seek(max(0, size - 200_000))
            tail = f.read().decode("utf-8", errors="replace")
    except OSError as exc:
        return {"error": repr(exc)}

    last_ts = None
    last_preview = ""
    for line in reversed(tail.splitlines()):
        try:
            d = json.loads(line)
        except json.JSONDecodeError:
            continue
        if d.get("type") != "response_item":
            continue
        payload = d.get("payload") or {}
        if payload.get("role") != "assistant" or payload.get("type") != "message":
            continue
        chunks = []
        for piece in payload.get("content") or []:
            if isinstance(piece, dict) and piece.get("type") == "output_text":
                chunks.append(piece.get("text") or "")
        if chunks:
            last_ts = d.get("timestamp", "")
            last_preview = "\n".join(chunks)[:500]
            break

    return {
        "rollout_size": rollout.stat().st_size,
        "last_assistant_ts": last_ts,
        "preview": last_preview,
        "preview_truncated": last_preview != "" and len(last_preview) == 500,
    }


def _get_queue() -> dict:
    """Filter Octogent's terminal snapshots to gcs-orchestrator tentacle."""
    try:
        resp = urllib.request.urlopen(
            "http://127.0.0.1:8787/api/terminal-snapshots", timeout=3,
        ).read()
        data = json.loads(resp)
    except (urllib.error.URLError, OSError, json.JSONDecodeError):
        return {"error": "octogent unreachable"}
    out: list[dict] = []
    for t in data:
        if t.get("tentacleId") not in ("gcs-orchestrator", "octogent-channel"):
            continue
        out.append({
            "terminalId": t.get("terminalId"),
            "state": t.get("lifecycleState"),
            "agent": t.get("agentRuntimeState"),
            "createdAt": t.get("createdAt"),
        })
    return {"count": len(out), "terminals": out}


def _get_status() -> dict:
    sup = _SUPERVISOR_REF
    base: dict = {
        "uptime_s": int(time.time() - _STARTED_AT) if _STARTED_AT else 0,
    }
    if sup is None:
        base["mode"] = "standalone"
        return base
    base["mode"] = "supervised"
    base["app_server_pid"] = sup._proc.pid if sup._proc else None  # type: ignore[attr-defined]
    base["backoff_s"] = sup._backoff_s  # type: ignore[attr-defined]
    base["ws_url"] = sup.ws_url
    base["readyz_url"] = sup.readyz_url
    # Daemon state, if running
    daemon_server = getattr(sup, "_daemon_server", None)
    if daemon_server is not None:
        try:
            worker = daemon_server.worker  # type: ignore[attr-defined]
            base["daemon"] = {
                "host": sup.daemon_host,
                "port": sup.daemon_port,
                "ws_connected": worker.is_ws_connected(),
                "queue_depth": worker.queue_depth(),
            }
        except Exception:
            base["daemon"] = {"error": "introspection failed"}
    return base


def _get_metrics() -> dict:
    from . import metrics as _metrics
    pcts = _metrics.percentiles("total_ms", n=20, ps=(50, 95))
    ttft = _metrics.percentiles("ttft_ms", n=20, ps=(50, 95))
    recent = _metrics.read_recent(n=1)
    return {
        "p50_total_ms": pcts.get(50),
        "p95_total_ms": pcts.get(95),
        "p50_ttft_ms": ttft.get(50),
        "p95_ttft_ms": ttft.get(95),
        "last_record_ts": recent[0].get("ts") if recent else None,
    }


_STARTED_AT: Optional[float] = None
_CFG = None  # populated by start()


class _StatusHandler(BaseHTTPRequestHandler):
    def log_message(self, fmt: str, *args: Any) -> None:
        # Suppress noisy stdout — log to supervisor.log via _SUPERVISOR_REF if present
        if _SUPERVISOR_REF is not None:
            try:
                _SUPERVISOR_REF.log("SIDECAR", f"{self.address_string()} {fmt % args}")
            except Exception:
                pass

    def _send_json(self, code: int, payload: dict) -> None:
        body = json.dumps(payload, ensure_ascii=False, indent=2).encode("utf-8")
        self.send_response(code)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Cache-Control", "no-store")
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self) -> None:  # noqa: N802
        if self.path == "/status":
            self._send_json(200, _get_status())
        elif self.path == "/last-turn":
            self._send_json(200, _LAST_TURN_CACHE.get_or_fetch(lambda: _get_last_turn(_CFG)))
        elif self.path == "/queue":
            self._send_json(200, _QUEUE_CACHE.get_or_fetch(_get_queue))
        elif self.path == "/version":
            self._send_json(200, _VERSION_CACHE.get_or_fetch(_get_version))
        elif self.path == "/metrics":
            self._send_json(200, _get_metrics())
        elif self.path in ("/", "/help"):
            self._send_json(200, {
                "endpoints": ["/status", "/last-turn", "/queue", "/version", "/metrics"],
                "note": "127.0.0.1 only; read-only; no auth"
            })
        else:
            self._send_json(404, {"error": "not found", "path": self.path})


def start(supervisor_ref=None, host: str = "127.0.0.1", port: int = 9788,
          cfg=None) -> tuple[ThreadingHTTPServer, threading.Thread]:
    """Start the sidecar in a background thread. Returns (server, thread)."""
    global _SUPERVISOR_REF, _STARTED_AT, _CFG
    _SUPERVISOR_REF = supervisor_ref
    _STARTED_AT = time.time()
    if cfg is None:
        from .config import load as load_config
        cfg = load_config()
    _CFG = cfg

    server = ThreadingHTTPServer((host, port), _StatusHandler)
    server.daemon_threads = True
    thread = threading.Thread(target=server.serve_forever, daemon=True,
                              name="StatusSidecar")
    thread.start()
    return server, thread


def main(argv: Optional[list[str]] = None) -> int:
    """Standalone CLI (sidecar without supervisor)."""
    import argparse
    parser = argparse.ArgumentParser(prog="gcs-app-status")
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=9788)
    args = parser.parse_args(argv)
    server, _ = start(host=args.host, port=args.port)
    print(f"status sidecar on http://{args.host}:{args.port}/")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    return 0
