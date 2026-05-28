# PROMPT 1848 — Autoplay Recipe Visible-Target Coverage Map

**Date**: 2026-05-28
**Branch**: `prompt-1848-autoplay-recipe-visible-target-coverage-map`
**Base commit**: `b856eef4` (PROMPT 1833 main-land)
**Scope**: Read-only audit — `tools/autoplay/recipes/`, `tools/autoplay/driver.py`, evidence logs
**Related workers**: 1842 (window size repair), 1843 (click-target viewport guard), 1844 (forensic audit)

---

## 1. Architecture Summary

### How coordinates are resolved

All interactive recipes use **fractional logical-window coordinates** (`fx, fy` in `[0.0, 1.0]`)
from `tools/autoplay/recipes/_coords.py`. At recipe build time, the driver multiplies fractions
against `window_logical_size` reported by the first `autoplay/status` RPC call (tick 1).

```
pixel_x = fx * window_logical_size[0]
pixel_y = fy * window_logical_size[1]
```

The driver reads `window_logical_size` **only once** at recipe-build time (driver.py line 226–229).
If the Bevy window is resized during a run, all subsequent coordinate math is stale.

### What `autoplay/status` exposes (schema v2)

```json
{
  "window_logical_size": [1280, 720],
  "frame": 2,
  "phase_label": "Lobby",
  "client_state_label": "Lobby",
  "cursor_logical": null,
  "keys_pressed": [], "mouse_pressed": []
}
```

`phase_label` and `client_state_label` are available but **no recipe reads them for gating**.
Recipes click unconditionally based on `wait(N)` tick counts alone.

### Evidence window sizes observed

| Evidence run | Logical size (from status) | Win32 capture size | Delta |
|---|---|---|---|
| 20260528-090613-Z | 1280 × 720 | 1296 × 759 | +16px W, +39px H (OS chrome) |

The Bevy game content renders at 1280×720 logical. The OS window adds ≈39px for the title bar
and borders, so win32 captures are 1296×759. All click fractions are applied to the 1280×720
logical space — this is correct for the RPC layer. The concern is what happens when the
**logical size is smaller than 1280×720** (too-small game window scenario reported by human).

---

## 2. Per-Recipe Visible-Target Map

### Legend
- **Mechanism**: `frac` = fractional coordinate click; `drag` = multi-step cursor drag; `passive` = no input
- **Visibility proof**: `checkpoint-screenshot` = screenshot taken but not checked by code; `none` = no capture
- **Fragility**: `LOW` / `MEDIUM` / `HIGH` relative risk at 1280×720 or smaller

---

### `smoke`
| Step | UI Element | Mechanism | Default coords (fx, fy) | Pixel @1280×720 | Visibility Proof | Fragility |
|---|---|---|---|---|---|---|
| 1 | Window center | `frac(0.5, 0.5)` | (0.5, 0.5) | (640, 360) | screenshot after | LOW |

**Notes**: No game UI element targeted. Proves RPC substrate only. No semantic dependency.

---

### `lobby-create`
| Step | UI Element | Mechanism | Default (fx, fy) | Pixel @1280×720 | Env override | Visibility Proof | Fragility |
|---|---|---|---|---|---|---|---|
| 1 | Lobby loaded | checkpoint | — | — | — | screenshot | — |
| 2 | Create button | `frac` click | (0.5, 0.55) | (640, 396) | `CCGS_AUTOPLAY_LOBBY_CREATE_BTN` | none before click | MEDIUM |
| 3 | Confirm button | `frac` click | (0.5, 0.85) | (640, **612**) | `CCGS_AUTOPLAY_LOBBY_CONFIRM_BTN` | none before click | **MEDIUM-HIGH** |
| 4 | Lobby confirmed | checkpoint | — | — | — | screenshot | — |

**Notes**: 8-tick (`wait(8)` = 0.8s at 10Hz) settlement between Create and Confirm assumes the
confirm panel mounts within 0.8s. The Confirm button at y=612 is 108px from the bottom of a 720px
window — inside the lower action panel. At window heights below ~680px the button could be
partially clipped by OS taskbar overlap if the game window is positioned at the screen bottom.

---

