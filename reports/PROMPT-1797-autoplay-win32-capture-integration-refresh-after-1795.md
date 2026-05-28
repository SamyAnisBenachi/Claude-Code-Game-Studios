# PROMPT 1797 — Autoplay Win32 Capture Integration Refresh After 1795

**Date:** 2026-05-28  
**Branch:** `prompt-1797-win32-capture-refresh`  
**Worktree:** `D:\_DEV\Work\Claude-Code-Game-Studios\tmpwt-1797-win32-capture-refresh`

## Summary

Integration refresh of PROMPT 1794 (Win32 GDI/PrintWindow capture backend) on top of
`origin/main@dd4d8a041c7ab1cd7cc829b919fe89ce179ca0da` (which includes PROMPT 1795
restoring the Bevy screenshot default to `primary_window`).

## Source Branch

`origin/prompt-1794-win32-capture` — was ahead 2, behind 2 vs origin/main after 1795.

## Commits Cherry-Picked

| Hash (original) | Hash (refreshed) | Message |
|---|---|---|
| `757698b4` | `1595b6c9` | feat(autoplay): PROMPT 1794 — Win32 GDI/PrintWindow capture backend for autoplay evidence |
| `942d399a` | `1b3e6287` | docs(reports): PROMPT 1794 — Win32 window capture backend report |

## Base Commit

`dd4d8a041c7ab1cd7cc829b919fe89ce179ca0da` — docs(reports): PROMPT 1795 — autoplay Bevy screenshot backend recovery report

## Refreshed Branch Tip

`1b3e6287` — docs(reports): PROMPT 1794 — Win32 window capture backend report

## Conflict Resolution

Both commits applied cleanly with no conflicts. `client/src/autoplay.rs` was not
touched — PROMPT 1795 behavior is preserved exactly.

## Files Changed (vs origin/main)

- `tools/autoplay/win_capture.py` — new Win32 GDI/PrintWindow capture module
- `tools/autoplay/driver.py` — wired win32 capture branch in screenshot logic
- `tests/tools/autoplay/test_win32_capture.py` — 28 tests covering capture backend
- `reports/PROMPT-1794-autoplay-win32-window-capture-backend-repair.md` — 1794 report

## Forbidden Files

`client/src/autoplay.rs` — **NOT modified** (verified via `git diff origin/main HEAD --name-only`).

## Validation

### Path Allowlist

All 4 changed files are within owned scope. No forbidden files touched.

### Whitespace Check

```
git diff --check HEAD → exit 0 (no whitespace errors)
```

### FF Readiness

```
git merge-base --is-ancestor origin/main HEAD → exit 0 (PASS)
```

### pytest

```
pytest tests/tools/autoplay/test_win32_capture.py -v
28 passed in 0.16s
```

All 28 tests pass: `is_available`, `write_png`, `capture_game_window`,
`capture_hwnd_to_png`, and `driver_win32_capture_wiring` suites.

## Push Status

Branch pushed to `origin/prompt-1797-win32-capture-refresh`. Strict-FF ready for MAINLAND_ENQUEUE.

---

1797: AUTOPLAY-WIN32-CAPTURE-INTEGRATION-REFRESH-AFTER-1795: SHIPPED
