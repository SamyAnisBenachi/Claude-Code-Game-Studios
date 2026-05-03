# ADR-008: Lightyear 0.26 Channel Configuration and Message Routing

## Status

Accepted

## Date

2026-04-29

## Last Verified

2026-04-29

## Decision Makers

User + network-programmer (design spike), technical-director (authority model validation)

## Summary

All Lanes and Lies network messages route over exactly two Lightyear channels: a `ReliableChannel` for all game-state and control messages that must arrive in order, and an `UnreliableChannel` for high-frequency updates (auction timer ticks, `C2SHeartbeat`) where a dropped packet is superseded by the next. Channel assignment is permanent per message type — no message ever switches channel at runtime.

## ⚠️ API Verification Required (Lightyear 0.26)

Lightyear 0.26 uses an **entity-per-connection** model (introduced in v0.25). The
older resource-based approach (`ClientConfig`, `ClientConnectionManager`) no longer
exists. All Lightyear API patterns in this ADR must be verified against
`docs.rs/lightyear/0.26` before any networking story begins implementation.

Key model changes to verify:
- Client is an entity with `Client` marker + `Link` + transport IO component
- Server is an entity with `NetcodeServer` (which auto-inserts `Server`)
- Connection lifecycle uses triggers (`Connect`/`Disconnect`, `Start`/`Stop`) and
  marker components (`Disconnected`/`Connecting`/`Connected`)
- `MessageSender<M>`/`MessageReceiver<M>` auto-added based on protocol registration
- Protocol registered via `ProtocolPlugin` AFTER `ClientPlugins`/`ServerPlugins`

See `liv-bevy-lightyear` skill for the complete 0.26 API reference.

---

## Engine Compatibility

| Field | Value |
|-------|-------|
| **Engine** | Bevy 0.18 + Lightyear 0.26 |
| **Domain** | Networking |
| **Knowledge Risk** | HIGH — the entire Lightyear 0.26 API is post-cutoff (released January 2026). Channel registration syntax, `MessageSender`/`MessageReceiver` types, and `NetworkTarget` variants must be verified against current docs before implementation. |
| **References Consulted** | `docs/engine-reference/bevy/current-best-practices.md` (Lightyear 0.26 message patterns), `docs/engine-reference/bevy/VERSION.md`, ADR-001 (unicast target shape) |
| **Post-Cutoff APIs Used** | `ReliableChannel`, `UnreliableChannel` channel definitions; `MessageSender<T>`, `MessageReceiver<T>` system params; `ServerMultiMessageSender` server S2C targeted-send system param; `NetworkTarget::Single(PeerId)` unicast target; `NetworkTarget::All` all-player target; `ChannelSettings` / channel registration macro |
| **Verification Required** | See Implementation Guidelines — full numbered checklist in that section |

> **Note**: Knowledge Risk is HIGH. This ADR must be re-validated if the project upgrades
> to Lightyear 0.27 or later. Flag as "Superseded" and write a new ADR on any upgrade.

## ADR Dependencies

| Field | Value |
|-------|-------|
| **Depends On** | ADR-002 (client-server authority model — must be Accepted so channel authority direction is settled); ADR-003 (workspace layout — `shared/src/protocol.rs` is where channel definitions live) |
| **Enables** | All networking implementation — every system that sends or receives Lightyear messages reads channel types from `shared/src/protocol.rs` |
| **Blocks** | Any story that implements a Lightyear `MessageSender` or `MessageReceiver` system param. No networking code may be written until this ADR is Accepted. |
| **Ordering Note** | ADR-002 and ADR-003 are both Accepted (2026-04-29). Channel definitions in this ADR are in effect. |

## Context

### Problem Statement

Lanes and Lies uses Lightyear 0.26 for all client-server communication. Every message must be assigned to exactly one delivery channel before any networking code can be written. Without a documented, stable channel assignment per message type, individual system programmers will make ad-hoc channel choices that are inconsistent — a phase transition on an unreliable channel, or an auction timer tick on a reliable channel consuming reliable bandwidth. The assignment also carries a critical ordering invariant (OQ-D): `S2CResolutionEvent` must be received by the client before `S2CPhaseChanged(DRAFT_SHOP)`. If these two messages were ever placed on different channels, the ordering guarantee vanishes silently, breaking the resolution replay on reconnect.

