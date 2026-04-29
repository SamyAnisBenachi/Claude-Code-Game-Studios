# Unit Tests

Unit tests for isolated logic: formulas, state machines, data validation.

## In Rust/Cargo projects, unit tests live in:

1. **Inline `#[cfg(test)]` modules** — inside `src/` files, next to the code
2. **`[crate]/tests/*.rs` files** — Cargo integration tests (can also test single systems)

## Naming pattern: `[system]_[feature]_test.rs`

## Current test files

| File | System | Covers |
|---|---|---|
| `server/tests/rsm_formula_test.rs` | RSM | Formula F1 (is_auction_round), RSM-3/4/5 |
| `server/tests/game_config_defaults_test.rs` | GameConfig | GCN-DEFAULTS (all default values) |

## Add tests here as systems are implemented

Create `server/tests/[system]_test.rs` for each new system.
All Logic-type stories require a passing unit test before they can be marked Done.
