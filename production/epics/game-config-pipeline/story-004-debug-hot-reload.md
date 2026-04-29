# Story 004: Debug Hot-Reload & Release Verification

> **Epic**: GameConfig & CardCatalog Loading Pipeline
> **Status**: Ready
> **Layer**: Foundation
> **Type**: Integration
> **Manifest Version**: 2026-04-29

## Context

**GDD**: `design/gdd/game-config.md`
**Requirement**: TR-??? (covers TR-GC-05: hot-reload supported, re-validates, rejects invalid; TR-CDP-07: CardCatalog NOT hot-reloaded — immutable for server lifetime)

**ADR Governing Implementation**: ADR-004: Asset Loading Pipeline
**ADR Decision Summary**: A debug-only `hot_reload_game_config` system watches `AssetEvent::<GameConfig>::Modified`. On change, it re-runs `validate_game_config` — on pass, re-inserts `Res<GameConfig>`; on fail, logs a warning and retains the prior config. `CardCatalog` intentionally has no hot-reload path. The `add_systems` call itself (not just the function body) must be gated behind `#[cfg(debug_assertions)]` to guarantee the system is absent from release builds.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: MEDIUM
**Engine Notes**: `AssetEvent::<T>::Modified` — verify whether `AssetEvent` uses Bevy Message or Observer pattern in 0.18 (asset events may retain the old API or have migrated). If `AssetEvent` is still an ECS event, the pattern is `MessageReader<AssetEvent<GameConfig>>::read()`. Verify `AssetEvent` variant name against Bevy 0.18 migration guide. TODO(liv-bevy-018): verify `AssetEvent` API and `MessageReader` vs other pattern in Bevy 0.18. Release-build symbol verification requires `nm`, `cargo-bloat`, or equivalent.

**Control Manifest Rules (Foundation layer)**:
- Required: Debug hot-reload of `GameConfig` must re-validate before applying. Reject invalid reload with warning; retain previous config.
- Required: `CardCatalog` is immutable after load. Never hot-reload card definitions.
- Forbidden: Hot-reload watcher must NOT appear in release builds — gate the `add_systems` call itself behind `#[cfg(debug_assertions)]`.

---

## Acceptance Criteria

**Hot-reload implementation:**
- [ ] `hot_reload_game_config` system exists in `server/src/foundation/`
- [ ] System is gated: the `app.add_systems(...)` call adding this system is inside `#[cfg(debug_assertions)]` — not just the function body
- [ ] System runs in `Update` schedule when `in_state(AppState::Lobby).or(in_state(AppState::InSession))`
- [ ] On `AssetEvent::<GameConfig>::Modified` for the tracked handle: reads the new asset, calls `validate_game_config()`
- [ ] On validation pass: `commands.insert_resource(new_cfg.clone())`; logs `info!("GameConfig hot-reloaded successfully")`
- [ ] On validation fail: logs `warn!("GameConfig hot-reload rejected (kept previous): {reason}")`; prior `Res<GameConfig>` is retained unchanged
- [ ] No `hot_reload_card_catalog` system exists — `CardCatalog` is intentionally not hot-reloaded

**Release-build verification (explicit deliverable):**
- [ ] `cargo build -p server --release` completes successfully
- [ ] `hot_reload_game_config` function is NOT present in the release binary symbols (verified via `nm`, `cargo-bloat`, or `strings` on the binary)
- [ ] Verification result documented in `tests/evidence/story-gcp-004-release-verify.md` with the command used and its output

---

## Implementation Notes

*Derived from ADR-004 §Implementation Guidelines §6:*

