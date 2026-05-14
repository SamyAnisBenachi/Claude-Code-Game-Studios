"""Single-shot worker DONE relay.

Workers invoke this from their Step 3:
    python -m gcs_orchestrator.relay <thread-id> <content-or-dash>

Resolves config, acquires file lock, checks idempotency receipt, opens
an `AppServerTransport`, runs the turn, writes a final receipt, exits
with a status code the caller can branch on.

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

import hashlib
import json
import os
import sys
import time
from pathlib import Path
from typing import Optional

from .config import GcsConfig, load as load_config
from .platform import FileLock, write_holder_diag
from .transport import AppServerTransport, TransportError, server_reachable


EXIT_OK = 0
EXIT_DUPLICATE = 2
EXIT_LOCK_TIMEOUT = 3
EXIT_APP_SERVER_ERROR = 4
EXIT_TURN_ABORTED = 5
EXIT_TIMEOUT = 6
EXIT_BAD_USAGE = 9


# ---- Receipts / idempotency ----

def _receipt_key(session_id: str, content: str) -> str:
    return hashlib.sha256(
        f"{session_id}\0{content}".encode("utf-8")
    ).hexdigest()[:32]


def _receipt_path(base: Path, key: str) -> Path:
    return base / "receipts" / f"{key}.receipt"


def _check_receipt(base: Path, key: str) -> bool:
    """True only if a SUCCESS receipt exists. Pending receipts don't block."""
    p = _receipt_path(base, key)
    if not p.exists():
        return False
    try:
        d = json.loads(p.read_text(encoding="utf-8"))
        return d.get("status") == "success"
    except (OSError, json.JSONDecodeError):
        return False


def _write_receipt_atomic(base: Path, key: str, payload: dict) -> None:
    """Write receipt via tmp + os.replace (Windows-safe atomic)."""
    receipts_dir = base / "receipts"
    receipts_dir.mkdir(parents=True, exist_ok=True)
    p = _receipt_path(base, key)
    tmp = p.with_suffix(".receipt.tmp")
    try:
        tmp.write_text(json.dumps(payload, indent=2), encoding="utf-8")
        os.replace(tmp, p)
    except OSError:
        pass


def _prune_receipts(base: Path, ttl_days: int, max_count: int) -> None:
    """Lazy prune. Only fires if last prune was >1 hour ago (sentinel file)."""
    receipts_dir = base / "receipts"
    if not receipts_dir.exists():
        return
    sentinel = base / ".last_prune_at"
    try:
        if sentinel.exists():
            if time.time() - sentinel.stat().st_mtime < 3600:
                return
        receipts_dir.mkdir(parents=True, exist_ok=True)
        files = sorted(receipts_dir.glob("*.receipt"), key=lambda p: p.stat().st_mtime, reverse=True)
        cutoff = time.time() - ttl_days * 86400
        for i, f in enumerate(files):
            try:
                if i >= max_count or f.stat().st_mtime < cutoff:
                    f.unlink(missing_ok=True)
            except OSError:
                pass
        sentinel.touch()
    except OSError:
        pass


# ---- Logging ----

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


# ---- Main relay flow ----

def relay_turn(cfg: GcsConfig, session_id: str, content: str) -> int:
    base = cfg.relay_base_dir()
    base.mkdir(parents=True, exist_ok=True)

    _prune_receipts(base, cfg.reliability.receipt_ttl_days, cfg.reliability.receipt_max_count)

    key = _receipt_key(session_id, content)

    if _check_receipt(base, key):
        _log(base, "SKIP", f"idempotent: receipt {key} already present for {session_id}")
        return EXIT_DUPLICATE

    if not server_reachable(cfg.transport.readyz_url, timeout_s=3.0):
        _log(base, "ERROR", f"app-server not reachable at {cfg.transport.readyz_url}")
        return EXIT_APP_SERVER_ERROR

    lock_path = base / "turn.lock"
    _log(base, "INFO", f"acquiring lock for session={session_id} receipt={key}")
    try:
        with FileLock(lock_path, cfg.transport.turn_timeout_s):
            write_holder_diag(lock_path, f"pid={os.getpid()} session={session_id[:12]} receipt={key[:12]} since={time.strftime('%H:%M:%S')}")
            _log(base, "INFO", "lock acquired")
            # Re-check receipt under lock (another concurrent relay might have completed it)
            if _check_receipt(base, key):
                _log(base, "SKIP", "duplicate detected after lock acquired")
                return EXIT_DUPLICATE
            return _do_turn(cfg, base, session_id, content, key)
    except TimeoutError as exc:
        _log(base, "ERROR", f"lock timeout: {exc}")
        return EXIT_LOCK_TIMEOUT


