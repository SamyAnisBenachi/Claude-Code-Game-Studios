# Story 004: Sub-steps 2 & 5 — Movement + Collision

> **Epic**: Combat Resolution
> **Status**: Complete
> **Layer**: Feature
> **Type**: Logic
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/combat-resolution.md`
**Requirement**: `TR-CR-016` (CR-5 STUN action suppression; this story verifies the SS2/SS5 movement clauses), `TR-CR-017` (CR-8 SS5 WALL halt movement clause only), `TR-CR-003` (CR-9 SS5 path-crossing halt movement clause only), `TR-CR-018` (CR-31 CHARGE X plus standard movement), `TR-CR-019` (CR-44 SS5 RANGE-vs-WALL movement exemption only)

**ADR Governing Implementation**: ADR-017: Combat Resolution Execution Architecture (Decision 2 — Movement Collision Boundary)
**ADR Decision Summary**: Two movement rules coexist. Rule A (destination rule): F1 formula computes each unit's intended destination once at SS5 entry — this governs Trap/Prism triggering. Rule B (collision loop): a per-tick advance loop inside `execute_movement` determines actual final position after enemy obstruction — this governs WALL halts and path-crossing halts. These rules are complementary layers, not contradictory. CHARGE X (SS2) uses the same tick loop as SS5 but with the CHARGE_X value instead of MP.

**Engine**: Bevy 0.18 | **Risk**: HIGH
**Engine Notes**: All position arithmetic must use `i16` intermediate before clamping to cell range `[1, 8]` — u8 subtraction for Player B movement would underflow. `world.resource_mut::<BoardState>()` is the correct World accessor (stable since Bevy 0.12). No breaking changes to `World::resource_mut` through Bevy 0.18.

**Control Manifest Rules (Feature layer)**:
- Required: `resolve_combat(world: &mut World)` remains one Bevy exclusive-system frame; stats snapshots are immutable for the full RESOLUTION; movement formula F1 uses `i16` intermediate arithmetic.
- Required: Current movement boundary is two-layered: F1 destination governs Trap/Prism triggering, while the enemy collision tick loop governs obstruction such as WALL halt and path crossing.
- Forbidden: Never break RESOLUTION into a multi-frame state machine; never stream per-sub-step S2C events; never use `unit.mp` for CHARGE X movement (use the CHARGE X value).
- Guardrail: Combat resolution worst case stays <= 15 ms; tick loop remains bounded by `iter_count` and must terminate pathological bounce loops.

---

## Acceptance Criteria

*From GDD `design/gdd/combat-resolution.md`, scoped to this story:*

- [ ] **CR-5**: GIVEN a STUNned unit (including a CHARGE unit STUNned in sub-step 1), WHEN RESOLUTION executes, THEN the unit does not advance in sub-step 2 (CHARGE X suppressed) and does not advance in sub-step 5
- [ ] **CR-8**: GIVEN an advancing enemy unit whose next step would reach a WALL unit's cell, WHEN sub-step 5 movement executes, THEN the attacker halts at the WALL's cell and records the SS5 movement result. This story does not verify SS6 damage to the WALL, WALL death processing, or CombatDamage events.
- [ ] **CR-9**: GIVEN two enemy units whose paths would cross in sub-step 5 (each moving to the other's cell in the same tick), WHEN sub-step 5 movement executes, THEN both halt at their pre-crossing cells (adjacent facing). This story does not verify the later SS6 fight.
- [ ] **CR-31**: GIVEN a unit with CHARGE X, WHEN sub-step 2 executes, THEN the unit advances X additional cells (subject to WALL-blocking and crossing rules); WHEN sub-step 5 executes, THEN the unit additionally advances its MP value as a separate standard movement
- [ ] **CR-44**: GIVEN a RANGE 1-3 unit at cell C with a WALL unit at cell C+2 (within RANGE), WHEN sub-step 5 movement executes, THEN the RANGE unit's cell position is unchanged and it does NOT halt at or advance toward the WALL. This story does not verify SS6 RANGE targeting, WALL damage, or CombatDamage events.

---

## Implementation Notes

*Derived from ADR-017 Decision 2 and GDD Sub-step 5 rules:*

```rust
fn execute_movement(
    board: &mut BoardState,
    snapshots: &mut Vec<UnitSnapshot>,
    log: &mut ResolutionLog,
    iter_count: &mut u32,
    charge_x_mode: bool,  // true = SS2 CHARGE X, false = SS5 standard
) {
    // 1. Compute each unit's destination ONCE (Rule A — destination rule)
    //    Use CHARGE_X value in SS2, MP in SS5
    for unit in snapshots.iter_mut() {
        if unit.is_stunned { continue; }
        let move_value = if charge_x_mode { unit.charge_x } else { unit.mp };
        if move_value == 0 { continue; }  // MP=0 excluded from tick loop
        unit.destination = clamp_i16(
            unit.cell as i16 + unit.direction as i16 * move_value as i16,
            1, 8
        ) as u8;
    }

    // 2. Tick loop — advance 1 cell per tick until all reach destination or halt
    loop {
        let mut any_moved = false;
        // Advance all non-stunned, non-halted units by 1 cell
        // Check WALL halt: if next_cell == enemy WALL cell → halt
        // Check path-cross: if A→B cell and B→A cell in same tick → both halt at current
        // Check same-cell: two enemies both arrive at X → both land; SS6 combat is later story scope
        // Increment iter_count for each unit-tick step
        *iter_count += moving_units.len() as u32;
        if *iter_count > 10_000 { return Err(IterationBudgetExceeded); }
        if !any_moved { break; }
    }
}
```

**RANGE unit + WALL exception (CR-44)**: Before the tick loop, check if a RANGE unit's target (WALL) is already within its range from its current cell. If yes, mark this RANGE unit's destination as its CURRENT cell (it does not advance). SS6 RANGE targeting and CombatDamage emission are owned by later combat stories.

**F1 destination formula** (`clamp_i16`): compute `current_cell as i16 + direction * mp as i16`, then clamp to `[1i16, 8i16]`, then cast to `u8`. Player A direction = +1, Player B direction = -1.

**SS2 CHARGE X behavior**: CHARGE X (movement keyword — `CHARGE X` advances X cells) is DISTINCT from the CHARGE combat keyword (removes summoning sickness). SS2 runs only for units with a numeric CHARGE X value. STUNned units skip SS2 entirely.

---

## Out of Scope

- Story 003: SS1 populates board state and applies STUN (which this story checks)
- Story 005: SS3 FIRST STRIKE attacks (units damaged in SS3 may affect SS5 via HP changes, but STUN from SS3 doesn't apply to SS5 — STUN must be applied before SS2)
- COMBAT-007: SS6 standard combat, including WALL attacks/damage, adjacent collision-halt fights, SHIELD, and COUNTERATTACK
- COMBAT-008: SS6 RANGE target selection, including RANGE-vs-WALL attacks from the RANGE unit's current cell
- COMBAT-011: ResolutionEvent log completeness, including CombatDamage event emission and ordering

---

## QA Test Cases

*(Lean mode — test cases authored inline)*

- **CR-5** (STUN suppresses movement):
  - Given: Unit with STUN flag, CHARGE X=2, MP=2
  - When: SS2 then SS5 execute
  - Then: unit cell position unchanged after both sub-steps

- **CR-8** (WALL halt):
  - Given: Melee unit at cell 3 (Player A, MP=3, destination=6); WALL at cell 5 (Player B)
  - When: SS5 tick loop runs
  - Then: melee unit halts at cell 5 (WALL's cell); `UnitMoved { to_cell: 5 }` logged
  - Edge case: WALL at cell 4 (immediately adjacent) → halts on tick 1

- **CR-9** (path-crossing halt):
  - Given: Unit A at cell 4 (Player A, MP=2); Unit B at cell 5 (Player B, MP=2) — would swap
  - When: SS5 tick 1
  - Then: both halt at cells 4 and 5 (adjacent facing); SS6 fight is not asserted in this story

- **CR-31** (CHARGE X + standard movement separate):
  - Given: Unit with CHARGE X=2, MP=1 at cell 2 (Player A)
  - When: SS2 executes → cell becomes 4; SS5 executes → cell becomes 5
  - Then: two separate `UnitMoved` entries in log (one in SS2, one in SS5)

- **CR-44** (RANGE unit not halted by WALL):
  - Given: RANGE 3 unit at cell C=2 (Player A, MP=0); WALL at cell 4 (Player B)
  - When: SS5 executes
  - Then: RANGE unit cell position unchanged (MP=0, no advancement); no WALL-halt event in log for this unit
  - Out of scope: SS6 RANGE attack and CombatDamage emission are covered by COMBAT-008 and COMBAT-011

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/combat/movement_collision_test.rs` — must exist and pass

**Status**: [x] Created and passing

---

## Dependencies

- Depends on: Story 001 (function called from `resolve_combat`), Story 003 (board state populated by SS1)
- Unlocks: Story 005 (SS3 FIRST STRIKE uses post-SS2 positions), Story 007 (SS6 combat uses post-SS5 positions)

---

## Completion Notes

**Completed**: 2026-05-04
**Criteria**: 5/5 passing (CR-5, CR-8, CR-9, CR-31, CR-44 movement/collision clauses)
**Deviations**: Advisory only - the current GDD/TR text for CR-8, CR-9, and CR-44 also references later SS6 attack, damage, death-processing, and CombatDamage behavior. Those clauses are explicitly out of scope for this story and remain owned by COMBAT-007, COMBAT-008, and COMBAT-011; they are not blockers for COMBAT-004 closure.
**Test Evidence**: Logic: `tests/unit/combat/movement_collision_test.rs` exists and passed 5/5 via `cargo test -p server --test movement_collision_test`.
**Code Review**: Skipped - lean mode.
