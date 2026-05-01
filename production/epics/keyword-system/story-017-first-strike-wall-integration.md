# Story 017: FIRST STRIKE × WALL Integration

> **Epic**: Keyword System
> **Status**: Blocked
> **Layer**: Feature (M3)
> **Type**: Integration
> **Manifest Version**: 2026-04-30

## Context

**GDD**: `design/gdd/keyword-system.md`
**Requirement**: `TR-KW-003` — FIRST STRIKE attacks in sub-step 3; no retaliation until SS6. `TR-KW-009` — WALL keyword: MP=0 unit; halts advancing enemy at adjacent cell; collision-halt rule applies.
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-018 (Keyword System — ECS State Architecture, `effects.rs` section) + ADR-022 (Timing Trigger Observer Architecture, inline COUNTERATTACK/INJURED dispatch — for completeness check)
**ADR Decision Summary**: FIRST STRIKE (`apply_first_strike`) and WALL (`has_keyword(SimpleKeyword::Wall)` + 0 damage in SS6, collision halt in SS5) are both implemented in `effects.rs` and called by `server/feature/combat/`. This story tests the R3 cross-keyword interaction: a FIRST STRIKE unit CAN attack a WALL in SS3; if the WALL dies in SS3, it is removed in SS4 and no longer halts advancing enemies in SS5. This is intentional counter-play.

**BLOCKED**: ADR-018 is Proposed. Stories 001 (scaffold), 003 (FIRST STRIKE), and 007 (WALL collision logic) must be Done. Both FIRST STRIKE and WALL effect implementations must be in place before this integration story can be opened.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**: SS3 applies FIRST STRIKE attacks; SS4 removes dead units (HP ≤ 0); SS5 checks WALL presence for movement halt. The ordering guarantee is structural — SS3 → SS4 → SS5 executes in sequence inside `resolve_combat`. No special coordination code is needed: WALL is simply absent from `BoardState` in SS5 if it died in SS3 and was removed in SS4.

**Control Manifest Rules (Feature layer)**:
- Required: WALL removal in SS4 is structural — `remove_unit_from_board` in `execute_ss4()` removes WALL from all board queries; SS5 movement scan does not find it (KW-060)
- Forbidden: Never hardcode a "WALL died this round" flag — rely on board state absence; if WALL is not in `BoardState`, it does not halt movement

---

## Acceptance Criteria

*From GDD `design/gdd/keyword-system.md` Acceptance Criteria (R3 additions):*

- [ ] KW-059: GIVEN a FIRST STRIKE unit is co-located with an enemy WALL at SS3 entry, WHEN SS3 resolves, THEN the FIRST STRIKE unit deals damage to the WALL in SS3; the WALL deals 0 damage in response (WALL always deals 0 damage regardless of ATK card stat); if the FIRST STRIKE damage reduces the WALL's HP to 0, the WALL is removed in SS4 as a normal death
- [ ] KW-060: GIVEN an enemy WALL was killed by FIRST STRIKE in SS3 and removed in SS4 via the normal death process, WHEN SS5 resolves, THEN advancing friendly units are no longer blocked at the WALL's former cell — they pass through freely because no WALL entity exists at that cell in `BoardState`

---

## Implementation Notes

*Derived from ADR-018 effects.rs and GDD FIRST STRIKE + WALL interaction:*

**No new code required beyond Stories 003 and 007.** This story is a cross-keyword integration test verifying that the structural execution order of `resolve_combat` produces the correct result:

```
SS3: apply_first_strike(attacker, wall_entity, world)
     → wall HP reduces to 0 (e.g., wall HP=2, FS ATK=3)
     → wall NOT yet removed (SS4 removes it)
     → WALL deals 0 damage (has_keyword(SimpleKeyword::Wall) → outgoing_damage = 0)

SS4: execute_ss4(world)
     → collect dead units (wall_entity HP ≤ 0)
     → remove_unit_from_board(world, wall_entity)
     → wall_entity no longer in BoardState.units

SS5: movement pass
     → scan BoardState.units for WALL at former wall cell
     → no entity found (wall was removed in SS4)
     → advancing enemy passes through freely (no collision halt)
```

**The test proves the ordering guarantee**, not any new behavior. Stories 003 and 007 implement the primitives; this story verifies their correct integration.

**Design intent (GDD Decision D6):** FIRST STRIKE + CHARGE X can clear a WALL anchor before standard movement resolves — this is explicitly intended counter-play against WALL lane anchors. FIRST STRIKE units doing this must be co-located with the WALL at SS3 entry (i.e., they reached the WALL's cell during SS2 or were placed there in SS1).

---

## Out of Scope

- Story 003: FIRST STRIKE implementation
- Story 007: WALL collision logic implementation (0 damage, SS5 halt)
- Story 012: DEATH chain from WALL death (if WALL has DEATH keyword — an uncommon card design)

---

## QA Test Cases

*Automated test specs (Integration story):*

- **KW-059**: FIRST STRIKE attacks WALL in SS3; WALL deals 0 damage
  - Given: Player A FIRST STRIKE unit (ATK=5) co-located with Player B WALL (HP=3) at Cell 4; WALL has no DEATH keyword; Player A advance_dir=+1
  - When: SS3 resolves (FIRST STRIKE pass)
  - Then: Player A unit deals 5 damage to WALL (net: max(0, 5 - WALL.AR)); WALL's outgoing damage to Player A unit = 0 (WALL always deals 0); WALL HP = 3 − 5 = 0 (or negative; stored as 0)
  - Edge cases: if WALL HP > FIRST STRIKE ATK_effective (WALL survives SS3) — WALL still present in SS5; movement still halted

- **KW-060**: WALL removed in SS4; SS5 movement passes through
  - Given: (continuing from KW-059 scenario) WALL HP = 0 after SS3; Player B standard unit (MP=3) at Cell 2 that would normally be halted by WALL at Cell 4
  - When: SS4 runs — WALL removed from `BoardState.units`; SS5 movement runs
  - Then: Player B unit at Cell 2 moves normally (advance_dir=−1 toward Cell 1) — passes Cell 4 without halt; WALL absence in `BoardState` is the sole mechanism for this (no special flag needed)
  - Assertion: Player B unit's final cell after SS5 = max(1, 2 − 3) = 1 (or its advance formula result); NOT halted at Cell 4

---

## Test Evidence

**Story Type**: Integration
**Required evidence**: `tests/integration/keyword/first_strike_wall_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 (scaffold), Story 003 (FIRST STRIKE implementation), Story 007 (WALL collision logic), Story 012 (DEATH chain — WALL death processed as normal death in SS4) must be Done
- Unlocks: Epic is complete (this is the final integration story)