### Current State

No channel definitions exist in the codebase. This ADR is a greenfield decision — it precedes any networking implementation.

### Constraints

- **Engine**: Lightyear 0.26 supports two primary delivery tiers for application messages: reliable-ordered and unreliable. No partially-reliable or sequenced-unreliable tier is exposed as a first-class channel primitive in 0.26.
- **Bandwidth budget**: < 1 KB per round message (technical-preferences.md). Auction timer ticks on reliable would inflate this with retransmit overhead.
- **WASM/WebSocket transport**: Lightyear operates over WebSocket on the WASM client. The underlying TCP framing of WebSocket means "unreliable" in Lightyear's sense is application-layer unreliable — packets are still delivered or the connection closes. True UDP unreliability applies only on native WebTransport. For WASM/WebSocket, `UnreliableChannel` messages may arrive in order in practice, but correctness must not depend on this.
- **Ordering invariant (OQ-D)**: `S2CResolutionEvent` must precede `S2CPhaseChanged(DRAFT_SHOP)` on the wire. Lightyear's reliable channel guarantees FIFO ordering only within a single channel. Splitting these two messages across channels would require an explicit sequence-number field and client-side reorder buffering — a significant complexity cost.
- **Reconnect snapshot**: `S2CGameSnapshot` must be the first S2C message processed after reconnect. Post-snapshot live messages must not overtake it. Since all are on `ReliableChannel`, enqueue order governs delivery order — the snapshot system must be scheduled before other systems in the same `Update` frame.

### Requirements

- Every message type has a single, permanent, documented channel assignment.
- Messages whose loss produces a recoverable or superseded state use `UnreliableChannel`.
- All messages that alter authoritative game state, phase, economy, hand, or snapshot delivery use `ReliableChannel`.
- The `S2CResolutionEvent → S2CPhaseChanged(DRAFT_SHOP)` ordering invariant is upheld by same-channel ordering alone, with no application-level sequence numbers required.
- Channel definitions live in `shared/src/protocol.rs` so both client and server compile against the same types.
- Bandwidth for unreliable messages does not contribute to the reliable retransmit queue.

## Decision

All game messages use exactly two channels. Channel assignment is permanent per message type and is defined in `shared/src/protocol.rs`. No message switches channel at runtime under any condition.

**`ReliableChannel`** — ordered, guaranteed delivery. Used for all messages that change game state, carry secret data, or must arrive to maintain consistency.

**`UnreliableChannel`** — best-effort, no ordering guarantee. Used only for messages where a dropped packet is immediately superseded by the next packet of the same type, and where late delivery of a stale packet causes no harm.

### Architecture

```
CLIENT                              SERVER
  |                                   |
  |  UnreliableChannel                |
  |  C2SHeartbeat (every ~5s) ------> |  reset disconnect_trackers[player]
  |                                   |
  |  ReliableChannel                  |
  |  C2SHello ----------------------> |  handshake
  |  C2SSubmitPlacement ------------> |  placement validation
  |  C2SAuctionBid -----------------> |  bid validation
  |  C2SReadySignal ----------------> |  ready state
  |  C2SConfirmClass ---------------> |  class lock
  |  C2SPurchaseCard / C2SRefreshShop |
  |  C2SActivateCard / C2SAcknowledge |
  |                                   |
  |       ReliableChannel             |
  | <-- S2CHandshake -------------    |  first message on any connection
  | <-- S2CGameSnapshot ----------    |  first game message after handshake
  | <-- S2CObjectiveIdentities ---    |  unicast per player at DRAFT_INITIAL
  | <-- S2CPhaseChanged ----------    |  phase transitions (broadcast)
  | <-- S2CResolutionEvent -------    |  MUST precede S2CPhaseChanged(DRAFT_SHOP)
  | <-- S2CPlacementReveal -------    |  atomic simultaneous reveal
  | <-- S2CGoldUpdate ------------    |  unicast per player
  | <-- S2CGoldBroadcast ---------    |  broadcast
  | <-- S2CShopSlots / S2CDraft --    |  unicast card data
  | <-- S2CGameOver --------------    |  terminal state
  | <-- (all other S2C) ----------    |
  |                                   |
  |       UnreliableChannel           |
  | <-- S2CAuctionUpdate (timer) -    |  high-frequency; stale = superseded
  |                                   |
  |  [OQ-D invariant]                 |
  |  ReliableChannel enqueue order:   |
  |  S2CResolutionEvent first,        |
  |  then S2CPhaseChanged             |
  |  Same channel = FIFO = safe       |
```

