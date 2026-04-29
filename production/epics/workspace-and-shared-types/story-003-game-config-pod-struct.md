# Story 003: GameConfig POD Struct

> **Epic**: Workspace & Shared Types
> **Status**: Complete
> **Layer**: Foundation
> **Type**: Config/Data
> **Manifest Version**: 2026-04-29

## Context

**GDD**: `design/gdd/game-config.md`
**Requirement**: TR-??? (TR registry not yet populated — covers TR-GC-01: all tuning knobs in one external struct)
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-003: Cargo Workspace Structure (Rule 2 — GameConfig in shared/)
**ADR Decision Summary**: `GameConfig` lives in `shared/src/config.rs` as a plain serde struct without `#[derive(Resource)]`. The server wraps it at `server/foundation/config.rs` via `app.insert_resource(config)`. This keeps `shared/` free of `bevy_ecs` dependencies that test code does not need, and allows integration tests to construct `GameConfig` directly without a Bevy world.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: MEDIUM
**Engine Notes**: `GameConfig` in `shared/` intentionally omits `#[derive(Asset, TypePath)]` to preserve the lean-shared-crate constraint (ADR-003 Rule 1). ADR-004 shows these derives on the shared struct — this is an unresolved tension. Two valid resolution paths exist: (a) add the `bevy_asset` feature to `shared/Cargo.toml` as a deliberate, narrowly-scoped exception citing ADR-004; (b) create a server-side wrapper struct in `server/foundation/config.rs` that adds `Asset + TypePath` without touching `shared/`. **This decision must be made and documented in Epic 2 (game-config-pipeline).** Do not resolve it here — leave a `// TODO(Epic 2): Asset+TypePath decision` comment in the struct.

**Control Manifest Rules (Foundation layer)**:
- Required: `GameConfig` struct lives in `shared/config.rs` without `#[derive(Resource)]`. Server wraps it via `app.insert_resource(config)` in `server/foundation/config.rs`.
- Forbidden: Never derive `Resource`, `Asset`, or plugin-related traits in `shared/`.
- Guardrail: `GameConfig` + `CardCatalog` load time < 100ms total at expected card count.

---

## Acceptance Criteria

- [x] `shared/src/config.rs` exists with `GameConfig` struct
- [x] `GameConfig` has `#[derive(Serialize, Deserialize, Debug, Clone)]` + manual `Default` impl — no `Resource`, no `Asset`, no `TypePath`, no `Reflect`
- [x] All fields from `design/gdd/game-config.md` Section G (Tuning Knobs) are present with correct Rust types:
  - Pool: `common_pool_copies: u32`, `uncommon_pool_copies: u32`, `rare_pool_copies: u32`, `shop_weight_per_card: f32`, `shop_weight_cap: f32`
  - Economy: `starting_gold: u32`, `gold_baseline_per_round: u32`, `interest_threshold_gold: u32`, `interest_max_bonus: u32`, `objective_gold_reward: u32`, `kill_gold_reward: u32`, `mana_cap: u32`, `refresh_base_cost: u32`
  - Objectives: `objective_hp: u32`, `fake_count: u32`, `fake_objective_spawn_advance: u32`
  - Timers: `draft_initial_timer_seconds: u32`, `draft_shop_timer_seconds: u32`, `placement_timer_seconds: u32`, `resolution_max_duration_seconds: u32`, `disconnect_grace_seconds: u32`, `lobby_timeout_seconds: u32`, `lobby_heartbeat_timeout_seconds: u32`
  - Protocol: `protocol_version: u32`, `hello_timeout_ms: u32`
- [x] All fields use `#[serde(default)]` (applied at struct level) so missing fields in `.ron` fall back to `Default` values (per GDD Rule 1)
- [x] `Default` impl encodes design-intent defaults from GDD Section G (not all zeros)
- [x] A `TODO(Epic 2)` comment on the struct explains the `Asset+TypePath` decision deferral
- [x] `cargo check -p shared` — covered by CI gate in `tests.yml`; ADVISORY for Config/Data story type

---

## Implementation Notes

*Derived from ADR-003 Rule 2 and game-config.md Rules 1–2:*

