# Story 002: Live Draft/Shop/Hand Bridge

> **Epic**: Playable Client
> **Status**: Ready
> **Layer**: Polish / Client Integration
> **Type**: Integration
> **Manifest Version**: 2026-05-05
> **Sprint**: Sprint 7 / PLAYABLE-002

## Context

PLAYABLE-001 gets two real clients into a server-confirmed session. PLAYABLE-002 bridges the first playable card-acquisition loop through real WebSocket messages so DRAFT_INITIAL, DRAFT_SHOP, ready, hand, and economy state are live on the primary client instead of harness-injected.

**Primary sources**:

- `production/sprints/sprint-7.md`
- `production/sprint-status.yaml`
- `design/gdd/card-data-pool.md`
- `design/gdd/card-acquisition.md`
- `design/gdd/economy-system.md`
- `design/gdd/network-protocol.md`
- `design/gdd/shop-auction-ui.md`
- `design/gdd/hand-ui.md`
- `design/gdd/hud.md`
- `design/gdd/round-state-machine.md`

**GDD and TR trace**:

- `design/gdd/card-data-pool.md` / `TR-CDP-010`: `S2CDraftOffering` and `S2CShopSlots` are reliable unicast after authoritative state is populated and before client phase/UI use.
- `design/gdd/network-protocol.md` / `TR-NP-001`: client emits intent only and the server owns all game logic.
- `design/gdd/network-protocol.md` / `TR-NP-005`: invalid-phase C2S messages are silently discarded without an S2C rejection.
- `design/gdd/network-protocol.md` / `TR-NP-010`: `S2CGoldUpdate` and `S2CGoldBroadcast` carry economy projections on reliable channel.
- `design/gdd/hand-ui.md` / `TR-HU-005`: DRAFT_INITIAL grid overlay shows 9 cards, 45s timer, and 5g budget.
- `design/gdd/hand-ui.md` / `TR-HU-008`: placement submit pre-validation reads `PlayerEconomyView`; server validation remains authoritative.
- `design/gdd/hand-ui.md` / `TR-PRES-001`: Hand UI, HUD, and Shop/Auction UI read shared `PlayerEconomyView` rather than independently draining economy messages.
- `design/gdd/shop-auction-ui.md` / `TR-SAU-006`: panel transitions and input gating follow authoritative phase and S2C data.
- `design/gdd/round-state-machine.md` / `TR-RSM-004`: DRAFT entry event order is `DraftStarted`, `ShopRefreshNeeded`, optional `AuctionPhaseEntered`, then `BroadcastPhaseChanged`.
- `design/gdd/round-state-machine.md` / `TR-RSM-009`: phase changes are reliable and emitted last.

**ADR Governing Implementation**:

- [ADR-002: Client-Server Authority](../../../docs/architecture/adr-002-client-server-authority.md)
- [ADR-008: Lightyear Channel Configuration](../../../docs/architecture/adr-008-lightyear-channel-config.md)
- [ADR-010: RSM Phase Event Bus](../../../docs/architecture/adr-010-rsm-event-bus.md)
- [ADR-015: Card Acquisition Shop State](../../../docs/architecture/adr-015-card-acquisition-shop-state.md)
- [ADR-019: Economy Resource Architecture](../../../docs/architecture/adr-019-economy-resource-architecture.md)
- [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)

**ADR Decision Summary**: Draft/shop/hand UI may show pending interaction states, but card ownership, shop slots, hand contents, ready state, phase, and economy values are server-authoritative and must be projected from reliable S2C messages or snapshot state.

**Engine**: Bevy 0.18 + Lightyear 0.26 + browser/WASM primary client | **Risk**: HIGH

**Engine Notes**: Use `liv-bevy-018` before editing any Bevy `.rs` file and `liv-bevy-lightyear` before editing any Lightyear `.rs` file. UI work must use Required Components API and existing Presentation sets. Do not use `NodeBundle`, `TextBundle`, `SpriteBundle`, `UiImage::new()`, `Parent`, `EventReader`, `EventWriter`, `Events<T>`, or `Color::rgba()`.

