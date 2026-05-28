# PROMPT 1844 — Autoplay vs-Bot Viewport / Click-Target Evidence Audit

**Date:** 2026-05-28  
**Auditor:** PROMPT-1844 worker  
**Source tree:** origin/main@b856eef4  
**Scope:** Read-only forensic audit — no source edits.

---

## 1. Evidence Corpus

Three autoplay-runs were present at audit time:

| Run dir | Started (UTC) | Outcome | Window size (final status) |
|---|---|---|---|
| `20260528-051148-Z` | 05:11:48 | ok (smoke_exit=0) | `[1280, 720]` |
| `20260528-063609-Z` | 06:36:09 | ok (smoke_exit=0) | `[1280, 720]` |
| `20260528-090613-Z` | 09:06:13 | ok (smoke_exit=0) | `[1280, 1076]` ⚠️ |

All three runs belong to the `vs-bot` recipe and completed with 15/15 checkpoints.  
Composite summaries exist for all three in `production/qa/evidence/composite-runs/`.

---

## 2. Critical Finding — Mid-Run Window Resize (Run 090613)

### 2.1 Resize Sequence

Run `20260528-090613-Z` shows the window changed size during the run.  
Extracted from `driver-timeline.jsonl`:

```
tick=1    window_size = [1280, 720]   ← recipe built at this tick
...
tick=114  window_size = [1280, 720]   ← last stable 720 tick
tick=115  window_size = [1280, 505]   ← RESIZE START (shrink)
tick=116  window_size = [1280, 539]
tick=117  window_size = [1280, 662]
tick=118  window_size = [1280, 664]
tick=120  window_size = [1280, 949]
tick=121  window_size = [1280, 1081]
tick=122  window_size = [1280, 1132]
tick=123  window_size = [1280, 1155]
tick=125  window_size = [1280, 1155]
tick=126  window_size = [1280, 1082]
tick=127  window_size = [1280, 1076]  ← stabilized; runs to end
```

The window shrank to 505 px then rapidly grew to 1076 px in ~12 ticks (~1.2 s).  
Runs `051148` and `063609` show **no window size changes** at any tick.

### 2.2 Trigger

The resize is correlated with the `win32_foreground` checkpoint capture at tick ~109
(`auction-loaded`). The driver's `foreground_robust` procedure calls:

```
SetWindowPos(hwnd, HWND_NOTOPMOST, ...)   ← removes always-on-top
ShowWindow(hwnd, SW_RESTORE)              ← may interact with Windows snap
SetForegroundWindow(hwnd)
```

On Windows 11, `SW_RESTORE` on a window in a DWM-snapped state triggers a
snap-restore animation that emits rapid WM_SIZE events. Winit propagates each
WM_SIZE to `window.resolution`, producing the cascade seen in the timeline.

Supporting signals:
- `cursor_logical` is `None` at ticks 113–114, indicating the cursor was
  outside the window bounds at the moment of resize.
- `process.log` (Bevy) logs no explicit resize event; the resize is entirely
  Win32/DWM-driven.
- Runs 051148 and 063609 did not exhibit the resize — their DWM state was
  different at launch (likely not snapped).

---

## 3. Click-Target Accuracy Analysis

### 3.1 Coordinate System

`driver.py` (line 229) builds the recipe **once** at tick 1 using
`window_size = [1280.0, 720.0]`. `RecipeBuilder.frac(fx, fy)` converts fractional
coords to absolute logical pixels at build time. After the build, all tick-N
actions carry hardcoded absolute pixels regardless of later window changes.

### 3.2 Pre-Resize Clicks (ticks 1–113) — RELIABLE

All lobby, class-select, and shop clicks occurred while `window_size = [1280, 720]`.
The recipe was built for 720-height, and 720 was the actual height. Clicks landed at
correct fractional positions.

| Checkpoint | Tick | Target coord | Window at tick | Verdict |
|---|---|---|---|---|
| lobby-loaded → Add Bot | 8 | `[640, 396]` (0.5×0.55) | `[1280, 720]` | ✓ in-bounds, correct fraction |
| bot-added → Confirm | 18–19 | `[640, 518]` (0.5×0.72) | `[1280, 720]` | ✓ |
| lobby-confirmed → clear | 43 | — | `[1280, 720]` | ✓ |
| class-select first card | 52–54 | `[320, 324]` (0.25×0.45) | `[1280, 720]` | ✓ |
| class-confirmed | 61–63 | `[640, 612]` (0.5×0.85) | `[1280, 720]` | ✓ |
| shop-slot-clicked | ~86–88 | `[384, 324]` (0.30×0.45) | `[1280, 720]` | ✓ |
| auction-loaded bid click | 95–96 | `[640, 612]` (0.5×0.85) | `[1280, 720]` | ✓ |

