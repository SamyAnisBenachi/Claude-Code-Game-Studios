"""Windows-only helper: foreground the primary Bevy/CCGS window before screenshot capture.

Imported by driver.py and called before each autoplay/screenshot RPC so the Bevy
window is the active composited surface when Screenshot::primary_window() fires.
No-op on non-Windows. No third-party dependencies — stdlib ctypes only.

PROMPT 1776: foreground repair for screenshot distinctness.
PROMPT 1786: window title diagnostic — added "lanes and lies" hint (actual client title
             from client/src/main.rs WindowPlugin), extended hints list, and bounded
             diagnostic logging of visible titles when no match is found.
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


# ---------------------------------------------------------------------------
# Public entry point
# ---------------------------------------------------------------------------

def ensure_foreground(log: Callable[[str], None]) -> None:
    """Foreground the CCGS/Bevy primary window before a screenshot is captured.

    Call this immediately before issuing an autoplay/screenshot RPC so the Bevy
    window's GPU backbuffer is actively composited rather than stale.

    No-op on non-Windows platforms or when no matching window can be identified.
    Every branch emits at least one log line so driver.log captures the outcome.

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
        _foreground_window(user32, hwnd, log)
    except Exception as exc:  # noqa: BLE001
        log(f"foreground: unexpected error — no-op: {exc}")
