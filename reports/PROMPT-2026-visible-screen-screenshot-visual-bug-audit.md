# PROMPT-2026 â€” Visible Screen Screenshot Visual Bug Audit (v2 â€” PROMPT-2024 truth set)

**Date**: 2026-05-28 (v2 refresh post PROMPT-2024 landed)
**Source**: origin/main@e7b51e84 (post PROMPT 2019 mainland) â€” same build as PROMPT-2024
**PROMPT-2024 forensic report**: `reports/PROMPT-2024-forensic-evidence-inventory-and-run-selection.md`
**Autoplay runs inspected**:
- `20260528-051148-Z` â€” 15 screenshots + driver-timeline (262 ticks)
- `20260528-063609-Z` â€” 15 screenshots + 15 win32_tick PNGs + driver-timeline (262 ticks)
- `20260528-090613-Z` â€” 15 screenshots + 15 win32_tick PNGs + 11 bitblt_tick PNGs + driver-timeline (262 ticks)

**Baseline captures inspected** (older, `production/qa/evidence/captures/`):
- `board-rendering-baseline-1920x1080.png`
- `shop-auction-ui-auction-bid-target-focus/sau-011-bidding-1366x768.png`
- `qa-cond-0007-hand-ui/01-normal-placement-timer.png`
- Various hand-ui-placement-staged-disclosure PNGs

---

## Artifact Quality Assessment

### Screenshot capture mechanism â€” FUNCTIONING (with caveats)

- The `autoplay/screenshot` API correctly captures the game client window.
- win32_tick images (BitBlt of full native window including title bar) and bitblt_tick
  images (desktop BitBlt fallback) both agree with the Bevy-internal screenshots.
- All three capture methods show identical content â€” confirming the capture is live,
  not stale/frozen.
- **Capture artifact noted**: bitblt_tick images show background windows (code editor)
  peeking around the edges of the game window, indicating the game window is not
  always topmost. This is a harness-level concern, not a game visual bug.
- **Window resize mid-run**: the window starts at `[1280, 720]` (tick 1) and the
  status.json at run end shows `[1280, 1076]`. Screenshots at ticks 000000â€“000022
  have a different viewport crop than ticks 000037+. This is the autoplay harness
  resizing the window during the run.

---

## Screenshot-by-Screenshot Findings

All 15 checkpoint screenshots from the latest run (20260528-090613-Z):

| Seq | Checkpoint | Phase at capture | What is visible |
|-----|-----------|-----------------|-----------------|
| 000000 | lobby-loaded | Lobby | Lobby screen â€” pre-room state, 1280Ã—720 |
| 000007 | bot-added | Lobby | **Identical to 000000** â€” no visible change |
| 000011 | lobby-confirmed | Lobby | **Identical to 000000** â€” no visible change |
| 000013 | class-select-loaded | Lobby | **Identical to 000000** â€” no visible change |
| 000020 | class-confirmed | Lobby | **Identical to 000000** â€” no visible change |
| 000022 | shop-loaded | Lobby | **Identical to 000000** â€” lobby visible, shop invisible |
| 000026 | shop-slot-clicked | Lobby | **Identical to 000022** â€” lobby still visible |
| 000030 | auction-loaded | Lobby | **Identical to 000022** â€” lobby still visible |
| 000037 | auction-ready | Lobby | Window resized to ~1280Ã—1076; **lobby still visible**, now with class picker expanded showing Iop/Cra/Sacrier/Xelor/Ecaflip/Sadida/Neutral row + opponent slot |
| 000039 | placement-loaded | Lobby | **Identical to 000037** â€” board/hand invisible |
| 000048 | placement-dragged | Lobby | **Identical to 000037** â€” drag had no visual effect |
| 000052 | placement-submitted | Lobby | **Identical to 000037** â€” submission had no visual effect |
| 000054 | resolution-started | Lobby | **Identical to 000037** â€” resolution invisible |
| 000055 | resolution-complete | Lobby | **Identical to 000037** â€” no result screen shown |
| 000057 | vs-bot-post-resolution | Lobby | **Identical to 000037** â€” lobby persists after full round |

