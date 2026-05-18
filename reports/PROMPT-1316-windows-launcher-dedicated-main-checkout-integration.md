# PROMPT 1316 -- Windows Launcher Dedicated-Main-Checkout Integration

Status: READY_FOR_MAIN_LAND

## Source / refs

| Field | Value |
|---|---|
| Source branch | `origin/work/windows-launcher-dedicated-main-checkout-1309` |
| Source tip SHA | `3c35e9cd5945f9a1a8392f9b9a3565011554fa63` (expected `3c35e9c`, matches) |
| Source commit | `3c35e9c PROMPT-1309 launcher dedicated play/build checkout` |
| Pre-integration main | `origin/main@3be6c25064993f29a6b3eaf524f1999260405fac` (PROMPT 1314 Sprint 18 activation main-land) |
| Merge base (main vs work) | `bb1c5964a91e5ff01bfb64f116c6a8b58fbe5140` (PROMPT 1307) |
| Integration branch | `integrate/windows-launcher-dedicated-main-checkout-1316` |
| Integration HEAD | `a17fd3555775db11ff2e62bea655854104c26877` |
| Worktree path | `D:/_DEV/claude-code-game-studios-worktrees/windows-launcher-dedicated-main-checkout-integration-1316` |

Worktree was created fresh from `origin/main` per the prompt; the orchestrator
root checkout (on `integrate/s18-server-dead-state-hygiene-story-authoring-1313`)
was not used.

## Integration method

`git cherry-pick 3c35e9c` applied cleanly onto `origin/main` with no conflicts.
Pre-cherry-pick check `git log bb1c596..origin/main -- tools/dev-launcher/ tools/dev-launcher-app/ docs/setup/dev-two-button-launcher.md` returned empty, confirming `main` has not touched any of the source branch's owned files since the merge base.

## Changed files (vs origin/main)

```
 docs/setup/dev-two-button-launcher.md              | 204 ++++++++-
 reports/PROMPT-1309-windows-launcher-dedicated-main-checkout-repair.md | 229 ++++++++++
 tools/dev-launcher-app/src/main.rs                 | 507 ++++++++++++++++++++-
 tools/dev-launcher/Start-TwoClients.ps1            |  50 +-
 tools/dev-launcher/Update-LatestMain.ps1           | 207 +++++++--
 5 files changed, 1106 insertions(+), 91 deletions(-)
```

All five paths are inside the prompt's allowed-files list. No forbidden surface touched (no `production/**`, no `client/**`, no `server/**`, no `shared/**`, no workspace `Cargo.toml`/`Cargo.lock`, no `docs/architecture/**`, no `design/**`, no `.claude/**`).

## Required-behavior verification (read-only inspection of integrated source)

- **Launcher no longer treats the orchestrator root as canonical play/build target.** `tools/dev-launcher-app/src/main.rs` introduces a distinct `PlayRootResolution` separate from `RepoRootResolution`. The "play/build" path is resolved by `locate_play_root()` independently of `locate_repo_root()`, so the script-source root (orchestrator/canonical) and the play/build root are now two different paths by design.
- **Launcher resolves or creates a dedicated stable main checkout for rebuild/play sessions.** Play root priority (in `resolve_play_root_pure`):
    1. `CCGS_PLAY_REPO_ROOT` env var (preferred, source label `PlayRootSource::Env`).
    2. `CCGS_CANONICAL_MAIN_ROOT` env var alias (source label `PlayRootSource::LegacyEnv`).
    3. Documented default `D:\_DEV\ccgs-play-main` (`PLAY_REPO_DEFAULT`, source label `PlayRootSource::DedicatedDefault`).
  Default and env-override surfaces are both documented in code (`main.rs:55-68`) and in `docs/setup/dev-two-button-launcher.md`.
- **Rebuild Latest Main operates against a checkout on branch `main`, not a `work/*` branch.** `tools/dev-launcher/Update-LatestMain.ps1` accepts `-PlayRepoRoot`, creates the play root as a linked git worktree off the launcher root when missing (`git -C $LauncherRoot worktree add $PlayRoot main`, with fallback `git worktree add -B main $PlayRoot origin/main`), and pulls/rebuilds inside `$PlayRoot`. The launcher EXE captures the resolved play-root status (`OnMain`, `OnOtherBranch(name)`, `DetachedOrUnknown`, `Missing`, `InvalidRepo(why)`) and surfaces it in the diagnostics panel before any rebuild starts.
- **Existing sidecar / `CCGS_REPO_ROOT` behavior remains sane.**
    - `CCGS_REPO_ROOT` (script-source repo root) is still honored when set and valid (`resolve_repo_root_pure` env branch).
    - Sidecar pointing at a valid repo on branch `main` is still accepted without falling back (PROMPT 1290 behavior preserved; covered by `resolve_repo_root_sidecar_on_main_is_accepted_without_canonical_fallback`).
    - Sidecar pointing at a worker worktree is rejected as before and falls through to the canonical-checkout fallback (`resolve_repo_root_sidecar_on_worker_branch_falls_to_canonical`).
    - Invalid sidecar falls through to EXE walk-up (`resolve_repo_root_falls_through_invalid_sidecar_to_exe_walkup`), so an invalid sidecar does not silently disable all fallback paths.
