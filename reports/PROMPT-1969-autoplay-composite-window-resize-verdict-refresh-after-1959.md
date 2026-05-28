# PROMPT 1969 — Autoplay Composite Window-Resize Verdict Refresh After 1959

**Date:** 2026-05-28
**Branch:** integrate/autoplay-composite-window-resize-verdict-1969
**Base:** origin/main @ 7fc1706e (PROMPT 1959)
**Source branch (file transplant origin):** origin/integrate/autoplay-composite-window-resize-verdict-1951 @ ae103ef5

---

## Context

PROMPT 1951 prepared `integrate/autoplay-composite-window-resize-verdict-1951` based on
origin/main@b58cdd66 (PROMPT 1937). Multiple PROMPTs (1959, 1957, 1920) landed on main
after that, making 1951 NOT_FF and causing the orchestrator to reject a wholesale merge
(it would have deleted PROMPT 1920 card-inspect changes, PROMPT 1957 auction tier-border
work, and PROMPT 1959 Krosmaga UI Stage3 reports).

This task reapplies the window-resize verdict payload cleanly onto current origin/main
(7fc1706e) using a file-level transplant — no wholesale cherry-pick, no stale deletions.

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
- `reports/PROMPT-1969-autoplay-composite-window-resize-verdict-refresh-after-1959.md` (this file)

### Not touched (forbidden scope respected)
- `client/**` — untouched
- `server/**` — untouched
- `Cargo.*` — untouched
- `production/**` — untouched
- PROMPT 1920 card-inspect reports — preserved
- PROMPT 1957 auction tier-border test/report — preserved
- PROMPT 1959 Krosmaga UI Stage3 reports — preserved
- All QA snapshot reports — preserved

---

## Transplant Method

File-level transplant from `origin/integrate/autoplay-composite-window-resize-verdict-1951`:

```bash
git show origin/integrate/autoplay-composite-window-resize-verdict-1951:<file> > <worktree>/<file>
```

Applied for all 11 owned files. No wholesale cherry-pick — avoids stale deletions in
1951 branch ancestry.

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

## Validation

### FF check
```
git merge-base --is-ancestor origin/main HEAD
```
Result: **PASS** — branch is strict-FF over origin/main@7fc1706e

### Diff scope check
```
git diff --name-status origin/main..HEAD
```
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
A  tests/tools/autoplay/test_window_resize_verdict.py
M  tools/autoplay/analyze_evidence_run.py
M  tools/autoplay/validate_composite_run.py
```
All 12 files are within owned scope. No deletions.

### Whitespace check
```
git diff --check origin/main..HEAD
```
Result: **PASS**

### Python tests
```
pytest tests/tools/autoplay/test_window_resize_verdict.py -v
```
**Result: 25/25 PASSED in 0.35s**

```
TestParseDriverTimeline::test_single_stable_size_no_resize_event PASSED
TestParseDriverTimeline::test_resize_event_counted PASSED
TestParseDriverTimeline::test_missing_timeline_adds_warning PASSED
TestParseDriverTimeline::test_empty_timeline_leaves_fields_none PASSED
TestWin32CaptureQuality::test_all_ok_is_good PASSED
TestWin32CaptureQuality::test_all_frozen_is_all_frozen PASSED
TestWin32CaptureQuality::test_mixed_is_partial_frozen PASSED
TestWin32CaptureQuality::test_no_win32_lines_is_unknown PASSED
TestAnalyzeVerdictDowngrade::test_window_resize_triggers_needs_human_gui PASSED
TestAnalyzeVerdictDowngrade::test_below_min_height_no_resize_triggers_needs_human_gui PASSED
TestAnalyzeVerdictDowngrade::test_all_frozen_win32_triggers_needs_human_gui PASSED
TestAnalyzeVerdictDowngrade::test_clean_run_is_not_downgraded PASSED
TestAnalyzeVerdictDowngrade::test_window_resize_into_acceptable_height_still_fails PASSED
TestAnalyzeVerdictDowngrade::test_blocked_outcome_takes_priority_over_window_resize PASSED
TestValidateWindowIntegrityGuard::test_window_resize_detected_fails PASSED
TestValidateWindowIntegrityGuard::test_window_height_too_small_fails PASSED
TestValidateWindowIntegrityGuard::test_win32_all_frozen_fails PASSED
TestValidateWindowIntegrityGuard::test_clean_run_no_window_failures PASSED
TestValidateWindowIntegrityGuard::test_missing_timeline_is_warning_not_failure PASSED
TestValidateWindowIntegrityGuard::test_missing_driver_log_is_warning_not_failure PASSED
TestValidateWindowIntegrityGuard::test_height_at_threshold_does_not_fail PASSED
TestValidateWindowIntegrityGuard::test_height_one_below_threshold_fails PASSED
TestValidateWindowIntegrityGuard::test_win32_mixed_frozen_ok_is_not_all_frozen PASSED
TestConstants::test_min_window_height_matches_between_modules PASSED
TestConstants::test_min_window_height_is_below_expected_resolution PASSED
```

---

## C0 / Human Review Caveat (preserved)

The window-resize verdict downgrade to `NEEDS_HUMAN_GUI` is intentionally conservative.
Autoplay evidence affected by mid-run window resize or sub-600px height cannot be treated
as machine-verified PASS — an operator must inspect the screenshots manually to confirm
click targets landed on visible UI elements.

---

1969: AUTOPLAY-COMPOSITE-WINDOW-RESIZE-VERDICT-REFRESH-AFTER-1959: READY_FOR_MAINLAND_ENQUEUE
