# PROMPT-1434 - S18 Placement Submit Silent No-op Integration

Status: READY_FOR_MAIN_LAND

## Base and Source

- Integration worktree: `D:\_DEV\claude-code-game-studios-worktrees\S18-PLACEMENT-SUBMIT-SILENT-NOOP-INTEGRATION`
- Branch: `work/S18-PLACEMENT-SUBMIT-SILENT-NOOP-INTEGRATION`
- Base: `origin/main@0a9580072c6e8f43ffb069f84d7b57b015c816d6`
- PROMPT-1438 verification:
  - `8ced20a9fe71cf7200440abd653a52f5ccef3d28` is an ancestor of `origin/main`
  - `c4a92223eafba109b577c218aa4df0a901764152` is an ancestor of `origin/main`
- Source worker commit: `894bf66741f75fea115ddee3c3d575452b6ce328`

## Integration

Cherry-picked PROMPT-1399 onto `origin/main@77c5316e0078fb1a590fd96ccdf4da625f47f70a` with no conflicts, then rebased cleanly onto the later `origin/main@0a9580072c6e8f43ffb069f84d7b57b015c816d6`.

Changed files:

- `client/src/ui/hand/mod.rs`
- `tests/unit/hand-ui/placement_submit_core_test.rs`
- `reports/PROMPT-1434-s18-placement-submit-silent-noop-integration.md`

Behavior integrated:

- Removed the redundant `HandSubmitInteractionState::Active` gates from the submit click path and `submit_pending_placements`.
- Kept `PlacementTimer::submitted` as the authoritative duplicate-submit guard.
- Added explicit tracing for click receipt and submit short-circuit reasons.
- Preserved the PROMPT-1438 board-picking/window-cursor producer and the `HandUiPlacementCursorMoved` screen/world-position contract.
- Added `hu_25` and `hu_26` regressions for the silent no-op and duplicate-submit guard contracts.

## Validation

- `git diff --check origin/main...HEAD` - PASS
- Changed-file allowlist verified: only allowed hand UI/test/report files changed.

Cargo policy applied before Cargo:

- `CARGO_TARGET_DIR=D:\_DEV\cargo-target\ccgs-msvc`
- `CARGO_PROFILE_DEV_DEBUG=0`
- `CARGO_PROFILE_TEST_DEBUG=0`
- `CARGO_INCREMENTAL=0`
- `RUSTFLAGS=-C debuginfo=0 -C link-arg=/DEBUG:NONE`

Targeted tests:

- `cargo test -p client --test hand_ui_placement_submit_core_test --test hand_ui_placement_timer_test --test hand_ui_placement_rejection_test --test hand_ui_placement_staged_disclosure_accessibility_test --test hand_ui_submit_prevalidation_test --test hand_ui_drag_window_cursor_to_board_cell_test` - PASS on final rebased tip

Test totals from the targeted run:

- `hand_ui_drag_window_cursor_to_board_cell_test`: 2 passed
- `hand_ui_placement_rejection_test`: 2 passed
- `hand_ui_placement_staged_disclosure_accessibility_test`: 6 passed
- `hand_ui_placement_submit_core_test`: 7 passed
- `hand_ui_placement_timer_test`: 5 passed
- `hand_ui_submit_prevalidation_test`: 8 passed

Notes:

- The first sandboxed Cargo attempt failed with `Access is denied` on `D:\_DEV\cargo-target\ccgs-msvc\debug\.cargo-lock`; the required rerun with approved access passed.
- Branch push command attempted: `git push origin work/S18-PLACEMENT-SUBMIT-SILENT-NOOP-INTEGRATION`.
- Push result: normal sandboxed push failed to connect to GitHub; escalated push was rejected by approval policy as external export risk. Branch remains local and ready for orchestrator main-land.
- Cargo emitted existing deprecation warnings around coarse universal UI markers; no new compile errors or test failures.
- Live two-client retest remains separate for full AUDIT-1392-P01 closure per prompt.

1434: S18-PLACEMENT-SUBMIT-SILENT-NOOP-INTEGRATION: READY_FOR_MAIN_LAND
