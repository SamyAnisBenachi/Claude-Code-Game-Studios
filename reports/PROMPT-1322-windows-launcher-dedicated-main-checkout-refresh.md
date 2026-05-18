# PROMPT 1322 -- Windows Launcher Dedicated-Main-Checkout Refresh

Status: READY_FOR_MAIN_LAND

Refresh of the PROMPT 1316 launcher integration rebased onto the current
`origin/main` (PROMPT 1320 sprint-18 qa-plan main-land). No `main` push
performed -- PROMPT 1321 may still advance `main` with story-authoring
docs, so the orchestrator owns the land decision.

## Source / refs

| Field | Value |
|---|---|
| Source branch | `origin/integrate/windows-launcher-dedicated-main-checkout-1316` |
| Source tip SHA | `7e83b6a42661406363d6cd007c0182937f3d42f3` (matches expected `7e83b6a`) |
| Source commits | `a17fd35 PROMPT-1309 launcher dedicated play/build checkout`, `7e83b6a report(prompt-1316): windows launcher dedicated main checkout integration` |
| Pre-refresh main | `origin/main@6e885b7a732a79ef29fd618908374d78402dc398` (PROMPT 1320 s18 qa-plan integration refresh main-land) |
| Merge base (1316 source vs current main) | `3be6c25064993f29a6b3eaf524f1999260405fac` (PROMPT 1314 Sprint 18 activation main-land) |
| Refresh branch | `integrate/windows-launcher-dedicated-main-checkout-1322` |
| Refresh HEAD | `e2713e7742962cb2a5c126ccc75d795284e93c98` |
| Worktree path | `D:/_DEV/claude-code-game-studios-worktrees/prompt-1322-launcher-refresh` |

Worktree created fresh from `origin/main` per the prompt; the orchestrator
root checkout (on `mainland/s18-server-dead-state-hygiene-1315`, dirty with
a staged `production/qa/qa-plan-sprint-18.md`) was not used.

## Refresh method

`git cherry-pick a17fd35 7e83b6a` applied cleanly onto current `origin/main`
with no conflicts:

```
[integrate/windows-launcher-dedicated-main-checkout-1322 84b945d] PROMPT-1309 launcher dedicated play/build checkout
 5 files changed, 1106 insertions(+), 91 deletions(-)
 create mode 100644 reports/PROMPT-1309-windows-launcher-dedicated-main-checkout-repair.md
[integrate/windows-launcher-dedicated-main-checkout-1322 e2713e7] report(prompt-1316): windows launcher dedicated main checkout integration
 1 file changed, 128 insertions(+)
 create mode 100644 reports/PROMPT-1316-windows-launcher-dedicated-main-checkout-integration.md
```

Between the PROMPT 1316 merge base (`3be6c25`) and the current `origin/main`
(`6e885b7`), main only advanced with PROMPT 1318/1320 qa-plan work:

```
6e885b7 report(prompt-1320): s18 qa-plan integration refresh main-land
8eedaf6 qa-plan(s18): author Sprint 18 QA plan (PROMPT 1318)
```

Both commits touched files outside the launcher's owned surface
(`production/qa/qa-plan-sprint-18.md`, `reports/PROMPT-1318-*`,
`reports/PROMPT-1320-*`), so the cherry-pick produced no conflicts and no
additional resolution was needed. A second `git fetch origin` after the
cherry-pick confirmed `origin/main` had not advanced during the refresh
(still `6e885b7`); no re-rebase required.

## Changed files (vs current origin/main)

```
 docs/setup/dev-two-button-launcher.md                                       | 204 ++++++++-
 reports/PROMPT-1309-windows-launcher-dedicated-main-checkout-repair.md      | 229 ++++++++++
 reports/PROMPT-1316-windows-launcher-dedicated-main-checkout-integration.md | 128 ++++++
 tools/dev-launcher-app/src/main.rs                                          | 507 ++++++++++++++++++++-
 tools/dev-launcher/Start-TwoClients.ps1                                     |  50 +-
 tools/dev-launcher/Update-LatestMain.ps1                                    | 207 +++++++--
 6 files changed, 1234 insertions(+), 91 deletions(-)
```

