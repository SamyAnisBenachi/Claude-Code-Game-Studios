# Story 012: Draft Initial Clear Objective Overlay

> **Epic**: Shop / Auction UI
> **Status**: Ready
> **Layer**: Presentation
> **Type**: UI
> **Manifest Version**: 2026-05-05

## Context

**Sprint Gate**: Sprint 6 S6-04 / QA-COND-0005 Standard-tier accessibility remediation.

**QA condition**: `production/qa/bugs/QA-COND-0005-standard-tier-accessibility-gaps.md` remains Open. The Sprint 6 accessibility disposition row `A11Y-ST-18` requires a clear DRAFT_INITIAL objective overlay with exact copy, dismissal behavior, in-phase retrieval, and browser/WASM evidence.

**GDD**: `design/gdd/shop-auction-ui.md`
**UX Spec**: `design/ux/shop-auction-ui.md`
**Accessibility Source**: `design/accessibility-requirements.md`
**Requirement**: `TR-SAU-006`, A11Y-ST-18
**ADR Governing Implementation**: [ADR-015: Card Acquisition Shop State](../../../docs/architecture/adr-015-card-acquisition-shop-state.md), [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)
**Control Manifest**: `docs/architecture/control-manifest.md` version `2026-05-05`.

**GDD trace**:

- `design/gdd/shop-auction-ui.md` DRAFT_INITIAL Rule 1 requires activation only after both `S2CPhaseChanged(DRAFT_INITIAL)` and `S2CDraftOffering` are available, initializes the 45-second timer from phase data, and keeps the panel blank before both inputs arrive.
- DRAFT_INITIAL Rule 2 requires the 3 x 3 offering grid sorted by rarity and cost.
- DRAFT_INITIAL Rule 3 and Rule 4 require purchase click behavior, insufficient-gold handling, confirmed purchased-slot state, and no optimistic ownership.
- DRAFT_INITIAL Rule 7 requires Ready and Retract Ready behavior while the grid remains interactive until phase transition.
- `design/gdd/shop-auction-ui.md` acceptance criteria `SAU-DI1`, `SAU-DI2`, `SAU-DI3`, `SAU-DI6`, `SAU-DI7`, `SAU-DI8`, `SAU-DI9`, `SAU-DI10`, and `SAU-DI11` are the existing behavior baseline this story must not regress.

**UX and accessibility trace**:

- `design/accessibility-requirements.md` A11Y-ST-18 requires that at session start DRAFT_INITIAL confirms: `Select up to 9 cards to keep. You have 45 seconds.` The objective must be dismissible but retrievable.
- `design/ux/shop-auction-ui.md` defines DRAFT_INITIAL as the first game economy decision and requires keyboard-reachable controls, visible focus indicators, non-occluding panel-scoped overlays, and browser/WASM evidence for final accessibility work.
- This story is a narrow cognitive-support remediation slice. It does not implement the full Help/tutorial registry from Settings / Accessibility.

**Engine**: Bevy 0.18 + Lightyear 0.26 + WASM browser evidence | **Risk**: HIGH

**Required skills**: use `liv-bevy-018` before editing any Bevy `.rs` file and `liv-bevy-lightyear` before editing any Lightyear/networking `.rs` file.

**ADR/control-manifest rules for this story**:

- `ShopAuctionUiPlugin` remains the fifth `PresentationPlugin` sub-plugin after Card Animations, Board Rendering, Hand UI, and HUD.
- Run UI work in the ADR-021 `PresentationSet` order: `PhaseTransition -> MessageDrain -> StateSync -> AnimationTick`.
- Do not drain `MessageReceiver<S2CPhaseChanged>` in Shop/Auction UI. Read `Res<CurrentClientPhase>` populated by `phase_sink_system`.
- Do not drain `MessageReceiver<S2CGoldUpdate>` in Shop/Auction UI. Read `Res<PlayerEconomyView>` for local own-gold state.
- Send `C2SPurchaseCard` and `C2SSignalReady` only as player intent. Never mutate authoritative purchase, gold, hand, ready, or phase state from overlay interaction.
- Use Bevy 0.18 Required Components API for all UI entities: `Node`, `Text`, `TextFont`, `TextColor`, `Button`, `Interaction`, `ImageNode`, and `ChildOf` where needed. Do not use `NodeBundle`, `TextBundle`, `UiImage::new()`, `Parent`, `commands.entity(e).set_parent(...)`, or `Color::rgba()`.
- Preserve pre-pooled DRAFT_INITIAL panel, grid, timer, and Ready entities. Do not rebuild the grid per frame.

---

## Scope

### In Scope

- Add a DRAFT_INITIAL objective overlay that appears when DRAFT_INITIAL becomes active after both phase and draft offering data are available.
- The overlay body copy is exactly: `Select up to 9 cards to keep. You have 45 seconds.`
- The overlay must not cover the top HUD, HUD gold counters, hand tray, or any card slot in a way that prevents reading the 3 x 3 offering.
- Dismissal behavior:
  - Explicit dismiss button.
  - Esc dismisses while the overlay has focus.
  - Click outside the overlay but inside the DRAFT_INITIAL panel dismisses if the click does not target a card slot, Ready, Retract Ready, timer, or retrieval control.
