# Hand UI

> **Status**: In Design
> **Author**: SamyAnisBenachi + Claude Code agents
> **Last Updated**: 2026-04-29
> **Implements Pillar**: No idle spectating · Simple surface

## Overview

Hand UI is the client-side card fan display and interaction layer through which players access and play their hand across all game phases. It subscribes to the server-authoritative hand state (a `Vec<CardId>` of up to 10 cards, delivered via unicast network messages) and renders those cards as an interactive fan at the bottom of the screen. Its behavior is phase-driven: during DRAFT_INITIAL it presents the 9-card offering for click-to-buy selection within the 45-second, 5-gold budget window; during DRAFT_SHOP and DRAFT_AUCTION it displays the current hand alongside the shop, enabling instant-effect card activation via `C2SActivateCard`; during PLACEMENT it enters the game's highest-tension state — 10 seconds for the player to select cards, assign each a PlayTarget (board cell, target unit, target objective, lane-wide, or instant), optionally split mana from the reserve pool, and submit the full batch via `C2SSubmitPlacement` as a single atomic commit with no retraction. During RESOLUTION, Hand UI fully suppresses all interaction and becomes invisible — the board is the sole display. Hand UI owns the hand's visual state and the card-play interaction chain; it does not own shop slot display, auction bidding, or board and unit rendering — those belong to Shop/Auction UI and Board Rendering respectively.

## Player Fantasy

The hand is a war map, and you are the commander who lays the pieces.

Hand UI serves the fantasy of **decisive authority under a hard clock**. The player who picks up this game on round 1 does not need to understand it fully; they see cards, they place a card, the board responds. But the player who has played twenty rounds knows a different experience: PLACEMENT opens, the 10-second timer begins, and the hand fan expands to show every committed card. In six seconds the read is complete — Iop to lane 3, Eniripsa behind it, Xelor split 2 reserve — and the last four seconds are confirmation, not discovery. Submit. The hand collapses. The board inherits everything.

**The four feelings Hand UI is responsible for delivering:**

1. **"I placed, I committed, I moved on."** — The batch commit is not a limitation; it is the signature of the system. The player should feel that submitting a clean placement plan — with no hedging, no take-backs — is a skill expression, not a surrender to a design constraint. The UI must make that decisiveness feel good to execute, not effortful.

2. **"I can read my hand in two seconds."** — Card layout, cost legibility, targeting affordances, and the mana split indicator for reserve cards must be parseable at a glance. The remaining eight seconds of the placement window belong to the *board*, not the *hand*. If the player is spending their time navigating the hand rather than reading the opponent's field, the UI has failed them.

3. **"Every card has a destination before the timer ends."** — By the third game, a player should be setting the table before PLACEMENT opens — knowing which cards they intend to play and to which lanes while still in DRAFT. Hand UI reinforces this mental pre-planning by being fast to execute, never fighting the player's intent with awkward targeting flows or accidental selections.

4. **"I was not confused during RESOLUTION."** — The hand's complete disappearance during RESOLUTION is not a loss of the interface; it is the *resolution* of the fantasy. The player placed, the board accepted, now the board plays. Watching the lanes resolve is the consequence of the hand's clarity, not a separate event.

**What Hand UI must NOT feel like:**
- A fidget toy — no rearranging for comfort, no shuffling for inspiration; the fan layout is the commitment surface
- A safety net — no "are you sure?" modal between selection and submission; the irrevocable commit is a feature
- A panic surface — 10 seconds at 60 FPS must feel tight but controlled; animations cannot slow the flow
- A tutorial moment — nothing in the hand fan should require a hover or tooltip to understand in play

## Detailed Design

### Core Rules

**Rule 1 — Hand Fan Layout**
The hand fan renders the player's hand as a row of up to 10 cards at the bottom of the screen. Cards use absolute positioning with per-card rotation angles (±10° arc spread) — the fan layout is not flexbox-driven. All 10 card slots are spawned at session start (pre-pooled); cards are shown/hidden per hand state, not spawned/despawned per round, to eliminate WASM GC spikes. Card art uses a TextureAtlas for GPU batching.

**Rule 2 — Hand State**
Hand UI subscribes to server-delivered hand state exclusively. The client tracks hand count from confirmed S2C messages only. The client never asserts hand state — it sends C2S intent messages and waits for server confirmation before updating any visual.

