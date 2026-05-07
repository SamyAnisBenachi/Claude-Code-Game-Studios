# Story 008: Reconnect Snapshot and Late Message Recovery

> **Epic**: Shop / Auction UI
> **Status**: Ready
> **Layer**: Presentation
> **Type**: Integration
> **Manifest Version**: 2026-05-05

## Context

**GDD**: `design/gdd/shop-auction-ui.md`
**UX Spec**: `design/ux/shop-auction-ui.md`
**Requirement**: `TR-SAU-003`, `TR-SAU-006`
**ADR Governing Implementation**: [ADR-002: Client-Server Authority](../../../docs/architecture/adr-002-client-server-authority.md), [ADR-008: Lightyear Channel Configuration](../../../docs/architecture/adr-008-lightyear-channel-config.md), [ADR-011: Reconnect and Snapshot](../../../docs/architecture/adr-011-reconnect-snapshot.md), [ADR-013: Auction System State](../../../docs/architecture/adr-013-auction-system-state.md), [ADR-015: Card Acquisition Shop State](../../../docs/architecture/adr-015-card-acquisition-shop-state.md), [ADR-019: Economy Resource Architecture](../../../docs/architecture/adr-019-economy-resource-architecture.md), [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)
**Control Manifest**: `docs/architecture/control-manifest.md` version `2026-05-05`.
**Readiness status**: Story content refreshed for the current manifest before `/dev-story`. SAU-004, SAU-005, SAU-006, and SAU-007 are Complete, so this story is ready for DRAFT_AUCTION/DRAFT_SHOP reconnect and late-message recovery implementation.

Shop/Auction UI must rebuild its auction/shop presentation from `S2CGameSnapshot` before applying live incremental messages on reconnect. After reconnect or phase exit, stale accepted/rejected, purchase, refresh, and settlement-adjacent messages must not revive an inactive panel, re-enable bid controls, or mutate client-side authoritative state.

## Current Snapshot Contract

Current `origin/main` supports the following `S2CGameSnapshot` fields relevant to SAU-008:

- `S2CGameSnapshot.phase`, `round_number`, and `timer_remaining_ms` identify the authoritative reconnect phase and timer state.
- `S2CGameSnapshot.auction_state: Option<AuctionSnapshot>` carries DRAFT_AUCTION reconnect data: `card_id`, `starting_price`, `last_accepted_bid`, `current_leader`, and `timer_remaining_ms`.
- `PlayerSnapshot.shop_slots: Vec<Option<CardId>>` carries the reconnecting player's private DRAFT_SHOP slots.
- `PlayerSnapshot.gold`, `reserved_gold`, `current_mana`, `reserve_mana`, `mana_cap`, `hand`, and `submitted` carry local presentation inputs. Free gold is still `gold - reserved_gold` with saturating subtraction.
- `S2CGameSnapshot` does not carry `S2CDraftOffering` or an `InitialDraftOffering` equivalent. DRAFT_INITIAL grid restoration from snapshot is therefore not supported by the current protocol/server snapshot contract.

## DRAFT_INITIAL Scope Decision

DRAFT_INITIAL reconnect restoration is out of scope for SAU-008. If a reconnect snapshot reports `phase = DraftInitial`, this story must not fabricate a 3x3 offering, card ownership, or initial-purchase state from shop slots or hand data. The UI may enter the existing non-interactive awaiting-offering state until a live `S2CDraftOffering` arrives through the existing single-drain fanout path.

Full DRAFT_INITIAL reconnect restoration requires a separate protocol/server snapshot dependency that adds the active initial draft offering and purchase/ready state to `S2CGameSnapshot` or defines another snapshot-first reconnect payload. That dependency is not implemented by this story.

## Control Manifest Rules

