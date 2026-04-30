# Story 008: LEADER Snapshot System

> **Epic**: Keyword System
> **Status**: Ready
> **Layer**: Feature (M3)
> **Type**: Logic
> **Manifest Version**: 2026-04-30

## Context

**GDD**: `design/gdd/keyword-system.md`
**Requirement**: TR-KW-??? (LEADER — untraced; no TR-ID registered)
*(Run `/architecture-review` to register this requirement in `docs/architecture/tr-registry.yaml`)*

**ADR Governing Implementation**: ADR-018 (Keyword System — ECS State Architecture, Part 5 LEADER Snapshot System)
**ADR Decision Summary**: `leader_snapshot_system` runs on `ResolutionPhaseEntered` (before SS1). It scans all LEADER units not silenced this round and writes `leader_bonus_atk`/`leader_bonus_hp` onto eligible family members' `UnitKeywordState`. LEADER bonus is snapshotted once per RESOLUTION — persists even if LEADER dies in SS4. Fields cleared at the start of the next round's snapshot run (overwritten to 0).

**BLOCKED**: ADR-018 Proposed. Story 001 must be Done. Story 006 (SILENCE system) must be Done (LEADER silence check requires `silenced_until_round` field to be readable).

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**: `leader_snapshot_system` uses `MessageWriter<KeywordTriggered>` — `app.add_message::<KeywordTriggered>()` must be registered in `KeywordPlugin` (Story 001). System runs on `ResolutionPhaseEntered` message, NOT as a standalone Update system — coordinate with RSM event bus (ADR-010).

**Control Manifest Rules (Feature layer)**:
- Required: `leader_snapshot_system` subscribes to `ResolutionPhaseEntered` event — never observe `RoundState` directly (ADR-010)
- Required: Every Feature system reacting to phase changes must subscribe to relevant RSM event (ADR-010)
- Forbidden: Never apply LEADER bonus during RESOLUTION after snapshot is taken — snapshot is fixed at entry; mid-RESOLUTION SILENCE of LEADER does NOT retroactively revoke it (KW-039)

---

## Acceptance Criteria

*From GDD `design/gdd/keyword-system.md` Acceptance Criteria:*

