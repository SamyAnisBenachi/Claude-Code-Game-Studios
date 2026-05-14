"""Persistent relay daemon — long-running TCP server on 127.0.0.1:9789.

Replaces the one-shot subprocess relay for the hot path:
- ONE persistent WebSocket connection to app-server (reused across turns)
- Drops per-call WS handshake + initialize (~500ms saved per call)
- In-memory queue serializes turns (app-server only handles one in-flight
  turn per thread anyway)
- Idempotency receipts still written per turn — daemon crash doesn't lose state

Wire protocol: length-prefixed JSON. Each frame =
    [4-byte big-endian length N][N bytes UTF-8 JSON]

Request types:
    {"type":"health"}                                  → {"type":"health_ok",...}
    {"type":"turn","session_id":..,"content":..,...}   → {"type":"turn_result",...}

The TCP server uses ThreadingTCPServer (one thread per client connection)
but all turn execution funnels through a single TurnWorker thread that
owns the persistent transport. This makes the daemon the serialization
point — no FileLock needed in the daemon path.

Backward compat: the spawn_watchdog should probe this socket first; if
unreachable or unresponsive in 1s, fall back to subprocess relay.
"""
from __future__ import annotations

import json
import queue
import socket
import socketserver
import struct
import threading
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Optional

from . import receipts as _receipts
from . import turn_executor as _executor
from .config import GcsConfig, load as load_config
from .transport import AppServerTransport, TransportError


_FRAME_MAX_BYTES = 10 * 1024 * 1024  # 10 MB cap per frame


@dataclass
class TurnRequest:
    session_id: str
    content: str
    worker_id: Optional[str]
    timeout_s: float
    # Response channel — TCP handler thread fills this from worker
    response: Any  # threading.Event-backed (set value via .set())
    result: "dict[str, Any]"


def _read_frame(conn: socket.socket) -> Optional[dict]:
    try:
        hdr = b""
        while len(hdr) < 4:
            chunk = conn.recv(4 - len(hdr))
            if not chunk:
                return None
            hdr += chunk
        (length,) = struct.unpack(">I", hdr)
        if length > _FRAME_MAX_BYTES:
            return None
        buf = b""
        while len(buf) < length:
            chunk = conn.recv(length - len(buf))
            if not chunk:
                return None
            buf += chunk
        return json.loads(buf.decode("utf-8"))
    except (OSError, struct.error, json.JSONDecodeError, UnicodeDecodeError):
        return None


def _write_frame(conn: socket.socket, payload: dict) -> None:
    raw = json.dumps(payload, ensure_ascii=False).encode("utf-8")
    conn.sendall(struct.pack(">I", len(raw)) + raw)


