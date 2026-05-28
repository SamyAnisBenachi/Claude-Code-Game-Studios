"""Tests for window resize / capture quality verdict downgrade (PROMPT 1850).

Covers:
  - analyze_evidence_run: window resize → NEEDS_HUMAN_GUI
  - analyze_evidence_run: below-min window height (no resize) → NEEDS_HUMAN_GUI
  - analyze_evidence_run: all win32 PrintWindow captures frozen → NEEDS_HUMAN_GUI
  - analyze_evidence_run: normal run (no resize, good captures) → PASS (not downgraded)
  - validate_composite_run: WINDOW-RESIZE-DETECTED failure
  - validate_composite_run: WINDOW-HEIGHT-TOO-SMALL failure
  - validate_composite_run: WIN32-ALL-FROZEN failure
  - validate_composite_run: clean timeline + good captures → no new failures

Run with:
    pytest tests/tools/autoplay/test_window_resize_verdict.py -v
"""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any

import pytest

# Make tools/autoplay importable without installing.
_TOOLS_AUTOPLAY = Path(__file__).resolve().parents[3] / "tools" / "autoplay"
if str(_TOOLS_AUTOPLAY) not in sys.path:
    sys.path.insert(0, str(_TOOLS_AUTOPLAY))

from analyze_evidence_run import (
    MIN_WINDOW_HEIGHT,
    EvidenceSummary,
    analyze,
    _parse_driver_timeline,
    _parse_driver_log,
)
from validate_composite_run import (
    EXPECTED_SCHEMA,
    SUMMARY_FILENAME,
    RUN_PATH_FILENAME,
    validate,
    MIN_WINDOW_HEIGHT as VCR_MIN_WINDOW_HEIGHT,
)


# ---------------------------------------------------------------------------
# Shared helpers
# ---------------------------------------------------------------------------

def _write_timeline(run_dir: Path, sizes: list[tuple[int, int]]) -> None:
    """Write driver-timeline.jsonl with the given window_logical_size sequence."""
    lines = []
    for i, (w, h) in enumerate(sizes):
        row = {
            "tick": i + 1,
            "recipe": "vs-bot",
            "elapsed_secs": float(i),
            "status": {"window_logical_size": [w, h], "frame": i},
            "action_results": [],
        }
        lines.append(json.dumps(row))
    (run_dir / "driver-timeline.jsonl").write_text(
        "\n".join(lines) + "\n", encoding="utf-8"
    )


def _write_driver_log(run_dir: Path, log_lines: list[str]) -> None:
    (run_dir / "driver.log").write_text("\n".join(log_lines) + "\n", encoding="utf-8")


def _write_launcher_status(run_dir: Path, outcome: str = "ok") -> None:
    (run_dir / "launcher-status.json").write_text(
        json.dumps({"schema": "autoplay_launcher_status_v1", "outcome": outcome,
                    "driver_exit_code": 0, "client_exit_code": 0}),
        encoding="utf-8",
    )


def _make_run_dir(
    tmp_path: Path,
    sizes: list[tuple[int, int]],
    log_lines: list[str] | None = None,
    outcome: str = "ok",
) -> Path:
    run_dir = tmp_path / "run"
    run_dir.mkdir()
    _write_launcher_status(run_dir, outcome)
    _write_timeline(run_dir, sizes)
    if log_lines is not None:
        _write_driver_log(run_dir, log_lines)
    # At least one screenshot so total_screenshots > 0 and we don't hit FAIL
    (run_dir / "win32_tick_000001.png").write_bytes(b"\x89PNG\r\n\x1a\n" + b"\x00" * 20)
    return run_dir


# ---------------------------------------------------------------------------
# analyze_evidence_run — window size parsing
# ---------------------------------------------------------------------------

