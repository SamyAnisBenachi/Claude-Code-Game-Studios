# Story 001: S13-UI-AUDIT-ROADMAP-PREP-001 -- PROMPT 802 Expert UI Layout Audit Roadmap Prep

> **Epic**: UI Clean-Pass
> **Story ID**: S13-UI-AUDIT-ROADMAP-PREP-001
> **Status**: Done — closed by PROMPT 840 `/story-done` on 2026-05-14
> against `origin/main@0d59ba3` (PROMPT 838 worker commit `825d41d` →
> PROMPT 839 integration commit `0d59ba3` on `origin/main`). Sprint 13
> remains `active`; PROMPT 840 is a per-story closure paperwork run only.
> **Layer**: UX / Producer planning -- roadmap-prep documentation only
> **Type**: Documentation only -- no UI overhaul attempted
> **Sprint**: Sprint 13 Nice to Have (activated by PROMPT 826; closed by
> PROMPT 840). PROMPT 802 §3 per-surface verdicts; §6 sequenced repair
> plan; §11 backlog-vs-recommendation matrix all reconciled in
> `docs/ux/ui-clean-pass-roadmap.md`.
> **Authored**: 2026-05-14 by PROMPT 819
> **Authoring source-of-truth**: `origin/main@be69f5c` (PROMPT 818
> `/sprint-plan sprint-13` DRAFT)
> **Closed**: 2026-05-14 by PROMPT 840 `/story-done` at
> `origin/main@0d59ba3`

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

- [x] **AC1 -- Roadmap note authored**: GIVEN the story commit,
  WHEN inspected, THEN `docs/ux/ui-clean-pass-roadmap.md` exists
  and is the new artifact for this story.

- [x] **AC2 -- 14 PROMPT 802 slugs sequenced**: GIVEN the
  roadmap note, WHEN inspected, THEN it lists all 14 PROMPT 802
  candidate slugs from `reports/PROMPT-802-*` §6 in their
  recommended pull-in order, with the §6 sequencing dependencies
  preserved.

- [x] **AC3 -- PROMPT 685 backlog reconciled**: GIVEN the roadmap
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

- [x] **AC4 -- 3-4 highest-impact rows named**: GIVEN the
  roadmap note, WHEN inspected, THEN exactly 3 or 4 of the 14
  PROMPT 802 candidate slugs are named as "must land before any
  polished friend-game-product showcase" rows for Sprint 14 Must
  Have framing, each with a one-sentence rationale.

- [x] **AC5 -- Accept-risk dispositions preserved**: GIVEN the
  roadmap note, WHEN inspected, THEN it explicitly preserves:
  - `PAW-TD-*-a` placeholder-art accept-risk across
    PAW-002..PAW-006.
  - `QA-COND-0005` Standard-tier-accessibility accepted-risk
    (friend-game scope only).
  - `QA-COND-0006` playtest / fun-hypothesis validation
    accepted-risk.

- [x] **AC6 -- Friend-game-scope boundary named**: GIVEN the
  roadmap note, WHEN inspected, THEN it explicitly names the
  friend-game-scope vs Standard-tier-accessibility scope boundary
  so Sprint 14+ activation does not silently expand the claim.

- [x] **AC7 -- 14 candidate slugs NOT activated**: GIVEN the
  story commit, WHEN `production/sprint-status.yaml`,
  `production/sprints/sprint-13.md`, and any new story files
  under `production/epics/` are inspected, THEN none of the 14
  PROMPT 802 candidate slugs has an activation row, an
  active-sprint row, or a story file authored under this prompt.

- [x] **AC8 -- No production-source change lands**: GIVEN the
  story commit, WHEN `git diff` is inspected, THEN no file under
  `client/`, `server/`, `shared/`, or `tests/` is modified.

- [x] **AC9 -- Sprint 13 disposition preserved**: GIVEN the story
  commit, WHEN `production/sprint-status.yaml`,
  `production/sprints/sprint-13.md`, `production/stage.txt`, and
  PROMPT 761 gate-check artifact are diffed, THEN none of them
  are modified by this story.

- [x] **AC10 -- No-claim restatement embedded**: GIVEN the
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

---

## Authoring / Implementation / Closure Trail

