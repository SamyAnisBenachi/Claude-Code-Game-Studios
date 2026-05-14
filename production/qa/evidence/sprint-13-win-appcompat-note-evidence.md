# Sprint 13 -- S13-OPS-WIN-APPCOMPAT-NOTE-001 Evidence

> Story: `S13-OPS-WIN-APPCOMPAT-NOTE-001`
> Source: `production/epics/devops/story-005-win-appcompat-note.md`
> Implementing prompt: PROMPT 886 (`/dev-story` doc-only worker)
> Source-of-truth at start: `origin/main@75b6602` (PROMPT 885
> `/story-done` for `S11-SERVER-R2-PLACEMENT-CRASH-AUDIT-001`).
> Worktree: `D:/_DEV/claude-code-game-studios-worktrees/s13-win-appcompat-note`
> Branch: `work/s13-win-appcompat-note`

---

## Choices recorded

### Doc-path option (story Prevention target)

**Option (a)** chosen -- the AppCompat section is appended to the
existing `docs/setup/dev-environment.md` that Story 004 / PROMPT 873
created on `origin/main`.

Rationale:

- `docs/setup/dev-environment.md` exists on `origin/main@75b6602`
  (created by Story 004 / PROMPT 873 for `S11-OPS-GH-CLI-001`). The
  Story 005 Dependency Notes explicitly anticipate sharing this doc
  with Story 004; the file is therefore the canonical onboarding
  target.
- Option (b) (amending a less-canonical sibling like
  `docs/WORKFLOW-GUIDE.md` or `docs/octogent-integration.md`) would
  muddy the scope of those docs and split AppCompat onboarding info
  across multiple files.
- The two paragraph-scale edits (Story 004 GitHub CLI + Story 005
  AppCompat) cleanly compose under separate H2 sections without
  same-file conflict.

### Workaround option (AC2)

**Option (a)** chosen -- the binary-rename / `[[test]] name` workaround
is documented. Option (b) embedded `asInvoker` manifest is **NOT
implemented** here; it is deferred to a separate, NOT-AUTHORED
candidate follow-on story slug **`S13-OPS-WIN-APPCOMPAT-MANIFEST-001`**
named in the dev-environment doc.

Rationale:

- The prompt brief explicitly asks for the workaround "used during
  Sprint 12: rename or avoid 'update' in generated test binary names
  when possible". That is option (a).
- Option (b) is workspace-cargo-affecting (a `build.rs` or
  equivalent that injects a Windows application manifest at link
  time), which is **forbidden** by the story's Control Manifest
  Rules and Out of Scope list for this story.
- Option (b)'s named candidate follow-on slug is recorded in the
  doc itself (see "Optional follow-on -- option (b), NOT
  IMPLEMENTED here" section in
  `docs/setup/dev-environment.md`).

---

## Acceptance-criteria verification

All AC verification is **read-only**; this story does not invoke
`/story-done` or modify the story file itself.

- **AC1 -- Dev-environment doc updated**: PASS. A new H2 section
  `## Windows AppCompat heuristic for Cargo test binaries` is
  appended to `docs/setup/dev-environment.md`. The section names
  the heuristic, lists the triggering substrings (`update`,
  `install`, `setup`, `patch`, `uninst`), explains that the
  classifier looks at filenames only, and records the chosen path
  option (a). This evidence file records the choice and rationale.

- **AC2 -- Single workaround chosen**: PASS. Exactly one workaround
  option (option (a) -- binary rename / `[[test]] name`) is
  documented with a PowerShell example and Cargo `[[test]]` example.
  Option (b) embedded manifest is explicitly marked **NOT
  IMPLEMENTED** and is documented only as a candidate follow-on
  story slug (`S13-OPS-WIN-APPCOMPAT-MANIFEST-001`), with the
  trade-off (workspace-wide `build.rs` change) named.

- **AC3 -- Cross-link to TQ-S12-C7**: PASS. The new section's
  "Cross-reference" subsection links to
  `production/qa/team-qa-sprint-12-2026-05-14.md` and references the
  TQ-S12-C7 line documenting the AppCompat smoke warning. The
  "Evidence trace" subsection above it also names TQ-S12-C7 directly.

- **AC4 -- No production-source change lands**: PASS. The PROMPT 886
  commit touches only:
  - `docs/setup/dev-environment.md` (existing file; append-only)
  - `production/qa/evidence/sprint-13-win-appcompat-note-evidence.md`
    (NEW evidence file; not a sprint-tracker artifact)

  No file under `client/`, `server/`, `shared/`, `tests/`, no
  `Cargo.toml` / `Cargo.lock` / `.cargo/**`, no `.github/**`, no
  `*.sh` / `*.ps1` is modified. No Cargo test target is renamed;
  no build-script change lands.

