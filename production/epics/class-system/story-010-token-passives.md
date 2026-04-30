# Story 010: Token Passive Behaviors — Sinistro, La Gonflable, La Sacrifiée

> **Epic**: Class System
> **Status**: Ready
> **Layer**: Feature (M3)
> **Type**: Integration
> **Manifest Version**: 2026-04-30

## Context

**GDD**: `design/gdd/class-system.md`
**Requirement**: `TR-CS-009` (partial — passive behaviors; spawn scaffold in Story 002)
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-014: Class System Architecture — PlayerSessionState, SourceClass Component, and Direct Effect Dispatch
**ADR Decision Summary**: Token passives are plain Rust functions called from within the RESOLUTION system body at the appropriate sub-steps. Token entities carry `SourceClass(ClassId)` and `TokenUnit` (from Story 002) — passives are dispatched by checking unit type via ECS queries on `TokenUnit` + `SourceClass`. Sinistro fires at sub-step 6 (recommended: after all combat — confirm with combat-resolution GDD). La Gonflable fires END-OF-MOVEMENT (sub-step 5 completion). La Sacrifiée fires on DEATH trigger.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: MEDIUM
**Engine Notes**:
- DEATH trigger: Bevy 0.17+ uses `#[derive(Event)]` + `commands.entity(unit).observe(on_death_event)` for per-entity reactive triggers (per `current-best-practices.md`). La Sacrifiée's DEATH passive must use this Observer pattern, NOT `EventWriter`/`EventReader` (removed in 0.17+).
- `query.single()` returns `Result` in Bevy 0.16+. Use `let Ok(x) = query.single()` or `query.single()?`.
- Sinistro sub-step assignment: GDD NP-6 flags that Sinistro's sub-step is unspecified — recommended sub-step 6, after all combat. Confirm with combat-resolution GDD and close NP-6 before this story opens for implementation.
- NP-6 (Sinistro ResolutionEvent gaps), NP-3 (UnitSpawned variant) block integration-level S2C assertions only; unit-level ObjectiveState assertions are independently testable.
- ADR-014 is NOT yet in the control manifest.

**Control Manifest Rules (Feature Layer)**:
- Required: Reactive keyword triggers (DEATH, APPEARANCE, FINAL BLOW) use `#[derive(Event)]` + `commands.entity(unit).observe(...)` — current-best-practices.md
- Required: Feature systems react to RSM Messages; never observe RoundState directly — ADR-010
- Forbidden: Never use `EventWriter`/`EventReader` — ADR-009
- Guardrail: RESOLUTION batch budget ≤ 15ms — ADR-002

---

## Acceptance Criteria

*From GDD `design/gdd/class-system.md`, CS-AC-28 through CS-AC-30:*

- [ ] **CS-AC-28** GIVEN a Sinistro spell is placed on a friendly objective in lane 2, WHEN a RESOLUTION completes (after all sub-steps conclude), THEN the enemy objective in lane 2 has taken 1 damage; Sinistro remains attached to its parent objective.
- [ ] **CS-AC-29** GIVEN a La Gonflable token (HP=3/ATK=2/MP=3) is in play in lane 3 and at least one other friendly unit is also present in lane 3, WHEN La Gonflable's movement ends during RESOLUTION sub-step 5, THEN each other friendly unit in lane 3 is healed for 2 HP (capped at that unit's max HP); La Gonflable itself is not healed.
- [ ] **CS-AC-30** GIVEN a La Sacrifiée token (HP=2/ATK=2/MP=3) is in play in lane 4, WHEN La Sacrifiée is destroyed (HP → 0) during RESOLUTION, THEN each enemy unit present in lane 4 at the moment of destruction takes 1 damage (routed through the AR reduction pipeline; effective damage = max(0, 1 − unit.AR) per enemy unit in the lane).

---

## Implementation Notes

*Derived from ADR-014 Decision §4 and GDD Detailed Rules §Token registry:*

**File location**: `server/src/core/resolution/effects.rs`

**CS-AC-28 — Sinistro 1 damage per RESOLUTION**:
```rust
/// Called at sub-step 6 (after all combat concludes) for each alive Sinistro entity.
/// Sinistro is a spell-attached entity on a friendly objective — it has a parent_lane.
pub fn apply_sinistro_damage(
    board: &BoardState,
    objectives: &mut ObjectiveState,
    owner: PlayerId,
) {
    for sinistro in board.alive_sinistros(owner) {
        let lane = sinistro.parent_lane;
        if !objectives.is_alive(owner, lane) {
            // Sinistro is destroyed if its parent objective takes damage — see GDD rule.
            // If parent already destroyed, skip (Sinistro is implicitly gone).
            continue;
        }
        objectives.take_damage(owner.opponent(), lane, owner, 1);
        // Sinistro remains attached (not removed here)
    }
}
```

NP-6 flags: (1) No `DamageKind` variant to discriminate Sinistro damage; (2) No `SinistroDestroyed` event for client. Both are NP-GDD gaps — unit test asserts ObjectiveState HP change; client rendering awaits NP-6 resolution.

**CS-AC-29 — La Gonflable END-OF-MOVEMENT heal**:
```rust
/// Called after La Gonflable's sub-step 5 movement completes.
pub fn apply_la_gonflable_heal(
    board: &mut BoardState,
    gonflable_entity: EntityId,
    lane: u8,
    owner: PlayerId,
) {
    for unit in board.friendly_units_in_lane(lane, owner) {
        if unit.entity_id == gonflable_entity { continue; }  // self-exclusion
        let heal = 2u32;
        unit.hp = unit.hp.saturating_add(heal).min(unit.max_hp);  // cap at max_hp
    }
}
```

