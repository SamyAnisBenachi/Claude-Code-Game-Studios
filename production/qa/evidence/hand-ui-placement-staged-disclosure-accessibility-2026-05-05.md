# HAND-UI-014 PLACEMENT Staged Disclosure Accessibility Evidence

| Field | Value |
|---|---|
| Story | HAND-UI-014: PLACEMENT Staged Disclosure Accessibility |
| QA row | A11Y-ST-14 |
| QA condition | QA-COND-0005 Standard-tier accessibility gaps |
| Source branch | `work/hand-ui-014-placement-staged-disclosure-accessibility` |
| Source base | `a87f19b` |
| Evidence date | 2026-05-06 |
| Evidence status | Automated ECS evidence captured. Browser/WASM visual capture still required before `/story-done`. |

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

## Browser/WASM Capture Status

Browser/WASM screenshot capture was not run in this dev-story pass. Required
manual or browser-automated capture before `/story-done`:

- Browser build/source identifier, viewport size, UI scale, input method, and
  whether mouse drag, keyboard focus, or both were used.
- PLACEMENT entry capture showing Submit, timer, selectable fan cards, no active
  target guidance, no board/fan target highlight, and no visible split controls.
- Card-selected capture showing selected/dragged card, visible target guidance,
  valid target highlights, and no split controls.
- Lane/cell target capture showing the selected board target step and valid target
  set, including lane and cell context.
- Valid-stage capture showing fan ghost state, board ghost or Instant fan ghost,
  Submit count increment, and split controls visible only after staging.
- Reserve/current split adjustment capture showing `+` and `-`, disabled increment
  at ceiling, reserve spend, and current spend as text.
- Invalid-submit capture showing no outbound submit, Submit still active, and
  inline correction guidance.
- Correction capture showing split adjustment or un-stage followed by exactly one
  successful submit, `Submitted` text, and submitted checkmark.

## QA-COND-0005 Impact

Story 014 implements and evidences A11Y-ST-14 for PLACEMENT staged disclosure. It
does not close QA-COND-0005 by itself. QA-COND-0005 remains Open until all
remaining Standard-tier rows are implemented and evidenced, reclassified,
dependency-blocked, or accepted as risk.
