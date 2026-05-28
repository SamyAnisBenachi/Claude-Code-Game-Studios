# PROMPT 1746 — BOT-SOAK-AC6-SERVER-GATE-POST-MAINLAND-VERIFY

**Date:** 2026-05-28
**Worktree:** `D:\tmp\wt-1746-ac6-verify`
**Branch:** `verify/1746-bot-soak-ac6-server-gate-post-mainland`
**Base:** `origin/main` @ `ffa6cf9d` (PROMPT 1745 tip — includes 1743 gate repair + 1745 whitespace fix)

---

## Objective

Verify that `BOT-SOAK-ENTRYPOINT-001 AC6` is satisfied on latest `origin/main`
after PROMPT 1743 (gate repair) and PROMPT 1745 (whitespace fix) landed.

---

## Verification Method

Static grep + file read of gate implementation, re-export chain, plugin
registration, and test registration. `git diff --check HEAD` for whitespace
cleanliness. No broad Cargo run (verify-only scope).

---

## Findings

### 1. Gate helper — `is_bot_soak_enabled()` ✅

File: `server/src/feature/bot/soak_config.rs:39-41`

```rust
pub fn is_bot_soak_enabled() -> bool {
    std::env::var(BOT_SOAK_ENABLED_ENV_VAR)
        .map(|v| v.trim() == "1")
        .unwrap_or(false)
}
```

- Reads `CCGS_BOT_SOAK_ENABLED` from process environment.
- Trims whitespace before comparing.
- Returns `true` only for the exact trimmed value `"1"`.
- Returns `false` for: absent (via `unwrap_or(false)`), `"0"`, `"true"`, `"yes"`, `""`, any other value.

### 2. Gate block in `handle_create_bot_room` ✅

File: `server/src/core/session/system.rs:618–645`

```rust
/// Gated by `CCGS_BOT_SOAK_ENABLED=1` (PROMPT 1743 / BOT-SOAK-ENTRYPOINT-001 AC6).
pub fn handle_create_bot_room(...) {
    if !is_bot_soak_enabled() {
        // Drain without acting — env gate not set.
        for (_remote, mut receiver) in receivers.iter_mut() {
            for _msg in receiver.receive() {
                tracing::warn!(
                    "c2s_create_bot_room: request blocked — CCGS_BOT_SOAK_ENABLED not set"
                );
            }
        }
        return;
    }
    // ... normal bot room creation logic ...
}
```

- Gate check is the **first action** in the handler body.
- When gate is closed: all `C2SCreateBotRoom` messages are drained, `warn!` emitted per message, function returns early.
- No bot room is created in the default/release path.
- When gate is open: handler proceeds to `create_bot_room(...)` normally.

### 3. Default / release-like behavior ✅

When `CCGS_BOT_SOAK_ENABLED` is absent (standard deploy, release builds):

- `std::env::var(...)` returns `Err`.
- `unwrap_or(false)` → `false`.
- `!is_bot_soak_enabled()` → `true` → early-drain branch taken.
- `C2SCreateBotRoom` messages are discarded with a `warn` log.
- **No bot room is created. Entrypoint not exposed.**

### 4. Enabled path (soak mode) ✅

When `CCGS_BOT_SOAK_ENABLED=1`:

- `is_bot_soak_enabled()` → `true`.
- Gate block not entered.
- Handler processes `C2SCreateBotRoom` and calls `create_bot_room(...)`.
- Covered by the focused test `bot_soak_entrypoint_gate_test` (see below).

### 5. Re-export chain ✅

File: `server/src/feature/bot/mod.rs:31-32`

```rust
is_bot_soak_enabled, BotSoakConfig, BotSoakPlugin, BOT_MAX_ROUNDS_ENV_VAR,
BOT_SOAK_ENABLED_ENV_VAR,
```

Both `is_bot_soak_enabled` and `BOT_SOAK_ENABLED_ENV_VAR` are re-exported from
the `bot` feature module — the test file imports them via `server::feature::bot`.

### 6. Plugin registration ✅

File: `server/src/core/session/plugin.rs:49`

```rust
handle_create_bot_room,
```

The system is registered unconditionally in `Update` — correct. The gate is a
runtime env-var check inside the function body, not a compile-time or startup
conditional. All `C2SCreateBotRoom` messages are always routed to the handler,
which then drains or processes them depending on the env var.

### 7. Test registration ✅

File: `server/Cargo.toml`

```toml
[[test]]
name = "bot_soak_entrypoint_gate_test"
path = "../tests/unit/bot/bot_soak_entrypoint_gate_test.rs"
```

Test binary is registered and will be picked up by `cargo test -p server --test
bot_soak_entrypoint_gate_test`.

### 8. Test coverage ✅

File: `tests/unit/bot/bot_soak_entrypoint_gate_test.rs` — 7 tests:

| Test | Scenario | Expected |
|------|----------|----------|
| `test_gate_disabled_by_default` | env var absent | `false` |
| `test_gate_enabled_for_exactly_one` | `"1"` | `true` |
| `test_gate_enabled_for_one_with_surrounding_whitespace` | `"  1  "` | `true` |
| `test_gate_disabled_for_zero` | `"0"` | `false` |
| `test_gate_disabled_for_true_string` | `"true"` | `false` |
| `test_gate_disabled_for_yes_string` | `"yes"` | `false` |
| `test_gate_disabled_for_empty_string` | `""` | `false` |

Tests use `ENV_LOCK: Mutex<()>` to serialize env mutations — race-safe.

### 9. Whitespace / diff cleanliness ✅

`git diff --check HEAD` → no trailing whitespace, no CRLF issues.

---

## AC6 Verdict

| Acceptance Criterion | Check | Result |
|---|---|---|
| `handle_create_bot_room` gated by `CCGS_BOT_SOAK_ENABLED=1` | Static read `system.rs:633` | ✅ PASS |
| Default (env absent) → no bot room created | Logic trace `soak_config.rs:39-41` | ✅ PASS |
| Enabled path (`=1`) → bot room allowed | Gate logic + test coverage | ✅ PASS |
| Whitespace-trimming of `"  1  "` | `soak_config.rs` + test | ✅ PASS |
| Near-misses (`"0"`, `"true"`, `"yes"`, `""`) rejected | 4 tests | ✅ PASS |
| Re-export chain intact | `bot/mod.rs` | ✅ PASS |
| Test registered in `server/Cargo.toml` | `[[test]]` entry | ✅ PASS |
| `git diff --check` clean | HEAD | ✅ PASS |

**BOT-SOAK-ENTRYPOINT-001 AC6 is SATISFIED on `origin/main` @ `ffa6cf9d`.**

---

1746: BOT-SOAK-AC6-SERVER-GATE-POST-MAINLAND-VERIFY: VERIFIED
