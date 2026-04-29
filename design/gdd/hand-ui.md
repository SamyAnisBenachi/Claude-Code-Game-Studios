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

### VA-1: Card Face Visual Hierarchy

Card display size: 120×180 px. At 10-card hand compression, the face must communicate this priority order:

| Priority | Element | Spec |
|---|---|---|
| 1 | Art | Full-bleed illustration dominates the face |
| 2 | Mana cost | Top-left diamond badge, class-color background |
| 3 | Card name | Bottom strip, Ivory `#F7F0DC`, Heavy weight |
| 4 | ATK | Top-right orange `#E07020` badge (orange is globally reserved for ATK) |
| 5 | HP | Below ATK, teal `#2AA8C4` gem (teal is globally reserved for HP) |
| 6 | Type/rarity | Small icon bottom corner, 18px — degrades gracefully if compressed |

ATK and HP badges must maintain a minimum 16×16 px render floor at 10-card overlap — they clip before they disappear.

**Hover state** (DRAFT_INITIAL grid and DRAFT_SHOP): card scales to 240×360 px zoom tier (80ms ease-out). Hovered card receives a gold outline pulse (`#F5C842`, 2px, 1Hz). Adjacent fan cards compress slightly.

### VA-2: Staged (Committed) Card Ghost

Fan slot retains a ghost: card art **desaturated to 40% chroma, 50% opacity**. No tint — desaturation alone is the "committed" signal. Card identity remains readable; the player can see which unit they staged.

**Sold-out slot** (DRAFT_INITIAL pool-exhausted card): art desaturated to 20% chroma + `#1A2D5A` Ink Blue overlay at 60% + "Sold Out" text label (Ivory Heavy, 1× base). Click suppressed.

**Hand-full grid lock**: remaining grid cards desaturate to 30% chroma + 40% Ink Blue overlay.

### VA-3: Drag Interaction Visuals

**Drag sprite:** world-space clone of card art (120×180 px logical) at 1.10× scale. Frame border dropped during drag — art + stat badges only, no card frame. Cursor follows drag sprite at 0 lag. No shadow or glow pass — scale + physical separation from the fan communicates lift; shadow would add GPU cost with no readability gain.

### VA-4: Valid Target Highlights

**BoardCell valid cells:** Sky Blue `#3A8EDB` semi-opaque overlay at 50% (Player A). Terracotta `#D45C22` for Player B. Static overlays — no pulsing. Highlights appear instantly on drag-start; disappear instantly on drag-end.

**No red/invalid overlay for occupied cells.** Occupied cells have no highlight — they are simply absent from the valid set. Red is reserved for combat events; showing red invalid cells trains players incorrectly.

**TargetUnit hover:** target unit receives Prism White `#EEF4FF` outline pulse (2px, 2Hz). Applied to the world-space unit sprite, not the board tile.

**LaneWide cards:** full lane column receives the Sky Blue/Terracotta overlay across all rows.

### VA-5: PLACEMENT Timer

| Time Remaining | Color | Scale | Effect |
|---|---|---|---|
| 10s – 6s | Ivory `#F7F0DC` | 1.0× | None |
| 5s – 1s | Amber `#E87C1E` → Crimson `#9C2000` (stepped per second) | 1.05× | Per-second pulse: 1.10×→1.05× over 120ms |

Heavy-weight whole-second numeral on a semi-opaque Ink Blue `#1A2D5A` 40% panel (minimum 60×36 px). Panel required — timer must never render directly over animated board content. After Submit: timer keeps running; a checkmark glyph (20px Ivory) appears left of the numeral.

### VA-6: DRAFT_INITIAL Grid

Centered overlay: 70% opacity Ink Blue backing (board visible behind), panel in dark `#0D1830` with 2px Arcane Gold `#F5C842` border (12px radius). Purchase-send feedback: 60ms Gold bloom flash. On S2CCardAcquired: card slides to fan (280ms ease-out); empty slot shows faint checkmark for 500ms.

### VA-7: Instant Card Fan Plate Drop Zone

Hand fan background plate receives: Prism White `#EEF4FF` 3px border glow at 60% opacity, 0.5Hz pulse. Plate background brightens to `#1E2A3A`. On Instant staged: border flashes Arcane Gold for 80ms then returns to rest. Slow pulse (0.5Hz) signals "inviting" not "urgent" — categorically different from board-cell highlights.

### VA-8: Audio Cues

