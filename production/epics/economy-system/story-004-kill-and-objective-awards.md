# Story 004: Kill and Objective Awards

> **Epic**: Economy System
> **Status**: Conditional backlog
> **Layer**: Core / Feature Integration
> **Type**: Integration
> **Manifest Version**: 2026-05-08

## Context

**GDD**: `design/gdd/economy-system.md`

**Requirement**: `TR-ECO-006`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` - read fresh at review time.)*

**Adjacent Traceability**:
- `TR-CR-008`: Combat Resolution awards kill gold through `resolve_combat` and ensures kill/objective rewards land before the economy interest snapshot.
- `TR-OBJ-004`: Objective destruction marks the slot destroyed, queues `ObjectiveDestroyed`, awards gold only when attacker and owner differ, and dispatches fake-objective rewards.
- `TR-OBJ-005`: Fake-objective D4 reward emits `ManaCapIncreased` or resolves `FreeCardPick`; hand-full fallback emits `AwardGold +1`.
- `TR-CR-015`: `S2CResolutionEvent` includes `GoldAwarded` entries in chronological order.

**GDD Requirement Summary**:
- EC11: self-inflicted objective destruction does not award objective gold.
- EC16: a unit kill immediately grants the killer `GameConfig.kill_gold_reward`, default 1.
- EC17: destroying an opponent objective immediately grants the attacker `GameConfig.objective_gold_reward`, default 3.
- EC9 adjacent scope: a fake-objective mana-cap reward increments the attacker's `mana_cap`, capped by `GameConfig.mana_cap_max`, and the increased cap applies at the next DRAFT mana ramp.

**Sprint 9 Conditionality**: ECO-004 sits in the Sprint 9 Conditional Backlog. Do not start `/dev-story` for this story unless Sprint 9 evidence (from S9-QA-001 manual route capture or equivalent observation) shows a concrete reward-loop gameplay issue. Pull condition mirrors sprint-9.md: "Pull only if Sprint 9 evidence shows a concrete reward-loop gameplay issue." No expansion into broad economy tuning, auction behavior, shop refresh, or placement mana-split work is permitted even if this story is pulled.

**ADR Governing Implementation**: ADR-010: RSM Phase Event Bus; ADR-017: Combat Resolution Execution Architecture; ADR-019: Economy Resource Architecture; ADR-005: Server-side RNG.

**ADR Decision Summary**: Current architecture uses no standalone kill-award production message and does not carry attacker or reward-selection fields on `ObjectiveDestroyed`. Kill gold is awarded inside `resolve_combat` from `CombatKillLog` via `economy_api::apply_gold_award`. Objective gold is awarded from the combat/objective consequence path when an `ObjectiveDestroyed { target_player_id, lane, was_fake }` event is observed for an opponent-owned objective. Objective fake reward logic emits Bevy `Message`s: `ManaCapIncreased { player, amount }` and, for some free-card fallback cases, `AwardGold { player, amount }`. Any new economy consumer for these messages must be ordered before the interest snapshot and must not double-award paths that already mutate `PlayerEconomies` directly inside `resolve_combat`.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: MEDIUM

**Engine Notes**:
- Use Bevy buffered messages with `#[derive(Message)]`, `MessageReader<T>`, `MessageWriter<T>`, and `app.add_message::<T>()`.
- Do not use `EventReader`, `EventWriter`, `Events<T>`, or `app.add_event`.
- Regular systems that consume `AwardGold` or `ManaCapIncreased` must use `MessageReader<T>::read()`.
- Existing exclusive `World` paths may interact with `Messages<T>` only where current Bevy 0.18 code already establishes that pattern; do not introduce old Bevy event APIs.
- `ObjectiveDestroyed` production payload is `ObjectiveDestroyed { target_player_id, lane, was_fake }`.
- `liv-bevy-018` is mandatory for every Bevy `.rs` file touched by a later implementation.

**Control Manifest Rules (Core / Feature layers)**:
- Required: All `PlayerEconomy` field mutations go through `server/src/core/economy/api.rs`.
- Required: Kill and objective gold awards land before `on_resolution_complete` captures `InterestSnapshots`.
- Required: `on_resolution_complete` reads `MessageReader<ResolutionComplete>`, not `ResolutionPhaseEntered`.
- Required: `resolve_combat` is the exclusive in-RESOLUTION writer for direct kill/objective gold awards.
- Required: If a later implementation adds `AwardGold` or `ManaCapIncreased` consumers in Economy, schedule them so they run after combat/objective message emission and before `on_resolution_complete` / `rsm_input_reader`.
- Forbidden: Do not add a second award path that can grant gold for the same objective destruction.
- Forbidden: Economy must not perform fake-objective RNG. Objective System owns the D4 reward draw per ADR-005.
- Forbidden: Do not add a standalone kill-award production message or attacker/reward-selection fields to `ObjectiveDestroyed` for this story.

---

## Acceptance Criteria

