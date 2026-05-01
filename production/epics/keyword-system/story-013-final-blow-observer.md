# Story 013: FINAL BLOW Observer (on_final_blow_dealt)

> **Epic**: Keyword System
> **Status**: Ready
> **Layer**: Feature (M3)
> **Type**: Logic
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/keyword-system.md`
**Requirement**: `TR-KW-???` — FINAL BLOW timing trigger has no registered TR-ID. Use placeholder. Run `/architecture-review` to register missing TRs before marking this story Done.
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-022 (Keyword System — Timing Trigger Observer Architecture, Part 1 & Part 2)
**ADR Decision Summary**: FINAL BLOW dispatched via `on_final_blow_dealt` global Observer fired on the ATTACKER entity (not the killed unit). Fires in the sub-step of the kill — SS3 for FIRST STRIKE kills, SS6 for standard kills. Not deferred to SS4. If two sequential damage sources in the same sub-step kill a unit, the second source (the one that reduced HP to 0) receives FINAL BLOW credit.

**Readiness Refresh (2026-05-01)**: Revalidated against control manifest version 2026-05-01. ADR-018 and ADR-022 are Accepted, ADR-022 verification is resolved in the current manifest, and Story 001 is Complete. The stale ADR-018 blocker is cleared.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**:
- `world.trigger_targets(FinalBlowDealt { killed, sub_step }, attacker)` — fired on the ATTACKER entity (not the killed unit); Observer fires synchronously within `resolve_combat` (ADR-017 exclusive system)
- `On<FinalBlowDealt>` — CONFIRMED correct observer param type (ADR-022 item 2 resolved; control manifest 2026-05-01)
- `MessageWriter<KeywordTriggered>` inside Observer — CONFIRMED valid (ADR-022 item 4 resolved)
- The killed unit (`FinalBlowDealt.killed`) is still present in `BoardState.units` at the point `FinalBlowDealt` fires — it is only removed in SS4. The observer can query the killed unit's state.

**Control Manifest Rules (Feature layer)**:
- Required: Guard check (`has_keyword(SimpleKeyword::FinalBlow)`) must be the FIRST operation in `on_final_blow_dealt` — global observers fire for ALL entities (ADR-022)
- Required: FINAL BLOW fires on the ATTACKER entity, not the killed unit — `world.trigger_targets(FinalBlowDealt { .. }, attacker)` (ADR-022)
- Forbidden: Never defer FINAL BLOW to SS4 — it fires in the sub-step of the kill (SS3 or SS6)
- Forbidden: Never fire FINAL BLOW on an attacker that did not deal the killing blow — only the source that reduced HP to 0 gets FINAL BLOW credit

---

## Acceptance Criteria

*From GDD `design/gdd/keyword-system.md` Acceptance Criteria, Timing Triggers section:*

- [ ] KW-004a: GIVEN a unit with FINAL BLOW kills an enemy in SS3 (via FIRST STRIKE), WHEN SS3 resolves, THEN FINAL BLOW fires on the attacker in SS3, before SS4 begins; the killed unit is still present in `BoardState.units` at the point FINAL BLOW fires
- [ ] KW-004b: GIVEN a unit with FINAL BLOW kills an enemy in SS6 (standard combat), WHEN SS6 resolves, THEN FINAL BLOW fires on the attacker in SS6, before RESOLUTION ends
- [ ] KW-035a: GIVEN unit A (FIRST STRIKE, FINAL BLOW) kills unit B in SS3, WHEN SS3 resolves, THEN FINAL BLOW's effect is reflected in the server's authoritative per-player resource (e.g., gold delta) at SS3 completion, before SS4 begins; unit B is still present in `BoardSnapshot.units` at SS3 completion (removal deferred to SS4)

---

## Implementation Notes

*Derived from ADR-022 Part 2 (event types) and GDD FINAL BLOW rules:*

**FinalBlowDealt event:**
```rust
// Fired on the ATTACKER entity (not the killed unit)
// sub_step = 3 for FIRST STRIKE kills, 6 for standard kills
pub struct FinalBlowDealt {
    pub killed: Entity,
    pub sub_step: u8,
}
```

**Call site in resolve_combat (SS3 and SS6):**
```rust
// After applying damage that reduces target HP to 0:
if target_hp_after_damage == 0 {
    world.trigger_targets(FinalBlowDealt { killed: target, sub_step }, attacker);
    // Note: target entity is still on board at this point — removal happens in SS4
}
```

**on_final_blow_dealt handler:**
```rust
pub fn on_final_blow_dealt(
    trigger: On<FinalBlowDealt>,
    units: Query<&UnitKeywordState>,
    mut keyword_triggered: MessageWriter<KeywordTriggered>,
) {
    let entity = trigger.target();  // the ATTACKER
    let Ok(kw_state) = units.get(entity) else { return; };
    if !kw_state.has_keyword(SimpleKeyword::FinalBlow) { return; }  // guard FIRST
    // Apply FINAL BLOW effect from card definition
    // Effect depends on card text — apply via CardCatalog lookup on trigger.target()
}
```

**KW-035a assertion note (from GDD):** The AC says "assert on the server's authoritative gold resource after SS3 system runs and before SS4 system runs — the gold delta from FINAL BLOW must already be recorded. Do NOT assert on event-emission ordering." This means the test must check that the gold `Resource` has been mutated, not just that a message was emitted.

**Two sequential damage sources:** If attacker A deals partial damage and attacker B's subsequent hit reduces HP to 0 in the same sub-step, FINAL BLOW fires on B (the killing blow source). Implementation: track which entity's `apply_damage` call resulted in HP ≤ 0 and fire on that attacker.

---

## Out of Scope

- Story 012: DEATH trigger (fires in SS4, after FINAL BLOW in SS3/SS6) — separate story
- Story 003: FIRST STRIKE attack mechanics — FIRST STRIKE attacks in SS3; FINAL BLOW fires if the SS3 attack kills
- Story 009: OUTNUMBERED re-evaluation — happens at sub-step boundaries, not per kill

---

## QA Test Cases

*Automated test specs (Logic story):*

- **KW-004a**: FINAL BLOW fires in SS3 (FIRST STRIKE kill)
  - Given: Unit A (FIRST STRIKE, FINAL BLOW effect: +1 gold); unit B at same cell, HP=3, A ATK=5
  - When: SS3 resolves (FIRST STRIKE pass)
  - Then: `on_final_blow_dealt` fires on A with `sub_step=3`; unit B still present in board state during observer execution; A's gold increments by +1 (FINAL BLOW effect applied in SS3)
  - Edge cases: if FIRST STRIKE does NOT kill (target survives SS3), FINAL BLOW does not fire in SS3

- **KW-004b**: FINAL BLOW fires in SS6 (standard kill)
  - Given: Unit A (FINAL BLOW effect: +2 gold); unit B HP=2, A ATK=5; standard combat in SS6
  - When: SS6 resolves
  - Then: FINAL BLOW fires on A with `sub_step=6`; gold effect applied; B removed in SS4 next resolution (not relevant to this test)

- **KW-035a**: FINAL BLOW gold delta before SS4
  - Given: Unit A (FIRST STRIKE, FINAL BLOW effect: grants +1 reserve_mana); unit B killed in SS3; unit B has DEATH trigger
  - When: SS3 completes, before SS4 begins
  - Then: A's reserve_mana has incremented by +1 (FINAL BLOW effect already applied); unit B is still in `BoardState.units` at SS3 end; B is removed only when SS4 runs; B's DEATH trigger fires in SS4 (Story 012)
  - Assertion: query `Res<PlayerEconomies>` after SS3 system, before SS4 system — reserve delta must be +1

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/keyword/final_blow_observer_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 (scaffold — registers `on_final_blow_dealt` observer stub) must be Done
- Unlocks: None specifically; Story 012 (DEATH chain) may execute after FINAL BLOW in SS3/SS6 — they are independent stories but share the same RESOLUTION pass
