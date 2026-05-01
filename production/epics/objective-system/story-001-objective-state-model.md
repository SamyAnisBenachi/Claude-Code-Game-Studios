# Story 001: Objective State Model

> **Epic**: Objective System
> **Status**: Complete
> **Layer**: Feature
> **Type**: Logic
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/objective-system.md`
**Requirements**:
- `TR-OBJ-001` — Each player has 5 objective slots (1 per lane); HP = `objective_hp` (default 5), AR = 0; real/fake identity immutable after DRAFT_INITIAL assignment
- `TR-OBJ-007` — `ObjectiveHp { hp: u32 }` is a replicated ECS component broadcast to both clients; `ObjectiveIdentity` is server-only in `HiddenObjectives` Resource and NEVER replicated (ADR-001)

**ADR Governing Implementation**: [ADR-001: Hidden Objective Identity via Targeted Unicast, Not Component Replication](docs/architecture/adr-001-objective-identity-unicast.md)

**ADR Decision Summary**: `ObjectiveHp { hp: u32 }` is a replicated ECS component — broadcast to both clients on every change. `ObjectiveIdentity { is_fake: bool }` is NEVER inserted into the replication graph; the server holds it in the non-replicated server-side `HiddenObjectives` resource. This split enforces privacy at the message-routing boundary, not via fragile per-component visibility flags.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**: Component replication is opt-in: entity must have `Replicate::default()` AND component registered via `app.register_component::<T>()` (checklist item 18, confirmed). Use Required Components pattern (`Sprite::from_image(..)` + `Transform`) — never `SpriteBundle`. `#[derive(Component)]` on `ObjectiveHp` is correct; do NOT `#[derive(Component)]` on any identity-carrying struct.

**Control Manifest Rules (Feature layer)**:
- Required: `ObjectiveIdentity { is_fake: bool }` held in server-only `HiddenObjectives` Resource, never replicated as an ECS component (ADR-001)
- Required: `ObjectiveHp { hp: u32 }` is a replicated ECS component, broadcast to both clients on every change (ADR-001)
- Forbidden: Never replicate `ObjectiveIdentity` as an ECS component; never use per-component Lightyear visibility workarounds (ADR-001)
- Forbidden: Never send opponent `is_fake` values in any broadcast message (ADR-001)

---

## Acceptance Criteria

*From GDD `design/gdd/objective-system.md`, scoped to this story:*

- [ ] **OS-1a (slot count)**: GIVEN a new game session starts and DRAFT_INITIAL fires, WHEN objective state is queried for any player, THEN that player has exactly 5 objective slots, one per lane.
- [ ] **OS-1b (initial stats)**: GIVEN objective slots are initialized, WHEN their visible state is queried, THEN each slot has `ObjectiveHp.hp == objective_hp` from `GameConfig` and no armor component is present (AR = 0).
- [ ] **OS-1c (server-only hidden state)**: GIVEN objective initialization completes, WHEN server resources are inspected, THEN `HiddenObjectives` and `ObjectiveCounters` exist, counters start at 0, and no objective identity component is inserted into replicated ECS state.

---

## Implementation Notes

*Derived from ADR-001 Implementation Guidelines:*

Define the following types in `server/feature/objective/`:

```rust
// ObjectiveHp: replicated ECS component — broadcast to both clients
#[derive(Component, Clone, Serialize, Deserialize, Debug)]
pub struct ObjectiveHp {
    pub hp: u32,
}

// ObjectiveSlot: server-side ECS component tracking slot state
#[derive(Component, Debug)]
pub struct ObjectiveSlot {
    pub lane: LaneId,
    pub player: PlayerId,
    pub destroyed: bool,
}

// HiddenObjectives: server-only Resource — never replicated, never broadcast
#[derive(Resource, Debug, Default)]
pub struct HiddenObjectives {
    pub identities: HashMap<(PlayerId, LaneId), bool>,  // true = fake
}

// ObjectiveCounters: server-only Resource — read by RSM for GAME_OVER check
#[derive(Resource, Debug, Default)]
pub struct ObjectiveCounters {
    pub real_destroyed: HashMap<PlayerId, u32>,
    pub fake_destroyed: HashMap<PlayerId, u32>,
}
```

Register `ObjectiveHp` for replication via `app.register_component::<ObjectiveHp>()` in the `ObjectivePlugin`. Add `Replicate::default()` to each spawned objective entity.

At session start (SessionReady observer or DraftStarted subscriber), spawn 5 objective entities per player, each with `ObjectiveHp { hp: config.objective_hp }`, `ObjectiveSlot { lane, player, destroyed: false }`, and `Replicate::default()`. Insert `HiddenObjectives::default()` and `ObjectiveCounters::default()` as Resources.

`HiddenObjectives` is wiped and re-populated at each new session — never carry it across sessions.

The `HiddenObjectives` Resource is populated by Story 002 (fake lane assignment). This story only establishes the data structures and initial HP values.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- [Story 002]: Fake lane assignment (populating `HiddenObjectives` with real/fake identity)
- [Story 003]: Sending `S2CObjectiveIdentities` unicast to players
- [Story 004]: The `take_damage()` interface and HP reduction logic
- [Story 005]: Destruction consequence path

---

## QA Test Cases

*Written by qa-lead at story creation. Implement against these — do not invent new test cases.*

- **OS-1**: 5 objective slots initialized per player with correct HP
  - Given: `World::new()` with `GameConfig { objective_hp: 5, fake_count: 2, .. }` and 2 players
  - When: Session initialization runs and DRAFT_INITIAL fires
  - Then: Each player has exactly 5 `ObjectiveSlot` entities; each `ObjectiveHp.hp == 5`; `HiddenObjectives` resource exists; `ObjectiveCounters` resource exists with all counts at 0
  - Edge cases: `objective_hp = 3` → all HP initialize to 3; `objective_hp = 8` → all HP initialize to 8

- **OS-1 variant — AR = 0**: Objectives have no armor value
  - Given: Any initialized objective entity
  - When: Armor is queried
  - Then: No AR component present (AR = 0 means no separate AR tracking — HP is the only defense stat)

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/objective/objective_state_test.rs` — must exist and pass

**Status**: [x] Created and passing (`cargo test -p server --test objective_state_test`)

---

## Dependencies

- Depends on: None — this is the foundation story for the Objective System
- Unlocks: Story 002 (fake assignment needs data structures), Story 004 (damage interface needs ObjectiveHp + HiddenObjectives)

## Completion Notes

**Completed**: 2026-05-01
**Verdict**: COMPLETE WITH NOTES
**Criteria**: 3/3 passing
**Test Evidence**: Logic unit test at `tests/unit/objective/objective_state_test.rs`; local `cargo test -p server --test objective_state_test` passed 4/4. `cargo check -p server` passed.
**Deviations**: Advisory only: `ObjectiveCounters` lives in `server/src/core/objective_contract.rs` and is re-exported by the objective feature so RSM can read counters without importing Feature-layer modules. Advisory only: Objective System GDD Rule 4 still contains older replicated-identity wording; current TR-OBJ-007 and ADR-001 are followed by the implementation.
**Code Review**: Skipped by lean review mode.
**Scope**: Implementation touched the objective feature module, the RSM-safe core counter contract, server plugin wiring, Cargo test registration, and unit test evidence. `production/sprint-status.yaml` was left unchanged because it has no entry for this story.
