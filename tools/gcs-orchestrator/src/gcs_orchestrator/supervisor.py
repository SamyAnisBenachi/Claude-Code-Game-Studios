"""Supervisor for codex app-server.

Spawns `codex app-server --listen <url>` as a subprocess, monitors via:
- Process liveness (subprocess.poll)
- HTTP `/readyz` probe every N seconds
- JSON-RPC liveness probe (`initialize` round-trip) every M seconds —
  catches "process alive but stuck" cases that PID checks miss

Restarts the subprocess on:
- Non-zero exit
- 3 consecutive HTTP probe failures
- 3 consecutive RPC probe timeouts
- PowerEventWatcher wake event (best-effort)

Backoff: exponential, capped at 60s. Logs all events to
`%LOCALAPPDATA%/gcs-app-relay/supervisor.log` with rotation.

Run via:
    python -m gcs_orchestrator.supervisor

Or wrap as a Windows service with NSSM:
    nssm install gcs-app-server "python" "-m" "gcs_orchestrator.supervisor"

The supervisor is a *foreground* process by design — NSSM handles
daemonization.
"""
from __future__ import annotations

import argparse
import json
import os
import socket
import subprocess
import sys
import threading
import time
import urllib.request
from pathlib import Path
from typing import Optional

from .config import load as load_config
from .platform import IS_WINDOWS, PowerEventWatcher


# ---- Logging with rotation ----

_LOG_MAX_BYTES = 10 * 1024 * 1024  # 10 MB
_LOG_BACKUP_COUNT = 5


def _rotate_log_if_needed(log_path: Path) -> None:
    try:
        if log_path.exists() and log_path.stat().st_size >= _LOG_MAX_BYTES:
            # Rotate: file.log -> file.log.1 -> file.log.2 ...
            for i in range(_LOG_BACKUP_COUNT, 0, -1):
                src = log_path.with_suffix(log_path.suffix + f".{i-1}") if i > 1 else log_path
                dst = log_path.with_suffix(log_path.suffix + f".{i}")
                if src.exists():
                    if dst.exists():
                        dst.unlink()
                    src.rename(dst)
    except OSError:
        pass


class SupervisorLogger:
    def __init__(self, log_path: Path) -> None:
        self.path = log_path
        self.path.parent.mkdir(parents=True, exist_ok=True)

    def __call__(self, level: str, msg: str) -> None:
        _rotate_log_if_needed(self.path)
        ts = time.strftime("%Y-%m-%d %H:%M:%S")
        line = f"[{ts}] {level} {msg}\n"
        try:
            with self.path.open("a", encoding="utf-8") as fh:
                fh.write(line)
        except OSError:
            pass
        sys.stderr.write(line)


# ---- Probes ----

def http_probe(url: str, timeout_s: float = 3.0) -> bool:
    try:
        with urllib.request.urlopen(url, timeout=timeout_s) as resp:
            return resp.status == 200
    except Exception:
        return False


def rpc_liveness_probe(ws_url: str, timeout_s: float = 10.0) -> bool:
    """Connect, send `initialize`, await response, close.

    Catches process-alive-but-stuck states that HTTP /readyz misses.
    """
    import websocket
    try:
        ws = websocket.WebSocket()
        ws.settimeout(timeout_s)
        ws.connect(ws_url, suppress_origin=True)
        ws.send(json.dumps({
            "jsonrpc": "2.0", "id": 1, "method": "initialize",
            "params": {"clientInfo": {"name": "supervisor-probe", "title": "probe", "version": "0.1"}},
        }))
        deadline = time.time() + timeout_s
        while time.time() < deadline:
            try:
                raw = ws.recv()
            except websocket.WebSocketTimeoutException:
                return False
            try:
                d = json.loads(raw)
            except json.JSONDecodeError:
                continue
            if d.get("id") == 1 and "result" in d:
                ws.close()
                return True
        ws.close()
        return False
    except Exception:
        return False


# ---- Supervisor loop ----

