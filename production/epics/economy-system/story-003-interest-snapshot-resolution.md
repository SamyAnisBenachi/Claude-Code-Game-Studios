# Story 003: Interest Snapshot & Resolution End

> **Epic**: Economy System
> **Status**: Ready
> **Layer**: Core
> **Type**: Logic
> **Manifest Version**: 2026-04-29

## Context

**GDD**: `design/gdd/economy-system.md`
**Requirement**: TR-??? (covers TR-ECO-05: interest snapshot at RESOLUTION end; EC13: snapshot formula correct; EC14: max interest cap; EC18: mana discard at resolution end)

**ADR Governing Implementation**: ADR-010: RSM Phase Event Bus — Phase Message Catalog and Subscriber Contracts
**ADR Decision Summary**: Economy System subscribes to `ResolutionPhaseEntered { round }`. The snapshot system runs `.after(ObjectiveSystemSet::ProcessDestructions)` and `.after(CombatSystemSet::ProcessKills)` and `.before(ResolutionCompleteEmitter)`. This ordering guarantees the snapshot includes all kill and objective gold awards fired during RESOLUTION before the next DRAFT income is calculated. For M1 (no live Combat), the snapshot system accepts a synthetic `ResolutionPhaseEntered` event with manually configured gold state.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: MEDIUM
**Engine Notes**: Uses `MessageReader<ResolutionPhaseEntered>::read()` (Bevy 0.18 — `EventReader` no longer exists). System ordering labels `ObjectiveSystemSet` and `CombatSystemSet` are defined by their respective epics; for M1 the placeholder label `EconomySystemSet::ResolutionEnd` is used. `liv-bevy-018` mandatory.

**Control Manifest Rules (Core layer)**:
- Required: Snapshot is taken from `gold` at RESOLUTION END — after all kill/objective rewards have fired, before `ResolutionComplete` is emitted. This is the F4 contract.
- Required: `discard_current_mana_at_resolution_end` and `on_resolution_phase_entered` (snapshot write) run in the same system step — share the same `ResolutionPhaseEntered` event reader or run in the same `EconomySystemSet::ResolutionEnd` label.
- Required: Snapshot overwrites any prior entry for the player — each round produces exactly one snapshot; a stale snapshot from a prior round (e.g., due to a reconnect edge case) is replaced, not accumulated.
- Forbidden: Snapshot must never be computed from `gold + baseline + interest` — the snapshot is taken BEFORE the next DRAFT's income step; it reflects only what the player held at the end of RESOLUTION.

---

## Acceptance Criteria

- [ ] `server/src/core/economy/system.rs` contains `on_resolution_phase_entered` system:
  - Reads `MessageReader<ResolutionPhaseEntered>`, `Res<PlayerEconomies>`, `ResMut<InterestSnapshots>`, `Res<SessionConfig>`
  - For each player: writes `InterestSnapshots.0.insert(player, economy.gold)` — overwrites any prior value
  - System is labelled `EconomySystemSet::ResolutionEnd`
- [ ] `server/src/core/economy/system.rs` contains `discard_current_mana_at_resolution_end` system:
  - Reads `MessageReader<ResolutionPhaseEntered>`, `ResMut<PlayerEconomies>`, `Res<SessionConfig>`
  - For each player: calls `api::discard_current_mana(economy)` → `current_mana = 0`
  - Runs in the same `EconomySystemSet::ResolutionEnd` label as the snapshot system
- [ ] `EconomyPlugin` schedules both systems `.after(advance_phase)` — using the M1 placeholder label; adds a `// TODO M2: also order .after(ObjectiveSystemSet::ProcessDestructions).after(CombatSystemSet::ProcessKills).before(ResolutionCompleteEmitter)` comment
- [ ] **EC13**: GIVEN `gold = 8` at RESOLUTION entry, WHEN `on_resolution_phase_entered` fires, THEN `InterestSnapshots[player] = 8`
- [ ] **EC14**: GIVEN `gold = 10`, WHEN snapshot taken, THEN `InterestSnapshots[player] = 10`; and WHEN `on_draft_started` (Story 002) subsequently reads this snapshot, interest = 2 (maximum)
- [ ] **EC18**: GIVEN `current_mana = 4` at RESOLUTION entry, WHEN `discard_current_mana_at_resolution_end` fires, THEN `current_mana = 0`
- [ ] Snapshot overwrite: GIVEN `InterestSnapshots[player] = 3` (stale), WHEN `on_resolution_phase_entered` fires with `gold = 9`, THEN `InterestSnapshots[player] = 9` (stale entry replaced)
- [ ] Gold = 0 case: GIVEN `gold = 0`, WHEN snapshot taken, THEN `InterestSnapshots[player] = 0`; interest at next DRAFT = 0; player still receives baseline income
- [ ] Kill-reward cross-threshold case (documented test): GIVEN `gold = 9` at RESOLUTION start, kill award fires (+1g → `gold = 10`), THEN snapshot = 10, interest at next DRAFT = 2 (not 1 from the pre-kill value)
- [ ] `cargo check -p server` passes after adding these systems to `system.rs` and scheduling them in `plugin.rs`

