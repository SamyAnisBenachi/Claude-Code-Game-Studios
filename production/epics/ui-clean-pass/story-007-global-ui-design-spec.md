# Story 007: S12-UX-GLOBAL-UI-DESIGN-SPEC-001 -- Canonical Global UI Design Spec

> **Epic**: UI Clean-Pass
> **Story ID**: S12-UX-GLOBAL-UI-DESIGN-SPEC-001
> **Status**: Done (PROMPT 922 `/story-done`; closure source-of-truth `origin/main@3d99a0482d24ce89230159ac3565f6e823b97c04` = PROMPT 912 `--no-ff` integration merge of PROMPT 911 worker tip `f4ef52a45eb80b192a70acee35d9416f092ed971` into prior `origin/main@ab3da3e`; ancestor of latest `origin/main@f6e538f` = PROMPT 921 tip)
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

- [x] **AC1 -- Spec authored**: GIVEN the story commit, WHEN the new
  spec file is inspected, THEN `docs/ux/global-ui-design-spec.md`
  exists. Verification: file presence.
  **PROMPT 922 verdict: PASS** -- `docs/ux/global-ui-design-spec.md`
  NEW 637 lines present on `origin/main@3d99a04` via PROMPT 912
  `--no-ff` integration of PROMPT 911 worker tip `f4ef52a`
  (`git ls-tree -r origin/main -- docs/ux/global-ui-design-spec.md`
  returns blob `b3dc3054`).

- [x] **AC2 -- All required sections present**: GIVEN the spec, WHEN
  the table of contents is inspected, THEN at least sections §1
  (Status / No-Claim Banner), §2 (Scope Boundaries), §3 (Z-Index Layer
  System), §4 (Spacing Scale), §5 (Typography Hierarchy), §6 (Overlay
  Alpha Tokens), §7 (Color Tokens), §8 (Responsive Layout Rules), and
  §9 (Strip Composition Patterns) are present. Verification: heading
  scan.
  **PROMPT 922 verdict: PASS** -- `grep "^## §" docs/ux/global-ui-design-spec.md`
  returns 10 §-headed sections at lines 24/76/116/155/186/246/288/322/379/431:
  §1 Status, §2 Scope Boundaries, §3 Z-Index Layer System, §4 Spacing
  Scale, §5 Typography Hierarchy, §6 Overlay Alpha Tokens, §7 Color
  Tokens, §8 Responsive Layout Rules, §9 Strip Composition Patterns,
  §10 Component Specifications (stretch). All required §1-§9 present
  plus optional §10.

- [x] **AC3 -- Z-layer canonical values**: GIVEN §3, WHEN inspected,
  THEN it enumerates the 8 named layers (Background, World, Units,
  UiBase, UiOverlay, Modal, Toast, Debug) with integer values
  ordered strictly ascending and with sufficient gap to allow future
  intermediate layers. Verification: doc review against story 002's
  layer module.
  **PROMPT 922 verdict: PASS** -- §3 table at
  `docs/ux/global-ui-design-spec.md` lines 128-137 enumerates 8 named
  layers `Background=0` / `World=100` / `Units=200` / `UiBase=300` /
  `UiOverlay=400` / `Modal=500` / `Toast=600` / `Debug=700`, strictly
  ascending, 100-unit gap, `LAYER_MIN_GAP=10` audit floor at :139-142.
  Ratified verbatim from `client/src/ui/design_tokens/z_layers.rs`
  (story 002, PROMPT 903 `/story-done`).

- [x] **AC4 -- Spacing canonical values**: GIVEN §4, WHEN inspected,
  THEN at least 5 named spacing tokens (`SPACING_XS`, `SM`, `MD`,
  `LG`, `XL` or equivalent) are defined with strictly increasing
  pixel values. Verification: doc review.
  **PROMPT 922 verdict: PASS** -- §4 at spec lines 161-167 enumerates
  `SPACING_XS=4` / `SPACING_SM=8` / `SPACING_MD=16` / `SPACING_LG=24` /
  `SPACING_XL=32` (5 tokens, strictly ascending; geometric step
  approximately ×2 per spec :169-172). Consumed verbatim by story 004
  module `client/src/ui/design_tokens/spacing.rs` (PROMPT 919
  `/story-done`).

