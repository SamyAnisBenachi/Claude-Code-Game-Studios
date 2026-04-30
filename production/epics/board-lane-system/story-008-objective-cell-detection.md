# Story 008: Objective Cell Detection (F3)

> **Epic**: Board / Lane System
> **Status**: Ready
> **Layer**: Feature
> **Type**: Logic
> **Manifest Version**: 2026-04-29

## Context

**GDD**: `design/gdd/board-lane-system.md`
**Requirement**: `TR-BLS-008`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-010: RSM Phase Event Bus — Phase Message Catalog and Subscriber Contracts
**ADR Decision Summary**: Board/Lane emits `UnitAtObjective(unit_id, lane)` at sub-step 6 end by writing a buffered `MessageWriter`; the Objective System [M2] is the downstream consumer; Board/Lane does NOT own objective HP and does NOT detect destruction — it only detects presence at the objective cell.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**: `MessageWriter<UnitAtObjective>` (buffered, Bevy 0.17+ API). `EventWriter`/`EventReader` do not exist in Bevy 0.17+. `UnitAtObjective` derives `Message`, `Clone`, `Debug`. Register via `app.add_message::<UnitAtObjective>()`. `liv-bevy-018` mandatory.

**Control Manifest Rules (this layer)**:
- Required: Use `MessageWriter::write()` — `EventWriter` no longer exists in Bevy 0.17+
- Required: Feature systems communicate upward via events — `UnitAtObjective` is emitted by Board/Lane, consumed by Objective System; Board/Lane does not call Objective System directly
- Required: Register `UnitAtObjective` via `app.add_message::<UnitAtObjective>()` — not `app.add_event::<T>()`
- Guardrail: F3 check is O(units) at end of sub-step 6 — negligible

---

## Acceptance Criteria

*From GDD `design/gdd/board-lane-system.md`, scoped to this story:*

- [ ] **BL-10**: GIVEN Player A's unit is at cell 8 at end of sub-step 6, WHEN sub-step 6 completes, THEN the Board emits `UnitAtObjective(unit_id, lane)` exactly once for that unit.
- [ ] **BL-11**: GIVEN Player A's unit at cell 8 survives round N, WHEN round N+1 sub-step 6 fires, THEN the unit is still at cell 8 and attacks the objective again (UnitAtObjective fires again for round N+1).
- [ ] **BL-25**: GIVEN Player B's unit is at cell 1 at end of sub-step 6, WHEN sub-step 6 completes, THEN the Board emits `UnitAtObjective(unit_id, lane)` for Player B's unit.

---

## Implementation Notes

*Derived from ADR-010 Subscriber Contracts and GDD Formula F3, Rule 7:*

**Formula F3** (implemented in `server/src/feature/board/objective.rs`):

```rust
/// Returns true if the unit is at its owner's objective cell.
/// Player A objective = cell 8; Player B objective = cell 1.
pub fn is_at_objective(owner: PlayerId, cell: u8, config: &BoardConfig) -> bool {
    match owner {
        PlayerId::A => cell == config.player_a_objective_cell,  // 8
        PlayerId::B => cell == config.player_b_objective_cell,  // 1
    }
}
```

**detect_objective_presence system** runs at end of sub-step 6:

```rust
pub fn detect_objective_presence(
    query: Query<(Entity, &UnitCell, &PlayerOwner, &LaneId)>,
    config: Res<BoardConfig>,
    mut writer: MessageWriter<UnitAtObjective>,
) {
    for (entity, cell, owner, lane) in query.iter() {
        if is_at_objective(owner.0, cell.0, &config) {
            writer.write(UnitAtObjective { unit_id: entity, lane: lane.0 });
        }
    }
}
```

Units at the objective cell remain there until killed (standard movement via F1 clamps to cell 8/1 without overshooting). No special handling needed — the unit entity persists between rounds, and if it is still alive and still at the objective cell at sub-step 6 of round N+1, `UnitAtObjective` fires again naturally.

`UnitAtObjective` is NOT emitted at sub-step 2 (CHARGE X landing at objective cell) — the event is gated to sub-step 6 end only (GDD Rule 7, Edge Cases). Objective damage fires only at sub-step 6 regardless of which sub-step brought the unit there.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- Objective System [M2]: consuming `UnitAtObjective`, applying damage, checking destruction
- Story 009: Prism collection — Player A's prism is at cell 1, not cell 8; Player B's is at cell 8, not cell 1; prism and objective are different

---

## QA Test Cases

*Written by qa-lead at story creation. The developer implements against these.*

- **BL-10**: `UnitAtObjective` fires exactly once for Player A at cell 8
  - Given: `World::new()` with entity `unit` having `UnitCell(8u8)`, `PlayerOwner(PlayerA)`, `LaneId(Lane(1))`; `BoardConfig::default()` (player_a_objective_cell=8); `Messages<UnitAtObjective>` resource registered
  - When: `detect_objective_presence` system runs at sub-step 6 end
  - Then: Exactly 1 `UnitAtObjective` message written with `unit_id = unit` and `lane = Lane(1)`
  - Edge cases: Two PlayerA units at cell 8 in different lanes → 2 events (one per lane); PlayerA unit at cell 7 → no event; PlayerB unit at cell 8 → no event for PlayerB (their objective is cell 1)

- **BL-11**: Unit at objective persists and fires again next round
  - Given: Entity `unit` for PlayerA at `(Lane(1), Cell(8))`; round N passes (unit survives, is not displaced); entity still present and at cell 8 in round N+1
  - When: `detect_objective_presence` runs at round N+1 sub-step 6
  - Then: `UnitCell` still = `8u8`; `UnitAtObjective` fires again for `unit` in round N+1
  - Edge cases: Unit killed before round N+1 sub-step 6 → entity despawned → no event; unit displaced away from cell 8 before sub-step 6 → no event

- **BL-25**: `UnitAtObjective` fires for Player B at cell 1
  - Given: Entity `unit` for PlayerB with `UnitCell(1u8)`, `LaneId(Lane(3))`; `BoardConfig::default()` (player_b_objective_cell=1)
  - When: `detect_objective_presence` runs
  - Then: `UnitAtObjective` written with `unit_id = unit`, `lane = Lane(3)`
  - Edge cases: PlayerA unit at cell 1 → no event (PlayerA's objective is cell 8); PlayerB unit at cell 2 → no event

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/board-lane-system/objective_detection_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 002 must be DONE (unit cells set by movement; F3 formula reads cell position)
- Unlocks: Nothing in this epic — consumed by Objective System [M2]
