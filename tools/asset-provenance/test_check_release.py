#!/usr/bin/env python3
"""Self-contained unit tests for the ADR-025 release-scan validator.

Run from the repo root:

    python -m unittest tools/asset-provenance/test_check_release.py

Or directly:

    python tools/asset-provenance/test_check_release.py

These tests cover the six failure rules plus the passing case, satisfying
Story 007 AC6.
"""

from __future__ import annotations

import json
import subprocess
import sys
import unittest
from pathlib import Path


TOOL_DIR = Path(__file__).resolve().parent
CHECK_SCRIPT = TOOL_DIR / "check_release.py"
FIXTURE_DIR = TOOL_DIR / "fixtures"

# Make `check_release` importable when running this file directly.
sys.path.insert(0, str(TOOL_DIR))
import check_release  # noqa: E402  (path-augmented import)


def _row(**overrides):
    base = {
        "logical_id": "lid_card_frame_common",
        "workflow_status": "approved",
        "source_class": "studio_original",
        "release_class": "release_allowed",
        "approval_evidence": "production/qa/sign-off-2026-05-20-card-frames.md",
        "path": "art/ui/card/ui_card_frame_common_hand.png",
    }
    base.update(overrides)
    return base


def _manifest(*rows):
    return {"logical_assets": list(rows)}


class CheckManifestUnitTests(unittest.TestCase):
    """Unit tests calling check_release.check_manifest() directly."""

    # PASS

    def test_clean_manifest_passes(self):
        failures = check_release.check_manifest(_manifest(_row()))
        self.assertEqual(failures, [])

    def test_empty_manifest_passes(self):
        failures = check_release.check_manifest(_manifest())
        self.assertEqual(failures, [])

    # Rule 1 — Krosmaga proxy source

    def test_rule_1_krosmaga_proxy_source_fails(self):
        row = _row(
            source_class="licensed_krosmaga_dev_proxy",
            release_class="dev_only",
            workflow_status="needed",
            approval_evidence=None,
        )
        failures = check_release.check_manifest(_manifest(row))
        self._assert_rule_triggered(failures, "rule_1_krosmaga_proxy_source")

    # Rule 2 — Unknown provenance

    def test_rule_2_unknown_provenance_fails(self):
        row = _row(source_class="unknown_provenance")
        failures = check_release.check_manifest(_manifest(row))
        self._assert_rule_triggered(failures, "rule_2_unknown_provenance")

    # Rule 3 — Release class blocks

    def test_rule_3_release_class_dev_only_fails(self):
        row = _row(release_class="dev_only")
        failures = check_release.check_manifest(_manifest(row))
        self._assert_rule_triggered(failures, "rule_3_release_class_blocks")

    def test_rule_3_release_class_internal_only_fails(self):
        row = _row(release_class="internal_only")
        failures = check_release.check_manifest(_manifest(row))
        self._assert_rule_triggered(failures, "rule_3_release_class_blocks")

    # Rule 4 — Workflow not approved

    def test_rule_4_workflow_not_approved_fails(self):
        row = _row(workflow_status="in_progress", approval_evidence=None)
        failures = check_release.check_manifest(_manifest(row))
        self._assert_rule_triggered(failures, "rule_4_workflow_not_approved")

    def test_rule_4_workflow_blocked_fails(self):
        row = _row(workflow_status="blocked", approval_evidence=None)
        failures = check_release.check_manifest(_manifest(row))
        self._assert_rule_triggered(failures, "rule_4_workflow_not_approved")

    # Rule 5 — Missing approval evidence with status=approved

    def test_rule_5_missing_approval_evidence_fails(self):
        row = _row(approval_evidence=None)
        failures = check_release.check_manifest(_manifest(row))
        self._assert_rule_triggered(failures, "rule_5_missing_approval_evidence")

    def test_rule_5_empty_string_approval_evidence_fails(self):
        row = _row(approval_evidence="")
        failures = check_release.check_manifest(_manifest(row))
        self._assert_rule_triggered(failures, "rule_5_missing_approval_evidence")

    # Rule 6 — Dev-assets path

    def test_rule_6_dev_assets_path_fails(self):
        row = _row(path="dev-assets/krosmaga-proxy/frames/card_frame_common.png")
        failures = check_release.check_manifest(_manifest(row))
        self._assert_rule_triggered(failures, "rule_6_dev_assets_path")

    def test_rule_6_dev_assets_with_backslashes_fails(self):
        row = _row(
            path="dev-assets\\krosmaga-proxy\\frames\\card_frame_common.png"
        )
        failures = check_release.check_manifest(_manifest(row))
        self._assert_rule_triggered(failures, "rule_6_dev_assets_path")

    def test_rule_6_dev_assets_leading_dot_slash_fails(self):
        row = _row(path="./dev-assets/krosmaga-proxy/frames/card_frame_common.png")
        failures = check_release.check_manifest(_manifest(row))
        self._assert_rule_triggered(failures, "rule_6_dev_assets_path")

    def test_rule_6_dev_assets_lookalike_does_not_fail(self):
        # 'dev-assets-archive' must NOT match 'dev-assets/' prefix.
        row = _row(path="dev-assets-archive/frames/card_frame_common.png")
        failures = check_release.check_manifest(_manifest(row))
        self.assertEqual(failures, [])

    # Multi-rule row

    def test_multiple_rules_can_trigger_for_single_row(self):
        row = _row(
            source_class="licensed_krosmaga_dev_proxy",
            release_class="dev_only",
            workflow_status="needed",
            approval_evidence=None,
            path="dev-assets/krosmaga-proxy/frames/card_frame_common.png",
        )
        failures = check_release.check_manifest(_manifest(row))
        triggered = {f["rule"] for f in failures}
        self.assertIn("rule_1_krosmaga_proxy_source", triggered)
        self.assertIn("rule_3_release_class_blocks", triggered)
        self.assertIn("rule_4_workflow_not_approved", triggered)
        self.assertIn("rule_6_dev_assets_path", triggered)

    # Shape errors

    def test_missing_required_key_raises(self):
        row = _row()
        del row["release_class"]
        with self.assertRaises(ValueError):
            check_release.check_manifest(_manifest(row))

    def test_invalid_workflow_status_value_raises(self):
        row = _row(workflow_status="brand_new_status")
        with self.assertRaises(ValueError):
            check_release.check_manifest(_manifest(row))

    def test_invalid_source_class_value_raises(self):
        row = _row(source_class="something_else")
        with self.assertRaises(ValueError):
            check_release.check_manifest(_manifest(row))

    def test_invalid_release_class_value_raises(self):
        row = _row(release_class="something_else")
        with self.assertRaises(ValueError):
            check_release.check_manifest(_manifest(row))

    def test_logical_assets_must_be_list(self):
        with self.assertRaises(ValueError):
            check_release.check_manifest({"logical_assets": {}})

    def test_root_must_be_object(self):
        with self.assertRaises(ValueError):
            check_release.check_manifest([])  # type: ignore[arg-type]

    # Helper

    def _assert_rule_triggered(self, failures, rule_name):
        triggered = {f["rule"] for f in failures}
        self.assertIn(
            rule_name,
            triggered,
            f"expected {rule_name} in {sorted(triggered)}",
        )


