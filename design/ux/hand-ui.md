# UX Spec: Hand UI

> **Status**: Complete - pending /ux-review
> **Author**: user + ux-designer
> **Last Updated**: 2026-05-05
> **Template**: UX Spec
> **Journey Phase(s)**: DRAFT_INITIAL, DRAFT_SHOP, DRAFT_AUCTION, PLACEMENT, RESOLUTION
> **Input Methods**: Mouse + Keyboard. Primary: mouse. No gamepad. No touch. WASM browser.
> **Accessibility Tier**: Standard (`design/accessibility-requirements.md`)
> **Source Docs**: `design/gdd/hand-ui.md`, `design/ux/hud.md`, `design/ux/interaction-patterns.md`, `production/epics/hand-ui/*.md`
> **Related Specs**: `design/ux/hud.md`, `design/ux/interaction-patterns.md`, `design/gdd/shop-auction-ui.md`

---

## Purpose & Player Need

Hand UI lets the player read, choose, stage, and commit cards without losing the tactical thread of the round. It is not a collection screen; it is the player's execution surface during the most time-sensitive parts of play.

The player arrives at Hand UI wanting to answer three questions quickly:

| Question | UX answer |
|---|---|
| What cards do I have or can I buy? | DRAFT_INITIAL grid and bottom hand fan expose card art, cost, type, and purchase/hand state. |
| What can this card legally do now? | PLACEMENT drag states expose valid board cells, valid units, valid objectives, lane-wide zones, or Instant fan-plate staging. |
| Have I committed my plan? | Fan ghosts, board ghosts, reserve strips, Submit count, timer state, and Submitted state make the staged plan visible until RESOLUTION hides the hand. |

The core player need is decisive authority under a hard clock. The interface must make the player feel that placing a card, assigning a target, splitting reserve mana, and submitting is a controlled plan, not a fight against the UI.

The Hand UI must satisfy these player-facing principles:

- The hand is readable in roughly two seconds at up to 10 cards.
- Valid targets are discovered by dragging and are never inferred from color alone.
- Submission is explicit and irreversible from the player's perspective; no confirmation modal appears.
- Empty submission is valid.
- RESOLUTION hides the hand immediately so board consequences become the focus.

---

## Player Context on Arrival

Hand UI appears inside the in-game HUD rather than as a standalone destination. It changes behavior according to the Round State Machine phase.

| Phase | Player context | Emotional load | Hand UI posture |
|---|---|---|---|
| DRAFT_INITIAL | First hand-building moment. Player has a 9-card offering, 45 seconds, and an initial gold budget. | Curious, planning, moderate time pressure. | Grid-first selection surface with fan below as acquisition feedback. |
| DRAFT_SHOP | Player is evaluating shop and current hand after auction/shop events. | Calculating, lower pressure. | Fan is visible. Instant cards can be activated by click. Non-Instant cards are read-only. |
| DRAFT_AUCTION | Auction panel owns attention and input. Hand is tactical context only. | Competitive, price pressure. | Fan is visible but read-only in `PASSIVE_LOCKED`. |
| PLACEMENT | Player has about 10 seconds to stage and submit a batch plan. | Highest pressure, decisive. | Fan becomes the primary action surface; drag-to-stage, reserve split, Submit, timer, and errors are active. |
| RESOLUTION | Player watches the committed plan execute. | Spectator, consequence-reading. | Hand UI hides immediately. |
| Reconnect during PLACEMENT | Player returns mid-deadline with a server snapshot and no local pending placements. | Recovery, urgent reorientation. | Rebuilt STAGING state with current hand, empty staged queue, hidden drag sprite, and snapshot timer. |

No `design/player-journey.md` exists, so phase context is inferred from the GDD, HUD spec, and story files.

---

## Navigation Position

This screen lives at:

`Game Session -> HUD -> Hand UI`

Hand UI is a phase-owned gameplay surface. It is not reached from menus and has no independent route. It is entered and exited only through game phase transitions, reconnect snapshot recovery, and the local submit state transition inside PLACEMENT.

Related layout ownership:

- HUD owns persistent resource, phase, timer, objective, and class figurine zones.
- Hand UI owns the bottom hand fan, DRAFT_INITIAL hand interaction state, PLACEMENT staging interaction chain, Submit control, reserve strips, hand-specific notifications, and hand-specific error labels.
- Shop/Auction UI owns shop slots, auction bidding, and Ready/Retract Ready controls.
- Board Rendering owns board cells, units, objectives, board ghosts, and board-side highlight rendering.

---

## Entry & Exit Points

