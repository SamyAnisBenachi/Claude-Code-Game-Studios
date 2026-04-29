# Story 004: Win Condition and Game Over

> **Epic**: Round State Machine
> **Status**: Ready
> **Layer**: Core
> **Type**: Logic
> **Manifest Version**: 2026-04-29

## Context

**GDD**: `design/gdd/round-state-machine.md`
**Requirement**: TR-RSM-08
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-009 (RSM Phase State as ECS Resource), ADR-010 (RSM Phase Event Bus)
**ADR Decision Summary**: Win condition evaluation reads `Res<ObjectiveCounters>` — a forward-declared contract owned by the Objective System epic. The RSM does NOT import from `server/feature/`. `GameOverEmitted { reason, loser }` is emitted before `BroadcastPhaseChanged` on GAME_OVER entry. Draw occurs when multiple players simultaneously meet the loss condition. `is_auction_round` guard prevents `round_number == 0` at evaluation time.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**: `Res<ObjectiveCounters>` forward-declared — the type must be defined in `server/core/` (not `server/feature/`) or in `shared/` so the RSM can read it without importing feature code. `liv-bevy-018` skill mandatory on all files in this story.

**Control Manifest Rules (Core layer)**:
- Required: Win condition system reads `Res<ObjectiveCounters>` only — no direct import from `server/feature/`.
- Required: Win condition evaluated after `ResolutionComplete` is received and after the interest snapshot signal fires.
- Required: Single loser path: `loser = Some(player_id)`, `reason = GameOverReason::ObjectivesDestroyed`.
- Required: Mutual destruction (Draw) path: `loser = None`, `reason = GameOverReason::Draw`.
- Required: `is_auction_round` guard on `round_number` — assertion `round_number >= 1` before evaluation.
- Forbidden: Any import of `use server::feature::*` in `server/src/core/rsm/`.
- CI grep gate: `grep -r "use server::feature" server/src/core/rsm/` must return zero matches.

---

## Acceptance Criteria

- [ ] `server/src/core/` (or `shared/src/`) defines `ObjectiveCounters` resource with at minimum: `fn real_objectives_destroyed(&self, player: PlayerId) -> u32` — this is a forward-declared contract; the Objective System epic implements it, this epic only reads it
- [ ] `server/src/core/rsm/system.rs` (or `transitions.rs`) defines a win condition evaluation function called after `ResolutionComplete` is processed by `rsm_input_reader`
- [ ] Win condition evaluation: for each player, check `objective_counters.real_objectives_destroyed(player) >= 2`; if exactly one player meets the condition → `GameOverEmitted { reason: ObjectivesDestroyed, loser: Some(player_id) }`; if multiple players meet the condition simultaneously → `GameOverEmitted { reason: Draw, loser: None }`; if no player meets the condition → transition to next DRAFT phase (auction round check)
- [ ] `is_auction_round(round_number)` is called only when `round_number >= 1`; a debug assertion `debug_assert!(rsm.round_number >= 1)` is present at the call site (RSM-33)
- [ ] GAME_OVER transition via `advance_phase` match arm emits `GameOverEmitted` then `BroadcastPhaseChanged { phase: GameOver, timer_ms: 0 }` (F2 ordering preserved: GAME_OVER entry arm follows the same last-broadcast rule)
- [ ] `GameOverEmitted { reason: Draw, loser: None }` is emitted for mutual destruction — the Game Session System subscriber handles the Draw case correctly (verified by integration test)
- [ ] CI grep gate: `grep -r "use server::feature" server/src/core/rsm/` returns zero matches
- [ ] Integration test `tests/integration/rsm/rsm_f2_ordering_test.rs` passes: in a DRAFT entry transition, `BroadcastPhaseChanged` is observed strictly after `DraftStarted` and `ShopRefreshNeeded` (use order-recording mock subscribers or event sequence inspection)
- [ ] `tests/unit/rsm/rsm_win_condition_test.rs` passes all tests listed in the QA Test Cases section (RSM-20 through RSM-22, RSM-36)

---

## Implementation Notes

*Derived from ADR-009 and ADR-010:*

**`ObjectiveCounters` forward declaration:** The RSM needs to read this resource to evaluate the win condition, but the type is owned by the Objective System epic. Define the type stub in `server/src/core/objective_contract.rs` (or `shared/`) with the minimum interface the RSM needs:
```rust
/// Forward-declared contract owned by the Objective System epic.
/// The RSM reads this resource to evaluate the GAME_OVER condition.
/// The Objective System epic is responsible for populating and updating this resource.
#[derive(Resource, Default)]
pub struct ObjectiveCounters {
    pub destroyed_per_player: HashMap<PlayerId, u32>,
}
impl ObjectiveCounters {
    pub fn real_objectives_destroyed(&self, player: PlayerId) -> u32 {
        *self.destroyed_per_player.get(&player).unwrap_or(&0)
    }
}
```
The RSM epic defines the interface; the Objective System epic fills in the data. The RSM plugin inserts a default `ObjectiveCounters` resource so it exists at session start (zero-destroyed state).

**Win condition evaluation placement:** In `rsm_input_reader`, after processing `ResolutionComplete` (when `phase == Resolution`), immediately evaluate the win condition before transitioning. The sequence:
1. Receive `ResolutionComplete`
2. Check `phase == Resolution` (guard)
3. Read `Res<ObjectiveCounters>` — evaluate all players
4. Determine: next DRAFT, single GAME_OVER, or Draw GAME_OVER
5. Set `rsm.phase` to the appropriate next phase
6. Call `advance_phase` with the correct match arm

