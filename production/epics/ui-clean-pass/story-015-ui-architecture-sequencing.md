# Story 015: S16-TD-UI-ARCHITECTURE-SEQUENCING-001 -- UI Architecture Split + Primitive Dependency Map

> **Epic**: UI Clean-Pass
> **Story ID**: S16-TD-UI-ARCHITECTURE-SEQUENCING-001
> **Status**: Draft -- Sprint 16/17 candidate, NOT activated
> **Layer**: Documentation -- sequencing roadmap
> **Type**: Documentation only -- no code, no test, no spec body change
> **Sprint**: Sprint 16/17 candidate per PROMPT 1035 §"Suggested
> refactor sequence" + PROMPT 1034 §5 "Parallel-safe repair roadmap" +
> §6 "Suggested prompt list". Authoring-only deliverable: a single
> sequencing note that names what must land before DraftShop /
> Auction / Placement re-skins and what can run in parallel.
> **Authored**: 2026-05-17 by PROMPT 1044
> **Authoring source-of-truth**: `origin/main@a7a8b079` (PROMPT 1041).
> **Estimated effort**: ~0.25d (single sequencing note + cross-link
> from `docs/ux/ui-clean-pass-roadmap.md`; no behaviour change)

---

## Status / No-Claim Banner

This story is authored as a Sprint 16/17 candidate. **No sprint is
activated by this authoring run.** PROMPT 1044 does NOT activate any
sprint, modify sprint-status / sprint plan / stage / session-state,
modify code under `client/` / `server/` / `shared/` / `tests/`, or
modify any spec body (the future `/dev-story` deliverable is a single
new `docs/ux/ui-architecture-split-sequencing.md` note + a one-line
cross-reference in `docs/ux/ui-clean-pass-roadmap.md`).

This story does **not** claim: public release readiness, full game
completion, broad / Standard-tier accessibility completion
(`QA-COND-0005`), Standard-tier hit-target conformance (≥44px),
playtest validation (`QA-COND-0006`), full playable-client manual QA,
two-client GAME_OVER closure (`S8-QA-001-W1`), final-art completion
(`PAW-TD-*-a`), `Polish->Release` retry, or stage advance.

---

## Overview

PROMPT 1034 §3 D1-D14 + PROMPT 1035 §"Phase A / B / C / D" each
publish their own dependency graphs across the same set of rows.
Stories 010 / 011 / 012 / 013 / 014 carry per-story dependency
sections, but the **producer needs a single canonical map** to plan
Sprint 16 / 17 / 18 activation order.

This story authors the future `/dev-story` worker's deliverable for a
**single sequencing note** at
`docs/ux/ui-architecture-split-sequencing.md` (NEW) that:

1. Catalogues every Sprint 16+ UI clean-pass row that touches the
   shop / auction / draft / placement / hand / HUD / lobby / modal
   surfaces.
2. For each row, lists prerequisites (rows that must land first) and
   unblocked rows (rows whose dependencies it satisfies).
3. Names parallel-safe lanes: which rows can run simultaneously
   under different workers.
4. Names file-conflict edges: which rows must serialize against each
   other.
5. Answers the four producer questions the audit raises:
   - "What must land before the DraftShop re-skin?" (PROMPT 1034 A5)
   - "What must land before the Auction re-skin?" (PROMPT 1034 A6 +
     §2.5)
   - "What must land before the Placement re-skin?" (PROMPT 1034 A8)
   - "What rows can run in parallel after Phase A clears?"

The note is **doc-only** and does NOT amend the global UI design
spec or the board-rendering spec. It is a producer-facing scheduling
aid, cross-referenced from `docs/ux/ui-clean-pass-roadmap.md` (which
remains the canonical roadmap; this note is its scheduling annex).

Per the PROMPT 1035 §"Phase A / B / C / D" tables, the sequencing
note's content is largely already drafted there; this story's
deliverable extracts the producer-facing summary into a single
docs-tree file that is easier to point Sprint 16+ activation prompts
at than a 27 k-token audit report.

---

## Scope

### In Scope

#### Sequencing note authoring

