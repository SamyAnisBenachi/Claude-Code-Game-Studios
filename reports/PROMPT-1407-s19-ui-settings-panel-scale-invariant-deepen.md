# PROMPT 1407 — S19-UI-SETTINGS-PANEL-SCALE-INVARIANT-DEEPEN-001

## Status

1407: S19-UI-SETTINGS-PANEL-SCALE-INVARIANT-DEEPEN-001: DONE

## Summary

PROMPT 1396 surfaced V-P1-09: the settings panel was migrated to flex
in PROMPT 1187 but inner control rows still carried fixed-px width and
height. At 75 % UI scale the panel shrank to ~570 px while header/footer
chrome stayed at their 100 % pixel sizes (136 px back-close, 124 px
footer close, 170 px category gutter); at 150 % the panel grew to
~1140 px but every inner control retained 100 % pixel size, so labels
collided with tight padding and the visual proportions broke.

This deepening introduces a `SettingsScaledDimensions` component
recording per-node base width / height. The existing
`sync_settings_shell_visibility_system` already re-applied the menu
factor to the panel itself; that loop now also walks every node
carrying `SettingsScaledDimensions` and re-applies `base × factor` each
frame. The status footer is intentionally NOT scaled — its width is
flex-grown inside the SpaceBetween footer row, and a new test guards
that contract from regression.

## Owned changes

- `client/src/ui/settings/mod.rs`:
  - New `pub struct SettingsScaledDimensions { base_width_px,
    base_height_px }` Component (line 482).
  - `sync_settings_shell_visibility_system` gains a second query
    `Query<(&mut Node, &SettingsScaledDimensions), (Without<SettingsPanel>,
    Without<SettingsMenuScaleApplied>)>` that re-applies the menu factor
    to every tagged inner row/control (line 962+).
  - Spawn-time tagging: back-close button, footer close button, category
    button, category column wrapper, colorblind selector, reduced motion
    toggle, four timer chips, effective timer display, menu scale
    control, HUD scale control all gain `SettingsScaledDimensions`.
  - Seven small helper constructors at the bottom of the file
    (`back_close_scaled_dimensions`, `footer_close_scaled_dimensions`,
    `category_scaled_dimensions`, `category_column_scaled_dimensions`,
    `control_scaled_dimensions`, `timer_option_scaled_dimensions`,
    `effective_timer_scaled_dimensions`).
  - Panel `width`, `min_width`, `max_width`, `max_height`, `height`,
    `min_height`, `SettingsMenuScaleApplied` are preserved unchanged so
    the existing PROMPT 1187 / PROMPT 1180 contracts still hold.

- `tests/integration/accessibility_settings/ui_scale_invariant_test.rs`:
  - Import `SettingsScaledDimensions`.
  - `test_settings_inner_controls_scale_with_menu_ui_scale_at_75_and_150_percent`
    — sweeps 75 / 100 / 150 % and asserts every scaled inner control's
    `node.width` / `node.height` equals `base × factor`.
  - `test_settings_inner_controls_do_not_use_position_absolute_at_75_or_150`
    — guards the flex-layout contract under extreme scales.
  - `test_settings_timer_options_row_total_width_fits_panel_at_75_percent`
    — proves the four scaled chips fit inside the scaled content-pane
    budget at 75 %, so the `flex_wrap` fallback stays the exception.
  - `test_settings_status_footer_keeps_intrinsic_layout_at_extreme_scales`
    — locks in that the status footer is NOT scaled (flex-grow stays
    in charge).
  - All seven pre-existing tests retained verbatim and still pass.

## Tests run

Cargo target dir overridden to a worker-private `D:\_DEV\cargo-target\ccgs-msvc-1407`
to avoid a `cargo-target\ccgs-msvc\debug\deps\client-*.exe` lock that
prevented an in-place rebuild on the shared MSVC target.

| Test target | Result |
|---|---|
| `accessibility_settings_ui_scale_invariant_test` | 12 passed / 0 failed |
| `accessibility_settings_shell_test` | 4 passed / 0 failed |
| `accessibility_settings_timer_selector_test` | 4 passed / 0 failed |
| `accessibility_settings_preferences_test` | 4 passed / 0 failed |
| `accessibility_settings_photosensitivity_warning_test` | 5 passed / 0 failed |

`cargo check -p client` was also run earlier; client lib compiles
with 88 warnings (all pre-existing deprecation noise from
`ShopAuctionUiEntity`), no errors from owned files.

## Windows / MSVC Cargo policy

Applied before every cargo invocation in this worktree:

```
$env:CARGO_TARGET_DIR        = 'D:\_DEV\cargo-target\ccgs-msvc-1407'
$env:CARGO_PROFILE_DEV_DEBUG = '0'
$env:CARGO_PROFILE_TEST_DEBUG = '0'
$env:CARGO_INCREMENTAL       = '0'
$env:RUSTFLAGS               = '-C debuginfo=0 -C link-arg=/DEBUG:NONE'
```

## Worktree

- Worktree: `D:\_DEV\claude-code-game-studios-worktrees\s19-ui-settings-panel-scale-invariant-deepen-1407`
- Branch: `work/s19-ui-settings-panel-scale-invariant-deepen-1407`
- Base: `origin/main@426d9b8` (latest at task start)
- Commit: see git log on worker branch after commit step

## Out-of-scope (deliberately not touched)

- hand / shop / lobby / HUD / board / qa_snapshot / sprint / session /
  status paperwork.
- Font sizes (`typography::BODY` / `typography::CAPTION`) — the prompt
  scoped this to row/control dimensions; font scaling is a separate
  accessibility deepening.

## Final status line

1407: S19-UI-SETTINGS-PANEL-SCALE-INVARIANT-DEEPEN-001: DONE
