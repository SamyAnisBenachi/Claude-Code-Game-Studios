# BOARD-010 Board Rendering Baseline CI/Perf Guard Evidence

Status: Baseline guard and fixture evidence captured for the narrowed BOARD-002/003 visible board path.

Story: `production/epics/board-rendering/story-010-performance-evidence-and-ci-guards.md`

Scope:
- Included: Board Rendering grid/camera/Z layers, snapshot-spawned units, standing objectives, and HP bars.
- Excluded: BOARD-009 status icons, co-occupancy indicators, OUTNUMBERED indicators, spawn range overlays, trap visuals, final VFX, and full Board Rendering epic closure.
- Status-icon atlas evidence remains gated by BOARD-009.

## Source Guards

CI now runs a focused Board Rendering source guard step in `.github/workflows/tests.yml`:

```text
cargo test -p client --test board_rendering_grid_camera_test test_board_z_layers_are_named_constants --verbose
cargo test -p client --test board_rendering_plugin_scaffold_test board_rendering_does_not_register_phase_receiver --verbose
```

The Z guard scans `client/src/presentation/board_rendering.rs` spawn transform code and fails if the third argument to `Transform::from_xyz(...)` or `translation: Vec3::new(...)` contains an inline numeric literal.

The phase guard normalizes whitespace and verifies `MessageReceiver<S2CPhaseChanged>` appears only in `client/src/presentation/mod.rs`; Board Rendering must read `Res<CurrentClientPhase>` and must not drain the Lightyear phase receiver.

## Baseline Fixture Evidence

Automated fixture: `cargo test -p client --test board_rendering_snapshot_spawn_test test_baseline_board_path_supports_twenty_units_and_two_atlased_images --verbose`

The fixture validates the current BOARD-002/003 path can produce:
- 40 `BoardCellNode` entities.
- 20 visible `BoardUnit` entities, arranged as 4 units per lane across all 5 lanes.
- HP bar background and fill children on every visible unit.
- 10 standing objective entities in the current snapshot renderer, representing 5 objectives per player.
- Exactly two atlased board-rendering image handles for atlased sprites: the unit atlas and the board-elements atlas.

Approved standalone/non-counted batches in the narrowed baseline:
- Board cell nodes are tint sprites from `Sprite::from_color(...)`, not atlas-counted.
- Field wash / ghost translucent batches are not part of the BOARD-002/003 snapshot baseline and remain logged separately if present in a future browser capture.
- Status icons are excluded here and must be validated after BOARD-009.

## Verification Run

Local verification on 2026-05-05:

```text
cargo test -p client --test board_rendering_grid_camera_test test_board_z_layers_are_named_constants --verbose
cargo test -p client --test board_rendering_plugin_scaffold_test board_rendering_does_not_register_phase_receiver --verbose
cargo test -p client --test board_rendering_snapshot_spawn_test test_baseline_board_path_supports_twenty_units_and_two_atlased_images --verbose
cargo test -p client --test board_rendering_grid_camera_test --test board_rendering_plugin_scaffold_test --test board_rendering_snapshot_spawn_test
cargo fmt -p client -- --check
cargo check -p client
git diff --check
```

Results:
- Focused Z source guard: passed.
- Focused phase receiver source guard: passed.
- Focused 20-unit baseline fixture / two-atlas handle guard: passed.
- Full affected Board Rendering test targets: 20 tests passed.
- `cargo fmt -p client -- --check`: passed.
- `cargo check -p client`: passed.
- `git diff --check`: passed with existing Windows line-ending warnings only.

## Browser/WASM Capture

No browser/WASM frame-time screenshot capture is claimed by this baseline guard commit. The current repository has native ECS fixture coverage for the visible board path, but no automated browser harness that can seed the 20-unit worst-case snapshot at a 1920x1080 viewport and record frame timing.

Perf targets retained for the eventual browser capture:
- Total browser/WASM frame budget: <= 16.67 ms.
- ADR-021 presentation steady-state budget: < 1 ms.
- ADR-021 phase-boundary spike budget: < 3 ms.

Blocker for final visual performance closure: add or expose a browser/WASM fixture harness that can seed the BOARD-002/003 baseline snapshot, capture a 1920x1080 screenshot, and record frame timing. This blocker does not expand BOARD-010 into BOARD-009 status-icon atlas evidence or full epic closure.
