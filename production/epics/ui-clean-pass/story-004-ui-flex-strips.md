# Story 004: S11-TD-UI-FLEX-STRIPS -- Flex-Based UI Strip Composition Primitives

> **Epic**: UI Clean-Pass
> **Story ID**: S11-TD-UI-FLEX-STRIPS
> **Status**: Done (Sprint 14 Must Have closed by PROMPT 919 on 2026-05-15;
> closure source-of-truth `origin/main@6ab4a2799c4b8c9f6627e128c745fe292d096afd`
> = PROMPT 918 `--no-ff` integration merge of PROMPT 915 worker tip
> `cae2f75be59552c83bc541bc8765d3a8e752a974` into prior `origin/main@3d99a04`;
> verdict **PASS**)
> **Layer**: Presentation / UX foundational tech-debt
> **Type**: Tech Debt -- foundational primitive
> **Sprint**: Sprint 14 (Tier 0 foundational; PROMPT 802 §4 rank 0.3;
> `docs/ux/ui-clean-pass-roadmap.md` rank 3). Activated by PROMPT 897;
> `/dev-story` by PROMPT 915; integrated by PROMPT 918; `/story-done` by
> PROMPT 919. Sprint 14 disposition `active`, stage `Polish` preserved.
> **Authored**: 2026-05-14 by PROMPT 878
> **Authoring source-of-truth**: `origin/main@51e6228` (PROMPT 871 `/story-done`
> on Sprint 13 row `S13-TWO-CLIENT-RUNTIME-HARNESS-001`)
> **Estimated effort**: ~1.0d (PROMPT 802 §4 Tier 0.3)

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

PROMPT 802 §3.9 G8 surfaced that `client/src/ui/` makes heavy use of
absolute positioning -- 226 `Node{}` / `Style{}` usages with
`PositionType::Absolute` dominant outside `result_screen.rs`. The HUD
top strip (gold / mana / phase) is **not** composed via a flex parent
(PROMPT 802 §3.2 H1, H8); each line is its own absolute child with
magic offsets (`hud_margin + 48.0`, `hud_margin + 60.0`,
`HUD_GOLD_ROW_GAP_PX = 48.0`, `HUD_SECONDARY_ROW_GAP_PX = 28.0`). The
lobby is a single absolute-positioned 420×?? column anchored at
`Val::Px(24.0), Val::Px(24.0)` on a 1920×1080 viewport with the rest
of the screen blank (§3.1 L1). The HUD bottom strip (figurine area) is
a single sprite + magic offset rather than a composed strip (§3.2 H9).
None of these layouts adapt to viewport changes.

This story introduces flex-based UI strip composition primitives so
header / lane / hand / footer bars can be expressed declaratively with
stable dimensions across 16:9 (1920×1080, 1366×768), 16:10 (1920×1200,
1280×800), and 4:3 (1280×960, 1024×768) viewports.

---

## Scope

### In Scope

- A new strip-composition primitive module (likely
  `client/src/ui/design_tokens/strips.rs` or `client/src/ui/primitives/strips.rs`;
  exact path TBD by the worker) that exports flex-based bundles or
  helper functions for the four canonical UI strips:
  - **HeaderBar** (top edge; HUD top strip: gold / mana / phase / timer)
  - **LaneBar** (mid; HUD lane indicators / board chrome border, IF
    bevy_ui rather than world-space sprites)
  - **HandBar** (lower mid; hand UI card row)
  - **FooterBar** (bottom edge; HUD bottom strip: figurine area,
    reserve-strip readouts)
- Each strip primitive expresses:
  - `Display::Flex` parent with explicit `flex_direction`,
    `align_items`, `justify_content`.
  - A stable height in `Val::Px` (deterministic across viewport sizes,
    derived from the global UI design spec from story 007).
  - A stable width as `Val::Percent(100.0)` (full viewport width) with
    optional `max_width` constraints.
  - A canonical anchor (`PositionType::Absolute` at the parent UI root
    level, then flex children inside).
- A spacing-scale constant set (likely `SPACING_XS / SM / MD / LG / XL`)
  shared across strips so per-module `_GAP_PX` constants (PROMPT 802
  §3.9 G2; e.g. `HUD_GOLD_ROW_GAP_PX = 48.0`,
  `HUD_SECONDARY_ROW_GAP_PX = 28.0`) can be replaced.
