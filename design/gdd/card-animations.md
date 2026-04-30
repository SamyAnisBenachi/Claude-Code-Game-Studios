# Card Animations

> **Status**: In Design
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
| **D5** | Teach-by-showing exemption | Animations that teach a rule on first encounter (REPEL shove, COUNTERATTACK recoil, TRAP flip) pass D2 even if experienced players know the rule, *provided* they stay within phase budget and do not gate input. |

**The protective rule (from Player Fantasy):** *If removing the animation would not change what the player knows, the animation is decoration and must be cut or shortened.*

---

**Rule C-2 — Animation Budget by Phase.**

Per-phase ceilings on animation duration. The Decoration Test (C-1) is necessary but not sufficient — duration ceilings are also enforced.

| Phase | Polish budget (non-blocking) | Transition budget (state-boundary) | Forbidden |
|---|---|---|---|
| `LOBBY` | 350 ms | 350 ms | Any animation > 500 ms |
| `DRAFT_INITIAL` (45 s) | 150 ms (gold tick, hover, slot-fade) | 350 ms (panel slide, card-draw 280 ms) | Any > 350 ms; sequential card acquisitions |
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
- **Queue-during-queue (defensive):** RSM guarantees this cannot happen (one `S2CResolutionEvent` per round). If a second arrives mid-queue: log error, discard in-flight queue, load new event as fresh queue from `groups[0]`. Do not silently merge or append.

---

**Rule C-9 — Plugin Architecture.**

One `CardAnimationsPlugin`. Single plugin, internal modules (`lenses/`, `animators/`, `queue/`, `events/`). Per-domain plugins would split the lens library across files for no gain and create plugin-ordering dependency problems.

`CardAnimationsPlugin::build()` responsibilities:
1. Add `bevy_tweening::TweeningPlugin` if not already registered.
2. Register the 5 custom lens types.
3. `app.add_message::<T>()` for each domain event Card Animations consumes (see Rule C-10).
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
                           │ Bevy 0.18 Messages (intra-client)
                           ▼
