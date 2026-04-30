// Story 003: Card Pool refresh_shop + Slot Variants
//
// Evidence file for /story-done review. Runnable tests live in the
// #[cfg(test)] module inside server/src/core/pool/api.rs and run with:
// `cargo test -p server refresh_shop`
//
// AC-1:
//   test_refresh_shop_3_slots_full
//   Three-slot refresh returns 3 distinct cards and decrements each by 1.
//
// AC-2:
//   test_refresh_shop_9_slots_initial_draft
//   Nine-slot refresh returns 9 distinct cards and decrements each by 1.
//
// AC-3:
//   test_refresh_shop_partial_fill
//   Partial pool exhaustion returns a compact Vec with only successful draws.
//
// AC-3 edge:
//   test_refresh_shop_fully_exhausted_returns_empty
//   Fully exhausted pool returns an empty Vec and does not panic.
//
// AC-4:
//   test_manual_refresh_count_reset_on_draft_entry
//   Existing player counter resets to 0 at DRAFT entry.
//
// AC-4 edge:
//   test_manual_refresh_count_reset_inserts_missing_player
//   Missing player gets a stable 0 entry when reset.
//
// AC-4 edge:
//   test_manual_refresh_count_reset_only_targets_one_player
//   Resetting one player leaves other players' counters unchanged.