- A new file at `docs/ux/ui-architecture-split-sequencing.md` (NEW)
  with at least these sections (worker MAY restructure for
  readability; required content is the bullet list below):

  1. **Provenance** — cites PROMPT 1034 audit + PROMPT 1035 audit +
     this story file. Cites the `origin/main` source-of-truth commit
     hash at authoring time.

  2. **Per-row inventory table** — one row per Sprint 16+ UI
     clean-pass story / family member:
     - Story 009 `S12-TD-UI-CARD-SLOT-PRIMITIVE-001` (existing Sprint
       16 candidate)
     - Story 010 `S16-TD-UI-SHOPAUCTION-MODSPLIT-001`
     - Story 011 `S16-TD-UI-HAND-MODSPLIT-001`
     - Story 012 `S16-TD-UI-MODAL-PRIMITIVE-001`
     - Story 013 `S16-TD-UI-BUTTON-PRIMITIVE-001`
     - Story 014 `S16-TD-UI-PANEL-PRIMITIVE-001`
     - Phase B.1 `S16-TD-UI-COLORS-TOKEN-001` (named in PROMPT 1035
       §"Phase B.1"; story file not yet authored)
     - Phase C.1 `S16-UI-CARD-SLOT-MIGRATION-HAND-001` (named in
       story 009 §"Parallelization")
     - Phase C.2 `S16-UI-CARD-SLOT-MIGRATION-AUCTION-001`
     - Phase C.3 `S16-UI-CARD-SLOT-MIGRATION-BOARD-GHOST-001`
     - Phase C.4 `S16-UI-AUCTION-FLEX-PRIMITIVES-001` (named in
       PROMPT 1035 §"Phase C.4")
     - Phase C.5 `S16-UI-SHOP-CONTROL-ROW-001` (named in PROMPT 1035
       §"Phase C.5")
     - Phase C.6 `S16-UI-MODAL-PANEL-CONSOLIDATION-001` (named in
       stories 012 + 014)
     - Phase C.7 `S16-UI-COLORS-MIGRATION-001` (palette sweep; named
       in PROMPT 1035 §"Phase C.7")
     - Phase C.8 `S16-UI-INTERACTION-STATE-MIGRATION-BID-001` +
       `S16-UI-INTERACTION-STATE-MIGRATION-{LOBBY,DRAFT,SHOP,HUD}-*`
       (per story 008 close-out family)
     - Phase C.9 `S16-UI-STATUS-ICON-TINTS-001` (named in PROMPT
       1035 §"Phase C.9")
     - Phase C.10 `S16-UI-HUD-PILL-PRIMITIVE-001` (named in PROMPT
       1035 §"Phase C.10")
     - P1 surface rows from PROMPT 1034 §5 Phase A (A5 DraftShop
       surface visible; A6 Auction bid amounts; A7 Keep-9 modal
       opaque scrim — superseded by story 012; A8 Placement action
       panel — partially superseded by stories 013 + 014)
     - Phase D test-discipline rows (D.1 palette grep guard; D.2
       panel consistency test; D.3 auction anchor derivation test)

     Each table row contains: **slug**, **status at authoring time**
     (DONE / story-authored / not-yet-authored), **prerequisite
     slugs**, **unblocks slugs**, **owned files (path globs)**,
     **conflicts-with slugs**.

  3. **Per-question producer answers**:
     - **"What must land before the DraftShop re-skin?"** Answer:
       Story 010 (shop_auction modsplit) + story 014 (panel
       primitive for shop control row chrome) + story 013 (button
       primitive for shop refresh / ready / hand-full controls). The
       missing-DraftShop-surface defect (PROMPT 1034 A5 / F-shop) is
       a P1 surface-content fix that can land BEFORE the primitives
       on the current legacy card / per-site styling, then re-skin
       through the primitives in a follow-on row. The re-skin
       depends on stories 010 + 013 + 014 + Phase C.5 shop control
       row.
     - **"What must land before the Auction re-skin?"** Answer:
       Story 010 (shop_auction modsplit) + story 013 (button
       primitive for bid buttons) + story 014 (panel primitive for
       bid-row chrome) + Phase C.4 auction flex primitives
       (`AuctionToolbar`, `AuctionBidRow`). The auction bid `?`
       placeholders (PROMPT 1034 A6 / F-4) MAY land as a P1
       content-only fix BEFORE the primitives, then re-skin in
       follow-on.
     - **"What must land before the Placement re-skin?"** Answer:
       Story 011 (hand modsplit) + story 013 (button primitive for
       `Submit`) + story 014 (panel primitive for action-panel
       chrome). The Placement action panel is the canonical
       migration site for stories 013 and 014; landing both stories
       discharges the Placement re-skin defect (PROMPT 1034 A8 / F-7).
     - **"What rows can run in parallel after Phase A clears?"**
       Answer: After stories 010 + 011 land on `origin/main`, the
       following lanes are file-disjoint and parallel-safe:
       - Lane 1 (primitives): stories 012 + 013 + 014 + Phase B.1
         colours can author in parallel under separate workers.
       - Lane 2 (existing Sprint 16 row): story 009 card-slot
         primitive runs independently.
       - Lane 3 (Phase C migrations): C.1 hand card-slot migration
         (after story 011 + story 009), C.2 auction card-slot (after
         story 010 + story 009), C.3 board-ghost card-slot (after
         story 009), C.4 auction flex (after story 010), C.5 shop
         control row (after story 010), C.6 modal-panel
         consolidation (after stories 010 + 012 + 014), C.9 status
         icon tints (independent).
       - Lane 4 (palette sweep): C.7 colours migration runs **LAST**
         within Phase C because it conflicts with C.1 / C.2 / C.4 /
         C.6 at the call-site level.
       - Lane 5 (test discipline): Phase D rows run after their
         corresponding Phase C rows land.

  4. **File-conflict matrix** — same row × column table where the
     cell value is "PARALLEL" / "SERIALIZE" / "N/A". The matrix is
     scoped to the 15-20 rows enumerated in the inventory table; the
     producer reads it to verify that any proposed parallel batch
     selects only rows whose pairwise cells say "PARALLEL".

  5. **Sequencing diagram (optional)** — a Mermaid graph (`flowchart
     LR`) showing the dependency DAG. Worker MAY embed it as a
     fenced Mermaid block; rendering is best-effort (the doc tree
     does not currently render Mermaid, but the source is readable
     in raw form).

  6. **Schedule recommendation** — a producer-facing recommended
     activation order for Sprint 16 / 17 / 18, with caveats:
     - Sprint 16 already has story 009 as Should Have headline.
       Adding stories 010 + 011 to Sprint 16 is feasible (Phase A is
       ~1.5 d combined; the existing Sprint 16 plan has 2 d of
       buffer per `production/sprints/sprint-16.md` §"Capacity").
       Producer decides at Sprint 16 activation.
     - Sprint 17 candidates: stories 012 + 013 + 014 + Phase B.1
       colours.
     - Sprint 18 candidates: Phase C migrations + Phase D test
       discipline.

