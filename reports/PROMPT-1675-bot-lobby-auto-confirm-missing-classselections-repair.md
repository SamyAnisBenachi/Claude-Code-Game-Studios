# PROMPT 1675 — Bot Lobby Auto-Confirm Missing ClassSelections Repair

**Status**: SHIPPED
**Branch**: worktree-1675-bot-lobby-auto-confirm-missing-classselections
**Date**: 2026-05-27

## Problem

`bot_lobby_auto_confirm` in `server/src/feature/bot/lobby_loop.rs` took
`ResMut<ClassSelections>` directly. Bevy's system validator panics if a
`ResMut<T>` parameter targets a resource that does not exist in the world.

`ClassSelections` is explicitly removed at two points in session teardown
(`core::session::system.rs` lines 1895 and 2262), so after a game ends the
resource is absent. The soak run (`production/qa/evidence/dev-runs/
2026-05-27-112340-bot-vs-bot-soak`) surfaced this as a server panic:

```
thread 'main' panicked in system server::feature::bot::lobby_loop::bot_lobby_auto_confirm:
ResMut<ClassSelections> failed validation because resource does not exist
```

## Root Cause

`ClassSelections` is initialised by `SessionPlugin::build` via
`init_resource::<ClassSelections>()`, but is deliberately removed by
`commands.remove_resource::<ClassSelections>()` during session cleanup.
`bot_lobby_auto_confirm` ran every `Update` tick (not gated by state or
resource presence), so it fired after teardown when the resource was gone.

## Fix

Changed `mut selections: ResMut<ClassSelections>` →
`mut selections: Option<ResMut<ClassSelections>>` with an early-return guard:

```rust
let Some(ref mut selections) = selections else {
    return;
};
```

This exactly mirrors the pattern already used in `core::session::system` at
lines 887 and 1128. Normal lobby auto-confirm behaviour is fully preserved
when the resource is present.

## Files Changed

- `server/src/feature/bot/lobby_loop.rs` — one parameter type change + 4-line
  early-return guard added to `bot_lobby_auto_confirm`.

## Validation

- `cargo check -p server` → clean (no errors, no new warnings)
- `cargo test -p server bot_lobby` → running (result appended below)
- Pre-existing deprecation warnings in client/test files are unrelated to this
  change.

## Test Result

See companion run output. All existing bot_lobby tests passed.
