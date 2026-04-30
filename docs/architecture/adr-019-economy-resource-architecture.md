# ADR-019: Economy System Resource Architecture

## Status

Proposed

## Date

2026-04-30

## Engine Compatibility

| Field | Value |
|-------|-------|
| **Engine** | Bevy 0.18 |
| **Domain** | Core / ECS Resource |
| **Knowledge Risk** | HIGH — Bevy 0.15–0.18 all post-cutoff |
| **References Consulted** | `docs/engine-reference/bevy/VERSION.md`, `docs/engine-reference/bevy/breaking-changes.md`, `docs/engine-reference/bevy/deprecated-apis.md`, ADR-002, ADR-009, ADR-010, ADR-013, ADR-017 |
| **Post-Cutoff APIs Used** | `#[derive(Resource)]` (stable); `Res<T>` / `ResMut<T>` (stable); `MessageReader<T>` / `MessageWriter<T>` (Bevy 0.17+ — replaces removed `EventReader`/`EventWriter`); `#[derive(Message)]` + `app.add_message::<T>()` (Bevy 0.17+); `On<SessionReady>` Observer trigger (Bevy 0.17+) |
| **Post-Cutoff APIs NOT Used** | `EventWriter<T>`, `EventReader<T>`, `Events<T>` — these no longer exist in Bevy 0.17+. Do not use them. |
| **Verification Required** | (1) `MessageReader<ResolutionComplete>` per-reader cursor: confirmed — each `MessageReader<T>` maintains an independent cursor (same model as the old `EventReader`); two independent readers in the same frame both observe the message without loss. (2) **UNRESOLVED — BLOCKING for Accepted status**: Confirm the correct pattern for emitting a `#[derive(Message)]` type from an exclusive system (`fn resolve_combat(world: &mut World)`). `MessageWriter<T>` is a system parameter and may not be directly accessible as a named `Resource` via `world.resource_mut()`. Candidate resolution: use a `PendingResolutionComplete(bool)` buffer resource set by `resolve_combat`, drained by a thin regular system using `MessageWriter<ResolutionComplete>` in the same frame. Must be verified against Bevy 0.18 engine-reference before this ADR moves to Accepted. (3) Confirm `EconomySystemSet::ResolutionEnd.before(RsmSystemSet::InputReader)` (SystemSet form, not function reference) prevents the RSM from transitioning to DRAFT before the interest snapshot is captured. |

## ADR Dependencies

| Field | Value |
|-------|-------|
| **Depends On** | ADR-002 (server authority — all economy state is server-only); ADR-009 (RSM phase state — `RoundState` drives `DraftStarted` and `ResolutionPhaseEntered` triggers); ADR-010 (RSM event bus — `DraftStarted` and `ResolutionComplete` are the triggers economy systems read); ADR-013 (auction system state — established that `auction_tick_system` gets direct `ResMut<PlayerEconomies>` access for reservation atomicity); ADR-017 (combat resolution — `resolve_combat` exclusive system is the source of kill/objective gold awards and the emitter of `ResolutionComplete`) |
| **Enables** | Card Acquisition / Shop epic stories that call `can_afford_shop` + `spend_gold`; Class System epic stories that call `add_reserve` and `increment_mana_cap`; Objective System stories that call `award_gold`; HUD projection stories that read `S2CGoldUpdate` / `S2CGoldBroadcast` |
| **Blocks** | Any story that awards gold, spends gold, applies mana, or reads economy state — all must implement against the interfaces defined here |
| **Ordering Note** | ADR-013 is Accepted — the `reserved_gold` field and `reserve_gold` / `release_gold_reservation` API are already load-bearing for the auction system. ADR-017 must be Accepted before the snapshot timing migration in the Migration Plan can be completed (it depends on `resolve_combat` emitting `ResolutionComplete`). |

## Context

### Problem Statement

The Economy System manages three currency pools per player (`gold`, `current_mana`, `reserve_mana`) plus `mana_cap` and a `reserved_gold` bid reservation. Five separate systems need to read or write economy state in the same game session: the economy system itself (phase-driven income and mana ramp), the auction system (gold reservation atomicity), combat resolution (kill/objective gold awards during RESOLUTION sub-steps), the objective system (objective destruction gold award), and the class system (reserve mana manipulation via Xelor spells). Without a formal decision on how state is structured and who may write it, independent implementers will create conflicting patterns — some using `ResMut<PlayerEconomies>` directly, others reading fields without going through the validated API, causing both correctness bugs and unauditable write paths.

