# Story 018: S17-UI-CARD-SLOT-INSET-WIRING-001 -- `card_slot_node(kind)` Image / Text Inset + GlobalZIndex Wiring

> **Epic**: UI Clean-Pass
> **Story ID**: S17-UI-CARD-SLOT-INSET-WIRING-001
> **Status**: Done -- closed by PROMPT 1110 on `origin/main@30f166f` (Sprint 17 Should Have). Worker PROMPT 1102 (`55c0dab`), integration PROMPT 1106 (`30f166f`).
> **Layer**: Presentation -- card-slot primitive (`client/src/ui/design_tokens/card_slot.rs`)
> **Type**: Tech Debt -- primitive ratification (no consumer-surface migration in this row)
> **Sprint**: Sprint 17 Should Have row per `production/sprints/sprint-17.md` §"Should Have". Activated by PROMPT 1099; closed by PROMPT 1110.
> **Authored**: 2026-05-18 by PROMPT 1095
> **Authoring source-of-truth**: `origin/main@7d36191fe94adf99d3448a58185d8079d828c29e`
> (`integrate(s17): merge Sprint 17 plan draft into main (PROMPT 1093 paperwork-only)`)
> **Closure source-of-truth**: `origin/main@30f166fb9b718bdb5a6a904da0d66cdcc9685f15` (PROMPT 1106 integration tip)
> **Estimated effort**: ~0.25d (single primitive extension; no consumer-surface migration)
> **Source audit**: PROMPT 1077 `reports/PROMPT-1077-ui-state-source-consistency-deep-audit.md` §"Per-finding evidence" SOURCE-1077-06 (P1)

---

## Status / No-Claim Banner

This story is a Sprint 17 Should Have **candidate** authored by PROMPT
1095. **No sprint is activated by this authoring run.** PROMPT 1095
does NOT modify `production/sprint-status.yaml`,
`production/sprints/sprint-17.md`, `production/sprints/sprint-16.md`,
`production/stage.txt`, any `production/session-state/*` file, any
QA-plan / smoke / Team-QA / gate-check / release-check artifact under
`production/qa/`, any code under `client/`, `server/`, `shared/`,
`tests/`, `Cargo.toml`, `Cargo.lock`, `.cargo/`, `.github/`, or
`Trunk.toml`. PROMPT 1095 does NOT run `/story-readiness`,
`/dev-story`, `/story-done`, `/smoke-check`, `/team-qa`,
`/gate-check`, `/release-check`, `/qa-plan`, `cargo`, `trunk`, or
any CI command.

This story does **not** claim: public release readiness, release-
candidate readiness, full game completion, broad / Standard-tier
accessibility completion (`QA-COND-0005`), Standard-tier hit-target
conformance (>=44 px), playtest / fun-hypothesis validation
(`QA-COND-0006`), full playable-client manual QA, two-client GAME_OVER
closure (`S8-QA-001-W1`), final-art / asset-production completion
(`PAW-TD-*-a`), `Polish->Release` gate-check retry, stage advance from
Polish to Release, closure of the Sprint 12 story 019 underlying
drag-runtime bug, closure of `S11-HUD-TIMER-EYEBALL-VISUAL-001`,
closure of any of the 24 PROMPT 1022 audit findings, closure of any
SOURCE-1077-* finding outside SOURCE-1077-06, closure of any
AUDIT-1076-* finding, or **per-surface migration of any consumer
surface** (HAND / DRAFT-GRID / AUCTION-FEATURED / BOARD-GHOST — those
remain Sprint 17+ Backlog candidates under the family
`S17-UI-CARD-SLOT-MIGRATION-*` per
`production/sprints/sprint-17.md` §"Wider Sprint 17 Backlog").

**No optimistic client-side authority is introduced or proposed.** No
protocol shape change. No new server-authoritative state. No new C2S /
S2C message.

Sprint 16 disposition `closed-with-conditions` per PROMPT 1082 +
PROMPT 1088 preserved unchanged. Sprint 15 / 14 / 13 / 12 / 11 / 10
dispositions preserved unchanged. PROMPT 761 Polish->Release gate-
check `FAIL` evidence preserved. `PAW-TD-*-a`, `QA-COND-0005`,
`QA-COND-0006`, `TQ-S12-C1..C7` preserved verbatim.

---

## Source Finding

### SOURCE-1077-06 (P1) — `card_slot_node` ships outer rectangle only

- **Audit location**: `reports/PROMPT-1077-ui-state-source-consistency-deep-audit.md`
  §"Per-finding evidence" SOURCE-1077-06 (P1).
- **Affected file lines at audit time**:
  `client/src/ui/design_tokens/card_slot.rs:615-626` `fn
  card_slot_node(kind: CardSlotKind) -> Node`.
- **Audited body**:

  ```rust
  pub fn card_slot_node(kind: CardSlotKind) -> Node {
      let geometry = card_slot_geometry(kind);
      Node {
          position_type: PositionType::Absolute,
          display: Display::Flex,
          flex_direction: FlexDirection::Column,
          width: Val::Px(geometry.outer_width_px),
          height: Val::Px(geometry.outer_height_px),
          border: UiRect::all(Val::Px(geometry.border_thickness_px)),
          ..default()
      }
  }
  ```

- **Geometry catalog**: `card_slot_geometry(kind)` exposes
  `image_inset_px`, `text_inset_px`, `hit_target_inset_px`, and
  `z_layer`. The `Node` builder above uses NONE of them. No
  `padding`, no `GlobalZIndex`, no image-child positioning, no
  text-child positioning.
- **User-visible symptom**: the four card-painting surfaces still
  drift in child composition. PROMPT 1034 §2.2 cited: title clips
  into card art; `BOUGHT` band paints across the title rather than
  as a corner ribbon; "3g" tag overlaps art. Each is downstream of
  `card_slot_node` not wiring per-kind text / image insets.
