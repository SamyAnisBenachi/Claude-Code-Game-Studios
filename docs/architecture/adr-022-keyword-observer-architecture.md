# ADR-022: Keyword System — Timing Trigger Observer Architecture

## Status
Accepted

## Date
2026-04-30

## Engine Compatibility

| Field | Value |
|-------|-------|
| **Engine** | Bevy 0.18 + Lightyear 0.26 |
| **Domain** | Core / ECS — Observer Pattern |
| **Knowledge Risk** | HIGH — Bevy 0.17 introduced the Event/Observer split (post-cutoff); Bevy 0.18 stable |
| **References Consulted** | `docs/engine-reference/bevy/breaking-changes.md`, `docs/engine-reference/bevy/current-best-practices.md`, `docs/engine-reference/bevy/VERSION.md`, ADR-017, ADR-018 |
| **Post-Cutoff APIs Used** | `#[derive(Event)]` + `world.trigger_targets()` / `commands.trigger_targets()` + `app.observe()` (Bevy 0.17+ Observer split); `On<T>` as observer handler param type (Bevy 0.17+); deferred `commands.trigger_targets()` for DRAFT-phase dispatch |
| **Verification Resolved** | All 5 items resolved 2026-04-30 (item 2 corrected 2026-05-01) against `breaking-changes.md`. **(1) `world.trigger_targets(event, entity)` CONFIRMED** as a valid `World` method in Bevy 0.18. It fires observers synchronously within the exclusive system call; `commands.trigger_targets()` is the deferred alternative and requires `world.flush()` — incompatible with synchronous RESOLUTION sub-step semantics. `world.trigger_targets()` is the correct path. **(2) `On<T>` CONFIRMED** as the correct observer handler parameter type for Bevy 0.17+. `breaking-changes.md` line 140 (Bevy 0.17 section) explicitly uses `On<UnitDied>` in the observer example. `current-best-practices.md` line 86 previously showed `Trigger<UnitDied>` — that was a stale value; the file has been corrected to `On<T>`. All ADR handler signatures updated to `On<T>`. **(3) `ResMut<T>` CONFIRMED** usable inside Observer handlers. Standard system params work in observer handlers by Bevy design. The drain loop in `execute_ss4()` drops the `ChainDeathBuffer` borrow (via `pop_front()` returning an owned value) before `trigger_targets()` is called — no simultaneous borrow conflict. Validate with integration smoke test during first keyword implementation story. **(4) `MessageWriter<T>` CONFIRMED** usable inside Observer handlers by the same reasoning as item 3 — it is a standard system param; messages are buffered and no re-entrancy conflict arises. Validate with smoke test (fire `UnitDied`, assert `KeywordTriggered` message emitted). **(5) `commands.trigger_targets()` CONFIRMED** — `breaking-changes.md` line 139 lists `commands.trigger() / trigger_targets()`. `DraftPhaseEntered` must be registered with `app.add_message::<DraftPhaseEntered>()` in the RSM plugin (the emitter), not in `KeywordPlugin` (the reader) — `KeywordPlugin` only reads via `MessageReader<DraftPhaseEntered>`. |

## ADR Dependencies

| Field | Value |
|-------|-------|
| **Depends On** | ADR-017 (Accepted — exclusive system execution; all RESOLUTION in `fn resolve_combat(world: &mut World)`); ADR-018 (Proposed — `UnitKeywordState` component, `server/feature/keyword/` module structure; this ADR extends that module) |
| **Enables** | Keyword timing trigger stories (APPEARANCE KW-001, DEATH KW-002/003, FINAL BLOW KW-004a/b, COUNTERATTACK KW-005/006, INJURED KW-007/008, START OF TURN KW-009a/b, END OF TURN KW-010a/b) |
| **Blocks** | Any story implementing a keyword timing trigger — requires this ADR Accepted before observer wiring can be coded |
| **Ordering Note** | This ADR may be Accepted concurrently with ADR-018 — it depends on ADR-018's module structure but does not alter any ADR-018 decision. The 5 Verification Required items above must be resolved before keyword implementation stories open. |

## Context

### Problem Statement

ADR-018 defined `UnitKeywordState` component storage, the `server/feature/keyword/` module structure, and the interface contract ("Combat Resolution calls into `keyword::effects::*` functions"). It left the dispatch mechanism for the 7 keyword timing triggers unspecified:

