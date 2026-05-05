# Story 006: Auction Accepted/Rejected Feedback

> **Epic**: Shop / Auction UI
> **Status**: Complete
> **Layer**: Presentation
> **Type**: Integration
> **Manifest Version**: 2026-05-05

## Context

**GDD**: `design/gdd/shop-auction-ui.md`
**UX Spec**: `design/ux/shop-auction-ui.md`, `design/ux/interaction-patterns.md`
**Requirement**: `TR-SAU-001`, `TR-SAU-005`
**ADR Governing Implementation**: [ADR-013: Auction System State](../../../docs/architecture/adr-013-auction-system-state.md), [ADR-019: Economy Resource Architecture](../../../docs/architecture/adr-019-economy-resource-architecture.md), [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)
**Control Manifest**: `docs/architecture/control-manifest.md` version `2026-05-05`.
**Readiness status**: Story content is repaired for current manifest rules. SAU-004 and SAU-005 are Complete, so SAU-006 is unblocked for `/dev-story`.

This story processes `S2CAuctionBidAccepted`, `S2CAuctionBidRejected`, and the associated `S2CGoldBroadcast` gate that prevents button re-enable against stale free-gold data. It resolves in-flight bid state and keeps accepted/rejected UI synchronized with server authority.

**GDD trace**:
- DRAFT_AUCTION Rule 6 requires accepted bids to update `current_price`, leader display, timer target, and the two-message opponent-bid re-enable gate: an opponent `S2CAuctionBidAccepted` and a local-player `S2CGoldBroadcast` must both arrive before bid buttons re-enable.
- DRAFT_AUCTION Rule 7 requires rejected bids to clear any `BIDDING...` label, re-enable buttons per current affordability, and display the exact reason-specific toast copy listed below.
- DRAFT_AUCTION Rule 9 requires settlement to clear in-flight bid state plus `pending_bid_accepted` and `pending_gold_broadcast_seen` unconditionally; post-settlement accepted/rejected messages must not revive bid controls.
- Acceptance criteria `SAU-DA4`, `SAU-DA5`, `SAU-DA6`, `SAU-DA7`, `SAU-DA10`, `SAU-DA12`, `SAU-DA13`, `SAU-DA14`, and `SAU-DA15` are the direct behavior basis for this story.

**UX trace**:
- `design/ux/shop-auction-ui.md` defines `Auction rejected` as: `S2CAuctionBidRejected` clears in-flight state, buttons re-evaluate, and a mapped toast appears.
- `design/ux/shop-auction-ui.md` maps auction rejection to `PTN-FDB-005 Notification Toast` and requires the toast area to support 2 lines without bid-button layout reflow.
- `design/ux/interaction-patterns.md` `PTN-FDB-005` requires toasts to stay inside active panel bounds, use 120ms fade in, hold for 2.0s, fade out over 120ms, and replace the previous toast instead of stacking.

**Rejection toast map**:
- `InsufficientGold` -> "Not enough gold"
- `AmountTooLow` -> "Bid must be at least [minimum_bid]g"
- `AlreadyLeader` -> "You are already leading"
- `HandFull` -> "Hand full — no bids possible this auction"
- `AuctionExpired` -> "Auction has ended"

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Required skills**: use `liv-bevy-018` before editing any Bevy `.rs` file and `liv-bevy-lightyear` before editing any Lightyear/networking `.rs` file.

**ADR/control-manifest rules for this story**:
- `ShopAuctionUiPlugin` remains the fifth `PresentationPlugin` sub-plugin after Card Animations, Board Rendering, Hand UI, and HUD.
- Run UI work in the ADR-021 `PresentationSet` order: `PhaseTransition` -> `MessageDrain` -> `StateSync` -> `AnimationTick`.
- Do not drain `MessageReceiver<S2CPhaseChanged>` in Shop/Auction UI. Read `Res<CurrentClientPhase>` populated by `phase_sink_system`.
- Do not drain `MessageReceiver<S2CGoldUpdate>` in Shop/Auction UI. Read `Res<PlayerEconomyView>` for local own-gold state.
- Keep Lightyear S2C handling single-drain per message type. If SAU-004/005 or HUD expose a bridge/resource for `S2CGoldBroadcast`, consume that path rather than registering a duplicate receiver.
- `S2CAuctionBidAccepted` and `S2CAuctionBidRejected` processing is server-authoritative view synchronization only. Never mutate authoritative price, leader, gold, reservation, ownership, or protocol state from local UI.
- Use Bevy 0.18 Required Components API for all UI entities (`Node`, `Text`, `TextSpan`, `ImageNode`, `ChildOf` where parenting is needed). Do not use `NodeBundle`, `SpriteBundle`, `Camera2dBundle`, `UiImage::new()`, `commands.entity(e).set_parent(...)`, or `Color::rgba()`.
- Update existing/pre-pooled auction feedback entities in steady state; do not spawn/despawn bid buttons or toast roots per message.

