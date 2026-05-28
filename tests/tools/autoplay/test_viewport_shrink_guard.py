"""Tests for the mid-run viewport/window-size shrink guard (PROMPT 1922 / PROMPT 1940).

Covers:
- check_viewport_size: returns (True, None) for large-enough window.
- check_viewport_size: returns (False, reason) when window is too small.
- check_viewport_size: returns (False, reason) when window size is missing.
- check_click_target: returns (True, None) for in-bounds coordinate.
- check_click_target: returns (False, reason) for out-of-bounds coordinate.
- check_click_target: returns (False, reason) when size is missing.
- check_before_input: passes through key-only actions (no cursor).
- check_before_input: passes a valid cursor on a normal viewport.
- check_before_input: blocks a cursor that is offscreen.
- check_before_input: blocks when viewport is smaller than minimum.
- check_before_input: blocks mid-run shrink (viewport was OK, now too small).
- driver.py contains inlined viewport guard logic (EXIT_VIEWPORT_GUARD, VIEWPORT-GUARD).

No GUI, no Bevy launch, no Cargo.  Run with:
    pytest tests/tools/autoplay/test_viewport_shrink_guard.py -v
"""

from __future__ import annotations

import sys
from pathlib import Path

_TOOLS_AUTOPLAY = Path(__file__).resolve().parents[3] / "tools" / "autoplay"
if str(_TOOLS_AUTOPLAY) not in sys.path:
    sys.path.insert(0, str(_TOOLS_AUTOPLAY))

from viewport_shrink_guard import (  # noqa: E402
    MIN_QA_VIEWPORT_H,
    MIN_QA_VIEWPORT_W,
    check_before_input,
    check_click_target,
    check_viewport_size,
)


def _status(w: float, h: float) -> dict:
    return {"window_logical_size": [w, h], "frame": 1}


def _status_no_size() -> dict:
    return {"frame": 1}


# ---------------------------------------------------------------------------
# check_viewport_size
# ---------------------------------------------------------------------------

class TestCheckViewportSize:

    def test_check_viewport_size_valid_returns_ok(self):
        # Arrange
        status = _status(1280.0, 720.0)

        # Act
        ok, reason = check_viewport_size(status)

        # Assert
        assert ok is True
        assert reason is None

    def test_check_viewport_size_larger_than_minimum_returns_ok(self):
        # Arrange
        status = _status(1920.0, 1080.0)

        # Act
        ok, reason = check_viewport_size(status)

        # Assert
        assert ok is True
        assert reason is None

    def test_check_viewport_size_width_too_small_blocked(self):
        # Arrange
        status = _status(800.0, 720.0)

        # Act
        ok, reason = check_viewport_size(status)

        # Assert
        assert ok is False
        assert reason is not None
        assert "viewport_too_small" in reason
        assert "800" in reason

    def test_check_viewport_size_height_too_small_blocked(self):
        # Arrange
        status = _status(1280.0, 600.0)

        # Act
        ok, reason = check_viewport_size(status)

        # Assert
        assert ok is False
        assert reason is not None
        assert "viewport_too_small" in reason
        assert "600" in reason

    def test_check_viewport_size_both_too_small_blocked(self):
        # Arrange
        status = _status(640.0, 480.0)

        # Act
        ok, reason = check_viewport_size(status)

        # Assert
        assert ok is False
        assert reason is not None
        assert "640" in reason

    def test_check_viewport_size_missing_size_blocked(self):
        # Arrange
        status = _status_no_size()

        # Act
        ok, reason = check_viewport_size(status)

        # Assert
        assert ok is False
        assert reason is not None
        assert "viewport_size_unknown" in reason

    def test_check_viewport_size_custom_minimum_respected(self):
        # Arrange: window exactly at custom minimum
        status = _status(800.0, 600.0)

        # Act: use a smaller custom minimum
        ok, reason = check_viewport_size(status, min_w=800.0, min_h=600.0)

        # Assert
        assert ok is True
        assert reason is None

    def test_check_viewport_size_reason_includes_minimum_dimensions(self):
        # Arrange
        status = _status(640.0, 360.0)

        # Act
        ok, reason = check_viewport_size(status)

        # Assert: reason must mention the minimum so the user knows the threshold
        assert ok is False
        assert str(int(MIN_QA_VIEWPORT_W)) in reason
        assert str(int(MIN_QA_VIEWPORT_H)) in reason


