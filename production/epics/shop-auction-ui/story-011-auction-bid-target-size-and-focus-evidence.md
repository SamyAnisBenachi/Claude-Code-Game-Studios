# Story 011: Auction Bid Target Size and Focus Evidence

> **Epic**: Shop / Auction UI
> **Status**: Complete
> **Layer**: Presentation
> **Type**: UI
> **Manifest Version**: 2026-05-05

## Context

**Sprint Gate**: Sprint 6 S6-04 / QA-COND-0005 Standard-tier accessibility remediation.

**QA condition**: `production/qa/bugs/QA-COND-0005-standard-tier-accessibility-gaps.md` remains Open. The Sprint 6 accessibility disposition row `A11Y-ST-12` states that auction bid buttons need browser/WASM evidence for immediate preset commitments, 44x44 targets, focus rings, affordability gating, in-flight disable, one-send semantics, and `BIDDING...` feedback.

**Observed risk**: The existing auction bid button visuals are believed to be approximately 108x36, which is below the 44px minimum target height. This story remediates or verifies equivalent target area for the +1, +3, and +5 bid controls without changing bid behavior.

**GDD**: `design/gdd/shop-auction-ui.md`
**UX Spec**: `design/ux/shop-auction-ui.md`, `design/ux/interaction-patterns.md`
**Accessibility Source**: `design/accessibility-requirements.md`
**Requirement**: `TR-SAU-002`, `TR-SAU-005`, `TR-SAU-006`, A11Y-ST-12
**ADR Governing Implementation**: [ADR-013: Auction System State](../../../docs/architecture/adr-013-auction-system-state.md), [ADR-019: Economy Resource Architecture](../../../docs/architecture/adr-019-economy-resource-architecture.md), [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)
**Control Manifest**: `docs/architecture/control-manifest.md` version `2026-05-05`.

**GDD trace**:

- `design/gdd/shop-auction-ui.md` DRAFT_AUCTION Rule 4 requires exactly three immediate preset bid buttons, no free-form bid field, no separate confirmation step, total-commitment primary labels, and same-frame `BIDDING...` feedback after a valid click.
- DRAFT_AUCTION Rule 5 requires proactive lockouts for insufficient free gold, per-button unaffordability, current leader replacement, bid in-flight state, hand full, and locally expired state.
- DRAFT_AUCTION Rule 6 and Rule 7 require accepted/rejected server responses to preserve the existing SAU-006 feedback and re-enable behavior.
- Acceptance criteria `SAU-DA1`, `SAU-DA3`, `SAU-DA6`, `SAU-DA11a`, `SAU-DA11b`, and `SAU-DA12` are the direct behavior basis that this story must preserve.

**UX and accessibility trace**:

- `design/ux/shop-auction-ui.md` Accessibility requires all bid targets to be at least 44x44 CSS px at 100 percent UI scale and reachable at 75 percent through 150 percent UI scale.
- `design/ux/shop-auction-ui.md` Keyboard focus order for DRAFT_AUCTION is +1, +3, +5 while bid buttons are visible. Disabled controls receive no keyboard focus. If `YOU ARE LEADING` replaces the bid buttons, no focusable bid target remains.
- `design/ux/shop-auction-ui.md` Focus indicators require a 2px Prism White outline or equivalent high-contrast focus ring.
- `design/ux/interaction-patterns.md` PTN-INP-004 requires Auction Bid Button content to show total commitment primary and increment secondary, e.g. `8g (+1)`, with minimum 44x44 CSS px target and two-line fallback support.
- `design/accessibility-requirements.md` A11Y-ST-12 records immediate preset commitments as addressed in design through total-commitment labels, 44x44 targets, focus rings, affordability gating, same-frame in-flight disable, one-send semantics, and visible `BIDDING...` feedback.

**Engine**: Bevy 0.18 + Lightyear 0.26 + WASM browser evidence | **Risk**: HIGH

**Required skills**: use `liv-bevy-018` before editing any Bevy `.rs` file and `liv-bevy-lightyear` before editing any Lightyear/networking `.rs` file.

**ADR/control-manifest rules for this story**:

