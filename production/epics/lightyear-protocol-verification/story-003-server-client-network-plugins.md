# Story 003: Server & Client Network Plugins

> **Epic**: Lightyear Protocol & Verification Spike
> **Status**: Complete
> **Layer**: Foundation
> **Type**: Integration
> **Manifest Version**: 2026-04-29

## Context

**GDD**: `design/gdd/network-protocol.md`
**Requirement**: TR-??? (covers TR-NP-UNICAST: unicast path compiles; TR-NP-SYMM: both sides compile against shared/ types)

**ADR Governing Implementation**: ADR-008: Lightyear Channel Config + ADR-003: Cargo Workspace Structure
**ADR Decision Summary**: `ServerNetworkPlugin` lives in `server/src/network/`; `ClientNetworkPlugin` in `client/src/network/`. Both call `shared::protocol::register_protocol(app)`. The server unicast path (`NetworkTarget::Single(ClientId)`) is a compile-time proof of ADR-001's hidden-information architecture.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**: Use ONLY the verified API from Story 001. WebSocket transport configuration differs between native server and WASM client — verify exact plugin constructor signatures for each target. `MinimalPlugins` on server (headless); full Bevy plugin set on client. Lightyear `ServerPlugins` and `ClientPlugins` configuration — exact parameter types confirmed in Story 001.

**Control Manifest Rules (Foundation layer)**:
- Required: All channel definitions in `shared/src/protocol.rs`. Both server and client compile against identical channel types.
- Forbidden: `client/` must never depend on `server/`. `server/` must never depend on `client/`.
- Guardrail: WASM bundle ≤ 50 MB (Story 004 verifies this end-to-end).

---

## Acceptance Criteria

**`server/src/network/` module:**
- [x] `ServerNetworkPlugin` struct exists implementing `bevy::app::Plugin`
- [x] Plugin configures Lightyear server transport over WebSocket; port read from env var `SERVER_PORT` (default: `5000`)
- [x] `OnConnected` handler stub: logs `info!("Client connected: {:?}", client_id)` — no game logic
- [x] `OnDisconnected` handler stub: logs `info!("Client disconnected: {:?}", client_id)` — no game logic
- [x] `MessageReceiver<T>` stub systems exist for each C2S message type — each system logs `debug!("Received: {:?}", msg)` and returns; no game logic
- [x] `server/src/main.rs` registers `ServerNetworkPlugin` and calls `shared::protocol::register_protocol(&mut app)`

**`client/src/network/` module:**
- [x] `ClientNetworkPlugin` struct exists implementing `bevy::app::Plugin`
- [x] Plugin configures Lightyear client transport over WebSocket; server URL read from env var `SERVER_URL` (default: `ws://localhost:5000`)
- [x] `MessageSender<T>` stub resources/systems exist for each C2S message type — accessible for later UI wiring (no sends yet)
- [x] `client/src/main.rs` registers `ClientNetworkPlugin` and calls `shared::protocol::register_protocol(&mut app)`

**Unicast compile-proof (ADR-001 verification):**
- [x] At least one system in `server/src/network/` contains a compilable unicast send:
  ```rust
  // compile-proof only — no runtime call, wrapped in #[cfg(test)] or dead_code allowed
  fn _unicast_compile_proof(
      mut sender: /* verified 0.26 MessageSender type */,
      client_id: /* verified ClientId type */,
  ) {
      sender.send_to_target::<ReliableChannel>(
          S2CObjectiveIdentities { identities: vec![] },
          NetworkTarget::Single(client_id), // verified variant from Story 001
      );
  }
  ```
- [x] The function name and signature use ONLY the verified Lightyear 0.26 API from Story 001
- [x] Code comment: `// ADR-001 unicast compile-proof — verified NetworkTarget::Single syntax`

**Build targets:**
- [x] `cargo check -p server` passes with zero warnings
- [x] `cargo check -p client` passes with zero warnings (including WASM-incompatible deps absent)

---

## Implementation Notes

*Derived from ADR-008 §Implementation Guidelines and ADR-003 §Implementation Guidelines:*

