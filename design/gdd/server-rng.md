# Server-side RNG

> **Status**: Approved (post-revision)
> **Author**: User + Agents
> **Last Updated**: 2026-04-29
> **Implements Pillar**: Simple surface — zero cognitive load for systems that need randomness

## Overview

The Server-side RNG system is the single source of randomness for all non-deterministic events in Lanes and Lies. It owns one `ChaCha20Rng` instance per game session, seeded from OS entropy at game start, and exposes a single operation — `next_seed() → u64` — that all other systems call when they need randomness. No system stores its own RNG source; they consume a seed, perform their draw locally, and discard the seed. All results (dice rolls, shop cards, objective rewards) are computed server-side and broadcast to clients as final values — seeds and RNG state are never transmitted. This design ensures every game event is deterministic given the initial seed sequence, impossible to predict client-side, and trivially auditable server-side.

## Player Fantasy

`ServerRng` is infrastructure with no direct player-facing behavior. Players never see RNG state or seeds. They experience the randomness it enables: the Ecaflip card that flips, the shop that shows an unexpected Rare, the fake objective reward that grants a mana cap boost. The fairness guarantee — that all randomness is computed by the server and the client cannot predict or manipulate it — is invisible to players who never cheat and non-negotiable to those who might try.

## Detailed Rules

### Core Rules

**1. One RNG per game session**
- `ServerRng` wraps a single `ChaCha20Rng` instance (from `rand_chacha 0.3`)
- Initialized once at game-session start using `ChaCha20Rng::from_entropy()` (OS entropy via `OsRng`)
- Stored as a Bevy `Resource`: `ResMut<ServerRng>` for callers that need randomness
- Destroyed when the game session ends. There is no persistence between sessions.

**2. Single public operation: `next_seed() → u64`**
- Advances the internal RNG state and returns one `u64` value
- Callers use this `u64` to seed their own local computation (e.g., `ChaChaRng::seed_from_u64(seed)` for a weighted draw)
- The `u64` is consumed by the caller; it is never stored, logged (as a seed), or transmitted to clients
- Each `next_seed()` call corresponds to exactly one random event. Multi-trigger events (e.g., an Ecaflip card that rolls twice on appearance) call `next_seed()` once per trigger — no sub-seeding

**3. Callers own their domain computation**
`ServerRng` knows nothing about card draws, dice ranges, or game rules. Callers that consume seeds:

| Caller | Event | Phase | Seeds consumed |
|---|---|---|---|
| Objective System | Initial fake-lane assignment (2 fakes from 5 lanes) | DRAFT_INITIAL | 2 per player: seed₁ → `gen_range(0..5)` picks first fake lane index; seed₂ → `gen_range(0..4)` picks second from the remaining 4 |
| Card Data & Pool | Initial draft draw (9 cards from class+neutral pool) | DRAFT_INITIAL | 1 per player — single seed drives all 9 draws internally via sub-RNG |
| Card Data & Pool | Shop slot draw — class-side result | SHOP | 2 per slot: seed₁ = Phase 1 split roll (`gen_range(0..2)` → class), seed₂ = Phase 2 weighted card pick (Formula 1b) |
| Card Data & Pool | Shop slot draw — neutral-side result | SHOP | 3 per slot: seed₁ = Phase 1 split roll (`gen_range(0..2)` → neutral), seed₂ = Phase 2 weighted family pick (Formula 1b), seed₃ = Phase 3 uniform card within family (`gen_range(0..family_size)`) |
| Card Data & Pool | Auction card draw | SHOP | 1 per auction round per player |
| Card Data & Pool | draw_random (prism Lane 3, draw effects) | RESOLUTION | 1 per draw event — uniform pick from filtered eligible subset (no weighting). **Exception:** 0 seeds consumed if the collecting player's hand is full at collection time (Prism Lane 3 only) — the hand-full pre-check in Prism System Rule 5 step 1 short-circuits before `next_seed()` is called. Replay tools must not assume a fixed 1 seed per Lane 3 `PrismCollected` event. |
| Combat Resolution | Ecaflip dice trigger | RESOLUTION | 1 per trigger instance |
| Combat Resolution | Ecaflip coin flip | RESOLUTION | 1 per flip instance |
| Objective System | Fake objective reward (mana cap or card pick) | RESOLUTION | 1 per fake destroyed |