**`#[serde(default)]` on every field:** This allows `game_config.ron` files to omit fields — missing fields silently fall back to the `Default` impl. This is critical for forward compatibility: adding a new field to `GameConfig` does not break existing `.ron` files. The `Default` impl must encode real design-intent values (e.g. `starting_gold: 3`, `fake_count: 2`), not `0`.

**Timer fields use `_seconds` suffix:** RSM phase duration fields use integer seconds (e.g. `draft_initial_timer_seconds`). Network protocol timeout fields retain `_ms` suffix (e.g. `hello_timeout_ms`, `ack_timeout_ms`, `heartbeat_interval_ms`) as they are millisecond-precision network values. This matches the canonical `game-config.md` GDD naming — accepted as Option A in the cross-review of 2026-04-29.

**Why no `Resource` derive:** If `#[derive(Resource)]` is added to `shared/`, then `shared/` must pull `bevy_ecs` as a dependency — which breaks the lean-shared-crate constraint and would add ~20 MB to the WASM bundle. The server inserts it manually: `app.insert_resource(game_config_value)`. Any `T: Send + Sync + 'static` can be inserted as a resource without deriving `Resource` in Bevy 0.18. If a future Bevy version requires the derive, add it at that point.

**Default values (from GDD Section G — canonical):**
```rust
impl Default for GameConfig {
    fn default() -> Self {
        Self {
            common_pool_copies: 6,
            uncommon_pool_copies: 5,
            rare_pool_copies: 4,
            shop_weight_per_card: 0.10,
            shop_weight_cap: 0.65,
            starting_gold: 5,
            gold_baseline_per_round: 2,
            interest_threshold_gold: 5,
            interest_max_bonus: 2,
            objective_gold_reward: 3,
            kill_gold_reward: 1,
            mana_cap: 10,
            refresh_base_cost: 1,
            objective_hp: 5,
            fake_count: 2,
            fake_objective_spawn_advance: 1,
            draft_initial_timer_seconds: 45,
            draft_shop_timer_seconds: 30,
            placement_timer_seconds: 10,
            resolution_max_duration_seconds: 60,
            disconnect_grace_seconds: 30,
            lobby_timeout_seconds: 90,
            lobby_heartbeat_timeout_seconds: 15,
            protocol_version: 1,
            hello_timeout_ms: 5000,
        }
    }
}
```

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- Story 002: Card types in `shared/src/card.rs`
- Story 004: CI dependency-tree gates
- Epic 2 (game-config-pipeline): Asset loading, validation, `Asset+TypePath` resolution, `game_config.ron` file, `Res<GameConfig>` insertion

---

## QA Test Cases

*QL-STORY-READY skipped — Lean mode. Basic checks below.*

- **AC: All GDD Section G fields present**
  - Given: `shared/src/config.rs` as implemented
  - When: `GameConfig::default()` is constructed
  - Then: No compile errors; all field names match GDD Section G naming (RSM timer fields use `_seconds` suffix; network timeout fields use `_ms` suffix)
  - Edge cases: Verify `fake_count` default is `2` (not `0`); verify `shop_weight_cap` default is in `(0.0, 1.0)` range per dangerous-value rules

- **AC: `#[serde(default)]` round-trip with missing field**
  - Given: A RON or JSON string that omits one field (e.g. `refresh_base_cost` absent)
  - When: Deserialised into `GameConfig`
  - Then: The missing field takes the `Default` value (e.g. `refresh_base_cost = 2`), no deserialisation error

- **AC: No `Resource`/`Asset` derive leaks into shared/**
  - Given: `shared/src/config.rs` as written
  - When: `cargo tree -p shared` is inspected (Story 004 gate)
  - Then: `bevy_ecs` does not appear in the dependency tree

---

## Test Evidence

**Story Type**: Config/Data
**Required evidence**: Smoke check — `cargo check -p shared` output showing zero warnings — paste into `tests/evidence/story-003-game-config-check.md`
**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 (workspace scaffolding) must be Done
- Unlocks: Story 004 (protocol skeleton + CI gates); Epic 2 (game-config-pipeline) stories
