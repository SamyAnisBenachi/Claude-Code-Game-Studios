# Story 005: SHIELD Sub-Step Scope

> **Epic**: Keyword System
> **Status**: Complete
> **Layer**: Feature (M3)
> **Type**: Logic
> **Manifest Version**: 2026-04-30

## Context

**GDD**: `design/gdd/keyword-system.md`
**Requirement**: TR-KW-008 — SHIELD persists across rounds until consumed; absorbs all sub-step damage once for simultaneous attackers
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-018 (Keyword System — ECS State Architecture)
**ADR Decision Summary**: `shield_active: bool` in `UnitKeywordState`. `check_shield_absorb()` in `effects.rs` returns true (and sets `shield_active = false`) when called in a sub-step where a shield is active. Sub-step scoped: consumed at most once per SS3 or SS6 in any RESOLUTION. Persists across rounds until triggered.

**BLOCKED**: ADR-018 Proposed. Story 001 must be Done.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**: `shield_active` must be read and written within the same `resolve_combat` exclusive-system call to prevent frame-split timing issues. No deferred commands.

**Control Manifest Rules (Feature layer)**:
- Required: SHIELD absorption must be a pre-check before damage application — not a post-damage refund (ADR-018 Note: "SHIELD canonical pre-check rule — removed from modifier stack step 10")
- Forbidden: Never apply SHIELD absorption as a modifier stack step — it is a pre-check gate

---

## Acceptance Criteria

*From GDD `design/gdd/keyword-system.md` Acceptance Criteria:*

- [ ] KW-024: GIVEN a unit with SHIELD is attacked by a RANGE+FIRST STRIKE attacker in SS3, then by two enemy melee units simultaneously in SS6, WHEN RESOLUTION executes, THEN SHIELD absorbs the SS3 attack (consumed in SS3); in SS6, SHIELD is no longer active and both melee attackers deal full damage
- [ ] KW-037: GIVEN unit X has SHIELD, and a RANGE+FIRST STRIKE attacker hits X in SS3 (consuming SHIELD), WHEN SS6 executes the RANGE unit's second attack, THEN X's HP after SS6 = HP-after-SS3 minus SS6 net damage (SHIELD gone in SS6)
- [ ] SHIELD absorbs ALL incoming damage from one sub-step — two simultaneous attackers in the same sub-step are BOTH absorbed by one SHIELD consumption (unit takes 0 damage; shield consumed once)
- [ ] SHIELD persists across rounds until triggered — a unit with SHIELD that is not attacked retains its SHIELD into the next round
- [ ] SHIELD consumed in SS3 does NOT protect in SS6 of the same RESOLUTION (sub-step scoped)
- [ ] SHIELD consumed in SS6 does NOT protect in SS3 of the NEXT round (consumed once, gone until re-acquired)
- [ ] `KeywordTriggered { payload: ShieldConsumed, sub_step }` is emitted when SHIELD is consumed

---

## Implementation Notes

*Derived from ADR-018 Key Interfaces and GDD Edge Cases:*

**check_shield_absorb signature:**
```rust
// server/feature/keyword/effects.rs
pub fn check_shield_absorb(kw_state: &mut UnitKeywordState, sub_step: SubStep) -> bool
```
Returns `true` if shield was active and absorbed the attack (sets `shield_active = false`). Returns `false` if no shield.

**Pre-check pattern (MUST be called before damage computation for the sub-step, not as a modifier stack step):**
```rust
// In execute_ss3() and execute_ss6():
if keyword::effects::check_shield_absorb(&mut kw_state, sub_step) {
    // Emit ShieldConsumed event; skip ALL damage computation for this unit this sub-step
    emit_shield_consumed_event(defender, sub_step, world);
    continue; // or return DamageResult::Absorbed
}
// ...proceed with normal damage computation...
```

**Simultaneous attackers:** when multiple melee attackers hit the unit in the same sub-step, call `check_shield_absorb` once before processing ANY of them. If it returns true, ALL attackers deal 0 damage — SHIELD absorbed the entire sub-step.

