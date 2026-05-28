"""Windows-only helper: foreground the primary Bevy/CCGS window before screenshot capture.

Imported by driver.py and called before each autoplay/screenshot RPC so the Bevy
window is the active composited surface when Screenshot::primary_window() fires.
No-op on non-Windows. No third-party dependencies — stdlib ctypes only.

PROMPT 1776: foreground repair for screenshot distinctness.
PROMPT 1786: window title diagnostic — added "lanes and lies" hint (actual client title
             from client/src/main.rs WindowPlugin), extended hints list, and bounded
             diagnostic logging of visible titles when no match is found.
PROMPT 1808: robust foreground bypass — AttachThreadInput + TOPMOST/NOTOPMOST +
             synthetic ALT key fallback to defeat Windows foreground lock.
             Fixes: SetForegroundWindow ret=0 on every tick (diagnosed in PROMPT 1807)
             which caused DWM to serve stale frames to PrintWindow/RPC captures.
"""
from __future__ import annotations

import sys
from typing import Callable

_IS_WINDOWS = sys.platform == "win32"

# Case-insensitive substrings matched against visible top-level window titles.
#
# Priority order: most-specific titles first to avoid accidentally matching an
# unrelated window that happens to contain a generic substring.
#
# "lanes and lies" — actual title set in client/src/main.rs WindowPlugin
# "lanes"          — substring fallback (title may be truncated by OS)
# "ccgs"           — legacy/test builds that used the CCGS working title
# "claude code game" — verbose version of the CCGS placeholder title
# "bevy app"       — Bevy default when no custom title is set (debug builds)
# "bevy"           — last-resort fallback for any unlabelled Bevy window
_WINDOW_TITLE_HINTS: tuple[str, ...] = (
    "lanes and lies",
    "lanes",
    "ccgs",
    "claude code game",
    "bevy app",
    "bevy",
)

# Max visible window titles to dump in diagnostics when no match is found.
# Bounded to avoid flooding logs with the full Windows desktop inventory.
_DIAG_TITLE_LIMIT = 30

_SW_RESTORE = 9  # ShowWindow: restore a minimised window to its normal size


# ---------------------------------------------------------------------------
# Pure logic — no ctypes, directly unit-testable on any platform
# ---------------------------------------------------------------------------

def _find_candidate(
    windows: list[tuple[int, str]],
) -> tuple[int, str] | None:
    """Return the first (hwnd, title) pair whose title contains a known hint.

    Case-insensitive substring match against each entry in _WINDOW_TITLE_HINTS.
    Returns None when no window matches.
    """
    for hwnd, title in windows:
        title_lower = title.lower()
        if any(hint in title_lower for hint in _WINDOW_TITLE_HINTS):
            return hwnd, title
    return None


def _format_diag_titles(windows: list[tuple[int, str]], limit: int = _DIAG_TITLE_LIMIT) -> str:
    """Return a compact, bounded diagnostic string of visible window titles.

    Titles are truncated at 60 chars each to prevent log spam. The list is
    capped at *limit* entries. Readable in a single log line for easy
    comparison against _WINDOW_TITLE_HINTS.
    """
    truncated = windows[:limit]
    parts = [repr(t[:60]) for _, t in truncated]
    suffix = f" … (+{len(windows) - limit} more)" if len(windows) > limit else ""
    return "[" + ", ".join(parts) + "]" + suffix


# ---------------------------------------------------------------------------
# Windows API wrappers — user32 is an injectable parameter for testability
# ---------------------------------------------------------------------------

def _list_visible_windows(user32) -> list[tuple[int, str]]:
    """Enumerate all visible top-level windows via user32.EnumWindows.

    Returns a list of (hwnd, title) for every visible window that has a title.
    Only valid to call on Windows (ctypes.wintypes is Windows-only).
    """
    import ctypes
    import ctypes.wintypes  # noqa: PLC0415

    results: list[tuple[int, str]] = []

    EnumWindowsProc = ctypes.WINFUNCTYPE(
        ctypes.wintypes.BOOL,
        ctypes.wintypes.HWND,
        ctypes.wintypes.LPARAM,
    )

    @EnumWindowsProc
    def _cb(hwnd: int, _lparam: int) -> bool:
        if not user32.IsWindowVisible(hwnd):
            return True
        length = user32.GetWindowTextLengthW(hwnd)
        if length > 0:
            buf = ctypes.create_unicode_buffer(length + 1)
            user32.GetWindowTextW(hwnd, buf, length + 1)
            results.append((hwnd, buf.value))
        return True

    user32.EnumWindows(_cb, 0)
    return results