**Interest snapshot signal (Rule 4):** Per the GDD, the interest snapshot fires at RESOLUTION end, after combat and kill rewards but before `OnResolutionEnd`. The RSM signals this via `ResolutionPhaseEntered` → Economy System subscriber. The snapshot is taken by the Economy System, not the RSM. The RSM does not need to explicitly sequence this — the Economy System reads `ResolutionPhaseEntered` and takes its snapshot. The win condition check (Step 3 above) happens after `ResolutionComplete` arrives, which is emitted by Combat Resolution after all sub-steps complete (including any Economy System interest snapshot work that was triggered by `ResolutionPhaseEntered`). The ordering is preserved by system scheduling: Combat Resolution runs before `rsm_input_reader`.

**`S2CGameOver` message:** The RSM emits `GameOverEmitted`. The Game Session System (Epic 2) subscribes to `GameOverEmitted` and sends `S2CGameOver` over `ReliableChannel`. The RSM does not send Lightyear messages directly in this story — that is Story 006 (network dispatch) for `S2CPhaseChanged`, and Epic 2 for `S2CGameOver`.

**F2 ordering integration test approach:** Create mock subscriber systems that record the order events were processed. Assert that the sequence is: `DraftStarted` → `ShopRefreshNeeded ×2` → `BroadcastPhaseChanged`. Use Bevy's `World::run_system` or a test-only schedule to replay the full `advance_phase` + subscriber pipeline.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- Story 001-003: All prior RSM infrastructure
- Story 005: Disconnect-triggered GAME_OVER path
- Story 006: `BroadcastPhaseChanged` → `S2CPhaseChanged` network send
- Objective System epic: Actual `real_objectives_destroyed` computation — this story only reads the resource
- Game Session System (Epic 2): Session teardown on `GameOverEmitted`, `S2CGameOver` broadcast

---

## QA Test Cases

Each test uses `World::new()` + event injection. No live Lightyear session or Objective System implementation required.

**RSM-20 — Single loser via objectives:**
- Given: `RoundState { phase: Resolution, round_number: 5 }`; `ObjectiveCounters { destroyed_per_player: {player_a: 2, player_b: 0} }`; `ResolutionComplete` in queue
- When: `rsm_input_reader` processes `ResolutionComplete`; win condition evaluated
- Then: `GameOverEmitted { reason: ObjectivesDestroyed, loser: Some(player_a) }` written; `BroadcastPhaseChanged { phase: GameOver, timer_ms: 0 }` written after `GameOverEmitted`; `rsm.phase == GameOver`
- Edge cases: player_a destroyed = 1 (below threshold → no GAME_OVER); player_a destroyed = 3 (above threshold → same result); player_b destroyed = 2 (player_b is loser)

**RSM-21 — No win condition → next DRAFT:**
- Given: `RoundState { phase: Resolution, round_number: 2 }`; `ObjectiveCounters { destroyed_per_player: {player_a: 1, player_b: 0} }`; `ResolutionComplete` in queue
- When: win condition evaluated
- Then: no `GameOverEmitted` written; `rsm.phase != GameOver`; `round_number == 3` after increment; `BroadcastPhaseChanged { phase: DraftAuction }` written (3%3==0 → auction)
- Edge cases: 0 objectives destroyed for all (clear no-win); both at 1 (below threshold for all)

**RSM-22 — Mutual destruction → Draw:**
- Given: `ObjectiveCounters { destroyed_per_player: {player_a: 2, player_b: 2} }`; `ResolutionComplete` in queue
- When: win condition evaluated
- Then: `GameOverEmitted { reason: Draw, loser: None }` written; no single player declared loser; exactly one `GameOverEmitted` in queue (not two)
- Edge cases: player_a = 3, player_b = 2 → Draw (both qualify); player_a = 2, player_b = 1 → ObjectivesDestroyed (single loser)

**RSM-36 — reason field matches cause (parametric):**
- Scenario A (objectives): `ObjectiveCounters` loss condition met → `reason == ObjectivesDestroyed`; `loser == Some(id)`
- Scenario B (disconnect): `pending_disconnect_outcome` set (from Story 005) → `reason == Disconnection`; `loser == Some(id)`
- Scenario C (mutual): both players qualify → `reason == Draw`; `loser == None`
- Assert: `GameOverEmitted.reason` matches the scenario's cause exactly; no cross-scenario contamination

**Integration test (F2 ordering)** — `tests/integration/rsm/rsm_f2_ordering_test.rs`:
- **RSM-32 integration**: Run full `rsm_input_reader → advance_phase → [mock subscribers]` pipeline for a DRAFT entry transition; assert `DraftStarted` is processed before `ShopRefreshNeeded`, which is processed before `BroadcastPhaseChanged`; no subscriber's effect is visible before the prior step's effect

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: Automated unit tests — `tests/unit/rsm/rsm_win_condition_test.rs` and `tests/integration/rsm/rsm_f2_ordering_test.rs` must pass; paste `cargo test -p server rsm_win_condition rsm_f2_ordering` output into `tests/evidence/rsm-story-004-tests.md`
**Gate Level**: BLOCKING — all tests listed in QA Test Cases must pass before this story is Done
**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 003 (timers and input reader) must be Done — `rsm_input_reader` must exist before win condition evaluation can be added to it
- Unlocks: Story 005 (disconnect handling)
