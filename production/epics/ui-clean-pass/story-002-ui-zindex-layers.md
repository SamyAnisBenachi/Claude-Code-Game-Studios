# Story 002: S11-TD-UI-ZINDEX-LAYERS -- Centralised UI Z-Index Layer Constants

> **Epic**: UI Clean-Pass
> **Story ID**: S11-TD-UI-ZINDEX-LAYERS
> **Status**: **Done** (PROMPT 903 `/story-done`, 2026-05-15; verdict **PASS**;
> source-of-truth `origin/main@36c0b4b9a45e5a27dfcf60c69e584dc3cd249405` =
> PROMPT 902 `--no-ff` integration of PROMPT 899 worker tip `8669982`;
> closure paperwork landed on `origin/main` after PROMPT 909 + PROMPT 908
> Sprint 14 `/story-done` closures via PROMPT 903 reconcile run on top of
> `origin/main@b39eedf05e3f0825775b6aae4aff8028f531fbc6`)
> **Layer**: Presentation / UX foundational tech-debt
> **Type**: Tech Debt -- foundational primitive
> **Sprint**: Sprint 14 (Tier 0 rank 1 Must Have; activated by PROMPT 897
> 2026-05-15; closed by PROMPT 903 2026-05-15).
> **Authored**: 2026-05-14 by PROMPT 878
> **Authoring source-of-truth**: `origin/main@51e6228` (PROMPT 871 `/story-done`
> on Sprint 13 row `S13-TWO-CLIENT-RUNTIME-HARNESS-001`)
> **Implemented**: 2026-05-15 by PROMPT 899 (`/dev-story` worker, branch
> `work/s14-ui-layout-foundation` tip `8669982`)
> **Integrated**: 2026-05-15 by PROMPT 902 (`--no-ff` merge; integration
> commit `36c0b4b9a45e5a27dfcf60c69e584dc3cd249405`)
> **Closed**: 2026-05-15 by PROMPT 903 (`/story-done`, paperwork-only)
> **Estimated effort**: ~1.0d (PROMPT 802 §4 Tier 0.1)

---

## Status / No-Claim Banner

This story is authored as a Sprint 14 candidate. **Sprint 14 is NOT activated
by this authoring run.** The story is paperwork only -- no code change is
attempted by PROMPT 878.

PROMPT 878 (this authoring run) does NOT:

- Activate Sprint 14.
- Modify `production/sprint-status.yaml`.
- Modify `production/sprints/sprint-13.md`, `production/sprints/sprint-14.md`,
  or any other sprint plan file.
- Modify `production/stage.txt` (remains `Polish`).
- Modify any `production/session-state/*` file.
- Modify any QA-plan / smoke / Team-QA / gate-check / release-check artifact.
- Run `/story-readiness`, `/dev-story`, `/story-done`, `/smoke-check`,
  `/team-qa`, `/gate-check`, `/release-check`, or `/qa-plan` on this story.
- Modify any code under `client/`, `server/`, `shared/`, or `tests/`.

This story does **not** claim: public release readiness, release-candidate
readiness, full game completion, broad / Standard-tier accessibility
completion (`QA-COND-0005`), playtest / fun-hypothesis validation
(`QA-COND-0006`), full playable-client manual QA, two-client GAME_OVER
closure (`S8-QA-001-W1`), or final-art / asset-production completion.

---

## Overview

PROMPT 802 §3.9 G1 surfaced that `client/src/ui/` contains **zero** uses of
`ZIndex` or `GlobalZIndex` across 226 `Node{}` / `Style{}` usages spanning 6
files. The only surface in the audit that declares an explicit z-index is
`client/src/presentation/result_screen.rs:512` (`GlobalZIndex(100)`), which
PROMPT 802 §3.8 also called out as the lone "acceptable" UI surface.

