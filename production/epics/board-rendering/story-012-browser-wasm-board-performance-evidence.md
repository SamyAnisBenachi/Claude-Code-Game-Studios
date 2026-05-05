# Story 012: Browser/WASM Board Performance Evidence

> **Epic**: Board Rendering
> **Status**: Blocked
> **Layer**: Presentation
> **Type**: Config/Data
> **Manifest Version**: 2026-05-05
> **Sprint**: Sprint 6 S6-03

## Context

**GDD**: `design/gdd/board-rendering.md`
**Requirement**: `TR-BR-003`, `TR-BR-005`, plus direct GDD trace to Rule 5, `BR-3`, `BR-FRAME-TIME`, and `BR-RECONNECT-TIME`.
**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)
**Engine**: Bevy 0.18 + Lightyear 0.26 + browser/WASM target | **Risk**: HIGH
**Engine Notes**: Use `liv-bevy-018` for Bevy rendering or harness code. Use `liv-bevy-lightyear` only if implementation touches Lightyear receiver or protocol code.

BOARD-010 created native baseline guard evidence for the narrowed BOARD-002/003 visible board path and documented the remaining browser/WASM capture blocker. This story is the Sprint 6 S6-03 follow-up that turns that blocker into a repeatable browser/WASM evidence path. The goal is evidence: a render-capable harness, a deterministic post-reveal baseline fixture, a 1920x1080 nonblank screenshot, frame timing, and an updated evidence document.

This story does not claim final Board Rendering epic closure. It measures only the narrowed BOARD-010 baseline: board grid/camera/Z layers, snapshot-spawned units, standing objectives, and HP bars.

## Traceability

- `TR-BR-003`: Board Rendering must use named Z constants and render from replicated board state. This story validates that the visible board path can be rendered in a live browser/WASM build at the target viewport.
- `TR-BR-005`: Reconnect and snapshot rebuild behavior traces to `BR-RECONNECT-TIME`; this story measures the phase-boundary or rebuild spike on the WASM target.
- GDD Rule 5 and `BR-3`: the browser capture must use a fully populated narrowed baseline and record whether the live board frame stays within the board rendering performance expectations.
- `BR-FRAME-TIME`: frame timing on the WASM target must be compared against the 60 FPS total frame budget of <=16.67 ms.
- ADR-021 Performance Implications: presentation steady state must stay below 1 ms per frame, and the phase-boundary or rebuild spike must stay below 3 ms.
- Sprint 6 S6-03: the capture must record a nonblank 1920x1080 browser/WASM board, framed lanes, frame timing, and ADR-021 budget observations.

## Scope

In scope for this story:

- Add or expose a browser/WASM render-capable performance harness for Board Rendering.
- Seed the narrowed BOARD-010 baseline fixture deterministically.
- Capture a 1920x1080 desktop browser screenshot showing a nonblank board with all 5 lanes framed.
- Record frame timing against the <=16.67 ms total frame budget.
- Record ADR-021 presentation timing observations: steady-state presentation < 1 ms/frame and phase-boundary or rebuild spike < 3 ms.
- Update `production/qa/evidence/board-rendering-performance-evidence.md` with the commands, environment, fixture counts, timing results, pass/fail status, and artifact links.

Out of scope for this story:

- BOARD-009 status-icon final evidence.
- Spawn range highlights.
- Traps.
- Final VFX.
- Final art production.
- Full Board Rendering epic closure.
- Any change to authoritative gameplay rules, server state, or networking contracts.

## Baseline Fixture

The browser/WASM harness must seed this narrowed BOARD-010 baseline:

- 5 lanes.
- 40 board cells.
- 20 visible units.
- 10 objective entities.
- HP bars on every visible unit.
- Post-reveal steady state with visible opponent entities at alpha 1.0 and scale 1.0.

The fixture seed must be stable and recorded in the evidence document. Use `board-rendering-baseline-v1` unless implementation discovers an existing project fixture seed convention, in which case the evidence must name the convention used.

The baseline must exclude status icons, spawn range highlights, traps, and final VFX. If any excluded feature is visible in the capture, the evidence must mark the capture as invalid for BOARD-012 until the fixture is narrowed again.

