# Epic: UI Clean-Pass

> **Layer**: Presentation / UX / Polish
> **GDD**: design/gdd/hand-ui.md, design/gdd/hud.md, design/gdd/shop-auction-ui.md, design/gdd/board-rendering.md (cross-cut)
> **Architecture Module**: `client/src/ui/`, `client/src/presentation/`
> (read-only roadmap; no code edits land in this epic during Sprint 13)
> **Status**: Draft -- Sprint 13 roadmap-prep landed (story 001 Done
> via PROMPT 840 on `origin/main@0d59ba3`); Sprint 14 foundation
> candidate stories 002-007 authored by PROMPT 878 as candidates and
> all six landed Done across Sprint 14 PROMPT 903 / 908 / 909 / 919 /
> 921 / 922; Sprint 15 Tier 0 Should-priority adjacent story 008
> authored by PROMPT 993, implemented by PROMPT 1005, integrated by
> PROMPT 1007, and closed Done by PROMPT 1009 as a Sprint 15 Nice
> to Have row; Sprint 16 Tier 3 rank 13 candidate story 009 authored
> by PROMPT 1025, repaired by PROMPT 1060 (integrated PROMPT 1063
> `e769757`), activated into Sprint 16 by PROMPT 1064 (`6f9308c`),
> implemented by PROMPT 1067 worker (`3bdf6ac`), integrated by
> PROMPT 1073 (`d12adc4`), and closed Done by PROMPT 1074 on
> `origin/main@c9b5716` as a Sprint 16 Should Have row (AC1..AC5 +
> AC7 + AC8 PASS; AC6 partial -- QA snapshot bundles human-operator
> deferred)
> **Stories**: 1 Sprint 13 roadmap-prep story (Done) + 6 Sprint 14
> Tier 0 foundation stories (Done) + 1 Sprint 15 Tier 0
> Should-priority adjacent story (Done via PROMPT 1009) + 1 Sprint 16
> Tier 3 rank 13 candidate story (Draft, NOT activated) + 6 Sprint
> 16/17 candidate stories authored by PROMPT 1044 (010 shop_auction
> modsplit / 011 hand modsplit / 012 modal primitive / 013 button
> primitive / 014 panel primitive / 015 architecture sequencing
> note; all Draft, NOT activated; address PROMPT 1034 visual audit
> D1-D4 + PROMPT 1035 architecture audit Phase A/B) + 1 Sprint 16
> Nice to Have test-hygiene candidate story authored by PROMPT 1058
> (016 workspace dead-code warning cleanup; Draft, NOT activated;
> addresses the `count_with_image_node` pre-existing warning surfaced
> by Sprint 14 PROMPT 983 smoke and preserved through Sprint 15). The
> remaining PROMPT 802 candidate UI repair rows outside this epic
> remain NOT activated.

## Overview

This epic indexes the **roadmap-prep paperwork** for the PROMPT 802
Expert UI Layout audit. Sprint 13 explicitly does **not** attempt the
full UI overhaul. The single Sprint 13 candidate row produces a
sequenced story index at `docs/ux/ui-clean-pass-roadmap.md` for
Sprint 14+ pull-in.

The 14 PROMPT 802 candidate slugs remain in the wider backlog and are
**not** activated by this epic. The existing PROMPT 685 8-story
milestone backlog is reconciled against the PROMPT 802 candidate set
inside the roadmap note. Friend-game / placeholder-art accept-risk
(`PAW-TD-*-a`) remains preserved; Standard-tier accessibility
(`QA-COND-0005`) and playtest validation (`QA-COND-0006`) remain
accepted-risk and are **not** advanced by this roadmap-prep.

## Governing ADRs

| ADR | Decision Summary | Engine Risk |
|-----|------------------|-------------|
| [ADR-021: Presentation Layer Architecture](../../../docs/architecture/adr-021-presentation-layer-architecture.md) | Roadmap rows reconcile against the canonical `PresentationPlugin` composition order | HIGH |
| [ADR-002: Client-Server Authority](../../../docs/architecture/adr-002-client-server-authority.md) | No optimistic client-side authority is introduced by any roadmap row | HIGH |

## Requirements

