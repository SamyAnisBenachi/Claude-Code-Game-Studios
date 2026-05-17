# Story 013: S16-TD-UI-BUTTON-PRIMITIVE-001 -- UI Button Primitive

> **Epic**: UI Clean-Pass
> **Story ID**: S16-TD-UI-BUTTON-PRIMITIVE-001
> **Status**: Draft -- Sprint 16/17 candidate, NOT activated
> **Layer**: Presentation / UX foundational tech-debt (shared primitive)
> **Type**: Tech Debt -- foundational primitive (shared widget)
> **Sprint**: Sprint 16/17 Phase B candidate per PROMPT 1035 §"Suggested
> refactor sequence" + PROMPT 1034 §3 D3 ("No reusable Button primitive").
> Per-surface migration of remaining button sites is the Phase C family
> `S16-UI-INTERACTION-STATE-MIGRATION-*` already named by story 008
> close-out.
> **Authored**: 2026-05-17 by PROMPT 1044
> **Authoring source-of-truth**: `origin/main@a7a8b079` (PROMPT 1041).
> **Estimated effort**: ~1.0d (primitive module + spec amendment +
> single canonical migration + interaction-state composition test)

---

## Status / No-Claim Banner

This story is authored as a Sprint 16/17 candidate. **No sprint is
activated by this authoring run.** PROMPT 1044 does NOT activate any
sprint, modify sprint-status / sprint plan / stage / session-state,
run any `/dev-story` / `/story-done` / `/smoke-check` / `/team-qa` /
`/gate-check` / `/release-check` / `/qa-plan` workflow, modify code
under `client/` / `server/` / `shared/` / `tests/`, or author the
future `client/src/ui/design_tokens/button.rs` module.

This story does **not** claim: public release readiness, full game
completion, broad / Standard-tier accessibility completion
(`QA-COND-0005`), Standard-tier hit-target conformance (≥44px),
playtest validation (`QA-COND-0006`), full playable-client manual QA,
two-client GAME_OVER closure (`S8-QA-001-W1`), final-art completion
(`PAW-TD-*-a`), `Polish->Release` retry, or stage advance.

---

## Overview

PROMPT 1034 §3 D3 ("No reusable Button primitive") catalogues five
button surfaces in the playable client and finds **none of them share
a base contract**:

- DraftInitial `Dismiss` and `Ready`: tinted text fields rather than
  bordered buttons (PROMPT 1034 §2.2 + evidence `000000-…30856`).
- Placement `Submit` + `(0`: rendered as two separate text lines, no
  background, no border, no hover/disabled state, budget readout
  truncated mid-string (PROMPT 1034 F-7 / evidence `000003-…45488`).
- Auction bid buttons: bare `?` glyphs in frames -- the user cannot
  tell what bid each one places (PROMPT 1034 F-4 / evidence
  `000013-…92701`).
- Shop refresh / ready: per-site button styling with magic-offset
  anchors (PROMPT 1035 §"Shop / Auction Not migrated / debt" — six
  absolute anchors, no flex parent).
- Lobby buttons: single `lobby_button_node` helper but bare RGB
  literals + 30 px height ≠ `QA-COND-0005` Standard-tier ≥44px
  (PROMPT 1035 §"Lobby Not migrated / debt").

The root cause is two missing layers:

1. A **button-chrome primitive** that publishes background colour,
   border colour, border thickness, border radius, padding, and text
   colour as named tokens consumable by every button site.
2. A **state-mapping primitive** that consumes story 008's
   `interaction_states` tokens (Sprint 15 DONE:
   `HOVER_BG_TINT_ALPHA`, `HOVER_BORDER_ALPHA`,
   `PRESSED_BG_TINT_ALPHA`, `PRESSED_TEXT_DARKEN`,
   `FOCUS_RING_COLOR`, `FOCUS_RING_OFFSET_PX`, `FOCUS_RING_THICKNESS_PX`,
   `DISABLED_BG_TINT_ALPHA`, `DISABLED_BORDER_ALPHA`,
   `DISABLED_TEXT_ALPHA`) and applies them per button state.

