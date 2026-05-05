# Story 005: Auction Bid Buttons, Affordability, and In-Flight

> **Epic**: Shop / Auction UI
> **Status**: Complete
> **Layer**: Presentation
> **Type**: UI
> **Manifest Version**: 2026-05-05

## Context

**GDD**: `design/gdd/shop-auction-ui.md`
**UX Spec**: `design/ux/shop-auction-ui.md`
**Requirement**: `TR-SAU-001`, `TR-SAU-002`, `TR-SAU-005`
**ADR Governing Implementation**: [ADR-013: Auction System State](../../../docs/architecture/adr-013-auction-system-state.md), [ADR-019: Economy Resource Architecture](../../../docs/architecture/adr-019-economy-resource-architecture.md), [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)
**Control Manifest**: `docs/architecture/control-manifest.md` version `2026-05-05`.

**GDD trace**:
- `design/gdd/shop-auction-ui.md` DRAFT_AUCTION Rule 4 requires exactly three immediate preset bid buttons, no free-form bid field, no separate confirmation step, total-commitment primary labels, and in-flight "BIDDING..." feedback after a valid click.
- DRAFT_AUCTION Rule 5 requires proactive lockouts for hand full, insufficient free gold, per-button unaffordability, current leader replacement, bid in-flight state, and locally expired state.
- DRAFT_AUCTION Rule 8 requires locally expired auctions to freeze the timer at 0, disable bids, show "Auction ending...", then "Awaiting server..." if unresolved.
- Formula D.1 and acceptance criteria `SAU-F1`, `SAU-F2`, `SAU-F3`, `SAU-DA1`, `SAU-DA2`, `SAU-DA3`, `SAU-DA6`, `SAU-DA8`, `SAU-DA9`, `SAU-DA11a`, `SAU-DA11b`, and `SAU-DA12` are the direct behavior basis for this story.

**UX/accessibility alignment**: `design/ux/shop-auction-ui.md` and `design/accessibility-requirements.md` now explicitly keep immediate preset bid buttons with no separate confirmation step. Misclick mitigation is the resolved path: total-commitment labels, 44 x 44 minimum targets, keyboard focus rings, affordability gating, same-frame in-flight disable, exact one-send semantics, and visible "BIDDING..." feedback on only the clicked button.

This story implements the three preset bid buttons, proactive affordability lockouts, "YOU ARE LEADING" replacement state, and the in-flight "BIDDING..." visual after a click. It sends `C2SPlaceBid` intent but does not commit local price/gold until server response stories process S2C messages.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Required skills**: use `liv-bevy-018` before editing any Bevy `.rs` file and `liv-bevy-lightyear` before editing any Lightyear/networking `.rs` file.

**ADR/control-manifest rules for this story**:
- `ShopAuctionUiPlugin` remains the fifth `PresentationPlugin` sub-plugin after Card Animations, Board Rendering, Hand UI, and HUD.
- Run UI work in the ADR-021 `PresentationSet` order: `PhaseTransition` -> `MessageDrain` -> `StateSync` -> `AnimationTick`.
- Do not drain `MessageReceiver<S2CPhaseChanged>` in Shop/Auction UI. Read `Res<CurrentClientPhase>` populated by `phase_sink_system`.
- Do not drain `MessageReceiver<S2CGoldUpdate>` in Shop/Auction UI. Read `Res<PlayerEconomyView>` for local own-gold state.
- Do not add a second Lightyear drain for `S2CGoldBroadcast`. Consume `HudGoldBroadcastMessage` or a shared public-gold bridge/resource produced from the existing HUD drain, so affordability can read `gold - reserved_gold` without violating the single-drain rule.
- Send `C2SPlaceBid` through the Lightyear reliable C2S sender only as player intent. Never mutate authoritative price, leader, gold, reservation, or ownership from local input.
- Use Bevy 0.18 Required Components API for all UI entities (`Node`, `Text`, `TextSpan`, `ImageNode`, `ChildOf` where parenting is needed). Do not use `NodeBundle`, `SpriteBundle`, `Camera2dBundle`, `UiImage::new()`, `commands.entity(e).set_parent(...)`, or `Color::rgba()`.
- Toggle pre-pooled auction/bid UI entities in steady state; do not spawn/despawn bid buttons per frame.

## Acceptance Criteria

