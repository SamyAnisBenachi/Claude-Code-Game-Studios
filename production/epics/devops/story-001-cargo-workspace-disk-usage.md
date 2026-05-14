# Story 001: S11-TD-CARGO-DISK-USAGE-001 -- Cargo Workspace Disk-Usage Reduction Strategy

> **Epic**: DevOps (Operational Hardening)
> **Story ID**: S11-TD-CARGO-DISK-USAGE-001
> **Status**: Done — closed by PROMPT 865 `/story-done` on 2026-05-14
> against `origin/main@9a85805` (PROMPT 861 worker commit `22f5f01` →
> PROMPT 863 integration merge commit `9a85805` on `origin/main`).
> Sprint 13 remains `active`; PROMPT 865 is a per-story closure
> paperwork run only.
> **Layer**: DevOps -- Cargo workspace investigation (doc only)
> **Type**: Investigation -- doc only; no build-script change lands
> **Sprint**: Sprint 13 Nice to Have (activated by PROMPT 826; closed by
> PROMPT 865). Sprint 12 close-out deferral; re-affirmed by PROMPT 815
> disk-pressure invocation cleaning 25 GB + ~200 GB worker `target/`
> directories.
> **Authored**: 2026-05-14 by PROMPT 819
> **Authoring source-of-truth**: `origin/main@be69f5c` (PROMPT 818
> `/sprint-plan sprint-13` DRAFT)
> **Closed**: 2026-05-14 by PROMPT 865 `/story-done` at
> `origin/main@9a85805`

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

- [x] **AC1 -- Footprint baseline recorded**: GIVEN the note at
  `docs/architecture/cargo-workspace-disk-usage.md`, WHEN inspected,
  THEN it records the per-worktree `target/` footprint observed
  (one orchestrator worktree's dev target + one orchestrator
  worktree's test target, at minimum, with absolute byte counts or
  GB rounded). **PASS** — §2 of the note records the two concrete
  per-worktree `target/` sizes observed by PROMPT 815 during the
  Sprint 12 Smoke Check: 25 GB (worker worktree `class-d-diag`,
  branch `work/fixture-clientstate-init-state-001`) + ~200 GB
  (integration worktree `integration-s11-fixture-d-residuals`,
  branch `integrate/s11-fixture-d-residuals`); plus the
  45-worktree roll-up extrapolation as motivation for §3
  expected-savings ranges. Verified at note lines 95-124 on
  `origin/main@9a85805`.

- [x] **AC2 -- Trim candidates enumerated**: GIVEN the note, WHEN
  inspected, THEN at least four trim candidates are enumerated:
  shared target dir, profile-knob set (`debug`, `split-debuginfo`,
  `strip`), build-cache (`sccache` / `cachepot`), periodic
  `cargo sweep` automation. **PASS** — §3 of the note enumerates
  exactly four candidates: A (Shared `target/` directory across
  worker worktrees), B (Profile knobs: `debug`,
  `split-debuginfo`, `strip`), C (Cross-worktree build cache:
  `sccache` / `cachepot`), D (Periodic `cargo sweep --time N`
  automation). Each candidate has its own subsection (lines
  134-292) with mechanism + estimated savings + trade-offs.

- [x] **AC3 -- Trade-offs articulated**: GIVEN each enumerated
  candidate, WHEN inspected, THEN the note articulates:
  - Windows compatibility concerns (file-lock concurrency on shared
    `target/`; symbol-debug viability with profile knobs).
  - Parallel-worker invalidation behaviour.
  - CI compatibility.
  - Estimated disk savings range (rough order of magnitude).
  **PASS** — each of the four candidates in §3 has all four
  trade-off axes addressed; the comparative summary table at
  lines 296-301 collapses them into a side-by-side grid
  (Disk savings / Windows-compat risk / Parallel-worker risk /
  CI risk / Tooling cost), with Candidate A flagged "Highest-risk
  Windows file-lock concurrency concern" mitigated by the
  per-worker subpath variant.