**Rule 3 — Phase Behavior (State Matrix)**

| RSM Phase | Hand UI Mode | Input |
|---|---|---|
| LOBBY | HIDDEN — all elements invisible | None |
| DRAFT_INITIAL | GRID — 3×3 offering overlay visible; fan below (empty at start) | Click-to-buy cards in grid |
| DRAFT_SHOP | PASSIVE — fan visible | C2SActivateCard for Instant cards; no drag |
| DRAFT_AUCTION | PASSIVE, read-only — fan visible | None — auction UI owns input focus |
| PLACEMENT | STAGING — fan in drag-and-stage mode; Submit button; timer visible | Drag-to-stage; Submit |
| RESOLUTION | HIDDEN — all elements invisible immediately | None |

**Rule 4 — DRAFT_INITIAL Grid**
On DRAFT_INITIAL entry, a 3×3 grid overlay appears centered on screen. Each cell shows: card art, name, mana cost, rarity indicator.

- Click sends C2SPurchaseCard. The client does not optimistically remove the card — it waits for server confirmation.
- On confirmation: the purchased slot empties and the card animates to the fan (slide, ≤300ms). Grid stays open.
- When local hand count reaches 10: all remaining grid cards enter a muted/locked visual state; click events are suppressed client-side; a "Hand full" notification appears near the fan for 2 seconds.
- The grid panel does not close until DRAFT_INITIAL ends (timer or all-ready).

**Rule 5 — DRAFT_SHOP: Instant Card Activation**
During DRAFT_SHOP, Instant-type cards in the hand fan can be activated with a single click. Click sends C2SActivateCard. No drag gesture. No board target. Non-Instant cards are display-only in the hand fan during DRAFT_SHOP (purchasing happens from shop slots, owned by Shop/Auction UI).

**Rule 6 — PLACEMENT: Drag-to-Stage**
On PLACEMENT entry: the Submit button appears immediately ("Submit (0 cards)"), and the timer starts. Staging flow:

1. **Drag-start** — mouse-down on a hand card: the UI card entity hides (Visibility::Hidden); a world-space drag sprite (card art clone) appears at the cursor's world position. The fan slot retains a dimmed, non-interactive ghost to preserve fan layout stability.
2. **Drag over board** — each frame, the cursor screen position converts to world coordinates via BoardLayout. Valid targets highlight per card type:
   - **BoardCell** (Minion/Trap/Structure): player's valid spawn cells for this round, minus cells occupied by prior-round board units, minus cells already targeted by staged Minions in the pending queue. Only valid cells highlight — occupied cells are simply absent from the highlight set (not shown as invalid).
   - **TargetUnit** (targeted spell): valid target units highlight. If no valid units exist on the board, the board shows a full-dim "no valid targets" overlay; no cells highlight; drag-release anywhere returns the card to hand.
   - **TargetObj** (objective spell): the opponent's 5 objective cells (one per lane) highlight.
   - **LaneWide** (Field card): the full column of each lane highlights. Drop anywhere within a lane column resolves to `LaneWide { lane }`.
   - **Instant**: see Rule 7.
3. **Drop on valid target** — drag sprite clears; card stages to that target. Board Rendering receives a GhostPlacementChanged message and renders the board ghost. Fan ghost dims further (desaturated). Submit count increments: "Submit (N cards)".
4. **Drop on invalid target or outside board** — drag sprite despawns; UI card entity reappears in its original fan slot; fan ghost clears.

**Rule 7 — PLACEMENT: Instant Card Staging**
When an Instant card is dragged from the fan during PLACEMENT:
- Board cells do not highlight (no board target exists).
- The hand fan's background plate highlights as the valid drop zone.
- Drop on the plate: card stages as Instant (no board position). Fan ghost dims. Submit count increments.
- Drop outside the plate: drag cancels; card returns to fan slot.

**Rule 8 — Un-Staging**
To un-stage a staged card: click its board ghost (rendered by Board Rendering) or drag the board ghost back to the hand fan zone. Either action: removes the card from the pending queue; fires GhostPlacementChanged to clear the board ghost; restores the fan slot to full opacity; decrements Submit count.