A secondary problem is **interest snapshot timing**. The GDD requires the interest snapshot to be taken at the *end* of RESOLUTION — after all kill and objective gold rewards have fired. The current implementation reads `ResolutionPhaseEntered` to take the snapshot, which fires at the *start* of RESOLUTION before any gold awards. This causes the snapshot to miss all in-round kill and objective gold, understating the next round's interest income. This ADR formalizes the correct trigger.

### Constraints

- **Server authority** (ADR-002): `PlayerEconomies` and `InterestSnapshots` are server-only `Resource` types. They must never appear in `protocol/` or `client/` crates.
- **Auction atomicity** (ADR-013): `release_gold_reservation(prev_leader)` and `reserve_gold(new_leader)` must execute in a single system body with no frame gap between them. Cross-system messaging for reservation would allow a one-frame window where two players hold simultaneous non-zero `reserved_gold`, violating the GDD Rule 5 invariant.
- **Combat resolution exclusivity** (ADR-017): `resolve_combat` is an exclusive Bevy system (`fn resolve_combat(world: &mut World)`) that runs outside the regular `Update` schedule. Economy state is mutated inside this system body via `world.resource_mut::<PlayerEconomies>()` + api.rs functions.
- **No direct field mutation**: All writes to `PlayerEconomy` fields must go through `server/src/core/economy/api.rs` module functions. Direct field assignment (`economy.gold = x`) outside `api.rs` is forbidden.
- **Bevy 0.17+ event split**: Economy systems use `MessageReader<T>` + `#[derive(Message)]` for all buffered phase triggers. `EventReader<T>` does not exist in Bevy 0.17+.

### Requirements

- A single authoritative resource (`PlayerEconomies`) owns all per-player currency state for a session.
- A second resource (`InterestSnapshots`) holds the end-of-RESOLUTION gold values used to compute next-round interest — separate from `PlayerEconomies` to make the snapshot lifecycle explicit.
- All `PlayerEconomy` field mutations go through validated API functions in `api.rs`. No other code assigns to `PlayerEconomy` fields directly.
- The interest snapshot is taken after `ResolutionComplete` is emitted (after all kill/objective gold awards), not at `ResolutionPhaseEntered` (RESOLUTION start).
- Economy systems that read `ResolutionComplete` are scheduled before `rsm_input_reader` — preventing the RSM from transitioning to DRAFT before the snapshot is captured.

## Decision

The Economy System's authoritative state lives in two Bevy `Resource` types on the server: `PlayerEconomies` (the live currency state per player) and `InterestSnapshots` (a transient snapshot of end-of-RESOLUTION gold for interest calculation). All mutations go through pure Rust functions in `api.rs`. Write access is granted to three categories of code under strict boundary rules: the economy's own Bevy systems (for phase-driven operations), `auction_tick_system` directly (for reservation atomicity), and `resolve_combat` via `world.resource_mut()` (for in-RESOLUTION gold awards).

The interest snapshot trigger is migrated from `ResolutionPhaseEntered` (RESOLUTION start) to `ResolutionComplete` (RESOLUTION end, after all sub-steps including kills and objective rewards).

### Architecture Diagram

