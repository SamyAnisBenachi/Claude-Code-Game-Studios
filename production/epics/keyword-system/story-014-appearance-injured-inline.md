# Story 014: APPEARANCE Observer + INJURED Inline Dispatch (eval_injured_bonuses)

> **Epic**: Keyword System
> **Status**: Blocked
> **Layer**: Feature (M3)
> **Type**: Logic
> **Manifest Version**: 2026-04-30

## Context

**GDD**: `design/gdd/keyword-system.md`
**Requirement**: `TR-KW-005` — INJURED re-evaluated at sub-step boundaries, not retroactive within sub-step. `TR-KW-???` — APPEARANCE timing trigger has no registered TR-ID; run `/architecture-review` to register before marking Done.
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-022 (Keyword System — Timing Trigger Observer Architecture, Part 1 & Part 2 for APPEARANCE; Part 5 for INJURED inline dispatch)
**ADR Decision Summary**: APPEARANCE dispatched via `on_unit_appeared` global Observer, fired per entity after board commit from `PlacementBuffer` in SS1. INJURED is NOT an observer — it is a state re-evaluation called inline at sub-step boundaries by `eval_injured_bonuses()` in `state_eval.rs`. `eval_injured_bonuses()` scans all board units and updates any INJURED-granted keyword bonuses.

**BLOCKED**: ADR-018 is Proposed. ADR-022 is Accepted. Story 001 (scaffold — registers `on_unit_appeared` observer stub and stubs `eval_injured_bonuses`) must be Done.

> ⚠️ **Story 006 (SILENCE + INJURED) depends on this story.** Story 006 tests how SILENCE strips INJURED-granted keyword bonuses; those bonuses are granted by `eval_injured_bonuses` implemented here. Story 006 must NOT be opened until this story is Done.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**:
- `world.trigger_targets(UnitAppeared { sub_step: 1 }, entity)` — CONFIRMED valid `World` method in Bevy 0.18 (ADR-022 item 1 resolved); fires synchronously within `resolve_combat`
- `Trigger<UnitAppeared>` — CONFIRMED correct observer param type (ADR-022 item 2 resolved)
- `eval_injured_bonuses(world: &mut World)` — called inline at SS3→SS4, SS5, and SS6 sub-step boundaries by `resolve_combat`
- `max_hp` server-side: INJURED derivation requires `current_hp < max_hp`. `max_hp` must be accessible server-side as a component field (coordinate with combat-resolution.md — `UnitStats` component must include or cache `max_hp`)

**Control Manifest Rules (Feature layer)**:
- Required: Guard check (`has_keyword(SimpleKeyword::Appearance)`) must be the FIRST operation in `on_unit_appeared` — global observers fire for ALL entities (ADR-022)
- Required: `eval_injured_bonuses` called at SS3→SS4 boundary, SS5 boundary, and SS6 boundary — NOT inline during SS3 damage computation (KW-007: INJURED bonus not active during the sub-step damage was received)
- Forbidden: Never store INJURED as a boolean flag in `UnitKeywordState` — always derive from `current_hp < max_hp` at boundary (ADR-018)
- Forbidden: Never call `eval_injured_bonuses` inside the SS3 damage loop — only at boundaries

---

## Acceptance Criteria

*From GDD `design/gdd/keyword-system.md` Acceptance Criteria:*