**CS-AC-30 — La Sacrifiée DEATH trigger**:

Using Bevy 0.17+ Observer pattern (NOT EventWriter):
```rust
// In plugin setup:
// commands.entity(la_sacrifiee_entity).observe(on_la_sacrifiee_death);

fn on_la_sacrifiee_death(
    trigger: Trigger<UnitDied>,
    mut board: ResMut<BoardState>,
    sacrifiee_query: Query<&BoardPosition, With<SourceClass>>,
) {
    let Ok(position) = sacrifiee_query.get(trigger.entity()) else { return; };
    let lane = position.lane;
    let attacker = position.owner;
    for enemy_unit in board.enemy_units_in_lane(lane, attacker.opponent()) {
        let effective_dmg = 1u32.saturating_sub(enemy_unit.ar);
        if effective_dmg > 0 {
            enemy_unit.hp = enemy_unit.hp.saturating_sub(effective_dmg);
        }
    }
}
```

1 damage is pre-AR (routed through AR reduction pipeline per GDD CS-AC-30). Effective damage = max(0, 1 − unit.AR). This may be 0 for units with AR ≥ 1 — that is correct behavior, not a bug.

**Sinistro destruction rule**: Sinistro is destroyed if its parent objective takes damage. The Board/Lane System must fire a `SinistroDestroyed` signal (NP-6 gap) when the parent objective is damaged. This story's `apply_sinistro_damage` checks `is_alive(parent_objective)` before dealing damage; the cleanup of the Sinistro entity on parent-objective-damage is a Board/Lane + Objective System concern.

---

## Out of Scope

*Handled by neighbouring stories:*

- Story 002: Token spawn functions (spawn_sinistro, spawn_la_gonflable, spawn_la_sacrifiee) — must be DONE
- Story 006: Sacrier effects — Objective System integration pattern established there
- Objective System: `take_damage()` implementation for Sinistro damage — `objective-system` epic
- Combat Resolution: DEATH trigger infrastructure for La Sacrifiée — `combat-resolution` epic
- NP-6 (Sinistro ResolutionEvent, DamageKind, SinistroDestroyed event) — Network Protocol epic; blocks client-side rendering only, not server logic
- Madoll spell-cost-reduction passive — not in these ACs; Madoll passive is a Card Acquisition / Economy concern when Madoll is in play; not implemented in this epic

---

## QA Test Cases

*Integration story — automated test specs. World-level assertions on ObjectiveState and BoardState; Observer-based DEATH trigger requires at minimum a real Bevy `World` with Observer registration.*

- **AC CS-AC-28 — Sinistro deals 1 dmg to opponent lane objective**:
  - Given: Sinistro entity alive, attached to owner's lane 2 objective (both alive); opponent's lane 2 objective HP=5
  - When: `apply_sinistro_damage(&board, &mut objectives, owner)` called at sub-step 6
  - Then: opponent lane 2 objective HP = 4; Sinistro entity still present (not removed)
  - Edge cases: parent objective HP → 0 before sub-step 6 → Sinistro skips (is_alive check); multiple Sinistros in different lanes → each deals 1 dmg to its respective opponent lane

- **AC CS-AC-29 — La Gonflable heals other friendly units in lane**:
  - Given: La Gonflable at HP=3 in lane 3; friendly unit A in lane 3 at HP=2 (max_hp=5); friendly unit B in lane 3 at HP=4 (max_hp=4); La Gonflable finished moving
  - When: `apply_la_gonflable_heal(&mut board, gonflable_id, lane=3, owner)` called
  - Then: unit A HP = 4 (+2); unit B HP = 4 (capped at max_hp=4, not 6); La Gonflable HP = 3 (self-excluded, no heal)
  - Edge cases: no other friendly units in lane → no healing; unit at max HP → HP unchanged (saturating_add.min(max_hp))

- **AC CS-AC-30 — La Sacrifiée DEATH triggers 1 dmg to enemy lane units**:
  - Given: La Sacrifiée in lane 4; 2 enemy units in lane 4 (unit A AR=0, unit B AR=2); La Sacrifiée HP → 0 (destroyed)
  - When: `UnitDied` Observer fires for La Sacrifiée entity
  - Then: unit A takes 1 effective damage (HP -= 1; AR=0, so max(0, 1−0)=1); unit B takes 0 effective damage (AR=2, max(0, 1−2)=0)
  - Edge cases: no enemy units in lane 4 → no damage; La Sacrifiée destroyed by own effect → still triggers (DEATH fires regardless of cause); enemy unit HP → 0 from this damage → standard DEATH handling fires for that unit

---

## Test Evidence

**Story Type**: Integration
**Required evidence**: `tests/integration/class/token_passives_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 002 (token spawn scaffold — spawn_sinistro, spawn_la_gonflable, spawn_la_sacrifiee, SourceClass, TokenUnit) — must be DONE
- Depends on: `objective-system` epic (take_damage(), is_alive() for Sinistro) — must be DONE
- Depends on: `combat-resolution` epic (DEATH trigger infrastructure using Bevy Observer; LA_SACRIFIEE's `on_la_sacrifiee_death` observer registration; sub-step 5 movement completion callback for La Gonflable) — must be DONE
- Depends on: NP-6 resolution (Sinistro ResolutionEvent gaps) — blocks client rendering; server logic independently testable
- Unlocks: None — final story in the Class System epic
