// Story 003: Determinism Proof & Session Reset
//
// Evidence file for /story-done review. The runnable tests live as a
// #[cfg(test)] module inside server/src/foundation/rng.rs and are executed
// by `cargo test -p server`. This file documents the test cases and their
// requirements mapping.
//
// Implements: ADR-005 §2 Lifecycle, §4 Consumption Order, VC1 + VC2
// GDD: design/gdd/server-rng.md — TR-RNG-04, RNG13, RNG15

// ---------------------------------------------------------------------------
// Determinism (ADR-005 VC1 + VC2)
// ---------------------------------------------------------------------------

// test_determinism_same_seed_produces_identical_audit_log
//   Given: Two ServerRng::from_seed(0xDEAD_BEEF_CAFE_1234) instances
//   When:  The same scripted call sequence (all 7 intent-named methods in
//          ADR-005 §4 order) is applied independently to each
//   Then:  Both audit_log() slices have identical length and all
//          (seed_index, event_type) pairs match structurally

// test_determinism_same_seed_same_process_run_twice
//   Given: The scripted session function run twice in the same test process
//   When:  Both use the same fixed seed
//   Then:  Identical audit_log structures — guards against state leakage
//          between test runs

// ---------------------------------------------------------------------------
// Session reset (RNG13)
// ---------------------------------------------------------------------------

// test_session_reset_new_instance_starts_clean
//   Given: Session A ServerRng::from_seed(1) after multiple calls
//          (seed_index advances to 3)
//   When:  Session B ServerRng::from_seed(1) is created independently
//   Then:  B.current_seed_index() == 1 (reset to post-sentinel, not continuing
//          from A); B.audit_log().len() == 1 (sentinel only);
//          B.audit_log()[0] is SessionInit

// test_session_reset_seed_index_not_inherited_from_prior_session
//   Given: Session A with seed_index advanced past 1
//   When:  Session B is created fresh
//   Then:  B's first non-sentinel entry records seed_index == 1, not
//          A's final seed_index + 1 — sessions are fully independent

// ---------------------------------------------------------------------------
// Overflow (RNG15 — ADVISORY)
// ---------------------------------------------------------------------------

// test_overflow_does_not_panic
//   Given: ServerRng::at_max_seed_index() — seed_index set to u32::MAX
//   When:  resolve_ecaflip(0) is called (any intent-named method)
//   Then:  No panic; the call completes normally

// test_overflow_wraps_seed_index_to_zero
//   Given: ServerRng::at_max_seed_index()
//   When:  Any intent-named method is called
//   Then:  current_seed_index() == 0 (wrapping_add behaviour)

// test_overflow_audit_entry_records_max_seed_index
//   Given: ServerRng::at_max_seed_index()
//   When:  resolve_ecaflip(0) is called
//   Then:  The new audit entry has seed_index == u32::MAX (the value AT
//          time of call, before wrap); the log has one more entry than before