- [x] **AC5 -- Typography canonical values**: GIVEN §5, WHEN
  inspected, THEN at least 6 named semantic-size tokens (Caption,
  Body, H3, H2, H1, Display) are defined with strictly increasing
  pixel values, plus at least 3 font-weight tokens, plus a canonical
  line-height ratio. Verification: doc review against story 003's
  typography module.
  **PROMPT 922 verdict: PASS** -- §5 at spec lines 196-203 enumerates
  6 sizes `Caption=13` / `Body=15` / `H3=18` / `H2=22` / `H1=30` /
  `Display=40` (strictly ascending), :213-217 enumerates 3 weights
  `WEIGHT_REGULAR=400` / `WEIGHT_SEMIBOLD=600` / `WEIGHT_BOLD=700`,
  :226 names `LINE_HEIGHT_DEFAULT_RATIO=1.25`. Ratified verbatim from
  `client/src/ui/design_tokens/typography.rs` (story 003, PROMPT 908
  `/story-done`). Accessibility-floor guard rails preserved at
  :234-239 (Display ≥ 40 HUD gold floor; H2 ≥ 20 HUD resource floor).

- [x] **AC6 -- Overlay alpha canonical values**: GIVEN §6, WHEN
  inspected, THEN `OVERLAY_DIM_ALPHA` and `OVERLAY_SCRIM_ALPHA` are
  named with their canonical float values (0.0 < alpha < 1.0) and
  rationale. Verification: doc review against story 006's overlay
  module.
  **PROMPT 922 verdict: PASS** -- §6 at spec lines 261-263 names
  `OVERLAY_DIM_ALPHA=0.45` (preserves HUD `hud/mod.rs:34`),
  `OVERLAY_SCRIM_ALPHA=0.55` (consolidates shipped 0.46 result + 0.58
  settlement values), `OVERLAY_TOAST_ALPHA=0.80`, each with rationale
  paragraph. Per-token range invariant `0.0 < alpha < 1.0` at :265-267.
  Per-existing-literal migration mapping table at :272-276. Consumed
  verbatim by story 006 module `client/src/ui/design_tokens/overlays.rs`
  (PROMPT 921 `/story-done`).

- [x] **AC7 -- Color palette named**: GIVEN §7, WHEN inspected, THEN
  at least 6 named color tokens (primary, secondary, accent,
  surface, surface-elevated, semantic-error) are listed with RGB hex
  + Bevy `Color::srgb()` reference. Verification: doc review.
  **PROMPT 922 verdict: PASS** -- §7 at spec lines 296-303 enumerates
  8 named tokens `PRIMARY=#4A90E2` / `SECONDARY=#E29E4A` /
  `ACCENT=#F2C94C` / `SURFACE=#0A0D14` / `SURFACE_ELEVATED=#161B27` /
  `SEMANTIC_SUCCESS=#27AE60` / `SEMANTIC_WARNING=#F2994A` /
  `SEMANTIC_ERROR=#EB5757`, each with RGB hex + canonical
  `Color::srgb(...)` literal. Friend-game placeholder palette per
  :305-307; not WCAG contrast-checked. `PAW-TD-*-a` accept-risk
  preserved at :292.

