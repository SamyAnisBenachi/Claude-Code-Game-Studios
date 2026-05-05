# BOARD-012 Browser/WASM Board Performance Evidence

Status: Browser/WASM capture completed; timing verdict FAIL. QA-COND-0004
remains Open / Needs Remediation because the captured harness report missed the
total frame and ADR-021 rebuild spike budgets.

Story: `production/epics/board-rendering/story-012-browser-wasm-board-performance-evidence.md`

## Scope

Included:
- BOARD-010 narrowed visible board path: grid/camera/Z layers,
  snapshot-spawned units, standing objectives, and HP bars.
- Deterministic browser/WASM harness seed:
  `board-rendering-baseline-v1`.
- 1920x1080 browser screenshot and raw timing trace under
  `production/qa/evidence/captures/`.

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
- Edge was present at
  `C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe`, but Chrome
  was used for the committed capture.
- Capture fallback:
  `production/qa/evidence/captures/board-rendering-baseline-capture.ps1`.

Harness build note:
- The harness page now sets `data-wasm-opt="0"` because Trunk's downloaded
  `wasm-opt version_123` failed on Rust bulk-memory instructions during this
  run. This is a harness build-pipeline change only; board rendering behavior
  was not changed.

## Commands

Focused harness checks:

```text
cargo test -p client --test board_rendering_browser_wasm_perf_harness_test
cargo check -p client --bin board_rendering_perf_harness --target wasm32-unknown-unknown
```

Browser/WASM build and serve:

```text
cd client
trunk build board-rendering-perf-harness.html --release
trunk serve board-rendering-perf-harness.html --release --port 8080 --address 127.0.0.1
```

Capture command, from the repo root:

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
- Nonblank: PASS.
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

Harness report:

| Budget | Required | Observed | Verdict |
|---|---:|---:|---|
| Total frame max | <= 16.67 ms | 98.2 ms | FAIL |
| ADR-021 steady-state presentation max | < 1.0 ms/frame | 0.2 ms | PASS |
| ADR-021 phase/rebuild spike | < 3.0 ms | 3.3 ms | FAIL |

Additional browser `requestAnimationFrame` sampler:

| Metric | Observed |
|---|---:|
| Sample count | 240 |
| Average frame delta | 2.922 ms |
| Max frame delta | 5.0 ms |
| Browser sampler budget flag | PASS |

The browser sampler passed after Chrome throttling safeguards were added to the
fallback capture script. The BOARD-012 verdict remains FAIL because the harness
published report still failed the total frame max and strict ADR-021 rebuild
spike budgets.

## Verification

Passed:

```text
cargo test -p client --test board_rendering_browser_wasm_perf_harness_test
cargo check -p client --bin board_rendering_perf_harness --target wasm32-unknown-unknown
trunk build board-rendering-perf-harness.html --release
powershell.exe -ExecutionPolicy Bypass -File production\qa\evidence\captures\board-rendering-baseline-capture.ps1
git diff --check
```

`git diff --check` exited 0 with Windows LF-to-CRLF working-copy warnings only.

## Verdict

Capture artifact production: PASS.

Browser/WASM timing budget status: FAIL.

QA-COND-0004: remains Open / Needs Remediation.

BOARD-012 blocker status: the previous "capture could not run" blocker is
resolved, but BOARD-012 must remain blocked because the actual captured timing
does not satisfy the required budgets.

No full Board Rendering epic closure is claimed by this evidence.

## Prior BOARD-010 Baseline Guards

Retained source guard commands from BOARD-010:

```text
cargo test -p client --test board_rendering_grid_camera_test test_board_z_layers_are_named_constants --verbose
cargo test -p client --test board_rendering_plugin_scaffold_test board_rendering_does_not_register_phase_receiver --verbose
cargo test -p client --test board_rendering_snapshot_spawn_test test_baseline_board_path_supports_twenty_units_and_two_atlased_images --verbose
```

These remain supporting native ECS fixture evidence only; they do not override
the browser/WASM timing failure recorded above.
