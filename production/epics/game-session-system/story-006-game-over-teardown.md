# Story 006: Game-Over Teardown

> **Epic**: Game Session System
> **Status**: Ready
> **Layer**: Core
> **Type**: Integration
> **Manifest Version**: 2026-04-29

## Context

**GDD**: `design/gdd/game-session-system.md`
**Requirement**: TR-GSS-07 (`SessionConfig` and `ServerRng` destroyed on `GameOverEmitted`)

**ADR Governing Implementation**: ADR-010 (RSM Event Bus), ADR-005 (Server-side RNG lifecycle), ADR-008 (Lightyear Channel Config)
**ADR Decision Summary**: The GSS subscribes to `GameOverEmitted` via `EventReader<GameOverEmitted>` (buffered event from the RSM event bus — ADR-010). On receipt, the GSS broadcasts `S2CGameOver` on `ReliableChannel`, then removes `SessionConfig` and `ServerRng` from the world, and transitions `LobbyState` to `GameOver`. `ReconnectTracker` cleanup is also performed here.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: LOW
**Engine Notes**: `EventReader<GameOverEmitted>` is standard Bevy buffered event consumption. `commands.remove_resource::<T>()` is a stable Bevy API. `liv-bevy-018` and `liv-bevy-lightyear` skills are mandatory on all `.rs` files.

**Control Manifest Rules (Core layer)**:
- Required: `SessionConfig` and `ServerRng` removed from the world exactly once — in `handle_game_over_teardown`.
- Required: `S2CGameOver` sent on `ReliableChannel` before resources are removed.
- Required: `ActiveSessions` entries for all session players removed in teardown.
- Forbidden: Any system other than `handle_game_over_teardown` calling `commands.remove_resource::<SessionConfig>()` or `commands.remove_resource::<ServerRng>()`.

---

## Acceptance Criteria

- [ ] `handle_game_over_teardown` system exists in `server/src/core/session/system.rs` and:
  - Subscribes to `EventReader<GameOverEmitted>` (from Epic 1 — RSM event bus)
  - On `GameOverEmitted { loser, round, reason }`: broadcasts `S2CGameOver { loser, round, reason }` to all session participants on `ReliableChannel`
  - Calls `commands.remove_resource::<SessionConfig>()` after the broadcast
  - Calls `commands.remove_resource::<ServerRng>()` after the broadcast
  - Calls `commands.insert_resource(LobbyState::GameOver)` to finalize session state
  - Removes all remaining session resources: `SessionSlots`, `ClassSelections`, `ClassPreviews`, `LobbyDeadline`, `LobbyHeartbeats` (if still present — they may have been removed by the disconnect path in Story 005)
  - Removes `ActiveSessions` entries for every player in the session
  - Cleans up `ReconnectTracker`: removes all token entries for this session from `ReconnectTracker.token_map`, clears `deferred_queue` entries, sets all `snapshot_sent` entries to `false` (or removes them)
  - Is idempotent: if `LobbyState` is already `GameOver` when the system runs, returns without action
- [ ] `S2CGameOver` message type exists in `shared/src/protocol.rs` (workspace-and-shared-types Story 004) with fields: `loser: PlayerId`, `round: u32`, `reason: GameOverReason`
- [ ] `GameOverEmitted` event type is consumed from Epic 1 (RSM event bus) — this story does not define the event type, only subscribes to it
- [ ] `GameSessionPlugin` registers `handle_game_over_teardown` in the Bevy `Update` schedule, after RSM event-emitting systems (use `.after(advance_phase)` or system set ordering per ADR-010)
- [ ] After teardown, `Res<SessionConfig>` and `Res<ServerRng>` do NOT exist in the world (accessing them via `Option<Res<...>>` returns `None`)
- [ ] `cargo check -p server` passes with zero warnings
- [ ] Integration test in `tests/integration/session/game_over_teardown_test.rs` passes

---

## Implementation Notes

*Derived from EPIC.md §Scope (handle_game_over_teardown) and ADR-005 §Resource Lifecycle:*

