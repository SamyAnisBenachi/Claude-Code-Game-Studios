"""Tests for the click-target viewport guard (PROMPT 1857).

Covers blocking/dirty-verdict semantics for:
  - _validate_cursor_coords(): in-bounds accepted; OOB triggers CLICK-OOB log
  - _parse_window_size(): valid/invalid raw values
  - _check_window_minimum(): window >= 1280x720 required; None blocks
  - _check_window_drift(): mid-run resize beyond +/-10 px aborts
  - cursor_logical None check: abort before input dispatch
  - EXIT_VIEWPORT_GUARD = 5 exported from driver
  - Structural: helpers defined and called correctly in main()

No GUI, no Bevy launch, no Cargo.  Run with:
    pytest tests/tools/autoplay/test_driver_click_viewport_guard.py -v
"""

from __future__ import annotations

import sys
from pathlib import Path

_TOOLS_AUTOPLAY = Path(__file__).resolve().parents[3] / "tools" / "autoplay"
if str(_TOOLS_AUTOPLAY) not in sys.path:
    sys.path.insert(0, str(_TOOLS_AUTOPLAY))

from driver import (  # noqa: E402
    EXIT_VIEWPORT_GUARD,
    _check_window_drift,
    _check_window_minimum,
    _parse_window_size,
    _validate_cursor_coords,
)


# ---------------------------------------------------------------------------
# _validate_cursor_coords unit tests (retained from PROMPT 1843, updated)
# ---------------------------------------------------------------------------

class TestValidateCursorCoords:
    """_validate_cursor_coords logs CLICK-OOB and returns (False, diag) for OOB."""

    def test_in_bounds_returns_true_no_log(self):
        messages: list[str] = []
        ok, diag = _validate_cursor_coords(640.0, 360.0, (1280.0, 720.0), tick=1, log_fn=messages.append)
        assert ok is True
        assert diag == ""
        assert messages == []

    def test_x_clip_returns_false_with_oob_log(self):
        messages: list[str] = []
        ok, diag = _validate_cursor_coords(1400.0, 360.0, (1280.0, 720.0), tick=5, log_fn=messages.append)
        assert ok is False
        assert "x_clip" in diag
        assert "y_clip" not in diag
        assert "CLICK-OOB" in diag
        assert len(messages) == 1
        assert "tick=5" in messages[0]

    def test_y_clip_returns_false_with_oob_log(self):
        messages: list[str] = []
        ok, diag = _validate_cursor_coords(640.0, 800.0, (1280.0, 720.0), tick=7, log_fn=messages.append)
        assert ok is False
        assert "y_clip" in diag
        assert "x_clip" not in diag
        assert "CLICK-OOB" in diag

    def test_both_axes_clipped_logs_both_labels(self):
        messages: list[str] = []
        ok, diag = _validate_cursor_coords(1500.0, 900.0, (1280.0, 720.0), tick=10, log_fn=messages.append)
        assert ok is False
        assert "x_clip" in diag
        assert "y_clip" in diag

    def test_origin_0_0_is_in_bounds(self):
        messages: list[str] = []
        ok, _ = _validate_cursor_coords(0.0, 0.0, (1280.0, 720.0), tick=1, log_fn=messages.append)
        assert ok is True
        assert messages == []

    def test_exact_width_height_is_out_of_bounds(self):
        messages: list[str] = []
        ok, diag = _validate_cursor_coords(1280.0, 720.0, (1280.0, 720.0), tick=2, log_fn=messages.append)
        assert ok is False
        assert "CLICK-OOB" in diag

    def test_negative_coords_are_out_of_bounds(self):
        messages: list[str] = []
        ok, diag = _validate_cursor_coords(-10.0, 200.0, (1280.0, 720.0), tick=3, log_fn=messages.append)
        assert ok is False
        assert "x_clip" in diag

    def test_zero_window_size_produces_invalid_diagnostic(self):
        messages: list[str] = []
        ok, diag = _validate_cursor_coords(640.0, 360.0, (0.0, 0.0), tick=1, log_fn=messages.append)
        assert ok is False
        assert "invalid" in diag.lower()
        assert "CLICK-OOB" in diag

    def test_invalid_negative_window_size_logs_diagnostic(self):
        messages: list[str] = []
        ok, diag = _validate_cursor_coords(100.0, 100.0, (-100.0, 720.0), tick=1, log_fn=messages.append)
        assert ok is False
        assert "invalid" in diag.lower() or "CLICK-OOB" in diag

    def test_log_includes_tick_number(self):
        messages: list[str] = []
        _validate_cursor_coords(9999.0, 9999.0, (1280.0, 720.0), tick=42, log_fn=messages.append)
        assert any("tick=42" in m for m in messages)

    def test_log_includes_window_dimensions(self):
        messages: list[str] = []
        _validate_cursor_coords(9999.0, 9999.0, (1280.0, 720.0), tick=1, log_fn=messages.append)
        assert any("1280" in m and "720" in m for m in messages)

    def test_log_includes_cursor_coords(self):
        messages: list[str] = []
        _validate_cursor_coords(1500.0, 850.0, (1280.0, 720.0), tick=1, log_fn=messages.append)
        assert any("1500" in m for m in messages)
        assert any("850" in m for m in messages)

    def test_log_includes_fractional_coordinates(self):
        messages: list[str] = []
        _validate_cursor_coords(1920.0, 360.0, (1280.0, 720.0), tick=1, log_fn=messages.append)
        assert any("frac=" in m for m in messages)


