# Hand UI

> **Status**: In Review (revision pass 2026-04-30 R2)
> **Author**: SamyAnisBenachi + Claude Code agents
> **Last Updated**: 2026-04-30
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
The hand fan renders the player's hand as a row of up to 10 cards at the bottom of the screen. Cards use absolute positioning with per-card rotation angles (±10° arc spread) — the fan layout is not flexbox-driven. All 10 fan card slots AND all 9 DRAFT_INITIAL grid slots are spawned at session start (pre-pooled); cards are shown/hidden per hand state, not spawned/despawned per round. The drag sprite is also pre-spawned (one entity, hidden) at PLACEMENT entry and reused across drags. Pre-pooling avoids per-round allocator churn and TextureAtlas re-bind overhead — Rust/Bevy has no GC, so the concern is allocation/render-state churn, not GC pauses. Card art uses a `TextureAtlas` for GPU sprite batching; see Dependencies for atlas-sharing decision with Board Rendering.

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
During DRAFT_SHOP, Instant-type cards in the hand fan can be activated with a single click. Click sends `C2SActivateCard`. No drag gesture. No board target. Non-Instant cards are display-only in the hand fan during DRAFT_SHOP (purchasing happens from shop slots, owned by Shop/Auction UI).

**Rule 5b — Click while zoomed.** If a card is in the hover-zoom state (240×360 px, see VA-1) when the player clicks it, `C2SActivateCard` fires immediately on that click. The zoom is not a confirmation barrier — there is no double-click requirement. (Resolves OQ2.)

**Rule 5c — Single-shot activation lock.** On click, the card slot enters a locked visual state (`Visibility::Hidden` + input suppressed on the slot) until one of: (a) `S2CGoldUpdate` confirms the activation side effects; (b) `S2CActivationRejected` is received — slot unlocks immediately with no gold change; or (c) `activate_timeout_ms` (3000 ms default) elapses without any server response — slot reverts and the player may retry. This prevents double-click message storms during latency spikes. `S2CCardAcquired` is NOT a valid resolver — instant card plays never add a card to hand; only `S2CGoldUpdate` or `S2CActivationRejected` serve as the activation acknowledgement per NP Rule 2. **Dependency:** `S2CActivationRejected` does not exist in the NP GDD as of 2026-04-30 — see OQ8; this AC cannot be fully implemented until OQ8 is resolved.

**Rule 5d — DRAFT_SHOP drag-start suppression.** During DRAFT_SHOP, drag-start on any card in the hand fan is suppressed: no drag sprite becomes visible, no card entity hides from the fan slot. The gesture is silently absorbed. Players who attempt to drag a card in DRAFT_SHOP see no lift animation and no response — the mode switch from PLACEMENT's drag-to-stage to DRAFT_SHOP's click-only is intentional, and suppression prevents confusion from PLACEMENT muscle-memory transfers.

**Rule 6 — PLACEMENT: Drag-to-Stage**
On PLACEMENT entry: the Submit button appears immediately ("Submit (0 cards)"), and the timer starts. Staging flow:

1. **Drag-start** — mouse-down on a hand card: the UI card entity hides (`Visibility::Hidden`); the pre-pooled drag sprite (card art clone, see Rule 1) becomes visible at the cursor position, anchored at the card's logical center. Coordinate conversion: cursor screen position → world position via `Res<BoardLayout>` (Board Rendering's resource). The fan slot retains a dimmed, non-interactive ghost to preserve fan layout stability.
2. **Drag over board** — each frame, the cursor screen position converts to world coordinates via `BoardLayout`. Valid targets highlight per card type:
   - **BoardCell** (Minion/Trap/Structure): player's valid spawn cells for this round, minus cells occupied by prior-round board units, minus cells already targeted by staged Minions in the pending queue. Only valid cells highlight — occupied cells are simply absent from the highlight set (not shown as invalid).
   - **TargetUnit** (targeted spell): valid target units highlight (Prism White outline pulse, VA-4). If no valid units exist on the board, the board shows a full-dim "no valid targets" overlay; no cells highlight; drag-release anywhere returns the card to hand.
   - **TargetObj** (objective spell): the opponent's 5 objective cells (one per lane) highlight.
   - **LaneWide** (Field card): the full column of each lane highlights. Drop anywhere within a lane column resolves to `LaneWide { lane }`.
   - **Instant**: see Rule 7.
3. **Drop on valid target** — drag sprite returns to hidden; card stages to that target. Hand UI writes a `GhostPlacementChanged { target: Some(<PlayTarget>), card_id: Some(card_id) }` message (where `<PlayTarget>` is the resolved variant — see Interactions table for full payload). Board Rendering reads it and renders the variant-specific board preview (see Board Rendering Rule 8). Fan ghost dims further (desaturated). Submit count increments: "Submit (N cards)".
4. **Drop on invalid target or outside board** — drag sprite hides; UI card entity reappears in its original fan slot; fan ghost clears. No `GhostPlacementChanged` is emitted.

**Rule 7 — PLACEMENT: Instant Card Staging**
When an Instant card is dragged from the fan during PLACEMENT:
- Board cells do not highlight (no board target exists).
- The hand fan's background plate highlights as the valid drop zone (VA-7).
- Drop on the plate: card stages as `PlayTarget::Instant`. Hand UI writes `GhostPlacementChanged { target: Some(Instant), card_id: Some(card_id) }`. Board Rendering renders no board ghost for Instant (per Board Rendering Rule 8) — the only "ghost" is the dimmed fan slot itself. Fan ghost dims. Submit count increments.
- Drop outside the plate: drag cancels; card returns to fan slot.

**Rule 8 — Un-Staging**
A staged card may be un-staged by any of the three gestures below. All three follow the same atomic operation: remove the card from the pending queue, write `GhostPlacementChanged { target: None, card_id: Some(card_id) }` to clear Board Rendering's ghost, restore the fan slot to full opacity, decrement Submit count.

| Gesture | Applies to | Mechanism |
|---|---|---|
| **Click board ghost** | BoardCell, TargetUnit, TargetObj, LaneWide | Board Rendering owns the ghost entity. On click, Board Rendering writes `GhostClickedEvent { card_id }` (see Interactions). Hand UI reads this event and runs the un-stage operation. |
| **Drag board ghost back to fan** | BoardCell, TargetUnit, TargetObj, LaneWide | Board Rendering detects mouse-down on a ghost and emits `GhostDragStartEvent { card_id }`. Hand UI takes drag ownership from that point forward (the gesture becomes a Hand UI drag). On release within the hand fan zone, Hand UI un-stages. On release outside the fan zone, the ghost returns to its board position (no un-stage). |
| **Click dimmed fan ghost (Instant cards only)** | Instant | Instant-staged cards have no board ghost; their fan slot ghost is the only un-stage surface. Click on the dimmed fan slot un-stages. |

