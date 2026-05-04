# Story 010: Persistent Keyword States — INJURED, LEADER, OUTNUMBERED

> **Epic**: Combat Resolution
> **Status**: Ready
> **Layer**: Feature
> **Type**: Logic
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/combat-resolution.md`
**Related GDD**: `design/gdd/keyword-system.md`
**Requirement**: `TR-CR-010` (CR-26 — INJURED boundary activation), `TR-CR-011` (KW-027a, KW-027b, KW-040 — OUTNUMBERED boundary evaluation), `TR-CR-026` (CR-33, KW-025, KW-068 — LEADER post-SS1 snapshot and persistence), `TR-CR-027` (CR-34, KW-007 — INJURED cross-round persistence)

**ADR Governing Implementation**: ADR-017: Combat Resolution Execution Architecture; ADR-018: Keyword System ECS State Architecture; ADR-022: Keyword Timing Trigger Observer Architecture
**ADR Decision Summary**: Persistent keyword states (INJURED, LEADER snapshot, OUTNUMBERED) are evaluated at explicit combat boundaries, not mid-effect. INJURED activates at the SS3→SS4 boundary and persists into future rounds (granting FIRST STRIKE in the next round's SS3). LEADER bonus is snapshotted post-SS1, before SS2, after all SS1 APPEARANCE effects resolve; the snapshot persists through SS5/SS6 even if the LEADER dies in SS4. OUTNUMBERED is recomputed at sub-step boundaries and, after SS4, only after `ChainDeathBuffer` fully drains.

**Engine**: Bevy 0.18 | **Risk**: HIGH
**Engine Notes**: OUTNUMBERED board count is re-evaluated at each sub-step boundary by querying live `BoardState` after the previous sub-step fully completes — not incrementally during a DEATH chain. LEADER bonus snapshot is stored in keyword/combat state after SS1 resolves and before SS2 starts. INJURED state persists as a field on the unit's server-side state — survives between rounds.

**Control Manifest Rules (Feature layer)**:
- Required: INJURED evaluated at SS3→SS4 boundary (not during SS3 damage); LEADER snapshot post-SS1/pre-SS2, NOT re-evaluated after LEADER death; OUTNUMBERED re-evaluated at each sub-step boundary from completed board state, and after SS4 only after `ChainDeathBuffer` fully drains
- Forbidden: SILENCE does not strip INJURED condition itself (only strips keywords INJURED grants); do not re-snapshot LEADER bonus mid-RESOLUTION
- Performance: Persistent-state scans are bounded by live board units and must keep the full ADR-017 RESOLUTION batch within <= 15 ms.

---

## Acceptance Criteria

*From GDD `design/gdd/combat-resolution.md`, scoped to this story:*

- [ ] **CR-26**: GIVEN a unit takes damage in sub-step 3 that puts HP below maximum (activating INJURED), WHEN sub-step 3 completes, THEN the INJURED bonus is NOT active in sub-step 3; it IS active from sub-step 4 onward for this RESOLUTION (INJURED activates at the SS3→SS4 boundary). Since sub-step 4 has no attacks, INJURED-granted FIRST STRIKE is not exercised until sub-step 3 of the next round (see CR-34)
- [ ] **CR-33**: GIVEN a LEADER unit (grants +1 ATK to family units) is alive after SS1 completes and the post-SS1 snapshot is taken, WHEN that LEADER is killed in sub-step 4 of round N, THEN family units' ATK_effective includes the +1 LEADER bonus in SS5 and SS6; WHEN round N+1 post-SS1 LEADER snapshot is computed with LEADER still dead, THEN family units' ATK_effective equals ATK_base only (no LEADER term)
- [ ] **CR-34**: GIVEN a unit gains FIRST STRIKE via INJURED (activated at sub-step boundary after sub-step 3 damage), WHEN sub-step 3 of the NEXT round executes, THEN the unit attacks as a FIRST STRIKE unit (INJURED state persists between rounds)
- [ ] **KW-040 / OUTNUMBERED boundary**: GIVEN a DEATH trigger chain in SS4 changes player board counts, WHEN OUTNUMBERED is evaluated for SS5, THEN the count reflects the final board state after `ChainDeathBuffer` fully drains and all resulting removals are processed — not any intermediate count during the chain; `OutnumberedFlipped` is emitted only if the final boolean differs from the previous cached value

---

## Implementation Notes

*Derived from ADR-017, ADR-018, ADR-022, and the GDD Persistent Keyword States sections:*

**INJURED activation (CR-26)**:
```
At SS3 entry: unit.hp_at_ss3_entry = unit.hp  (snapshot before SS3 damage)
After SS3 completes (at SS3→SS4 boundary):
  for each surviving unit:
    if unit.hp < unit.hp_at_ss3_entry:  // damaged during SS3
      unit.is_injured = true
      // INJURED keywords (e.g., FIRST STRIKE granted by INJURED) now active
