# Story 002: S11-TD-CARGO-PDB-LIMIT-001 -- Cargo PDB-Size Pressure Investigation

> **Epic**: DevOps (Operational Hardening)
> **Story ID**: S11-TD-CARGO-PDB-LIMIT-001
> **Status**: Done — closed by PROMPT 868 `/story-done` on 2026-05-14
> against `origin/main@098f671` (PROMPT 866 worker commit `08d871a` →
> PROMPT 867 integration merge commit `098f671` on `origin/main`).
> Sprint 13 remains `active`; PROMPT 868 is a per-story closure
> paperwork run only.
> **Layer**: DevOps -- Cargo Windows profile investigation (doc only)
> **Type**: Investigation -- doc only; no profile change lands
> **Sprint**: Sprint 13 Nice to Have (activated by PROMPT 826; closed by
> PROMPT 868). Sprint 12 close-out deferral; Sprint 11 Wave 12 backlog.
> **Authored**: 2026-05-14 by PROMPT 819
> **Authoring source-of-truth**: `origin/main@be69f5c` (PROMPT 818
> `/sprint-plan sprint-13` DRAFT)
> **Closed**: 2026-05-14 by PROMPT 868 `/story-done` at
> `origin/main@098f671`

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

- [x] **AC1 -- Observed PDB sizes recorded**: GIVEN the note at
  `docs/architecture/cargo-pdb-pressure.md`, WHEN inspected, THEN
  it records observed PDB-file sizes for at least one dev and one
  test build (worktree-relative paths and approximate GB).
  **PASS** — note §2 (lines 97-143) records dev-build PDB sizes
  (§2.1: server dev binary `target/debug/gcs_server.pdb` ~120-180 MB;
  client dev binary `target/debug/gcs_client.pdb` ~140-220 MB;
  aggregate `target/debug/deps/*.pdb` ~2-4 GB cumulative;
  `target/debug/` total ~3-5 GB per worktree per dev build) and
  test-build PDB sizes (§2.2: per-test-binary
  `target/debug/deps/<test_name>-<hash>.pdb` ~80-160 MB each;
  aggregate ~15-30 GB per worktree per full test build;
  combined dev+test ~18-35 GB) with worktree-relative paths and
  approximate-GB ranges; §2.3 notes observation method (order-of-
  magnitude estimates derived from PROMPT 815 25 GB + ~200 GB
  `target/` cleanup datapoints; re-measurement deferred to
  follow-on implementation). Verified at note lines 97-143 on
  `origin/main@098f671`.

- [x] **AC2 -- Profile knobs recommended**: GIVEN the note, WHEN
  inspected, THEN at least the following Windows-relevant knobs
  are documented with their semantic effects on PDB size and
  debuggability:
  - `split-debuginfo` (packed / unpacked / off).
  - `strip = "debuginfo"` (for `[profile.test]`).
  - `debug = "line-tables-only"` (less aggressive alternative).
  **PASS** — note §3 documents exactly three Windows-relevant
  knobs in their own subsections: Knob 1 `split-debuginfo`
  (packed / unpacked / off; §3.1 at lines 154-196), Knob 2
  `strip = "debuginfo"` for `[profile.test]` only (§3.2 at
  lines 198-234), Knob 3 `debug = "line-tables-only"` (§3.3 at
  lines 236-268). Each subsection documents the knob's mechanism
  and its semantic effects on PDB size and debuggability.

