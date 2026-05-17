# Story 009: S12-TD-UI-CARD-SLOT-PRIMITIVE-001 -- UI Card Slot Primitive

> **Epic**: UI Clean-Pass
> **Story ID**: S12-TD-UI-CARD-SLOT-PRIMITIVE-001
> **Status**: Draft -- Sprint 16 candidate, NOT activated
> **Layer**: Presentation / UX foundational tech-debt
> **Type**: Tech Debt -- foundational primitive (multi-surface layout contract)
> **Sprint**: Sprint 16 candidate (Tier 3 rank 13 per
> `docs/ux/ui-clean-pass-roadmap.md`; PROMPT 802 §3.3 HA1 / §3.3 HA5 /
> §4 Tier 3.1). Deliberately deferred from Sprint 15 per
> `production/sprints/sprint-15.md` "Wider Sprint 15 Backlog
> (NOT scheduled into this draft; deferred) -- Deliberately deferred to
> Sprint 16+ (size or coordination overhead)" (≈1.5d + ~0.5d authoring +
> ~0.5d integration friction; multi-surface refactor touching hand +
> shop + auction together per PROMPT 802 §8).
> **Authored**: 2026-05-17 by PROMPT 1025
> **Authoring source-of-truth**: `origin/main@7b663df75e63a4e46512c5d88e0de2aa704a114a`
> (PROMPT 1023 `integrate(s15): default QA snapshot enabled in dev builds`).
> **Estimated effort**: ~1.5d primitive + per-phase migration; an alternative
> phased breakdown is captured in §"Parallelization and Phase Breakdown"
> below if Sprint 16 producer prefers four narrow follow-ons over one
> bundled story.

---

## Status / No-Claim Banner

This story is authored as a Sprint 16 candidate. **Sprint 16 is NOT activated
by this authoring run.** The story is paperwork only -- no code change is
attempted by PROMPT 1025.

PROMPT 1025 (this authoring run) does NOT:

- Activate Sprint 16.
- Modify `production/sprint-status.yaml`.
- Modify `production/sprints/sprint-14.md`, `production/sprints/sprint-15.md`,
  `production/sprints/sprint-16.md` (draft), or any other sprint plan file.
- Modify `production/stage.txt` (remains `Polish`).
- Modify any `production/session-state/*` file.
- Modify any QA-plan / smoke / Team-QA / gate-check / release-check artifact.
- Run `/story-readiness`, `/dev-story`, `/story-done`, `/smoke-check`,
  `/team-qa`, `/gate-check`, `/release-check`, or `/qa-plan` on this story.
- Modify any code under `client/`, `server/`, `shared/`, or `tests/`.
- Modify `docs/ux/global-ui-design-spec.md`, `docs/ux/board-rendering-spec.md`,
  or `docs/ux/ui-clean-pass-roadmap.md` (the spec amendment that pairs with
  this primitive module is a future `/dev-story` deliverable, NOT a
  story-authoring deliverable).
