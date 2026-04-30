// Story S2-02 evidence mapping.
//
// Executable tests live in server/src/core/economy/api.rs so they compile with
// the server crate without requiring a separate lib target. Run:
//   cargo test -p server economy::api::tests
//
// Coverage:
// - EC1: test_auto_split_draws_current_first_then_reserve
// - EC2: test_auto_split_with_no_current_draws_from_reserve
// - EC3: test_auto_split_exact_current_cost_leaves_reserve_untouched
// - EC4: test_validate_spend_rejects_insufficient_total_without_mutation
// - EC5: test_reserve_only_rejects_when_reserve_insufficient
// - EC7: test_add_reserve_then_discard_current_transfers_gelure_contract
// - EC8: test_zero_add_reserve_then_discard_current_is_legal_noop
// - EC9: test_increment_mana_cap_increases_by_one_below_ceiling
// - EC10: test_increment_mana_cap_clamps_at_config_ceiling
// - EC11 pure API half: test_apply_gold_award_zero_is_noop
// - Zero-cost card: test_zero_cost_spend_is_noop
// - reserve_gold overflow: test_reserve_gold_rejects_when_unreserved_gold_insufficient
// - release_gold_reservation clamp: test_release_gold_reservation_clamps_to_zero
