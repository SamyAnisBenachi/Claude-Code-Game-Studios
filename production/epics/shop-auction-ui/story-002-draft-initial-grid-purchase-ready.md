# Story 002: Draft Initial Grid Purchase Ready

> **Epic**: Shop / Auction UI
> **Status**: Ready
> **Layer**: Presentation
> **Type**: UI
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/shop-auction-ui.md`
**Requirement**: `TR-SAU-006`
**ADR Governing Implementation**: [ADR-015: Card Acquisition Shop State](../../../docs/architecture/adr-015-card-acquisition-shop-state.md), [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)

DRAFT_INITIAL activates only after both `S2CPhaseChanged(DRAFT_INITIAL)` and `S2CDraftOffering` are available. The UI shows the sorted 3 x 3 offering, supports purchase clicks, marks confirmed purchases, handles hand-full/insufficient-gold lockouts, and exposes Ready/Retract Ready without disabling further purchases.

## Acceptance Criteria

- [ ] Panel remains blank until both phase and draft offering data have arrived.
- [ ] The 9-card grid is sorted by rarity descending, then cost descending.
- [ ] Valid slot click sends exactly one `C2SPurchaseCard { card_id }`.
- [ ] Insufficient gold does not send C2S and flashes the gold counter request.
- [ ] Hand size 10 locks unowned slots and shows the hand-full banner.
- [ ] `S2CCardAcquired` plus `S2CGoldUpdate` marks the slot as purchased and shows "BOUGHT" overlay.
- [ ] Ready sends `C2SSignalReady { retract: false }`; Retract Ready sends `C2SSignalReady { retract: true }`.
- [ ] Grid remains interactive after Ready until phase changes to PLACEMENT.
- [ ] `S2CPhaseChanged(PLACEMENT)` dismisses the panel and blocks further purchase sends.

## Implementation Notes

- `S2CDraftOffering` and `S2CPhaseChanged` may arrive in either order.
- Do not optimistically mark a purchase as bought before S2C confirmation.
- First-session tooltip placement and persistence are constrained by the GDD but exact UX is deferred to `design/ux/shop-auction-ui.md`; implement only once the UX gate is cleared or explicitly scoped.
- Use `C2SSignalReady` for both Ready and Retract Ready.

## Out of Scope

- DRAFT_SHOP panel refresh/purchase flow (Story 003).
- Auction panel state (Stories 004-007).
- Final visual/accessibility evidence (Story 009).

## QA Test Cases

- **Activation buffer**
  - Given: `S2CDraftOffering` arrives before `DRAFT_INITIAL` phase
  - When: phase later arrives
  - Then: panel activates once with the buffered offering.

- **Purchase send**
  - Given: local gold is sufficient and hand size is below 10
  - When: a slot is clicked
  - Then: one `C2SPurchaseCard` is queued and the slot enters pending state.

- **Ready remains interactive**
  - Given: Ready has been clicked
  - When: player clicks another affordable slot
  - Then: purchase still sends.

## Test Evidence

**Required evidence**:
- UI/integration: `tests/integration/shop_auction_ui/draft_initial_grid_test.rs`
- Visual evidence for overlay/tooltip later: `production/qa/evidence/shop-auction-ui-draft-initial-evidence.md`

**Status**: [ ] Not yet created

## Dependencies

- Depends on: [Story 001](story-001-plugin-scaffold-panel-tree-and-formulas.md), Card Acquisition draft offering dispatch.
- Unlocks: DRAFT_INITIAL M2 playable path.