**Expected total seeds per round (2-player 1v1, typical):**
- RESOLUTION: ~10–20 (varies with Ecaflip trigger count; 0 if no Ecaflip units active)
- SHOP: ~15–17 (3 slots × ~2.5 avg seeds × 2 players; +2 for auction draw on auction rounds)
- DRAFT_INITIAL (round 0 only): 8 (2 players × 2 fake-assign seeds + 2 players × 1 draft seed)

**Ecaflip Spell cards:** Specific Ecaflip Spell cards with dice/coin effects (e.g., Craps, Dé du Chateux, Shava Shavien) will be added as additional rows under Combat Resolution when the Combat Resolution GDD is authored. This table is the exhaustive inventory of all `next_seed()` callers; new random events require adding a row here first.

**4. Broadcast rule — results only, never seeds**
- After consuming a seed and computing a result, the result is broadcast to all clients via an S2C Lightyear message
- Seeds and RNG state are never included in any network message
- Clients are views: they display what the server tells them

**5. Fixed execution order — per phase**
All systems that call `next_seed()` must be scheduled with explicit ordering constraints within their respective Bevy state schedule. Because game phases run as separate Bevy `States`, ordering constraints (`.before()`/`.after()`) are expressed within a single phase's schedule — cross-phase ordering is enforced by the Round State Machine's state transitions, not by system ordering.

**DRAFT_INITIAL phase (once per game, before round 1):**
```
assign_fake_objectives  →  draw_initial_draft
```

**RESOLUTION phase (each round):**
```
apply_placement_effects  →  resolve_ecaflip_triggers  →  resolve_prism_draws  →  award_fake_objective_rewards
```

**SHOP/AUCTION phase (each round):**
```
generate_shop_slots  →  draw_auction_card
```
(`draw_auction_card` is skipped on non-auction rounds. Manual shop refreshes call `generate_shop_slots` mid-phase; each refresh is ordered after the initial auto-refresh by server receipt time.)

Bevy's scheduler may run same-priority systems in any order — absence of explicit ordering makes the seed sequence non-deterministic. Any system calling `next_seed()` that does not appear in the above chains is a correctness error, not an edge case.

**6. Inter-player ordering**
When multiple players have simultaneous random events within the same system call (e.g., both players have Ecaflip triggers resolving in the same sub-step), events are processed in this deterministic order:

1. **Ascending `player_id`** — the server-assigned numeric ID; lower ID processes first
2. Within one player's events: **ascending lane index** (lane 1 → 5)
3. Within a lane: **ascending board position** (cell 1 → 8, from spawn cell toward opponent)

This ordering applies to all per-player concurrent events and must be implemented consistently across all consuming systems. Without it, two correct server implementations of this GDD produce different audit logs for identical game states.

**7. Session-forfeit on server crash**
If the server process terminates mid-round, the game session is forfeit. All connected clients receive a disconnect notification. There is no reconnect-and-resume: a server restart begins a new game session with a new RNG.

*Scope note: Session-forfeit is an explicit hackathon scope tradeoff, not a technical constraint. Post-launch resumption is feasible — the audit log already contains the `seed_index` needed to continue from the crash point, and persisting the root seed at session start would enable a reconnecting server to advance the ChaCha20 stream to the last confirmed event. Resumption support is deferred to post-launch.*

**8. Server-side audit log**
Each `next_seed()` call appends one `AuditEntry` to the server session log:

```rust
AuditEntry {
    event_type: &'static str,  // identifies the random event (see table below)
    seed_index: u32,           // monotonically increasing counter starting at 0 per session
    result: Option<String>,    // human-readable result; None for empty-pool draws
}
```

