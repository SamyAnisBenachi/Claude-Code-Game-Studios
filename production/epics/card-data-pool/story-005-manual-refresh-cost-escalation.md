# Story 005: Manual Refresh & Cost Escalation

> **Epic**: Card Data & Pool
> **Status**: Ready
> **Layer**: Core
> **Type**: Integration
> **Manifest Version**: 2026-04-29

## Context

**GDD**: `design/gdd/card-data-pool.md` and `design/gdd/economy-system.md` (EC24/EC25/EC26)
**Requirement**: Manual refresh cost escalation during DRAFT_SHOP; counter reset on DRAFT entry; Economy spend validation before refresh is performed

**ADRs Governing Implementation**:
- ADR-006: Card Data Schema and Pool State Architecture — `ManualRefreshCount` tracks paid manual refreshes per player per DRAFT phase; `n`th refresh in a phase costs `refresh_base_cost + (n - 1)` gold; counter resets to 0 at every DRAFT entry (handled by Story 004's `on_shop_refresh_needed`)
- ADR-010: RSM Phase Event Bus — `on_manual_refresh` subscribes to `EventReader<C2SShopRefresh>`; phase-gate to `Phase::DraftShop` only; any request in a non-DraftShop phase is rejected with a `S2CError` response (not silently dropped)

**ADR Decision Summary**: `on_manual_refresh` reads `C2SShopRefresh { player_id }`, verifies phase is `DraftShop`, computes cost as `refresh_base_cost + (n - 1)` where `n = ManualRefreshCount[player] + 1`, calls Economy's `validate_spend(economy, cost, from_reserve_only=false)` — if `Err`, sends `S2CError::InsufficientFunds` and returns. On `Ok`, calls `apply_spend`, increments `ManualRefreshCount[player]`, calls `refresh_shop(pool, catalog, family_index, rng, config, 3)`, writes new slots to `ShopSlots[player]`, enqueues `S2CShopSlots`.

**Engine**: Bevy 0.18 | **Risk**: MEDIUM
**Engine Notes**:
- Bevy 0.18: `C2SShopRefresh` messages use `MessageReader<C2SShopRefresh>::read()` — `EventReader` no longer exists. `liv-bevy-018` mandatory.
- Economy `validate_spend` and `apply_spend` are pure functions (no ECS surface) but `PlayerEconomies` is a `ResMut` — the borrow checker enforces that `PlayerPools` and `PlayerEconomies` are not simultaneously mutably borrowed in the same system call. Destructure borrows carefully.
- `C2SShopRefresh` is a client-to-server Lightyear message. At the system layer it arrives as an ECS event. Confirm with `liv-bevy-lightyear` skill the exact event type name for deserialized C2S messages in Lightyear 0.26.

**Control Manifest Rules (Core layer)**:
- Required: `on_manual_refresh` phase-gate check must occur BEFORE accessing `ManualRefreshCount` or calling Economy functions. Reject with `S2CError` if phase != `DraftShop`.
- Required: `validate_spend` must be called before `apply_spend`. Never deduct gold without first verifying the player can afford it.
- Required: `ManualRefreshCount[player]` is incremented ONLY after a successful spend. A failed spend (insufficient funds) must not increment the counter.
- Required: `refresh_base_cost` is read from `GameConfig.manual_refresh_base_cost` — not hardcoded. This is a gate knob.
- Forbidden: Calling `refresh_shop()` before `apply_spend()` succeeds. The shop must not refresh if the player cannot afford it.
- Forbidden: Processing `C2SShopRefresh` for a player not present in `PlayerPools` — log error and return.

---

## Acceptance Criteria

- [ ] `server/src/core/pool/system.rs` is extended with `on_manual_refresh`:
  - `on_manual_refresh(mut events: MessageReader<C2SShopRefresh>, mut pools: ResMut<PlayerPools>, mut shop_slots: ResMut<ShopSlots>, mut refresh_count: ResMut<ManualRefreshCount>, mut economies: ResMut<PlayerEconomies>, round_state: Res<RoundState>, catalog: Res<CardCatalog>, family_index: Res<FamilyIndex>, mut rng: ResMut<ServerRng>, config: Res<GameConfig>, mut errors: MessageWriter<S2CError>)` — TODO(liv-bevy-018): verify MessageReader/MessageWriter type names in Bevy 0.18
- [ ] Phase gate: GIVEN `round_state.phase != Phase::DraftShop`, WHEN `C2SShopRefresh` received for any player, THEN event is consumed; `S2CError::WrongPhase` is enqueued for that player; no pool or economy mutation occurs
- [ ] Cost escalation formula: cost for the `n`th refresh (1-indexed) in a DRAFT phase = `config.manual_refresh_base_cost + (n - 1)` gold
  - 1st refresh: `base_cost + 0 = base_cost` (e.g., 1g)
  - 2nd refresh: `base_cost + 1` (e.g., 2g)
  - 3rd refresh: `base_cost + 2` (e.g., 3g)
- [ ] **EC24**: GIVEN `ManualRefreshCount[player] = 0`, `gold >= base_cost`, WHEN `C2SShopRefresh` received in `DraftShop`, THEN spend of `base_cost` gold applied; `ManualRefreshCount[player] = 1`; new 3-card shop emitted as `S2CShopSlots`
- [ ] **EC25**: GIVEN `ManualRefreshCount[player] = 2`, `gold >= base_cost + 2`, WHEN `C2SShopRefresh` received, THEN spend of `base_cost + 2` gold applied; `ManualRefreshCount[player] = 3`
- [ ] **EC26**: GIVEN `ManualRefreshCount[player] = 1`, `gold < base_cost + 1` (insufficient), WHEN `C2SShopRefresh` received, THEN `validate_spend` returns `Err`; no gold deducted; `ManualRefreshCount[player]` remains `1`; `S2CError::InsufficientFunds` enqueued for player; no `S2CShopSlots` emitted
- [ ] Counter reset: GIVEN `ManualRefreshCount[player] = 5` (from prior DRAFT_SHOP), WHEN next DRAFT phase begins and `ShopRefreshNeeded` fires (Story 004), THEN `ManualRefreshCount[player] == 0`; confirmed in this story's integration test via the full flow
- [ ] Successful refresh produces a new `S2CShopSlots` event with exactly 3 card IDs (or fewer on pool exhaustion)
- [ ] Failed refresh (wrong phase or insufficient funds) produces NO `S2CShopSlots` event
- [ ] `on_manual_refresh` is registered in `CardPoolPlugin` (Story 004's plugin) with `.after(AdvancePhaseSet)` scheduling label
- [ ] Economy integration: calls `validate_spend(economy, cost, from_reserve_only=false)` then `apply_spend(economy, cost, from_reserve_only=false)` from `server/src/core/economy/api.rs`
- [ ] Unit test: cost escalation formula produces correct values across 3 consecutive refreshes with `base_cost = 1`: costs are [1, 2, 3]
- [ ] Integration test: full flow — DRAFT_SHOP entry → refresh × 2 (costs 1g, 2g) → next DRAFT entry → counter reset → refresh × 1 (costs 1g again)
- [ ] `cargo check -p server` passes after extending `system.rs`

---

## Implementation Notes

*Derived from EPIC.md §Deliverables, ADR-006 §Implementation Notes, and `economy-system.md` EC24/EC25/EC26:*

**Cost formula:**
```rust
let n = refresh_count.0.get(&player_id).copied().unwrap_or(0);
let cost = config.manual_refresh_base_cost + n;  // n = 0 for first refresh
```
`n` here is the current counter value (0-indexed), so the 1st refresh costs `base_cost + 0`. The counter is incremented after a successful spend.

**`on_manual_refresh` control flow:**
```
1. read C2SShopRefresh { player_id }
2. phase gate: if phase != DraftShop → S2CError::WrongPhase, return
3. get pool for player_id → if not found, log error, return
4. n = ManualRefreshCount[player_id]
5. cost = config.manual_refresh_base_cost + n
6. economy = PlayerEconomies[player_id]
7. validate_spend(economy, cost, false) → if Err(InsufficientFunds) → S2CError::InsufficientFunds, return
8. apply_spend(economy, cost, false)
9. ManualRefreshCount[player_id] += 1
10. slots = refresh_shop(pool, catalog, family_index, rng, config, 3)
11. ShopSlots[player_id] = slots.clone()
12. MessageWriter<S2CShopSlots>.write(S2CShopSlots { player_id, slots })  // TODO(liv-bevy-018): verify MessageWriter type name
```

**Borrow splitting:** `PlayerPools` and `PlayerEconomies` are two separate `ResMut` parameters — Bevy allows multiple mutable resource borrows in the same system as long as they are distinct resource types. Do not attempt to hold both as `&mut` references to fields of the same struct.

**`C2SShopRefresh` event name:** This is the Lightyear C2S message type from `shared/src/protocol.rs`. In Lightyear 0.26 with Bevy 0.18, C2S messages arrive as ECS events after deserialization by the network plugin. Confirm the exact event wrapper type with the `liv-bevy-lightyear` skill — it may be `MessageEvent<C2SShopRefresh>` or a direct event depending on channel configuration.

**`S2CError` event:** If `S2CError` is not yet defined in `shared/src/protocol.rs`, define a minimal version for this story with variants `WrongPhase` and `InsufficientFunds`. Coordinate with `workspace-and-shared-types` Story 004 for the canonical definition.

**`manual_refresh_base_cost` tuning knob:** Default value 1 (gold). This is a gate knob — it controls the cost floor for manual refreshes. It lives in `GameConfig.manual_refresh_base_cost: u32`. Document in `assets/config/game_config.ron` alongside other pool tuning knobs.

---

## Out of Scope

- Story 004: `ManualRefreshCount` initialization and reset on DRAFT entry
- Story 006: Network dispatch of `S2CShopSlots` — this story enqueues the event; Story 006 sends it over Lightyear
- Economy epic: `validate_spend` and `apply_spend` implementation — this story CALLS them; they are already defined in the Economy epic
- Story 003: `refresh_shop()` implementation — this story CALLS it

---

## QA Test Cases

- **EC24: First refresh costs base_cost**
  - Given: `ManualRefreshCount[player] = 0`; `economy.gold = 5`; `config.manual_refresh_base_cost = 1`; phase = `DraftShop`
  - When: `C2SShopRefresh { player }` received
  - Then: `economy.gold == 4`; `ManualRefreshCount[player] == 1`; `S2CShopSlots` with 3 cards emitted

- **EC25: Third refresh costs base_cost + 2**
  - Given: `ManualRefreshCount[player] = 2`; `economy.gold = 5`; `config.manual_refresh_base_cost = 1`; phase = `DraftShop`
  - When: `C2SShopRefresh { player }` received
  - Then: `economy.gold == 2`; `ManualRefreshCount[player] == 3`

- **EC26: Insufficient funds — no mutation**
  - Given: `ManualRefreshCount[player] = 1`; `economy.gold = 1`; `config.manual_refresh_base_cost = 1`; phase = `DraftShop`
  - When: `C2SShopRefresh { player }` (cost would be 2g)
  - Then: `economy.gold == 1` (unchanged); `ManualRefreshCount[player] == 1` (unchanged); `S2CError::InsufficientFunds` emitted; no `S2CShopSlots`

- **Phase gate — non-DraftShop rejected**
  - Given: phase = `DraftInitial`
  - When: `C2SShopRefresh { player }` received
  - Then: `S2CError::WrongPhase` emitted; `economy.gold` unchanged; no `S2CShopSlots`

- **Counter reset across DRAFT phases (integration)**
  - Given: `ManualRefreshCount[player] = 3` after DRAFT_SHOP round 1
  - When: next DRAFT entry fires `ShopRefreshNeeded`
  - Then: `ManualRefreshCount[player] == 0`; next `C2SShopRefresh` costs `base_cost + 0 = base_cost`

---

## Test Evidence

**Story Type**: Integration
**Required evidence**:
- `tests/unit/pool/manual_refresh_test.rs` — cost escalation formula; phase gate rejection; EC24/EC25/EC26
- `tests/integration/pool/manual_refresh_integration_test.rs` — full DRAFT_SHOP entry → refresh × 2 → next-round DRAFT entry → counter reset → refresh × 1 flow with Economy integration
**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 (provides `ManualRefreshCount`, `PlayerPools`, resource types)
- Depends on: Story 002 (provides `draw()` — called inside `refresh_shop()`)
- Depends on: Story 003 (provides `refresh_shop(pool, ..., 3)`)
- Depends on: Story 004 (provides `CardPoolPlugin` — `on_manual_refresh` is registered there; `ManualRefreshCount` reset behavior confirmed)
- Depends on: `economy-system` epic Story 001 — `validate_spend(economy, cost, false)` and `apply_spend(economy, cost, false)` pure API functions; `PlayerEconomies` resource type
- Depends on: `round-state-machine` epic — `Phase::DraftShop` variant, `RoundState` resource, `C2SShopRefresh` event type (or Lightyear message type)
- Depends on: `workspace-and-shared-types` Story 003 — `GameConfig.manual_refresh_base_cost` field
- Unlocks: Story 006 (network dispatch — depends on `S2CShopSlots` events being correctly enqueued by both Story 004 and Story 005)
