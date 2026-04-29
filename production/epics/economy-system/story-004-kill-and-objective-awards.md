# Story 004: Kill & Objective Awards

> **Epic**: Economy System
> **Status**: Ready
> **Layer**: Core
> **Type**: Logic
> **Manifest Version**: 2026-04-29

## Context

**GDD**: `design/gdd/economy-system.md`
**Requirement**: TR-??? (covers TR-ECO-08 partial — kill/objective gold awards; EC11: self-inflicted no-award guard; EC16: kill gold; EC17: objective gold; EC9: mana_cap increment via fake reward)

**ADR Governing Implementation**: ADR-010: RSM Phase Event Bus — Phase Message Catalog and Subscriber Contracts
**ADR Decision Summary**: Economy subscribes to `EventReader<UnitKilled>` and `EventReader<ObjectiveDestroyed>`. Both events are defined by their respective epics (Combat Resolution M2 and Objective System); this story consumes them as read-only subscribers. The self-inflicted guard (`attacker_player == defending_player`) is enforced in the Economy System handler — the Objective System does not filter this. `increment_mana_cap` is called only when `ObjectiveDestroyed.was_fake == true` and the server's RNG draw selects the mana reward (50/50 against free card pick).

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: MEDIUM
**Engine Notes**: Uses `MessageReader<UnitKilled>::read()` and `MessageReader<ObjectiveDestroyed>::read()` (Bevy 0.18 — `EventReader` no longer exists). Message types `UnitKilled` and `ObjectiveDestroyed` are defined by Combat Resolution (M2) and Objective System epics respectively and must derive `Message`. For M1 unit testing, these types are defined as minimal stubs in `tests/unit/economy/`. `liv-bevy-018` mandatory on all `.rs` files touched here.

**Control Manifest Rules (Core layer)**:
- Required: Self-inflicted objective guard: if `attacker_player_id == defending_player_id`, `handle_objective_award` returns early without calling `apply_gold_award`. Loss condition is evaluated by the RSM independently.
- Required: After each award, enqueue `S2CGoldUpdate` unicast and `S2CGoldBroadcast` broadcast (same pattern as `on_draft_started`).
- Required: `increment_mana_cap` is called inside `handle_objective_award` only when `event.was_fake == true` AND `event.reward_type == FakeReward::ManaCap`. The RNG draw is performed by the Objective System, which passes the result in the event — Economy is not responsible for the 50/50 roll.
- Forbidden: Economy must never evaluate `was_fake` reward RNG itself — the Objective System owns the random draw per ADR-005 (server-side RNG).

---

## Acceptance Criteria

- [ ] `server/src/core/economy/system.rs` contains `handle_kill_award` system:
  - Reads `MessageReader<UnitKilled>`, `ResMut<PlayerEconomies>`, `Res<GameConfig>`, `MessageWriter<S2CGoldUpdate>`, `MessageWriter<S2CGoldBroadcast>` — TODO(liv-bevy-018): verify MessageReader/MessageWriter type names
  - For each `UnitKilled` message: calls `api::apply_gold_award(killer_economy, config.kill_gold_reward)`; enqueues `S2CGoldUpdate` and `S2CGoldBroadcast` for the killer
  - Does NOT award gold to the killed player
- [ ] `server/src/core/economy/system.rs` contains `handle_objective_award` system:
  - Reads `MessageReader<ObjectiveDestroyed>`, `ResMut<PlayerEconomies>`, `Res<GameConfig>`, `MessageWriter<S2CGoldUpdate>`, `MessageWriter<S2CGoldBroadcast>`
  - For each `ObjectiveDestroyed` event where `attacker_player_id != defending_player_id`: calls `api::apply_gold_award(attacker_economy, config.objective_gold_reward)`; enqueues `S2CGoldUpdate` and `S2CGoldBroadcast`
  - For each `ObjectiveDestroyed` event where `attacker_player_id == defending_player_id`: returns early — no gold awarded (EC11 self-inflicted guard)
  - If `event.was_fake == true` AND `event.reward_type == FakeReward::ManaCap`: calls `api::increment_mana_cap(attacker_economy, config)`; enqueues `S2CGoldUpdate` for the attacker (mana_cap updated)
- [ ] `EconomyPlugin` schedules both systems in `Update` `.after(advance_phase)`:
  - Adds comment: `// TODO M2: also order .after(CombatSystemSet::ProcessKills) for handle_kill_award` and `.after(ObjectiveSystemSet::ProcessDestructions) for handle_objective_award`
- [ ] **EC11**: GIVEN `attacker_player_id == defending_player_id` (self-inflicted), WHEN `ObjectiveDestroyed` fires, THEN `attacker.gold` is unchanged; no `S2CGoldUpdate` enqueued for this event
- [ ] **EC16**: GIVEN a `UnitKilled` event with `killer_player_id = A`, WHEN `handle_kill_award` fires, THEN `PlayerEconomies[A].gold += config.kill_gold_reward` (default: 1)
- [ ] **EC17**: GIVEN `ObjectiveDestroyed { attacker_player_id: A, defending_player_id: B, was_fake: false, .. }`, WHEN `handle_objective_award` fires, THEN `PlayerEconomies[A].gold += config.objective_gold_reward` (default: 3)
- [ ] **EC9 via fake reward**: GIVEN `ObjectiveDestroyed { was_fake: true, reward_type: FakeReward::ManaCap, attacker_player_id: A, defending_player_id: B }`, WHEN `handle_objective_award` fires, THEN `PlayerEconomies[A].mana_cap += 1` (if not already at max); `S2CGoldUpdate` for A includes updated `mana_cap`
- [ ] Dual kill: GIVEN two `UnitKilled` events both with `killer_player_id = A` in the same frame, WHEN `handle_kill_award` processes both, THEN `gold += 2 × kill_gold_reward`
- [ ] `S2CGoldBroadcast` is enqueued (not suppressed) for kill rewards — gold is publicly visible
- [ ] `cargo check -p server` passes