#### Cross-link from existing roadmap

- A one-line cross-reference added to
  `docs/ux/ui-clean-pass-roadmap.md` pointing at the new sequencing
  note (e.g. a new bullet under the existing §"Sequencing Rules"
  section). The roadmap remains canonical; the new note is its
  scheduling annex.

### Out of Scope

- **No new story files.** Stories 010-014 are authored by PROMPT
  1044 alongside this note; Phase B.1 / C.1-C.10 / D.1-D.3 story
  files remain to be authored by future story-authoring prompts and
  are referenced by slug only in this note.
- **No code change.** Doc-only deliverable.
- **No spec body change.** The global UI design spec is unchanged.
  The board-rendering spec is unchanged.
- **No sprint-plan amendment.** `production/sprints/sprint-16.md`
  draft is unchanged.
- **No sprint-status / session-state / stage edit.**
- **No Sprint 16 activation, no Sprint 15 close-out, no Polish ->
  Release retry, no QA / smoke / Team-QA / gate-check / release-check
  workflow run, no `/qa-plan` authoring.**

---

## Acceptance Criteria

All BLOCKING.

- [ ] **AC1 -- Sequencing note authored**:
  `docs/ux/ui-architecture-split-sequencing.md` (NEW) exists and
  contains the six required sections (Provenance / Per-row inventory
  / Per-question producer answers / File-conflict matrix /
  Sequencing diagram (optional) / Schedule recommendation).
  Verification: file presence + section-header scan.

- [ ] **AC2 -- Per-row inventory completeness**: The inventory table
  contains AT LEAST every slug enumerated in §"In Scope" above. Each
  row has non-empty `slug`, `status at authoring`, `prerequisites`,
  `unblocks`, `owned files`, `conflicts-with` cells.

