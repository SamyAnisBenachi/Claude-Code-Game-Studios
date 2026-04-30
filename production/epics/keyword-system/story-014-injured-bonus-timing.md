# Story 014: Timing Trigger — INJURED Bonus Activation

> **Epic**: Keyword System
> **Status**: Blocked
> **Layer**: Feature (M3)
> **Type**: Logic
> **Manifest Version**: 2026-04-30

## Context

**GDD**: `design/gdd/keyword-system.md`
**Requirement**: TR-KW-005 — INJURED re-evaluated at sub-step boundaries; not retroactive within sub-step
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-022 (Timing Trigger Observer Architecture, Part 5 — INJURED inline dispatch)
**ADR Decision Summary**: INJURED is re-evaluated by `eval_injured_bonuses()` called inline by `resolve_combat` at SS3→SS4, SS5, and SS6 sub-step boundaries. Not per-attack — scan-based at boundaries. INJURED state is derived (`current_hp < max_hp`) not stored. The 4 INJURED-grantable keywords: FIRST STRIKE, COUNTERATTACK, RANGE, SHIELD (closed list — `InjuredGrantedKeyword` enum).

**BLOCKED**: ADR-022 Proposed. ADR-018 Proposed. Story 001 and Story 006 (SILENCE+INJURED state system) must be Done.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**: `eval_injured_bonuses()` called inline from `resolve_combat` — no Bevy Observer; plain function call with `world: &mut World`. Must access both `UnitStats.hp` (live, replicated) and a cached `max_hp` value (from server-side component — not in `UnitStats`).

**Control Manifest Rules (Feature layer)**:
- Required: INJURED is a derived state — never store as boolean flag (ADR-018)
- Forbidden: Never grant INJURED bonuses retroactively within a sub-step — boundary-only evaluation (ADR-022)

---

## Acceptance Criteria

*From GDD `design/gdd/keyword-system.md` Acceptance Criteria:*

- [ ] KW-007: GIVEN unit X has max_HP=4, current_HP=4, and gains FIRST STRIKE when INJURED; X receives 2 damage in SS3 (HP→2), WHEN SS3 resolves, THEN INJURED bonus NOT active during SS3 (was not INJURED at SS3 start); INJURED activates at the SS3→SS4 boundary; bonus granted from SS4 onward in same RESOLUTION
- [ ] KW-057: GIVEN unit X gains SHIELD via INJURED at the SS3→SS4 boundary (damaged in SS3), WHEN SS6 attacker A deals damage to X, THEN SHIELD (granted at SS3→SS4 boundary) absorbs the SS6 attack; it is NOT retroactive to SS3
- [ ] `eval_injured_bonuses()` called at SS3→SS4, SS5, and SS6 sub-step boundaries — NOT inline during SS3 damage computation
- [ ] The 4 INJURED-grantable keywords are: FIRST STRIKE, COUNTERATTACK, RANGE, SHIELD — closed list matching `InjuredGrantedKeyword` enum (FirstStrike, Counterattack, Range, Shield)
- [ ] `KeywordTriggered { source_unit_id: Some(unit_id), sub_step, payload: InjuredBonusActive { granted_keyword: GrantedKeyword } }` emitted when INJURED bonus becomes active
- [ ] SILENCE strips INJURED-granted keyword bonuses (KW-008a — see Story 006): if unit is silenced, INJURED grants no bonuses during that RESOLUTION

---

## Implementation Notes

*Derived from ADR-022 Part 5 and GDD INJURED rules:*

**eval_injured_bonuses signature:**
```rust
// server/feature/keyword/state_eval.rs
// Called inline by resolve_combat at sub-step boundaries
pub fn eval_injured_bonuses(world: &mut World)
```

**Algorithm:**
1. Query all alive board units
2. For each unit: compute `injured = unit.current_hp < unit.max_hp`
3. If injured AND not silenced:
   - Read card definition for INJURED-bonus keyword declarations (from `CardCatalog`)
   - For each INJURED-bonus keyword declared: set the granted flag in `UnitKeywordState`
   - Emit `InjuredBonusActive { granted_keyword }` if newly activated (state transition)