- Author `client/src/ui/design_tokens/card_slot.rs` (that is the future
  `/dev-story` output, not this authoring prompt's output).

This story does **not** claim: public release readiness, release-candidate
readiness, full game completion, broad / Standard-tier accessibility
completion (`QA-COND-0005`), Standard-tier hit-target conformance (≥44px),
playtest / fun-hypothesis validation (`QA-COND-0006`), full playable-client
manual QA, two-client GAME_OVER closure (`S8-QA-001-W1`), final-art /
asset-production completion (`PAW-TD-*-a`), `Polish->Release` gate-check
retry (PROMPT 761 FAIL preserved), or stage advance from `Polish` to
`Release`.

---

## Overview

PROMPT 802 §3.3 HA1 and §3.3 HA5 surfaced that the playable client has
**no canonical card-slot primitive**: every surface that paints a card --
hand fan, draft initial grid, shop slots, auction featured card, board
staged ghost preview -- authors its own slot Node with per-site width /
height / aspect-ratio / image-bounds / text-bounds / hover-target /
z-layer literals. The result is **layout drift across surfaces**:

- Hand fan card display: 96 × 136 px portrait
  (`HAND_CARD_DISPLAY_WIDTH_PX` / `HAND_CARD_DISPLAY_HEIGHT_PX` at
  `client/src/ui/hand/mod.rs:62-63`).
- Draft initial grid card: 120 × 56 px landscape
  (`HAND_DRAFT_GRID_CARD_WIDTH_PX` / `HAND_DRAFT_GRID_CARD_HEIGHT_PX` at
  `client/src/ui/hand/mod.rs:64-65`).
- Shop slot well: 136 × 78 px landscape (inline literals in
  `shop_slot_node` at `client/src/ui/shop_auction/mod.rs:4642-4652`).
- Auction featured card: 380 × 280 px landscape
  (`AUCTION_FEATURED_CARD_WIDTH_PX` / `AUCTION_FEATURED_CARD_HEIGHT_PX` at
  `client/src/ui/shop_auction/mod.rs:71-72`).
- Board staged ghost preview: world-space sprite per
  `docs/ux/board-rendering-spec.md` §3 (BR-001 / BR-002 ratified
  `cell_to_world` geometry; not a bevy_ui consumer but participates in the
  drag-from-hand visual contract per `docs/ux/board-rendering-spec.md`
  §4.5).

Each surface re-authors its own aspect ratio, image bounds, text bounds,
hit target, hover / focus / pressed / disabled state mapping (currently
absent for card slots; tracked separately by story 008
`S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001` DONE), and z-layer.
PROMPT 802 §8 calls out that a refactor here "touches hand + shop +
auction together"; that breadth is why Sprint 15 explicitly deferred
this row per `production/sprints/sprint-15.md` "Wider Sprint 15 Backlog
... Deliberately deferred to Sprint 16+ (size or coordination
overhead)".

This story authors a new shared card-slot primitive module at
`client/src/ui/design_tokens/card_slot.rs` (future `/dev-story` output;
NOT authored by this story file) that exports:

1. A named **card-slot size catalog** with one entry per canonical
   in-game card surface (hand fan, draft grid, shop slot, auction
   featured, board staged-ghost). Each entry carries width, height,
   aspect-ratio band, image-region inset, text-region inset, named
   z-layer reference (`UI_BASE` / `UiOverlay` / `Modal` per §3
   z-layer spec), and a doc comment naming its consumer surface.
2. A named **layout contract** API that returns a fully-composed
   `bevy::ui::Node` (or equivalent reusable component / bundle) for each
   catalog entry so that surfaces no longer write inline `Node { width:
   Val::Px(...), height: Val::Px(...), ... }` literals. The contract
   guarantees stable aspect-ratio (no layout shift when the surface
   re-renders at a different `Display` viewport per §8 of the global UI
   spec) and prevents nested card composition (a card slot never
   contains another card slot).
3. A named **hit-target rectangle** API that returns the canonical
   pointer-hit / focus-target geometry for each catalog entry, separate
   from the visual rectangle so that hover / focus / pressed / disabled
   states (from story 008's `interaction_states` module) compose cleanly
   even when the visual rectangle has e.g. a 1 px chrome border or a
   2 px outset focus ring.
4. A named **image / text bounds** API that returns the inset
   `bevy::ui::UiRect` (or equivalent) for the card's art region and
   text-block region. This guarantees that long card names, long cost
   readouts, and rare-art atlas frames do not overflow the slot's outer
   rectangle at 1280 × 720 (the smallest canonical viewport per §8) or
   at any larger canonical viewport. Containment is asserted by the
   AC8 viewport-invariant test below.
5. A doc-comment cross-reference to `docs/ux/global-ui-design-spec.md`
   (new spec section, future `/dev-story` output) so consumers see one
   ratified source of truth for card-slot geometry.

Per Sprint 16 plan draft (`production/sprints/sprint-16.md` `sprint-plan/
sprint-16-draft`, NOT activated by this story-authoring prompt):

- **Per-surface migration of existing Sprint 14 / 15 card surfaces** is
  authored in this story as **phased** sub-acceptance per AC5 below.
  The Sprint 16 producer MAY (a) bundle the primitive + all phases into
  one story OR (b) split the primitive into its own row and author
  follow-on rows per phase (hand surfaces / shop+auction surfaces / draft
  surfaces / board-ghost surface). The default scope in this story file
  is "primitive module + spec amendment + one canonical migration phase
  (shop slot, as the lowest-risk surface per PROMPT 802 §3.5 S2) +
  evidence captures"; the remaining three phases are scoped as Sprint
  16+ follow-on rows in the family `S16-UI-CARD-SLOT-MIGRATION-*` so
  Sprint 16 has the freedom to size them per producer discretion.

Per `docs/ux/ui-clean-pass-roadmap.md` rank 13 (Tier 3, Should, 1.5d,
net-new): "depends on ranks 1, 2, 3, 6 + at least one Tier 1 surface
stable". Dependencies satisfied at story-authoring time:

- Rank 1 (`S11-TD-UI-ZINDEX-LAYERS`) DONE Sprint 14 PROMPT 903.
- Rank 2 (`S11-TD-UI-FONT-CONSTANTS`) DONE Sprint 14 PROMPT 908.
- Rank 3 (`S11-TD-UI-FLEX-STRIPS`) DONE Sprint 14 PROMPT 919.
- Rank 6 (`S12-UX-GLOBAL-UI-DESIGN-SPEC-001`) DONE Sprint 14 PROMPT 922.
- Multiple Tier 1 surfaces stable: HUD top strip
  (`S11-UX-HUD-TOP-STRIP-LAYOUT` DONE Sprint 14 PROMPT 942), auction
  featured card (`S11-UX-AUCTION-FEATURED-CARD` DONE Sprint 14
  PROMPT 931), draft centered modal
  (`S11-UX-DRAFT-GRID-CENTERED-MODAL` DONE Sprint 14 PROMPT 953), lobby
  modal (`S12-UX-LOBBY-LAYOUT-MODAL-001` DONE Sprint 14 PROMPT 939).
- Adjacent rank: story 008 (`S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001`)
  DONE Sprint 15 PROMPT 1009 -- interaction-state visual primitives
  consumable by the card-slot focus / hover / pressed / disabled
  mappings authored by this story.

---

## Scope

### In Scope

#### Primitive module (always in scope; first phase)

- A new design-token primitive module at
  `client/src/ui/design_tokens/card_slot.rs` (NEW; future `/dev-story`
  output, NOT authored by this story file). The module exports:

  - **`CardSlotKind` enum** with one variant per canonical card surface:
    - `HandFan` -- the player's hand fan card (96 × 136 px portrait,
      consumed by `client/src/ui/hand/mod.rs`).
    - `DraftGrid` -- the draft initial grid card (120 × 56 px landscape,
      consumed by `client/src/ui/hand/mod.rs` draft initial grid pane).
    - `ShopSlot` -- the shop slot well (136 × 78 px landscape, consumed
      by `client/src/ui/shop_auction/mod.rs` shop panel).
    - `AuctionFeatured` -- the auction featured card (380 × 280 px
      landscape, consumed by `client/src/ui/shop_auction/mod.rs` auction
      panel).
    - `BoardStagedGhost` -- the staged ghost preview (world-space; sized
      to one board cell per `docs/ux/board-rendering-spec.md` BR-001
      `cell_to_world`; not a bevy_ui consumer but participates in the
      drag-from-hand visual contract).

  - **`CardSlotGeometry` struct** carrying the per-kind layout data:
    - `outer_width_px: f32`, `outer_height_px: f32` -- outer visual
      rectangle.
    - `aspect_ratio_band: (f32, f32)` -- (min, max) aspect ratio that
      the slot is allowed to settle into. The contract enforces a
      stable aspect ratio: `outer_width_px / outer_height_px` MUST fall
      inside this band, and the layout API returned for the kind MUST
      preserve the aspect ratio across the canonical viewport matrix
      (see §8 of the global UI spec).
    - `image_inset_px: bevy::ui::UiRect` -- inset between the slot's
      outer rectangle and the card art's region.
    - `text_inset_px: bevy::ui::UiRect` -- inset between the slot's
      outer rectangle and the card's text-block region (name + cost
      readout).
    - `hit_target_inset_px: bevy::ui::UiRect` -- inset between the slot
      outer rectangle and the canonical hit-target rectangle. Default
      is `UiRect::ZERO` (hit target equals visual outer rectangle); a
      surface that needs a larger or offset hit target (e.g. focus
      ring outset) reads this directly.
    - `z_layer: bevy::ui::GlobalZIndex` -- named z-layer reference
      from `client/src/ui/design_tokens/z_layers.rs` (`UI_BASE` for
      hand fan / shop slot / draft grid / auction featured; `UiOverlay`
      for the board staged ghost while dragged from hand; no kind
      uses `Modal` or `Toast`).

  - **`card_slot_geometry(CardSlotKind) -> CardSlotGeometry`** -- a
    `const fn` (if Rust 2021 allows for this combination of `UiRect`
    + `GlobalZIndex`) or a plain `fn` lookup that returns the
    canonical geometry for a given kind. Implementing as `const fn`
    is worker-discretion; the API shape is "given a kind, return the
    geometry".

  - **`card_slot_node(CardSlotKind) -> bevy::ui::Node`** -- builds a
    fully-composed `Node` for the given kind, with `position_type`,
    `width`, `height`, `border`, and `padding` set from the geometry.
    The Node MUST have a deterministic `Display` and `FlexDirection`
    (worker-discretion within `Display::Flex` / `Display::Block` per
    surface; default `Display::Flex` with `FlexDirection::Column` so
    image-then-text vertical stacking is the cheap default for the
    landscape kinds and image-with-overlay-text composition is the
    cheap default for the portrait `HandFan` kind via an `Absolute`
    text child).

  - **`card_slot_image_inset(CardSlotKind) -> bevy::ui::UiRect`** and
    **`card_slot_text_inset(CardSlotKind) -> bevy::ui::UiRect`** --
    convenience accessors that return the per-kind insets.

  - **`card_slot_hit_target(CardSlotKind) -> bevy::ui::UiRect`** --
    accessor that returns the per-kind hit-target rectangle (relative
    to the slot outer rectangle, in `Val::Px` units).

  - Doc comments on every published item naming the canonical
    consumer surface(s), the cross-link to the new global UI design
    spec section, and the friend-game scope guard (this primitive is
    a *layout* contract, not an accessibility contract; ≥44px
    hit-targets are NOT enforced per `QA-COND-0005` accept-risk; WCAG
    contrast checks are NOT enforced; final-art replacement is NOT
    advanced under `PAW-TD-*-a` accept-risk).

  - A re-export of the new module from
    `client/src/ui/design_tokens/mod.rs` (existing aggregator from
    Sprint 14 Tier 0 stories 002 / 003 / 004 / 006 + Sprint 15 story
    008), so consumers can
    `use client::ui::design_tokens::card_slot::*;` (or the
    project-idiomatic equivalent).

#### Global UI design spec amendment (always in scope; first phase)

