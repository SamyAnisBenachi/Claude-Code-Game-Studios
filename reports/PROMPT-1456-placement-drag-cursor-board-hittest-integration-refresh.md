# PROMPT 1456 -- Placement Drag Cursor Board Hit-Test Integration Refresh

Status: INTEGRATED_BRANCH_PUSHED

## Base and Source

- Base: `origin/main` at `86e50e831befde7e0a4978c93b40556c1383fd77` (`Repair HUD phase timer countdown snapshots`).
- Integration branch: `integrate/placement-drag-cursor-board-hittest-1456`.
- Worktree: `D:\_DEV\claude-code-game-studios-worktrees\placement-drag-cursor-board-hittest-integration-refresh-1456`.
- Source branch: `origin/work/placement-drag-cursor-board-hittest-live-repair-1453`.
- Source commit applied: `8e3fbc07f68b0fa0ae0f0c99d4170c84e468fea4` (`PROMPT-1453 diagnose placement drag board hit-test`).

## Integration Handling

- Fetched `origin` before creating the worktree.
- Created a fresh isolated worktree from current `origin/main`.
- Cherry-picked only source commit `8e3fbc07`.
- Conflict handling: no conflicts; cherry-pick applied cleanly on top of PROMPT 1452 main.
- Preserved landed PROMPT 1450 auction leader perspective and PROMPT 1452 HUD/timer snapshot countdown by basing directly on `origin/main` `86e50e83`.
- Copied PROMPT 1453 evidence report into `reports/PROMPT-1453-placement-drag-cursor-board-hittest-live-repair.md`.

## Changed Files

- `client/src/ui/hand/mod.rs`
  - Integrated `CursorBoardHitDiagnostic` around placement cursor board hit-testing.
  - `placement_cursor_move` logs include board-hit diagnostics: resolved cell, nearest clamped cell, nearest cell world center, board min/max/center, and reject reason.
  - `placement_drop_resolved` logs include equivalent final-cursor board-hit diagnostics.
  - Strict target resolution is preserved: out-of-envelope cursor positions still do not stage hidden placements.
- `client/src/presentation/board_rendering.rs`
  - Board session insertion logs `client::presentation::board_rendering::board_envelope` with origin, cell size, min/max, center, lane count, and cell count.
- `tests/integration/hand-ui/hand_ui_drag_window_cursor_to_board_cell_test.rs`
  - Fixture camera now matches the production/live board-centered camera.
  - Added coverage proving the PROMPT 1449 fixed screen points are outside the board envelope while known `BoardLayout::cell_to_world` centers resolve and stage the final board cell.
- `reports/PROMPT-1453-placement-drag-cursor-board-hittest-live-repair.md`
  - Added copied PROMPT 1453 source report.
- `reports/PROMPT-1456-placement-drag-cursor-board-hittest-integration-refresh.md`
  - Added this integration report.

## Verification

Cargo policy applied: yes.

```powershell
$env:CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'
$env:CARGO_PROFILE_DEV_DEBUG='0'
$env:CARGO_PROFILE_TEST_DEBUG='0'
$env:CARGO_INCREMENTAL='0'
$env:RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'
```

Targeted tests:

- `cargo test -p client --test hand_ui_drag_window_cursor_to_board_cell_test` -- PASS, 4 passed.
- `cargo test -p client --test hand_ui_drag_end_non_instant_test` -- PASS, 4 passed.
- `cargo test -p client --test board_rendering_targeting_feedback_test` -- PASS, 10 passed.
- `git diff --check HEAD~1..HEAD` -- PASS before adding report files.

Notes:

- Initial sandboxed Cargo attempt failed opening `D:\_DEV\cargo-target\ccgs-msvc\debug\.cargo-lock` with access denied; reran with approved escalation and the required policy.
- Cargo emitted existing deprecated broad UI marker warnings (`HudEntity`, `HandUiEntity`, `ShopAuctionUiEntity`); no new warnings were triaged in this lane.

## Required Next Verify

A new live verify is required after main-land. The live verifier should drag through projected board-cell centers from the live `BoardLayout`/`BoardEnvelope`, not fixed bottom-window points from PROMPT 1449. PASS evidence should show `client::presentation::board_rendering::board_envelope`, multiple `placement_cursor_move` logs with different `resolved_board_cell=Some(...)`, and `placement_drop_resolved target=Some(BoardCell { ... })` followed by placement submit acceptance or clear server rejection.

Final relay line:

1456: PLACEMENT-DRAG-CURSOR-BOARD-HITTEST-INTEGRATION-REFRESH: INTEGRATED_BRANCH_PUSHED
