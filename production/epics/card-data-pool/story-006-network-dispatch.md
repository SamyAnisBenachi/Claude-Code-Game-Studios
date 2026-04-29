# Story 006: Network Dispatch

> **Epic**: Card Data & Pool
> **Status**: Ready
> **Layer**: Core
> **Type**: Integration
> **Manifest Version**: 2026-04-29

## Context

**GDD**: `design/gdd/card-data-pool.md`
**Requirement**: TR-CDP-09 (shop slots and draft offering unicast to correct player via `ReliableChannel`); RNG audit log integration; reconnect guard

**ADRs Governing Implementation**:
- ADR-006: Card Data Schema and Pool State Architecture — `S2CShopSlots` and `S2CDraftOffering` are unicast messages sent on `ReliableChannel` to the owning `PlayerId`; client is a view only, no pool state on client
- ADR-008: Lightyear Channel Configuration — `ReliableChannel` guarantees ordered delivery; shop slot messages must not arrive out-of-order; use this channel for all shop-related unicast
- ADR-010: RSM Phase Event Bus — network dispatch is downstream of the pool refresh subscriber; dispatch reads `EventReader<S2CShopSlots>` and `EventReader<S2CDraftOffering>` enqueued by Stories 004 and 005

**ADR Decision Summary**: Network dispatch lives in `server/src/network/` — NOT in `server/src/core/pool/`. Core pool emits ECS events; the network layer consumes them and sends Lightyear unicast messages. This separation keeps the core pool free of Lightyear dependencies. The dispatch system guards on `ReconnectTracker.snapshot_sent` before enqueuing a message — if the reconnecting client has not yet received the session snapshot, the shop message is queued (or regenerated post-snapshot delivery). Each draw produces one `RngEvent::DrawShopSlot` entry in `ServerRng.audit_log`; the integration test verifies 3 draws produce exactly 3 audit entries.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: MEDIUM
**Engine Notes**:
- `liv-bevy-lightyear` skill is MANDATORY for all files in `server/src/network/` that touch Lightyear send APIs.
- `liv-bevy-018` skill is also MANDATORY — Lightyear 0.26 runs on Bevy 0.18; both skills apply simultaneously.
- Unicast send in Lightyear 0.26: use `ConnectionManager.send_message_to_target::<ReliableChannel, _>(client_id, message)`. Verify exact API surface against `liv-bevy-lightyear` skill — the send API changed between Lightyear 0.20 and 0.26.
- `EventReader::read()` (not `.iter()`) — Bevy 0.16 rename.
- The `ReconnectTracker` resource name and `snapshot_sent` field: confirm exact type against `game-session-system` epic deliverables before implementing.

**Control Manifest Rules (Network layer)**:
- Required: All network sends use `ReliableChannel` for shop slot messages (ordered, acknowledged delivery).
- Required: The dispatch system checks `ReconnectTracker.snapshot_sent` before sending. A client that has not received the session snapshot must not receive incremental shop updates — they will receive the full snapshot instead.
- Required: The network dispatch system lives in `server/src/network/pool_dispatch.rs` — not in `server/src/core/pool/`. Cross-layer boundary enforced by module ownership.
- Required: `RngEvent::DrawShopSlot` entries in `ServerRng.audit_log` count must equal the number of cards drawn (not slots attempted). A partial-fill draw that yields 2 cards from 3 slots produces 2 audit entries, not 3.
- Forbidden: No direct pool state reads in the network dispatch layer — the dispatch system reads only the outbound ECS events enqueued by the pool systems.

---

## Acceptance Criteria

- [ ] `server/src/network/pool_dispatch.rs` exists and defines:
  - `dispatch_shop_slots(mut events: EventReader<S2CShopSlots>, connection_manager: ResMut<ConnectionManager>, reconnect: Res<ReconnectTracker>)` — reads `S2CShopSlots` events; unicasts each on `ReliableChannel` to the target `client_id` mapped from `player_id`; skips if `!reconnect.snapshot_sent[player_id]`
  - `dispatch_draft_offering(mut events: EventReader<S2CDraftOffering>, connection_manager: ResMut<ConnectionManager>, reconnect: Res<ReconnectTracker>)` — same pattern for `S2CDraftOffering`