Every overlay, modal, drag ghost, toast, dim, and settlement panel in the
current client UI relies on spawn-order for paint order. That is fragile under
reconnect / snapshot rebuild / late-message recovery / replay: any code path
that respawns UI roots out of their initial order silently breaks the visual
stack. PROMPT 802 §5 sequencing rule names this work as the refactor that must
land **first** because every Tier 1 surface story (HUD strip layout, draft
centered modal, auction featured card, lobby class-picker, lobby layout modal,
hand drag-state visuals) depends on having a named layer to spawn into rather
than relying on inline `GlobalZIndex` literals or spawn-order.

This story introduces a named-layer enum + const module so all UI roots and
overlays declare their layer by name, eliminating magic z values across the
playable client.

---

## Scope

### In Scope

- A new design-token module (likely `client/src/ui/design_tokens/z_layers.rs`,
  exact path TBD by the worker) that exports a named `UiLayer` enum or const
  module covering at minimum the following layer order (lowest to highest):
  `Background`, `World`, `Units`, `UiBase`, `UiOverlay`, `Modal`, `Toast`,
  `Debug`. Each layer maps to a stable `GlobalZIndex` integer with sufficient
  gap (e.g. multiples of 100) to allow future intermediate layers without
  re-ordering existing values.
- A doc comment on each layer explaining the canonical UI elements expected
  there (e.g. `UiOverlay` = dim overlays, settlement scrim, draft-initial
  objective overlay; `Modal` = centered panels; `Toast` = transient
  notifications).
- Migration of all existing UI roots in `client/src/ui/` (lobby, HUD top-strip,
  HUD bottom-strip, hand cards, shop panel, draft offering panel, auction
  panel, shop footer, settlement overlay, toast) to declare their layer via
  the new module rather than relying on spawn-order.
- Migration of the existing explicit `GlobalZIndex(100)` in
  `client/src/presentation/result_screen.rs:512` to the new module's `Modal`
  (or equivalent) layer constant, preserving the existing visual stack.
- A grep guard or compile-time pattern (TBD by the worker) that prevents
  reintroduction of inline `ZIndex(N)` / `GlobalZIndex(N)` literals outside
  the design-token module.

### Out of Scope

- **No Sprint 14 activation** by this story. The story exists only as a
  candidate for future Sprint 14 pull-in.
- **No public release readiness** work. Z-layer constants land as a Polish-stage
  foundational primitive; they do not advance the `Polish->Release` gate-check
  retry.
- **No Standard-tier accessibility** (`QA-COND-0005`) completion. Layer
  constants do not address hit-target sizes, keyboard navigation, screen
  reader support, colorblind modes, text scaling, or WCAG contrast ratios.
- **No final-art / asset-production** work (`PAW-TD-*-a`). Layer constants are
  layout / composition primitives; placeholder PNG replacement is a separate
  scope.
- **No playtest validation** (`QA-COND-0006`). Layer constants do not produce
  playtest evidence.
- **No re-design of the layer hierarchy itself**. The named layers listed
  above are the canonical set per PROMPT 802 §3.9 G1; intermediate layers can
  be added later without re-ordering, but the canonical set is fixed.
- **No spacing / typography / overlay-alpha** token work. Those are separate
  stories (002 = z-index only; 003 = font constants; 005 = viewport-invariant
  tests; 006 = overlay alpha; 007 = global UI design spec).
- **No interaction-state primitive** work (hover / focus / pressed / disabled).
  That is `S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001` (Tier 0 Should-priority
  adjacent row, NOT in this story).
- **No `client/src/presentation/board_rendering.rs` sprite z-order** change.
  Board sprite z-order is governed by ADR-021 `PresentationPlugin` composition
  order, not by `bevy_ui` `GlobalZIndex`.

---

## Acceptance Criteria

All criteria are independently checkable BLOCKING criteria.

