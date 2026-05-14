# Story 001: S11-TD-CARGO-DISK-USAGE-001 -- Cargo Workspace Disk-Usage Reduction Strategy

> **Epic**: DevOps (Operational Hardening)
> **Story ID**: S11-TD-CARGO-DISK-USAGE-001
> **Status**: Draft -- Sprint 13 candidate (Nice to Have); NOT activated;
> Sprint 12 closed-with-conditions per PROMPT 817
> **Layer**: DevOps -- Cargo workspace investigation (doc only)
> **Type**: Investigation -- doc only; no build-script change lands
> **Sprint**: Sprint 13 candidate (Sprint 12 close-out deferral; re-affirmed
> by PROMPT 815 disk-pressure invocation cleaning 25 GB + ~200 GB worker
> `target/` directories); NOT activated
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
- Modify `Cargo.toml`, build scripts, or CI workflows.
- Retry the PROMPT 761 Polish->Release gate-check.

This story does **not** claim: public release readiness, release-candidate
readiness, full game completion, broad / Standard-tier accessibility
completion (`QA-COND-0005`), playtest / fun-hypothesis validation
(`QA-COND-0006`), full playable-client manual QA, two-client GAME_OVER
closure (`S8-QA-001-W1`), or final-art / asset-production completion.

Sprint 10 / Sprint 11 / Sprint 12 dispositions unchanged. PROMPT 761
Polish->Release gate-check FAIL evidence preserved.

**This story is investigation-only. NO BUILD-SCRIPT CHANGES LAND.**
A single recommended follow-on story is named at the end of the note;
that follow-on is itself a separate story file authored later.

---

## Source Finding

- Sprint 11 close-out raised `S11-TD-CARGO-DISK-USAGE-001` as a
  developer-quality-of-life concern: per-worktree `target/`
  directories balloon disk usage on Windows orchestrator hosts.
- Sprint 12 close-out (PROMPT 817) deferred the row forward to Sprint
  13 planning.
- PROMPT 815 Sprint 12 smoke-check invocation re-affirmed the
  concern by hitting a disk-pressure threshold; the cleanup invocation
  freed 25 GB + ~200 GB of worker `target/` directories during the
  smoke run.

---

## Problem Class / Prevention Target

**Defect class**: Cargo `target/` directories per worktree consume
gigabytes of disk under the active orchestrator pattern (multiple
parallel workers, each with its own worktree, each with its own
`target/`). At smoke / Team-QA time the host has hit disk-pressure
thresholds; the PROMPT 815 cleanup is reactive, not preventive.

**Prevention target**: An investigation note at
`docs/architecture/cargo-workspace-disk-usage.md` (NEW) that:

- Documents the current per-worktree `target/` footprint (sizes
  observed; approximate dev / test breakdown).
- Identifies trim candidates:
  - Shared target directory (`CARGO_TARGET_DIR=...`) across worker
    worktrees.
  - Profile knobs (`debug = "line-tables-only"` /
    `debug = false`; `split-debuginfo = "packed"`; `strip` per
    profile).
  - `sccache` or `cachepot` cross-worktree build cache.
  - Periodic `cargo sweep --time N` automated invocation.
- Trades off each candidate (Windows compatibility, parallel-worker
  invalidation behaviour, CI compatibility, file-lock concurrency
  risk on shared `target/`).
- Recommends a **single** change to land in a follow-on story
  (names the follow-on story slug).

**No build-script change lands in this story.** The follow-on story
owns the actual change and its own AC set.

---

## Context

### Existing surface

- **`Cargo.toml`** workspace root: defines workspace members; no
  `target` override.
- **Per-worktree `target/`**: default Cargo behaviour; each worktree
  builds its own.
- **PROMPT 815 disk-pressure cleanup**: precedent for periodic
  cleanup; not a preventive change.

### Engine / skills

- **Engine**: Bevy 0.18 (Rust). No `.rs` edits expected.
- **Mandatory skills**: none (doc-only story). The follow-on story
  invokes `liv-bevy-018` only if its scope touches `.rs` code.

### Control Manifest Rules

- Required: Note at `docs/architecture/cargo-workspace-disk-usage.md`
  documents current footprint, trim candidates, trade-offs, and a
  single recommended follow-on.
- Required: No `Cargo.toml`, `.cargo/config.toml`, CI workflow, or
  build script is modified.