### `add-bot-lobby`
| Step | UI Element | Mechanism | Default (fx, fy) | Pixel @1280×720 | Env override | Visibility Proof | Fragility |
|---|---|---|---|---|---|---|---|
| 0 | `CCGS_DEBUG_UI=1` gate | env check | — | — | — | `local.block` if absent | — |
| 1 | Lobby loaded | checkpoint | — | — | — | screenshot | — |
| 2 | Create button | `frac` click | (0.5, 0.55) | (640, 396) | `CCGS_AUTOPLAY_LOBBY_CREATE_BTN` | none | MEDIUM |
| 3 | Add Bot button | `frac` click | (0.5, 0.72) | (640, **518**) | `CCGS_AUTOPLAY_LOBBY_ADD_BOT_BTN` | none before click | **HIGH** |
| 4 | Bot seated | checkpoint | — | — | — | screenshot | — |
| 5 | Confirm button | `frac` click | (0.5, 0.85) | (640, **612**) | `CCGS_AUTOPLAY_LOBBY_CONFIRM_BTN` | none | MEDIUM-HIGH |
| 6 | Lobby confirmed | checkpoint | — | — | — | screenshot | — |

**Notes**: The Add Bot button (fy=0.72) is a **debug-only UI element** guarded by `CCGS_DEBUG_UI=1`.
If the button layout changes between debug and release builds, or if the debug panel renders at a
different vertical position, the click lands in blank space with no detection. 6-tick settlement
(0.6s) for the bot round-trip may be insufficient under server load.

---

### `class-select`
| Step | UI Element | Mechanism | Default (fx, fy) | Pixel @1280×720 | Env override | Visibility Proof | Fragility |
|---|---|---|---|---|---|---|---|
| 1 | Class select loaded | checkpoint | — | — | — | screenshot | — |
| 2 | First class card | `frac` click | (0.25, 0.45) | (320, 324) | `CCGS_AUTOPLAY_CLASS_FIRST_CARD` | none | MEDIUM |
| 3 | Confirm button | `frac` click | (0.5, 0.85) | (640, 612) | `CCGS_AUTOPLAY_CLASS_CONFIRM_BTN` | none | MEDIUM-HIGH |
| 4 | Class confirmed | checkpoint | — | — | — | screenshot | — |

**Notes**: 6-tick settlement after card click. No check that the class card was actually selected
(highlight state). Confirm fires regardless.

---

### `draft-auction-probe`
| Step | UI Element | Mechanism | Default (fx, fy) | Pixel @1280×720 | Env override | Visibility Proof | Fragility |
|---|---|---|---|---|---|---|---|
| 1 | Shop loaded | checkpoint | — | — | — | screenshot | — |
| 2 | First shop slot | `frac` click | (0.30, 0.45) | (384, 324) | `CCGS_AUTOPLAY_SHOP_FIRST_SLOT` | none | MEDIUM |
| 3 | Shop slot clicked | checkpoint | — | — | — | screenshot | — |
| 4 | Shop Confirm/Ready | `frac` click | (0.5, 0.85) | (640, **612**) | `CCGS_AUTOPLAY_SHOP_CONFIRM_BTN` | none | **HIGH** |
| 5 | Auction mount wait | `wait(12)` = 1.2s | — | — | `CCGS_AUTOPLAY_AUCTION_MOUNT_WAIT` | none | HIGH |
| 6 | Auction loaded | checkpoint | — | — | — | screenshot | — |
| 7 | Bid CTA | `frac` click | (0.5, 0.55) | (640, 396) | `CCGS_AUTOPLAY_AUCTION_BID_BTN` | none | MEDIUM |
| 8 | Bid wait | `wait(10)` = 1.0s | — | — | `CCGS_AUTOPLAY_AUCTION_BID_WAIT` | none | HIGH |
| 9 | Ready CTA | `frac` click | (0.5, 0.85) | (640, **612**) | `CCGS_AUTOPLAY_AUCTION_READY_BTN` | none | HIGH |
| 10 | Auction ready | checkpoint | — | — | — | screenshot | — |

**Notes**: Two separate clicks land at fy=0.85. The auction overlay may not have fully mounted
at `wait(12)` — no status polling for auction phase. Both the Shop Confirm and Auction Ready
buttons share the same fractional coordinate, which is correct only if both UIs position their
primary CTA in the same screen region.

---

