# Story 002: Combat Modifier Stack — Pure Function

> **Epic**: Combat Resolution
> **Status**: Ready
> **Layer**: Feature
> **Type**: Logic
> **Manifest Version**: 2026-04-30

## Context

**GDD**: `design/gdd/combat-resolution.md`
**Requirement**: `TR-CR-???` (TR-CR-002, TR-CR-003 — unregistered)

**ADR Governing Implementation**: ADR-017: Combat Resolution Execution Architecture
**ADR Decision Summary**: `apply_combat_modifier_stack(attacker, defender) -> CombatResult` is a pure function with no ECS access. It applies 9 modifier steps in fixed order and returns `net_damage` + `ar_attacker_combat`. A two-pass algorithm handles simultaneous bilateral combat by running the stack twice with Pass 1's `ar_attacker_combat` feeding into Pass 2's defender AR.

**Engine**: Bevy 0.18 | **Risk**: HIGH (Bevy 0.18 struct/enum patterns)
**Engine Notes**: This function has no Bevy API dependencies — it is pure Rust. All tests run without a Bevy `World`. Place in `server/src/feature/combat/modifier_stack.rs`. No `#[derive(Component)]` or ECS traits needed here.

**Control Manifest Rules (Feature layer)**:
- Required: All intermediate arithmetic in `i32` before clamping to `u8` — never raw `u8` subtraction (panics in debug, wraps in release); read `type_advantage_atk_bonus` and `type_advantage_ar_bonus` from `Res<GameConfig>` — pass as parameters to keep the function pure
- Forbidden: No `unwrap()` in production paths; no hardcoded bonus values — read from `GameConfig`
- Guardrail: Function must complete in < 1µs per call; max 10 calls per unit pair per RESOLUTION

---

## Acceptance Criteria

*From GDD `design/gdd/combat-resolution.md`, scoped to this story:*

- [ ] **CR-12**: GIVEN ATK_attacker = 3 and AR_defender = 5, WHEN combat resolves, THEN net_damage = 0 (floor at 0, never negative)
- [ ] **CR-13**: GIVEN RESISTANCE 2 (AR=1) attacked by ATK=4, WHEN combat resolves, THEN ATK_effective = max(0, 4−2) = 2; net_damage = max(0, 2−1) = 1
- [ ] **CR-14**: GIVEN ARMOR-PIERCING (ATK=3) vs AR=4 + RESISTANCE 1, WHEN combat resolves, THEN ATK_effective = max(0, 3−1) = 2; AR_effective = 0 (ARMOR-PIERCING after RESISTANCE); net_damage = 2
- [ ] **CR-15**: GIVEN Blade-type unit attacks Arcane-type unit, WHEN combat resolves, THEN ATK_combat += `type_advantage_atk_bonus` (default +1); `ar_attacker_combat` = `type_advantage_ar_bonus` (default +1); base card stats unaffected
- [ ] **CR-42**: GIVEN VULNERABILITY 2 (AR=1) attacked by ATK=3, WHEN combat resolves, THEN ATK_effective = 3+2 = 5; net_damage = max(0, 5−1) = 4
- [ ] **CR-43**: GIVEN a SILENCEd unit with FIRST STRIKE and ARMOR-PIERCING attacks, THEN FIRST STRIKE stripped (step 1 SILENCE); ARMOR-PIERCING stripped (step 1 SILENCE); defender AR_base used normally

---

## Implementation Notes

*Derived from ADR-017 Key Interfaces and GDD Combat Modifier Stack section:*

