# Story 006: Sub-step 4 — Dead Removal + DEATH Chains + Kill Gold

> **Epic**: Combat Resolution
> **Status**: Complete
> **Layer**: Feature
> **Type**: Logic
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/combat-resolution.md`
**Requirement**: `TR-CR-008` (CR-16 — SS4 kill gold), `TR-CR-007` (CR-23 — FINAL BLOW stays in kill sub-step), `TR-CR-022` (CR-25 — sequential DEATH chains)

**ADR Governing Implementation**: ADR-017: Combat Resolution Execution Architecture; ADR-022: Keyword System Timing Trigger Observer Architecture
**ADR Decision Summary**: SS4 removes all units at 0 HP, fires DEATH trigger chains sequentially in lane order via `ChainDeathBuffer`, and drains the `kill_log` for SS3 kills into `GoldAwarded` log entries. FINAL BLOW for SS3 kills fires IN SS3 (Story 005) — not consolidated here. SS4 handles the despawn and the SS3 kill gold only; SS6 kills are drained in a post-SS6 cleanup pass.

**Engine**: Bevy 0.18 | **Risk**: HIGH
**Engine Notes**: `commands.entity(e).despawn()` — `despawn_recursive()` is removed since Bevy 0.16. Inside the exclusive system, use `world.despawn(entity_id)` directly (stable API since Bevy 0.14). DEATH trigger dispatch uses `ChainDeathBuffer` and `world.trigger_targets(UnitDied { ... }, entity)` from the exclusive system; do not recursively call `world.trigger_targets()` from inside `on_unit_died`. Increment `iter_count` per DEATH chain link.

**Control Manifest Rules (Feature layer)**:
- Required: Drain `kill_log` for SS3 kills after removing dead units; emit `GoldAwarded` entries into ResolutionLog; DEATH trigger chains are sequential (fire A's DEATH, then B's if B died from A's trigger)
- Required: Manage DEATH chains via `ChainDeathBuffer` `VecDeque`; clear it at SS4 start; seed initial deaths in lane order; drain one entity at a time
- Forbidden: Never consolidate SS6 FINAL BLOW into SS4; never award kill gold for objective destruction; never use recursive `world.trigger_targets()` inside `on_unit_died`
- Guardrail: DEATH chain depth bounded by `iter_count`; infinite DEATH loops terminate at 10,000; SS4 must stay within ADR-017's full RESOLUTION <= 15 ms frame budget

---

## Acceptance Criteria

*From GDD `design/gdd/combat-resolution.md`, scoped to this story:*

- [x] **CR-16**: GIVEN a unit kills an enemy unit, WHEN sub-step 4 processes the dead unit, THEN the killing player immediately receives +1 gold (emitted as `GoldAwarded { player, amount: 1, reason: Kill }` in ResolutionLog)
- [x] **CR-23**: GIVEN a unit kills another unit in sub-step 6 (standard combat), WHEN FINAL BLOW fires, THEN it fires in sub-step 6 — NOT consolidated to sub-step 4 (verified by checking log order: FINAL_BLOW entry has `sub_step: 6`, not `sub_step: 4`)
- [x] **CR-25**: GIVEN unit A's DEATH trigger kills unit B in sub-step 4, WHEN DEATH triggers process, THEN B's DEATH trigger fires AFTER A's DEATH trigger completes (sequential chain, not simultaneous)

---

## Implementation Notes

*Derived from ADR-017 Architecture Diagram (sub-step 4 section) and GDD kill-log mechanism:*

```rust
fn remove_dead(
    world: &mut World,
    snapshots: &mut Vec<UnitSnapshot>,
    log: &mut ResolutionLog,
    kill_log: &mut Vec<KillRecord>,
    chain_death_buffer: &mut ChainDeathBuffer,
    iter_count: &mut u32,
) {
    // 1. Clear SS4 chain state and collect all units at HP == 0
    chain_death_buffer.0.clear();
    let dead_units: Vec<UnitId> = snapshots.iter()
        .filter(|u| u.hp == 0 && !u.already_removed)
        .map(|u| u.unit_id)
        .collect();

    // 2. Sort by lane order for determinism and seed the explicit queue
    dead_units.sort_by_key(|id| snapshots[id].lane);
    chain_death_buffer.0.extend(dead_units);

    // 3. Drain one entity at a time. on_unit_died may push chain deaths here.
    while let Some(unit_id) = chain_death_buffer.0.pop_front() {
        *iter_count += 1;
        if *iter_count > 10_000 {
            abort_resolution_as_draw(world, log);
            break;
        }

        let snapshot = snapshots.by_unit_id_mut(unit_id);
        if snapshot.already_removed {
            continue;
        }

        // Despawn ECS entity
        world.despawn(snapshot.entity_id);
        snapshot.already_removed = true;

        log.push(ResolutionEvent::UnitRemoved { unit_id, lane, cell });

        // Fire DEATH observer synchronously from the exclusive system.
        // The observer may enqueue new deaths into ChainDeathBuffer.
        if snapshot.keywords.contains(Death) {
            world.trigger_targets(UnitDied { attacker: snapshot.killer }, snapshot.entity_id);
            log.push(ResolutionEvent::KeywordTriggered {
                unit_id,
                keyword: KeywordKind::Death,
                sub_step: 4,
            });
        }
    }

    // 4. Drain kill_log for SS3 kills → emit GoldAwarded entries
    for record in kill_log.drain(..).filter(|r| r.lethal_sub_step == 3) {
        log.push(ResolutionEvent::GoldAwarded {
            player: record.killer_player_id,
            amount: 1,
            reason: GoldReason::Kill,
        });
        apply_gold_award(world, record.killer_player_id, 1);
    }
}
```

**DEATH chain rule (CR-25)**: SS4 uses `ChainDeathBuffer` as an explicit FIFO queue. Initial dead units are seeded in lane order. Each queue entry is removed from board state, logs `UnitRemoved`, then fires `world.trigger_targets(UnitDied { ... }, entity)` synchronously from the exclusive system. If A's DEATH trigger kills B, the observer enqueues B into `ChainDeathBuffer`; B is drained only after A's observer completes. Do not recurse from `on_unit_died`; queueing is the sequencing mechanism. The `iter_count` guard aborts pathological loops at 10,000 iterations.

**SS6 kill gold**: The `kill_log` for SS6 kills is drained in a post-SS6-combat cleanup pass within `execute_combat` (Story 007). Story 006 only drains SS3 kills (records where `lethal_sub_step == 3`).

**CR-23 verification**: This AC is primarily verified by checking that FINAL_BLOW triggered in SS6 has `sub_step: 6` in its log entry — Story 007 owns the actual implementation. This story's responsibility is to NOT fire FINAL_BLOW during SS4.

**Performance budget**: SS4 is part of ADR-017's single-frame RESOLUTION batch and must remain within the full RESOLUTION <= 15 ms frame budget. Queue draining, death-trigger dispatch, kill-log draining, and gold application must remain bounded by live unit count plus the 10,000 iteration safety guard.

---

## Out of Scope

- Story 005: SS3 FIRST STRIKE kills (kill_log populated there; drained here for kill gold)
- Story 007: SS6 standard combat kills and post-SS6 kill gold drain
- Story 009: Objective destruction gold (+3g) — separate from kill gold

---

## QA Test Cases

*(Lean mode — test cases authored inline)*

- **CR-16** (kill gold in SS4):
  - Given: Unit A (Player 1 controller) kills Unit B in SS3 (HP=0 after SS3); `kill_log` has one record `{ killer: Player1, lethal_sub_step: 3 }`
  - When: SS4 `remove_dead` runs
  - Then: `GoldAwarded { player: Player1, amount: 1, reason: Kill }` appears in log; Player1 gold incremented by 1

- **CR-23** (FINAL BLOW NOT in SS4):
  - Given: Unit A kills Unit B in SS6 (standard combat); FINAL BLOW keyword fires
  - When: ResolutionLog is inspected
  - Then: `KeywordTriggered { keyword: FINAL_BLOW }` has `sub_step: 6`; there is NO `KeywordTriggered { keyword: FINAL_BLOW }` with `sub_step: 4` in the log

- **CR-25** (DEATH chain sequential):
  - Given: Unit A has DEATH trigger that deals damage killing Unit B; Unit B has DEATH trigger
  - When: SS4 processes A's death
  - Then: log order = A's `UnitRemoved` → A's DEATH `KeywordTriggered` → B's `UnitRemoved` → B's DEATH `KeywordTriggered`; B's DEATH does NOT appear before A's DEATH completes
  - Edge case: iter_count guard — if DEATH chain exceeds 10,000 iterations, abort with Draw

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/combat/substep4_dead_removal_test.rs` — must exist and pass

**Status**: [x] Created and passing

---

## Dependencies

- Depends on: Story 001 (scaffold), Story 002 (CombatResult.net_damage feeds kill_log), Story 005 (kill_log populated with SS3 kills)
- Unlocks: Story 007 (SS6 combat uses board state after SS4 removals)

---

## Completion Notes

**Completed**: 2026-05-04
**Criteria**: 3/3 passing (CR-16, CR-23, CR-25)
**Deviations**: Advisory only - implementation dispatches `UnitDied` with Bevy's targeted `EntityEvent` path via `world.trigger(...)` rather than the manifest wording `world.trigger_targets(...)`; `cargo check` and CR-25 coverage confirm synchronous targeted observer behavior, `ChainDeathBuffer` FIFO sequencing, and no recursive DEATH-chain dispatch.
**Test Evidence**: Logic: `tests/unit/combat/substep4_dead_removal_test.rs` exists and passed 3/3 via `cargo test -p server --test substep4_dead_removal_test`. `cargo check -p server` passed. `git diff --check` passed.
**Code Review**: Skipped - lean mode.