| Event | Character |
|---|---|
| Card lift (drag-start) | Crisp, papery pick-up transient, 60–80ms, no reverb |
| Valid target highlights appear | Subtle crystalline shimmer, ~100ms, nearly subliminal — one cue per drag, not per cell |
| Successful stage (valid drop) | Weighted "thunk" or placement click, warm and physical, 80–120ms — heavier than lift |
| Snap-back / invalid drop | Soft whoosh-back, shorter and quieter than stage sound, 80ms |
| Instant card staged to plate | Same register as valid stage + subtle crystalline overtone, 100ms |
| Submit pressed | Sharp leading click + resonant ring decaying over 400ms — the ring signals permanence |
| Timer urgency (5s) | Single heartbeat tone at 5-second mark; per-second visual pulse runs silently after |
| Card acquired (DRAFT_INITIAL) | Light ascending two-note chime, ~150ms |
| Hand full notification | Soft neutral bell, ~200ms, informs rather than scolds |

**Hard audio constraints:** (1) No looping audio during PLACEMENT — the single 5-second cue is the entire timer audio budget. (2) Submit sound must be audible at low browser volume. (3) All hand audio in `ui_hand` audio channel for independent volume control.

> **Art bible cross-references are provisional** — the art bible has not been authored yet. Color values and visual principles above are grounded in the master GDD's Ankama/Wakfu art direction. When `/art-bible` is run, reconcile these values against the authored sections.

> **📌 Asset Spec** — Visual requirements are defined. After the art bible is approved, run `/asset-spec system:hand-ui` to produce per-asset visual descriptions, dimensions, and generation prompts from this section.

## UI Requirements

Hand UI is itself a UI system. This section documents the interaction surfaces that require a formal UX spec before implementation.

### Screens and Surfaces Requiring UX Specs

| Surface | Phase | Spec needed |
|---|---|---|
| Hand fan (PLACEMENT staging) | PLACEMENT | Drag-and-stage flow, valid/invalid states, fan ghost, submit button placement |
| DRAFT_INITIAL grid overlay | DRAFT_INITIAL | 3×3 grid layout, slot anatomy, purchase feedback, hand-full lock state |
| Instant card drop zone | PLACEMENT | Fan plate highlight, drop confirmation |
| PLACEMENT timer display | PLACEMENT | Timer position, urgency visual states |
| "Hand full" notification | DRAFT_INITIAL | Notification placement, duration, dismissal |

### Input Model

- **Primary input:** Mouse (cursor) — click and drag
- **PLACEMENT drag:** mouse-down on card → drag to board or fan plate → mouse-up to drop
- **DRAFT_INITIAL / DRAFT_SHOP:** click-only (no drag)
- **Keyboard:** no keybindings defined for Hand UI in this GDD (Sprint 1 scope)
- **Touch:** not in scope (hackathon scope per technical preferences)

### Layout Constraints

- Hand fan occupies the bottom strip of the screen (~80–100 px tall at rest; expands to show card zoom on hover)
- DRAFT_INITIAL grid overlay: centered, modal — blocks other input behind it
- Submit button: fixed position above the fan during PLACEMENT, does not shift when fan layout updates
- Timer: fixed position, upper-right of the screen (does not overlap board center)

> **📌 UX Flag — Hand UI**: This system has UI requirements. Run `/ux-design hand-ui` to create a UX spec for the hand fan, DRAFT_INITIAL grid, and PLACEMENT staging flow **before** writing implementation epics. Stories that reference Hand UI interaction should cite `design/ux/hand-ui.md`, not this GDD directly.

## Acceptance Criteria

All criteria BLOCKING unless noted ADVISORY.

### Hand Fan Display

| # | Criterion | Type |
|---|---|---|
| HU-01 | **GIVEN** the game session starts with 0 cards in hand, **WHEN** Hand UI initializes, **THEN** 10 pre-pooled card slot entities exist in the scene (all hidden), with no runtime spawn or despawn occurring during the session. | BLOCKING |
| HU-02 | **GIVEN** the hand has 5 cards, **WHEN** the fan layout renders, **THEN** card at index 2 (center) has `t=0`, `card_x = fan_center_x`, `card_rotation_deg = 0°`. Card at index 4 (rightmost) has `t=1.0`, `card_x = fan_center_x + fan_half_spread`, `card_rotation_deg = max_rotation_deg`. | BLOCKING |
| HU-03 | **GIVEN** the hand has exactly 1 card, **WHEN** the fan renders, **THEN** the single card is centered (t=0, Formula 1 denominator clamped to 1), no arc lift, 0° rotation. | BLOCKING |

