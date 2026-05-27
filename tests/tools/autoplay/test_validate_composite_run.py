"""Unit tests for tools/autoplay/validate_composite_run.py (PROMPT 1651).

Tests use synthetic in-memory fixture directories (tmp_path) so they run
without a running game client, Cargo, or network access.

Run with:
    pytest tests/tools/autoplay/test_validate_composite_run.py -v
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

from validate_composite_run import (
    EXPECTED_SCHEMA,
    RECIPE_REQUIRED_CHECKPOINTS,
    SUMMARY_FILENAME,
    RUN_PATH_FILENAME,
    validate,
)


# ---------------------------------------------------------------------------
# Fixture helpers
# ---------------------------------------------------------------------------

_VALID_SUMMARY: dict[str, Any] = {
    "schema": EXPECTED_SCHEMA,
    "prompt": "PROMPT-1644",
    "outcome": "ok",
    "recipe": "full-game",
    "soak_port": 5000,
    "rpc_port": 15873,
    "skip_soak_launch": False,
    "soak_duration_secs": 300,
    "smoke_exit_code": 0,
    "autoplay_artifact_dir": "",  # filled per-test
    "evidence_dir": "",           # filled per-test
    "dry_run": False,
    "generated_utc": "2026-05-27T12:00:00.000000+00:00",
    "live_pass_status": (
        "NOT-CLAIMED -- AUTOPLAY-VS-BOT-QA-001 requires human operator "
        "sign-off for live PASS evidence"
    ),
    "notes": "PROMPT 1644 composite harness v1.",
}


def _make_evidence_dir(
    tmp_path: Path,
    *,
    summary_override: dict[str, Any] | None = None,
    include_run_path: bool = True,
    run_path_content: str | None = None,
    include_artifact_dir: bool = True,
    artifact_checkpoints: list[dict] | None = None,
) -> tuple[Path, Path]:
    """Build a minimal synthetic evidence directory.

    Returns (evidence_dir, artifact_dir).
    """
    evidence_dir = tmp_path / "2026-05-27-120000-autoplay-vs-bot"
    evidence_dir.mkdir()

    artifact_dir = tmp_path / "autoplay-runs" / "20260527-120001-Z"
    if include_artifact_dir:
        artifact_dir.mkdir(parents=True)
        # launcher-status.json (minimal)
        (artifact_dir / "launcher-status.json").write_text(
            json.dumps({"schema": "autoplay_launcher_status_v1", "outcome": "ok"}),
            encoding="utf-8",
        )
        if artifact_checkpoints is not None:
            lines = "\n".join(json.dumps(row) for row in artifact_checkpoints)
            (artifact_dir / "checkpoints.jsonl").write_text(lines + "\n", encoding="utf-8")

    summary = dict(_VALID_SUMMARY)
    summary["autoplay_artifact_dir"] = str(artifact_dir)
    summary["evidence_dir"] = str(evidence_dir)
    if summary_override:
        summary.update(summary_override)

    (evidence_dir / SUMMARY_FILENAME).write_text(
        json.dumps(summary), encoding="utf-8"
    )

    if include_run_path:
        content = run_path_content if run_path_content is not None else str(artifact_dir)
        (evidence_dir / RUN_PATH_FILENAME).write_text(content, encoding="utf-8")

    return evidence_dir, artifact_dir


def _checkpoint_rows(*labels: str) -> list[dict]:
    return [
        {"tick": i + 1, "kind": "checkpoint", "label": lbl, "elapsed_secs": float(i)}
        for i, lbl in enumerate(labels)
    ]


# ---------------------------------------------------------------------------
# Happy path
# ---------------------------------------------------------------------------

class TestValidHappyPath:
    def test_validate_composite_run_valid_summary_passes(self, tmp_path):
        # Arrange: minimal valid evidence dir with full-game required checkpoints
        checkpoints = _checkpoint_rows(
            "lobby-loaded", "lobby-confirmed",
            "class-select-loaded",
            "placement-loaded", "placement-submitted",
            "full-game-post-resolution",
        )
        evidence_dir, _ = _make_evidence_dir(
            tmp_path, artifact_checkpoints=checkpoints
        )
        # Act
        result = validate(evidence_dir)
        # Assert
        assert result.ok, f"Expected PASS but got failures: {result.failures}"

    def test_validate_composite_run_no_artifact_dir_warns_not_fails(self, tmp_path):
        # Arrange: artifact dir absent (dry-run scenario)
        evidence_dir, artifact_dir = _make_evidence_dir(
            tmp_path, include_artifact_dir=False
        )
        # Act
        result = validate(evidence_dir, strict=False)
        # Assert: warnings allowed, no failures
        assert result.ok, f"Expected PASS (non-strict) but got failures: {result.failures}"
        assert any("ARTIFACT DIR NOT FOUND" in w for w in result.warnings)

    def test_validate_composite_run_blocked_outcome_skips_checkpoints(self, tmp_path):
        # Arrange: blocked run — no checkpoints expected
        evidence_dir, artifact_dir = _make_evidence_dir(
            tmp_path,
            summary_override={"outcome": "blocked-recipe-guard", "smoke_exit_code": 4},
            artifact_checkpoints=[],  # empty checkpoints.jsonl
        )
        # Act
        result = validate(evidence_dir)
        # Assert
        assert result.ok, f"Blocked run should pass checkpoint check: {result.failures}"

    def test_validate_composite_run_recipe_override_respected(self, tmp_path):
        # Arrange: summary says full-game but we override to smoke (no checkpoints needed)
        evidence_dir, _ = _make_evidence_dir(
            tmp_path,
            artifact_checkpoints=[],  # no checkpoints emitted
        )
        # Act: override to smoke — no checkpoint requirement
        result = validate(evidence_dir, recipe_override="smoke")
        # Assert
        assert result.ok, f"smoke requires no checkpoints: {result.failures}"

    def test_validate_composite_run_no_checkpoints_file_is_warning_not_fail(self, tmp_path):
        # Arrange: artifact dir present but checkpoints.jsonl absent
        evidence_dir, artifact_dir = _make_evidence_dir(
            tmp_path, include_artifact_dir=True, artifact_checkpoints=None
        )
        # Act
        result = validate(evidence_dir)
        # Assert
        assert result.ok
        assert any("checkpoints.jsonl not found" in w for w in result.warnings)


# ---------------------------------------------------------------------------
# Schema / field checks
# ---------------------------------------------------------------------------

class TestSummarySchemaChecks:
    def test_validate_composite_run_wrong_schema_fails(self, tmp_path):
        evidence_dir, _ = _make_evidence_dir(
            tmp_path, summary_override={"schema": "wrong_schema_v99"}
        )
        result = validate(evidence_dir)
        assert not result.ok
        assert any("SCHEMA MISMATCH" in f for f in result.failures)

    def test_validate_composite_run_missing_schema_field_fails(self, tmp_path):
        evidence_dir, _ = _make_evidence_dir(
            tmp_path, summary_override={"schema": None}
        )
        result = validate(evidence_dir)
        assert not result.ok
        assert any("SCHEMA MISMATCH" in f for f in result.failures)

    def test_validate_composite_run_empty_outcome_fails(self, tmp_path):
        evidence_dir, _ = _make_evidence_dir(
            tmp_path, summary_override={"outcome": ""}
        )
        result = validate(evidence_dir)
        assert not result.ok
        assert any("INVALID OUTCOME" in f for f in result.failures)

    def test_validate_composite_run_missing_outcome_fails(self, tmp_path):
        summary_no_outcome = {k: v for k, v in _VALID_SUMMARY.items() if k != "outcome"}
        evidence_dir, _ = _make_evidence_dir(
            tmp_path, summary_override=summary_no_outcome
        )
        result = validate(evidence_dir)
        assert not result.ok

    def test_validate_composite_run_missing_summary_file_exits_early(self, tmp_path):
        evidence_dir = tmp_path / "empty-dir"
        evidence_dir.mkdir()
        result = validate(evidence_dir)
        assert not result.ok
        assert any("MISSING" in f and SUMMARY_FILENAME in f for f in result.failures)

    def test_validate_composite_run_invalid_json_fails(self, tmp_path):
        evidence_dir = tmp_path / "bad-json"
        evidence_dir.mkdir()
        (evidence_dir / SUMMARY_FILENAME).write_text("not { valid json }", encoding="utf-8")
        result = validate(evidence_dir)
        assert not result.ok
        assert any("INVALID JSON" in f for f in result.failures)

    def test_validate_composite_run_nonexistent_evidence_dir_fails(self, tmp_path):
        missing = tmp_path / "does-not-exist"
        result = validate(missing)
        assert not result.ok
        assert any("NOT FOUND" in f for f in result.failures)


# ---------------------------------------------------------------------------
# Live-pass-status check
# ---------------------------------------------------------------------------

class TestLivePassStatusCheck:
    def test_validate_composite_run_live_pass_claim_fails(self, tmp_path):
        evidence_dir, _ = _make_evidence_dir(
            tmp_path,
            summary_override={"live_pass_status": "PASS -- operator signed off"},
        )
        result = validate(evidence_dir)
        assert not result.ok
        assert any("LIVE PASS CLAIM" in f for f in result.failures)

    def test_validate_composite_run_empty_live_pass_status_fails(self, tmp_path):
        evidence_dir, _ = _make_evidence_dir(
            tmp_path, summary_override={"live_pass_status": ""}
        )
        result = validate(evidence_dir)
        assert not result.ok
        assert any("LIVE PASS CLAIM" in f for f in result.failures)

    def test_validate_composite_run_not_claimed_text_passes(self, tmp_path):
        evidence_dir, _ = _make_evidence_dir(
            tmp_path,
            summary_override={
                "live_pass_status": "NOT-CLAIMED -- some custom text here"
            },
            artifact_checkpoints=_checkpoint_rows(
                "lobby-loaded", "lobby-confirmed",
                "class-select-loaded",
                "placement-loaded", "placement-submitted",
            ),
        )
        result = validate(evidence_dir)
        assert result.ok


# ---------------------------------------------------------------------------
# autoplay-run-path.txt checks
# ---------------------------------------------------------------------------

class TestRunPathFileChecks:
    def test_validate_composite_run_missing_run_path_file_fails(self, tmp_path):
        evidence_dir, _ = _make_evidence_dir(tmp_path, include_run_path=False)
        result = validate(evidence_dir)
        assert not result.ok
        assert any(RUN_PATH_FILENAME in f for f in result.failures)

    def test_validate_composite_run_path_mismatch_fails(self, tmp_path):
        evidence_dir, artifact_dir = _make_evidence_dir(
            tmp_path,
            run_path_content="/some/other/path/entirely",
        )
        result = validate(evidence_dir)
        assert not result.ok
        assert any("PATH MISMATCH" in f for f in result.failures)

    def test_validate_composite_run_path_matches_passes(self, tmp_path):
        evidence_dir, artifact_dir = _make_evidence_dir(
            tmp_path,
            artifact_checkpoints=_checkpoint_rows(
                "lobby-loaded", "lobby-confirmed",
                "class-select-loaded",
                "placement-loaded", "placement-submitted",
            ),
        )
        result = validate(evidence_dir)
        assert result.ok


# ---------------------------------------------------------------------------
# Artifact directory / launcher-status checks
# ---------------------------------------------------------------------------

class TestArtifactDirChecks:
    def test_validate_composite_run_missing_launcher_status_fails(self, tmp_path):
        evidence_dir, artifact_dir = _make_evidence_dir(
            tmp_path, include_artifact_dir=True
        )
        # Remove the launcher-status.json that the helper created
        (artifact_dir / "launcher-status.json").unlink()
        result = validate(evidence_dir)
        assert not result.ok
        assert any("launcher-status.json" in f for f in result.failures)

    def test_validate_composite_run_strict_missing_artifact_dir_fails(self, tmp_path):
        evidence_dir, _ = _make_evidence_dir(tmp_path, include_artifact_dir=False)
        result = validate(evidence_dir, strict=True)
        assert not result.ok
        assert any("ARTIFACT DIR NOT FOUND" in f for f in result.failures)


# ---------------------------------------------------------------------------
# Checkpoint validation
# ---------------------------------------------------------------------------

class TestCheckpointChecks:
    def test_validate_composite_run_missing_checkpoint_fails(self, tmp_path):
        # Arrange: full-game run but lobby-confirmed checkpoint absent
        checkpoints = _checkpoint_rows(
            "lobby-loaded",
            # "lobby-confirmed" intentionally omitted
            "class-select-loaded",
            "placement-loaded", "placement-submitted",
        )
        evidence_dir, _ = _make_evidence_dir(tmp_path, artifact_checkpoints=checkpoints)
        result = validate(evidence_dir)
        assert not result.ok
        assert any("MISSING CHECKPOINTS" in f for f in result.failures)
        assert any("lobby-confirmed" in f for f in result.failures)

    def test_validate_composite_run_all_required_checkpoints_present_passes(self, tmp_path):
        checkpoints = _checkpoint_rows(
            "lobby-loaded", "lobby-confirmed",
            "class-select-loaded", "class-confirmed",
            "placement-loaded", "placement-dragged", "placement-submitted",
            "full-game-post-resolution",
        )
        evidence_dir, _ = _make_evidence_dir(tmp_path, artifact_checkpoints=checkpoints)
        result = validate(evidence_dir)
        assert result.ok, f"Expected PASS: {result.failures}"

    def test_validate_composite_run_extra_checkpoints_do_not_fail(self, tmp_path):
        # Extra checkpoints beyond required must not cause failure
        checkpoints = _checkpoint_rows(
            "lobby-loaded", "lobby-confirmed",
            "class-select-loaded", "class-confirmed",
            "placement-loaded", "placement-dragged", "placement-submitted",
            "extra-checkpoint-a", "extra-checkpoint-b",
            "full-game-complete",
        )
        evidence_dir, _ = _make_evidence_dir(tmp_path, artifact_checkpoints=checkpoints)
        result = validate(evidence_dir)
        assert result.ok

    def test_validate_composite_run_lobby_create_required_checkpoints(self, tmp_path):
        # Arrange: recipe is lobby-create with its checkpoints
        checkpoints = _checkpoint_rows("lobby-loaded", "lobby-confirmed")
        evidence_dir, _ = _make_evidence_dir(
            tmp_path,
            summary_override={"recipe": "lobby-create"},
            artifact_checkpoints=checkpoints,
        )
        result = validate(evidence_dir)
        assert result.ok

    def test_validate_composite_run_lobby_create_missing_checkpoint_fails(self, tmp_path):
        checkpoints = _checkpoint_rows("lobby-loaded")  # lobby-confirmed missing
        evidence_dir, _ = _make_evidence_dir(
            tmp_path,
            summary_override={"recipe": "lobby-create"},
            artifact_checkpoints=checkpoints,
        )
        result = validate(evidence_dir)
        assert not result.ok
        assert any("lobby-confirmed" in f for f in result.failures)

    def test_validate_composite_run_smoke_recipe_no_checkpoints_required(self, tmp_path):
        # smoke recipe: zero checkpoints in file → still passes
        evidence_dir, _ = _make_evidence_dir(
            tmp_path,
            summary_override={"recipe": "smoke"},
            artifact_checkpoints=[],  # empty
        )
        result = validate(evidence_dir)
        assert result.ok

    def test_validate_composite_run_bad_jsonl_line_warns_not_fails(self, tmp_path):
        evidence_dir, artifact_dir = _make_evidence_dir(
            tmp_path,
            artifact_checkpoints=None,
        )
        # Write a checkpoints.jsonl with one bad line and all required rows
        rows = _checkpoint_rows(
            "lobby-loaded", "lobby-confirmed",
            "class-select-loaded",
            "placement-loaded", "placement-submitted",
        )
        lines = ["not-valid-json"] + [json.dumps(r) for r in rows]
        (artifact_dir / "checkpoints.jsonl").write_text(
            "\n".join(lines) + "\n", encoding="utf-8"
        )
        result = validate(evidence_dir)
        # Should warn about parse error but pass because required checkpoints are present
        assert result.ok
        assert any("could not be parsed" in w for w in result.warnings)


# ---------------------------------------------------------------------------
# RECIPE_REQUIRED_CHECKPOINTS registry completeness
# ---------------------------------------------------------------------------

class TestCheckpointRegistry:
    KNOWN_RECIPES = {
        "smoke", "idle", "lobby-create", "add-bot-lobby",
        "class-select", "draft-auction-probe", "placement-drag-probe",
        "resolution-observe", "game-over-observe", "full-game", "round-loop",
    }

    def test_validate_composite_run_all_known_recipes_in_registry(self):
        missing = self.KNOWN_RECIPES - RECIPE_REQUIRED_CHECKPOINTS.keys()
        assert not missing, (
            f"Recipes missing from RECIPE_REQUIRED_CHECKPOINTS: {missing}"
        )

    def test_validate_composite_run_registry_values_are_lists_of_strings(self):
        for recipe, labels in RECIPE_REQUIRED_CHECKPOINTS.items():
            assert isinstance(labels, list), (
                f"Recipe '{recipe}': expected list, got {type(labels)}"
            )
            for lbl in labels:
                assert isinstance(lbl, str) and lbl, (
                    f"Recipe '{recipe}': label {lbl!r} is not a non-empty string"
                )
