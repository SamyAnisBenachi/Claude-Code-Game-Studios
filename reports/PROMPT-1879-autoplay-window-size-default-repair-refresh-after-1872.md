# PROMPT 1879 — Autoplay Window-Size Default Repair Refresh After PROMPT 1872

**Date:** 2026-05-28  
**Branch:** `integrate/autoplay-window-size-default-1879`  
**Base:** `origin/main` @ `2ce3dc6b` (PROMPT 1872)  
**Status:** SHIPPED

---

## Context

PROMPT 1865 refreshed the AC-VPT-01 window-size default repair onto
`origin/main@bb90d7c2`, but `origin/main` now includes PROMPT 1845/1846/1858/1859/1872
report artifacts that `origin/integrate/autoplay-window-size-default-1865` does not have.
A FF merge of the 1865 branch would delete those reports.

This prompt creates a fresh worktree branch from the **latest** `origin/main` and
cherry-applies only the AC-VPT-01 owned-scope changes.

---

## Payload Applied

### `client/src/autoplay.rs`

1. **New constants** (after `DEFAULT_AUTOPLAY_PORT`):
   - `AUTOPLAY_WINDOW_WIDTH_ENV = "CCGS_WINDOW_WIDTH"`
   - `AUTOPLAY_WINDOW_HEIGHT_ENV = "CCGS_WINDOW_HEIGHT"`
   - `AUTOPLAY_MIN_WINDOW_W: f32 = 1280.0`
   - `AUTOPLAY_MIN_WINDOW_H: f32 = 720.0`

2. **`enforce_autoplay_window_size_system`** (new Startup system):
   - Reads `CCGS_WINDOW_WIDTH` / `CCGS_WINDOW_HEIGHT` from env (optional).
   - Falls back to `AUTOPLAY_MIN_WINDOW_W` / `AUTOPLAY_MIN_WINDOW_H` (1280×720).
   - Applies `max(current, target)` — never shrinks an already-larger window.
   - Registered via `app.add_systems(Startup, enforce_autoplay_window_size_system)` inside `AutoplayPlugin::build`.

3. **Unit test** `autoplay_window_size_constants_match_dev_floor`:
   - Asserts constants equal `SAFETY_VIEWPORT_DEV_FLOOR` values (1280.0, 720.0).
   - Asserts env-var name strings are correct.

### `tools/autoplay/Run-AutoplaySmoke.ps1`

Added window-size guard block (PROMPT 1842 label) after the vs-bot env gate, before the cargo build:

```powershell
if (-not $env:CCGS_WINDOW_WIDTH)  { $env:CCGS_WINDOW_WIDTH  = '1280' }
if (-not $env:CCGS_WINDOW_HEIGHT) { $env:CCGS_WINDOW_HEIGHT = '720'  }
Write-Host "[autoplay-smoke] viewport target: ..."
```

Defensive (Rust fallback already applies same floor), provides a log trail.

---

## Files NOT Touched

- `tools/autoplay/driver.py` — AC-VPT-02/08 blocking guards are separate work.
- `production/session-state/**`, `production/sprints/**`, sprint-status.yaml.
- PROMPT 1845/1846/1858/1859/1872 report files — all preserved on latest main.

---

## Validation

| Check | Result |
|---|---|
| `git diff --check` | PASS — no whitespace errors |
| PS1 parse (static review) | PASS — syntax is identical to 1865 branch payload |
| `git diff --name-status origin/main..HEAD` | `M client/src/autoplay.rs`, `M tools/autoplay/Run-AutoplaySmoke.ps1`, `A reports/PROMPT-1879-...md` only |
| PROMPT 1845/1846/1858/1859/1872 artifacts present | PASS — untouched on base main |
| No deletions of existing reports | PASS |

---

## Commit

Reapplied in PROMPT 1893 on `integrate/autoplay-window-size-default-1893` (fresh branch from `origin/main@c35750d8`).

---

1879: AUTOPLAY-WINDOW-SIZE-DEFAULT-REPAIR-REFRESH-AFTER-1872: SHIPPED