**Result encoding by event type:**

| event_type | result | Example |
|---|---|---|
| `"session_init"` | `None` | First entry; logged once at session start |
| `"fake_lane_assign"` | `Some("3")` | 0-based lane index assigned as fake |
| `"shop_split"` | `Some("0")` or `Some("1")` | 0 = class-side, 1 = neutral-side |
| `"shop_weighted_pick"` | `Some("0.734")` | Raw CDF float used for weighted family/card selection |
| `"shop_family_card"` | `Some("gobball_soldier")` | CardId picked uniformly within selected neutral family |
| `"shop_class_card"` | `Some("iop_saber_knight")` | CardId for class-side weighted pick |
| `"ecaflip_dice"` | `Some("4")` | Value in {1, 2, 3, 4, 5, 6} |
| `"ecaflip_coin"` | `Some("1")` | 0 = heads, 1 = tails |
| `"fake_reward"` | `Some("0")` | 0 = mana cap +1, 1 = free card pick |
| `"auction_draw"` | `Some("cra_bow_meow")` | CardId |
| `"draw_random"` | `Some("neutral_mushroom")` | CardId |
| Any empty-pool draw | `None` | No eligible cards in subset |

`seed_index` starts at 0; entry 0 is always `("session_init", 0, None)`. Given the root seed (retained server-side only, never transmitted), any event can be reproduced by seeding a ChaCha20 stream and advancing to the recorded `seed_index`.

---

### States and Transitions

| State | Description | Valid transitions |
|---|---|---|
| `Uninitialized` | Before game session starts; `ServerRng` resource not yet inserted | → `Active` (game session starts; `ChaCha20Rng::from_entropy()` called) |
| `Active` | `ResMut<ServerRng>` available; `next_seed()` may be called | → `Destroyed` (game session ends normally or server crash) |

---

### Interactions with Other Systems

| System | Direction | Interface |
|---|---|---|
| **Card Data & Pool** | Consumes seeds | Calls `next_seed()` before each `draw_*(seed)` call |
| **Combat Resolution** | Consumes seeds | Calls `next_seed()` before each Ecaflip dice roll or coin flip |
| **Objective System** | Consumes seeds | Calls `next_seed()` before each fake objective reward draw |
| **Round State Machine** | Owns frame ordering | Schedules all seed-consuming systems in the fixed order from Rule 5 |
| **Lightyear (network)** | Receives results (not seeds) | Consuming systems broadcast results after seed use; `ServerRng` itself sends nothing |

## Formulas

`ServerRng` contains no game logic formulas. It produces raw entropy; consuming systems define their own domain formulas. This section documents the standard caller pattern and the seed index accounting.

---

**Formula 1: Bounded value from seed (caller pattern)**

All callers that need a bounded integer result from a seed must use this pattern:

```
bounded_value(seed, min, max_exclusive) =
    ChaCha20Rng::seed_from_u64(seed).gen_range(min..max_exclusive)
```

**Variables:**

| Variable | Type | Description |
|---|---|---|
| `seed` | u64 | Value returned by `next_seed()` for this event |
| `min` | T (integer) | Inclusive lower bound (e.g., 1 for a d6) |
| `max_exclusive` | T (integer) | Exclusive upper bound (e.g., 7 for a d6) |
| output | T (integer) | Uniformly distributed in [min, max_exclusive) |
| **PRECONDITION** | — | `min < max_exclusive` required. Calling with `min >= max_exclusive` panics at runtime (`rand` crate contract). Callers must verify the eligible set is non-empty before constructing the range. |

**Type restriction:** `T` must be an integer type. For weighted probability selection (shop draws), use Formula 1b below.

