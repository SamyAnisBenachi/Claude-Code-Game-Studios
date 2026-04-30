# Story 010: Persistent Keyword States — INJURED, LEADER, OUTNUMBERED

> **Epic**: Combat Resolution
> **Status**: Ready
> **Layer**: Feature
> **Type**: Logic
> **Manifest Version**: 2026-04-30

## Context

**GDD**: `design/gdd/combat-resolution.md`
**Requirement**: `TR-CR-???` (TR-CR-012 — unregistered)

**ADR Governing Implementation**: ADR-017: Combat Resolution Execution Architecture
**ADR Decision Summary**: Persistent keyword states (INJURED, LEADER snapshot, OUTNUMBERED) are evaluated at sub-step boundaries, not mid-sub-step. INJURED activates at the SS3→SS4 boundary and persists into future rounds (granting FIRST STRIKE in the next round's SS3). LEADER bonus is snapshotted at RESOLUTION entry and persists through the full RESOLUTION even if LEADER dies in SS4.

**Engine**: Bevy 0.18 | **Risk**: HIGH
**Engine Notes**: OUTNUMBERED board count is re-evaluated at each sub-step entry by querying live `BoardState` — not cached between sub-steps. LEADER bonus snapshot is stored in `UnitSnapshot.leader_atk_bonus` (captured at `resolve_combat` entry, before SS1). INJURED state persists as a field on the unit's server-side state — survives between rounds.

**Control Manifest Rules (Feature layer)**:
- Required: INJURED evaluated at SS3→SS4 boundary (not during SS3 damage); LEADER snapshot at RESOLUTION entry, NOT re-evaluated after LEADER death; OUTNUMBERED re-evaluated at each sub-step entry from post-removal board state
- Forbidden: SILENCE does not strip INJURED condition itself (only strips keywords INJURED grants); do not re-snapshot LEADER bonus mid-RESOLUTION

---

## Acceptance Criteria

*From GDD `design/gdd/combat-resolution.md`, scoped to this story:*

- [ ] **CR-26**: GIVEN a unit takes damage in sub-step 3 that puts HP below maximum (activating INJURED), WHEN sub-step 3 completes, THEN the INJURED bonus is NOT active in sub-step 3; it IS active from sub-step 4 onward for this RESOLUTION (INJURED activates at the SS3→SS4 boundary). Since sub-step 4 has no attacks, INJURED-granted FIRST STRIKE is not exercised until sub-step 3 of the next round (see CR-34)
- [ ] **CR-33**: GIVEN a LEADER unit (grants +1 ATK to family units) is killed in sub-step 4 of round N, WHEN round N sub-steps 5 and 6 execute, THEN family units' ATK_effective includes the +1 LEADER bonus (snapshotted at RESOLUTION entry); WHEN round N+1 RESOLUTION begins with LEADER still dead, THEN family units' ATK_effective equals ATK_base only (no LEADER term)
- [ ] **CR-34**: GIVEN a unit gains FIRST STRIKE via INJURED (activated at sub-step boundary after sub-step 3 damage), WHEN sub-step 3 of the NEXT round executes, THEN the unit attacks as a FIRST STRIKE unit (INJURED state persists between rounds)

---

## Implementation Notes

*Derived from ADR-017 and GDD Persistent Keyword States section:*

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
// At resolve_combat entry (before SS1):
for unit in snapshots.iter_mut() {
    // Check if a living LEADER of the same family exists on this unit's team
    unit.leader_atk_bonus = compute_leader_bonus(&snapshots, unit);
    // This value is FIXED for the entire RESOLUTION — never re-computed
}
```
When `apply_combat_modifier_stack` uses `attacker.leader_atk_bonus`, it reads this snapshotted value. Even if the LEADER is killed in SS4, the surviving units' `leader_atk_bonus` remains non-zero in their snapshots for this RESOLUTION. Next round: snapshot is rebuilt fresh — LEADER is dead, bonus is 0.

**OUTNUMBERED evaluation** (not explicitly tested in these CRs but referenced in the GDD):
```
At each sub-step boundary:
  1. Remove dead units (HP ≤ 0) — SS4 only
  2. Recompute OUTNUMBERED from post-removal board state
  3. Evaluate INJURED for all surviving units from completed sub-step
```

**CR-34 (INJURED persists across rounds)**: `is_injured` is stored on the server-side unit state, not just in `UnitSnapshot`. At the start of each RESOLUTION, the snapshot is built from server state — `is_injured` is read from the persisted unit data. A unit INJURED in round N has `is_injured = true` at round N+1 RESOLUTION entry → grants FIRST STRIKE for round N+1 SS3.

INJURED is NOT a keyword — SILENCE strips keywords granted BY INJURED (e.g., FIRST STRIKE), but SILENCE does not set `is_injured = false`.

---

## Out of Scope

- Story 005: SS3 FIRST STRIKE attacks (INJURED-granted FIRST STRIKE applies in the next round's SS3, not this story's implementation scope)
- Story 007: SHIELD and COUNTERATTACK persistent state mechanics (separate story)
- The full STUN passive defense (STUN suppresses outgoing actions but not SHIELD — verified in Story 004)

---

## QA Test Cases

*(Lean mode — test cases authored inline)*

- **CR-26** (INJURED activates at SS3→SS4 boundary):
  - Given: Unit with keyword "INJURED grants FIRST STRIKE"; HP at SS3 entry = 5; takes 2 damage in SS3 (HP = 3, below max)
  - When: SS3→SS4 boundary evaluation runs
  - Then: `unit.is_injured = true`; FIRST STRIKE keyword is now in unit's effective keyword set
  - And: FIRST STRIKE was NOT active during SS3 itself (verified by checking unit was not in the SS3 FIRST STRIKE collection)

- **CR-33** (LEADER snapshot persists after death):
  - Given: LEADER unit killed in SS4 of round N; family units had `leader_atk_bonus = 1` at RESOLUTION entry
  - When: SS5 and SS6 execute (round N)
  - Then: `apply_combat_modifier_stack` uses `leader_atk_bonus = 1` for family units (bonus persists this RESOLUTION)
  - When: Round N+1 RESOLUTION begins; LEADER still dead
  - Then: `snapshot.leader_atk_bonus = 0` for family units (fresh snapshot, no living LEADER)

- **CR-34** (INJURED grants FIRST STRIKE next round):
  - Given: Unit INJURED in round N (is_injured persisted to server state)
  - When: Round N+1 RESOLUTION begins; SS3 collects FIRST STRIKE units
  - Then: INJURED unit is in the FIRST STRIKE collection (attacks in SS3 of round N+1)
  - Edge case: if SILENCE is applied before SS3 of round N+1 → FIRST STRIKE stripped (INJURED itself not stripped)

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/combat/persistent_states_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 005 (SS3 damage activates INJURED), Story 007 (SS6 context for LEADER snapshot verification)
- Unlocks: Story 011 (log completeness can now include keyword trigger entries for persistent states)