- Required: `S2CGameSnapshot` is snapshot-first on reconnect. Shop/Auction UI processes the shared presentation snapshot message before live accepted/rejected, settlement, shop-slot, or card-acquired effects can update visible state.
- Required: Shop/Auction UI reads phase from `Res<CurrentClientPhase>` and timer context from the shared presentation phase view/snapshot data. It must not drain `MessageReceiver<S2CPhaseChanged>` directly.
- Required: Own economy and affordability read from `Res<PlayerEconomyView>` or snapshot-seeded local Shop/Auction view state derived from the local `PlayerSnapshot`. Do not mutate economy from local input.
- Required: `PlayerEconomyView` remains authoritative for the local player's own gold/current mana/reserve mana/mana cap projection, seeded only from `S2CGoldUpdate` or the local reconnect snapshot.
- Required: Keep Lightyear S2C handling single-drain per message type. If a shared bridge already drains `S2CGameSnapshot`, `S2CShopSlots`, `S2CDraftOffering`, `S2CCardAcquired`, `S2CGoldUpdate`, `S2CGoldBroadcast`, or `S2CPhaseChanged`, consume the bridge/message resource instead of registering another production receiver.
- Required: Snapshot rebuild clears transient/pending state: bid in-flight, accepted/gold re-enable gates, locally expired auction wait state, stale settlement animation requests, pending shop purchases, pending refresh, and any buffered late accepted/rejected effects unless the snapshot contains equivalent active auction state.
- Forbidden: No optimistic client state. Local input may show transient pending controls, but price, leader, gold, reservation, shop slots, card ownership, phase, and ready/submitted state come only from authoritative S2C or snapshot data.
- Forbidden: Do not send `C2SRequestSnapshot` or other C2S messages from this UI story as a workaround for missing DRAFT_INITIAL offering data.

## Acceptance Criteria

- [ ] **Snapshot-first rebuild ordering**: Given `PresentationGameSnapshotMessage` and live Shop/Auction messages are available in the same frame, when `PresentationSet::MessageDrain` and `StateSync` run, then snapshot-derived state is applied first and late live messages cannot resurrect a prior phase's panel state.
- [ ] **DRAFT_AUCTION snapshot rebuild**: Given `S2CGameSnapshot { phase = DraftAuction, auction_state = Some(AuctionSnapshot { card_id, starting_price, last_accepted_bid, current_leader, timer_remaining_ms }) }`, when Shop/Auction UI rebuilds, then the auction panel becomes active from the snapshot without requiring `S2CAuctionCard`.
- [ ] **Auction price/leader/timer restoration**: Given the auction snapshot has `last_accepted_bid > 0`, `current_leader = Some(player)`, and `timer_remaining_ms = X`, when rebuild completes, then current price, leader display, bid affordability, locally-expired state, and timer target use the snapshot values.
- [ ] **Auction no-bid restoration**: Given the auction snapshot has `last_accepted_bid = 0` and `current_leader = None`, when rebuild completes, then the auction price uses `starting_price`, no player is shown as leader, and bid buttons evaluate from current free gold.
- [ ] **DRAFT_SHOP snapshot rebuild**: Given `S2CGameSnapshot { phase = DraftShop }` whose local `PlayerSnapshot.shop_slots` contains three slot entries, when Shop/Auction UI rebuilds, then the shop panel/slot state is populated from those exact slot entries without waiting for a new `S2CShopSlots`.
- [ ] **Shop slot privacy**: Given the snapshot contains opponent `PlayerSnapshot.shop_slots = []` from server secret stripping, when Shop/Auction UI rebuilds, then the UI never attempts to infer or display opponent private shop slots.
- [ ] **Economy rebuild**: Given local `PlayerSnapshot.gold = G` and `reserved_gold = R`, when rebuild completes, then local free gold and affordability use `saturating_sub(G, R)` and are not computed from `reserve_mana`, local text, or client input.
- [ ] **Shared economy authority preserved**: Given a local purchase, refresh, bid, ready, or reconnect-state click occurs without an authoritative S2C/snapshot update, when the UI updates, then `PlayerEconomyView` and authoritative price/slot/card ownership values remain unchanged.
- [ ] **Late accepted ignored after settlement or phase exit**: Given settlement has completed or `CurrentClientPhase.phase != DraftAuction`, when a late `S2CAuctionBidAccepted` arrives, then current price, leader, bid buttons, timer target, and panel mode do not change.
- [ ] **Late rejected ignored after settlement or phase exit**: Given settlement has completed or `CurrentClientPhase.phase != DraftAuction`, when a late `S2CAuctionBidRejected` arrives, then no rejected toast is shown and bid controls are not re-enabled.
- [ ] **Late shop purchase/refresh confirmations ignored after phase exit**: Given `CurrentClientPhase.phase != DraftShop`, when late shop `S2CCardAcquired`, `S2CShopSlots`, or refresh-related confirmation state arrives, then the DRAFT_SHOP panel remains inactive and no stale pending/slot state is restored.
- [ ] **DRAFT_INITIAL non-restoration guard**: Given `S2CGameSnapshot { phase = DraftInitial }` without an initial draft offering payload, when Shop/Auction UI rebuilds, then it does not populate a 3x3 grid from `PlayerSnapshot.shop_slots`, does not claim DRAFT_INITIAL restoration, and remains non-interactive until a live `S2CDraftOffering` arrives.
- [ ] **Single-drain guard evidence**: Production client source still has exactly one owner for `MessageReceiver<S2CPhaseChanged>`, exactly one owner for `MessageReceiver<S2CGoldUpdate>`, and no duplicate Shop/Auction UI drain for snapshot or draft/shop fanout messages already owned by shared presentation code.

