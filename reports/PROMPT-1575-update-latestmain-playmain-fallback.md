# PROMPT 1575 -- Update-LatestMain.ps1 play-main worktree fallback

- Branch: `work/launcher-playmain-fallback-1575`
- Worktree: `D:\_DEV\Work\.worktrees\launcher-playmain-fallback-1575`
- Base: `origin/main@e8e4651b`
- Status: SHIPPED

## Problem

User-reported launcher rebuild from PROMPT 1571's shipped UI:

```
==== Create play/build worktree ====
Path D:\_DEV\ccgs-play-main does not exist -- creating as a linked git worktree.
Attempting: git -C D:\_DEV\Work\Claude-Code-Game-Studios worktree add D:\_DEV\ccgs-play-main main
[err] Preparing worktree (checking out 'main')
[err] fatal: 'main' is already used by worktree at 'D:/_DEV/Work/Claude-Code-Game-Studios'
git worktree add failed. Resolve the conflict (likely 'main' is checked out elsewhere) and retry.
==== FINISHED: Rebuild Latest Main (exit 1) ====
```

Root cause: the launcher root checkout is on `main`, so `git worktree add
<play-root> main` is refused by git. `Update-LatestMain.ps1` did not have a
fallback for this case -- it exited 1 and the user had no path to bootstrap
the dedicated play/build checkout.

## Change

`tools/dev-launcher/Update-LatestMain.ps1` (+65 / -21):

1. **Worktree-create fallback**: before invoking `worktree add`, parse
   `git worktree list --porcelain` and detect whether `branch refs/heads/main`
   is already checked out by another worktree. When yes (the user's case --
   launcher root holds `main`), use a dedicated local branch:

   ```
   git -C <launcher> worktree add -B play-main <play-root> origin/main
   ```

   The new branch `play-main` tracks `origin/main` and is the canonical
   branch for the dedicated play/build checkout when `main` is taken. When
   `main` is free, the script still prefers `worktree add ... main` exactly
   as before -- no behavior change for fresh setups.

2. **Branch pre-check made name-agnostic**: `$CanonicalPlayBranches = @('main', 'play-main')`. If the play root is on either of those, no
   switch is needed. If it's on something else and clean, attempt
   `git switch main`; on failure (typically because main is already checked
   out elsewhere), try `git switch play-main`; if that branch doesn't exist
   yet, create it from `origin/main` (`git switch -c play-main origin/main`).
   This handles re-runs against an existing play root that was created on
   `play-main` by a prior bootstrap.

3. **Fast-forward against current branch, not literal `main`**:
   `git rev-list --left-right --count HEAD...origin/main` instead of
   `main...origin/main`. The local `main` ref is shared across worktrees and
   would mis-compare against the launcher root's HEAD when the play root is
   on `play-main`. Comparing `HEAD` is the right semantic: "how does THIS
   worktree's branch compare to origin/main". The `git merge --ff-only
   origin/main` call is unchanged -- it FFs whichever branch is checked out.

4. **Docs**: top-of-file `What it does` block and `-Help` SAFETY section
   document the `play-main` fallback explicitly.

## UX behavior (now)

First click of `Rebuild Latest Main` from the launcher (the user's exact
flow):

- Old behavior: `worktree add ... main` fails -> exit 1 -> red FAIL badge.
- New behavior: detected `main` checked out in launcher root ->
  `worktree add -B play-main <play-root> origin/main` -> succeeds ->
  proceeds to FF check (already up-to-date) -> proceeds to cargo build
  -> exit 0 -> green SUCCESS badge.

Subsequent clicks (play root exists, on `play-main`):
- Branch check accepts `play-main` -> no `git switch` attempt.
- FF check compares `HEAD...origin/main` -> behaves identically to
  the previous `main...origin/main` semantics for a healthy mirror.

The launcher/orchestrator checkout is still NEVER touched.

## Validation

- `git diff --check`: clean.
- PowerShell AST parse: PARSE OK -- 1944 tokens, 0 errors.
- Dry-run end-to-end (`-DryRun -PlayRepoRoot D:/_DEV/ccgs-play-main` against a
  worktree whose play root does not exist):
  ```
  ==== Create play/build worktree ====
  Path D:/_DEV/ccgs-play-main does not exist -- creating as a linked git worktree.
  [dry-run] git -C ... fetch origin
  [dry-run] git -C ... worktree add D:/_DEV/ccgs-play-main main (or -B play-main origin/main if main is already checked out)
  ```
  Script now correctly advertises the fallback in `-DryRun` output.
- No Pester suite exists for `Update-LatestMain.ps1` (only `BuildProvenance.Tests.ps1` is present in `tools/dev-launcher`), so no automated suite to extend.
- No broad workspace Cargo run (per PROMPT scope: launcher-only change).

Allowlist review: only `tools/dev-launcher/Update-LatestMain.ps1` and this
report touched. No production/, no sprint state, no shared gameplay code,
no Cargo manifest edits, no other launcher files.

## How to test (user-facing)

After this branch lands on `main`:

1. Restart the dev launcher EXE (or close + reopen `ccgs-dev-launcher.exe`).
2. Click **Rebuild Latest Main**.
3. Watch the Script Output panel:
   - Should print `Local 'main' is already checked out in another worktree
     (likely the launcher root).`
   - Then `Attempting: git -C ... worktree add -B play-main D:\_DEV\ccgs-play-main origin/main`.
   - Then the rest of the rebuild (fetch, FF check, cargo build).
4. Status badge should land on green **SUCCESS** (exit 0).
5. Diagnostics block should show `Play/build status: exists, on branch
   'play-main'` on subsequent launcher restarts (after the first
   successful rebuild).

Before this branch lands on `main`, the user can validate the fix locally
by running the worker-branch script directly:

```
powershell -NoProfile -ExecutionPolicy Bypass `
  -File D:\_DEV\Work\.worktrees\launcher-playmain-fallback-1575\tools\dev-launcher\Update-LatestMain.ps1 `
  -PlayRepoRoot D:\_DEV\ccgs-play-main
```

(That call uses this worker worktree as the launcher root; same repo, so
the `main` collision still triggers, and the new fallback path runs.)

## Files changed

- `tools/dev-launcher/Update-LatestMain.ps1` (+65 / -21)
- `reports/PROMPT-1575-update-latestmain-playmain-fallback.md` (new)

## Blockers

None.

1575: UPDATE-LATESTMAIN-PLAYMAIN-FALLBACK: SHIPPED
