# Story 015: Timing Trigger — START OF TURN + END OF TURN

> **Epic**: Keyword System
> **Status**: Ready
> **Layer**: Feature (M3)
> **Type**: Logic
> **Manifest Version**: 2026-04-30

## Context

**GDD**: `design/gdd/keyword-system.md`
**Requirement**: TR-KW-??? (START OF TURN — untraced); TR-KW-??? (END OF TURN — untraced)
*(Run `/architecture-review` to register START OF TURN and END OF TURN TRs in the registry)*

**ADR Governing Implementation**: ADR-022 (Timing Trigger Observer Architecture, Parts 4 + 6)
**ADR Decision Summary**: START OF TURN uses a normal Bevy system (`start_of_turn_dispatch_system`) that reads `MessageReader<DraftPhaseEntered>` and dispatches `StartOfTurnTriggered` via `commands.trigger_targets()` (deferred). END OF TURN uses a global Observer (`on_end_of_turn`) fired synchronously per alive unit inside `resolve_combat` after SS6. Both are observer-based but at different phase entry points.

**BLOCKED**: ADR-022 Proposed. Verification Required item 5 from ADR-022 must be resolved (`commands.trigger_targets()` exists in Bevy 0.18; `DraftPhaseEntered` registered with `app.add_message::<DraftPhaseEntered>()`). Story 001 must be Done.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**:
- `commands.trigger_targets(StartOfTurnTriggered, entity)` — confirm this is valid in Bevy 0.18 (Verification Required item 5 from ADR-022)
- Deferred dispatch: `commands.trigger_targets()` fires when Commands flush — schedule `apply_deferred` after `start_of_turn_dispatch_system` in system set to avoid one-frame gap
- `MessageReader<DraftPhaseEntered>` — confirm `DraftPhaseEntered` is registered with `app.add_message::<DraftPhaseEntered>()` in RSM plugin (ADR-010)

**Control Manifest Rules (Feature layer)**:
- Required: START OF TURN fires at DRAFT phase entry after mana ramp + gold income (RSM Rule 3) — subscribe to `DraftPhaseEntered` event, NOT `RoundState` observation (ADR-010)
- Required: END OF TURN fires after SS6, before `ResolutionComplete` is written (ADR-022)
- Forbidden: Never use `EventWriter`/`EventReader` — use `MessageWriter`/`MessageReader` for Lightyear messages and `#[derive(Event)]` + Observer for ECS events

---

## Acceptance Criteria

*From GDD `design/gdd/keyword-system.md` Acceptance Criteria:*

- [ ] KW-009a: GIVEN a unit with START OF TURN is alive at the start of round R+1, WHEN the DRAFT phase begins after mana ramp + gold income are applied, THEN the START OF TURN effect fires
- [ ] KW-009b: GIVEN a unit with START OF TURN enters play on round R, WHEN round R's DRAFT phase begins, THEN START OF TURN does NOT fire for that unit on round R; it fires on round R+1 if the unit survives (unit placed in SS1 of round R; START OF TURN fires on R+1 entry)
- [ ] KW-010a: GIVEN a unit with END OF TURN is alive when SS6 completes, WHEN RESOLUTION ends, THEN END OF TURN fires before the RSM round counter increments
- [ ] KW-010b: GIVEN a unit with END OF TURN entered play on round R and survives SS6, WHEN RESOLUTION ends for round R, THEN END OF TURN fires — a unit that entered play this round IS eligible for END OF TURN
- [ ] `start_of_turn_dispatch_system` reads `MessageReader<DraftPhaseEntered>` — fires once per DRAFT phase entry, after RSM Rule 3 (mana ramp + gold income already applied by the time this system reads)
- [ ] `on_end_of_turn` fires per alive unit inside `resolve_combat` after SS6, before `MessageWriter<ResolutionComplete>` is written
- [ ] Both observer handlers guard for keyword presence as first operation

---

## Implementation Notes

*Derived from ADR-022 Part 6 and GDD Timing Trigger Catalog:*

**start_of_turn_dispatch_system (ADR-022 Part 6):**
```rust
// server/feature/keyword/observers.rs
pub fn start_of_turn_dispatch_system(
    mut reader: MessageReader<DraftPhaseEntered>,
    units: Query<(Entity, &UnitKeywordState)>,
    mut commands: Commands,
) {
    for _event in reader.read() {
        for (entity, kw_state) in units.iter() {
            if kw_state.has_keyword(SimpleKeyword::StartOfTurn) {
                // Deferred: fires when Commands flush (apply_deferred after this system)
                commands.trigger_targets(StartOfTurnTriggered, entity);
            }
        }
    }
}
```
Schedule `apply_deferred` after this system in the same system set to guarantee handlers fire before next system reads their effects.

