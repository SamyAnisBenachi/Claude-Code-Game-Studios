# Story 005: Destruction Consequence Path

> **Epic**: Objective System
> **Status**: Ready
> **Layer**: Feature
> **Type**: Logic
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/objective-system.md`
**Requirements**:
- `TR-OBJ-004` — Destruction consequence: mark destroyed; queue `ObjectiveDestroyed`; `AwardGold +3` if attacker ≠ owner; fake rewards if fake AND attacker ≠ owner; increment `real_count` if real
- `TR-OBJ-006` — Formula D2 loss condition: `real_objectives_destroyed(player) >= loss_threshold (2, fixed constant)`; checked by RSM at RESOLUTION end; mutual destruction = Draw; `ObjectiveCounters` Resource exposed for RSM reads

**ADR Governing Implementation**: [ADR-010: RSM Phase Event Bus](docs/architecture/adr-010-rsm-event-bus.md) (event bus for `AwardGold` and `ObjectiveDestroyed` emission); [ADR-001](docs/architecture/adr-001-objective-identity-unicast.md) (consequence path uses `HiddenObjectives` to determine real/fake)

**ADR Decision Summary (ADR-010)**: Feature systems communicate upward via Messages. `AwardGold` and `ManaCapIncreased` are domain events emitted by the Objective System and consumed by the Economy System. `ObjectiveDestroyed` is queued during sub-step processing and broadcast at RESOLUTION-end (Story 007 handles the batch broadcast timing). The RSM reads `ObjectiveCounters` Resource for GAME_OVER check — it never imports from `feature/objective/`. `take_damage()` calls within sub-step 6 are processed in ascending lane order; `ObjectiveDestroyed` is queued in that same order.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**: `MessageWriter<T>::write()` for emitting `AwardGold` and `ManaCapIncreased` — `EventWriter` no longer exists in Bevy 0.17+. Register these with `app.add_message::<T>()`. Destruction sets `ObjectiveSlot.destroyed = true` — mutate via `Commands` or `Query<&mut ObjectiveSlot>`. `loss_threshold = 2` is a compile-time constant `const LOSS_THRESHOLD: u32 = 2` — do NOT derive it from `fake_count`. `liv-bevy-018` mandatory.

**Control Manifest Rules (Feature layer)**:
- Required: Feature systems communicate upward through Bevy buffered Messages where a cross-system signal is needed; never call Economy System directly (ADR-010)
- Required: Use `#[derive(Message)]`, `MessageWriter<T>::write()`, and `app.add_message::<T>()` for buffered signals; `EventWriter`/`EventReader`/`Events<T>` do not exist in Bevy 0.17+ (ADR-009, ADR-010)
- Required: `ObjectiveCounters { real_destroyed, fake_destroyed }` is a server-side Resource; RSM reads it for GAME_OVER without importing from `feature/objective/` (control manifest)
- Required: If consequence logic is invoked from an exclusive RESOLUTION system, do not use `MessageWriter<T>` as an exclusive-system param; route effects through world resources or a bridge drained by a regular system (control manifest 2026-05-01)
- Forbidden: Never let Feature systems call Core/Foundation systems directly (architecture)
- Forbidden: `loss_threshold` must NOT be derived from `fake_count` — it is always the fixed constant `2` (GDD D2 formula note)
- Performance Guardrail: RESOLUTION work, including objective consequence processing, must stay within the 15 ms worst-case combat-resolution frame budget.

---

## Acceptance Criteria

*From GDD `design/gdd/objective-system.md`, scoped to this story:*