- Migration of HUD top strip (gold / mana / phase row) to use
  `HeaderBar` primitive with flex children for each readout, replacing
  absolute-positioned `hud_margin + N` magic offsets.
- Migration of HUD bottom strip (figurine area + reserve-strip
  readouts) to use `FooterBar` primitive.
- Migration of hand UI card row to use `HandBar` primitive (preserving
  the existing card-fan layout already repaired at `f190cc7`).
- A viewport-invariance smoke check in the strip module (basic
  `cargo test` ensuring the strip bundles compile and produce the
  expected `Node` style fields). Note: this is a *unit-level* check;
  the cross-viewport integration test lives in story 005.

### Out of Scope

- **No Sprint 14 activation** by this story.
- **No public release readiness** work.
- **No Standard-tier accessibility** (`QA-COND-0005`) completion.
  Specifically NOT in scope: ≥44px hit-target enforcement, keyboard
  navigation, screen reader hints, focus-order semantics. The lobby
  `LOBBY_BUTTON_HEIGHT = 30.0` defect (PROMPT 802 §3.1 L5) remains
  accept-risk under `QA-COND-0005`.
- **No final-art / asset-production** work (`PAW-TD-*-a`). Strip
  background art / chrome PNG selection is a separate scope.
- **No playtest validation** (`QA-COND-0006`).
- **No lobby layout repair**. Lobby refactor to a centered modal or
  full-viewport hero is `S12-UX-LOBBY-LAYOUT-MODAL-001` (Tier 1, not
  this story). This story only delivers the strip primitives that the
  HUD consumes.
- **No shop / auction panel** layout repair. Those are Tier 1 stories.
- **No draft centered modal** repair. That is `S11-UX-DRAFT-GRID-CENTERED-MODAL`
  (Tier 1, not this story).
- **No viewport-invariant test bin**. That is story 005 (separate; the
  strip module gets only unit-level checks here).
- **No z-index, typography, overlay-alpha** token work. Those are stories
  002 / 003 / 006.
- **No interaction-state primitive** work.
- **No board-rendering (world-space sprite) layout** change. If
  `LaneBar` ends up not making sense as bevy_ui because lane indicators
  are world-space sprites, the `LaneBar` primitive may be downgraded to
  optional and only `HeaderBar` / `HandBar` / `FooterBar` are required.
- **No animation / tween** of strip layout transitions. Static layout
  only.

---

## Acceptance Criteria

All criteria are independently checkable BLOCKING criteria.

- [x] **AC1 -- Strip primitive module authored** -- **PASS**: `client/src/ui/design_tokens/strips.rs` (NEW; 407 lines on `origin/main@6ab4a27`) exports four marker components (`HeaderBar` at `:106-107`, `LaneBar` at `:112-113`, `HandBar` at `:120-121`, `FooterBar` at `:125-126`) plus `header_bar_node()` / `lane_bar_node()` / `hand_bar_node()` / `footer_bar_node()` builder helpers at `:192-261`. Each helper returns a `Node` with `display: Display::Flex` plus documented `flex_direction` / `justify_content` / `align_items` axes sourced from per-strip `StripContract` constants at `:143-180` (HeaderBar Row/SpaceBetween/Center; LaneBar Row/Center/Center; HandBar Row/Center/FlexEnd; FooterBar Row/SpaceBetween/Center). `LaneBar` worker decision: documented-only per `docs/ux/global-ui-design-spec.md` §9 because lane indicators are world-space sprites under `client/src/presentation/board_rendering.rs` (ADR-021 R2); marker + helper exported for testability. Inline tests `ac1_three_required_strip_primitives_are_exported` + `ac1_each_strip_node_declares_display_flex_and_documented_axes` + `ac1_each_strip_is_full_viewport_width_at_absolute_position` + `ac1_canonical_strip_heights_match_spec_section_9` PASS; integration tests `ac1_three_required_strip_primitives_exported_with_flex_display` + `ac1_each_strip_documents_flex_direction_justify_align` PASS.