- **PROMPT 819** (2026-05-14) — Authoring. Story file created at
  `production/epics/ui-clean-pass/story-001-prompt-802-audit-roadmap-prep.md`
  against `origin/main@be69f5c` (PROMPT 818 Sprint 13 plan DRAFT).
  No sprint-status flip, no activation, no UI overhaul. Paperwork only.
- **PROMPT 823** (2026-05-14) — `/story-readiness` rerun batch.
  Verdict **READY** (advisory: `reports/PROMPT-685-*` does not exist
  on `origin/main`; story has escape hatch *"or canonical
  equivalent"*; worker is authorised to locate the canonical PROMPT
  685 backlog wherever it actually lives or mark each row obsolete).
- **PROMPT 826** (2026-05-14) — Sprint 13 activation. Row
  `S13-UI-AUDIT-ROADMAP-PREP-001` recorded in
  `production/sprint-status.yaml` Sprint 13 Nice to Have block with
  `status: ready`; `sprint_13_activation:` snapshot block records
  `status_at_activation: "ready"` and PROMPT 823 readiness verdict.
- **PROMPT 827** (2026-05-14) — Sprint 13 QA plan
  (`production/qa/qa-plan-sprint-13.md` NEW). Story 001
  classification: documentation only; reviewer sign-off; no automated
  test required (the roadmap *is* the artifact). No `/dev-story`
  blocker beyond the QA plan existence gate.
- **PROMPT 838** (2026-05-14) — Worker execution (paperwork-only doc
  authoring). Worker branch `work/s13-ui-audit-roadmap-prep`; worker
  commit `825d41d` (`docs(ux): author UI clean-pass roadmap
  reconciling PROMPT 802 vs PROMPT 685 (PROMPT 838)`). Authored
  `docs/ux/ui-clean-pass-roadmap.md` (NEW; +411 / -0 lines). PROMPT
  685 canonical source located at
  `production/sprints/sprint-11.md:279-286` and
  `production/sprints/sprint-12.md:385-392` (identical content;
  AC3 satisfied via option (a)). Worker pushed worker branch only
  (`origin/work/s13-ui-audit-roadmap-prep`); worker did NOT push
  `main`. Worker report:
  `reports/PROMPT-838-S13-UI-Audit-Roadmap-Prep.md`.
- **PROMPT 839** (2026-05-14) — Integration. Cherry-pick of worker
  tip `825d41d` onto `origin/main@4f7ba78` produced integration
  commit `0d59ba3` (tree byte-identical to worker tip; +411 / -0;
  single file `docs/ux/ui-clean-pass-roadmap.md`). Push: fast-forward
  `4f7ba78..0d59ba3` to `origin/main` (no force, no merge commit).
  Integration verification: `git diff --check` clean;
  `git diff --cached --check` clean; no forbidden paths
  (`client/`, `server/`, `shared/`, `tests/`,
  `production/sprint-status.yaml`, `production/session-state/`,
  `production/stage.txt`) touched. Integration report:
  `reports/PROMPT-839-S13-UI-Audit-Roadmap-Prep-Integration.md`.
  PROMPT 839 did NOT run `/story-done`.
