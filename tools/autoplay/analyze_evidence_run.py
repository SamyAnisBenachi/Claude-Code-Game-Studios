#!/usr/bin/env python3
"""Analyzer for a single autoplay-run evidence directory (PROMPT 1833).

Summarizes an autoplay run without mutating any evidence files:
  - Launcher outcome and driver exit code (from launcher-status.json)
  - Capture label families seen in driver.log
    (win32_capture, win32_printwindow, desktop_bitblt, FROZEN)
  - Screenshot counts: root win32_tick_*.png vs screenshots/ Bevy dir
  - Distinct pixel_hash count and values (from driver.log)
  - Likely verdict: PASS / PARTIAL / FAIL / NEEDS_HUMAN_GUI

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
SCREENSHOTS_SUBDIR = "screenshots"

# Regex patterns for driver.log lines
_RE_PIXEL_HASH = re.compile(r"pixel_hash=(0x[0-9a-fA-F]+)")
_RE_CAPTURE_LABEL = re.compile(
    r"\b(win32_printwindow|win32_capture|desktop_bitblt)\b"
)
_RE_FROZEN_LABEL = re.compile(r"\bFROZEN\b")

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

        # pixel_hash tracking
        self.pixel_hashes: list[str] = []          # in order of appearance

        # Screenshot counts
        self.root_win32_png_count: int = 0         # win32_tick_*.png at run root
        self.bevy_screenshot_count: int = 0        # screenshots/*.png

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


# ---------------------------------------------------------------------------
# Verdict logic
# ---------------------------------------------------------------------------

def _compute_verdict(summary: EvidenceSummary) -> tuple[str, str]:
    """Return (verdict, reason) where verdict is one of PASS/PARTIAL/FAIL/NEEDS_HUMAN_GUI."""

    outcome = (summary.launcher_outcome or "").lower()

    # NEEDS_HUMAN_GUI: run was blocked before the game GUI was reached
    if outcome in _BLOCKED_HUMAN_GUI_OUTCOMES:
        return "NEEDS_HUMAN_GUI", f"launcher outcome={outcome!r} — run never reached game GUI"

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
        "--- Capture Labels ---",
        f"  families seen    : {sorted(summary.capture_labels) or '(none)'}",
        f"  FROZEN lines     : {summary.frozen_labels_count}",
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
        "capture": {
            "label_families": sorted(summary.capture_labels),
            "frozen_log_lines": summary.frozen_labels_count,
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
