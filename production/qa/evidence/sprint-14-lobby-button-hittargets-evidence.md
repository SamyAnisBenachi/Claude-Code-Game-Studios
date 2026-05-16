# Sprint 14 Lobby Button Hit-Targets Evidence

Story: `S11-UX-LOBBY-BUTTON-HITTARGETS` / Story 026

Branch: `work/s14-lobby-button-hit-targets`

## Diff Summary

- `client/src/ui/lobby.rs` now exposes named lobby dimension constants for room-code field, create button, join button, requested-slot buttons, class buttons, confirm CTA, room-code chip, slot panels, class cells, and portraits.
- Lobby spawn call sites now consume named constants instead of inline button-class `Val::Px(...)` literals.
- Requested-slot buttons changed from 72x30 to 80x30 so selected labels such as `Slot 3 *` fit at the canonical body font size.
- Button height remains 30 px. This is friend-game scope only and does not advance Standard-tier hit-target conformance.

## UX Consultation Note

The named CCGS `ux-designer` worker role was not available in this Codex toolset, so the worker performed the required UX pass locally under the prompt's "otherwise perform locally" instruction.

Chosen literals:

| Surface | Width | Height | Rationale |
| --- | ---: | ---: | --- |
| Room-code field | 100% | 30 px | Preserves existing full-width field and canonical lobby button height. |
| Create Room | 128 px | 30 px | Existing friend-game-tier width fits `Create Room`. |
| Join Room | 128 px | 30 px | Existing width fits `Join ABCDEFGH` for max room-code input. |
| Requested slot | 80 px | 30 px | Modest increase from 72 px so `Slot 3 *` fits at body font size. |
| Class buttons | 96 px | 30 px | Preserves story 025 class-picker contract and fits `Ecaflip *`. |
| Confirm CTA | 100% | 30 px | Preserves story 024 full-width CTA contract. |
| Room-code chip | 200 px | 40 px | Preserves existing chip dimensions. |
| Slot panels | 160 px | 48 px | Preserves existing PAW-006-b panel dimensions. |
| Portraits | 64 px | 80 px | Preserves story 025 portrait contract. |

## QA-COND-0005 Carve-Out

`docs/ux/ui-clean-pass-roadmap.md` explicitly preserves `QA-COND-0005` accept-risk on the `LOBBY_BUTTON_HEIGHT = 30.0` defect (PROMPT 802 section 3.1 L5). This story does **not** advance `QA-COND-0005`. Standard-tier accessibility conformance - including but not limited to **>=44 Px hit-target minimums** - is **out of scope**.

This evidence makes no claim of >=44 Px Standard-tier compliance.

This story does **not** claim: public release readiness, release-candidate readiness, full game completion, broad / Standard-tier accessibility completion (`QA-COND-0005`), playtest / fun-hypothesis validation (`QA-COND-0006`), full playable-client manual QA, two-client `GAME_OVER` closure (`S8-QA-001-W1`), final-art / asset-production completion (`PAW-TD-*-a`), or `Polish->Release` gate-check retry.

## Paired Story Preservation

- Story 024 (`S12-UX-LOBBY-LAYOUT-MODAL-001`): confirm CTA remains full-width and the centered modal panel constants are untouched.
- Story 025 (`S11-UX-LOBBY-CLASS-PICKER`): class-picker cell, portrait, grid, and class-button dimensions are still named and stable.

## ADR Preservation

- ADR-002 preserved: no client-side authority, protocol, shared, or server change.
- ADR-021 preserved: presentation-only Bevy UI composition and dimension constants.

## Automated Evidence

Commands run under the required Windows/MSVC Cargo policy:

```text
$env:CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'
$env:CARGO_PROFILE_DEV_DEBUG='0'
$env:CARGO_PROFILE_TEST_DEBUG='0'
$env:CARGO_INCREMENTAL='0'
$env:RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'
```

Results:

| Command | Result | Notes |
| --- | --- | --- |
| `cargo fmt --all -- --check` | PASS | Formatting clean. |
| `cargo check --workspace --all-targets` | PASS | One existing warning: `count_with_image_node` dead code in `tests/integration/presentation/hand_ui_asset_wiring_test.rs`. |
| `cargo test -p client --test playable_client_lobby_button_dimensions_test -- --nocapture` | PASS | 4 passed, 0 failed. |
| `cargo test -p client --test playable_client_lobby_layout_viewport_invariant_test --test playable_client_lobby_class_picker_layout_test --test playable_client_lobby_entry_test --test playable_client_lobby_confirm_state_text_test --test playable_client_native_operator_controls_test -- --nocapture` | PASS | Adjacent lobby regressions passed. |

## Visual Capture Limitation

No pixel screenshots were captured in this worker run. Coverage is via ECS node-dimension sampling and analytical text-fit checks at the canonical lobby dimensions; visual capture remains the residual manual evidence gap for 1920x1080 and 1366x768.

## Friend-Game Scope Restatement

This story is the friend-game-scope-only repair: it locks canonical button dimensions and asserts dimension stability across rebuild so the lobby button hit-zones do not surprise the player. It does **not** raise the button heights to Standard-tier 44 Px. The chosen friend-game-tier literals remain near the current 30-40 Px range, and `QA-COND-0005` remains accept-risk.
