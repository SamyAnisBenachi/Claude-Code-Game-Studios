# Story 013: Timing Trigger — FINAL BLOW + COUNTERATTACK Dispatch

> **Epic**: Keyword System
> **Status**: Blocked
> **Layer**: Feature (M3)
> **Type**: Logic
> **Manifest Version**: 2026-04-30

## Context

**GDD**: `design/gdd/keyword-system.md`
**Requirement**: TR-KW-004 (COUNTERATTACK proximity-gated inline dispatch); TR-KW-??? (FINAL BLOW — untraced)
*(Run `/architecture-review` to register FINAL BLOW TR)*

**ADR Governing Implementation**: ADR-022 (Timing Trigger Observer Architecture, Parts 4–5)
**ADR Decision Summary**: FINAL BLOW uses global Observer (`on_final_blow_dealt`) fired on the ATTACKER entity at the killing blow in SS3 or SS6. COUNTERATTACK uses inline dispatch (`check_and_apply_counterattack`) — proximity precondition must be evaluated against live board state before dispatch; inline conditional call is simpler than Observer wrapping.

**BLOCKED**: ADR-022 Proposed. ADR-018 Proposed. Verification Required items 3–4 from ADR-022 must be resolved (`ResMut<T>` and `MessageWriter<T>` usable inside Observer handler from exclusive system). Story 001, Story 003, and Story 005 must be Done.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**:
- `world.trigger_targets(FinalBlowDealt { killed, sub_step }, attacker)` — confirm `Trigger<T>` param and `ResMut<T>`/`MessageWriter<T>` usable inside Observer (Verification Required items 2–4 from ADR-022)
- `world.get::<UnitKeywordState>(entity)` returning `Option<&C>` — confirmed stable exclusive-system API (ADR-022 Part 5)

**Control Manifest Rules (Feature layer)**:
- Required: COUNTERATTACK proximity check gates all dispatch — `check_counterattack_proximity()` must run before any COUNTERATTACK effect (ADR-022)
- Forbidden: Never fire COUNTERATTACK for RANGE attackers — RANGE attackers do not advance to target's cell; proximity condition not met

---

## Acceptance Criteria

*From GDD `design/gdd/keyword-system.md` Acceptance Criteria:*

- [ ] KW-004a: GIVEN a unit with FINAL BLOW is killed by a FIRST STRIKE attacker in SS3, WHEN the killing blow reduces HP to 0 in SS3, THEN FINAL BLOW fires in SS3 and NOT in SS4
- [ ] KW-004b: GIVEN a unit with FINAL BLOW is killed by standard combat in SS6, WHEN the killing blow reduces HP to 0 in SS6, THEN FINAL BLOW fires in SS6 and NOT in SS4
- [ ] KW-005: GIVEN a COUNTERATTACK unit receives damage from a RANGE attacker that did not advance to the COUNTERATTACK unit's cell, WHEN the RANGE attack resolves, THEN COUNTERATTACK does NOT fire
- [ ] KW-006: GIVEN a COUNTERATTACK unit is halted at an adjacent cell from a sub-step 5 collision, WHEN the two units exchange melee damage in SS6, THEN COUNTERATTACK fires
- [ ] KW-035a: GIVEN unit A (FIRST STRIKE) kills unit B (has DEATH trigger and FINAL BLOW) in SS3, WHEN SS3 resolves, THEN FINAL BLOW's effect is reflected in the server's `GoldLedger` (or equivalent gold resource) at SS3 completion, before SS4 begins; unit B is still present in board state at SS3 completion
- [ ] KW-048: GIVEN unit A (ATK=5, COUNTERATTACK) and unit B (ATK=3, COUNTERATTACK) fight in SS6, WHEN B's COUNTERATTACK fires against A, THEN A's COUNTERATTACK does NOT fire a second time; chain terminates after one COUNTERATTACK each
- [ ] KW-049: GIVEN unit X (ATK=4, COUNTERATTACK) is attacked simultaneously in SS6 by attacker A (ATK=3) and attacker B (ATK=2), WHEN SS6 resolves, THEN X retaliates against A for 4 damage AND against B for 4 damage; pre-retaliation HP snapshot used for both
- [ ] KW-050: GIVEN unit X has COUNTERATTACK and active SHIELD; attacker A deals damage to X in SS6, WHEN SHIELD absorbs all incoming damage, THEN X's COUNTERATTACK still fires against A; X's SHIELD is consumed
- [ ] KW-055: GIVEN unit X has COUNTERATTACK granted via INJURED (X.ATK=3, INJURED=true), attacker A deals 1 damage in SS6, WHEN SS6 resolves, THEN X's COUNTERATTACK fires for 3 damage against A (using pre-retaliation snapshot)

---

## Implementation Notes

*Derived from ADR-022 Parts 4–5 and GDD COUNTERATTACK + FINAL BLOW rules:*

**on_final_blow_dealt handler (ADR-022 Part 4):**
```rust
pub fn on_final_blow_dealt(
    trigger: Trigger<FinalBlowDealt>,   // fired on ATTACKER entity
    units: Query<&UnitKeywordState>,
    mut keyword_triggered: MessageWriter<KeywordTriggered>,
) {
    let attacker = trigger.target();
    let Ok(kw_state) = units.get(attacker) else { return; };
    if !kw_state.has_keyword(SimpleKeyword::FinalBlow) { return; } // mandatory guard
    // Apply FINAL BLOW effect (per card — e.g., gold award, stat buff)
    // KW-035a: gold award must be recorded in GoldLedger BEFORE SS4 begins
}
```
Firing point: `world.trigger_targets(FinalBlowDealt { killed, sub_step }, attacker)` called AT the killing blow in SS3 or SS6 — NOT deferred to SS4.

