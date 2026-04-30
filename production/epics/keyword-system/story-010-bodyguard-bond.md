# Story 010: BODYGUARD Bond Management

> **Epic**: Keyword System
> **Status**: Blocked
> **Layer**: Feature (M3)
> **Type**: Logic
> **Manifest Version**: 2026-04-30

## Context

**GDD**: `design/gdd/keyword-system.md`
**Requirement**: TR-KW-012 — BODYGUARD protection: unit-to-unit bond stored as `UnitKeywordState.bodyguard_protects: Option<EntityId>`; survives CHANGE LANE
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-018 (Keyword System — ECS State Architecture, Part 1 bodyguard_protects field)
**ADR Decision Summary**: `bodyguard_protects: Option<Entity>` on the BODYGUARD entity — typed Bevy handle, NOT a lane/cell index. Stable across CHANGE LANE (entity ID unchanged by position moves). `bodyguard_cleanup_system` runs in PostUpdate using `&Entities` for alive-check; clears stale refs when BODYGUARD despawns. Protection is board-wide (can protect any friendly unit regardless of lane).

**BLOCKED**: ADR-018 Proposed. Story 001 must be Done.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**: `bodyguard_cleanup_system` uses `&Entities` system param for O(1) alive-check in Bevy 0.18 — verify exact symbol path against Bevy 0.18 docs before coding (`bevy::ecs::entity::Entities`). `bodyguard_cleanup_system` runs in PostUpdate (after despawn commands flush) to guarantee it executes after unit removal.

**Control Manifest Rules (Feature layer)**:
- Required: BODYGUARD bond stored as `Option<Entity>` — NEVER as lane index or lane-scoped attribute (ADR-018)
- Forbidden: Never serialize `bodyguard_protects: Option<Entity>` into protocol types — use `EntityId` (session-scoped u32) in network messages (ADR-002)

---

## Acceptance Criteria

*From GDD `design/gdd/keyword-system.md` Acceptance Criteria:*

- [ ] KW-019: GIVEN unit B is protected by BODYGUARD unit G; G receives lethal damage in SS3 or SS6 and HP reaches 0, WHEN G's HP is reduced to 0, THEN unit B's Spell/Order protection ends at that instant (not at SS4 removal); from the next PLACEMENT phase, B is targetable by opponent Spells and Orders
- [ ] KW-038: GIVEN unit X is BODYGUARD-protected and an enemy RANGE unit's proximity selection identifies X as the nearest enemy, WHEN the RANGE attack resolves, THEN BODYGUARD does not intercept; X can be hit by RANGE regardless of BODYGUARD protection
- [ ] KW-053: GIVEN BODYGUARD unit G (Lane 3) protects unit P (Lane 3); P executes CHANGE LANE from Lane 3 to Lane 2, WHEN P is in Lane 2, THEN P is still protected — opponent Spell/Order targeting P is still blocked; G's `bodyguard_protects` field still references P's entity ID
- [ ] `BodyguardBondCreated { bodyguard_id, protected_id, sub_step: 1 }` emitted in SS1 when BODYGUARD enters play and controller chooses protected unit
- [ ] `BodyguardBondBroken { bodyguard_id }` emitted when BODYGUARD's HP reaches 0 (in the sub-step of death, before SS4 removal)
- [ ] `bodyguard_cleanup_system` clears `bodyguard_protects = None` within one PostUpdate frame of BODYGUARD entity despawn (integration test: bond cleared within 1 frame of despawn)
- [ ] BODYGUARD protection is board-wide: controller may name any friendly unit regardless of lane

---

## Implementation Notes

*Derived from ADR-018 Part 1 and GDD BODYGUARD rules:*

**apply_bodyguard_bond signature:**
```rust
pub fn apply_bodyguard_bond(bodyguard: Entity, protected: Entity, world: &mut World)
```
Sets `bodyguard_kw_state.bodyguard_protects = Some(protected)` in SS1 when BODYGUARD's APPEARANCE fires and controller names the protected unit.