- **Likely root cause**: deliberate per-spec scope. Story 009 §"Scope"
  explicitly deferred per-surface migration of image / text
  positioning to the Sprint 16+ `S16-UI-CARD-SLOT-MIGRATION-*` family.
  The primitive landed minimal so the shop-slot Phase 1 migration
  (PROMPT 1067) could be a thin re-author.
- **Sprint 17 plan rationale**: per `production/sprints/sprint-17.md`
  §"Should Have" row `S17-UI-CARD-SLOT-INSET-WIRING-001` Source
  column: "This row does NOT migrate consumer surfaces; it ratifies
  the primitive so per-surface migration siblings can land cleanly."

---

## Problem Class / Prevention Target

**Defect class**: a primitive that exposes geometry but does not
honour it. Every consumer must re-author the same child-positioning
arithmetic; that arithmetic drifts; downstream the same visible
defects appear on multiple surfaces (PROMPT 1034 §2.2).

**Prevention target**: extend the `card_slot_node` builder (or add
sibling `card_slot_image_inset_node(kind)` /
`card_slot_text_inset_node(kind)` companion builders) so the
primitive wires `position_type: Absolute` + the per-kind inset
rectangle + `GlobalZIndex(card_slot_geometry(kind).z_layer)`. Then
when the four `S17-UI-CARD-SLOT-MIGRATION-*` sibling rows land
(Sprint 17+ Backlog), each per-surface migration is a thin re-author
of three component-set inserts instead of bespoke arithmetic.

**This row does NOT migrate any consumer surface.** It is a
primitive-level ratification only.

---

## Context

### Existing surface

- **`client/src/ui/design_tokens/card_slot.rs`** — defines
  `CardSlotKind` enum (variants per surface: `ShopSlot`, `HandFan`,
  `DraftGrid`, `AuctionFeatured`, `BoardStagedGhost` and possibly
  others; the implementing worker must re-verify the variant list
  at activation HEAD). Defines `card_slot_geometry(kind)`
  returning a struct with `outer_width_px`, `outer_height_px`,
  `border_thickness_px`, `image_inset_px`, `text_inset_px`,
  `hit_target_inset_px`, `z_layer`. Defines `card_slot_node(kind)`
  per the audited body.
- **PROMPT 1067 / 1073 / 1074 worker history**: Sprint 16 story
  009 (`S12-TD-UI-CARD-SLOT-PRIMITIVE-001`) authored the primitive
  module + the Phase 1 shop-slot migration. Closure at
  `origin/main@c9b5716` (PROMPT 1074). Per-surface migration of
  HAND / DRAFT-GRID / AUCTION-FEATURED / BOARD-GHOST is the open
  family.
- **`docs/ux/global-ui-design-spec.md`** §12 (or current
  equivalent) — the card-slot primitive spec section. The
  implementation prompt MAY append a one-line note documenting the
  new image / text inset wiring; otherwise leave untouched.
- **Existing primitive test bin**: `tests/integration/ui_clean_pass/
  card_slot_primitive_test.rs` (or equivalent — re-verify at
  activation HEAD). Story 009 closure (`AC1..AC5 + AC7 + AC8 PASS`)
  cited these tests. The new inset / padding / z-index wiring needs
  new assertions in this bin (or a sibling bin).

### GDD / ADR / TR trace

- **GDD**: no GDD update in scope. The card-slot primitive is a
  presentation-layer module spec.
- **ADR-021** (Presentation Layer Architecture): no schedule or
  system-set change. `card_slot_node` is a pure builder function;
  consumers (when they migrate later) consume the new wiring at
  their existing schedule slot.
- **ADR-002** (Client-Server Authority): no change.
- **TR registry**: no new TR. This is a primitive extension; no
  user-visible behavioural change until per-surface migration
  rows land.

### Engine / skills

- **Engine**: Bevy 0.18 (Rust).
- **Mandatory skills**: `liv-bevy-018` on every `.rs` edit. No
  Lightyear edits — `liv-bevy-lightyear` NOT required.

### Control Manifest Rules

- Required: extend the primitive to honour the geometry catalog
  fields `image_inset_px`, `text_inset_px`, and `z_layer`. Concrete
  shape (extend `card_slot_node` to nest two child builders, OR add
  two new sibling builder functions, OR return a 3-tuple `(outer,
  image_inset, text_inset)`) is **TBD by the implementing worker**
  and MUST be justified in the commit message.
- Required: `GlobalZIndex(card_slot_geometry(kind).z_layer)` is
  threaded through the new wiring. The z-layer constant comes from
  the existing `card_slot_geometry` definition; no new z layer
  number is invented in this row.
- Required: no consumer-surface call site is migrated. Existing
  PROMPT 1067 / 1073 shop-slot Phase 1 migration continues to work
  unchanged; the new builder functions are net-additive.
- Required: `PAW-TD-*-a` preserved (no asset edit).
- Required: `QA-COND-0005` preserved (no hit-target sizing claim
  on the new inset rectangles beyond what `hit_target_inset_px`
  already specifies; this row does NOT introduce a >=44 px claim).
- Required: `QA-COND-0006` preserved (no playtest validation
  claim).
- Forbidden: migrating any consumer surface
  (`client/src/ui/hand/`, `client/src/ui/shop_auction/auction_*`,
  `client/src/ui/shop_auction/draft_*`,
  `client/src/presentation/board_rendering.rs`). Those are out of
  scope per the Sprint 17 plan and per Sprint 16 story 009 §"Scope".
- Forbidden: changing `card_slot_geometry(kind)` constants. This
  row consumes the existing geometry; it does NOT retune.
- Forbidden: introducing animation / tween on inset positioning.
- Forbidden: modifying `shared/`, `server/`, or any test under
  `tests/integration/server/` or `tests/unit/server/`.