All six paths are inside the prompt's allowed-files list (the PROMPT 1316
report file is grandfathered in via the source branch; the PROMPT 1322
report file added by this commit is the seventh allowed entry). No forbidden
surface touched: no `production/**`, no `client/server/shared/**`,
no `Cargo.toml`/`Cargo.lock` outside tool-local manifest changes already
present in source, no `docs/architecture/**`, no `design/**`, no `.claude/**`.

## Required-behavior verification (read-only inspection of integrated source)

Identical behavior surface as PROMPT 1316 -- the refresh is a pure rebase,
not a code change. Re-confirmed against the rebased source in the fresh
worktree:

- **Launcher no longer treats the orchestrator root as canonical play/build target.** `tools/dev-launcher-app/src/main.rs` keeps the distinct `PlayRootResolution` separate from `RepoRootResolution`; the play/build path is resolved by `locate_play_root()` independently of `locate_repo_root()`.
- **Launcher resolves or creates a dedicated stable main checkout for rebuild/play sessions.** Play root priority (`resolve_play_root_pure`): `CCGS_PLAY_REPO_ROOT` env -> `CCGS_CANONICAL_MAIN_ROOT` env alias -> documented default `D:\_DEV\ccgs-play-main` (`PLAY_REPO_DEFAULT`, source label `PlayRootSource::DedicatedDefault`). Both env surfaces and the default are documented in code (`main.rs:55-68`) and `docs/setup/dev-two-button-launcher.md`.
- **Rebuild Latest Main operates against a checkout on branch `main`, not a `work/*` branch.** `tools/dev-launcher/Update-LatestMain.ps1` accepts `-PlayRepoRoot`, creates the play root as a linked git worktree off the launcher root when missing (`git -C $LauncherRoot worktree add $PlayRoot main`, with fallback `git worktree add -B main $PlayRoot origin/main`), and pulls/rebuilds inside `$PlayRoot`. The launcher EXE captures the play-root status (`OnMain`, `OnOtherBranch(name)`, `DetachedOrUnknown`, `Missing`, `InvalidRepo(why)`) and surfaces it in diagnostics before any rebuild.
- **Existing sidecar / `CCGS_REPO_ROOT` behavior remains sane.**
    - `CCGS_REPO_ROOT` (script-source repo root) is still honored when set and valid (`resolve_repo_root_pure` env branch).
    - Sidecar pointing at a valid repo on branch `main` is still accepted without falling back (PROMPT 1290 behavior preserved; `resolve_repo_root_sidecar_on_main_is_accepted_without_canonical_fallback`).
    - Sidecar pointing at a worker worktree is rejected as before and falls through to the canonical-checkout fallback (`resolve_repo_root_sidecar_on_worker_branch_falls_to_canonical`).
    - Invalid sidecar falls through to EXE walk-up (`resolve_repo_root_falls_through_invalid_sidecar_to_exe_walkup`).
- **Start Two Clients uses the same resolved play root as rebuild.** Both jobs are spawned by `run_powershell_job`, which passes the launcher's resolved `play_root_path` as `-PlayRepoRoot` to whichever script runs (`main.rs:601-622`, `main.rs:779-780`). `Start-TwoClients.ps1:41,92-94` consumes the same `-PlayRepoRoot` and sets `$PlayRoot` accordingly.

## Verification commands