**System implementation:**
```rust
#[cfg(debug_assertions)]
fn hot_reload_game_config(
    // TODO(liv-bevy-018): verify if AssetEvent uses MessageReader or a different pattern in Bevy 0.18
    mut events: MessageReader<AssetEvent<GameConfig>>,
    game_assets: Res<GameAssets>,
    configs: Res<Assets<GameConfig>>,
    mut commands: Commands,
) {
    for ev in events.read() {
        // Pattern match on Bevy 0.18 AssetEvent shape — verify variant name
        let AssetEvent::Modified { id } = ev else { continue; };
        if *id != game_assets.game_config.id() { continue; }

        let Some(new_cfg) = configs.get(&game_assets.game_config) else { continue; };
        match validate_game_config(new_cfg) {
            Ok(()) => {
                commands.insert_resource(new_cfg.clone());
                info!("GameConfig hot-reloaded successfully");
            }
            Err(e) => {
                warn!("GameConfig hot-reload rejected (kept previous): {e}");
            }
        }
    }
}
```

**`add_systems` gate in `server/main.rs`:**
```rust
fn main() {
    let mut app = App::new();
    // ... existing plugin setup ...
    #[cfg(debug_assertions)]
    app.add_systems(
        Update,
        hot_reload_game_config
            .run_if(in_state(AppState::Lobby).or(in_state(AppState::InSession))),
    );
    app.run();
}
```

Note: Gating the `add_systems` call (not just the function) ensures the system is not even scheduled in release builds, removing it from the Bevy system graph entirely. A function-only gate would still register a no-op system.

**Release binary symbol check commands:**
```bash
# Option 1: nm (Linux/Mac)
nm target/release/lanes-and-lies-server | grep hot_reload_game_config
# Expected: no output

# Option 2: cargo-bloat (install with: cargo install cargo-bloat)
cargo bloat -p server --release --crates | grep hot_reload
# Expected: not present in output

# Option 3: strings
strings target/release/lanes-and-lies-server | grep hot_reload_game_config
# Expected: no output
```

Any of the three methods is acceptable evidence. Document which was used and its output in `tests/evidence/story-gcp-004-release-verify.md`.

**Why CardCatalog is not hot-reloaded:** The `CardCatalog` is the source for `PlayerPool` initialization (Core layer). If the catalog were reloaded mid-session, in-progress `PlayerPool` instances would be inconsistent with the new catalog (cards that existed in the old catalog but not the new one would have dangling references in `copies_remaining`). Preventing hot-reload of `CardCatalog` is a correctness guarantee, not a limitation. Card data changes always require a server restart.

---

## Out of Scope

- Story 003: The `validate_game_config` function used here (already implemented)
- Story 002: The `GameAssets` resource and `AppState` used here (already implemented)

---

## QA Test Cases

*QL-STORY-READY skipped — Lean mode.*

- **AC: Valid hot-reload accepted**
  - Given: Server running in debug build; `GameConfig` loaded; a valid `game_config.ron` edit that changes `starting_gold` from 5 to 7
  - When: File is saved, triggering `AssetEvent::Modified`
  - Then: `info!("GameConfig hot-reloaded successfully")` is logged; `Res<GameConfig>.starting_gold == 7` in the next frame

- **AC: Invalid hot-reload rejected, prior config retained**
  - Given: Server running in debug build; `GameConfig` loaded with valid `shop_weight_cap: 0.65`
  - When: File is edited to set `shop_weight_cap: 1.5` and saved
  - Then: `warn!("GameConfig hot-reload rejected (kept previous): ...")` is logged; `Res<GameConfig>.shop_weight_cap` remains `0.65`

- **AC: Release build does not contain hot-reload system**
  - Given: `cargo build -p server --release` completes
  - When: Binary symbols are inspected for "hot_reload_game_config"
  - Then: No match found in symbol output

---

## Test Evidence

**Story Type**: Integration
**Required evidence**:
- Debug hot-reload test (manual or automated): log output showing accept + reject cases → `tests/evidence/story-gcp-004-hot-reload.md`
- Release build symbol check → `tests/evidence/story-gcp-004-release-verify.md`
**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 003 (validation gate — `validate_game_config` must exist)
- Unlocks: Epic `game-config-pipeline` **complete** → Epic 3 (server-rng) and Epic 4 (lightyear-protocol-verification) fully unblocked