**Lightyear Notes**: `C2SPurchaseCard`, `C2SRefreshShop`, `C2SSignalReady`, `S2CDraftOffering`, `S2CShopSlots`, `S2CCardAcquired`, `S2CGoldUpdate`, and `S2CGoldBroadcast` use `ReliableChannel`. `C2SSignalReady` must reach the server-side readiness path for DRAFT phases and must not remain a logging stub. Do not add duplicate production drains for S2C phase or economy messages.

**Control Manifest Rules (2026-05-05)**:

- Required: no optimistic client authority for purchase, ready, hand, shop, economy, or phase state.
- Required: `S2CPhaseChanged` is drained only by the shared phase sink.
- Required: `PlayerEconomyView` is seeded from `S2CGoldUpdate` and `S2CGameSnapshot`, then read by presentation sub-plugins.
- Required: Lightyear `MessageReceiver<T>` and Bevy `MessageReader<T>` stay distinct.
- Required: Card Acquisition owns `ShopStates` and `PlayerHands` writes in DRAFT phases.
- Guardrail: client S2C processing plus view update remains at or below 2 ms per frame.

---

## Scope

### In Scope

- Connect PLAYABLE-001 session entry to live DRAFT_INITIAL presentation.
- Ensure DRAFT_INITIAL card grid renders from real `S2CDraftOffering`.
- Ensure DRAFT_INITIAL purchase sends `C2SPurchaseCard` and hand/economy views update from authoritative `S2CCardAcquired` plus `S2CGoldUpdate`.
- Ensure `C2SSignalReady { retract: false }` and `C2SSignalReady { retract: true }` from DRAFT_INITIAL or DRAFT_SHOP reach the server-side draft-ready path.
- Ensure server-side draft-ready handling advances only when the authoritative all-ready condition is met.
- Ensure DRAFT_SHOP slots render from real `S2CShopSlots`.
- Ensure DRAFT_SHOP purchase, refresh, and ready controls use real WebSocket messages.
- Ensure Hand UI, HUD, and Shop/Auction UI show the same authoritative hand and economy outcome after purchase confirmation.
- Preserve auction display if the server enters DRAFT_AUCTION, but auction settlement polish remains conditional Sprint 7 should-have scope.

### Out of Scope

- PLAYABLE-001 lobby/session bootstrap changes.
- PLAYABLE-003 two-real-client manual end-to-end evidence.
- Public release readiness, store readiness, deployment readiness, broad accessibility completion, playtest validation, fun-hypothesis validation, or full playable-client manual QA.
- Editing `QA-COND-0005` or `QA-COND-0006` accepted-risk disposition.
- New card acquisition rules, balance tuning, card data expansion, or client-side card-pool simulation.
- Optimistic client-side card ownership or local economy mutation before authoritative S2C.
- Full auction settlement polish unless needed as a direct blocker to the friend-game path.

---

## Acceptance Criteria