- [ ] Buttons show total commitment as primary label and increment as secondary label.
- [ ] +1/+3/+5 buttons evaluate affordability from `local_free_gold = gold - reserved_gold`.
- [ ] Hand size 10 disables all bids and shows hand-full auction message.
- [ ] If local player is current leader, all three buttons are hidden and "YOU ARE LEADING" fills the area.
- [ ] Clicking an enabled button immediately sends exactly one `C2SPlaceBid { amount }` with no confirmation step.
- [ ] Clicked button enters `BIDDING...`; other buttons become generic disabled.
- [ ] No additional bid can be sent while a bid is in-flight.
- [ ] Locally expired timer disables all buttons and shows ending/awaiting-server state according to elapsed time.

## Implementation Notes

- Current price source is last accepted amount or starting price if no bids have been accepted.
- Do not add a free-form text bid field.
- Do not add a separate bid confirmation prompt/modal; the immediate preset-button interaction is the resolved UX/accessibility path.
- Do not optimistically update current leader or current price on click.
- Do not optimistically update local gold, reserved gold, opponent gold, or affordability after click; only the in-flight visual is local.
- Use `local_free_gold(gold, reserved_gold)` from the story-001 formula path and keep defensive saturating subtraction for server-invariant violations.
- Consume public gold/reservation data through the existing `S2CGoldBroadcast` single-drain bridge (`HudGoldBroadcastMessage`) or a shared bridge/resource if one exists by implementation time. Do not read HUD text strings or duplicate the Lightyear receiver.
- The current leader state may be seeded from existing auction panel state/reconnect snapshot state; accepted/rejected response processing remains Story 006.
- Use Card Animations for timer and button feedback if animation hooks already exist; otherwise expose message hooks for later animation story.

## Performance Budget

Presentation steady-state must remain under 1 ms per frame. Bid-state evaluation is fixed-size over three preset buttons plus one leader badge and must not scan card catalogs, all hands, or all player state per frame. On click, update only the clicked button, the other two button states, and a single in-flight flag in the same frame; no allocation-heavy per-frame work is expected. Locally expired timer checks and elapsed-label transitions must stay within the ADR-021 phase-boundary spike budget of less than 3 ms.

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

**Status**: [x] Created and passing

## Dependencies

- Depends on: [Story 001](story-001-plugin-scaffold-panel-tree-and-formulas.md) - Complete; provides `ShopAuctionUiPlugin`, panel roots, bid-label formula, `local_free_gold`, and auction border formula scaffolding.
- Depends on: [Story 004](story-004-auction-panel-activation-and-preparing-state.md) - implementation must be complete before SAU-005 `/dev-story` launches; it provides the active auction panel, preparing/activation state, locked footer boundary, and timer source that bid buttons attach to.
- Depends on: HUD gold broadcast handling (`production/epics/hud/story-002-gold-mana-display.md` and `production/epics/hud/story-006-economy-auction-inline-gold.md`) - Complete; provides the existing `S2CGoldBroadcast` Lightyear drain and public reserved-gold state/message path. SAU-005 must consume that path or a shared bridge, not add a duplicate receiver.
- Depends on: Presentation Layer Story 001 and Story 002 - Complete; provide `PresentationPlugin`, `PresentationSet`, `phase_sink_system`, `CurrentClientPhase`, and `PlayerEconomyView`.
- Depends on: `shared/src/protocol.rs` protocol registration - Complete; `C2SPlaceBid`, `S2CGoldBroadcast`, `S2CAuctionBidAccepted`, and `S2CAuctionBidRejected` are registered on `ReliableChannel`.
- Depends on: Auction System bid dispatch and Economy gold broadcast dispatch being integrated for runtime end-to-end behavior.
- Unlocks: Story 006.

## Completion Notes

**Completed**: 2026-05-05
**Criteria**: 8/8 passing
**Deviations**: None blocking. Advisory only: manual visual/accessibility evidence for the final bid-button presentation remains deferred to Story 009 / `production/qa/evidence/shop-auction-ui-bid-buttons-evidence.md`.
**Test Evidence**: `cargo test -p client --test shop_auction_ui_auction_bid_buttons_test` passed 5/5. Adjacent regressions passed: `shop_auction_ui_auction_activation_test` 6/6, `shop_auction_ui_shop_panel_test` 8/8, `shop_auction_ui_draft_initial_grid_test` 9/9, `presentation_plugin_scaffold_test` 3/3, and `shop_auction_ui_plugin_scaffold_formulas_test` 8/8. `cargo fmt -p client -- --check`, `cargo check -p client`, and `git diff --check` passed.
**Code Review**: Skipped - lean mode.
**Integration Notes**: Worker commit `b118dcabcb755e606eb212b55010baf05e8228eb` was applied onto current `main` after Sprint 5 smoke gate commit `38f613a`. Integration fixes updated the pre-pooled entity count test for the new bid-status text entity and moved bid-status text below the button row to avoid overlap with disabled hand-full buttons.
