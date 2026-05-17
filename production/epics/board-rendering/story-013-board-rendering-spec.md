# Story 013: S11-UX-BOARD-RENDERING-SPEC -- Canonical Board Rendering Spec

> **Epic**: Board Rendering
> **Story ID**: S11-UX-BOARD-RENDERING-SPEC
> **Status**: Done via PROMPT 1009 (2026-05-17) after PROMPT 1004
> `/dev-story` authored the canonical spec and PROMPT 1006 integrated
> it onto `origin/main@08f389b276fba73769816fcb206de61a6bb9fda8`.
> **Layer**: UX / Producer-planning / Design-spec authoring
> **Type**: UX -- design-spec authoring (doc-only)
> **Sprint**: Sprint 15 Should Have (Tier 3 rank 14, Should; PROMPT 685 row 6
> re-validated by PROMPT 802 §3.7 B1 / §4 Tier 3.2;
> `docs/ux/ui-clean-pass-roadmap.md` rank 14). Sprint 15 activated by
> PROMPT 997; Sprint 14 disposition (`closed-with-conditions`, `Polish`
> stage) preserved.
> **Authored**: 2026-05-16 by PROMPT 992
> **Authoring source-of-truth**: `origin/main@2c84d6e` (PROMPT 990
> Sprint 15 plan draft integration on top of PROMPT 988 sprint plan)
> **Estimated effort**: ~0.75d (Sprint 15 plan §"Should Have"; roadmap rank 14)

---

## Status / No-Claim Banner

This story is authored as a Sprint 15 candidate. **Sprint 15 is NOT activated
by this authoring run.** The story is paperwork only -- no code change is
attempted by PROMPT 992. The future `/dev-story` for this row is **doc-only**
and does NOT modify any file under `client/`, `server/`, `shared/`, or
`tests/`.

PROMPT 992 (this authoring run) does NOT:

- Activate Sprint 15.
- Modify `production/sprint-status.yaml`.
- Modify `production/sprints/sprint-15.md` or any other sprint plan file.
- Modify `production/stage.txt` (remains `Polish`).
- Modify any `production/session-state/*` file.
- Modify any QA-plan / smoke / Team-QA / gate-check / release-check artifact.
- Run `/story-readiness`, `/dev-story`, `/story-done`, `/smoke-check`,
  `/team-qa`, `/gate-check`, `/release-check`, or `/qa-plan` on this story.
- Author the canonical board rendering spec at
  `docs/ux/board-rendering-spec.md` (that is the future `/dev-story`
  output; explicitly out of scope for this story-authoring prompt).
- Modify any code under `client/`, `server/`, `shared/`, or `tests/`.
- Modify any file under `production/epics/` other than the new story file
  and the board-rendering `EPIC.md` index.

This story, and the future `/dev-story` that consumes it, does **not**
claim, advance, or close any of:

- Public release readiness.
- Release-candidate (RC) readiness.
- Full game completion.
- Broad / Standard-tier accessibility completion (`QA-COND-0005`).
- Playtest / fun-hypothesis validation (`QA-COND-0006`).
- Full playable-client manual QA.
- Two-client GAME_OVER closure (`S8-QA-001-W1`).
- Final-art / asset-production completion (`PAW-TD-002-a` ..
  `PAW-TD-006-a`).
- Sprint 14 row reopen (any of the 16 closed Sprint 14 rows).
- `Polish -> Release` gate-check retry (PROMPT 761 `FAIL` preserved; NO
  retry is in scope).
- Stage advance from `Polish` to `Release`.
- Underlying drag-runtime bug fix (Sprint 12 story 019 remains
  `closed-with-conditions / cannot-reproduce`).
- Closure of `TQ-S12-C7` or any other `TQ-S12-C1..C7` Team-QA condition.
- Full Board Rendering epic closure (BR-011 spawn range highlights and
  the final visual / evidence split-follow-ups remain open).

### Accept-risk dispositions preserved verbatim

- **`QA-COND-0005`** -- Standard-tier accessibility remains
  **accepted-risk** (friend-game scope only). The future board rendering
  spec is **friend-game visual polish only**; it does NOT pursue WCAG
  contrast ratios on board overlays, ≥44px hit-targets, keyboard
  navigation, screen reader support, colorblind modes, or text scaling.
- **`QA-COND-0006`** -- playtest / fun-hypothesis validation remains
  **accepted-risk / deferred**. A visibly documented board does not by
  itself produce playtest evidence.
- **`PAW-TD-002-a` .. `PAW-TD-006-a`** -- placeholder-art accept-risk
  across PAW-002..PAW-006. Board rendering spec authoring is layout /
  composition / hierarchy / overlay / status-icon-legend work and does
  **not** advance placeholder-art resolution. Final atlas / icon art
  replacement is out of spec scope.

---

## Overview