- [x] **AC4 -- Single follow-on named**: GIVEN the note, WHEN
  inspected, THEN exactly one recommended follow-on story slug is
  named with a one-sentence rationale (e.g.,
  "S14-DEVOPS-CARGO-SHARED-TARGET-001 -- migrate orchestrator
  worktrees to a shared CARGO_TARGET_DIR with per-worker subpath
  prefix; rationale: highest expected disk-saving payoff with
  lowest CI/Cargo-correctness risk"). **PASS** — §4 names
  exactly one follow-on `S14-DEVOPS-CARGO-SHARED-TARGET-001`
  (Candidate A per-worker-subpath variant) with the one-sentence
  rationale at lines 311-318: "highest expected disk-saving
  payoff (order-of-magnitude reduction in aggregate workspace
  `target/` footprint, addressing the structural cause of the
  PROMPT 815 disk-pressure incident) with lowest Cargo-correctness
  and CI compatibility risk". Out-of-scope clause at lines
  320-328 explicitly defers Candidates B/C/D. Activation
  requirement clause at lines 330-334 states the follow-on is NOT
  activated by this story.

- [x] **AC5 -- No code / config change lands**: GIVEN the story
  commit, WHEN `git diff` is inspected, THEN no file under
  `client/`, `server/`, `shared/`, `tests/`, `Cargo.toml`,
  `.cargo/`, `.github/`, or any build script is modified. Only the
  new note (and optionally this story file's status) is touched.
  **PASS** — `git show --stat 22f5f01` (PROMPT 861 worker
  commit) + `git show --stat 9a85805` (PROMPT 863 integration
  merge) both confirm a single-file diff: `docs/architecture/
  cargo-workspace-disk-usage.md` (NEW; +376 / -0). Zero file under
  `client/` / `server/` / `shared/` / `tests/` / `Cargo.toml` /
  `.cargo/` / `.github/` / any build script touched by worker
  or integration. PROMPT 865 (this `/story-done` closure) only
  touches paperwork files (this story file + `sprint-status.yaml`
  + `session-state/active.md` + `session-state/codex-orchestrator-state.md`),
  which is the permitted disposition-preserving paperwork edit
  per AC5's "Only the new note (and optionally this story file's
  status) is touched" clause.

- [x] **AC6 -- Sprint 13 disposition preserved**: GIVEN the story
  commit, WHEN `production/sprint-status.yaml`,
  `production/sprints/sprint-13.md`, `production/stage.txt`, and
  PROMPT 761 gate-check artifact are diffed, THEN none of them are
  modified by this story. **PASS** — PROMPT 861 worker commit
  `22f5f01` and PROMPT 863 integration merge `9a85805` both touch
  exactly one file (`docs/architecture/cargo-workspace-disk-usage.md`);
  `production/sprint-status.yaml` / `production/sprints/sprint-13.md` /
  `production/stage.txt` / `production/gate-checks/gate-polish-release-2026-05-12.md`
  are all unchanged by those commits. The PROMPT 865 row-level
  `status: ready -> done` flip + `completed: 2026-05-14`
  annotation in `production/sprint-status.yaml` is the permitted
  disposition-preserving `/story-done` paperwork edit (top-level
  `sprint:` / `status:` / `stage:` unchanged); `production/sprints/sprint-13.md`
  is NOT touched by PROMPT 865 (per allowed-files list excluding
  it; precedent PROMPT 856 / 854 / 851 / 844 / 843 / 840 / 835).

- [x] **AC7 -- Cross-link to PROMPT 815 disk-pressure cleanup**:
  GIVEN the note, WHEN inspected, THEN it cross-links to the
  PROMPT 815 Sprint 12 smoke disk-pressure invocation as the
  re-affirming evidence. **PASS** — §1 of the note at lines
  68-73 cross-links to `production/qa/smoke-sprint-12-2026-05-14.md`
  § "Disk Pressure Policy Invocation" (lines 146-170 of that
  file), citing the authorised cleanup that freed 25 GB + ~200 GB
  and restored the smoke host from 0 GB free / 82 GB free entry →
  225 GB free post-cleanup. Re-affirming-evidence framing at lines
  78-81 notes the PROMPT 815 cleanup was reactive (run only when
  smoke failed with `os error 112`) and motivates the structural
  preventive measure that this note prepares.

- [x] **AC8 -- No-claim restatement embedded**: GIVEN the note,
  WHEN inspected, THEN it includes the verbatim "Status / No-Claim
  Banner" no-claim restatement. **PASS** — note lines 12-54
  carry the verbatim "Status / No-Claim Banner" restatement from
  the story file, with the PROMPT 819 authoring-prompt
  no-modification list preserved verbatim AND an additional
  PROMPT 861 implementation-prompt no-modification clause
  appended at lines 47-54 (the same pattern PROMPT 854 / PROMPT
  856 closures used: verbatim original banner + per-prompt
  inclusion).

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

---

## Authoring / Implementation / Closure Trail

- **PROMPT 819** (2026-05-14) — Authoring. Story file created at
  `production/epics/devops/story-001-cargo-workspace-disk-usage.md`
  against `origin/main@be69f5c` (PROMPT 818 Sprint 13 plan DRAFT).
  Sprint 13 candidate (Nice to Have); NOT activated. No
  sprint-status flip, no investigation-note authoring, no
  build-script changes. Paperwork only.
- **PROMPT 822** (2026-05-14) — Sprint 13 missing-story authoring
  batch. Existing story file preserved unchanged by PROMPT 822
  (already authored by PROMPT 819 prior); referenced by the
  PROMPT 822 row enumeration that landed the 10 sibling missing
  story files at integration tip `5470d30`.
- **PROMPT 823** (2026-05-14) — `/story-readiness` rerun batch.
  Verdict **READY** (no advisories; doc-only Investigation story
  with single new evidence path `docs/architecture/
  cargo-workspace-disk-usage.md`).
- **PROMPT 826** (2026-05-14) — Sprint 13 activation. Row
  `S11-TD-CARGO-DISK-USAGE-001` recorded in
  `production/sprint-status.yaml` Sprint 13 Nice to Have block
  with `status: ready`; `sprint_13_activation:` snapshot block
  records `status_at_activation: "ready"` and PROMPT 823
  readiness verdict.
- **PROMPT 827** (2026-05-14) — Sprint 13 QA plan
  (`production/qa/qa-plan-sprint-13.md` NEW). Story 001
  classification: documentation only; reviewer sign-off;
  devops-engineer review of `docs/architecture/
  cargo-workspace-disk-usage.md` (NEW). No automated test
  required (the note *is* the deliverable). No build-script
  change lands.
- **PROMPT 857** (2026-05-14) — Pre-implementation
  `/story-readiness` re-check. Confirmed READY-NOT-STARTED and
  fully parallel-safe.
- **PROMPT 861** (2026-05-14) — Worker execution (doc-only
  `/dev-story`). Worker branch `work/s13-cargo-workspace-disk-usage`;
  worker commit `22f5f01550c2f3058613a7dc100340ea6af76ec2`
  (`docs(s13/devops): S13-CARGO-WORKSPACE-DISK-USAGE
  investigation note (PROMPT 861)`). Authored
  `docs/architecture/cargo-workspace-disk-usage.md` (NEW;
  +376 / -0 lines) covering: footprint baseline (§2; AC1),
  trim candidates A/B/C/D (§3; AC2 + AC3), single recommended
  follow-on `S14-DEVOPS-CARGO-SHARED-TARGET-001` Candidate A
  per-worker-subpath variant (§4; AC4), cross-link to PROMPT
  815 disk-pressure cleanup at `production/qa/smoke-sprint-12-2026-05-14.md`
  § "Disk Pressure Policy Invocation" lines 146-170 (§1; AC7),
  verbatim Status / No-Claim Banner restatement (top of note;
  AC8). Worker pushed worker branch only
  (`origin/work/s13-cargo-workspace-disk-usage`); worker did
  NOT push `main`. No `cargo` command invoked (story Cargo
  policy "N/A"). Worker report:
  `reports/PROMPT-861-S13-Cargo-Workspace-Disk-Usage-Note.md`.
- **PROMPT 863** (2026-05-14) — Integration. Worker tip
  `22f5f01` merged into prior `origin/main@9b65439` via
  `--no-ff` merge commit
  `9a85805e184b4129128d8a5d0807af26bf138f52`
  (`integrate(s13): merge work/s13-cargo-workspace-disk-usage
  (devops story 001 / PROMPT 861) (PROMPT 863)`). Fast-forward
  push `9b65439..9a85805` to `origin/main` (no force, no
  conflict). Integration verification: `git diff --check` clean;
  `git diff --cached --check` clean; single-file diff
  `docs/architecture/cargo-workspace-disk-usage.md` (+376 / -0);
  no forbidden paths (`client/` / `server/` / `shared/` /
  `tests/` / `Cargo.toml` / `.cargo/` / `production/sprint-status.yaml`
  / `production/session-state/` / `production/stage.txt`)
  touched. Integration report:
  `reports/PROMPT-863-S13-Cargo-Workspace-Disk-Usage-Integration.md`.
  PROMPT 863 did NOT run `/story-done`.
- **PROMPT 865** (2026-05-14) — `/story-done` closure paperwork
  (this closure). Verified each AC1-AC8 against
  `origin/main@9a85805` (PROMPT 861 worker `22f5f01` + PROMPT
  863 integration merge `9a85805`); flipped Status header
  `Draft -> Done` with closure context including PROMPT 861
  worker hash + PROMPT 863 integration merge hash + PROMPT 865
  story-done; flipped AC1-AC8 checkboxes `[ ]` -> `[x]` with
  per-AC closure-evidence annotations; appended this
  Authoring / Implementation / Closure Trail section; flipped
  `production/sprint-status.yaml` Sprint 13 Nice to Have row
  `S11-TD-CARGO-DISK-USAGE-001` `status: ready -> done` with
  `completed: 2026-05-14`, `worker_prompt: 861`, `worker_commit:
  22f5f01550c2f3058613a7dc100340ea6af76ec2`,
  `integration_prompt: 863`, `integration_commit:
  9a85805e184b4129128d8a5d0807af26bf138f52`,
  `story_done_prompt: 865`, `acceptance_evidence:
  docs/architecture/cargo-workspace-disk-usage.md`; refreshed
  top-level `updated:` annotation for PROMPT 865; appended
  `sprint_13_story_done:` entry at end of file as a sibling to
  the prior PROMPT 833 / 840 / 843 / 844 / 850 / 851 / 854 /
  856 entries. Prepended PROMPT 865 banners to
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
  preserved untouched. PROMPT 865 final report:
  `reports/PROMPT-865-S13-Cargo-Workspace-Disk-Usage-Story-Done.md`
  (NOT staged or committed; `reports/` is gitignored).

### Conditions carried forward unchanged by PROMPT 865

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
  865 does NOT re-attempt the Sprint 12 capture).
