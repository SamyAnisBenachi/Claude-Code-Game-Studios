# PROMPT 1979 — Autoplay Composite Window-Resize Verdict Refresh After 1976

**Date:** 2026-05-28
**Branch:** work/PROMPT-1979
**Base:** origin/main @ 32a59256d1de9a4fee362a2aa9006d1bb69b59db (post-PROMPT-1976)
**Source branch (file transplant origin):** origin/integrate/autoplay-composite-window-resize-verdict-1969 @ 1495f59705498948384aca38e1148bfb2501a429

---

## Context

PROMPT 1969 prepared `integrate/autoplay-composite-window-resize-verdict-1969` based on
origin/main@7fc1706e (PROMPT 1959). Multiple PROMPTs (1972, 1976) landed on main after
that, making 1969 NOT_FF and causing the orchestrator to reject a wholesale merge (it
would have deleted PROMPT 1972/1976 work).

This task reapplies the window-resize verdict payload cleanly onto current origin/main
(32a59256) using a file-level transplant — no wholesale cherry-pick, no stale deletions.

---

## Source
- Reapplied from: PROMPT 1969 branch tip `1495f59705498948384aca38e1148bfb2501a429`
- Base: `origin/main@32a59256d1de9a4fee362a2aa9006d1bb69b59db`
- Method: file-level transplant (no wholesale cherry-pick)

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
- `reports/PROMPT-1979-autoplay-composite-window-resize-verdict-refresh-after-1976.md` (this file)

### Not touched (forbidden scope respected)
- `client/**` — untouched
- `server/**` — untouched
- `Cargo.*` — untouched
- `production/**` — untouched
- PROMPT 1972 reports — preserved
- PROMPT 1976 reports — preserved

---

## Transplant Method

File-level transplant from `origin/integrate/autoplay-composite-window-resize-verdict-1969`:

```bash
git show 1495f59705498948384aca38e1148bfb2501a429:<file> > <worktree>/<file>
```

Applied for all 11 owned files from the 1969 branch. No wholesale cherry-pick — avoids
stale deletions in 1969 branch ancestry.

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
============================= test session starts =============================
platform win32 -- Python 3.12.10, pytest-9.0.3, pluggy-1.6.0
rootdir: D:\_DEV\Work\gcs-app-worktrees\lanesandlies\PROMPT-1979
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

- `git merge-base --is-ancestor origin/main HEAD`: PASS
- `git diff --name-status origin/main..HEAD`:

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
A  tests/tools/autoplay/test_window_resize_verdict.py
M  tools/autoplay/analyze_evidence_run.py
M  tools/autoplay/validate_composite_run.py
```

All 13 files are within owned scope. No deletions (zero D-prefixed lines).

---

## C0 / Human Review Caveat (preserved from 1969)

The window-resize verdict downgrade to `NEEDS_HUMAN_GUI` is intentionally conservative.
Autoplay evidence affected by mid-run window resize or sub-600px height cannot be treated
as machine-verified PASS — an operator must inspect the screenshots manually to confirm
click targets landed on visible UI elements.

---

1979: AUTOPLAY-COMPOSITE-WINDOW-RESIZE-VERDICT-REFRESH-AFTER-1976: READY_FOR_MAINLAND_ENQUEUE