- An amendment to `docs/ux/global-ui-design-spec.md` (future
  `/dev-story` output; NOT authored by this story file) adding a new
  spec section (suggested location: §12 "Card Slot Primitive", after
  §11 "Interaction State Primitives") that:

  - Names the five `CardSlotKind` variants and lists each one's
    canonical width / height / aspect-ratio band / image inset / text
    inset / hit-target inset / z-layer.
  - Points at the new module file
    `client/src/ui/design_tokens/card_slot.rs`.
  - Replaces the existing §10 "Card slot composition" stub
    ("Owned by Tier 3 story 13 ... This spec does not bind a card slot
    composition; story 13 authors the primitive after Tier 1 surfaces
    stabilise.") with a forward reference to the new §12.
  - Updates the Spec Adoption Matrix "Tier 3 deferred to Sprint 15"
    row (currently at ~line 628 in the spec) to cite the new §12 and
    flips its status from "Sprint 15 refactor; reads this spec." to a
    closed reference reflecting whichever Sprint 16 row(s) consume the
    primitive.
  - Preserves the friend-game-vs-Standard-tier scope boundary verbatim
    per §2: the card-slot primitive is a *layout* primitive only; it
    does NOT advance `QA-COND-0005` Standard-tier hit-target
    conformance (≥44px); WCAG contrast on the slot chrome is NOT
    introduced; `PAW-TD-*-a` placeholder-art accept-risk is preserved
    (the primitive composes the *layout*, not the art).

#### Phase 1 canonical migration -- shop slot (always in scope; first phase)

- A first canonical migration of one existing card surface call site to
  the new primitive, **demonstrating the contract**. The default
  canonical first surface is the **shop slot well**
  (`client/src/ui/shop_auction/mod.rs::shop_slot_node` at ~line 4642):

  - Replace the inline `Node { width: Val::Px(136.0), height:
    Val::Px(78.0), ... }` literal with a call to
    `card_slot_node(CardSlotKind::ShopSlot)`.
  - Verify the shop slot still composes at the canonical viewport
    matrix (per §8 of the global UI spec; the existing
    `tests/integration/helpers/ui_viewport.rs::CANONICAL_VIEWPORTS`
    six-viewport set).
  - The image / text bounds and hit-target rectangle for the shop slot
    are validated against the existing shop slot well's PNG dimensions
    (`assets/ui/shop_slot_well.png` or whichever asset path is in
    play at `/dev-story` time per
    `client/src/presentation/asset_paths.rs` constants).

  Shop slot is chosen as canonical phase 1 because:

  - It is the lowest-risk surface per PROMPT 802 §3.5 S2 (it currently
    uses placeholder PAW-003 PNGs with bare-literal layout; layout-only
    repair has no behavioural side-effect).
  - It already has a stable Tier 1 surface story closed
    (`S11-UX-AUCTION-FEATURED-CARD` DONE Sprint 14 PROMPT 931
    -- closely-related surface; both sit inside
    `client/src/ui/shop_auction/mod.rs`).
  - It is small (3 shop slots per round, 1 atlas frame each) so the
    migration touches a narrow code surface.
  - It does NOT depend on hand-fan re-spawn timing, draft initial
    overlay state machine, or board ghost spawn / despawn drain. The
    other three migration phases (hand surfaces / draft surfaces /
    board ghost) carry that incremental risk.

#### Phase 1 evidence (always in scope; first phase)

- A new viewport-invariant integration test bin at
  `tests/integration/ui_clean_pass/card_slot_primitive_test.rs` (NEW)
  asserting:
  - All five `CardSlotKind` variants are importable from the module's
    public path.
  - Each kind's `outer_width_px` and `outer_height_px` are strictly
    positive.
  - Each kind's `outer_width_px / outer_height_px` falls within its
    declared `aspect_ratio_band`.
  - Each kind's image inset + text inset fit inside the outer rectangle
    (i.e., `image_inset.left + image_inset.right < outer_width_px` and
    similarly for vertical / for text inset).
  - Each kind's image region and text region do not overlap (worker
    chooses how to express the disjointness; e.g. they sit in
    different vertical bands).
  - The aspect ratio is preserved at every canonical viewport in
    `CANONICAL_VIEWPORTS` per §8 of the global UI spec (this is the
    "stable aspect-ratio across viewports" assertion).
  - The hit-target rectangle for each kind is a superset of or equal
    to the visual outer rectangle (hit target is never smaller than
    the visual rectangle; the per-surface migration story is allowed
    to outset further).
  - Each `CardSlotKind` variant resolves to a distinct
    `(outer_width_px, outer_height_px, z_layer)` triple (no two kinds
    accidentally collapse to the same geometry).
  - `card_slot_node(CardSlotKind::ShopSlot)` returns a Node whose
    `width` / `height` match `card_slot_geometry(...)` for the same
    kind (so the Node builder does not silently disagree with the
    geometry struct).

- Evidence artifacts under
  `production/qa/evidence/sprint-16-ui-card-slot-primitive/` (NEW;
  future `/dev-story` worker authors):
  - `card_slot_primitive_test` cargo test pass log.
  - `cargo check -p client` pass log.
  - Doc-review checklist enumerating each AC1..AC8 against the
    authored module + spec amendment.
  - Spec heading scan output confirming the new §12 presence.
  - Spec Adoption Matrix diff showing the row for
    `S12-TD-UI-CARD-SLOT-PRIMITIVE-001` updated with the new spec
    section reference.
  - `git diff` showing the shop slot migration applied and showing
    the other surfaces' call sites UNCHANGED.
  - QA snapshot bundle from a manual playable-client run at 1280 × 720
    and 1920 × 1080 (per `S15-QA-SNAPSHOT-DEFAULT-DEV` flow per
    PROMPT 1021 / 1023; the QA snapshot button defaults to enabled in
    dev builds) showing the shop slot rendered via the new primitive.
    The snapshot bundle MAY be saved as a single PNG plus an audit
    report under the evidence dir; exact format is worker-discretion.

#### Phases 2-4 follow-on migration sub-acceptance (deferred to Sprint 16+ siblings)

- **Phase 2 (hand surfaces)** -- migrate `HandFan` + `DraftGrid` call
  sites in `client/src/ui/hand/mod.rs` to consume
  `card_slot_node(CardSlotKind::HandFan)` and
  `card_slot_node(CardSlotKind::DraftGrid)`. Touches the hand fan
  re-spawn timing (existing `client/src/ui/hand/drag_state_visuals.rs`
  from Sprint 15 story `S12-UX-HAND-DRAG-STATE-VISUALS-001` DONE
  PROMPT 1009 must keep working). Owned by a Sprint 16+ follow-on
  story `S16-UI-CARD-SLOT-MIGRATION-HAND-001`.