### Phase Behavior

| # | Criterion | Type |
|---|---|---|
| HU-04 | **GIVEN** RSM transitions to RESOLUTION, **WHEN** S2CPhaseChanged(RESOLUTION) is received, **THEN** all Hand UI elements (fan, submit button, timer) are hidden (Visibility::Hidden) within the same frame. No exit animation plays. | BLOCKING |
| HU-05 | **GIVEN** Hand UI is in RESOLUTION (hidden), **WHEN** S2CPhaseChanged(DRAFT_SHOP) is received, **THEN** the hand fan becomes visible with the updated hand state. | BLOCKING |
| HU-06 | **GIVEN** RSM is in DRAFT_AUCTION and the player clicks a card in the hand fan, **THEN** no C2SActivateCard or any other message is sent (input fully suppressed in PASSIVE_LOCKED state). | BLOCKING |

### DRAFT_INITIAL Grid

| # | Criterion | Type |
|---|---|---|
| HU-07 | **GIVEN** DRAFT_INITIAL begins and S2CDraftOffering is received with 9 card IDs, **WHEN** the grid renders, **THEN** exactly 9 card slots are visible in the 3×3 panel, each showing the correct card's art, name, mana cost. | BLOCKING |
| HU-08 | **GIVEN** the player clicks a card in the DRAFT_INITIAL grid, **WHEN** S2CCardAcquired confirms the purchase, **THEN** the grid slot empties AND the card appears in the hand fan at its computed fan position, arriving within `card_draw_animation_ms` (280ms default). | BLOCKING |
| HU-09 | **GIVEN** local hand count reaches 10 during DRAFT_INITIAL, **WHEN** S2CCardAcquired delivers the 10th card, **THEN** all remaining grid cards enter the locked visual state (30% chroma, 40% Ink Blue overlay) and click events are suppressed client-side within the same frame. | BLOCKING |
| HU-10 | **GIVEN** the player clicks a grid card during DRAFT_INITIAL and the server silently rejects it (pool exhausted — dead slot), **WHEN** no S2CCardAcquired arrives, **THEN** the card remains visible in the grid with a "Sold Out" visual indicator; no gold is deducted. | BLOCKING |

### PLACEMENT — Drag and Stage

| # | Criterion | Type |
|---|---|---|
| HU-11 | **GIVEN** PLACEMENT begins, **WHEN** Hand UI enters STAGING state, **THEN** the Submit button is visible, labeled "Submit (0 cards)", and is active (clickable) from the first frame of PLACEMENT. | BLOCKING |
| HU-12 | **GIVEN** the player drag-starts a Minion card during PLACEMENT, **WHEN** the cursor enters the board area, **THEN** only the player's valid spawn cells (minus occupied and already-staged cells) highlight in Sky Blue. Occupied cells have no highlight. | BLOCKING |
| HU-13 | **GIVEN** the player stages a card by dropping it on a valid board target, **WHEN** the drop is confirmed, **THEN** a GhostPlacementChanged message is sent to Board Rendering, the fan slot dims to 40% chroma / 50% opacity ghost, and the Submit button updates to "Submit (N cards)". | BLOCKING |
| HU-14 | **GIVEN** the player drops a dragged card on an unhighlighted (invalid) target, **WHEN** the drop fires, **THEN** the drag sprite despawns and the card entity reappears at its original fan slot position. No GhostPlacementChanged message is sent. | BLOCKING |
| HU-15 | **GIVEN** the player has 2 cards staged and the PLACEMENT timer reaches 0 while a third card is mid-drag, **WHEN** timer expiry fires, **THEN** C2SSubmitPlacement is sent with exactly the 2 staged placements. The in-flight card returns to its fan slot. The third card is NOT included in the submission. | BLOCKING — Integration |
| HU-16 | **GIVEN** the player clicks Submit with 0 staged cards, **THEN** C2SSubmitPlacement is sent with an empty placements vec, Submit button becomes inactive ("Submitted"), and no confirmation modal appears. | BLOCKING |
| HU-17 | **GIVEN** the player clicks Submit once and the button becomes inactive, **WHEN** the player attempts to click Submit again, **THEN** no second C2SSubmitPlacement is sent. | BLOCKING |