**Rule 9 — Timer Expiry During Active Drag**
If the PLACEMENT timer reaches 0 while a card is mid-drag (lifted from fan, not yet dropped): the drag cancels; the drag sprite despawns; the in-flight card returns to its fan slot (fan ghost clears; card entity reappears). C2SSubmitPlacement fires with only the previously staged cards — the in-flight card is NOT included in the submission.

**Rule 10 — Submit**
The Submit button is active from PLACEMENT entry regardless of staged count. Pressing Submit:
1. Fires C2SSubmitPlacement with all staged placements.
2. Submit button immediately becomes inactive ("Submitted") — prevents double-submit.
3. No confirmation modal. Irrevocable.
4. Fan cards dim slightly; staging interaction locked.
The player may still watch the timer and board after submitting. Hand UI does not hide on submit — only on RESOLUTION entry.

**Rule 11 — PLACEMENT Timer Display**
- Shows whole seconds (not milliseconds); large enough to read peripherally.
- At 5 seconds remaining: timer shifts visual state (color change — Art Director defines specifics).
- Timer expiry fires a single audio cue (not repeating ticks — see Audio Requirements).

**Rule 12 — RESOLUTION Hide**
On RESOLUTION entry: all Hand UI elements hide immediately (Visibility::Hidden). No exit animation. On RESOLUTION exit to DRAFT: Hand UI restores visibility with the updated hand state from the completed round.

---

### States and Transitions

| State | RSM Phase(s) | Description |
|---|---|---|
| `HIDDEN` | LOBBY, RESOLUTION | All fan elements hidden. No input. |
| `GRID` | DRAFT_INITIAL | 9-card grid overlay. Click-to-buy. Fan below (read-only). |
| `PASSIVE` | DRAFT_SHOP | Fan visible. Instant cards clickable (C2SActivateCard). |
| `PASSIVE_LOCKED` | DRAFT_AUCTION | Fan visible, read-only. All input suppressed. |
| `STAGING` | PLACEMENT | Drag-to-stage active. Submit button visible. Timer visible. |
| `SUBMITTED` | PLACEMENT (post-submit) | Fan dimmed. Submit button inactive ("Submitted"). Timer still visible. |

```
HIDDEN → GRID              on RSM: DRAFT_INITIAL entry
GRID → HIDDEN              on RSM: DRAFT_INITIAL exits (grid dismissed)
HIDDEN → STAGING           on RSM: PLACEMENT entry
STAGING → SUBMITTED        on: C2SSubmitPlacement sent
SUBMITTED → HIDDEN         on RSM: PLACEMENT → RESOLUTION
HIDDEN → PASSIVE           on RSM: RESOLUTION → DRAFT_SHOP (non-auction round)
HIDDEN → PASSIVE_LOCKED    on RSM: RESOLUTION → DRAFT_AUCTION (auction round)
PASSIVE_LOCKED → PASSIVE   on RSM: DRAFT_AUCTION → DRAFT_SHOP
PASSIVE → STAGING          on RSM: DRAFT_SHOP → PLACEMENT
STAGING → HIDDEN           on RSM: PLACEMENT → RESOLUTION (timer expiry or all-submit early exit)
```

---

### Interactions with Other Systems

| System | Direction | What flows |
|---|---|---|
| **Network Protocol** | NP → Hand UI | S2CDraftOffering (9 card IDs for grid), S2CCardAcquired (hand additions on purchase), S2CPhaseChanged (drives state transitions) |
| **Network Protocol** | Hand UI → NP | C2SPurchaseCard (DRAFT_INITIAL grid buy), C2SActivateCard (DRAFT_SHOP Instant play), C2SSubmitPlacement (PLACEMENT batch commit) |
| **Round State Machine** | RSM → Hand UI | Phase transitions via S2CPhaseChanged drive Hand UI state machine (Rule 3) |
| **Board Rendering** | Hand UI → Board Rendering | GhostPlacementChanged messages when cards stage/un-stage. Hand UI writes the intent; Board Rendering renders the ghost at the board position. |
| **Board Rendering** | Board Rendering → Hand UI | Reads BoardLayout resource for cursor-to-world-position conversion during PLACEMENT hover detection. |
| **Card Data & Pool** | Pool → Hand UI | Card definitions (name, mana cost, card type, PlayTarget category, TextureAtlas frame index). Read at session start; not polled each frame. |
| **Card Acquisition** | CA → Hand UI | Upstream authority on hand state. Hand UI is read-only downstream — displays CA's server-delivered hand state. CA explicitly excludes itself from visual ownership. |
| **Game Config** | Config → Hand UI | placement_timer_seconds (10s), draft_initial_timer_seconds (45s), draft_shop_timer_seconds (30s). Loaded at session start. |