**Examples:**
- d6 roll: `gen_range(1..7)` → output ∈ {1, 2, 3, 4, 5, 6}
- Coin flip: `gen_range(0..2)` → 0 = heads, 1 = tails
- 50/50 fake objective reward: `gen_range(0..2)` → 0 = mana cap +1, 1 = free card pick
- 50/50 shop split roll: `gen_range(0..2)` → 0 = class-side, 1 = neutral-side
- Fake lane assignment (first): `gen_range(0..5)` → lane index 0–4
- Fake lane assignment (second): `gen_range(0..4)` → index into remaining 4 lanes

No caller may derive randomness from the seed by any other method (e.g., bitmasking, modulo) unless documented in Formula 1b.

---

**Formula 1b: Weighted selection from seed (caller pattern)**

For Phase 2 weighted draws in `draw_shop_slot` (see card-data-pool.md Formula 2 Phase 2):

```
weighted_pick(seed, eligible_types, normalized_weights) =
    let u = ChaCha20Rng::seed_from_u64(seed).gen::<f64>();
    find first t in eligible_types (sorted deterministically, e.g., by card_id ascending)
        where cumulative_normalized_weight(t) >= u
```

**Variables:**

| Variable | Type | Description |
|---|---|---|
| `seed` | u64 | Value returned by `next_seed()` for this event |
| `u` | f64 | Uniform [0.0, 1.0) draw via `gen::<f64>()` |
| `eligible_types` | sorted set | Types with non-zero normalized weight; **precondition: ≥1 member** (guaranteed by card-data-pool.md Formula 2 fallback logic before calling) |
| `normalized_weights` | f64 per type | Probabilities summing to 1.0 from card-data-pool.md Formula 2 |
| output | type element | One element from eligible_types |

**Note:** Formula 1b is the only documented exception to "use `gen_range`." It uses `gen::<f64>()` for CDF-based weighted selection. No other method may be used to derive randomness from a seed.

---

**Formula 2: Seed index (audit counter)**

```
seed_index(n) = n
```

| Variable | Type | Description |
|---|---|---|
| `n` | u32 | Monotonically increasing counter starting at 0; incremented by 1 on each `next_seed()` call |

**Output range:** 0 to u32::MAX (4,294,967,295). Not reachable in a normal game session.

**Audit log entry per event:** `AuditEntry { event_type: &'static str, seed_index: u32, result: Option<String> }` — see Rule 8 for field encoding.

Given the root seed (logged at session start, server-side only), any single event can be reproduced by advancing the ChaCha20 stream to position `seed_index` and applying the caller's bounded formula.

## Edge Cases

- **`next_seed()` called but draw returns `None` (depleted pool):** The seed IS consumed and `seed_index` advances. The caller discards the seed when no eligible result exists. This is mandatory — consuming the seed even on a no-op result keeps the seed index consistent with the audit log. Logged as `AuditEntry { event_type, seed_index, result: None }`.

- **Ecaflip card with multiple dice triggers in one resolution sub-step:** Each trigger instance constitutes one `next_seed()` call. Two triggers → two `seed_index` increments. The ordering of calls within `resolve_ecaflip_triggers` follows Rule 6: ascending `player_id`, then ascending lane index, then ascending board position. This rule applies to all per-player concurrent events, not just Ecaflip.

- **Two Bevy systems attempt concurrent access to `ResMut<ServerRng>`:** Prevented by Bevy's scheduler. A system holding `ResMut<ServerRng>` has exclusive access; any other system that also takes `ResMut<ServerRng>` is scheduled sequentially. No runtime lock contention — the conflict is detected at schedule-build time.

- **New system calls `next_seed()` without explicit ordering in Rules 5–6:** Programming error. Its seeds are inserted at an undefined position in the phase sequence, making the session audit log non-reproducible. All callers must appear in the fixed execution chains (Rule 5) with inter-player ordering applied (Rule 6); omission is a correctness bug and must be fixed, not handled gracefully.

- **Server crash mid-round:** Session is forfeit. Clients receive a disconnect notification. The partial audit log up to the crash point is preserved if the log was flushed. No reconnect-and-resume — a new connection starts a new session with a new RNG.

