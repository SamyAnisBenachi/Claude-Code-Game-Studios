# Story 012: S16-TD-UI-MODAL-PRIMITIVE-001 -- UI Modal Primitive

> **Epic**: UI Clean-Pass
> **Story ID**: S16-TD-UI-MODAL-PRIMITIVE-001
> **Status**: Draft -- Sprint 16/17 candidate, NOT activated
> **Layer**: Presentation / UX foundational tech-debt (shared primitive)
> **Type**: Tech Debt -- foundational primitive (design token / shared widget)
> **Sprint**: Sprint 16/17 Phase B candidate per PROMPT 1035 §"Suggested
> refactor sequence" + PROMPT 1034 §3 D2 ("No reusable Modal primitive").
> Authoring + single canonical migration target a Sprint 16/17 slot;
> per-surface migration of the remaining modals is the Phase C.6 family
> (`S16-UI-MODAL-PANEL-CONSOLIDATION-001`).
> **Authored**: 2026-05-17 by PROMPT 1044
> **Authoring source-of-truth**: `origin/main@a7a8b079` (PROMPT 1041).
> **Estimated effort**: ~1.0d (primitive module + spec amendment +
> single canonical migration + viewport-invariant + scrim-opacity test)

---

## Status / No-Claim Banner

This story is authored as a Sprint 16/17 candidate. **No sprint is
activated by this authoring run.** PROMPT 1044 does NOT activate any
sprint, modify any sprint-status / sprint plan / stage / session-state
file, run any `/dev-story` / `/story-done` / `/smoke-check` /
`/team-qa` / `/gate-check` / `/release-check` / `/qa-plan` workflow,
modify any code under `client/` / `server/` / `shared/` / `tests/`,
or author the future `client/src/ui/design_tokens/modal.rs` (or
`client/src/ui/primitives/modal.rs`) module.

This story does **not** claim: public release readiness, full game
completion, broad / Standard-tier accessibility completion
(`QA-COND-0005`), Standard-tier hit-target conformance (≥44px),
playtest / fun-hypothesis validation (`QA-COND-0006`), full
playable-client manual QA, two-client GAME_OVER closure
(`S8-QA-001-W1`), final-art / asset-production completion
(`PAW-TD-*-a`), `Polish->Release` gate-check retry, or stage advance
from `Polish` to `Release`.

---

## Overview

PROMPT 1034 §3 D2 ("No reusable Modal primitive") catalogues four
modals in the playable client and finds **none of them share a base
contract**:

- DraftInitial keep-9 modal: no opaque scrim; photosensitivity warning
  text bleeds through the modal frame (PROMPT 1034 F-1 / F-5 /
  evidence snapshots `000000-…30856`, `000000-…34015`).
- Photosensitivity warning: z-fights with keep-9 (resolved separately
  by PROMPT 1026 / 1030 pre-session gating, but the underlying "two
  modals can co-exist with no z-stack policy" structural problem is
  unresolved).
- Auction featured-card modal: single yellow border with no
  header / body / footer structure (PROMPT 1034 §2.5 + evidence
  `000013-…92701`).
- "Dismiss" lives in the keep-9 modal header but `Ready` lives outside
  it (orphan footer action).

PROMPT 1035 §"Surface-by-surface architecture findings" adds the
result-screen modal, the connection-lost overlay, and the settlement
overlay -- all centred modals with three different panel background
RGB triples, three different border RGB triples, and three different
"text label" colours. Spec §7 ratifies one `SURFACE_ELEVATED` for
raised panel surfaces but **no `ModalRoot` builder consumes it**;
every modal site re-authors the scrim + panel composition.

The root cause is two missing primitives:

1. A **scrim layer** with mandatory opacity ≥ `OVERLAY_SCRIM_ALPHA` (=
   0.55 per `client/src/ui/design_tokens/overlays.rs`; PROMPT 924 /
   story 006 DONE) that blocks pointer events to anything beneath.
2. A **modal-panel layout** with mandatory `header / body / footer`
   slots so primary actions live in the footer (`Ready`,
   `Confirm`) and secondary actions live in the header (`Dismiss`,
   close-glyph) -- not as twin floating actions in unrelated regions.