## Formulas

### Formula 1: Fan Card Screen Position

```
t = (index - (count - 1) / 2.0) / max((count - 1) / 2.0, 1.0)

card_x = fan_center_x + t × fan_half_spread
card_y = fan_base_y + arc_height × t²
```

**Variables:**

| Variable | Symbol | Type | Range | Description |
|---|---|---|---|---|
| Card index (0-based) | `index` | u8 | 0 – count-1 | Position of the card in the hand array |
| Hand size | `count` | u8 | 1 – 10 | Current number of cards in hand |
| Normalized position | `t` | f32 | −1.0 – +1.0 | −1 = leftmost card, +1 = rightmost, 0 = center |
| Fan horizontal center | `fan_center_x` | f32 | screen_width / 2 | Center anchor of the fan (px) |
| Fan vertical baseline | `fan_base_y` | f32 | screen_height − margin | Y-coordinate of the fan bottom edge (px) |
| Fan half-spread | `fan_half_spread` | f32 | 80–400 px | Half the total fan width; tuning knob (see Section G) |
| Arc height | `arc_height` | f32 | 0–20 px | Vertical lift of edge cards above center; tuning knob |

**Output Range:** `card_x` ∈ [fan_center_x − fan_half_spread, fan_center_x + fan_half_spread]; `card_y` ∈ [fan_base_y, fan_base_y + arc_height]

**Example:** 5-card hand, center card (index=2): t=0 → card_x=fan_center_x, card_y=fan_base_y. Rightmost card (index=4): t=1.0 → card_x=fan_center_x+fan_half_spread, card_y=fan_base_y+arc_height.

**Edge case (count=1):** denominator clamped to 1; t=0; single card centered. No arc or rotation.

---

### Formula 2: Fan Card Rotation

```
card_rotation_deg = max_rotation_deg × t
```

**Variables:**

| Variable | Symbol | Type | Range | Description |
|---|---|---|---|---|
| Normalized position | `t` | f32 | −1.0 – +1.0 | From Formula 1 |
| Maximum tilt angle | `max_rotation_deg` | f32 | 5°–15° | Maximum rotation for the outermost card; tuning knob |

**Output Range:** [−max_rotation_deg, +max_rotation_deg]. Default: ±10°.

**Note:** Positive rotation = clockwise (right-leaning cards tilt right). Bevy UI `Transform.rotation` applies this as a `Quat::from_rotation_z` with sign convention per Bevy's coordinate system (verify at implementation). Rotated node AABB hit-detection is acceptable at ≤15° tilt — no special picking override needed.

---

### Formula 3: Cursor-to-Board-Cell Mapping (reference)

This formula is owned by **Board Rendering** via the `BoardLayout` resource. Hand UI reads `BoardLayout` during PLACEMENT drag to convert cursor screen position to board cell coordinates. See `design/gdd/board-rendering.md` Formula 1 (`cell_to_world` and its inverse) for the full definition. Hand UI does not re-define this formula.

## Edge Cases

- **If a TargetUnit card is dragged during PLACEMENT and no valid target units exist on the board:** the board shows a full-dim "no valid targets" overlay with text indicator; no cells highlight. Mouse-up anywhere cancels the drag and returns the card to its fan slot. The card is not locked — the player may attempt to drag it; they discover the constraint via the board overlay.

- **If a Minion card is dragged and the player's lane slot is already occupied** (by a prior-round unit or a currently staged Minion): the occupied lane's cells simply do not appear in the valid highlight set. No "invalid" indicator. The lane column stays dark. Staged Minions in the pending queue count as occupancy for highlight validation on subsequent drags — the client must exclude those cells even though the authoritative server board (prior rounds only) may not yet show them as occupied.

