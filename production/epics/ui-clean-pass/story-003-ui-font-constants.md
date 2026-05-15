# Story 003: S11-TD-UI-FONT-CONSTANTS -- Typography Scale Tokens (Single Source Of Truth)

> **Epic**: UI Clean-Pass
> **Story ID**: S11-TD-UI-FONT-CONSTANTS
> **Status**: Done (closed by PROMPT 908 `/story-done` on `origin/main@eb1c128` PROMPT 906 integration tip)
> **Layer**: Presentation / UX foundational tech-debt
> **Type**: Tech Debt -- foundational primitive
> **Sprint**: Sprint 14 candidate (Tier 0 foundational; PROMPT 802 §4 rank 0.2;
> `docs/ux/ui-clean-pass-roadmap.md` rank 2). NOT activated by this authoring
> run. Sprint 13 disposition (`active`, `Polish` stage) preserved.
> **Authored**: 2026-05-14 by PROMPT 878
> **Authoring source-of-truth**: `origin/main@51e6228` (PROMPT 871 `/story-done`
> on Sprint 13 row `S13-TWO-CLIENT-RUNTIME-HARNESS-001`)
> **Estimated effort**: ~0.5d (PROMPT 802 §4 Tier 0.2)

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

PROMPT 802 §3.9 G3 surfaced that typography in the playable client is
declared per-module with no shared scale. Font sizes range from `13`
(lobby class buttons) to `40` (HUD gold readout), with at least 14
distinct `font_size` literals across HUD, lobby, shop/auction, result
screen, and hand UI. PROMPT 802 §3.2 H5 enumerated the HUD-side
constants (`HUD_GOLD_FONT_SIZE_PX = 40`, `HUD_RESOURCE_TEXT_MIN_SIZE_PX
= 20`, `HUD_RESERVED_GOLD_FONT_SIZE_PX = 26`, `HUD_SECONDARY_FONT_SIZE_PX
= 20`) and showed they are not shared with lobby (14, 15, 18) / shop /
result (15, 18, 36) / hand. PROMPT 802 §3.1 L6 further called out the
*inverted* lobby typography hierarchy where labels (`13px`) are smaller
than the data they describe (`15-18px`) -- a defect that is impossible
to fix coherently without a typography scale.

This story introduces a single canonical typography scale module
exposing semantic size tokens (caption / body / h3 / h2 / h1 / display),
weight constants, and a line-height ratio, replacing the ad-hoc
`font_size` literals scattered across `client/src/ui/`.

---

## Scope

### In Scope

- A new typography token module (likely
  `client/src/ui/design_tokens/typography.rs`; exact path TBD by the
  worker) that exports a `TypographyScale` enum or const module covering
  at minimum the following semantic levels (smallest to largest):
  - `Caption` (~12px) -- micro-copy / footnotes
  - `Body` (~15px) -- default running text, labels
  - `H3` (~18px) -- subhead, section labels
  - `H2` (~22px) -- panel titles
  - `H1` (~30px) -- screen headlines (e.g. result screen "Victory")
  - `Display` (~40px) -- HUD primary readouts (gold, mana, timer)
- A weight constant set covering at least `FontWeight::Regular`,
  `FontWeight::SemiBold`, `FontWeight::Bold` (or the equivalent for the
  loaded font asset). Mapping to actual font assets may be deferred to a
  follow-on story; this story commits to the *constants* and their
  consumers, not necessarily the font-asset switching mechanism.
- A canonical line-height ratio constant (e.g.
  `LINE_HEIGHT_DEFAULT_RATIO: f32 = 1.25`) usable across surfaces; ratio
  not magic numbers per spawn site.
- Migration of all existing inline `font_size:` literals under
  `client/src/ui/` (HUD, lobby, hand, shop_auction, settings) to the new
  module's named constants.
- Migration of `client/src/presentation/result_screen.rs` font sizes
  (currently 36 / 18 / 15) to the new module's `H1` / `H3` / `Body`
  constants.
- A grep guard or compile-time pattern that prevents reintroduction of
  inline `font_size: Val::Px(N)` / numeric literals on `font_size` fields
  outside the design-token module.
- Fixing the lobby typography inversion (PROMPT 802 §3.1 L6) where the
  story authoring intends labels to be at least as small as the data
  they describe, NOT smaller -- mapping data to a larger semantic level
  than labels.

