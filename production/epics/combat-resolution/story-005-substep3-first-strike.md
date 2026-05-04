# Story 005: Sub-step 3 — FIRST STRIKE Attacks

> **Epic**: Combat Resolution
> **Status**: Ready
> **Layer**: Feature
> **Type**: Logic
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/combat-resolution.md`
**Requirement**: `TR-CR-002` (CR-1, CR-2 - FIRST STRIKE SS3 damage and simultaneous HP snapshots), `TR-CR-007` (CR-22 - FINAL BLOW fires in the kill sub-step), `TR-CR-020` (CR-4 - RANGE + FIRST STRIKE emits SS3 and SS6 damage events), `TR-CR-021` (CR-37 - lane-order multi-source FIRST STRIKE damage and FINAL BLOW credit)

**ADR Governing Implementation**: ADR-017: Combat Resolution Execution Architecture
**ADR Decision Summary**: SS3 is a sequential function call within `resolve_combat`. All FIRST STRIKE units deal damage simultaneously using pre-combat HP snapshots. Multi-source damage on one target applies sequentially in lane order (Lane 1 first). Dead units from SS3 damage are NOT removed until SS4 — their FIRST STRIKE attack still resolves in SS3. `apply_combat_modifier_stack` (Story 002) is used for each individual attack.

**Engine**: Bevy 0.18 | **Risk**: HIGH
**Engine Notes**: HP mutations use `unit.hp = unit.hp.saturating_sub(damage)` — never raw `u8 -= u8`. FINAL BLOW (if applicable) fires as a `KeywordTriggered` log entry in SS3, not deferred to SS4. All ECS reads/writes go through `world.resource_mut::<BoardState>()`.

**Control Manifest Rules (Feature layer)**:
- Required: Pre-combat HP snapshots taken before any SS3 damage is applied; `apply_combat_modifier_stack` used for every individual attack; lane-order tiebreak for multi-source; dead units stay on board until SS4
- Forbidden: Never remove a dead unit in SS3; never apply HP mutations before all snapshot reads complete for bilateral pairs
- Performance: SS3 contributes to ADR-017's <= 15 ms RESOLUTION budget; collection, targeting, and damage application must remain bounded by lane count and live unit count.

---

## Acceptance Criteria

*From GDD `design/gdd/combat-resolution.md`, scoped to this story:*

- [ ] **CR-1**: GIVEN a unit with FIRST STRIKE in any lane, WHEN sub-step 3 executes, THEN that unit deals net_damage to any enemy unit sharing its cell before sub-step 5 movement occurs
- [ ] **CR-2**: GIVEN two FIRST STRIKE units sharing a cell, WHEN sub-step 3 executes, THEN both deal damage simultaneously (HP snapshots taken before either mutation is applied); if both receive lethal damage, both die and both DEATH triggers fire (in SS4)
- [ ] **CR-4**: GIVEN a unit with RANGE 1-X AND FIRST STRIKE, WHEN RESOLUTION executes, THEN a damage event is emitted in sub-step 3 (FIRST STRIKE pass) AND separately in sub-step 6 (standard combat pass) — two distinct `CombatDamage` entries in the log
- [ ] **CR-22**: GIVEN a unit kills another unit in sub-step 3 (FIRST STRIKE), WHEN FINAL BLOW fires, THEN it fires in sub-step 3 (before sub-step 4); the killed unit is still present on the board during FINAL BLOW resolution
- [ ] **CR-37**: GIVEN unit X (Lane 2, FS) and unit Y (Lane 4, FS) both target unit Z (HP=4, AR=0) in sub-step 3, WHEN sub-step 3 resolves, THEN Lane 2 damage applied first (Z HP → 1), then Lane 4 damage (Z HP → 0, killed); FINAL BLOW credit to Lane 4 unit's controller

---

## Implementation Notes

*Derived from ADR-017 Architecture Diagram (sub-step 3 section) and GDD SS3 rules:*

```
SS3 execution (execute_first_strike):
1. Collect all FIRST STRIKE units across all lanes
2. For each FS unit: determine target
   - Melee FS: target = enemy unit sharing the same cell
   - RANGE + FS: target = nearest enemy in forward direction within RANGE
     (uses range_equidistant_select RNG seed if equidistant)