**on_start_of_turn handler:**
```rust
pub fn on_start_of_turn(
    trigger: Trigger<StartOfTurnTriggered>,
    units: Query<(&UnitKeywordState, &UnitBoardOwner)>,
    // Economy resources per card effect (e.g., GoldLedger for gold-generating cards)
) {
    let entity = trigger.target();
    let Ok((kw_state, owner)) = units.get(entity) else { return; };
    if !kw_state.has_keyword(SimpleKeyword::StartOfTurn) { return; } // mandatory guard
    // Apply START OF TURN effect (per card definition in CardCatalog)
}
```

**END OF TURN dispatch in resolve_combat (ADR-022 architecture diagram):**
```rust
// In resolve_combat, after SS6, before ResolutionComplete write:
for alive_unit in collect_alive_units(world) {
    world.trigger_targets(EndOfTurnTriggered, alive_unit);
}
// Then: keyword_triggered_writer.write(ResolutionComplete { ... });
```

**KW-009b — START OF TURN placement round exclusion:**
- RSM fires `DraftPhaseEntered` at the start of each DRAFT phase
- A unit placed in SS1 of round R does NOT receive `StartOfTurnTriggered` on round R's DRAFT entry (it wasn't on board when round R started)
- The unit IS on board at round R+1's DRAFT entry → START OF TURN fires

**KW-010b — END OF TURN same-round eligibility:**
- `collect_alive_units(world)` at end of SS6 includes units that entered play in SS1 of this round
- A unit placed in SS1 and surviving SS6 IS in the alive set → END OF TURN fires

---

## Out of Scope

- RSM `DraftPhaseEntered` emission (owned by round-state-machine epic — coordinate timing)
- Card-specific START/END OF TURN effects (card data authoring; this story implements dispatch machinery only)

---

## QA Test Cases

- **AC-1**: KW-009a — START OF TURN fires at DRAFT entry (existing unit)
  - Given: unit with START OF TURN was placed in round R-1; alive at round R DRAFT entry
  - When: RSM emits `DraftPhaseEntered` for round R (after mana ramp + gold income)
  - Then: `StartOfTurnTriggered` dispatched to unit; `on_start_of_turn` fires; effect applied
  - Edge cases: RSM Rule 3 (mana ramp + gold) must run BEFORE `DraftPhaseEntered` is emitted

- **AC-2**: KW-009b — START OF TURN does NOT fire round of placement
  - Given: unit placed in SS1 of round R; has START OF TURN keyword
  - When: round R's DRAFT phase begins
  - Then: unit NOT in `start_of_turn_dispatch_system` dispatch list (it wasn't on board at round R DRAFT entry); `StartOfTurnTriggered` NOT dispatched for this unit in round R
  - Edge cases: if unit survives to round R+1 DRAFT: START OF TURN fires then

- **AC-3**: KW-010a — END OF TURN fires after SS6, before round counter increments
  - Given: unit with END OF TURN alive after SS6; round counter = R
  - When: resolve_combat END OF TURN dispatch runs (after SS6, before ResolutionComplete)
  - Then: `EndOfTurnTriggered` dispatched; `on_end_of_turn` fires; effect applied; round counter still = R at time of effect
  - Edge cases: round counter increments AFTER ResolutionComplete is written (not during END OF TURN)

- **AC-4**: KW-010b — END OF TURN fires for unit that entered play this round
  - Given: unit placed in SS1 of round R (HASTE — survived SS5/SS6); has END OF TURN keyword
  - When: resolve_combat END OF TURN dispatch runs after SS6 of round R
  - Then: unit in `collect_alive_units` list; `EndOfTurnTriggered` dispatched and fires
  - Edge cases: unit that died in SS4 or SS6 is NOT in alive set; END OF TURN does not fire

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/keyword/start_end_of_turn_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 (scaffold — observers.rs stubs exist), Story 003 (HASTE — for KW-010b test with HASTE unit)
- Depends on: ADR-022 Verification Required item 5 resolved (`commands.trigger_targets()` in Bevy 0.18)
- Depends on: round-state-machine epic (`DraftPhaseEntered` message registered and emitted after RSM Rule 3)
- Unlocks: None directly
