// Story 002: Intent-Named API & Consumption Invariants
//
// Evidence file for /story-done review. The runnable tests live as a
// #[cfg(test)] module inside server/src/foundation/rng.rs and are executed
// by `cargo test -p server`. This file documents the test cases and their
// requirements mapping.

// RNG2: Two ServerRng with different seeds → different first output
//   test_rng2_different_seeds_produce_different_first_output
//   Given: ServerRng::from_seed(1) and ServerRng::from_seed(2)
//   When:  both call resolve_ecaflip(0)
//   Then:  returned u64 values differ

// RNG6: Empty-pool draw still increments seed_index
//   test_rng6_draw_always_increments_seed_index
//   Given: ServerRng::from_seed(99), current_seed_index() == 1
//   When:  resolve_ecaflip(0) is called
//   Then:  current_seed_index() == 2; audit_log().last().result == None

// RNG7: Two Ecaflip triggers → consecutive seed_index entries
//   test_rng7_consecutive_ecaflip_calls_have_sequential_seed_indices
//   Given: ServerRng::from_seed(7), current_seed_index() == 1
//   When:  resolve_ecaflip(1) called twice
//   Then:  audit_log()[1].seed_index == 1, audit_log()[2].seed_index == 2,
//          both event_type == ResolveEcaflip { lane: 1 }

// RNG12: assign_fake_objectives produces 2 audit entries per call
//   test_rng12_assign_fake_objectives_produces_two_entries
//   Given: ServerRng::from_seed(3)
//   When:  assign_fake_objectives(1) is called
//   Then:  audit_log().len() == 3 (sentinel + 2 entries),
//          both entries have event_type = AssignFakeObjectives { player_id: 1 },
//          seed_index 1 and 2 respectively

// RNG12: Ordering contract — ascending lane order produces deterministic audit log
//   test_rng12_ascending_lane_order_produces_ordered_audit_entries
//   Given: ServerRng::from_seed(42)
//   When:  resolve_ecaflip called for lanes 1, 2, 3 in ascending order
//   Then:  audit_log()[1..=3] records lanes 1, 2, 3 with seed_index 1, 2, 3

// All 7 intent-named methods exist and push correct audit entries
//   test_all_seven_methods_push_one_entry_each
//   Verifies assign_fake_objectives (+2), draw_initial_draft (+1), draw_shop_slot (+1),
//   resolve_ecaflip (+1), resolve_prism (+1), award_fake_objective_reward (+1),
//   draw_free_card (+1) — total 8 entries + sentinel = 9; seed_index == 9
