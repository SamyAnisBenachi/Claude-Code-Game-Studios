# PROMPT 1466 -- BOARD-GRID-OVERLAY-TOGGLE-INTEGRATION-REFRESH

## Integration Summary

Reapplied the PROMPT 1459 board grid overlay toggle onto current `origin/main`
at `4e4de4e6c6c57bab19585d094387e1f99d649345`, after PROMPT 1460.

The source branch was not merged wholesale. The source commit
`330918f4ccfa572b7a84e8f35faec485895856f7` was cherry-picked with conflict
resolution limited to `client/src/presentation/board_rendering.rs`.

## Conflict Resolution

`board_rendering.rs` conflicted where PROMPT 1460's `BoardUnitRenderSource`
definition and PROMPT 1459's `BoardGridOverlayState` definition occupied the
same location.

Resolution kept both:

- `BoardUnitRenderSource::{AuthoritativeSnapshot, PlacementReveal}` remains
  intact for accepted placement reveal unit visibility.
- `BoardGridOverlayState` was layered in as the overlay toggle resource.

No changes were made to `client/src/presentation/qa_snapshot.rs`.

## Files Changed

- `client/src/presentation/board_rendering.rs`
- `client/src/presentation/board_rendering/rendering_constants.rs`
- `tests/unit/board_rendering/board_grid_camera_test.rs`
- `reports/PROMPT-1459-board-grid-overlay-toggle.md`
- `reports/PROMPT-1466-board-grid-overlay-toggle-integration-refresh.md`

## Verification

Cargo/MSVC policy was applied before every Cargo command:

- `CARGO_TARGET_DIR=D:\_DEV\cargo-target\ccgs-msvc`
- `CARGO_PROFILE_DEV_DEBUG=0`
- `CARGO_PROFILE_TEST_DEBUG=0`
- `CARGO_INCREMENTAL=0`
- `RUSTFLAGS=-C debuginfo=0 -C link-arg=/DEBUG:NONE`

Results:

- `cargo test -p client --test board_rendering_grid_camera_test` -- PASS, 9 passed.
- `cargo test -p client --test board_rendering_placement_reveal_test` -- PASS, 4 passed.
- `git diff --check origin/main...HEAD` -- PASS.

Both targeted tests emitted existing broad UI marker deprecation warnings.

## Branch / Commit

- Branch: `work/board-grid-overlay-toggle-integration-refresh-1466`
- Commit: current `HEAD` on the worker branch
- Push: `origin/work/board-grid-overlay-toggle-integration-refresh-1466`

1466: BOARD-GRID-OVERLAY-TOGGLE-INTEGRATION-REFRESH: INTEGRATED_BRANCH_PUSHED
