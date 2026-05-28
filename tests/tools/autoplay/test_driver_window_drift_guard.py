"""Tests for the window-drift / pre-click safety gate (PROMPT 1844).

The exact failure mode from the audit:
  Run 090613 — recipe built at window_logical_size=(1280, 720).
  At tick 115 the window was resized to (1280, 1076).
  The driver continued dispatching mouse_down/mouse_up with 720-baked coords
  and reported a clean PASS.  That must not happen.

Covers:
  _check_window_stable_for_mouseclick():
    - Returns (True, "") when all four invariants hold
    - Returns (False, ABORT WINDOW-DRIFT diag) when size drifts > 10 px
    - Drift exactly at threshold (10 px) passes; 11 px fails
    - Returns (False, ABORT WINDOW-TOO-SMALL) when height < 720
    - Returns (False, ABORT CURSOR-LOGICAL-NONE) when cursor_logical is None
    - Returns (False, ABORT CLICK-OOB-AT-MOUSEDOWN) when last cursor is OOB
    - Drift in width only triggers WINDOW-DRIFT
    - Drift in height only triggers WINDOW-DRIFT
    - last_cursor_screen=None skips the OOB check (cursor never moved)

  Structural checks (driver.py):
    - EXIT_WINDOW_MISMATCH = 5 defined
    - _check_window_stable_for_mouseclick defined
    - NEEDS_HUMAN_GUI sentinel present
    - Guard gated on mouse_down/mouse_up params
    - recipe_build_win_size tracked
    - Guard fires only after recipe_build_win_size is set
    - EXIT_WINDOW_MISMATCH breaks the tick loop (post-action check present)

No GUI, no Bevy launch, no Cargo.  Run with:
    pytest tests/tools/autoplay/test_driver_window_drift_guard.py -v
"""

from __future__ import annotations

import sys
from pathlib import Path

_TOOLS_AUTOPLAY = Path(__file__).resolve().parents[3] / "tools" / "autoplay"
if str(_TOOLS_AUTOPLAY) not in sys.path:
    sys.path.insert(0, str(_TOOLS_AUTOPLAY))

from driver import _check_window_stable_for_mouseclick, EXIT_WINDOW_MISMATCH  # noqa: E402


# ---------------------------------------------------------------------------
# _check_window_stable_for_mouseclick unit tests
# ---------------------------------------------------------------------------

class TestCheckWindowStableForMouseclick:
    """Unit tests for the pre-click stability gate."""

    _STABLE = dict(
        current_win=(1280.0, 720.0),
        build_win=(1280.0, 720.0),
        cursor_logical=[640.0, 360.0],
        last_cursor_screen=(640.0, 360.0),
    )

    def _call(self, **overrides):
        args = {**self._STABLE, **overrides}
        messages: list[str] = []
        ok, diag = _check_window_stable_for_mouseclick(
            **args,
            tick=1,
            log_fn=messages.append,
        )
        return ok, diag, messages

    # --- invariant 1: size drift ---

    def test_stable_window_returns_true(self):
        ok, diag, msgs = self._call()
        assert ok is True
        assert diag == ""
        assert msgs == []

    def test_height_drift_11px_triggers_abort(self):
        # Exact failure mode from run 090613: window became 1076 from 720
        ok, diag, msgs = self._call(current_win=(1280.0, 1076.0), build_win=(1280.0, 720.0))
        assert ok is False
        assert "WINDOW-DRIFT" in diag
        assert "ABORT" in diag
        assert "NEEDS_HUMAN_GUI" in diag
        assert len(msgs) == 1

    def test_width_drift_11px_triggers_abort(self):
        ok, diag, msgs = self._call(current_win=(1291.0, 720.0), build_win=(1280.0, 720.0))
        assert ok is False
        assert "WINDOW-DRIFT" in diag

    def test_drift_exactly_at_threshold_10px_passes(self):
        ok, diag, msgs = self._call(current_win=(1280.0, 730.0), build_win=(1280.0, 720.0))
        assert ok is True, "drift of exactly 10 px is at threshold and must pass"

    def test_drift_11px_fails(self):
        ok, diag, msgs = self._call(current_win=(1280.0, 731.0), build_win=(1280.0, 720.0))
        assert ok is False, "drift of 11 px must fail"

    def test_drift_diag_includes_build_and_current_sizes(self):
        ok, diag, msgs = self._call(current_win=(1280.0, 1076.0), build_win=(1280.0, 720.0))
        assert "1280" in diag
        assert "720" in diag
        assert "1076" in diag

    # --- invariant 2: minimum height ---

    def test_height_below_720_triggers_abort(self):
        ok, diag, msgs = self._call(current_win=(1280.0, 600.0), build_win=(1280.0, 600.0))
        assert ok is False
        assert "WINDOW-TOO-SMALL" in diag
        assert "NEEDS_HUMAN_GUI" in diag

    def test_height_exactly_720_passes(self):
        ok, diag, msgs = self._call(current_win=(1280.0, 720.0), build_win=(1280.0, 720.0))
        assert ok is True

    def test_height_719_fails(self):
        ok, diag, msgs = self._call(current_win=(1280.0, 719.0), build_win=(1280.0, 719.0))
        assert ok is False
        assert "WINDOW-TOO-SMALL" in diag

    # --- invariant 3: cursor_logical not None ---

    def test_cursor_logical_none_triggers_abort(self):
        ok, diag, msgs = self._call(cursor_logical=None)
        assert ok is False
        assert "CURSOR-LOGICAL-NONE" in diag
        assert "NEEDS_HUMAN_GUI" in diag

    def test_cursor_logical_present_passes(self):
        ok, diag, msgs = self._call(cursor_logical=[100.0, 200.0])
        assert ok is True

    # --- invariant 4: last cursor screen OOB ---

    def test_oob_cursor_at_mousedown_triggers_abort(self):
        # cursor moved to x=1400 which is past window width=1280
        ok, diag, msgs = self._call(last_cursor_screen=(1400.0, 360.0))
        assert ok is False
        assert "CLICK-OOB-AT-MOUSEDOWN" in diag
        assert "NEEDS_HUMAN_GUI" in diag

    def test_in_bounds_cursor_passes(self):
        ok, diag, msgs = self._call(last_cursor_screen=(640.0, 360.0))
        assert ok is True

    def test_none_last_cursor_screen_skips_oob_check(self):
        # No cursor move yet — guard must not fail on this alone
        ok, diag, msgs = self._call(last_cursor_screen=None)
        assert ok is True

    # --- priority: drift check fires first ---

    def test_drift_check_fires_before_cursor_none(self):
        # Both drift and cursor_logical=None; should get WINDOW-DRIFT message
        ok, diag, msgs = self._call(
            current_win=(1280.0, 1076.0),
            build_win=(1280.0, 720.0),
            cursor_logical=None,
        )
        assert ok is False
        assert "WINDOW-DRIFT" in diag

    # --- log content ---

    def test_abort_log_includes_tick_number(self):
        messages: list[str] = []
        _check_window_stable_for_mouseclick(
            current_win=(1280.0, 1076.0),
            build_win=(1280.0, 720.0),
            cursor_logical=[640.0, 360.0],
            last_cursor_screen=(640.0, 360.0),
            tick=115,
            log_fn=messages.append,
        )
        assert any("tick=115" in m for m in messages), "log must include driver tick"


