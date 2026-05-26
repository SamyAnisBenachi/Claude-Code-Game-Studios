# PROMPT 1640 — Bot Vs Bot Max Rounds Bounded Soak

**Branch:** `prompt-1640-bot-max-rounds`
**Worktree:** `D:/_DEV/Work/ccgs-1640-bot-max-rounds`

---

## Problem

`GameOver` in bot-vs-bot soak runs currently depends on natural objective-HP
drain. Without a round-count limit, CI-style bounded tests have no guaranteed
exit point within a predictable tick count. Gap F from
`reports/PROMPT-1625-autoplay-recipe-coverage-gap-audit.md`.

---

## Solution

Smallest safe slice: an **opt-in env-var bound** (`CCGS_BOT_MAX_ROUNDS=N`) that
causes the server to emit `GameOverReason::MaxRoundsReached` after `N` completed
rounds. The bound is **off by default** — unset var, or value `0`, leaves the
server's normal behaviour completely untouched.

---

## Files Changed

| File | Change |
|---|---|
| `shared/src/protocol.rs` | Added `MaxRoundsReached` variant to `GameOverReason` enum |
| `server/src/feature/bot/soak_config.rs` | **New file** — `BotSoakConfig` resource + `BotSoakPlugin`, reads `CCGS_BOT_MAX_ROUNDS` at startup |
| `server/src/feature/bot/mod.rs` | `pub mod soak_config;` + re-export of `BotSoakConfig`, `BotSoakPlugin`, `BOT_MAX_ROUNDS_ENV_VAR` |
| `server/src/main.rs` | `app.add_plugins(feature::bot::BotSoakPlugin)` (after `BotDebugPushPlugin`) |
| `server/src/core/rsm/transitions.rs` | Import `BotSoakConfig`; added `soak_config: Option<Res<BotSoakConfig>>` param to `advance_phase`; max-rounds check injected in `Resolution` branch after `round_number += 1` |
| `tools/dev-launcher/Start-BotVsBotSoak.ps1` | Added `-MaxRounds N` param (default 0); sets `CCGS_BOT_MAX_ROUNDS` env var when `> 0`; shown in output and `soak-summary.json` |
| `server/Cargo.toml` | Registered `bot_soak_config_test` under `[[test]]` |
| `tests/unit/bot/bot_soak_config_test.rs` | **New file** — 6 unit tests covering resource insertion, from_env parsing for absent/zero/positive/whitespace/invalid values |

---

## Design Notes

### Env-var over CLI arg

The project convention (established by `CCGS_BOT_QA_SNAPSHOT`, `CCGS_BOT_DEBUG_UI`, etc.)
is to use env vars for server-side feature toggles. CLI args are not parsed today
(`main.rs` has no arg parsing). Following the existing pattern avoids introducing
a new parsing mechanism for a single soak-only knob.

### Opt-in guarantee

`BotSoakConfig::from_env()` returns `max_rounds: None` when:
- `CCGS_BOT_MAX_ROUNDS` is absent
- `CCGS_BOT_MAX_ROUNDS=0`
- The value is not a valid positive integer

The RSM check is `if let Some(max) = soak_config.as_deref().and_then(|c| c.max_rounds)`
— the entire path is skipped when `BotSoakConfig` is absent or `max_rounds` is `None`.

### Where the check fires

`advance_phase` → `RoundPhase::Resolution` branch → after `rsm.round_number += 1`.

The comparison is `round_number >= max` (not `==`), so the first round that equals or
exceeds the limit triggers GameOver even if the limit is set while a session is already
mid-run.

### `loser: None`

`MaxRoundsReached` is not a win/loss; both bots ran the full soak. `loser: None`
signals draw-like termination to downstream result screens.

---

## Validation

### `git diff --check`

Clean — no trailing whitespace errors.

### Unit tests (`bot_soak_config_test`)

Six tests:
- `test_bot_soak_config_default_is_disabled` — `Default` yields `None`
- `test_bot_soak_config_inserts_into_fresh_world` — `App` resource round-trip
- `test_from_env_absent_yields_none` — unset var → disabled
- `test_from_env_zero_yields_none` — `0` → disabled
- `test_from_env_positive_integer_activates_bound` — `10` → `Some(10)`
- `test_from_env_whitespace_trimmed` — `"  3  "` → `Some(3)`
- `test_from_env_non_integer_yields_none` — invalid string → disabled

**VERIFY command (focused, no broad workspace build):**
```
cd D:/_DEV/Work/ccgs-1640-bot-max-rounds
cargo test --test bot_soak_config_test -p server
```

### Broad Cargo

Not run per task rules. The `transitions.rs` change adds an `Option<Res<...>>`
param which Bevy handles as an optional system parameter — no `unwrap` path.

---

## Launcher Usage

```powershell
# Normal (no bound — original behaviour):
.\tools\dev-launcher\Start-BotVsBotSoak.ps1

# Bounded to 5 rounds then clean exit:
.\tools\dev-launcher\Start-BotVsBotSoak.ps1 -MaxRounds 5

# Bounded + shorter wall-clock ceiling:
.\tools\dev-launcher\Start-BotVsBotSoak.ps1 -MaxRounds 10 -DurationSeconds 120
```

---

## Remaining Live-GUI Limitations

- `MaxRoundsReached` has no dedicated client-side result-screen copy; it falls
  through to the draw/disconnection result path. A future prompt can add
  dedicated UI copy.
- The bound fires at round entry, not at round exit; a round that is already
  mid-way through Resolution when the limit is hit will still complete Resolution
  before exiting.

---

1640: BOT-VS-BOT-MAX-ROUNDS-BOUNDED-SOAK: SHIPPED
