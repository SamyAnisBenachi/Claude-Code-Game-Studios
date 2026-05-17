# Story 014: S16-TD-UI-PANEL-PRIMITIVE-001 -- UI Panel Primitive

> **Epic**: UI Clean-Pass
> **Story ID**: S16-TD-UI-PANEL-PRIMITIVE-001
> **Status**: Draft -- Sprint 16/17 candidate, NOT activated
> **Layer**: Presentation / UX foundational tech-debt (shared primitive)
> **Type**: Tech Debt -- foundational primitive (shared widget)
> **Sprint**: Sprint 16/17 Phase B candidate per PROMPT 1035 §"Suggested
> refactor sequence" Phase B.3 + PROMPT 1034 §3 D4 ("No reusable Panel /
> Container chrome"). Per-surface migration of remaining panel sites is
> the Phase C.6 family `S16-UI-MODAL-PANEL-CONSOLIDATION-001`.
> **Authored**: 2026-05-17 by PROMPT 1044
> **Authoring source-of-truth**: `origin/main@a7a8b079` (PROMPT 1041).
> **Estimated effort**: ~0.5d (primitive module + spec amendment +
> single canonical migration + chrome-consistency test)

---

## Status / No-Claim Banner

This story is authored as a Sprint 16/17 candidate. **No sprint is
activated by this authoring run.** PROMPT 1044 does NOT activate any
sprint, modify sprint-status / sprint plan / stage / session-state,
run any `/dev-story` / `/story-done` / `/smoke-check` / `/team-qa` /
`/gate-check` / `/release-check` / `/qa-plan` workflow, modify code
under `client/` / `server/` / `shared/` / `tests/`, or author the
future `client/src/ui/design_tokens/panel.rs` module.

This story does **not** claim: public release readiness, full game
completion, broad / Standard-tier accessibility completion
(`QA-COND-0005`), Standard-tier hit-target conformance (≥44px),
playtest validation (`QA-COND-0006`), full playable-client manual QA,
two-client GAME_OVER closure (`S8-QA-001-W1`), final-art completion
(`PAW-TD-*-a`), `Polish->Release` retry, or stage advance.

---

## Overview

PROMPT 1034 §3 D4 ("No reusable Panel / Container chrome") + PROMPT
1035 §"Result screen / connection-lost / photosensitivity (cluster of
three centred-modal panels)" catalogues:

- Three modal panels each with **three different panel background RGB
  triples and three different border RGB triples**:
  - `result_screen.rs:546` panel bg `Color::srgba(0.055, 0.062, 0.078,
    0.94)` + border `(0.82, 0.86, 0.9, 0.26)`.
  - `connection_lost_overlay.rs:237` panel bg `Color::srgba(0.16,
    0.10, 0.04, 0.92)` + border `(0.96, 0.74, 0.30, 0.85)`
    (brown-amber).
  - `photosensitivity_warning.rs:89` panel bg `Color::srgb(0.08,
    0.12, 0.18)` + border `(0.92, 0.94, 0.96)`.
- Two panel-size triples that hard-mirror each other:
  `LOBBY_PANEL_*` (`lobby.rs:32-40`: 88 % / 860 px / 92 %) ↔
  `result_screen.rs:537-539` (verbatim 88 % / 860 px / 92 %) -- per
  Sprint 14 story 024 PROMPT 933 Option A intentional mirror.
- One "narrow" panel size for connection-lost: 60 % / 520 px.
- Placement left column ("Select a card / ? / 8 / Submit / (0") with
  **no container chrome** (PROMPT 1034 F-7).
- Auction roster strip with no container chrome (PROMPT 1034 §2.5).
- HUD has no badge containers (PROMPT 1034 D4).

The root cause is missing **`PanelKind` chrome tokens**. Spec §10
"Panel chrome" reserves the slot but no primitive consumes it.