class CliIntegrationTests(unittest.TestCase):
    """End-to-end CLI tests using the fixture files."""

    def test_clean_fixture_exits_zero(self):
        result = self._run(FIXTURE_DIR / "release-manifest-clean.json")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stderr.strip(), "")

    def test_krosmaga_leak_fixture_exits_one(self):
        result = self._run(FIXTURE_DIR / "release-manifest-krosmaga-leak.json")
        self.assertEqual(result.returncode, 1, result.stderr)
        payload = json.loads(result.stderr)
        rules = {entry["rule"] for entry in payload}
        self.assertIn("rule_1_krosmaga_proxy_source", rules)
        self.assertIn("rule_3_release_class_blocks", rules)
        self.assertIn("rule_4_workflow_not_approved", rules)

    def test_dev_path_fixture_exits_one(self):
        result = self._run(FIXTURE_DIR / "release-manifest-dev-path.json")
        self.assertEqual(result.returncode, 1, result.stderr)
        payload = json.loads(result.stderr)
        rules = {entry["rule"] for entry in payload}
        self.assertIn("rule_6_dev_assets_path", rules)

    def test_unapproved_fixture_exits_one(self):
        result = self._run(FIXTURE_DIR / "release-manifest-unapproved.json")
        self.assertEqual(result.returncode, 1, result.stderr)
        payload = json.loads(result.stderr)
        rules = {entry["rule"] for entry in payload}
        self.assertIn("rule_4_workflow_not_approved", rules)

    def test_missing_manifest_exits_two(self):
        result = self._run(FIXTURE_DIR / "does-not-exist.json")
        self.assertEqual(result.returncode, 2)

    def test_dev_pack_example_toml_is_not_a_release_manifest(self):
        # The example pack TOML at design/assets/provenance/dev-pack-example.toml
        # is documentation, NOT a release manifest. Sanity-check the validator
        # rejects it (rather than silently passing).
        result = self._run(
            Path(__file__).resolve().parents[2]
            / "design/assets/provenance/dev-pack-example.toml"
        )
        self.assertEqual(result.returncode, 2)

    def _run(self, manifest_path):
        return subprocess.run(
            [sys.executable, str(CHECK_SCRIPT), str(manifest_path)],
            capture_output=True,
            text=True,
        )


if __name__ == "__main__":
    unittest.main()
