# Story 003: Timers and Input Reader

> **Epic**: Round State Machine
> **Status**: Complete
> **Layer**: Core
> **Type**: Logic
> **Manifest Version**: 2026-04-29

## Context

**GDD**: `design/gdd/round-state-machine.md`
**Requirement**: TR-RSM-02, TR-RSM-03, TR-RSM-07
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-009 (RSM Phase State as ECS Resource), ADR-010 (RSM Phase Event Bus), ADR-012 (SessionReady Observer Delivery)
**ADR Decision Summary**: `SessionReady` is delivered via `Trigger<SessionReady>` Observer (same-frame), not `EventReader<SessionReady>` (ADR-012). Timer durations come from `Res<GameConfig>` fields — never hardcoded. `rsm_input_reader` reads inbound events (`AuctionSettled`, `ResolutionComplete`) and schedules `.before(advance_phase)`. Timer tick system activates only the timer for the current phase.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**: `Trigger<SessionReady>` Observer handler parameter — verify `Trigger<E>` vs `On<E>` signature in Bevy 0.18 (post-cutoff API, ADR-012 verification required). `Timer::tick(time.delta())` API — verify in Bevy 0.18. RSM inbound messages use `MessageReader::read()` — `EventReader` no longer exists in Bevy 0.18. `liv-bevy-018` skill mandatory on all files in this story.

**Control Manifest Rules (Core layer)**:
- Required: `on_session_ready` is registered as `app.observe(on_session_ready)` in `RsmPlugin::build()` — NOT via `app.add_systems`.
- Required: All timer duration values read from `Res<GameConfig>`: `config.draft_initial_timer_seconds`, `config.draft_shop_timer_seconds`, `config.placement_timer_seconds`. Default values: 45s, 30s, 10s respectively.
- Required: `rsm_input_reader` scheduled `.before(advance_phase)` in the `Update` set.
- Required: Timer tick system activates only the timer for the current phase (single active timer per tick).
- Required: Inbound event guard pattern in `rsm_input_reader`: `if rsm.phase != expected { continue; }` for both `AuctionSettled` and `ResolutionComplete`.
- Forbidden: Hardcoded timer duration values in `server/src/core/rsm/` — all timer durations from `GameConfig`.
- Forbidden: `EventReader<SessionReady>` in any system — `SessionReady` is Observer-only per ADR-012.

---

## Acceptance Criteria

- [ ] `server/src/core/rsm/system.rs` defines `rsm_input_reader` system: reads `MessageReader<AuctionSettled>` and `MessageReader<ResolutionComplete>`; applies inbound message guard (`if rsm.phase != RoundPhase::DraftAuction { continue; }` for `AuctionSettled`; `if rsm.phase != RoundPhase::Resolution { continue; }` for `ResolutionComplete`); updates `rsm.phase` to the appropriate next phase and calls or schedules `advance_phase` on match
- [ ] `rsm_input_reader` is registered in `RsmPlugin` with `.before(advance_phase)` scheduling constraint in the `Update` set
- [ ] `server/src/core/rsm/system.rs` defines a timer tick system that: ticks `placement_timer` only when `phase == Placement`; ticks `draft_shop_timer` only when `phase == DraftShop`; ticks `draft_initial_timer` only when `phase == DraftInitial`; calls `advance_phase` (or sets a trigger) when the active timer's `just_finished()` returns true
- [ ] `server/src/core/rsm/system.rs` defines `on_session_ready(trigger: Trigger<SessionReady>, mut rsm: ResMut<RoundState>, config: Res<GameConfig>)`: sets `rsm.phase = RoundPhase::DraftInitial`, `rsm.round_number = 1`, initialises `rsm.draft_initial_timer = Some(Timer::from_seconds(config.draft_initial_timer_seconds, TimerMode::Once))`; then calls or schedules `advance_phase` so F2 emission for DRAFT_INITIAL fires from the standard match arm in the same tick
- [ ] `on_session_ready` is registered via `app.observe(on_session_ready)` in `RsmPlugin::build()` — not via `app.add_systems`
- [ ] `RsmPlugin::build()` scheduling: `rsm_input_reader.before(advance_phase)`; timer tick system also `.before(advance_phase)` or `.after(rsm_input_reader)`; all subscriber systems (Economy, Card Pool, Board/Lane stubs) scheduled `.after(advance_phase)`
- [ ] Timer durations sourced from `GameConfig`: `draft_initial_timer_seconds = 45.0`, `draft_shop_timer_seconds = 30.0`, `placement_timer_seconds = 10.0` — if `GameConfig` fields do not yet exist, add them with these defaults in `assets/config/game_config.ron` and in the `GameConfig` struct
- [ ] Stale `AuctionSettled` arriving when `phase != DraftAuction` is silently discarded — RSM does not transition (RSM-31 guard path)
- [ ] Stale `ResolutionComplete` arriving when `phase != Resolution` is silently discarded — RSM does not transition (RSM-35 guard path)
- [ ] `tests/unit/rsm/rsm_timers_test.rs` passes all tests listed in the QA Test Cases section (RSM-2, RSM-10, RSM-15 through RSM-19, RSM-30)