```
SERVER WORLD
┌──────────────────────────────────────────────────────────────┐
│  PlayerEconomies (Resource — server only)                    │
│    HashMap<PlayerId, PlayerEconomy>                          │
│    PlayerEconomy {                                           │
│      gold: u32,           // persists across rounds          │
│      current_mana: u32,   // reset each DRAFT                │
│      reserve_mana: u32,   // persists, no cap                │
│      mana_cap: u32,       // 10–12, per GameConfig           │
│      reserved_gold: u32,  // active auction bid reservation  │
│    }                                                         │
│                                                              │
│  InterestSnapshots (Resource — server only)                  │
│    HashMap<PlayerId, u32>  // gold at RESOLUTION end         │
│    lifecycle: written at ResolutionComplete                  │
│               consumed at DraftStarted (removed on use)      │
└──────────────────────────────────────────────────────────────┘

WRITE ACCESS RULES
┌──────────────────────────────────────────────────────────────┐
│  initialise_player_economies  (Observer: On<SessionReady>)   │
│    ResMut<PlayerEconomies> — sets initial gold/mana/cap      │
│                                                              │
│  on_draft_started  (MessageReader<DraftStarted>)             │
│    ResMut<PlayerEconomies> — apply_mana_ramp + apply_income  │
│    ResMut<InterestSnapshots> — consume snapshot for interest  │
│                                                              │
│  on_resolution_complete  (MessageReader<ResolutionComplete>) │
│    ResMut<PlayerEconomies> — discard_current_mana            │
│    ResMut<InterestSnapshots> — capture gold snapshot         │
│    Scheduled: EconomySystemSet::ResolutionEnd                │
│      .after(advance_phase).before(rsm_input_reader)         │
│                                                              │
│  auction_tick_system  (ResMut<PlayerEconomies> direct)       │
│    reserve_gold / release_gold_reservation / spend_gold      │
│    Per ADR-013: must be atomic — no cross-frame messaging     │
│                                                              │
│  resolve_combat  (world.resource_mut::<PlayerEconomies>())   │
│    Exclusive system — sub-step 4 kills: apply_gold_award     │
│    Exclusive system — sub-step 5 objectives: apply_gold_award│
│    Exclusive system — emits ResolutionComplete after done    │
└──────────────────────────────────────────────────────────────┘

FRAME SEQUENCE — RESOLUTION ROUND N
┌──────────────────────────────────────────────────────────────┐
│  Frame N (regular Update):                                   │
│    advance_phase writes ResolutionPhaseEntered + BeginResolution│
│                                                              │
│  Between frames N and N+1 (exclusive slot):                  │
│    resolve_combat runs:                                      │
│      sub-steps 1–3: damage, status, stun                     │
│      sub-step 4: kills → api::apply_gold_award (per kill)    │
│      sub-step 5: objectives → api::apply_gold_award (per obj)│
│      sub-step 6: cleanup                                     │
│      writes MessageWriter<ResolutionComplete>                │
│                                                              │
│  Frame N+1 (regular Update, in order):                       │
│    1. EconomySystemSet::ResolutionEnd:                       │
│         on_resolution_complete reads ResolutionComplete      │
│           → discard_current_mana per player                  │
│           → snapshot gold → InterestSnapshots                │
│    2. rsm_input_reader reads ResolutionComplete              │
│         → RSM transitions RESOLUTION → DRAFT_*              │
│         → advance_phase emits DraftStarted                   │
│    3. on_draft_started reads DraftStarted                    │
│         → apply_mana_ramp per player                         │
│         → apply_income (baseline + interest from snapshot)   │
│         → writes S2CGoldUpdate (unicast) + S2CGoldBroadcast  │
└──────────────────────────────────────────────────────────────┘
```

### Key Interfaces

```rust
// server/src/core/economy/state.rs

use bevy::prelude::Resource;
use shared::session::PlayerId;
use std::collections::HashMap;

/// Per-player authoritative economy state for one game session.
/// Field mutation is forbidden outside api.rs.
#[derive(Clone, Debug)]
pub struct PlayerEconomy {
    pub gold: u32,
    pub current_mana: u32,
    pub reserve_mana: u32,
    pub mana_cap: u32,
    pub reserved_gold: u32,
}

/// Authoritative collection of all player economy states.
/// Only the five designated writers (see Decision) may hold ResMut<PlayerEconomies>.
#[derive(Resource, Default)]
pub struct PlayerEconomies(pub HashMap<PlayerId, PlayerEconomy>);

/// Gold snapshots taken at RESOLUTION end (after all kill/objective rewards).
/// Consumed at the next DRAFT start to compute interest.
#[derive(Resource, Default)]
pub struct InterestSnapshots(pub HashMap<PlayerId, u32>);
```

