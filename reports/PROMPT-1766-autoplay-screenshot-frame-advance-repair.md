# PROMPT 1766 — AUTOPLAY-SCREENSHOT-FRAME-ADVANCE-REPAIR

**Date:** 2026-05-28  
**Branch:** `prompt-1766-autoplay-screenshot-frame-advance`  
**Worktree:** `tmpwt-1766-screenshot-frame-advance`

---

## 1. Problem recap (from PROMPT 1763 audit)

All 15 PNG screenshots captured during PROMPT 1758's autoplay vs-bot smoke run
were byte-for-byte identical (MD5 `7F16A04F3C723EF6B2A448194C0517B4`, 85,865
bytes each).  The images contained real game content (dark navy lobby palette,
1280×720, 2 513 unique colours) but the pixel data never changed despite the game
advancing through 10 distinct phases over 21.5 seconds.

**Root cause (two layers)**

| Layer | Description |
|-------|-------------|
| R-1 (timing) | `RecipeBuilder.checkpoint()` emitted `autoplay/screenshot` exactly 1 driver tick (100 ms at 10 Hz) after the `local.checkpoint` action.  At that interval the Bevy renderer may not yet have composited the new game state onto the window surface. |
| R-2 (no guard) | The driver dispatched `autoplay/screenshot` with no verification that the Bevy `status.frame` counter had advanced since the previous screenshot.  If the render pipeline was throttled (window unfocused / not front) the same stale framebuffer was re-captured every time. |

---

## 2. Fix

### 2a. `tools/autoplay/recipes/_builder.py` — settle_ticks gap

Added `settle_ticks: int = 3` parameter to `RecipeBuilder.checkpoint()`.  When
`screenshot=True` (the default), the builder now inserts `settle_ticks` idle
driver ticks between the `local.checkpoint` emit and the `autoplay/screenshot`
emit.

```
Before (1 tick gap, ~100 ms):
  tick N   → local.checkpoint
  tick N+1 → autoplay/screenshot   ← too soon

After (default 4-tick gap, ~400 ms total):
  tick N   → local.checkpoint
  tick N+1 to N+3 → idle (settle)
  tick N+4 → autoplay/screenshot
```

At the default driver rate of 10 Hz, `settle_ticks=3` gives 300 ms of render
settle time before capture.  At 60 FPS this covers ~18 rendered frames — more
than sufficient for any game-state-to-visual pipeline delay.

`settle_ticks=0` restores the legacy immediate behaviour for callers that need it.

### 2b. `tools/autoplay/driver.py` — frame-advance barrier

Added `last_screenshot_frame: int = -1` tracking variable (initialised before
the tick loop).  Before dispatching any `autoplay/screenshot` RPC, the driver
now:

1. Reads `status["frame"]` from the current-tick status poll.
2. If `status["frame"] <= last_screenshot_frame` (no new frame rendered since
   the last capture), polls `autoplay/status` up to 5 more times with 50 ms
   sleep between attempts (total up to 250 ms extra wait).
3. If the frame counter still has not advanced after 5 retries, logs a
   `WARNING screenshot frame-advance barrier: frame stuck` message to
   `driver.log` and proceeds (non-blocking — does not abort the recipe).
4. Updates `last_screenshot_frame = current_frame` after dispatching.

Added `"frame"` field to checkpoint rows written to `checkpoints.jsonl` so
future auditors can correlate checkpoint events with Bevy render frames.

### 2c. `tests/tools/autoplay/test_driver_screenshot_barrier.py` — new test file

11 static/pure-Python tests (no game server, no Cargo):

| Class | Tests |
|-------|-------|
| `TestCheckpointSettleTicksGap` | default gap=4 ticks, zero reverts to 1, custom value, no screenshot, multiple checkpoints, reason label, settle ignored when screenshot=False |
| `TestDriverFrameAdvanceBarrierPresent` | structural checks that driver.py contains `last_screenshot_frame`, the screenshot branch, the stuck-frame warning, and `-1` initialisation |

---

## 3. Files modified

| File | Change |
|------|--------|
| `tools/autoplay/recipes/_builder.py` | Add `settle_ticks` param to `checkpoint()` |
| `tools/autoplay/driver.py` | `last_screenshot_frame` tracking + frame-advance barrier + `frame` field in checkpoint rows |
| `tests/tools/autoplay/test_driver_screenshot_barrier.py` | **New** — 11 static tests |

---

## 4. Test results

```
88 passed in 0.13s
  - 11 new tests in test_driver_screenshot_barrier.py (all PASS)
  - 77 pre-existing tests in test_recipe_static.py (all PASS, no regressions)
```

`git diff --check`: clean (no whitespace errors).

---

## 5. Known limitation / follow-up lane

If the Bevy window is minimised or the render pipeline is fully suspended
(`WinitSettings` unfocused mode), `status.frame` will not advance and the
barrier will log a warning but still capture a stale screenshot.  The
correct fix in that case is a Rust-side change: configure
`WinitSettings::game()` with `UpdateMode::Continuous` for both focused and
unfocused modes when `CCGS_AUTOPLAY=1`.  That change requires a `cargo build`
cycle and is deferred as a follow-up story.

---

1766: AUTOPLAY-SCREENSHOT-FRAME-ADVANCE-REPAIR: SHIPPED
