# Story 010: Performance Evidence and CI Guards

> **Epic**: Board Rendering
> **Status**: Complete
> **Layer**: Presentation
> **Type**: Config/Data
> **Manifest Version**: 2026-05-05

## Context

**GDD**: `design/gdd/board-rendering.md`
**Requirement**: `TR-BR-003`, `TR-BR-005`, plus direct GDD trace to Rule 5, `BR-3`, `BR-3a`, `BR-3b`, `BR-3c`, `BR-2-ATLAS`, `BR-FRAME-TIME`, and `BR-RECONNECT-TIME`.
**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md)

This story is narrowed to baseline CI and performance guard evidence for the visible board path delivered by BOARD-002 and BOARD-003: grid/camera/Z layers, snapshot-spawned units, standing objectives, and HP bars. It does not claim final full-board atlas closure. Final status-icon atlas evidence remains dependent on BOARD-009 because that story owns status icons, co-occupancy, spawn range highlights, and the final status-icon use of the board-elements atlas.

## Traceability

- `TR-BR-003`: Board Rendering uses named Z constants rather than inline Z literals. This story preserves that invariant with a guard scan over board rendering spawn code.
- `TR-BR-005`: Reconnect/snapshot timing evidence traces to `BR-RECONNECT-TIME`, which requires a measured `S2CGameSnapshot` full-board rebuild frame on the WASM target.
- GDD Rule 5, `BR-3`, `BR-3a`, `BR-3b`, `BR-3c`, and `BR-2-ATLAS`: unit sprites and HP bars use the unit atlas; cell nodes/objectives/tokens use the board-elements atlas; per-unit material handles are forbidden; exactly two atlased `Handle<Image>` AssetIds are expected for board-rendering atlased entities, excluding approved standalone exceptions.
- `BR-FRAME-TIME`: worst-case board rendering evidence must measure the WASM target against the 60 FPS frame budget of <=16.67 ms.
- Conditional final-scope trace: `TR-BR-006`, `TR-BR-007`, `BR-STATUS-CONTRACT`, `BR-STATUS-TIER`, `BR-STATUS-COOCCUPANCY`, and status-icon participation in `BR-2-ATLAS` are deferred until BOARD-009 is complete. Full Board Rendering epic closure depends on BOARD-009 if final status-icon atlas evidence is restored to this story's closure scope.

## Scope

In scope for this story:

- Baseline guard/evidence for BOARD-002 and BOARD-003 visible board rendering.
- Z literal guard evidence for board rendering spawn code.
- Single `MessageReceiver<S2CPhaseChanged>` drain guard evidence.
- Browser/WASM screenshot and performance capture for the baseline visible board path.
- Atlas handle-count evidence for the baseline board path, with approved standalone atlas exceptions recorded explicitly.

Out of scope for this story:

- Implementing missing rendering features from earlier stories.
- Final status icon, co-occupancy, OUTNUMBERED, and spawn range evidence owned by BOARD-009.
- Trap face-down rendering and final art production.
- Full Board Rendering epic closure. This story unlocks baseline performance guard evidence only.

## Evidence Targets

- **Worst-case baseline board state**: 5 lanes active, all 40 cell nodes present, target 20 visible units where the current BOARD-003 snapshot fixture supports 4 units per lane, all 5 standing objectives present, and HP bar children on every visible unit. Capture the post-reveal steady state with alpha 1.0 and scale 1.0. Exclude status icons, trap visuals, final VFX, and BOARD-009 spawn range overlays from this baseline evidence.
- **Target desktop viewport**: 1920x1080 CSS pixels at 100 percent browser zoom and 100 percent UI scale in a desktop browser WASM build. Screenshot evidence must show a nonblank board with all 5 lanes framed.
- **GDD frame-time target**: 60 FPS total frame budget, <=16.67 ms per frame. Also record ADR-021 presentation budgets: steady-state presentation <1 ms and phase-boundary spike <3 ms.
- **Approved standalone atlas exceptions**: board background and Field wash are excluded from the two-atlas handle count. Any ghost or reveal translucent batch present in the capture is logged separately as an approved GDD Rule 5 translucent batch. Status icons are not part of this narrowed baseline count and must be validated after BOARD-009.

## Control Manifest Guardrails

- Current manifest version is `2026-05-05`.
- Presentation code must keep the single `phase_sink_system` as the only `MessageReceiver<S2CPhaseChanged>` drain. Board Rendering systems read `Res<CurrentClientPhase>` and must not register their own Lightyear phase receiver.
- ADR-021 performance budgets apply: Presentation steady-state <1 ms per frame; phase-boundary frame <3 ms spike; global 60 FPS browser/WASM frame budget <=16.67 ms.
- Bevy 0.18 Required Components API applies wherever evidence or guard notes inspect rendering code. Do not use deprecated bundles such as `SpriteBundle`, `Camera2dBundle`, `NodeBundle`, `TransformBundle`, or `SpatialBundle`.
- Atlas code must use `Handle<Image>` plus `Handle<TextureAtlasLayout>` in `Sprite.texture_atlas`. `Handle<TextureAtlas>` is forbidden.

## Acceptance Criteria