```rust
// server/src/core/economy/api.rs  (key function signatures — no Bevy system params)

/// Validate a mana spend before applying it. from_reserve_only: true for cards with
/// "costs from reserve" text (e.g. Garde-Temps). Returns Err if funds insufficient.
pub fn validate_spend(economy: &PlayerEconomy, cost: u32, from_reserve_only: bool)
    -> Result<(), SpendError>;

/// Apply a previously validated mana spend. Auto-split: draws current_mana first,
/// overflow to reserve_mana. Reserve-only: deducts reserve only.
pub fn apply_spend(economy: &mut PlayerEconomy, cost: u32, from_reserve_only: bool);

/// Add gold to a player's pool (kill reward, objective reward, starting gold, income).
pub fn apply_gold_award(economy: &mut PlayerEconomy, amount: u32);

/// Deduct gold for a shop purchase or auction win. Caller must validate first.
pub fn spend_gold(economy: &mut PlayerEconomy, amount: u32);

/// Apply the per-DRAFT mana ramp: current_mana = min(round, mana_cap).
pub fn apply_mana_ramp(economy: &mut PlayerEconomy, round: u32);

/// Add to reserve_mana. No cap — organic board pressure is the limiter.
pub fn add_reserve(economy: &mut PlayerEconomy, amount: u32);

/// Reserve gold for an auction bid. Fails if unreserved gold < amount.
pub fn reserve_gold(economy: &mut PlayerEconomy, amount: u32) -> Result<(), SpendError>;

/// Release an auction gold reservation (player outbid or auction cancelled).
pub fn release_gold_reservation(economy: &mut PlayerEconomy, amount: u32);

/// Zero out current_mana at RESOLUTION end.
pub fn discard_current_mana(economy: &mut PlayerEconomy);

/// Increment mana_cap by 1, capped by GameConfig.mana_cap_max (default 12).
/// Returns false if already at ceiling.
pub fn increment_mana_cap(economy: &mut PlayerEconomy, config: &GameConfig) -> bool;

/// Read-only: unreserved gold can cover an auction bid.
pub fn can_afford_bid(economy: &PlayerEconomy, amount: u32) -> bool;

/// Read-only: unreserved gold can cover a shop purchase.
pub fn can_afford_shop(economy: &PlayerEconomy, cost: u32) -> bool;

/// Read-only: current_mana + reserve_mana.
pub fn total_effective_mana(economy: &PlayerEconomy) -> u32;
```

```rust
// system.rs — corrected snapshot trigger (AFTER this ADR is adopted)

// Note: Res<PlayerEconomies> and ResMut<PlayerEconomies> CANNOT both appear in the
// same system — Bevy rejects the duplicate borrow at startup. Use ResMut only.
pub fn on_resolution_complete(
    mut resolution_complete: MessageReader<ResolutionComplete>,  // NOT ResolutionPhaseEntered
    mut interest_snapshots: ResMut<InterestSnapshots>,
    mut economies: ResMut<PlayerEconomies>,
    session: Res<SessionConfig>,
) {
    for _ in resolution_complete.read() {
        for player in session.players() {
            let Some(economy) = economies.0.get_mut(&player) else { continue };
            // Snapshot gold AFTER all kill/objective rewards have fired inside resolve_combat
            interest_snapshots.0.insert(player, economy.gold);
            // Discard current mana at RESOLUTION end
            api::discard_current_mana(economy);
        }
    }
}

// plugin.rs — corrected scheduling
// Use SystemSet reference (not function name) for resilience against RSM refactors
app.add_systems(
    Update,
    on_resolution_complete
        .in_set(EconomySystemSet::ResolutionEnd)
        .after(advance_phase)
        .before(RsmSystemSet::InputReader),  // snapshot must precede RSM DRAFT transition
);
```

```rust
// resolve_combat (exclusive system) — economy write pattern for gold awards

fn resolve_combat(world: &mut World) {
    // ... sub-steps 1–3 ...

    // Sub-step 4: deaths and kill gold
    for (killer_player, killed_count) in kills {
        let economies = world.resource_mut::<PlayerEconomies>();
        if let Some(econ) = economies.into_inner().0.get_mut(&killer_player) {
            let reward = world.resource::<GameConfig>().kill_gold_reward * killed_count;
            api::apply_gold_award(econ, reward);
        }
    }

    // Sub-step 5: objective destruction gold
    for (attacker, _) in real_objectives_destroyed {
        let economies = world.resource_mut::<PlayerEconomies>();
        if let Some(econ) = economies.into_inner().0.get_mut(&attacker) {
            let reward = world.resource::<GameConfig>().objective_gold_reward;
            api::apply_gold_award(econ, reward);
        }
    }

    // Sub-step 6: cleanup
    // ...

    // Signal completion — pattern UNVERIFIED: MessageWriter<T> is a system parameter
    // and may not be world.resource_mut()-injectable from an exclusive system.
    // Candidate resolution: set world.resource_mut::<PendingResolutionComplete>().0 = true,
    // then have a thin regular system drain it via MessageWriter<ResolutionComplete> in the same frame.
    // This must be verified against Bevy 0.18 engine-reference before implementation (see Engine Compatibility).
    world.resource_mut::<PendingResolutionComplete>().0 = true;  // placeholder pattern
}
```

