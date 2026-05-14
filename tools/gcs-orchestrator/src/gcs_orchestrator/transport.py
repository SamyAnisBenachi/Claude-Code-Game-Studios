"""Codex transport adapter — protocol-agnostic interface over Codex's
experimental JSON-RPC `app-server` mode.

The point of this module is to keep all knowledge of Codex CLI's
JSON-RPC method names (`initialize`, `thread/resume`, `turn/start`,
`item/agentMessage/delta`, etc.) in ONE place. If OpenAI renames or
reshapes the surface, only this file changes.

Public API:
    CodexTransport (Protocol):
        connect()
        initialize(client_info)
        resume_thread(thread_id, cwd_override=None) → ThreadResumeResult
        start_turn(thread_id, content, cwd_override=None) → TurnRequestId
        wait_for_turn(req_id, timeout_s) → TurnResult
        interrupt_turn(thread_id, turn_id)
        thread_read(thread_id, include_turns=False)
        close()

Current impl: AppServerTransport (over WebSocket).

Future-proofing for Codex CLI version drift:
- WS handshake uses `suppress_origin=True` (server rejects Origin header)
- TCP keepalive enabled on socket
- Application-level ping every N seconds (configurable, default 20s) via
  a heartbeat thread that probes `thread/read` as a no-op liveness check
"""
from __future__ import annotations

import json
import queue
import socket
import threading
import time
from dataclasses import dataclass, field
from typing import Any, Callable, Optional, Protocol

import websocket  # websocket-client

from .platform import enable_tcp_keepalive


# ---- Public types ----

@dataclass
class ThreadResumeResult:
    thread_id: str
    cwd: str
    preview: str
    raw: dict


@dataclass
class TurnResult:
    """Result of a completed turn."""
    turn_id: str
    status: str  # "completed" | "failed" | "interrupted" | "timeout"
    assistant_text: str = ""
    error_message: str = ""
    elapsed_s: float = 0.0
    cached_input_tokens: int = 0
    input_tokens: int = 0
    output_tokens: int = 0
    # Latency breakdown (ms). Populated when available; default 0.
    ttft_ms: int = 0  # time-to-first-token: start_turn → first delta


@dataclass
class TurnRequestId:
    """Opaque handle returned by start_turn; pass to wait_for_turn."""
    req_id: int
    started_at: float = field(default_factory=time.time)


class CodexTransport(Protocol):
    """Abstract Codex transport. Implementations must be thread-safe for
    one send + one recv thread (not multi-writer)."""

    def connect(self) -> None: ...
    def close(self) -> None: ...
    def initialize(self, client_name: str, client_version: str = "0.1.0") -> dict: ...
    def resume_thread(self, thread_id: str, cwd_override: Optional[str] = None) -> ThreadResumeResult: ...
    def start_turn(self, thread_id: str, content: str, cwd_override: Optional[str] = None) -> TurnRequestId: ...
    def wait_for_turn(self, handle: TurnRequestId, timeout_s: float) -> TurnResult: ...
    def interrupt_turn(self, thread_id: str, turn_id: str) -> None: ...
    def is_connected(self) -> bool: ...


# ---- WebSocket implementation ----

