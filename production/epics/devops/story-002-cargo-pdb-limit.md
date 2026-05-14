# Story 002: S11-TD-CARGO-PDB-LIMIT-001 -- Cargo PDB-Size Pressure Investigation

> **Epic**: DevOps (Operational Hardening)
> **Story ID**: S11-TD-CARGO-PDB-LIMIT-001
> **Status**: Draft -- Sprint 13 candidate (Nice to Have); NOT activated;
> Sprint 12 closed-with-conditions per PROMPT 817
> **Layer**: DevOps -- Cargo Windows profile investigation (doc only)
> **Type**: Investigation -- doc only; no profile change lands
> **Sprint**: Sprint 13 candidate (Sprint 12 close-out deferral; Sprint 11
> Wave 12 backlog); NOT activated
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
- Modify `Cargo.toml`, `.cargo/config.toml`, build scripts, or CI
  workflows.
- Retry the PROMPT 761 Polish->Release gate-check.

This story does **not** claim: public release readiness, release-candidate
readiness, full game completion, broad / Standard-tier accessibility
completion (`QA-COND-0005`), playtest / fun-hypothesis validation
(`QA-COND-0006`), full playable-client manual QA, two-client GAME_OVER
closure (`S8-QA-001-W1`), or final-art / asset-production completion.

Sprint 10 / Sprint 11 / Sprint 12 dispositions unchanged. PROMPT 761
Polish->Release gate-check FAIL evidence preserved.

**This story is investigation-only. NO PROFILE CHANGES LAND.** A
recommendation is documented; the actual profile knobs land in a
follow-on story.

---

## Source Finding

- Sprint 11 Wave 12 backlog noted that Cargo PDB (Program Database)
  files for Windows debug builds were a major contributor to
  per-worktree `target/dev/` size.
- Sprint 11 close-out raised `S11-TD-CARGO-PDB-LIMIT-001` as a
  Nice-to-Have follow-on investigation.
- Sprint 12 close-out (PROMPT 817) deferred the row forward to
  Sprint 13 planning.

---

## Problem Class / Prevention Target

**Defect class**: On Windows, Cargo's default debug-info emission
produces PDB files that contribute heavily to per-worktree `target/`
size and CI runtime (PDB copy time, antivirus scan time). The
default `[profile.dev]` and `[profile.test]` settings are not tuned
for PDB compactness.

**Prevention target**: An investigation note at
`docs/architecture/cargo-pdb-pressure.md` (NEW) that:

- Documents observed PDB-size impact on per-worktree `target/`
  size and on workspace build / test wall-clock time.
- Recommends Windows-side profile knobs for `[profile.dev]` and
  `[profile.test]`:
  - `split-debuginfo = "packed"` (or `"unpacked"`, depending on
    rust-lld availability and Cargo-correctness on Windows).
  - `strip = "debuginfo"` (for `[profile.test]` only -- removes
    debug info from test binaries while preserving dev-binary
    debuggability).
  - `debug = "line-tables-only"` as a less aggressive alternative.
- Trades off debuggability vs disk + CI savings.
- Names a single recommended follow-on story slug to land the
  actual profile change.

**No profile change lands in this story.**

---

## Context

### Existing surface

- **`Cargo.toml`** workspace root: no `[profile.dev]` /
  `[profile.test]` override (defaults).
- **Per-worktree `target/dev/` and `target/test/`**: PDB files
  observed at multi-GB sizes in the orchestrator hosts.

### Engine / skills

- **Engine**: Bevy 0.18 (Rust). No `.rs` edits expected.
- **Mandatory skills**: none (doc-only story).

### Control Manifest Rules

- Required: Note at `docs/architecture/cargo-pdb-pressure.md`
  documents observed sizes, recommended knobs, trade-offs, and a
  single recommended follow-on.
- Required: No `Cargo.toml`, `.cargo/config.toml`, CI workflow, or
  build script is modified.
