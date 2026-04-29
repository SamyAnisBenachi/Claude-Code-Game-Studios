# ADR-012: SessionReady Delivery — Observer (Same-Frame) vs Buffered Events

## Status

Accepted

## Date

2026-04-29

## Last Verified

2026-04-29

## Decision Makers

User + lead-programmer (ordering guarantee analysis), technical-director
(authority model and ECS scheduling validation)

## Summary

`SessionReady` is delivered via a Bevy `trigger` / `Observer` (same-frame,
synchronous), not a buffered `Events<T>` (next-frame). This guarantees that
`SessionConfig` and `ServerRng` — which the GSS inserts immediately before
triggering — are present in the ECS world when the RSM's `SessionReady`
observer runs in the same `Update` tick. The alternative (buffered Events)
introduces a one-frame gap during which the RSM would attempt to read
`SessionConfig` before it exists, producing a fatal resource-not-found panic.

## ⚠️ API Verification Required (Bevy 0.18 Observer Pattern)

This ADR uses Bevy 0.17+ Observer semantics. Key verification items:
- `Commands::trigger(SessionReady)` fires Observer in the **same** `Update` tick
- Resource inserted via `Commands::insert_resource()` before `Commands::trigger()`
  is visible to the Observer handler (command queue ordering)
- `Trigger<SessionReady>` is the correct Observer handler parameter in Bevy 0.18
  (skill shows `On<E>` as the trigger type — confirm which is correct for 0.18)
- `app.observe(on_session_ready)` is the correct registration API

Note: `SessionReady` uses `#[derive(Event)]` (not `#[derive(Message)]`) because
it is a one-shot Observer trigger, NOT a recurring buffered message. This is the
correct Bevy 0.17/0.18 pattern for lifecycle events.

---

## Engine Compatibility

| Field | Value |
|-------|-------|
| **Engine** | Bevy 0.18 |
| **Domain** | Core / ECS |
| **Knowledge Risk** | HIGH — Bevy 0.17 formalized the Event/Observer split; 0.18 is post-cutoff. Observer semantics (immediate trigger, same-frame delivery), `trigger()` call site, and `ObserverTrigger` / `Trigger<E>` / `On<E>` handler signature must be verified against 0.18 docs before implementation. |
| **References Consulted** | `docs/engine-reference/bevy/VERSION.md`, `design/gdd/game-session-system.md` (Rule 11, OQ-C note), `design/gdd/round-state-machine.md` (Rule 1 LOBBY exit guard), `docs/architecture/architecture.md` (OQ-C) |
| **Post-Cutoff APIs Used** | Bevy 0.17+ `World::trigger()` / `Commands::trigger()` for Observer dispatch; `Observer` system registration; `Trigger<E>` parameter in observer handler systems. The `Event`/`Observer` split was formalized in 0.17 — both `#[derive(Event)]` and `#[derive(Event)]` + trigger coexist; verify the trigger path does not require a separate derive or registration flag in 0.18. |
| **Verification Required** | (1) Confirm `Commands::trigger(SessionReady)` runs the registered observer in the **same** `Update` tick (not deferred to next frame). (2) Confirm a resource inserted via `Commands::insert_resource` in the same system that calls `Commands::trigger` is visible to the triggered observer (i.e., Commands flush order: inserts before triggers, or triggers after all commands). (3) Confirm the observer handler receives a `Trigger<SessionReady>` parameter and can access `Res<SessionConfig>` and `Res<ServerRng>` from world. (4) Confirm `World::trigger()` vs `Commands::trigger()` — which is appropriate for a regular system (not exclusive system)? |

> **Note**: Knowledge Risk is HIGH. If Bevy upgrades to 0.19 or beyond, re-validate
> Observer trigger semantics and Commands flush ordering. Flag this ADR as Superseded
> and author a new one if the ordering guarantees change.

## ADR Dependencies

