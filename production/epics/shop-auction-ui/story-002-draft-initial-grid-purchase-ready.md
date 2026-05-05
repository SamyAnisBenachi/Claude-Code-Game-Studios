# Story 002: Draft Initial Grid Purchase Ready

> **Epic**: Shop / Auction UI
> **Status**: Ready
> **Layer**: Presentation
> **Type**: UI
> **Manifest Version**: 2026-05-05

## Context

**GDD**: `design/gdd/shop-auction-ui.md`
**Requirement**: `TR-SAU-006` - Panel transitions, timings, and input gating across DRAFT_INITIAL, DRAFT_AUCTION, and DRAFT_SHOP.
**GDD trace**: `design/gdd/shop-auction-ui.md` DRAFT_INITIAL Rules 1, 3, 4, and 7; acceptance criteria `SAU-DI1`, `SAU-DI2`, `SAU-DI3`, `SAU-DI6`, `SAU-DI7`, `SAU-DI8`, `SAU-DI9`, `SAU-DI10`, and `SAU-DI11`.
**ADR Governing Implementation**: [ADR-015: Card Acquisition Shop State](../../../docs/architecture/adr-015-card-acquisition-shop-state.md), [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)
**Control Manifest**: `docs/architecture/control-manifest.md` version `2026-05-05`.
**Readiness dependency status**: SAU-001 is Complete; Card Acquisition DRAFT_INITIAL offering generation and dispatch are Complete; CDP-006 Network Dispatch Wiring is Complete; ADR-015 and ADR-021 are Accepted; `TR-SAU-006` is active.
**UX scope status**: `design/ux/shop-auction-ui.md` exists and resolves the first-session tooltip placement, dismissal, and persistence contract. Tooltip behavior is deferred from SAU-002.

DRAFT_INITIAL activates only after both `S2CPhaseChanged(DRAFT_INITIAL)` and `S2CDraftOffering` are available. The UI shows the sorted 3 x 3 offering, supports purchase clicks, marks confirmed purchases, handles hand-full/insufficient-gold lockouts, and exposes Ready/Retract Ready without disabling further purchases.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Required skills**: use `liv-bevy-018` before editing any Bevy `.rs` file and `liv-bevy-lightyear` before editing any Lightyear/networking `.rs` file.

**ADR-021 / control-manifest rules for this story**:
- `ShopAuctionUiPlugin` remains the fifth `PresentationPlugin` sub-plugin after Card Animations, Board Rendering, Hand UI, and HUD.
- Run phase handling in the ADR-021 `PresentationSet` order: `PhaseTransition` -> `MessageDrain` -> `StateSync` -> `AnimationTick`.
- Do not drain `MessageReceiver<S2CPhaseChanged>` in Shop/Auction UI. Read `Res<CurrentClientPhase>` populated by the shared `phase_sink_system`.
- Do not drain `MessageReceiver<S2CGoldUpdate>` in Shop/Auction UI. Read `Res<PlayerEconomyView>` populated by the shared economy-view system.
- Any Lightyear `MessageReceiver<T>` introduced for DRAFT_INITIAL-only messages must have exactly one production owner; first drain consumes the frame's messages for all later systems.
- Use bevy_ui `Node`, `Text`, `TextSpan`, and `ImageNode` entities. Never use world-space sprites for this panel.
- Use Bevy 0.18 Required Components API. Never use `NodeBundle`, `SpriteBundle`, `Camera2dBundle`, or any other `*Bundle` type.
- Toggle pre-pooled panel/slot `Visibility` in steady state; do not spawn/despawn UI entities during normal DRAFT_INITIAL updates.
- Respect the presentation performance guardrail: steady state under 1 ms per frame and activation/dismissal phase-boundary spikes under 3 ms.

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
- Drain/consume Lightyear messages only in `PresentationSet::MessageDrain`; apply ECS/UI changes in `StateSync`.
- If this story owns the `S2CDraftOffering` and `S2CCardAcquired` receivers, it must be the only production drain for those receiver types. If another shared presentation bridge already owns either receiver, consume that bridge's local resource/message instead of adding a second receiver.
- Send `C2SPurchaseCard` and `C2SSignalReady` through the Lightyear reliable C2S sender only as player intent. Do not mutate confirmed purchase, gold, hand, or ownership state from local input.
- Do not optimistically mark a purchase as bought before S2C confirmation.
- Use Bevy 0.18 Required Components API for all UI entities (`Node`, `Text`, `TextSpan`, `ImageNode`, `ChildOf` where parenting is needed). Do not use any `*Bundle` types, `UiImage::new()`, `commands.entity(e).set_parent(...)`, or `Color::rgba()`.
- Use `C2SSignalReady` for both Ready and Retract Ready.
- Tooltip behavior is not implemented in this story. `design/ux/shop-auction-ui.md` defines non-occluding placement, dismiss behavior, and the local preference key `lanes_and_lies.shop_auction.draft_tooltip_dismissed`; implementation and evidence are deferred to Story 009 or a separately scoped tooltip story.

