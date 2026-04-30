# Story 002: Advance Phase and F2 Ordering

> **Epic**: Round State Machine
> **Status**: Complete
> **Layer**: Core
> **Type**: Logic
> **Manifest Version**: 2026-04-29

## Context

**GDD**: `design/gdd/round-state-machine.md`
**Requirement**: TR-RSM-01, TR-RSM-02, TR-RSM-04, TR-RSM-05, TR-RSM-06
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-009 (RSM Phase State as ECS Resource), ADR-010 (RSM Phase Event Bus)
**ADR Decision Summary**: `advance_phase` is the SOLE writer of `ResMut<RoundState>`. All 7 phase match arms enforce F2 emission ordering by linear `MessageWriter::write()` call order within the arm — not by Bevy system scheduling. `BroadcastPhaseChanged` is always the last `.write()` call in every match arm. A double-transition guard (`if rsm.phase != expected_source_phase { return; }`) runs at the top of `advance_phase`. `round_number` increments on RESOLUTION exit before economy messages fire.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**: `MessageWriter::write()` is the correct API — `EventWriter`/`EventReader` do NOT exist in Bevy 0.17+. `ResMut<RoundState>` must appear in exactly one system (`advance_phase`) across the entire `server/src/` codebase — CI grep gate enforces this. `liv-bevy-018` skill mandatory on all files in this story.

**Control Manifest Rules (Core layer)**:
- Required: `advance_phase` is the sole system taking `ResMut<RoundState>`. All other systems take `Res<RoundState>` (read-only).
- Required: Inside each match arm, `MessageWriter<BroadcastPhaseChanged>.write()` is the final call — no message writes after it.
- Required: F2 emission order within DRAFT entry arms: `DraftStarted` → `ShopRefreshNeeded` (×N players) → `AuctionPhaseEntered` (if applicable) → `BroadcastPhaseChanged`.
- Required: `round_number` increments on RESOLUTION → DRAFT_* transition, before `DraftStarted` is written, so economy systems receive the already-incremented value.
- Required: Double-transition guard at entry: `if rsm.phase != expected_source { return; }`.
- Forbidden: `ResMut<RoundState>` in any system other than `advance_phase`.
- Forbidden: Any `EventWriter`, `EventReader`, or `Events<T>` usage in `server/src/core/rsm/`.
- CI grep gate: `grep -r "ResMut<RoundState>" server/src/ | grep -v transitions.rs` must return zero matches.

---

## Acceptance Criteria

