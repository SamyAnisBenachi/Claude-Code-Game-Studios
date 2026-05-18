# Dev Two-Button Launcher

> Story-scope: PROMPT 1155 -- One-Button (now Two-Button) Latest-Main
> Two-Client Test Launcher.
> Path: `tools/dev-launcher/` (scripts), `update-latest-main.bat` +
> `start-two-clients.bat` (one-click wrappers at repo root).
> Authored: 2026-05-18.

Two one-click developer-only launchers for the manual two-client friend-game
test loop. Each script is self-contained and exits without modifying any
git remote, production tracker, sprint state, QA artifact, or evidence
runbook.

## Where to click

| Button | Wrapper at repo root | Real script |
|--------|----------------------|-------------|
| **Update + Rebuild** | `update-latest-main.bat` | `tools\dev-launcher\Update-LatestMain.ps1` |
| **Launch 2 Clients** | `start-two-clients.bat` | `tools\dev-launcher\Start-TwoClients.ps1` |

Both `.bat` files invoke `powershell -NoProfile -ExecutionPolicy Bypass -File`
under the hood and pass `%*` through, so any extra PowerShell flags below can
be appended to the `.bat` invocation directly, e.g.
`update-latest-main.bat -Release` or `start-two-clients.bat -Port 5050`.

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