### PLACEMENT — Instant Cards

| # | Criterion | Type |
|---|---|---|
| HU-18 | **GIVEN** the player drag-starts an Instant card during PLACEMENT, **WHEN** the drag sprite lifts, **THEN** the hand fan background plate highlights (Prism White border, 0.5Hz pulse) and no board cells highlight. | BLOCKING |
| HU-19 | **GIVEN** the player drops an Instant card on the highlighted fan plate zone, **WHEN** the drop fires, **THEN** the card stages as `PlayTarget::Instant`, Submit count increments, and the plate border flashes gold for 80ms then returns to rest. | BLOCKING |

### PLACEMENT — TargetUnit Edge Case

| # | Criterion | Type |
|---|---|---|
| HU-20 | **GIVEN** the player drag-starts a TargetUnit card during a round where no valid target units exist on the board, **WHEN** the drag sprite is over the board, **THEN** no cells highlight and a "no valid targets" overlay covers the board. Drop anywhere returns the card to hand. | BLOCKING |

### Un-Staging

| # | Criterion | Type |
|---|---|---|
| HU-21 | **GIVEN** a card is staged (fan ghost visible, board ghost active), **WHEN** the player clicks the board ghost, **THEN** the card is removed from the pending queue, GhostPlacementChanged clears the board ghost, the fan slot restores to full opacity, and Submit count decrements. | BLOCKING |

### Timer

| # | Criterion | Type |
|---|---|---|
| HU-22 | **GIVEN** the placement timer shows 5 seconds remaining, **WHEN** the 5-second threshold fires, **THEN** the timer numeral color shifts to Amber `#E87C1E` and a single urgency audio cue plays. No looping audio begins. | ADVISORY |
| HU-23 | **GIVEN** the player submits at 7 seconds remaining, **WHEN** Submit fires, **THEN** the timer continues running and a checkmark glyph appears left of the numeral. The timer color ramps normally through urgency states. | BLOCKING |

### Reconnect

| # | Criterion | Type |
|---|---|---|
| HU-24 | **GIVEN** the player reconnects during PLACEMENT (S2CGameSnapshot received with phase=PLACEMENT), **WHEN** Hand UI rebuilds, **THEN** the fan enters STAGING state with 0 staged cards, Submit shows "Submit (0 cards)", and the timer displays timer_remaining_ms from the snapshot. | BLOCKING — Integration |

## Open Questions

| # | Question | Owner | Notes |
|---|---|---|---|
| OQ1 | **Reserve mana split UI for non-Xelor classes** — the `reserve_amount: u32` field in `PlacedCard` is currently spec'd as Xelor-only (+/- buttons per staged card). If future classes also use reserve mana (Class System GDD), will the same +/- control work for them, or does each class need a different UI? | Class System GDD | Flag for Class System designer when authoring class-specific rules. The +/- control should be designed to be parameterizable, not hard-coded to Xelor. |
| OQ2 | **Card zoom → click activation interaction** — when a card is in the zoomed hover state (240×360 px) during DRAFT_SHOP and the player clicks, does the click fire C2SActivateCard immediately, or does the zoom dismiss first (requiring a second click)? | UX spec (`/ux-design hand-ui`) | A UX call. Recommend: click while zoomed = activate immediately (no double-click barrier). |
| OQ3 | ~~Ready signal button (C2SSignalReady).~~ **RESOLVED** — Shop/Auction UI GDD owns the Ready/Retract Ready button (DRAFT_SHOP Rule 7, shop-auction-ui.md). Hand UI has no Ready button. `C2SSignalReady` is already registered in network-protocol.md. | — | Closed 2026-04-30 |
| OQ4 | **GhostPlacementChanged interface** — Board Rendering's GDD is currently a skeleton (Detailed Design not yet authored). The formal message type and payload for GhostPlacementChanged needs to be specified when Board Rendering's Interactions section is written. Hand UI depends on this interface being stable before implementation. | Board Rendering GDD | Do not implement Hand UI's staging system until Board Rendering's GDD defines this interface. |
| OQ5 | **Card ID → visual asset mapping** — at session start, Hand UI reads card definitions to get TextureAtlas frame indices. Is this via a `CardDataPlugin` resource loaded by `bevy_asset_loader`, or does Hand UI query Card Data & Pool directly? The asset loading architecture is not defined in this GDD. | Architecture / ADR | Needs an ADR for the client-side card data pipeline before implementation. |
