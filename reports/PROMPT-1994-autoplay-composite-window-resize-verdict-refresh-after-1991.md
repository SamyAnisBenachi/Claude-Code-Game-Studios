# PROMPT 1994 — Autoplay Composite Window-Resize Verdict Refresh After 1991

**Date:** 2026-05-28
**Branch:** work/PROMPT-1994
**Base:** origin/main @ 17b68aac (post-PROMPT-1991)
**Source branch (cherry-pick origin):** origin/work/PROMPT-1979 @ 075107d81fb1162ab8432f5eb859b3a76a0f4eb3

---

## Context

PROMPT 1979 prepared `work/PROMPT-1979` based on origin/main@32a59256 (PROMPT 1976).
Multiple PROMPTs (1985, 1988, 1991) landed on main after that, making 1979 NOT_FF.
The orchestrator rejected a wholesale merge because it would have deleted:
- bot/autoplay readiness reports: 1935/1970/1985
- Krosmaga tier-border reports: 1933/1961/1974/1986/1988
- Krosmaga hand-fan reports/payload: 1854/1878/1910/1947/1955/1963/1981/1991 and hand UI changes

This task reapplies the window-resize verdict payload cleanly onto current origin/main
(17b68aac) using a cherry-pick of the PROMPT 1979 commit — no wholesale merge, no stale
deletions.

---

## Source
- Reapplied from: PROMPT 1979 commit `075107d81fb1162ab8432f5eb859b3a76a0f4eb3`
- Base: `origin/main@17b68aac`
- Method: `git cherry-pick 075107d8 --no-commit` (clean, zero conflicts)

---

## Files Changed

### Modified
- `tools/autoplay/analyze_evidence_run.py` — window size tracking, `_parse_driver_timeline()`, `NEEDS_HUMAN_GUI` verdict downgrade for resize/height/ALL_FROZEN
- `tools/autoplay/validate_composite_run.py` — `_check_window_and_capture_integrity()` guard, tags: `WINDOW-RESIZE-DETECTED`, `WINDOW-HEIGHT-TOO-SMALL`, `WIN32-ALL-FROZEN`

### Added (new to main)
- `tests/tools/autoplay/test_window_resize_verdict.py` — 25 pytest cases covering both modules
- `reports/PROMPT-1850-autoplay-composite-window-resize-verdict-downgrade.md`
- `reports/PROMPT-1864-autoplay-composite-window-resize-verdict-mainland-refresh-after-1844.md`
- `reports/PROMPT-1873-autoplay-composite-window-resize-verdict-refresh-after-1858.md`
- `reports/PROMPT-1875-autoplay-composite-window-resize-verdict-refresh-after-1872.md`
- `reports/PROMPT-1913-autoplay-composite-window-resize-verdict-refresh-after-1894.md`
- `reports/PROMPT-1918-autoplay-composite-window-resize-verdict-refresh-after-1912.md`
- `reports/PROMPT-1945-autoplay-composite-window-resize-verdict-refresh-after-1939.md`
- `reports/PROMPT-1951-autoplay-composite-window-resize-verdict-refresh-after-1937.md`
- `reports/PROMPT-1969-autoplay-composite-window-resize-verdict-refresh-after-1959.md`
- `reports/PROMPT-1979-autoplay-composite-window-resize-verdict-refresh-after-1976.md`
- `reports/PROMPT-1994-autoplay-composite-window-resize-verdict-refresh-after-1991.md` (this file)

### Not touched (forbidden scope respected)
- `client/src/ui/hand/mod.rs` — untouched
- `tests/integration/hand-ui/**` — untouched
- `tests/unit/hand-ui/**` — untouched
- `production/**` — untouched
- PROMPT 1854/1878/1910/1947/1955/1963/1981/1991 hand-fan reports — preserved
- PROMPT 1933/1961/1974/1986/1988 tier-border reports — preserved
- PROMPT 1935/1970/1985 readiness reports — preserved

---

## Verdict Logic Summary

| Condition | Effect |
|---|---|
| Mid-run window resize detected | Verdict → `NEEDS_HUMAN_GUI` |
| Window height < 600px at any tick | Verdict → `NEEDS_HUMAN_GUI` |
| All win32 PrintWindow captures frozen | Verdict → `NEEDS_HUMAN_GUI` |
| Clean run (none of the above) | Verdict unchanged |
| `BLOCKED` outcome | Takes priority over window-resize downgrade |

MIN_WINDOW_HEIGHT = 600 is consistent between both modules (enforced by `TestConstants::test_min_window_height_matches_between_modules`).

---

## Test Output

