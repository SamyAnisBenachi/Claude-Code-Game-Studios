# Story 003: Startup Validation Gate

> **Epic**: GameConfig & CardCatalog Loading Pipeline
> **Status**: Complete
> **Layer**: Foundation
> **Type**: Logic
> **Manifest Version**: 2026-04-30

## Context

**GDD**: `design/gdd/game-config.md`
**Requirement**: TR-??? (covers TR-GC-03: fatal on failure; TR-GC-04: all dangerous-value checks; TR-CDP-02: duplicate CardIds fatal; TR-CDP-09: soft error for override ≤ 0)

**ADR Governing Implementation**: ADR-004: Asset Loading Pipeline
**ADR Decision Summary**: `validate_and_promote` runs once on entry to `AppState::ConfigValidation`. On any invariant failure, it calls `exit.write(AppExit::error())` — never `panic!`. On full success, it clones both assets into resources and transitions to `AppState::Lobby`. The distinction between fatal errors and soft errors (warn + continue) is explicitly defined.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: LOW
**Engine Notes**: `AppExit::error()` is the correct non-panicking fatal exit in Bevy 0.18. In Bevy 0.18, `AppExit` is an Observer event — send via `commands.trigger(AppExit::error())` or check if `EventWriter<AppExit>` still exists (post-cutoff, verify against `liv-bevy-018`). Verify `AppExit` import path: `bevy::app::AppExit` or `bevy::prelude::AppExit`. TODO(liv-bevy-018): verify AppExit dispatch mechanism in Bevy 0.18.

**Control Manifest Rules (Foundation layer)**:
- Required: Abort startup if any dangerous `GameConfig` value is invalid (full list in ACs below).
- Required: `pool_copies_override ≤ 0` is a soft error — log warning, use rarity default, continue.
- Required: `CardCatalog` is immutable after load. Fatal on duplicate CardIds.

---

## Acceptance Criteria

**`validate_game_config()` — all 10 invariants (fatal on violation):**
- [x] `shop_weight_cap > 0.0` — fails: "shop_weight_cap must be > 0.0"
- [x] `shop_weight_cap < 1.0` — fails: "shop_weight_cap must be < 1.0"
- [x] `shop_weight_per_card < shop_weight_cap` — fails: "shop_weight_per_card must be < shop_weight_cap"
- [x] `common_pool_copies >= 1` — fails: "common_pool_copies must be >= 1"
- [x] `uncommon_pool_copies >= 1` — fails: "uncommon_pool_copies must be >= 1"
- [x] `rare_pool_copies >= 1` — fails: "rare_pool_copies must be >= 1"
- [x] `fake_count >= 1` — fails: "fake_count must be >= 1 — the bluffing mechanic is a load-bearing design pillar"
- [x] `fake_count <= 3` — fails: "fake_count must be <= 3"
- [x] `objective_hp >= 1` — fails: "objective_hp must be >= 1"
- [x] `placement_timer_ms >= 1` (or `placement_timer_seconds >= 1` depending on field naming) — fails with descriptive message
- [x] `auction_timer_ms >= 1` — fails with descriptive message
- [x] `auction_timer_reset_ms < auction_timer_ms` — fails: "auction_timer_reset must be < auction_timer"

**`validate_card_catalog()` — fatal errors:**
- [x] Empty catalog (`cards.len() == 0`) → fatal: "CardCatalog is empty — no cards to draft"
- [x] Key-mismatch: `map_key != card.id` for any entry → fatal with both values logged

**Soft error:**
- [x] Card with `pool_copies_override: Some(-1)` → `warn!` log line referencing card ID and value; server continues; card uses rarity-default copy count

**`validate_and_promote` system:**
- [x] Replaces the stub from Story 002 with the real implementation
- [x] On ANY validation failure: calls `exit.write(AppExit::error())` — **never `panic!`**
- [x] On full success: `commands.insert_resource(cfg.clone())`, `commands.insert_resource(cat.clone())`, `next_state.set(AppState::Lobby)`
- [x] Error log on failure includes a human-readable description of which check failed and the offending value

**Unit tests (all passing, in `tests/unit/foundation/` or `server/tests/`):**
- [x] One passing case for `validate_game_config` (all 10 invariants satisfied)
- [x] One failing case per dangerous-value check (minimum: `shop_weight_cap = 1.5`, `fake_count = 0`, `fake_count = 4`, `objective_hp = 0`, `placement_timer = 0`, `auction_timer_reset >= auction_timer`)
- [x] `validate_card_catalog`: passing case (valid catalog), failing case (empty), failing case (key-mismatch)
- [x] Soft-error case: `pool_copies_override = -1` does not cause `validate_card_catalog` to return `Err` (this check lives in pool init, not catalog validation — confirm correct placement)

---

## Implementation Notes

*Derived from ADR-004 §Implementation Guidelines §3–5:*

