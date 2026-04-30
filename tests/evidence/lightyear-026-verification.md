# Lightyear 0.26 Verification Report

> **Story**: `production/epics/lightyear-protocol-verification/story-001-lightyear-026-verification-spike.md`
> **Date**: 2026-04-29
> **Verified against**: lightyear 0.26.4 (docs.rs — 0.26.0 build failed on docs.rs; 0.26.4 is the latest 0.26.x patch)
> **Sources**: `docs.rs/lightyear/0.26.4`, `docs.rs/lightyear_messages/0.26.4`, `docs.rs/lightyear_connection/0.26.4`, `liv-bevy-lightyear` skill `api_patterns.md`
> **ADR-012 test**: Written at `server/tests/session_ready_observer_test.rs`. Result: ✅ PASS locally from Developer PowerShell for VS 2026 after `.cargo/config.toml` set `target-dir = "target/msvc-local"`; command: `cargo test -p server session_ready_observer` (2 passed, 0 failed). Normal PowerShell still does not load MSVC `link.exe`.

---

## Items 1–3: Channel Definition and Modes

### Item 1: Channel definition syntax
⚠️ **DIFFERS** — `#[derive(Channel)]` macro does NOT exist in Lightyear 0.26.

**Actual API:**
```rust
// Channels are plain empty structs — no derive needed
pub struct ReliableChannel;
pub struct UnreliableChannel;

// Registered in ProtocolPlugin::build():
app.add_channel::<ReliableChannel>(ChannelSettings {
    mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
    ..default()
}).add_direction(NetworkDirection::Bidirectional);
app.add_channel::<UnreliableChannel>(ChannelSettings {
    mode: ChannelMode::UnorderedUnreliable,
    ..default()
}).add_direction(NetworkDirection::Bidirectional);
```

**`ChannelSettings` fields (0.26.4):**
- `mode: ChannelMode` — ordering/reliability guarantee
- `send_frequency: Duration` — how often to send (default = every frame)
- `priority: f32` — bandwidth priority

**No `direction` field on `ChannelSettings`.** In Lightyear 0.26.4, direction is attached to the returned `ChannelRegistration` with `.add_direction(NetworkDirection::...)`, while each message type also needs its own `MessageRegistration::add_direction(...)` (see Item 3). Story 004 verified that omitting channel `add_direction` leaves `Transport` present but without channel senders/receivers.

**Resolution for `shared/src/protocol.rs`:** Change channel definition from `#[derive(Channel)] pub struct ReliableChannel;` to a plain struct, registered via `app.add_channel::<T>(ChannelSettings { ... }).add_direction(NetworkDirection::Bidirectional)` in the protocol adapter.

---

### Item 2: `ChannelMode` enum variants
✅ **CONFIRMED** (with additional variants discovered)

`ChannelMode::OrderedReliable(ReliableSettings::default())` ✅ exists  
`ChannelMode::UnorderedUnreliable` ✅ exists

**Full variant list in 0.26.4:**
- `OrderedReliable(ReliableSettings)` — FIFO, guaranteed delivery ← use for `ReliableChannel`
- `UnorderedReliable(ReliableSettings)` — any order, guaranteed
- `SequencedReliable(ReliableSettings)` — newest-wins, guaranteed
- `UnorderedUnreliable` — fire-and-forget ← use for `UnreliableChannel`
- `SequencedUnreliable` — newest-wins, may drop
- `UnorderedUnreliableWithAcks` — unreliable with delivery tracking

The two variants used by ADR-008 (`OrderedReliable` and `UnorderedUnreliable`) are confirmed correct.

---

### Item 3: `ChannelDirection` enum variants
⚠️ **DIFFERS** — `ChannelDirection` does NOT appear in `ChannelSettings` in 0.26.4.

**Actual pattern:** Direction is configured on both protocol layers:
- Channel direction is configured on `ChannelRegistration` returned by `app.add_channel::<C>(...)`; this installs channel senders/receivers on `Transport`.
- Message direction is configured on `MessageRegistration` returned by `app.register_message::<M>()`; this installs `MessageSender<M>` / `MessageReceiver<M>` components.

