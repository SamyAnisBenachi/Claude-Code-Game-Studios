// Story 001: ServerRng Type Definitions & Audit Infrastructure
//
// Evidence file for /story-done review. The runnable tests live as a
// #[cfg(test)] module inside server/src/foundation/rng.rs and are executed
// by `cargo test -p server`. This file documents the test cases and their
// requirements mapping.
//
// Tests updated in Story 002 to use intent-named API (next_seed was refactored
// to private with no parameters; callers use resolve_ecaflip, draw_free_card, etc.)

// RNG1: After ServerRng::new(), current_seed_index() == 1
//   test_new_seed_index_is_one

// RNG5: 0 gameplay calls → 1 audit entry (sentinel only)
//   test_zero_calls_has_one_audit_entry

// RNG5: N gameplay calls → N+1 audit entries
//   test_n_calls_produces_n_plus_one_audit_entries
//   Uses: resolve_ecaflip(0), resolve_ecaflip(1), resolve_ecaflip(2)

// RNG11: audit_log()[0] is SessionInit with result = None
//   test_sentinel_is_session_init_with_no_result

// RNG11: no raw seed bytes appear in any AuditEntry.result
//   test_no_raw_seed_in_audit_log

// seed_index values are monotonically ordered 0..N
//   test_audit_log_seed_indices_are_sequential
//   Uses: draw_free_card(1), draw_free_card(2)
