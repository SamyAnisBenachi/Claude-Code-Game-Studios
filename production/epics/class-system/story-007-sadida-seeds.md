# Story 007: Sadida Seeds and Graines de Folie

> **Epic**: Class System
> **Status**: Ready
> **Layer**: Feature (M3)
> **Type**: Integration
> **Manifest Version**: 2026-04-30

## Context

**GDD**: `design/gdd/class-system.md`
**Requirement**: `TR-CS-006`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-014: Class System Architecture — PlayerSessionState, SourceClass Component, and Direct Effect Dispatch
**ADR Decision Summary**: Class effects are plain Rust functions called within the RESOLUTION system body. Sadida Seed walk-over fires during sub-step 5 (standard movement); both intermediate and final destination cells trigger. Graines de Folie iterates board Seeds, removes each, and calls `spawn_madoll` from Story 002. Integration points: Combat Resolution damage pipeline (enemy walk-over 1 dmg pre-AR), Board/Lane System (Madoll spawn with over-capacity guard).

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: MEDIUM
**Engine Notes**:
- Seed entities may be separate ECS entities with `SeedMarker` + `BoardPosition` + `SourceClass(ClassId::Sadida)` + `TokenUnit` — spawned by `spawn_seed` from Story 002.
- Walk-over traversal logic must integrate with Combat Resolution sub-step 5 movement loop — confirm the exact hook point (cell-by-cell traversal callback or post-movement batch check) with `combat-resolution` GDD before implementing.
- PIERCE keyword bypasses AR entirely — verify PIERCE is defined in `keyword-system.md` (pre-implementation gate from EPIC.md) before this story's Seed AR-stacking behavior is accepted as final.
- ADR-014 is NOT yet in the control manifest.

**Control Manifest Rules (Feature Layer)**:
- Required: Feature systems react to RSM Messages; never observe RoundState directly — ADR-010
- Required: Board movement formula uses i16 intermediate to prevent u8 underflow — ADR (BLS-002)
- Forbidden: Never let Feature systems call Core/Foundation directly — ADR-010
- Guardrail: Server tick budget ≤ 5ms steady state; RESOLUTION batch ≤ 15ms — ADR-002

---

## Acceptance Criteria

*From GDD `design/gdd/class-system.md`, CS-7 and CS-8 formulas:*

- [ ] **CS-AC-18** GIVEN a Seed on cell 3 lane 2, WHEN a friendly unit's movement path passes through cell 3 lane 2 during sub-step 5 (whether as an intermediate cell or the final destination), THEN the unit gains +1 AR permanently; the Seed remains on the cell.
- [ ] **CS-AC-19** GIVEN a Seed on cell 3 lane 2, WHEN an enemy unit's movement path passes through cell 3 lane 2 during sub-step 5 (whether intermediate or final cell), THEN that unit takes 1 damage pre-AR (effective damage = max(0, 1 − unit.AR), routed through the AR reduction pipeline); the Seed persists.
- [ ] **CS-AC-20** GIVEN Sadida player has 3 Seeds on the board across different lanes, WHEN Graines de Folie is cast, THEN 3 Madolls (HP=3, ATK=1, MP=3) are spawned at the exact Seed cells and all 3 Seeds are removed from the board.
- [ ] **CS-AC-21** GIVEN Sadida player has 2 Seeds — one on cell 2 lane 3 (empty) and one on cell 1 lane 1 (lane at unit capacity), WHEN Graines de Folie resolves, THEN 1 Madoll spawns at cell 2 lane 3; no Madoll spawns in lane 1; both Seeds are consumed.

---

## Implementation Notes

*Derived from ADR-014 Decision §4 and GDD Formulas CS-7, CS-8:*

**File location**: `server/src/core/resolution/effects.rs`

**CS-7 — Seed walk-over** (fires during sub-step 5 per-cell traversal):
```rust
/// Called for each cell a unit traverses during sub-step 5 movement.
/// Caller (sub-step 5 loop) must call this for EVERY cell in the movement path
/// (intermediate + destination), not just the final cell.
pub fn apply_seed_walkover(
    board: &mut BoardState,
    unit_id: EntityId,
    cell: u8,
    lane: u8,
) {
    let Some(seed_entity) = board.seed_at(lane, cell) else { return; };
    let unit = board.unit_mut(unit_id).expect("seed walkover: unit not found");
    if unit.owner == board.seed_owner(seed_entity) {
        // Friendly walk-over: +1 AR permanently
        unit.ar += 1;
        // seed persists — do NOT remove
    } else {
        // Enemy walk-over: 1 damage pre-AR (routed through damage pipeline)
        let effective_dmg = 1u32.saturating_sub(unit.ar);
        if effective_dmg > 0 {
            unit.hp = unit.hp.saturating_sub(effective_dmg);
            // If HP → 0, caller RESOLUTION system handles DEATH trigger (not this function)
        }
        // seed persists — do NOT remove
    }
}
```

