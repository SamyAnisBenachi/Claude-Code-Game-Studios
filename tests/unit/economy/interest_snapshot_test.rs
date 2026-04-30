// Story S3-08 evidence mapping.
//
// Executable tests live in server/tests/economy_interest_snapshot_test.rs so
// they compile against the server crate. Run:
//   cargo test -p server --test economy_interest_snapshot_test
//
// Coverage:
// - EC13: test_resolution_snapshot_captures_gold_at_resolution_end
// - EC14: test_resolution_snapshot_gold_ten_yields_max_interest_next_draft
// - EC18: test_resolution_discards_current_mana
// - Snapshot overwrite: test_resolution_snapshot_overwrites_stale_value
// - Gold = 0: test_zero_gold_snapshot_gives_baseline_only_next_draft
// - Kill-reward threshold: test_kill_reward_cross_threshold_uses_post_award_gold