- [ ] **AC3 -- Per-question producer answers**: All four producer
  questions ("What must land before DraftShop / Auction / Placement
  re-skins?" + "What can run in parallel after Phase A?") have
  explicit answers that name specific story slugs.

- [ ] **AC4 -- File-conflict matrix coverage**: The matrix is square
  on the inventory row set. Every pair has a cell value (no
  unfilled cells). No two rows that share the same file glob are
  marked PARALLEL.

- [ ] **AC5 -- Cross-link added**:
  `docs/ux/ui-clean-pass-roadmap.md` carries a one-line bullet or
  reference pointing at the new sequencing note.

- [ ] **AC6 -- Provenance**: The note cites PROMPT 1034 audit, PROMPT
  1035 audit, this story file, and the `origin/main` source-of-truth
  commit hash at authoring time.

- [ ] **AC7 -- Non-claims**: No code / test / server / shared /
  protocol change. No spec body amendment. No sprint activation /
  close-out / stage advance. No release / full-game / Standard-tier
  / playtest / final-art / two-client closure claim. No closed-row
  reopen. Verification: `git diff origin/main...HEAD --stat --
  'client/' 'server/' 'shared/' 'tests/' 'production/sprint-status.yaml'
  'production/sprints/' 'production/stage.txt' 'production/session-state/'
  'production/qa/'` is empty.

- [ ] **AC8 -- Doc-only**: The only files changed are
  `docs/ux/ui-architecture-split-sequencing.md` (NEW) and
  `docs/ux/ui-clean-pass-roadmap.md` (one-line cross-link).

---

## Implementation Notes

### Owned files

| Path | Expected change |
|------|-----------------|
| `docs/ux/ui-architecture-split-sequencing.md` (NEW) | Author the sequencing note per AC1-AC4 + AC6. |
| `docs/ux/ui-clean-pass-roadmap.md` | One-line cross-reference per AC5. |

### Forbidden files

- Everything under `client/`, `server/`, `shared/`, `tests/`,
  `Cargo.toml`, `Cargo.lock`, `.cargo/`.
- `production/sprint-status.yaml`, `production/sprints/*`,
  `production/stage.txt`, `production/session-state/*`,
  `production/qa/qa-plan-*.md`, `production/qa/smoke-*.md`,
  `production/qa/evidence/*`.
- All existing `docs/` files except the one-line cross-link target
  `ui-clean-pass-roadmap.md`.
- `docs/ux/global-ui-design-spec.md`, `docs/ux/board-rendering-spec.md`
  -- not amended.
- `.claude/**`, `AGENTS.md`, `CLAUDE.md`.

---

## Parallelization and Dependencies

| Sibling story | Parallel-safe? |
|---|---|
| **Stories 010 / 011 / 012 / 013 / 014** | **YES**, doc-only deliverable; the sequencing note REFERENCES the other stories by slug but does not edit them. |
| **Phase B.1 / C.* / D.* future story-authoring prompts** | **YES**, this note pre-publishes their slugs for cross-link convenience. |

### Dependencies

- **Prerequisite**: stories 010 / 011 / 012 / 013 / 014 authored on
  `origin/main` (PROMPT 1044 batch). If a producer activates this
  story before the other five land, the sequencing note's inventory
  rows for missing stories are marked `not-yet-authored` rather
  than carrying dangling references.
- **Unblocks**: Sprint 16 / 17 / 18 activation planning.

---

## Worker Contract (for `/dev-story`)

The future `/dev-story` worker MUST:

1. Run `git checkout -b work/s16-ui-architecture-sequencing` from
   `origin/main`.
2. Read PROMPT 1034 audit + PROMPT 1035 audit + stories 009-014.
3. Author the sequencing note per AC1-AC4 + AC6.
4. Add the one-line cross-link per AC5.
5. Verify only the two doc files changed per AC7-AC8.
6. Push `work/s16-ui-architecture-sequencing`.

The worker MUST NOT:

- Modify any code / test / spec body file.
- Modify any sprint-plan / status / session-state file.
- Run `/dev-story` workflows beyond `/story-readiness` (which is
  appropriate for paperwork rows).
- Run `/story-done` / `/smoke-check` / `/team-qa` / `/gate-check` /
  `/release-check` / `/qa-plan`.

---

`015: S16-TD-UI-ARCHITECTURE-SEQUENCING-001: DRAFT`