- [x] **AC1 -- Layer module authored** -- **PASS** (PROMPT 903 verification at
  `origin/main@36c0b4b9`): `client/src/ui/design_tokens/z_layers.rs:50-87`
  exports 8 named constants `BACKGROUND=GlobalZIndex(0)`, `WORLD=100`,
  `UNITS=200`, `UI_BASE=300`, `UI_OVERLAY=400`, `MODAL=500`, `TOAST=600`,
  `DEBUG=700` with `ALL_LAYERS_ASCENDING` table at `:92-101` and
  `LAYER_MIN_GAP=10` at `:106`. Inline tests
  `ac1_layer_constants_are_strictly_ascending`,
  `ac1_layer_constants_have_minimum_gap_for_future_intermediates`,
  `ac1_layer_constants_are_pairwise_distinct`,
  `ac1_named_set_covers_at_least_eight_canonical_layers`,
  `ac1_canonical_layer_ordering_matches_story_spec` at `:112-180` all pass.
  Ordering `Background < World < Units < UiBase < UiOverlay < Modal < Toast <
  Debug` holds.

- [x] **AC2 -- Doc comments on each layer** -- **PASS** (PROMPT 903
  verification): `client/src/ui/design_tokens/z_layers.rs:1-46` carries
  module-level doc naming ADR-021 + ADR-002 + canonical surfaces table; per-
  constant `///` doc at `:50-87` names canonical UI elements expected at each
  layer. Module text indirectly verified by
  `ac8_module_doc_names_adr_021_and_presentation_plugin_load_order` at
  `tests/integration/ui_clean_pass/z_layers_test.rs:228-252`. `cargo doc -p
  client` deferred per evidence doc but not required for compile success.

- [x] **AC3 -- All UI roots migrated** -- **PASS** (PROMPT 903 verification):
  `client/src/ui/lobby.rs:894` `z_layers::UI_BASE`; `client/src/ui/hud/mod.rs:517`
  `UI_BASE` + `:636` `UI_OVERLAY` (HUD dim overlay); `client/src/ui/hand/mod.rs:2813`
  `UI_BASE` + `:2932` `UI_OVERLAY` (drag sprite); `client/src/ui/shop_auction/mod.rs:3446/3464/3490/3506/3531`
  `UI_BASE` (root + DraftOffering + Shop + Auction + ShopFooter sub-roots),
  `:3480/3551` `UI_OVERLAY` (settlement + draft-initial objective overlays),
  `:3540` `TOAST` (toast root); `client/src/ui/settings/mod.rs:562` `MODAL`;
  `client/src/ui/photosensitivity_warning.rs:76` `MODAL`. Surface-level grep
  spot-check via `ac7_production_migration_sites_reference_design_tokens` at
  `z_layers_test.rs:195-226`; workspace-wide negative assertion via AC5 grep
  guard.

- [x] **AC4 -- Result-screen migrated** -- **PASS** (PROMPT 903 verification):
  `client/src/presentation/result_screen.rs:520` now reads `z_layers::MODAL`
  (resolves to `GlobalZIndex(500)`) replacing the prior inline
  `GlobalZIndex(100)`. Inline test
  `ac4_modal_is_above_ui_overlay_so_result_screen_wins_over_conn_lost` at
  `client/src/ui/design_tokens/z_layers.rs:182-196` asserts `MODAL.0 >
  UI_OVERLAY.0` so the result screen continues to win over the connection-lost
  overlay on GameOver. Connection-lost overlay also migrated:
  `client/src/presentation/connection_lost_overlay.rs:33`
  `CONNECTION_LOST_OVERLAY_Z_INDEX = z_layers::UI_OVERLAY.0`,
  `:210` overlay root insert reads `z_layers::UI_OVERLAY`. Existing
  `tests/integration/playable_client/connection_lost_overlay_test.rs`
  `ac7_overlay_z_index_is_below_result_screen` test updated to assert against
  `MODAL.0` instead of bare `100` and continues to pass (16/16 green).
  Visual capture against `production/qa/evidence/captures/result-screen-*`
  baseline left as a manual operator step per evidence doc -- foundation-level
  migration does not gate on a screenshot for a layer-name swap that
  preserves the relative integer ordering.

