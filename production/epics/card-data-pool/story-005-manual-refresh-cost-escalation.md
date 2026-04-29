# Story 005: Manual Refresh + Cost Escalation

> **Epic**: Card Data & Pool
> **Status**: Ready
> **Layer**: Core
> **Type**: Integration
> **Manifest Version**: 2026-04-29

## Context

**GDD**: `design/gdd/card-data-pool.md`
**Requirement**: `TR-CDP-09` (manual refresh aspect)
*(TR-IDs are informal — `docs/architecture/tr-registry.yaml` is unpopulated.)*

**ADRs Governing Implementation**:
- [ADR-006: Card Data Schema and Pool State Architecture](../../../docs/architecture/adr-006-card-data-schema.md) — `ManualRefreshCount`; refresh cost formula; phase-gate to `DraftShop`
- [ADR-010: RSM Phase Event Bus](../../../docs/architecture/adr-010-rsm-event-bus.md) — `C2SShopRefresh` is a client→server Lightyear message; phase-gate pattern required

**ADR Decision Summary**: `on_manual_refresh` subscribes to `MessageReader<C2SShopRefresh>`. Phase-gate: silently discard if `phase != DraftShop` (debug log only — zero S2C response per control manifest). Cost formula: `refresh_base_cost + ManualRefreshCount[player]` gold. Calls Economy `validate_spend` + `apply_spend` before drawing new cards. Counter increments only on successful spend. Reset to 0 at each DRAFT entry (owned by Story 004).

**Engine**: Bevy 0.18 | **Risk**: MEDIUM (Bevy 0.18 MessageReader API; cross-system Economy call; Lightyear C2S message shape)
**Engine Notes**:
- `EventWriter`/`EventReader` no longer exist in Bevy 0.17+. Use `MessageReader<C2SShopRefresh>` + `app.add_message::<C2SShopRefresh>()`.
- `C2SShopRefresh` is a Lightyear C2S message — confirm exact deserialized event type with `liv-bevy-lightyear` skill before implementing.
- `liv-bevy-018` mandatory on all `.rs` files; `liv-bevy-lightyear` mandatory on this system (lightyear message handling).

**Control Manifest Rules (Core layer)**:
- Required: Phase-gate pattern in every C2S handler — `if phase != DraftShop { debug!(...); continue; }`. Invalid phase → silently discard, debug log only, **zero S2C response**.
- Required: `validate_spend` called BEFORE `apply_spend`. Never deduct gold without first confirming the player can afford it.
- Required: `ManualRefreshCount[player]` incremented ONLY after successful spend.
- Required: `refresh_base_cost` read from `GameConfig` — never hardcoded.
- Forbidden: `refresh_shop()` called before `apply_spend()` succeeds.
- Forbidden: `unwrap()` in production paths.

---

## Acceptance Criteria

*From EPIC.md deliverables and Economy GDD §8 criteria EC24/EC25/EC26:*

