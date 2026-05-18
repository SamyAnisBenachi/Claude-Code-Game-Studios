# Dev Two-Button Launcher

> Story-scope: PROMPT 1155 -- One-Button (now Two-Button) Latest-Main
> Two-Client Test Launcher.
> PROMPT 1162 added the optional native Windows EXE wrapper under
> `tools/dev-launcher-app/`.
> PROMPT 1309 added the dedicated play/build checkout (`-PlayRepoRoot`,
> `CCGS_PLAY_REPO_ROOT`) so the rebuild flow no longer depends on the
> orchestrator/launcher checkout being on `main`.
> Path: `tools/dev-launcher/` (scripts), `update-latest-main.bat` +
> `start-two-clients.bat` (one-click wrappers at repo root),
> `tools/dev-launcher-app/` (EXE wrapper).
> Authored: 2026-05-18; last updated 2026-05-19.

Three equivalent entry points for the manual two-client friend-game
test loop:

1. The raw PowerShell scripts (always available, no build step).
2. The `.bat` wrappers at the repo root (one-click).
3. The native Windows EXE wrapper (`ccgs-dev-launcher.exe`) -- a small
   two-button GUI that invokes the same scripts and streams their output
   into a log window. Build it once with `build-launcher-exe.ps1`.

All three paths converge on the same scripts and produce the same
artifacts. None of them modify git remotes, production trackers, sprint
state, QA artifacts, or evidence runbooks.

## Where to click

| Button | EXE button label | Wrapper at repo root | Real script |
|--------|------------------|----------------------|-------------|
| **Update + Rebuild** | `Rebuild Latest Main` | `update-latest-main.bat` | `tools\dev-launcher\Update-LatestMain.ps1` |
| **Launch 2 Clients** | `Start Two-Client Play Session` | `start-two-clients.bat` | `tools\dev-launcher\Start-TwoClients.ps1` |

Both `.bat` files invoke `powershell -NoProfile -ExecutionPolicy Bypass -File`
under the hood and pass `%*` through, so any extra PowerShell flags below can
be appended to the `.bat` invocation directly, e.g.
`update-latest-main.bat -Release` or `start-two-clients.bat -Port 5050`.