| Source | Requirement |
|--------|-------------|
| `reports/PROMPT-802-Expert-UI-Layout-Audit-And-Repair-Roadmap.md` §3 per-surface verdicts | 14 candidate slugs across hand UI, HUD, shop/auction UI, board rendering, lobby |
| `reports/PROMPT-802-Expert-UI-Layout-Audit-And-Repair-Roadmap.md` §6 sequenced repair plan | Provides the sequencing dependencies the roadmap note must preserve |
| `reports/PROMPT-802-Expert-UI-Layout-Audit-And-Repair-Roadmap.md` §11 backlog-vs-recommendation matrix | Reconciles the 14 PROMPT 802 slugs against the existing PROMPT 685 8-story milestone backlog |

## Scope

### In Scope

- A single roadmap note at `docs/ux/ui-clean-pass-roadmap.md` (NEW)
  authored by the single Sprint 13 candidate story.
- Reconciliation of 14 PROMPT 802 candidate slugs against the existing
  PROMPT 685 8-story milestone backlog.
- Identification of the 3-4 highest-impact "must land before any
  polished friend-game-product showcase" rows for Sprint 14 Must Have
  framing.

### Out of Scope

- Activation of any of the 14 PROMPT 802 candidate slugs in Sprint 13.
- Any UI repair implementation in Sprint 13.
- Any change to `PAW-TD-*-a` placeholder-art accept-risk,
  `QA-COND-0005` Standard-tier accessibility, or `QA-COND-0006`
  playtest validation.
- Sprint 13 stage advance or release-scope claims.
- Polish->Release gate-check retry.

## Control Manifest Rules

- Roadmap-prep story lands **doc only**; no code changes under
  `client/`, `server/`, `shared/`, or `tests/`.
- Roadmap note preserves the PROMPT 802 sequencing and explicitly
  names friend-game scope vs Standard-tier-accessibility scope so
  Sprint 14+ pull-in does not silently expand the claim.

## Dependency Map

| Dependency | Use |
|------------|-----|
| PROMPT 802 Expert UI Layout audit | Source of the 14 candidate slugs and sequenced repair plan |
| PROMPT 685 8-story milestone backlog | Prior UI-clean-pass milestone roadmap to reconcile against |
| Hand UI / HUD / Shop-Auction UI / Board Rendering / Lobby epics | Targets of the 14 PROMPT 802 candidate slugs (no story landed in those epics by this roadmap-prep) |

## Stories