| Field | Value |
|-------|-------|
| **Depends On** | ADR-009 (RSM Phase State — establishes that the RSM transitions to DRAFT_INITIAL on `SessionReady`; this ADR determines when that observer fires relative to resource insertion) |
| **Enables** | Any story implementing the `SessionReady` trigger path, the RSM's LOBBY→DRAFT_INITIAL transition, or reading `Res<SessionConfig>` / `Res<ServerRng>` at session start |
| **Blocks** | GSS implementation story (session readiness path); RSM LOBBY→DRAFT_INITIAL story |
| **Ordering Note** | ADR-009 (RSM phase state) must be Accepted before this ADR is implemented, as it defines what the RSM observer does with `SessionReady`. ADR-005 (server-side RNG) must also be Accepted — it establishes when `ServerRng` is initialized (at session start, owned by GSS). |

## Context

### Problem Statement

The Game Session System GDD (Rule 11) specifies that when `SessionReady` fires,
`SessionConfig` and `ServerRng` must both be available to all systems that handle
the event — specifically the RSM, which immediately transitions to DRAFT_INITIAL
and begins reading `SessionConfig` for player-count and class-map data.

Bevy 0.18 offers two delivery mechanisms for inter-system events:

- **Buffered `Events<T>`** (`#[derive(Event)]`, written via `EventWriter`, read
  via `EventReader`): events written in frame N are readable in frame N+1. This
  is the standard Bevy event pattern for game-loop messages.
- **Observers** (`trigger()`, `Observer` system): the observer handler runs
  synchronously during the same `Update` tick as the trigger call, within the
  same command-flush cycle.

The GSS must insert `SessionConfig` and `ServerRng` as ECS resources before any
system can read them. If `SessionReady` is delivered via buffered Events, the RSM
reads the event in frame N+1 — but `SessionConfig` was inserted in frame N. This
is safe if command application happens before the next frame's systems run, which
is the normal Bevy `Update` flush behavior. However, if the RSM runs in the same
system set as the GSS (before the command flush at the set boundary), it could
attempt `Res<SessionConfig>` before the resource exists.

Beyond resource availability, the GDD's "same system" write sequencing guarantee
for `SessionSlot.class` and `class_selections` (Rule 7) and the `SessionConfig`
build invariant (Rule 11 panic guard) both require that the RSM cannot begin
DRAFT_INITIAL until `SessionConfig` is provably populated — not just scheduled to
be populated.

### Current State

No implementation exists. This is a greenfield decision. The GSS, RSM, `SessionConfig`,
and `ServerRng` systems are all unimplemented. This ADR must be resolved before
any story implementing the session readiness path can begin.

### Constraints

- **Same-tick resource visibility requirement**: The RSM observer for `SessionReady`
  must be able to call `Res<SessionConfig>` and `Res<ServerRng>` without a
  resource-not-found panic. The only safe guarantee is same-frame insertion before
  the observer fires.
- **Bevy 0.17+ Event/Observer split**: Prior to 0.17, all cross-system signals
  used `Events<T>`. From 0.17 onward, Observers are the idiomatic reactive pattern
  for one-shot, low-frequency events; buffered `Events<T>` are for high-frequency
  game-loop messages. `SessionReady` fires exactly once per game session — it is
  a one-shot reactive trigger, not a game-loop message.
- **No game-loop frequency**: `SessionReady` fires once per session. Observer
  overhead (which is higher than buffered events for high-frequency paths) is not
  a concern here.
- **RSM zero-import constraint** (architecture.md crate dependency rules): The RSM
  must not import from feature/ or other core/ modules via direct function calls.
  It may subscribe to events/observers from other core modules. `SessionReady`
  crossing the GSS→RSM boundary via Observer is consistent with the "emit, don't
  call" principle.
- **Hackathon timeline**: The session start path is on the critical path for M1.
  The chosen mechanism must be implementable without exotic Bevy patterns.

### Requirements

- TR-GSS-01: `SessionConfig` is present in the ECS world when the RSM observer
  for `SessionReady` executes.
- TR-GSS-02: `ServerRng` is present in the ECS world when the RSM observer for
  `SessionReady` executes.
- TR-GSS-03: `SessionReady` fires exactly once per session. The delivery mechanism
  must not allow duplicate delivery.
