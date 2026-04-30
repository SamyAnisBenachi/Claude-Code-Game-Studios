# Story 008: Sub-step 6 — RANGE Targeting

> **Epic**: Combat Resolution
> **Status**: Ready
> **Layer**: Feature
> **Type**: Logic
> **Manifest Version**: 2026-04-30

## Context

**GDD**: `design/gdd/combat-resolution.md`
**Requirement**: `TR-CR-???` (TR-CR-009 partial — unregistered)

**ADR Governing Implementation**: ADR-017: Combat Resolution Execution Architecture
**ADR Decision Summary**: RANGE units attack the nearest enemy in the forward direction without advancing to that cell. RANGE + FIRST STRIKE units fire in SS3 AND again in SS6 (two independent damage events). Equidistant target selection uses the `range_equidistant_select` RNG seed slot from `ServerRng`. RANGE attackers at distance cannot trigger COUNTERATTACK on the target.

**Engine**: Bevy 0.18 | **Risk**: HIGH
**Engine Notes**: RANGE targeting formula: `valid_targets = enemy units at cells [C+1..C+RANGE_X]` (Player A) or `[C-RANGE_X..C-1]` (Player B); clamped to `[1, 8]`. RNG seed consumed via `ServerRng::next_seed(RngEvent::RangeEquidistantSelect)` only when equidistant targets exist (0 seeds consumed when single nearest target).

**Control Manifest Rules (Feature layer)**:
- Required: RANGE targeting checks `[C+1..C+X]` forward only; equidistant selection consumes exactly 1 RNG seed; RANGE units do NOT advance to target cell; `range_equidistant_select` seed consumed in RNG consumption order
- Forbidden: Never target friendly units; never target cells behind the RANGE unit; never consume RNG seed if only one nearest target exists

---

## Acceptance Criteria

*From GDD `design/gdd/combat-resolution.md`, scoped to this story:*

- [ ] **CR-3**: GIVEN a unit with RANGE 1-X at cell C with a single nearest enemy, WHEN sub-step 6 executes, THEN it attacks that nearest enemy (Player A: minimum cell-distance in cells C+1 to C+X; Player B: C-X to C-1); it does not advance to do so. Equidistant target selection (ADVISORY until `range_equidistant_select` seed wired; BLOCKING for single-nearest case)
- [ ] **CR-4**: GIVEN a unit with RANGE 1-X AND FIRST STRIKE, WHEN RESOLUTION executes, THEN two distinct `CombatDamage` entries are in the ResolutionLog: one with `sub_step: 3` and one with `sub_step: 6`
- [ ] **CR-28**: GIVEN a RANGE unit with enemies both forward and behind it (both within numeric RANGE value), WHEN sub-step 6 executes, THEN only the forward enemy is targeted; the enemy behind is never a valid RANGE target
- [ ] **CR-44**: GIVEN a RANGE 1-3 unit at cell C with a WALL unit at cell C+2 (within RANGE), WHEN sub-step 5 executes, THEN the RANGE unit's cell position is unchanged; WHEN sub-step 6 executes, THEN the RANGE unit attacks the WALL from cell C and a `CombatDamage` record is emitted with defender = WALL
- [ ] **CR-45**: GIVEN a RANGE + FIRST STRIKE unit kills its sub-step 3 target, AND a different enemy unit exists within range at sub-step 6 entry, WHEN sub-step 4 removes the killed unit AND sub-step 6 executes, THEN the RANGE unit acquires the surviving enemy as its sub-step 6 target and a `CombatDamage` record is emitted for that target

---

## Implementation Notes

*Derived from ADR-017 and GDD Formula 4 (RANGE Target Selection):*