| Step | Command | Result |
|---|---|---|
| Fetch | `git fetch origin --prune` | OK |
| Source SHA | `git rev-parse origin/integrate/windows-launcher-dedicated-main-checkout-1316` | `7e83b6a42661406363d6cd007c0182937f3d42f3` |
| Main at launch | `git rev-parse origin/main` | `6e885b7a732a79ef29fd618908374d78402dc398` |
| Merge base | `git merge-base origin/main origin/integrate/windows-launcher-dedicated-main-checkout-1316` | `3be6c25064993f29a6b3eaf524f1999260405fac` |
| Worktree create | `git worktree add -b integrate/windows-launcher-dedicated-main-checkout-1322 D:/_DEV/.../prompt-1322-launcher-refresh origin/main` | OK, HEAD `6e885b7` |
| Cherry-pick | `git cherry-pick a17fd35 7e83b6a` | OK, no conflicts, new HEAD `e2713e7` |
| Whitespace check | `git diff --check origin/main..HEAD` | clean (exit 0) |
| Path scope | `git diff --name-only origin/main..HEAD` | 6 files, all inside allowed list |
| Workspace Cargo | `git diff --stat origin/main..HEAD -- Cargo.toml Cargo.lock` | empty (no change) |
| Tool-local Cargo | `git diff --stat origin/main..HEAD -- tools/dev-launcher-app/Cargo.toml` | empty (unchanged from main) |
| Re-fetch main | `git fetch origin && git rev-parse origin/main` | still `6e885b7`, no re-rebase needed |
| Launcher tests | `cargo test` in `tools/dev-launcher-app` under MSVC policy | `57 passed; 0 failed; 0 ignored` |

### Cargo policy applied for launcher-app tests

```
CARGO_TARGET_DIR=D:/_DEV/cargo-target/ccgs-msvc
CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0
CARGO_INCREMENTAL=0
RUSTFLAGS=-C debuginfo=0 -C link-arg=/DEBUG:NONE
```

Tests were scoped to the `dev-launcher-app` tool crate
(`cargo test --manifest-path tools/dev-launcher-app/Cargo.toml`); no
full-workspace cargo command was run, so the workspace `Cargo.lock` is
untouched.

### Test output excerpt

```
   Compiling dev-launcher-app v0.1.0 (D:\_DEV\claude-code-game-studios-worktrees\prompt-1322-launcher-refresh\tools\dev-launcher-app)
    Finished `test` profile [optimized] target(s) in 1.47s
     Running unittests src\main.rs (D:/_DEV/cargo-target/ccgs-msvc\debug\deps\ccgs_dev_launcher-0427248ed90834db.exe)

running 57 tests
...
test result: ok. 57 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

Same 57 tests as PROMPT 1316 -- including the play-root resolution and
status coverage (`resolve_play_root_prefers_env_over_legacy_and_default`,
`resolve_play_root_uses_documented_default_when_no_env`,
`resolve_play_root_status_other_branch_when_worker_checkout`,
`play_root_default_is_not_inside_worktree_directory`, etc.) and the
PROMPT 1290 sidecar-on-worker-branch coverage
(`resolve_repo_root_sidecar_on_worker_branch_falls_to_canonical`,
`resolve_repo_root_sidecar_on_main_is_accepted_without_canonical_fallback`).

## Push policy

Per prompt, only the integration branch is pushed; `main` is NOT advanced
from this worker (PROMPT 1321 may concurrently land story-authoring docs).
Push of `integrate/windows-launcher-dedicated-main-checkout-1322` to origin
is the final step of this prompt.

## Main-land readiness

`READY_FOR_MAIN_LAND` -- fast-forward eligible (refresh branch is current
`origin/main` plus the two cherry-picked PROMPT 1309 + PROMPT 1316 commits).
If `main` advances again before land (e.g. PROMPT 1321 story-authoring
land), the orchestrator should either re-fetch and confirm a clean
fast-forward, or spawn another refresh prompt; the launcher surface is
small and isolated from the surfaces that other in-flight prompts touch.

## Files touched (final list)

- `docs/setup/dev-two-button-launcher.md`
- `reports/PROMPT-1309-windows-launcher-dedicated-main-checkout-repair.md`
- `reports/PROMPT-1316-windows-launcher-dedicated-main-checkout-integration.md`
- `reports/PROMPT-1322-windows-launcher-dedicated-main-checkout-refresh.md` (this file)
- `tools/dev-launcher-app/src/main.rs`
- `tools/dev-launcher/Start-TwoClients.ps1`
- `tools/dev-launcher/Update-LatestMain.ps1`

## Final status line

```
1322: WINDOWS-LAUNCHER-DEDICATED-MAIN-CHECKOUT-REFRESH: READY_FOR_MAIN_LAND
```
