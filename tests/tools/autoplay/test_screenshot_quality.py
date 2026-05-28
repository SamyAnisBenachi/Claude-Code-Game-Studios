"""Tests for screenshot quality checks in validate_composite_run.py (PROMPT 1796).

Covers:
  - pass: valid screenshots (distinct hashes, non-dark)
  - identical-fail: all screenshots share the same content hash
  - black-fail: one or more screenshots are near-black (mean brightness < threshold)
  - missing-screenshot fail: screenshots/ dir exists but is empty

Tests use synthetic PNG files created with Pillow so they run without a game
client, Cargo, or network access.

Run with:
    pytest tests/tools/autoplay/test_screenshot_quality.py -v
"""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any

import pytest

_TOOLS_AUTOPLAY = Path(__file__).resolve().parents[3] / "tools" / "autoplay"
if str(_TOOLS_AUTOPLAY) not in sys.path:
    sys.path.insert(0, str(_TOOLS_AUTOPLAY))

from validate_composite_run import (
    EXPECTED_SCHEMA,
    MIN_SCREENSHOT_COUNT,
    MIN_UNIQUE_HASH_COUNT,
    NEAR_BLACK_BRIGHTNESS_THRESHOLD,
    SCREENSHOTS_SUBDIR,
    SUMMARY_FILENAME,
    RUN_PATH_FILENAME,
    _check_screenshot_quality,
    _Result,
    validate,
)

# ---------------------------------------------------------------------------
# PNG fixture helpers
# ---------------------------------------------------------------------------

def _require_pil() -> None:
    """Skip test if Pillow is not available."""
    try:
        import PIL  # noqa: F401
    except ImportError:
        pytest.skip("Pillow not installed — brightness tests require PIL")


def _make_png(path: Path, brightness: int, width: int = 4, height: int = 4) -> None:
    """Write a small solid-colour grayscale PNG with the given brightness (0–255)."""
    _require_pil()
    from PIL import Image
    img = Image.new("L", (width, height), color=brightness)
    img.save(str(path), format="PNG")


def _make_rgb_png(path: Path, r: int, g: int, b: int, width: int = 4, height: int = 4) -> None:
    """Write a small solid RGB PNG."""
    _require_pil()
    from PIL import Image
    img = Image.new("RGB", (width, height), color=(r, g, b))
    img.save(str(path), format="PNG")


# ---------------------------------------------------------------------------
# Evidence-dir fixture (minimal, reused from test_validate_composite_run.py
# pattern but scoped to this module to stay independent)
# ---------------------------------------------------------------------------

_VALID_SUMMARY: dict[str, Any] = {
    "schema": EXPECTED_SCHEMA,
    "prompt": "PROMPT-1644",
    "outcome": "ok",
    "recipe": "smoke",           # smoke → no required checkpoints
    "soak_port": 5000,
    "rpc_port": 15873,
    "skip_soak_launch": False,
    "soak_duration_secs": 300,
    "smoke_exit_code": 0,
    "autoplay_artifact_dir": "",
    "evidence_dir": "",
    "dry_run": False,
    "generated_utc": "2026-05-27T12:00:00.000000+00:00",
    "live_pass_status": (
        "NOT-CLAIMED -- AUTOPLAY-VS-BOT-QA-001 requires human operator "
        "sign-off for live PASS evidence"
    ),
    "notes": "PROMPT 1796 screenshot quality test fixture.",
}


def _make_evidence_dir(
    tmp_path: Path,
    *,
    summary_override: dict[str, Any] | None = None,
    with_screenshots: bool = False,
) -> tuple[Path, Path]:
    """Build a minimal synthetic evidence directory.

    Returns (evidence_dir, artifact_dir).  The artifact_dir is always created
    with a minimal launcher-status.json.  A screenshots/ subdirectory is only
    created when *with_screenshots* is True (caller populates it).
    """
    evidence_dir = tmp_path / "2026-05-27-120000-autoplay-vs-bot"
    evidence_dir.mkdir()

    artifact_dir = tmp_path / "autoplay-runs" / "20260527-120001-Z"
    artifact_dir.mkdir(parents=True)
    (artifact_dir / "launcher-status.json").write_text(
        json.dumps({"schema": "autoplay_launcher_status_v1", "outcome": "ok"}),
        encoding="utf-8",
    )

    if with_screenshots:
        (artifact_dir / SCREENSHOTS_SUBDIR).mkdir()

    summary = dict(_VALID_SUMMARY)
    summary["autoplay_artifact_dir"] = str(artifact_dir)
    summary["evidence_dir"] = str(evidence_dir)
    if summary_override:
        summary.update(summary_override)

    (evidence_dir / SUMMARY_FILENAME).write_text(json.dumps(summary), encoding="utf-8")
    (evidence_dir / RUN_PATH_FILENAME).write_text(str(artifact_dir), encoding="utf-8")

    return evidence_dir, artifact_dir