class TestParseDriverTimeline:
    def test_single_stable_size_no_resize_event(self, tmp_path):
        run_dir = tmp_path / "run"
        run_dir.mkdir()
        summary = EvidenceSummary()
        _write_timeline(run_dir, [(1280, 720)] * 5)
        _parse_driver_timeline(run_dir, summary)
        assert summary.initial_window_size == (1280, 720)
        assert summary.final_window_size == (1280, 720)
        assert summary.window_resize_event_count == 0
        assert summary.min_window_height_seen == 720

    def test_resize_event_counted(self, tmp_path):
        run_dir = tmp_path / "run"
        run_dir.mkdir()
        summary = EvidenceSummary()
        # 1280x720 → 1280x505 → 1280x1076
        _write_timeline(run_dir, [(1280, 720), (1280, 720), (1280, 505), (1280, 1076)])
        _parse_driver_timeline(run_dir, summary)
        assert summary.initial_window_size == (1280, 720)
        assert summary.final_window_size == (1280, 1076)
        assert summary.window_resize_event_count == 2  # 720→505, 505→1076
        assert summary.min_window_height_seen == 505

    def test_missing_timeline_adds_warning(self, tmp_path):
        run_dir = tmp_path / "run"
        run_dir.mkdir()
        summary = EvidenceSummary()
        _parse_driver_timeline(run_dir, summary)
        assert any("driver-timeline.jsonl" in w for w in summary.warnings)
        assert summary.initial_window_size is None

    def test_empty_timeline_leaves_fields_none(self, tmp_path):
        run_dir = tmp_path / "run"
        run_dir.mkdir()
        (run_dir / "driver-timeline.jsonl").write_text("", encoding="utf-8")
        summary = EvidenceSummary()
        _parse_driver_timeline(run_dir, summary)
        assert summary.initial_window_size is None
        assert summary.final_window_size is None
        assert summary.window_resize_event_count == 0


# ---------------------------------------------------------------------------
# analyze_evidence_run — win32 capture quality
# ---------------------------------------------------------------------------

class TestWin32CaptureQuality:
    def test_all_ok_is_good(self, tmp_path):
        run_dir = tmp_path / "run"
        run_dir.mkdir()
        summary = EvidenceSummary()
        lines = [
            "2026-01-01T00:00:01Z tick=1 win32_printwindow=OK path=win32_tick_000001.png",
            "2026-01-01T00:00:02Z tick=2 win32_printwindow=OK path=win32_tick_000002.png",
        ]
        _write_driver_log(run_dir, lines)
        _parse_driver_log(run_dir, summary)
        assert summary.win32_ok_count == 2
        assert summary.win32_failed_or_frozen_count == 0
        assert summary.win32_capture_quality == "GOOD"

    def test_all_frozen_is_all_frozen(self, tmp_path):
        run_dir = tmp_path / "run"
        run_dir.mkdir()
        summary = EvidenceSummary()
        lines = [
            "2026-01-01T00:00:01Z tick=1 win32_printwindow=FROZEN hash=0xabc",
            "2026-01-01T00:00:02Z tick=2 win32_printwindow=FAILED path=win32_tick_000002.png",
        ]
        _write_driver_log(run_dir, lines)
        _parse_driver_log(run_dir, summary)
        assert summary.win32_ok_count == 0
        assert summary.win32_failed_or_frozen_count == 2
        assert summary.win32_capture_quality == "ALL_FROZEN"

    def test_mixed_is_partial_frozen(self, tmp_path):
        run_dir = tmp_path / "run"
        run_dir.mkdir()
        summary = EvidenceSummary()
        lines = [
            "2026-01-01T00:00:01Z tick=1 win32_printwindow=OK path=win32_tick_000001.png",
            "2026-01-01T00:00:02Z tick=2 win32_printwindow=FROZEN hash=0xabc",
        ]
        _write_driver_log(run_dir, lines)
        _parse_driver_log(run_dir, summary)
        assert summary.win32_capture_quality == "PARTIAL_FROZEN"

    def test_no_win32_lines_is_unknown(self, tmp_path):
        run_dir = tmp_path / "run"
        run_dir.mkdir()
        summary = EvidenceSummary()
        _write_driver_log(run_dir, ["2026-01-01T00:00:01Z some unrelated line"])
        _parse_driver_log(run_dir, summary)
        assert summary.win32_capture_quality == "UNKNOWN"