### Key Interfaces

```rust
// shared/src/protocol.rs

use lightyear::prelude::*;

/// Ordered, guaranteed delivery.
/// All game-state, economy, hand, phase, snapshot, and control messages.
pub struct ReliableChannel;

/// Best-effort, no ordering guarantee.
/// High-frequency messages where a dropped packet is superseded by the next.
pub struct UnreliableChannel;

// Channel settings — exact ChannelSettings struct fields must be verified
// against Lightyear 0.26 docs before implementation (see Implementation
// Guidelines checklist item 1).
//
// Expected shape (verify):
//   app.add_channel::<ReliableChannel>(ChannelSettings {
//       mode: ChannelMode::OrderedReliable(ReliableSettings::default()),
//       ..default()
//   }).add_direction(NetworkDirection::Bidirectional);
//   app.add_channel::<UnreliableChannel>(ChannelSettings {
//       mode: ChannelMode::UnorderedUnreliable,
//       ..default()
//   }).add_direction(NetworkDirection::Bidirectional);
```

```
Message → Channel Assignment Table

Message                        Channel         Scope     Reason
─────────────────────────────────────────────────────────────────────────────
C2SHello                       Reliable        C→S       Handshake must not be dropped
C2SSubmitPlacement             Reliable        C→S       Input must not be dropped
C2SAuctionBid (C2SPlaceBid)    Reliable        C→S       Input must not be dropped
C2SReadySignal                 Reliable        C→S       Lobby signal must arrive
C2SConfirmClass                Reliable        C→S       Class lock is irrevocable
C2SPurchaseCard                Reliable        C→S       Economy mutation
C2SRefreshShop                 Reliable        C→S       Economy mutation
C2SActivateCard                Reliable        C→S       Card play must be processed
C2SAcknowledgeResult           Reliable        C→S       Session cleanup gate
C2SHeartbeat                   Unreliable      C→S       Presence signal; gaps detected
                                                         by timer not drop; high frequency

S2CHandshake                   Reliable        Unicast   First message on connect
S2CHandshakeRejected           Reliable        Unicast   Version gate
S2CGameSnapshot                Reliable        Unicast   Full state rebuild; must arrive first
S2CObjectiveIdentities         Reliable        Unicast   Secret data; must arrive before placement
S2CPhaseChanged                Reliable        Broadcast Phase transition must be ordered
S2CPlacementReveal             Reliable        Broadcast Authoritative board state; ordered
S2CResolutionEvent             Reliable        Broadcast Ordered replay log; sub_step order
                                                         is contractual
S2CGoldUpdate                  Reliable        Unicast   Economy state must be consistent
S2CGoldBroadcast               Reliable        Broadcast Opponent gold always visible
S2CShopSlots                   Reliable        Unicast   Card data must not be dropped
S2CDraftOffering               Reliable        Unicast   Card data must not be dropped
S2CPoolUpdate                  Reliable        Unicast   Pool delta must not be dropped
S2CCardAcquired                Reliable        Unicast   Hand state must not be dropped
S2CPrismRewardDropped          Reliable        Unicast   Owner-only hand-full prism drop notice
S2CPrismRespawned              Reliable        Broadcast Full-set respawn notice for all clients
S2CAuctionCard                 Reliable        Broadcast Card selection must arrive before
                                                         DRAFT_AUCTION UI activates
S2CAuctionBidAccepted          Reliable        Broadcast Bid state must be consistent
S2CAuctionBidRejected          Reliable        Unicast   Client feedback; must arrive
S2CAuctionSettled              Reliable        Broadcast Terminal auction state; must arrive
S2CAuctionUpdate (timer/price) Unreliable      Broadcast High-frequency; stale superseded
S2CGameOver                    Reliable        Broadcast Terminal state; must arrive
S2COpponentDisconnected        Reliable        Unicast   Must arrive; drives UI countdown
S2COpponentReconnected         Reliable        Broadcast Session continuity
S2CRoomCreated                 Reliable        Unicast   Room creation confirmation
S2CJoinAck                     Reliable        Unicast   Lobby state on join
S2CJoinRejected                Reliable        Unicast   Must arrive for client feedback
S2CSlotUpdated                 Reliable        Broadcast Lobby slot changes
S2CClassLocked                 Reliable        Unicast   Own lock confirmation
S2CClassesRevealed             Reliable        Broadcast All-locked reveal
S2CConfirmClassRejected        Reliable        Unicast   Client feedback
S2CSessionCancelled            Reliable        Broadcast Session lifecycle
S2CSangMepriseReveal           Reliable        Unicast   Secret data; must arrive
                                                (opponent)
```