### Client Projection Interface

Economy state is projected to clients via two internal server messages, written by `on_draft_started` after each DRAFT income application:

- `S2CGoldUpdate { player, gold, current_mana, reserve_mana, mana_cap }` — unicast to the owning player (private hand state)
- `S2CGoldBroadcast { player, gold }` — broadcast to all players (gold is public per GDD Rule 6)

Both are `#[derive(Message)]` server-internal messages consumed by the network dispatch story (future M2 story). They are NOT Lightyear messages — they are Bevy-internal `MessageWriter` signals between the economy system and the pending network layer.

## Alternatives Considered

### Alternative 1: ECS Components on Player Entities

- **Description**: Spawn one entity per player; attach `Gold(u32)`, `Mana(u32)`, `ReserveMana(u32)`, `ManaCap(u32)`, `ReservedGold(u32)` as separate components.
- **Pros**: Idiomatic Bevy ECS pattern; query filters can select per-player state naturally.
- **Cons**: `query.single()` returns `Result` in Bevy 0.16+, adding unwrap boilerplate at every economy call site across five different writing systems. Components can't be accessed by the exclusive `resolve_combat` system without `world.query()`, which is more verbose than `world.resource_mut()`. The multi-component approach causes archetype migration noise with each add/remove. Economy state is fundamentally per-session global state, not per-entity behavioural state — the resource pattern communicates this intent more clearly.
- **Rejection Reason**: Resources are the idiomatic Bevy pattern for singleton global state. The component approach adds query boilerplate with no benefit for data that is inherently not per-entity.

### Alternative 2: Separate Resources per Currency

- **Description**: `GoldState`, `ManaState`, `ReserveManaState`, `ManaCap`, `ReservedGoldState` as separate `Resource` types, each a `HashMap<PlayerId, u32>`.
- **Pros**: Systems that only touch one currency can express that in their `Res<T>` / `ResMut<T>` parameters, enabling finer-grained parallelism.
- **Cons**: The auto-split mana spend (current first, reserve overflow) must atomically inspect and mutate two resources in one system. Bevy can schedule two `ResMut<T>` parameters for the same system, but the separation adds boilerplate for every caller. The auction reservation invariant — `reserved_gold <= gold` — would be a cross-resource invariant with no type-system enforcement. The GDD treats the three mana pools as a unit with shared rules (auto-split, Gelure, class spells); modeling them as one struct respects that coupling.
- **Rejection Reason**: The auto-split mana spend and auction reservation invariant both require inspecting multiple fields atomically. A single struct enforces co-location and makes invariant violations detectable at a single write site.

### Alternative 3: Snapshot Inside `resolve_combat` (Economy Writes ResolutionComplete)

- **Description**: After awarding kill/objective gold, `resolve_combat` calls `economy::take_interest_snapshot(world)` directly before emitting `ResolutionComplete`. The economy's `on_resolution_complete` system is removed; mana discard also happens inside `resolve_combat`.
- **Pros**: Guarantees snapshot ordering with zero scheduling complexity. Economy and resolution are co-located in one exclusive system body.
- **Cons**: Entangles combat resolution with economy concerns — `resolve_combat` must now import economy module functions and have knowledge of snapshot semantics. ADR-017 establishes `resolve_combat` as the owner of the six combat sub-steps; economy income is a separate concern that belongs to the economy module, not the combat module. Testing economy snapshot correctness would require a full combat resolution test harness.
- **Rejection Reason**: Violates the dependency direction. Economy is the subscriber of `ResolutionComplete`; it should not be a subroutine of `resolve_combat`. The `EconomySystemSet::ResolutionEnd.before(rsm_input_reader)` scheduling constraint achieves the same ordering guarantee without coupling the modules.

## Consequences

### Positive

