# Story 002: Room Create and Join

> **Epic**: Game Session System
> **Status**: Complete
> **Layer**: Core
> **Type**: Integration
> **Manifest Version**: 2026-04-29

## Context

**GDD**: `design/gdd/game-session-system.md`
**Requirement**: TR-GSS-10 (one active session per player; idempotent retry), TR-GSS-09 (slot state broadcast)

**ADR Governing Implementation**: ADR-008 (Lightyear Channel Config), ADR-002 (Client-Server Authority)
**ADR Decision Summary**: All S2C lobby messages use `ReliableChannel`. The server is the sole authority on slot state — clients receive the full slot vector on any change, never deltas. `S2CJoinAck` is unicast to the joining player; `S2CSlotUpdated` is broadcast to all session participants. Room codes are 6-character strings generated server-side.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: MEDIUM
**Engine Notes**: `NetworkTarget::Single(ClientId)` unicast variant must be verified against Lightyear 0.26 docs before implementation (ADR-011 Engine Compatibility checklist item 2 — may be `NetworkTarget::Only(vec![client_id])`). `MessageSender<T>` is the Lightyear 0.26 API for sending S2C messages from server systems. `liv-bevy-018` and `liv-bevy-lightyear` skills are mandatory on all `.rs` files in this story.

**Control Manifest Rules (Core layer)**:
- Required: One active session per `PlayerId` — enforced via a server-level `HashMap<PlayerId, SessionId>` resource (not per-session).
- Required: `S2CSlotUpdated` carries the full `Vec<SessionSlot>`, never a delta.
- Required: Idempotent `C2SCreateRoom` retry path: same player, same `LOBBY_WAITING` session → return existing room code.
- Forbidden: No client-side room code generation.

---

## Acceptance Criteria

- [x] `handle_create_room` system exists in `server/src/core/session/system.rs` and:
  - Generates a 6-character alphanumeric uppercase room code (e.g. `"G7TK2M"`) using server-side generation (no client input)
  - Assigns a new `SessionId` (UUID v4)
  - Initialises `SessionSlots` per `GameMode` (2 slots for `OneVOne`)
  - Sets `LobbyDeadline(now + game_config.lobby_timeout_seconds as f64)`
  - Initialises `LobbyHeartbeats` with the creating player's `PlayerId` at `now`
  - Sends `S2CRoomCreated { session_id, room_code, slots }` unicast to the creating player on `ReliableChannel`
  - Idempotent retry: if the requesting `PlayerId` already owns a session in `LobbyState::LobbyWaiting`, returns `S2CRoomCreated` with the existing `session_id` and `room_code` — does not create a second session
  - If the requesting `PlayerId` already owns a session in any state other than `LobbyState::LobbyWaiting`, sends `S2CCreateRoomRejected { reason: AlreadyInSession }` and returns
- [x] `handle_join_room` system exists in `server/src/core/session/system.rs` and:
  - Looks up the session by `RoomCode`; sends `S2CJoinRejected { reason: RoomNotFound }` if not found
  - Sends `S2CJoinRejected { reason: SessionFull }` if all slots are occupied
  - Sends `S2CJoinRejected { reason: SessionNotJoinable }` if `LobbyState` is not `LobbyWaiting`
  - Sends `S2CJoinRejected { reason: AlreadyInSession }` if the joining `PlayerId` is already in any active session
  - On success: assigns the joining player to the first empty slot, inserts `PlayerId` into `LobbyHeartbeats` at `now`
  - On success: sends `S2CJoinAck { session_id, room_code, slots }` unicast to the joining player on `ReliableChannel`
  - On success: broadcasts `S2CSlotUpdated { slots }` (full `Vec<SessionSlot>`) to all session participants including the joining player on `ReliableChannel`
- [x] A server-level `ActiveSessions(HashMap<PlayerId, SessionId>)` resource exists, is initialised at server startup, and is updated by both `handle_create_room` and `handle_join_room`
- [x] Room code generation produces only uppercase alphanumeric characters (A–Z, 0–9) and is exactly 6 characters
- [x] `GameSessionPlugin` registers both handlers in the Bevy `Update` schedule
- [x] `cargo check -p server` passes with zero warnings
- [x] Integration test in `tests/integration/session/room_create_join_test.rs` covers:
  - Happy path: create room → join room → assert both players in slots, `LobbyState == LobbyWaiting`
  - Idempotent create: same player sends `C2SCreateRoom` twice → same `session_id` returned
  - Full session rejection: third player attempts to join a two-player session → `SessionFull`
  - Already-in-session rejection: player already in a session attempts to create a new one → `AlreadyInSession`

---

## Implementation Notes

*Derived from EPIC.md §Scope (Deliverables — handle_create_room, handle_join_room) and GDD Rules 13:*

