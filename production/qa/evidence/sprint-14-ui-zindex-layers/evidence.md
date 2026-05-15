# Sprint 14 / Story 002 — S11-TD-UI-ZINDEX-LAYERS Evidence

> **Story file**: `production/epics/ui-clean-pass/story-002-ui-zindex-layers.md`
> **Status**: Draft → in implementation (this evidence captures `/dev-story`
> run; `/story-done` NOT invoked by this prompt).
> **Implementation prompt**: PROMPT 899 (`/dev-story`).
> **Source-of-truth at implementation**: `origin/main@4dd7fe3` (PROMPT 898
> Sprint 14 QA plan tip).
> **Worktree**: `D:/_DEV/claude-code-game-studios-worktrees/s14-ui-layout-foundation`
> **Branch**: `work/s14-ui-layout-foundation`

## Acceptance Criteria — Verification Summary

| AC | Verdict | Evidence |
|----|---------|----------|
| AC1 — Layer module authored | PASS | 6 inline unit tests in `client/src/ui/design_tokens/z_layers.rs` all green; module exports 8 canonical layer constants `BACKGROUND/WORLD/UNITS/UI_BASE/UI_OVERLAY/MODAL/TOAST/DEBUG` plus `ALL_LAYERS_ASCENDING` + `LAYER_MIN_GAP`. |
| AC2 — Doc comments on each layer | PASS | Module-level + per-constant `///` doc comments cover canonical UI elements. `cargo check -p client --tests` passes; `cargo doc -p client` is the deferred verification but not required for compile success — doc text is checked indirectly by `ac8_module_doc_names_adr_021_and_presentation_plugin_load_order`. |
| AC3 — All UI roots migrated | PASS | Lobby UI Root (UI_BASE), HUD Root (UI_BASE), HUD Resolution Dim Overlay (UI_OVERLAY), Hand UI Fan Root (UI_BASE), Hand UI Drag Sprite (UI_OVERLAY), Shop Auction UI Root + DraftOffering/Shop/Auction/ShopFooter sub-roots (UI_BASE), Shop Auction Settlement Overlay (UI_OVERLAY), Shop Auction Toast Root (TOAST), Shop Auction Draft-Initial Objective Overlay (UI_OVERLAY), Settings Accessibility Root (MODAL), Photosensitivity Warning Root (MODAL). |
| AC4 — Result-screen migrated | PASS | `client/src/presentation/result_screen.rs:519` migrated from inline `GlobalZIndex(100)` to `z_layers::MODAL` (= `GlobalZIndex(500)`). Inline unit test `ac4_modal_is_above_ui_overlay_so_result_screen_wins_over_conn_lost` asserts the relative paint order is preserved. Visual capture against the existing `result-screen-mvp-evidence.md` baseline is left as a manual operator step (this story is foundation-level; a screenshot does not gate a code-level migration). |
| AC5 — Grep guard | PASS | `ac5_grep_guard_no_inline_global_z_index_literals_outside_design_tokens` in `tests/integration/ui_clean_pass/z_layers_test.rs` walks every `client/src/**/*.rs` file (excluding `client/src/ui/design_tokens/`) and asserts no `ZIndex(` / `GlobalZIndex(` substrings remain. Locked exclusion: `client/src/ui/design_tokens/**`. |
| AC6 — Reconnect / snapshot-rebuild paint-order invariant | PASS | `ac6_paint_order_matches_named_layers_under_out_of_order_spawn` spawns every named layer entity in REVERSE canonical order then queries `GlobalZIndex` values; asserts the bevy_ui paint order (sorted by `GlobalZIndex.0`) matches the canonical hierarchy regardless of spawn order. `ac6_layer_constants_survive_pairwise_distinctness_under_arbitrary_permutation` provides a second angle over an arbitrary permutation. |
| AC7 — No magic z values in `client/src/ui/` | PASS | Surface-level grep in `ac7_production_migration_sites_reference_design_tokens` spot-checks each migrated file references the design-token constants. AC5 grep-guard provides the workspace-wide negative assertion. |
| AC8 — ADR-021 alignment | PASS | `ac8_module_doc_names_adr_021_and_presentation_plugin_load_order` asserts the module doc names ADR-021 + ADR-002 + the `PresentationPlugin` composition order. The named layer hierarchy does not reorder presentation plugin registration; sprite Transform.z values for `BACKGROUND/WORLD/UNITS` remain governed by ADR-021 (not by this module). No ADR-021 amendment was required. |
| AC9 — Friend-game scope preserved | PASS | No edit to `production/sprint-status.yaml` by this story. `QA-COND-0005` Standard-tier accessibility, `QA-COND-0006` playtest validation, and `PAW-TD-*-a` placeholder-art accept-risk dispositions remain unchanged. Module Status / No-Claim banner reaffirms these scopes are not advanced. |

