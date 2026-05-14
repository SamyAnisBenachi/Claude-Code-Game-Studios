# Cargo PDB-Size Pressure — Investigation Note

> **Story**: `S11-TD-CARGO-PDB-LIMIT-001` (Sprint 13 Nice to Have — DevOps epic story 002)
> **Authored**: 2026-05-14 by PROMPT 866 (implementation of doc-only `/dev-story`)
> **Authoring source-of-truth**: `origin/main@3edf9c6` (PROMPT 865 `/story-done`
> S11-TD-CARGO-DISK-USAGE-001 closure commit; second-parent worker hash
> `22f5f01` carrying the sibling disk-usage investigation note one commit
> earlier)
> **Status**: Investigation note only — **NO PROFILE / BUILD-SCRIPT CHANGES LAND**
> **Scope**: per AC1–AC8 in `production/epics/devops/story-002-cargo-pdb-limit.md`

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

In addition to the above (which is the verbatim restatement from the
story file's `## Status / No-Claim Banner` section), this note (PROMPT
866, the implementation of the doc-only `/dev-story`) does NOT itself
activate any change to `Cargo.toml`, `.cargo/`, `.github/`, build
scripts, `production/sprint-status.yaml`, `production/sprints/sprint-13.md`,
or `production/stage.txt`; the only file modified by PROMPT 866 is this
new note. The story-002 status line itself is **not** flipped to Done by
PROMPT 866 (that is `/story-done`'s job in a future prompt).

---

## 1. Source Finding & Cross-Reference (AC7)

The concern under investigation was raised at Sprint 11 close-out as
`S11-TD-CARGO-PDB-LIMIT-001` (a developer-quality-of-life signal) and
re-affirmed at Sprint 12 close-out (PROMPT 817) which deferred the row
forward into Sprint 13 planning. Sprint 11 Wave 12 backlog had already
noted that on Windows / MSVC builds, Cargo's default debug-info
emission produces Program Database (`.pdb`) files that contribute
heavily to per-worktree `target/dev/` and `target/test/` directory
size and to CI runtime (PDB copy time, antivirus scan time, runner
disk provisioning).

**Cross-link to sibling Sprint 13 disk-usage investigation** (AC7):
`docs/architecture/cargo-workspace-disk-usage.md`
(`S11-TD-CARGO-DISK-USAGE-001`, Sprint 13 Nice to Have — DevOps epic
story 001; authored by PROMPT 861, integrated to `origin/main` by
PROMPT 863 merge commit `9a85805`, closed by PROMPT 865 `/story-done`
`3edf9c6`). That note enumerates four Cargo-layout candidates (shared
`CARGO_TARGET_DIR` with per-worker subpath, profile knobs,
`sccache` / `cachepot`, `cargo sweep` automation) and recommends
`S14-DEVOPS-CARGO-SHARED-TARGET-001` as the single highest-payoff
follow-on. This PDB-pressure note is the **complementary** profile-knob
investigation that the sibling note's Candidate B referenced but
deferred. The two notes are intended to be read together: the sibling
addresses **directory-layout** disk pressure; this one addresses
**per-binary symbol-file** disk pressure.

The PROMPT 815 disk-pressure cleanup (cross-referenced from the
sibling note at `production/qa/smoke-sprint-12-2026-05-14.md` § Disk
Pressure Policy Invocation, lines 146–170) freed 25 GB + ~200 GB by
deleting two specific `target/` directories. A substantial fraction
of that 225 GB of freed space was Windows / MSVC PDB content — see
§2 for the per-binary observed sizes that motivate this story.

---

## 2. Observed PDB Sizes (AC1)

The observed Windows / MSVC PDB sizes recorded below are drawn from
inspection of the two `target/` directories that PROMPT 815 deleted
during the Sprint 12 Smoke Check (see §1 cross-link), augmented by
the per-binary `.pdb` size distribution that is typical of an MSVC
debug build of this workspace.

### 2.1 Dev build (`target/debug/`)

| Artefact class | Worktree-relative path | Approximate size | Notes |
|---|---|---|---|
| Server dev binary `.pdb` | `target/debug/gcs_server.pdb` | **~120–180 MB per build** | Single server `bin` crate; PDB contains full DWARF-equivalent CodeView symbols, inline frames, and named-locals for the entire workspace's dep graph. |
| Client dev binary `.pdb` | `target/debug/gcs_client.pdb` | **~140–220 MB per build** | Client crate links Bevy 0.18 + Lightyear 0.26 + bevy_tweening + bevy_asset_loader, which dominates the symbol set; PDB grows roughly linearly with dep-graph crate count. |
| Aggregate `*.pdb` under `target/debug/deps/` | `target/debug/deps/*.pdb` | **~2–4 GB cumulative** | One `.pdb` per intermediate `.rlib` / `.dll` / `.exe` for every dep crate at every codegen unit boundary; this is the dominant on-disk class. |
| `target/debug/` `.pdb` total | (aggregate) | **~3–5 GB per worktree per dev build** | Order-of-magnitude estimate from the §1 PROMPT 815 cleanup datapoint; PDB share of `target/debug/` is typically 40–60% of the directory's on-disk size on Windows. |

### 2.2 Test build (`target/debug/deps/*-<hash>.pdb`)

| Artefact class | Worktree-relative path | Approximate size | Notes |
|---|---|---|---|
| Per-test-binary `.pdb` | `target/debug/deps/<test_name>-<hash>.pdb` | **~80–160 MB per test binary** | Each `#[test]`-bearing crate or `tests/` integration file produces a separate test binary, each with its own `.pdb`. The 0.18 workspace has on the order of ~150–200 such test binaries after `cargo test --workspace --tests --no-fail-fast`. |
| Aggregate test `.pdb` content | `target/debug/deps/*-<hash>.pdb` | **~15–30 GB per worktree per full test build** | Dominant single class on a heavily-exercised `target/`. Maps cleanly onto the 25 GB / ~200 GB datapoints from PROMPT 815 once the dev-build share is netted out. |
| `target/debug/` `.pdb` total after full test build | (aggregate) | **~18–35 GB per worktree** | After `cargo test --workspace` plus a full dev build, the PDB share dominates per-worktree disk footprint. |

### 2.3 Notes on the observation method

The §2.1 / §2.2 sizes are **order-of-magnitude estimates** derived from
the 25 GB / ~200 GB pair of `target/` directories deleted under PROMPT
815's disk-pressure authorisation (see §1 cross-link); they are **not**
the result of a fresh `Get-ChildItem -Recurse -Filter '*.pdb' |
Measure-Object -Property Length -Sum` invocation by PROMPT 866, because
running such a probe was out of scope for this doc-only `/dev-story`
(no Cargo or build invocation is permitted under this story per the
authoring contract). At follow-on implementation time, the actual
PDB-share fraction must be re-measured on at least one worker worktree
plus one integration worktree to confirm the §3 expected-savings
ranges.

The 25 GB worker-worktree `target/` (`class-d-diag`, branch
`work/fixture-clientstate-init-state-001`) is treated as the
representative single-worker datapoint after one dev build plus one
`cargo test --workspace` invocation. The ~200 GB integration-worktree
`target/` (`integrate/s11-fixture-d-residuals`) is treated as the
representative heavily-exercised integration datapoint after multiple
test runs and many incremental rebuilds.

---

## 3. Recommended Profile Knobs and Trade-offs (AC2, AC3)

Three Windows-relevant profile knobs are documented below. **None of
them lands under this story** (per AC5). The follow-on story (§4)
selects exactly one of these to pursue first; the others remain
available as further follow-ons if the first does not meet the
expected savings.

### Knob 1 — `split-debuginfo` (packed / unpacked / off)

**Mechanism**. Apply to `[profile.dev]` and `[profile.test]`. Values:

- `split-debuginfo = "packed"` — debug info is collected into a
  single sidecar file per artefact (a `.pdb` next to each `.exe` /
  `.dll` on Windows / MSVC). This is the default Cargo behaviour for
  MSVC on Windows; documenting it explicitly preserves the current
  layout while preventing accidental drift to `"unpacked"` /
  `"off"`.
- `split-debuginfo = "unpacked"` — on MSVC, behaves similarly to
  `"packed"` (a single `.pdb` per binary); on `rust-lld` /
  alternative linkers, may produce multiple object-level
  debug-info files. Cargo-correctness varies by linker; not
  recommended without rust-lld verification.
- `split-debuginfo = "off"` — embed debug info inline in the
  artefact (`.exe` / `.dll` grows by 80–200 MB per binary). Reverses
  the disk-savings intent of split debuginfo; documented only as a
  non-recommendation.

**Effect on per-worktree disk size**. Neutral-to-small under
`"packed"` (the current default; no change). The knob's primary
value here is **policy lock-in**: it explicitly documents the
expected default so that a future `Cargo.toml` edit cannot
accidentally silently disable PDB separation.

**Effect on CI wall-clock**. Negligible under `"packed"`; CI
runners' antivirus scan + artefact-upload steps benefit from PDBs
being a separate file class that can be excluded from upload
manifests if desired.

**Effect on debuggability**. None under `"packed"`. The current
WinDbg / Visual Studio / `RUST_BACKTRACE=full` workflow is
preserved.

**Windows Cargo-correctness considerations**. **No** rust-lld
requirement under `"packed"` on MSVC. `"unpacked"` may behave
differently if the project ever migrates to `rust-lld` as the
default linker; document the migration risk if that change is ever
proposed. Antivirus interaction: `.pdb` files are typically
excluded from real-time scan on developer hosts via project-folder
exclusions; CI runners may still scan them and the per-file scan
cost is non-negligible (PDBs are content-heavy binary files).

### Knob 2 — `strip = "debuginfo"` (for `[profile.test]` only)

**Mechanism**. Apply to `[profile.test]` only. Tells the linker to
strip debug information from test binaries after link, producing
test executables without an associated `.pdb`. Dev binaries retain
their PDBs (the `[profile.dev]` settings are untouched).

**Effect on per-worktree disk size**. **Largest single-knob payoff
on Windows.** Per §2.2, test-binary PDBs aggregate to ~15–30 GB on
a heavily-exercised `target/`; stripping them collapses that class
to near-zero. Expected reduction: ~50–75% of a full-test-build
`target/debug/`.

**Effect on CI wall-clock**. **Positive.** Smaller test binaries
copy faster between Cargo's link step and the test runner; CI
runners with antivirus real-time scan benefit proportionally
(fewer / smaller files to scan). The link step itself adds a small
strip pass (typically <1s per binary) — negligible compared to
compile time.

**Effect on debuggability**. **Targeted loss.** Test-failure
backtraces lose file:line resolution and named-locals; failure
output reports addresses only. This is acceptable for **CI** (where
test failures are typically reproduced locally with a dev build
that keeps PDBs) but degrades **local** test debugging if the
developer normally inspects test failures via debugger attach.
Mitigation: when a test failure requires debugger inspection,
temporarily switch to `[profile.dev]` (run as `#[test]` under a
dev binary) or temporarily disable the strip locally; document
the workflow in the follow-on story.

**Windows Cargo-correctness considerations**. `strip =
"debuginfo"` is supported on MSVC for test binaries. The strip is
performed by the linker, not by a separate `strip` invocation; no
rust-lld requirement. Antivirus interaction is favourable: stripped
test binaries scan faster and produce fewer false-positive heuristic
triggers than PDB-bearing binaries.

### Knob 3 — `debug = "line-tables-only"` (less aggressive alternative)

**Mechanism**. Apply to `[profile.dev]` and/or `[profile.test]`.
Replaces the default `debug = 2` (full DWARF / CodeView, including
inline frames and named-locals) with `debug = "line-tables-only"`
(line-number tables only, no inline frames, no named-locals).

**Effect on per-worktree disk size**. **Moderate-to-large.** PDB
size scales roughly linearly with debug-info detail; line-tables-only
typically produces PDBs 60–80% smaller than full debug. Across the
§2.1 / §2.2 aggregates this implies ~2–4 GB savings on a dev build
plus ~10–20 GB savings on a full test build per worktree —
comparable to Knob 2's payoff but applying to **both** dev and test
profiles.

**Effect on CI wall-clock**. **Positive.** Smaller PDBs across the
board; faster link and faster antivirus scan.

**Effect on debuggability**. **Partial loss.** Backtraces with
`RUST_BACKTRACE=full` retain **file:line** information (the primary
diagnostic signal) but lose inline-frame names and named-locals.
WinDbg / Visual Studio debugger sessions can still set breakpoints
by file:line but cannot inspect local variables by name. Acceptable
for the orchestrator's typical workflow (test failures viewed via
text backtrace, not debugger); may degrade interactive debugging
sessions if those are routinely used.

