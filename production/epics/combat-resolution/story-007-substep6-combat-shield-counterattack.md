# Story 007: Sub-step 6 — Standard Combat + SHIELD + COUNTERATTACK

> **Epic**: Combat Resolution
> **Status**: Complete
> **Layer**: Feature
> **Type**: Logic
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/combat-resolution.md`
**Requirement**: `TR-CR-004` (SS6 bilateral snapshot combat), `TR-CR-005` (CR-6, CR-7, CR-29, CR-36 — SHIELD consumption and persistence), `TR-CR-006` (CR-20, CR-21, CR-35 — COUNTERATTACK melee eligibility and retaliation)

**ADR Governing Implementation**: ADR-017: Combat Resolution Execution Architecture
**ADR Decision Summary**: SS6 runs the two-pass bilateral combat algorithm, SHIELD pre-check (runs before the modifier stack, absorbs all simultaneous attackers, consumed once per sub-step), and COUNTERATTACK retaliation (melee-only, fires once per sub-step after all incoming damage, runs full modifier stack). Dead unit cleanup and kill gold drain run as a post-SS6 pass.

**Engine**: Bevy 0.18 | **Risk**: HIGH
**Engine Notes**: HP mutations use `saturating_sub`. Two-pass bilateral algorithm reads HP before either mutation is applied (snapshot discipline). COUNTERATTACK chain stops after one retaliation — do not allow infinite chains. All ECS mutations via `world.resource_mut::<BoardState>()`.

**Control Manifest Rules (Feature layer)**:
- Required: SHIELD pre-check runs BEFORE modifier stack; SHIELD absorbs all simultaneous attackers in one sub-step; COUNTERATTACK fires AFTER all incoming damage resolved (including SHIELD absorption); post-SS6 cleanup pass drains kill_log for SS6 kills
- Forbidden: SHIELD does not suppress COUNTERATTACK; COUNTERATTACK does not fire for RANGE attackers at distance
- Performance: SS6 contributes to ADR-017's <= 15 ms RESOLUTION budget; combat pair collection, SHIELD absorption grouping, COUNTERATTACK retaliation, and post-SS6 cleanup must remain bounded by live unit count plus the 10,000 iteration safety guard.

---

## Acceptance Criteria

*From GDD `design/gdd/combat-resolution.md`, scoped to this story:*

- [ ] **CR-6**: GIVEN a unit with SHIELD receives damage in sub-step 3, WHEN sub-step 3 resolves, THEN all sub-step 3 damage is negated and SHIELD is consumed; WHEN sub-step 6 attacks that same unit, THEN damage is applied normally (SHIELD already consumed)
- [ ] **CR-7**: GIVEN a unit with SHIELD receives no damage during RESOLUTION, WHEN the next round's RESOLUTION begins, THEN SHIELD is still active (persists between rounds until consumed)
- [ ] **CR-20**: GIVEN a unit with COUNTERATTACK receives damage from a RANGE attacker that did NOT occupy the target's cell, WHEN damage is applied, THEN COUNTERATTACK does NOT fire
- [ ] **CR-21**: GIVEN a unit with COUNTERATTACK receives damage from a melee attacker (same-cell contact OR adjacent-cell collision-halt combat) in sub-step 3 or sub-step 6, WHEN damage is received (after SHIELD pre-check), THEN COUNTERATTACK fires in that same sub-step
- [ ] **CR-29**: GIVEN a RANGE + FIRST STRIKE unit attacks a SHIELD unit in sub-step 3 (consuming SHIELD), WHEN sub-step 6 executes the second attack from the same unit, THEN the attack deals full damage (SHIELD consumed in sub-step 3 does not protect in sub-step 6)
- [ ] **CR-35**: GIVEN two melee units that halted on adjacent cells after path-crossing collision in sub-step 5, WHEN sub-step 6 resolves, THEN COUNTERATTACK fires for any unit with the COUNTERATTACK keyword (collision-halt adjacency satisfies melee contact)
- [ ] **CR-36**: GIVEN a unit with SHIELD is attacked simultaneously by two FIRST STRIKE units from different lanes in sub-step 3, WHEN sub-step 3 resolves, THEN the SHIELD unit takes 0 damage from BOTH attackers AND SHIELD is consumed exactly once

---

## Implementation Notes

*Derived from ADR-017 Architecture Diagram (sub-step 6) and GDD SS6 rules:*

```
SS6 execution order (execute_combat):

