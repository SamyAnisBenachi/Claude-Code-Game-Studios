# PROMPT 1778 — AUTOPLAY-COMPOSITE-VALIDATOR-UTF8-BOM-REPAIR

**Date:** 2026-05-28  
**Branch:** worker/1778-bom-repair  
**Status:** SHIPPED

## Problem

`validate_composite_run.py` opened `composite-summary.json` with
`encoding="utf-8"`, which raises `json.JSONDecodeError: Unexpected UTF-8 BOM`
when PowerShell writes the file (PowerShell's default is UTF-8 with BOM).
This broke the PROMPT 1775 live verification validator step.

## Fix

Changed `encoding="utf-8"` → `encoding="utf-8-sig"` in `_load_summary`
(`tools/autoplay/validate_composite_run.py` line ~128).  
Python's `utf-8-sig` codec silently strips the BOM when present and is
identical to `utf-8` for BOM-free files — no semantic change, no masked errors.

## Changed Files

| File | Change |
|---|---|
| `tools/autoplay/validate_composite_run.py` | `encoding="utf-8"` → `encoding="utf-8-sig"` in `_load_summary` |
| `tests/tools/autoplay/test_validate_composite_run.py` | Added `TestUtf8BomTolerance` class (3 new tests) |

## New Regression Tests (`TestUtf8BomTolerance`)

| Test | What it verifies |
|---|---|
| `test_validate_composite_run_bom_prefixed_summary_passes` | BOM-prefixed valid JSON → PASS |
| `test_validate_composite_run_bom_prefixed_summary_wrong_schema_still_fails` | BOM tolerance does not hide real validation failures |
| `test_validate_composite_run_bom_plus_malformed_json_fails_with_invalid_json` | BOM + garbage → INVALID JSON error surfaces correctly |

## Test Results

```
37 passed in 0.72s
```

All 34 pre-existing tests continue to pass. 3 new BOM tests pass.  
`git diff --check`: clean (no whitespace errors).

## Closes

The `Unexpected UTF-8 BOM` validator failure reported in PROMPT 1775.

1778: AUTOPLAY-COMPOSITE-VALIDATOR-UTF8-BOM-REPAIR: SHIPPED