- [ ] Both dispatch systems are registered in a `PoolNetworkPlugin` (or added to the existing network plugin) with `.after(on_shop_refresh_needed)` and `.after(on_manual_refresh)` scheduling — dispatch always runs after the events are written
- [ ] Unicast correctness: GIVEN `S2CShopSlots { player_id: A, slots: [X, Y, Z] }` enqueued, WHEN dispatch runs, THEN `ConnectionManager.send_message_to_target(client_id_for_A, S2CShopSlots { ... })` is called exactly once; Player B does not receive Player A's shop slots
- [ ] Reconnect guard: GIVEN `ReconnectTracker.snapshot_sent[player_id] == false`, WHEN `S2CShopSlots` enqueued for that player, THEN the dispatch system does NOT call `send_message_to_target` for that player; the ECS event is consumed (not re-queued)
- [ ] Reconnect guard: GIVEN `ReconnectTracker.snapshot_sent[player_id] == true` (normal connected player), WHEN `S2CShopSlots` enqueued, THEN message is dispatched normally
- [ ] RNG audit log — 3 draws produce 3 entries: GIVEN a `refresh_shop(pool, ..., slot_count=3)` call that draws 3 cards, WHEN the system completes, THEN `ServerRng.audit_log` contains exactly 3 `RngEvent::DrawShopSlot` entries with the correct `player_id` and ascending `slot_index` (0, 1, 2)
- [ ] RNG audit log — partial fill: GIVEN a pool where only 2 eligible cards remain, WHEN `refresh_shop(pool, ..., slot_count=3)` completes, THEN `ServerRng.audit_log` contains exactly 2 `RngEvent::DrawShopSlot` entries (not 3)
- [ ] `S2CDraftOffering` is dispatched on `ReliableChannel` for DRAFT_INITIAL; `S2CShopSlots` is dispatched on `ReliableChannel` for DRAFT_SHOP and manual refresh
- [ ] Integration test: mock `ConnectionManager` records all calls to `send_message_to_target`; assert that after `ShopRefreshNeeded { player: A }` → pool draw → dispatch, exactly one call with `client_id_for_A` and payload matching `ShopSlots[A]` is recorded; no call with `client_id_for_B`
- [ ] `cargo check -p server` passes after adding `pool_dispatch.rs` and the network plugin registration

---

## Implementation Notes

*Derived from EPIC.md §Network dispatch wiring, ADR-006 §Key Interfaces, ADR-008 §Channel Config, ADR-010 §Subscriber Contracts:*

**Module boundary discipline:** The core pool (`server/src/core/pool/`) emits `S2CShopSlots` and `S2CDraftOffering` as ECS events. The network dispatch (`server/src/network/pool_dispatch.rs`) reads those events and calls Lightyear. Neither layer imports from the other's module — they communicate only through the shared ECS event types defined in `shared/src/protocol.rs`. This is the Boundary 3 rule from `architecture.md`.

**Lightyear 0.26 unicast pattern (verify with `liv-bevy-lightyear` skill):**
```rust
connection_manager.send_message_to_target::<ReliableChannel, _>(
    NetworkTarget::Single(client_id),
    &message,
)?;
```
`client_id` is a `ClientId` (Lightyear type) mapped from `PlayerId` via a `PlayerClientMap` resource (defined in the Game Session System or network protocol epic). If this mapping resource is not yet available, stub with `// TODO: map PlayerId to ClientId from PlayerClientMap`.

**`ReconnectTracker` guard implementation:**
```rust
fn dispatch_shop_slots(
    mut events: EventReader<S2CShopSlots>,
    connection_manager: ResMut<ConnectionManager>,
    reconnect: Res<ReconnectTracker>,
) {
    for event in events.read() {
        if reconnect.snapshot_sent.get(&event.player_id).copied().unwrap_or(false) {
            let client_id = ...; // map from player_id
            let _ = connection_manager.send_message_to_target::<ReliableChannel, _>(
                NetworkTarget::Single(client_id),
                &S2CShopSlots { slots: event.slots.clone() },
            );
        }
        // else: silently consume — reconnect snapshot will carry current state
    }
}
```

