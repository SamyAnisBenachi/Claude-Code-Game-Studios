# Story 003: Shop Panel Slots Refresh Purchase Ready

> **Epic**: Shop / Auction UI
> **Status**: Ready
> **Layer**: Presentation
> **Type**: UI
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/shop-auction-ui.md`
**Requirement**: `TR-SAU-004`, `TR-SAU-006`
**ADR Governing Implementation**: [ADR-015: Card Acquisition Shop State](../../../docs/architecture/adr-015-card-acquisition-shop-state.md), [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)

The DRAFT_SHOP panel displays three server-supplied shop slots, supports purchases, handles manual refresh with escalating cost labels, and provides Ready/Retract Ready. It also owns the post-auction shop state after settlement and transition.

## Acceptance Criteria

- [ ] Non-auction DRAFT_SHOP waits for `S2CShopSlots` before becoming interactive.
- [ ] Three shop slots render card data or empty/dead-slot state from server data.
- [ ] Purchase click sends `C2SPurchaseCard` only for valid, affordable, non-empty slots.
- [ ] Rapid clicks on two valid slots can send two purchase messages while tracking each clicked slot independently.
- [ ] Refresh click disables the refresh button in the same frame and sends exactly one `C2SRefreshShop`.
- [ ] Refresh count increments only when `S2CShopSlots` confirms the refresh.
- [ ] Refresh counter resets on next DRAFT_SHOP entry.
- [ ] Hand size 10 locks shop slots but does not lock Refresh if affordable.
- [ ] Ready/Retract Ready uses `C2SSignalReady` and does not disable purchases before phase transition.
- [ ] `S2CPhaseChanged(PLACEMENT)` dismisses the panel and blocks late purchase/refresh sends.

## Implementation Notes

- No optimistic gold or card ownership changes. Wait for `S2CGoldUpdate`, `S2CShopSlots`, and `S2CCardAcquired`.
- Refresh cost label follows the GDD escalation and must update only from confirmed refresh state.
- If `S2CShopSlots` arrives during DRAFT_AUCTION, buffer it for the post-settlement shop transition rather than mutating visible locked footer slots mid-auction.

## Out of Scope

- Auction locked footer display while bidding (Story 004).
- Settlement panel-to-shop transition animation (Story 007).
- Final layout evidence (Story 009).

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

- Depends on: [Story 001](story-001-plugin-scaffold-panel-tree-and-formulas.md), Card Acquisition shop slot and refresh dispatch.
- Unlocks: post-auction M2 shop path and Story 007.
