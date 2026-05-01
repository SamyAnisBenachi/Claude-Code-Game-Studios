# RSM Story 004 Test Evidence

Story: `production/epics/round-state-machine/story-004-win-condition-and-game-over.md`

Environment: Visual Studio 2026 Developer Command Prompt via:

```text
call "C:\Program Files\Microsoft Visual Studio\18\Community\Common7\Tools\VsDevCmd.bat" -arch=x64 -host_arch=x64
```

Required local command:

```text
cargo test -p server --test rsm_win_condition_test --test rsm_f2_ordering_test
```

Result:

```text
running 1 test
test rsm_f2_ordering_draft_entry_subscribers_process_broadcast_last ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

running 7 tests
test rsm_win_condition_single_loser_emits_objectives_destroyed_game_over ... ok
test rsm_win_condition_mutual_destruction_emits_one_draw_without_loser ... ok
test rsm_win_condition_mutual_destruction_draws_even_with_uneven_counts ... ok
test rsm_win_condition_direct_game_over_request_preserves_non_objective_reason ... ok
test rsm_win_condition_no_loss_below_threshold_for_both_players ... ok
test rsm_win_condition_no_loss_advances_to_next_draft_after_round_increment ... ok
test rsm_win_condition_above_threshold_keeps_single_loser_path ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Additional local check attempted:

```text
cargo test -p server rsm_
```

Result: blocked during compilation by local Windows paging/resource exhaustion (`os error 1455`, invalid metadata mmap errors) before the RSM tests completed. The required story-targeted Developer Command Prompt checks above passed.
