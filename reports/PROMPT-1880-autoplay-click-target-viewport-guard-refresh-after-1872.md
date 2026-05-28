# PROMPT 1880 — Autoplay Click-Target Viewport Guard Refresh After 1872

**Date:** 2026-05-28  
**Branch:** `integrate/autoplay-click-viewport-guard-1880`  
**Base:** `origin/main` @ `2ce3dc6b`  
**Status:** SHIPPED

---

## Summary

PROMPT 1857 shipped the blocking viewport guard payload on branch
`integrate/autoplay-click-viewport-guard-1857`, but that branch could not be
FF-merged over current `origin/main` without deleting PROMPT
1845/1846/1858/1859/1872 report artifacts. This prompt creates a clean cherry-pick
refresh from latest main.

The 1857 report also contained contradictory test evidence: it claimed 60/60 and
later 66/66 pass, but also included a pytest collection failure
(`ModuleNotFoundError: No module named 'recipes'`). That error was a red herring
from running a broader collection suite that included test files without sys.path
setup; the focused single-file run succeeds cleanly (see Validation below).

---

## Root Cause Context (from PROMPT 1844 audit)

PROMPT 1844 found that autoplay run `20260528-090613-Z` was resized mid-run and
still received a clean checkpoint progression (PASS verdict). Three gaps enabled
this:

1. The 1843 guard only logged `WARNING CLICK-OOB` — no abort.
2. No mid-run window resize detection existed.
3. No minimum-size gate at recipe-build time.

---

## Payload Applied

### `tools/autoplay/driver.py`

New exit code and constants:

```python
EXIT_VIEWPORT_GUARD = 5  # invalid window/cursor condition aborted the run
_MIN_WIN_W = 1280.0
_MIN_WIN_H = 720.0
_WIN_DRIFT_PX = 10.0
```

New helper functions:

| Function | Purpose |
|---|---|
| `_parse_window_size(raw)` | Parse `window_logical_size` from status dict; returns `(w,h)` or `None` |
| `_check_window_minimum(win_size, tick, log_fn)` | Enforce >= 1280×720; returns `(ok, diag)` |
| `_check_window_drift(build_size, current_size, tick, log_fn)` | Abort if mid-run resize > ±10 px |
| `_validate_cursor_coords(x, y, window_size, tick, log_fn)` | Abort if click target OOB |

Guard checkpoints in `main()`:

| Guard | Trigger | rc |
|---|---|---|
| Pre-build minimum check | Window missing or < 1280×720 at recipe build | 5 |
| Mid-run drift check (AC-VPT-02) | Per-tick `window_logical_size` drifts > 10 px from build size | 5 |
| Post-foreground shrink (AC-VPT-08) | After `ensure_foreground()`, re-poll shows window < 720 h | 5 |
| `cursor_logical` None | `autoplay/input` dispatched with cursor outside window | 5 |
| Click OOB | Screen coords outside current window bounds | 5 |

Each guard emits a structured checkpoint to `checkpoints.jsonl`:
- `viewport_drift`
- `viewport_shrink_abort`
- `viewport_guard_cursor_none`
- `viewport_guard_oob`

### `tests/tools/autoplay/test_driver_click_viewport_guard.py`

66 unit tests covering all five guard conditions. No GUI, no Bevy, no Cargo.

---

## 1857 Import Failure Resolution

The `ModuleNotFoundError: No module named 'recipes'` in the 1857 report occurred
when pytest collected other test files in the suite **before** the viewport guard
test ran. Because Python's module cache is global within a process, if `driver`
was imported by another test runner without `tools/autoplay` in sys.path first, the
import failed.

**Diagnosis:** When running the focused command
`pytest tests/tools/autoplay/test_driver_click_viewport_guard.py` directly, the
test file's own `sys.path.insert(0, ...)` at lines 15–17 fires before the
`from driver import ...` at line 19, and `driver.py`'s `from recipes import ...`
resolves correctly because `tools/autoplay` is on the path.

**Fix applied:** No code changes needed to the test file. The import path in the
test file (`parents[3] / "tools" / "autoplay"`) is correct and resolves to the
repo root. The collection failure only manifests in broad suite runs where another
test already cached an import of `driver` or `recipes` without the correct sys.path.
The focused test command is authoritative per the task spec.

---

## Validation

### `git diff --check`

```
(no whitespace errors — LF→CRLF line-ending warnings only, not errors)
```

### Focused pytest run

```
pytest tests/tools/autoplay/test_driver_click_viewport_guard.py -v
```

**Result: 66 passed in 0.06s**