class TurnWorker(threading.Thread):
    """Single thread that owns the persistent transport and processes the queue."""

    QUEUE_CAP = 50

    def __init__(self, cfg: GcsConfig, log) -> None:
        super().__init__(daemon=True, name="TurnWorker")
        self.cfg = cfg
        self.log = log
        self.q: "queue.Queue[TurnRequest]" = queue.Queue(maxsize=self.QUEUE_CAP)
        self._stop = threading.Event()
        self._transport: Optional[AppServerTransport] = None
        self._connect_metrics: dict = {}
        self._connected_since: Optional[float] = None
        self._lock = threading.Lock()

    def submit(self, req: TurnRequest) -> bool:
        """Returns False if queue is full."""
        try:
            self.q.put_nowait(req)
            return True
        except queue.Full:
            return False

    def queue_depth(self) -> int:
        return self.q.qsize()

    def is_ws_connected(self) -> bool:
        with self._lock:
            return self._transport is not None and self._transport.is_connected()

    def stop(self) -> None:
        self._stop.set()
        # Wake the queue.get if waiting
        try:
            self.q.put_nowait(None)  # type: ignore[arg-type]
        except queue.Full:
            pass

    def request_reconnect(self) -> None:
        """Force a transport reconnect on next idle loop."""
        with self._lock:
            if self._transport is not None:
                try:
                    self._transport.close()
                except Exception:
                    pass
                self._transport = None

    def _ensure_connected(self) -> bool:
        with self._lock:
            if self._transport is not None and self._transport.is_connected():
                return True
            # (re)connect
            try:
                self._transport, self._connect_metrics = _executor.connect_and_initialize(self.cfg)
                self._connected_since = time.time()
                self.log("INFO", f"daemon transport connected "
                                  f"ws_connect_ms={self._connect_metrics['ws_connect_ms']} "
                                  f"initialize_ms={self._connect_metrics['initialize_ms']}")
                return True
            except Exception as exc:
                self.log("ERROR", f"daemon transport connect failed: {exc!r}")
                self._transport = None
                return False

    def run(self) -> None:
        backoff = 2.0
        while not self._stop.is_set():
            if not self._ensure_connected():
                time.sleep(min(backoff, 60.0))
                backoff = min(backoff * 2, 60.0)
                continue
            backoff = 2.0
            try:
                req = self.q.get(timeout=5.0)
            except queue.Empty:
                continue
            if req is None or self._stop.is_set():
                break

            try:
                # Get the transport reference under lock then operate on it
                # outside the lock (long-running call)
                with self._lock:
                    transport = self._transport
                    cm = self._connect_metrics
                if transport is None:
                    req.result.update({"exit_code": _executor.EXIT_APP_SERVER_ERROR,
                                       "error_message": "transport unavailable"})
                    req.response.set()
                    continue

                # NOTE: only pass connect_metrics on the FIRST turn after reconnect
                # (sentinel: cm contains values; we zero them after first use).
                first_after_connect = bool(cm.get("ws_connect_ms")) or bool(cm.get("initialize_ms"))
                cm_to_pass = cm if first_after_connect else {"ws_connect_ms": 0, "initialize_ms": 0}
                if first_after_connect:
                    with self._lock:
                        self._connect_metrics = {"ws_connect_ms": 0, "initialize_ms": 0}

                exec_result = _executor.run_turn(
                    self.cfg, transport, req.session_id, req.content,
                    worker_id=req.worker_id,
                    lock_wait_ms=0,  # daemon serializes via queue, no lock
                    connect_metrics=cm_to_pass,
                    log=self.log,
                )

                req.result.update({
                    "exit_code": exec_result.exit_code,
                    "agent_text": exec_result.agent_text,
                    "turn_id": exec_result.turn_id,
                    "elapsed_s": exec_result.elapsed_s,
                    "error_message": exec_result.error_message,
                })
                req.response.set()

                # Detect transport death — if exit_code is APP_SERVER_ERROR
                # and transport reports disconnected, drop the reference so
                # the next loop reconnects.
                if exec_result.exit_code == _executor.EXIT_APP_SERVER_ERROR:
                    if not transport.is_connected():
                        with self._lock:
                            self._transport = None
                        self.log("WARN", "daemon transport lost, will reconnect")
            except Exception as exc:
                self.log("ERROR", f"daemon worker exception: {exc!r}")
                req.result.update({"exit_code": _executor.EXIT_APP_SERVER_ERROR,
                                   "error_message": str(exc)})
                req.response.set()


class _Handler(socketserver.BaseRequestHandler):
    """Per-connection thread: read one request, enqueue, write response."""

    def handle(self) -> None:
        server: "DaemonServer" = self.server  # type: ignore[assignment]
        conn = self.request
        try:
            conn.settimeout(server.cfg.transport.turn_timeout_s + 10)
            msg = _read_frame(conn)
            if msg is None:
                return
            typ = msg.get("type", "")
            if typ == "health":
                _write_frame(conn, {
                    "type": "health_ok",
                    "ws_connected": server.worker.is_ws_connected(),
                    "queue_depth": server.worker.queue_depth(),
                    "version": _executor._relay_version(),
                })
                return

            if typ != "turn":
                _write_frame(conn, {"type": "error", "message": f"unknown type: {typ}"})
                return

            session_id = msg.get("session_id") or ""
            content = msg.get("content") or ""
            worker_id = msg.get("worker_id")
            timeout_s = float(msg.get("timeout_s") or server.cfg.transport.turn_timeout_s)

            if not session_id or not content.strip():
                _write_frame(conn, {"type": "error", "message": "missing session_id or content"})
                return

            # Pre-check idempotency receipt for fast path
            base = server.cfg.relay_base_dir()
            key = _receipts.key_for(session_id, content)
            if _receipts.is_success(base, key):
                _write_frame(conn, {
                    "type": "turn_result",
                    "exit_code": _executor.EXIT_DUPLICATE,
                    "agent_text": "",
                    "turn_id": "",
                    "elapsed_s": 0.0,
                    "error_message": "",
                })
                return

            req = TurnRequest(
                session_id=session_id, content=content,
                worker_id=worker_id, timeout_s=timeout_s,
                response=threading.Event(), result={},
            )
            ok = server.worker.submit(req)
            if not ok:
                _write_frame(conn, {
                    "type": "turn_result",
                    "exit_code": _executor.EXIT_LOCK_TIMEOUT,
                    "agent_text": "",
                    "turn_id": "",
                    "elapsed_s": 0.0,
                    "error_message": "daemon queue full",
                })
                return

            # Block until worker fills result, with a generous timeout
            if not req.response.wait(timeout=timeout_s + 30):
                _write_frame(conn, {
                    "type": "turn_result",
                    "exit_code": _executor.EXIT_TIMEOUT,
                    "agent_text": "",
                    "turn_id": "",
                    "elapsed_s": 0.0,
                    "error_message": "daemon worker did not respond",
                })
                return

            _write_frame(conn, {"type": "turn_result", **req.result})
        except Exception as exc:
            try:
                _write_frame(conn, {"type": "error", "message": repr(exc)})
            except Exception:
                pass