3. Take HP snapshots of all targets BEFORE applying any damage
4. Apply damage to each target:
   - Single attacker on target: apply immediately
   - Multiple attackers on same target: apply in LANE ORDER (Lane 1 → Lane 5)
     HP updated between each source (sequential, not simultaneous)
5. For each unit reduced to 0 HP: record in kill_log; mark as dead (do NOT despawn)
   - Check FINAL BLOW: if kill occurred, fire KeywordTriggered(FINAL_BLOW) in SS3
6. After all SS3 damage: check for COUNTERATTACK triggers
   (melee FS attackers that moved to target's cell may trigger COUNTERATTACK on target)
```

**HP snapshot timing (CR-2)**: When two FS units face each other in the same cell, both compute their damage from pre-combat HP. Use a snapshot vector: snapshot all HPs before the SS3 loop, mutate after all computations in the loop. For multi-source on one target, HP is updated after each source (sequential rule per GDD).

**Dead units stay (CR-22)**: Set `snapshot.hp = 0` and push to `kill_log`. Do NOT call `world.despawn(entity)` here. SS4 handles despawning.

**STUN suppresses SS3 (CR-5)**: Check `if snapshot.is_stunned { continue; }` at the top of the FS loop.

**RANGE + FS targets (minimal SS3 scope)**: This story owns the SS3 FIRST STRIKE pass for RANGE + FIRST STRIKE units when there is a single nearest forward enemy within range, plus the CR-4 proof that this attacker can emit separate SS3 and SS6 `CombatDamage` entries during RESOLUTION. Full RANGE targeting remains Story 008: equidistant RNG, full forward-only matrix coverage, WALL-specific targeting, and target reacquisition after SS4 removal.

---

## Out of Scope

- Story 006: SS4 dead unit removal and kill gold drain (kill_log populated here, drained in SS4)
- Story 007: SHIELD pre-check and COUNTERATTACK response after SS3 damage
- Story 008: Full RANGE targeting matrix beyond the single-nearest SS3 RANGE + FIRST STRIKE case defined above

---

## QA Test Cases

*(Lean mode — test cases authored inline)*

- **CR-1** (FS damage before SS5):
  - Given: FS unit at cell 4, enemy at cell 4; both share a cell post-SS1
  - When: SS3 executes
  - Then: `CombatDamage` entry logged with `sub_step: 3`; target HP reduced before SS5 runs

- **CR-2** (mutual FS simultaneous):
  - Given: Unit A (FS, ATK=3, HP=2) and Unit B (FS, ATK=3, HP=2) sharing a cell, both AR=0
  - When: SS3 executes
  - Then: both HP snapshots taken before mutations; both reduced to -1 (clamped to 0); both marked dead; both DEATH triggers fire in SS4

- **CR-4** (RANGE+FS two damage events):
  - Given: RANGE 2 + FS unit at cell 3, enemy at cell 5
  - When: RESOLUTION completes
  - Then: ResolutionLog contains two `CombatDamage` entries for this attacker-defender pair: one with `sub_step: 3`, one with `sub_step: 6`

- **CR-22** (FINAL BLOW in SS3):
  - Given: FS unit kills enemy in SS3 (ATK=5, enemy HP=3, AR=0)
  - When: SS3 resolves
  - Then: `KeywordTriggered { keyword: FINAL_BLOW, sub_step: 3 }` appears in log; killed unit still present in board state (not despawned)

- **CR-37** (lane-order FINAL BLOW credit):
  - Given: Unit X (Lane 2, FS, ATK=3, AR=0) and Unit Y (Lane 4, FS, ATK=3, AR=0) both target Unit Z (HP=4, AR=0)
  - When: SS3 resolves
  - Then: Lane 2 damage applied first (Z.hp → 1); Lane 4 damage applied second (Z.hp → 0); `GoldAwarded` for FINAL BLOW credits Lane 4 controller

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/combat/substep3_first_strike_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 002 (calls `apply_combat_modifier_stack`), Story 003 (units placed in SS1), Story 004 (positions from SS2 used for SS3 targeting)
- Unlocks: Story 006 (SS4 drains kill_log populated here)
