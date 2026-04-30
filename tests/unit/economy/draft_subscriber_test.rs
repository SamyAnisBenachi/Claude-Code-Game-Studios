// Story S2-08 evidence mapping.
//
// Executable tests live in server/tests/economy_draft_subscriber_test.rs so
// they compile against the server crate. Run:
//   cargo test -p server economy_draft
//
// Coverage:
// - EC12: test_economy_draft_initialises_players_on_session_ready
// - EC6: test_economy_draft_preserves_reserve_and_applies_round_mana_ramp
// - EC13/EC15: test_economy_draft_applies_baseline_plus_interest_and_clears_snapshot
// - Round 1 guard: test_economy_draft_round_one_initial_adds_no_gold
// - Missing snapshot: test_economy_draft_missing_snapshot_adds_baseline_only
// - Message enqueue counts: test_economy_draft_writes_gold_update_and_broadcast_per_player