- [ ] KW-025: GIVEN a LEADER unit is alive at RESOLUTION entry (bonus snapshotted) and is killed in SS4, WHEN SS6 resolves, THEN the ATK bonus remains active for all eligible family units in SS6 (fields persist until RESOLUTION ends)
- [ ] KW-026: GIVEN a LEADER unit is SILENCEd at RESOLUTION entry, WHEN the bonus snapshot is computed, THEN the SILENCEd LEADER grants no bonus to family units for this RESOLUTION
- [ ] KW-039: GIVEN a LEADER is un-SILENCEd at RESOLUTION entry (bonus snapshotted), then SILENCEd during SS3, WHEN SS6 resolves, THEN the snapshot bonus remains active; mid-RESOLUTION SILENCE does NOT retroactively invalidate a legally-taken snapshot
- [ ] KW-046: GIVEN a LEADER unit with HASTE is placed in SS1 of round R, WHEN the LEADER bonus snapshot is computed at RESOLUTION entry for round R, THEN no bonus is applied this round (LEADER placed in SS1 was not on board at RESOLUTION entry); bonus applies from round R+1 onward
- [ ] KW-047: GIVEN Player A has two LEADER units of the same family (LEADER-1 placed in round R, LEADER-2 placed in round R+1) both alive at RESOLUTION entry of round R+1, WHEN the snapshot is computed, THEN eligible family units receive LEADER-1's bonus only; LEADER-2's bonus is suppressed (not summed)
- [ ] `KeywordTriggered { payload: LeaderSnapshotTaken { leader_unit_id }, sub_step: 0 }` emitted at RESOLUTION entry for each LEADER whose bonus is applied
- [ ] `leader_bonus_atk` and `leader_bonus_hp` on all eligible family members set to 0 at start of each new snapshot run (clearing previous round's values before writing new ones)

---

## Implementation Notes

*Derived from ADR-018 Part 5 and GDD LEADER rules:*

**leader_snapshot_system signature (ADR-018 Part 5):**
```rust
pub fn leader_snapshot_system(
    mut units: Query<(Entity, &CardId, &UnitBoardOwner, &UnitKeywordState)>,
    mut family_members: Query<(&CardId, &UnitBoardOwner, &mut UnitKeywordState)>,
    card_catalog: Res<CardCatalog>,
    current_round: Res<CurrentRound>,
    mut keyword_triggered: MessageWriter<KeywordTriggered>,
)
```

**Algorithm:**
1. Clear `leader_bonus_atk` and `leader_bonus_hp` to 0 on ALL units (reset previous round values)
2. Collect all LEADER units: `has_keyword(SimpleKeyword::Leader)` + alive at RESOLUTION entry (not in SS1 yet)
3. For each LEADER: check `silenced_until_round` — if silenced, skip (KW-026)
4. Apply LEADER stacking rule (KW-047): among LEADER units of the same family, only the earliest-placed grants its bonus; suppress later LEADERs of same family. "Earlier-placed" determined by placement timestamp within session (deterministic, observable)
5. For each eligible LEADER, find all alive family members owned by same player; write `leader_bonus_atk`/`leader_bonus_hp` to their `UnitKeywordState`
6. Emit `LeaderSnapshotTaken { leader_unit_id }` for each LEADER whose bonus was applied

**Snapshot semantics (KW-025):** Once written to `UnitKeywordState`, the bonus fields persist for the entire RESOLUTION even if the LEADER dies in SS4. No "LEADER alive" check during SS6 — the snapshot field drives everything.

**LEADER placed in SS1 (KW-046):** `leader_snapshot_system` runs BEFORE SS1. A LEADER entering in SS1 is not yet on the board at snapshot time → no bonus this round.

**Cross-family LEADERs (KW-047 note):** Two LEADERS of different families each grant their own bonuses independently. Only same-family LEADERS are subject to the stacking suppression rule.

---

## Out of Scope

- Story 006: SILENCE system (provides `silenced_until_round` field read by this story)
- Story 009: OUTNUMBERED (separate eval system)

---

## QA Test Cases

- **AC-1**: KW-025 — LEADER bonus persists after LEADER death in SS4
  - Given: LEADER alive at RESOLUTION entry; bonus snapshotted on 3 family members; LEADER killed in SS4
  - When: SS6 resolves for family members
  - Then: `leader_bonus_atk > 0` still present on all family members' UnitKeywordState; damage computed using buffed ATK
  - Edge cases: LEADER's own bonus fields (as a family member of itself) should also persist

- **AC-2**: KW-026 — SILENCEd LEADER at RESOLUTION entry grants no bonus
  - Given: LEADER unit has `silenced_until_round = Some(current_round)` at RESOLUTION entry
  - When: leader_snapshot_system runs
  - Then: `leader_bonus_atk = 0` on all family members; no `LeaderSnapshotTaken` emitted for this LEADER
  - Edge cases: other non-silenced LEADERs of different families still grant their bonuses

- **AC-3**: KW-039 — Mid-RESOLUTION SILENCE does not revoke snapshot
  - Given: LEADER un-silenced at RESOLUTION entry; bonus snapshotted; LEADER SILENCEd during SS3
  - When: SS6 resolves
  - Then: `leader_bonus_atk` still set on family members; LEADER's `silenced_until_round` is now set but snapshot was already taken
  - Edge cases: only RESOLUTION-entry SILENCE matters; mid-RESOLUTION SILENCE is irrelevant for snapshot

- **AC-4**: KW-046 — HASTE LEADER placed SS1 gets no snapshot this round
  - Given: LEADER unit with HASTE placed in SS1 of round R
  - When: leader_snapshot_system runs at RESOLUTION entry (before SS1)
  - Then: LEADER is not yet on board at snapshot time; `leader_bonus_atk = 0` on family members this round; bonus applies in round R+1 if LEADER survives
  - Edge cases: HASTE makes the unit participate this round in SS5/SS6 but snapshot already ran

- **AC-5**: KW-047 — Two same-family LEADERs; earlier-placed wins
  - Given: LEADER-1 (family A, placed round R) and LEADER-2 (family A, placed round R+1) both alive at RESOLUTION entry round R+1
  - When: leader_snapshot_system runs
  - Then: eligible family-A members receive LEADER-1's bonus value only (not sum); LEADER-2's bonus is suppressed; one `LeaderSnapshotTaken` emitted (LEADER-1)
  - Edge cases: if LEADER-1 dies between rounds and LEADER-2 is the only family-A LEADER at RESOLUTION entry, LEADER-2 grants its bonus normally

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/keyword/leader_snapshot_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 (scaffold), Story 006 (SILENCE — silenced_until_round readable)
- Unlocks: None directly (LEADER used by combat-resolution integration tests)
