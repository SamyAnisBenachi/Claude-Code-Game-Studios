# Story 001: S13-UI-AUDIT-ROADMAP-PREP-001 -- PROMPT 802 Expert UI Layout Audit Roadmap Prep

> **Epic**: UI Clean-Pass
> **Story ID**: S13-UI-AUDIT-ROADMAP-PREP-001
> **Status**: Draft -- Sprint 13 candidate (Nice to Have); NOT activated;
> Sprint 12 closed-with-conditions per PROMPT 817
> **Layer**: UX / Producer planning -- roadmap-prep documentation only
> **Type**: Documentation only -- no UI overhaul attempted
> **Sprint**: Sprint 13 candidate (PROMPT 802 §3 per-surface verdicts; §6
> sequenced repair plan; §11 backlog-vs-recommendation matrix); NOT
> activated
> **Authored**: 2026-05-14 by PROMPT 819
> **Authoring source-of-truth**: `origin/main@be69f5c` (PROMPT 818
> `/sprint-plan sprint-13` DRAFT)

---

## Status / No-Claim Banner

This story is authored as a Sprint 13 candidate. Sprint 13 is **NOT**
activated by PROMPT 819. Sprint 12 is closed-with-conditions per PROMPT
817 and is not changed by this authoring run.

PROMPT 819 (this authoring run) does NOT:

- Activate Sprint 13.
- Modify `production/sprint-status.yaml`.
- Modify `production/sprints/sprint-13.md` or any other sprint plan file.
- Modify `production/stage.txt` (remains `Polish`).
- Modify any `production/session-state/*` file.
- Modify any QA-plan / smoke / Team-QA / gate-check / release-check
  artifact.
- Run `/story-readiness`, `/dev-story`, `/story-done`, `/smoke-check`,
  `/team-qa`, `/gate-check`, `/release-check`, or `/qa-plan` on this
  story.
- Modify any code under `client/`, `server/`, `shared/`, or `tests/`.
- Activate any of the 14 PROMPT 802 candidate UI repair slugs.
- Attempt any UI overhaul.
- Retry the PROMPT 761 Polish->Release gate-check.

This story does **not** claim: public release readiness, release-candidate
readiness, full game completion, broad / Standard-tier accessibility
completion (`QA-COND-0005`), playtest / fun-hypothesis validation
(`QA-COND-0006`), full playable-client manual QA, two-client GAME_OVER
closure (`S8-QA-001-W1`), or final-art / asset-production completion.

Sprint 10 / Sprint 11 / Sprint 12 dispositions unchanged. PROMPT 761
Polish->Release gate-check FAIL evidence preserved. PROMPT 685 8-story
milestone backlog disposition preserved unchanged.

**This story is paperwork only. NO UI OVERHAUL IS ATTEMPTED IN
SPRINT 13.** The 14 PROMPT 802 candidate slugs are NOT activated by
this story.

---

## Source Finding

- `reports/PROMPT-802-Expert-UI-Layout-Audit-And-Repair-Roadmap.md`:
  - §3 per-surface verdicts (hand UI, HUD, shop/auction UI, board
    rendering, lobby).
  - §6 sequenced repair plan listing 14 candidate UI repair slugs.
  - §11 backlog-vs-recommendation matrix reconciling the 14 PROMPT
    802 slugs against the existing PROMPT 685 8-story milestone
    backlog.

---

## Problem Class / Prevention Target

**Defect class**: PROMPT 802 surfaced 14 distinct UI-layout repair
candidates across multiple surfaces. Activating them piecemeal
risks:

- Re-introducing visual inconsistency between surfaces (each story
  fixes one surface; cross-surface coherence not asserted).
- Conflicting with the older PROMPT 685 8-story milestone backlog
  (some PROMPT 685 rows are now stale; some are still valid; some
  are subsumed by PROMPT 802 candidates).
- Pulling in Standard-tier accessibility work (`QA-COND-0005`) by
  accident through UI repairs that touch contrast / text-size /
  focus-order.

**Prevention target**: A roadmap-prep note at
`docs/ux/ui-clean-pass-roadmap.md` (NEW) that:

- Reconciles the 14 PROMPT 802 candidate slugs against the
  existing PROMPT 685 8-story milestone backlog. For each
  PROMPT 685 row, one of: `subsumed-by <PROMPT 802 slug>`,
  `still valid (sequence after PROMPT 802)`, `obsolete`.
- Sequences the 14 PROMPT 802 candidates into the correct
  Sprint 14+ pull-in order, preserving the §6 dependency
  ordering.
- Names the 3-4 highest-impact "must land before any polished
  friend-game-product showcase" rows for Sprint 14 Must Have
  framing.
- Explicitly preserves `PAW-TD-*-a` placeholder-art accept-risk,
  `QA-COND-0005` Standard-tier-accessibility accepted-risk, and
  `QA-COND-0006` playtest accepted-risk.