Message example:
```rust
app.register_message::<C2SHello>()
    .add_direction(NetworkDirection::ClientToServer);

app.register_message::<S2CPhaseChanged>()
    .add_direction(NetworkDirection::ServerToClient);

app.register_message::<ChatMessage>()
    .add_direction(NetworkDirection::Bidirectional);
```

`NetworkDirection` enum (confirmed in prelude): `ServerToClient`, `ClientToServer`, `Bidirectional`.

**Resolution:** Do not add a `direction` field to `ChannelSettings`. Use `.add_direction(NetworkDirection::Bidirectional)` on both shared channels, and set direction per message type in `ProtocolPlugin::build()` using `add_direction(NetworkDirection::...)`.

---

## Items 4–6: MessageSender / MessageReceiver

### Item 4: `MessageSender<T>` and `MessageReceiver<T>` type names
✅ **CONFIRMED** (type names exist)  
⚠️ **DIFFERS** (usage pattern)

Both `MessageSender<M>` and `MessageReceiver<M>` are confirmed in `lightyear::prelude`. However, they are **Bevy components** added to client/server entities — not standalone system params accessed via `ResMut<T>`.

- `MessageSender<M>` — "A component that allows an entity to send messages of type M over the network"
- `MessageReceiver<M>` — "A component that receives messages of type M from the network"

For server-to-multi-client sending (unicast/broadcast), use `ServerMultiMessageSender` (see Item 9).

---

### Item 5: Client→server send method: `sender.send_to_server(msg)`
⚠️ **DIFFERS** — no `send_to_server()` method exists.

**Actual `MessageSender<M>` methods:**
```rust
// Channel chosen by generic type param C — no NetworkTarget param
fn send<C: Channel>(&mut self, message: M)
fn send_with_priority<C: Channel>(&mut self, message: M, priority: f32)
```

The client's `MessageSender<M>` component has a `send<C>()` method where the channel is the generic type. There is no runtime `NetworkTarget` parameter — destination is always the server.

**Resolution:** Client sends via:
```rust
fn send_c2s_hello(mut sender: Query<&mut MessageSender<C2SHello>>) {
    if let Ok(mut s) = sender.single_mut() {
        s.send::<ReliableChannel>(C2SHello { protocol_version: 1, session_token: None });
    }
}
```

---

### Item 6: Server receive method: `receiver.receive_messages()`
⚠️ **DIFFERS** — no `receive_messages()` method.

**Actual `MessageReceiver<M>` methods:**
```rust
fn receive(&mut self) -> impl Iterator<Item = M>          // basic iteration
fn receive_with_tick(&mut self) -> impl Iterator<Item = ReceivedMessage<M>>  // with tick metadata
fn has_messages(&self) -> bool
fn num_messages(&self) -> usize
```

Messages are **cleared every frame** in the `Last` schedule — no manual drain needed.

**Resolution:** Server reads C2S messages via:
```rust
fn handle_c2s_hello(receivers: Query<&mut MessageReceiver<C2SHello>>) {
    for mut recv in receivers.iter_mut() {
        for msg in recv.receive() {
            // handle msg
        }
    }
}
```

---

## Items 7–9: NetworkTarget and Server Send API

### Item 7: `NetworkTarget::Single(ClientId)` — unicast shape
⚠️ **DIFFERS** — `NetworkTarget` is a type alias, and the identifier type is `PeerId`, not `ClientId`.

**Actual definition:**
```rust
// lightyear_connection::network_target
pub type NetworkTarget = Target<PeerId>;  // type alias

pub enum Target<T> {
    None,
    Single(T),           // unicast — use: NetworkTarget::Single(peer_id)
    Only(Vec<T>),        // subset
    All,                 // broadcast
    AllExceptSingle(T),  // broadcast except one
    AllExcept(Vec<T>),   // broadcast except subset
}
```

**Critical:** The identifier is `PeerId` (confirmed in `lightyear::prelude`), not `ClientId`. ADR-011's `SessionToken` as identity bridge across reconnects remains correct — `PeerId` is what gets reassigned on reconnect.

**Resolution for all ADRs:** Replace `ClientId` with `PeerId` in all networking API references. Unicast: `NetworkTarget::Single(peer_id)`.

---

