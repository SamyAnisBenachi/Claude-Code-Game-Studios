"""Unit tests for tools/autoplay/win_foreground.py (PROMPT 1776, PROMPT 1786, PROMPT 1808).

All tests run without a GUI, without a live Bevy client, and without Cargo.
ctypes boundaries are mocked via unittest.mock so the suite passes headlessly.

Run with:
    pytest tests/tools/autoplay/test_win_foreground.py -v
"""
from __future__ import annotations

import sys
from pathlib import Path
from unittest.mock import MagicMock, call, patch

import pytest

# Make tools/autoplay importable without installing.
_TOOLS_AUTOPLAY = Path(__file__).resolve().parents[3] / "tools" / "autoplay"
if str(_TOOLS_AUTOPLAY) not in sys.path:
    sys.path.insert(0, str(_TOOLS_AUTOPLAY))

import win_foreground  # noqa: E402
from win_foreground import (  # noqa: E402
    _WINDOW_TITLE_HINTS,
    _find_candidate,
    _foreground_window,
    _foreground_window_robust,
    _format_diag_titles,
    ensure_foreground,
)


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _log() -> tuple[list[str], object]:
    """Return (lines, log_fn) where log_fn appends to lines."""
    lines: list[str] = []
    return lines, lines.append


def _make_kernel32(*, current_thread_id: int = 12345) -> MagicMock:
    """Return a kernel32 mock with GetCurrentThreadId wired up."""
    k = MagicMock()
    k.GetCurrentThreadId.return_value = current_thread_id
    return k


# ---------------------------------------------------------------------------
# 1. _find_candidate — pure Python, no ctypes
# ---------------------------------------------------------------------------

class TestFindCandidate:
    def test_win_foreground_find_candidate_returns_none_for_empty_list(self):
        assert _find_candidate([]) is None

    def test_win_foreground_find_candidate_returns_none_when_no_match(self):
        windows = [(0x1001, "Notepad"), (0x1002, "Calculator"), (0x1003, "Task Manager")]
        assert _find_candidate(windows) is None

    def test_win_foreground_find_candidate_matches_ccgs_title(self):
        windows = [(0x1001, "CCGS Game Client"), (0x1002, "Notepad")]
        result = _find_candidate(windows)
        assert result == (0x1001, "CCGS Game Client")

    def test_win_foreground_find_candidate_matches_bevy_app_title(self):
        windows = [(0x1001, "Notepad"), (0x1002, "Bevy App")]
        result = _find_candidate(windows)
        assert result == (0x1002, "Bevy App")

    def test_win_foreground_find_candidate_case_insensitive_ccgs(self):
        windows = [(0x1001, "CCGS CLIENT"), (0x1002, "Notepad")]
        result = _find_candidate(windows)
        assert result is not None
        assert result[0] == 0x1001

    def test_win_foreground_find_candidate_case_insensitive_bevy(self):
        windows = [(0x1001, "BEVY APP")]
        result = _find_candidate(windows)
        assert result is not None

    def test_win_foreground_find_candidate_returns_first_match(self):
        # Both match — first one must be returned.
        windows = [(0x1001, "CCGS Window A"), (0x1002, "CCGS Window B")]
        result = _find_candidate(windows)
        assert result == (0x1001, "CCGS Window A")

    def test_win_foreground_find_candidate_substring_match(self):
        # "ccgs" appears inside a longer title string.
        windows = [(0x1001, "My CCGS Debug Overlay")]
        result = _find_candidate(windows)
        assert result is not None
        assert result[0] == 0x1001

    def test_win_foreground_find_candidate_ignores_non_matching(self):
        # Verify that unrelated titles do not match any hint.
        non_matching = [
            (0x1001, "Visual Studio Code"),
            (0x1002, "Windows Terminal"),
            (0x1003, "Chrome"),
            (0x1004, "File Explorer"),
        ]
        assert _find_candidate(non_matching) is None

    def test_win_foreground_find_candidate_matches_lanes_and_lies_title(self):
        # Regression: the actual client title from client/src/main.rs was
        # "Lanes and Lies" but the hints list previously only had "ccgs"/"bevy"
        # substrings, causing every screenshot foreground attempt to miss.
        windows = [(0x3001, "Lanes and Lies"), (0x3002, "Notepad")]
        result = _find_candidate(windows)
        assert result == (0x3001, "Lanes and Lies")

    def test_win_foreground_find_candidate_matches_lanes_and_lies_case_insensitive(self):
        windows = [(0x3001, "LANES AND LIES")]
        result = _find_candidate(windows)
        assert result is not None
        assert result[0] == 0x3001

    def test_win_foreground_find_candidate_matches_lanes_substring(self):
        # "lanes" hint covers hypothetical shorter title variants.
        windows = [(0x3001, "Lanes — Debug")]
        result = _find_candidate(windows)
        assert result is not None

    def test_win_foreground_hints_constant_is_not_empty(self):
        assert len(_WINDOW_TITLE_HINTS) >= 1
        assert all(isinstance(h, str) and h for h in _WINDOW_TITLE_HINTS)

    def test_win_foreground_hints_contains_lanes_and_lies(self):
        assert "lanes and lies" in _WINDOW_TITLE_HINTS