The EXE wrapper (`ccgs-dev-launcher.exe`) calls those same scripts with no
extra flags and streams their stdout/stderr into a scrolling log window.
See [Button 3 -- the EXE wrapper](#button-3----ccgs-dev-launcherexe) below.

## Dedicated play/build checkout (PROMPT 1309)

Both buttons now operate against a **dedicated play/build checkout** that is
structurally separate from the orchestrator/launcher checkout. The
orchestrator checkout (the one that owns the launcher scripts) is allowed to
be dirty or on a worker/integration branch at any time -- the dedicated
checkout is the only one the rebuild flow is ever permitted to switch.

### Resolution priority

The launcher EXE and the PowerShell scripts pick the dedicated path in this
order (first non-empty wins):

1. `-PlayRepoRoot <absolute-path>` script argument (the EXE always passes this).
2. `$env:CCGS_PLAY_REPO_ROOT` (preferred environment override).
3. `$env:CCGS_CANONICAL_MAIN_ROOT` (alias retained for back-compat).
4. `D:\_DEV\ccgs-play-main` -- documented default. **Distinct** from
   `D:\_DEV\Work\Claude-Code-Game-Studios` and from worker worktrees under
   `D:\_DEV\claude-code-game-studios-worktrees\`.

### Auto-creation

If the resolved play/build path does not exist on disk, `Update-LatestMain.ps1`
materialises it as a linked git worktree off the launcher repo root:

```text
git -C <launcher-root> fetch origin
git -C <launcher-root> worktree add <play-root> main
# (or, if local `main` is missing, the script auto-falls-back to:)
git -C <launcher-root> worktree add -B main <play-root> origin/main
```

This is a non-destructive operation: it never modifies the launcher checkout's
branch, and it never deletes anything. The new worktree starts on `main`. The
rebuild then continues inside the new worktree.

### Branch-safety policy

Inside the dedicated play/build checkout only:

| Tree state | Outcome |
|---|---|
| On `main`, clean | rebuild proceeds normally |
| On a non-main branch, **clean** | `git switch main` then rebuild |
| On a non-main branch, **dirty** | refuse (exit 2) unless `-Force` |
| Dirty on any branch | refuse (exit 2) unless `-Force` |
| `main` ahead of `origin/main` | refuse (exit 2) unless `-Force` (destructive reset) |

`Update-LatestMain.ps1` **never** switches the launcher/orchestrator checkout,
regardless of `-Force`.

## Button 1 -- Update + Rebuild (`update-latest-main.bat`)

### What it does (in order)

1. Resolves two roots: the **launcher root** (where the script lives) and the
   **play/build root** (the dedicated checkout) per the priority above.
2. If the play/build path is missing, creates it as a linked git worktree off
   the launcher root, starting on `main`.
3. Inside the play/build root: if on a non-main branch, attempts
   `git switch main` (clean tree only); aborts on dirty tree unless `-Force`.
4. `git fetch origin` (inside the play/build root).
5. Fast-forwards local `main` to `origin/main` inside the play/build root.
   Aborts on non-FF unless `-Force` (which then performs a destructive
   `git reset --hard origin/main`).
6. Applies the documented Windows / MSVC Cargo resource policy:
   - `CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'`
   - `CARGO_PROFILE_DEV_DEBUG='0'`
   - `CARGO_PROFILE_TEST_DEBUG='0'`
   - `CARGO_INCREMENTAL='0'`
   - `RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'`
7. Checks D: free space. Under 40 GB plus `-AllowCacheClean` triggers
   cleanup of stale subdirectories **only under** the resolved
   `CARGO_TARGET_DIR` (a hard-coded match against
   `D:\_DEV\cargo-target\ccgs-msvc` -- never source, reports, production,
   `.git`, or evidence).
8. `cargo build -p server`.
9. `cargo build -p client --bin client`.
10. Prints the resolved binary paths and a "Next: ..." hint.

### What it does NOT do

- Does **not** start the server or any client.
- Does **not** push, force-push, or modify any remote branch.
- Does **not** modify `production/`, `qa/`, sprint trackers, story files,
  or evidence runbooks.
- Does **not** run tests.

### Flags

| Flag | Effect |
|------|--------|
| `-Force` | Allow dirty tree (no stash) and non-FF main reset. DESTRUCTIVE. Limited to the play/build root; the launcher/orchestrator checkout is never switched. |
| `-Release` | Build in release profile (default is debug). |
| `-AllowCacheClean` | Under-40-GB free space cleans stale Cargo target subdirs. |
| `-DryRun` | Print every step; run no git, cargo, or rm command. |
| `-PlayRepoRoot P` | Absolute path of the dedicated play/build checkout. Overrides the env / default. |
| `-Help` | Print usage and exit. |

### Logs / artifacts created

- None on disk beyond Cargo's own `$CARGO_TARGET_DIR\debug\` artifacts.

## Button 2 -- Launch 2 Clients (`start-two-clients.bat`)

### What it does (in order)

1. Resolves the repo root from the script location.
2. Applies the same Cargo resource policy as Button 1.
3. Chooses a server port. Default `5000`. If busy, auto-bumps up to 50
   ports unless `-StrictPort` is passed.
4. If `server.exe` or `client.exe` is missing under the resolved
   `CARGO_TARGET_DIR\debug\` (or `\release\` with `-Release`), runs
   `cargo build` to produce them. Otherwise skips the build.
5. Creates a timestamped evidence directory at
   `production/qa/evidence/dev-runs/<UTC-YYYY-MM-DD-HHMMSS>/`.
6. Starts `server.exe` with `SERVER_PORT` set, redirecting stdout/stderr
   to `server.log` (and a sibling `server.log.err`). Waits up to
   `-ServerWaitSeconds` (default 8) for the port to be bound.
7. Starts two `client.exe` processes, each with
   `SERVER_URL=ws://127.0.0.1:<port>`, redirecting to `client_a.log` /
   `client_b.log` (and `.err` siblings).
8. Writes `launch-summary.json` with timestamps, PIDs, port, log paths,
   binary paths, and profile.
9. Prints PIDs, log paths, and tail-with-Get-Content hints. Exits 0
   while the three processes keep running.

### What it does NOT do

- Does **not** fetch, pull, merge, push, or otherwise modify git.
- Does **not** run tests, story-done, smoke, or any QA workflow.
- Does **not** modify `production/sprints/`, `production/session-state/`,
  `production/qa/team-qa-*`, `docs/architecture/`, or any tracker.
- Does **not** auto-close the client processes -- they stay up for
  manual testing. Use the X button or `Stop-Process -Id <pid>`.

### Flags

| Flag | Effect |
|------|--------|
| `-Port N` | Server bind port (default 5000). Auto-bumps if busy. |
| `-StrictPort` | Fail instead of auto-bumping when the chosen port is busy. |
| `-Release` | Use release-profile binaries. |
| `-ServerWaitSeconds N` | Bind-wait budget (default 8). |
| `-DryRun` | Print every step; start no process. |
| `-PlayRepoRoot P` | Absolute path of the dedicated play/build checkout under which the build runs and the evidence dir is created. Falls back to `$env:CCGS_PLAY_REPO_ROOT`, then `$env:CCGS_CANONICAL_MAIN_ROOT`, then `D:\_DEV\ccgs-play-main`, then the launcher root if none exists. |
| `-Help` | Print usage and exit. |

### Logs / artifacts created (per run)

```
production/qa/evidence/dev-runs/<UTC-stamp>/
  server.log            -- server stdout
  server.log.err        -- server stderr
  client_a.log          -- client A stdout
  client_a.log.err      -- client A stderr
  client_b.log          -- client B stdout
  client_b.log.err      -- client B stderr
  launch-summary.json   -- machine-readable run record
```

`production/qa/evidence/dev-runs/` is the new directory introduced by this
story; it is intentionally separate from existing capture / runbook trees
under `production/qa/evidence/captures/` so dev-only one-button runs
cannot be confused with formal QA-lead-signed evidence bundles.

## Button 3 -- `ccgs-dev-launcher.exe`

A small native Windows GUI (no console window in release builds) that wraps
the same two PowerShell scripts. Built from the in-workspace Rust crate
`tools/dev-launcher-app/` using `native-windows-gui` (Win32 controls).

### What it looks like

- Title: `CCGS Dev Launcher`
- Two buttons, exact labels:
  - `Rebuild Latest Main`
  - `Start Two-Client Play Session`
- A status line above the buttons (Idle / RUNNING / DONE / ERROR with exit code).
- A `Logs / evidence:` line that fills in once `Start Two-Client Play Session`
  prints its `Evidence dir:` marker.
- A read-only log box below the buttons that streams stdout (and `[err]`-prefixed
  stderr) from the underlying script.

### Button-to-script mapping

| EXE button | What it spawns |
|------------|----------------|
| `Rebuild Latest Main` | `powershell.exe -NoProfile -ExecutionPolicy Bypass -File tools\dev-launcher\Update-LatestMain.ps1 -PlayRepoRoot <play-root>` |
| `Start Two-Client Play Session` | `powershell.exe -NoProfile -ExecutionPolicy Bypass -File tools\dev-launcher\Start-TwoClients.ps1 -PlayRepoRoot <play-root>` |

Both invocations use the **launcher repo root** as the working directory and
pass `-PlayRepoRoot` so the script operates inside the dedicated checkout
described above. The launcher root is where the scripts live; the play/build
root is where rebuild + start actually run.

### Repo-root resolution order

The EXE resolves **two** independent roots at startup:

- **Launcher repo root** -- where `tools\dev-launcher\Update-LatestMain.ps1`
  and `tools\dev-launcher\Start-TwoClients.ps1` live. This is what gets
  passed as the spawned PowerShell's working directory.
- **Play/build root** -- the dedicated checkout the rebuild + start actually
  operate inside. This is what gets passed via `-PlayRepoRoot <path>` to the
  spawned PowerShell.

Both are surfaced in the Diagnostics panel along with each one's branch and
the resolution source. Examples:

```text
Launcher repo root: D:\_DEV\Work\Claude-Code-Game-Studios
Resolved via: canonical-checkout fallback
Launcher branch: work/s18-server-dead-state-hygiene-...
Play/build root: D:\_DEV\ccgs-play-main
Play/build source: documented dedicated default
Play/build status: missing -- will be created as a worktree on first rebuild
```

#### Launcher root resolution

The EXE typically lives at
`D:\_DEV\cargo-target\ccgs-msvc\debug\ccgs-dev-launcher.exe` -- **outside** the
repo tree. Walking up from there never reaches the worktree. The launcher
therefore resolves the launcher repo root in the following order at startup,
stopping at the first source whose path passes the validation check
(`Cargo.toml`, `tools/dev-launcher/`, and `.git` all present):

1. `CCGS_REPO_ROOT` environment variable (any branch).
2. `ccgs-dev-launcher.repo-root.txt` sidecar file living **next to the EXE** --
   accepted **only when its repo is on branch `main`**. If the sidecar points
   at a valid repo on any other branch (typically `work/...` from a worker
   worktree where the build was run) it is treated as unsuitable for
   `Rebuild Latest Main` and the launcher continues to step 3 instead.
3. **Canonical-checkout fallback.** First entry in the canonical candidate
   list that validates as a real CCGS workspace. The default list is
   `[D:\_DEV\Work\Claude-Code-Game-Studios]`. Override by setting
   `CCGS_CANONICAL_REPO_ROOT=<absolute-path>` before launching the EXE -- the
   override replaces (not augments) the default list.
4. The EXE's own directory, walked upward (useful when the EXE was copied
   inside the worktree).