- [x] **AC5 -- Grep guard or lint** -- **PASS** (PROMPT 903 verification):
  `ac5_grep_guard_no_inline_global_z_index_literals_outside_design_tokens` at
  `tests/integration/ui_clean_pass/z_layers_test.rs:69-99` walks every
  `client/src/**/*.rs` file excluding `client/src/ui/design_tokens/` and
  asserts zero `ZIndex(` / `GlobalZIndex(` substrings. Manual grep at
  PROMPT 903 closure shows the only hits in `client/src/ui/design_tokens/z_layers.rs:52/57/61/66/72/77/82/87`
  (the constant declarations themselves) plus `:184/185` (doc-comment
  historical references inside the AC4 inline test docstring). All
  production sites under `client/src/ui/` + `client/src/presentation/` go
  through the `z_layers::*` module path.

- [x] **AC6 -- Reconnect / snapshot-rebuild invariant** -- **PASS** (PROMPT
  903 verification): `ac6_paint_order_matches_named_layers_under_out_of_order_spawn`
  at `tests/integration/ui_clean_pass/z_layers_test.rs:101-167` spawns every
  named layer entity in REVERSE canonical order, queries `GlobalZIndex`
  values, sorts by `.0`, and asserts the resulting paint order matches
  `ALL_LAYERS_ASCENDING` canonical sequence (precondition asserts the spawn
  order really was reversed so the test cannot trivially pass against an
  accidentally-canonical spawn). `ac6_layer_constants_survive_pairwise_distinctness_under_arbitrary_permutation`
  at `:169-193` provides a second angle over an arbitrary permutation. Both
  tests pass against the integration tip.

- [x] **AC7 -- No magic z values remain in `client/src/ui/`** -- **PASS**
  (PROMPT 903 verification): AC5 grep guard provides the workspace-wide
  negative assertion; `ac7_production_migration_sites_reference_design_tokens`
  at `tests/integration/ui_clean_pass/z_layers_test.rs:195-226` spot-checks
  12 migration sites including lobby / hud / hand / shop_auction / settings /
  photosensitivity / result_screen / connection_lost_overlay reference
  `z_layers::UI_BASE` / `UI_OVERLAY` / `MODAL` / `TOAST` appropriately.

- [x] **AC8 -- ADR-021 alignment** -- **PASS** (PROMPT 903 verification):
  `ac8_module_doc_names_adr_021_and_presentation_plugin_load_order` at
  `tests/integration/ui_clean_pass/z_layers_test.rs:228-252` asserts the
  module doc names ADR-021 + `PresentationPlugin` composition order +
  ADR-002. Module-level doc at `client/src/ui/design_tokens/z_layers.rs:27-38`
  explicitly affirms ADR-021 §R2 binds world-space sprites to render below
  bevy_ui regardless of `GlobalZIndex` values; the `BACKGROUND` / `WORLD` /
  `UNITS` constants are conceptual references for sprite Transform.z and
  NOT direct bevy_ui consumers; the `PresentationPlugin` composition order
  `CardAnimations -> BoardRendering -> HandUi -> Hud -> ShopAuctionUi`
  remains the authoritative load-order; ADR-002 client-server authority is
  preserved -- no optimistic client-side authority is introduced. ADR-021
  amendment NOT required.

- [x] **AC9 -- Friend-game scope preserved** -- **PASS** (PROMPT 903
  verification): `git diff 36c0b4b9^1..36c0b4b9 -- 'production/sprint-status.yaml'`
  empty across worker + integration commits; `QA-COND-0005` Standard-tier
  accessibility, `QA-COND-0006` playtest validation, and `PAW-TD-*-a`
  placeholder-art accept-risk dispositions preserved unchanged. Module-level
  doc at `client/src/ui/design_tokens/z_layers.rs:40-46` explicitly affirms
  friend-game scope boundary preservation. PROMPT 903's row-level flip is
  the permitted disposition-preserving paperwork edit and does NOT touch any
  accept-risk field.