This story authors the future `/dev-story` worker's contract for a
**`ModalRoot` primitive** that publishes a canonical scrim + centred
panel + header / body / footer slot layout, consumed by at least ONE
canonical migration site (the DraftInitial keep-9 modal -- the
highest-impact P1 fix per PROMPT 1034 A7). Per-surface migration of
the remaining four modals (photosensitivity, auction featured, result
screen, connection-lost, settlement) is a follow-on story family
`S16-UI-MODAL-PANEL-CONSOLIDATION-001` (per PROMPT 1035 §"Phase C.6").

Per PROMPT 1035 §"Parallelization map", this story's primitive
authoring is **file-disjoint** from story 010 (shop_auction split) and
story 011 (hand split). The canonical migration target (DraftInitial)
**depends on story 010** -- DraftInitial code moves to
`shop_auction/draft_initial.rs` after the split. The producer
schedules this story AFTER story 010 lands on `origin/main`, OR picks
a non-shop_auction canonical migration target (suggested fallback:
result-screen modal, which lives in `client/src/presentation/result_screen.rs`
and is not blocked by the shop_auction split).

---

## Scope

### In Scope

#### Primitive module

- A new primitive module at
  `client/src/ui/design_tokens/modal.rs` (NEW; future `/dev-story`
  output). The `/dev-story` worker MAY choose
  `client/src/ui/primitives/modal.rs` if a `primitives/` directory
  better fits the project's mental model (Sprint 14 Tier 0 used
  `design_tokens/`; the `/dev-story` worker MUST justify the choice
  in the story-done close-out). The module exports:

  - **`ModalKind` enum** with at least these variants (extensible):
    - `Standard` -- the canonical centred modal (DraftInitial,
      result-screen, lobby-style chrome).
    - `Narrow` -- a tighter centred modal (connection-lost-style;
      ≤ 520 px max-width per PROMPT 1035 §"Result screen /
      connection-lost / photosensitivity cluster").
    - `Featured` -- the auction featured-card modal (negative-margin
      anchoring replaced by canonical centred-panel pattern).

  - **`ModalRoot` Bundle / Component cluster** that composes:
    - **Scrim layer**: full-viewport `Node` with `position_type:
      Absolute`, `top / right / bottom / left: Val::Px(0.0)`,
      `BackgroundColor` consuming `overlays::OVERLAY_SCRIM_ALPHA`
      (= 0.55) over the canonical `SURFACE` colour (per spec §7;
      pending `colors.rs` from story 013-related Phase B.1). The
      scrim's `z_layer` reads `z_layers::MODAL` (per Sprint 14 story
      002 DONE).
    - **Panel layer**: centred child of the scrim with `align_self:
      Center / justify_self: Center` (canonical centring pattern per
      spec §8 / §10; **negative-margin trick is forbidden**), width
      and height per `ModalKind`. The panel `z_layer` reads
      `z_layers::MODAL + 1` (or whatever per-modal index policy is
      authored; see AC4).
    - **Header slot**: top child of the panel. Carries the modal
      title text + optional `Dismiss` action (close-glyph or
      secondary button).
    - **Body slot**: middle child of the panel. Carries the modal's
      surface-specific content (e.g. the keep-9 grid, the featured
      card body, the auction lot details).
    - **Footer slot**: bottom child of the panel. Carries the
      modal's primary action (`Ready`, `Confirm`, `Place Bid`) and
      optional secondary action. **`Ready` MUST live in the footer
      slot, NOT outside the panel.**

  - **`modal_root_node(kind: ModalKind) -> Node`** that returns the
    scrim Node configured per kind. Convenience builders
    `modal_panel_node(kind)`, `modal_header_slot_node()`,
    `modal_body_slot_node()`, `modal_footer_slot_node()` return the
    child Nodes.

  - **`MODAL_SCRIM_BACKGROUND: Color`** (or per-kind variants) reading
    the canonical scrim colour from existing tokens. The current
    `connection_lost_overlay.rs:214` 0.32-alpha exception per
    `overlays.rs` doc is preserved for the `Narrow` kind ONLY if the
    `/dev-story` worker chooses to keep it; otherwise the `Narrow`
    kind also uses `OVERLAY_SCRIM_ALPHA`. The choice is documented
    in the spec amendment.

  - **Per-modal z-stack policy**: `ModalRoot` carries a per-instance
    `ZIndex(modal_layer + index)` where `index` is incremented as
    each modal is spawned. The integration test asserts that two
    concurrently-spawned modals do NOT z-fight (no two modals share
    the same `ZIndex`).

