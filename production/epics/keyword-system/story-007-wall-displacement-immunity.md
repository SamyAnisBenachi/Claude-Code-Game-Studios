# Story 007: WALL + IRREMOVABLE + UNTARGETABLE

> **Epic**: Keyword System
> **Status**: Blocked
> **Layer**: Feature (M3)
> **Type**: Logic
> **Manifest Version**: 2026-04-30

## Context

**GDD**: `design/gdd/keyword-system.md`
**Requirement**: TR-KW-009 (WALL keyword: MP=0 unit; halts advancing enemy at adjacent cell); TR-KW-??? (IRREMOVABLE — untraced); TR-KW-??? (UNTARGETABLE — untraced)
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

> ⚠️ IRREMOVABLE and UNTARGETABLE have no TR-ID in the registry. Stories use TR-KW-??? placeholders. Run `/architecture-review` to register missing TRs.

**ADR Governing Implementation**: ADR-018 (effects.rs)
**ADR Decision Summary**: WALL, IRREMOVABLE, and UNTARGETABLE are stateless keywords (no field in `UnitKeywordState`). They are checked via `has_keyword(SimpleKeyword::Wall/Irremovable/Untargetable)` on the unit's card definition at the point of evaluation. IRREMOVABLE check happens in `check_irremovable()` before any displacement is applied; if true, emit `DisplacementEvent { was_blocked: true }` and return without moving the unit.

**BLOCKED**: ADR-018 Proposed. Story 001 must be Done.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**: WALL collision logic must be in the SS5 movement system within `resolve_combat`. IRREMOVABLE check must occur before `apply_repel`, `apply_attract`, and `apply_teleport` calls.

**Control Manifest Rules (Feature layer)**:
- Required: WALL collision logic runs in SS5 (standard movement) — WALL does not self-move (MP=0); enemy halts at WALL's cell and deals/takes damage in SS6
- Forbidden: Never allow WALL to deal damage — WALL always deals 0 damage in SS6 regardless of ATK card stat

---

## Acceptance Criteria

*From GDD `design/gdd/keyword-system.md` Acceptance Criteria:*

- [ ] KW-018: GIVEN a WALL unit is at Cell 4 and an enemy unit has MP sufficient to reach or pass Cell 4, WHEN SS5 movement resolves, THEN the enemy unit stops at Cell 4 and fights WALL in SS6; WALL deals 0 damage
- [ ] KW-020a: GIVEN an IRREMOVABLE unit is the target of REPEL X, WHEN REPEL resolves, THEN the unit's cell position does not change; the Void flat flash animation event is emitted
- [ ] KW-020b: GIVEN an IRREMOVABLE unit is the target of ATTRACT X, WHEN ATTRACT resolves, THEN the unit's cell position does not change
- [ ] KW-020c: GIVEN an IRREMOVABLE unit is the target of TELEPORT, WHEN TELEPORT resolves, THEN the unit's cell position does not change
- [ ] KW-021: GIVEN an UNTARGETABLE unit is in combat range of an enemy RANGE unit, WHEN SS6 resolves, THEN the RANGE attack hits the UNTARGETABLE unit normally; UNTARGETABLE only blocks Spell/Order targeting
- [ ] IRREMOVABLE does NOT affect the unit's own movement (MP, CHARGE X, CHANGE LANE) — only blocks external displacement
- [ ] WALL is not IRREMOVABLE by default — WALL can be displaced by REPEL/ATTRACT/TELEPORT unless it also carries the IRREMOVABLE keyword
- [ ] SILENCEd WALL loses blocking behavior (KW-036 — see also Story 006): enemies advance past Cell 4; WALL retains MP=0 card stat (cannot self-move)

---

## Implementation Notes

*Derived from ADR-018 effects.rs interface and GDD Detailed Design:*

**WALL collision logic (SS5):**
- WALL unit has MP=0 (card stat) — it cannot self-move in SS5
- Advancing enemy movement: iterate enemy's movement path; if enemy reaches WALL's cell, halt enemy there; fight WALL in SS6
- WALL deals 0 damage in SS6: `if has_keyword(SimpleKeyword::Wall) { outgoing_damage = 0; }`
- SILENCE check: before applying WALL collision, check `silenced_until_round` — SILENCEd WALL is treated as a normal unit (no collision halt)