- [x] **AC8 -- Responsive layout rules named**: GIVEN §8, WHEN
  inspected, THEN the minimum viewport (1366×768), target viewports
  (1920×1080, 1920×1200, 1280×960), max viewport (3840×2160), and
  aspect-ratio handling (16:9, 16:10, 4:3, 21:9) are enumerated.
  Verification: doc review against story 005's viewport matrix.
  **PROMPT 922 verdict: PASS** -- §8 canonical viewport matrix at
  spec lines 335-340 enumerates `1366×768` minimum (16:9, laptop
  default), `1920×1080` baseline (16:9), `1920×1200` (16:10),
  `1280×960` (4:3 legacy), `3840×2160` 4K max (16:9), `2560×1080`
  ultrawide (21:9). All four aspect ratios (16:9 / 16:10 / 4:3 / 21:9)
  enumerated. Ratified verbatim from
  `tests/integration/helpers/ui_viewport.rs::CANONICAL_VIEWPORTS`
  (story 005, PROMPT 909 `/story-done`). Per-class scaling rules
  table at :344-350; strip-height determinism invariant at :374-375.

- [x] **AC9 -- Strip composition patterns named**: GIVEN §9, WHEN
  inspected, THEN HeaderBar / HandBar / FooterBar canonical heights
  + flex direction + child alignment are defined. LaneBar is
  defined IFF it makes sense as bevy_ui (TBD). Verification: doc
  review against story 004's strip primitives.
  **PROMPT 922 verdict: PASS** -- §9 strip-primitive table at spec
  lines 392-397 enumerates `HeaderBar=60`px `Row` `SpaceBetween`
  `Center` top-anchored, `LaneBar=60`px `Row` `Center` `Center`
  (documented-only per story 004 worker discretion), `HandBar=180`px
  `Row` `Center` `End` bottom-anchored, `FooterBar=40`px `Row`
  `SpaceBetween` `Center` above-HandBar-anchored. Ratified verbatim
  from `tests/integration/fixtures/ui_viewport_baseline.rs` baseline
  fixture (story 005); landed as production primitive by story 004
  `client/src/ui/design_tokens/strips.rs` (PROMPT 919 `/story-done`).
  Strip column composition + default child spacing rules at :402-417.

- [x] **AC10 -- Spec adoption matrix present**: GIVEN the spec,
  WHEN inspected, THEN a "Spec adoption matrix" section enumerates
  which Sprint 14+ stories consume which spec sections (at minimum:
  story 002 → §3; story 003 → §5; story 004 → §4 + §9; story 005 →
  §8; story 006 → §6). Verification: doc review.
  **PROMPT 922 verdict: PASS** -- "Spec Adoption Matrix" section at
  spec line 477. Tier 0 token-module consumer table at :483-491
  enumerates story 002 (`S11-TD-UI-ZINDEX-LAYERS`) → §3; story 003
  (`S11-TD-UI-FONT-CONSTANTS`) → §5; story 004 (`S11-TD-UI-FLEX-STRIPS`)
  → §4 + §9; story 005 (`S11-TD-UI-VIEWPORT-INVARIANT-TESTS`) → §8;
  story 006 (`S12-TD-UI-OVERLAY-ALPHA-TOKEN-001`) → §6. Tier 1
  surface story consumers at :495-502, Tier 1 Should-priority
  adjacent rows at :506-512, Tier 0 Should-priority adjacent row at
  :516-518, Tier 3 deferred to Sprint 15 at :522-525.

- [x] **AC11 -- Friend-game scope boundary named**: GIVEN §2, WHEN
  inspected, THEN the friend-game-vs-Standard-tier scope boundary is
  explicitly stated; `QA-COND-0005` accessibility, `QA-COND-0006`
  playtest, and `PAW-TD-*-a` placeholder-art accept-risk are each
  named as out of spec scope. Verification: doc review.
  **PROMPT 922 verdict: PASS** -- §1 Status banner at spec :41-46
  enumerates `QA-COND-0005`, `QA-COND-0006`, and `PAW-TD-002-a` ..
  `PAW-TD-006-a` as explicitly NOT claimed; :54-72 enumerates each
  disposition as preserved-unchanged with per-condition rationale. §2
  Scope Boundaries at :78-104 explicitly states friend-game-vs-Standard-tier
  boundary; :85-89 lists Standard-tier accessibility components
  (WCAG contrast / ≥44px hit-targets / focus order / keyboard /
  screen-reader / colorblind / text scaling) as out-of-spec; :90-93
  names `PAW-TD-*-a` placeholder-art accept-risk preserved.