- [ ] **DRAFT_INITIAL offering is live**: GIVEN PLAYABLE-001 has placed the client in `ClientState::InSession`, WHEN the server sends `S2CDraftOffering`, THEN the primary client renders the 9-card DRAFT_INITIAL grid from that payload and does not use harness card IDs.
- [ ] **DRAFT_INITIAL purchase is authoritative**: GIVEN a DRAFT_INITIAL card is affordable and clicked, WHEN the click is processed, THEN exactly one `C2SPurchaseCard` is sent over `ReliableChannel`; the card appears in the client hand only after authoritative `S2CCardAcquired` or snapshot state includes it.
- [ ] **Economy projection stays shared**: GIVEN purchase confirmation changes gold or hand state, WHEN the client receives `S2CGoldUpdate` and related acquisition messages, THEN `PlayerEconomyView`, HUD, Hand UI, and Shop/Auction UI converge on the same gold/current mana/reserve mana/hand state without any extra `S2CGoldUpdate` drainer.
- [ ] **Draft ready reaches server RSM path**: GIVEN either client clicks Ready in DRAFT_INITIAL or DRAFT_SHOP, WHEN `C2SSignalReady { retract: false }` reaches the server, THEN the server writes the owning draft-ready signal or equivalent RSM input for that player and does not leave the message as a logging-only stub.
- [ ] **Ready retract is live**: GIVEN a player has clicked Ready but the phase has not advanced, WHEN they click Retract Ready, THEN `C2SSignalReady { retract: true }` reaches the same server authority path and the ready state changes only from authoritative state.
- [ ] **All-ready phase progression is server-owned**: GIVEN both real clients are ready in the active draft phase, WHEN the server processes the all-ready condition, THEN any phase advance comes from RSM/server state and reaches both clients as `S2CPhaseChanged`.
- [ ] **DRAFT_SHOP slots are live**: GIVEN the server enters DRAFT_SHOP, WHEN `S2CShopSlots` arrives, THEN the shop panel renders exactly the server slots, including empty slots, and does not generate local shop contents.
- [ ] **DRAFT_SHOP purchase and refresh use real messages**: GIVEN a valid DRAFT_SHOP purchase or refresh click, WHEN the UI processes the action, THEN it sends `C2SPurchaseCard` or `C2SRefreshShop` once per valid click and waits for `S2CCardAcquired`, `S2CShopSlots`, and economy S2C to update visible state.
- [ ] **Snapshot recovery seeds the bridge**: GIVEN a reconnect or late snapshot occurs during DRAFT_INITIAL or DRAFT_SHOP, WHEN `S2CGameSnapshot` arrives, THEN hand, shop slots, economy, phase, and board-facing presentation resources rebuild from snapshot state before additional live messages become actionable.
- [ ] **Regression commands pass**: `cargo test -p client --test playable_client_draft_shop_hand_bridge_test`, `cargo test -p server --test playable_client_draft_ready_bridge_test`, `cargo check -p client`, `cargo check -p server`, and `git diff --check` pass.
- [ ] **Evidence document exists**: `production/qa/evidence/playable-client-draft-shop-hand-bridge.md` records commit, commands, two-client setup, live message path, purchase/ready/refresh observations, defects, and friend-game-only scope statement.

---

## Likely Files Touched

- `client/src/ui/shop_auction/mod.rs`
- `client/src/ui/hand/mod.rs`
- `client/src/ui/hud/mod.rs`
- `client/src/presentation/mod.rs`
- `client/src/presentation/shared/economy_view.rs`
- `client/src/state/mod.rs`
- `client/src/network/mod.rs`
- `client/Cargo.toml`
- `server/src/network/mod.rs`
- `server/src/network/economy_dispatch.rs`
- `server/src/core/rsm/events.rs`
- `server/src/core/rsm/system.rs`
- `server/src/core/rsm/plugin.rs`
- `server/src/feature/acquisition/system.rs`
- `server/src/feature/acquisition/messages.rs`
- `server/src/feature/acquisition/hands.rs`
- `server/src/core/session/reconnect.rs`
- `server/Cargo.toml`
- `tests/integration/playable_client/draft_shop_hand_bridge_test.rs`
- `tests/integration/playable_client/draft_ready_bridge_test.rs`
- `production/qa/evidence/playable-client-draft-shop-hand-bridge.md`

`shared/src/protocol.rs` should not need changes because the required draft, shop, hand, ready, and economy messages already exist. Any protocol change must preserve existing reliable-channel assignments and be justified in evidence.

## Implementation Notes

- Start from the live message gaps, not from UI redesign. The key risk is that visible controls exist but some C2S messages do not reach authoritative server systems.
- `C2SSignalReady` currently appears in the server network receive surface. This story must connect it to the real draft-ready server path rather than merely logging receipt.
- Purchase and refresh UI may show pending/disabled visual states required by existing GDDs, but ownership, hand contents, economy, ready completion, and phase remain S2C-driven.
- If a DRAFT_AUCTION phase appears before DRAFT_SHOP, preserve current Shop/Auction UI locking/buffering semantics and do not expand settlement polish beyond what is needed to keep the path moving.
- Do not introduce a direct client dependency on server acquisition, economy, or RSM modules.