### Implementation Guidelines

The following checklist MUST be completed before writing any networking code. These are verification items for Lightyear 0.26 API shapes that are post-cutoff and may differ from the patterns shown above.

**Lightyear 0.26 Verification Checklist**

1. [x] **Channel definition syntax** — Verified against Lightyear 0.26.4 evidence. Channels are plain empty structs; `#[derive(Channel)]` does not exist. Register each channel with `app.add_channel::<T>(ChannelSettings { mode, ..default() }).add_direction(NetworkDirection::Bidirectional)`. Evidence: `tests/evidence/lightyear-026-verification.md` items 1-3, `shared/src/protocol.rs::register_protocol`, `server/src/network/mod.rs::LightyearProtocolRegistry::add_channel`, and local crate source `lightyear_transport-0.26.4/src/channel/registry.rs` / `builder.rs`.

2. [ ] **`ChannelMode` enum variants** — Verify the exact variants for ordered-reliable and unreliable modes. Expected: `ChannelMode::OrderedReliable(ReliableSettings::default())` and `ChannelMode::UnorderedUnreliable`. Confirm these exist in 0.26 and have not been renamed.

3. [ ] **`ChannelDirection` enum** — Verify that `Bidirectional`, `ClientToServer`, and `ServerToClient` are the correct variants. Confirm whether the channel direction is set per-registration or per-channel-definition.

4. [ ] **`MessageSender<T>` / `MessageReceiver<T>` system param names** — `current-best-practices.md` uses `MessageSender<T>` and `MessageReceiver<T>`. Verify these are the canonical 0.26 system params (not `MessageChannel`, `NetworkWriter`, or similar names from earlier Lightyear versions).

5. [x] **`send` method on `MessageSender`** — verified in Lightyear 0.26.4 evidence: client sends use `sender.send::<ChannelType>(msg)`. There is no `send_to_server(msg)` method in the verified API.

6. [x] **`receive` method on `MessageReceiver`** — verified in Lightyear 0.26.4 evidence: use `receiver.receive()`; do not use the older `receive_messages()` sketch.

7. [x] **`NetworkTarget::Single(PeerId)` unicast shape** — verified in Lightyear 0.26.4 source/evidence. `NetworkTarget` is a `Target<PeerId>` alias; use `NetworkTarget::Single(peer_id)` for unicast. The identifier type is `PeerId`, not `ClientId`.

8. [x] **`NetworkTarget::All` / `NetworkTarget::AllExcept*` broadcast shapes** — verified in Lightyear 0.26.4 source/evidence. Use `NetworkTarget::All` for all connected clients; `AllExceptSingle(PeerId)` and `AllExcept(Vec<PeerId>)` exist for exclusion cases.

9. [x] **Server-side targeted send API** — verified in Lightyear 0.26.4 source/evidence. Use `ServerMultiMessageSender::send::<MessageType, ChannelType>(&msg, server, &NetworkTarget::Single(peer_id))` for unicast and the same method with `NetworkTarget::All` for all-player delivery. Generic order is message first, channel second; there is no `server.send_message_to_target` method.

