# PROMPT 1951 — Autoplay Composite Window-Resize Verdict Refresh After 1937

**Date:** 2026-05-28
**Branch:** integrate/autoplay-composite-window-resize-verdict-1951
**Base:** origin/main @ b58cdd660d726476f998f3a25ab810320a694fd1 (PROMPT 1937)
**Source branch:** origin/integrate/autoplay-composite-window-resize-verdict-1945-v2

---

## Context

PROMPT 1937 (docs(reports): QA snapshot observability gap report refresh after 1931) landed on main after the 1945-v2 integration branch was prepared. That made 1945-v2 NOT_FF and the orchestrator rejected it because a wholesale merge would have deleted the newly-landed QA snapshot reports (PROMPT 1839, 1867, 1900, 1937).

This task reapplies the window-resize verdict payload cleanly onto current origin/main (b58cdd66) using a file-level transplant from 1945-v2 — no wholesale cherry-pick, no stale deletions.

The user-observed blocker remains: too-small game windows can cause autoplay to click outside visible UI. All C0/human-review caveats in verdict docs and tool output are preserved.

---

## Files Changed

### Modified
- `tools/autoplay/analyze_evidence_run.py` — window size tracking, `_parse_driver_timeline()`, `NEEDS_HUMAN_GUI` verdict downgrade for resize/height/ALL_FROZEN
- `tools/autoplay/validate_composite_run.py` — `_check_window_and_capture_integrity()` guard, tags: `WINDOW-RESIZE-DETECTED`, `WINDOW-HEIGHT-TOO-SMALL`, `WIN32-ALL-FROZEN`

### Added (new)
- `tests/tools/autoplay/test_window_resize_verdict.py` — 25 pytest cases covering both modules
- `reports/PROMPT-1850-autoplay-composite-window-resize-verdict-downgrade.md`
- `reports/PROMPT-1864-autoplay-composite-window-resize-verdict-mainland-refresh-after-1844.md`
- `reports/PROMPT-1873-autoplay-composite-window-resize-verdict-refresh-after-1858.md`
- `reports/PROMPT-1875-autoplay-composite-window-resize-verdict-refresh-after-1872.md`
- `reports/PROMPT-1913-autoplay-composite-window-resize-verdict-refresh-after-1894.md`
- `reports/PROMPT-1918-autoplay-composite-window-resize-verdict-refresh-after-1912.md`
- `reports/PROMPT-1945-autoplay-composite-window-resize-verdict-refresh-after-1939.md`
- `reports/PROMPT-1951-autoplay-composite-window-resize-verdict-refresh-after-1937.md` (this file)

### Not touched (forbidden scope respected)
- `client/src/autoplay.rs` — untouched
- `tools/dev-launcher/**` — untouched
- `tools/autoplay/Run-AutoplaySmoke.ps1` — untouched
- `tests/tools/autoplay/test_driver_click_viewport_guard.py` — untouched
- `production/**` — untouched
- All QA snapshot reports (1839, 1867, 1900, 1937 and others) — preserved

---

## Transplant Method

File-level transplant from `origin/integrate/autoplay-composite-window-resize-verdict-1945-v2`:

```bash
git show origin/integrate/autoplay-composite-window-resize-verdict-1945-v2:tools/autoplay/analyze_evidence_run.py > tools/autoplay/analyze_evidence_run.py
git show origin/integrate/autoplay-composite-window-resize-verdict-1945-v2:tools/autoplay/validate_composite_run.py > tools/autoplay/validate_composite_run.py
git show origin/integrate/autoplay-composite-window-resize-verdict-1945-v2:tests/tools/autoplay/test_window_resize_verdict.py > tests/tools/autoplay/test_window_resize_verdict.py
# + 7 owned report files from 1945-v2 tree
```

No wholesale cherry-pick — avoids any stale deletes or reverts that were in the old branch ancestry.

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

### Diff scope check
```
git diff --name-status origin/main..HEAD
```
Expected output (10 files, all owned, no deletions):
```
A  reports/PROMPT-1850-autoplay-composite-window-resize-verdict-downgrade.md
A  reports/PROMPT-1864-autoplay-composite-window-resize-verdict-mainland-refresh-after-1844.md
A  reports/PROMPT-1873-autoplay-composite-window-resize-verdict-refresh-after-1858.md
A  reports/PROMPT-1875-autoplay-composite-window-resize-verdict-refresh-after-1872.md
A  reports/PROMPT-1913-autoplay-composite-window-resize-verdict-refresh-after-1894.md
A  reports/PROMPT-1918-autoplay-composite-window-resize-verdict-refresh-after-1912.md
A  reports/PROMPT-1945-autoplay-composite-window-resize-verdict-refresh-after-1939.md
A  reports/PROMPT-1951-autoplay-composite-window-resize-verdict-refresh-after-1937.md
A  tests/tools/autoplay/test_window_resize_verdict.py
M  tools/autoplay/analyze_evidence_run.py
M  tools/autoplay/validate_composite_run.py
```

