# Story 010: Displacement Keywords and Spawn Range Expansion

> **Epic**: Board / Lane System
> **Status**: Ready
> **Layer**: Feature
> **Type**: Logic
> **Manifest Version**: 2026-04-29

## Context

**GDD**: `design/gdd/board-lane-system.md`
**Requirement**: `TR-BLS-010`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-007: Placement Buffer and Simultaneous Reveal Architecture; ADR-010: RSM Phase Event Bus
**ADR Decision Summary**: ADR-007 — Board/Lane executes displacement movements (REPEL, ATTRACT, CHANGE LANE) as spatial operations; the `apply_f1` function is reused for REPEL and ATTRACT with the appropriate direction argument from the GDD F1 table; IRREMOVABLE units silently ignore all displacement. ADR-010 — spawn range expansion (`SpawnRangeState`) is updated when a fake objective destruction event is received; Formula F2 (Story 003) reads the updated state at the next PLACEMENT phase — deferred naturally because F2 only runs during PLACEMENT.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**: IRREMOVABLE is an ECS marker component (`#[derive(Component)]`), no data needed. Displacement keyword processing uses `MessageReader` to receive keyword dispatch from Keyword System [M3] — or direct system call from Combat Resolution [M2] for now. `SpawnRangeState` update subscribes to a `FakeObjectiveDestroyed` message (ADR-010 pattern). `liv-bevy-018` mandatory.

**Control Manifest Rules (this layer)**:
- Required: Displacement keyword functions reuse `apply_f1` — do not duplicate movement arithmetic
- Required: Spawn range expansion deferred to next PLACEMENT phase — never applied mid-RESOLUTION
- Required: IRREMOVABLE check must be the first guard in every displacement function — silent discard on match
- Required: CHANGE LANE silent no-op at boundary lanes (lane 1 and lane 5) and when destination lane is full
- Forbidden: No hardcoded direction values — REPEL/ATTRACT direction computed at call site per GDD F1 table
- Guardrail: Each displacement is O(1) per keyword invocation — no iteration required

---

## Acceptance Criteria

*From GDD `design/gdd/board-lane-system.md`, scoped to this story:*

- [ ] **BL-15**: GIVEN REPEL 3 targets Player A's unit at cell 2, WHEN REPEL fires, THEN unit moves to cell 1 (clamped at spawn — does not go to cell −1).
- [ ] **BL-19**: GIVEN Strich is in lane 5 and an enemy unit enters lane 5, WHEN Strich's auto-switch fires, THEN Strich stays in lane 5 — no valid lane to switch to.
- [ ] **BL-20**: GIVEN Strich's adjacent lanes are both occupied by the player's own Minions, WHEN Strich's auto-switch fires, THEN Strich stays in its current lane.
- [ ] **BL-23**: GIVEN an IRREMOVABLE unit is targeted by REPEL, WHEN REPEL fires, THEN the unit does not move and no error is raised.
- [ ] **BL-24**: GIVEN Player B's unit (the caster) is at cell 8, and ATTRACT 5 targets Player A's unit at cell 5 (direction = sign(8 − 5) = +1 per F1 direction table), WHEN ATTRACT fires, THEN Player A's unit moves to cell 8 (clamp(5 + 5, 1, 8) = 8, clamped at board boundary).
- [ ] **BL-26**: GIVEN a fake objective is destroyed during RESOLUTION sub-step 6, WHEN the next PLACEMENT phase begins, THEN `SpawnRangeState.fakes_destroyed[PlayerA.index()]` is incremented by 1 AND the spawn range expansion does NOT apply to the current round's already-committed placements.
- [ ] **BL-28**: GIVEN any unit is in lane 1, WHEN CHANGE LANE leftward fires, THEN unit stays in lane 1 — no error raised.
- [ ] **NEW-010a**: GIVEN Player A's unit (caster) is at cell 2, and ATTRACT 3 targets Player B's unit at cell 6 (direction = sign(2 − 6) = −1), WHEN ATTRACT fires, THEN Player B's unit moves to cell 3 (clamp(6 + (−1 × 3), 1, 8) = 3).
- [ ] **NEW-010b**: GIVEN REPEL 3 targets Player B's unit at cell 7, WHEN REPEL fires, THEN unit moves to cell 8 (clamped at Player B's spawn boundary — does not overshoot past cell 8).
- [ ] **NEW-010c (BL-26 observable)**: GIVEN a fake objective is destroyed during RESOLUTION, WHEN `PlacementPhaseEntered` fires for the next round AND `validate_spawn_range` is called for PlayerA at cell 2 with the updated state, THEN the validation returns `true` (cell 2 is now valid). Verify by querying `SpawnRangeState.fakes_destroyed[PlayerA.index()]` after `PlacementPhaseEntered` — it must equal the incremented count.

