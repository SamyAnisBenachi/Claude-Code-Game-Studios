# Story 008: Sub-step 6 - RANGE Targeting

> **Epic**: Combat Resolution
> **Status**: Complete
> **Layer**: Feature
> **Type**: Logic
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/combat-resolution.md`
**Requirement**: `TR-CR-023` (CR-3 - nearest and equidistant RANGE target selection), `TR-CR-020` (CR-4 - RANGE + FIRST STRIKE two damage events), `TR-CR-024` (CR-28 - forward-only RANGE filtering), `TR-CR-019` (CR-44 - RANGE attacks WALL from current cell), `TR-CR-025` (CR-45 - fresh SS6 target acquisition after SS4 removal)

**ADR Governing Implementation**: ADR-017: Combat Resolution Execution Architecture; ADR-005: Server-side RNG
**ADR Decision Summary**: RANGE units attack the nearest enemy in the forward direction without advancing to that cell. RANGE + FIRST STRIKE units fire in SS3 and again in SS6 as two independent damage events. Equidistant target selection uses the `RangeEquidistantSelect` RNG slot from `ServerRng` and follows ADR-005 RESOLUTION ordering. RANGE attackers at distance cannot trigger COUNTERATTACK on the target.

**Engine**: Bevy 0.18 | **Risk**: HIGH
**Engine Notes**: RANGE targeting formula: `valid_targets = enemy units at cells [C+1..C+RANGE_X]` for Player A or `[C-RANGE_X..C-1]` for Player B, clamped to `[1, 8]`. Equidistant selection must call an intent-named `ServerRng` RANGE equidistant method backed by `RngEvent::RangeEquidistantSelect`; combat targeting code must not call the raw seed-advance helper directly. Single-nearest RANGE targeting consumes zero RNG seeds.

**Control Manifest Rules (Feature layer)**:
- Required: RANGE targeting checks `[C+1..C+X]` forward only for Player A and `[C-X..C-1]` forward only for Player B; equidistant selection consumes exactly one `RangeEquidistantSelect` seed; RANGE units do not advance to the target cell; `RangeEquidistantSelect` is consumed in strict ADR-005 RESOLUTION order before `TeleportRandomDest`.
- Required: all RNG access goes through intent-named `ServerRng` methods; every consumption writes an audit entry in the same call; candidate ordering for equidistant selection is deterministic.
- Forbidden: never target friendly units; never target cells behind the RANGE unit; never consume RNG if only one nearest target exists; never trigger COUNTERATTACK from a RANGE attacker that stayed at distance.

---

## Acceptance Criteria

*From GDD `design/gdd/combat-resolution.md`, scoped to this story:*

- [x] **CR-3**: GIVEN a unit with RANGE 1-X at cell C with a single nearest enemy, WHEN sub-step 6 executes, THEN it attacks that nearest enemy (Player A: minimum cell-distance in cells C+1 to C+X; Player B: C-X to C-1), does not advance, and consumes zero RNG seeds. GIVEN two or more nearest valid targets are equidistant, WHEN sub-step 6 executes, THEN eligible targets are ordered by `(distance_from_attacker, target_cell, target_unit_id)`, exactly one `RangeEquidistantSelect` seed is consumed through the intent-named `ServerRng` method, and the selected target is deterministic for the same seed and ordered eligible set.
- [x] **CR-4**: GIVEN a unit with RANGE 1-X and FIRST STRIKE, WHEN RESOLUTION executes, THEN two distinct `CombatDamage` entries are in the ResolutionLog: one with `sub_step: 3` from the FIRST STRIKE pass and one with `sub_step: 6` from the standard RANGE pass.
- [x] **CR-28**: GIVEN a RANGE unit with enemies both forward and behind it, both within the numeric RANGE value, WHEN sub-step 6 executes, THEN only the forward enemy is eligible and the enemy behind is never selected.
- [x] **CR-44**: GIVEN a RANGE 1-3 unit at cell C with a WALL unit at cell C+2 within RANGE, WHEN sub-step 5 executes, THEN the RANGE unit's cell position is unchanged; WHEN sub-step 6 executes, THEN the RANGE unit attacks the WALL from cell C and a `CombatDamage` record is emitted with defender = WALL.
- [x] **CR-45**: GIVEN a RANGE + FIRST STRIKE unit kills its sub-step 3 target, AND a different enemy unit exists within range at sub-step 6 entry, WHEN sub-step 4 removes the killed unit AND sub-step 6 executes, THEN the RANGE unit acquires the surviving enemy as its sub-step 6 target and a `CombatDamage` record is emitted for that target.

---

## Implementation Notes

*Derived from ADR-017, ADR-005, and GDD Formula 4 (RANGE Target Selection):*

```rust
/// RANGE target selection formula (Formula 4 from GDD).
fn select_range_target(
    attacker: &UnitSnapshot,
    board: &BoardState,
    rng: &mut ServerRng,
) -> Option<UnitId> {
    let range_x = attacker.range_value; // must be >= 1

    // Forward direction only.
    let (cell_min, cell_max) = match attacker.player {
        Player::A => (
            (attacker.cell + 1).min(8),
            (attacker.cell + range_x).min(8),
        ),
        Player::B => (
            (attacker.cell as i16 - range_x as i16).max(1) as u8,
            attacker.cell.saturating_sub(1),
        ),
    };

    // Collect enemy units within [cell_min, cell_max] in the attacker's lane.
    let mut candidates: Vec<(UnitId, u8)> = board.enemy_units_in_range(
        attacker.player, attacker.lane, cell_min, cell_max
    ).map(|u| (u.unit_id, u.cell)).collect();

    if candidates.is_empty() { return None; }

    // Exact deterministic ordering before resolving nearest/equidistant targets.
    candidates.sort_by_key(|(id, cell)| (cell.abs_diff(attacker.cell), *cell, *id));
    let min_dist = candidates[0].1.abs_diff(attacker.cell);
    let equidistant: Vec<UnitId> = candidates.iter()
        .filter(|(_, cell)| cell.abs_diff(attacker.cell) == min_dist)
        .map(|(id, _)| *id)
        .collect();

    if equidistant.len() == 1 {
        Some(equidistant[0]) // No RNG consumed.
    } else {
        // Intent-named ServerRng method consumes exactly one RangeEquidistantSelect seed.
        let seed = rng.range_equidistant_select(attacker.player, attacker.lane);
        let idx = (seed as usize) % equidistant.len();
        Some(equidistant[idx])
    }
}
```

**CR-3 (equidistant target selection)**: The ordered eligible set is stable before RNG is applied. For a fixed seed and the same ordered eligible set, the selected target is identical across repeated runs. `RangeEquidistantSelect` is consumed once per equidistant RANGE attack only, never for a single nearest target or for no valid targets.

**CR-45 (RANGE + FIRST STRIKE target reacquisition)**: At SS6 entry, call `select_range_target` fresh. Do not cache the SS3 target. If the SS3 target was removed in SS4, a new target is selected from the current board state. If no target exists within range, no SS6 attack occurs.

**CR-44 (RANGE attacks WALL)**: WALL units are valid RANGE targets. Include WALLs in `board.enemy_units_in_range()`. The RANGE unit's cell does not change in SS5; Story 004 owns the SS5 movement exemption and this story verifies the resulting SS6 attack.

**CR-28 (forward-only)**: The `cell_min/cell_max` computation strictly uses cells ahead of the attacker's position. For Player A, cells behind are `< attacker.cell`. For Player B, cells behind are `> attacker.cell`.

---

## Performance Note

RANGE targeting runs inside the ADR-017 `resolve_combat(world: &mut World)` RESOLUTION pass. Candidate collection, deterministic ordering, equidistant selection, and SS6 damage emission must remain bounded by live units in the attacker's lane and keep the full worst-case combat resolution frame within the control-manifest budget of `<= 15 ms`.

---

## Out of Scope

- Story 004: SS5 movement and the RANGE-vs-WALL movement exemption. This story consumes that completed position state and verifies the SS6 RANGE attack from the unchanged cell.
- Story 005: SS3 FIRST STRIKE damage execution. This story handles the SS6 RANGE pass and the CR-45 fresh target acquisition after SS4.
- Story 006: SS4 dead removal. This story assumes SS4 has removed dead units before SS6 target selection.
- Story 007: SHIELD pre-check and COUNTERATTACK internals. RANGE attacks in SS6 use the existing SS6 damage path, and RANGE attackers at distance must not trigger COUNTERATTACK.
- Story 009: Objective damage at Cell 8. A RANGE unit at Cell 8 cannot attack forward; objective damage is governed by objective damage rules, not RANGE targeting.

---

## QA Test Cases

*(Lean mode - test cases authored inline)*

- **CR-3** (single nearest target selection):
  - Given: RANGE 3 unit at cell 2 (Player A); enemies at cell 4 (nearer) and cell 5 (farther)
  - When: SS6 RANGE targeting runs
  - Then: attack targets the cell 4 unit; no `RangeEquidistantSelect` audit entry is recorded

- **CR-3** (equidistant target selection):
  - Given: RANGE 3 unit at cell 2 (Player A); two enemy targets at cell 4 with UnitIds 41 and 42; the ordered nearest eligible set is `[41, 42]`; a deterministic test seed yields `seed % 2 == 1`
  - When: SS6 RANGE targeting runs
  - Then: exactly one `RangeEquidistantSelect` audit entry is recorded; target UnitId 42 is selected; rerunning with the same seed and eligible set selects UnitId 42 again

- **CR-4** (RANGE + FIRST STRIKE two damage events):
  - Given: RANGE 2 + FIRST STRIKE unit at cell 3; enemy at cell 5
  - When: RESOLUTION completes
  - Then: log contains `CombatDamage { sub_step: 3 }` and `CombatDamage { sub_step: 6 }` for the same attacker-defender pair

- **CR-28** (forward-only):
  - Given: RANGE 3 unit (Player A) at cell 4; enemy at cell 2 behind and enemy at cell 6 forward
  - When: SS6 targeting runs
  - Then: only the cell 6 enemy is eligible; the cell 2 enemy is never selected

- **CR-44** (RANGE attacks WALL from distance):
  - Given: RANGE 3 unit at cell 2 (MP=0, position unchanged); WALL at cell 4
  - When: SS6 runs
  - Then: `CombatDamage { attacker: RANGE_unit, defender: WALL, sub_step: 6 }` is in the log

- **CR-45** (target reacquisition after SS3 kill):
  - Given: RANGE + FIRST STRIKE unit; target A is killed in SS3; target B exists at cell C+3 in range
  - When: SS4 removes A; SS6 runs
  - Then: `CombatDamage { defender: B, sub_step: 6 }` is in the log

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/combat/range_targeting_test.rs` - must exist and pass