```rust
/// RANGE target selection formula (Formula 4 from GDD).
fn select_range_target(
    attacker: &UnitSnapshot,
    board: &BoardState,
    rng: &mut ServerRng,
) -> Option<UnitId> {
    let range_x = attacker.range_value; // must be >= 1

    // Forward direction only
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

    // Collect enemy units within [cell_min, cell_max]
    let mut candidates: Vec<(UnitId, u8)> = board.enemy_units_in_range(
        attacker.player, cell_min, cell_max
    ).map(|u| (u.unit_id, u.cell)).collect();

    if candidates.is_empty() { return None; }

    // Nearest first
    candidates.sort_by_key(|(_, cell)| cell.abs_diff(attacker.cell));
    let min_dist = candidates[0].1.abs_diff(attacker.cell);
    let equidistant: Vec<UnitId> = candidates.iter()
        .filter(|(_, cell)| cell.abs_diff(attacker.cell) == min_dist)
        .map(|(id, _)| *id)
        .collect();

    if equidistant.len() == 1 {
        Some(equidistant[0])  // No RNG consumed
    } else {
        // Consume 1 RNG seed for equidistant selection
        let seed = rng.next_seed(RngEvent::RangeEquidistantSelect);
        let idx = (seed as usize) % equidistant.len();
        Some(equidistant[idx])
    }
}
```

**CR-45 (RANGE+FS target reacquisition)**: At SS6 entry, call `select_range_target` fresh — do NOT cache the SS3 target. If the SS3 target was removed in SS4, a new target is selected. If no target exists within range, no SS6 attack occurs (null return from `select_range_target`).

**CR-44 (RANGE attacks WALL)**: WALL units are valid targets for RANGE. Include WALLs in `board.enemy_units_in_range()`. The RANGE unit's cell does not change in SS5 (handled by Story 004's movement exemption for RANGE units).

**CR-28 (forward-only)**: The `cell_min/cell_max` computation strictly uses cells AHEAD of the attacker's position. For Player A (advances in +direction), cells behind are `< attacker.cell` — excluded. For Player B, cells behind are `> attacker.cell` — excluded.

---

## Out of Scope

- Story 005: SS3 FIRST STRIKE attack (RANGE+FS fires there; this story handles the SS6 attack)
- Story 007: SHIELD pre-check and COUNTERATTACK (applies to RANGE attacks in SS6 the same way, but RANGE at distance cannot trigger COUNTERATTACK — enforced in Story 007)
- Story 009: Objective damage at Cell 8 (RANGE unit at Cell 8 cannot attack forward; governed by objective damage rule, not RANGE targeting)

---

## QA Test Cases

*(Lean mode — test cases authored inline)*

- **CR-3** (nearest target selection):
  - Given: RANGE 3 unit at cell 2 (Player A); enemies at cell 4 (nearer) and cell 5 (farther)
  - When: SS6 RANGE targeting runs
  - Then: attack targets cell 4 unit; no RNG consumed (single nearest)

- **CR-4** (RANGE+FS two damage events):
  - Given: RANGE 2 + FS unit at cell 3; enemy at cell 5
  - When: RESOLUTION completes
  - Then: log contains `CombatDamage { sub_step: 3 }` and `CombatDamage { sub_step: 6 }` for same attacker-defender pair

- **CR-28** (forward-only):
  - Given: RANGE 3 unit (Player A) at cell 4; enemy at cell 2 (behind) and enemy at cell 6 (forward)
  - When: SS6 targeting runs
  - Then: only cell 6 enemy targeted; cell 2 enemy never selected

- **CR-44** (RANGE attacks WALL from distance):
  - Given: RANGE 3 unit at cell 2 (MP=0, position unchanged); WALL at cell 4
  - When: SS6 runs
  - Then: `CombatDamage { attacker: RANGE_unit, defender: WALL, sub_step: 6 }` in log

- **CR-45** (target reacquisition after SS3 kill):
  - Given: RANGE+FS unit; target A killed in SS3, target B exists at cell C+3 in range
  - When: SS4 removes A; SS6 runs
  - Then: `CombatDamage { defender: B, sub_step: 6 }` in log

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/combat/range_targeting_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 007 (RANGE attacks are part of SS6 combat execution, sharing SHIELD/COUNTERATTACK rules from 007)
- Unlocks: Story 009 (objective damage fires after all SS6 unit combat, including RANGE)