### `placement-drag-probe`
| Step | UI Element | Mechanism | Default (fx, fy) | Pixel @1280×720 | Env override | Visibility Proof | Fragility |
|---|---|---|---|---|---|---|---|
| 1 | Placement loaded | checkpoint | — | — | — | screenshot | — |
| 2 | Hand first card (drag src) | `drag` start | (0.35, **0.92**) | (448, **662**) | `CCGS_AUTOPLAY_HAND_FIRST_CARD` | none | **CRITICAL** |
| 3 | Board first cell (drag dst) | `drag` end | (0.5, 0.55) | (640, 396) | `CCGS_AUTOPLAY_BOARD_FIRST_CELL` | none | MEDIUM |
| 4 | Placement dragged | checkpoint | — | — | — | screenshot | — |
| 5 | Submit button | `frac` click | (0.85, **0.92**) | (**1088**, **662**) | `CCGS_AUTOPLAY_SUBMIT_BTN` | none | **CRITICAL** |
| 6 | Placement submitted | checkpoint | — | — | — | screenshot | — |

**Notes**: `HAND_FIRST_CARD` at fy=0.92 → y=**662px** is only **58px from the bottom** of a 720px
window. `SUBMIT_BTN` at (0.85, 0.92) → (1088, 662) is both near-bottom **and** near-right edge
(192px from right at 1280px width). These are the most fragile coordinates in the entire recipe
library:

- At 720px logical height, y=662 is correct if the hand strip genuinely occupies the bottom 8%
  of the game viewport. If the hand strip is taller, or if the OS taskbar intrudes, the drag
  source lands in blank space.
- At window widths below 1280px, x=1088 at fx=0.85 compresses proportionally — a 1024px-wide
  window puts the Submit button at x=870, which may land outside the button boundary if it is
  not centered.
- The human-observed symptom ("clicks into blank/offscreen areas") is most consistent with this
  recipe at a sub-nominal window size.

---

### `resolution-observe`
| Step | UI Element | Mechanism | Visibility Proof | Fragility |
|---|---|---|---|---|
| 1 | resolution-started | checkpoint (passive) | screenshot | NONE |
| 2 | Soak | `wait(60)` = 6s default | none | NONE |
| 3 | resolution-complete | checkpoint (passive) | screenshot | NONE |

**Notes**: No input. Fully passive. Zero fragility from coordinate targeting.

---

### `game-over-observe`
| Step | UI Element | Mechanism | Visibility Proof | Fragility |
|---|---|---|---|---|
| 1 | game-over-wait-start | checkpoint (passive, no screenshot) | none | NONE |
| 2 | Soak | `wait(120)` = 12s default | none | NONE |
| 3 | game-over-screen | checkpoint | screenshot | NONE |
| 4 | Result soak | `wait(30)` = 3s | none | NONE |
| 5 | winner-confirmed | checkpoint | screenshot | NONE |

**Notes**: No input. Fully passive. Zero fragility.

---

### `full-game`
Composes: `lobby-create → class-select → draft-auction-probe → placement-drag-probe`
(+ optionally `resolution-observe`, `game-over-observe`)

Fragility inherits all fragility from composed recipes. 4-tick (0.4s) inter-phase settling gap.
The settling gap has no phase-label check — if any phase takes longer than the previous recipe's
wait ticks, the next recipe clicks into the wrong overlay.

---

### `vs-bot`
Composes: `add-bot-lobby → class-select → draft-auction-probe → placement-drag-probe`
(+ optionally `resolution-observe`, `game-over-observe`)

Same as `full-game` but replaces `lobby-create` with `add-bot-lobby`. Adds
`CCGS_DEBUG_UI=1` + `CCGS_AUTOPLAY_BOT_ROOM_READY=1` env guards. The Add Bot button
fragility (FRAG-02 below) is this recipe's unique risk beyond those shared with `full-game`.

---

### `round-loop`
Composes N iterations of: `(full-game) + (resolution-observe) + N×(draft-auction-probe + placement-drag-probe + resolution-observe) + (game-over-observe)`

The placement fragility (FRAG-01) repeats **N times** per run. `settle_ticks` default = 4 ticks
(0.4s) between phase boundaries; this is the only guard between the end of one phase and the
first click of the next.

---

### `idle`
No actions. Zero fragility.

---

## 3. Fragility Register

### FRAG-01 — CRITICAL: Bottom-strip clicks at fy=0.92