```
The `is_injured` flag is checked at SS3 entry of each subsequent sub-step (including next round). Do NOT flip `is_injured = true` during SS3 damage processing — only at the boundary.

**LEADER snapshot (CR-33)**:
```rust
// After SS1 fully resolves (post-SS1, before SS2):
for unit in snapshots.iter_mut() {
    // Check if a living, unsilenced LEADER of the same family exists on this unit's team
    unit.leader_atk_bonus = compute_leader_bonus(&snapshots, unit);
    // This value is FIXED for the entire RESOLUTION — never re-computed
}
```
A LEADER that enters during SS1 is included if it is alive after all SS1 APPEARANCE effects resolve. A LEADER killed during SS1 before this snapshot grants no bonus this round. When `apply_combat_modifier_stack` uses `attacker.leader_atk_bonus`, it reads this snapshotted value. Even if the LEADER is killed in SS4, the surviving units' `leader_atk_bonus` remains non-zero in their snapshots for this RESOLUTION. Next round: snapshot is rebuilt fresh post-SS1 — if the LEADER is still dead, bonus is 0.

**OUTNUMBERED evaluation** (not explicitly tested in these CRs but referenced in the GDD):
```
At each sub-step boundary:
  1. Complete all effects from the previous sub-step
  2. If leaving SS4, drain ChainDeathBuffer fully and process all removals
  3. Recompute OUTNUMBERED from the final post-sub-step board state
  4. Evaluate INJURED for all surviving units from completed sub-step
```
OUTNUMBERED is not updated during an in-progress DEATH chain. The SS5 value is computed once from the final board state after all SS4 deaths and chained deaths have resolved.

**CR-34 (INJURED persists across rounds)**: `is_injured` is stored on the server-side unit state, not just in `UnitSnapshot`. At the start of each RESOLUTION, the snapshot is built from server state — `is_injured` is read from the persisted unit data. A unit INJURED in round N has `is_injured = true` at round N+1 RESOLUTION entry → grants FIRST STRIKE for round N+1 SS3.

INJURED is NOT a keyword — SILENCE strips keywords granted BY INJURED (e.g., FIRST STRIKE), but SILENCE does not set `is_injured = false`.

---

## Out of Scope

- Story 005: SS3 FIRST STRIKE attacks (INJURED-granted FIRST STRIKE applies in the next round's SS3, not this story's implementation scope)
- Story 007: SHIELD and COUNTERATTACK persistent state mechanics (separate story)
- The full STUN passive defense (STUN suppresses outgoing actions but not SHIELD — verified in Story 004)
- Story 011: full `S2CResolutionEvent` log completeness and ordering. This story may produce state transitions, but complete log audit coverage remains COMBAT-011.

---

## QA Test Cases

*(Lean mode — test cases authored inline)*

- **CR-26** (INJURED activates at SS3→SS4 boundary):
  - Given: Unit with keyword "INJURED grants FIRST STRIKE"; HP at SS3 entry = 5; takes 2 damage in SS3 (HP = 3, below max)
  - When: SS3→SS4 boundary evaluation runs
  - Then: `unit.is_injured = true`; FIRST STRIKE keyword is now in unit's effective keyword set
  - And: FIRST STRIKE was NOT active during SS3 itself (verified by checking unit was not in the SS3 FIRST STRIKE collection)

- **CR-33** (LEADER snapshot persists after death):
  - Given: LEADER unit alive after SS1 of round N; family units had `leader_atk_bonus = 1` from the post-SS1 snapshot
  - When: SS5 and SS6 execute (round N)
  - Then: `apply_combat_modifier_stack` uses `leader_atk_bonus = 1` for family units (bonus persists this RESOLUTION)
  - When: Round N+1 post-SS1 snapshot runs; LEADER still dead
  - Then: `snapshot.leader_atk_bonus = 0` for family units (fresh snapshot, no living LEADER)

- **CR-34** (INJURED grants FIRST STRIKE next round):
  - Given: Unit INJURED in round N (is_injured persisted to server state)
  - When: Round N+1 RESOLUTION begins; SS3 collects FIRST STRIKE units
  - Then: INJURED unit is in the FIRST STRIKE collection (attacks in SS3 of round N+1)
  - Edge case: if SILENCE is applied before SS3 of round N+1 → FIRST STRIKE stripped (INJURED itself not stripped)

- **KW-040 / OUTNUMBERED post-SS4 boundary**:
  - Given: Player A is outnumbered before SS4, and a SS4 DEATH chain removes enough units to change the final count
  - When: `ChainDeathBuffer` drains completely and SS5 boundary evaluation runs
  - Then: OUTNUMBERED is computed from the final board counts only; no intermediate chain count is observable; `OutnumberedFlipped` appears only if the final boolean changed

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/combat/persistent_states_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 003 (SS1 placement/APPEARANCE complete before LEADER snapshot), Story 005 (SS3 damage activates INJURED), Story 006 (SS4 dead removal and `ChainDeathBuffer`), Story 007 (SS6 context for LEADER snapshot verification)
- Unlocks: Story 011 (log completeness can now include keyword trigger entries for persistent states)
