# PROMPT-2027: Autoplay Input Click-Target Forensic Audit

**Date**: 2026-05-28
**Source-of-truth**: `origin/main@e7b51e84` (PROMPT 2019 mainland); updated against PROMPT-2024 truth set (`origin/main@5016bc07`)
**Evidence root**: `D:\_DEV\Work\Claude-Code-Game-Studios\production\qa\evidence\autoplay-runs\`
**Auditor**: PROMPT-2027 forensic worker

---

## 1. Evidence Inventory

Three runs found in the production artifact directory, all using recipe `vs-bot`:

| Run ID | Started (UTC) | Driver version (inferred) | Win32 captures | Exit code | Notes |
|--------|---------------|--------------------------|----------------|-----------|-------|
| `20260528-051148-Z` | 05:11:48 | Pre-PROMPT-1794 (no Win32 capture) | None | 0 | Stable 1280×720 |
| `20260528-063609-Z` | 06:36:09 | PROMPT-1794–1803 era | All 15 frozen (same hash) | 0 | No BitBlt fallback |
| `20260528-090613-Z` | 09:06:13 | PROMPT-1818 era (frozen detection + BitBlt) | 5 OK, 10 frozen+BitBlt | 0 | **Window resize event** |

**Commit timeline establishing driver versions** (all UTC, +0100 offset):

| UTC time | Commit | Feature added |
|----------|--------|---------------|
| 04:49 | PROMPT-1794 | Win32 GDI PrintWindow capture |
| 06:16 | PROMPT-1803 | Win32 foreground hardening |
| 07:28 | PROMPT-1813 | Desktop BitBlt fallback |
| 07:52 | PROMPT-1818 | Frozen-frame detection (MD5 hash) |
| **11:34** | **PROMPT-1880** | **Click-target viewport guard + drift check** |

All three runs occurred **before PROMPT-1880** was committed. PROMPT-1880 is the first commit to add the mid-run drift guard and build-time minimum-size guard.

---

## 2. Click-Target Correlation Table

### Runs 1 & 2 (window stable at 1280×720)

All 11 click events per run are **OK**. Coordinates exactly match expected fractional targets computed at build-time window size. No offscreen or stale-target events.

### Run 3 (20260528-090613-Z) — window resize event

Build-time window: `[1280, 720]`
Final window (status.json at run end): `[1280, 1076]`
Win32 capture peak: `1296×1115` (frame including title bar/chrome)

| Tick | Elapsed (s) | Action | Click coords | Screenshot size | Window at tick | Expected target | Actual frac | Verdict |
|------|-------------|--------|-------------|-----------------|---------------|-----------------|-------------|---------|
| 8 | 1.09 | mouse_down | (640, 396) | 1280×720 | 1280×720 | LOBBY_CREATE_BTN (0.50, 0.55) | (0.50, 0.55) | **OK** |
| 19 | 2.20 | mouse_down | (640, 518) | 1280×720 | 1280×720 | LOBBY_ADD_BOT_BTN (0.50, 0.72) | (0.50, 0.72) | **OK** |
| 33 | 4.08 | mouse_down | (609, 604) | 1280×720 | 1280×720 | LOBBY_CONFIRM_BTN (0.50, 0.85) | (0.48, 0.84) | **OK** (≤3% drift) |
| 54 | 6.33 | mouse_down | (320, 324) | 1280×720 | 1280×720 | CLASS_FIRST_CARD (0.25, 0.45) | (0.25, 0.45) | **OK** |
| 63 | 7.39 | mouse_down | (640, 612) | 1280×720 | 1280×720 | CLASS_CONFIRM (0.50, 0.85) | (0.50, 0.85) | **OK** |
| 84 | 9.67 | mouse_down | (384, 324) | 1280×720 | 1280×720 | SHOP_FIRST_SLOT (0.30, 0.45) | (0.30, 0.45) | **OK** |
| 96 | 11.44 | mouse_down | (640, 612) | 1280×720 | 1280×720 | AUCTION_CONFIRM (0.50, 0.85) | (0.50, 0.85) | **OK** |
| **116** | **12.89** | **mouse_down** | **(640, 396)** | 1280×720 (pre-resize screenshot) | **1280×539** (mid-resize shrink) | AUCTION_BID_BTN intended at y=396 (build-720) | **(0.50, 0.73)** | **STALE — mid-resize** |
| **129** | **14.75** | **mouse_down** | **(640, 612)** | 1296×1115 | **1280×1076** | Confirm/bid2 at y=612 (build-720) | **(0.50, 0.57)** | **STALE — y miss +302px** |
| **150** | **18.75** | **drag start** | **(448, 662)** | 1296×1115 | **1280×1076** | HAND_FIRST_CARD at y=662 (build-720 → actual y≈990) | **(0.35, 0.62)** | **WRONG_TARGET — y miss +328px** |
| 151–154 | 18.85–19.25 | drag cursor | (486–640, 609–396) | 1296×1115 | 1280×1076 | Drag path: hand→board (all stale) | (0.38–0.50, 0.37–0.57) | **STALE path** |
| **167** | **20.98** | **mouse_down** | **(1088, 662)** | 1296×1115 | **1280×1076** | SUBMIT_BTN at y=662 (build-720 → actual y≈990) | **(0.85, 0.62)** | **WRONG_TARGET — y miss +328px** |

**Summary of bad clicks in Run 3**: 4 actions (3 click events + 1 drag sequence) landed at stale 720px-height coordinates while the window was 1076px tall. The hand-card drag source and submit button were each **328px below their actual on-screen position**, clicking into empty game-board area or off the UI strip entirely.

---

## 3. Window Resize Timeline (Run 3)

The OS-level resize was triggered between ticks 114 and 127, consistent with a DWM window snap or user drag that first shrank then expanded the window:

| Tick | Status RPC window size | Drift from build (720h) | Win32 capture size |
|------|------------------------|------------------------|--------------------|
| 1–114 | [1280, 720] | 0px | 1296×759 |
| **115** | **[1280, 505]** | **215px** ← below minimum | — |
| 116 | [1280, 539] | 181px | — |
| 117 | [1280, 662] | 58px | — |
| 118 | [1280, 664] | 56px | — |
| 120 | [1280, 949] | 229px | — |
| 121 | [1280, 1081] | 361px | — |
| 122 | [1280, 1132] | 412px | — |
| 123–126 | [1280, 1155–1082] | 435–362px | — |
| **127** | **[1280, 1076]** | 356px ← stabilises | — |
| 128–260 | [1280, 1076] | 356px | 1296×1115 (from tick 138) |

**At tick 115 the window temporarily shrank below the 720px minimum** before the OS snapped it upward to the new 1076px height. No guard fired because PROMPT-1818's driver had no drift check.

**Win32 vs Bevy size discrepancy**: Win32 capture consistently reports 16px wider and 39px taller than Bevy's logical window size (e.g., 1296×759 vs 1280×720). This is the Windows window chrome (border + title bar). The discrepancy is **expected and correct** — recipe coordinates use the Bevy logical size (client area), not the OS frame size.

---

## 4. Guard Coverage Matrix

### Against PROMPT-1818 (driver version active during all three runs)

| Guard | Status in PROMPT-1818 | Evidence of failure |
|-------|-----------------------|-------------------|
| Pre-build minimum window size | **ABSENT** — uses `or [1280.0, 720.0]` fallback | No direct evidence; would silently build recipe with wrong size if window was missing |
| Mid-run drift check | **ABSENT** | RUN3: window drifted 215px (tick 115) with no abort; clicks dispatched at stale coords |
| Click-target OOB check | **ABSENT** | All stale clicks were technically in-bounds but wrong position |
| `cursor_logical=None` guard | **ABSENT** | RUN3 ticks 112–113: cursor=None (cursor left window) with no abort |
| Post-foreground shrink check | **ABSENT** | RUN3 tick 113: foreground call preceded window shrink at tick 115 |
| `build_win=` log line | **ABSENT** | All three run logs show `recipe=vs-bot actions=74 last_recipe_tick=260` with NO `build_win=` field; confirms pre-1880 driver |

### Against PROMPT-1880 / current `origin/main@e7b51e84`

| Guard | Status in PROMPT-1880 | Would it have prevented Run 3 failures? |
|-------|-----------------------|----------------------------------------|
| Pre-build minimum window size | **PRESENT** — fails if `w < 1280 or h < 720` | N/A — build window was valid (720px) |
| **Mid-run drift check** | **PRESENT** — aborts at tick N if `|current_h - build_h| > 10px` | **YES** — tick 115: `|505 - 720| = 215 > 10` → exit code 5, no action dispatched. Ticks 116/129/150/167 never reached. |
| Click-target OOB check | **PRESENT** — aborts if `coord >= window_size` | Not needed here (all stale clicks were technically in-bounds) |
| `cursor_logical=None` guard | **PRESENT** — aborts before input if cursor left window | N/A (cursor returned before first input at tick 6) |
| Post-foreground shrink check | **PRESENT** — re-polls after `ensure_foreground` | Would catch if SW_RESTORE triggered the shrink at the foreground call |
| `build_win=` log line | **PRESENT** | Future runs with 1880+ code will show `build_win=(1280x720)` confirming effective build size |

---

## 5. Frozen Screenshot Analysis (PROMPT-2024 Findings Applied)

PROMPT-2024 confirmed that Bevy client screenshots are **identical byte-for-byte** across all checkpoints in Runs 1 and 2, and through the first 8 checkpoints of Run 3:

| Run | All PNGs identical? | Bytes each | Change point |
|-----|---------------------|-----------|--------------|
| Run 1 (051148-Z) | **YES** — all 15 identical | 86,080 B | Never changes |
| Run 2 (063609-Z) | **YES** — all 15 identical | 86,148 B | Never changes |
| Run 3 (090613-Z) | Partial — first 8 identical, last 7 changed | 86,108 B → 117,843 B | After checkpoint `auction-ready` (tick 134) |

The size jump in Run 3 from 86,108 B to 117,843 B at tick 134 coincides exactly with the OS window resize completing (window stabilized at [1280, 1076] by tick 127; screenshot at tick 138 is the first post-resize capture). The larger PNG file corresponds directly to the larger pixel area (1280×1076 vs 1280×720).

**Critical implication**: The Bevy client was rendering the **same frozen frame** through all lobby, class-select, shop, auction phases in Runs 1 and 2. This means:

1. The Bevy screenshots are NOT showing the current game state — they show a cached/static render that does not update during gameplay.
2. The autoplay click-target coordinates land on fractional positions of a **visually static** screen. Whether the underlying input events actually reach the correct interactive elements is not confirmed by visual evidence.
3. The `phase_label: "Lobby"` throughout all 262 ticks in ALL three runs (confirmed by PROMPT-2024) correlates with this: the client UI may never have transitioned to the in-session rendering path.

**What the screenshots DO confirm for Run 3**:
- Pre-resize (tickets 0–30, sizes 86,108 B): Client was rendering the same static frame as Runs 1&2 (Lobby state).
- Post-resize (tickets 37–57, sizes 117,843 B): A different frame was captured. The resize event may have forced a render flush, or the game did progress but only became visually captured after the window grew.

**Artifact paths**:
- `production/qa/evidence/autoplay-runs/20260528-051148-Z/screenshots/` (all 86,080 B)
- `production/qa/evidence/autoplay-runs/20260528-063609-Z/screenshots/` (all 86,148 B)
- `production/qa/evidence/autoplay-runs/20260528-090613-Z/screenshots/` (86,108 B × 8, 117,843 B × 7)

---

## 6. Remaining Gaps (Not Closed by PROMPT-1880)

### GAP-1: `phase_label` / `client_state_label` stuck at "Lobby" — and Bevy screenshots frozen (CRITICAL)

All three runs show `phase_label="Lobby"` and `client_state_label="Lobby"` in **every** status poll for the **entire** game session (lobby → class-select → shop → auction → placement → resolution). The status.json from all three runs confirms this at run end. Combined with PROMPT-2024's finding that Bevy screenshots are **identical byte-for-byte** across all 15 checkpoints in Runs 1 and 2, this indicates:

- The Bevy client is NOT visually rendering or reporting phase transitions to the autoplay subsystem.
- Phase-aware guards (e.g., abort if submitting a unit during the wrong phase) cannot be implemented until this field updates correctly.
- All 11 click events per run landed on a **visually static/frozen** screen. The clicks may be reaching the underlying input system, but visual confirmation is unavailable.

**Impact**: HIGH — the entire visual QA value of the autoplay run is undermined. PROMPT-2024 confirms this finding applies to all three runs uniformly (not just Run 3).

**Root cause location**: `client::autoplay` Rust plugin (`client/src/autoplay/`) — the `phase_label` and `client_state_label` fields are not being updated when Bevy game state transitions beyond the Lobby phase. Additionally, the Bevy render pipeline appears to be serving cached frames via `save_to_disk` rather than the live render output.

### GAP-2: Intra-tick polling race (LOW)

The drift check polls `autoplay/status` once per tick (at tick start). An OS-level window resize occurring **after** the status poll but **before** the action dispatch in the same tick would pass the drift check while dispatching stale coordinates. At 10 Hz tick rate, the exposure window is ~100ms per tick.

In Run 3, the resize began at tick 115 (drift detected in status), which is why the drift check would abort at that tick under PROMPT-1880. The race is only relevant if the resize starts and completes within the same ~100ms window where status was already polled.

**Severity**: LOW. The 100ms window is very narrow. Not actionable without moving to a polling-free architecture.

### GAP-3: Win32 frozen capture invalidates visual QA evidence (MEDIUM)

- **Run 2**: All 15 win32 captures have identical pixel hash (`0x26207c4c`). The BitBlt fallback (PROMPT-1813) was not yet in the production checkout when Run 2 ran. The result: every win32 capture shows the same frozen frame. Game progression is invisible from the win32 evidence.
- **Run 3**: Win32 captures are frozen from tick 51 onward (hash `0874d30f`). BitBlt fallback correctly triggers and provides valid captures from tick 51+. The Bevy client-side `screenshots/` folder always has valid evidence.

**Impact**: Win32 evidence quality is unreliable without the BitBlt fallback (PROMPT-1813+). Under current code (PROMPT-1880), both BitBlt fallback AND frozen detection are active; this gap is mitigated for future runs.

**Remaining concern**: Even in Run 3, the frozen win32 from tick 51–137 (stable at `0874d30f`) matches the pre-resize game state. After the resize, win32 shows a different hash starting at tick 138 (1296×1115 captures). The BitBlt evidence does show the resized window. So post-1818 code provides valid visual coverage via BitBlt.

### GAP-4: Placement drag arc clips through board area at wrong height (MEDIUM)

The placement drag in Run 3 (ticks 148–155) was computed at build time for window height 720px:
- Drag source (HAND_FIRST_CARD): frac (0.35, 0.92) → y=662 at 720h
- Drag dest (BOARD_FIRST_CELL): frac (0.50, 0.55) → y=396 at 720h

At 1076h window:
- Actual hand strip: y ≈ 0.92 × 1076 = **990px**
- Actual board center: y ≈ 0.55 × 1076 = **592px**

The drag arc went from y=662 (empty area, below the board but above the hand strip) to y=396 (upper board area). Neither endpoint hit the intended UI elements. The Bevy game still advanced to resolution because the bot player's own actions filled the round, but the drag was a complete no-op.

**This is the exact user-reported symptom**: "clicking into empty/offscreen areas because the game window was too small and UI was not fully visible."

Under PROMPT-1880, the drift guard aborts before this drag is ever attempted. This gap is **already closed** in current code.

---

## 6. P0 Fixes Required Before Autoplay Can Be Trusted as QA

| Priority | ID | Description | Status | Artifact path |
|----------|----|-------------|--------|--------------|
| P0-1 | FIXED | Mid-run window resize bypasses coordinate guard → stale clicks | **Fixed in PROMPT-1880** (drift check) | `autoplay-runs/20260528-090613-Z/driver-timeline.jsonl` ticks 115–167 |
| P0-2 | FIXED | Build-time window below minimum silently accepted | **Fixed in PROMPT-1880** | — |
| P0-3 | OPEN | Bevy `phase_label` / `client_state_label` always "Lobby" — client never reports in-session phases | **Needs fix in `client/src/autoplay/`** | All runs: `status.json` + all 262 `driver-timeline.jsonl` rows |
| P0-4 | OPEN | Bevy client screenshots are frozen (identical bytes) — no visual phase progression captured | **Needs investigation; may be same root cause as P0-3** | `autoplay-runs/20260528-051148-Z/screenshots/` (all 86,080 B); Run 2 all 86,148 B; Run 3 first 8 identical |
| P1-1 | OPEN | Win32 frozen captures for Run 2 (no BitBlt fallback) — all 15 win32 PNGs identical hash | **Mitigated in code ≥ PROMPT-1818; production checkout must be updated** | `autoplay-runs/20260528-063609-Z/win32_tick_*.png` (all hash=0x26207c4c) |
| P1-2 | LOW | Intra-tick polling race: resize between status poll and action dispatch cannot be caught | **Not actionable; inherent to polling** | — |

**Verification gate**: Before trusting autoplay runs as QA evidence, confirm:

1. `driver.log` contains `build_win=(1280x720)` — confirms PROMPT-1880+ driver is active.
2. `checkpoints.jsonl` has no `viewport_drift` kind entries — no mid-run resize occurred.
3. `phase_label` in timeline rows progresses through Lobby → ClassSelect → Shop → Auction → Placement → Resolution — verifies phase tracking is fixed.
4. At least one bitblt capture per checkpoint — confirms frozen detection is active.

---

## 7. Summary

The user-observed symptom ("autoplay moves mouse and clicks empty/offscreen areas when the game window is too small") is confirmed by Run 3 evidence. The direct cause was an OS-level window resize from 720px to 1076px height mid-run, with no drift guard in the PROMPT-1818 driver that ran the affected tests. All three runs preceded the PROMPT-1880 commit by 2–6 hours.

The current code at `origin/main@e7b51e84` (PROMPT-1880) **does prevent this class of bug**: the drift guard aborts the run at tick 115 (window height 505px, drift 215px > 10px tolerance) before any stale-coordinate action is dispatched.

Incorporating PROMPT-2024 truth set: the Bevy client screenshots are frozen (identical bytes) across all 15 checkpoints in Runs 1&2, and through the first 8 checkpoints of Run 3. The `phase_label` is "Lobby" for all 262 ticks in all three runs. These are **P0 open issues** that undermine the entire visual QA value of autoplay runs regardless of coordinate correctness.

---

2027: AUTOPLAY-CLICK-WINDOW-METADATA-AUDIT: COMPLETE
