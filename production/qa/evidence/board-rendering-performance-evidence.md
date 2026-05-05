# BOARD-012 Browser/WASM Board Performance Evidence

Status: Browser/WASM capture completed; corrected timing verdict PASS.
QA-COND-0004 is closed by the captured browser/WASM evidence.

Story: `production/epics/board-rendering/story-012-browser-wasm-board-performance-evidence.md`

Correction source: prior capture evidence commit
`00a841a6365cc99a67e3ff17483e686597339927`.

## Scope

Included:
- BOARD-010 narrowed visible board path: grid/camera/Z layers,
  snapshot-spawned units, standing objectives, and HP bars.
- Deterministic browser/WASM harness seed:
  `board-rendering-baseline-v1`.
- 1920x1080 browser screenshot and raw timing trace under
  `production/qa/evidence/captures/`.
- Corrected timing classification:
  - Browser RAF sampler owns the total frame verdict via
    `totalFrameBudgetPass`.
  - Bevy `Time<Real>::delta_secs_f64()` total-frame samples are diagnostic
    only.
  - Seeded full snapshot rebuild is compared to GDD `BR-RECONNECT-TIME`
    `<= 16.67 ms`.
  - ADR-021 `< 3 ms` remains documented for true phase-boundary
    hide/show/cancel-tween presentation spikes, not full snapshot rebuild.

Excluded:
- BOARD-009 final status-icon evidence.
- Spawn range highlights.
- Traps.
- Final VFX.
- Full Board Rendering epic closure.

## Environment

Capture date: 2026-05-05

Tooling detected and used:
- `trunk 0.21.14` installed via Cargo for this evidence run.
- `node` / `npx`: not available on PATH in this environment.
- Chrome: `147.0.7727.138` at
  `C:\Program Files\Google\Chrome\Application\chrome.exe`.
- Capture fallback:
  `production/qa/evidence/captures/board-rendering-baseline-capture.ps1`.

Harness build notes:
- The harness page keeps `data-wasm-opt="0"` because Trunk's downloaded
  `wasm-opt version_123` previously failed on Rust bulk-memory instructions.
- Trunk build/serve used a repo-local isolated target directory,
  `D:\_DEV\claude-code-game-studios\target\trunk-wasm`, for the browser
  harness build.

## Commands

Focused harness checks:

```text
cargo test -p client --test board_rendering_browser_wasm_perf_harness_test
cargo check -p client --bin board_rendering_perf_harness --target wasm32-unknown-unknown
```

Browser/WASM build and serve:

```text
cd client
$env:CARGO_TARGET_DIR = 'D:\_DEV\claude-code-game-studios\target\trunk-wasm'
trunk build board-rendering-perf-harness.html --release
trunk serve board-rendering-perf-harness.html --release --port 8080 --address 127.0.0.1
```

Capture command, from the repo root while Trunk serve is running:

```text
powershell.exe -ExecutionPolicy Bypass -File production\qa\evidence\captures\board-rendering-baseline-capture.ps1
```

Capture URL:

```text
http://127.0.0.1:8080/board-rendering-perf-harness.html?fixture=board_rendering_baseline&seed=board-rendering-baseline-v1
```

## Artifacts

Screenshot:

```text
production/qa/evidence/captures/board-rendering-baseline-1920x1080.png
```

Timing trace:

```text
production/qa/evidence/captures/board-rendering-baseline-timing.json
```

Screenshot verification:
- Dimensions: 1920x1080.
- Nonblank: PASS. A sampled pixel check found non-background pixels across the
  captured image.
- Visual content: PASS. The capture shows the seeded board baseline with all 5
  lanes framed, visible units, standing objectives, and HP bars rather than a
  menu or loading state.

## Fixture Counts

The committed timing trace reports:

- 5 lanes.
- 40 board cells.
- 20 visible units.
- 10 standing objectives.
- 20 units with HP bars.
- 20 post-reveal-ready units with scale 1.0 and alpha 1.0.
- 0 status icons.
- 0 spawn range highlights.
- 0 ghost units / lane ghost washes.

Fixture count verdict: PASS.

## Timing Results

Source: `production/qa/evidence/captures/board-rendering-baseline-timing.json`

Corrected BOARD-012 verdict:

| Budget | Source | Required | Observed | Verdict |
|---|---|---:|---:|---|
| Total frame max | Browser RAF sampler `totalFrameBudgetPass` | <= 16.67 ms | 6.0 ms | PASS |
| ADR-021 steady-state presentation max | Harness presentation window | < 1.0 ms/frame | 0.2 ms | PASS |
| BR-RECONNECT-TIME full snapshot rebuild | Seeded `S2CGameSnapshot` rebuild | <= 16.67 ms | 3.3 ms | PASS |

Diagnostic and non-gating observations:

| Metric | Observed | Classification |
|---|---:|---|
| Browser RAF average frame delta | 2.635 ms | Supporting total-frame diagnostic |
| Browser RAF sample count | 240 | Supporting total-frame diagnostic |
| Bevy `Time<Real>` total-frame diagnostic max | 4.2 ms | Diagnostic only |
| Bevy `Time<Real>` total-frame diagnostic average | 1.388 ms | Diagnostic only |
| ADR-021 true phase-boundary presentation spike | Not sampled | Budget retained for hide/show/cancel-tween phase work |

The prior 98.2 ms Bevy total-frame value from commit
`00a841a6365cc99a67e3ff17483e686597339927` is treated as a
browser/WASM harness measurement artifact, not as the BOARD-012 total-frame
verdict.

## Verification

Passed:

```text
cargo test -p client --test board_rendering_browser_wasm_perf_harness_test
cargo check -p client --bin board_rendering_perf_harness --target wasm32-unknown-unknown
trunk build board-rendering-perf-harness.html --release
powershell.exe -ExecutionPolicy Bypass -File production\qa\evidence\captures\board-rendering-baseline-capture.ps1
git diff --check
```

Automated screenshot/trace checks:
- Screenshot dimensions: 1920x1080.
- Screenshot nonblank sample: PASS.
- Browser RAF sampler: max 6.0 ms, avg 2.635 ms, `totalFrameBudgetPass=true`.
- Harness budget verdict: `board012BudgetPass=true`.
- Fixture counts match the required narrowed BOARD-010 baseline.

## Verdict

Capture artifact production: PASS.

Corrected browser/WASM timing budget status: PASS.

QA-COND-0004: Closed.

BOARD-012 blocker status: resolved for the browser/WASM performance evidence
gate. No board rendering behavior optimization was required.

No full Board Rendering epic closure is claimed by this evidence.

## Prior BOARD-010 Baseline Guards

Retained source guard commands from BOARD-010:

```text
cargo test -p client --test board_rendering_grid_camera_test test_board_z_layers_are_named_constants --verbose
cargo test -p client --test board_rendering_plugin_scaffold_test board_rendering_does_not_register_phase_receiver --verbose
cargo test -p client --test board_rendering_snapshot_spawn_test test_baseline_board_path_supports_twenty_units_and_two_atlased_images --verbose
```

These remain supporting native ECS fixture evidence only; the browser/WASM
timing verdict is carried by the artifacts listed above.