**Rule 9 — Timer Expiry During Active Drag**
If the PLACEMENT timer reaches 0 while a card is mid-drag (lifted from fan, not yet dropped), the system enters a **200ms grace window**:

- During the grace window, the drag sprite remains visible, highlights remain active, and the player may complete a drop gesture normally. If mouse-up lands on a valid target within the window, the card stages (`GhostPlacementChanged` fires) and is included in the submission. This preserves the "not a twitch game" principle — a player completing an intentional drop is not penalised for 50–200ms of buzzer lag.
- If the 200ms window elapses without a mouse-up on a valid target, the drag cancels: the drag sprite hides, the in-flight card returns to its fan slot, and the card is NOT included in the submission.

`C2SSubmitPlacement` fires at the end of grace window resolution (after the card is staged or returned).

**Why 200ms and not cursor-position detection:** The prior design (auto-stage if cursor over valid target at expiry) would commit a card based on cursor position at an arbitrary clock moment — indistinguishable from the player hovering while reconsidering. The grace window instead requires the player's explicit mouse-up gesture within a short window, preserving player agency as the commit agent (Pillar 1: "I placed, I committed, I moved on").

**Rule 10 — Submit (with client-side pre-validation)**
The Submit button is active from PLACEMENT entry regardless of staged count. Pressing Submit triggers a two-step sequence:

1. **Client-side pre-validation** (before sending the message). Hand UI mirrors the server's validation (per `network-protocol.md` line 86):
   - `sum(placements[i].reserve_amount) ≤ player.reserve_mana`
   - `sum(card[i].cost − placements[i].reserve_amount) ≤ player.current_mana`
   - For each placement, the `card_id` is in the player's current hand
   - For each `BoardCell`, `TargetUnit`, `TargetObj`, `LaneWide`: the lane (1–5) and cell (1–8 if applicable) values are in range
   
   If pre-validation fails, the Submit button does NOT lock. An inline error label appears beneath the button (Crimson `#9C2000`, max 1 line, e.g. `"Reserve overdrawn"` / `"Mana overdrawn"` / `"Out-of-range placement"`). The player corrects locally (un-stage, adjust reserve split) and may re-submit. Pre-validation does not require server round-trip — every input is locally available.

2. **If pre-validation passes:**
   - Fires `C2SSubmitPlacement` with all staged placements.
   - Submit button immediately becomes inactive ("Submitted") — prevents double-submit.
   - No confirmation modal. Irrevocable from the player's perspective.
   - Fan cards dim slightly; staging interaction locked.

The server still performs the same validation server-side and silently discards invalid batches per NP Rule 4 — pre-validation is defence-in-depth, not a replacement for server validation. Hand UI does not hide on submit — only on RESOLUTION entry. The player may still watch the timer and board after submitting.

**Stale card_id edge case.** If a card in the staged queue is removed from the player's hand by a server-side event between PLACEMENT entry and submission (e.g., a future Class System mechanic), the server will silently discard the entire batch per NP Rule 4. The Submit button will show "Submitted" with no recovery path. This is accepted silent-failure behavior for the current scope. Revisit when the Class System GDD (M3) is authored and its PLACEMENT-phase hand-modification interactions are defined.

**Rule 11 — PLACEMENT Timer Display**
- Shows whole seconds (not milliseconds); large enough to read peripherally.
- At 5 seconds remaining: timer shifts visual state (color change — Art Director defines specifics).
- Timer expiry fires a single audio cue (not repeating ticks — see Audio Requirements).

**Rule 12 — RESOLUTION Hide**
On RESOLUTION entry: all Hand UI elements hide immediately (`Visibility::Hidden`). No exit animation. On RESOLUTION exit to DRAFT: Hand UI restores visibility with the updated hand state from the completed round.

**Rule 13 — Reserve Mana Split UI**
Cards with `cost > 0` may have a portion of their cost paid from the player's reserve mana pool instead of the current-round mana. The split is encoded per staged card as `PlacedCard.reserve_amount: u32` (NP line 69). The default at stage time is `reserve_amount = 0` (pay all from current).

**When the control appears.** The split control attaches to each staged card's *fan ghost* (not the board ghost) — anchored just above the dimmed fan slot in the bottom strip. It appears the moment a card stages and disappears the moment it un-stages. It is visible only during PLACEMENT in `STAGING` state and is disabled (display-only, non-interactive) once the player presses Submit.

**Strip positioning at high card counts.** Each strip is centered horizontally on its fan ghost slot's `card_x` position (from Formula 1). At `fan_half_spread = 280 px` with 10 staged cards, adjacent fan slots are spaced ≈62 px apart center-to-center. Strips (96 px wide) will overlap adjacent strips by ≈34 px at full staging — this overlap is intentional and acceptable given the absolute positioning of each strip over its ghost. The `[ − ]` and `[ + ]` buttons (24 px each) remain accessible at the left and right edges of each strip despite overlap. The minimum spatial separation between a strip button and the Instant card un-stage gesture (clicking the dimmed fan ghost): the un-stage click target is the full 120-px slot; the strip is positioned 8 px above it. Vertical separation of ≥8 px is maintained at all card counts.

**Layout.** Each control is a single horizontal strip (height 24 px, width 96 px) showing `[ − ] [N / cost] [ + ]`:
- `[ − ]` decrements `reserve_amount` by 1 (clamped to 0).
- `[ + ]` increments `reserve_amount` by 1 (clamped to `card.cost` and to `player.reserve_mana − sum(other_staged.reserve_amount)`).
- `[N / cost]` displays the current split, e.g. `2 / 5` = "2 mana from reserve, 3 from current."

**Interaction model.**
- Click `[ − ]` or `[ + ]` to step `reserve_amount` by 1. No drag, no slider, no modal — single-click steps only. Each click runs the clamp checks and updates the display in the same frame.
- The strip is non-modal: it does not block board reading or other staging actions. The 10-second timer continues to run during adjustment.
- If the player has no reserve mana available at all (`player.reserve_mana == 0`), all `[ + ]` buttons are disabled (greyed out) and the strip displays `0 / cost`. No interaction possible.
- For cards with `cost == 0` (free cards), the entire strip is hidden (no decision to make).
- The `[ + ]` button becomes `Disabled` immediately when `reserve_amount` reaches `min(card.cost, player.reserve_mana − sum(other_staged.reserve_amount))` — the remaining pool ceiling. **No auto-decrement of other staged cards occurs.** If the player wants to allocate more reserve to card B, they must first press `[ − ]` on card A to free reserve. This keeps the interaction model explicit and reversible.

**Pre-submit validation interaction.** The Rule 10 pre-validation sums all `reserve_amount` and all `cost − reserve_amount` across staged cards. The `[ + ]` button-disable logic prevents most overdraw cases proactively, but the Rule 10 check is the final gate.