**`GameOverEmitted` source**: This event is defined and emitted by Epic 1 (Round State Machine). This story is a subscriber only. Do not define `GameOverEmitted` here — import from the RSM module. If the RSM epic is not yet complete when this story is implemented, use a placeholder stub event type marked `// TODO: replace with RSM GameOverEmitted after Epic 1 merges`.

**Resource removal ordering**: Remove resources in this order to prevent any system that reads `SessionConfig` or `ServerRng` from running after they are removed but before the game-over broadcast fires:
1. Broadcast `S2CGameOver` (while resources still exist, in case the broadcast assembly reads them — it doesn't, but the ordering is defensive)
2. `remove_resource::<SessionConfig>()`
3. `remove_resource::<ServerRng>()`
4. Transition `LobbyState` to `GameOver`
5. Remove remaining session resources

**`ReconnectTracker` cleanup**: The `ReconnectTracker` resource is defined in ADR-011 and owned by the session. Its `token_map` grows with session duration. Cleanup here is the canonical teardown point. If `ReconnectTracker` does not yet exist as a resource (e.g., Story 007 is not yet merged), the cleanup step is a no-op guarded by `Option<ResMut<ReconnectTracker>>`.

**Idempotency**: `GameOverEmitted` may theoretically be emitted more than once if there is a bug in the RSM. The `LobbyState::GameOver` guard prevents double-teardown, double resource removal (which would panic in Bevy if the resource does not exist), and double `S2CGameOver` broadcast.

**`remove_resource` safety**: Bevy 0.18 panics if `remove_resource::<T>()` is called when `T` does not exist. Guard each removal with an existence check via `world.contains_resource::<T>()`, or use `Option<Res<T>>` in the system parameters to test existence before removal. The idempotency guard on `LobbyState` makes double-removal nearly impossible in practice, but defensive guards are good form.

---

## Out of Scope

- `C2SAcknowledgeResult` handler (post-game acknowledgment — post-MVP)
- `ReconnectTracker.token_map` stale-entry TTL cleanup (post-MVP; the cleanup here covers the normal path)
- Match history persistence / leaderboard updates (post-MVP)
- Lobby re-queue / rematch flow (post-MVP)

---

## QA Test Cases

*QL-STORY-READY skipped — Lean mode.*

- **AC: game-over teardown — resources removed**
  - Given: `World` with `SessionConfig`, `ServerRng`, `LobbyState::GameActive` all present; `GameOverEmitted { loser: A, round: 5, reason: ObjectivesDestroyed }` enqueued
  - When: `handle_game_over_teardown` runs
  - Then: `Res<SessionConfig>` does not exist (`Option<Res<SessionConfig>>` returns `None`); `Res<ServerRng>` does not exist; `LobbyState == GameOver`

- **AC: game-over teardown — S2CGameOver broadcast**
  - Given: Same setup as above; session has participants A and B
  - When: `handle_game_over_teardown` runs
  - Then: `S2CGameOver { loser: A, round: 5 }` in outbound broadcast queue targeting both A and B

- **AC: idempotency — second GameOverEmitted is a no-op**
  - Given: `LobbyState::GameOver` (teardown already ran)
  - When: `handle_game_over_teardown` runs again (e.g., RSM emitted a second `GameOverEmitted`)
  - Then: No resource removal attempted; no second `S2CGameOver` sent; no panic

- **AC: ActiveSessions cleaned up**
  - Given: `ActiveSessions` contains entries for both session players
  - When: `handle_game_over_teardown` runs
  - Then: `ActiveSessions` is empty (or contains entries only for other unrelated sessions)

---

## Test Evidence

**Story Type**: Integration
**Required evidence**: `tests/integration/session/game_over_teardown_test.rs` — all test cases passing
**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 (session types — `LobbyState`, `SessionConfig`)
- Depends on: Story 004 (F4 + SessionReady — `SessionConfig` and `ServerRng` are inserted here; they must exist before teardown can remove them)
- Depends on: round-state-machine epic — `GameOverEmitted` event type definition (ADR-010 RSM event bus)
- Unlocks: Story 007 (reconnect story documents `ReconnectTracker` cleanup; teardown here is the canonical cleanup path that reconnect must not conflict with)