---

## Implementation Notes

*Derived from ADR-009, ADR-010, ADR-012:*

**`on_session_ready` and `advance_phase` interaction:** The Observer fires in the same `Update` tick as `evaluate_session_ready` in the GSS. Inside `on_session_ready`, set `rsm.phase = RoundPhase::DraftInitial` and `rsm.round_number = 1`. Then trigger `advance_phase` for the DRAFT_INITIAL entry (so F2 emission — `DraftStarted`, `ShopRefreshNeeded`, `BroadcastPhaseChanged` — fires from the standard match arm, not duplicated in the observer). The exact mechanism to call `advance_phase` from an Observer context: use `commands.run_system(advance_phase_system_id)` or use a flag resource that the timer tick system reads next frame. Document the chosen mechanism in the implementation.

**Timer tick — only one active timer per tick:** The timer tick system checks `rsm.phase` and ticks only the relevant `Option<Timer>`. Example pattern:
```rust
fn tick_rsm_timers(
    mut rsm: ResMut<RoundState>,
    time: Res<Time>,
    // ... trigger mechanism for advance_phase
) {
    let elapsed = time.delta();
    match rsm.phase {
        RoundPhase::DraftInitial => {
            if let Some(ref mut t) = rsm.draft_initial_timer {
                t.tick(elapsed);
                if t.just_finished() { /* trigger advance_phase */ }
            }
        }
        RoundPhase::DraftShop => { /* tick draft_shop_timer */ }
        RoundPhase::Placement => { /* tick placement_timer */ }
        _ => {} // No RSM-owned timer for other phases
    }
}
```

**`rsm_input_reader` design:** This system reads `AuctionSettled` and `ResolutionComplete` events. It does NOT call `advance_phase` directly (to preserve the single-writer contract). Instead, it updates `rsm.phase` to the transitional state and sets a trigger that `advance_phase` checks on its next execution. Alternatively, `rsm_input_reader` can directly invoke `advance_phase` if `advance_phase` is exposed as a callable function (not just a system). Implementer chooses the cleanest approach consistent with Bevy 0.18 system scheduling rules.

**`DRAFT_SHOP` submission ("ready" signal):** The GDD (Rule 8) states players can signal ready before the timer expires. A `C2SPlayerReady` event (or similar) is read in `rsm_input_reader`. When all players are ready, `rsm_input_reader` triggers `advance_phase` for DRAFT_SHOP → PLACEMENT. The `C2SPlayerReady` type may not exist yet — if not, add it to the protocol as a stub.

**`PLACEMENT` full-submission check:** Read `MessageReader<C2SSubmitPlacement>`. On each receipt, add `player_id` to `rsm.submissions_received`. When `rsm.submissions_received.len() == session.player_count`, trigger `advance_phase`. Phase gate: reject if `rsm.phase != Placement`.

**`GameConfig` fields:** If `draft_initial_timer_seconds`, `draft_shop_timer_seconds`, `placement_timer_seconds` are not yet fields on `GameConfig`, add them as `f32` fields with the defaults above. These are Feel Knobs (ADR-009 Tuning Knobs) and must be in `assets/config/game_config.ron`, not hardcoded.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- Story 001: Event type definitions, `RoundState` struct
- Story 002: `advance_phase` match arm logic
- Story 004: Win condition evaluation, GAME_OVER transition from RESOLUTION
- Story 005: Disconnect tracking (`disconnect_trackers`, Lightyear `OnDisconnected` subscription)
- Story 006: Network dispatch wiring
- Auction System (M2): `AuctionSettled` emitter — this story only reads the event, not emits it

---

## QA Test Cases

Each test uses `World::new()` + event injection + `Time` resource injection. No live Lightyear session required.