---

## Evidence Path

`production/qa/evidence/sprint-14-ui-foundation/ui-zindex-layers/`

Expected artifacts:

- Integration test output for AC6 reconnect / snapshot-rebuild invariant.
- Visual capture comparison for AC4 (result-screen pre/post).
- Grep-guard output for AC5.
- Layer-ordering unit test output for AC1.

---

## Likely Files Touched

| Path | Expected change |
|------|-----------------|
| `client/src/ui/design_tokens/z_layers.rs` (NEW; exact path TBD by worker) | Author named-layer enum / const module. |
| `client/src/ui/design_tokens/mod.rs` (NEW or extended) | Re-export the layer constants. |
| `client/src/ui/mod.rs` | Declare `design_tokens` submodule. |
| `client/src/ui/lobby.rs` | Replace any spawn-order-implicit z with named layer. |
| `client/src/ui/hud/mod.rs` | Replace HUD-root z reliance with named layer. |
| `client/src/ui/hand/mod.rs` | Replace card-root z reliance with named layer. |
| `client/src/ui/shop_auction/mod.rs` | Replace panel-root z reliance with named layer (shop, draft offering, auction, settlement overlay, toast). |
| `client/src/presentation/result_screen.rs` | Migrate inline `GlobalZIndex(100)` to named `Modal` constant. |
| `tests/integration/ui_zindex_layers_test.rs` (NEW) | AC6 reconnect / respawn invariant test. |
| `tests/unit/ui_zindex_layers_test.rs` (NEW) | AC1 ordering / distinctness unit test. |
| `docs/architecture/adr-021-presentation-layer-architecture.md` | Possible amendment if reconciliation requires it (AC8). |

This table is a planning estimate. The implementation prompt is authoritative
for the realised set.

---

## Verification

- `cargo test -p client --lib ui_zindex` (or equivalent) -- AC1 unit test.
- `cargo test -p client --test ui_zindex_layers_test` -- AC6 integration test.
- `cargo doc -p client` -- AC2 doc coverage.
- `rg "ZIndex\(|GlobalZIndex\(" client/src/ --glob '!client/src/ui/design_tokens/**'`
  -- AC5 grep guard.
- Visual capture against `production/qa/evidence/captures/result-screen-baseline-*`
  -- AC4 result-screen invariant.

---

## Dependencies / Sequencing

- **Authoring prompt (this PROMPT 878)** is the *story-authoring* prompt; it
  creates the story file only. No `/dev-story` runs here.
- **Activation**: Requires Sprint 14 activation (separate prompt; not this
  one). Cannot land in Sprint 13.
- **Tier 0 internal sequencing** (per PROMPT 802 §8 and roadmap §3
  "Sequencing Rules"): this story is **mostly serial** with stories 003
  (font constants), 004 (flex strips), and 006 (overlay alpha token) because
  all four touch the shared design-token host module (`client/src/ui/design_tokens/`).
  Story 005 (viewport-invariant tests) is parallel-safe with this story
  (new test bin).
- **Tier 0 design-spec dependency** (per roadmap §3 rule 2): story 007
  (`S12-UX-GLOBAL-UI-DESIGN-SPEC-001`) should be authored *first* in Phase 1
  because Tier 0 token modules need its numeric values as input. However,
  for *z-layers specifically*, the layer ordering is structural rather than
  numeric, so this story can land slightly before the spec is finalized;
  the spec authoring should then ratify the chosen integer values.
- **Tier 1 surface stories depend on this**: PROMPT 802 §5 names z-index as
  the foundational refactor that must land first. Future Tier 1 stories
  (HUD top-strip, HUD bottom-strip, draft centered modal, auction featured
  card, lobby class-picker, lobby layout modal) all expect to spawn into a
  named layer.

