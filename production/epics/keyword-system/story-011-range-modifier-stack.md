# Story 011: RANGE Targeting + Modifier Keywords (RESISTANCE, ARMOR-PIERCING, VULNERABILITY)

> **Epic**: Keyword System
> **Status**: Blocked
> **Layer**: Feature (M3)
> **Type**: Logic
> **Manifest Version**: 2026-04-30

## Context

**GDD**: `design/gdd/keyword-system.md`
**Requirement**: TR-KW-011 (RANGE target selection nearest-enemy; equidistant tie-break via `range_equidistant_select` RNG slot); TR-KW-??? (RESISTANCE X — untraced); TR-KW-??? (ARMOR-PIERCING — untraced); TR-KW-??? (VULNERABILITY X — untraced)
*(Run `/architecture-review` to register RESISTANCE, ARMOR-PIERCING, VULNERABILITY TRs)*

**ADR Governing Implementation**: ADR-018 (effects.rs interface)
**ADR Decision Summary**: RANGE attack target selection is in `effects.rs`. RANGE bypasses BODYGUARD. RESISTANCE X, VULNERABILITY X, and ARMOR-PIERCING are applied within the `net_damage` modifier stack (steps 1–10) owned by `combat-resolution.md`. The modifier stack integration point is in the combat resolution epic; this story implements the effect function stubs that feed values into that stack.

**BLOCKED**: ADR-018 Proposed. Story 001 must be Done. Also blocked on ADR-005 `range_equidistant_select` seed slot registration (equidistant tie-break requires RNG — KW-016 RANGE+BODYGUARD test can run without RNG; equidistant test requires it).

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**: RANGE target selection reads `BoardState` positions to find nearest-forward enemy. Equidistant tie-break uses `ServerRng` (ADR-005 `range_equidistant_select` slot) — must verify slot is registered before implementing.

**Control Manifest Rules (Feature layer)**:
- Required: All randomness (equidistant tie-break) must use server-side seeded RNG via `ServerRng` resource (ADR-005)
- Forbidden: Never use client-side RNG — all RANGE tie-breaks computed server-side; result broadcast to clients

---

## Acceptance Criteria

*From GDD `design/gdd/keyword-system.md` Acceptance Criteria:*

