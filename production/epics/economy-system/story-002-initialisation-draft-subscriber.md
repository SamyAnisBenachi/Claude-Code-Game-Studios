# Story 002: Initialisation & DraftStarted Subscriber

> **Epic**: Economy System
> **Status**: Complete
> **Layer**: Core
> **Type**: Logic
> **Manifest Version**: 2026-04-29

## Context

**GDD**: `design/gdd/economy-system.md`
**Requirement**: TR-??? (covers TR-ECO-02: mana ramp on DRAFT entry; TR-ECO-04: gold income on DRAFT entry; EC12: initialisation at SessionReady; EC15: round 1→2 interest trace)

**ADR Governing Implementation**: ADR-010: RSM Phase Event Bus — Phase Message Catalog and Subscriber Contracts
**ADR Decision Summary**: Economy System subscribes to `DraftStarted { round, phase: DraftPhase }` emitted by the RSM's `advance_phase`. The `phase` field selects the income formula: `DraftPhase::Initial` grants `starting_gold = 5` (already set at init — no income applied at round 1 DRAFT entry); `DraftPhase::Auction | Shop` grants `baseline + interest` (interest read from `InterestSnapshots`). Economy subscriber must be scheduled `.after(advance_phase)`. Economy emits `S2CGoldUpdate` unicast and `S2CGoldBroadcast` broadcast as internal events after each state mutation — the network dispatch system (Story 006) delivers them to clients.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: MEDIUM
**Engine Notes**: Uses `MessageReader<DraftStarted>::read()` (Bevy 0.18 — `EventReader` no longer exists) and `MessageWriter<S2CGoldUpdate>::write()` (Bevy 0.18 — `EventWriter` no longer exists). `initialise_player_economies` subscribes to `SessionReady` via `MessageReader<SessionReady>` — but see ADR-012: `SessionReady` is an Observer trigger. Correct approach: use `app.observe(init_player_economies)` instead. Both enforced by `liv-bevy-018`.

**Control Manifest Rules (Core layer)**:
- Required: `on_draft_started` scheduled `.after(advance_phase)` in `EconomyPlugin`'s `Update` set.
- Required: Round 1 (Initial): `starting_gold` already set at init; `on_draft_started` applies only mana ramp, enqueues updates. No gold is added on the first `DraftStarted`.
- Required: All `HashMap::get(player_id)` lookups use `.ok_or(SpendError::PlayerNotFound)` — no `unwrap()`.
- Forbidden: Economy must not read `GameConfig` timer fields — only income/mana config fields.
- Guardrail: `InterestSnapshots` entry is cleared immediately after consumption in `on_draft_started`; a missing entry is treated as `interest = 0` (first round, no prior snapshot).

---

## Acceptance Criteria

- [ ] `server/src/core/economy/system.rs` exists with `on_draft_started` system:
  - Reads `MessageReader<DraftStarted>`, `Res<SessionConfig>`, `ResMut<PlayerEconomies>`, `Res<InterestSnapshots>`, `Res<GameConfig>`, `MessageWriter<S2CGoldUpdate>`, `MessageWriter<S2CGoldBroadcast>` — TODO(liv-bevy-018): verify MessageWriter/MessageReader type names in Bevy 0.18
  - For each player in `SessionConfig.team_map.keys()`: calls `api::apply_mana_ramp(player, round)` setting `current_mana = min(round, mana_cap)`
  - If `phase == DraftPhase::Initial` AND `round == 1`: no gold added (starting gold already at 5 from init)
  - If `phase == DraftPhase::Auction | Shop`: reads `InterestSnapshots[player]` (default 0 if absent), computes `interest = min(floor(snap / interest_threshold_gold), interest_max_bonus)`, calls `api::apply_gold_award(player, gold_baseline_per_round + interest)`, clears snapshot entry
  - Enqueues `S2CGoldUpdate { player, gold, current_mana, reserve_mana, mana_cap }` per player
  - Enqueues `S2CGoldBroadcast { player, gold }` per player