**Stacking rule**: Max 1 seed per cell enforced at placement time (by the Graines de Folie / Pollinisation placement path). Walk-over from multiple seeds in one path is legal — each seeded cell in the path triggers once.

**CS-8 — Graines de Folie conversion**:
```rust
pub fn apply_graines_de_folie(
    board: &mut BoardState,
    commands: &mut Commands,
    owner: PlayerId,
) {
    let seeds: Vec<(u8, u8)> = board.seeds_for_player(owner)
        .map(|s| (s.lane, s.cell))
        .collect();  // collect before mutating
    for (lane, cell) in seeds {
        board.remove_seed(lane, cell, owner);  // seed consumed unconditionally
        if board.can_spawn_in_lane(lane) {
            spawn_madoll(commands, owner, lane, cell);  // from Story 002
        }
        // over-capacity: seed still removed, Madoll skipped — no error or warn
    }
}
```

**Pre-implementation gate**: PIERCE keyword must be defined in `keyword-system.md` before this story closes, as PIERCE is the design counter to high-AR Sadida units (CS-7 note). If PIERCE is undefined, AR stacking via Seeds is unbalanced — block story sign-off until confirmed.

---

## Out of Scope

*Handled by neighbouring stories:*

- Story 002: `spawn_madoll()` function — provided by token spawn scaffold; this story calls it
- Story 005: Miss Nuit trigger — separate formula
- Combat Resolution: DEATH trigger processing when enemy unit HP → 0 from Seed damage — Combat Resolution epic
- Keyword System: PIERCE bypass logic — Keyword System epic
- Board/Lane System: `can_spawn_in_lane()` implementation — Board/Lane epic
- GDD CS-7 PIERCE counter note: verify PIERCE defined before closing this story (pre-impl gate, not impl work here)

---

## QA Test Cases

*Integration story — automated test specs using `World::new()` with board state.*

- **AC CS-AC-18 — Friendly walk-over grants AR, seed persists**:
  - Given: Seed on cell 3 lane 2 (owner = Sadida player); friendly unit (same owner) with AR=0 moving from cell 1 to cell 5 through cell 3
  - When: `apply_seed_walkover(board, friendly_unit_id, cell=3, lane=2)` called during sub-step 5 traversal
  - Then: unit AR = 1; seed entity still present on cell 3 lane 2
  - Edge cases: unit traverses 2 seeded cells → AR = +2 total; each seeded cell triggers independently

- **AC CS-AC-19 — Enemy walk-over deals 1 dmg pre-AR, seed persists**:
  - Given: Seed on cell 3 lane 2 (Sadida owner); enemy unit (opponent) with HP=5, AR=0 moving through cell 3
  - When: `apply_seed_walkover(board, enemy_unit_id, cell=3, lane=2)` called
  - Then: enemy HP = 4 (1 dmg pre-AR, AR=0 so effective=1); seed still present
  - Edge cases: enemy unit AR=2 → effective damage = max(0, 1−2) = 0; unit HP unchanged; seed still triggers but no damage

- **AC CS-AC-20 — Graines de Folie spawns Madolls, removes all seeds**:
  - Given: Sadida player has 3 Seeds: (lane=1, cell=2), (lane=3, cell=1), (lane=5, cell=4); all lanes have capacity
  - When: `apply_graines_de_folie(board, commands, sadida_player_id)` called
  - Then: all 3 Seeds removed from board; 3 Madoll entities spawned at (1,2), (3,1), (5,4); each Madoll has HP=3, ATK=1, MP=3

- **AC CS-AC-21 — Over-capacity: seed consumed, Madoll skipped**:
  - Given: 2 Seeds: (lane=3, cell=2) with capacity; (lane=1, cell=1) with lane at unit cap
  - When: `apply_graines_de_folie(...)` called
  - Then: Seed at (lane=3, cell=2) removed → Madoll spawned; Seed at (lane=1, cell=1) removed → NO Madoll spawned (over-capacity skip); net: 2 seeds consumed, 1 Madoll spawned

---

## Test Evidence

**Story Type**: Integration
**Required evidence**: `tests/integration/class/sadida_seeds_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 002 (spawn_madoll, spawn_seed — token spawn scaffold) — must be DONE
- Depends on: `keyword-system` epic (PIERCE definition — pre-impl gate for story sign-off, not for code compilation)
- Depends on: `combat-resolution` epic (sub-step 5 traversal hook; damage pipeline for enemy walk-over; DEATH trigger from HP→0)
- Depends on: `board-lane-system` epic (BoardState.can_spawn_in_lane(), remove_seed()) — must be DONE
- Unlocks: Story 010 (Token passives — Madoll passive cost-reduction requires Madoll units to be spawnable via this story's Graines de Folie)