1. How does `resolve_combat` discover which units have APPEARANCE keywords and fire them after SS1 spawn commits?
2. How is the DEATH trigger chain dispatched — including sequential chaining (A's effect fires completely before B's) and lane-ordered simultaneous deaths?
3. How is FINAL BLOW wired to fire on the attacker entity in SS3 or SS6 at the killing blow?
4. How does COUNTERATTACK dispatch given its proximity precondition (same cell or collision-halted adjacent contact; never RANGE)?
5. How is INJURED re-evaluated at each sub-step boundary (not per-attack)?
6. How do START OF TURN and END OF TURN fire at their respective RSM phase events?

Without this ADR, implementers either inline all keyword discovery into `resolve_combat` (coupling combat with keyword logic) or invent ad-hoc dispatch patterns. The engine reference (`current-best-practices.md`) explicitly recommends Bevy Observers for APPEARANCE, DEATH, and FINAL BLOW. This ADR formalizes which triggers use Observers and which use inline dispatch.

### Constraints

- All RESOLUTION sub-step processing runs inside `fn resolve_combat(world: &mut World)` (ADR-017). Any trigger mechanism used within RESOLUTION must be synchronous — no frame-deferred dispatch.
- `EventWriter`/`EventReader` do not exist in Bevy 0.17+. Timing triggers must use `#[derive(Event)]` + Observer (synchronous push) or `#[derive(Message)]` + `MessageWriter/Reader` (buffered, frame-delayed — unacceptable within RESOLUTION sub-steps).
- ADR-014 forbids standalone Bevy systems for RESOLUTION-phase effects. Bevy Observers fired via `world.trigger_targets()` inside `resolve_combat` are not standalone systems — they fire synchronously inline. This pattern is not banned.
- The GDD boundary: "Combat Resolution owns when; Keyword System owns what." CR fires the trigger event; the Observer in `keyword/observers.rs` owns the effect.

### Requirements

- APPEARANCE, DEATH, FINAL BLOW, END OF TURN must fire synchronously within `resolve_combat`'s single frame
- DEATH chain: A's effect resolves completely before B's fires; initial deaths seeded in lane order (Lane 1 first)
- COUNTERATTACK dispatch must be gated on proximity check before any effect runs
- INJURED re-evaluation must scan all units at sub-step boundaries, not per-attack
- START OF TURN fires at DRAFT phase entry (outside `resolve_combat`) — deferred dispatch acceptable

## Decision

### Part 1: Trigger Classification

Five timing triggers use global Bevy Observers; two use inline dispatch:

| Trigger | Pattern | Reason |
|---------|---------|--------|
| APPEARANCE | Observer (`UnitAppeared`) | Reactive per-entity event in SS1; entity-scoped, no precondition |
| DEATH | Observer (`UnitDied`) | Reactive per-entity event in SS4; chain managed via explicit queue (Part 3) |
| FINAL BLOW | Observer (`FinalBlowDealt`) | Fires on the attacker entity; entity-scoped, attacker context required |
| START OF TURN | Observer (`StartOfTurnTriggered`) | Per-entity at DRAFT phase entry; dispatched by a normal Bevy system |
| END OF TURN | Observer (`EndOfTurnTriggered`) | Per-entity after SS6; consistent with APPEARANCE pattern |
| COUNTERATTACK | **Inline** | Proximity precondition (same cell or collision-halted adjacent; RANGE excluded) must be evaluated against live board state before dispatch; inline conditional call is simpler and more explicit |
| INJURED | **Inline** | State re-evaluation (not a per-event trigger); scan-based at sub-step boundaries |

### Part 2: Event Types