- [ ] KW-001: GIVEN units with APPEARANCE keywords enter play in SS1 across multiple lanes, WHEN SS1 resolves, THEN all SS1 APPEARANCE effects across all lanes complete before any DEATH chains from APPEARANCE-caused deaths begin; the killed unit (if any) is still present during any FINAL BLOW that results from the APPEARANCE effect
- [ ] KW-007: GIVEN unit X has max_HP=4, current_HP=4, and gains FIRST STRIKE when INJURED; X receives 2 damage in SS3 (reducing HP to 2), WHEN SS3 resolves, THEN X does NOT receive the INJURED-granted FIRST STRIKE during SS3; the INJURED bonus activates at the SS3→SS4 boundary (evaluated by `eval_injured_bonuses`), granting FIRST STRIKE from SS4 onward in this RESOLUTION
- [ ] KW-069: GIVEN unit X receives lethal damage from an APPEARANCE trigger in SS1 (another unit's APPEARANCE effect deals damage to X, reducing HP below max_HP); unit X has a card-granted FIRST STRIKE when INJURED, WHEN the SS1→SS2 boundary arrives and `eval_injured_bonuses` runs, THEN the INJURED-granted FIRST STRIKE IS active for SS3 — INJURED was acquired at the SS1→SS2 boundary, which precedes SS3

---

## Implementation Notes

*Derived from ADR-022 Part 2 (APPEARANCE event) and Part 5 (INJURED inline):*

**UnitAppeared event and call site:**
```rust
// In resolve_combat SS1, after board::api::spawn_unit() commits each unit:
world.trigger_targets(UnitAppeared { sub_step: 1 }, entity);
// Fires on_unit_appeared for that entity synchronously
```

**on_unit_appeared handler:**
```rust
pub fn on_unit_appeared(
    trigger: Trigger<UnitAppeared>,
    units: Query<(&UnitKeywordState, &CardId)>,
    card_catalog: Res<CardCatalog>,
    mut keyword_triggered: MessageWriter<KeywordTriggered>,
) {
    let entity = trigger.target();
    let Ok((kw_state, card_id)) = units.get(entity) else { return; };
    if !kw_state.has_keyword(SimpleKeyword::Appearance) { return; }  // guard FIRST
    // Apply APPEARANCE effect from card definition via card_catalog.get(card_id)
    // May emit KeywordTriggered for client animation
}
```

**KW-001 ordering guarantee:** The APPEARANCE Observer fires synchronously within SS1's spawn loop. All spawned units' observers fire before `execute_ss4()` is called. This structural ordering satisfies KW-001 without explicit sequencing code.

**eval_injured_bonuses — call sites in resolve_combat:**
```rust
execute_ss3(world);
keyword::state_eval::eval_injured_bonuses(world);  // SS3→SS4 boundary

execute_ss4(world);  // DEATH chain (KW-007: SS4 entry has INJURED active if damaged in SS3)

// [SS5 execution includes standard movement]
keyword::state_eval::eval_injured_bonuses(world);  // SS5 boundary

execute_ss6(world);
keyword::state_eval::eval_injured_bonuses(world);  // SS6 boundary (for any END OF TURN cleanup)
```

**eval_injured_bonuses implementation sketch:**
```rust
// server/feature/keyword/state_eval.rs
pub fn eval_injured_bonuses(world: &mut World) {
    // For each board unit with a card that grants keywords when INJURED:
    //   injured = unit.current_hp < unit.max_hp
    //   if injured: apply INJURED-granted keyword bonuses to UnitKeywordState
    //   else: remove any previously-granted INJURED bonuses
    //   emit InjuredBonusActive KeywordTriggered when bonus activates
}
```

**INJURED + INJURED-granted SHIELD (KW-057, tested in Story 015):** SHIELD granted via INJURED at SS3→SS4 boundary is active from SS4 onward (including SS6). `eval_injured_bonuses` must write `shield_active = true` if the card grants SHIELD while INJURED.

**KW-069 ordering guarantee:** SS1→SS2 boundary calls `eval_injured_bonuses`. If a unit was damaged by an APPEARANCE effect in SS1 (HP dropped below max_HP), `eval_injured_bonuses` at the SS1→SS2 boundary activates the INJURED-granted bonus (e.g., FIRST STRIKE). The bonus is then active for SS3.

> Note: The SS1→SS2 boundary call for `eval_injured_bonuses` is not explicitly listed in ADR-022's call-site table (which lists SS3→SS4, SS5, SS6). The SS1→SS2 boundary call is required for KW-069 and must be added to `resolve_combat`'s SS1→SS2 transition.

---

## Out of Scope

- Story 006: SILENCE strips INJURED-granted keywords (depends on this story being Done)
- Story 015: COUNTERATTACK granted via INJURED (KW-055), RANGE via INJURED (KW-056), SHIELD via INJURED (KW-057) — cross-keyword tests using `eval_injured_bonuses`
- Story 012: DEATH chain triggered by APPEARANCE-caused lethal damage (handled in Story 012; ordering guaranteed by structural sequencing in SS1)

---

## QA Test Cases

*Automated test specs (Logic story):*

- **KW-001**: APPEARANCE fires before DEATH chains
  - Given: Unit A (APPEARANCE: deals 5 damage to unit B); unit B (DEATH trigger: grants +2 gold); unit B HP=3
  - When: SS1 resolves (both A and B spawned); A's APPEARANCE fires → B HP becomes -2 (dies)
  - Then: A's APPEARANCE effect completes in SS1; B is still present during A's APPEARANCE observer execution; B's DEATH trigger fires in SS4 (after all SS1 APPEARANCEs complete); gold from B's DEATH is awarded in SS4, not SS1
  - Edge cases: if A's APPEARANCE effect kills B and B has FINAL BLOW eligibility — FINAL BLOW fires on A in SS1 (since FINAL BLOW fires in the kill sub-step); B's DEATH fires in SS4

- **KW-007**: INJURED bonus NOT active during damage sub-step
  - Given: Unit X (max_HP=4, current_HP=4, card grants FIRST STRIKE when INJURED)
  - When: SS3 resolves — X takes 2 damage (HP drops to 2, below max_HP=4); FIRST STRIKE attack in SS3 is evaluated for X
  - Then: X does NOT use FIRST STRIKE in SS3 (INJURED bonus not yet active during SS3); `eval_injured_bonuses` runs at SS3→SS4 boundary → X now has FIRST STRIKE bonus active for SS4+
  - Edge cases: if SS5 heals X back to max_HP, `eval_injured_bonuses` at SS5 boundary clears the bonus

- **KW-069**: INJURED from APPEARANCE → bonus active in SS3
  - Given: Unit X (max_HP=4, current_HP=4, card grants FIRST STRIKE when INJURED); unit Y (APPEARANCE: deals 2 damage to X in SS1)
  - When: SS1 resolves — Y's APPEARANCE fires → X HP drops to 2; `eval_injured_bonuses` runs at SS1→SS2 boundary
  - Then: X has FIRST STRIKE bonus active from SS2 onward (SS3 included); X attacks in SS3 via INJURED-granted FIRST STRIKE
  - Edge cases: if X also has STUN (applied in SS1), STUN overrides HASTE; FIRST STRIKE would still be granted but STUN suppresses SS3 action

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/keyword/appearance_injured_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 (scaffold — registers `on_unit_appeared` stub and `eval_injured_bonuses` stub) must be Done
- Unlocks: Story 006 (SILENCE + INJURED — depends on `eval_injured_bonuses` implemented here); Story 015 (COUNTERATTACK/SHIELD/RANGE via INJURED)