5. The current working directory, walked upward.

If none of these resolves to a valid launcher repo root the EXE **does not**
silently adopt the EXE directory as the repo root. Instead it opens with both
buttons disabled, the status line shows `ERROR: could not locate a canonical
CCGS repo root...`, and the log box lists every resolution attempt that was
tried, including which sidecar branch was rejected and which canonical
candidate(s) were tried. To recover:

- Rebuild via `powershell -ExecutionPolicy Bypass -File tools\dev-launcher\build-launcher-exe.ps1`
  from your canonical checkout so the sidecar is regenerated next to the EXE
  pointing at the canonical root, or
- Set `CCGS_REPO_ROOT` to your canonical checkout before launching the EXE, or
- Set `CCGS_CANONICAL_REPO_ROOT` if your canonical checkout lives outside the
  documented default location, or
- Copy the EXE inside the worktree so walk-up resolves.

#### Play/build root resolution

Independent of how the launcher root was found, the **play/build root** is
chosen in this order:

1. `$env:CCGS_PLAY_REPO_ROOT` (preferred environment override).
2. `$env:CCGS_CANONICAL_MAIN_ROOT` (alias retained for back-compat).
3. `D:\_DEV\ccgs-play-main` -- documented dedicated default.

The play/build path is **not** validated for existence at launcher startup --
the rebuild script handles the "doesn't exist yet" case by creating the path
as a linked git worktree off the launcher root. The Diagnostics panel reports
the current status (`exists, on main` / `exists, on branch 'X'` /
`missing -- will be created` / `path exists but is not a CCGS workspace`)
along with the source so testers can predict the rebuild outcome.

