# Story 012: DEATH Chain Observer (on_unit_died)

> **Epic**: Keyword System
> **Status**: Blocked
> **Layer**: Feature (M3)
> **Type**: Integration
> **Manifest Version**: 2026-04-30

## Context

**GDD**: `design/gdd/keyword-system.md`
**Requirement**: `TR-KW-???` — DEATH chain timing triggers have no registered TR-ID. Use placeholder. Run `/architecture-review` to register missing TRs before marking this story Done.
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-022 (Keyword System — Timing Trigger Observer Architecture, Part 3: DEATH Chain Explicit Queue Architecture)
**ADR Decision Summary**: DEATH triggers dispatched via `on_unit_died` global Observer; simultaneous deaths seeded into `ChainDeathBuffer` (`VecDeque`) in lane order (Lane 1 first) and drained sequentially — A's effect completes before B fires. Recursive `world.trigger_targets()` inside the observer handler is explicitly rejected; the explicit queue is always safe. `ChainDeathBuffer` cleared at SS4 start before seeding.

**BLOCKED**: ADR-018 is Proposed (provides `UnitKeywordState` component and module structure that this story builds on). ADR-022 is Accepted. Story 001 (scaffold, which registers `on_unit_died` and `ChainDeathBuffer`) must be Done.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**:
- `world.trigger_targets(UnitDied { attacker }, entity)` — CONFIRMED valid `World` method in Bevy 0.18 (ADR-022 Verification Required item 1 resolved)
- `Trigger<UnitDied>` — CONFIRMED correct observer param type (not `On<UnitDied>`) (ADR-022 item 2 resolved)
- `ResMut<ChainDeathBuffer>` inside Observer handler — CONFIRMED valid (ADR-022 item 3 resolved); borrow ends before next `trigger_targets()` call
- Sequential borrow safety: `pop_front()` returns owned value; `ChainDeathBuffer` borrow ends before `world.trigger_targets()` call

**Control Manifest Rules (Feature layer)**:
- Required: Guard check (`has_keyword(SimpleKeyword::Death)`) must be the FIRST operation in `on_unit_died` — global observers fire for ALL entities receiving the trigger (ADR-022)
- Required: `ChainDeathBuffer` CLEARED at SS4 start before `extend()` seeds initial deaths — clearing at end only is insufficient for crash-recovery safety (ADR-022)
- Required: Initial deaths sorted in ascending lane order before seeding into `ChainDeathBuffer` (ADR-022)
- Forbidden: Never call `world.trigger_targets(UnitDied { .. }, entity)` recursively inside `on_unit_died` — push chain deaths to `ChainDeathBuffer` instead (ADR-022)
- Forbidden: Never process a DEATH trigger for an entity that has already been processed in this RESOLUTION (already-dead guard, KW-065)

---

## Acceptance Criteria

*From GDD `design/gdd/keyword-system.md` Acceptance Criteria, Timing Triggers section:*

- [ ] KW-002: GIVEN multiple units die simultaneously in SS4 (e.g., mutual SS6 kill, both HP reach 0 in same pass), WHEN SS4 processes deaths, THEN DEATH triggers fire in lane order (Lane 1 before Lane 5); units in the same lane with simultaneous deaths fire in cell order (lower cell first)
- [ ] KW-003: GIVEN unit A's DEATH trigger deals lethal damage to unit B (chaining), WHEN SS4 processes, THEN A's DEATH trigger effect resolves completely before B's DEATH trigger fires; the chain is sequential, not concurrent
- [ ] KW-035b: GIVEN unit B was killed by unit A's FIRST STRIKE attack in SS3, WHEN SS4 resolves, THEN unit B is removed from the board AND B's DEATH trigger effect executes (if B has DEATH keyword); kill gold (+1) is added to attacker A's gold total in SS4 via `apply_gold_award`
- [ ] KW-065: GIVEN unit A's DEATH trigger deals damage to unit B in SS4, and B's DEATH trigger would re-target unit A (which is already dead and removed), WHEN the DEATH chain resolves, THEN the already-dead guard prevents unit A from entering the `ChainDeathBuffer` a second time; the chain terminates without a loop; unit B's DEATH fires once against any valid live targets

---

## Implementation Notes

*Derived from ADR-022 Part 3 (DEATH Chain Explicit Queue Architecture):*