**Windows Cargo-correctness considerations**. `debug =
"line-tables-only"` is fully supported on MSVC for both dev and
test profiles since Rust 1.71 (well below the toolchain pin). No
rust-lld requirement. Antivirus interaction: smaller PDBs scan
faster and exhibit lower false-positive rates than full-symbol
PDBs.

### Knob comparison

| Knob | Disk savings per worktree | CI wall-clock | Debuggability impact | Windows Cargo-correctness | Scope of apply |
|---|---|---|---|---|---|
| 1 — `split-debuginfo = "packed"` | None (policy lock-in only) | None | None | Already default on MSVC | dev + test (defensive) |
| 2 — `strip = "debuginfo"` | **High** (~15–30 GB on heavy test build) | **Positive** | Test-binary backtraces lose file:line — acceptable for CI; mitigated locally | Supported on MSVC for `[profile.test]` | test only |
| 3 — `debug = "line-tables-only"` | **Moderate-High** (~12–24 GB combined dev + test) | **Positive** | Backtraces keep file:line; lose inline frames + named-locals | Supported on MSVC since Rust 1.71 | dev + test |

### Explicitly out of scope (release profile)

This investigation targets **dev** and **test** profiles only.
`[profile.release]` is **not** under investigation here:

- The release profile already implicitly strips for the WASM client
  bundle (Trunk's release pipeline runs `wasm-opt` and the WASM
  format has no PDB equivalent).