| Entry Source | Trigger | Player carries this context |
|---|---|---|
| Session start | `ClientState::InSession` entry initializes pre-pooled hand entities | Empty hand fan, hidden grid slots, hidden drag sprite, no pending placements |
| RSM phase change | `DRAFT_INITIAL` | 9-card draft offering, current gold, 45s timer |
| RSM phase change | `DRAFT_SHOP` | Current hand, current mana/reserve state, shop context |
| RSM phase change | `DRAFT_AUCTION` | Current hand for reading only; auction owns input |
| RSM phase change | `PLACEMENT` | Current hand, current mana, reserve mana, board state, spawn range, 10s timer |
| Local submit | `C2SSubmitPlacement` sent after pre-validation | Pending placements become submitted, interactions lock, timer remains visible |
| Reconnect | `S2CGameSnapshot { phase: PLACEMENT, timer_remaining_ms, hand }` | Current hand from snapshot, empty pending placements, remaining timer, no active drag |

| Exit Destination | Trigger | Notes |
|---|---|---|
| DRAFT_SHOP Hand UI | DRAFT_INITIAL ends normally | Grid hides; fan remains visible with confirmed hand state. |
| PLACEMENT Hand UI | DRAFT_SHOP ends | Fan enters STAGING; Submit and timer appear immediately. |
| RESOLUTION | Timer expiry, all-submit, or phase change | All Hand UI elements hide within one update. No exit animation. |
| SUBMITTED local state | Valid Submit press or timer auto-submit path | Submit button text becomes `Submitted`; no second submit can fire. |
| STAGING after failed submit | Client pre-validation fails | No phase/state transition. Inline error appears; player corrects staging locally. |
| STAGING after invalid drop | Mouse-up outside valid target | Drag cancels; original fan slot restores; no board ghost message. |

---

## Layout Specification

### Information Hierarchy

PLACEMENT hierarchy:

| Priority | Information | Reason |
|---|---|---|
| 1 | Time remaining and Submit state | The player must know whether decisions are still open. |
| 2 | Active dragged card and valid target set | Determines the immediate action. |
| 3 | Staged cards and their destinations | Confirms the batch plan before irreversible submit. |
| 4 | Mana affordability and reserve allocation | Prevents invalid submits and supports tactical reserve use. |
| 5 | Remaining hand cards | Supports next-card planning once the current card is staged/cancelled. |
| 6 | Errors and recovery states | Must be visible but not steal attention from valid action paths. |

DRAFT_INITIAL hierarchy:

| Priority | Information | Reason |
|---|---|---|
| 1 | 9-card offering | The offering is the main decision surface. |
| 2 | Card cost and identity | Player needs fast purchase evaluation. |
| 3 | Purchase pending/confirmed state | Server confirmation is authoritative; the UI cannot imply optimistic ownership. |
| 4 | Hand count/fan feedback | Shows acquired cards and hand-full lock. |
| 5 | Timer | Important but less urgent than PLACEMENT. |

DRAFT_AUCTION and RESOLUTION reduce hand priority. In DRAFT_AUCTION, the fan is context only. In RESOLUTION, the hand is hidden.

### Layout Zones

Hand UI follows the HUD perimeter-ring philosophy. The board center remains unobstructed except for DRAFT_INITIAL's modal grid and temporary drag/highlight feedback.

| Zone | Contents | Phase visibility | Constraints |
|---|---|---|---|
| Bottom fan strip | Hand fan plate, up to 10 fan card slots, fan ghosts, Instant plate highlight | DRAFT_INITIAL, DRAFT_SHOP, DRAFT_AUCTION, PLACEMENT | Absolute-positioned shallow fan row. No empty slot frames. At 10 cards, all mana badges remain readable. |
| Center modal grid | 3x3 DRAFT_INITIAL offering grid | DRAFT_INITIAL only | Centered overlay with dim backing. Blocks board and other non-grid input. |
| Board interaction layer | Valid cell highlights, unit hover outline, no-valid-target overlay, board ghosts | PLACEMENT only | Board Rendering owns rendering; Hand UI writes/consumes ghost and highlight intent. |
| Submit cluster | Submit button, Submit count, inline validation error | PLACEMENT and SUBMITTED | Fixed above fan. Never shifts when fan count or reserve strips change. |
| Reserve strip layer | Per-staged-card `[-] [N / cost] [+]` strips | PLACEMENT STAGING only | Anchored 8px above each staged fan ghost. May overlap horizontally; edge buttons remain reachable. |
| Timer cluster | Whole-second PLACEMENT timer, submitted checkmark | PLACEMENT and SUBMITTED | Upper-right or HUD timer slot per `design/ux/hud.md`; never over animated board content without backing. |
| Notification layer | Hand full toast, no valid targets text, submit validation label | Contextual | Near the cause: hand full near fan, no valid targets over board, submit errors under Submit. |

### Hand Fan Layout

The GDD requires absolute positioning with a shallow fan arc; the art/HUD docs require row readability at 10 cards. This spec resolves the two constraints as a **shallow fan row**:

