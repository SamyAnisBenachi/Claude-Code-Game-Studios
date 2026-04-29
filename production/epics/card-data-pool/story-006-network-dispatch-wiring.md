# Story 006: Network Dispatch Wiring

> **Epic**: Card Data & Pool
> **Status**: Ready
> **Layer**: Core
> **Type**: Integration
> **Manifest Version**: 2026-04-29

## Context

**GDD**: `design/gdd/card-data-pool.md`
**Requirement**: `TR-CDP-09` (S2CShopSlots / S2CDraftOffering unicast)
*(TR-IDs are informal — `docs/architecture/tr-registry.yaml` is unpopulated.)*

**ADRs Governing Implementation**:
- [ADR-010: RSM Phase Event Bus](../../../docs/architecture/adr-010-rsm-event-bus.md) — `BroadcastPhaseChanged` is always last; shop slot messages must be enqueued before `BroadcastPhaseChanged`
- ADR-008: Lightyear Channel Config — `S2CShopSlots` and `S2CDraftOffering` go on `ReliableChannel`; unicast target per player

**ADR Decision Summary**: A system in `server/src/network/` reads `ShopSlots` / `InitialDraftOffering` resources and sends unicast `S2CShopSlots` / `S2CDraftOffering` on `ReliableChannel` to each player's `ClientId`. Must run after Stories 004/005 populate the resources. `ReconnectTracker` integration (deferred — not yet defined in any ADR).

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH (Lightyear 0.26 is post-LLM-cutoff; unicast send API not verified)
**Engine Notes**:
- `liv-bevy-lightyear` skill is **mandatory** on every `.rs` file in this story.
- Lightyear 0.26 unicast API must be verified before implementing: `NetworkTarget::Single(ClientId)` vs other variant — unverified. See control manifest Lightyear 0.26 verification checklist items #7 and #9.
- `ReliableChannel` struct name and registration API — verify against Lightyear 0.26 docs (checklist #1).
- Server unicast send: `server.send_message_to_target::<Channel, Msg>(msg, target)` — confirm exact signature (checklist #9).

**Control Manifest Rules (Core layer + Foundation)**:
- Required: `S2CShopSlots` and `S2CDraftOffering` on `ReliableChannel`. Never on `UnreliableChannel`.
- Required: Client notification via network only AFTER server state is ready (`ShopSlots` / `InitialDraftOffering` populated first).
- Required: Unicast per player — never broadcast shop slot contents to all clients.
- Required: Systems sending unicast S2C must check `ReconnectTracker.snapshot_sent[player]` before enqueuing — **DEFERRED** (see below).
- Forbidden: Sending opponent's shop contents to wrong client.

---

## Acceptance Criteria

*QA Lead verdict: ADEQUATE with 2 ACs DEFERRED pending architecture work.*

- [ ] **AC-1**: GIVEN `ShopSlots[player_A]` populated with 3 `CardId`s, WHEN the network dispatch system runs, THEN a `S2CShopSlots` message is assembled with those 3 slots and enqueued for unicast to `player_A`'s `ClientId` on `ReliableChannel`. *(Unicast routing verified via pure payload assembly test — see DEFERRED note for routing verification.)*
- [ ] **AC-2**: GIVEN `InitialDraftOffering[player_A]` populated with 9 `CardId`s (DRAFT_INITIAL), WHEN network dispatch runs, THEN a `S2CDraftOffering` message is assembled with those 9 IDs and enqueued for unicast to `player_A`'s `ClientId` on `ReliableChannel`. *(Same DEFERRED note.)*
- [ ] **AC-3**: GIVEN `ShopSlots[player_A]` contains a partial fill (e.g., Vec with 2 entries after pool exhaustion), WHEN `S2CShopSlots` assembled, THEN the `slots` field in the message has length 2 — partial fill faithfully represented.
- [ ] ~~**AC-4** (DEFERRED)~~: GIVEN `ReconnectTracker.snapshot_sent[player_A] == false`, WHEN `S2CShopSlots` for `player_A` would be sent, THEN message deferred until `snapshot_sent` is true. **DEFERRED — `ReconnectTracker` is not yet defined in any ADR. Implement after ADR-011 or a new ADR defines this struct.**

---

## DEFERRED Items

**AC-1/AC-2 unicast routing verification**: Testing that `S2CShopSlots` arrives at the correct `ClientId` and NOT at opponent's `ClientId` requires a Lightyear 0.26 test harness. This harness is not available in headless unit tests.

Resolution: AC-1 and AC-2 are implemented as **payload assembly tests** (pure function, no networking). The unicast routing itself is marked DEFERRED — manual evidence after the Lightyear spike confirms the send API.

**AC-4 (`ReconnectTracker`)**: `ReconnectTracker.snapshot_sent` is referenced in the control manifest but not defined in any existing ADR. Before implementing AC-4:
1. Author an ADR (or extend ADR-011) defining `ReconnectTracker { snapshot_sent: HashMap<PlayerId, bool> }`, its location, and which system sets it.
2. Then implement the guard in this story's dispatch system.

Until then, Story 006 ships without reconnect guard. The reconnect safety net is required before any multiplayer public release — flag in `tech-debt` register.

---

## Implementation Notes

*From ADR-010 and ADR-008:*

**File location**: `server/src/network/pool_dispatch.rs` (or extend an existing network dispatch file in `server/src/network/`)

**Dispatch system** (conceptual — verify exact Lightyear 0.26 API with `liv-bevy-lightyear` skill):
```rust
fn dispatch_shop_slots(
    shop_slots: Res<ShopSlots>,
    mut server: ResMut<RenetServer>,  // or Lightyear equivalent
    client_map: Res<ClientIdMap>,     // PlayerId → ClientId mapping
) {
    for (player_id, slots) in shop_slots.0.iter() {
        let Some(client_id) = client_map.get(player_id) else { continue; };
        let msg = S2CShopSlots { slots: slots.clone() };
        // Lightyear 0.26 unicast — verify exact call:
        server.send_message_to_target::<ReliableChannel, _>(msg, NetworkTarget::Single(*client_id));
    }
}
```

**`S2CShopSlots` and `S2CDraftOffering`** message types are defined in `shared/src/protocol.rs`. Do NOT define them locally — use the shared protocol types.

**Scheduling**: The dispatch system should run after `on_shop_refresh_needed` and `on_manual_refresh` (Stories 004/005) have written to `ShopSlots`. Use system ordering or a dedicated `PostUpdate` dispatch set.

**Lightyear 0.26 verification checklist items** (from control manifest) that must be ✅ before merge:
- #1: `ReliableChannel` definition syntax
- #7: `NetworkTarget::Single(ClientId)` variant name
- #9: `server.send_message_to_target::<Channel, Msg>(msg, target)` signature

---

## Out of Scope

*Handled by neighbouring stories or other epics — do not implement here:*

- [Story 004]: Populating `ShopSlots` / `InitialDraftOffering` — must be DONE first
- [Story 005]: Updating `ShopSlots` on manual refresh — must be DONE first
- ADR-011 / future ADR: `ReconnectTracker` definition — gates AC-4
- `S2CPoolUpdate` delta messages (per-round copy count deltas) — separate network story, deferred to a future story

---

## QA Test Cases

*Written by QA Lead at story creation. AC-1/AC-2 are payload assembly tests; unicast routing is DEFERRED.*

- **AC-1** — `test_shop_slots_message_assembled_correctly`
  - Given: `ShopSlots(map with player_A → vec![card_1, card_2, card_3])`
  - When: Message assembly function called for `player_A`
  - Then: `S2CShopSlots { slots: vec![card_1, card_2, card_3] }` constructed correctly; `slots.len() == 3`
  - Edge cases: Empty ShopSlots map → no messages assembled; player not in map → no message

- **AC-2** — `test_draft_offering_message_assembled_correctly`
  - Given: `InitialDraftOffering(map with player_A → vec![c1, c2, ..., c9])`
  - When: Message assembly for `player_A`
  - Then: `S2CDraftOffering { offering: vec![c1..c9] }` constructed correctly; `offering.len() == 9`
  - Edge cases: Partial initial offering (7 cards) → `offering.len() == 7`; no panic

- **AC-3** — `test_partial_fill_reflected_in_shop_slots_message`
  - Given: `ShopSlots(map with player_A → vec![card_1, card_2])` (partial fill, only 2 cards)
  - When: Message assembly for `player_A`
  - Then: `S2CShopSlots { slots }.len() == 2` (not padded to 3 with None)
  - Edge cases: Empty vec → `S2CShopSlots { slots: vec![] }`; single card → len == 1

- **AC-4** — DEFERRED (ReconnectTracker not yet defined in any ADR)

---

## Test Evidence

**Story Type**: Integration
**Required evidence**:
- `tests/unit/network/shop_dispatch_test.rs` — payload assembly tests (AC-1, AC-2, AC-3) — must pass
- Unicast routing: DEFERRED — manual evidence after Lightyear spike confirms send API works end-to-end
- Lightyear 0.26 verification checklist items #1, #7, #9 must be marked ✅ before merge

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 004 (ShopSlots + InitialDraftOffering populated) must be **DONE**
- Depends on: Story 005 (ShopSlots updated on manual refresh) must be **DONE**
- Depends on: `S2CShopSlots` and `S2CDraftOffering` message types in `shared/src/protocol.rs` — Foundation `workspace-and-shared-types` epic
- Depends on: Lightyear spike (verify unicast API on `ReliableChannel` before implementing routing code)
- Blocked by (AC-4 only): `ReconnectTracker` ADR — write and accept before adding reconnect guard
- Unlocks: None — this is the final story in the card-data-pool epic