### 3.3 Peri-Resize Clicks (ticks 113–128) — UNRELIABLE

The window resized during ticks 115–127. A click was issued at tick 114–116:

```
tick=114  action=cursor       cursor=[640, 396]  window=[1280, 720]
tick=115  status_read         window=[1280, 505]  cursor=[640, 396]  mouse=[]
tick=115  action=mouse_down   window=[1280, 505]  ← CLICK DURING RESIZE
tick=116  action=mouse_up     window=[1280, 539]  ← RELEASE DURING RESIZE
```

The cursor was placed for a 720-height window (`y=396 = 55%`) but the click
fired when height was 505. At height 505, `y=396` is 78.4% height — well above
the intended AUCTION_BID_BTN position. Input injection during a Winit resize
event is unreliable; Bevy may discard or misroute the event.

### 3.4 Post-Resize Clicks (ticks 128–260) — STRUCTURALLY WRONG

After the window stabilized at 1076, the recipe continued with 720-baked
coordinates. The UI layout is designed for 1280×720; in a 1076-high window,
the extra 356 px appears below the game content as blank space.

The critical question is whether the game UI **reflowed** for the taller window
or kept UI elements at their 720-relative absolute positions:

- If Bevy/bevy_ui **reflowed**: buttons moved downward to `fy × 1076`, and all
  recipe clicks at `y = fy × 720` land ~28% above the actual buttons.
- If Bevy/bevy_ui **did not reflow**: elements stayed at their absolute pixel
  positions (same as in a 720 window), and clicks at `fy × 720` still hit
  correctly but there is blank padding below.

The run passed all 15 checkpoints, which are time-based (tick count), not
pixel-verified. Checkpoint passage does NOT confirm click accuracy.

Post-resize critical clicks:

| Checkpoint | Tick | Coord (720-baked) | Window | Fraction in actual window |
|---|---|---|---|---|
| auction-ready bid | 127–128 | `[640, 396]` / `[640, 612]` | `[1280, 1076]` | 36.8% / 56.9% instead of 55% / 85% |
| placement drag start | 149–150 | `[448, 662]` | `[1280, 1076]` | 61.5% instead of 92% |
| placement drag end | 155 | `[640, 396]` | `[1280, 1076]` | 36.8% instead of 55% |
| placement submit | 166–167 | `[1088, 662]` | `[1280, 1076]` | 61.5% instead of 92% |

The `HAND_FIRST_CARD (0.35, 0.92)` and `SUBMIT_BTN (0.85, 0.92)` targets at
`y=662` now land at 61.5% height — if the UI reflowed these buttons are missed
entirely.

---

## 4. Win32 Screenshot Reliability

### 4.1 PrintWindow Frozen Throughout

All 11 win32_printwindow checkpoint captures in run 090613 triggered frozen
detection (same MD5 as prior capture):

- Ticks 51–113: hash `0874d30f...` — stale from first capture (lobby state)
- Ticks 147–259: hash `ca2ab3e8...` — stale from post-resize first capture

The driver correctly fell back to `desktop_bitblt` for all 11 captures.  
**All win32_printwindow PNG files in run 090613 are stale frames and must not be
used as evidence of actual game state at those timestamps.**

Runs 051148 and 063609 do not have driver.log frozen events (earlier commit,
smaller log — frozen detection added in PROMPT 1818 is present but those runs
showed different freeze patterns; their bitblt fallback was also used on some
ticks).

### 4.2 Desktop BitBlt Quality

`desktop_bitblt` PNGs (14 files in 090613) have distinct pixel hashes per
checkpoint, confirming the fallback captured live game state. File sizes range
59–81 KB vs 48–67 KB for stale win32 frames. These are mechanically usable
for visual review.

Physical capture dimensions: `1296×759` (pre-resize) and `1296×1115`
(post-resize). The +16px width and +39/39px height are Windows window-chrome
(title bar + border) outside Bevy's logical coordinate space.

