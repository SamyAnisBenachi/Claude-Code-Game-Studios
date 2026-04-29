# Story 001: State & Pure API Scaffold

> **Epic**: Economy System
> **Status**: Ready
> **Layer**: Core
> **Type**: Logic
> **Manifest Version**: 2026-04-29

## Context

**GDD**: `design/gdd/economy-system.md`
**Requirement**: TR-??? (covers TR-ECO-01: three independent currency pools; TR-ECO-06: spend validation + auto-split; TR-ECO-07: reserve-only restriction; TR-ECO-03: mana cap increment + ceiling)

**ADR Governing Implementation**: ADR-010: RSM Phase Event Bus — Phase Message Catalog and Subscriber Contracts
**ADR Decision Summary**: `PlayerEconomy` is mutated ONLY through the single-writer API in `economy/api.rs`. The module is accessed as a `PlayerEconomies(HashMap<PlayerId, PlayerEconomy>)` resource. All u32 arithmetic uses `saturating_sub` to prevent underflow. `reserved_gold` never exceeds `gold` (debug-mode panic). `mana_cap` never exceeds `GameConfig.mana_cap_max` (default 12).

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: LOW
**Engine Notes**: `state.rs` and `api.rs` are pure Rust — no Bevy ECS surface except `#[derive(Resource)]` on `PlayerEconomies` and `InterestSnapshots`. No post-cutoff API risk on pure data types. `liv-bevy-018` is still mandatory on this file because `#[derive(Resource)]` requires correct Bevy 0.18 import path (`bevy::prelude::Resource`).

**Control Manifest Rules (Core layer)**:
- Required: `PlayerEconomy` struct fields are all `u32`. Newtype wrapper not needed — `PlayerId` from `shared/` disambiguates map keys.
- Required: `PlayerEconomies` and `InterestSnapshots` are the only `Resource` types defined in this story. `PlayerEconomy` itself is NOT a `Resource`.
- Required: All spend/award mutations go through `economy/api.rs` functions. Direct field assignment outside `economy/api.rs` is forbidden and enforced by CI grep gate.
- Forbidden: No `unwrap()` in production paths — all `HashMap::get` calls use `.ok_or(SpendError::PlayerNotFound)` propagation.
- Forbidden: No `checked_sub` — use `saturating_sub` throughout to prevent u32 underflow panics.
- Guardrail: `total_effective_mana(economy)` = `current_mana + reserve_mana`. This is a read-only helper, never a mutation.

---

## Acceptance Criteria

- [ ] `server/src/core/economy/state.rs` exists and defines:
  - `PlayerEconomy { gold: u32, current_mana: u32, reserve_mana: u32, mana_cap: u32, reserved_gold: u32 }` with `#[derive(Clone, Debug)]`
  - `PlayerEconomies(pub HashMap<PlayerId, PlayerEconomy>)` with `#[derive(Resource)]`
  - `InterestSnapshots(pub HashMap<PlayerId, u32>)` with `#[derive(Resource)]`
  - `SpendError` enum with variants: `InsufficientFunds`, `ReserveOnlyButCurrentProvided`, `HandFull`, `WrongPhase`, `PlayerNotFound`
- [ ] `server/src/core/economy/api.rs` exists and exports all 10 pure API functions:
  - `validate_spend(economy: &PlayerEconomy, cost: u32, from_reserve_only: bool) -> Result<(), SpendError>`
  - `apply_spend(economy: &mut PlayerEconomy, cost: u32, from_reserve_only: bool)` — auto-split current-first if `from_reserve_only == false`; pure reserve deduction otherwise
  - `apply_gold_award(economy: &mut PlayerEconomy, amount: u32)`
  - `add_reserve(economy: &mut PlayerEconomy, amount: u32)` — no cap
  - `reserve_gold(economy: &mut PlayerEconomy, amount: u32) -> Result<(), SpendError>`
  - `release_gold_reservation(economy: &mut PlayerEconomy, amount: u32)`
  - `discard_current_mana(economy: &mut PlayerEconomy)` — sets `current_mana = 0`
  - `increment_mana_cap(economy: &mut PlayerEconomy, config: &GameConfig)` — saturates at `config.mana_cap_max` (12)
  - `can_afford_bid(economy: &PlayerEconomy, amount: u32) -> bool` — `(gold - reserved_gold) >= amount`
  - `can_afford_shop(economy: &PlayerEconomy, cost: u32) -> bool` — `(gold - reserved_gold) >= cost`
  - `total_effective_mana(economy: &PlayerEconomy) -> u32` — `current_mana + reserve_mana`
