# Epic: UI Clean-Pass

> **Layer**: Presentation / UX / Polish
> **GDD**: design/gdd/hand-ui.md, design/gdd/hud.md, design/gdd/shop-auction-ui.md, design/gdd/board-rendering.md (cross-cut)
> **Architecture Module**: `client/src/ui/`, `client/src/presentation/`
> (read-only roadmap; no code edits land in this epic during Sprint 13)
> **Status**: Draft -- Sprint 13 candidate index for the UI clean-pass
> audit roadmap; NOT activated
> **Stories**: 1 Sprint 13 candidate roadmap-prep story; NOT activated.
> The 14 PROMPT 802 candidate UI repair rows are NOT activated here.

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

| # | Story | Type | Status | Sprint 13 Slug |
|---|-------|------|--------|----------------|
| 001 | [PROMPT 802 Expert UI Layout Audit -- Roadmap Prep](story-001-prompt-802-audit-roadmap-prep.md) | Documentation only | Draft -- Sprint 13 candidate (Nice to Have), NOT activated | S13-UI-AUDIT-ROADMAP-PREP-001 |

## Definition of Done

- Story 001 passes `/story-readiness` against Sprint 13 activation
  HEAD.
- `docs/ux/ui-clean-pass-roadmap.md` (NEW) is authored with the 14
  PROMPT 802 slugs sequenced and reconciled against PROMPT 685.
- Sprint 13 activation does **not** silently pull in any of the 14
  PROMPT 802 candidate slugs.