- **RSM-2**: `round_number` increments correctly on RESOLUTION → DRAFT_* (inject `ResolutionComplete` while phase == Resolution; verify `round_number` after `rsm_input_reader` + `advance_phase` run)
- **RSM-10**: `rsm_input_reader` discards `AuctionSettled` when `phase != DraftAuction`; no transition occurs; no `BroadcastPhaseChanged` emitted
- **RSM-15**: `draft_initial_timer` starts at 45s on DRAFT_INITIAL entry; after advancing simulated time to 45s, timer fires; `advance_phase` transitions to PLACEMENT
- **RSM-16**: `draft_shop_timer` starts at 30s on DRAFT_SHOP entry; timer fires at 30s; `advance_phase` transitions to PLACEMENT
- **RSM-17**: `placement_timer` starts at 10s on PLACEMENT entry; timer fires at 10s; `advance_phase` transitions to RESOLUTION regardless of submission count; `BroadcastPhaseChanged { phase: Resolution }` emitted — RSM obligation ends here; Board/Lane System owns the "no cards placed for non-submitters" invariant (out of scope for this story)
  - Given: `RoundState { phase: Placement, placement_timer: Some(Timer at 10s) }`, submissions_received = {player_a} only (1 of 2)
  - When: simulated time advances to 10s; timer tick system runs
  - Then: `rsm.phase == Resolution`; `BroadcastPhaseChanged { phase: Resolution }` written; `rsm.submissions_received` unchanged (Board/Lane reads it separately)
  - Edge cases: 0 submissions at expiry (still transitions); timer must not trigger at 9.999s
- **RSM-18**: PLACEMENT early exit — all players submit before timer; `advance_phase` transitions to RESOLUTION immediately; timer is not ticked to zero
- **RSM-19**: DRAFT_SHOP early exit — all players signal ready before timer; `advance_phase` transitions to PLACEMENT immediately
- **RSM-30** (RSM obligation only — gold forfeiture is DEFERRED to Epic 3 Economy System): `draft_initial_timer` fires after `draft_initial_timer_seconds`; `advance_phase` transitions to `Placement`; `BroadcastPhaseChanged { phase: Placement }` emitted; `round_number` remains 1 on DraftInitial exit
  - Given: `RoundState { phase: DraftInitial, round_number: 1, draft_initial_timer: Some(Timer at 45s) }`, Player A has purchased some cards but timer expires
  - When: simulated time advances to 45s; timer tick system runs
  - Then: `rsm.phase == Placement`; `BroadcastPhaseChanged { phase: Placement }` written; `round_number` still 1
  - Edge cases: early-exit (all-submit before timer — RSM-16 pattern applies); gold forfeiture ("use-it-or-lose-it") is Economy System's responsibility — RSM does NOT assert on gold values
  - DEFERRED: GDD RSM-30 gold assertion (`Player A's gold = 0`) is tracked in Epic 3 (Economy System) story readiness review

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: Automated unit tests — `tests/unit/rsm/rsm_timers_test.rs` must pass; paste `cargo test -p server rsm_timers` output into `tests/evidence/rsm-story-003-tests.md`
**Gate Level**: BLOCKING — all tests listed in QA Test Cases must pass before this story is Done
**Status**: [x] Verified - `cargo test -p server rsm_timers` passed on 2026-05-01; executable coverage lives in `server/tests/rsm_timers_test.rs` with evidence pointer at `tests/unit/rsm/rsm_timers_test.rs`.

---

## Dependencies

- Depends on: Story 002 (advance_phase and F2 ordering) must be Done — `advance_phase` match arms must exist before the input reader and timer systems can call into them
- Unlocks: Story 004 (win condition and game over)

## Completion Notes
**Completed**: 2026-05-01
**Criteria**: 10/10 passing.
**Deviations**:
- Advisory: story manifest v2026-04-29 is older than current control manifest v2026-05-01.
- Advisory: story text uses stale Bevy observer wording (`Trigger<SessionReady>` / `app.observe`); implementation uses the verified Bevy 0.18 API (`On<SessionReady>` / `add_observer`).
**Test Evidence**: Logic evidence at `tests/unit/rsm/rsm_timers_test.rs`; executable suite `server/tests/rsm_timers_test.rs`; `cargo test -p server rsm_timers` passed with 10/10 RSM timer tests plus the timer default test. `cargo test -p server --test rsm_timers_test --test rsm_transitions_test` passed 24/24, including `ShopRefreshTriggered` draft-entry coverage. `cargo check -p server` passed.
**Blocker Check**: `ShopRefreshNeeded` search under `server` and `tests` returned no matches; RSM now emits `ShopRefreshTriggered`.
**Code Review**: Skipped - Lean mode.
