# Story 004: Damage Interface

> **Epic**: Objective System
> **Status**: Complete
> **Layer**: Feature
> **Type**: Logic
> **Manifest Version**: 2026-04-29

## Context

**GDD**: `design/gdd/objective-system.md`
**Requirements**:
- `TR-OBJ-003` — Formula D1 damage: `HP_new = max(0, HP_current − amount)` via `saturating_sub`; objectives cannot be healed; destruction fires once; `amount=0` and already-destroyed are both no-ops
- `TR-OBJ-010` — `take_damage(lane, attacker_player, amount)` is the sole damage interface; Garde-Temps routes lethal damage (`amount = objective_hp`) through this same interface; `ObjectiveCounters` Resource read by RSM for GAME_OVER check without importing from `feature/objective/`

**ADR Governing Implementation**: [ADR-001: Hidden Objective Identity via Targeted Unicast, Not Component Replication](docs/architecture/adr-001-objective-identity-unicast.md)

**ADR Decision Summary**: All damage to objectives flows through a single `take_damage(lane, attacker_player, amount)` interface. The D1 formula uses `saturating_sub` to prevent underflow. The destruction guard checks `HP_new == 0 AND HP_current > 0` to ensure the consequence path fires exactly once. `amount == 0` short-circuits immediately. Objectives at HP = 0 are no-ops on repeated calls.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**: `ResMut<T>` / `Res<T>` system parameters for `ObjectiveHp` component mutation. Query pattern for finding an objective entity by lane + player: use a `Query<(Entity, &ObjectiveSlot, &mut ObjectiveHp)>` and filter by `slot.lane == lane && slot.player == defending_player`. Use `let Ok(x) = query.single()` not the panicking `query.single()` — Bevy 0.16+ returns `Result`. `liv-bevy-018` skill mandatory.

**Control Manifest Rules (Feature layer)**:
- Required: `take_damage(lane, attacker_player, amount)` is the sole damage entry point; no other system may directly mutate `ObjectiveHp`
- Required: `HP_new = HP_current.saturating_sub(amount)`; destruction fires when `HP_new == 0 AND HP_current > 0`
- Forbidden: No `unwrap()` in production paths — use `expect("message")` or `?` propagation
- Guardrail: Server tick budget ≤ 5ms steady state — damage processing is O(1) per call

---

## Acceptance Criteria

*From GDD `design/gdd/objective-system.md`, scoped to this story:*

- [ ] OS-3 (BLOCKING): GIVEN an objective at HP = 3, WHEN `take_damage(lane, attacker, 2)` is called, THEN objective HP = 1 and no destruction event or consequence path fires.
- [ ] OS-4 (BLOCKING): GIVEN an objective at HP = 2, WHEN `take_damage(lane, attacker, 5)` is called, THEN objective HP = 0 (not negative) and the destruction consequence path fires exactly once.
- [ ] OS-5 (BLOCKING): GIVEN an objective already at HP = 0 (destroyed), WHEN `take_damage(lane, attacker, 3)` is called, THEN HP remains 0, no destruction event fires, and no rewards are emitted.
- [ ] OS-6 (BLOCKING): GIVEN an objective at HP = 3, WHEN any healing effect targets it (i.e., a negative-damage or heal call), THEN HP remains 3 — objectives have no heal interface; `take_damage` does not accept negative `amount` (caller must not pass negative values; a `u32` parameter enforces this at the type level).
- [ ] OS-16 (BLOCKING): GIVEN `take_damage(lane, attacker, 0)` is called, WHEN the system processes it, THEN HP is unchanged, no destruction check runs, and no events are emitted.
- [ ] OS-20 (BLOCKING): GIVEN an objective at HP = 3 and two sequential `take_damage(lane, attacker, 5)` calls in the same sub-step, WHEN both calls are processed, THEN HP = 0, the consequence path fires exactly once (one `AwardGold`, one `ObjectiveDestroyed` queued), and the second call is a no-op.
- [ ] OS-25 (BLOCKING): GIVEN Garde-Temps (Xelor Krosmic) targets an intact objective at HP = 3 (below `objective_hp`), WHEN the effect resolves as `take_damage(lane, attacker_player, objective_hp)` (damage = `objective_hp` = 5), THEN HP = 0 and the full consequence path fires.

---

## Implementation Notes

*Derived from ADR-001 Implementation Guidelines:*

Implement `take_damage` as a function called by systems that need to apply objective damage (Combat Resolution in M2; spell effects in M3):

```rust
pub fn take_damage(
    lane: LaneId,
    attacker_player: PlayerId,
    amount: u32,
    objectives: &mut Query<(&ObjectiveSlot, &mut ObjectiveHp)>,
    hidden: &Res<HiddenObjectives>,
    counters: &mut ResMut<ObjectiveCounters>,
    pending_events: &mut ResMut<PendingObjectiveEvents>,
    config: &Res<GameConfig>,
) {
    if amount == 0 { return; }  // OS-16: short-circuit

    let defending_player = lane_owner(lane); // determined by board context

    let Some((slot, mut hp)) = find_objective(objectives, defending_player, lane) else {
        return;  // lane has no objective (should not happen in valid game state)
    };

    if slot.destroyed { return; }  // OS-5: already-destroyed no-op

    let hp_before = hp.hp;
    hp.hp = hp.hp.saturating_sub(amount);  // OS-3, OS-4: D1 formula

    if hp.hp == 0 && hp_before > 0 {
        // Destruction detected — delegate to consequence path (Story 005)
        trigger_consequence_path(lane, attacker_player, defending_player, hidden, counters, pending_events, config);
    }
}
```