- Use the GDD formulas for `t`, `card_x`, `card_y`, and `card_rotation_deg`.
- Keep default `max_rotation_deg` at 10 degrees and `arc_height` at 10px unless testing shows badge readability failure.
- Treat the result visually as a row with fan character, not a deep radial fan.
- At 10 cards, cost badges, ATK, HP, and card type/rarity must remain identifiable.
- At 0 cards, no fan slots render; Submit remains available during PLACEMENT.
- At 1 card, the card is centered with no rotation.
- At 2 cards, both cards use the full `t = -1.0` and `t = +1.0` endpoints.

### Component Inventory

| Component | Type | Content | Interactive | Pattern |
|---|---|---|---|---|
| Hand fan plate | UI container/drop zone | Background plate for card fan and Instant drop zone | Yes in PLACEMENT for Instant drops | New: Fan Plate Drop Zone |
| Fan card slot | Card display/action | Card art, cost, stats, name, type, slot state | Phase-dependent | Extends PTN-DSP-008 Horizontal Card Row |
| Fan ghost | Staged-state display | Desaturated staged card identity | Instant ghost clickable for un-stage | New: Staged Card Ghost |
| Drag sprite | Follow-cursor affordance | Card art clone and stat badges | Pointer-driven only | New: Drag-to-Stage Card |
| DRAFT_INITIAL grid slot | Purchasable card tile | Card art, name, mana cost, rarity, pending/locked state | Yes until pending/locked/hand full | New: Draft Offering Grid Slot |
| Submit button | Primary action | `Submit (N cards)` or `Submitted` | Yes until submitted/pre-validation pass | PTN-NAV-001 variant |
| Inline submit error | Feedback | `Reserve overdrawn`, `Mana overdrawn`, `Out-of-range placement` | No | PTN-FDB-002 Inline Error Message |
| Reserve strip | Stepper control | `[-] [N / cost] [+]` | Yes before submit | New: Compact Stepper Strip |
| PLACEMENT timer | Countdown display | Whole seconds, urgent state, submitted checkmark | No | PTN-DSP-005 Urgency Countdown Timer |
| Hand full notification | Temporary toast | `Hand full` | No | New: Notification Toast (pattern library gap) |
| No valid target overlay | Board overlay | Text plus full-board dim state | No | New: No Valid Target Overlay |
| Read-only auction label | Small status label | `Auction in progress` | No | PTN-DSP-004 style text label |

### ASCII Wireframe

Default PLACEMENT:

```text
+--------------------------------------------------------------------------+
| [own class]        [objective dots] [PHASE / ROUND]        [opp class]   |
|                                                            [TIMER 08]    |
|                                                                          |
|               BOARD: valid target highlights / board ghosts              |
|                                                                          |
|                    [no valid targets overlay, if needed]                 |
|                                                                          |
|                              [Submit (2 cards)]                          |
|                              [inline error if any]                       |
|                                                                          |
|          [-][0/3][+]        [-][2/5][+]                                  |
|   [card] [ghost] [card] [ghost] [card] [card] [card]                     |
+--------------------------------------------------------------------------+
```

DRAFT_INITIAL:

```text
+--------------------------------------------------------------------------+
| [HUD top strip remains visible]                              [TIMER 45]  |
|                                                                          |
|                       +--------------------------+                       |
|                       |   3x3 DRAFT OFFERING    |                       |
|                       |  [card] [card] [card]   |                       |
|                       |  [card] [card] [card]   |                       |
|                       |  [card] [card] [card]   |                       |
|                       +--------------------------+                       |
|                                                                          |
|        [fan receives confirmed purchases; no empty ghost slots]          |
+--------------------------------------------------------------------------+
```

---

## States & Variants

