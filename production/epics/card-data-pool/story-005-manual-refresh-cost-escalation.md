# Story 005: Manual Refresh Cost Escalation Integration

> **Epic**: Card Data & Pool
> **Status**: Ready
> **Layer**: Core
> **Type**: Integration
> **Manifest Version**: 2026-05-01

## Readiness Refresh

2026-05-03: Revalidated against control manifest version 2026-05-01.
The original story text used stale names from the pre-ADR-015 shop model:

- `C2SShopRefresh` is now `C2SRefreshShop` in `shared/src/protocol.rs`.
- Lightyear C2S inbound uses `MessageReceiver<C2SRefreshShop>`, not Bevy
  `MessageReader<C2SShopRefresh>`.
- Manual refresh counter state now lives on `PlayerShopState` as
  `refresh_count_this_draft`; the older `ManualRefreshCount` resource remains
  a Card Pool session resource but is not the manual refresh authority.
- Card Acquisition owns shop slots, refresh deduplication, Lightyear dispatch,
  and the manual refresh cost formula. Card Data & Pool owns the player pool API
  that the refresh path draws from.

QL-STORY-READY skipped - Lean mode.

---

## Context

**GDDs**:
- `design/gdd/card-data-pool.md`
- `design/gdd/card-acquisition.md`
- `design/gdd/economy-system.md`

**Requirements**:
- `TR-CA-004`: Manual refresh cost formula:
  `refresh_base_cost + min(count, refresh_cap)`; counter resets at DRAFT entry.
- `TR-ECO-008`: Refresh cost escalation: first refresh uses base cost, later
  refreshes escalate, and the counter resets at the start of each DRAFT phase.
- `TR-CDP-004`: Pool API draw helpers return `Option<CardId>` and never panic.

**ADRs Governing Implementation**:
- [ADR-006: Card Data Schema and Pool State Architecture](../../../docs/architecture/adr-006-card-data-schema.md) -
  `PlayerPools` is authoritative per-player pool state; draw helpers never
  panic on empty pools.
- [ADR-010: RSM Phase Event Bus](../../../docs/architecture/adr-010-rsm-event-bus.md) -
  `ShopRefreshTriggered` is a Bevy buffered Message emitted on DRAFT entry.
- [ADR-015: Card Acquisition Shop State Machine Architecture](../../../docs/architecture/adr-015-card-acquisition-shop-state.md) -
  Card Acquisition owns `ShopStates`, `PlayerHands`, manual refresh, and
  shop-slot network dispatch.

**ADR Decision Summary**: `card_acquisition_tick_system` is the single shop
state writer. It drains Lightyear `MessageReceiver<C2SRefreshShop>` values,
maps `RemoteId` to `PlayerId` through `PlayerConnectionMap`, phase-gates to
`ShopPhase::ShopActive`, computes cost from `Res<GameConfig>`, spends gold via
Economy API before drawing, draws replacement slots from `PlayerPools`, sends
`S2CShopSlots` only on success, and increments `refresh_count_this_draft` only
after successful spend and draw.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH

**Engine Notes**:
- Use Bevy `MessageReader<ShopRefreshTriggered>` only for the RSM internal
  DRAFT-entry bus.
- Use Lightyear `MessageReceiver<C2SRefreshShop>` for client-to-server manual
  refresh requests. Do not register this as a Bevy message.
- The request payload has no player field; resolve player identity from
  `RemoteId` and `PlayerConnectionMap`.
- `liv-bevy-018` is mandatory for Bevy `.rs` changes.
- `liv-bevy-lightyear` is mandatory for the Lightyear message receiver path.

**Control Manifest Rules (Core and Feature layers)**:
- Required: every C2S handler resolves sender identity, phase-gates, validates,
  then mutates authoritative state.
- Required: invalid C2S inputs are silently discarded; no error response to
  clients.