The `amount` parameter is `u32` — the type system prevents negative values (OS-6: no heal interface).

Garde-Temps routing (OS-25): the caller passes `config.objective_hp` as `amount`. No special-case branching inside `take_damage` — the same formula handles lethal damage regardless of source.

`find_objective` must not panic: use `query.iter_mut().find(|(slot, _)| slot.player == defending_player && slot.lane == lane)` — returns `Option`, not `.single()`.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- [Story 005]: The full consequence path body (`trigger_consequence_path` implementation)
- [Story 006]: D4 fake reward draw
- [Story 007]: RESOLUTION-end batch broadcast of `ObjectiveDestroyed` events

---

## QA Test Cases

*Written by qa-lead at story creation. Implement against these — do not invent new test cases.*

- **OS-3**: Non-lethal damage reduces HP without triggering consequence path
  - Given: `World::new()` with one objective entity at `ObjectiveHp { hp: 3 }`; consequence path tracking via a `PendingObjectiveEvents` resource
  - When: `take_damage(lane, attacker, 2)` called
  - Then: `assert_eq!(hp.hp, 1)`; `assert!(pending_events.queue.is_empty())`
  - Edge cases: Damage exactly equal to HP (HP → 0, consequence fires); damage greater than HP (saturating: HP → 0)

- **OS-4**: Lethal damage reduces HP to 0 and fires consequence path exactly once
  - Given: Objective at HP = 2; consequence path tracking via `PendingObjectiveEvents`
  - When: `take_damage(lane, attacker, 5)` called
  - Then: `assert_eq!(hp.hp, 0)`; `assert_eq!(pending_events.queue.len(), 1)` (one `ObjectiveDestroyed` queued)
  - Edge cases: Exact-kill (amount == HP exactly)

- **OS-5**: Already-destroyed objective is a no-op
  - Given: Objective with `ObjectiveSlot { destroyed: true }` and `ObjectiveHp { hp: 0 }`
  - When: `take_damage(lane, attacker, 3)` called
  - Then: `assert_eq!(hp.hp, 0)`; no consequence path triggered; `pending_events.queue.is_empty()`

- **OS-6**: Healing is impossible via type safety
  - Given: `take_damage` function signature uses `amount: u32`
  - When: A caller attempts to pass a negative value
  - Then: Compile error — `u32` does not accept negative literals. Document this as the type-level no-heal guarantee.

- **OS-16**: Zero-damage short-circuits
  - Given: Objective at HP = 3
  - When: `take_damage(lane, attacker, 0)` called
  - Then: `assert_eq!(hp.hp, 3)`; no destruction check; no events queued

- **OS-25**: Garde-Temps lethal damage goes through the standard interface
  - Given: Objective at HP = 3, `GameConfig { objective_hp: 5 }`
  - When: Caller invokes `take_damage(lane, attacker, config.objective_hp)` (amount = 5)
  - Then: `assert_eq!(hp.hp, 0)`; consequence path fires (one event queued); same result as any other lethal call

- **OS-20**: Two sequential lethal calls — consequence fires exactly once
  - Given: Objective at HP = 3
  - When: `take_damage(lane, attacker, 5)` called, then `take_damage(lane, attacker, 5)` called again
  - Then: `assert_eq!(hp.hp, 0)`; `assert_eq!(pending_events.queue.len(), 1)` (exactly one, not two)

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/objective/damage_interface_test.rs` — must exist and pass

**Status**: [x] Created and passing

---

## Dependencies

- Depends on: Story 001 must be DONE (`ObjectiveHp`, `HiddenObjectives`, `ObjectiveCounters` types must exist)
- Unlocks: Story 005 (consequence path is triggered by this story's destruction detection)

## Completion Notes

**Completed**: 2026-05-03
**Verdict**: COMPLETE
**Criteria**: 7/7 passing (OS-3, OS-4, OS-5, OS-6, OS-16, OS-20, OS-25)
**Deviations**: None
**Advisories**: Story manifest `2026-04-29` is older than current control manifest `2026-05-01`; lean review found no applicable rule conflict.
**Test Evidence**: Logic unit test `tests/unit/objective/damage_interface_test.rs` exists and `cargo test -p server --test damage_interface_test` passed 7/7.
**Code Review**: Skipped - lean mode.
**QA Coverage Gate**: Skipped - lean mode.
**Verification**: `cargo fmt -p server -- --check`; `cargo test -p server --test damage_interface_test`; `cargo check -p server`; `git diff --check 033c212^..033c212`.
