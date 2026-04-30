# Story 006: CHARGE X Bonus Movement and Intermediate Cell Skip

> **Epic**: Board / Lane System
> **Status**: Ready
> **Layer**: Feature
> **Type**: Logic
> **Manifest Version**: 2026-04-29

## Context

**GDD**: `design/gdd/board-lane-system.md`
**Requirement**: `TR-BLS-006`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-007: Placement Buffer and Simultaneous Reveal Architecture
**ADR Decision Summary**: Board/Lane owns sub-step execution during RESOLUTION — sub-step 2 (CHARGE X bonus movement) fires before standard movement (sub-step 5); the same `apply_f1` formula used in Story 002 is applied independently at each sub-step using the unit's current cell as input; intermediate cells are never occupied.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**: Query for `ChargeBonus` component (new ECS component, Bevy 0.18 Required Components API — no Bundle). Units without `ChargeBonus` skip sub-step 2. `query.iter_mut()` is the correct iteration pattern. `liv-bevy-018` mandatory.

**Control Manifest Rules (this layer)**:
- Required: `apply_f1` from Story 002 is the shared formula — do not duplicate movement arithmetic; call `apply_f1` with the unit's current cell and the CHARGE amount as `mp`
- Required: No hardcoded values — CHARGE bonus is read from `ChargeBonus` ECS component, not inlined
- Forbidden: No intermediate cell occupancy — only the final destination cell of each sub-step is treated as occupied; Trap checks (Story 007) only run at final destination
- Guardrail: CHARGE X pass runs per-unit per-lane per-sub-step — O(units with CHARGE). No performance concern at hackathon scale.

---

## Acceptance Criteria

*From GDD `design/gdd/board-lane-system.md`, scoped to this story:*

- [ ] **BL-22**: GIVEN a unit entity in lane 1 with `ChargeBonus(2)` and `MovementPoints(3)` components at cell 1 (set via direct ECS World state, not card pool lookup), WHEN resolution runs, THEN unit ends sub-step 2 at cell 3 AND ends sub-step 5 at cell 6. F1 is applied independently at each sub-step using the unit's current cell as input.
- [ ] **BL-27**: GIVEN an enemy Trap is at cell 3 and Player A's unit is at cell 1 with MP=3, WHEN sub-step 5 fires, THEN the unit moves to cell 4 and the Trap at cell 3 does NOT trigger.
- [ ] **BL-27b**: GIVEN an enemy Trap is at (lane 1, cell 2) and Player A's unit is in lane 1 at cell 1 with CHARGE 3, WHEN sub-step 2 fires, THEN the unit moves to cell 4 and the Trap at cell 2 does NOT trigger (CHARGE X movement skips intermediate cells, same as standard movement).

---

## Implementation Notes

*Derived from ADR-007 system scheduling and GDD Rule 7, Edge Cases:*

**Sub-step 2 system** (`apply_charge_movement`) in `server/src/feature/board/movement.rs`:

```rust
/// Sub-step 2: CHARGE X bonus movement.
/// Only fires for units with the ChargeBonus component (units without it skip this sub-step).
/// Uses apply_f1 with the unit's current cell — NOT the cell from sub-step 5.
/// Final cell written back to UnitCell; sub-step 5 then reads this updated value.
pub fn apply_charge_movement(
    mut query: Query<(&mut UnitCell, &ChargeBonus, &PlayerOwner)>,
    board_config: Res<BoardConfig>,
) {
    for (mut cell, charge, owner) in query.iter_mut() {
        let direction = board_config.direction_for(owner.0);
        cell.0 = apply_f1(cell.0, direction, charge.0, board_config.cell_min, board_config.cell_max);
    }
}
```

**Intermediate cell skip rule**: Movement teleports the unit directly to its final destination cell. The unit never "occupies" intermediate cells at any point during sub-step 2 or sub-step 5. Trap trigger checks (Story 007) are called only at the final destination after each sub-step completes — not at each intermediate cell traversed.

**A unit with both CHARGE X and MP>0**:
- Sub-step 2: CHARGE advances unit from current_cell by X cells
- Sub-step 5: standard movement advances unit from the CHARGE-updated current_cell by MP cells
- Both F1 applications are independent; the second one reads the cell value written by the first

**Units without `ChargeBonus`** skip sub-step 2 entirely (the query simply yields no results for them). WALL units (MP=0) still participate in sub-step 5 but produce no movement.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- Story 007: Trap trigger at final destination of CHARGE X landing (positive trigger case) — this story only verifies non-trigger at intermediate cells
- Story 002: Standard movement sub-step 5 (tested separately; `apply_f1` is shared)
- Story 009: Prism collection — no collection fires from sub-step 2 CHARGE movement

---

## QA Test Cases

*Written by qa-lead at story creation. The developer implements against these.*

- **BL-22**: CHARGE X advances at sub-step 2; standard MP advances at sub-step 5 (independent F1 applications)
  - Given: `World::new()`; entity `unit` with `UnitCell(1u8)`, `ChargeBonus(2i16)`, `MovementPoints(3i16)`, `PlayerOwner(PlayerA)`, `LaneId(Lane(1))`; `BoardConfig::default()`
  - When: `apply_charge_movement` system runs (sub-step 2)
  - Then: `UnitCell` = `3u8` (1 + 2)
  - When: `apply_standard_movement` system runs (sub-step 5), reading the updated cell
  - Then: `UnitCell` = `6u8` (3 + 3)
  - Edge cases: `ChargeBonus(5)` + `MP(3)` from cell 1 → sub-step 2 = cell 6, sub-step 5 = cell 8 (clamped); `ChargeBonus(7)` from cell 1 → sub-step 2 = cell 8 (clamped); sub-step 5 MP=3 from 8 → stays 8

- **BL-27**: Standard movement final cell skips intermediate Trap
  - Given: entity `unit` for PlayerA at `(Lane(1), Cell(1))`, `MP(3i16)`; `OccupancyMap` shows enemy Trap for PlayerB at `(Lane(1), Cell(3))`
  - When: `apply_standard_movement` runs; Trap check runs at final destination only
  - Then: `UnitCell` = `4u8`; Trap at `(Lane(1), Cell(3))` still present; no `TrapTrigger` event emitted
  - Edge cases: Trap at destination cell 4 → DOES trigger (landing, not skip); Trap at cell 2 → also not triggered

- **BL-27b**: CHARGE X final cell skips intermediate Trap
  - Given: entity `unit` for PlayerA at `(Lane(1), Cell(1))`, `ChargeBonus(3i16)`; enemy Trap at `(Lane(1), Cell(2))`
  - When: `apply_charge_movement` runs; Trap check at final destination only
  - Then: `UnitCell` = `4u8` (1 + 3); Trap at `(Lane(1), Cell(2))` still present; no trigger
  - Edge cases: Trap at CHARGE landing cell 4 → DOES trigger; multiple Traps at cells 2 and 3 with CHARGE 4 → neither triggers (both intermediate to landing at 5)

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/board-lane-system/charge_movement_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 002 must be DONE (`apply_f1` function implemented)
- Unlocks: Stories 007 (CHARGE X landing Trap trigger), 009 (CHARGE pass prism non-collection BL-30)