## Control Manifest Guardrails

- Current manifest version is `2026-05-05`.
- Board content is world-space `Sprite` plus `Transform` rendered by `Camera2d`; do not implement board units, objectives, cells, or HP bars as bevy_ui nodes.
- `MessageReceiver<S2CPhaseChanged>` is drained only by the shared `phase_sink_system`. Board Rendering reads `Res<CurrentClientPhase>` and must not add a second phase receiver.
- ADR-021 performance budgets apply: presentation steady state < 1 ms/frame; phase-boundary or rebuild spike < 3 ms; total browser/WASM frame budget <=16.67 ms at 1920x1080.
- Bevy 0.18 Required Components API applies to any harness or rendering code touched by the implementation. Do not use deprecated bundles such as `SpriteBundle`, `Camera2dBundle`, `NodeBundle`, `TransformBundle`, or `SpatialBundle`.

## Acceptance Criteria

- [ ] **Browser harness**: Given the client is built for browser/WASM, when the BOARD-012 performance harness starts with seed `board-rendering-baseline-v1`, then it renders the narrowed baseline fixture without requiring a live multiplayer session.
- [ ] **Fixture counts**: Given the harness reaches post-reveal steady state, when the fixture is inspected or logged, then it reports 5 lanes, 40 board cells, 20 visible units, 10 objectives, and HP bars on every visible unit.
- [ ] **Viewport capture**: Given the harness is running at 1920x1080 browser viewport size, when screenshot capture runs, then it writes `production/qa/evidence/captures/board-rendering-baseline-1920x1080.png`.
- [ ] **Nonblank board evidence**: Given the screenshot artifact exists, when QA reviews it, then the image is nonblank, shows board content rather than a menu/loading state, and frames all 5 lanes.
- [ ] **Total frame timing**: Given the harness is in post-reveal steady state, when frame timing is sampled on the browser/WASM build, then the evidence records observed frame timing and compares it against <=16.67 ms.
- [ ] **ADR-021 steady-state budget**: Given the presentation steady-state sample window is recorded, when the results are written to evidence, then the story records pass/fail against < 1 ms/frame for presentation work.
- [ ] **ADR-021 spike budget**: Given the harness triggers the snapshot rebuild or equivalent phase-boundary frame, when the spike is measured, then the story records pass/fail against < 3 ms for presentation work.
- [ ] **Evidence update**: Given capture and timing are complete, when the evidence document is updated, then `production/qa/evidence/board-rendering-performance-evidence.md` lists the exact build command, browser, viewport, seed, fixture counts, screenshot path, timing method, raw timing values, pass/fail status, and any trace/log artifact path.
- [ ] **Scope guard**: Given the evidence is reviewed, when BOARD-012 is closed, then the evidence explicitly states that BOARD-009 status-icon final evidence, spawn range highlights, traps, final VFX, and full Board Rendering epic closure are not claimed.
- [ ] **Failure handling**: Given the board is blank, not fully framed, fixture counts differ, or timing exceeds any required budget, when evidence is written, then the result is recorded as a failure or blocker rather than weakening the fixture or budgets.

## Implementation Notes

- This story may add dev/test-only browser harness code when implemented. This story file creation does not implement that harness.
- Use `liv-bevy-018` for any Bevy code touched during implementation.
- Prefer a dev/test-only URL fixture mode for the WASM client using parameters such as `fixture=board_rendering_baseline` and `seed=board-rendering-baseline-v1`, so browser automation can load the same render state repeatedly.
- The harness must wait until the post-reveal steady state is visible before taking the screenshot. Alpha must be 1.0 and scale must be 1.0 for all visible units in the fixture.
- Frame timing may use browser Performance API samples, browser devtools trace output, Bevy diagnostics exposed by the harness, or an equivalent repeatable timing method. The evidence must describe the method well enough for another contributor to repeat it.
- The phase-boundary or rebuild spike sample should use the same seeded fixture and trigger the board rebuild path closest to `S2CGameSnapshot` full-board rebuild behavior.
- The harness must not add or drain a second `MessageReceiver<S2CPhaseChanged>`.
- Do not edit `production/sprint-status.yaml` or `production/session-state/**` as part of this story unless a later orchestration prompt explicitly authorizes those files.