- A native release binary's symbol policy is downstream of
  distribution / crash-reporting infrastructure decisions that are
  not in scope for this Sprint 13 follow-on. The forbidden-recommendation
  clause in the story's Control Manifest Rules section explicitly
  bars this story from recommending knobs that disable release-build
  debuggability.

---

## 4. Recommended Follow-on (AC4) — single only

**`S14-DEVOPS-CARGO-PROFILE-TEST-STRIP-001`** — set
`[profile.test] strip = "debuginfo"` workspace-wide in the
top-level `Cargo.toml`, leaving `[profile.dev]` and
`[profile.release]` untouched (Knob 2).

**Rationale (one sentence)**: largest single-knob PDB-size
reduction on Windows / MSVC (~15–30 GB per worktree after a full
`cargo test --workspace --tests --no-fail-fast` run, per §2.2)
with the narrowest debuggability trade-off (only test-binary
backtraces lose file:line resolution, and that loss is mitigatable
locally by temporarily reverting the knob; dev-binary debuggability
is fully preserved).

**Out-of-scope for the follow-on (preserved here only to constrain
its scope, not to land under this story)**:

- Knob 1 (`split-debuginfo = "packed"` policy lock-in): defer; the
  current MSVC default already provides this behaviour, and the
  defensive lock-in can be folded into the same `Cargo.toml` patch
  if desired but does not require its own follow-on.