- **Phase 3 (auction surfaces)** -- migrate `AuctionFeatured` call
  site in `client/src/ui/shop_auction/mod.rs` to consume
  `card_slot_node(CardSlotKind::AuctionFeatured)`. Touches the auction
  featured card differentiation contract from Sprint 14 PROMPT 931
  (must keep being strictly larger than the shop slot per
  `client/src/ui/shop_auction/mod.rs` doc comment at line 68: "larger
  than any shop slot well (`shop_slot_node` = 136 × 78 px)"). Owned
  by a Sprint 16+ follow-on story
  `S16-UI-CARD-SLOT-MIGRATION-AUCTION-001`.

- **Phase 4 (board staged-ghost surface)** -- the `BoardStagedGhost`
  variant is a world-space sprite, not a bevy_ui consumer; phase 4
  consumes the geometry struct to keep the ghost's screen-space
  preview sized consistently with the hand-fan card it was dragged
  from (per `docs/ux/board-rendering-spec.md` §4.5 ghost preview
  rules). Owned by a Sprint 16+ follow-on story
  `S16-UI-CARD-SLOT-MIGRATION-BOARD-GHOST-001`.

These three sub-phases are **explicitly listed here for traceability**
but are **not in this story's AC set**. Sprint 16 may bundle the
primitive + all four phases under one row OR split as a primitive
story (this row, phase 1) + three migration siblings. The default
scope encoded in this story file is the **primitive + spec + shop slot
phase 1 + evidence**.

### Out of Scope

- **No Sprint 16 activation** by this story.
- **No public release readiness** work.
- **No Standard-tier accessibility (`QA-COND-0005`) completion.** The
  card-slot primitive does NOT implement ≥44px hit-target enforcement,
  WCAG contrast checking, full keyboard-navigation focus order, screen
  reader hints, colorblind modes, or text scaling. The hit-target API
  returns the *current* hit rectangle per kind; it does not enforce a
  Standard-tier floor.
- **No final-art / asset-production** work (`PAW-TD-*-a`). The card
  slot composes the *layout*; it does not replace
  `assets/ui/shop_slot_well.png`, the draft-initial atlas frames, the
  hand-fan placeholder card, or the auction featured-card chrome. The
  underlying placeholder art remains `PAW-TD-002-a` / `PAW-TD-003-a`
  accepted-risk.
- **No playtest / fun-hypothesis validation** (`QA-COND-0006`).
- **No card-slot interaction states.** Hover / focus / pressed /
  disabled visual state mapping is owned by story 008
  (`S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001` DONE) -- this story's
  AC3 binds the card-slot kinds to the existing `interaction_states`
  tokens via a doc-comment cross-reference + integration-test
  presence assertion; per-surface migration of hover / focus / pressed
  / disabled is the Sprint 16+ `S16-UI-INTERACTION-STATE-MIGRATION-*`
  family (NOT this row).
- **No drag-state visuals re-author.** Hand fan drag-state visuals are
  owned by `S12-UX-HAND-DRAG-STATE-VISUALS-001` DONE Sprint 15
  PROMPT 1009; this row's `HandFan` kind preserves the existing
  drag-state contract (the primitive is the *source* surface the
  drag pulls from, not the drag ghost layer itself).
- **No board-rendering spec change.** `docs/ux/board-rendering-spec.md`
  (Sprint 15 PROMPT 1006 DONE) is the authority for board-cell
  geometry; this row's `BoardStagedGhost` kind reads cell geometry
  from that spec and only adds the screen-space ghost preview sizing
  contract.
- **No new color-palette tokens.** Card slot chrome reads from existing
  §7 palette tokens (`SURFACE_ELEVATED` for default chrome,
  `ACCENT` for featured-card chrome differentiation per Sprint 14
  PROMPT 931); no new base palette entries.
- **No nested cards.** The primitive forbids placing a card slot
  inside another card slot (AC2 contract). Composition that needs to
  render multiple cards (draft initial grid; shop slot row) does so
  by placing N siblings under a flex parent, not by nesting.
- **No HUD-timer urgency / overlay alpha / z-index / typography /
  flex-strip / viewport-invariant test** change (those are the
  existing Sprint 14 Tier 0 stories 002 / 003 / 004 / 005 / 006 + the
  Tier 2 future cosmetic captures bundle).
- **No `Polish->Release` gate-check retry** (PROMPT 761 FAIL preserved).
- **No stage advance** from `Polish` to `Release`.
- **No Sprint 14 / Sprint 15 row reopen.** All Sprint 14 16 closed
  rows + all Sprint 15 closed rows remain Done unchanged.

---

## Acceptance Criteria

All criteria are independently checkable BLOCKING criteria. They define
the future `/dev-story` worker's deliverable; this authoring run does
**not** verify any of them (no AC is `[x]` until `/dev-story` completes
and `/story-done` runs).

- [ ] **AC1 -- Authoritative primitive module + token usage**: GIVEN the
  story commit, WHEN the new module file is inspected, THEN
  `client/src/ui/design_tokens/card_slot.rs` exists, is declared from
  `client/src/ui/design_tokens/mod.rs`, and exports the
  `CardSlotKind` enum (with the five variants `HandFan`, `DraftGrid`,
  `ShopSlot`, `AuctionFeatured`, `BoardStagedGhost`), the
  `CardSlotGeometry` struct, and the four named accessor functions
  (`card_slot_geometry`, `card_slot_node`, `card_slot_image_inset`,
  `card_slot_text_inset`, `card_slot_hit_target`). Every numeric
  value inside the module is named (`const NAME: f32 = ...;` or
  `pub const NAME: UiRect = ...;`); no inline magic literal is
  introduced at a public-API boundary. Each public item has a `///`
  doc comment that names its consumer surface and forward-references
  the new spec §12. Verification: module read + module re-export +
  doc-comment scan + grep for `Val::Px(\d` inside the module body
  catches any naked literal.

- [ ] **AC2 -- No nested cards, no layout shift, stable aspect ratio**:
  GIVEN the primitive module + the new integration test, WHEN the
  test runs, THEN it asserts that:
  - For each `CardSlotKind` variant, `outer_width_px / outer_height_px`
    falls inside `aspect_ratio_band` (declared `(min, max)` tuple per
    kind).
  - The Node returned by `card_slot_node(kind)` for each kind has a
    `width` and `height` that match `card_slot_geometry(kind)` for the
    same kind (no silent divergence between the geometry struct and
    the Node builder).
  - No `CardSlotKind` variant's geometry includes another
    `CardSlotKind`'s geometry as a nested child (the contract is
    "leaf-only"; a slot has image + text regions, NOT a child slot).
    Verification is structural: the `card_slot_node` builder for kind
    K MUST NOT instantiate `card_slot_node(_)` for any other kind K'.
  - The aspect ratio is preserved across the six canonical viewports
    (`tests/integration/helpers/ui_viewport.rs::CANONICAL_VIEWPORTS`):
    rendering the same Node at each viewport must keep
    `outer_width_px / outer_height_px` constant (the slot is
    pixel-fixed per §4 spec spacing scale; no viewport-driven scaling
    is introduced).
  Verification: integration-test assertions (AC8) + module read.

