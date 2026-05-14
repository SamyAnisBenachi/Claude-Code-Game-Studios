# Story 003: S11-OPS-ORCHESTRATOR-LOCK-001 -- Orchestrator-Root Concurrent-Session Lock Pattern

> **Epic**: DevOps (Operational Hardening)
> **Story ID**: S11-OPS-ORCHESTRATOR-LOCK-001
> **Status**: Draft -- Sprint 13 candidate (Nice to Have); NOT activated;
> Sprint 12 closed-with-conditions per PROMPT 817
> **Layer**: DevOps / Orchestration -- documentation only
> **Type**: Documentation only -- no code lands
> **Sprint**: Sprint 13 candidate (Sprint 12 close-out deferral; reinforced
> by the 2026-05-13 override rule "only one shared-status writer at a
> time"); NOT activated
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
- Implement any actual lock file or runtime tooling.
- Retry the PROMPT 761 Polish->Release gate-check.

This story does **not** claim: public release readiness, release-candidate
readiness, full game completion, broad / Standard-tier accessibility
completion (`QA-COND-0005`), playtest / fun-hypothesis validation
(`QA-COND-0006`), full playable-client manual QA, two-client GAME_OVER
closure (`S8-QA-001-W1`), or final-art / asset-production completion.

Sprint 10 / Sprint 11 / Sprint 12 dispositions unchanged. PROMPT 761
Polish->Release gate-check FAIL evidence preserved.

**This story documents a pattern only. NO CODE OR LOCK FILE LANDS.**

---

## Source Finding

- Sprint 11 Wave 12 backlog observed that two parallel orchestrator
  sessions mutating root checkout `main` HEAD concurrently produced
  rebase / merge conflicts during the close-out phase.
- The 2026-05-13 override rule "only one shared-status writer at a
  time per coordination window" was introduced as the operational
  fix; this story documents the underlying detection / avoidance
  pattern so the rule has a backing reference document.
- Sprint 12 close-out (PROMPT 817) deferred the row forward to
  Sprint 13 planning.

---

## Problem Class / Prevention Target

**Defect class**: Two parallel orchestrator sessions can each
attempt to mutate root checkout `main` HEAD concurrently -- e.g.,
both running close-out paperwork against
`production/session-state/active.md`, `production/sprint-status.yaml`,
or `production/session-state/codex-orchestrator-state.md`. The
collision produces:

- Rebase / merge conflicts.
- Lost paperwork (one session's update overwrites the other's
  without a merge).
- Confused audit trail (the conflict resolution itself is not
  itself recorded in either session's report).

**Prevention target**: A lock-pattern doc at
`.octogent/orchestrator-lock.md` (NEW) (or appended to the existing
`docs/octogent-integration.md`; final location chosen by the
implementing worker) that:

- Documents the failure mode (with the Sprint 11 Wave 12
  cross-link).
- Documents the **detection** pattern: how a starting session
  detects that another session is already a "shared-status writer"
  in the current coordination window.
- Documents the **avoidance** pattern: e.g., a lock-file convention
  (`production/session-state/.lock-shared-writer`) with timestamp
  and session ID, plus a write-acquire protocol (read-only sessions
  don't acquire; shared-status writers must acquire before any
  paperwork-write turn).
- Documents the **release** pattern: lock file is deleted at end
  of paperwork-write turn (or expires after a documented timeout).
- Cross-links to the 2026-05-13 override rule and to the current
  GCS Orchestrator Contract block in
  `production/session-state/codex-orchestrator-state.md`.

**No code, lock file, or runtime tooling lands under this story.**
A separate follow-on story can implement the actual lock file
convention (or extend the orchestrator runtime) if the doc surfaces
a recommendation worth wiring up.

---

## Context

### Existing surface

- **`production/session-state/active.md`**: Lanes-and-Lies session
  state (shared-status writer surface #1).
- **`production/sprint-status.yaml`**: top-level sprint state
  (shared-status writer surface #2).
- **`production/session-state/codex-orchestrator-state.md`**:
  orchestrator contract + recent decisions (shared-status writer
  surface #3).
- **2026-05-13 override rule**: operational rule already in force;
  this doc backs it.

### Engine / skills

- **Engine**: N/A (operational pattern; not a game engine concern).
- **Mandatory skills**: none (doc-only).

### Control Manifest Rules

- Required: Doc names the three shared-status writer surfaces.
- Required: Doc documents detection / avoidance / release patterns.
- Required: Doc cross-links to the 2026-05-13 override rule.
- Forbidden: Implementing the actual lock file under this story.
- Forbidden: Adding any runtime tooling, scripts, or hooks.

---

## Story Classification

**Story type**: Documentation only.

This is **NOT** a:

- Code change.
- Hook / tooling change.
- Runtime change.

---

## Acceptance Criteria

All criteria are independently checkable.

- [ ] **AC1 -- Doc authored**: GIVEN the story commit, WHEN
  inspected, THEN exactly one new (or appended) doc file exists at
  `.octogent/orchestrator-lock.md` (or as a clearly-named section
  appended to `docs/octogent-integration.md` per worker's
  judgement) documenting the lock pattern.

- [ ] **AC2 -- Shared-status writer surfaces enumerated**: GIVEN
  the doc, WHEN inspected, THEN it enumerates the three current
  shared-status writer surfaces (`production/session-state/active.md`,
  `production/sprint-status.yaml`,
  `production/session-state/codex-orchestrator-state.md`).

- [ ] **AC3 -- Detection / avoidance / release patterns
  documented**: GIVEN the doc, WHEN inspected, THEN it documents
  all three pattern phases (detection, avoidance, release) for
  the proposed lock-file convention.

- [ ] **AC4 -- Cross-link to 2026-05-13 override rule**: GIVEN
  the doc, WHEN inspected, THEN it cross-links to the override
  rule "only one shared-status writer at a time per coordination
  window" in
  `production/session-state/codex-orchestrator-state.md` (or
  wherever the override rule lives at Sprint 13 activation HEAD).

- [ ] **AC5 -- No code / lock file lands**: GIVEN the story
  commit, WHEN `git diff` is inspected, THEN no file under
  `client/`, `server/`, `shared/`, `tests/`, `.cargo/`,
  `.github/`, or any build script is modified, and no lock file
  is created. Only the new doc (and optionally this story file's
  status) is touched.

- [ ] **AC6 -- Sprint 13 disposition preserved**: GIVEN the story
  commit, WHEN `production/sprint-status.yaml`,
  `production/sprints/sprint-13.md`, `production/stage.txt`, and
  PROMPT 761 gate-check artifact are diffed, THEN none of them
  are modified by this story.

- [ ] **AC7 -- Single optional follow-on named (or explicitly
  none)**: GIVEN the doc, WHEN inspected, THEN it either names
  exactly one recommended follow-on story slug to implement the
  lock-file convention as runtime tooling, **or** explicitly
  states "no implementation follow-on recommended; the pattern is
  enforced operationally via the 2026-05-13 override rule".

- [ ] **AC8 -- No-claim restatement embedded**: GIVEN the doc,
  WHEN inspected, THEN it includes the verbatim "Status / No-Claim
  Banner" no-claim restatement.

---

## Likely Files

| Path | Anticipated change |
|------|--------------------|
| `.octogent/orchestrator-lock.md` (NEW) **or** an appended section in `docs/octogent-integration.md` | NEW doc / new section. |
| This story file | Status update on `/story-done`. |

---

## Required Skills

- None (doc-only).

---

## Evidence Path

The doc itself **is** the artifact for this story.

**Required doc content** (deferred to implementation prompt):

- Failure mode description (per AC1).
- Three shared-status writer surfaces (per AC2).
- Detection / avoidance / release patterns (per AC3).
- Cross-link to 2026-05-13 override rule (per AC4).
- Single optional follow-on or explicit none (per AC7).
- No-claim restatement (per AC8).

---

## Regression Commands Expected

For the implementation prompt:

- `git diff <pre-impl-sha>..<impl-sha> -- 'client/**' 'server/**' 'shared/**' 'tests/**' 'Cargo.toml' '.cargo/**' '.github/**' '*.sh' '*.ps1'`
  (verifies AC5: zero code/config/CI change)
- `git diff --check origin/main...HEAD`
- `git diff --cached --check`

No `cargo` command is required by this story.

---

## Out of Scope

- Implementing the actual lock-file or runtime tooling.
- Modifying any orchestrator script or hook.
- Sprint 13 activation, `S8-QA-001-W1` closure, or Polish->Release
  gate-check retry.
- `QA-COND-0005` / `QA-COND-0006` advancement.
- No `/dev-story`, `/story-done`, `/smoke-check`, `/team-qa`,
  `/gate-check`, `/release-check`, or `/qa-plan` run under the
  authoring prompt.

---

## Dependency Notes Against Sprint 13 Active Scope

- Doc-only; no file-scope collision with any Sprint 13 Must Have or
  Should Have row.
- Sequences independently any time during Sprint 13.