**Status**: [x] Created and passing

---

## Dependencies

- Depends on: Story 004 (RANGE-vs-WALL SS5 movement exemption; Complete)
- Depends on: Story 005 (RANGE + FIRST STRIKE SS3 behavior; Complete)
- Depends on: Story 006 (SS4 dead removal before CR-45 target reacquisition; Complete)
- Depends on: Story 007 (SS6 SHIELD and COUNTERATTACK rules; Complete)
- Unlocks: Story 009 (objective damage fires after all SS6 unit combat, including RANGE)

## Completion Notes

**Completed**: 2026-05-04
**Criteria**: 5/5 passing.
**Test Evidence**: Logic evidence at `tests/unit/combat/range_targeting_test.rs`; `cargo test -p server --test range_targeting_test` passed 6/6.
**Verification**: CR-3 single-nearest RANGE targeting attacks the nearest forward enemy without moving or consuming `RangeEquidistantSelect`; CR-3 equidistant targeting sorts the eligible set by `(distance, target_cell, target_unit_id)`, consumes exactly one `RangeEquidistantSelect` seed, and is deterministic for the same seed and ordered set. CR-4 emits distinct SS3 and SS6 `CombatDamage` entries for RANGE + FIRST STRIKE. CR-28 excludes enemies behind the attacker. CR-44 keeps the RANGE unit at its current cell and attacks the WALL from distance. CR-45 reacquires a surviving SS6 target after SS4 removes the SS3 kill.
**Regression Evidence**: `cargo test -p server --test substep6_combat_shield_counterattack_test --test substep4_dead_removal_test --test substep3_first_strike_test --test movement_collision_test` passed 20/20. `cargo check -p server` passed. `git diff --check` passed.
**Deviations**: None blocking.
**Code Review**: Skipped - lean mode.
**QA Coverage Gate**: Skipped - lean mode.
