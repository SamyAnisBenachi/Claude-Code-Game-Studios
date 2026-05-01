# Story 016: Displacement Keywords (CHARGE X, REPEL, ATTRACT, TELEPORT, CHANGE LANE)

> **Epic**: Keyword System
> **Status**: Blocked
> **Layer**: Feature (M3)
> **Type**: Logic
> **Manifest Version**: 2026-04-30

## Context

**GDD**: `design/gdd/keyword-system.md`
**Requirement**: `TR-KW-002` — CHARGE X bonus movement applied at sub-step 2; cells parameter clamped per Board/Lane F1. `TR-KW-???` — REPEL, ATTRACT, TELEPORT, and CHANGE LANE effect dispatchers have no registered TR-ID. Run `/architecture-review` to register missing TRs before marking this story Done.
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-018 (Keyword System — ECS State Architecture, `effects.rs` and `movement.rs` sections)
**ADR Decision Summary**: Displacement effect functions live in `server/feature/keyword/effects.rs` and are called by `server/feature/combat/` as plain function calls. Pure formula functions (`repel_destination`, `attract_destination`) live in `movement.rs` and are tested in Story 002. This story implements the effect callers: `apply_repel`, `apply_attract`, `apply_teleport`, `apply_change_lane`, and CHARGE X's SS2 execution. `check_irremovable()` (Story 007) is called before any displacement.

**BLOCKED**: ADR-018 is Proposed — advance to Accepted before opening. Stories 001 (scaffold — stubs all effect functions), 002 (movement formula pure functions), and 007 (IRREMOVABLE — `check_irremovable` must be callable) must be Done.

> ⚠️ **KW-033b is permanently BLOCKED** until `strich_change_lane_select` seed slot is registered in ADR-005 RNG consumption order table. Do not attempt to implement KW-033b.

> ⚠️ **KW-051 and KW-052 are BLOCKED** until OQ-KS4 (Trap design) is resolved. Trap card design spec must be authored and Trap behavior defined in `card-data-pool.md` before these ACs can be implemented.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**:
- `apply_repel/apply_attract/apply_teleport` take `world: &mut World` (exclusive system access per ADR-017)
- `apply_change_lane` queries `BoardState` for lane-slot availability before moving
- Strich CHANGE LANE: triggered from `on_unit_appeared` observer when enemy enters Strich's lane in SS1
- CHARGE X in SS2: called inline from `resolve_combat`'s SS2 pass for each unit with `ChargeXMove { cells }` keyword (not STUNned, not summoning-sickness-blocked)

**Control Manifest Rules (Feature layer)**:
- Required: Call `check_irremovable(target, world)` before any `apply_repel`, `apply_attract`, or `apply_teleport` — if IRREMOVABLE, emit `DisplacementEvent { was_blocked: true }` and return (Story 007)
- Required: CHARGE X applies F1 formula with WALL-blocking and collision rules identical to SS5
- Required: ATTRACT enemy target: apply 1-cell-apart collision rule (`effective_pull = min(X, max(0, |caster_cell − target_cell| − 1))`) — enemy stops 1 cell short of caster (GDD Formula 2 enemy-target branch)
- Required: TELEPORT bypasses spawn-range restrictions and 1-cell-apart collision rule; co-occupation allowed
- Forbidden: Never trigger APPEARANCE or COUNTERATTACK from TELEPORT (GDD)
- Forbidden: Never allow CHANGE LANE to a lane that already has a friendly Minion — silent no-op (KW-032)
- Forbidden: Never use `strich_change_lane_select` seed slot until registered in ADR-005 (KW-033b BLOCKED)

---

## Acceptance Criteria

*From GDD `design/gdd/keyword-system.md` Acceptance Criteria:*

### CHARGE X (SS2)
- [ ] KW-028: GIVEN a unit with CHARGE 2 is in a lane where an enemy WALL is 1 cell ahead, WHEN SS2 resolves, THEN the unit is blocked at the WALL's cell and does not pass through it; the same WALL-blocking and collision rules as SS5 apply to CHARGE X movement

