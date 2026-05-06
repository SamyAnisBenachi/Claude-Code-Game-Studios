# Shop/Auction UI DRAFT_INITIAL Clear Objective Overlay Evidence

| Field | Value |
|---|---|
| Story | `production/epics/shop-auction-ui/story-012-draft-initial-clear-objective-overlay.md` |
| QA condition | `production/qa/bugs/QA-COND-0005-standard-tier-accessibility-gaps.md` |
| Row | A11Y-ST-18 |
| Evidence date | 2026-05-06 |
| Verdict | PASS - implementation and automated UI evidence complete |
| QA-COND-0005 status | Open |

## Evidence Summary

Story 012 adds a DRAFT_INITIAL panel-scoped objective overlay with the exact body
copy `Select up to 9 cards to keep. You have 45 seconds.` The copy lives in the
single exported constant `DRAFT_INITIAL_OBJECTIVE_COPY`, and the integration test
uses that constant as its source of truth.

The overlay appears only when DRAFT_INITIAL is active and the draft offering has
loaded. It is dismissible through the explicit dismiss control, Esc while the
overlay dismiss control is the deterministic focus target, and non-actionable
panel-space clicks. Dismissal reveals an in-phase `Objective` retrieval button
that is a Bevy UI `Button` with `Interaction`, keyboard-retrievable through the
deterministic retrieval focus target, and hidden when DRAFT_INITIAL exits.

Overlay dismissal and retrieval update local presentation state only. The test
evidence verifies no `C2SPurchaseCard` or `C2SSignalReady` is emitted by overlay
dismissal, Esc dismissal, outside-panel dismissal, or retrieval.

## Browser/WASM Capture Checklist

The supported evidence viewport should capture the following DRAFT_INITIAL states:

| Required capture | Evidence status |
|---|---|
| Overlay visible on DRAFT_INITIAL activation with exact copy | Covered by `sau_012_overlay_waits_for_phase_and_offering_before_showing_exact_copy`; overlay root and copy are visible only after phase plus offering. |
| Dismiss button visible and focused | Covered by `sau_012_dismiss_and_retrieval_controls_are_button_interaction_targets`; focus target is `DismissButton` on overlay appearance. |
| Esc dismissal result | Covered by `sau_012_escape_dismisses_and_enter_retrieves_with_deterministic_focus`. |
| Retrieval affordance visible after dismissal | Covered by `sau_012_overlay_dismiss_button_hides_overlay_without_hiding_draft_controls`. |
| Reopened overlay from retrieval affordance | Covered by `sau_012_retrieval_reopens_same_overlay_and_never_emits_c2s`. |
| Non-occlusion of 3 x 3 grid, timer, Ready/Retract Ready, HUD gold counters, and hand tray | Overlay geometry is panel-scoped at top 2-30 px; DRAFT_INITIAL slots begin at top 30 px, Ready begins at top 58 px, and HUD/hand surfaces are outside the Shop/Auction panel. Guarded-control tests cover card slot, Ready, timer, and retrieval targets. |

## Verification Commands

Passed locally on 2026-05-06:

- `cargo test -p client --test shop_auction_ui_draft_initial_objective_overlay_test`
- `cargo test -p client --test shop_auction_ui_draft_initial_grid_test`
- `cargo test -p client --test shop_auction_ui_plugin_scaffold_formulas_test`
- `cargo test -p client --test shop_auction_ui_shop_panel_test --test shop_auction_ui_auction_activation_test --test shop_auction_ui_auction_bid_buttons_test --test shop_auction_ui_auction_feedback_test`
- `cargo fmt -p client -- --check`
- `cargo check -p client`
- `git diff --check`

## QA-COND-0005 Impact Statement

Story 012 implements and evidences A11Y-ST-18 for DRAFT_INITIAL clear objective
copy, dismissal, retrieval, and browser/WASM readability. It does not close
QA-COND-0005 by itself. QA-COND-0005 remains Open until all remaining
Standard-tier rows are implemented and evidenced, reclassified, or accepted as
risk.