# ---------------------------------------------------------------------------
# analyze_evidence_run — verdict downgrade
# ---------------------------------------------------------------------------

class TestAnalyzeVerdictDowngrade:
    def test_window_resize_triggers_needs_human_gui(self, tmp_path):
        # Arrange: run with mid-run resize 720→505→1076
        log_lines = [
            "2026-01-01T00:00:01Z tick=1 win32_printwindow=OK path=win32_tick_000001.png",
            "pixel_hash=0xaabbccdd",
        ]
        run_dir = _make_run_dir(
            tmp_path,
            sizes=[(1280, 720), (1280, 505), (1280, 1076)],
            log_lines=log_lines,
        )
        # Act
        summary, verdict, reason = analyze(run_dir)
        # Assert
        assert verdict == "NEEDS_HUMAN_GUI", f"Expected NEEDS_HUMAN_GUI, got {verdict}: {reason}"
        assert "mid-run window resize" in reason
        assert "2" in reason  # 2 resize events

    def test_below_min_height_no_resize_triggers_needs_human_gui(self, tmp_path):
        # Arrange: window always at 1280x400 (below MIN_WINDOW_HEIGHT=600), no resize
        log_lines = [
            "2026-01-01T00:00:01Z tick=1 win32_printwindow=OK path=win32_tick_000001.png",
            "pixel_hash=0xaabbccdd",
        ]
        run_dir = _make_run_dir(
            tmp_path,
            sizes=[(1280, 400)] * 5,
            log_lines=log_lines,
        )
        summary, verdict, reason = analyze(run_dir)
        assert verdict == "NEEDS_HUMAN_GUI", f"Expected NEEDS_HUMAN_GUI, got {verdict}: {reason}"
        assert "below minimum" in reason
        assert str(MIN_WINDOW_HEIGHT) in reason

    def test_all_frozen_win32_triggers_needs_human_gui(self, tmp_path):
        # Arrange: all win32 captures are frozen/failed, good window size
        log_lines = [
            "2026-01-01T00:00:01Z tick=1 win32_printwindow=FROZEN hash=0xdeadbeef",
            "2026-01-01T00:00:02Z tick=2 win32_printwindow=FROZEN hash=0xdeadbeef",
            "pixel_hash=0xaabbccdd",
        ]
        run_dir = _make_run_dir(
            tmp_path,
            sizes=[(1280, 720)] * 5,
            log_lines=log_lines,
        )
        summary, verdict, reason = analyze(run_dir)
        assert verdict == "NEEDS_HUMAN_GUI", f"Expected NEEDS_HUMAN_GUI, got {verdict}: {reason}"
        assert "ALL_FROZEN" in reason or "all win32" in reason.lower()

    def test_clean_run_is_not_downgraded(self, tmp_path):
        # Arrange: stable window size, good win32 captures, distinct pixel hashes
        log_lines = [
            "2026-01-01T00:00:01Z tick=1 win32_printwindow=OK path=win32_tick_000001.png",
            "pixel_hash=0xaabbccdd",
            "2026-01-01T00:00:02Z tick=2 win32_printwindow=OK path=win32_tick_000002.png",
            "pixel_hash=0x11223344",
        ]
        run_dir = _make_run_dir(
            tmp_path,
            sizes=[(1280, 720)] * 5,
            log_lines=log_lines,
        )
        summary, verdict, reason = analyze(run_dir)
        assert verdict in ("PASS", "PARTIAL"), (
            f"Clean run should not be downgraded to {verdict}: {reason}"
        )

    def test_window_resize_into_acceptable_height_still_fails(self, tmp_path):
        # Resize from 720 to 800 — still a resize event, still NEEDS_HUMAN_GUI
        log_lines = [
            "2026-01-01T00:00:01Z tick=1 win32_printwindow=OK path=win32_tick_000001.png",
            "pixel_hash=0xaabbccdd",
        ]
        run_dir = _make_run_dir(
            tmp_path,
            sizes=[(1280, 720), (1280, 800)],
            log_lines=log_lines,
        )
        summary, verdict, reason = analyze(run_dir)
        assert verdict == "NEEDS_HUMAN_GUI"
        assert "mid-run window resize" in reason

    def test_blocked_outcome_takes_priority_over_window_resize(self, tmp_path):
        # blocked-recipe-guard should produce NEEDS_HUMAN_GUI for the blocked reason,
        # not the window resize reason (blocked check runs first)
        log_lines = [
            "2026-01-01T00:00:01Z tick=1 win32_printwindow=OK path=win32_tick_000001.png",
        ]
        run_dir = tmp_path / "run"
        run_dir.mkdir()
        _write_launcher_status(run_dir, outcome="blocked-recipe-guard")
        _write_timeline(run_dir, [(1280, 720), (1280, 505)])
        _write_driver_log(run_dir, log_lines)
        (run_dir / "win32_tick_000001.png").write_bytes(b"\x89PNG\r\n\x1a\n" + b"\x00" * 20)
        summary, verdict, reason = analyze(run_dir)
        assert verdict == "NEEDS_HUMAN_GUI"
        assert "blocked-recipe-guard" in reason  # blocked reason, not resize


