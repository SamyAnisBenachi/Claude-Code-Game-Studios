# Story 005: Lobby Disconnect — Dual-Signal Cancel

> **Epic**: Game Session System
> **Status**: Complete
> **Layer**: Core
> **Type**: Integration
> **Manifest Version**: 2026-05-01

## Pre-conditions

**Lightyear 0.26 verification (ADR-011 checklist items 12–14)**: The following must be confirmed against Lightyear 0.26 docs/source before writing `handle_lobby_disconnect`:
- Item 12: `OnDisconnected` event fires in the Bevy `Update` schedule (not `FixedUpdate` or `PostUpdate`) and carries the disconnected `ClientId`.
- Item 13: The `ClientId` in `OnDisconnected` is the same `ClientId` that was used in `SessionParticipants` — it is not remapped before the event fires.
- Item 14: `OnDisconnected` fires at most once per transport disconnect, even for half-open WebSocket states (no spurious duplicate fires during browser sleep).

If any of these items cannot be confirmed, the `handle_lobby_disconnect` fallback path (dual-signal via heartbeat gap) still functions correctly — but the primary `OnDisconnected` path must be marked as "unverified" in a code comment until confirmed.

**2026-05-02 API alignment**: Current compiled Lightyear 0.26.4 usage in this repo observes connection state through `Connected` / `Disconnected` marker components with Bevy observers (`On<Add, Connected>` and `On<Add, Disconnected>`). Implement the "OnDisconnected" acceptance wording via that verified marker-component observer path, not via legacy `EventReader`/`EventWriter` APIs.

**Note**: `lobby_heartbeat_timeout_seconds` value reads from `Res<GameConfig>`. The GDD documents an open question (OQ-8) about whether this field name is `lobby_heartbeat_timeout_seconds` or a different variant. Verify the exact field name in `shared/src/config.rs` (implemented in workspace-and-shared-types Story 003) before using it. The default value is 15s per EPIC.md.

---

## Context

**GDD**: `design/gdd/game-session-system.md`
**Requirement**: TR-GSS-005 (lobby cancel on player disconnect during LOBBY — MVP: immediate cancel)

**ADR Governing Implementation**: ADR-011 (Reconnect Flow — LOBBY is explicitly out of scope for reconnect-with-grace; any disconnect in LOBBY is immediate cancel), ADR-008 (Lightyear Channel Config — `S2CSessionCancelled` on `ReliableChannel`)
**ADR Decision Summary**: LOBBY disconnect does not have a grace window at MVP. `OnDisconnected` is the primary cancel signal; `C2SHeartbeat` gap is the fallback. First signal wins — if the session is already `LobbyCancelled`, subsequent cancel attempts are no-ops. `S2CSessionCancelled` is broadcast to all remaining connected session participants.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: MEDIUM
**Engine Notes**: `OnDisconnected` is a Lightyear 0.26 event (post-cutoff). The API shape — whether it is an ECS Event, an Observer trigger, or an ECS component — must be confirmed (ADR-011 checklist item 12). `liv-bevy-018` and `liv-bevy-lightyear` skills are mandatory on all `.rs` files.

**Control Manifest Rules (Core layer)**:
- Required: Dual-signal first-wins: once `LobbyState == LobbyCancelled`, `handle_lobby_disconnect` and `tick_lobby_heartbeats` are both no-ops.
- Required: All session resources (`SessionSlots`, `ClassSelections`, `ClassPreviews`, `LobbyDeadline`, `LobbyHeartbeats`, `LobbyState`) are removed from the world on cancel.
- Required: `ActiveSessions` map entry for the disconnecting player is removed on cancel.
- Forbidden: Reconnect-with-grace during LOBBY. This is a post-MVP feature; do not add grace window logic here.

**Performance note**: The added Update-loop work must stay lobby-only and O(players) over current session participants/heartbeats. It must early-return outside LOBBY states and avoid broad world scans so it has no impact on active gameplay phases.

---

## Acceptance Criteria

- [ ] `handle_lobby_disconnect` system exists in `server/src/core/session/system.rs` and:
  - Subscribes to Lightyear `OnDisconnected` (or equivalent Lightyear 0.26 API — verify checklist item 12)
  - Looks up the `ClientId` in `SessionParticipants` to resolve `PlayerId`
  - If the `PlayerId` is not in any active session, or if the session is not in `LobbyWaiting` or `LobbyReady`, returns without action (no double-cancel)
  - Calls the shared cancel procedure (see Implementation Notes) with `reason: PlayerDisconnected`
