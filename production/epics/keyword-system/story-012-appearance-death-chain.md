# Story 012: Timing Trigger — APPEARANCE + DEATH Chain Observer

> **Epic**: Keyword System
> **Status**: Ready
> **Layer**: Feature (M3)
> **Type**: Logic
> **Manifest Version**: 2026-04-30

## Context

**GDD**: `design/gdd/keyword-system.md`
**Requirement**: TR-KW-??? (APPEARANCE — untraced); TR-KW-??? (DEATH — untraced)
*(Run `/architecture-review` to register APPEARANCE and DEATH TRs in the registry)*

**ADR Governing Implementation**: ADR-022 (Timing Trigger Observer Architecture, Parts 2–3)
**ADR Decision Summary**: APPEARANCE and DEATH use global Bevy Observers (`app.observe(on_unit_appeared)`, `app.observe(on_unit_died)`). DEATH chain managed by `ChainDeathBuffer` (VecDeque) — NOT recursive Observer calls. `execute_ss4` seeds the buffer with lane-ordered initial deaths and drains it sequentially. Every observer handler guards for keyword presence as its first operation.

**BLOCKED**: ADR-022 Proposed. ADR-018 Proposed. Verification Required items 1–2 from ADR-022 must be resolved (`world.trigger_targets()` API, `Trigger<T>` param type). Story 001 (scaffold) and Story 003 (FIRST STRIKE, for KW-035b) must be Done.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**:
- `world.trigger_targets(UnitAppeared { sub_step: 1 }, entity)` — confirm this is a valid `World` method in Bevy 0.18 (Verification Required item 1 from ADR-022)
- `Trigger<T>` (not `On<T>`) is the correct param type for observer handlers (Verification Required item 2)
- Sequential borrow safety: `world.trigger_targets()` followed by `world.resource_mut::<ChainDeathBuffer>()` is safe — each call is a discrete mutable borrow that ends before the next begins

**Control Manifest Rules (Feature layer)**:
- Required: `resolve_combat` fires trigger events; keyword observers own the effects — GDD boundary enforced at code level (ADR-022)
- Forbidden: Never use recursive `world.trigger_targets()` inside an observer handler for DEATH chains — use explicit `ChainDeathBuffer` queue (ADR-022)
- Forbidden: Never use `EventWriter`/`EventReader` for timing triggers — these don't exist in Bevy 0.17+ (use `#[derive(Event)]` + Observer)

---

## Acceptance Criteria

*From GDD `design/gdd/keyword-system.md` Acceptance Criteria, Timing Triggers section:*

- [ ] KW-001: GIVEN a unit with an APPEARANCE trigger enters the board in SS1, WHEN sub-step 1 resolves, THEN the APPEARANCE effect executes before any DEATH trigger chains from APPEARANCE-caused kills
- [ ] KW-002: GIVEN two units in different lanes are killed in the same sub-step, WHEN sub-step 4 removes dead units, THEN DEATH triggers fire in lane order (Lane 1 before Lane 5); Lane 2 unit's DEATH trigger resolves completely before Lane 4 unit's begins
- [ ] KW-003: GIVEN unit A has a DEATH trigger that deals lethal damage to unit B (also has a DEATH trigger), WHEN unit A is removed in SS4, THEN A's DEATH trigger resolves completely, then B is removed, then B's DEATH trigger fires
- [ ] KW-035b: GIVEN unit B has a DEATH trigger and was killed in SS3 by FIRST STRIKE, WHEN SS4 resolves, THEN unit B is removed from the board AND B's DEATH trigger effect executes; kill gold is added to the attacker's gold total in SS4
- [ ] `ChainDeathBuffer` is empty at RESOLUTION end (integration test assertion)
- [ ] All observer handlers guard for keyword presence as first operation (`has_keyword(SimpleKeyword::Appearance/Death)`) — no effect fires on units without the relevant keyword
- [ ] Max DEATH chain depth bounded by board size (≤10 units → ≤9 chain links); no infinite loop possible

---

## Implementation Notes

*Derived from ADR-022 Parts 2–3 and GDD Edge Cases:*

**on_unit_appeared handler structure:**
```rust
pub fn on_unit_appeared(
    trigger: Trigger<UnitAppeared>,       // Trigger<T> — verify item 2
    units: Query<(&UnitKeywordState, &CardId)>,
    card_catalog: Res<CardCatalog>,
    mut keyword_triggered: MessageWriter<KeywordTriggered>, // verify item 4
) {
    let entity = trigger.target();
    let Ok((kw_state, card_id)) = units.get(entity) else { return; };
    if !kw_state.has_keyword(SimpleKeyword::Appearance) { return; } // mandatory guard
    // Execute APPEARANCE effect (per card definition in CardCatalog)
}
```

