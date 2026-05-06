# HAND-UI-014 PLACEMENT Staged Disclosure Accessibility Evidence

| Field | Value |
|---|---|
| Story | HAND-UI-014: PLACEMENT Staged Disclosure Accessibility |
| QA row | A11Y-ST-14 |
| QA condition | QA-COND-0005 Standard-tier accessibility gaps |
| Source branch | `work/hand-ui-014-placement-staged-disclosure-accessibility` |
| Source base | `1b295b68158a24eeffc47acfc20ef6e233b9e996` plus HAND-UI-014 browser evidence repair |
| Evidence date | 2026-05-06 |
| Evidence status | Automated ECS evidence and Browser/WASM visual capture captured. `/story-done` was not run in this worker. |

## Implementation Evidence

The Hand UI PLACEMENT flow now exposes a stable presentation-only
`PlacementDisclosureState` and a pooled bevy_ui `PlacementDisclosureGuidance`
text entity. The state sequence is observable in automated tests:

- `CardSelection`: PLACEMENT entry; Submit and timer visible; no target guidance,
  target highlights, fan plate highlight, or reserve/current split controls.
- `TargetSelection { Minion }`: selected Minion card; drag sprite visible; guidance
  text says `Choose a lane and cell`; HU-12 valid cells are highlighted; split
  controls remain hidden.
- `StagedCard`: valid board target or Instant fan plate target confirmed; fan slot
  is `Ghost`; `GhostPlacementChanged` is written; Submit count increments; reserve
  strip appears only for cost > 0 staged cards.
- `Correction { ... }`: submit pre-validation failure keeps Submit active, attaches
  the existing `SubmitValidationError`, and shows correction guidance.
- `Submitted`: corrected submission sends exactly once, disables Submit, and shows
  the existing submitted checkmark.

Reserve/current split text now names both pools: `Reserve N Current M`. This makes
the split non-color-only while preserving the existing Story 011 clamp and ceiling
behavior.

## Automated Evidence

Command:

```text
cargo test -p client --test hand_ui_placement_staged_disclosure_accessibility_test
```

Result:

```text
6 passed; 0 failed
```

Coverage:

| Required capture | Automated evidence |
|---|---|
| PLACEMENT entry | `a11y_st_14_entry_exposes_only_card_selection_stage` |
| Card selected | `a11y_st_14_minion_selection_discloses_lane_cell_before_split_controls` |
| Lane/cell target guidance | `a11y_st_14_minion_selection_discloses_lane_cell_before_split_controls` |
| Valid target highlight | `a11y_st_14_minion_selection_discloses_lane_cell_before_split_controls` |
| Valid stage | `a11y_st_14_valid_stage_reveals_staged_guidance_and_split_text` |
| Instant stage | `a11y_st_14_instant_stage_uses_same_staged_disclosure_without_board_highlights` |
| Reserve/current split adjustment | `a11y_st_14_submit_correction_keeps_player_in_disclosure_flow` |
| Invalid submit | `a11y_st_14_submit_correction_keeps_player_in_disclosure_flow` |
| Correction and successful submit | `a11y_st_14_submit_correction_keeps_player_in_disclosure_flow` |
| Later controls hidden before their step | Entry, target-selection, and invalid-drop tests assert reserve strips remain hidden before staging. |

## Regression Evidence

Command:

```text
cargo test -p client --test hand_ui_placement_submit_core_test --test hand_ui_placement_drag_highlights_test --test hand_ui_placement_instant_staging_test --test hand_ui_placement_unstaging_test --test hand_ui_reserve_mana_strip_test --test hand_ui_submit_prevalidation_test --test hand_ui_placement_timer_test
```

Result:

```text
33 passed; 0 failed
```

Command:

```text
cargo check -p client
```

Result:

```text
PASS
```

Command:

```text
git -c safe.directory=D:/_DEV/claude-code-game-studios-worktrees/HAND-UI-014 diff --check
```

Result:

```text
PASS
```

## Browser/WASM Capture Evidence

Capture command:

```text
powershell -ExecutionPolicy Bypass -File production\qa\evidence\captures\hand-ui-placement-staged-disclosure\hand-ui-placement-staged-disclosure-capture.ps1
```

Capture metadata:

| Field | Value |
|---|---|
| Browser URL | `http://127.0.0.1:8081/` |
| Viewport | 1366x768 |
| UI scale | 100% |
| Input method | Deterministic mouse drag and click sequence |
| Capture tool | PowerShell Chrome DevTools Protocol + Trunk WASM harness |
| Browser | Chrome `147.0.7727.139` |
| Trace | `production/qa/evidence/captures/hand-ui-placement-staged-disclosure/hand-ui-placement-staged-disclosure-trace.json` |

The Browser/WASM harness compiles and runs
`hand_ui_placement_staged_disclosure_harness`, drives the real `HandUiPlugin`
PLACEMENT sequence, publishes the observed disclosure state to the browser page,
and captures each required stage as PNG evidence.

| Required capture | Browser/WASM artifact | Observed state |
|---|---|---|
| PLACEMENT entry | `production/qa/evidence/captures/hand-ui-placement-staged-disclosure/01-placement-entry.png` | `CardSelection`; Submit/timer visible; reserve strip hidden; no highlights. |
| Card selected | `production/qa/evidence/captures/hand-ui-placement-staged-disclosure/02-card-selected.png` | `TargetSelection(Minion)`; selected card/drag visible; target guidance visible. |
| Lane/cell target guidance | `production/qa/evidence/captures/hand-ui-placement-staged-disclosure/03-lane-cell-target-guidance.png` | Guidance says `Choose a lane and cell`; valid lane/cell set listed. |
| Valid target highlight | `production/qa/evidence/captures/hand-ui-placement-staged-disclosure/04-valid-target-highlight.png` | HU-12 valid cells highlighted while split controls remain hidden. |
| Valid stage | `production/qa/evidence/captures/hand-ui-placement-staged-disclosure/05-valid-stage.png` | `StagedCard`; fan slot `Ghost`; target lane 1 cell 1; Submit count is 1. |
| Reserve/current split adjustment | `production/qa/evidence/captures/hand-ui-placement-staged-disclosure/06-reserve-current-split-adjustment.png` | Split text shows `Reserve 1 Current 2`. |
| Invalid submit | `production/qa/evidence/captures/hand-ui-placement-staged-disclosure/07-invalid-submit.png` | `Correction(ManaOverdrawn)`; Submit remains active; outbound submissions remain 0. |
| Correction and successful submit | `production/qa/evidence/captures/hand-ui-placement-staged-disclosure/08-correction-successful-submit.png` | `Submitted`; split text shows `Reserve 3 Current 0`; outbound submissions is 1; checkmark visible. |

## QA-COND-0005 Impact

Story 014 implements and evidences A11Y-ST-14 for PLACEMENT staged disclosure. It
does not close QA-COND-0005 by itself. QA-COND-0005 remains Open until all
remaining Standard-tier rows are implemented and evidenced, reclassified,
dependency-blocked, or accepted as risk.
