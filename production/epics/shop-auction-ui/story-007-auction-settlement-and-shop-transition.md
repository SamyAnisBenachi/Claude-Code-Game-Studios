# Story 007: Auction Settlement and Shop Transition

> **Epic**: Shop / Auction UI
> **Status**: Ready
> **Layer**: Presentation
> **Type**: Visual/Feel
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/shop-auction-ui.md`
**Requirement**: `TR-SAU-003`, `TR-SAU-006`
**ADR Governing Implementation**: [ADR-013: Auction System State](../../../docs/architecture/adr-013-auction-system-state.md), [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)

This story handles terminal auction settlement, winner/no-bid presentation, card-to-hand feedback for local wins, stale late-message suppression, and the transition into the interactive DRAFT_SHOP panel.

## Acceptance Criteria

- [ ] `S2CAuctionSettled { winner: Some(local_player) }` enters settling state and emits/acquires local card feedback toward the hand area.
- [ ] `winner: Some(opponent)` enters settling state without moving a card toward the local hand.
- [ ] `winner: None` enters settling state with no gold/card movement.
- [ ] Settlement clears in-flight bid state, pending gate flags, and queued late accepted/rejected effects.
- [ ] Accepted or rejected messages after settlement do not update current price, leader, or bid button state.
- [ ] Auction panel slides/dismisses and shop panel expands according to the GDD transition timing.
- [ ] DRAFT_SHOP timer starts after the shop expansion animation completes.
- [ ] If `S2CPhaseChanged(PLACEMENT)` arrives during settlement animation, animation cancels immediately and phase entry is not delayed.

## Implementation Notes

- Settlement is terminal and takes precedence over in-transit accepted/rejected messages.
- Card movement animation should be requested through Card Animations; do not block phase transition on animation completion.
- Buffered `S2CShopSlots` from auction should be applied before shop expansion begins so the shop is populated when revealed.
- Exact panel layout depends on the UX spec; keep state tests separate from final visual evidence.

## Out of Scope

- Server settlement logic.
- Purchase/refresh interactions after shop is active (Story 003).
- Full accessibility evidence (Story 009).

## QA Test Cases

- **Local winner**
  - Given: local player wins settlement
  - When: settlement handler runs
  - Then: panel enters settling state and local card-acquired visual request is emitted.

- **Late rejection ignored**
  - Given: settlement has arrived while a bid was in-flight
  - When: a late `S2CAuctionBidRejected` arrives
  - Then: buttons remain disabled/dismissed and no stale toast re-enables bidding.

- **Phase interrupt**
  - Given: settlement animation is running
  - When: `S2CPhaseChanged(PLACEMENT)` arrives
  - Then: animation cancels and panel becomes inactive immediately.

## Test Evidence

**Required evidence**:
- Visual/Feel: `production/qa/evidence/shop-auction-ui-settlement-transition-evidence.md`
- Integration support: `tests/integration/shop_auction_ui/auction_settlement_test.rs`

**Status**: [ ] Not yet created

## Dependencies

- Depends on: [Story 006](story-006-auction-accepted-rejected-feedback.md), Auction System settlement dispatch, Card Acquisition card-acquired dispatch.
- Unlocks: Story 008 and post-auction shop visual path.