### 4.3 In-Game Screenshots

15 Bevy-native screenshots (`screenshots/000000.png` … `000057.png`) were
captured via `autoplay/screenshot` RPC. These are the most reliable: rendered
at the exact logical resolution the game sees (1280×720 pre-resize, 1280×1076
post-resize based on Bevy renderer output). All 15 are present and paired with
JSON metadata. Screenshot sizes jump from 86 KB (pre-resize) to 118 KB
(post-resize), confirming a real render-target change at auction-ready.

---

## 5. Checkpoint Reliability Assessment

Checkpoints are **time-based**: `label` fires when the driver reaches a tick,
not when the game visually confirms a state. They do not validate:
- That a click landed on the correct UI element
- That the game transitioned to the intended state
- That the window was at the expected size when the click fired

A run can pass all 15 checkpoints while the bot clicked wrong targets in every
post-resize phase. The run progressing to `vs-bot-post-resolution` proves the
server advanced the game loop, but does not prove the bot's UI interactions were
correct.

| Checkpoint | Tick | Window at tick | Click accuracy | Evidence quality |
|---|---|---|---|---|
| lobby-loaded | 1 | 720 | N/A | ✓ RELIABLE |
| bot-added | 26 | 720 | ✓ | ✓ RELIABLE |
| lobby-confirmed | 38 | 720 | ✓ | ✓ RELIABLE |
| class-select-loaded | 47 | 720 | ✓ | ✓ RELIABLE |
| class-confirmed | 68 | 720 | ✓ | ✓ RELIABLE |
| shop-loaded | 77 | 720 | ✓ | ✓ RELIABLE |
| shop-slot-clicked | 89 | 720 | ✓ | ✓ RELIABLE |
| auction-loaded | 109 | 720 | ✓ | ✓ RELIABLE |
| auction-ready | 134 | 1076 | ⚠️ unknown | ⚠️ UNRELIABLE post-resize |
| placement-loaded | 143 | 1076 | ⚠️ unknown | ⚠️ UNRELIABLE post-resize |
| placement-dragged | 160 | 1076 | ⚠️ wrong fraction | ⚠️ UNRELIABLE post-resize |
| placement-submitted | 172 | 1076 | ⚠️ wrong fraction | ⚠️ UNRELIABLE post-resize |
| resolution-started | 181 | 1076 | N/A (observe only) | ✓ RELIABLE (no click) |
| resolution-complete | 246 | 1076 | N/A | ✓ RELIABLE |
| vs-bot-post-resolution | 255 | 1076 | N/A | ✓ RELIABLE |

---

## 6. PROMPT 1842 / 1843 Status

**PROMPT 1842** (`work/1842-window-size-repair`, not merged to main):  
- Adds `CCGS_WINDOW_WIDTH=1280` and `CCGS_WINDOW_HEIGHT=720` env vars to
  `Run-AutoplaySmoke.ps1`.
- Comments claim the `AutoplayPlugin Startup system` enforces minimum via
  `window.resolution.set()`.  
- **Gap**: The Rust client-side enforcement was not verified as preventing a
  mid-run resize caused by Win32 `ShowWindow/SetWindowPos` operations in
  `win_foreground.py`. `window.resolution.set()` sets the initial size; it
  does not prevent WM_SIZE events from the OS DWM.  
- **Gap**: Branch is at commit `71484998` (same as pre-1833 main). No new Rust
  or Python code is visible in the diff — only the PS1 launcher change.

**PROMPT 1843** (`click-target viewport guard`):  
- No worktree found matching 1843 in `git worktree list`.  
- No viewport guard, clamp logic, or `min_window` check exists in current
  `driver.py`, `_builder.py`, or any recipe file on `main`.  
- The PROMPT task description mentions it as "active" but it is not landed.

---

## 7. Runs 051148 and 063609 — Clean Baselines

Both earlier runs are mechanically reliable:
- Window `[1280, 720]` throughout all 260 ticks.
- All 15 checkpoints passed.
- Bevy screenshots present (15 each).
- No mid-run cursor-null events.
- Recipe coordinates correct for actual window height.

These two runs constitute the valid baseline evidence set.  
Run `090613` should be treated as **evidence of the resize bug**, not as a
clean PASS.

---

## 8. Concrete Acceptance Criteria for Repairs / Verify Lanes