**`validate_game_config` full implementation:**
```rust
pub fn validate_game_config(c: &GameConfig) -> Result<(), String> {
    if !(c.shop_weight_cap > 0.0) {
        return Err(format!("shop_weight_cap must be > 0.0; got {}", c.shop_weight_cap));
    }
    if !(c.shop_weight_cap < 1.0) {
        return Err(format!("shop_weight_cap must be < 1.0; got {}", c.shop_weight_cap));
    }
    if !(c.shop_weight_per_card < c.shop_weight_cap) {
        return Err(format!(
            "shop_weight_per_card ({}) must be < shop_weight_cap ({})",
            c.shop_weight_per_card, c.shop_weight_cap
        ));
    }
    if c.common_pool_copies < 1 { return Err("common_pool_copies must be >= 1".into()); }
    if c.uncommon_pool_copies < 1 { return Err("uncommon_pool_copies must be >= 1".into()); }
    if c.rare_pool_copies < 1 { return Err("rare_pool_copies must be >= 1".into()); }
    if c.fake_count < 1 {
        return Err("fake_count must be >= 1 — the bluffing mechanic is a load-bearing design pillar".into());
    }
    if c.fake_count > 3 { return Err(format!("fake_count must be <= 3; got {}", c.fake_count)); }
    if c.objective_hp < 1 { return Err("objective_hp must be >= 1".into()); }
    // Timer field names: adjust to match actual GameConfig field names (ms vs seconds)
    if c.placement_timer_ms < 1 { return Err("placement_timer must be >= 1ms".into()); }
    if c.auction_timer_ms < 1 { return Err("auction_timer must be >= 1ms".into()); }
    if c.auction_timer_reset_ms >= c.auction_timer_ms {
        return Err(format!(
            "auction_timer_reset_ms ({}) must be < auction_timer_ms ({})",
            c.auction_timer_reset_ms, c.auction_timer_ms
        ));
    }
    Ok(())
}
```

**`validate_card_catalog`:**
```rust
pub fn validate_card_catalog(c: &CardCatalog) -> Result<(), String> {
    if c.is_empty() {
        return Err("CardCatalog is empty — no cards to draft".into());
    }
    for (key, card) in c {
        if key != &card.id {
            return Err(format!(
                "CardCatalog key '{:?}' does not match CardData.id '{:?}'",
                key, card.id
            ));
        }
    }
    Ok(())
}
```

**`pool_copies_override` soft error placement:** The soft error for `override <= 0` is NOT in `validate_card_catalog` — that function only validates catalog structure. The soft error fires during `PlayerPool::initialize()` (Core layer). What this story tests is that `validate_card_catalog` does NOT reject a card with `pool_copies_override: Some(-1)` — the catalog is structurally valid; the override is a runtime concern for the pool.

**`AppExit::error()` not `panic!`:** Using `panic!` in a Bevy system causes the thread to unwind in a way that may not produce a clean non-zero exit on all platforms. `AppExit::error()` lets Bevy shut down cleanly, guarantees a non-zero exit code for Railway/Docker health checks, and produces a final log flush. Always use this path for fatal startup errors.

---

## Out of Scope

- Story 002: The loading pipeline wiring (already done — this story replaces the stub)
- Story 004: Hot-reload re-validation (same validation functions, called from a different context)
- Core epic (Card Data & Pool): `pool_copies_override` soft error during pool init — NOT here

---

## QA Test Cases

*QL-STORY-READY skipped — Lean mode.*

- **AC: shop_weight_cap = 1.5 → fatal**
  - Given: `GameConfig { shop_weight_cap: 1.5, ..Default::default() }`
  - When: `validate_game_config(&config)` is called
  - Then: Returns `Err(msg)` where `msg` contains "shop_weight_cap" and "1.5"

- **AC: fake_count = 0 → fatal with design-pillar message**
  - Given: `GameConfig { fake_count: 0, ..Default::default() }`
  - When: `validate_game_config(&config)` is called
  - Then: Returns `Err(msg)` where `msg` contains "load-bearing design pillar"

- **AC: auction_timer_reset >= auction_timer → fatal**
  - Given: `GameConfig { auction_timer_ms: 20_000, auction_timer_reset_ms: 20_000, ..Default::default() }`
  - When: `validate_game_config(&config)` is called
  - Then: Returns `Err(msg)` containing both values

- **AC: empty catalog → fatal**
  - Given: Empty `CardCatalog` (zero entries)
  - When: `validate_card_catalog(&catalog)` is called
  - Then: Returns `Err(msg)` containing "empty"

- **AC: card with pool_copies_override = -1 → catalog validates OK**
  - Given: `CardCatalog` with one card having `pool_copies_override: Some(-1)`
  - When: `validate_card_catalog(&catalog)` is called
  - Then: Returns `Ok(())` — the override is not a catalog-level fatal error

- **AC: validate_game_config default → passes**
  - Given: `GameConfig::default()`
  - When: `validate_game_config(&config)` is called
  - Then: Returns `Ok(())`

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/foundation/game_config_validation_test.rs` — all test cases passing
**Status**: [x] Created and mapped to embedded unit tests in `server/src/foundation/config.rs`

---

## Dependencies

- Depends on: Story 002 (loading pipeline stub must exist to replace)
- Unlocks: Story 004 (hot-reload uses the same validation functions)

---

## Completion Notes

**Completed**: 2026-04-30
**Criteria**: 22/22 passing
**Deviations**: None. Bevy 0.18 implementation uses `MessageWriter<AppExit>` / `AppExit::error()` per current control-manifest guidance.
**Test Evidence**: Logic evidence at `tests/unit/foundation/game_config_validation_test.rs`; runnable tests are embedded in `server/src/foundation/config.rs` and covered by CI run `25176947506`.
**Code Review**: Skipped (lean mode).
**Local Verification**: Attempted `C:\Users\Sam\.cargo\bin\cargo.exe test -p server game_config`; normal PowerShell failed before story tests ran due Windows resource/toolchain metadata errors (`libtest` paging-file mmap / invalid metadata cascade). CI run `25176947506` is green on `main`.