```rust
// server/feature/keyword/events.rs

/// Fires at SS1 after a unit commits from PlacementBuffer to the board.
/// resolve_combat fires this per-entity after board::api::spawn_unit() commits.
#[derive(Event)]
pub struct UnitAppeared {
    pub sub_step: u8,
}

/// Fires in SS4 when a unit's HP reaches 0.
/// Fired for each link in the DEATH chain via ChainDeathBuffer drain loop.
///
/// NAMING NOTE: This is a server-internal Bevy Event (#[derive(Event)]) — distinct
/// from the `ResolutionEvent::UnitDied { unit_id, lane, cell, killer_id }` protocol
/// variant in S2CResolutionEvent (addressed by OQ-NP3 in network-protocol.md).
/// Both share the name but live in different crates and serve different purposes.
#[derive(Event)]
pub struct UnitDied {
    pub attacker: Option<Entity>,
}

/// Fires on the ATTACKER entity at the killing blow in SS3 or SS6.
/// Not deferred to SS4 — fires in the sub-step of the kill (SS3 for FIRST STRIKE,
/// SS6 for standard). If two damage sources in the same sub-step kill a unit, fires
/// on the second source (the one that reduced HP to 0).
#[derive(Event)]
pub struct FinalBlowDealt {
    pub killed: Entity,
    pub sub_step: u8,
}

/// Fires per-unit at DRAFT phase entry after mana ramp + gold income (RSM Rule 3).
/// Dispatched by start_of_turn_dispatch_system (normal Bevy system, DRAFT phase only).
#[derive(Event)]
pub struct StartOfTurnTriggered;

/// Fires per alive unit after SS6 completes, before ResolutionComplete is written.
#[derive(Event)]
pub struct EndOfTurnTriggered;
```

### Part 3: DEATH Chain — Explicit Queue Architecture

DEATH chains are sequential: A's effect resolves completely before B's fires. The chain is managed by a temporary `ChainDeathBuffer` resource — **not** by recursive `world.trigger_targets()` calls inside the observer handler. Recursive `trigger_targets()` within an observer handler has unverified borrow semantics in Bevy 0.18 (see Verification Required item 1). Explicit queue is always safe and directly expresses the GDD's sequential contract.

```rust
// server/feature/keyword/resources.rs

/// Temporary buffer for DEATH chain dispatch during SS4.
/// Cleared at SS4 start. on_unit_died pushes chain deaths here when a DEATH
/// effect reduces another unit's HP to 0. resolve_combat's SS4 loop drains
/// until empty. VecDeque preserves lane-ordered insertion order.
#[derive(Resource, Default)]
pub struct ChainDeathBuffer(pub VecDeque<(Entity, Option<Entity>)>);
```

**resolve_combat SS4 pattern:**

```rust
fn execute_ss4(world: &mut World) {
    // 1. Collect units with HP <= 0; sort by lane order (Lane 1 first)
    let initial_deaths = collect_dead_units_lane_ordered(world);

    // 2. Seed chain buffer
    world.resource_mut::<ChainDeathBuffer>()
        .0
        .extend(initial_deaths.into_iter().map(|e| (e, None)));

    // 3. Drain — on_unit_died may push new entries for chain deaths
    loop {
        let next = world.resource_mut::<ChainDeathBuffer>().0.pop_front();
        let Some((entity, attacker)) = next else { break };

        board::api::remove_unit_from_board(world, entity);
        economy::api::award_kill_gold(world, attacker, entity);
        // Fire observer — see Verification Required item 1 for exact call site API
        world.trigger_targets(UnitDied { attacker }, entity);
    }

    world.resource_mut::<ChainDeathBuffer>().0.clear(); // defensive
}
```

**Chain depth bound:** Max 9 links (≤10 units on board; each unit dies once). `remove_unit_from_board` removes the entity from `BoardState`'s position index — subsequent board queries exclude it, guaranteeing structural termination without an explicit "already-dead" set.

**Sequential borrow safety:** `world.trigger_targets()` followed by `world.resource_mut::<ChainDeathBuffer>()` in the same function body is safe in Rust — each call is a discrete mutable borrow that ends before the next begins.

### Part 4: Observer Registration and Handler Signatures

```rust
// server/feature/keyword/mod.rs

impl Plugin for KeywordPlugin {
    fn build(&self, app: &mut App) {
        // 5 global observers for timing triggers
        app.observe(on_unit_appeared);
        app.observe(on_unit_died);
        app.observe(on_final_blow_dealt);
        app.observe(on_start_of_turn);
        app.observe(on_end_of_turn);

        // DRAFT-phase dispatcher (normal system — not subject to the RESOLUTION
        // standalone-system forbidden pattern)
        app.add_systems(Update, start_of_turn_dispatch_system);

        // Temporary resource for DEATH chain
        app.init_resource::<ChainDeathBuffer>();

        // Messages (from ADR-018, unchanged)
        app.add_message::<KeywordTriggered>();
    }
}
```

**Guard pattern — mandatory in all observer handlers.** Global observers fire for ALL entities that receive the trigger. Each handler must check keyword presence as its first operation and return early if the entity lacks the relevant keyword:

