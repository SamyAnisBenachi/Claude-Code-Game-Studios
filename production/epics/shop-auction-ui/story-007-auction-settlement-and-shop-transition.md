# Story 007: Auction Settlement and Shop Transition

> **Epic**: Shop / Auction UI
> **Status**: Blocked
> **Layer**: Presentation
> **Type**: Visual/Feel
> **Manifest Version**: 2026-05-05

## Context

**GDD**: `design/gdd/shop-auction-ui.md`
**UX Spec**: `design/ux/shop-auction-ui.md`
**Requirement**: `TR-SAU-003`, `TR-SAU-006`
**ADR Governing Implementation**: [ADR-013: Auction System State](../../../docs/architecture/adr-013-auction-system-state.md), [ADR-019: Economy Resource Architecture](../../../docs/architecture/adr-019-economy-resource-architecture.md), [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)
**Control Manifest**: `docs/architecture/control-manifest.md` version `2026-05-05`.
**Readiness status**: Story content is repaired for current ADR, UX, engine, and manifest guidance, but `/dev-story` remains blocked until SAU-004, SAU-005, and SAU-006 are implemented and marked Complete.

This story handles terminal auction settlement, winner/no-bid presentation, card-to-hand feedback for local wins, stale late-message suppression, and the transition into the interactive DRAFT_SHOP panel.

**GDD trace**:
- DRAFT_AUCTION Rule 9 requires `S2CAuctionSettled { winner, amount }` to be terminal: clear in-flight bid tracking, discard late accepted/rejected effects, show the correct winner/no-bid settlement state, and transition into DRAFT_SHOP.
- DRAFT_SHOP Rule 1 requires auction-round shop activation from settlement with already-populated shop slots; the DRAFT_SHOP timer starts from the phase duration only after the auction-to-shop transition completes.
- GDD edge cases require same-tick accepted-plus-settled handling, locally-expired settlement recovery, post-settlement stale message suppression, and immediate cancellation when `S2CPhaseChanged(PLACEMENT)` arrives during settlement.
- Acceptance criteria `SAU-SET1a`, `SAU-SET1b`, `SAU-SET2`, `SAU-SET3`, `SAU-DA13`, `SAU-EG3`, `SAU-V10`, `SAU-V11`, and `SAU-V14` are the direct behavior basis for this story.

**UX trace**:
- `design/ux/shop-auction-ui.md` defines panel-scoped settlement overlays for local win, opponent win, and no-bid outcomes; overlays must never cover HUD chips or the top strip.
- The auction-to-shop transition is a 350ms standard-motion sequence: settlement overlay holds, auction panel slides/dismisses, and shop panel expands up already populated.
- The DRAFT_SHOP timer starts after the shop expand animation completes, not when settlement first arrives.
- The vertical layout contract reserves the top HUD band, bottom HUD/hand band, and active content band so settlement overlays and transition panels do not occlude HUD, hand, or board-critical areas.
- `S2CPhaseChanged(PLACEMENT)` interrupts any settlement animation immediately; phase entry is never delayed by visual polish.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Required skills**: use `liv-bevy-018` before editing any Bevy `.rs` file and `liv-bevy-lightyear` before editing any Lightyear/networking `.rs` file.

**ADR/control-manifest rules for this story**:
- Run UI work in the ADR-021 `PresentationSet` order: `PhaseTransition` -> `MessageDrain` -> `StateSync` -> `AnimationTick`.
- Keep Lightyear `MessageReceiver<T>` handling single-drain per message type. Register exactly one production drainer for each S2C message type consumed by this flow.
- Do not drain `MessageReceiver<S2CPhaseChanged>` in Shop/Auction UI. Read `Res<CurrentClientPhase>` populated by `phase_sink_system`.
- Use Bevy 0.18 Required Components API for all UI entities (`Node`, `Text`, `TextSpan`, `ImageNode`, `ChildOf` where parenting is needed). Do not use deprecated bundles or APIs such as `NodeBundle`, `SpriteBundle`, `Camera2dBundle`, `UiImage::new()`, `commands.entity(e).set_parent(...)`, or `Color::rgba()`.
- This is presentation synchronization only. Never mutate authoritative auction, economy, card ownership, gold, reservation, or phase state from local UI.

## Acceptance Criteria