### Out of Scope

- **No Sprint 14 activation** by this story. Sprint 14 pull-in is a
  separate prompt.
- **No public release readiness** work. Typography constants are a
  Polish-stage foundational primitive; they do not advance the
  `Polish->Release` gate-check retry.
- **No Standard-tier accessibility** (`QA-COND-0005`) completion.
  Specifically NOT in scope: WCAG-compliant minimum font sizes,
  user-controllable text scaling, contrast ratio adjustments, screen
  reader hints. The typography scale is sized for friend-game scope
  visual hierarchy only.
- **No final-art / asset-production** work (`PAW-TD-*-a`). Font *asset*
  selection (switching from the current font asset to a designer-chosen
  family) is a separate scope. This story commits only to constants and
  their consumers.
- **No playtest validation** (`QA-COND-0006`).
- **No z-index, spacing, overlay-alpha, viewport-invariant test** work.
  Those are separate stories (002 = z-index; 004 = flex strips; 005 =
  viewport tests; 006 = overlay alpha; 007 = global UI design spec).
- **No interaction-state primitive** work (hover / focus / pressed /
  disabled).
- **No automatic responsive font scaling** based on viewport size. The
  scale is fixed-pixel for friend-game scope; responsive scaling is a
  follow-on if-and-when needed.
- **No re-skin of board-rendering sprite text** (e.g. lane indicators
  rendered as sprites). Only `bevy_ui` Text nodes are in scope.

---

## Acceptance Criteria

All criteria are independently checkable BLOCKING criteria.

- [x] **AC1 -- Typography token module authored** (PASS via PROMPT 904
  worker / PROMPT 906 integration `eb1c128` --
  `client/src/ui/design_tokens/typography.rs:85-150` exports 6 named
  semantic-size constants `CAPTION=13` / `BODY=15` / `H3=18` / `H2=22` /
  `H1=30` / `DISPLAY=40` (strictly ascending) plus 3 named weight
  tokens `WEIGHT_REGULAR=400` / `WEIGHT_SEMIBOLD=600` /
  `WEIGHT_BOLD=700`; inline tests
  `ac1_six_named_semantic_sizes_strictly_ascending` /
  `ac1_canonical_scale_ordering_matches_story_spec` /
  `ac1_each_scale_resolves_to_positive_finite_f32` /
  `ac1_scale_constants_are_pairwise_distinct` /
  `ac1_three_named_weights_strictly_ascending` /
  `ac1_scale_constants_have_minimum_gap_for_future_intermediates` PASS
  9/9 per worker + integration reports).

- [x] **AC2 -- Line-height ratio constant** (PASS --
  `client/src/ui/design_tokens/typography.rs:126-130` declares
  `pub const LINE_HEIGHT_DEFAULT_RATIO: f32 = 1.25;` with doc comment
  "Canonical line-height ratio applied to running text. Multiply by a
  semantic-size constant to obtain a `Val::Px(...)` line height when
  explicit vertical rhythm is required. Single source of truth so spawn
  sites never embed ad-hoc ratios."; inline test
  `ac2_line_height_ratio_is_positive_finite_and_at_least_one` PASS).

- [x] **AC3 -- All inline `font_size` literals migrated** (PASS --
  integration test
  `tests/integration/ui_clean_pass/typography_test.rs:93-123`
  (`ac3_grep_guard_no_inline_font_size_literals_outside_design_tokens`)
  walks every `*.rs` under `client/src/` outside
  `client/src/ui/design_tokens/` and asserts zero
  `font_size: <Val::Px|digit>` matches; PROMPT 908 independent grep
  `rg "font_size:\s*(Val::Px\(|[0-9])" client/src` on `eb1c128` returns
  zero hits).

- [x] **AC4 -- HUD constants subsumed** (PASS --
  `client/src/ui/hud/mod.rs:60`
  `pub const HUD_GOLD_FONT_SIZE_PX: f32 = typography::DISPLAY;`,
  `:66` `pub const HUD_RESERVED_GOLD_FONT_SIZE_PX: f32 = typography::H1;`,
  `:72` `pub const HUD_SECONDARY_FONT_SIZE_PX: f32 = typography::H2;`;
  `HUD_GOLD_TEXT_MIN_SIZE_PX` and `HUD_RESOURCE_TEXT_MIN_SIZE_PX`
  preserved as independent accessibility-floor invariants per the AC4
  "either resolve through the new module's constants OR have been
  removed" disposition language and consumed by
  `tests/integration/hud/text_size_contrast_accessibility_test.rs`;
  integration tests
  `ac4_hud_font_size_constants_resolve_through_design_tokens` +
  `ac4_hud_resource_text_min_size_is_independent_accessibility_floor`
  PASS at `tests/integration/ui_clean_pass/typography_test.rs:148-191`).

