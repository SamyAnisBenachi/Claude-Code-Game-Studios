# PROMPT 1453 -- Placement Drag Cursor Board Hit-Test Live Repair

Status: REPAIRED_NEEDS_LIVE_VERIFY

## Root Cause

PROMPT 1449 did not prove a bad board hit-test conversion. It proved the live cursor never reached the board envelope.

The live 1449 screen points project through the live board camera as:

- Client B screen `Vec2(972, 643)` -> world `Vec2(556, -443)`
- Client A screen `Vec2(955, 683)` -> world `Vec2(539, -483)`

The default board envelope for the live camera is:

- world min `Vec2(-32, -360)`
- world max `Vec2(480, 40)`
- world center `Vec2(224, -160)`

Those cursor world positions are below/right of the board envelope, so `resolved_board_cell=None` and `placement_drop_resolved target=None` were correct for the coordinates in the evidence. The old deterministic fixture also had a gap: it used a world-origin camera instead of the production board-centered camera, so it did not directly encode the live viewport conversion envelope.

## Changes

- `client/src/ui/hand/mod.rs`
  - Added `CursorBoardHitDiagnostic` around the existing `cursor_to_lane_cell` calculation.
  - `placement_cursor_move` logs now include `board_hit`, with resolved cell, nearest clamped cell, nearest cell world center, board min/max/center, and reject reason.
  - `placement_drop_resolved` logs now include the same board-hit diagnostic for the final cursor.
  - Target resolution semantics are unchanged: out-of-envelope cursor positions still do not stage hidden placements.

- `client/src/presentation/board_rendering.rs`
  - Logs `client::presentation::board_rendering::board_envelope` when board session resources are inserted, including origin, cell size, min/max, center, lane count, and cell count.

- `tests/integration/hand-ui/hand_ui_drag_window_cursor_to_board_cell_test.rs`
  - Updated the fixture camera to match live board camera placement.
  - Added `live_1449_fixed_screen_points_are_outside_board_but_cell_centers_resolve`.
  - The new test proves the 1449 fixed screen points are outside the board envelope while known `BoardLayout::cell_to_world` centers project to distinct viewport points and stage the final board cell.

## Tests

Cargo policy applied: yes.

Policy:

```powershell
$env:CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'
$env:CARGO_PROFILE_DEV_DEBUG='0'
$env:CARGO_PROFILE_TEST_DEBUG='0'
$env:CARGO_INCREMENTAL='0'
$env:RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'
```

Results:

- `cargo test -p client --test hand_ui_drag_window_cursor_to_board_cell_test`
  - PASS: 4 passed.
- `cargo test -p client --test hand_ui_drag_end_non_instant_test`
  - PASS: 4 passed.
- `cargo test -p client --test board_rendering_targeting_feedback_test`
  - PASS: 10 passed.
- `git diff --check`
  - PASS.

One command was mistyped before the correct board test:

- `cargo test -p client --test targeting_feedback_test`
  - FAIL before execution: no such test target. Correct target was run and passed.

Existing deprecation warnings for broad UI marker components remained.

## Live Evidence

Prior evidence used:

- `D:\_DEV\Work\Claude-Code-Game-Studios\.codex-worktrees\prompt-1449\production\qa\evidence\captures\placement-drag-cursor-live-verify-2026-05-19-123740\`

No new live two-client proof was produced in this lane. The code now emits enough live diagnostics to distinguish:

- cursor outside board envelope,
- nearest clamped board-cell candidate,
- actual resolved cell,
- final drop target.

## Required Next Verify

Another live verify lane is required. The live operator/harness must drag through projected board-cell centers from the live `BoardLayout`/`BoardEnvelope`, not fixed bottom-window points like the 1449 run.

PASS evidence should show:

- `client::presentation::board_rendering::board_envelope` with min/max/center,
- at least two `placement_cursor_move` lines with different `resolved_board_cell=Some(...)`,
- `placement_drop_resolved target=Some(BoardCell { ... })`,
- non-empty placement submit accepted or rejected with a clear server reason.

Final relay line:

1453: PLACEMENT-DRAG-CURSOR-BOARD-HITTEST-LIVE-REPAIR: REPAIRED_NEEDS_LIVE_VERIFY
