# Story 004: Placement Occupancy Enforcement

> **Epic**: Board / Lane System
> **Status**: Ready
> **Layer**: Feature
> **Type**: Logic
> **Manifest Version**: 2026-04-29

## Context

**GDD**: `design/gdd/board-lane-system.md`
**Requirement**: `TR-BLS-004`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-007: Placement Buffer and Simultaneous Reveal Architecture
**ADR Decision Summary**: Occupancy validation is part of the all-or-nothing placement submission check that runs before writing to `PendingPlacements`; if any card in the batch fails, the entire submission is silently discarded with no S2C response.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**: Occupancy checks query `BoardOccupancy` resource — a plain Rust `HashMap`-backed `Resource`, no ECS query needed. Pure functions are testable with `World::new()` + resource insertion only.

**Control Manifest Rules (this layer)**:
- Required: Placement validation is all-or-nothing per player — if occupancy check fails for any card, discard entire batch silently
- Required: Invalid placement submissions produce no S2C response to the client
- Required: Mana deduction happens at PLACEMENT close, not at submission receipt — do NOT deduct gold here
- Forbidden: Never spawn ECS entity for a pending placement before `S2CPlacementReveal` is enqueued
- Guardrail: O(N) validation per submission batch; HashMap occupancy lookup is O(1)

---

## Acceptance Criteria

*From GDD `design/gdd/board-lane-system.md`, scoped to this story:*

- [ ] **BL-8**: GIVEN Player A already has a Minion in lane 3, WHEN they submit another Minion to lane 3, THEN the second placement is rejected and gold is not deducted.
- [ ] **BL-9**: GIVEN Player A has 10 gold AND has a Trap at (lane 2, cell 3), WHEN they attempt a second Trap at the same cell, THEN the placement is rejected AND Player A's gold remains 10.
- [ ] **BL-29**: GIVEN Player A already has a Field active in lane 2, WHEN Player A submits a second Field to lane 2, THEN the placement is rejected and gold is not deducted.
- [ ] **BL-32**: GIVEN Player A has a Field active in lane 2, WHEN Player B submits a Field to lane 2, THEN Player B's placement is accepted AND both Fields are present in lane 2 occupancy state simultaneously (each player may have one Field per lane independently).
- [ ] **BL-33**: GIVEN a 2v2 game and Team A's Player 1 already has a Minion in lane 1, WHEN Team A's Player 2 submits a Minion to lane 1, THEN the placement is accepted (team has used 1 of 2 allowed slots). WHEN Team A's Player 2 also submits their own second Minion to lane 1 (personal slot already occupied), THEN the second placement is rejected and gold is not deducted.

---

## Implementation Notes

*Derived from ADR-007 Validation Rules table and GDD Rule 3, Rule 5:*

Implement occupancy validation functions in `server/src/feature/board/placement.rs`:

```rust
/// Occupancy checks — called per card in the batch before writing to PendingPlacements.
/// Returns false if the slot is already occupied (triggers all-or-nothing batch discard).

pub fn is_minion_slot_available(
    occupancy: &BoardOccupancy,
    player: PlayerId,
    lane: LaneId,
    game_mode: &GameMode,
) -> bool {
    match game_mode {
        GameMode::OneVOne => {
            // 1v1: exactly 1 Minion slot per player per lane
            occupancy.minion_slots[player.index()][lane.index()].is_none()
        }
        GameMode::TwoVTwo => {
            // 2v2: 2 Minion slots per team per lane (1 per player on team)
            // Personal slot must be empty AND team has capacity
            let personal_empty = occupancy.minion_slots[player.index()][lane.index()].is_none();
            let team_count = occupancy.team_minion_count_in_lane(player.team(), lane);
            personal_empty && team_count < 2
        }
    }
}

pub fn is_trap_slot_available(
    occupancy: &BoardOccupancy,
    player: PlayerId,
    lane: LaneId,
    cell: u8,
) -> bool {
    !occupancy.traps.contains_key(&(player, lane, cell))
}

pub fn is_structure_slot_available(
    occupancy: &BoardOccupancy,
    player: PlayerId,
    lane: LaneId,
    cell: u8,
) -> bool {
    !occupancy.structures.contains_key(&(player, lane, cell))
}

pub fn is_field_slot_available(
    occupancy: &BoardOccupancy,
    player: PlayerId,
    lane: LaneId,
) -> bool {
    !occupancy.fields.contains_key(&(player, lane))
}
```