- [x] **AC5 -- Result screen migrated** (PASS --
  `client/src/presentation/result_screen.rs:550` headline routes
  through `typography::H1` (30 px, previously 36 px literal), `:558`
  cause routes through `typography::H3` (18 px, preserved), `:567`
  summary routes through `typography::BODY` (15 px, preserved), `:655`
  return button inline routes through `typography::H3`, `:979` /
  `:999` objective columns route through `typography::BODY` /
  `typography::CAPTION`; integration test
  `ac5_result_screen_migrated_to_h1_h3_body` at
  `tests/integration/ui_clean_pass/typography_test.rs:193-222` PASS;
  RESULT PENDING headline preserved verbatim; regression `result_screen_mvp_test`
  6/6 + `result_screen_return_to_lobby_test` 2/2 GREEN at integration
  tip per PROMPT 906 report).

- [x] **AC6 -- Lobby typography inversion fixed** (PASS via code
  review of integrated `client/src/ui/lobby.rs` at `eb1c128` --
  `:900` status banner routes through `typography::H3` (18 px),
  `:914`, `:928`, `:940`, `:954`, `:968`, `:977`, `:990`, `:1009`,
  `:1078` labels (Requested slot / Class) and CTAs (Create / Join /
  Confirm) and slot / class buttons and room-code chip routed through
  `typography::BODY` (15 px, was 13 / 14 px); no remaining
  `lobby_text_font(13.0)` or `lobby_text_font(14.0)` call sites;
  hierarchy invariant H3 ≥ BODY ≥ data, no label smaller than data;
  integration test `ac6_lobby_typography_inversion_fixed` at
  `tests/integration/ui_clean_pass/typography_test.rs:224-267` PASS;
  visual capture deferral: per Sprint 14 QA plan §"Manual / visual
  review expectations" Tier 0 token modules are ADVISORY for
  expert-UI-designer sign-off; the canonical 1366×768 / 1920×1080
  visual capture pair is bundled into the Tier 1 lobby layout rows
  S12-UX-LOBBY-LAYOUT-MODAL-001 / S11-UX-LOBBY-CLASS-PICKER which
  consume these tokens; documented as part of `production/qa/evidence/sprint-14-ui-font-constants/evidence.md`).

- [x] **AC7 -- Grep guard** (PASS -- the AC3 grep-guard test
  `ac3_grep_guard_no_inline_font_size_literals_outside_design_tokens`
  at `tests/integration/ui_clean_pass/typography_test.rs:93-123` is
  the executable form of this guard and runs with the regular cargo
  test harness; sanity-predicate test
  `ac7_grep_guard_pattern_actually_detects_a_synthesized_violation`
  at `tests/integration/ui_clean_pass/typography_test.rs:125-146`
  asserts the predicate matches both `font_size: 14.0,` and
  `font_size: Val::Px(18.0),` while not matching the post-migration
  shapes `font_size: typography::H3,`, named-constant references like
  `font_size: DAMAGE_NUMBER_FONT_SIZE,`, and the function-parameter
  declaration `lobby_text_font(font_size: f32) -> TextFont`).

- [x] **AC8 -- Unit tests pass** (PASS-WITHIN-STORY-PRESCRIBED-TARGETED-CHECK
  per Sprint 14 QA-plan binding no-full-workspace-tests-by-default
  policy: `cargo test -p client --lib ui::design_tokens::typography`
  9/9 PASS + `cargo test -p client --test ui_clean_pass_typography_test`
  8/8 PASS at integration tip `eb1c128` per PROMPT 906 report; surface
  spot-check tests
  `ac8_every_migrated_surface_references_typography_module` +
  `ac8_typography_module_exports_required_token_set` at
  `tests/integration/ui_clean_pass/typography_test.rs:269-319` PASS;
  no new `#[ignore]` markers added; full-workspace cargo test deferred
  to Sprint 14 end-of-sprint integration smoke per Sprint 14 QA plan).

