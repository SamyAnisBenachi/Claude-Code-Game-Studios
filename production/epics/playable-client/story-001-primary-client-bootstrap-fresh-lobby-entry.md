# Story 001: Primary Client Bootstrap + Fresh Lobby Entry

> **Epic**: Playable Client
> **Status**: Complete
> **Layer**: Polish / Client Integration
> **Type**: Integration
> **Manifest Version**: 2026-05-05
> **Sprint**: Sprint 7 / PLAYABLE-001

## Context

Sprint 7 requires the real primary client path to become usable for an internal friend-game session. Current planning notes identify that `client/src/main.rs` starts networking and presentation plugins, but the primary client has no real lobby/menu flow that sends player intent and transitions into `ClientState::InSession` from server-confirmed state.

**Primary sources**:

- `production/sprints/sprint-7.md`
- `production/sprint-status.yaml`
- `design/gdd/network-protocol.md`
- `design/gdd/game-session-system.md`
- `design/gdd/round-state-machine.md`
- `design/gdd/hud.md`
- `design/gdd/shop-auction-ui.md`
- `design/gdd/hand-ui.md`

**GDD and TR trace**:

- `design/gdd/network-protocol.md` / `TR-NP-001`: server is sole authority; clients hold read-only mirrors and express intent through C2S messages only.
- `design/gdd/network-protocol.md` / `TR-NP-003`: `C2SHello` is the first message on any connection and results in `S2CHandshake` or `S2CHandshakeRejected`.
- `design/gdd/network-protocol.md` / `TR-NP-006`: `S2CGameSnapshot` is unicast per player on every connect or reconnect before live messages become actionable.
- `design/gdd/game-session-system.md` / `TR-GSS-001`: `C2SCreateRoom` creates a session and returns a room code.
- `design/gdd/game-session-system.md` / `TR-GSS-004`: class lock is unicast to the locking player and class reveal broadcasts only when all slots are locked.
- `design/gdd/game-session-system.md` / `TR-GSS-007`: `SessionReady` inserts `SessionConfig`, initializes `ServerRng`, and hands the session to RSM.
- `design/gdd/round-state-machine.md` / `TR-RSM-002`: LOBBY to DRAFT_INITIAL transition is guarded exclusively by `SessionReady`.
- `design/gdd/round-state-machine.md` / `TR-RSM-009`: `S2CPhaseChanged` is the reliable phase broadcast emitted on every transition.

**ADR Governing Implementation**:

- [ADR-002: Client-Server Authority](../../../docs/architecture/adr-002-client-server-authority.md)
- [ADR-003: Cargo Workspace Structure](../../../docs/architecture/adr-003-cargo-workspace-structure.md)
- [ADR-008: Lightyear Channel Configuration](../../../docs/architecture/adr-008-lightyear-channel-config.md)
- [ADR-011: Reconnect Snapshot](../../../docs/architecture/adr-011-reconnect-snapshot.md)
- [ADR-012: SessionReady Delivery](../../../docs/architecture/adr-012-session-ready-delivery.md)
- [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)

**ADR Decision Summary**: The primary client may gather local menu input, but it must only emit C2S intent messages. Player identity, room membership, class locks, phase, snapshot, and transition into `ClientState::InSession` must be driven by server-confirmed S2C state.

**Engine**: Bevy 0.18 + Lightyear 0.26 + browser/WASM primary client | **Risk**: HIGH

**Engine Notes**: Use `liv-bevy-018` before editing any Bevy `.rs` file and `liv-bevy-lightyear` before editing any networking `.rs` file. Primary client UI must use Bevy 0.18 Required Components API. Do not use `NodeBundle`, `TextBundle`, `SpriteBundle`, `Camera2dBundle`, `UiImage::new()`, `Parent`, `EventReader`, `EventWriter`, `Events<T>`, or `Color::rgba()`. Lobby UI state should be resources/components in the client crate, not server imports.

