# PROMPT 1444 -- Placement Drag Cursor Target Integration Refresh

## Summary

Integrated PROMPT 1442 placement drag cursor target repair onto current `origin/main`.

## Base and Source

- Base `origin/main`: `9c317ef25a34d6c65351f4283d6af444a8babddb`
- Source worker branch: `origin/work/prompt-1442-placement-drag-cursor-target-live-trace-repair`
- Source worker commit: `036d9aa4530799b6874e2425bdb9528939747f19`
- Integration branch: `integrate/placement-drag-cursor-target-1444`
- Resulting integration commit: branch HEAD of
  `integrate/placement-drag-cursor-target-1444` at delivery. The exact final
  HEAD is recorded in the completion relay/final response after the report
  amend, because embedding a commit's own hash inside that same commit is not
  stable.

## Changed Files

- `client/src/ui/hand/mod.rs`
- `tests/integration/hand-ui/hand_ui_drag_window_cursor_to_board_cell_test.rs`
- `reports/PROMPT-1442-placement-drag-cursor-target-live-trace-repair.md`
- `reports/PROMPT-1444-placement-drag-cursor-target-integration-refresh.md`

## Conflict Handling

The cherry-pick of `036d9aa4530799b6874e2425bdb9528939747f19` onto current
`origin/main` applied cleanly with no conflicts.

Current-main work was preserved, including landed bot protocol/session fields,
board picking, lobby CTA reachability, placement submit silent-noop behavior,
phase banner, result screen, bot foundation, and HUD microbadge work.

The integrated behavior preserves the intended PROMPT 1442 repair:

- `Interaction::Pressed` on an active staging fan slot starts placement drag
  instead of immediately using the default fan click/drop path.
- Window cursor position updates can feed `ActivePlacementDrag` cursor position
  while the drag is live.
- Drag release resolves from primary `Pointer<Release>` or primary mouse
  `ButtonInput` release.
- Cursor/drop logs include cursor data and resolved board-cell/target data.
- Ghost/unstage fan clicks and non-staging fan clicks retain the existing
  `HandFanCardClicked` behavior.

The old PROMPT 1442 report remains because it was tracked in the source
cherry-pick. This PROMPT 1444 report is the integration-refresh record.

## Verification

Cargo policy applied: yes.

Policy applied before every Cargo command in the same shell command:

- `CARGO_TARGET_DIR=D:\_DEV\cargo-target\ccgs-msvc`
- `CARGO_PROFILE_DEV_DEBUG=0`
- `CARGO_PROFILE_TEST_DEBUG=0`
- `CARGO_INCREMENTAL=0`
- `RUSTFLAGS=-C debuginfo=0 -C link-arg=/DEBUG:NONE`

Checks run:

- `cargo test -p client --test hand_ui_drag_window_cursor_to_board_cell_test`
  - Passed: 3 passed, 0 failed.
- `cargo test -p client --test hand_ui_drag_end_non_instant_test`
  - Passed: 4 passed, 0 failed.
- `cargo test -p client --test hand_ui_placement_action_panel_test -- ac5_submit_click_still_emits_c2s_submit_placement`
  - Passed: 1 passed, 0 failed, 4 filtered out.
- `git diff --check origin/main...HEAD`
  - Passed before adding this report.

Notes:

- Cargo emitted existing deprecation warnings around broad UI marker components
  such as `HudEntity`, `HandUiEntity`, and `ShopAuctionUiEntity`.
- No full workspace test run was performed, per prompt instruction.

## Remaining Verification

A separate live two-client VERIFY lane is still required after main-land to
prove runtime cursor movement and final target cell.

## Main-Land Readiness

Ready for main-land after the integration report commit is amended and the
branch push succeeds.

Final relay line:

`1444: PLACEMENT-DRAG-CURSOR-TARGET-LIVE-TRACE-INTEGRATION-REFRESH: READY_FOR_MAIN_LAND`
