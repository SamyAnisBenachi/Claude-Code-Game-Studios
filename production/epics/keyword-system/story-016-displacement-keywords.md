# Story 016: Displacement Keywords — REPEL + ATTRACT + TELEPORT + CHANGE LANE

> **Epic**: Keyword System
> **Status**: Blocked
> **Layer**: Feature (M3)
> **Type**: Logic
> **Manifest Version**: 2026-04-30

## Context

**GDD**: `design/gdd/keyword-system.md`
**Requirement**: TR-KW-002 (CHARGE X bonus movement at SS2); TR-KW-??? (REPEL X — untraced); TR-KW-??? (ATTRACT X — untraced); TR-KW-??? (TELEPORT — untraced); TR-KW-??? (CHANGE LANE — untraced)
*(Run `/architecture-review` to register REPEL, ATTRACT, TELEPORT, CHANGE LANE TRs)*

**ADR Governing Implementation**: ADR-018 (effects.rs, movement.rs)
**ADR Decision Summary**: `apply_repel()` and `apply_attract()` call the pure `repel_destination()` / `attract_destination()` formulas from `movement.rs` (Story 002), then handle IRREMOVABLE check, `DisplacementEvent` emission, and Trap traversal hooks (Trap traversal calls are stubs pending OQ-KS4). TELEPORT: no APPEARANCE, no COUNTERATTACK, co-occupation allowed. CHANGE LANE: rejected silently if destination lane has friendly Minion.

**BLOCKED**: ADR-018 Proposed. Story 001 (scaffold), Story 002 (movement formulas), and Story 007 (IRREMOVABLE check) must be Done.
KW-033b additionally BLOCKED pending ADR-005 `strich_change_lane_select` seed slot registration.
KW-051/KW-052 additionally BLOCKED pending OQ-KS4 Trap design completion.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**: Displacement effects modify board position data (cell + lane) — coordinate with board-lane-system epic's position update API. CHANGE LANE lane-slot validation requires reading board occupancy from `BoardState` resource.

**Control Manifest Rules (Feature layer)**:
- Required: All randomness (Strich lane selection) must use server-side seeded RNG via `ServerRng` resource (ADR-005)
- Required: REPEL 0 and ATTRACT 0 are forbidden by card authoring rule — server silently skips if `x == 0` (no DisplacementEvent emitted)
- Forbidden: Never trigger APPEARANCE on TELEPORTed unit — TELEPORT is NOT a board entry from PlacementBuffer (ADR-007)

---

## Acceptance Criteria

*From GDD `design/gdd/keyword-system.md` Acceptance Criteria:*

- [ ] KW-028: GIVEN a unit with CHARGE 2 is in a lane where an enemy WALL is 1 cell ahead, WHEN SS2 resolves, THEN the unit is blocked at the WALL's cell and does not pass through it (CHARGE X uses same WALL-blocking and collision rules as SS5)
- [ ] KW-029a: GIVEN Player A unit at Cell 2 is REPELled 3 cells, WHEN `repel_destination` resolves, THEN unit lands at Cell 1 (clamped); `DisplacementEvent { from_cell: 2, to_cell: 1, kind: Repel }` emitted
- [ ] KW-029b: GIVEN WALL unit at Cell 5 is REPELled 2 cells toward Cell 8, WHEN REPEL resolves, THEN WALL moves to Cell 7; `DisplacementEvent { from_cell: 5, to_cell: 7 }` emitted
- [ ] KW-030: GIVEN caster at Cell 3, target at Cell 7, ATTRACT 4, WHEN ATTRACT resolves, THEN target lands at Cell 3; `DisplacementEvent { from_cell: 7, to_cell: 3 }` emitted
- [ ] KW-031a: GIVEN a unit is TELEPORTed to a cell occupied by an enemy unit, WHEN TELEPORT resolves, THEN no APPEARANCE trigger fires on the teleported unit
- [ ] KW-031b: GIVEN a unit is TELEPORTed to a cell occupied by an enemy unit, WHEN TELEPORT resolves, THEN no COUNTERATTACK fires from the enemy unit at that cell
- [ ] KW-032: GIVEN a unit attempts CHANGE LANE to an adjacent lane that already has a friendly Minion, WHEN CHANGE LANE resolves, THEN the lane change does not execute; unit remains in current lane; no error state created
- [ ] KW-033a: GIVEN Strich in Lane 3; exactly one adjacent lane valid; enemy enters Lane 3 in SS1, WHEN SS1 resolves, THEN Strich automatically executes CHANGE LANE to the only valid adjacent lane
- [ ] KW-033c: GIVEN Strich in Lane 3; both adjacent lanes (Lane 2, Lane 4) have friendly Minions, WHEN enemy enters Lane 3 in SS1, THEN CHANGE LANE rejected silently; Strich stays in Lane 3
- [ ] KW-041: GIVEN Player A uses ATTRACT to pull a Player B unit to Player A's Cell 1, WHEN SS6 objective damage resolves, THEN Player A's objective HP decreases by the Player B unit's ATK (ATTRACT does not grant immunity from backfire positioning)
- [ ] `DisplacementEvent { was_blocked: true }` emitted for IRREMOVABLE units (no position change; client plays Void flash)
- [ ] REPEL 0 and ATTRACT 0: server silently skips (no effect, no DisplacementEvent emitted)

