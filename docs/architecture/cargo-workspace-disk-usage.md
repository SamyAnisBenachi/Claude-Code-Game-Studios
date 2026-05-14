# Cargo Workspace Disk Usage — Investigation Note

> **Story**: `S11-TD-CARGO-DISK-USAGE-001` (Sprint 13 Nice to Have — DevOps epic story 001)
> **Authored**: 2026-05-14 by PROMPT 861 (implementation of doc-only `/dev-story`)
> **Authoring source-of-truth**: `origin/main@9b65439` (PROMPT 856 `/story-done`
> S13-PROTO-ORPHAN-DRAIN-001 closure commit)
> **Status**: Investigation note only — **NO BUILD-SCRIPT CHANGES LAND**
> **Scope**: per AC1–AC8 in `production/epics/devops/story-001-cargo-workspace-disk-usage.md`

---

## Status / No-Claim Banner (verbatim restatement)

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

In addition to the above (which is the verbatim restatement from the
story file's `## Status / No-Claim Banner` section), this note (PROMPT
861, the implementation of the doc-only `/dev-story`) does NOT itself
activate any change to `Cargo.toml`, `.cargo/`, `.github/`,
`production/sprint-status.yaml`, `production/sprints/sprint-13.md`, or
`production/stage.txt`; the only files modified by PROMPT 861 are this
new note and (on `/story-done`, in a future prompt) the story-001 status
line.

---

## 1. Source Finding & Cross-Reference (AC7)

The concern under investigation was raised at Sprint 11 close-out as
`S11-TD-CARGO-DISK-USAGE-001` (a developer-quality-of-life signal) and
re-affirmed at Sprint 12 close-out (PROMPT 817) which deferred the row
forward into Sprint 13 planning. The re-affirming concrete evidence is
**PROMPT 815 (Sprint 12 Smoke Check)** which hit a disk-pressure
threshold mid-run and required two reactive Cargo cleanup invocations
on existing per-worktree `target/` directories.

**Cross-link to PROMPT 815 disk-pressure cleanup** (AC7):
`production/qa/smoke-sprint-12-2026-05-14.md` § **Disk Pressure Policy
Invocation** (lines 146–170 of that file). That section records the
authorised cleanup of two specific `target/` directories whose
combined freed space (25 GB + ~200 GB) restored the smoke host from
0 GB free / 82 GB free entry → 225 GB free post-cleanup.

The PROMPT 815 cleanup was **reactive** (run only when the smoke
invocation failed with `error: failed to write query cache to ...
There is not enough space on the disk. (os error 112)`). The Sprint
12 close-out concluded that the underlying disk-pressure pattern is
structural to the current per-worktree `target/` layout under the
parallel-worker orchestrator pattern and that a preventive measure
(rather than reactive cleanup) belongs in Sprint 14+.

This note is the prep work for that preventive measure: it does not
choose or land the fix, but enumerates the candidate space, articulates
trade-offs, and names a single recommended follow-on story (§4).

---

## 2. Footprint Baseline (AC1)

The two concrete per-worktree `target/` sizes observed and recorded by
PROMPT 815 during the Sprint 12 Smoke Check (re-affirming evidence
cross-referenced in §1):

| Worktree | Branch | `target/` size | Notes |
|---|---|---|---|
| `D:\_DEV\claude-code-game-studios-worktrees\class-d-diag\target` | `work/fixture-clientstate-init-state-001` | **25 GB** | Old, no-longer-active worker worktree, May 12 timestamp. Predates current Sprint 11 / Sprint 12 work. Treat as representative of a single worker's dev+test build cache after one full integration cycle (compile + `cargo test --workspace`). |
| `D:\_DEV\claude-code-game-studios-worktrees\integration-s11-fixture-d-residuals\target` | `integrate/s11-fixture-d-residuals` | **~200 GB** | Integration worktree whose work is already on `origin/main` (PROMPT 813 integration commit `a3c624e`). Treat as representative of a heavily-exercised integration worktree's `target/` after multiple `cargo test --workspace` runs and many incremental rebuilds (debug + test binaries + PDBs + dep-info). |

Both directories were deleted by PROMPT 815 under the prompt's
disk-pressure policy authorisation (`Remove-Item -LiteralPath ...
-Recurse -Force`), restoring 225 GB free. The deletion itself is
preserved in the PROMPT 815 smoke evidence and is **not** repeated or
reversed by this note.

