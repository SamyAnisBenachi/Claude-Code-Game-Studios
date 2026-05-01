// Story CS-002 evidence mapping.
//
// Executable tests live in server/tests/token_spawn_test.rs so they compile
// against the server crate. Run:
//   cargo test -p server --test token_spawn_test
//
// Coverage:
// - All 7 token types carry SourceClass and TokenUnit:
//   test_all_token_spawns_have_source_class_and_marker
// - Standard units have no SourceClass:
//   test_standard_unit_has_no_source_class
// - UnitBoardState.source_class derives from SourceClass for tokens:
//   test_unit_board_state_derives_source_class_for_tokens
// - S2CGameSnapshot includes token and standard source_class values:
//   test_game_snapshot_includes_token_source_class
// - Miranda-style owner transfer does not mutate SourceClass:
//   test_miranda_control_transfer_does_not_change_source_class