**check_irremovable pattern:**
```rust
// Called before any displacement attempt in apply_repel / apply_attract / apply_teleport
pub fn check_irremovable(target: Entity, world: &World) -> bool {
    let card_def = get_card_definition(target, world);
    card_def.has_keyword(SimpleKeyword::Irremovable)
}

// In apply_repel() / apply_attract() / apply_teleport():
if keyword::effects::check_irremovable(target, world) {
    emit_displacement_blocked_event(target, keyword, world);
    return; // unit does not move
}
```

**DisplacementEvent when IRREMOVABLE blocks:**
- `DisplacementEvent { unit_id, keyword: DisplacementKind::Repel/Attract/Teleport, from_cell: current_cell, to_cell: current_cell, was_blocked: true, sub_step }`
- Client plays Void flat flash animation (1-frame, 15% opacity, fades 100ms) instead of slide animation

**UNTARGETABLE — server enforcement:**
- `UNTARGETABLE` only blocks Spell/Order targeting; does NOT prevent standard combat or RANGE attacks
- Server validates all `C2SActivateCard` targeting: if target has `has_keyword(SimpleKeyword::Untargetable)` and card type is Spell or Order → reject (silent discard per NP protocol)
- RANGE attack target selection is unaffected by UNTARGETABLE (proximity selection, not Spell/Order targeting)

**IRREMOVABLE vs own movement:**
- `check_irremovable()` only applies to external displacement (REPEL, ATTRACT, TELEPORT from card effects)
- IRREMOVABLE unit's own MP movement (SS5), CHARGE X (SS2), and CHANGE LANE are all unaffected

---

## Out of Scope

- Story 006: SILENCE+WALL (KW-036) — SILENCE system; referenced here for WALL collision pre-check
- Story 016: `apply_repel()`, `apply_attract()`, `apply_teleport()` full implementations — call `check_irremovable()` from here

---

## QA Test Cases

- **AC-1**: KW-018 — WALL halts enemy; deals 0 damage
  - Given: WALL unit at Cell 4 (MP=0); enemy unit at Cell 1 with MP=5 (enough to reach Cell 4+)
  - When: SS5 movement resolves
  - Then: enemy unit stops at Cell 4; in SS6, enemy attacks WALL (taking damage); WALL deals 0 damage to enemy
  - Edge cases: WALL with ATK=3 card stat still deals 0 damage; WALL is not destroyed by 0-damage SS6 combat if HP > 0

- **AC-2**: KW-020a — IRREMOVABLE blocks REPEL
  - Given: IRREMOVABLE unit at Cell 5; REPEL 3 effect targets it
  - When: REPEL effect resolves
  - Then: unit stays at Cell 5; `DisplacementEvent { was_blocked: true, from_cell: 5, to_cell: 5 }` emitted
  - Edge cases: IRREMOVABLE unit's own SS5 movement (MP=2) still works — IRREMOVABLE only blocks external displacement

- **AC-3**: KW-020b — IRREMOVABLE blocks ATTRACT
  - Given: IRREMOVABLE unit at Cell 7; ATTRACT 4 effect targets it
  - When: ATTRACT effect resolves
  - Then: unit stays at Cell 7; displacement blocked event emitted
  - Edge cases: caster of ATTRACT still at its cell (ATTRACT targets the IRREMOVABLE unit, not caster)

- **AC-4**: KW-020c — IRREMOVABLE blocks TELEPORT
  - Given: IRREMOVABLE unit; TELEPORT card targets it to Cell 2
  - When: TELEPORT resolves
  - Then: unit stays at current cell; displacement blocked event emitted

- **AC-5**: KW-021 — UNTARGETABLE does not block RANGE attacks
  - Given: UNTARGETABLE unit at Cell 6; enemy RANGE attacker at Cell 3 (within range)
  - When: SS6 resolves
  - Then: RANGE attacker selects UNTARGETABLE unit as nearest enemy; attack resolves normally (full damage)
  - Edge cases: UNTARGETABLE does block Spells/Orders; RANGE combat targeting is not a Spell/Order

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/keyword/wall_displacement_immunity_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 (scaffold)
- Depends on: Story 006 (SILENCE — for SILENCEd WALL test KW-036; SILENCE system must exist)
- Unlocks: Story 016 (displacement keywords — call `check_irremovable()` from apply_repel/attract/teleport)