---

## Notes

- PROMPT 802 §3.9 G1 source: zero `ZIndex` / `GlobalZIndex` hits in
  `client/src/ui/`; the sole exception is `result_screen.rs:512`
  (`GlobalZIndex(100)`).
- PROMPT 802 §3.2 H3 specifically calls out the HUD dim overlay as fragile
  under reconnect / snapshot rebuild / late-message recovery.
- The 8-layer canonical set comes from PROMPT 878's task brief and is
  consistent with PROMPT 802 §3.9 G1's "z-index layer system" recommendation.
- ADR-021 (Presentation Layer Architecture) governs the `PresentationPlugin`
  composition order; named UI layers must remain consistent with that ADR.
- Accept-risk preservation: `PAW-TD-*-a` (placeholder art), `QA-COND-0005`
  (Standard-tier accessibility, friend-game scope only), `QA-COND-0006`
  (playtest validation) remain preserved unchanged. This story does not
  advance any of them.

---

## Authoring / Implementation / Closure Trail

- **PROMPT 878** (2026-05-14): authored this story file as a Sprint 14
  candidate (Tier 0 rank 1). Source-of-truth at authoring:
  `origin/main@51e6228` (PROMPT 871 `/story-done` on Sprint 13 row
  `S13-TWO-CLIENT-RUNTIME-HARNESS-001`). NOT activated by PROMPT 878.

- **PROMPT 893** (2026-05-15): integrated the four Sprint 14 story-authoring
  branches via four sequential `--no-ff` merges (commits `9f36663` +
  `2d8eaac` + `2bdb277` + `466d3d4`) -- this story file landed on
  `origin/main` via the first merge `9f36663`.

- **PROMPT 897** (2026-05-15): activated Sprint 14 (`origin/main@fffaf1c`).
  Row entered `production/sprint-status.yaml` with `status: ready` under
  `must_have_rows`; `sprint_14_activation:` block snapshot recorded
  `roadmap_rank: 1`. Stage UNCHANGED `Polish`. Sprint 13 disposition
  `closed-with-conditions` preserved.

- **PROMPT 898** (2026-05-15): authored
  `production/qa/qa-plan-sprint-14.md` covering all 17 Sprint 14 rows
  including this row. No-full-workspace-tests-by-default policy + binding
  Windows/MSVC Cargo resource policy enumerated.

- **PROMPT 899** (2026-05-15): `/dev-story` worker on branch
  `work/s14-ui-layout-foundation` (worker tip `8669982`). Authored
  `client/src/ui/design_tokens/{mod.rs, z_layers.rs}` (z_layers.rs ~197 LOC
  with 8 named layer constants + 6 inline AC1 unit tests). Migrated 12 UI
  roots across `lobby.rs`, `hud/mod.rs` (root + dim overlay), `hand/mod.rs`
  (fan root + drag sprite), `shop_auction/mod.rs` (root + 5 sub-roots +
  settlement + draft-initial + toast), `settings/mod.rs`,
  `photosensitivity_warning.rs`. Migrated `result_screen.rs:519`
  `GlobalZIndex(100)` -> `z_layers::MODAL`. Migrated
  `connection_lost_overlay.rs` `CONNECTION_LOST_OVERLAY_Z_INDEX` const to
  `z_layers::UI_OVERLAY.0` and updated existing connection-lost overlay test
  to assert against `MODAL.0`. Authored
  `tests/integration/ui_clean_pass/z_layers_test.rs` (6 integration tests
  covering AC5/AC6/AC7/AC8) registered in `client/Cargo.toml`. Cargo policy
  applied per Sprint 14 QA plan. `cargo fmt -p client -- --check` clean;
  `cargo check -p client --tests` passes; `cargo test -p client --lib
  ui::design_tokens::z_layers` 6/6 pass; `cargo test -p client --test
  ui_clean_pass_z_layers_test` 6/6 pass; `cargo test -p client --test
  connection_lost_overlay_test` 16/16 pass; regression spread (result_screen
  / presentation_plugin_scaffold / accessibility / hand / hud / shop_auction
  / lobby_entry) all green. Full-workspace `cargo test` deferred per QA
  plan. Authored evidence document
  `production/qa/evidence/sprint-14-ui-zindex-layers/evidence.md` (118
  lines). Pushed worker branch only; did NOT push main.