```rust
// Example guard pattern (on_unit_died):
pub fn on_unit_died(
    trigger: On<UnitDied>,
    units: Query<(&UnitKeywordState, &UnitBoardOwner)>,
    mut chain_buffer: ResMut<ChainDeathBuffer>,   // see Verification Required (3)
    mut keyword_triggered: MessageWriter<KeywordTriggered>, // see Verification Required (4)
) {
    let entity = trigger.target();
    let Ok((kw_state, owner)) = units.get(entity) else { return; };
    if !kw_state.has_keyword(SimpleKeyword::Death) { return; }
    // Apply DEATH effect; push chain deaths to chain_buffer if any unit HP reaches 0
}
```

Handler signatures for the other four:

```rust
// server/feature/keyword/observers.rs

pub fn on_unit_appeared(
    trigger: On<UnitAppeared>,
    units: Query<(&UnitKeywordState, &CardId)>,
    card_catalog: Res<CardCatalog>,
    mut keyword_triggered: MessageWriter<KeywordTriggered>,
) { ... }

pub fn on_final_blow_dealt(
    trigger: On<FinalBlowDealt>,
    units: Query<&UnitKeywordState>,
    mut keyword_triggered: MessageWriter<KeywordTriggered>,
) { ... }

pub fn on_start_of_turn(
    trigger: On<StartOfTurnTriggered>,
    units: Query<(&UnitKeywordState, &UnitBoardOwner)>,
    // economy / hand resources per card effect
) { ... }

pub fn on_end_of_turn(
    trigger: On<EndOfTurnTriggered>,
    units: Query<(&UnitKeywordState, &UnitBoardOwner)>,
    mut keyword_triggered: MessageWriter<KeywordTriggered>,
) { ... }
```

### Part 5: COUNTERATTACK and INJURED — Inline Dispatch

**COUNTERATTACK** — called inline from `resolve_combat` at SS3 and SS6 after damage resolves. The proximity check gates all dispatch:

```rust
// Called from resolve_combat after damage is applied to `defender` in SS3/SS6
fn check_and_apply_counterattack(
    world: &mut World,
    defender: Entity,
    attacker: Entity,
    sub_step: u8,
) {
    let Ok(kw_state) = world.get::<UnitKeywordState>(defender) else { return; };
    if !kw_state.has_keyword(SimpleKeyword::Counterattack) { return; }
    if !keyword::effects::check_counterattack_proximity(world, defender, attacker) { return; }
    keyword::effects::apply_counterattack(world, defender, attacker, sub_step);
}
```

`world.get::<C>(entity)` returning `Option<&C>` is a confirmed stable exclusive-system API (Verification Required item 4 confirmed). Use `expect("UnitKeywordState must be present on all board entities")` in production paths where the component is guaranteed — `Ok(...)` with `else { return; }` is appropriate at board entry points where presence is not guaranteed.

**INJURED** — state re-evaluation called inline at each sub-step boundary:

```rust
// Called by resolve_combat at the SS3→SS4, SS5, and SS6 sub-step boundaries
keyword::state_eval::eval_injured_bonuses(world);
// Reads CurrentHp vs MaxHp per unit; updates INJURED-granted keyword bonuses
// in UnitKeywordState (e.g., FIRST STRIKE granted while INJURED)
```

### Part 6: START OF TURN — DRAFT Phase Dispatch

START OF TURN fires at DRAFT phase entry, outside `resolve_combat`. A normal Bevy system reads `MessageReader<DraftPhaseEntered>` (requires `app.add_message::<DraftPhaseEntered>()` — see Verification Required item 5) and dispatches per-unit via `commands.trigger_targets()`. Deferred dispatch is acceptable for DRAFT phase — no sub-step timing constraints apply outside RESOLUTION.

```rust
// server/feature/keyword/observers.rs

pub fn start_of_turn_dispatch_system(
    mut reader: MessageReader<DraftPhaseEntered>,
    units: Query<(Entity, &UnitKeywordState)>,
    mut commands: Commands,
) {
    for _event in reader.read() {
        for (entity, kw_state) in units.iter() {
            if kw_state.has_keyword(SimpleKeyword::StartOfTurn) {
                // Deferred: fires when Commands flush (apply_deferred after this system set)
                commands.trigger_targets(StartOfTurnTriggered, entity);
            }
        }
    }
}
```

### Part 7: Module Extensions to server/feature/keyword/