| State / Variant | Trigger | What changes |
|---|---|---|
| HIDDEN | LOBBY or RESOLUTION | Fan root, grid, Submit, timer, drag sprite, reserve strips hidden. No input. |
| GRID | DRAFT_INITIAL entry and offering received | 3x3 grid visible. Fan below shows confirmed cards. Click-to-buy enabled per available slot. |
| Grid slot pending | Player clicks a DRAFT_INITIAL grid card | Slot enters pending state; click suppressed; waits for `S2CCardAcquired` or timeout. |
| Grid slot confirmed | `S2CCardAcquired` matches clicked card | Slot hides or shows short check state; acquired card animates to fan. |
| Hand full | Local confirmed hand count reaches 10 | Remaining visible grid slots lock; clicks suppressed; `Hand full` notification appears for 2s near fan. |
| Purchase timeout | No acquisition confirmation after `purchase_timeout_ms` | Slot returns to available. Gold remains unchanged. |
| PASSIVE | DRAFT_SHOP | Fan visible. Instant cards clickable. Drag-start suppressed for all cards. |
| Activation locked | Instant card clicked in DRAFT_SHOP | Slot locks until `S2CGoldUpdate`, `S2CActivationRejected`, or `activate_timeout_ms`. |
| PASSIVE_LOCKED | DRAFT_AUCTION | Fan visible at reduced opacity, no input, `Auction in progress` label above fan. |
| STAGING | PLACEMENT entry | Fan fully active; Submit visible as `Submit (0 cards)`; timer visible; pending placements empty. |
| Dragging card | Mouse-down on fan card in PLACEMENT | Original card hides; drag sprite follows cursor; original slot remains as dim placeholder. |
| Valid board target hover | Dragged card has legal board targets | Valid cells/units/objectives/lanes highlight according to card target type. |
| No valid target | TargetUnit card has no valid units | Board dims with text indicator; no highlights; mouse-up anywhere cancels drag. |
| Invalid drop | Mouse-up outside legal target/zone | Drag sprite hides; original slot returns active; no ghost message written. |
| Staged board card | Mouse-up on legal board target | Fan slot becomes ghost; board ghost appears via Board Rendering; Submit count increments; reserve strip appears if cost > 0. |
| Staged Instant card | Instant dropped on highlighted fan plate | Fan ghost is the only ghost; Submit count increments; no board ghost renders. |
| Un-staging | Board ghost clicked, board ghost dragged to fan, or Instant fan ghost clicked | Pending placement removed; board ghost clears; fan slot active; Submit count decrements. |
| Reserve strip disabled plus | Reserve allocation reaches per-card ceiling | `+` disabled immediately. No click effect. |
| Invalid submit | Client pre-validation fails | Submit remains active. Inline error appears. No `C2SSubmitPlacement` is sent. |
| Timer grace | Timer reaches 0 during active drag | 200ms window opens. Valid mouse-up can still stage the card before auto-submit. |
| SUBMITTED | Valid submit or timer resolution sends placement batch | Submit becomes inactive with `Submitted`; cards dim; staging locks; timer continues and shows submitted checkmark. |
| Reconnect STAGING | Snapshot says `phase = PLACEMENT` | Fan rebuilt from snapshot hand; pending placements empty; drag sprite hidden; Submit `Submit (0 cards)`; timer from snapshot. |
| Reconnect timer 0 | Snapshot timer is 0 | Expected UX: show Submitted/locked state while waiting for RESOLUTION phase message. This remains an open confirmation item. |

---

## Interaction Map

Mapping assumes Mouse + Keyboard on WASM browser. Mouse is the primary input. Keyboard focus is required for accessibility and layout evidence, even where Sprint 1 implementation stories originally scoped gameplay input to mouse.

| Component | Player action | Mouse input | Keyboard input | Immediate feedback | Outcome |
|---|---|---|---|---|---|
| DRAFT_INITIAL grid slot | Buy card | Click slot | Focus slot, Enter/Space | 60ms gold bloom; pending state | Send `C2SPurchaseCard`; wait for confirmation. |
| Pending grid slot | Retry after timeout | Click after timeout reverts | Focus slot, Enter/Space | Pending state returns only after click | New purchase attempt. |
| Locked grid slot | Try to buy while hand full | Click suppressed | Focus skipped or announced disabled | Muted locked state; optional `Hand full` notification if lock just happened | No message. |
| Fan card in DRAFT_SHOP | Activate Instant card | Click card | Focus card, Enter/Space | Slot locks/hides input | Send `C2SActivateCard`; wait for ack/reject/timeout. |
| Fan card in DRAFT_SHOP | Drag attempt | Mouse down + move | N/A | No lift, no drag sprite | Gesture absorbed. |
| Fan card in DRAFT_AUCTION | Attempt input | Click/drag | Focus skipped or read-only | No click sound; read-only label remains | No message. |
| Fan card in PLACEMENT | Start staging | Mouse down on card | Focus card, Enter then choose target by keyboard | Card hides; drag sprite appears; valid targets show | Drag ownership begins. |
| Board target | Stage board-target card | Mouse-up on highlighted cell/unit/objective/lane | Enter/Space on focused valid target | Placement thunk; fan ghost; board ghost | Add pending placement; write `GhostPlacementChanged`. |
| Fan plate | Stage Instant card | Mouse-up on highlighted fan plate | Enter/Space on focused fan plate | Plate flashes gold; fan ghost dims | Add pending Instant placement; write `GhostPlacementChanged`. |
| Invalid board/outside area | Cancel drag | Mouse-up outside valid target | Escape while dragging | Snap-back 100-220ms | No pending placement; no ghost message. |
| Board ghost | Un-stage | Click board ghost | Focus ghost, Enter/Space | Ghost clears; fan card restores | Consume `GhostClickedEvent`; write clear `GhostPlacementChanged`. |
| Board ghost | Drag back to fan | Mouse down on ghost, release in fan zone | Keyboard equivalent: focused ghost, Backspace/Delete | Fan zone highlights as return zone | Consume `GhostDragStartEvent`; remove pending placement if released in fan. |
| Instant fan ghost | Un-stage Instant | Click dimmed fan slot | Focus ghost, Backspace/Delete or Enter | Ghost clears; Submit count decrements | Remove pending Instant placement. |
| Reserve `-` | Decrease reserve split | Click `-` | Focus `-`, Enter/Space | Number updates same frame; soft click | Decrement clamped to 0. |
| Reserve `+` | Increase reserve split | Click `+` | Focus `+`, Enter/Space | Number updates same frame; plus disables at ceiling | Increment clamped to card cost and remaining reserve. |
| Submit button | Submit valid batch | Click Submit | Focus Submit, Enter/Space | Button press, permanent submit sound, text `Submitted` | Pre-validate; send `C2SSubmitPlacement`; lock staging. |
| Submit button | Submit invalid batch | Click Submit | Focus Submit, Enter/Space | Inline error appears under button | No send; player adjusts reserve/staging. |
| Timer grace | Complete in-flight drag after 0 | Mouse-up on valid target within 200ms | Enter/Space on focused valid target within grace | Stage feedback then auto-submit | Include card in submitted batch. |
| Timer grace expiry | No valid mouse-up in grace | No action | No action | Drag cancels; card returns | Auto-submit existing staged placements only. |