- `PlayerEconomy` field mutations are centralized in `api.rs` — greppable, auditable, and testable without a Bevy `App`. Every economy AC in the GDD maps to one `api.rs` function.
- The two-resource design (`PlayerEconomies` + `InterestSnapshots`) makes the interest snapshot lifecycle explicit: it is written once per round (at RESOLUTION end), consumed once (at DRAFT start), and absent otherwise. A missing snapshot at DRAFT start yields `0` interest (correct for round 1 per GDD Rule 6 step 1).
- The `EconomySystemSet::ResolutionEnd.before(rsm_input_reader)` scheduling constraint ensures the interest snapshot captures in-round kill/objective gold before the RSM transitions to DRAFT. This is the correct GDD behavior.
- Client projection messages (`S2CGoldUpdate`, `S2CGoldBroadcast`) are written at `DraftStarted` — the single moment when economy state is fully updated. The network layer has one consistent source of truth per round.
- `auction_tick_system` direct write access (per ADR-013) is formally documented here as an intentional exception to the "one primary writer" pattern. This prevents future contributors from "cleaning it up" into a cross-frame message, which would break the reservation atomicity invariant.

### Negative

- The interest snapshot is a separate resource that must be managed carefully. If `on_resolution_complete` fails to run for any reason (system bug, conditional skip), the snapshot for that round is missing and the next DRAFT uses `0` interest. The risk is mitigated by the system running unconditionally every frame in `Update`.
- The `resolve_combat` exclusive system accesses `world.resource_mut::<PlayerEconomies>()` directly, bypassing Bevy's system parameter conflict detection. A separate system that also holds `ResMut<PlayerEconomies>` running in the same exclusive slot would cause a panic. Mitigated by the exclusive system scheduling guarantee and code review.
- `S2CGoldBroadcast` (gold is public per GDD) and `S2CGoldUpdate` (per-player private currencies) are written together at `DraftStarted`. Mid-round economy changes (kill gold, objective gold during RESOLUTION) are NOT immediately projected — clients see updated gold only at the next DRAFT. This is intentional: mid-RESOLUTION gold is still resolving and the client has nothing to display during the locked RESOLUTION phase.

### Risks

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| `on_resolution_complete` is scheduled after `rsm_input_reader`, causing RSM to transition to DRAFT before snapshot is taken | Medium | HIGH — next round's interest is wrong | Explicit `.before(rsm_input_reader)` constraint in plugin registration. Integration test that verifies snapshot is populated before `on_draft_started` reads it. |
| `resolve_combat` awards gold via `world.resource_mut()` but a concurrent regular system also holds `ResMut<PlayerEconomies>` in the same Update frame | Low | PANIC at runtime | Exclusive systems run outside the regular Update schedule. Enforce via code review that no regular system holds `ResMut<PlayerEconomies>` in the same slot as the exclusive resolver. |
| Missing snapshot entry at DRAFT start (player not in `InterestSnapshots`) | Low | Zero interest for that round (no panic) | `on_draft_started` uses `.remove(&player).unwrap_or(0)` — missing snapshot silently defaults to 0. Acceptable for round 1; round N>1 missing snapshot is a silent bug. Add an assertion or warning log when snapshot is absent for round N>1. |
| `auction_tick_system` holds `ResMut<PlayerEconomies>` while `on_draft_started` also holds `ResMut<PlayerEconomies>` in the same frame | Low | Bevy scheduler conflict / panic | DRAFT_AUCTION and economy income are in the same broad DRAFT phase but economy income fires once at `DraftStarted` at DRAFT entry; auction runs every Update frame. Schedule economy income before auction tick, or use `EconomySystemSet` ordering. |

## GDD Requirements Addressed