#### Global UI design spec amendment

- An amendment to `docs/ux/global-ui-design-spec.md` (future
  `/dev-story` output) adding a new spec section (suggested location:
  §13 "Modal Primitive", after §12 "Card Slot Primitive" if story 009
  ratifies §12; otherwise §12). The section:
  - Names the `ModalKind` variants and per-kind dimensions / scrim
    alpha / z-layer.
  - Names the mandatory `header / body / footer` slot pattern.
  - Forbids the negative-margin centring trick.
  - Forbids `Ready` / `Confirm` / primary actions outside the footer
    slot.
  - Cross-references `overlays::OVERLAY_SCRIM_ALPHA` (story 006 DONE)
    and `z_layers::MODAL` (story 002 DONE).
  - Preserves the friend-game vs Standard-tier-accessibility scope
    boundary verbatim: the modal primitive is a *layout* primitive
    only; ≥44px hit-target enforcement, WCAG contrast on header /
    body text, screen-reader role announcements, and full
    keyboard-navigation focus order are NOT introduced. (Focus
    trapping inside the modal is OUT OF SCOPE here; if a future
    Standard-tier accessibility row pulls focus-trapping, the
    primitive must be amended at that time.)

#### Phase 1 canonical migration -- DraftInitial keep-9 modal

- A first canonical migration of one existing modal site to the new
  primitive: the **DraftInitial keep-9 modal**.
  - DraftInitial modal lives in
    `client/src/ui/shop_auction/draft_initial.rs` after story 010
    splits `shop_auction/mod.rs`; if story 010 has NOT landed at
    activation time, the producer either (a) schedules this story
    AFTER story 010 OR (b) picks a fallback canonical site
    (recommended fallback: `client/src/presentation/result_screen.rs`,
    which is not blocked by the split).
  - Replace the inline scrim + panel composition in the DraftInitial
    modal with `modal_root_node(ModalKind::Standard)` +
    `modal_panel_node` + `modal_header_slot_node` +
    `modal_body_slot_node` + `modal_footer_slot_node`.
  - **Move the `Ready` action from its current floating right-rail
    position into the modal footer slot** (PROMPT 1034 §2.2
    "professional expectation" #4). The `Dismiss` action stays in
    the header.
  - Verify the DraftInitial modal renders at the canonical viewport
    matrix per §8 (1280 × 720, 1366 × 768, 1920 × 1080, 2560 × 1440,
    plus the two sentinel viewports in
    `tests/integration/helpers/ui_viewport.rs::CANONICAL_VIEWPORTS`).

#### Tests

- A new integration test bin at
  `tests/integration/ui_clean_pass/modal_primitive_test.rs` (NEW)
  asserting:
  - **AC2 -- Mandatory scrim opacity**: `ModalRoot` scrim
    `BackgroundColor` alpha is `>= overlays::OVERLAY_SCRIM_ALPHA`
    (0.55) for every `ModalKind`. The `Narrow` kind 0.32-alpha
    exception is allowed iff doc-commented per spec amendment.
  - **AC3 -- Mandatory footer-action slot**: For every `ModalKind`,
    `modal_footer_slot_node()` returns a Node that is a child of the
    panel and is reachable from `modal_root_node()` via the panel.
  - **AC4 -- No z-fight**: Spawning two `ModalRoot` instances yields
    two distinct `ZIndex` values; the second spawn is strictly above
    the first.
  - **AC5 -- No negative-margin centring**: `modal_panel_node()` has
    `align_self: Center` and `justify_self: Center` (or the canonical
    `align_items / justify_content` pattern on the scrim parent); no
    `margin_left: Val::Px(-WIDTH/2)` pattern.
  - **AC6 -- Header / body / footer ordering**: The panel's three
    child slots are in `header / body / footer` declaration order.
    Header is `flex: 0 0 auto`, body is `flex: 1 1 auto` (absorbs
    free space), footer is `flex: 0 0 auto`.
  - **AC8 -- Scrim covers viewport**: The scrim's computed
    rectangle equals the viewport rectangle at every
    `CANONICAL_VIEWPORTS` entry.

- A QA snapshot bundle at 1280 × 720 and 1920 × 1080 across the
  DraftInitial phase showing:
  - Photosensitivity warning text does NOT bleed through the
    DraftInitial modal (PROMPT 1034 F-1 / F-5 fix).
  - `Ready` action sits inside the modal footer (PROMPT 1034 §2.2
    professional expectation #4).
  - `Dismiss` action sits inside the modal header.

### Out of Scope

- **Per-surface migration of all four other modals.** Photosensitivity,
  auction featured, result screen, connection-lost, settlement
  overlay are owned by Phase C.6 family
  `S16-UI-MODAL-PANEL-CONSOLIDATION-001`.
- **Panel chrome consolidation.** Background colour / border / padding
  / border-radius tokens are owned by story 014 panel primitive.
  `ModalRoot` consumes `PanelKind::Standard` / `Narrow` from story
  014 when both are available; if story 014 has NOT landed at
  activation, this story authors inline panel chrome that matches
  the current DraftInitial panel's RGB triple and migrates to
  `PanelKind` in a follow-on row.
- **Button primitive integration.** `Ready` / `Dismiss` in the footer
  / header are rendered with the current per-site button styling
  until story 013 button primitive lands; migration to the button
  primitive is a follow-on row.
- **Focus trapping / screen-reader semantics.** ≥44px hit-targets,
  WCAG contrast, focus-trap on modal open, focus-restoration on
  modal close, ARIA role announcements are OUT OF SCOPE.
- **No new color tokens.** `MODAL_SCRIM_BACKGROUND` (if authored) is
  a re-export of an existing colour or an inline literal that the
  Phase B.1 colours story consolidates. No net-new RGB triple is
  introduced.
- **No `Polish->Release` gate-check retry, no stage advance, no
  Sprint 14 / 15 / 16 row reopen.**

---

## Acceptance Criteria

All criteria are independently checkable BLOCKING criteria.

- [ ] **AC1 -- Primitive module exports**: GIVEN the worker commit,
  WHEN the new module is inspected, THEN
  `client/src/ui/design_tokens/modal.rs` (or
  `client/src/ui/primitives/modal.rs`) exists, is declared from the
  appropriate aggregator, and exports the `ModalKind` enum
  (`Standard`, `Narrow`, `Featured`), the `modal_root_node`,
  `modal_panel_node`, `modal_header_slot_node`,
  `modal_body_slot_node`, `modal_footer_slot_node` builders, and
  per-kind dimension constants. Every public item has a `///` doc
  comment naming its consumer surface(s). No inline magic literal at
  a public-API boundary. Verification: module read + doc-comment scan.

- [ ] **AC2 -- Mandatory opaque scrim**: GIVEN the integration test,
  WHEN run, THEN every `ModalKind`'s `modal_root_node()` returns a
  Node whose `BackgroundColor` alpha is `>= overlays::OVERLAY_SCRIM_ALPHA`
  (0.55). The `Narrow` kind MAY use the documented `overlays.rs`
  0.32-alpha exception ONLY if its doc comment explicitly cites the
  connection-lost AC6 exception. PROMPT 1034 F-1 / F-5 fix is
  verified by AC9 evidence below.

- [ ] **AC3 -- Mandatory footer-action slot**: GIVEN the integration
  test, WHEN run, THEN for every `ModalKind`, `modal_footer_slot_node()`
  returns a Node reachable from `modal_root_node()` via the panel.
  The DraftInitial migration places `Ready` in the footer slot
  (verified by manual evidence in AC9). PROMPT 1034 §2.2
  "professional expectation" #4 is satisfied.

- [ ] **AC4 -- Per-modal z-stack policy**: GIVEN the integration
  test, WHEN run, THEN spawning two `ModalRoot` instances yields two
  distinct `ZIndex` values with the second strictly above the first.
  The base `ZIndex` reads `z_layers::MODAL` (story 002 DONE). PROMPT
  1034 D6 "Z-layer policy is implicit and drifting" is mitigated.

- [ ] **AC5 -- No negative-margin centring**: GIVEN the worker diff
  + integration test, WHEN inspected, THEN the migrated DraftInitial
  modal does NOT use the `Val::Percent(50.0) + margin_left:
  Val::Px(-WIDTH/2)` pattern. `modal_panel_node()` uses
  `align_items / justify_content: Center` on the scrim parent OR
  `align_self / justify_self: Center` on the panel. PROMPT 1035
  §"The featured card is centred via the negative-margin trick"
  pattern is eliminated from at least the DraftInitial site.

- [ ] **AC6 -- Header / body / footer ordering**: GIVEN the
  integration test, WHEN run, THEN the panel's child slots appear in
  `header / body / footer` declaration order with `flex: 0 0 auto /
  1 1 auto / 0 0 auto` sizing.

- [ ] **AC7 -- Phase 1 migration of DraftInitial (or fallback site)**:
  GIVEN the worker diff, WHEN inspected, THEN exactly ONE existing
  modal site is migrated to `ModalRoot`. Default: the DraftInitial
  modal in `client/src/ui/shop_auction/draft_initial.rs` (post
  story 010 split). Fallback (if story 010 has NOT landed): the
  result-screen modal in `client/src/presentation/result_screen.rs`.
  The worker report enumerates which migration site was chosen and
  why. Photosensitivity, auction featured, lobby, connection-lost,
  settlement modals are EXPLICITLY UNCHANGED. Verification: `git
  diff origin/main...HEAD --stat -- 'client/src/ui/' 'client/src/presentation/'`.

- [ ] **AC8 -- Scrim covers viewport at every canonical viewport**:
  GIVEN the integration test, WHEN run, THEN the scrim's computed
  rectangle equals the viewport rectangle at every entry of
  `CANONICAL_VIEWPORTS` (1280 × 720, 1366 × 768, 1920 × 1080, 2560 ×
  1440, plus the sentinel viewports). No viewport produces a scrim
  smaller than the viewport.

- [ ] **AC9 -- Visual evidence**: GIVEN the future `/dev-story`
  worker's evidence directory at
  `production/qa/evidence/sprint-1X-modal-primitive/` (NEW), THEN
  it contains:
  - QA snapshot bundle at 1280 × 720 and 1920 × 1080 showing the
    migrated modal renders correctly with footer action.
  - Screenshot or QA snapshot evidence showing photosensitivity
    warning text does NOT bleed through the migrated modal (PROMPT
    1034 F-1 / F-5 fix). NOTE: photosensitivity gating is independently
    handled by PROMPT 1026 / 1030 (Sprint 15); this AC asserts the
    modal primitive's scrim is opaque enough that even if the gating
    fails, the bleed-through is prevented at the modal layer.
  - Doc-review checklist enumerating each AC1-AC11.
  - Integration-test pass log.
  - `git diff --stat` proving only ONE modal site was migrated.

- [ ] **AC10 -- Non-claims**: GIVEN the worker commit, WHEN closure
  paperwork is inspected, THEN this row does NOT modify any
  gameplay / server / shared / protocol module; does NOT claim
  release readiness / full game completion / Standard-tier
  accessibility / playtest validation / final-art / two-client
  GAME_OVER closure / `Polish->Release` retry / stage advance;
  does NOT reopen any closed row; does NOT introduce focus trapping
  / ARIA roles / Standard-tier ≥44px hit-targets. Verification: `git
  diff origin/main...HEAD --stat -- 'server/' 'shared/'` is empty;
  paperwork review of `/story-done` close-out.

- [ ] **AC11 -- Forbidden literal guards**: GIVEN the integration
  test, WHEN run, THEN no `Color::srgba(..., 0.45)` or `0.46` literal
  is present in the migrated modal site (already forbidden by the
  Sprint 14 `overlay_alpha_test.rs` grep guard; this AC asserts the
  modal primitive does NOT reintroduce them). No `margin_left:
  Val::Px(-` negative-margin literal is present in the migrated
  modal site.

---

## Implementation Notes

### Owned files

| Path | Expected change |
|------|-----------------|
| `client/src/ui/design_tokens/modal.rs` (NEW) | Author `ModalKind` + `modal_root_node` + slot builders + per-kind constants. |
| `client/src/ui/design_tokens/mod.rs` | Declare `pub mod modal;`. |
| `client/src/ui/shop_auction/draft_initial.rs` (post story 010) OR `client/src/presentation/result_screen.rs` (fallback) | Migrate ONE canonical modal site to `ModalRoot`. Move `Ready` action into footer slot for DraftInitial site. |
| `docs/ux/global-ui-design-spec.md` | Amend with new §13 "Modal Primitive" section. |
| `tests/integration/ui_clean_pass/modal_primitive_test.rs` (NEW) | Integration test per AC2-AC8 + AC11. |
| `client/Cargo.toml` | Register `[[test]]` bin if pattern requires (mirror Sprint 14/15 Tier 0 test bins). |
| `production/qa/evidence/sprint-1X-modal-primitive/` (NEW) | Evidence dir per AC9. |

### Forbidden files

- `client/src/ui/photosensitivity_warning.rs` -- migration is a
  separate Phase C.6 row.
- `client/src/ui/lobby.rs` -- migration is a separate Phase C.6 row.
- `client/src/presentation/connection_lost_overlay.rs` -- migration
  is a separate Phase C.6 row.
- `client/src/ui/shop_auction/auction.rs` (post story 010) /
  `auction_featured_card_node` -- migration is a separate Phase C.6
  row.
- `client/src/ui/shop_auction/settlement.rs` (post story 010) --
  migration is a separate Phase C.6 row.
- `client/src/ui/hand/**` -- not a modal consumer.
- `client/src/ui/hud/**` -- not a modal consumer.
- `client/src/ui/design_tokens/{z_layers,typography,spacing,strips,overlays,interaction_states,card_slot}.rs`
  -- read-only inputs.
- `server/src/**`, `shared/src/**` -- UNCHANGED.
- `production/sprint-status.yaml`, `production/sprints/*`,
  `production/stage.txt`, `production/session-state/*`,
  `production/qa/qa-plan-*.md` -- shared-state writers.

---

## Parallelization and Phase Breakdown

| Sibling story | Parallel-safe with this row? |
|---|---|
| **Story 010 shop_auction modsplit** | Primitive authoring: **YES**, file-disjoint. DraftInitial migration target: **conflict** (same file). Producer schedules this AFTER story 010 OR picks fallback migration site. |
| **Story 011 hand modsplit** | **YES**, file-disjoint. |
| **Story 013 button primitive** | Primitive authoring file-disjoint (`button.rs`). Migration: this story's DraftInitial footer-action uses the current per-site `Ready` button; migration to button primitive is a follow-on. |
| **Story 014 panel primitive** | **Coupling**: `ModalRoot::Standard` consumes `PanelKind::Standard` if available. If story 014 lands first, this story consumes the primitive; if not, this story authors inline panel chrome and a follow-on row migrates to `PanelKind`. |
| **Story 015 sequencing doc** | **YES**, doc-only. |
| **Story 009 card-slot primitive** | **YES**, file-disjoint. |

### Dependencies

- **Prerequisite**: `overlays::OVERLAY_SCRIM_ALPHA` (story 006 DONE)
  and `z_layers::MODAL` (story 002 DONE) -- both DONE on `origin/main`.
- **Soft prerequisite**: story 010 shop_auction modsplit, if
  DraftInitial is chosen as canonical migration site.
- **Soft prerequisite**: story 014 panel primitive, for `PanelKind`
  consumption; otherwise inline chrome with follow-on migration.
- **Unblocks**: Phase C.6 family `S16-UI-MODAL-PANEL-CONSOLIDATION-001`
  (per-surface migration of the remaining four modals).

---

## Worker Contract (for `/dev-story`)

The future `/dev-story` worker MUST:

1. Run `git checkout -b work/s16-modal-primitive` from `origin/main`.
2. Read `overlays.rs`, `z_layers.rs`, and the current DraftInitial /
   result-screen modal spawn sites.
3. Author the primitive module + spec amendment per AC1.
4. Migrate ONE canonical site per AC7.
5. Author the integration test bin per AC2-AC8 + AC11.
6. Capture evidence per AC9.
7. Verify `cargo test -p client` across all bin families.
8. Push `work/s16-modal-primitive`. Do NOT push `main`.

The worker MUST NOT:

- Migrate more than ONE modal site (AC7).
- Add focus trapping, ARIA roles, or Standard-tier ≥44px hit-targets.
- Introduce new RGB triples (consume existing or via story 013-related
  Phase B.1 colours story).
- Touch any forbidden file.
- Run `/story-done` / `/smoke-check` / `/team-qa` / `/gate-check` /
  `/release-check` / `/qa-plan`.
- Modify `production/sprint-status.yaml`,
  `production/sprints/sprint-XX.md`, `production/stage.txt`, or
  `production/session-state/*`.

---

`012: S16-TD-UI-MODAL-PRIMITIVE-001: DRAFT`
