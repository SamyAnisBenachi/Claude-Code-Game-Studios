# Story 001: State Types and Single-Writer API

> **Epic**: Economy System
> **Status**: Ready
> **Layer**: Core
> **Type**: Logic
> **Manifest Version**: 2026-04-29

## Context

**GDD**: `design/gdd/economy-system.md`
**Requirement**: `TR-ECO-01`, `TR-ECO-03`, `TR-ECO-06`, `TR-ECO-07`, `TR-ECO-08` (partial — reservation functions)

> ⚠️ TR-IDs are informal — `docs/architecture/tr-registry.yaml` is empty. Populate via `/architecture-review` before `/story-done`.

**ADR Governing Implementation**: [ADR-010: RSM Phase Event Bus](../../../docs/architecture/adr-010-rsm-event-bus.md)
**ADR Decision Summary**: Economy subscribes to `DraftStarted` and `ResolutionPhaseEntered` via `MessageReader<T>`. All economy state mutation goes through a single-writer API; direct field mutation outside `economy/api.rs` is forbidden and CI-gated.

**Engine**: Bevy 0.18 | **Risk**: LOW (this story contains pure Rust functions — no Bevy ECS machinery required)
**Engine Notes**: No `EventWriter`/`EventReader` used here (deprecated in Bevy 0.17+). This story defines plain Rust structs and pure functions. ECS integration is in Stories 002–006.

**Control Manifest Rules (Core Layer)**:
- Required: `PlayerEconomy` mutated ONLY through functions in `economy/api.rs`
- Required: All u32 arithmetic uses `saturating_sub` to prevent underflow
- Forbidden: `EventWriter`, `EventReader`, `Events<T>`, `app.add_event::<T>()`
- Forbidden: `unwrap()` in production paths — use `?` or `expect("message")`
- Guardrail: `reserved_gold` must never exceed `gold` (debug-assert in API)

---

## Acceptance Criteria

*From GDD `design/gdd/economy-system.md`, Rules 1–5 and 7:*

- [ ] **EC1** `PlayerEconomy` struct defined with fields `gold: u32`, `current_mana: u32`, `reserve_mana: u32`, `mana_cap: u32`, `reserved_gold: u32`; derives `Clone`, `Debug`
- [ ] **EC2 (auto-split)** `validate_spend` + `apply_spend` with `from_reserve_only=false`: given `current_mana=2`, `reserve_mana=3`, cost=4 → `current_mana=0`, `reserve_mana=1`
- [ ] **EC3 (current=0)** Given `current_mana=0`, `reserve_mana=5`, cost=3, `from_reserve_only=false` → `current_mana=0`, `reserve_mana=2`
- [ ] **EC4 (exact-current)** Given `current_mana=4`, `reserve_mana=2`, cost=4 → `current_mana=0`, `reserve_mana=2` (reserve untouched)
- [ ] **EC5 (rejection)** Given `current_mana=1`, `reserve_mana=1`, cost=3 → `validate_spend` returns `Err(InsufficientFunds)`; neither pool deducted
- [ ] **EC6 (reserve-only)** Given card with `from_reserve_only=true`, cost=4, `reserve_mana=3`, `current_mana=10` → `validate_spend` returns `Err(InsufficientFunds)`; `current_mana` does not substitute
- [ ] **EC7 (reserve persistence)** Calling `apply_spend`, `apply_gold_award`, `discard_current_mana`, `increment_mana_cap` on a `PlayerEconomy` with `reserve_mana=7` leaves `reserve_mana=7` after each call (reserve not cleared by any function except explicit `apply_spend(from_reserve_only=true)`)
- [ ] **EC8 (Gelure transfer)** Given `current_mana=5`: call `add_reserve(&mut e, 5)` then `discard_current_mana(&mut e)` → `reserve_mana` increases by 5, `current_mana=0`
- [ ] **EC9 (Gelure no-op)** Given `current_mana=0`: call `add_reserve(&mut e, 0)` then `discard_current_mana(&mut e)` → no error, `reserve_mana` unchanged
- [ ] **EC10 (mana cap increment)** Given `mana_cap=10`, `config.mana_cap_max=12`: `increment_mana_cap(&mut e, &config)` → `mana_cap=11`
- [ ] **EC11 (mana cap ceiling)** Given `mana_cap=12`: `increment_mana_cap` called three times → `mana_cap` remains 12
- [ ] **EC-GOLD-AWARD** `apply_gold_award(&mut e, 3)` with `gold=7` → `gold=10`; `apply_gold_award(&mut e, 0)` → gold unchanged
- [ ] **EC-SPEND-GOLD** `spend_gold(&mut e, 5)` with `gold=8` → `gold=3`, returns `Ok(())`; `spend_gold(&mut e, 5)` with `gold=2` → returns `Err(InsufficientFunds)`, `gold=2` unchanged
- [ ] **EC-ZERO-COST** `apply_spend(&mut e, 0, false)` with any pool values → both pools unchanged, no error
- [ ] **EC-RESERVE-GOLD** `reserve_gold(&mut e, 6)` with `gold=10`, `reserved_gold=0` → `reserved_gold=6`, `gold=10` unchanged
- [ ] **EC-RESERVE-EXCEEDS** `reserve_gold(&mut e, 5)` with `gold=3` → `Err(InsufficientFunds)`, `reserved_gold=0`
- [ ] **EC-RELEASE** `release_gold_reservation(&mut e, 5)` with `reserved_gold=5` → `reserved_gold=0`; release with amount > `reserved_gold` (saturating_sub) → `reserved_gold=0`, no panic
- [ ] **EC-AFFORD-BID** `can_afford_bid(&e, 5)` with `gold=3` → `false`; `can_afford_bid(&e, 5)` with `gold=5`, `reserved_gold=0` → `true`; `gold=8`, `reserved_gold=5`, bid=4 → `false` (effective=3)
- [ ] **EC-AFFORD-SHOP** `can_afford_shop(&e, 4)` with `gold=8`, `reserved_gold=5` → `false` (effective=3); `can_afford_shop(&e, 3)` same state → `true`
- [ ] **SpendError enum** defined: `InsufficientFunds`, `ReserveOnlyButCurrentProvided` (unused in this story — defined for completeness), `HandFull`, `WrongPhase`, `PlayerNotFound`