```rust
// server/src/feature/combat/modifier_stack.rs

pub struct CombatResult {
    pub net_damage: u8,
    pub ar_attacker_combat: u8,  // type-advantage AR bonus for bilateral 2-pass
}

/// Pure function — no ECS access. Called for every individual attack.
pub fn apply_combat_modifier_stack(
    attacker: &UnitSnapshot,
    defender: &UnitSnapshot,
    config: &GameConfig,
) -> CombatResult {
    // Step 1: SILENCE — strip all keywords from attacker for this combat
    let keywords = if attacker.keywords.contains(Silenced) {
        KeywordSet::empty()
    } else {
        attacker.keywords
    };

    // Step 2: STUN — if attacker is STUNned, attack does not execute
    if keywords.contains(Stunned) {
        return CombatResult { net_damage: 0, ar_attacker_combat: 0 };
    }

    // Step 3: LEADER bonus
    let atk_leader = attacker.leader_atk_bonus as i32;

    // Step 4: Type advantage ATK
    let atk_type = if type_beats(attacker.unit_type, defender.unit_type) {
        config.type_advantage_atk_bonus as i32
    } else { 0 };

    // Step 5: VULNERABILITY X
    let atk_vuln = defender.vulnerability_x as i32;

    // Step 6: RESISTANCE X
    let atk_resist = defender.resistance_x as i32;

    // Step 7: ARMOR-PIERCING
    let armor_piercing = keywords.contains(ArmorPiercing);

    // Step 8: Type advantage AR
    let ar_attacker_combat = if type_beats(attacker.unit_type, defender.unit_type) {
        config.type_advantage_ar_bonus
    } else { 0 };

    // Formula — all in i32 to prevent underflow/overflow
    let atk_effective = (attacker.atk as i32
        + atk_leader + atk_type + atk_vuln - atk_resist)
        .max(0) as u8;

    let ar_effective = if armor_piercing { 0 } else { defender.ar };

    let net_damage = (atk_effective as i32 - ar_effective as i32).max(0) as u8;

    CombatResult { net_damage, ar_attacker_combat }
}

/// Cyclic type advantage triangle.
fn type_beats(attacker: UnitType, defender: UnitType) -> bool {
    matches!(
        (attacker, defender),
        (UnitType::Blade, UnitType::Arcane)
        | (UnitType::Arcane, UnitType::Shield)
        | (UnitType::Shield, UnitType::Blade)
    )
}
```

**Two-pass bilateral algorithm** (called from SS6 standard combat, not from this function directly):

```rust
// Pass 1: A attacks B
let result_a = apply_combat_modifier_stack(&a, &b, config);

// Pass 2: B attacks A — include A's type-advantage AR bonus from Pass 1
let mut a_as_defender = a.clone();
a_as_defender.ar = a.ar.saturating_add(result_a.ar_attacker_combat);
let result_b = apply_combat_modifier_stack(&b, &a_as_defender, config);

// Apply simultaneously
a.hp = a.hp.saturating_sub(result_b.net_damage);
b.hp = b.hp.saturating_sub(result_a.net_damage);
```

**Pre-implementation gate**: Add `type_advantage_atk_bonus` and `type_advantage_ar_bonus` fields to `game-config.md` and `assets/config/game_config.ron` before implementing (OQ2 action item from GDD).

---

## Out of Scope

- Story 001: Integration into `resolve_combat` exclusive system
- Story 005: Calling the modifier stack from SS3 FIRST STRIKE context
- Story 007: Calling the modifier stack from SS6 standard combat context

---

## QA Test Cases

*(Lean mode — test cases authored inline)*

- **CR-12** (damage floor):
  - Given: `UnitSnapshot { atk: 3, ar: 0 }` attacker, `UnitSnapshot { ar: 5 }` defender, no keywords
  - When: `apply_combat_modifier_stack` called
  - Then: `result.net_damage == 0`

- **CR-13** (RESISTANCE):
  - Given: attacker ATK=4, defender AR=1 + RESISTANCE 2
  - When: stack runs
  - Then: `ATK_effective = max(0, 4-2) = 2`; `net_damage = max(0, 2-1) = 1`

- **CR-14** (ARMOR-PIERCING + RESISTANCE):
  - Given: attacker ATK=3 + ARMOR-PIERCING, defender AR=4 + RESISTANCE 1
  - When: stack runs
  - Then: `ATK_effective = max(0, 3-1) = 2`; `AR_effective = 0`; `net_damage = 2`
  - Edge case: RESISTANCE applied BEFORE ARMOR-PIERCING strips AR

- **CR-15** (type advantage):
  - Given: Blade attacker vs Arcane defender, `type_advantage_atk_bonus=1`, `type_advantage_ar_bonus=1`
  - When: stack runs
  - Then: `ATK_effective` includes +1; `ar_attacker_combat = 1`

- **CR-42** (VULNERABILITY):
  - Given: attacker ATK=3, defender AR=1 + VULNERABILITY 2
  - When: stack runs
  - Then: `ATK_effective = 3+2 = 5`; `net_damage = max(0, 5-1) = 4`

- **CR-43** (SILENCE strips everything):
  - Given: attacker with SILENCE status, keywords include ARMOR-PIERCING
  - When: stack runs (SILENCE present in step 1)
  - Then: ARMOR-PIERCING stripped; defender AR_base applied normally; `net_damage = max(0, atk - ar_base)`

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/combat/modifier_stack_test.rs` — must exist and pass without Bevy `World`

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 (UnitSnapshot struct defined in scaffold)
- Unlocks: Story 005 (SS3 FIRST STRIKE calls modifier stack), Story 007 (SS6 standard combat calls modifier stack)