**Roll-up estimate**. The orchestrator pattern at PROMPT 861 time runs
**~45 active git worktrees** (per `git worktree list`), each with the
default Cargo behaviour of a per-worktree `target/` directory. If a
single worker `target/` averages on the order of ~10–25 GB after a
typical `cargo test --workspace --tests --no-fail-fast` invocation
(extrapolating from the PROMPT 815 25 GB datapoint as a representative
single-worker run), then even at the low end the steady-state
workspace-wide `target/` footprint runs to several hundred GB on the
D: drive of the orchestrator host. Heavily-exercised worktrees can
reach an order of magnitude higher (the PROMPT 815 ~200 GB datapoint
on the `integrate/s11-fixture-d-residuals` `target/` is the
empirically observed upper bound to date).

The 25 GB / ~200 GB pair is the **only datapoint** this note relies on
for AC1; the roll-up is a back-of-envelope extrapolation, not an
authoritative measurement, and is included only to motivate §3's
expected-savings ranges. The actual disk-saving payoff of the §4
follow-on must be re-measured at implementation time on at least two
worker worktrees plus one integration worktree.

---

## 3. Trim Candidates and Trade-offs (AC2, AC3)

Four candidate preventive measures are enumerated below. **None of
them lands under this story** (per AC5). The follow-on story (§4)
selects exactly one of these to pursue.

### Candidate A — Shared `target/` directory across worker worktrees

**Mechanism**. Set `CARGO_TARGET_DIR=...` (or `[build] target-dir =
"..."` in `.cargo/config.toml`) so every worktree's Cargo invocation
resolves to a single shared on-disk `target/` location, instead of
each worktree maintaining its own. Cargo's incremental cache and dep
graph live in `target/`; sharing the directory means parallel workers
share that cache.

**Estimated disk savings**. **Highest** of the four candidates.
Order-of-magnitude reduction: from ~45 × per-worktree `target/` to
**one** `target/`. Empirical lower-bound expectation from the §2
roll-up: if the current steady-state is on the order of several
hundred GB across all worktrees combined, a single shared `target/`
should cap that at a single worktree's worth (tens of GB), saving
hundreds of GB in aggregate.

**Trade-offs**:

- **Windows compatibility (file-lock concurrency)**. **Highest-risk
  concern of all four candidates.** Cargo locks `target/.cargo-lock`
  during builds, and Windows file-locking is strict (lock holders
  block read-AND-write from other processes, not just write). Two
  parallel Cargo invocations against the same shared `target/` will
  serialise on the lock; one will wait. This may eliminate the
  parallelism benefit that the orchestrator pattern relies on for
  multiple workers building / testing concurrently. Mitigation: use
  a **per-worker subpath prefix** under the shared `target-dir`
  (e.g. `CARGO_TARGET_DIR=D:\_cargo\$WORKTREE_SLUG`) so each worker
  still gets its own `target/`, but they all live under a single
  controlled root directory that can be `cargo sweep`'d as a unit.
  This trades the "single shared cache" payoff for a "controlled
  pool" payoff.
- **Parallel-worker invalidation behaviour**. With a single shared
  `target/`, divergent worker branches checking out different
  `Cargo.lock` / different source content will cause Cargo to
  invalidate and rebuild large fractions of the dep graph on each
  worker switch. Effective cache hit rate could be lower than
  per-worktree caches for workers on long-divergent branches.
  Mitigation (per-worker subpath): preserves per-worker incremental
  hit rate; only the parent dir is shared, not the cache itself.
- **CI compatibility**. CI runs on ephemeral runners and is not
  affected by orchestrator-host disk pressure. A `.cargo/config.toml`
  `target-dir` override would also apply on CI runners, which may
  resolve to non-writable / non-existent paths there. Mitigation:
  use the env var `CARGO_TARGET_DIR` set per-shell on the
  orchestrator host only, **not** `.cargo/config.toml` at the repo
  root.

### Candidate B — Profile knobs (`debug`, `split-debuginfo`, `strip`)

**Mechanism**. Tune the dev / test build profiles in `Cargo.toml` so
each compiled artifact is smaller on disk. Concrete knobs:

- `[profile.dev] debug = "line-tables-only"` (or `debug = 1`)
  instead of the default `debug = 2`. Drops most DWARF / PDB symbol
  detail, keeping just enough for line-number-only backtraces.
- `[profile.test] debug = "line-tables-only"` (same rationale for
  test binaries).
- `[profile.dev] split-debuginfo = "packed"` (Linux/macOS) or
  `"unpacked"` (Windows / MSVC) so debug info goes to separate
  files that are easier to sweep without invalidating the
  executable cache.
- `[profile.release] strip = "debuginfo"` (already implicitly true
  for the WASM client per Trunk's release profile; document the
  policy explicitly).