# ---------------------------------------------------------------------------
# _parse_window_size unit tests (PROMPT 1857)
# ---------------------------------------------------------------------------

class TestParseWindowSize:
    """_parse_window_size returns (w, h) for valid input, None otherwise."""

    def test_valid_list_returns_tuple(self):
        assert _parse_window_size([1280.0, 720.0]) == (1280.0, 720.0)

    def test_int_values_coerced_to_float(self):
        assert _parse_window_size([1280, 720]) == (1280.0, 720.0)

    def test_none_returns_none(self):
        assert _parse_window_size(None) is None

    def test_empty_list_returns_none(self):
        assert _parse_window_size([]) is None

    def test_one_element_returns_none(self):
        assert _parse_window_size([1280.0]) is None

    def test_three_elements_returns_none(self):
        assert _parse_window_size([1280.0, 720.0, 0.0]) is None

    def test_zero_width_returns_none(self):
        assert _parse_window_size([0.0, 720.0]) is None

    def test_zero_height_returns_none(self):
        assert _parse_window_size([1280.0, 0.0]) is None

    def test_negative_width_returns_none(self):
        assert _parse_window_size([-1.0, 720.0]) is None

    def test_non_numeric_returns_none(self):
        assert _parse_window_size(["bad", 720.0]) is None

    def test_dict_returns_none(self):
        assert _parse_window_size({"w": 1280, "h": 720}) is None


# ---------------------------------------------------------------------------
# _check_window_minimum unit tests (PROMPT 1857)
# ---------------------------------------------------------------------------

class TestCheckWindowMinimum:
    """_check_window_minimum blocks when window < 1280x720 or is None."""

    def test_exact_minimum_passes(self):
        messages: list[str] = []
        ok, diag = _check_window_minimum((1280.0, 720.0), tick=1, log_fn=messages.append)
        assert ok is True
        assert diag == ""
        assert messages == []

    def test_larger_than_minimum_passes(self):
        messages: list[str] = []
        ok, _ = _check_window_minimum((1920.0, 1080.0), tick=1, log_fn=messages.append)
        assert ok is True
        assert messages == []

    def test_none_window_size_blocks(self):
        messages: list[str] = []
        ok, diag = _check_window_minimum(None, tick=1, log_fn=messages.append)
        assert ok is False
        assert "VIEWPORT-GUARD" in diag
        assert len(messages) == 1

    def test_width_below_minimum_blocks(self):
        messages: list[str] = []
        ok, diag = _check_window_minimum((1000.0, 720.0), tick=3, log_fn=messages.append)
        assert ok is False
        assert "VIEWPORT-GUARD" in diag
        assert "too small" in diag

    def test_height_below_minimum_blocks(self):
        messages: list[str] = []
        ok, diag = _check_window_minimum((1280.0, 600.0), tick=3, log_fn=messages.append)
        assert ok is False
        assert "VIEWPORT-GUARD" in diag
        assert "too small" in diag

    def test_both_axes_below_minimum_blocks(self):
        messages: list[str] = []
        ok, diag = _check_window_minimum((800.0, 600.0), tick=2, log_fn=messages.append)
        assert ok is False
        assert "VIEWPORT-GUARD" in diag

    def test_log_includes_tick(self):
        messages: list[str] = []
        _check_window_minimum(None, tick=99, log_fn=messages.append)
        assert any("tick=99" in m for m in messages)

    def test_log_includes_minimum_dimensions(self):
        messages: list[str] = []
        _check_window_minimum((800.0, 600.0), tick=1, log_fn=messages.append)
        assert any("1280" in m and "720" in m for m in messages)


