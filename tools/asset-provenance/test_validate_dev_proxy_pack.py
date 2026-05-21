#!/usr/bin/env python3
"""Self-contained tests for the dev-proxy pack validator.

Run from the repo root:

    python -m unittest tools/asset-provenance/test_validate_dev_proxy_pack.py
"""

from __future__ import annotations

import json
import subprocess
import sys
import unittest
from pathlib import Path


TOOL_DIR = Path(__file__).resolve().parent
CHECK_SCRIPT = TOOL_DIR / "validate_dev_proxy_pack.py"
FIXTURE_DIR = TOOL_DIR / "fixtures"

sys.path.insert(0, str(TOOL_DIR))
import validate_dev_proxy_pack  # noqa: E402


def _entry(**overrides):
    base = {
        "logical_id": "lid_card_frame_common",
        "source_path": "D:/_GAMES/Ankama/Krosmaga/_extracted/_extracted/ui/card_frame_common.png",
        "match_quality": "good",
        "dev_only": True,
        "source_class": "licensed_krosmaga_dev_proxy",
        "release_class": "dev_only",
        "workflow_status": "needed",
        "license_provenance_warning": "Dev-only Krosmaga proxy; not release-approved.",
        "expected_consumer_surface": "hand.card_frame",
    }
    base.update(overrides)
    return base


def _manifest(*entries, **pack_overrides):
    pack = {
        "pack_id": "krosmaga-proxy-v1",
        "dev_only": True,
        "source_class": "licensed_krosmaga_dev_proxy",
        "release_class": "dev_only",
    }
    pack.update(pack_overrides)
    return {"pack": pack, "entries": list(entries)}


