# Story 008: Ecaflip RNG Effects — Dé du Chateux and Coin Flip

> **Epic**: Class System
> **Status**: Ready
> **Layer**: Feature (M3)
> **Type**: Logic
> **Manifest Version**: 2026-04-30

## Context

**GDD**: `design/gdd/class-system.md`
**Requirement**: `TR-CS-007`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-014: Class System Architecture — PlayerSessionState, SourceClass Component, and Direct Effect Dispatch
**ADR Decision Summary**: Class effects are plain Rust functions. Ecaflip dice effects consume from the RESOLUTION RNG chain per server-rng.md Rule 5 (ordering: ascending player_id → lane → trigger_index_within_card). No class-private RNG. Results broadcast as `S2CResolutionEvent` outcome variants.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: MEDIUM
**Engine Notes**:
- All Ecaflip RNG consumes from `ServerRng` (ChaCha20 RESOLUTION chain) — never `rand::thread_rng()` or any other RNG source.
- **Test setup** (from GDD ACs 22/23/24/25): Extract resolution logic as pure functions taking `roll: u8` or `flip: u8` directly (preferred for unit tests). Do NOT seed the ChaCha RNG with a fixed seed to produce known values — extract the pure formula function and pass the roll/flip value directly. This is the only correct approach for deterministic unit tests.
- `S2CSingleObjectiveReveal` (NP-4, Dé du Chateux single-lane reveal) may not be registered in NP GDD yet. Unit test asserts the formula output (reveal=true/false, damage amount); integration test requires NP-4 resolution.
- `alive == 0` division guard: use explicit `if alive == 0 { return no_op; }` before `share = total / alive` — prevents integer division-by-zero panic in Rust (GDD CS-10 edge case).
- ADR-014 is NOT yet in the control manifest.

**Control Manifest Rules (Feature Layer)**:
- Required: All game randomness uses `ServerRng` (ChaCha20) only — ADR-005
- Required: Every RNG draw writes an `AuditEntry` in the same call — ADR-005
- Forbidden: Never use `rand::thread_rng()`, `StdRng`, or `SmallRng` in server game logic — ADR-005
- Forbidden: Never transmit RNG seeds or audit_log in any S2C message — ADR-005
- Guardrail: RESOLUTION batch budget ≤ 15ms — ADR-002

---

## Acceptance Criteria

*From GDD `design/gdd/class-system.md`, CS-9 (Dé du Chateux) and CS-10 (Coin flip):*

- [ ] **CS-AC-22** GIVEN Ecaflip player's Dé du Chateux server RNG roll = 2 (test setup: inject roll via pure function), WHEN the effect resolves, THEN 2 damage is dealt to the target AND the enemy objective in the target lane is revealed (unicast to Ecaflip player only; roll ≤ 3).
- [ ] **CS-AC-23** GIVEN Ecaflip player's Dé du Chateux server RNG roll = 5, WHEN the effect resolves, THEN 5 damage is dealt to target AND no objective reveal occurs (roll > 3).
- [ ] **CS-AC-24** GIVEN Ecaflip player's Craps coin flip = heads (8 damage) with 3 alive opponent objectives, WHEN Craps resolves, THEN objectives in lanes 1 and 2 receive 3 damage each and lane 3 receives 2 damage (`floor(8/3)=2, remainder=2` → first 2 lanes get +1).
- [ ] **CS-AC-25** GIVEN Ecaflip player's Shava Shavien dies with coin flip = tails, WHEN the DEATH trigger resolves, THEN the Shava Shavien card enters the **opponent's** hand.

---

## Implementation Notes

*Derived from ADR-014 Decision §4 and GDD Formulas CS-9, CS-10:*

**File location**: `server/src/core/resolution/effects.rs`

**CS-9 — Dé du Chateux**:

Extract the pure formula for testability:
```rust
/// Pure function — takes the pre-computed roll value (from ServerRng externally).
/// Returns (damage, reveal).
pub fn de_du_chateux_outcome(roll: u8, threshold: u32) -> (u32, bool) {
    (roll as u32, roll as u32 <= threshold)
}

/// Integration entry point — called from within RESOLUTION system body.
pub fn apply_de_du_chateux(
    rng: &mut ServerRng,
    config: &GameConfig,
    objectives: &mut ObjectiveState,
    unicast_queue: &mut UnicastQueue,
    player_id: PlayerId,
    target: Target,          // unit or objective + lane info from cards.json
    ecaflip_peer: PeerId,
) {
    let seed = rng.next_seed();  // writes AuditEntry; ResolveEcaflip intent
    let roll = (seed % 6 + 1) as u8;  // uniform [1, 6]
    let (damage, reveal) = de_du_chateux_outcome(roll, config.de_chateux_reveal_threshold);

    // Apply damage via standard take_damage pipeline (unit or objective)
    target.apply_damage(damage, objectives, ...);

    if reveal {
        // Unicast single-objective reveal to Ecaflip player only (NP-4 pending)
        // Until NP-4 resolved: log the reveal intent; defer S2C send
        unicast_queue.push(ecaflip_peer, S2CSingleObjectiveReveal {
            lane: target.lane(),
            is_fake: objectives.is_fake(player_id.opponent(), target.lane()),
        });
    }
}
```

