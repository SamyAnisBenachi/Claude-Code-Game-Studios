# Story 002: Standard Unit Movement Formula (F1)

> **Epic**: Board / Lane System
> **Status**: Complete
> **Layer**: Feature
> **Type**: Logic
> **Manifest Version**: 2026-04-29

## Context

**GDD**: `design/gdd/board-lane-system.md`
**Requirements**: `TR-BLS-001`, `TR-BLS-002`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-007: Placement Buffer and Simultaneous Reveal Architecture
**ADR Decision Summary**: Board/Lane owns the spatial execution layer — the `apply_standard_movement` system applies Formula F1 to advance all surviving units each round during sub-step 5 of RESOLUTION; this ADR governs the board module's file layout and system ordering.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**: Use `Query<(&mut UnitCell, &MovementPoints, &PlayerOwner)>` — no Bundle-based access. `query.iter_mut()` is the correct iteration pattern. `query.single()` returns `Result` in Bevy 0.16+ — use `let Ok(...)` pattern. `liv-bevy-018` mandatory.

**Control Manifest Rules (this layer)**:
- Required: All board movement systems in `server/src/feature/board/movement.rs`
- Required: No `unwrap()` in production paths — use `?` or `expect("message")`
- Forbidden: No hardcoded balance values — direction constants read from `BoardConfig` resource, not inline literals
- Guardrail: Movement system applies F1 to all units per lane per sub-step — O(units). Must complete in single frame well within the 2ms game logic budget.

---

## Acceptance Criteria

*From GDD `design/gdd/board-lane-system.md`, scoped to this story:*

- [x] **BL-1**: GIVEN Player A's unit is in lane 1 at cell 1 with MP=3, WHEN sub-step 5 fires, THEN unit position = cell 4.
- [x] **BL-2**: GIVEN Player A's unit is in lane 1 at cell 6 with MP=3, WHEN sub-step 5 fires, THEN unit position = cell 8 (clamped — does not overshoot).
- [x] **BL-3**: GIVEN Player B's unit is in lane 1 at cell 5 with MP=2, WHEN sub-step 5 fires, THEN unit position = cell 3.
- [x] **BL-4**: GIVEN Player A's WALL unit (MP=0) is in lane 1 at cell 1, WHEN sub-step 5 fires, THEN unit position = cell 1 (no movement).

---

## Implementation Notes

*Derived from ADR-007 Implementation Guidelines and GDD Formula F1:*

Implement the movement formula in `server/src/feature/board/movement.rs`:

```rust
/// Applies Formula F1 to compute the new cell after one movement sub-step.
/// Cast to i16 before arithmetic to prevent u8 underflow/overflow panics
/// on boundary values (e.g., Player B at cell 1 with MP=3 → i16: 1 - 3 = -2, clamped to 1).
pub fn apply_f1(current_cell: u8, direction: i16, mp: u8, cell_min: u8, cell_max: u8) -> u8 {
    let new_cell = current_cell as i16 + direction * mp as i16;
    new_cell.clamp(cell_min as i16, cell_max as i16) as u8
}
```

The `apply_standard_movement` system calls `apply_f1` with:
- `direction` = `board_config.player_a_direction` (+1) for PlayerA units
- `direction` = `board_config.player_b_direction` (-1) for PlayerB units
- `cell_min = board_config.cell_min` (1), `cell_max = board_config.cell_max` (8)

This same function is reused by:
- Story 006: CHARGE X bonus movement (sub-step 2) — pass same direction, CHARGE amount as `mp`
- Story 010: Displacement keywords — REPEL passes `-advance_direction(target.owner)`, ATTRACT passes `sign(caster_cell - target_cell)`

Direction argument by use case (GDD Formula F1 table):