The board rendering surface in `client/src/ui/board/` (BR-001 through
BR-012 on `origin/main`) renders cells, unit sprites, objective sprites,
HP bars, status icons, co-occupancy offsets, placement reveal tweens,
resolution playback, ghost previews, and (per BR-011) spawn range
highlights. The implementation is healthy on `main`: BR-001..BR-005 are
Complete, BR-006 is Ready, BR-007/008 are Blocked on protocol gaps,
BR-009 (status icons + co-occupancy) is Complete, BR-010 / BR-012 are
Complete (browser/WASM baseline evidence), and BR-011 (spawn range
highlights) is Ready.

Two adjacent visual concerns were surfaced as future-candidate cosmetic
captures by PROMPT 802 / PROMPT 685 but never folded into a single
canonical spec:

- `S11-UX-BOARD-GHOST-PREVIEW-OPACITY-001` -- ghost preview opacity
  rule (Tier 2 cosmetic capture).
- `S11-UX-BOARD-STATUS-ICON-LEGEND-001` -- status icon legend (Tier 2
  cosmetic capture).

The Sprint 14 Global UI Design Spec (`docs/ux/global-ui-design-spec.md`;
`S12-UX-GLOBAL-UI-DESIGN-SPEC-001` DONE per PROMPT 911 / 912 / 922)
established the **cross-cutting** spec for layers, spacing, typography,
overlay alpha, color, and responsive rules. Its §2 Scope Boundaries
explicitly defers board-rendering spec authoring to a separate Tier 3
story: "**Board-rendering** -- world-space sprite layout under
`client/src/presentation/board_rendering.rs` is owned by Tier 3 story 14
(`S11-UX-BOARD-RENDERING-SPEC`, doc-only; depends on this spec)."

This story authors that companion spec at `docs/ux/board-rendering-spec.md`,
folding the two cosmetic-capture future candidates above into named
sections (NOT as separate Sprint 15 stories), so the board-rendering
surface has a single canonical spec readable alongside the global UI
design spec.

Per `docs/ux/ui-clean-pass-roadmap.md` §3 Sequencing Rule 4, Tier 3
rank 14 is doc-only and depends on rank 6 (`S12-UX-GLOBAL-UI-DESIGN-SPEC-001`)
as its parent design-spec doc. Both rank-6 prerequisite and the BR-011
spawn range data-source story are already on `origin/main` (rank 6 DONE
Sprint 14; BR-011 Ready), so the doc is authorable from existing repo
state without additional implementation prerequisites.

---

## Scope

### In Scope

The future `/dev-story` for this row authors a new design-spec document
at `docs/ux/board-rendering-spec.md` (NEW) covering at least the following
sections:

- **§1 Status / No-Claim Banner**: friend-game scope only;
  `QA-COND-0005` / `QA-COND-0006` / `PAW-TD-*-a` accept-risk preserved
  verbatim; this spec does not advance Standard-tier accessibility, does
  not advance playtest validation, does not advance placeholder-art
  resolution, does not close `S8-QA-001-W1`, does not retry the
  Polish->Release gate-check, does not advance stage from `Polish` to
  `Release`. Mirrors the structure of
  `docs/ux/global-ui-design-spec.md` §1.

- **§2 Scope Boundaries**: friend-game visual polish only; board
  rendering spec governs world-space sprite composition, cell rendering,
  unit placement, range overlay, status icon legend, and ghost preview
  opacity. It does NOT govern accessibility (WCAG contrast on overlays,
  hit-targets, keyboard navigation, screen reader support, colorblind
  modes, text scaling on board labels), gameplay rules (combat
  resolution, keyword resolution, spawn validation), or networking
  contracts (protocol shape, message ownership, single-drain). Lists
  cross-cutting concerns that delegate UP to
  `docs/ux/global-ui-design-spec.md` (z-layers, typography, overlay
  alpha, color palette, responsive rules) and OUT to ADR-021 / ADR-020
  (presentation plugin order, board layout authority).

- **§3 Cell Rendering Rules**: canonical 5-lane x 8-cell grid layout;
  `cell_to_world` coordinate authority (per ADR-021 R2 and
  TR-BR-002); cell sprite atlas frame selection; lane-color tinting
  rule (if any); cell-border / cell-fill composition; world-space
  z-layer reference back to `docs/ux/global-ui-design-spec.md` §3
  `World` (z=100) and `Units` (z=200). Names the canonical
  cell sprite sizes (pixels) and the relation between cell pixel size
  and camera zoom across the canonical 6-viewport matrix (from
  `docs/ux/global-ui-design-spec.md` §8). Friend-game palette only;
  does NOT specify final-art cell tiles.

- **§4 Unit Placement Rules**: canonical unit sprite anchor on a
  `(team, lane, cell)` tuple; co-occupancy offsets per ADR-021 R3 and
  TR-BR-007 (the F3 ±half-offset rule for two allied co-occupants;
  index-2 `assert!` per BR-009 acceptance); ChildOf hierarchy for HP
  bars and status icons so co-occupancy offsets propagate to children
  through hierarchy (NOT re-centered on the cell); world-space z
  reference back to `docs/ux/global-ui-design-spec.md` §3 `Units`
  (z=200). Names canonical HP-bar geometry (above-unit anchor, width,
  height, color thresholds) without re-specifying gameplay HP
  thresholds (those live in the GDD).