# ---------------------------------------------------------------------------
# 1b. _format_diag_titles — pure Python diagnostic helper (PROMPT 1786)
# ---------------------------------------------------------------------------

class TestFormatDiagTitles:
    def test_win_foreground_diag_titles_empty_list(self):
        result = _format_diag_titles([])
        assert result == "[]"

    def test_win_foreground_diag_titles_single_window(self):
        result = _format_diag_titles([(1, "Notepad")])
        assert "Notepad" in result

    def test_win_foreground_diag_titles_respects_limit(self):
        windows = [(i, f"Window {i}") for i in range(50)]
        result = _format_diag_titles(windows, limit=10)
        assert "(+40 more)" in result

    def test_win_foreground_diag_titles_no_suffix_when_under_limit(self):
        windows = [(i, f"App {i}") for i in range(5)]
        result = _format_diag_titles(windows, limit=10)
        assert "more" not in result

    def test_win_foreground_diag_titles_truncates_long_titles(self):
        long_title = "A" * 100
        result = _format_diag_titles([(1, long_title)], limit=10)
        # Truncated to 60 chars, shown inside repr()
        assert "A" * 60 in result
        assert "A" * 61 not in result

    def test_win_foreground_diag_titles_returns_string(self):
        result = _format_diag_titles([(1, "Chrome"), (2, "Explorer")])
        assert isinstance(result, str)


# ---------------------------------------------------------------------------
# 2. _foreground_window — mocked user32 (no real Win32 calls)
# ---------------------------------------------------------------------------

class TestForegroundWindow:
    def _make_user32(self, *, setforeground_ok: int = 1) -> MagicMock:
        u = MagicMock()
        u.ShowWindow.return_value = 1
        u.SetForegroundWindow.return_value = setforeground_ok
        u.BringWindowToTop.return_value = 1
        return u

    def test_win_foreground_window_calls_show_window(self):
        user32 = self._make_user32()
        lines, log = _log()
        _foreground_window(user32, 0x1001, log)
        user32.ShowWindow.assert_called_once_with(0x1001, 9)  # _SW_RESTORE = 9

    def test_win_foreground_window_calls_set_foreground_window(self):
        user32 = self._make_user32()
        lines, log = _log()
        _foreground_window(user32, 0x1001, log)
        user32.SetForegroundWindow.assert_called_once_with(0x1001)

    def test_win_foreground_window_returns_true_when_set_ok(self):
        user32 = self._make_user32(setforeground_ok=1)
        lines, log = _log()
        result = _foreground_window(user32, 0x1001, log)
        assert result is True

    def test_win_foreground_window_logs_ok_when_set_succeeds(self):
        user32 = self._make_user32(setforeground_ok=1)
        lines, log = _log()
        _foreground_window(user32, 0x1001, log)
        assert any("SetForegroundWindow OK" in line for line in lines)

    def test_win_foreground_window_falls_back_to_bring_on_zero(self):
        # SetForegroundWindow returns 0 → should call BringWindowToTop.
        user32 = self._make_user32(setforeground_ok=0)
        lines, log = _log()
        result = _foreground_window(user32, 0x1001, log)
        user32.BringWindowToTop.assert_called_once_with(0x1001)
        assert result is True

    def test_win_foreground_window_logs_fallback_on_zero(self):
        user32 = self._make_user32(setforeground_ok=0)
        lines, log = _log()
        _foreground_window(user32, 0x1001, log)
        assert any("BringWindowToTop" in line for line in lines)

    def test_win_foreground_window_returns_false_on_os_error(self):
        user32 = self._make_user32()
        user32.ShowWindow.side_effect = OSError("access denied")
        lines, log = _log()
        result = _foreground_window(user32, 0x1001, log)
        assert result is False

    def test_win_foreground_window_logs_error_on_os_error(self):
        user32 = self._make_user32()
        user32.SetForegroundWindow.side_effect = OSError("blocked by focus rules")
        lines, log = _log()
        _foreground_window(user32, 0x1001, log)
        assert any("Win32 call failed" in line for line in lines)