- Names the friend-game-scope vs Standard-tier-accessibility
  scope boundary so Sprint 14+ activation does not silently
  expand the claim.

**Sprint 13 does NOT activate any of the 14 PROMPT 802 candidate
slugs.** Activation happens in a separate `/sprint-plan` revision
in Sprint 14+.

---

## Context

### Existing surface

- **`reports/PROMPT-802-Expert-UI-Layout-Audit-And-Repair-Roadmap.md`**:
  the source audit (read-only by this story).
- **PROMPT 685 8-story milestone backlog**: **no
  `reports/PROMPT-685*` file exists on `origin/main` at PROMPT 823
  readiness verification.** The implementing worker MUST locate
  the canonical 8-story milestone backlog wherever it actually
  lives at implementation time (likely candidates: a
  milestone-level doc under `production/milestones/`, an older
  sprint plan under `production/sprints/`, a backlog row in
  `production/sprint-status.yaml`, or a session-state archive)
  and record the canonical source in roadmap evidence; **or**, if
  no canonical source exists, the worker proceeds with AC3 by
  marking each formerly-PROMPT-685 row obsolete with rationale,
  citing the missing source.
- **`production/epics/hand-ui/`, `production/epics/hud/`,
  `production/epics/shop-auction-ui/`, `production/epics/board-rendering/`,
  `production/epics/playable-client/` (lobby surface)**: target
  epics for future PROMPT 802 candidate stories. **Not modified**
  by this story.

### GDD / ADR / TR trace

- **GDD**: `design/gdd/hand-ui.md`, `design/gdd/hud.md`,
  `design/gdd/shop-auction-ui.md`, `design/gdd/board-rendering.md`
  (cross-cut; read-only).
- **ADR-021** (Presentation Layer Architecture): roadmap rows
  reconcile against canonical `PresentationPlugin` composition
  order.
- **ADR-002** (Client-Server Authority): no optimistic client-side
  authority introduced.
- **TR registry**: no new TR (roadmap-prep only).

### Engine / skills

- **Engine**: Bevy 0.18 (Rust). No `.rs` edits expected.
- **Mandatory skills**: none (doc-only).

### Control Manifest Rules

- Required: Roadmap note authored at
  `docs/ux/ui-clean-pass-roadmap.md` (NEW).
- Required: 14 PROMPT 802 slugs sequenced.
- Required: PROMPT 685 8-story backlog reconciled against
  PROMPT 802 candidates.
- Required: 3-4 highest-impact rows named for Sprint 14 Must Have
  framing.
- Required: Accepted-risk dispositions (`PAW-TD-*-a`,
  `QA-COND-0005`, `QA-COND-0006`) explicitly preserved.
- Forbidden: Activating any of the 14 PROMPT 802 candidate slugs.
- Forbidden: Modifying any production code under `client/`,
  `server/`, `shared/`, or `tests/`.
- Forbidden: Advancing or claiming `QA-COND-0005` /
  `QA-COND-0006`.
- Forbidden: Closing PROMPT 685 backlog disposition under this
  story (only documenting subsumption status).

---

## Story Classification

**Story type**: Documentation only.

This is **NOT** a:

- UI implementation story.
- Activation prompt for any of the 14 PROMPT 802 candidates.
- Closure of PROMPT 685.

---

## Acceptance Criteria

All criteria are independently checkable.

- [ ] **AC1 -- Roadmap note authored**: GIVEN the story commit,
  WHEN inspected, THEN `docs/ux/ui-clean-pass-roadmap.md` exists
  and is the new artifact for this story.

- [ ] **AC2 -- 14 PROMPT 802 slugs sequenced**: GIVEN the
  roadmap note, WHEN inspected, THEN it lists all 14 PROMPT 802
  candidate slugs from `reports/PROMPT-802-*` §6 in their
  recommended pull-in order, with the §6 sequencing dependencies
  preserved.

- [ ] **AC3 -- PROMPT 685 backlog reconciled**: GIVEN the roadmap
  note, WHEN inspected, THEN one of:
  - (a) The implementing worker has located the canonical
    PROMPT 685 8-story milestone backlog (no `reports/PROMPT-685*`
    file exists on `main` at PROMPT 823 verification; canonical
    source must be located wherever it actually lives -- likely a
    milestone-level doc, older sprint plan, `sprint-status.yaml`
    row, or session-state archive). For each row in the located
    backlog, the note records one disposition:
    - `subsumed-by <PROMPT 802 slug>` (with the slug named).
    - `still valid (sequence after PROMPT 802)`.
    - `obsolete (rationale)`.
    The canonical source path is recorded in roadmap evidence.
  - (b) **No canonical source located**: the roadmap note
    explicitly records that no canonical PROMPT 685 source exists
    on `main` at implementation time, and each formerly-PROMPT-685
    row is marked `obsolete (rationale: PROMPT 685 backlog
    not-findable per PROMPT 823 / 824 hygiene; superseded by
    PROMPT 802 audit)` with a brief one-line rationale.