- **PROMPT 840** (2026-05-14) — `/story-done` closure paperwork (this
  closure). Verified each AC against `origin/main@0d59ba3` (PROMPT
  838 worker + PROMPT 839 integration); flipped Status header Draft
  -> Done with closure context; flipped AC1-AC10 checkboxes `[ ]` ->
  `[x]`; appended this Authoring / Implementation / Closure Trail
  section; flipped `production/sprint-status.yaml` Sprint 13 Nice to
  Have row `S13-UI-AUDIT-ROADMAP-PREP-001` `status: ready -> done`
  with `completed: 2026-05-14`, `worker_prompt: 838`, `worker_commit:
  825d41dd021379b4c0df25bec344525b4f12f43d`, `integration_prompt:
  839`, `integration_commit:
  0d59ba3c284f931e471db2a20f2058266f4ab707`, `story_done_prompt:
  840`, `acceptance_evidence: docs/ux/ui-clean-pass-roadmap.md`;
  refreshed top-level `updated:` annotation; appended
  `sprint_13_story_done:` entry at end of file as a sibling to the
  prior PROMPT 833 / PROMPT 835 entries. Prepended PROMPT 840
  banners to `production/session-state/active.md` and
  `production/session-state/codex-orchestrator-state.md`. Paperwork
  only: no `/dev-story`, `/story-readiness`, `/smoke-check`,
  `/team-qa`, `/gate-check`, `/release-check`, `/qa-plan` invoked;
  no production code under `client/` / `server/` / `shared/` /
  `tests/` modified; no `production/stage.txt`,
  `production/sprints/sprint-13.md`, `production/sprints/sprint-12.md`,
  `production/qa/qa-plan-sprint-13.md`, or
  `production/gate-checks/*` modified; no cargo invoked. Root
  checkout dirt (` M .claude/settings.json`, untracked
  `Dtmpworkspace-test-output.txt`, untracked
  `production/session-state/autonomous-monitor-task.md`) preserved
  untouched. PROMPT 840 final report:
  `reports/PROMPT-840-S13-UI-Audit-Roadmap-Prep-Story-Done.md`
  (NOT staged or committed; `reports/` is gitignored).

### Conditions carried forward unchanged by PROMPT 840

- `S8-QA-001-W1` manual/browser two-client GAME_OVER gap remains
  **OPEN**. Story 017 AC12 forbid-auto-closure preserved.
- `QA-COND-0005` Standard-tier accessibility remains **accepted-risk**
  (friend-game scope only). The lobby `LOBBY_BUTTON_HEIGHT = 30.0`
  L5 hit-target defect (PROMPT 802 §3.1 L5) preserved as accepted-
  risk; the roadmap explicitly does not advance this.
- `QA-COND-0006` playtest / fun-hypothesis validation remains
  **accepted-risk / deferred**.
- `PAW-TD-*-a` placeholder-art accept-risk preserved across
  `PAW-002..PAW-006`. UI clean-pass roadmap is layout / composition
  / hierarchy / typography / z-order work only; final-art is out
  of scope.
- `TQ-S12-C1..C7` (all 7 Sprint 12 Team-QA conditions) preserved
  verbatim.
- PROMPT 683-era runtime divergence question preserved (folded into
  Sprint 12 story 019 cannot-reproduce closure; third same-scope
  retest NOT authorised per `TQ-S12-C2`).
- PROMPT 761 Polish->Release gate-check **FAIL** preserved at
  `production/gate-checks/gate-polish-release-2026-05-12.md`. No
  retry attempted.
- Sprint 12 disposition (`closed-with-conditions` per PROMPT 817)
  preserved unchanged.
- Sprint 11 / Sprint 10 closeouts preserved unchanged.
- Story 019 (Sprint 12 hand-ui) terminal disposition preserved —
  underlying drag-runtime bug NOT claimed fixed.
- PROMPT 685 8-story milestone backlog disposition is **NOT closed**
  by PROMPT 840. Only its subsumption status was documented (the
  roadmap records `subsumed-by` PROMPT 802 for all 8 rows).
- The 14 PROMPT 802 candidate slugs remain **NOT activated**. None
  of them has a Sprint 13 activation row, an active-sprint row, or
  a new story file authored under PROMPT 840.

### Explicitly NOT claimed by PROMPT 840

- Public release readiness.
- Release-candidate readiness.
- Full game completion.
- Broad / Standard-tier accessibility completion.
- Playtest / fun-hypothesis validation.
- Full playable-client manual QA.
- Two-client GAME_OVER closure (`S8-QA-001-W1`).
- Final-art / asset-production completion.
- Polish->Release gate-check retry.
- Stage advance from Polish to Release.
- Underlying drag-runtime bug fix.
- Full UI clean-pass repair.
- Closure of PROMPT 685.
- Activation of any of the 14 PROMPT 802 candidate slugs.
- Sprint 13 close-out (Sprint 13 remains `active`; only 3 of 19
  rows closed after PROMPT 840: stories 023 (PROMPT 835), 001
  server (PROMPT 833), and 001 ui-clean-pass (PROMPT 840 — this
  row); 6 Must Have + 4 of 6 Should Have + 6 of 7 Nice to Have
  rows remain).