## QA Test Cases

- **Browser fixture launch**
  - Given: a browser/WASM client build with the BOARD-012 fixture mode enabled
  - When: the harness opens the baseline fixture URL at 1920x1080
  - Then: the rendered scene reaches post-reveal steady state with the required fixture counts.

- **Screenshot capture**
  - Given: the fixture is stable
  - When: screenshot capture runs
  - Then: `production/qa/evidence/captures/board-rendering-baseline-1920x1080.png` is produced and shows a nonblank board with all 5 lanes framed.

- **Frame budget capture**
  - Given: the fixture is stable
  - When: steady-state frame timing is sampled and a rebuild or phase-boundary frame is triggered
  - Then: total frame time, presentation steady-state time, and presentation spike time are recorded against the required budgets.

- **Scope guard**
  - Given: the evidence document is updated
  - When: QA reviews the BOARD-012 evidence
  - Then: the document does not claim BOARD-009 final evidence, spawn range highlights, traps, final VFX, or full Board Rendering epic closure.

## Test Evidence

**Required evidence**:
- Evidence update: `production/qa/evidence/board-rendering-performance-evidence.md`
- Required screenshot output: `production/qa/evidence/captures/board-rendering-baseline-1920x1080.png`
- Optional trace/log output: `production/qa/evidence/captures/board-rendering-baseline-*.json`, `production/qa/evidence/captures/board-rendering-baseline-*.log`, or an equivalent browser trace artifact under `production/qa/evidence/captures/`

**Asset reference note**: the `.png`, `.json`, and `.log` paths above are future output artifacts for this story, not pre-existing input assets. They are expected not to exist before BOARD-012 implementation.

**Status**: [ ] Browser capture blocked. The harness and capture path are implemented, but the required 1920x1080 browser screenshot and browser/WASM timing artifacts were not produced because `trunk`, `node`, and a browser executable were unavailable on PATH during story-done verification.

## Dependencies

- Depends on: [Story 010](story-010-performance-evidence-and-ci-guards.md), which created the narrowed BOARD-010 baseline guard evidence and documented the browser/WASM capture blocker.
- Supporting completed baseline: [Story 002](story-002-board-grid-camera-and-z-layers.md) and [Story 003](story-003-snapshot-spawn-units-objectives-and-hp-bars.md).
- Unlocks: Sprint 6 S6-03 evidence for QA-COND-0004 browser/WASM board performance capture.

## Story Done Review Notes

**Reviewed**: 2026-05-05

**Verdict**: BLOCKED - the BOARD-012 harness implementation is integrated and verified, but the story cannot be marked Complete because the required browser/WASM screenshot and timing artifacts were not produced.

**Integrated implementation**: Worker commit reviewed: `8e0fce4772c0164ba3a879e081575f2576e3d473`. The integration branch was rebased on `origin/main` during story-done verification.

**Verified coverage**:
- Harness binary and WASM target build check pass.
- Baseline fixture coverage verifies 5 lanes, 40 board cells, 20 visible units, 10 objectives, HP bars on every visible unit, and post-reveal-ready state.
- The fixture excludes BOARD-009 status-icon final evidence, spawn range highlights, traps, final VFX, and full Board Rendering epic closure claims.
- `production/qa/evidence/board-rendering-performance-evidence.md` records the capture commands, artifact paths, fixture counts, and explicit blocker.

**Blocked acceptance criteria**:
- Viewport capture and nonblank board evidence remain unverified because `production/qa/evidence/captures/board-rendering-baseline-1920x1080.png` was not produced.
- Browser total frame timing and ADR-021 steady/spike budget verdicts remain unverified because `production/qa/evidence/captures/board-rendering-baseline-timing.json` was not produced.
- Evidence remains partial because it does not contain raw browser timing values, browser environment details, or browser budget pass/fail verdicts.

**QA-COND-0004**: Remains Open / Needs Evidence.