- **§5 Range Overlay Rules**: canonical spawn range highlight rendering
  per TR-BR-008 (BR-011 source contract: `PlayerSnapshot.spawn_range_cells`
  for snapshot rebuild + `SpawnRangeChanged` resolution-log event for
  live updates); cell-tint or outline composition; persistence vs
  transient overlay rule (per ADR-020 / ADR-011 source ownership);
  draft-phase placement-ghost cursor mapping rule (per BR-004 ghost
  preview bridge); z-layer reference back to
  `docs/ux/global-ui-design-spec.md` §3 (overlay layering on `UiOverlay`
  for any bevy_ui overlay vs world-space sprite for world-space
  overlay). Range overlay color uses friend-game palette tokens from
  `docs/ux/global-ui-design-spec.md` §7; final-art replacement is out
  of spec scope.

- **§6 Status Icon Legend**: canonical mapping of persistent keyword /
  state kinds to status icon atlas frames (SHIELD, STUN, SILENCE,
  INJURED, LEADER, HASTE, BODYGUARD, OUTNUMBERED, and INJURED-granted
  keyword indicators per BR-009 Complete scope); Tier 1 vs Tier 2
  display priority and sort key (per TR-BR-006); overflow-badge rule
  (top 3 visible icons + `+N` badge per BR-009 acceptance); per-unit
  vs per-lane distinction for OUTNUMBERED (per TR-BR-007 / TR-KW-010,
  `OQ-KS5` closed in `design/gdd/keyword-system.md`); ChildOf hierarchy
  rule so icons inherit co-occupancy parent offsets. **This section
  folds the `S11-UX-BOARD-STATUS-ICON-LEGEND-001` cosmetic-capture
  candidate into the spec** -- it does NOT remain a separate Sprint 15
  story.

- **§7 Ghost Preview Opacity**: canonical ghost preview opacity rule for
  hand drag-and-drop placement preview on the board; alpha value
  rationale (must be a single named value, not an ad-hoc literal); the
  rule that ghost preview alpha is sprite-level alpha, NOT modal scrim
  alpha (i.e. does NOT consume `OVERLAY_DIM_ALPHA` /
  `OVERLAY_SCRIM_ALPHA` from `docs/ux/global-ui-design-spec.md` §6
  because those govern bevy_ui modal scrims, not world-space sprite
  alpha; explicit scope-guard cross-link to the global UI spec §6
  "Scope guard" paragraph). Names the ghost preview lifecycle (spawn
  on hover / move with cursor / despawn on drop-or-cancel) but does
  NOT re-specify the bridge protocol (which lives in BR-004 ghost
  preview bridge story already Complete). **This section folds the
  `S11-UX-BOARD-GHOST-PREVIEW-OPACITY-001` cosmetic-capture candidate
  into the spec** -- it does NOT remain a separate Sprint 15 story.

- **§8 References to docs/ux/global-ui-design-spec.md**: a dedicated
  cross-reference section enumerating which subsections of the global
  UI design spec the board rendering spec depends on, with a brief
  one-line rationale per reference. At minimum the section names:
  - `docs/ux/global-ui-design-spec.md` §3 (Z-Index Layer System) --
    canonical `World` / `Units` layer integer values; ADR-021 R2 paint
    order preserved.
  - `docs/ux/global-ui-design-spec.md` §6 (Overlay Alpha Tokens) --
    `OVERLAY_DIM_ALPHA` / `OVERLAY_SCRIM_ALPHA` are bevy_ui modal-scrim
    tokens; board ghost preview alpha is **out of scope** for those
    tokens per the §6 "Scope guard" paragraph; ghost preview opacity
    is named in §7 of this spec instead.
  - `docs/ux/global-ui-design-spec.md` §7 (Color Tokens) -- friend-game
    palette tokens (e.g. `SEMANTIC_SUCCESS`, `SEMANTIC_WARNING`,
    `SEMANTIC_ERROR`) referenced for range overlay / status icon tint
    where applicable.
  - `docs/ux/global-ui-design-spec.md` §8 (Responsive Layout Rules) --
    canonical 6-viewport matrix; world board scales with viewport via
    camera zoom per §8 per-class scaling rules.

- **§9 ADR / GDD Cross-References**: read-only links to ADR-021
  (Presentation Layer Architecture), ADR-020 (Board / Lane System State
  Architecture), ADR-017 (Combat Resolution Execution Architecture),
  ADR-011 (Reconnect and Snapshot), ADR-008 (Lightyear Channel
  Configuration), ADR-002 (Client-Server Authority), and
  `design/gdd/board-rendering.md`. Read-only; this spec does NOT modify
  the ADRs or the GDD.

- **§10 Producer Ratification Checklist**: UX-designer + art-director +
  producer sign-off rows (mirrors `docs/ux/global-ui-design-spec.md`
  Producer Ratification Checklist). Each role records a one-paragraph
  rationale per ratification at the future `/dev-story` authoring run.
  This is the AC6 ratification gate.

Cross-references back to existing per-system docs (read-only):