- [ ] **AC-1 (EC24)**: GIVEN `ManualRefreshCount[player] == 0` and player has sufficient gold, WHEN `C2SShopRefresh` received during `DraftShop` phase, THEN gold deducted == `refresh_base_cost` (1g); `ManualRefreshCount[player]` incremented to 1; `ShopSlots[player]` updated with up to 3 new cards.
- [ ] **AC-2**: GIVEN `ManualRefreshCount[player] == 1`, WHEN second `C2SShopRefresh` received during `DraftShop`, THEN cost == `refresh_base_cost + 1` (2g); `ManualRefreshCount[player]` incremented to 2.
- [ ] **AC-3 (EC25)**: GIVEN `ManualRefreshCount[player] == 2`, WHEN third `C2SShopRefresh`, THEN cost == `refresh_base_cost + 2` (3g); `ManualRefreshCount[player]` incremented to 3.
- [ ] **AC-4 (EC26)**: GIVEN player gold < required refresh cost, WHEN `C2SShopRefresh` received during `DraftShop`, THEN `ShopSlots[player]` unchanged; player gold unchanged; no S2C response enqueued; `ManualRefreshCount[player]` unchanged.
- [ ] **AC-5**: GIVEN `C2SShopRefresh` received while `Res<RoundState>.phase != DraftShop` (e.g., Placement or DraftInitial), WHEN `on_manual_refresh` runs, THEN message silently discarded — no shop refresh, no Economy call, no S2C response, `debug!` log only.
- [ ] **AC-6**: GIVEN `ManualRefreshCount[player] == 3` from previous DRAFT phase, WHEN a new DRAFT auto-refresh fires via `ShopRefreshNeeded` (Story 004's system), THEN `ManualRefreshCount[player] == 0` (reset by Story 004 — tested here as integration).

---

## Implementation Notes

*From ADR-006 EPIC deliverables:*

**Cost formula**: `cost = config.manual_refresh_base_cost + ManualRefreshCount[player]`
- 1st refresh: `ManualRefreshCount == 0` → cost = `refresh_base_cost` (e.g., 1g)
- 2nd refresh: `ManualRefreshCount == 1` → cost = 2g
- 3rd refresh: `ManualRefreshCount == 2` → cost = 3g

**`on_manual_refresh` control flow**:
```
1. for msg in reader.read():
2.   phase gate: if phase != DraftShop → debug!(...); continue
3.   n = ManualRefreshCount[player].unwrap_or(0)
4.   cost = config.manual_refresh_base_cost + n
5.   validate_spend(economy[player], cost) → if Err → continue (no S2C, no mutation)
6.   apply_spend(economy[player], cost)
7.   ManualRefreshCount[player] = n + 1
8.   slots = refresh_shop(pool[player], catalog, family_index, &mut rng, config, 3)
9.   ShopSlots[player] = slots
   // S2CShopSlots dispatch is Story 006's responsibility
```

**Economy integration**: `validate_spend` and `apply_spend` are from the Economy System epic API (`server/src/core/economy/api.rs`). Coordinate exact signatures with the Economy epic. The key invariant: validate BEFORE spend, spend BEFORE draw.

**`C2SShopRefresh` message type**: Defined in `shared/src/protocol.rs`. `liv-bevy-lightyear` skill must verify that Lightyear 0.26 delivers this as a `MessageReader<C2SShopRefresh>` system parameter.

**`manual_refresh_base_cost`** is a `GameConfig` tuning knob (default 1g). Add to `assets/config/game_config.ron` alongside pool copy counts.

**Registration in `CardPoolPlugin`**:
```rust
app.add_systems(Update, on_manual_refresh.after(advance_phase));
```

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- [Story 004]: `ManualRefreshCount` resource declaration and reset at DRAFT entry — Story 004 owns the reset; this story only increments
- [Story 006]: Sending `S2CShopSlots` to client — this story writes to `ShopSlots` resource; network dispatch is Story 006's responsibility
- Economy System epic: `validate_spend` and `apply_spend` — this story calls them; does not implement them

---

## QA Test Cases

*Written by QA Lead at story creation. Implement against these — do not invent new test cases.*

- **AC-1** — `test_first_manual_refresh_costs_base_gold`
  - Given: `ManualRefreshCount[A] == 0`; `PlayerEconomy[A].gold == 10`; `config.manual_refresh_base_cost == 1`; phase == DraftShop; `C2SShopRefresh { player: A }` written
  - When: `on_manual_refresh` processes message
  - Then: `PlayerEconomy[A].gold == 9`; `ManualRefreshCount[A] == 1`; `ShopSlots[A]` has up to 3 new entries
  - Edge cases: `manual_refresh_base_cost == 2` → first refresh costs 2g

- **AC-2** — `test_second_manual_refresh_cost_escalated`
  - Given: `ManualRefreshCount[A] == 1`; `PlayerEconomy[A].gold == 10`; `config.manual_refresh_base_cost == 1`; phase == DraftShop
  - When: `C2SShopRefresh` processed
  - Then: `PlayerEconomy[A].gold == 8` (2g deducted); `ManualRefreshCount[A] == 2`
  - Edge cases: Cost is additive — `refresh_base_cost + ManualRefreshCount` (not multiplicative)

- **AC-3** — `test_third_manual_refresh_cost_escalated`
  - Given: `ManualRefreshCount[A] == 2`; `PlayerEconomy[A].gold == 10`; phase == DraftShop
  - When: `C2SShopRefresh` processed
  - Then: `PlayerEconomy[A].gold == 7` (3g); `ManualRefreshCount[A] == 3`
  - Edge cases: nth refresh → cost == `base_cost + (n-1)` for all n ≥ 1

- **AC-4** — `test_insufficient_gold_no_refresh`
  - Given: `ManualRefreshCount[A] == 0`; `PlayerEconomy[A].gold == 0`; `config.manual_refresh_base_cost == 1`; phase == DraftShop
  - When: `C2SShopRefresh` processed
  - Then: `PlayerEconomy[A].gold == 0` (unchanged); `ManualRefreshCount[A] == 0` (unchanged); `ShopSlots[A]` unchanged
  - Edge cases: gold == cost → succeeds; gold == cost - 1 → fails; verify no S2C message enqueued on failure

- **AC-5** — `test_wrong_phase_discards_message`
  - Given: `Res<RoundState>.phase == RoundPhase::Placement`; `C2SShopRefresh { player: A }` written; `PlayerEconomy[A].gold == 10`
  - When: `on_manual_refresh` processes message
  - Then: `PlayerEconomy[A].gold == 10` (Economy not called); `ShopSlots[A]` unchanged; `ManualRefreshCount[A]` unchanged
  - Edge cases: DraftInitial → discarded; DraftAuction → discarded; Resolution → discarded; only DraftShop allows refresh

- **AC-6** — `test_draft_entry_resets_refresh_count` *(joint with Story 004)*
  - Given: `ManualRefreshCount[A] == 3`; `ShopRefreshNeeded { player: A }` written (simulating new DRAFT entry)
  - When: `on_shop_refresh_needed` (Story 004) processes message
  - Then: `ManualRefreshCount[A] == 0` post-processing
  - Edge cases: Next `C2SShopRefresh` after reset costs `base_cost + 0 = base_cost` again

---

## Test Evidence

**Story Type**: Integration
**Required evidence**:
- `tests/integration/pool/manual_refresh_test.rs` — must exist and all tests must pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 004 (ShopRefreshNeeded + SessionReady Init) must be **DONE** — `ManualRefreshCount` resource and `ShopSlots` population required
- Depends on: Economy System epic — `validate_spend` and `apply_spend` API must be defined and callable
- Depends on: `C2SShopRefresh` message type defined in `shared/src/protocol.rs` (workspace-and-shared-types Foundation epic)
- Unlocks: Story 006 (Network Dispatch — reads `ShopSlots` updated by this story)