> **BLOCKED ACs** (annotated — do not implement until gates resolve):
> - KW-033b: Strich both-lanes-valid RNG — BLOCKED until `strich_change_lane_select` seed slot registered in ADR-005
> - KW-051: REPEL through Trap lethal — BLOCKED until OQ-KS4 Trap design completed
> - KW-052: ATTRACT through Trap lethal — BLOCKED until OQ-KS4 Trap design completed

---

## Implementation Notes

*Derived from ADR-018 effects.rs, movement.rs, and GDD Movement Keyword Catalog:*

**apply_repel (effects.rs):**
```rust
pub fn apply_repel(target: Entity, distance: u8, owner: PlayerSide, world: &mut World) -> u8 {
    if distance == 0 { return current_cell; } // card authoring rule: REPEL 0 no-op
    if keyword::effects::check_irremovable(target, world) {
        emit_displacement_event(target, DisplacementKind::Repel, current_cell, current_cell, true, world);
        return current_cell;
    }
    let dest = keyword::movement::repel_destination(current_cell, owner, distance);
    // TODO (OQ-KS4): iterate intermediate cells for Trap triggers
    update_unit_position(target, dest, world);
    emit_displacement_event(target, DisplacementKind::Repel, current_cell, dest, false, world);
    dest
}
```

**apply_attract (effects.rs) — same pattern with attract_destination.**

**apply_teleport (effects.rs):**
- No IRREMOVABLE check override (IRREMOVABLE blocks; same `check_irremovable()` call)
- No APPEARANCE trigger (TELEPORT ≠ board entry from PlacementBuffer)
- No COUNTERATTACK trigger at destination (TELEPORT is not a melee advance)
- Cross-lane TELEPORT allowed only if card text specifies destination lane
- Co-occupation allowed at destination cell
- Spawn-range restrictions do NOT apply (PLACEMENT rule only)

**apply_change_lane (effects.rs):**
```rust
pub fn apply_change_lane(unit: Entity, target_lane: u8, world: &mut World) -> bool {
    // 1. Check destination lane for friendly Minion occupancy (BoardState query)
    let occupied = board_state_has_friendly_minion(target_lane, unit_owner, world);
    if occupied { return false; } // rejected silently
    // 2. Update unit lane (not cell — same cell, different lane)
    update_unit_lane(unit, target_lane, world);
    true
}
```

**CHARGE X (SS2) — same blocking rules as SS5 (KW-028):**
- CHARGE X advance in SS2 uses same cell-by-cell movement logic as SS5
- WALL collision halts CHARGE X advance at WALL's cell
- STUN suppresses CHARGE X (checked before SS2 execution)

**Strich auto-CHANGE LANE (KW-033a/c):**
- Triggered when enemy unit enters Strich's lane in SS1 (via APPEARANCE event)
- Check adjacent lanes: if 0 valid → rejected (KW-033c); if 1 valid → change to it (KW-033a); if 2 valid → use `strich_change_lane_select` RNG (KW-033b — BLOCKED)