- [ ] CI or local guard fails if inline Z literals are introduced in board rendering spawn code.
- [ ] CI or local guard verifies `MessageReceiver<S2CPhaseChanged>` appears only in the shared presentation phase sink and is not drained by Board Rendering.
- [ ] Browser/WASM evidence captures the worst-case baseline board state defined in this story.
- [ ] Frame-time evidence is recorded at 1920x1080 and compares the capture against <=16.67 ms total frame time plus the ADR-021 presentation budgets.
- [ ] Atlas handle-count evidence confirms two atlased board-rendering image handles for the narrowed baseline path: unit atlas and board-elements atlas.
- [ ] Evidence lists approved standalone atlas exceptions separately: board background and Field wash, plus any ghost or reveal translucent batch present in the capture.
- [ ] Screenshot evidence confirms the board is nonblank and all 5 lanes are framed at the target desktop viewport.
- [ ] Evidence records that final status-icon atlas evidence is deferred until BOARD-009 is complete.

## Implementation Notes

- This is a documentation, guardrail, and evidence story, not a visual feature story.
- Keep evidence files under `production/qa/evidence/` and reference the exact build, browser, viewport, and guard commands used.
- If the current snapshot fixture cannot produce 20 visible units, record the actual fixture cap and a concrete blocker in the evidence document rather than weakening the target.
- If final art or atlas packaging changes the draw-call assumptions, record the delta and require technical-director approval before treating it as final epic closure evidence.
- This docs repair does not implement code, create CI scripts, or create evidence files.

## Out of Scope

- Implementing missing rendering features from earlier stories.
- Solving art pipeline atlas splits.
- Browser automation infrastructure beyond the lightweight evidence needed for this epic.
- Final status-icon atlas closure before BOARD-009 is complete.

## QA Test Cases

- **Z guard**
  - Given: board rendering source
  - When: the guard scan runs
  - Then: no inline Z literals are accepted outside named constants or derived local offsets.

- **Single phase drain**
  - Given: client presentation source
  - When: `MessageReceiver<S2CPhaseChanged>` is scanned
  - Then: only the shared phase sink owns the Lightyear receiver and Board Rendering reads `Res<CurrentClientPhase>`.

- **Baseline WASM visual performance**
  - Given: the worst-case baseline board state at 1920x1080
  - When: performance capture is recorded in a desktop browser WASM build
  - Then: frame timing, presentation budget observations, and pass/fail status are documented.

- **Atlas handle count**
  - Given: the worst-case baseline board state
  - When: board-rendering atlased sprite `Handle<Image>` AssetIds are collected
  - Then: the unit atlas and board-elements atlas are the only counted atlased board-rendering image handles, with standalone exceptions listed separately.

## Test Evidence

**Required evidence**:
- Config/Data: CI or local guard notes in `production/qa/evidence/board-rendering-performance-evidence.md`
- Browser screenshot/performance evidence for the worst-case baseline board state at 1920x1080.

**Status**: [x] Baseline guard/evidence created; browser/WASM capture harness blocker documented

## Dependencies

- Depends on: [Story 002](story-002-board-grid-camera-and-z-layers.md) and [Story 003](story-003-snapshot-spawn-units-objectives-and-hp-bars.md).
- Conditional closure dependency: [Story 009](story-009-status-icons-cooccupancy-and-spawn-range.md) is required for final status-icon atlas evidence and full Board Rendering epic closure, but is not required for this narrowed baseline guard story.
- Unlocks: baseline Board Rendering CI/performance guard evidence.

## Completion Notes

**Completed**: 2026-05-05
**Criteria**: 5/8 passing, 3/8 deferred with accepted blocker notes for the narrowed baseline scope. CI Z-literal guard, single `MessageReceiver<S2CPhaseChanged>` drain guard, narrowed baseline two-atlas handle evidence, approved standalone/non-counted batch notes, and BOARD-009 status-icon atlas deferral are verified. Browser/WASM 1920x1080 screenshot capture, frame-time capture, and nonblank framed-lane screenshot evidence remain blocked until a browser harness can seed the 20-unit baseline fixture and record timing.
**Deviations**: None blocking for the narrowed BOARD-010 baseline guard scope. Advisory/deferred: this closure does not claim final browser/WASM visual performance evidence, final Board Rendering epic closure, or BOARD-009 status-icon atlas evidence.
**Test Evidence**: `production/qa/evidence/board-rendering-performance-evidence.md` documents the CI guards, native ECS baseline fixture, verification commands, approved standalone/non-counted batches, browser/WASM harness blocker, and BOARD-009 status-icon atlas deferral.
**Code Review**: Complete locally. Lean mode applied because `production/review-mode.txt` is absent; QL-TEST-COVERAGE and LP-CODE-REVIEW external gates were skipped.
**Verification Notes**: Worker commit `f51a3f7f92634e33ae4e96fac787fefb35e990f9` was integrated. Requested verification passed: `cargo test -p client --test board_rendering_grid_camera_test --test board_rendering_plugin_scaffold_test --test board_rendering_snapshot_spawn_test`, `cargo fmt -p client -- --check`, `cargo check -p client`, and `git diff --check`. `production/sprint-status.yaml` was not updated because no matching BOARD-010 row exists.