- `ShopAuctionUiPlugin` remains the fifth `PresentationPlugin` sub-plugin after Card Animations, Board Rendering, Hand UI, and HUD.
- Run UI work in the ADR-021 `PresentationSet` order: `PhaseTransition -> MessageDrain -> StateSync -> AnimationTick`.
- Do not drain `MessageReceiver<S2CPhaseChanged>` in Shop/Auction UI. Read `Res<CurrentClientPhase>` populated by `phase_sink_system`.
- Do not drain `MessageReceiver<S2CGoldUpdate>` in Shop/Auction UI. Read `Res<PlayerEconomyView>` for local own-gold state.
- Do not add a second Lightyear drain for `S2CGoldBroadcast`. Continue using the existing HUD/shared bridge or resource path used by SAU-005 and SAU-006.
- Send `C2SPlaceBid` only as player intent. Never mutate authoritative price, leader, gold, reservation, ownership, or protocol state from local focus, hover, click, or keyboard input.
- Use Bevy 0.18 Required Components API for all UI entities: `Node`, `Text`, `TextFont`, `TextColor`, `Button`, `Interaction`, `ImageNode`, and `ChildOf` where needed. Do not use `NodeBundle`, `TextBundle`, `UiImage::new()`, `Parent`, `commands.entity(e).set_parent(...)`, or `Color::rgba()`.
- Preserve pre-pooled auction/bid UI entity ownership. Do not spawn/despawn bid buttons per frame.

---

## Scope

### In Scope

- Adjust the +1, +3, and +5 auction bid control layout so each interactive target is at least 44 CSS px high at 100 percent UI scale, or has an explicitly testable equivalent target area that is at least 44x44 CSS px.
- Preserve visible bid button labels from SAU-005: total commitment remains primary text and the increment remains visible as `(+1)`, `(+3)`, or `(+5)`.
- Preserve SAU-005 bid behavior: immediate send with no confirmation, exact one-send semantics, affordability gating, in-flight disable, and clicked-button `BIDDING...` feedback.
- Preserve SAU-006 feedback behavior: accepted/rejected messages clear or restore in-flight state, preserve the two-message local gold gate, preserve mapped rejection toast behavior, and do not revive bid controls after settlement.
- Add or update test-observable UI state so automated tests can assert target bounds, focus state, focus order, hidden disabled focus behavior, label text, and state readability without relying only on manual screenshots.
- Capture browser/WASM evidence showing +1, +3, +5 labels, target bounds, focus bounds, affordable and unaffordable states, `BIDDING...`, and local leader replacement state.

### Out of Scope

- Do not change auction bid increments, bid amounts, current price calculation, free-gold calculation, or server validation.
- Do not add a bid confirmation modal, undo button, free-form bid field, or delayed confirmation step.
- Do not change SAU-005 `C2SPlaceBid` send behavior.
- Do not change SAU-006 accepted/rejected response semantics, toast copy, or two-message gold re-enable gate.
- Do not implement broader SAU-009 layout evidence for DRAFT_INITIAL, DRAFT_SHOP, settlement, OQ9 playtest observation, or full epic visual sign-off.
- Do not close QA-COND-0005 from this story alone.
- Do not modify `production/sprint-status.yaml`, `production/session-state/**`, `AGENTS.md`, or unrelated epics.

---

## Acceptance Criteria