ADR-018 defined the module tree. ADR-021 adds three files and extends two:

```
server/feature/keyword/
  mod.rs            ← Extended: registers 5 observers, ChainDeathBuffer resource,
                       start_of_turn_dispatch_system
  components.rs     ← UnitKeywordState (ADR-018, unchanged)
  events.rs         ← NEW: UnitAppeared, UnitDied, FinalBlowDealt,
                       StartOfTurnTriggered, EndOfTurnTriggered
  observers.rs      ← NEW: on_unit_appeared, on_unit_died, on_final_blow_dealt,
                       on_start_of_turn, on_end_of_turn,
                       start_of_turn_dispatch_system
  resources.rs      ← NEW: ChainDeathBuffer
  effects.rs        ← keyword effect functions (ADR-018, extended)
  state_eval.rs     ← eval_injured_bonuses (NEW), eval_outnumbered_system,
                       leader_snapshot_system, bodyguard_cleanup_system (ADR-018)
  movement.rs       ← repel_destination, attract_destination (ADR-018, unchanged)
```

### Architecture Diagram

```
KeywordPlugin::build()
  app.observe(on_unit_appeared)     ← global; guard: has_keyword(Appearance)
  app.observe(on_unit_died)         ← global; guard: has_keyword(Death)
  app.observe(on_final_blow_dealt)  ← global; guard: has_keyword(FinalBlow)
  app.observe(on_start_of_turn)     ← global; guard: has_keyword(StartOfTurn)
  app.observe(on_end_of_turn)       ← global; guard: has_keyword(EndOfTurn)
  app.add_systems(Update, start_of_turn_dispatch_system)
  app.init_resource::<ChainDeathBuffer>()

resolve_combat (fn resolve_combat(world: &mut World) — exclusive):
  SS1: world.trigger_targets(UnitAppeared { sub_step: 1 }, entity)  [per spawn commit]
  SS3: world.trigger_targets(FinalBlowDealt { killed, sub_step: 3 }, attacker)
       check_and_apply_counterattack(world, defender, attacker, 3)  [inline, proximity-gated]
       eval_injured_bonuses(world)  [inline, SS3→SS4 boundary]
  SS4: ChainDeathBuffer seeded with lane-ordered initial deaths
       loop { world.trigger_targets(UnitDied { attacker }, entity) }
         on_unit_died → guard check → effects::apply_death_trigger()
                      → may push (chain_entity, attacker) to ChainDeathBuffer
  SS5: eval_injured_bonuses(world)  [inline, SS5 boundary]
  SS6: world.trigger_targets(FinalBlowDealt { killed, sub_step: 6 }, attacker)
       check_and_apply_counterattack(world, defender, attacker, 6)  [inline]
       eval_injured_bonuses(world)  [inline, SS6 boundary]
       for alive_unit: world.trigger_targets(EndOfTurnTriggered, alive_unit)

DRAFT phase (normal Bevy system):
  start_of_turn_dispatch_system reads MessageReader<DraftPhaseEntered>
  → commands.trigger_targets(StartOfTurnTriggered, entity) per eligible unit
  → on_start_of_turn fires when Commands flush
```

### Key Interfaces

```rust
// All calls within fn resolve_combat(world: &mut World) — exact API pending
// Verification Required item 1 (world.trigger_targets vs world.commands().trigger_targets + flush):

world.trigger_targets(UnitAppeared { sub_step: 1 }, entity);          // SS1
world.trigger_targets(FinalBlowDealt { killed, sub_step }, attacker); // SS3/SS6 kill
world.trigger_targets(UnitDied { attacker }, entity);                 // SS4 chain loop
world.trigger_targets(EndOfTurnTriggered, alive_unit);                // end of SS6

// Inline (no Observer):
check_and_apply_counterattack(world, defender, attacker, sub_step);   // SS3/SS6
keyword::state_eval::eval_injured_bonuses(world);                     // sub-step boundaries

// DRAFT phase (deferred — normal system):
commands.trigger_targets(StartOfTurnTriggered, entity);
```

## Alternatives Considered

### Alternative A: Direct Inline Dispatch Throughout (no Observers)

