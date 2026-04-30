# Card Animations

> **Status**: In Review (Pass 2 — pending re-review)
> **Author**: Sam + design-system skill
> **Last Updated**: 2026-04-30
> **Implements Pillar**: No idle spectating · Simple surface (animations as silent tutorial)

## Overview

**Card Animations** is the client-side animation pipeline — a thin polish layer that owns *how* visual transitions feel without owning *when* they happen or *what* they depict. It is built on **`bevy_tweening`** and exposes a shared library of **lenses, easing curves, duration constants, and animator-lifecycle utilities** consumed by every visible system: Hand UI (card-to-fan slides, drag-lift, snap-back), Board Rendering (unit advance, fog-lift, health-bar fill, objective destruction), Combat Resolution (placement-reveal flip, damage-number floats, death fades, COUNTERATTACK and RANGE projectiles), Shop/Auction UI (panel transitions, gold-counter ticking, timer-bar ease, NO BIDS desaturation), and Keyword System visuals (REPEL/ATTRACT displacement, TRAP card-flip).

The system has two faces. The **data layer** is the `Animator<T>` lifecycle, the custom `SpriteAlphaLens` (and any other lenses 0.18 doesn't ship), the cancel-and-replace tween protocol that prevents game-state loss when units die mid-animation, and the small set of timing constants that downstream systems pull from `GameConfig` (`pre_animation_pause_ms`, `resolution_sub_step_duration_ms`, `inter_step_pause_ms`, `fog_lift_duration_ms`, `card_draw_animation_ms`, `snap_back_duration_ms`). The **player-facing layer** is what makes RESOLUTION readable, makes the auction settlement feel earned, and makes 10-second placement feel tight without feeling rushed. When animations are correct the player never thinks about them — they just absorb information faster than they realise. When animations are wrong the game feels broken even though every formula is correct.

**Critical boundary:** Card Animations contains zero gameplay logic. It never delays a phase transition (the `GAME_OVER` message can override any animation in flight), never holds server-authoritative state (units' true positions are written by ECS systems consuming Lightyear messages — animators only translate the visual `Transform`), never produces randomness (no client-side RNG), and is never replayed on reconnect (full board rebuild from `S2CGameSnapshot` discards all in-progress tweens). The system exists to translate the server's authoritative event stream into legible motion.

## Player Fantasy

The player should feel like the game **respects their time**. Every animation in Lanes and Lies is load-bearing — the gold counter ticks because you need to see the `+interest` land before you bid; the timer bar pulses red because the auction tempo *is* the pressure; the fog lifts because RESOLUTION is *starting* and your attention should already be on the board. Nothing here is decoration. The hardcore audience this game targets has zero patience for animation that performs rather than informs, and the entire animation pipeline is built to honour that.

**Two moments anchor the fantasy:**

**The placement-reveal flip** — the single most loaded 100 ms in the game. Both players' hidden cards turn face-up across all five lanes simultaneously, and that's the instant the auction read either pays off or doesn't. The flip must feel *committed*: cards land with weight, impact frames don't smear into ambiguity, all five lanes parse in a single glance. This is the auction's emotional payoff and the bluff's verdict, delivered in one frame.

**RESOLUTION as fluent reading** — three to five seconds where five lanes animate in parallel and the player stitches the round into a story as it plays out. A Cra slides forward, a Prism White flash punches the enemy, the damage number floats up in the same beat the HP bar drains, the enemy fades. Across all five lanes that information is arriving at once — death fades, advance slides, REPEL displacements — and the player already knows the round outcome before any log line could have told them. Good animation here means the player feels like a *fluent reader* of the board, not a viewer of it. They learn what REPEL does the first time they see a unit get shoved, no tooltip needed.

**Pillar service:** *No idle spectating* (animation IS the active reading — every frame is information), *Simple surface* (animation teaches state changes faster than rules text), *Auction as signature* (the placement-reveal flip is the auction's payoff moment, and the auction-panel transitions sell the gravity of bidding).

**The central failure mode — "Animation became a cutscene."** The worst thing this system can do is insert waiting time between the player's decision and the next one. Three ways this happens: animations that are too long (a 600 ms card flip when 100 ms suffices), animations that are sequential when they should be parallel (lanes resolving one-at-a-time instead of all five simultaneously), or animations that gate input (the player can't queue their next bid because the gold-tick is still playing). Any of these turns the game from "active reading" into "watching a slot machine," directly violates *No idle spectating*, and corrodes the hardcore audience's trust that the game respects their time.

**The rule that protects against this:** *If removing the animation would not change what the player knows, the animation is decoration and must be cut or shortened.*

## Detailed Design

### Core Rules

**Rule C-1 — The Decoration Test (the master rule).**

Every animation in this game must pass these five tests. Animations that fail any test must be cut or shortened.

| # | Test | Question to ask |
|---|---|---|
| **D1** | State change | After this animation, does the player know something they did not know before, or have confirmed a state change they need to act on? If no, cut. |
| **D2** | Removal | Remove the animation mentally. If the player's decision quality in the next phase is identical, the animation is decoration. |
| **D3** | Parallel delivery | If two animations deliver independent information, they must play simultaneously. Sequential delivery of parallel-state information is decoration. (Exception: stagger when ordering itself is the information.) |
| **D4** | Input gating | Is the player's next meaningful action blocked until this animation completes? If yes, the duration must be the minimum needed to make the state change legible (no stylistic hold). If no, the animation must never block input. |
| **D5** | Teach-by-showing exemption | Animations that teach a rule on first encounter (REPEL shove, COUNTERATTACK recoil, TRAP flip) pass **D2 only** even if experienced players know the rule, *provided* they stay within phase budget and do not gate input. **D1, D3, and D4 apply unconditionally** — no animation gates input or exceeds phase budget under this exemption. |

**The protective rule (from Player Fantasy):** *If removing the animation would not change what the player knows, the animation is decoration and must be cut or shortened.*

---

**Rule C-2 — Animation Budget by Phase.**

Per-phase ceilings on animation duration. The Decoration Test (C-1) is necessary but not sufficient — duration ceilings are also enforced.

| Phase | Polish budget (non-blocking) | Transition budget (state-boundary) | Forbidden |
|---|---|---|---|
| `LOBBY` | 350 ms | 350 ms | Any animation > 500 ms |
| `DRAFT_INITIAL` (45 s) | 150 ms (gold tick, hover, slot-fade) | 350 ms (panel slide-in) + 280 ms (card-draws, start after panel completes at t=350 ms — sequenced to maintain Rule C-14 ≤ 2 regions) | Any > 350 ms per window; sequential card acquisitions (all initial card draws fire simultaneously at t=350 ms) |
| `DRAFT_AUCTION` (20 s) | 150 ms (timer-bar ease 120–150 ms; gold tick) | 350 ms (panel slide-in, settlement transition) | Any > 350 ms; idle/looping animations |
| `DRAFT_SHOP` (30 s) | 150 ms (slot-fade, refresh gray-out) | 350 ms (refresh slide-in) | Any > 350 ms |
| `PLACEMENT` (10 s) | **250 ms HARD CAP** (drag-lift, snap-back, hover, cell-highlight) | 250 ms | Any > 250 ms; entry animation for hand or timer |
| `RESOLUTION` (3–5 s) | 600 ms per sub-step (`resolution_sub_step_duration_ms`) | 400 ms `pre_animation_pause_ms` + 150 ms `inter_step_pause_ms` | Per-lane stagger; AnimQueue blocking phase advance |
| `GAME_OVER` | 500 ms | 1.5 s (overlay) | Any animation continuing into GAME_OVER beyond completion of mandatory ObjectiveReveal |

**150 ms threshold rationale:** below the perceptual "system responded" boundary (Nielsen 0.1 s / 1.0 s / 10 s model). Above 150 ms the player registers "I am waiting."
**250 ms PLACEMENT cap rationale:** 2.5% of the entire phase budget. Above this, animation becomes a meaningful time sink at a moment where seconds matter psychologically.
**350 ms transition cap rationale:** the threshold above which a panel transition compresses the next phase's perceived time.

---

**Rule C-3 — Input-Gating.**

An animation gates input *if and only if* (a) the destination UI state does not exist yet at the start of the animation AND (b) the duration is the minimum needed to make the state change legible.

**Animations that legitimately gate input:**

| Animation | Gating reason | Duration |
|---|---|---|
| Auction panel slide-in (DRAFT_INITIAL/SHOP → DRAFT_AUCTION) | Bid UI does not exist yet | ≤ 350 ms |
| Settlement panel expansion (DRAFT_AUCTION → DRAFT_SHOP) | DRAFT_SHOP timer hasn't started | 350 ms |
| Placement-reveal flip | RESOLUTION has begun; no input exists during RESOLUTION | 80–100 ms |
| Phase-transition fade (LOBBY → DRAFT_INITIAL) | No game UI exists during the fade | per Board Rendering |

**Animations that must NEVER gate input:**

| Animation | Correct behavior |
|---|---|
| Gold counter tick on bid accepted | Bid buttons re-enable on `S2CAuctionBidAccepted` receipt; tick plays concurrently |
| Timer-bar ease-out on bid accepted | Buttons re-enable on message receipt, not on animation completion |
| Card-to-fan slide on `S2CCardAcquired` (280 ms) | Hand readable as soon as card lands; clicks on other slots accepted during slide |
| Purchased-slot gray-out in DRAFT_SHOP | Click on other slots accepted during fade |
| Sub-step animations during RESOLUTION | AnimQueue advances on `group_timer.finished()`, not on tween-completion callback |

---

**Rule C-4 — Parallelism.**

| When | Rule |
|---|---|
| Same server event triggers multiple animations | **MANDATORY simultaneous start.** One S2C message → one animation start time. (Placement reveal: all 5 lanes flip simultaneously. Sub-step movements: all advances start same frame.) |
| Multi-objective destruction same RESOLUTION | **Stagger by ascending lane, 80–120 ms cadence.** Ordering IS the information — simultaneous would merge into visual noise. |
| Idle ambient effects (AURA pulse) | Stagger permitted only if the effect carries no state information. AURA pulses signaling a buff activation must fire simultaneously with the triggering event. |

**Parallelism is not a preference; it is mechanically required for fairness.** Stagger on parallel events would mean some lanes reveal before others, creating a read advantage for whichever lane the eye lands on first.

---

**Rule C-5 — Feedback Latency.**

Maximum acceptable latency from input to animation start.

| Player action | Animation response | Max latency |
|---|---|---|
| Click Bid button | Buttons disable; pending state begins | 0 frames (same frame as click) |
| Receive `S2CAuctionBidAccepted` | Timer bar ease-out begins; gold counter tick begins | 0 frames |
| Receive `S2CAuctionBidRejected` | Buttons re-enable; toast appears | 0 frames |
| Drag-start on hand card | Drag sprite appears; fan-ghost shows | 0 frames (mouse-down) |
| Drop on valid board cell | Placement commit visual; ghost on board; fan-ghost dims | 0 frames |
| Drop on invalid target | Snap-back begins (220 ms) | 0 frames |
| Click card to purchase (DRAFT_INITIAL/SHOP) | Slot enters pending state (desaturation) | 0 frames (optimistic pending, reversed on rejection) |
| Receive `S2CCardAcquired` | Card-to-fan slide begins | 0 frames |
| `S2CPhaseChanged(PLACEMENT)` | Hand fan snap-on (no entry animation) | 0 frames |

**Critical distinction:** The client never waits for a network round-trip to show response animation. "Button pressed" feedback is instantaneous; "server confirmed" feedback drives subsequent updates.

**WASM note:** "0 frames" refers to 0 frames from the frame Bevy processes the input event — not from physical mouse-down. In WASM, browser input events arrive via the JS event loop with 16–50 ms of platform latency before Bevy sees them. This floor is not addressable at this layer; the 0-frame rule governs Bevy-side animation latency only.

**Optimistic pending rollback:** All optimistic pending states are reversible on server rejection. Bid button disable is reversed on `S2CAuctionBidRejected` (buttons re-enable, matching the pattern for `S2CAuctionBidAccepted`). Card purchase slot desaturation is reversed on acquisition rejection. Neither state is final until the server confirmation event arrives.

---

**Rule C-6 — Custom Lens Library.**

Card Animations ships five custom `Lens<T>` implementations that `bevy_tweening` 0.18 does not provide. Do not author lenses bevy_tweening already ships (`TransformPositionLens`, `TransformRotationLens`, `TransformScaleLens` — uniform scale).

| Lens | Target | Interpolates | Used by |
|---|---|---|---|
| `SpriteAlphaLens` | `Sprite` | `sprite.color.set_alpha(value)` (f32) | Fog lift, death fade, damage-number fade-out, NO BIDS desaturation, card-draw alpha-in |
| `BackgroundColorAlphaLens` | `BackgroundColor` (UI) | `background_color.0.set_alpha(value)` | "YOU WON" overlay fade, Prism White flash on auction panels, 60 ms flash burst on `S2CAuctionBidAccepted` |
| `SpriteColorLens` | `Sprite` | full `Color` (RGBA) | Timer-bar zone cross-fade (300 ms), AURA color cycling, NO BIDS color → grayscale |
| `TransformScaleXLens` | `Transform` | `transform.scale.x` only (Y/Z untouched) | Health-bar fill-width animation, placement-reveal squash flash |
| `TextColorLens` | `TextColor` | `TextColor(Color::...)` newtype | Gold counter tick color flash, damage-number color fade |

**Visual contracts (per art-director):**

- `SpriteAlphaLens`: easing `EaseOutQuad` for exits, `EaseInQuad` for entrances. Tween must complete cleanly — no mid-value sticking. If interrupted (unit dies mid-advance), alpha completes at 0 in next frame as hard cut.
- `BackgroundColorAlphaLens`: easing `EaseOutCubic`, 200–350 ms. Clamp [0.0, 1.0] enforced at lens level (no overshoot).
- `SpriteColorLens`: 300 ms default. Used for color-zone transitions; never animate to a color outside the registered palette.
- `TransformScaleXLens`: easing `EaseOutQuad`, 250–350 ms per damage event. Clamp ≥ 0.0. Health-bar color threshold swaps (green→yellow at 0.6, yellow→red at 0.3) are *instantaneous Transform writes on the same frame the tween starts* — color change and scale drain are read in the same beat.
- `TextColorLens`: easing `EaseOutCubic`, 500 ms for damage numbers.

---

**Rule C-7 — Animator Component Lifecycle.**

| Event | Behavior |
|---|---|
| Tween start | `Animator<T>` inserted by the system responding to the triggering event, in the same frame the tween begins. Never pre-inserted on entity spawn. |
| Tween completion | **Animator is NOT removed.** Component remains in `TweenCompleted` state on the entity. Removing on completion costs an archetype move per cycle; the next animation replaces it via `set_tweenable`. Animator is removed only when the entity itself is despawned (e.g., damage number entity at end-of-life). |
| Tween cancel-replace | `animator.set_tweenable(new_tween)` (Bevy 0.18 API — see OQ-CA-01). NEVER despawn-and-respawn entities (loses game-state components — `BoardCell`, `UnitStats`, `LeaderTag`). NEVER write `Transform.translation` directly while an animator is active (board-rendering BR-16 invariant). |
| Entity despawn during active tween | Animator and tween are released atomically with entity despawn. No follow-up cleanup needed — Bevy's archetype removal handles it. |
| Concurrent animators on same entity | `Animator<Transform>` and `Animator<Sprite>` may coexist on one entity (target different components, advanced independently). |
| Same-component parallel animation | If one entity needs two `Transform` animations simultaneously (REPEL'd unit also advancing), use `Tracks<Transform>` (parallel-track container) — NOT two `Animator<Transform>` components, which would conflict. |

---

**Rule C-8 — `AnimGroup` / `AnimQueue` (RESOLUTION) — Card Animations side of the contract.**

The `AnimGroup` and `AnimQueue` data shape is owned by `board-rendering.md` (Rule 5; see lines 36–59 of that file). Card Animations owns the lens implementations and the system that drains the queue. The contract:

- **Within a group:** all `ResolutionEvent`s in `AnimGroup.events` spawn their tweens in a single frame. No ordering within a group. The group's `duration_ms` is the wall-time budget; tweens may complete earlier but the group timer still runs to full before the inter-step pause starts.
- **Sequential execution:** the `ResolutionExecuting` system checks `AnimQueue.group_timer.finished()` each frame (ticked by `Time<Virtual>`). When finished: start `inter_step_timer`. When that finishes: advance `current_index`, spawn next group's tweens, reset `group_timer`.
- **Sub-step grouping invariant:** the queue partitions by `sub_step` field. All events sharing a `sub_step` are in the same `AnimGroup`. This guarantees the combat-resolution display contract: "sub-step 3 effects must complete visually before sub-step 5 movement begins."
- **Queue-during-queue (defensive):** RSM guarantees this cannot happen (one `S2CResolutionEvent` per round). If a second arrives mid-queue: log error, discard in-flight queue, load new event as fresh queue from `groups[0]`. Do not silently merge or append. **This discard MUST use the same cancellation code path as `BoardRebuildRequested`** — cancel all active `Animator<T>` components, then reset `AnimQueue`.
- **Queue lifecycle:** When RESOLUTION completes (all groups drained and the post-queue `PendingPhaseChange` is emitted), `AnimQueue.groups` is cleared and `current_index` reset to 0. On `BoardRebuildRequested` (reconnect), `AnimQueue` is reset to `Default::default()` immediately — stale groups from the previous RESOLUTION must not persist across a reconnect.

---

**Rule C-9 — Plugin Architecture.**

One `CardAnimationsPlugin`. Single plugin, internal modules (`lenses/`, `animators/`, `queue/`, `events/`). Per-domain plugins would split the lens library across files for no gain and create plugin-ordering dependency problems.

`CardAnimationsPlugin::build()` responsibilities:
1. Add `bevy_tweening::TweeningPlugin` if not already registered.
2. Register the 5 custom lens types.
3. `app.add_message::<T>()` for each domain `Message` type Card Animations consumes (see Rule C-10). Note: `add_message` is idempotent in Bevy 0.18 — registration by multiple plugins does not panic. (**Bevy 0.18 API:** `add_event`/`EventReader` were removed in 0.17. Use `#[derive(Message)]` + `app.add_message::<T>()` + `MessageWriter<T>`/`MessageReader<T>` for all intra-client domain events.)
4. Add the systems that consume domain events and spawn tweens.
5. Add the `ResolutionExecuting` queue-drain system.

---

**Rule C-10 — Cross-System Event Flow (domain-event indirection).**

Card Animations does **NOT** subscribe directly to `S2C*` Lightyear messages. The upstream systems consume those messages, update authoritative state, and emit narrow domain events. Card Animations subscribes only to those domain events.

```
┌────────────────────────────────────────────────────────────────────┐
│ Lightyear S2C messages (S2CPlacementReveal, S2CResolutionEvent,   │
│ S2CCardAcquired, S2CAuctionBidAccepted, S2CAuctionSettled, ...)   │
└──────────────────────────┬─────────────────────────────────────────┘
                           │ consumed by upstream systems
                           ▼
┌────────────────────────────────────────────────────────────────────┐
│ Upstream systems update authoritative state, then emit:           │
│   Board Rendering → PlacementRevealAnimReady, ResolutionGroupReady,│
│                     ObjectiveDestroyedAnimReady, FogLiftReady      │
│   Hand UI         → CardAcquiredAnimReady, SnapBackRequested,      │
│                     HandHideRequested                              │
│   Shop/Auction UI → AuctionPanelTransitionRequested,               │
│                     TimerBarEaseRequested, GoldTickRequested,      │
│                     SettlementOverlayRequested                     │
│   Keyword System  → DisplacementAnimRequested (REPEL/ATTRACT),     │
│                     TrapFlipRequested                              │
└──────────────────────────┬─────────────────────────────────────────┘
                           │ Bevy 0.18 Messages (#[derive(Message)], intra-client)
                           ▼
┌────────────────────────────────────────────────────────────────────┐
│ Card Animations: spawn tweens via custom lens library             │
│ (no Lightyear dependency, no game-state mutation)                 │
└────────────────────────────────────────────────────────────────────┘
```

**Domain events Card Animations consumes** (intra-client `#[derive(Message)]` types — full schemas TBD per upstream GDD):

- From Board Rendering: `PlacementRevealAnimReady`, `ResolutionGroupReady`, `ObjectiveDestroyedAnimReady`, `FogLiftReady`, `BoardRebuildRequested` (cancel all animators), **`PlacementCancelAllAnimsRequested`** (cancel all PLACEMENT animators on `S2CPhaseChanged(RESOLUTION)` — normal round path, not reconnect; see CA-21 and Edge Cases §2)
- From Hand UI: `CardAcquiredAnimReady`, `SnapBackRequested`, `HandHideRequested`, `HandShowRequested`
- From Shop/Auction UI: `AuctionPanelTransitionRequested`, `TimerBarEaseRequested`, `TimerColorZoneRequested`, `GoldTickRequested`, `SettlementOverlayRequested`, `NoBidsTransitionRequested`
- From Keyword System: `DisplacementAnimRequested`, `TrapFlipRequested`, `AuraPulseRequested`

**Why indirection beats direct S2C subscription:**

- Ordering invariant ("game state updated *before* animation starts") is encoded in message emission, not in `before()`/`after()` system constraints.
- Card Animations has zero Lightyear dependency — pure presentation layer.
- Adding a new animation trigger = one new message type + one `MessageWriter` call upstream.

**SystemSet ordering requirement:** CA-21 requires same-frame delivery of `PlacementCancelAllAnimsRequested`. `CardAnimationsPlugin::build()` must declare `CardAnimationsSet::React.after(BoardRenderSet::ScheduleTweens)` to guarantee the cancellation system runs in the same `Update` schedule tick that Board Rendering emits the event.

**Cost:** ~16 narrow domain message types. Naming convention: `[Action][Subject]Requested` for triggers; `[Action][Subject]AnimReady` when the upstream wants to signal "state is updated, animation may begin." All types use `#[derive(Message)]` + `app.add_message::<T>()` registered by the owning upstream plugin in its `build()`. Card Animations consumes via `MessageReader<T>` — no re-registration needed.

---

**Rule C-11 — Easing Catalog.**

| Animation category | Curve | Duration range | Reasoning |
|---|---|---|---|
| Card slide to hand (draw, fan-in) | `EaseOutQuint` | 280 ms (`card_draw_animation_ms`) | Fast arrival, lingering settle. Card "lands" with authority. |
| Card snap-back (drag released) | `EaseOutBack` (overshoot ~1.1) | 220 ms (`snap_back_duration_ms`) | Mild overshoot communicates rejection. The hand has a home. Sole non-UI-confirmation use of overshoot. |
| Panel slide in/out (shop, auction) | `EaseOutCubic` | 350 ms | Softer than Quad. Panels are environment, not events. |
| Timer bar drain | `Linear` | continuous | Drain is a clock. Easing would lie about pace of time pressure. |
| Timer bar ease-out (bid accepted) | `EaseOutQuad` | 120–150 ms | Brief snap from current to new fill, then resume linear drain. |
| Timer color zone cross-fade | `Linear` (color interpolation) | 300 ms | Smooth perceptual color blend. |
| Gold counter tick (per increment) | `EaseOutQuad` | ≤ 150 ms total tick run | Each tick is a micro-event — brief pop registers as received-information without wait. |
| Hover scale (card in hand) | `EaseOutQuad` | 60–80 ms | Sub-100 ms expand. Must not compete with player reading the card. Max scale 1.12× (above this occludes adjacent fan cards). |
| Fade in (fog lift, unit spawn) | `EaseInQuad` | `fog_lift_duration_ms` (board-rendering) | Slow start — information is arriving, not slamming in. |
| Fade out (death, phase dismiss) | `EaseOutQuad` | 200–250 ms (death) / 200 ms (NO BIDS desat) / 400 ms (NO BIDS card) | Exits faster than entrances. Resolved information leaves immediately. |
| Unit advance (combat translate) | `EaseOutQuad` | `resolution_sub_step_duration_ms` (600 ms) | Locked. Fast charge, reads as purposeful. |
| Damage number float | `EaseOutCubic` | 500 ms (+60 px float, alpha fade) | Natural arc, decelerates as it fades. Physical without physics. |
| REPEL displacement | `EaseOutQuint` | per Keyword System spec | Steep curve: fast launch, sudden stop. Arrested motion communicates resistance. |
| ATTRACT displacement | `EaseInOutQuad` | per Keyword System spec | Symmetrical in/out — gravitational, not explosive. |
| UI confirmation overshoot | `EaseOutBack` | 120–150 ms | Reserved for UI confirmations only (snap-back is the principal example). Forbidden in combat. |
| Objective destruction overlay | step (3-frame: 80% → 60% → 30% Prism White over 240 ms) | 240 ms total | Frame-stepped, not interpolated. Locked by Combat Resolution. |

**Forbidden:** any combat translate using spring/elastic curves. `EaseOutQuad` is the combat baseline (locked by Combat Resolution Visual/Audio section).

---

**Rule C-12 — Anticipation Policy (Restrained Hearthstone).**

Anticipation (slight backwards before forward, squash before stretch) is **restricted** in this system. The pillar "No idle spectating" cannot absorb 50–80 ms of pre-motion on every combat step on 5 parallel lanes.

**Permitted anticipation moments (signature only):**

1. **Placement-reveal flip** — the 3-frame squash (back-of-card silhouette → Prism White edge-on squash flash → front-face sprite, 80–100 ms total) is anticipation. Sells the weight of the reveal. Already specified by Combat Resolution.
2. **Objective destruction** — a 1-frame scale-down to 95% before the 3-frame Prism White overlay. Adds weight to the hit without extending duration.

**Forbidden anticipation:**

- Unit advance (no wind-up frame — the smear IS the departure read)
- Card draw (no pre-lift pause — the hand fills, it does not prepare to fill)
- Panel transitions (no pull-back before slide — panels are navigation, not characters)
- Damage number floats (linear-arc EaseOutCubic; no anticipation jiggle)

**Follow-through:** permitted only as inherent overshoot in the specified `EaseOutBack` snap-back. No secondary motion on unit arrival after advance — they land, they stop.

---

**Rule C-13 — Restraint Rules (what we deliberately do NOT animate).**

- **No idle animation on units at rest.** No breathing cycles, no hovering bob, no ambient particle. Units that are idle declare: "I am waiting for your decision." Motion would lie.
- **No passive HUD animation.** Gold total, mana indicator, hand count do not pulse, glow, or shift unless their value just changed. Passive animation on resource counters creates false urgency.
- **Cards in hand do not bob.** Fan is static at rest. Only the hovered card scales. **De-hover spec:** when the cursor leaves a card, scale returns to 1.0× via `EaseOutQuad` at the same duration as hover-in (60–80 ms). If a new hover-in starts on another card before de-hover completes, the de-hover on the previous card is cancelled via `set_tweenable` — the replacement tween is a new `EaseOutQuad` tween from the card's *current intermediate scale* back to 1.0× over the remaining de-hover duration (≤ 60–80 ms). The previous card does not snap instantly — it completes the return to 1.0× via this replacement tween. At most one card may be in hover-scaled or hover-scaling state at any time — entering a new hover triggers cancel-replace on any other card's active hover animator.
- **No phase-transition fanfare on routine rounds.** Phase banner slides in and out once. No pulse, no lingering, no particles.
- **During PLACEMENT, the board is frozen.** No unit shuffle, no lane-indicator pulse, no "ready" state animation. The board is a static canvas the player is reading.
- **No glow/bloom on unit sprites.** Style guide forbids it. Impact flashes are flat 1-frame color fills (Prism White / warm orange). The flash is the signal — glow adds nothing and breaks the cel-shaded contract.

---

**Rule C-14 — Failure-Mode Catalog (what to prevent in code review).**

| Anti-pattern | Trigger to watch for | Severity |
|---|---|---|
| Sequential per-lane RESOLUTION | `for lane in lanes { play_animation(lane); wait(); }` | BLOCKER — pillar violation |
| Motion soup on phase change | More than 2 UI regions animating same phase change | BLOCKER — readability violation |
| Glow/bloom on units | Shader with bloom enabled on `Sprite` | BLOCKER — style violation |
| Death animation longer than unit advance | Death `SpriteAlphaLens` duration > 250 ms | CONCERN |
| Damage number obscuring health bar | Float origin = bar position | CONCERN |
| Animation gating phase transition | `S2CPhaseChanged` waits on `tween.finished()` | BLOCKER — locked rule |
| Animator removed on tween complete | `commands.entity(e).remove::<Animator<T>>()` after tween | CONCERN — archetype churn |
| Direct S2C subscription in Card Animations | `MessageReader<S2C*>` in `card_animations` module | BLOCKER — boundary violation |

### States and Transitions

**Card Animations does not own its own state machine.** Each domain owns its FSM (`BoardRenderState` in board-rendering, panel state machine in shop-auction-ui, hand visibility in hand-ui). Card Animations is event-driven.

**The only state Card Animations manages internally is per-tween:**

| Tween state | Description | Transition |
|---|---|---|
| Spawned | `Animator<T>` inserted on entity, tween ticking | → `Active` |
| Active | `Time<Virtual>` advancing the tween | → `Completed` (on duration elapsed) or → `Replaced` (on `set_tweenable`) |
| Completed | Tween finished; `Animator` remains on entity for reuse | → `Replaced` (next animation) or terminal |
| Replaced | Previous tween cancelled; new tween installed via `set_tweenable` | → `Active` |
| Released | Entity despawned; animator released atomically | terminal |

**The `AnimQueue` (for RESOLUTION only) has its own internal stepping** — owned by `BoardRenderState` (board-rendering Rule 5), driven by `Time<Virtual>` group/inter-step timers. Card Animations supplies the system that drains it.

### Interactions with Other Systems

| System | Direction | Contract |
|---|---|---|
| **Board Rendering** | Upstream (events in) → Card Animations | Emits `PlacementRevealAnimReady`, `ResolutionGroupReady`, `ObjectiveDestroyedAnimReady`, `FogLiftReady`, `BoardRebuildRequested`, **`PlacementCancelAllAnimsRequested`** after authoritative state updates. Card Animations consumes these and spawns tweens. **Critical:** `BoardRebuildRequested` (on `S2CGameSnapshot`) cancels all in-flight tweens, clears all per-entity animator state, and resets `AnimQueue` to `Default` — no replay on reconnect. `PlacementCancelAllAnimsRequested` is emitted by Board Rendering when it processes `S2CPhaseChanged(RESOLUTION)` on the normal round path; Card Animations cancels PLACEMENT-phase animators and writes entity `Transform` to `BoardCell.world_position` in the same frame. |
| **Hand UI** | Upstream → Card Animations | Emits `CardAcquiredAnimReady` (card-to-fan slide, 280 ms), `SnapBackRequested` (220 ms `EaseOutBack`), `HandHideRequested` (instant `Visibility::Hidden`, no exit animation), `HandShowRequested` (instant snap-on, no entry animation). **DRAFT_INITIAL entry sequencing (Rule C-14 compliance):** Hand UI delays emitting `CardAcquiredAnimReady` for initial hand cards by `anim_panel_slide_duration_ms` (350 ms) after `S2CPhaseChanged(DRAFT_INITIAL)`, so card-draw animations start after the auction panel slide completes. This keeps simultaneous animated regions ≤ 2 at phase entry (panel slide only; `HandShowRequested` is instant `Visibility` change, not a tween). `S2CCardAcquired` events arriving mid-DRAFT_INITIAL (not at phase entry) fire immediately without delay. |
| **Shop/Auction UI** | Upstream → Card Animations | Emits `AuctionPanelTransitionRequested` (350 ms slide), `TimerBarEaseRequested` (120–150 ms ease-out + 60 ms flash), `TimerColorZoneRequested` (300 ms cross-fade), `GoldTickRequested` (≤ 150 ms tick), `SettlementOverlayRequested` (1.5 s overlay — **Decoration Test justification:** the settlement overlay is an affective signal, not a pure information transition; WON/LOST is the auction's payoff moment and the dwell time is intentional — the player's emotional state at this moment benefits from a deliberate beat before the next phase begins. This satisfies D4 as an affective minimum, not an informational minimum. OQ-CA-07 resolved.), `NoBidsTransitionRequested` (200 ms desat + 400 ms fade — see OQ-CA-08). |
| **Combat Resolution** | Display contract (no direct messages) | Defines RESOLUTION animation budget: pre-pause 400 ms, sub-step 600 ms, inter-step 150 ms. Specifies smear-frame + impact-flash + recover (200–250 ms attack), placement-reveal flip (80–100 ms × all units), Prism White / warm orange flash colors. Card Animations honours these constants from `GameConfig`. |
| **Keyword System** | Upstream → Card Animations | Emits `DisplacementAnimRequested` (REPEL `EaseOutQuint` / ATTRACT `EaseInOutQuad`, lane-axis only), `TrapFlipRequested` (Y-axis card flip), `AuraPulseRequested` (cosmetic stagger permitted). |
| **Game Config** | Config → Card Animations | Provides timing constants: `pre_animation_pause_ms`, `resolution_sub_step_duration_ms`, `inter_step_pause_ms`, `fog_lift_duration_ms`, `card_draw_animation_ms`, `snap_back_duration_ms`. Loaded once at startup. |
| **Round State Machine** | Phase events → Card Animations | `S2CPhaseChanged` is consumed by upstream systems first; Card Animations sees only the resulting domain events. Emergency override: `S2CPhaseChanged(GAME_OVER)` mid-RESOLUTION causes Board Rendering to emit `BoardRebuildRequested` after current AnimGroup completes (per board-rendering Rule 10). |
| **Network Protocol** | NONE | Card Animations does not subscribe to any Lightyear `S2C*` message directly. All triggers come via domain events from upstream systems. |
| **Server-side RNG** | NONE | No client-side randomness in animations. Any visual variation must be deterministic from event payload (e.g., damage-number jitter offset = `event_id % jitter_table_len`). |

## Formulas

Card Animations is a presentation system — it introduces no gameplay balance formulas. Two formulas are owned here; remaining timing calculations are owned by upstream GDDs and referenced only.

---

**F1 — Multi-objective reveal stagger**

The `reveal_stagger` formula is defined as:

`reveal_start_ms[i] = i × stagger_cadence_ms`

**Variables:**

| Variable | Symbol | Type | Range | Description |
|---|---|---|---|---|
| Lane sort index | `i` | u8 | 0–3 | Zero-indexed position in the ascending-lane-sorted `ObjectiveDestroyed` list. `i=0` = lowest lane number with a destruction event this RESOLUTION. Maximum simultaneous destructions = 4 (both players lose 2 objectives each), so `i` reaches 3 at most. |
| Stagger cadence | `stagger_cadence_ms` | u32 | ≥ 80 (range 80–120, default 100) | ms between sequential objective reveal animations. Loaded from `GameConfig`. Values below 80 ms are inadvisable (perceptually merges into simultaneous burst); enforced via GameConfig minimum. |

**Output range:** 0 ms (`i=0`) to 3 × 120 ms = 360 ms (`i=3` at max cadence).

**Example:** Two objectives destroyed in lanes 3 and 5 → sorted: lane 3 (`i=0`), lane 5 (`i=1`). At default 100 ms cadence: lane 3 reveals at 0 ms, lane 5 at 100 ms.

**Invariant:** Board Rendering sorts `ObjectiveDestroyed` events ascending by lane number before emitting `ObjectiveDestroyedAnimReady` events (Board Rendering Rule 12 — authoritative sorter). Events arrive at Card Animations pre-sorted. Card Animations does not re-sort. *(If Board Rendering's sort is ever removed, add a defensive sort here.)*

**Execution model — concurrent-with-stagger:** Each reveal animation starts at its `reveal_start_ms[i]` offset but runs concurrently with all subsequent reveals. The total wall-clock window for N simultaneous objective destructions is `(N-1) × stagger_cadence_ms + single_reveal_duration` — not `N × single_reveal_duration`. Example: 4 destructions at default 100 ms cadence = 3 × 100 + 240 ms overlay = 540 ms total. **Note for board-rendering.md:** F4's `N_destroyed × (objective_reveal_hold_ms + objective_reveal_anim_ms)` term uses the sequential model. F4 must be updated to the concurrent formula: `(N_destroyed - 1) × stagger_cadence_ms + single_obj_duration`. The F1 stagger window runs in `ResolutionObjectiveReveal` after the `AnimQueue` drains — `AnimQueue.total_duration_ms()` does not include it.

---

**F2 — Damage number despawn delay**

The `damage_number_despawn` formula is defined as:

`despawn_delay_ms = max(float_tween_duration_ms, fade_tween_duration_ms)`

**Variables:**

| Variable | Symbol | Type | Range | Description |
|---|---|---|---|---|
| Float tween duration | `float_tween_duration_ms` | u32 | 400–600 | Duration of the `Tween<Transform>` (+60 px upward float). Default 500 ms. |
| Fade tween duration | `fade_tween_duration_ms` | u32 | 400–600 | Duration of the `Tween<TextColor>` alpha fade-out. Default 500 ms. |

**Output range:** 500 ms when both durations are equal (default). Upper bound is the larger of the two if tuned asymmetrically.

**Example:** `float_tween_duration_ms=500`, `fade_tween_duration_ms=500` → `despawn_delay_ms=500`. Entity despawned 500 ms after spawn.

**Why this formula is explicit:** Both tween components (`Animator<Transform>` float and `TextColorLens` fade) run concurrently on the same entity. The entity must not be despawned until both complete — premature despawn causes mid-air blink-out; missed despawn causes an entity leak. Using `max()` accounts for any asymmetric tuning without code changes.

---

**Cross-references (not redefined here):**

- **Health bar fill fraction** → defined in `board-rendering.md § F2 (health_bar_fill)`. `TransformScaleXLens` reads this value as the tween target; Card Animations does not own the formula.
- **Resolution animation total duration** → defined in `board-rendering.md § F4`. `AnimQueue.total_duration_ms()` exposes this as a read-only computed property; Card Animations does not redefine it.

## Edge Cases

- **If `animator.set_tweenable(new_tween)` is called on an entity whose `Animator<T>` is in `TweenCompleted` state (no in-flight tween):** the replace proceeds identically — `set_tweenable` is unconditional on animator state. This is the normal path for any entity playing its second or later animation; no guard clause needed.

- **If a PLACEMENT animation (snap-back, drag-lift, cell-highlight) is mid-tween when `S2CPhaseChanged(RESOLUTION)` arrives on the normal game path (not reconnect):** Board Rendering processes `S2CPhaseChanged(RESOLUTION)` first, updates `BoardRenderState`, then emits `PlacementCancelAllAnimsRequested` (see Rule C-10). Card Animations' cancellation system consumes this event in the same `App::update()` tick (same-frame delivery guaranteed by `CardAnimationsSet::React.after(BoardRenderSet::ScheduleTweens)` — see Rule C-10 SystemSet ordering note). All in-flight PLACEMENT animators are cancelled and each entity's `Transform.translation` is overwritten with the value from its `BoardCell` component (the authoritative board-position component owned by Board Rendering — field name and layout specified in board-rendering.md). The 250 ms hard cap (Rule C-2) bounds the maximum mid-tween duration discarded (at most 250 ms of animation is lost). This is the common-case path — every round transitions PLACEMENT → RESOLUTION this way. See CA-21.

- **If a PLACEMENT animation (snap-back, drag-lift, cell-highlight) is mid-tween when `S2CPhaseChanged(RESOLUTION)` arrives and `BoardRebuildRequested` also fires (reconnect path):** all in-flight animators are cancelled and cleared atomically by the board rebuild. The entity's `Transform` is overwritten in the same frame. The 250 ms PLACEMENT hard cap (Rule C-2) makes this race window narrow, but any mid-tween state is discarded — no partial-tween visual artifact persists.

- **If `S2CGameSnapshot` (→ `BoardRebuildRequested`) arrives mid-RESOLUTION while an `AnimGroup` is executing:** `AnimQueue` is reset to `Default::default()`, all active `Animator<Transform>` and `Animator<Sprite>` components are cancelled, and any in-flight damage-number entities are despawned as part of the full board rebuild. No stale floating numbers survive the reconnect. The reconnecting client enters the post-snapshot phase directly with no animation replay (board-rendering Rule 11). **Damage-number entity cleanup:** damage number entities carry a `#[derive(Component)] struct DamageNumber;` marker component inserted at spawn time. Board Rendering Rule 11's despawn pass includes a `Query<Entity, With<DamageNumber>>` despawn sweep to find and clean up these ephemeral entities. **Stagger timer cleanup:** the multi-objective stagger sequence is managed via a `StagedObjectiveRevealQueue` resource (a `VecDeque<(lane_index, Timer)>`) owned by Card Animations. On `BoardRebuildRequested`, Card Animations clears this resource — all pending stagger timers are discarded.

- **If `S2CPhaseChanged(GAME_OVER)` arrives during an `AnimGroup` execution mid-RESOLUTION:** the current `AnimGroup` completes in full (board-rendering Rule 10 — no mid-tween interrupt), remaining groups are skipped, then `ResolutionObjectiveReveal` runs for any buffered `ObjectiveDestroyed` events, then the system transitions to `GameOver`. Card Animations continues ticking the current group's tweens normally — the decision to complete or skip the queue belongs to the Board Rendering state machine, not to Card Animations. **Known design compromise:** at worst-case GAME_OVER arrival (t=1 ms into a 600 ms AnimGroup), the player waits up to 599 ms of animation after the game is already decided. **Note:** this wait will most commonly land on the game-deciding AnimGroup — the one containing the final ObjectiveDestruction event — since GAME_OVER arrives precisely when the last objective is destroyed. This is not a rare edge case; it is the modal case for winning and losing rounds. This compromise is accepted for the friend-game scope because completing the current group is simpler than a mid-group interrupt. A lower-complexity mitigation (advance `group_timer.elapsed` to `group_timer.duration` on GAME_OVER receipt, allowing the group to drain in the same frame without mid-tween interrupt) is viable but out of scope for M2/M3. If post-friend-game playtests surface this as a felt pain point, implement the timer-advance mitigation before the mid-group interrupt path.

- **If a unit dies during its own advance `Tween<Transform>` in the same `AnimGroup` (simultaneous death and movement):** both events are in the same group so both tweens start in the same frame. The advance `Animator<Transform>` and the death `Animator<Sprite>` (`SpriteAlphaLens` fade) run concurrently on the same entity — separate components, independent advancement. Neither cancels the other. If this is semantically wrong for a specific combat sequence (dead unit should not complete its advance), that is a Combat Resolution sequencing decision; Card Animations animates whatever events it receives in a group.

- **If the same unit entity receives multiple `DamageNumberSpawnRequested` events in the same `AnimGroup` (unit takes damage from two simultaneous sources in one sub-step):** each event spawns a distinct damage-number entity. No entity reuse. The second entity's origin position is offset by `event_id % jitter_table_len` (deterministic jitter from event payload — no client-side RNG) to prevent numbers stacking at identical world positions. Each entity has its own independent `Animator<Transform>` and `TextColorLens` with its own `despawn_delay_ms` computed via F2.

- **If F2 is applied with asymmetric tuning (`float_tween_duration_ms ≠ fade_tween_duration_ms`, e.g., 400 ms float, 600 ms fade):** `despawn_delay_ms = max(400, 600) = 600`. The float tween finishes 200 ms before the entity despawns — the number coasts at its final position while alpha continues draining to zero. The entity must NOT be despawned early on the float tween's completion. The despawn timer is set at spawn time from F2 and runs independently of any tween-completion event.

- **If `stagger_cadence_ms = 0` (degenerate config):** F1 produces `reveal_start_ms[i] = 0` for all lanes — all objective destruction animations start simultaneously. Not undefined behavior; degenerates to a zero-stagger burst. Below 80 ms, sequential reveals are perceptually indistinguishable from simultaneous, defeating the stagger. Minimum safe value is 80 ms (see Tuning Knobs).

- **If the `ObjectiveDestroyed` event list at F1 has length 0 (no objectives destroyed this RESOLUTION):** formula is not evaluated. `ResolutionObjectiveReveal` finds an empty sorted list, completes immediately, and drains `PendingPhaseChange`. No stagger timer is started. Normal case in most rounds.

- **If four `ObjectiveDestroyed` events arrive simultaneously (both players each lose two objectives in one RESOLUTION):** `i` reaches 3 at most (0-indexed, `fake_count = 2` per player). At default 100 ms cadence: last reveal starts at 300 ms. All four reveals complete within 300 ms + single-reveal duration. Within budget.

- **If `BoardRebuildRequested` fires mid-way through a multi-objective stagger sequence (between reveals `i=0` and `i=1`):** pending stagger timers are discarded with the rebuild. The partial stagger is abandoned; the reconnecting client enters the post-snapshot phase immediately and sees the final objective state without animation. Consistent with the general reconnect contract: animation is sacrificed for deterministic recovery.

- **If `PlacementRevealAnimReady` targets a unit entity that also carries an active `Animator<Transform>` from a concurrent `DisplacementAnimRequested` in the same `AnimGroup`:** the placement-reveal flip uses `SpriteColorLens` + `TransformScaleXLens` — not a position `Tween<Transform>`. There is no component conflict. If both translation and scale transforms must animate simultaneously on one entity, use `Tracks<Transform>` (per Rule C-7 — same-component parallel animation).

## Dependencies

**Hard upstream dependencies** (system cannot function without these):

| System | File | Nature of dependency |
|---|---|---|
| Board Rendering | `gdd/board-rendering.md` | Emits `PlacementRevealAnimReady`, `ResolutionGroupReady`, `ObjectiveDestroyedAnimReady`, `FogLiftReady`, `BoardRebuildRequested`. Card Animations drives all RESOLUTION tweens from these events. Owns `AnimGroup`/`AnimQueue` data schema. |
| Hand UI | `gdd/hand-ui.md` | Emits `CardAcquiredAnimReady`, `SnapBackRequested`, `HandHideRequested`, `HandShowRequested`. Owns card-draw and snap-back timing constants. |
| Shop / Auction UI | `gdd/shop-auction-ui.md` | Emits `AuctionPanelTransitionRequested`, `TimerBarEaseRequested`, `TimerColorZoneRequested`, `GoldTickRequested`, `SettlementOverlayRequested`, `NoBidsTransitionRequested`. Owns timer-bar and panel animation specs. |
| Game Config | `gdd/game-config.md` | Provides all timing constants consumed at startup: `pre_animation_pause_ms`, `resolution_sub_step_duration_ms`, `inter_step_pause_ms`, `fog_lift_duration_ms`, `card_draw_animation_ms`, `snap_back_duration_ms`, `stagger_cadence_ms` (new — to be added to game-config.md). |
| bevy_tweening | `Cargo.toml` (version 0.18-compatible) | Core tween library. Provides `Animator<T>`, `TweeningPlugin`, `Tween`, `Sequence`, `Tracks`. Custom lenses extend it via the `Lens<T>` trait. |

**Soft dependencies** (enhanced by but functions without):

| System | File | Nature of dependency |
|---|---|---|
| Keyword System | `gdd/keyword-system.md` | Emits `DisplacementAnimRequested` (REPEL/ATTRACT), `TrapFlipRequested`, `AuraPulseRequested`. M3 scope — Board Rendering M2 placeholder tweens cover RESOLUTION until Keyword System is implemented. |
| Combat Resolution | `gdd/combat-resolution.md` | Display contract only (no direct messages). Defines RESOLUTION animation budget and visual constants (Prism White, warm orange, 80–100 ms flip, 200–250 ms attack). Honoured by Card Animations from `GameConfig`. |
| HUD | `gdd/hud.md` | Downstream. HUD may consume animation-driving events for gold/mana tick updates. HUD GDD (Designed — `hud.md`) should list Card Animations as a provider of gold counter tick animation. No reverse dependency. |

**Downstream dependents of Card Animations:** None — terminal node in the dependency graph. All other systems fire events that Card Animations consumes; no system depends on Card Animations emitting anything.

## Tuning Knobs

| Knob | Config key | Default | Safe range | Too high | Too low |
|---|---|---|---|---|---|
| Card draw animation | `card_draw_animation_ms` | 280 ms | 150–400 ms | Competes with player's DRAFT_INITIAL decisions (sluggish) | Motion too abrupt — card appears without travel |
| Snap-back duration | `snap_back_duration_ms` | 220 ms | 100–250 ms | Consumes meaningful PLACEMENT timer seconds; 250 ms PLACEMENT hard cap is the ceiling — values above 250 ms violate Rule C-2 and will be clamped by CA-12 enforcement | Rejection feedback too abrupt |
| Objective reveal stagger cadence | `stagger_cadence_ms` | 100 ms | 80–120 ms | Reveals feel slow; total reveal window inflates | Below 80 ms: perceptually merges into simultaneous burst, stagger loses purpose |
| Damage number float duration | `float_tween_duration_ms` | 500 ms | 400–550 ms *(see joint constraint below)* | Numbers linger, cluttering board during subsequent sub-steps | Numbers exit before player reads damage |
| Damage number fade duration | `fade_tween_duration_ms` | 500 ms | 400–550 ms *(see joint constraint below)* | Visible numbers stack; combined despawn_delay_ms must satisfy ≤ `resolution_sub_step_duration_ms` | Abrupt flicker disappearance |

**Interaction constraint:** `max(float_tween_duration_ms, fade_tween_duration_ms)` must be **strictly less than** `resolution_sub_step_duration_ms` by at least 50 ms margin. Equal values create a one-frame race at the sub-step boundary; at-or-above values leave damage numbers visible when the next sub-step begins, creating overlap noise. At defaults (500 ms float/fade vs 600 ms sub-step), the **50 ms** margin is satisfied.

**Joint constraint — safe range correction:** The table above shows "400–550 ms" for float/fade safe range at the default `resolution_sub_step_duration_ms=600 ms`. The effective upper bound is `resolution_sub_step_duration_ms - 50 ms`, not a fixed 600 ms. If sub-step is reduced, the float/fade ceiling falls with it.

**Sub-step floor:** `resolution_sub_step_duration_ms` must be ≥ 450 ms to support any float/fade value within the stated minimum of 400 ms (constraint: 400 + 50 = 450 ≤ sub_step). At the stated range minimum of 400 ms for resolution_sub_step, no valid float/fade value exists — the startup assert will catch this. Avoid reducing resolution_sub_step below 550 ms without also reducing float/fade defaults.

**Assert ordering:** the startup assert `max(float_tween_duration_ms, fade_tween_duration_ms) + 50 <= resolution_sub_step_duration_ms` must fire as a **post-deserialization validation** on the complete `GameConfig` struct (not inline during individual field loading — inline checks may compare against not-yet-loaded fields and silently pass incorrect values). Panic with a clear message if violated.

**Cross-referenced knobs** (owned by upstream GDDs — do not redefine here):

- `pre_animation_pause_ms` → `board-rendering.md` (default 400 ms, range 200–800 ms). **Decoration Test ownership note:** the cognitive justification for this pause — what the player is doing during the 400 ms between placement-reveal and first RESOLUTION sub-step — is owned by board-rendering.md. Card Animations references this constant in its budget math but does not justify it against the Decoration Test. board-rendering.md must document the cognitive purpose before this value can be considered Decoration-Test-compliant.
- `resolution_sub_step_duration_ms` → `board-rendering.md` (default 600 ms, range 400–1000 ms)
- `inter_step_pause_ms` → `board-rendering.md` (default 150 ms, range 100–300 ms)
- `fog_lift_duration_ms` → `board-rendering.md`
- Timer bar ease-out duration (120–150 ms), timer color cross-fade (300 ms), settlement panel transition (350 ms) → `shop-auction-ui.md`

## Visual/Audio Requirements

Card Animations does not own any specific visual assets or audio cues — it owns *how* existing assets are moved and timed. The visual and audio contracts below define what the animation pipeline must honour.

### V.1 — Visual Style Constraints

- **No glow/bloom on unit sprites.** Cel-shaded contract (Ankama/Wakfu aesthetic). Impact flashes are flat 1-frame color fills only.
- **Color palette is locked.** Animations must not use colors outside the registered palette: Prism White `#EEF4FF` (FIRST STRIKE / placement reveal), Warm Orange `#E07020` (standard combat impact), Crimson Slate (damage), Sky Blue `#3A8EDB` (Player A), Terracotta `#D45C22` (Player B), Arcane Gold `#F5C842` (UI accent), Ink Blue `#1A2D5A` (panels), Ivory (text). Flash colors in particular encode timing tier — Prism White = FIRST STRIKE; Warm Orange = standard. Using the wrong color for the wrong tier misinforms the player.
- **Smear frames are texture-atlas frame swaps, not blur.** 1 frame pre-impact (limb stretched), 1 frame impact (Prism White / Warm Orange fill), 1 frame recover. Total 200–250 ms. No motion blur post-process.
- **REPEL/ATTRACT displacement direction is strictly horizontal along the lane axis.** No diagonal or center-screen drift — the direction IS the rule explanation.
- **`TransformScaleXLens` health bar color threshold swaps are instantaneous.** Color change (green→yellow at 0.6, yellow→red at 0.3) and scale drain begin in the same frame — not animated as a color transition.
- **Damage numbers originate from unit torso position, not the health bar.** Float start position must be visually separated from the health bar to prevent occlusion.
- **Duration hierarchy for significance signaling:** objective death > unit death > combat advance > damage number > UI tick. No decorative animation may exceed 300 ms or it claims equal significance to an objective destruction.

### V.2 — Restraint Catalog (what must NOT animate)

- Units at rest: no idle breathing, bobbing, or ambient particles
- HUD resource counters: no passive pulse, glow, or shift except on value change
- Cards in hand at rest: fan is static; only the hovered card scales
- Phase transitions on routine rounds: phase banner slides once, no fanfare
- Board during PLACEMENT: frozen — no unit shuffle, no lane-indicator pulse
- Post-completion entities: completed-state `Animator<T>` components are inert; they must not produce residual motion

### V.3 — Audio Requirements

Card Animations does not own audio assets or trigger audio directly. All audio cues are owned by the systems that define the events (Board Rendering, Combat Resolution, Auction System, Shop/Auction UI). Card Animations is a terminal node and emits no signals, so upstream systems cannot observe internal animation frames directly. Audio timing uses an **offset-based model**: audio fires at a fixed millisecond offset after the domain event that triggers the animation, computed from the known tween timing constants in `GameConfig`.

**Audio timing offsets (upstream systems apply these relative to their domain event emit time):**
- Placement reveal flip: fire audio at `+27 ms` from `PlacementRevealAnimReady` receipt (≈ frame 2 of the 3-frame flip at the 80 ms minimum duration; frame 2 onset ≈ ⌊80 ms / 3⌋ = 27 ms). At flip duration 100 ms (top of range), frame 2 onset = ⌊100 / 3⌋ = 33 ms — the offset fires 6 ms early. If the art team settles on a specific flip duration, the offset should be updated to `⌊flip_duration_ms / 3⌋` using the config constant. **Polyphony:** `PlacementRevealAnimReady` carries all 5 lanes — fire **one** audio trigger per batch event, not one per lane. Five simultaneous triggers would clip the mix.
- Unit advance: fire footstep-shuffle audio at `+0 ms` from domain event emit (tween start). Per-lane audio stagger of ≤ 8 ms per lane is permitted for audio mix clarity (total spread across 5 lanes ≤ 32 ms — within the ~45 ms auditory simultaneity detection threshold). Stagger direction: lanes 1→5, ascending. Larger stagger values may cause the player to audibly perceive sequential rather than simultaneous advancing, which contradicts Rule C-4's parallelism guarantee. Visual remains simultaneous per Rule C-4.
- Impact flash: fire audio at `+impact_flash_audio_offset_ms` from domain event. **Config field required:** add `impact_flash_audio_offset_ms` to `game-config.md`. Default = 1 frame at 60fps ≈ **17 ms** (one frame duration — the impact flash is 1 frame, at the second frame of a 3-frame smear+impact+recover sequence; onset ≈ `⌈1000 / target_fps⌉`). **Note:** the former default of 67 ms was incorrect (assumed ~15 fps). **Sub-step coupling:** this offset is derived from frame count, not from `resolution_sub_step_duration_ms` — it does not need to change when sub-step duration is tuned. However, if the attack animation frame count changes, the offset must be recalculated.
- Objective destruction: fire audio at `+0 ms` from `ObjectiveDestroyedAnimReady` receipt (Prism White overlay begins on frame 1).
- Settlement overlay: fire audio at `+0 ms` from `SettlementOverlayRequested` receipt. **Audio ownership required:** the 1.5 s overlay hold is the most emotionally weighted moment in the draft loop. Audio content (win jingle, loss sting, or neutral music swell) is unspecified — the audio-director must assign ownership before the settlement story is implemented.
- Damage numbers: no audio — number visibility is sufficient feedback.

**Why offset-based:** Card Animations emits nothing (terminal node). Upstream systems already know the tween start moment (they emitted the domain event that triggered it). Fixed offsets derived from the timing constants in `GameConfig` are reproducible without any signal from Card Animations.

> **📌 Asset Spec** — Visual animation requirements are defined. After the art bible is approved, run `/asset-spec system:card-animations` to produce per-asset visual descriptions, dimensions, and generation prompts from this section.

## UI Requirements

Card Animations does not own any interactive panel, screen, or HUD element. It provides animation primitives invoked by upstream systems' UI layers. UI requirements are specified in the owning GDDs:

- Card fan animations, drag/snap-back → `hand-ui.md`
- Auction panel transitions, timer bar, gold counter → `shop-auction-ui.md`
- Board fog-lift, objective reveal, unit advance → `board-rendering.md`
- HUD gold/mana tick animation (once HUD GDD is authored) → `hud.md`

**One UI contract owned by Card Animations:** At most two UI regions may animate simultaneously during any phase transition (Rule C-14: Motion Soup prevention). This is a behavioral contract this system enforces; it does not require a dedicated UI screen but does require coordination with upstream systems during integration.

## Acceptance Criteria

| ID | Criterion | Type |
|---|---|---|
| CA-1 | **GIVEN** `CardAnimationsPlugin` is registered, **WHEN** `App::new()` builds with the plugin and executes one update, **THEN** the app completes without panic; each of the 5 custom lens types (`SpriteAlphaLens`, `BackgroundColorAlphaLens`, `SpriteColorLens`, `TransformScaleXLens`, `TextColorLens`) can be constructed in a `World`-based unit test and inserted into a `Tween` without compile or runtime error. | BLOCKING |
| CA-2 | **GIVEN** board entities exist with `Animator<Transform>` or `Animator<Sprite>` in `AnimatorState::Playing`, **WHEN** `BoardRebuildRequested` is written and `App::update()` is called once, **THEN**: (a) no entity that had a Playing animator is in `AnimatorState::Playing` after the tick (the cancellation system completes within the same tick), AND (b) those entities **still have** their `Animator<T>` components present — verified by `world.get::<Animator<Transform>>(entity).is_some()`. Clause (b) confirms cancel-in-place (Rule C-7: archetype churn avoidance) rather than incorrect component removal. | BLOCKING |
| CA-3 | **GIVEN** `PlacementRevealAnimReady` is received with 5 lane entries, **WHEN** one `App::update()` tick runs, **THEN** all 5 unit entities have `Animator` components in `AnimatorState::Playing`. Verified by asserting `Playing` state on all 5 entities post-update. Implementation note: the system must iterate all 5 entries in a single pass before any `apply_deferred` flush separates them; no `apply_deferred` may be injected mid-system by any registered plugin. Scheduling invariant must be validated via system-order inspection during code review in addition to this test. | BLOCKING |
| CA-4 | **GIVEN** a PLACEMENT-phase animation (snap-back or drag-lift) is in `AnimatorState::Playing`, **WHEN** `BoardRebuildRequested` fires in the same frame, **THEN** the `Animator` is no longer in `AnimatorState::Playing` after that frame. | BLOCKING |
| CA-4b | **GIVEN** CA-4's cancellation scenario, **WHEN** the frame renders, **THEN** no partially-tweened `Transform` visual position persists on screen. *(Screenshot evidence; manual QA only.)* | ADVISORY |
| CA-5a | **GIVEN** an `AnimQueue` with one `AnimGroup` of duration 600 ms and `S2CPhaseChanged(GAME_OVER)` written to the event queue, **WHEN** `Time<Virtual>` is advanced by 600 ms and `App::update()` runs, **THEN** `world.resource::<AnimQueue>().current_index == 0` (queue did not advance to a second group). | BLOCKING |
| CA-5b | **GIVEN** the same setup as CA-5a with `PendingObjectiveDestroyedEvents` pre-populated for 1 lane, **WHEN** `Time<Virtual>` is advanced by 600 ms and `App::update()` runs, **THEN** the objective-reveal `Animator` for that lane exists on the entity and is in `Playing` state (confirming `ResolutionObjectiveReveal` ran). | BLOCKING |
| CA-5c | **GIVEN** the same setup as CA-5b, **WHEN** `App::update()` runs at t=599 ms (one ms before group drains), **THEN** no `GameOverTransitionEvent` is present in the event queue; **WHEN** `advance_by(1 ms)` + `App::update()` runs, **THEN** CA-5b side-effects precede `GameOverTransitionEvent`. Two-tick test design required — single-tick ordering cannot be verified otherwise. | BLOCKING |
| CA-6 | **GIVEN** a unit entity has an `Animator<Transform>` in `TweenCompleted` state, **WHEN** a new tween is requested, **THEN** the entity retains its ECS entity ID (no despawn/respawn) AND the new animation begins within one `App::update()` tick. | BLOCKING |
| CA-7 | **GIVEN** a unit needs a simultaneous advance `Tween<Transform>` and death `SpriteAlphaLens` fade in the same `AnimGroup`, **WHEN** both tweens are spawned, **THEN** both `Animator<Transform>` and `Animator<Sprite>` exist on the entity and are both in `AnimatorState::Playing` after one tick. | BLOCKING |
| CA-8 | **GIVEN** a damage number entity spawned with `float_tween_duration_ms=500` and `fade_tween_duration_ms=500`, **WHEN** `Time<Virtual>` is advanced by 500 ms (`world.resource_mut::<Time<Virtual>>().advance_by(Duration::from_millis(500))`) and `App::update()` runs, **THEN** the entity is despawned (`World::get_entity()` returns `Err`). F2: `max(500, 500) = 500`. The despawn timer is set at spawn time from F2 and runs independently of any tween-completion event. | BLOCKING |
| CA-9 | **GIVEN** `float_tween_duration_ms=400` and `fade_tween_duration_ms=600`, **WHEN** `Time<Virtual>` is advanced by 400 ms and `App::update()` runs, **THEN** entity still exists; **WHEN** advanced a further 200 ms (total 600 ms) and `App::update()` runs, **THEN** entity is despawned. F2: `max(400, 600) = 600`. Entity must NOT be despawned at the float tween's 400 ms completion — the despawn timer is set from F2 at spawn time. | BLOCKING |
| CA-10 | **GIVEN** `GameConfig.stagger_cadence_ms=100` and `ObjectiveDestroyed` events for lanes 3 and 5, **WHEN** the reveal sequence starts, **THEN** lane 3's `Animator` is in `Playing` at t=0 ms and lane 5's `Animator` enters `Playing` at t=100 ms (±16 ms frame tolerance). Timing verified via `Time<Virtual>` injection: call `App::update()` after writing the events (no time advance needed — stagger timer at i=0 fires immediately on first tick); assert lane 3 `Animator` is `Playing`. Then `advance_by(Duration::from_millis(100))` + `App::update()`; assert lane 5 `Animator` enters `Playing`. | BLOCKING |
| CA-11 | **GIVEN** `GameConfig.stagger_cadence_ms=0` and two `ObjectiveDestroyed` events, **WHEN** the reveal sequence starts, **THEN** both reveal animations start in the same frame and no panic or undefined behavior occurs. F1: `reveal_start_ms[i] = 0` for all `i`. | BLOCKING |
| CA-12 | **GIVEN** any animation is requested during PLACEMENT phase with any `GameConfig` value for `snap_back_duration_ms`, **WHEN** the `Tween` is constructed, **THEN** the constructed tween's duration is clamped to ≤ 250 ms at construction time (`duration.min(Duration::from_millis(250))`). Verified by asserting `tween.duration().as_millis() <= 250` for each PLACEMENT animation type (drag-lift, snap-back, hover, cell-highlight) in separate spawn tests. Note: `snap_back_duration_ms` is a runtime `GameConfig` knob (range 100–250 ms); the 250 ms ceiling is a runtime clamp applied at animation construction, not a compile-time constant. | BLOCKING |
| CA-13 | **GIVEN** `TimerBarEaseRequested` is emitted, **WHEN** one `App::update()` tick runs, **THEN** the timer bar entity's `Animator` is in `AnimatorState::Playing` in the same frame as event receipt (0-frame latency). | BLOCKING |
| CA-13b | **GIVEN** `TimerBarEaseRequested` is processing a tween, **WHEN** the tween is in flight, **THEN** bid preset buttons are in enabled state. *(Manual UI walkthrough; screenshot evidence.)* | ADVISORY |
| CA-14 | **GIVEN** the CI pipeline runs on every merge to main, **WHEN** `grep -rn "EventReader<S2C\|MessageReader<S2C"` is run against `src/card_animations/`, **THEN** exit code is 1 (no matches found). Story is not Done until this CI step exists in the pipeline configuration and passes. If CI is not yet configured, this AC is BLOCKED until CI exists. (Rule C-14 classifies direct S2C subscription as BLOCKER-severity — its test AC must be BLOCKING.) | BLOCKING |
| CA-15 | **GIVEN** a unit entity needs simultaneous REPEL displacement (`Tween<Transform>`) and placement-reveal flip (`SpriteColorLens` + `TransformScaleXLens`), **WHEN** both tweens are spawned, **THEN** all three animators exist on the entity simultaneously and are in `AnimatorState::Playing` after one tick. | BLOCKING |
| CA-16 | **GIVEN** `GameConfig.stagger_cadence_ms=120` (maximum defined cadence) and `ObjectiveDestroyed` events for 4 lanes (i=0 through i=3 — the maximum in-production count), **WHEN** the reveal sequence starts, **THEN** the `i=3` lane's `Animator` enters `Playing` at 360 ms (3 × 120 ms, ±16 ms tolerance), verified via `Time<Virtual>` injection: `advance_by(Duration::from_millis(360))` + `update()`. This tests F1's upper bound at its defined domain limit (i max=3, cadence max=120 ms). | BLOCKING |
| CA-17 | **GIVEN** an `AnimQueue` resource with `groups.len() == 0`, **WHEN** the `ResolutionExecuting` drain system runs and `Time<Virtual>` is advanced by `pre_animation_pause_ms`, **THEN** no tween-spawning commands are issued, no panic occurs, and `PendingPhaseChange` is drained (the buffered `S2CPhaseChanged` event is emitted after the pre-pause timer fires). Verified by asserting zero new `Animator` inserts and `PendingPhaseChange` is `None` after the tick. | BLOCKING |
| CA-18 | **GIVEN** a unit entity at `Transform::IDENTITY` requires two simultaneous `Tween<Transform>` animations — tween A moving X from 0.0 to 100.0 over 600 ms, tween B moving Y from 0.0 to 60.0 over 600 ms — wrapped in a `Tracks<Transform>`, **WHEN** `Time<Virtual>` is advanced by 16 ms and `App::update()` runs, **THEN**: (a) `world.query::<&Animator<Transform>>().iter(&world).filter(|(e, _)| *e == target).count() == 1` (exactly one `Animator<Transform>` on the entity), AND (b) the entity's `Transform.translation.x` is in the range (0.0, 100.0) exclusive, AND (c) the entity's `Transform.translation.y` is in the range (0.0, 60.0) exclusive. The range assertions prove both tweens are advancing simultaneously without prescribing an exact easing-curve value. | BLOCKING |
| CA-19 | **GIVEN** an `Animator<Transform>` reaches `TweenCompleted` on a unit entity, **WHEN** no new tween is requested, **THEN** the `Animator<Transform>` component remains on the entity. Verified via `World::get::<Animator<Transform>>(entity)` returning `Some(...)` after completion. | BLOCKING |
| CA-20 | **GIVEN** the client app runs with `CardAnimationsPlugin`, **WHEN** each domain event is fired in a controlled test, **THEN** no "EventReader has no receivers" warning is logged. *(Smoke test — confirms all events registered by upstream plugins before CardAnimationsPlugin runs.)* | ADVISORY |
| CA-21 | **GIVEN** a PLACEMENT-phase animation (snap-back, drag-lift, or cell-highlight) is in `AnimatorState::Playing` on an entity that carries a `BoardCell` component (owned by Board Rendering — field `world_position: Vec3`), **WHEN** `PlacementCancelAllAnimsRequested` is processed (emitted by Board Rendering on `S2CPhaseChanged(RESOLUTION)`, delivered same-frame via SystemSet ordering), **THEN**: (a) all in-flight PLACEMENT animators are no longer in `Playing` state after `App::update()`, verified via `Query<&Animator<Transform>>`, AND (b) `world.get::<Transform>(entity).unwrap().translation == world.get::<BoardCell>(entity).unwrap().world_position`. This AC is **BLOCKED** until board-rendering.md specifies the `BoardCell` component field name. | BLOCKING |
| CA-22 | The "≤ 2 animated UI regions per phase transition" rule (Rule C-14: Motion Soup) is a design discipline constraint enforced by upstream event sequencing — not by a Card Animations runtime guard. DRAFT_INITIAL entry is compliant by construction: panel slide fires tick 1; card-draws fire at t+350 ms (separate tick); `HandShowRequested` is an instant `Visibility` change (not a tween). Compliance is verified by: (a) code review of upstream plugin event-emit timing to confirm sequencing invariants, (b) manual playtest walkthrough at DRAFT_INITIAL entry confirming panel and card-draw animations do not fire simultaneously (screenshot evidence in `production/qa/evidence/`). HUD entry animations at phase transitions must also be confirmed non-overlapping during integration. *(Demoted from BLOCKING: automated ECS testing of "UI region ancestry" requires marker components — `UiRegionTag{Hand\|AuctionPanel\|Board\|Hud}` — not yet specified in any GDD. If these are added, promote to BLOCKING with `Added<Animator<T>>` + `With<UiRegionTag>` query.)* | ADVISORY |
| CA-23 | **GIVEN** a card in the player's hand during PLACEMENT phase, **WHEN** a drag-start input event is processed by `App::update()`, **THEN** the drag sprite entity has an `Animator` in `AnimatorState::Playing` in the same tick (0-frame Bevy-side latency). Note: WASM platform input latency (16–50 ms from physical mouse-down to Bevy event receipt) is a known floor not addressable at this layer; this AC validates Bevy-side responsiveness only. | BLOCKING |
| CA-24 | **GIVEN** card A has an active `Animator<Transform>` in `AnimatorState::Playing` (de-hover tween returning to 1.0×), **WHEN** a hover-in event fires on card B before card A's tween completes, **THEN**: (a) card A's `Animator<Transform>` is no longer in `AnimatorState::Playing` (cancel-replace executed), AND (b) card B's `Animator<Transform>` is in `AnimatorState::Playing`, AND (c) a query `Query<&Animator<Transform>, With<HandCard>>` returns exactly one entity in `Playing` state. Confirms Rule C-13's "at most one card in hover-scaled state" invariant and the replacement tween spec (return to 1.0× from intermediate scale). | BLOCKING |

## Open Questions

**OQ-CA-01 — `Animator::set_tweenable()` API and `AnimatorState` enum in bevy_tweening 0.18** *(Owner: gameplay-programmer)*
The `set_tweenable()` method is the cornerstone of the cancel-replace pattern (Rule C-7, board-rendering BR-16). Confirm: (a) method exists with this exact name in bevy_tweening 0.18, (b) it resets progress to 0 (not resume from current). Also confirm: (c) the animator state enum is named `AnimatorState` (historically `AnimationState` in earlier versions) with variant `Playing` — ACs CA-2, CA-4, CA-7, CA-15 depend on the correct public name. If renamed or removed, the cancel-replace pattern requires a different implementation. **Must resolve before any story touching cancel-replace.**

**OQ-CA-02 — `Tracks<T>` API in bevy_tweening 0.18** *(Owner: gameplay-programmer)*
`Tracks` is required for same-entity parallel `Transform` animations (Rule C-7, CA-18). Confirm: (a) `Tracks<T>` is present in 0.18, (b) it wraps into an `Animator<T>` directly. If gone, workaround is child entities for the secondary transform. **Must resolve before any story touching REPEL + simultaneous advance.**

**OQ-CA-03 — `TweenCompleted` signal in Bevy 0.18** *(Owner: gameplay-programmer)*
bevy_tweening previously emitted a `TweenCompleted` event (`user_data: u64`) on tween completion. In Bevy 0.16+ the Event system changed (`EventWriter`/`EventReader` removed — `MessageWriter`/`MessageReader` only). Confirm how bevy_tweening 0.18 signals completion: Bevy `Message`, `Observer`, or internal callback. The `AnimQueue` drain uses `Time<Virtual>` timers (not completion callbacks) precisely to sidestep this uncertainty — this OQ informs whether completion callbacks are available for any use at all.

**OQ-CA-04 — `TextColor` as a lens target** *(Owner: gameplay-programmer)*
`TextColor` is a newtype wrapper around `Color` introduced in Bevy 0.15+. Confirm `TextColor` satisfies the `Component` bound required by bevy_tweening's `Lens<T>`. If not, damage-number fade must use a `Sprite` overlay entity instead of a `Text` entity, or a `Lens<Text>` impl that reaches into text sections.

**OQ-CA-05 — `bevy_tweening = "0.18"` on crates.io** *(Owner: devops-engineer)*
Confirm that `bevy_tweening = "0.18"` resolves to a published crate compatible with Bevy 0.18 (not just version-number matching). Verify before locking into `Cargo.toml`. **Highest priority OQ — blocks all implementation.**

**OQ-CA-06 — Domain event API: Bevy 0.18 Message API** *(Owner: lead-programmer)* ⚠️ REOPENED — previously marked "Resolved" with wrong API
`EventWriter<T>` / `EventReader<T>` / `app.add_event::<T>()` **do not exist in Bevy 0.17+** (confirmed by project engine-reference `breaking-changes.md`). The correct 0.18 intra-client message API is: `#[derive(Message)]` on the event struct, `app.add_message::<T>()` for registration, `MessageWriter<T>` for emission, `MessageReader<T>` for consumption. **All ACs referencing event receipt must be written against `MessageReader<T>` once this is confirmed.** Convention (same as prior): each upstream plugin registers its own message types in its own `build()`; `CardAnimationsPlugin` consumes via `MessageReader<T>` — no re-registration needed. Confirm `add_message` idempotency in 0.18 before finalising. **Must resolve before any story is written — all 24 ACs reference event receipt.**

**OQ-CA-07 — "YOU WON" overlay duration** *(RESOLVED — 2026-04-30)*
The 1.5 s overlay duration is justified as an affective signal: the settlement moment is the auction's emotional payoff and warrants a deliberate dwell that goes beyond informational minimums. This satisfies D4 as an affective-minimum hold (not an information-delivery hold). The Decoration Test exemption is documented in the Shop/Auction UI Interactions row. No change to this GDD required. The shop-auction-ui.md revision recommendation (reduce to 0.8 s) is on record but deferred — this GDD accepts 1.5 s as designed.

**OQ-CA-08 — "NO BIDS" fade duration (cross-system, Shop/Auction UI)** *(Owner: shop-auction-ui.md)*
Game-designer recommends reducing the "NO BIDS — CARD LOST" card fade from 400 ms to 250 ms total (200 ms desaturation + 50 ms collapse, or desaturate-and-hold until panel transition). The 400 ms is the most generous animation budget in DRAFT phase for the outcome with the least emotional weight. Decision belongs to Shop/Auction UI GDD.

**OQ-CA-09 — RANGE projectile speed floor (cross-system, Combat Resolution)** *(Owner: combat-resolution.md)*
At the current `projectile_speed_px_per_s` floor of 600 px/s, a cross-board RANGE projectile on an 8-cell board travels ~640 ms — exceeding the 600 ms `resolution_sub_step_duration_ms` cap by 40 ms. Game-designer recommends raising the floor to 850 px/s in `GameConfig` so maximum travel stays ≤ 600 ms. Alternatively, the 150 ms `inter_step_pause_ms` absorbs the 40 ms overflow. Decision belongs to Combat Resolution / Game Config.

**OQ-CA-11 — Jitter table definition for simultaneous damage numbers** *(Owner: gameplay-programmer)*
Edge Cases §6 references `event_id % jitter_table_len` for deterministic damage-number position offset when two sources hit the same unit in one sub-step. `jitter_table_len` and the table contents are undefined. Define as F3 with a variable table (e.g., 8-entry table of Vec2 offsets, `jitter_table_len = 8`) or cross-reference to the `DamageNumberSpawnRequested` event payload schema that owns `event_id`. **Must resolve before damage number spawn stories are implemented.**

**OQ-CA-12 — `ResolutionExecuting` run condition** *(Owner: gameplay-programmer)*
The `ResolutionExecuting` drain system should run only during `BoardRenderState::ResolutionExecuting` to avoid unnecessary per-frame polls during DRAFT and PLACEMENT phases. Confirm the correct Bevy 0.18 pattern for run conditions (`.run_if(in_state(BoardRenderState::ResolutionExecuting))`) and add it to the system registration in `CardAnimationsPlugin::build()`.

**OQ-CA-10 — Bevy 0.18 `Sprite` and `BackgroundColor` alpha API verification** *(Owner: gameplay-programmer)*
`SpriteAlphaLens` calls `sprite.color.set_alpha(value)` and `SpriteColorLens` writes to `sprite.color`. `BackgroundColorAlphaLens` calls `background_color.0.set_alpha(value)`. In Bevy 0.14 these paths were valid; in 0.15–0.18 the `Color` type was overhauled and alpha access may require a different path (e.g., `sprite.color.to_linear().alpha` or a `LinearRgba` accessor). Confirm: (a) `Sprite.color` field name and type in 0.18, (b) `set_alpha()` method availability on that type, (c) `BackgroundColor` struct layout (still a `Color` newtype?). **Must resolve before any lens implementation — same priority as OQ-CA-05.**