**Sub-step scope:** `shield_active` is consumed within a sub-step but persists unchanged between rounds. A unit with `shield_active = true` at the end of RESOLUTION carries it into the next round's SS1.

**COUNTERATTACK + SHIELD (KW-050):** SHIELD absorbs incoming damage, but COUNTERATTACK still fires — "the unit was attacked regardless" of whether damage was absorbed. This is tested in Story 013 (COUNTERATTACK); SHIELD absorption pre-check must be designed to not suppress COUNTERATTACK dispatch.

---

## Out of Scope

- Story 013: COUNTERATTACK fires even when SHIELD absorbed (KW-050) — COUNTERATTACK dispatch logic
- Story 011: RANGE+FIRST STRIKE attacks in SS3 AND SS6 (KW-017 setup) — RANGE logic itself

---

## QA Test Cases

- **AC-1**: KW-024 — SHIELD consumed in SS3 doesn't protect in SS6
  - Given: unit X has SHIELD (shield_active=true); RANGE+FIRST STRIKE attacker hits X in SS3
  - When: SS3 resolves
  - Then: SHIELD absorbs SS3 attack (X HP unchanged; shield_active→false; ShieldConsumed event emitted at sub_step=3)
  - When: two melee units attack X simultaneously in SS6
  - Then: both melee attackers deal full damage to X; no SHIELD protection (shield_active=false)
  - Edge cases: verify shield_active is false at SS6 start when consumed in SS3

- **AC-2**: KW-037 — RANGE+FIRST STRIKE second attack (SS6) hits unshielded unit
  - Given: unit X (HP=6) has SHIELD; RANGE+FIRST STRIKE attacker (ATK=2, net_damage=2)
  - When: SS3 RANGE attack hits X → SHIELD absorbs (X HP stays 6, shield_active→false); SS6 RANGE second attack hits X
  - Then: X HP after SS6 = 6 - 2 = 4 (full damage, no SHIELD)
  - Edge cases: assert shield_active=false before SS6 damage computation

- **AC-3**: Simultaneous attackers absorbed by one SHIELD consumption
  - Given: unit X has SHIELD; attacker A (ATK=3) and attacker B (ATK=2) both attack in SS6
  - When: SS6 resolves
  - Then: X takes 0 damage from A AND 0 damage from B; shield_active→false; ShieldConsumed emitted once
  - Edge cases: both attackers processed in one SHIELD check, not per-attacker

- **AC-4**: SHIELD persists across rounds if not consumed
  - Given: unit X has SHIELD; no enemy attacks X in round R
  - When: RESOLUTION R ends; RESOLUTION R+1 begins
  - Then: shield_active=true at SS1 of round R+1; SHIELD available for absorption

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/keyword/shield_test.rs` — must exist and pass

**Status**: [x] Passed 2026-05-01 (`cargo test -p server --test shield_test`)

---

## Dependencies

- Depends on: Story 001 (scaffold)
- Unlocks: Story 013 (COUNTERATTACK+SHIELD cross-test KW-050)

## Completion Notes

**Completed**: 2026-05-01
**Criteria**: 7/7 passing. KW-024, KW-037, simultaneous same-sub-step absorption, persistence across untriggered rounds, SS3-to-SS6 consumption, SS6-to-next-round consumption, and `KeywordTriggered { payload: ShieldConsumed, sub_step }` emission are covered by `tests/unit/keyword/shield_test.rs`.
**Deviations**: Advisory only - story manifest v2026-04-30 is older than current control manifest v2026-05-01. Advisory only - story text still says ADR-018 was Proposed/BLOCKED, while current ADR-018 is Accepted. Advisory only - ADR-018 contains an older immutable `check_shield_absorb` signature snippet, while the story-scoped behavior requires and implements mutable shield consumption.
**Test Evidence**: Logic evidence at `tests/unit/keyword/shield_test.rs`; `cargo test -p server --test shield_test` passed 7/7. Regression evidence: `cargo test -p server --test keyword_plugin_smoke_test` passed 2/2; `cargo check -p server` passed.
**Code Review**: Skipped - lean mode.
