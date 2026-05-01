# Story 001: Lobby Scaffold

> **Epic**: Game Session System
> **Status**: Complete
> **Layer**: Core
> **Type**: Config/Data
> **Manifest Version**: 2026-04-29

## Context

**GDD**: `design/gdd/game-session-system.md`
**Requirement**: TR-GSS-01, TR-GSS-02, TR-GSS-03, TR-GSS-05, TR-GSS-09, TR-GSS-10

**ADR Governing Implementation**: ADR-012 (SessionReady Delivery), ADR-005 (Server-side RNG)
**ADR Decision Summary**: `SessionReady` is an Observer trigger (zero-sized marker), not a buffered Event. `SessionConfig` and `ServerRng` are inserted immediately before the trigger fires. All types established here become the foundation every downstream GSS story depends on.

**Engine**: Bevy 0.18 | **Risk**: LOW
**Engine Notes**: `#[derive(Resource)]` and `#[derive(Event)]` are stable Bevy 0.18 APIs. `SessionReady` uses Observer dispatch — do NOT use `app.add_event::<SessionReady>()`. `liv-bevy-018` skill is mandatory on all `.rs` files in this story.

**Control Manifest Rules (Core layer)**:
- Required: `SessionReady` doc comment must explicitly state it is an Observer trigger and that `EventReader<SessionReady>` will never fire.
- Required: All tuning knobs (`lobby_timeout_seconds`, `lobby_heartbeat_timeout_seconds`) must be read from `Res<GameConfig>`, never hardcoded.
- Forbidden: No `app.add_event::<SessionReady>()` call anywhere in the codebase.

---

## Acceptance Criteria

- [x] `server/src/core/session/state.rs` exists and contains all of the following:
  - `SessionSlot { index: u8, team: TeamId, player: Option<PlayerId>, class: Option<ClassId> }` — derives `Debug, Clone, PartialEq`
  - `LobbyState` enum with exactly 5 variants: `LobbyWaiting`, `LobbyReady`, `GameActive`, `LobbyCancelled`, `GameOver` — derives `Debug, Clone, PartialEq, Resource`
  - `SessionId(Uuid)` newtype — derives `Debug, Clone, Copy, PartialEq, Eq, Hash`
  - `RoomCode(String)` newtype — derives `Debug, Clone, PartialEq, Eq, Hash`
  - `SessionToken` type alias: `pub type SessionToken = [u8; 16]`
  - `SessionSlots(Vec<SessionSlot>)` newtype resource — derives `Debug, Resource`
  - `ClassSelections(HashMap<PlayerId, ClassId>)` newtype resource — derives `Debug, Resource`
  - `LobbyDeadline(f64)` newtype resource — derives `Debug, Clone, Copy, Resource`
  - `LobbyHeartbeats(HashMap<PlayerId, f64>)` newtype resource — derives `Debug, Resource`
- [x] `server/src/core/session/config.rs` exists and contains:
  - `SessionConfig { mode: GameMode, player_count: u8, team_map: HashMap<PlayerId, TeamId>, class_map: HashMap<PlayerId, ClassId> }` — derives `Debug, Clone, Resource`
  - `build_session_config(slots: &SessionSlots, selections: &ClassSelections) -> SessionConfig` — panics if any occupied slot has `class = None`
- [x] `server/src/core/session/events.rs` exists and contains:
  - `SessionReady` zero-sized struct — `#[derive(Event)]` — doc comment explicitly states: "DELIVERY: Observer trigger (same-frame). NOT a buffered Event. Subscribe via `app.observe(on_session_ready)`. Adding `EventReader<SessionReady>` will silently never fire."
  - `SessionCancelled { reason: SessionCancelledReason }` — `#[derive(Event)]`
  - `SessionCancelledReason` enum with at least: `PlayerDisconnected`, `HeartbeatTimeout`, `LobbyTimeout`, `RngInitFailure` variants
- [x] `server/src/core/session/plugin.rs` exists and contains a skeleton `GameSessionPlugin` struct with `impl Plugin for GameSessionPlugin` that compiles (systems can be empty stubs in this story)
- [x] `server/src/core/session/mod.rs` exists and re-exports `state`, `config`, `events`, `plugin`
- [x] `build_session_config` panics with a descriptive message when called with a slot that has `player = Some(_)` but `class = None`
- [x] `build_session_config` sets `player_count` equal to the number of occupied slots (those with `player = Some(_)`)
- [x] `SessionReady` doc comment is present and contains the literal text "EventReader<SessionReady> will silently never fire" (CI grep gate)
- [x] `cargo check -p server` passes with zero warnings on all new files
- [x] Smoke test in `tests/unit/session/scaffold_test.rs` passes: constructs each new type, verifies `LobbyState::LobbyWaiting != LobbyState::GameActive`, calls `build_session_config` with a valid two-slot setup and asserts `player_count == 2`

---

## Implementation Notes