- [x] The +1, +3, and +5 bid controls each expose an interactive target with measured browser/WASM bounds of at least 44 CSS px height and 44 CSS px width at 100 percent UI scale.
- [x] If the visible button art remains smaller than 44px high, the accepted equivalent target area is documented in tests and browser evidence, and focus/click hit bounds still measure at least 44x44 CSS px.
- [x] Bid labels still show total commitment as primary text and increment as secondary text for all three controls, including exact visible increment labels `(+1)`, `(+3)`, and `(+5)`.
- [x] At narrow supported auction widths, bid labels either remain on one line without clipping or use the PTN-INP-004 two-line fallback with total commitment on one line and increment on the next.
- [x] Keyboard focus order in DRAFT_AUCTION is exactly +1, then +3, then +5 while all three bid controls are visible and enabled.
- [x] The focused bid control shows a visible high-contrast focus ring or component state equivalent to the UX requirement of a 2px Prism White outline.
- [x] Disabled unaffordable bid controls are not reachable by keyboard focus and remain visually distinguishable from enabled affordable controls through opacity plus disabled affordance.
- [x] The affordable enabled state, disabled unaffordable state, clicked `BIDDING...` in-flight state, and local `YOU ARE LEADING` replacement state all render text fully inside their target or badge bounds with no clipping, sibling overlap, or unreadable truncation in browser/WASM evidence.
- [x] The local `YOU ARE LEADING` state hides the +1, +3, and +5 bid controls and leaves no focusable item in the bid area.
- [x] Clicking or pressing Enter on an enabled bid control still sends exactly one `C2SPlaceBid { amount }` and does not show a confirmation modal.
- [x] While a bid is in flight, the clicked bid control reads `BIDDING...`, the other bid controls are non-interactive, and no additional bid can be sent until the authoritative accepted or rejected response resolves the state.
- [x] Existing SAU-005 bid behavior tests remain green.
- [x] Existing SAU-006 accepted/rejected feedback tests remain green.
- [x] Browser/WASM evidence records target bounds and focus bounds for +1, +3, and +5, plus screenshot observations for affordable, unaffordable, `BIDDING...`, and `YOU ARE LEADING` states.
- [x] The evidence document includes a QA-COND-0005 impact statement saying A11Y-ST-12 is implemented and evidenced by this story, but QA-COND-0005 remains Open until all remaining Standard-tier rows are implemented, evidenced, reclassified, or accepted as risk.
- [x] `git diff --check` passes.

---

## Implementation Notes

- Prefer increasing the Bevy UI `Node` hit area or padding for the bid controls over adding invisible overlapping hit boxes. If an equivalent target area is used, give it a stable marker component and test its measured bounds directly.
- Keep target-size changes local to the auction bid area. Do not alter shop slot, refresh, ready, HUD, hand tray, or board layout unless a direct overlap regression requires a minimal local correction.
- Focus visibility can be implemented as a focus-specific component, outline node, border color, or equivalent high-contrast state, as long as tests can observe it and browser evidence can see it.
- Disabled bid controls should not be focusable. Hidden leading-state controls must be removed from the focus traversal model rather than visually hidden while remaining keyboard-reachable.
- Keep `BIDDING...` text in the same target area as the clicked bid control. Do not move feedback into a separate confirmation surface.
- If the implementation exposes browser/WASM bounds through a debug overlay, measurement log, test marker, or evidence harness, keep that instrumentation test-only or evidence-only and avoid shipping persistent debug visuals.

## Performance Budget

No gameplay-loop performance impact expected. This story changes a fixed three-control auction bid cluster plus one leader badge. Steady-state bid UI updates must remain O(1), with no per-frame tree rebuild, catalog scan, or allocation-heavy focus traversal. Presentation steady-state must remain under 1 ms per frame and phase-boundary/focus-state spikes must stay under the ADR-021 3 ms guardrail.

---

## QA Test Cases

- **Target size**
  - Given: DRAFT_AUCTION renders +1, +3, and +5 bid controls at 100 percent UI scale
  - When: target bounds are queried from the test-observable marker components
  - Then: each interactive target is at least 44 CSS px high and 44 CSS px wide

- **Focus order and focus ring**
  - Given: all three bid controls are visible and enabled
  - When: keyboard focus advances through the bid area
  - Then: focus order is +1, +3, +5 and each focused control exposes a visible focus-ring state

- **Disabled and hidden focus**
  - Given: +1 is affordable and +3/+5 are unaffordable
  - When: keyboard focus traverses the bid area
  - Then: +1 is reachable and +3/+5 are skipped
  - Edge case: when the local player is leader, +1/+3/+5 are hidden and no bid-area item is focusable

- **State readability**
  - Given: auction bid controls are rendered in affordable, unaffordable, in-flight, and local leader states
  - When: browser/WASM evidence is captured
  - Then: all visible state text is inside its target or badge bounds, with no clipping or sibling overlap

- **Behavior preservation**
  - Given: an enabled +3 bid control is activated by click or Enter
  - When: the input handler completes
  - Then: exactly one bid intent is sent, +3 reads `BIDDING...`, +1/+5 are non-interactive, and SAU-006 accepted/rejected responses still resolve the state as before

