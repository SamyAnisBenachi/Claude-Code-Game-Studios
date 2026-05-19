#!/usr/bin/env python3
"""Release-scan validator for asset provenance (ADR-025).

Reads a JSON release manifest and fails (non-zero exit) when any logical
asset row violates one of the six release-gate rules defined in
docs/architecture/adr-025-asset-pack-provenance-architecture.md and
design/assets/provenance/schema.md.

The six rules:
  1. source_class == licensed_krosmaga_dev_proxy
  2. source_class == unknown_provenance
  3. release_class != release_allowed
  4. workflow_status != approved
  5. approval_evidence missing while workflow_status == approved
  6. path begins with dev-assets/

Exit codes:
  0  every row passes
  1  at least one row fails (JSON error report on stderr)
  2  manifest could not be parsed or required keys are missing

Pure Python 3 stdlib. No third-party dependencies.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any


REQUIRED_ROW_KEYS = (
    "logical_id",
    "workflow_status",
    "source_class",
    "release_class",
    "path",
)

VALID_WORKFLOW_STATUS = {
    "needed",
    "in_progress",
    "done",
    "approved",
    "blocked",
}

VALID_SOURCE_CLASS = {
    "studio_original",
    "licensed_external_release",
    "licensed_krosmaga_dev_proxy",
    "unknown_provenance",
}

VALID_RELEASE_CLASS = {
    "release_allowed",
    "dev_only",
    "internal_only",
}


def _check_row(row: dict[str, Any]) -> list[dict[str, Any]]:
    """Return a list of failure records for a single row (possibly empty)."""
    failures: list[dict[str, Any]] = []
    logical_id = row.get("logical_id", "<missing>")
    path = row.get("path", "")
    source_class = row.get("source_class")
    release_class = row.get("release_class")
    workflow_status = row.get("workflow_status")
    approval_evidence = row.get("approval_evidence")

    # Rule 1
    if source_class == "licensed_krosmaga_dev_proxy":
        failures.append(
            {
                "logical_id": logical_id,
                "rule": "rule_1_krosmaga_proxy_source",
                "value": source_class,
                "path": path,
                "detail": "Krosmaga proxy material is never release-eligible (ADR-025).",
            }
        )

    # Rule 2
    if source_class == "unknown_provenance":
        failures.append(
            {
                "logical_id": logical_id,
                "rule": "rule_2_unknown_provenance",
                "value": source_class,
                "path": path,
                "detail": "Unclassified provenance must be classified before release.",
            }
        )

    # Rule 3
    if release_class != "release_allowed":
        failures.append(
            {
                "logical_id": logical_id,
                "rule": "rule_3_release_class_blocks",
                "value": release_class,
                "path": path,
                "detail": "release_class must be 'release_allowed' for packaged assets.",
            }
        )

    # Rule 4
    if workflow_status != "approved":
        failures.append(
            {
                "logical_id": logical_id,
                "rule": "rule_4_workflow_not_approved",
                "value": workflow_status,
                "path": path,
                "detail": "workflow_status must be 'approved' to ship.",
            }
        )

    # Rule 5
    if workflow_status == "approved" and not approval_evidence:
        failures.append(
            {
                "logical_id": logical_id,
                "rule": "rule_5_missing_approval_evidence",
                "value": None,
                "path": path,
                "detail": "approval_evidence is required when workflow_status == approved.",
            }
        )

    # Rule 6
    normalized = path.replace("\\", "/").lstrip("./")
    if normalized.startswith("dev-assets/") or normalized == "dev-assets":
        failures.append(
            {
                "logical_id": logical_id,
                "rule": "rule_6_dev_assets_path",
                "value": path,
                "path": path,
                "detail": "Concrete path resolves under dev-assets/ — never release-eligible.",
            }
        )

    return failures


def _validate_row_shape(row: Any, row_index: int) -> str | None:
    """Return an error message if the row is malformed; None if shape is OK."""
    if not isinstance(row, dict):
        return f"logical_assets[{row_index}] is not an object."
    for key in REQUIRED_ROW_KEYS:
        if key not in row:
            return (
                f"logical_assets[{row_index}] missing required key '{key}'."
            )
    if row["workflow_status"] not in VALID_WORKFLOW_STATUS:
        return (
            f"logical_assets[{row_index}] workflow_status "
            f"'{row['workflow_status']}' is not in {sorted(VALID_WORKFLOW_STATUS)}."
        )
    if row["source_class"] not in VALID_SOURCE_CLASS:
        return (
            f"logical_assets[{row_index}] source_class "
            f"'{row['source_class']}' is not in {sorted(VALID_SOURCE_CLASS)}."
        )
    if row["release_class"] not in VALID_RELEASE_CLASS:
        return (
            f"logical_assets[{row_index}] release_class "
            f"'{row['release_class']}' is not in {sorted(VALID_RELEASE_CLASS)}."
        )
    return None


def check_manifest(manifest: dict[str, Any]) -> list[dict[str, Any]]:
    """Return a list of failure records. Empty list means PASS.

    Raises ValueError if the manifest is structurally malformed.
    """
    if not isinstance(manifest, dict):
        raise ValueError("release manifest root must be an object.")
    rows = manifest.get("logical_assets")
    if not isinstance(rows, list):
        raise ValueError(
            "release manifest must contain a 'logical_assets' list."
        )

    failures: list[dict[str, Any]] = []
    for index, row in enumerate(rows):
        shape_error = _validate_row_shape(row, index)
        if shape_error is not None:
            raise ValueError(shape_error)
        failures.extend(_check_row(row))

    return failures


def _load_manifest(path: Path) -> dict[str, Any]:
    text = path.read_text(encoding="utf-8")
    return json.loads(text)


def main(argv: list[str]) -> int:
    if len(argv) != 2:
        sys.stderr.write(
            "Usage: check_release.py <release-manifest.json>\n"
        )
        return 2
    manifest_path = Path(argv[1])
    if not manifest_path.is_file():
        sys.stderr.write(
            f"Manifest not found or not a file: {manifest_path}\n"
        )
        return 2
    try:
        manifest = _load_manifest(manifest_path)
    except json.JSONDecodeError as exc:
        sys.stderr.write(f"Manifest JSON parse error: {exc}\n")
        return 2

    try:
        failures = check_manifest(manifest)
    except ValueError as exc:
        sys.stderr.write(f"Manifest structural error: {exc}\n")
        return 2

    if failures:
        sys.stderr.write(json.dumps(failures, indent=2, sort_keys=True))
        sys.stderr.write("\n")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