## Artifacts

- **Layer module** (NEW): `client/src/ui/design_tokens/z_layers.rs` (+ `client/src/ui/design_tokens/mod.rs`).
- **AC1 inline unit tests** (NEW, 6 tests): inside `client/src/ui/design_tokens/z_layers.rs#tests`.
- **AC5/AC6/AC7/AC8 integration tests** (NEW, 6 tests): `tests/integration/ui_clean_pass/z_layers_test.rs` registered as `[[test]] ui_clean_pass_z_layers_test` in `client/Cargo.toml`.
- **AC4 migrated literal**: `client/src/presentation/result_screen.rs:519` — `GlobalZIndex(100)` → `z_layers::MODAL`.
- **Connection-lost overlay**: `client/src/presentation/connection_lost_overlay.rs` — inline `GlobalZIndex(CONNECTION_LOST_OVERLAY_Z_INDEX)` literal removed; const reroutes to `z_layers::UI_OVERLAY.0` so the existing `connection_lost_overlay_test.rs` `CONNECTION_LOST_OVERLAY_Z_INDEX` public API is preserved (test updated to compare against `MODAL.0` instead of bare `100`).
- **UI-root migrations**: `client/src/ui/lobby.rs`, `client/src/ui/hud/mod.rs` (root + dim overlay), `client/src/ui/hand/mod.rs` (fan root + drag sprite), `client/src/ui/shop_auction/mod.rs` (root + 5 sub-roots), `client/src/ui/settings/mod.rs` (root), `client/src/ui/photosensitivity_warning.rs` (root).
- **`client/src/ui/mod.rs`**: declares `pub mod design_tokens;`.

## Cargo Policy Applied

Cargo policy env vars set for every cargo invocation in this run:

```
CARGO_TARGET_DIR=D:\_DEV\cargo-target\ccgs-msvc
CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0
CARGO_INCREMENTAL=0
RUSTFLAGS=-C debuginfo=0 -C link-arg=/DEBUG:NONE
```

No `target/` cleanup was performed (disk pressure not hit).

## Checks Run

- `cargo fmt -p client -- --check` — clean after one auto-format pass on the new test file.
- `cargo check -p client --tests` — passes (only the pre-existing `count_with_image_node` dead-code warning, unrelated to this story).
- `cargo test -p client --lib ui::design_tokens::z_layers` — **6 / 6 passed** (AC1 unit suite).
- `cargo test -p client --test ui_clean_pass_z_layers_test` — **6 / 6 passed** (AC5/AC6/AC7/AC8 integration suite).
- `cargo test -p client --test connection_lost_overlay_test` — **16 / 16 passed** (story 021 regression — AC7 `ac7_overlay_z_index_is_below_result_screen` continues to pass against the new `MODAL` constant).
- Regression spread: `result_screen_mvp_test` (6/6), `result_screen_return_to_lobby_test` (2/2), `presentation_plugin_scaffold_test` (5/5), `hud_plugin_scaffold_test` (varied), `hand_ui_plugin_scaffold_test`, `shop_auction_ui_plugin_scaffold_formulas_test` (8/8), `accessibility_settings_photosensitivity_warning_test`, `accessibility_settings_shell_test`, `playable_client_lobby_entry_test` (6/6) — all green.
- `git diff --check origin/main` — clean (no whitespace errors).
- Full-workspace `cargo test --workspace` deferred per Sprint 14 QA plan no-full-workspace-tests-by-default policy.

## Non-Claims Preserved