def _do_turn(cfg: GcsConfig, base: Path, session_id: str, content: str, key: str) -> int:
    transport = AppServerTransport(cfg.transport.ws_url, handshake_timeout_s=cfg.transport.handshake_timeout_s)
    t_start = time.time()
    try:
        try:
            transport.connect()
        except Exception as exc:
            _log(base, "ERROR", f"WS connect failed: {exc!r}")
            return EXIT_APP_SERVER_ERROR

        try:
            transport.initialize(client_name="gcs-app-relay")
        except TransportError as exc:
            _log(base, "ERROR", f"initialize: {exc}")
            return EXIT_APP_SERVER_ERROR

        cwd = cfg.orchestrator.cwd_override or None
        try:
            transport.resume_thread(session_id, cwd_override=cwd)
        except TransportError as exc:
            _log(base, "ERROR", f"resume: {exc}")
            return EXIT_APP_SERVER_ERROR
        _log(base, "INFO", f"thread resumed in {time.time()-t_start:.2f}s")

        handle = transport.start_turn(session_id, content, cwd_override=cwd)

        # Write pending receipt so a mid-turn crash doesn't permanently block retry
        _write_receipt_atomic(base, key, {
            "status": "pending",
            "session_id": session_id,
            "content_sha256": key,
            "started_at": time.strftime("%Y-%m-%d %H:%M:%S"),
        })

        result = transport.wait_for_turn(handle, cfg.transport.turn_timeout_s)
        elapsed = result.elapsed_s

        if result.status == "timeout":
            _log(base, "ERROR", f"turn timeout after {elapsed:.1f}s")
            return EXIT_TIMEOUT
        if result.status == "failed":
            _log(base, "ERROR", f"turn FAILED after {elapsed:.1f}s: {result.error_message}")
            # Drop pending receipt so retry isn't blocked
            try:
                _receipt_path(base, key).unlink(missing_ok=True)
            except OSError:
                pass
            return EXIT_APP_SERVER_ERROR
        if result.status == "interrupted":
            _log(base, "ERROR", f"turn INTERRUPTED after {elapsed:.1f}s: {result.error_message}")
            try:
                _receipt_path(base, key).unlink(missing_ok=True)
            except OSError:
                pass
            return EXIT_TURN_ABORTED

        _log(base, "INFO", f"turn completed in {elapsed:.1f}s, agent text len={len(result.assistant_text)}, "
                            f"tokens in={result.input_tokens} cached={result.cached_input_tokens} out={result.output_tokens}")

        _write_receipt_atomic(base, key, {
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
        })

        if result.assistant_text:
            print(result.assistant_text)
        return EXIT_OK

    except Exception as exc:
        _log(base, "ERROR", f"unexpected: {exc!r}")
        return EXIT_APP_SERVER_ERROR
    finally:
        transport.close()


def _usage() -> None:
    sys.stderr.write(
        "Usage: python -m gcs_orchestrator.relay <thread-id> <content-or-dash>\n"
        "       (use '-' as content to read from stdin)\n"
    )


def main(argv: Optional[list[str]] = None) -> int:
    argv = argv if argv is not None else sys.argv
    if len(argv) != 3:
        _usage()
        return EXIT_BAD_USAGE
    # Force UTF-8 stdin on Windows
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
    return relay_turn(cfg, session_id, content)


if __name__ == "__main__":
    sys.exit(main())
