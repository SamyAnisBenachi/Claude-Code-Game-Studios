# S8-QA-001 Command Summary

**Date**: 2026-05-07
**Commit**: `3cc620cdeee6f5249e404703365b160ccbc34f6c`
**Branch status before evidence edits**: `## main...origin/main`
**Target**: local Windows Cargo, Bevy 0.18 + Lightyear 0.26
**Smoke port**: `5018` for bounded server startup note
**Scope**: internal 1v1 friend-game smoke/evidence only

All Cargo commands used:

```powershell
$env:CARGO_PROFILE_DEV_DEBUG='0'
$env:RUSTFLAGS='-C link-arg=/DEBUG:NONE'
```

| Command | Result |
|---|---|
| `git rev-parse HEAD` | PASS: `3cc620cdeee6f5249e404703365b160ccbc34f6c` |
| `git status --short --branch` | PASS: `## main...origin/main` |
| `cargo test -p server --test playable_client_active_loop_polish_test` | PASS: 4 passed |
| `cargo test -p client --test playable_client_active_loop_ui_state_test` | PASS: 4 passed |
| `cargo test -p server --test playable_client_friend_game_result_endpoint_test` | PASS: 1 passed |
| `cargo test -p server --test playable_client_real_e2e_loop_test` | PASS: 4 passed |
| `cargo test -p client --tests` | PASS: valid equivalent for the incomplete prompt command `cargo test -p client --test`; `--list` reports 292 client tests |
| `cargo check --workspace` | PASS |
| `git diff --check` | PASS |

The prompt line `cargo test -p client --test` is not a complete Cargo command
because `--test` requires a test target name. For smoke coverage it was
interpreted as:

```powershell
cargo test -p client --tests
```

No source code, sprint status, story status, QA sign-off, team-qa, gate-check,
Sprint 8 close-out, public release readiness, broad accessibility completion,
playtest validation, full playable-client manual QA, or full game completion is
claimed by this command summary.
