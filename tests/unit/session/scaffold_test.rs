// Story S3-01 evidence mapping.
//
// Executable tests live in server/tests/session_scaffold_test.rs so they compile
// against the server crate. Run:
//   cargo test -p server --test session_scaffold_test
//
// Coverage:
// - LobbyState equality: test_lobby_state_waiting_and_active_are_distinct
// - Type construction smoke: test_session_scaffold_constructs_all_new_types
// - build_session_config valid setup: test_build_session_config_valid_two_player_setup
// - build_session_config missing class panic: test_build_session_config_panics_when_occupied_slot_has_no_class
// - SessionReady zero-sized Observer trigger: test_session_ready_is_zero_sized_observer_trigger
// - SessionReady doc-comment grep literal: test_session_ready_doc_comment_contains_grep_gate_literal
// - GameSessionPlugin skeleton compiles/registers: test_game_session_plugin_registers_cleanly