- **Start Two Clients uses the same resolved play root as rebuild.** Both jobs are spawned by `run_powershell_job`, which passes the launcher's resolved `play_root_path` as `-PlayRepoRoot` to whichever script runs (`main.rs:601-622`, `main.rs:779-780`). `Start-TwoClients.ps1:41,92-94` consumes the same `-PlayRepoRoot` argument and sets `$PlayRoot` accordingly.

## Verification commands

| Step | Command | Result |
|---|---|---|
| Fetch | `git fetch origin --prune` | OK |
| Source SHA | `git rev-parse origin/work/windows-launcher-dedicated-main-checkout-1309` | `3c35e9cd5945f9a1a8392f9b9a3565011554fa63` |
| Main at launch | `git rev-parse origin/main` | `3be6c25064993f29a6b3eaf524f1999260405fac` |
| Merge base | `git merge-base origin/main origin/work/windows-launcher-dedicated-main-checkout-1309` | `bb1c5964a91e5ff01bfb64f116c6a8b58fbe5140` |
| Worktree create | `git worktree add -b integrate/windows-launcher-dedicated-main-checkout-1316 D:/_DEV/.../windows-launcher-dedicated-main-checkout-integration-1316 origin/main` | OK, HEAD `3be6c25` |
| Cherry-pick | `git cherry-pick 3c35e9cd5945f9a1a8392f9b9a3565011554fa63` | OK, no conflicts, new HEAD `a17fd3555775db11ff2e62bea655854104c26877` |
| Whitespace check | `git diff --check origin/main` | clean |
| Path scope | `git diff --name-only origin/main` | 5 files, all inside allowed list |
| Workspace Cargo | `git diff --stat origin/main -- Cargo.toml Cargo.lock` | empty (no change) |
| Launcher tests | `cargo test` in `tools/dev-launcher-app` under MSVC policy | `57 passed; 0 failed; 0 ignored` |

### Cargo policy applied for launcher-app tests

```
CARGO_TARGET_DIR=D:/_DEV/cargo-target/ccgs-msvc
CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0
CARGO_INCREMENTAL=0
RUSTFLAGS=-C debuginfo=0 -C link-arg=/DEBUG:NONE
```

Tests were scoped to the `dev-launcher-app` tool crate (`cd tools/dev-launcher-app && cargo test`); no full-workspace cargo command was run.

### Test output excerpt

```
test result: ok. 57 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

Notable coverage exercised by these tests for the dedicated-checkout behavior:

- `resolve_play_root_prefers_env_over_legacy_and_default`
- `resolve_play_root_uses_legacy_env_when_primary_unset`
- `resolve_play_root_uses_documented_default_when_no_env`
- `resolve_play_root_treats_empty_or_whitespace_env_as_unset`
- `resolve_play_root_status_missing_when_path_absent`
- `resolve_play_root_status_on_main_when_validated_and_main`
- `resolve_play_root_status_other_branch_when_worker_checkout`
- `resolve_play_root_status_detached_when_branch_unknown`
- `resolve_play_root_status_invalid_when_path_exists_but_not_repo`
- `play_root_default_constant_is_separate_from_canonical_root`
- `play_root_default_is_not_inside_worktree_directory`
- `play_root_env_constant_names_match_documented_pair`
- `diagnostics_text_reports_play_root_status_distinctly_from_launcher`
- `diagnostics_text_shows_play_branch_when_play_root_on_other_branch`
- (plus prior PROMPT 1290 sidecar-on-worker-branch tests, all still green)

## Push policy

Per prompt, preferred output is an integration branch pushed to origin with status `READY_FOR_MAIN_LAND`. No main push performed from this worker. Push of `integrate/windows-launcher-dedicated-main-checkout-1316` to origin is the final step.

## Main-land status

`READY_FOR_MAIN_LAND` -- fast-forward eligible (integration branch is `origin/main` + the single cherry-picked PROMPT 1309 commit). Land decision belongs to the orchestrator.

## Files touched (final list)

- `docs/setup/dev-two-button-launcher.md`
- `reports/PROMPT-1309-windows-launcher-dedicated-main-checkout-repair.md`
- `reports/PROMPT-1316-windows-launcher-dedicated-main-checkout-integration.md` (this file)
- `tools/dev-launcher-app/src/main.rs`
- `tools/dev-launcher/Start-TwoClients.ps1`
- `tools/dev-launcher/Update-LatestMain.ps1`

## Final status line

```
1316: WINDOWS-LAUNCHER-DEDICATED-MAIN-CHECKOUT-INTEGRATION: READY_FOR_MAIN_LAND
```