class AppServerSupervisor:
    def __init__(self, codex_bin: str, ws_url: str, readyz_url: str,
                 log: SupervisorLogger,
                 http_probe_interval_s: float = 30.0,
                 rpc_probe_interval_s: float = 60.0,
                 max_http_misses: int = 3,
                 max_rpc_misses: int = 3,
                 daemon_enabled: bool = True,
                 daemon_host: str = "127.0.0.1",
                 daemon_port: int = 9789,
                 ) -> None:
        self.codex_bin = codex_bin
        self.ws_url = ws_url
        self.readyz_url = readyz_url
        self.log = log
        self.http_probe_interval_s = http_probe_interval_s
        self.rpc_probe_interval_s = rpc_probe_interval_s
        self.max_http_misses = max_http_misses
        self.max_rpc_misses = max_rpc_misses

        self._proc: Optional[subprocess.Popen] = None
        self._stop = threading.Event()
        self._reconnect_now = threading.Event()
        self._backoff_s = 2.0
        self._max_backoff_s = 60.0

        # Relay daemon (sibling thread, co-located with app-server supervision)
        self.daemon_enabled = daemon_enabled
        self.daemon_host = daemon_host
        self.daemon_port = daemon_port
        self._daemon_server: Optional[object] = None
        self._daemon_thread: Optional[threading.Thread] = None
        # Status sidecar (read-only HTTP, port 9788)
        self._sidecar_server: Optional[object] = None
        self._sidecar_thread: Optional[threading.Thread] = None

    def _ws_url_to_listen_arg(self) -> str:
        return self.ws_url  # codex app-server --listen takes the ws:// URL directly

    def _spawn(self) -> None:
        self.log("INFO", f"spawning: {self.codex_bin} app-server --listen {self.ws_url}")
        creationflags = 0
        if IS_WINDOWS:
            creationflags = 0x08000000  # CREATE_NO_WINDOW
        try:
            self._proc = subprocess.Popen(
                [self.codex_bin, "app-server", "--listen", self.ws_url],
                stdin=subprocess.DEVNULL,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True,
                bufsize=1,
                encoding="utf-8",
                shell=False,
                creationflags=creationflags,
            )
            self.log("INFO", f"spawned PID {self._proc.pid}")
        except OSError as exc:
            self.log("ERROR", f"spawn failed: {exc!r}")
            self._proc = None

    def _kill_proc(self) -> None:
        if self._proc is None:
            return
        self.log("INFO", f"terminating PID {self._proc.pid}")
        try:
            self._proc.terminate()
            self._proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            try:
                self._proc.kill()
            except OSError:
                pass
        except OSError:
            pass
        self._proc = None

    def _drain_stdout_thread(self) -> None:
        """Echo app-server stdout/stderr into supervisor.log for forensics."""
        assert self._proc is not None
        def _drain(stream, tag: str) -> None:
            try:
                for line in iter(stream.readline, ""):
                    line = line.rstrip()
                    if line:
                        self.log(tag, line)
            except Exception:
                pass
        if self._proc.stdout:
            threading.Thread(target=_drain, args=(self._proc.stdout, "STDOUT"), daemon=True).start()
        if self._proc.stderr:
            threading.Thread(target=_drain, args=(self._proc.stderr, "STDERR"), daemon=True).start()

    def _wait_for_ready(self, timeout_s: float = 30.0) -> bool:
        """Poll /readyz until it returns 200 or timeout."""
        deadline = time.time() + timeout_s
        while time.time() < deadline:
            if http_probe(self.readyz_url):
                return True
            time.sleep(1.0)
        return False

    def stop(self) -> None:
        self._stop.set()
        self._kill_proc()

    def request_reconnect(self) -> None:
        """Called by power-resume hook: force a probe cycle now."""
        self._reconnect_now.set()
        # Also force the relay daemon's transport to reconnect
        if self._daemon_server is not None:
            try:
                self._daemon_server.worker.request_reconnect()  # type: ignore[attr-defined]
            except Exception:
                pass

    def _start_daemon(self) -> None:
        if not self.daemon_enabled or self._daemon_thread is not None:
            return
        try:
            from . import daemon as _daemon_mod
            from .config import load as load_config
            cfg = load_config()
            self._daemon_server = _daemon_mod.DaemonServer(
                self.daemon_host, self.daemon_port, cfg,
                lambda lvl, msg: self.log(f"DAEMON.{lvl}", msg),
            )
            self._daemon_server.worker.start()
            self._daemon_thread = threading.Thread(
                target=self._daemon_server.serve_forever,
                daemon=True, name="RelayDaemonServer",
            )
            self._daemon_thread.start()
            self.log("INFO", f"relay daemon up on {self.daemon_host}:{self.daemon_port}")
        except OSError as exc:
            # Port already in use, etc. Daemon is optional — log and continue.
            self.log("WARN", f"relay daemon NOT started: {exc!r}")
            self._daemon_server = None
            self._daemon_thread = None

    def _stop_daemon(self) -> None:
        if self._daemon_server is None:
            return
        try:
            self._daemon_server.worker.stop()  # type: ignore[attr-defined]
            self._daemon_server.shutdown()  # type: ignore[attr-defined]
            self._daemon_server.server_close()  # type: ignore[attr-defined]
        except Exception:
            pass
        self._daemon_server = None
        self._daemon_thread = None

    def _start_sidecar(self) -> None:
        if self._sidecar_thread is not None:
            return
        try:
            from . import sidecar
            self._sidecar_server, self._sidecar_thread = sidecar.start(
                supervisor_ref=self, host="127.0.0.1", port=9788,
            )
            self.log("INFO", "status sidecar up on 127.0.0.1:9788")
        except OSError as exc:
            self.log("WARN", f"status sidecar NOT started: {exc!r}")
            self._sidecar_server = None
            self._sidecar_thread = None

    def _stop_sidecar(self) -> None:
        if self._sidecar_server is None:
            return
        try:
            self._sidecar_server.shutdown()  # type: ignore[attr-defined]
            self._sidecar_server.server_close()  # type: ignore[attr-defined]
        except Exception:
            pass
        self._sidecar_server = None
        self._sidecar_thread = None

    def run(self) -> int:
        """Main loop. Blocks until self.stop() is called."""
        watcher = PowerEventWatcher(on_resume=self.request_reconnect)
        watcher.start()
        try:
            while not self._stop.is_set():
                self._spawn()
                if self._proc is None:
                    time.sleep(self._backoff_s)
                    self._backoff_s = min(self._backoff_s * 2, self._max_backoff_s)
                    continue
                self._drain_stdout_thread()
                if not self._wait_for_ready(30):
                    self.log("ERROR", "app-server did not become ready in 30s — restarting")
                    self._kill_proc()
                    time.sleep(self._backoff_s)
                    self._backoff_s = min(self._backoff_s * 2, self._max_backoff_s)
                    continue

                self.log("INFO", "app-server READY")
                self._backoff_s = 2.0  # reset
                # Start the relay daemon + status sidecar after app-server is READY
                # (first time only — both survive across app-server restarts)
                self._start_daemon()
                self._start_sidecar()
                self._monitor_loop()
                # If we get here, _monitor_loop returned because of a failure
                self._kill_proc()
                # Force daemon to drop its transport so it reconnects to the
                # restarted app-server cleanly
                if self._daemon_server is not None:
                    try:
                        self._daemon_server.worker.request_reconnect()  # type: ignore[attr-defined]
                    except Exception:
                        pass
                time.sleep(self._backoff_s)
                self._backoff_s = min(self._backoff_s * 2, self._max_backoff_s)
        finally:
            watcher.stop()
            self._kill_proc()
            self._stop_daemon()
            self._stop_sidecar()
        return 0

    def _monitor_loop(self) -> None:
        """Run until self._proc dies, probes consistently fail, or stop signaled."""
        last_http = time.time()
        last_rpc = time.time()
        http_misses = 0
        rpc_misses = 0

        while not self._stop.is_set():
            if self._proc is None or self._proc.poll() is not None:
                code = self._proc.returncode if self._proc else None
                self.log("WARN", f"app-server process exited with code {code}")
                return

            time.sleep(2.0)
            now = time.time()
            force = self._reconnect_now.is_set()
            if force:
                self._reconnect_now.clear()
                self.log("INFO", "power-resume event — forcing probe cycle")

            if force or now - last_http >= self.http_probe_interval_s:
                if http_probe(self.readyz_url):
                    http_misses = 0
                else:
                    http_misses += 1
                    self.log("WARN", f"http probe miss {http_misses}/{self.max_http_misses}")
                    if http_misses >= self.max_http_misses:
                        self.log("ERROR", "http probe exhausted — restarting app-server")
                        return
                last_http = now

            if force or now - last_rpc >= self.rpc_probe_interval_s:
                if rpc_liveness_probe(self.ws_url, timeout_s=10):
                    rpc_misses = 0
                else:
                    rpc_misses += 1
                    self.log("WARN", f"rpc probe miss {rpc_misses}/{self.max_rpc_misses} — process alive but unresponsive")
                    if rpc_misses >= self.max_rpc_misses:
                        self.log("ERROR", "rpc probe exhausted — restarting app-server")
                        return
                last_rpc = now