---

## Implementation Notes

*Derived from ADR-007, ADR-010, and GDD Rules 9, Edge Cases:*

**REPEL implementation** (pushes target toward own spawn):
```rust
pub fn apply_repel(
    target_cell: u8,
    target_owner: PlayerId,
    repel_amount: u8,
    config: &BoardConfig,
) -> u8 {
    // REPEL direction = -advance_direction(target) per GDD F1 table
    let direction = -config.direction_for(target_owner);
    apply_f1(target_cell, direction, repel_amount, config.cell_min, config.cell_max)
}
```

**ATTRACT implementation** (pulls target toward caster):
```rust
pub fn apply_attract(
    caster_cell: u8,
    target_cell: u8,
    attract_amount: u8,
    config: &BoardConfig,
) -> u8 {
    // ATTRACT direction = sign(caster_cell - target_cell) per GDD F1 table
    // If caster and target on same cell, sign = 0 → no movement
    let diff = caster_cell as i16 - target_cell as i16;
    let direction = diff.signum();
    apply_f1(target_cell, direction, attract_amount, config.cell_min, config.cell_max)
}
```

**CHANGE LANE implementation**:
```rust
pub fn apply_change_lane(
    unit_lane: LaneId,
    delta: i8,  // +1 = right, -1 = left
    occupancy: &BoardOccupancy,
    unit_entity: Entity,
    unit_owner: PlayerId,
    config: &BoardConfig,
) -> LaneId {
    let new_lane_idx = unit_lane.index() as i8 + delta;
    // Boundary check
    if new_lane_idx < 0 || new_lane_idx >= config.lane_count as i8 {
        return unit_lane; // silent no-op
    }
    let new_lane = LaneId(new_lane_idx as u8);
    // Full slot check — if destination has player's own Minion, no-op
    if occupancy.minion_slots[unit_owner.index()][new_lane.index()].is_some() {
        return unit_lane; // silent no-op
    }
    new_lane
}
```

**IRREMOVABLE guard** — must be the first check in all displacement functions:
```rust
if query.get::<Irremovable>(target_entity).is_ok() {
    return; // silent discard
}
```

**Spawn range expansion** — subscribes to `FakeObjectiveDestroyed` message:
```rust
pub fn update_spawn_range(
    mut reader: MessageReader<FakeObjectiveDestroyed>,
    mut spawn_range: ResMut<SpawnRangeState>,
) {
    for event in reader.read() {
        let idx = event.destroyed_by.index();
        spawn_range.fakes_destroyed[idx] =
            (spawn_range.fakes_destroyed[idx] + 1).min(2);
    }
}
```

Expansion takes effect at next PLACEMENT because F2 validation (Story 003) only runs on `C2SSubmitPlacement` receipt — which only occurs during PLACEMENT phase. No additional deferral mechanism is needed; the phase gate in `handle_placement_submission` (Story 005) enforces this.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- Story 003: F2 spawn range formula (reads `SpawnRangeState` but formula logic is there)
- Keyword System [M3]: keyword dispatch, card effect parsing — Board/Lane only executes the spatial result
- Story 007: Trap trigger when displaced unit lands on a Trap — tested there

---

## QA Test Cases

*Written by qa-lead at story creation. The developer implements against these.*

- **BL-15**: REPEL clamped at Player A's spawn boundary (cell 1)
  - Given: `UnitCell(2u8)`, `PlayerOwner(PlayerA)`, REPEL amount = 3; `BoardConfig::default()` (player_a_direction=+1 → REPEL direction = -1)
  - When: `apply_repel(2, PlayerA, 3, &config)`
  - Then: returns `1u8` (clamp(2 + (-1×3), 1, 8) = clamp(-1, 1, 8) = 1)
  - Edge cases: PlayerA at cell 1, REPEL 1 → stays at 1; PlayerA at cell 5, REPEL 2 → cell 3 (no clamp needed)