- [x] **AC2 -- Spacing scale constants** -- **PASS**: `client/src/ui/design_tokens/spacing.rs` (NEW; 194 lines) exports `SPACING_XS = 4.0` at `:61`, `SPACING_SM = 8.0` at `:65`, `SPACING_MD = 16.0` at `:70`, `SPACING_LG = 24.0` at `:74`, `SPACING_XL = 32.0` at `:80`, plus `ALL_SPACINGS_ASCENDING` table at `:85-91` and `SPACING_MIN_GAP = 2.0` at `:97`. Strict-ascending, pairwise-distinctness, positive-finite, min-gap-for-future-intermediates, canonical-values, and recomposition-rule (`SPACING_XL + SPACING_MD = 48`) invariants all asserted by inline `ac2_*` tests at `:103-180` + integration `ac2_*` tests in `tests/integration/ui_clean_pass/strips_test.rs` (3 tests). Doc comments at `:13-25` name canonical use per spec §4. All 7 inline + 3 integration AC2 tests PASS.

- [x] **AC3 -- HUD top strip migrated** -- **PASS**: `client/src/ui/hud/mod.rs` lines 77-83 document that `HUD_GOLD_ROW_GAP_PX = 48.0` and `HUD_SECONDARY_ROW_GAP_PX = 28.0` have been DELETED in this story; remaining `_GAP_PX` mentions are doc-comment migration notes only. Lines 557-571 spawn the canonical `HUD HeaderBar` via `strips::HeaderBar` marker + `strips::header_bar_node()` helper as a child of the HUD root. The 48-pixel opponent-gold vertical offset is recomposed at `:617` via `spacing::SPACING_XL + spacing::SPACING_MD` (32 + 16). The 28-pixel secondary-row offset is recomposed at `:2178` via `spacing::SPACING_XL - spacing::SPACING_XS` (32 - 4). The timer bar anchors at `:680` via `strips::HEADER_BAR_HEIGHT_PX` (60). Integration test `ac3_hud_module_spawns_header_bar_primitive` PASS; `ac3_hud_module_imports_spacing_and_strips_design_tokens` PASS; `ac3_hud_gold_row_gap_recomposes_through_xl_plus_md` PASS; `ac3_hud_timer_bar_anchors_at_header_bar_height` PASS.

- [x] **AC4 -- HUD bottom strip migrated** -- **PASS**: `client/src/ui/hud/mod.rs` lines 574-578 spawn the canonical `HUD FooterBar` via `strips::FooterBar` marker + `strips::footer_bar_node()` helper as a child of the HUD root. The figurine `hud_margin + 60.0` magic offset is replaced at `:647` with strip-relative anchoring expressed as `strips::FOOTER_BAR_HEIGHT_PX + spacing::SPACING_XL` (40 + 32 = 72 px, same pixel value, now expressed via design tokens). Integration test `ac4_hud_module_spawns_footer_bar_primitive` PASS; `ac4_hud_figurine_anchors_to_footer_bar_and_spacing_tokens` PASS.

- [x] **AC5 -- Hand UI card row migrated** -- **PASS**: `client/src/ui/hand/mod.rs` lines 2807-2824 spawn the canonical `Hand UI HandBar` via `strips::HandBar` marker + `strips::hand_bar_node()` helper, tagged with `HandUiEntity` so the existing despawn pipeline reclaims it. `HandFanRoot` re-parents to `hand_bar` via `ChildOf(hand_bar)` at `:2847`; the existing card-fan chrome (`f190cc7` repair: 7 chrome children 100×100% / 20×20% / 15×15%) is preserved verbatim inside `HandFanRoot`. `HAND_FAN_STRIP_HEIGHT_PX = 260` retained at `:35` as the local `HandFanRoot` height; `HAND_BAR_HEIGHT_PX = 180` is the strip footprint with `overflow: visible` on the strip parent so the fan extends 80 px above the strip footprint per PROMPT 913 readiness Concern #2 option (b). `HandUiEntities.hand_bar` field added at `:727`; `despawn_hand_ui` despawns `hand_bar` at `:3139` (recursively reclaims `fan_root`); `HAND_UI_ENTITY_COUNT` bumped `+ 1` at `:43`. Integration test `ac5_hand_module_imports_strips_and_spawns_hand_bar_primitive` PASS; `ac5_hand_fan_root_is_a_child_of_hand_bar` PASS.

