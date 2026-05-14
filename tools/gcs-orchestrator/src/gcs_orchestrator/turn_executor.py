"""Turn lifecycle executor — shared between one-shot relay and persistent daemon.

Given a connected transport + session_id + content, this runs ONE turn:
- resume thread
- start turn
- write PENDING receipt (so a mid-turn crash doesn't block retry)
- wait for turn/completed
- write SUCCESS or delete pending receipt on failure
- emit metrics record

Returns (exit_code, agent_text, metrics).

The DAEMON path: connects ONCE up-front, runs many `run_turn()`s serially,
metrics still captured per turn. Receipts still written per turn.

The ONE-SHOT path: a fresh transport per call (legacy behavior).
"""
from __future__ import annotations

import time
from dataclasses import dataclass
from pathlib import Path
from typing import Optional

from . import receipts as _receipts
from . import metrics as _metrics
from .config import GcsConfig
from .transport import (
    AppServerTransport,
    CodexTransport,
    TransportError,
    TurnResult,
)

# Exit codes mirror relay.py (single source of truth still in relay.py to
# avoid circular imports, but re-exported here as constants for daemon use).
EXIT_OK = 0
EXIT_DUPLICATE = 2
EXIT_LOCK_TIMEOUT = 3
EXIT_APP_SERVER_ERROR = 4
EXIT_TURN_ABORTED = 5
EXIT_TIMEOUT = 6


@dataclass
class TurnExecution:
    exit_code: int
    agent_text: str = ""
    turn_id: str = ""
    error_message: str = ""
    elapsed_s: float = 0.0


def _emit_metrics(cfg: GcsConfig, *,
                  thread_id: str,
                  status: str,
                  total_ms: int,
                  result: Optional[TurnResult] = None,
                  ws_connect_ms: int = 0,
                  initialize_ms: int = 0,
                  resume_ms: int = 0,
                  lock_wait_ms: int = 0,
                  worker_id: Optional[str] = None,
                  content_sha256: Optional[str] = None,
                  error_class: Optional[str] = None,
                  retry_count: int = 0) -> None:
    rec: dict = {
        "thread_id": thread_id,
        "status": status,
        "total_ms": total_ms,
        "ws_connect_ms": ws_connect_ms,
        "initialize_ms": initialize_ms,
        "resume_ms": resume_ms,
        "lock_wait_ms": lock_wait_ms,
        "retry_count": retry_count,
        "relay_version": _relay_version(),
    }
    if result is not None:
        rec["turn_id"] = result.turn_id
        rec["turn_ms"] = int(result.elapsed_s * 1000)
        rec["ttft_ms"] = result.ttft_ms
        rec["input_tokens"] = result.input_tokens
        rec["cached_input_tokens"] = result.cached_input_tokens
        rec["output_tokens"] = result.output_tokens
    if worker_id:
        rec["worker_id"] = worker_id
    if content_sha256:
        rec["content_sha256"] = content_sha256
    if error_class:
        rec["error_class"] = error_class
    _metrics.append(rec, base=cfg.relay_base_dir())


def _relay_version() -> str:
    try:
        from importlib.metadata import version
        return version("gcs-orchestrator")
    except Exception:
        return "0.0.0"