- [ ] `server/src/core/rsm/transitions.rs` defines `advance_phase` as a Bevy system taking `ResMut<RoundState>` and `MessageWriter<T>` params for all 7 outbound message types
- [ ] `advance_phase` contains exactly 7 match arms covering all source phases: `Lobby`, `DraftInitial`, `DraftAuction`, `DraftShop`, `Placement`, `Resolution`, `GameOver`
- [ ] Each match arm begins with a double-transition guard: `if rsm.phase != [expected_source] { return; }` — the second of two simultaneous triggers finds the phase already changed and silently no-ops (RSM-31, RSM-34)
- [ ] `GameOver` match arm is a no-op terminal: sets nothing, emits nothing, returns immediately
- [ ] DRAFT entry arms (from `DraftInitial`, `DraftAuction` → `DraftShop`, `Resolution` → `DraftAuction` or `DraftShop`) emit events in strict F2 order: (1) `DraftStarted`, (2) `ShopRefreshNeeded` once per player, (3) `AuctionPhaseEntered` if and only if entering `DraftAuction`, (4) `BroadcastPhaseChanged` last
- [ ] `PLACEMENT` entry arm emits: (1) `PlacementPhaseEntered`, (2) `BroadcastPhaseChanged` last; resets `submissions_received` to empty set
- [ ] `RESOLUTION` entry arm emits: (1) `ResolutionPhaseEntered`, (2) `BroadcastPhaseChanged` last
- [ ] `GAME_OVER` entry arm emits: (1) `GameOverEmitted { reason, loser }`, (2) `BroadcastPhaseChanged { timer_ms: 0 }` last
- [ ] `round_number` increments on any RESOLUTION → DRAFT_* transition, before `DraftStarted` is emitted; `DraftStarted.round` carries the incremented value (RSM-2, RSM-33)
- [ ] `round_number` is set to 1 on LOBBY → `DraftInitial` transition (handled by `on_session_ready` in Story 003 — `advance_phase` match arm for `DraftInitial` entry assumes `round_number` is already 1)
- [ ] `is_auction_round(R)` helper function defined: returns `true` if `R % 3 == 0`; RESOLUTION exit checks this to determine whether next DRAFT phase is `DraftAuction` or `DraftShop` (RSM-33: `round_number == 0` is unreachable at any `is_auction_round` call site because `round_number` is set to 1 before first DRAFT_INITIAL entry)
- [ ] `BroadcastPhaseChanged` emitted for DRAFT_AUCTION has `timer_ms = 0` (Auction System drives its own countdown; RSM does not own DRAFT_AUCTION timer)
- [ ] `BroadcastPhaseChanged` for GAME_OVER and LOBBY phases has `timer_ms = 0`
- [ ] CI grep gate: `grep -r "ResMut<RoundState>" server/src/ | grep -v transitions.rs` returns zero matches
- [ ] CI grep gate: `grep -rE "EventWriter|EventReader|Events<|add_event" server/src/core/rsm/` returns zero matches
- [ ] `tests/unit/rsm/rsm_transitions_test.rs` passes all tests listed in the QA Test Cases section (RSM-1 through RSM-12, RSM-31, RSM-32, RSM-33)

---

## Implementation Notes

*Derived from ADR-009 and ADR-010:*

**F2 emission ordering is enforced by code order, not system scheduling:** All messages within a single `advance_phase` call are written before any subscriber system runs. The subscriber systems are scheduled `.after(advance_phase)` in `RsmPlugin`. The ordering guarantee within a transition is: the code order of `MessageWriter::write()` calls in the match arm is the canonical ordering. Do not use Bevy system ordering to enforce F2 steps — that approach was rejected (ADR-010 Alternative 1).

**`advance_phase` system signature example:**
```rust
pub fn advance_phase(
    mut rsm: ResMut<RoundState>,
    mut draft_started: MessageWriter<DraftStarted>,
    mut shop_refresh: MessageWriter<ShopRefreshNeeded>,
    mut auction_entered: MessageWriter<AuctionPhaseEntered>,
    mut placement_entered: MessageWriter<PlacementPhaseEntered>,
    mut resolution_entered: MessageWriter<ResolutionPhaseEntered>,
    mut game_over_emitted: MessageWriter<GameOverEmitted>,
    mut broadcast: MessageWriter<BroadcastPhaseChanged>,
    config: Res<GameConfig>,
    // Player list needed for per-player ShopRefreshNeeded fan-out:
    session: Res<SessionConfig>,
)
// TODO(liv-bevy-018): verify MessageWriter<T> is the correct system param name
// in Bevy 0.18. See SKILL.md — do NOT use EventWriter<T> (removed in 0.17).
```

**Per-player fan-out for `ShopRefreshNeeded`:** In DRAFT entry arms, emit one `ShopRefreshNeeded` per player. The player list comes from `Res<SessionConfig>`. In a 1v1 game, exactly 2 `ShopRefreshNeeded` events are written. The Card Pool reads N events and draws N independent shops.

**`advance_phase` is NOT a timer-driven system:** It does not tick timers itself. It is called by `rsm_input_reader` (Story 003) when a condition is met (timer expired, all submissions received, `AuctionSettled` received, etc.). Story 002 only implements the match arm logic — the calling mechanism is Story 003.