- [x] **AC12 -- Producer ratification checklist**: GIVEN the spec,
  WHEN inspected, THEN a producer-ratification section names the
  UX-designer + art-director sign-off rows per PROMPT 802 §9
  producer-decision-2. Verification: doc review.
  **PROMPT 922 verdict: PASS** -- "Producer Ratification Checklist"
  section at spec line 529 with §"Producer-decision-2 resolution"
  subsection at :537 closing PROMPT 802 §9 producer-decision-2 (numeric
  values for Tier 0 token modules). Sign-off rows table at :554-566
  names three ratification rows: **Producer** (PROMPT 911),
  **UX-designer** (PROMPT 911), **Art-director** (PROMPT 911), each
  with rationale. Ratification scope guard at :568-587 limits
  ratification to friend-game visual polish; explicitly does NOT
  ratify Standard-tier accessibility, final-art, playtest, or
  producer-decisions 3 / 4 / 5.

- [x] **AC13 -- No code change**: GIVEN the story commit, WHEN `git
  diff` is inspected, THEN no file under `client/`, `server/`,
  `shared/`, or `tests/` is modified. Verification: `git diff
  origin/main...HEAD -- 'client/**' 'server/**' 'shared/**' 'tests/**'`
  returns no output.
  **PROMPT 922 verdict: PASS** -- `git diff 3d99a04^1..3d99a04 --stat
  -- 'client/' 'server/' 'shared/' 'tests/' 'Cargo.toml' 'Cargo.lock'`
  EMPTY across the PROMPT 912 integration merge. PROMPT 911 worker
  changed 2 NEW files only (`docs/ux/global-ui-design-spec.md` 637
  lines + `production/qa/evidence/sprint-14-ui-foundation/global-ui-design-spec/doc-review-checklist.md`
  163 lines; 800 insertions total). No code surface touched.

- [x] **AC14 -- Friend-game scope preserved**: GIVEN the story
  commit, WHEN `QA-COND-0005`, `QA-COND-0006`, and `PAW-TD-*-a`
  accept-risk dispositions are inspected, THEN none of them has
  been flipped to `closed` by this story. Verification: `git diff`
  of `production/sprint-status.yaml` shows no accept-risk disposition
  change.
  **PROMPT 922 verdict: PASS** -- `git diff 3d99a04^1..3d99a04 --stat
  -- 'production/sprint-status.yaml'` EMPTY across the PROMPT 912
  integration merge; no `QA-COND-0005` / `QA-COND-0006` / `PAW-TD-*-a`
  field touched by worker or integration. Accept-risk dispositions
  preserved verbatim throughout PROMPT 911 / 912 / 922. PROMPT 922
  row-level flip is the permitted disposition-preserving paperwork
  edit per /story-done scope.

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

---

## Closure Trail

