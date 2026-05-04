# Story 009: Sub-step 6 — Objective Damage + GAME_OVER

> **Epic**: Combat Resolution
> **Status**: Ready
> **Layer**: Feature
> **Type**: Logic
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/combat-resolution.md`
**Requirements**:
- `TR-CR-009` — CR-10 / CR-27 objective damage: bypasses AR, uses `ATK_effective`, and clamps objective HP with `saturating_sub`.
- `TR-OBJ-004` — objective destruction consequence: mark destroyed, queue `ObjectiveDestroyed`, award +3g if attacker != owner, dispatch fake rewards, increment real objective count if real.
- `TR-OBJ-006` — objective loss counter: `real_objectives_destroyed >= loss_threshold (2)`, mutual destruction = Draw.
- `TR-OBJ-010` — `take_damage(lane, attacker_player, amount)` is the sole objective damage interface; `ObjectiveCounters` is the RSM-readable contract.
- `TR-RSM-008` — GAME_OVER detection is evaluated by the RSM reading `ObjectiveCounters` after `ResolutionComplete`.

**ADR Governing Implementation**: ADR-017: Combat Resolution Execution Architecture + ADR-002: Client-Server Authority + ADR-010: RSM Phase Event Bus + ADR-019: Economy System Resource Architecture + ADR-001: Hidden Objective Identity
**ADR Decision Summary**: After all unit-vs-unit combat in SS6 resolves, any unit at Cell 8 (Player A) or Cell 1 (Player B) deals its `ATK_effective` to the objective in that lane via Objective System `take_damage`. Objective destruction awards +3g through the economy API, not +1 kill gold. Combat Resolution completes normally: it records the Story 009 local log entries, queues/drains `ResolutionComplete` through the current completion bridge, and leaves objectives-triggered GAME_OVER evaluation to the RSM. Economy consumes `ResolutionComplete` first to snapshot interest after kill/objective rewards; the RSM then reads `ObjectiveCounters` and emits `GameOverEmitted` for single-loser or Draw outcomes.

**Engine**: Bevy 0.18 | **Risk**: HIGH
**Engine Notes**: Objective HP mutations route through `objective-system`'s `take_damage` interface (ADR-010 boundary — Combat Resolution calls into Objective System via the API boundary, not directly). Gold award uses `EconomySystem::apply_gold_award`. Normal completion uses the ADR-019 `PendingResolutionComplete` bridge: `resolve_combat` sets the pending flag, `drain_pending_resolution_complete` writes `ResolutionComplete`, Economy snapshots interest before `rsm_input_reader`, and the RSM owns `GameOverEmitted`. Direct `S2CGameOver` broadcast is not a Combat Resolution responsibility.
**Performance Budget**: The objective damage pass runs inside ADR-017's full RESOLUTION budget of <= 15 ms for a worst-case 5-lane contested round. Candidate collection and objective API calls must stay bounded by live units at objective cells plus objective destruction events.

**Control Manifest Rules (Feature layer)**:
- Required: Objective damage uses `ATK_effective` including LEADER/spell buffs but excludes AR modifiers (objectives have AR=0; ARMOR-PIERCING/RESISTANCE/VULNERABILITY don't apply); `ObjectiveDestroyed` event emitted by Objective System; `ResolutionComplete` is emitted through the ADR-019 bridge after combat completes; Economy snapshots interest from `MessageReader<ResolutionComplete>` before the RSM input reader evaluates GAME_OVER.
- Forbidden: Never award +1 kill gold for objective destruction; FIRST STRIKE does NOT advance objective damage to SS3 — objective damage is always SS6 only; Combat Resolution must not broadcast `S2CGameOver` directly or suppress normal `ResolutionComplete` for objectives-triggered GAME_OVER.

---

## Acceptance Criteria

*From GDD `design/gdd/combat-resolution.md`, scoped to this story:*

- [ ] **CR-10**: GIVEN a unit alive at Cell 8 at the end of sub-step 6, WHEN sub-step 6 completes, THEN that unit deals its ATK value as damage to the objective in that lane AND the unit remains at Cell 8 (attacks again next round unless killed)
- [ ] **CR-11**: GIVEN a unit with FIRST STRIKE is at Cell 8 and is killed in sub-step 3, WHEN sub-step 4 removes it, THEN it does NOT deal objective damage in sub-step 6 (unit must be alive at end of SS6 to deal objective damage)
- [ ] **CR-17**: GIVEN a unit at Cell 8 destroys an objective, WHEN sub-step 6 completes, THEN the attacking player receives +3 gold AND does NOT additionally receive +1 kill gold (objectives are not units)
- [ ] **CR-18**: GIVEN the 2nd real objective of Player B is destroyed, WHEN Combat Resolution completes normally and the RSM evaluates `ObjectiveCounters`, THEN `GameOverEmitted { loser: Some(PlayerB), reason: ObjectivesDestroyed }` is emitted and the server broadcasts `S2CGameOver { loser: Some(PlayerB), reason: ObjectivesDestroyed }` on the reliable channel
- [ ] **CR-19**: GIVEN both players' 2nd real objectives are destroyed in the same sub-step 6, WHEN Combat Resolution completes normally and the RSM evaluates `ObjectiveCounters`, THEN `GameOverEmitted { loser: None, reason: Draw }` is emitted and the server broadcasts `S2CGameOver { loser: None, reason: Draw }`
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

3. Completion and GAME_OVER handoff:
   - Combat Resolution does not own the win-condition decision. It finishes the objective pass after all objective destruction consequences have updated `ObjectiveCounters`.
   - Combat Resolution completes normally and uses the existing completion path: append Story 009 local entries to `ResolutionLog`, preserve the existing `S2CResolutionEvent` before `ResolutionComplete` ordering, set the `PendingResolutionComplete` bridge flag, and let `drain_pending_resolution_complete` emit `ResolutionComplete`.
   - Economy reads `ResolutionComplete` before `rsm_input_reader` to snapshot post-reward gold for interest.
   - RSM reads `ObjectiveCounters.real_objectives_destroyed(player)` after `ResolutionComplete`; if one player has `>= 2`, RSM emits `GameOverEmitted { loser: Some(player), reason: ObjectivesDestroyed }`; if both players have `>= 2`, RSM emits `GameOverEmitted { loser: None, reason: Draw }`.
   - Game Session / network dispatch owns the reliable `S2CGameOver` broadcast that follows `GameOverEmitted`.
```