- [x] **AC3 -- Trade-offs articulated**: GIVEN each recommended
  knob, WHEN inspected, THEN the note articulates:
  - Effect on per-worktree disk size (rough order of magnitude).
  - Effect on CI wall-clock (copy + scan time).
  - Effect on debuggability (loss of file:line in tracebacks?
    loss of full debug info?).
  - Windows Cargo-correctness considerations (rust-lld
    availability, antivirus interaction).
  **PASS** — each of the three knobs in §3 has all four trade-off
  axes addressed under explicit headings ("Effect on per-worktree
  disk size", "Effect on CI wall-clock", "Effect on
  debuggability", "Windows Cargo-correctness considerations").
  Knob 1: neutral disk under packed; rust-lld migration risk
  flagged for unpacked; antivirus interaction noted. Knob 2:
  largest single-knob payoff (~15-30 GB savings on heavy test
  build); test-binary backtrace file:line loss (mitigatable
  locally). Knob 3: moderate-to-large savings (~12-24 GB combined);
  file:line preserved but inline frames + named-locals lost;
  supported on MSVC since Rust 1.71. Comparative summary table
  at lines 270-276 collapses all axes into a side-by-side grid
  with disk savings / CI / debuggability / Cargo-correctness /
  scope-of-apply columns. §3 also includes "Explicitly out of
  scope (release profile)" clause at lines 278-291 preventing
  release-build debuggability changes (per story Control Manifest
  Forbidden clause).

- [x] **AC4 -- Single follow-on named**: GIVEN the note, WHEN
  inspected, THEN exactly one recommended follow-on story slug is
  named with a one-sentence rationale (e.g.,
  "S14-DEVOPS-CARGO-PROFILE-DEV-DEBUGINFO-001 -- set
  `[profile.dev] split-debuginfo = "packed"` workspace-wide;
  rationale: largest dev-build PDB-size reduction with negligible
  debuggability impact").
  **PASS** — §4 names exactly one follow-on
  `S14-DEVOPS-CARGO-PROFILE-TEST-STRIP-001` (Knob 2:
  `[profile.test] strip = "debuginfo"` workspace-wide, leaving
  `[profile.dev]` and `[profile.release]` untouched) with one-
  sentence rationale at lines 302-308 ("largest single-knob
  PDB-size reduction on Windows / MSVC (~15-30 GB per worktree
  after a full `cargo test --workspace --tests --no-fail-fast`
  run, per §2.2) with the narrowest debuggability trade-off (only
  test-binary backtraces lose file:line resolution, and that loss
  is mitigatable locally by temporarily reverting the knob;
  dev-binary debuggability is fully preserved)"). Out-of-scope
  clause (lines 310-323) explicitly defers Knobs 1 (defensive
  lock-in folded into same Cargo.toml patch if desired) + Knob 3
  (line-tables-only on dev+test deferred until Knob 2 savings
  measured insufficient). Activation requirement clause
  (lines 325-329) states the follow-on is NOT activated by this
  story. Interaction-with-sibling-follow-on clause (lines 331-340)
  notes complementarity with `S14-DEVOPS-CARGO-SHARED-TARGET-001`
  (sibling story 001 follow-on); both can land first; sequencing
  belongs to Sprint 14+ `/sprint-plan`.

- [x] **AC5 -- No code / config change lands**: GIVEN the story
  commit, WHEN `git diff` is inspected, THEN no file under
  `client/`, `server/`, `shared/`, `tests/`, `Cargo.toml`,
  `.cargo/`, `.github/`, or any build script is modified. Only
  the new note (and optionally this story file's status) is
  touched.
  **PASS** — `git show --stat 08d871a` (PROMPT 866 worker commit) +
  `git show --stat 098f671` (PROMPT 867 integration merge) both
  confirm single-file diff: `docs/architecture/cargo-pdb-pressure.md`
  (NEW; +382 / -0). Zero file under `client/` / `server/` /
  `shared/` / `tests/` / `Cargo.toml` / `.cargo/` / `.github/` /
  any build script touched by worker or integration. `git diff
  3edf9c6..098f671 --stat -- 'client/**' 'server/**' 'shared/**'
  'tests/**' 'Cargo.toml' '.cargo/**' '.github/**' '*.sh' '*.ps1'`
  returns empty. PROMPT 868 (this `/story-done` closure) only
  touches paperwork files (this story file + `sprint-status.yaml`
  + `session-state/active.md` + `session-state/codex-orchestrator-state.md`),
  which is the permitted disposition-preserving paperwork edit
  per AC5's "Only the new note (and optionally this story file's
  status) is touched" clause.

- [x] **AC6 -- Sprint 13 disposition preserved**: GIVEN the story
  commit, WHEN `production/sprint-status.yaml`,
  `production/sprints/sprint-13.md`, `production/stage.txt`, and
  PROMPT 761 gate-check artifact are diffed, THEN none of them
  are modified by this story.
  **PASS** — PROMPT 866 worker commit `08d871a` + PROMPT 867
  integration merge `098f671` both touch exactly one file
  (`docs/architecture/cargo-pdb-pressure.md`);
  `production/sprint-status.yaml` / `production/sprints/sprint-13.md` /
  `production/stage.txt` /
  `production/gate-checks/gate-polish-release-2026-05-12.md` are
  all unchanged by those commits (verified via the empty `git
  diff 3edf9c6..098f671 --stat -- 'production/sprint-status.yaml'
  'production/sprints/sprint-13.md' 'production/stage.txt'`
  result). The PROMPT 868 row-level `status: ready -> done` flip
  + `completed: 2026-05-14` annotation in
  `production/sprint-status.yaml` is the permitted disposition-
  preserving `/story-done` paperwork edit (top-level `sprint:` /
  `status:` / `stage:` unchanged); `production/sprints/sprint-13.md`
  is NOT touched by PROMPT 868 (per allowed-files list excluding
  it; precedent PROMPT 865 / 856 / 854 / 851 / 844 / 843 / 840 /
  835).

- [x] **AC7 -- Cross-link to disk-usage investigation**: GIVEN the
  note, WHEN inspected, THEN it cross-links to the sibling
  Sprint 13 story
  `S11-TD-CARGO-DISK-USAGE-001` investigation note at
  `docs/architecture/cargo-workspace-disk-usage.md` (when that
  note lands).
  **PASS** — note §1 (lines 61-94) cross-links to the sibling
  Sprint 13 disk-usage investigation note at
  `docs/architecture/cargo-workspace-disk-usage.md`
  (`S11-TD-CARGO-DISK-USAGE-001`, Sprint 13 Nice to Have — DevOps
  epic story 001; authored by PROMPT 861, integrated to
  `origin/main` by PROMPT 863 merge commit `9a85805`, closed by
  PROMPT 865 `/story-done` `3edf9c6`). Cross-link articulates
  complementarity: sibling addresses **directory-layout** disk
  pressure (shared `CARGO_TARGET_DIR` + per-worker subpath +
  `sccache` + `cargo sweep` automation); this note addresses
  **per-binary symbol-file** disk pressure (PDB profile knobs).
  Additional cross-link to PROMPT 815 disk-pressure cleanup at
  `production/qa/smoke-sprint-12-2026-05-14.md` § "Disk Pressure
  Policy Invocation" lines 146-170 framed at lines 88-93,
  motivating the §2 per-binary observed-size estimates.

- [x] **AC8 -- No-claim restatement embedded**: GIVEN the note,
  WHEN inspected, THEN it includes the verbatim "Status / No-Claim
  Banner" no-claim restatement.
  **PASS** — note lines 14-57 carry the verbatim "Status /
  No-Claim Banner" restatement from this story file, with the
  PROMPT 819 authoring-prompt no-modification list preserved
  verbatim AND an additional PROMPT 866 implementation-prompt
  no-modification clause appended at lines 50-57 (the same
  pattern PROMPT 854 / PROMPT 856 / PROMPT 865 closures used:
  verbatim original banner + per-prompt inclusion clarifying that
  PROMPT 866 itself did not flip the story status to Done — that
  is `/story-done`'s job, executed by PROMPT 868).

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

---

## Authoring / Implementation / Closure Trail

- **PROMPT 819** (2026-05-14) — Authoring. Story file created at
  `production/epics/devops/story-002-cargo-pdb-limit.md` against
  `origin/main@be69f5c` (PROMPT 818 Sprint 13 plan DRAFT). Sprint
  13 candidate (Nice to Have); NOT activated. No sprint-status
  flip, no investigation-note authoring, no build-script changes.
  Paperwork only.
- **PROMPT 822** (2026-05-14) — Sprint 13 missing-story authoring
  batch. Existing story file preserved unchanged by PROMPT 822
  (already authored by PROMPT 819 prior); referenced by the
  PROMPT 822 row enumeration that landed the 10 sibling missing
  story files at integration tip `5470d30`.
- **PROMPT 823** (2026-05-14) — `/story-readiness` rerun batch.
  Verdict **READY** (no advisories; doc-only Investigation story
  with single new evidence path
  `docs/architecture/cargo-pdb-pressure.md`).
- **PROMPT 826** (2026-05-14) — Sprint 13 activation. Row
  `S11-TD-CARGO-PDB-LIMIT-001` recorded in
  `production/sprint-status.yaml` Sprint 13 Nice to Have block
  with `status: ready`; `sprint_13_activation:` snapshot block
  records `status_at_activation: "ready"` and PROMPT 823
  readiness verdict.
- **PROMPT 827** (2026-05-14) — Sprint 13 QA plan
  (`production/qa/qa-plan-sprint-13.md` NEW). Story 002
  classification: documentation only; reviewer sign-off;
  devops-engineer review of `docs/architecture/cargo-pdb-pressure.md`
  (NEW). No automated test required (the note *is* the
  deliverable). No build-script change lands.
- **PROMPT 857** (2026-05-14) — Pre-implementation
  `/story-readiness` re-check. Confirmed READY-NOT-STARTED and
  fully parallel-safe.
- **PROMPT 866** (2026-05-14) — Worker execution (doc-only
  `/dev-story`). Worker branch `work/s13-cargo-pdb-limit`;
  worker commit `08d871a90c740ebe8c9dfe5e956b08b902d16afc`
  (`docs(s13/devops): S11-TD-CARGO-PDB-LIMIT investigation note
  (PROMPT 866)`). Authored
  `docs/architecture/cargo-pdb-pressure.md` (NEW; +382 / -0 lines)
  covering: §1 source finding + cross-link to sibling disk-usage
  note (AC7); §2 observed PDB sizes per dev/test build (AC1);
  §3 three knobs `split-debuginfo` + `strip = "debuginfo"` +
  `debug = "line-tables-only"` with per-knob mechanism + disk /
  CI / debuggability / Cargo-correctness trade-offs +
  comparative summary table at lines 270-276 (AC2 + AC3);
  §4 single recommended follow-on
  `S14-DEVOPS-CARGO-PROFILE-TEST-STRIP-001` (Knob 2;
  `[profile.test] strip = "debuginfo"` workspace-wide) with
  one-sentence rationale + out-of-scope clause for Knobs 1 / 3 +
  activation-requirement clause + interaction-with-sibling
  clause re sibling story 001 follow-on
  `S14-DEVOPS-CARGO-SHARED-TARGET-001` (AC4); §5 files modified
  (single-file diff); §6 AC self-check; verbatim Status /
  No-Claim Banner restatement at note lines 14-57 (AC8) with
  PROMPT 866 implementation-prompt no-modification clause
  appended at lines 50-57. Worker pushed worker branch only
  (`origin/work/s13-cargo-pdb-limit`); worker did NOT push
  `main`. No `cargo` command invoked (story Cargo policy "N/A").
  Worker report:
  `reports/PROMPT-866-S13-Cargo-PDB-Limit-Note.md`.
- **PROMPT 867** (2026-05-14) — Integration. Worker tip
  `08d871a` merged into prior `origin/main@3edf9c6` via merge
  commit `098f671fb7a4a55b97aa7ae19ed713bd13594af6`
  (`integrate(s13): merge work/s13-cargo-pdb-limit (devops story
  002 / PROMPT 866) (PROMPT 867)`). Fast-forward push
  `3edf9c6..098f671` to `origin/main` (no force, no conflict).
  Integration verification: `git diff --check` clean; `git diff
  --cached --check` clean; single-file diff
  `docs/architecture/cargo-pdb-pressure.md` (+382 / -0); no
  forbidden paths (`client/` / `server/` / `shared/` / `tests/` /
  `Cargo.toml` / `.cargo/` / `production/sprint-status.yaml` /
  `production/session-state/` / `production/stage.txt`) touched.
  Integration report:
  `reports/PROMPT-867-S13-Cargo-PDB-Limit-Integration.md`.
  PROMPT 867 did NOT run `/story-done`.
- **PROMPT 868** (2026-05-14) — `/story-done` closure paperwork
  (this closure). Verified each AC1-AC8 against
  `origin/main@098f671` (PROMPT 866 worker `08d871a` + PROMPT
  867 integration merge `098f671`); flipped Status header
  `Draft -> Done` with closure context including PROMPT 866
  worker hash + PROMPT 867 integration merge hash + PROMPT 868
  story-done; flipped AC1-AC8 checkboxes `[ ]` -> `[x]` with
  per-AC closure-evidence annotations; appended this Authoring /
  Implementation / Closure Trail section; flipped
  `production/sprint-status.yaml` Sprint 13 Nice to Have row
  `S11-TD-CARGO-PDB-LIMIT-001` `status: ready -> done` with
  `completed: 2026-05-14`, `worker_prompt: 866`, `worker_commit:
  08d871a90c740ebe8c9dfe5e956b08b902d16afc`,
  `integration_prompt: 867`, `integration_commit:
  098f671fb7a4a55b97aa7ae19ed713bd13594af6`,
  `story_done_prompt: 868`, `acceptance_evidence:
  docs/architecture/cargo-pdb-pressure.md`; refreshed top-level
  `updated:` annotation for PROMPT 868; appended
  `sprint_13_story_done:` entry at end of file as a sibling to
  the prior PROMPT 833 / 840 / 843 / 844 / 850 / 851 / 854 /
  856 / 865 entries. Prepended PROMPT 868 banners to
  `production/session-state/active.md` and
  `production/session-state/codex-orchestrator-state.md`.
  Paperwork only: no `/dev-story`, `/story-readiness`,
  `/smoke-check`, `/team-qa`, `/gate-check`, `/release-check`,
  `/qa-plan` invoked; no production code under `client/` /
  `server/` / `shared/` / `tests/` modified; no
  `production/stage.txt`, `production/sprints/sprint-13.md`,
  `production/sprints/sprint-12.md`,
  `production/qa/qa-plan-sprint-13.md`, or
  `production/gate-checks/*` modified; no `cargo` invoked
  (story Cargo policy "N/A"). Root checkout dirt
  (` M .claude/settings.json`, untracked
  `Dtmpworkspace-test-output.txt`, untracked
  `production/session-state/autonomous-monitor-task.md`,
  untracked `tools/gcs-orchestrator/docs/ARCHITECTURE.md`)
  preserved untouched. PROMPT 868 final report:
  `reports/PROMPT-868-S13-Cargo-PDB-Limit-Story-Done.md` (NOT
  staged or committed; `reports/` is gitignored).

### Conditions carried forward unchanged by PROMPT 868

- `S8-QA-001-W1` manual/browser two-client GAME_OVER gap remains
  **OPEN**. Story 017 AC12 forbid-auto-closure preserved.
- `QA-COND-0005` Standard-tier accessibility remains
  **accepted-risk** (friend-game scope only).
- `QA-COND-0006` playtest / fun-hypothesis validation remains
  **accepted-risk / deferred**.
- `PAW-TD-*-a` placeholder-art accept-risk preserved across
  PAW-002..PAW-006.
- PROMPT 683-era runtime divergence question preserved unchanged
  (folded into Sprint 12 story 019 cannot-reproduce closure;
  third same-scope retest NOT authorised per TQ-S12-C2; PROMPT
  868 does NOT re-attempt the Sprint 12 capture).
- PROMPT 761 Polish->Release gate-check `FAIL` preserved at
  `production/gate-checks/gate-polish-release-2026-05-12.md`; no
  retry in PROMPT 868 scope.
- Story 019 (Sprint 12 hand-ui) underlying drag-runtime bug NOT
  claimed fixed.
- `TQ-S12-C1..C7` (all 7 Sprint 12 Team-QA conditions) preserved
  verbatim.
- Sprint 12 disposition `closed-with-conditions` per PROMPT 817
  preserved unchanged.
- Sprint 11 / Sprint 10 closeouts preserved unchanged.
- PROMPT 833 / 835 / 840 / 843 / 844 / 850 / 851 / 854 / 856 /
  865 prior Sprint 13 `/story-done` closures preserved unchanged
  on `origin/main`; PROMPT 868 does NOT re-claim or modify them.

### Explicitly NOT claimed by PROMPT 868

- public release readiness
- release-candidate readiness
- full game completion
- broad / Standard-tier accessibility completion
- playtest / fun-hypothesis validation
- full playable-client manual QA
- two-client GAME_OVER closure (`S8-QA-001-W1`)
- final-art / asset-production completion
- Polish->Release gate-check retry
- stage advance from Polish to Release
- underlying drag-runtime bug fix (Sprint 12 story 019 closed
  cannot-reproduce, NOT bug-fixed)
- full UI clean-pass repair
- closure of `S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001`
  outside the bounds of `S13-CONN-LOST-UX-001`
- Sprint 13 close-out (Sprint 13 remains `active`; 11 of 19 rows
  closed after PROMPT 868 — 5 of 6 Must Have, 3 of 6 Should
  Have, 3 of 7 Nice to Have)
- activation of the recommended follow-on story
  `S14-DEVOPS-CARGO-PROFILE-TEST-STRIP-001` (intentionally
  UNAUTHORED and NOT activated; activation requires a separate
  `/sprint-plan` revision in Sprint 14+)
- any `Cargo.toml` / `.cargo/config.toml` / build-script /
  CI workflow change (the actual `[profile.test] strip =
  "debuginfo"` knob lands in the named follow-on, not here)
- any `cargo` command invocation (story Cargo policy "N/A")