# ---- Entry point ----

def main(argv: Optional[list[str]] = None) -> int:
    parser = argparse.ArgumentParser(prog="gcs-app-supervisor")
    parser.add_argument("--listen", default="", help="override transport.ws_url from config")
    parser.add_argument("--codex-bin", default="", help="override pin.codex_bin from config")
    args = parser.parse_args(argv)

    cfg = load_config()
    ws_url = args.listen or cfg.transport.ws_url
    readyz_url = cfg.transport.readyz_url
    codex_bin = args.codex_bin or cfg.pin.codex_bin or "codex"

    # Resolve codex_bin: if not absolute and not on PATH, fall back to the
    # standard npm install location.
    if not Path(codex_bin).exists():
        default_npm = r"C:\Users\Sam\AppData\Roaming\npm\codex.cmd"
        if Path(default_npm).exists():
            codex_bin = default_npm

    log_path = cfg.relay_base_dir() / "supervisor.log"
    logger = SupervisorLogger(log_path)
    logger("INFO", f"supervisor starting — codex_bin={codex_bin} ws={ws_url}")

    sup = AppServerSupervisor(
        codex_bin=codex_bin,
        ws_url=ws_url,
        readyz_url=readyz_url,
        log=logger,
    )
    return sup.run()


if __name__ == "__main__":
    sys.exit(main())
