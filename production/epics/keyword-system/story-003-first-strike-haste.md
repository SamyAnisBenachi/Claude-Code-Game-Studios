# Story 003: FIRST STRIKE + HASTE Keywords

> **Epic**: Keyword System
> **Status**: Complete
> **Layer**: Feature (M3)
> **Type**: Logic
> **Manifest Version**: 2026-04-30

## Context

**GDD**: `design/gdd/keyword-system.md`
**Requirement**: TR-KW-001 (HASTE rename; summoning-sickness removal), TR-KW-003 (FIRST STRIKE attacks in SS3)
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-018 (Keyword System — ECS State Architecture, effects.rs section)
**ADR Decision Summary**: Keyword effect functions live in `server/feature/keyword/effects.rs` and are called by `server/feature/combat/` as plain function calls. FIRST STRIKE is handled in `apply_first_strike()`. HASTE is a flag in the `cards.json` keyword array (via `SimpleKeyword::Haste` after ADR-006 amendment); the combat sub-step checks `has_keyword(SimpleKeyword::Haste)` to skip summoning sickness for that unit in SS2/SS3/SS5/SS6.

**BLOCKED**: ADR-018 Proposed. Also requires ADR-006 amendment merged (`SimpleKeyword::Haste` replaces `SimpleKeyword::Charge`). Story 001 must be Done.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**: `apply_first_strike` takes `world: &mut World` (exclusive system access per ADR-017). Query access pattern: `world.get::<UnitKeywordState>(entity)`. HP snapshots for simultaneous FIRST STRIKE must be taken BEFORE either damage computation mutates HP.

**Control Manifest Rules (Feature layer)**:
- Required: `apply_first_strike()` called by `server/feature/combat/` — keyword module does not schedule against combat timeline (ADR-018)
- Forbidden: Never read card balance values from hardcoded constants — read through `CardCatalog` resource loaded from `cards.json` (ADR-006)

---

## Acceptance Criteria

*From GDD `design/gdd/keyword-system.md` Acceptance Criteria, Combat Keywords section:*

- [x] KW-011: GIVEN a FIRST STRIKE unit is in the same cell as a standard enemy unit, WHEN SS3 resolves, THEN the FIRST STRIKE unit deals damage in SS3; the enemy does NOT deal damage in SS3; if the enemy survives, it attacks in SS6
- [x] KW-012: GIVEN two FIRST STRIKE units are co-located, WHEN SS3 resolves, THEN both deal damage simultaneously using pre-combat HP snapshots; neither's damage is computed after seeing the other's result
- [x] KW-013: GIVEN a unit with HASTE (no STUN, no FIRST STRIKE, no CHARGE X) is placed in SS1, WHEN RESOLUTION proceeds, THEN the unit participates in SS5 movement and SS6 attacks in the same round it entered play
- [x] KW-014: GIVEN a unit with HASTE has STUN applied via an SS1 APPEARANCE trigger, WHEN RESOLUTION proceeds, THEN the unit skips SS2, SS3, SS5, and SS6; HASTE does not override STUN
- [x] KW-042: GIVEN a unit with HASTE and FIRST STRIKE is placed in SS1 of round R, WHEN RESOLUTION proceeds through SS3, THEN the unit executes its FIRST STRIKE attack in SS3 of round R; it attacks again in SS6 if applicable
- [x] KW-043: GIVEN a unit with HASTE and CHARGE X (movement keyword) is placed in SS1 of round R at Cell 1, WHEN RESOLUTION proceeds, THEN the unit advances X extra cells in SS2 AND participates in SS5/SS6 — HASTE removes summoning sickness for SS2 as well as SS5/SS6

---

## Implementation Notes

*Derived from ADR-018 effects.rs interface and GDD Detailed Design:*

**FIRST STRIKE execution (GDD: Sub-step timing reference):**
- Fires in SS3 (before standard SS6 combat). Kills the target before it can retaliate.
- Two FIRST STRIKE units facing each other: deal damage simultaneously — snapshot both HP values before applying either damage delta.
- `apply_first_strike(attacker: Entity, target: Entity, world: &mut World) -> DamageResult` — called by `execute_ss3()` in combat resolution.
- FIRST STRIKE does NOT advance objective damage to SS3 (objective damage always resolves in SS6 from movement, not combat keyword attacks).

**HASTE (renamed from CHARGE per OQ-KS2 / ADR-018 Part 3):**
- `SimpleKeyword::Haste` in the extended enum — NOT `SimpleKeyword::Charge`
- HASTE is a flag: `has_keyword(SimpleKeyword::Haste)` on the unit's card definition (via `CardCatalog`)
- No separate field in `UnitKeywordState` — it is stateless; summoning sickness is removed by checking for the keyword at each sub-step gate
- Combat resolution checks: if unit entered play this round AND does NOT have HASTE → skip SS2 (CHARGE X), SS3 (FIRST STRIKE), SS5 (movement), SS6 (attacks)
- STUN overrides HASTE: `stun_active == true` suppresses ALL actions regardless of HASTE