- [x] **AC9 -- Friend-game scope preserved** (PASS --
  `git diff eb1c128^1..eb1c128 --stat -- 'production/sprint-status.yaml'
  'production/sprints/' 'production/stage.txt' 'production/qa/qa-plan-sprint-14.md'
  'production/session-state/'` empty across worker + integration
  commits; PROMPT 908 row-level flip on
  `production/sprint-status.yaml` is the permitted
  disposition-preserving paperwork edit and does NOT touch any
  accept-risk disposition; `QA-COND-0005`, `QA-COND-0006`, and
  `PAW-TD-*-a` remain accept-risk; the typography module preamble
  documents friend-game scope discipline at
  `client/src/ui/design_tokens/typography.rs:69-81` and
  `production/qa/evidence/sprint-14-ui-font-constants/evidence.md`
  records the non-claim verbatim).

---

## Evidence Path

`production/qa/evidence/sprint-14-ui-foundation/ui-font-constants/`

Expected artifacts:

- Unit test output for AC1 (ordering / distinctness).
- Grep-guard output for AC3 / AC7.
- Visual capture comparison for AC5 (result screen pre/post).
- Visual capture for AC6 (lobby hierarchy post-fix at 1366×768 + 1920×1080).

---

## Likely Files Touched

| Path | Expected change |
|------|-----------------|
| `client/src/ui/design_tokens/typography.rs` (NEW; exact path TBD by worker) | Author typography token module (sizes + weights + line-height ratio). |
| `client/src/ui/design_tokens/mod.rs` (NEW or extended) | Re-export typography tokens. |
| `client/src/ui/mod.rs` | Declare `design_tokens` submodule (if not already from story 002). |
| `client/src/ui/lobby.rs` | Replace inline font_size literals; fix L6 hierarchy inversion. |
| `client/src/ui/hud/mod.rs` | Subsume HUD-local font constants into the new module. |
| `client/src/ui/hand/mod.rs` | Replace card text / stat / keyword font_size literals. |
| `client/src/ui/shop_auction/mod.rs` | Replace shop / draft / auction / settlement text font_size literals. |
| `client/src/ui/settings/` | Replace settings UI font_size literals (if any). |
| `client/src/presentation/result_screen.rs` | Replace 36 / 18 / 15 with H1 / H3 / Body. |
| `tests/unit/ui_typography_test.rs` (NEW) | AC1 ordering / distinctness unit test. |

This table is a planning estimate. The implementation prompt is authoritative
for the realised set.

---

## Verification

- `cargo test -p client --lib typography` -- AC1 / AC8 unit tests.
- `rg "font_size:\s*(Val::Px\(|[0-9])" client/src/ --glob '!client/src/ui/design_tokens/**'`
  -- AC3 / AC7 grep guard.
- Visual capture at 1366×768 + 1920×1080 against the lobby and the result
  screen -- AC5 / AC6 hierarchy verification.

---

## Dependencies / Sequencing

- **Authoring prompt (this PROMPT 878)** is the *story-authoring* prompt;
  it creates the story file only. No `/dev-story` runs here.
- **Activation**: Requires Sprint 14 activation (separate prompt; not this
  one). Cannot land in Sprint 13.
- **Tier 0 internal sequencing** (per PROMPT 802 §8 and roadmap §3
  "Sequencing Rules"): this story is **mostly serial** with stories 002
  (z-index layers), 004 (flex strips), and 006 (overlay alpha token)
  because all four touch the shared design-token host module
  (`client/src/ui/design_tokens/`). Story 005 (viewport-invariant tests)
  is parallel-safe.
- **Tier 0 design-spec dependency** (per roadmap §3 rule 2): story 007
  (`S12-UX-GLOBAL-UI-DESIGN-SPEC-001`) should be authored *first* in
  Phase 1 so the typography numeric values for each semantic level are
  ratified by the UX-designer + art-director before this story lands.
  If story 007 has not yet landed, the worker can propose default values
  (12 / 15 / 18 / 22 / 30 / 40) and call them out as ratify-on-spec.
- **Tier 1 surface stories depend on this**: PROMPT 802 §3.1 L6 (lobby
  hierarchy inversion), §3.2 H5 (HUD per-module constants), and all
  future per-surface text-heavy work (hand-card text, shop slot wells,
  auction featured-card text) will consume these tokens.