- In-phase retrieval:
  - Add a stable DRAFT_INITIAL objective retrieval affordance inside the active panel, near the header or timer zone.
  - The retrieval affordance is visible after dismissal and reopens the same overlay with the exact copy above.
  - The retrieval affordance remains available only while DRAFT_INITIAL is active.
- Keyboard and focus behavior:
  - Initial focus may remain on the first actionable DRAFT_INITIAL control or move to the overlay dismiss control, but the focus order must be deterministic and test-documented.
  - The overlay dismiss control and retrieval affordance must be keyboard-reachable and activatable with Enter.
  - Disabled or hidden controls remain absent from focus traversal.
- Browser/WASM evidence showing overlay presence, exact copy, dismissal, retrieval, focus visibility, and non-occlusion at the supported evidence viewport.
- Regression coverage proving existing DRAFT_INITIAL grid, purchase, Ready, Retract Ready, and PLACEMENT dismissal behavior still works.

### Out of Scope

- Do not implement the full Settings Help tab, tutorial prompt registry, replay library, reset-all tutorial persistence, or per-prompt persistence system.
- Do not change the existing first-session tooltip copy or its persistence key.
- Do not change DRAFT_INITIAL offering generation, sort order, purchase validation, hand cap, gold display, Ready/Retract Ready messages, or phase transition behavior.
- Do not add a new C2S message, server state, or protocol field for the overlay.
- Do not close QA-COND-0005 from this story alone.
- Do not modify sprint status, session-state files, the project asset directory, or `AGENTS.md`.

---

## Acceptance Criteria

- [ ] On DRAFT_INITIAL activation, after both `S2CPhaseChanged(DRAFT_INITIAL)` and `S2CDraftOffering` are available, the objective overlay appears in the same active panel state as the 3 x 3 offering.
- [ ] The visible overlay body copy is exactly `Select up to 9 cards to keep. You have 45 seconds.`
- [ ] The overlay appears only during DRAFT_INITIAL and is hidden or cleared on `S2CPhaseChanged(PLACEMENT)` and on any non-DRAFT_INITIAL phase.
- [ ] The overlay does not render before the DRAFT_INITIAL panel is active and does not activate from `S2CDraftOffering` alone.
- [ ] Dismissing through the explicit dismiss button hides the overlay without hiding the DRAFT_INITIAL grid, timer, Ready/Retract Ready, purchased-slot state, or gold affordances.
- [ ] Pressing Esc while overlay focus is active hides the overlay and returns focus to a deterministic DRAFT_INITIAL control.
- [ ] Clicking outside the overlay but inside non-actionable DRAFT_INITIAL panel space hides the overlay and does not send `C2SPurchaseCard` or `C2SSignalReady`.
- [ ] Clicking a card slot, Ready, Retract Ready, timer, or retrieval affordance does not count as outside-dismiss and preserves that control's existing behavior.
- [ ] After dismissal, the in-phase retrieval affordance remains visible, keyboard-reachable, and reopens the same overlay while DRAFT_INITIAL is active.
- [ ] The retrieval affordance is removed from focus traversal or hidden when DRAFT_INITIAL exits.
- [ ] Overlay dismissal and retrieval are local presentation state only and do not emit C2S messages.
- [ ] Existing DRAFT_INITIAL card purchase behavior remains unchanged: valid affordable slot clicks still send exactly one `C2SPurchaseCard { card_id }`, insufficient gold still does not send, and confirmed purchases still show the existing bought state.
- [ ] Existing DRAFT_INITIAL Ready/Retract Ready behavior remains unchanged: Ready sends `C2SSignalReady { retract: false }`, Retract Ready sends `C2SSignalReady { retract: true }`, and the grid remains interactive while ready.
- [ ] Existing DRAFT_INITIAL PLACEMENT dismissal behavior remains unchanged: `S2CPhaseChanged(PLACEMENT)` dismisses the panel and blocks further purchase sends.
- [ ] Browser/WASM evidence shows overlay presence, exact copy, dismiss button focus, Esc dismissal, retrieval affordance, reopened overlay, and no overlap with required grid, timer, Ready, HUD, or hand surfaces.
- [ ] The evidence document includes a QA-COND-0005 impact statement saying A11Y-ST-18 is implemented and evidenced by this story, but QA-COND-0005 remains Open until all remaining Standard-tier rows are implemented, evidenced, reclassified, or accepted as risk.
- [ ] `git diff --check` passes.

---

## Implementation Notes