Notes:

- The play/build root **must** be distinct from the launcher root and from any
  worker worktree under `D:\_DEV\claude-code-game-studios-worktrees\`. If you
  set the env override to the launcher root, the EXE prints a warning that the
  dedicated-checkout safety net is disabled.
- `CCGS_REPO_ROOT` and `CCGS_CANONICAL_REPO_ROOT` continue to govern the
  **launcher** repo root only. `CCGS_PLAY_REPO_ROOT` (or its alias
  `CCGS_CANONICAL_MAIN_ROOT`) governs the play/build root.

### Why the sidecar must point at a canonical (on-main) checkout

`Rebuild Latest Main` (Button 1) calls `Update-LatestMain.ps1`, which refuses
to fast-forward unless the current branch is `main`. If
`build-launcher-exe.ps1` is run from a **worker worktree** on a `work/...`
branch (e.g. `D:\_DEV\claude-code-game-studios-worktrees\windows-dev-launcher-visual-polish-1255`)
and we blindly wrote that worktree path into the sidecar, the EXE would then
launch the rebuild against the worker worktree and the rebuild script would
refuse with:

```text
Refusing to fast-forward: current branch is 'work/<slug>', not 'main'.
```

PROMPT 1290 fixes this on both sides:

- The build script (`build-launcher-exe.ps1`) resolves a **canonical root** at
  write time (see "Build the EXE -- canonical-root resolution" below) and
  refuses to silently write a worker-worktree path. The dev-only escape hatch
  is `-AllowWorkerWorktreeSidecar`.
- The launcher (`ccgs-dev-launcher.exe`) reads `.git/HEAD` on whatever path
  the sidecar names. If the branch is not `main`, the sidecar is skipped and
  the canonical fallback is tried instead. The diagnostics panel shows the
  exact branch found so testers can see what happened.

### Sidecar file format

`ccgs-dev-launcher.repo-root.txt` is plain UTF-8 text (**no BOM**) written by
`build-launcher-exe.ps1` next to the built EXE. The first non-blank,
non-comment line is the absolute repo root path. Example contents:

```text
# ccgs-dev-launcher.repo-root.txt
# Generated by tools\dev-launcher\build-launcher-exe.ps1
# Consumed by tools/dev-launcher-app/src/main.rs at startup.
# Format: first non-blank, non-comment line is the absolute repo root.
D:\_DEV\Work\Claude-Code-Game-Studios
```

The sidecar is regenerated on every successful `build-launcher-exe.ps1` run.
`-DryRun` prints the resolved path that *would* be written without modifying
the filesystem.

> **Encoding note.** PowerShell 5.x `Set-Content -Encoding UTF8` writes a
> UTF-8 BOM (`0xEF 0xBB 0xBF`) before the first byte. With a comment header as
> line 1, the on-disk first line then reads `\u{FEFF}# ccgs-dev-launcher...`,
> which the parser's pre-1173 `trim().starts_with('#')` check did not
> recognise as a comment (U+FEFF is not Unicode `White_Space`), so the BOM-
> decorated comment line leaked through as the resolved "path" and the EXE
> surfaced `launcher script not found ...`. PROMPT 1173 fixes this on both
> sides: `build-launcher-exe.ps1` now writes the sidecar via
> `[System.IO.File]::WriteAllText(..., UTF8Encoding($false))` so no BOM is
> emitted, and `parse_sidecar_content` defensively strips a leading
> `\u{FEFF}` before parsing (covering any older BOM-prefixed sidecars left on
> disk). The build script also re-reads the first three bytes after writing
> and prints a warning if a BOM was emitted anyway.