10. [ ] **In-order delivery guarantee within a single reliable channel, across different message types** — The OQ-D invariant (see below) depends on `S2CResolutionEvent` and `S2CPhaseChanged` being delivered in enqueue order when both are on `ReliableChannel`. Verify that Lightyear 0.26 guarantees FIFO ordering for all message types on the same `OrderedReliable` channel, not just per-type. This is acceptance criterion NP-8 in the Network Protocol GDD and is BLOCKING for the resolution/phase handoff.

11. [ ] **Snapshot delivery sequencing** — Verify that messages enqueued to `ReliableChannel` for the same connection are delivered in strict enqueue order. The reconnect invariant requires `S2CGameSnapshot` to be dequeued before any subsequent live-game messages on the same connection. If Lightyear processes different message types in parallel or in type-registration order, an explicit `snapshot_confirmed` flag and a system ordering constraint are required.

12. [ ] **`C2SHeartbeat` on `UnreliableChannel` in WASM/WebSocket** — Lightyear's unreliable channel over WebSocket is application-layer unreliable. Verify that the WebSocket transport respects the channel mode and does not silently upgrade unreliable messages to reliable delivery. If WebSocket transport forces all messages to reliable, the heartbeat channel choice is cosmetic but the disconnect detection logic is still correct.

**OQ-D Invariant (Critical Ordering Constraint)**

`S2CResolutionEvent` MUST be received by the client before `S2CPhaseChanged(DRAFT_SHOP)`.

Both messages are on `ReliableChannel`. The server MUST enqueue `S2CResolutionEvent` before `S2CPhaseChanged` in the same `Update` schedule run. Use Bevy's system ordering to enforce this: the system that sends `S2CResolutionEvent` must be scheduled with `.before(send_phase_changed_system)` or placed in an earlier system set in the `Update` schedule.

Do NOT split these messages onto different channels. If they are ever moved to separate channels, an explicit `sub_step_sequence_number` field must be added to each message type and the client must implement a reorder buffer — a significant complexity cost that same-channel ordering avoids for free.

**Reconnect Invariant**

After transport reconnect, the server sends `S2CGameSnapshot` before any live game messages. The implementation must:

1. Set a per-connection `snapshot_sent: bool = false` flag on `OnConnected`.
2. In the snapshot system (scheduled first in `Update`), send `S2CGameSnapshot` and set `snapshot_sent = true`.
3. All other game systems that send live S2C messages must check `snapshot_sent` before enqueuing. If `snapshot_sent = false` for the target connection, buffer the message and flush after snapshot delivery.
4. Schedule the snapshot system before all live-game message systems in the same `Update` schedule to prevent same-frame races.

This invariant is acceptance criterion NP-9 in the Network Protocol GDD and is BLOCKING.

**`S2CAuctionUpdate` — Unreliable Channel**

`S2CAuctionUpdate` (carrying the live auction timer countdown and current price) is the only S2C message on `UnreliableChannel`. The client renders the timer from the most recently received packet; stale or out-of-order packets from the same tick may be rendered briefly then overwritten. The authoritative auction result is always `S2CAuctionSettled` on `ReliableChannel` — `S2CAuctionUpdate` is display-only and a dropped packet causes at most one frame of stale timer display.

Note: `S2CAuctionBidAccepted` is on `ReliableChannel` because it changes the authoritative bid state that drives the client's bidding UI (minimum valid next bid amount). It is not the same as the timer update.

## Alternatives Considered

### Alternative 1: Three Channels (Reliable, Sequenced-Unreliable, Fire-and-Forget Unreliable)