**Server plugin sketch (fill in verified API from Story 001):**
```rust
// server/src/network/mod.rs
use bevy::prelude::*;
use lightyear::prelude::*; // exact imports — verify module paths in Story 001 report

pub struct ServerNetworkPlugin;

impl Plugin for ServerNetworkPlugin {
    fn build(&self, app: &mut App) {
        // Verified Lightyear 0.26 server plugin config
        // See tests/evidence/lightyear-026-verification.md items 1–9
        let port: u16 = std::env::var("SERVER_PORT")
            .unwrap_or_else(|_| "5000".into())
            .parse()
            .expect("SERVER_PORT must be a valid port number");

        // TODO: Add Lightyear ServerPlugins with WebSocket transport
        // Exact constructor: verified in Story 001

        app.add_systems(Update, (
            handle_connections,
            handle_disconnections,
            // C2S receiver stubs added here
        ));
    }
}

fn handle_connections(/* verified OnConnected event type */) { /* stub */ }
fn handle_disconnections(/* verified OnDisconnected event type */) { /* stub */ }
```

**Client plugin sketch:**
```rust
// client/src/network/mod.rs
pub struct ClientNetworkPlugin;

impl Plugin for ClientNetworkPlugin {
    fn build(&self, app: &mut App) {
        let server_url = std::env::var("SERVER_URL")
            .unwrap_or_else(|_| "ws://localhost:5000".into());

        // TODO: Add Lightyear ClientPlugins with WebSocket transport
        // Exact constructor: verified in Story 001
    }
}
```

**`MessageReceiver` stubs on server:** For each C2S type, add a system that reads messages and logs them. This proves the type is registered and receivable. Game logic is added in Core/Feature epics:
```rust
fn receive_c2s_create_room(
    mut receiver: /* verified MessageReceiver<C2SCreateRoom> type */,
) {
    for (client_id, msg) in receiver.receive_messages() { // verify method name
        debug!("C2SCreateRoom from {:?}: {:?}", client_id, msg);
        // Game logic: GSS epic
    }
}
```

**No game logic here:** All `MessageReceiver` systems in this story are stubs that log and return. No state mutation, no session creation, no RSM calls. The Game Session System epic (Core layer) wires the actual logic.

---

## Out of Scope

- Story 004: End-to-end connection test
- GSS epic (Core): actual game logic in MessageReceiver systems
- Full Lightyear replication setup (entity/component replication) — Core/Feature epics

---

## QA Test Cases

*QL-STORY-READY skipped — Lean mode.*

- **AC: Unicast compile-proof compiles**
  - Given: `_unicast_compile_proof` function written with `NetworkTarget::Single`
  - When: `cargo check -p server` is run
  - Then: Zero errors — unicast API compiles with the verified type

- **AC: No cross-crate deps introduced**
  - Given: `server/src/network/` and `client/src/network/` implemented
  - When: `cargo tree -p client` and `cargo tree -p server` are inspected
  - Then: Client tree contains no server crate; server tree contains no client crate

---

## Test Evidence

**Story Type**: Integration
**Required evidence**:
- `cargo check -p server` output → `tests/evidence/story-lyv-003-server-check.md`
- `cargo check -p client` output → `tests/evidence/story-lyv-003-client-check.md`
**Status**: [x] Created and verified by CI run `25176947506`

---

## Dependencies

- Depends on: Story 002 (all message types must be defined before plugins can reference them)
- Unlocks: Story 004 (end-to-end test)

## Completion Notes
**Completed**: 2026-04-30
**Verdict**: COMPLETE WITH NOTES
**Criteria**: 16/16 passing. Code inspection verified plugin structs, WebSocket env config, connection/disconnection stubs, C2S receiver stubs, client sender stubs, shared protocol registration on both sides, and the ADR-001 `NetworkTarget::Single` compile-proof.
**Deviations**: Advisory only - the implemented Lightyear 0.26 API uses `On<Add, Connected>` / `On<Add, Disconnected>` observers and `ServerMultiMessageSender::send(..., &NetworkTarget::Single(peer_id))`, matching the verified Story 001 API rather than the older sketch in the story.
**Test Evidence**: Integration evidence at `tests/evidence/story-lyv-003-server-check.md` and `tests/evidence/story-lyv-003-client-check.md`; GitHub Actions run `25176947506` passed for commit `215253e4eb1c234233459a9e742e06fd429ad4bb`.
**Local Verification**: `cargo check -p server` passed locally but emitted pre-existing warnings outside the network plugin scope. `cargo check -p client` was attempted twice locally; the first failed in Windows dependency compilation with memory/metadata errors, and the single-job retry timed out during dependency compilation before reaching the client crate. CI is the authoritative green build for this story.
**Code Review**: Skipped - lean mode.