- TR-GSS-04: The RSM's LOBBY→DRAFT_INITIAL transition executes in the same `Update`
  tick as `SessionReady` triggers — not one frame later.
- TR-GSS-05: The Objective System's fake-objective assignment (which reads
  `Res<SessionConfig>` and `Res<ServerRng>` at DRAFT_INITIAL entry) may run in
  the same frame as DRAFT_INITIAL begins or in the immediately following frame,
  provided both resources are guaranteed present.

## Decision

**`SessionReady` is delivered as a Bevy `Observer` trigger, not a buffered `Events<T>`.**

The GSS system that evaluates the F4 predicate (`is_ready`) and detects `SessionReady`
executes the following steps in strict order within a single exclusive system or
via Commands:

1. Build `SessionConfig` from the finalized `SessionSlot` vector. Panic if any
   occupied slot has `class = None` (GSS Rule 11 invariant).
2. Initialize `ServerRng` from `OsRng` seed (per ADR-005).
3. Insert `SessionConfig` via `Commands::insert_resource(SessionConfig { ... })`.
4. Insert `ServerRng` via `Commands::insert_resource(ServerRng::new(seed))`.
5. Trigger `SessionReady` via `Commands::trigger(SessionReady)`.

Bevy's command queue processes inserts and triggers in the order they are issued.
The observer registered for `SessionReady` fires after the preceding inserts are
applied. The RSM observer handler (`fn on_session_ready(trigger: Trigger<SessionReady>, config: Res<SessionConfig>, rng: Res<ServerRng>)`) can therefore access both
resources without a resource-not-found panic.

**`SessionReady` is a marker struct, not a data carrier.** All session data is
accessed via `Res<SessionConfig>` from the world — the trigger payload is empty.
This is the idiomatic Bevy Observer pattern for reactive lifecycle events.

**Only one Observer is registered for `SessionReady`**: the RSM's LOBBY→DRAFT_INITIAL
handler. No other system may independently observe `SessionReady` — downstream
systems (Objective System, Card Pool) react to `DRAFT_INITIAL` phase entry via
the RSM's phase Messages (`DraftStarted`), not directly to `SessionReady`. This
preserves the RSM-as-sole-orchestrator invariant.

### Architecture

```
GSS system (runs in Update — single-tick, F4 evaluated to true)
│
│  1. Build SessionConfig from SessionSlot vec
│     - Panic if any occupied slot.class == None
│  2. Seed ServerRng from OsRng
│
│  Commands queue (applied before observer fires):
│  3. insert_resource(SessionConfig)
│  4. insert_resource(ServerRng)
│  5. trigger(SessionReady)       ←── Observer fires AFTER inserts applied
│                                      (same Update tick, same command flush)
│
▼
RSM Observer: fn on_session_ready(
    trigger: Trigger<SessionReady>,
    config: Res<SessionConfig>,       ← present: inserted in step 3
    rng:    Res<ServerRng>,           ← present: inserted in step 4
    mut phase: ResMut<RoundPhase>,    ← RSM-owned phase state
    /* Lightyear message sender */
)
│
│  RSM writes:  RoundPhase → DRAFT_INITIAL
│               round_number = 1
│               broadcasts S2CPhaseChanged(DRAFT_INITIAL, round=1)
│
▼
Next system set (same or next frame — guaranteed after RSM observer):
  Objective System (reads Res<SessionConfig>, Res<ServerRng>)
  → assign_fake_objectives
  → S2CObjectiveIdentities (unicast per player, ADR-001)

  Card Pool (reads Res<SessionConfig>, Res<ServerRng>)
  → draw_initial_draft_offerings
  → S2CDraftOffering (unicast per player)
```

### Key Interfaces