- **PROMPT 902** (2026-05-15): `/integrate` `--no-ff` merge of worker tip
  `8669982` into prior `origin/main@4dd7fe3`, producing integration commit
  `36c0b4b9a45e5a27dfcf60c69e584dc3cd249405` on
  `integrate/s14-ui-layout-foundation-902`. Zero conflicts. 15 files / +672
  / -16 lines. Worker reachable as merge's second parent. Pushed integration
  branch + fast-forward to `origin/main`. AC1-AC9 verified PASS at the
  integration tip.

- **PROMPT 903** (2026-05-15): `/story-done` paperwork-only closure of
  this row (1 row). Source-of-truth at AC verification:
  `origin/main@36c0b4b9a45e5a27dfcf60c69e584dc3cd249405` (PROMPT 902
  integration tip). Closure paperwork landed on top of
  `origin/main@b39eedf05e3f0825775b6aae4aff8028f531fbc6` (PROMPT 908 tip,
  the third Sprint 14 paperwork closure to land via this reconcile run --
  PROMPT 909 viewport-invariant-tests landed first at commit `4a7f72e`;
  PROMPT 908 font-constants landed second at commit `b39eedf`; PROMPT 903
  z-index-layers landing as the third Sprint 14 `/story-done` entry).
  Worktree: `D:/_DEV/wt/ccgs-prompt-903-reconcile` (fresh detached on
  `origin/main@b39eedf` because root checkout was behind `origin/main` and
  had unrelated dirt; the original PROMPT 903 closure commit `6934e01`
  was authored against `origin/main@36c0b4b9` but never pushed, so it was
  discarded and recreated here). AC1-AC9 verdict **PASS** verified by
  reading the integrated module + tests + migration sites + diff against
  forbidden surfaces at `36c0b4b9` (the binding source-of-truth for AC
  evidence; the four `--no-ff` integration merges PROMPT 906 + 907 and the
  two paperwork closures PROMPT 908 + 909 that landed between `36c0b4b9`
  and `b39eedf` touch disjoint surfaces from this row and do not change
  any AC1-AC9 evidence). Allowed-files write set: this story file (Status
  header + AC checkboxes + Closure Trail) + `production/sprint-status.yaml`
  (row flip + top-level `updated:` annotation refresh + new entry in
  `sprint_14_story_done:` block appended as the **third** Sprint 14
  `/story-done` entry after PROMPT 909 + PROMPT 908) +
  `production/session-state/active.md` (PROMPT 903 banner prepended above
  PROMPT 908 banner) +
  `production/session-state/codex-orchestrator-state.md` (PROMPT 903
  section prepended above PROMPT 909 section). Cargo policy N/A by
  PROMPT 903 itself (paperwork-only). Expected worker report at
  `reports/PROMPT-899-S14-UI-Layout-Foundation-Dev-Story.md` + integration
  report at `reports/PROMPT-902-S14-UI-LAYOUT-FOUNDATION-Integration.md`
  NOT present on disk and NOT in any git tree; documented as **non-blocking**
  per PROMPT 884 / 891 precedent because integration commit-message body +
  worker commit-message body + the evidence document collectively cover all
  nine ACs with concrete file:line references + test names + diff
  verifications. A reconcile report
  `reports/PROMPT-903-S14-UI-LAYOUT-FOUNDATION-STORY-DONE-RECONCILE.md`
  documents the discard of the never-pushed `6934e01` commit and the
  recreation against latest `origin/main`.

