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

## Partial Evidence Updates

- 2026-05-05: PLACEMENT timer-extension sub-gap implemented and verified by
  GSS-008. Evidence:
  `production/qa/evidence/gss-008-placement-timer-multiplier-authority-2026-05-05.md`.
  This does not close QA-COND-0005 as a whole; all remaining Standard-tier
  accessibility gaps stay Open for later evidence, remediation, reclassification,
  or accepted-risk disposition.
- 2026-05-05: Sprint 6 Standard-tier accessibility disposition created at
  `production/qa/evidence/accessibility-standard-tier-sprint-6-2026-05-05.md`.
  It records QA-COND-0005 as Open, marks only the PLACEMENT timer-extension
  sub-gap as implemented/verified via GSS-008, and lists the remaining
  Standard-tier rows that still block closure.
- 2026-05-05: S6-04 disposition register completed at
  `production/qa/evidence/accessibility-standard-tier-sprint-6-2026-05-05.md`.
  Every source row now has exactly one allowed final disposition. QA-COND-0005
  remains Open because rows still require evidence, Sprint 6 implementation,
  blocked-dependency follow-up, or producer decisions before closure. No
  accepted-risk or reclassification signoff is recorded in this update.
- 2026-05-05: SAU-011 automated ECS evidence added at
  `production/qa/evidence/shop-auction-ui-auction-bid-target-focus-2026-05-05.md`
  for A11Y-ST-12 target size, focus order, focus ring, disabled/unaffordable
  focus skip, `BIDDING...` state, and `YOU ARE LEADING` replacement behavior.
  Browser/WASM visual screenshot evidence is deferred until a playable or
  renderable Shop/Auction shell exists. This does not claim A11Y-ST-12 visual
  closure and does not close QA-COND-0005.
- 2026-05-06: HUD-011 implemented and verified the A11Y-ST-13 current/reserve
  mana shape-distinction sub-gap. Evidence:
  `production/qa/evidence/hud-011-mana-shapes-evidence.md` with browser/WASM
  color and grayscale captures at `1366x768` and `1920x1080`, plus
  `cargo test -p client --test hud_mana_shape_distinction_test` and
  `cargo test -p client --test hud_gold_mana_display_test` passing. This closes
  only the A11Y-ST-13 sub-gap; QA-COND-0005 remains Open.
- 2026-05-06: A11Y Settings 001 implemented and verified the Settings /
  Accessibility foundation, preference storage fallback, safe/unsafe settings
  entry, keyboard-operable shell markers, and multiplayer-safe PLACEMENT timer
  selector UI. Evidence:
  `production/qa/evidence/accessibility-settings-foundation-2026-05-05.md`.
  This clears only the A11Y-DEP-01/A11Y-DEP-02 foundation dependency evidence
  for future accessibility rows; QA-COND-0005 remains Open.

## Non-Goals

- Does not assign Sprint 6 capacity.
- Does not edit accessibility requirements.
- Does not implement accessibility features.
