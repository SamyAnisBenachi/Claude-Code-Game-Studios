# Story 003: Shop Panel Slots Refresh Purchase Ready

> **Epic**: Shop / Auction UI
> **Status**: Ready
> **Layer**: Presentation
> **Type**: UI
> **Manifest Version**: 2026-05-05

## Context

**GDD**: `design/gdd/shop-auction-ui.md`
**Requirement**: `TR-SAU-006` plus the DRAFT_SHOP rules and acceptance criteria below.
**ADR Governing Implementation**: [ADR-015: Card Acquisition Shop State](../../../docs/architecture/adr-015-card-acquisition-shop-state.md), [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)

The DRAFT_SHOP panel displays three server-supplied shop slots, supports purchases, handles manual refresh with escalating cost labels, and provides Ready/Retract Ready. It also owns the visible post-auction shop panel after settlement, using already-buffered shop slots rather than creating any client-side pool state.

**GDD trace**:
- `design/gdd/shop-auction-ui.md` DRAFT_SHOP Rule 1: activation waits for `S2CPhaseChanged(DRAFT_SHOP)` plus `S2CShopSlots` on non-auction rounds; auction rounds use the slot data already populated from the DRAFT_AUCTION auto-refresh path.
- DRAFT_SHOP Rule 2: three horizontal slots render card art, name, rarity badge, cost, Refresh, and Ready.
- DRAFT_SHOP Rule 3 and Formula D.4: refresh label starts at `REFRESH · 1g`, becomes `REFRESH · 2g` after the first confirmed manual refresh, and remains capped at 2g with default config.
- DRAFT_SHOP Rule 4: purchase click pre-validates `gold >= cost` and `hand_size < 10`, sends `C2SPurchaseCard`, and waits for `S2CGoldUpdate` plus `S2CCardAcquired`.
- DRAFT_SHOP Rule 5: refresh click disables Refresh before sending `C2SRefreshShop`; `refresh_count_this_draft` increments only on `S2CShopSlots` receipt, never on send.
- DRAFT_SHOP Rule 6: hand size 10 locks all shop slots but leaves Refresh active when affordable.
- DRAFT_SHOP Rule 7: Ready/Retract Ready sends `C2SSignalReady { retract }`; the shop stays interactive while ready and freezes only on `S2CPhaseChanged(PLACEMENT)`.
- Edge cases: `S2CShopSlots` received during DRAFT_AUCTION is buffered until DRAFT_SHOP (`SAU-EG2`); late purchase confirmations after `S2CPhaseChanged(PLACEMENT)` restore pre-click slot state and leave gold unchanged (`SAU-DS7`).

**GDD acceptance IDs owned by this story**: `SAU-DS1`, `SAU-DS2`, `SAU-DS3`, `SAU-DS4`, `SAU-DS5`, `SAU-DS6`, `SAU-DS7`, `SAU-DS8`, `SAU-DS9`, `SAU-DS10`, `SAU-DS11`, `SAU-DS12`, and `SAU-EG2`.

**TR registry note**: `TR-SAU-006` is active and covers panel transitions/input gating. The current TR registry does not yet contain one TR per `SAU-DS*` DRAFT_SHOP criterion, so this story traces those criteria directly to `design/gdd/shop-auction-ui.md`.

## Acceptance Criteria

- [ ] `SAU-DS1` - Non-auction DRAFT_SHOP waits for both `S2CPhaseChanged(DRAFT_SHOP)` and `S2CShopSlots` before becoming interactive.
- [ ] DRAFT_SHOP renders exactly three server-supplied shop slots, including card data or empty/dead-slot state. UI holds no card-pool knowledge.
- [ ] `SAU-DS6` - Purchase click sends `C2SPurchaseCard` only for valid, affordable, non-empty slots, and rapid clicks on two valid slots send two purchase messages while tracking only each clicked slot as pending.
- [ ] `SAU-DS8` - Refresh click disables the refresh button in the same frame before any second click can fire, and sends exactly one `C2SRefreshShop`.
- [ ] `SAU-DS2`, `SAU-DS3`, `SAU-DS5` - Refresh count increments only when `S2CShopSlots` confirms the refresh, does not increment on timeout/failure, and resets to 0 on the next DRAFT_SHOP entry.
- [ ] `SAU-DS4` - Hand size 10 locks all shop slots but does not lock Refresh when `local_gold >= displayed_refresh_cost`.
- [ ] `SAU-DS9`, `SAU-DS10`, `SAU-DS11` - Ready/Retract Ready sends exactly one `C2SSignalReady { retract: false/true }` per click, updates button/status text, and does not disable purchases before phase transition.
- [ ] `SAU-DS7`, `SAU-DS12` - `S2CPhaseChanged(PLACEMENT)` dismisses the panel, blocks late purchase/refresh sends, and late purchase confirmation restores pre-click slot state with gold unchanged.
- [ ] `SAU-EG2` - If `S2CShopSlots` arrives during `AUCTION_ACTIVE`, the footer does not update mid-auction; the buffered slots are applied when DRAFT_SHOP becomes active post-transition.

