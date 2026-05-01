# Story 003: Spawn Range Validation (F2)

> **Epic**: Board / Lane System
> **Status**: Ready
> **Layer**: Feature
> **Type**: Logic
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/board-lane-system.md`
**Requirement**: `TR-BLS-003`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-007: Placement Buffer and Simultaneous Reveal Architecture
**ADR Decision Summary**: All placement validation — including spawn range (F2) — runs server-side before writing to `PendingPlacements`; invalid submissions are silently discarded in their entirety (all-or-nothing per player); Structures and Traps bypass spawn range entirely.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**: `validate_spawn_range` is a pure function (no ECS system params needed) — test with plain Rust `#[test]`, no `World::new()` required. Reads `SpawnRangeState` resource at call time from the system that invokes it.

**Control Manifest Rules (this layer)**:
- Required: Spawn range validation (Formula F2): Minions only; Structures and Traps bypass range entirely
- Required: Placement validation is all-or-nothing per player — if any card fails, discard entire batch silently
- Required: Invalid placement submissions produce no S2C response to the client
- Forbidden: Never spawn ECS entity for a pending placement — validation runs before any ECS mutation
- Guardrail: `PendingPlacements` validation is O(N) where N = cards in submission; must complete within single frame

---

## Acceptance Criteria

*From GDD `design/gdd/board-lane-system.md`, scoped to this story:*

- [ ] **BL-5**: GIVEN Player A has 0 fakes destroyed, WHEN they attempt Minion placement at cell 2, THEN the placement is rejected.
- [ ] **BL-5b**: GIVEN Player B has 0 fakes destroyed, WHEN they attempt Minion placement at cell 7, THEN the placement is rejected.
- [ ] **BL-6**: GIVEN Player A has 1 fake destroyed, WHEN they place a Minion at cell 2, THEN the placement is accepted.
- [ ] **BL-6b**: GIVEN Player B has 1 fake destroyed, WHEN they place a Minion at cell 7, THEN the placement is accepted.
- [ ] **BL-7**: GIVEN Player A has 0 fakes destroyed, WHEN they place a Structure at cell 3, THEN the placement is accepted (Structures bypass spawn range).

---

## Implementation Notes

*Derived from ADR-007 Key Interfaces and GDD Formula F2:*

Implement `validate_spawn_range` in `server/src/feature/board/placement.rs` (per ADR-007 file layout):

```rust
/// Implements GDD Formula F2 — spawn range validation for Minion placements only.
/// Structures and Traps bypass this check (caller must gate by card type).
/// `fakes_destroyed` is read from SpawnRangeState resource at call site.
/// spawn_cell_A (1) and spawn_cell_B (8) are structural constants — NOT GameConfig fields.
pub fn validate_spawn_range(
    target_cell: u8,
    player: PlayerId,
    fakes_destroyed: u8,
) -> bool {
    match player {
        PlayerId::A => {
            let spawn_cell_a: u8 = 1;
            target_cell >= spawn_cell_a && target_cell <= spawn_cell_a + fakes_destroyed
        }
        PlayerId::B => {
            let spawn_cell_b: u8 = 8;
            target_cell >= spawn_cell_b.saturating_sub(fakes_destroyed) && target_cell <= spawn_cell_b
        }
    }
}
```

Call site pattern in `handle_placement_submission` (Story 005):

```rust
// Only apply spawn range check to Minions
if card.kind == CardKind::Minion {
    let fakes = spawn_range_state.fakes_destroyed[player.index()];
    if !validate_spawn_range(target_cell, player, fakes) {
        return; // silent discard — entire batch dropped
    }
}
// Structures and Traps fall through without the range check
```

Valid cells by fakes count (from GDD Formula F2 table):

| fakes | Player A valid cells | Player B valid cells |
|---|---|---|
| 0 | {1} | {8} |
| 1 | {1, 2} | {7, 8} |
| 2 | {1, 2, 3} | {6, 7, 8} |

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- Story 004: Minion slot, Trap, Structure, and Field occupancy checks (separate validation)
- Story 005: Full placement submission pipeline that calls this function
- Story 010: Spawn range expansion on fake objective destruction (updates `SpawnRangeState`)

---

## QA Test Cases

*Written by qa-lead at story creation. The developer implements against these.*

- **BL-5**: Player A default range — cell 2 rejected at 0 fakes
  - Given: `fakes_destroyed = 0u8`; `target_cell = 2u8`; `player = PlayerA`
  - When: `validate_spawn_range(2, PlayerA, 0)` called
  - Then: returns `false`
  - Edge cases: cell=3 → false; cell=1 → true (default range boundary); cell=4 → false

- **BL-5b**: Player B default range — cell 7 rejected at 0 fakes
  - Given: `fakes_destroyed = 0u8`; `target_cell = 7u8`; `player = PlayerB`
  - When: `validate_spawn_range(7, PlayerB, 0)`
  - Then: returns `false`
  - Edge cases: cell=6 → false; cell=8 → true (default range boundary); cell=5 → false

- **BL-6**: Player A range expanded by 1 fake
  - Given: `fakes_destroyed = 1u8`; `target_cell = 2u8`; `player = PlayerA`
  - When: `validate_spawn_range(2, PlayerA, 1)`
  - Then: returns `true`
  - Edge cases: cell=3 with fakes=1 → false (not yet expanded to 3); cell=3 with fakes=2 → true

- **BL-6b**: Player B range expanded by 1 fake
  - Given: `fakes_destroyed = 1u8`; `target_cell = 7u8`; `player = PlayerB`
  - When: `validate_spawn_range(7, PlayerB, 1)`
  - Then: returns `true`
  - Edge cases: cell=6 with fakes=1 → false; cell=6 with fakes=2 → true

- **BL-7**: Structure bypasses spawn range (caller responsibility)
  - Given: placement handler receives a Structure card; `fakes_destroyed = 0`; `target_cell = 3`
  - When: placement handler checks `card.kind == CardKind::Minion` before calling `validate_spawn_range`
  - Then: `validate_spawn_range` is NOT called; placement proceeds to occupancy check
  - Edge cases: Trap card at cell 3, fakes=0 → also bypasses; Minion at cell 3, fakes=0 → validate_spawn_range called → false

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/board-lane-system/spawn_range_validation_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 must be DONE (`SpawnRangeState` resource initialized)
- Unlocks: Story 005 (placement buffer pipeline calls `validate_spawn_range`)

---

## Readiness Refresh

- 2026-05-01: Revalidated against control manifest version 2026-05-01.
  ADR-007 remains accepted, `TR-BLS-003` remains active, Story 001 is complete,
  and no implementation requirements changed.
