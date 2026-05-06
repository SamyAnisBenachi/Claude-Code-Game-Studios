# QA-COND-0007: Deferred Manual Visual Evidence

| Field | Value |
|---|---|
| ID | QA-COND-0007 |
| Kind | Evidence Gap |
| Severity | S3 Medium |
| Priority | P2 Sprint 6 validation |
| Status | Open |
| Action State | Needs Evidence |
| Reported | 2026-05-05 |
| Source | Sprint 5 QA sign-off and Production-to-Polish gate check |

## Summary

Manual and visual QA evidence remains deferred for several player-visible
presentation paths: placement timer urgency/checkmark, reserve strip affordance,
submit validation inline feedback, and resolution replay readability. Sprint 5
QA accepted this as a condition, but the evidence remains open for future visual
validation.

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

## Current Blocker Status

This is not a Sprint 5 close-out blocker. It is a Sprint 6 validation condition
until the deferred manual/visual checks are evidenced or explicitly reclassified.

## Infrastructure Impact

2026-05-06 BR-006 story-done verified that the integrated board-rendering
resolution replay queue and phase-buffering infrastructure is in place. This
supports future resolution replay readability evidence capture, but it does not
close QA-COND-0007, does not satisfy the missing manual/visual readability
evidence, and does not claim playable-client QA.

## Non-Goals

- Does not assign Sprint 6 capacity.
- Does not create visual captures.
- Does not edit UI, HUD, board rendering, hand UI, or resolution code.