**Room code generation**: Use the `ServerRng` if available, or fall back to `OsRng`-seeded generation at lobby create time. Room codes do not require cryptographic randomness — they are short-lived UX identifiers. 6 alphanumeric uppercase characters gives 36^6 ≈ 2.18 billion combinations — sufficient for a single-server deployment.

**`ActiveSessions` resource scope**: This resource is global to the server process, not scoped per session. A `PlayerId` maps to at most one `SessionId` at any time. It must be cleaned up by `handle_game_over_teardown` (Story 006) and by `handle_lobby_disconnect` (Story 005). Failing to clean it up creates a permanently blocked player — test this path explicitly.

**`S2CSlotUpdated` broadcast scope**: "All session participants" means all `ClientId`s that have joined this specific session (tracked in the session's participant list), not all connected clients. The broadcast targets only the players in this room.

**Lightyear unicast pattern**: At the time of writing, ADR-011 Engine Compatibility checklist item 2 (unicast variant name) is unverified. The implementation note in ADR-011 says it may be `NetworkTarget::Single(ClientId)` or `NetworkTarget::Only(vec![client_id])`. Resolve this before writing any `MessageSender` call. Document the verified variant in a comment at the call site.

**Message types**: `C2SCreateRoom`, `C2SJoinRoom`, `S2CRoomCreated`, `S2CCreateRoomRejected`, `S2CJoinAck`, `S2CJoinRejected`, `S2CSlotUpdated` are defined in `shared/src/protocol.rs` (workspace-and-shared-types Story 004). Do not redefine locally.

**Session participant tracking**: The session needs a list of `ClientId`s for broadcasting slot updates within the session. Store this in a `SessionParticipants(HashMap<PlayerId, ClientId>)` resource or as a field on the session entity. Update it in both `handle_create_room` and `handle_join_room`. Story 007 (reconnect) will update the `ClientId` on reconnect via this same map.

---

## Out of Scope

- Class selection and confirm (Story 003)
- `evaluate_session_ready` F4 predicate (Story 004)
- Disconnect cancel path (Story 005)
- Game-over teardown (Story 006)
- Reconnect handshake (Story 007)
- Room code collision handling for multi-room server deployments (post-MVP)

---

## QA Test Cases

*QL-STORY-READY skipped — Lean mode.*

- **AC: create room — happy path**
  - Given: Player A sends `C2SCreateRoom`
  - When: `handle_create_room` processes the message
  - Then: `S2CRoomCreated` unicast to A; `ActiveSessions` contains `player_a → session_id`; `LobbyState == LobbyWaiting`; room code is 6 uppercase alphanumeric characters

- **AC: join room — happy path**
  - Given: Player A has created a room; Player B sends `C2SJoinRoom { room_code }`
  - When: `handle_join_room` processes the message
  - Then: `S2CJoinAck` unicast to B; `S2CSlotUpdated` sent to both A and B; both slots occupied in `SessionSlots`

- **AC: join room — room not found**
  - Given: No session with code `"XXXXXX"` exists
  - When: Player B sends `C2SJoinRoom { room_code: "XXXXXX" }`
  - Then: `S2CJoinRejected { reason: RoomNotFound }` unicast to B; session state unchanged

- **AC: idempotent create**
  - Given: Player A already owns a `LobbyWaiting` session with `room_code == "ABC123"`
  - When: Player A sends `C2SCreateRoom` again
  - Then: `S2CRoomCreated { room_code: "ABC123" }` unicast to A; no new session created; `ActiveSessions` unchanged

- **AC: already-in-session rejection on create**
  - Given: Player A is in a session with `LobbyState::LobbyReady`
  - When: Player A sends `C2SCreateRoom`
  - Then: `S2CCreateRoomRejected { reason: AlreadyInSession }` unicast to A

---

## Test Evidence

**Story Type**: Integration
**Required evidence**: `tests/integration/session/room_create_join_test.rs` — all test cases passing
**Status**: [x] Passing locally (`cargo test --test room_create_join_test`: 7 passed; `cargo check -p server`: passed with zero warnings)

---

## Dependencies

- Depends on: Story 001 (all session types — `SessionSlot`, `LobbyState`, `SessionId`, `RoomCode`, `SessionSlots`, etc.)
- Unlocks: Story 003 (class selection requires room + slot setup), Story 005 (disconnect cancel requires participant tracking established here)

## Completion Notes

**Completed**: 2026-05-01
**Criteria**: 7/7 passing
**Deviations**: Advisory only - story manifest v2026-04-29 is older than current control manifest v2026-05-01. Current GDD/registry supersedes the story wording for slot updates: joiner receives only `S2CJoinAck`; existing occupants receive `S2CSlotUpdated`. Implementation matches the current rule.
**Test Evidence**: Integration test at `tests/integration/session/room_create_join_test.rs`; `cargo test --test room_create_join_test` passed 7/7 and `cargo check -p server` passed with zero warnings.
**Code Review**: Skipped - Lean mode.
