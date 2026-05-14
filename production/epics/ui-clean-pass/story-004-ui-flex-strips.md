# Story 004: S11-TD-UI-FLEX-STRIPS -- Flex-Based UI Strip Composition Primitives

> **Epic**: UI Clean-Pass
> **Story ID**: S11-TD-UI-FLEX-STRIPS
> **Status**: Draft (Sprint 14 candidate; NOT activated)
> **Layer**: Presentation / UX foundational tech-debt
> **Type**: Tech Debt -- foundational primitive
> **Sprint**: Sprint 14 candidate (Tier 0 foundational; PROMPT 802 §4 rank 0.3;
> `docs/ux/ui-clean-pass-roadmap.md` rank 3). NOT activated by this authoring
> run. Sprint 13 disposition (`active`, `Polish` stage) preserved.
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

- [ ] **AC1 -- Strip primitive module authored**: GIVEN the story
  commit, WHEN the new module is inspected, THEN it exports at least
  three named strip primitives (`HeaderBar`, `HandBar`, `FooterBar`),
  each with a `Display::Flex` parent and documented
  `flex_direction` / `align_items` / `justify_content`. `LaneBar` is
  exported IFF it makes sense as bevy_ui (worker decision; documented
  in the worker report either way). Verification: code review.

- [ ] **AC2 -- Spacing scale constants**: GIVEN the new module, WHEN
  inspected, THEN a shared spacing-scale constant set exists (e.g.
  `SPACING_XS / SM / MD / LG / XL` or equivalent named scale) with
  strictly increasing pixel values and doc comments naming intended
  usage. Verification: code review + unit test asserting ordering.

- [ ] **AC3 -- HUD top strip migrated**: GIVEN the story commit, WHEN
  `client/src/ui/hud/mod.rs` is inspected, THEN the gold / mana / phase
  row is spawned via the `HeaderBar` primitive with flex children
  rather than absolute-positioned `hud_margin + N` magic offsets.
  Specifically, `HUD_GOLD_ROW_GAP_PX = 48.0` and
  `HUD_SECONDARY_ROW_GAP_PX = 28.0` either resolve through the new
  spacing-scale constants or are removed. Verification: code review +
  visual capture at 1366×768 / 1920×1080 / 4K.

- [ ] **AC4 -- HUD bottom strip migrated**: GIVEN the story commit,
  WHEN `client/src/ui/hud/mod.rs` is inspected, THEN the figurine
  area + reserve-strip readouts are spawned via the `FooterBar`
  primitive. The figurine `bottom: hud_margin + 60.0` magic offset
  is replaced with strip-relative anchoring. Verification: code review
  + visual capture.

- [ ] **AC5 -- Hand UI card row migrated**: GIVEN the story commit,
  WHEN `client/src/ui/hand/mod.rs` is inspected, THEN the card row is
  spawned inside the `HandBar` primitive. The existing card-fan layout
  (`f190cc7` repair: 7 chrome children 100×100% / 20×20% / 15×15%) is
  preserved unchanged inside the strip; only the *parent strip*
  composition changes. Verification: code review + visual capture
  confirming card fan still reads correctly.

- [ ] **AC6 -- Stable dimensions across viewport ratios**: GIVEN the
  story commit, WHEN the playable client is spawned at 16:9
  (1920×1080), 16:10 (1920×1200), and 4:3 (1280×960), THEN each
  migrated strip:
  - Has a deterministic pixel height (same across all three viewports).
  - Spans the full viewport width.
  - Does not overflow any other strip.
  - Does not clip any of its flex children.
  Verification: visual capture at the three viewport ratios stored
  under the evidence path.

- [ ] **AC7 -- No per-module `_GAP_PX` magic constants remain (HUD)**:
  GIVEN the story commit, WHEN `client/src/ui/hud/mod.rs` is grepped
  for `_GAP_PX`, THEN any remaining constants either reference the new
  spacing-scale module or have been removed. Verification: code review.

- [ ] **AC8 -- Strip primitive unit test**: GIVEN the story commit,
  WHEN the strip primitive unit test is run, THEN each strip's `Node`
  style fields match the expected (Display::Flex, deterministic height,
  100% width, documented justification). Verification: `cargo test -p
  client --lib ui_strips` (or equivalent).

- [ ] **AC9 -- Friend-game scope preserved**: GIVEN the story commit,
  WHEN `QA-COND-0005`, `QA-COND-0006`, and `PAW-TD-*-a` accept-risk
  dispositions are inspected, THEN none of them has been flipped to
  `closed` by this story. Verification: `git diff` of
  `production/sprint-status.yaml` shows no accept-risk disposition
  change.

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