---

## Implementation Notes

*Derived from EPIC.md §system.rs `handle_kill_award` and `handle_objective_award` and economy-system.md Rule 6, Edge Cases:*

**`UnitKilled` and `ObjectiveDestroyed` event types for M1 unit tests:** These types are not yet defined in production code (Combat Resolution and Objective System are M2/later epics). For unit test purposes, define minimal stub types at the top of the test file:
```rust
// tests/unit/economy/awards_test.rs — TEST STUBS ONLY
#[derive(Event, Clone, Debug)]
struct UnitKilled { pub killer_player_id: PlayerId, pub killed_player_id: PlayerId }

#[derive(Event, Clone, Debug)]
enum FakeReward { ManaCap, FreeCard }

#[derive(Event, Clone, Debug)]
struct ObjectiveDestroyed {
    pub attacker_player_id: PlayerId,
    pub defending_player_id: PlayerId,
    pub was_fake: bool,
    pub reward_type: Option<FakeReward>,
}
```
These stubs live ONLY in `tests/`. When Combat Resolution and Objective System are implemented, the production `handle_kill_award` and `handle_objective_award` systems import the real event types from those crates. The system function signatures match — only the import paths change.

**`increment_mana_cap` takes effect next DRAFT:** Per GDD Rule 5, `mana_cap` is permanent and takes effect at the START OF THE NEXT DRAFT PHASE. The `increment_mana_cap` API call in this system updates `mana_cap` immediately in the `PlayerEconomy` struct. When `on_draft_started` later calls `apply_mana_ramp`, it reads the updated `mana_cap`. No deferred application needed — the value is stored correctly, and `apply_mana_ramp` reads it at the right time.

**Free card reward path:** `FakeReward::FreeCard` is handled by the Objective System — it draws a card from the pool. Economy has no action for this reward type. The `handle_objective_award` system MUST NOT call any card-draw logic. A `FakeReward::FreeCard` event: award objective gold (if not self-inflicted), skip the mana cap branch.

**`S2CGoldUpdate` payload for mana_cap increment:** The `S2CGoldUpdate` message includes `mana_cap` as a field (per EPIC.md §system.rs). When `increment_mana_cap` fires, the enqueued `S2CGoldUpdate` must carry the new `mana_cap` value so the client's HUD updates correctly.

---

## Out of Scope

- Story 005: Auction reservation and bid validation
- Story 006: Actual Lightyear send — this story enqueues Bevy events; Story 006 dispatches them
- Class System Gelure / Miss Nuit reserve gain — those handlers call `add_reserve` directly; Economy only provides the API
- Prism "+1 reserve" spell card — Prism System calls `add_reserve(player, 1)`; Economy provides the API
- Objective System RNG draw for fake reward type (50/50 ManaCap vs FreeCard) — Objective System epic owns this

---

## QA Test Cases

*QL-STORY-READY skipped — Lean mode.*

- **EC16: Kill award**
  - Given: `World` with `PlayerEconomies` (player A: `gold = 5`), `GameConfig` (`kill_gold_reward = 1`)
  - When: `UnitKilled { killer_player_id: A, killed_player_id: B }` event written; `handle_kill_award` runs
  - Then: `PlayerEconomies[A].gold == 6`; `S2CGoldUpdate` and `S2CGoldBroadcast` enqueued for A

- **EC17: Objective award (non-self)**
  - Given: Player A: `gold = 5`; `config.objective_gold_reward = 3`
  - When: `ObjectiveDestroyed { attacker: A, defender: B, was_fake: false, reward_type: None }` written
  - Then: `PlayerEconomies[A].gold == 8`; B's gold unchanged

- **EC11: Self-inflicted no-award**
  - Given: Player A: `gold = 5`
  - When: `ObjectiveDestroyed { attacker: A, defender: A, was_fake: false, reward_type: None }` written
  - Then: `PlayerEconomies[A].gold == 5` (unchanged); no `S2CGoldUpdate` enqueued for this event

- **EC9: mana_cap increment via fake reward**
  - Given: Player A: `mana_cap = 10`, `gold = 3`; `config.mana_cap_max = 12`
  - When: `ObjectiveDestroyed { attacker: A, defender: B, was_fake: true, reward_type: Some(ManaCap) }` written
  - Then: `PlayerEconomies[A].mana_cap == 11`; `S2CGoldUpdate` for A includes `mana_cap = 11`

- **Dual kill: two kills in one frame**
  - Given: Player A: `gold = 3`; two `UnitKilled` events both with `killer = A`
  - When: `handle_kill_award` processes both in one `update()`
  - Then: `PlayerEconomies[A].gold == 5` (3 + 1 + 1)

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/economy/awards_test.rs` — covers EC11, EC16, EC17, mana_cap increment via fake reward, dual-kill accumulation, self-inflicted no-award
**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 (API functions `apply_gold_award`, `increment_mana_cap` must exist)
- Depends on: Story 002 (`system.rs` and `plugin.rs` established; `S2CGoldUpdate`/`S2CGoldBroadcast` event types registered)
- Depends on: RSM epic (for scheduling context — `advance_phase` label must be available)
- Note: `UnitKilled` and `ObjectiveDestroyed` production event types are defined by Combat Resolution (M2) and Objective System epics respectively. This story's unit tests use stubs; production wiring requires those epics to be complete.
- Unlocks: M2 Combat Resolution (can subscribe to kill events knowing Economy handler is ready); Objective System (can emit `ObjectiveDestroyed` knowing Economy will respond correctly)