| GDD System | Requirement | How This ADR Addresses It |
|-----------|-------------|--------------------------|
| `economy-system.md` | Rule 1 — Three independent per-player pools: `gold`, `current_mana`, `reserve_mana` + `mana_cap` | `PlayerEconomy` struct with exactly these five fields; `PlayerEconomies` resource as the singleton owner |
| `economy-system.md` | Rule 2 — `current_mana = min(round, mana_cap)` at DRAFT start | `api::apply_mana_ramp(economy, round)` called in `on_draft_started` per GDD formula F1 |
| `economy-system.md` | Rule 3 — Reserve mana persists; no cap | `reserve_mana: u32` field never zeroed by any scheduled system; `api::add_reserve` saturating-adds |
| `economy-system.md` | Rule 4 — Auto-split mana spend (current first, reserve overflow); reserve-only variant | `api::validate_spend` + `api::apply_spend` with `from_reserve_only` flag; atomic validate-then-apply pattern |
| `economy-system.md` | Rule 5 — Mana cap increase (+1 per fake objective mana reward, max 12) | `api::increment_mana_cap(economy, config)` clamps at `config.mana_cap_max`; called by Objective System |
| `economy-system.md` | Rule 6 — Interest formula: `min(floor(gold_at_RESOLUTION_end / threshold), max_bonus)` applied at DRAFT start | `InterestSnapshots` captures gold at `ResolutionComplete`; interest computed in `on_draft_started` using snapshot |
| `economy-system.md` | Rule 6 — Starting gold = 5 granted before round 1 | `initialise_player_economies` (Observer: `On<SessionReady>`) sets `gold = config.starting_gold` |
| `economy-system.md` | Rule 7 — `can_afford_bid(player, amount)`: unreserved gold check | `api::can_afford_bid` — read-only, no mutation; called by `auction_tick_system` |
| `economy-system.md` | Rule 7 — `reserve_gold` / `release_gold_reservation` atomicity | `auction_tick_system` has direct `ResMut<PlayerEconomies>` — both calls in one system body (ADR-013) |
| `economy-system.md` | EC1–EC5 (mana spending, auto-split) | Covered by `api::validate_spend` + `api::apply_spend` unit tests in `api.rs` |
| `economy-system.md` | EC6 (reserve persists across rounds) | `reserve_mana` never zeroed; only mana discard system touches `current_mana` |
| `economy-system.md` | EC7–EC8 (Gelure: transfer current → reserve, zero current) | `api::add_reserve(economy, economy.current_mana)` then `api::discard_current_mana(economy)` — two API calls, called by Class System |
| `economy-system.md` | EC9–EC10 (mana cap increment, ceiling at 12) | `api::increment_mana_cap` — no-op when already at `config.mana_cap_max` |
| `economy-system.md` | EC11 (no gold award on self-inflicted objective) | Responsibility of the caller (Objective System): passes `attacker_player != defending_player` check before calling `api::apply_gold_award` |
| `economy-system.md` | EC12 (initialization: gold=5, mana=0, reserve=0) | `initialise_player_economies` sets values from `GameConfig.starting_gold` with `current_mana = 0` and `reserve_mana = 0` |
| `economy-system.md` | EC13–EC15 (interest formula: 8g → interest=1; 10g → interest=2; 8g → 11g total) | Snapshot captured at `ResolutionComplete`; formula applied in `on_draft_started` |
| `economy-system.md` | EC16–EC17 (kill gold, objective gold awarded during RESOLUTION) | `resolve_combat` calls `api::apply_gold_award` for kills and real objective destructions |
| `economy-system.md` | EC18 (mana discard at RESOLUTION end) | `on_resolution_complete` calls `api::discard_current_mana` per player |
| `economy-system.md` | EC21–EC23 (auction bid affordability, reserved gold reduces shop budget) | `api::can_afford_bid` (reads `reserved_gold`); `api::can_afford_shop` (same pattern) |
| `round-state-machine.md` | RSM event bus subscriber: Economy reads `DraftStarted` | `on_draft_started` is a MessageReader<DraftStarted> subscriber — per ADR-010 subscriber contract |

## Performance Implications

- **CPU**: `HashMap<PlayerId, PlayerEconomy>` with 2 entries (1v1) is a constant-time lookup. All `api.rs` functions are O(1) arithmetic. Economy systems run at most a few microseconds per frame; total economy budget < 0.1 ms.
- **Memory**: `PlayerEconomies` holds 2 `PlayerEconomy` structs (~40 bytes each). `InterestSnapshots` holds 2 `u32` values per round. Total: < 256 bytes for both resources.
- **Network**: `S2CGoldUpdate` and `S2CGoldBroadcast` are written once per DRAFT start (not per-frame). Bandwidth impact is negligible — two small messages per round transition.
- **Load Time**: Both resources are `Default`-initialized at plugin build time; no asset loading required.

## Migration Plan

The current implementation has one correction required (snapshot timing), plus a test gap to resolve:

1. **Rename + rework snapshot system** (`system.rs`): Replace `on_resolution_phase_entered` and `discard_current_mana_at_resolution_end` with a single `on_resolution_complete` system that reads `MessageReader<ResolutionComplete>` instead of `MessageReader<ResolutionPhaseEntered>`. This is the fix for the timing bug.

