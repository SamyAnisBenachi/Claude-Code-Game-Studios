# Dev Environment Setup

> Story: `S11-OPS-GH-CLI-001` (Sprint 13 Nice to Have; doc-only)
> Source: `production/epics/devops/story-004-gh-cli-setup.md`
> Authored: 2026-05-14 (PROMPT 873)
> Path option chosen: **(a)** -- new file at `docs/setup/dev-environment.md`.
> `docs/setup/` already exists on `origin/main` (created by Story 017 /
> PROMPT 858 for `two-client-runtime-harness.md`), so only the file is
> new under this story; the directory is not.

This document lists the required developer tooling for working on the
Claude Code Game Studios repo. It is **onboarding documentation only**.
Nothing in this file installs or configures tooling automatically.

## Required tooling

### GitHub CLI (`gh`)

The GitHub CLI (`gh`) is **required** for orchestrator and producer
workflows in this repo. Worker and integration prompts use `gh` for:

- Pull-request creation, review-comment fetching, and merge
  inspection without leaving the terminal.
- Issue triage and labelling when bug rows are surfaced from the
  CLI rather than the GitHub web UI.
- Token-scoped API access (`gh api ...`) for read-only checks
  against branch protection, workflow runs, and release artifacts.

During Sprint 11 Wave 12 the GitHub CLI was logged as absent from
the dev machine 3+ times during orchestrator workflows, forcing
worker fallback to the GitHub web UI. This note exists to prevent
that recurrence by naming `gh` as required onboarding tooling.

#### Install commands

Pick the command for your platform. Run it once per machine; the CLI
self-updates from there.

**Windows (primary supported dev platform)** -- via `winget`:

```pwsh
winget install --id GitHub.cli
```

**macOS** -- via Homebrew:

```bash
brew install gh
```

**Linux (Debian / Ubuntu)** -- via the official apt repo (see
<https://cli.github.com/> for the current key + repo line; package
managers on other distros also ship `gh`):

```bash
sudo apt update
sudo apt install gh
```

#### One-time authentication (optional but recommended)

After install, authenticate once per machine:

```bash
gh auth login
```

Choose **GitHub.com**, then **HTTPS**, then **Login with a web
browser**. The default token scope (`repo`, `read:org`, `gist`,
`workflow`) is sufficient for the orchestrator workflows above; no
extra scopes are required by this story.

Verify with:

```bash
gh auth status
gh --version
```

## Out of scope for this story

This story is **documentation only**. The following are explicitly **not**
performed by the story commit:

- Installing `gh` on any machine.
- Adding `gh` to any CI workflow or build script
  (`.github/workflows/**`, `*.sh`, `*.ps1`).
- Recommending or adding any other tooling beyond `gh`.
- Activating or closing any Sprint 13 row.

See also: `production/epics/devops/story-005-win-appcompat-note.md`
(Sprint 13 Nice to Have `S13-OPS-WIN-APPCOMPAT-NOTE-001`) for the
sibling Windows App Compatibility note. That note is authored
separately and is not modified by this story.

---

## Status / No-Claim Banner (verbatim restatement, per AC5)

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

## Implementation note for PROMPT 873

The implementing prompt (PROMPT 873, `/dev-story` doc-only) chose
option **(a)** from the story's Prevention target: create
`docs/setup/dev-environment.md` as the canonical onboarding doc rather
than amending an existing sibling. Rationale:

- `docs/setup/` already exists on `origin/main@3cf5e41` (created by
  Story 017 / PROMPT 858 for `two-client-runtime-harness.md`), so
  option (a) only adds a new file under the established convention.
- The sibling story 005 (`S13-OPS-WIN-APPCOMPAT-NOTE-001`) is also
  expected to land in `docs/setup/dev-environment.md` per the story's
  Dependency Notes. Establishing the file now gives that future story
  an unambiguous target and avoids same-file conflict risk between
  the two paragraph-scale edits.
- No canonical sibling onboarding doc (e.g. `CONTRIBUTING.md`,
  `docs/onboarding.md`) currently exists on `origin/main`, so option
  (b) would require choosing a less-canonical host such as
  `docs/WORKFLOW-GUIDE.md` or `docs/octogent-integration.md`; both
  are not onboarding docs by purpose and would muddy their scope.

The single doc-only commit under this story touches **only** this
file. No code, CI, build script, or sprint-tracker artifact is
modified.

---

## Windows AppCompat heuristic for Cargo test binaries

> Story: `S13-OPS-WIN-APPCOMPAT-NOTE-001` (Sprint 13 Nice to Have; doc-only)
> Source: `production/epics/devops/story-005-win-appcompat-note.md`
> Authored: 2026-05-14 (PROMPT 886)
> Path option chosen: **(a)** -- appended to the existing
> `docs/setup/dev-environment.md` established by Story 004 / PROMPT 873.
> Workaround option chosen: **(a)** -- binary-rename / `[[test]] name`
> workaround. Option (b) embedded `asInvoker` manifest is deferred to a
> separate, NOT-AUTHORED follow-on story (see "Optional follow-on" below)
> and is **not implemented** by this story.

