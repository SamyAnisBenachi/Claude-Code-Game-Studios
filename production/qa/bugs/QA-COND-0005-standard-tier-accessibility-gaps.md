# QA-COND-0005: Standard-Tier Accessibility Gaps

| Field | Value |
|---|---|
| ID | QA-COND-0005 |
| Kind | Accessibility Gap |
| Severity | S2 High |
| Priority | P1 Sprint 6 gate blocker |
| Status | Open |
| Action State | Needs Remediation |
| Reported | 2026-05-05 |
| Source | Accessibility requirements and Production-to-Polish gate check |

## Summary

The project has committed to Standard-tier accessibility, but the requirements
document remains Draft and many Standard-tier rows are still Not Started. The
Production-to-Polish gate marks accessibility verification as a hard blocker.

## Source Evidence

- `design/accessibility-requirements.md` is Draft and targets Standard tier.
- `design/accessibility-requirements.md` lists multiple Standard-tier features
  as Not Started, including HUD/card text sizing, contrast verification, UI
  scaling, motion reduction, full input remapping, placement timer extension,
  cognitive supports, and visual backups.
- `production/gate-checks/gate-production-polish-sprint-5-2026-05-05.md`
  marks accessibility compliance verification as failed and identifies
  unverified accessibility as a hard Production-to-Polish blocker.

## Expected Closure Evidence

Provide all applicable evidence:

- Standard-tier implementation status is updated and no longer left as broad
  Not Started rows for committed requirements.
- Verification evidence confirms the committed Standard-tier requirements
  against the browser/WASM target.
- Any remaining gaps are explicitly reclassified or accepted as risks by the
  user or producer.

## Current Blocker Status

This is a P1 Sprint 6 gate blocker for Production-to-Polish readiness until
Standard-tier accessibility is remediated, verified, or explicitly reclassified.

## Non-Goals

- Does not assign Sprint 6 capacity.
- Does not edit accessibility requirements.
- Does not implement accessibility features.
