#!/usr/bin/env python3
"""Validate a dev-only Krosmaga proxy asset pack manifest.

This is a tooling-only guard for future local proxy packs. It validates
metadata only; it does not copy, inspect, or materialize Krosmaga payload
assets.

Exit codes:
  0  manifest passes
  1  manifest parsed, but one or more rows violate the dev-proxy policy
  2  manifest could not be parsed or has an invalid top-level shape
"""

from __future__ import annotations

import json
import sys
from pathlib import PurePosixPath, PureWindowsPath, Path
from typing import Any


VALID_MATCH_QUALITY = {
    "exact",
    "good",
    "needs_conversion",
    "ambiguous",
    "missing",
    "no_art_needed",
}

VALID_SOURCE_CLASS = {"licensed_krosmaga_dev_proxy"}
VALID_RELEASE_CLASS = {"dev_only"}
VALID_WORKFLOW_STATUS = {"needed"}

LOGICAL_ID_PREFIX = "lid_"
_LOGICAL_ID_BODY_CHARS = set(
    "abcdefghijklmnopqrstuvwxyz0123456789_"
)
_CONSUMER_SURFACE_CHARS = set(
    "abcdefghijklmnopqrstuvwxyz0123456789_."
)

REQUIRED_ENTRY_KEYS = (
    "logical_id",
    "source_path",
    "match_quality",
    "dev_only",
    "source_class",
    "release_class",
    "workflow_status",
    "license_provenance_warning",
    "expected_consumer_surface",
)


def _failure(
    logical_id: str,
    rule: str,
    value: Any,
    detail: str,
    source_path: Any = None,
) -> dict[str, Any]:
    return {
        "logical_id": logical_id,
        "rule": rule,
        "value": value,
        "source_path": source_path,
        "detail": detail,
    }


def _path_parts(path_value: str) -> list[str]:
    normalized = path_value.replace("\\", "/").strip()
    return [part.lower() for part in PurePosixPath(normalized).parts]


def _is_repo_assets_path(path_value: str) -> bool:
    """Return True when a source path resolves inside this repo's assets tree."""
    if not path_value:
        return False

    posix_parts = _path_parts(path_value)
    if posix_parts[:1] == ["assets"]:
        return True
    if len(posix_parts) >= 2 and posix_parts[0] == "." and posix_parts[1] == "assets":
        return True

    windows_parts = [part.lower() for part in PureWindowsPath(path_value).parts]
    return "claude-code-game-studios" in windows_parts and "assets" in windows_parts


def _warning_mentions_dev_only(warning: str) -> bool:
    lowered = warning.lower()
    return (
        "dev-only" in lowered
        or "dev only" in lowered
        or "not release" in lowered
        or "never release" in lowered
        or "not approved" in lowered
    )


def _validate_entry_shape(row: Any, row_index: int) -> str | None:
    if not isinstance(row, dict):
        return f"entries[{row_index}] is not an object."
    for key in REQUIRED_ENTRY_KEYS:
        if key not in row:
            return f"entries[{row_index}] missing required key '{key}'."
    return None