| # | Story | Type | Status | Story ID |
|---|-------|------|--------|----------|
| 001 | [PROMPT 802 Expert UI Layout Audit -- Roadmap Prep](story-001-prompt-802-audit-roadmap-prep.md) | Documentation only | Done -- Sprint 13 Nice to Have (closed PROMPT 840 on `origin/main@0d59ba3`) | S13-UI-AUDIT-ROADMAP-PREP-001 |
| 002 | [UI Z-Index Layer Constants](story-002-ui-zindex-layers.md) | Tech Debt -- Tier 0 foundational | Draft -- Sprint 14 candidate, NOT activated | S11-TD-UI-ZINDEX-LAYERS |
| 003 | [UI Typography Scale Tokens](story-003-ui-font-constants.md) | Tech Debt -- Tier 0 foundational | Draft -- Sprint 14 candidate, NOT activated | S11-TD-UI-FONT-CONSTANTS |
| 004 | [UI Flex-Based Strip Composition Primitives](story-004-ui-flex-strips.md) | Tech Debt -- Tier 0 foundational | Draft -- Sprint 14 candidate, NOT activated | S11-TD-UI-FLEX-STRIPS |
| 005 | [UI Viewport-Invariant Test Bin](story-005-ui-viewport-invariant-tests.md) | Tech Debt -- Tier 0 foundational | Draft -- Sprint 14 candidate, NOT activated | S11-TD-UI-VIEWPORT-INVARIANT-TESTS |
| 006 | [UI Overlay Alpha Token (Single Source)](story-006-ui-overlay-alpha-token.md) | Tech Debt -- Tier 0 foundational | Draft -- Sprint 14 candidate, NOT activated | S12-TD-UI-OVERLAY-ALPHA-TOKEN-001 |
| 007 | [Canonical Global UI Design Spec](story-007-global-ui-design-spec.md) | UX -- design-spec authoring | Draft -- Sprint 14 candidate, NOT activated | S12-UX-GLOBAL-UI-DESIGN-SPEC-001 |
| 008 | [UI Interaction State Primitives](story-008-ui-interaction-state-primitives.md) | Tech Debt -- Tier 0 Should-priority adjacent primitive | Done -- Sprint 15 Nice to Have (closed PROMPT 1009 on `origin/main` after PROMPT 1005 dev-story + PROMPT 1007 integration `5d36c4b`) | S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001 |
| 009 | [UI Card Slot Primitive](story-009-ui-card-slot-primitive.md) | Tech Debt -- Tier 3 rank 13 multi-surface primitive | Done -- Sprint 16 Should Have (closed PROMPT 1074 on `origin/main@c9b5716`; PROMPT 1067 worker `3bdf6ac` + PROMPT 1073 integration `d12adc4`; AC1..AC5 + AC7 + AC8 PASS; AC6 partial -- QA snapshot bundles human-operator deferred) | S12-TD-UI-CARD-SLOT-PRIMITIVE-001 |
| 010 | [Shop/Auction Module Split](story-010-ui-shop-auction-module-split.md) | Tech Debt -- structural refactor (file split) | Draft -- Sprint 16/17 candidate (Phase A.1 per PROMPT 1035), NOT activated | S16-TD-UI-SHOPAUCTION-MODSPLIT-001 |
| 011 | [Hand Module Split](story-011-ui-hand-module-split.md) | Tech Debt -- structural refactor (file split) | Draft -- Sprint 16/17 candidate (Phase A.2 per PROMPT 1035), NOT activated | S16-TD-UI-HAND-MODSPLIT-001 |
| 012 | [UI Modal Primitive](story-012-ui-modal-primitive.md) | Tech Debt -- foundational primitive (shared widget) | Draft -- Sprint 16/17 candidate (Phase B per PROMPT 1035; addresses PROMPT 1034 D2), NOT activated | S16-TD-UI-MODAL-PRIMITIVE-001 |
| 013 | [UI Button Primitive](story-013-ui-button-primitive.md) | Tech Debt -- foundational primitive (shared widget) | Draft -- Sprint 16/17 candidate (Phase B per PROMPT 1035; addresses PROMPT 1034 D3), NOT activated | S16-TD-UI-BUTTON-PRIMITIVE-001 |
| 014 | [UI Panel Primitive](story-014-ui-panel-primitive.md) | Tech Debt -- foundational primitive (shared widget) | Draft -- Sprint 16/17 candidate (Phase B.3 per PROMPT 1035; addresses PROMPT 1034 D4), NOT activated | S16-TD-UI-PANEL-PRIMITIVE-001 |
| 015 | [UI Architecture Split + Primitive Sequencing](story-015-ui-architecture-sequencing.md) | Documentation -- sequencing roadmap (doc only) | Draft -- Sprint 16/17 candidate, NOT activated | S16-TD-UI-ARCHITECTURE-SEQUENCING-001 |
| 016 | [Workspace Dead-Code Warning Cleanup](story-016-workspace-dead-code-warning.md) | Tech Debt -- test hygiene (single-helper cleanup) | Done -- Sprint 16 Nice to Have (closed PROMPT 1072 on `origin/main@bd374dd`; PROMPT 1069 worker `2251a93` + PROMPT 1070 integration tip `bd374dd`) | S15-TD-WORKSPACE-DEAD-CODE-WARNING-001 |
| 017 | [Card Display Art Helper / Chrome Preservation + Dedup + Leak Fix + Existence Check](story-017-card-display-art-helper-bundle.md) | Tech Debt -- structural dedup + correctness bundle (Logic + Integration) | Draft -- Sprint 17 candidate (Must Have, `S17-UI-CARD-DISPLAY-ART-HELPER-001`; PROMPT 1077 SOURCE-1077-01 + 02 + 03 + 04 bundle), NOT activated | S17-UI-CARD-DISPLAY-ART-HELPER-001 |
| 018 | [Card-Slot `card_slot_node` Image / Text Inset + GlobalZIndex Wiring](story-018-card-slot-inset-wiring.md) | Tech Debt -- primitive ratification (no consumer-surface migration) | Draft -- Sprint 17 candidate (Should Have, `S17-UI-CARD-SLOT-INSET-WIRING-001`; PROMPT 1077 SOURCE-1077-06), NOT activated | S17-UI-CARD-SLOT-INSET-WIRING-001 |
| 019 | [QA Snapshot Marker Split + Visibility-Aware Counts + Session ID Prefix](story-019-qa-snapshot-marker-split.md) | Tech Debt -- structural refactor + tooling correctness (Integration) | Draft -- Sprint 17 candidate (Should Have, `S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001`; PROMPT 1077 SOURCE-1077-08 + 09 + 16 bundle), NOT activated | S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001 |

