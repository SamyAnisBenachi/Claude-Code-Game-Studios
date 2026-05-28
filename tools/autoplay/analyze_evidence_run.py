#!/usr/bin/env python3
"""Analyzer for a single autoplay-run evidence directory (PROMPT 1833).

Summarizes an autoplay run without mutating any evidence files:
  - Launcher outcome and driver exit code (from launcher-status.json)
  - Capture label families seen in driver.log
    (win32_capture, win32_printwindow, desktop_bitblt, FROZEN)
  - Screenshot counts: root win32_tick_*.png vs screenshots/ Bevy dir
  - Distinct pixel_hash count and values (from driver.log)
  - Window size tracking from driver-timeline.jsonl:
    initial_window_size, final_window_size, window_resize_event_count,
    min_window_height_seen
  - Win32 capture quality: GOOD / PARTIAL_FROZEN / ALL_FROZEN / UNKNOWN
  - Likely verdict: PASS / PARTIAL / FAIL / NEEDS_HUMAN_GUI

NEEDS_HUMAN_GUI is emitted when:
  - Mid-run window resize was detected (window_resize_event_count > 0)
  - Window height dropped below MIN_WINDOW_HEIGHT during the run
  - All win32 PrintWindow captures were frozen or failed (ALL_FROZEN quality)

Usage:
    python tools/autoplay/analyze_evidence_run.py <run-dir>
    python tools/autoplay/analyze_evidence_run.py <run-dir> --json

Exit codes:
  0  PASS
  1  PARTIAL
  2  FAIL
  3  NEEDS_HUMAN_GUI
  4  Cannot analyse (missing or unreadable evidence)
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Optional

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

LAUNCHER_STATUS_FILE = "launcher-status.json"
DRIVER_LOG_FILE = "driver.log"
DRIVER_TIMELINE_FILE = "driver-timeline.jsonl"
SCREENSHOTS_SUBDIR = "screenshots"

# Minimum window height (px) below which a run is flagged for human review.
# The expected game resolution is 1280x720; a height below 600 indicates the
# window was resized/obscured mid-run in a way that invalidates capture evidence.
MIN_WINDOW_HEIGHT: int = 600

# Regex patterns for driver.log lines
_RE_PIXEL_HASH = re.compile(r"pixel_hash=(0x[0-9a-fA-F]+)")
_RE_CAPTURE_LABEL = re.compile(
    r"\b(win32_printwindow|win32_capture|desktop_bitblt)\b"
)
_RE_FROZEN_LABEL = re.compile(r"\bFROZEN\b")
# win32 PrintWindow result lines emitted by driver.py
_RE_WIN32_OK = re.compile(r"\bwin32_printwindow=OK\b")
_RE_WIN32_FAILED_OR_FROZEN = re.compile(r"\bwin32_printwindow=(?:FAILED|FROZEN)\b")

# Blocked-run outcomes that mean the game GUI was never reached
_BLOCKED_HUMAN_GUI_OUTCOMES = {
    "blocked-human-gui",
    "blocked-precondition",
    "blocked-recipe-guard",
    "blocked-soak-timeout",
}

# Verdict exit codes
_EXIT_PASS = 0
_EXIT_PARTIAL = 1
_EXIT_FAIL = 2
_EXIT_NEEDS_HUMAN_GUI = 3
_EXIT_UNREADABLE = 4


# ---------------------------------------------------------------------------
# Data classes
# ---------------------------------------------------------------------------

class EvidenceSummary:
    def __init__(self) -> None:
        # Launcher / driver
        self.launcher_outcome: Optional[str] = None
        self.driver_exit_code: Optional[int] = None
        self.client_exit_code: Optional[int] = None

        # Capture label families observed in driver.log
        self.capture_labels: set[str] = set()      # e.g. {"win32_capture"}
        self.frozen_labels_count: int = 0

        # Win32 PrintWindow capture quality (PROMPT 1850)
        self.win32_ok_count: int = 0               # win32_printwindow=OK lines
        self.win32_failed_or_frozen_count: int = 0  # win32_printwindow=FAILED/FROZEN lines

        # pixel_hash tracking
        self.pixel_hashes: list[str] = []          # in order of appearance

        # Screenshot counts
        self.root_win32_png_count: int = 0         # win32_tick_*.png at run root
        self.bevy_screenshot_count: int = 0        # screenshots/*.png

        # Window size tracking from driver-timeline.jsonl (PROMPT 1850)
        self.initial_window_size: Optional[tuple[int, int]] = None
        self.final_window_size: Optional[tuple[int, int]] = None
        self.window_resize_event_count: int = 0
        self.min_window_height_seen: Optional[int] = None

        # Parse errors / warnings
        self.warnings: list[str] = []

    # Derived properties

    @property
    def distinct_pixel_hashes(self) -> list[str]:
        seen: list[str] = []
        s: set[str] = set()
        for h in self.pixel_hashes:
            if h not in s:
                seen.append(h)
                s.add(h)
        return seen

    @property
    def total_screenshots(self) -> int:
        return self.root_win32_png_count + self.bevy_screenshot_count

    @property
    def is_frozen(self) -> bool:
        """True if all pixel_hashes are the same (frozen renderer)."""
        return len(self.pixel_hashes) >= 2 and len(set(self.pixel_hashes)) == 1

    @property
    def win32_capture_quality(self) -> str:
        """GOOD / PARTIAL_FROZEN / ALL_FROZEN / UNKNOWN based on driver.log counts."""
        total = self.win32_ok_count + self.win32_failed_or_frozen_count
        if total == 0:
            return "UNKNOWN"
        if self.win32_failed_or_frozen_count == 0:
            return "GOOD"
        if self.win32_ok_count == 0:
            return "ALL_FROZEN"
        return "PARTIAL_FROZEN"


# ---------------------------------------------------------------------------
# Parsing helpers
# ---------------------------------------------------------------------------

def _parse_launcher_status(run_dir: Path, summary: EvidenceSummary) -> None:
    path = run_dir / LAUNCHER_STATUS_FILE
    if not path.exists():
        summary.warnings.append(f"MISSING: {LAUNCHER_STATUS_FILE}")
        return
    try:
        data = json.loads(path.read_text(encoding="utf-8-sig"))
    except Exception as exc:
        summary.warnings.append(f"UNREADABLE {LAUNCHER_STATUS_FILE}: {exc}")
        return

    summary.launcher_outcome = data.get("outcome")
    raw_exit = data.get("driver_exit_code")
    summary.driver_exit_code = int(raw_exit) if raw_exit is not None else None
    raw_client = data.get("client_exit_code")
    summary.client_exit_code = int(raw_client) if raw_client is not None else None


def _parse_driver_log(run_dir: Path, summary: EvidenceSummary) -> None:
    path = run_dir / DRIVER_LOG_FILE
    if not path.exists():
        summary.warnings.append(f"MISSING: {DRIVER_LOG_FILE}")
        return
    try:
        text = path.read_text(encoding="utf-8", errors="replace")
    except Exception as exc:
        summary.warnings.append(f"UNREADABLE {DRIVER_LOG_FILE}: {exc}")
        return

    for line in text.splitlines():
        # Capture label families
        for m in _RE_CAPTURE_LABEL.finditer(line):
            summary.capture_labels.add(m.group(1))

        # FROZEN label
        if _RE_FROZEN_LABEL.search(line):
            summary.frozen_labels_count += 1

        # Win32 PrintWindow quality tracking
        if _RE_WIN32_OK.search(line):
            summary.win32_ok_count += 1
        elif _RE_WIN32_FAILED_OR_FROZEN.search(line):
            summary.win32_failed_or_frozen_count += 1

        # pixel_hash values
        for m in _RE_PIXEL_HASH.finditer(line):
            summary.pixel_hashes.append(m.group(1).lower())


def _count_screenshots(run_dir: Path, summary: EvidenceSummary) -> None:
    # Root-level win32 captures
    summary.root_win32_png_count = len(list(run_dir.glob("win32_tick_*.png")))

    # Bevy screenshots subdirectory
    scr_dir = run_dir / SCREENSHOTS_SUBDIR
    if scr_dir.is_dir():
        summary.bevy_screenshot_count = len(list(scr_dir.glob("*.png")))


def _parse_driver_timeline(run_dir: Path, summary: EvidenceSummary) -> None:
    """Extract window size history from driver-timeline.jsonl (PROMPT 1850).

    Populates initial_window_size, final_window_size, window_resize_event_count,
    and min_window_height_seen on *summary*.  A "resize event" is any tick where
    window_logical_size differs from the previous tick's value.
    """
    path = run_dir / DRIVER_TIMELINE_FILE
    if not path.exists():
        # Not all runs produce a timeline; warn but do not fail.
        summary.warnings.append(f"MISSING: {DRIVER_TIMELINE_FILE} — window size tracking unavailable")
        return
    try:
        text = path.read_text(encoding="utf-8", errors="replace")
    except Exception as exc:
        summary.warnings.append(f"UNREADABLE {DRIVER_TIMELINE_FILE}: {exc}")
        return

    last_size: Optional[tuple[int, int]] = None
    parse_errors = 0
    for line in text.splitlines():
        line = line.strip()
        if not line:
            continue
        try:
            row = json.loads(line)
        except json.JSONDecodeError:
            parse_errors += 1
            continue
        status = row.get("status") or {}
        raw_size = status.get("window_logical_size")
        if not isinstance(raw_size, (list, tuple)) or len(raw_size) != 2:
            continue
        try:
            w, h = int(raw_size[0]), int(raw_size[1])
        except (ValueError, TypeError):
            continue
        size = (w, h)
        if summary.initial_window_size is None:
            summary.initial_window_size = size
        if last_size is not None and size != last_size:
            summary.window_resize_event_count += 1
        last_size = size
        if summary.min_window_height_seen is None or h < summary.min_window_height_seen:
            summary.min_window_height_seen = h

    summary.final_window_size = last_size

    if parse_errors:
        summary.warnings.append(
            f"{parse_errors} line(s) in {DRIVER_TIMELINE_FILE} could not be parsed as JSON."
        )


# ---------------------------------------------------------------------------
# Verdict logic
# ---------------------------------------------------------------------------

def _compute_verdict(summary: EvidenceSummary) -> tuple[str, str]:
    """Return (verdict, reason) where verdict is one of PASS/PARTIAL/FAIL/NEEDS_HUMAN_GUI."""

    outcome = (summary.launcher_outcome or "").lower()

    # NEEDS_HUMAN_GUI: run was blocked before the game GUI was reached
    if outcome in _BLOCKED_HUMAN_GUI_OUTCOMES:
        return "NEEDS_HUMAN_GUI", f"launcher outcome={outcome!r} — run never reached game GUI"

    # NEEDS_HUMAN_GUI: window resize / capture quality issues (PROMPT 1850)
    # These conditions mean captured evidence cannot be trusted as clean PASS.
    needs_human_reasons: list[str] = []

    if summary.window_resize_event_count > 0:
        needs_human_reasons.append(
            f"mid-run window resize detected: {summary.window_resize_event_count} resize event(s); "
            f"initial={summary.initial_window_size}, final={summary.final_window_size}, "
            f"min_height_seen={summary.min_window_height_seen}px"
        )

    if (
        summary.min_window_height_seen is not None
        and summary.min_window_height_seen < MIN_WINDOW_HEIGHT
        and summary.window_resize_event_count == 0
    ):
        # Standalone below-minimum height (e.g. window launched too small) without a resize event.
        needs_human_reasons.append(
            f"window height below minimum throughout run: "
            f"min_height_seen={summary.min_window_height_seen}px < threshold={MIN_WINDOW_HEIGHT}px"
        )

    if summary.win32_capture_quality == "ALL_FROZEN":
        needs_human_reasons.append(
            f"all win32 PrintWindow captures were frozen/failed "
            f"(ok={summary.win32_ok_count}, failed_or_frozen={summary.win32_failed_or_frozen_count})"
            f" — capture evidence unreliable; win32_capture_quality=ALL_FROZEN"
        )

    if needs_human_reasons:
        return "NEEDS_HUMAN_GUI", "; ".join(needs_human_reasons)

    # FAIL conditions (hard)
    fail_reasons: list[str] = []

    if summary.driver_exit_code not in (None, 0):
        fail_reasons.append(f"driver_exit_code={summary.driver_exit_code}")

    if summary.total_screenshots == 0:
        fail_reasons.append("no screenshots captured (0 root win32 + 0 bevy)")

    if summary.pixel_hashes and len(set(summary.pixel_hashes)) == 0:
        fail_reasons.append("pixel_hash list is empty despite log entries")

    if fail_reasons:
        return "FAIL", "; ".join(fail_reasons)

    # PARTIAL conditions
    partial_reasons: list[str] = []

    if summary.is_frozen:
        partial_reasons.append(
            f"all {len(summary.pixel_hashes)} pixel_hash captures share "
            f"the same value ({summary.distinct_pixel_hashes[0]}) — renderer may be frozen"
        )

    if summary.frozen_labels_count > 0:
        partial_reasons.append(
            f"FROZEN label appeared {summary.frozen_labels_count} time(s) in driver.log"
        )

    if summary.total_screenshots > 0 and not summary.capture_labels:
        partial_reasons.append("screenshots present but no recognised capture label in driver.log")

    if partial_reasons:
        return "PARTIAL", "; ".join(partial_reasons)

    # PASS
    distinct = len(summary.distinct_pixel_hashes)
    total_ss = summary.total_screenshots
    labels = sorted(summary.capture_labels)
    return (
        "PASS",
        f"driver_exit_code={summary.driver_exit_code}, "
        f"{total_ss} screenshot(s), "
        f"{distinct} distinct pixel_hash(es), "
        f"labels={labels}",
    )


# ---------------------------------------------------------------------------
# Formatting
# ---------------------------------------------------------------------------

def _format_human(run_dir: Path, summary: EvidenceSummary, verdict: str, reason: str) -> str:
    lines = [
        f"=== analyze_evidence_run: {run_dir.name} ===",
        "",
        "--- Launcher / Driver ---",
        f"  launcher_outcome : {summary.launcher_outcome!r}",
        f"  driver_exit_code : {summary.driver_exit_code}",
        f"  client_exit_code : {summary.client_exit_code}",
        "",
        "--- Window Size (driver-timeline.jsonl) ---",
        f"  initial_window_size    : {summary.initial_window_size}",
        f"  final_window_size      : {summary.final_window_size}",
        f"  window_resize_events   : {summary.window_resize_event_count}",
        f"  min_window_height_seen : {summary.min_window_height_seen}px"
        f"  (threshold={MIN_WINDOW_HEIGHT}px)",
        "",
        "--- Capture Labels ---",
        f"  families seen    : {sorted(summary.capture_labels) or '(none)'}",
        f"  FROZEN lines     : {summary.frozen_labels_count}",
        f"  win32_ok_count   : {summary.win32_ok_count}",
        f"  win32_fail/frozen: {summary.win32_failed_or_frozen_count}",
        f"  win32_quality    : {summary.win32_capture_quality}",
        "",
        "--- Screenshots ---",
        f"  root win32_tick  : {summary.root_win32_png_count} PNG(s)",
        f"  bevy screenshots : {summary.bevy_screenshot_count} PNG(s)",
        f"  total            : {summary.total_screenshots}",
        "",
        "--- pixel_hash ---",
        f"  total captures   : {len(summary.pixel_hashes)}",
        f"  distinct count   : {len(summary.distinct_pixel_hashes)}",
        f"  distinct values  : {summary.distinct_pixel_hashes or '(none)'}",
        f"  frozen pattern   : {summary.is_frozen}",
    ]

    if summary.warnings:
        lines += ["", "--- Warnings ---"]
        for w in summary.warnings:
            lines.append(f"  WARN: {w}")

    lines += [
        "",
        f"VERDICT: {verdict}",
        f"REASON : {reason}",
        "",
    ]
    return "\n".join(lines)


def _format_json(run_dir: Path, summary: EvidenceSummary, verdict: str, reason: str) -> str:
    obj = {
        "run_dir": str(run_dir),
        "launcher": {
            "outcome": summary.launcher_outcome,
            "driver_exit_code": summary.driver_exit_code,
            "client_exit_code": summary.client_exit_code,
        },
        "window": {
            "initial_window_size": list(summary.initial_window_size) if summary.initial_window_size else None,
            "final_window_size": list(summary.final_window_size) if summary.final_window_size else None,
            "window_resize_event_count": summary.window_resize_event_count,
            "min_window_height_seen": summary.min_window_height_seen,
            "min_window_height_threshold": MIN_WINDOW_HEIGHT,
        },
        "capture": {
            "label_families": sorted(summary.capture_labels),
            "frozen_log_lines": summary.frozen_labels_count,
            "win32_ok_count": summary.win32_ok_count,
            "win32_failed_or_frozen_count": summary.win32_failed_or_frozen_count,
            "win32_capture_quality": summary.win32_capture_quality,
        },
        "screenshots": {
            "root_win32_png_count": summary.root_win32_png_count,
            "bevy_screenshot_count": summary.bevy_screenshot_count,
            "total": summary.total_screenshots,
        },
        "pixel_hash": {
            "total_captures": len(summary.pixel_hashes),
            "distinct_count": len(summary.distinct_pixel_hashes),
            "distinct_values": summary.distinct_pixel_hashes,
            "frozen_pattern": summary.is_frozen,
        },
        "warnings": summary.warnings,
        "verdict": verdict,
        "reason": reason,
    }
    return json.dumps(obj, indent=2)


# ---------------------------------------------------------------------------
# Public API
# ---------------------------------------------------------------------------

def analyze(run_dir: Path) -> tuple[EvidenceSummary, str, str]:
    """Analyse *run_dir*; return (summary, verdict, reason)."""
    summary = EvidenceSummary()

    if not run_dir.exists() or not run_dir.is_dir():
        summary.warnings.append(f"RUN DIR NOT FOUND OR NOT A DIRECTORY: {run_dir}")
        return summary, "FAIL", f"run dir not found: {run_dir}"

    _parse_launcher_status(run_dir, summary)
    _parse_driver_log(run_dir, summary)
    _parse_driver_timeline(run_dir, summary)
    _count_screenshots(run_dir, summary)

    verdict, reason = _compute_verdict(summary)
    return summary, verdict, reason


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        prog="analyze_evidence_run",
        description=(
            "Analyse a single autoplay-run evidence directory and emit a "
            "PASS/PARTIAL/FAIL/NEEDS_HUMAN_GUI verdict."
        ),
    )
    parser.add_argument(
        "run_dir",
        metavar="RUN_DIR",
        help="Path to the autoplay-run evidence directory (e.g. production/qa/evidence/autoplay-runs/20260528-Z)",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        default=False,
        dest="json_out",
        help="Emit machine-readable JSON instead of the human summary.",
    )
    args = parser.parse_args(argv)

    run_dir = Path(args.run_dir)
    summary, verdict, reason = analyze(run_dir)

    if args.json_out:
        out = _format_json(run_dir, summary, verdict, reason)
    else:
        out = _format_human(run_dir, summary, verdict, reason)

    try:
        print(out)
    except UnicodeEncodeError:
        print(out.encode(sys.stdout.encoding or "ascii", errors="replace").decode(
            sys.stdout.encoding or "ascii"
        ))

    exit_codes = {
        "PASS": _EXIT_PASS,
        "PARTIAL": _EXIT_PARTIAL,
        "FAIL": _EXIT_FAIL,
        "NEEDS_HUMAN_GUI": _EXIT_NEEDS_HUMAN_GUI,
    }
    return exit_codes.get(verdict, _EXIT_UNREADABLE)


if __name__ == "__main__":
    sys.exit(main())