**The client renders only one screen across all 15 checkpoints and all 262 driver ticks.**

---

## Root Cause Confirmation (via driver-timeline.jsonl)

```
grep '"phase_label"' driver-timeline.jsonl | sort | uniq -c
â†’  262  "phase_label":"Lobby"

grep '"client_state_label"' driver-timeline.jsonl | sort | uniq -c
â†’  262  "client_state_label":"Lobby"
```

Identical result for all three runs (051148-Z, 063609-Z, 090613-Z).

**The client `ClientState` machine never transitions from `Lobby` to `InSession`.**
The autoplay driver successfully reaches checkpoints via ECS/server-side signals (the
bot, server, and protocol are all progressing), but the *client* visual layer stays
in `Lobby` state. Every in-game screen (shop, auction, board/placement, resolution,
result) is unreachable in the current build.

---

## Detailed Visual Bug Catalogue

### BUG-01 â€” CRITICAL: No in-game screen ever renders
**Impact**: All gameplay phases invisible. Player sees lobby for entire game.
**Evidence**: All 15 checkpoint screenshots across 3 runs; driver-timeline confirms
`client_state_label:"Lobby"` for all 262 ticks in every run.
**Checkpoints never showing their screen**: shop-loaded, auction-loaded, auction-ready,
placement-loaded, placement-dragged, placement-submitted, resolution-started,
resolution-complete, vs-bot-post-resolution.
**Expected**: At `shop-loaded`, the shop/draft card grid should be visible. At
`auction-loaded`, the auction panel with bid controls should appear. At
`placement-loaded`, the 5Ã—8 board grid with hand cards at bottom should appear.

---

### BUG-02 â€” HIGH: Class card art renders as solid black
**Screens affected**: Lobby class picker row, selected-class preview card.
**Evidence**: win32_tick_000185.png, win32_tick_000259.png, screenshots/000037.png
and all subsequent screenshots showing the expanded class picker.
**Visible symptom**: All 7 class buttons (Iop, Cra, Sacrier, Xelor, Ecaflip, Sadida,
Neutral) show a solid black rectangle as the card face. Only the colored element gem
in the top-right corner is rendered. No character art, no card border art.
The large "Selected: Iop" preview card on the left also shows a black rectangle.
**Expected**: Character sprite or proxy art should fill the card face area.

---

### BUG-03 â€” HIGH: Room code never populates, player count never updates
**Screens affected**: Lobby header bar.
**Evidence**: All 15 screenshots â€” header shows `Room: ----` and `Players: 0/1`
throughout the entire run, including after `bot-added` and `lobby-confirmed`
checkpoints.
**Expected**: After CreateRoom, `Room:` should show the 4-character room code.
After `bot-added`, `Players:` should increment (0/1 â†’ 1/1 or 2/2 depending on
counting convention). After `lobby-confirmed`, the class status `not confirmed`
should update.

---

### BUG-04 â€” HIGH: Class confirmation status "not confirmed" never clears
**Screens affected**: Lobby header, bottom slot strip.
**Evidence**: Every screenshot â€” `Class: Iop  â–¡  not confirmed` persists even after
`class-confirmed` checkpoint fires.
**Visible symptom**: Bottom status strip shows `You - Iop - slot 1 - not confirmed`
with a highlighted bracket on "not confirmed" throughout the post-class-select phase.
**Expected**: After confirmation, should show "confirmed" or remove the status entirely.

---

### BUG-05 â€” MEDIUM: Broken separator characters in lobby header
**Screens affected**: Lobby header (line 1 and 2).
**Evidence**: All screenshots. Header text reads:
`Connected as player 9  â–¡  Room: ----  â–¡  Players: 0/1`
`Slot 1  â–¡  Class: Iop  â–¡  not confirmed`
The `â–¡` characters appear to be Unicode replacement boxes â€” the separator glyphs
are not rendering (likely a missing glyph in the monospace font used by bevy_ui text).
**Expected**: Clean separator characters (e.g. `|` or `â€¢`) should display without
rendering as tofu boxes.