### Keyboard Focus Order

DRAFT_INITIAL focus order:

1. Grid slots, row-major, available slots only.
2. Confirmed/empty grid slots are skipped.
3. Locked grid slots may be focusable only if the UI can announce `Hand full`; otherwise skip disabled slots.
4. Fan cards are read-only during DRAFT_INITIAL and should not interrupt the purchase loop unless needed for card inspection.

PLACEMENT focus order:

1. Fan cards left-to-right.
2. If a card is selected for keyboard staging, focus moves to the valid target set: board cells row-major from player's perspective, valid units in lane order, opponent objective cells in lane order, lane-wide columns left-to-right, or fan plate for Instant.
3. Staged card controls: reserve `-`, reserve display, reserve `+`, then staged ghost un-stage control, grouped by fan position.
4. Submit button.
5. Timer is not focusable because it is read-only.

Focus behavior:

- PLACEMENT traps focus inside the game canvas.
- Every focused interactive element has a 2px Prism White outline or equivalent high-contrast focus ring.
- Escape cancels an active drag/selection before it leaves PLACEMENT focus trap.
- Reduced-motion mode must not remove focus visibility.

---

## Events Fired

| Player Action | Event / Message Fired | Payload / Data | Notes |
|---|---|---|---|
| Click DRAFT_INITIAL grid card | `C2SPurchaseCard` | Card ID / offering slot identity as defined by NP | No optimistic hand mutation. |
| Receive purchase confirmation | No player event; reacts to `S2CCardAcquired` | Confirmed card ID | Grid slot hides; fan updates from authoritative hand state. |
| Purchase timeout | No C2S | Slot ID, elapsed timeout locally | Reverts pending visual state only. |
| Click Instant card in DRAFT_SHOP | `C2SActivateCard` | Card ID | Slot enters activation lock. |
| DRAFT_SHOP activation rejected | No player event; reacts to `S2CActivationRejected` | Card ID / reason if defined later | NP message is still a GDD blocker. |
| Drop board-target card on valid target | `GhostPlacementChanged` | `{ target: Some(PlayTarget), card_id: Some(CardId) }` | Bevy-internal message for Board Rendering. |
| Drop Instant card on fan plate | `GhostPlacementChanged` | `{ target: Some(Instant), card_id: Some(CardId) }` | Board Rendering renders no board ghost. |
| Un-stage board ghost | `GhostPlacementChanged` | `{ target: None, card_id: Some(CardId) }` | Triggered after consuming board ghost click/drag event. |
| Invalid drop | None | None | Explicitly no ghost message. |
| Submit valid batch | `C2SSubmitPlacement` | Pending placements with `card_id`, `PlayTarget`, `reserve_amount` | Persistent game-state intent; server remains authoritative. |
| Submit invalid batch | None | Validation error kind | Local inline error only. |
| Timer reaches urgency threshold | `TimerUrgencyAudio` | None or timer context | Fires exactly once at threshold. |
| Timer expires after grace resolution | `C2SSubmitPlacement` if not already submitted | Existing staged placements, possibly including grace-window drop | No duplicate after manual submit. |
| Reconnect snapshot processed | None from player | Snapshot phase, hand, timer remaining | Local pending placements clear; no ghost animations. |

Analytics are not specified in the current GDDs. If analytics are added, they should mirror the same action names without changing gameplay messages.

---

## Transitions & Animations

Animations confirm state change. They must never consume meaningful PLACEMENT time or obscure the board.