# ---------------------------------------------------------------------------
# validate_composite_run — window integrity failures
# ---------------------------------------------------------------------------

_VALID_SUMMARY: dict[str, Any] = {
    "schema": EXPECTED_SCHEMA,
    "prompt": "PROMPT-1644",
    "outcome": "ok",
    "recipe": "vs-bot",
    "soak_port": 5000,
    "rpc_port": 15873,
    "skip_soak_launch": False,
    "soak_duration_secs": 300,
    "smoke_exit_code": 0,
    "autoplay_artifact_dir": "",
    "evidence_dir": "",
    "dry_run": False,
    "generated_utc": "2026-05-28T09:00:00.000000+00:00",
    "live_pass_status": (
        "NOT-CLAIMED -- AUTOPLAY-VS-BOT-QA-001 requires human operator "
        "sign-off for live PASS evidence"
    ),
    "notes": "PROMPT 1850 test.",
}

_VS_BOT_CHECKPOINTS = [
    "lobby-loaded", "bot-added", "lobby-confirmed",
    "class-select-loaded",
    "placement-loaded", "placement-submitted",
]


def _make_composite_evidence(
    tmp_path: Path,
    sizes: list[tuple[int, int]] | None = None,
    log_lines: list[str] | None = None,
    summary_override: dict[str, Any] | None = None,
) -> tuple[Path, Path]:
    """Build a minimal composite evidence directory with artifact dir."""
    evidence_dir = tmp_path / "composite"
    evidence_dir.mkdir()
    artifact_dir = tmp_path / "autoplay-run"
    artifact_dir.mkdir()

    # launcher-status.json
    (artifact_dir / "launcher-status.json").write_text(
        json.dumps({"schema": "autoplay_launcher_status_v1", "outcome": "ok"}),
        encoding="utf-8",
    )

    # checkpoints.jsonl with all vs-bot required checkpoints
    rows = [
        {"tick": i + 1, "kind": "checkpoint", "label": lbl, "elapsed_secs": float(i)}
        for i, lbl in enumerate(_VS_BOT_CHECKPOINTS)
    ]
    (artifact_dir / "checkpoints.jsonl").write_text(
        "\n".join(json.dumps(r) for r in rows) + "\n", encoding="utf-8"
    )

    # driver-timeline.jsonl
    if sizes is not None:
        _write_timeline(artifact_dir, sizes)

    # driver.log
    if log_lines is not None:
        _write_driver_log(artifact_dir, log_lines)

    # composite-summary.json
    summary = dict(_VALID_SUMMARY)
    summary["autoplay_artifact_dir"] = str(artifact_dir)
    summary["evidence_dir"] = str(evidence_dir)
    if summary_override:
        summary.update(summary_override)
    (evidence_dir / SUMMARY_FILENAME).write_text(json.dumps(summary), encoding="utf-8")

    # autoplay-run-path.txt
    (evidence_dir / RUN_PATH_FILENAME).write_text(str(artifact_dir), encoding="utf-8")

    return evidence_dir, artifact_dir