**check_and_apply_counterattack (ADR-022 Part 5):**
```rust
fn check_and_apply_counterattack(
    world: &mut World,
    defender: Entity,
    attacker: Entity,
    sub_step: u8,
) {
    let Ok(kw_state) = world.get::<UnitKeywordState>(defender) else { return; };
    // Guard: check keyword presence (includes INJURED-granted COUNTERATTACK)
    if !kw_state.has_counterattack_active() { return; }
    // Proximity check: same-cell OR collision-halted adjacent contact; RANGE excluded
    if !keyword::effects::check_counterattack_proximity(world, defender, attacker) { return; }
    keyword::effects::apply_counterattack(world, defender, attacker, sub_step);
}
```

**COUNTERATTACK chain termination (KW-048):**
- A attacks B → B fires COUNTERATTACK against A → A checks COUNTERATTACK: if A has it, A retaliates ONCE → chain ends
- Track `counterattack_chain_depth` (u8) per COUNTERATTACK call; if depth >= 1, skip further chain

**Multi-attacker pre-retaliation snapshot (KW-049):**
- If multiple attackers hit X in same sub-step, snapshot X's ATK BEFORE applying any retaliation damage
- COUNTERATTACK fires against ALL attackers using the same snapshot ATK value

**COUNTERATTACK fires even when SHIELD absorbed (KW-050):**
- `check_shield_absorb()` called BEFORE COUNTERATTACK check
- Even if SHIELD absorbed (return early from damage), COUNTERATTACK check must still run: `was_attacked = true` regardless of damage absorbed

**INJURED-granted COUNTERATTACK (KW-055):**
- `has_counterattack_active()` checks both `has_keyword(SimpleKeyword::Counterattack)` AND `kw_state.injured_granted_counterattack` (set by `eval_injured_bonuses()` in Story 014)

---

## Out of Scope

- Story 012: DEATH chain that fires AFTER FINAL BLOW in SS4 (KW-035b)
- Story 014: INJURED bonus activation granting COUNTERATTACK (KW-055 depends on Story 014)
- Story 005: SHIELD mechanics (KW-050 depends on Story 005)

---

## QA Test Cases

- **AC-1**: KW-004a — FINAL BLOW fires in SS3 (FIRST STRIKE kill)
  - Given: FINAL BLOW unit B (HP=2); FIRST STRIKE attacker A (ATK=2)
  - When: SS3 damage reduces B's HP to 0
  - Then: `on_final_blow_dealt` fires on A in SS3; FINAL BLOW effect applied at SS3; B still in board state at SS3 completion
  - Edge cases: assert FINAL BLOW effect recorded before SS4 starts

- **AC-2**: KW-005 — COUNTERATTACK does NOT fire for RANGE attacker
  - Given: unit X has COUNTERATTACK; RANGE attacker Y (did not advance to X's cell)
  - When: RANGE attack resolves
  - Then: `check_counterattack_proximity(X, Y)` returns false; COUNTERATTACK does NOT fire
  - Edge cases: Y at Cell 4, X at Cell 6, RANGE=3 — Y never advanced; proximity fails

- **AC-3**: KW-006 — COUNTERATTACK fires for collision-halted adjacent
  - Given: unit X (COUNTERATTACK) at Cell 5; enemy Y halted at Cell 4 from SS5 collision (adjacent)
  - When: SS6 melee exchange resolves (Y attacks X from Cell 4)
  - Then: `check_counterattack_proximity(X, Y)` returns true (adjacent, collision-halted); COUNTERATTACK fires
  - Edge cases: Y halted by WALL, not collision — confirm collision-halt adjacency is tracked

- **AC-4**: KW-035a — FINAL BLOW gold award before SS4
  - Given: attacker A (FINAL BLOW) kills B in SS3 via FIRST STRIKE
  - When: SS3 resolves
  - Then: GoldLedger has FINAL BLOW gold delta recorded; B still in board entity list at SS3 end
  - Edge cases: assert GoldLedger delta BEFORE SS4 drain loop starts

- **AC-5**: KW-048 — COUNTERATTACK chain terminates at one retaliation each
  - Given: A (COUNTERATTACK, ATK=5) attacks B (COUNTERATTACK, ATK=3)
  - When: SS6 resolves
  - Then: B fires COUNTERATTACK against A; A fires COUNTERATTACK back ONCE; no further retaliation
  - Edge cases: chain depth counter prevents infinite loop

- **AC-6**: KW-049 — Multi-attacker COUNTERATTACK uses pre-retaliation snapshot
  - Given: X (COUNTERATTACK, ATK=4); A (ATK=3) and B (ATK=2) attack X simultaneously
  - When: SS6 resolves
  - Then: X retaliates 4 dmg against A AND 4 dmg against B; snapshot of ATK=4 taken before any retaliation delta
  - Edge cases: if X's HP drops during retaliation A, X's ATK snapshot for retaliation B is unchanged (snapshot-before-retaliation)

- **AC-7**: KW-050 — COUNTERATTACK fires when SHIELD absorbed damage
  - Given: X (COUNTERATTACK, SHIELD active); attacker A deals 3 damage in SS6
  - When: SHIELD absorbs (X takes 0 damage)
  - Then: COUNTERATTACK fires against A (X was attacked even though shielded); SHIELD consumed
  - Edge cases: SHIELD absorption must not suppress COUNTERATTACK check

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/keyword/final_blow_counterattack_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 (scaffold), Story 003 (FIRST STRIKE — for KW-035a), Story 005 (SHIELD — for KW-050), Story 012 (DEATH chain — for sequencing context), Story 014 (INJURED bonus — for KW-055)
- Depends on: ADR-022 Verification Required items 2–4 resolved
- Unlocks: No stories directly (all remaining stories are independent)
