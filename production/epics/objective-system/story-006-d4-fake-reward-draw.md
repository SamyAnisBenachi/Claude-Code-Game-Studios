# Story 006: D4 Fake Reward Draw

> **Epic**: Objective System
> **Status**: Ready
> **Layer**: Feature
> **Type**: Logic
> **Manifest Version**: 2026-04-29

## Context

**GDD**: `design/gdd/objective-system.md`
**Requirement**: `TR-OBJ-005` — Formula D4 fake reward: `RNG.gen_range(0..2)`: `0 = ManaCapIncreased`, `1 = FreeCardPick`; FreeCardPick hand-full fallback = `AwardGold +1` without consuming `draw_random` seed; pool-exhausted = no-op (no re-roll)

**ADR Governing Implementation**: [ADR-001: Hidden Objective Identity via Targeted Unicast, Not Component Replication](docs/architecture/adr-001-objective-identity-unicast.md)

**ADR Decision Summary**: When a fake objective is destroyed by the opponent (after `fake_objectives_destroyed` increments), draw 1 seed from `ServerRng` (`gen_range(0..2)`): outcome 0 emits `ManaCapIncreased { player: attacker, amount: 1 }`; outcome 1 checks the attacker's hand size first — if full (10 cards), emit `AwardGold { player: attacker, amount: 1 }` without consuming the draw seed; otherwise draw a second seed for `draw_random`. The filter passed to `draw_random` is always `FAKE_REWARD_POOL_FILTER` (all-None). If `draw_random` returns `None` (pool exhausted), the result is a silent no-op — do NOT re-roll to ManaCapIncreased.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**: RNG seed consumption order within RESOLUTION is strict (ADR-005): `AwardFakeObjectiveReward` (this story's step) comes after `ResolveEcaflip` and `ResolvePrism`, and before `DrawFreeCard` — when drawing for FreeCardPick, the `draw_random` seed is consumed as `DrawFreeCard` in the consumption log. `MessageWriter<ManaCapIncreased>` and `MessageWriter<AwardGold>` — `EventWriter` does not exist in Bevy 0.17+. `liv-bevy-018` mandatory.

**Control Manifest Rules (Feature layer)**:
- Required: All game randomness through `ServerRng` — never `thread_rng()` (ADR-005)
- Required: Consumption order in RESOLUTION: `ResolveEcaflip` → `ResolvePrism` → `AwardFakeObjectiveReward` → `DrawFreeCard` (ADR-005)
- Required: Feature systems emit `AwardGold` and `ManaCapIncreased` as Messages consumed by Economy System — never call Economy System directly (ADR-010)
- Forbidden: No re-roll from pool-exhausted `None` result to ManaCapIncreased — this would bias the D4 50/50 (GDD edge case)
- Forbidden: Do not consume the `draw_random` seed if hand is full — AwardGold fallback terminates without calling draw_random (GDD OS-15)

---

## Acceptance Criteria

*From GDD `design/gdd/objective-system.md`, scoped to this story:*