class DevProxyPackUnitTests(unittest.TestCase):
    def test_clean_manifest_passes(self):
        failures = validate_dev_proxy_pack.validate_manifest(_manifest(_entry()))
        self.assertEqual(failures, [])

    def test_release_safe_claim_fails(self):
        failures = validate_dev_proxy_pack.validate_manifest(
            _manifest(
                _entry(
                    dev_only=False,
                    source_class="studio_original",
                    release_class="release_allowed",
                    workflow_status="approved",
                )
            )
        )
        rules = {failure["rule"] for failure in failures}
        self.assertIn("dev_only_required", rules)
        self.assertIn("source_class_must_be_krosmaga_proxy", rules)
        self.assertIn("release_class_must_be_dev_only", rules)
        self.assertIn("workflow_status_must_remain_needed", rules)

    def test_source_path_under_repo_assets_fails(self):
        failures = validate_dev_proxy_pack.validate_manifest(
            _manifest(_entry(source_path="assets/art/ui/card/copied_krosmaga.png"))
        )
        self.assert_rule(failures, "source_path_must_not_be_repo_assets")

    def test_source_path_under_absolute_repo_assets_fails(self):
        failures = validate_dev_proxy_pack.validate_manifest(
            _manifest(
                _entry(
                    source_path="D:/_DEV/Work/Claude-Code-Game-Studios/assets/art/ui/copied.png"
                )
            )
        )
        self.assert_rule(failures, "source_path_must_not_be_repo_assets")

    def test_ambiguous_requires_manual_review(self):
        failures = validate_dev_proxy_pack.validate_manifest(
            _manifest(
                _entry(
                    match_quality="ambiguous",
                    manual_review_required=False,
                    ambiguity_notes="",
                )
            )
        )
        self.assert_rule(failures, "ambiguous_requires_manual_review")
        self.assert_rule(failures, "ambiguity_notes_required")

    def test_ambiguous_with_review_passes(self):
        failures = validate_dev_proxy_pack.validate_manifest(
            _manifest(
                _entry(
                    match_quality="ambiguous",
                    manual_review_required=True,
                    ambiguity_notes="Multiple candidate frame crops need art direction review.",
                )
            )
        )
        self.assertEqual(failures, [])

    def test_missing_requires_handling_and_no_source_path(self):
        failures = validate_dev_proxy_pack.validate_manifest(
            _manifest(
                _entry(
                    match_quality="missing",
                    source_path="D:/_GAMES/Ankama/Krosmaga/_extracted/no_match.png",
                )
            )
        )
        self.assert_rule(failures, "missing_rows_must_not_claim_source")
        self.assert_rule(failures, "missing_handling_required")

    def test_missing_with_handling_passes(self):
        failures = validate_dev_proxy_pack.validate_manifest(
            _manifest(
                _entry(
                    source_path=None,
                    match_quality="missing",
                    missing_handling="Leave unresolved until original CCGS art is produced.",
                )
            )
        )
        self.assertEqual(failures, [])

    def test_logical_id_prefix_required(self):
        failures = validate_dev_proxy_pack.validate_manifest(
            _manifest(_entry(logical_id="card_frame_common"))
        )
        self.assert_rule(failures, "logical_id_prefix_required")

    def test_logical_id_body_charset(self):
        failures = validate_dev_proxy_pack.validate_manifest(
            _manifest(_entry(logical_id="lid_Card_Frame"))
        )
        self.assert_rule(failures, "logical_id_body_charset")

    def test_consumer_surface_format(self):
        failures = validate_dev_proxy_pack.validate_manifest(
            _manifest(_entry(expected_consumer_surface="HandCardFrame"))
        )
        self.assert_rule(failures, "consumer_surface_format")

    def test_consumer_surface_trailing_dot(self):
        failures = validate_dev_proxy_pack.validate_manifest(
            _manifest(_entry(expected_consumer_surface="hand."))
        )
        self.assert_rule(failures, "consumer_surface_format")

    def test_needs_conversion_requires_notes(self):
        failures = validate_dev_proxy_pack.validate_manifest(
            _manifest(_entry(match_quality="needs_conversion"))
        )
        self.assert_rule(failures, "conversion_notes_required")

    def test_needs_conversion_with_notes_passes(self):
        failures = validate_dev_proxy_pack.validate_manifest(
            _manifest(
                _entry(
                    match_quality="needs_conversion",
                    conversion_notes="Resize 64x64 -> 32x32 and pack into hand atlas.",
                )
            )
        )
        self.assertEqual(failures, [])

    def test_pack_workflow_status_must_remain_needed(self):
        failures = validate_dev_proxy_pack.validate_manifest(
            _manifest(_entry(), workflow_status="approved")
        )
        self.assert_rule(failures, "pack_workflow_status_must_remain_needed")

    def test_pack_workflow_status_needed_passes(self):
        failures = validate_dev_proxy_pack.validate_manifest(
            _manifest(_entry(), workflow_status="needed")
        )
        self.assertEqual(failures, [])

    def test_pack_id_required(self):
        manifest = _manifest(_entry())
        manifest["pack"]["pack_id"] = ""
        failures = validate_dev_proxy_pack.validate_manifest(manifest)
        self.assert_rule(failures, "pack_id_required")

    def test_duplicate_logical_id_fails(self):
        failures = validate_dev_proxy_pack.validate_manifest(
            _manifest(_entry(), _entry(source_path="D:/_GAMES/Ankama/Krosmaga/other.png"))
        )
        self.assert_rule(failures, "duplicate_logical_id")

    def test_missing_required_key_raises(self):
        row = _entry()
        del row["match_quality"]
        with self.assertRaises(ValueError):
            validate_dev_proxy_pack.validate_manifest(_manifest(row))

    # --- Stage 3 readiness: atlas_binding -----------------------------------

    def test_atlas_binding_optional_no_failure_when_absent(self):
        failures = validate_dev_proxy_pack.validate_manifest(_manifest(_entry()))
        rules = {failure["rule"] for failure in failures}
        self.assertNotIn("atlas_binding_shape", rules)

    def test_atlas_binding_full_shape_passes(self):
        failures = validate_dev_proxy_pack.validate_manifest(
            _manifest(
                _entry(
                    atlas_binding={
                        "atlas_id": "hand_card_frames_v1",
                        "frame_index": 0,
                        "frame_size_px": [64, 96],
                        "frame_origin_px": [0, 0],
                    }
                )
            )
        )
        self.assertEqual(failures, [])

    def test_atlas_binding_shape_must_be_dict(self):
        failures = validate_dev_proxy_pack.validate_manifest(
            _manifest(_entry(atlas_binding=["hand_card_frames_v1", 0]))
        )
        self.assert_rule(failures, "atlas_binding_shape")

    def test_atlas_binding_atlas_id_required(self):
        failures = validate_dev_proxy_pack.validate_manifest(
            _manifest(
                _entry(
                    atlas_binding={
                        "atlas_id": "",
                        "frame_index": 0,
                        "frame_size_px": [64, 96],
                    }
                )
            )
        )
        self.assert_rule(failures, "atlas_binding_atlas_id_required")

    def test_atlas_binding_atlas_id_format(self):
        failures = validate_dev_proxy_pack.validate_manifest(
            _manifest(
                _entry(
                    atlas_binding={
                        "atlas_id": "HandCardFrames-V1",
                        "frame_index": 0,
                        "frame_size_px": [64, 96],
                    }
                )
            )
        )
        self.assert_rule(failures, "atlas_binding_atlas_id_format")

    def test_atlas_binding_frame_index_non_negative(self):
        failures = validate_dev_proxy_pack.validate_manifest(
            _manifest(
                _entry(
                    atlas_binding={
                        "atlas_id": "hand_card_frames_v1",
                        "frame_index": -1,
                        "frame_size_px": [64, 96],
                    }
                )
            )
        )
        self.assert_rule(failures, "atlas_binding_frame_index_non_negative")

    def test_atlas_binding_frame_index_must_be_int_not_bool(self):
        failures = validate_dev_proxy_pack.validate_manifest(
            _manifest(
                _entry(
                    atlas_binding={
                        "atlas_id": "hand_card_frames_v1",
                        "frame_index": True,
                        "frame_size_px": [64, 96],
                    }
                )
            )
        )
        self.assert_rule(failures, "atlas_binding_frame_index_non_negative")

    def test_atlas_binding_frame_size_must_be_positive_pair(self):
        failures = validate_dev_proxy_pack.validate_manifest(
            _manifest(
                _entry(
                    atlas_binding={
                        "atlas_id": "hand_card_frames_v1",
                        "frame_index": 0,
                        "frame_size_px": [0, 96],
                    }
                )
            )
        )
        self.assert_rule(failures, "atlas_binding_frame_size_px_shape")

    def test_atlas_binding_frame_size_wrong_length(self):
        failures = validate_dev_proxy_pack.validate_manifest(
            _manifest(
                _entry(
                    atlas_binding={
                        "atlas_id": "hand_card_frames_v1",
                        "frame_index": 0,
                        "frame_size_px": [64, 96, 8],
                    }
                )
            )
        )
        self.assert_rule(failures, "atlas_binding_frame_size_px_shape")

    def test_atlas_binding_frame_origin_optional_pair(self):
        failures = validate_dev_proxy_pack.validate_manifest(
            _manifest(
                _entry(
                    atlas_binding={
                        "atlas_id": "hand_card_frames_v1",
                        "frame_index": 0,
                        "frame_size_px": [64, 96],
                        "frame_origin_px": [-1, 0],
                    }
                )
            )
        )
        self.assert_rule(failures, "atlas_binding_frame_origin_px_shape")

    def test_atlas_binding_forbidden_on_missing(self):
        failures = validate_dev_proxy_pack.validate_manifest(
            _manifest(
                _entry(
                    source_path=None,
                    match_quality="missing",
                    missing_handling="Leave unresolved until original CCGS art is produced.",
                    atlas_binding={
                        "atlas_id": "hand_card_frames_v1",
                        "frame_index": 0,
                        "frame_size_px": [64, 96],
                    },
                )
            )
        )
        self.assert_rule(failures, "atlas_binding_forbidden_for_missing_or_no_art")

    def test_atlas_binding_forbidden_on_ambiguous(self):
        failures = validate_dev_proxy_pack.validate_manifest(
            _manifest(
                _entry(
                    match_quality="ambiguous",
                    manual_review_required=True,
                    ambiguity_notes="Two candidate crops; needs art-lead disambiguation.",
                    atlas_binding={
                        "atlas_id": "hand_card_frames_v1",
                        "frame_index": 0,
                        "frame_size_px": [64, 96],
                    },
                )
            )
        )
        self.assert_rule(failures, "atlas_binding_forbidden_for_ambiguous")

    # --- Stage 3 readiness: pack.sprite_sheets registry ---------------------

    def test_sprite_sheets_optional_when_absent(self):
        failures = validate_dev_proxy_pack.validate_manifest(_manifest(_entry()))
        rules = {failure["rule"] for failure in failures}
        self.assertNotIn("pack_sprite_sheets_shape", rules)

    def test_sprite_sheets_registry_passes_and_cross_references(self):
        manifest = _manifest(
            _entry(
                atlas_binding={
                    "atlas_id": "hand_card_frames_v1",
                    "frame_index": 0,
                    "frame_size_px": [64, 96],
                }
            ),
            sprite_sheets=[
                {
                    "sheet_id": "hand_card_frames_v1",
                    "dimensions_px": [256, 192],
                    "frame_count": 4,
                }
            ],
        )
        failures = validate_dev_proxy_pack.validate_manifest(manifest)
        self.assertEqual(failures, [])

    def test_sprite_sheets_atlas_id_unknown(self):
        manifest = _manifest(
            _entry(
                atlas_binding={
                    "atlas_id": "missing_sheet",
                    "frame_index": 0,
                    "frame_size_px": [64, 96],
                }
            ),
            sprite_sheets=[
                {
                    "sheet_id": "hand_card_frames_v1",
                    "dimensions_px": [256, 192],
                    "frame_count": 4,
                }
            ],
        )
        failures = validate_dev_proxy_pack.validate_manifest(manifest)
        self.assert_rule(failures, "atlas_binding_atlas_id_unknown")

    def test_sprite_sheets_shape_must_be_list(self):
        failures = validate_dev_proxy_pack.validate_manifest(
            _manifest(_entry(), sprite_sheets={"hand_card_frames_v1": {}})
        )
        self.assert_rule(failures, "pack_sprite_sheets_shape")

    def test_sprite_sheets_entry_must_be_object(self):
        failures = validate_dev_proxy_pack.validate_manifest(
            _manifest(_entry(), sprite_sheets=["hand_card_frames_v1"])
        )
        self.assert_rule(failures, "pack_sprite_sheet_entry_shape")

    def test_sprite_sheets_sheet_id_required(self):
        failures = validate_dev_proxy_pack.validate_manifest(
            _manifest(
                _entry(),
                sprite_sheets=[
                    {"sheet_id": "", "dimensions_px": [256, 192], "frame_count": 4}
                ],
            )
        )
        self.assert_rule(failures, "pack_sprite_sheet_sheet_id_required")

    def test_sprite_sheets_sheet_id_format(self):
        failures = validate_dev_proxy_pack.validate_manifest(
            _manifest(
                _entry(),
                sprite_sheets=[
                    {
                        "sheet_id": "Hand-Card-Frames",
                        "dimensions_px": [256, 192],
                        "frame_count": 4,
                    }
                ],
            )
        )
        self.assert_rule(failures, "pack_sprite_sheet_sheet_id_format")

    def test_sprite_sheets_dimensions_must_be_positive_pair(self):
        failures = validate_dev_proxy_pack.validate_manifest(
            _manifest(
                _entry(),
                sprite_sheets=[
                    {
                        "sheet_id": "hand_card_frames_v1",
                        "dimensions_px": [256, 0],
                        "frame_count": 4,
                    }
                ],
            )
        )
        self.assert_rule(failures, "pack_sprite_sheet_dimensions_px_shape")

    def test_sprite_sheets_frame_count_positive(self):
        failures = validate_dev_proxy_pack.validate_manifest(
            _manifest(
                _entry(),
                sprite_sheets=[
                    {
                        "sheet_id": "hand_card_frames_v1",
                        "dimensions_px": [256, 192],
                        "frame_count": 0,
                    }
                ],
            )
        )
        self.assert_rule(failures, "pack_sprite_sheet_frame_count_positive")

    def test_sprite_sheets_sheet_id_duplicate(self):
        failures = validate_dev_proxy_pack.validate_manifest(
            _manifest(
                _entry(),
                sprite_sheets=[
                    {
                        "sheet_id": "hand_card_frames_v1",
                        "dimensions_px": [256, 192],
                        "frame_count": 4,
                    },
                    {
                        "sheet_id": "hand_card_frames_v1",
                        "dimensions_px": [128, 96],
                        "frame_count": 2,
                    },
                ],
            )
        )
        self.assert_rule(failures, "pack_sprite_sheet_sheet_id_duplicate")

    # --- Stage 3 readiness: pack.license_provenance block -------------------

    def test_license_provenance_optional_when_absent(self):
        failures = validate_dev_proxy_pack.validate_manifest(_manifest(_entry()))
        rules = {failure["rule"] for failure in failures}
        self.assertNotIn("pack_license_provenance_shape", rules)

    def test_license_provenance_full_block_passes(self):
        failures = validate_dev_proxy_pack.validate_manifest(
            _manifest(
                _entry(),
                license_provenance={
                    "holder": "Ankama Games",
                    "kind": "licensed_krosmaga_dev_proxy",
                    "dev_only_statement": "Dev-only Krosmaga reference; never release-approved.",
                },
            )
        )
        self.assertEqual(failures, [])

    def test_license_provenance_shape_must_be_object(self):
        failures = validate_dev_proxy_pack.validate_manifest(
            _manifest(_entry(), license_provenance="Ankama Games — dev-only")
        )
        self.assert_rule(failures, "pack_license_provenance_shape")

    def test_license_provenance_holder_required(self):
        failures = validate_dev_proxy_pack.validate_manifest(
            _manifest(
                _entry(),
                license_provenance={
                    "holder": "",
                    "kind": "licensed_krosmaga_dev_proxy",
                    "dev_only_statement": "Dev-only Krosmaga reference; never release-approved.",
                },
            )
        )
        self.assert_rule(failures, "pack_license_provenance_holder_required")

    def test_license_provenance_kind_value(self):
        failures = validate_dev_proxy_pack.validate_manifest(
            _manifest(
                _entry(),
                license_provenance={
                    "holder": "Ankama Games",
                    "kind": "studio_original",
                    "dev_only_statement": "Dev-only Krosmaga reference; never release-approved.",
                },
            )
        )
        self.assert_rule(failures, "pack_license_provenance_kind_value")

    def test_license_provenance_kind_must_match_source_class(self):
        failures = validate_dev_proxy_pack.validate_manifest(
            _manifest(
                _entry(),
                license_provenance={
                    "holder": "Some Font Foundry",
                    "kind": "licensed_external_release",
                    "dev_only_statement": "Dev-only proxy; not release-approved.",
                },
            )
        )
        self.assert_rule(
            failures, "pack_license_provenance_kind_must_match_source_class"
        )

    def test_license_provenance_dev_only_statement_required(self):
        failures = validate_dev_proxy_pack.validate_manifest(
            _manifest(
                _entry(),
                license_provenance={
                    "holder": "Ankama Games",
                    "kind": "licensed_krosmaga_dev_proxy",
                    "dev_only_statement": "",
                },
            )
        )
        self.assert_rule(
            failures, "pack_license_provenance_dev_only_statement_required"
        )

    def test_license_provenance_dev_only_statement_must_block_release_claim(self):
        failures = validate_dev_proxy_pack.validate_manifest(
            _manifest(
                _entry(),
                license_provenance={
                    "holder": "Ankama Games",
                    "kind": "licensed_krosmaga_dev_proxy",
                    "dev_only_statement": "Reference material owned by Ankama.",
                },
            )
        )
        self.assert_rule(
            failures,
            "pack_license_provenance_dev_only_statement_must_block_release_claim",
        )

    # --- Stage 3 readiness: stage_readiness opt-in marker -------------------

    def test_stage_readiness_optional_when_absent(self):
        failures = validate_dev_proxy_pack.validate_manifest(_manifest(_entry()))
        rules = {failure["rule"] for failure in failures}
        self.assertNotIn("stage_readiness_value", rules)

    def test_stage_readiness_value_must_be_known(self):
        failures = validate_dev_proxy_pack.validate_manifest(
            _manifest(_entry(stage_readiness="ready_for_release"))
        )
        self.assert_rule(failures, "stage_readiness_value")

    def test_stage3_binding_requires_atlas_binding(self):
        failures = validate_dev_proxy_pack.validate_manifest(
            _manifest(_entry(stage_readiness="stage3_binding"))
        )
        self.assert_rule(failures, "stage3_binding_requires_atlas_binding")

    def test_stage3_binding_requires_concrete_match_quality(self):
        failures = validate_dev_proxy_pack.validate_manifest(
            _manifest(
                _entry(
                    match_quality="ambiguous",
                    manual_review_required=True,
                    ambiguity_notes="Two candidate crops; needs art-lead disambiguation.",
                    stage_readiness="stage3_binding",
                )
            )
        )
        self.assert_rule(failures, "stage3_binding_requires_concrete_match")

    def test_stage3_binding_full_row_passes(self):
        failures = validate_dev_proxy_pack.validate_manifest(
            _manifest(
                _entry(
                    match_quality="needs_conversion",
                    conversion_notes="Resize Krosmaga frame to CCGS hand frame dimensions.",
                    atlas_binding={
                        "atlas_id": "hand_card_frames_v1",
                        "frame_index": 0,
                        "frame_size_px": [64, 96],
                        "frame_origin_px": [0, 0],
                    },
                    stage_readiness="stage3_binding",
                ),
                sprite_sheets=[
                    {
                        "sheet_id": "hand_card_frames_v1",
                        "dimensions_px": [256, 192],
                        "frame_count": 4,
                    }
                ],
                license_provenance={
                    "holder": "Ankama Games",
                    "kind": "licensed_krosmaga_dev_proxy",
                    "dev_only_statement": "Dev-only Krosmaga reference; never release-approved.",
                },
            )
        )
        self.assertEqual(failures, [])

    def assert_rule(self, failures, rule):
        rules = {failure["rule"] for failure in failures}
        self.assertIn(rule, rules, f"expected {rule} in {sorted(rules)}")


