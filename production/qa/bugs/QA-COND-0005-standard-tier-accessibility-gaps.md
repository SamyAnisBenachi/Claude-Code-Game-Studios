# QA-COND-0005: Standard-Tier Accessibility Gaps

| Field | Value |
|---|---|
| ID | QA-COND-0005 |
| Kind | Accessibility Gap |
| Severity | S2 High |
| Priority | P4 Accepted risk condition |
| Status | Accepted Risk |
| Action State | Producer Waived |
| Reported | 2026-05-05 |
| Source | Accessibility requirements and Production-to-Polish gate check |

## Summary

The project originally targeted Standard-tier accessibility, but the producer
has waived the remaining Standard-tier accessibility scope for the Sprint 6
Production-to-Polish gate under friend-game scope. This is not verified
accessibility completion; it is an accepted-risk disposition for a non-public
build.

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

This condition is no longer an active P1 Sprint 6 blocker. On 2026-05-06, the
producer accepted the remaining Standard-tier accessibility exposure as risk
for friend-game scope:

> Friend-game scope - Standard-tier accessibility waived by producer. No
> public release, no obligation.

This does not mark Standard-tier accessibility as verified and does not remove
future accessibility debt. Any public, external, commercial, or broader release
candidate must revisit the remaining rows and either implement, evidence,
reclassify, or accept risk under that new release scope.

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
- 2026-05-06: SAU-011 automated ECS and Browser/WASM screenshot evidence added
  at
  `production/qa/evidence/shop-auction-ui-auction-bid-target-focus-2026-05-05.md`
  with captures under
  `production/qa/evidence/captures/shop-auction-ui-auction-bid-target-focus/`.
  It verifies A11Y-ST-12 target size, focus order, focus ring,
  disabled/unaffordable focus skip, affordability gating, one-send semantics,
  in-flight disable, `BIDDING...` state, and `YOU ARE LEADING` replacement
  behavior. This closes only the A11Y-ST-12 sub-gap and does not close
  QA-COND-0005.
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
- 2026-05-06: HUD-012 implemented and verified HUD-owned A11Y-ST-01 text-size
  evidence and HUD-owned A11Y-ST-03 contrast evidence. Evidence:
  `production/qa/evidence/hud-012-text-size-contrast-accessibility.md` with
  browser/WASM captures at `1366x768` and `1920x1080`, plus
  `cargo test -p client --test hud_text_size_contrast_accessibility_test` and
  HUD regression targets passing. This closes only the HUD-owned portions of
  A11Y-ST-01 and A11Y-ST-03; QA-COND-0005 remains Open because remaining
  Standard-tier rows and non-HUD / auction-price exposure still need evidence,
  reclassification, dependency-blocking, or accepted-risk disposition.
- 2026-05-06: HAND-UI-014 implemented and verified the A11Y-ST-14 PLACEMENT
  staged-disclosure sub-gap. Evidence:
  `production/qa/evidence/hand-ui-placement-staged-disclosure-accessibility-2026-05-05.md`
  with Browser/WASM captures for entry, card selection, lane/cell guidance,
  valid highlight, valid stage, reserve/current split adjustment, invalid
  submit, correction, and successful submit, plus the focused Hand UI
  accessibility test and requested regression targets passing. This closes only
  the A11Y-ST-14 sub-gap; QA-COND-0005 remains Open.
- 2026-05-06: SAU-012 implemented and verified the A11Y-ST-18 DRAFT_INITIAL
  clear-objective sub-gap. Evidence:
  `production/qa/evidence/shop-auction-ui-draft-initial-clear-objective-overlay-2026-05-05.md`
  with Browser/WASM captures for overlay entry, focused dismiss control, Esc
  dismissal, retrieval, reopened overlay, and non-occlusion/readability, plus
  focused Shop/Auction UI and regression targets passing. This closes only the
  A11Y-ST-18 sub-gap; QA-COND-0005 remains Open.
- 2026-05-06: A11Y-BS-03 photosensitivity warning and flash-frequency audit
  evidence added at
  `production/qa/evidence/accessibility-photosensitivity-warning-flash-audit-2026-05-05.md`.
  The warning is implemented in
  `client/src/ui/photosensitivity_warning.rs`, verified by
  `cargo test -p client --test accessibility_settings_photosensitivity_warning_test --jobs 1`,
  and registered through the presentation plugin regression target. The audit
  records objective destruction full-screen Prism White as exceeding or unable
  to prove the local max 3 flashes/sec rule; the row is closed as warning plus
  audit evidence, with scoped remediation or producer disposition still
  available before release if warning-only disposition is insufficient.
  QA-COND-0005 remains Open.
- 2026-05-06: Producer reclassified the remaining QA-COND-0005 Standard-tier
  accessibility exposure as accepted risk for friend-game scope only. Reason:
  "Friend-game scope - Standard-tier accessibility waived by producer. No
  public release, no obligation." This unblocks S6-06 for Sprint 6 gate
  execution, but does not verify Standard-tier accessibility and does not apply
  to any future public/external release candidate.

## Non-Goals

- Does not assign Sprint 6 capacity.
- Does not edit accessibility requirements.
- Does not implement accessibility features.