- `design/gdd/board-rendering.md` -- GDD source of truth for board
  rendering rules, formulas, and acceptance criteria.
- `docs/ux/global-ui-design-spec.md` -- parent design-spec doc.
- `docs/ux/ui-clean-pass-roadmap.md` -- rank 14 sequencing entry.
- `docs/architecture/adr-021-presentation-layer-architecture.md` --
  presentation plugin order and PresentationSet.
- `docs/architecture/adr-020-board-lane-state-architecture.md` -- board
  state replication contract.
- `production/epics/board-rendering/EPIC.md` -- epic-level index.
- BR-009 closure notes for status icon implementation surface.
- BR-011 story file for spawn range data-source contract.

### Out of Scope

- **No Sprint 15 activation** by this story.
- **No `/dev-story` execution** by this story-authoring prompt;
  PROMPT 992 authors the story file only. The actual spec authoring
  happens at a future `/dev-story` run after Sprint 15 activation and
  after this story's `/story-readiness` passes against activation HEAD.
- **No spec file authored**: `docs/ux/board-rendering-spec.md` does
  NOT exist on `origin/main` at story-authoring time and MUST NOT be
  created by this story-authoring prompt. It is the future `/dev-story`
  output, explicitly forbidden by PROMPT 992 scope.
- **No public release readiness** work.
- **No Standard-tier accessibility** (`QA-COND-0005`) completion. The
  spec explicitly DOES NOT govern accessibility. A separate
  accessibility spec is a follow-on scope.
- **No final-art / asset-production** work (`PAW-TD-*-a`). Color tints,
  status icon atlas frame art, and cell tile art remain placeholder /
  friend-game palette; final-art replacement is a separate sprint scope.
- **No playtest validation** (`QA-COND-0006`).
- **No code change** under `client/`, `server/`, `shared/`, or
  `tests/`. The future `/dev-story` authors a design document only.
  Board rendering implementation work is owned by BR-001..BR-012
  (already mostly Complete on `main`) and by BR-011 (Ready for spawn
  range highlights `/dev-story`).
- **No protocol / network-shape change**. The spec is doc-only and
  does NOT modify Lightyear receivers, message ownership, single-drain
  discipline, or replication contracts. ADR-021 / ADR-020 / ADR-011 /
  ADR-008 / ADR-017 preserved verbatim.
- **No HUD spec**. HUD per-element layout, top-strip child order, and
  resource readouts are owned by Sprint 14 stories
  `S11-UX-HUD-TOP-STRIP-LAYOUT` (DONE) /
  `S11-UX-HUD-BOTTOM-STRIP-LAYOUT` (DONE) /
  `S11-UX-HUD-OPP-FIGURINE` (DONE). This spec does NOT re-specify HUD
  rules.
- **No hand UI spec**. Hand UI drag-state visuals are owned by Sprint 15
  Should Have `S12-UX-HAND-DRAG-STATE-VISUALS-001` (separate story).
  This spec only references ghost preview opacity on the **board** side
  (the sprite painted on the board, not the dragged card in the hand).
- **No animation / motion** spec. Placement reveal tweens (BR-005
  Complete) and resolution playback (BR-006 Ready) animation spec
  remain owned by their existing stories and by ADR-017.
- **No interaction-state primitive** spec. Hover / focus / pressed /
  disabled state primitives are owned by Sprint 15 Nice to Have
  `S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001` (separate story).
- **No localization** spec.
- **No update of `design/gdd/board-rendering.md`** or any other GDD.
  Cross-links only; GDD edits are out of scope here.
- **No `production/sprint-status.yaml`, `production/sprints/*`,
  `production/session-state/*`, `production/qa/*`, or
  `production/stage.txt` modification** by this story-authoring prompt
  or by the future `/dev-story`. Future `/dev-story` is doc-only.
- **No update of `production/epics/board-rendering/` story files other
  than this NEW story-013 file and the EPIC.md index update**. BR-001
  through BR-012 are not modified by this authoring prompt.
- **No Sprint 15 row activation** in `production/sprint-status.yaml`
  (the Sprint 15 plan draft on `main` carries this row as a draft
  candidate; activation is a separate prompt).

---

## Acceptance Criteria

All criteria are independently checkable BLOCKING criteria. They apply
to the **future `/dev-story` output** at `docs/ux/board-rendering-spec.md`,
not to this story-authoring run.

- [x] **AC1 -- Spec authored**: GIVEN the future `/dev-story` commit,
  WHEN the new spec file is inspected, THEN
  `docs/ux/board-rendering-spec.md` exists. Verification: file presence.

- [x] **AC2 -- All required sections present**: GIVEN the spec, WHEN
  the table of contents is inspected, THEN at least sections §1
  (Status / No-Claim Banner), §2 (Scope Boundaries), §3 (Cell Rendering
  Rules), §4 (Unit Placement Rules), §5 (Range Overlay Rules), §6
  (Status Icon Legend), §7 (Ghost Preview Opacity), §8 (References to
  `docs/ux/global-ui-design-spec.md`), §9 (ADR / GDD Cross-References),
  and §10 (Producer Ratification Checklist) are present.
  Verification: heading scan
  (`rg "^## " docs/ux/board-rendering-spec.md`).