# ---------------------------------------------------------------------------
# _check_window_drift unit tests (PROMPT 1857)
# ---------------------------------------------------------------------------

class TestCheckWindowDrift:
    """_check_window_drift blocks when mid-run resize exceeds +/-10 px."""

    def test_no_drift_passes(self):
        messages: list[str] = []
        ok, diag = _check_window_drift((1280.0, 720.0), (1280.0, 720.0), tick=5, log_fn=messages.append)
        assert ok is True
        assert diag == ""
        assert messages == []

    def test_drift_within_tolerance_passes(self):
        messages: list[str] = []
        ok, _ = _check_window_drift((1280.0, 720.0), (1289.0, 729.0), tick=5, log_fn=messages.append)
        assert ok is True

    def test_drift_exactly_at_tolerance_passes(self):
        messages: list[str] = []
        ok, _ = _check_window_drift((1280.0, 720.0), (1290.0, 730.0), tick=5, log_fn=messages.append)
        assert ok is True

    def test_width_drift_beyond_tolerance_blocks(self):
        messages: list[str] = []
        ok, diag = _check_window_drift((1280.0, 720.0), (1291.0, 720.0), tick=10, log_fn=messages.append)
        assert ok is False
        assert "VIEWPORT-GUARD" in diag
        assert "mid-run resize" in diag
        assert len(messages) == 1

    def test_height_drift_beyond_tolerance_blocks(self):
        messages: list[str] = []
        ok, diag = _check_window_drift((1280.0, 720.0), (1280.0, 731.0), tick=10, log_fn=messages.append)
        assert ok is False
        assert "VIEWPORT-GUARD" in diag
        assert "mid-run resize" in diag

    def test_shrink_drift_also_blocks(self):
        messages: list[str] = []
        ok, diag = _check_window_drift((1280.0, 720.0), (1268.0, 720.0), tick=10, log_fn=messages.append)
        assert ok is False
        assert "VIEWPORT-GUARD" in diag

    def test_none_current_size_blocks(self):
        messages: list[str] = []
        ok, diag = _check_window_drift((1280.0, 720.0), None, tick=7, log_fn=messages.append)
        assert ok is False
        assert "VIEWPORT-GUARD" in diag
        assert "lost mid-run" in diag

    def test_log_includes_build_and_current_size(self):
        messages: list[str] = []
        _check_window_drift((1280.0, 720.0), (1400.0, 720.0), tick=5, log_fn=messages.append)
        assert any("1280" in m and "1400" in m for m in messages)

    def test_log_includes_drift_amount(self):
        messages: list[str] = []
        _check_window_drift((1280.0, 720.0), (1400.0, 720.0), tick=5, log_fn=messages.append)
        assert any("drift=" in m for m in messages)

    def test_log_includes_tick(self):
        messages: list[str] = []
        _check_window_drift((1280.0, 720.0), (1400.0, 720.0), tick=77, log_fn=messages.append)
        assert any("tick=77" in m for m in messages)


# ---------------------------------------------------------------------------
# EXIT_VIEWPORT_GUARD constant (PROMPT 1857)
# ---------------------------------------------------------------------------

class TestExitViewportGuard:
    """EXIT_VIEWPORT_GUARD must be defined and distinct from other exit codes."""

    def test_exit_viewport_guard_is_5(self):
        assert EXIT_VIEWPORT_GUARD == 5

    def test_exit_viewport_guard_distinct_from_ok(self):
        from driver import EXIT_OK
        assert EXIT_VIEWPORT_GUARD != EXIT_OK

    def test_exit_viewport_guard_distinct_from_rpc_error(self):
        from driver import EXIT_RPC_ERROR
        assert EXIT_VIEWPORT_GUARD != EXIT_RPC_ERROR

    def test_exit_viewport_guard_distinct_from_blocked(self):
        from driver import EXIT_BLOCKED
        assert EXIT_VIEWPORT_GUARD != EXIT_BLOCKED