- [ ] **EC1**: GIVEN `current_mana = 2`, `reserve_mana = 3`, WHEN `validate_spend` and `apply_spend` called for cost 4 (`from_reserve_only = false`), THEN `current_mana = 0`, `reserve_mana = 1`
- [ ] **EC2**: GIVEN `current_mana = 0`, `reserve_mana = 5`, WHEN `apply_spend` for cost 3 (`from_reserve_only = false`), THEN `current_mana = 0`, `reserve_mana = 2`
- [ ] **EC3**: GIVEN `current_mana = 4`, `reserve_mana = 2`, WHEN `apply_spend` for cost 4 (`from_reserve_only = false`), THEN `current_mana = 0`, `reserve_mana = 2` (reserve untouched)
- [ ] **EC4**: GIVEN `current_mana = 1`, `reserve_mana = 1`, WHEN `validate_spend` for cost 3, THEN returns `Err(SpendError::InsufficientFunds)`; neither pool modified
- [ ] **EC5**: GIVEN `reserve_mana = 3`, `current_mana = 10`, WHEN `validate_spend` for cost 4 with `from_reserve_only = true`, THEN returns `Err(SpendError::InsufficientFunds)` (reserve insufficient; current does not substitute)
- [ ] **EC7**: GIVEN `current_mana = 5`, WHEN `add_reserve(economy, 5)` then `discard_current_mana(economy)`, THEN `reserve_mana` increases by 5 and `current_mana = 0` (Gelure contract tested via API primitives)
- [ ] **EC8**: GIVEN `current_mana = 0`, WHEN `add_reserve(economy, 0)` then `discard_current_mana(economy)`, THEN `reserve_mana` unchanged, no error (legal no-op)
- [ ] **EC9**: GIVEN `mana_cap = 10`, WHEN `increment_mana_cap(economy, config)` called, THEN `mana_cap = 11`
- [ ] **EC10**: GIVEN `mana_cap = 12`, WHEN `increment_mana_cap(economy, config)` called, THEN `mana_cap` remains 12
- [ ] **EC11 (pure API half)**: GIVEN any `PlayerEconomy`, WHEN `apply_gold_award(economy, 0)` is called (simulating self-inflicted guard — the caller passes 0), THEN `gold` is unchanged (zero-amount award is a no-op)
- [ ] Zero-cost card: GIVEN `current_mana = 3`, `reserve_mana = 2`, WHEN `apply_spend` for cost 0, THEN neither pool modified, returns `Ok(())`
- [ ] `reserve_gold` returns `Err(SpendError::InsufficientFunds)` when `amount > gold - reserved_gold`
- [ ] `release_gold_reservation` uses `saturating_sub` — releasing more than reserved clamps to 0, no panic
- [ ] CI grep gate: `grep -rE "economy\.(gold|current_mana|reserve_mana|mana_cap|reserved_gold)\s*=" server/src/ | grep -v "core/economy/"` returns zero matches
- [ ] `cargo check -p server` passes after adding these two files

---

## Implementation Notes

*Derived from EPIC.md §Deliverables and economy-system.md Rules 1–5, 7:*

**Struct layout:**
```rust
// server/src/core/economy/state.rs
use bevy::prelude::Resource;
use std::collections::HashMap;
use shared::protocol::PlayerId;

#[derive(Clone, Debug)]
pub struct PlayerEconomy {
    pub gold: u32,
    pub current_mana: u32,
    pub reserve_mana: u32,
    pub mana_cap: u32,
    pub reserved_gold: u32,
}

#[derive(Resource)]
pub struct PlayerEconomies(pub HashMap<PlayerId, PlayerEconomy>);

#[derive(Resource)]
pub struct InterestSnapshots(pub HashMap<PlayerId, u32>);

#[derive(Debug, PartialEq)]
pub enum SpendError {
    InsufficientFunds,
    ReserveOnlyButCurrentProvided,
    HandFull,
    WrongPhase,
    PlayerNotFound,
}
```