---

### BUG-06 â€” MEDIUM: Class picker row clips the Neutral card
**Screens affected**: Lobby class picker row (visible in 000037 onward when expanded).
**Evidence**: Screenshots 000037, 000039, 000048, 000052, 000054, 000055, 000057,
win32_tick_000185, win32_tick_000259.
**Visible symptom**: The 7th class card (Neutral) is half-visible at the right edge
of the card row. The label "Neutral" shows, but the card art area is cut off.
The row overflows its container width.
**Expected**: All 7 class cards should fit within the panel, either by scrolling or
by making the panel wide enough.

---

### BUG-07 â€” MEDIUM: Game content does not fill window â€” large dark margins
**Screens affected**: All win32_tick and bitblt_tick captures.
**Evidence**: win32_tick_000185.png (1165Ã—900 usable area), win32_tick_000259.png.
At the 1280Ã—1076 window size, the lobby panel is ~790px wide and ~870px tall,
centered, leaving ~245px dark unused space on each horizontal side and ~100px
on top/bottom.
**Expected**: At minimum, the lobby panel should expand to use the available window
width, or the background should be a designed background rather than solid black.

---

### BUG-08 â€” MEDIUM: Board renders as a tiny island â€” no HUD or chrome
**Screens affected**: Board/placement phase (visible only in older baseline captures).
**Evidence**: `captures/board-rendering-baseline-1920x1080.png` â€” the entire 5-lane
board (both player sides + objective column) renders as a ~400Ã—310px block centered
on a 1920Ã—1080 canvas. Approximately 87% of the screen is unused dark background.
No HUD strip, no hand cards, no lane labels, no objective panel is attached.
**Expected**: Board should fill most of the play area with HUD chrome flanking it.

---

### BUG-09 â€” MEDIUM: Auction screen has no card art, sparse layout
**Screens affected**: Auction phase (visible only in baseline captures).
**Evidence**: `captures/shop-auction-ui-auction-bid-target-focus/sau-011-bidding-1366x768.png`.
The auction panel shows card title ("SAU-011 Test Card"), rarity/cost ("Rare - 4g"),
and bid buttons, but no card face art. Three "Locked empty" hand slots at the bottom
have no visual context (no border, background, or label to indicate they are slots).
The auction timer bar spans full width with no break.
**Expected**: Card face art or large artwork should be prominently visible in the
auction panel as the primary focus.

---

### BUG-10 â€” LOW: "Type room code" field renders as plain text, not an input
**Screens affected**: Lobby before room creation.
**Evidence**: Screenshots 000000â€“000030. Field shows:
`Type room code: -------- - idle`
There is no visible text-input styling (cursor, focus border, placeholder styling).
The 8-dash placeholder looks like a debug value, not user-facing copy.
**Expected**: Either a styled input box, or (if the field is intentionally read-only)
cleaner UX copy rather than "--------".

---

### BUG-11 â€” LOW: Snapshot button rendered in game corner (debug artifact)
**Screens affected**: All screenshots.
**Evidence**: Every screenshot shows a "Snapshot" button rendered in the top-right
corner of the game canvas (dark button, white label). This is a debug/QA feature that
should not be visible in standard play.
**Expected**: "Snapshot" button should be gated behind `CCGS_QA_SNAPSHOT=1` env var
or completely hidden from non-debug builds.
**Note**: Per memory reference `reference_ccgs_qa_snapshot_button.md`, this IS the
intended behaviour when `CCGS_QA_SNAPSHOT=1` is set on the client. If the autoplay
harness sets this flag, this is not a bug. Flag this as advisory only.

---

### BUG-12 â€” LOW (capture artifact): bitblt captures show background windows
**Evidence**: bitblt_tick_000051.png shows code editor content visible to the left
and right of the game window.
**Classification**: Capture artifact â€” the BitBlt desktop fallback captures the full
desktop, not just the game window. The game window content itself is correct.
Not a game visual bug.

---

## Top 10 Visual Blockers Ranked by Player Impact

