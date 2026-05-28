# PROMPT 2009 — Autoplay Midrun Viewport Shrink Guard Refresh After 2005

## Summary

Clean re-land of the autoplay midrun viewport shrink guard payload onto a
branch that is strict-FF over `origin/main` after PROMPT 2005 landed.

PROMPT 2003 was rejected by orchestrator verification: it was NOT fast-forward
over current main (PROMPT 2005 had since landed on main), and the PROMPT-2003
branch included unrelated lobby files (`client/src/ui/lobby.rs`,
`lobby_class_picker_layout_test.rs`, `lobby_layout_viewport_invariant_test.rs`)
that would have deleted PROMPT 1958/1973/1987/1998/2005 lobby reports.

This refresh recovers only the owned autoplay payload via targeted
`git checkout origin/work/PROMPT-2003 -- <files>` onto a clean branch from
`origin/main@fa189edf`.

## Branch Details

| Field | Value |
|-------|-------|
| Branch | `work/PROMPT-2009-autoplay-midrun-viewport-shrink-guard-after-2005` |
| Base | `origin/main@fa189edf403dcf9bc12eddc315caf23bb6095e9b` |
| Source for recovery | `origin/work/PROMPT-2003@616bdc9fc81ab0c422bd4a46090962373d5c7123` |
| Strict FF over origin/main | YES (`git merge-base --is-ancestor origin/main HEAD` → exit 0) |

## Files Added

| File | Status |
|------|--------|
| `tools/autoplay/viewport_shrink_guard.py` | A |
| `tests/tools/autoplay/test_viewport_shrink_guard.py` | A |
| `reports/PROMPT-2003-autoplay-midrun-viewport-shrink-guard-refresh-after-1980.md` | A |
| `reports/PROMPT-2009-autoplay-midrun-viewport-shrink-guard-refresh-after-2005.md` | A |

## git diff --name-status origin/main..HEAD

```
A       reports/PROMPT-2003-autoplay-midrun-viewport-shrink-guard-refresh-after-1980.md
A       reports/PROMPT-2009-autoplay-midrun-viewport-shrink-guard-refresh-after-2005.md
A       tests/tools/autoplay/test_viewport_shrink_guard.py
A       tools/autoplay/viewport_shrink_guard.py
```

Zero deletions. Zero modifications to existing files.

## git diff --check

Clean — no trailing whitespace or other diff issues.

## Python Test Results