- [ ] OS-8 (BLOCKING): GIVEN a fake objective is destroyed by the opponent, WHEN the consequence path runs, THEN `fake_objectives_destroyed(attacker)` increments by 1, AND exactly one of `{ManaCapIncreased, FreeCardPick}` is emitted (not both, not neither).
- [ ] OS-11 (BLOCKING): GIVEN both of a player's fake objectives are destroyed with both D4 draws producing ManaCapIncreased, WHEN the Objective System processes both rewards, THEN `ManaCapIncreased { player, amount: 1 }` is emitted exactly twice. (This story tests the two-emission behavior. The Economy System integration — that `mana_cap_effective = min(mana_cap_base + 2, mana_cap_max)` — is verified in Economy System tests.)
- [ ] OS-12 (BLOCKING): GIVEN a fake objective is destroyed and D4 draws ManaCapIncreased, WHEN `ManaCapIncreased` is emitted, THEN it is emitted exactly once regardless of the current mana cap value. (Ceiling enforcement is the Economy System's job, not this story's.)
- [ ] OS-15 (BLOCKING): GIVEN a FreeCardPick reward fires for a player whose hand is at 10 cards, WHEN the consequence path processes the reward, THEN `draw_random()` is NOT called and `AwardGold { player, amount: 1 }` is emitted instead.
- [ ] OS-19 (BLOCKING): GIVEN a fake objective is destroyed with a known server-side RNG seed producing `gen_range(0..2) = 0`, WHEN the reward draw executes, THEN `ManaCapIncreased` is emitted (not `FreeCardPick`). GIVEN a seed producing `gen_range(0..2) = 1`, THEN `FreeCardPick` path enters (not ManaCapIncreased). (Requires seeded `ChaCha20Rng::seed_from_u64(known_seed)` — not live randomness.)
- [ ] OS-22 (BLOCKING): GIVEN a FreeCardPick reward fires and `draw_random(FAKE_REWARD_POOL_FILTER, seed)` returns `None` (pool exhausted), WHEN the consequence path processes the reward, THEN no card is granted, no gold is emitted, and no re-roll to mana cap occurs. (Distinct from OS-15: pool-exhausted → no-op; hand-full → +1g.)
- [ ] OS-26 (BLOCKING): GIVEN both fake objectives destroyed in the same RESOLUTION with D4 draws producing results 0 (ManaCapIncreased) and 1 (FreeCardPick) respectively, WHEN both rewards are processed, THEN `ManaCapIncreased { player, amount: 1 }` is emitted once AND `draw_random(FAKE_REWARD_POOL_FILTER, seed)` is called once. (Requires two seeded draws — pre-compute seeds before writing the test.)
- [ ] OS-27 (BLOCKING): GIVEN a FreeCardPick reward fires and the attacker's hand is below max capacity, WHEN the draw is issued, THEN the filter used is `FAKE_REWARD_POOL_FILTER` with all fields `None`. Verified by: (a) asserting `FAKE_REWARD_POOL_FILTER` constant has `rarity: None, class: None, card_type: None, max_cost: None`; (b) code review confirms `FAKE_REWARD_POOL_FILTER` is the constant passed to `draw_random` at the call site.

---

## Implementation Notes

*Derived from ADR-001 and ADR-005 Implementation Guidelines:*

Define the filter constant in `server/feature/objective/`:

```rust
// FAKE_REWARD_POOL_FILTER: unfiltered draw — all cards eligible
// OS-27 unit test asserts all fields are None on this constant
pub const FAKE_REWARD_POOL_FILTER: PoolFilter = PoolFilter {
    rarity: None,
    class: None,
    card_type: None,
    max_cost: None,
};
```

Implement `draw_fake_reward`:

```rust
pub fn draw_fake_reward(
    attacker_player: PlayerId,
    rng: &mut ResMut<ServerRng>,
    pool: &mut ResMut<CardPool>,
    hands: &Res<PlayerHands>,
    award_gold_writer: &mut MessageWriter<AwardGold>,
    mana_cap_writer: &mut MessageWriter<ManaCapIncreased>,
) {
    let reward_seed = rng.next_seed();
    let outcome = ChaCha20Rng::seed_from_u64(reward_seed).gen_range(0u32..2);

    match outcome {
        0 => {
            // ManaCapIncreased — OS-12: emit regardless of current cap
            mana_cap_writer.write(ManaCapIncreased { player: attacker_player, amount: 1 });
        }
        1 => {
            let hand_size = hands.get(attacker_player).len();
            if hand_size >= MAX_HAND_SIZE {
                // OS-15: hand full → AwardGold +1; do NOT consume draw seed
                award_gold_writer.write(AwardGold { player: attacker_player, amount: 1 });
                return;
            }
            // Consume the free-card draw seed
            let draw_seed = rng.next_seed();
            match pool.draw_random(FAKE_REWARD_POOL_FILTER, draw_seed) {
                Some(card_id) => {
                    pool.distribute(card_id);
                    hands.add_card(attacker_player, card_id);
                }
                None => {
                    // OS-22: pool exhausted → no-op, no re-roll, no gold fallback
                }
            }
        }
        _ => unreachable!(),
    }
}
```