- **Description:** `resolve_combat` reads `UnitKeywordState` directly at each decision point and calls `keyword::effects::*` functions inline. No Bevy Observers — pattern matches the class-effect-dispatch from ADR-014.
- **Pros:** Maximum explicitness; no framework indirection; identical pattern to class effects; zero Observer registration overhead.
- **Cons:** Combat resolution module must enumerate every timing trigger keyword at every sub-step — couples `combat/sub_steps.rs` to keyword type knowledge. The engine reference explicitly recommends Observers for APPEARANCE, DEATH, and FINAL BLOW. Entity-scoped reactive events (unit entered, unit died) are what Observers are designed for.
- **Rejection Reason:** Coupling the combat module to keyword enumeration violates the "CR owns when; Keyword owns what" GDD boundary. Observer pattern is the engine-recommended approach and is fully compatible with the exclusive system model.

### Alternative B: Entity-Scoped Observers Registered at Spawn

- **Description:** Each unit spawned in SS1 gets `world.entity_mut(entity).observe(handler)` called for each timing trigger keyword it has. Observers auto-clean when entity despawns.
- **Pros:** Observers only fire for units that have the keyword — guard check not needed. Automatic cleanup on despawn.
- **Cons:** SS1 must scan all keywords at spawn time and register multiple observers per unit (up to 7 per entity if all timing triggers present). Global observers with a guard check achieve identical behavior with simpler spawn logic and a single registration point at `KeywordPlugin::build()`.
- **Rejection Reason:** Global observers with the guard pattern are simpler and equally correct. Spawn-time registration adds complexity to SS1 for no meaningful benefit.

## Consequences

### Positive

- Combat resolution fires trigger events; keyword observers own the effects — GDD ownership boundary enforced at the code level
- All timing trigger wiring is in `KeywordPlugin::build()` — single location to audit all hooks
- DEATH chain is bounded, sequential, and explicit — no recursion risk, no hidden depth
- COUNTERATTACK and INJURED retain inline dispatch — preserves proximity-check and state-scan semantics without artificial Observer wrapping
- `start_of_turn_dispatch_system` is a normal Bevy system — compatible with standard scheduling

### Negative

- Guard pattern is mandatory boilerplate in every observer handler — if omitted, effects silently fire for units without the keyword
- `ChainDeathBuffer` must be cleared at SS4 start — failing to clear leaves stale deaths from a previous round
- `commands.trigger_targets()` for START OF TURN is deferred — if Commands are not flushed before the handler's effects are needed, a one-frame gap exists

### Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Guard check omitted in a new observer handler | MEDIUM | Effects over-fire for units without the keyword (silent) | Code review checklist: every observer handler must have guard as first operation |
| `ChainDeathBuffer` not cleared before SS4 starts | LOW | Stale deaths pollute current round | `execute_ss4()` clears at entry before `extend()`; integration test asserts buffer is empty at RESOLUTION end |
| `world.trigger_targets()` does not exist in Bevy 0.18 (requires `commands + flush`) | LOW | Compile error blocking all keyword implementation | RESOLVED 2026-04-30 — `world.trigger_targets()` confirmed as valid `World` method; synchronous exclusive-system path is correct |
| Observer handler param type: `On<T>` vs `Trigger<T>` | LOW | Compile error | RESOLVED 2026-05-01 — `On<T>` confirmed correct per `breaking-changes.md` line 140 (Bevy 0.17 section). `Trigger<T>` is the pre-0.17 form. All handler signatures and reference docs updated. |
| `ResMut<T>` or `MessageWriter<T>` not usable inside Observer handler from exclusive system | LOW | Compile error or silent message loss | RESOLVED architecturally 2026-04-30 — standard system params work in observers; sequential borrow pattern in drain loop is safe. Smoke test required during first keyword story: fire `UnitDied`, assert `KeywordTriggered` emitted |
| `commands.trigger_targets()` flush timing leaves a gap before START OF TURN effects read | LOW | State mismatch on START OF TURN effects | Schedule `apply_deferred` after `start_of_turn_dispatch_system` in system set; verify with integration test |

## GDD Requirements Addressed