```
platform win32 -- Python 3.12.10, pytest-9.0.3
collected 31 items

tests/tools/autoplay/test_viewport_shrink_guard.py::TestCheckViewportSize::test_check_viewport_size_valid_returns_ok PASSED
tests/tools/autoplay/test_viewport_shrink_guard.py::TestCheckViewportSize::test_check_viewport_size_larger_than_minimum_returns_ok PASSED
tests/tools/autoplay/test_viewport_shrink_guard.py::TestCheckViewportSize::test_check_viewport_size_width_too_small_blocked PASSED
tests/tools/autoplay/test_viewport_shrink_guard.py::TestCheckViewportSize::test_check_viewport_size_height_too_small_blocked PASSED
tests/tools/autoplay/test_viewport_shrink_guard.py::TestCheckViewportSize::test_check_viewport_size_both_too_small_blocked PASSED
tests/tools/autoplay/test_viewport_shrink_guard.py::TestCheckViewportSize::test_check_viewport_size_missing_size_blocked PASSED
tests/tools/autoplay/test_viewport_shrink_guard.py::TestCheckViewportSize::test_check_viewport_size_custom_minimum_respected PASSED
tests/tools/autoplay/test_viewport_shrink_guard.py::TestCheckViewportSize::test_check_viewport_size_reason_includes_minimum_dimensions PASSED
tests/tools/autoplay/test_viewport_shrink_guard.py::TestCheckClickTarget::test_check_click_target_centre_of_window_allowed PASSED
tests/tools/autoplay/test_viewport_shrink_guard.py::TestCheckClickTarget::test_check_click_target_top_left_corner_allowed PASSED
tests/tools/autoplay/test_viewport_shrink_guard.py::TestCheckClickTarget::test_check_click_target_just_inside_right_edge_allowed PASSED
tests/tools/autoplay/test_viewport_shrink_guard.py::TestCheckClickTarget::test_check_click_target_at_right_edge_blocked PASSED
tests/tools/autoplay/test_viewport_shrink_guard.py::TestCheckClickTarget::test_check_click_target_negative_x_blocked PASSED
tests/tools/autoplay/test_viewport_shrink_guard.py::TestCheckClickTarget::test_check_click_target_negative_y_blocked PASSED
tests/tools/autoplay/test_viewport_shrink_guard.py::TestCheckClickTarget::test_check_click_target_far_offscreen_blocked PASSED
tests/tools/autoplay/test_viewport_shrink_guard.py::TestCheckClickTarget::test_check_click_target_missing_size_blocked PASSED
tests/tools/autoplay/test_viewport_shrink_guard.py::TestCheckClickTarget::test_check_click_target_reason_includes_coordinates PASSED
tests/tools/autoplay/test_viewport_shrink_guard.py::TestCheckBeforeInput::test_check_before_input_key_only_action_passes PASSED
tests/tools/autoplay/test_viewport_shrink_guard.py::TestCheckBeforeInput::test_check_before_input_mouse_button_only_passes PASSED
tests/tools/autoplay/test_viewport_shrink_guard.py::TestCheckBeforeInput::test_check_before_input_valid_cursor_on_normal_viewport_passes PASSED
tests/tools/autoplay/test_viewport_shrink_guard.py::TestCheckBeforeInput::test_check_before_input_cursor_offscreen_blocked PASSED
tests/tools/autoplay/test_viewport_shrink_guard.py::TestCheckBeforeInput::test_check_before_input_viewport_too_small_blocked PASSED
tests/tools/autoplay/test_viewport_shrink_guard.py::TestCheckBeforeInput::test_check_before_input_midrun_shrink_from_valid_to_invalid_blocked PASSED
tests/tools/autoplay/test_viewport_shrink_guard.py::TestCheckBeforeInput::test_check_before_input_missing_window_size_blocked PASSED
tests/tools/autoplay/test_viewport_shrink_guard.py::TestCheckBeforeInput::test_check_before_input_reason_is_a_non_empty_string_when_blocked PASSED
tests/tools/autoplay/test_viewport_shrink_guard.py::TestCheckBeforeInput::test_check_before_input_empty_params_passes PASSED
tests/tools/autoplay/test_viewport_shrink_guard.py::TestDriverViewportGuardPresent::test_driver_has_exit_viewport_guard_constant PASSED
tests/tools/autoplay/test_viewport_shrink_guard.py::TestDriverViewportGuardPresent::test_driver_logs_viewport_guard_prefix PASSED
tests/tools/autoplay/test_viewport_shrink_guard.py::TestDriverViewportGuardPresent::test_driver_emits_viewport_checkpoint_kind PASSED
tests/tools/autoplay/test_viewport_shrink_guard.py::TestDriverViewportGuardPresent::test_driver_guard_is_inside_autoplay_input_branch PASSED
tests/tools/autoplay/test_viewport_shrink_guard.py::TestDriverViewportGuardPresent::test_driver_has_minimum_window_constants PASSED

31 passed in 0.05s
```

## Preservation Verification

PROMPT 1980 reports (1916/1948/1966/1980) — all present on main, untouched:
- `reports/PROMPT-1916-post-1912-autoplay-viewport-window-guard-focused-verify.md` ✓
- `reports/PROMPT-1948-post-1912-autoplay-viewport-window-guard-verify-report-refresh-after-1943.md` ✓
- `reports/PROMPT-1966-post-1912-autoplay-viewport-window-guard-verify-report-refresh-after-1957.md` ✓
- `reports/PROMPT-1980-post-1912-autoplay-viewport-window-guard-verify-report-refresh-after-1976.md` ✓

PROMPT 2005 lobby reports/payload (1958/1973/1987/1998/2005) — all present on main, untouched:
- `reports/PROMPT-1958-ui-1280-lobby-class-picker-reachability-refresh-after-1920.md` ✓
- `reports/PROMPT-1973-ui-1280-lobby-class-picker-reachability-refresh-after-1959.md` ✓
- `reports/PROMPT-1987-ui-1280-lobby-class-picker-reachability-refresh-after-1976.md` ✓
- `reports/PROMPT-1998-ui-1280-lobby-class-picker-reachability-refresh-after-1994.md` ✓
- `reports/PROMPT-2005-ui-1280-lobby-class-picker-reachability-refresh-after-1980.md` ✓
- `client/src/ui/lobby.rs` ✓
- `tests/integration/playable_client/lobby_class_picker_layout_test.rs` ✓
- `tests/integration/playable_client/lobby_layout_viewport_invariant_test.rs` ✓

2009: AUTOPLAY-MIDRUN-VIEWPORT-SHRINK-GUARD-REFRESH-AFTER-2005: READY_FOR_MAINLAND_ENQUEUE