- [x] **AC3 -- Status / No-Claim Banner**: GIVEN §1, WHEN inspected,
  THEN the friend-game-vs-Standard-tier scope is explicitly stated and
  `QA-COND-0005` (Standard-tier accessibility), `QA-COND-0006`
  (playtest validation), `PAW-TD-002-a..006-a` (placeholder-art),
  `S8-QA-001-W1` (two-client GAME_OVER), and the PROMPT 761
  `Polish->Release` gate-check `FAIL` are each named as preserved
  accept-risk / not-claimed-by-this-spec. Verification: doc review.

- [x] **AC4 -- Cell rendering rules named**: GIVEN §3, WHEN inspected,
  THEN canonical 5-lane x 8-cell grid layout, `cell_to_world` authority
  reference, world-space z-layer reference to
  `docs/ux/global-ui-design-spec.md` §3 (`World` / `Units`), and the
  relation between cell pixel size and the canonical 6-viewport
  matrix from `docs/ux/global-ui-design-spec.md` §8 are present.
  Verification: doc review against ADR-021 R2, TR-BR-002.

- [x] **AC5 -- Unit placement rules named**: GIVEN §4, WHEN inspected,
  THEN canonical unit sprite anchor on `(team, lane, cell)`, co-occupancy
  ±half-offset rule with the F3 index-2 `assert!` reference, and ChildOf
  hierarchy for HP bars and status icons are present. Verification: doc
  review against ADR-021 R3, TR-BR-007, BR-009 acceptance.

- [x] **AC6 -- Range overlay rules named**: GIVEN §5, WHEN inspected,
  THEN canonical spawn range highlight rendering rule with reference to
  the TR-BR-008 / BR-011 source contract
  (`PlayerSnapshot.spawn_range_cells` for snapshot rebuild +
  `SpawnRangeChanged` resolution-log event for live updates), and the
  draft-phase placement-ghost cursor mapping rule (per BR-004 Complete)
  are present. Verification: doc review.

- [x] **AC7 -- Status icon legend present**: GIVEN §6, WHEN inspected,
  THEN canonical mapping of persistent keyword / state kinds (SHIELD,
  STUN, SILENCE, INJURED, LEADER, HASTE, BODYGUARD, OUTNUMBERED) to
  status icon atlas frames, Tier 1 / Tier 2 priority ordering, overflow
  badge rule, and per-unit OUTNUMBERED distinction are present. The
  section text explicitly states that this section folds the
  `S11-UX-BOARD-STATUS-ICON-LEGEND-001` cosmetic-capture future
  candidate into the spec rather than as a separate story.
  Verification: doc review against BR-009 closure notes, TR-BR-006,
  TR-BR-007, TR-KW-010.

- [x] **AC8 -- Ghost preview opacity present**: GIVEN §7, WHEN inspected,
  THEN a single canonical ghost preview opacity rule with named alpha
  value and rationale, an explicit scope-guard cross-link to
  `docs/ux/global-ui-design-spec.md` §6 confirming ghost preview alpha
  is sprite-level (NOT bevy_ui modal scrim), and the ghost preview
  lifecycle hand-off are present. The section text explicitly states
  that this section folds the `S11-UX-BOARD-GHOST-PREVIEW-OPACITY-001`
  cosmetic-capture future candidate into the spec rather than as a
  separate story. Verification: doc review against BR-004 Complete.

- [x] **AC9 -- References to docs/ux/global-ui-design-spec.md present**:
  GIVEN §8, WHEN inspected, THEN the cross-reference enumeration names
  the global UI design spec §3 (Z-Index Layer System), §6 (Overlay Alpha
  Tokens scope-guard cross-link), §7 (Color Tokens), and §8
  (Responsive Layout Rules) with one-line rationale per reference.
  Verification: doc review; `rg` for the literal
  `docs/ux/global-ui-design-spec.md` returns at least 4 matches inside
  `docs/ux/board-rendering-spec.md`.

- [x] **AC10 -- ADR / GDD cross-references present**: GIVEN §9, WHEN
  inspected, THEN read-only links to ADR-021, ADR-020, ADR-017,
  ADR-011, ADR-008, ADR-002, and `design/gdd/board-rendering.md` are
  present. Verification: doc review.

- [x] **AC11 -- Producer + UX-designer + art-director ratification
  checklist present**: GIVEN §10, WHEN inspected, THEN producer +
  UX-designer + art-director sign-off rows are present with a one-
  paragraph rationale per role recorded at the future `/dev-story`
  authoring time. The future `/dev-story` commit body (or a sibling
  evidence doc under
  `production/qa/evidence/sprint-15-board-rendering-spec/`) records
  the three sign-offs. Verification: doc review of §10 + commit body /
  evidence doc inspection.