# ---------------------------------------------------------------------------
# Driver structural checks (PROMPT 1857)
# ---------------------------------------------------------------------------

class TestDriverViewportGuardStructure:
    """Structural checks: driver.py must define and invoke the viewport guard."""

    _DRIVER_SOURCE = (_TOOLS_AUTOPLAY / "driver.py").read_text(encoding="utf-8")

    def test_driver_defines_validate_cursor_coords(self):
        assert "def _validate_cursor_coords(" in self._DRIVER_SOURCE

    def test_driver_logs_click_oob_sentinel(self):
        assert "CLICK-OOB" in self._DRIVER_SOURCE

    def test_driver_invokes_validate_cursor_coords(self):
        assert "_validate_cursor_coords(" in self._DRIVER_SOURCE

    def test_driver_defines_parse_window_size(self):
        assert "def _parse_window_size(" in self._DRIVER_SOURCE

    def test_driver_defines_check_window_minimum(self):
        assert "def _check_window_minimum(" in self._DRIVER_SOURCE

    def test_driver_defines_check_window_drift(self):
        assert "def _check_window_drift(" in self._DRIVER_SOURCE

    def test_driver_exports_exit_viewport_guard(self):
        assert "EXIT_VIEWPORT_GUARD" in self._DRIVER_SOURCE

    def test_driver_exits_5_on_viewport_guard(self):
        assert "EXIT_VIEWPORT_GUARD = 5" in self._DRIVER_SOURCE

    def test_driver_checks_cursor_logical_none(self):
        assert "cursor_logical" in self._DRIVER_SOURCE

    def test_driver_checks_window_minimum_at_recipe_build(self):
        assert "_check_window_minimum(" in self._DRIVER_SOURCE

    def test_driver_checks_window_drift_mid_run(self):
        assert "_check_window_drift(" in self._DRIVER_SOURCE

    def test_driver_guard_gated_on_autoplay_input(self):
        src = self._DRIVER_SOURCE
        def_idx = src.index("def _validate_cursor_coords(")
        call_idx = src.index(
            "_validate_cursor_coords(",
            def_idx + len("def _validate_cursor_coords("),
        )
        input_check_idx = src.rindex('method == "autoplay/input"', 0, call_idx)
        assert input_check_idx < call_idx

    def test_driver_logs_viewport_guard_abort(self):
        assert "VIEWPORT-GUARD ABORT" in self._DRIVER_SOURCE

    def test_driver_stores_recipe_build_win_size(self):
        assert "recipe_build_win_size" in self._DRIVER_SOURCE

    # AC-VPT-08: post-foreground shrink check
    def test_driver_checks_post_foreground_window_size(self):
        assert "viewport_shrink_abort" in self._DRIVER_SOURCE, (
            "driver must emit viewport_shrink_abort checkpoint when post-ensure_foreground "
            "window drops below minimum (AC-VPT-08)"
        )

    def test_driver_post_foreground_check_after_ensure_foreground(self):
        src = self._DRIVER_SOURCE
        fg_idx = src.index("ensure_foreground(log)")
        shrink_idx = src.index("viewport_shrink_abort", fg_idx)
        assert shrink_idx > fg_idx, (
            "viewport_shrink_abort must appear after ensure_foreground call"
        )

    # Checkpoint emission for all guard conditions
    def test_driver_emits_viewport_drift_checkpoint(self):
        assert '"viewport_drift"' in self._DRIVER_SOURCE or "'viewport_drift'" in self._DRIVER_SOURCE

    def test_driver_emits_viewport_shrink_abort_checkpoint(self):
        assert '"viewport_shrink_abort"' in self._DRIVER_SOURCE or "'viewport_shrink_abort'" in self._DRIVER_SOURCE

    def test_driver_emits_cursor_none_checkpoint(self):
        assert "viewport_guard_cursor_none" in self._DRIVER_SOURCE

    def test_driver_emits_oob_checkpoint(self):
        assert "viewport_guard_oob" in self._DRIVER_SOURCE
