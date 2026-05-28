# PROMPT 1743 — BOT-SOAK-ENTRYPOINT-AC6-SERVER-GATE-REPAIR

**Date:** 2026-05-28  
**Branch:** `work/bot-soak-entrypoint-ac6-1743`  
**Base:** `origin/main` @ `511b193e`

---

## Problem

`BOT-SOAK-ENTRYPOINT-001 AC6` was PARTIAL: the server-side
`handle_create_bot_room` system was registered and active in release with no
cfg, feature, or env gate. Any connected client could send `C2SCreateBotRoom`
and get a bot room created unconditionally.

---

## Fix

**Approach chosen:** Runtime env gate inside `handle_create_bot_room`
(priority 1 from the task spec — smallest maintainable fix, no Cargo feature
topology change needed).

### Env var contract

| Var | Required value | Effect |
|-----|----------------|--------|
| `CCGS_BOT_SOAK_ENABLED` | `"1"` (exact, whitespace trimmed) | Bot room creation allowed |
| any other value / unset | — | All `C2SCreateBotRoom` messages silently drained; `warn` log emitted per message |

### Files changed

| File | Change |
|------|--------|
| `server/src/feature/bot/soak_config.rs` | Added `BOT_SOAK_ENABLED_ENV_VAR` constant + `is_bot_soak_enabled()` helper |
| `server/src/feature/bot/mod.rs` | Re-exported both new items |
| `server/src/core/session/system.rs` | Added `use crate::feature::bot::is_bot_soak_enabled` import + gate block at top of `handle_create_bot_room` |
| `server/Cargo.toml` | Registered new `bot_soak_entrypoint_gate_test` `[[test]]` entry |
| `tests/unit/bot/bot_soak_entrypoint_gate_test.rs` | 7 unit tests for `is_bot_soak_enabled()` |

### Gate block in `handle_create_bot_room` (system.rs)

```rust
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
```

The handler continues to its normal logic only when the gate is open.

---

## Tests

7 new unit tests in `tests/unit/bot/bot_soak_entrypoint_gate_test.rs`:

| Test | Expectation |
|------|-------------|
| `test_gate_disabled_by_default` | `false` when env var absent |
| `test_gate_enabled_for_exactly_one` | `true` for `"1"` |
| `test_gate_enabled_for_one_with_surrounding_whitespace` | `true` for `"  1  "` (trim) |
| `test_gate_disabled_for_zero` | `false` for `"0"` |
| `test_gate_disabled_for_true_string` | `false` for `"true"` |
| `test_gate_disabled_for_yes_string` | `false` for `"yes"` |
| `test_gate_disabled_for_empty_string` | `false` for `""` |

**Result:** 7/7 PASS. `cargo check -p server` PASS. `git diff --check` PASS.

---

## Path allowlist check

All edits within owned scope:
- `server/src/feature/bot/soak_config.rs` ✓
- `server/src/feature/bot/mod.rs` ✓
- `server/src/core/session/system.rs` ✓
- `server/Cargo.toml` ✓
- `tests/unit/bot/bot_soak_entrypoint_gate_test.rs` ✓
- `reports/PROMPT-1743-bot-soak-entrypoint-ac6-server-gate-repair.md` ✓

No forbidden files touched (no client UI, no production state, no sprint files,
no unrelated CI/Cargo files).

---

1743: BOT-SOAK-ENTRYPOINT-AC6-SERVER-GATE-REPAIR: SHIPPED