---

## Implementation Notes

*From ADR-010 and EPIC.md scope:*

File layout:
- `server/src/core/economy/state.rs` — `PlayerEconomy`, `PlayerEconomies(HashMap<PlayerId, PlayerEconomy>)`, `InterestSnapshots(HashMap<PlayerId, u32>)`, `SpendError`
- `server/src/core/economy/api.rs` — all public functions below

Public API to implement (full signatures):

```rust
pub fn validate_spend(economy: &PlayerEconomy, cost: u32, from_reserve_only: bool) -> Result<(), SpendError>;
pub fn apply_spend(economy: &mut PlayerEconomy, cost: u32, from_reserve_only: bool);
pub fn apply_gold_award(economy: &mut PlayerEconomy, amount: u32);
pub fn spend_gold(economy: &mut PlayerEconomy, amount: u32) -> Result<(), SpendError>;
pub fn add_reserve(economy: &mut PlayerEconomy, amount: u32);
pub fn discard_current_mana(economy: &mut PlayerEconomy);
pub fn reserve_gold(economy: &mut PlayerEconomy, amount: u32) -> Result<(), SpendError>;
pub fn release_gold_reservation(economy: &mut PlayerEconomy, amount: u32);
pub fn increment_mana_cap(economy: &mut PlayerEconomy, config: &GameConfig);
pub fn can_afford_bid(economy: &PlayerEconomy, amount: u32) -> bool;
pub fn can_afford_shop(economy: &PlayerEconomy, cost: u32) -> bool;
```

**Auto-split logic** (`apply_spend`, `from_reserve_only=false`):
1. Draw `min(cost, current_mana)` from `current_mana`
2. Draw remainder from `reserve_mana`
Both use `saturating_sub`.

**Reserve-only** (`apply_spend`, `from_reserve_only=true`):
- `reserve_mana -= cost` (saturating_sub). `current_mana` untouched.

**`reserve_gold` invariant** (debug assert): `reserved_gold <= gold` after call. Enforce with `debug_assert!(e.reserved_gold <= e.gold)`.