# ---------------------------------------------------------------------------
# 3. _foreground_window_robust — PROMPT 1808 AttachThreadInput bypass
# ---------------------------------------------------------------------------

class TestForegroundWindowRobust:
    """Tests for the PROMPT 1808 robust foreground bypass function.

    All Win32 API calls are mocked — no real GUI or ctypes calls occur.
    """

    def _make_user32(
        self,
        *,
        current_fg_hwnd: int = 0xAA00,
        fg_thread: int = 99,
        target_thread: int = 77,
        setforeground_ok: int = 1,
        attach_ok: int = 1,
    ) -> MagicMock:
        u = MagicMock()
        u.GetForegroundWindow.return_value = current_fg_hwnd
        u.GetWindowThreadProcessId.side_effect = [fg_thread, target_thread]
        u.AttachThreadInput.return_value = attach_ok
        u.ShowWindow.return_value = 1
        u.SetWindowPos.return_value = 1
        u.SetForegroundWindow.return_value = setforeground_ok
        u.SetFocus.return_value = 1
        u.SetActiveWindow.return_value = 1
        u.keybd_event.return_value = None
        u.BringWindowToTop.return_value = 1
        return u

    def _make_kernel32(self, *, thread_id: int = 42) -> MagicMock:
        k = MagicMock()
        k.GetCurrentThreadId.return_value = thread_id
        return k

    # --- fast path: already foreground ---

    def test_win_foreground_robust_already_foreground_returns_true(self):
        # Arrange: GetForegroundWindow returns same hwnd as target
        hwnd = 0x1001
        user32 = self._make_user32(current_fg_hwnd=hwnd)
        user32.GetWindowThreadProcessId.side_effect = None  # reset side_effect
        user32.GetWindowThreadProcessId.return_value = 42
        kernel32 = self._make_kernel32()
        lines, log = _log()

        # Act
        result = _foreground_window_robust(user32, kernel32, hwnd, log)

        # Assert
        assert result is True
        assert any("already foreground" in l for l in lines)

    def test_win_foreground_robust_already_foreground_calls_show_window(self):
        # Arrange
        hwnd = 0x2001
        user32 = self._make_user32(current_fg_hwnd=hwnd)
        user32.GetWindowThreadProcessId.side_effect = None
        user32.GetWindowThreadProcessId.return_value = 10
        kernel32 = self._make_kernel32()
        lines, log = _log()

        # Act
        _foreground_window_robust(user32, kernel32, hwnd, log)

        # Assert: ShowWindow must still be called with SW_RESTORE=9
        user32.ShowWindow.assert_called_once_with(hwnd, 9)

    # --- AttachThreadInput sequence ---

    def test_win_foreground_robust_attaches_to_fg_thread(self):
        # Arrange
        hwnd = 0x3001
        fg_thread = 99
        current_thread = 42
        user32 = self._make_user32(
            current_fg_hwnd=0xAA00, fg_thread=fg_thread, target_thread=77
        )
        kernel32 = self._make_kernel32(thread_id=current_thread)
        lines, log = _log()

        # Act
        _foreground_window_robust(user32, kernel32, hwnd, log)

        # Assert: AttachThreadInput(current, fg, True) must be called
        attach_calls = user32.AttachThreadInput.call_args_list
        assert any(
            c == call(current_thread, fg_thread, True) for c in attach_calls
        ), f"Expected AttachThreadInput({current_thread}, {fg_thread}, True) — got {attach_calls}"

    def test_win_foreground_robust_detaches_fg_thread_in_finally(self):
        # Arrange: AttachThreadInput(current->fg) succeeds
        hwnd = 0x3001
        fg_thread = 99
        current_thread = 42
        user32 = self._make_user32(
            current_fg_hwnd=0xAA00, fg_thread=fg_thread, target_thread=77, attach_ok=1
        )
        kernel32 = self._make_kernel32(thread_id=current_thread)
        lines, log = _log()

        # Act
        _foreground_window_robust(user32, kernel32, hwnd, log)

        # Assert: AttachThreadInput(current, fg, False) must be called to detach
        attach_calls = user32.AttachThreadInput.call_args_list
        assert any(
            c == call(current_thread, fg_thread, False) for c in attach_calls
        ), f"Expected detach call — got {attach_calls}"

    def test_win_foreground_robust_detaches_even_when_setforeground_fails(self):
        # Arrange: SetForegroundWindow always returns 0
        hwnd = 0x3001
        fg_thread = 99
        current_thread = 42
        user32 = self._make_user32(
            current_fg_hwnd=0xAA00, fg_thread=fg_thread, target_thread=77,
            setforeground_ok=0, attach_ok=1
        )
        # keybd_event doesn't help either — second SetForegroundWindow also fails
        user32.SetForegroundWindow.return_value = 0
        kernel32 = self._make_kernel32(thread_id=current_thread)
        lines, log = _log()

        # Act
        _foreground_window_robust(user32, kernel32, hwnd, log)

        # Assert: detach must happen regardless of SetForegroundWindow result
        attach_calls = user32.AttachThreadInput.call_args_list
        assert any(
            c == call(current_thread, fg_thread, False) for c in attach_calls
        ), "Detach call must occur in finally even when foreground fails"

    def test_win_foreground_robust_attaches_target_thread_to_fg(self):
        # Arrange: target_thread differs from fg_thread
        hwnd = 0x3001
        fg_thread = 99
        target_thread = 77
        current_thread = 42
        user32 = self._make_user32(
            current_fg_hwnd=0xAA00, fg_thread=fg_thread, target_thread=target_thread
        )
        kernel32 = self._make_kernel32(thread_id=current_thread)
        lines, log = _log()

        # Act
        _foreground_window_robust(user32, kernel32, hwnd, log)

        # Assert: AttachThreadInput(target, fg, True) must be called
        attach_calls = user32.AttachThreadInput.call_args_list
        assert any(
            c == call(target_thread, fg_thread, True) for c in attach_calls
        ), f"Expected target->fg attach — got {attach_calls}"

    # --- SetWindowPos TOPMOST/NOTOPMOST ---

    def test_win_foreground_robust_calls_setwindowpos_topmost_then_notopmost(self):
        # Arrange
        hwnd = 0x4001
        user32 = self._make_user32(current_fg_hwnd=0xAA00, fg_thread=99, target_thread=77)
        kernel32 = self._make_kernel32()
        lines, log = _log()
        swp_order: list[int] = []

        def _track_swp(h, z_order, *args):
            swp_order.append(z_order)
            return 1

        user32.SetWindowPos.side_effect = _track_swp

        # Act
        _foreground_window_robust(user32, kernel32, hwnd, log)

        # Assert: TOPMOST (-1) must precede NOTOPMOST (-2)
        assert swp_order == [-1, -2], (
            f"Expected TOPMOST→NOTOPMOST sequence but got {swp_order}"
        )

    # --- SetForegroundWindow success path ---

    def test_win_foreground_robust_returns_true_when_setforeground_succeeds(self):
        # Arrange
        hwnd = 0x5001
        user32 = self._make_user32(
            current_fg_hwnd=0xAA00, fg_thread=99, target_thread=77, setforeground_ok=1
        )
        kernel32 = self._make_kernel32()
        lines, log = _log()

        # Act
        result = _foreground_window_robust(user32, kernel32, hwnd, log)

        # Assert
        assert result is True

    def test_win_foreground_robust_calls_setfocus_on_success(self):
        # Arrange
        hwnd = 0x5001
        user32 = self._make_user32(
            current_fg_hwnd=0xAA00, fg_thread=99, target_thread=77, setforeground_ok=1
        )
        kernel32 = self._make_kernel32()
        lines, log = _log()

        # Act
        _foreground_window_robust(user32, kernel32, hwnd, log)

        # Assert
        user32.SetFocus.assert_called_once_with(hwnd)

    def test_win_foreground_robust_calls_setactivewindow_on_success(self):
        # Arrange
        hwnd = 0x5001
        user32 = self._make_user32(
            current_fg_hwnd=0xAA00, fg_thread=99, target_thread=77, setforeground_ok=1
        )
        kernel32 = self._make_kernel32()
        lines, log = _log()

        # Act
        _foreground_window_robust(user32, kernel32, hwnd, log)

        # Assert
        user32.SetActiveWindow.assert_called_once_with(hwnd)

    def test_win_foreground_robust_logs_setforeground_return_value(self):
        # Arrange
        hwnd = 0x5001
        user32 = self._make_user32(
            current_fg_hwnd=0xAA00, fg_thread=99, target_thread=77, setforeground_ok=1
        )
        kernel32 = self._make_kernel32()
        lines, log = _log()

        # Act
        _foreground_window_robust(user32, kernel32, hwnd, log)

        # Assert
        assert any("SetForegroundWindow ret=" in l for l in lines), (
            "SetForegroundWindow return value must be logged for diagnosis"
        )

    # --- ALT key fallback ---

    def test_win_foreground_robust_tries_alt_key_when_setforeground_fails(self):
        # Arrange: first SetForegroundWindow fails, ALT key sent, second succeeds
        hwnd = 0x6001
        user32 = self._make_user32(
            current_fg_hwnd=0xAA00, fg_thread=99, target_thread=77
        )
        user32.SetForegroundWindow.side_effect = [0, 1]  # fail, then succeed
        kernel32 = self._make_kernel32()
        lines, log = _log()

        # Act
        result = _foreground_window_robust(user32, kernel32, hwnd, log)

        # Assert: keybd_event was called (ALT down + ALT up = 2 calls)
        assert user32.keybd_event.call_count == 2, (
            f"Expected 2 keybd_event calls (ALT down+up) — got {user32.keybd_event.call_count}"
        )
        assert result is True

    def test_win_foreground_robust_alt_key_sends_vk_menu(self):
        # Arrange: SetForegroundWindow fails first time, succeeds after ALT
        hwnd = 0x6001
        user32 = self._make_user32(
            current_fg_hwnd=0xAA00, fg_thread=99, target_thread=77
        )
        user32.SetForegroundWindow.side_effect = [0, 1]
        kernel32 = self._make_kernel32()
        lines, log = _log()

        # Act
        _foreground_window_robust(user32, kernel32, hwnd, log)

        # Assert: first keybd_event must use VK_MENU=0x12
        first_call = user32.keybd_event.call_args_list[0]
        assert first_call.args[0] == 0x12, (
            f"ALT key fallback must use VK_MENU=0x12 — got {first_call.args[0]:#x}"
        )

    def test_win_foreground_robust_logs_alt_key_fallback_attempt(self):
        # Arrange: primary SetForegroundWindow fails
        hwnd = 0x6001
        user32 = self._make_user32(
            current_fg_hwnd=0xAA00, fg_thread=99, target_thread=77
        )
        user32.SetForegroundWindow.side_effect = [0, 1]
        kernel32 = self._make_kernel32()
        lines, log = _log()

        # Act
        _foreground_window_robust(user32, kernel32, hwnd, log)

        # Assert
        assert any("ALT" in l for l in lines), (
            "ALT key fallback must be logged when primary SetForegroundWindow fails"
        )

    def test_win_foreground_robust_returns_false_when_all_attempts_fail(self):
        # Arrange: both SetForegroundWindow calls return 0
        hwnd = 0x7001
        user32 = self._make_user32(
            current_fg_hwnd=0xAA00, fg_thread=99, target_thread=77
        )
        user32.SetForegroundWindow.return_value = 0  # always fails
        kernel32 = self._make_kernel32()
        lines, log = _log()

        # Act
        result = _foreground_window_robust(user32, kernel32, hwnd, log)

        # Assert
        assert result is False
        assert any("all foreground attempts failed" in l for l in lines)

    def test_win_foreground_robust_capture_proceeds_when_all_fail(self):
        # Regression: PROMPT 1807 — capture must not abort even when focus fails.
        # _foreground_window_robust must return False (not raise) so driver continues.
        hwnd = 0x7001
        user32 = self._make_user32(
            current_fg_hwnd=0xAA00, fg_thread=99, target_thread=77
        )
        user32.SetForegroundWindow.return_value = 0
        kernel32 = self._make_kernel32()
        lines, log = _log()

        # Act — must not raise
        result = _foreground_window_robust(user32, kernel32, hwnd, log)

        # Assert: False returned, no exception
        assert result is False

    # --- OSError handling ---

    def test_win_foreground_robust_returns_false_on_os_error(self):
        # Arrange: GetForegroundWindow raises OSError
        hwnd = 0x8001
        user32 = MagicMock()
        user32.GetForegroundWindow.side_effect = OSError("access denied")
        kernel32 = self._make_kernel32()
        lines, log = _log()

        # Act — must not propagate
        result = _foreground_window_robust(user32, kernel32, hwnd, log)

        # Assert
        assert result is False
        assert any("Win32 call failed" in l for l in lines)

    def test_win_foreground_robust_logs_thread_ids_for_diagnosis(self):
        # Arrange
        hwnd = 0x9001
        user32 = self._make_user32(
            current_fg_hwnd=0xAA00, fg_thread=99, target_thread=77, setforeground_ok=1
        )
        kernel32 = self._make_kernel32(thread_id=42)
        lines, log = _log()

        # Act
        _foreground_window_robust(user32, kernel32, hwnd, log)

        # Assert: the diagnostic header must include fg_thread and current_thread
        first_line = lines[0] if lines else ""
        assert "fg_thread=99" in first_line, (
            f"fg_thread must appear in the first log line — got: {first_line!r}"
        )
        assert "current_thread=42" in first_line, (
            f"current_thread must appear in the first log line — got: {first_line!r}"
        )

    # --- attach skipped when threads are the same ---

    def test_win_foreground_robust_skips_attach_when_fg_thread_equals_current(self):
        # Arrange: fg_thread == current_thread → no AttachThreadInput needed
        hwnd = 0xA001
        same_thread = 42
        user32 = self._make_user32(
            current_fg_hwnd=0xAA00, fg_thread=same_thread, target_thread=77
        )
        kernel32 = self._make_kernel32(thread_id=same_thread)
        lines, log = _log()

        # Act
        _foreground_window_robust(user32, kernel32, hwnd, log)

        # Assert: AttachThreadInput should NOT be called with current→fg (same thread)
        attach_calls = user32.AttachThreadInput.call_args_list
        bad_calls = [c for c in attach_calls if c == call(same_thread, same_thread, True)]
        assert not bad_calls, (
            "AttachThreadInput(same, same, True) must not be called when threads are identical"
        )

    def test_win_foreground_robust_skips_attach_when_fg_hwnd_is_zero(self):
        # Arrange: GetForegroundWindow returns 0 (no foreground window)
        hwnd = 0xA001
        user32 = self._make_user32(current_fg_hwnd=0, fg_thread=0, target_thread=77)
        user32.GetWindowThreadProcessId.side_effect = [0, 77]
        kernel32 = self._make_kernel32()
        lines, log = _log()

        # Act — must not raise
        result = _foreground_window_robust(user32, kernel32, hwnd, log)

        # Assert: no crash; AttachThreadInput(current, 0, True) must not be called
        attach_calls = user32.AttachThreadInput.call_args_list
        bad_calls = [c for c in attach_calls if c.args[1] == 0 and c.args[2] is True]
        assert not bad_calls, (
            "AttachThreadInput must not be called with a zero fg_thread"
        )