**ATTRACT backfire (KW-041):**
- No special handling needed — unit position updated to Player A's Cell 1 by `apply_attract()`
- SS6 objective damage logic in combat-resolution epic reads unit positions at SS6; unit at Cell 1 (Player A's side) will trigger objective damage for Player A
- No immunity granted by being displaced there

---

## Out of Scope

- KW-033b: Strich + RNG tie-break — BLOCKED (annotated above)
- KW-051, KW-052: REPEL/ATTRACT Trap traversal lethal — BLOCKED (Trap traversal stub exists; full implementation after OQ-KS4)
- Story 002: repel_destination() and attract_destination() pure functions (called from here)
- Story 007: check_irremovable() (called from here)

---

## QA Test Cases

- **AC-1**: KW-028 — CHARGE X blocked by WALL in SS2
  - Given: unit with CHARGE 2 at Cell 1; WALL at Cell 2
  - When: SS2 executes
  - Then: unit halted at Cell 2 (WALL's cell); CHARGE X advance stops; no SS2 pass-through
  - Edge cases: CHARGE X beyond WALL range still halts at WALL

- **AC-2**: KW-029a/b — REPEL displacement + DisplacementEvent
  - Given: Player A unit at Cell 2; REPEL 3 effect (target_owner=PlayerA, advance_dir=+1, negated=-1)
  - When: apply_repel executes
  - Then: unit moves to Cell 1; `DisplacementEvent { from_cell: 2, to_cell: 1, kind: Repel, was_blocked: false }` emitted

- **AC-3**: KW-030 — ATTRACT displacement
  - Given: caster at Cell 3; target (PlayerB) at Cell 7; ATTRACT 4
  - When: apply_attract executes
  - Then: target moves to Cell 3; `DisplacementEvent { from_cell: 7, to_cell: 3, kind: Attract }` emitted

- **AC-4**: KW-031a — TELEPORT does not trigger APPEARANCE
  - Given: unit TELEPORTed to enemy-occupied Cell 5
  - When: apply_teleport executes
  - Then: unit position updated; no `UnitAppeared` event fired; no APPEARANCE effect triggered
  - Edge cases: unit entering via PlacementBuffer in SS1 DOES trigger APPEARANCE — TELEPORT explicitly does not

- **AC-5**: KW-031b — TELEPORT does not trigger COUNTERATTACK
  - Given: COUNTERATTACK unit at Cell 5; another unit TELEPORTed to Cell 5
  - When: apply_teleport executes
  - Then: COUNTERATTACK does NOT fire; no proximity contact via melee advance occurred
  - Edge cases: `check_and_apply_counterattack()` must NOT be called by apply_teleport

- **AC-6**: KW-032 — CHANGE LANE rejected if destination full
  - Given: unit in Lane 3; Lane 2 already has a friendly Minion
  - When: apply_change_lane(unit, lane=2) called
  - Then: returns false; unit stays in Lane 3; no position update; no error emitted

- **AC-7**: KW-033a — Strich auto-CHANGE LANE (one valid adjacent lane)
  - Given: Strich in Lane 3; Lane 2 valid (no friendly Minion); Lane 4 occupied (friendly Minion); enemy enters Lane 3 in SS1
  - When: SS1 resolves
  - Then: Strich moves to Lane 2; `apply_change_lane(strich, lane=2)` returns true

- **AC-8**: KW-041 — ATTRACT backfire (no immunity)
  - Given: Player A ATTRACT pulls Player B unit to Player A's Cell 1
  - When: SS6 objective damage resolves
  - Then: Player A's objective HP decremented by Player B unit's net ATK value
  - Edge cases: intentional design — no special immunity granted by displacement

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/keyword/displacement_keywords_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 (scaffold), Story 002 (movement formulas — repel/attract_destination), Story 007 (IRREMOVABLE check — check_irremovable)
- Depends on: board-lane-system epic (position update API, BoardState lane occupancy)
- Unlocks: Story 010 (BODYGUARD bond survives CHANGE LANE — KW-053 test)

**Permanently BLOCKED sub-stories:**
- KW-033b: awaiting ADR-005 `strich_change_lane_select` seed slot
- KW-051/052: awaiting OQ-KS4 Trap design
