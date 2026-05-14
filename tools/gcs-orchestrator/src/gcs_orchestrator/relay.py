"""Single-shot worker DONE relay (one-shot fallback path).

Workers invoke this from their Step 3:
    python -m gcs_orchestrator.relay <thread-id> <content-or-dash>

Resolves config, acquires file lock, checks idempotency receipt, opens
an AppServerTransport, runs the turn via turn_executor.run_turn, exits
with a status code the caller can branch on.

This is the FALLBACK path. The persistent daemon (`gcs-relay-daemon`)
is the preferred fast path (no per-call WS handshake). The spawn_watchdog
probes the daemon first and falls back to this module when the daemon
is unreachable.

Exit codes:
    0  EXIT_OK              turn delivered and processed
    2  EXIT_DUPLICATE       prior identical content already delivered
    3  EXIT_LOCK_TIMEOUT    file lock not acquired within timeout
    4  EXIT_APP_SERVER_ERROR  unreachable / RPC error
    5  EXIT_TURN_ABORTED    turn interrupted / aborted by orchestrator
    6  EXIT_TIMEOUT         turn did not complete within turn_timeout_s
    9  EXIT_BAD_USAGE       missing arguments
"""
from __future__ import annotations

import os
import sys
import time
from pathlib import Path
from typing import Optional

from . import daemon as _daemon
from . import receipts as _receipts
from . import turn_executor as _executor
from .config import GcsConfig, load as load_config
from .platform import FileLock, write_holder_diag
from .transport import TransportError, server_reachable


EXIT_OK = _executor.EXIT_OK
EXIT_DUPLICATE = _executor.EXIT_DUPLICATE
EXIT_LOCK_TIMEOUT = _executor.EXIT_LOCK_TIMEOUT
EXIT_APP_SERVER_ERROR = _executor.EXIT_APP_SERVER_ERROR
EXIT_TURN_ABORTED = _executor.EXIT_TURN_ABORTED
EXIT_TIMEOUT = _executor.EXIT_TIMEOUT
EXIT_BAD_USAGE = 9


def _log(base: Path, level: str, msg: str) -> None:
    ts = time.strftime("%Y-%m-%d %H:%M:%S")
    line = f"[{ts}] {level} {msg}\n"
    try:
        base.mkdir(parents=True, exist_ok=True)
        with (base / "relay.log").open("a", encoding="utf-8") as fh:
            fh.write(line)
    except OSError:
        pass
    sys.stderr.write(line)


def _try_daemon(cfg: GcsConfig, session_id: str, content: str,
                worker_id: Optional[str], base: Path) -> Optional[int]:
    """Probe daemon at 127.0.0.1:9789 and submit there if healthy.

    Returns the exit code on daemon success, or None if daemon is
    unreachable / unhealthy (caller should fall back to one-shot path).
    """
    health = _daemon.client_health(timeout_s=1.0)
    if health is None:
        return None
    if not health.get("ws_connected", False):
        _log(base, "INFO", "daemon up but ws_connected=false — fallback to one-shot")
        return None
    try:
        resp = _daemon.client_turn(
            session_id, content,
            worker_id=worker_id,
            timeout_s=cfg.transport.turn_timeout_s,
        )
        if resp.get("type") != "turn_result":
            _log(base, "WARN", f"daemon non-turn response: {resp}")
            return None
        exit_code = int(resp.get("exit_code", EXIT_APP_SERVER_ERROR))
        agent_text = resp.get("agent_text", "") or ""
        if exit_code == EXIT_OK and agent_text:
            print(agent_text)
        _log(base, "INFO", f"delivered via daemon, exit={exit_code} "
                            f"elapsed={resp.get('elapsed_s', 0):.1f}s")
        return exit_code
    except (ConnectionError, OSError) as exc:
        _log(base, "WARN", f"daemon mid-request failure ({exc!r}) — fallback")
        return None


