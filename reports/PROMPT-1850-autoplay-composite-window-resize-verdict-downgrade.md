# PROMPT 1850 — Autoplay Composite Window-Resize Verdict Downgrade

## Summary

Added composite-level guard that prevents vs-bot runs with mid-run window resize,
below-minimum window height, or all-frozen PrintWindow captures from being reported
as clean PASS. This backfill report was added during PROMPT 1864 mainland refresh,
as the original 1850 commit omitted the report file.

## Source Commit

`21947388` on `origin/prompt-1850-composite-window-resize-verdict`

## Changes Shipped

### `tools/autoplay/analyze_evidence_run.py`
- Parse `driver-timeline.jsonl` for `window_logical_size` per tick
- Track `initial_window_size`, `final_window_size`, `window_resize_event_count`,
  `min_window_height_seen` (MIN_WINDOW_HEIGHT=600px threshold)
- Count win32 PrintWindow OK vs FAILED/FROZEN lines in `driver.log`
- Expose `win32_capture_quality`: GOOD / PARTIAL_FROZEN / ALL_FROZEN / UNKNOWN
- Downgrade verdict to `NEEDS_HUMAN_GUI` for: mid-run resize, below-min height,
  ALL_FROZEN capture quality
- Include new fields in both human and JSON output formats

### `tools/autoplay/validate_composite_run.py`
- Add `_check_window_and_capture_integrity()` reading `driver-timeline.jsonl`
  and `driver.log` from the artifact dir
- FAIL with `WINDOW-RESIZE-DETECTED` if `window_logical_size` changes during run
- FAIL with `WINDOW-HEIGHT-TOO-SMALL` if height < MIN_WINDOW_HEIGHT throughout
- FAIL with `WIN32-ALL-FROZEN` if all win32 PrintWindow attempts failed/frozen
- Missing files emit warnings, not failures

### `tests/tools/autoplay/test_window_resize_verdict.py` (new)
- 25 focused tests covering all new guard conditions and edge cases
- All 25 pass

## Validation

- 25/25 tests pass (`pytest tests/tools/autoplay/test_window_resize_verdict.py -v`)
- No forbidden files touched
- No deletion of PROMPT-1844 or PROMPT-1833 report files

## Status

`1850: AUTOPLAY-COMPOSITE-WINDOW-RESIZE-VERDICT-DOWNGRADE: BACKFILL-REPORTED`