┌────────────────────────────────────────────────────────────────────┐
│ Card Animations: spawn tweens via custom lens library             │
│ (no Lightyear dependency, no game-state mutation)                 │
└────────────────────────────────────────────────────────────────────┘
```

**Domain events Card Animations consumes** (intra-client `Message` types — full schemas TBD per upstream GDD):

- From Board Rendering: `PlacementRevealAnimReady`, `ResolutionGroupReady`, `ObjectiveDestroyedAnimReady`, `FogLiftReady`, `BoardRebuildRequested` (cancel all animators)
- From Hand UI: `CardAcquiredAnimReady`, `SnapBackRequested`, `HandHideRequested`, `HandShowRequested`
- From Shop/Auction UI: `AuctionPanelTransitionRequested`, `TimerBarEaseRequested`, `TimerColorZoneRequested`, `GoldTickRequested`, `SettlementOverlayRequested`, `NoBidsTransitionRequested`
- From Keyword System: `DisplacementAnimRequested`, `TrapFlipRequested`, `AuraPulseRequested`

**Why indirection beats direct S2C subscription:**

- Ordering invariant ("game state updated *before* animation starts") is encoded in message emission, not in `before()`/`after()` system constraints.
- Card Animations has zero Lightyear dependency — pure presentation layer.
- Adding a new animation trigger = one new message type + one `MessageWriter` call upstream.

**Cost:** ~15 narrow domain message types. Naming convention: `[Action][Subject]Requested` for triggers; `[Action][Subject]AnimReady` when the upstream wants to signal "state is updated, animation may begin." (See OQ-CA-06 for owner-side registration coordination.)

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
- **Cards in hand do not bob.** Fan is static at rest. Only the hovered card scales.
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
| **Board Rendering** | Upstream (events in) → Card Animations | Emits `PlacementRevealAnimReady`, `ResolutionGroupReady`, `ObjectiveDestroyedAnimReady`, `FogLiftReady`, `BoardRebuildRequested` after authoritative state updates. Card Animations consumes these and spawns tweens. **Critical:** `BoardRebuildRequested` (on `S2CGameSnapshot`) cancels all in-flight tweens and clears any per-entity animator state — no replay on reconnect. |
| **Hand UI** | Upstream → Card Animations | Emits `CardAcquiredAnimReady` (card-to-fan slide, 280 ms), `SnapBackRequested` (220 ms `EaseOutBack`), `HandHideRequested` (instant Visibility::Hidden, no exit animation), `HandShowRequested` (instant snap-on, no entry animation). |
| **Shop/Auction UI** | Upstream → Card Animations | Emits `AuctionPanelTransitionRequested` (350 ms slide), `TimerBarEaseRequested` (120–150 ms ease-out + 60 ms flash), `TimerColorZoneRequested` (300 ms cross-fade), `GoldTickRequested` (≤ 150 ms tick), `SettlementOverlayRequested` (1.5 s overlay — see OQ-CA-07), `NoBidsTransitionRequested` (200 ms desat + 400 ms fade — see OQ-CA-08). |
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
| Lane sort index | `i` | u8 | 0–4 | Zero-indexed position in the ascending-lane-sorted `ObjectiveDestroyed` list. `i=0` = lowest lane number with a destruction event this RESOLUTION. |
| Stagger cadence | `stagger_cadence_ms` | u32 | 80–120 | ms between sequential objective reveal animations. Loaded from `GameConfig`. Default 100 ms. |

**Output range:** 0 ms (`i=0`) to 4 × 120 ms = 480 ms (`i=4` at max cadence; edge case only — at most 2 objectives per player can be destroyed in one session).

**Example:** Two objectives destroyed in lanes 3 and 5 → sorted: lane 3 (`i=0`), lane 5 (`i=1`). At default 100 ms cadence: lane 3 reveals at 0 ms, lane 5 at 100 ms.

**Invariant:** The input list must be sorted ascending by lane number before this formula is applied. Message arrival order from `ObjectiveDestroyed` is not guaranteed (NP-OQ-3). Card Animations must sort before computing.

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

- **If a PLACEMENT animation (snap-back, drag-lift, cell-highlight) is mid-tween when `S2CPhaseChanged(RESOLUTION)` arrives and `BoardRebuildRequested` fires:** all in-flight animators are cancelled and cleared atomically by the board rebuild. The entity's `Transform` is overwritten in the same frame. The 250 ms PLACEMENT hard cap (Rule C-2) makes this race window narrow, but any mid-tween state is discarded — no partial-tween visual artifact persists.

- **If `S2CGameSnapshot` (→ `BoardRebuildRequested`) arrives mid-RESOLUTION while an `AnimGroup` is executing:** the queue is cleared, all active `Animator<Transform>` and `Animator<Sprite>` components are cancelled, and any in-flight damage-number entities are despawned as part of the full board rebuild. No stale floating numbers survive the reconnect. The reconnecting client enters the post-snapshot phase directly with no animation replay (board-rendering Rule 11).

- **If `S2CPhaseChanged(GAME_OVER)` arrives during an `AnimGroup` execution mid-RESOLUTION:** the current `AnimGroup` completes in full (board-rendering Rule 10 — no mid-tween interrupt), remaining groups are skipped, then `ResolutionObjectiveReveal` runs for any buffered `ObjectiveDestroyed` events, then the system transitions to `GameOver`. Card Animations continues ticking the current group's tweens normally — the decision to complete or skip the queue belongs to the Board Rendering state machine, not to Card Animations.

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
| HUD | `gdd/hud.md` | Downstream. HUD may consume animation-driving events for gold/mana tick updates. HUD GDD (Not Started) should list Card Animations as a provider of gold counter tick animation. No reverse dependency. |

**Downstream dependents of Card Animations:** None — terminal node in the dependency graph. All other systems fire events that Card Animations consumes; no system depends on Card Animations emitting anything.

## Tuning Knobs

[To be designed]

## Visual/Audio Requirements

[To be designed]

## UI Requirements

[To be designed]

## Acceptance Criteria

[To be designed]

## Open Questions

[To be designed]