- **`seed_index` overflow (wraps past `u32::MAX`):** Requires ~4.3 billion random events per session — not reachable in practice. If it occurs, `seed_index` wraps to 0. The session log must note the wrap-around to prevent audit confusion.

- **Client disputes a random result:** The server audit log (Rule 8) is the authoritative record. There is no client-side dispute resolution; the result is final. Post-game verification is possible by replaying the ChaCha20 stream from the root seed (server-side only; the root seed is never transmitted to clients).

- **`next_seed()` called before session starts (`Uninitialized` state):** Programming error. `ServerRng` resource is not yet inserted. All seed-consuming systems must be gated on the `Active` game session state via a run condition — they must not attempt `ResMut<ServerRng>` access until the session is active.

## Dependencies

| System | Relationship | Interface |
|---|---|---|
| **OS / OsRng** | Hard upstream | Provides entropy for initial seeding via `ChaCha20Rng::from_entropy()` |
| **Card Data & Pool** | Downstream (hard) | Calls `next_seed()` before each `draw_*(seed: u64)` call |
| **Combat Resolution** | Downstream (hard) | Calls `next_seed()` before each Ecaflip dice roll and coin flip |
| **Objective System** | Downstream (hard) | Calls `next_seed()` for initial fake-lane assignment at game start (2 seeds per player) and before each fake objective reward draw |
| **Card Acquisition** | Downstream (hard) | Calls `next_seed()` for shop draws (via Card Data & Pool queries) |
| **Auction System** | Downstream (soft) | Calls `next_seed()` for auction card draw (via Card Data & Pool) |
| **Round State Machine** | Downstream (coordination) | Owns and enforces the fixed execution order (Rule 5) and inter-player ordering (Rule 6) of all seed-consuming systems |
| **Lightyear (network)** | Downstream (indirect) | Consuming systems broadcast results after seed use; `ServerRng` sends nothing directly |

**Bidirectionality:** `card-data-pool.md` lists Server-side RNG as a hard upstream dependency ✓. All other downstream GDDs are not yet authored — they must list Server-side RNG as a dependency when written.

## Tuning Knobs

`ServerRng` has no designer-adjustable tuning knobs. The ChaCha20 algorithm is fixed — it is not parameterizable at the game design level. The seeding strategy (OS entropy) is an architectural decision. No fields from this system appear in `GameConfig`.

## Visual/Audio Requirements

None. `ServerRng` is a server-side utility with no visual or audio output.

## UI Requirements

None. RNG results are surfaced via the consuming system's UI (e.g., Ecaflip dice result in Combat Resolution UI). `ServerRng` itself has no UI surface.

## Acceptance Criteria

### Core Behavior

| # | Criterion | Type |
|---|---|---|
| RNG1 | **GIVEN** a new game session starts, **WHEN** `ServerRng` is initialized via `ChaCha20Rng::from_entropy()`, **THEN** `world.contains_resource::<ServerRng>()` returns true and `server_rng.seed_index()` equals 0. | BLOCKING |
| RNG2 | **GIVEN** two `ServerRng` instances initialized with distinct known seeds via a test-only `ServerRng::from_seed(u64)` constructor (e.g., seed A=1 and seed B=2), **WHEN** `next_seed()` is called once on each, **THEN** the two returned values differ. *(Guards against constant-output bugs; tests that the seed input propagates to the RNG output.)* | ADVISORY |
| RNG5 | **GIVEN** `next_seed()` is called N times in a session, **WHEN** `server_rng.audit_log()` is inspected, **THEN** exactly N+1 entries exist (entry 0 is `session_init`), with `seed_index` values 0 through N in ascending order. | BLOCKING |
| RNG6 | **GIVEN** a draw event where the eligible subset is empty and the draw returns `None`, **WHEN** `next_seed()` is called for that event, **THEN** `seed_index` increments by 1 and `audit_log().last().result` is `None`. | BLOCKING |
| RNG7 | **GIVEN** an Ecaflip card with two dice triggers resolving in the same sub-step, **WHEN** both triggers are processed, **THEN** `server_rng.call_count()` (or equivalent spy) equals 2, the first audit entry has `seed_index` K and `event_type` `"ecaflip_dice"`, and the second has `seed_index` K+1 and `event_type` `"ecaflip_dice"`. | BLOCKING |

