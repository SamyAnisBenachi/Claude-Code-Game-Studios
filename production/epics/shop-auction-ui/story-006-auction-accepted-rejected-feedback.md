# Story 006: Auction Accepted/Rejected Feedback

> **Epic**: Shop / Auction UI
> **Status**: Ready
> **Layer**: Presentation
> **Type**: Integration
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/shop-auction-ui.md`
**Requirement**: `TR-SAU-001`, `TR-SAU-005`
**ADR Governing Implementation**: [ADR-013: Auction System State](../../../docs/architecture/adr-013-auction-system-state.md), [ADR-019: Economy Resource Architecture](../../../docs/architecture/adr-019-economy-resource-architecture.md)

This story processes `S2CAuctionBidAccepted`, `S2CAuctionBidRejected`, and the associated `S2CGoldBroadcast` gate that prevents button re-enable against stale free-gold data. It resolves in-flight bid state and keeps accepted/rejected UI synchronized with server authority.

## Acceptance Criteria

- [ ] Accepted local bid updates current price, sets local player as leader, hides buttons, and shows "YOU ARE LEADING".
- [ ] Accepted opponent bid updates current price and leader but does not re-enable buttons until local-player `S2CGoldBroadcast` arrives.
- [ ] If local-player `S2CGoldBroadcast` arrives before opponent accepted bid, the two-message gate still re-enables immediately once both are present.
- [ ] Opponent `S2CGoldBroadcast` does not satisfy the local re-enable gate.
- [ ] Accepted bid writes timer target fill from `new_timer_ms / auction_timer_ms`.
- [ ] Rejected bid clears `BIDDING...`, re-evaluates affordability, and shows the mapped toast text.
- [ ] Rejection reasons map to the exact GDD toast messages.
- [ ] Settlement clears pending accepted/gold gate flags unconditionally.

## Implementation Notes

- Store pending gate flags/resources explicitly so tests can assert both arrival orders.
- Do not compute affordability from stale local gold after an opponent accepted bid.
- Rejected messages arriving after settlement must not re-enable bid buttons; Story 007 owns terminal settlement state, but this story should prepare for the terminal guard.
- Timer easing is a Card Animations concern; this story writes a test-observable target.

## Out of Scope

- Initial bid click/in-flight send (Story 005).
- Settlement overlay and panel transition (Story 007).
- Visual polish of toast animations (Story 009).

## QA Test Cases

- **Opponent accepted, gold after**
  - Given: opponent accepted bid amount 7 and local gold broadcast has not arrived
  - When: accepted handler runs
  - Then: buttons remain disabled and pending gate flag is set.

- **Gold before accepted**
  - Given: local gold broadcast arrives before opponent accepted bid
  - When: accepted bid later arrives
  - Then: both gate flags are satisfied and buttons re-enable per affordability.

- **Rejected mapping**
  - Given: each rejection reason
  - When: rejection handler runs
  - Then: toast text matches the GDD table and in-flight label is cleared.

## Test Evidence

**Required evidence**:
- Integration: `tests/integration/shop_auction_ui/auction_feedback_gold_gate_test.rs`

**Status**: [ ] Not yet created

## Dependencies

- Depends on: [Story 005](story-005-auction-bid-buttons-affordability-and-inflight.md), Auction System bid accepted/rejected dispatch, Economy gold broadcast dispatch.
- Unlocks: Story 007.