4. If NOT injured OR silenced: clear all INJURED-granted keyword flags

**max_hp access:** `max_hp` is NOT in `UnitStats` (replicated component). It must be cached in a server-only component (e.g., `UnitMaxHp { value: u8 }`) or derived from `CardCatalog` on the unit's `CardId`. Coordinate with combat-resolution epic's unit spawning logic to ensure `max_hp` is accessible server-side.

**KW-057 — INJURED-granted SHIELD timing:**
- `eval_injured_bonuses()` called at SS3→SS4 boundary (AFTER SS3 damage applied)
- If X was damaged in SS3 and is now INJURED: SHIELD flag set at SS3→SS4 boundary
- SHIELD becomes available from SS4 onward (including SS6)
- NOT retroactive to SS3 (damage already applied; SHIELD wasn't active during SS3)

**SILENCE interaction (KW-008a, Story 006):**
```rust
if kw_state.silenced_until_round.is_some_and(|r| current_round <= r) {
    // Clear all INJURED-granted flags; do not emit bonuses
    clear_injured_bonuses(&mut kw_state);
    return;
}
```

**InjuredGrantedKeyword enum (closed list):**
```rust
pub enum InjuredGrantedKeyword { FirstStrike, Counterattack, Range, Shield }
```
Adding a new INJURED-grantable keyword requires updating BOTH this GDD and the NP GDD's enum.

---

## Out of Scope

- Story 006: SILENCE system (provides `silenced_until_round` check used here)
- Story 011: INJURED-granted RANGE attack execution (KW-056 — this story grants the RANGE flag; Story 011 uses it)
- Story 013: INJURED-granted COUNTERATTACK firing (KW-055 — this story grants the flag; Story 013 checks it)

---

## QA Test Cases

- **AC-1**: KW-007 — INJURED bonus not retroactive within SS3
  - Given: unit X (max_HP=4, HP=4) has FIRST STRIKE when INJURED; receives 2 damage in SS3 (HP→2)
  - When: SS3 resolves (damage applied); eval_injured_bonuses called at SS3→SS4 boundary
  - Then: X does NOT use FIRST STRIKE in SS3 (was not INJURED at SS3 start); INJURED bonus activated at SS3→SS4 boundary (InjuredBonusActive emitted); X has FIRST STRIKE available from SS4 onward
  - Edge cases: eval_injured_bonuses must NOT be called before SS3 damage is applied

- **AC-2**: KW-057 — INJURED-granted SHIELD available from SS3→SS4 boundary
  - Given: unit X (max_HP=6, HP=6) has SHIELD when INJURED; receives 3 damage in SS3 (HP→3)
  - When: eval_injured_bonuses called at SS3→SS4 boundary; SS6 attacker A deals 2 damage
  - Then: SHIELD granted at SS3→SS4 boundary; SS6 attack absorbed by SHIELD; X HP stays at 3; SHIELD consumed in SS6
  - Edge cases: SHIELD NOT available retroactively in SS3 (damage already applied at SS3)

- **AC-3**: eval_injured_bonuses called at correct sub-step boundaries
  - Given: any RESOLUTION with units taking damage in SS3
  - When: resolve_combat executes
  - Then: eval_injured_bonuses called at SS3→SS4 boundary, at SS5 boundary, and at SS6 boundary — total 3 calls per RESOLUTION (can verify via call count in test)
  - Edge cases: NOT called inline during SS3 damage computation (timing matters for KW-007)

- **AC-4**: SILENCE prevents INJURED bonus grant
  - Given: unit X is INJURED and silenced (`silenced_until_round = Some(current_round)`)
  - When: eval_injured_bonuses called
  - Then: no INJURED-granted keywords activated; InjuredBonusActive NOT emitted; all prior INJURED grants cleared
  - Edge cases: INJURED state still true (HP comparison unchanged); only bonus grants suppressed

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/keyword/injured_bonus_timing_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 (scaffold), Story 006 (SILENCE system — silenced_until_round check)
- Unlocks: Story 011 (INJURED-granted RANGE — KW-056), Story 013 (INJURED-granted COUNTERATTACK — KW-055)