# ---------------------------------------------------------------------------
# Unit-level tests targeting _check_screenshot_quality directly
# ---------------------------------------------------------------------------

class TestScreenshotQualityUnit:
    """Tests that call _check_screenshot_quality() directly for isolation."""

    def test_screenshot_quality_no_screenshots_dir_warns_not_fails(self, tmp_path):
        # Arrange: artifact dir without a screenshots/ subdir
        artifact_dir = tmp_path / "artifact"
        artifact_dir.mkdir()
        result = _Result()
        # Act
        _check_screenshot_quality(artifact_dir, result)
        # Assert: warning, no failure
        assert result.ok
        assert any(SCREENSHOTS_SUBDIR in w for w in result.warnings)

    def test_screenshot_quality_empty_screenshots_dir_fails(self, tmp_path):
        # Arrange: screenshots/ exists but has zero PNGs
        artifact_dir = tmp_path / "artifact"
        artifact_dir.mkdir()
        (artifact_dir / SCREENSHOTS_SUBDIR).mkdir()
        result = _Result()
        # Act
        _check_screenshot_quality(artifact_dir, result)
        # Assert
        assert not result.ok
        assert any("MISSING-SCREENSHOTS" in f for f in result.failures)

    def test_screenshot_quality_single_bright_png_passes(self, tmp_path):
        # Arrange: one bright PNG → count OK, brightness OK, identity check skipped
        _require_pil()
        artifact_dir = tmp_path / "artifact"
        artifact_dir.mkdir()
        scr_dir = artifact_dir / SCREENSHOTS_SUBDIR
        scr_dir.mkdir()
        _make_png(scr_dir / "001.png", brightness=200)
        result = _Result()
        # Act
        _check_screenshot_quality(artifact_dir, result)
        # Assert
        assert result.ok, f"Expected PASS; failures: {result.failures}"

    def test_screenshot_quality_two_distinct_bright_pngs_passes(self, tmp_path):
        # Arrange: two PNGs with different brightness → distinct hashes, non-dark
        _require_pil()
        artifact_dir = tmp_path / "artifact"
        artifact_dir.mkdir()
        scr_dir = artifact_dir / SCREENSHOTS_SUBDIR
        scr_dir.mkdir()
        _make_png(scr_dir / "001.png", brightness=200)
        _make_png(scr_dir / "002.png", brightness=150)
        result = _Result()
        # Act
        _check_screenshot_quality(artifact_dir, result)
        # Assert
        assert result.ok, f"Expected PASS; failures: {result.failures}"

    def test_screenshot_quality_identical_pngs_fail(self, tmp_path):
        # Arrange: two PNGs that are byte-for-byte identical
        _require_pil()
        artifact_dir = tmp_path / "artifact"
        artifact_dir.mkdir()
        scr_dir = artifact_dir / SCREENSHOTS_SUBDIR
        scr_dir.mkdir()
        _make_png(scr_dir / "001.png", brightness=180)
        # Write identical content to second file
        import shutil
        shutil.copy(scr_dir / "001.png", scr_dir / "002.png")
        result = _Result()
        # Act
        _check_screenshot_quality(artifact_dir, result)
        # Assert
        assert not result.ok
        assert any("IDENTICAL-SCREENSHOTS" in f for f in result.failures), (
            f"Expected IDENTICAL-SCREENSHOTS failure; got: {result.failures}"
        )

    def test_screenshot_quality_identical_pngs_failure_names_files(self, tmp_path):
        # Verify that the IDENTICAL-SCREENSHOTS failure message includes filenames
        _require_pil()
        artifact_dir = tmp_path / "artifact"
        artifact_dir.mkdir()
        scr_dir = artifact_dir / SCREENSHOTS_SUBDIR
        scr_dir.mkdir()
        _make_png(scr_dir / "001.png", brightness=180)
        import shutil
        shutil.copy(scr_dir / "001.png", scr_dir / "002.png")
        result = _Result()
        _check_screenshot_quality(artifact_dir, result)
        identical_failures = [f for f in result.failures if "IDENTICAL-SCREENSHOTS" in f]
        assert identical_failures, "Expected an IDENTICAL-SCREENSHOTS failure"
        msg = identical_failures[0]
        assert "001.png" in msg or "002.png" in msg, (
            f"Failure message should name the offending files; got: {msg}"
        )

    def test_screenshot_quality_near_black_png_fails(self, tmp_path):
        # Arrange: one PNG with mean brightness below threshold
        _require_pil()
        artifact_dir = tmp_path / "artifact"
        artifact_dir.mkdir()
        scr_dir = artifact_dir / SCREENSHOTS_SUBDIR
        scr_dir.mkdir()
        dark_brightness = max(0, NEAR_BLACK_BRIGHTNESS_THRESHOLD - 5)
        _make_png(scr_dir / "001.png", brightness=dark_brightness)
        result = _Result()
        # Act
        _check_screenshot_quality(artifact_dir, result)
        # Assert
        assert not result.ok
        assert any("NEAR-BLACK-SCREENSHOT" in f for f in result.failures), (
            f"Expected NEAR-BLACK-SCREENSHOT failure; got: {result.failures}"
        )

    def test_screenshot_quality_near_black_failure_names_file(self, tmp_path):
        # Verify that the NEAR-BLACK failure message includes the offending filename
        _require_pil()
        artifact_dir = tmp_path / "artifact"
        artifact_dir.mkdir()
        scr_dir = artifact_dir / SCREENSHOTS_SUBDIR
        scr_dir.mkdir()
        _make_png(scr_dir / "bad-capture.png", brightness=0)
        result = _Result()
        _check_screenshot_quality(artifact_dir, result)
        near_black = [f for f in result.failures if "NEAR-BLACK-SCREENSHOT" in f]
        assert near_black, "Expected NEAR-BLACK-SCREENSHOT failure"
        assert "bad-capture.png" in near_black[0], (
            f"Failure should name the offending file; got: {near_black[0]}"
        )

    def test_screenshot_quality_fully_black_png_fails(self, tmp_path):
        # brightness=0 is all-black
        _require_pil()
        artifact_dir = tmp_path / "artifact"
        artifact_dir.mkdir()
        scr_dir = artifact_dir / SCREENSHOTS_SUBDIR
        scr_dir.mkdir()
        _make_png(scr_dir / "001.png", brightness=0)
        result = _Result()
        _check_screenshot_quality(artifact_dir, result)
        assert not result.ok
        assert any("NEAR-BLACK-SCREENSHOT" in f for f in result.failures)

    def test_screenshot_quality_mixed_one_dark_one_bright_fails(self, tmp_path):
        # One bad screenshot in a multi-screenshot run must still fail
        _require_pil()
        artifact_dir = tmp_path / "artifact"
        artifact_dir.mkdir()
        scr_dir = artifact_dir / SCREENSHOTS_SUBDIR
        scr_dir.mkdir()
        _make_png(scr_dir / "001.png", brightness=200)   # bright — OK
        _make_png(scr_dir / "002.png", brightness=0)     # all-black — FAIL
        result = _Result()
        _check_screenshot_quality(artifact_dir, result)
        assert not result.ok
        assert any("NEAR-BLACK-SCREENSHOT" in f for f in result.failures)
        # The bright screenshot should NOT generate a failure
        dark_failures = [f for f in result.failures if "NEAR-BLACK-SCREENSHOT" in f]
        assert all("002.png" in f for f in dark_failures), (
            f"Only 002.png should be cited as near-black; got: {dark_failures}"
        )

    def test_screenshot_quality_threshold_boundary_just_above_passes(self, tmp_path):
        # brightness == threshold is not near-black (strictly <)
        _require_pil()
        artifact_dir = tmp_path / "artifact"
        artifact_dir.mkdir()
        scr_dir = artifact_dir / SCREENSHOTS_SUBDIR
        scr_dir.mkdir()
        _make_png(scr_dir / "001.png", brightness=NEAR_BLACK_BRIGHTNESS_THRESHOLD)
        result = _Result()
        _check_screenshot_quality(artifact_dir, result)
        assert result.ok, (
            f"Brightness exactly at threshold should pass; failures: {result.failures}"
        )

    def test_screenshot_quality_threshold_boundary_just_below_fails(self, tmp_path):
        # brightness == threshold - 1 is near-black
        _require_pil()
        if NEAR_BLACK_BRIGHTNESS_THRESHOLD == 0:
            pytest.skip("Threshold is 0; below-threshold case not applicable")
        artifact_dir = tmp_path / "artifact"
        artifact_dir.mkdir()
        scr_dir = artifact_dir / SCREENSHOTS_SUBDIR
        scr_dir.mkdir()
        _make_png(scr_dir / "001.png", brightness=NEAR_BLACK_BRIGHTNESS_THRESHOLD - 1)
        result = _Result()
        _check_screenshot_quality(artifact_dir, result)
        assert not result.ok
        assert any("NEAR-BLACK-SCREENSHOT" in f for f in result.failures)


