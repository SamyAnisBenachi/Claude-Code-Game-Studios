# Story 009: Prism Collection

> **Epic**: Board / Lane System
> **Status**: Complete
> **Layer**: Feature
> **Type**: Logic
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/board-lane-system.md`
**Requirement**: `TR-BLS-009`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-010: RSM Phase Event Bus — Phase Message Catalog and Subscriber Contracts
**ADR Decision Summary**: Board/Lane emits `PrismCollected(player, lane)` during sub-step 5 standard movement when a player's unit ends at their own prism cell; prism collection is ONLY gated to the sub-step 5 endpoint — TELEPORT, CHARGE X sub-step 2 landing, and displacement arrivals do not collect the prism.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**: `MessageWriter<PrismCollected>` (buffered). `PrismCollected` derives `Message`, `Clone`, `Debug`. Register via `app.add_message::<PrismCollected>()`. `PrismState` is a plain `Resource` (not ECS entity). `liv-bevy-018` mandatory.

**Control Manifest Rules (this layer)**:
- Required: Feature systems communicate upward via events — `PrismCollected` is emitted by Board/Lane, consumed by Prism System [M3]; Board/Lane does not apply prism rewards
- Required: Use `MessageWriter::write()` — `EventWriter` does not exist in Bevy 0.17+
- Forbidden: Never collect prism from TELEPORT, CHARGE X, REPEL, ATTRACT, or CHANGE LANE arrivals — only sub-step 5 standard movement final cell
- Guardrail: Prism check is O(units) appended to sub-step 5 movement — no additional pass needed

---

## Acceptance Criteria

*From GDD `design/gdd/board-lane-system.md`, scoped to this story:*

- [x] **BL-12**: GIVEN Player A's WALL unit ends sub-step 5 at cell 1 (Player A's prism cell), WHEN the prism check runs, THEN `PrismCollected(Player A, lane)` fires and the prism token is removed.
- [x] **BL-13**: GIVEN Player B's unit reaches cell 1, WHEN the prism check runs, THEN no `PrismCollected` fires for Player B — Player B's prism is at cell 8, not cell 1.
- [x] **BL-18**: GIVEN a unit is TELEPORT'd to its own spawn cell (cell 1 for Player A), THEN no prism is collected — TELEPORT is not sub-step 5 standard movement.
- [x] **BL-30**: GIVEN Player A's unit has `ChargeBonus(2)` and `MovementPoints(2)` and is at cell 1 (the prism cell) at the start of sub-step 2 in lane 3, WHEN sub-steps 2 and 5 both fire, THEN no `PrismCollected` event is emitted — the unit ends at cell 5 (1+2+2), not at the prism cell. Prism collection requires ending sub-step 5 at the prism cell.

---

## Implementation Notes

*Derived from ADR-010 and GDD Rules 11, 12, Edge Cases:*

**Prism check runs inside `apply_standard_movement`** (or immediately after, in the same system), keyed to the sub-step 5 pass only. It does NOT run after sub-step 2 (CHARGE X) or after displacement keyword processing.

```rust
/// Check if a unit's sub-step 5 final cell equals the owner's prism cell.
/// Player A prism = cell 1 (same as spawn cell); Player B prism = cell 8.
/// Only call this function from the sub-step 5 movement path.
pub fn check_prism_collection(
    owner: PlayerId,
    final_cell: u8,
    config: &BoardConfig,
    prism_state: &mut PrismState,
    lane: LaneId,
    writer: &mut MessageWriter<PrismCollected>,
) {
    let prism_cell = match owner {
        PlayerId::A => config.player_a_spawn_cell,  // 1
        PlayerId::B => config.player_b_spawn_cell,  // 8
    };
    if final_cell == prism_cell && prism_state.present[owner.index()][lane.index()] {
        prism_state.present[owner.index()][lane.index()] = false;
        writer.write(PrismCollected { player: owner, lane });
    }
}
```

**Sub-step gating**: The `apply_charge_movement` system (sub-step 2) does NOT call `check_prism_collection`. Displacement systems (TELEPORT, REPEL, ATTRACT) do NOT call `check_prism_collection`. Only `apply_standard_movement` (sub-step 5) calls it.

**Player B's prism at cell 8**: Player B's unit at cell 1 does NOT collect their prism — Player B's prism is at their spawn cell (cell 8), not cell 1. Cell 1 is Player A's prism cell (BL-13).

**WALL prism farming (GDD Edge Case)**: A WALL unit (MP=0) at cell 1 stays at cell 1 after sub-step 5 (F1 clamp: 1 + 0 = 1). The prism check runs after movement; `final_cell == 1 == prism_cell` for PlayerA → prism collected.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- Prism System [M3]: consuming `PrismCollected`, applying rewards, respawn cycle
- Story 006: BL-30 CHARGE movement — already tested that CHARGE advances unit away from prism cell

---

## QA Test Cases

*Written by qa-lead at story creation. The developer implements against these.*

- **BL-12**: WALL unit ends sub-step 5 at prism cell → PrismCollected fires
  - Given: `World::new()` with entity `unit` having `UnitCell(1u8)`, `MovementPoints(0i16)`, `PlayerOwner(PlayerA)`, `LaneId(Lane(2))`; `PrismState` with `present[PlayerA.index()][Lane(2).index()] = true`; `BoardConfig::default()`
  - When: `apply_standard_movement` runs (F1: 1 + 0 = 1, stays at cell 1); `check_prism_collection` runs
  - Then: `PrismCollected { player: PlayerA, lane: Lane(2) }` written; `prism_state.present[PlayerA.index()][Lane(2).index()] = false`
  - Edge cases: Prism already collected (`present = false`) → no second event; standard-moving non-WALL unit that happens to land at cell 1 → also collects

- **BL-13**: Player B unit at cell 1 does not collect Player B's prism
  - Given: entity `unit` for PlayerB, `UnitCell(1u8)`, `LaneId(Lane(1))`; `PrismState` all present
  - When: `check_prism_collection` runs for PlayerB at final_cell=1
  - Then: No `PrismCollected` emitted (Player B's prism cell = 8, not 1); prism state unchanged
  - Edge cases: PlayerB unit at cell 8 after sub-step 5 → `PrismCollected` for PlayerB fires

- **BL-18**: TELEPORT to spawn cell does not collect prism
  - Given: entity `unit` for PlayerA; TELEPORT event fires moving unit to cell 1; sub-step is TELEPORT (not sub-step 5 standard movement)
  - When: prism check is NOT called after TELEPORT (gated to sub-step 5 path only)
  - Then: No `PrismCollected` emitted; prism token present
  - Edge cases: REPEL into cell 1 → no collection; ATTRACT into cell 1 → no collection

- **BL-30**: CHARGE moves unit away from prism cell; standard movement advances further; no collection
  - Given: entity `unit` for PlayerA at `(Lane(3), Cell(1))`, `ChargeBonus(2i16)`, `MovementPoints(2i16)`; `PrismState.present[PlayerA][Lane(3)] = true`
  - When: sub-step 2 CHARGE runs → unit at cell 3 (1+2); prism check NOT called
  - When: sub-step 5 standard runs → unit at cell 5 (3+2); prism check runs at cell 5
  - Then: `final_cell = 5 ≠ prism_cell = 1` → no `PrismCollected`; prism token still present
  - Edge cases: `ChargeBonus(0)` + `MP(0)` from cell 1 → stays at 1 after both sub-steps → `PrismCollected` fires after sub-step 5 (correct, cell 1 is endpoint of sub-step 5)

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/board-lane-system/prism_collection_test.rs` — must exist and pass

**Status**: [x] Exists and passed (`cargo test -p server --test prism_collection_test`, 6/6).

---

## Dependencies

- Depends on: Story 002 must be DONE (standard movement), Story 006 must be DONE (CHARGE movement — needed for BL-30 compound sub-step scenario)
- Unlocks: Nothing in this epic — consumed by Prism System [M3]

---

## Completion Notes

**Completed**: 2026-05-03
**Criteria**: 4/4 passing. BL-12, BL-13, BL-18, and BL-30 are covered by `tests/unit/board-lane-system/prism_collection_test.rs`.
**Deviations**: None.
**Test Evidence**: Logic: `tests/unit/board-lane-system/prism_collection_test.rs`; `cargo test -p server --test prism_collection_test` passed 6/6. Movement regressions also passed via `cargo test -p server --test standard_movement_test --test charge_movement_test --test trap_trigger_test`. `cargo check -p server` passed.
**Code Review**: Skipped - Lean mode.
**Sprint Status**: Unchanged per user instruction; no matching BOARD-009 row exists in `production/sprint-status.yaml`.
