# Story 004: F4 Predicate and SessionReady Trigger

> **Epic**: Game Session System
> **Status**: Ready
> **Layer**: Core
> **Type**: Logic
> **Manifest Version**: 2026-05-01

## Readiness Refresh

2026-05-01: Revalidated against control manifest version 2026-05-01. The stale
ADR-012 verification blocker is cleared by the current manifest rules:
`SessionReady` uses `#[derive(Event)]` plus `commands.trigger(SessionReady)`,
GSS inserts `SessionConfig` then `ServerRng` before triggering, and exactly one
RSM observer handles `SessionReady`. Implement against those rules; do not use
buffered messages for `SessionReady`.

---

## Context

**GDD**: `design/gdd/game-session-system.md`
**Requirement**: TR-GSS-01, TR-GSS-02, TR-GSS-03, TR-GSS-04, TR-GSS-05 (all five depend on this story)

**ADR Governing Implementation**: ADR-012 (SessionReady Delivery), ADR-005 (Server-side RNG), ADR-009 (RSM Phase State)
**ADR Decision Summary**: `SessionReady` is delivered via Observer trigger in the same `Update` tick as `SessionConfig` and `ServerRng` are inserted. The F4 predicate runs every tick while `LobbyState == LobbyWaiting`. Single-fire guard: `LobbyState::GameActive` prevents re-evaluation. `ServerRng` init failure cancels the session — `SessionReady` is never triggered.

**Engine**: Bevy 0.18 | **Risk**: HIGH
**Engine Notes**: The Commands flush ordering invariant (insert-before-trigger) is the single highest-risk implementation detail in the entire epic. Verify before writing any code in this story. If unverifiable, use the `World::trigger` exclusive system fallback. `liv-bevy-018` skill is mandatory on all `.rs` files.

**CI grep gates (enforced by CI — these MUST pass for Definition of Done):**
- `grep -r "EventReader<SessionReady>" server/src/` returns **zero matches**
- `grep -r "app.add_event::<SessionReady>" server/src/` returns **zero matches**
- `grep -r "app.observe(on_session_ready" server/src/` returns **exactly one match**

**Control Manifest Rules (Core layer)**:
- Required: `evaluate_session_ready` is in `SystemSet::LobbyEval`, scheduled `.before(advance_phase)`.
- Required: `on_session_ready` is registered via `app.observe(on_session_ready)` exactly once across all plugin registrations.
- Forbidden: `app.add_event::<SessionReady>()` anywhere in the codebase.
- Forbidden: Any second `app.observe` on `SessionReady`.

---

## Acceptance Criteria

- [ ] `evaluate_session_ready` system exists in `server/src/core/session/system.rs` and:
  - Returns immediately if `LobbyState != LobbyWaiting` (single-fire guard — `GameActive` state prevents re-evaluation)
  - F4 predicate: evaluates `all_slots_filled(slots) && all_classes_confirmed(slots, selections) && now < lobby_deadline.0`
  - Returns without triggering if F4 is false
  - On F4 true:
    1. Calls `build_session_config(&slots, &selections)` — panics if invariant violated (any occupied slot `class = None`)
    2. Calls `ServerRng::new()` — if init fails, transitions `LobbyState` to `LobbyCancelled`, broadcasts `S2CSessionCancelled { reason: RngInitFailure }`, returns without triggering `SessionReady`
    3. `commands.insert_resource(session_config)` — applied first
    4. `commands.insert_resource(server_rng)` — applied second
    5. `commands.trigger(SessionReady)` — observer fires after both inserts in same flush
    6. `commands.insert_resource(LobbyState::GameActive)` — prevents re-evaluation on subsequent ticks
- [ ] If ADR-012 item (2) verification fails: `evaluate_session_ready` is implemented as `fn evaluate_session_ready_exclusive(world: &mut World)` using `world.insert_resource` and `world.trigger` — same observable behaviour, documented in a comment referencing ADR-012 Alternative 2
- [ ] `on_session_ready` observer exists in `server/src/core/rsm/system.rs` and:
  - Signature: `fn on_session_ready(_trigger: On<SessionReady>, config: Res<SessionConfig>, _rng: Res<ServerRng>, mut phase: ResMut<RoundPhase>, ...)`
  - Sets `RoundPhase` to `DraftInitial`
  - Sets `round_number = 1`
  - Broadcasts `S2CPhaseChanged { phase: DraftInitial, round: 1, timer_remaining_ms: config.draft_initial_timer_ms }` to all connected players on `ReliableChannel`
- [ ] `GameSessionPlugin` registers `evaluate_session_ready` in `SystemSet::LobbyEval` scheduled `.before(advance_phase)`
- [ ] `GameSessionPlugin` calls `app.observe(on_session_ready)` exactly once
- [ ] `LobbyHeartbeats` resource is removed (or handed off to `RoundState.disconnect_trackers`) in `on_session_ready` or immediately after `SessionReady` fires, so the GSS heartbeat tracker and RSM disconnect tracker do not overlap
- [ ] All three CI grep gates pass (see above)
- [ ] `cargo check -p server` passes with zero warnings
- [ ] Unit tests in `tests/unit/session/session_ready_test.rs` pass — GSS-30
- [ ] Unit tests in `tests/unit/session/single_fire_test.rs` pass — single-fire invariant
- [ ] Unit tests in `tests/unit/session/rng_init_failure_test.rs` pass — GSS-29
- [ ] Integration test in `tests/integration/session/lobby_to_draft_initial_test.rs` passes — RSM-1

