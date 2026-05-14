# Story 004: S11-OPS-GH-CLI-001 -- `gh` CLI Installation Note

> **Epic**: DevOps (Operational Hardening)
> **Story ID**: S11-OPS-GH-CLI-001
> **Status**: Draft -- Sprint 13 candidate (Nice to Have); NOT activated;
> Sprint 12 closed-with-conditions per PROMPT 817
> **Layer**: DevOps -- onboarding documentation only
> **Type**: Documentation only -- no tooling changes land
> **Sprint**: Sprint 13 candidate (Sprint 12 close-out deferral; Sprint 11
> Wave 12 backlog: `gh` absent 3+ times during orchestrator workflows);
> NOT activated
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
- Install or change `gh` or any other tool.
- Retry the PROMPT 761 Polish->Release gate-check.

This story does **not** claim: public release readiness, release-candidate
readiness, full game completion, broad / Standard-tier accessibility
completion (`QA-COND-0005`), playtest / fun-hypothesis validation
(`QA-COND-0006`), full playable-client manual QA, two-client GAME_OVER
closure (`S8-QA-001-W1`), or final-art / asset-production completion.

Sprint 10 / Sprint 11 / Sprint 12 dispositions unchanged. PROMPT 761
Polish->Release gate-check FAIL evidence preserved.

**This story is documentation only. NO TOOLING CHANGES LAND.**

---

## Source Finding

- Sprint 11 Wave 12 backlog logged the GitHub CLI (`gh`) being
  absent from the dev machine 3+ times during orchestrator
  workflows (e.g., PR creation, issue triage from CLI), forcing
  worker fall-back to manual GitHub web UI.
- Sprint 11 close-out raised `S11-OPS-GH-CLI-001` as a
  Nice-to-Have onboarding-doc fix.
- Sprint 12 close-out (PROMPT 817) deferred the row forward to
  Sprint 13 planning.

---

## Problem Class / Prevention Target

**Defect class**: The dev-environment onboarding doc does not
name `gh` as required tooling; new worktrees / new sessions hit
"`gh` not found" the first time they need it, forcing a context
switch to install or to fall back to web UI.

**Path-existence note (PROMPT 823 / PROMPT 824 verification)**:
`docs/setup/` does **NOT** exist on `origin/main` at PROMPT 823
readiness verification. `docs/setup/dev-environment.md` is therefore
a **target path**, not an existing file. The implementing worker
chooses one of two paths and records the chosen path in story
evidence:

- (a) Create `docs/setup/` and `docs/setup/dev-environment.md`
  in the same doc-only commit as the `gh` paragraph; **or**