- **Description**: Add a third `SequencedUnreliableChannel` that drops out-of-order packets, for messages like the auction timer where delivering a stale packet is harmful. Standard in games with UDP transports (e.g., GGPO, Valve netcode).
- **Pros**: Eliminates the stale-auction-timer display artifact. Semantically cleaner for high-frequency state updates.
- **Cons**: Lightyear 0.26 channel primitives may not expose a first-class sequenced-unreliable mode distinct from unordered-unreliable. Over WebSocket (our WASM transport), TCP framing makes out-of-order delivery rare to impossible in practice — the distinction is largely moot. Adds a third channel type to verify and maintain.
- **Estimated Effort**: Low if Lightyear supports it; medium if it requires a custom channel implementation.
- **Rejection Reason**: Over WebSocket/WebTransport, the practical difference between unordered-unreliable and sequenced-unreliable is negligible. A stale timer packet that arrives out of order will be overwritten within one render frame. The two-channel model is simpler to verify against Lightyear 0.26 and sufficient for our delivery tier requirements.

### Alternative 2: Single Reliable Channel for All Messages

- **Description**: Route every message — including `C2SHeartbeat` and `S2CAuctionUpdate` — through `ReliableChannel`.
- **Pros**: No channel selection logic. No risk of heartbeats being dropped and triggering false disconnects. Simpler protocol definition.
- **Cons**: Heartbeat retransmit pressure on the reliable queue during connection degradation is the worst time to add retransmit load — exactly when we are trying to detect a dropping connection. Auction timer ticks (fired ~10 times over 20 seconds) create 10 reliable retransmit entries per auction; stale retransmits arrive after the auction settles and must be explicitly ignored. Reliable channel head-of-line blocking means a dropped heartbeat retransmit delays all other reliable messages behind it.
- **Estimated Effort**: No extra effort — it is the simpler approach.
- **Rejection Reason**: Head-of-line blocking on reliable channels is a known WebSocket pathology. Placing heartbeats and display-only timer ticks on the reliable channel adds avoidable retransmit pressure in the scenarios (connection degradation) where that pressure is most costly.

### Alternative 3: Per-Message-Type Channels (One Channel Per Message)

- **Description**: Define a dedicated channel for each message type, similar to Lightyear's component replication approach.
- **Pros**: Maximum isolation — a large `S2CGameSnapshot` cannot block `S2CPhaseChanged`. Per-type priority possible in principle.
- **Cons**: Lightyear 0.26 channel registration is not free (each channel has bookkeeping overhead). The OQ-D invariant becomes impossible to enforce without explicit sequence numbers — per-type channels have no cross-type ordering guarantee. Dramatically increases the verification surface for implementation checklist items.
- **Estimated Effort**: High — every new message type requires a channel registration, ordering annotation, and per-channel tests.
- **Rejection Reason**: Breaks the OQ-D invariant at zero marginal benefit for our message volume. Rejected.

## Consequences

### Positive

- Channel assignment is a one-time lookup per message type — no runtime logic, no conditional channel selection, no per-frame decisions.
- The OQ-D invariant (`S2CResolutionEvent` before `S2CPhaseChanged`) is enforced by same-channel enqueue order alone. No application-level sequence numbers, no client reorder buffer.
- `C2SHeartbeat` and `S2CAuctionUpdate` on `UnreliableChannel` keep the reliable retransmit queue clear of high-frequency, low-priority traffic.
- All channel definitions live in `shared/src/protocol.rs` — client and server share the same types. A channel mismatch between client and server is a compile error, not a runtime desync.
- The assignment table in this ADR is the single source of truth. Reviewers checking whether a new message belongs on reliable or unreliable have a clear precedent for every existing type.

### Negative

- Over WebSocket in WASM, `UnreliableChannel` is application-layer unreliable only — TCP still buffers and orders packets at the transport layer. The semantic distinction provides correctness guarantees but the practical delivery behaviour is mostly the same as reliable for short-lived connections with good signal.
- The reconnect invariant (snapshot before live messages) requires explicit per-connection state (`snapshot_sent` flag) and system scheduling discipline. This is not enforced by the channel configuration — it is a convention that every message-sending system must respect.
- `S2CAuctionUpdate` on unreliable means the client may occasionally render a briefly stale timer. This is an acceptable display artifact — `S2CAuctionSettled` is the authoritative terminal signal and always arrives on reliable.

### Neutral