### Race / double-click protection

- Both buttons are **disabled** while a job is running. They re-enable when
  the script exits (or errors).
- The status line shows the job kind plus the exit code (`DONE: ... (exit 0)`
  or `DONE WITH ERRORS: ... (exit N)`).
- The log box has a 2000-line ring buffer so a long run cannot grow it
  unboundedly.

### Build the EXE

```text
powershell -ExecutionPolicy Bypass -File tools\dev-launcher\build-launcher-exe.ps1
# release-mode (smaller / faster, slower build):
powershell -ExecutionPolicy Bypass -File tools\dev-launcher\build-launcher-exe.ps1 -Release
# build from a worker worktree but pin the sidecar at an explicit canonical:
powershell -ExecutionPolicy Bypass -File tools\dev-launcher\build-launcher-exe.ps1 `
    -CanonicalRepoRoot D:\_DEV\Work\Claude-Code-Game-Studios
```

#### Canonical-root resolution (sidecar contents)

Independent of which worktree you compile from, the sidecar is written with
the **canonical** repo root resolved in this order (first hit wins):

1. `-CanonicalRepoRoot <path>` (explicit argument).
2. `$env:CCGS_CANONICAL_REPO_ROOT` (environment override).
3. The build checkout itself, **only if** its current branch is `main`.
4. `D:\_DEV\Work\Claude-Code-Game-Studios` (the documented default), if it
   exists and is a valid CCGS workspace.
5. **Refuse** to write the sidecar (exit 2) unless `-AllowWorkerWorktreeSidecar`
   is also passed -- in which case the worker-worktree path is written with
   an inline warning that `Rebuild Latest Main` will not work against it.

This is the PROMPT 1290 fix for "EXE built in worker worktree -> sidecar pins
to `work/...` branch -> Rebuild Latest Main refuses". Day-to-day you do not
need to think about it: building from a canonical checkout on `main` writes
the canonical path automatically; building from anywhere else falls back to
the documented default.

The build script applies the documented Windows/MSVC Cargo resource policy
(`CARGO_TARGET_DIR=D:\_DEV\cargo-target\ccgs-msvc`,
`CARGO_PROFILE_DEV_DEBUG=0`, etc.) and then runs:

```text
cargo build -p dev-launcher-app --bin ccgs-dev-launcher [--release]
```

Resulting EXE path:

```text
D:\_DEV\cargo-target\ccgs-msvc\debug\ccgs-dev-launcher.exe
# or with -Release:
D:\_DEV\cargo-target\ccgs-msvc\release\ccgs-dev-launcher.exe
```

The debug-profile EXE is ~450 KB; the release-profile EXE is smaller still
(LTO + strip + panic=abort per the workspace `[profile.release]`).

Alternatively, build directly with cargo from the repo root:

```text
$env:CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'
$env:CARGO_PROFILE_DEV_DEBUG='0'
$env:CARGO_PROFILE_TEST_DEBUG='0'
$env:CARGO_INCREMENTAL='0'
$env:RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'
cargo build -p dev-launcher-app --bin ccgs-dev-launcher
```

### Run the EXE

Double-click the EXE in Explorer, or run from PowerShell:

```text
D:\_DEV\cargo-target\ccgs-msvc\debug\ccgs-dev-launcher.exe
```

If you copy the EXE outside the worktree (e.g. to a tester's desktop) **without
copying the sidecar with it**, set `CCGS_REPO_ROOT` before launching so it can
find the scripts:

```text
$env:CCGS_REPO_ROOT='D:\_DEV\Work\Claude-Code-Game-Studios'
D:\_DEV\cargo-target\ccgs-msvc\debug\ccgs-dev-launcher.exe
```

Otherwise, keep `ccgs-dev-launcher.repo-root.txt` in the same directory as
`ccgs-dev-launcher.exe` -- copying the EXE to a new location is enough as
long as that sibling sidecar travels with it.

### What the EXE does NOT do

- It does not replace the `.bat` / `.ps1` paths. Both remain supported and
  are the source of truth for behavior, safety, and flags.
- It does not pass any flags through. To use `-Release`, `-Port`,
  `-StrictPort`, `-Force`, etc., run the underlying script directly. The EXE
  is the zero-friction tester path; the script is the power-user path.
- It does not start any sidecar, fetch git, or write outside the script's
  own evidence dir.

### Validation (PROMPT 1162)

- `cargo build -p dev-launcher-app --bin ccgs-dev-launcher` -- compiles
  clean under the documented Cargo policy. Resulting EXE is ~450 KB debug.
- `cargo test -p dev-launcher-app` -- 7 unit tests pass (evidence-dir
  parsing, log ring-buffer truncation, button-label / script-path
  invariants).
- `powershell -File tools\dev-launcher\build-launcher-exe.ps1 -Help` --
  prints usage; exit 0.
- `powershell -File tools\dev-launcher\build-launcher-exe.ps1 -DryRun` --
  prints the cargo command without invoking it; exit 0.
- End-to-end click of the rebuild / launch buttons was **not** exercised
  under PROMPT 1162 (out of scope per the prompt: "Do not actually click /
  execute rebuild or two-client launch unless using a dry-run mode").

### Validation (PROMPT 1170 -- sidecar repair)

- Added `ccgs-dev-launcher.repo-root.txt` sidecar written by
  `build-launcher-exe.ps1` and consumed by the EXE at startup. Resolves
  the original "Repo root: D:\\_DEV\\cargo-target\\ccgs-msvc\\debug" bug
  when the EXE is launched from the external Cargo target directory.
- New resolution order: env -> sidecar -> EXE walk-up -> cwd walk-up ->
  clear error (no silent fallback to `target/debug`).
- New unit tests in `tools/dev-launcher-app/src/main.rs` cover:
  sidecar parsing (trim, comment lines, empty rejection), sidecar
  read-from-disk via the system temp dir, env-wins-over-sidecar,
  invalid-env-falls-through-to-sidecar, sidecar-falls-through-to-walk-up,
  failed-resolution surfaces every attempt, and the explicit assertion
  that `target/debug` is never accepted as the repo root.

### Validation (PROMPT 1290 -- canonical-main sidecar repair)

- Root cause: `build-launcher-exe.ps1` previously wrote whatever
  `Split-Path -Parent (Split-Path -Parent $ScriptDir)` returned -- i.e. the
  build checkout. Building from a worker worktree (e.g.
  `D:\_DEV\claude-code-game-studios-worktrees\windows-dev-launcher-visual-polish-1255`)
  on a `work/...` branch pinned the EXE to that worktree. `Rebuild Latest
  Main` then refused: `current branch is 'work/...', not 'main'`.
- Launcher fix: `resolve_repo_root_pure` now (a) accepts the sidecar only
  when its repo is on branch `main` (`.git/HEAD` is read; linked-worktree
  `.git` file pointers are followed); (b) on a non-main sidecar, falls back
  to a configurable canonical candidate list (`CANONICAL_REPO_CANDIDATES`
  in `main.rs`, default `[D:\_DEV\Work\Claude-Code-Game-Studios]`, override
  via `CCGS_CANONICAL_REPO_ROOT`); (c) surfaces the resolved branch in the
  Diagnostics panel and the rejected branch in the attempts list.
- Build-script fix: `build-launcher-exe.ps1` resolves a canonical root
  (`-CanonicalRepoRoot` > `$env:CCGS_CANONICAL_REPO_ROOT` > build checkout
  if on main > documented default), refuses to write the sidecar when no
  canonical is discoverable unless `-AllowWorkerWorktreeSidecar` is passed,
  and stamps the resolved canonical source into a sidecar comment.
- Both buttons (`Rebuild Latest Main`, `Start Two-Client Play Session`) use
  the same resolved repo root -- a worker-worktree sidecar can no longer
  mislead either flow.
- New unit tests in `tools/dev-launcher-app/src/main.rs`:
  - `resolve_repo_root_sidecar_on_worker_branch_falls_to_canonical`
  - `resolve_repo_root_sidecar_on_main_is_accepted_without_canonical_fallback`
  - `resolve_repo_root_env_overrides_valid_sidecar_pointing_elsewhere`
  - `resolve_repo_root_invalid_canonical_yields_actionable_error`
  - `resolve_repo_root_canonical_fallback_records_branch_label_for_unknown_head`
  - `canonical_repo_candidates_has_at_least_one_entry`
  - `read_head_branch_returns_main_for_regular_checkout`
  - `read_head_branch_returns_worker_branch_name`
  - `read_head_branch_returns_none_for_detached_head`
  - `read_head_branch_follows_worktree_gitdir_pointer`

### Validation (PROMPT 1309 -- dedicated play/build checkout)

- User-reported regression after PROMPT 1304: the canonical-checkout
  fallback resolved the launcher repo root to
  `D:\_DEV\Work\Claude-Code-Game-Studios`, but that orchestrator checkout was
  on branch `work/s18-server-dead-state-hygiene-...`. `Update-LatestMain.ps1`
  refused with `current branch is 'work/...', not 'main'`.
- Root cause: the two-button launcher conflated the **launcher** repo root
  (where the scripts live) with the **play/build** root (where the rebuild
  switches and merges). The orchestrator checkout can legitimately be dirty
  or on a worker branch at any time; coupling the two meant the rebuild was
  blocked whenever the orchestrator was busy.
- Fix: introduced a structurally separate **play/build root** with its own
  resolution priority (`-PlayRepoRoot` > `CCGS_PLAY_REPO_ROOT` >
  `CCGS_CANONICAL_MAIN_ROOT` > `D:\_DEV\ccgs-play-main`). `Update-LatestMain.ps1`
  materialises this path as a linked git worktree off the launcher root the
  first time it runs, and only ever switches branches inside the dedicated
  checkout. `Start-TwoClients.ps1` builds and writes evidence under the same
  dedicated path. The launcher EXE always passes `-PlayRepoRoot <path>` so
  the script gets a single, well-defined target.
- Diagnostics panel now shows two blocks: launcher repo root + branch +
  resolution source, and play/build root + branch + status + source. Status
  values are `exists, on main` / `exists, on branch '<X>'` /
  `exists, detached HEAD or unknown` / `missing -- will be created as a
  worktree on first rebuild` / `path exists but is not a CCGS workspace`.
- New unit tests in `tools/dev-launcher-app/src/main.rs`:
  - `play_root_default_constant_is_separate_from_canonical_root`
  - `play_root_default_is_not_inside_worktree_directory`
  - `resolve_play_root_prefers_env_over_legacy_and_default`
  - `resolve_play_root_uses_legacy_env_when_primary_unset`
  - `resolve_play_root_uses_documented_default_when_no_env`
  - `resolve_play_root_treats_empty_or_whitespace_env_as_unset`
  - `resolve_play_root_status_missing_when_path_absent`
  - `resolve_play_root_status_on_main_when_validated_and_main`
  - `resolve_play_root_status_other_branch_when_worker_checkout`
  - `resolve_play_root_status_detached_when_branch_unknown`
  - `resolve_play_root_status_invalid_when_path_exists_but_not_repo`
  - `play_root_source_human_strings_are_distinct`
  - `play_root_env_constant_names_match_documented_pair`
  - `diagnostics_text_reports_play_root_status_distinctly_from_launcher`
  - `diagnostics_text_shows_play_branch_when_play_root_on_other_branch`
  - `play_root_status_human_labels_are_actionable`
- Existing PROMPT 1170 / 1173 / 1290 sidecar + BOM + canonical-fallback
  tests remain green; the dedicated-checkout change is additive.

### Validation (PROMPT 1173 -- BOM repair on integration refresh)

- Runtime bug discovered: with PROMPT 1170's writer
  (`Set-Content -Encoding UTF8`) on PowerShell 5.x, the sidecar gained a
  UTF-8 BOM and the parser then resolved the BOM-prefixed comment header
  as the "repo root" path, re-triggering "launcher script not found ..."
  even though the file existed on disk.
- Writer fix: `build-launcher-exe.ps1` now uses
  `[System.IO.File]::WriteAllText(..., UTF8Encoding($false))` so the
  sidecar is written as UTF-8 **without** BOM, and re-reads the first
  three bytes post-write to warn if a BOM is somehow still present.
- Parser fix: `parse_sidecar_content` strips a leading `\u{FEFF}` BOM
  from the raw text before iterating lines, and `trim_matches` ignores
  any stray BOM characters on individual lines.
- New tests in `tools/dev-launcher-app/src/main.rs` cover:
  `parse_sidecar_content_skips_bom_prefixed_comment_header`,
  `parse_sidecar_content_strips_bom_directly_before_path`, and
  `read_sidecar_root_handles_utf8_bom_with_comment_header` (end-to-end:
  write a BOM+comment-header+path body via `fs::write` into a temp dir
  and assert the parser returns the bare path, not the BOM line).

## Recommended workflow

```text
# First time per session, or after a tracked-upstream change has landed:
update-latest-main.bat