**Class System dependency.** The Hand UI control is class-agnostic: any card with `cost > 0` shows the strip. The Class System GDD (M3, Not Started) may add class-specific reserve mana behaviours (e.g., Xelor `Rollback` mechanics). The +/- control's *interaction model* is fixed by this GDD; the *availability* and *meaning* of reserve mana per class is owned by Class System.

**Rule 14 — `C2SPurchaseCard` Pending Timeout (DRAFT_INITIAL grid)**
When the player clicks a grid card during DRAFT_INITIAL, the slot enters a "pending" visual state (grid card dimmed, click suppressed on that slot). The pending state resolves on `S2CCardAcquired` (success → grid slot empties, card slides to fan). If no `S2CCardAcquired` arrives within `purchase_timeout_ms` (3000 ms default), the slot reverts to its pre-click state and the player may retry. This covers all non-arrival cases — delayed server response, phase transition, and pool-exhausted silent rejection — uniformly. No sold-out visual state exists; the ambiguity between "sold out" and "server drop" is accepted (see Edge Cases).

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
GRID → PASSIVE             on RSM: DRAFT_INITIAL → DRAFT_SHOP (round 1, normal exit)
GRID → STAGING             on RSM: DRAFT_INITIAL → PLACEMENT (round 1, no DRAFT_SHOP — should not occur in current flow but reserved)
HIDDEN → STAGING           on RSM: PLACEMENT entry (recovery path only)
STAGING → SUBMITTED        on: C2SSubmitPlacement sent (after pre-validation pass)
STAGING → STAGING          on: C2SSubmitPlacement pre-validation FAILED (no state change; inline error displayed)
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
| **Network Protocol** | NP → Hand UI | `S2CDraftOffering` (9 card IDs for grid), `S2CCardAcquired` (hand additions on purchase), `S2CPhaseChanged` (drives state transitions), `S2CGameSnapshot` (reconnect rebuild), `S2CGoldUpdate` (resolves `C2SActivateCard` activation lock per Rule 5c), `S2CActivationRejected` (immediate unlock on server discard — **pending NP registration, see OQ8**) |
| **Network Protocol** | Hand UI → NP | `C2SPurchaseCard` (DRAFT_INITIAL grid buy), `C2SActivateCard` (DRAFT_SHOP Instant play), `C2SSubmitPlacement` (PLACEMENT batch commit) |
| **Round State Machine** | RSM → Hand UI | Phase transitions via `S2CPhaseChanged` drive Hand UI state machine (Rule 3) |
| **Board Rendering** | Hand UI → Board Rendering | `GhostPlacementChanged { target: Option<PlayTarget>, card_id: Option<CardId> }` intra-client message. `target` carries the full `PlayTarget` variant (BoardCell, TargetUnit, TargetObj, LaneWide, Instant) so Board Rendering can render variant-specific previews. `target: None` clears the ghost for that `card_id`. `target: Some(Instant)` is a no-op for Board Rendering (no board ghost) but is sent for protocol completeness. |
| **Board Rendering** | Board Rendering → Hand UI | (1) `Res<BoardLayout>` resource — read for cursor-to-world-position conversion during PLACEMENT hover detection. (2) `GhostClickedEvent { card_id }` intra-client event — emitted when the player clicks a board ghost; consumed by Hand UI to un-stage (Rule 8). (3) `GhostDragStartEvent { card_id }` intra-client event — emitted when the player mouse-downs on a board ghost; Hand UI takes drag ownership from this point (Rule 8). |
| **Card Data & Pool** | Pool → Hand UI | Card definitions (name, mana cost, card type, `PlayTarget` category, TextureAtlas frame index). Read at session start; not polled each frame. |
| **Card Acquisition** | CA → Hand UI | Upstream authority on hand state. Hand UI is read-only downstream — displays CA's server-delivered hand state. CA explicitly excludes itself from visual ownership. |
| **Game Config** | Config → Hand UI | `placement_timer_seconds` (10s), `draft_initial_timer_seconds` (45s), `draft_shop_timer_seconds` (30s), `purchase_timeout_ms` (3000), `activate_timeout_ms` (3000). Loaded at session start. |

## Formulas

### Formula 1: Fan Card Screen Position

```
// PRECONDITION: 1 ≤ count ≤ 10, 0 ≤ index ≤ count − 1
// count == 0 → no cards rendered; formula NOT evaluated.
// count == 1 → t = 0 (special-case bypass; do not enter the divide).

if count == 1:
    t = 0.0
else:
    half_span = (count - 1) / 2.0   // valid range: 0.5 ≤ half_span ≤ 4.5 for count ∈ [2,10]
    t = (index - half_span) / half_span    // safe: half_span ≥ 0.5 always when count ≥ 2

card_x = fan_center_x + t × fan_half_spread
card_y = fan_base_y − arc_height × t²     // SUBTRACTION: edge cards lift UP in bevy_ui screen-space (+Y is down)
```

**Coordinate convention.** The hand fan is rendered as `bevy_ui` (Node-based UI), not world-space sprites. In `bevy_ui` screen-space, **+Y is downward**. To produce a fan that arcs **upward** at the edges (the visual fantasy), `card_y` must DECREASE from `fan_base_y` as `|t|` grows — hence the subtraction. The drag sprite, in contrast, is world-space (drawn over the board); coordinate conversion is handled by `Res<BoardLayout>` from Board Rendering at drag time.

**Variables:**

| Variable | Symbol | Type | Range | Description |
|---|---|---|---|---|
| Card index (0-based) | `index` | u8 | 0 – count−1 | Position of the card in the hand array |
| Hand size | `count` | u8 | 0 – 10 | Current number of cards in hand. `count = 0` skips formula evaluation entirely. |
| Normalized position | `t` | f32 | −1.0 – +1.0 | −1 = leftmost card, +1 = rightmost, 0 = center. Hard-set to 0 when `count == 1`. |
| Fan horizontal center | `fan_center_x` | f32 | screen_width / 2 | Center anchor of the fan (px) |
| Fan vertical baseline | `fan_base_y` | f32 | screen_height − margin | Y-coordinate of the fan bottom edge (px) — the lowest point of any card |
| Fan half-spread | `fan_half_spread` | f32 | 180–400 px | Half the total fan width; tuning knob. Practical minimum 180 px (below this, mana-cost badge readability fails at count=10 — see Tuning Knobs note). |
| Arc height | `arc_height` | f32 | 0–20 px | Upward lift of edge cards relative to center; tuning knob. Geometrically: how much edge cards rise above `fan_base_y`. |

**Output Range:** `card_x ∈ [fan_center_x − fan_half_spread, fan_center_x + fan_half_spread]`; `card_y ∈ [fan_base_y − arc_height, fan_base_y]` (arc lifts up).