- Forbidden: closure of SOURCE-1077-01 / 02 / 03 / 04 / 05 / 07 /
  any other SOURCE-1077-* finding outside SOURCE-1077-06.

---

## Story Classification

**Story type**: **Logic** (primitive builder function; unit-testable
via `World::new()` ECS test pattern that spawns an entity with the
new builder + asserts the resulting `Node` component shape).

Per `.claude/docs/coding-standards.md` "Test Evidence by Story Type"
matrix, Logic stories require automated unit test (BLOCKING gate).

This is **NOT** a:

- Integration story (single-module change; no multi-system wiring).
- Visual / feel story (no consumer surface visibly changes).
- UI layout story (no menu / HUD / screen flow change).
- Final-art story (`PAW-TD-*-a` preserved).
- Accessibility story (`QA-COND-0005` preserved).

---

## Dependencies and Parallelism

### Prerequisites (must be on `origin/main` at Sprint 17 activation HEAD)

- **Sprint 16 story 009 `S12-TD-UI-CARD-SLOT-PRIMITIVE-001`** Done
  (closed PROMPT 1074 on `origin/main@c9b5716`). This row builds
  on the primitive module landed by story 009.

### Parallelism summary

| Sibling story | Parallel-safe? | Notes |
|---|---|---|
| `S17-UI-CARD-DISPLAY-ART-HELPER-001` (Must) | **YES** | disjoint (`asset_wiring.rs` + `shop_auction/mod.rs` + `hand/mod.rs`; not `design_tokens/card_slot.rs`). |
| `S17-UI-MODAL-BLACK-SLAB-001` (conditional Must) | **YES** | disjoint (`shop_auction/mod.rs`). |
| `S17-UI-SHOP-AUCTION-SURFACE-PAINT-001` (conditional Must) | **YES** | disjoint. |
| `S17-UI-HUD-OPP-MANA-CLEANUP-001` (Should) | **YES** | disjoint (`client/src/ui/hud/`). |
| `S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001` (Should) | **YES** | disjoint (`qa_snapshot.rs` + per-surface markers). |
| `S17-UI-BID-BUTTON-PHASE-RACE-001` (Should) | **YES** | disjoint. |
| `S17-OPS-VULKAN-VALIDATION-GATING-001` (Nice) | **YES** | disjoint. |
| `S17-SERVER-START-OF-TURN-DEBUG-001` (Nice) | **YES** | disjoint. |
| `S17-UI-HAND-B0004-CLEANUP-001` (Nice) | **YES** | disjoint. |

This row is the **most parallel-safe Sprint 17 row** because the
primitive module is a leaf with no consumer-surface migration in
scope.

### Per-surface migration siblings (Sprint 17+ Backlog)

The following four rows remain Sprint 17+ Backlog candidates per
`production/sprints/sprint-17.md` §"Wider Sprint 17 Backlog":

- `S17-UI-CARD-SLOT-MIGRATION-HAND-001` — migrate
  `client/src/ui/hand/mod.rs::hand_fan_card_node` to
  `card_slot_node(CardSlotKind::HandFan)` + the new inset wiring
  authored by this row.
- `S17-UI-CARD-SLOT-MIGRATION-DRAFT-GRID-001` — migrate draft
  initial keep-9 modal grid to `card_slot_node(CardSlotKind::
  DraftGrid)`.
- `S17-UI-CARD-SLOT-MIGRATION-AUCTION-FEATURED-001` — migrate
  `client/src/ui/shop_auction/mod.rs::auction_featured_card_node`
  to `card_slot_node(CardSlotKind::AuctionFeatured)`.
- `S17-UI-CARD-SLOT-MIGRATION-BOARD-GHOST-001` — migrate board
  staged-ghost rendering to `card_slot_node(CardSlotKind::
  BoardStagedGhost)`.

Each is owned by its respective consumer module and requires its
own story file before activation. Producer MAY pull one of the four
into Sprint 17 if Sprint 17 capacity allows; do NOT pull more than
one per sprint without separate sprint scoping. **None of the four
are authored by PROMPT 1095.**

---

## Acceptance Criteria

All criteria are independently checkable.

- [x] **AC1 -- Primitive exposes per-kind image inset**: GIVEN the
  post-implementation `client/src/ui/design_tokens/card_slot.rs`,
  WHEN inspected, THEN there is a public builder (either
  `card_slot_image_inset_node(kind)` returning a `Node` configured
  with `position_type: Absolute` + the rectangle defined by
  `card_slot_geometry(kind).image_inset_px`, OR `card_slot_node`
  itself returns a structure including the image inset). The
  builder threads the `z_layer` via `GlobalZIndex`.
  **VERDICT: PASS** — verified on `origin/main@30f166f` at
  `client/src/ui/design_tokens/card_slot.rs:687`:
  `pub fn card_slot_image_inset_node(kind: CardSlotKind) -> (Node,
  GlobalZIndex)` emits `PositionType::Absolute` and the four
  `left/right/top/bottom` edges read verbatim from
  `card_slot_geometry(kind).image_inset_px`; the returned
  `GlobalZIndex` reads from `card_slot_geometry(kind).z_layer`. No
  numeric literal authored in the builder body.

- [x] **AC2 -- Primitive exposes per-kind text inset**: GIVEN the
  same module, WHEN inspected, THEN there is a public builder
  (either `card_slot_text_inset_node(kind)` returning a `Node`
  configured with `position_type: Absolute` + the rectangle defined
  by `card_slot_geometry(kind).text_inset_px`, OR `card_slot_node`
  itself returns a structure including the text inset). The
  builder threads the `z_layer` via `GlobalZIndex`.
  **VERDICT: PASS** — verified on `origin/main@30f166f` at
  `client/src/ui/design_tokens/card_slot.rs:728`:
  `pub fn card_slot_text_inset_node(kind: CardSlotKind) -> (Node,
  GlobalZIndex)` emits `PositionType::Absolute` and the four
  edges read verbatim from `card_slot_geometry(kind).text_inset_px`;
  `GlobalZIndex` mirrors the image-inset builder so the image and
  text children share the same `z_layer` for a given kind.