**Double-transition guard implementation:**
```rust
pub fn advance_phase(
    mut rsm: ResMut<RoundState>,
    // ... MessageWriter params ...
    trigger: Local<RsmTrigger>,  // or passed as param from rsm_input_reader
) {
    let expected_source = trigger.expected_source;
    if rsm.phase != expected_source {
        return; // Already transitioned this tick — silent no-op
    }
    match rsm.phase {
        RoundPhase::Lobby => { /* LOBBY → DraftInitial */ }
        // ... etc
    }
}
```
The exact mechanism for passing the trigger's expected source phase to `advance_phase` is an implementation detail — a `Local<T>` resource, a system parameter, or an event. The guard invariant (check phase before acting) is the binding requirement.

**RSM-33 invariant (`round_number == 0` unreachable at `is_auction_round`):** `round_number` is initialised to 0 in `RoundState::new()`. It is set to 1 inside `on_session_ready` (Story 003) before the first call to `advance_phase` for the `DraftInitial` entry. The RESOLUTION → DRAFT_* arm increments `round_number` before calling `is_auction_round`. Therefore, `is_auction_round` is never called with `round_number == 0`. The `round_number == 0` state is a pre-session sentinel only.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- Story 001: `RoundState`, `RoundPhase`, all event type definitions — must be Done before this story begins

**Deferred GDD ACs (Economy System territory — do NOT test here):**
- **GDD RSM-7** (mana formula): `current_mana = min(round_number, mana_cap)` — this is an Economy System formula applied when it reads `DraftStarted`. The RSM obligation tested here is only that `DraftStarted { round: N, phase: DraftPhase::Shop }` is emitted with the correct `round` field.
- **GDD RSM-9** (interest formula): `gold += baseline + interest` calculation — Economy System territory. RSM obligation: `DraftStarted` is emitted; Economy reads it and applies interest. Not asserted in RSM tests.
- **GDD RSM-13** (purchase accepted when gold ≥ cost) — Economy System + C2S handler. RSM phase-gate only: `phase == DraftShop` is the correct gate for purchases.
- **GDD RSM-14** (purchase rejected when gold < cost) — Economy System + C2S handler. RSM obligation: phase guard rejects purchases outside `DraftShop`. The gold check belongs to Epic 3.

These ACs are tracked in the Economy System epic story readiness review (Epic 3 / `production/epics/economy-system/`).
- Story 003: Timer tick systems, `rsm_input_reader`, `on_session_ready` — the mechanism that calls `advance_phase`
- Story 004: Win condition check — reads `Res<ObjectiveCounters>` at RESOLUTION end to decide GAME_OVER vs next DRAFT
- Story 005: Disconnect tracking systems
- Story 006: Network dispatch wiring

---

## QA Test Cases

Each test uses `World::new()` + event injection. No live Lightyear session required.

**RSM-2 — round_number increments before DraftStarted:**
- Given: `RoundState { phase: Resolution, round_number: 1 }`; `ResolutionComplete` in message queue; no loss condition in `ObjectiveCounters`
- When: `rsm_input_reader` runs, then `advance_phase` runs
- Then: `DraftStarted.round == 2`; `rsm.round_number == 2`; `BroadcastPhaseChanged { phase: DraftShop }` written (round 2: 2%3≠0)
- Edge cases: round_number = 2 → DraftStarted.round == 3; assert DraftStarted.round == rsm.round_number (must match)

**RSM-3 / RSM-4 — is_auction_round correctness (pure function):**
- Given: Pure function test, no World needed
- When: `is_auction_round(R)` called for R in {1, 2, 3, 4, 5, 6, 7, 8, 9}
- Then: returns true for {3, 6, 9}; returns false for {1, 2, 4, 5, 7, 8}
- Edge cases: R=12 → true; R=0 → unreachable (debug_assert guard fires in debug builds)