### Whitespace check
```
git diff --check origin/main..HEAD
```
Result: PASS (no whitespace errors)

### Python tests
```
pytest tests/tools/autoplay/test_window_resize_verdict.py -v
```
**Result: 25/25 PASSED**

```
tests/tools/autoplay/test_window_resize_verdict.py::TestParseDriverTimeline::test_single_stable_size_no_resize_event PASSED
tests/tools/autoplay/test_window_resize_verdict.py::TestParseDriverTimeline::test_resize_event_counted PASSED
tests/tools/autoplay/test_window_resize_verdict.py::TestParseDriverTimeline::test_missing_timeline_adds_warning PASSED
tests/tools/autoplay/test_window_resize_verdict.py::TestParseDriverTimeline::test_empty_timeline_leaves_fields_none PASSED
tests/tools/autoplay/test_window_resize_verdict.py::TestWin32CaptureQuality::test_all_ok_is_good PASSED
tests/tools/autoplay/test_window_resize_verdict.py::TestWin32CaptureQuality::test_all_frozen_is_all_frozen PASSED
tests/tools/autoplay/test_window_resize_verdict.py::TestWin32CaptureQuality::test_mixed_is_partial_frozen PASSED
tests/tools/autoplay/test_window_resize_verdict.py::TestWin32CaptureQuality::test_no_win32_lines_is_unknown PASSED
tests/tools/autoplay/test_window_resize_verdict.py::TestAnalyzeVerdictDowngrade::test_window_resize_triggers_needs_human_gui PASSED
tests/tools/autoplay/test_window_resize_verdict.py::TestAnalyzeVerdictDowngrade::test_below_min_height_no_resize_triggers_needs_human_gui PASSED
tests/tools/autoplay/test_window_resize_verdict.py::TestAnalyzeVerdictDowngrade::test_all_frozen_win32_triggers_needs_human_gui PASSED
tests/tools/autoplay/test_window_resize_verdict.py::TestAnalyzeVerdictDowngrade::test_clean_run_is_not_downgraded PASSED
tests/tools/autoplay/test_window_resize_verdict.py::TestAnalyzeVerdictDowngrade::test_window_resize_into_acceptable_height_still_fails PASSED
tests/tools/autoplay/test_window_resize_verdict.py::TestAnalyzeVerdictDowngrade::test_blocked_outcome_takes_priority_over_window_resize PASSED
tests/tools/autoplay/test_window_resize_verdict.py::TestValidateWindowIntegrityGuard::test_window_resize_detected_fails PASSED
tests/tools/autoplay/test_window_resize_verdict.py::TestValidateWindowIntegrityGuard::test_window_height_too_small_fails PASSED
tests/tools/autoplay/test_window_resize_verdict.py::TestValidateWindowIntegrityGuard::test_win32_all_frozen_fails PASSED
tests/tools/autoplay/test_window_resize_verdict.py::TestValidateWindowIntegrityGuard::test_clean_run_no_window_failures PASSED
tests/tools/autoplay/test_window_resize_verdict.py::TestValidateWindowIntegrityGuard::test_missing_timeline_is_warning_not_failure PASSED
tests/tools/autoplay/test_window_resize_verdict.py::TestValidateWindowIntegrityGuard::test_missing_driver_log_is_warning_not_failure PASSED
tests/tools/autoplay/test_window_resize_verdict.py::TestValidateWindowIntegrityGuard::test_height_at_threshold_does_not_fail PASSED
tests/tools/autoplay/test_window_resize_verdict.py::TestValidateWindowIntegrityGuard::test_height_one_below_threshold_fails PASSED
tests/tools/autoplay/test_window_resize_verdict.py::TestValidateWindowIntegrityGuard::test_win32_mixed_frozen_ok_is_not_all_frozen PASSED
tests/tools/autoplay/test_window_resize_verdict.py::TestConstants::test_min_window_height_matches_between_modules PASSED
tests/tools/autoplay/test_window_resize_verdict.py::TestConstants::test_min_window_height_is_below_expected_resolution PASSED

25 passed in 0.43s
```

### FF readiness
Branch `integrate/autoplay-composite-window-resize-verdict-1951` is based directly on `origin/main@b58cdd66` (PROMPT 1937) with only additive commits — strict FF-ready.

---

## C0 / Human Review Caveat (preserved)

The window-resize verdict downgrade to `NEEDS_HUMAN_GUI` is intentionally conservative. Autoplay evidence affected by mid-run window resize or sub-600px height cannot be treated as machine-verified PASS — an operator must inspect the screenshots manually to confirm click targets landed on visible UI elements.

---

1951: AUTOPLAY-COMPOSITE-WINDOW-RESIZE-VERDICT-REFRESH-AFTER-1937: READY_FOR_MAINLAND_ENQUEUE
