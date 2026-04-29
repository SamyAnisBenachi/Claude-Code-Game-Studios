// Story 001: Pool State + Core API
//
// Evidence file for /story-done review. The runnable tests live as a
// #[cfg(test)] module inside server/src/core/pool/api.rs and are executed
// by `cargo test -p server`. This file documents the test cases and their
// requirements mapping.

// AC-1: initialize maps both copies_remaining and initial_count to catalog length
//   test_pool_initialize_catalog_length
//   test_pool_initialize_empty_catalog_no_panic
//   Given: CardCatalog with N cards
//   When:  PlayerPool::initialize(&catalog, &config) called
//   Then:  copies_remaining.len() == N, initial_count.len() == N

// AC-2: all cards have copies >= 1 after initialization
//   test_pool_initialize_all_cards_have_copies
//   Given: Catalog with one card of each rarity, all override=None
//   When:  initialize() with default config
//   Then:  Common=6, Uncommon=5, Rare=4, Epic=1, Legendary=1; all >= 1

// AC-3: pool_copies_override <= 0 is soft error — uses rarity default, no panic
//   test_pool_soft_error_override_zero_no_panic_uses_rarity_default
//   test_pool_soft_error_override_negative_no_panic_uses_rarity_default
//   test_pool_soft_error_override_i32_min_no_panic
//   Given: Rare card with Some(0) or Some(-3)
//   When:  initialize() called
//   Then:  copies_remaining == 4 (Rare default); no panic; tracing::error! emitted

// AC-4: Rare card with no override gets rarity default
//   test_pool_rare_no_override_gets_rarity_default
//   test_pool_rare_no_override_respects_config_value
//   Given: Rare card, override=None, config.rare_pool_copies=4
//   When:  initialize() called
//   Then:  copies_remaining == 4

// AC-5: distribute decrements copies_remaining by 1
//   test_pool_distribute_decrements_correctly
//   test_pool_distribute_four_times_sequence
//   Given: Rare card with 4 copies
//   When:  distribute(id) called once
//   Then:  Ok(()), copies_remaining == 3; sequence: 4→3→2→1→0

// AC-6: positive pool_copies_override is applied instead of rarity default
//   test_pool_positive_override_applied
//   test_pool_positive_override_one_minimum
//   Given: Rare card, Some(2)
//   When:  initialize() called
//   Then:  copies_remaining == 2 (not 4)

// AC-7: distribute on exhausted card returns Err(Exhausted), no underflow
//   test_pool_distribute_exhausted_error
//   test_pool_distribute_exhausted_repeatedly_stays_exhausted
//   Given: Card with copies_remaining == 0
//   When:  distribute(id) called
//   Then:  Err(DistributeError::Exhausted); copies_remaining still 0

// AC-8: is_available returns false at zero copies
//   test_pool_is_available_false_at_zero
//   test_pool_is_available_false_for_unknown_card
//   Given: copies_remaining == 0 (or unknown card_id)
//   When:  is_available(id)
//   Then:  false; no panic on unknown id

// AC-9: is_available returns true when copies >= 1
//   test_pool_is_available_true_above_zero
//   test_pool_is_available_true_at_one_copy_remaining
//   Given: copies_remaining >= 1
//   When:  is_available(id)
//   Then:  true

// AC-10: initial_count never mutated; total_acquired is derived correctly
//   test_pool_initial_count_immutable_total_acquired_correct
//   test_pool_total_acquired_zero_before_any_distribute
//   test_pool_total_acquired_equals_n_when_all_distributed
//   Given: initial_count == N; distribute K times (K < N)
//   When:  check initial_count, copies_remaining, total_acquired
//   Then:  initial_count unchanged; copies_remaining == N-K; total_acquired == K