| Transition | Animation | Duration | Reduced-motion behavior |
|---|---|---|---|
| DRAFT_INITIAL grid appears | Fade in backing and grid at fixed center position | 80-150ms | Instant appear. |
| Grid card purchase send | Gold bloom flash on slot | 60ms | Static pending state. |
| Purchase confirmed | Card slides from grid slot to fan | 280ms default, max 400ms | Instant fan update plus brief check state. |
| Fan hover / inspect | Scale to 240x360 zoom tier, lift from fan | 80ms | Instant scale or static enlarged detail. |
| PLACEMENT drag start | Card lifts to drag sprite at 1.10 scale | Immediate to 80ms | Instant drag sprite. |
| Valid target highlights | Appear instantly | 0ms | Same. |
| Valid drop | Weighted stage click; fan ghost dims; board ghost appears | 80-120ms | Instant ghost state. |
| Invalid drop | Snap-back to original fan slot | 100-220ms | Instant restore. |
| Instant fan plate highlight | Prism White border pulse at 0.5Hz | While dragging Instant | Static border. |
| Reserve adjustment | Numeric value updates same frame, soft click | 0-50ms | Same without sound if UI audio disabled. |
| Submit valid | Button press, label changes to `Submitted`, submit ring sound | Press 80ms, sound 400ms | Static state change, no scale pulse. |
| PLACEMENT timer urgent | Per-second scale pulse and color step | 120ms per second | Remove pulse, keep text and color/shape state. |
| RESOLUTION entry | All Hand UI hides immediately | 0ms | Same. |
| Reconnect rebuild | No animation | 0ms | Same. |

Hard constraints:

- No Hand UI animation during PLACEMENT may exceed 250ms.
- RESOLUTION hide has no exit animation.
- Reconnect does not replay card draw, stage, or resource animations.
- Timer is backed by a semi-opaque panel and never floats directly over animated board content.

---

## Data Requirements

| Data | Source System | Read / Write | Notes |
|---|---|---|---|
| Current hand `Vec<CardId>` | Card Acquisition / Network | Read | Server-authoritative. Hand UI never asserts hand state. |
| DRAFT_INITIAL offering | Network Protocol | Read | 9 card IDs from `S2CDraftOffering`. |
| Card name, art, cost, stats, type, rarity | Card Data & Pool / asset pipeline | Read | Needed for grid, fan, drag sprite, target category, reserve strip. |
| Current mana | Economy | Read | Used by submit pre-validation and HUD mana display. |
| Reserve mana | Economy | Read | Used by reserve strip ceilings and submit pre-validation. |
| Pending placements | Hand UI local presentation state | Write local only | Cleared on PLACEMENT entry, submit, RESOLUTION, reconnect. Sent only as C2S submit payload. |
| Board layout | Board Rendering | Read | Cursor-to-board mapping during PLACEMENT. |
| Valid spawn cells, valid target units/objectives/lanes | Board/Lane, Objective, Board Rendering | Read | Drives highlight sets and no-valid-target overlay. |
| RSM phase | Round State Machine / shared phase sink | Read | Drives state machine. |
| Timer duration and remaining time | Game Config / RSM / snapshot | Read/write local countdown | Snapshot remaining time overrides default duration on reconnect. |
| Activation lock state | Hand UI local presentation state | Write local only | Clears on ack/reject/timeout. |
| Purchase pending state | Hand UI local presentation state | Write local only | Clears on acquisition, timeout, hand-full lock, or phase exit. |
| Accessibility timer multiplier | Settings / accessibility | Read | Applies to PLACEMENT duration and urgency proportions. |

Architectural guardrails:

- UI does not own authoritative hand, mana, reserve, board, or objective state.
- Local pending placements are intent staging, not server truth.
- Pre-validation is defense-in-depth and does not replace server validation.
- Reconnect clears local pending placements because snapshots do not include in-progress staging.

---

## Accessibility

Hand UI follows the project's Standard accessibility tier.

### Visual Accessibility

- Card cost, ATK, HP, and type/rarity must remain readable at 10-card hand compression.
- ATK orange and HP teal are reserved stat colors and must not be repurposed for Hand UI warnings or highlights.
- Valid target highlights are not color-only. BoardCell target availability is also expressed by presence/absence of highlight; TargetUnit uses outline shape; no-valid-target uses text overlay.
- Invalid cells do not show red overlays. Red/Crimson is reserved for combat and error text.
- Submit errors use text labels, not color alone.
- Hand full state uses muted/locked visuals plus notification text.
- Reserve mana uses diamond/strip labeling and numeric `N / cost`, not color alone.
- Focus indicators use high-contrast outline and are visible at browser zoom 75%-150%.

### Motor Accessibility

- Submit button and reserve strip buttons must provide at least 44x44 CSS px effective pointer targets where layout allows. If the visual reserve buttons remain 24x24, their hit area must be expanded invisibly or keyboard operation must be fully supported.
- PLACEMENT focus is trapped in the game canvas.
- PLACEMENT timer multiplier supports 0.5x, 1x, 1.5x, 2x, and 3x unless OQ-HUD-5 changes the range.
- No hold-to-confirm interaction exists in Hand UI.
- Drag interactions must have keyboard alternatives for final accessibility sign-off: select card, move focus through valid targets, confirm target, adjust reserve, submit.

### Cognitive Accessibility

- The reserve strip appears only after a card is staged; it is not shown before the player has chosen a card and destination.
- Submit count uses text (`Submit (N cards)`) rather than relying on ghost count only.
- Submitted state is explicit and persistent until RESOLUTION.
- Reconnect state does not replay stale animations; it presents current state directly.
- No confirmation modal appears after Submit; the absence of a modal is part of the skill expression and must be taught by state feedback, not by surprise.

