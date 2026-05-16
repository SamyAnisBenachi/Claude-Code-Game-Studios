# Epic: UI Clean-Pass

> **Layer**: Presentation / UX / Polish
> **GDD**: design/gdd/hand-ui.md, design/gdd/hud.md, design/gdd/shop-auction-ui.md, design/gdd/board-rendering.md (cross-cut)
> **Architecture Module**: `client/src/ui/`, `client/src/presentation/`
> (read-only roadmap; no code edits land in this epic during Sprint 13)
> **Status**: Draft -- Sprint 13 roadmap-prep landed (story 001 Done
> via PROMPT 840 on `origin/main@0d59ba3`); Sprint 14 foundation
> candidate stories 002-007 authored by PROMPT 878 as candidates and
> all six landed Done across Sprint 14 PROMPT 903 / 908 / 909 / 919 /
> 921 / 922; Sprint 15 Tier 0 Should-priority adjacent candidate story
> 008 authored by PROMPT 993 as a Sprint 15 Nice to Have candidate,
> NOT activated by this authoring run
> **Stories**: 1 Sprint 13 roadmap-prep story (Done) + 6 Sprint 14
> Tier 0 foundation stories (Done) + 1 Sprint 15 Tier 0
> Should-priority adjacent candidate story (authored PROMPT 993;
> NOT activated). The remaining PROMPT 802 candidate UI repair rows
> outside this epic remain NOT activated.

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
| 008 | [UI Interaction State Primitives](story-008-ui-interaction-state-primitives.md) | Tech Debt -- Tier 0 Should-priority adjacent primitive | Draft -- Sprint 15 candidate (Nice to Have), NOT activated | S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001 |

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
- Story 008 (Sprint 15 Tier 0 Should-priority adjacent candidate):
  Done when it passes its AC matrix after Sprint 15 activation,
  separate `/dev-story` run, separate `/code-review`, and separate
  `/story-done` closure. **Per-surface migration explicitly OUT OF
  SCOPE for Sprint 15** -- a Sprint 16+ follow-on story family is
  required for migrating existing button surfaces to the primitive
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
