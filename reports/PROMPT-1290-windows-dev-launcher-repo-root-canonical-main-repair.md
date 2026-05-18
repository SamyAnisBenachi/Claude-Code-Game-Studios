# PROMPT 1290 — Windows dev launcher repo-root sidecar canonical main repair

Branch (worker only, no main push):
`work/windows-launcher-canonical-main-root-1290`
Worktree: `D:\_DEV\claude-code-game-studios-worktrees\windows-launcher-canonical-main-root-1290`
Source-of-truth at start of work: `origin/main@d73e25e`

## Root cause

`tools/dev-launcher/build-launcher-exe.ps1` previously wrote whatever
`Split-Path -Parent (Split-Path -Parent $ScriptDir)` returned into
`ccgs-dev-launcher.repo-root.txt`. When the EXE is compiled from a worker
worktree on a `work/...` branch (e.g.
`D:\_DEV\claude-code-game-studios-worktrees\windows-dev-launcher-visual-polish-1255`),
that worktree path was pinned into the sidecar. The launcher
(`tools/dev-launcher-app/src/main.rs::resolve_repo_root_pure`) accepted the
sidecar without checking the branch, so the EXE then drove
`Update-LatestMain.ps1` against the worker worktree, and that script
correctly refused with:

```text
Refusing to fast-forward: current branch is 'work/windows-dev-launcher-visual-polish-1255', not 'main'.
```

The `Start Two-Client Play Session` button suffered the same defect — both
buttons used the same stale worker-worktree root.

## Exact fix behavior

Two-sided fix so neither side can re-introduce the bug.

### Launcher (Rust — `tools/dev-launcher-app/src/main.rs`)

New resolution order in `resolve_repo_root_pure`:

1. `CCGS_REPO_ROOT` env var — accepted for any branch (escape hatch unchanged).
2. Sidecar — accepted **only if the resolved repo is on branch `main`**.
   `read_head_branch` opens `.git/HEAD`, following the `gitdir:` pointer when
   `.git` is a file (linked worktree). A worker-branch sidecar is recorded in
   the attempts list (`"sidecar ... pointed at <path> (branch 'work/...') -- not on 'main', unsuitable for Rebuild Latest Main"`) and skipped.
3. **Canonical-checkout fallback** (new) — first entry in
   `CANONICAL_REPO_CANDIDATES` that validates. Default list is
   `[D:\_DEV\Work\Claude-Code-Game-Studios]`; override via
   `CCGS_CANONICAL_REPO_ROOT` (replaces the default list, not appends).
4. EXE-dir walk-up (existing).
5. CWD walk-up (existing).
6. `Failed` with an actionable error message listing every attempt.

Diagnostics panel now also shows the resolved branch, so testers can see at a
glance which checkout the EXE is talking to.

Both buttons consume the same resolved root, so `Start Two-Client Play
Session` no longer fires against a stale worker worktree either.

### Build script (PowerShell — `tools/dev-launcher/build-launcher-exe.ps1`)

New canonical-root resolution before writing the sidecar:

1. `-CanonicalRepoRoot <path>` (new explicit param).
2. `$env:CCGS_CANONICAL_REPO_ROOT`.
3. Build checkout if `git rev-parse --abbrev-ref HEAD` is `main`.
4. `D:\_DEV\Work\Claude-Code-Game-Studios` if it exists and validates.
5. **Refuse** with `exit 2` and an actionable message, unless
   `-AllowWorkerWorktreeSidecar` is passed (the explicit dev-only escape).

Sidecar body now also embeds a `# Canonical resolution: <source>` comment so
the resolution path is forensically visible to anyone inspecting the file on
disk.

## Changed files

| File | Change |
|---|---|
| `tools/dev-launcher-app/src/main.rs` | `ResolutionSource::CanonicalFallback`; `read_head_branch` helper (handles regular `.git/HEAD` AND linked-worktree `gitdir:` pointer); `CANONICAL_REPO_CANDIDATES` / `MAIN_BRANCH` constants; `canonical_lookup` helper; sidecar must-be-main check in `resolve_repo_root_pure`; diagnostics panel surfaces resolved branch; updated error message; 10 new tests; updated 9 existing tests for the new signature. |
| `tools/dev-launcher/build-launcher-exe.ps1` | New params `-CanonicalRepoRoot` and `-AllowWorkerWorktreeSidecar`; `Test-IsValidCcgsRepo` helper; full canonical-root resolution block before sidecar write; refuses worker-worktree sidecar by default; embeds resolution source in sidecar comments. |
| `docs/setup/dev-two-button-launcher.md` | Updated repo-root resolution-order section (5 steps now); new "Why the sidecar must point at a canonical (on-main) checkout" subsection; new "Canonical-root resolution (sidecar contents)" subsection under Build; new "Validation (PROMPT 1290)" log entry listing all new tests. |