**bodyguard_cleanup_system (ADR-018 Part 2):**
```rust
pub fn bodyguard_cleanup_system(
    mut units: Query<&mut UnitKeywordState>,
    entities: &Entities, // bevy::ecs::entity::Entities — O(1) alive check
) {
    for mut kw_state in units.iter_mut() {
        if let Some(bond_target) = kw_state.bodyguard_protects {
            if !entities.contains(bond_target) {
                kw_state.bodyguard_protects = None;
            }
        }
    }
}
```
Runs in `PostUpdate` — after despawn commands applied. Uses `&Entities` NOT a query (avoids needing the protected entity to still have components).

**Bond semantics:**
- Unidirectional: `bodyguard_protects` on BODYGUARD entity; protected unit carries NO field
- Server tracks bond; client reconstructs connector procedurally from `UnitBoardState.bodyguard_protects: Option<EntityId>` in snapshot
- Bond is entity-scoped: stable across CHANGE LANE because entity ID doesn't change when position changes

**Protection ends when (KW-019):**
- BODYGUARD HP reaches 0 → emit `BodyguardBondBroken` immediately in the sub-step of death (SS3 or SS6), before SS4 removal
- `bodyguard_protects` NOT explicitly cleared at death-time (rely on cleanup system); but `BodyguardBondBroken` signals client immediately

**RANGE bypass (KW-038):**
- BODYGUARD protection only blocks Spell/Order targeting (same as UNTARGETABLE — Spells/Orders)
- RANGE attack target selection uses proximity (nearest cell) — not "targeting" in the Spell/Order sense
- Server's `apply_range_attack()` does NOT check `bodyguard_protects` when selecting targets

---

## Out of Scope

- Story 007: UNTARGETABLE (complementary protection mechanism)
- Story 016: CHANGE LANE implementation (bond survival verified here; CHANGE LANE itself in Story 016)

---

## QA Test Cases

- **AC-1**: KW-019 — Bond broken when BODYGUARD HP reaches 0
  - Given: BODYGUARD G protects unit P; G receives lethal damage in SS6 (HP→0)
  - When: G's HP reaches 0 in SS6
  - Then: `BodyguardBondBroken { bodyguard_id: G }` emitted at sub_step=6; P is no longer protected from Spell/Order targeting from next PLACEMENT phase onward
  - Edge cases: bond broken at instant of HP=0, NOT at SS4 removal; protection ends before SS4 cleanup

- **AC-2**: KW-038 — RANGE bypasses BODYGUARD
  - Given: unit X protected by BODYGUARD G; enemy RANGE unit selects nearest enemy (X is nearest)
  - When: RANGE attack resolves
  - Then: RANGE attack hits X directly; G does not intercept; X takes damage
  - Edge cases: RANGE target selection never checks `bodyguard_protects`

- **AC-3**: KW-053 — Bond survives CHANGE LANE
  - Given: G (Lane 3) protects P (Lane 3); P executes CHANGE LANE to Lane 2
  - When: P is now in Lane 2
  - Then: `G.bodyguard_protects = Some(P_entity_id)` unchanged; Spell/Order targeting P is still blocked by G
  - Edge cases: entity ID of P does not change on CHANGE LANE (only position data changes)

- **AC-4**: bodyguard_cleanup_system clears bond within 1 frame
  - Given: G protects P; G is despawned in SS4 (removed from world)
  - When: PostUpdate runs after SS4 despawn
  - Then: G.bodyguard_protects = None (entities.contains(P) check passes; but G's own entity is despawned — test via checking all remaining UnitKeywordState.bodyguard_protects for stale refs)
  - Edge cases: cleanup must run AFTER despawn commands are applied (PostUpdate guarantee)

- **AC-5**: Board-wide protection
  - Given: G in Lane 1; P in Lane 5 (different lane)
  - When: G enters play and controller names P as protected unit
  - Then: `G.bodyguard_protects = Some(P_entity_id)`; Spell/Order targeting P is blocked

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/keyword/bodyguard_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 (scaffold)
- Depends on: Story 016 (CHANGE LANE, for KW-053 bond-survives-lane-change test — can implement KW-053 after Story 016 Done)
- Unlocks: Story 011 (RANGE — for KW-038 RANGE bypass test coordination)