### REPEL (effect application — pure formula tested in Story 002)
- [ ] KW-029c / KW-067: GIVEN a Player A unit at Cell 1 is REPELled 6 cells (maximum X), WHEN `apply_repel` resolves, THEN destination = `clamp(1 + (−1)×6, 1, 8) = 1`; unit stays at Cell 1; zero intermediate cells traversed (no Trap triggers at Cell 1 itself); KW-067 is the R3 reinforcement of this rule — one test satisfies both
- [ ] KW-029d: GIVEN a Player B unit at Cell 8 is REPELled 6 cells, WHEN `apply_repel` resolves, THEN destination = `clamp(8 + (+1)×6, 1, 8) = 8`; unit stays at Cell 8; zero intermediate cells traversed
- [ ] **KW-051 — BLOCKED until OQ-KS4 (Trap design) resolved**: GIVEN a unit is REPELled through a cell containing a lethal Trap, displacement halts at Trap cell; unit dies at Trap cell. Do not implement until Trap behavior is defined in `card-data-pool.md`.

### ATTRACT (effect application — enemy target collision rule, not covered in Story 002)
- [ ] KW-030b / KW-068: GIVEN Player A caster at Cell 3 ATTRACTs an enemy (Player B) target at Cell 7 with ATTRACT 6, WHEN `apply_attract` resolves (enemy target branch), THEN `effective_pull = min(6, max(0, |3−7|−1)) = min(6, 3) = 3`; enemy target lands at Cell 4 (1 cell short of caster — collision rule enforced); KW-068 tests the same rule with different numbers — one implementation satisfies both
- [ ] **KW-052 — BLOCKED until OQ-KS4 (Trap design) resolved**: GIVEN a unit is ATTRACTed through a Trap cell, displacement halts at Trap cell. Do not implement until Trap behavior is defined.

### TELEPORT
- [ ] KW-031a: GIVEN a unit is TELEPORTed to a cell occupied by an enemy unit, WHEN TELEPORT resolves, THEN no APPEARANCE trigger fires for the teleported unit — TELEPORT is not a board entry from PlacementBuffer
- [ ] KW-031b: GIVEN a unit is TELEPORTed to a cell occupied by an enemy unit, WHEN TELEPORT resolves, THEN no COUNTERATTACK fires from the unit at the destination — TELEPORT does not constitute a melee attack

### CHANGE LANE
- [ ] KW-032: GIVEN a unit attempts CHANGE LANE to an adjacent lane that already has a friendly Minion, WHEN CHANGE LANE resolves, THEN the lane change does NOT execute; unit remains in current lane; no error state; no network event emitted (silent no-op)
- [ ] KW-033a: GIVEN Strich is in Lane 3; exactly one adjacent lane (Lane 2 or Lane 4) is valid (the other is full with a friendly Minion), WHEN an enemy unit enters Lane 3 in SS1, THEN Strich automatically executes CHANGE LANE to the only valid adjacent lane
- [ ] **KW-033b — PERMANENTLY BLOCKED**: requires `strich_change_lane_select` seed slot registered in ADR-005. Do not implement.
- [ ] KW-033c: GIVEN Strich is in Lane 3; both adjacent lanes (Lane 2 and Lane 4) already contain a friendly Minion, WHEN an enemy unit enters Lane 3 in SS1, THEN CHANGE LANE is rejected; Strich remains in Lane 3; no error state

---

## Implementation Notes

*Derived from ADR-018 effects.rs interface and GDD Movement Keyword Catalog:*

**CHARGE X execution (SS2 inline):**
```rust
// In execute_ss2(world): for each unit with ChargeXMove { cells } not STUNned/sickness-blocked:
let dest = board_f1_formula(current_cell, advance_dir, charge_cells);
apply_movement_with_wall_check(unit, dest, world);  // same WALL logic as SS5
```