# ---------------------------------------------------------------------------
# check_click_target
# ---------------------------------------------------------------------------

class TestCheckClickTarget:

    def test_check_click_target_centre_of_window_allowed(self):
        # Arrange
        status = _status(1280.0, 720.0)

        # Act
        ok, reason = check_click_target(640.0, 360.0, status)

        # Assert
        assert ok is True
        assert reason is None

    def test_check_click_target_top_left_corner_allowed(self):
        # Arrange
        status = _status(1280.0, 720.0)

        # Act
        ok, reason = check_click_target(0.0, 0.0, status)

        # Assert
        assert ok is True
        assert reason is None

    def test_check_click_target_just_inside_right_edge_allowed(self):
        # Arrange
        status = _status(1280.0, 720.0)

        # Act: x=1279 is inside [0, 1280)
        ok, reason = check_click_target(1279.0, 360.0, status)

        # Assert
        assert ok is True
        assert reason is None

    def test_check_click_target_at_right_edge_blocked(self):
        # Arrange
        status = _status(1280.0, 720.0)

        # Act: x=1280 equals width → offscreen
        ok, reason = check_click_target(1280.0, 360.0, status)

        # Assert
        assert ok is False
        assert reason is not None
        assert "click_target_offscreen" in reason

    def test_check_click_target_negative_x_blocked(self):
        # Arrange
        status = _status(1280.0, 720.0)

        # Act
        ok, reason = check_click_target(-1.0, 360.0, status)

        # Assert
        assert ok is False
        assert "click_target_offscreen" in reason

    def test_check_click_target_negative_y_blocked(self):
        # Arrange
        status = _status(1280.0, 720.0)

        # Act
        ok, reason = check_click_target(640.0, -5.0, status)

        # Assert
        assert ok is False
        assert "click_target_offscreen" in reason

    def test_check_click_target_far_offscreen_blocked(self):
        # Arrange
        status = _status(1280.0, 720.0)

        # Act
        ok, reason = check_click_target(9999.0, 9999.0, status)

        # Assert
        assert ok is False
        assert reason is not None

    def test_check_click_target_missing_size_blocked(self):
        # Arrange
        status = _status_no_size()

        # Act
        ok, reason = check_click_target(640.0, 360.0, status)

        # Assert
        assert ok is False
        assert "click_target_unverifiable" in reason

    def test_check_click_target_reason_includes_coordinates(self):
        # Arrange
        status = _status(1280.0, 720.0)

        # Act
        ok, reason = check_click_target(1500.0, 800.0, status)

        # Assert
        assert ok is False
        assert "1500" in reason
        assert "800" in reason


# ---------------------------------------------------------------------------
# check_before_input (combined guard)
# ---------------------------------------------------------------------------