# ---------------------------------------------------------------------------
# Exit code value
# ---------------------------------------------------------------------------

class TestExitWindowMismatch:
    def test_exit_window_mismatch_is_5(self):
        assert EXIT_WINDOW_MISMATCH == 5, (
            "EXIT_WINDOW_MISMATCH must be 5 to remain distinct from EXIT_BLOCKED=4"
        )


# ---------------------------------------------------------------------------
# Driver structural checks
# ---------------------------------------------------------------------------

class TestDriverWindowDriftGuardStructure:
    """Structural checks: driver.py must define and invoke the drift guard."""

    _DRIVER_SOURCE = (_TOOLS_AUTOPLAY / "driver.py").read_text(encoding="utf-8")

    def test_driver_defines_check_window_stable(self):
        assert "def _check_window_stable_for_mouseclick(" in self._DRIVER_SOURCE

    def test_driver_exit_window_mismatch_constant(self):
        assert "EXIT_WINDOW_MISMATCH = 5" in self._DRIVER_SOURCE

    def test_driver_needs_human_gui_sentinel(self):
        assert "NEEDS_HUMAN_GUI" in self._DRIVER_SOURCE, (
            "driver.py must emit NEEDS_HUMAN_GUI checkpoint kind on window drift"
        )

    def test_driver_guard_gated_on_mouse_down_or_up(self):
        src = self._DRIVER_SOURCE
        assert '"mouse_down" in params or "mouse_up" in params' in src, (
            "guard must fire on mouse_down or mouse_up params"
        )

    def test_driver_tracks_recipe_build_win_size(self):
        assert "recipe_build_win_size" in self._DRIVER_SOURCE

    def test_driver_calls_check_window_stable(self):
        assert "_check_window_stable_for_mouseclick(" in self._DRIVER_SOURCE

    def test_driver_breaks_tick_loop_on_window_mismatch(self):
        src = self._DRIVER_SOURCE
        assert "rc == EXIT_WINDOW_MISMATCH" in src, (
            "driver must break the tick loop when EXIT_WINDOW_MISMATCH is set"
        )

    def test_driver_emits_needs_human_gui_checkpoint_on_drift(self):
        src = self._DRIVER_SOURCE
        # The checkpoint emit must be inside the drift guard block
        guard_idx = src.index("_check_window_stable_for_mouseclick(")
        needs_human_idx = src.index('"NEEDS_HUMAN_GUI"', guard_idx)
        assert needs_human_idx > guard_idx, (
            "NEEDS_HUMAN_GUI checkpoint must be emitted after the stability check"
        )