- [ ] **AC3 -- Hover / focus / pressed / disabled state mapping via
  existing interaction primitives**: GIVEN the primitive module, WHEN
  inspected, THEN each `CardSlotKind` carries a doc-comment forward
  reference to the four token sets published by
  `client/src/ui/design_tokens/interaction_states.rs` (Sprint 15
  story 008 DONE). The doc comment names which interaction-state
  primitives compose with that card-slot kind (e.g. `ShopSlot` consumes
  `HOVER_BG_TINT_ALPHA` / `HOVER_BORDER_ALPHA` for pointer hover;
  `FOCUS_RING_*` for Tab focus; `PRESSED_BG_TINT_ALPHA` for mouse-down
  while affordable; `DISABLED_BG_TINT_ALPHA` /
  `DISABLED_TEXT_ALPHA` / `DISABLED_BORDER_ALPHA` for the "cannot
  afford" state). The integration test asserts that the interaction
  state tokens are importable from the published path and that the
  card-slot module's doc comments name each of the four primitive
  set families. **Per-surface migration of interaction-state visuals
  is OUT OF SCOPE for this story** (that is the Sprint 16+
  `S16-UI-INTERACTION-STATE-MIGRATION-*` family); the contract here is
  that the card-slot kinds reference the right primitive families,
  not that every clickable surface is fully wired. Verification:
  doc-comment scan + integration-test import assertion.

- [ ] **AC4 -- Image / text containment at 1280 × 720 and a smaller
  viewport**: GIVEN the integration test, WHEN run, THEN it asserts
  that for each `CardSlotKind` the image inset rectangle and text
  inset rectangle each fit inside the outer rectangle (per AC2's
  per-kind containment check) AND that the same containment holds at
  the smallest canonical viewport (`1280 × 720` per §8 of the global
  UI spec; this is the friend-game smallest viewport in the canonical
  matrix) and at one smaller-than-canonical sentinel viewport
  (`1024 × 600`; chosen as a worker sentinel below the canonical
  matrix to prove that the slot's pixel-fixed sizing does NOT shift
  even when the viewport drops below the canonical floor). The
  containment assertion must hold without truncation (no negative
  width / height; no image or text region extending past the slot's
  outer rectangle). Verification: integration-test viewport-iteration
  loop + per-kind containment assertion.

- [ ] **AC5 -- Per-surface migration boundaries split into phases**:
  GIVEN the story file + the future `/dev-story` worker's commit, WHEN
  the diff is inspected, THEN exactly ONE existing card-surface call
  site is migrated to the new primitive (default canonical: shop slot
  per phase 1 above). The other three call-site families (hand fan +
  draft initial grid; auction featured card; board staged ghost) are
  EXPLICITLY UNCHANGED with respect to their card-slot geometry. The
  worker report enumerates each unmigrated surface, names the
  Sprint 16+ follow-on row that owns its migration, and confirms the
  `card_slot_node` API surface is sufficient for that future migration
  (i.e., the relevant `CardSlotKind` variant exists with the right
  `outer_width_px` / `outer_height_px` / `aspect_ratio_band`). If the
  Sprint 16 producer decides to bundle multiple migration phases into
  this story at activation time, the producer MUST amend AC5 at
  activation to expand the migrated surface set. The story file as
  authored encodes only the shop slot phase 1 migration. Verification:
  `git diff origin/main...HEAD --stat -- 'client/src/ui/hand/mod.rs'
  'client/src/ui/shop_auction/mod.rs'` shows changes only to the
  shop slot call site in `shop_auction/mod.rs`, and no change to
  `hand/mod.rs` or to the auction featured-card call site or to the
  board ghost preview path under `client/src/presentation/`.

- [ ] **AC6 -- Visual evidence / screenshot harness expectations**:
  GIVEN the future `/dev-story` worker's evidence directory at
  `production/qa/evidence/sprint-16-ui-card-slot-primitive/` (NEW),
  THEN the directory contains at minimum:
  - A QA snapshot bundle (per the existing
    `S15-QA-SNAPSHOT-DEFAULT-DEV` flow per PROMPT 1021 / 1023; the QA
    snapshot overlay button is enabled by default in dev builds via
    `CCGS_QA_SNAPSHOT=1`) captured from a manual playable-client run
    at `1280 × 720` showing the shop panel composed via the migrated
    primitive. The snapshot bundle MAY include a single screenshot PNG
    + a feedback / audit log; exact format is worker-discretion within
    the existing QA snapshot harness.
  - The same QA snapshot bundle at `1920 × 1080`.
  - A doc-review checklist enumerating each AC1..AC8 against the
    authored artifacts.
  - The integration-test pass log per AC8.
  - The `git diff` for the migrated shop slot call site, with the
    other surfaces' call sites verified UNCHANGED via the
    `git diff --stat` assertion in AC5.
  No new "screenshot harness" infrastructure is authored by this row;
  the existing QA snapshot capture flow is the harness. Verification:
  evidence-directory file presence + integration-test pass log + spec
  cross-reference.

- [ ] **AC7 -- Tests expected, including viewport-invariant /
  layout-contract test**: GIVEN the story commit, WHEN the new
  integration test bin at
  `tests/integration/ui_clean_pass/card_slot_primitive_test.rs` (NEW)
  is run, THEN it passes and asserts at minimum:
  - All five `CardSlotKind` variants are importable from the module's
    public path.
  - Each kind's `outer_width_px / outer_height_px` falls in
    `aspect_ratio_band` (AC2).
  - Image inset + text inset fit inside the outer rectangle and do
    not mutually overlap (AC4).
  - Hit-target inset is a superset of (or equal to) the visual outer
    rectangle.
  - Each kind resolves to a distinct `(outer_width_px,
    outer_height_px, z_layer)` triple (no two kinds collapse).
  - `card_slot_node(CardSlotKind::ShopSlot)` returns a Node whose
    `width` / `height` match `card_slot_geometry(ShopSlot)`.
  - Aspect ratio is preserved across `CANONICAL_VIEWPORTS` from
    `tests/integration/helpers/ui_viewport.rs` (the viewport-invariant
    layout-contract test).
  - The four interaction-state token families
    (`HOVER_*`, `FOCUS_*`, `PRESSED_*`, `DISABLED_*` per story 008)
    are importable from the published path (AC3 cross-reference).
  - The migrated shop-slot Node has the expected outer
    width / height matching `CardSlotKind::ShopSlot` (so the
    primitive + the call site agree).
  The integration test bin is registered in `client/Cargo.toml`
  consistent with the existing Sprint 14 / Sprint 15 Tier 0
  integration-test pattern (see e.g.
  `tests/integration/ui_clean_pass/interaction_state_primitives_test.rs`
  registration from Sprint 15 PROMPT 1005). Verification: `cargo
  test` of the new bin passes.

- [ ] **AC8 -- Non-claims (no gameplay / no server / no release / no
  final-art)**: GIVEN the story commit, WHEN the closure paperwork is
  inspected, THEN this row does NOT:
  - Modify any gameplay logic (no change under
    `client/src/gameplay/`, `server/src/`, or
    `shared/src/` except trivial re-export wiring on the client
    boundary).
  - Modify any server / shared / protocol module
    (`server/src/**`, `shared/src/**` UNCHANGED).
  - Claim public release readiness, release-candidate readiness,
    full game completion, full playable-client manual QA, playtest /
    fun-hypothesis validation (`QA-COND-0006`), Standard-tier
    accessibility (`QA-COND-0005`), final-art / asset-production
    completion (`PAW-TD-*-a`), two-client GAME_OVER closure
    (`S8-QA-001-W1`), the `Polish->Release` gate-check retry, stage
    advance from `Polish` to `Release`, or the underlying drag-runtime
    bug fix from Sprint 12 story 019.
  - Reopen any Sprint 14 / Sprint 15 closed row.
  Verification: `git diff origin/main...HEAD --stat -- 'server/src/'
  'shared/src/'` is empty; paperwork review of the `/story-done`
  close-out section confirms the non-claims; `git diff` of
  `production/sprint-status.yaml` shows no accept-risk disposition
  flip.

---

## Implementation Notes

### Owned files (the future `/dev-story` worker authors these)

| Path | Expected change |
|------|-----------------|
| `client/src/ui/design_tokens/card_slot.rs` (NEW) | Author the `CardSlotKind` enum (5 variants), the `CardSlotGeometry` struct, and the named accessor functions (`card_slot_geometry`, `card_slot_node`, `card_slot_image_inset`, `card_slot_text_inset`, `card_slot_hit_target`) with doc comments and named numeric constants. |
| `client/src/ui/design_tokens/mod.rs` | Declare `pub mod card_slot;` alongside the existing Sprint 14 / 15 Tier 0 modules (z_layers / typography / spacing / strips / overlays / interaction_states). |
| `client/src/ui/shop_auction/mod.rs` | **Phase 1 migration only.** Replace the inline `Node { width: Val::Px(136.0), height: Val::Px(78.0), ... }` literal in `shop_slot_node` at ~line 4642 with a call to `card_slot_node(CardSlotKind::ShopSlot)`. Optionally retire the inline numeric literals; keep the helper function signature `fn shop_slot_node(index: usize) -> Node` so existing call sites do not need a sweep. **No change** to `AUCTION_FEATURED_CARD_*` constants, `auction_featured_card_node`, or any other card-related call site in this file. |
| `docs/ux/global-ui-design-spec.md` | Amend: new §12 spec section enumerating the five `CardSlotKind` variants + canonical defaults + image / text / hit-target insets + z-layer references; flip the §10 "Card slot composition" stub to a forward reference; update the Spec Adoption Matrix row for `S12-TD-UI-CARD-SLOT-PRIMITIVE-001`. |
| `tests/integration/ui_clean_pass/card_slot_primitive_test.rs` (NEW; suggested filename) | Integration test asserting primitive module shape + per-kind aspect-ratio + image / text containment at canonical viewports + hit-target superset + interaction-state token presence + migrated shop-slot Node sanity (AC1-AC8). |
| `client/Cargo.toml` | Register the new `[[test]]` bin if the project pattern requires explicit registration, mirroring the Sprint 14 / Sprint 15 Tier 0 integration-test bins (e.g. `ui_clean_pass_interaction_state_primitives_test` from PROMPT 1005). |
| `production/qa/evidence/sprint-16-ui-card-slot-primitive/` (NEW) | Evidence dir per AC6: QA snapshot bundles at `1280 × 720` and `1920 × 1080`, integration-test pass log, doc-review checklist, `git diff` proving phase-1-only migration, spec heading scan output. |

This table is a planning estimate. The implementation prompt is
authoritative for the realised set.

### Forbidden files (the future `/dev-story` worker MUST NOT touch these)

- `client/src/ui/hand/mod.rs` -- phase 2 migration (hand fan + draft
  grid) is OUT OF SCOPE; owned by Sprint 16+
  `S16-UI-CARD-SLOT-MIGRATION-HAND-001`.
- `client/src/ui/hand/drag_state_visuals.rs` -- owned by Sprint 15
  story `S12-UX-HAND-DRAG-STATE-VISUALS-001` DONE; no change here.
- `client/src/ui/shop_auction/mod.rs` -- ONLY the `shop_slot_node`
  helper changes per phase 1; the `auction_featured_card_node` helper
  and all other card-related call sites are UNCHANGED.
- `client/src/presentation/board_rendering.rs` -- board staged-ghost
  geometry is owned by `docs/ux/board-rendering-spec.md` (Sprint 15
  PROMPT 1006 DONE); phase 4 migration is OUT OF SCOPE.
- `client/src/presentation/` other modules -- the `BoardStagedGhost`
  variant exists in the primitive enum but no call site under
  `presentation/` is migrated by this row.
- `client/src/ui/design_tokens/{z_layers,typography,spacing,strips,overlays,interaction_states}.rs`
  -- those modules are read-only inputs for this story; no edit.
- `client/src/ui/hud/mod.rs`, `client/src/ui/settings/mod.rs`,
  `client/src/ui/lobby.rs` -- not card-slot consumers; no change.
- `server/src/**`, `shared/src/**` -- UNCHANGED.
- `tests/integration/ui_clean_pass/{z_layers,typography,strips,
  overlay_alpha,interaction_state_primitives,hud_top_strip_layout,
  hud_bottom_strip_layout}_test.rs` -- existing Tier 0 / Tier 1 tests
  are UNCHANGED; only the new `card_slot_primitive_test.rs` is added.
- `tests/integration/helpers/ui_viewport.rs` -- read-only input; the
  `CANONICAL_VIEWPORTS` set is consumed, not modified.
- `tests/integration/fixtures/ui_viewport_baseline.rs` -- read-only
  input.
- `production/sprint-status.yaml` -- only touched by activation /
  `/story-done` paperwork, not by `/dev-story`.
- `production/sprints/sprint-15.md`, `production/sprints/sprint-16.md`,
  any earlier sprint plan -- not modified by `/dev-story`.
- `production/stage.txt` (`Polish`; not advanced).
- `production/session-state/**`.
- `production/qa/qa-plan-sprint-15.md` or a future
  `production/qa/qa-plan-sprint-16.md` -- not modified by `/dev-story`
  for this row.
- `production/qa/evidence/*` outside the dedicated evidence directory
  `production/qa/evidence/sprint-16-ui-card-slot-primitive/` (NEW).
- `docs/ux/board-rendering-spec.md` -- read-only input; the
  `BoardStagedGhost` variant references it but does not amend it.
- `docs/ux/ui-clean-pass-roadmap.md` -- read-only input.

### Module integration touch points

- `client/src/ui/design_tokens/mod.rs` is the existing aggregator; the
  new `pub mod card_slot;` declaration is **additive** to the six
  existing siblings (z_layers / typography / spacing / strips /
  overlays / interaction_states). No file collision is expected with
  any active Sprint 15 row because Sprint 15 implementation rows are
  closed Done as of `origin/main@7b663df`.
- `client/src/ui/design_tokens/z_layers.rs` constants (`UI_BASE`,
  `UI_OVERLAY`) are read into the `CardSlotGeometry::z_layer` field
  by value. No edit to `z_layers.rs` is required.
- `client/src/ui/design_tokens/interaction_states.rs` is referenced
  by doc comment + integration-test import only; no edit to
  `interaction_states.rs` is required.
- `tests/integration/ui_clean_pass/` is the existing test path; the
  new bin is registered alongside the seven existing bins.

---

## Parallelization and Phase Breakdown

This is a deliberately-scoped story. The Sprint 16 producer has two
shapes available; this story file encodes the **single-row + three
follow-on siblings** shape as the default. The alternative **single
bundled row** shape is available if Sprint 16 producer prefers and
amends AC5 at activation.

### Default shape: 1 story (this row) + 3 follow-on migration rows

| Row | Slug | Scope | Owns files |
|-----|------|-------|------------|
| **This story (S12-TD-UI-CARD-SLOT-PRIMITIVE-001)** | as above | Primitive module + spec amendment + shop slot phase 1 migration + evidence + viewport-invariant test | `client/src/ui/design_tokens/card_slot.rs` (NEW), `client/src/ui/design_tokens/mod.rs`, `docs/ux/global-ui-design-spec.md`, `tests/integration/ui_clean_pass/card_slot_primitive_test.rs` (NEW), `client/Cargo.toml`, `client/src/ui/shop_auction/mod.rs` (shop_slot_node only), `production/qa/evidence/sprint-16-ui-card-slot-primitive/` (NEW) |
| **S16-UI-CARD-SLOT-MIGRATION-HAND-001** | net-new | Phase 2: migrate `HandFan` + `DraftGrid` call sites in `client/src/ui/hand/mod.rs` to consume the primitive | `client/src/ui/hand/mod.rs`, `tests/integration/hand_ui/` evidence + bumper |
| **S16-UI-CARD-SLOT-MIGRATION-AUCTION-001** | net-new | Phase 3: migrate `AuctionFeatured` call site in `client/src/ui/shop_auction/mod.rs` to consume the primitive | `client/src/ui/shop_auction/mod.rs` (auction_featured_card_node only) |
| **S16-UI-CARD-SLOT-MIGRATION-BOARD-GHOST-001** | net-new | Phase 4: consume the `BoardStagedGhost` geometry in the board ghost preview path | `client/src/presentation/board_rendering.rs` (or whichever module owns the staged-ghost screen-space preview) |

### Parallel-safety after this primitive row lands

If Sprint 16 adopts the default shape, the three migration siblings
are **pairwise file-disjoint**:

| Row | File scope | Parallel-safe with |
|-----|------------|---------------------|
| `S16-UI-CARD-SLOT-MIGRATION-HAND-001` | `client/src/ui/hand/mod.rs` + `tests/integration/hand_ui/` evidence | Auction (different file: shop_auction/mod.rs vs hand/mod.rs); Board-ghost (different file: presentation/board_rendering.rs vs hand/mod.rs) |
| `S16-UI-CARD-SLOT-MIGRATION-AUCTION-001` | `client/src/ui/shop_auction/mod.rs` (auction_featured_card_node only; shop_slot_node already migrated by primitive row) | Hand (different surface), Board-ghost (different file) |
| `S16-UI-CARD-SLOT-MIGRATION-BOARD-GHOST-001` | `client/src/presentation/board_rendering.rs` + ghost screen-space preview path | Hand (different file), Auction (different file) |

All three depend on the primitive module landing first. Once the
primitive is on `origin/main`, the three migration siblings can run as
a parallel batch under the Sprint 16 "Suggested First Parallel Batch"
guidance.

### Files that MUST serialize

- `client/src/ui/design_tokens/card_slot.rs` is owned exclusively by
  this primitive row. The three migration siblings READ this module
  but MUST NOT edit it. If a migration sibling discovers a missing
  variant or geometry field, it MUST open a follow-on amendment to
  this primitive row rather than edit `card_slot.rs` inline.
- `client/src/ui/design_tokens/mod.rs` is the shared aggregator; only
  this primitive row touches it (to declare `pub mod card_slot;`).
- `docs/ux/global-ui-design-spec.md` is shared with the existing UI
  clean-pass milestone; only this primitive row authors the new §12
  spec section. Migration siblings cross-reference §12 in their
  evidence docs but do NOT amend the spec body.
- `tests/integration/ui_clean_pass/card_slot_primitive_test.rs` is
  owned exclusively by this primitive row. Migration siblings author
  per-surface integration tests under their respective
  `tests/integration/hand_ui/`, `tests/integration/shop_auction/`, or
  `tests/integration/board_rendering/` paths (or extend an existing
  bin), NOT this primitive's test bin.
- `production/sprint-status.yaml` is a shared-status writer per
  `.claude/docs/coordination-rules.md` -- the orchestrator serializes
  all `/story-done` updates against Sprint 16 row status.

### Alternative: single bundled story

If the Sprint 16 producer prefers one bundled story over the
primitive + three siblings, the producer MUST amend AC5 at activation
to expand "exactly ONE existing card-surface call site" to "all four
card-surface call site families", and MUST extend the integration
test bin to cover the migrated hand fan / draft grid / auction
featured / board ghost surfaces. The estimated effort under the
bundled shape is ~3.0d -- larger than the original
`docs/ux/ui-clean-pass-roadmap.md` rank 13 ~1.5d estimate, because
the original estimate did not anticipate the full per-surface
migration. The split shape (primitive ~1.5d + three follow-on rows
at ~0.5-1.0d each) preserves the original rank 13 envelope for the
primitive and explicitly costs the migration follow-ons.

---

## QA Evidence Expectations

The future `/dev-story` worker authors evidence under
`production/qa/evidence/sprint-16-ui-card-slot-primitive/` (NEW). The
evidence is designed to consume future QA snapshot bundles (per the
`S15-QA-SNAPSHOT-DEFAULT-DEV` flow integrated via PROMPT 1023
`origin/main@7b663df`) without re-authoring snapshot infrastructure.

### Expected evidence files

| File | Purpose | Format |
|------|---------|--------|
| `evidence.md` | Doc-review checklist enumerating AC1..AC8 against the authored artifacts. Each AC checkbox is `[x]` only if the worker has independently verified it. | Markdown |
| `cargo-test-card-slot-primitive.log` | Raw `cargo test --test ui_clean_pass_card_slot_primitive_test` (or equivalent bin name) output. Must show PASS for every assertion. | Plain text |
| `cargo-check-client.log` | Raw `cargo check -p client` output proving the module + migration compile. | Plain text |
| `git-diff-stat-disjoint-surfaces.txt` | Output of `git diff origin/main...HEAD --stat -- 'client/src/ui/hand/mod.rs' 'client/src/ui/shop_auction/mod.rs' 'client/src/presentation/'` proving only the shop_slot_node call site changed (no hand/mod.rs change; no presentation/ change; no auction_featured_card_node change). | Plain text |
| `spec-heading-scan.txt` | Output of `grep "^## §" docs/ux/global-ui-design-spec.md` showing the new §12 "Card Slot Primitive" section is present. | Plain text |
| `spec-adoption-matrix-diff.md` | `git diff docs/ux/global-ui-design-spec.md` excerpt showing the Spec Adoption Matrix row for `S12-TD-UI-CARD-SLOT-PRIMITIVE-001` updated. | Markdown |
| `qa-snapshot-1280x720/` | QA snapshot bundle from a manual playable-client run at `1280 × 720` showing the migrated shop slot composed via the new primitive. Captured via the `S15-QA-SNAPSHOT-DEFAULT-DEV` overlay button per PROMPT 1021 / 1023 (defaults to enabled in dev builds via `CCGS_QA_SNAPSHOT=1`). Bundle MAY include screenshot PNG, feedback text, and audit log per the existing QA snapshot flow's output shape. | Per `S15-QA-SNAPSHOT-DEFAULT-DEV` bundle format (worker-discretion within that contract) |
| `qa-snapshot-1920x1080/` | Same as above at `1920 × 1080` (canonical 1080p viewport from §8 of the global UI spec). | Same |
| `qa-snapshot-1024x600-optional/` | OPTIONAL: snapshot at `1024 × 600` (smaller-than-canonical sentinel from AC4) if the playable-client launcher supports that viewport. If not, document the limitation in `evidence.md`. | Same / optional |

### QA snapshot harness compatibility

The QA snapshot capture flow per PROMPT 1019 / 1020 / 1021 / 1023 is
**already in place on `origin/main`** as of `7b663df`:

- `CCGS_QA_SNAPSHOT=1` defaults to enabled in dev builds (PROMPT 1021
  / 1023).
- `F9` shortcut triggers an in-game snapshot per PROMPT 1019.
- Snapshot bundle is auto-captured per the
  `client/src/presentation/qa_snapshot.rs` module (recently updated by
  PROMPT 1023 integration tip `7b663df`).

This story's evidence dir consumes those bundles **without** authoring
any new QA snapshot infrastructure. If the worker discovers the QA
snapshot harness is not capturing the shop slot at expected viewports,
the worker MUST flag this as a blocker in the worker report and STOP;
this story does NOT author or extend the QA snapshot harness.

### Cross-link to `production/qa/qa-plan-sprint-16.md`

If `production/qa/qa-plan-sprint-16.md` exists at `/dev-story` time,
this story's evidence dir is referenced from the QA plan's
"S12-TD-UI-CARD-SLOT-PRIMITIVE-001" row. The QA plan authoring is a
separate prompt (`/qa-plan sprint-16` after Sprint 16 activation);
this story does NOT author the QA plan. If the QA plan does not exist
yet at `/dev-story` time, the evidence dir stands alone and is
referenced from the `/story-done` paperwork directly.

---

## Verification

- File presence check on
  `client/src/ui/design_tokens/card_slot.rs`.
- Module declaration check on
  `client/src/ui/design_tokens/mod.rs`.
- `cargo test --test ui_clean_pass_card_slot_primitive_test` (or
  the project-idiomatic bin name) passes -- AC7.
- `cargo check -p client` passes -- proves the shop slot migration
  compiles.
- `git diff origin/main...HEAD --stat -- 'client/src/ui/hand/mod.rs'
  'client/src/presentation/'` is empty -- AC5 + AC8.
- `git diff origin/main...HEAD --stat -- 'client/src/ui/shop_auction/mod.rs'`
  shows ONLY the `shop_slot_node` helper changed (no
  `auction_featured_card_node` change).
- Spec heading scan (`grep "^## §" docs/ux/global-ui-design-spec.md`
  or project-idiomatic equivalent) shows the new §12 "Card Slot
  Primitive" section -- AC1 + AC2 cross-reference.
- Spec Adoption Matrix diff shows the row for
  `S12-TD-UI-CARD-SLOT-PRIMITIVE-001` updated.
- `git diff` of `production/sprint-status.yaml` empty across the
  worker tip and integration merge (only the activation paperwork
  + `/story-done` paperwork edits `sprint-status.yaml`).
- QA snapshot bundles at `1280 × 720` and `1920 × 1080` present
  under `production/qa/evidence/sprint-16-ui-card-slot-primitive/`
  -- AC6.

---

## Dependencies / Sequencing

- **Authoring prompt (this PROMPT 1025)** is the *story-authoring*
  prompt; it creates the story file only. No `/dev-story` runs here.
- **Activation**: Requires Sprint 16 activation (separate prompt; not
  this one). Cannot land in Sprint 15 (Sprint 15 deferred this row
  explicitly per `production/sprints/sprint-15.md` "Wider Sprint 15
  Backlog -- Deliberately deferred to Sprint 16+").
- **Tier 0 host-module sequencing** (per
  `docs/ux/ui-clean-pass-roadmap.md` §3 "Sequencing Rules"): the host
  module `client/src/ui/design_tokens/` is already populated by the
  Sprint 14 / Sprint 15 DONE Tier 0 stories (002 z_layers / 003
  typography / 004 spacing + strips / 006 overlays / 008
  interaction_states). This row is **additive** to the host module:
  it adds a new sibling submodule `card_slot.rs` alongside the six
  existing ones. No file collision is expected.
- **Spec dependency**: the canonical global UI design spec at
  `docs/ux/global-ui-design-spec.md` is DONE on Sprint 14 PROMPT 922,
  with story 008 (Sprint 15 PROMPT 1009) authoring §11 "Interaction
  State Primitives". The spec amendment this row authors is a forward
  reference flip on the existing §10 "Card slot composition" stub
  ("Owned by Tier 3 story 13 ... This spec does not bind a card slot
  composition; story 13 authors the primitive after Tier 1 surfaces
  stabilise.") -- replaced with a forward reference to the new §12.
- **Roadmap rank 13 dependencies satisfied**: per
  `docs/ux/ui-clean-pass-roadmap.md` rank 13 "depends on ranks 1, 2,
  3, 6 + at least one Tier 1 surface stable":
  - Rank 1 (`S11-TD-UI-ZINDEX-LAYERS`) DONE Sprint 14 PROMPT 903.
  - Rank 2 (`S11-TD-UI-FONT-CONSTANTS`) DONE Sprint 14 PROMPT 908.
  - Rank 3 (`S11-TD-UI-FLEX-STRIPS`) DONE Sprint 14 PROMPT 919.
  - Rank 6 (`S12-UX-GLOBAL-UI-DESIGN-SPEC-001`) DONE Sprint 14
    PROMPT 922.
  - Tier 1 stable: HUD top strip (rank 7) DONE PROMPT 942, auction
    featured (rank 10) DONE PROMPT 931, draft grid centered modal
    (rank 9) DONE PROMPT 953, lobby modal (rank 12) DONE PROMPT 939.
  All dependencies satisfied at authoring time
  (`origin/main@7b663df`).
- **File-disjoint with any active Sprint 15 row**: Sprint 15 is in
  closeout (dev rows complete per the active session state; remaining
  Sprint 15 row is human QA / evidence per the task brief). The
  Sprint 15 row that touches `client/src/ui/design_tokens/` (story
  008 `interaction_states`) is DONE PROMPT 1009. No file collision
  with active Sprint 15 work is expected.
- **No producer-decision blocker.** PROMPT 802 §9 producer-decisions
  1 through 5 are all RESOLVED or not-applicable to this row. The
  card-slot geometry values in this story (`HandFan` 96 × 136 px,
  `DraftGrid` 120 × 56 px, `ShopSlot` 136 × 78 px, `AuctionFeatured`
  380 × 280 px, `BoardStagedGhost` cell-sized) are read verbatim
  from the currently-shipped per-surface literals on
  `origin/main@7b663df`. The Sprint 16 producer MAY re-tune the
  values at activation time as a separate decision; this story
  defaults to "preserve current per-surface values" so phase 1 is a
  no-visual-regression migration.

---

## Notes

- PROMPT 802 §3.3 HA1 + §3.3 HA5: no canonical card-slot primitive;
  layout drift across hand + draft + shop + auction surfaces.
- PROMPT 802 §4 Tier 3 rank 13: net-new, Should, ~1.5d, refactor
  touches hand + shop + auction together (the breadth that justified
  the Sprint 15 deferral).
- PROMPT 802 §8 sequencing dependency: rank 13 must wait for at least
  one Tier 1 surface to stabilise; all six Tier 1 Must rows are now
  DONE on Sprint 14, plus the Sprint 15 Tier 0 adjacent rank
  `interaction_states` DONE PROMPT 1009. Authoring at `origin/main@7b663df`
  is unblocked.
- `docs/ux/ui-clean-pass-roadmap.md` rank 13 "Sequencing dependency":
  satisfied at story-authoring time.
- `production/sprints/sprint-15.md` "Wider Sprint 15 Backlog ...
  Deliberately deferred to Sprint 16+ (size or coordination
  overhead)": this story is the Sprint 16 author target.
- `docs/ux/global-ui-design-spec.md` §10 "Card slot composition" stub
  is the deferral point this story flips to a forward reference.
- Per Sprint 16 plan draft (`production/sprints/sprint-16.md`
  `sprint-plan/sprint-16-draft`, NOT activated): this row is a
  Sprint 16 candidate; activation is a separate prompt.
- Accept-risk preservation: `PAW-TD-*-a`, `QA-COND-0005`,
  `QA-COND-0006`, `S8-QA-001-W1`, `TQ-S12-C7` preserved unchanged.
  This story does not advance any of them.
- The `BoardStagedGhost` variant is a *bridge* variant -- the
  enum carries it so the primitive is the single source of truth for
  card-slot geometry across both bevy_ui and world-space surfaces.
  Phase 4 (`S16-UI-CARD-SLOT-MIGRATION-BOARD-GHOST-001`, future) is
  responsible for actually consuming the variant; this row only
  declares it in the enum + populates its geometry struct.

---

## Closure Trail

| PROMPT | Action | Commit / Reference |
|--------|--------|---------------------|
| 1025 | Authored story file (Sprint 16 candidate; NOT activated) on branch `story/s16-ui-card-slot-primitive` from base `origin/main@7b663df75e63a4e46512c5d88e0de2aa704a114a` (PROMPT 1023 `integrate(s15): default QA snapshot enabled in dev builds`). EPIC index updated to include this row as story 009. | `production/epics/ui-clean-pass/story-009-ui-card-slot-primitive.md` NEW + `production/epics/ui-clean-pass/EPIC.md` MODIFIED |

Subsequent prompts (integration, `/story-readiness` against Sprint 16
activation HEAD, Sprint 16 activation, `/qa-plan sprint-16`,
`/dev-story` for this row (and for the 3 Sprint 16+ follow-on
migration siblings if Sprint 16 producer adopts the split shape),
integration of the worker, `/story-done`) are TBD and will be
appended to this Closure Trail as they land.
