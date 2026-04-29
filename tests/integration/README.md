# Integration Tests

Cross-system tests: multiple Bevy systems interacting, save/load round-trips,
multi-phase RSM sequences.

## When to write integration tests

- A story's acceptance criterion requires multiple systems to cooperate
- Testing a full RSM phase transition (Economy + Pool + RSM together)
- Testing reconnect flow (GSS + RSM + Snapshot delivery)

## Pattern (minimal Bevy App)

```rust
#[test]
fn test_draft_entry_applies_mana_ramp() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(GameConfig::default());
    // add only the plugins needed for this test
    app.add_plugins(EconomyPlugin);
    app.add_plugins(RsmPlugin);

    // Set up initial state
    // Run one update
    app.update();
    // Assert outcome
}
```

## Current integration test files

(None yet — add as M1 systems are implemented)

## Naming: `[system_a]_[system_b]_integration_test.rs`

Example: `rsm_economy_integration_test.rs` for RSM + Economy cross-system test.