# Every time you want a fresh manual two-client test:
start-two-clients.bat
```

If port 5000 is already used (e.g. another local server still running):

```text
start-two-clients.bat -Port 5050
```

For a release-mode build (slower compile, faster runtime), pass `-Release`
to both buttons:

```text
update-latest-main.bat -Release
start-two-clients.bat -Release
```

## Safety contract

- **Dirty tree** -- Button 1 refuses to fast-forward unless `-Force`.
- **Non-main branch** -- Button 1 refuses unless `-Force`.
- **Non-FF main** -- Button 1 refuses unless `-Force`, in which case it
  hard-resets and warns DESTRUCTIVE inline.
- **Busy port** -- Button 2 auto-bumps to the next free port unless
  `-StrictPort` is set, in which case it exits 2.
- **Low disk on D:** -- Button 1 warns. Only with `-AllowCacheClean` does
  it remove anything, and only inside the resolved `CARGO_TARGET_DIR`
  (with a guard that aborts cleanup if the resolved path drifted from
  the documented policy root).

## Validation (PROMPT 1155)

Both scripts were validated with:

- `[System.Management.Automation.Language.Parser]::ParseFile(...)` --
  zero parser errors on both `.ps1` files.
- `... -Help` -- prints the usage banner and exits 0 without touching
  git, cargo, or the filesystem.
- `... -DryRun -Help` semantics -- not combined in this story (see the
  per-script help banners).

Full launch (`start-two-clients.bat` without `-DryRun`) was **not** run
in CI under PROMPT 1155 -- the story is tooling/docs only and explicitly
out of scope for cargo full builds in this branch's commit. Run it
locally to validate the end-to-end path.