**Simultaneous damage snapshot pattern:**
```rust
// For two FIRST STRIKE units in the same cell:
let hp_a_before = world.get::<UnitStats>(unit_a)?.hp;
let hp_b_before = world.get::<UnitStats>(unit_b)?.hp;
let damage_a_to_b = compute_damage(unit_a, unit_b, world); // uses hp_b_before in modifier stack
let damage_b_to_a = compute_damage(unit_b, unit_a, world); // uses hp_a_before in modifier stack
apply_hp_delta(unit_a, -damage_b_to_a, world);
apply_hp_delta(unit_b, -damage_a_to_b, world);
```

---

## Out of Scope

- Story 004: STUN suppression logic (tested here only as HASTE+STUN interaction via KW-014)
- Story 013: FINAL BLOW trigger when FIRST STRIKE kills (KW-035a)
- Story 012: DEATH chain from FIRST STRIKE kills (KW-035b)

---

## QA Test Cases

- **AC-1**: KW-011 — FIRST STRIKE kills before retaliation
  - Given: FIRST STRIKE unit A (ATK=3) at Cell 5; standard unit B (ATK=3, HP=2) at Cell 5
  - When: SS3 executes
  - Then: unit B receives 3 damage in SS3 (HP→0, removed in SS4); unit A receives 0 damage in SS3; if B survives (HP>0), it attacks A in SS6
  - Edge cases: B survives SS3 with 1 HP → B attacks A in SS6 normally

- **AC-2**: KW-012 — Two FIRST STRIKE units attack simultaneously
  - Given: FIRST STRIKE unit A (ATK=3, HP=4) at Cell 5; FIRST STRIKE unit B (ATK=2, HP=3) at Cell 5
  - When: SS3 executes
  - Then: A's post-SS3 HP = 4 - 2 = 2; B's post-SS3 HP = 3 - 3 = 0; neither sees the other's reduced HP during computation
  - Edge cases: if A also dies from B's damage, both die in SS4 simultaneously

- **AC-3**: KW-013 — HASTE unit participates same round
  - Given: HASTE unit placed in SS1 (no STUN, no FIRST STRIKE, no CHARGE X)
  - When: RESOLUTION executes SS5 and SS6
  - Then: unit advances its MP in SS5; unit attacks eligible target in SS6
  - Edge cases: HASTE unit with MP=0 (WALL) — still attacked in SS6

- **AC-4**: KW-014 — STUN overrides HASTE
  - Given: HASTE unit placed in SS1; STUN applied via APPEARANCE trigger in SS1
  - When: RESOLUTION proceeds
  - Then: unit skips SS2, SS3, SS5, SS6; `stun_active = true` overrides HASTE flag
  - Edge cases: HASTE without STUN is NOT suppressed; only STUN prevents action

- **AC-5**: KW-042 — HASTE+FIRST STRIKE same round
  - Given: HASTE+FIRST STRIKE unit placed in SS1 of round R
  - When: SS3 of round R executes
  - Then: unit attacks in SS3 (HASTE removes summoning sickness for SS3); unit may attack again in SS6 if applicable
  - Edge cases: without HASTE, a unit placed in SS1 skips SS3 (summoning sickness)

- **AC-6**: KW-043 — HASTE+CHARGE X same round
  - Given: HASTE+CHARGE 2 unit at Cell 1, placed in SS1 of round R
  - When: RESOLUTION executes
  - Then: SS2 — unit advances 2 extra cells (Cell 1 → Cell 3); SS5 — unit advances its MP; SS6 — unit attacks
  - Edge cases: HASTE removes summoning sickness for SS2 (CHARGE X) as well as SS5/SS6

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/keyword/first_strike_haste_test.rs` — must exist and pass

**Status**: [x] Created and passing

---

## Dependencies

- Depends on: Story 001 (scaffold — `effects.rs` exists with stub)
- Depends on: ADR-006 amendment merged (`SimpleKeyword::Haste` available)
- Unlocks: Story 012 (FINAL BLOW from FIRST STRIKE kill), Story 013 (COUNTERATTACK after FIRST STRIKE)

## Completion Notes

**Completed**: 2026-05-01
**Criteria**: 6/6 passing (KW-011, KW-012, KW-013, KW-014, KW-042, KW-043).
**Deviations**: Advisory only - story text says ADR-018 is Proposed/BLOCKED and embeds manifest v2026-04-30; current control manifest is v2026-05-01 and ADR-018 is Accepted. This was treated as stale story text, not a blocker. Full combat timeline integration remains owned by Combat Resolution Story 005; this story verifies the keyword-side logic.
**Test Evidence**: Logic test evidence exists at `tests/unit/keyword/first_strike_haste_test.rs`; executable suite `server/tests/first_strike_haste_test.rs`; `cargo test -p server --test first_strike_haste_test` passed 7/7. Shared card schema coverage also passed with `cargo test -p shared card::tests` (3/3).
**Code Review**: Skipped — lean review mode.