- [x] **AC12 -- No code change**: GIVEN the future `/dev-story`
  commit, WHEN `git diff` is inspected, THEN no file under `client/`,
  `server/`, `shared/`, or `tests/` is modified. Verification:
  `git diff origin/main...HEAD -- 'client/**' 'server/**' 'shared/**'
  'tests/**'` returns no output.

- [x] **AC13 -- Friend-game scope preserved**: GIVEN the future
  `/dev-story` commit, WHEN `QA-COND-0005`, `QA-COND-0006`, and
  `PAW-TD-002-a..006-a` accept-risk dispositions are inspected, THEN
  none of them has been flipped to `closed` by this story.
  `S8-QA-001-W1` remains OPEN. PROMPT 761 `Polish->Release` gate-check
  `FAIL` remains preserved. Verification: `git diff` of
  `production/sprint-status.yaml` shows no accept-risk disposition
  change; `production/gate-checks/gate-polish-release-2026-05-12.md`
  remains untouched.

- [x] **AC14 -- No Sprint 15 activation by future /dev-story**: GIVEN
  the future `/dev-story` commit, WHEN
  `production/sprint-status.yaml`, `production/sprints/sprint-15.md`,
  `production/session-state/*`, `production/stage.txt`, and
  `production/qa/*` are inspected, THEN none of them is modified by
  the doc-only `/dev-story` (Sprint 15 activation is a separate
  prompt and is NOT carried by the spec authoring run). Verification:
  `git diff origin/main...HEAD -- 'production/sprint-status.yaml'
  'production/sprints/**' 'production/session-state/**'
  'production/stage.txt' 'production/qa/**'` returns no output.

- [x] **AC15 -- Status Icon Legend folded as section**: GIVEN §6 of
  the future spec, WHEN inspected, THEN the section text explicitly
  names `S11-UX-BOARD-STATUS-ICON-LEGEND-001` as a future-candidate
  cosmetic capture that this spec folds into a section rather than as
  a separate Sprint 15 story. Verification: doc review.

- [x] **AC16 -- Ghost Preview Opacity folded as section**: GIVEN §7
  of the future spec, WHEN inspected, THEN the section text explicitly
  names `S11-UX-BOARD-GHOST-PREVIEW-OPACITY-001` as a future-candidate
  cosmetic capture that this spec folds into a section rather than as
  a separate Sprint 15 story. Verification: doc review.

---

## Evidence Path

`production/qa/evidence/sprint-15-board-rendering-spec/` (NEW; future
`/dev-story` populates this directory).

Expected artifacts:

- Doc-review checklist (markdown) confirming each AC against the
  authored spec.
- Section heading enumeration
  (`rg "^## " docs/ux/board-rendering-spec.md`) for AC2 verification.
- Cross-reference matrix between the new spec and
  `docs/ux/global-ui-design-spec.md` (AC9).
- UX-designer + art-director + producer sign-off rationale captured
  either inline in §10 of the spec OR in a sibling
  `ratification.md` evidence file under the evidence directory (AC11).

---

## Likely Files Touched (by future /dev-story)

| Path | Expected change |
|------|-----------------|
| `docs/ux/board-rendering-spec.md` (NEW) | Author canonical board rendering spec. |
| `production/qa/evidence/sprint-15-board-rendering-spec/` (NEW) | Doc-review checklist + optional ratification.md. |
| `docs/ux/ui-clean-pass-roadmap.md` | Possible amendment to cross-link the new spec (optional; worker discretion at /dev-story time). |
| `docs/ux/global-ui-design-spec.md` | Possible Cross-References section addition naming this spec as the board-rendering companion (optional; worker discretion). |

This table is a planning estimate. The implementation prompt is
authoritative for the realised set.

**Explicitly NOT touched** by the future `/dev-story`:

- `client/src/**`
- `server/src/**`
- `shared/src/**`
- `tests/**`
- `Cargo.toml`, `Cargo.lock`, `.cargo/`, `Trunk.toml`
- `production/sprint-status.yaml`
- `production/sprints/sprint-15.md` or any other sprint plan file
- `production/stage.txt`
- `production/session-state/**`
- `production/qa/qa-plan-sprint-15.md` or any other QA-plan / smoke /
  Team-QA / gate-check / release-check file
- `production/epics/board-rendering/` story files other than this story
  013 file (BR-001..BR-012 untouched by the future `/dev-story`)
- `design/gdd/board-rendering.md` (read-only cross-link only)
- `docs/architecture/adr-*.md` (read-only cross-link only)
- `.claude/`, `.octogent/`, `.github/`

---

## Verification (by future /dev-story)

- File presence check on `docs/ux/board-rendering-spec.md`.
- `git diff --check origin/main...HEAD` -- clean diff.
- `git diff origin/main...HEAD -- 'client/**' 'server/**' 'shared/**'
  'tests/**'` -- returns empty (AC12).
- `git diff origin/main...HEAD -- 'production/sprint-status.yaml'
  'production/sprints/**' 'production/session-state/**'
  'production/stage.txt' 'production/qa/**'` -- returns empty (AC14).