**Lightyear Notes**: `ClientPlugins` must remain registered before protocol registration. `C2SHello`, `C2SCreateRoom`, `C2SJoinRoom`, `C2SSelectClass`, and `C2SConfirmClass` use `ReliableChannel`. The client must not replace the server session system with harness state. Any new receive path must respect one production drainer per `MessageReceiver<T>`.

**Control Manifest Rules (2026-05-05)**:

- Required: `shared/` stays protocol-only and `client/` never depends on `server/`.
- Required: client state is a read-only projection; no optimistic session, room, class, phase, or snapshot authority.
- Required: `S2CPhaseChanged` is drained by the shared phase sink only.
- Required: `PresentationPlugin` order remains Card Animations, Board Rendering, Hand UI, HUD, Shop/Auction UI.
- Required: UI overlays use `bevy_ui`; board content remains world-space.
- Guardrail: client S2C processing plus view update remains at or below 2 ms per frame and presentation phase-boundary spikes remain below 3 ms.

---

## Scope

### In Scope

- Ensure the primary client launches with the real window/browser path and the plugin set needed for friend-game use.
- Send `C2SHello` on a fresh connection using the real Lightyear client path.
- Receive and store `S2CHandshake` and session token data needed by the client session view.
- Provide minimal friend-game lobby UI for create room, join room by code, class select, and class confirm.
- Send `C2SCreateRoom`, `C2SJoinRoom`, `C2SSelectClass`, and `C2SConfirmClass` from user actions.
- Render enough room, slot, class-lock, class-reveal, rejected-create, rejected-join, and cancelled-session feedback for two internal players to coordinate.
- Transition to `ClientState::InSession` only after server-confirmed session state is actionable through `S2CGameSnapshot` or `S2CPhaseChanged(DRAFT_INITIAL)` plus session identity.
- Preserve current Presentation phase sink ownership. This story must not add a second `MessageReceiver<S2CPhaseChanged>` drain.

### Out of Scope

- Public release readiness, store readiness, external matchmaking, auth accounts, invite links, deployment, or reconnect UX polish.
- Broad Standard-tier accessibility work. Only readability issues that directly block friend-game lobby use are in scope.
- Editing `QA-COND-0005` or `QA-COND-0006` accepted-risk disposition.
- Playtest validation, fun-hypothesis validation, full playable-client manual QA, or full QA sign-off.
- Draft/shop/hand behavior after DRAFT_INITIAL starts. PLAYABLE-002 owns that bridge.
- Real end-to-end manual friend-game evidence. PLAYABLE-003 owns that evidence.

---

## Acceptance Criteria

- [x] **Primary client boots on the real path**: GIVEN the primary client is started with the normal client binary or browser/WASM entry, WHEN it reaches the first interactive screen, THEN it includes real networking and presentation plugins, does not rely on harness-injected session state, and keeps a stable friend-game lobby view visible.
- [x] **Fresh hello maps identity**: GIVEN a fresh client connects with no stored session token, WHEN the user reaches the lobby entry screen, THEN the client sends exactly one `C2SHello { protocol_version, session_token: None }` over `ReliableChannel` and records the resulting `S2CHandshake` session token without entering `ClientState::InSession`.
- [x] **Create room uses real C2S/S2C flow**: GIVEN the host clicks Create Room, WHEN `S2CRoomCreated` arrives, THEN the room code, mode, and full slots vector render from that S2C payload and no local room state is treated as authoritative before the message arrives.
- [x] **Join room uses real C2S/S2C flow**: GIVEN the second client enters a room code and clicks Join, WHEN `S2CJoinAck` arrives, THEN the joiner sees the full slots vector from the server and the host receives `S2CSlotUpdated`; rejection cases render from `S2CJoinRejected` without changing confirmed room state.
- [x] **Class confirm is server-confirmed**: GIVEN both clients choose and confirm a class, WHEN the server sends `S2CClassLocked` and then `S2CClassesRevealed`, THEN the UI shows own lock and revealed class map only from those messages.
- [x] **Session entry waits for authoritative phase or snapshot**: GIVEN all required slots are filled and classes are confirmed, WHEN the server transitions out of lobby, THEN `ClientState::InSession` is set only after authoritative `S2CGameSnapshot` or `S2CPhaseChanged(DRAFT_INITIAL)` state is received and the client has a local player identity.
- [x] **No optimistic authority is introduced**: GIVEN any create, join, class, or confirm action is clicked, WHEN the outbound C2S message is queued, THEN no server-owned phase, slot, class lock, room membership, or hand/economy state changes locally until the corresponding S2C state arrives.
- [x] **Friend-game lobby readability is adequate**: GIVEN two internal users run the flow at the supported test viewport, WHEN they create, join, pick classes, and wait for session entry, THEN room code, connection state, slot occupancy, own class lock, revealed class map, and actionable controls are readable without overlapping each other.
- [x] **Regression commands pass**: `cargo test -p client --test playable_client_lobby_entry_test`, `cargo test -p server --test playable_client_lobby_entry_server_test`, `cargo check -p client`, `cargo check -p server`, and `git diff --check` pass.
- [x] **Evidence document exists**: `production/qa/evidence/playable-client-lobby-entry.md` records commit, commands, target, two-client setup, screenshots or captures, observed C2S/S2C messages, pass/fail summary, and friend-game-only scope statement.