**Example (count=5):** Center card (index=2): t=0 → `card_x = fan_center_x`, `card_y = fan_base_y`. Rightmost card (index=4): half_span=2; t=(4−2)/2=+1.0 → `card_x = fan_center_x + fan_half_spread`, `card_y = fan_base_y − arc_height`.

**Example (count=2):** half_span=0.5. index=0: t=(0−0.5)/0.5=−1.0. index=1: t=(1−0.5)/0.5=+1.0. ✓ Both cards reach full ±1.0; the previous `max(_, 1.0)` clamp bug — which compressed count=2 to ±0.5 — is removed by the `if count == 1` early-return.

**Edge case (count=1):** explicit early-return: `t = 0`. Single card centered at `(fan_center_x, fan_base_y)`. No arc, no rotation (Formula 2: `0° × 0 = 0°`).

**Edge case (count=0):** Formula not evaluated. No card slots are visible. The fan root entity remains visible (it is the parent for the Submit button anchor during PLACEMENT) but contains no rendered cards.

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

**Edge case (count=1):** From Formula 1, t=0 → `card_rotation_deg = 0`. Single card has no tilt.

**Edge case (count=0):** Not evaluated.

**Note:** Positive rotation = clockwise (right-leaning cards tilt right). `bevy_ui` `Transform.rotation` applies this as a `Quat::from_rotation_z` with sign convention per Bevy's coordinate system (verify at implementation). Rotated node AABB hit-detection is acceptable at ≤15° tilt — no special picking override needed.

---

### Formula 3: Cursor-to-Board-Cell Mapping (reference)

This formula is owned by **Board Rendering** via the `BoardLayout` resource. Hand UI reads `BoardLayout` during PLACEMENT drag to convert cursor screen position to board cell coordinates. See `design/gdd/board-rendering.md` Formula 1 (`cell_to_world` and its inverse) for the full definition. Hand UI does not re-define this formula.

## Edge Cases

- **If a TargetUnit card is dragged during PLACEMENT and no valid target units exist on the board:** the board shows a full-dim "no valid targets" overlay with text indicator; no cells highlight. Mouse-up anywhere cancels the drag and returns the card to its fan slot. The card is not locked — the player may attempt to drag it; they discover the constraint via the board overlay.

- **If a Minion card is dragged and the player's lane slot is already occupied** (by a prior-round unit or a currently staged Minion): the occupied lane's cells simply do not appear in the valid highlight set. No "invalid" indicator. The lane column stays dark. Staged Minions in the pending queue count as occupancy for highlight validation on subsequent drags — the client must exclude those cells even though the authoritative server board (prior rounds only) may not yet show them as occupied.

- **If the player drops a dragged card on an unhighlighted cell, outside the board, or outside the Instant play zone:** drag cancels. The drag sprite despawns. The UI card entity reappears at its original fan slot position (snap-back to origin, not to nearest open slot).

- **If the PLACEMENT timer reaches 0 while a card is mid-drag** (lifted from fan, not yet dropped): the system enters a 200ms grace window per Rule 9. If the player completes a drop on a valid target within the window, the card stages and is included in the submission. If the window elapses without a valid drop, the drag cancels and the card returns to its fan slot. `C2SSubmitPlacement` fires at the end of grace window resolution. Only deliberate mouse-up gestures stage cards; cursor position at timer expiry has no effect.

- **If the player presses Submit during PLACEMENT with 0 staged cards:** C2SSubmitPlacement fires with an empty placements vec. This is a valid no-op play (the player chooses to play nothing this round). The Submit button becomes inactive and the board does not change. This is intentional — the irrevocable commit applies equally to empty submits.

- **If the player has already submitted and the timer expires:** idempotent. C2SSubmitPlacement has already been sent; the button is inactive; no second message is sent. The timer expiry fires the server's own auto-submit logic but the player's submission is already logged.

- **If a card is bought during DRAFT_INITIAL and hand count reaches 10:** on receiving server confirmation (S2CCardAcquired), the client immediately locks all remaining grid cards (muted/non-interactive visual state) and shows a "Hand full" notification near the fan for 2 seconds. If a second C2SPurchaseCard was already sent before the first confirmation arrived (race condition), the server silently discards it; the client's lock fires on confirmation of the 10th card regardless.

- **If the pool for a displayed DRAFT_INITIAL grid card is exhausted by the opponent** (copies_remaining → 0 between display and click — CA OQ3): the server silently rejects the purchase with no `S2CCardAcquired` response. After `purchase_timeout_ms` (3000 ms), the slot reverts to its pre-click state per Rule 14. No gold is deducted. The player may retry the click; if the pool is still exhausted, they receive another timeout-revert. No sold-out visual indicator is shown — the ambiguity between "sold out" and "slow server response" is accepted. The player must infer unavailability from repeated failed attempts.

- **If the hand has exactly 1 card** (count=1): Formula 1's denominator clamps to 1; t=0 for the single card; it is centered horizontally at fan_center_x with no arc lift and no rotation (Formula 2 produces 0°). No edge-case layout failure.

- **If the player reconnects during PLACEMENT:** S2CGameSnapshot delivers the current phase (PLACEMENT) and timer_remaining_ms. The client rebuilds Hand UI in STAGING state with zero staged cards — the local pending queue is lost on reconnect and is not included in the snapshot. The player must re-stage and re-submit within the remaining timer window. If the timer has already expired by reconnect time, the RSM has auto-submitted for them with zero placements (server-side behavior, not Hand UI behavior).

- **If a TargetObj card is dragged during PLACEMENT and some opponent objective lanes are destroyed** (fewer than 5 live objectives): only the cells corresponding to surviving opponent objectives highlight. Destroyed objective lanes show no highlight target for TargetObj cards.

- **If a LaneWide (Field) card is dropped anywhere within a lane's column highlight:** resolves to `LaneWide { lane }` regardless of which cell row the cursor was over at drop time. The exact cell position within the lane is irrelevant for Field cards.

- **If the player has 0 cards when PLACEMENT begins:** the fan renders no card slots (Formula 1 not evaluated). The Submit button still appears with label `"Submit (0 cards)"` and is active. Pressing Submit fires `C2SSubmitPlacement` with empty `placements` (HU-16 covers the 0-card submit path). This is a valid no-op round.

- **If client-side pre-validation fails on Submit press (Rule 10):** Submit does NOT lock; an inline Crimson error label appears beneath the button (e.g. `"Reserve overdrawn"`); no `C2SSubmitPlacement` is sent. The player must un-stage or adjust reserve splits to bring the batch within bounds, then re-press Submit. There is no auto-fix — the player resolves the conflict explicitly.

