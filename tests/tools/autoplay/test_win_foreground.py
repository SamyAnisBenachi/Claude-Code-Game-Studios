"""Unit tests for tools/autoplay/win_foreground.py (PROMPT 1776, PROMPT 1786).

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
# 3. ensure_foreground — monkeypatched _IS_WINDOWS + mocked internals
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
            lines, log = _log()
            ensure_foreground(log)
        assert any("no CCGS/Bevy window found" in line for line in lines)

    def test_win_foreground_ensure_calls_foreground_window_when_found(self, monkeypatch):
        monkeypatch.setattr(win_foreground, "_IS_WINDOWS", True)
        fake_windows = [(0x2001, "CCGS Client Window")]
        monkeypatch.setattr(win_foreground, "_list_visible_windows", lambda _u: fake_windows)

        called_with: list[tuple] = []

        def _fake_foreground(user32, hwnd, log):
            called_with.append((hwnd,))
            return True

        monkeypatch.setattr(win_foreground, "_foreground_window", _fake_foreground)

        with patch("ctypes.windll") as mock_windll:
            mock_windll.user32 = MagicMock()
            lines, log = _log()
            ensure_foreground(log)

        assert called_with == [(0x2001,)]

    def test_win_foreground_ensure_logs_matched_title(self, monkeypatch):
        monkeypatch.setattr(win_foreground, "_IS_WINDOWS", True)
        fake_windows = [(0x2001, "CCGS Debug Client")]
        monkeypatch.setattr(win_foreground, "_list_visible_windows", lambda _u: fake_windows)
        monkeypatch.setattr(win_foreground, "_foreground_window", lambda *a: True)

        with patch("ctypes.windll") as mock_windll:
            mock_windll.user32 = MagicMock()
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
            lines, log = _log()
            ensure_foreground(log)

        assert any("Notepad" in line for line in lines)

    def test_win_foreground_ensure_matches_lanes_and_lies_window(self, monkeypatch):
        # Regression: "Lanes and Lies" must now be matched and foregrounded.
        monkeypatch.setattr(win_foreground, "_IS_WINDOWS", True)
        fake_windows = [(0x6001, "Lanes and Lies")]
        monkeypatch.setattr(win_foreground, "_list_visible_windows", lambda _u: fake_windows)

        called_with: list[tuple] = []

        def _fake_foreground(user32, hwnd, log):
            called_with.append((hwnd,))
            return True

        monkeypatch.setattr(win_foreground, "_foreground_window", _fake_foreground)

        with patch("ctypes.windll") as mock_windll:
            mock_windll.user32 = MagicMock()
            lines, log = _log()
            ensure_foreground(log)

        assert called_with == [(0x6001,)]
