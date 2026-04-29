# Story 001: Asset Data Files

> **Epic**: GameConfig & CardCatalog Loading Pipeline
> **Status**: Complete
> **Layer**: Foundation
> **Type**: Config/Data
> **Manifest Version**: 2026-04-29

## Context

**GDD**: `design/gdd/game-config.md` · `design/gdd/card-data-pool.md`
**Requirement**: TR-GC-01, TR-CDP-01 (TR-GC-01: all tuning knobs in external file; TR-CDP-01: card data in external JSON)

**ADR Governing Implementation**: ADR-004: Asset Loading Pipeline
**ADR Decision Summary**: `game_config.ron` is the single source of all tuning knobs — no hardcoded values in systems. `cards.json` is the card catalog loaded at startup. Both files are bundled with the server binary. Load failure is always fatal.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: MEDIUM
**Engine Notes**: RON is no longer re-exported via `bevy_asset` in 0.18 — `ron = "0.8"` must be a direct dep in `server/Cargo.toml`. The RON file format must be valid for the `ron` 0.8 parser specifically.

**Control Manifest Rules (Foundation layer)**:
- Required: Load `game_config.ron` via `bevy_asset_loader` at server startup; fatal on missing or malformed file.
- Required: `CardCatalog` is immutable after load. Card data changes require server restart.
- Forbidden: No hardcoded balance values in systems — all tuning knobs go through `GameConfig`.

---

## Acceptance Criteria

- [ ] `assets/config/game_config.ron` exists and is valid RON parseable by `ron 0.8`
- [ ] Every field from `design/gdd/game-config.md` Section G (Tuning Knobs) is present with its design-intent default value (see Implementation Notes for complete list)
- [ ] Timer fields use seconds suffix matching the GDD field names (e.g. `placement_timer_seconds: 10`)
- [ ] `assets/data/cards.json` exists and is valid JSON parseable by `serde_json`
- [ ] `cards.json` fixture contains at minimum: one card of each rarity (`Common`, `Uncommon`, `Rare`, `Epic`, `Legendary`), cards from at least 2 different `ClassId` values, one `Neutral` card with a `family` field set, one card with a valid `pool_copies_override` (≥ 1), one card with `pool_copies_override: -1` (for soft-error testing), one card with `pool_copies_override: null` (uses rarity default)
- [ ] All cards in `cards.json` have unique `id` values — no duplicates
- [ ] All `cards.json` entries include all required base fields: `id`, `name_fr`, `name_en`, `class`, `rarity`, `card_type`, `unit_type`, `cost`, `atk`, `hp`, `mp`, `ar`, `keywords`, `effect_text`, `art_id`
- [ ] `cargo check --workspace` still passes after adding these files (no code change — files are data only)

---

## Implementation Notes

*Derived from ADR-004 §5 and game-config.md Section G:*

**`assets/config/game_config.ron` design-intent defaults:**
```ron
(
    // Pool
    common_pool_copies: 6,
    uncommon_pool_copies: 5,
    rare_pool_copies: 4,
    shop_weight_per_card: 0.10,
    shop_weight_cap: 0.65,

    // Economy
    starting_gold: 5,
    gold_baseline_per_round: 2,
    interest_threshold_gold: 5,
    interest_max_bonus: 2,
    objective_gold_reward: 3,
    kill_gold_reward: 1,
    mana_cap: 10,
    refresh_base_cost: 1,

    // Objectives
    objective_hp: 5,
    fake_count: 2,
    fake_objective_spawn_advance: 1,

    // Timers — RSM
    draft_initial_timer_seconds: 45,
    draft_shop_timer_seconds: 30,
    placement_timer_seconds: 10,
    resolution_max_duration_seconds: 60,
    disconnect_grace_seconds: 30,
    lobby_timeout_seconds: 90,
    lobby_heartbeat_timeout_seconds: 15,

    // Timers — Auction
    auction_timer_seconds: 20,
    auction_timer_reset_seconds: 5,
    auction_max_duration_seconds: 120,

    // Class mechanics
    xelor_sablier_steal: 1,

    // Network
    protocol_version: 1,
    hello_timeout_ms: 10000,
    ack_timeout_ms: 5000,
    heartbeat_interval_ms: 3000,
)
```

**`assets/data/cards.json` fixture structure:**
Use JSON array form: `[{ "id": 1, "name_fr": "...", ... }, ...]`. The fixture uses array form `[{...}]`; ADR-004 §5 note on array form applies — duplicate CardId check in AC 6 is explicit per-element validation, not HashMap key deduplication. Each entry must include all `CardData` fields defined in `shared/src/card.rs` (Story 002 of workspace-and-shared-types epic). For the fixture, `keywords` can be `[]` on most cards; `art_id` can be a placeholder string like `"placeholder_iop_001"`. Stat fields (`atk`, `hp`, `mp`, `ar`) should be non-zero only on `Minion` and `Structure` types.

**Timer field naming:** The GDD Section G uses `_seconds` suffixes for all timer fields. The `shared/src/config.rs` GameConfig struct (Story 003 of workspace-and-shared-types) uses `_ms` suffixes per the network-protocol.md correction. **Verify the field names match what was implemented in shared/src/config.rs before authoring this RON file.** If there is a mismatch, the serde `#[serde(default)]` will silently ignore unrecognised fields — catching this early is better than discovering it during hot-reload testing.

---

## Out of Scope

- Story 002: Implementing the Rust loading pipeline that reads these files
- Story 003: Validation logic that enforces Rule 5 invariants
- Epic 1 Story 003: The `GameConfig` Rust struct definition

---

## QA Test Cases

*QL-STORY-READY skipped — Lean mode.*

- **AC: game_config.ron parses**
  - Given: `assets/config/game_config.ron` as written
  - When: `ron::from_str::<GameConfig>(contents)` is called in a unit test
  - Then: Returns `Ok(config)` with no error; `config.fake_count == 2`, `config.objective_hp == 5`, `config.shop_weight_cap == 0.65`

- **AC: cards.json parses and contains expected fixture entries**
  - Given: `assets/data/cards.json` as written
  - When: `serde_json::from_str::<Vec<CardData>>(contents)` is called
  - Then: Returns `Ok(cards)`; `cards.len() >= 5`; contains one card per rarity level; no duplicate IDs; the card with `pool_copies_override: -1` deserialises correctly as `Some(-1)`

---

## Test Evidence

**Story Type**: Config/Data
**Required evidence**: Smoke check pass — `cargo check --workspace` output showing zero warnings after adding files → `tests/evidence/story-gcp-001-data-files.md`
**Status**: [x] `tests/evidence/story-gcp-001-data-files.md` — PASS (2026-04-29)

---

## Dependencies

- Depends on: `workspace-and-shared-types` Story 003 (GameConfig POD struct must be defined before RON field names can be verified)
- Unlocks: Story 002 (asset loading pipeline)