**execute_ss4 pattern (called by `resolve_combat`):**
```rust
fn execute_ss4(world: &mut World) {
    // 1. Collect units with HP <= 0; sort by lane order ascending (Lane 1 first)
    let initial_deaths = collect_dead_units_lane_ordered(world);

    // 2. Clear buffer (defensive), then seed with initial deaths
    world.resource_mut::<ChainDeathBuffer>().0.clear();
    world.resource_mut::<ChainDeathBuffer>()
        .0
        .extend(initial_deaths.into_iter().map(|e| (e, None)));

    // 3. Drain — on_unit_died may push new entries for chain deaths
    //    Already-dead guard: track processed entities to prevent loops (KW-065)
    let mut already_dead: HashSet<Entity> = HashSet::new();
    loop {
        let next = world.resource_mut::<ChainDeathBuffer>().0.pop_front();
        let Some((entity, attacker)) = next else { break };

        if already_dead.contains(&entity) { continue; }  // KW-065 guard
        already_dead.insert(entity);

        board::api::remove_unit_from_board(world, entity);
        economy::api::award_kill_gold(world, attacker, entity);
        world.trigger_targets(UnitDied { attacker }, entity);  // fires on_unit_died
    }

    world.resource_mut::<ChainDeathBuffer>().0.clear();  // defensive
}
```

**on_unit_died handler (ADR-022 Part 4 guard pattern):**
```rust
pub fn on_unit_died(
    trigger: Trigger<UnitDied>,
    units: Query<(&UnitKeywordState, &UnitBoardOwner)>,
    mut chain_buffer: ResMut<ChainDeathBuffer>,
    mut keyword_triggered: MessageWriter<KeywordTriggered>,
) {
    let entity = trigger.target();
    let Ok((kw_state, owner)) = units.get(entity) else { return; };
    if !kw_state.has_keyword(SimpleKeyword::Death) { return; }  // guard FIRST
    // Apply DEATH effect from card definition
    // If effect deals damage reducing another unit to 0 HP: push to chain_buffer
    // chain_buffer.0.push_back((damaged_entity, Some(entity)));
}
```

**Sequential guarantee:** `pop_front()` returns an owned `(Entity, Option<Entity>)`. The `ChainDeathBuffer` borrow ends before `world.trigger_targets()` is called — no simultaneous borrow conflict.

**Chain depth bound:** Max 9 links (≤10 board units; each dies once). `remove_unit_from_board` removes entity from board queries — subsequent scans exclude it; `already_dead` set prevents re-entry.

---

## Out of Scope

- Story 001: observer registration and `ChainDeathBuffer` resource definition (scaffold)
- Story 013: FINAL BLOW (fires in SS3/SS6, not SS4) — DEATH is SS4 only
- Story 015: COUNTERATTACK fired by the same attack that caused the initial death (fires in SS3/SS6, before SS4)
- Story 009: OUTNUMBERED re-evaluation after SS4 deaths (evaluated at SS5 boundary)

---

## QA Test Cases

*Automated test specs (Integration story):*

- **KW-002**: Simultaneous deaths fire in lane order
  - Given: ECS World with units in Lane 1 (HP=0) and Lane 3 (HP=0), both with DEATH triggers; both died in same SS6 pass
  - When: `execute_ss4()` runs
  - Then: Lane 1 unit's DEATH effect fires first; Lane 3 unit's DEATH effect fires second; verify via event log or sequence counter
  - Edge cases: same-lane simultaneous deaths — lower cell fires first

- **KW-003**: DEATH chain sequential completion
  - Given: Unit A (DEATH trigger — damages unit B on death); unit B (DEATH trigger — damages unit C)
  - When: A dies in SS4 → `on_unit_died` fires → B takes lethal damage and enters `ChainDeathBuffer` → B fires → C enters buffer
  - Then: A's full DEATH effect (including any gold/board mutation) completes before B's DEATH fires; B's full effect completes before C's fires; verify C is dead after chain

- **KW-035b**: DEATH after FIRST STRIKE kill
  - Given: Unit B (DEATH trigger; killed by unit A's FIRST STRIKE in SS3)
  - When: SS4 processes unit B's removal
  - Then: B is removed from board AND B's DEATH trigger fires; kill gold (+1) awarded to A; `award_kill_gold` called with A as attacker
  - Edge cases: B's DEATH trigger that kills another unit — that chain death is also processed

- **KW-065**: DEATH chain loop prevention
  - Given: Unit A (DEATH trigger — damages unit B); unit B (DEATH trigger — would target unit A, already dead)
  - When: A dies → A's DEATH fires → B takes lethal damage and enters buffer → B's DEATH fires → looks for A
  - Then: `already_dead.contains(&A_entity)` is true; A does not re-enter chain; chain terminates; no infinite loop
  - Assertion: `ChainDeathBuffer` is empty at RESOLUTION end (`defensive clear` assertion)

---

## Test Evidence

**Story Type**: Integration
**Required evidence**: `tests/integration/keyword/death_chain_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 (scaffold — registers `on_unit_died` observer stub and `ChainDeathBuffer`) must be Done
- Unlocks: Story 013 (FINAL BLOW observer references DEATH chain ordering context for KW-035a)
