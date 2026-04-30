// Story 003: Startup Validation Gate
//
// Evidence file for /story-done review. Runnable tests live in the
// #[cfg(test)] module inside server/src/foundation/config.rs and run with:
// `cargo test -p server game_config`.

// validate_game_config fatal invariants:
//   test_game_config_validation_default_passes
//   test_game_config_validation_rejects_shop_weight_cap_above_one
//   test_game_config_validation_rejects_shop_weight_cap_zero
//   test_game_config_validation_rejects_shop_weight_per_card_at_cap
//   test_game_config_validation_rejects_zero_common_pool_copies
//   test_game_config_validation_rejects_zero_uncommon_pool_copies
//   test_game_config_validation_rejects_zero_rare_pool_copies
//   test_game_config_validation_rejects_fake_count_zero_with_design_message
//   test_game_config_validation_rejects_fake_count_four
//   test_game_config_validation_rejects_objective_hp_zero
//   test_game_config_validation_rejects_placement_timer_zero
//   test_game_config_validation_rejects_auction_timer_zero
//   test_game_config_validation_rejects_auction_timer_reset_at_timer

// validate_card_catalog fatal invariants:
//   test_game_config_validation_card_catalog_valid_catalog_passes
//   test_game_config_validation_card_catalog_empty_catalog_fails
//   test_game_config_validation_card_catalog_key_mismatch_fails

// Soft-error placement:
//   test_game_config_validation_card_catalog_allows_non_positive_pool_override
//   Confirms catalog validation does not reject pool_copies_override <= 0;
//   PlayerPool::initialize owns the soft-error log + rarity-default fallback.

// validate_and_promote startup gate:
//   test_game_config_validation_promote_success_inserts_resources_and_enters_lobby
//   test_game_config_validation_promote_failure_writes_app_exit_and_does_not_promote
