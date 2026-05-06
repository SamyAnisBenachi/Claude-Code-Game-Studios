# Shop/Auction UI DRAFT_INITIAL Clear Objective Overlay Evidence

| Field | Value |
|---|---|
| Story | `production/epics/shop-auction-ui/story-012-draft-initial-clear-objective-overlay.md` |
| QA condition | `production/qa/bugs/QA-COND-0005-standard-tier-accessibility-gaps.md` |
| Row | A11Y-ST-18 |
| Evidence date | 2026-05-06 |
| Verdict | PASS - implementation, automated UI evidence, and Browser/WASM capture evidence complete |
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

## Browser/WASM Capture Evidence

Capture command:

```text
trunk serve shop-auction-draft-initial-objective-overlay-harness.html --release --port 8082 --address 127.0.0.1 --no-autoreload true --no-error-reporting true
powershell -NoProfile -ExecutionPolicy Bypass -File production/qa/evidence/captures/shop-auction-ui-draft-initial-clear-objective-overlay/capture.ps1 -Url http://127.0.0.1:8082/shop-auction-draft-initial-objective-overlay-harness.html -ReadyTimeoutSeconds 240
```

Capture summary:

- Summary JSON: `production/qa/evidence/captures/shop-auction-ui-draft-initial-clear-objective-overlay/capture-summary.json`
- Capture tool: PowerShell Chrome DevTools Protocol
- Browser: Chrome `147.0.7727.139`
- Captured at: `2026-05-06T13:53:02.9763719Z`
- Viewport: 1366x768, device scale factor 1, UI scale 1.0

Artifact set:

- Entry overlay visible on DRAFT_INITIAL activation with exact copy and the
  dismiss control as the deterministic focus target:
  `production/qa/evidence/captures/shop-auction-ui-draft-initial-clear-objective-overlay/sau-012-entry-1366x768.png`
  and `production/qa/evidence/captures/shop-auction-ui-draft-initial-clear-objective-overlay/sau-012-entry-1366x768-report.json`
- Esc-dismissed state with overlay hidden, retrieval affordance visible, and
  zero overlay-originated `C2SPurchaseCard` / `C2SSignalReady` sends:
  `production/qa/evidence/captures/shop-auction-ui-draft-initial-clear-objective-overlay/sau-012-esc-dismissed-1366x768.png`
  and `production/qa/evidence/captures/shop-auction-ui-draft-initial-clear-objective-overlay/sau-012-esc-dismissed-1366x768-report.json`
- Retrieved state proving the retrieval affordance reopens the same overlay
  with the exact copy and still emits no C2S messages:
  `production/qa/evidence/captures/shop-auction-ui-draft-initial-clear-objective-overlay/sau-012-retrieved-1366x768.png`
  and `production/qa/evidence/captures/shop-auction-ui-draft-initial-clear-objective-overlay/sau-012-retrieved-1366x768-report.json`

Summary verdict:

- Screenshots nonblank: PASS.
- Exact copy: PASS.
- Overlay visible on entry: PASS.
- Dismiss control focused/visible: PASS.
- Esc dismissal without C2S sends: PASS.
- Retrieval visible after dismissal: PASS.
- Retrieval reopens same overlay: PASS.
- Grid, Ready, HUD gold, and hand surfaces non-occluded by the overlay: PASS.
- DRAFT_INITIAL phase exit behavior remains covered by
  `sau_012_overlay_and_retrieval_hide_on_placement_phase_exit`.

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
