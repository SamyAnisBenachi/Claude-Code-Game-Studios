# PROMPT 1785 — AUTOPLAY-VALIDATOR-RUN-PATH-BOM-REPAIR

## Summary

Repaired `tools/autoplay/validate_composite_run.py` so that `autoplay-run-path.txt`
is read with `encoding="utf-8-sig"` (BOM-tolerant), preventing false `PATH MISMATCH`
failures when PowerShell writes the file with `Set-Content -Encoding utf8`.
Also hardened `_print_result` against Windows CP1252 console encoding errors.
Added 4 focused Python tests covering BOM-prefix scenarios.

## Base SHA

`4ab3e71e9785d949fb644f8cb63b4a21a2511726` (origin/main — PROMPT 1770 docs mainland)

## Branch

`fix/1785-validator-run-path-bom-repair`

## Root Cause

`_check_run_path_file` read `autoplay-run-path.txt` with `encoding="utf-8"`.
PowerShell `Set-Content -Encoding utf8` prepends a 3-byte UTF-8 BOM (`\xef\xbb\xbf`).
When the BOM was present, `rp_text` became `﻿<path>` which never compared equal
to the bare `<path>` from `autoplay_artifact_dir`, causing a spurious `PATH MISMATCH`
failure on every run where `autoplay-run-path.txt` was written by PowerShell.

PROMPT 1778 fixed the same class of bug for `composite-summary.json` (which is
loaded with `json.load` after opening with `encoding="utf-8-sig"`) but did not
touch the `read_text` call for `autoplay-run-path.txt`.

## Changes

### `tools/autoplay/validate_composite_run.py`

- **Line 177**: `rp_path.read_text(encoding="utf-8")` → `read_text(encoding="utf-8-sig")`
- **`_print_result`**: extracted `_safe_print()` helper that catches `UnicodeEncodeError`
  and re-prints with `errors="replace"` so BOM/control characters in paths do not crash
  the validator on narrow-codepage consoles (CP1252, CP850). Validation outcome is
  unaffected — encoding errors are display-only.

### `tests/tools/autoplay/test_validate_composite_run.py`

Added `TestRunPathBomTolerance` class (4 tests):

| Test | What it verifies |
|------|-----------------|
| `test_run_path_bom_prefix_does_not_cause_path_mismatch` | BOM-only prefix → PASS |
| `test_run_path_bom_prefix_with_trailing_newline_passes` | BOM + `\n` → PASS (PowerShell newline) |
| `test_run_path_bom_with_wrong_path_still_fails` | BOM + wrong path → PATH MISMATCH still fires |
| `test_run_path_missing_with_bom_summary_fails` | Missing run-path file stays an error |

## Test Results

```
41 passed in 0.73s
```

All 41 tests pass (37 pre-existing + 4 new BOM tests for `autoplay-run-path.txt`).

## `git diff --check`

Clean — no trailing whitespace or other whitespace errors.

## Scope Clarification

This repair closes the `autoplay-run-path.txt` PATH MISMATCH false failure found
during PROMPT 1782 live verify. It does **not** address the screenshot distinctness
failure from PROMPT 1782 — that is a separate issue.

---

1785: AUTOPLAY-VALIDATOR-RUN-PATH-BOM-REPAIR: SHIPPED