**Estimated disk savings**. Moderate. PDB / DWARF symbols are
typically 40–70% of a Rust debug binary's on-disk size. Across a
full workspace test build (~200 test binaries × server + client +
shared crates per PROMPT 815 baseline of 189-binary aggregate),
this could plausibly halve a single worktree's `target/`. Lower
order of magnitude than Candidate A.

**Trade-offs**:

- **Windows compatibility (PDB / symbol viability)**. `debug =
  "line-tables-only"` keeps backtraces functional — line numbers
  survive — but strips inline-frame info and named-locals. Acceptable
  for the orchestrator's typical workflow (test failures with line
  numbers); may degrade WinDbg / Visual Studio debugger sessions
  if those are ever used. Mitigation: confine to dev/test
  profiles; release-symbol artefacts for distribution use a
  separate profile.
- **Parallel-worker invalidation behaviour**. Profile knobs apply
  uniformly to every worker; a profile change in `Cargo.toml`
  invalidates every active worktree's `target/` on first rebuild.
  This is a **one-time cost** but it is large (every worktree
  recompiles from scratch on its next build). Mitigation: time the
  change at a sprint-boundary cleanup point.
- **CI compatibility**. CI also recompiles on first build after the
  `Cargo.toml` change. CI runners have ephemeral disk, so smaller
  artefacts speed up runner provisioning slightly; no negative CI
  impact expected. CI debug output (test failure backtraces) will
  show line-numbers-only, which is the typical CI norm.

### Candidate C — Cross-worktree build cache (`sccache` / `cachepot`)

**Mechanism**. Install `sccache` (or its drop-in fork `cachepot`)
and set `RUSTC_WRAPPER=sccache` so identical source files
(content-addressed) produce identical compiled `.rlib` outputs that
are shared across worktrees and across CI / dev hosts. Cache backend
options: local disk (default), Redis, S3.

**Estimated disk savings**. Cache itself **adds** disk usage (the
content-addressed cache directory grows as workers populate it).
Net **disk** savings: low. Net **build time** savings: high — cold
worktree builds with a warm sccache cache hit rate of 60–80% can
shave significant compile time. This is more of a build-speed
candidate than a disk-savings candidate; included here per AC2 but
ranked low on the disk-saving payoff metric this story targets.

**Trade-offs**:

- **Windows compatibility**. `sccache` works on Windows but has had
  historical edge cases around long-path support and the MSVC
  `/showIncludes` parser. Functional in 2025–2026 versions but
  requires Long Path Support enabled on the Windows host.
- **Parallel-worker invalidation behaviour**. Content-addressed; no
  worker invalidates another worker's cache entries.
- **CI compatibility**. Compatible (this is `sccache`'s primary use
  case). Requires a shared backend (S3 / Redis) for CI to benefit
  cross-runner.
- **Tooling-dependency cost**. Adds a tool to install + version-pin
  on every developer + CI host. Story-001 forbids this story from
  adding any tooling dependency (per its Control Manifest
  Forbidden list), so this candidate is enumerated but cannot land
  under this story; it is preserved as an option for the follow-on.

### Candidate D — Periodic `cargo sweep --time N` automation

**Mechanism**. Schedule `cargo sweep --time N` (where N is days, e.g.
`--time 7`) as a periodic task on the orchestrator host to delete
build artifacts that have not been touched in N days. Optionally
combined with `cargo sweep --installed` to delete artifacts from
toolchain versions that are no longer installed.

**Estimated disk savings**. Moderate to high. The 25 GB cleanup in
PROMPT 815 (on a `target/` for a worktree on the May-12 branch that
had not been built in ~2 days) is exactly the case `cargo sweep
--time 1` or `--time 2` would have handled preventively. Aggregate
saving over the 45-worktree footprint depends on actual build
recency distribution.

**Trade-offs**:

- **Windows compatibility**. `cargo sweep` is cross-platform; no
  Windows-specific concerns. Runs as a normal CLI command.
- **Parallel-worker invalidation behaviour**. Periodic sweep can
  delete `target/` content for a worktree that an in-flight worker
  is about to rebuild from. Mitigation: schedule sweep during a
  known-idle window or use `--time` thresholds large enough to
  exclude any active sprint's worktrees.
- **CI compatibility**. Not applicable — sweep is host-local; CI
  runners are ephemeral.
- **Tooling-dependency cost**. Adds `cargo install cargo-sweep` to
  the developer environment. Same constraint as Candidate C: this
  story forbids adding the tooling under itself; the follow-on
  story would handle the install + scheduling.

### Comparative summary