- [x] **AC6 -- Stable dimensions across viewport ratios** -- **PASS**: Strip heights declared as `pub const HEADER_BAR_HEIGHT_PX: f32 = 60.0` / `LANE_BAR_HEIGHT_PX = 60.0` / `HAND_BAR_HEIGHT_PX = 180.0` / `FOOTER_BAR_HEIGHT_PX = 40.0` at `client/src/ui/design_tokens/strips.rs:84/90/97/102`. Each `*_node()` helper returns `height: Val::Px(<const>)` (pixel-fixed) and `width: Val::Percent(100.0)` (viewport-scaled). Integration test `ac6_strip_heights_are_identical_across_every_canonical_viewport` iterates the canonical 6-viewport matrix (1366×768 / 1920×1080 / 1920×1200 / 1280×960 / 3840×2160 / 2560×1080) and asserts every strip height resolves to the canonical pixel value at every viewport, plus a positive-centre-play-area constraint. `ac6_top_strip_does_not_overlap_bottom_strips_in_canonical_viewport` inline test in `strips.rs` extends the no-overlap invariant. Story 005 viewport-invariant test bin (`tests/integration/ui_viewport_invariants_test.rs`, 12/12 GREEN on `origin/main@6ab4a27` per worker + integration evidence) confirms strip-height-determinism invariant holds across the canonical matrix. Visual capture at the five viewport ratios under the evidence path is a follow-on QA-tester deliverable under `/team-qa` and is OUT OF SCOPE for this `/story-done` paperwork closure per the PROMPT 913 readiness no-claim list.

- [x] **AC7 -- No per-module `_GAP_PX` magic constants remain (HUD)** -- **PASS**: `HUD_GOLD_ROW_GAP_PX` and `HUD_SECONDARY_ROW_GAP_PX` constant declarations DELETED from `client/src/ui/hud/mod.rs`. Independent `grep -n "_GAP_PX" client/src/ui/hud/mod.rs` on `origin/main@6ab4a27` returns 6 matches; ALL six are doc-comment migration-note references (lines 77-79 + 83 in the module preamble, line 606 in the recomposed gold-row site, line 2171 in the recomposed secondary-row site). No `_GAP_PX` identifier survives as a live `const` declaration or expression operand outside doc comments. Integration test `ac7_no_gap_px_identifier_in_hud_module` walks the file, strips doc-comment lines, and asserts zero surviving `_GAP_PX` identifier. Test PASS.

- [x] **AC8 -- Strip primitive unit test** -- **PASS-WITHIN-STORY-PRESCRIBED-TARGETED-CHECK**: Per Sprint 14 QA-plan binding no-full-workspace-tests-by-default policy: `cargo test -p client --test ui_clean_pass_strips_test` runs all 20 integration tests in 0.00s, all passing (PROMPT 915 worker evidence + PROMPT 918 integration evidence; the bin includes `ac8_each_strip_node_resolves_to_documented_flex_axis_set`, `ac8_strip_anchors_match_spec_column_composition`, and `ac8_strip_marker_components_are_distinct_zero_sized_components` as the canonical AC8 unit-style assertions). Inline `cargo test -p client --lib ui::design_tokens::strips` 7/7 + `cargo test -p client --lib ui::design_tokens::spacing` 7/7 also runnable per qa-plan §line 205. Story 002 / 003 / 005 regression bins (`ui_clean_pass_z_layers_test` 6/6, `ui_clean_pass_typography_test` 8/8, `ui_viewport_invariants_test` 12/12) GREEN on integration tip. No new `#[ignore]` markers. Full-workspace `cargo test` deferred to Sprint 14 end-of-sprint integration smoke.

- [x] **AC9 -- Friend-game scope preserved** -- **PASS**: `git diff 6ab4a27^1..6ab4a27 --stat -- 'production/sprint-status.yaml' 'production/sprints/sprint-14.md' 'production/stage.txt' 'production/qa/qa-plan-sprint-14.md' 'production/session-state/' 'server/' 'shared/'` returns EMPTY across the integration commit. `QA-COND-0005` Standard-tier accessibility accept-risk preserved unchanged. `QA-COND-0006` playtest validation accept-risk preserved unchanged. `PAW-TD-*-a` placeholder-art accept-risk PAW-002..PAW-006 preserved unchanged. Module-level scope-discipline doc-comments at `client/src/ui/design_tokens/spacing.rs:50-57` + `client/src/ui/design_tokens/strips.rs:67-77` + evidence document §"Carried non-claims" all preserve `QA-COND-0005` + `QA-COND-0006` + `PAW-TD-*-a` references verbatim. PROMPT 919 row-level flip is the permitted disposition-preserving paperwork edit; no accept-risk disposition is flipped to `closed` by this story.

---

## Evidence Path

`production/qa/evidence/sprint-14-ui-foundation/ui-flex-strips/`

Expected artifacts:

- Visual captures at 1920×1080 / 1366×768 / 1920×1200 / 1280×960 / 4K
  showing each migrated strip with stable dimensions.
- Strip primitive unit test output for AC8.
- HUD pre/post-migration diff capture (top strip + bottom strip).
- Hand UI pre/post-migration diff capture (card row inside HandBar).

---

## Likely Files Touched

| Path | Expected change |
|------|-----------------|
| `client/src/ui/design_tokens/strips.rs` (NEW; or `client/src/ui/primitives/strips.rs` -- exact path TBD by worker) | Author strip-composition primitives. |
| `client/src/ui/design_tokens/spacing.rs` (NEW; or extended) | Spacing-scale constants. |
| `client/src/ui/design_tokens/mod.rs` (NEW or extended) | Re-export strip + spacing tokens. |
| `client/src/ui/mod.rs` | Declare `design_tokens` submodule (if not already from story 002 / 003). |
| `client/src/ui/hud/mod.rs` | Migrate top strip + bottom strip to flex primitives; replace `_GAP_PX` constants. |
| `client/src/ui/hand/mod.rs` | Migrate card row into HandBar primitive (preserving existing card fan). |
| `tests/unit/ui_strips_test.rs` (NEW) | AC8 strip primitive unit test. |

This table is a planning estimate. The implementation prompt is authoritative
for the realised set.

---

## Verification

- `cargo test -p client --lib ui_strips` -- AC8 unit test.
- `cargo test -p client --lib spacing` -- AC2 spacing-scale ordering test.
- Visual captures at the five required viewport sizes -- AC3 / AC4 / AC5 /
  AC6.

---

## Dependencies / Sequencing

- **Authoring prompt (this PROMPT 878)** is the *story-authoring* prompt;
  it creates the story file only. No `/dev-story` runs here.
- **Activation**: Requires Sprint 14 activation (separate prompt; not this
  one). Cannot land in Sprint 13.
- **Tier 0 internal sequencing** (per PROMPT 802 §8 and roadmap §3
  "Sequencing Rules"): this story is **mostly serial** with stories 002
  (z-index layers), 003 (font constants), and 006 (overlay alpha token)
  because all four touch the shared design-token host module
  (`client/src/ui/design_tokens/`). Story 005 (viewport-invariant tests)
  is parallel-safe.
- **Tier 0 design-spec dependency** (per roadmap §3 rule 2): story 007
  (`S12-UX-GLOBAL-UI-DESIGN-SPEC-001`) should be authored *first* so
  the strip height pixel values + the spacing scale numeric values are
  ratified by UX + art before this story lands. If story 007 has not
  landed, the worker can propose default values and call them out as
  ratify-on-spec.
- **Tier 1 surface stories depend on this**: PROMPT 802 §6 Lane C
  per-surface stories (S11-UX-HUD-TOP-STRIP-LAYOUT,
  S11-UX-HUD-BOTTOM-STRIP-LAYOUT, S11-UX-LOBBY-CLASS-PICKER, etc.) all
  expect to consume the strip primitives delivered here. PROMPT 685
  row 2 explicitly bundles `S11-TD-UI-FLEX-STRIPS` with the HUD
  top-strip + bottom-strip + opp-figurine stories.

---

## Notes

- PROMPT 802 §3.2 H1: every HUD child uses `PositionType::Absolute`
  with hard-coded `Val::Px(margin + N)` offsets relative to the corner.
- PROMPT 802 §3.2 H2: magic offsets (figurine `bottom: hud_margin +
  60.0`, timer_bar `top: hud_margin + 48.0`, `HUD_GOLD_ROW_GAP_PX =
  48.0`, `HUD_SECONDARY_ROW_GAP_PX = 28.0`) -- no shared spacing scale.
- PROMPT 802 §3.2 H8: top-strip layout not composed via a flex parent.
- PROMPT 802 §3.2 H9: bottom-strip layout is a single sprite + magic
  offset, not composed.
- PROMPT 802 §3.9 G2: no shared spacing scale; G8: 226 `Node` /
  `Style` usages with absolute positioning dominant.
- PROMPT 685 row 2 explicitly bundles `S11-TD-UI-FLEX-STRIPS` with the
  HUD top-strip + bottom-strip + opp-figurine surface stories
  (`docs/ux/ui-clean-pass-roadmap.md` PROMPT 685 reconciliation matrix).