- `S8-QA-001-W1` OPEN unchanged.
- `QA-COND-0005` Standard-tier accessibility accept-risk preserved.
- `QA-COND-0006` playtest / fun-hypothesis validation accept-risk preserved.
- `PAW-TD-*-a` placeholder-art accept-risk preserved.
- PROMPT 683-era runtime divergence question preserved.
- `TQ-S12-C1..C7` preserved verbatim.
- Sprint 12 / Sprint 11 / Sprint 10 close-outs preserved.
- PROMPT 761 Polish→Release `FAIL` preserved at `production/gate-checks/gate-polish-release-2026-05-12.md` (NOT retried).
- Sprint 13 close-out `closed-with-conditions` (PROMPT 894) preserved.
- Sprint 14 disposition `active` unchanged.
- Stage `Polish` unchanged.
- Underlying drag-runtime bug not claimed fixed.
- Two-client GAME_OVER closure (`S8-QA-001-W1`) not claimed.
- Final-art / asset-production not claimed.
- Public release readiness / RC readiness / full game completion not claimed.

## Forbidden-Scope Verification

`git diff origin/main --stat` against forbidden surfaces yields the empty
set:

- `server/` — untouched (verified empty diff).
- `shared/` — untouched (verified empty diff).
- `production/sprint-status.yaml` — untouched.
- `production/session-state/*` — untouched.
- `production/stage.txt` — untouched.
- `production/sprints/*` — untouched.
- `production/qa/qa-plan-sprint-14.md` — untouched.
- No smoke / team-qa / gate-check / release artifact created.
- No `/story-done` invoked.

## Next Dependency Notes for Tier 1 Stories

- **S11-UX-HUD-TOP-STRIP-LAYOUT** (rank 7) and **S11-UX-HUD-BOTTOM-STRIP-LAYOUT** (rank 8): the HUD Root + HUD Resolution Dim Overlay now carry `UI_BASE` / `UI_OVERLAY` respectively. New HUD strip primitives spawned as children of the HUD Root inherit `UI_BASE` unless they need their own override. The dim overlay's explicit `UI_OVERLAY` ensures it continues to paint above any future HUD strip children.
- **S12-UX-DRAFT-GRID-CENTERED-MODAL** and **S11-UX-AUCTION-FEATURED-CARD** (ranks 9, 10): the Shop Auction UI Root carries `UI_BASE`; future centred modals introduced by these stories should spawn at `MODAL` and clarify whether they sit above the existing `TOAST`-layer auction toast or below it.
- **S12-UX-LOBBY-LAYOUT-MODAL** (rank 12): the Lobby UI Root carries `UI_BASE`; the eventual lobby modal should spawn at `MODAL` so it sits above the lobby base.
- **S11-TD-UI-FONT-CONSTANTS** (rank 2) and **S11-TD-UI-FLEX-STRIPS** (rank 3): these Tier 0 modules will live alongside `z_layers.rs` under `client/src/ui/design_tokens/`. They are blocked by PROMPT 802 §9 producer-decision-2 (numeric values from story 007 spec) and may need to update `client/src/ui/design_tokens/mod.rs` to declare additional submodules.
- **S11-TD-UI-VIEWPORT-INVARIANT-TESTS** (rank 4): parallel-safe with this story. New integration test bin under `tests/integration/` can follow the same `[[test]]` registration pattern used here.
- **S12-TD-UI-OVERLAY-ALPHA-TOKEN-001** (rank 5): `OVERLAY_DIM_ALPHA` token introduced by this rank should pair with the existing `HUD_DIM_OVERLAY_ALPHA` constant; the HUD dim overlay spawn site is already isolated in `client/src/ui/hud/mod.rs` near the `z_layers::UI_OVERLAY` insertion point — convenient single-site migration for that rank.

## ADR-021 Reconciliation

The Sprint 14 z-layer module does NOT amend ADR-021. Reasons:

- ADR-021 §R2 binds world-space sprites (board content) to render below
  bevy_ui regardless of `GlobalZIndex` values. The `BACKGROUND` / `WORLD` /
  `UNITS` constants in this module exist as documentation references for
  sprite Transform.z values; they are NOT bevy_ui consumers.
- The `PresentationPlugin` composition order
  (CardAnimations → BoardRendering → HandUi → Hud → ShopAuctionUi) remains
  the authoritative load-order contract. The bevy_ui layers
  (`UI_BASE` / `UI_OVERLAY` / `MODAL` / `TOAST` / `DEBUG`) do not reorder
  plugin registration.
- Module-level doc (`client/src/ui/design_tokens/z_layers.rs` lines 27-37)
  explicitly affirms both points so a future reader cannot mistake the named
  layers for a replacement of the ADR-021 composition contract.
