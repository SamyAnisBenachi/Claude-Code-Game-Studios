# Story 005: S13-OPS-WIN-APPCOMPAT-NOTE-001 -- Windows AppCompat Heuristic + Workaround Note

> **Epic**: DevOps (Operational Hardening)
> **Story ID**: S13-OPS-WIN-APPCOMPAT-NOTE-001
> **Status**: Draft -- Sprint 13 candidate (Nice to Have); NOT activated;
> Sprint 12 closed-with-conditions per PROMPT 817
> **Layer**: DevOps -- Windows dev-environment documentation only
> **Type**: Documentation only -- no production-source change lands
> **Sprint**: Sprint 13 candidate (informational from TQ-S12-C7; PROMPT 815
> / 816 / 817 evidence on the AppCompat heuristic triggering off the
> substring `update` in `spawn_range_live_update_contract-*.exe`); NOT
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
- Modify any test-binary name or Cargo target name.
- Retry the PROMPT 761 Polish->Release gate-check.

This story does **not** claim: public release readiness, release-candidate
readiness, full game completion, broad / Standard-tier accessibility
completion (`QA-COND-0005`), playtest / fun-hypothesis validation
(`QA-COND-0006`), full playable-client manual QA, two-client GAME_OVER
closure (`S8-QA-001-W1`), or final-art / asset-production completion.

Sprint 10 / Sprint 11 / Sprint 12 dispositions unchanged. PROMPT 761
Polish->Release gate-check FAIL evidence preserved.

**This story is informational doc-only. NO PRODUCTION-SOURCE CHANGE
LANDS.** It is **NOT a Sprint 12 close-out blocker** (already
accepted-risk per TQ-S12-C7); landing here is purely so the next
smoke check is not surprised by the same Windows AppCompat warning.

---

## Source Finding

- TQ-S12-C7 informational condition: Windows AppCompat smoke warning
  during Sprint 12 PROMPT 815 / PROMPT 816 / PROMPT 817 invocations.
- Evidence trace: AppCompat heuristic triggers on the substring
  `update` in the test-binary name
  `spawn_range_live_update_contract-*.exe`, classifying the binary
  as a potential installer and prompting elevated-permissions UAC.
- PROMPT 815 unblocked the smoke by renaming the binary at
  invocation time; the workaround is documented here so the next
  smoke check picks up the workaround automatically (or the
  embedded-manifest decision lands as a follow-on).

---

## Problem Class / Prevention Target

**Defect class**: Windows AppCompat heuristic auto-classifies any
executable whose **filename** contains the substring `update`,
`install`, `setup`, etc. as a potential installer, prompting an
elevated-permissions UAC dialog when the executable launches. Cargo
test binaries for tests named `spawn_range_live_update_contract_*`
inherit a filename containing `update`, triggering the heuristic
falsely.

**Path-existence note (PROMPT 823 / PROMPT 824 verification)**:
`docs/setup/` does **NOT** exist on `origin/main` at PROMPT 823
readiness verification. `docs/setup/dev-environment.md` is therefore
a **target path**, not an existing file. The implementing worker
chooses one of two paths and records the chosen path in story
evidence:

- (a) Create `docs/setup/` and `docs/setup/dev-environment.md`
  in the same doc-only commit as the AppCompat paragraph; **or**
- (b) Amend a canonical sibling doc that already exists on `main`
  (e.g. `docs/WORKFLOW-GUIDE.md`, `docs/octogent-integration.md`,
  `CONTRIBUTING.md`) and record the chosen path in story evidence.

The implementing worker picks (a) **or** (b) -- not both -- and
records the rationale in story evidence.

**Prevention target**: A paragraph in the chosen target doc (per
(a) or (b) above) that:

- Documents the AppCompat heuristic and the exact substrings that
  trigger it (`update`, `install`, `setup`, `patch`, `uninst`,
  etc.).
- Documents **either**:
  - (a) The PROMPT 815 binary-rename workaround used during the
    smoke run (rename the test target before invocation, or use a
    Cargo `[[test]]` `name = "..."` attribute that avoids the
    triggering substring); **or**
  - (b) A small embedded manifest (`level="asInvoker"`) decision
    documented for a separate follow-on story (with the trade-offs
    and the workspace-wide build-script change scope).

The implementing worker picks (a) **or** (b) -- not both -- and
records the rationale.

**No production-source change lands.** If option (b) is chosen,
its follow-on story owns the actual manifest/build-script change.

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
- **Test target name**: `spawn_range_live_update_contract` --
  trigger source. Renaming is workspace-cargo affecting; **out of
  scope** for this story.
- **PROMPT 815 / 816 / 817 evidence**: contemporaneous AppCompat
  warnings logged during the Sprint 12 smoke / Team-QA / close-out
  invocations.

### Engine / skills

- **Engine**: N/A (Windows OS heuristic; not a game engine
  concern).
- **Mandatory skills**: none (doc-only).

### Control Manifest Rules

- Required: Paragraph in the chosen target doc (per option (a)
  newly created `docs/setup/dev-environment.md`, or option (b)
  amended canonical sibling doc -- see Prevention target)
  documents the heuristic and the chosen workaround. Chosen path
  is recorded in story evidence.
- Required: If option (b) embedded manifest is chosen, its
  follow-on story slug is named with rationale.
- Forbidden: Renaming any Cargo test target under this story.
- Forbidden: Adding any build-script manifest change under this
  story.