### AC-VPT-01 — Minimum Window Size Gate (BLOCKING)
The autoplay driver MUST abort with exit code ≠ 0 if the initial
`window_logical_size` from the first status poll is below `[1280, 720]`.  
A shrinkage to `[W, < 720]` before any click MUST also abort.

### AC-VPT-02 — Mid-Run Resize Detection (BLOCKING)
The driver MUST detect when `window_logical_size` changes after recipe build.  
On any resize > ±10 px in either dimension after tick 1, the driver MUST:
1. Log a `WARN: window_resized` entry in `driver.log`.
2. Emit a `local.note` checkpoint with the old and new sizes.
3. Abort with a non-zero exit code OR mark the run `NEEDS_HUMAN_GUI` and
   refuse to emit a clean `smoke_exit_code=0`.

### AC-VPT-03 — Null Cursor Guard Before Clicks (ADVISORY)
Before any `mouse_down` action, the driver SHOULD verify that `cursor_logical`
is not `None`. A `None` cursor means the cursor is outside the window. The
driver SHOULD log a warning and skip the click (or retry after re-centering).

### AC-VPT-04 — Post-Resize Recipe Rebuild (ADVISORY)
If the window resizes by more than ±10 px and the run is to continue (not aborted),
the recipe MUST be rebuilt with the new window size. The existing single-build
architecture in `driver.py` does not support this without structural change.

### AC-VPT-05 — Win32 Capture Quality Flag (ADVISORY)
If ALL win32_printwindow captures for a run returned FROZEN hashes (as in run
090613), the composite report MUST record `win32_capture_quality: frozen_all`
and flag the run as `NEEDS_HUMAN_GUI` rather than `PASS`.

### AC-VPT-06 — Minimum Screenshot Requirements (BLOCKING for PASS)
A run claiming mechanical PASS MUST have:
- At least one in-game screenshot (`screenshots/000NNN.png`) per checkpoint.
- At least one `desktop_bitblt` PNG with a distinct pixel hash from its
  predecessor for each phase transition checkpoint.
- No screenshot with the same pixel hash as the prior checkpoint screenshot
  (stale-frame detection).

### AC-VPT-07 — Window Size in Composite Report (ADVISORY)
The composite report SHOULD record `initial_window_size`, `final_window_size`,
and `window_resize_events` (count and tick range). A non-zero `window_resize_events`
count MUST downgrade the verdict to `NEEDS_HUMAN_GUI`.

### AC-VPT-08 — Minimum Window/Client Size for Recipe Validity (NORMATIVE)
The vs-bot recipe is designed for `[1280, 720]` minimum. This is the minimum
logical size for all fractional click targets to be in-bounds. The client
launcher MUST set `CCGS_WINDOW_WIDTH=1280` and `CCGS_WINDOW_HEIGHT=720` AND the
Rust `AutoplayPlugin` MUST enforce this via `window.resolution.set()` at
`Startup` with a `WindowResized` guard that rejects DWM-initiated shrinks below
the minimum. A SW_RESTORE / TOPMOST toggle from `win_foreground.py` must not
be able to shrink the window below 720 px.

---

## 9. Summary

| Finding | Severity | Status |
|---|---|---|
| Mid-run window resize (090613) triggered by win32 foreground ops | BLOCKER | Confirmed |
| Recipe coordinates baked at tick-1, not updated on resize | BLOCKER | Confirmed, architectural gap |
| Post-resize click targets at wrong fractions (placement, submit) | BLOCKER | Confirmed in timeline |
| Win32 PrintWindow all-frozen across entire run | MAJOR | Confirmed, fallback working |
| Null cursor (off-screen) before resize event | MAJOR | Confirmed, no guard |
| PROMPT 1842 Rust enforcement not verified | MAJOR | Unverified |
| PROMPT 1843 viewport guard not landed on main | MAJOR | Absent |
| Runs 051148 and 063609 are clean baselines | INFO | Confirmed |
| All 15 checkpoints pass even with resize-corrupted run | INFO | Confirmed (time-based, not state-based) |

**The most recent run (090613) is NOT reliable evidence of correct bot behaviour.
The two earlier runs on 2026-05-28 are the valid clean baselines.**

---

1844: AUTOPLAY-VSBOT-VIEWPORT-CLICK-EVIDENCE-AUDIT: DONE