# ---------------------------------------------------------------------------
# 4. ensure_foreground — monkeypatched _IS_WINDOWS + mocked internals
# ---------------------------------------------------------------------------

class TestEnsureForeground:
    def test_win_foreground_ensure_noop_on_non_windows(self, monkeypatch):
        monkeypatch.setattr(win_foreground, "_IS_WINDOWS", False)
        lines, log = _log()
        ensure_foreground(log)
        assert any("non-Windows" in line or "no-op" in line for line in lines)

    def test_win_foreground_ensure_emits_at_least_one_log_line_non_windows(self, monkeypatch):
        monkeypatch.setattr(win_foreground, "_IS_WINDOWS", False)
        lines, log = _log()
        ensure_foreground(log)
        assert len(lines) >= 1

    def test_win_foreground_ensure_logs_no_window_found(self, monkeypatch):
        monkeypatch.setattr(win_foreground, "_IS_WINDOWS", True)
        monkeypatch.setattr(win_foreground, "_list_visible_windows", lambda _u: [])
        with patch("ctypes.windll") as mock_windll:
            mock_windll.user32 = MagicMock()
            mock_windll.kernel32 = MagicMock()
            lines, log = _log()
            ensure_foreground(log)
        assert any("no CCGS/Bevy window found" in line for line in lines)

    def test_win_foreground_ensure_calls_robust_foreground_when_found(self, monkeypatch):
        # PROMPT 1808: ensure_foreground must call _foreground_window_robust,
        # not the basic _foreground_window, so the AttachThreadInput bypass is used.
        monkeypatch.setattr(win_foreground, "_IS_WINDOWS", True)
        fake_windows = [(0x2001, "CCGS Client Window")]
        monkeypatch.setattr(win_foreground, "_list_visible_windows", lambda _u: fake_windows)

        called_with: list[tuple] = []

        def _fake_robust(user32, kernel32, hwnd, log):
            called_with.append((hwnd,))
            return True

        monkeypatch.setattr(win_foreground, "_foreground_window_robust", _fake_robust)

        with patch("ctypes.windll") as mock_windll:
            mock_windll.user32 = MagicMock()
            mock_windll.kernel32 = MagicMock()
            lines, log = _log()
            ensure_foreground(log)

        assert called_with == [(0x2001,)], (
            "ensure_foreground must delegate to _foreground_window_robust "
            f"with the matched hwnd — got {called_with}"
        )

    def test_win_foreground_ensure_logs_matched_title(self, monkeypatch):
        monkeypatch.setattr(win_foreground, "_IS_WINDOWS", True)
        fake_windows = [(0x2001, "CCGS Debug Client")]
        monkeypatch.setattr(win_foreground, "_list_visible_windows", lambda _u: fake_windows)
        monkeypatch.setattr(win_foreground, "_foreground_window_robust", lambda *a: True)

        with patch("ctypes.windll") as mock_windll:
            mock_windll.user32 = MagicMock()
            mock_windll.kernel32 = MagicMock()
            lines, log = _log()
            ensure_foreground(log)

        assert any("CCGS Debug Client" in line for line in lines)

    def test_win_foreground_ensure_swallows_unexpected_exceptions(self, monkeypatch):
        monkeypatch.setattr(win_foreground, "_IS_WINDOWS", True)

        def _boom(_u):
            raise RuntimeError("simulated ctypes failure")

        monkeypatch.setattr(win_foreground, "_list_visible_windows", _boom)

        with patch("ctypes.windll") as mock_windll:
            mock_windll.user32 = MagicMock()
            mock_windll.kernel32 = MagicMock()
            lines, log = _log()
            # Must not raise.
            ensure_foreground(log)

        assert any("unexpected error" in line for line in lines)

    def test_win_foreground_ensure_logs_visible_window_count_when_no_match(self, monkeypatch):
        monkeypatch.setattr(win_foreground, "_IS_WINDOWS", True)
        many_windows = [(i, f"App {i}") for i in range(7)]
        monkeypatch.setattr(win_foreground, "_list_visible_windows", lambda _u: many_windows)

        with patch("ctypes.windll") as mock_windll:
            mock_windll.user32 = MagicMock()
            mock_windll.kernel32 = MagicMock()
            lines, log = _log()
            ensure_foreground(log)

        assert any("7" in line for line in lines)

    def test_win_foreground_ensure_logs_hints_when_no_match(self, monkeypatch):
        # PROMPT 1786: no-match log line must include the hints list so the
        # mismatch is diagnosable without re-reading the source.
        monkeypatch.setattr(win_foreground, "_IS_WINDOWS", True)
        monkeypatch.setattr(win_foreground, "_list_visible_windows", lambda _u: [])

        with patch("ctypes.windll") as mock_windll:
            mock_windll.user32 = MagicMock()
            mock_windll.kernel32 = MagicMock()
            lines, log = _log()
            ensure_foreground(log)

        assert any("hints=" in line for line in lines)

    def test_win_foreground_ensure_logs_visible_titles_when_no_match(self, monkeypatch):
        # PROMPT 1786: no-match log line must include the visible title list.
        monkeypatch.setattr(win_foreground, "_IS_WINDOWS", True)
        fake_windows = [(0x5001, "Notepad"), (0x5002, "Chrome")]
        monkeypatch.setattr(win_foreground, "_list_visible_windows", lambda _u: fake_windows)

        with patch("ctypes.windll") as mock_windll:
            mock_windll.user32 = MagicMock()
            mock_windll.kernel32 = MagicMock()
            lines, log = _log()
            ensure_foreground(log)

        assert any("Notepad" in line for line in lines)

    def test_win_foreground_ensure_matches_lanes_and_lies_window(self, monkeypatch):
        # Regression: "Lanes and Lies" must now be matched and foregrounded via robust path.
        monkeypatch.setattr(win_foreground, "_IS_WINDOWS", True)
        fake_windows = [(0x6001, "Lanes and Lies")]
        monkeypatch.setattr(win_foreground, "_list_visible_windows", lambda _u: fake_windows)

        called_with: list[tuple] = []

        def _fake_robust(user32, kernel32, hwnd, log):
            called_with.append((hwnd,))
            return True

        monkeypatch.setattr(win_foreground, "_foreground_window_robust", _fake_robust)

        with patch("ctypes.windll") as mock_windll:
            mock_windll.user32 = MagicMock()
            mock_windll.kernel32 = MagicMock()
            lines, log = _log()
            ensure_foreground(log)

        assert called_with == [(0x6001,)]

    def test_win_foreground_ensure_passes_kernel32_to_robust(self, monkeypatch):
        # PROMPT 1808: ensure_foreground must supply kernel32 to _foreground_window_robust
        # so GetCurrentThreadId is available for AttachThreadInput bypass.
        monkeypatch.setattr(win_foreground, "_IS_WINDOWS", True)
        fake_windows = [(0xB001, "Lanes and Lies")]
        monkeypatch.setattr(win_foreground, "_list_visible_windows", lambda _u: fake_windows)

        received_kernel32: list = []

        def _fake_robust(user32, kernel32, hwnd, log):
            received_kernel32.append(kernel32)
            return True

        monkeypatch.setattr(win_foreground, "_foreground_window_robust", _fake_robust)

        fake_kernel32 = MagicMock()
        with patch("ctypes.windll") as mock_windll:
            mock_windll.user32 = MagicMock()
            mock_windll.kernel32 = fake_kernel32
            lines, log = _log()
            ensure_foreground(log)

        assert received_kernel32 == [fake_kernel32], (
            "ensure_foreground must pass ctypes.windll.kernel32 to _foreground_window_robust"
        )