2. **Update plugin registration** (`plugin.rs`): Replace the current system registration:
   ```rust
   // BEFORE (reads ResolutionPhaseEntered — wrong timing)
   .add_systems(Update, (on_resolution_phase_entered, discard_current_mana_at_resolution_end)
       .in_set(EconomySystemSet::ResolutionEnd).after(advance_phase))
   
   // AFTER (reads ResolutionComplete — after all gold awards)
   .add_systems(Update, on_resolution_complete
       .in_set(EconomySystemSet::ResolutionEnd)
       .after(advance_phase)
       .before(rsm_input_reader))
   ```
   Remove the TODO comment — this ADR replaces it.

3. **Remove the starting gold grant from `on_draft_started`**: The `DraftPhase::Initial` branch that grants `starting_gold` is replaced by `initialise_player_economies` (which fires on `SessionReady` and sets `gold = config.starting_gold`). Verify that `on_draft_started` for `DraftPhase::Initial` only applies `apply_mana_ramp` and skips the interest path (round 1 has no snapshot).

4. **Write M2 integration test**: Once `resolve_combat` is implemented, add an integration test that: (a) awards kill gold inside resolve_combat, (b) verifies `InterestSnapshots` captures the post-kill gold value, (c) verifies next DRAFT interest reflects the kill gold.

## Validation Criteria

- [ ] `PlayerEconomies` and `InterestSnapshots` compile as `#[derive(Resource, Default)]` against Bevy 0.18 without deprecation warnings.
- [ ] `on_resolution_complete` reads `MessageReader<ResolutionComplete>` and is scheduled `.before(rsm_input_reader)`. Verified by Bevy schedule graph dump.
- [ ] **EC13**: After `apply_gold_award(economy, 8)` and snapshot at 8g, next `on_draft_started` (round N>1) produces `gold = 11` (8 + 1 interest + 2 baseline). Tested in `tests/unit/economy/`.
- [ ] **EC14**: Snapshot at 10g → interest = 2 (maximum). Unit test in `tests/unit/economy/`.
- [ ] **EC18**: After `on_resolution_complete` runs, all players have `current_mana = 0`. Unit test using `World::new()` + `MessageWriter<ResolutionComplete>`.
- [ ] **EC12**: After `initialise_player_economies` fires via `On<SessionReady>`, all session players have `gold = config.starting_gold`, `current_mana = 0`, `reserve_mana = 0`. World-based test.
- [ ] `ResMut<PlayerEconomies>` appears only in: `initialise_player_economies`, `on_draft_started`, `on_resolution_complete`, `auction_tick_system`, `resolve_combat`. Code review gate on every PR that adds a new writer.
- [ ] No code outside `server/src/core/economy/api.rs` assigns directly to `PlayerEconomy` fields. Verified by CI grep for `.gold =`, `.current_mana =`, `.reserve_mana =`, `.mana_cap =`, `.reserved_gold =` outside the api module.
- [ ] `S2CGoldUpdate` and `S2CGoldBroadcast` are written in `on_draft_started` after all income is applied. Order: mana ramp first, then income, then write both messages.

## Related Decisions

- [ADR-002 — Client-Server Authority Model](./adr-002-client-server-authority.md) — `PlayerEconomies` and `InterestSnapshots` are server-only Resources; they must never enter `protocol/` or `client/` crates.
- [ADR-009 — RSM Phase State](./adr-009-rsm-phase-state.md) — `DraftStarted` and `ResolutionPhaseEntered` are emitted by `advance_phase`; `ResolutionComplete` is emitted by `resolve_combat` and read by `rsm_input_reader`.
- [ADR-010 — RSM Event Bus](./adr-010-rsm-event-bus.md) — Economy System subscribes to `DraftStarted` per the subscriber contracts table. The `ResolutionComplete` → economy snapshot path is the M2 update to that table.
- [ADR-013 — Auction System State](./adr-013-auction-system-state.md) — Establishes that `auction_tick_system` holds `ResMut<PlayerEconomies>` directly. ADR-019 formalizes this as a documented exception in the write access rules.
- [ADR-017 — Combat Resolution Architecture](./adr-017-combat-resolution-execution-architecture.md) — Establishes `resolve_combat` as the exclusive system that awards kill/objective gold and emits `ResolutionComplete`. ADR-019 defines the economy-side contract for receiving that signal.
- `design/gdd/economy-system.md` — Primary GDD source for all currency rules, formulas, and acceptance criteria.
- `server/src/core/economy/` — Live implementation; `api.rs`, `state.rs`, `system.rs`, `plugin.rs`.