- **AC5 -- Sprint 13 disposition preserved**: PASS. The PROMPT 886
  commit does not modify
  `production/sprint-status.yaml`,
  `production/sprints/sprint-13.md`,
  `production/stage.txt`, or the PROMPT 761 gate-check artifact.
  Verification command:
  `git diff origin/main...HEAD --stat -- 'production/sprint-status.yaml' 'production/sprints/**' 'production/stage.txt' 'production/session-state/**'`
  expected output: empty.

- **AC6 -- TQ-S12-C7 NOT closed by this story**: PASS. The new
  section explicitly states "TQ-S12-C7 is **NOT closed** by this
  story" in two places (Evidence trace + Cross-reference) and
  preserves the informational disposition. No edit to
  `production/qa/team-qa-sprint-12-2026-05-14.md` lands under this
  story.

- **AC7 -- No-claim restatement embedded**: PASS. The verbatim
  Status / No-Claim Banner from the story is appended below the
  AppCompat section under
  `## Status / No-Claim Banner (verbatim restatement for Story 005, per AC7)`.
  The "NOT a Sprint 12 close-out blocker" line from the story is
  preserved verbatim at the bottom of that section.

---

## File-scope verification (planned diffs at commit time)

`git diff origin/main...HEAD --stat`:
```
docs/setup/dev-environment.md                              | <N>+
production/qa/evidence/sprint-13-win-appcompat-note-evidence.md | <N>+
```

`git diff origin/main...HEAD --stat -- 'client/**' 'server/**' 'shared/**' 'tests/**' 'Cargo.toml' 'Cargo.lock' '.cargo/**' '.github/**' '*.sh' '*.ps1'`:
expected output: empty (zero code/config/CI change).

`git diff origin/main...HEAD --stat -- 'production/sprint-status.yaml' 'production/sprints/**' 'production/stage.txt' 'production/session-state/**'`:
expected output: empty (Sprint 13 disposition preserved).

`git diff origin/main...HEAD --stat -- 'production/qa/team-qa-sprint-12-2026-05-14.md' 'production/qa/qa-plan-sprint-12.md'`:
expected output: empty (TQ-S12-C7 not closed).

---

## Cargo policy

**N/A.** PROMPT 886 is doc-only and invokes no `cargo` command. The
binding Windows/MSVC Cargo resource policy (`CARGO_TARGET_DIR=D:\_DEV\cargo-target\ccgs-msvc`
+ `CARGO_PROFILE_DEV_DEBUG=0` + `CARGO_PROFILE_TEST_DEBUG=0` +
`CARGO_INCREMENTAL=0` + `RUSTFLAGS=-C debuginfo=0 -C link-arg=/DEBUG:NONE`)
is therefore not applied by this prompt.

---

## No-claim restatement (PROMPT 886 doc-only)

PROMPT 886 does **NOT**:

- Activate Sprint 13 or close any Sprint 13 row.
- Modify `production/sprint-status.yaml`,
  `production/sprints/sprint-13.md`, or `production/stage.txt`.
- Modify any `production/session-state/*` file.
- Modify any QA-plan / smoke / Team-QA / gate-check / release-check
  artifact.
- Run `/story-readiness`, `/story-done`, `/smoke-check`, `/team-qa`,
  `/gate-check`, `/release-check`, or `/qa-plan` on this story.
- Modify any code under `client/`, `server/`, `shared/`, or `tests/`.
- Modify `Cargo.toml`, `Cargo.lock`, or any `.cargo/` config.
- Rename any Cargo test target, including
  `spawn_range_live_update_contract`.
- Add any build-script (`build.rs`) or embedded Windows manifest.
- Retry the PROMPT 761 Polish->Release gate-check.
- Close TQ-S12-C7 (preserved as informational).
- Merge `work/s13-win-appcompat-note` to `main` (worker branch
  push only).

PROMPT 886 does **not** claim: public release readiness,
release-candidate readiness, full game completion, broad /
Standard-tier accessibility completion (`QA-COND-0005`), playtest /
fun-hypothesis validation (`QA-COND-0006`), full playable-client
manual QA, two-client GAME_OVER closure (`S8-QA-001-W1`), or
final-art / asset-production completion.

Sprint 10 / Sprint 11 / Sprint 12 dispositions unchanged. PROMPT 761
Polish->Release gate-check FAIL evidence preserved.

**PROMPT 886 lands two doc-only files. NO PRODUCTION-SOURCE CHANGE
LANDS.** The Story 005 doc edit is **NOT a Sprint 12 close-out
blocker** (already accepted-risk per TQ-S12-C7).
