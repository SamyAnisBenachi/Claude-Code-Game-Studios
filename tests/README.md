# Test Infrastructure — Lanes and Lies

**Engine**: Bevy 0.18 (Rust/Cargo)
**Test Framework**: Rust built-in `#[test]` + Bevy `World::new()` for ECS tests
**CI**: `.github/workflows/tests.yml` — runs on every push to `main` and every PR
**Setup date**: 2026-04-29

## Directory Layout

```
tests/
  unit/           # Documentation — actual tests live in each crate's tests/ dir
  integration/    # Documentation — actual tests live in each crate's tests/ dir
  smoke/          # Critical path test list for /smoke-check gate
  evidence/       # Screenshot logs and manual test sign-off records
```

## Where Tests Actually Live (Rust/Cargo)

In Rust, tests live close to the code they test:

```
server/
  src/            # #[cfg(test)] modules for inline unit tests
  tests/          # Cargo integration tests (run with cargo test -p server)
    rsm_formula_test.rs        ← RSM Formula F1 — already written
    game_config_defaults_test.rs ← GCN-DEFAULTS — already written

shared/
  src/            # #[cfg(test)] modules for shared type tests
```

## Running Tests

```bash
# Run all tests in the workspace
cargo test --workspace

# Run only server tests
cargo test -p server

# Run a specific test
cargo test -p server test_round_3_is_auction_round

# Run with output (see println! in tests)
cargo test --workspace -- --nocapture
```

## Test Naming Conventions

| Element | Convention | Example |
|---|---|---|
| Test files | `[system]_[feature]_test.rs` | `rsm_formula_test.rs` |
| Test functions | `test_[scenario]_[expected]` | `test_round_3_is_auction_round` |
| Test modules (inline) | `mod tests { ... }` | standard Rust |

## Story Type → Test Evidence Required

| Story Type | Required Evidence | Location | Gate |
|---|---|---|---|
| **Logic** | Automated test — must pass | `[crate]/tests/unit/[system]/` or inline | BLOCKING |
| **Integration** | Integration test OR documented playtest | `[crate]/tests/integration/[system]/` | BLOCKING |
| **Visual/Feel** | Screenshot + lead sign-off | `tests/evidence/` | ADVISORY |
| **UI** | Manual walkthrough OR interaction test | `tests/evidence/` | ADVISORY |
| **Config/Data** | Smoke check pass | `production/qa/smoke-*.md` | ADVISORY |

## Writing Bevy ECS Tests (World-based)

Per `liv-bevy-018` skill and `docs/engine-reference/bevy/current-best-practices.md`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn test_my_system_does_thing() {
        let mut world = World::new();
        // Insert resources needed by the system
        world.insert_resource(GameConfig::default());
        // Run the system directly
        // Assert expected state
    }
}
```

**No full `App` needed for unit tests.** Use `App` only for integration tests
that require the full scheduler.

## CI

Tests run on every push to `main` and every pull request.
A failed test suite blocks merging — see `.github/workflows/tests.yml`.