def _check_entry(row: dict[str, Any]) -> list[dict[str, Any]]:
    logical_id = str(row.get("logical_id", "<missing>"))
    source_path = row.get("source_path")
    match_quality = row.get("match_quality")
    failures: list[dict[str, Any]] = []

    if not isinstance(row.get("logical_id"), str) or not row["logical_id"].strip():
        failures.append(
            _failure(
                logical_id,
                "logical_id_required",
                row.get("logical_id"),
                "logical_id must be a non-empty string.",
                source_path,
            )
        )
    elif not row["logical_id"].startswith(LOGICAL_ID_PREFIX) or not row["logical_id"][len(LOGICAL_ID_PREFIX):]:
        failures.append(
            _failure(
                logical_id,
                "logical_id_prefix_required",
                row.get("logical_id"),
                f"logical_id must start with '{LOGICAL_ID_PREFIX}' and carry a non-empty body (schema.md Logical Asset ID Layer).",
                source_path,
            )
        )
    elif not set(row["logical_id"][len(LOGICAL_ID_PREFIX):]).issubset(_LOGICAL_ID_BODY_CHARS):
        failures.append(
            _failure(
                logical_id,
                "logical_id_body_charset",
                row.get("logical_id"),
                "logical_id body must use only lowercase letters, digits, and underscores after the 'lid_' prefix.",
                source_path,
            )
        )

    if row.get("dev_only") is not True:
        failures.append(
            _failure(
                logical_id,
                "dev_only_required",
                row.get("dev_only"),
                "Krosmaga proxy entries must explicitly set dev_only=true.",
                source_path,
            )
        )

    if row.get("source_class") not in VALID_SOURCE_CLASS:
        failures.append(
            _failure(
                logical_id,
                "source_class_must_be_krosmaga_proxy",
                row.get("source_class"),
                "Krosmaga proxy entries must use source_class=licensed_krosmaga_dev_proxy.",
                source_path,
            )
        )

    if row.get("release_class") not in VALID_RELEASE_CLASS:
        failures.append(
            _failure(
                logical_id,
                "release_class_must_be_dev_only",
                row.get("release_class"),
                "Krosmaga proxy entries must use release_class=dev_only.",
                source_path,
            )
        )

    if row.get("workflow_status") not in VALID_WORKFLOW_STATUS:
        failures.append(
            _failure(
                logical_id,
                "workflow_status_must_remain_needed",
                row.get("workflow_status"),
                "Krosmaga proxy entries must remain workflow_status=needed.",
                source_path,
            )
        )

    warning = row.get("license_provenance_warning")
    if not isinstance(warning, str) or not warning.strip():
        failures.append(
            _failure(
                logical_id,
                "license_warning_required",
                warning,
                "license_provenance_warning must be a non-empty string.",
                source_path,
            )
        )
    elif not _warning_mentions_dev_only(warning):
        failures.append(
            _failure(
                logical_id,
                "license_warning_must_block_release_claim",
                warning,
                "license_provenance_warning must state that the proxy is dev-only or not release-approved.",
                source_path,
            )
        )

    if match_quality not in VALID_MATCH_QUALITY:
        failures.append(
            _failure(
                logical_id,
                "invalid_match_quality",
                match_quality,
                f"match_quality must be one of {sorted(VALID_MATCH_QUALITY)}.",
                source_path,
            )
        )

    consumer_surface = row.get("expected_consumer_surface")
    if not isinstance(consumer_surface, str) or not consumer_surface.strip():
        failures.append(
            _failure(
                logical_id,
                "consumer_surface_required",
                consumer_surface,
                "expected_consumer_surface must name the future consumer surface.",
                source_path,
            )
        )
    elif (
        "." not in consumer_surface
        or not set(consumer_surface).issubset(_CONSUMER_SURFACE_CHARS)
        or consumer_surface.startswith(".")
        or consumer_surface.endswith(".")
    ):
        failures.append(
            _failure(
                logical_id,
                "consumer_surface_format",
                consumer_surface,
                "expected_consumer_surface must be a dotted lowercase token path (e.g. 'hand.card_frame').",
                source_path,
            )
        )

    if match_quality in {"missing", "no_art_needed"}:
        if source_path not in (None, ""):
            failures.append(
                _failure(
                    logical_id,
                    "missing_rows_must_not_claim_source",
                    source_path,
                    "missing/no_art_needed rows must not name a source_path.",
                    source_path,
                )
            )
        handling = row.get("missing_handling")
        if not isinstance(handling, str) or not handling.strip():
            failures.append(
                _failure(
                    logical_id,
                    "missing_handling_required",
                    handling,
                    "missing/no_art_needed rows must document missing_handling.",
                    source_path,
                )
            )
    else:
        if not isinstance(source_path, str) or not source_path.strip():
            failures.append(
                _failure(
                    logical_id,
                    "source_path_required",
                    source_path,
                    "Non-missing proxy rows must name a source_path.",
                    source_path,
                )
            )
        elif _is_repo_assets_path(source_path):
            failures.append(
                _failure(
                    logical_id,
                    "source_path_must_not_be_repo_assets",
                    source_path,
                    "source_path points inside repo assets/**, which would imply copied Krosmaga content.",
                    source_path,
                )
            )

    if match_quality == "needs_conversion":
        notes = row.get("conversion_notes")
        if not isinstance(notes, str) or not notes.strip():
            failures.append(
                _failure(
                    logical_id,
                    "conversion_notes_required",
                    notes,
                    "needs_conversion rows must document conversion_notes (resize/atlas/audio remux/etc).",
                    source_path,
                )
            )

    if match_quality == "ambiguous":
        if row.get("manual_review_required") is not True:
            failures.append(
                _failure(
                    logical_id,
                    "ambiguous_requires_manual_review",
                    row.get("manual_review_required"),
                    "ambiguous rows must set manual_review_required=true.",
                    source_path,
                )
            )
        notes = row.get("ambiguity_notes")
        if not isinstance(notes, str) or not notes.strip():
            failures.append(
                _failure(
                    logical_id,
                    "ambiguity_notes_required",
                    notes,
                    "ambiguous rows must explain the ambiguity.",
                    source_path,
                )
            )

    return failures