- Treat the objective overlay as a DRAFT_INITIAL panel-scoped overlay, not as a global modal. It should never block HUD readability or hand tray visibility.
- Reuse the existing Shop/Auction UI panel state resource if it already owns local DRAFT_INITIAL presentation state. If a new state field is needed, keep it local to DRAFT_INITIAL overlay visibility and retrieval.
- Keep the overlay copy in one named constant or one test-observable source so exact-copy tests do not duplicate a second untracked string.
- The retrieval affordance can be a text button, icon-plus-text control, or project-standard help control, but tests must be able to identify it through a stable marker component.
- If the existing first-session tooltip and this objective overlay are both eligible on first DRAFT_INITIAL activation, the objective overlay has priority. The first-session tooltip may appear after the objective overlay is dismissed only if it remains non-occluding and the existing UX contract allows it.
- Do not persist this objective overlay dismissal across sessions in this story. The required behavior is per DRAFT_INITIAL phase: appears at session start, dismisses locally, and remains retrievable during that phase.

## Performance Budget

No gameplay-loop performance impact expected. This story adds one bounded DRAFT_INITIAL overlay and one retrieval affordance. Steady-state UI updates must remain O(1), with no per-frame grid rebuild, no card catalog scan, and no allocation-heavy focus traversal. Presentation steady-state must remain under 1 ms per frame and phase-boundary overlay toggles must stay under the ADR-021 3 ms guardrail.

---

## QA Test Cases

- **Overlay activation and exact copy**
  - Given: `S2CDraftOffering` arrives before `S2CPhaseChanged(DRAFT_INITIAL)`
  - When: DRAFT_INITIAL phase data later arrives and the panel activates
  - Then: the objective overlay is visible and its body text exactly matches `Select up to 9 cards to keep. You have 45 seconds.`

- **Dismissal**
  - Given: DRAFT_INITIAL is active and the objective overlay is visible
  - When: the dismiss button is activated
  - Then: the overlay is hidden, no C2S message is sent, and the grid, timer, and Ready control remain visible
  - Edge case: Esc dismisses when overlay focus is active

- **Retrieval**
  - Given: the objective overlay has been dismissed during DRAFT_INITIAL
  - When: the retrieval affordance is activated by keyboard or click
  - Then: the same overlay reopens with the exact copy and the dismiss path still works

- **Outside click safety**
  - Given: the objective overlay is visible
  - When: a non-actionable panel background area is clicked
  - Then: the overlay is dismissed and no purchase or ready message is sent
  - Edge case: clicking a card slot or Ready control follows that control's existing behavior and is not interpreted as outside-dismiss

- **DRAFT_INITIAL regression**
  - Given: overlay presence, dismissal, and retrieval have each occurred in the current DRAFT_INITIAL phase
  - When: purchase, insufficient-gold, Ready, Retract Ready, and `S2CPhaseChanged(PLACEMENT)` paths are exercised
  - Then: the existing Story 002 behavior remains unchanged

---

## Test Evidence

**Story Type**: UI

**Required automated test targets**:

- `tests/integration/shop_auction_ui/draft_initial_objective_overlay_test.rs`
  - Registered as `shop_auction_ui_draft_initial_objective_overlay_test`
  - Command: `cargo test -p client --test shop_auction_ui_draft_initial_objective_overlay_test`
- Regression: `cargo test -p client --test shop_auction_ui_draft_initial_grid_test`
- Regression: `cargo test -p client --test shop_auction_ui_plugin_scaffold_formulas_test`
- Compile check: `cargo check -p client`
- Whitespace check: `git diff --check`

**Required browser/WASM evidence document**:

- `production/qa/evidence/shop-auction-ui-draft-initial-clear-objective-overlay-2026-05-05.md`

**Required browser/WASM capture contents**:

- Overlay visible on DRAFT_INITIAL activation with exact copy.
- Dismiss button visible and focused.
- Esc dismissal result.
- Retrieval affordance visible after dismissal.
- Reopened overlay from retrieval affordance.
- Non-occlusion observation for the 3 x 3 grid, timer, Ready/Retract Ready, HUD gold counters, and hand tray.
- A QA-COND-0005 impact statement for A11Y-ST-18.

**QA-COND-0005 impact statement required in evidence**:

Story 012 implements and evidences A11Y-ST-18 for DRAFT_INITIAL clear objective copy, dismissal, retrieval, and browser/WASM readability. It does not close QA-COND-0005 by itself. QA-COND-0005 remains Open until all remaining Standard-tier rows are implemented and evidenced, reclassified, or accepted as risk.

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: [Story 001](story-001-plugin-scaffold-panel-tree-and-formulas.md) - Complete; provides `ShopAuctionUiPlugin`, panel roots, and shared formula scaffolding.
- Depends on: [Story 002](story-002-draft-initial-grid-purchase-ready.md) - Complete; provides active DRAFT_INITIAL grid, purchase, Ready/Retract Ready, and PLACEMENT dismissal behavior that this story must preserve.
- Depends on: `design/ux/shop-auction-ui.md` for DRAFT_INITIAL layout, focus, and non-occlusion requirements.
- Depends on: ADR-015 and ADR-021 Accepted.
- Unlocks: A11Y-ST-18 evidence contribution for QA-COND-0005. Does not unlock QA-COND-0005 closure by itself.

## Blockers

None.