## Implementation Notes

- Implement snapshot handling as a full reset of Shop/Auction UI transient state before phase-specific rebuild. Snapshot-derived state wins over queued/incremental messages for the rebuild frame.
- Reuse existing shared presentation fanout resources/messages for `S2CGameSnapshot`, `S2CShopSlots`, `S2CDraftOffering`, `S2CCardAcquired`, and economy updates. Do not register duplicate Lightyear receivers to make tests easier.
- For DRAFT_AUCTION snapshots, rebuild from `AuctionSnapshot` directly. `S2CAuctionCard` is not replayed on reconnect and must not be required for auction panel activation.
- For DRAFT_SHOP snapshots, derive local slot state only from the local player snapshot entry. Preserve server secret stripping by ignoring opponent private vectors.
- The rebuild must clear settlement and bid feedback state even if the snapshot remains in DRAFT_AUCTION. Reintroduce only the active auction card, price, leader, timer, and affordability state represented by `AuctionSnapshot` plus current economy data.
- Late-message guards should be phase- and terminal-state based, not arrival-order assumptions. Reliable ordering on a new reconnect connection does not replay the old connection's queue.
- If the current code already seeds Shop/Auction local gold from snapshot in addition to `PlayerEconomyView`, keep the two paths consistent and test-observable. Do not introduce a local economy authority that can diverge from shared presentation state.
- Keep DRAFT_INITIAL behavior explicit in tests: the correct SAU-008 result is "not restored from snapshot", not a partial or inferred grid.

## Performance Budget

- Snapshot rebuild must fit inside ADR-021 presentation budgets: steady-state under 1 ms per frame and reconnect/phase-boundary spike under 3 ms.
- Rebuild work is fixed-size for Shop/Auction UI: one panel mode, one auction state resource, three shop slots, local hand/economy summaries, and a small set of transient flags. Do not scan card catalogs, all player pools, all entities, or all historical messages.
- Late-message guards must be O(1) over current phase/terminal flags and message payloads.
- Do not spawn/despawn steady-state panel entities during reconnect. Reuse pre-pooled UI entities and update `Visibility`, text, and resource state.
- Do not add per-frame allocations, persistent debug overlays, duplicate message queues, or protocol-level polling to compensate for missing DRAFT_INITIAL snapshot data.

## Out of Scope

- Server snapshot assembly or protocol schema changes.
- Adding DRAFT_INITIAL offering data to `S2CGameSnapshot`.
- Full DRAFT_INITIAL reconnect restoration.
- Board Rendering snapshot recovery.
- HUD reconnect snapshot recovery.
- Hand UI staging/placement reconnect behavior.
- Public release readiness, broad accessibility completion, playtest validation, full playable-client QA, or full game completion claims.

## QA Test Cases

- **Auction snapshot rebuild**
  - Given: a snapshot contains `phase = DraftAuction` plus `AuctionSnapshot` with card, price, leader, and timer data
  - When: Shop/Auction UI processes the snapshot
  - Then: auction panel is active with those values and no `S2CAuctionCard` is required.