| Use case | `direction` argument |
|---|---|
| Standard movement (sub-step 5) | `advance_direction(unit.owner)` |
| CHARGE X bonus movement (sub-step 2) | `advance_direction(unit.owner)` |
| REPEL X | `−advance_direction(target.owner)` |
| ATTRACT X | `sign(caster_cell − target_cell)` |

Units at the objective cell (cell 8 for PlayerA, cell 1 for PlayerB) remain there — the clamp handles this naturally.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- Story 006: CHARGE X sub-step 2 movement (uses `apply_f1` but fires at sub-step 2, not sub-step 5)
- Story 010: REPEL, ATTRACT, CHANGE LANE displacement (direction variants of `apply_f1`)
- Story 008: Objective cell detection — fires after units have moved, not part of movement itself

---

## QA Test Cases

*Written by qa-lead at story creation. The developer implements against these.*

- **BL-1**: Player A forward movement, no clamp
  - Given: `World::new()`; entity `unit` with `UnitCell(1u8)`, `MovementPoints(3i16)`, `PlayerOwner(PlayerA)`; `BoardConfig::default()` inserted as resource
  - When: `apply_standard_movement` system runs
  - Then: `UnitCell` on `unit` = `4u8`
  - Edge cases: Starting cell=5, MP=3 → cell=8 (upper clamp, BL-2); starting cell=8, MP=1 → cell=8 (at boundary)

- **BL-2**: Player A movement clamped at cell 8
  - Given: `UnitCell(6u8)`, `MovementPoints(3i16)`, `PlayerOwner(PlayerA)`, `BoardConfig::default()`
  - When: `apply_standard_movement` system runs
  - Then: `UnitCell` = `8u8` (6+3=9, clamped to 8)
  - Edge cases: Starting cell=7, MP=1 → cell=8 (exact boundary, not clamped)

- **BL-3**: Player B negative-direction movement with i16 intermediate
  - Given: `UnitCell(5u8)`, `MovementPoints(2i16)`, `PlayerOwner(PlayerB)`, `BoardConfig::default()` (player_b_direction=-1)
  - When: `apply_standard_movement` system runs
  - Then: `UnitCell` = `3u8`
  - Edge cases: Starting cell=3, MP=5, direction=-1 → i16 intermediate = -2, clamped to 1 (proves i16 required); starting cell=1, MP=1 → stays at 1 (no u8 underflow)

- **BL-4**: WALL unit (MP=0) does not move
  - Given: `UnitCell(1u8)`, `MovementPoints(0i16)`, `PlayerOwner(PlayerA)`
  - When: `apply_standard_movement` system runs
  - Then: `UnitCell` = `1u8`
  - Edge cases: PlayerB WALL at cell 8, MP=0 → stays at 8; any cell, MP=0 → unchanged

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/board-lane-system/standard_movement_test.rs` — must exist and pass

**Status**: [x] Created and passed locally with `cargo test -p server --test standard_movement_test`

---

## Dependencies

- Depends on: Story 001 must be DONE (`BoardConfig` resource with direction constants)
- Unlocks: Stories 006, 008, 009, 010 (all use `apply_f1` or the movement endpoint)

## Completion Notes

**Completed**: 2026-05-01
**Criteria**: 4/4 passing. BL-1, BL-2, BL-3, and BL-4 are covered by `tests/unit/board-lane-system/standard_movement_test.rs`; the suite also covers the i16 intermediate clamp boundary for Player B.
**Deviations**: Advisory only: story manifest v2026-04-29 is older than current control manifest v2026-05-01. Advisory only: implementation uses the current project ECS components (`BoardPosition`, `UnitStats`, `UnitOwner`) instead of the story's older placeholder component names; Formula F1 behavior and `BoardConfig` direction/bounds usage match the current GDD and TR registry.
**Test Evidence**: Logic: `tests/unit/board-lane-system/standard_movement_test.rs`; `cargo test -p server --test standard_movement_test` passed 5/5. `cargo check -p server` passed.
**Code Review**: Skipped - Lean mode.