- [ ] **Sprint 9 conditional gate is preserved**: Given this story is in the Sprint 9 Conditional Backlog, when implementation is considered, then the worker confirms Sprint 9 evidence shows a concrete reward-loop gameplay issue per the pull condition in sprint-9.md, and no Sprint 9 Must Have story status is changed by this story.

- [ ] **Stale event contract is absent**: Given the implementation plan is reviewed, when reward events are named, then it uses current production contracts only: `CombatKillLog`, `AwardGold`, `ManaCapIncreased`, and `ObjectiveDestroyed { target_player_id, lane, was_fake }`. No standalone kill-award production message and no attacker, defender, or reward-selection fields are added to `ObjectiveDestroyed`.

- [ ] **EC16 / TR-ECO-006 kill gold**: Given a unit is killed during combat, when `resolve_combat` drains the matching kill record, then exactly one `api::apply_gold_award(killer_economy, config.kill_gold_reward)` equivalent is applied for that kill, a `GoldAwarded { player, amount, reason: Kill }` resolution-log entry is produced, and the killed player's gold is unchanged.

- [ ] **Dual kill accumulation**: Given two kill records for the same killer are drained in one RESOLUTION, when kill gold is applied, then the killer gains exactly `2 * config.kill_gold_reward` and receives two chronological `GoldAwarded` resolution-log entries or one explicitly documented equivalent that preserves both reward facts.

- [ ] **EC17 / TR-ECO-006 objective gold**: Given an opponent objective is destroyed, when the consequence path completes, then exactly one objective gold award of `config.objective_gold_reward` is applied to the attacker, and a `GoldAwarded { player, amount, reason: ObjectiveDestroyed }` resolution-log entry is produced.

- [ ] **EC11 / TR-ECO-006 self-inflicted guard**: Given `attacker_player == target_player_id`, when the objective destruction consequence path runs, then no objective gold is awarded, no `AwardGold` consumer or direct combat path can add gold for that event, and objective loss counters remain owned by Objective/RSM logic.

- [ ] **No duplicate objective reward**: Given both Objective System `AwardGold` messages and combat-side direct objective award paths exist in current code, when the story is implemented, then each objective destruction can mutate `PlayerEconomies` at most once. If a new `AwardGold` consumer is added, the direct path is guarded, removed, or otherwise proven not to double-award by tests.

- [ ] **Fake mana-cap reward applies once**: Given `ManaCapIncreased { player, amount }` is emitted for a fake-objective reward, when the reward-loop polish runs, then Economy applies `api::increment_mana_cap` once per message amount up to `GameConfig.mana_cap_max`, and the updated `mana_cap` is visible to the next `DraftStarted` mana ramp.

- [ ] **Fake free-card fallback gold applies once**: Given a fake reward resolves to the hand-full `AwardGold +1` fallback, when the reward-loop polish runs, then that fallback gold is applied exactly once and does not reuse `objective_gold_reward`.

- [ ] **Client reward visibility remains coherent**: Given kill gold, objective gold, fake fallback gold, or mana-cap increase changes `PlayerEconomies`, when the reward is applied, then the implementation either enqueues the necessary `S2CGoldUpdate` / `S2CGoldBroadcast` messages for current client visibility or documents and tests that the existing `S2CResolutionEvent::GoldAwarded` plus next DRAFT economy update is the intended friend-game-visible path.

- [ ] **Interest snapshot includes all rewards**: Given kill/objective/fake fallback rewards are applied during RESOLUTION, when `on_resolution_complete` captures `InterestSnapshots`, then the snapshot includes those rewards before next DRAFT income is calculated.

- [ ] **Bevy 0.18 API check**: Given touched Rust files are reviewed, when searching the affected files, then no `EventReader`, `EventWriter`, `Events<`, or `add_event` usage is introduced.

- [ ] `cargo check -p server` passes.

---

## Implementation Notes

The story is a reward-loop reconciliation slice, not a request to recreate the old M1 event plan.

Current production facts to preserve:
- `server/src/feature/objective/state.rs` defines `ObjectiveDestroyed { target_player_id, lane, was_fake }` as a Bevy `Message`.
- `server/src/core/economy/system.rs` defines `AwardGold`, `ManaCapIncreased`, `S2CGoldUpdate`, and `S2CGoldBroadcast` as Bevy `Message`s.
- `server/src/feature/combat/mod.rs` drains `CombatKillLog` and directly mutates `PlayerEconomies` through `economy_api::apply_gold_award` for kill gold.
- `server/src/feature/combat/mod.rs` also awards objective gold from the combat/objective consequence path after observing a newly queued `ObjectiveDestroyed`.
- `server/src/feature/objective/system.rs` emits `AwardGold` and `ManaCapIncreased` messages from objective fake-reward paths.

Implementation choice for a later `/dev-story`:
- If Sprint 9 evidence shows the direct combat paths already satisfy friend-game reward visibility, keep the implementation narrow and add tests/documentation only for the observed gap.
- If `AwardGold` or `ManaCapIncreased` messages are unconsumed in the observed path, add Economy-side consumers only with ordering and duplicate-award guards.
- If direct combat award paths remain authoritative, do not also consume the same objective gold `AwardGold` message unless the direct award path is guarded or removed.

