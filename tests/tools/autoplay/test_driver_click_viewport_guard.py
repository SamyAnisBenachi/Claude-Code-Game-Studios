"""Tests for the click-target viewport guard (PROMPT 1843).

Covers:
  - _validate_cursor_coords() returns (True, "") for in-bounds coords
  - Logs WARNING CLICK-OOB and returns (False, diag) for x-clipped coords
  - Logs WARNING CLICK-OOB and returns (False, diag) for y-clipped coords
  - Logs WARNING CLICK-OOB and returns (False, diag) for both axes clipped
  - Boundary: (0, 0) is in-bounds; (w, h) is out-of-bounds (strict < check)
  - Invalid window_size (0x0 or negative) produces a clear diagnostic
  - driver.py structural: _validate_cursor_coords defined and called at
    autoplay/input dispatch

No GUI, no Bevy launch, no Cargo.  Run with:
    pytest tests/tools/autoplay/test_driver_click_viewport_guard.py -v
"""

from __future__ import annotations

import sys
from pathlib import Path

_TOOLS_AUTOPLAY = Path(__file__).resolve().parents[3] / "tools" / "autoplay"
if str(_TOOLS_AUTOPLAY) not in sys.path:
    sys.path.insert(0, str(_TOOLS_AUTOPLAY))

from driver import _validate_cursor_coords  # noqa: E402


# ---------------------------------------------------------------------------
# _validate_cursor_coords unit tests
# ---------------------------------------------------------------------------

class TestValidateCursorCoords:
    """Unit tests for the viewport-bounds validation helper."""

    def test_in_bounds_returns_true_no_log(self):
        # Arrange
        messages: list[str] = []

        # Act
        ok, diag = _validate_cursor_coords(640.0, 360.0, (1280.0, 720.0), tick=1, log_fn=messages.append)

        # Assert
        assert ok is True
        assert diag == ""
        assert messages == [], "no log emitted for in-bounds cursor"

    def test_x_clip_returns_false_with_oob_log(self):
        # Arrange — x=1400 exceeds width=1280
        messages: list[str] = []

        # Act
        ok, diag = _validate_cursor_coords(1400.0, 360.0, (1280.0, 720.0), tick=5, log_fn=messages.append)

        # Assert
        assert ok is False
        assert "x_clip" in diag
        assert "y_clip" not in diag
        assert "CLICK-OOB" in diag
        assert len(messages) == 1
        assert "CLICK-OOB" in messages[0]
        assert "tick=5" in messages[0]

    def test_y_clip_returns_false_with_oob_log(self):
        # Arrange — y=800 exceeds height=720
        messages: list[str] = []

        # Act
        ok, diag = _validate_cursor_coords(640.0, 800.0, (1280.0, 720.0), tick=7, log_fn=messages.append)

        # Assert
        assert ok is False
        assert "y_clip" in diag
        assert "x_clip" not in diag
        assert "CLICK-OOB" in diag

    def test_both_axes_clipped_logs_both_labels(self):
        # Arrange
        messages: list[str] = []

        # Act
        ok, diag = _validate_cursor_coords(1500.0, 900.0, (1280.0, 720.0), tick=10, log_fn=messages.append)

        # Assert
        assert ok is False
        assert "x_clip" in diag
        assert "y_clip" in diag

    def test_origin_0_0_is_in_bounds(self):
        # Arrange — (0,0) must be valid; guard uses [0, w) range
        messages: list[str] = []

        # Act
        ok, _ = _validate_cursor_coords(0.0, 0.0, (1280.0, 720.0), tick=1, log_fn=messages.append)

        # Assert
        assert ok is True
        assert messages == []

    def test_exact_width_height_is_out_of_bounds(self):
        # Arrange — (w, h) is one pixel past the edge (strict < check)
        messages: list[str] = []

        # Act
        ok, diag = _validate_cursor_coords(1280.0, 720.0, (1280.0, 720.0), tick=2, log_fn=messages.append)

        # Assert
        assert ok is False
        assert "CLICK-OOB" in diag

    def test_negative_coords_are_out_of_bounds(self):
        # Arrange — negative cursor position (e.g. cursor dragged off-screen)
        messages: list[str] = []

        # Act
        ok, diag = _validate_cursor_coords(-10.0, 200.0, (1280.0, 720.0), tick=3, log_fn=messages.append)

        # Assert
        assert ok is False
        assert "x_clip" in diag

    def test_zero_window_size_produces_invalid_diagnostic(self):
        # Arrange — window_size (0,0) means size not yet reported
        messages: list[str] = []

        # Act
        ok, diag = _validate_cursor_coords(640.0, 360.0, (0.0, 0.0), tick=1, log_fn=messages.append)

        # Assert
        assert ok is False
        assert "invalid" in diag.lower()
        assert "CLICK-OOB" in diag

    def test_invalid_negative_window_size_logs_diagnostic(self):
        # Arrange
        messages: list[str] = []

        # Act
        ok, diag = _validate_cursor_coords(100.0, 100.0, (-100.0, 720.0), tick=1, log_fn=messages.append)

        # Assert
        assert ok is False
        assert "invalid" in diag.lower() or "CLICK-OOB" in diag

    def test_log_includes_tick_number(self):
        # Arrange
        messages: list[str] = []

        # Act
        _validate_cursor_coords(9999.0, 9999.0, (1280.0, 720.0), tick=42, log_fn=messages.append)

        # Assert
        assert any("tick=42" in m for m in messages), "log must include the driver tick"

    def test_log_includes_window_dimensions(self):
        # Arrange
        messages: list[str] = []

        # Act
        _validate_cursor_coords(9999.0, 9999.0, (1280.0, 720.0), tick=1, log_fn=messages.append)

        # Assert
        assert any("1280" in m and "720" in m for m in messages), (
            "log must include window dimensions for diagnosis"
        )

    def test_log_includes_cursor_coords(self):
        # Arrange
        messages: list[str] = []

        # Act
        _validate_cursor_coords(1500.0, 850.0, (1280.0, 720.0), tick=1, log_fn=messages.append)

        # Assert
        assert any("1500" in m for m in messages), "log must include cursor x"
        assert any("850" in m for m in messages), "log must include cursor y"

    def test_log_includes_fractional_coordinates(self):
        # Arrange — fraction helps diagnose how far off-screen the click landed
        messages: list[str] = []

        # Act
        _validate_cursor_coords(1920.0, 360.0, (1280.0, 720.0), tick=1, log_fn=messages.append)

        # Assert: frac_x = 1920/1280 = 1.500
        assert any("frac=" in m for m in messages), "log must include fractional coords"


