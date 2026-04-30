# Story 007: Trap Trigger Mechanics

> **Epic**: Board / Lane System
> **Status**: Ready
> **Layer**: Feature
> **Type**: Integration
> **Manifest Version**: 2026-04-29

## Context

**GDD**: `design/gdd/board-lane-system.md`
**Requirement**: `TR-BLS-007`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-007: Placement Buffer and Simultaneous Reveal Architecture
**ADR Decision Summary**: Board/Lane owns Trap trigger logic — a Trap triggers when an enemy unit's final destination cell after any sub-step equals the Trap's cell; the Trap is removed after exactly one trigger; this is Integration because it requires the movement system and the board occupancy/trigger event system to interact correctly across sub-steps.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**: Use `#[derive(Event)]` + `commands.trigger()` pattern (Observer) for `TrapTrigger` event if it fires immediately within the same sub-step; OR use `MessageWriter<TrapTrigger>` if buffered for the next frame resolution. Decision: use `MessageWriter<TrapTrigger>` (buffered) so Combat Resolution can process it in sequence. Verify `app.add_message::<TrapTrigger>()` in 0.18. `despawn()` replaces `despawn_recursive()` as of Bevy 0.16. `liv-bevy-018` mandatory.

**Control Manifest Rules (this layer)**:
- Required: `despawn()` — not `despawn_recursive()` — to remove triggered Trap entities (Bevy 0.16+)
- Required: All board movement systems in `server/src/feature/board/` — Trap trigger check runs as part of movement commit, not as a separate pass
- Forbidden: Traps must never trigger on intermediate cells — only the final destination of each sub-step
- Guardrail: Trap check is O(traps_in_lane) per unit movement; HashMap lookup; negligible at any expected board density

---

## Acceptance Criteria

*From GDD `design/gdd/board-lane-system.md`, scoped to this story:*

- [ ] **BL-16**: GIVEN a unit enters a cell containing an enemy Trap via standard movement, WHEN it arrives, THEN the Trap triggers AND the Trap entity is removed from the board.
- [ ] **BL-17**: GIVEN a unit is REPEL'd into a cell containing an enemy Trap, WHEN it arrives, THEN the Trap triggers AND the Trap entity is removed from the board (displacement counts as enemy entry).
- [ ] **BL-31**: GIVEN Player A has a Trap at (lane 2, cell 3), Player B's unit X is in lane 1 at cell 3 and Player B's unit Y is in lane 3 at cell 3, and both CHANGE LANE to lane 2 in the same inter-sub-step pass (both arrive at cell 3), THEN the Trap triggers exactly once — triggered by unit X (lower original lane = 1 wins the tiebreak). The Trap is removed. Unit Y enters lane 2 at cell 3 and is not affected.
- [ ] **NEW-007a**: GIVEN a unit with CHARGE 2 is at (lane 1, cell 2) and an enemy Trap is at (lane 1, cell 4), WHEN sub-step 2 fires, THEN the unit moves to cell 4 (2+2) and the Trap triggers and is removed from the board (CHARGE X landing on a Trap triggers it).

---

## Implementation Notes

*Derived from ADR-007 and GDD Rules 10, Edge Cases:*

**Trap trigger rule (GDD Rule 10)**: A Trap triggers when an **enemy** unit occupies the Trap's cell, regardless of how the unit arrived (standard movement, CHARGE X, or displacement). "Enemy" = Trap.owner ≠ arriving_unit.owner.

**Trigger check function** (runs after each sub-step movement commit):

```rust
/// Check if the unit's final destination cell contains an enemy Trap.
/// Returns the Trap entity to despawn, if any.
pub fn check_trap_trigger(
    occupancy: &BoardOccupancy,
    unit_owner: PlayerId,
    lane: LaneId,
    destination_cell: u8,
) -> Option<Entity> {
    // Only triggers on enemy Traps (Trap owner ≠ unit owner)
    let enemy = unit_owner.opponent();
    occupancy.traps.get(&(enemy, lane, destination_cell)).copied()
}
```

After movement, for each unit that moved to a new cell:
```rust
if let Some(trap_entity) = check_trap_trigger(&occupancy, unit_owner, lane, new_cell) {
    // Remove Trap from occupancy map
    occupancy.traps.remove(&(enemy_owner, lane, new_cell));
    // Despawn Trap ECS entity (Bevy 0.16+ syntax)
    commands.entity(trap_entity).despawn();
    // Emit TrapTrigger message for Combat Resolution
    trap_trigger_writer.write(TrapTrigger { trap_entity, unit_entity, lane, cell: new_cell });
}
```