- Forbidden: Recommending profile knobs that disable RELEASE-build
  debuggability (this investigation targets dev / test profiles
  only).

---

## Story Classification

**Story type**: Investigation / Documentation only.

This is **NOT** a:

- Profile change.
- Build-script change.

---

## Acceptance Criteria

All criteria are independently checkable.

- [ ] **AC1 -- Observed PDB sizes recorded**: GIVEN the note at
  `docs/architecture/cargo-pdb-pressure.md`, WHEN inspected, THEN
  it records observed PDB-file sizes for at least one dev and one
  test build (worktree-relative paths and approximate GB).

- [ ] **AC2 -- Profile knobs recommended**: GIVEN the note, WHEN
  inspected, THEN at least the following Windows-relevant knobs
  are documented with their semantic effects on PDB size and
  debuggability:
  - `split-debuginfo` (packed / unpacked / off).
  - `strip = "debuginfo"` (for `[profile.test]`).
  - `debug = "line-tables-only"` (less aggressive alternative).

- [ ] **AC3 -- Trade-offs articulated**: GIVEN each recommended
  knob, WHEN inspected, THEN the note articulates:
  - Effect on per-worktree disk size (rough order of magnitude).
  - Effect on CI wall-clock (copy + scan time).
  - Effect on debuggability (loss of file:line in tracebacks?
    loss of full debug info?).
  - Windows Cargo-correctness considerations (rust-lld
    availability, antivirus interaction).

- [ ] **AC4 -- Single follow-on named**: GIVEN the note, WHEN
  inspected, THEN exactly one recommended follow-on story slug is
  named with a one-sentence rationale (e.g.,
  "S14-DEVOPS-CARGO-PROFILE-DEV-DEBUGINFO-001 -- set
  `[profile.dev] split-debuginfo = "packed"` workspace-wide;
  rationale: largest dev-build PDB-size reduction with negligible
  debuggability impact").

- [ ] **AC5 -- No code / config change lands**: GIVEN the story
  commit, WHEN `git diff` is inspected, THEN no file under
  `client/`, `server/`, `shared/`, `tests/`, `Cargo.toml`,
  `.cargo/`, `.github/`, or any build script is modified. Only
  the new note (and optionally this story file's status) is
  touched.

- [ ] **AC6 -- Sprint 13 disposition preserved**: GIVEN the story
  commit, WHEN `production/sprint-status.yaml`,
  `production/sprints/sprint-13.md`, `production/stage.txt`, and
  PROMPT 761 gate-check artifact are diffed, THEN none of them
  are modified by this story.

- [ ] **AC7 -- Cross-link to disk-usage investigation**: GIVEN the
  note, WHEN inspected, THEN it cross-links to the sibling
  Sprint 13 story
  `S11-TD-CARGO-DISK-USAGE-001` investigation note at
  `docs/architecture/cargo-workspace-disk-usage.md` (when that
  note lands).

- [ ] **AC8 -- No-claim restatement embedded**: GIVEN the note,
  WHEN inspected, THEN it includes the verbatim "Status / No-Claim
  Banner" no-claim restatement.

---

## Likely Files

| Path | Anticipated change |
|------|--------------------|
| `docs/architecture/cargo-pdb-pressure.md` | NEW investigation note. |
| This story file | Status update on `/story-done`. |

---

## Required Skills

- None (doc-only).

---

## Evidence Path

`docs/architecture/cargo-pdb-pressure.md` (NEW) **is** the artifact
for this story; there is no separate evidence document.

**Required note content** (deferred to implementation prompt):

- Observed PDB sizes (per AC1).
- Recommended profile knobs with semantics (per AC2).
- Trade-offs per knob (per AC3).
- Single recommended follow-on slug + rationale (per AC4).
- Cross-link to the disk-usage investigation note (per AC7).
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

- Any profile / build-script / tooling change.
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
- Sequences cleanly alongside or after
  `S11-TD-CARGO-DISK-USAGE-001` (Sprint 13 Nice to Have story 001
  in this epic).
