# PROMPT 1786 — Autoplay Foreground Window Title Diagnostic Repair

**Date:** 2026-05-28
**Branch:** `fix/1786-foreground-window-title`
**Worktree:** `tmpwt-1786-foreground-window-title-repair`

---

## What PROMPT 1782 Proved

- `status.frame` advanced correctly across 15 checkpoints.
- All 15 screenshots were byte-identical and predominantly black.
- `win_foreground.py` was called before each screenshot and logged:
  > `foreground: no CCGS/Bevy window found among 21 visible top-level windows — screenshot will capture current foreground`
- This confirmed PROMPT 1781's foreground code was present and executing, but
  the window title discovery was silently failing every time.

---

## Root Cause

`_WINDOW_TITLE_HINTS` in `win_foreground.py` contained:

```python
("ccgs", "claude code game", "bevy app", "bevy")
```

The actual Bevy client window title is **"Lanes and Lies"**, set explicitly in
`client/src/main.rs`:

```rust
.set(WindowPlugin {
    primary_window: Some(Window {
        title: "Lanes and Lies".to_string(),
        ...
    }),
    ...
})
```

None of the prior hints contain "lanes" or "lies", so every `_find_candidate`
call returned `None` and the foreground call was skipped. The existing diagnostic
log did not emit the visible window titles, making the mismatch invisible without
a manual enumeration session.

---

## Matching Strategy Before vs After

| | Before (PROMPT 1776/1781) | After (PROMPT 1786) |
|---|---|---|
| Hints list | `("ccgs", "claude code game", "bevy app", "bevy")` | `("lanes and lies", "lanes", "ccgs", "claude code game", "bevy app", "bevy")` |
| Primary match | Would only succeed if binary used "ccgs" or "Bevy App" title | Matches the actual `"Lanes and Lies"` title from `client/src/main.rs` |
| Fallback | `"bevy"` substring (risky for IDE/tool windows with "bevy" in path) | Same `"bevy"` last-resort, now preceded by more-specific hints |
| No-match log | Count only: `"among 21 visible top-level windows"` | Count + hints list + bounded visible title dump (max 30 entries, 60-char truncation per title) |

---

## Changes Made

### `tools/autoplay/win_foreground.py`

1. **Added `"lanes and lies"` and `"lanes"` to `_WINDOW_TITLE_HINTS`** — covers
   the actual window title and a truncated fallback, placed before the legacy
   hints so the most-specific match is tried first.

2. **Added `_DIAG_TITLE_LIMIT = 30`** constant bounding diagnostic output.

3. **Added `_format_diag_titles(windows, limit)`** — pure Python helper that
   returns a bounded, truncated (60 chars/title) diagnostic string listing all
   visible window titles. No ctypes; directly unit-testable.

4. **Updated `ensure_foreground` no-match log** to include:
   - The active `_WINDOW_TITLE_HINTS` list
   - The `_format_diag_titles` output so future mismatches are diagnosable from
     `driver.log` alone, without a separate debug session.

### `tests/tools/autoplay/test_win_foreground.py`

Added 13 new tests across new `TestFindCandidate` entries and new `TestFormatDiagTitles` class and new `TestEnsureForeground` entries:

- `test_win_foreground_find_candidate_matches_lanes_and_lies_title` — regression guard
- `test_win_foreground_find_candidate_matches_lanes_and_lies_case_insensitive`
- `test_win_foreground_find_candidate_matches_lanes_substring`
- `test_win_foreground_hints_contains_lanes_and_lies`
- `TestFormatDiagTitles` (6 tests): empty list, single window, limit cap, no-suffix when under limit, long-title truncation, return type
- `test_win_foreground_ensure_logs_hints_when_no_match`
- `test_win_foreground_ensure_logs_visible_titles_when_no_match`
- `test_win_foreground_ensure_matches_lanes_and_lies_window` — end-to-end regression

---

## Test Results

```
38 passed in 0.07s
```

All 38 tests pass headlessly (no GUI, no live Bevy client, no Cargo).

---

## Will This Fix Screenshot Distinctness?

**Yes, if the window title is exactly "Lanes and Lies" at runtime.** The prior
failure was purely a title-matching miss. With the corrected hints list, the next
autoplay run should log `foreground: matched window title='Lanes and Lies'` and
call `SetForegroundWindow` / `ShowWindow` successfully.

If screenshots remain byte-identical black after this fix, the remaining causes
are:

1. The Bevy GPU backbuffer capture path (`Screenshot::primary_window()`) fires
   before the window compositor has actually composited the foregrounded frame.
   The `time.sleep(0.3)` added in PROMPT 1781 should cover typical DWM latency,
   but a higher value may be needed.
2. PROMPT 1784's offscreen/GPU-capture branch (`client/src/autoplay.rs`) may be
   the required fix for headless/offscreen scenarios where foreground alone is
   insufficient.

---

## Verify Command (after mainland)

```powershell
# 1. Start server and client normally
# 2. Run the autoplay smoke:
python tools/autoplay/driver.py smoke
# 3. Expect in driver.log:
#    foreground: matched window title='Lanes and Lies' hwnd=0x...
#    foreground: SetForegroundWindow OK hwnd=0x...
# 4. Screenshots should be non-identical (different game frames)
```

---

1786: AUTOPLAY-FOREGROUND-WINDOW-TITLE-DIAGNOSTIC-REPAIR: SHIPPED
