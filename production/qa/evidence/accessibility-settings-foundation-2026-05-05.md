# Accessibility Settings Foundation Evidence - 2026-05-05

## Story

- Story: `production/epics/accessibility-settings/story-001-settings-accessibility-foundation-and-preferences.md`
- Branch: `work/accessibility-settings-001-foundation-and-preferences`
- Worktree: `D:\_DEV\claude-code-game-studios-worktrees\A11Y-SETTINGS-001`
- QA condition: `QA-COND-0005`
- Story-done verification: re-run from root `main` checkout aligned with `origin/main` on 2026-05-06

## Commands

| Command | Result |
|---|---|
| `cargo test -p client --test accessibility_settings_preferences_test` | PASS - 4 passed, 0 failed |
| `cargo test -p client --test accessibility_settings_shell_test` | PASS - 4 passed, 0 failed |
| `cargo test -p client --test accessibility_settings_timer_selector_test` | PASS - 4 passed, 0 failed |
| `cargo test -p client --test presentation_plugin_scaffold_test` | PASS - 5 passed, 0 failed |
| `cargo fmt -p client -- --check` | PASS |
| `cargo check -p client` | PASS |
| `git diff --check` | PASS |

## Story-Done Verification - 2026-05-06

The `/story-done` verification was re-run from the root checkout on `main`,
after `git fetch origin`, with `HEAD` aligned to `origin/main` before closure
documentation edits. No implementation code was changed during closure.

## Implemented Preference Fields

- `colorblind_mode`: defaults to `Off`; supports `Off`, `Protanopia`, `Deuteranopia`, and `Tritanopia`.
- `reduced_motion`: defaults to `false`.
- `placement_timer_multiplier_request`: defaults to `1x`; accepts only `1x`, `1.5x`, `2x`, and `3x`.
- `menu_ui_scale_percent`: defaults to `100`; clamps to `75..=150`; applies a tested Settings panel scale hook.
- `hud_ui_scale_percent`: defaults to `100`; clamps to `75..=150`; stored independently from menu scale.

## Storage Behavior

- Preferences serialize through the single namespaced key `lanes_and_lies.accessibility_preferences.v1`.
- The payload is versioned as `version: 1`.
- WASM builds use browser `localStorage` when available.
- Native/debug builds use the same preference resource with an in-memory storage backend.
- If storage is unavailable or a write fails, runtime values remain active and the Settings status footer reports a save warning.

## Timer Selector Boundary

- The Settings timer selector exposes exactly `1x`, `1.5x`, `2x`, and `3x`.
- `0.5x`, custom values, player IDs, requester names, and player-specific accessibility labels are not exposed.
- In `LOBBY` before `SessionReady`, changing the selector records one `C2SSetPlacementTimerMultiplier` intent for the selected value and updates local preference storage.
- After `SessionReady`, changing the selector updates only the next-session preference and does not emit a C2S timer request.
- The visible effective timer value is read from `SessionSettingsView`, which is populated from neutral server session settings or snapshot data.

## QA-COND-0005 Impact Statement

Story 001 reduces QA-COND-0005 risk by creating the Settings / Accessibility
preference foundation and timer selector UI. It does not close QA-COND-0005.
The bug remains Open until the remaining Standard-tier rows are implemented and
browser/WASM-evidenced, reclassified, or accepted as risk.