- [ ] OS-7 (BLOCKING): GIVEN any objective (real or fake) is destroyed by the opponent (`attacker_player ≠ defending_player`), WHEN the consequence path runs, THEN `AwardGold { player: attacker_player, amount: 3 }` is emitted exactly once.
- [ ] OS-9 (BLOCKING): GIVEN `real_objectives_destroyed(player) = 1`, WHEN that player's second real objective is destroyed, THEN `real_objectives_destroyed(player) = 2`. (Unit test scope: assert the count only — RSM transition to GAME_OVER is verified in RSM integration tests.)
- [ ] OS-10 (BLOCKING): GIVEN both players each have `real_objectives_destroyed` = 1, WHEN one unit in each player's lane deals lethal damage to a real opponent objective in the same RESOLUTION sub-step sequence, THEN `real_objectives_destroyed(player_a) = 2` AND `real_objectives_destroyed(player_b) = 2` after all sub-steps complete. (Unit test scope: assert counts only.)
- [ ] OS-13a (BLOCKING): GIVEN an objective is destroyed by the opponent, WHEN the consequence path runs, THEN `ObjectiveDestroyed { target_player_id, lane, was_fake: bool }` is queued with the correct payload. (This story tests queuing. Story 007 tests that it broadcasts at RESOLUTION-end, not mid-sub-step.)
- [ ] OS-14 (BLOCKING): GIVEN `attacker_player == defending_player` (self-destruction), WHEN a real objective is destroyed, THEN no gold is awarded, no fake rewards fire, and `real_objectives_destroyed(defending_player)` increments by 1.
- [ ] OS-18a (BLOCKING): GIVEN units at Cell 8 in both lane 1 and lane 3 dealing lethal damage to the opponent's objectives in the same RESOLUTION sub-step 6, WHEN `take_damage()` is processed, THEN the lane 1 consequence path fires before the lane 3 path (verified by event emission order in unit test).
- [ ] OS-21 (BLOCKING): GIVEN `attacker_player == defending_player` and the destroyed objective is a fake, WHEN the consequence path runs, THEN `fake_objectives_destroyed(defender)` is unchanged, no `AwardGold` is emitted, and no mana cap or card reward fires. The slot is marked destroyed and `ObjectiveDestroyed` is queued.

---

## Implementation Notes

*Derived from ADR-010 and ADR-001 Implementation Guidelines:*

```rust
pub fn apply_consequence_path(
    lane: LaneId,
    attacker_player: PlayerId,
    defending_player: PlayerId,
    hidden: &Res<HiddenObjectives>,
    counters: &mut ResMut<ObjectiveCounters>,
    pending_events: &mut ResMut<PendingObjectiveEvents>,
    award_gold_writer: &mut MessageWriter<AwardGold>,
    // D4 reward draw parameters passed in from Story 006 integration
    rng: Option<&mut ResMut<ServerRng>>,
    card_pool: Option<&mut ResMut<CardPool>>,
    player_hands: Option<&mut ResMut<PlayerHands>>,
    mana_cap_writer: Option<&mut MessageWriter<ManaCapIncreased>>,
) {
    let is_fake = *hidden.identities.get(&(defending_player, lane)).unwrap_or(&false);
    let self_destruction = attacker_player == defending_player;

    // Step 1: Mark destroyed and queue ObjectiveDestroyed
    // (slot.destroyed already set by caller before this function is called)
    pending_events.queue.push(ObjectiveDestroyed {
        target_player_id: defending_player,
        lane,
        was_fake: is_fake,
    });

    if !self_destruction {
        // Step 2: Gold award (attacker ≠ owner)
        award_gold_writer.write(AwardGold { player: attacker_player, amount: 3 });

        if is_fake {
            // Step 3: Fake-specific rewards
            counters.fake_destroyed.entry(attacker_player).and_modify(|c| *c += 1).or_insert(1);
            // D4 reward draw delegated to Story 006
            if let (Some(rng), Some(pool), Some(hands), Some(mana_writer)) = (rng, card_pool, player_hands, mana_cap_writer) {
                draw_fake_reward(attacker_player, rng, pool, hands, award_gold_writer, mana_writer);
            }
        }
    }

    if !is_fake {
        // Step 4: Real objective destroyed — increment real counter (fires for self-destruction too)
        counters.real_destroyed.entry(defending_player).and_modify(|c| *c += 1).or_insert(1);
    }
    // self_destruction + fake: step 3 skipped, step 4 skipped (not real) — OS-21 covered
}
```