- Forbidden: Implementing the embedded-manifest workaround under
  this story.

---

## Story Classification

**Story type**: Documentation only.

This is **NOT** a:

- Cargo target rename.
- Build-script change.
- Manifest embedding.

---

## Acceptance Criteria

All criteria are independently checkable.

- [ ] **AC1 -- Dev-environment doc updated**: GIVEN the chosen
  target doc post-story (either newly created
  `docs/setup/dev-environment.md` per option (a), or an amended
  canonical sibling doc per option (b) -- see Prevention target),
  WHEN inspected, THEN it contains a clearly named paragraph or
  section documenting the Windows AppCompat heuristic and the
  substrings that trigger it, and story evidence records which
  path was chosen and the rationale.

- [ ] **AC2 -- Single workaround chosen**: GIVEN the new
  paragraph / section, WHEN inspected, THEN exactly one of:
  - (a) Binary-rename workaround documented (specific command or
    Cargo `[[test]] name = "..."` example), **or**
  - (b) Embedded manifest follow-on documented (with named
    follow-on story slug + rationale).

- [ ] **AC3 -- Cross-link to TQ-S12-C7**: GIVEN the new
  paragraph / section, WHEN inspected, THEN it cross-links to
  TQ-S12-C7 in
  `production/qa/team-qa-sprint-12-2026-05-14.md` (or the
  Sprint 12 close-out disposition referencing TQ-S12-C7).

- [ ] **AC4 -- No production-source change lands**: GIVEN the
  story commit, WHEN inspected, THEN no file under `client/`,
  `server/`, `shared/`, `tests/` is modified. No Cargo target is
  renamed. No build-script change lands. Only the doc edit (and
  optionally this story file's status) is touched.

- [ ] **AC5 -- Sprint 13 disposition preserved**: GIVEN the story
  commit, WHEN `production/sprint-status.yaml`,
  `production/sprints/sprint-13.md`, `production/stage.txt`, and
  PROMPT 761 gate-check artifact are diffed, THEN none of them
  are modified by this story.

- [ ] **AC6 -- TQ-S12-C7 NOT closed by this story**: GIVEN the
  story commit, WHEN the team-qa Sprint 12 condition tracking is
  reviewed, THEN TQ-S12-C7 remains preserved as informational
  (this story does NOT mark it closed). Closure of TQ-S12-C7 (if
  ever) happens via a separate `/team-qa` or close-out prompt.

- [ ] **AC7 -- No-claim restatement embedded**: GIVEN the new
  paragraph / section, WHEN inspected, THEN it (or this story
  file) includes the verbatim "Status / No-Claim Banner" no-claim
  restatement plus the explicit "NOT a Sprint 12 close-out
  blocker" line from TQ-S12-C7.

---

## Likely Files

| Path | Anticipated change |
|------|--------------------|
| `docs/setup/dev-environment.md` (NEW; created by option (a)) **or** an existing canonical sibling doc on `main` (e.g. `docs/WORKFLOW-GUIDE.md`, `docs/octogent-integration.md`, `CONTRIBUTING.md`) per option (b) | Paragraph or small section added documenting the AppCompat heuristic + chosen workaround. Chosen path recorded in story evidence. |
| This story file | Status update on `/story-done`. |

---

## Required Skills

- None (doc-only).

---

## Evidence Path

The dev-environment doc edit **is** the artifact for this story.

**Required doc content** (deferred to implementation prompt):

- AppCompat heuristic substrings (per AC1).
- Single chosen workaround -- option (a) or option (b) per AC2.
- Cross-link to TQ-S12-C7 per AC3.
- No-claim restatement + "NOT a Sprint 12 close-out blocker"
  restatement per AC7.

---

## Regression Commands Expected

For the implementation prompt:

- `git diff <pre-impl-sha>..<impl-sha> -- 'client/**' 'server/**' 'shared/**' 'tests/**' 'Cargo.toml' '.cargo/**' '.github/**' '*.sh' '*.ps1'`
  (verifies AC4: zero code/config/CI change)
- `git diff --check origin/main...HEAD`
- `git diff --cached --check`

No `cargo` command is required by this story.

---

## Out of Scope

- Renaming the Cargo test target `spawn_range_live_update_contract`.
- Implementing the embedded manifest workaround.
- Any production-source change.
- Sprint 13 activation, `S8-QA-001-W1` closure, or Polish->Release
  gate-check retry.
- `QA-COND-0005` / `QA-COND-0006` advancement.
- TQ-S12-C7 closure (preserved as informational).
- No `/dev-story`, `/story-done`, `/smoke-check`, `/team-qa`,
  `/gate-check`, `/release-check`, or `/qa-plan` run under the
  authoring prompt.

---

## Dependency Notes Against Sprint 13 Active Scope

- Doc-only; no file-scope collision with any Sprint 13 Must Have
  or Should Have row.
- May share a target doc with Sprint 13 Nice to Have row
  `S11-OPS-GH-CLI-001` (story 004 in this epic) if both stories
  pick the same option (a) target path
  (`docs/setup/dev-environment.md`) or the same option (b)
  canonical sibling doc. Both edits are paragraph-scale; same-file
  conflict risk is low. The worker can sequence them in either
  order; whichever runs first creates the file/directory (option
  (a)) or establishes the chosen sibling-doc path (option (b)),
  and the second worker appends.