### Item 8: `NetworkTarget::All` broadcast shape
✅ **CONFIRMED** — `Target::All` variant exists exactly as assumed.

Also confirmed: `Target::AllExceptSingle(PeerId)` for broadcasting to all except one peer.

---

### Item 9: Server unicast send API
⚠️ **DIFFERS SIGNIFICANTLY** — the API is `ServerMultiMessageSender`, not a method on a `server` handle.

**Actual server send API — `ServerMultiMessageSender` system param:**
```rust
// M = Message type, C = Channel type
// server: Res<Server> or &Server from query

fn send<M: Message, C: Channel>(
    &mut self,
    message: &M,
    server: &Server,
    target: &NetworkTarget,
) -> Result

fn send_with_priority<M: Message, C: Channel>(
    &mut self,
    message: &M,
    server: &Server,
    target: &NetworkTarget,
    priority: Priority,
) -> Result

// Alternative: send to specific client entities directly
fn send_to_entities<M: Message, C: Channel>(
    &mut self,
    message: &M,
    target: impl EntitySet,
) -> Result
```

**Resolution:** Server unicast example:
```rust
fn send_objective_identities(
    mut sender: ServerMultiMessageSender,
    server: Query<&Server>,
    // ... other params
) {
    let Ok(server) = server.single() else { return; };
    let msg = S2CObjectiveIdentities { /* ... */ };
    // Unicast to specific peer:
    let _ = sender.send::<S2CObjectiveIdentities, ReliableChannel>(
        &msg, server, &NetworkTarget::Single(peer_id)
    );
}
```

Note: generics order is **Message first, Channel second** — opposite of what ADR-008 assumed (`send_message_to_target::<ChannelType, MessageType>`).

---

## Item 10: In-order delivery guarantee on `ReliableChannel`

✅ **CONFIRMED** (by definition + architecture)

`ChannelMode::OrderedReliable` guarantees FIFO delivery of all messages on that channel, across all message types registered to it. The `OrderedReliable` mode processes messages in the order they are enqueued, regardless of type. The OQ-D invariant (`S2CResolutionEvent` before `S2CPhaseChanged`) is upheld by same-channel enqueue order as designed in ADR-008.

**Caveat:** Cross-type ordering within a single `OrderedReliable` channel requires that the transport layer preserves enqueue order. Over WebSocket (WASM client), the TCP framing provides this. Not yet verified via integration test — consider adding to Story 004 (end-to-end connection test) acceptance criteria.

---

## Items 11–14: Reconnect / Connection Lifecycle

### Item 11: Snapshot-before-live-messages guarantee
⚠️ **DIFFERS** — Lightyear 0.26 does NOT provide a built-in snapshot-before-live guarantee.

This is an **application-level concern**, not a Lightyear feature. The implementation must:
1. Schedule the snapshot-send system **before** all other S2C message systems in the `Update` schedule.
2. Track `snapshot_sent: bool` per-connection as specified in ADR-011.
3. Check `snapshot_sent` before enqueuing any live S2C message.

Since `ReliableChannel` guarantees FIFO ordering, enqueuing `S2CGameSnapshot` first in the `Update` tick ensures it arrives before any subsequent enqueued messages.

**Resolution:** This is implementation-level discipline, not a missing Lightyear feature. ADR-011 design is valid.

---

### Item 12: `ClientId` reassigned on reconnect
⚠️ **DIFFERS** (naming/model)

There is no `ClientId` type in Lightyear 0.26 prelude — the identifier is `PeerId`.

In the entity-per-connection model: when a client disconnects, its entity (with `LinkOf`) is despawned. On reconnect, a **new entity** is spawned with a new `PeerId`. There is no continuity of identity at the Lightyear level — identity continuity is entirely the application's responsibility (ADR-011's `SessionToken` design).

**Resolution:** Replace `ClientId` with `PeerId` throughout. `SessionToken` as cross-reconnect bridge is confirmed correct.

---

### Item 13: `OnConnected` event timing
⚠️ **DIFFERS** (naming)