Suggested scheduling if message consumers are needed:
- Place reward consumers in a named Economy system set before `EconomySystemSet::ResolutionEnd` and before `rsm_input_reader`.
- Keep `on_resolution_complete` as the final economy resolution-end snapshot/discard step.
- Keep `DraftStarted` handling unchanged except for reading any already-mutated `mana_cap`.

Suggested test shape:
- Prefer World/App tests with real Bevy messages, not pure mocks of Bevy system params.
- Test duplicate prevention with both a direct objective award path and an `AwardGold` message present.
- Test `ManaCapIncreased` below cap and at cap.
- Test snapshot timing by applying rewards before `ResolutionComplete` and asserting the next DRAFT interest uses the rewarded gold.

---

## Out of Scope

- No implementation starts as part of this readiness prep.
- No `/dev-story`, `/story-done`, sprint-status, session-state, QA-COND-0005, or QA-COND-0006 edits.
- No standalone kill-award production message.
- No change to `ObjectiveDestroyed` payload shape.
- No Objective System RNG changes for fake reward selection.
- No broad economy balancing, tuning, auction behavior, shop refresh behavior, or placement mana-split work.
- No public release readiness, broad accessibility completion, playtest validation, or full playable-client QA claim.

---

## QA Test Cases

- **Kill gold**
  - Given: Player A kills Player B's unit during RESOLUTION.
  - When: the kill record is drained.
  - Then: Player A gains `kill_gold_reward`; Player B is unchanged; one kill `GoldAwarded` entry exists.

- **Dual kill**
  - Given: two kill records both credit Player A.
  - When: kill rewards are drained.
  - Then: Player A gains exactly two kill rewards and both reward facts are visible in the resolution log.

- **Objective gold**
  - Given: Player A destroys Player B's objective.
  - When: the destruction consequence path completes.
  - Then: Player A gains exactly `objective_gold_reward`; the resolution log includes one objective-gold entry.

- **Self-inflicted objective**
  - Given: Player A destroys Player A's own objective.
  - When: the destruction consequence path completes.
  - Then: Player A gains no objective gold and no fallback award for that destruction.

- **Mana-cap fake reward**
  - Given: `ManaCapIncreased { player: A, amount: 1 }` is emitted while A is below `mana_cap_max`.
  - When: the reward-loop polish processes the message.
  - Then: A's `mana_cap` increases by one and the next DRAFT mana ramp reads the increased cap.

- **Duplicate guard**
  - Given: a direct objective award path and an `AwardGold` message are both present for the same destruction.
  - When: reward processing completes.
  - Then: `PlayerEconomies[A].gold` increases only once for that destruction.

---

## Test Evidence

**Required evidence**:
- Integration or focused server test: `tests/integration/economy/reward_loop_awards_test.rs`
- Existing adjacent regressions to keep passing:
  - `tests/unit/objective/consequence_path_test.rs`
  - `tests/unit/objective/fake_reward_test.rs`
  - `tests/unit/combat/substep4_dead_removal_test.rs`
  - `tests/unit/combat/objective_damage_gameover_test.rs`
  - `tests/integration/combat/resolution_event_log_test.rs`
  - `server/tests/economy_interest_snapshot_test.rs`

**Status**: [ ] Not yet created for this repaired story scope.

---

## Dependencies

- Depends on: Economy Story 001 (Complete) for `PlayerEconomies` and API functions.
- Depends on: Economy Story 002 (Complete) for `S2CGoldUpdate` / `S2CGoldBroadcast` message types and draft update behavior.
- Depends on: Economy Story 003 (Complete) for `ResolutionComplete`-based interest snapshot timing.
- Depends on: Combat Resolution Story 006 (Complete) for kill-gold direct path and `GoldAwarded { reason: Kill }`.
- Depends on: Combat Resolution Story 009 (Complete) for objective damage / GAME_OVER path and objective-gold integration.
- Depends on: Objective System Story 005 (Complete) for destruction consequence and self-inflicted guard.
- Depends on: Objective System Story 006 (Complete) for fake reward `ManaCapIncreased` / fallback `AwardGold` emission.
- Depends on: Objective System Story 007 (Complete) for RESOLUTION-end `ObjectiveDestroyed` broadcast timing.
- Sprint 9 conditional pull: pull only if Sprint 9 evidence (S9-QA-001 route capture or equivalent) shows a concrete reward-loop gameplay issue; no broad economy tuning expansion permitted.
- Unlocks: later reward-loop polish only; does not unlock broad economy scope.

---

## Performance Budget

No broad performance impact is expected. Reward processing must be O(number of reward messages plus kill records in the RESOLUTION frame), with no continuous per-frame polling and no client asset work. Server RESOLUTION work must remain within the existing 15 ms RESOLUTION budget from the control manifest.