- [ ] `server/src/core/economy/system.rs` contains `initialise_player_economies` Observer:
  - Registered via `app.observe(initialise_player_economies)` — NOT via `app.add_systems`. `SessionReady` is an Observer trigger (ADR-012); `MessageReader<SessionReady>` will never fire.
  - Receives `_trigger: Trigger<SessionReady>`, `Res<SessionConfig>`, `Res<GameConfig>`, `ResMut<PlayerEconomies>`
  - On `SessionReady`: for each player in `SessionConfig.team_map.keys()`: inserts `PlayerEconomy { gold: config.starting_gold, current_mana: 0, reserve_mana: 0, mana_cap: config.mana_cap, reserved_gold: 0 }`
  - `InterestSnapshots` resource is also inserted empty (`HashMap::new()`)
- [ ] `server/src/core/economy/plugin.rs` exists and defines `EconomyPlugin`:
  - Registers `PlayerEconomies` and `InterestSnapshots` resources (via `app.init_resource` or `app.insert_resource`)
  - Registers `S2CGoldUpdate` and `S2CGoldBroadcast` as Bevy messages (`app.add_message::<S2CGoldUpdate>()`, `app.add_message::<S2CGoldBroadcast>()`)
  - Registers `initialise_player_economies` via `app.observe(initialise_player_economies)` — NOT `app.add_systems` (ADR-012)
  - Schedules `on_draft_started` in `Update` `.after(advance_phase)`
- [ ] **EC12**: GIVEN game start, WHEN `initialise_player_economies` fires on `SessionReady`, THEN each player's `gold = 5`, `current_mana = 0`, `reserve_mana = 0`, `mana_cap = GameConfig.mana_cap`
- [ ] **EC6**: GIVEN `reserve_mana = 7` at RESOLUTION end, WHEN next `DraftStarted` fires (round 2), THEN `reserve_mana = 7` (unchanged), `current_mana = min(2, 10) = 2`
- [ ] **EC13**: GIVEN `InterestSnapshots[player] = 8`, WHEN `on_draft_started` fires (phase Auction/Shop), THEN interest computed as `floor(8/5) = 1`
- [ ] **EC15**: GIVEN `InterestSnapshots[player] = 8`, WHEN `on_draft_started` fires, THEN `gold = gold_before + 1 + 2` (interest + baseline = 3); for a player starting with 8g → `gold = 11`
- [ ] Round 1 guard: GIVEN `DraftStarted { round: 1, phase: Initial }`, WHEN `on_draft_started` fires, THEN `gold` is NOT increased (starting gold is not applied again); `current_mana = 1`
- [ ] Missing snapshot: GIVEN no entry in `InterestSnapshots` for a player, WHEN `on_draft_started` fires for a non-Initial phase, THEN `interest = 0` (treated as `snap = 0`)
- [ ] `S2CGoldUpdate` event is enqueued once per player per `DraftStarted` event processed
- [ ] `S2CGoldBroadcast` event is enqueued once per player per `DraftStarted` event processed
- [ ] `EconomyPlugin` registers cleanly in a headless Bevy `App::new()` startup test — `app.update()` does not panic

---

## Implementation Notes

*Derived from EPIC.md §system.rs `on_draft_started` and economy-system.md Rules 2, 3, 6:*

**`apply_mana_ramp` helper** (defined in `api.rs` or inline in `system.rs`):
```rust
pub fn apply_mana_ramp(economy: &mut PlayerEconomy, round: u32) {
    economy.current_mana = round.min(economy.mana_cap);
}
```
Note: mana ramp is applied regardless of `DraftPhase` — both Initial and Auction/Shop phases reset current_mana.

**Round 1 income guard:** The `DraftStarted { phase: Initial, round: 1 }` path applies ONLY mana ramp. The GDD specifies starting gold is granted during initialisation (`gold = 5` at `SessionReady`). Round 1's DRAFT entry must not add another `gold_baseline_per_round + interest` on top of the 5g — that would give 7g round 1 rather than 5g. Implement as:
```rust
match event.phase {
    DraftPhase::Initial => {
        // Mana ramp only. Gold already set to starting_gold at SessionReady.
    }
    DraftPhase::Auction | DraftPhase::Shop => {
        let snap = interest_snapshots.0.remove(&player).unwrap_or(0);
        let interest = (snap / config.interest_threshold_gold).min(config.interest_max_bonus);
        api::apply_gold_award(economy, config.gold_baseline_per_round + interest);
    }
}
```

