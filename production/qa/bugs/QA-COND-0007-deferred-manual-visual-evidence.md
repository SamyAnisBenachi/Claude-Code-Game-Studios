# QA-COND-0007: Deferred Manual Visual Evidence

| Field | Value |
|---|---|
| ID | QA-COND-0007 |
| Kind | Evidence Gap |
| Severity | S3 Medium |
| Priority | P2 Sprint 6 validation |
| Status | Closed |
| Action State | N/A - Closed |
| Reported | 2026-05-05 |
| Source | Sprint 5 QA sign-off and Production-to-Polish gate check |

## Summary

Manual and visual QA evidence was deferred for several player-visible
presentation paths: placement timer urgency/checkmark, reserve strip affordance,
submit validation inline feedback, and resolution replay readability. The
condition is now closed by focused visual/harness evidence for all listed paths.

## Source Evidence

- `production/qa/qa-signoff-sprint-5-2026-05-05.md` records visual/manual
  evidence deferred for placement timer urgency/checkmark, reserve strip
  affordance, submit validation inline feedback, and later resolution replay
  readability.
- `production/gate-checks/gate-production-polish-sprint-5-2026-05-05.md`
  carries deferred visual/manual evidence forward as an open QA condition.

## Expected Closure Evidence

Provide manual or visual QA evidence for:

- Placement timer urgency and checkmark feedback.
- Reserve strip affordance.
- Submit validation inline feedback.
- Resolution replay readability.

Each evidence item should include the tested build or commit, environment,
steps, expected result, actual result, and screenshot or capture reference when
visual confirmation is required.

## Closure Evidence

Captured and documented 2026-05-06.

- Hand UI evidence report:
  `production/qa/evidence/deferred-visual-manual-sprint-6-2026-05-06.md`.
- Hand UI capture root:
  `production/qa/evidence/captures/qa-cond-0007-hand-ui/`.
- Resolution replay evidence report:
  `production/qa/evidence/qa-cond-0007-resolution-replay-readability-2026-05-06.md`.
- Resolution replay capture root:
  `production/qa/evidence/captures/qa-cond-0007-resolution-replay/`.
- Resolution replay trace:
  `production/qa/evidence/captures/qa-cond-0007-resolution-replay/qa-cond-0007-resolution-replay-trace.json`.
- Resolution replay screenshots:
  `01-replay-start.png`, `02-replay-mid-first-sub-step.png`,
  `03-replay-second-sub-step.png`, `04-replay-final-sub-step-buffered.png`,
  `05-replay-drained-next-phase.png`, and
  `06-recovery-snapshot-requested.png`.

## Current Blocker Status

Closed. QA-COND-0007 is no longer an open P2 validation condition after the
Hand UI evidence and resolution replay readability evidence cover every listed
deferred path.

## Infrastructure Impact

2026-05-06 BR-006 story-done verified that the integrated board-rendering
resolution replay queue and phase-buffering infrastructure is in place. The
resolution replay evidence pass now captures that infrastructure in a focused
visual/harness flow and verifies result progression, phase buffering/no
premature phase jump, and out-of-range replay recovery.

## Non-Goals

- Does not assign Sprint 6 capacity.
- Does not edit UI, HUD, board rendering, hand UI, or resolution code.
- Does not claim full playable-client manual QA.