*Derived from EPIC.md §Scope (Deliverables) and ADR-012 §Key Interfaces:*

**`uuid` crate dependency**: `SessionId` wraps `Uuid`. Add `uuid = { version = "1", features = ["v4"] }` to `server/Cargo.toml`. Confirm the workspace does not already pull in a conflicting version.

**`ClassId`, `TeamId`, `GameMode` import**: These types are defined in `shared/` (workspace-and-shared-types Story 002 and 003). Import from `shared::types` — do not redefine locally.

**`PlayerId` import**: Defined in `shared/` (workspace-and-shared-types Story 002). Import from `shared::types`.

**`LobbyState` as a Resource**: The `LobbyState` enum is both an enum (for pattern matching) and a `Resource` (for ECS access via `Res<LobbyState>`). The `#[derive(Resource)]` on the enum itself is the correct pattern here — `evaluate_session_ready` reads it as `Res<LobbyState>` to gate re-evaluation.

**`build_session_config` panic invariant**: The panic message must include the slot index and player ID that violated the invariant, e.g.: `"build_session_config: slot {index} for player {player_id} has no class confirmed — invariant violation"`. This is a programming error, not a runtime condition.

**`SessionToken` type**: `[u8; 16]` — UUID v4 bytes, server-generated. This is the wire type used in `C2SHello` and `S2CHandshake` (defined in `shared/src/protocol.rs` by workspace-and-shared-types Story 004). The alias here is for use within the session module without a protocol import.

**Skeleton `GameSessionPlugin`**: The plugin need only compile at this stage. Stub systems are fine. Full system registration happens across Stories 002–007. The plugin must NOT call `app.add_event::<SessionReady>()` even as a stub — the CI grep gate catches this.

---

## Out of Scope

- Story 002: C2S message handlers (`handle_create_room`, `handle_join_room`)
- Story 003: Class selection and reveal logic
- Story 004: `evaluate_session_ready`, `on_session_ready`, `ServerRng` lifecycle
- Story 005: Disconnect and heartbeat cancel logic
- Story 006: Game-over teardown
- Story 007: Reconnect snapshot path
- `RoundState`, `advance_phase`, RSM types: Epic 1 — Round State Machine
- `ServerRng` struct definition: server-rng Foundation epic Story 001

---

## QA Test Cases

*QL-STORY-READY skipped — Lean mode.*

- **AC: LobbyState equality**
  - Given: `LobbyState::LobbyWaiting` and `LobbyState::GameActive`
  - When: compared with `==`
  - Then: returns `false`

- **AC: build_session_config — valid two-player setup**
  - Given: `SessionSlots` with 2 occupied slots (`player = Some(...)`, `class = Some(...)`)
  - When: `build_session_config(&slots, &selections)` is called
  - Then: Returns `SessionConfig` with `player_count == 2`, `class_map.len() == 2`

- **AC: build_session_config — panics on missing class**
  - Given: `SessionSlots` with 1 occupied slot where `player = Some(p1)` but `class = None`
  - When: `build_session_config(&slots, &selections)` is called in a `#[should_panic]` test
  - Then: Panics with a message containing the slot index

- **AC: SessionReady is zero-sized**
  - Given: `SessionReady`
  - When: `std::mem::size_of::<SessionReady>()` is called
  - Then: Returns `0`

---

## Test Evidence

**Story Type**: Config/Data
**Required evidence**: `tests/unit/session/scaffold_test.rs` — all smoke checks passing; `cargo check -p server` output showing zero warnings
**Status**: [x] Complete. Evidence pointer exists at `tests/unit/session/scaffold_test.rs`; executable test is crate-wired at `server/tests/session_scaffold_test.rs`.

---

## Dependencies

- Depends on: `workspace-and-shared-types` Story 004 (protocol skeleton — `PlayerId`, `ClassId`, `TeamId`, `GameMode`, `SessionId` types)
- Depends on: `game-config-pipeline` (all tuning knobs read from `Res<GameConfig>` — GameConfig struct must exist before plugin compiles)
- Unlocks: Stories 002, 003, 004, 005, 006, 007 (all depend on the types defined here)

## Completion Notes

**Completed**: 2026-05-01
**Criteria**: 10/10 passing
**Deviations**: Advisory only -- story manifest version 2026-04-29 is older than current control manifest version 2026-05-01. Test evidence is split between `tests/unit/session/scaffold_test.rs` (evidence pointer) and `server/tests/session_scaffold_test.rs` (executable Cargo test). `cargo check -p server` passes; current warnings are pre-existing outside the S3-01 session scaffold files.
**Test Evidence**: `cargo test -p server --test session_scaffold_test` passed locally (9/9); `cargo check -p server` passed locally; GitHub Actions run `25194696023` passed for commit `17e3fc352ad1f843daafba4fa8ac484847311f9e`.
**Code Review**: Skipped -- Lean mode.
