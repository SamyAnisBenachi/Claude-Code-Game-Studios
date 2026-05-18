# Dev Two-Button Launcher

> Story-scope: PROMPT 1155 -- One-Button (now Two-Button) Latest-Main
> Two-Client Test Launcher.
> PROMPT 1162 added the optional native Windows EXE wrapper under
> `tools/dev-launcher-app/`.
> Path: `tools/dev-launcher/` (scripts), `update-latest-main.bat` +
> `start-two-clients.bat` (one-click wrappers at repo root),
> `tools/dev-launcher-app/` (EXE wrapper).
> Authored: 2026-05-18.

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

## Button 1 -- Update + Rebuild (`update-latest-main.bat`)

### What it does (in order)

1. Resolves the repo root from the script location.
2. Aborts unless the working tree is clean and the current branch is `main`
   (override with `-Force`).
3. `git fetch origin`.
4. Fast-forwards local `main` to `origin/main`. Aborts on non-FF unless
   `-Force` is passed (which then performs a destructive
   `git reset --hard origin/main`).
5. Applies the documented Windows / MSVC Cargo resource policy:
   - `CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'`
   - `CARGO_PROFILE_DEV_DEBUG='0'`
   - `CARGO_PROFILE_TEST_DEBUG='0'`
   - `CARGO_INCREMENTAL='0'`
   - `RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'`
6. Checks D: free space. Under 40 GB plus `-AllowCacheClean` triggers
   cleanup of stale subdirectories **only under** the resolved
   `CARGO_TARGET_DIR` (a hard-coded match against
   `D:\_DEV\cargo-target\ccgs-msvc` -- never source, reports, production,
   `.git`, or evidence).
7. `cargo build -p server`.
8. `cargo build -p client --bin client`.
9. Prints the resolved binary paths and a "Next: ..." hint.

### What it does NOT do

- Does **not** start the server or any client.
- Does **not** push, force-push, or modify any remote branch.
- Does **not** modify `production/`, `qa/`, sprint trackers, story files,
  or evidence runbooks.
- Does **not** run tests.

### Flags

| Flag | Effect |
|------|--------|
| `-Force` | Allow dirty tree (no stash) and non-FF main reset. DESTRUCTIVE. |
| `-Release` | Build in release profile (default is debug). |
| `-AllowCacheClean` | Under-40-GB free space cleans stale Cargo target subdirs. |
| `-DryRun` | Print every step; run no git, cargo, or rm command. |
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
| `Rebuild Latest Main` | `powershell.exe -NoProfile -ExecutionPolicy Bypass -File tools\dev-launcher\Update-LatestMain.ps1` |
| `Start Two-Client Play Session` | `powershell.exe -NoProfile -ExecutionPolicy Bypass -File tools\dev-launcher\Start-TwoClients.ps1` |

Both invocations use the **resolved repo root** as the working directory.

### Repo-root resolution order

The EXE typically lives at
`D:\_DEV\cargo-target\ccgs-msvc\debug\ccgs-dev-launcher.exe` -- **outside** the
repo tree. Walking up from there never reaches the worktree. The launcher
therefore resolves the repo root in the following order at startup, stopping at
the first source whose path passes the validation check
(`Cargo.toml`, `tools/dev-launcher/`, and `.git` all present):

1. `CCGS_REPO_ROOT` environment variable.
2. `ccgs-dev-launcher.repo-root.txt` sidecar file living **next to the EXE**.
   `build-launcher-exe.ps1` writes this sidecar with the absolute repo root on
   the first non-blank line; lines starting with `#` are comments. This is the
   default working path when running the EXE from the external Cargo target
   dir.
3. The EXE's own directory, walked upward (useful when the EXE was copied
   inside the worktree).
4. The current working directory, walked upward.

If none of these resolves to a valid repo root the EXE **does not** silently
adopt the EXE directory as the repo root. Instead it opens with both buttons
disabled, the status line shows `ERROR: could not locate CCGS repo root...`,
and the log box lists every resolution attempt that was tried. To recover:

- Rebuild via `powershell -ExecutionPolicy Bypass -File tools\dev-launcher\build-launcher-exe.ps1`
  so the sidecar is regenerated next to the EXE, or
- Set `CCGS_REPO_ROOT` to the repo path before launching the EXE, or
- Copy the EXE inside the worktree so walk-up resolves.

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
```

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