---

## Implementation Notes

*Derived from EPIC.md §Scope (evaluate_session_ready), ADR-012 §Decision and §Implementation Guidelines:*

**Verification-first discipline**: Do not write `evaluate_session_ready` until the ADR-012 verification spike is complete. The call order `insert_resource → insert_resource → trigger` is load-bearing; an incorrect assumption produces a panic in `on_session_ready` that is difficult to reproduce in unit tests (the panic happens inside an Observer handler, which may surface differently than a normal system panic).

**`ServerRng::new()` failure handling**: In production, `OsRng::from_entropy()` failing is extremely unlikely on Linux/macOS servers. However, the failure path must exist and must be tested. The `rng_init_failure_test.rs` test injects a mock/failing factory. See EPIC.md §Implementation Notes: "For deterministic tests, `evaluate_session_ready` accepts a `Resource` providing the `ServerRng` factory (default impl uses `OsRng`; tests inject a fixed seed)."

**`LobbyHeartbeats` handoff**: The GSS owns heartbeats from slot occupation until `SessionReady` fires. The RSM owns `disconnect_trackers` with the 30s grace window after DRAFT_INITIAL begins. The `on_session_ready` observer (or immediately after it in the same flush) must clear `LobbyHeartbeats` and copy the current connection state into `RoundState.disconnect_trackers`. The two trackers must not overlap — a player in both trackers simultaneously gets double-counted on disconnect.

**`build_session_config` panic is a programming error gate**: If `on F4 true` the predicate was correctly evaluated, every occupied slot has `class = Some(_)` — the panic in `build_session_config` should never be reachable. If it panics, a bug in the F4 predicate or class confirmation path exists. The panic message must include enough context (slot index, player ID) to diagnose.

**Exclusive system performance**: If the exclusive system fallback is adopted, it blocks all parallel system execution for the duration of `evaluate_session_ready`. This runs every `Update` tick during LOBBY. The F4 predicate is 3 bool checks and 1 f64 comparison — negligible. The exclusive lock is not a performance concern at MVP.

**`on_session_ready` location**: The observer handler lives in `server/src/core/rsm/system.rs` (RSM module), but is registered by `GameSessionPlugin` via `app.observe(on_session_ready)`. This is the correct pattern: the GSS owns the trigger; the RSM owns the handler; the GSS plugin wires them together. This preserves the RSM-zero-import constraint (RSM does not import from GSS).

---

## Out of Scope

- `advance_phase` implementation: Epic 1 — Round State Machine
- `RoundPhase` type definition: Epic 1 — Round State Machine  
- `disconnect_trackers` resource definition: Epic 1 — Round State Machine
- `ServerRng` struct definition: server-rng Foundation epic
- Objective System and Card Pool reactions to `DraftInitial`: their respective epics

---

## QA Test Cases

*QL-STORY-READY skipped — Lean mode.*

- **GSS-30: post-trigger world state**
  - Given: `World` with 2 occupied slots, both classes confirmed, `LobbyDeadline` in the future, `LobbyState::LobbyWaiting`
  - When: `evaluate_session_ready` runs (or is triggered via `World::run_system`)
  - Then: `Res<SessionConfig>` exists with `player_count == 2`, `class_map.len() == 2`; `Res<ServerRng>` exists; `RoundPhase == DraftInitial`; all three observable in the same test assertion block

- **GSS-29: RNG init failure — no trigger**
  - Given: `World` with valid F4 state; `ServerRng` factory injected to always fail
  - When: `evaluate_session_ready` runs
  - Then: `SessionReady` NOT triggered; `LobbyState == LobbyCancelled`; `RoundPhase` remains `LOBBY`; `S2CSessionCancelled { reason: RngInitFailure }` in outbound message queue

- **Single-fire invariant**
  - Given: Session has already triggered `SessionReady` (`LobbyState == GameActive`)
  - When: `evaluate_session_ready` runs 3 additional ticks
  - Then: Observer trigger count remains 1; no second `SessionReady` fired; no second resource insertion

- **RSM-1: LOBBY → DRAFT_INITIAL same tick (integration)**
  - Given: Full session setup in Bevy `App` with both GSS and RSM plugins registered
  - When: F4 becomes true and one `App::update()` is called
  - Then: `S2CPhaseChanged(DraftInitial, round=1)` is in the outbound broadcast queue; `RoundPhase == DraftInitial`; `Res<SessionConfig>` exists — all in the same update tick

---

## Test Evidence

**Story Type**: Logic
**Required evidence**:
- `tests/unit/session/session_ready_test.rs` (GSS-30) — passing
- `tests/unit/session/single_fire_test.rs` (single-fire invariant) — passing
- `tests/unit/session/rng_init_failure_test.rs` (GSS-29) — passing
- `tests/integration/session/lobby_to_draft_initial_test.rs` (RSM-1) — passing
- Control manifest 2026-05-01 alignment documented in this story's Readiness Refresh
**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 (session types), Story 002 (slot state), Story 003 (ClassSelections populated by confirm)
- Depends on: round-state-machine epic (ADR-009 — `RoundPhase`, `advance_phase` must be defined before `on_session_ready` can set them)
- Depends on: server-rng epic (ADR-005 — `ServerRng::new()` must exist)
- Depends on: ADR-012 accepted and control manifest 2026-05-01 alignment (Readiness Refresh above)
- Unlocks: Story 006 (game-over teardown subscribes to `GameOverEmitted` which requires the RSM to be active, which requires DRAFT_INITIAL to have begun)