For each combat pair (melee or RANGE who advanced to target's cell):

  A. SHIELD pre-check (runs BEFORE modifier stack):
     - If defender has SHIELD: negate ALL incoming damage this sub-step (all attackers)
     - Consume SHIELD once (set shield_active = false)
     - Log: KeywordTriggered(SHIELD)
     - Proceed to COUNTERATTACK check (SHIELD consumed does not prevent COUNTERATTACK)

  B. If SHIELD did NOT absorb (or defender has no SHIELD):
     - Bilateral pair (A↔B): run two-pass algorithm
       Pass 1: result_a = apply_modifier_stack(A, B)
       Pass 2: B_as_defender.ar += result_a.ar_attacker_combat
               result_b = apply_modifier_stack(B, B_as_defender)
       Apply simultaneously: A.hp = A.hp.saturating_sub(result_b.net_damage)
                            B.hp = B.hp.saturating_sub(result_a.net_damage)
     - Multi-source on single target: apply in lane order sequentially

  C. COUNTERATTACK (after all incoming damage for this sub-step):
     - Check if defender has COUNTERATTACK AND attacker was in melee contact
       (same-cell OR collision-halt adjacent cell — NOT RANGE attacker at distance)
     - If yes: run modifier_stack(defender_as_attacker, original_attacker)
       - Original attacker's SHIELD pre-check applies independently
       - FINAL BLOW eligible for COUNTERATTACK kill
       - If original attacker also has COUNTERATTACK: one retaliatory retaliation (chain stops)

Post-SS6 cleanup pass:
  - Despawn all units at HP == 0
  - Drain kill_log for SS6 kills → emit GoldAwarded entries
  - Emit FINAL BLOW entries for SS6 kills
```

**CR-29 (SHIELD across sub-steps)**: `shield_active` is a field on `UnitSnapshot`. SS3 SHIELD pre-check sets `shield_active = false` after consumption. SS6 checks `shield_active` independently — if it was consumed in SS3, it is `false` in SS6 and no protection applies.

**CR-36 (multiple FS attackers in SS3 → one SHIELD consumption)**: The SHIELD pre-check in SS3 collects ALL attackers targeting this unit in SS3 before running the check. If any attackers exist → SHIELD absorbs all → consumed once, regardless of how many attackers.

---

## Out of Scope

- Story 008: RANGE targeting logic (this story handles melee and adjacent-cell combat; RANGE is Story 008)
- Story 009: Objective damage (fires after unit combat in SS6; separate story)
- Story 006: Kill gold drain for SS3 kills (SS6 kills drained in this story's post-SS6 pass)

---

## QA Test Cases

*(Lean mode — test cases authored inline)*

- **CR-6** (SHIELD consumed in SS3, gone in SS6):
  - Given: Unit with SHIELD; FS attacker deals 3 damage in SS3
  - When: SS3 runs (SHIELD absorbs) → SS6 runs (same unit attacked again)
  - Then: SS3 damage = 0 (SHIELD absorbed); SS6 damage = normal net_damage (no SHIELD)

- **CR-7** (SHIELD persists between rounds):
  - Given: Unit with SHIELD, no attacks land this RESOLUTION
  - When: RESOLUTION completes; next round's RESOLUTION begins
  - Then: `snapshot.shield_active == true` at next RESOLUTION entry

- **CR-20** (COUNTERATTACK NOT triggered by RANGE at distance):
  - Given: COUNTERATTACK unit at cell 5; RANGE attacker at cell 3 (within range, did not advance)
  - When: SS6 RANGE attack deals damage
  - Then: No `KeywordTriggered { keyword: COUNTERATTACK }` in log for this pair

- **CR-21** (COUNTERATTACK fires for melee):
  - Given: COUNTERATTACK unit at cell 5; melee attacker also at cell 5 (same-cell)
  - When: SS6 melee damage applied
  - Then: `KeywordTriggered { keyword: COUNTERATTACK, sub_step: 6 }` in log; retaliation `CombatDamage` emitted

- **CR-35** (COUNTERATTACK at collision-halt adjacency):
  - Given: Unit A (cell 4) and Unit B with COUNTERATTACK (cell 5) — path-crossing halted in SS5
  - When: SS6 executes (A attacks B at adjacent cells)
  - Then: COUNTERATTACK fires; B retaliates against A with full modifier stack

- **CR-36** (SHIELD absorbs multiple FS sources once):
  - Given: SHIELD unit targeted by FS unit X (Lane 2) and FS unit Y (Lane 4) in SS3
  - When: SS3 resolves
  - Then: SHIELD unit takes 0 damage; SHIELD consumed exactly once; `KeywordTriggered(SHIELD)` appears once

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/combat/substep6_combat_shield_counterattack_test.rs` — must exist and pass

**Status**: [x] Created and passed

---

## Dependencies

- Depends on: Story 002 (modifier stack), Story 004 (post-SS5 positions), Story 006 (dead units from SS4 removed before SS6)
- Unlocks: Story 008 (RANGE targeting is part of SS6), Story 009 (objective damage follows SS6 combat)

## Completion Notes

**Completed**: 2026-05-04
**Verdict**: COMPLETE WITH NOTES
**Criteria**: 7/7 passing. CR-6, CR-7, CR-20, CR-21, CR-29, CR-35, and CR-36 are covered by `tests/unit/combat/substep6_combat_shield_counterattack_test.rs`.
**Test Evidence**: `cargo test -p server --test substep6_combat_shield_counterattack_test` passed 7/7. Regression slice `cargo test -p server --test movement_collision_test --test substep3_first_strike_test --test substep4_dead_removal_test` passed 13/13. `cargo check -p server` and `git diff --check` passed.
**Verification**: Current `main` includes integrated commit `2c8f752`; SS6 standard unit-vs-unit combat runs through `execute_standard_combat`, SHIELD absorbs grouped incoming damage once per sub-step and persists until consumed, COUNTERATTACK fires for same-cell and collision-halt adjacent melee contact after incoming damage or SHIELD absorption, and RANGE attackers at distance do not trigger COUNTERATTACK.
**Notes**: No blocking GDD, ADR, Bevy 0.18, or implementation deviation found. Advisory only - current GDD CR-21 wording still says COUNTERATTACK fires before the SHIELD absorption check, while active `TR-CR-006`, this story, and ADR-017 specify after incoming damage or SHIELD absorption; implementation follows the active TR/story contract. No Story 008 RANGE targeting scope creep found; this closure only verifies the narrow RANGE + FIRST STRIKE support needed by CR-20 and CR-29. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates.
**Tech Debt Logged**: None.
