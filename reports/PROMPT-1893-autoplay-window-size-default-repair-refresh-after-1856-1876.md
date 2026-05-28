# PROMPT 1893 â€” Autoplay Window-Size Default Repair Refresh After PROMPT 1856/1876

**Date:** 2026-05-28
**Branch:** `integrate/autoplay-window-size-default-1893`
**Base:** `origin/main` @ `c35750d8` (PROMPT 1856)
**Status:** SHIPPED

---

## Context

PROMPT 1879 shipped AC-VPT-01 on `origin/integrate/autoplay-window-size-default-1879`,
but that branch diverged before PROMPT 1876 and PROMPT 1856 landed on main. A direct
FF merge would delete:
- `reports/PROMPT-1856-ui-1280x720-layout-smoke-slice-f.md`
- `reports/PROMPT-1876-dev-launcher-autoplay-evidence-ux-refresh-after-1872.md`
- Revert `tools/dev-launcher/Start-AutoplayVsBot.ps1` section 10 (PROMPT 1876)

This prompt creates a **fresh worktree** from `origin/main@c35750d8` and reapplies
only the PROMPT 1879 owned-scope payload without touching the 1856/1876 artifacts.

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
   - Falls back to `AUTOPLAY_MIN_WINDOW_W` / `AUTOPLAY_MIN_WINDOW_H` (1280Ã—720).
   - Applies `max(current, target)` â€” never shrinks an already-larger window.
   - Registered via `app.add_systems(Startup, enforce_autoplay_window_size_system)` inside `AutoplayPlugin::build`.

3. **Unit test** `autoplay_window_size_constants_match_dev_floor`:
   - Asserts constants equal `SAFETY_VIEWPORT_DEV_FLOOR` values (1280.0, 720.0).
   - Asserts env-var name strings are correct.

### `tools/autoplay/Run-AutoplaySmoke.ps1`

Added window-size guard block after the vs-bot env gate, before the cargo build:

```powershell
if (-not $env:CCGS_WINDOW_WIDTH)  { $env:CCGS_WINDOW_WIDTH  = '1280' }
if (-not $env:CCGS_WINDOW_HEIGHT) { $env:CCGS_WINDOW_HEIGHT = '720'  }
Write-Host "[autoplay-smoke] viewport target: $($env:CCGS_WINDOW_WIDTH)x$($env:CCGS_WINDOW_HEIGHT) (CCGS_WINDOW_WIDTH/CCGS_WINDOW_HEIGHT)"
```

### Reports

- `reports/PROMPT-1879-autoplay-window-size-default-repair-refresh-after-1872.md` â€” backfilled
- `reports/PROMPT-1893-autoplay-window-size-default-repair-refresh-after-1856-1876.md` â€” this report

---

## Files NOT Touched

- `tools/dev-launcher/Start-AutoplayVsBot.ps1` â€” FORBIDDEN per scope
- `reports/PROMPT-1856-ui-1280x720-layout-smoke-slice-f.md` â€” preserved (main)
- `reports/PROMPT-1876-dev-launcher-autoplay-evidence-ux-refresh-after-1872.md` â€” preserved (main)
- `production/session-state/**`, `production/sprints/**`, sprint-status.yaml

---

## Validation

| Check | Result |
|---|---|
| Worktree created from `origin/main@c35750d8` | PASS |
| `git diff --check` | PASS â€” no whitespace errors |
| `git diff --name-status origin/main..HEAD` | Only M/A on owned scope files â€” no deletions |
| PROMPT 1856 report present | PASS â€” untouched |
| PROMPT 1876 report present | PASS â€” untouched |
| `Start-AutoplayVsBot.ps1` not touched | PASS |
| Constants match 1879 branch payload exactly | PASS |

---

1893: AUTOPLAY-WINDOW-SIZE-DEFAULT-REPAIR-REFRESH-AFTER-1856-1876: SHIPPED