- **Shop snapshot rebuild**
  - Given: a snapshot contains `phase = DraftShop` and the local player's three `shop_slots`
  - When: Shop/Auction UI processes the snapshot
  - Then: DRAFT_SHOP slot UI reflects those exact local slots.

- **Economy/free-gold rebuild**
  - Given: local snapshot economy has `gold = 8` and `reserved_gold = 5`
  - When: affordability is evaluated after rebuild
  - Then: local free gold is `3`, and `reserve_mana` does not affect the result.

- **Late accepted/rejected ignored**
  - Given: auction settlement has completed or phase has exited DRAFT_AUCTION
  - When: late accepted and rejected messages arrive
  - Then: price, leader, bid buttons, timer, and rejection toast remain unchanged.

- **Late shop confirmation ignored**
  - Given: phase is PLACEMENT and DRAFT_SHOP has an old in-flight purchase or refresh
  - When: a late shop confirmation arrives
  - Then: DRAFT_SHOP remains inactive and stale pending state is not rendered.

- **DRAFT_INITIAL unsupported snapshot**
  - Given: a snapshot reports `phase = DraftInitial` but carries no offering payload
  - When: Shop/Auction UI processes the snapshot
  - Then: no 3x3 grid is restored from snapshot and no DRAFT_INITIAL purchase sends are possible until a live `S2CDraftOffering` arrives.

## Test Evidence

**Required evidence**:
- Integration: `tests/integration/shop_auction_ui/reconnect_late_message_test.rs`
- Grep guard evidence in the implementation notes or completion notes for single-drain ownership:
  - `rg -n "MessageReceiver<S2CPhaseChanged>" client/src`
  - `rg -n "MessageReceiver<S2CGoldUpdate>" client/src`
  - targeted grep for `MessageReceiver<S2CGameSnapshot>` and Shop/Auction duplicate receiver additions

**Status**: [ ] Not yet created

## Dependencies

- Depends on: [Story 001](story-001-plugin-scaffold-panel-tree-and-formulas.md) - Complete; provides Shop/Auction panel roots, formula resources, and plugin registration.
- Depends on: [Story 003](story-003-shop-panel-slots-refresh-purchase-ready.md) - Complete; provides DRAFT_SHOP slot, refresh, purchase, ready, and late confirmation behavior.
- Depends on: [Story 004](story-004-auction-panel-activation-and-preparing-state.md) - Complete; provides auction panel activation, preparing state, and footer locking.
- Depends on: [Story 005](story-005-auction-bid-buttons-affordability-and-inflight.md) - Complete; provides bid buttons, free-gold affordability, local in-flight state, and leader replacement.
- Depends on: [Story 006](story-006-auction-accepted-rejected-feedback.md) - Complete; provides accepted/rejected response handling and gate flags that reconnect must clear.
- Depends on: [Story 007](story-007-auction-settlement-and-shop-transition.md) - Complete; provides terminal settlement state and stale late-message suppression baseline.
- Depends on: [Presentation Layer Story 001](../presentation-layer/story-001-presentation-plugin-set-and-phase-sink.md) - Complete; provides `PresentationPlugin`, `PresentationSet`, `phase_sink_system`, and `CurrentClientPhase`.
- Depends on: [Presentation Layer Story 002](../presentation-layer/story-002-shared-economy-view.md) - Complete; provides `PlayerEconomyView` and the single `S2CGoldUpdate`/snapshot economy authority.
- Depends on: Game Session System reconnect snapshot implementation and current `S2CGameSnapshot` protocol fields.
- Does not depend on: DRAFT_INITIAL reconnect offering restoration. That is a future protocol/server snapshot dependency.
- Unlocks: reconnect QA for the DRAFT_AUCTION and DRAFT_SHOP visual playable path.

## Blockers

- None for DRAFT_AUCTION and DRAFT_SHOP reconnect/late-message recovery.
- Blocked/dependent follow-up: full DRAFT_INITIAL reconnect restoration until protocol/server snapshot support carries the active initial draft offering and any required purchase/ready state.