**Affected recipes**: `placement-drag-probe` (and all composites that include it)
**Affected coordinates**: `HAND_FIRST_CARD (0.35, 0.92)`, `SUBMIT_BTN (0.85, 0.92)`
**Pixel at 1280×720**: `(448, 662)` and `(1088, 662)` — 58px from bottom edge
**Root cause of observed symptom**: A Bevy game window launched at a smaller-than-nominal
logical size (e.g. due to monitor resolution, Bevy `WindowResolution` config, or OS scaling)
compresses these coordinates into the window chrome or into an area that is blank because the
hand strip did not render at that height.

**Why file-disjoint repair is possible**: `_coords.py` DEFAULTS dict is the single source.
Lowering `HAND_FIRST_CARD` to `FracPoint(0.35, 0.88)` and `SUBMIT_BTN` to `FracPoint(0.85, 0.88)`
adds 29px headroom at 720p (y=634 vs y=662) without touching any other file. Worker 1843
(click-target viewport guard) is the natural owner.

---

### FRAG-02 — HIGH: Add Bot button (debug-only, no visibility proof)

**Affected recipes**: `add-bot-lobby`, `vs-bot`
**Affected coordinate**: `LOBBY_ADD_BOT_BTN (0.5, 0.72)` → (640, 518) @720p
**Root cause**: The Add Bot button exists only when `CCGS_DEBUG_UI=1`. If the button renders
at a different vertical position in the debug panel (e.g. below other debug controls added
since PROMPT 1603), the click misses. The recipe has an env-guard but no positional guard.
**File-disjoint repair**: Add a `CCGS_AUTOPLAY_LOBBY_ADD_BOT_BTN` env override note in
`docs/autoplay.md` with a measurement protocol. Worker 1842 (window size repair) should
confirm the Add Bot panel layout when debug UI is active.

---

### FRAG-03 — HIGH: Time-based phase waits (no `phase_label` polling)

**Affected recipes**: All interactive recipes
**Root cause**: `autoplay/status` exposes `phase_label` (e.g. `"Lobby"`, `"DraftAuction"`,
`"Placement"`) but the driver only uses it for timeline logging. Recipes use static `wait(N)`
between clicks instead of polling `phase_label` for the expected value.
**Evidence**: The status schema v2 (capabilities.json in evidence run 20260528-090613-Z)
explicitly documents `phase_label` as available. At tick 1, `phase_label = "Lobby"` was
confirmed. The value changes on each `S2CPhaseChanged` message from the server.
**Impact**: A slow server round-trip (network jitter, high load, VS Code rebuild) causes the
next recipe phase to fire before the expected UI has mounted. The wait between
`draft-auction-probe`'s Shop Confirm click and the auction overlay mount (`CCGS_AUTOPLAY_AUCTION_MOUNT_WAIT` default = 12 ticks = 1.2s) is the most time-sensitive gap.
**File-disjoint repair**: Add a `poll_phase(label, max_ticks)` helper to `_builder.py` that
emits a new `local.poll_phase` pseudo-action. The driver handles it by re-querying status
until `phase_label` matches or the tick cap is reached. No recipe files need edits.

---

### FRAG-04 — MEDIUM-HIGH: Confirm/Ready button cluster at fy=0.85

**Affected recipes**: `lobby-create`, `add-bot-lobby`, `class-select`, `draft-auction-probe` (×2)
**Affected coordinates**: `LOBBY_CONFIRM_BTN`, `CLASS_CONFIRM_BTN`, `SHOP_CONFIRM_BTN`,
`AUCTION_READY_BTN` — all at `(0.5, 0.85)` → (640, 612) @720p
**Root cause**: Four distinct UI elements share the same fractional coordinate. This is by
design (all are lower-action-panel CTAs). Fragility arises when any phase's CTA is NOT in
the lower action panel — e.g. a modal dialog, error banner, or phase mismatch that covers
the lower panel renders a different element at that coordinate.
**File-disjoint repair**: None needed if FRAG-03 (phase-label polling) is fixed — phase
gating prevents clicking when the wrong overlay is active.

---

### FRAG-05 — LOW: `window_logical_size` fallback with no warning

**Affected code**: `driver.py` lines 226–229
**Behavior**: If `autoplay/status` returns `null` or a malformed `window_logical_size`,
the driver silently falls back to `[1280.0, 720.0]` and proceeds. This is the nominal
window size and is correct in practice, but if a Bevy config change causes a different
startup size, the fallback masks the mismatch.
**File-disjoint repair**: Add a `log()` call for the fallback path in `driver.py` — one-line
change, does not alter recipe files.