## Performance Budget

Presentation steady-state must remain under 1 ms per frame. This story is expected to have no material steady-state impact because DRAFT_INITIAL updates are bounded to 9 pre-pooled slots plus Ready/status controls, with normal frames limited to visibility/text/state changes and no per-frame allocation-heavy scans. Activation/phase-dismiss frames must stay within the ADR-021 phase-boundary spike budget of less than 3 ms.

## Out of Scope

- DRAFT_SHOP panel refresh/purchase flow (Story 003).
- Auction panel state (Stories 004-007).
- Final visual/accessibility evidence (Story 009).
- First-session tooltip behavior. The UX contract is resolved in `design/ux/shop-auction-ui.md`, including non-occluding placement, dismiss behavior, and persistence key `lanes_and_lies.shop_auction.draft_tooltip_dismissed`; implementation is deferred from SAU-002 to Story 009 or a separately scoped tooltip story.

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

- Readiness gates confirmed: SAU-001 Complete; Card Acquisition draft offering dispatch Complete; CDP-006 Complete; ADR-015/ADR-021 Accepted; `TR-SAU-006` active.
- Depends on: `production/epics/presentation-layer/story-001-presentation-plugin-set-and-phase-sink.md` is Complete and provides `PresentationPlugin`, `PresentationSet`, `phase_sink_system`, and `CurrentClientPhase`.
- Depends on: `production/epics/board-rendering/story-001-plugin-scaffold-board-layout-card-atlas.md` is Complete and provides the ADR-021 `BoardRenderingPlugin` registration slot and shared `CardAtlas` resource contract.
- Depends on: `production/epics/shop-auction-ui/story-001-plugin-scaffold-panel-tree-and-formulas.md` is Complete and provides `ShopAuctionUiPlugin`, panel roots, and shared formula scaffolding.
- Depends on: `production/epics/lightyear-protocol-verification/story-002-all-protocol-message-types.md` is Complete and provides `C2SPurchaseCard`, `C2SSignalReady`, `S2CDraftOffering`, `S2CCardAcquired`, and `S2CGoldUpdate` protocol types on the reliable channel.
- Depends on: `production/epics/card-acquisition/story-002-draft-initial.md` is Complete and provides the server-authoritative DRAFT_INITIAL offering branch and `S2CDraftOffering` behavior.
- Depends on: `production/epics/card-acquisition/story-005-purchase-flow.md` is Complete and provides server-authoritative purchase validation, hand-cap rejection, and confirmed acquisition behavior.
- Depends on: `production/epics/card-data-pool/story-006-network-dispatch-wiring.md` is Complete and provides reliable unicast dispatch for `S2CDraftOffering`.
- Depends on: `production/epics/economy-system/story-006-network-dispatch-wiring.md` is Complete and provides reliable unicast dispatch for own `S2CGoldUpdate`.
- Depends on: `production/epics/hud/story-002-gold-mana-display.md` is Complete and provides the own-gold display surface that receives the insufficient-gold flash request.
- Unlocks: DRAFT_INITIAL M2 playable path.
