# PROMPT 1468 - Placement Rejection Recovery UX Repair Relaunch

Status: BRANCH_PUSHED

Final line:
1468: PLACEMENT-REJECTION-RECOVERY-UX-REPAIR-RELAUNCH: BRANCH_PUSHED

## Base

- Base origin/main commit: `5cf9844f94d04d09aad2f36da06677bfa630a59a`
- PROMPT 1460 was treated as landed. The work started from `origin/main` at or after `4e4de4e6c6c57bab19585d094387e1f99d649345`, then rebased cleanly onto current `origin/main` `5cf9844f94d04d09aad2f36da06677bfa630a59a`.
- The stale PROMPT 1464 `WAITING_FOR_1460` conclusion was treated as obsolete.

## Implementation Summary

- Added rejected-batch tracking to `PendingPlacements` so the client remembers the exact pending placement batch that the server rejected.
- On `S2CPlacementRejected`, the placement panel now keeps the rejected reason visible, hides the submitted checkmark, reopens the timer submitted flag, and marks the unchanged rejected batch as inactive instead of immediately allowing the same invalid payload to be resent.
- `submit_pending_placements` now short-circuits an unchanged rejected batch with explicit `ServerRejected` correction state, preventing repeated silent invalid submit loops.
- Retargeting a staged card, changing its reserve/current split, or unstaging it clears the stale rejected-batch marker and restores the editable recovery path.
- Server rejection guidance now names the recovery action for spawn-range, occupancy, and invalid-target cases: retarget or unstage.
- Preserved accepted placement unit visibility from PROMPT 1460 by avoiding `client/src/presentation/board_rendering.rs` and `client/src/presentation/qa_snapshot.rs`.

## Files Changed

- `client/src/ui/hand/mod.rs`
- `tests/integration/hand-ui/hand_ui_placement_rejection_test.rs`
- `tests/integration/playable_client/active_loop_ui_state_test.rs`

Note: `tests/integration/playable_client/active_loop_ui_state_test.rs` received only the required `PendingPlacements` struct initializer update after the new rejected-batch field was added.

## Tests Run

Cargo/MSVC policy applied: yes.

Environment used before each Cargo command:

- `CARGO_TARGET_DIR=D:\_DEV\cargo-target\ccgs-msvc`
- `CARGO_PROFILE_DEV_DEBUG=0`
- `CARGO_PROFILE_TEST_DEBUG=0`
- `CARGO_INCREMENTAL=0`
- `RUSTFLAGS=-C debuginfo=0 -C link-arg=/DEBUG:NONE`

Commands/results:

- `cargo test -p client --test hand_ui_placement_rejection_test --test hand_ui_placement_unstaging_test --test hand_ui_submit_prevalidation_test`
  - Initial run exposed compile/test issues during development; fixed.
  - Final pre-rebase run: PASS, 17 tests passed.
  - Final post-rebase run: PASS, 17 tests passed.
- `git diff --check`
  - PASS.

Existing deprecation warnings for broad UI marker components were emitted during Cargo test; no failures.

## Branch / Commit / Push State

- Worker branch: `work/placement-rejection-recovery-ux-1468`
- Implementation commit: `e342c1e6`
- Push state: pushed to `origin/work/placement-rejection-recovery-ux-1468`
- Main push: not attempted; main integration remains orchestrator-owned.

1468: PLACEMENT-REJECTION-RECOVERY-UX-REPAIR-RELAUNCH: BRANCH_PUSHED
