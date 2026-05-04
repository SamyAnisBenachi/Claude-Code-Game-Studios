# Story 004: Auction Panel Activation and Preparing State

> **Epic**: Shop / Auction UI
> **Status**: Ready
> **Layer**: Presentation
> **Type**: Integration
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/shop-auction-ui.md`
**Requirement**: `TR-SAU-006`
**ADR Governing Implementation**: [ADR-013: Auction System State](../../../docs/architecture/adr-013-auction-system-state.md), [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)

Auction activation requires both `S2CAuctionCard` and `S2CPhaseChanged(DRAFT_AUCTION)`. The UI must handle either arrival order, a preparing state when the card arrives first, and normal dismissal if a non-auction phase arrives while preparing.

## Acceptance Criteria

- [ ] `S2CAuctionCard` before phase enters `AUCTION_PREPARING`, shows card/price, greys the timer bar, and does not start countdown.
- [ ] `S2CPhaseChanged(DRAFT_AUCTION)` before card does not activate auction UI by itself.
- [ ] When both card and phase are present, panel becomes active and countdown starts.
- [ ] `AUCTION_PREPARING` times out after 10 seconds without DRAFT_AUCTION phase and shows connection error state.
- [ ] Non-DRAFT_AUCTION phase during preparing clears the buffer and dismisses the panel.
- [ ] DRAFT_AUCTION shop footer is visible but locked; refresh button is hidden.
- [ ] Footer shop slots are read-only and do not send `C2SPurchaseCard` or `C2SRefreshShop`.

## Implementation Notes

- Do not rely on cross-type FIFO ordering between different reliable message types.
- The activation buffer is the primary path, not an edge-case guard.
- Timer starts only from `S2CPhaseChanged { timer_duration_ms }`.
- Shop footer content should use available shop slot data without allowing interaction.

## Out of Scope

- Bid button affordability and in-flight state (Story 005).
- Accepted/rejected response handling (Story 006).
- Settlement and transition (Story 007).

## QA Test Cases

- **Card first**
  - Given: `S2CAuctionCard` arrives with no auction phase
  - When: UI updates
  - Then: preparing state is visible and countdown is inactive.

- **Phase first**
  - Given: DRAFT_AUCTION phase arrives with no card
  - When: UI updates
  - Then: auction panel is not active.

- **Preparing dismiss**
  - Given: preparing state is active
  - When: `S2CPhaseChanged(GAME_OVER)` arrives
  - Then: auction panel becomes inactive and phase is processed normally.

## Test Evidence

**Required evidence**:
- Integration: `tests/integration/shop_auction_ui/auction_activation_test.rs`
- Visual evidence later: `production/qa/evidence/shop-auction-ui-auction-preparing-evidence.md`

**Status**: [ ] Not yet created

## Dependencies

- Depends on: [Story 001](story-001-plugin-scaffold-panel-tree-and-formulas.md), Auction System card dispatch, Card Acquisition shop slot data for footer.
- Unlocks: Stories 005, 006, 007.