- The four canonical strip names (HeaderBar / LaneBar / HandBar /
  FooterBar) come from PROMPT 878's task brief; final naming may be
  ratified by story 007's UX spec.
- Accept-risk preservation: `PAW-TD-*-a`, `QA-COND-0005`, `QA-COND-0006`
  preserved unchanged. This story does not advance any of them.

---

## Closure Trail

| Step | Prompt | Date | Source-of-truth | Commit | Outcome |
|------|--------|------|-----------------|--------|---------|
| Authored | PROMPT 878 | 2026-05-14 | `origin/main@51e6228` | merged via PROMPT 893 `9f36663` | Story file authored as Sprint 14 candidate. |
| Activated | PROMPT 897 | 2026-05-15 | `origin/main@ce8f590` | `fffaf1c` | Sprint 14 activated; row added to `production/sprint-status.yaml` `stories:` block with `status: ready`. |
| QA-plan | PROMPT 898 | 2026-05-15 | `origin/main@fffaf1c` | `4dd7fe3` | `production/qa/qa-plan-sprint-14.md` authored covering all 17 Sprint 14 rows including this row. |
| `/dev-story` | PROMPT 915 | 2026-05-15 | `origin/main@3d99a04` (PROMPT 912 tip) | worker `cae2f75` on `work/s14-flex-strips` | 8 files / +1425 / -11; `cargo fmt -p client -- --check` clean; `cargo check -p client` clean; `cargo test -p client --lib ui::design_tokens::spacing` 7/7 PASS; `cargo test -p client --lib ui::design_tokens::strips` 7/7 PASS; `cargo test -p client --test ui_clean_pass_strips_test` 20/20 PASS; nearby regression bins (z-layers / typography / viewport-invariants / hud / hand) all GREEN. PROMPT 913 readiness Concerns #1 + #2 reconciled. Verdict PASS. |
| `/integrate` | PROMPT 918 | 2026-05-15 | `origin/main@3d99a04` | merge `6ab4a27` on `integrate/s14-flex-strips-918` | `--no-ff` merge of `cae2f75` into prior `origin/main@3d99a04`; zero conflicts; 8 files / +1425 / -11; PROMPT 915 worker reachable as merge's second parent. Forbidden-paths diff (`server/`, `shared/`, `production/sprint-status.yaml`, `production/session-state/`, `production/stage.txt`) empty. Pushed `3d99a04..6ab4a27 integrate/s14-flex-strips-918 -> main`. Verdict PASS. |
| `/story-done` | PROMPT 919 | 2026-05-15 | `origin/main@6ab4a27` | this commit | Paperwork-only closure: this story file Status header `Draft -> Done` + AC1-AC9 checkboxes `[ ] -> [x]` + this Closure Trail section appended; `production/sprint-status.yaml` row `S11-TD-UI-FLEX-STRIPS` flipped `ready -> done` with `completed: 2026-05-15` + worker/integration/story-done metadata + `sprint_14_story_done:` block extended with PROMPT 919 entry as **fourth** `/story-done` block of Sprint 14 (PROMPT 909 first + PROMPT 908 second + PROMPT 903 third entries preserved verbatim); `production/session-state/active.md` PROMPT 919 banner prepended; `production/session-state/codex-orchestrator-state.md` PROMPT 919 section prepended. Verdict PASS. |

### Conditions carried forward unchanged

- `S8-QA-001-W1` manual / browser two-client GAME_OVER gap remains OPEN.
  Story 017 AC12 forbid-auto-closure preserved through Sprint 13 close-out
  and into Sprint 14. Flex-strip primitives do not touch the two-client
  GAME_OVER surface.
- `QA-COND-0005` Standard-tier accessibility remains accepted-risk
  (friend-game scope only). Strip primitives are layout primitives, not
  Standard-tier accessibility conformance.
- `QA-COND-0006` playtest / fun-hypothesis validation remains
  accepted-risk / deferred.
- `PAW-TD-*-a` placeholder-art accept-risk preserved across
  PAW-002..PAW-006.
- PROMPT 683-era runtime divergence question preserved (folded into
  Sprint 12 story 019 cannot-reproduce closure; third same-scope retest
  NOT authorised per `TQ-S12-C2`).
- PROMPT 761 `Polish->Release` gate-check `FAIL` preserved at
  `production/gate-checks/gate-polish-release-2026-05-12.md`; NO retry
  in PROMPT 919 scope.