# ---------------------------------------------------------------------------
# Integration-level tests: screenshot checks run end-to-end through validate()
# ---------------------------------------------------------------------------

class TestScreenshotQualityIntegration:
    """Tests that exercise screenshot checks through the top-level validate() call."""

    def test_validate_with_valid_screenshots_passes(self, tmp_path):
        _require_pil()
        evidence_dir, artifact_dir = _make_evidence_dir(
            tmp_path, with_screenshots=True
        )
        scr_dir = artifact_dir / SCREENSHOTS_SUBDIR
        _make_png(scr_dir / "001.png", brightness=200)
        _make_png(scr_dir / "002.png", brightness=150)
        result = validate(evidence_dir)
        assert result.ok, f"Expected PASS; failures: {result.failures}"

    def test_validate_with_identical_screenshots_fails(self, tmp_path):
        _require_pil()
        import shutil
        evidence_dir, artifact_dir = _make_evidence_dir(
            tmp_path, with_screenshots=True
        )
        scr_dir = artifact_dir / SCREENSHOTS_SUBDIR
        _make_png(scr_dir / "001.png", brightness=180)
        shutil.copy(scr_dir / "001.png", scr_dir / "002.png")
        result = validate(evidence_dir)
        assert not result.ok
        assert any("IDENTICAL-SCREENSHOTS" in f for f in result.failures)

    def test_validate_with_black_screenshot_fails(self, tmp_path):
        _require_pil()
        evidence_dir, artifact_dir = _make_evidence_dir(
            tmp_path, with_screenshots=True
        )
        scr_dir = artifact_dir / SCREENSHOTS_SUBDIR
        _make_png(scr_dir / "001.png", brightness=0)
        result = validate(evidence_dir)
        assert not result.ok
        assert any("NEAR-BLACK-SCREENSHOT" in f for f in result.failures)

    def test_validate_with_empty_screenshots_dir_fails(self, tmp_path):
        evidence_dir, artifact_dir = _make_evidence_dir(
            tmp_path, with_screenshots=True
        )
        # screenshots/ is empty — no PNGs written
        result = validate(evidence_dir)
        assert not result.ok
        assert any("MISSING-SCREENSHOTS" in f for f in result.failures)

    def test_validate_without_screenshots_dir_passes_with_warning(self, tmp_path):
        evidence_dir, artifact_dir = _make_evidence_dir(
            tmp_path, with_screenshots=False
        )
        result = validate(evidence_dir)
        assert result.ok, f"Expected PASS; failures: {result.failures}"
        assert any(SCREENSHOTS_SUBDIR in w for w in result.warnings)

    def test_validate_screenshot_failures_do_not_suppress_other_failures(self, tmp_path):
        # Both a near-black screenshot and a wrong schema should appear in failures
        _require_pil()
        evidence_dir, artifact_dir = _make_evidence_dir(
            tmp_path,
            with_screenshots=True,
            summary_override={"schema": "wrong_schema_v0"},
        )
        scr_dir = artifact_dir / SCREENSHOTS_SUBDIR
        _make_png(scr_dir / "001.png", brightness=0)
        result = validate(evidence_dir)
        assert not result.ok
        # Both failure kinds must be present
        has_schema = any("SCHEMA MISMATCH" in f for f in result.failures)
        has_dark = any("NEAR-BLACK-SCREENSHOT" in f for f in result.failures)
        assert has_schema, f"Expected SCHEMA MISMATCH in failures: {result.failures}"
        assert has_dark, f"Expected NEAR-BLACK-SCREENSHOT in failures: {result.failures}"


# ---------------------------------------------------------------------------
# Constant sanity checks
# ---------------------------------------------------------------------------

class TestScreenshotQualityConstants:
    def test_near_black_threshold_is_positive(self):
        assert NEAR_BLACK_BRIGHTNESS_THRESHOLD > 0, (
            "NEAR_BLACK_BRIGHTNESS_THRESHOLD must be > 0 to catch all-zero captures"
        )

    def test_min_screenshot_count_is_positive(self):
        assert MIN_SCREENSHOT_COUNT >= 1

    def test_min_unique_hash_count_is_at_least_two(self):
        assert MIN_UNIQUE_HASH_COUNT >= 2, (
            "MIN_UNIQUE_HASH_COUNT < 2 cannot detect byte-identical pairs"
        )

    def test_screenshots_subdir_name(self):
        assert SCREENSHOTS_SUBDIR == "screenshots"
