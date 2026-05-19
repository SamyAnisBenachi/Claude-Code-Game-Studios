# PROMPT 1459 -- BOARD-GRID-OVERLAY-TOGGLE

## Implementation Summary

Added a session-visible QA/debug board grid overlay toggle to `BoardRenderingPlugin`.
The toggle defaults to off and appears as a `QA Grid: OFF` button while in-session.
Pressing it flips `BoardGridOverlayState.enabled`, updates the button label/color
immediately, logs the change, and spawns/despawns world-space grid overlay lines.

The overlay lines are generated from the active `BoardLayout` dimensions and
`BOARD_CELL_COUNT` / `BOARD_LANE_COUNT`; no duplicate hardcoded board layout was
introduced. Lines render at `Z_GRID_OVERLAY` above current board terrain, cell
nodes, units, health bars, and ghost previews. Overlay line entities do not carry
`Pickable`, so they do not block placement interaction, picking, drag, or drop.

## UI Location

The control is a QA/debug button named `QA Board Grid Overlay Toggle`, positioned
at the upper-right of the in-session UI, below the QA snapshot overlay slot
(`top: 48px`, `right: 8px`). Label states are `QA Grid: OFF` and `QA Grid: ON`.

## Files Changed

- `client/src/presentation/board_rendering.rs`
- `client/src/presentation/board_rendering/rendering_constants.rs`
- `tests/unit/board_rendering/board_grid_camera_test.rs`
- `reports/PROMPT-1459-board-grid-overlay-toggle.md`

## PROMPT 1455 Overlap

No active local PROMPT 1455 edits were present in the worktree. This task edited
`client/src/presentation/board_rendering.rs`, but no conflicting local 1455 change
to that file was detected, so implementation proceeded without waiting.

## Tests Run / Results

- `cargo fmt --all` -- PASS
- `cargo test -p client --test board_rendering_grid_camera_test` -- PASS, 9 passed

The first Cargo test invocation used the wrong test target name
(`board_grid_camera_test`) and failed before compilation. The corrected target
(`board_rendering_grid_camera_test`) passed. The focused test emitted existing
deprecation warnings for broad UI marker types; no new failures.

## Cargo Resource Policy Applied

Yes. Every Cargo command in this session used:

- `CARGO_TARGET_DIR=D:\_DEV\cargo-target\ccgs-msvc`
- `CARGO_PROFILE_DEV_DEBUG=0`
- `CARGO_PROFILE_TEST_DEBUG=0`
- `CARGO_INCREMENTAL=0`
- `RUSTFLAGS=-C debuginfo=0 -C link-arg=/DEBUG:NONE`

The focused test required escalation because the sandbox could not open the
shared MSVC target directory lock file.

## Remaining Visual / Manual QA Needed

Manual in-client QA should verify the button placement does not cover any
operator-critical HUD surface at common desktop/windowed resolutions, and that
the cyan grid remains readable over the current terrain/background in live
placement screenshots.

1459: BOARD-GRID-OVERLAY-TOGGLE: DONE
