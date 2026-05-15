# Story 007: S12-UX-GLOBAL-UI-DESIGN-SPEC-001 -- Canonical Global UI Design Spec

> **Epic**: UI Clean-Pass
> **Story ID**: S12-UX-GLOBAL-UI-DESIGN-SPEC-001
> **Status**: Draft (Sprint 14 candidate; NOT activated)
> **Layer**: UX / Producer-planning / Design-spec authoring
> **Type**: UX -- design-spec authoring (doc-only)
> **Sprint**: Sprint 14 candidate (Tier 0 foundational; PROMPT 802 §4 rank 0.7;
> `docs/ux/ui-clean-pass-roadmap.md` rank 6). NOT activated by this authoring
> run. Sprint 13 disposition (`active`, `Polish` stage) preserved.
> **Authored**: 2026-05-14 by PROMPT 878
> **Authoring source-of-truth**: `origin/main@51e6228` (PROMPT 871 `/story-done`
> on Sprint 13 row `S13-TWO-CLIENT-RUNTIME-HARNESS-001`)
> **Estimated effort**: ~1.0d (PROMPT 802 §4 Tier 0.7)

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

PROMPT 802 §3.9 G6 surfaced that there is no global UI design spec
document. The closest existing artifacts are per-system UX docs
(`design/ux/hand-ui.md`, `design/ux/shop-auction-ui.md`, etc.) and per-
GDD design docs, none of which establish a *cross-cutting* spec for
layers, spacing, typography, alpha, color, and responsive rules. The
absence is the root cause of every Tier 0 numeric drift: stories 002
(z-layers), 003 (typography), 004 (flex strips + spacing), and 006
(overlay alpha) each have to either propose their own numeric values
or call them out as ratify-on-spec.

This story authors a canonical global UI design spec at
`docs/ux/global-ui-design-spec.md` consolidating all design tokens
(layers, spacing, typography, alpha, color) and responsive-layout rules
into one source of truth. The Tier 0 token modules from stories 002 /
003 / 004 / 006 then read their numeric values from this spec.

Per `docs/ux/ui-clean-pass-roadmap.md` §3 "Sequencing Rules" rule 2,
this story **should land first** in Phase 1 because the other Tier 0
token modules need its numeric values as input. Per PROMPT 802 §9
producer-decision-2, this is also a producer-decision item: UX-designer
+ art-director must ratify the spec before token modules consume it.

---

## Scope

### In Scope

- A new design-spec document at `docs/ux/global-ui-design-spec.md`
  (NEW) covering at least the following sections:
  - **§1 Status / No-Claim Banner**: friend-game scope only;
    `QA-COND-0005` / `QA-COND-0006` / `PAW-TD-*-a` accept-risk
    preserved; this spec does not advance Standard-tier accessibility.
  - **§2 Scope Boundaries**: explicit friend-game-vs-Standard-tier
    scope; this spec does NOT govern accessibility (WCAG contrast,
    ≥44px hit-targets, keyboard navigation, screen reader, colorblind,
    text scaling).
  - **§3 Z-Index Layer System**: canonical 8-layer ordering
    (Background, World, Units, UiBase, UiOverlay, Modal, Toast,
    Debug) with integer values for each (e.g. multiples of 100), doc
    text on which surfaces live at which layer. Source of truth for
    story 002's `S11-TD-UI-ZINDEX-LAYERS` module.
  - **§4 Spacing Scale**: canonical spacing tokens (e.g.
    `SPACING_XS / SM / MD / LG / XL` with pixel values, suggested
    4 / 8 / 16 / 24 / 32 pixel scale). Source of truth for story
    004's `S11-TD-UI-FLEX-STRIPS` module.
  - **§5 Typography Hierarchy**: canonical semantic levels (Caption
    / Body / H3 / H2 / H1 / Display) with pixel sizes, font-weight
    constants, and line-height ratio. Source of truth for story 003's
    `S11-TD-UI-FONT-CONSTANTS` module.
  - **§6 Overlay Alpha Tokens**: canonical `OVERLAY_DIM_ALPHA` and
    `OVERLAY_SCRIM_ALPHA` values with rationale. Source of truth for
    story 006's `S12-TD-UI-OVERLAY-ALPHA-TOKEN-001` module.
  - **§7 Color Tokens**: canonical color palette (primary, secondary,
    accent, semantic-success, semantic-warning, semantic-error,
    surface, surface-elevated). Each token named + RGB hex + Bevy
    `Color::srgb()` constructor reference. Friend-game palette only;
    not WCAG contrast-checked.
  - **§8 Responsive Layout Rules**: minimum viewport (1366×768),
    target viewports (1920×1080, 1920×1200, 1280×960), max viewport
    (3840×2160), aspect ratio handling (16:9, 16:10, 4:3, 21:9). Rules
    on what scales with viewport (e.g. world board) vs what stays
    pixel-fixed (e.g. strip heights, font sizes). Source of truth for
    story 005's `S11-TD-UI-VIEWPORT-INVARIANT-TESTS` baseline fixture.
  - **§9 Strip Composition Patterns**: HeaderBar / LaneBar / HandBar /
    FooterBar canonical heights + flex direction + child alignment.
    Source of truth for story 004's strip primitives.
  - **§10 Component Specifications** (optional / stretch): primary
    button vs secondary button affordance, panel chrome, card slot
    composition, modal centering pattern.