- Component replication (`BoardPosition`, `UnitStats`, `ObjectiveHp`, `PrismPresence`) is handled by Lightyear's built-in replication system, not by these two channels. The channel table in this ADR covers explicit `MessageSender`/`MessageReceiver` messages only.
- Future message types not listed here must be assigned a channel by the implementing programmer before the implementing story is marked done. The default assignment is `ReliableChannel` unless the message meets the unreliable criterion: dropped packet is immediately superseded, stale arrival causes no state corruption, and the message is sent more than once per phase.

## Risks

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| Lightyear 0.26 channel definition syntax differs from expected `#[derive(Channel)]` pattern | HIGH | High — compilation failure on first build | Implementation checklist item 1 must be resolved before any networking code is written. Spike against `docs.rs/lightyear/0.26` before coding starts. |
| `NetworkTarget::Single` or server targeted-send API drifts in a future Lightyear upgrade | LOW | High — all unicast messages route to wrong clients or fail at runtime | Checklist items 7–9 are resolved for Lightyear 0.26.4; re-run the verification evidence if upgrading Lightyear. Keep a minimal channel send compile test in networking code. |
| In-order cross-type delivery on `ReliableChannel` is not guaranteed by Lightyear 0.26 | MEDIUM | Critical — breaks OQ-D invariant; resolution events arrive after phase change on client | Checklist item 10. If not guaranteed, add `sequence_id: u32` to `S2CResolutionEvent` and `S2CPhaseChanged` and implement client reorder buffer. |
| Snapshot delivery sequencing fails under concurrent system execution | LOW | High — live game messages arrive before snapshot on reconnect; client renders corrupt state | Checklist item 11. Enforce with Bevy system ordering (`.before()`) and `snapshot_sent` per-connection flag. |
| WASM WebSocket transport silently upgrades unreliable to reliable | LOW | Low — heartbeat and timer ticks work correctly regardless; only the performance optimisation is lost | Checklist item 12. Accept the cost if confirmed — correctness is unaffected. |

## Performance Implications

| Metric | Before | Expected After | Budget |
|--------|--------|----------------|--------|
| Per-round reliable message bytes | N/A (greenfield) | < 900 bytes (snapshot) + ~200 bytes (phase messages) | < 1 KB round message target |
| Reliable retransmit queue depth (auction round) | N/A | 0 entries for timer ticks (unreliable) vs. 10 entries if on reliable | Minimise reliable queue during auction |
| `C2SHeartbeat` channel overhead | N/A | ~10 bytes per 5s (unreliable, no retransmit) | Negligible |
| `S2CAuctionUpdate` send frequency | N/A | ~1 packet/second × 20s = 20 packets (unreliable) | ~20 × 20 bytes = 400 bytes per auction; no retransmit cost |

## Migration Plan

This is a greenfield decision — no existing networking code requires migration.

1. Verify all Implementation Guidelines checklist items against `docs.rs/lightyear/0.26` before writing any `MessageSender` or `MessageReceiver` code.
2. Add `ReliableChannel` and `UnreliableChannel` definitions to `shared/src/protocol.rs`.
3. Register both channels in the Lightyear `ProtocolPlugin` (or equivalent registration point — verify exact registration API, checklist item 1).
4. Write a minimal integration test that sends one message on each channel and verifies delivery. This test doubles as verification for checklist items 4, 5, and 6.
5. Implement message sends for each system in the order defined by the story roadmap, using the assignment table in Key Interfaces as the reference.

**Rollback plan**: If Lightyear 0.26 cannot support the two-channel model as described (e.g., no unreliable channel mode), fall back to Alternative 2 (single reliable channel). Update this ADR to Superseded, write ADR-008b with the single-channel rationale, and re-evaluate `C2SHeartbeat` disconnect detection (it becomes fully reliable — remove the application-layer timer redundancy concern).

## Validation Criteria