- Sprint 12 story 019 underlying drag-runtime bug NOT claimed fixed
  (closed cannot-reproduce, NOT bug-fixed).
- `TQ-S12-C1..C7` (all seven Sprint 12 Team-QA conditions) preserved
  verbatim; `TQ-S12-C7` NOT closed by PROMPT 919.
- Sprint 13 close-out (`closed-with-conditions` per PROMPT 894) preserved
  unchanged.
- Sprint 12 / Sprint 11 / Sprint 10 closeouts preserved unchanged.
- All prior Sprint 14 `/story-done` closures (PROMPT 909 viewport-tests
  first, PROMPT 908 font-constants second, PROMPT 903 z-layers third
  reconcile) preserved verbatim on `origin/main`.
- `S11-HUD-TIMER-EYEBALL-VISUAL-001` Sprint 14 Should Have carry remains
  `ready` and human-operator-blocked; no LLM `/story-done` authorised.
- `S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001` backlog row remains
  as-is.
- PROMPT 802 §9 producer-decision-2 (numeric values) STILL UNRESOLVED on
  `origin/main`. Story 004 implementation used the spec ratified values
  (60 / 60 / 180 / 40 strip heights; 4 / 8 / 16 / 24 / 32 spacing scale)
  documented in `docs/ux/global-ui-design-spec.md` §9 + §4.
- PROMPT 802 §9 producer-decision-3 (lobby modal-panel vs full-viewport
  hero) STILL UNRESOLVED.
- PROMPT 802 §9 producer-decision-4 (auction lead-loss visual language)
  STILL UNRESOLVED.

### Explicitly NOT claimed by PROMPT 919 closure

- Public release readiness.
- Release-candidate readiness.
- Full game completion.
- Broad / Standard-tier accessibility completion (`QA-COND-0005`).
- Playtest / fun-hypothesis validation (`QA-COND-0006`).
- Full playable-client manual QA.
- Two-client GAME_OVER closure (`S8-QA-001-W1`).
- Final-art / asset-production completion (`PAW-TD-*-a`).
- `Polish->Release` gate-check retry.
- Stage advance from `Polish` to `Release`.
- Underlying drag-runtime bug fix (Sprint 12 story 019 closed
  cannot-reproduce, NOT bug-fixed).
- Closure of `S11-HUD-TIMER-EYEBALL-VISUAL-001` (Sprint 14 Should Have
  carry; human-operator-blocked; no LLM `/story-done` authorised).
- Closure of `S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001` backlog row.
- `TQ-S12-C7` closure.
- Sprint 14 close-out (Sprint 14 remains `active`; 4 of 17 rows closed
  after PROMPT 919).
- Tier 1 surface-story readiness (HUD top strip 015 / auction featured
  card 016 / lobby layout modal 024 / HUD bottom strip 016 / draft
  centered modal 015 / lobby class-picker 025 etc. remain `ready` /
  gated on remaining Tier 0 ranks 5 + 6 + producer-decisions).
- Overlay alpha migration (story 006 scope; PROMPT 917 integration not
  yet on `origin/main`).
- Global UI design spec ratification (story 007 scope; remaining
  Tier 0 rank 6).
- Visual capture at 1366×768 / 1920×1080 / 1920×1200 / 1280×960 / 4K
  viewports under the evidence path is a follow-on QA-tester
  deliverable under `/team-qa` and is OUT OF SCOPE for this
  `/story-done` paperwork closure.

### Downstream unblock notes

- Tier 1 surface stories (015 HUD top strip, 016 HUD bottom strip, 024
  lobby layout modal, 025 lobby class-picker) can now consume
  `strips::HeaderBar` / `strips::FooterBar` / `strips::HandBar`
  primitives and `spacing::SPACING_*` tokens instead of recreating
  flex-parent + magic-offset patterns.
- Tier 0 rank 5 (`S12-TD-UI-OVERLAY-ALPHA-TOKEN-001`, story 006) and
  rank 6 (`S12-UX-GLOBAL-UI-DESIGN-SPEC-001`, story 007) remain
  `ready` on the shared design-token host module.
- The PROMPT 802 §9 producer-decision-2 numeric values are documented
  in the spacing + strips module preambles as the spec-§4 / §9
  ratified set; future producer ratification edits values in one place
  (`client/src/ui/design_tokens/spacing.rs` constants / `strips.rs`
  height constants) without disturbing consumer call sites.