```rust
// server/core/session/events.rs

/// Fired once per game session when all LOBBY conditions are satisfied.
/// Payload is intentionally empty — all session data is in Res<SessionConfig>
/// and Res<ServerRng>, which are inserted immediately before this trigger fires.
///
/// DELIVERY: Observer trigger (same-frame). NOT a buffered Event.
/// SUBSCRIBERS: RSM only (one registered Observer). Other systems react to
///   DraftStarted Message emitted by the RSM after it handles this trigger.
#[derive(Event)]
pub struct SessionReady;

// server/core/session/system.rs

/// Evaluates the F4 session ready predicate every tick.
/// Runs in SystemSet::LobbyEval (before RSM phase systems).
/// Called only while GSS state == LOBBY_WAITING.
pub fn evaluate_session_ready(
    slots: Res<SessionSlots>,
    class_selections: Res<ClassSelections>,
    time: Res<Time>,
    lobby_state: Res<LobbyState>,
    mut commands: Commands,
    config: Res<GameConfig>,
) {
    if !is_ready(&slots, &class_selections, time.elapsed_secs_f64(),
                 lobby_state.deadline) {
        return;
    }
    // Build SessionConfig — panic on invariant violation (see Rule 11)
    let session_config = build_session_config(&slots, &class_selections);
    let server_rng = ServerRng::new_from_os();

    commands.insert_resource(session_config);
    commands.insert_resource(server_rng);
    commands.trigger(SessionReady);
    // GSS transitions to GAME_ACTIVE state — no further F4 evaluation
    commands.insert_resource(LobbyState::GameActive);
}

// server/core/rsm/system.rs

/// Registered as the sole Observer for SessionReady.
/// Runs in the same Update tick as evaluate_session_ready,
/// after Commands are applied (inserts precede trigger).
pub fn on_session_ready(
    _trigger: Trigger<SessionReady>,
    config: Res<SessionConfig>,     // guaranteed present
    _rng: Res<ServerRng>,           // guaranteed present
    mut phase: ResMut<RoundPhase>,
    mut round: ResMut<RoundNumber>,
    sender: /* Lightyear MessageSender */,
) {
    *phase = RoundPhase::DraftInitial;
    round.0 = 1;
    // broadcast S2CPhaseChanged to all connected players
    sender.broadcast(S2CPhaseChanged { phase: RoundPhase::DraftInitial,
                                       round: 1,
                                       timer_remaining_ms: config.draft_initial_timer_ms });
}

// Plugin registration (server/core/session/plugin.rs)
app.observe(on_session_ready);
// NOT: app.add_event::<SessionReady>()
// NOT: app.add_systems(Update, rsm_read_session_ready)
```

### Implementation Guidelines

**Commands flush ordering — the critical invariant:**

Bevy applies Commands in the order they are issued within a single flush point.
`insert_resource` and `trigger` both go into the same Commands queue. The flush
happens at the next apply_deferred point or at the end of the system set.
Provided `insert_resource(SessionConfig)` is issued before `trigger(SessionReady)`
in the same `commands` call sequence, the resource is guaranteed to exist when
the observer handler runs.

Do NOT reorder these calls:

```rust
// CORRECT — resource insertion before trigger
commands.insert_resource(session_config);   // applied first
commands.insert_resource(server_rng);       // applied second
commands.trigger(SessionReady);             // observer fires after both inserts

// WRONG — trigger before insertion: observer fires before resource exists
commands.trigger(SessionReady);
commands.insert_resource(session_config);   // too late
```

**No `apply_deferred` call needed** (in most Bevy 0.18 setups): Commands are
applied automatically at set boundaries. If `evaluate_session_ready` and
`on_session_ready` are in different system sets with a set boundary between them,
the commands from `evaluate_session_ready` are applied before `on_session_ready`
runs. However, since Observer handlers run synchronously during the command flush
(not at a set boundary), this sequencing is provided by the Observer mechanism
itself — not by set ordering.

**Verify the flush model before implementing** (see Verification Required checklist).
If Bevy 0.18 does not guarantee same-flush observer dispatch for `Commands::trigger`,
fall back to the `World::trigger` exclusive system approach (see Alternative 2).

**`evaluate_session_ready` runs only while LOBBY_WAITING:**

Gate the system on a `LobbyState` resource or a Bevy `State<LobbyPhase>`. Once
`SessionReady` fires and `LobbyState` transitions to `GameActive`, this system
must not evaluate F4 again. This prevents spurious re-triggers if any edge case
leaves `is_ready` logically true on subsequent ticks.