## Performance Budget

The live bridge is fixed-size over visible draft grid slots, shop slots, hand slots, and pending input controls. Client S2C processing plus view update must remain at or below 2 ms per frame. Presentation steady-state remains below 1 ms and phase-boundary spikes remain below 3 ms. Server-side ready and purchase dispatch must remain inside the server steady-state budget of 5 ms per tick.

---

## QA Test Cases

- **Live DRAFT_INITIAL purchase**
  - Given: Two real clients enter DRAFT_INITIAL through PLAYABLE-001.
  - When: A client buys a card from the 9-card offering.
  - Then: outbound `C2SPurchaseCard` is observed, server confirms acquisition, hand updates, and gold updates from S2C.

- **Ready bridge**
  - Given: Both clients are in the same active draft phase.
  - When: Each client clicks Ready.
  - Then: the server receives both `C2SSignalReady` messages through the authoritative ready path and phase progression is server-driven.

- **Live DRAFT_SHOP purchase and refresh**
  - Given: Server enters DRAFT_SHOP and sends shop slots.
  - When: A client purchases a shop card and then refreshes.
  - Then: purchase, acquisition, gold, and refreshed slots are visible only after authoritative S2C messages.

- **Snapshot rebuild**
  - Given: A client receives a snapshot during DRAFT_INITIAL or DRAFT_SHOP.
  - When: snapshot state is applied.
  - Then: hand, shop, economy, phase, and board-facing presentation resources align before live messages are processed.

---

## Test Evidence

**Story Type**: Integration

**Required automated test targets**:

- `tests/integration/playable_client/draft_shop_hand_bridge_test.rs`
  - Registered as `playable_client_draft_shop_hand_bridge_test`
  - Command: `cargo test -p client --test playable_client_draft_shop_hand_bridge_test`
- `tests/integration/playable_client/draft_ready_bridge_test.rs`
  - Registered as `playable_client_draft_ready_bridge_test`
  - Command: `cargo test -p server --test playable_client_draft_ready_bridge_test`

**Required regression commands**:

- `cargo test -p client --test shop_auction_ui_draft_initial_grid_test`
- `cargo test -p client --test shop_auction_ui_shop_panel_test`
- `cargo test -p client --test hand_ui_draft_initial_grid_test`
- `cargo test -p client --test shared_economy_view_test`
- `cargo test -p server --test card_acquisition_draft_initial_test`
- `cargo test -p server --test card_acquisition_purchase_atomicity_test`
- `cargo test -p server --test economy_network_dispatch_test`
- `cargo check -p client`
- `cargo check -p server`
- `git diff --check`

**Required evidence document**:

- `production/qa/evidence/playable-client-draft-shop-hand-bridge.md`

**Final evidence expectations**:

- Exact commit and build target.
- Commands used to run server and the relevant client test or two-client local run.
- Message trace covering `S2CDraftOffering`, `C2SPurchaseCard`, `S2CCardAcquired`, `S2CGoldUpdate`, `C2SSignalReady`, `S2CPhaseChanged`, `S2CShopSlots`, and `C2SRefreshShop` where exercised.
- Screenshot or capture summary of DRAFT_INITIAL grid, updated hand, updated economy, DRAFT_SHOP slots, and ready state.
- Explicit statement that evidence is friend-game draft/shop/hand bridge evidence only, not public release readiness, not playtest validation, not broad accessibility completion, and not full playable-client manual QA.

**Status**: [ ] Not yet implemented or captured.

---

## Dependencies

- Depends on: [PLAYABLE-001 Primary Client Bootstrap + Fresh Lobby Entry](story-001-primary-client-bootstrap-fresh-lobby-entry.md) - Ready; must complete before implementation starts.
- Depends on: existing Card Data Pool, Card Acquisition, Economy, Presentation, Hand UI, HUD, Shop/Auction UI, and RSM story implementations on `main`.
- Unlocks: [PLAYABLE-003 Real End-to-End Loop Verification](story-003-real-end-to-end-loop-verification.md).

## Blockers

None.
