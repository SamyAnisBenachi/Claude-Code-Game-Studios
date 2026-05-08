# S9-QA-001 Command Summary

## Baseline

| Field | Value |
|---|---|
| Date | 2026-05-08 |
| Start time (UTC) | 2026-05-08T00:26:00Z |
| Commit (HEAD) | d2ac17ccb2c19be18dd1b2de63d2f2e968c235c8 |
| Origin/main | d2ac17ccb2c19be18dd1b2de63d2f2e968c235c8 |
| Branch | main |
| Dirty state | 3 staged files: production/session-state/active.md, production/sprint-status.yaml, production/sprints/sprint-9.md — all sprint-management files; no source or test files dirty |
| OS / target | Windows 11 Home 10.0.26200 (MINGW64_NT-10.0-26200) |
| Rust toolchain | rustc 1.95.0 (59807616e 2026-04-14); cargo 1.95.0 (f2d3ce0bd 2026-03-21) |
| Build target | msvc-local (target/msvc-local/debug/) |

## Route Choice

**Route chosen**: Native (headless server + two native client windows).
Browser/WASM route not selected: trunk not invoked; browser context not opened.

## Server Launch Command

```bash
SERVER_PORT=5000 target/msvc-local/debug/server.exe
```

Server binary launched and ran for 8 seconds without panic (killed by timeout
to stay non-interactive). Exit code 124 (timeout kill). Bevy logging wrote to
Windows console API rather than the redirected file descriptor; server-startup.log
is therefore empty. No panic, no crash, no immediate exit.

## Client Launch Commands (documented but not executed)

Two clients from the same commit are required:

```powershell
# Terminal 1 — server (running)
$env:SERVER_PORT='5000'
cargo run -p server

# Terminal 2 — Client A (host)
$env:SERVER_URL='ws://localhost:5000'
cargo run -p client --bin client

# Terminal 3 — Client B (joiner)
$env:SERVER_URL='ws://localhost:5000'
cargo run -p client --bin client
```

These commands are correctly documented in
`production/qa/evidence/manual-friend-game-evidence-runbook.md`.
They were NOT executed during this run because the full GUI route requires
a human operator with visual access to running Bevy windows (see blocker below).

## Regression Commands Run

| Command | Result |
|---|---|
| `cargo test -p server --test result_acknowledgement_contract_test` | PASS — 5/5 |
| `cargo test -p server --test result_acknowledgement_cleanup_handshake_test` | PASS — 3/3 |
| `cargo test -p client --test result_screen_mvp_test` | PASS — 6/6 |
| `cargo test -p client --test result_screen_return_to_lobby_test` | PASS — 2/2 |
| `cargo check --workspace` | PASS |
| `git diff --check` | PASS (exit 0) |
| `git diff --cached --check` | Not run — committing evidence files only by explicit path (see commit notes) |

Total automated regression: 16/16 tests pass.

## Manual Route Execution

**NOT EXECUTED.** The full manually driven two-client GUI route is blocked
because this evidence run was executed by a non-interactive AI agent that
cannot operate Bevy windowed client applications.

See `defects.md` for the formal blocker record (MANUAL-FG-001).

## Stop Time

2026-05-08T00:27:00Z (approximate; evidence writing phase follows)