### Sprint 16/17 Architecture-Split + Primitive Candidate Sequencing Notes (PROMPT 1044)

Stories 010-015 are the **architecture split + missing-primitives** wave
authored by PROMPT 1044 in response to PROMPT 1034 visual audit (`reports/
PROMPT-1034-full-ui-visual-quality-audit.md`) and PROMPT 1035 architecture
audit (`reports/PROMPT-1035-ui-code-architecture-layout-debt-audit.md`).
Sprint activation decisions for these rows are deferred to a future
producer prompt; none of stories 010-015 are activated by PROMPT 1044.

- **Story 010** (`S16-TD-UI-SHOPAUCTION-MODSPLIT-001`) -- Phase A.1
  split of the 5 435-line `client/src/ui/shop_auction/mod.rs` into a
  thin aggregator + 7 per-surface submodules (draft_initial / shop /
  auction / settlement / toasts / state / spawn). Re-exports preserved,
  no behaviour change. Unblocks story 009 phase-1 shop slot migration,
  Phase C.2 / C.4 / C.5 / C.6 / C.7. **Parallel-safe with story 011.**
- **Story 011** (`S16-TD-UI-HAND-MODSPLIT-001`) -- Phase A.2 split of
  the 4 149-line `client/src/ui/hand/mod.rs` into a thin aggregator +
  5 per-surface submodules (fan / draft_grid / reserve / submit /
  state). Re-exports preserved; `drag_state_visuals.rs` unchanged.
  Unblocks Phase C.1 hand card-slot migration. **Parallel-safe with
  story 010.**
- **Story 012** (`S16-TD-UI-MODAL-PRIMITIVE-001`) -- Phase B modal
  primitive addressing PROMPT 1034 D2. Authors `ModalKind::{Standard,
  Narrow, Featured}` + mandatory opaque scrim + mandatory
  header/body/footer slots + per-modal z-stack policy. Phase 1
  canonical migration: DraftInitial keep-9 modal (PROMPT 1034 A7 P1
  fix; moves `Ready` action into footer slot). Per-surface migration
  of the remaining four modals is Phase C.6 family
  `S16-UI-MODAL-PANEL-CONSOLIDATION-001`.
- **Story 013** (`S16-TD-UI-BUTTON-PRIMITIVE-001`) -- Phase B button
  primitive addressing PROMPT 1034 D3. Authors `ButtonKind::{Primary,
  Secondary, Bid}` + `ButtonState::{Default, Hover, Pressed,
  Disabled, Focused}` composed from story 008's `interaction_states::*`
  tokens. Phase 1 canonical migration: Placement `Submit` button
  (PROMPT 1034 A8 P1 fix). Per-surface migration of remaining button
  families is the Phase C.8 + Phase C-mid family
  `S16-UI-INTERACTION-STATE-MIGRATION-*` (already named by story 008
  close-out).