| PROMPT | Action | Commit / Reference |
|--------|--------|---------------------|
| 878 | Authored story file (Sprint 14 candidate; not activated) | `production/epics/ui-clean-pass/story-007-global-ui-design-spec.md` NEW |
| 893 | Sprint 14 candidate-story authoring batch integration | merge `9f36663` |
| 897 | Sprint 14 activation (story row `status: ready`) | `origin/main@fffaf1c` |
| 898 | Sprint 14 QA plan authored covering this row | `production/qa/qa-plan-sprint-14.md` NEW |
| 910 | `/story-readiness` re-run -- verdict READY | `reports/PROMPT-910-S14-GLOBAL-UI-DESIGN-SPEC-READINESS.md` (gitignored) |
| 911 | `/dev-story` worker -- authored `docs/ux/global-ui-design-spec.md` (NEW 637 lines) + `production/qa/evidence/sprint-14-ui-foundation/global-ui-design-spec/doc-review-checklist.md` (NEW 163 lines) on branch `work/s14-global-ui-design-spec` | worker commit `f4ef52a45eb80b192a70acee35d9416f092ed971` |
| 912 | Integration `--no-ff` merge of worker into `origin/main@ab3da3e` (zero conflicts; PROMPT 911 worker reachable as merge's second-parent) | merge commit `3d99a0482d24ce89230159ac3565f6e823b97c04`, fast-forward push `ab3da3e..3d99a04` |
| 922 | `/story-done` paperwork closure: row `status: ready -> done`; AC1-AC14 checkboxes `[ ] -> [x]`; this Closure Trail appended | paperwork commit on `origin/main` from worktree `D:/_DEV/wt/ccgs-prompt-922-storydone` |

### AC verdicts (PROMPT 922 closure)

| AC | Title | Verdict | Evidence |
|----|-------|---------|----------|
| AC1 | Spec authored | **PASS** | `docs/ux/global-ui-design-spec.md` NEW 637 lines present on `origin/main@3d99a04`. |
| AC2 | All required sections present | **PASS** | `grep "^## §"` returns 10 sections §1..§10; required §1..§9 all present. |
| AC3 | Z-layer canonical values | **PASS** | §3 enumerates 8 layers Background=0..Debug=700 strictly ascending with 100-unit gap; cited from `client/src/ui/design_tokens/z_layers.rs`. |
| AC4 | Spacing canonical values | **PASS** | §4 enumerates `SPACING_XS=4 / SM=8 / MD=16 / LG=24 / XL=32` strictly ascending. |
| AC5 | Typography canonical values | **PASS** | §5 enumerates 6 sizes Caption=13..Display=40 + 3 weights + `LINE_HEIGHT_DEFAULT_RATIO=1.25`. |
| AC6 | Overlay alpha canonical values | **PASS** | §6 names `OVERLAY_DIM_ALPHA=0.45` + `OVERLAY_SCRIM_ALPHA=0.55` + `OVERLAY_TOAST_ALPHA=0.80` with rationale + per-existing-literal migration mapping. |
| AC7 | Color palette named | **PASS** | §7 enumerates 8 tokens (`PRIMARY`/`SECONDARY`/`ACCENT`/`SURFACE`/`SURFACE_ELEVATED`/`SEMANTIC_SUCCESS`/`SEMANTIC_WARNING`/`SEMANTIC_ERROR`) with RGB hex + `Color::srgb()` literal. Friend-game placeholder palette. |
| AC8 | Responsive layout rules named | **PASS** | §8 enumerates 6-viewport matrix (1366×768 / 1920×1080 / 1920×1200 / 1280×960 / 3840×2160 / 2560×1080) with 16:9 / 16:10 / 4:3 / 21:9 aspect ratios. |
| AC9 | Strip composition patterns named | **PASS** | §9 enumerates `HeaderBar=60` / `LaneBar=60` (bevy_ui deferred) / `HandBar=180` / `FooterBar=40` px with flex direction + child alignment. |
| AC10 | Spec adoption matrix present | **PASS** | "Spec Adoption Matrix" section enumerates story 002→§3 / 003→§5 / 004→§4+§9 / 005→§8 / 006→§6 + Tier 1 + Tier 3 deferred. |
| AC11 | Friend-game scope boundary named | **PASS** | §1 Status Banner + §2 Scope Boundaries enumerate `QA-COND-0005` + `QA-COND-0006` + `PAW-TD-*-a` accept-risk verbatim as out-of-spec scope. |
| AC12 | Producer ratification checklist | **PASS** | "Producer Ratification Checklist" names producer + UX-designer + art-director sign-off rows; PROMPT 802 §9 producer-decision-2 RESOLVED. |
| AC13 | No code change | **PASS** | `git diff 3d99a04^1..3d99a04 -- client/ server/ shared/ tests/ Cargo.toml Cargo.lock` empty. 2 NEW files only: spec (+637) + evidence checklist (+163). |
| AC14 | Friend-game scope preserved | **PASS** | `git diff 3d99a04^1..3d99a04 -- production/sprint-status.yaml` empty across worker + integration; no accept-risk row touched. |

### Conditions carried forward unchanged

- `S8-QA-001-W1` (two-client GAME_OVER manual / browser gap) remains **OPEN**.
- `QA-COND-0005` (Standard-tier accessibility) remains **accepted-risk** (friend-game scope only; lobby `LOBBY_BUTTON_HEIGHT = 30.0` defect preserved).
- `QA-COND-0006` (playtest / fun-hypothesis validation) remains **accepted-risk / deferred**.
- `PAW-TD-002-a` ... `PAW-TD-006-a` remain **accepted-risk**. §7 Color Tokens is friend-game placeholder palette.
- PROMPT 761 Polish->Release gate-check `FAIL` preserved; **NO** retry attempted by PROMPT 922.
- Stage UNCHANGED `Polish`.
- Sprint 14 disposition UNCHANGED `active`.
- Sprint 12 story 019 underlying drag-runtime bug NOT claimed fixed; third same-scope retest NOT authorised per `TQ-S12-C2`.
- `TQ-S12-C1..C7` preserved verbatim.
- Sprint 13 / 12 / 11 / 10 closeouts preserved unchanged.
- All prior Sprint 14 `/story-done` closures preserved verbatim: PROMPT 909 (story 005) + PROMPT 908 (story 003) + PROMPT 903 (story 002) + PROMPT 919 (story 004) + PROMPT 921 (story 006). PROMPT 922 entry appended as sixth `sprint_14_story_done` list item.
- `S11-HUD-TIMER-EYEBALL-VISUAL-001` Sprint 14 Should Have carry preserved unchanged (status: ready, human-operator-blocked).
- `S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001` backlog row remains as-is.
- PROMPT 802 §9 producer-decision-2 RESOLVED in spec body; producer-decision-3 / -4 STILL UNRESOLVED.

### Explicitly NOT claimed by PROMPT 922

- Public release readiness / RC readiness / full game completion.
- Broad / Standard-tier accessibility completion.
- Playtest / fun-hypothesis validation.
- Full playable-client manual QA.
- Two-client GAME_OVER closure (`S8-QA-001-W1`).
- Final-art / asset-production completion.
- Polish->Release gate-check retry.
- Stage advance from `Polish` to `Release`.
- Underlying drag-runtime bug fix.
- Sprint 14 close-out (Sprint 14 remains `active`; 6 of 17 rows closed after PROMPT 922; Tier 0 foundation now 6 of 6 landed).
- Closure of `S11-HUD-TIMER-EYEBALL-VISUAL-001`, `S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001`, or `TQ-S12-C7`.
- Closure of any other Sprint 14 row (only `S12-UX-GLOBAL-UI-DESIGN-SPEC-001` flipped by PROMPT 922).
- Tier 1 surface story `/dev-story` or `/story-done`.
- PROMPT 802 §9 producer-decision-3 / -4 resolution.
- Sprint 15 planning.

### Downstream unblock

With this `/story-done`, Tier 0 foundation is now **6 of 6 landed** on `origin/main`:

1. `S11-TD-UI-ZINDEX-LAYERS` (rank 1, PROMPT 903)
2. `S11-TD-UI-FONT-CONSTANTS` (rank 2, PROMPT 908)
3. `S11-TD-UI-FLEX-STRIPS` (rank 3, PROMPT 919)
4. `S11-TD-UI-VIEWPORT-INVARIANT-TESTS` (rank 4, PROMPT 909)
5. `S12-TD-UI-OVERLAY-ALPHA-TOKEN-001` (rank 5, PROMPT 921)
6. `S12-UX-GLOBAL-UI-DESIGN-SPEC-001` (rank 6, **PROMPT 922 -- this row**)

All Tier 1 surface stories (ranks 7-12) can now consume the canonical spec + the five Tier 0 token modules implementing it. Story 005's provisional baseline-fixture numeric values are ratified by §8 + §9 of this spec.