**One observer, one subscriber:**

Register exactly one Observer for `SessionReady` — the RSM's `on_session_ready`
handler. If another system (e.g., an analytics hook) needs to react to session
start, it must subscribe to `DraftStarted` (which the RSM emits immediately after
handling `SessionReady`), not to `SessionReady` itself. Multiple observers on a
single trigger complicate ordering and reduce traceability of the session start
sequence.

**Exclusive system fallback (if Commands::trigger ordering cannot be verified):**

If Verification Required item (2) cannot be confirmed — i.e., if `Commands::trigger`
does not guarantee inserts before observer dispatch — replace `evaluate_session_ready`
with an exclusive system using `World` directly:

```rust
pub fn evaluate_session_ready_exclusive(world: &mut World) {
    // Check F4 predicate using world.resource::<T>()
    // ...
    // Insert resources directly into world (immediately visible)
    world.insert_resource(session_config);
    world.insert_resource(server_rng);
    // Trigger observer — resource is already in world
    world.trigger(SessionReady);
}
```

This is the safe fallback: `World::insert_resource` is immediately visible to any
subsequent `World::trigger` observer call within the same exclusive system. The
trade-off is that exclusive systems cannot run in parallel with other systems in
the same frame — acceptable here because session start is a rare, one-time event.

## Alternatives Considered

### Alternative 1: Buffered `Events<T>` (next-frame delivery)

- **Description**: `SessionReady` is declared with `#[derive(Event)]` and sent via
  `EventWriter<SessionReady>`. The RSM reads it via `EventReader<SessionReady>` in
  the following `Update` tick. `SessionConfig` and `ServerRng` are inserted in the
  same tick as the event write — they are available in the world by the next tick
  when the RSM reads the event.
- **Pros**: Standard, well-understood Bevy pattern. No Observer semantics to verify.
  Safe against double-read — `EventReader` drains the queue on read.
- **Cons**: One-frame delay between `SessionReady` firing and DRAFT_INITIAL beginning.
  During that frame, the world is in a half-initialized state: `SessionConfig` exists
  but `RoundPhase` is still LOBBY. Any system that checks phase and config in the
  same frame (e.g., an integration test asserting post-ready invariants) would see
  an inconsistent state. More importantly, the GDD (Rule 11) states the behavior
  requirement as "SessionConfig and ServerRng are available to all systems that
  handle SessionReady" — a one-frame gap is technically correct if no system reads
  them before the next frame, but it requires all downstream systems to be careful
  about the LOBBY→DRAFT_INITIAL frame boundary. This is a subtle ordering constraint
  that is easy to violate and hard to detect until a race condition manifests.
- **Rejection Reason**: The one-frame delay introduces a window of inconsistent ECS
  world state that must be defended by convention rather than by engine guarantee.
  Observer triggers eliminate the window entirely. Given that `SessionReady` fires
  once per session and is not on a hot path, the Observer approach adds no meaningful
  overhead and provides stronger ordering guarantees. Rejected in favor of Observer.

### Alternative 2: Exclusive System with `World::trigger`

- **Description**: The `evaluate_session_ready` function is written as an exclusive
  system (`fn(world: &mut World)`). It inserts resources directly into the world
  via `world.insert_resource()` (immediately visible, no command queue) and then
  calls `world.trigger(SessionReady)`. The observer runs synchronously within the
  exclusive system call.
- **Pros**: Strongest possible ordering guarantee — resource insertion and observer
  dispatch happen in a single synchronous call chain, no command queue involved.
  Immune to any ambiguity about when commands are applied.
- **Cons**: Exclusive systems block all other systems in the same `Update` schedule
  for their duration. `evaluate_session_ready` runs every tick while in LOBBY_WAITING
  and spends most ticks doing nothing (F4 is false). This is a consistent but
  unnecessary stall. The session-start tick stall is negligible (resource insertion
  + observer dispatch is O(1)), but the per-tick stall during LOBBY is wasteful.