Story 008 (`S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001` DONE Sprint
15 PROMPT 1009) published the state tokens but **did not publish a
button widget that consumes them**. Per story 008 §"Out of Scope"
line 95-103, "Per-surface migration of existing Sprint 14 button
surfaces (lobby buttons, auction bid buttons, HUD action buttons,
draft buttons, shop slot buttons) is **explicitly OUT OF SCOPE** for
Sprint 15. The primitive module is authored and the spec body is
amended, but the existing button call sites are NOT migrated by this
row. Per-surface migration is a follow-on story (slug TBD; expected
`S16-UI-INTERACTION-STATE-MIGRATION-*` family)."

This story authors the future `/dev-story` worker's contract for a
**`Button` primitive** that publishes the button chrome + composes
story 008's interaction-state tokens into per-state visuals,
consumed by at least ONE canonical migration site (the Placement
`Submit` button -- the highest-impact P1 fix per PROMPT 1034 A8
"Placement action panel"). Per-surface migration of the remaining
four button families is the Phase C.8 + Phase C-mid follow-on family
`S16-UI-INTERACTION-STATE-MIGRATION-*`.

Per PROMPT 1035, this primitive authoring is **file-disjoint** from
stories 010 / 011 (the module splits). The canonical migration target
(Placement `Submit`) lives under `client/src/presentation/placement*`
or `client/src/ui/hand/submit.rs` (post story 011 split); the producer
chooses based on which split has landed.

---

## Scope

### In Scope

#### Primitive module

- A new primitive module at
  `client/src/ui/design_tokens/button.rs` (NEW; future `/dev-story`
  output). Worker MAY choose `client/src/ui/primitives/button.rs`
  with justification. The module exports:

  - **`ButtonKind` enum** with at least these variants:
    - `Primary` -- the canonical primary CTA (`Ready`, `Submit`,
      `Confirm`, `Place Bid`).
    - `Secondary` -- the canonical secondary action (`Dismiss`,
      `Cancel`, `Skip`).
    - `Bid` -- the auction bid-button kind (fixed-amount bid + Pass
      variants per PROMPT 1034 §2.5).
    - `Icon` -- an icon-only button (e.g. Help / Settings rail) --
      OPTIONAL; worker may defer if no canonical migration site
      consumes it.

  - **`ButtonState` enum**: `Default`, `Hover`, `Pressed`, `Disabled`,
    `Focused`. (`Focused` combines with the other states; the
    integration test asserts that focus is composable with hover /
    pressed / disabled.)

  - **`Button` Bundle / Component cluster** that composes:
    - **Backplate**: `Node` with `width`, `height` (per `ButtonKind`
      default; surface MAY override), `border` thickness, `border_radius`,
      `padding`, `BackgroundColor`, `BorderColor` (per
      `ButtonKind::Default` chrome).
    - **Label**: child `Text` with default text colour per
      `ButtonKind`, font from `typography::*` (story 003 DONE; default
      `typography::H3` for `Primary` / `Secondary`,
      `typography::CAPTION` for `Bid` numeric label).
    - **Focus-ring**: optional outset `Node` consumed only when
      `ButtonState::Focused` (per `interaction_states::FOCUS_RING_*`
      tokens). Worker MAY implement as a sibling Node toggled by
      `Visibility`.

  - **`button_node(kind: ButtonKind) -> Node`** that returns the
    backplate Node configured per kind. Convenience builders
    `button_label_text_font(kind)`, `button_focus_ring_node(kind)`.

  - **`apply_button_state(kind: ButtonKind, state: ButtonState) ->
    (BackgroundColor, BorderColor, TextColor)`** that returns the
    composed visual triple for the given (kind, state) pair, reading
    from `interaction_states::HOVER_*` / `PRESSED_*` / `FOCUS_*` /
    `DISABLED_*` tokens per story 008.

  - **Per-kind dimension constants** (`PRIMARY_BUTTON_HEIGHT_PX`,
    `PRIMARY_BUTTON_MIN_WIDTH_PX`, `BID_BUTTON_WIDTH_PX`,
    `BID_BUTTON_HEIGHT_PX`, `ICON_BUTTON_SIZE_PX`, etc.). These are
    pixel-fixed per spec §10 button affordance.

  - **Friend-game accept-risk doc-comment**: heights below
    `QA-COND-0005` ≥44px floor (e.g. existing
    `LOBBY_BUTTON_HEIGHT_PX = 30.0`) MUST carry a `///` doc comment
    citing `QA-COND-0005` accept-risk. Raising button heights to
    Standard-tier ≥44px is NOT done by this row.

#### Global UI design spec amendment

