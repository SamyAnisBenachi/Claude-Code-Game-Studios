# PROMPT 1865 — Autoplay Window Size Default Repair: Main-Land Refresh After 1844

**Date:** 2026-05-28  
**Branch:** `integrate/autoplay-window-size-default-1865`  
**Source commit (1842 tip):** `a9f2e9d5bd7def908d2a2a6296bb29c622829abd`  
**Base:** `origin/main @ bb90d7c2`

---

## Summary

Refreshed the safe AC-VPT-01 portion of PROMPT 1842 (initial/default window size
enforcement) onto current main. PROMPT 1842 could not be fast-forwarded directly
because it pre-dates PROMPT 1833 (`analyze_evidence_run.py`) and PROMPT 1844
(viewport/click-target evidence audit report) — a direct FF would have deleted
those files. This refresh cherry-picks only the owned scope as a clean diff on top
of `bb90d7c2`.

---

## What Was Carried

### `client/src/autoplay.rs`

- **Constants block** (after `DEFAULT_AUTOPLAY_PORT`):
  - `AUTOPLAY_WINDOW_WIDTH_ENV = "CCGS_WINDOW_WIDTH"`
  - `AUTOPLAY_WINDOW_HEIGHT_ENV = "CCGS_WINDOW_HEIGHT"`
  - `AUTOPLAY_MIN_WINDOW_W = 1280.0`
  - `AUTOPLAY_MIN_WINDOW_H = 720.0`

- **Startup system registration** (inside `AutoplayPlugin::build`, after
  `WinitSettings::game()` insertion):
  ```rust
  app.add_systems(Startup, enforce_autoplay_window_size_system);
  ```
  Gated behind `AutoplayPlugin` (requires `autoplay-remote` feature +
  `CCGS_AUTOPLAY=1`).

- **`enforce_autoplay_window_size_system` function**: reads
  `CCGS_WINDOW_WIDTH` / `CCGS_WINDOW_HEIGHT` env vars (fallback: 1280×720
  floor), applies `max(current, target)` so larger windows are never shrunk.
  Uses `windows.single_mut()` with `Ok()` guard (Bevy 0.18 `Query::single_mut`
  returns `Result`).

- **Unit test** `autoplay_window_size_constants_match_dev_floor`: asserts that
  the min constants equal 1280×720 and env var names match spec.

### `tools/autoplay/Run-AutoplaySmoke.ps1`

- Window size guard block inserted before the `cargo build` step:
  ```powershell
  if (-not $env:CCGS_WINDOW_WIDTH)  { $env:CCGS_WINDOW_WIDTH  = '1280' }
  if (-not $env:CCGS_WINDOW_HEIGHT) { $env:CCGS_WINDOW_HEIGHT = '720'  }
  Write-Host "[autoplay-smoke] viewport target: ..."
  ```
  Defensive: if env vars are already set by the caller, the script does not
  override them.

---

## What Was Intentionally NOT Carried

### `tools/autoplay/driver.py`

PROMPT 1842 added a `WARNING` log when `window_logical_size` is observed below
1280×720. This change is **owned by PROMPT 1857** (driver.py blocking/downgrade
logic for AC-VPT-02/08). Carrying it here would create a merge conflict with
PROMPT 1857's in-flight work. Deferred per task scope.

---

## Validation

| Check | Result |
|-------|--------|
| `git merge-base --is-ancestor origin/main HEAD` | PASS — FF-ready |
| Changed files | `client/src/autoplay.rs`, `tools/autoplay/Run-AutoplaySmoke.ps1` only |
| `tools/autoplay/analyze_evidence_run.py` touched | NO (PROMPT 1833 file preserved) |
| `reports/PROMPT-1844-*.md` touched | NO (PROMPT 1844 file preserved) |
| `tools/autoplay/driver.py` touched | NO (PROMPT 1857 boundary respected) |
| `git diff --check` whitespace lint | PASS |
| Path allowlist review | Only owned scope modified |

---

## FF Readiness

Branch `integrate/autoplay-window-size-default-1865` is fast-forward mergeable
from `origin/main`. No driver.py conflict surface. PROMPT 1857 can land
independently on its own branch after this merges.

---

1865: AUTOPLAY-WINDOW-SIZE-DEFAULT-REPAIR-MAINLAND-REFRESH-AFTER-1844: SHIPPED
