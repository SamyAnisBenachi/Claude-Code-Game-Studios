# BOARD-012 Browser/WASM Board Performance Evidence

Status: Harness enabled; browser capture artifact is still pending in this
worker environment because `trunk`, `node`, and a browser executable were not
available on PATH.

Story: `production/epics/board-rendering/story-012-browser-wasm-board-performance-evidence.md`

## Scope

Included:
- BOARD-010 narrowed visible board path: grid/camera/Z layers,
  snapshot-spawned units, standing objectives, and HP bars.
- Deterministic browser/WASM harness seed:
  `board-rendering-baseline-v1`.
- Repeatable capture paths under `production/qa/evidence/captures/`.

Excluded:
- BOARD-009 final status-icon evidence.
- Spawn range highlights.
- Traps.
- Final VFX.
- Full Board Rendering epic closure.

## Harness

Browser entry point:

```text
client/board-rendering-perf-harness.html
```

Harness binary:

```text
cargo check -p client --bin board_rendering_perf_harness
cargo check -p client --bin board_rendering_perf_harness --target wasm32-unknown-unknown
```

Serve command, from the repo root:

```text
cd client
trunk serve board-rendering-perf-harness.html --release --port 8080
```

Capture command, from a second shell at the repo root:

```text
node production/qa/evidence/captures/board-rendering-baseline-capture.mjs
```

Default capture URL:

```text
http://127.0.0.1:8080/board-rendering-perf-harness.html?fixture=board_rendering_baseline&seed=board-rendering-baseline-v1
```

Expected artifacts:

```text
production/qa/evidence/captures/board-rendering-baseline-1920x1080.png
production/qa/evidence/captures/board-rendering-baseline-timing.json
```

## Fixture Counts

Automated harness test:

```text
cargo test -p client --test board_rendering_browser_wasm_perf_harness_test
```

Validated counts:
- 5 lanes.
- 40 board cells.
- 20 visible units.
- 10 standing objectives.
- HP bar background and fill children on every visible unit.
- 20 post-reveal-ready units with scale 1.0 and alpha 1.0.
- 0 status icons.
- 0 spawn range highlights.
- 0 ghost units / lane ghost washes.

## Timing Method

Total browser frame timing is sampled by
`production/qa/evidence/captures/board-rendering-baseline-capture.mjs` from the
browser `requestAnimationFrame` loop and compared against `<= 16.67 ms`.

Presentation timing is sampled inside the Bevy harness around the board
presentation systems. The harness records:
- Steady-state presentation max against `< 1 ms/frame`.
- Snapshot rebuild spike against `< 3 ms`.

The harness logs a `BOARD-012 harness result ...` JSON line to the browser
console; the capture script stores matching console lines in
`board-rendering-baseline-timing.json`.

## Current Results

Local worker verification:

```text
cargo test -p client --test board_rendering_grid_camera_test --test board_rendering_plugin_scaffold_test --test board_rendering_snapshot_spawn_test --test board_rendering_browser_wasm_perf_harness_test
cargo fmt -p client -- --check
cargo check -p client
cargo check -p client --bin board_rendering_perf_harness --target wasm32-unknown-unknown
git diff --check
```

Results:
- Harness fixture and budget-report integration test: passed.
- BOARD-010 supporting guard tests: passed.
- `cargo fmt -p client -- --check`: passed.
- `cargo check -p client`: passed.
- Harness WASM target check: passed.
- `git diff --check`: passed with existing Windows line-ending warnings only.
- Browser screenshot: not captured in this worker environment.
- Browser total frame timing: not sampled in this worker environment.
- ADR-021 browser presentation budget status: not sampled in a browser in this
  worker environment.

Pass/fail status:
- Fixture counts: PASS.
- Capture path readiness: PASS.
- Browser artifact production: BLOCKED until a machine with Trunk, Node, and a
  Playwright-supported browser runs the capture command.
- Browser timing budgets: NOT SAMPLED; no budget is weakened or claimed.

## Prior BOARD-010 Baseline Guards

Retained source guard commands from BOARD-010:

```text
cargo test -p client --test board_rendering_grid_camera_test test_board_z_layers_are_named_constants --verbose
cargo test -p client --test board_rendering_plugin_scaffold_test board_rendering_does_not_register_phase_receiver --verbose
cargo test -p client --test board_rendering_snapshot_spawn_test test_baseline_board_path_supports_twenty_units_and_two_atlased_images --verbose
```

These remain supporting native ECS fixture evidence only; they do not claim the
browser/WASM screenshot or timing capture.