| Rank | ID | Summary | Impact |
|------|-----|---------|--------|
| 1 | BUG-01 | **Client stuck in Lobby â€” zero in-game screens visible** | Total blocker: shop, auction, placement, resolution, result screens are all invisible. The game is unplayable as a visual experience. |
| 2 | BUG-02 | **All class card art is black** | First impression blocker: the lobby class-select (the only visible screen) shows broken art on every class button and preview. Players cannot distinguish classes visually. |
| 3 | BUG-08 | **Board renders as tiny island, no HUD chrome** | Even if BUG-01 is fixed, the board would be illegible at ~400px on a 1080p canvas with no surrounding HUD. |
| 4 | BUG-03 | **Room code / player count never update** | Lobby responsiveness completely broken â€” no feedback that room creation or bot join succeeded. |
| 5 | BUG-04 | **"not confirmed" status never clears** | Confirms lobby state machine is not visually updating. Player sees conflicting state. |
| 6 | BUG-07 | **Game content doesn't fill window** | At any window size above 1280Ã—720, large black margins appear. Feels unpolished and crops the UI on wider displays. |
| 7 | BUG-09 | **Auction card art absent, layout bare** | Auction phase has no visual identity â€” bids are made on a text label, no art. Key engagement moment is unimpressive. |
| 8 | BUG-06 | **Class picker clips the Neutral card** | Neutral class is not fully selectable; row overflows. |
| 9 | BUG-05 | **Broken separator glyphs in header** | Debug/tofu appearance throughout lobby header undermines polish. |
| 10 | BUG-10 | **Room code input is unstyled with debug placeholder** | Minor UX roughness but directly player-facing. |

---

## Summary

The single most critical finding is **BUG-01**: the client `ClientState` machine never
transitions from `Lobby` to `InSession`. Every in-game screen (shop, auction,
board/placement, resolution, result) is unreachable. The driver confirms this via
`client_state_label: "Lobby"` for all 262 ticks across three independent runs today
(2026-05-28). The server side and bot logic are progressing (checkpoints fire), but
the client visual layer does not follow.

All other bugs are real and should be fixed, but BUG-01 makes the rest moot until
the `Lobby â†’ InSession` transition is restored.

The older baseline captures (`qa-cond-0007-hand-ui/`, `board-rendering-baseline-*`,
`sau-011-*`) show that in-game screens exist in the codebase and were functional at
an earlier commit, confirming this is a regression rather than missing implementation.

---

---

## PROMPT-2024 Truth-Set Additions (v2)

### Frozen-frame forensics (driver.log â€” run 20260528-090613-Z)

PrintWindow reports FROZEN for **every** bitblt tick, not just the later ones:

| Ticks | Window size | Frozen hash | Meaning |
|-------|------------|-------------|---------|
| 51, 72, 81, 93, 113 | 1296Ã—759 | `0874d30f35473105db8c06eb94943d77` | Pre-resize frozen lobby frame |
| 147, 164, 176, 185, 250, 259 | 1296Ã—1115 | `ca2ab3e8456d5f81d2fc5f3f0c5703f2` | Post-resize frozen lobby frame |

Win32 PrintWindow captures the same `pixel_hash=0xb4db8636` at 1296Ã—759 **and** the same
`pixel_hash=0xb4db8636` at 1296Ã—1115 â€” a single frame hash, both window sizes. Bevy is
outputting exactly one render frame for the entire 40-second run.

BitBlt fallback pixel hashes (6 unique values across 11 ticks) vary due to cursor position
and background window content on the desktop, **not** game phase changes.
`bitblt_tick_000164.png` (placement-dragged, 1296Ã—1115) visually confirms: **lobby
class-select screen with class picker row and "not confirmed" strip â€” identical content
to all other frames.**

**GAP-06 from PROMPT-2024 is now CLOSED**: BitBlt frames contain no undiscovered
game-phase content. The lobby is the only screen ever rendered.

### Server-side context (bot-qa-snapshots, 2026-05-27)

