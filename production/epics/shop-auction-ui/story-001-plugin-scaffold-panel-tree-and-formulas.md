# Story 001: Plugin Scaffold, Panel Tree, and Formulas

> **Epic**: Shop / Auction UI
> **Status**: Complete
> **Layer**: Presentation
> **Type**: Logic
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/shop-auction-ui.md`
**UX Spec**: `design/ux/shop-auction-ui.md` (root/lifecycle constraints only; exact pixel layout remains out of scope for this Logic story)
**Requirement**: `TR-SAU-001`, `TR-SAU-002`, `TR-SAU-006`
**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md), [ADR-019: Economy Resource Architecture](../../../docs/architecture/adr-019-economy-resource-architecture.md)

This story creates the `ShopAuctionUiPlugin` shell, panel state resources, stable root nodes, and pure UI formulas used by all later stories. It should not wire gameplay messages beyond registering resources and local Bevy messages needed by the UI.

Shared ADR-021 infrastructure (`PresentationPlugin`, `PresentationSet`, `phase_sink_system`, and the canonical `CurrentClientPhase` path) is owned by [Presentation Layer Story 001](../presentation-layer/story-001-presentation-plugin-set-and-phase-sink.md). Do not implement those shared surfaces here.

## Acceptance Criteria

- [x] `ShopAuctionUiPlugin` registers without panic in a minimal client `App`.
- [x] Root panel entities for draft offering, shop, auction, footer, toast, and settlement overlay are created or pre-pooled according to the chosen UI lifecycle.
- [x] All panel roots are bevy_ui entities, not world-space sprites.
- [x] Plugin reads phase through `Res<CurrentClientPhase>` only.
- [x] This story relies on `PresentationPlugin`, `PresentationSet`, and `phase_sink_system` from Presentation Layer Story 001 rather than defining them locally.
- [x] `local_free_gold(gold, reserved_gold)` returns `gold - reserved_gold` without underflow.
- [x] Bid labels render total commitment: `current_price + {1, 3, 5}` with secondary increment text.
- [x] Auction border color tier formula maps 0-3, 4-6, 7-9, and 10+ to the GDD tiers.

## Implementation Notes

- Use Bevy 0.18 `Node`, `Text`, `TextSpan`, `ImageNode`, and Required Components API. Do not use `NodeBundle`.
- Register the plugin fifth in `PresentationPlugin`.
- Do not drain `MessageReceiver<S2CPhaseChanged>` here.
- Keep formulas pure and unit-testable outside a live Lightyear session.
- Use `design/ux/shop-auction-ui.md` for root names, panel lifecycle, and layout constraints; exact pixel layout and visual/accessibility evidence remain out of scope for this Logic story.

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

**Status**: [x] Created and passing

## Dependencies

- Depends on: `production/epics/presentation-layer/story-001-presentation-plugin-set-and-phase-sink.md` - shared `PresentationPlugin`, `PresentationSet`, `phase_sink_system`, and `CurrentClientPhase` path must be complete before ShopAuctionUiPlugin is implemented or registered. Complete on `main` as of `d303155`.
- Sequencing note: launch after `production/epics/board-rendering/story-001-plugin-scaffold-board-layout-card-atlas.md` so `BoardRenderingPlugin`, `BoardLayout`, and `CardAtlas` exist before `ShopAuctionUiPlugin` is registered fifth in `PresentationPlugin`.
- Unlocks: Stories 002, 003, 004, 005, 006, 007, 008, 009.

## Completion Notes

**Completed**: 2026-05-05
**Verdict**: COMPLETE WITH NOTES
**Criteria**: 8/8 passing.
**Test Evidence**: Logic evidence at `tests/unit/shop_auction_ui/plugin_scaffold_formulas_test.rs`; `cargo test -p client --test shop_auction_ui_plugin_scaffold_formulas_test` passed 8/8.
**Verification**: `cargo fmt -p client -- --check`, `cargo check -p client`, `git diff --check 2dbc988^..2dbc988`, and `git diff --check` passed.
**Implementation Commit**: worker `733158c`; main integration `2dbc988`; current `origin/main`/`HEAD` before closure commit `f527247`.
**Deviations**: Advisory only - story manifest version is 2026-05-01 and current control manifest version is 2026-05-05. No blocking GDD, ADR-019, ADR-021, or Bevy 0.18 deviation found.
**Scope**: Implementation changed only `client/Cargo.toml`, `client/src/presentation/mod.rs`, `client/src/ui/mod.rs`, `client/src/ui/shop_auction/mod.rs`, and `tests/unit/shop_auction_ui/plugin_scaffold_formulas_test.rs`. Unrelated dirty UX and asset files were not included.
**Code Review**: Skipped - Lean mode.
**Sprint Status**: Unchanged; no matching `SHOP-AUCTION-UI-001` / `SAU-001` row exists in `production/sprint-status.yaml`.