**`S2CGoldUpdate` and `S2CGoldBroadcast`** are defined in `shared/src/protocol.rs` (workspace-and-shared-types epic). Import from there. If not yet defined, define temporary structs with a `// TODO: replace with shared::protocol` comment — this story's unit tests do not require the Lightyear send path (that is Story 006).

**`SessionConfig.team_map`**: A `HashMap<PlayerId, ClassId>` resource owned by the GSS epic. For M1 tests, construct a minimal `SessionConfig` with two entries and insert it manually into the test `World`.

**`InterestSnapshots.remove(player)`**: Use `HashMap::remove` (not `get` + manual delete) to atomically read and clear in one step. A `remove` returning `None` is the no-snapshot case — treat as 0.

---

## Out of Scope

- Story 003: `on_resolution_phase_entered` (interest snapshot write and mana discard)
- Story 004: Kill and objective award event handling
- Story 005: Auction reservation
- Story 006: Actual Lightyear network dispatch — this story enqueues internal Bevy events; Story 006 sends them via `MessageSender`
- `DraftStarted` event definition — owned by RSM epic (`server/core/rsm/events.rs`)
- `SessionReady` event definition — owned by RSM epic (emitted by GSS)

---

## QA Test Cases

*QL-STORY-READY skipped — Lean mode.*

- **EC12: Initialisation state**
  - Given: `World` with `SessionConfig` (2 players), `GameConfig` (defaults), `PlayerEconomies` empty
  - When: `SessionReady` event written + `initialise_player_economies` system run
  - Then: Both players present in `PlayerEconomies`; each has `gold = 5`, `current_mana = 0`, `reserve_mana = 0`, `mana_cap = 10`

- **Round 1 mana ramp, no gold increase**
  - Given: `PlayerEconomies` initialised (gold = 5); `DraftStarted { round: 1, phase: Initial }`
  - When: `on_draft_started` runs
  - Then: Each player's `current_mana = 1`; `gold` still 5 (not 7)

- **EC15: Round 2 income with interest**
  - Given: Player has `gold = 8`; `InterestSnapshots[player] = 8` (taken at R1 RESOLUTION end)
  - When: `DraftStarted { round: 2, phase: Shop }` processed by `on_draft_started`
  - Then: Player `gold = 11` (8 + 1 interest + 2 baseline); snapshot entry removed from map

- **Missing snapshot → interest = 0**
  - Given: `InterestSnapshots` empty; `DraftStarted { round: 2, phase: Shop }`
  - When: `on_draft_started` runs
  - Then: `gold += 0 + 2` (no interest, only baseline); no panic

---

## Test Evidence

**Story Type**: Logic
**Required evidence**:
- `tests/unit/economy/draft_subscriber_test.rs` — unit tests covering EC12, EC6, EC13, EC15, round-1 guard, missing-snapshot path
- `tests/integration/economy/round_trace_test.rs` — integration test simulating round 1 → 2 → 3 gold/mana trace; asserts `current_mana = min(R, 10)` at each DRAFT entry and `gold` evolves correctly through interest at each step; asserts `S2CGoldUpdate` enqueued per player per DRAFT entry
**Status**: [x] Created and passing in CI run `25167672501`

---

## Completion Notes

**Completed**: 2026-04-30
**Criteria**: 12/12 passing
**Deviations**: None blocking. The observer path uses Bevy 0.18 observer registration for `SessionReady`, and `on_draft_started` is scheduled after `advance_phase` now that S2-07 landed.
**Test Evidence**: Logic evidence at `tests/unit/economy/draft_subscriber_test.rs` and `tests/integration/economy/round_trace_test.rs`; runnable tests in `server/tests/economy_draft_subscriber_test.rs` and `server/tests/economy_round_trace_test.rs`, covered by `cargo test -p server` in CI run `25167672501`.
**Implementation Commits**: `9396d32` (S2-08 implementation), `e4ac84e` (restored files after S2-04 scope cleanup + doctest CI fix)
**Code Review**: Lean mode skipped; CI green.

---

## Dependencies

- Depends on: Story 001 (state types and API functions must exist)
- Depends on: RSM epic — `DraftStarted` event type defined in `server/core/rsm/events.rs`
- Depends on: GSS epic (S2-xx) — `SessionReady` event type and `SessionConfig` resource defined
- Unlocks: Story 006 (network dispatch needs `S2CGoldUpdate`/`S2CGoldBroadcast` events flowing)