- **If the player presses `[ + ]` on a reserve strip when `player.reserve_mana` is fully committed across all staged cards:** the `[ + ]` button is already `Disabled` per Rule 13 — the click is not processed. No state change; no error label. The player must first press `[ − ]` on another staged card to free reserve before incrementing this one.

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
| `fan_half_spread` | 280 px | 180–400 px (readable); 80 px (absolute minimum) | Total width of the card fan. **Practical minimum 180 px**: at lower values, the mana-cost badge of right-side cards is occluded at count=10 and the fan stops being parseable at a glance (Player Fantasy bullet 2 fails). The 80–180 range is technically renderable but visually degenerate; reserve for debug/test fixtures only. At 400 px: fan wider than most screens. Target: cards slightly overlapping at 10/hand with mana-cost badge of every card visible. | Client render config |
| `arc_height` | 10 px | 0–20 px | Upward lift of edge cards above center (Formula 1 subtracts this from `fan_base_y` — convention is upward arc in screen-space). At 0: flat horizontal fan. At 20 px: visible arc curve. Above 20 px: edge cards lift above the bottom strip's reserved area and may collide with overlapping board UI. | Client render config |
| `max_rotation_deg` | 10° | 5°–15° | Tilt angle of the outermost card. At 5°: nearly flat, minimal fan feel. At 15°: strong fan, edge cards start to feel awkward to read. Above 15°: AABB hit-detection divergence becomes noticeable. | Client render config |
| `card_draw_animation_ms` | 280 ms | 150–400 ms | Duration for card-from-offer-to-fan slide animation (DRAFT_INITIAL purchases). Below 150ms: motion too abrupt. Above 400ms: competes with player's ongoing DRAFT_INITIAL decisions. | Client render config |
| `drag_lift_scale` | 1.10 | 1.05–1.20 | Scale multiplier applied to a card on drag-start (lift feel). Below 1.05: imperceptible. Above 1.20: card becomes oversized and obscures board cells. | Client render config |
| `snap_back_duration_ms` | 220 ms | 100–300 ms | Duration of snap-back animation when a drag is cancelled. During PLACEMENT, must be short enough not to consume meaningful timer seconds. | Client render config |
| `placement_urgency_threshold_seconds` | 5 s | 3–8 s | Seconds remaining on placement timer when the timer shifts to urgent visual state. Must be < placement_timer_seconds (10s). At 3s: very short warning window. At 8s: player is always in urgent state; defeats purpose. | `GameConfig` |
| `hand_full_notification_duration_ms` | 2000 ms | 1000–4000 ms | How long the "Hand full" notification shows before auto-dismissing during DRAFT_INITIAL. | Client render config |
| `placement_animation_cap_ms` | 250 ms | — | Hard cap on any animation that runs during PLACEMENT (drag lift, snap-back). No placement-window animation may exceed this value — timer seconds are too valuable to consume with motion. Not independently tunable; enforced in implementation. | Fixed code constant |
| `purchase_timeout_ms` | 3000 ms | 2000–5000 ms | Maximum time a DRAFT_INITIAL grid slot stays in "pending" state after `C2SPurchaseCard` before reverting to its pre-click state. Covers all non-arrival cases: dropped response, phase transition, or pool exhaustion (no sold-out signal exists). See Rule 14. | Client render config |
| `activate_timeout_ms` | 3000 ms | 2000–5000 ms | Maximum time a hand card slot stays in "activation locked" state after `C2SActivateCard` before reverting. Guards against latency-induced double-click message storms. See Rule 5c. | Client render config |
| `max_concurrent_animators` | 24 | — | Worst-case ceiling on concurrent `bevy_tweening::Animator<T>` components active during PLACEMENT (10 fan ghost transitions + drag lift + timer pulse + Instant plate pulse + up to ~10 board ghost transitions in Board Rendering). Exceeding this number indicates an unintended cascade — investigate before shipping. Not enforced at runtime; advisory ceiling for performance review. | Code constant (advisory) |

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

### VA-9: Reserve Mana Split Strip

The split control attaches to each staged card's fan ghost (anchored 8 px above the dimmed fan slot). Strip dimensions: 96×24 px. Layout: `[ − ] [N / cost] [ + ]`.

| Element | Spec |
|---|---|
| Strip background | Ink Blue `#1A2D5A` 70% opacity, 4 px corner radius |
| Buttons (`[ − ]`, `[ + ]`) | 24×24 px each. Active: Ivory `#F7F0DC` glyph on Ink Blue. Hovered: glyph brightens to Prism White `#EEF4FF`. Disabled: glyph desaturated to 30% chroma. |
| `[N / cost]` numeric display | Ivory `#F7F0DC`, Heavy weight, centered, e.g. `2 / 5`. The leading number is the reserve_amount; the trailing number is the card's cost. |
| `cost == 0` cards | Strip not rendered (no decision available) |
| `player.reserve_mana == 0` | Both `[ − ]` and `[ + ]` disabled; display shows `0 / cost` |

**Audio:** Reserve adjust click (`[ − ]` or `[ + ]`) — soft mid-register click, ~50 ms, no reverb. In `ui_hand` channel.

### VA-10: PASSIVE_LOCKED Read-Only State

During DRAFT_AUCTION (`PASSIVE_LOCKED` state), the hand fan is visible but non-interactive. Visual treatment to communicate read-only status:

| Element | Spec |
|---|---|
| Fan card opacity | All slots reduced to 70% opacity (uniform dimming signals non-interactivity) |
| Cursor | Default cursor on hover — no pointer/hand cursor |
| Label | Small `"Auction in progress"` text above the fan (Ivory `#F7F0DC`, 12 px, 40% Ink Blue `#1A2D5A` backing panel) |

The label and reduced opacity together communicate that cards are readable but unclickable. No click-feedback sound plays on input attempts in this state.

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
| Reserve Mana Split strip | PLACEMENT | Per-staged-card +/- control anatomy, disabled states, position relative to fan ghost |
| Submit pre-validation error | PLACEMENT | Inline error label position beneath Submit button, error copy variants ("Reserve overdrawn", "Mana overdrawn", "Out-of-range placement") |

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

All criteria BLOCKING unless noted ADVISORY. Where an AC asserts visual properties (colour, chroma, opacity, animation), the BLOCKING gate is the underlying *state component* (e.g. `FanSlotState::Ghost`, `GridSlotState::Pending`, `NoValidTargetsOverlay` marker entity present); the visual rendering of that state is verified by lead sign-off as ADVISORY. This is the convention for every AC below that mixes state and visual assertions.

### Hand Fan Display