- An amendment to `docs/ux/global-ui-design-spec.md` adding a new
  spec section (suggested §14 "Button Primitive", after §13 Modal
  primitive if story 012 lands first; otherwise §13). The section:
  - Names the `ButtonKind` variants and per-kind dimensions / chrome.
  - Cross-references `interaction_states::*` tokens (story 008 DONE).
  - Forbids the "text-only button" anti-pattern (PROMPT 1034 D3).
  - Forbids bare `?` glyphs as button content (PROMPT 1034 D5
    placeholder-icon policy is partially mitigated by requiring
    `Bid` buttons to render their amount, not `?`).
  - Preserves the friend-game vs Standard-tier scope boundary
    verbatim: the button primitive is a *layout* + *visual-state*
    contract only; ≥44px enforcement, WCAG contrast, full keyboard
    navigation, screen-reader role announcements are NOT introduced.

#### Phase 1 canonical migration -- Placement `Submit` button

- A first canonical migration of one existing button site to the
  primitive: the **Placement `Submit` button** (PROMPT 1034 A8 P1
  fix; PROMPT 1034 §2.3 "professional expectation": real Submit
  button styled as primary CTA when ready).
  - The site lives under `client/src/ui/hand/submit.rs` (post story
    011 split) OR `client/src/presentation/placement*`; the producer
    chooses at activation based on the current source-tree state.
  - Replace the inline text-only `Submit` + `(0` composition with
    `button_node(ButtonKind::Primary)` + `Text` label "Submit (N / 3
    placed)" (or whichever copy the GDD specifies; the canonical
    string "Submit (0 / 3 selected)" / "Submit (2 / 3 placed)"
    appears in PROMPT 1034 §2.3 professional expectation).
  - Wire the four `ButtonState` transitions (`Default` / `Hover` /
    `Pressed` / `Disabled`) via `apply_button_state(ButtonKind::Primary,
    state)`. `Disabled` applies when the player has not selected a
    valid card / lane (existing game-state check is preserved
    verbatim).

#### Tests

- A new integration test bin at
  `tests/integration/ui_clean_pass/button_primitive_test.rs` (NEW)
  asserting:
  - **AC2 -- Backplate present**: For every `ButtonKind`,
    `button_node(kind)` returns a Node with non-zero `border`
    thickness and non-default `BackgroundColor`. (No text-only
    buttons.)
  - **AC3 -- State composition**: For every (kind, state) pair,
    `apply_button_state(kind, state)` returns visual values that
    reference `interaction_states::*` tokens (assert that the
    returned `BackgroundColor` differs between `Default` and
    `Hover`; differs between `Default` and `Pressed`; differs
    between `Default` and `Disabled`).
  - **AC4 -- Focus composable**: `apply_button_state(kind,
    ButtonState::Focused)` returns a `BorderColor` or focus-ring
    Node that consumes `interaction_states::FOCUS_RING_COLOR` /
    `FOCUS_RING_OFFSET_PX` / `FOCUS_RING_THICKNESS_PX`.
  - **AC6 -- No bare-glyph button label**: Migrated Placement
    `Submit` button label is the canonical string (assert the
    `Text` child contains "Submit" and contains a digit). For
    `ButtonKind::Bid` variants, assert the test fixture's
    `button_label_text_font(Bid)` returns a numeric-formatting
    helper, NOT a `?` glyph default.
  - **AC7 -- Token re-use**: No new `Color::srgb*(...)` literal is
    introduced in `button.rs` that duplicates an existing
    `interaction_states::*` or §7 palette token. Worker may rely on
    `interaction_states::*` re-exports; net-new RGB triples are
    forbidden.

- A QA snapshot bundle at 1280 × 720 and 1920 × 1080 in Placement
  showing:
  - Migrated `Submit` button has a visible backplate, border, and
    label.
  - Disabled state visually distinct from default state.
  - Hover state visually distinct from default state.

### Out of Scope

- **Per-surface migration of all four other button families.**
  DraftInitial `Dismiss` / `Ready`, auction bid buttons, shop
  refresh / ready / hand-full, lobby buttons are owned by the
  follow-on family `S16-UI-INTERACTION-STATE-MIGRATION-*` per story
  008 close-out.