- Doc-review checklist (manual) -- AC2 through AC11, AC15, AC16.
- Reviewer sign-off from UX-designer + art-director + producer on the
  AC11 ratification checklist (paper sign-off acceptable; captured in
  spec §10 or evidence `ratification.md`).

---

## Dependencies / Sequencing

- **Story-authoring prompt (PROMPT 992)** is the *story-authoring*
  prompt; it creates this story file only. No `/dev-story` runs here.
- **Activation**: Requires Sprint 15 activation (separate prompt; not
  this one). Cannot run `/dev-story` in Sprint 14.
- **Sequencing within Sprint 15**: doc-only; parallel-safe with all
  other Sprint 15 rows per the Sprint 15 plan §"Suggested First
  Parallel Batch":
  - Parallel-safe with `S12-UX-HAND-DRAG-STATE-VISUALS-001`
    (hand-ui surface vs `docs/ux/` doc).
  - Parallel-safe with `S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001`
    (design_tokens module vs `docs/ux/` doc).
  - Parallel-safe with the two Must Have paperwork rows
    (`S11-HUD-TIMER-EYEBALL-VISUAL-001` cosmetic visual check;
    `S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001-ROWFLIP` row-status
    flip).
- **Tier 3 dependency**: per `docs/ux/ui-clean-pass-roadmap.md` rank 14
  "depends on rank 6 (design-spec parent doc)". Rank 6
  (`S12-UX-GLOBAL-UI-DESIGN-SPEC-001`) is DONE on `origin/main` per
  Sprint 14 PROMPT 911 / 912 / 922; dependency satisfied.
- **BR-011 spawn range source dependency**: the §5 Range Overlay Rules
  section references the BR-011 source contract
  (`PlayerSnapshot.spawn_range_cells` snapshot + `SpawnRangeChanged`
  live event). BR-011 is Ready on `origin/main`; the spec can reference
  the contract from its story file even if BR-011 `/dev-story` has
  not yet run. If BR-011 closes before this spec `/dev-story` runs,
  the spec references the closure evidence directly.
- **Producer-decision dependency**: UX-designer + art-director +
  producer must ratify the spec values at the future `/dev-story`
  authoring time (AC11). PROMPT 802 §9 producer-decision-5 (Tier 2
  cosmetic captures bundling) is INAPPLICABLE here because this spec
  folds the two relevant Tier 2 captures
  (`S11-UX-BOARD-GHOST-PREVIEW-OPACITY-001` +
  `S11-UX-BOARD-STATUS-ICON-LEGEND-001`) as spec sections rather than
  as separate cosmetic-capture stories.
- **No blocking dependency on Sprint 15 QA plan**: the
  `/qa-plan sprint-15` authoring is a separate prompt that must
  precede any `/dev-story` per Sprint 15 plan; for this doc-only row
  the QA plan content is light (manual doc-review checklist), but the
  QA plan must exist on `main` before `/dev-story` runs.

---

## Notes

- PROMPT 685 row 6: original "BOARD-RENDERING-SPEC" candidate
  (subsumed-by `S11-UX-BOARD-RENDERING-SPEC` per PROMPT 802 §3.7 B1 /
  §4 Tier 3.2 reconciliation).
- PROMPT 802 §3.7 B1 / §4 Tier 3.2: re-validated the rank against
  `origin/main@b5eef0d`.
- `docs/ux/ui-clean-pass-roadmap.md` rank 14: "doc-only; depends on
  rank 6 (design-spec parent doc)".
- `docs/ux/global-ui-design-spec.md` §2 Scope Boundaries: "**Board-
  rendering** -- world-space sprite layout under
  `client/src/presentation/board_rendering.rs` is owned by Tier 3
  story 14 (`S11-UX-BOARD-RENDERING-SPEC`, doc-only; depends on this
  spec)."
- `docs/ux/global-ui-design-spec.md` §6 Scope guard: "this token
  covers *modal scrim / dim* surfaces only. Board ghost preview
  opacity (a future Tier 2 row
  `S11-UX-BOARD-GHOST-PREVIEW-OPACITY-001`) and any sprite-level alpha
  remain out of scope." This story's §7 Ghost Preview Opacity is the
  canonical home for the ghost preview alpha rule per that scope
  guard.
- Two Tier 2 cosmetic-capture future candidates folded as spec
  sections (NOT as separate Sprint 15 stories):
  - `S11-UX-BOARD-GHOST-PREVIEW-OPACITY-001` -> §7.
  - `S11-UX-BOARD-STATUS-ICON-LEGEND-001` -> §6.
  The Sprint 15 plan §"Wider Sprint 15 Backlog" explicitly records
  this folding: "Two of these
  (`S11-UX-BOARD-GHOST-PREVIEW-OPACITY-001` +
  `S11-UX-BOARD-STATUS-ICON-LEGEND-001`) are folded as spec sections
  into Sprint 15 Should Have `S11-UX-BOARD-RENDERING-SPEC` rather than
  as separate captures."