- **Story 014** (`S16-TD-UI-PANEL-PRIMITIVE-001`) -- Phase B.3 panel
  primitive addressing PROMPT 1034 D4 + PROMPT 1035 §"Result screen /
  connection-lost / photosensitivity cluster". Authors `PanelKind::{
  Standard, Narrow, Toolbar}` + per-kind chrome (background + border
  + padding + border-radius) consolidating three deprecation-target
  RGB triples. Phase 1 canonical migration: Placement action panel
  OR result-screen panel. Per-surface migration of remaining panel
  sites is Phase C.6 family `S16-UI-MODAL-PANEL-CONSOLIDATION-001`.
- **Story 015** (`S16-TD-UI-ARCHITECTURE-SEQUENCING-001`) --
  Documentation-only sequencing note at
  `docs/ux/ui-architecture-split-sequencing.md` (NEW). Inventories
  every Sprint 16+ UI clean-pass story / family member with
  prerequisites, unblocks, owned files, conflicts-with; answers the
  four producer questions ("What must land before DraftShop /
  Auction / Placement re-skins?" + "What can run in parallel after
  Phase A?"). Cross-linked from `docs/ux/ui-clean-pass-roadmap.md`.

Dependency summary (per PROMPT 1035 §"Suggested refactor sequence"
condensed; canonical sequencing note is story 015 deliverable):

- **Phase A (parallel-safe)**: stories 010 + 011. Foundation for every
  Phase B / C row that touches shop / auction / draft / settlement /
  toasts / footer / hand surfaces.
- **Phase B (parallel-safe after Phase A)**: stories 009 (existing
  Sprint 16 candidate) + 012 + 013 + 014. Authoring is file-disjoint
  (different submodules of `design_tokens/`); each story's phase-1
  migration site collides with other Phase B / C rows that target the
  same site, so the producer schedules migration sites accordingly.
- **Phase C (parallel-safe after Phase A + B; per-row file conflicts
  apply)**: hand / shop / auction / board card-slot migrations,
  auction flex primitives, shop control row, modal-panel
  consolidation, palette sweep (runs LAST within Phase C), bid
  interaction states, status icon tints, HUD pill lift. Story 015's
  sequencing note enumerates the file-conflict matrix.
- **Phase D (test discipline)**: cross-surface palette grep guard,
  modal panel consistency test, auction anchor derivation test.

Story 015 is the canonical sequencing reference; the table above is a
summary.

### Sprint 16 Nice to Have Test Hygiene Candidate Sequencing Notes (PROMPT 1058)

Story 016 (`S15-TD-WORKSPACE-DEAD-CODE-WARNING-001`) is a Sprint 16
**Nice to Have** test-hygiene row authored by PROMPT 1058 in response
to `production/sprints/sprint-16.md` §"Nice to Have" row 2 +
§"Pre-activation paperwork inventory" row 3. The row addresses the
pre-existing `count_with_image_node` dead-code warning at
`tests/integration/presentation/hand_ui_asset_wiring_test.rs:43`
surfaced by Sprint 14 PROMPT 983 smoke
(`production/qa/smoke-sprint-14-2026-05-16-rerun.md` §"`cargo check
--workspace --all-targets`" lines ~89-104) and preserved through
Sprint 15.

- Story 016 is **test-hygiene only**: single helper in one test file;
  no production-code change; no spec body amendment; no PROMPT 802 /
  PROMPT 1034 / PROMPT 1035 row dependency.
- Story 016 is **file-disjoint with every other Sprint 16 row**
  (story 009 Should Have card-slot primitive lives under
  `client/src/ui/design_tokens/` or `client/src/ui/primitives/` +
  consumer-surface migrations; the AppCompat manifest Nice to Have
  row lives under `Cargo.toml` + the `spawn_range_live_update_contract`
  test binary; the HUD timer eyeball visual carry is doc / manual
  evidence). It can run in parallel under a separate worker.
- Story 016 is NOT activated by PROMPT 1058. Sprint 16 activation is
  a separate prompt. Sprint 15 disposition (`active` with 4 closed
  Sprint 15 rows + 1 human-operator-blocked Must Have carry per
  PROMPT 1009 / PROMPT 1054) is preserved unchanged. Sprint 14
  disposition (`closed-with-conditions`, `Polish` stage, PROMPT 987)
  is preserved unchanged. The story explicitly does NOT claim
  release readiness, Standard-tier accessibility, playtest
  validation, final-art replacement, or any `QA-COND-*` closure.

### Sprint 16 Candidate Sequencing Notes

Story 009 is the Tier 3 rank 13 multi-surface card-slot primitive
refactor per `docs/ux/ui-clean-pass-roadmap.md` rank 13 (Tier 3,
Should, 1.5d, net-new, PROMPT 802 §3.3 HA1 / §3.3 HA5 / §4 Tier 3.1).
Deferred from Sprint 15 per `production/sprints/sprint-15.md`
"Wider Sprint 15 Backlog (NOT scheduled into this draft; deferred) --
Deliberately deferred to Sprint 16+ (size or coordination overhead)"
because the refactor touches hand + shop + auction together per
PROMPT 802 §8 and would have inflated Sprint 15 into a mega-sprint.

- Story 009 is authored as a **single primitive + spec amendment +
  shop slot phase 1 migration + evidence** row by default, with three
  follow-on migration sibling rows scoped under the family
  `S16-UI-CARD-SLOT-MIGRATION-*` (hand surfaces / auction featured /
  board staged ghost). The Sprint 16 producer MAY bundle the
  primitive + all four migration phases into one row at activation
  time by amending AC5; the default split-shape keeps Sprint 16
  scope discrete and parallel-safe.
- Story 009 depends on:
  - Sprint 14 ranks 1 + 2 + 3 + 6 (DONE: stories 002 / 003 / 004 /
    007) -- the foundational design-token modules and the global UI
    design spec that this story amends with §12.
  - At least one Tier 1 surface stable (DONE: HUD top strip PROMPT 942,
    auction featured PROMPT 931, draft grid centered modal PROMPT 953,
    lobby modal PROMPT 939).
  - Story 008 (`S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001` DONE
    PROMPT 1009) -- the card-slot kinds reference the four
    interaction-state primitive families via doc-comment cross
    reference (AC3); no `interaction_states` edit is authored by
    story 009.
- Story 009 is NOT activated by PROMPT 1025. Sprint 16 activation is
  a separate prompt. Sprint 15 disposition (active; closeout in
  progress) and Sprint 14 disposition (`closed-with-conditions`,
  `Polish` stage) are preserved unchanged. **Per-surface migration
  of hand fan + draft grid + auction featured + board staged ghost
  is OUT OF SCOPE for story 009 by default** -- only the shop slot
  call site is migrated in phase 1; the other three migration phases
  are Sprint 16+ follow-on sibling stories.

### Sprint 15 Candidate Sequencing Notes

Story 008 is the Tier 0 Should-priority adjacent row per
`docs/ux/ui-clean-pass-roadmap.md` Tier 0 Should-priority adjacent
row table (line ~210-214). Per the Sprint 15 plan draft (PROMPT 988)
"Suggested First Parallel Batch" section:

- Story 008 is file-disjoint with the other two Sprint 15
  implementation candidates (`S12-UX-HAND-DRAG-STATE-VISUALS-001`
  hand UI surface; `S11-UX-BOARD-RENDERING-SPEC` doc-only board
  spec). All three can run as a parallel post-activation batch.
- Story 008 pairs with story 007 (`S12-UX-GLOBAL-UI-DESIGN-SPEC-001`)
  Done on Sprint 14 PROMPT 922 -- the existing spec already names
  story 008 in its Spec Adoption Matrix and defers hover / focus /
  pressed / disabled state to story 008's primitive module. The
  spec amendment that story 008's future `/dev-story` authors is a
  forward-reference flip rather than a back-fill.
- Story 008 is NOT activated by PROMPT 993. Sprint 15 activation is
  a separate prompt. Sprint 14 disposition (`closed-with-conditions`,
  `Polish` stage) is preserved unchanged. **Per-surface migration of
  existing Sprint 14 button surfaces (lobby / auction / HUD / shop /
  draft) is OUT OF SCOPE for Sprint 15** -- the primitive module is
  authored and the spec body is amended, but the existing button call
  sites remain on their per-site styling for the duration of Sprint
  15. Per-surface migration is a Sprint 16+ follow-on story family
  (expected slug `S16-UI-INTERACTION-STATE-MIGRATION-*`).

### Sprint 14 Candidate Sequencing Notes

Stories 002-007 are the Tier 0 foundational set per
`docs/ux/ui-clean-pass-roadmap.md` ranks 1-6. Per the roadmap §3
sequencing rules:

- Story 007 (global UI design spec) should land **first** because the
  other five Tier 0 modules need its numeric values as input. Tier 0
  internal sequencing is mostly serial across stories 002 / 003 / 004
  / 006 (shared `client/src/ui/design_tokens/` host module); story 005
  (viewport-invariant test bin) is parallel-safe with the design-token
  work because it lives in `tests/integration/`.
- None of stories 002-007 are activated by PROMPT 878. Sprint 14
  activation is a separate prompt. Sprint 13 disposition (`active`,
  `Polish` stage) is preserved unchanged.

## Definition of Done

- Story 001 (Sprint 13 roadmap-prep): Done -- closed by PROMPT 840 on
  `origin/main@0d59ba3`.
- Stories 002-007 (Sprint 14 Tier 0 foundation): All Done -- closed by
  PROMPT 903 (story 002 `S11-TD-UI-ZINDEX-LAYERS`), PROMPT 908 (story
  003 `S11-TD-UI-FONT-CONSTANTS`), PROMPT 909 (story 005
  `S11-TD-UI-VIEWPORT-INVARIANT-TESTS`), PROMPT 919 (story 004
  `S11-TD-UI-FLEX-STRIPS`), PROMPT 921 (story 006
  `S12-TD-UI-OVERLAY-ALPHA-TOKEN-001`), PROMPT 922 (story 007
  `S12-UX-GLOBAL-UI-DESIGN-SPEC-001`).
- Story 008 (Sprint 15 Tier 0 Should-priority adjacent row): Done
  via PROMPT 1009 on top of PROMPT 1007 integration `5d36c4b` --
  primitive module
  `client/src/ui/design_tokens/interaction_states.rs`,
  `docs/ux/global-ui-design-spec.md` §11 + Spec Adoption Matrix +
  Ratification scope guard amendments, and integration test bin
  `tests/integration/ui_clean_pass/interaction_state_primitives_test.rs`
  ship on `origin/main`. AC1-AC12 all PASS. **Per-surface
  migration explicitly OUT OF SCOPE for Sprint 15** (AC10) -- a
  Sprint 16+ follow-on story family
  (`S16-UI-INTERACTION-STATE-MIGRATION-*`) is required for
  migrating existing Sprint 14 button surfaces to the primitive
  module.
- Sprint 13 activation does **not** silently pull in any of the 14
  PROMPT 802 candidate slugs (preserved by PROMPT 826 activation,
  PROMPT 840 closure, and PROMPT 878 authoring).
- Sprint 14 activation preserved the friend-game vs Standard-tier-
  accessibility scope boundary; `QA-COND-0005`, `QA-COND-0006`, and
  `PAW-TD-*-a` accept-risk dispositions remain preserved through
  Sprint 14 close-out disposition (`closed-with-conditions`, PROMPT
  987).
- Sprint 15 activation (when it happens, in a separate prompt) MUST
  preserve the friend-game vs Standard-tier-accessibility scope
  boundary verbatim; `QA-COND-0005`, `QA-COND-0006`, and `PAW-TD-*-a`
  accept-risk dispositions MUST remain preserved through Sprint 15.
  Story 008 explicitly does NOT claim Standard-tier hit-target
  conformance, broad accessibility completion, playtest validation,
  final-art replacement, or release readiness.
- Story 009 (Sprint 16 Tier 3 rank 13 candidate): Draft -- authored by
  PROMPT 1025 on branch `story/s16-ui-card-slot-primitive` from base
  `origin/main@7b663df` (PROMPT 1023 `integrate(s15): default QA
  snapshot enabled in dev builds`). Sprint 16 activation is a separate
  prompt; story 009 is NOT activated by PROMPT 1025. The story
  preserves the friend-game vs Standard-tier-accessibility scope
  boundary verbatim and explicitly does NOT claim Standard-tier
  hit-target conformance (≥44px), broad accessibility completion,
  playtest validation, final-art replacement, or release readiness.
  Per-surface migration of hand fan + draft grid + auction featured +
  board staged ghost is scoped as Sprint 16+ follow-on sibling stories
  under the family `S16-UI-CARD-SLOT-MIGRATION-*`.
- Story 016 (Sprint 16 Nice to Have test-hygiene candidate): Draft --
  authored by PROMPT 1058 on branch `story/s16-dead-code-warning` from
  base `origin/main@8bec9dca624a191fbc7c12409b2ea4690a1040ab`
  (PROMPT 1055 `chore(state): record P1 UI snapshot retest human
  block`). Sprint 16 activation is a separate prompt; story 016 is
  NOT activated by PROMPT 1058. The story preserves the friend-game
  vs Standard-tier-accessibility scope boundary verbatim and
  explicitly does NOT claim Standard-tier hit-target conformance
  (≥44px), broad accessibility completion, playtest validation,
  final-art replacement, or release readiness. Test-hygiene only:
  single helper in `tests/integration/presentation/hand_ui_asset_wiring_test.rs`
  at line 43; no production-code change; no spec body amendment;
  no `QA-COND-*` closure.