# ---------------------------------------------------------------------------
# Driver structural checks
# ---------------------------------------------------------------------------

class TestDriverViewportGuardStructure:
    """Structural checks: driver.py must define and invoke the viewport guard."""

    _DRIVER_SOURCE = (_TOOLS_AUTOPLAY / "driver.py").read_text(encoding="utf-8")

    def test_driver_defines_validate_cursor_coords(self):
        assert "def _validate_cursor_coords(" in self._DRIVER_SOURCE, (
            "driver.py must define _validate_cursor_coords"
        )

    def test_driver_logs_click_oob_sentinel(self):
        assert "CLICK-OOB" in self._DRIVER_SOURCE, (
            "driver.py _validate_cursor_coords must use 'CLICK-OOB' sentinel for grep-ability"
        )

    def test_driver_invokes_guard_for_input_method(self):
        assert '_validate_cursor_coords(' in self._DRIVER_SOURCE, (
            "driver.py must call _validate_cursor_coords in the action dispatch loop"
        )

    def test_driver_guard_gated_on_autoplay_input(self):
        src = self._DRIVER_SOURCE
        # Skip the function definition; find the call site.
        def_idx = src.index("def _validate_cursor_coords(")
        call_idx = src.index(
            "_validate_cursor_coords(",
            def_idx + len("def _validate_cursor_coords("),
        )
        # The autoplay/input branch check must appear before the call.
        input_check_idx = src.rindex('method == "autoplay/input"', 0, call_idx)
        assert input_check_idx < call_idx, (
            "_validate_cursor_coords call must be inside an autoplay/input branch"
        )

    def test_driver_extracts_tick_win_size_from_status(self):
        assert "_tick_win_size" in self._DRIVER_SOURCE, (
            "driver.py must extract _tick_win_size from the status response each tick"
        )

    def test_driver_guard_uses_screen_field(self):
        assert '"screen"' in self._DRIVER_SOURCE or "\"screen\"" in self._DRIVER_SOURCE, (
            "driver.py must extract the 'screen' sub-field from cursor params"
        )