class DaemonServer(socketserver.ThreadingTCPServer):
    allow_reuse_address = True
    daemon_threads = True

    def __init__(self, host: str, port: int, cfg: GcsConfig, log) -> None:
        super().__init__((host, port), _Handler)
        self.cfg = cfg
        self.log = log
        self.worker = TurnWorker(cfg, log)


def serve(host: str = "127.0.0.1", port: int = 9789,
          cfg: Optional[GcsConfig] = None, log = None) -> None:
    """Run the daemon. Blocks until KeyboardInterrupt or socket bind error."""
    cfg = cfg or load_config()
    if log is None:
        def log(level: str, msg: str) -> None:
            ts = time.strftime("%Y-%m-%d %H:%M:%S")
            try:
                base = cfg.relay_base_dir()
                base.mkdir(parents=True, exist_ok=True)
                with (base / "daemon.log").open("a", encoding="utf-8") as fh:
                    fh.write(f"[{ts}] {level} {msg}\n")
            except OSError:
                pass

    server = DaemonServer(host, port, cfg, log)
    server.worker.start()
    log("INFO", f"relay daemon listening on {host}:{port}")
    try:
        server.serve_forever()
    finally:
        server.worker.stop()
        server.server_close()


# ---- Client API (called from spawn_watchdog) ----

def client_health(host: str = "127.0.0.1", port: int = 9789,
                  timeout_s: float = 1.0) -> Optional[dict]:
    """Probe the daemon. Returns the health dict or None if unreachable."""
    try:
        with socket.create_connection((host, port), timeout=timeout_s) as conn:
            _write_frame(conn, {"type": "health"})
            conn.settimeout(timeout_s)
            return _read_frame(conn)
    except (OSError, socket.timeout):
        return None


def client_turn(session_id: str, content: str, *,
                worker_id: Optional[str] = None,
                timeout_s: float = 600.0,
                host: str = "127.0.0.1", port: int = 9789) -> dict:
    """Synchronous turn submission. Returns the turn_result dict.

    Caller is responsible for falling back to subprocess relay if this
    raises ConnectionError or returns exit_code that maps to a transport
    issue.
    """
    with socket.create_connection((host, port), timeout=5.0) as conn:
        _write_frame(conn, {
            "type": "turn",
            "session_id": session_id,
            "content": content,
            "worker_id": worker_id,
            "timeout_s": timeout_s,
        })
        # Generous read timeout: turn_timeout + buffer
        conn.settimeout(timeout_s + 60)
        resp = _read_frame(conn)
        if resp is None:
            raise ConnectionError("daemon closed connection mid-request")
        return resp


def main(argv: Optional[list[str]] = None) -> int:
    import argparse
    parser = argparse.ArgumentParser(prog="gcs-relay-daemon")
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=9789)
    args = parser.parse_args(argv)
    serve(host=args.host, port=args.port)
    return 0


if __name__ == "__main__":
    import sys
    sys.exit(main())