- [x] **AC3 -- `GlobalZIndex` wired from geometry**: GIVEN both
  inset builders (or the extended outer builder), WHEN inspected,
  THEN they emit a `GlobalZIndex(z_layer)` component where `z_layer`
  comes from `card_slot_geometry(kind).z_layer`. The implementing
  worker confirms the variant-by-variant z_layer constants match
  the existing geometry catalog by reading them from the same
  source.
  **VERDICT: PASS** — per PROMPT 1102 evidence
  `production/qa/evidence/sprint-17-card-slot-inset-wiring/evidence.md`
  §AC3 + integration test
  `s17_inset_image_and_text_builders_thread_global_z_index_per_kind`
  at `tests/integration/ui_clean_pass/card_slot_primitive_test.rs:714`:
  `UI_BASE (300)` for `HandFan` / `DraftGrid` / `ShopSlot` /
  `AuctionFeatured`; `UI_OVERLAY (400)` for `BoardStagedGhost`.
  Values flow through the existing `card_slot_geometry` catalog
  (no inline constant authored in the builder).

- [x] **AC4 -- Padding wired from geometry (if exposed by the
  catalog)**: IF `card_slot_geometry(kind)` exposes a padding
  rectangle (re-verify at activation HEAD; not strictly required
  by SOURCE-1077-06 but called out in the audit minimal-repair
  surface), WHEN the new builder is inspected, THEN the padding is
  applied via `Node.padding`. IF the catalog does not currently
  expose padding, this AC is satisfied trivially by a doc-comment
  noting the catalog does not expose padding and the new builder
  emits no `padding` field.
  **VERDICT: PASS (fallback clause)** — `card_slot_geometry(kind)`
  does NOT currently expose a padding rectangle (only
  `image_inset_px` / `text_inset_px` / `hit_target_inset_px` /
  `z_layer`). Per the AC4 fallback wording, PROMPT 1102 added a
  doc-comment on `card_slot_node` noting the catalog does not
  expose padding; the new builders emit no `Node.padding` field.
  Future revision that promotes padding into `CardSlotGeometry` is
  out of scope (AC8 forbids retuning the geometry catalog).

- [x] **AC5 -- Existing PROMPT 1067 shop-slot Phase 1 migration
  remains green**: GIVEN the existing
  `tests/integration/ui_clean_pass/card_slot_primitive_test.rs`
  (or the post-PROMPT 1074 equivalent), WHEN run, THEN every test
  previously PASS at `origin/main@c9b5716` remains PASS. This row
  is purely additive at the primitive level; it does NOT regress
  any Sprint 16 story 009 closed assertion.
  **VERDICT: PASS** — per PROMPT 1106 integration report:
  `cargo test -p client --test ui_clean_pass_card_slot_primitive_test`
  Finished `27 passed; 0 failed; 0 ignored` on the integration tip
  (19 pre-existing AC1..AC8 assertions PASS + 8 new `s17_*` tests
  PASS). Every Sprint 16 story 009 closed assertion remains green.
  `cargo test -p client --lib card_slot` Finished `9 passed; 0
  failed; 0 ignored` (6 pre-existing + 3 new `s17_*` inline tests).