- PROMPT 761 Polish->Release gate-check `FAIL` preserved at
  `production/gate-checks/gate-polish-release-2026-05-12.md`; no
  retry in PROMPT 865 scope.
- Story 019 (Sprint 12 hand-ui) underlying drag-runtime bug NOT
  claimed fixed.
- `TQ-S12-C1..C7` (all 7 Sprint 12 Team-QA conditions) preserved
  verbatim.
- Sprint 12 disposition `closed-with-conditions` per PROMPT 817
  preserved unchanged.
- Sprint 11 / Sprint 10 closeouts preserved unchanged.
- PROMPT 833 / 835 / 840 / 843 / 844 / 850 / 851 / 854 / 856
  prior Sprint 13 `/story-done` closures preserved unchanged on
  `origin/main`; PROMPT 865 does NOT re-claim or modify them.

### Explicitly NOT claimed by PROMPT 865

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
- Sprint 13 close-out (Sprint 13 remains `active`; 10 of 19 rows
  closed after PROMPT 865 — 5 of 6 Must Have, 3 of 6 Should
  Have, 2 of 7 Nice to Have)
- activation of the recommended follow-on story
  `S14-DEVOPS-CARGO-SHARED-TARGET-001` (intentionally UNAUTHORED
  and NOT activated; activation requires a separate
  `/sprint-plan` revision in Sprint 14+)
- any `Cargo.toml` / `.cargo/config.toml` / build-script /
  CI workflow change (preventive disk-usage measure deferred to
  the recommended follow-on)
- any `sccache` / `cachepot` / `cargo-sweep` tooling install
- any `cargo` command invocation (story Cargo policy "N/A")