def _foreground_window(
    user32,
    hwnd: int,
    log: Callable[[str], None],
) -> bool:
    """Restore + foreground *hwnd* using ShowWindow then SetForegroundWindow.

    Falls back to BringWindowToTop when SetForegroundWindow returns 0 (Windows
    focus-theft rules can block it silently when another app holds keyboard focus).
    Returns True if the call sequence completed without an OS error.

    Note: prefer _foreground_window_robust for interactive sessions where the
    foreground lock is actively held by another process.
    """
    try:
        user32.ShowWindow(hwnd, _SW_RESTORE)
        ok = user32.SetForegroundWindow(hwnd)
        if ok:
            log(f"foreground: SetForegroundWindow OK hwnd={hwnd:#010x}")
            return True
        # SetForegroundWindow returned 0 — no exception, but focus may not have
        # transferred.  BringWindowToTop is a weaker fallback that at least
        # ensures the window is not buried.
        log(
            f"foreground: SetForegroundWindow returned 0 — "
            f"trying BringWindowToTop hwnd={hwnd:#010x}"
        )
        user32.BringWindowToTop(hwnd)
        return True
    except OSError as exc:
        log(f"foreground: Win32 call failed hwnd={hwnd:#010x}: {exc}")
        return False


def _foreground_window_robust(
    user32,
    kernel32,
    hwnd: int,
    log: Callable[[str], None],
) -> bool:
    """Robust foreground activation using AttachThreadInput bypass + ALT key fallback.

    Windows foreground lock (SPI_GETFOREGROUNDLOCKTIMEOUT) prevents arbitrary
    processes from calling SetForegroundWindow when they are not the current
    foreground process.  PROMPT 1807 confirmed this: SetForegroundWindow returned
    0 on every tick across a 15-checkpoint vs-bot run, leaving DWM serving a
    frozen frame to both PrintWindow and Bevy RPC screenshot calls.

    This function defeats the lock with the following sequence:

    1. GetForegroundWindow + GetWindowThreadProcessId + GetCurrentThreadId —
       identify which threads are involved.

    2. AttachThreadInput (current → fg, target → fg) — attach our thread's input
       queue to the foreground thread's input queue.  Windows considers attached
       threads foreground-capable, allowing SetForegroundWindow to succeed.

    3. ShowWindow(SW_RESTORE) + SetWindowPos TOPMOST → NOTOPMOST — restore from
       minimised and bring to Z-top without making the window permanently always-
       on-top.

    4. SetForegroundWindow + SetFocus + SetActiveWindow — the actual focus
       transfer; SetFocus and SetActiveWindow are ancillary activations that
       together trigger DWM re-composition of fresh GPU frames.

    5. Synthetic ALT key (keybd_event VK_MENU down+up) — last-resort: a key
       event briefly satisfies the foreground guard independently of thread
       attachment state.  Isolated to a single down+up pair so no visible side-
       effect remains; PROMPT 1807 confirmed this is the only remaining bypass
       when AttachThreadInput alone does not help in console-process contexts.

    6. AttachThreadInput detach (in finally) — always detached to avoid
       corrupting shared input state.

    Capture continues even when all foreground attempts fail (returns False);
    the log contains enough detail to diagnose the exact failure path.

    *user32* and *kernel32* are injected for testability; pass
    ctypes.windll.user32 / ctypes.windll.kernel32 in production.
    """
    # Win32 constants
    _HWND_TOPMOST = -1
    _HWND_NOTOPMOST = -2
    _SWP_NOMOVE = 0x0002
    _SWP_NOSIZE = 0x0001
    _VK_MENU = 0x12         # ALT key virtual-key code
    _KEYEVENTF_KEYUP = 0x0002

    try:
        current_hwnd_fg = user32.GetForegroundWindow()
        current_thread = kernel32.GetCurrentThreadId()
        fg_thread = (
            user32.GetWindowThreadProcessId(current_hwnd_fg, None)
            if current_hwnd_fg
            else 0
        )
        target_thread = user32.GetWindowThreadProcessId(hwnd, None)

        log(
            f"foreground_robust: hwnd={hwnd:#010x} "
            f"current_fg={current_hwnd_fg:#010x} "
            f"fg_thread={fg_thread} target_thread={target_thread} "
            f"current_thread={current_thread}"
        )

        # Fast path: already foreground — just ensure not minimised.
        if current_hwnd_fg == hwnd:
            user32.ShowWindow(hwnd, _SW_RESTORE)
            log(f"foreground_robust: already foreground — ShowWindow SW_RESTORE OK")
            return True

        attached_fg = False
        attached_target = False

        try:
            # Step 1a: attach our thread to the foreground thread's input queue.
            if fg_thread and fg_thread != current_thread:
                ret = user32.AttachThreadInput(current_thread, fg_thread, True)
                attached_fg = bool(ret)
                log(
                    f"foreground_robust: AttachThreadInput"
                    f"(current={current_thread}->fg={fg_thread}) ret={ret}"
                )
            else:
                log(
                    f"foreground_robust: AttachThreadInput(current->fg) skipped "
                    f"(fg_thread={fg_thread} same as current or zero)"
                )

            # Step 1b: attach target window's thread to the foreground thread
            # so SetFocus on the target succeeds inside the attached context.
            if target_thread and fg_thread and target_thread != fg_thread:
                ret2 = user32.AttachThreadInput(target_thread, fg_thread, True)
                attached_target = bool(ret2)
                log(
                    f"foreground_robust: AttachThreadInput"
                    f"(target={target_thread}->fg={fg_thread}) ret={ret2}"
                )

            # Step 2: restore from minimised, then manipulate Z-order.
            user32.ShowWindow(hwnd, _SW_RESTORE)
            log(f"foreground_robust: ShowWindow(SW_RESTORE) hwnd={hwnd:#010x}")

            user32.SetWindowPos(
                hwnd, _HWND_TOPMOST, 0, 0, 0, 0, _SWP_NOMOVE | _SWP_NOSIZE
            )
            user32.SetWindowPos(
                hwnd, _HWND_NOTOPMOST, 0, 0, 0, 0, _SWP_NOMOVE | _SWP_NOSIZE
            )
            log(f"foreground_robust: SetWindowPos TOPMOST->NOTOPMOST hwnd={hwnd:#010x}")

            # Step 3: primary SetForegroundWindow — should succeed after AttachThreadInput.
            sfg_ret = user32.SetForegroundWindow(hwnd)
            log(
                f"foreground_robust: SetForegroundWindow ret={sfg_ret} "
                f"hwnd={hwnd:#010x}"
            )

            if sfg_ret:
                user32.SetFocus(hwnd)
                user32.SetActiveWindow(hwnd)
                log(
                    f"foreground_robust: SetFocus+SetActiveWindow OK "
                    f"hwnd={hwnd:#010x}"
                )
                return True

            # Step 4: synthetic ALT key fallback.
            log(
                "foreground_robust: SetForegroundWindow failed after AttachThreadInput"
                " — trying synthetic ALT key fallback"
            )
            user32.keybd_event(_VK_MENU, 0, 0, 0)                   # ALT down
            user32.keybd_event(_VK_MENU, 0, _KEYEVENTF_KEYUP, 0)    # ALT up
            sfg_ret2 = user32.SetForegroundWindow(hwnd)
            log(
                f"foreground_robust: SetForegroundWindow after ALT key "
                f"ret={sfg_ret2} hwnd={hwnd:#010x}"
            )

            if sfg_ret2:
                user32.SetFocus(hwnd)
                user32.SetActiveWindow(hwnd)
                log(
                    f"foreground_robust: SetFocus+SetActiveWindow OK (via ALT) "
                    f"hwnd={hwnd:#010x}"
                )
                return True

            # All attempts exhausted — capture proceeds with stale DWM frame.
            log(
                f"foreground_robust: all foreground attempts failed hwnd={hwnd:#010x}"
                f" — capture proceeds with last DWM-composited frame"
            )
            return False

        finally:
            # Always detach to avoid corrupting shared input state.
            if attached_fg:
                user32.AttachThreadInput(current_thread, fg_thread, False)
                log(
                    f"foreground_robust: AttachThreadInput detach"
                    f"(current={current_thread}->fg={fg_thread})"
                )
            if attached_target:
                user32.AttachThreadInput(target_thread, fg_thread, False)
                log(
                    f"foreground_robust: AttachThreadInput detach"
                    f"(target={target_thread}->fg={fg_thread})"
                )

    except OSError as exc:
        log(f"foreground_robust: Win32 call failed hwnd={hwnd:#010x}: {exc}")
        return False