PROMPT-2024 BUG-002: Server `disconnect_trackers` shows
`seconds_since_disconnect: 29991` (~8.3 h) for both players from the very first
Placement phase snapshot. This is likely the root cause of BUG-01 (client stuck in
Lobby): if the server considers both players permanently disconnected, it may suppress
`S2CPhaseChanged` delivery, preventing the client `ClientState` machine from ever
receiving the signal needed to transition `Lobby â†’ InSession`.

PROMPT-2024 BUG-003: Bot submits `empty_placement_failsafe` every round
(legal_action_count=0). Board `minion_count=0` in all snapshots. This means even if
BUG-01 is fixed, the placement and combat phases will show empty boards.

---

## P0 / P1 Issue Register (against PROMPT-2024 truth set)

### P0 â€” Total playability blockers

| ID | Issue | Exact screenshot paths | Evidence |
|----|-------|----------------------|----------|
| P0-01 | **Client never exits Lobby â€” zero in-game screens visible** | `autoplay-runs/20260528-090613-Z/screenshots/000022.png` (shop-loaded, still lobby) Â· `000039.png` (placement-loaded, still lobby) Â· `000054.png` (resolution-started, still lobby) Â· `bitblt_tick_000164.png` (placement-dragged, still lobby) | All 262 driver-timeline ticks: `client_state_label:"Lobby"`. Frozen hash `0xb4db8636` constant across all win32 captures. |
| P0-02 | **Bevy renderer outputs a single frozen frame the entire session** | `autoplay-runs/20260528-090613-Z/win32_tick_000005.png` (hash `0xb4db8636`) through `win32_tick_000259.png` (hash `0xb4db8636`) | All 15 win32_tick PNGs share the same PrintWindow pixel hash. Lobby state machine in Bevy is not advancing `ClientState`. Root cause likely the `disconnect_tracker` bug (P2024 BUG-002). |

### P1 â€” High-severity bugs visible on the only reachable screen (Lobby)

| ID | Issue | Exact screenshot paths | Symptom |
|----|-------|----------------------|---------|
| P1-01 | **All class card art solid black â€” no character sprites** | `screenshots/000037.png` Â· `screenshots/000039.png` Â· `bitblt_tick_000164.png` Â· `bitblt_tick_000176.png` | All 7 class buttons (Iop, Cra, Sacrier, Xelor, Ecaflip, Sadida, Neutral) and the large preview card show pitch-black art panels. Only colored element gems in corners are visible. |
| P1-02 | **Lobby header state never updates after any action** | `screenshots/000007.png` (bot-added â€” still `Players: 0/1`) Â· `screenshots/000020.png` (class-confirmed â€” still `not confirmed`) Â· `screenshots/000030.png` (auction-loaded â€” `Room: ----`) | Room code, player count, and confirmation status are frozen at their initial values for the entire run. |
| P1-03 | **Neutral class card clipped at right edge of picker row** | `screenshots/000037.png` Â· `bitblt_tick_000185.png` | 7th class card is partially outside the container. Label "Neutral" visible but card body is cut off. All 6 prior cards fit; the row overflows by ~1 card width. |

---

## Capture Method Reliability Summary (for future workers)

| Method | Ticks | Reliability | Notes |
|--------|-------|------------|-------|
| Bevy `autoplay/screenshot` API | checkpoint-triggered (15) | FROZEN (8 of 15 pre-resize; 0 of 7 post-resize varied) | Pre-resize screenshots 000000â€“000030 are byte-identical (86,108 B). Post-resize 000037â€“000057 are byte-identical at larger size (117,843 B). Neither set shows frame-level animation. |
| Win32 PrintWindow | 15 | ALL FROZEN â€” `pixel_hash=0xb4db8636` constant | Every tick same hash. BitBlt fallback triggered for 11 of 15. |
| Desktop BitBlt fallback | 11 | LIVE desktop â€” varying hashes | Hash variation from cursor/background, not game phases. Visually confirms lobby only. Most reliable for confirming what the window actually shows. |

---

2026: VISIBLE-SCREEN-VISUAL-BUG-AUDIT: COMPLETE
