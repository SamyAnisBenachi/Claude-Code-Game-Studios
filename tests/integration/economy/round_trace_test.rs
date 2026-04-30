// Story S2-08 integration evidence mapping.
//
// Executable integration trace lives in server/tests/economy_round_trace_test.rs.
// Run:
//   cargo test -p server economy_round_trace
//
// Coverage:
// - Round 1 -> 2 -> 3 mana trace: test_economy_round_trace_rounds_one_to_three
// - Gold evolves through baseline + interest at each DRAFT entry.
// - S2CGoldUpdate is enqueued once per player per DraftStarted event.
