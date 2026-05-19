# PROMPT 1442 -- Placement Drag Cursor Target Live Trace Repair

Status: REPAIRED

## Root Cause

PROMPT 1440 live logs showed `fan_active_default_drop` with no `placement_cursor_move` evidence because the real fan-slot UI path was translating `Interaction::Pressed` directly into `HandFanCardClicked`, which immediately staged the default target. The drag runtime path only started from `Pointer<Press>` and ended from `Pointer<Release>`, but the live fan-slot interaction path was not proving those pointer messages were produced. As a result, active placement drag state never became live, the window cursor producer had nothing to consume, and the drop path stayed on the fixed click/default fallback.

## Files Changed

- `client/src/ui/hand/mod.rs`
- `tests/integration/hand-ui/hand_ui_drag_window_cursor_to_board_cell_test.rs`

## Behavior Before

- Pressing an active staging fan slot through Bevy UI `Interaction::Pressed` emitted `HandFanCardClicked`.
- `HandFanCardClicked` in staging immediately used `default_click_stage_target`, logging `fan_active_default_drop`.
- Cursor movement over the board was only useful after `ActivePlacementDrag` existed, so no drag meant no `placement_cursor_move` evidence and no cursor-selected target changes.

## Behavior After

- `Interaction::Pressed` on an active staging fan slot now emits `HandUiPlacementDragStarted` instead of the default click message.
- The existing window cursor producer can update `ActivePlacementDrag.cursor_world_position` while dragging over the board.
- Drag release now resolves from either `Pointer<Release>` or primary mouse `ButtonInput` release, so the Interaction-started path can complete without relying on a picking release message.
- Cursor-move logs now include the resolved board cell when `BoardLayout` is available.
- Drop-resolution logs now expose card, owner, cursor world/screen positions, and resolved target.
- Ghost/unstage fan clicks and non-staging fan clicks still use the existing `HandFanCardClicked` path.

## Verification

Cargo policy applied for all Cargo test runs:

```powershell
$env:CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'
$env:CARGO_PROFILE_DEV_DEBUG='0'
$env:CARGO_PROFILE_TEST_DEBUG='0'
$env:CARGO_INCREMENTAL='0'
$env:RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'
```

Targeted tests run:

- `cargo test -p client --test hand_ui_drag_window_cursor_to_board_cell_test`
  - Result: PASS, 3 passed.
  - Includes new coverage proving Interaction-started drag moves from one board cell to another and drops/stages the final cursor-derived cell.
- `cargo test -p client --test hand_ui_drag_end_non_instant_test`
  - Result: PASS, 4 passed.
- `cargo test -p client --test hand_ui_placement_action_panel_test -- ac5_submit_click_still_emits_c2s_submit_placement`
  - Result: PASS, 1 passed.

Existing deprecation warnings for broad UI marker components remained; no new warnings were introduced by this repair.

## Live Retest

Not run. Per prompt guidance, the full live two-client retest is left to a follow-up VERIFY lane because this repair is client-side and targeted tests now cover the missing Interaction-started drag path.

## Remaining Risks

- The live harness must perform a real press/move/release over two distinct board cells to prove runtime logs show `placement_drag_interaction_start`, `placement_cursor_move` with distinct resolved cells, `placement_drag_release`, and `placement_drop_resolved` using the final cell.
- If the native launcher still cannot provide usable cursor positions, the next VERIFY prompt should capture that as an environment/tooling blocker rather than a hand UI drag-path failure.

## Next Required VERIFY Prompt

Run a live two-client placement drag verification from this branch/commit. Required evidence: at least two `placement_cursor_move` lines with different `resolved_board_cell` values during one active drag, followed by `placement_drop_resolved target=Some(BoardCell { ... })` matching the final cursor cell and a staged/submitted placement using that same cell.

Final line: `1442: PLACEMENT-DRAG-CURSOR-TARGET-LIVE-TRACE-REPAIR: REPAIRED`
