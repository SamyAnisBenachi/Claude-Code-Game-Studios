# PROMPT 1894 — AUTOPLAY-CLICK-TARGET-VIEWPORT-GUARD-REFRESH-AFTER-1856-1876

**Date:** 2026-05-28
**Branch:** `integrate/autoplay-click-viewport-guard-1894`
**Base:** `origin/main` @ `c35750d8`
**Source branch:** `origin/integrate/autoplay-click-viewport-guard-1880` @ `4dfdb28c`

## Summary

PROMPT 1880 shipped branch `integrate/autoplay-click-viewport-guard-1880` onto
`origin/main@2ce3dc6b`, but that base became stale after PROMPT 1876
(`674ba870`) and PROMPT 1856 (`c35750d8`) landed. A direct FF of the 1880 branch
onto current main would have deleted `reports/PROMPT-1856-ui-1280x720-layout-smoke-slice-f.md`,
deleted `reports/PROMPT-1876-dev-launcher-autoplay-evidence-ux-refresh-after-1872.md`,
and reverted `tools/dev-launcher/Start-AutoplayVsBot.ps1` changes.

This task cherry-picks the 1880 payload commit cleanly onto current
`origin/main@c35750d8`, producing a new branch that is strict-FF-ready without
touching any forbidden files.

## Files Changed vs origin/main

| Status | File |
|--------|------|
| A | reports/PROMPT-1880-autoplay-click-target-viewport-guard-refresh-after-1872.md |
| A | tests/tools/autoplay/test_driver_click_viewport_guard.py |
| M | tools/autoplay/driver.py |

**No deletions. No changes to:**
- `reports/PROMPT-1856-ui-1280x720-layout-smoke-slice-f.md` — NOT touched
- `reports/PROMPT-1876-dev-launcher-autoplay-evidence-ux-refresh-after-1872.md` — NOT touched
- `tools/dev-launcher/Start-AutoplayVsBot.ps1` — NOT touched

## Payload (from 1880 cherry-pick)

- `EXIT_VIEWPORT_GUARD = 5` exit code constant
- Pre-build minimum window size check (`check_window_minimum`)
- Mid-run drift check — `check_window_drift` (AC-VPT-02)
- Post-foreground shrink abort (AC-VPT-08)
- `cursor_logical=None` abort
- Out-of-bounds click-target abort (`validate_cursor_coords`)
- `parse_window_size` helper
- Checkpoint emissions: `viewport_drift`, `viewport_shrink_abort`, `cursor_none`, `click_oob`

## Validation

### Path allowlist review
```
git diff --name-status origin/main HEAD
A  reports/PROMPT-1880-autoplay-click-target-viewport-guard-refresh-after-1872.md
A  tests/tools/autoplay/test_driver_click_viewport_guard.py
M  tools/autoplay/driver.py
```
PASS — no forbidden files touched.

### git diff --check
Trailing whitespace in markdown report only (intentional MD line-break spaces).
No whitespace issues in Python source files.

### FF-readiness
```
git merge-base --is-ancestor origin/main HEAD
→ FF-READY: origin/main is ancestor
```

### pytest — focused click viewport guard
```
pytest tests/tools/autoplay/test_driver_click_viewport_guard.py -v
66 passed in 0.16s
```
All 66 tests pass.

## Branch / Commit

| Field | Value |
|-------|-------|
| Branch | `integrate/autoplay-click-viewport-guard-1894` |
| Tip commit | `e8a40f81` |
| Base (origin/main) | `c35750d8` |
| FF status | FF-ready |
| Source cherry-pick | `4dfdb28c` (PROMPT 1880) |

---

1894: AUTOPLAY-CLICK-TARGET-VIEWPORT-GUARD-REFRESH-AFTER-1856-1876: SHIPPED