**CS-10 — Coin flip (Chatar / Shava Shavien / Craps)**:
```rust
pub fn coin_flip_outcome(flip: u8) -> CoinFlipResult {
    if flip == 0 { CoinFlipResult::Heads } else { CoinFlipResult::Tails }
}

pub fn apply_craps(
    rng: &mut ServerRng,
    objectives: &mut ObjectiveState,
    player_id: PlayerId,
    total: u32,  // 8 on heads, 4 on tails — determined by coin flip call site
) {
    let alive = objectives.count_alive(player_id.opponent());
    if alive == 0 { return; }  // game already over; no-op (MUST guard before division)
    let share = total / alive;
    let remainder = total % alive;
    let mut lane_order: Vec<u8> = objectives.alive_lanes(player_id.opponent())
        .collect::<Vec<_>>();
    lane_order.sort();  // ascending lane order for remainder distribution
    for (i, lane) in lane_order.iter().enumerate() {
        let dmg = share + if (i as u32) < remainder { 1 } else { 0 };
        objectives.take_damage(player_id.opponent(), *lane, player_id, dmg);
    }
}
```

**Shava Shavien DEATH** (CS-AC-25): On DEATH trigger (flip = tails), `card.move_to_hand(opponent_player_id)`. This is intentional extreme-variance design — not a bug. Implementers must NOT treat tails-to-opponent as an error.

**RNG consumption order** (per ADR-005 Rule RESOLUTION): ResolveEcaflip → ResolvePrism → AwardFakeObjectiveReward → DrawFreeCard. All Ecaflip die/flip calls use the RESOLUTION chain in ascending player_id → lane → trigger_index_within_card order.

---

## Out of Scope

*Handled by neighbouring stories:*

- Story 001: PlayerSessions scaffold
- Server-side RNG: `next_seed()`, `AuditEntry`, consumption order — `server-rng` epic
- Objective System: `take_damage()` for objective targets — `objective-system` epic
- NP-4: `S2CSingleObjectiveReveal` message registration — Network Protocol epic (unicast delivery blocked until NP-4 resolved; formula logic is independently testable)
- Combat Resolution: Chatar `+2 ATK` / `2 self-damage` effect application on units — `combat-resolution` epic
- Card Acquisition: Shava Shavien tails card-to-opponent-hand routing — `card-acquisition` epic

---

## QA Test Cases

*Logic story — use pure function extraction pattern (inject roll/flip directly; do NOT rely on seeded RNG to produce specific values in unit tests).*

- **AC CS-AC-22 — Dé du Chateux roll=2, reveals**:
  - Given: `threshold = 3`
  - When: `de_du_chateux_outcome(roll=2, threshold=3)` called
  - Then: returns `(damage=2, reveal=true)`
  - Edge cases: roll=3 → reveal=true (boundary); roll=4 → reveal=false (just over)

- **AC CS-AC-23 — Dé du Chateux roll=5, no reveal**:
  - Given: `threshold = 3`
  - When: `de_du_chateux_outcome(roll=5, threshold=3)` called
  - Then: returns `(damage=5, reveal=false)`
  - Edge cases: roll=1 → damage=1, reveal=true; roll=6 → damage=6, reveal=false

- **AC CS-AC-24 — Craps distribution: heads (8 dmg), 3 alive objectives**:
  - Given: opponent has 3 alive objectives (lanes 1, 2, 3); total=8
  - When: `apply_craps(rng, objectives, player_id, total=8)` called (pure distribution logic with pre-computed total)
  - Then: lane 1 objective takes 3 dmg; lane 2 takes 3 dmg; lane 3 takes 2 dmg (`floor(8/3)=2, rem=2 → first 2 get +1`)
  - Edge cases: total=4, alive=3 → `floor(4/3)=1, rem=1` → lane 1 gets 2, lanes 2/3 get 1 each; alive=0 → no-op (guard must fire before division)

- **AC CS-AC-25 — Shava Shavien tails → card to opponent hand**:
  - Given: `flip = 1 (tails)`
  - When: Shava Shavien DEATH trigger handler receives `CoinFlipResult::Tails`
  - Then: `card_moved_to_opponent_hand = true`; opponent hand now contains Shava Shavien card
  - Edge cases: flip=0 (heads) → card returns to own hand (NOT opponent's)

- **Craps alive=0 guard**:
  - Given: opponent has 0 alive objectives (game-over edge case)
  - When: `apply_craps(...)` called
  - Then: returns immediately without panicking; no take_damage calls; no division performed

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/class/ecaflip_rng_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 (PlayerSessions) — must be DONE
- Depends on: `server-rng` epic story-001 (ServerRng Resource, next_seed(), AuditEntry) — must be DONE
- Depends on: `objective-system` epic (take_damage, is_alive, count_alive for Craps) — must be DONE for integration
- Depends on: NP-4 resolution (`S2CSingleObjectiveReveal` registration) — blocks Dé du Chateux reveal unicast delivery; formula logic independently unit-testable
- Unlocks: No direct downstream story dependency; completes Ecaflip's class effect suite