- [ ] `tick_lobby_heartbeats` system exists in `server/src/core/session/system.rs` and:
  - Runs every `Update` tick while `LobbyState == LobbyWaiting`
  - For each entry in `LobbyHeartbeats`: if `now - last_heartbeat_time > game_config.lobby_heartbeat_timeout_seconds as f64`, calls the cancel procedure with `reason: HeartbeatTimeout`
  - Updates `LobbyHeartbeats[player_id] = now` when a `C2SHeartbeat` is received (in a separate `handle_lobby_heartbeat` system)
- [ ] `lobby_timeout_check` system exists in `server/src/core/session/system.rs` and:
  - Runs every `Update` tick while `LobbyState == LobbyWaiting`
  - If `now > lobby_deadline.0` and F4 is false, calls the cancel procedure with `reason: LobbyTimeout`
- [ ] Shared cancel procedure (private function or inline logic) performs all of the following atomically within the system:
  - Sets `LobbyState` to `LobbyCancelled`
  - Broadcasts `S2CSessionCancelled { reason }` to all remaining connected session participants on `ReliableChannel`
  - Removes all session resources: `SessionSlots`, `ClassSelections`, `ClassPreviews`, `LobbyDeadline`, `LobbyHeartbeats`
  - Removes the session from `ActiveSessions` for every player that was in the session
  - Does NOT remove `LobbyState` itself (remains as `LobbyCancelled` for inspection)
- [ ] Dual-signal first-wins: if the session is already `LobbyCancelled` when any cancel path fires, no action is taken and no second `S2CSessionCancelled` is sent
- [ ] `GameSessionPlugin` registers `handle_lobby_disconnect`, `tick_lobby_heartbeats`, `handle_lobby_heartbeat`, and `lobby_timeout_check` in the Bevy `Update` schedule
- [ ] `cargo check -p server` passes with zero warnings
- [ ] Unit tests in `tests/unit/session/dual_signal_disconnect_test.rs` pass
- [ ] Unit tests in `tests/unit/session/lobby_timeout_test.rs` pass

---

## Implementation Notes

*Derived from EPIC.md §Scope (handle_lobby_disconnect, tick_lobby_heartbeats, lobby_timeout_check) and ADR-011 §Constraints:*

**`OnDisconnected` API shape**: Lightyear 0.26 `OnDisconnected` is post-cutoff. Before implementation, confirm whether this is:
- An ECS `Event<OnDisconnected>` read via `EventReader<OnDisconnected>`
- A Bevy `Observer` trigger (with `Trigger<OnDisconnected>`)
- A component added to a Lightyear client entity

The implementation pattern changes significantly depending on which it is. Document the verified pattern in a comment in `handle_lobby_disconnect`.

**Dual-signal semantics**: Both `handle_lobby_disconnect` (immediate on `OnDisconnected`) and `tick_lobby_heartbeats` (gap > 15s) may fire in the same or consecutive ticks. The first to run transitions `LobbyState` to `LobbyCancelled`. The second call checks `LobbyState == LobbyCancelled` at entry and returns without action — this is the first-wins invariant. No locking or atomic compare-and-swap is needed in Bevy's single-threaded `Update` schedule.

**Heartbeat gap covers half-open TCP**: Browser tabs that are backgrounded may not fire `OnDisconnected` for 2–7 minutes due to OS-level TCP keepalive delays. `C2SHeartbeat` is sent on `UnreliableChannel` every ~3s (`heartbeat_interval_ms: 3000` in GameConfig). A 15s gap (5 missed heartbeats) is a reliable secondary signal. The gap threshold is configurable via `GameConfig.lobby_heartbeat_timeout_seconds`.

**Resource cleanup order**: The cancel procedure removes all per-session resources via `commands.remove_resource::<T>()` in this order: `LobbyHeartbeats` first, then `ClassPreviews`, `ClassSelections`, `LobbyDeadline`, `SessionSlots`. Remove `LobbyState` last if at all — leaving it as `LobbyCancelled` allows post-cancel inspection in tests. The `ActiveSessions` map entry removal must happen in the cancel procedure, not deferred, to prevent the `AlreadyInSession` rejection from blocking the player permanently.