- Accept-risk preservation: `PAW-TD-002-a..006-a`, `QA-COND-0005`,
  `QA-COND-0006`, `S8-QA-001-W1`, PROMPT 761 `Polish->Release`
  gate-check `FAIL`, PROMPT 683-era runtime divergence question,
  Sprint 12 story 019 `closed-with-conditions / cannot-reproduce`
  preserved unchanged. This story does not advance any of them.
- The future `/dev-story` is doc-only and authors only
  `docs/ux/board-rendering-spec.md` (+ optional evidence directory and
  optional cross-link amendments per the Likely Files Touched table).
  No code change. No protocol change. No GDD change.

---

## Completion Notes

**Completed**: 2026-05-17 by PROMPT 1009 `/story-done` serialized
paperwork closure (Sprint 15 integrated story-done batch).

**Criteria**: 16 / 16 accepted. AC1-AC16 PASS via the
PROMPT 1004 `/dev-story` worker and PROMPT 1006 integration
verification. The doc-only spec at `docs/ux/board-rendering-spec.md`
ships 865 lines covering §1-§10 plus Spec Adoption Matrix,
Cross-References, and Authoring Trail meta sections; cross-references
to `docs/ux/global-ui-design-spec.md` resolved at 26 hits (>> AC9 ≥ 4
threshold); folded captures `S11-UX-BOARD-STATUS-ICON-LEGEND-001` and
`S11-UX-BOARD-GHOST-PREVIEW-OPACITY-001` enumerated 10 combined
matches; `GHOST_PREVIEW_ALPHA` named in §7 token table, scope-guard
table, §10 ratification, and Spec Adoption Matrix.

**Deviations**: None. The spec is doc-only as scoped; no `client/`,
`server/`, `shared/`, or `tests/` file touched; `production/
sprint-status.yaml` and `production/sprints/sprint-15.md` untouched
by PROMPT 1006 integration; producer + UX-designer + art-director
ratifications captured under
`production/qa/evidence/sprint-15-board-rendering-spec/ratification.md`
per AC11.

**Test Evidence**: AC1-AC16 verified per
`production/qa/evidence/sprint-15-board-rendering-spec/doc-review-checklist.md`
and the PROMPT 1006 integration command table (spec heading
enumeration, global-UI-spec cross-reference count, folded-capture
match count, ghost-preview-alpha token count, allowed-path scan,
forbidden-path scan, `git diff --check`, and `git diff --cached
--check` — all PASS at integration commit
`08f389b276fba73769816fcb206de61a6bb9fda8`). Cargo not run (doc-only;
"No Cargo expected" per task spec).

**Code Review**: PROMPT 1009 verified integration commit
`08f389b276fba73769816fcb206de61a6bb9fda8` is reachable from
`origin/main` (current tip `88a6db16e8abec6b2e7df1f8efac0fc933b5c0b3`
via PROMPT 1008), reviewed PROMPT 1004 worker report and PROMPT 1006
integration report for AC coverage, and performed paperwork-only
closure. No `client/`, `server/`, `shared/`, `tests/`, Cargo,
`production/sprints/sprint-15.md`, `production/qa/qa-plan-sprint-15.md`,
`production/stage.txt`, or gate artifact was edited by PROMPT 1009.

## Closure Trail

- PROMPT 992 (2026-05-16) -- story authoring on branch
  `story-authoring/sprint-15-board-rendering-spec`, worker commit
  `29953a1`. Integrated into `origin/main` by PROMPT 995 batch
  merge `8294f9a`.
- PROMPT 1004 (2026-05-17) -- `/dev-story` authoring of the
  canonical board rendering spec on branch
  `work/s15-board-rendering-spec`, worker commit
  `477806ac88e71da152f4852399450dba6f4ee1de`. Authored
  `docs/ux/board-rendering-spec.md` (NEW; 865 lines) plus three
  evidence files under
  `production/qa/evidence/sprint-15-board-rendering-spec/`
  (`doc-review-checklist.md`, `cross-ref-matrix.md`,
  `ratification.md`).
- PROMPT 1006 (2026-05-17) -- integration merge
  `08f389b276fba73769816fcb206de61a6bb9fda8` onto `origin/main`.
  No-ff merge from `origin/main@84e621e`; 4 files / 1232 insertions;
  allowed-path scan PASS; forbidden-path scan PASS;
  `git diff --check` PASS; `git diff --cached --check` PASS.
- PROMPT 1009 (2026-05-17) -- serialized `/story-done` paperwork
  closure within the Sprint 15 integrated story-done batch.
  Story status marked Done, Sprint 15 row flipped `ready -> done`
  with completed date 2026-05-17, AC1-AC16 checkboxes marked
  complete, session-state banners prepended, and
  `sprint_15_story_done` block appended at EOF of
  `production/sprint-status.yaml`. Sprint 15 remains active; stage
  remains Polish; PROMPT 761 Polish->Release FAIL, `S8-QA-001-W1`
  OPEN, `QA-COND-0005/0006` accepted-risk, `PAW-TD-*-a` accepted-risk,
  `S11-HUD-TIMER-EYEBALL-VISUAL-001` human-operator-blocked carry,
  and `S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001-ROWFLIP` open
  status all preserved.