**RNG audit log integration:** The audit log entries are written by `ServerRng.next_seed()` inside `draw()` (Story 002). This story does not write audit entries directly — it only verifies in the integration test that the expected number of entries appeared. The test checks `Res<ServerRng>.audit_log.iter().filter(|e| matches!(e, RngEvent::DrawShopSlot { player_id: P, .. })).count() == expected`.

**`PoolNetworkPlugin`:** If a network plugin already exists (from `lightyear-protocol-verification` epic), add the dispatch systems there rather than creating a new plugin. Coordinate with the network programmer.

---

## Out of Scope

- Story 004: `S2CShopSlots` and `S2CDraftOffering` event enqueueing (pool-side)
- Story 005: Manual refresh `S2CShopSlots` enqueueing
- `ReconnectTracker` resource definition — Game Session System epic (ADR-011/ADR-012)
- `PlayerClientMap` resource (PlayerId → Lightyear ClientId mapping) — Game Session System or network protocol epic
- Reconnect snapshot delivery (`S2CReconnectSnapshot`) — Game Session System epic owns the full reconnect flow; this story only applies the guard
- `S2CHandUpdate` dispatch — Card Acquisition epic owns hand state; this story does not touch hand messages

---

## QA Test Cases

- **Unicast correctness — Player A's shop does not reach Player B**
  - Given: mock `ConnectionManager` tracking all `send_message_to_target` calls; two connected players A and B; `snapshot_sent[A] = true`, `snapshot_sent[B] = true`
  - When: `S2CShopSlots { player_id: A, slots: [1, 2, 3] }` enqueued; dispatch runs
  - Then: exactly one `send_message_to_target` call with `client_id_for_A`; zero calls with `client_id_for_B`

- **Reconnect guard — message held when snapshot not yet sent**
  - Given: player A reconnecting; `snapshot_sent[A] = false`
  - When: `S2CShopSlots { player_id: A, ... }` enqueued; dispatch runs
  - Then: zero `send_message_to_target` calls; event consumed

- **RNG audit log — 3 draws produce 3 entries**
  - Given: pool with >= 3 eligible cards; `slot_count = 3`
  - When: `refresh_shop()` called
  - Then: `ServerRng.audit_log` contains exactly 3 `DrawShopSlot` entries with `slot_index` in {0, 1, 2}

- **RNG audit log — partial fill produces correct entry count**
  - Given: pool with exactly 2 eligible cards; `slot_count = 3`
  - When: `refresh_shop()` called
  - Then: `ServerRng.audit_log` contains exactly 2 `DrawShopSlot` entries (not 3)

---

## Test Evidence

**Story Type**: Integration
**Required evidence**: `tests/integration/pool/network_dispatch_test.rs` — all acceptance criteria passing; covers unicast correctness, reconnect guard (snapshot_sent = false and true), RNG audit log entry count for full and partial fills
**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 004 (provides `S2CShopSlots` and `S2CDraftOffering` event enqueueing; `CardPoolPlugin`)
- Depends on: Story 005 (provides manual refresh `S2CShopSlots` enqueueing; all pool systems registered)
- Depends on: `workspace-and-shared-types` Story 004 — `S2CShopSlots` and `S2CDraftOffering` message types in `shared/src/protocol.rs`; `C2SShopRefresh` type
- Depends on: `lightyear-protocol-verification` epic — `ConnectionManager`, `ReliableChannel`, `NetworkTarget`, `ClientId` Lightyear types verified against 0.26 API; `liv-bevy-lightyear` skill patterns confirmed
- Depends on: `game-session-system` epic — `ReconnectTracker` resource with `snapshot_sent: HashMap<PlayerId, bool>`; `PlayerClientMap` resource mapping `PlayerId` to Lightyear `ClientId`
- Depends on: `server-rng` Story 001 — `RngEvent::DrawShopSlot` variant in audit log; `ServerRng.audit_log` access pattern
- Completes the Card Data & Pool epic — all six stories done unblocks the Auction System GDD (M2) and Card Acquisition epic