class DevProxyPackCliTests(unittest.TestCase):
    def test_clean_fixture_exits_zero(self):
        result = self._run(FIXTURE_DIR / "dev-proxy-pack-clean.json")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stderr.strip(), "")

    def test_release_claim_fixture_exits_one(self):
        result = self._run(FIXTURE_DIR / "dev-proxy-pack-release-claim.json")
        self.assertEqual(result.returncode, 1, result.stderr)
        payload = json.loads(result.stderr)
        rules = {entry["rule"] for entry in payload}
        self.assertIn("dev_only_required", rules)
        self.assertIn("release_class_must_be_dev_only", rules)

    def test_repo_assets_fixture_exits_one(self):
        result = self._run(FIXTURE_DIR / "dev-proxy-pack-repo-assets-source.json")
        self.assertEqual(result.returncode, 1, result.stderr)
        payload = json.loads(result.stderr)
        rules = {entry["rule"] for entry in payload}
        self.assertIn("source_path_must_not_be_repo_assets", rules)

    def test_stage2_candidate_fixture_exits_zero(self):
        result = self._run(FIXTURE_DIR / "dev-proxy-pack-stage2-candidate.json")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stderr.strip(), "")

    def test_bad_logical_id_fixture_exits_one(self):
        result = self._run(FIXTURE_DIR / "dev-proxy-pack-bad-logical-id.json")
        self.assertEqual(result.returncode, 1, result.stderr)
        payload = json.loads(result.stderr)
        rules = {entry["rule"] for entry in payload}
        self.assertTrue(
            "logical_id_prefix_required" in rules
            or "logical_id_body_charset" in rules,
            f"expected logical-id prefix/charset failure in {sorted(rules)}",
        )

    def test_stage3_candidate_fixture_exits_zero(self):
        result = self._run(FIXTURE_DIR / "dev-proxy-pack-stage3-candidate.json")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stderr.strip(), "")

    def test_atlas_binding_bad_fixture_exits_one(self):
        result = self._run(FIXTURE_DIR / "dev-proxy-pack-atlas-binding-bad.json")
        self.assertEqual(result.returncode, 1, result.stderr)
        payload = json.loads(result.stderr)
        rules = {entry["rule"] for entry in payload}
        self.assertIn("stage3_binding_requires_atlas_binding", rules)
        self.assertIn("atlas_binding_atlas_id_unknown", rules)

    def test_missing_manifest_exits_two(self):
        result = self._run(FIXTURE_DIR / "does-not-exist.json")
        self.assertEqual(result.returncode, 2)

    def _run(self, manifest_path):
        return subprocess.run(
            [sys.executable, str(CHECK_SCRIPT), str(manifest_path)],
            capture_output=True,
            text=True,
        )


if __name__ == "__main__":
    unittest.main()