## Test results

`cargo fmt -p dev-launcher-app -- --check` → clean (no diff).

`cargo test -p dev-launcher-app` → **41 passed; 0 failed; 0 ignored**.

```text
running 41 tests
... 41 individual lines, all `... ok` ...
test result: ok. 41 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

`cargo build -p dev-launcher-app` → builds clean under the Windows/MSVC
Cargo resource policy (CARGO_TARGET_DIR=D:\_DEV\cargo-target\ccgs-msvc,
CARGO_PROFILE_DEV_DEBUG=0, CARGO_PROFILE_TEST_DEBUG=0, CARGO_INCREMENTAL=0,
RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'), 1.08 s.

`[System.Management.Automation.Language.Parser]::ParseFile(...)` against
the modified `build-launcher-exe.ps1` → zero parser errors.

Note: I did not actually execute `build-launcher-exe.ps1 -Help` end-to-end
because the host execution policy blocks the EXE-launching of `.ps1` files
in this session and the user denied the `-ExecutionPolicy Bypass` escape;
the AST parser already validates syntax and parameter binding. The user
will exercise it locally via the documented bat/ps1 entry points.

### New / updated tests directly covering the PROMPT 1290 spec

| Spec requirement | Test |
|---|---|
| Sidecar points to worker branch → canonical main fallback is selected | `resolve_repo_root_sidecar_on_worker_branch_falls_to_canonical` |
| `CCGS_REPO_ROOT` overrides sidecar | `resolve_repo_root_env_overrides_valid_sidecar_pointing_elsewhere`, `resolve_repo_root_prefers_env_when_valid` |
| Sidecar points to main → accepted | `resolve_repo_root_sidecar_on_main_is_accepted_without_canonical_fallback`, `resolve_repo_root_falls_through_invalid_env_to_sidecar_on_main` |
| Invalid / no canonical root → actionable error | `resolve_repo_root_invalid_canonical_yields_actionable_error` |
| BOM handling remains fixed | `parse_sidecar_content_skips_bom_prefixed_comment_header`, `parse_sidecar_content_strips_bom_directly_before_path`, `read_sidecar_root_handles_utf8_bom_with_comment_header` (all preserved unchanged from PROMPT 1173) |
| Branch reader correctness | `read_head_branch_returns_main_for_regular_checkout`, `read_head_branch_returns_worker_branch_name`, `read_head_branch_returns_none_for_detached_head`, `read_head_branch_follows_worktree_gitdir_pointer` |
| Detached HEAD / unknown branch → canonical fallback | `resolve_repo_root_canonical_fallback_records_branch_label_for_unknown_head` |
| Default canonical list invariant | `canonical_repo_candidates_has_at_least_one_entry` |

## Worker branch / commit / push status

Local worker branch (not pushed to main):
`work/windows-launcher-canonical-main-root-1290`

Worker commit: see `git log -1` on that branch (commit hash appended after
the commit step below).

Push status: see DONE relay; if push to `origin` blocked, this report's
status flips to `LOCAL_ONLY` and the work is still safely committed in the
local worker branch / worktree above. Main is never pushed.

## User workaround before main-land

Until this branch lands on main, a user hitting the original bug can choose
any of the following (no rebuild required):

1. Launch the EXE with `CCGS_REPO_ROOT` pointed at the canonical checkout:
   ```text
   $env:CCGS_REPO_ROOT='D:\_DEV\Work\Claude-Code-Game-Studios'
   D:\_DEV\cargo-target\ccgs-msvc\debug\ccgs-dev-launcher.exe
   ```
   (Highest precedence; works on the *unfixed* EXE on disk today.)

2. Manually edit `D:\_DEV\cargo-target\ccgs-msvc\debug\ccgs-dev-launcher.repo-root.txt`
   so the first non-blank, non-comment line is
   `D:\_DEV\Work\Claude-Code-Game-Studios` (i.e. point the sidecar at the
   canonical checkout). The unfixed parser accepts this without branch
   checking.

3. Rebuild the EXE from the canonical checkout
   (`D:\_DEV\Work\Claude-Code-Game-Studios`) so the sidecar already points
   there:
   ```text
   cd D:\_DEV\Work\Claude-Code-Game-Studios
   powershell -ExecutionPolicy Bypass -File tools\dev-launcher\build-launcher-exe.ps1
   ```

Once this branch lands on main and the user does a fresh rebuild, none of
these workarounds are needed: the build script will refuse to pin a worker
worktree by default and the launcher will fall back to canonical on its
own.

## Final status line

1290: WINDOWS-DEV-LAUNCHER-REPO-ROOT-CANONICAL-MAIN-REPAIR: PASS
