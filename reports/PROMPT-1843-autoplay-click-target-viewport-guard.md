# PROMPT 1843 — Autoplay Click-Target Viewport Guard

**Date**: 2026-05-28  
**Branch**: `wt-1843-click-viewport-guard`  
**Status**: SHIPPED

## Problem

Autoplay was silently clicking blank/offscreen space when the game window was
smaller than expected (UI clipped off-screen). The driver dispatched cursor
positions without validating them against the live window dimensions, so
out-of-bounds clicks produced no diagnostic output.

## Root cause

`driver.py` read `window_logical_size` from the status response once at recipe
init time (to build the `RecipeContext`), but never checked that computed click
coordinates fell inside the visible client area before dispatching
`autoplay/input` actions. A window that was resized, minimized, or launched at
a smaller resolution than the recipe assumed would cause all recipe coordinates
to land outside the visible area.

## Changes

### `tools/autoplay/driver.py`

1. **`_validate_cursor_coords(x, y, window_size, tick, log_fn)`** — new helper
   (pure function, fully tested). Checks `0 ≤ x < w` and `0 ≤ y < h`. On
   failure emits a `WARNING CLICK-OOB` log line containing:
   - `cursor=(x,y)` — the exact pixel coordinate
   - `window=(WxH)` — the live logical window size
   - `frac=(fx,fy)` — fractional position (useful for diagnosing how far
     off-screen the click landed)
   - `x_clip` / `y_clip` labels — which axis(es) are out of range
   - Invalid window size (0×0 or negative) gets its own "invalid window_size"
     diagnostic rather than a misleading fraction.

2. **Per-tick `_tick_win_size` extraction** — after each `autoplay/status`
   response the driver now extracts `window_logical_size` into `_tick_win_size`
   as a `tuple[float, float]`. Falls back to `(0.0, 0.0)` (which triggers the
   "invalid" diagnostic path) when the field is absent.

3. **Guard call before dispatch** — in the `autoplay/input` action branch,
   just before the `rpc(url, method, params)` call, the driver validates the
   `cursor.screen` sub-field (if present) against `_tick_win_size`. The RPC
   still fires (warning-only, not a hard block) so existing recipes are not
   broken; the log line makes the problem visible without silently swallowing
   the click.

### `tests/tools/autoplay/test_driver_click_viewport_guard.py`

19 new tests in two classes:

| Class | Count | Coverage |
|-------|-------|----------|
| `TestValidateCursorCoords` | 13 | `_validate_cursor_coords` behavior: in-bounds, x-clip, y-clip, both-clip, boundary at (0,0) and (w,h), negative coords, zero/negative window size, log content (tick, dims, coords, fractions) |
| `TestDriverViewportGuardStructure` | 6 | Structural: function defined, `CLICK-OOB` sentinel present, call site exists, call gated on `autoplay/input`, `_tick_win_size` extraction present, `screen` field extracted |

## Test results

```
319 passed in 1.94s
```

All 319 autoplay tests pass (19 new + 300 pre-existing). No regressions.

## Design notes

- Guard is **warning-only** (not a hard block). This is intentional: a hard
  block would break existing smoke/vs-bot runs on dev machines that happen to
  have slightly different resolutions. The WARNING line in `driver.log` is
  grep-able (`grep CLICK-OOB driver.log`) and human-reviewers can act on it.
- PROMPT 1842 owns default window/viewport sizing. This guard is
  complementary: even after 1842 lands, the guard will catch future
  regressions if window size drifts again.
- `_validate_cursor_coords` is a pure function (no imports, no side effects
  beyond calling `log_fn`) so it is trivially unit-testable and can be reused
  from recipe code if needed later.

## Files changed

- `tools/autoplay/driver.py` — `_validate_cursor_coords` function + per-tick
  window extraction + dispatch-site guard
- `tests/tools/autoplay/test_driver_click_viewport_guard.py` — 19 new tests
- `reports/PROMPT-1843-autoplay-click-target-viewport-guard.md` — this report