**apply_attract — enemy vs friendly branch (KW-030b):**
```rust
let dest = if target_owner == caster_owner {
    // Friendly: full pull, co-occupation allowed
    movement::attract_destination(caster_cell, target_cell, distance)
} else {
    // Enemy: 1-cell-apart collision rule
    let adj_dist = distance.min(target_cell.abs_diff(caster_cell).saturating_sub(1));
    movement::attract_destination(caster_cell, target_cell, adj_dist)
};
```

**apply_teleport — no APPEARANCE, no COUNTERATTACK (KW-031a/b):**
No `world.trigger_targets(UnitAppeared, ..)` call. No `check_and_apply_counterattack` at destination. Set position directly. Co-occupation allowed.

**apply_change_lane — slot validation (KW-032):**
```rust
if board::api::lane_has_friendly_minion(dest_lane, owner, world) { return false; }
board::api::move_unit_to_lane(unit, dest_lane, world);
```

**Strich CHANGE LANE (KW-033a/c) — called from on_unit_appeared:**
Count valid adjacent lanes (lane ± 1 without friendly Minion). If 1 valid: call `apply_change_lane`. If 0: no-op. If 2: BLOCKED (KW-033b).

---

## Out of Scope

- Story 002: `repel_destination()` and `attract_destination()` pure function tests (KW-029a/b, KW-030a)
- Story 007: `check_irremovable()` implementation
- KW-033b: PERMANENTLY BLOCKED (strich_change_lane_select seed slot pending ADR-005)
- KW-051/KW-052: BLOCKED until OQ-KS4 Trap design resolved

---

## QA Test Cases

*Automated test specs (Logic story):*

- **KW-028**: CHARGE X blocked by WALL
  - Given: Player A unit (CHARGE 2) at Cell 2; Player B WALL at Cell 3; advance_dir = +1
  - When: SS2 CHARGE X movement runs (attempting Cell 4)
  - Then: unit stops at Cell 3 (WALL cell); WALL not displaced; CHARGE X did not overshoot

- **KW-029c/d**: REPEL clamping at board edges
  - Given: Player A unit at Cell 1; REPEL 6 by Player B
  - When: `apply_repel` runs
  - Then: dest = Cell 1 (clamped); unit position unchanged; zero intermediate cells; `DisplacementEvent { from_cell: 1, to_cell: 1 }` emitted (or no event if no movement)
  - Edge cases: Player B at Cell 8, REPEL 6 → same result (Cell 8)

- **KW-030b**: ATTRACT enemy stops 1 cell short
  - Given: Player A caster at Cell 3; Player B enemy at Cell 7; ATTRACT 6
  - When: `apply_attract` runs (enemy branch)
  - Then: `adjusted_dist = min(6, max(0, 4-1)) = 3`; enemy lands at Cell 4 (not Cell 3); `DisplacementEvent { from_cell: 7, to_cell: 4 }` emitted

- **KW-031a/b**: TELEPORT — no APPEARANCE, no COUNTERATTACK
  - Given: Player A unit teleported to Cell 5; Player B unit (COUNTERATTACK) already at Cell 5
  - When: `apply_teleport` runs
  - Then: no `UnitAppeared` event fired for teleporting unit; no COUNTERATTACK fired from Player B unit; both units now at Cell 5

- **KW-032**: CHANGE LANE rejected if lane full
  - Given: Player A unit in Lane 2 attempting CHANGE LANE to Lane 3; friendly Minion already in Lane 3
  - When: `apply_change_lane(unit, Lane 3)` runs
  - Then: returns false; unit still in Lane 2; board state unchanged

- **KW-033a**: Strich CHANGE LANE to only valid adjacent lane
  - Given: Strich in Lane 3; Lane 2 full; Lane 4 empty; enemy unit enters Lane 3 in SS1
  - When: `on_unit_appeared` fires; Strich CHANGE LANE check runs
  - Then: `apply_change_lane(strich, Lane 4)` succeeds; Strich now in Lane 4

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/keyword/displacement_keywords_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 (scaffold), Story 002 (movement formula pure functions), Story 007 (IRREMOVABLE) must be Done
- Unlocks: Story 017 (FIRST STRIKE × WALL uses displacement-adjacent movement path)
