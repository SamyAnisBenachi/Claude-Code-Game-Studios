# PROMPT 1844 — Autoplay Window-Drift / Pre-Click Safety Gate

**Date**: 2026-05-28  
**Branch**: `wt-1844-window-drift-guard`  
**Status**: SHIPPED

## Exact failure mode addressed

Run `090613`: recipe built at `window_logical_size=(1280, 720)`. At tick 115
the window was resized to `(1280, 1076)`. The driver continued dispatching
`mouse_down`/`mouse_up` using 720-baked recipe coordinates (e.g. `y=0.85*720 =
612`) which now pointed to y=612 in a 1076-tall window — well below the actual
UI buttons. The run completed without error and was marked clean PASS.

## Root cause

The existing driver (post-PROMPT-1843) validated cursor coordinates on every
`autoplay/input cursor` move, but did so **warning-only** and **never checked
against the size the recipe was built with**. There was no guard on
`mouse_down`/`mouse_up` and no mechanism to abort when the window had been
resized since recipe initialization.

## Changes

### `tools/autoplay/driver.py`

**New exit code**
```python
EXIT_WINDOW_MISMATCH = 5  # window drifted from recipe build size
```
Documented in the module docstring. Distinct from EXIT_BLOCKED=4. Any run that
exits with code 5 must NOT be marked clean PASS.

**`_check_window_stable_for_mouseclick(current_win, build_win, cursor_logical, last_cursor_screen, tick, log_fn, drift_px=10, min_height=720)`**

Pure function, four invariants checked in order:
1. **Size drift** — `|current_w - build_w| <= 10` AND `|current_h - build_h| <= 10`.
   The run-090613 case: drift = 356 px height → immediate ABORT WINDOW-DRIFT.
2. **Minimum height** — `current_h >= 720`. Catches a window smaller than the
   minimum required for the UI to be fully visible.
3. **cursor_logical not None** — `status.cursor_logical` must be present;
   None means the window has lost focus and cursor tracking has stopped.
4. **Last cursor OOB** — the most recent `cursor.screen` position must lie
   within the current window bounds. Guards the case where a cursor move landed
   OOB and was followed immediately by mouse_down.

On any violation: logs `ABORT <SENTINEL>` with build/current sizes and tick,
emits a `NEEDS_HUMAN_GUI` checkpoint row to `checkpoints.jsonl`, sets
`rc = EXIT_WINDOW_MISMATCH`, breaks out of both the action loop and the tick
loop.

**State tracking added to `main()`**
- `recipe_build_win_size` — set once when the recipe is first built; `None`
  before that so the guard is a no-op on the first tick.
- `last_cursor_screen` — updated every time a `cursor.screen` position is
  dispatched (reuses the PROMPT-1843 cursor-move branch).

**Guard invocation** — in the `autoplay/input` action branch, after the PROMPT-
1843 OOB warning, fires before `rpc()` when `"mouse_down" in params or
"mouse_up" in params`.

### `tests/tools/autoplay/test_driver_window_drift_guard.py`

25 new tests in three classes:

| Class | Count | Coverage |
|-------|-------|----------|
| `TestCheckWindowStableForMouseclick` | 16 | All four invariants; boundary values (10 px pass, 11 px fail; h=720 pass, h=719 fail); None last_cursor; drift-before-cursor-none priority; log content |
| `TestExitWindowMismatch` | 1 | EXIT_WINDOW_MISMATCH == 5 |
| `TestDriverWindowDriftGuardStructure` | 8 | Structural: function defined, exit const, NEEDS_HUMAN_GUI sentinel, mouse_down/mouse_up gate, recipe_build_win_size tracked, call site, tick-loop break, checkpoint emit order |

## Test results

```
344 passed in 1.88s
```

All 344 autoplay tests pass (25 new + 319 inherited from PROMPT-1843 + earlier).

## Verification: run-090613 scenario is now caught

With this guard, a driver run where tick 115 reports
`window_logical_size=[1280, 1076]` while `recipe_build_win_size=(1280, 720)`:
- drift_h = 356 > 10 → `_check_window_stable_for_mouseclick` returns `(False, "ABORT WINDOW-DRIFT ...")`
- `rc = EXIT_WINDOW_MISMATCH = 5`
- `checkpoints.jsonl` contains `{"kind": "NEEDS_HUMAN_GUI", ...}`
- Driver exits nonzero → run cannot be marked clean PASS

## Files changed

- `tools/autoplay/driver.py` — `EXIT_WINDOW_MISMATCH`, `_check_window_stable_for_mouseclick`, `recipe_build_win_size`, `last_cursor_screen` tracking, guard at mouse_down/mouse_up, tick-loop break
- `tests/tools/autoplay/test_driver_window_drift_guard.py` — 25 new tests
- `reports/PROMPT-1844-autoplay-click-target-viewport-guard.md` — this report