```
============================= test session results ==============================
platform win32 -- Python 3.12.10, pytest-9.0.3, pluggy-1.6.0
rootdir: D:\_DEV\Work\gcs-app-worktrees\lanesandlies\PROMPT-1994
collected 25 items

tests/tools/autoplay/test_window_resize_verdict.py::TestParseDriverTimeline::test_single_stable_size_no_resize_event PASSED [  4%]
tests/tools/autoplay/test_window_resize_verdict.py::TestParseDriverTimeline::test_resize_event_counted PASSED [  8%]
tests/tools/autoplay/test_window_resize_verdict.py::TestParseDriverTimeline::test_missing_timeline_adds_warning PASSED [ 12%]
tests/tools/autoplay/test_window_resize_verdict.py::TestParseDriverTimeline::test_empty_timeline_leaves_fields_none PASSED [ 16%]
tests/tools/autoplay/test_window_resize_verdict.py::TestWin32CaptureQuality::test_all_ok_is_good PASSED [ 20%]
tests/tools/autoplay/test_window_resize_verdict.py::TestWin32CaptureQuality::test_all_frozen_is_all_frozen PASSED [ 24%]
tests/tools/autoplay/test_window_resize_verdict.py::TestWin32CaptureQuality::test_mixed_is_partial_frozen PASSED [ 28%]
tests/tools/autoplay/test_window_resize_verdict.py::TestWin32CaptureQuality::test_no_win32_lines_is_unknown PASSED [ 32%]
tests/tools/autoplay/test_window_resize_verdict.py::TestAnalyzeVerdictDowngrade::test_window_resize_triggers_needs_human_gui PASSED [ 36%]
tests/tools/autoplay/test_window_resize_verdict.py::TestAnalyzeVerdictDowngrade::test_below_min_height_no_resize_triggers_needs_human_gui PASSED [ 40%]
tests/tools/autoplay/test_window_resize_verdict.py::TestAnalyzeVerdictDowngrade::test_all_frozen_win32_triggers_needs_human_gui PASSED [ 44%]
tests/tools/autoplay/test_window_resize_verdict.py::TestAnalyzeVerdictDowngrade::test_clean_run_is_not_downgraded PASSED [ 48%]
tests/tools/autoplay/test_window_resize_verdict.py::TestAnalyzeVerdictDowngrade::test_window_resize_into_acceptable_height_still_fails PASSED [ 52%]
tests/tools/autoplay/test_window_resize_verdict.py::TestAnalyzeVerdictDowngrade::test_blocked_outcome_takes_priority_over_window_resize PASSED [ 56%]
tests/tools/autoplay/test_window_resize_verdict.py::TestValidateWindowIntegrityGuard::test_window_resize_detected_fails PASSED [ 60%]
tests/tools/autoplay/test_window_resize_verdict.py::TestValidateWindowIntegrityGuard::test_window_height_too_small_fails PASSED [ 64%]
tests/tools/autoplay/test_window_resize_verdict.py::TestValidateWindowIntegrityGuard::test_win32_all_frozen_fails PASSED [ 68%]
tests/tools/autoplay/test_window_resize_verdict.py::TestValidateWindowIntegrityGuard::test_clean_run_no_window_failures PASSED [ 72%]
tests/tools/autoplay/test_window_resize_verdict.py::TestValidateWindowIntegrityGuard::test_missing_timeline_is_warning_not_failure PASSED [ 76%]
tests/tools/autoplay/test_window_resize_verdict.py::TestValidateWindowIntegrityGuard::test_missing_driver_log_is_warning_not_failure PASSED [ 80%]
tests/tools/autoplay/test_window_resize_verdict.py::TestValidateWindowIntegrityGuard::test_height_at_threshold_does_not_fail PASSED [ 84%]
tests/tools/autoplay/test_window_resize_verdict.py::TestValidateWindowIntegrityGuard::test_height_one_below_threshold_fails PASSED [ 88%]
tests/tools/autoplay/test_window_resize_verdict.py::TestValidateWindowIntegrityGuard::test_win32_mixed_frozen_ok_is_not_all_frozen PASSED [ 92%]
tests/tools/autoplay/test_window_resize_verdict.py::TestConstants::test_min_window_height_matches_between_modules PASSED [ 96%]
tests/tools/autoplay/test_window_resize_verdict.py::TestConstants::test_min_window_height_is_below_expected_resolution PASSED [100%]

============================= 25 passed in 0.38s ==============================
```

---

## FF Status

- `git merge-base --is-ancestor origin/main HEAD`: PASS (branch fast-forwarded from origin/main)
- `git diff --name-status origin/main..HEAD` (post-commit):

```
A  reports/PROMPT-1850-autoplay-composite-window-resize-verdict-downgrade.md
A  reports/PROMPT-1864-autoplay-composite-window-resize-verdict-mainland-refresh-after-1844.md
A  reports/PROMPT-1873-autoplay-composite-window-resize-verdict-refresh-after-1858.md
A  reports/PROMPT-1875-autoplay-composite-window-resize-verdict-refresh-after-1872.md
A  reports/PROMPT-1913-autoplay-composite-window-resize-verdict-refresh-after-1894.md
A  reports/PROMPT-1918-autoplay-composite-window-resize-verdict-refresh-after-1912.md
A  reports/PROMPT-1945-autoplay-composite-window-resize-verdict-refresh-after-1939.md
A  reports/PROMPT-1951-autoplay-composite-window-resize-verdict-refresh-after-1937.md
A  reports/PROMPT-1969-autoplay-composite-window-resize-verdict-refresh-after-1959.md
A  reports/PROMPT-1979-autoplay-composite-window-resize-verdict-refresh-after-1976.md
A  reports/PROMPT-1994-autoplay-composite-window-resize-verdict-refresh-after-1991.md
A  tests/tools/autoplay/test_window_resize_verdict.py
M  tools/autoplay/analyze_evidence_run.py
M  tools/autoplay/validate_composite_run.py
```

All 14 files are within owned scope. No deletions (zero D-prefixed lines).

---

## C0 / Human Review Caveat (preserved from 1969/1979)

The window-resize verdict downgrade to `NEEDS_HUMAN_GUI` is intentionally conservative.
Autoplay evidence affected by mid-run window resize or sub-600px height cannot be treated
as machine-verified PASS — an operator must inspect the screenshots manually to confirm
click targets landed on visible UI elements.

---

1994: AUTOPLAY-COMPOSITE-WINDOW-RESIZE-VERDICT-REFRESH-AFTER-1991: READY_FOR_MAINLAND_ENQUEUE