**execute_ss4 pattern (ADR-022 Part 3):**
```rust
fn execute_ss4(world: &mut World) {
    // 1. Clear ChainDeathBuffer defensively
    world.resource_mut::<ChainDeathBuffer>().0.clear();
    // 2. Collect units with HP <= 0; sort by lane order (Lane 1 first)
    let initial_deaths = collect_dead_units_lane_ordered(world);
    // 3. Seed buffer
    world.resource_mut::<ChainDeathBuffer>().0.extend(
        initial_deaths.into_iter().map(|e| (e, None))
    );
    // 4. Drain loop — on_unit_died may push new entries
    loop {
        let next = world.resource_mut::<ChainDeathBuffer>().0.pop_front();
        let Some((entity, attacker)) = next else { break };
        board::api::remove_unit_from_board(world, entity);
        economy::api::award_kill_gold(world, attacker, entity);
        world.trigger_targets(UnitDied { attacker }, entity); // verify item 1
    }
    // 5. Clear defensively again
    world.resource_mut::<ChainDeathBuffer>().0.clear();
}
```

**on_unit_died handler — chain growth:**
```rust
pub fn on_unit_died(
    trigger: Trigger<UnitDied>,
    units: Query<(&UnitKeywordState, &UnitBoardOwner)>,
    mut chain_buffer: ResMut<ChainDeathBuffer>, // verify item 3
    mut keyword_triggered: MessageWriter<KeywordTriggered>, // verify item 4
) {
    let entity = trigger.target();
    let Ok((kw_state, owner)) = units.get(entity) else { return; };
    if !kw_state.has_keyword(SimpleKeyword::Death) { return; } // mandatory guard
    // Apply DEATH effect; if any unit's HP drops to 0, push to chain_buffer:
    // chain_buffer.0.push_back((dying_entity, Some(trigger.target())));
}
```

**Chain termination guarantee:** `remove_unit_from_board` removes entity from board position index — subsequent queries exclude it. A unit can only die once (already-removed entities not queryable). Max 9 chain links for 10-unit board.

**KW-001 sequencing:** SS1 APPEARANCE observer fires synchronously via `world.trigger_targets(UnitAppeared { sub_step: 1 }, entity)` per-spawn. The SS4 loop runs AFTER all SS1 observers complete — structural guarantee.

---

## Out of Scope

- Story 003: FIRST STRIKE kills that trigger DEATH in SS4 (KW-035b dependency on Story 003)
- Story 013: FINAL BLOW trigger fired in SS3 (before SS4 DEATH chain)

---

## QA Test Cases

- **AC-1**: KW-001 — APPEARANCE fires before DEATH chain
  - Given: unit A has APPEARANCE trigger (deals lethal damage to unit B in SS1); unit B has DEATH trigger
  - When: SS1 resolves
  - Then: A's APPEARANCE effect fires (B's HP → 0); SS4 DEATH chain then fires B's DEATH trigger; A's APPEARANCE completed before B's DEATH fires
  - Edge cases: APPEARANCE-caused deaths are NOT processed in SS1 — they queue to SS4 DEATH chain

- **AC-2**: KW-002 — Simultaneous deaths fire in lane order
  - Given: units in Lane 2 and Lane 4 die simultaneously in same sub-step
  - When: SS4 resolves
  - Then: Lane 2's DEATH trigger fires first and resolves completely; Lane 4's DEATH trigger then fires
  - Edge cases: ChainDeathBuffer seeded with `collect_dead_units_lane_ordered()` — VecDeque preserves insertion order

- **AC-3**: KW-003 — DEATH chain sequential
  - Given: unit A has DEATH trigger that deals lethal damage to unit B; B has DEATH trigger
  - When: A dies in SS4
  - Then: A's DEATH trigger fires; A's effect kills B; A's observer handler pushes B to ChainDeathBuffer; A's handler completes; then B is dequeued; B's DEATH trigger fires
  - Edge cases: B's DEATH trigger NOT fired inside A's observer handler — must wait for queue drain

- **AC-4**: KW-035b — DEATH trigger fires in SS4 after FIRST STRIKE kill in SS3
  - Given: FIRST STRIKE attacker kills unit B (has DEATH trigger) in SS3
  - When: SS4 drain loop runs
  - Then: B is removed from board; B's DEATH trigger fires; kill gold awarded to attacker
  - Edge cases: B is present in board state at SS3 completion (removal deferred to SS4)

- **AC-5**: ChainDeathBuffer empty at RESOLUTION end
  - Given: any RESOLUTION with DEATH chains
  - When: RESOLUTION ends
  - Then: `ChainDeathBuffer.0.is_empty() == true`

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/keyword/appearance_death_chain_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 (scaffold — ChainDeathBuffer + observers.rs stubs exist), Story 003 (FIRST STRIKE — for KW-035b)
- Depends on: ADR-022 Verification Required items 1–2 resolved
- Unlocks: Story 013 (FINAL BLOW fires in SS3, before SS4 DEATH chain)