- [ ] `S2CAuctionSettled { winner: Some(local_player) }` enters settling state and requests local card feedback toward the hand area.
- [ ] `winner: Some(opponent)` enters settling state without moving a card toward the local hand.
- [ ] `winner: None` enters settling state with no gold/card movement.
- [ ] Settlement clears in-flight bid state, pending gate flags, and queued late accepted/rejected effects.
- [ ] Accepted or rejected messages after settlement do not update current price, leader, or bid button state.
- [ ] Auction panel slides/dismisses and shop panel expands over the UX-specified 350ms standard-motion transition, with reduced-motion preserving state ordering.
- [ ] DRAFT_SHOP timer starts only after the shop expansion animation completes.
- [ ] If `S2CPhaseChanged(PLACEMENT)` arrives during settlement animation, animation cancels immediately and phase entry is not delayed.

## Implementation Notes

- Settlement is terminal and takes precedence over in-transit accepted/rejected messages.
- If `S2CAuctionBidAccepted` and `S2CAuctionSettled` are processed in the same batch, process accepted first, then settled, and render only the final settlement state.
- Card movement animation should be requested through Card Animations; do not block phase transition on animation completion.
- Buffered `S2CShopSlots` from auction should be applied before shop expansion begins so the shop is populated when revealed.
- Auction-win `S2CCardAcquired { source: AuctionWon }` is produced by Auction System settlement code, not by Card Acquisition shop/draft dispatch. This story may consume that message for local presentation/hand feedback but must not route auction wins through the shop/draft acquisition handler.
- Keep all Lightyear `MessageReceiver<T>` consumers single-drain. If SAU-004, SAU-005, SAU-006, HUD, Hand UI, or shared presentation code already exposes a bridge/resource for a needed S2C message, consume that bridge rather than registering a duplicate receiver.
- Phase interrupt handling reads `CurrentClientPhase`; Shop/Auction UI must not register its own `MessageReceiver<S2CPhaseChanged>`.
- Settlement overlays, the 350ms transition, timer-start deferral, vertical layout reserves, and phase interrupt behavior follow `design/ux/shop-auction-ui.md`.
- Keep state/integration tests separate from final screenshot evidence; Story 009 owns the full visual/accessibility evidence pass.

## Performance Budget

- Presentation steady-state remains under the ADR-021 budget of < 1 ms/frame.
- Settlement and auction-to-shop phase-boundary work must remain inside the ADR-021 < 3 ms spike budget.
- Settlement cleanup is O(1): clear the in-flight bid flag, pending accepted/gold gate flags, queued late accepted/rejected state, and one settlement state resource/component.
- Transition work updates pre-pooled panel/overlay/timer entities and animation targets only; do not allocate or spawn/despawn per frame during steady state.

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

- Depends on: [Story 004](story-004-auction-panel-activation-and-preparing-state.md) - must be implemented and marked Complete before SAU-007 `/dev-story`; provides active auction panel state, preparing/activation buffers, locked footer boundary, and auction timer source.
- Depends on: [Story 005](story-005-auction-bid-buttons-affordability-and-inflight.md) - must be implemented and marked Complete before SAU-007 `/dev-story`; provides bid button state, local affordability/in-flight flags, leader replacement state, and locally-expired state that settlement clears.
- Depends on: [Story 006](story-006-auction-accepted-rejected-feedback.md) - must be implemented and marked Complete before SAU-007 `/dev-story`; provides accepted/rejected message handling, late-message guards, and the two-message gold gate state that settlement clears.
- Depends on: Auction System settlement dispatch, including `S2CAuctionSettled` broadcast and auction-win `S2CCardAcquired { source: AuctionWon }` unicast from auction code, not Card Acquisition shop/draft dispatch.
- Depends on: Card Acquisition shop slot data for pre-populated DRAFT_SHOP slots; auction-win card acquisition is not owned by Card Acquisition shop/draft dispatch.
- Unlocks: Story 008 and post-auction shop visual path.

## Blockers

- SAU-004 is currently `Ready`, not `Complete`.
- SAU-005 is currently `Ready`, not `Complete`.
- SAU-006 is currently `Blocked`, not `Complete`.
- `/dev-story` safe next: No. Launch only after SAU-004, SAU-005, and SAU-006 are Complete.
