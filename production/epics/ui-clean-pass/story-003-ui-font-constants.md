# Story 003: S11-TD-UI-FONT-CONSTANTS -- Typography Scale Tokens (Single Source Of Truth)

> **Epic**: UI Clean-Pass
> **Story ID**: S11-TD-UI-FONT-CONSTANTS
> **Status**: Draft (Sprint 14 candidate; NOT activated)
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

- [ ] **AC1 -- Typography token module authored**: GIVEN the story
  commit, WHEN the new module is inspected, THEN it exports at least 6
  named semantic-size constants (Caption, Body, H3, H2, H1, Display)
  with strictly increasing pixel values, plus at least 3 named
  font-weight constants. Verification: code review + unit test
  asserting the ordering Caption < Body < H3 < H2 < H1 < Display holds
  and each constant resolves to a positive `f32`.

- [ ] **AC2 -- Line-height ratio constant**: GIVEN the new module, WHEN
  inspected, THEN a canonical line-height ratio constant exists (e.g.
  `LINE_HEIGHT_DEFAULT_RATIO: f32`) with a doc comment naming its
  intended usage. Verification: code review.

- [ ] **AC3 -- All inline `font_size` literals migrated**: GIVEN the
  story commit, WHEN `client/src/ui/` is grepped, THEN no inline
  `font_size: Val::Px(N)` or `font_size: N.0` numeric literal remains
  outside the design-token module. Verification: `rg "font_size:\s*(Val::Px\(|[0-9])"
  client/src/ --glob '!client/src/ui/design_tokens/**'` returns zero hits
  (exact glob TBD by worker).

- [ ] **AC4 -- HUD constants subsumed**: GIVEN the story commit, WHEN
  `client/src/ui/hud/mod.rs` is inspected, THEN the HUD-local constants
  `HUD_GOLD_FONT_SIZE_PX`, `HUD_RESOURCE_TEXT_MIN_SIZE_PX`,
  `HUD_RESERVED_GOLD_FONT_SIZE_PX`, `HUD_SECONDARY_FONT_SIZE_PX` either
  resolve through the new module's constants or have been removed in
  favour of direct references to the new module. Verification: code
  review.

- [ ] **AC5 -- Result screen migrated**: GIVEN the story commit, WHEN
  `client/src/presentation/result_screen.rs` is inspected, THEN the
  existing 36 / 18 / 15 font sizes are replaced with the new module's
  H1 / H3 / Body (or equivalent semantic mapping). Verification: visual
  capture comparison against pre-migration baseline ensuring the result
  screen still reads with the same visual hierarchy.

- [ ] **AC6 -- Lobby typography inversion fixed**: GIVEN the story
  commit, WHEN `client/src/ui/lobby.rs` is inspected against the
  PROMPT 802 §3.1 L6 finding, THEN labels are no longer smaller than
  the data they describe; primary CTAs are no longer smaller than the
  status banner. Specifically: labels and CTAs are mapped to a
  semantic level at least as large as the data text. Verification:
  code review + visual capture at 1366×768 and 1920×1080.

- [ ] **AC7 -- Grep guard**: GIVEN the story commit, WHEN
  `client/src/` is grepped (excluding the design-token module), THEN
  no inline `font_size: Val::Px(N)` numeric literal remains outside the
  design-token module. Verification: `rg` pattern from AC3.

- [ ] **AC8 -- Unit tests pass**: GIVEN the story commit, WHEN the
  Rust test suite is run, THEN AC1 ordering test + any new typography
  unit tests pass. Verification: `cargo test -p client --lib
  typography` (or equivalent) returns success.

- [ ] **AC9 -- Friend-game scope preserved**: GIVEN the story commit,
  WHEN `QA-COND-0005`, `QA-COND-0006`, and `PAW-TD-*-a` accept-risk
  dispositions are inspected, THEN none of them has been flipped to
  `closed` by this story. Verification: `git diff` of
  `production/sprint-status.yaml` shows no accept-risk disposition
  change.

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