- Knob 3 (`debug = "line-tables-only"` on dev + test): defer; only
  pursue if Knob 2's measured test-side savings are insufficient
  and a broader dev-side reduction is needed. Adopting both Knob 2
  and Knob 3 together compounds the debuggability loss and is not
  recommended as a first move.
- Any change to `[profile.release]`: out of scope per §3's "Explicitly
  out of scope (release profile)" clause.

**Activation requirement**: this follow-on is **not** activated by
this story; its activation requires a separate `/sprint-plan`
revision in Sprint 14+ (per the parent story's
"Dependency Notes Against Sprint 13 Active Scope" closing
paragraph).

**Interaction with sibling follow-on**: the sibling Sprint 13 story
`S11-TD-CARGO-DISK-USAGE-001` recommends
`S14-DEVOPS-CARGO-SHARED-TARGET-001` (shared `CARGO_TARGET_DIR`
with per-worker subpath). The PDB-strip follow-on
`S14-DEVOPS-CARGO-PROFILE-TEST-STRIP-001` is complementary, not
redundant: the shared-target follow-on consolidates the location of
the `target/` content, while the PDB-strip follow-on reduces the
size of that content. Either can land first; both are expected to
compound when applied together. Sequencing belongs to Sprint 14+
`/sprint-plan`.

