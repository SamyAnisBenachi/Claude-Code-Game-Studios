# QA-COND-0004: Browser/WASM Board Performance Capture

| Field | Value |
|---|---|
| ID | QA-COND-0004 |
| Kind | Performance Gap |
| Severity | S2 High |
| Priority | P1 Sprint 6 gate blocker |
| Status | Closed |
| Action State | N/A - Closed |
| Reported | 2026-05-05 |
| Source | Board rendering performance evidence and Production-to-Polish gate check |

## Summary

QA-COND-0004 is resolved by the BOARD-012 browser/WASM capture. The committed
1920x1080 screenshot is nonblank and shows the narrowed board baseline, and the
corrected timing trace passes the browser RAF total-frame budget, ADR-021
steady-state presentation budget, and GDD `BR-RECONNECT-TIME` snapshot rebuild
budget.

## Source Evidence

- `production/qa/evidence/board-rendering-performance-evidence.md` originally
  recorded missing or failing browser/WASM performance evidence.
- `production/gate-checks/gate-production-polish-sprint-5-2026-05-05.md`
  marked browser/WASM board frame-time capture as a hard
  Production-to-Polish blocker.

## Expected Closure Evidence

Satisfied by browser/WASM evidence for the accepted board-rendering baseline:

- Seed or otherwise reproduce the 20-unit board baseline in a browser/WASM
  build.
- Capture a 1920x1080 visual screenshot proving the board renders as intended.
- Record frame timing against the retained budgets:
  - Total frame budget: `<= 16.67 ms`.
  - ADR-021 presentation steady-state budget: `< 1 ms`.
  - GDD `BR-RECONNECT-TIME` full snapshot rebuild budget: `<= 16.67 ms`.
- Preserve ADR-021 `< 3 ms` as the budget for true phase-boundary
  hide/show/cancel-tween presentation work; do not apply it to full snapshot
  rebuild.

## Closure Evidence

Captured 2026-05-05 from `D:\_DEV\claude-code-game-studios`.

- Evidence report:
  `production/qa/evidence/board-rendering-performance-evidence.md`.
- Screenshot:
  `production/qa/evidence/captures/board-rendering-baseline-1920x1080.png`.
- Timing trace:
  `production/qa/evidence/captures/board-rendering-baseline-timing.json`.
- Fixture counts: 5 lanes, 40 cells, 20 visible units, 10 objectives, 20 units
  with HP bars, and no status icons, spawn range highlights, ghost units, or
  lane ghost washes.
- Browser RAF sampler: 240 samples, max 6.0 ms, average 2.635 ms,
  `totalFrameBudgetPass=true`.
- ADR-021 steady-state presentation: max 0.2 ms, pass.
- GDD `BR-RECONNECT-TIME` full snapshot rebuild: 3.3 ms, pass.
- Bevy `Time<Real>` total-frame values are retained as diagnostics only:
  max 4.2 ms, average 1.388 ms in the committed trace.
- Corrected harness verdict: `board012BudgetPass=true`.
- Focused checks passed:
  `cargo test -p client --test board_rendering_browser_wasm_perf_harness_test`
  and
  `cargo check -p client --bin board_rendering_perf_harness --target wasm32-unknown-unknown`.

## Current Blocker Status

Closed. QA-COND-0004 is no longer a Sprint 6 P1 gate blocker after the
BOARD-012 browser/WASM capture verified the narrowed board rendering baseline
against the corrected timing budgets.

## Non-Goals

- Does not assign Sprint 6 capacity.
- Does not optimize or otherwise change board rendering behavior.
- Does not close BOARD-009 final status-icon evidence.
- Does not close the full Board Rendering epic.