| Candidate | Disk savings | Windows-compat risk | Parallel-worker risk | CI risk | Tooling cost |
|---|---|---|---|---|---|
| A — Shared target dir (per-worker subpath variant) | **High** (hundreds of GB) | Medium (file-lock; mitigated by per-worker subpath) | Low (per-worker subpath isolates caches) | Low (env var, not `.cargo/config.toml`) | None |
| B — Profile knobs | Moderate (~50% per `target/`) | Low (line-tables-only preserves backtraces) | Medium (one-time mass invalidation) | Low | None |
| C — sccache / cachepot | Low net (cache itself grows) | Medium (long-path) | Low (content-addressed) | Low (compatible) | High (new tool) |
| D — `cargo sweep` automation | Moderate (matches PROMPT 815 25 GB precedent) | Low | Medium (must avoid in-flight worktrees) | N/A | Medium (new tool) |

---

## 4. Recommended Follow-on (AC4) — single only

**`S14-DEVOPS-CARGO-SHARED-TARGET-001`** — migrate orchestrator
worktrees to a shared `CARGO_TARGET_DIR` with per-worker subpath
prefix (Candidate A, per-worker-subpath variant).

**Rationale (one sentence)**: highest expected disk-saving payoff
(order-of-magnitude reduction in aggregate workspace `target/`
footprint, addressing the structural cause of the PROMPT 815
disk-pressure incident) with lowest Cargo-correctness and CI
compatibility risk, because the per-worker subpath form preserves
each worker's incremental-build cache isolation while consolidating
all `target/` content under one controlled root that future
`cargo sweep` / cleanup work can safely operate on as a unit.

**Out-of-scope for the follow-on (preserved here only to constrain
its scope, not to land under this story)**:

- Profile knob changes (Candidate B): defer; only consider after
  Candidate A's measured savings are insufficient.
- sccache / cachepot install (Candidate C): defer; tooling-dep
  decision belongs in its own story if pursued at all.
- `cargo sweep` automation (Candidate D): defer; complementary to
  Candidate A but adds a tooling dep and is separately authored.

**Activation requirement**: this follow-on is **not** activated by
this story; its activation requires a separate `/sprint-plan`
revision in Sprint 14+ (per the parent story's
"Dependency Notes Against Sprint 13 Active Scope" closing
paragraph).

---

## 5. Files Modified by PROMPT 861 (Implementation Trail)

| Path | Status |
|---|---|
| `docs/architecture/cargo-workspace-disk-usage.md` | NEW — this note. |

Explicitly **not** modified by PROMPT 861 (per AC5, AC6, and the
story's Out of Scope list): any file under `client/`, `server/`,
`shared/`, `tests/`, `Cargo.toml`, `.cargo/`, `.github/`, any build
script, `production/sprint-status.yaml`,
`production/sprints/sprint-13.md`, `production/stage.txt`,
`production/session-state/*`, `.claude/settings.json`. The story-001
status line itself is also **not** flipped to Done by PROMPT 861
(that is `/story-done`'s job in a future prompt).

**Authoring Trail**:

- PROMPT 819 — authored `production/epics/devops/story-001-cargo-workspace-disk-usage.md` (Draft, Sprint 13 candidate; NOT activated).
- PROMPT 857 — confirmed READY-NOT-STARTED and fully parallel-safe.
- **PROMPT 861 (this note)** — implementation of the doc-only `/dev-story`; authored this investigation note. **No build-script / source / config change.**

---

## 6. Acceptance Criteria Self-Check (informational only)

| AC | Status | Where in this note |
|---|---|---|
| AC1 — Footprint baseline recorded | Done | §2 (25 GB + ~200 GB observed; 45-worktree roll-up). |
| AC2 — Trim candidates enumerated (≥4) | Done | §3 (Candidates A, B, C, D). |
| AC3 — Trade-offs articulated | Done | §3 (Windows-compat, parallel-worker invalidation, CI compat, savings range per candidate; comparative summary table). |
| AC4 — Single follow-on named | Done | §4 (`S14-DEVOPS-CARGO-SHARED-TARGET-001`; one-sentence rationale). |
| AC5 — No code/config change lands | Done | §5 (only file modified is this new note). |
| AC6 — Sprint 13 disposition preserved | Done | §5 (sprint-status.yaml / sprint-13.md / stage.txt / PROMPT 761 gate-check artifact all unmodified). |
| AC7 — Cross-link to PROMPT 815 disk-pressure cleanup | Done | §1 (cross-links `production/qa/smoke-sprint-12-2026-05-14.md` § Disk Pressure Policy Invocation, lines 146–170). |
| AC8 — No-claim restatement embedded | Done | top of note (verbatim Status / No-Claim Banner from story file). |

The AC self-check above is informational and does **not** itself
verify the story; that verification belongs to `/story-readiness` and
`/story-done` in future prompts.