This story authors the future `/dev-story` worker's contract for a
**`Panel` primitive** that publishes `PanelKind::{Standard, Narrow}`
with backgrounds derived from a canonical `SURFACE_ELEVATED` colour
(pending Phase B.1 colours story; inline-literal fallback acceptable
here), borders, padding, and border-radius. Consumed by at least ONE
canonical migration site (the Placement action panel -- PROMPT 1034
A8 P1 fix; OR the result-screen panel if Placement migration is
blocked).

Per-surface migration of the remaining four panel sites (lobby,
photosensitivity, connection-lost, draft-initial modal, settlement
overlay) is the Phase C.6 family `S16-UI-MODAL-PANEL-CONSOLIDATION-001`.

Per PROMPT 1035, this primitive authoring is **file-disjoint** from
stories 010 / 011 / 012 / 013. The `Standard` variant is consumed by
story 012 modal primitive's `ModalKind::Standard`; story 012 and
this story are mutually compatible (either lands first; the other
re-exports / consumes).

---

## Scope

### In Scope

#### Primitive module

- A new primitive module at
  `client/src/ui/design_tokens/panel.rs` (NEW; future `/dev-story`
  output). The module exports:

  - **`PanelKind` enum**:
    - `Standard` -- the canonical centred panel chrome (lobby modal,
      result-screen, photosensitivity, draft-initial). Size: 88 %
      width / 860 px max-width / 92 % max-height (mirrors current
      `LOBBY_PANEL_*` triple).
    - `Narrow` -- the connection-lost-style tight panel. Size: 60 %
      width / 520 px max-width.
    - `Toolbar` -- a non-modal panel chrome for surfaces like the
      Placement action panel (PROMPT 1034 A8) and auction roster
      strip. No max-width / max-height; sized to content.
    - `Hud` -- OPTIONAL; a HUD-readout-cluster chrome consumed by
      future HUD primitive lift (per PROMPT 1035 §"HUD pill primitive
      lift" Phase C.10).

  - **`PanelGeometry` struct** carrying:
    - `width: Val`, `max_width_px: Option<f32>`,
      `max_height_percent: Option<f32>`.
    - `padding: UiRect`.
    - `border: UiRect` thickness.
    - `border_radius: f32`.
    - `background_color: Color`.
    - `border_color: Color`.

  - **`panel_node(kind: PanelKind) -> Node`** that returns a Node
    configured per kind. The panel chrome (background +
    border + border_radius + padding) is set from the geometry struct.

  - **`panel_chrome_color(kind: PanelKind) -> (Color, Color)`** that
    returns `(background, border)` for the kind. This is the
    consolidation point: the three different panel bg + border RGB
    pairs converge to one `Standard` set and one `Narrow` set.

  - **Per-kind constants**: `PANEL_STANDARD_*`, `PANEL_NARROW_*`,
    `PANEL_TOOLBAR_*` with documented values. Each constant has a
    `///` doc comment naming its consumer surface(s).

  - **Forbidden-pattern doc-comment**: `panel.rs` MUST contain a doc
    comment listing the three RGB triples being consolidated (the
    PROMPT 1035 §"Result screen / connection-lost / photosensitivity
    cluster" inventory) so a future Phase C.6 migration sweep can
    grep against them.

#### Global UI design spec amendment

- An amendment to `docs/ux/global-ui-design-spec.md` adding a new
  spec section (suggested §15 "Panel Primitive" or §13 if it lands
  before story 012 modal; section number is worker-discretion). The
  section:
  - Names the `PanelKind` variants and per-kind dimensions / chrome.
  - Documents the three RGB triples being consolidated (lobby /
    result-screen / photosensitivity / connection-lost / draft-initial
    / settlement) as deprecation targets.
  - Cross-references `SPACING_*` (story 003 DONE) for padding.
  - Cross-references `z_layers::*` for stacking.
  - Preserves the friend-game vs Standard-tier scope boundary
    verbatim: panel chrome is a *layout / colour* contract only;
    WCAG contrast on background-to-text contrast is NOT introduced
    here (a future Standard-tier row would amend `panel_chrome_color`
    to enforce contrast).

#### Phase 1 canonical migration -- Placement action panel

- A first canonical migration of one existing panel site to the
  primitive: the **Placement action panel** -- the floating
  "Select a card / ? / 8 / Submit / (0" column that PROMPT 1034 F-7
  flags as having no container chrome.
  - The site is OWNED by `client/src/ui/hand/submit.rs` (post story
    011 split) OR `client/src/presentation/placement*`. Producer
    chooses based on which split has landed.
  - Wrap the existing action-panel contents in
    `panel_node(PanelKind::Toolbar)` so the column gains a visible
    backplate + border. The countdown integer, "Select a card"
    caption, and `Submit` button (rendered via story 013 button
    primitive when available; per-site styling otherwise) become
    flex children of the panel.
  - **Fallback canonical site** if Placement action panel migration
    is blocked: the result-screen panel in
    `client/src/presentation/result_screen.rs` -- replace the inline
    `BackgroundColor + BorderColor + padding + border_radius`
    composition with `panel_node(PanelKind::Standard) +
    panel_chrome_color(PanelKind::Standard)`.

#### Tests

- A new integration test bin at
  `tests/integration/ui_clean_pass/panel_primitive_test.rs` (NEW)
  asserting:
  - **AC2 -- Per-kind chrome distinct**: For every `PanelKind`,
    `panel_chrome_color(kind)` returns a distinct `(background,
    border)` pair (no two kinds collapse).
  - **AC3 -- Standard / Narrow size invariants**:
    `PanelKind::Standard` `max_width_px = 860.0`,
    `PanelKind::Narrow` `max_width_px = 520.0` (matches current
    canonical sizes per PROMPT 1035 §"Result screen / connection-lost
    / photosensitivity cluster").
  - **AC4 -- Padding consumes SPACING_***: Per-kind padding is
    derived from `spacing::SPACING_*` constants (story 003 DONE),
    not bare `Val::Px` literals at the primitive layer.
  - **AC5 -- Toolbar has no max-width**: `PanelKind::Toolbar`
    `max_width_px = None` and `max_height_percent = None` (sized to
    content; consumed by Placement action panel and future auction
    roster).
  - **AC6 -- Deprecation grep guard**: The three RGB triples being
    consolidated (`Color::srgba(0.055, 0.062, 0.078, 0.94)`,
    `Color::srgba(0.16, 0.10, 0.04, 0.92)`, `Color::srgb(0.08, 0.12,
    0.18)`) are NOT present in the migrated panel's source line.
    (Grep guard is scoped to the migrated site; the un-migrated sites
    still carry the literals until Phase C.6 sweep lands.)

- A QA snapshot bundle at 1280 × 720 and 1920 × 1080 showing the
  migrated panel renders with a visible backplate + border (Placement
  action panel: floating column gains chrome; OR result-screen panel:
  same render as before but via primitive).

### Out of Scope

- **Per-surface migration of all six other panel sites.** Lobby
  modal, photosensitivity, connection-lost, result-screen,
  draft-initial, settlement overlay are owned by Phase C.6 family
  `S16-UI-MODAL-PANEL-CONSOLIDATION-001`.
- **HUD pill primitive lift.** PROMPT 1035 §"HUD pill primitive lift"
  Phase C.10 is a separate row; the `Hud` variant of `PanelKind`
  here is optional and reserved for that future row.
- **Auction roster strip chrome.** PROMPT 1034 §2.5 "auction roster
  strip has no container chrome" is fixed by a Phase C row that
  consumes `PanelKind::Toolbar`; not authored here.
- **Auction bid-row chrome.** PROMPT 1034 D4 mentions the green
  auction timer bar floats with no container -- that surface lives
  under `AuctionToolbar` (Phase C.4 `S16-UI-AUCTION-FLEX-PRIMITIVES-001`),
  not this primitive.
- **Phase B.1 colours.** This primitive may carry inline RGB
  literals for `Standard` / `Narrow` / `Toolbar` chrome at authoring
  time; consolidation to `colors::SURFACE_ELEVATED` / `colors::ACCENT`
  (etc.) is the Phase B.1 / Phase C.7 sweep.
- **Standard-tier contrast enforcement.** WCAG ratios on
  background-to-text contrast are NOT introduced.
- **No `Polish->Release` retry, no stage advance, no closed-row
  reopen.**

---

## Acceptance Criteria

All BLOCKING.

- [ ] **AC1 -- Primitive module exports**: `client/src/ui/design_tokens/panel.rs`
  exists, declared from aggregator, exports `PanelKind` (≥ `Standard`,
  `Narrow`, `Toolbar`), `PanelGeometry`, `panel_node`,
  `panel_chrome_color`, per-kind dimension constants, with `///`
  doc comments naming consumer surfaces and listing the three RGB
  triples being consolidated.

- [ ] **AC2 -- Per-kind chrome distinct**: For every `PanelKind`,
  `panel_chrome_color(kind)` returns a distinct `(background,
  border)` pair.

- [ ] **AC3 -- Standard / Narrow size invariants**:
  `Standard.max_width_px = 860.0`, `Standard.width = Val::Percent(88.0)`,
  `Narrow.max_width_px = 520.0`. These match the current canonical
  sizes used by lobby + result-screen + connection-lost.

- [ ] **AC4 -- Padding consumes SPACING_***: At least one per-kind
  padding value reads `spacing::SPACING_*` (e.g.
  `PANEL_STANDARD_PADDING = UiRect::all(Val::Px(spacing::SPACING_LG))`).
  Bare pixel literals at the public-API boundary are forbidden.

- [ ] **AC5 -- Toolbar has no max-width**:
  `PanelKind::Toolbar.max_width_px == None` and
  `max_height_percent == None`.

- [ ] **AC6 -- Phase 1 migration of single canonical site**: Exactly
  ONE existing panel site is migrated. Default: Placement action
  panel. Fallback: result-screen panel. Lobby, photosensitivity,
  connection-lost, draft-initial, settlement, auction roster,
  auction bid-row are EXPLICITLY UNCHANGED. Verification: `git diff
  origin/main...HEAD --stat`.

- [ ] **AC7 -- Deprecation grep guard**: The three deprecation-target
  RGB triples are absent from the migrated panel's source. Sweep of
  the other panel sites is OUT OF SCOPE for this row.

- [ ] **AC8 -- Visual evidence**: Evidence directory at
  `production/qa/evidence/sprint-1X-panel-primitive/` contains QA
  snapshot bundles at 1280 × 720 and 1920 × 1080 showing the
  migrated panel renders with visible backplate + border. PROMPT
  1034 F-7 ("action panel has no container chrome") is shown
  resolved for the migrated site.

- [ ] **AC9 -- Non-claims**: No gameplay / server / shared change.
  No release / full-game / Standard-tier / playtest / final-art /
  two-client closure claim. No closed-row reopen. No WCAG contrast
  enforcement.

- [ ] **AC10 -- No new tokens beyond what spec amendment names**:
  No new `Color::srgb*(...)` literal that isn't either (a)
  consolidating one of the three deprecation-target RGB triples or
  (b) a worker-justified default for the `Toolbar` kind.

---

## Implementation Notes

### Owned files

| Path | Expected change |
|------|-----------------|
| `client/src/ui/design_tokens/panel.rs` (NEW) | Author `PanelKind` + `PanelGeometry` + builders + per-kind constants. |
| `client/src/ui/design_tokens/mod.rs` | Declare `pub mod panel;`. |
| Migrated panel site (Placement action panel OR result-screen) | Wrap inline panel composition in `panel_node` + `panel_chrome_color`. |
| `docs/ux/global-ui-design-spec.md` | Amend with new "Panel Primitive" section. |
| `tests/integration/ui_clean_pass/panel_primitive_test.rs` (NEW) | Integration test per AC2-AC5 + AC7. |
| `client/Cargo.toml` | Register `[[test]]` bin if pattern requires. |
| `production/qa/evidence/sprint-1X-panel-primitive/` (NEW) | Evidence dir per AC8. |

### Forbidden files

- `client/src/ui/lobby.rs` -- migration is separate Phase C.6.
- `client/src/ui/photosensitivity_warning.rs` -- migration is
  separate Phase C.6.
- `client/src/presentation/connection_lost_overlay.rs` --
  migration is separate Phase C.6.
- `client/src/ui/shop_auction/draft_initial.rs` /
  `client/src/ui/shop_auction/settlement.rs` (post story 010) --
  migration is separate Phase C.6.
- `client/src/ui/hud/**` -- HUD pill primitive lift is Phase C.10.
- `client/src/ui/design_tokens/{z_layers,typography,spacing,strips,overlays,interaction_states,card_slot,modal,button}.rs`
  -- read-only inputs (or sibling primitives that may be lifted in
  parallel; story 014 does not edit them).
- `server/src/**`, `shared/src/**` -- UNCHANGED.
- `production/sprint-status.yaml`, `production/sprints/*`,
  `production/stage.txt`, `production/session-state/*` --
  shared-state writers.

---

## Parallelization and Dependencies

| Sibling story | Parallel-safe? |
|---|---|
| **Story 003 spacing tokens (DONE)** | **Hard prerequisite.** AC4 requires `spacing::SPACING_*` consumption. Already DONE. |
| **Story 010 shop_auction modsplit** | Primitive authoring: **YES**. Per-shop_auction panel migration: blocked until story 010 lands. |
| **Story 011 hand modsplit** | Primitive authoring: **YES**. Placement migration: producer picks site. |
| **Story 012 modal primitive** | **YES**, file-disjoint. `ModalKind::Standard` consumes `PanelKind::Standard` if both available. |
| **Story 013 button primitive** | **YES**, file-disjoint. |
| **Story 015 sequencing doc** | **YES**, doc-only. |
| **Phase B.1 colours story** | This row authors inline RGB triples for `Standard` / `Narrow` / `Toolbar`; Phase B.1 sweeps them later. No conflict. |

### Dependencies

- **Prerequisite**: `spacing::SPACING_*` (story 003 DONE).
- **Soft prerequisite**: story 011 hand modsplit, if Placement
  action panel lives under hand/submit.rs after split.
- **Unblocks**: Phase C.6 family `S16-UI-MODAL-PANEL-CONSOLIDATION-001`
  (per-surface migration of lobby / photosensitivity / connection-lost
  / result-screen / draft-initial / settlement panels).
- **Unblocks**: Phase C.10 `S16-UI-HUD-PILL-PRIMITIVE-001` consumes
  the `Hud` variant if authored.

---

## Worker Contract (for `/dev-story`)

The future `/dev-story` worker MUST:

1. Run `git checkout -b work/s16-panel-primitive` from `origin/main`.
2. Read `spacing.rs` and the current Placement action panel /
   result-screen panel spawn sites.
3. Author the primitive module + spec amendment per AC1.
4. Migrate ONE canonical site per AC6.
5. Author the integration test bin per AC2-AC5 + AC7.
6. Capture evidence per AC8.
7. Verify `cargo test -p client`.
8. Push `work/s16-panel-primitive`. Do NOT push `main`.

The worker MUST NOT:

- Migrate more than ONE panel site.
- Introduce WCAG contrast checks or Standard-tier accessibility.
- Sweep the three deprecation-target RGB triples from un-migrated
  sites (that is Phase C.6).
- Touch any forbidden file.
- Run `/story-done` / `/smoke-check` / `/team-qa` / `/gate-check` /
  `/release-check` / `/qa-plan`.
- Modify `production/sprint-status.yaml`,
  `production/sprints/sprint-XX.md`, `production/stage.txt`, or
  `production/session-state/*`.

---

`014: S16-TD-UI-PANEL-PRIMITIVE-001: DRAFT`
