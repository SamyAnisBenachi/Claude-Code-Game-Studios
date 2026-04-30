# Story 009: Sub-step 6 — Objective Damage + GAME_OVER

> **Epic**: Combat Resolution
> **Status**: Ready
> **Layer**: Feature
> **Type**: Logic
> **Manifest Version**: 2026-04-30

## Context

**GDD**: `design/gdd/combat-resolution.md`
**Requirement**: `TR-CR-???` (TR-CR-009 partial — unregistered)

**ADR Governing Implementation**: ADR-017: Combat Resolution Execution Architecture + ADR-002: Client-Server Authority
**ADR Decision Summary**: After all unit-vs-unit combat in SS6 resolves, any unit at Cell 8 (Player A) or Cell 1 (Player B) deals its ATK_effective to the objective in that lane. Objective destruction awards +3g (not +1 kill gold). Loss condition is checked after all objective damage; simultaneous destruction → Draw. Server broadcasts `S2CGameOver` and does NOT write `ResolutionComplete` (the Draw path in Story 001's iteration guard covers the timeout case; this story covers the normal combat-driven GAME_OVER).

**Engine**: Bevy 0.18 | **Risk**: HIGH
**Engine Notes**: Objective HP mutations route through `objective-system`'s `take_damage` interface (ADR-010 boundary — Combat Resolution calls into Objective System via the API boundary, not directly). Gold award uses `EconomySystem::apply_gold_award`. `S2CGameOver` is a Lightyear reliable broadcast.

**Control Manifest Rules (Feature layer)**:
- Required: Objective damage uses `ATK_effective` including LEADER/spell buffs but excludes AR modifiers (objectives have AR=0; ARMOR-PIERCING/RESISTANCE/VULNERABILITY don't apply); `ObjectiveDestroyed` event emitted by Objective System; `ResolutionComplete` NOT written if `GameOver` emitted here
- Forbidden: Never award +1 kill gold for objective destruction; FIRST STRIKE does NOT advance objective damage to SS3 — objective damage is always SS6 only

---

## Acceptance Criteria

*From GDD `design/gdd/combat-resolution.md`, scoped to this story:*

- [ ] **CR-10**: GIVEN a unit alive at Cell 8 at the end of sub-step 6, WHEN sub-step 6 completes, THEN that unit deals its ATK value as damage to the objective in that lane AND the unit remains at Cell 8 (attacks again next round unless killed)
- [ ] **CR-11**: GIVEN a unit with FIRST STRIKE is at Cell 8 and is killed in sub-step 3, WHEN sub-step 4 removes it, THEN it does NOT deal objective damage in sub-step 6 (unit must be alive at end of SS6 to deal objective damage)
- [ ] **CR-17**: GIVEN a unit at Cell 8 destroys an objective, WHEN sub-step 6 completes, THEN the attacking player receives +3 gold AND does NOT additionally receive +1 kill gold (objectives are not units)
- [ ] **CR-18**: GIVEN the 2nd real objective of Player B is destroyed, WHEN the loss condition check runs, THEN the server broadcasts `S2CGameOver { loser: Player B, reason: ObjectivesDestroyed }` on the reliable channel
- [ ] **CR-19**: GIVEN both players' 2nd real objectives are destroyed in the same sub-step 6, WHEN the loss condition check runs, THEN the server broadcasts `S2CGameOver { loser: None, reason: Draw }`
- [ ] **CR-27**: GIVEN a unit at Cell 8 with ATK=3 attacks an objective with HP=2, WHEN sub-step 6 completes, THEN objective HP = 0 (floor at 0; NOT negative); the objective is destroyed

---

## Implementation Notes

*Derived from ADR-017, GDD SS6 objective damage rules, and objective-system.md interface:*

```
Objective damage pass (runs AFTER all unit-vs-unit SS6 combat):

1. Collect all alive units at Cell 8 (Player A objectives) or Cell 1 (Player B objectives)
2. For each such unit:
   a. Compute ATK_effective (include LEADER bonus + active spell buffs; no AR modifiers)
   b. Call objective_system.take_damage(lane, attacking_player, atk_effective)
      → Objective System applies: HP_new = max(0, HP_current - atk_effective)
      → If HP → 0: ObjectiveDestroyed fires, gold awarded, fake rewards dispatched
   c. Log: ObjectiveDamage { attacker_id, lane, damage_amount, objective_hp_after }
   d. If destroyed: Log: ObjectiveDestroyed { lane, owner, is_fake }
                    Log: GoldAwarded { player: attacker, amount: 3, reason: ObjectiveDestroyed }
                         (only if attacker != owner — no self-destruction reward)

3. Loss condition check (reads ObjectiveCounters resource):
   - If any player has real_objectives_destroyed >= 2:
     - Mutual (both players): broadcast S2CGameOver { loser: None, reason: Draw }
     - Single player: broadcast S2CGameOver { loser: that_player, reason: ObjectivesDestroyed }
   - On GAME_OVER: write GameOverEmitted message to RSM; do NOT write ResolutionComplete
```

**CR-11 verification**: Unit at Cell 8 that is killed in SS3 is removed in SS4. The objective damage pass in step 1 only checks units **alive at SS6 end** — dead units are not present in `snapshots` (marked `already_removed = true`). If such a unit was in the kill_log with `lethal_sub_step: 3`, it was removed in SS4 and will not appear here.

**CR-19 (simultaneous Draw)**: Collect ALL objective destructions in the SS6 pass before checking loss conditions. If after processing all destructions both players have `real_objectives_destroyed >= 2`, broadcast Draw. Do not short-circuit on the first destruction.

**Fake objectives**: `is_fake` is only known server-side (ADR-001). `ObjectiveDestroyed { is_fake: bool }` in the ResolutionLog is populated server-side. Clients with fake objectives that are destroyed see this field as `false` (their own objective is real from their perspective); the opponent sees `?`. This is handled by Objective System — Combat Resolution just passes through the `is_fake` from `ObjectiveDestroyed` event.

---

## Out of Scope

- Story 007: Unit-vs-unit combat that precedes objective damage within SS6
- Objective System epic: `take_damage` API, fake reward dispatch, `ObjectiveCounters` update — owned by that system

---

## QA Test Cases

*(Lean mode — test cases authored inline)*

- **CR-10** (objective damage applied, unit remains):
  - Given: Unit at cell 8 (Player A), ATK=3; objective HP=5
  - When: SS6 objective pass runs
  - Then: objective HP = 2; `ObjectiveDamage { damage_amount: 3, objective_hp_after: 2 }` in log; unit still in snapshots (not removed)

- **CR-11** (FS unit killed in SS3 → no objective damage):
  - Given: FS unit at cell 8, killed in SS3 (HP=0), removed in SS4
  - When: SS6 objective pass checks cell 8
  - Then: no `ObjectiveDamage` entry for this unit; objective HP unchanged

- **CR-17** (+3g not +1g):
  - Given: Unit destroys objective in SS6
  - When: post-destruction gold processing runs
  - Then: `GoldAwarded { amount: 3, reason: ObjectiveDestroyed }` in log; no `GoldAwarded { amount: 1, reason: Kill }` for objective
  - Edge case: self-inflicted destruction → no gold awarded (attacker == owner)

- **CR-18** (loss condition):
  - Given: Player B's `real_objectives_destroyed` counter = 1; Player B's objective at lane 3 is real and destroyed this SS6
  - When: loss condition check runs
  - Then: `S2CGameOver { loser: Some(PlayerB), reason: ObjectivesDestroyed }` broadcast; `ResolutionComplete` NOT written

- **CR-19** (simultaneous Draw):
  - Given: Both players' 2nd real objective destroyed in same SS6 pass
  - When: loss check runs after collecting all destructions
  - Then: `S2CGameOver { loser: None, reason: Draw }`; no single-loser broadcast

- **CR-27** (HP floor):
  - Given: Objective HP=2; unit ATK=3 (would reduce to -1)
  - When: `take_damage` called with amount=3
  - Then: objective HP = 0 (NOT -1); objective destroyed

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/combat/objective_damage_gameover_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 007 (unit-vs-unit SS6 combat completes before objective pass), Story 008 (RANGE attacks in SS6 complete before objective pass)
- Unlocks: Story 010 (persistent states span multiple rounds, building on full RESOLUTION execution), Story 011 (log completeness needs objective events)
