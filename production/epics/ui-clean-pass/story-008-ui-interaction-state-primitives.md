# Story 008: S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001 -- UI Interaction State Primitives

> **Epic**: UI Clean-Pass
> **Story ID**: S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001
> **Status**: Done via PROMPT 1009 (2026-05-17) after PROMPT 1005
> `/dev-story` authored the primitive module + spec amendment and
> PROMPT 1007 integrated it onto
> `origin/main@5d36c4b4fe061981b6c1949f3f4f7986ca0cf0cb`.
> **Layer**: Presentation / UX foundational tech-debt
> **Type**: Tech Debt -- foundational primitive (design token)
> **Sprint**: Sprint 15 Nice to Have (Tier 0 Should-priority adjacent;
> `docs/ux/ui-clean-pass-roadmap.md` Tier 0 Should-priority adjacent row;
> PROMPT 802 §3.9 G7). Paired with rank 6
> (`S12-UX-GLOBAL-UI-DESIGN-SPEC-001` Done on Sprint 14 PROMPT 922).
> Sprint 15 activated by PROMPT 997; Sprint 14 disposition
> (`closed-with-conditions`, `Polish` stage) preserved.
> **Authored**: 2026-05-16 by PROMPT 993
> **Authoring source-of-truth**: `origin/main@2c84d6e37f2ec58b729064b6dbe4c9b017e5ceb3`
> (PROMPT 990 `integrate(s15): merge Sprint 15 plan draft (PROMPT 990)`).
> **Estimated effort**: ~1.0d (Sprint 15 plan Nice to Have row;
> `docs/ux/ui-clean-pass-roadmap.md` Tier 0 Should-priority adjacent row)

---

## Status / No-Claim Banner

This story is authored as a Sprint 15 candidate. **Sprint 15 is NOT activated
by this authoring run.** The story is paperwork only -- no code change is
attempted by PROMPT 993.

PROMPT 993 (this authoring run) does NOT:

- Activate Sprint 15.
- Modify `production/sprint-status.yaml`.
- Modify `production/sprints/sprint-14.md`, `production/sprints/sprint-15.md`,
  or any other sprint plan file.
- Modify `production/stage.txt` (remains `Polish`).
- Modify any `production/session-state/*` file.
- Modify any QA-plan / smoke / Team-QA / gate-check / release-check artifact.
- Run `/story-readiness`, `/dev-story`, `/story-done`, `/smoke-check`,
  `/team-qa`, `/gate-check`, `/release-check`, or `/qa-plan` on this story.
- Modify any code under `client/`, `server/`, `shared/`, or `tests/`.
- Modify `docs/ux/global-ui-design-spec.md` (the spec amendment that pairs
  with this primitive module is a future `/dev-story` deliverable, NOT a
  story-authoring deliverable).
