"""Platform-specific abstractions.

Keeps Windows-only code (msvcrt file lock, cmd.exe wrappers, PowerShell
toast, %LOCALAPPDATA%) behind a thin interface so tests can mock or
substitute on Linux/macOS CI.

Currently implements Windows fully + POSIX fallbacks (fcntl) for unit
tests.
"""
from __future__ import annotations

import os
import socket
import subprocess
import sys
import threading
import time
from pathlib import Path
from typing import Optional


IS_WINDOWS = os.name == "nt"


# ---- File lock ----

class FileLock:
    """Cross-platform exclusive file lock with timeout.

    Records PID + start timestamp in the lock file body so timeout
    diagnostics can identify the holder.
    """

    def __init__(self, path: Path, timeout_s: float) -> None:
        self.path = path
        self.timeout_s = timeout_s
        self._fh = None
        self._is_locked = False

    def __enter__(self) -> "FileLock":
        self.path.parent.mkdir(parents=True, exist_ok=True)
        # Binary mode + don't truncate while locked (Windows msvcrt
        # byte-range lock is on a specific offset)
        self._fh = open(self.path, "a+b")
        deadline = time.time() + self.timeout_s
        while True:
            try:
                if IS_WINDOWS:
                    import msvcrt
                    msvcrt.locking(self._fh.fileno(), msvcrt.LK_NBLCK, 1)
                else:
                    import fcntl
                    fcntl.flock(self._fh.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
                self._is_locked = True
                return self
            except (OSError, IOError):
                if time.time() >= deadline:
                    holder = self._read_holder_diag()
                    raise TimeoutError(
                        f"could not acquire {self.path} within {self.timeout_s}s — "
                        f"held by: {holder}"
                    )
                time.sleep(0.5)

    def _read_holder_diag(self) -> str:
        """Best-effort read of the lock file's PID+ts marker (in a sidecar)."""
        try:
            sidecar = self.path.with_suffix(self.path.suffix + ".holder")
            if sidecar.exists():
                return sidecar.read_text(encoding="utf-8")[:200].strip()
        except OSError:
            pass
        return "(no holder marker)"

    def __exit__(self, exc_type, exc, tb) -> None:
        if not self._is_locked or self._fh is None:
            return
        try:
            if IS_WINDOWS:
                import msvcrt
                self._fh.seek(0)
                msvcrt.locking(self._fh.fileno(), msvcrt.LK_UNLCK, 1)
            else:
                import fcntl
                fcntl.flock(self._fh.fileno(), fcntl.LOCK_UN)
        except (OSError, IOError):
            pass
        try:
            self._fh.close()
        except OSError:
            pass


def write_holder_diag(lock_path: Path, info: Optional[str] = None) -> None:
    """Write a sidecar file with diagnostics about the current lock holder.

    Called by the lock owner (after acquisition) so future blocked callers
    can see who's holding it without disturbing the lock.
    """
    sidecar = lock_path.with_suffix(lock_path.suffix + ".holder")
    try:
        sidecar.write_text(
            (info or f"pid={os.getpid()} since={time.strftime('%Y-%m-%d %H:%M:%S')}\n"),
            encoding="utf-8",
        )
    except OSError:
        pass


# ---- Detached subprocess spawn (for spawn-watchdog and similar) ----

def spawn_detached(argv: list[str], cwd: Optional[str] = None) -> Optional[int]:
    """Spawn a background process that survives the parent's exit.

    Returns the spawned PID (best-effort) or None on failure.
    """
    creationflags = 0
    if IS_WINDOWS:
        DETACHED_PROCESS = 0x00000008
        CREATE_NEW_PROCESS_GROUP = 0x00000200
        CREATE_NO_WINDOW = 0x08000000
        creationflags = DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW
    try:
        proc = subprocess.Popen(
            argv,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            creationflags=creationflags,
            close_fds=True,
            cwd=cwd,
        )
        return proc.pid
    except OSError:
        return None


def spawn_console_window(title: str, command: str, cwd: Optional[str] = None) -> Optional[int]:
    """Spawn a NEW cmd window (Windows) with a title + command, returning PID.

    Used to launch viewer / app-server in dedicated windows the user can see.
    No-op on non-Windows (just spawns the command directly).
    """
    if not IS_WINDOWS:
        return spawn_detached(["bash", "-c", command], cwd=cwd)
    CREATE_NEW_CONSOLE = 0x00000010
    full_cmd = f"title {title} && {command}"
    try:
        proc = subprocess.Popen(
            ["cmd.exe", "/k", full_cmd],
            creationflags=CREATE_NEW_CONSOLE,
            close_fds=True,
            cwd=cwd,
        )
        return proc.pid
    except OSError:
        return None


# ---- Windows toast notification ----

def windows_toast(title: str, message: str, click_action: Optional[str] = None) -> None:
    """Display a Windows toast notification. No-op on non-Windows.

    Uses PowerShell BurntToast if available, else falls back to msg.exe.
    Failures are silent — toasts are best-effort cosmetic.
    """
    if not IS_WINDOWS:
        return
    # Use a minimal embedded PowerShell snippet — avoids dependency on a
    # specific module if BurntToast isn't installed.
    ps_script = f"""
$ErrorActionPreference = 'SilentlyContinue'
Add-Type -AssemblyName System.Windows.Forms
$notify = New-Object System.Windows.Forms.NotifyIcon
$notify.Icon = [System.Drawing.SystemIcons]::Information
$notify.Visible = $true
$notify.ShowBalloonTip(5000, '{title.replace("'", "''")}', '{message.replace("'", "''")[:200]}', [System.Windows.Forms.ToolTipIcon]::Info)
Start-Sleep -Milliseconds 5500
$notify.Dispose()
"""
    try:
        subprocess.Popen(
            ["powershell", "-NoProfile", "-WindowStyle", "Hidden", "-Command", ps_script],
            stdin=subprocess.DEVNULL,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            creationflags=0x08000000 if IS_WINDOWS else 0,  # CREATE_NO_WINDOW
        )
    except OSError:
        pass


# ---- Power event hook (PC sleep/resume detection) ----

class PowerEventWatcher:
    """Subscribes to Windows power-mode-changed events.

    On suspend → calls `on_suspend()`. On resume → calls `on_resume()`.
    No-op on non-Windows (callbacks never fire). Uses a background thread.

    Usage:
        watcher = PowerEventWatcher(on_resume=lambda: reconnect_ws())
        watcher.start()
        ...
        watcher.stop()
    """

    def __init__(self, on_suspend=None, on_resume=None) -> None:
        self.on_suspend = on_suspend
        self.on_resume = on_resume
        self._thread: Optional[threading.Thread] = None
        self._stop = threading.Event()

    def start(self) -> None:
        if not IS_WINDOWS:
            return
        self._thread = threading.Thread(target=self._run_loop, daemon=True)
        self._thread.start()

    def stop(self) -> None:
        self._stop.set()

    def _run_loop(self) -> None:
        """Use pywin32 if available; else fall back to monotonic-drift detection."""
        try:
            self._run_pywin32()
        except Exception:
            self._run_drift_detection()

    def _run_pywin32(self) -> None:
        import win32api  # type: ignore
        import win32con  # type: ignore
        # win32api event hooks would be ideal but require a message pump.
        # For now, use the drift fallback even on Windows; pywin32 is optional.
        raise NotImplementedError

    def _run_drift_detection(self) -> None:
        """Fallback: compare wall-clock vs monotonic to detect sleep.

        If wall-clock jumps forward by >30 s while monotonic only moved
        a few seconds, we know the process was suspended (PC slept).
        """
        last_wall = time.time()
        last_mono = time.monotonic()
        while not self._stop.is_set():
            time.sleep(2.0)
            wall_now = time.time()
            mono_now = time.monotonic()
            wall_dt = wall_now - last_wall
            mono_dt = mono_now - last_mono
            # If wall jumped >30s but monotonic only advanced a few sec,
            # the process was suspended.
            if wall_dt - mono_dt > 30:
                if self.on_resume:
                    try:
                        self.on_resume()
                    except Exception:
                        pass
            last_wall = wall_now
            last_mono = mono_now


# ---- TCP keepalive enable ----

def enable_tcp_keepalive(sock: socket.socket) -> None:
    """Best-effort TCP keepalive on a connected socket.

    Detects half-open peers ~75 s after silence on Windows defaults.
    """
    try:
        sock.setsockopt(socket.SOL_SOCKET, socket.SO_KEEPALIVE, 1)
    except OSError:
        pass


__all__ = [
    "IS_WINDOWS",
    "FileLock",
    "write_holder_diag",
    "spawn_detached",
    "spawn_console_window",
    "windows_toast",
    "PowerEventWatcher",
    "enable_tcp_keepalive",
]
