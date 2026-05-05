# QA-COND-0004: Browser/WASM Board Performance Capture

| Field | Value |
|---|---|
| ID | QA-COND-0004 |
| Kind | Performance Gap |
| Severity | S2 High |
| Priority | P1 Sprint 6 gate blocker |
| Status | Open |
| Action State | Needs Evidence |
| Reported | 2026-05-05 |
| Source | Board rendering performance evidence and Production-to-Polish gate check |

## Summary

Native ECS fixture evidence exists for the visible board path, but no browser/WASM
frame-time capture exists for the board-rendering baseline. The Production-to-
Polish gate identifies this as a hard blocker because performance has not been
measured against the browser/WASM frame budget.

## Source Evidence

- `production/qa/evidence/board-rendering-performance-evidence.md` states that
  no browser/WASM frame-time screenshot capture is claimed.
- `production/gate-checks/gate-production-polish-sprint-5-2026-05-05.md`
  marks performance as a concern and identifies missing browser/WASM frame-time
  capture as a hard Production-to-Polish blocker.

## Expected Closure Evidence

Capture browser/WASM evidence for the accepted board-rendering baseline:

- Seed or otherwise reproduce the 20-unit board baseline in a browser/WASM build.
- Capture a 1920x1080 visual screenshot or equivalent evidence proving the board
  renders as intended.
- Record frame timing against the retained budgets:
  - Total frame budget: `<= 16.67 ms`.
  - ADR-021 presentation steady-state budget: `< 1 ms`.
  - ADR-021 phase-boundary spike budget: `< 3 ms`.

## Current Blocker Status

This is a P1 Sprint 6 gate blocker for Production-to-Polish readiness until
browser/WASM performance evidence exists or the gate requirement is explicitly
reclassified.

## Non-Goals

- Does not assign Sprint 6 capacity.
- Does not implement a browser harness.
- Does not edit board-rendering code, tests, or performance budgets.
