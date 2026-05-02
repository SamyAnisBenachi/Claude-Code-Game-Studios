# HUD-006 Economy Auction Inline Gold Evidence

Status: Automated implementation evidence captured; manual visual walkthrough pending playable auction UI integration.

## Automated Evidence

- HUD-17: Covered by `hud_economy_auction_inline_gold_test::draft_auction_entry_formats_both_gold_labels_inline_reserved_zero`.
- HUD-08: Covered by `hud_economy_auction_inline_gold_test::opponent_gold_broadcast_rendering_adapts_to_hud_mode`.
- HUD-29: Covered by `hud_economy_auction_inline_gold_test::auction_exit_clears_reserved_spans_without_despawning_them`.
- HUD-28: Covered by `hud_economy_auction_inline_gold_test::opponent_gold_broadcast_rendering_adapts_to_hud_mode`.
- Server-invariant display guard: Covered by `hud_economy_auction_inline_gold_test::reserved_gold_display_clamps_to_total_gold`.

## Verification Run

- `cargo fmt -p client -- --check` passed.
- `cargo test -p client --test hud_economy_auction_inline_gold_test` passed 4/4.
- `cargo test -p client --test hud_plugin_scaffold_test --test hud_gold_mana_display_test --test hud_phase_label_round_counter_test --test hud_phase_transitions_test --test scoreboard_dot_message_test --test hud_economy_auction_inline_gold_test` passed 29/29.
- `cargo check -p client` passed.

## Manual Walkthrough Pending

- Manual visual check: Confirm the top-right HUD shows `Xg (Yr)` inline during a live DRAFT_AUCTION and reverts to `Xg` after DRAFT_SHOP.
- Blocker: Full Shop/Auction UI and live playable auction flow are not implemented in this branch.
- Required capture later: screenshot or clip of both gold labels during DRAFT_AUCTION and after transition back to DRAFT_SHOP.

Lead sign-off: Pending manual walkthrough.