`ObjectiveCounters.real_destroyed` is the read interface for RSM GAME_OVER check. The RSM reads `Res<ObjectiveCounters>` — it does not call into `feature/objective/` directly.

Emission ordering for `PendingObjectiveEvents.queue`: callers must call `take_damage()` in ascending lane order (lane 1 → 2 → 3 → 4 → 5). The event queue order reflects the call order, which is the RESOLUTION sub-step 6 processing order. This is enforced by the Combat Resolution system (M2) — for M1 unit testing, call `take_damage()` in lane order manually.

`const LOSS_THRESHOLD: u32 = 2;` — this constant is used by the RSM, not by this story. This story updates the count; the RSM evaluates the threshold.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- [Story 006]: The D4 fake reward draw (`draw_fake_reward` body — ManaCapIncreased vs FreeCardPick branching)
- [Story 007]: Batch broadcast of `PendingObjectiveEvents.queue` at RESOLUTION-end sync; Sang Méprise unicast

---

## QA Test Cases

*Written by qa-lead at story creation. Implement against these — do not invent new test cases.*

- **OS-7**: Gold award on opponent destruction
  - Given: `World::new()` with `ObjectiveHp { hp: 2 }`, attacker ≠ defender, real objective, `MessageWriter<AwardGold>` tracked in test
  - When: `apply_consequence_path(lane, attacker, defender, ..)` called
  - Then: `assert_eq!(collected_award_gold.len(), 1)`; `assert_eq!(collected_award_gold[0].player, attacker)`; `assert_eq!(collected_award_gold[0].amount, 3)`
  - Edge cases: Fake objective — same gold award (gold fires regardless of real/fake); self-destruction — no gold

- **OS-9**: Real count increments correctly
  - Given: Two real objectives, `counters.real_destroyed[player_b] = 1`
  - When: Second real objective for `player_b` is destroyed
  - Then: `assert_eq!(counters.real_destroyed[player_b], 2)`
  - Edge cases: Fake objective destroyed — `real_destroyed` unchanged

- **OS-10**: Both players' real counts reach 2 simultaneously
  - Given: Both players at `real_destroyed = 1`; one real objective per player left
  - When: `apply_consequence_path` called for both players in same sub-step sequence
  - Then: `assert_eq!(counters.real_destroyed[player_a], 2)`; `assert_eq!(counters.real_destroyed[player_b], 2)`

- **OS-13a**: `ObjectiveDestroyed` queued with correct payload
  - Given: Real objective destroyed in lane 3 by opponent
  - When: `apply_consequence_path(LaneId(3), attacker, defender, ..)` called
  - Then: `assert_eq!(pending_events.queue[0].lane, LaneId(3))`; `assert_eq!(pending_events.queue[0].was_fake, false)`; `assert_eq!(pending_events.queue[0].target_player_id, defender)`

- **OS-14**: Self-destruction of real objective — no gold, real count advances
  - Given: Self-destruction scenario (`attacker == defender`), real objective
  - When: Consequence path runs
  - Then: No `AwardGold` emitted; `counters.real_destroyed[defender]` increments by 1

- **OS-18a**: Lane-ascending consequence path order
  - Given: Two lethal `take_damage` calls: lane 1 then lane 3 (both against opponent objectives)
  - When: Both are processed
  - Then: `pending_events.queue[0].lane == LaneId(1)`; `pending_events.queue[1].lane == LaneId(3)`

- **OS-21**: Self-destruction of fake — no rewards, slot destroyed, event queued
  - Given: Self-destruction (`attacker == defender`), fake objective in lane 2
  - When: Consequence path runs
  - Then: No `AwardGold`; `counters.fake_destroyed[defender]` unchanged; `counters.real_destroyed` unchanged; `pending_events.queue.len() == 1` with `was_fake: true`

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/objective/consequence_path_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 004 must be DONE (consequence path is triggered by the destruction detection in `take_damage`)
- Unlocks: Story 006 (D4 fake reward draw is called from within the consequence path)
