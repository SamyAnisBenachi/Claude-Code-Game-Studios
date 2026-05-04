# Story 005: Auction Bid Buttons, Affordability, and In-Flight

> **Epic**: Shop / Auction UI
> **Status**: Ready
> **Layer**: Presentation
> **Type**: UI
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/shop-auction-ui.md`
**Requirement**: `TR-SAU-001`, `TR-SAU-002`, `TR-SAU-005`
**ADR Governing Implementation**: [ADR-013: Auction System State](../../../docs/architecture/adr-013-auction-system-state.md), [ADR-019: Economy Resource Architecture](../../../docs/architecture/adr-019-economy-resource-architecture.md), [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)

This story implements the three preset bid buttons, proactive affordability lockouts, "YOU ARE LEADING" replacement state, and the in-flight "BIDDING..." visual after a click. It sends `C2SPlaceBid` intent but does not commit local price/gold until server response stories process S2C messages.

## Acceptance Criteria

- [ ] Buttons show total commitment as primary label and increment as secondary label.
- [ ] +1/+3/+5 buttons evaluate affordability from `local_free_gold = gold - reserved_gold`.
- [ ] Hand size 10 disables all bids and shows hand-full auction message.
- [ ] If local player is current leader, all three buttons are hidden and "YOU ARE LEADING" fills the area.
- [ ] Clicking an enabled button sends exactly one `C2SPlaceBid { amount }`.
- [ ] Clicked button enters `BIDDING...`; other buttons become generic disabled.
- [ ] No additional bid can be sent while a bid is in-flight.
- [ ] Locally expired timer disables all buttons and shows ending/awaiting-server state according to elapsed time.

## Implementation Notes

- Current price source is last accepted amount or starting price if no bids have been accepted.
- Do not add a free-form text bid field.
- Do not optimistically update current leader or current price on click.
- Use Card Animations for timer and button feedback if animation hooks already exist; otherwise expose message hooks for later animation story.

## Out of Scope

- Processing `S2CAuctionBidAccepted` or `S2CAuctionBidRejected` (Story 006).
- Settlement handling (Story 007).
- Full visual evidence and accessibility pass (Story 009).

## QA Test Cases

- **Affordability**
  - Given: `local_free_gold = 2`, `current_price = 0`
  - When: buttons evaluate
  - Then: +1 is enabled; +3 and +5 are disabled.

- **In-flight**
  - Given: +3 button is clicked
  - When: the click handler returns
  - Then: one `C2SPlaceBid` is sent, +3 reads `BIDDING...`, and +1/+5 are disabled.

- **Leading**
  - Given: local player is current leader
  - When: button area renders
  - Then: bid buttons are hidden and "YOU ARE LEADING" is visible.

## Test Evidence

**Required evidence**:
- UI/integration: `tests/integration/shop_auction_ui/auction_bid_buttons_test.rs`
- Visual evidence later: `production/qa/evidence/shop-auction-ui-bid-buttons-evidence.md`

**Status**: [ ] Not yet created

## Dependencies

- Depends on: [Story 004](story-004-auction-panel-activation-and-preparing-state.md), economy gold broadcast state.
- Unlocks: Story 006.
