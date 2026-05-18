# PROMPT 1309 — Windows launcher dedicated-main checkout repair

| Field | Value |
|---|---|
| Branch | `work/windows-launcher-dedicated-main-checkout-1309` |
| Worktree | `D:\_DEV\claude-code-game-studios-worktrees\windows-launcher-dedicated-main-1309` |
| Status | COMPLETE |
| Scope | tools/dev-launcher-app/**, tools/dev-launcher/**, docs/setup/dev-two-button-launcher.md, this report |
| Date | 2026-05-19 |

## User-reported failure (recap)

After PROMPT 1304 the launcher EXE resolved repo root as
`D:\_DEV\Work\Claude-Code-Game-Studios (via canonical-checkout fallback)`.
The fallback itself worked correctly, but the rebuild script refused
because the orchestrator checkout was on a worker branch:

```text
Current branch: work/s18-server-dead-state-hygiene-story-authoring-1305
Refusing to fast-forward: current branch is
'work/s18-server-dead-state-hygiene-story-authoring-1305', not 'main'.
Re-run with -Force only if you really want to switch to main first.
```

The launcher must not depend on the orchestrator/root checkout being on
`main`. The root checkout can legitimately be dirty or on a worker /
integration branch at any time.

## Root cause

The two-button launcher conflated **two different roots**:

1. **Launcher repo root** — where `tools\dev-launcher\Update-LatestMain.ps1`
   and `tools\dev-launcher\Start-TwoClients.ps1` live. Resolved via the
   PROMPT 1290 canonical-checkout fallback chain.
2. **Rebuild target** — where the rebuild flow does `git fetch / git switch
   main / git merge --ff-only origin/main`.

Both were the same path. So whenever the canonical fallback resolved (1)
to the orchestrator root, that path *also* became the rebuild target — and
the rebuild refused because the orchestrator was on a worker branch.

## Fix

Introduced a structurally separate **play/build root** that is the only
checkout the rebuild flow is ever permitted to switch.

### Resolution priority (play/build root)

1. `-PlayRepoRoot <absolute-path>` script argument. The launcher EXE
   always passes this.
2. `$env:CCGS_PLAY_REPO_ROOT` — preferred environment override.
3. `$env:CCGS_CANONICAL_MAIN_ROOT` — alias retained for back-compat.
4. `D:\_DEV\ccgs-play-main` — documented dedicated default. **Distinct**
   from `D:\_DEV\Work\Claude-Code-Game-Studios` and from worker worktrees
   under `D:\_DEV\claude-code-game-studios-worktrees\`.

### Auto-creation

If the play/build path is missing, `Update-LatestMain.ps1` materialises it
as a linked git worktree off the launcher root, non-destructively:

```text
git -C <launcher-root> fetch origin
git -C <launcher-root> worktree add <play-root> main
# (or, if local `main` is missing locally:)
git -C <launcher-root> worktree add -B main <play-root> origin/main
```

The launcher/orchestrator checkout is **never** switched or reset by this
flow, regardless of `-Force`.

### Branch-safety policy (play/build root only)

| Tree state | Outcome |
|---|---|
| On `main`, clean | rebuild proceeds normally |
| On a non-main branch, clean | `git switch main` then rebuild |
| On a non-main branch, dirty | refuse (exit 2) unless `-Force` |
| Dirty on any branch | refuse (exit 2) unless `-Force` |
| `main` ahead of `origin/main` | refuse (exit 2) unless `-Force` (destructive reset) |

## Files changed

- `tools/dev-launcher-app/src/main.rs` — new `PlayRootSource`,
  `PlayRootStatus`, `PlayRootResolution` types; `locate_play_root` +
  pure `resolve_play_root_pure` resolver; `LauncherState` carries play
  root; `start_job` / `run_powershell_job` pass `-PlayRepoRoot <path>`
  to the spawned PowerShell; `diagnostics_text` shows both launcher and
  play roots with branch + source + status; `on_init` populates the
  play root into state.
- `tools/dev-launcher/Update-LatestMain.ps1` — `-PlayRepoRoot` parameter;
  resolves play root in the documented priority; auto-creates the
  worktree if missing; switches play root to `main` only when clean;
  never touches the launcher root; help banner updated; dry-run no
  longer crashes when the dedicated path does not exist.
- `tools/dev-launcher/Start-TwoClients.ps1` — `-PlayRepoRoot` parameter;
  same fallback chain; warns when falling back to the launcher root
  because no dedicated checkout is configured.
- `docs/setup/dev-two-button-launcher.md` — new "Dedicated play/build
  checkout" section; Button 1 / Button 2 flag tables updated; resolution
  order split into "Launcher root" and "Play/build root" subsections;
  PROMPT 1309 validation block added.
- `reports/PROMPT-1309-windows-launcher-dedicated-main-checkout-repair.md`
  — this file.

## Diagnostics surface (verified by tests)

The EXE diagnostics panel now shows:

```text
Launcher repo root: D:\_DEV\Work\Claude-Code-Game-Studios
Resolved via: canonical-checkout fallback
Launcher branch: work/s18-server-dead-state-hygiene-...
Rebuild script: D:\_DEV\Work\...\tools\dev-launcher\Update-LatestMain.ps1
Two-client script: D:\_DEV\Work\...\tools\dev-launcher\Start-TwoClients.ps1
Play/build root: D:\_DEV\ccgs-play-main
Play/build source: documented dedicated default
Play/build status: missing -- will be created as a worktree on first rebuild
Play/build branch: <none -- not yet created>
```

All six fields required by the prompt (launcher repo root / script source,
play/build checkout path, play/build branch, dedicated-vs-override
indicator) are present.

## Tests

`cargo test -p dev-launcher-app` under the Windows/MSVC Cargo resource
policy:

```text
CARGO_TARGET_DIR=D:\_DEV\cargo-target\ccgs-msvc
CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0
CARGO_INCREMENTAL=0
RUSTFLAGS=-C debuginfo=0 -C link-arg=/DEBUG:NONE
```

Result: **57 passed; 0 failed; 0 ignored**.

49 prior tests (sidecar / BOM / canonical-fallback / branch reading /
walk-up resolution / log truncation) remain green. 8 new tests cover:

- `play_root_default_constant_is_separate_from_canonical_root` — the
  dedicated path must not equal the orchestrator/canonical root.
- `play_root_default_is_not_inside_worktree_directory` — the dedicated
  path must not live under `claude-code-game-studios-worktrees`.
- `resolve_play_root_prefers_env_over_legacy_and_default`,
  `resolve_play_root_uses_legacy_env_when_primary_unset`,
  `resolve_play_root_uses_documented_default_when_no_env`,
  `resolve_play_root_treats_empty_or_whitespace_env_as_unset` —
  resolution priority.
- `resolve_play_root_status_missing_when_path_absent`,
  `resolve_play_root_status_on_main_when_validated_and_main`,
  `resolve_play_root_status_other_branch_when_worker_checkout`,
  `resolve_play_root_status_detached_when_branch_unknown`,
  `resolve_play_root_status_invalid_when_path_exists_but_not_repo` —
  status reporting for every path/branch combination.
- `play_root_source_human_strings_are_distinct`,
  `play_root_env_constant_names_match_documented_pair`,
  `play_root_status_human_labels_are_actionable` — diagnostic strings.
- `diagnostics_text_reports_play_root_status_distinctly_from_launcher` —
  reproduces the user-reported scenario (launcher on worker branch +
  play root missing) and asserts both are surfaced.
- `diagnostics_text_shows_play_branch_when_play_root_on_other_branch` —
  branch label propagates into diagnostics.

PowerShell scripts validated via
`[System.Management.Automation.Language.Parser]::ParseFile` — 0 parser
errors across `Update-LatestMain.ps1`, `Start-TwoClients.ps1`,
`build-launcher-exe.ps1`. `Update-LatestMain.ps1 -Help` and
`Start-TwoClients.ps1 -Help` print clean banners (env var names render
as `$env:CCGS_PLAY_REPO_ROOT` after escaping the here-string
interpolation).

Dry-run smoke test:

```text
powershell -File Update-LatestMain.ps1 -DryRun \
    -PlayRepoRoot D:\tmp\nonexistent-play-root-1309
```

Resolves play root, prints what worktree-add it *would* run, exits 0
without invoking any git/cargo command and without crashing on the
missing `Set-Location` target.

## What is intentionally not done

- Did not invoke `Update-LatestMain.ps1` without `-DryRun`. Creating
  `D:\_DEV\ccgs-play-main` as a live worktree is a state-changing
  operation the user/operator should run when they're ready.
- Did not modify `build-launcher-exe.ps1`. The sidecar contract is
  unchanged — the play root is resolved at launcher *runtime* from env
  + default, not baked into the sidecar.
- Did not rebuild the EXE in this branch. Existing
  `D:\_DEV\cargo-target\ccgs-msvc\debug\ccgs-dev-launcher.exe` will not
  contain the new code path until the operator runs
  `powershell -ExecutionPolicy Bypass -File tools\dev-launcher\build-launcher-exe.ps1`
  from a canonical checkout once this branch lands on main.
- Did not edit production trackers, session state, sprint state, QA
  artifacts, gate checks, or non-launcher source. The full-workspace
  Cargo build was deliberately avoided per the prompt.

## Operator rebuild command (post-merge)

From a canonical checkout on `main`:

```text
$env:CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'
$env:CARGO_PROFILE_DEV_DEBUG='0'
$env:CARGO_PROFILE_TEST_DEBUG='0'
$env:CARGO_INCREMENTAL='0'
$env:RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'
powershell -ExecutionPolicy Bypass `
    -File tools\dev-launcher\build-launcher-exe.ps1
```

After that, launching
`D:\_DEV\cargo-target\ccgs-msvc\debug\ccgs-dev-launcher.exe` will show
the new two-root diagnostics and pass `-PlayRepoRoot` on both buttons.
The first `Rebuild Latest Main` click will create
`D:\_DEV\ccgs-play-main` as a linked worktree on `main` and proceed
with the fast-forward + cargo build there — without touching the
orchestrator checkout.

---

1309: WINDOWS-LAUNCHER-DEDICATED-MAIN-CHECKOUT-REPAIR: COMPLETE