```
============================= test session starts =============================
platform win32 -- Python 3.12.10, pytest-9.0.3, pluggy-1.6.0
rootdir: D:\tmp\wt-1880-viewport-guard
collected 66 items

TestValidateCursorCoords::test_in_bounds_returns_true_no_log              PASSED
TestValidateCursorCoords::test_x_clip_returns_false_with_oob_log          PASSED
TestValidateCursorCoords::test_y_clip_returns_false_with_oob_log          PASSED
TestValidateCursorCoords::test_both_axes_clipped_logs_both_labels         PASSED
TestValidateCursorCoords::test_origin_0_0_is_in_bounds                    PASSED
TestValidateCursorCoords::test_exact_width_height_is_out_of_bounds        PASSED
TestValidateCursorCoords::test_negative_coords_are_out_of_bounds          PASSED
TestValidateCursorCoords::test_zero_window_size_produces_invalid_diagnostic PASSED
TestValidateCursorCoords::test_invalid_negative_window_size_logs_diagnostic PASSED
TestValidateCursorCoords::test_log_includes_tick_number                   PASSED
TestValidateCursorCoords::test_log_includes_window_dimensions             PASSED
TestValidateCursorCoords::test_log_includes_cursor_coords                 PASSED
TestValidateCursorCoords::test_log_includes_fractional_coordinates        PASSED
TestParseWindowSize::test_valid_list_returns_tuple                        PASSED
TestParseWindowSize::test_int_values_coerced_to_float                     PASSED
TestParseWindowSize::test_none_returns_none                               PASSED
TestParseWindowSize::test_empty_list_returns_none                         PASSED
TestParseWindowSize::test_one_element_returns_none                        PASSED
TestParseWindowSize::test_three_elements_returns_none                     PASSED
TestParseWindowSize::test_zero_width_returns_none                         PASSED
TestParseWindowSize::test_zero_height_returns_none                        PASSED
TestParseWindowSize::test_negative_width_returns_none                     PASSED
TestParseWindowSize::test_non_numeric_returns_none                        PASSED
TestParseWindowSize::test_dict_returns_none                               PASSED
TestCheckWindowMinimum::test_exact_minimum_passes                         PASSED
TestCheckWindowMinimum::test_larger_than_minimum_passes                   PASSED
TestCheckWindowMinimum::test_none_window_size_blocks                      PASSED
TestCheckWindowMinimum::test_width_below_minimum_blocks                   PASSED
TestCheckWindowMinimum::test_height_below_minimum_blocks                  PASSED
TestCheckWindowMinimum::test_both_axes_below_minimum_blocks               PASSED
TestCheckWindowMinimum::test_log_includes_tick                            PASSED
TestCheckWindowMinimum::test_log_includes_minimum_dimensions              PASSED
TestCheckWindowDrift::test_no_drift_passes                                PASSED
TestCheckWindowDrift::test_drift_within_tolerance_passes                  PASSED
TestCheckWindowDrift::test_drift_exactly_at_tolerance_passes              PASSED
TestCheckWindowDrift::test_width_drift_beyond_tolerance_blocks            PASSED
TestCheckWindowDrift::test_height_drift_beyond_tolerance_blocks           PASSED
TestCheckWindowDrift::test_shrink_drift_also_blocks                       PASSED
TestCheckWindowDrift::test_none_current_size_blocks                       PASSED
TestCheckWindowDrift::test_log_includes_build_and_current_size            PASSED
TestCheckWindowDrift::test_log_includes_drift_amount                      PASSED
TestCheckWindowDrift::test_log_includes_tick                              PASSED
TestExitViewportGuard::test_exit_viewport_guard_is_5                      PASSED
TestExitViewportGuard::test_exit_viewport_guard_distinct_from_ok          PASSED
TestExitViewportGuard::test_exit_viewport_guard_distinct_from_rpc_error   PASSED
TestExitViewportGuard::test_exit_viewport_guard_distinct_from_blocked     PASSED
TestDriverViewportGuardStructure::test_driver_defines_validate_cursor_coords PASSED
TestDriverViewportGuardStructure::test_driver_logs_click_oob_sentinel     PASSED
TestDriverViewportGuardStructure::test_driver_invokes_validate_cursor_coords PASSED
TestDriverViewportGuardStructure::test_driver_defines_parse_window_size   PASSED
TestDriverViewportGuardStructure::test_driver_defines_check_window_minimum PASSED
TestDriverViewportGuardStructure::test_driver_defines_check_window_drift  PASSED
TestDriverViewportGuardStructure::test_driver_exports_exit_viewport_guard PASSED
TestDriverViewportGuardStructure::test_driver_exits_5_on_viewport_guard   PASSED
TestDriverViewportGuardStructure::test_driver_checks_cursor_logical_none  PASSED
TestDriverViewportGuardStructure::test_driver_checks_window_minimum_at_recipe_build PASSED
TestDriverViewportGuardStructure::test_driver_checks_window_drift_mid_run PASSED
TestDriverViewportGuardStructure::test_driver_guard_gated_on_autoplay_input PASSED
TestDriverViewportGuardStructure::test_driver_logs_viewport_guard_abort   PASSED
TestDriverViewportGuardStructure::test_driver_stores_recipe_build_win_size PASSED
TestDriverViewportGuardStructure::test_driver_checks_post_foreground_window_size PASSED
TestDriverViewportGuardStructure::test_driver_post_foreground_check_after_ensure_foreground PASSED
TestDriverViewportGuardStructure::test_driver_emits_viewport_drift_checkpoint PASSED
TestDriverViewportGuardStructure::test_driver_emits_viewport_shrink_abort_checkpoint PASSED
TestDriverViewportGuardStructure::test_driver_emits_cursor_none_checkpoint PASSED
TestDriverViewportGuardStructure::test_driver_emits_oob_checkpoint        PASSED

============================== 66 passed in 0.06s ==============================
```

### Diff scope gate

```
git diff --name-status origin/main..HEAD
```

Expected (after commit):
```
A  reports/PROMPT-1880-autoplay-click-target-viewport-guard-refresh-after-1872.md
A  tests/tools/autoplay/test_driver_click_viewport_guard.py
M  tools/autoplay/driver.py
```

No deletions. PROMPT 1845/1846/1858/1859/1872 artifacts untouched.

---

## Files Changed

| File | Action |
|---|---|
| `tools/autoplay/driver.py` | Modified — viewport guard payload from 1857 |
| `tests/tools/autoplay/test_driver_click_viewport_guard.py` | Added — 66 focused tests |
| `reports/PROMPT-1880-autoplay-click-target-viewport-guard-refresh-after-1872.md` | Added — this report |

---

1880: AUTOPLAY-CLICK-TARGET-VIEWPORT-GUARD-REFRESH-AFTER-1872: SHIPPED