def run_turn(cfg: GcsConfig,
             transport: CodexTransport,
             session_id: str,
             content: str,
             *,
             receipt_key: Optional[str] = None,
             worker_id: Optional[str] = None,
             lock_wait_ms: int = 0,
             connect_metrics: Optional[dict] = None,
             log = None) -> TurnExecution:
    """Run ONE turn over an already-connected transport.

    Caller is responsible for:
    - Acquiring any external lock (one-shot relay does FileLock; daemon
      serializes via its worker queue and doesn't need a file lock)
    - Calling transport.connect()/initialize() at least once before this
    - Idempotency receipt CHECK before calling (cheap optimization);
      this function ALSO writes pending+success receipts as the source of
      truth.

    `connect_metrics`: optional dict with ws_connect_ms, initialize_ms set
    by caller. Daemon will populate these once at startup and pass 0/0 for
    subsequent turns (no re-connect).
    """
    base = cfg.relay_base_dir()
    base.mkdir(parents=True, exist_ok=True)
    key = receipt_key or _receipts.key_for(session_id, content)
    cwd = cfg.orchestrator.cwd_override or None
    started_at = time.time()
    ws_connect_ms = (connect_metrics or {}).get("ws_connect_ms", 0)
    initialize_ms = (connect_metrics or {}).get("initialize_ms", 0)

    # Resume thread (cheap if recently resumed — codex caches in-process)
    t_resume = time.time()
    try:
        transport.resume_thread(session_id, cwd_override=cwd)
    except TransportError as exc:
        if log: log("ERROR", f"resume: {exc}")
        _emit_metrics(cfg, thread_id=session_id, status="error",
                      total_ms=int((time.time() - started_at) * 1000),
                      ws_connect_ms=ws_connect_ms, initialize_ms=initialize_ms,
                      lock_wait_ms=lock_wait_ms, worker_id=worker_id,
                      content_sha256=key, error_class="resume")
        return TurnExecution(exit_code=EXIT_APP_SERVER_ERROR, error_message=str(exc))
    resume_ms = int((time.time() - t_resume) * 1000)

    # Start turn
    handle = transport.start_turn(session_id, content, cwd_override=cwd)

    # Pending receipt so a mid-turn crash doesn't permanently block retry
    _receipts.write_atomic(base, key, {
        "status": "pending",
        "session_id": session_id,
        "content_sha256": key,
        "started_at": time.strftime("%Y-%m-%d %H:%M:%S"),
        "worker_id": worker_id,
    })

    result = transport.wait_for_turn(handle, cfg.transport.turn_timeout_s)
    elapsed = result.elapsed_s
    total_ms = int((time.time() - started_at) * 1000)

    if result.status == "timeout":
        if log: log("ERROR", f"turn timeout after {elapsed:.1f}s")
        _emit_metrics(cfg, thread_id=session_id, status="timeout", total_ms=total_ms,
                      result=result, ws_connect_ms=ws_connect_ms,
                      initialize_ms=initialize_ms, resume_ms=resume_ms,
                      lock_wait_ms=lock_wait_ms, worker_id=worker_id,
                      content_sha256=key, error_class="turn_timeout")
        return TurnExecution(exit_code=EXIT_TIMEOUT, elapsed_s=elapsed)

    if result.status == "failed":
        if log: log("ERROR", f"turn FAILED after {elapsed:.1f}s: {result.error_message}")
        _receipts.delete(base, key)
        _emit_metrics(cfg, thread_id=session_id, status="error", total_ms=total_ms,
                      result=result, ws_connect_ms=ws_connect_ms,
                      initialize_ms=initialize_ms, resume_ms=resume_ms,
                      lock_wait_ms=lock_wait_ms, worker_id=worker_id,
                      content_sha256=key, error_class="turn_failed")
        return TurnExecution(exit_code=EXIT_APP_SERVER_ERROR,
                             elapsed_s=elapsed, error_message=result.error_message)

    if result.status == "interrupted":
        if log: log("ERROR", f"turn INTERRUPTED after {elapsed:.1f}s: {result.error_message}")
        _receipts.delete(base, key)
        _emit_metrics(cfg, thread_id=session_id, status="aborted", total_ms=total_ms,
                      result=result, ws_connect_ms=ws_connect_ms,
                      initialize_ms=initialize_ms, resume_ms=resume_ms,
                      lock_wait_ms=lock_wait_ms, worker_id=worker_id,
                      content_sha256=key, error_class="turn_interrupted")
        return TurnExecution(exit_code=EXIT_TURN_ABORTED,
                             elapsed_s=elapsed, error_message=result.error_message)

    if log:
        log("INFO", f"turn completed in {elapsed:.1f}s, agent text len={len(result.assistant_text)}, "
                    f"tokens in={result.input_tokens} cached={result.cached_input_tokens} out={result.output_tokens}")

    _receipts.write_atomic(base, key, {
        "status": "success",
        "session_id": session_id,
        "content_sha256": key,
        "turn_id": result.turn_id,
        "elapsed_s": elapsed,
        "completed_at": time.strftime("%Y-%m-%d %H:%M:%S"),
        "input_tokens": result.input_tokens,
        "cached_input_tokens": result.cached_input_tokens,
        "output_tokens": result.output_tokens,
        "assistant_text_preview": result.assistant_text[:500],
        "worker_id": worker_id,
    })

    _emit_metrics(cfg, thread_id=session_id, status="ok", total_ms=total_ms,
                  result=result, ws_connect_ms=ws_connect_ms,
                  initialize_ms=initialize_ms, resume_ms=resume_ms,
                  lock_wait_ms=lock_wait_ms, worker_id=worker_id,
                  content_sha256=key)

    return TurnExecution(exit_code=EXIT_OK, agent_text=result.assistant_text,
                         turn_id=result.turn_id, elapsed_s=elapsed)


def connect_and_initialize(cfg: GcsConfig) -> tuple[AppServerTransport, dict]:
    """Helper for daemon startup + one-shot relay: returns transport + metrics dict.

    Metrics dict contains ws_connect_ms and initialize_ms for the caller
    to pass into run_turn().
    """
    transport = AppServerTransport(
        cfg.transport.ws_url,
        handshake_timeout_s=cfg.transport.handshake_timeout_s,
    )
    t0 = time.time()
    transport.connect()
    ws_connect_ms = int((time.time() - t0) * 1000)
    t1 = time.time()
    transport.initialize(client_name="gcs-app-relay")
    initialize_ms = int((time.time() - t1) * 1000)
    return transport, {"ws_connect_ms": ws_connect_ms, "initialize_ms": initialize_ms}