**RSM-5 / RSM-9 / RSM-10 — RESOLUTION → DRAFT routing:**
- DRAFT_AUCTION: Given `round_number=2` (increments to 3, 3%3==0) → `DraftAuction`; `AuctionPhaseEntered { round: 3 }` written; `BroadcastPhaseChanged { timer_ms: 0 }` last
- DRAFT_SHOP: Given `round_number=3` (increments to 4, 4%3≠0) → `DraftShop`; no `AuctionPhaseEntered` written

**RSM-6 / RSM-7 / RSM-8 — F2 write order for DRAFT entry arms (all use order-recording stub writers):**
- DRAFT_INITIAL: write log = [DraftStarted {phase:Initial, round:1}, ShopRefreshNeeded(p1), ShopRefreshNeeded(p2), BroadcastPhaseChanged]; BroadcastPhaseChanged is index 3 (last)
- DRAFT_SHOP: write log = [DraftStarted {phase:Shop}, ShopRefreshNeeded(p1), ShopRefreshNeeded(p2), BroadcastPhaseChanged]
- DRAFT_AUCTION: write log = [DraftStarted {phase:Auction}, ShopRefreshNeeded(p1), ShopRefreshNeeded(p2), AuctionPhaseEntered, BroadcastPhaseChanged]
- In each case: BroadcastPhaseChanged index > all other indices

**RSM-11 — per-player ShopRefreshNeeded fan-out in 1v1:**
- Given: 2-player session; advance_phase enters any DRAFT phase
- Then: exactly 2 `ShopRefreshNeeded` events written; each carries a distinct `player` field; no event carries the same player_id twice

**RSM-12 — PLACEMENT → RESOLUTION (timer or all-submit):**
- Timer path: inject `placement_timer.just_finished() = true`; assert `rsm.phase == Resolution`
- All-submit path (see Story 003 for submission tracking)

**RSM-31 — Double-transition guard:**
- Given: `RoundState { phase: Placement }`; advance_phase called twice in same tick
- When: first call → `rsm.phase = Resolution`; second call runs
- Then: second call finds `phase ≠ Placement` → early return; exactly 1 `BroadcastPhaseChanged` in queue
- Edge cases: two simultaneous timer+submit triggers — same result

**RSM-32 — F2 ordering invariant (integration):**
- Given: full `rsm_input_reader → advance_phase → mock-subscribers` pipeline; DRAFT entry transition
- Then: position(DraftStarted) < position(ShopRefreshNeeded[*]) < position(BroadcastPhaseChanged); no subscriber effect visible before its triggering event

**RSM-33 — debug_assert prevents is_auction_round(0):**
- Given: `RoundState { round_number: 0 }` (pre-session sentinel, should never reach is_auction_round)
- Then: In debug builds, `debug_assert!(rsm.round_number >= 1)` panics before `is_auction_round` is reached; no normal game flow path reaches this state

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: Automated unit tests — `tests/unit/rsm/rsm_transitions_test.rs` must pass; paste `cargo test -p server rsm_transitions` output into `tests/evidence/rsm-story-002-tests.md`
**Gate Level**: BLOCKING — all tests listed in QA Test Cases must pass before this story is Done
**Status**: [x] Created and passing in CI run `25167672501`

---

## Completion Notes

**Completed**: 2026-04-30
**Criteria**: 16/16 passing
**Deviations**: None blocking. The implementation uses Bevy 0.18 `MessageWriter`/`MessageReader` APIs and keeps `ResMut<RoundState>` isolated to `transitions.rs`, matching ADR-009/ADR-010 and CI gates.
**Test Evidence**: Logic evidence at `tests/unit/rsm/rsm_transitions_test.rs` and `tests/evidence/rsm-story-002-tests.md`; runnable tests in `server/tests/rsm_transitions_test.rs` covered by `cargo test -p server` in CI run `25167672501`.
**Implementation Commit**: `cb550b9`
**Code Review**: Lean mode skipped; CI green.

---

## Dependencies

- Depends on: Story 001 (state and events scaffold) must be Done — `RoundState`, `RoundPhase`, all event types must be defined and compiling
- Unlocks: Story 003 (timers and input reader)