Gold is NOT deducted in this function. Deduction happens in `close_placement_phase` (Story 005) at PLACEMENT close. The gold invariant in BL-9 is upheld because the batch is discarded before any deduction occurs.

Fields per lane are per-player, not per-team. Each player may have one Field per lane independently (BL-32). Two opposing players may have Fields in the same lane simultaneously.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- Story 003: Spawn range (F2) validation — separate check, runs before occupancy
- Story 005: Full placement submission pipeline that calls these functions
- Story 007: Trap trigger mechanics — this story only validates placement, not combat interaction

---

## QA Test Cases

*Written by qa-lead at story creation. The developer implements against these.*

- **BL-8**: Duplicate Minion in same lane rejected
  - Given: `BoardOccupancy` with `minion_slots[PlayerA.index()][Lane(3).index()] = Some(entity1)`; `GameMode::OneVOne`
  - When: `is_minion_slot_available(&occupancy, PlayerA, Lane(3), &GameMode::OneVOne)`
  - Then: returns `false`
  - Edge cases: PlayerA Minion in Lane(3), PlayerB submits to Lane(3) → PlayerB's check returns true (independent); PlayerA submits to Lane(4) → true (different lane)

- **BL-9**: Duplicate Trap at same (lane, cell) rejected; gold untouched
  - Given: `BoardOccupancy` with `traps[(PlayerA, Lane(2), 3)] = Some(entity1)`
  - When: `is_trap_slot_available(&occupancy, PlayerA, Lane(2), 3u8)`
  - Then: returns `false`; gold value in `PlayerEconomy` resource not modified (no deduction call made)
  - Edge cases: PlayerA Trap at (Lane(2), Cell(3)), submit Trap at (Lane(2), Cell(4)) → true; PlayerB Trap at same cell → true (independent)

- **BL-29**: Duplicate Field in same lane rejected
  - Given: `BoardOccupancy` with `fields[(PlayerA, Lane(2))] = Some(entity1)`
  - When: `is_field_slot_available(&occupancy, PlayerA, Lane(2))`
  - Then: returns `false`
  - Edge cases: PlayerA Field in Lane(2), submit Field to Lane(3) → true; empty board → true

- **BL-32**: Two players may independently have one Field per lane
  - Given: `BoardOccupancy` with PlayerA's Field in Lane(2); checking for PlayerB
  - When: `is_field_slot_available(&occupancy, PlayerB, Lane(2))`
  - Then: returns `true` (PlayerA's Field does not block PlayerB)
  - After placement: query occupancy for `fields` in Lane(2) → 2 entries, one per player

- **BL-33**: 2v2 team Minion slot semantics
  - Given: `GameMode::TwoVTwo`; Team A = [Player1, Player2]; `minion_slots[Player1.index()][Lane(1).index()] = Some(e1)`; `team_minion_count(TeamA, Lane(1)) = 1`
  - When: `is_minion_slot_available(&occupancy, Player2, Lane(1), &GameMode::TwoVTwo)` (slot empty, team count < 2)
  - Then: returns `true` (accepted)
  - When (second): Player2 places Minion in Lane(1) (their slot now occupied); check again for Player2 Minion in Lane(1)
  - Then: returns `false` (personal slot occupied)

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/board-lane-system/placement_occupancy_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 must be DONE (`BoardOccupancy` resource initialized)
- Unlocks: Story 005 (placement buffer pipeline calls these occupancy functions)