# ---------------------------------------------------------------------------
# Public entry point
# ---------------------------------------------------------------------------

def ensure_foreground(log: Callable[[str], None]) -> None:
    """Foreground the CCGS/Bevy primary window before a screenshot is captured.

    Call this immediately before issuing an autoplay/screenshot RPC so the Bevy
    window's GPU backbuffer is actively composited rather than stale.

    No-op on non-Windows platforms or when no matching window can be identified.
    Every branch emits at least one log line so driver.log captures the outcome.

    Uses _foreground_window_robust (AttachThreadInput bypass + ALT key fallback)
    to defeat the Windows foreground lock that caused SetForegroundWindow to
    return 0 on every tick (diagnosed in PROMPT 1807).

    When no matching window is found, emits a bounded diagnostic listing of all
    visible top-level window titles so the mismatch can be diagnosed without a
    live debug session.
    """
    if not _IS_WINDOWS:
        log("foreground: non-Windows platform — no-op")
        return
    try:
        import ctypes  # noqa: PLC0415
        user32 = ctypes.windll.user32  # type: ignore[attr-defined]
        kernel32 = ctypes.windll.kernel32  # type: ignore[attr-defined]
        windows = _list_visible_windows(user32)
        candidate = _find_candidate(windows)
        if candidate is None:
            diag = _format_diag_titles(windows)
            log(
                f"foreground: no CCGS/Bevy window found among "
                f"{len(windows)} visible top-level windows — "
                f"hints={list(_WINDOW_TITLE_HINTS)!r} — "
                f"visible titles: {diag}"
            )
            return
        hwnd, title = candidate
        log(f"foreground: matched window title={title!r} hwnd={hwnd:#010x}")
        _foreground_window_robust(user32, kernel32, hwnd, log)
    except Exception as exc:  # noqa: BLE001
        log(f"foreground: unexpected error — no-op: {exc}")