### Conditions carried forward unchanged

- `S8-QA-001-W1` manual/browser two-client GAME_OVER gap remains OPEN.
  Story 017 AC12 forbid-auto-closure preserved.
- `QA-COND-0005` Standard-tier accessibility remains accepted-risk
  (friend-game scope only).
- `QA-COND-0006` playtest / fun-hypothesis validation remains
  accepted-risk / deferred.
- `PAW-TD-*-a` placeholder-art accept-risk preserved across
  PAW-002..PAW-006.
- PROMPT 683-era runtime divergence question preserved; third same-scope
  retest NOT authorised per `TQ-S12-C2`.
- PROMPT 761 Polish->Release gate-check `FAIL` preserved at
  `production/gate-checks/gate-polish-release-2026-05-12.md`. NO retry.
- Sprint 12 story 019 underlying drag-runtime bug NOT claimed fixed.
- `TQ-S12-C1..C7` preserved verbatim.
- Sprint 13 close-out `closed-with-conditions` (PROMPT 894); Sprint 12 / 11
  / 10 closeouts; all 16 prior Sprint 13 `/story-done` closures preserved
  unchanged on `origin/main`.
- Sprint 14 activation snapshot under `sprint_14_activation:` block
  preserved unchanged.
- Sprint 14 QA plan `production/qa/qa-plan-sprint-14.md` (PROMPT 898)
  preserved unchanged.
- PROMPT 909 (`/story-done S11-TD-UI-VIEWPORT-INVARIANT-TESTS`) closure
  entry preserved verbatim as the first `sprint_14_story_done:` list item
  on `origin/main`.
- PROMPT 908 (`/story-done S11-TD-UI-FONT-CONSTANTS`) closure entry
  preserved verbatim as the second `sprint_14_story_done:` list item on
  `origin/main`.
- `S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001` backlog row preserved.
- `S11-HUD-TIMER-EYEBALL-VISUAL-001` Sprint 13 carry preserved (Sprint 14
  Should Have, status: ready, human-operator-blocked, no LLM `/story-done`
  authorised).

### Explicitly NOT claimed by PROMPT 903

- public release readiness; release-candidate readiness; full game
  completion; broad / Standard-tier accessibility completion; playtest /
  fun-hypothesis validation; full playable-client manual QA; two-client
  GAME_OVER closure (`S8-QA-001-W1`); final-art / asset-production
  completion; Polish->Release gate-check retry; Stage advance from Polish to
  Release.
- closure of any other Sprint 14 row (only `S11-TD-UI-ZINDEX-LAYERS` closed
  by PROMPT 903; PROMPT 909 and PROMPT 908 each closed their own rows;
  remaining 14 Sprint 14 rows untouched).
- Sprint 14 close-out (Sprint 14 remains active; 3 of 17 rows closed after
  PROMPT 903).
- Sprint 14 Tier 0 burn-down completion (3 of 6 Tier 0 ranks remain ready
  -- ranks 3 / 5 / 6).
- Sprint 14 Tier 1 readiness (Tier 1 ranks 7 / 10 / 12 remain ready, gated
  on remaining Tier 0 ranks 3 / 5 / 6 + producer-decisions 2 / 3 / 4).
- PROMPT 802 §9 producer-decision-2 / -3 / -4 resolution.
- Sprint 15 planning.
- closure of `S11-HUD-TIMER-EYEBALL-VISUAL-001` Sprint 13 carry.
- closure of `S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001` backlog row.
- `TQ-S12-C7` closure.
- full-workspace `cargo test --workspace --tests --no-fail-fast` result
  claim (N/A; no cargo run by PROMPT 903; PROMPT 899 worker scope explicitly
  deferred per QA-plan-sprint-14 no-full-workspace-tests-by-default policy).
- any code change under `client/` / `server/` / `shared/` / `tests/` by
  PROMPT 903 (paperwork-only closure).