- Required: `GameConfig.refresh_base_cost` and `GameConfig.refresh_cap` are read
  from `Res<GameConfig>`; no hardcoded tuning values.
- Required: all `PlayerEconomy` mutations go through
  `server/src/core/economy/api.rs`.
- Required: Card Acquisition is the sole steady-state writer for `ShopStates`.
- Forbidden: Bevy `MessageReader<C2SRefreshShop>` or `app.add_message` for
  Lightyear C2S traffic.
- Forbidden: `unwrap()` in production paths.

---

## Acceptance Criteria

*From Card Acquisition GDD CA8/CA9/CA10/CA11/CA15/CA22 and Economy GDD
EC24/EC25/EC26, scoped to the Card Data & Pool integration path.*

- [ ] **AC-1 (first refresh succeeds)**: GIVEN `refresh_count_this_draft == 0`,
  `refresh_base_cost == 1`, player gold is sufficient, and shop phase is
  `ShopActive`, WHEN a Lightyear `C2SRefreshShop` request is processed, THEN
  gold decreases by 1, replacement shop slots are produced from the player's
  pool, an `S2CShopSlots` payload exists, and `refresh_count_this_draft == 1`.
- [ ] **AC-2 (escalated refresh succeeds)**: GIVEN
  `refresh_count_this_draft == 1`, `refresh_base_cost == 1`, and
  `refresh_cap == 1`, WHEN a second manual refresh is processed, THEN gold
  decreases by 2 and `refresh_count_this_draft == 2`.
- [ ] **AC-3 (cap is honored)**: GIVEN `refresh_count_this_draft > refresh_cap`,
  WHEN manual refresh is processed, THEN cost is
  `refresh_base_cost + refresh_cap`, not an unbounded counter value.
- [ ] **AC-4 (insufficient gold is silent no-op)**: GIVEN player gold is less
  than required refresh cost, WHEN `C2SRefreshShop` is processed, THEN gold,
  shop slots, and `refresh_count_this_draft` are unchanged and no `S2CShopSlots`
  payload is produced.
- [ ] **AC-5 (wrong phase is silent no-op)**: GIVEN the player's shop phase is
  `Inactive`, `DraftInitial`, or `AuctionLock`, WHEN `C2SRefreshShop` is
  processed, THEN no Economy call succeeds, no pool draw runs, no slots change,
  no `S2CShopSlots` payload is produced, and the counter is unchanged.
- [ ] **AC-6 (draw failure refunds)**: GIVEN gold spend succeeds but the pool
  integration cannot produce replacement slots, WHEN manual refresh is
  processed, THEN spent gold is refunded, no `S2CShopSlots` payload is
  produced, and the counter is unchanged.
- [ ] **AC-7 (draft-entry reset)**: GIVEN a previous DRAFT phase left
  `refresh_count_this_draft > 0`, WHEN a new DRAFT-entry
  `ShopRefreshTriggered` (`DraftInitial`, `AuctionLock`, `ShopOpen`, or
  `ShopUnlock`) is applied, THEN `refresh_count_this_draft == 0` before the
  next manual refresh cost is computed.

---

## Implementation Notes

**Primary implementation surface**:
- `server/src/feature/acquisition/system.rs`
- Existing helper functions to use in tests:
  - `manual_refresh_cost`
  - `process_manual_refresh_shop_request`
  - `apply_shop_refresh_trigger`

**Lightyear receiver surface**:

```rust
Query<(&RemoteId, &mut MessageReceiver<C2SRefreshShop>)>
```

The system should iterate `receiver.receive()`, resolve the sender to a
`PlayerId`, and drop requests from unknown senders without panicking.

**Manual refresh control flow**:

```rust
for _ in receiver.receive() {
    let Some(player_id) = resolve_player(remote, connections) else {
        continue;
    };
    let (result, slots) = process_manual_refresh_shop_request(..., player_id);
    if let Some(slots) = slots {
        send S2CShopSlots to that player's peer or defer for reconnect;
    }
}
```

