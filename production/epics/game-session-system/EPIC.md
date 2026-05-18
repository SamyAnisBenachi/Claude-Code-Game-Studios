# Epic: Game Session System

> **Layer**: Core
> **GDD**: design/gdd/game-session-system.md
> **Architecture Module**: `server/core/session/` (full module — `state.rs`, `events.rs`, `system.rs`, `config.rs`, `plugin.rs`); contributes `on_session_ready` Observer registration to `server/core/rsm/`
> **Status**: Ready
> **Stories**: 10 stories — see Stories section below

## Overview

Implements the lobby finite-state machine and the session-readiness handoff that bridges connection-time concerns to the round loop. This epic owns `SessionSlot`, room creation and join, public class selection with deferred simultaneous reveal, the F4 readiness predicate (all slots filled + all classes confirmed + lobby deadline not expired), lobby/session PLACEMENT timer multiplier negotiation, the lobby heartbeat / `OnDisconnected` immediate-cancel path, and — critically — the `SessionReady` Observer-trigger delivery that hands `SessionConfig` and `ServerRng` to the RSM in the same `Update` tick. After `SessionReady` fires, the GSS becomes a passive read-only configuration store: `Res<SessionConfig>` (mode, player_count, team_map, class_map, placement_timer_multiplier_effective) is the single source of session data for every Feature system. The GSS also owns the `ServerRng` lifecycle: it seeds the RNG from `OsRng` immediately before triggering `SessionReady` (per ADR-005) and destroys both `SessionConfig` and `ServerRng` resources on `GameOverEmitted` (subscribed from Epic 1's RSM event bus). Sprint 9 adds the result acknowledgement/result data contract as Story 009: GSS owns the server-side `C2SAcknowledgeResult` handling, retained GAME_OVER result data during the acknowledgement window, and reconnect behavior needed by the Result Screen MVP. This epic is the load-bearing gate between LOBBY and DRAFT_INITIAL: an Observer-trigger ordering bug here panics the RSM with "resource not found" and breaks every game session.

Story 010 owns the S9-RS-003 cross-boundary Return to Lobby acknowledgement
cleanup handshake after Story 009 and Presentation Layer Story 006 exist.

## Governing ADRs

| ADR | Decision Summary | Engine Risk |
|-----|-----------------|-------------|
| ADR-011: Reconnect Flow and Game Snapshot Protocol | `SessionToken` issued at first connect; `ClientId` re-mapped on every Lightyear transport reconnect; live message queue held until `S2CGameSnapshot` delivery confirmed; secret-stripping rules per player at snapshot send (own vs opponent) | HIGH |
| ADR-012: SessionReady Delivery (Observer, same-frame) | `SessionReady` is delivered via `Commands::trigger(SessionReady)` (Observer pattern), NOT buffered `Events<T>`; `SessionConfig` and `ServerRng` inserted via Commands BEFORE the trigger, in the same system; one Observer only (`on_session_ready` in RSM); exclusive-system `World::trigger` fallback documented if Commands flush ordering cannot be verified | HIGH |
| ADR-023: Placement Timer Accessibility Authority | GSS negotiates the multiplayer PLACEMENT timer multiplier during LOBBY; effective value is the highest requested multiplayer-safe value capped at 3x; value is neutral, frozen into `SessionConfig` at `SessionReady`, and consumed by RSM/client timer display through server-provided phase data | HIGH |

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
| TR-GSS-11 | GSS owns multiplayer PLACEMENT timer multiplier negotiation before `SessionReady`: highest requested multiplayer-safe value wins, capped at 3x, neutral, and frozen into `SessionConfig` | ADR-023 |
| S9-RS-001 | Result acknowledgement and retained GAME_OVER result data: `C2SAcknowledgeResult` is handled server-side during GAME_OVER, duplicate acks are idempotent, reconnect during the retained acknowledgement window receives authoritative final snapshot plus re-sent `S2CGameOver`, and cleanup waits for all acks or `ack_timeout_ms` | ADR-002, ADR-008, ADR-011 |

| S9-RS-003 | Result acknowledgement cleanup handshake: Return to Lobby sends the agreed acknowledgement, clears local ended-session UI, keeps server cleanup authoritative, and verifies duplicate ack, all-ack cleanup, timeout cleanup, and reconnect-after-cleanup fallback end to end | ADR-002, ADR-008, ADR-011, ADR-021 |

## Scope

### Deliverables

**`server/src/core/session/state.rs`**
- `SessionSlot { index: u8, team: TeamId, player: Option<PlayerId>, class: Option<ClassId> }`
- `LobbyState` enum: `LobbyWaiting | LobbyReady | GameActive | LobbyCancelled | GameOver`
- Per-session resources: `SessionSlots(Vec<SessionSlot>)`, `ClassSelections(HashMap<PlayerId, ClassId>)`, `LobbyDeadline(f64)`, `LobbyHeartbeats(HashMap<PlayerId, f64>)`, `LobbyState`
- PLACEMENT timer multiplier lobby resources: per-player `PlacementTimerMultiplier` requests and neutral effective room/session value
- `SessionId(Uuid)`, `RoomCode(String)` newtypes
- `SessionToken` (issued at first `C2SHello`, used by ADR-011 reconnect path)

**`server/src/core/session/events.rs`**
- `SessionReady` — zero-sized marker, `#[derive(Event)]`, doc comment explicitly stating "DELIVERY: Observer trigger (same-frame). NOT a buffered Event. Subscribe via `app.observe(on_session_ready)`. Adding `EventReader<SessionReady>` will silently never fire."
- `SessionCancelled { reason: SessionCancelledReason }` — buffered Event for post-cancel teardown subscribers (logging, etc.)

**`server/src/core/session/config.rs`**
- `SessionConfig { mode: GameMode, player_count: u8, team_map: HashMap<PlayerId, TeamId>, class_map: HashMap<PlayerId, ClassId>, placement_timer_multiplier_effective: PlacementTimerMultiplier }` — `#[derive(Resource, Clone)]`. Inserted ONCE at `SessionReady` time. Never mutated after insertion. Removed by GSS on `GameOverEmitted`.
- `build_session_config(slots: &SessionSlots, selections: &ClassSelections) -> SessionConfig` — panics if any occupied slot has `class = None` (GDD Rule 11 invariant; ADR-012 Verification Required item).

**`server/src/core/session/system.rs`**
- `handle_create_room(C2SCreateRoom)` — assigns `session_id`, generates 6-char room code, initialises slots per mode, sets `lobby_deadline = now + lobby_timeout_seconds`. Includes the GDD Rule 13 idempotent-retry path: same player resending `C2SCreateRoom` for an existing `LOBBY_WAITING` session returns the existing room code.
- `handle_join_room(C2SJoinRoom)` — slot validation; on success: `S2CJoinAck` to joiner (full slot state), `S2CSlotUpdated` broadcast to others (full slot vector — never deltas). One-active-session check per Rule 13 (`AlreadyInSession` rejection).
- `handle_select_class(C2SSelectClass)` — updates preview only; not broadcast.
- `handle_confirm_class(C2SConfirmClass)` — same-system sequential write of `SessionSlot.class = Some(_)` and `class_selections[player_id] = class_id`; `S2CClassLocked` unicast to locking player; if all slots locked → `S2CClassesRevealed` broadcast.
- `handle_set_placement_timer_multiplier(C2SSetPlacementTimerMultiplier)` — valid only in LOBBY before `SessionReady`; accepts multiplayer-safe values `1x`, `1.5x`, `2x`, and `3x`; computes the neutral effective value as highest request capped at 3x; broadcasts `S2CSessionSettingsUpdated` without requester attribution; requests after `SessionReady` do not mutate active `SessionConfig`.
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
- `handle_game_over_teardown` — subscribes to `MessageReader<GameOverEmitted>` (from Epic 1 RSM message bus); broadcasts `S2CGameOver { loser, round, reason }` on `ReliableChannel`; removes `SessionConfig` and `ServerRng` resources from world; transitions `LobbyState` to `GameOver`. The GAME_OVER → session destruction path lives here per ADR-010 subscriber contract.
- `handle_result_acknowledgement` — drains `C2SAcknowledgeResult` during GAME_OVER only; resolves the sender to a stable `PlayerId`; records idempotent acknowledgement in retained ended-session result state; silently discards invalid-phase, unknown-sender, stale-token, and non-participant messages; triggers terminal cleanup once all result participants acknowledge or `ack_timeout_ms` expires.

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
- `S2CGameSnapshot` builder is implemented here (it touches `Res<RoundState>`, `Res<SessionConfig>`, `Res<PlayerEconomy>`, `Res<HiddenObjectives>`, `Res<BoardGrid>`, `Res<PlayerPool>` — broadest cross-system read in the codebase). Snapshot data includes the frozen neutral `placement_timer_multiplier_effective` and never attributes the requester.
- During the retained GAME_OVER acknowledgement window, reconnect uses the final per-player snapshot retained at GAME_OVER plus a re-sent retained `S2CGameOver`. This keeps result-screen reconnect authoritative without requiring `S2CGameSnapshot` to grow `loser`, `round`, or `reason` fields.

**Tests**
- `tests/unit/session/` — F4 predicate truth table (all combinations of slot fill, class confirm, deadline expiry).
- `tests/unit/session/session_ready_test.rs` — GSS-30 invariant: after `evaluate_session_ready` runs, `Res<SessionConfig>` and `Res<ServerRng>` exist with correct values, `RoundPhase == DraftInitial`, observer fired exactly once. All asserted in the same test tick.
- `tests/unit/session/single_fire_test.rs` — running `evaluate_session_ready` 3 ticks after the first trigger: trigger count remains 1.
- `tests/unit/session/rng_init_failure_test.rs` — inject failing `ServerRng::new`: `SessionReady` is NOT triggered, `RoundPhase` stays LOBBY, `LobbyState == LobbyCancelled` (GSS-29).
- `tests/unit/session/class_reveal_test.rs` — both players confirm: `S2CClassLocked` to each player only; `S2CClassesRevealed` broadcast only after second confirm (Rule 7).
- `tests/unit/session/dual_signal_disconnect_test.rs` — Lightyear `OnDisconnected` cancels immediately; heartbeat gap > 15s cancels via fallback path; first signal wins.
- `tests/integration/session/reconnect_snapshot_test.rs` — full reconnect flow: live message queue held until snapshot sent (ADR-011 invariant).
- `tests/unit/session/placement_timer_multiplier_test.rs` — ADR-023 GSS coverage: highest request wins, no request defaults to 1x, no post-`SessionReady` mutation, neutral `S2CSessionSettingsUpdated`.
- `tests/unit/rsm/rsm_placement_timer_multiplier_test.rs` — ADR-023 RSM coverage: standard and auction-followup PLACEMENT base durations multiplied by frozen `SessionConfig.placement_timer_multiplier_effective`.
- `tests/integration/hand-ui/server_timer_duration_test.rs` — Hand UI initializes PLACEMENT timer from server-provided phase/snapshot duration rather than the local 10s default.

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
- Result acknowledgement integration tests demonstrate GAME_OVER-only ack handling, duplicate ack idempotence, all-ack cleanup, `ack_timeout_ms` cleanup, retained final snapshot plus `S2CGameOver` resend on GAME_OVER reconnect, and post-cleanup reconnect fallback.
- ADR-023 validation passes: `C2SSetPlacementTimerMultiplier`, `S2CSessionSettingsUpdated`, `PlacementTimerMultiplier { 1x, 1.5x, 2x, 3x }`, frozen `SessionConfig.placement_timer_multiplier_effective`, RSM effective PLACEMENT duration, snapshot frozen multiplier, and Hand UI server timer consumption are implemented and tested.
- No multiplayer Standard-tier `0.5x` timer option exists, and no S2C/session settings payload exposes requester identity.

## Stories

| # | Story | Type | Status | Primary ADR |
|---|-------|------|--------|-------------|
| 001 | Lobby Scaffold | Config/Data | Ready | ADR-012, ADR-005 |
| 002 | Room Create and Join | Integration | Ready | ADR-008, ADR-002 |
| 003 | Class Selection and Reveal | Logic | Ready | ADR-008 |
| 004 | F4 Predicate and SessionReady Trigger | Logic | **Blocked** | ADR-012, ADR-005, ADR-009 |
| 005 | Lobby Disconnect — Dual-Signal Cancel | Integration | Ready | ADR-011, ADR-008 |
| 006 | Game-Over Teardown | Integration | Ready | ADR-010, ADR-005, ADR-008 |
| 007 | Reconnect and Game Snapshot | Integration | Ready | ADR-011, ADR-001, ADR-008, ADR-002 |
| 008 | PLACEMENT Timer Multiplier Authority | Integration | Ready | ADR-023, ADR-002, ADR-009, ADR-012, ADR-021 |
| 009 | [Result Acknowledgement and Result Data Contract](story-009-result-acknowledgement-and-result-data-contract.md) | Integration | Ready | ADR-002, ADR-008, ADR-010, ADR-011 |
| 010 | [Result Acknowledgement Cleanup Handshake](story-010-result-acknowledgement-cleanup-handshake.md) | Integration | Blocked - depends on S9-RS-001 and S9-RS-002 | ADR-002, ADR-008, ADR-011, ADR-021 |
| 012 | [S18-OPPONENT-DISCONNECT-BROADCAST-001 -- Wire Server Send-Site for S2COpponentDisconnected (F-01 Close)](story-012-opponent-disconnect-broadcast.md) | Logic + Integration | Draft -- Sprint 18 candidate / retro paperwork (landed at commit `dbacb85`, PROMPT 1211); NOT activated | ADR-002, ADR-008 |
| 013 | [S18-SESSION-SETTINGS-ON-JOIN-001 -- Unicast S2CSessionSettingsUpdated to Joiner (F-03 Close)](story-013-session-settings-on-join.md) | Logic + Integration | Draft -- Sprint 18 candidate / retro paperwork (landed at commit `6a18c78`, PROMPT 1212); NOT activated | ADR-002, ADR-008 |

> Stories 012 and 013 are PROMPT 1296 retro paperwork stubs covering Sprint 18 candidate F-01 / F-03 protocol-orphan closures that already landed on `origin/main`. They are NOT folded into the active story counts above; `/story-done` paperwork after Sprint 18 activation will reconcile the table.

> ⚠️ Story 004 is **Blocked** pending ADR-012 verification (Commands::trigger ordering invariant — 4 checklist items must be confirmed against Bevy 0.18). Run the verification spike before Story 004 can be marked Ready.

## Story Breakdown Hint

Suggested decomposition (final story list to be authored via `/create-stories`):

1. **Lobby scaffold** (Config/Data) — `state.rs`, `LobbyState` enum, `SessionSlot`, `SessionConfig`, `events.rs` with `SessionReady` marker.
2. **Room create + join** (Integration) — C2S handlers; `S2CRoomCreated`, `S2CJoinAck`, `S2CSlotUpdated`; one-active-session enforcement; idempotent retry.
3. **Class selection + reveal** (Logic) — `C2SSelectClass`, `C2SConfirmClass`; deferred simultaneous reveal (`S2CClassLocked` unicast + `S2CClassesRevealed` broadcast on all-locked).
4. **F4 + SessionReady trigger** (Logic) — `evaluate_session_ready`; ADR-012 verification (Commands path or exclusive-system fallback); `on_session_ready` observer; `LobbyState::GameActive` single-fire guard. **This is the highest-risk story** — block on ADR-012 verification before merge.
5. **Lobby disconnect (dual-signal)** (Integration) — Lightyear `OnDisconnected` immediate cancel; heartbeat gap fallback (15s); lobby timeout cancel.
6. **GAME_OVER teardown** (Integration) — Subscribe to `GameOverEmitted` from Epic 1; broadcast `S2CGameOver`; destroy `SessionConfig` + `ServerRng`.
7. **Reconnect + snapshot** (Integration) — `SessionToken` handshake; `S2CGameSnapshot` builder with secret-stripping (ADR-011); live-message queue gated on `snapshot_sent`.
8. **PLACEMENT timer multiplier authority** (Integration) — ADR-023 protocol, GSS request negotiation, `SessionConfig` freeze, RSM effective PLACEMENT duration, reconnect snapshot field, and Hand UI server-provided timer consumption.
9. **Result acknowledgement + result data contract** (Integration) - S9-RS-001 server acknowledgement handling, retained final result data, GAME_OVER reconnect resend, idempotent ack safety, and ack-timeout cleanup.
10. **Result acknowledgement cleanup handshake** (Integration) - S9-RS-003 Return to Lobby acknowledgement dispatch, local ended-session UI cleanup, duplicate ack idempotence, all-ack cleanup, timeout cleanup, and reconnect-after-cleanup fallback verification.

## Next Step

Run `/create-stories production/epics/game-session-system/EPIC.md` to author the story files. **Before authoring Story 4 (F4 + SessionReady trigger)**, complete the ADR-012 verification checks against Bevy 0.18 and document results inline. Story 4 is gated on this verification — it cannot be marked Ready until the four checks are confirmed.