| # | Criterion | Type |
|---|---|---|
| HU-01 | **GIVEN** the game session starts with 0 cards in hand, **WHEN** Hand UI initializes, **THEN** 10 pre-pooled fan card slot entities and 9 pre-pooled DRAFT_INITIAL grid slot entities exist in the scene (all `Visibility::Hidden`), with no runtime spawn or despawn occurring during a normal session. (Reconnect rebuild per HU-24 may despawn-and-rebuild.) | BLOCKING |
| HU-02 | **GIVEN** the hand has 5 cards, **WHEN** the fan layout renders, **THEN** card at index 2 (center) has `t=0`, `card_x = fan_center_x`, `card_rotation_deg = 0°`. Card at index 4 (rightmost) has `t=+1.0`, `card_x = fan_center_x + fan_half_spread`, `card_rotation_deg = max_rotation_deg`. | BLOCKING |
| HU-02b | **GIVEN** the hand has exactly 2 cards, **WHEN** the fan layout renders, **THEN** card at index 0 has `t = −1.0` and card at index 1 has `t = +1.0`. (Surfaces the count=2 clamp bug fix in Formula 1.) | BLOCKING |
| HU-03 | **GIVEN** the hand has exactly 1 card, **WHEN** the fan renders, **THEN** the single card is centered (`t = 0` via Formula 1 early-return), `card_y = fan_base_y`, `card_rotation_deg = 0°`. | BLOCKING |
| HU-03b | **GIVEN** the hand has 0 cards, **WHEN** PLACEMENT begins, **THEN** Formula 1 is not evaluated, no card slot entities are visible, and the Submit button still appears active with label `"Submit (0 cards)"`. | BLOCKING |

### Phase Behavior

| # | Criterion | Type |
|---|---|---|
| HU-04 | **GIVEN** RSM transitions to RESOLUTION, **WHEN** `S2CPhaseChanged(RESOLUTION)` is received, **THEN** after exactly one `App::update()` tick: (a) the fan root entity, Submit button entity, and timer entity all have `Visibility::Hidden`; (b) no `Animator<Transform>`, `Animator<BackgroundColor>`, or `Animator<Style>` component exists on any Hand UI entity (enumerate all `Animator<T>` specializations used in the implementation). | BLOCKING |
| HU-05 | **GIVEN** Hand UI is in RESOLUTION (hidden), **WHEN** `S2CPhaseChanged(DRAFT_SHOP)` is received, **THEN** the fan root entity has `Visibility::Visible` AND each rendered card slot's `card_id` matches the current hand contents from the most recent `S2CCardAcquired` / snapshot delivery. | BLOCKING |
| HU-06 | **GIVEN** RSM is in DRAFT_AUCTION and the player clicks a card in the hand fan, **THEN** no `C2SActivateCard` or any other C2S message is written to the message queue (input fully suppressed in `PASSIVE_LOCKED` state). | BLOCKING |

### DRAFT_INITIAL Grid

| # | Criterion | Type |
|---|---|---|
| HU-07 | **GIVEN** DRAFT_INITIAL begins and `S2CDraftOffering` is received with 9 card IDs, **WHEN** the grid renders, **THEN** exactly 9 grid slot entities have `Visibility::Visible` AND each slot's bound card data matches its corresponding ID in the offering (name and mana cost components verified). Art rendering is ADVISORY (lead sign-off). | BLOCKING |
| HU-08 | **GIVEN** the player clicks a grid card during DRAFT_INITIAL, **WHEN** `S2CCardAcquired` confirms the purchase, **THEN** (a) the grid slot's `Visibility` becomes `Hidden` within one tick of receipt; (b) the corresponding fan slot becomes `Visible` and an `Animator<Transform>` interpolating to the computed fan position is attached; (c) after advancing virtual time by `card_draw_animation_ms`, the fan slot's `Transform.translation` equals the formula-computed fan position. | BLOCKING |
| HU-09 | **GIVEN** the 10th card has been added to the hand during DRAFT_INITIAL, **WHEN** `S2CCardAcquired` delivers the 10th card, **THEN** within the same `App::update()` tick: (a) all remaining visible grid slots have a `GridSlotState::HandFullLocked` marker component; (b) clicks on locked grid slots produce no `C2SPurchaseCard` message. The 30% chroma / Ink Blue overlay rendering is ADVISORY. | BLOCKING |
| HU-10 | **GIVEN** the player clicks a grid card and no `S2CCardAcquired` arrives (pool exhausted or server drop), **WHEN** `purchase_timeout_ms` (3000 ms) elapses, **THEN** (a) `player.gold` resource is unchanged; (b) the slot reverts from `GridSlotState::Pending` to `GridSlotState::Available`; (c) clicks on the slot are accepted again (player may retry). | BLOCKING |
| HU-10b | **GIVEN** the player clicks a grid card and `S2CCardAcquired` does not arrive within `purchase_timeout_ms` (simulating a dropped server response), **THEN** the slot reverts from `GridSlotState::Pending` to `GridSlotState::Available` and clicks are accepted again. (This is now identical to HU-10 by design — both non-arrival paths produce the same revert outcome.) | BLOCKING |
| HU-10c | **GIVEN** the hand reaches 10 cards (locking grid) AND a previously clicked grid card is still in `GridSlotState::Pending` (purchase in flight), **THEN** the slot's state becomes `GridSlotState::HandFullLocked` (hand-full lock takes precedence — click suppressed, pending state cleared). | BLOCKING |

### PLACEMENT — Drag and Stage