- [ ] `ReliableChannel` and `UnreliableChannel` types compile against Lightyear 0.26 without `#[allow(deprecated)]` or workaround attributes.
- [ ] A minimal two-player integration test (not `World::new()` — requires a live Lightyear session) confirms that a message sent on `ReliableChannel` from the server is received by the client with the same content and in enqueue order relative to a second `ReliableChannel` message sent immediately after.
- [ ] NP-8 acceptance criterion passes: `S2CResolutionEvent` is observed in the reliable channel stream before `S2CPhaseChanged(DRAFT_SHOP)` in a simulated resolution → phase-exit sequence.
- [ ] NP-9 acceptance criterion passes: `S2CGameSnapshot` is the first S2C message processed by a reconnecting client — no `S2CPhaseChanged` or `S2CGoldUpdate` precedes it in the channel stream.
- [ ] `C2SHeartbeat` sent on `UnreliableChannel` is received by the server and resets `disconnect_trackers[player]` (NP-24). Verify even if the WebSocket transport makes delivery effectively reliable.
- [ ] `S2CAuctionUpdate` packets sent in rapid succession (simulating 10 timer ticks) do not appear in the reliable retransmit queue after the connection artificially drops and recovers — they are discarded as unreliable.
- [ ] All implementation checklist items (1–12) are checked off and recorded with the relevant `docs.rs` or source link as evidence before any networking story is marked Done.

## GDD Requirements Addressed

| GDD Document | System | Requirement | How This ADR Satisfies It |
|-------------|--------|-------------|--------------------------|
| `design/gdd/network-protocol.md` | Network Protocol | Rule 2 — Channel assignment: every message uses exactly one channel tier (reliable ordered-guaranteed vs. unreliable best-effort) | Defines both channels and assigns every message type in the protocol to one |
| `design/gdd/network-protocol.md` | Network Protocol | TR-NP-07 — Disconnect detection via `C2SHeartbeat` (WASM half-open TCP state) | `C2SHeartbeat` assigned to `UnreliableChannel`; justification and disconnect detection correctness documented |
| `design/gdd/network-protocol.md` | Network Protocol | TR-NP-08 — `S2CAuctionUpdate` high-frequency timer delivery | Assigns `S2CAuctionUpdate` to `UnreliableChannel`; rate and stale-superseded rationale documented |
| `design/gdd/network-protocol.md` | Network Protocol | TR-NP-09 — `S2CResolutionEvent` before `S2CPhaseChanged(DRAFT_SHOP)` ordering (OQ-D invariant) | OQ-D invariant documented; enforced by same-channel enqueue order on `ReliableChannel`; same-channel split explicitly forbidden |
| `design/gdd/network-protocol.md` | Network Protocol | TR-RSM-09 — Phase transitions must be delivered reliably and in order | `S2CPhaseChanged` assigned to `ReliableChannel` |
| `design/gdd/network-protocol.md` | Network Protocol | TR-BLS-10 — Economy state (`S2CGoldUpdate`, `S2CGoldBroadcast`) must be consistent across clients | Both assigned to `ReliableChannel` |
| `design/gdd/network-protocol.md` | Network Protocol | TR-NP-05 — Reconnect snapshot delivered before all live messages | Reconnect invariant and `snapshot_sent` flag mechanism documented in Implementation Guidelines |
| `design/gdd/network-protocol.md` | Network Protocol | Open Question 3 — Reliable channel in-order delivery across message types | Resolution: use same channel (not cross-channel ordering); verification checklist item 10 |

## Related

- `docs/architecture/adr-001-objective-identity-unicast.md` — Establishes owner-only unicast as the pattern used by the majority of unicast S2C messages in this ADR's assignment table. Lightyear 0.26.4 resolves the concrete target as `NetworkTarget::Single(PeerId)`.
- ADR-002 (pending) — Client-server authority model. This ADR assumes the server is authoritative and all channel sends originate from the server for S2C and from the client for C2S. If ADR-002 changes this model, channel direction assignments in `ChannelSettings` must be updated.
- ADR-003 (pending) — Workspace layout. This ADR assumes `shared/src/protocol.rs` as the file where channel definitions live. If ADR-003 defines a different layout, update the file references in Key Interfaces and Implementation Guidelines.
- `design/gdd/network-protocol.md` — Full message catalogue, payload schemas, and acceptance criteria (NP-7 through NP-29) that this ADR's channel assignments must satisfy.
- `docs/engine-reference/bevy/current-best-practices.md` — Lightyear 0.26 `MessageSender`/`MessageReceiver` usage examples that informed the Key Interfaces section.
