# PLAYABLE-001 Primary Client Bootstrap And Fresh Lobby Entry Evidence

Date: 2026-05-06
Scope: Sprint 7 friend-game internal path only
Story: `production/epics/playable-client/story-001-primary-client-bootstrap-fresh-lobby-entry.md`

## Scope Boundary

This evidence covers PLAYABLE-001 only: primary client bootstrap, fresh hello identity mapping, lobby create/join/class confirmation messages, authoritative lobby state, server-confirmed `ClientState::InSession` entry, and the Lightyear heartbeat path.

It does not claim PLAYABLE-002 draft/shop/hand bridging, PLAYABLE-003 two-real-client end-to-end evidence, public release readiness, broad accessibility completion, playtest validation, or full playable-client manual QA.

QA-COND-0005 and QA-COND-0006 remain accepted-risk context, not blockers for this friend-game lobby-entry story.

## Protocol Note

`S2CHandshake` now carries `player_id` because the fresh primary-client hello path needs a server-assigned identity before lobby C2S messages can be attributed, and the client must not infer identity from room codes, tokens, or local assumptions.

The existing channel assignment remains unchanged: `C2SHello` and `S2CHandshake` stay reliable; `C2SHeartbeat` remains unreliable.

## Evidence Expectations

Automated evidence for this worker branch should include:

- Fresh `C2SHello { session_token: None }` maps a Lightyear `PeerId` to a stable `PlayerId` and returns `S2CHandshake`.
- Mapped fresh players can create a room, join by room code/slot, select class, and confirm class through existing authoritative server functions.
- Lobby client state updates room code, slots, class lock, and class reveal only from S2C messages.
- `ClientState::InSession` is eligible only after server phase/snapshot state and a known server-assigned identity.
- Heartbeat timer compiles through the Lightyear 0.26 `MessageSender<C2SHeartbeat>.send::<UnreliableChannel>` path.

Manual friend-game evidence remains deferred to PLAYABLE-003. PLAYABLE-001 may record that the lobby-entry path is implemented and tested, but must not claim a completed two-real-client friend-game critical path.

## Commands To Record

Focused PLAYABLE-001 checks:

```powershell
cargo test -p server --test playable_client_lobby_entry_server_test
cargo test -p client --test playable_client_lobby_entry_test
```

Relevant regressions:

```powershell
cargo test -p server --test room_create_join_test
cargo test -p server --test class_reveal_test
cargo test -p server --test lobby_to_draft_initial_test
cargo test -p server --test reconnect_snapshot_test
cargo test -p server --test e2e_websocket_test --no-run
cargo test -p client --test presentation_plugin_scaffold_test
```

Format/checks:

```powershell
cargo fmt -p client -p server -- --check
cargo check -p client
cargo check -p server
git diff --check origin/main...HEAD
```

## Result

Worker branch evidence document created. The following checks passed on the PLAYABLE-001 worker branch before commit:

- `cargo test -p server --test playable_client_lobby_entry_server_test`
- `cargo test -p client --test playable_client_lobby_entry_test`
- `cargo test -p server --test room_create_join_test --test class_reveal_test --test lobby_to_draft_initial_test --test reconnect_snapshot_test`
- `cargo test -p server --test e2e_websocket_test --no-run`
- `cargo test -p client --test presentation_plugin_scaffold_test`
- `cargo fmt -p client -p server -- --check`
- `cargo check -p client`
- `cargo check -p server`
- `git diff --check`
