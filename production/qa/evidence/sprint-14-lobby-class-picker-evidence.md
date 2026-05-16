# Sprint 14 Lobby Class-Picker Evidence

## No-Claim Banner

- Sprint 14 remains active.
- Stage remains Polish.
- PROMPT 761 Polish->Release gate-check remains FAIL and was not retried.
- `S8-QA-001-W1` remains OPEN.
- `QA-COND-0005` and `QA-COND-0006` remain accepted-risk.
- `PAW-TD-*-a` remains accepted-risk.
- No Standard-tier accessibility completion, public release readiness, release-candidate readiness, full game completion, broad accessibility completion, playtest validation, final-art completion, full playable-client manual QA, Sprint 14 close-out, or `S11-UX-LOBBY-BUTTON-HITTARGETS` closure is claimed.

## Implementation Summary

- `client/src/ui/lobby.rs` replaces the prior independent Class label, portrait wrap row, and button wrap row with one `LobbyClassPickerBlock`.
- The block owns one `LobbyClassPickerHeading` and one no-wrap `LobbyClassPickerGrid`.
- Each selectable class from `lobby_class_options()` is represented by one `LobbyClassPickerCell` containing both its `LobbyClassPortrait` and its `LobbyClassButton`.
- `LobbyClassButton` and `LobbyClassPortrait` marker semantics are preserved.
- `refresh_lobby_ui_system` now updates class-cell background and border colors from `LobbyInputState.selected_class` so selection affordance refreshes without respawning the lobby.
- No server, shared, protocol, or class-lock authority path changed.

## UX-Designer Consultation

No callable external `ux-designer` agent was available in this worker window, so the consultation was performed locally against `docs/ux/global-ui-design-spec.md`, `docs/ux/ui-clean-pass-roadmap.md`, and Story 025.

Locked choices:

- Pairing pattern: heading -> fixed grid -> cell; each selectable cell stacks portrait over the selectable button for the same `ClassId`.
- Selected-cell affordance: selected selectable cell uses an accent border matching `#F2C94C` plus a warmer dark surface. Non-selected selectable cells keep the neutral lobby surface. Neutral is muted and non-selectable.
- Grid columns: 7 fixed columns, no wrap, centered in the Sprint 14 lobby modal panel at both `1366x768` and `1920x1080`.
- Cell dimensions: `108x132` px, `6` px padding, `8` px grid gap.
- Portrait dimensions: `64x80` px.
- Button width: `96` px, preserving existing `LOBBY_BUTTON_HEIGHT = 30.0`.
- Font sizes: heading uses `typography::H3` (`18` px), selectable class buttons use `typography::BODY` (`15` px), Neutral caption uses `typography::CAPTION` (`13` px). This keeps the Class heading above per-cell labels and avoids silent ellipsis.

## ClassId Reconciliation

`lobby_class_options()` and `lobby_all_class_ids()` intentionally differ:

- Selectable options: `Iop`, `Cra`, `Sacrier`, `Xelor`, `Ecaflip`, `Sadida`.
- Portrait set: the six selectable options plus `Neutral`.

Implementation reconciliation:

- The six selectable options each get a paired portrait/button cell.
- `Neutral` remains visible as a seventh portrait cell for existing portrait coverage and visual completeness.
- `Neutral` does not get a `LobbyClassButton` and is marked `selectable: false`.
- No client-side class-lock authority or optimistic lock state was added. `S2CClassLocked` remains server-authoritative.

## Test Evidence

All Cargo commands used the Windows/MSVC low-debug policy:

`CARGO_TARGET_DIR=D:\_DEV\cargo-target\ccgs-msvc`
`CARGO_PROFILE_DEV_DEBUG=0`
`CARGO_PROFILE_TEST_DEBUG=0`
`CARGO_INCREMENTAL=0`
`RUSTFLAGS="-C debuginfo=0 -C link-arg=/DEBUG:NONE"`

Passed:

- `cargo fmt --all`
- `cargo fmt --all -- --check`
- `cargo test -p client --test playable_client_lobby_class_picker_layout_test -- --nocapture` -> 5 passed, 0 failed.
- `cargo test -p client --test playable_client_lobby_layout_viewport_invariant_test -- --nocapture` -> 12 passed, 0 failed.
- `cargo test -p client --test playable_client_lobby_entry_test -- --nocapture` -> 6 passed, 0 failed.
- `cargo test -p client --test playable_client_lobby_confirm_state_text_test -- --nocapture` -> 5 passed, 0 failed.
- `cargo test -p client --test lobby_chrome_wiring_test -- --nocapture` -> 5 passed, 0 failed.
- `cargo test -p client --test lobby_asset_wiring_test -- --nocapture` -> 7 passed, 0 failed.
- `git diff --check` -> passed.
- `git diff --cached --check` -> passed.

The minimal Bevy asset fixtures emit expected `AssetServer` loader errors for PNG handles while the tests assert UI handle wiring and ECS marker placement. These errors did not fail the tests.

Runtime browser PNG captures were not produced in this headless worker window and are not claimed here. The new integration test verifies the class-picker hierarchy, grid intent, viewport fit at `1366x768` and `1920x1080`, text-fit estimate, stable cell dimensions, and selected-cell refresh without respawn.

## Acceptance Criteria Summary

- AC1 PASS: one `LobbyClassPickerBlock` owns the Class heading and grid.
- AC2 PASS: each selectable `ClassId` pairs portrait and button in one cell; Neutral reconciliation is documented above.
- AC3 PASS: the grid is locked to 7 no-wrap columns and fits both `1366x768` and `1920x1080` panel content widths.
- AC4 PASS: fixed cell and button dimensions plus the label-width estimate prevent overlap and silent ellipsis.
- AC5 PASS: repeat lobby spawns preserve cell width and height within 1 px.
- AC6 PASS: selected cell has first-spawn affordance and refreshes from `LobbyInputState` without respawn.
- AC7 PASS: new integration test covers class-picker hierarchy, bounds intent, viewport fit, and row-wrap divergence prevention.
- AC8 PASS: UX consultation choices are recorded above.
- AC9 PASS: no client-side class-lock authority, server change, shared protocol change, or Lightyear protocol change.
- AC10 PASS: targeted and adjacent lobby UI tests listed above pass.
- AC11 PASS: implementation commit does not edit shared sprint trackers.
- AC12 PASS: no-claim banner is restated at the top of this evidence file.

## Cargo File Note

`client/Cargo.toml` was touched only to register the new external integration test target:

`playable_client_lobby_class_picker_layout_test`

No dependency, feature, profile, workspace, or lockfile change was introduced.

## Forbidden Path Review

Expected changed paths:

- `client/src/ui/lobby.rs`
- `client/Cargo.toml`
- `tests/integration/playable_client/lobby_class_picker_layout_test.rs`
- `production/qa/evidence/sprint-14-lobby-class-picker-evidence.md`

Forbidden paths not modified:

- `server/`
- `shared/`
- `production/sprint-status.yaml`
- `production/sprints/`
- `production/session-state/`
- `production/stage.txt`
- `production/qa/qa-plan-sprint-14.md`
- PROMPT 761 gate-check artifact