---

## Implementation Notes

*Derived from EPIC.md §system.rs `on_resolution_phase_entered` and economy-system.md Rule 6, Edge Cases:*

**Snapshot ordering concern for M1:** Combat Resolution (kill awards) and Objective System (objective awards) are not implemented in M1. For M1 acceptance, the ordering contract is documented but enforced by placeholder. The M1 test injects gold manually before triggering `ResolutionPhaseEntered`, simulating post-award gold state. M2 will enforce the real ordering via system labels.

**Single `EventReader<ResolutionPhaseEntered>` or two?** If `on_resolution_phase_entered` and `discard_current_mana_at_resolution_end` are two separate systems, each needs its own `EventReader<ResolutionPhaseEntered>`. Bevy buffered events support multiple readers — each reader sees each event independently. Both systems are scheduled in the same label set so they run in the same frame and both see the event.

Alternatively, merge into one system `resolve_economy_at_resolution_end` that both takes the snapshot and discards mana in one pass. This is simpler for M1 — both operations happen atomically from the event. The merged approach is recommended unless a specific reason to separate arises.

**Interest formula reminder (implement in `on_draft_started`, not here):** The snapshot write stores raw gold. The interest computation (`min(floor(snap / threshold), max_bonus)`) happens in Story 002's `on_draft_started`. This story only stores the gold value.

**`discard_current_mana` does not enqueue `S2CGoldUpdate`**: Per the GDD, `current_mana` is discarded silently at RESOLUTION end. Clients are informed of the new `current_mana` value when `on_draft_started` sends the full `S2CGoldUpdate` at the next DRAFT entry. No intermediate broadcast is needed here.

---

## Out of Scope

- Story 002: Reading `InterestSnapshots` in `on_draft_started` and computing income — this story only writes the snapshot
- Story 004: Kill and objective awards that fire BEFORE the snapshot is taken (Combat and Objective epics)
- M2 ordering: `.after(ObjectiveSystemSet::ProcessDestructions)` and `.after(CombatSystemSet::ProcessKills)` — added when those epics define their system set labels

---

## QA Test Cases

*QL-STORY-READY skipped — Lean mode.*

- **EC13: Snapshot captures gold at resolution entry**
  - Given: `PlayerEconomies[player].gold = 8`; `InterestSnapshots` empty
  - When: `ResolutionPhaseEntered { round: 1 }` event written; snapshot system runs
  - Then: `InterestSnapshots.0[player] == 8`

- **EC18: Mana discarded at resolution end**
  - Given: `PlayerEconomies[player].current_mana = 4`
  - When: `ResolutionPhaseEntered` written; `discard_current_mana_at_resolution_end` runs
  - Then: `PlayerEconomies[player].current_mana == 0`

- **Kill-reward cross-threshold (integration with Story 004)**
  - Given: `gold = 9`; kill award manually applied (`gold = 10`) BEFORE snapshot system runs
  - When: Snapshot taken
  - Then: `InterestSnapshots[player] == 10`; when round 2 `DraftStarted` runs (via Story 002), interest = 2

- **Snapshot overwrite**
  - Given: `InterestSnapshots[player] = 99` (stale from test setup); `gold = 5`
  - When: Snapshot system runs
  - Then: `InterestSnapshots[player] == 5` (overwritten, not summed)

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/economy/interest_snapshot_test.rs` — test cases covering EC13, EC14, EC18, overwrite behaviour, gold = 0 case, and kill-cross-threshold scenario
**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 (API function `discard_current_mana` must exist)
- Depends on: Story 002 (`system.rs` file established; `EconomyPlugin` exists for scheduling)
- Depends on: RSM epic — `ResolutionPhaseEntered` event type defined in `server/core/rsm/events.rs`
- Unlocks: Story 002 integration test (round trace requires snapshot to be written at RESOLUTION end for rounds 2+); M2 Combat and Objective epics that fire awards before the snapshot