- **NEW-010b**: REPEL clamped at Player B's spawn boundary (cell 8)
  - Given: `UnitCell(7u8)`, `PlayerOwner(PlayerB)`, REPEL amount = 3 (player_b_direction=-1 → REPEL direction = +1)
  - When: `apply_repel(7, PlayerB, 3, &config)`
  - Then: returns `8u8` (clamp(7 + (+1×3), 1, 8) = clamp(10, 1, 8) = 8)
  - Edge cases: PlayerB at cell 8, REPEL 1 → stays at 8

- **BL-24**: ATTRACT direction = +1 (caster cell > target cell)
  - Given: caster at cell 8; target `UnitCell(5u8)`, ATTRACT amount = 5
  - When: `apply_attract(8, 5, 5, &config)`
  - Then: returns `8u8` (direction = sign(8-5) = +1; clamp(5 + (+1×5), 1, 8) = clamp(10, 1, 8) = 8)
  - Edge cases: caster and target on same cell → sign=0 → no movement; caster at cell 3, target at cell 3 → no movement

- **NEW-010a**: ATTRACT direction = -1 (caster cell < target cell)
  - Given: caster at cell 2; target `UnitCell(6u8)`, ATTRACT amount = 3
  - When: `apply_attract(2, 6, 3, &config)`
  - Then: returns `3u8` (direction = sign(2-6) = -1; clamp(6 + (-1×3), 1, 8) = clamp(3, 1, 8) = 3)
  - Edge cases: ATTRACT 10 with direction = -1 from cell 6 → clamp to 1

- **BL-23**: IRREMOVABLE unit silently ignores REPEL
  - Given: entity `unit` with `Irremovable` marker component, `UnitCell(4u8)`, `PlayerOwner(PlayerA)`
  - When: REPEL targets `unit`; IRREMOVABLE guard fires
  - Then: `UnitCell` unchanged (still `4u8`); no error raised; no `TrapTrigger` from displacement
  - Edge cases: IRREMOVABLE unit targeted by ATTRACT → also no movement; IRREMOVABLE + CHANGE LANE → also no movement

- **BL-28**: CHANGE LANE leftward from lane 1 is a silent no-op
  - Given: entity `unit` in `LaneId(Lane(1))`; CHANGE LANE delta = -1
  - When: `apply_change_lane(Lane(1), -1, &occupancy, entity, PlayerA, &config)`
  - Then: returns `Lane(1)` (unchanged); no error raised
  - Edge cases: lane 5, delta = +1 → stays at lane 5; lane 3, delta = +1 → moves to lane 4 (not blocked)

- **BL-19 / BL-20**: Strich CHANGE LANE blocked by boundary or full adjacent slots
  - Given (BL-19): Strich in Lane(5); CHANGE LANE fires (delta = +1)
  - Then: lane 5 + 1 = 6 > 5 lanes → boundary check → no-op (BL-19 satisfied via BL-28 logic)
  - Given (BL-20): Strich in Lane(3); both Lane(2) and Lane(4) have PlayerA's Minion
  - When: CHANGE LANE delta = -1 targets Lane(2) → occupancy check: `minion_slots[PlayerA][Lane(2)] = Some(e)` → blocked
  - Then: stays in Lane(3); no error

- **NEW-010c / BL-26**: Spawn range expansion deferred to next PLACEMENT
  - Given: `World::new()` with `SpawnRangeState { fakes_destroyed: [0, 0] }`; `FakeObjectiveDestroyed { destroyed_by: PlayerA }` message written
  - When: `update_spawn_range` system runs
  - Then: `SpawnRangeState.fakes_destroyed[PlayerA.index()] = 1`
  - When: `validate_spawn_range(2, PlayerA, 1)` called (simulating next PLACEMENT check)
  - Then: returns `true` (cell 2 now valid with 1 fake destroyed)
  - When: `validate_spawn_range(2, PlayerA, 0)` called (simulating current-round check with old count)
  - Then: returns `false` (confirms expansion doesn't retroactively affect already-committed placements)

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/board-lane-system/displacement_keywords_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 002 must be DONE (`apply_f1` function)
- Unlocks: Nothing in this epic — terminal for displacement keyword execution