No `OnConnected` event type in `lightyear_connection`. Connection lifecycle uses:
- **Client-side marker components**: `Disconnected`, `Connecting`, `Connected`
- **Server-side**: when a client connects, a new entity with `LinkOf` (child of the server entity) is spawned with the `Connected` marker
- **Detection**: observe via `Trigger<OnAdd, Connected>` on client entities, or query for newly spawned `LinkOf` entities on the server

**Resolution:** Replace `OnConnected` with either `Trigger<OnAdd, Connected>` observer or system checking for new `LinkOf` entities. ADR-011 logic is sound; only the event name needs updating.

---

### Item 14: Messages sent pre-`OnConnected` not delivered to new `PeerId`
✅ **CONFIRMED** (by architecture)

In the entity-per-connection model, each reconnect creates a new `LinkOf` entity with a new `PeerId`. The previous connection entity is despawned. Any messages buffered for the old `PeerId` are discarded with the old entity. The new entity starts with an empty message queue. This guarantees that no pre-connect messages leak to the reconnecting client.

---

## Items 15–17: Bevy Observer / Trigger Semantics

### Item 15: `Commands::trigger(SessionReady)` fires Observer same frame
✅ **CONFIRMED**

Test written at `server/tests/session_ready_observer_test.rs` (`test_session_ready_observer_fires_in_same_frame`).

**Actual: PASS.** Verified locally from Developer PowerShell for VS 2026 with `cargo test -p server session_ready_observer` after `.cargo/config.toml` set `target-dir = "target/msvc-local"`.

This confirms Bevy 0.18 applies `Commands::trigger(E)` in the command queue flush and dispatches registered observers in the same flush cycle.

---

### Item 16: Resource visible to Observer after `Commands::insert_resource()` before `Commands::trigger()`
✅ **CONFIRMED**

Test written at `server/tests/session_ready_observer_test.rs` (`test_session_ready_observer_resource_visible_after_commands_insert`).

**Actual: PASS.** Commands in Bevy's command queue are applied in the order they are issued. When the queue flushes: `insert_resource(SessionConfig)` is applied first (command 0), then `trigger(SessionReady)` fires the observer (command 1). The observer can access `Res<SessionConfig>` without panic.

**Verification command:** `cargo test -p server session_ready_observer` from Developer PowerShell for VS 2026. Result: 2 passed; 0 failed.

---

### Item 17: Observer parameter type and registration API
⚠️ **DIFFERS** — `Trigger<T>` renamed to `On<T>`; `App::observe()` renamed to `App::add_observer()`

**Initial assumption:** `Trigger<EventType>` + `app.observe()`.

**Confirmed by CI compilation (2026-04-29, commit f498671):** The correct Bevy 0.18 API is `On<T>` (renamed from `Trigger<T>` in 0.16) and `App::add_observer()` (renamed from `App::observe()`).

```rust
// ✅ Correct Bevy 0.18 observer signature:
fn on_session_ready(_trigger: On<SessionReady>, config: Res<SessionConfig>) {
    // ...
}
app.add_observer(on_session_ready);

// ❌ Old API (pre-0.16) — does not compile on Bevy 0.18:
// Trigger<SessionReady>, app.observe()
```

**Resolution:** Replace `Trigger<T>` with `On<T>` and `app.observe()` with `app.add_observer()` everywhere. `commands.trigger(E)` and `commands.trigger_targets(E, entity)` are unchanged.

---

## Items 18–20: Replication

### Item 18: Component replication opt-in (no auto-replication)
✅ **CONFIRMED**

Newly spawned entities are NOT automatically replicated. Two requirements must both be met:
1. Register the component: `app.register_component::<T>()` in `ProtocolPlugin`
2. Add the `Replicate` component to the entity: `commands.spawn((MyComponent, Replicate::default()))`

Entities spawned without `Replicate` are local-only. This confirms ADR-007's invariant: unit ECS entities spawned before `S2CPlacementReveal` is enqueued will NOT leak to clients, because they would not have `Replicate` at spawn time. The reveal + spawn sequencing is a belt-and-suspenders precaution; the replication opt-in provides the actual enforcement.

---

### Item 19: `ReplicationGroup` API
✅ **CONFIRMED**