- Cross-references from the spec to existing per-system UX docs
  (`design/ux/hand-ui.md`, `design/ux/hud.md`, `design/ux/shop-auction-ui.md`,
  `design/ux/lobby.md` if it exists, `design/ux/board-rendering-spec.md`
  if it exists -- the latter is also a separate Tier 3 story 14
  `S11-UX-BOARD-RENDERING-SPEC`).
- A "Spec adoption matrix" enumerating which Sprint 14+ stories
  consume which sections of the spec (story 002 reads §3; story 003
  reads §5; story 004 reads §4 + §9; story 005 reads §8; story 006
  reads §6).
- A "Producer ratification checklist" naming the UX-designer +
  art-director sign-off rows (per PROMPT 802 §9 producer-decision-2).

### Out of Scope

- **No Sprint 14 activation** by this story.
- **No public release readiness** work.
- **No Standard-tier accessibility** (`QA-COND-0005`) completion. The
  spec explicitly DOES NOT govern accessibility. A separate
  accessibility spec is a follow-on scope.
- **No final-art / asset-production** work (`PAW-TD-*-a`). Color
  palette and font asset selection are *placeholder palette / font*
  for friend-game scope; final-art replacement is a separate sprint.
- **No playtest validation** (`QA-COND-0006`).
- **No code change** under `client/`, `server/`, `shared/`, `tests/`.
  This story authors a design document only. Token-module
  implementation is in stories 002 / 003 / 004 / 005 / 006.
- **No board-rendering spec**. `design/ux/board-rendering-spec.md` is
  authored by Tier 3 story 14 (`S11-UX-BOARD-RENDERING-SPEC`),
  separate from this story. This story may reference it as a
  cross-link if it exists.
- **No HUD per-element layout spec**. The spec covers cross-cutting
  tokens and patterns; per-element layout (e.g. HUD top-strip exact
  child order) is owned by the per-surface Tier 1 stories.
- **No animation / motion** spec. Tween / transition spec is a
  separate scope.
- **No interaction-state primitive** spec. That is covered by the
  Tier 0 Should-priority adjacent row
  `S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001`, not this story.
- **No localization** spec.
- **No update of `design/ux/` per-system docs**. Cross-links only;
  per-system doc edits are out of scope here.

---

## Acceptance Criteria

All criteria are independently checkable BLOCKING criteria.

- [ ] **AC1 -- Spec authored**: GIVEN the story commit, WHEN the new
  spec file is inspected, THEN `docs/ux/global-ui-design-spec.md`
  exists. Verification: file presence.

- [ ] **AC2 -- All required sections present**: GIVEN the spec, WHEN
  the table of contents is inspected, THEN at least sections §1
  (Status / No-Claim Banner), §2 (Scope Boundaries), §3 (Z-Index Layer
  System), §4 (Spacing Scale), §5 (Typography Hierarchy), §6 (Overlay
  Alpha Tokens), §7 (Color Tokens), §8 (Responsive Layout Rules), and
  §9 (Strip Composition Patterns) are present. Verification: heading
  scan.