**CR-11 verification**: Unit at Cell 8 that is killed in SS3 is removed in SS4. The objective damage pass in step 1 only checks units **alive at SS6 end** — dead units are not present in `snapshots` (marked `already_removed = true`). If such a unit was in the kill_log with `lethal_sub_step: 3`, it was removed in SS4 and will not appear here.

**CR-19 (simultaneous Draw)**: Process ALL objective destruction consequences in the SS6 pass before completing resolution. If both players have `real_objectives_destroyed >= 2`, the RSM detects the Draw after `ResolutionComplete`. Combat Resolution must not short-circuit on the first destruction.

**Fake objectives**: `is_fake` is only known server-side (ADR-001). `ObjectiveDestroyed { is_fake: bool }` in the ResolutionLog is populated server-side. Client-visible identity filtering remains owned by Objective System / protocol projection. Combat Resolution just passes through the server-side `is_fake` value from the `ObjectiveDestroyed` event into local log data where required.

---

## Out of Scope

- Story 007: Unit-vs-unit combat that precedes objective damage within SS6
- Objective System epic: `take_damage` API, fake reward dispatch, `ObjectiveCounters` update — owned by that system
- RSM win-condition implementation: `ObjectiveCounters` evaluation and `GameOverEmitted` are owned by Round State Machine Story 004
- Game Session / network dispatch: reliable `S2CGameOver` broadcast is owned outside Combat Resolution
- COMBAT-011: full `S2CResolutionEvent` completeness, chronological ordering, and client-observable ordering verification. Story 009 may add local `ObjectiveDamage`, `ObjectiveDestroyed`, and `GoldAwarded` log records only as needed for its objective-damage behavior.

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
  - When: Combat Resolution completes and the RSM evaluates counters after `ResolutionComplete`
  - Then: Player B's counter reaches 2; `ResolutionComplete` remains emitted through the normal bridge; RSM emits `GameOverEmitted { loser: Some(PlayerB), reason: ObjectivesDestroyed }`; Combat Resolution does not directly broadcast `S2CGameOver`

- **CR-19** (simultaneous Draw):
  - Given: Both players' 2nd real objective destroyed in same SS6 pass
  - When: Combat Resolution completes and the RSM evaluates counters after all destructions are processed
  - Then: RSM emits `GameOverEmitted { loser: None, reason: Draw }`; no single-loser `GameOverEmitted`; Combat Resolution does not short-circuit before normal completion

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

- Depends on: COMBAT-007 (Complete — unit-vs-unit SS6 combat completes before objective pass), COMBAT-008 (Complete — RANGE attacks in SS6 complete before objective pass)
- Depends on: OBJECTIVE-005 (Complete — destruction consequence path updates objective HP/counters and queues `ObjectiveDestroyed`), OBJECTIVE-006 (Complete — fake reward draw path), OBJECTIVE-007 (Complete — RESOLUTION-end objective sync / pending objective event broadcast)
- Depends on: RSM Story 004 (Complete — RSM reads `ObjectiveCounters` after `ResolutionComplete` and emits `GameOverEmitted`) and Economy Story 003 current ADR-019 completion contract (Complete — `ResolutionComplete` drives post-reward interest snapshot before RSM input)
- Unlocks: Story 010 (persistent states span multiple rounds, building on full RESOLUTION execution), Story 011 (log completeness needs objective events)
