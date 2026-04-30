# Story 006: Sacrier Effects — Sang Méprise and Punition

> **Epic**: Class System
> **Status**: Ready
> **Layer**: Feature (M3)
> **Type**: Integration
> **Manifest Version**: 2026-04-30

## Context

**GDD**: `design/gdd/class-system.md`
**Requirement**: `TR-CS-005`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-014: Class System Architecture — PlayerSessionState, SourceClass Component, and Direct Effect Dispatch
**ADR Decision Summary**: Class effects are plain Rust functions called from within the RESOLUTION system body. Sang Méprise and Punition both interact with the Objective System: Sang Méprise triggers the reliable unicast reveal channel (`S2CSangMepriseReveal`); Punition calls `take_damage()` on self and opponent objectives. Both resolve at sub-step 1 (PLACEMENT commit). RSM evaluates loss condition at RESOLUTION end via `ObjectiveCounters` — Punition-triggered self-elimination routes through this standard path.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: MEDIUM
**Engine Notes**:
- `S2CSangMepriseReveal` must use Lightyear's reliable channel unicast — separate unicast per player (both Player A and Player B receive it). Verify Lightyear 0.26 unicast target API before implementing (`NetworkTarget::Single(peer_id)` — check `docs/engine-reference/bevy/current-best-practices.md`).
- `sang_meprise_active: bool` is owned by Objective System resource, cleared at RESOLUTION end — this story does not own the clear; the Objective System's RESOLUTION-end hook does.
- NP-1 (PlayerSnapshot.class_id) is resolved by Story 001. NP-5 (reserve mutations mid-RESOLUTION) does not block this story.
- Integration test requires Lightyear unicast infrastructure (NP-level) — unit test can assert `sang_meprise_active = true` and `reveal_set` populated without asserting message delivery.

**Control Manifest Rules (Feature Layer)**:
- Required: Objective System `take_damage()` is sole damage interface — ADR (Objective System)
- Required: Feature systems communicate upward via events; never call Core systems directly — ADR-010
- Required: Sang Méprise reveal: reliable unicast to opponent via `S2CSangMepriseReveal` — ADR-001
- Forbidden: Never send opponent `is_fake` values in any broadcast message — ADR-001
- Guardrail: `S2CSangMepriseReveal` payload: ~10 slots × small struct — negligible bandwidth

---

## Acceptance Criteria

*From GDD `design/gdd/class-system.md`, CS-5 and CS-6 formulas:*

- [ ] **CS-AC-13** GIVEN Sacrier player who submitted Sang Méprise at PLACEMENT, WHEN RESOLUTION begins and placements are committed, THEN both players receive a unicast `S2CSangMepriseReveal` containing the `is_fake` status for every alive objective slot across both players. *(Integration test — requires NP-1 and Lightyear unicast infrastructure. Unit test: assert `sang_meprise_active = true` and `reveal_set` is correctly populated after placement commit.)*
- [ ] **CS-AC-14a** GIVEN Sang Méprise was active during a RESOLUTION, WHEN that RESOLUTION ends (RSM exits sub-step 6 or transitions to next phase), THEN the server's `sang_meprise_active` flag is `false`; subsequent messages for the next round do not include objective reveal data. *(Unit-testable: assert `sang_meprise_active = false` after RESOLUTION-end hook fires.)*
- [ ] **CS-AC-15** GIVEN Sacrier player with 2 alive real objectives, WHEN Punition is played targeting one real objective, THEN that objective HP→0 AND 3 damage is applied to each alive opponent objective individually.
- [ ] **CS-AC-16** GIVEN Sacrier player with 0 alive real objectives, WHEN Punition play is submitted, THEN server rejects; mana untouched.
- [ ] **CS-AC-17** GIVEN Sacrier player has exactly 1 alive real objective remaining (2 already destroyed), WHEN Punition targets that last real objective, THEN the objective HP→0 AND the RSM transitions to GAME_OVER with the Sacrier as the losing player (self-elimination is valid, not a bug).

---

## Implementation Notes

*Derived from ADR-014 Decision §4 and GDD Formulas CS-5, CS-6:*

**File location**: `server/src/core/resolution/effects.rs`

**CS-5 — Sang Méprise**:
```rust
pub fn apply_sang_meprise(
    objectives: &mut ObjectiveState,
    unicast_queue: &mut UnicastQueue,  // server-side outbound queue for Lightyear sends
    player_a_peer: PeerId,
    player_b_peer: PeerId,
) {
    objectives.sang_meprise_active = true;
    let reveal_set: Vec<ObjectiveSlotReveal> = objectives
        .all_slots()
        .filter(|s| s.is_alive)
        .map(|s| ObjectiveSlotReveal { owner: s.owner, lane: s.lane, is_fake: s.is_fake })
        .collect();
    // Unicast identical payload to BOTH players (separate sends per ADR-001 and GDD CS-5)
    unicast_queue.push(player_a_peer, S2CSangMepriseReveal { slots: reveal_set.clone() });
    unicast_queue.push(player_b_peer, S2CSangMepriseReveal { slots: reveal_set });
}
```

`sang_meprise_active = false` is NOT set here — the Objective System's RESOLUTION-end hook owns the clear. This story should NOT touch that field on the clear path.