class TestCheckBeforeInput:

    def test_check_before_input_key_only_action_passes(self):
        # Arrange: key_down action has no cursor field
        params = {"keys_down": ["Space"]}
        status = _status(1280.0, 720.0)

        # Act
        ok, reason = check_before_input(params, status)

        # Assert: non-cursor actions are not position-checked
        assert ok is True
        assert reason is None

    def test_check_before_input_mouse_button_only_passes(self):
        # Arrange: mouse_down without cursor
        params = {"mouse_down": ["Left"]}
        status = _status(1280.0, 720.0)

        # Act
        ok, reason = check_before_input(params, status)

        # Assert
        assert ok is True
        assert reason is None

    def test_check_before_input_valid_cursor_on_normal_viewport_passes(self):
        # Arrange: cursor inside 1280x720 window
        params = {"cursor": {"screen": [640.0, 360.0]}}
        status = _status(1280.0, 720.0)

        # Act
        ok, reason = check_before_input(params, status)

        # Assert
        assert ok is True
        assert reason is None

    def test_check_before_input_cursor_offscreen_blocked(self):
        # Arrange: cursor x is past the right edge
        params = {"cursor": {"screen": [1300.0, 360.0]}}
        status = _status(1280.0, 720.0)

        # Act
        ok, reason = check_before_input(params, status)

        # Assert
        assert ok is False
        assert reason is not None
        assert "offscreen" in reason

    def test_check_before_input_viewport_too_small_blocked(self):
        # Arrange: window shrank mid-run
        params = {"cursor": {"screen": [640.0, 360.0]}}
        status = _status(800.0, 480.0)

        # Act
        ok, reason = check_before_input(params, status)

        # Assert
        assert ok is False
        assert reason is not None
        assert "viewport_too_small" in reason

    def test_check_before_input_midrun_shrink_from_valid_to_invalid_blocked(self):
        # Arrange: simulate two consecutive ticks — first tick OK, second blocked
        params = {"cursor": {"screen": [640.0, 360.0]}}

        status_tick1 = _status(1280.0, 720.0)
        status_tick2 = _status(640.0, 480.0)

        # Act
        ok1, _ = check_before_input(params, status_tick1)
        ok2, reason2 = check_before_input(params, status_tick2)

        # Assert
        assert ok1 is True, "first tick (valid viewport) must pass"
        assert ok2 is False, "second tick (shrunken viewport) must block"
        assert reason2 is not None
        assert "viewport_too_small" in reason2

    def test_check_before_input_missing_window_size_blocked(self):
        # Arrange: status arrives without window_logical_size
        params = {"cursor": {"screen": [640.0, 360.0]}}
        status = _status_no_size()

        # Act
        ok, reason = check_before_input(params, status)

        # Assert
        assert ok is False
        assert reason is not None

    def test_check_before_input_reason_is_a_non_empty_string_when_blocked(self):
        # Arrange
        params = {"cursor": {"screen": [9999.0, 9999.0]}}
        status = _status(1280.0, 720.0)

        # Act
        ok, reason = check_before_input(params, status)

        # Assert
        assert ok is False
        assert isinstance(reason, str)
        assert len(reason) > 0

    def test_check_before_input_empty_params_passes(self):
        # Arrange: empty params (unusual but should not crash)
        params: dict = {}
        status = _status(1280.0, 720.0)

        # Act
        ok, reason = check_before_input(params, status)

        # Assert: no cursor → no position check
        assert ok is True
        assert reason is None


# ---------------------------------------------------------------------------
# Structural checks: driver.py contains inlined viewport guard
# (PROMPT 1940: guard was inlined in driver.py via PROMPT 1857;
#  the standalone viewport_shrink_guard module coexists as a utility.)
# ---------------------------------------------------------------------------

class TestDriverViewportGuardPresent:
    """Structural checks: driver.py must contain the inlined viewport guard."""

    _DRIVER_SOURCE = (_TOOLS_AUTOPLAY / "driver.py").read_text(encoding="utf-8")

    def test_driver_has_exit_viewport_guard_constant(self):
        assert "EXIT_VIEWPORT_GUARD" in self._DRIVER_SOURCE, (
            "driver.py must define EXIT_VIEWPORT_GUARD exit code"
        )

    def test_driver_logs_viewport_guard_prefix(self):
        assert "VIEWPORT-GUARD" in self._DRIVER_SOURCE, (
            "driver.py must emit VIEWPORT-GUARD log lines when the guard fires"
        )

    def test_driver_emits_viewport_checkpoint_kind(self):
        # Any of the inlined guard checkpoint kinds is acceptable
        assert any(
            k in self._DRIVER_SOURCE
            for k in (
                "viewport_shrink_block",
                "viewport_shrink_abort",
                "viewport_guard_oob",
                "viewport_guard_cursor_none",
                "viewport_drift",
            )
        ), "driver.py must emit at least one viewport guard checkpoint kind to checkpoints.jsonl"

    def test_driver_guard_is_inside_autoplay_input_branch(self):
        src = self._DRIVER_SOURCE
        input_branch_idx = src.index('method == "autoplay/input"')
        # Find the first VIEWPORT-GUARD occurrence AFTER the autoplay/input check
        guard_idx = src.find("VIEWPORT-GUARD", input_branch_idx)
        assert guard_idx != -1, (
            "viewport guard must appear after the autoplay/input method check"
        )

    def test_driver_has_minimum_window_constants(self):
        assert "_MIN_WIN_W" in self._DRIVER_SOURCE and "_MIN_WIN_H" in self._DRIVER_SOURCE, (
            "driver.py must define _MIN_WIN_W and _MIN_WIN_H minimum window size constants"
        )
