"""Static regression tests for the screenshot frame-advance and file-ready barriers.

Covers (PROMPT 1766 — frame-advance barrier):
  - RecipeBuilder.checkpoint() inserts the correct settle_ticks gap between
    the local.checkpoint action and the autoplay/screenshot action so the
    Bevy renderer has time to produce a fresh frame (GAP-SCR-01).
  - The settle_ticks default is 3 (300 ms at 10 Hz driver rate).
  - settle_ticks=0 preserves the legacy 1-tick gap.
  - screenshot=False emits no screenshot action.
  - The driver source contains the frame-advance barrier guard for
    autoplay/screenshot dispatches.

Covers (PROMPT 1793 — file-ready poll / GAP-SCR-02):
  - wait_for_screenshot_file() returns True when the PNG lands before timeout.
  - wait_for_screenshot_file() returns False and logs a warning on timeout.
  - wait_for_screenshot_file() waits for non-zero size (ignores empty files).
  - driver.py imports screenshot_poll and calls wait_for_screenshot_file after
    a successful autoplay/screenshot RPC.

No GUI, no Bevy launch, no Cargo.  Run with:
    pytest tests/tools/autoplay/test_driver_screenshot_barrier.py -v
"""

from __future__ import annotations

import sys
import time
from pathlib import Path

_TOOLS_AUTOPLAY = Path(__file__).resolve().parents[3] / "tools" / "autoplay"
if str(_TOOLS_AUTOPLAY) not in sys.path:
    sys.path.insert(0, str(_TOOLS_AUTOPLAY))

from recipes._builder import RecipeBuilder  # noqa: E402
from screenshot_poll import wait_for_screenshot_file  # noqa: E402


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


# ---------------------------------------------------------------------------
# File-ready poll unit tests (PROMPT 1793 / GAP-SCR-02)
# ---------------------------------------------------------------------------

class TestWaitForScreenshotFile:
    """Behavioural tests for wait_for_screenshot_file using real tmp files."""

    def test_screenshot_file_ready_poll_returns_true_when_file_exists(self, tmp_path):
        # Arrange: write a non-empty PNG stub before calling the poll
        png = tmp_path / "screenshots" / "000001.png"
        png.parent.mkdir(parents=True)
        png.write_bytes(b"\x89PNG stub")
        messages: list[str] = []

        # Act
        result = wait_for_screenshot_file(png, tick=1, log_fn=messages.append)

        # Assert
        assert result is True
        assert any("file ready" in m for m in messages), (
            "poll must log a 'file ready' line when PNG lands"
        )

    def test_screenshot_file_ready_poll_returns_false_on_timeout(self, tmp_path):
        # Arrange: the PNG is never written
        png = tmp_path / "screenshots" / "000002.png"
        png.parent.mkdir(parents=True)
        messages: list[str] = []

        # Act: use tiny timeout so the test doesn't hang
        result = wait_for_screenshot_file(
            png, tick=2, log_fn=messages.append, poll_interval=0.005, timeout=0.02
        )

        # Assert
        assert result is False
        assert any("WARNING" in m and "timed out" in m for m in messages), (
            "poll must log a WARNING timeout line when PNG never appears"
        )

    def test_screenshot_file_ready_poll_ignores_zero_byte_file(self, tmp_path):
        # Arrange: empty file exists (save_to_disk not yet flushed)
        png = tmp_path / "screenshots" / "000003.png"
        png.parent.mkdir(parents=True)
        png.write_bytes(b"")  # zero bytes — not ready yet
        messages: list[str] = []

        # Act: short timeout so the test is fast; file stays empty
        result = wait_for_screenshot_file(
            png, tick=3, log_fn=messages.append, poll_interval=0.005, timeout=0.02
        )

        # Assert: zero-byte file must not be treated as ready
        assert result is False

    def test_screenshot_file_ready_poll_logs_filename_and_size_on_success(self, tmp_path):
        # Arrange
        png = tmp_path / "screenshots" / "000004.png"
        png.parent.mkdir(parents=True)
        png.write_bytes(b"\x89PNG\r\n\x1a\n" + b"\x00" * 100)
        messages: list[str] = []

        # Act
        wait_for_screenshot_file(png, tick=4, log_fn=messages.append)

        # Assert: success log must include filename and byte count
        assert any("000004.png" in m for m in messages), (
            "success log must include the PNG filename"
        )
        assert any("bytes" in m for m in messages), (
            "success log must include the file size in bytes"
        )

    def test_screenshot_file_ready_poll_includes_tick_in_all_messages(self, tmp_path):
        # Arrange: file never arrives
        png = tmp_path / "screenshots" / "000005.png"
        png.parent.mkdir(parents=True)
        messages: list[str] = []

        # Act
        wait_for_screenshot_file(
            png, tick=42, log_fn=messages.append, poll_interval=0.005, timeout=0.02
        )

        # Assert: all log lines must reference tick=42
        assert all("tick=42" in m for m in messages), (
            "all poll log lines must reference the current driver tick"
        )


# ---------------------------------------------------------------------------
# Driver source contains file-ready poll (structural check, PROMPT 1793)
# ---------------------------------------------------------------------------

class TestDriverFileReadyPollPresent:
    """Structural checks: driver.py must import and call wait_for_screenshot_file."""

    _DRIVER_SOURCE = (_TOOLS_AUTOPLAY / "driver.py").read_text(encoding="utf-8")

    def test_driver_imports_screenshot_poll(self):
        assert "from screenshot_poll import wait_for_screenshot_file" in self._DRIVER_SOURCE, (
            "driver.py must import wait_for_screenshot_file from screenshot_poll"
        )

    def test_driver_calls_wait_for_screenshot_file(self):
        assert "wait_for_screenshot_file(" in self._DRIVER_SOURCE, (
            "driver.py must call wait_for_screenshot_file after the screenshot RPC"
        )

    def test_driver_resolves_relative_path_from_result(self):
        assert 'result.get("relative_path")' in self._DRIVER_SOURCE, (
            "driver.py must extract relative_path from the RPC result dict"
        )

    def test_driver_file_poll_only_runs_for_screenshot_method(self):
        # The guard must be nested inside the autoplay/screenshot branch
        src = self._DRIVER_SOURCE
        screenshot_idx = src.index('method == "autoplay/screenshot"')
        poll_idx = src.index("wait_for_screenshot_file(")
        assert poll_idx > screenshot_idx, (
            "wait_for_screenshot_file call must appear after the autoplay/screenshot method check"
        )