- [ ] **AC4 -- 3-4 highest-impact rows named**: GIVEN the
  roadmap note, WHEN inspected, THEN exactly 3 or 4 of the 14
  PROMPT 802 candidate slugs are named as "must land before any
  polished friend-game-product showcase" rows for Sprint 14 Must
  Have framing, each with a one-sentence rationale.

- [ ] **AC5 -- Accept-risk dispositions preserved**: GIVEN the
  roadmap note, WHEN inspected, THEN it explicitly preserves:
  - `PAW-TD-*-a` placeholder-art accept-risk across
    PAW-002..PAW-006.
  - `QA-COND-0005` Standard-tier-accessibility accepted-risk
    (friend-game scope only).
  - `QA-COND-0006` playtest / fun-hypothesis validation
    accepted-risk.

- [ ] **AC6 -- Friend-game-scope boundary named**: GIVEN the
  roadmap note, WHEN inspected, THEN it explicitly names the
  friend-game-scope vs Standard-tier-accessibility scope boundary
  so Sprint 14+ activation does not silently expand the claim.

- [ ] **AC7 -- 14 candidate slugs NOT activated**: GIVEN the
  story commit, WHEN `production/sprint-status.yaml`,
  `production/sprints/sprint-13.md`, and any new story files
  under `production/epics/` are inspected, THEN none of the 14
  PROMPT 802 candidate slugs has an activation row, an
  active-sprint row, or a story file authored under this prompt.

- [ ] **AC8 -- No production-source change lands**: GIVEN the
  story commit, WHEN `git diff` is inspected, THEN no file under
  `client/`, `server/`, `shared/`, or `tests/` is modified.

- [ ] **AC9 -- Sprint 13 disposition preserved**: GIVEN the story
  commit, WHEN `production/sprint-status.yaml`,
  `production/sprints/sprint-13.md`, `production/stage.txt`, and
  PROMPT 761 gate-check artifact are diffed, THEN none of them
  are modified by this story.

- [ ] **AC10 -- No-claim restatement embedded**: GIVEN the
  roadmap note, WHEN inspected, THEN it includes the verbatim
  "Status / No-Claim Banner" no-claim restatement plus the
  explicit "Sprint 13 does NOT attempt the full UI overhaul"
  line.

---

## Likely Files

| Path | Anticipated change |
|------|--------------------|
| `docs/ux/ui-clean-pass-roadmap.md` | NEW roadmap note. |
| This story file | Status update on `/story-done`. |

This table is a planning estimate. The implementation prompt is
authoritative for the realised set.

---

## Required Skills

- None (doc-only). UX-designer agent (`ux-designer`) is the natural
  worker; producer + ux-designer collaboration is implied by the
  Sprint 13 plan agent/owner column.

---

## Evidence Path

The roadmap note **is** the artifact for this story.

**Required note content** (deferred to implementation prompt):

- 14 PROMPT 802 slugs sequenced (per AC2).
- PROMPT 685 backlog reconciliation (per AC3).
- 3-4 highest-impact rows named (per AC4).
- Accept-risk dispositions preserved (per AC5).
- Friend-game-scope boundary named (per AC6).
- No-claim restatement (per AC10).

---

## Regression Commands Expected

For the implementation prompt:

- `git diff <pre-impl-sha>..<impl-sha> -- 'client/**' 'server/**' 'shared/**' 'tests/**'`
  (verifies AC8: zero production-source change)
- `git diff --check origin/main...HEAD`
- `git diff --cached --check`

No `cargo` command is required by this story.

---

## Out of Scope

- Activation of any of the 14 PROMPT 802 candidate slugs.
- Authoring of any of the 14 PROMPT 802 candidate story files
  (those land in their respective epic folders during a Sprint
  14+ activation prompt).
- Any UI implementation work.
- Closing PROMPT 685 8-story backlog (only documenting
  subsumption status).
- Advancing `QA-COND-0005` Standard-tier-accessibility or
  `QA-COND-0006` playtest validation.
- Sprint 13 stage advance, `S8-QA-001-W1` closure, or
  Polish->Release gate-check retry.
- Final-art / asset-production scope.
- No `/dev-story`, `/story-done`, `/smoke-check`, `/team-qa`,
  `/gate-check`, `/release-check`, or `/qa-plan` run under the
  authoring prompt.

---

## Dependency Notes Against Sprint 13 Active Scope

- Doc-only; no file-scope collision with any Sprint 13 Must Have
  or Should Have row.
- Sequences independently any time during Sprint 13.
- Sprint 14 activation prompt can pull the named 3-4
  highest-impact rows directly from the roadmap note authored by
  this story.