| GDD System | Requirement | How This ADR Addresses It |
|---|---|---|
| `keyword-system.md` KW-001 | APPEARANCE fires before DEATH chains from APPEARANCE-caused kills | `UnitAppeared` observer fires per-entity in SS1; the SS4 DEATH chain loop begins after all SS1 observers complete |
| `keyword-system.md` KW-002 | Simultaneous DEATH triggers fire in lane order (Lane 1 before Lane 5) | `ChainDeathBuffer` seeded with `collect_dead_units_lane_ordered()` — insertion order preserved in `VecDeque` |
| `keyword-system.md` KW-003 | DEATH chains are sequential: A resolves completely before B fires | `ChainDeathBuffer` drain loop pops one entity at a time; `on_unit_died` completes fully before next `pop_front()` |
| `keyword-system.md` KW-004a/b | FINAL BLOW fires in the sub-step of the kill, not SS4 | `FinalBlowDealt { sub_step }` fired on attacker in SS3 or SS6 at the killing blow; not deferred |
| `keyword-system.md` KW-005 | COUNTERATTACK does NOT fire for RANGE attackers | `check_counterattack_proximity()` requires same-cell or collision-halted adjacent contact — RANGE attackers that did not advance are excluded |
| `keyword-system.md` KW-007 | INJURED bonus not active during the sub-step damage was received | `eval_injured_bonuses()` called at SS3→SS4 boundary, not inline during SS3 damage calculation |
| `keyword-system.md` KW-009a/b | START OF TURN fires at DRAFT entry after mana ramp + gold income; not on round of placement | `start_of_turn_dispatch_system` reads `DraftPhaseEntered` — RSM emits this only after RSM Rule 3 (mana ramp + gold income) executes |
| `keyword-system.md` KW-010a/b | END OF TURN fires after SS6, before round counter increments | `EndOfTurnTriggered` fired per alive unit inside `resolve_combat` after SS6, before `MessageWriter<ResolutionComplete>` is written |

## Performance Implications

- **CPU:** 5 global observers registered once at startup — zero per-frame cost outside RESOLUTION. During RESOLUTION: at most ~10 APPEARANCE + 10 DEATH + 10 FINAL BLOW + 10 END OF TURN `trigger_targets` calls per round (≤10 units). Each call is a synchronous function with a keyword guard check.
- **Memory:** `ChainDeathBuffer` — at most 9 entries of `(Entity, Option<Entity>)` ≈ 144 bytes peak; cleared between rounds.
- **Network:** No additional network traffic beyond ADR-018's `KeywordTriggered` messages, which observer handlers write to `MessageWriter<KeywordTriggered>`.

## Migration Plan

Greenfield — no keyword timing trigger implementation exists. Sequence:

1. Add `events.rs` with 5 event types; add `resources.rs` with `ChainDeathBuffer`
2. Extend `KeywordPlugin::build()` to register observers, `ChainDeathBuffer`, and `start_of_turn_dispatch_system`
3. Add `observers.rs` with 5 handler stubs (guard + `todo!()` bodies) and `start_of_turn_dispatch_system`
4. Resolve all 5 Verification Required items before any keyword story opens
5. Implement `eval_injured_bonuses()` in `state_eval.rs`
6. Implement `check_and_apply_counterattack()` and add to `effects.rs`
7. Fill in observer handler bodies one per keyword AC story (KW-001 through KW-010b)

## Validation Criteria

All timing trigger BLOCKING acceptance criteria in `keyword-system.md` must pass: KW-001 through KW-010b.

Pre-implementation gates:
- [x] Verification Required items 1–5 resolved (see Engine Compatibility table — resolved 2026-04-30)
- [ ] ADR-018 Accepted (provides `UnitKeywordState` component and module structure)
- [ ] `ChainDeathBuffer` is empty at RESOLUTION end confirmed by integration test
- [ ] Observer guard pattern enforced in code review — every handler must guard first

## Related Decisions

- ADR-017: Exclusive system execution model — `resolve_combat` is the sole caller of `world.trigger_targets()` during RESOLUTION
- ADR-018: `UnitKeywordState` component — observer handlers read this; all guards check keyword presence here
- ADR-010: RSM event bus — `DraftPhaseEntered` message triggers `start_of_turn_dispatch_system`
- ADR-014: class-effect-dispatch API decision — COUNTERATTACK/INJURED inline pattern is consistent with this stance; Observer pattern for entity-scoped triggers is compatible (not banned by the standalone-system forbidden pattern)
- `design/gdd/keyword-system.md` — all 7 timing triggers, KW-001 through KW-010b
- `design/gdd/combat-resolution.md` — sub-step execution context for all RESOLUTION observers
- `design/gdd/network-protocol.md` — OQ-NP3 references `ResolutionEvent::UnitDied` (protocol variant, distinct from the `UnitDied` Bevy Event defined here)