### What the heuristic is

Windows ships an Application Compatibility (**AppCompat**) installer-
detection heuristic that auto-classifies an executable as a potential
installer based on its **filename**. When the OS believes an executable
is an installer, it requires elevation and pops a User Account Control
(**UAC**) prompt before the binary can launch. The classifier triggers
on any of these case-insensitive substrings in the executable filename:

- `update`
- `install`
- `setup`
- `patch`
- `uninst`

The heuristic looks at the **filename only** -- not the executable
content, signature, or manifest. A Cargo test target named, e.g.,
`spawn_range_live_update_contract` produces a test binary named
`spawn_range_live_update_contract-<hash>.exe` whose filename contains
the substring `update`, and Windows therefore classifies the test
binary as an installer and prompts for elevation. This is a Windows
OS behavior, not a Cargo or Rust issue.

### Evidence trace

The heuristic was observed during Sprint 12 smoke / Team-QA / close-out
invocations:

- PROMPT 815 / 816 / 817 smoke runs logged AppCompat warnings against
  `spawn_range_live_update_contract-*.exe` test binaries.
- The warning is informational and does not prevent smoke completion
  once the prompt is dismissed (or the binary is renamed before
  invocation).
- Tracked as **TQ-S12-C7** (informational) in
  `production/qa/team-qa-sprint-12-2026-05-14.md` (see the
  `TQ-S12-C7 -- Windows AppCompat smoke warning is informational ...`
  line). TQ-S12-C7 is **NOT closed** by this story.

### Workaround used during Sprint 12 (option (a))

PROMPT 815 unblocked the Sprint 12 smoke by **renaming the affected
test binary at invocation time** so its filename did not contain the
substring `update`. The general form of the workaround for any test
target whose name contains a triggering substring is one of:

1. **Rename the Cargo test target** so its name avoids the triggering
   substring (out of scope for this story; would require a Cargo
   target change). Example -- a test named
   `spawn_range_live_update_contract` could be renamed to
   `spawn_range_live_refresh_contract` to avoid `update`, **or**
2. **Override the test binary name** in `Cargo.toml` via a
   `[[test]]` entry whose `name = "..."` avoids the triggering
   substring (also out of scope for this story; would require a
   Cargo manifest change):

   ```toml
   # NOT applied by S13-OPS-WIN-APPCOMPAT-NOTE-001 -- shown for
   # future reference only. A separate story is required to land
   # any actual Cargo target rename.
   [[test]]
   name = "spawn_range_live_refresh_contract"
   path = "tests/spawn_range_live_update_contract.rs"
   ```

3. **Rename the produced .exe at invocation time** (the in-place
   Sprint 12 workaround). On Windows PowerShell this looks like:

   ```pwsh
   # Locate the offending test binary under the cargo target dir
   # (the exact directory depends on $env:CARGO_TARGET_DIR; the
   # workspace-binding policy points it at D:\_DEV\cargo-target\ccgs-msvc).
   $bin = Get-ChildItem -Path "$env:CARGO_TARGET_DIR\debug\deps" `
                        -Filter "spawn_range_live_update_contract-*.exe" `
                        | Select-Object -First 1
   if ($bin) {
       $renamed = $bin.FullName -replace 'update', 'refresh'
       Copy-Item $bin.FullName $renamed
       & $renamed
   }
   ```

   This avoids the AppCompat heuristic for the renamed copy; the
   original Cargo-emitted binary is left untouched (Cargo will
   re-emit it on the next build with the offending name).

This story documents these workarounds; it does **not** apply any
of them. Renaming the actual Cargo test target or adding a
`[[test]]` entry to `Cargo.toml` is **out of scope** here and is
explicitly forbidden by the story's Control Manifest Rules.

### Optional follow-on -- option (b), NOT IMPLEMENTED here

A more permanent fix would be to embed a small Windows application
manifest declaring `requestedExecutionLevel level="asInvoker"`
into every Cargo-emitted test binary. With an explicit `asInvoker`
manifest the AppCompat installer heuristic is suppressed, and the
test binary launches without a UAC prompt regardless of filename.

This would require a workspace-wide `build.rs` (or equivalent
build-script) change to inject the manifest at link time, and is
**out of scope for this story**. If TQ-S12-C7 ever needs to be
closed (e.g., the warning persists across multiple hosts and is
judged disruptive), a separate Sprint 13+ Nice to Have story with
the candidate slug **`S13-OPS-WIN-APPCOMPAT-MANIFEST-001`**
should be authored to scope and implement the embedded-manifest
fix. **No such story is authored or activated by
S13-OPS-WIN-APPCOMPAT-NOTE-001.**

### Cross-reference

- **TQ-S12-C7** (informational): see
  `production/qa/team-qa-sprint-12-2026-05-14.md` line documenting
  the AppCompat smoke warning. **TQ-S12-C7 is NOT closed by this
  story** and remains preserved as informational; closure (if ever)
  happens via a separate `/team-qa` or close-out prompt.