- **Estimated Effort**: Equal — the exclusive system variant is the same code with
  `World` instead of `Commands`.
- **Role**: This is the designated **fallback** if Verification Required item (2)
  cannot be confirmed for `Commands::trigger`. It is not rejected — it is held in
  reserve.
- **Rejection Reason (as primary approach)**: Unnecessary per-tick exclusive lock
  during LOBBY while F4 is false. Prefer `Commands::trigger` with verification.
  If verification fails, promote this alternative.

### Alternative 3: System Ordering (`.before()`) with Buffered Events

- **Description**: Use buffered `Events<T>` but enforce that the RSM's reader system
  runs in the same frame as the writer by putting both in the same system set with
  explicit `.before()` ordering: GSS writer runs first, RSM reader runs second.
  A manual `apply_deferred` call between them ensures commands (resource inserts)
  are flushed before the RSM reads the event.
- **Pros**: No Observer semantics. Same-frame delivery without explicit Observer API.
- **Cons**: Requires a manual `apply_deferred` insertion in the system set schedule.
  `apply_deferred` between two systems in the same set is legal but unusual and
  easy to break (removing it, reordering systems, or adding a new set boundary can
  silently break the ordering guarantee). The EventReader reads the event on the
  same tick it was written, which requires the event double-buffer to not rotate
  before the reader runs — this is normally safe within a single frame but is an
  implementation detail that depends on event buffer behavior. This approach is
  more fragile than the Observer pattern for a one-shot event.
- **Rejection Reason**: Fragile. Requires manual `apply_deferred` that is easy to
  accidentally remove. Observer triggers are the idiomatic Bevy 0.17+ pattern for
  exactly this use case. Rejected.

## Consequences

### Positive

- `SessionConfig` and `ServerRng` are provably present when `on_session_ready`
  executes — not "probably present" by convention.
- DRAFT_INITIAL begins in the same `Update` tick as `SessionReady` triggers.
  No half-initialized frame exists.
- The `SessionReady` → DRAFT_INITIAL path is self-documenting: a single
  `app.observe(on_session_ready)` registration makes the subscriber explicit
  and traceable.
- Consistent with the Bevy 0.17+ Event/Observer idiom: one-shot reactive
  lifecycle events use `trigger()`; high-frequency game-loop messages use
  `EventWriter`/`EventReader`.

### Negative

- Observer dispatch semantics in Bevy 0.18 must be verified before implementing
  (see Verification Required). If they differ from the assumed model, the exclusive
  system fallback must be adopted.
- Developers unfamiliar with Bevy Observers may mistakenly attempt to read
  `SessionReady` via `EventReader` — it will never fire for buffered readers.
  The doc comment on `SessionReady` must be explicit about this.
- Only one Observer is permitted on `SessionReady`. Any future system that wants
  to react to session start must subscribe to `DraftStarted` instead. This is a
  constraint on future development that must be documented and enforced in code review.

### Neutral

- `SessionReady` is a zero-sized marker struct (no payload). All data flows via
  ECS resources. This is consistent with the project's "resources carry state,
  events carry signals" convention.
- The exclusive system fallback (Alternative 2) is documented and ready to adopt
  without design changes — only the call site changes (`world.*` instead of
  `commands.*`).

## Risks

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|-----------|
| `Commands::trigger` does not guarantee inserts-before-observer in Bevy 0.18 | MEDIUM — post-cutoff API, unverified | Critical — `Res<SessionConfig>` panics in observer handler | Verification Required item (2). If unconfirmed, use exclusive system fallback (`World::trigger`) — identical behavior, guaranteed ordering. |
| Developer adds a second Observer for `SessionReady` (e.g., analytics) | MEDIUM — natural extension point | Medium — unpredictable observer execution order; race on resource reads | Document the "one observer" rule in the `SessionReady` struct doc comment. Code review gate: PR adding a second `app.observe(on_session_ready_*)` is a CHANGES REQUIRED. Route to `DraftStarted` instead. |
| `evaluate_session_ready` fires again after `SessionReady` has triggered | LOW — gated on `LobbyState` | Medium — double `SessionConfig` insert overwrites first; double `ServerRng` init loses seed 0 state | `LobbyState::GameActive` guard prevents re-evaluation. Integration test: assert `SessionReady` fires exactly once per session (trigger counter == 1). |
| Bevy 0.18 removes or renames `Observer` / `trigger()` API | LOW — API was stable at 0.17 formalization | High — requires rewriting delivery mechanism | `liv-bevy-018` skill review on any `.rs` file using Observer APIs. ADR supersession required on engine upgrade. |