**`lobby_timeout_check` and heartbeat tick run-condition**: Gate both systems on a Bevy run condition checking `LobbyState == LobbyWaiting`. This avoids unnecessary iteration after cancellation.

---

## Out of Scope

- In-game (post-DRAFT_INITIAL) disconnect handling — that is ADR-011 reconnect flow, Story 007
- Grace window for LOBBY disconnect (post-MVP feature)
- `S2COpponentDisconnected` broadcast during game — out of scope for LOBBY
- Lightyear `OnConnected` handling — covered by Story 007 reconnect path

---

## QA Test Cases

*QL-STORY-READY skipped — Lean mode.*

- **AC: OnDisconnected immediate cancel**
  - Given: 2-slot session in `LobbyWaiting`; Player A and Player B in slots
  - When: Lightyear fires `OnDisconnected` for Player A's `ClientId`
  - Then: `LobbyState == LobbyCancelled`; `S2CSessionCancelled { reason: PlayerDisconnected }` in outbound queue to B; all session resources removed; `ActiveSessions` no longer contains A or B

- **AC: heartbeat gap cancel**
  - Given: Session in `LobbyWaiting`; `LobbyHeartbeats[A]` set to `now - 20s` (gap > 15s threshold)
  - When: `tick_lobby_heartbeats` runs
  - Then: `LobbyState == LobbyCancelled`; cancel procedure executed with `reason: HeartbeatTimeout`

- **AC: dual-signal first-wins**
  - Given: Session already in `LobbyCancelled`
  - When: `handle_lobby_disconnect` runs (delayed `OnDisconnected`) and `tick_lobby_heartbeats` runs in the same tick
  - Then: Cancel procedure not called again; no second `S2CSessionCancelled` sent; no panic

- **AC: lobby timeout cancel**
  - Given: Session in `LobbyWaiting`; `LobbyDeadline` set to `now - 1s` (past deadline); F4 is false (one slot empty)
  - When: `lobby_timeout_check` runs
  - Then: `LobbyState == LobbyCancelled`; cancel procedure executed with `reason: LobbyTimeout`

- **AC: heartbeat received — no cancel**
  - Given: Session in `LobbyWaiting`; `LobbyHeartbeats[A]` set to `now - 5s` (within threshold)
  - When: `tick_lobby_heartbeats` runs; then Player A sends `C2SHeartbeat`
  - Then: `LobbyHeartbeats[A]` updated to `now`; no cancel; `LobbyState == LobbyWaiting`

---

## Test Evidence

**Story Type**: Integration
**Required evidence**:
- `tests/unit/session/dual_signal_disconnect_test.rs` — all test cases passing
- `tests/unit/session/lobby_timeout_test.rs` — all test cases passing
**Status**: [x] Created and passing locally

---

## Dependencies

- Depends on: Story 001 (session types — `LobbyHeartbeats`, `LobbyState`, `SessionSlots`, etc.)
- Depends on: Story 002 (room create/join establishes `SessionParticipants` and `ActiveSessions`)
- Depends on: lightyear-protocol-verification epic (ADR-011 checklist items 12–14 for `OnDisconnected` API verification)
- Unlocks: Story 007 (reconnect — requires dual-signal disconnect detection to be in place before reconnect can be robustly tested)

## Completion Notes

**Completed**: 2026-05-02
**Criteria**: 9/9 top-level acceptance criteria passing.
**Deviations**: None blocking. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates. Scope note: the implementation uses Cargo integration-test wrappers under `server/tests/` to execute the required `tests/unit/session/` evidence files.
**Repair**: `lobby_timeout_check` now cancels when `now > lobby_deadline.0` and F4 is false. Regression coverage added for the fully filled/class-locked-after-deadline case that previously remained `LobbyWaiting`.
**Test Evidence**: Integration story covered by `tests/unit/session/dual_signal_disconnect_test.rs` and `tests/unit/session/lobby_timeout_test.rs`; `cargo fmt -p server -- --check`, `cargo test -p server --test dual_signal_disconnect_test --test lobby_timeout_test`, and `cargo check -p server` passed on 2026-05-02.
**Code Review**: Skipped - Lean mode.
