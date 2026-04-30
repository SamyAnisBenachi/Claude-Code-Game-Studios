# Story 004: End-to-End WebSocket Round-Trip Test

> **Epic**: Lightyear Protocol & Verification Spike
> **Status**: Complete
> **Layer**: Foundation
> **Type**: Integration
> **Manifest Version**: 2026-04-29

## Context

**GDD**: `design/gdd/network-protocol.md`
**Requirement**: TR-??? (covers TR-NP-RELIABLE + TR-NP-UNRELIABLE runtime verification; end-to-end connection proof; 28 BLOCKING network-protocol.md ACs at compile-level coverage)

**ADR Governing Implementation**: ADR-008: Lightyear Channel Config
**ADR Decision Summary**: Both reliable and unreliable channel paths must be proven functional before Core epics begin. The heartbeat round-trip (`C2SHeartbeat` → server receives → server sends `S2CHeartbeat` → client receives) is the minimal proof that: transport connects, both channel types carry messages, and the message type catalog compiles end-to-end.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**: The end-to-end test runs server and client as separate processes connected over localhost WebSocket. WASM cannot be used for integration tests — use native client build for CI. Exact Lightyear 0.26 test patterns must follow the verified API from Story 001. If WebSocket transport requires async runtime (tokio), confirm this does not conflict with Bevy's scheduler on the server side.

**Control Manifest Rules (Foundation layer)**:
- Required: Exactly two channels. Channel assignment permanent per message type.
- Guardrail: WASM bundle ≤ 50 MB (measure in this story's CI job).

---

## Acceptance Criteria

**End-to-end heartbeat round-trip:**
- [x] Server binary starts, listens on `localhost:PORT`
- [x] Client (native build) connects to server over WebSocket
- [x] Client sends `C2SHeartbeat {}` on `UnreliableChannel`
- [x] Server receives `C2SHeartbeat` via its `MessageReceiver<C2SHeartbeat>` stub
- [x] Server sends `S2CHeartbeat {}` broadcast on `UnreliableChannel`
- [x] Client receives `S2CHeartbeat` via its message receiver
- [x] Test exits 0; CI job passes

**Reliable channel proof:**
- [x] Server sends a `S2CHandshakeRejected` (or any `ReliableChannel` message) to the client after connection
- [x] Client receives it — proves `ReliableChannel` is functional
- [x] Note: full handshake flow is owned by the GSS epic — only channel functionality is verified here

**WASM bundle size:**
- [x] `cargo build -p client --target wasm32-unknown-unknown --release` completes
- [x] Output `.wasm` artefact size measured and documented in `tests/evidence/story-lyv-004-wasm-size.md`
- [x] Size ≤ 50 MB; if > 50 MB, document mitigation path (drop unused Bevy features, run `wasm-opt`) but do not block the story

**DIFFERS resolution documentation:**
- [x] Any checklist items marked `⚠️ DIFFERS` in Story 001 that affected implementation in Stories 002–004 are documented with their resolution in `tests/evidence/lightyear-026-verification.md` — no open DIFFERS without a resolution path

**ADR-012 open condition final status:**
- [x] `tests/evidence/lightyear-026-verification.md` records final ADR-012 status: either "RESOLVED — no apply_deferred needed" or "apply_deferred path documented in GSS epic story"

---

## Implementation Notes

*Derived from ADR-008 §Verification Required and ADR-003 §Migration Plan step 4:*

**Test architecture:**
The end-to-end test should use Bevy's `App::update()` loop rather than spawning OS processes — spin up a server `App` and a client `App` in the same test process, run `app.update()` N times, and assert messages were received. This avoids port-binding race conditions in CI:

```rust
#[test]
fn test_e2e_heartbeat_roundtrip() {
    // Server app
    let mut server_app = App::new();
    server_app.add_plugins(MinimalPlugins);
    server_app.add_plugins(ServerNetworkPlugin);
    // register_protocol already called by ServerNetworkPlugin

    // Client app (native, not WASM)
    let mut client_app = App::new();
    client_app.add_plugins(MinimalPlugins);
    client_app.add_plugins(ClientNetworkPlugin);

    // Track receipt
    let heartbeat_received = Arc::new(AtomicBool::new(false));
    let flag = heartbeat_received.clone();
    client_app.add_systems(Update, move |/* receiver */: /* MessageReceiver<S2CHeartbeat> */| {
        // if message received: flag.store(true, ...)
    });

    // Run both apps for N frames
    for _ in 0..60 {
        server_app.update();
        client_app.update();
    }

    assert!(heartbeat_received.load(Ordering::SeqCst),
        "S2CHeartbeat not received — check Lightyear 0.26 transport setup");
}
```

Exact Lightyear 0.26 in-process test pattern: verify against the lightyear examples repo. If Lightyear requires spawning separate threads for server/client, document this as a test infrastructure decision.

**If Lightyear 0.26 does not support in-process server+client test:**
Fallback: separate process test via `std::process::Command`. Document in `tests/evidence/story-lyv-004-test-infrastructure.md`. This is an acceptable fallback — the important thing is the round-trip is proven.

**CI job structure:**
```yaml
# .github/workflows/tests.yml addition
- name: E2E WebSocket round-trip
  run: cargo test -p server e2e_websocket --release
  # or: cargo test --test e2e_websocket if in tests/integration/
```

**WASM bundle size check:**
```bash
cargo build -p client --target wasm32-unknown-unknown --release
ls -la target/wasm32-unknown-unknown/release/lanes-and-lies-client.wasm
# Must be ≤ 50,000,000 bytes
```
If Trunk produces a different output path, adjust accordingly.

---

## Out of Scope

- Full handshake flow (C2SHello → S2CHandshake) — GSS epic (Core layer)
- Full game state replication — Board/Lane and Objective System epics
- WASM client browser testing — requires manual verification; CI uses native build

---

## QA Test Cases

*QL-STORY-READY skipped — Lean mode.*

- **AC: Heartbeat round-trip completes**
  - Given: Server and client running (same process or separate)
  - When: Client sends `C2SHeartbeat`; test runs for 60+ frames
  - Then: Client's `S2CHeartbeat` receiver fires at least once; `heartbeat_received == true`

- **AC: Reliable channel message received**
  - Given: Server sends any `ReliableChannel` message after connection
  - When: Client polls for N frames
  - Then: Message received — `ReliableChannel` functional confirmed

- **AC: WASM bundle ≤ 50 MB**
  - Given: `cargo build -p client --target wasm32-unknown-unknown --release`
  - When: `.wasm` artefact size is measured
  - Then: ≤ 50,000,000 bytes; or mitigation documented if over budget

---

## Test Evidence

**Story Type**: Integration
**Required evidence**:
- `tests/integration/network/e2e_websocket_test.rs` — test passing
- WASM bundle size measurement → `tests/evidence/story-lyv-004-wasm-size.md`
- Final DIFFERS resolutions → `tests/evidence/lightyear-026-verification.md` (updated)
**Status**: [x] Created

---

## Dependencies

- Depends on: Story 003 (server + client plugins must be wired before connection test can run)
- Unlocks: Epic `lightyear-protocol-verification` **complete** → ALL Core layer epics (GSS, RSM, Economy, Card Pool) can begin