- [ ] **AC3 -- Z-layer canonical values**: GIVEN §3, WHEN inspected,
  THEN it enumerates the 8 named layers (Background, World, Units,
  UiBase, UiOverlay, Modal, Toast, Debug) with integer values
  ordered strictly ascending and with sufficient gap to allow future
  intermediate layers. Verification: doc review against story 002's
  layer module.

- [ ] **AC4 -- Spacing canonical values**: GIVEN §4, WHEN inspected,
  THEN at least 5 named spacing tokens (`SPACING_XS`, `SM`, `MD`,
  `LG`, `XL` or equivalent) are defined with strictly increasing
  pixel values. Verification: doc review.

- [ ] **AC5 -- Typography canonical values**: GIVEN §5, WHEN
  inspected, THEN at least 6 named semantic-size tokens (Caption,
  Body, H3, H2, H1, Display) are defined with strictly increasing
  pixel values, plus at least 3 font-weight tokens, plus a canonical
  line-height ratio. Verification: doc review against story 003's
  typography module.

- [ ] **AC6 -- Overlay alpha canonical values**: GIVEN §6, WHEN
  inspected, THEN `OVERLAY_DIM_ALPHA` and `OVERLAY_SCRIM_ALPHA` are
  named with their canonical float values (0.0 < alpha < 1.0) and
  rationale. Verification: doc review against story 006's overlay
  module.

- [ ] **AC7 -- Color palette named**: GIVEN §7, WHEN inspected, THEN
  at least 6 named color tokens (primary, secondary, accent,
  surface, surface-elevated, semantic-error) are listed with RGB hex
  + Bevy `Color::srgb()` reference. Verification: doc review.

- [ ] **AC8 -- Responsive layout rules named**: GIVEN §8, WHEN
  inspected, THEN the minimum viewport (1366×768), target viewports
  (1920×1080, 1920×1200, 1280×960), max viewport (3840×2160), and
  aspect-ratio handling (16:9, 16:10, 4:3, 21:9) are enumerated.
  Verification: doc review against story 005's viewport matrix.

- [ ] **AC9 -- Strip composition patterns named**: GIVEN §9, WHEN
  inspected, THEN HeaderBar / HandBar / FooterBar canonical heights
  + flex direction + child alignment are defined. LaneBar is
  defined IFF it makes sense as bevy_ui (TBD). Verification: doc
  review against story 004's strip primitives.

- [ ] **AC10 -- Spec adoption matrix present**: GIVEN the spec,
  WHEN inspected, THEN a "Spec adoption matrix" section enumerates
  which Sprint 14+ stories consume which spec sections (at minimum:
  story 002 → §3; story 003 → §5; story 004 → §4 + §9; story 005 →
  §8; story 006 → §6). Verification: doc review.

- [ ] **AC11 -- Friend-game scope boundary named**: GIVEN §2, WHEN
  inspected, THEN the friend-game-vs-Standard-tier scope boundary is
  explicitly stated; `QA-COND-0005` accessibility, `QA-COND-0006`
  playtest, and `PAW-TD-*-a` placeholder-art accept-risk are each
  named as out of spec scope. Verification: doc review.

- [ ] **AC12 -- Producer ratification checklist**: GIVEN the spec,
  WHEN inspected, THEN a producer-ratification section names the
  UX-designer + art-director sign-off rows per PROMPT 802 §9
  producer-decision-2. Verification: doc review.

- [ ] **AC13 -- No code change**: GIVEN the story commit, WHEN `git
  diff` is inspected, THEN no file under `client/`, `server/`,
  `shared/`, or `tests/` is modified. Verification: `git diff
  origin/main...HEAD -- 'client/**' 'server/**' 'shared/**' 'tests/**'`
  returns no output.

- [ ] **AC14 -- Friend-game scope preserved**: GIVEN the story
  commit, WHEN `QA-COND-0005`, `QA-COND-0006`, and `PAW-TD-*-a`
  accept-risk dispositions are inspected, THEN none of them has
  been flipped to `closed` by this story. Verification: `git diff`
  of `production/sprint-status.yaml` shows no accept-risk disposition
  change.