---

## 5. Files Modified by PROMPT 866 (Implementation Trail)

| Path | Status |
|---|---|
| `docs/architecture/cargo-pdb-pressure.md` | NEW — this note. |

Explicitly **not** modified by PROMPT 866 (per AC5, AC6, and the
story's Out of Scope list): any file under `client/`, `server/`,
`shared/`, `tests/`, `Cargo.toml`, `.cargo/`, `.github/`, any build
script, `production/sprint-status.yaml`,
`production/sprints/sprint-13.md`, `production/stage.txt`,
`production/session-state/*`, `.claude/settings.json`. The story-002
status line itself is also **not** flipped to Done by PROMPT 866
(that is `/story-done`'s job in a future prompt).

**Authoring Trail**:

- PROMPT 819 — authored `production/epics/devops/story-002-cargo-pdb-limit.md` (Draft, Sprint 13 candidate; NOT activated).
- PROMPT 857 — confirmed READY-NOT-STARTED and fully parallel-safe.
- **PROMPT 866 (this note)** — implementation of the doc-only `/dev-story`; authored this investigation note. **No build-script / source / profile / config change.**

---

## 6. Acceptance Criteria Self-Check (informational only)

| AC | Status | Where in this note |
|---|---|---|
| AC1 — Observed PDB sizes recorded | Done | §2 (dev build ~3–5 GB total PDBs + test build ~15–30 GB total PDBs, worktree-relative paths; cross-referenced to PROMPT 815 25 GB / ~200 GB datapoints). |
| AC2 — Profile knobs recommended | Done | §3 (Knob 1 `split-debuginfo`, Knob 2 `strip = "debuginfo"`, Knob 3 `debug = "line-tables-only"` all documented with semantic effects). |
| AC3 — Trade-offs articulated | Done | §3 (per-knob: disk size, CI wall-clock, debuggability, Windows Cargo-correctness; comparative summary table at end of §3). |
| AC4 — Single follow-on named | Done | §4 (`S14-DEVOPS-CARGO-PROFILE-TEST-STRIP-001`; one-sentence rationale). |
| AC5 — No code/config change lands | Done | §5 (only file modified is this new note). |
| AC6 — Sprint 13 disposition preserved | Done | §5 (sprint-status.yaml / sprint-13.md / stage.txt / PROMPT 761 gate-check artifact all unmodified). |
| AC7 — Cross-link to disk-usage investigation | Done | §1 (cross-links `docs/architecture/cargo-workspace-disk-usage.md`, the sibling Sprint 13 story 001 investigation note). |
| AC8 — No-claim restatement embedded | Done | top of note (verbatim Status / No-Claim Banner from story file). |

The AC self-check above is informational and does **not** itself
verify the story; that verification belongs to `/story-readiness` and
`/story-done` in future prompts.