---

## Test Evidence

**Story Type**: UI

**Required automated test targets**:

- `tests/integration/shop_auction_ui/auction_bid_target_focus_test.rs`
  - Registered as `shop_auction_ui_auction_bid_target_focus_test`
  - Command: `cargo test -p client --test shop_auction_ui_auction_bid_target_focus_test`
- Regression: `cargo test -p client --test shop_auction_ui_auction_bid_buttons_test`
- Regression: `cargo test -p client --test shop_auction_ui_auction_feedback_test`
- Regression: `cargo test -p client --test shop_auction_ui_auction_activation_test`
- Compile check: `cargo check -p client`
- Whitespace check: `git diff --check`

**Required browser/WASM evidence document**:

- `production/qa/evidence/shop-auction-ui-auction-bid-target-focus-2026-05-05.md`

**Required browser/WASM capture contents**:

- 100 percent UI scale target-bound measurements for +1, +3, and +5.
- Focus-bound measurements for +1, +3, and +5.
- Screenshot evidence for enabled affordable buttons with visible `(+1)`, `(+3)`, and `(+5)` labels.
- Screenshot evidence for unaffordable disabled bid buttons skipped by keyboard focus.
- Screenshot evidence for clicked-button `BIDDING...` with other bid controls non-interactive.
- Screenshot evidence for local `YOU ARE LEADING` replacement with no focusable bid-area control.
- A note confirming whether visual button art was enlarged to 44px high or an equivalent 44x44 target area was accepted and measured.

**QA-COND-0005 impact statement required in evidence**:

Story 011 implements and evidences A11Y-ST-12 for auction bid target size, focus visibility, immediate preset commitment labels, affordability gating, in-flight disable, one-send semantics, and `BIDDING...` feedback. It does not close QA-COND-0005 by itself. QA-COND-0005 remains Open until all remaining Standard-tier rows are implemented and evidenced, reclassified, or accepted as risk.

**Status**: [x] Created and verified

---

## Dependencies

- Depends on: [Story 004](story-004-auction-panel-activation-and-preparing-state.md) - Complete; provides active auction panel, timer state, and button area ownership.
- Depends on: [Story 005](story-005-auction-bid-buttons-affordability-and-inflight.md) - Complete; provides immediate preset bid buttons, affordability, in-flight state, and exact one-send behavior that this story must preserve.
- Depends on: [Story 006](story-006-auction-accepted-rejected-feedback.md) - Complete; provides accepted/rejected response behavior, local gold re-enable gate, and rejection feedback that this story must preserve.
- Depends on: `design/ux/shop-auction-ui.md` and `design/ux/interaction-patterns.md` for final bid target, focus, and immediate preset commitment requirements.
- Depends on: ADR-013, ADR-019, and ADR-021 Accepted.
- Unlocks: A11Y-ST-12 evidence contribution for QA-COND-0005. Does not unlock QA-COND-0005 closure by itself.

## Blockers

None.

## Completion Notes

**Completed**: 2026-05-06
**Criteria**: 16/16 passing.
**Deviations**: None blocking. The visible bid button target itself is 108x44 CSS px in browser/WASM evidence, so no invisible equivalent hit box is used.
**Test Evidence**: `production/qa/evidence/shop-auction-ui-auction-bid-target-focus-2026-05-05.md`; Browser/WASM capture artifacts under `production/qa/evidence/captures/shop-auction-ui-auction-bid-target-focus/`; `cargo test -p client --test shop_auction_ui_auction_bid_target_focus_test --jobs 1` passed 4/4; requested SAU-004/005/006 regression group passed 17/17; `cargo test -p client --test shop_auction_ui_draft_initial_objective_overlay_test --jobs 1` passed 8/8; `cargo fmt -p client -- --check`, `cargo check -p client --jobs 1`, and `git diff --check` passed.
**Code Review**: Skipped by lean review mode because `production/review-mode.txt` is absent.
**QA-COND-0005**: A11Y-ST-12 is implemented and evidenced by SAU-011. QA-COND-0005 remains Open until all remaining Standard-tier rows are implemented and evidenced, reclassified, or accepted as risk.