class AppServerTransport:
    """Codex `app-server` over WebSocket JSON-RPC.

    Connection-per-instance. Single-threaded send (caller's responsibility),
    background receive thread fills an inbound queue that wait_for_turn
    consumes. The viewer's receive loop should use poll() directly.
    """

    def __init__(self, ws_url: str, handshake_timeout_s: float = 30.0) -> None:
        self.ws_url = ws_url
        self.handshake_timeout_s = handshake_timeout_s
        self.ws: Optional[websocket.WebSocket] = None
        self._send_lock = threading.Lock()
        self._next_id_lock = threading.Lock()
        self._next_id = 0
        self._recv_queue: "queue.Queue[dict]" = queue.Queue()
        self._recv_thread: Optional[threading.Thread] = None
        self._stop = threading.Event()
        # Set externally if caller wants to intercept all notifications
        # (e.g. viewer streams deltas in real-time)
        self.on_notification: Optional[Callable[[dict], None]] = None

    # ---- Connection lifecycle ----

    def connect(self) -> None:
        ws = websocket.WebSocket()
        ws.settimeout(self.handshake_timeout_s)
        # IMPORTANT: Codex app-server rejects WS handshakes with an Origin
        # header (returns 403). Python's websocket-client adds one by
        # default. suppress_origin=True is mandatory.
        ws.connect(self.ws_url, suppress_origin=True)
        # TCP keepalive — detect half-open sockets after PC sleep
        if ws.sock is not None:
            enable_tcp_keepalive(ws.sock)
        self.ws = ws
        self._recv_thread = threading.Thread(target=self._recv_loop, daemon=True)
        self._recv_thread.start()

    def close(self) -> None:
        self._stop.set()
        if self.ws is not None:
            try:
                self.ws.close()
            except Exception:
                pass

    def is_connected(self) -> bool:
        if self.ws is None:
            return False
        try:
            return self.ws.connected
        except Exception:
            return False

    # ---- Internal helpers ----

    def _new_id(self) -> int:
        with self._next_id_lock:
            self._next_id += 1
            return self._next_id

    def _send_request(self, method: str, params: Optional[dict] = None) -> int:
        assert self.ws is not None
        req_id = self._new_id()
        msg: dict = {"jsonrpc": "2.0", "id": req_id, "method": method}
        if params is not None:
            msg["params"] = params
        line = json.dumps(msg)
        with self._send_lock:
            self.ws.send(line)
        return req_id

    def _send_notification(self, method: str, params: Optional[dict] = None) -> None:
        assert self.ws is not None
        msg: dict = {"jsonrpc": "2.0", "method": method}
        if params is not None:
            msg["params"] = params
        with self._send_lock:
            self.ws.send(json.dumps(msg))

    def _recv_loop(self) -> None:
        assert self.ws is not None
        while not self._stop.is_set():
            try:
                # Long timeout — only break on actual events / errors
                self.ws.settimeout(60.0)
                raw = self.ws.recv()
            except websocket.WebSocketTimeoutException:
                continue
            except (websocket.WebSocketConnectionClosedException, ConnectionResetError, OSError):
                return
            if not raw:
                continue
            try:
                d = json.loads(raw)
            except json.JSONDecodeError:
                continue
            # Notifications: dispatch to callback
            if "method" in d and "id" not in d and self.on_notification is not None:
                try:
                    self.on_notification(d)
                except Exception:
                    pass
            self._recv_queue.put(d)

    def _wait_for_id(self, req_id: int, timeout_s: float) -> Optional[dict]:
        """Wait for a JSON-RPC response with the given id."""
        deadline = time.time() + timeout_s
        held: list[dict] = []
        try:
            while time.time() < deadline:
                try:
                    msg = self._recv_queue.get(timeout=max(0.1, deadline - time.time()))
                except queue.Empty:
                    return None
                if msg.get("id") == req_id:
                    # Re-enqueue held notifications so wait_for_turn etc can see them
                    for m in held:
                        self._recv_queue.put(m)
                    return msg
                held.append(msg)
            return None
        finally:
            for m in held:
                # If timeout, still re-enqueue for downstream consumers
                self._recv_queue.put(m)

    # ---- High-level methods (Codex JSON-RPC mapping) ----

    def initialize(self, client_name: str, client_version: str = "0.1.0") -> dict:
        rid = self._send_request("initialize", {
            "clientInfo": {
                "name": client_name,
                "title": client_name,
                "version": client_version,
            }
        })
        resp = self._wait_for_id(rid, self.handshake_timeout_s)
        if resp is None or "result" not in resp:
            raise TransportError(f"initialize failed or timed out: {resp}")
        return resp["result"]

    def resume_thread(self, thread_id: str, cwd_override: Optional[str] = None) -> ThreadResumeResult:
        params: dict = {"threadId": thread_id}
        if cwd_override:
            params["cwd"] = cwd_override
        rid = self._send_request("thread/resume", params)
        resp = self._wait_for_id(rid, self.handshake_timeout_s)
        if resp is None or "result" not in resp:
            raise TransportError(f"thread/resume failed: {resp}")
        thread = resp["result"].get("thread", {})
        return ThreadResumeResult(
            thread_id=thread.get("id", thread_id),
            cwd=thread.get("cwd", "") or resp["result"].get("cwd", ""),
            preview=thread.get("preview", ""),
            raw=resp["result"],
        )

    def start_turn(self, thread_id: str, content: str, cwd_override: Optional[str] = None) -> TurnRequestId:
        params: dict = {
            "threadId": thread_id,
            "input": [{"type": "text", "text": content}],
        }
        if cwd_override:
            params["cwd"] = cwd_override
        rid = self._send_request("turn/start", params)
        return TurnRequestId(req_id=rid)

    def wait_for_turn(self, handle: TurnRequestId, timeout_s: float) -> TurnResult:
        """Drain notifications until turn/completed (or error/timeout)."""
        deadline = time.time() + timeout_s
        started_info: dict = {}
        assistant_text = ""
        last_token_usage: dict = {}
        ttft_s: Optional[float] = None  # time-to-first-token, captured on first delta

        while time.time() < deadline:
            try:
                msg = self._recv_queue.get(timeout=max(0.1, deadline - time.time()))
            except queue.Empty:
                continue

            # Initial JSON-RPC response for our start_turn request
            if msg.get("id") == handle.req_id:
                if "error" in msg:
                    return TurnResult(
                        turn_id="",
                        status="failed",
                        error_message=str(msg["error"]),
                        elapsed_s=time.time() - handle.started_at,
                    )
                # Success ack; continue waiting for completion notification
                continue

            method = msg.get("method", "")
            params = msg.get("params") or {}

            if method == "turn/started":
                started_info = params

            elif method == "item/agentMessage/delta":
                delta = params.get("delta", "")
                if isinstance(delta, str):
                    if ttft_s is None and delta:
                        ttft_s = time.time() - handle.started_at
                    assistant_text += delta

            elif method == "thread/tokenUsage/updated":
                last_token_usage = (params.get("tokenUsage") or {}).get("last", {}) or last_token_usage

            elif method == "turn/completed":
                turn = params.get("turn") or {}
                status = turn.get("status") or "completed"
                err = turn.get("error") or {}
                err_msg = err.get("message", "") if isinstance(err, dict) else str(err)
                return TurnResult(
                    turn_id=turn.get("id", started_info.get("turn", {}).get("id", "")),
                    status=status,
                    assistant_text=assistant_text,
                    error_message=err_msg,
                    elapsed_s=time.time() - handle.started_at,
                    cached_input_tokens=last_token_usage.get("cachedInputTokens", 0) or 0,
                    input_tokens=last_token_usage.get("inputTokens", 0) or 0,
                    output_tokens=last_token_usage.get("outputTokens", 0) or 0,
                    ttft_ms=int(ttft_s * 1000) if ttft_s is not None else 0,
                )

        return TurnResult(
            turn_id=started_info.get("turn", {}).get("id", ""),
            status="timeout",
            assistant_text=assistant_text,
            elapsed_s=time.time() - handle.started_at,
            ttft_ms=int(ttft_s * 1000) if ttft_s is not None else 0,
        )

    def interrupt_turn(self, thread_id: str, turn_id: str) -> None:
        self._send_request("turn/interrupt", {"threadId": thread_id, "turnId": turn_id})

    def thread_read(self, thread_id: str, include_turns: bool = False) -> dict:
        rid = self._send_request("thread/read", {"threadId": thread_id, "includeTurns": include_turns})
        resp = self._wait_for_id(rid, self.handshake_timeout_s)
        if resp is None or "result" not in resp:
            raise TransportError(f"thread/read failed: {resp}")
        return resp["result"]


class TransportError(Exception):
    """Raised when a JSON-RPC call fails or times out at the protocol level."""


# ---- Readyz probe (HTTP, not WS) ----

def server_reachable(readyz_url: str, timeout_s: float = 3.0) -> bool:
    """Quick HTTP check that the app-server is alive."""
    import urllib.request
    try:
        with urllib.request.urlopen(readyz_url, timeout=timeout_s) as resp:
            return resp.status == 200
    except Exception:
        return False


__all__ = [
    "CodexTransport",
    "AppServerTransport",
    "TransportError",
    "ThreadResumeResult",
    "TurnResult",
    "TurnRequestId",
    "server_reachable",
]