- [ ] KW-016: GIVEN a RANGE 1-X unit has an enemy BODYGUARD protecting another unit within range, WHEN the RANGE unit selects its target, THEN RANGE selects by proximity (nearest cell in forward direction); BODYGUARD Spell/Order protection does not intercept RANGE targeting
- [ ] KW-017: GIVEN a unit with RANGE and FIRST STRIKE, WHEN RESOLUTION executes, THEN the unit attacks in SS3 AND again in SS6; SHIELD consumed in SS3 does NOT protect the same unit in SS6 (SHIELD sub-step scoped — see Story 005)
- [ ] KW-022: GIVEN a defender with RESISTANCE 2 is attacked by a unit with ARMOR-PIERCING, WHEN the modifier stack resolves, THEN RESISTANCE 2 reduces ATK_effective by 2 first; ARMOR-PIERCING sets AR_defender to 0 independently; RESISTANCE is NOT bypassed by ARMOR-PIERCING
- [ ] KW-056: GIVEN unit X has RANGE granted via INJURED (INJURED-granted RANGE active at SS6 entry), WHEN SS6 resolves, THEN X attacks the nearest enemy within RANGE without advancing; X does NOT trigger COUNTERATTACK from the RANGE attack
- [ ] RANGE attacks cannot trigger COUNTERATTACK on the defender (RANGE attacker did not advance to target's cell — proximity condition not met)
- [ ] RANGE target selection: nearest enemy in the forward direction within `max_range` cells; equidistant tie-break via `range_equidistant_select` RNG slot (ADR-005) — KW-033b remains separately BLOCKED for equidistant test

---

## Implementation Notes

*Derived from ADR-018 effects.rs and GDD Detailed Design — RANGE:*

**RANGE target selection:**
```rust
pub fn select_range_target(attacker: Entity, world: &World) -> Option<Entity>
```
- Find all enemies in the forward direction within `max_range` cells of attacker's cell
- Select nearest (smallest cell distance in advance direction)
- If multiple enemies at same distance (equidistant): use `ServerRng.range_equidistant_select` slot to break tie (ADR-005 amendment required)
- If no enemies in range: RANGE unit does not attack this sub-step

**RANGE bypasses BODYGUARD (KW-016):**
- `select_range_target` does NOT check `bodyguard_protects` on any target entity
- Proximity selection only — BODYGUARD protection is irrelevant to RANGE targeting

**RANGE + FIRST STRIKE (KW-017):**
- `has_keyword(SimpleKeyword::Range)` AND `has_keyword(SimpleKeyword::FirstStrike)`
- Attacks in SS3 via FIRST STRIKE path AND attacks again in SS6 via RANGE path
- Two separate attack computations; SHIELD consumed in SS3 does not protect in SS6 (Story 005)

**RESISTANCE X modifier (GDD Formula 4 reference):**
- Applied at modifier stack step 6: `ATK_effective = max(0, ATK_effective - X)` — after LEADER bonus and VULNERABILITY, before AR step
- This reduces the running `ATK_effective` value (not the base card ATK)

**ARMOR-PIERCING modifier:**
- Applied at modifier stack step 7: `AR_effective = 0` for the defender
- Does NOT bypass RESISTANCE — RESISTANCE is applied at step 6 (before AR step); ARMOR-PIERCING only zeroes AR at step 7

**VULNERABILITY X modifier:**
- Applied at modifier stack step 5: `ATK_effective = ATK_effective + X`

**INJURED-granted RANGE (KW-056):**
- When `eval_injured_bonuses()` grants RANGE to a unit, the unit attacks via `select_range_target()` in SS6 using the `max_range` value from the card's INJURED-RANGE definition
- Does not trigger COUNTERATTACK (RANGE attacker did not advance to target cell)

---

## Out of Scope

- Story 014: INJURED-granted RANGE activation timing (`eval_injured_bonuses` call)
- Story 005: SHIELD consumed in SS3 not protecting in SS6 (KW-017 depends on Story 005)
- Story 013: COUNTERATTACK not triggered by RANGE (KW-005 — proximity condition fails for RANGE attacker)
- Full modifier stack integration lives in combat-resolution epic

---

## QA Test Cases

- **AC-1**: KW-016 — RANGE bypasses BODYGUARD protection
  - Given: BODYGUARD G protects unit X; RANGE unit Y (RANGE=3) at Cell 2; X at Cell 5 (nearest in range); G at Cell 5 too
  - When: RANGE target selection runs
  - Then: Y selects X as target (nearest within range); BODYGUARD G does not intercept; X takes RANGE damage
  - Edge cases: BODYGUARD at Cell 4 protecting X at Cell 5 — RANGE still hits X directly

- **AC-2**: KW-017 — RANGE+FIRST STRIKE attacks twice
  - Given: RANGE+FIRST STRIKE unit; enemy at Cell 6 within range
  - When: SS3 executes (FIRST STRIKE) then SS6 executes (RANGE second attack)
  - Then: unit attacks in SS3; unit attacks again in SS6; total 2 attacks this RESOLUTION
  - Edge cases: if enemy dies in SS3, no target in SS6 (RANGE attack skipped)

- **AC-3**: KW-022 — RESISTANCE + ARMOR-PIERCING stack correctly
  - Given: attacker (ATK=5, ARMOR-PIERCING); defender (AR=3, RESISTANCE 2)
  - When: damage computed via modifier stack
  - Then: step 6 — `ATK_effective = max(0, 5 - 2) = 3` (RESISTANCE applied); step 7 — `AR_effective = 0` (ARMOR-PIERCING, independent of RESISTANCE); net_damage = 3 - 0 = 3
  - Edge cases: ARMOR-PIERCING does NOT bypass RESISTANCE; RESISTANCE reduces effective ATK before AR zeroing

- **AC-4**: KW-056 — INJURED-granted RANGE does not trigger COUNTERATTACK
  - Given: unit X with INJURED-granted RANGE active; attacks enemy Y from range (did not advance to Y's cell)
  - When: SS6 RANGE attack resolves
  - Then: Y takes damage; Y's COUNTERATTACK does NOT fire (RANGE attacker proximity condition fails — KW-005)
  - Edge cases: Y has COUNTERATTACK keyword; still does not fire for RANGE attack

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/keyword/range_modifier_stack_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 (scaffold), Story 005 (SHIELD — for KW-017 RANGE+FIRST STRIKE+SHIELD interaction), Story 010 (BODYGUARD — for KW-016 bypass test), Story 014 (INJURED bonus — for KW-056 INJURED-granted RANGE)
- Depends on: ADR-005 `range_equidistant_select` seed slot registered (for equidistant tie-break; non-equidistant test KW-016 can proceed without it)
- Unlocks: Story 013 (COUNTERATTACK — proximity condition already verified here for RANGE)
