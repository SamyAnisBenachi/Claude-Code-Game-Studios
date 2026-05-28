# PROMPT 1912 — AUTOPLAY-WINDOW-SIZE-DEFAULT-REPAIR-REFRESH-AFTER-1894

**Date:** 2026-05-28
**Branch:** `integrate/autoplay-window-size-default-1912`
**Base:** `origin/main` @ `71484fc4` (PROMPT 1894)
**Final commit:** `e02d132f797cce6fdd1cd7787a2a4f90145f1569`
**Status:** SHIPPED

---

## Summary

Refreshed the PROMPT 1893 AC-VPT-01 window-size default repair payload onto
`origin/main` after PROMPT 1894 landed. The original `integrate/autoplay-window-size-default-1893`
branch was NOT_FF against current main because PROMPT 1894 had since added the
viewport guard payload (PROMPT-1880/1894 reports, `test_driver_click_viewport_guard.py`,
`driver.py` edits). A direct merge would have deleted those files.

This worker applied only the PROMPT 1893 payload files in a dedicated worktree,
skipping all forbidden files, resulting in a strict-FF-ready branch.

---

## Source Branch

- **Source:** `origin/integrate/autoplay-window-size-default-1893`
- **Source commit:** `6e4438698ce18f5ebe63f6370ce6e3e6135ebe61`
- **Source base:** `origin/main` @ `c35750d8` (PROMPT 1856)
- **NOT_FF against current main:** yes — 1894 had landed after 1893 was authored

---

## Files Changed

| File | Change | Notes |
|------|--------|-------|
| `client/src/autoplay.rs` | Modified | AC-VPT-01 constants + `enforce_autoplay_window_size_system` Startup system + test |
| `tools/autoplay/Run-AutoplaySmoke.ps1` | Modified | Default `CCGS_WINDOW_WIDTH/HEIGHT` to 1280x720 + log line |
| `reports/PROMPT-1879-autoplay-window-size-default-repair-refresh-after-1872.md` | Added | Carried from 1893 payload |
| `reports/PROMPT-1893-autoplay-window-size-default-repair-refresh-after-1856-1876.md` | Added | Carried from 1893 payload |

**No deletes.** `tools/autoplay/driver.py`, `tests/tools/autoplay/test_driver_click_viewport_guard.py`,
`tools/dev-launcher/**`, `reports/PROMPT-1880-*.md`, `reports/PROMPT-1894-*.md` untouched.

---

## Validation

### Path allowlist review

```
git diff --name-status origin/main..HEAD
M   client/src/autoplay.rs
A   reports/PROMPT-1879-autoplay-window-size-default-repair-refresh-after-1872.md
A   reports/PROMPT-1893-autoplay-window-size-default-repair-refresh-after-1856-1876.md
M   tools/autoplay/Run-AutoplaySmoke.ps1
```

Result: **PASS** — only allowed-scope files, no deletes.

### FF status

```
git merge-base --is-ancestor origin/main HEAD
```

Result: **FF-READY** — `origin/main` is ancestor of HEAD.

### git diff --check

Trailing whitespace flagged only in the markdown report files (standard markdown
`  ` line-break syntax from the 1893 source). No whitespace issues in `.rs` or `.ps1`.

### PowerShell static parse check

```powershell
[System.Management.Automation.Language.Parser]::ParseFile(
    'tools/autoplay/Run-AutoplaySmoke.ps1', [ref]$null, [ref]$null
)
```

Result: **PS1 parse: PASS**

---

## AC-VPT-01 Payload Description

### `client/src/autoplay.rs`

Added:
- `AUTOPLAY_WINDOW_WIDTH_ENV` = `"CCGS_WINDOW_WIDTH"`
- `AUTOPLAY_WINDOW_HEIGHT_ENV` = `"CCGS_WINDOW_HEIGHT"`
- `AUTOPLAY_MIN_WINDOW_W: f32` = `1280.0`
- `AUTOPLAY_MIN_WINDOW_H: f32` = `720.0`
- `enforce_autoplay_window_size_system` — Startup system registered in `AutoplayPlugin::build`; reads env vars, applies `max(current, target)` to `PrimaryWindow.resolution`
- `autoplay_window_size_constants_match_dev_floor` test

### `tools/autoplay/Run-AutoplaySmoke.ps1`

Added block (before cargo build):
```powershell
if (-not $env:CCGS_WINDOW_WIDTH)  { $env:CCGS_WINDOW_WIDTH  = '1280' }
if (-not $env:CCGS_WINDOW_HEIGHT) { $env:CCGS_WINDOW_HEIGHT = '720'  }
Write-Host "[autoplay-smoke] viewport target: ..."
```

---

1912: AUTOPLAY-WINDOW-SIZE-DEFAULT-REPAIR-REFRESH-AFTER-1894: SHIPPED
