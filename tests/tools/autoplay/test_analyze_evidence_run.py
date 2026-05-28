"""Tests for analyze_evidence_run.py (PROMPT 1833).

Covers:
  - PASS: clean run, driver exit 0, multiple distinct pixel_hashes, screenshots present
  - PARTIAL: frozen renderer (all pixel_hashes identical)
  - PARTIAL: FROZEN label in driver.log
  - FAIL: driver_exit_code non-zero
  - FAIL: no screenshots captured at all
  - NEEDS_HUMAN_GUI: blocked-human-gui outcome
  - NEEDS_HUMAN_GUI: blocked-precondition outcome
  - Missing launcher-status.json (warning, not crash)
  - Missing driver.log (warning, not crash)
  - JSON output round-trips cleanly
  - Capture label families parsed correctly
  - pixel_hash deduplication

Run with:
    pytest tests/tools/autoplay/test_analyze_evidence_run.py -v
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

import pytest

_TOOLS_AUTOPLAY = Path(__file__).resolve().parents[3] / "tools" / "autoplay"
if str(_TOOLS_AUTOPLAY) not in sys.path:
    sys.path.insert(0, str(_TOOLS_AUTOPLAY))

from analyze_evidence_run import (
    analyze,
    _format_json,
    _format_human,
    EvidenceSummary,
    LAUNCHER_STATUS_FILE,
    DRIVER_LOG_FILE,
    SCREENSHOTS_SUBDIR,
)


# ---------------------------------------------------------------------------
# Fixture helpers
# ---------------------------------------------------------------------------

def _write_launcher_status(run_dir: Path, *, outcome: str = "ok", driver_exit: int = 0,
                            client_exit: int | None = None) -> None:
    data: dict = {
        "schema": "autoplay_launcher_status_v1",
        "outcome": outcome,
        "driver_exit_code": driver_exit,
        "client_exit_code": client_exit,
    }
    (run_dir / LAUNCHER_STATUS_FILE).write_text(json.dumps(data), encoding="utf-8")


def _write_driver_log(run_dir: Path, lines: list[str]) -> None:
    (run_dir / DRIVER_LOG_FILE).write_text("\n".join(lines), encoding="utf-8")


def _make_png(path: Path) -> None:
    """Write a minimal valid 1x1 PNG (no Pillow required)."""
    # Minimal 1x1 red PNG — hardcoded bytes
    PNG_1x1_RED = (
        b'\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01'
        b'\x00\x00\x00\x01\x08\x02\x00\x00\x00\x90wS\xde\x00\x00'
        b'\x00\x0cIDATx\x9cc\xf8\x0f\x00\x00\x01\x01\x00\x05\x18'
        b'\xd8N\x00\x00\x00\x00IEND\xaeB`\x82'
    )
    path.write_bytes(PNG_1x1_RED)


def _make_run_dir(tmp_path: Path) -> Path:
    run_dir = tmp_path / "20260528-120000-Z"
    run_dir.mkdir()
    return run_dir


# ---------------------------------------------------------------------------
# PASS cases
# ---------------------------------------------------------------------------

class TestPassVerdict:
    def test_analyze_clean_run_is_pass(self, tmp_path):
        # Arrange: valid launcher, driver log with 2 distinct hashes, 2 screenshots
        run_dir = _make_run_dir(tmp_path)
        _write_launcher_status(run_dir, outcome="ok", driver_exit=0)
        _write_driver_log(run_dir, [
            "win32_capture: pixel_hash=0xaabbccdd width=1296 height=759",
            "win32_capture: pixel_hash=0x11223344 width=1296 height=759",
        ])
        scr_dir = run_dir / SCREENSHOTS_SUBDIR
        scr_dir.mkdir()
        _make_png(scr_dir / "000000.png")
        _make_png(scr_dir / "000001.png")

        # Act
        summary, verdict, reason = analyze(run_dir)

        # Assert
        assert verdict == "PASS", f"Expected PASS; got {verdict!r}: {reason}"
        assert summary.driver_exit_code == 0
        assert len(summary.distinct_pixel_hashes) == 2

    def test_analyze_pass_has_win32_capture_label(self, tmp_path):
        run_dir = _make_run_dir(tmp_path)
        _write_launcher_status(run_dir, outcome="ok", driver_exit=0)
        _write_driver_log(run_dir, [
            "win32_capture: pixel_hash=0xaabbccdd width=1296 height=759",
            "win32_capture: pixel_hash=0x11223344 width=1296 height=759",
        ])
        scr_dir = run_dir / SCREENSHOTS_SUBDIR
        scr_dir.mkdir()
        _make_png(scr_dir / "000000.png")

        summary, verdict, _ = analyze(run_dir)

        assert "win32_capture" in summary.capture_labels

    def test_analyze_root_win32_pngs_counted(self, tmp_path):
        # Arrange: root-level win32_tick_*.png files
        run_dir = _make_run_dir(tmp_path)
        _write_launcher_status(run_dir, outcome="ok", driver_exit=0)
        _write_driver_log(run_dir, [
            "win32_capture: pixel_hash=0xaabb1122 width=1296 height=759",
            "win32_capture: pixel_hash=0xccdd3344 width=1296 height=759",
        ])
        _make_png(run_dir / "win32_tick_000005.png")
        _make_png(run_dir / "win32_tick_000010.png")

        summary, verdict, _ = analyze(run_dir)

        assert summary.root_win32_png_count == 2
        assert verdict == "PASS"

    def test_analyze_desktop_bitblt_label_detected(self, tmp_path):
        run_dir = _make_run_dir(tmp_path)
        _write_launcher_status(run_dir, outcome="ok", driver_exit=0)
        _write_driver_log(run_dir, [
            "desktop_bitblt: pixel_hash=0xaabb1122 width=1920 height=1080",
            "desktop_bitblt: pixel_hash=0xccdd5566 width=1920 height=1080",
        ])
        scr_dir = run_dir / SCREENSHOTS_SUBDIR
        scr_dir.mkdir()
        _make_png(scr_dir / "000000.png")

        summary, verdict, _ = analyze(run_dir)

        assert "desktop_bitblt" in summary.capture_labels
        assert verdict == "PASS"


# ---------------------------------------------------------------------------
# PARTIAL cases
# ---------------------------------------------------------------------------

class TestPartialVerdict:
    def test_analyze_frozen_pixel_hashes_is_partial(self, tmp_path):
        # Arrange: all pixel_hashes are identical → frozen pattern
        run_dir = _make_run_dir(tmp_path)
        _write_launcher_status(run_dir, outcome="ok", driver_exit=0)
        _write_driver_log(run_dir, [
            "win32_capture: pixel_hash=0x26207c4c width=1296 height=759",
            "win32_capture: pixel_hash=0x26207c4c width=1296 height=759",
            "win32_capture: pixel_hash=0x26207c4c width=1296 height=759",
        ])
        scr_dir = run_dir / SCREENSHOTS_SUBDIR
        scr_dir.mkdir()
        _make_png(scr_dir / "000000.png")

        summary, verdict, reason = analyze(run_dir)

        assert verdict == "PARTIAL", f"Expected PARTIAL for frozen hashes; got {verdict!r}: {reason}"
        assert summary.is_frozen
        assert len(summary.distinct_pixel_hashes) == 1

    def test_analyze_frozen_label_in_log_is_partial(self, tmp_path):
        # Arrange: FROZEN label appears in driver.log
        run_dir = _make_run_dir(tmp_path)
        _write_launcher_status(run_dir, outcome="ok", driver_exit=0)
        _write_driver_log(run_dir, [
            "win32_capture: pixel_hash=0xaabb1122 FROZEN width=1296 height=759",
            "win32_capture: pixel_hash=0xccdd3344 width=1296 height=759",
        ])
        scr_dir = run_dir / SCREENSHOTS_SUBDIR
        scr_dir.mkdir()
        _make_png(scr_dir / "000000.png")

        summary, verdict, reason = analyze(run_dir)

        assert verdict == "PARTIAL", f"Expected PARTIAL for FROZEN label; got {verdict!r}: {reason}"
        assert summary.frozen_labels_count > 0


# ---------------------------------------------------------------------------
# FAIL cases
# ---------------------------------------------------------------------------

class TestFailVerdict:
    def test_analyze_nonzero_driver_exit_is_fail(self, tmp_path):
        # Arrange: driver exited with non-zero code
        run_dir = _make_run_dir(tmp_path)
        _write_launcher_status(run_dir, outcome="error", driver_exit=1)
        _write_driver_log(run_dir, [
            "win32_capture: pixel_hash=0xaabbccdd width=1296 height=759",
        ])
        scr_dir = run_dir / SCREENSHOTS_SUBDIR
        scr_dir.mkdir()
        _make_png(scr_dir / "000000.png")

        summary, verdict, reason = analyze(run_dir)

        assert verdict == "FAIL", f"Expected FAIL for exit code 1; got {verdict!r}: {reason}"
        assert "driver_exit_code=1" in reason

    def test_analyze_no_screenshots_is_fail(self, tmp_path):
        # Arrange: driver exited cleanly but no screenshots at all
        run_dir = _make_run_dir(tmp_path)
        _write_launcher_status(run_dir, outcome="ok", driver_exit=0)
        _write_driver_log(run_dir, [])

        summary, verdict, reason = analyze(run_dir)

        assert verdict == "FAIL", f"Expected FAIL for zero screenshots; got {verdict!r}: {reason}"
        assert "no screenshots" in reason.lower() or "0" in reason

    def test_analyze_run_dir_not_found_is_fail(self, tmp_path):
        run_dir = tmp_path / "does-not-exist"

        summary, verdict, reason = analyze(run_dir)

        assert verdict == "FAIL"
        assert summary.warnings


# ---------------------------------------------------------------------------
# NEEDS_HUMAN_GUI cases
# ---------------------------------------------------------------------------

class TestNeedsHumanGuiVerdict:
    def test_analyze_blocked_human_gui_outcome(self, tmp_path):
        run_dir = _make_run_dir(tmp_path)
        _write_launcher_status(run_dir, outcome="blocked-human-gui", driver_exit=0)
        _write_driver_log(run_dir, [])

        summary, verdict, reason = analyze(run_dir)

        assert verdict == "NEEDS_HUMAN_GUI", f"Got {verdict!r}: {reason}"

    def test_analyze_blocked_precondition_outcome(self, tmp_path):
        run_dir = _make_run_dir(tmp_path)
        _write_launcher_status(run_dir, outcome="blocked-precondition", driver_exit=0)
        _write_driver_log(run_dir, [])

        _, verdict, _ = analyze(run_dir)

        assert verdict == "NEEDS_HUMAN_GUI"

    def test_analyze_blocked_recipe_guard_outcome(self, tmp_path):
        run_dir = _make_run_dir(tmp_path)
        _write_launcher_status(run_dir, outcome="blocked-recipe-guard", driver_exit=0)
        _write_driver_log(run_dir, [])

        _, verdict, _ = analyze(run_dir)

        assert verdict == "NEEDS_HUMAN_GUI"


# ---------------------------------------------------------------------------
# Missing-file resilience
# ---------------------------------------------------------------------------

class TestMissingFiles:
    def test_analyze_missing_launcher_status_warns_not_crashes(self, tmp_path):
        # Arrange: no launcher-status.json
        run_dir = _make_run_dir(tmp_path)
        _write_driver_log(run_dir, [
            "win32_capture: pixel_hash=0xaabbccdd width=1296 height=759",
            "win32_capture: pixel_hash=0x11223344 width=1296 height=759",
        ])
        scr_dir = run_dir / SCREENSHOTS_SUBDIR
        scr_dir.mkdir()
        _make_png(scr_dir / "000000.png")

        summary, verdict, reason = analyze(run_dir)

        assert any(LAUNCHER_STATUS_FILE in w for w in summary.warnings)
        # Should not crash; verdict derived from what is available
        assert verdict in ("PASS", "PARTIAL", "FAIL", "NEEDS_HUMAN_GUI")

    def test_analyze_missing_driver_log_warns_not_crashes(self, tmp_path):
        # Arrange: no driver.log
        run_dir = _make_run_dir(tmp_path)
        _write_launcher_status(run_dir, outcome="ok", driver_exit=0)
        scr_dir = run_dir / SCREENSHOTS_SUBDIR
        scr_dir.mkdir()
        _make_png(scr_dir / "000000.png")

        summary, verdict, reason = analyze(run_dir)

        assert any(DRIVER_LOG_FILE in w for w in summary.warnings)
        assert verdict in ("PASS", "PARTIAL", "FAIL", "NEEDS_HUMAN_GUI")


# ---------------------------------------------------------------------------
# pixel_hash parsing
# ---------------------------------------------------------------------------

class TestPixelHashParsing:
    def test_pixel_hash_deduplication(self, tmp_path):
        run_dir = _make_run_dir(tmp_path)
        _write_launcher_status(run_dir, outcome="ok", driver_exit=0)
        _write_driver_log(run_dir, [
            "win32_capture: pixel_hash=0xAAAABBBB width=1296 height=759",
            "win32_capture: pixel_hash=0xaaaabbbb width=1296 height=759",  # same, different case
            "win32_capture: pixel_hash=0x11223344 width=1296 height=759",
        ])
        scr_dir = run_dir / SCREENSHOTS_SUBDIR
        scr_dir.mkdir()
        _make_png(scr_dir / "000000.png")

        summary, _, _ = analyze(run_dir)

        # 0xAAAABBBB and 0xaaaabbbb should normalise to the same value
        assert len(summary.distinct_pixel_hashes) == 2

    def test_pixel_hash_total_vs_distinct(self, tmp_path):
        run_dir = _make_run_dir(tmp_path)
        _write_launcher_status(run_dir, outcome="ok", driver_exit=0)
        _write_driver_log(run_dir, [
            "win32_capture: pixel_hash=0xaabb1122 width=1296 height=759",
            "win32_capture: pixel_hash=0xaabb1122 width=1296 height=759",
            "win32_capture: pixel_hash=0xccdd3344 width=1296 height=759",
        ])
        scr_dir = run_dir / SCREENSHOTS_SUBDIR
        scr_dir.mkdir()
        _make_png(scr_dir / "000000.png")

        summary, _, _ = analyze(run_dir)

        assert len(summary.pixel_hashes) == 3
        assert len(summary.distinct_pixel_hashes) == 2

    def test_no_pixel_hashes_in_empty_log(self, tmp_path):
        run_dir = _make_run_dir(tmp_path)
        _write_launcher_status(run_dir, outcome="ok", driver_exit=0)
        _write_driver_log(run_dir, ["some line with no hash"])

        summary, _, _ = analyze(run_dir)

        assert summary.pixel_hashes == []
        assert summary.distinct_pixel_hashes == []


# ---------------------------------------------------------------------------
# JSON output
# ---------------------------------------------------------------------------

class TestJsonOutput:
    def test_json_output_is_valid_json(self, tmp_path):
        run_dir = _make_run_dir(tmp_path)
        _write_launcher_status(run_dir, outcome="ok", driver_exit=0)
        _write_driver_log(run_dir, [
            "win32_capture: pixel_hash=0xaabbccdd width=1296 height=759",
            "win32_capture: pixel_hash=0x11223344 width=1296 height=759",
        ])
        scr_dir = run_dir / SCREENSHOTS_SUBDIR
        scr_dir.mkdir()
        _make_png(scr_dir / "000000.png")

        summary, verdict, reason = analyze(run_dir)
        out = _format_json(run_dir, summary, verdict, reason)

        obj = json.loads(out)
        assert obj["verdict"] == "PASS"
        assert "pixel_hash" in obj
        assert "launcher" in obj

    def test_json_output_has_required_keys(self, tmp_path):
        run_dir = _make_run_dir(tmp_path)
        _write_launcher_status(run_dir)
        _write_driver_log(run_dir, [])

        summary, verdict, reason = analyze(run_dir)
        obj = json.loads(_format_json(run_dir, summary, verdict, reason))

        for key in ("run_dir", "launcher", "capture", "screenshots", "pixel_hash",
                    "warnings", "verdict", "reason"):
            assert key in obj, f"Missing key {key!r} in JSON output"

    def test_json_capture_labels_sorted(self, tmp_path):
        run_dir = _make_run_dir(tmp_path)
        _write_launcher_status(run_dir, outcome="ok", driver_exit=0)
        _write_driver_log(run_dir, [
            "desktop_bitblt: pixel_hash=0xaabb1122",
            "win32_capture: pixel_hash=0xccdd3344",
        ])
        scr_dir = run_dir / SCREENSHOTS_SUBDIR
        scr_dir.mkdir()
        _make_png(scr_dir / "000000.png")

        summary, verdict, reason = analyze(run_dir)
        obj = json.loads(_format_json(run_dir, summary, verdict, reason))

        labels = obj["capture"]["label_families"]
        assert labels == sorted(labels), "label_families should be sorted"


# ---------------------------------------------------------------------------
# Human output smoke
# ---------------------------------------------------------------------------

class TestHumanOutput:
    def test_human_output_contains_verdict(self, tmp_path):
        run_dir = _make_run_dir(tmp_path)
        _write_launcher_status(run_dir, outcome="ok", driver_exit=0)
        _write_driver_log(run_dir, [
            "win32_capture: pixel_hash=0xaabb1122 width=1296 height=759",
            "win32_capture: pixel_hash=0xccdd3344 width=1296 height=759",
        ])
        scr_dir = run_dir / SCREENSHOTS_SUBDIR
        scr_dir.mkdir()
        _make_png(scr_dir / "000000.png")

        summary, verdict, reason = analyze(run_dir)
        out = _format_human(run_dir, summary, verdict, reason)

        assert "VERDICT: PASS" in out
        assert "REASON" in out