*(Note: tests for `gen_range` output bounds belong in the Combat Resolution and Objective System test suites, which own those formulas — not here.)*

### Scheduling & Ordering

| # | Criterion | Type |
|---|---|---|
| RNG8 | **GIVEN** a Bevy App with the full RESOLUTION chain registered per Rule 5 (`apply_placement_effects → resolve_ecaflip_triggers → resolve_prism_draws → award_fake_objective_rewards`), **WHEN** `app.update()` runs one test round, **THEN** execution-order test hooks record the systems running in that exact sequence and `audit_log()` entries appear in the same order. *(Requires test hook infrastructure to be defined before sprint start.)* | BLOCKING |
| RNG9 | **GIVEN** the game session is not in `Active` state (`ServerRng` not yet inserted), **WHEN** a seed-consuming system's run condition is evaluated, **THEN** the system does not run (run condition returns false), preventing any `ResMut<ServerRng>` access. *(Tests the guard mechanism — not Bevy's missing-resource panic.)* | BLOCKING |
| RNG12 | **GIVEN** one player has Ecaflip triggers in lane 1 (position 2) and lane 3 (position 1) resolving in the same sub-step, **WHEN** `resolve_ecaflip_triggers` runs, **THEN** the audit log records the lane 1 trigger at seed_index K and the lane 3 trigger at seed_index K+1, confirming Rule 6 lane-ordering. | BLOCKING |
| RNG13 | **GIVEN** session A ends (`ServerRng` resource removed) and session B starts (`ServerRng` re-inserted), **WHEN** `next_seed()` is called once in session B, **THEN** `audit_log()[0].event_type == "session_init"` and `audit_log()[1].seed_index == 0` (counter resets). | BLOCKING |

### Distribution (Statistical)

| # | Criterion | Type |
|---|---|---|
| RNG10 | **GIVEN** 10,000 calls to `next_seed()` each used to produce `gen_range(1..7)`, **WHEN** a chi-squared uniformity test is applied (5 degrees of freedom), **THEN** the chi-squared statistic is below the critical value at p = 0.01 (χ² < 15.09). | ADVISORY |

### Audit Integrity

| # | Criterion | Type |
|---|---|---|
| RNG11 | **GIVEN** a game session starts and `ServerRng` is initialized, **WHEN** `audit_log()[0]` is inspected, **THEN** `event_type == "session_init"` and `result == None`. The root seed is NOT present in any audit entry. | BLOCKING |
| RNG14 | **GIVEN** a complete round resolves with N random events, **WHEN** all S2C Lightyear messages broadcast during that round are captured by a test network listener, **THEN** none contain a byte sequence matching any raw `u64` seed value returned by `next_seed()` in that round. | BLOCKING |
| RNG15 | **GIVEN** a `ServerRng` with `seed_index` at `u32::MAX` (via test-only constructor), **WHEN** `next_seed()` is called, **THEN** `seed_index` wraps to 0 without panic and the audit log records the wrap. | ADVISORY |

## Open Questions

| # | Question | Owner | Priority |
|---|---|---|---|
| OQ1 | Should the session audit log be in-memory only (lost on server restart) or written to disk? In-memory is sufficient for a hackathon; disk persistence enables post-launch cheat investigation. | Lead Programmer | Before Polish phase |
| OQ2 | Verify that `rand_chacha 0.3` is compatible with `rand 0.9` — confirm version pairing on crates.io before implementation. | Lead Programmer | Before sprint start |
