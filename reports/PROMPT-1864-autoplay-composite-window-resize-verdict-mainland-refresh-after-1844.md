# PROMPT 1864 — Autoplay Composite Window-Resize Verdict Mainland Refresh After 1844

## Branch

`integrate/autoplay-composite-window-resize-verdict-1864`

## Base SHA

`bb90d7c2` (PROMPT 1844 — autoplay vs-bot viewport/click-target evidence audit)

## Source Commit

`21947388` on `origin/prompt-1850-composite-window-resize-verdict`

## FF Readiness from main

YES — branch is a direct descendant of `origin/main` @ `bb90d7c2`

## Problem

The `prompt-1850-composite-window-resize-verdict` branch diverged from main before
PROMPT 1844 landed. It cannot be FF-merged because cherry-picking the full branch
would delete `reports/PROMPT-1844-autoplay-vsbot-viewport-click-evidence-audit.md`.

## Solution

Created a fresh branch from current `origin/main` (`bb90d7c2`) and cherry-picked
ONLY the allowlisted 1850 files using `git checkout origin/prompt-1850 -- <file>`.

## Files Carried from 1850

| File | Change |
|------|--------|
| `tools/autoplay/analyze_evidence_run.py` | Modified — window size tracking + win32 quality |
| `tools/autoplay/validate_composite_run.py` | Modified — window/capture integrity guard |
| `tests/tools/autoplay/test_window_resize_verdict.py` | Added — 25 tests |
| `reports/PROMPT-1850-autoplay-composite-window-resize-verdict-downgrade.md` | Added — backfill (was missing from 1850 commit) |

## Validation Results

```
25 passed in 0.41s
```

All 25 tests in `tests/tools/autoplay/test_window_resize_verdict.py` pass.

`git diff --name-status origin/main..HEAD` shows only the 4 allowed files — no
deletion of PROMPT-1844 or PROMPT-1833 report files.

## Files NOT Touched

- `tools/autoplay/driver.py` — untouched
- `reports/PROMPT-1844-autoplay-vsbot-viewport-click-evidence-audit.md` — present and intact
- `reports/PROMPT-1833-autoplay-evidence-distinctness-analyzer.md` — present and intact
- All Bevy/Rust source files — untouched
- All production sprint/session state files — untouched

---

`1864: AUTOPLAY-COMPOSITE-WINDOW-RESIZE-VERDICT-MAINLAND-REFRESH-AFTER-1844: SHIPPED`
