# Story 001: Plugin Scaffold, Panel Tree, and Formulas

> **Epic**: Shop / Auction UI
> **Status**: Ready
> **Layer**: Presentation
> **Type**: Logic
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/shop-auction-ui.md`
**Requirement**: `TR-SAU-001`, `TR-SAU-002`, `TR-SAU-006`
**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md), [ADR-019: Economy Resource Architecture](../../../docs/architecture/adr-019-economy-resource-architecture.md)

This story creates the `ShopAuctionUiPlugin` shell, panel state resources, stable root nodes, and pure UI formulas used by all later stories. It should not wire gameplay messages beyond registering resources and local Bevy messages needed by the UI.

Shared ADR-021 infrastructure (`PresentationPlugin`, `PresentationSet`, `phase_sink_system`, and the canonical `CurrentClientPhase` path) is owned by [Presentation Layer Story 001](../presentation-layer/story-001-presentation-plugin-set-and-phase-sink.md). Do not implement those shared surfaces here.

## Acceptance Criteria

- [ ] `ShopAuctionUiPlugin` registers without panic in a minimal client `App`.
- [ ] Root panel entities for draft offering, shop, auction, footer, toast, and settlement overlay are created or pre-pooled according to the chosen UI lifecycle.
- [ ] All panel roots are bevy_ui entities, not world-space sprites.
- [ ] Plugin reads phase through `Res<CurrentClientPhase>` only.
- [ ] This story relies on `PresentationPlugin`, `PresentationSet`, and `phase_sink_system` from Presentation Layer Story 001 rather than defining them locally.
- [ ] `local_free_gold(gold, reserved_gold)` returns `gold - reserved_gold` without underflow.
- [ ] Bid labels render total commitment: `current_price + {1, 3, 5}` with secondary increment text.
- [ ] Auction border color tier formula maps 0-3, 4-6, 7-9, and 10+ to the GDD tiers.

## Implementation Notes

- Use Bevy 0.18 `Node`, `Text`, `TextSpan`, `ImageNode`, and Required Components API. Do not use `NodeBundle`.
- Register the plugin fifth in `PresentationPlugin`.
- Do not drain `MessageReceiver<S2CPhaseChanged>` here.
- Keep formulas pure and unit-testable outside a live Lightyear session.
- Treat exact pixel layout as provisional until `design/ux/shop-auction-ui.md` exists.

## Out of Scope

- `PresentationPlugin`, `PresentationSet`, `phase_sink_system`, and shared `CurrentClientPhase` path ownership (Presentation Layer Story 001).
- Draft offering purchase flow (Story 002).
- Shop slot refresh/purchase flow (Story 003).
- Auction message handling (Stories 004-007).

## QA Test Cases

- **Free gold**
  - Given: `gold = 10`, `reserved_gold = 3`
  - When: free gold formula runs
  - Then: result is 7.

- **Bid labels**
  - Given: `current_price = 7`
  - When: bid button labels are computed
  - Then: labels are `8g (+1)`, `10g (+3)`, and `12g (+5)`.

- **Border tiers**
  - Given: current prices 3, 4, 7, and 10
  - When: tier formula runs
  - Then: tiers are PaleInkBlue, AuctionAmber, DeepAmber, and CrimsonAmber.

## Test Evidence

**Required evidence**:
- Logic: `tests/unit/shop_auction_ui/plugin_scaffold_formulas_test.rs`

**Status**: [ ] Not yet created

## Dependencies

- Depends on: `production/epics/presentation-layer/story-001-presentation-plugin-set-and-phase-sink.md` - shared `PresentationPlugin`, `PresentationSet`, `phase_sink_system`, and `CurrentClientPhase` path must be complete before ShopAuctionUiPlugin is implemented or registered.
- Unlocks: Stories 002, 003, 004, 005, 006, 007, 008, 009.