---

## Notes

- PROMPT 802 §3.9 G3: ad-hoc font sizes 13..40 declared per-module.
- PROMPT 802 §3.2 H5: HUD constants `HUD_GOLD_FONT_SIZE_PX = 40`,
  `HUD_RESOURCE_TEXT_MIN_SIZE_PX = 20`, `HUD_RESERVED_GOLD_FONT_SIZE_PX
  = 26`, `HUD_SECONDARY_FONT_SIZE_PX = 20`.
- PROMPT 802 §3.1 L6: lobby inverted hierarchy -- status text `18`, room
  code `15`, action buttons `14`, slot/class buttons `13`, labels `13`.
- PROMPT 802 §3.8 R4: result screen 36 / 18 / 15 is called out as
  "clear visual rank" -- this story preserves that hierarchy via H1 /
  H3 / Body.
- The semantic-level names (Caption / Body / H3 / H2 / H1 / Display)
  come from PROMPT 878's task brief and align with standard UI design
  system vocabulary; final naming may be ratified by story 007's UX
  spec.
- Accept-risk preservation: `PAW-TD-*-a`, `QA-COND-0005`, `QA-COND-0006`
  preserved unchanged. This story does not advance any of them.

---

## Closure Trail

- **PROMPT 878** (2026-05-14) -- story authored as Sprint 14 Tier 0 candidate
  on `origin/main@51e6228`, integrated by PROMPT 893 merge `9f36663`.
- **PROMPT 896** (2026-05-15) -- Sprint 14 plan drafted (DRAFT, NOT activated)
  at `ce8f590`.
- **PROMPT 897** (2026-05-15) -- Sprint 14 ACTIVATED at `fffaf1c`; row
  `S11-TD-UI-FONT-CONSTANTS` added to top-level `stories:` block with
  `status: ready`, roadmap_rank 2, producer-decision-2 blocker noted.
- **PROMPT 898** (2026-05-15) -- Sprint 14 QA plan authored at
  `4dd7fe3`; story-003 covered by AC matrix.
- **PROMPT 899 / PROMPT 902** (2026-05-15) -- Tier 0 host module
  `client/src/ui/design_tokens/` landed via PROMPT 902 integration
  `36c0b4b` of `S11-TD-UI-ZINDEX-LAYERS` (story 002 sibling),
  unblocking the story-003 dependency on the host module's existence.
- **PROMPT 904** (2026-05-15) -- `/dev-story` worker on
  `work/s14-ui-token-scale-typography` from base
  `origin/main@36c0b4b`; commit `aa1672b`. Authored typography token
  module + integration test bin + evidence document; migrated 9 UI
  surfaces from inline `font_size:` literals to symbolic
  `typography::` references. AC1-AC9 covered. Cargo policy applied.
  17/17 typography unit + integration tests + regression sweep across
  touched surfaces all PASS.
- **PROMPT 906** (2026-05-15) -- Integration `--no-ff` merge of
  `aa1672b` into prior `origin/main@36c0b4b`; integration commit
  `eb1c128`; zero conflicts; 12 files / +965 / -55. Pre-merge +
  post-merge verification per integration report; all checks GREEN.
- **PROMPT 908** (2026-05-15, this closure) -- `/story-done` paperwork
  closure on `origin/main@eb1c128` PROMPT 906 integration tip
  (latest `origin/main@42eae31` includes PROMPT 907 viewport-tests
  integration on top; eb1c128 reachable as ancestor). All AC1-AC9
  PASS verified against the integrated diff at `eb1c128`. Story
  flipped `Draft -> Done`; sprint-status row flipped `ready -> done`;
  no code touched; no smoke / Team-QA / gate-check / release-check /
  Sprint 14 close-out invoked.

### Conditions carried forward unchanged

- `S8-QA-001-W1` OPEN (Story 017 AC12 forbid-auto-closure preserved).
- `QA-COND-0005` Standard-tier accessibility accepted-risk (friend-game
  scope only -- this story does NOT advance Standard-tier
  text-size / WCAG conformance).
- `QA-COND-0006` playtest / fun-hypothesis validation accepted-risk /
  deferred.