**CS-6 — Punition**:
```rust
pub fn apply_punition(
    sessions: &PlayerSessions,
    objectives: &mut ObjectiveState,
    player_id: PlayerId,
    target_lane: u8,
) -> Result<(), PunitionError> {
    let has_eligible_real = objectives.count_alive_real(player_id) >= 1;
    if !has_eligible_real {
        return Err(PunitionError::NoEligibleRealObjective);
    }
    // Self-destroy chosen real objective
    let lethal_hp = objectives.hp(player_id, target_lane);
    objectives.take_damage(player_id, target_lane, player_id, lethal_hp);  // self-attacker

    // 3 damage to each alive opponent objective
    for lane in 1..=5_u8 {
        if objectives.is_alive(player_id.opponent(), lane) {
            objectives.take_damage(player_id.opponent(), lane, player_id, 3);
        }
    }
    Ok(())
}
```

**Self-elimination path** (CS-AC-17): After `take_damage` on the last real objective, `real_objectives_destroyed(sacrier) >= 3` becomes true. RSM evaluates this condition at RESOLUTION end via `ObjectiveCounters` resource — Punition does NOT directly trigger GAME_OVER. The standard RSM loss check fires at RESOLUTION-end boundary and picks up the count. This is not a bug; self-elimination is explicitly documented (GDD CS-AC-17).

**Simultaneous mutual destruction** (GDD edge case): If Punition simultaneously destroys the opponent's last real objective AND Sacrier's own loss condition is met, RSM evaluates both simultaneously → Draw. This is the only scenario where Punition produces a Draw.

**Mana rejection**: On `Err(PunitionError::NoEligibleRealObjective)`, mana is NOT deducted. The calling site in RESOLUTION must check `Result` before deducting mana (validation precedes deduction — ADR-002).

---

## Out of Scope

*Handled by neighbouring stories:*

- Story 001: PlayerSessions scaffold
- Objective System: `take_damage()` implementation, `sang_meprise_active` field definition, RESOLUTION-end clear hook — owned by `objective-system` epic
- RSM: GAME_OVER detection from `ObjectiveCounters` at RESOLUTION end — owned by `round-state-machine` epic
- CS-AC-14b (ADVISORY): Client renders objectives as hidden after Sang Méprise clears — Presentation layer

---

## QA Test Cases

*Integration story — automated test specs. Unit-level assertions on ObjectiveState; Lightyear delivery assertions require integration environment.*

- **AC CS-AC-13 — Sang Méprise reveal set populated**:
  - Given: 2 players; Player A has 5 objective slots (2 real alive, 3 fake alive); Player B has 5 slots (1 real alive, 3 fake alive, 1 real destroyed)
  - When: `apply_sang_meprise(&mut objectives, &mut unicast_queue, ...)` called
  - Then: `objectives.sang_meprise_active == true`; `reveal_set.len() == 8` (all alive slots from both players, 1 destroyed excluded); each slot's `is_fake` value matches `HiddenObjectives`
  - Edge cases: both players played Sang Méprise same RESOLUTION → second call is idempotent (set-once-per-RESOLUTION rule per GDD edge case)

- **AC CS-AC-14a — sang_meprise_active cleared at RESOLUTION end**:
  - Given: `sang_meprise_active = true` after Sang Méprise fired
  - When: RESOLUTION-end hook in Objective System fires
  - Then: `sang_meprise_active == false`; this story verifies the Objective System's hook clears it correctly (integration assertion)

- **AC CS-AC-15 — Punition: self-destroy + AOE**:
  - Given: Sacrier has 2 alive real objectives (lanes 1, 3); opponent has 3 alive objectives (lanes 1, 2, 4)
  - When: `apply_punition(..., target_lane = 1)` called
  - Then: Sacrier lane 1 objective HP = 0; each alive opponent objective takes 3 damage (lanes 1, 2, 4); total opponent damage = 9
  - Edge cases: opponent has 5 alive objectives → each receives exactly 3 damage (not 15 total to one)

- **AC CS-AC-16 — Punition rejection when no alive real**:
  - Given: Sacrier has 0 alive real objectives (all 3 real destroyed OR all slots are fakes)
  - When: `apply_punition(...)` called
  - Then: returns `Err(PunitionError::NoEligibleRealObjective)`; Sacrier's objective HP unchanged; opponent objective HP unchanged

- **AC CS-AC-17 — Self-elimination via Punition**:
  - Given: Sacrier has 1 alive real objective (lane 2), 2 real already destroyed
  - When: `apply_punition(..., target_lane = 2)` called; then RSM RESOLUTION-end check runs
  - Then: Sacrier lane 2 objective HP = 0; `real_objectives_destroyed(sacrier) == 3 >= 2` → RSM fires GAME_OVER with Sacrier as loser

---

## Test Evidence

**Story Type**: Integration
**Required evidence**: `tests/integration/class/sacrier_effects_test.rs` — must exist and pass; OR a documented playtest confirming CS-AC-13 message delivery

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 (PlayerSessions) — must be DONE
- Depends on: `objective-system` epic (ObjectiveState, take_damage(), sang_meprise_active flag, ObjectiveCounters) — must be DONE
- Depends on: `round-state-machine` epic (GAME_OVER detection at RESOLUTION end) — must be DONE for CS-AC-17
- Depends on: NP-1 resolution (PlayerSnapshot.class_id) — resolved by Story 001 (protocol additions)
- Unlocks: Story 007 (Sadida Seeds — validates Objective System integration pattern established here); Story 010 (Token passives also use ObjectiveState)
