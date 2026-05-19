# PROMPT-1438 - S18 Board Picking Backend Drag To Cell Main Land

## Summary

Refreshed PROMPT-1410 onto current `origin/main` after PROMPT-1433 landed. The worker commit cherry-picked cleanly with no conflicts and preserved the current-main hand-ui changes.

## Base

- Fetched `origin` before integration.
- `origin/main`: `30a270d071a85135c66ecfc5c11b5cc990075dc2`

## Source

- Worker branch: `origin/work/s18-board-picking-backend-1410`
- Worker commit: `12a547c9b582673235dd2deea721bc5aee4af903`
- Worker commit already on main: no

## Result

- Integration branch: `work/s18-board-picking-backend-1438`
- Replayed commit: `8ced20a9fe71cf7200440abd653a52f5ccef3d28`
- First pushed main tip with replay plus report: `c4a92223eafba109b577c218aa4df0a901764152`

## Changed Files

- `client/Cargo.toml`
- `client/src/ui/hand/mod.rs`
- `reports/PROMPT-1410-s18-board-picking-backend-drag-to-cell.md`
- `reports/PROMPT-1438-s18-board-picking-backend-drag-to-cell-main-land.md`
- `tests/integration/hand-ui/hand_ui_drag_window_cursor_to_board_cell_test.rs`

No forbidden paths were changed.

## Conflicts

- None.

## Verification

- `git diff --check HEAD~1 HEAD`: PASS before report file was added.
- Full workspace tests: not run, per prompt.
- Targeted Cargo tests: not run. The cherry-pick was clean and there was no semantic uncertainty requiring Cargo.

## Cargo Policy

Cargo was not invoked. If a follow-up verify lane runs Cargo on Windows/MSVC, use:

```powershell
$env:CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'
$env:CARGO_PROFILE_DEV_DEBUG='0'
$env:CARGO_PROFILE_TEST_DEBUG='0'
$env:CARGO_INCREMENTAL='0'
$env:RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'
```

## Live Retest

NOT RUN - separate runtime VERIFY lane required.

## Push Status

LANDED. Pushed `work/s18-board-picking-backend-1438` to `origin/main`:

- Before: `30a270d071a85135c66ecfc5c11b5cc990075dc2`
- After first push: `c4a92223eafba109b577c218aa4df0a901764152`
- Command: `git push origin HEAD:main`