## Implementation Notes

- Use Bevy 0.18 UI APIs with Required Components. Do not use deprecated `*Bundle` API (`NodeBundle`, `SpriteBundle`, `Camera2dBundle`, etc.).
- `S2CPhaseChanged` remains drained only by `phase_sink_system`; Shop/Auction UI reads `Res<CurrentClientPhase>` and must not add a second `MessageReceiver<S2CPhaseChanged>` drain.
- Use the shared economy view (`PlayerEconomyView`) for gold/mana display and affordability checks. Do not drain `S2CGoldUpdate` or `S2CGoldBroadcast` directly in this story.
- Respect Lightyear sender/receiver ownership: each `MessageReceiver<T>` has exactly one production drainer, and this story must not duplicate drains owned by phase, economy, card-acquisition, or pool-dispatch systems.
- No optimistic authoritative client state for purchase, refresh, or ready. Local click handling may show a pending/disabled control required by the GDD, but gold, card ownership, refresh count, phase, and ready completion remain driven by server messages.
- Refresh cost label follows Formula D.4 and must update only from confirmed refresh state: `refresh_count_this_draft` increments on `S2CShopSlots`, not on `C2SRefreshShop`.
- If `S2CShopSlots` arrives during DRAFT_AUCTION, buffer it for the post-settlement shop transition rather than mutating visible locked footer slots mid-auction.
- `S2CPhaseChanged(PLACEMENT)` wins over any in-flight purchase/refresh UI state. After phase change, do not apply late DRAFT_SHOP confirmations to the inactive panel.

## Performance Budget

The DRAFT_SHOP UI/message path must stay within ADR-021 Presentation budgets: steady-state processing under 1 ms per frame, and phase-boundary spikes under 3 ms when toggling panel entities, clearing pending slot states, applying buffered `S2CShopSlots`, or canceling tweens. Per-frame slot/affordability evaluation is fixed-size over three shop slots and must not scan hands, card pools, catalogs, or all player state.

## Out of Scope

- Auction locked footer display while bidding (Story 004).
- Settlement panel-to-shop transition animation (Story 007).
- Final layout evidence (Story 009).
- DRAFT_INITIAL first-session tooltip placement, dismissal, and persistence storage. Tooltip persistence remains outside SAU-003.
- Server-side purchase, refresh, shop draw, and network dispatch behavior. SAU-003 consumes their S2C results and sends C2S requests only.

## QA Test Cases

- **Refresh confirmation**
  - Given: refresh count is 0
  - When: refresh is clicked
  - Then: button disables and one C2S refresh sends; count remains 0 until `S2CShopSlots` arrives.

- **Hand full**
  - Given: hand size is 10 and local gold can afford refresh
  - When: shop state evaluates
  - Then: slots are locked and Refresh remains enabled.

- **Late confirmation ignored**
  - Given: purchase is in-flight and phase changes to PLACEMENT
  - When: late confirmation arrives
  - Then: shop panel remains inactive and no stale visual purchase state is applied.

## Test Evidence

**Required evidence**:
- UI/integration: `tests/integration/shop_auction_ui/shop_panel_test.rs`
- Visual evidence later: `production/qa/evidence/shop-auction-ui-shop-panel-evidence.md`

**Status**: [ ] Not yet created

## Dependencies

- Depends on: [Shop/Auction UI Story 001](story-001-plugin-scaffold-panel-tree-and-formulas.md) - Complete; provides panel tree, formula resources, and plugin registration scaffold.
- Depends on: [Card Acquisition Story 002](../card-acquisition/story-002-draft-initial.md) - Complete; establishes draft offering dispatch and reliable `S2CDraftOffering` unicast behavior that the UI activation model mirrors.
- Depends on: [Card Acquisition Story 003](../card-acquisition/story-003-draw-pipeline.md) - Complete; provides authoritative DRAFT_SHOP `S2CShopSlots` production for auto-refresh/shop entry.
- Depends on: [Card Acquisition Story 004](../card-acquisition/story-004-refresh-cost.md) - Complete; owns manual refresh cost/counter semantics consumed by the UI label and disable state.
- Depends on: [Card Acquisition Story 005](../card-acquisition/story-005-purchase-flow.md) - Complete; owns authoritative purchase, dead-slot, and late phase-race behavior.
- Depends on: [Card Data & Pool Story 006](../card-data-pool/story-006-network-dispatch-wiring.md) - Complete; dispatches `S2CShopSlots` and `S2CDraftOffering` over `ReliableChannel` to the owning player.
- Depends on: [RSM Story 006](../round-state-machine/story-006-network-dispatch-wiring.md) - Complete; dispatches `S2CPhaseChanged` over `ReliableChannel` for phase sink consumption.
- Depends on: [HUD Story 002](../hud/story-002-gold-mana-display.md) - Complete; provides the shared economy display/state pattern that Shop/Auction UI must reuse.
- Unlocks: post-auction M2 shop path and Story 007.
