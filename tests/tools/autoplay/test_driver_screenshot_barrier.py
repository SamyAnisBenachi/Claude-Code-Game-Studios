"""Static regression tests for the screenshot frame-advance barrier (PROMPT 1766).

Verifies:
  - RecipeBuilder.checkpoint() inserts the correct settle_ticks gap between
    the local.checkpoint action and the autoplay/screenshot action so the
    Bevy renderer has time to produce a fresh frame (GAP-SCR-01).
  - The settle_ticks default is 3 (300 ms at 10 Hz driver rate).
  - settle_ticks=0 preserves the legacy 1-tick gap.
  - screenshot=False emits no screenshot action.
  - The driver source contains the frame-advance barrier guard for
    autoplay/screenshot dispatches.

No GUI, no Bevy launch, no Cargo.  Run with:
    pytest tests/tools/autoplay/test_driver_screenshot_barrier.py -v
"""

from __future__ import annotations

import sys
from pathlib import Path

_TOOLS_AUTOPLAY = Path(__file__).resolve().parents[3] / "tools" / "autoplay"
if str(_TOOLS_AUTOPLAY) not in sys.path:
    sys.path.insert(0, str(_TOOLS_AUTOPLAY))

from recipes._builder import RecipeBuilder  # noqa: E402


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _checkpoint_tick(actions: list[dict], label: str) -> int:
    for a in actions:
        if a.get("method") == "local.checkpoint" and a.get("params", {}).get("label") == label:
            return a["tick"]
    raise AssertionError(f"local.checkpoint label={label!r} not found in {actions}")


def _screenshot_tick_for_checkpoint(actions: list[dict], label: str) -> int:
    reason = f"checkpoint:{label}"
    for a in actions:
        if a.get("method") == "autoplay/screenshot" and a.get("params", {}).get("reason") == reason:
            return a["tick"]
    raise AssertionError(f"autoplay/screenshot reason={reason!r} not found in {actions}")


# ---------------------------------------------------------------------------
# settle_ticks gap contract
# ---------------------------------------------------------------------------

class TestCheckpointSettleTicksGap:
    """The settle_ticks gap prevents capturing a stale cached frame."""

    def test_checkpoint_settle_default_is_3_ticks(self):
        # Arrange
        b = RecipeBuilder((1280.0, 720.0))

        # Act
        b.checkpoint("lobby-loaded")
        actions = b.build()

        # Assert: screenshot tick = checkpoint_tick + 1 (checkpoint→next) + 3 (settle) = cp+4
        cp = _checkpoint_tick(actions, "lobby-loaded")
        sc = _screenshot_tick_for_checkpoint(actions, "lobby-loaded")
        assert sc == cp + 4, (
            f"Default settle_ticks=3 must place screenshot 4 ticks after checkpoint "
            f"(checkpoint_tick={cp}, screenshot_tick={sc})"
        )

    def test_checkpoint_settle_ticks_zero_reverts_to_immediate(self):
        # Arrange
        b = RecipeBuilder((1280.0, 720.0))

        # Act
        b.checkpoint("lobby-loaded", settle_ticks=0)
        actions = b.build()

        # Assert: screenshot tick = checkpoint_tick + 1 (no settle gap)
        cp = _checkpoint_tick(actions, "lobby-loaded")
        sc = _screenshot_tick_for_checkpoint(actions, "lobby-loaded")
        assert sc == cp + 1, (
            f"settle_ticks=0 must place screenshot immediately after checkpoint "
            f"(checkpoint_tick={cp}, screenshot_tick={sc})"
        )

    def test_checkpoint_settle_ticks_custom_value(self):
        # Arrange
        b = RecipeBuilder((1280.0, 720.0))

        # Act
        b.checkpoint("phase-start", settle_ticks=5)
        actions = b.build()

        # Assert: screenshot tick = checkpoint_tick + 1 + 5
        cp = _checkpoint_tick(actions, "phase-start")
        sc = _screenshot_tick_for_checkpoint(actions, "phase-start")
        assert sc == cp + 6, (
            f"settle_ticks=5 must place screenshot 6 ticks after checkpoint "
            f"(checkpoint_tick={cp}, screenshot_tick={sc})"
        )

    def test_checkpoint_no_screenshot_emits_no_screenshot_action(self):
        # Arrange
        b = RecipeBuilder((1280.0, 720.0))

        # Act
        b.checkpoint("silent-gate", screenshot=False)
        actions = b.build()

        # Assert
        screenshots = [a for a in actions if a.get("method") == "autoplay/screenshot"]
        assert screenshots == [], (
            "screenshot=False must emit no autoplay/screenshot action"
        )

    def test_multiple_checkpoints_each_get_settle_gap(self):
        # Arrange
        b = RecipeBuilder((1280.0, 720.0))

        # Act
        b.checkpoint("first")
        b.wait(2)
        b.checkpoint("second")
        actions = b.build()

        # Assert each checkpoint has the correct settle gap independently
        for label in ("first", "second"):
            cp = _checkpoint_tick(actions, label)
            sc = _screenshot_tick_for_checkpoint(actions, label)
            assert sc == cp + 4, (
                f"checkpoint {label!r}: screenshot_tick={sc} should be cp+4={cp+4}"
            )

    def test_checkpoint_screenshot_reason_matches_label(self):
        # Arrange
        b = RecipeBuilder((1280.0, 720.0))

        # Act
        b.checkpoint("placement-loaded")
        actions = b.build()

        # Assert
        sc_actions = [
            a for a in actions
            if a.get("method") == "autoplay/screenshot"
            and a.get("params", {}).get("reason") == "checkpoint:placement-loaded"
        ]
        assert len(sc_actions) == 1, (
            "checkpoint must emit exactly one screenshot with reason=checkpoint:<label>"
        )

    def test_settle_ticks_does_not_affect_checkpoint_without_screenshot(self):
        # Arrange — settle_ticks ignored when screenshot=False
        b = RecipeBuilder((1280.0, 720.0))

        # Act
        b.checkpoint("guard", screenshot=False, settle_ticks=10)
        b.checkpoint("after")
        actions = b.build()

        # Assert: the "after" checkpoint screenshot is settle_ticks=3 (default)
        # away from the "after" local.checkpoint, not from "guard".
        cp_after = _checkpoint_tick(actions, "after")
        sc_after = _screenshot_tick_for_checkpoint(actions, "after")
        assert sc_after == cp_after + 4


# ---------------------------------------------------------------------------
# Driver source contains frame-advance barrier (structural check)
# ---------------------------------------------------------------------------

class TestDriverFrameAdvanceBarrierPresent:
    """Structural checks: driver.py must contain the barrier guard."""

    _DRIVER_SOURCE = (_TOOLS_AUTOPLAY / "driver.py").read_text(encoding="utf-8")

    def test_driver_tracks_last_screenshot_frame(self):
        assert "last_screenshot_frame" in self._DRIVER_SOURCE, (
            "driver.py must declare last_screenshot_frame to track the barrier state"
        )

    def test_driver_checks_frame_before_screenshot(self):
        assert 'method == "autoplay/screenshot"' in self._DRIVER_SOURCE, (
            "driver.py must branch on autoplay/screenshot before dispatching"
        )

    def test_driver_logs_stale_frame_warning(self):
        assert "renderer may not be producing new frames" in self._DRIVER_SOURCE, (
            "driver.py must log a warning when the frame counter is stuck"
        )

    def test_driver_frame_guard_initialised_to_minus_one(self):
        assert "last_screenshot_frame: int = -1" in self._DRIVER_SOURCE, (
            "last_screenshot_frame must be initialised to -1 so the first screenshot always passes"
        )