- [x] **AC6 -- New tests assert inset / z-index wiring**: GIVEN
  `tests/integration/ui_clean_pass/card_slot_inset_wiring_test.rs`
  (NEW; or new assertions in the existing primitive test bin —
  worker's choice, justified in the commit message), WHEN run,
  THEN it asserts for each `CardSlotKind` variant:
  (a) The image-inset builder (or the outer builder's image-inset
  field) produces a `Node` whose width / height / top / left match
  `card_slot_geometry(kind).image_inset_px` (precise pixel match).
  (b) The text-inset builder (or the outer builder's text-inset
  field) produces a `Node` whose width / height / top / left match
  `card_slot_geometry(kind).text_inset_px`.
  (c) Both inset builders emit a `GlobalZIndex` equal to
  `card_slot_geometry(kind).z_layer`.
  (d) The variant set covered by the tests matches the variant set
  defined by `CardSlotKind` (no variant uncovered).
  **VERDICT: PASS** — PROMPT 1102 added new assertions in the
  existing `tests/integration/ui_clean_pass/card_slot_primitive_test.rs`
  bin (justified in PROMPT 1102 commit message: avoids duplicating
  test fixture setup; sibling bin would have required re-deriving
  the variant-iteration helper). 8 new `s17_*` integration tests
  cover (a) `s17_inset_image_node_edges_match_geometry_per_kind`,
  (b) `s17_inset_text_node_edges_match_geometry_per_kind`,
  (c) `s17_inset_image_and_text_builders_thread_global_z_index_per_kind`,
  (d) `s17_inset_builders_cover_every_card_slot_kind_variant`
  asserting `ALL_CARD_SLOT_KINDS.len() == 5`. Plus three defensive
  guards against SOURCE-1077-06 regression:
  `s17_inset_image_node_carries_no_inline_size_overrides_per_kind`
  + the text counterpart +
  `s17_inset_builders_dimensions_resolve_to_positive_interior_per_kind`.
  Three additional inline `s17_*` tests in the
  `#[cfg(test)] mod tests` block of `card_slot.rs`. All assertions
  PASS on `origin/main@30f166f` per PROMPT 1106 cargo test re-run.

- [x] **AC7 -- No consumer surface migrated**: GIVEN
  `git diff <activation HEAD>..HEAD` for the worker's commit, WHEN
  inspected, THEN there are ZERO changes under
  `client/src/ui/hand/`, `client/src/ui/shop_auction/auction_*`,
  `client/src/ui/shop_auction/draft_*`,
  `client/src/presentation/board_rendering.rs`. The only changes
  are in `client/src/ui/design_tokens/card_slot.rs` (and optional
  doc updates in `docs/ux/global-ui-design-spec.md` §12) plus new
  test assertions in `tests/integration/ui_clean_pass/`.
  **VERDICT: PASS** — per PROMPT 1106 integration report,
  `git diff --name-only origin/main..HEAD` returns exactly three
  paths and ZERO of them lie under any consumer-surface directory:
  `client/src/ui/design_tokens/card_slot.rs`,
  `tests/integration/ui_clean_pass/card_slot_primitive_test.rs`,
  `production/qa/evidence/sprint-17-card-slot-inset-wiring/evidence.md`.
  The four per-surface migration siblings remain Sprint 17+
  Backlog under the `S17-UI-CARD-SLOT-MIGRATION-*` family.

- [x] **AC8 -- No `card_slot_geometry` constant change**: GIVEN
  the same diff, WHEN inspected, THEN the body of
  `card_slot_geometry(kind)` (the geometry catalog) is UNCHANGED.
  This row consumes the existing constants; it does NOT retune.
  **VERDICT: PASS** — per PROMPT 1102 evidence §AC8: the diff adds
  new builder functions AFTER the existing `card_slot_node`
  builder; the 14 named per-kind constants (`CARD_SLOT_HAND_FAN_*`,
  `CARD_SLOT_DRAFT_GRID_*`, `CARD_SLOT_SHOP_SLOT_*`,
  `CARD_SLOT_AUCTION_FEATURED_*`, `CARD_SLOT_BOARD_GHOST_*`) and
  the body of `card_slot_geometry(kind)` are UNCHANGED.

- [x] **AC9 -- ADR-021 schedule preserved**: GIVEN `cargo build -p
  client` under the Cargo resource policy, WHEN run, THEN no new
  system-set or schedule wiring is introduced. The primitive is a
  pure builder function; it adds no `App::add_systems`.
  **VERDICT: PASS** — PROMPT 1102 + PROMPT 1106 both ran
  `cargo check -p client` under the Cargo resource policy with
  zero substantive diff-line warnings; no `App::add_systems`
  introduced (only pure builder functions added). ADR-021's
  presentation-layer schedule is preserved.

- [x] **AC10 -- No protocol or server change**: GIVEN
  `git diff <activation HEAD>..HEAD`, WHEN inspected, THEN there
  are zero changes under `server/`, `shared/`, or
  `tests/integration/server/`. The implementation is client-side
  only.
  **VERDICT: PASS** — PROMPT 1106 integration diff confirms zero
  changes under `server/`, `shared/`, or `tests/integration/server/`.
  No protocol-shape change; no new server-authoritative state; no
  new C2S / S2C message.

- [x] **AC11 -- No accept-risk closure claimed**: GIVEN the commit
  message and any evidence document, WHEN inspected, THEN they
  explicitly do NOT claim closure of `S8-QA-001-W1`, `QA-COND-0005`,
  `QA-COND-0006`, `PAW-TD-*-a`, or any other accept-risk
  disposition. Standard-tier hit-target conformance (>=44 px) is
  NOT claimed; per-surface migration is NOT claimed; playtest
  validation is NOT claimed; final-art replacement is NOT claimed.
  **VERDICT: PASS** — PROMPT 1102 worker commit message
  `dev-story(s17): wire card-slot image/text inset + GlobalZIndex
  (PROMPT 1102 S17-UI-CARD-SLOT-INSET-WIRING-001)` carries no
  accept-risk closure language; `evidence.md` §"AC11" + §"Conditions
  carried forward (UNCHANGED)" preserves `S8-QA-001-W1`,
  `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a`, `TQ-S12-C1..C7`,
  PROMPT 761 Polish→Release FAIL, `S11-HUD-TIMER-EYEBALL-VISUAL-001`
  carry, all AUDIT-1076-* findings, all SOURCE-1077-* findings
  outside SOURCE-1077-06, and the 24 PROMPT 1022 audit findings.
  PROMPT 1106 integration report §"Non-claims" preserves the same.

- [x] **AC12 -- Sprint 17 disposition preserved**: GIVEN the
  implementation commit(s), WHEN
  `production/sprint-status.yaml`, `production/sprints/sprint-17.md`
  (and earlier sprint plans), `production/stage.txt`,
  `production/session-state/*`, `production/qa/*`,
  `production/gate-checks/*`, and `docs/architecture/adr-*.md` are
  diffed, THEN none are modified by this story's `/dev-story`
  worker.
  **VERDICT: PASS** — PROMPT 1102 worker + PROMPT 1106 integration
  diffs touch only `client/src/ui/design_tokens/card_slot.rs`,
  `tests/integration/ui_clean_pass/card_slot_primitive_test.rs`,
  and the new
  `production/qa/evidence/sprint-17-card-slot-inset-wiring/evidence.md`.
  `production/sprint-status.yaml`, `production/sprints/sprint-17.md`,
  `production/stage.txt`, `production/session-state/*`, the Sprint 17
  QA plan, `production/qa/smoke-*.md`, `production/qa/team-qa-*.md`,
  `production/gate-checks/*`, and `docs/architecture/adr-*.md` were
  all preserved unchanged across worker + integration. PROMPT 1110
  story-done paperwork is the first authorised modifier of
  `production/sprint-status.yaml` + `production/session-state/*` for
  this row.

- [x] **AC13 -- Worker branch scope contained**: GIVEN the worker
  branch (slug recommendation: `work/s17-card-slot-inset-wiring`),
  WHEN inspected, THEN it pushes only the worker branch — never
  `main`. Files changed at worker time are scoped to
  `client/src/ui/design_tokens/card_slot.rs`, optionally
  `docs/ux/global-ui-design-spec.md` (one-line addendum), and the
  new test assertions / bin under `tests/integration/ui_clean_pass/`.
  **VERDICT: PASS** — PROMPT 1102 pushed branch
  `work/s17-card-slot-inset-wiring` with worker commit
  `55c0dab11ab7572e0cb88827c3ed5f3b241c0fe8`; the worker did NOT
  push `main`. PROMPT 1106 integration was performed separately via
  integration branch `integrate/s17-card-slot-inset-wiring-1106` ->
  `30f166fb9b718bdb5a6a904da0d66cdcc9685f15`. The optional
  `docs/ux/global-ui-design-spec.md` amendment was not authored
  (scope kept tight; §12 reference already clear about the
  primitive's child-positioning contract).

- [x] **AC14 -- Cargo resource policy applied for every Cargo
  command**: future implementation MUST set the binding Cargo
  resource policy env vars (`CARGO_TARGET_DIR=
  D:\_DEV\cargo-target\ccgs-msvc`, `CARGO_PROFILE_DEV_DEBUG=0`,
  `CARGO_PROFILE_TEST_DEBUG=0`, `CARGO_INCREMENTAL=0`,
  `RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'`) before
  every `cargo check` / `cargo test` invocation on Windows / MSVC.
  Disk preflight (~>= 50 GB free on D:) recorded in the worker's
  evidence file. Story authoring (PROMPT 1095) does NOT invoke
  Cargo.
  **VERDICT: PASS-WORKER + ADVISORY-INTEGRATION** — PROMPT 1102
  worker applied all 5 env vars before every Cargo invocation;
  `cargo check -p client` Finished `dev` profile in 6.41s with
  zero warnings; D: free space ~761.8 GB recorded in evidence.
  PROMPT 1106 integration applied the same env-var block but the
  first `cargo test` invocation built into the worktree-local
  `target/` (env vars not applied on the first call due to
  bash-shell `$env:` mangling); subsequent calls correctly routed
  to `D:\_DEV\cargo-target\ccgs-msvc`. The build correctness gate
  is unaffected (all 27/27 + 9/9 tests PASS). Recorded explicitly
  here, not hidden as a product failure. PROMPT 1110 story-done
  paperwork does NOT invoke Cargo.

---

## Implementation Notes

### Owned files (likely change set)

| Path | Expected change |
|------|-----------------|
| `client/src/ui/design_tokens/card_slot.rs` | Extend the primitive to honour `image_inset_px`, `text_inset_px`, and `z_layer`. Add new public builder(s) OR extend the return type of `card_slot_node`. |
| `docs/ux/global-ui-design-spec.md` (optional) | One-line note in §12 (card-slot primitive section) documenting the new inset wiring. Implementation prompt may skip this if the spec is already clear. |
| `tests/integration/ui_clean_pass/card_slot_inset_wiring_test.rs` (NEW; or new assertions in the existing `card_slot_primitive_test.rs`) | AC6 inset / z-index assertions per variant. |
| `production/qa/evidence/sprint-17-card-slot-inset-wiring/evidence.md` (NEW, by `/dev-story` worker) | Evidence document; NOT authored by PROMPT 1095. |

### Forbidden files

- Everything under `server/`, `shared/`.
- Everything under `tests/integration/server/`,
  `tests/unit/server/`, `tests/integration/lightyear*`,
  `tests/unit/lightyear*`.
- `Cargo.toml`, `Cargo.lock`, `.cargo/`, `.github/`, `Trunk.toml`.
- `production/sprint-status.yaml`, `production/sprints/*`,
  `production/stage.txt`, `production/session-state/*`,
  `production/qa/qa-plan-*.md`, `production/qa/smoke-*.md`,
  `production/qa/team-qa-*.md`, `production/gate-checks/*`.
- All other `production/epics/*` story files (no cross-epic edit).
- All four consumer-surface migration sites
  (`client/src/ui/hand/`, `client/src/ui/shop_auction/auction_*`,
  `client/src/ui/shop_auction/draft_*`,
  `client/src/presentation/board_rendering.rs`) — those are out of
  scope per AC7.
- `docs/architecture/adr-*.md` (no ADR amendment in scope).
- `.claude/`, `AGENTS.md`, `CLAUDE.md`, `CODEX.md`.

### Cargo resource policy

Per the Sprint 17 plan and the binding precedent across Sprint 15+
QA plans, every `cargo` invocation on Windows / MSVC MUST set the
five env vars under AC14. This story file MUST NOT amend the policy.

### Target citations

- Sprint 17 plan row source:
  `production/sprints/sprint-17.md` §"Should Have" row
  `S17-UI-CARD-SLOT-INSET-WIRING-001`.
- Source audit:
  `reports/PROMPT-1077-ui-state-source-consistency-deep-audit.md`
  §"Per-finding evidence" SOURCE-1077-06.
- Predecessor: Sprint 16 story 009 closure PROMPT 1074
  (`origin/main@c9b5716`).

---

## Worker Contract (for future `/dev-story`)

The future `/dev-story` worker MUST:

1. Run `git checkout` against Sprint 17 activation HEAD on a fresh
   worktree (suggested slug `work/s17-card-slot-inset-wiring`).
2. Read this story file end-to-end before any code change.
3. Re-verify the audit-time line ranges by reading the current
   `client/src/ui/design_tokens/card_slot.rs`.
4. Re-verify the `CardSlotKind` variant list.
5. Activate `liv-bevy-018` skill before any `.rs` edit. Do NOT
   activate `liv-bevy-lightyear`.
6. Choose the shape extension strategy (extend `card_slot_node`
   vs add new sibling builders vs return a multi-node struct) and
   justify in the commit message.
7. Set the Cargo resource policy env vars per AC14 before every
   `cargo check` / `cargo test` invocation.
8. Run `cargo check -p client` and the targeted `cargo test -p
   client --test card_slot_primitive_test` (or new bin) under the
   Cargo resource policy; confirm zero new warnings on the touched
   file.
9. Push the worker branch (never `main`).
10. Stop. Closure paperwork (`/story-done`, integration `/no-ff`
    merge) is a later prompt's scope.

The worker MUST NOT:

- Modify any consumer surface (HAND / DRAFT-GRID / AUCTION-FEATURED /
  BOARD-GHOST) — those are Sprint 17+ Backlog rows.
- Modify `card_slot_geometry(kind)` constants.
- Modify `server/`, `shared/`, or anything under
  `tests/integration/server/` or `tests/unit/server/`.
- Modify Cargo / Trunk / CI files.
- Modify `production/sprint-status.yaml`, `production/sprints/`,
  `production/stage.txt`, `production/session-state/`,
  `production/qa/qa-plan-*.md`, `production/qa/smoke-*.md`,
  `production/qa/team-qa-*.md`, `production/gate-checks/`.
- Run `/story-done`, `/smoke-check`, `/team-qa`, `/gate-check`,
  `/release-check`, or `/qa-plan` on this story.
- Run the full workspace `cargo test --workspace` invocation
  (targeted bins only per Sprint 15+ QA Policy §"Test Scope Per
  Prompt Type").
- Run `trunk` or any CI command.
- Push to `main`.
- Claim closure of any AUDIT-1076-* or SOURCE-1077-* finding
  outside SOURCE-1077-06.
- Claim release-readiness, accessibility-completion, playtest-
  validation, two-client GAME_OVER closure, final-art completion,
  or stage advance.

### Build gate scope (parallel-agent isolation)

The build gate for this story MUST be scoped to the file this
worker owns (`client/src/ui/design_tokens/card_slot.rs`) plus the
new test bin. The worker MUST NOT block on workspace-wide
compilation errors introduced by other in-flight Sprint 17
workers' branches. This row is file-disjoint with every other
Sprint 17 row except Sprint 16 story 009's already-landed primitive
module — so the parallel risk is minimal.

### Relay / reporting expectation for future workers

Per the Sprint 17 plan and orchestrator contract, every implementing
worker reports back through the GCS local app-server relay (one
single-line DONE summary). The final status line for this story
SHALL be:

```
N: S17-UI-CARD-SLOT-INSET-WIRING-001: STATUS
```

where `N` is the prompt number that ran `/dev-story`.

---

## Completion Notes (PROMPT 1110)

PROMPT 1110 is the paperwork-only `/story-done` closure for
`S17-UI-CARD-SLOT-INSET-WIRING-001` on the strength of:

- **PROMPT 1102 worker** (`55c0dab11ab7572e0cb88827c3ed5f3b241c0fe8`):
  two net-additive sibling builders extending the Sprint 16 PROMPT
  1074 card-slot primitive to honour the geometry catalog's per-kind
  image inset, text inset, and `z_layer`:
  `card_slot_image_inset_node(kind) -> (Node, GlobalZIndex)` and
  `card_slot_text_inset_node(kind) -> (Node, GlobalZIndex)`. Both
  emit `PositionType::Absolute` `Node`s whose `left/right/top/bottom`
  read verbatim from `card_slot_geometry(kind).{image_inset_px,
  text_inset_px}` and a `GlobalZIndex` from
  `card_slot_geometry(kind).z_layer`. `card_slot_node` body and
  `card_slot_geometry` constants UNCHANGED; no consumer-surface
  migration. Worker branch pushed
  `origin/work/s17-card-slot-inset-wiring`. `cargo check -p client`
  passed under Sprint 15+ Cargo resource policy (6.41s; zero
  warnings); `cargo test -p client --test
  ui_clean_pass_card_slot_primitive_test` Finished `27 passed; 0
  failed`; `cargo test -p client --lib card_slot` Finished `9
  passed; 0 failed`. Worker report:
  `reports/PROMPT-1102-s17-card-slot-inset-wiring.md`.
- **PROMPT 1106 integration** (`30f166fb9b718bdb5a6a904da0d66cdcc9685f15`):
  no-ff merge onto `origin/main` via integration branch
  `integrate/s17-card-slot-inset-wiring-1106`. `origin/main`
  advanced from `ff47075` -> `dc8adb6` mid-integration (PROMPT 1107
  server warn->debug landed in parallel; disjoint with this row's
  client/test/qa-evidence diff); integration branch was reset to the
  new `origin/main` and re-merged with no conflict. `git diff
  --name-only origin/main..HEAD` returns exactly three paths
  (`client/src/ui/design_tokens/card_slot.rs`,
  `tests/integration/ui_clean_pass/card_slot_primitive_test.rs`,
  and the new
  `production/qa/evidence/sprint-17-card-slot-inset-wiring/evidence.md`).
  `cargo check -p client` PASS; `cargo test -p client --test
  ui_clean_pass_card_slot_primitive_test` PASS `27 passed; 0 failed`;
  `cargo test -p client --lib card_slot` PASS `9 passed; 0 failed`.
  `liv-bevy-018` review notes confirmed Bevy 0.18 idioms (Required
  Components API; no deprecated `*Bundle`; no `unwrap()`; no
  client-side RNG). Integration tip fast-forward pushed to `main`
  (`dc8adb6..30f166f`). Integration report:
  `reports/PROMPT-1106-s17-card-slot-inset-wiring-integration.md`.

### PROMPT 1106 evidence-file trailing-whitespace advisory

`production/qa/evidence/sprint-17-card-slot-inset-wiring/evidence.md`
inherits two pre-existing trailing-whitespace lines from the PROMPT
1102 source branch (lines 92 and 107, both inside a Markdown bullet:
`...edges_match_geometry_per_kind` + trailing space). `git diff
--check origin/main...HEAD` at integration flagged both. The
condition is **inside a documentation artifact, not code**;
integration did not introduce the whitespace; the source was already
reviewed at PROMPT 1102. PROMPT 1106 explicitly recorded the
condition as a non-blocking documentation artifact in the
integration report §"Step 7" and did not rewrite the evidence file
to preserve worker-authored provenance. PROMPT 1110 preserves this
record verbatim and does NOT rewrite or strip the trailing
whitespace from `evidence.md` (which is already on `origin/main` via
PROMPT 1106 integration). The condition is surfaced explicitly here,
in the `production/sprint-status.yaml` `sprint_17_story_done:`
PROMPT 1110 entry, in the `production/session-state/active.md`
PROMPT 1110 banner, and in the
`production/session-state/codex-orchestrator-state.md` PROMPT 1110
paragraph; it is not hidden.

### Test Evidence

- **Story type**: Logic (per `.claude/docs/coding-standards.md` "Test
  Evidence by Story Type" matrix — BLOCKING gate satisfied by
  automated unit + integration tests).
- **Required evidence per matrix**: automated unit test (BLOCKING).
- **Worker / integration evidence on `origin/main@30f166f`**:
  - `client/src/ui/design_tokens/card_slot.rs` adds 3 new inline
    `s17_*` tests in the `#[cfg(test)] mod tests` block (alongside
    6 pre-existing inline tests; 9 total, all PASS).
  - `tests/integration/ui_clean_pass/card_slot_primitive_test.rs`
    adds 8 new `s17_*` integration tests (alongside 19 pre-existing
    AC1..AC8 assertions; 27 total, all PASS).
  - `production/qa/evidence/sprint-17-card-slot-inset-wiring/evidence.md`
    documents shape extension strategy, files changed, AC1..AC14
    verdicts with command output, Cargo resource policy applied
    record, and conditions carried forward UNCHANGED.
- **Worker + integration cargo gate**: both passed
  (worker `cargo check -p client` 6.41s under policy; integration
  `cargo check -p client` PASS with first-call Cargo resource policy
  advisory noted in AC14 verdict).

---

## Closure Trail

| Prompt | Date | Source-of-truth | Commit | Disposition |
|---|---|---|---|---|
| PROMPT 1095 | 2026-05-18 | `origin/main@7d36191` | (authoring batch tip) | Story authored as Sprint 17 Should Have candidate |
| PROMPT 1097 | 2026-05-18 | `origin/main@bc3db29` | (paperwork-only main integration tip) | Story file integrated to `origin/main` |
| PROMPT 1099 | 2026-05-18 | `origin/main@cb62a9e` | (Sprint 17 activation tip) | Sprint 17 activated; this row included in 9-row active set as Should Have |
| PROMPT 1100 | 2026-05-18 | `origin/main@ff47075` | (Sprint 17 QA plan tip) | Sprint 17 QA plan authored; this row classified as Logic (BLOCKING automated unit test gate) |
| PROMPT 1102 | 2026-05-18 | `origin/main@ff47075` | `55c0dab` | `/dev-story` worker: two new sibling builders + 8 integration tests + 3 inline tests + evidence.md; cargo gate pass under policy |
| PROMPT 1106 | 2026-05-18 | `origin/main@ff47075` → `30f166f` | `30f166f` | Integration: no-ff merge onto `origin/main`; cargo gate pass; trailing-whitespace advisory in evidence.md preserved as documentation artifact; fast-forward push |
| PROMPT 1110 | 2026-05-18 | `origin/main@30f166f` | (this story-done paperwork commit) | `/story-done` paperwork closure: status Draft → Done; AC1..AC13 PASS; AC14 PASS-WORKER + ADVISORY-INTEGRATION |

### Conditions carried forward unchanged

- Sprint 16 disposition `closed-with-conditions` (UNCHANGED).
- Sprint 17 stage `Polish` (UNCHANGED).
- PROMPT 761 Polish->Release gate-check `FAIL` preserved.
- `S8-QA-001-W1` OPEN preserved.
- `QA-COND-0005` + `QA-COND-0006` accepted-risk preserved.
- `PAW-TD-*-a` placeholder-art accept-risk preserved.
- `TQ-S12-C1..C7` preserved verbatim.
- Sprint 15 / 14 / 13 / 12 / 11 / 10 dispositions preserved
  unchanged.
- HUD timer row `S11-HUD-TIMER-EYEBALL-VISUAL-001` human-operator-
  blocked carry preserved; NOT closed by this row.
- 24 PROMPT 1022 audit findings preserved as report-only; NOT
  closed by this row.
- Per-surface migration of HAND / DRAFT-GRID / AUCTION-FEATURED /
  BOARD-GHOST remains Sprint 17+ Backlog under the family
  `S17-UI-CARD-SLOT-MIGRATION-*` (NOT migrated by this row).

### Explicitly NOT claimed by this story or its `/dev-story` worker

- Per-surface migration of any consumer surface (HAND / DRAFT-GRID
  / AUCTION-FEATURED / BOARD-GHOST). Those remain Sprint 17+
  Backlog under the family `S17-UI-CARD-SLOT-MIGRATION-*`.
- Closure of any AUDIT-1076-* finding.
- Closure of any SOURCE-1077-* finding outside SOURCE-1077-06.
- Closure of any of the 24 PROMPT 1022 audit findings.
- Sprint 17 close-out.
- Public release readiness; release-candidate readiness; full game
  completion.
- Broad / Standard-tier accessibility completion; playtest /
  fun-hypothesis validation; full playable-client manual QA;
  two-client GAME_OVER closure; final-art completion;
  Polish->Release gate-check retry; stage advance.

`1110: S17-UI-CARD-SLOT-INSET-WIRING-001: DONE`