- Forbidden: Adding any tooling dependency (sccache, cachepot,
  cargo-sweep) under this story.
- Forbidden: Recommending more than one follow-on story (constrains
  Sprint 14+ scope; multi-recommendation can be a separate
  audit-roadmap-prep story if needed).

---

## Story Classification

**Story type**: Investigation / Documentation only.

This is **NOT** a:

- Build-script change.
- Profile change.
- Tooling installation.

---

## Acceptance Criteria

All criteria are independently checkable.

- [ ] **AC1 -- Footprint baseline recorded**: GIVEN the note at
  `docs/architecture/cargo-workspace-disk-usage.md`, WHEN inspected,
  THEN it records the per-worktree `target/` footprint observed
  (one orchestrator worktree's dev target + one orchestrator
  worktree's test target, at minimum, with absolute byte counts or
  GB rounded).

- [ ] **AC2 -- Trim candidates enumerated**: GIVEN the note, WHEN
  inspected, THEN at least four trim candidates are enumerated:
  shared target dir, profile-knob set (`debug`, `split-debuginfo`,
  `strip`), build-cache (`sccache` / `cachepot`), periodic
  `cargo sweep` automation.

- [ ] **AC3 -- Trade-offs articulated**: GIVEN each enumerated
  candidate, WHEN inspected, THEN the note articulates:
  - Windows compatibility concerns (file-lock concurrency on shared
    `target/`; symbol-debug viability with profile knobs).
  - Parallel-worker invalidation behaviour.
  - CI compatibility.
  - Estimated disk savings range (rough order of magnitude).

- [ ] **AC4 -- Single follow-on named**: GIVEN the note, WHEN
  inspected, THEN exactly one recommended follow-on story slug is
  named with a one-sentence rationale (e.g.,
  "S14-DEVOPS-CARGO-SHARED-TARGET-001 -- migrate orchestrator
  worktrees to a shared CARGO_TARGET_DIR with per-worker subpath
  prefix; rationale: highest expected disk-saving payoff with
  lowest CI/Cargo-correctness risk").

- [ ] **AC5 -- No code / config change lands**: GIVEN the story
  commit, WHEN `git diff` is inspected, THEN no file under
  `client/`, `server/`, `shared/`, `tests/`, `Cargo.toml`,
  `.cargo/`, `.github/`, or any build script is modified. Only the
  new note (and optionally this story file's status) is touched.

- [ ] **AC6 -- Sprint 13 disposition preserved**: GIVEN the story
  commit, WHEN `production/sprint-status.yaml`,
  `production/sprints/sprint-13.md`, `production/stage.txt`, and
  PROMPT 761 gate-check artifact are diffed, THEN none of them are
  modified by this story.

- [ ] **AC7 -- Cross-link to PROMPT 815 disk-pressure cleanup**:
  GIVEN the note, WHEN inspected, THEN it cross-links to the
  PROMPT 815 Sprint 12 smoke disk-pressure invocation as the
  re-affirming evidence.

- [ ] **AC8 -- No-claim restatement embedded**: GIVEN the note,
  WHEN inspected, THEN it includes the verbatim "Status / No-Claim
  Banner" no-claim restatement.

---

## Likely Files

| Path | Anticipated change |
|------|--------------------|
| `docs/architecture/cargo-workspace-disk-usage.md` | NEW investigation note. |
| This story file | Status update on `/story-done`. |

---

## Required Skills

- None (doc-only). The recommended follow-on story may invoke
  `liv-bevy-018` if its scope touches `.rs` code.

---

## Evidence Path

`docs/architecture/cargo-workspace-disk-usage.md` (NEW) **is** the
artifact for this story; there is no separate evidence document.

**Required note content** (deferred to implementation prompt):

- Footprint baseline (per AC1).
- Trim candidates (per AC2).
- Trade-offs (per AC3).
- Single recommended follow-on slug + rationale (per AC4).
- Cross-link to PROMPT 815 disk-pressure cleanup (per AC7).
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

- Any build-script / profile / tooling change.
- Multiple recommended follow-ons (single follow-on only).
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
- Sequences independently any time in Sprint 13.
- The recommended follow-on story is **not** activated by this
  story; activation requires a separate `/sprint-plan` revision in
  Sprint 14+.