---

### FRAG-06 — LOW: `window_logical_size` read once at tick 1

**Affected code**: `driver.py` line 225 (`if recipe_actions is None`)
**Behavior**: Window size is snapshotted at recipe build time. If the window is resized
during a run (user interaction or OS snap), subsequent fractions are stale.
**Scope**: Unlikely in CI/automated context where no human touches the window. Negligible
risk in current usage.

---

## 4. Summary Table

| Recipe | Interactive? | Coord mechanism | Phase guard | Visibility proof | Max fragility |
|---|---|---|---|---|---|
| `smoke` | yes (1 click) | frac | none | checkpoint-screenshot | LOW |
| `lobby-create` | yes (2 clicks) | frac | none | checkpoint-screenshots | MEDIUM-HIGH |
| `add-bot-lobby` | yes (3 clicks) | frac | env `CCGS_DEBUG_UI` | checkpoint-screenshots | HIGH |
| `class-select` | yes (2 clicks) | frac | none | checkpoint-screenshots | MEDIUM-HIGH |
| `draft-auction-probe` | yes (4 clicks) | frac | none | checkpoint-screenshots | HIGH |
| `placement-drag-probe` | yes (1 drag + 1 click) | frac drag | none | checkpoint-screenshots | **CRITICAL** |
| `resolution-observe` | no | passive | n/a | screenshots | NONE |
| `game-over-observe` | no | passive | n/a | screenshots | NONE |
| `full-game` | yes (composite) | frac | env `BOT_ROOM_READY` | composite screenshots | CRITICAL |
| `vs-bot` | yes (composite) | frac | env `DEBUG_UI` + `BOT_ROOM_READY` | composite screenshots | CRITICAL |
| `round-loop` | yes (composite ×N) | frac | env `BOT_ROOM_READY` | composite screenshots | CRITICAL |
| `idle` | no | passive | n/a | none | NONE |

---

## 5. Proposed File-Disjoint Repairs

Each repair touches only files not owned by other active workers (1842, 1843, 1844).

| ID | Fragility | File(s) to edit | Change |
|---|---|---|---|
| R-01 | FRAG-01 | `tools/autoplay/recipes/_coords.py` | Lower `HAND_FIRST_CARD` fy from 0.92 → 0.88; lower `SUBMIT_BTN` fy from 0.92 → 0.88 |
| R-02 | FRAG-03 | `tools/autoplay/recipes/_builder.py`, `tools/autoplay/driver.py` | Add `poll_phase(label, max_ticks)` pseudo-action; driver handles `local.poll_phase` by re-querying status |
| R-03 | FRAG-05 | `tools/autoplay/driver.py` | Add `log()` warning when `window_logical_size` fallback fires |
| R-04 | FRAG-02 | `docs/autoplay.md` | Document Add Bot button coordinate measurement protocol |

> **Coordination note**: R-01 (`_coords.py`) is the only file-level overlap with worker 1843
> (click-target viewport guard). If 1843 is already editing `_coords.py`, defer R-01 to that
> worker. R-02 and R-03 touch `driver.py` and `_builder.py` which are in scope for this
> report's follow-up only if 1843 does not own them.

---

## 6. Evidence Artifacts Referenced

| File | Key finding |
|---|---|
| `tools/autoplay/recipes/_coords.py` | All 12 coordinate defaults; fy=0.92 is the critical outlier |
| `tools/autoplay/driver.py` lines 226–229 | `window_logical_size` read once, fallback to [1280,720] |
| `tools/autoplay/driver.py` line 219 | `autoplay/status` polled every tick but `phase_label` not used by recipes |
| `production/qa/evidence/autoplay-runs/20260528-090613-Z/driver-timeline.jsonl` tick 1 | `window_logical_size=[1280,720]`, `phase_label="Lobby"` confirmed in live run |
| `production/qa/evidence/autoplay-runs/20260528-090613-Z/driver.log` | Win32 capture 1296×759 (OS chrome delta = 16×39px above logical size) |
| `tools/autoplay/recipes/capabilities.json` (same run) | `phase_label` and `client_state_label` documented in schema v2 |

---

1848: AUTOPLAY-RECIPE-VISIBLE-TARGET-COVERAGE-MAP: DONE