## Performance Implications

| Metric | Before | Expected After | Budget |
|--------|--------|----------------|--------|
| CPU (session start tick) | N/A | < 0.1ms (resource insertion + observer dispatch — O(1), once per session) | Not in 16.67ms frame budget hot path — fires once per game session |
| CPU (LOBBY_WAITING ticks) | N/A | Negligible — F4 predicate is 3 boolean checks + 1 f64 comparison per tick | < 0.01ms per tick; not budget-relevant |
| Memory | N/A | `SessionConfig` ~200 bytes; `ServerRng` ~64 bytes | Negligible |
| Observer registration overhead | N/A | One `app.observe()` call at startup | Startup-only; zero runtime cost |

## Migration Plan

This is a greenfield decision — no existing `SessionReady` mechanism exists.

Implementation order:

1. Verify all Engine Compatibility checklist items (1–4) against
   Bevy 0.18 docs and release notes before writing any session-start code.
2. Add `SessionReady` marker struct to `server/core/session/events.rs`.
   Add `#[derive(Event)]` — required for both buffered and Observer use in Bevy.
3. Implement `evaluate_session_ready` in `server/core/session/system.rs` using
   `Commands::trigger`. If Verification Required item (2) cannot be confirmed,
   implement as an exclusive system using `World::trigger`.
4. Register the RSM observer: `app.observe(on_session_ready)` in the RSM plugin.
5. Implement `on_session_ready` in `server/core/rsm/system.rs`: set phase, set
   round number, broadcast `S2CPhaseChanged`.
6. Write a unit test (`tests/unit/session/session_ready_test.rs`):
   - Construct a `World` with slots filled and classes confirmed.
   - Call `evaluate_session_ready` (or trigger it via `World::run_system`).
   - Assert `Res<SessionConfig>` exists with correct values.
   - Assert `Res<ServerRng>` exists.
   - Assert `RoundPhase == DRAFT_INITIAL`.
   - Assert the observer fired exactly once.
7. Write an integration test verifying GSS-30 (from the GSS GDD acceptance
   criteria): `SessionConfig` and `ServerRng` are present when the RSM handles
   `SessionReady`.

**Rollback plan**: If `Commands::trigger` ordering cannot be verified and the
exclusive system fallback is adopted, no architectural changes are needed — only
the `evaluate_session_ready` implementation changes from `commands.*` calls to
`world.*` calls. The Observer registration and handler (`on_session_ready`)
remain unchanged.

## Validation Criteria

- [ ] **GSS-29**: `ServerRng` initialization failure before `SessionReady` triggers
  prevents `SessionReady` from firing. Session transitions to `LOBBY_CANCELLED`.
  `RoundPhase` remains LOBBY. (Unit test — inject a failing `ServerRng::new`.)
- [ ] **GSS-30**: After `SessionReady` fires, `Res<SessionConfig>` exists in the
  ECS world with `mode`, `player_count`, `team_map`, and `class_map` all correctly
  populated from `SessionSlot` data. `Res<ServerRng>` exists. `RoundPhase` is
  `DRAFT_INITIAL`. All three are observable in the same test tick as the trigger.
  (Unit test — assert all three post-trigger invariants in the same test.)
- [ ] **Single-fire invariant**: `SessionReady` fires exactly once per session.
  A second evaluation of F4 (after GSS enters `GAME_ACTIVE`) does not re-trigger.
  (Unit test — run `evaluate_session_ready` 3 ticks after trigger; assert trigger
  count == 1.)