### Motion and Audio Accessibility

- Reduced-motion mode removes hover bounce, timer pulse, plate pulse, and card slide motion while preserving state changes.
- Timer urgency has one audio cue at threshold and visible numeric backup.
- No looping timer audio runs during PLACEMENT.
- Hand audio belongs to the `ui_hand` channel for independent volume control.

### Screen Reader Scope

In-game board screen reader support is out of current scope because Bevy 0.18 browser accessibility integration is not established in the project. Menus and overlay controls should still carry accessible labels where the UI stack supports them. For Hand UI, keyboard focus labels should at minimum expose:

- Card name, cost, type, and hand position.
- Grid slot state: available, pending, confirmed, locked/hand full.
- Reserve strip value and per-card ceiling.
- Submit count and submitted/disabled state.
- Timer seconds remaining.

---

## Localization Considerations

Hand UI is mostly icon, number, and card-art driven, but several labels must tolerate translation expansion.

| Text | Max English copy | Layout rule |
|---|---|---|
| Submit active | `Submit (10 cards)` | Fixed-width button. Allow 40% expansion or abbreviate count format per locale. |
| Submit locked | `Submitted` | Same button footprint as active Submit. No layout shift. |
| Hand full notification | `Hand full` | Toast near fan. Width can expand upward/left; must not cover Submit. |
| No valid target overlay | `No valid targets` | Centered over board dim. Can wrap to two lines without covering fan. |
| Reserve overdraw error | `Reserve overdrawn` | One line under Submit. If localized copy exceeds width, wrap to two compact lines rather than shrink below 14px. |
| Mana overdraw error | `Mana overdrawn` | Same as above. |
| Out-of-range error | `Out-of-range placement` | Same as above. |
| Auction read-only label | `Auction in progress` | Above fan, small status label. Can expand horizontally. |
| Reserve strip value | `N / cost` | Numeric and symbolic; no translation needed except spacing. |

Numbers must use locale-safe formatting only where grouping is expected. Mana, reserve, timer seconds, and card counts should not use thousands separators in normal play.

---

## Acceptance Criteria

- [ ] Hand fan renders as a shallow absolute-positioned fan row with 0, 1, 2, 5, and 10-card cases matching GDD formulas and preserving card badge readability.
- [ ] DRAFT_INITIAL displays exactly 9 offering slots in a centered 3x3 grid, with available, pending, confirmed, timeout, and hand-full locked states visually distinct.
- [ ] Clicking an available DRAFT_INITIAL grid card sends purchase intent but does not update the hand until server confirmation.
- [ ] When hand count reaches 10, all remaining grid cards lock, clicks are suppressed, and a `Hand full` notification appears near the fan for 2 seconds.
- [ ] DRAFT_AUCTION shows the fan in read-only `PASSIVE_LOCKED` state with input suppressed and no click/drag feedback.
- [ ] PLACEMENT entry shows Submit as `Submit (0 cards)`, activates the fan for staging, and displays the PLACEMENT timer immediately.
- [ ] Drag-start from a fan card hides the original card, shows a drag sprite, and preserves a stable fan slot placeholder.
- [ ] Dropping a card on a valid board target stages the card, creates/updates the board ghost through `GhostPlacementChanged`, dims the fan ghost, and increments Submit count.
- [ ] Dropping a card on an invalid target or outside the valid zone restores the card to its original fan slot and emits no ghost message.
- [ ] Instant cards in PLACEMENT highlight the fan plate as the valid drop zone, stage to `PlayTarget::Instant`, and can be un-staged from the dimmed fan ghost.
- [ ] Board-target staged cards can be un-staged by board ghost click or by dragging the board ghost back to the fan zone.
- [ ] Reserve mana strip appears above each staged card with `cost > 0`, uses single-click `-` and `+` controls, clamps to card cost and remaining reserve, and hides for `cost == 0`.
- [ ] Submit pre-validation blocks invalid batches, keeps Submit active, shows a one-line inline error, and sends no `C2SSubmitPlacement`.
- [ ] Valid Submit sends exactly one `C2SSubmitPlacement`, changes the button to `Submitted`, locks staging interaction, and keeps the timer visible with a submitted checkmark.
- [ ] Timer expiry during active drag opens a 200ms grace window; a valid release during grace includes the card, while grace expiry cancels the drag and submits only already staged cards.
- [ ] Reconnect during PLACEMENT rebuilds STAGING from snapshot hand, clears pending placements, hides the drag sprite, sets Submit to `Submit (0 cards)`, and uses snapshot timer remaining.
- [ ] RESOLUTION entry hides all Hand UI elements within one update and does not run exit animations.
- [ ] PLACEMENT traps keyboard focus inside the game canvas and all interactive Hand UI elements have visible focus indicators.
- [ ] Keyboard operation can reach DRAFT_INITIAL grid slots, PLACEMENT fan cards, valid targets, reserve controls, Instant fan ghosts, and Submit in a logical order.
- [ ] Reduced-motion mode preserves all state communication while removing nonessential motion and pulses.