**Auto-split logic in `apply_spend`:**
```rust
pub fn apply_spend(economy: &mut PlayerEconomy, cost: u32, from_reserve_only: bool) {
    if from_reserve_only {
        economy.reserve_mana = economy.reserve_mana.saturating_sub(cost);
    } else {
        let from_current = cost.min(economy.current_mana);
        let from_reserve = cost.saturating_sub(from_current);
        economy.current_mana = economy.current_mana.saturating_sub(from_current);
        economy.reserve_mana = economy.reserve_mana.saturating_sub(from_reserve);
    }
}
```

**`can_afford_bid` / `can_afford_shop` share the same formula** — `(gold - reserved_gold) >= amount`. Both are read-only. Implement as separate named functions per GDD Rule 7 to enable future divergence (e.g., hand-full check on bid side is enforced at the system layer, not here).

**`increment_mana_cap` cap enforcement:**
```rust
pub fn increment_mana_cap(economy: &mut PlayerEconomy, config: &GameConfig) {
    if economy.mana_cap < config.mana_cap_max {
        economy.mana_cap += 1;
    }
}
```
`mana_cap_max` is a `GameConfig` field (default 12). Do not hardcode 12 in this function.

**`PlayerId` type**: Import from `shared::protocol::PlayerId`. If not yet defined there when this story is implemented, use a local `type PlayerId = u32` placeholder with a `// TODO: import from shared/` comment — this story does not block on PlayerId finalisation.

---

## Out of Scope

- Story 002: `PlayerEconomies` initialisation on `SessionReady`; `on_draft_started` system
- Story 003: Interest snapshot and `on_resolution_phase_entered` system
- Story 004: Kill and objective award systems
- Story 005: Auction reservation systems
- Story 006: Network dispatch wiring (`S2CGoldUpdate`, `S2CGoldBroadcast`)
- `EconomyPlugin` registration — Plugin is authored as part of Story 002

---

## QA Test Cases

*QL-STORY-READY skipped — Lean mode.*

- **EC1: Auto-split draws current first, reserve as overflow**
  - Given: `PlayerEconomy { current_mana: 2, reserve_mana: 3, .. defaults }`
  - When: `validate_spend(&e, 4, false)` → `Ok(())`; then `apply_spend(&mut e, 4, false)`
  - Then: `e.current_mana == 0`, `e.reserve_mana == 1`

- **EC4: Rejection leaves pools unchanged**
  - Given: `PlayerEconomy { current_mana: 1, reserve_mana: 1, .. }`
  - When: `validate_spend(&e, 3, false)`
  - Then: Returns `Err(InsufficientFunds)`; `e.current_mana == 1`, `e.reserve_mana == 1`

- **EC5: Reserve-only card rejects when reserve insufficient even with current > cost**
  - Given: `PlayerEconomy { current_mana: 10, reserve_mana: 3, .. }`
  - When: `validate_spend(&e, 4, true)`
  - Then: Returns `Err(InsufficientFunds)` (not `ReserveOnlyButCurrentProvided` — cost exceeds reserve)

- **EC9/EC10: mana_cap increment and ceiling**
  - Given: `mana_cap = 10`, `config.mana_cap_max = 12`
  - When: `increment_mana_cap(&mut e, &config)` × 3
  - Then: After 1st call: `mana_cap == 11`; after 2nd: `mana_cap == 12`; after 3rd: `mana_cap == 12` (ceiling holds)

- **reserve_gold overflow guard**
  - Given: `gold = 3`, `reserved_gold = 0`
  - When: `reserve_gold(&mut e, 5)`
  - Then: Returns `Err(InsufficientFunds)`; `reserved_gold` unchanged

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/economy/state_api_test.rs` — all test cases passing; covers EC1–EC11
**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: S1-09 (Server-side RNG Story 001) for `PlayerId` type definition in `shared/`; workspace scaffolding (`workspace-and-shared-types` Story 001) must exist
- Unlocks: Story 002 (Initialisation + DraftStarted subscriber), Story 003 (Interest snapshot), Story 004 (Kill and objective awards), Story 005 (Auction reservation)