**Cost formula**:

```rust
refresh_base_cost + refresh_count_this_draft.min(refresh_cap)
```

Use `saturating_add` to avoid overflow if a bad config or corrupted counter is
ever present.

**Economy invariant**: Spend before draw. If draw fails after spend, refund
immediately in the same function body. Rejected refreshes do not increment the
counter.

---

## Out of Scope

- Changing Card Pool draw algorithms or pool copy-count formulas.
- Implementing purchase flow atomicity; Card Acquisition Story 005 owns that.
- Changing Lightyear protocol registration beyond the existing
  `C2SRefreshShop` registration.
- Updating `production/session-state/active.md` or `production/sprint-status.yaml`.
- Editing duplicate stale Card Data & Pool Story 004 files.

---

## QA Test Cases

- **AC-1** - `test_first_manual_refresh_costs_base_gold`
  - Given: `ShopPhase::ShopActive`, `refresh_count_this_draft == 0`,
    `PlayerEconomy.gold == 10`, and a populated `PlayerPools` fixture.
  - When: `process_manual_refresh_shop_request` runs.
  - Then: gold is 9, counter is 1, and `S2CShopSlots` contains up to 3 slots.

- **AC-2** - `test_second_manual_refresh_cost_escalated`
  - Given: `refresh_count_this_draft == 1`, base 1, cap 1, and gold 10.
  - When: manual refresh runs.
  - Then: gold is 8 and counter is 2.

- **AC-3** - `test_refresh_cap_limits_cost`
  - Given: `refresh_count_this_draft == 5`, base 1, cap 1, and gold 10.
  - When: manual refresh runs.
  - Then: gold is 8, confirming cost 2.

- **AC-4** - `test_insufficient_gold_no_refresh`
  - Given: gold is below the computed cost.
  - When: manual refresh runs.
  - Then: gold, slots, and counter are unchanged; no slots payload is returned.

- **AC-5** - `test_wrong_phase_discards_refresh`
  - Given: phase is not `ShopActive`.
  - When: manual refresh runs.
  - Then: result is `DiscardedWrongPhase`; no state changes occur.

- **AC-6** - `test_draw_failure_refunds_gold`
  - Given: gold is sufficient but required pool/session/catalog data cannot
    produce slots.
  - When: manual refresh runs.
  - Then: gold is refunded and counter is unchanged.

- **AC-7** - `test_draft_entry_resets_refresh_count`
  - Given: `refresh_count_this_draft == 3`.
  - When: each relevant `ShopRefreshTriggered` variant is applied.
  - Then: `refresh_count_this_draft == 0`.

---

## Test Evidence

**Story Type**: Integration

**Required evidence**:
- `tests/integration/pool/manual_refresh_test.rs`
- Cargo test target: `cargo test -p server --test pool_manual_refresh_test`

**Status**: [x] Created and passing locally with
`cargo test -p server --test pool_manual_refresh_test` on 2026-05-03

---

## Dependencies

- Depends on: Card Data & Pool Story 004 is complete; `PlayerPools` lifecycle
  and teardown are implemented.
- Depends on: Card Acquisition Story 003 is complete; draw pipeline can produce
  shop slots from `PlayerPools`.
- Depends on: Card Acquisition Story 004 is complete; refresh formula and reset
  model exist.
- Depends on: Economy API is callable through
  `server/src/core/economy/api.rs`.
- Depends on: `C2SRefreshShop` exists and is registered in
  `shared/src/protocol.rs`.
- Unlocks: Pool manual refresh integration evidence for code review and story
  closure.

---

## Performance Budget

No new steady-state budget is expected beyond one C2S receiver drain per frame.
Manual refresh runs only on player request, draws at most three slots, and stays
within the server steady-state game-logic budget of 5 ms. Wrong-phase and
insufficient-gold requests return before pool drawing.