- `PAW-TD-*-a` placeholder-art accept-risk across PAW-002..PAW-006.
- PROMPT 683-era runtime divergence question preserved (folded into
  Sprint 12 story 019 cannot-reproduce closure; third same-scope
  retest NOT authorised per `TQ-S12-C2`).
- Sprint 12 story 019 underlying drag-runtime bug NOT claimed fixed.
- `TQ-S12-C1..C7` preserved verbatim (TQ-S12-C7 explicitly NOT closed
  by this story).
- PROMPT 761 Polish->Release gate-check `FAIL` preserved at
  `production/gate-checks/gate-polish-release-2026-05-12.md` (NO
  retry attempted by PROMPT 908).
- Sprint 13 / Sprint 12 / Sprint 11 / Sprint 10 closeouts preserved
  unchanged.
- Sprint 13 close-out (`closed-with-conditions`, PROMPT 894) preserved.
- `S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001` backlog row NOT
  flipped.
- `S11-HUD-TIMER-EYEBALL-VISUAL-001` Sprint 13 carry remains
  human-operator-blocked, no LLM `/story-done` authorised.
- PROMPT 802 §9 producer-decision-2 (numeric values) STILL UNRESOLVED
  on `origin/main`; the typography module preamble documents the
  proposed default scale (13 / 15 / 18 / 22 / 30 / 40 px) as
  ratify-on-spec; named constants are the stable contract for
  consumers.
- PROMPT 802 §9 producer-decision-3 (lobby modal-panel vs
  full-viewport hero) STILL UNRESOLVED; remains blocker for
  `S12-UX-LOBBY-LAYOUT-MODAL-001`.
- PROMPT 802 §9 producer-decision-4 (auction lead-loss visual
  language) STILL UNRESOLVED; remains blocker for Nice to Have
  `S12-UX-AUCTION-LEAD-LOSS-STATE-001`.

### Explicitly NOT claimed

- public release readiness
- release-candidate readiness
- full game completion
- broad / Standard-tier accessibility completion
- playtest / fun-hypothesis validation
- full playable-client manual QA
- two-client GAME_OVER closure (`S8-QA-001-W1`)
- final-art / asset-production completion
- Polish->Release gate-check retry
- Stage advance from Polish to Release
- underlying drag-runtime bug fix (Sprint 12 story 019 closed
  cannot-reproduce, NOT bug-fixed)
- closure of `S11-HUD-TIMER-EYEBALL-VISUAL-001`
- closure of `S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001` backlog row
- `TQ-S12-C7` closure
- Sprint 14 close-out
- ratification of producer-decision-2 numeric font values
- font-asset switching (still bevy default font; weight tokens are
  semantic contract only)
- responsive viewport-driven font scaling
- migration of sprite-rendered text under `client/src/card_animations/`
  (only `bevy_ui` Text nodes are in scope)
- automated test of any other Sprint 14 row

### Downstream unblock notes

- **`S11-TD-UI-FLEX-STRIPS`** (story 004): unblocked typography-side --
  flex-strip primitives can reference `typography::H2 / H3 / BODY`
  for label / content sizing inside row / column containers without
  reintroducing inline `font_size:` literals; AC3 / AC7 grep guard
  enforces this discipline at PR / CI time.
- **`S12-TD-UI-OVERLAY-ALPHA-TOKEN-001`** (story 006): unblocked
  typography-side -- the connection-lost overlay (`H1` / `H3`) and
  result screen (`H1` / `H3` / `BODY` / `CAPTION`) demonstrate the
  H1 / H3 overlay-typography pairing for future overlay surfaces.
- **Tier 1 layout candidates** (HUD layout 015-017, lobby layout
  024-026, shop-auction Tier 1 015-018): unblocked typography-side --
  Tier 1 layout-composition prompts can place `bevy_ui::Text` nodes
  referencing semantic typography tokens; AC3 / AC7 grep guard
  catches any layout-PR-introduced regression. HUD / lobby / auction
  surfaces in this integration are migrated end-to-end so Tier 1
  layout PRs touch positioning / sizing, not font-size literals.
- **`S12-UX-GLOBAL-UI-DESIGN-SPEC-001`** (story 007, currently NOT
  authored on `origin/main`): the named-constant contract isolates
  consumer call sites; a future producer ratification of numeric
  values (PROMPT 802 §9 producer-decision-2) is a single-file edit
  to `client/src/ui/design_tokens/typography.rs` constants and does
  not require touching consumers.
