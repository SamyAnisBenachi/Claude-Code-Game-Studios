# Shop Auction UI Draft Grid Centered Modal Evidence - 2026-05-15

## No-Claim Banner

Story 015 implements DRAFT_INITIAL centered-modal layout only. It does
not advance `QA-COND-0005` (Standard-tier accessibility),
`QA-COND-0006` (playtest / fun-hypothesis validation), `PAW-TD-002-a`
/ `PAW-TD-003-a` (placeholder PNG accept-risk), `S8-QA-001-W1`
(two-client GAME_OVER closure), the PROMPT 761 Polish->Release
gate-check, or any release-readiness claim. All conditions remain
accept-risk / open per their existing dispositions.

## Automated Evidence

- `cargo test -p client --test shop_auction_ui_draft_initial_centered_modal_layout_test`
  - 4 passed.
  - Asserts the DRAFT_INITIAL centering root uses `Display::Flex`,
    `AlignItems::Center`, `JustifyContent::Center`, and the `MODAL`
    z-layer.
  - Asserts the modal content panel uses
    `width: Val::Percent(88.0)`, `max_width: Val::Px(860.0)`,
    `height: Val::Px(300.0)`, and `max_height: Val::Percent(92.0)`.
  - Asserts the DRAFT_INITIAL and DRAFT_SHOP roots are siblings and
    mutually exclusive by visibility.
  - Asserts the 3 x 3 grid has stable 120 px columns, 56 px rows, and
    16 px row / column gaps.
  - Asserts the objective overlay / retrieval affordance and Ready
    button do not overlap the grid band.

- Adjacent regressions:
  - `cargo test -p client --test shop_auction_ui_draft_initial_grid_test`
  - `cargo test -p client --test shop_auction_ui_draft_initial_objective_overlay_test`
  - `cargo test -p client --test shop_auction_ui_shop_panel_test`
  - `cargo test -p client --test shop_auction_ui_plugin_scaffold_formulas_test`

- Compile / formatting:
  - `cargo fmt --all -- --check`
  - `cargo check --workspace --all-targets`

## Visual Capture Limitation

Browser/WASM screenshots at 1920 x 1080 and 1366 x 768 were not captured
in this worker session. The implementation is covered by ECS layout
assertions and adjacent behavior regressions, but screenshot capture
still needs a follow-up pass in an environment with the local WASM
client and browser automation available.