- **If the player drops a dragged card on an unhighlighted cell, outside the board, or outside the Instant play zone:** drag cancels. The drag sprite despawns. The UI card entity reappears at its original fan slot position (snap-back to origin, not to nearest open slot).

- **If the PLACEMENT timer reaches 0 while a card is mid-drag** (lifted from fan, cursor mid-air): the active drag cancels immediately; the drag sprite despawns; the in-flight card returns to its fan slot (fan ghost clears, card entity becomes visible again). C2SSubmitPlacement fires with only the previously confirmed-staged cards. The in-flight card is NOT included.

- **If the player presses Submit during PLACEMENT with 0 staged cards:** C2SSubmitPlacement fires with an empty placements vec. This is a valid no-op play (the player chooses to play nothing this round). The Submit button becomes inactive and the board does not change. This is intentional — the irrevocable commit applies equally to empty submits.

- **If the player has already submitted and the timer expires:** idempotent. C2SSubmitPlacement has already been sent; the button is inactive; no second message is sent. The timer expiry fires the server's own auto-submit logic but the player's submission is already logged.

- **If a card is bought during DRAFT_INITIAL and hand count reaches 10:** on receiving server confirmation (S2CCardAcquired), the client immediately locks all remaining grid cards (muted/non-interactive visual state) and shows a "Hand full" notification near the fan for 2 seconds. If a second C2SPurchaseCard was already sent before the first confirmation arrived (race condition), the server silently discards it; the client's lock fires on confirmation of the 10th card regardless.

- **If the pool for a displayed DRAFT_INITIAL grid card is exhausted by the opponent** (copies_remaining → 0 between display and click — CA OQ3): the server silently rejects the purchase. The client, which did not optimistically remove the card (Rule 4), shows the card still in the grid. The card renders with a "sold out" visual indicator (desaturated art + overlay text — Art Director defines the exact form). The slot remains visible but locked for the remainder of DRAFT_INITIAL. No gold is deducted.

- **If the hand has exactly 1 card** (count=1): Formula 1's denominator clamps to 1; t=0 for the single card; it is centered horizontally at fan_center_x with no arc lift and no rotation (Formula 2 produces 0°). No edge-case layout failure.

- **If the player reconnects during PLACEMENT:** S2CGameSnapshot delivers the current phase (PLACEMENT) and timer_remaining_ms. The client rebuilds Hand UI in STAGING state with zero staged cards — the local pending queue is lost on reconnect and is not included in the snapshot. The player must re-stage and re-submit within the remaining timer window. If the timer has already expired by reconnect time, the RSM has auto-submitted for them with zero placements (server-side behavior, not Hand UI behavior).

- **If a TargetObj card is dragged during PLACEMENT and some opponent objective lanes are destroyed** (fewer than 5 live objectives): only the cells corresponding to surviving opponent objectives highlight. Destroyed objective lanes show no highlight target for TargetObj cards.

- **If a LaneWide (Field) card is dropped anywhere within a lane's column highlight:** resolves to `LaneWide { lane }` regardless of which cell row the cursor was over at drop time. The exact cell position within the lane is irrelevant for Field cards.

## Dependencies

| System | Relationship | Interface |
|---|---|---|
| **Network Protocol** | Hard upstream | Receives S2CDraftOffering (9-card DRAFT_INITIAL offering), S2CCardAcquired (hand additions), S2CPhaseChanged (phase transitions), S2CGameSnapshot (reconnect rebuild). Sends C2SPurchaseCard, C2SActivateCard, C2SSubmitPlacement. |
| **Round State Machine** | Hard upstream (coordination) | RSM phase transitions drive all Hand UI state changes (Rule 3). Hand UI reads phase from S2CPhaseChanged; has no authority to trigger phase transitions. |
| **Card Acquisition** | Hard upstream (authority) | CA owns hand state. Hand UI is a read-only downstream consumer — it displays what CA delivers via network messages. CA explicitly excludes itself from visual display ownership. |
| **Board Rendering** | Bidirectional (peer) | Hand UI → Board Rendering: GhostPlacementChanged messages on stage/un-stage (Board Rendering renders ghosts at board positions). Board Rendering → Hand UI: BoardLayout resource for cursor-to-cell conversion during PLACEMENT drag. |
| **Card Data & Pool** | Hard upstream (read-only) | Card definitions: name, mana cost, card type (Minion/Trap/Structure/Spell/Field/Instant), PlayTarget category, TextureAtlas frame index. Read at session start. |
| **Game Config** | Hard upstream (read-only) | placement_timer_seconds (10s), draft_initial_timer_seconds (45s), draft_shop_timer_seconds (30s), fan layout tuning knobs (fan_half_spread, arc_height, max_rotation_deg). |
| **Shop / Auction UI** | Peer (no direct interaction) | Shop/Auction UI renders shop slots and the auction bidding panel. Hand UI renders the hand fan. They share screen space but do not exchange data directly — both subscribe to the same network messages independently. |