**`increment_mana_cap`**: `e.mana_cap = (e.mana_cap + 1).min(config.mana_cap_max)`.

---

## Out of Scope

*Handled by neighbouring stories:*

- [Story 002]: `PlayerEconomies` resource registration, `on_draft_started` subscriber, `SessionReady` init
- [Story 003]: `on_resolution_phase_entered` snapshot system, `discard_current_mana_at_resolution_end`
- [Story 004]: `handle_kill_award`, `handle_objective_award` systems
- [Story 005]: `validate_auction_bid` (hand-full check + affordability combined)
- [Story 006]: `S2CGoldUpdate` / `S2CGoldBroadcast` network dispatch

---

## QA Test Cases

*Written by qa-lead at story creation.*

- **EC2 (auto-split)**
  - Given: `PlayerEconomy { current_mana: 2, reserve_mana: 3, .. }`
  - When: `validate_spend(&e, 4, false)` → `Ok(())`; `apply_spend(&mut e, 4, false)`
  - Then: `e.current_mana == 0`; `e.reserve_mana == 1`
  - Edge cases: cost exactly equals current (EC4); cost=0 (EC-ZERO-COST)

- **EC5 (rejection)**
  - Given: `PlayerEconomy { current_mana: 1, reserve_mana: 1, .. }`
  - When: `validate_spend(&e, 3, false)`
  - Then: `Err(SpendError::InsufficientFunds)`; `e.current_mana == 1`; `e.reserve_mana == 1`
  - Edge cases: total == cost-1 (exactly one short); total == 0

- **EC6 (reserve-only)**
  - Given: `PlayerEconomy { current_mana: 10, reserve_mana: 3, .. }`
  - When: `validate_spend(&e, 4, true)`
  - Then: `Err(InsufficientFunds)`; neither pool modified
  - Edge cases: `reserve_mana == cost` → `Ok(())`; `current_mana < cost` with `from_reserve_only=false` → also `Err`

- **EC7 (reserve persistence)**
  - Given: `PlayerEconomy { current_mana: 5, reserve_mana: 7, gold: 10, mana_cap: 10, reserved_gold: 0 }`
  - When: call `apply_spend(&mut e, 5, false)`; `apply_gold_award(&mut e, 3)`; `discard_current_mana(&mut e)`; `increment_mana_cap(&mut e, &config)` — assert after each
  - Then: `e.reserve_mana == 7` after every call
  - Edge cases: all four calls in sequence

- **EC10 (mana cap increment)**
  - Given: `PlayerEconomy { mana_cap: 10 }`; `GameConfig { mana_cap_max: 12 }`
  - When: `increment_mana_cap(&mut e, &config)`
  - Then: `e.mana_cap == 11`
  - Edge cases: `mana_cap=11 → 12`; `mana_cap=12 → 12` (ceiling)

- **EC11 (mana cap ceiling)**
  - Given: `PlayerEconomy { mana_cap: 12 }`
  - When: `increment_mana_cap` called 3×
  - Then: `e.mana_cap == 12` after every call; no panic

- **EC-SPEND-GOLD**
  - Given: `PlayerEconomy { gold: 8, .. }`
  - When: `spend_gold(&mut e, 5)`
  - Then: `e.gold == 3`; `Ok(())`
  - When (insufficient): `PlayerEconomy { gold: 2 }`; `spend_gold(&mut e, 5)`
  - Then: `Err(InsufficientFunds)`; `e.gold == 2`
  - Edge cases: `spend_gold(gold exactly)` → `gold == 0`; `spend_gold(0)` → unchanged

- **EC-RESERVE-GOLD lifecycle**
  - Given: `PlayerEconomy { gold: 10, reserved_gold: 0 }`
  - When: (1) `reserve_gold(&mut e, 7)` → `Ok(())`; (2) `can_afford_shop(&e, 4)` → false; (3) `release_gold_reservation(&mut e, 7)`; (4) `can_afford_shop(&e, 4)` → true
  - Then: All 4 assertions pass; `e.gold == 10` throughout

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/economy/state_api_test.rs` — must exist and pass
**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: None — pure Rust, no Bevy ECS
- Unlocks: Story 002 (needs API functions), Story 005 (needs reservation API)
