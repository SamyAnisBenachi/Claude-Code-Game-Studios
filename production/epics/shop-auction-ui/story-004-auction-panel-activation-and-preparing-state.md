# Story 004: Auction Panel Activation and Preparing State

> **Epic**: Shop / Auction UI
> **Status**: Complete
> **Layer**: Presentation
> **Type**: Integration
> **Manifest Version**: 2026-05-05

## Context

**GDD**: `design/gdd/shop-auction-ui.md`
**Requirement**: `TR-SAU-004`, `TR-SAU-006`
**ADR Governing Implementation**: [ADR-013: Auction System State](../../../docs/architecture/adr-013-auction-system-state.md), [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)

**Requirement basis**: GDD `DRAFT_AUCTION Panel` Rule 1 and edge cases `SAU-EG4a`, `SAU-EG4b`, `SAU-EG4c`, `SAU-EG5`, `SAU-EG6a`, and `SAU-EG6b` require auction activation to buffer `S2CAuctionCard` and `S2CPhaseChanged(DRAFT_AUCTION)` in either arrival order, enter `AUCTION_PREPARING` only for card-first arrival, dismiss on non-auction phase, and time out after 10 seconds. GDD Rule 2 requires the DRAFT_AUCTION shop footer to remain visible but fully locked, with refresh hidden and no purchase or refresh messages sent.

Auction activation requires both `S2CAuctionCard` and `S2CPhaseChanged(DRAFT_AUCTION)`. The UI must handle either arrival order, a preparing state when the card arrives first, and normal dismissal if a non-auction phase arrives while preparing.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**: Use `liv-bevy-018` before touching any Bevy `.rs` implementation and `liv-bevy-lightyear` before touching any Lightyear network code. Treat each Lightyear `MessageReceiver<T>` drain as a single-consumer boundary: `S2CPhaseChanged` is drained only by `phase_sink_system`, and any Shop/Auction S2C message drain added for this story must have one production owner. Use Bevy UI Required Components (`Node`, `Text`, `ImageNode`, etc.) and do not use deprecated Bevy bundle APIs such as `NodeBundle`, `SpriteBundle`, or `Camera2dBundle`.

**Control Manifest Rules (Presentation layer)**:
- Required: `S2CPhaseChanged` is drained through `phase_sink_system`, then exposed to sub-plugins through `Res<CurrentClientPhase>`.
- Forbidden: Shop/Auction sub-plugin systems must not directly drain `MessageReceiver<S2CPhaseChanged>`.
- Required: Bevy UI entities use the Required Components API; no deprecated `*Bundle` types.
- Required: Lightyear message handling is single-drain per message type; do not confuse Lightyear `MessageReceiver<T>` with Bevy `MessageReader<T>`.

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

## Performance Budget

- Presentation steady-state: < 1 ms/frame.
- Phase-boundary activation/preparing transition: < 3 ms.
- Activation and preparing-state buffer checks must be O(1).
- Countdown/preparing updates must not allocate per frame; update pre-pooled UI entities and existing fields only.

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

**Status**: [x] Integration evidence created and passing

## Dependencies

- Depends on: [Story 001](story-001-plugin-scaffold-panel-tree-and-formulas.md), Auction System card dispatch, Card Acquisition shop slot data for footer.
- Unlocks: Stories 005, 006, 007.

## Completion Notes

**Completed**: 2026-05-05
**Criteria**: 7/7 passing. `S2CAuctionCard` card-first enters `AUCTION_PREPARING` with card/price, grey timer, and no countdown; phase-first remains inactive until card arrives; both orders activate countdown from `S2CPhaseChanged.timer_duration_ms`; preparing times out after 10 seconds into connection error; non-auction phase clears/dismisses preparing; DRAFT_AUCTION footer is visible and locked with refresh hidden; shop slot and refresh clicks send no purchase/refresh messages during auction.
**Deviations**: None blocking. Advisory only: manual visual evidence remains deferred to Story 009 / `production/qa/evidence/shop-auction-ui-auction-preparing-evidence.md`.
**Test Evidence**: Integration test `tests/integration/shop_auction_ui/auction_activation_test.rs` passed 6/6. Requested adjacent regressions passed: `shop_auction_ui_plugin_scaffold_formulas_test` 8/8, `shop_auction_ui_draft_initial_grid_test` 9/9, `shop_auction_ui_shop_panel_test` 8/8, and `presentation_plugin_scaffold_test` 3/3. `cargo fmt -p client -- --check`, `cargo check -p client`, and `git diff --check` passed on final `main`.
**Code Review**: Skipped - lean mode; `production/review-mode.txt` is absent.