**Bidirectionality notes:**
- Board Rendering's GDD (currently a skeleton, In Design) must list Hand UI as a downstream consumer of `BoardLayout` and as the sender of `GhostPlacementChanged` when its Interactions section is authored.
- Card Acquisition's GDD already lists Hand UI as downstream (read-only). ✓
- Network Protocol's GDD lists all hand-related messages. C2SSubmitPlacement, C2SPurchaseCard, C2SActivateCard are already defined. ✓

## Tuning Knobs

| Knob | Default | Safe Range | Impact | Config Source |
|---|---|---|---|---|
| `fan_half_spread` | 280 px | 80–400 px | Total width of the card fan. At 80px: heavy overlap (10 cards nearly stacked); at 400px: fan wider than most screens. Target: cards slightly overlapping at 10/hand. | Client render config |
| `arc_height` | 10 px | 0–20 px | Vertical lift of edge cards above center. At 0: flat horizontal fan. At 20px: visible arc curve. Above 20px: edge cards clip below screen bottom. | Client render config |
| `max_rotation_deg` | 10° | 5°–15° | Tilt angle of the outermost card. At 5°: nearly flat, minimal fan feel. At 15°: strong fan, edge cards start to feel awkward to read. Above 15°: AABB hit-detection divergence becomes noticeable. | Client render config |
| `card_draw_animation_ms` | 280 ms | 150–400 ms | Duration for card-from-offer-to-fan slide animation (DRAFT_INITIAL purchases). Below 150ms: motion too abrupt. Above 400ms: competes with player's ongoing DRAFT_INITIAL decisions. | Client render config |
| `drag_lift_scale` | 1.10 | 1.05–1.20 | Scale multiplier applied to a card on drag-start (lift feel). Below 1.05: imperceptible. Above 1.20: card becomes oversized and obscures board cells. | Client render config |
| `snap_back_duration_ms` | 220 ms | 100–300 ms | Duration of snap-back animation when a drag is cancelled. During PLACEMENT, must be short enough not to consume meaningful timer seconds. | Client render config |
| `placement_urgency_threshold_seconds` | 5 s | 3–8 s | Seconds remaining on placement timer when the timer shifts to urgent visual state. Must be < placement_timer_seconds (10s). At 3s: very short warning window. At 8s: player is always in urgent state; defeats purpose. | `GameConfig` |
| `hand_full_notification_duration_ms` | 2000 ms | 1000–4000 ms | How long the "Hand full" notification shows before auto-dismissing during DRAFT_INITIAL. | Client render config |
| `placement_animation_cap_ms` | 250 ms | — | Hard cap on any animation that runs during PLACEMENT (drag lift, snap-back). No placement-window animation may exceed this value — timer seconds are too valuable to consume with motion. Not independently tunable; enforced in implementation. | Fixed code constant |

**Knobs that affect Hand UI but are owned elsewhere:**

| Knob | Default | Owner | Impact on Hand UI |
|---|---|---|---|
| `placement_timer_seconds` | 10s | game-config.md | The timer Hand UI displays during PLACEMENT. The primary pressure parameter. |
| `draft_initial_timer_seconds` | 45s | game-config.md | Duration of the DRAFT_INITIAL grid overlay. |
| `draft_shop_timer_seconds` | 30s | game-config.md | Duration of DRAFT_SHOP passive mode. |

## Visual/Audio Requirements

[To be designed]

## UI Requirements

[To be designed]

## Acceptance Criteria

[To be designed]

## Open Questions

[To be designed]