class TestValidateWindowIntegrityGuard:
    def test_window_resize_detected_fails(self, tmp_path):
        # Arrange: 3 distinct sizes (2 resize events)
        evidence_dir, _ = _make_composite_evidence(
            tmp_path,
            sizes=[(1280, 720), (1280, 505), (1280, 1076)],
            log_lines=["tick=1 win32_printwindow=OK path=win32_tick_000001.png"],
        )
        # Act
        result = validate(evidence_dir)
        # Assert
        assert not result.ok
        assert any("WINDOW-RESIZE-DETECTED" in f for f in result.failures), (
            f"Expected WINDOW-RESIZE-DETECTED in failures; got: {result.failures}"
        )

    def test_window_height_too_small_fails(self, tmp_path):
        # Arrange: stable but below-threshold height (400px < MIN_WINDOW_HEIGHT)
        evidence_dir, _ = _make_composite_evidence(
            tmp_path,
            sizes=[(1280, 400)] * 5,
            log_lines=["tick=1 win32_printwindow=OK path=win32_tick_000001.png"],
        )
        result = validate(evidence_dir)
        assert not result.ok
        assert any("WINDOW-HEIGHT-TOO-SMALL" in f for f in result.failures), (
            f"Expected WINDOW-HEIGHT-TOO-SMALL; got: {result.failures}"
        )

    def test_win32_all_frozen_fails(self, tmp_path):
        # Arrange: all win32 PrintWindow captures frozen, good window size
        evidence_dir, _ = _make_composite_evidence(
            tmp_path,
            sizes=[(1280, 720)] * 5,
            log_lines=[
                "tick=1 win32_printwindow=FROZEN hash=0xdeadbeef",
                "tick=2 win32_printwindow=FROZEN hash=0xdeadbeef",
                "tick=3 win32_printwindow=FAILED path=win32_tick_000003.png",
            ],
        )
        result = validate(evidence_dir)
        assert not result.ok
        assert any("WIN32-ALL-FROZEN" in f for f in result.failures), (
            f"Expected WIN32-ALL-FROZEN; got: {result.failures}"
        )

    def test_clean_run_no_window_failures(self, tmp_path):
        # Arrange: stable window, good win32 captures — no window integrity failures
        evidence_dir, _ = _make_composite_evidence(
            tmp_path,
            sizes=[(1280, 720)] * 5,
            log_lines=[
                "tick=1 win32_printwindow=OK path=win32_tick_000001.png",
                "tick=2 win32_printwindow=OK path=win32_tick_000002.png",
            ],
        )
        result = validate(evidence_dir)
        window_failures = [
            f for f in result.failures
            if any(tag in f for tag in ("WINDOW-RESIZE", "WINDOW-HEIGHT", "WIN32-ALL-FROZEN"))
        ]
        assert not window_failures, (
            f"Clean run should have no window integrity failures; got: {window_failures}"
        )

    def test_missing_timeline_is_warning_not_failure(self, tmp_path):
        # Arrange: no driver-timeline.jsonl → should warn, not fail
        evidence_dir, _ = _make_composite_evidence(
            tmp_path,
            sizes=None,  # no timeline written
            log_lines=[
                "tick=1 win32_printwindow=OK path=win32_tick_000001.png",
            ],
        )
        result = validate(evidence_dir)
        window_failures = [
            f for f in result.failures
            if any(tag in f for tag in ("WINDOW-RESIZE", "WINDOW-HEIGHT", "WIN32-ALL-FROZEN"))
        ]
        assert not window_failures, (
            "Missing driver-timeline.jsonl should not cause window integrity failures"
        )
        assert any("driver-timeline.jsonl" in w for w in result.warnings)

    def test_missing_driver_log_is_warning_not_failure(self, tmp_path):
        # Arrange: timeline present (stable), no driver.log → should warn, not fail
        evidence_dir, _ = _make_composite_evidence(
            tmp_path,
            sizes=[(1280, 720)] * 3,
            log_lines=None,  # no driver.log written
        )
        result = validate(evidence_dir)
        assert not any("WIN32-ALL-FROZEN" in f for f in result.failures), (
            "Missing driver.log should not trigger WIN32-ALL-FROZEN failure"
        )
        assert any("driver.log" in w for w in result.warnings)

    def test_height_at_threshold_does_not_fail(self, tmp_path):
        # Arrange: height exactly at MIN_WINDOW_HEIGHT — boundary value, should not fail
        evidence_dir, _ = _make_composite_evidence(
            tmp_path,
            sizes=[(1280, VCR_MIN_WINDOW_HEIGHT)] * 3,
            log_lines=["tick=1 win32_printwindow=OK path=win32_tick_000001.png"],
        )
        result = validate(evidence_dir)
        height_failures = [f for f in result.failures if "WINDOW-HEIGHT-TOO-SMALL" in f]
        assert not height_failures, (
            f"Height exactly at threshold should not fail; got: {height_failures}"
        )

    def test_height_one_below_threshold_fails(self, tmp_path):
        # Arrange: height = MIN_WINDOW_HEIGHT - 1, no resize
        evidence_dir, _ = _make_composite_evidence(
            tmp_path,
            sizes=[(1280, VCR_MIN_WINDOW_HEIGHT - 1)] * 3,
            log_lines=["tick=1 win32_printwindow=OK path=win32_tick_000001.png"],
        )
        result = validate(evidence_dir)
        assert any("WINDOW-HEIGHT-TOO-SMALL" in f for f in result.failures), (
            f"Height one below threshold should fail; failures: {result.failures}"
        )

    def test_win32_mixed_frozen_ok_is_not_all_frozen(self, tmp_path):
        # Arrange: some OK, some FROZEN — PARTIAL_FROZEN, not ALL_FROZEN
        evidence_dir, _ = _make_composite_evidence(
            tmp_path,
            sizes=[(1280, 720)] * 5,
            log_lines=[
                "tick=1 win32_printwindow=OK path=win32_tick_000001.png",
                "tick=2 win32_printwindow=FROZEN hash=0xabc",
            ],
        )
        result = validate(evidence_dir)
        assert not any("WIN32-ALL-FROZEN" in f for f in result.failures), (
            "Mixed OK/FROZEN should not trigger WIN32-ALL-FROZEN failure"
        )


# ---------------------------------------------------------------------------
# Constants sanity
# ---------------------------------------------------------------------------

class TestConstants:
    def test_min_window_height_matches_between_modules(self):
        assert MIN_WINDOW_HEIGHT == VCR_MIN_WINDOW_HEIGHT, (
            "MIN_WINDOW_HEIGHT must be the same in analyze_evidence_run and validate_composite_run"
        )

    def test_min_window_height_is_below_expected_resolution(self):
        # Expected game resolution is 1280x720; threshold should be below that
        assert MIN_WINDOW_HEIGHT < 720, (
            "MIN_WINDOW_HEIGHT should be below the expected 720px game resolution"
        )