`ReplicationGroup` struct confirmed in `lightyear::prelude`. Usage for entity grouping:
```rust
commands.spawn((
    MyComponent { target: other_entity },
    Replicate {
        group: ReplicationGroup::new_id(my_group_id),
        ..default()
    },
));
```

`ReplicationGroup::new_id(id)` API matches ADR-001 assumptions. No changes required.

---

### Item 20: `LocalTimeline` is a Resource
✅ **CONFIRMED**

`LocalTimeline` is a struct confirmed in `lightyear::core::prelude` with description "The local timeline that matches `Time<Virtual>`". It is accessible as a `Res<LocalTimeline>` in server systems. No changes required from ADR assumptions.

---

## ADR-012 Open Condition Summary

| Test | File | Expected Result | Actual Result |
|------|------|-----------------|---------------|
| `test_session_ready_observer_fires_in_same_frame` | `server/tests/session_ready_observer_test.rs` | PASS | ✅ PASS (CI run 25133926012, commit f498671) |
| `test_session_ready_observer_resource_visible_after_commands_insert` | `server/tests/session_ready_observer_test.rs` | PASS | ✅ PASS (CI run 25133926012, commit f498671) |

**ADR-012 open condition: RESOLVED** — `Commands::trigger()` flush ordering confirmed. `Res<SessionConfig>` visible to Observer after `Commands::insert_resource()`. No `apply_deferred` needed in `RsmPlugin::build()`.

**If Item 16 test FAILS:** Add `apply_deferred` to the `.chain()` in `RsmPlugin::build()` between `evaluate_session_ready` and the RSM observer trigger system. Alternatively, adopt the `World::trigger()` exclusive system fallback from ADR-012 §Alternative 2.

---

## Story 004 Final Runtime Verification

**Date:** 2026-04-30

**Local command:** `cargo test -p server --test e2e_websocket_test e2e_websocket_heartbeat_roundtrip_and_reliable_channel`

**Result:** PASS - 1 passed, 0 failed.

**Verified runtime APIs/patterns:**
- `ServerPlugins` and `ClientPlugins` with in-process Bevy `App::update()` loops.
- WebSocket IO via `WebSocketServerIo` / `WebSocketClientIo`.
- Raw connection markers `RawServer` / `RawClient` for non-netcode WebSocket links.
- `ServerConfig::builder().with_bind_address(bind_addr).with_no_encryption()` so the listener matches `LocalAddr`.
- Channel registration via `app.add_channel::<C>(ChannelSettings { ... }).add_direction(NetworkDirection::Bidirectional)`.
- Message registration via `app.register_message::<M>().add_direction(NetworkDirection::...)`.
- Client send via `MessageSender<C2SHeartbeat>::send::<UnreliableChannel>(...)`.
- Server receive via `MessageReceiver<C2SHeartbeat>::receive()`.
- Server broadcast via `ServerMultiMessageSender::send::<M, C>(..., &NetworkTarget::All)`.

**ADR-012 open condition final status:** RESOLVED - no `apply_deferred` needed.

---

## Resolution Paths for DIFFERS Items

| Item | Correct API | Files to Update |
|------|-------------|-----------------|
| 1 | `app.add_channel::<T>(ChannelSettings { mode: ..., ..default() }).add_direction(NetworkDirection::Bidirectional)` | `shared/src/protocol.rs`, protocol adapters |
| 3 | Direction on channel registration and message registration via `.add_direction(NetworkDirection::...)` | `shared/src/protocol.rs`, protocol adapters |
| 5 | `sender.send::<Channel>(message)` (channel via generic) | Server C2S handler stub |
| 6 | `receiver.receive()` → `impl Iterator<Item = M>` | Server C2S handler stub |
| 7 | `NetworkTarget::Single(PeerId)` (not ClientId) | All ADRs referencing `ClientId`, `shared/src/protocol.rs` |
| 9 | `ServerMultiMessageSender::send::<M, C>(&msg, &server, &target)` | All S2C send stubs |
| 12 | Use `PeerId` not `ClientId`; `SessionToken` as bridge | ADR-011, all handlers |
| 13 | `Trigger<OnAdd, Connected>` not `OnConnected` event | GSS/reconnect implementation |

**All DIFFERS items have concrete resolution paths. No blockers remain for Stories 002–004.**