def relay_turn(cfg: GcsConfig, session_id: str, content: str,
               worker_id: Optional[str] = None) -> int:
    """Synchronously deliver a worker DONE to the orchestrator.

    Fast path: probe relay daemon, submit there.
    Fallback: one-shot WS handshake (legacy behavior).

    Returns one of the EXIT_* constants.
    """
    base = cfg.relay_base_dir()
    base.mkdir(parents=True, exist_ok=True)

    _receipts.prune_if_stale(base,
                             cfg.reliability.receipt_ttl_days,
                             cfg.reliability.receipt_max_count)

    key = _receipts.key_for(session_id, content)

    if _receipts.is_success(base, key):
        _log(base, "SKIP", f"idempotent: receipt {key} already present for {session_id}")
        return EXIT_DUPLICATE

    # Try the persistent daemon first (saves ~500ms WS handshake)
    daemon_result = _try_daemon(cfg, session_id, content, worker_id, base)
    if daemon_result is not None:
        return daemon_result

    if not server_reachable(cfg.transport.readyz_url, timeout_s=3.0):
        _log(base, "ERROR", f"app-server not reachable at {cfg.transport.readyz_url}")
        return EXIT_APP_SERVER_ERROR

    lock_path = base / "turn.lock"
    _log(base, "INFO", f"acquiring lock for session={session_id} receipt={key}")
    lock_t0 = time.time()
    try:
        with FileLock(lock_path, cfg.transport.turn_timeout_s):
            lock_wait_ms = int((time.time() - lock_t0) * 1000)
            write_holder_diag(
                lock_path,
                f"pid={os.getpid()} session={session_id[:12]} "
                f"receipt={key[:12]} since={time.strftime('%H:%M:%S')}",
            )
            _log(base, "INFO", f"lock acquired in {lock_wait_ms}ms")
            # Re-check receipt under lock (concurrent relay might have completed)
            if _receipts.is_success(base, key):
                _log(base, "SKIP", "duplicate detected after lock acquired")
                return EXIT_DUPLICATE
            return _do_turn(cfg, base, session_id, content, key,
                            lock_wait_ms=lock_wait_ms, worker_id=worker_id)
    except TimeoutError as exc:
        _log(base, "ERROR", f"lock timeout: {exc}")
        return EXIT_LOCK_TIMEOUT


def _do_turn(cfg: GcsConfig, base: Path, session_id: str, content: str, key: str,
             lock_wait_ms: int, worker_id: Optional[str]) -> int:
    try:
        transport, connect_metrics = _executor.connect_and_initialize(cfg)
    except TransportError as exc:
        _log(base, "ERROR", f"initialize: {exc}")
        return EXIT_APP_SERVER_ERROR
    except Exception as exc:
        _log(base, "ERROR", f"WS connect failed: {exc!r}")
        return EXIT_APP_SERVER_ERROR

    try:
        execution = _executor.run_turn(
            cfg, transport, session_id, content,
            receipt_key=key,
            worker_id=worker_id,
            lock_wait_ms=lock_wait_ms,
            connect_metrics=connect_metrics,
            log=lambda lvl, msg: _log(base, lvl, msg),
        )
        if execution.exit_code == EXIT_OK and execution.agent_text:
            print(execution.agent_text)
        return execution.exit_code
    except Exception as exc:
        _log(base, "ERROR", f"unexpected: {exc!r}")
        return EXIT_APP_SERVER_ERROR
    finally:
        transport.close()


def _usage() -> None:
    sys.stderr.write(
        "Usage: python -m gcs_orchestrator.relay <thread-id> <content-or-dash>\n"
        "       (use '-' as content to read from stdin)\n"
        "       set GCS_WORKER_ID=PROMPT-NNN to tag the metric record\n"
    )


def main(argv: Optional[list[str]] = None) -> int:
    argv = argv if argv is not None else sys.argv
    if len(argv) != 3:
        _usage()
        return EXIT_BAD_USAGE
    try:
        sys.stdin.reconfigure(encoding="utf-8", errors="strict")
    except Exception:
        pass

    try:
        cfg = load_config()
    except Exception as exc:
        sys.stderr.write(f"config load failed: {exc!r}\n")
        return EXIT_BAD_USAGE

    session_id = argv[1] or cfg.orchestrator.session_id
    content = argv[2]
    if content == "-":
        content = sys.stdin.read()
    if not session_id or not content.strip():
        _usage()
        return EXIT_BAD_USAGE
    worker_id = os.environ.get("GCS_WORKER_ID") or None
    return relay_turn(cfg, session_id, content, worker_id=worker_id)


if __name__ == "__main__":
    sys.exit(main())