---

## Likely Files Touched

- `client/src/main.rs`
- `client/src/lib.rs`
- `client/src/network/mod.rs`
- `client/src/state/mod.rs`
- `client/src/ui/mod.rs`
- `client/src/ui/lobby.rs`
- `client/src/presentation/mod.rs`
- `client/Cargo.toml`
- `server/src/core/session/reconnect.rs`
- `server/src/core/session/system.rs`
- `server/src/core/session/state.rs`
- `server/src/core/session/plugin.rs`
- `server/src/network/mod.rs`
- `server/Cargo.toml`
- `tests/integration/playable_client/lobby_entry_test.rs`
- `tests/integration/playable_client/lobby_entry_server_test.rs`
- `production/qa/evidence/playable-client-lobby-entry.md`

`shared/src/protocol.rs` should not need changes because the required C2S/S2C messages already exist. If implementation proves a protocol field is missing, the developer must keep the change minimal, preserve existing channel assignments, and document the reason in this story evidence.

## Implementation Notes

- Keep lobby UI minimal and functional. Friend-game players need a room code, join field, class choices, class confirm, status feedback, and clear errors. Marketing-style menus and public onboarding copy are out of scope.
- Prefer a new `client/src/ui/lobby.rs` module or the nearest existing UI module that preserves client layering.
- Replace sender stubs only where real user actions need to send messages. Keep compile-proof helpers if they remain useful, but they must not mask absence of real flow.
- Treat `S2CGameSnapshot` as authoritative session data. Do not manufacture snapshots or phase changes in the primary client.
- If both `C2SSelectClass` and `C2SConfirmClass` are exposed, make the confirm button the only action that locks the class for session readiness.
- Rejections and cancellation states should keep the user in the lobby and display a stable retry path.

## Performance Budget

No measurable gameplay-loop performance impact is expected before session entry. Lobby UI and message drains must remain fixed-size over local lobby widgets and server-provided slots. After session entry, this story must preserve the global client S2C processing plus view update budget of at most 2 ms per frame and the ADR-021 phase-boundary presentation spike budget below 3 ms.

---

## QA Test Cases

- **Fresh host path**
  - Given: Local server is running and the primary client starts without a stored session token.
  - When: Host connects, sends hello, creates a room, chooses a class, and confirms it.
  - Then: UI state is derived from `S2CHandshake`, `S2CRoomCreated`, and `S2CClassLocked`; the client remains out of `ClientState::InSession` until server session entry state arrives.

- **Second-client join path**
  - Given: Host room code is visible.
  - When: Second client joins with that code and confirms a class.
  - Then: Joiner receives `S2CJoinAck`, host receives `S2CSlotUpdated`, and both clients receive `S2CClassesRevealed` after all classes are locked.