- [ ] **Resource order invariant**: `Res<SessionConfig>` and `Res<ServerRng>` are
  accessible without panic inside `on_session_ready`. (Covered by GSS-30 test —
  if the test runs without panic, resources were present when the observer fired.)
- [ ] **RSM-1**: LOBBY transitions to DRAFT_INITIAL and broadcasts `S2CPhaseChanged`
  in the same tick as `SessionReady`. (Integration test — observe broadcast message
  timing relative to trigger tick.)
- [ ] **No-second-observer rule**: No system other than `on_session_ready` is
  registered as an Observer for `SessionReady`. (Code review gate — verify
  `app.observe` is called exactly once for `SessionReady` across all plugin
  registrations.)

## GDD Requirements Addressed

| GDD Document | System | Requirement | How This ADR Addresses It |
|-------------|--------|-------------|--------------------------|
| `design/gdd/game-session-system.md` | Game Session System | Rule 11 — "SessionConfig and ServerRng are available to all systems that handle SessionReady. The specific Bevy mechanism is documented in the session system ADR." | This ADR is the referenced session system ADR. Defines the Observer trigger pattern that guarantees resource availability in the same Update tick. |
| `design/gdd/game-session-system.md` | Game Session System | Rule 11 — "`SessionConfig` and `ServerRng` are available when the RSM handles `SessionReady`" | Observer trigger with insert-before-trigger ordering satisfies this requirement — no one-frame gap. |
| `design/gdd/game-session-system.md` | Game Session System | GSS-29 — `ServerRng` init failure must prevent `SessionReady` from firing | `evaluate_session_ready` checks `ServerRng::new` result before calling `commands.trigger(SessionReady)` — if init fails, the trigger is never issued. |
| `design/gdd/game-session-system.md` | Game Session System | GSS-30 — `SessionConfig` and `ServerRng` present when RSM handles `SessionReady` | Satisfied by the insert-before-trigger ordering in `evaluate_session_ready`. |
| `design/gdd/round-state-machine.md` | Round State Machine | Rule 1 — LOBBY→DRAFT_INITIAL requires `SessionReady` event from GSS | The RSM's `on_session_ready` observer is the LOBBY→DRAFT_INITIAL trigger. Same-frame delivery means the transition happens in the tick `SessionReady` fires. |
| `design/gdd/round-state-machine.md` | Round State Machine | RSM-1 — LOBBY transitions to DRAFT_INITIAL and broadcasts `S2CPhaseChanged` on `SessionReady` | Satisfied by `on_session_ready` setting `RoundPhase::DraftInitial` and broadcasting `S2CPhaseChanged`. |
| `docs/architecture/architecture.md` | Master Architecture | OQ-C — "`SessionReady` delivery: Observer (same-frame) vs buffered Events<T> (next-frame)" | This ADR resolves OQ-C: Observer (same-frame) is the chosen mechanism. |

## Related

- `docs/architecture/adr-009-rsm-phase-state.md` — RSM phase state definition.
  The `on_session_ready` observer transitions `RoundPhase` to `DRAFT_INITIAL` —
  the phase state type is defined in ADR-009.
- `docs/architecture/adr-010-rsm-event-bus.md` — RSM event bus (buffered Events).
  `DraftStarted` and all other phase messages from the RSM use buffered Events,
  not Observers. `SessionReady` is an exception to the buffered pattern precisely
  because it is a one-shot lifecycle trigger, not a recurring game-loop message.
  ADR-010 and this ADR together define the complete event communication model:
  Observers for one-shot lifecycle signals; buffered Events for game-loop messages.
- `docs/architecture/adr-005-server-side-rng.md` — RNG initialization contract.
  The GSS owns `ServerRng` lifecycle (init + destroy). This ADR specifies when
  `ServerRng` is inserted (immediately before `SessionReady` triggers).
- `design/gdd/game-session-system.md` — Source GDD. Rule 11 and the OQ-C note
  that explicitly deferred this mechanism decision to this ADR.
- `design/gdd/round-state-machine.md` — RSM Rule 1 (LOBBY exit guard) and RSM-1
  acceptance criterion both depend on this ADR being Accepted.
