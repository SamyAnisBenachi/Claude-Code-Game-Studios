# Story 006: Network Dispatch Wiring

> **Epic**: Card Data & Pool
> **Status**: Complete
> **Layer**: Core
> **Type**: Integration
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/card-data-pool.md`
**Requirement**: `TR-CDP-010` (S2CShopSlots / S2CDraftOffering reliable unicast)
**GDD trace**: `design/gdd/card-data-pool.md` Interactions with Other Systems -> Lightyear Network and acceptance criterion `CP-NET-01`.

**ADRs Governing Implementation**:
- [ADR-010: RSM Phase Event Bus](../../../docs/architecture/adr-010-rsm-event-bus.md) - `BroadcastPhaseChanged` is always last; shop slot messages must be enqueued before `BroadcastPhaseChanged`.
- [ADR-008: Lightyear Channel Config](../../../docs/architecture/adr-008-lightyear-channel-config.md) - `S2CShopSlots` and `S2CDraftOffering` go on `ReliableChannel`; unicast target per player.
- [ADR-011: Reconnect Snapshot](../../../docs/architecture/adr-011-reconnect-snapshot.md) - live unicast S2C is queued while `ReconnectTracker.snapshot_sent[player] == false`.

**ADR Decision Summary**: The Card Acquisition server tick prepares `S2CShopSlots` and `S2CDraftOffering` only after authoritative `ShopSlots` or `InitialDraftOffering` state exists. The dispatch path resolves the owning `PlayerId` to that player's `PeerId`, checks `ReconnectTracker.snapshot_sent[player]`, queues the message if the reconnect snapshot is still pending, and otherwise sends reliable unicast via `ServerMultiMessageSender::send::<MessageType, ReliableChannel>(&message, server, &NetworkTarget::Single(peer_id))`.

**Engine**: Bevy 0.18 + Lightyear 0.26.4 | **Risk**: HIGH (verified locally before implementation)
**Engine Notes**:
- `liv-bevy-lightyear` skill is mandatory on every `.rs` file in this story.
- Lightyear 0.26.4 unicast target is `NetworkTarget::Single(PeerId)`, verified by `tests/evidence/lightyear-026-verification.md` item #7 and current `server/src/feature/acquisition/system.rs` usage.
- Channel registration is plain channel structs plus `app.add_channel::<T>(ChannelSettings { mode, ..default() }).add_direction(NetworkDirection::Bidirectional)`, verified by ADR-008 checklist item #1 and local Lightyear 0.26.4 source.
- Server unicast send API is `ServerMultiMessageSender::send::<MessageType, ReliableChannel>(&message, server, &NetworkTarget::Single(peer_id))`, verified by `tests/evidence/lightyear-026-verification.md` item #9 and current server dispatch usage.

**Control Manifest Rules (Core layer + Foundation)**:
- Required: `S2CShopSlots` and `S2CDraftOffering` on `ReliableChannel`. Never on `UnreliableChannel`.
- Required: Client notification via network only after server state is ready (`ShopSlots` / `InitialDraftOffering` populated first).
- Required: Unicast per player; never broadcast shop slot contents to all clients.
- Required: Systems sending live unicast S2C must check `ReconnectTracker.snapshot_sent[player]` before sending and queue the message while the reconnect snapshot is pending.
- Forbidden: Sending an opponent's shop contents to the wrong client.

---

## Acceptance Criteria

- [x] **AC-1**: GIVEN `ShopSlots[player_A]` is populated with three `CardId` values and `ReconnectTracker.snapshot_sent[player_A] == true`, WHEN the Card Acquisition dispatch path runs, THEN it assembles `S2CShopSlots` with those three slots and targets only `player_A`'s `PeerId` using `ReliableChannel`.
- [x] **AC-2**: GIVEN `InitialDraftOffering[player_A]` is populated with nine `CardId` values for DRAFT_INITIAL and `ReconnectTracker.snapshot_sent[player_A] == true`, WHEN the Card Acquisition dispatch path runs, THEN it assembles `S2CDraftOffering` with those nine IDs and targets only `player_A`'s `PeerId` using `ReliableChannel`.
- [x] **AC-3**: GIVEN `ShopSlots[player_A]` contains a partial fill with two cards and one empty slot, WHEN `S2CShopSlots` is assembled, THEN the message preserves the empty slot as `None` and does not invent a replacement `CardId`.
- [x] **AC-4**: GIVEN `ReconnectTracker.snapshot_sent[player_A] == false`, WHEN `S2CShopSlots` or `S2CDraftOffering` for `player_A` would be sent, THEN the message is stored in `ReconnectTracker.deferred_queue[player_A]` and is not live-sent until the snapshot gate is open.

---

## Implementation Notes

*From ADR-010, ADR-008, and ADR-011:*

**Primary implementation surface**: `server/src/feature/acquisition/system.rs`

The existing Card Acquisition tick is the correct ownership boundary because it already observes the authoritative draft/shop resources and phase transition timing. Do not add a second independent dispatch loop that can double-send payloads.

**Dispatch shape**:
```rust
let target = NetworkTarget::Single(peer_id);
sender.send::<S2CShopSlots, ReliableChannel>(&message, server, &target);
sender.send::<S2CDraftOffering, ReliableChannel>(&message, server, &target);
```

**Reconnect guard**:
```rust
if !tracker.snapshot_sent.get(&player_id).copied().unwrap_or(true) {
    defer_unicast_for_reconnect(tracker, player_id, DeferredMessage::ShopSlots(message));
    return;
}
```

**Protocol types**: `S2CShopSlots` and `S2CDraftOffering` are defined in `shared/src/protocol.rs`. Do not define local duplicates.

**Scheduling**: Dispatch remains in the Card Acquisition tick path after Stories 004/005 populate or refresh `ShopSlots` / `InitialDraftOffering` and before phase broadcast consumers rely on client UI state.

**Lightyear 0.26 verification checklist evidence**:
- #1 `ReliableChannel` definition and registration syntax: resolved in ADR-008 checklist item #1 with Lightyear 0.26.4 source and `tests/evidence/lightyear-026-verification.md`.
- #7 unicast target type: resolved as `NetworkTarget::Single(PeerId)`.
- #9 server send signature: resolved as `ServerMultiMessageSender::send::<MessageType, ReliableChannel>(&message, server, &NetworkTarget::Single(peer_id))`.

---

## Out of Scope

*Handled by neighbouring stories or other epics; do not implement here:*

- [Story 004](story-004-shop-refresh-subscriber-session-ready.md): Populating `ShopSlots` / `InitialDraftOffering`; must already be done.
- [Story 005](story-005-manual-refresh-cost-escalation.md): Updating `ShopSlots` on manual refresh; must already be done.
- `S2CPoolUpdate` delta messages for per-round copy count deltas.
- Live browser receipt evidence that a remote client rendered the payload; this story verifies server-side target selection, message construction, and reconnect queue behavior.

---

## QA Test Cases

- **AC-1** - `test_shop_slots_message_targets_owner_peer`
  - Given: `ShopSlots` state for `player_A` and a peer mapping for `player_A`.
  - When: The shop slot dispatch helper prepares the outgoing message.
  - Then: `S2CShopSlots` contains `player_A`'s slots and the dispatch target is exactly `player_A`'s `PeerId`.
  - Edge cases: Missing peer mapping produces no live peer target.

- **AC-2** - `test_draft_offering_message_targets_owner_peer`
  - Given: `InitialDraftOffering` state for `player_A` containing nine cards and a peer mapping for `player_A`.
  - When: The draft offering dispatch helper prepares the outgoing message.
  - Then: `S2CDraftOffering` contains the nine-card offering and the dispatch target is exactly `player_A`'s `PeerId`.
  - Edge cases: A shorter offering is copied exactly and does not panic.

- **AC-3** - `test_partial_shop_slots_preserve_empty_slots`
  - Given: `ShopSlots[player_A]` contains `[Some(card_1), Some(card_2), None]`.
  - When: `S2CShopSlots` is assembled.
  - Then: The outgoing `slots` field equals `[Some(card_1), Some(card_2), None]`.

- **AC-4** - `test_dispatch_queues_messages_while_reconnect_snapshot_pending`
  - Given: `ReconnectTracker.snapshot_sent[player_A] == false`.
  - When: shop slots or draft offering would be sent to `player_A`.
  - Then: The payload is appended to `ReconnectTracker.deferred_queue[player_A]` and no live send target is required for that tick.

---

## Performance Budget

No new steady-state loop budget is introduced beyond the existing Card Acquisition server tick. Dispatch work must remain O(players) for payload preparation, O(1) for reconnect gate lookup per player, and within the server gameplay budget of 5 ms per tick for this subsystem path.

---

## Test Evidence

**Story Type**: Integration
**Required evidence**:
- `tests/unit/network/shop_dispatch_test.rs` - server-side payload assembly, owner peer targeting, partial slot preservation, and reconnect queue tests for AC-1 through AC-4.
- Existing reconnect regression coverage in `tests/integration/session/reconnect_snapshot_test.rs` may be referenced as supporting evidence, but the story-specific test file above is required.
- Lightyear 0.26.4 verification checklist items #1, #7, and #9 must remain checked before merge.

**Status**: [x] Created and passing via `cargo test -p server --test shop_dispatch_test`.

---

## Dependencies

- Depends on: `production/epics/card-data-pool/story-004-shop-refresh-subscriber-session-ready.md` is Complete and provides `ShopSlots` / `InitialDraftOffering` population.
- Depends on: `production/epics/card-data-pool/story-005-manual-refresh-cost-escalation.md` is Complete and provides manual refresh shop updates.
- Depends on: `S2CShopSlots` and `S2CDraftOffering` message types in `shared/src/protocol.rs`.
- Depends on: Lightyear 0.26.4 verification evidence in `tests/evidence/lightyear-026-verification.md` items #1, #7, and #9.
- Unlocks: None; this is the final story in the card-data-pool epic.

## Completion Notes

**Completed**: 2026-05-04
**Verdict**: COMPLETE WITH NOTES
**Criteria**: 4/4 passing. AC-1 through AC-4 are covered by `tests/unit/network/shop_dispatch_test.rs`; AC-4 also has supporting reconnect regression coverage in `tests/integration/session/reconnect_snapshot_test.rs`.
**Deviations**: None blocking. Note: `production/sprint-status.yaml` has no matching CDP-006/S5-04 row and was not updated per approval boundary.
**Test Evidence**: `cargo test -p server --test shop_dispatch_test` passed 5/5. `cargo test -p server --test reconnect_snapshot_test acquisition_unicast_helpers_defer_while_snapshot_pending` passed 1/1 filtered test. `cargo check -p server` passed.
**Code Review**: Skipped - lean mode.