| # | Criterion | Type |
|---|---|---|
| HU-11 | **GIVEN** PLACEMENT begins, **WHEN** Hand UI enters STAGING state, **THEN** the Submit button entity has `Visibility::Visible`, its text component reads exactly `"Submit (0 cards)"`, and its interaction component is `Active` from the first frame of PLACEMENT. | BLOCKING |
| HU-12 | **GIVEN** the player drag-starts a Minion card during PLACEMENT, **WHEN** the cursor enters the board area, **THEN** the highlighted-cell set (queryable via the `BoardCellHighlighted` marker component on cell entities) equals exactly: (player's valid spawn cells) ∖ (cells with prior-round units) ∖ (cells already targeted by staged Minions). Sky Blue rendering is ADVISORY. | BLOCKING |
| HU-12b | **GIVEN** the player drag-starts a TargetObj card during PLACEMENT, **WHEN** the cursor enters the board area, **THEN** the highlighted-cell set equals exactly the surviving opponent objective cells (one per surviving lane; destroyed objectives produce no highlight). | BLOCKING |
| HU-12c | **GIVEN** the player drag-starts a LaneWide (Field) card during PLACEMENT, **WHEN** the cursor enters the board area, **THEN** the highlighted-cell set equals all cells of all 5 lane columns (full board excluding objectives). | BLOCKING |
| HU-12d | **GIVEN** the player drag-starts a TargetUnit card during a round where ≥ 1 valid target unit exists, **WHEN** the cursor hovers a valid unit, **THEN** the unit entity receives a `TargetUnitHover` marker component AND no `BoardCellHighlighted` markers are added (this is unit-targeting, not cell-targeting). Prism White outline rendering is ADVISORY. | BLOCKING |
| HU-13 | **GIVEN** the player stages a card by dropping it on a valid board target, **WHEN** the drop is confirmed, **THEN** (a) a `GhostPlacementChanged { target: Some(<resolved variant>), card_id: Some(card_id) }` message is written; (b) the fan slot enters `FanSlotState::Ghost` marker component; (c) the Submit button text updates to `"Submit (N cards)"` where N is the new count; (d) the staged card's reserve strip entity (Rule 13) becomes `Visible`. The 40% chroma / 50% opacity ghost rendering is ADVISORY. | BLOCKING |
| HU-14 | **GIVEN** the player drops a dragged card on an unhighlighted (invalid) target, **WHEN** the drop fires, **THEN** (a) the drag sprite returns to `Visibility::Hidden`; (b) the original fan slot returns to `FanSlotState::Active` marker component; (c) no `GhostPlacementChanged` message is written. | BLOCKING |
| HU-15 | **GIVEN** the player has 2 cards staged and the PLACEMENT timer reaches 0 while a third card is mid-drag, **WHEN** the 200ms grace window elapses without a mouse-up on a valid target, **THEN** `C2SSubmitPlacement` is sent with exactly the 2 staged placements; the in-flight card returns to its fan slot; the third card is NOT included. | BLOCKING — Integration |
| HU-15b | **GIVEN** the player has 2 cards staged and the PLACEMENT timer reaches 0 while a third Minion card is mid-drag over a valid highlighted board cell, **WHEN** the player releases mouse-up on that valid cell during the 200ms grace window, **THEN** the third card stages to that cell and `C2SSubmitPlacement` is sent with all 3 placements. | BLOCKING — Integration |
| HU-16 | **GIVEN** the player clicks Submit with 0 staged cards, **THEN** `C2SSubmitPlacement` is sent with an empty `placements` vec, the Submit button enters `Inactive` interaction state with text `"Submitted"`, and no confirmation modal entity is spawned. | BLOCKING |
| HU-17 | **GIVEN** the player clicks Submit once and the button becomes inactive, **WHEN** the player attempts to click Submit again, **THEN** no second `C2SSubmitPlacement` is written to the message queue. | BLOCKING |

### PLACEMENT — Pre-Validation (Rule 10)

| # | Criterion | Type |
|---|---|---|
| HU-17b | **GIVEN** the player has staged cards whose `sum(reserve_amount) > player.reserve_mana`, **WHEN** Submit is pressed, **THEN** (a) no `C2SSubmitPlacement` message is written; (b) the Submit button does NOT enter the `Inactive` state; (c) a `SubmitValidationError::ReserveOverdrawn` marker is attached to the Submit button entity (the Crimson inline label rendering is ADVISORY). | BLOCKING |
| HU-17c | **GIVEN** the player un-stages a card that was causing the reserve overdraw, **WHEN** Submit is pressed again, **THEN** pre-validation passes and `C2SSubmitPlacement` is sent. The previous `SubmitValidationError` marker is cleared. | BLOCKING |

### PLACEMENT — Instant Cards

| # | Criterion | Type |
|---|---|---|
| HU-18 | **GIVEN** the player drag-starts an Instant card during PLACEMENT, **WHEN** the drag sprite becomes visible, **THEN** (a) the fan plate entity receives a `FanPlateHighlighted` marker component; (b) the `BoardCellHighlighted` marker set on board cells is empty. The Prism White border + 0.5 Hz pulse rendering is ADVISORY. | BLOCKING |
| HU-19 | **GIVEN** the player drops an Instant card on the highlighted fan plate zone, **WHEN** the drop fires, **THEN** (a) the card stages with `target: PlayTarget::Instant` in the local pending queue; (b) `GhostPlacementChanged { target: Some(Instant), card_id: Some(card_id) }` is written; (c) the Submit count increments by 1. The 80 ms gold flash rendering is ADVISORY. | BLOCKING |

### PLACEMENT — TargetUnit Edge Case

| # | Criterion | Type |
|---|---|---|
| HU-20 | **GIVEN** the player drag-starts a TargetUnit card during a round where no valid target units exist on the board, **WHEN** the drag sprite is over the board, **THEN** (a) the `BoardCellHighlighted` marker set is empty; (b) a `NoValidTargetsOverlay` marker entity exists with `Visibility::Visible`; (c) drop anywhere returns the card to its fan slot via the Rule 6 step 4 path. The full-dim overlay rendering is ADVISORY. | BLOCKING |

### Un-Staging

| # | Criterion | Type |
|---|---|---|
| HU-21 | **GIVEN** a card is staged with a `BoardCell`, `TargetUnit`, `TargetObj`, or `LaneWide` target (board ghost active), **WHEN** Board Rendering writes a `GhostClickedEvent { card_id }` for that card, **THEN** Hand UI: (a) removes the card from the pending queue; (b) writes `GhostPlacementChanged { target: None, card_id: Some(card_id) }`; (c) the fan slot enters `FanSlotState::Active`; (d) Submit count decrements. | BLOCKING |
| HU-21b | **GIVEN** a card is staged with a board target, **WHEN** Board Rendering writes `GhostDragStartEvent { card_id }` (player mouse-down on ghost) and the player releases inside the hand fan zone, **THEN** Hand UI runs the same un-stage operation as HU-21. Submit count decrements. | BLOCKING |
| HU-21c | **GIVEN** a card is staged with `target: Instant` (no board ghost; only a dimmed fan slot ghost), **WHEN** the player clicks the dimmed fan slot, **THEN** Hand UI runs the same un-stage operation. Submit count decrements. | BLOCKING |

### PLACEMENT — Reserve Mana Split (Rule 13)

| # | Criterion | Type |
|---|---|---|
| HU-25 | **GIVEN** a card with `cost = 5` is staged AND `player.reserve_mana = 3`, **WHEN** the player clicks `[ + ]` on its reserve strip 3 times, **THEN** the strip's `reserve_amount` increments to 1, 2, 3; after the third click `reserve_amount == min(5, 3) = 3` so `[ + ]` immediately enters `Disabled` state. A fourth click produces no state change. | BLOCKING |
| HU-26 | **GIVEN** card A is staged with `reserve_amount = 2` AND `player.reserve_mana = 3`, **WHEN** card B (cost ≥ 2) is staged (default `reserve_amount = 0`) AND the player presses `[ + ]` on card B's reserve strip, **THEN** (a) card B's `reserve_amount` increments to 1 (ceiling = `player.reserve_mana − sum_other = 3 − 2 = 1`); (b) card B's `[ + ]` button immediately enters `Disabled` state; (c) card A's `reserve_amount` remains 2 (no auto-decrement occurs). | BLOCKING |
| HU-27 | **GIVEN** a card with `cost = 0` is staged, **WHEN** the staged ghost renders, **THEN** the reserve strip entity for that card has `Visibility::Hidden` (no decision available). | BLOCKING |

### Activation Lock (Rule 5c)

| # | Criterion | Type |
|---|---|---|
| HU-28 | **GIVEN** the player clicks an Instant card in hand during DRAFT_SHOP, **WHEN** `C2SActivateCard` is sent, **THEN** the card slot enters `HandSlotState::ActivationLocked` and clicks on it produce no further `C2SActivateCard` messages until one of: (a) `S2CGoldUpdate` is received; (b) `S2CActivationRejected` is received; or (c) `activate_timeout_ms` (3000 ms default) elapses. (`S2CCardAcquired` is NOT a valid unlock signal — see Rule 5c.) **Gate: OQ8 must be resolved before this AC can be fully implemented.** | BLOCKING |
| HU-28b | **GIVEN** an Instant card is in `HandSlotState::ActivationLocked` AND `S2CActivationRejected` is received, **THEN** the slot immediately reverts to `HandSlotState::Active` and clicks are accepted again (no timeout wait). **Gate: OQ8.** | BLOCKING |
| HU-29 | **GIVEN** an Instant card is in `ActivationLocked` state and `activate_timeout_ms` elapses with no S2C confirmation, **THEN** the slot reverts to `HandSlotState::Active` and clicks are accepted again. | BLOCKING |

### Hand Full Notification

| # | Criterion | Type |
|---|---|---|
| HU-30 | **GIVEN** the 10th card is acquired during DRAFT_INITIAL, **WHEN** the hand-full lock fires, **THEN** a `HandFullNotification` entity is spawned with a duration timer of `hand_full_notification_duration_ms` (2000 ms default); after the timer elapses (verifiable via virtual time advance) the entity is despawned. The notification's visual rendering is ADVISORY. | BLOCKING |

### Timer

| # | Criterion | Type |
|---|---|---|
| HU-22 | **GIVEN** the placement timer shows 5 seconds remaining, **WHEN** the 5-second threshold fires, **THEN** the timer entity enters `TimerState::Urgent` and a single `TimerUrgencyAudio` event is written exactly once (no looping audio system started). The Amber colour rendering is ADVISORY. | ADVISORY (visual + audio); BLOCKING (state + single-shot event) |
| HU-23 | **GIVEN** the player submits at 7 seconds remaining, **WHEN** Submit fires (and pre-validation passes), **THEN** (a) the timer continues decrementing each frame; (b) a `TimerSubmittedCheckmark` marker entity exists with `Visibility::Visible` adjacent to the timer numeral. | BLOCKING |

### Reconnect

| # | Criterion | Type |
|---|---|---|
| HU-24 | **GIVEN** the player reconnects during PLACEMENT (`S2CGameSnapshot` received with `phase = PLACEMENT`), **WHEN** Hand UI rebuilds, **THEN** (a) Hand UI's state machine is `STAGING`; (b) the local pending placements vec is empty; (c) the Submit button text reads `"Submit (0 cards)"`; (d) the timer's `remaining_ms` resource value equals `snapshot.timer_remaining_ms`; (e) the pre-pooled drag sprite entity has `Visibility::Hidden` (any in-flight drag at disconnect is cancelled and does not persist after rebuild). | BLOCKING — Integration |

## Open Questions

| # | Question | Owner | Notes |
|---|---|---|---|
| OQ1 | ~~**Reserve mana split UI for non-Xelor classes.**~~ **RESOLVED** — Rule 13 specifies a class-agnostic +/- control for any card with `cost > 0`. Class System GDD (M3) may add class-specific reserve mana *behaviours* (e.g., Xelor `Rollback`), but the Hand UI control's *interaction model* is fixed by this GDD. | — | Closed 2026-04-30 |
| OQ2 | ~~**Card zoom → click activation interaction.**~~ **RESOLVED** — Promoted to Rule 5b: click while zoomed activates immediately, no double-click barrier. | — | Closed 2026-04-30 |
| OQ3 | ~~Ready signal button (C2SSignalReady).~~ **RESOLVED** — Shop/Auction UI GDD owns the Ready/Retract Ready button (DRAFT_SHOP Rule 7, shop-auction-ui.md). Hand UI has no Ready button. `C2SSignalReady` is already registered in network-protocol.md. | — | Closed 2026-04-30 |
| OQ4 | ~~**GhostPlacementChanged interface.**~~ **RESOLVED** — Board Rendering is now Designed (status updated 2026-04-30). Payload extended in this revision pass to `{ target: Option<PlayTarget>, card_id: Option<CardId> }`. Reverse interfaces `GhostClickedEvent` and `GhostDragStartEvent` added to Board Rendering's outgoing messages (see Interactions table). | — | Closed 2026-04-30 |
| OQ5 | **Card ID → visual asset mapping** — at session start, Hand UI reads card definitions to get TextureAtlas frame indices. Is this via a `CardDataPlugin` resource loaded by `bevy_asset_loader`, or does Hand UI query Card Data & Pool directly? The asset loading architecture is not defined in this GDD. | Architecture / ADR | Needs an ADR for the client-side card data pipeline before implementation. |
| OQ6 | **Atlas sharing with Board Rendering** — Hand UI's `TextureAtlas` (Rule 1) and Board Rendering's unit atlas (Rule 5 of board-rendering.md, ≤ 15 draw call ceiling) — are these the same atlas or separate atlases? Separate atlases add a per-frame atlas-switch draw call during PLACEMENT (when both systems render simultaneously). Same atlas keeps the budget tight but couples asset pipeline. | Architecture / ADR | Needs decision before asset pipeline implementation. Recommend: shared atlas with all card art (units + cards reuse the same source) for batching, with separate atlas for board-element sprites (cells, prisms, objectives). |
| OQ7 | **Card art zoom resolution** — VA-1 specifies card hover-zoom from 120×180 px to 240×360 px (2× scale). Does the source `TextureAtlas` contain native 240×360 art (doubles atlas size), or does the zoom upscale 120×180 source (visible blur)? | Asset spec / Art Director | Resolve when `/asset-spec system:hand-ui` runs after the art bible is approved. Recommend: native 240×360 source for zoomed states; 120×180 derived via mipmap. |
| OQ8 | **`S2CActivationRejected` not in NP GDD.** Rule 5c and HU-28/HU-28b depend on `S2CActivationRejected` being added to the Network Protocol GDD. This message does not exist in NP as of 2026-04-30. **BLOCKING gate:** HU-28 and HU-28b cannot be implemented until this message is registered in NP. | Architecture / NP GDD | Add `S2CActivationRejected` to NP GDD (server → client, sent when `C2SActivateCard` is discarded due to wrong phase or invalid card state) before implementing the activation-lock story. |