- **Server-confirmed session entry**
  - Given: Both clients are in the same room and all classes are confirmed.
  - When: `SessionReady` causes the RSM to enter DRAFT_INITIAL.
  - Then: each client enters `ClientState::InSession` from authoritative snapshot or phase state and not from local button clicks.

- **Failure and retry**
  - Given: A client enters an invalid room code or tries a rejected create/join action.
  - When: rejection S2C arrives.
  - Then: confirmed room state is unchanged and the lobby remains usable for another attempt.

---

## Test Evidence

**Story Type**: Integration

**Required automated test targets**:

- `tests/integration/playable_client/lobby_entry_test.rs`
  - Registered as `playable_client_lobby_entry_test`
  - Command: `cargo test -p client --test playable_client_lobby_entry_test`
- `tests/integration/playable_client/lobby_entry_server_test.rs`
  - Registered as `playable_client_lobby_entry_server_test`
  - Command: `cargo test -p server --test playable_client_lobby_entry_server_test`

**Required regression commands**:

- `cargo test -p server --test lobby_to_draft_initial_test`
- `cargo test -p server --test room_create_join_test`
- `cargo test -p server --test class_reveal_test`
- `cargo test -p client --test presentation_plugin_scaffold_test`
- `cargo check -p client`
- `cargo check -p server`
- `git diff --check`

**Required evidence document**:

- `production/qa/evidence/playable-client-lobby-entry.md`

**Final evidence expectations**:

- Exact commit and build target.
- Commands used to run local server and both primary clients.
- Message trace covering `C2SHello`, `S2CHandshake`, create or join, class confirm, class reveal, and session entry trigger.
- Screenshots or captures showing host room, joiner room, class locks, class reveal, and first in-session state.
- Explicit statement that evidence is friend-game lobby/session-entry evidence only, not public release readiness, not playtest validation, not broad accessibility completion, and not full playable-client manual QA.

**Status**: [x] Created and passing.

---

## Dependencies

- Depends on: None for story readiness.
- Implementation depends on the existing accepted Game Session System, Network Protocol, RSM, Presentation Layer, HUD, Hand UI, and Shop/Auction UI contracts on `main`.
- Unlocks: [PLAYABLE-002 Live Draft/Shop/Hand Bridge](story-002-live-draft-shop-hand-bridge.md).

## Blockers

None.

## Completion Notes

**Completed**: 2026-05-06
**Criteria**: 10/10 passing.
**Deviations**: None blocking. Story manifest version `2026-05-05` matches the current control manifest. Lean review mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW because `production/review-mode.txt` is absent.
**Test Evidence**: Integration tests at `tests/integration/playable_client/lobby_entry_test.rs` and `tests/integration/playable_client/lobby_entry_server_test.rs`; story evidence at `production/qa/evidence/playable-client-lobby-entry.md`.
**Verification**: `cargo test -p server --test playable_client_lobby_entry_server_test` passed 3/3; `cargo test -p client --test playable_client_lobby_entry_test` passed 5/5; server regressions `room_create_join_test`, `class_reveal_test`, `lobby_to_draft_initial_test`, and `reconnect_snapshot_test` passed 22/22; `cargo test -p server --test e2e_websocket_test --no-run`, `cargo test -p client --test presentation_plugin_scaffold_test`, `cargo fmt -p client -p server -- --check`, `cargo check -p client`, `cargo check -p server`, and `git diff --check` passed.
**Scope Boundary**: Complete for PLAYABLE-001 lobby/bootstrap only. This does not claim PLAYABLE-002 draft/shop/hand bridge completion, PLAYABLE-003 two-real-client end-to-end evidence, public release readiness, broad accessibility completion, playtest validation, full playable-client manual QA, or a complete primary-client game loop.
**PLAYABLE-002 Impact**: PLAYABLE-002 remains not started and is unblocked for `/dev-story`.
**Code Review**: Skipped per lean review mode.
