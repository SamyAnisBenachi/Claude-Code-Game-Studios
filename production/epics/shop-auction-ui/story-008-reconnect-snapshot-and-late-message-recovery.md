# Story 008: Reconnect Snapshot and Late Message Recovery

> **Epic**: Shop / Auction UI
> **Status**: Ready
> **Layer**: Presentation
> **Type**: Integration
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/shop-auction-ui.md`
**Requirement**: `TR-SAU-003`, `TR-SAU-006`
**ADR Governing Implementation**: [ADR-011: Reconnect and Snapshot](../../../docs/architecture/adr-011-reconnect-snapshot.md), [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)

Shop/Auction UI must rebuild cleanly from `S2CGameSnapshot` and ignore or reconcile late messages that arrive after the snapshot or after a terminal phase change. This protects the visual playable path during reconnects and high-latency auction endings.

## Acceptance Criteria

- [ ] Snapshot rebuild restores the correct panel for current phase: draft initial, auction, shop, placement/inactive, or game over.
- [ ] Snapshot auction state with active card/leader/price/timer rebuilds auction panel without requiring `S2CAuctionCard`.
- [ ] Snapshot shop slots rebuild shop panel/auction footer content.
- [ ] Snapshot economy values rebuild local free gold and affordability.
- [ ] Late `S2CAuctionBidAccepted` after settlement is ignored.
- [ ] Late `S2CAuctionBidRejected` after settlement is ignored and does not re-enable buttons.
- [ ] Late purchase/refresh confirmations after phase transition do not resurrect inactive panels.
- [ ] Reconnect rebuild clears in-flight and pending gate flags unless snapshot explicitly contains equivalent active auction state.

## Implementation Notes

- Snapshot should win over incremental messages in the rebuild frame.
- Keep snapshot rebuild idempotent.
- Do not assume reliable replay across reconnect; ADR-011 requires explicit snapshot data.
- This story may need to coordinate with Game Session snapshot field names at story-readiness.

## Out of Scope

- Server snapshot assembly.
- Board Rendering snapshot recovery.
- UX evidence for reconnect visual state.

## QA Test Cases

- **Auction snapshot**
  - Given: snapshot contains active auction state with price, leader, and timer
  - When: UI rebuild runs
  - Then: auction panel is active with those values and no card message is required.

- **Late accepted ignored**
  - Given: settlement has already been processed
  - When: late accepted bid arrives
  - Then: current price and leader do not change.

- **Inactive panel guard**
  - Given: phase is PLACEMENT
  - When: late shop confirmation arrives
  - Then: shop panel remains inactive.

## Test Evidence

**Required evidence**:
- Integration: `tests/integration/shop_auction_ui/reconnect_late_message_test.rs`

**Status**: [ ] Not yet created

## Dependencies

- Depends on: Stories 002, 003, 004, 005, 006, 007 and Game Session snapshot field availability.
- Unlocks: reconnect QA for M2 visual playable path.
