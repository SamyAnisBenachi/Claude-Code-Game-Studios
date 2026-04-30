// Story 002: Card Pool Weighted Draw Functions
//
// Evidence file for /story-done review. Runnable tests live in the
// #[cfg(test)] module inside server/src/core/pool/api.rs and run with:
// `cargo test -p server`.

// AC-1 / CP7:
//   test_draw_class_card_all_exhausted_returns_none
//   Exhausted Iop class subset returns None for 100 distinct seeds.

// AC-2 / CP-SHC:
//   test_draw_class_card_returns_correct_class
//   Class draw returns only available cards whose catalog class is Iop.

// AC-3 / CP-SHN:
//   test_draw_neutral_family_then_draw_family_card
//   Neutral family draw returns Gobball, then family card draw returns an
//   available Neutral Gobball card.

// AC-4 / CP-NW:
//   test_normalized_weights_sum_to_one
//   Formula 2 normalized weights sum to 1.0 within 1e-6.

// AC-5 / CP-A:
//   test_draw_auction_card_exhausted_returns_none
//   Auction draw returns None when Neutral Rare and Legendary cards are
//   exhausted; Common, Epic, and non-Neutral cards are ignored.

// AC-6 / CP-B:
//   test_draw_random_exhausted_filter_returns_none
//   Exhausted filter returns None and does not mutate copies_remaining.

// AC-7 / CP-C:
//   test_draw_initial_draft_9_distinct_ids
//   Initial draft returns 9 distinct IDs.

// AC-8 / CP-C2:
//   test_draw_initial_draft_class_and_neutral_only
//   Initial draft excludes other classes.

// AC-9 / CP-C3:
//   test_draw_initial_draft_does_not_call_distribute
//   Initial draft leaves copy counts unchanged.

// AC-10 / CP9:
//   test_formula2_raw_weight_at_3_owned
//   25 eligible types, target acquired 3: raw = 0.34 and normalized ~= 0.2615.

// AC-11 / CP10:
//   test_formula2_weight_clamped_at_cap
//   25 eligible types, target acquired 7: raw clamps to shop_weight_cap 0.65.