## Acceptance Criteria

- [ ] Accepted local bid updates current price, sets local player as leader, hides buttons, and shows "YOU ARE LEADING".
- [ ] Accepted opponent bid updates current price and leader but does not re-enable buttons until local-player `S2CGoldBroadcast` arrives.
- [ ] If local-player `S2CGoldBroadcast` arrives before opponent accepted bid, the two-message gate still re-enables immediately once both are present.
- [ ] Opponent `S2CGoldBroadcast` does not satisfy the local re-enable gate.
- [ ] Accepted bid writes timer target fill from `new_timer_ms / auction_timer_ms`.
- [ ] Rejected bid clears `BIDDING...`, re-evaluates affordability from current `local_free_gold = gold - reserved_gold`, and shows the mapped toast text from the story's rejection toast map.
- [ ] Rejection reasons map to exact GDD Rule 7 toast messages: `InsufficientGold` -> "Not enough gold"; `AmountTooLow` -> "Bid must be at least [minimum_bid]g"; `AlreadyLeader` -> "You are already leading"; `HandFull` -> "Hand full — no bids possible this auction"; `AuctionExpired` -> "Auction has ended".
- [ ] Rejection toast uses `PTN-FDB-005`: panel-scoped position, 120ms fade in, 2.0s hold, 120ms fade out, replacement resets the hold timer, and no vertical toast stacking.
- [ ] Settlement clears pending accepted/gold gate flags unconditionally.

## Implementation Notes

- Store pending gate flags/resources explicitly so tests can assert both arrival orders.
- Do not compute affordability from stale local gold after an opponent accepted bid.
- Rejected messages arriving after settlement must not re-enable bid buttons; Story 007 owns terminal settlement state, but this story should prepare for the terminal guard.
- Timer easing is a Card Animations concern; this story writes a test-observable target.
- `AmountTooLow` toast interpolation uses the current minimum bid from GDD Formula D.3 (`minimum_bid = current_price + 1`) unless the protocol supplies a server minimum. Do not invent client-side bid correction logic beyond formatting the toast.
- The toast root and text node should already exist or be pre-pooled by the Shop/Auction UI panel tree. This story updates text/visibility/timers only.

## Performance Budget

- Presentation steady-state remains under the ADR-021 budget of < 1 ms/frame.
- Accepted/rejected feedback processing must be O(1): update at most current price, leader display, three bid buttons, one leader badge, one timer-target resource, the two gate flags, and one toast state.
- Toast and feedback UI state updates must fit inside the global client S2C processing + view update budget of <= 2 ms for the frame; the SAU-006 portion should avoid allocation-heavy per-frame work and should not scan card catalogs, all hands, or all player state.
- Phase-boundary or same-frame settlement cleanup must remain inside the ADR-021 < 3 ms spike budget.

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
- Integration: `tests/integration/shop_auction_ui/auction_feedback_test.rs`

**Status**: [x] Created and passing

## Dependencies

- Depends on: [Story 004](story-004-auction-panel-activation-and-preparing-state.md) - Complete; provides the active auction panel, preparing/activation state, locked footer boundary, and timer source that accepted/rejected feedback updates.
- Depends on: [Story 005](story-005-auction-bid-buttons-affordability-and-inflight.md) - Complete; provides bid buttons, affordability/in-flight state, `BIDDING...` label ownership, and initial `C2SPlaceBid` send semantics that SAU-006 resolves.
- Depends on: Auction System bid accepted/rejected dispatch being integrated for runtime end-to-end behavior.
- Depends on: Economy gold broadcast dispatch and the existing/shared `S2CGoldBroadcast` bridge/resource path being available for the local-player re-enable gate.
- Unlocks: Story 007.

## Blockers

None.

## Completion Notes

**Completed**: 2026-05-05
**Criteria**: 9/9 passing
**Deviations**: None blocking. Advisory only: visual polish for toast animation remains deferred to Story 009, as scoped.
**Test Evidence**: `cargo test -p client --test shop_auction_ui_auction_feedback_test` passed 6/6. Requested regressions passed: `shop_auction_ui_auction_bid_buttons_test` 5/5, `shop_auction_ui_auction_activation_test` 6/6, and `shop_auction_ui_shop_panel_test` 8/8. `cargo fmt -p client -- --check`, `cargo check -p client`, and `git diff --check` passed.
**Code Review**: Skipped - lean mode.
**Integration Notes**: Worker commit `abbbe0f1498d7a949e19e2377244b40e83ad5c91` was merged from `work/sau-006-auction-accepted-rejected-feedback`. Integration fixes corrected the exact `HandFull` rejection toast copy and added coverage for phase-exit cleanup of the accepted/gold gate plus ignored late rejections.
