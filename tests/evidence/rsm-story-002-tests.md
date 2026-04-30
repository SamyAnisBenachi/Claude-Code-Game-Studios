# RSM Story 002 Test Evidence

## Local Command

Run from Developer PowerShell for VS 2026:

```powershell
C:\Users\Sam\.cargo\bin\cargo.exe test -p server rsm_transitions
```

## Expected Coverage

- `advance_phase` handles all seven source phases.
- `BroadcastPhaseChanged` is asserted as the sole/last externally visible phase-broadcast payload per transition.
- DRAFT entry routing covers DraftInitial, DraftShop, and DraftAuction.
- Resolution exit increments `round_number` before `DraftStarted`.
- Double-transition guard prevents a second same-source transition.

## Result

PASS in clean `HEAD` snapshot with only S2-07 files applied.

```text
running 14 tests
test rsm_transitions_is_auction_round_matches_f1 ... ok
test rsm_transitions_is_auction_round_rejects_zero_in_debug - should panic ... ok
test rsm_transitions_lobby_to_draft_initial_emits_f2_order_payloads ... ok
test rsm_transitions_resolution_to_draft_shop_increments_before_draft_started ... ok
test rsm_transitions_resolution_to_draft_auction_emits_auction_before_broadcast ... ok
test rsm_transitions_draft_entry_shop_refresh_fans_out_once_per_player ... ok
test rsm_transitions_draft_auction_to_draft_shop_emits_shop_entry ... ok
test rsm_transitions_double_transition_guard_noops_after_first_advance ... ok
test rsm_transitions_game_over_source_is_terminal_noop ... ok
test rsm_transitions_wrong_expected_source_silently_noops ... ok
test rsm_transitions_draft_initial_to_placement_clears_submissions_and_broadcasts_last_payload ... ok
test rsm_transitions_game_over_entry_emits_game_over_then_zero_timer_broadcast ... ok
test rsm_transitions_draft_shop_to_placement_clears_submissions ... ok
test rsm_transitions_placement_to_resolution_emits_resolution_then_broadcast_payload ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Shared worktree note: the direct run in `D:\_DEV\claude-code-game-studios` is currently blocked by unrelated in-progress S2-08 economy files. The clean snapshot run proves the S2-07 patch against `HEAD`.
