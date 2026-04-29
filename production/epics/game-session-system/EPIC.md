# Epic: Game Session System

> **Layer**: Core
> **GDD**: design/gdd/game-session-system.md
> **Architecture Module**: `server/core/session/` (full module — `state.rs`, `events.rs`, `system.rs`, `config.rs`, `plugin.rs`); contributes `on_session_ready` Observer registration to `server/core/rsm/`
> **Status**: Ready
> **Stories**: To be created — see Story Breakdown Hint below

## Overview

Implements the lobby finite-state machine and the session-readiness handoff that bridges connection-time concerns to the round loop. This epic owns `SessionSlot`, room creation and join, public class selection with deferred simultaneous reveal, the F4 readiness predicate (all slots filled + all classes confirmed + lobby deadline not expired), the lobby heartbeat / `OnDisconnected` immediate-cancel path, and — critically — the `SessionReady` Observer-trigger delivery that hands `SessionConfig` and `ServerRng` to the RSM in the same `Update` tick. After `SessionReady` fires, the GSS becomes a passive read-only configuration store: `Res<SessionConfig>` (mode, player_count, team_map, class_map) is the single source of session data for every Feature system. The GSS also owns the `ServerRng` lifecycle: it seeds the RNG from `OsRng` immediately before triggering `SessionReady` (per ADR-005) and destroys both `SessionConfig` and `ServerRng` resources on `GameOverEmitted` (subscribed from Epic 1's RSM event bus). This epic is the load-bearing gate between LOBBY and DRAFT_INITIAL: an Observer-trigger ordering bug here panics the RSM with "resource not found" and breaks every game session.

## Governing ADRs

| ADR | Decision Summary | Engine Risk |
|-----|-----------------|-------------|
| ADR-011: Reconnect Flow and Game Snapshot Protocol | `SessionToken` issued at first connect; `ClientId` re-mapped on every Lightyear transport reconnect; live message queue held until `S2CGameSnapshot` delivery confirmed; secret-stripping rules per player at snapshot send (own vs opponent) | HIGH |
| ADR-012: SessionReady Delivery (Observer, same-frame) | `SessionReady` is delivered via `Commands::trigger(SessionReady)` (Observer pattern), NOT buffered `Events<T>`; `SessionConfig` and `ServerRng` inserted via Commands BEFORE the trigger, in the same system; one Observer only (`on_session_ready` in RSM); exclusive-system `World::trigger` fallback documented if Commands flush ordering cannot be verified | HIGH |

## Engine Risk: HIGH

Three high-risk post-cutoff API behaviours converge in this epic:

1. **`Commands::trigger` ordering invariant** — The decision in ADR-012 hinges on whether `Commands::insert_resource` issued before `Commands::trigger` is guaranteed visible to the Observer handler. This must be verified against Bevy 0.18 docs/source before any session-start code is written. If unverifiable, fall back to an exclusive system using `World::insert_resource` + `World::trigger` (ADR-012 Alternative 2 — implementation is identical except `commands.*` becomes `world.*`).
2. **Observer registration vs `EventReader`** — A developer mistakenly adding `EventReader<SessionReady>` will silently never receive the trigger. The doc comment on `SessionReady` must be explicit. Code review gate: only `app.observe(on_session_ready)` is permitted; no `app.add_event::<SessionReady>()`.
3. **Lightyear 0.26 `OnDisconnected` / `OnConnected`** — Both events are post-LLM-cutoff. ADR-011 verification checklist items 12–14 must be confirmed before disconnect-cancel logic ships. Half-open WebSocket states (browser tab background / OS sleep) require the dual-signal pattern (Lightyear event + `C2SHeartbeat` gap fallback at 15s).

`liv-bevy-018` skill is mandatory on every `.rs` file in this epic. `liv-bevy-lightyear` is mandatory wherever `OnConnected`, `OnDisconnected`, `MessageSender`, or `SessionToken` handshake code lives.

## GDD Requirements

> Note: `docs/architecture/tr-registry.yaml` has not yet been populated. TR-IDs below are informal references from the ADR "GDD Requirements Addressed" sections.

| Informal TR-ID | Requirement | ADR Coverage |
|----------------|-------------|--------------|
| TR-GSS-01 | `SessionConfig` present in ECS world when RSM observer for `SessionReady` runs | ADR-012 ✅ (insert-before-trigger) |
| TR-GSS-02 | `ServerRng` present in ECS world when RSM observer for `SessionReady` runs | ADR-012 ✅ (insert-before-trigger) |
| TR-GSS-03 | `SessionReady` fires exactly once per session | ADR-012 ✅ (`LobbyState::GameActive` guard) |
| TR-GSS-04 | LOBBY → DRAFT_INITIAL transition runs in same Update tick as `SessionReady` | ADR-012 ✅ (Observer same-frame) |
| TR-GSS-05 | Objective System / Card Pool can read `Res<SessionConfig>` and `Res<ServerRng>` at DRAFT_INITIAL entry | ADR-012 ✅ |
| TR-GSS-06 | Lobby cancel on player disconnect during LOBBY (MVP: immediate cancel) | GDD Rule 9 + ADR-011 |
| TR-GSS-07 | `SessionConfig` and `ServerRng` destroyed on `GameOverEmitted` | ADR-005 + ADR-010 ✅ |
| TR-GSS-08 | `SessionToken` enables reconnect across new `ClientId` | ADR-011 ✅ |
| TR-GSS-09 | Class reveal: deferred simultaneous broadcast (`S2CClassesRevealed`) | GDD Rule 7 |
| TR-GSS-10 | One active session per `PlayerId`; idempotent `C2SCreateRoom` retry | GDD Rule 13 |

## Scope

### Deliverables

**`server/src/core/session/state.rs`**
- `SessionSlot { index: u8, team: TeamId, player: Option<PlayerId>, class: Option<ClassId> }`
- `LobbyState` enum: `LobbyWaiting | LobbyReady | GameActive | LobbyCancelled | GameOver`
- Per-session resources: `SessionSlots(Vec<SessionSlot>)`, `ClassSelections(HashMap<PlayerId, ClassId>)`, `LobbyDeadline(f64)`, `LobbyHeartbeats(HashMap<PlayerId, f64>)`, `LobbyState`
- `SessionId(Uuid)`, `RoomCode(String)` newtypes
- `SessionToken` (issued at first `C2SHello`, used by ADR-011 reconnect path)

**`server/src/core/session/events.rs`**
- `SessionReady` — zero-sized marker, `#[derive(Event)]`, doc comment explicitly stating "DELIVERY: Observer trigger (same-frame). NOT a buffered Event. Subscribe via `app.observe(on_session_ready)`. Adding `EventReader<SessionReady>` will silently never fire."
- `SessionCancelled { reason: SessionCancelledReason }` — buffered Event for post-cancel teardown subscribers (logging, etc.)

**`server/src/core/session/config.rs`**
- `SessionConfig { mode: GameMode, player_count: u8, team_map: HashMap<PlayerId, TeamId>, class_map: HashMap<PlayerId, ClassId> }` — `#[derive(Resource, Clone)]`. Inserted ONCE at `SessionReady` time. Never mutated after insertion. Removed by GSS on `GameOverEmitted`.
- `build_session_config(slots: &SessionSlots, selections: &ClassSelections) -> SessionConfig` — panics if any occupied slot has `class = None` (GDD Rule 11 invariant; ADR-012 Verification Required item).

**`server/src/core/session/system.rs`**
- `handle_create_room(C2SCreateRoom)` — assigns `session_id`, generates 6-char room code, initialises slots per mode, sets `lobby_deadline = now + lobby_timeout_seconds`. Includes the GDD Rule 13 idempotent-retry path: same player resending `C2SCreateRoom` for an existing `LOBBY_WAITING` session returns the existing room code.
- `handle_join_room(C2SJoinRoom)` — slot validation; on success: `S2CJoinAck` to joiner (full slot state), `S2CSlotUpdated` broadcast to others (full slot vector — never deltas). One-active-session check per Rule 13 (`AlreadyInSession` rejection).
- `handle_select_class(C2SSelectClass)` — updates preview only; not broadcast.
- `handle_confirm_class(C2SConfirmClass)` — same-system sequential write of `SessionSlot.class = Some(_)` and `class_selections[player_id] = class_id`; `S2CClassLocked` unicast to locking player; if all slots locked → `S2CClassesRevealed` broadcast.
- `evaluate_session_ready` — runs every tick while `LobbyState == LobbyWaiting`. F4 predicate: all slots filled AND all classes confirmed AND `now < lobby_deadline`. If true:
  1. Build `SessionConfig` from finalised slots (panic on `None` invariant violation).
  2. Initialise `ServerRng` from `OsRng`. If init fails → transition to `LobbyCancelled`, broadcast `S2CSessionCancelled`, do NOT trigger `SessionReady`.
  3. `commands.insert_resource(session_config)` — applied first.
  4. `commands.insert_resource(server_rng)` — applied second.
  5. `commands.trigger(SessionReady)` — Observer fires after both inserts in same flush.
  6. `commands.insert_resource(LobbyState::GameActive)` — guards re-evaluation.
- `handle_lobby_disconnect` — subscribes to Lightyear `OnDisconnected`; if `LobbyState == LobbyWaiting | LobbyReady`, immediately cancels session, broadcasts `S2CSessionCancelled { reason: PlayerDisconnected }`, destroys session resources.
- `tick_lobby_heartbeats` — fallback dual-signal: tracks `C2SHeartbeat` per occupied slot; if gap > `lobby_heartbeat_timeout_seconds` (default 15s — separate from RSM's 30s grace), cancel as if `OnDisconnected` fired. Tracker is destroyed on `SessionReady` (RSM takes over with `disconnect_grace_seconds`).
- `lobby_timeout_check` — at `now > lobby_deadline` with F4 false, transitions to `LobbyCancelled` with `reason: LobbyTimeout`.
- `handle_game_over_teardown` — subscribes to `EventReader<GameOverEmitted>` (from Epic 1); broadcasts `S2CGameOver { loser, round, reason }` on `ReliableChannel`; removes `SessionConfig` and `ServerRng` resources from world; transitions `LobbyState` to `GameOver`. The GAME_OVER → session destruction path lives here per ADR-010 subscriber contract.

**`on_session_ready` (lives in `server/core/rsm/system.rs`, registered by GSS plugin)**
- Per ADR-012: `fn on_session_ready(_t: Trigger<SessionReady>, config: Res<SessionConfig>, rng: Res<ServerRng>, mut round_state: ResMut<RoundState>, ...)`.
- Sets `round_state.phase = DraftInitial`, `round_state.round_number = 1`.
- Triggers the standard DRAFT_INITIAL match arm in `advance_phase` (so F2 emission for round 1 fires from one place — not duplicated in the observer).
- Single Observer invariant: only one `app.observe(on_session_ready)` call across all plugin registrations. Code review gate.

**`server/src/core/session/plugin.rs`**
- `GameSessionPlugin`: registers C2S handlers, `evaluate_session_ready` system (in `SystemSet::LobbyEval`, scheduled `.before(advance_phase)` per ADR-012), heartbeat tick, lobby timeout check, `OnDisconnected` subscriber, `GameOverEmitted` teardown subscriber, `app.observe(on_session_ready)`.

**Reconnect path (ADR-011)**
- `SessionToken` issued in response to first `C2SHello { protocol_version, session_token: None }`; stored on the session.
- On reconnect (`C2SHello { session_token: Some(t) }`): map new `ClientId` to existing `PlayerId`, hold live messages in `ReconnectTracker.snapshot_sent[player] = false` queue, send `S2CHandshake`, then `S2CGameSnapshot` (built per ADR-011 secret-stripping rules — own player gets full hand/objectives, opponent fields stripped to public only), then re-send `S2CObjectiveIdentities` (ADR-001), then `S2CPhaseChanged`, then unfreeze the live queue.
- `S2CGameSnapshot` builder is implemented here (it touches `Res<RoundState>`, `Res<SessionConfig>`, `Res<PlayerEconomy>`, `Res<HiddenObjectives>`, `Res<BoardGrid>`, `Res<PlayerPool>` — broadest cross-system read in the codebase).

**Tests**
- `tests/unit/session/` — F4 predicate truth table (all combinations of slot fill, class confirm, deadline expiry).
- `tests/unit/session/session_ready_test.rs` — GSS-30 invariant: after `evaluate_session_ready` runs, `Res<SessionConfig>` and `Res<ServerRng>` exist with correct values, `RoundPhase == DraftInitial`, observer fired exactly once. All asserted in the same test tick.
- `tests/unit/session/single_fire_test.rs` — running `evaluate_session_ready` 3 ticks after the first trigger: trigger count remains 1.
- `tests/unit/session/rng_init_failure_test.rs` — inject failing `ServerRng::new`: `SessionReady` is NOT triggered, `RoundPhase` stays LOBBY, `LobbyState == LobbyCancelled` (GSS-29).
- `tests/unit/session/class_reveal_test.rs` — both players confirm: `S2CClassLocked` to each player only; `S2CClassesRevealed` broadcast only after second confirm (Rule 7).
- `tests/unit/session/dual_signal_disconnect_test.rs` — Lightyear `OnDisconnected` cancels immediately; heartbeat gap > 15s cancels via fallback path; first signal wins.
- `tests/integration/session/reconnect_snapshot_test.rs` — full reconnect flow: live message queue held until snapshot sent (ADR-011 invariant).

### Out of Scope (owned by other epics)

- `RoundState` resource definition, `advance_phase`, F2 emission ordering, all phase Messages: Epic 1 — Round State Machine.
- `ServerRng` Resource type, audit log, `next_seed()` API: `server-rng` Foundation epic.
- `GameConfig` loading: `game-config-pipeline` Foundation epic.
- C2S/S2C message type definitions: `workspace-and-shared-types` Foundation epic. This epic uses them; it does not define them.
- Lightyear plugin setup: `lightyear-protocol-verification` Foundation epic.

### Implementation Notes

**ADR-012 verification first** — Before writing `evaluate_session_ready`, perform the four ADR-012 verification checks against Bevy 0.18:
1. `Commands::trigger(SessionReady)` runs the registered observer in the **same** Update tick (not deferred).
2. A resource inserted via `Commands::insert_resource` in the same system that calls `Commands::trigger` is visible to the observer.
3. The observer handler signature is `fn(trigger: Trigger<SessionReady>, ...)`.
4. `World::trigger` is the appropriate fallback for an exclusive system, if needed.

If any check fails: implement `evaluate_session_ready` as `fn(world: &mut World)` and replace `commands.insert_resource(...)` with `world.insert_resource(...)`, `commands.trigger(...)` with `world.trigger(...)`. The Observer handler and registration are unchanged.

**One-active-session enforcement (Rule 13)** — A `PlayerId` map is maintained at the server level (across all sessions), not per-session. Idempotent retry uses this map to find the player's existing `LOBBY_WAITING` session and return its room code. Any other state returns `S2CCreateRoomRejected { reason: AlreadyInSession }`.

**Heartbeat tracker hand-off** — The GSS owns heartbeats from slot occupation until `SessionReady` fires. After `SessionReady`, the RSM owns `disconnect_trackers` with the 30s grace. The two trackers must NOT overlap. The `on_session_ready` Observer (or the Game Session plugin, depending on cleanest implementation) clears `LobbyHeartbeats` and copies the current connection state into `RoundState.disconnect_trackers`.

**Test isolation via mock `ServerRng`** — For deterministic tests, `evaluate_session_ready` accepts a `Resource` providing the `ServerRng` factory (default impl uses `OsRng`; tests inject a fixed seed). This avoids OS entropy variance breaking tests.

## Definition of Done

- All deliverables above implemented and passing.
- All ADR-012 validation criteria pass: GSS-29, GSS-30, single-fire invariant, resource order invariant, RSM-1 (LOBBY → DRAFT_INITIAL same tick), no-second-observer rule.
- ADR-012 verification checks (1–4) documented with results in an implementation note (`docs/architecture/adr-012-session-ready-delivery.md` "Last Verified" date updated, or implementation note in this epic file marking which path — Commands or World fallback — was taken).
- `cargo check --workspace` green; zero warnings on `server/src/core/session/**`.
- CI grep gate: `grep -r "EventReader<SessionReady>" server/src/` returns zero matches (must use Observer).
- CI grep gate: `grep -r "app.add_event::<SessionReady>" server/src/` returns zero matches (must use `app.observe`).
- CI grep gate: `grep -r "app.observe(on_session_ready" server/src/` returns exactly ONE match (single Observer invariant).
- An integration test demonstrates the full LOBBY → DRAFT_INITIAL path: room created, second player joins, both players confirm class, `SessionReady` triggers, RSM transitions, `S2CPhaseChanged(DRAFT_INITIAL)` broadcast — all within the same Update run.
- An integration test demonstrates session teardown: `GameOverEmitted` event causes `SessionConfig` and `ServerRng` to be removed from the World; `S2CGameOver` broadcast on `ReliableChannel`.
- Reconnect integration test: `S2CGameSnapshot` is sent before any live message after a reconnect; live messages queued during the snapshot window are delivered after `snapshot_sent[player] = true`.

## Story Breakdown Hint

Suggested decomposition (final story list to be authored via `/create-stories`):

1. **Lobby scaffold** (Config/Data) — `state.rs`, `LobbyState` enum, `SessionSlot`, `SessionConfig`, `events.rs` with `SessionReady` marker.
2. **Room create + join** (Integration) — C2S handlers; `S2CRoomCreated`, `S2CJoinAck`, `S2CSlotUpdated`; one-active-session enforcement; idempotent retry.
3. **Class selection + reveal** (Logic) — `C2SSelectClass`, `C2SConfirmClass`; deferred simultaneous reveal (`S2CClassLocked` unicast + `S2CClassesRevealed` broadcast on all-locked).
4. **F4 + SessionReady trigger** (Logic) — `evaluate_session_ready`; ADR-012 verification (Commands path or exclusive-system fallback); `on_session_ready` observer; `LobbyState::GameActive` single-fire guard. **This is the highest-risk story** — block on ADR-012 verification before merge.
5. **Lobby disconnect (dual-signal)** (Integration) — Lightyear `OnDisconnected` immediate cancel; heartbeat gap fallback (15s); lobby timeout cancel.
6. **GAME_OVER teardown** (Integration) — Subscribe to `GameOverEmitted` from Epic 1; broadcast `S2CGameOver`; destroy `SessionConfig` + `ServerRng`.
7. **Reconnect + snapshot** (Integration) — `SessionToken` handshake; `S2CGameSnapshot` builder with secret-stripping (ADR-011); live-message queue gated on `snapshot_sent`.

## Next Step

Run `/create-stories production/epics/game-session-system/EPIC.md` to author the story files. **Before authoring Story 4 (F4 + SessionReady trigger)**, complete the ADR-012 verification checks against Bevy 0.18 and document results inline. Story 4 is gated on this verification — it cannot be marked Ready until the four checks are confirmed.
