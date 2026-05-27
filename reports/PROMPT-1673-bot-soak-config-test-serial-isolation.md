# PROMPT 1673 -- BOT-SOAK-CONFIG-TEST-SERIAL-ISOLATION

Date: 2026-05-27
Status: SHIPPED
Branch: `work/1673-bot-soak-config-test-serial-isolation`
Worker commit: `f296d472`
Integration refresh commit: `bcad0b74`

## Summary

`bot_soak_config_test` global-env flakiness is fixed. Five tests that mutate
`CCGS_BOT_MAX_ROUNDS` now acquire a process-local `static ENV_LOCK: Mutex<()>`
before calling `std::env::set_var` or `std::env::remove_var`.

This prevents Cargo's default parallel test runner from racing process-global
environment state. No production code was changed.

## Root Cause

`std::env::set_var` and `remove_var` mutate a process-global environment block.
The affected tests all run inside the same test binary under default Cargo
parallelism, so one test could overwrite the env var while another was reading
it. PROMPT 1664 observed the concrete failure mode:
`test_from_env_positive_integer_activates_bound` read `Some(3)` instead of
`Some(10)` when `test_from_env_whitespace_trimmed` raced in.

## Changes

Changed file:

- `tests/unit/bot/bot_soak_config_test.rs`

Applied changes:

- Added `use std::sync::Mutex;`.
- Added module-level `static ENV_LOCK: Mutex<()> = Mutex::new(());`.
- Added `let _guard = ENV_LOCK.lock().unwrap();` to each `from_env_*` test that
  touches `CCGS_BOT_MAX_ROUNDS`.
- Left pure struct/resource tests unlocked because they do not access env vars.
- Added comments documenting the parallelism contract.

## Validation From Worker

- `cargo test --test bot_soak_config_test -p server` => PASS, 7/7.
- `cargo test --test bot_soak_config_test -p server -- --test-threads=1` =>
  PASS, 7/7.

## Scope

- Test-only change.
- No production runtime semantics changed.
- No Cargo, CI, sprint, session-state, client, or launcher files changed by the
  worker.

1673: BOT-SOAK-CONFIG-TEST-SERIAL-ISOLATION: SHIPPED