- Author `client/src/ui/design_tokens/interaction_states.rs` (that is the
  future `/dev-story` output, not this authoring prompt's output).

This story does **not** claim: public release readiness, release-candidate
readiness, full game completion, broad / Standard-tier accessibility
completion (`QA-COND-0005`), Standard-tier hit-target conformance (≥44px),
playtest / fun-hypothesis validation (`QA-COND-0006`), full playable-client
manual QA, two-client GAME_OVER closure (`S8-QA-001-W1`), or final-art /
asset-production completion (`PAW-TD-*-a`).

---

## Overview

PROMPT 802 §3.9 G7 surfaced that the playable client has **no canonical
interaction-state primitive set**: hover, focus, pressed, and disabled
visual states for buttons / clickable surfaces are either absent or
authored per-site with ad-hoc literals. The Tier 1 surface stories landed
in Sprint 14 (lobby buttons per `S11-UX-LOBBY-BUTTON-HITTARGETS` DONE;
auction featured card per `S11-UX-AUCTION-FEATURED-CARD` DONE; HUD top
strip per `S11-UX-HUD-TOP-STRIP-LAYOUT` DONE) are tolerable without this
row but **degrade to per-site button styling** -- each surface picks its
own hover / pressed visual tweak, with no shared token.

`docs/ux/global-ui-design-spec.md` (Sprint 14 PROMPT 922 DONE) explicitly
defers interaction-state primitives to this row at §"Scope Boundaries"
line ~95-97 ("Interaction-state primitives — hover / focus / pressed /
disabled state is owned by the Tier 0 Should-priority adjacent row
`S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001`, not this spec") and at the
"Primary button affordance" subsection of §10 line ~442-443 ("Hover /
pressed: future scope under `S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001`
(Tier 0 Should-priority)"). The Spec Adoption Matrix row at line ~518
also names this row as the consumer of §7 color tokens + §10 button
affordance for hover / focus / pressed / disabled.

This story authors a new design-token primitive module at
`client/src/ui/design_tokens/interaction_states.rs` (future `/dev-story`
output, NOT authored by this story file) that exports named hover /
focus / pressed / disabled visual-state tokens consumable by any
Sprint 16+ button-surface migration. It also amends the global UI design
spec to add the named primitives and their default values to a new
spec section, paired with the existing §10 button-affordance subsection.

Per Sprint 15 plan draft (PROMPT 988):

- This is a **Nice to Have** row. It lands only if Sprint 15 Must Have +
  Should Have closure is on track.
- **Per-surface migration of existing Sprint 14 button surfaces** (lobby
  buttons, auction bid buttons, HUD action buttons, draft buttons,
  shop slot buttons) is **explicitly OUT OF SCOPE** for Sprint 15. The
  primitive module is authored and the spec body is amended, but the
  existing button call sites are NOT migrated by this row. Per-surface
  migration is a follow-on story (slug TBD; expected
  `S16-UI-INTERACTION-STATE-MIGRATION-*` family) to keep Sprint 15
  small and disjoint from the other Sprint 15 candidate rows
  (`S12-UX-HAND-DRAG-STATE-VISUALS-001` Should + `S11-UX-BOARD-RENDERING-SPEC`
  Should).

Per `docs/ux/ui-clean-pass-roadmap.md` §"Tier 0 Should-Priority Adjacent
Row" (the one-row table at line ~210-214), this row is "Net-new / PROMPT
802 §3.9 G7. Pair with rank 6 (design spec); ranks 7-12 are tolerable
without it but degrade to per-site button styling." Rank 6 (the global
UI design spec) is DONE on Sprint 14 PROMPT 922, so the pairing
prerequisite is satisfied at story-authoring time.

---

## Scope

### In Scope

- A new design-token primitive module at
  `client/src/ui/design_tokens/interaction_states.rs` (NEW; future
  `/dev-story` output, NOT authored by this story file). The module
  exports the following four named interaction-state token sets, each
  giving the visual transform applied to a base palette token (e.g.
  `PRIMARY`, `SURFACE_ELEVATED`) when the user-input state is the named
  state:
  - **`HOVER_*`** -- subtle highlight applied when the pointer is over
    a clickable surface but no mouse-button is pressed. Includes at
    least: `HOVER_BG_TINT_ALPHA: f32` (default ~`0.08`, RGBA white
    overlay on the base palette token), `HOVER_BORDER_ALPHA: f32`
    (default ~`0.40`), and a doc comment naming surfaces that consume
    it (e.g. lobby Join / Create buttons; auction bid buttons; HUD
    action buttons; shop slot purchase buttons).
  - **`FOCUS_*`** -- keyboard / accessibility focus ring applied when a
    clickable surface is the focused element via Tab navigation or
    equivalent. Includes at least: `FOCUS_RING_COLOR: Color` (default
    `ACCENT` palette token from spec §7), `FOCUS_RING_WIDTH_PX: f32`
    (default ~`2.0`), `FOCUS_RING_OFFSET_PX: f32` (default ~`2.0`),
    and a doc comment. **Note**: friend-game scope only; this token
    set provides a *visual* focus ring but does NOT implement full
    keyboard-navigation focus order, screen-reader hints, or
    Standard-tier focus conformance per `QA-COND-0005`.
  - **`PRESSED_*`** -- depressed-state visual applied while a mouse
    button is held down on a clickable surface. Includes at least:
    `PRESSED_BG_TINT_ALPHA: f32` (default ~`0.16`, RGBA black overlay
    on the base palette token), `PRESSED_OFFSET_Y_PX: f32` (default
    `1.0`, a one-pixel press-down nudge), and a doc comment.
  - **`DISABLED_*`** -- visual state applied when a clickable surface
    is not interactable in the current game state. Includes at least:
    `DISABLED_BG_TINT_ALPHA: f32` (default ~`0.50`, RGBA black overlay
    on the base palette token to flatten saturation),
    `DISABLED_TEXT_ALPHA: f32` (default ~`0.40`), `DISABLED_BORDER_ALPHA:
    f32` (default ~`0.20`), and a doc comment naming where disabled is
    the right surface state (e.g. auction bid button when the player
    already holds the lead; shop slot when the player cannot afford
    the unit).
- A doc comment on each token explaining the canonical surfaces that
  consume it. The exact pixel / alpha defaults above are
  worker-discretion within the order-of-magnitude bands listed
  (canonical numeric values are ratified by the global UI design spec
  amendment per the next bullet).
- An amendment to `docs/ux/global-ui-design-spec.md` (NEW spec section,
  future `/dev-story` output; NOT authored by this story file) adding
  the four interaction-state primitive sets to the spec body. The new
  spec section MUST:
  - Reference the new design-token module by file path.
  - List the four named token sets (hover / focus / pressed / disabled)
    with their canonical default values.
  - Cross-link from the existing §10 "Primary button affordance" /
    "Secondary button affordance" subsections so the deferral note
    ("Hover / pressed: future scope under
    `S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001`") flips to a forward
    reference to the new section.
  - Update the Spec Adoption Matrix row for
    `S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001` (currently line ~518
    of the spec, "§7 color tokens + §10 button affordance") to also
    name the new interaction-state spec section as the source of
    truth for its default values.
  - Preserve the friend-game-vs-Standard-tier scope boundary
    explicitly: the new section MUST state that focus-ring visual
    presence is friend-game scope only and does NOT advance
    `QA-COND-0005` Standard-tier focus-order / keyboard-navigation /
    screen-reader conformance.
- A re-export of the new module from `client/src/ui/design_tokens/mod.rs`
  (existing aggregator from Sprint 14 Tier 0 stories 002 / 003 / 004 /
  006), so consumers can `use client::ui::design_tokens::interaction_states::*;`
  (or the project-idiomatic equivalent).
- A new integration test bin at `tests/integration/ui_clean_pass/` (NEW
  path; exact filename TBD by future `/dev-story` worker -- suggested
  `interaction_state_primitives_test.rs`) asserting **primitive module
  shape**:
  - Each of the four named token sets is published from the module.
  - Each numeric default falls within a documented range
    (`0.0 <= alpha <= 1.0` for alpha tokens; `0.0 < px` for pixel
    tokens).
  - Each token has at least one doc comment line.
  - A grep guard prevents re-introduction of inline hover / pressed /
    disabled literal tweaks on the named Sprint 14 button surfaces
    *that the worker chooses to assert* (lobby / auction / HUD --
    full per-surface migration enforcement is the follow-on Sprint 16+
    story, not this row).
- A worker report enumerating any surfaces still using inline
  interaction-state literals at the time of `/dev-story` completion,
  flagged as deferred-to-Sprint-16+ per the "Per-surface migration
  OUT OF SCOPE" rule.

### Out of Scope

- **No Sprint 15 activation** by this story.
- **No public release readiness** work.
- **No Standard-tier accessibility (`QA-COND-0005`) completion.** The
  primitive module does NOT implement full keyboard-navigation focus
  order, screen-reader hints, ≥44px hit-target enforcement, WCAG
  contrast checking on the hover / pressed / disabled tints, colorblind
  modes, or text scaling. Visual focus-ring presence is friend-game
  scope only; it does not satisfy Standard-tier focus-order
  requirements. **No Standard-tier hit-target conformance is claimed.**
- **No broad accessibility completion** of any kind. The primitive
  module is a *visual* primitive, not an accessibility primitive.
- **No final-art / asset-production** work (`PAW-TD-*-a`). The hover /
  pressed / disabled tints are friend-game placeholder tweaks layered
  over the spec §7 placeholder palette; final-art replacement of the
  underlying palette is a separate sprint scope.
- **No playtest / fun-hypothesis validation** (`QA-COND-0006`). The
  primitives are not playtest-validated.
- **No per-surface migration of existing Sprint 14 button surfaces.**
  Lobby buttons (`S11-UX-LOBBY-BUTTON-HITTARGETS` DONE), auction bid
  buttons (`S11-UX-AUCTION-FEATURED-CARD` DONE), HUD action buttons
  (`S11-UX-HUD-TOP-STRIP-LAYOUT` DONE), draft buttons, shop slot
  buttons, and any other clickable surface remain on their existing
  per-site styling for the duration of Sprint 15. Per-surface migration
  is a Sprint 16+ follow-on story (expected slug family
  `S16-UI-INTERACTION-STATE-MIGRATION-*`).
- **No tween / animation** of state transitions. Static visual states
  only. Future per-state easing (e.g. 100 ms fade-in on hover enter) is
  a separate scope under the not-yet-authored animation / motion spec.
- **No re-pick of the canonical default values** outside the
  worker-proposed defaults above. The values listed are
  worker-suggested defaults intended to be ratified by the global UI
  design spec amendment (this story authors both -- the primitive
  module and the spec amendment -- so the defaults converge). If a
  Sprint 15+ producer needs to retune the defaults, that is a
  separate spec-amendment scope and is NOT triggered by this row.
- **No new color-palette tokens.** The four interaction-state token
  sets layer on top of the existing spec §7 palette
  (`PRIMARY` / `SURFACE_ELEVATED` / `ACCENT` / `SEMANTIC_*`); they do
  NOT introduce new base palette entries.
- **No HUD-timer urgency** color/alpha change (separate
  `S11-UX-HUD-TIMER-URGENCY-VISUAL-001` Tier 2 future candidate).
- **No board ghost preview opacity** change (separate
  `S11-UX-BOARD-GHOST-PREVIEW-OPACITY-001` Tier 2 future candidate).
- **No overlay alpha** change (separate
  `S12-TD-UI-OVERLAY-ALPHA-TOKEN-001` DONE Sprint 14 PROMPT 921).
- **No z-index / typography / flex-strip / viewport-invariant test**
  change (those are the existing Sprint 14 Tier 0 stories 002 / 003 /
  004 / 005).
- **No hand-drag-state visuals** (separate Sprint 15 Should row
  `S12-UX-HAND-DRAG-STATE-VISUALS-001`; orthogonal surface; hand UI
  drag state is not a button-affordance interaction state).
- **No board-rendering spec** (separate Sprint 15 Should row
  `S11-UX-BOARD-RENDERING-SPEC`; orthogonal surface; world-space spec
  doc-only).
- **No card-slot primitive refactor** (separate
  `S12-TD-UI-CARD-SLOT-PRIMITIVE-001` Tier 3 rank 13 deferred to
  Sprint 16+).
- **No `Polish->Release` gate-check retry** (PROMPT 761 FAIL preserved;
  no retry authorised in Sprint 15).
- **No stage advance** from `Polish` to `Release`.

---

## Acceptance Criteria

All criteria are independently checkable BLOCKING criteria. They define
the future `/dev-story` worker's deliverable; this authoring run does
**not** verify any of them (no AC is `[x]` until `/dev-story` completes
and `/story-done` runs).

- [x] **AC1 -- Interaction-state primitive module authored**: GIVEN
  the story commit, WHEN the new module file is inspected, THEN
  `client/src/ui/design_tokens/interaction_states.rs` exists, is
  declared from `client/src/ui/design_tokens/mod.rs`, and exports the
  four named token sets `HOVER_*`, `FOCUS_*`, `PRESSED_*`, `DISABLED_*`.
  Verification: file presence + module re-export + grep for the four
  token-set prefixes.

- [x] **AC2 -- Named hover tokens with defaults**: GIVEN the module,
  WHEN inspected, THEN at least the following hover tokens are
  exported with documented numeric defaults:
  - `HOVER_BG_TINT_ALPHA: f32` (range `0.0 <= alpha <= 1.0`, default
    in band `0.04..=0.16`).
  - `HOVER_BORDER_ALPHA: f32` (range `0.0 <= alpha <= 1.0`, default
    in band `0.20..=0.60`).
  Each token has a `///` doc comment naming canonical consumer
  surfaces. Verification: module read + integration-test assertions
  (AC8).

- [x] **AC3 -- Named focus tokens with defaults**: GIVEN the module,
  WHEN inspected, THEN at least the following focus tokens are
  exported with documented numeric defaults:
  - `FOCUS_RING_COLOR: Color` (default referenced from spec §7
    palette `ACCENT` token; not a fresh RGB triple).
  - `FOCUS_RING_WIDTH_PX: f32` (range `0.0 < px <= 8.0`, default in
    band `1.0..=3.0`).
  - `FOCUS_RING_OFFSET_PX: f32` (range `0.0 <= px <= 8.0`, default
    in band `0.0..=4.0`).
  Each token has a `///` doc comment. The doc comment for
  `FOCUS_RING_*` MUST explicitly state friend-game scope and that
  `QA-COND-0005` Standard-tier focus-order conformance is NOT
  advanced by token presence. Verification: module read +
  integration-test assertions (AC8) + doc-comment scan.

- [x] **AC4 -- Named pressed tokens with defaults**: GIVEN the module,
  WHEN inspected, THEN at least the following pressed tokens are
  exported with documented numeric defaults:
  - `PRESSED_BG_TINT_ALPHA: f32` (range `0.0 <= alpha <= 1.0`,
    default in band `0.08..=0.24`).
  - `PRESSED_OFFSET_Y_PX: f32` (range `0.0 <= px <= 4.0`, default in
    band `0.0..=2.0`).
  Each token has a `///` doc comment. Verification: module read +
  integration-test assertions (AC8).

- [x] **AC5 -- Named disabled tokens with defaults**: GIVEN the module,
  WHEN inspected, THEN at least the following disabled tokens are
  exported with documented numeric defaults:
  - `DISABLED_BG_TINT_ALPHA: f32` (range `0.0 <= alpha <= 1.0`,
    default in band `0.30..=0.70`).
  - `DISABLED_TEXT_ALPHA: f32` (range `0.0 <= alpha <= 1.0`, default
    in band `0.20..=0.60`).
  - `DISABLED_BORDER_ALPHA: f32` (range `0.0 <= alpha <= 1.0`,
    default in band `0.10..=0.40`).
  Each token has a `///` doc comment naming canonical
  disabled-state surfaces (e.g. auction bid button when already
  leading; shop slot when unaffordable). Verification: module read
  + integration-test assertions (AC8).

- [x] **AC6 -- Export shape**: GIVEN the module, WHEN consumed from
  a downstream file via `use client::ui::design_tokens::interaction_states::*;`
  (or project-idiomatic equivalent), THEN each named token from
  AC2..AC5 is resolvable as a public symbol. The module is
  re-exported from `client/src/ui/design_tokens/mod.rs` consistent
  with the existing Sprint 14 Tier 0 modules (z-layers / typography /
  spacing / strips / overlays). Verification: integration-test
  imports the module by its public path; compile success
  demonstrates export shape.

- [x] **AC7 -- Global UI spec amendment**: GIVEN the story commit,
  WHEN `docs/ux/global-ui-design-spec.md` is inspected, THEN:
  - A new spec section is present (heading style consistent with the
    spec's existing `## §N <Title>` pattern; suggested location
    after §10 "Component Specifications" or as a new §11
    "Interaction State Primitives") that names the four token sets
    (hover / focus / pressed / disabled), lists their canonical
    default values, and points at the new module file.
  - The existing §10 "Primary button affordance" / "Secondary button
    affordance" deferral notes ("Hover / pressed: future scope under
    `S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001`...") are amended to
    a forward reference to the new section.
  - The Spec Adoption Matrix row for
    `S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001` cites the new
    interaction-state section in addition to the existing §7 + §10
    citations.
  - The new section preserves the friend-game-vs-Standard-tier
    scope boundary verbatim: it MUST state that focus-ring presence
    is friend-game scope only and does NOT advance `QA-COND-0005`,
    `QA-COND-0006`, or `PAW-TD-*-a` accept-risk.
  Verification: spec heading scan + cross-reference grep.

- [x] **AC8 -- Integration test asserts primitive module shape**:
  GIVEN the story commit, WHEN the new integration test bin at
  `tests/integration/ui_clean_pass/` is run (suggested filename
  `interaction_state_primitives_test.rs`; exact name TBD by
  `/dev-story` worker), THEN it passes and asserts at minimum:
  - All four named token-set families (`HOVER_*`, `FOCUS_*`,
    `PRESSED_*`, `DISABLED_*`) are importable from the module's
    public path.
  - Each numeric alpha default falls in `0.0 <= alpha <= 1.0`.
  - Each numeric pixel default falls in `0.0 <= px` (with the
    upper-bound bands from AC2..AC5 enforced).
  - `FOCUS_RING_COLOR` ratifies the spec §7 `ACCENT` palette token
    (no fresh RGB triple introduced).
  - At least one doc-comment line is present on each named token
    (sanity check; not a full doc-coverage gate).
  The integration test is registered in the workspace
  `[[test]]` set under `tests/integration/ui_clean_pass/`,
  consistent with the existing Sprint 14 Tier 0 integration test
  pattern. Verification: `cargo test` of the new bin passes.

- [x] **AC9 -- No inline literal regressions on the module's own
  surface**: GIVEN the story commit, WHEN the new module file is
  inspected, THEN every numeric value in the module is named
  (`const NAME: f32 = ...;` or `pub const NAME: Color = ...;`)
  rather than introduced as an inline literal at a downstream use
  site. The module itself MUST NOT export `pub const` aliases that
  collapse to magic numbers without a comment. Verification: module
  read + integration-test sanity assertion (AC8 doc-comment scan).
  **Note**: this AC is *narrow* to the new module's own surface; it
  does NOT enforce per-surface migration of existing Sprint 14
  button surfaces (that is the Sprint 16+ follow-on story).

- [x] **AC10 -- Per-surface migration explicitly OUT OF SCOPE**:
  GIVEN the story commit, WHEN `git diff` is inspected, THEN no
  existing Sprint 14 button surface call site is migrated to read
  the new interaction-state tokens. Specifically:
  - `client/src/ui/lobby.rs` button styling at the
    `S11-UX-LOBBY-BUTTON-HITTARGETS` call sites is UNCHANGED.
  - `client/src/ui/shop_auction/mod.rs` auction bid button styling
    at the `S11-UX-AUCTION-FEATURED-CARD` call sites is UNCHANGED.
  - `client/src/ui/hud/mod.rs` HUD action / top-strip button
    styling at the `S11-UX-HUD-TOP-STRIP-LAYOUT` call sites is
    UNCHANGED.
  - Any other clickable-surface file under `client/src/ui/` or
    `client/src/presentation/` is UNCHANGED with respect to
    hover / focus / pressed / disabled visual states.
  Verification: `git diff origin/main...HEAD --stat -- 'client/src/ui/lobby.rs' 'client/src/ui/shop_auction/mod.rs' 'client/src/ui/hud/mod.rs' 'client/src/presentation/**'`
  shows no interaction-state literal changes. The expected
  modifications by `/dev-story` are confined to:
  `client/src/ui/design_tokens/interaction_states.rs` (NEW),
  `client/src/ui/design_tokens/mod.rs` (extended; module
  declaration only), `docs/ux/global-ui-design-spec.md` (amended;
  new spec section + Spec Adoption Matrix row),
  `tests/integration/ui_clean_pass/...` (NEW integration test),
  and `Cargo.toml` (only if a new `[[test]]` registration is
  required, mirroring the Sprint 14 Tier 0 integration-test
  pattern). Per-surface migration is the Sprint 16+ follow-on
  story.

- [x] **AC11 -- Friend-game scope preserved**: GIVEN the story
  commit, WHEN `QA-COND-0005`, `QA-COND-0006`, and `PAW-TD-*-a`
  accept-risk dispositions are inspected, THEN none of them has
  been flipped to `closed` by this story. The new module's `FOCUS_*`
  token set does NOT advance `QA-COND-0005` Standard-tier focus
  conformance; the new tints do NOT pursue WCAG contrast ratios;
  the new ≥44px hit-target band is NOT enforced (`QA-COND-0005`
  hit-target gap preserved at `L5 LOBBY_BUTTON_HEIGHT = 30.0`);
  the friend-game placeholder palette is NOT replaced
  (`PAW-TD-*-a` preserved). Verification: `git diff` of
  `production/sprint-status.yaml` shows no accept-risk disposition
  change.

- [x] **AC12 -- No release / playtest / final-art / two-client
  GAME_OVER claims**: GIVEN the story commit, WHEN the closure
  paperwork is inspected, THEN this row does NOT claim public
  release readiness, release-candidate readiness, full game
  completion, full playable-client manual QA, playtest /
  fun-hypothesis validation, final-art / asset-production
  completion, two-client GAME_OVER closure (`S8-QA-001-W1`), the
  `Polish->Release` gate-check retry, stage advance from `Polish`
  to `Release`, the underlying drag-runtime bug fix from Sprint 12
  story 019, `TQ-S12-C7` closure, or closure of any other Sprint 14
  / 13 / 12 / 11 / 10 row. Verification: paperwork review of the
  `/story-done` close-out section.

---

## Evidence Path

`production/qa/evidence/sprint-15-ui-interaction-state-primitives/`
(NEW; future `/dev-story` worker authors).

Expected artifacts:

- Module-shape integration test pass log (AC8) -- raw `cargo test
  --test ui_clean_pass_interaction_state_primitives` (or
  equivalent) output.
- Doc-review checklist enumerating each AC1..AC12 against the
  authored module + spec amendment.
- Spec heading scan output (`grep "^## §"
  docs/ux/global-ui-design-spec.md`) confirming AC7 new-section
  presence.
- Spec Adoption Matrix diff showing the row for
  `S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001` updated with the new
  spec section reference.
- `git diff` showing the per-surface call sites (AC10) UNCHANGED.

---

## Likely Files Touched

| Path | Expected change |
|------|-----------------|
| `client/src/ui/design_tokens/interaction_states.rs` (NEW) | Author four named interaction-state token sets (`HOVER_*`, `FOCUS_*`, `PRESSED_*`, `DISABLED_*`) with doc comments and numeric defaults. |
| `client/src/ui/design_tokens/mod.rs` | Declare `pub mod interaction_states;` alongside the existing Sprint 14 Tier 0 modules (z_layers / typography / spacing / strips / overlays). |
| `docs/ux/global-ui-design-spec.md` | Amend: new spec section enumerating the four token sets + canonical defaults; flip the §10 button-affordance deferral notes to a forward reference; update the Spec Adoption Matrix row for `S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001`. |
| `tests/integration/ui_clean_pass/interaction_state_primitives_test.rs` (NEW; suggested filename) | Integration test asserting primitive module shape (AC8). |
| `Cargo.toml` (project workspace; possibly only the relevant member crate) | Register the new `[[test]]` bin if the project pattern requires explicit registration, mirroring the Sprint 14 Tier 0 integration-test bins. |

This table is a planning estimate. The implementation prompt is
authoritative for the realised set.

**Explicitly NOT touched** by this story file (the authoring run is
paperwork only) and **explicitly NOT touched by Sprint 15 `/dev-story`
for this row**:

- `client/src/ui/lobby.rs` (per AC10 -- per-surface migration deferred).
- `client/src/ui/shop_auction/mod.rs` button call sites at
  `S11-UX-AUCTION-FEATURED-CARD` (per AC10).
- `client/src/ui/hud/mod.rs` button / top-strip call sites at
  `S11-UX-HUD-TOP-STRIP-LAYOUT` (per AC10).
- Any other clickable-surface file under `client/src/ui/` or
  `client/src/presentation/` with respect to hover / focus / pressed /
  disabled visual states (per AC10).
- `server/src/**`, `shared/src/**`.
- `tests/**` outside the new
  `tests/integration/ui_clean_pass/interaction_state_primitives_test.rs`
  bin.
- `production/sprint-status.yaml` (only touched by activation /
  `/story-done` paperwork, not by `/dev-story`).
- `production/sprints/*` (Sprint 15 plan / Sprint 14 plan / earlier
  plans not modified).
- `production/stage.txt` (`Polish`; not advanced).
- `production/session-state/**`.
- `production/qa/*` outside the dedicated evidence directory
  `production/qa/evidence/sprint-15-ui-interaction-state-primitives/`
  (NEW) authored by the future `/dev-story` worker.

---

## Verification

- File presence check on
  `client/src/ui/design_tokens/interaction_states.rs`.
- Module declaration check on
  `client/src/ui/design_tokens/mod.rs`.
- `cargo test --test ui_clean_pass_interaction_state_primitives` (or
  the project-idiomatic bin name) passes -- AC8.
- `git diff origin/main...HEAD -- 'client/src/ui/lobby.rs' 'client/src/ui/shop_auction/mod.rs' 'client/src/ui/hud/mod.rs' 'client/src/presentation/**'`
  shows no interaction-state literal migration -- AC10.
- Spec heading scan
  (`grep "^## §"  docs/ux/global-ui-design-spec.md` or
  project-idiomatic equivalent) shows the new interaction-state
  section -- AC7.
- Spec Adoption Matrix diff shows the row for
  `S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001` updated.
- `git diff` of `production/sprint-status.yaml` empty across the
  worker tip and integration merge -- AC11.

---

## Dependencies / Sequencing

- **Authoring prompt (this PROMPT 993)** is the *story-authoring* prompt;
  it creates the story file only. No `/dev-story` runs here.
- **Activation**: Requires Sprint 15 activation (separate prompt; not
  this one). Cannot land in Sprint 14 (closed-with-conditions).
- **Tier 0 host-module sequencing** (per `docs/ux/ui-clean-pass-roadmap.md`
  §3 "Sequencing Rules"): the host module
  `client/src/ui/design_tokens/` is already populated by the Sprint 14
  DONE Tier 0 stories 002 (z_layers) / 003 (typography) / 004 (spacing
  + strips) / 006 (overlays). This row is **additive** to the host
  module: it adds a new sibling submodule `interaction_states.rs`
  alongside the five existing ones. No file collision is expected.
- **Spec dependency** (per
  `docs/ux/ui-clean-pass-roadmap.md` Tier 0 Should-priority adjacent
  row note "Pair with rank 6 (design spec)"): the canonical global UI
  design spec at `docs/ux/global-ui-design-spec.md` is **DONE on Sprint
  14 PROMPT 922**. The spec already defers interaction-state primitives
  to this row at §"Scope Boundaries" line ~95-97 and §10 button-
  affordance lines ~442-443, and names this row in the Spec Adoption
  Matrix at line ~518. The spec amendment this row authors is therefore
  a forward-reference flip rather than a back-fill.
- **No producer-decision blocker.** PROMPT 802 §9 producer-decisions 1
  through 4 are all RESOLVED on Sprint 14 (decisions 2 / 3 / 4 per
  PROMPT 911 + 922 + 933 + 935 + 967 per Sprint 15 plan draft §9 G6
  Sequencing Notes). Producer-decision 5 (cosmetic captures bundle) is
  NOT relevant to this row. The Sprint 15 plan draft (PROMPT 988)
  notes "PROMPT 802 §9 producer-decision candidate (visual language
  for interaction states) MAY apply -- producer to confirm at
  story-authoring time"; this authoring run confirms that the
  worker-proposed defaults above (hover ~0.08 tint, pressed ~0.16
  tint, disabled ~0.50 tint, focus ring `ACCENT` + 2 px) are
  worker-discretion within the documented bands and ratified by the
  spec amendment this row authors. No separate producer-decision
  artifact is required.
- **File-disjoint with the other Sprint 15 candidates**: this row
  touches the design-token module + spec body; the Sprint 15 Should
  row `S12-UX-HAND-DRAG-STATE-VISUALS-001` touches the hand-UI
  surface only; the Sprint 15 Should row `S11-UX-BOARD-RENDERING-SPEC`
  is doc-only on `docs/ux/board-rendering-spec.md` (NEW). All three
  Sprint 15 implementation rows are pairwise file-disjoint per the
  Sprint 15 plan draft "Suggested First Parallel Batch" section.
- **Future Tier 1 / Tier 2 / Tier 3 surface stories may consume the
  primitive module**: Sprint 16+ per-surface migration stories
  (`S16-UI-INTERACTION-STATE-MIGRATION-*` family; not yet authored)
  will migrate lobby / auction / HUD / shop / draft button call sites
  from per-site styling to the token reads. This row is the
  prerequisite for that follow-on family.

---

## Notes

- PROMPT 802 §3.9 G7: no canonical interaction-state primitive set.
- `docs/ux/ui-clean-pass-roadmap.md` Tier 0 Should-priority adjacent
  row: "Net-new / PROMPT 802 §3.9 G7. Pair with rank 6 (design spec);
  ranks 7-12 are tolerable without it but degrade to per-site button
  styling."
- `docs/ux/global-ui-design-spec.md` §"Scope Boundaries" line ~95-97 +
  §10 button-affordance lines ~442-443 + Spec Adoption Matrix line
  ~518: this row is the named owner of hover / focus / pressed /
  disabled state in the spec architecture.
- The default proposed alpha / pixel values (hover ~0.08 tint, pressed
  ~0.16 tint, disabled ~0.50 tint, focus ring ~2 px width / ~2 px
  offset on `ACCENT` palette) come from PROMPT 993's task brief
  embedded in the Sprint 15 plan draft (PROMPT 988) plus PROMPT 802's
  §3.9 G7 surface enumeration. The exact final values are ratified
  by the spec amendment that this row authors.
- Per Sprint 15 plan draft (PROMPT 988): this row is Nice to Have;
  lands only if Must Have + Should Have closure is on track.
- Per Sprint 15 plan draft (PROMPT 988): per-surface migration of
  existing Sprint 14 button surfaces is OUT OF SCOPE for Sprint 15
  ("Migration of existing Sprint 14 button surfaces ... out of scope
  for Sprint 15 -- the primitive module is authored and the spec
  body in `docs/ux/global-ui-design-spec.md` is amended to reference
  it, but per-surface migration is a follow-on story").
- Accept-risk preservation: `PAW-TD-*-a`, `QA-COND-0005`, `QA-COND-0006`
  preserved unchanged. This story does not advance any of them.
- The `FOCUS_*` token set is a *visual* primitive only. Standard-tier
  focus-order, keyboard navigation, screen-reader hints, and
  ≥44px hit-target enforcement remain accepted-risk under
  `QA-COND-0005`. Authoring the focus-ring visual does NOT
  short-circuit the Standard-tier accessibility scope.

---

## Closure Trail

| PROMPT | Action | Commit / Reference |
|--------|--------|---------------------|
| 993 | Authored story file (Sprint 15 candidate; NOT activated) on branch `story-authoring/sprint-15-ui-interaction-state-primitives` from base `origin/main@2c84d6e37f2ec58b729064b6dbe4c9b017e5ceb3` (PROMPT 990 Sprint 15 plan-draft integration merge). EPIC index updated to include this row as story 008. | `production/epics/ui-clean-pass/story-008-ui-interaction-state-primitives.md` NEW + `production/epics/ui-clean-pass/EPIC.md` MODIFIED |

Subsequent prompts (integration, `/story-readiness` rerun against
Sprint 15 activation HEAD, Sprint 15 activation, `/qa-plan sprint-15`,
`/dev-story`, integration of the worker, `/story-done`) are TBD and
will be appended to this Closure Trail as they land. Per Sprint 15
plan draft (PROMPT 988), the expected sequencing is:

1. Story-authoring integration prompt -- merges this branch (and the
   sibling `story-authoring/sprint-15-hand-drag-state-visuals` and
   `story-authoring/sprint-15-board-rendering-spec` branches) into
   `origin/main` (one `--no-ff` per branch or one consolidated
   merge, mirroring PROMPT 893).
2. Sprint 15 activation prompt -- flips `production/sprint-status.yaml`
   top-level and appends the `sprint_15_activation:` block.
3. `/qa-plan sprint-15` -- authors the Sprint 15 QA plan.
4. `/story-readiness` rerun against Sprint 15 activation HEAD -- this
   row.
5. `/dev-story` for this row -- worker authors the module + spec
   amendment + integration test on a worker branch
   (`work/s15-ui-interaction-state-primitives`).
6. Sprint 15 integration prompt -- merges the worker tip into
   `origin/main`.
7. `/story-done` for this row -- paperwork-only closure flip
   `ready -> done` + AC1..AC12 verdict capture + Closure Trail
   append.

---

## Completion Notes

**Completed**: 2026-05-17 by PROMPT 1009 `/story-done` serialized
paperwork closure (Sprint 15 integrated story-done batch).

**Criteria**: 12 / 12 accepted. AC1-AC12 PASS via the
PROMPT 1005 `/dev-story` worker and PROMPT 1007 integration
verification. The primitive module
`client/src/ui/design_tokens/interaction_states.rs` (NEW; 445 lines
post-integration) exports four named token-set families
(`HOVER_*`, `FOCUS_*`, `PRESSED_*`, `DISABLED_*`); the global UI
design spec `docs/ux/global-ui-design-spec.md` was amended in §2 /
§10 / NEW §11 / Spec Adoption Matrix / Ratification scope guard;
the integration test bin
`tests/integration/ui_clean_pass/interaction_state_primitives_test.rs`
ships 8 ECS-query tests covering AC1-AC9 module-shape assertions;
inline lib unit tests under
`design_tokens::interaction_states::tests` cover the same families
with 10 alpha-range / pixel-range / ordering / doc-comment /
pairwise-distinctness assertions.

**Deviations**: Per-surface migration of existing Sprint 14 button
surfaces is explicitly OUT OF SCOPE for Sprint 15 (AC10) and
deferred to Sprint 16+ family `S16-UI-INTERACTION-STATE-MIGRATION-*`.
No existing button call sites under `client/src/ui/lobby.rs`,
`client/src/ui/shop_auction/`, `client/src/ui/hud/`, or
`client/src/presentation/` are touched -- PROMPT 1007 AC10 disjoint-
surface verification PASS (empty diff).

**Test Evidence**: PROMPT 1007 integration runs reported `cargo
check -p client` PASS (12.87s), `cargo fmt -p client -- --check`
PASS (no diff), targeted primitive test bin
`ui_clean_pass_interaction_state_primitives_test` PASS 8/8, inline
lib unit tests for `design_tokens::interaction_states` PASS 10/10,
`git diff --check origin/main..HEAD` PASS (no whitespace errors),
`git diff --cached --check` PASS. Cargo resource policy applied
(`CARGO_TARGET_DIR=D:/_DEV/cargo-target/ccgs-msvc` +
`CARGO_PROFILE_DEV_DEBUG=0` + `CARGO_PROFILE_TEST_DEBUG=0` +
`CARGO_INCREMENTAL=0` + `RUSTFLAGS=-C debuginfo=0 -C
link-arg=/DEBUG:NONE`); PROMPT 1009 itself did NOT re-run cargo
(paperwork-only closure).

**Code Review**: PROMPT 1009 verified integration commit
`5d36c4b4fe061981b6c1949f3f4f7986ca0cf0cb` is reachable from
`origin/main` (current tip `88a6db16e8abec6b2e7df1f8efac0fc933b5c0b3`
via PROMPT 1008), reviewed PROMPT 1005 worker report and PROMPT 1007
integration report for AC coverage, and performed paperwork-only
closure. No `client/`, `server/`, `shared/`, `tests/`, Cargo,
`production/sprints/sprint-15.md`, `production/qa/qa-plan-sprint-15.md`,
`production/stage.txt`, or gate artifact was edited by PROMPT 1009.

## Closure Trail

- PROMPT 993 (2026-05-16) -- story authoring on branch
  `story-authoring/sprint-15-ui-interaction-state-primitives`,
  worker commit `0f113e1`. Integrated into `origin/main` by
  PROMPT 995 batch merge `8294f9a`.
- PROMPT 1005 (2026-05-17) -- `/dev-story` implementation on branch
  `work/s15-ui-interaction-state-primitives`, worker commit
  `ea26e34cedbbed61cbf377751b386c18b55f8fcd`. Authored
  `client/src/ui/design_tokens/interaction_states.rs` (NEW), modified
  `client/src/ui/design_tokens/mod.rs` to register the submodule,
  amended `docs/ux/global-ui-design-spec.md` per AC7, added
  `tests/integration/ui_clean_pass/interaction_state_primitives_test.rs`
  (NEW; 8 tests), registered the integration test bin in
  `client/Cargo.toml`, and wrote
  `production/qa/evidence/sprint-15-ui-interaction-state-primitives/evidence.md`.
- PROMPT 1007 (2026-05-17) -- integration merge
  `5d36c4b4fe061981b6c1949f3f4f7986ca0cf0cb` onto `origin/main`.
  No-ff merge from `origin/main@08f389b` (PROMPT 1006 board-rendering
  tip; first integration worktree was abandoned after origin/main
  drift and a v2 worktree was created from the new tip per the
  concurrency clause); 6 files / 1048 insertions / 6 deletions;
  AC10 disjoint-surface verification PASS (no lobby / shop / hud /
  presentation file touched); cargo verification PASS; diff checks
  PASS.
- PROMPT 1009 (2026-05-17) -- serialized `/story-done` paperwork
  closure within the Sprint 15 integrated story-done batch. Story
  status marked Done, Sprint 15 row flipped `ready -> done` with
  completed date 2026-05-17, AC1-AC12 checkboxes marked complete,
  session-state banners prepended, and `sprint_15_story_done` block
  appended at EOF of `production/sprint-status.yaml`. Sprint 15
  remains active; stage remains Polish; PROMPT 761 Polish->Release
  FAIL, `S8-QA-001-W1` OPEN, `QA-COND-0005/0006` accepted-risk,
  `PAW-TD-*-a` accepted-risk, `S11-HUD-TIMER-EYEBALL-VISUAL-001`
  human-operator-blocked carry, and
  `S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001-ROWFLIP` open status
  all preserved. Per-surface migration remains deferred to Sprint
  16+ family `S16-UI-INTERACTION-STATE-MIGRATION-*`.
