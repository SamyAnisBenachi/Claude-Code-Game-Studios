// Story CS-001 evidence mapping.
//
// Executable tests live in server/tests/class_lifecycle_test.rs so they compile
// against the server crate. Run:
//   cargo test -p server --test class_lifecycle_test
//
// Coverage:
// - CS-AC-01a valid class choice accepted:
//   test_class_choice_unlocked_accepts_valid_class
// - CS-AC-01a Neutral choice discarded:
//   test_class_choice_rejects_neutral_class
// - CS-AC-01b locked class change discarded:
//   test_class_choice_locked_rejects_change
// - CS-AC-02a LOBBY gate rejects Neutral class:
//   test_lobby_gate_rejects_transition_when_any_class_is_neutral
// - CS-AC-02b LOBBY gate locks all non-Neutral classes:
//   test_lobby_gate_passes_and_locks_all_classes
// - CS-AC-02b locked classes reject later changes:
//   test_lobby_gate_lock_prevents_subsequent_class_change
// - CS-AC-02b debug invariant:
//   test_lock_all_classes_panics_when_gate_invariant_is_violated
// - CS-AC-03a PlayerSnapshot.class_id:
//   test_snapshot_contains_locked_class_id
// - ADR-010 payload continuity:
//   test_lobby_gate_emits_draft_initial_payloads_after_locking
