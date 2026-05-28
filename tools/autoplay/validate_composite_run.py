#!/usr/bin/env python3
"""Validator for autoplay-vs-bot composite evidence directories (PROMPT 1651).

Validates the structure and content produced by
``tools/dev-launcher/Start-AutoplayVsBot.ps1`` (PROMPT 1644) without
mutating any evidence file.

Checks performed:
  1. Evidence directory exists.
  2. ``composite-summary.json`` present and is valid JSON.
  3. ``schema`` field == ``"autoplay_vs_bot_composite_summary_v1"``.
  4. ``outcome`` field present and non-empty string.
  5. ``live_pass_status`` present and contains ``NOT-CLAIMED`` (does not
     falsely assert a live PASS for AUTOPLAY-VS-BOT-QA-001).
  6. ``autoplay-run-path.txt`` present.
  7. Path in ``autoplay-run-path.txt`` matches ``autoplay_artifact_dir``
     from the summary (when both are non-empty).
  8. If the autoplay artifact directory exists on disk:
     a. ``launcher-status.json`` is present inside it.
     b. If ``checkpoints.jsonl`` is present, the expected checkpoint labels
        for the recipe are a subset of the observed labels.
  When ``--strict`` is set, a missing artifact directory is an error rather
  than a warning.

Exit codes:
  0  all checks passed
  1  one or more checks failed (details printed to stdout)
  2  evidence directory not found or composite-summary.json missing/unparseable

Pure Python 3 stdlib only. No Cargo, no GUI.

Usage:
    python tools/autoplay/validate_composite_run.py <evidence-dir>
    python tools/autoplay/validate_composite_run.py <evidence-dir> [--recipe NAME] [--strict]
    python tools/autoplay/validate_composite_run.py --help
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Optional

# ---------------------------------------------------------------------------
# Schema constants
# ---------------------------------------------------------------------------

EXPECTED_SCHEMA = "autoplay_vs_bot_composite_summary_v1"
SUMMARY_FILENAME = "composite-summary.json"
RUN_PATH_FILENAME = "autoplay-run-path.txt"

REQUIRED_SUMMARY_FIELDS = [
    "schema",
    "outcome",
    "recipe",
    "live_pass_status",
    "autoplay_artifact_dir",
]

# Minimal checkpoint labels that MUST appear in checkpoints.jsonl (kind="checkpoint")
# for a successful (non-blocked) run of each recipe.  Recipes with no required
# checkpoints (smoke, idle) map to an empty list — the check is a no-op.
RECIPE_REQUIRED_CHECKPOINTS: dict[str, list[str]] = {
    "smoke":               [],
    "idle":                [],
    "lobby-create":        ["lobby-loaded", "lobby-confirmed"],
    "add-bot-lobby":       ["lobby-loaded", "bot-added", "lobby-confirmed"],
    "class-select":        ["class-select-loaded", "class-confirmed"],
    "draft-auction-probe": ["shop-loaded", "auction-ready"],
    "placement-drag-probe":["placement-loaded", "placement-submitted"],
    "resolution-observe":  ["resolution-started", "resolution-complete"],
    "game-over-observe":   ["game-over-screen", "winner-confirmed"],
    # full-game ends with one of three terminal checkpoints depending on env flags;
    # require only the lobby and placement bookends that are always present.
    "full-game":           ["lobby-loaded", "lobby-confirmed",
                            "class-select-loaded",
                            "placement-loaded", "placement-submitted"],
    "round-loop":          ["round-loop-complete"],
    # vs-bot uses add-bot-lobby instead of lobby-create; require the same
    # phase bookends that are always present regardless of env flag overrides.
    "vs-bot":              ["lobby-loaded", "bot-added", "lobby-confirmed",
                            "class-select-loaded",
                            "placement-loaded", "placement-submitted"],
}

# Outcomes that indicate the run was blocked before the recipe ran; skip
# checkpoint validation for these because no checkpoints will have been emitted.
BLOCKED_OUTCOMES = {
    "blocked-recipe-guard",
    "blocked-human-gui",
    "blocked-precondition",
    "blocked-soak-timeout",
}


# ---------------------------------------------------------------------------
# Result helpers
# ---------------------------------------------------------------------------

class _Result:
    def __init__(self) -> None:
        self.failures: list[str] = []
        self.warnings: list[str] = []

    def fail(self, msg: str) -> None:
        self.failures.append(msg)

    def warn(self, msg: str) -> None:
        self.warnings.append(msg)

    @property
    def ok(self) -> bool:
        return not self.failures


# ---------------------------------------------------------------------------
# Core validation logic
# ---------------------------------------------------------------------------

def _load_summary(evidence_dir: Path, result: _Result) -> Optional[dict]:
    summary_path = evidence_dir / SUMMARY_FILENAME
    if not summary_path.exists():
        result.fail(f"MISSING: {SUMMARY_FILENAME} not found in {evidence_dir}")
        return None
    try:
        with summary_path.open(encoding="utf-8") as fh:
            return json.load(fh)
    except json.JSONDecodeError as exc:
        result.fail(f"INVALID JSON: {SUMMARY_FILENAME}: {exc}")
        return None


def _check_summary_schema(summary: dict, result: _Result) -> None:
    schema = summary.get("schema")
    if schema != EXPECTED_SCHEMA:
        result.fail(
            f"SCHEMA MISMATCH: expected '{EXPECTED_SCHEMA}', got '{schema}'"
        )


def _check_required_fields(summary: dict, result: _Result) -> None:
    for field in REQUIRED_SUMMARY_FIELDS:
        if field not in summary:
            result.fail(f"MISSING FIELD: '{field}' absent from {SUMMARY_FILENAME}")


def _check_outcome(summary: dict, result: _Result) -> None:
    outcome = summary.get("outcome")
    if not outcome or not isinstance(outcome, str):
        result.fail(
            f"INVALID OUTCOME: 'outcome' must be a non-empty string, got {outcome!r}"
        )


def _check_live_pass_status(summary: dict, result: _Result) -> None:
    lps = summary.get("live_pass_status", "")
    if not isinstance(lps, str) or "NOT-CLAIMED" not in lps:
        result.fail(
            "LIVE PASS CLAIM: 'live_pass_status' must contain 'NOT-CLAIMED'. "
            f"Found: {lps!r}. "
            "AUTOPLAY-VS-BOT-QA-001 live PASS requires human operator sign-off."
        )


def _check_run_path_file(
    evidence_dir: Path, summary: dict, result: _Result
) -> None:
    rp_path = evidence_dir / RUN_PATH_FILENAME
    if not rp_path.exists():
        result.fail(f"MISSING: {RUN_PATH_FILENAME} not found in {evidence_dir}")
        return

    rp_text = rp_path.read_text(encoding="utf-8").strip()
    summary_dir = (summary.get("autoplay_artifact_dir") or "").strip()
    if rp_text and summary_dir and Path(rp_text) != Path(summary_dir):
        result.fail(
            f"PATH MISMATCH: {RUN_PATH_FILENAME} contains:\n  {rp_text}\n"
            f"but autoplay_artifact_dir in summary is:\n  {summary_dir}"
        )


def _check_artifact_dir(
    summary: dict, result: _Result, strict: bool
) -> Optional[Path]:
    artifact_dir_str = (summary.get("autoplay_artifact_dir") or "").strip()
    if not artifact_dir_str:
        result.fail("MISSING VALUE: 'autoplay_artifact_dir' is empty in summary")
        return None

    artifact_dir = Path(artifact_dir_str)
    if not artifact_dir.exists():
        msg = (
            f"ARTIFACT DIR NOT FOUND: {artifact_dir} does not exist on disk. "
            "Run may be incomplete or on a different machine."
        )
        if strict:
            result.fail(msg)
        else:
            result.warn(msg)
        return None

    launcher_status = artifact_dir / "launcher-status.json"
    if not launcher_status.exists():
        result.fail(
            f"MISSING: launcher-status.json not found inside artifact dir {artifact_dir}"
        )

    return artifact_dir


def _check_checkpoints(
    artifact_dir: Path,
    recipe: str,
    outcome: str,
    result: _Result,
) -> None:
    if outcome in BLOCKED_OUTCOMES:
        result.warn(
            f"SKIPPING checkpoint check: outcome is '{outcome}' (blocked run); "
            "no recipe checkpoints expected."
        )
        return

    checkpoints_path = artifact_dir / "checkpoints.jsonl"
    if not checkpoints_path.exists():
        result.warn(
            f"checkpoints.jsonl not found in {artifact_dir}; "
            "skipping checkpoint label check."
        )
        return

    expected = RECIPE_REQUIRED_CHECKPOINTS.get(recipe)
    if expected is None:
        result.warn(
            f"No checkpoint expectations defined for recipe '{recipe}'; "
            "skipping checkpoint label check."
        )
        return

    if not expected:
        return  # recipe requires no specific checkpoints (smoke, idle)

    observed_labels: set[str] = set()
    parse_errors = 0
    with checkpoints_path.open(encoding="utf-8") as fh:
        for lineno, line in enumerate(fh, 1):
            line = line.strip()
            if not line:
                continue
            try:
                row = json.loads(line)
            except json.JSONDecodeError:
                parse_errors += 1
                continue
            if row.get("kind") == "checkpoint":
                label = row.get("label", "")
                if label:
                    observed_labels.add(label)

    if parse_errors:
        result.warn(
            f"{parse_errors} line(s) in checkpoints.jsonl could not be parsed as JSON."
        )

    missing = [lbl for lbl in expected if lbl not in observed_labels]
    if missing:
        result.fail(
            f"MISSING CHECKPOINTS for recipe '{recipe}': "
            + ", ".join(f"'{lbl}'" for lbl in missing)
            + f"\n  Observed labels: {sorted(observed_labels) or '(none)'}"
        )


# ---------------------------------------------------------------------------
# Top-level entry point
# ---------------------------------------------------------------------------

def validate(
    evidence_dir: Path,
    recipe_override: Optional[str] = None,
    strict: bool = False,
) -> _Result:
    """Validate a composite evidence directory; returns a _Result."""
    result = _Result()

    if not evidence_dir.exists():
        result.fail(f"EVIDENCE DIR NOT FOUND: {evidence_dir}")
        return result

    if not evidence_dir.is_dir():
        result.fail(f"NOT A DIRECTORY: {evidence_dir}")
        return result

    summary = _load_summary(evidence_dir, result)
    if summary is None:
        return result  # fatal — cannot continue without the summary

    _check_summary_schema(summary, result)
    _check_required_fields(summary, result)
    _check_outcome(summary, result)
    _check_live_pass_status(summary, result)
    _check_run_path_file(evidence_dir, summary, result)

    artifact_dir = _check_artifact_dir(summary, result, strict=strict)

    recipe = recipe_override or summary.get("recipe") or ""
    outcome = summary.get("outcome") or ""

    if artifact_dir is not None and recipe:
        _check_checkpoints(artifact_dir, recipe, outcome, result)

    return result


def _print_result(result: _Result, evidence_dir: Path) -> None:
    if result.warnings:
        print(f"[validate_composite_run] WARNINGS ({len(result.warnings)}):")
        for w in result.warnings:
            print(f"  WARN: {w}")

    if result.ok:
        print(f"[validate_composite_run] PASS: {evidence_dir}")
    else:
        print(f"[validate_composite_run] FAIL: {evidence_dir}")
        print(f"  {len(result.failures)} check(s) failed:")
        for f in result.failures:
            print(f"  FAIL: {f}")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        prog="validate_composite_run",
        description=(
            "Validate an autoplay-vs-bot composite evidence directory produced "
            "by Start-AutoplayVsBot.ps1 (PROMPT 1644)."
        ),
    )
    parser.add_argument(
        "evidence_dir",
        metavar="EVIDENCE_DIR",
        help=(
            "Path to the composite evidence directory "
            "(e.g. production/qa/evidence/composite-runs/2026-05-27-120000-autoplay-vs-bot)"
        ),
    )
    parser.add_argument(
        "--recipe",
        metavar="NAME",
        default=None,
        help=(
            "Override the recipe name for checkpoint validation "
            "(default: read from composite-summary.json)."
        ),
    )
    parser.add_argument(
        "--strict",
        action="store_true",
        default=False,
        help=(
            "Treat a missing autoplay artifact directory as a failure "
            "instead of a warning."
        ),
    )
    args = parser.parse_args(argv)

    evidence_dir = Path(args.evidence_dir)
    result = validate(evidence_dir, recipe_override=args.recipe, strict=args.strict)
    _print_result(result, evidence_dir)

    if not result.ok:
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
