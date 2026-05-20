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
