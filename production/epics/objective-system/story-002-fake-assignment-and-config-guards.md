# Story 002: Fake Assignment & Config Guards

> **Epic**: Objective System
> **Status**: Ready
> **Layer**: Feature
> **Type**: Logic
> **Manifest Version**: 2026-04-29

## Context

**GDD**: `design/gdd/objective-system.md`
**Requirements**:
- `TR-OBJ-002` — Formula D5 fake assignment: 2 RNG seeds per player at DRAFT_INITIAL via remove-and-re-pick (no collision); ascending `player_id` ordering
- `TR-OBJ-009` — Session invariants: `fake_count >= 1`, `fake_count <= lane_count - loss_threshold` (max 3), `objective_hp >= 1`; asserted at DRAFT_INITIAL entry

**ADR Governing Implementation**: [ADR-001: Hidden Objective Identity via Targeted Unicast, Not Component Replication](docs/architecture/adr-001-objective-identity-unicast.md)

**ADR Decision Summary**: Fake lane assignment uses `ServerRng` at DRAFT_INITIAL with 2 seeds per player in ascending `player_id` order. The result is stored in the `HiddenObjectives` Resource (server-only, never replicated). Config invariants are asserted at session initialization — the session is refused with a fatal error if any invariant is violated. `HiddenObjectives` is wiped and re-populated at each new session.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**: `ServerRng` resource uses `ChaCha20Rng` from `rand_chacha 0.3`. RNG consumption order within DRAFT_INITIAL is strict: AssignFakeObjectives comes first (2 seeds/player, ascending `player_id`), then DrawInitialDraft. Do NOT use `rand::thread_rng()` or any other RNG source. Confirm `ResMut<ServerRng>` system parameter is available at the point of call.

**Control Manifest Rules (Feature layer)**:
- Required: `ObjectiveIdentity { is_fake: bool }` held in server-only `HiddenObjectives` Resource, never replicated (ADR-001)
- Required (Foundation/RNG): All game randomness through single `ServerRng` resource — never `thread_rng()`, never `StdRng`, never `SmallRng` (ADR-005)
- Required (Foundation/RNG): Consumption order at DRAFT_INITIAL: (1) AssignFakeObjectives — 2 seeds/player, ascending `player_id`; (2) DrawInitialDraft — per player, ascending `player_id` (ADR-005)
- Required (Foundation/Config): Abort startup if `fake_count` or `objective_hp` violate invariants; no fallback, no partial initialization (ADR-004)
- Forbidden: Never transmit RNG seeds to clients in any S2C message (ADR-005)
- Forbidden: Never use client-side RNG for gameplay (ADR-005)

---

## Acceptance Criteria

*From GDD `design/gdd/objective-system.md`, scoped to this story:*

- [ ] OS-2 (BLOCKING): GIVEN fake lane assignment runs at DRAFT_INITIAL using two RNG seeds per player, WHEN the two fake lane indices are recorded, THEN the two assigned fake lanes are always distinct — no player receives the same lane index as both fakes.
- [ ] OS-23a (BLOCKING): GIVEN a game session initialization attempt with `fake_count > lane_count - loss_threshold` (e.g., `fake_count = 4`, `lane_count = 5`, `loss_threshold = 2`), WHEN the server evaluates the upper-bound config invariant, THEN the session is refused with an error before LOBBY state is entered.
- [ ] OS-23b (BLOCKING): GIVEN a game session initialization attempt with `objective_hp = 0` OR `fake_count = 0` in GameConfig, WHEN the server evaluates the lower-bound config invariants, THEN the session is refused with an error before LOBBY state is entered. (Two separate assertions; test each independently.)
- [ ] OS-28 (BLOCKING): GIVEN a game session initializes with `fake_count = 1` in GameConfig, WHEN DRAFT_INITIAL runs, THEN exactly one fake lane is assigned per player, the remaining 4 lanes are real, all 5 objective slots initialize with HP = `objective_hp`, and `real_objectives_destroyed(player) = 0` for all players.

---

## Implementation Notes

*Derived from ADR-001 and ADR-005 Implementation Guidelines:*

Implement `assign_fake_objectives(rng: &mut ServerRng, players: &[PlayerId], fake_count: usize, hidden: &mut HiddenObjectives)`:

```rust
// D5 formula — runs for each player in ascending player_id order
fn assign_fakes_for_player(rng: &mut ServerRng, player: PlayerId, fake_count: usize, hidden: &mut HiddenObjectives) {
    let mut lanes: Vec<LaneId> = (1..=5).map(LaneId).collect();
    for _ in 0..fake_count {
        let idx = rng.next_seed() as usize % lanes.len();  // consume 1 seed
        let fake_lane = lanes.remove(idx);
        hidden.identities.insert((player, fake_lane), true);
    }
    // remaining lanes: real
    for lane in lanes {
        hidden.identities.insert((player, lane), false);
    }
}
```

Note: `fake_count` is typically 2 (default), giving C(5,2) = 10 equally likely pairs. The remove-and-re-pick approach ensures distinct fakes — this is the critical correctness property.

RNG consumption order within DRAFT_INITIAL (binding per ADR-005):
1. `assign_fake_objectives`: 2 seeds per player × N players (ascending `player_id`)
2. `draw_initial_draft`: next seeds per player (ascending `player_id`)

Config invariant assertions (run before any fake assignment):
```rust
assert!(config.fake_count >= 1, "fake_count = 0 is invalid: D5 would assign 0 fakes but still draw 2 seeds");
assert!(config.fake_count <= 3, "fake_count > lane_count - loss_threshold = 3: loss condition unreachable");
assert!(config.objective_hp >= 1, "objective_hp = 0: objectives would spawn destroyed");
```

If any assertion fails, do NOT emit `SessionReady`; transition to LOBBY_CANCELLED and broadcast `S2CSessionCancelled`. These are hard failures, not warnings.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- [Story 001]: Data structure definitions for `HiddenObjectives`, `ObjectiveHp`, `ObjectiveCounters`
- [Story 003]: Sending `S2CObjectiveIdentities` unicast (the delivery of the assignment result)
- [Story 004]: `take_damage()` interface

---

## QA Test Cases

*Written by qa-lead at story creation. Implement against these — do not invent new test cases.*

- **OS-2**: Fake lanes assigned are always distinct
  - Given: `World::new()` with `ServerRng` seeded from `ChaCha20Rng::seed_from_u64(ANY_SEED)`, `GameConfig { fake_count: 2, .. }`, 1 player
  - When: `assign_fake_objectives()` runs
  - Then: The two fake `LaneId` values stored in `HiddenObjectives` for this player are not equal
  - Edge cases: Run for 100 different seeds; assert distinctness holds for all. Also test with `fake_count = 1` (one fake, trivially distinct). Test with `fake_count = 3` (three distinct fakes from 5 lanes).

- **OS-2 variant — all 5 lanes assigned**: No lane left unassigned
  - Given: Same setup
  - When: Assignment completes
  - Then: `HiddenObjectives` contains exactly 5 entries for this player, with exactly `fake_count` entries set to `true` and `5 - fake_count` entries set to `false`

- **OS-23a**: `fake_count = 4` refused
  - Given: `GameConfig { fake_count: 4, lane_count: 5, .. }`
  - When: Config invariant check runs at session initialization
  - Then: Session initialization returns an error / panics with a diagnostic message; `HiddenObjectives` is not populated

- **OS-23b**: `fake_count = 0` refused
  - Given: `GameConfig { fake_count: 0, .. }`
  - When: Config invariant check runs
  - Then: Error before any assignment runs

- **OS-23b variant — `objective_hp = 0`**: Invalid HP refused
  - Given: `GameConfig { objective_hp: 0, .. }`
  - When: Config invariant check runs
  - Then: Error before any objective slot is initialized

- **OS-28**: `fake_count = 1` assigns exactly one fake
  - Given: `GameConfig { fake_count: 1, objective_hp: 5 }`, 2 players, seeded RNG
  - When: DRAFT_INITIAL runs
  - Then: Each player has exactly 1 fake lane and 4 real lanes in `HiddenObjectives`; `ObjectiveCounters.real_destroyed` = 0 for all players; `ObjectiveCounters.fake_destroyed` = 0 for all players

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/objective/fake_assignment_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 must be DONE (`HiddenObjectives` Resource must exist)
- Unlocks: Story 003 (identity unicast delivers the fake assignment result)