Pre-compute seed values for OS-19 tests: use `ChaCha20Rng::seed_from_u64(S)` and call `.gen_range(0u32..2)` to find S values that produce 0 or 1 before writing the test. Document these seeds in the test file as named constants (e.g., `const SEED_PRODUCES_MANA_CAP: u64 = ...`).

`MAX_HAND_SIZE = 10` is a compile-time constant from GameConfig defaults (or defined as `const MAX_HAND_SIZE: u32 = 10`).

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- [Story 005]: The routing decision to enter this function (consequence path decides fake + attacker ≠ owner → call draw_fake_reward)
- [Story 007]: Broadcasting `ObjectiveDestroyed` at RESOLUTION-end

---

## QA Test Cases

*Written by qa-lead at story creation. Implement against these — do not invent new test cases.*

- **OS-8**: Exactly one of {ManaCapIncreased, FreeCardPick} per fake destruction
  - Given: `World::new()`, fake objective destroyed by opponent, seeded `ServerRng`, hand below max capacity, non-empty pool
  - When: `draw_fake_reward` called
  - Then: Exactly one of `{mana_cap_events.len() == 1, free_card_drawn == true}` — never both, never neither
  - Edge cases: Test both seed outcomes (0 and 1)

- **OS-11**: Two ManaCapIncreased events when both fakes draw outcome 0
  - Given: Two fake destructions; seeds pre-computed to produce outcome 0 for both
  - When: `draw_fake_reward` called twice
  - Then: `assert_eq!(mana_cap_events.len(), 2)`; both events have `amount: 1`

- **OS-12**: ManaCapIncreased emitted even when cap already at ceiling
  - Given: Player's mana_cap_effective already at 12; seed produces outcome 0
  - When: `draw_fake_reward` called
  - Then: `ManaCapIncreased` is still emitted once; the ceiling enforcement is the Economy System's responsibility (not asserted here)

- **OS-15**: Hand-full → AwardGold +1, draw_random NOT called
  - Given: Player hand has 10 cards; seed produces outcome 1 (FreeCardPick path); mock pool to track calls
  - When: `draw_fake_reward` called
  - Then: `award_gold_events.last() == AwardGold { player, amount: 1 }`; pool draw count = 0 (draw_random not invoked)

- **OS-19**: Seeded RNG determinism — outcome 0 → ManaCapIncreased; outcome 1 → FreeCardPick path
  - Given: Pre-computed seeds `SEED_PRODUCES_MANA_CAP` and `SEED_PRODUCES_FREE_CARD`
  - When: `draw_fake_reward` called with each seed in `ServerRng`
  - Then: Outcome 0 seed → `mana_cap_events.len() == 1`; outcome 1 seed → FreeCardPick branch entered

- **OS-22**: Pool exhausted → no-op, no re-roll
  - Given: Empty pool (all cards distributed); seed produces outcome 1
  - When: `draw_fake_reward` called
  - Then: No card in hand; no `AwardGold` emitted; no `ManaCapIncreased` emitted; `rng.seed_index` advanced by 2 (reward seed + draw seed consumed)

- **OS-26**: Both fakes destroyed in same RESOLUTION — one of each outcome
  - Given: Two fake destructions; seeds pre-computed to produce outcomes 0 then 1
  - When: `draw_fake_reward` called twice
  - Then: `assert_eq!(mana_cap_events.len(), 1)`; `assert_eq!(draw_random_call_count, 1)`

- **OS-27**: `FAKE_REWARD_POOL_FILTER` constant has all-None fields
  - Given: The `FAKE_REWARD_POOL_FILTER` constant in source
  - When: All four fields are accessed
  - Then: `assert_eq!(FAKE_REWARD_POOL_FILTER.rarity, None)`; `assert_eq!(FAKE_REWARD_POOL_FILTER.class, None)`; `assert_eq!(FAKE_REWARD_POOL_FILTER.card_type, None)`; `assert_eq!(FAKE_REWARD_POOL_FILTER.max_cost, None)`

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/objective/fake_reward_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 005 must be DONE (the consequence path decides to call `draw_fake_reward` — this function's entry point lives in Story 005's routing logic)
- Unlocks: Story 007 (all reward draw logic is complete before RESOLUTION-end sync is implemented)