def validate_manifest(manifest: dict[str, Any]) -> list[dict[str, Any]]:
    """Return policy failures. Raise ValueError for malformed manifest shape."""
    if not isinstance(manifest, dict):
        raise ValueError("dev proxy pack manifest root must be an object.")

    pack = manifest.get("pack")
    if not isinstance(pack, dict):
        raise ValueError("dev proxy pack manifest must contain a 'pack' object.")

    pack_failures: list[dict[str, Any]] = []
    if pack.get("dev_only") is not True:
        pack_failures.append(
            _failure(
                "<pack>",
                "pack_dev_only_required",
                pack.get("dev_only"),
                "The pack must explicitly set dev_only=true.",
            )
        )
    if pack.get("source_class") != "licensed_krosmaga_dev_proxy":
        pack_failures.append(
            _failure(
                "<pack>",
                "pack_source_class_must_be_krosmaga_proxy",
                pack.get("source_class"),
                "The pack must use source_class=licensed_krosmaga_dev_proxy.",
            )
        )
    if pack.get("release_class") != "dev_only":
        pack_failures.append(
            _failure(
                "<pack>",
                "pack_release_class_must_be_dev_only",
                pack.get("release_class"),
                "The pack must use release_class=dev_only.",
            )
        )
    if "workflow_status" in pack and pack.get("workflow_status") not in VALID_WORKFLOW_STATUS:
        pack_failures.append(
            _failure(
                "<pack>",
                "pack_workflow_status_must_remain_needed",
                pack.get("workflow_status"),
                "If pack.workflow_status is present it must remain 'needed' — a Krosmaga pack cannot advance the workflow.",
            )
        )
    pack_id = pack.get("pack_id")
    if not isinstance(pack_id, str) or not pack_id.strip():
        pack_failures.append(
            _failure(
                "<pack>",
                "pack_id_required",
                pack_id,
                "pack.pack_id must be a non-empty string.",
            )
        )

    entries = manifest.get("entries")
    if not isinstance(entries, list):
        raise ValueError("dev proxy pack manifest must contain an 'entries' list.")

    failures = list(pack_failures)
    seen: set[str] = set()
    for index, row in enumerate(entries):
        shape_error = _validate_entry_shape(row, index)
        if shape_error is not None:
            raise ValueError(shape_error)
        logical_id = row["logical_id"]
        if isinstance(logical_id, str):
            if logical_id in seen:
                failures.append(
                    _failure(
                        logical_id,
                        "duplicate_logical_id",
                        logical_id,
                        "Each logical_id may appear at most once in a dev proxy pack manifest.",
                        row.get("source_path"),
                    )
                )
            seen.add(logical_id)
        failures.extend(_check_entry(row))

    return failures


def _load_manifest(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def main(argv: list[str]) -> int:
    if len(argv) != 2:
        sys.stderr.write(
            "Usage: validate_dev_proxy_pack.py <dev-proxy-pack-manifest.json>\n"
        )
        return 2

    manifest_path = Path(argv[1])
    if not manifest_path.is_file():
        sys.stderr.write(f"Manifest not found or not a file: {manifest_path}\n")
        return 2

    try:
        manifest = _load_manifest(manifest_path)
    except json.JSONDecodeError as exc:
        sys.stderr.write(f"Manifest JSON parse error: {exc}\n")
        return 2

    try:
        failures = validate_manifest(manifest)
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