- The PROMPT 761 Polish->Release gate-check FAIL evidence is
  preserved and is **NOT** affected by this story.

### Out of scope for this story

This story is **documentation only**. The following are explicitly
**not** performed by the story commit:

- Renaming any Cargo test target (e.g.,
  `spawn_range_live_update_contract`).
- Adding any `[[test]]` `name = "..."` entry to `Cargo.toml`.
- Adding any `build.rs` or workspace build-script change.
- Embedding any Windows application manifest in any binary.
- Closing or advancing TQ-S12-C7.
- Activating or closing any Sprint 13 row.
- Modifying any file under `client/`, `server/`, `shared/`, `tests/`,
  `Cargo.toml`, `Cargo.lock`, `.cargo/`, `.github/`, or any
  `production/sprint-status.yaml` / `production/sprints/**` /
  `production/stage.txt` / `production/session-state/**` artifact.

### S15-OPS-APPCOMPAT-MANIFEST-001 follow-up (Sprint 16; PROMPT 1068)

> Story: `S15-OPS-APPCOMPAT-MANIFEST-001` (Sprint 16 Nice to Have;
> implementation follow-on to Story 005's doc-only note).
> Source: `production/epics/devops/story-006-appcompat-manifest.md`
> Implemented: 2026-05-17 (PROMPT 1068).
> Mechanism chosen: **(d)** -- Cargo `[[test]] name` rename of the
> `spawn_range_live_update_contract` target to
> `spawn_range_live_refresh_contract` in `shared/Cargo.toml`. The
> source file `tests/unit/protocol/spawn_range_live_update_contract_test.rs`
> is **NOT renamed** under this story; the Cargo `[[test]]` `path`
> attribute still pointers it verbatim. Mechanism (d) was selected
> over the manifest-embed alternatives (a) / (b) / (c) because it
> requires zero new build dependency, zero `build.rs`, zero new
> resource file, and is cross-platform clean -- it changes only
> one Cargo manifest line on every host class regardless of OS,
> whereas the embed-manifest mechanisms require a Windows-only
> `build.rs`, a new build-dependency, and a Cargo resource compiler.
> Evidence: `production/qa/evidence/sprint-16-appcompat-manifest-evidence.md`.

**Primary path now**: the Cargo-emitted test binary filename is
`spawn_range_live_refresh_contract-<hash>.exe`, which contains no
AppCompat installer-detection trigger substring (`update`,
`install`, `setup`, `patch`, `uninst`). Invoke the test directly:

```pwsh
$env:CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'
$env:CARGO_PROFILE_DEV_DEBUG='0'
$env:CARGO_PROFILE_TEST_DEBUG='0'
$env:CARGO_INCREMENTAL='0'
$env:RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'
cargo test -p shared --test spawn_range_live_refresh_contract
```

The per-run binary-rename workaround at the smoke level (see
"Workaround used during Sprint 12 (option (a))" above) is **no
longer required** by the primary path after S15-OPS-APPCOMPAT-MANIFEST-001
lands; future smoke invocations call the renamed Cargo target
directly without any `cp ... srluc_appcompat_renamed.exe` step.

**Fallback retained (documented, NOT deleted)**: the per-run
binary-rename workaround documented above (option 3 under
"Workaround used during Sprint 12") remains valid as a
documented fallback for:

- non-MSVC Windows builds (where Cargo's emitted test-binary
  naming differs);
- hosts where a future re-introduction of a trigger substring
  (`update`, `install`, `setup`, `patch`, `uninst`) in any new
  Cargo test target name occurs and the rename has not yet been
  applied to that target;
- hosts where the AppCompat heuristic intercepts a binary whose
  name only incidentally contains a trigger substring (e.g.,
  third-party test binaries surfaced under workspace cargo runs);
- forensic / archeological reproduction of the Sprint 12 / 13 /
  14 smoke / Team-QA evidence (where the rename workaround was
  the historical mechanism).

**What this follow-up does NOT do**:

- It does **not** delete or alter the existing Story 005 text
  above. Story 005's AC1-AC7 evidence remains valid for the
  historical record.
- It does **not** close TQ-S12-C7 (Sprint 12 Team-QA AppCompat
  informational condition); closure is a separate `/team-qa`
  decision outside Sprint 16 scope.
- It does **not** retry the PROMPT 761 Polish->Release
  gate-check.
- It does **not** modify any code under `client/`, `server/`,
  `shared/src/`, or the test source file itself.
- It does **not** claim public release readiness, release-candidate
  readiness, full game completion, broad / Standard-tier
  accessibility completion (`QA-COND-0005`), playtest /
  fun-hypothesis validation (`QA-COND-0006`), two-client
  GAME_OVER closure (`S8-QA-001-W1`), or `PAW-TD-*-a`
  resolution.

---

## Status / No-Claim Banner (verbatim restatement for Story 005, per AC7)

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