**CHANGE LANE simultaneous tiebreak (BL-31)**: When two units CHANGE LANE in the same inter-sub-step pass, process them in ascending original lane order. The first unit to arrive triggers the Trap and removes it. Subsequent arrivals find no Trap (already removed) and are not affected.

**Integration harness**: Tests that require CHANGE LANE tiebreak behaviour use `App::new()` with a minimal `BoardPlugin` (registers movement + trap trigger systems) rather than bare `World::new()`. Standard movement and REPEL trigger tests can use `World::new()` with manual system calls.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- Story 006: BL-27, BL-27b — non-trigger on intermediate cells (already tested)
- Story 010: REPEL displacement implementation — this story only tests that a REPEL-displaced unit triggers the Trap; REPEL itself is implemented in Story 010

---

## QA Test Cases

*Written by qa-lead at story creation. Integration story.*

- **BL-16**: Standard movement into enemy Trap triggers and removes it
  - Given: `World::new()` with enemy Trap entity for PlayerB at `(Lane(1), Cell(4))`; unit for PlayerA at `(Lane(1), Cell(1))`, `MP(3i16)` (will land at cell 4)
  - When: `apply_standard_movement` runs; `check_trap_trigger` runs on PlayerA's new destination
  - Then: `TrapTrigger` message written; Trap entity despawned from World; `occupancy.traps` no longer contains `(PlayerB, Lane(1), 4)`
  - Edge cases: Own Trap in destination cell → no trigger (ally Trap); no Trap in destination → no trigger

- **BL-17**: REPEL displacement into enemy Trap triggers and removes it
  - Given: enemy Trap at `(Lane(1), Cell(5))`; PlayerA unit at `(Lane(1), Cell(2))`; REPEL 3 targeting PlayerA unit (direction = -advance_direction(PlayerA) = -1 → new_cell = 2 + (-1×3) clamped = 1... wait this moves toward spawn. Actually REPEL pushes toward OWN spawn. REPEL 3 on PlayerA at cell 2 → cell max(2-3,1) = 1. That's not cell 5. Let me reconsider. REPEL pushes target toward their own spawn. If PlayerA is at cell 6 and REPEL 3 → new_cell = 6 - 3 = cell 3. For the test: PlayerA unit at cell 8, REPEL 3 → cell 5. That lands on the Trap.
  - Given: enemy Trap for PlayerB at `(Lane(1), Cell(5))`; PlayerA unit at `(Lane(1), Cell(8))`; REPEL 3 fired (pushes PlayerA toward spawn: new_cell = 8 - 3 = 5)
  - When: displacement movement resolves; `check_trap_trigger` runs at destination
  - Then: Trap triggers; Trap entity despawned; `TrapTrigger` message emitted
  - Edge cases: ATTRACT into Trap cell → also triggers; own Trap → no trigger

- **BL-31**: CHANGE LANE simultaneous arrival — lower lane tiebreak
  - Given: `App::new()` with `BoardPlugin`; Trap for PlayerA at `(Lane(2), Cell(3))`; PlayerB unit X at `(Lane(1), Cell(3))`; PlayerB unit Y at `(Lane(3), Cell(3))`; both CHANGE LANE to Lane(2) in same pass
  - When: CHANGE LANE pass processes in ascending original lane order (Lane(1) first, then Lane(3))
  - Then: Trap triggers for unit X (Lane(1) lower → processed first); Trap entity despawned; unit Y enters Lane(2) at Cell(3) and finds no Trap; no second `TrapTrigger` event

- **NEW-007a**: CHARGE X landing on Trap triggers it
  - Given: enemy Trap at `(Lane(1), Cell(4))`; PlayerA unit at `(Lane(1), Cell(2))`, `ChargeBonus(2i16)`
  - When: `apply_charge_movement` runs; `check_trap_trigger` runs at new cell 4
  - Then: unit at cell 4; Trap triggers; Trap entity despawned

---

## Test Evidence

**Story Type**: Integration
**Required evidence**: `tests/integration/board-lane-system/trap_trigger_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 002 must be DONE (standard movement), Story 006 must be DONE (CHARGE X movement)
- Unlocks: Nothing in this epic (terminal for Trap interaction)