- **Auction bid-amount labeling.** PROMPT 1034 A6 ("Replace `?`
  auction bid buttons with real bid amounts + Pass") is a separate
  P1 surface fix that USES this primitive after this story lands;
  not authored here.
- **Standard-tier ≥44px hit-targets.** Heights below 44 px (e.g.
  lobby button) preserve `QA-COND-0005` accept-risk; raising heights
  is a separate accessibility-track row.
- **Icon button family** (Help / Settings rail glyphs). The `Icon`
  variant is OPTIONAL per AC1; if the worker does not author it, the
  Phase C row that migrates the rail icons authors the variant.
- **Focus order / keyboard navigation.** ARIA roles, focus-restore
  on disable, programmatic focus management are OUT OF SCOPE.
- **No new color tokens** beyond `interaction_states::*` re-export
  consumption. Phase B.1 colours story owns net-new RGB triples.
- **No `Polish->Release` retry, no stage advance, no closed-row
  reopen.**

---

## Acceptance Criteria

All BLOCKING.

- [ ] **AC1 -- Primitive module exports**: `client/src/ui/design_tokens/button.rs`
  exists, declared from aggregator, exports `ButtonKind` (≥
  `Primary`, `Secondary`, `Bid`), `ButtonState` (5 variants),
  `button_node`, `apply_button_state`, per-kind dimension constants,
  with `///` doc comments naming consumer surfaces. Friend-game
  accept-risk doc-comment cites `QA-COND-0005` for heights below
  44 px.

- [ ] **AC2 -- Backplate present (no text-only buttons)**: Every
  `button_node(kind)` returns a Node with non-zero `border`
  thickness and non-default `BackgroundColor`. PROMPT 1034 D3 text-only
  anti-pattern is forbidden for all migrated sites.

- [ ] **AC3 -- State composition via story 008 tokens**: For every
  (kind, state) pair, `apply_button_state` returns a visual triple
  that consumes `interaction_states::HOVER_*` / `PRESSED_*` /
  `FOCUS_*` / `DISABLED_*` tokens. Assert per-state distinctness:
  `Default ≠ Hover`, `Default ≠ Pressed`, `Default ≠ Disabled`.

- [ ] **AC4 -- Focus composable**: `ButtonState::Focused` applies the
  `interaction_states::FOCUS_RING_*` set as an outset ring (sibling
  Node or border treatment) that composes with the other four states.

- [ ] **AC5 -- Phase 1 migration of Placement `Submit`**: Exactly ONE
  existing button site is migrated. Default: Placement `Submit`
  button at the Placement action-panel call site. DraftInitial
  `Dismiss` / `Ready`, auction bid buttons, shop refresh / ready /
  hand-full banner, lobby buttons are EXPLICITLY UNCHANGED.
  Verification: `git diff origin/main...HEAD --stat -- 'client/src/'`.

- [ ] **AC6 -- No bare-glyph button label**: Migrated `Submit`
  button label contains "Submit" and a digit (not `?`); for
  `ButtonKind::Bid`, the helper API exposes a numeric formatter
  not a `?` glyph default.

- [ ] **AC7 -- Token re-use, no net-new RGB**: No `Color::srgb*(...)`
  literal in `button.rs` duplicates an existing
  `interaction_states::*` or §7 palette token (sweep this when Phase
  B.1 colours lands).

- [ ] **AC8 -- Visual evidence**: Evidence directory at
  `production/qa/evidence/sprint-1X-button-primitive/` contains QA
  snapshot bundles at 1280 × 720 and 1920 × 1080 showing the
  migrated `Submit` button with visible backplate + state diff
  (default vs disabled, default vs hover), doc-review checklist,
  integration-test pass log, `git diff --stat` proving single-site
  migration.

- [ ] **AC9 -- Non-claims**: No gameplay / server / shared / protocol
  change. No release / full-game / Standard-tier / playtest /
  final-art / two-client closure claim. No closed-row reopen. No
  focus-trap / ARIA / ≥44px hit-target conformance.

- [ ] **AC10 -- Forbidden literal guards**: Migrated button site has
  no bare `Color::srgb(0.98, 0.73, 0.30)` or similar ad-hoc
  per-state literal (Phase B.1 colours sweep is a separate row;
  this AC asserts the primitive consumes story 008 tokens
  correctly).

---

## Implementation Notes

### Owned files

| Path | Expected change |
|------|-----------------|
| `client/src/ui/design_tokens/button.rs` (NEW) | Author `ButtonKind` + `ButtonState` + `Button` Bundle + `button_node` + `apply_button_state` + dimension constants. |
| `client/src/ui/design_tokens/mod.rs` | Declare `pub mod button;`. |
| Placement `Submit` button site (`client/src/ui/hand/submit.rs` post story 011 OR `client/src/presentation/placement*`) | Migrate ONE canonical site. |
| `docs/ux/global-ui-design-spec.md` | Amend with new §14 "Button Primitive". |
| `tests/integration/ui_clean_pass/button_primitive_test.rs` (NEW) | Integration test per AC2-AC4 + AC6-AC7 + AC10. |
| `client/Cargo.toml` | Register `[[test]]` bin if pattern requires. |
| `production/qa/evidence/sprint-1X-button-primitive/` (NEW) | Evidence dir per AC8. |

### Forbidden files

- `client/src/ui/lobby.rs` -- migration is separate
  `S16-UI-INTERACTION-STATE-MIGRATION-LOBBY-001`.
- `client/src/ui/shop_auction/auction.rs` (post story 010) -- bid
  buttons are separate Phase C.8 `S16-UI-INTERACTION-STATE-MIGRATION-BID-001`.
- `client/src/ui/shop_auction/shop.rs` (post story 010) -- shop
  controls are separate Phase C.5 `S16-UI-SHOP-CONTROL-ROW-001` +
  follow-on interaction-state migration.
- `client/src/ui/shop_auction/draft_initial.rs` (post story 010) --
  Dismiss / Ready migration is separate
  `S16-UI-INTERACTION-STATE-MIGRATION-DRAFT-001`.
- `client/src/ui/hud/**` -- HUD action buttons are separate.
- `client/src/ui/design_tokens/{z_layers,typography,spacing,strips,overlays,interaction_states,card_slot,modal,panel}.rs`
  -- read-only inputs.
- `server/src/**`, `shared/src/**` -- UNCHANGED.
- `production/sprint-status.yaml`, `production/sprints/*`,
  `production/stage.txt`, `production/session-state/*` --
  shared-state writers.

---

## Parallelization and Dependencies

| Sibling story | Parallel-safe? |
|---|---|
| **Story 008 interaction-state primitives (DONE Sprint 15)** | **Hard prerequisite.** Story 008 published the state tokens this primitive consumes. Already DONE on `origin/main`. |
| **Story 010 shop_auction modsplit** | Primitive authoring: **YES**. Per-shop / per-auction button migration: blocked until story 010 lands. |
| **Story 011 hand modsplit** | Primitive authoring: **YES**. Placement `Submit` migration: producer picks site based on which split has landed. |
| **Story 012 modal primitive** | **YES**, file-disjoint. `Ready` / `Dismiss` in the modal footer (per story 012) are rendered with per-site styling until this story's per-modal button migration follow-on lands. |
| **Story 014 panel primitive** | **YES**, file-disjoint. |
| **Story 015 sequencing doc** | **YES**, doc-only. |

### Dependencies

- **Prerequisite**: `interaction_states::*` tokens (story 008 DONE).
- **Soft prerequisite**: story 011 hand modsplit, if Placement
  `Submit` lives under hand/submit.rs after split.
- **Unblocks**: `S16-UI-INTERACTION-STATE-MIGRATION-*` family
  (lobby / draft-initial / bid / shop-controls / HUD button
  migrations).

---

## Worker Contract (for `/dev-story`)

The future `/dev-story` worker MUST:

1. Run `git checkout -b work/s16-button-primitive` from `origin/main`.
2. Read `interaction_states.rs` and the current Placement Submit
   button spawn site.
3. Author the primitive module + spec amendment per AC1.
4. Migrate ONE canonical site per AC5.
5. Author the integration test bin per AC2-AC4 + AC6-AC7 + AC10.
6. Capture evidence per AC8.
7. Verify `cargo test -p client` across all bin families.
8. Push `work/s16-button-primitive`. Do NOT push `main`.

The worker MUST NOT:

- Migrate more than ONE button site.
- Add ARIA / focus-trap / ≥44px hit-target enforcement.
- Introduce net-new RGB triples that duplicate `interaction_states::*`
  or §7 palette tokens.
- Touch any forbidden file.
- Run `/story-done` / `/smoke-check` / `/team-qa` / `/gate-check` /
  `/release-check` / `/qa-plan`.
- Modify `production/sprint-status.yaml`,
  `production/sprints/sprint-XX.md`, `production/stage.txt`, or
  `production/session-state/*`.

---

`013: S16-TD-UI-BUTTON-PRIMITIVE-001: DRAFT`