---

## Cross-Reference Check

| Source requirement area | Covered in this spec |
|---|---|
| Hand fan layout | Layout Specification, States & Variants, Acceptance Criteria |
| DRAFT_INITIAL grid interactions | Layout Specification, Interaction Map, States & Variants |
| PLACEMENT staging/unstaging | Interaction Map, Events Fired, States & Variants |
| Reserve mana strip | Layout Zones, Interaction Map, Data Requirements, Accessibility |
| Timer/grace/submitted states | States & Variants, Transitions, Acceptance Criteria |
| Reconnect state | Player Context, States & Variants, Data Requirements |
| Hand full state | States & Variants, Interaction Map, Acceptance Criteria |
| No valid target state | States & Variants, Accessibility, Acceptance Criteria |
| Invalid submit state | Interaction Map, Events Fired, Acceptance Criteria |
| Accessibility and keyboard focus | Interaction Map, Accessibility, Acceptance Criteria |

Pattern library gaps introduced or strengthened:

| Pattern gap | Needed by |
|---|---|
| Fan Plate Drop Zone | PLACEMENT Instant card staging |
| Staged Card Ghost | PLACEMENT staged plan readability |
| Drag-to-Stage Card | PLACEMENT card play |
| Draft Offering Grid Slot | DRAFT_INITIAL purchase flow |
| Compact Stepper Strip | Reserve mana allocation |
| Notification Toast | Hand full and future round summary |
| No Valid Target Overlay | TargetUnit edge case |

---

## Story Traceability

| Story | UX surface unblocked by this spec | Status in source docs |
|---|---|---|
| 001 - Plugin Scaffold | Entity pool has UX ownership and visibility states defined | Complete |
| 002 - Fan Layout Formula | Shallow fan row layout and readability evidence criteria | Complete |
| 003 - Phase State Machine | Phase-to-mode presentation states and input gating | Complete |
| 004 - DRAFT_INITIAL Grid | Grid slot states, purchase feedback, hand-full lock | Complete |
| 005 - Placement Submit Core | Submit placement, invalid drop, submitted state | Complete |
| 006 - Placement Drag Highlights | Valid target, no-valid-target, highlight behavior | Complete |
| 007 - Placement Instant Staging | Fan plate drop zone and Instant fan ghost | Complete |
| 008 - Placement Un-Staging | Board ghost click/drag and Instant ghost un-stage | Complete |
| 009 - Placement Timer | Urgency, grace window, submitted checkmark | Complete |
| 010 - Submit Pre-Validation | Invalid submit label and recovery loop | Ready |
| 011 - Reserve Mana Strip | Strip anatomy, disabled states, accessibility notes | Complete |
| 012 - Activation Lock | DRAFT_SHOP lock state UX, but implementation remains blocked by NP OQ8 | Blocked |
| 013 - Reconnect Rebuild | Snapshot rebuild UX and timer recovery state | Ready |

This spec primarily unblocks visual/layout evidence and later polish for Hand UI and the hand/shop panel integration. It does not resolve the Network Protocol blocker for Story 012.

---

## Open Questions

| # | Question | Owner | Priority |
|---|---|---|---|
| OQ-HU-UX-1 | Keyboard scope conflict: the Hand UI GDD says no Hand UI keybindings in Sprint 1, while this UX spec defines keyboard focus and operation for Standard accessibility. Should implementation treat keyboard operation as visual polish, accessibility hardening, or immediate acceptance scope? | producer / ux-designer | High |
| OQ-HU-UX-2 | Fan layout wording: confirm that "shallow fan row" is the intended reconciliation between the GDD's absolute fan formulas and the art/HUD readability warning against a deep fan at 10 cards. | ux-designer / art-director | Medium |
| OQ-HU-UX-3 | `S2CActivationRejected` is still absent from the Network Protocol GDD. Story 012 remains blocked until that message or an equivalent rejection/confirmation path is registered. | architecture / network-protocol | High |
| OQ-HU-UX-4 | Reconnect with `timer_remaining_ms = 0`: should the client show `Submitted` with zero placements while waiting for RESOLUTION, or hide Hand UI immediately if the server has already auto-submitted? Story 013 flags this as designer confirmation. | game-designer / ux-designer | Medium |
| OQ-HU-UX-5 | Reserve strip visual size: GDD VA-9 specifies 96px width; Story 011 implementation evidence notes 104px. Which width should the final visual polish target enforce? | ux-designer / ui-programmer | Low |
| OQ-HU-UX-6 | Card zoom asset resolution and card atlas sharing remain architecture/asset-pipeline questions from GDD OQ5-OQ7. These affect hover zoom quality and batching, not the interaction model. | architecture / art-director | Medium |