- (b) Amend a canonical sibling doc that already exists on `main`
  (e.g. `docs/WORKFLOW-GUIDE.md`, `docs/octogent-integration.md`,
  `CONTRIBUTING.md`, or `docs/onboarding.md` if any of these are
  the project's canonical onboarding doc at implementation time).

The implementing worker picks (a) **or** (b) -- not both -- and
records the rationale in story evidence.

**Prevention target**: A one-paragraph note in the chosen target
doc (per (a) or (b) above) that:

- Names `gh` as required tooling for orchestrator and producer
  workflows.
- Provides the platform-appropriate install commands (Windows
  `winget`, macOS `brew`, Linux package manager) -- the
  implementing worker chooses scope based on supported dev
  platforms.
- Optionally names a one-line auth command
  (`gh auth login`) with the appropriate scopes.

**No actual installation or tooling change lands under this
story.** The note is the artifact.

---

## Context

### Existing surface

- **`docs/setup/dev-environment.md`**: **target path, not an
  existing file** as of PROMPT 823 readiness verification.
  `docs/setup/` does not exist on `origin/main`. Implementing
  worker either creates the directory + file in the same doc-only
  commit, or amends a canonical sibling doc that already exists
  (e.g. `docs/WORKFLOW-GUIDE.md`, `docs/octogent-integration.md`,
  `CONTRIBUTING.md`) and records the chosen path in story
  evidence.

### Engine / skills

- **Engine**: N/A.
- **Mandatory skills**: none (doc-only).

### Control Manifest Rules

- Required: One paragraph (or a small section) in the canonical
  onboarding doc names `gh` as required and provides install
  commands.
- Forbidden: Installing `gh` under this story.
- Forbidden: Adding `gh` to any CI workflow or build script.
- Forbidden: Adding any other tooling beyond `gh`.

---

## Story Classification

**Story type**: Documentation only.

This is **NOT** a:

- Code change.
- Tooling installation.
- CI workflow change.

---

## Acceptance Criteria

All criteria are independently checkable.

- [ ] **AC1 -- Onboarding doc updated**: GIVEN the canonical
  onboarding doc chosen by the implementing worker (either newly
  created `docs/setup/dev-environment.md` per option (a), or an
  amended canonical sibling doc per option (b) -- see Prevention
  target), WHEN inspected post-story, THEN it contains a clearly
  named paragraph or section listing `gh` as required tooling and
  story evidence records which path was chosen and the rationale.

- [ ] **AC2 -- Install commands provided**: GIVEN the new
  paragraph / section, WHEN inspected, THEN it provides
  platform-appropriate install commands for at least Windows
  (`winget install --id GitHub.cli` or canonical equivalent).
  macOS / Linux commands are optional but encouraged.

- [ ] **AC3 -- No tooling installed**: GIVEN the story commit,
  WHEN inspected, THEN no actual installation, CI workflow
  change, or build-script change lands. Only the doc edit (and
  optionally this story file's status) is touched.

- [ ] **AC4 -- Sprint 13 disposition preserved**: GIVEN the story
  commit, WHEN `production/sprint-status.yaml`,
  `production/sprints/sprint-13.md`, `production/stage.txt`, and
  PROMPT 761 gate-check artifact are diffed, THEN none of them
  are modified by this story.

- [ ] **AC5 -- No-claim restatement embedded**: GIVEN the new
  paragraph / section, WHEN inspected, THEN it (or this story
  file) includes the verbatim "Status / No-Claim Banner" no-claim
  restatement.

---

## Likely Files

| Path | Anticipated change |
|------|--------------------|
| `docs/setup/dev-environment.md` (NEW; created by option (a)) **or** an existing canonical sibling doc on `main` (e.g. `docs/WORKFLOW-GUIDE.md`, `docs/octogent-integration.md`, `CONTRIBUTING.md`) per option (b) | Paragraph or small section added naming `gh` and install commands. Chosen path recorded in story evidence. |
| This story file | Status update on `/story-done`. |

---

## Required Skills

- None (doc-only).

---

## Evidence Path

The onboarding doc edit **is** the artifact for this story.

**Required doc content** (deferred to implementation prompt):

- Named tooling: `gh` (GitHub CLI).
- Install commands (at least Windows; macOS / Linux optional).
- Optional `gh auth login` hint.

---

## Regression Commands Expected

For the implementation prompt:

- `git diff <pre-impl-sha>..<impl-sha> -- 'client/**' 'server/**' 'shared/**' 'tests/**' 'Cargo.toml' '.cargo/**' '.github/**' '*.sh' '*.ps1'`
  (verifies AC3: zero code/config/CI change)
- `git diff --check origin/main...HEAD`
- `git diff --cached --check`

No `cargo` command is required by this story.

---

## Out of Scope

- Actually installing `gh` on any machine.
- Any CI workflow change requiring `gh`.
- Recommending alternative GitHub-CLI tools.
- Sprint 13 activation, `S8-QA-001-W1` closure, or Polish->Release
  gate-check retry.
- `QA-COND-0005` / `QA-COND-0006` advancement.
- No `/dev-story`, `/story-done`, `/smoke-check`, `/team-qa`,
  `/gate-check`, `/release-check`, or `/qa-plan` run under the
  authoring prompt.

---

## Dependency Notes Against Sprint 13 Active Scope

- Doc-only; no file-scope collision with any Sprint 13 Must Have
  or Should Have row.
- May share `docs/setup/dev-environment.md` with Sprint 13 Nice to
  Have row `S13-OPS-WIN-APPCOMPAT-NOTE-001` (story 005 in this
  epic). Both edits are paragraph-scale; they can be applied in
  sequence or together; same-file conflict risk is low.
