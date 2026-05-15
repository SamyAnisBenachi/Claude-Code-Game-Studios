# Manual Browser/WASM Capture Instructions — `S11-UX-AUCTION-FEATURED-CARD`

> **Companion to**: `evidence.md` in this directory.
> **Story**: `production/epics/shop-auction-ui/story-016-auction-featured-card.md`
> **AC**: AC7 (BLOCKING).

This document lists the exact steps the capturer must execute to
produce the two browser/WASM screenshots story 016 AC7 requires. The
PROMPT 928 worker ran in a headless environment without browser
rendering capability, so the capture step cannot be performed by the
implementation worker. The Node-intent invariants AC1 / AC2 / AC3 / AC4
already verified geometrically constrain what these screenshots will
exhibit.

---

## §1 Required environment

| Component | Required |
|-----------|----------|
| OS | Windows 11 Pro (matches the Sprint 14 capture baseline). |
| Browser | Chromium-based (Chrome / Edge) at the canonical Sprint 14 versions; same browser used for prior Sprint 14 captures. |
| Trunk dev server | `trunk serve --release` at the project root, listening on `http://127.0.0.1:8080`. Release build keeps the WASM bundle under the 50 MB budget per `.claude/docs/technical-preferences.md`. |
| Game server | `cargo run -p server --release` running locally so the WASM client can connect; same `bevy_lightyear` WebSocket transport as production. |
| Viewport sizes | **1920×1080** (canonical baseline reference) AND **1366×768** (minimum supported). Use Chromium devtools "Device Toolbar" to lock the viewport exactly. |
| Source-of-truth commit | The worker-branch commit at PROMPT 928 push (recorded in the final worker report). |

---

## §2 Setup steps

1. Pull the worker branch:
   ```bash
   git fetch origin work/s14-auction-featured-card
   git checkout work/s14-auction-featured-card
   ```
2. Build the WASM client:
   ```bash
   # In a powershell terminal at the worktree root
   $env:CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'
   $env:CARGO_PROFILE_DEV_DEBUG='0'
   $env:CARGO_PROFILE_TEST_DEBUG='0'
   $env:CARGO_INCREMENTAL='0'
   $env:RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'
   trunk serve --release --no-default-features
   ```
3. In a second terminal, run the server:
   ```bash
   cargo run -p server --release
   ```
4. Open two Chromium browser windows pointed at `http://127.0.0.1:8080`
   (one for each player; the auction phase requires a 2-player room).

---

## §3 In-game reach point: `DRAFT_AUCTION` panel active

Story 016 AC7 captures the `DRAFT_AUCTION` panel during an **Active**
auction. Reach it by:

1. Both players connect to the lobby; create a room (Player A) and
   join via the room-code chip (Player B).
2. Both players pick a class and lock in (`S2CClassLocked` →
   `S2CRoomReady`).
3. Both players ready up. The session enters `DraftInitial`.
4. During `DraftInitial`, both players ready up again (the player can
   click "Ready" early or wait the 45-second timer).
5. The session enters `DraftAuction`. The auction panel renders the
   featured card centered, with the ACCENT-colored frame visible, and
   the bid cluster sitting at the panel bottom.
6. Verify Player A or Player B sees:
   - Status text near panel top: "Auction starting..." then "Auction
     live".
   - Timer bar near panel top (green, animated countdown).
   - Featured card framed in `#F2C94C` ACCENT yellow, centered.
   - Three bid buttons at panel bottom, all enabled (assuming
     starting-price affordability).
   - Bid status text below bid buttons.

---

## §4 Capture #1 — `auction-featured-1920x1080-active.png`

1. Resize the Player A browser to 1920×1080 (Chromium devtools "Device
   Toolbar" → "Edit" → "Add custom device" → preset width=1920,
   height=1080, DPR=1).
2. Reach the `DRAFT_AUCTION` panel via §3.
3. Wait until the timer bar shows a partial fill (between 50% and 80%
   remaining) so the timer state is visibly mid-countdown.
4. Press F12 → "Capture full size screenshot" (Chromium command palette
   → `Capture full size screenshot`).
5. Crop / save the PNG to exactly the 1920×1080 viewport rectangle (no
   browser chrome).
6. Save as
   `production/qa/evidence/sprint-14-auction-featured-card/auction-featured-1920x1080-active.png`.

### Required visible elements in the capture

- ✓ Featured card visibly dominant (largest single element inside the
  auction panel).
- ✓ ACCENT-colored frame border around the featured card.
- ✓ Featured card centered horizontally and vertically within the
  auction panel.
- ✓ Bid cluster visible (three bid buttons + bid status text) below
  the featured card.
- ✓ Timer bar visible (above the featured card).
- ✓ Gold counters visible on the HUD top strip (NOT occluded by the
  auction panel).
- ✓ Hand-tray visible (NOT occluded by the auction panel) at the
  bottom of the viewport.
- ✓ HUD primary readouts (own gold, opponent gold) visible at the top
  of the viewport.

---

## §5 Capture #2 — `auction-featured-1366x768-active.png`

Repeat §4 with the Chromium devtools viewport locked to **1366×768**.
Save as
`production/qa/evidence/sprint-14-auction-featured-card/auction-featured-1366x768-active.png`.

The same required visible elements (§4) must hold. Specifically verify
non-occlusion at the smaller minimum-supported viewport (less vertical
real-estate; the featured card vertical span 134-414 px in panel coords
leaves a 62-px clearance to the bid cluster — this is intentional and
should appear comfortable in the capture).

---

## §6 Cross-check: AC6 Story 013 card-text readability evidence

Story 013's readability evidence was captured against the hand / shop /
draft card surfaces. Story 016 does not change those surfaces. After
the two §4 / §5 captures land, perform a one-glance visual sanity check
on the featured-card name / stat / keyword text:

- Card name at `H1 = 30 px`: legible at both viewports without
  squinting.
- Card stat-line / rarity text (`H2 = 22 px` semantic, rendered on the
  parent Text in this story for backward-compat): legible.
- No silent ellipsis on the name (longest name in the test catalog is
  `"Card N"` — well below the line-break threshold; production card
  names with longer text should also fit because card-name length is
  bounded by the card-pool authoring spec).

Record any concerns in §7 below.

---

## §7 Capturer log

Once captures land, fill in the table:

| Capture | Captured at | Captured by | Notes |
|---------|-------------|-------------|-------|
| `auction-featured-1920x1080-active.png` | YYYY-MM-DD | (capturer name + role) | (text-fit, sibling-overlap, non-occlusion observations) |
| `auction-featured-1366x768-active.png`  | YYYY-MM-DD | (capturer name + role) | (text-fit, sibling-overlap, non-occlusion observations) |

The `/story-done` paperwork worker will flip story 016's AC7 / AC8
checkboxes once both captures land and the capturer log is filled in.