---

## Evidence Path

`production/qa/evidence/sprint-14-ui-foundation/global-ui-design-spec/`

Expected artifacts:

- Doc-review checklist (markdown) confirming each AC against the
  authored spec.
- Section heading enumeration (`rg "^## " docs/ux/global-ui-design-spec.md`)
  for AC2 verification.
- Cross-reference matrix between spec sections and consumer stories
  (AC10).

---

## Likely Files Touched

| Path | Expected change |
|------|-----------------|
| `docs/ux/global-ui-design-spec.md` (NEW) | Author canonical global UI design spec. |
| `docs/ux/ui-clean-pass-roadmap.md` | Possible amendment to cross-link the new spec (optional; worker discretion). |
| `design/ux/*.md` (existing per-system UX docs) | Read-only; cross-links into the new spec optional. |

This table is a planning estimate. The implementation prompt is authoritative
for the realised set.

**Explicitly NOT touched** by this story:

- `client/src/**`
- `server/src/**`
- `shared/src/**`
- `tests/**`
- `Cargo.toml`, `Cargo.lock`
- `production/sprint-status.yaml`
- `production/sprints/sprint-13.md`, `production/sprints/sprint-14.md`
- `production/stage.txt`
- `production/session-state/**`

---

## Verification

- File presence check on `docs/ux/global-ui-design-spec.md`.
- `git diff --check origin/main...HEAD` -- clean diff.
- `git diff origin/main...HEAD -- 'client/**' 'server/**' 'shared/**' 'tests/**'`
  -- returns empty (AC13).
- Doc-review checklist (manual) -- AC2 through AC12.
- Reviewer sign-off from UX-designer + art-director on AC12 ratification
  checklist (paper sign-off acceptable).

---

## Dependencies / Sequencing

- **Authoring prompt (this PROMPT 878)** is the *story-authoring* prompt;
  it creates the story file only. No `/dev-story` runs here.
- **Activation**: Requires Sprint 14 activation (separate prompt; not this
  one). Cannot land in Sprint 13.
- **Sequencing within Tier 0** (per `docs/ux/ui-clean-pass-roadmap.md`
  §3 "Sequencing Rules" rule 2): this story **should land first** in
  Phase 1 because Tier 0 token modules (stories 002 / 003 / 004 / 006)
  need its numeric values as input. Story 005 (viewport-invariant tests)
  also reads its baseline-fixture canonical viewport sizes from this
  spec's §8.
- **Sprint 14 critical-path note**: if Sprint 14 activation pulls all
  six Tier 0 stories in parallel, this story is the de-facto critical
  path because the other five each have a "ratify-on-spec" escape hatch
  but converge cleanly only once this spec lands. Producer should
  schedule this story first (or at least mark it as the highest-priority
  Tier 0 Must-Have row).
- **Producer-decision dependency** (per PROMPT 802 §9 producer-
  decision-2): UX-designer + art-director must ratify the spec values
  before stories 002 / 003 / 004 / 006 cite this spec as their source
  of truth.
- **Tier 1 surface stories depend on this transitively**: every Tier 1
  per-surface layout story consumes the spec via the Tier 0 token
  modules.

---

## Notes

- PROMPT 802 §3.9 G6: no global UI design spec doc.
- PROMPT 802 §9 producer-decision-2: UX-designer + art-director
  decision on the global UI design spec is named as a prerequisite
  for Tier 0 token modules.
- `docs/ux/ui-clean-pass-roadmap.md` §3 "Sequencing Rules" rule 2:
  explicitly names this story as "authored first in Phase 1".
- The 10 spec sections (§1..§10) come from PROMPT 878's task brief
  plus PROMPT 802 §3.9 G1..G7 cross-cutting findings. The exact
  numeric values within each section are owned by the UX-designer +
  art-director per AC12.
- Accept-risk preservation: `PAW-TD-*-a`, `QA-COND-0005`, `QA-COND-0006`
  preserved unchanged. This story does not advance any of them.
