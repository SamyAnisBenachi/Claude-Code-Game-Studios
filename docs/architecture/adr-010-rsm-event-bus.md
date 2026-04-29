# ADR-010: RSM Phase Event Bus — Phase Message Catalog and Subscriber Contracts

## Status

Accepted

## Date

2026-04-29

## Last Verified

2026-04-29

## Decision Makers

User + lead-programmer + gameplay-programmer + network-programmer

## Summary

The Round State Machine communicates all phase transitions exclusively via Bevy
buffered Messages (`MessageWriter`/`MessageReader`). The RSM's `advance_phase` function
is the sole emitter of phase boundary messages. All Core and Feature systems subscribe
to these messages to react to phase changes. The RSM has zero direct imports from
`feature/` modules — it never calls into them. Emission ordering within a single
phase transition is enforced by linear code order in `advance_phase` match arms,
not by Bevy system scheduling constraints.

> **M2 Update Required**: This ADR documents the M1 event catalog. When Auction
> System and Combat Resolution are implemented in M2, their subscriber contracts
> must be added to this ADR and verified against the inbound event shapes defined
> here. See the Subscriber Contracts table for the two M2 placeholder entries.

---

## Engine Compatibility

| Field | Value |
|-------|-------|
| **Engine** | Bevy 0.18 |
| **Domain** | Core / ECS Events |
| **Knowledge Risk** | HIGH — Bevy 0.15–0.18 all post-cutoff; Event API changed significantly across these versions |
| **References Consulted** | `docs/engine-reference/bevy/VERSION.md`, ADR-009 (RSM state structure) |
| **Post-Cutoff APIs Used** | `MessageWriter::write()` / `MessageReader::read()` (Bevy 0.17+ Message/Event split — `EventWriter`/`EventReader` no longer exist); `#[derive(Message)]` for buffered pull-based messages; `#[derive(Event)]` + `Observer` for push-based triggers only |
| **Verification Required** | Confirm `MessageWriter<T>` / `MessageReader<T>` system param names in Bevy 0.18; confirm `app.add_message::<T>()` registration API; confirm that RSM phase messages correctly use `#[derive(Message)]` not `#[derive(Event)]` |

> **Critical API note — Bevy 0.18 Message/Event split**: `EventWriter`/`EventReader`/`Events<T>` **no longer exist** in Bevy 0.17+. They were replaced by two distinct mechanisms:
> - **Buffered Messages** (pull-based, polled each frame): `#[derive(Message)]` + `MessageWriter<T>` + `MessageReader<T>` + `app.add_message::<T>()`. This is the correct pattern for RSM phase transitions.
> - **Observer Events** (push-based, immediate same-frame trigger): `#[derive(Event)]` + `commands.trigger()` + `Observer`. Reserved for one-shot lifecycle events (`SessionReady` per ADR-012) and reactive keyword triggers in the Feature layer.
>
> Any code using `EventWriter<T>`, `EventReader<T>`, or `app.add_event::<T>()` will fail to compile on Bevy 0.18.
>
> **`liv-bevy-018` mandatory** on all `.rs` files that define, emit, or read these
> message types. The skill enforces correct 0.18 API patterns and prevents pre-0.17
> regressions.

---

## ADR Dependencies

| Field | Value |
|-------|-------|
| **Depends On** | ADR-009 (RSM states and `advance_phase` function — that ADR defines the state machine; this ADR defines what it emits); ADR-003 (workspace layout — events defined in `server/core/rsm/events.rs`); ADR-008 (channel config — `BroadcastPhaseChanged` triggers the network dispatch that uses `ReliableChannel`) |
| **Enables** | Economy System implementation (reads `DraftStarted`); Card Pool implementation (reads `ShopRefreshNeeded`); Board/Lane System phase gating (reads `PlacementPhaseEntered`, `ResolutionPhaseEntered`); Game Session System teardown (reads `GameOverEmitted`); Network dispatch system (reads `BroadcastPhaseChanged`) |
| **Blocks** | Any story that implements a phase-reactive server system. No system may write a phase-reactive subscriber until this ADR is Accepted and the event types in `server/core/rsm/events.rs` are defined. |
| **Ordering Note** | ADR-009 must be Accepted first — `advance_phase` is where all `EventWriter::write()` calls live. This ADR is a companion to ADR-009, not a replacement. |

---

## Context

### Problem Statement

The Round State Machine is the server-side phase orchestrator for Lanes and Lies. Every
system that reacts to a phase change — Economy, Card Pool, Auction, Combat Resolution,
Board/Lane, Game Session teardown, and the network broadcast layer — needs to know when
a phase transition occurs and what the new phase is.

Without a documented event catalog and subscriber contract, individual programmers will
implement phase reactions inconsistently: some will poll the current phase every frame
(O(n) per system per frame, fragile to ordering), some will add direct calls from
`advance_phase` into feature modules (coupling the RSM to every downstream system),
and some will define their own ad-hoc event types with no cross-system coordination.

The core constraint is that the RSM must be decoupled from feature systems. The RSM
lives in `server/core/rsm/`. Feature systems live in `server/feature/`. The RSM may
not import from `server/feature/` — doing so would create a dependency inversion that
makes the feature layer part of the core loop, preventing independent feature
development and testing.

The secondary constraint is emission ordering. Several systems have ordering dependencies
within a single DRAFT entry transition: mana must be set before gold income is applied,
gold must be available before the shop is drawn, the shop must be drawn before the
Auction System initialises, and clients must be notified only after all server state
is ready. The RSM's GDD (Formula F2) specifies this order explicitly.

### Constraints

- The RSM (`server/core/rsm/`) must have zero direct imports from `server/feature/`.
- All phase-reactive logic must be triggered by an event, not by direct function call.
- The F2 emission order (GDD round-state-machine.md) is contractual and must be enforced.
- Events are Bevy buffered events — they are processed within the same `Update` set,
  by systems scheduled after `advance_phase`.
- Observers are reserved for the Feature layer's keyword trigger system. The RSM
  event bus uses `EventWriter`/`EventReader` only.
- Configuration values (timer durations, round numbers) must be passed as event fields
  — subscribers must not re-query `GameConfig` to determine context they could receive
  from the event directly.

### Requirements

- A complete, stable event type catalog defined in `server/core/rsm/events.rs`.
- Every outbound event emitted by the RSM documented with: fields, emitting context,
  and which subscriber system reads it and what action it takes.
- Every inbound event read by the RSM documented with: which system emits it and
  what RSM state transition it triggers.
- Emission ordering for DRAFT entry transitions documented and enforced by code order
  in `advance_phase`, not by system ordering constraints.
- The catalog remains the single source of truth. Any new phase-reactive system must
  add its subscriber contract to this ADR before implementation.

---

## Decision

The RSM communicates all phase transitions via Bevy buffered Messages. The RSM's
`advance_phase` function (defined in ADR-009) is the sole emitter of outbound
phase messages. Feature and Core systems subscribe via `MessageReader`. The RSM reads
a small set of inbound messages from other systems to detect completion signals
(auction settled, resolution complete). `SessionReady` is delivered via Bevy Observer
(same-frame trigger) per ADR-012 — it is NOT a buffered Message.

All message types are defined in `server/core/rsm/events.rs` and re-exported through
the `server/core` module boundary. Feature systems import message types from `core`,
not from `rsm` directly.

### Architecture Diagram

```
server/core/rsm/advance_phase
          │
          │  emits (in strict order per F2)
          ▼
┌─────────────────────────────────────────────────────────────┐
│                  RSM Message Bus                            │
│  (Bevy buffered MessageWriter — same Update set)            │
│                                                             │
│  DraftStarted ──────────────────────► Economy System        │
│    { round, phase: DraftPhase }           apply_mana_ramp   │
│                                           apply_gold_income  │
│                                                             │
│  ShopRefreshNeeded ─────────────────► Card Pool             │
│    { player: PlayerId }                   refresh_shop       │
│                                           (per player)      │
│                                                             │
│  AuctionPhaseEntered ───────────────► Auction System [M2]   │
│    { round: u32 }                         start_auction      │
│                                                             │
│  PlacementPhaseEntered ─────────────► Board/Lane System     │
│    { round: u32 }                         open_buffer        │
│                                                             │
│  ResolutionPhaseEntered ────────────► Combat Resolution [M2]│
│    { round: u32 }                         execute_sub_steps  │
│                                 ├──► Objective System       │
│                                 │       eval_destruction     │
│                                 └──► Board/Lane System       │
│                                          prepare_cleanup     │
│                                                             │
│  GameOverEmitted ───────────────────► Game Session System   │
│    { reason, loser }                      session_teardown   │
│                                                             │
│  BroadcastPhaseChanged ─────────────► Network Dispatch      │
│    { phase, round, timer_ms }             S2CPhaseChanged    │
│                                           (always LAST)     │
└─────────────────────────────────────────────────────────────┘
          ▲
          │  reads (inbound — emitted by other systems)
          │
┌─────────────────────────────────────────────────────────────┐
│  SessionReady          ◄── Game Session System              │
│  AuctionSettled [M2]   ◄── Auction System                   │
│    { winner, price, card_id }                               │
│  ResolutionComplete [M2] ◄── Combat Resolution              │
└─────────────────────────────────────────────────────────────┘
```

### Complete Event Catalog

All outbound message types are defined in `server/core/rsm/events.rs`. All derive
`Message`, `Clone`, and `Debug`. Types imported from other modules (`PlayerId`,
`CardId`, `RoundPhase`, `GameOverReason`, `DraftPhase`) are defined in
`shared/src/protocol.rs`.

`SessionReady` is an Observer Event (`#[derive(Event)]`) not a buffered Message —
it uses `commands.trigger()` per ADR-012 and must NOT be registered via
`app.add_message::<SessionReady>()`.

#### Outbound Messages (RSM writes, systems read)

```rust
// server/core/rsm/events.rs

use bevy::prelude::*;
use shared::protocol::{PlayerId, CardId, RoundPhase, GameOverReason, DraftPhase};

/// Written on entry into DRAFT_INITIAL, DRAFT_AUCTION, or DRAFT_SHOP.
/// The Economy System reads this to apply mana ramp and gold income.
/// The `phase` field tells the Economy System which income formula applies
/// (round 1 starting gold vs. baseline + interest).
#[derive(Message, Clone, Debug)]
pub struct DraftStarted {
    /// The round number as of this DRAFT entry (already incremented per Rule 2).
    pub round: u32,
    /// Which DRAFT sub-phase was entered.
    pub phase: DraftPhase,  // DraftPhase::Initial | Auction | Shop
}

/// Written once per player on entry into any DRAFT phase, after DraftStarted.
/// The Card Pool reads this to draw 3 weighted cards for that player's shop.
/// For DRAFT_INITIAL, this populates the initial 9-card selection.
/// Written separately per player so the Card Pool can draw independently per player.
#[derive(Message, Clone, Debug)]
pub struct ShopRefreshNeeded {
    /// The player whose shop should be refreshed.
    pub player: PlayerId,
}

/// Written on entry into DRAFT_AUCTION, after ShopRefreshNeeded for all players.
/// The Auction System [M2] reads this to initialise auction state and start its
/// 20-second timer. Clients are NOT notified of DRAFT_AUCTION until after this
/// message is processed (see F2 step 4 — BroadcastPhaseChanged comes last).
#[derive(Message, Clone, Debug)]
pub struct AuctionPhaseEntered {
    pub round: u32,
}

/// Written on entry into PLACEMENT.
/// The Board/Lane System reads this to open the placement submission buffer
/// and begin tracking per-player submission status.
#[derive(Message, Clone, Debug)]
pub struct PlacementPhaseEntered {
    pub round: u32,
}

/// Written on entry into RESOLUTION.
/// Multiple systems subscribe:
///   - Combat Resolution [M2]: execute all six global sub-steps
///   - Objective System: evaluate destruction damage (reads real_objectives_destroyed)
///   - Board/Lane System: prepare for board cleanup after resolution completes
#[derive(Message, Clone, Debug)]
pub struct ResolutionPhaseEntered {
    pub round: u32,
}

/// Written when the RSM transitions to GAME_OVER for any reason.
/// The Game Session System reads this to execute session teardown:
/// destroy SessionConfig, release ServerRng, and clean up Lightyear session state.
/// `loser: None` signals a Draw (mutual destruction, mutual disconnection, or
/// resolution safety timeout). This field mirrors the `S2CGameOver` payload shape.
#[derive(Message, Clone, Debug)]
pub struct GameOverEmitted {
    pub reason: GameOverReason,
    /// None = Draw; Some(id) = the player who lost.
    pub loser: Option<PlayerId>,
}

/// Always the LAST message written in any phase transition.
/// The network dispatch system reads this to send `S2CPhaseChanged` via
/// `ReliableChannel` to all connected clients (see ADR-008).
/// `timer_ms = 0` for GAME_OVER and LOBBY phases (no client-side timer).
/// `timer_ms = 0` for DRAFT_AUCTION (Auction System drives its own countdown;
/// clients must not render an RSM-owned timer for DRAFT_AUCTION).
#[derive(Message, Clone, Debug)]
pub struct BroadcastPhaseChanged {
    /// The RoundPhase that was just entered.
    pub phase: RoundPhase,
    pub round: u32,
    /// Duration of the new phase's timer in milliseconds, for client countdown display.
    /// 0 = no RSM-owned timer for this phase.
    pub timer_ms: u32,
}
```

#### Inbound Messages (Other systems write, RSM reads)

```rust
/// DELIVERY: Observer trigger per ADR-012. NOT a buffered Message.
/// Triggered by the Game Session System via `commands.trigger(SessionReady)` when
/// all LOBBY conditions are satisfied. The RSM observes this via `app.observe(on_session_ready)`.
/// Do NOT register via `app.add_message::<SessionReady>()`.
/// Do NOT read via `MessageReader<SessionReady>` — it will never fire.
#[derive(Event, Clone, Debug)]
pub struct SessionReady;

/// Written by the Auction System [M2] when the auction concludes
/// (winner found or 20-second timer expired with no bids).
/// The RSM reads this in DRAFT_AUCTION to transition to DRAFT_SHOP.
/// The RSM validates `phase == DRAFT_AUCTION` before acting — a stale
/// AuctionSettled message (e.g., written after GAME_OVER) is silently discarded.
#[derive(Message, Clone, Debug)]
pub struct AuctionSettled {
    /// None if the auction timed out with no bids.
    pub winner: Option<PlayerId>,
    /// The price the winner paid. 0 if no winner.
    pub final_price: u32,
    pub card_id: CardId,
}

/// Written by Combat Resolution [M2] when all six global sub-steps complete.
/// The RSM reads this in RESOLUTION to evaluate the GAME_OVER condition
/// (real_objectives_destroyed) and then transition to the next DRAFT or GAME_OVER.
/// The RSM validates `phase == RESOLUTION` before acting.
#[derive(Message, Clone, Debug)]
pub struct ResolutionComplete;
```

### Key Interfaces

#### `DraftPhase` Enum

`DraftPhase` is defined in `shared/src/protocol.rs` alongside `RoundPhase`:

```rust
// shared/src/protocol.rs

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum DraftPhase {
    /// Round 1 only — DRAFT_INITIAL. Starting gold = 5; no prior interest.
    Initial,
    /// Rounds where is_auction_round(R) = true. Economy applies baseline + interest.
    Auction,
    /// Standard shop rounds. Economy applies baseline + interest.
    Shop,
}
```

The Economy System uses `DraftPhase` to select the correct income formula without
needing to read `round_number` directly from RSM state.

#### Subscriber Contracts Table

This table is the canonical subscriber contract. It is BLOCKING for story authoring:
any story implementing a phase-reactive system MUST be traceable to a row in this table.
M2 rows are placeholders — they must be filled in when M2 systems are implemented.
All subscribers read via `MessageReader<T>`, not `EventReader<T>`.

| Event emitted by RSM | Subscriber System | Action | Story milestone |
|---|---|---|---|
| `DraftStarted { phase: Initial }` | Economy System | `apply_mana_ramp(all_players)` — sets `current_mana = min(1, mana_cap)`; then `apply_gold_income(all_players)` — grants starting_gold = 5 (not baseline + interest) | M1 |
| `DraftStarted { phase: Auction \| Shop }` | Economy System | `apply_mana_ramp(all_players)` — sets `current_mana = min(round, mana_cap)`; then `apply_gold_income(all_players)` — grants `baseline + interest` (interest from prior RESOLUTION snapshot) | M1 |
| `ShopRefreshNeeded { player }` | Card Pool | `refresh_shop(player)` — draws 3 weighted cards for that player's personal shop; for DRAFT_INITIAL, populates the initial 9-card offering | M1 |
| `AuctionPhaseEntered { round }` | Auction System | Start the 20-second auction timer; initialise auction bid state; prepare the `S2CAuctionCard` broadcast. **[M2 — not yet implemented]** | M2 |
| `PlacementPhaseEntered { round }` | Board/Lane System | Open the placement submission buffer (`PendingPlacements` resource per ADR-007); start tracking `submissions_received: Set<PlayerId>`; begin accepting `C2SSubmitPlacement` messages | M1 |
| `ResolutionPhaseEntered { round }` | Combat Resolution | Execute all six global combat sub-steps; emit `ResolutionComplete` when done. **[M2 — not yet implemented]** | M2 |
| `ResolutionPhaseEntered { round }` | Objective System | Evaluate `real_objectives_destroyed(player)` across all lanes after combat sub-steps complete; make results available for RSM's GAME_OVER check | M1 |
| `ResolutionPhaseEntered { round }` | Board/Lane System | Prepare for post-resolution board cleanup: mark board as locked for new placements; listen for `ResolutionComplete` to execute unit cleanup and carry-over logic | M1 |
| `GameOverEmitted { reason, loser }` | Game Session System | Session teardown: destroy `SessionConfig` resource; destroy `ServerRng` resource (per ADR-005); broadcast `S2CGameOver` on `ReliableChannel`; signal Lightyear to close session; log session outcome | M1 |
| `BroadcastPhaseChanged { phase, round, timer_ms }` | Network Dispatch System | Send `S2CPhaseChanged { phase, round_number: round, timer_duration_ms: timer_ms }` via `ReliableChannel` to `NetworkTarget::All` (per ADR-008); must be the final system to execute in the phase-transition `Update` run | M1 |

#### Inbound Event Subscriber Contracts (RSM reads these)

| Event read by RSM | Emitting System | RSM Action | Guard |
|---|---|---|---|
| `SessionReady` | Game Session System | Transition LOBBY → DRAFT_INITIAL; begin F2 emission sequence for round 1 | Phase must be LOBBY; discarded otherwise |
| `AuctionSettled { .. }` | Auction System [M2] | Transition DRAFT_AUCTION → DRAFT_SHOP; emit F2 sequence for DRAFT_SHOP | Phase must be DRAFT_AUCTION; discarded otherwise |
| `ResolutionComplete` | Combat Resolution [M2] | Evaluate GAME_OVER condition; transition to GAME_OVER or next DRAFT | Phase must be RESOLUTION; discarded otherwise |

### Emission Ordering Within a Phase Transition

The emission order within `advance_phase` is STRICT and derived from GDD Formula F2.
It is enforced by the linear code order within each match arm — NOT by Bevy system
ordering constraints. All events are emitted in a single `advance_phase` call before
any subscriber system runs.

#### DRAFT Entry (DRAFT_INITIAL, DRAFT_AUCTION, DRAFT_SHOP)

```
In advance_phase match arm for DRAFT_* entry:

1. MessageWriter<DraftStarted>.write(...)         // Economy: mana + income
2. MessageWriter<ShopRefreshNeeded>.write(...)     // Card Pool: per player, in order
   MessageWriter<ShopRefreshNeeded>.write(...)     // (one write per player)
3. MessageWriter<AuctionPhaseEntered>.write(...)   // DRAFT_AUCTION only
4. MessageWriter<BroadcastPhaseChanged>.write(...) // ALWAYS LAST — clients notified
                                                    // after all server state is ready
```

Rationale for ordering:
- Step 1 before step 2: players need mana and gold set before the shop draw; the
  Economy System's `refresh_shop` call within Card Pool may inspect gold for
  weighting hints (if applicable).
- Step 2 before step 3: the Auction System must not start before shops are populated;
  players see their shop during the auction (a deliberate UX: players can make informed
  bid decisions knowing their upcoming shop).
- Step 3 before step 4: the Auction System must have initialised before clients are
  told DRAFT_AUCTION is live — a `C2SAuctionBid` arriving before the Auction System is
  ready would reach an uninitialised state (see RSM GDD F2 step 4 rationale).
- Step 4 is always last: the F2 contract guarantees clients are notified only after
  all server state is correct.

#### PLACEMENT Entry

```
1. MessageWriter<PlacementPhaseEntered>.write(...)
2. MessageWriter<BroadcastPhaseChanged>.write(...) // ALWAYS LAST
```

#### RESOLUTION Entry

```
1. MessageWriter<ResolutionPhaseEntered>.write(...)
2. MessageWriter<BroadcastPhaseChanged>.write(...) // ALWAYS LAST
```

#### GAME_OVER Entry

```
1. MessageWriter<GameOverEmitted>.write(...)
2. MessageWriter<BroadcastPhaseChanged>.write(...) // ALWAYS LAST
```

Note: `S2CGameOver` is a separate network message from `S2CPhaseChanged(GAME_OVER)`.
Per RSM GDD Rule 14, both are broadcast. The Game Session System's `GameOverEmitted`
subscriber sends `S2CGameOver`; the network dispatch system's `BroadcastPhaseChanged`
subscriber sends `S2CPhaseChanged`. Both messages arrive on the `ReliableChannel`
broadcast; delivery order is enqueue order (per ADR-008).

#### What the RSM Does NOT Signal

The RSM emits phase entry signals only. It does not emit events for:

- Economy formula details (Economy System owns those formulas; it reads `DraftStarted`
  and applies them internally).
- Combat sub-steps (Combat Resolution owns the six sub-steps; it reads
  `ResolutionPhaseEntered` and drives its own sub-step sequence internally).
- Auction bid validation (Auction System owns that; it reads `AuctionPhaseEntered`
  and manages its own bid acceptance loop).
- Interest snapshot timing (the Economy System emits an internal snapshot event at
  RESOLUTION end; the RSM does not need to signal this separately — the Economy System
  observes `ResolutionPhaseEntered` and takes its snapshot after the resolution
  sub-steps complete, before emitting `ResolutionComplete`).

The RSM only signals phase entry and exit. Systems own their responses.

### Implementation Guidelines

1. **File location**: All event types in `server/core/rsm/events.rs`. Re-export
   through `server/core/rsm/mod.rs` so feature systems import from
   `use server::core::rsm::events::*` — not from internal module paths.

2. **`advance_phase` emitter discipline**: The `advance_phase` system takes
   `MessageWriter<T>` parameters for all outbound messages it may emit. It does NOT
   take any `MessageReader<T>` for inbound signals — those are read by a separate
   `rsm_input_reader` system that updates RSM state before `advance_phase` runs.

3. **System scheduling in `Update`**: In the RSM plugin's `Update` set:
   ```
   rsm_input_reader  →  advance_phase  →  [all subscriber systems]
   ```
   All subscriber systems must be scheduled `.after(advance_phase)` so they read
   messages written in the current frame. Subscribers scheduled before `advance_phase`
   will not see the current frame's messages.

4. **Bevy message lifetime**: By default, Bevy clears messages two frames after they
   are written. Subscriber systems that are not guaranteed to run every frame must
   handle the case where a message was missed. For RSM messages, all subscribers run
   every frame in `Update` — missed messages are a bug, not an expected code path.

5. **Guard pattern for inbound messages**: The RSM `rsm_input_reader` system must
   validate `phase == expected_phase` before acting on inbound messages:
   ```rust
   for _ in auction_settled_messages.read() {
       if rsm_state.phase != RoundPhase::DraftAuction {
           continue; // Stale message — discard silently
       }
       rsm_state.phase = RoundPhase::DraftShop;
       // ... queue advance_phase
   }
   ```

6. **No Observers for RSM messages**: Using `Observer::new(callback)` to watch RSM
   message types would bypass the scheduling guarantees described above. RSM messages
   are polled via `MessageReader` only. `SessionReady` is the sole exception — it uses
   Observer per ADR-012. The `liv-bevy-018` skill enforces this distinction.

---

## Alternatives Considered

### Alternative 1: Direct Function Calls from `advance_phase` into Feature Systems

- **Description**: `advance_phase` calls `economy_system::apply_draft_entry()`,
  `card_pool::refresh_shop()`, etc. directly via mutable references.
- **Pros**: Simple call stack; debuggable in a single frame trace; no event buffering overhead.
- **Cons**: Couples `server/core/rsm/` to every feature module. Adding a new
  phase-reactive system requires modifying `advance_phase`. Feature systems cannot
  be tested in isolation from the RSM. Violates the dependency direction rule:
  core must not import feature.
- **Rejection Reason**: Direct coupling from Core to Feature inverts the dependency
  direction that the workspace layout (ADR-003) establishes. Rejected.

### Alternative 2: Bevy Observers (Reactive Triggers) for Phase Events

- **Description**: Use Bevy 0.17+ Observer pattern — `app.observe(on_draft_started)`
  — rather than `MessageWriter`/`MessageReader`. Systems register callbacks that fire
  immediately when the event is triggered.
- **Pros**: No per-frame polling overhead; observer callbacks fire immediately on
  trigger, not on the next `MessageReader` poll; no two-frame message lifetime concern.
- **Cons**: Observers in Bevy 0.17/0.18 are designed for component mutation reactions
  (ADDED, REMOVED, CHANGED on components) and one-shot lifecycle triggers. They are
  NOT suited for explicit phase transitions where multiple subscribers must fire in a
  defined order. Observer callbacks run in an implicit order that is harder to reason
  about than explicit system scheduling. The F2 emission ordering guarantee (steps 1–4)
  cannot be enforced across Observer callbacks without additional coordination.
  The Bevy 0.17 Message/Observer split explicitly designates buffered `Message` types
  for "system A writes, systems B/C/D read next frame" patterns, and Observers for
  one-shot reactive triggers (`SessionReady` per ADR-012). Using Observers for
  recurring RSM phase messages would violate this convention.
- **Rejection Reason**: Observer semantics are wrong for RSM phase messages. Buffered
  Messages with explicit system ordering are the correct Bevy 0.17/0.18 pattern for
  recurring, polled signals. Rejected.

### Alternative 3: Single `PhaseChanged { old_phase, new_phase }` Omnibus Event

- **Description**: Replace the entire catalog with one `PhaseChanged` event that
  carries the old and new phase. All subscribers pattern-match on the new phase and
  act accordingly.
- **Pros**: Simpler event catalog (one type); any system can subscribe without a
  separate contract per phase.
- **Cons**: Subscribers must pattern-match on `new_phase` — a subscriber that cares
  only about DRAFT entry must filter for three variants (`DraftInitial`, `DraftAuction`,
  `DraftShop`). The `DraftPhase` sub-type (which selects the Economy income formula)
  cannot be cleanly encoded in a single `PhaseChanged` event without a discriminated
  union of the full `RoundPhase` enum. The `ShopRefreshNeeded { player }` event
  fundamentally cannot be collapsed — it is a per-player signal, not a phase signal.
  An omnibus event also couples all subscribers to the full `RoundPhase` enum, forcing
  every new subscriber to understand all phases.
- **Rejection Reason**: The per-player `ShopRefreshNeeded` pattern cannot be collapsed
  into a single phase event without duplicating the per-player fan-out inside every
  subscriber. The named event catalog makes each subscriber's contract explicit and
  testable. Rejected.

### Alternative 4: Shared `RsmState` Resource Polling (No Events)

- **Description**: The RSM writes to a `RsmState { phase, round, prev_phase }` resource.
  Subscribers poll this resource each frame and compare `phase != prev_phase` to detect
  transitions.
- **Pros**: No event infrastructure; subscribers always have the current state available.
- **Cons**: Every subscriber runs every frame even when no transition has occurred —
  O(n systems × frames) polling overhead. The "first frame of new phase" detection
  (`phase != prev_phase`) is fragile: if a subscriber is scheduled before `prev_phase`
  is updated, it misses the transition. The F2 ordering guarantee is impossible to
  enforce — all subscribers see the new state simultaneously, with no ordering between
  them. Per-player signals (`ShopRefreshNeeded`) require additional per-player polling
  state in each subscriber.
- **Rejection Reason**: Polling violates the event-driven architecture established by
  the RSM GDD. The ordering guarantee and per-player fan-out pattern are both
  inexpressible with simple resource polling. Rejected.

---

## Consequences

### Positive

- The RSM has zero imports from `server/feature/`. Core and Feature are cleanly
  separated. Feature systems can be added, removed, or replaced without touching
  `advance_phase`.
- The F2 emission ordering guarantee is enforced by linear code order in a single
  function — not by Bevy system scheduling constraints that can silently break when
  systems are moved between plugins.
- Each event type documents its subscriber contract explicitly. A programmer implementing
  a new phase-reactive system has a contract to implement against, not an implicit
  convention to discover.
- Per-player fan-out (`ShopRefreshNeeded`) is first-class: the RSM writes one event
  per player, the Card Pool reads N events and draws N independent shops. No subscriber
  needs to know the player count.
- Events are testable in isolation: a unit test can write `DraftStarted` to a
  Bevy `World` and assert that the Economy System processes it correctly, without
  running the full RSM state machine.
- The `BroadcastPhaseChanged` event as the last emission is a compile-time-enforceable
  contract: client notification happens after all server state mutations in `advance_phase`
  are complete, preventing the class of bug where a client receives `S2CPhaseChanged`
  before server state is ready to accept its responses.

### Negative

- The event catalog is a shared contract: adding a new phase-reactive system requires
  updating `events.rs` and this ADR, not just adding a new subscriber. This is overhead,
  but it is intentional — undocumented subscribers are a maintenance hazard.
- Bevy's two-frame event lifetime means a subscriber that skips a frame (due to system
  ordering or conditional execution) may miss an event. All RSM event subscribers must
  run every frame in `Update`. This constraint must be enforced by code review, not
  by the type system.
- The `AuctionSettled` and `ResolutionComplete` inbound events introduce a
  read-then-advance pattern in the RSM: `rsm_input_reader` reads inbound events and
  updates RSM state; `advance_phase` emits outbound events based on the updated state.
  This two-system pattern adds scheduling complexity that direct return values would
  not have. The coupling benefit (RSM does not call into Auction or Combat directly)
  justifies the cost.

### Risks

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| Bevy 0.18 `EventWriter::write()` API differs from expected | MEDIUM | Compilation failure | Verify against Bevy 0.18 migration guide before writing any emitter code. `liv-bevy-018` skill enforces correct method name. |
| Subscriber scheduled before `advance_phase` misses events | MEDIUM | Silent logic failure — phase entry action never executes | Enforce `.after(advance_phase)` scheduling in each subscriber's plugin registration. Add an integration test that verifies each subscriber fires on the expected phase. |
| M2 subscriber contracts added without updating this ADR | MEDIUM | Undocumented coupling between RSM events and M2 systems | Gate M2 stories on updating this ADR's Subscriber Contracts table before implementation begins. |
| Stale `AuctionSettled` or `ResolutionComplete` event processed out-of-phase | LOW | RSM double-transitions or transitions from wrong phase | Guard pattern in `rsm_input_reader` (see Implementation Guidelines item 5) must be implemented and tested. RSM-31 and RSM-35 acceptance criteria cover the double-transition case. |
| `BroadcastPhaseChanged` accidentally emitted before server state is ready | LOW | Client receives phase change before server accepts responses | Linear code order enforcement in `advance_phase` — `BroadcastPhaseChanged` is always the last `write()` call. Code review gate. |

---

## Performance Implications

- **CPU**: Bevy event handling is O(n events × n subscribers). Per round, the RSM
  emits at most ~6 events (DraftStarted, 2× ShopRefreshNeeded in 1v1,
  AuctionPhaseEntered if applicable, BroadcastPhaseChanged, and any resolution events).
  Event processing is negligible at this volume.
- **Memory**: Bevy events are buffered for two frames. With ~6 events per phase
  transition and transitions occurring at most once per 10 seconds (PLACEMENT timer),
  the event buffer holds at most ~12 event structs between GC cycles. Each struct is
  <64 bytes. Memory impact is immeasurable.
- **Per-player fan-out**: `ShopRefreshNeeded` is emitted once per player per DRAFT
  entry. In 1v1: 2 events. In 2v2: 4 events. All processed in the same `Update`
  run. No performance concern at any supported player count.
- **Network**: `BroadcastPhaseChanged` is the trigger for one `S2CPhaseChanged`
  Lightyear send per phase transition. Per ADR-008, this is on `ReliableChannel` and
  is ~20 bytes per broadcast. Network impact is identical to the pre-ADR baseline.
- **Load Time**: No impact — event type registration happens at plugin init and is
  amortised across the session lifetime.

---

## Migration Plan

This is a greenfield decision. No existing networking or system code reacts to RSM
phase transitions yet — the RSM itself has not been implemented.

Implementation sequence:

1. Define all message types in `server/core/rsm/events.rs` per the catalog above.
   Outbound RSM types derive `Message`; `SessionReady` derives `Event` (Observer only).
2. Register all message types in the RSM plugin: `app.add_message::<DraftStarted>()`, etc.
   Register `SessionReady` via `app.observe(on_session_ready)` — NOT `add_message`.
3. Implement `advance_phase` (per ADR-009) with `MessageWriter<T>` parameters for all
   outbound messages. Enforce F2 emission ordering by code order within each match arm.
4. Implement `rsm_input_reader` system: reads `AuctionSettled`, `ResolutionComplete`
   via `MessageReader<T>`; updates RSM phase state; schedules `.before(advance_phase)`.
   `SessionReady` is handled by the RSM Observer (`on_session_ready`), not this system.
5. Implement subscriber systems for M1 events: Economy System (`DraftStarted`),
   Card Pool (`ShopRefreshNeeded`), Board/Lane System (`PlacementPhaseEntered`,
   `ResolutionPhaseEntered`), Game Session System (`GameOverEmitted`), Network Dispatch
   (`BroadcastPhaseChanged`).
6. Write integration tests for each subscriber (see Validation Criteria).
7. When M2 begins: implement `AuctionPhaseEntered` subscriber in Auction System and
   `ResolutionPhaseEntered` subscriber in Combat Resolution. Update this ADR's
   Subscriber Contracts table with confirmed M2 contracts before the M2 story is
   marked Done.

---

## Validation Criteria

Each criterion maps to one or more RSM acceptance criteria from `round-state-machine.md`.

- [ ] **Message catalog compiles**: All outbound types in `server/core/rsm/events.rs`
  derive `Message`, `Clone`, `Debug` and compile against Bevy 0.18 without deprecation
  warnings. `SessionReady` derives `Event` (Observer). `app.add_message::<T>()` used
  for all types except `SessionReady` (which uses `app.observe(on_session_ready)`).
- [ ] **F2 ordering — mana before income**: In a test that writes `DraftStarted` and
  runs the Economy System, `current_mana` is set before `gold` is credited. No ordering
  inversion observed. (RSM-6, RSM-7, RSM-8)
- [ ] **F2 ordering — income before shop**: In a test that writes `DraftStarted`
  followed by `ShopRefreshNeeded`, the Card Pool's `refresh_shop` does not execute
  before the Economy System's `apply_gold_income`. (RSM-10, RSM-32)
- [ ] **F2 ordering — broadcast last**: `S2CPhaseChanged` is not sent before all
  other phase-entry actions complete. Verified by asserting that the network dispatch
  system's send call does not precede the Economy or Card Pool subscriber systems in
  a logged execution trace. (RSM-32)
- [ ] **Per-player shop refresh in 1v1**: Exactly 2 `ShopRefreshNeeded` events are
  emitted per DRAFT entry in a 1v1 game. Each event carries a distinct `player` field.
  The Card Pool draws two independent shops. (RSM-11)
- [ ] **DRAFT_INITIAL income formula**: `DraftStarted { phase: Initial }` causes the
  Economy System to grant `starting_gold = 5`, not `baseline + interest`. (RSM-6)
- [ ] **Inbound guard — stale AuctionSettled discarded**: If `AuctionSettled` is
  written while `phase != DraftAuction`, the RSM does not transition. (RSM-31, Edge Cases)
- [ ] **Inbound guard — stale ResolutionComplete discarded**: If `ResolutionComplete`
  is written while `phase != Resolution`, the RSM does not transition. (RSM-35)
- [ ] **GameOverEmitted triggers teardown**: The Game Session System's teardown logic
  executes when `GameOverEmitted` is written. `SessionConfig` and `ServerRng` resources
  are removed from the World. (RSM-36, TR-GSS-07)
- [ ] **BroadcastPhaseChanged → S2CPhaseChanged**: The network dispatch system sends
  exactly one `S2CPhaseChanged` per `BroadcastPhaseChanged` event, on `ReliableChannel`,
  to `NetworkTarget::All`. No `S2CPhaseChanged` is sent without a preceding
  `BroadcastPhaseChanged` in the same frame. (RSM-26)

---

## GDD Requirements Addressed

| GDD System | Section / Rule | Requirement | How This ADR Addresses It |
|---|---|---|---|
| `round-state-machine.md` | Rule 3 — Economy events at DRAFT entry | RSM fires `apply_mana_ramp` and `apply_gold_income` on DRAFT entry before accepting player input | `DraftStarted` event; Economy System subscriber; F2 emission ordering (step 1 before broadcast) |
| `round-state-machine.md` | Rule 5 — Shop refresh timing | RSM fires `refresh_shop(player)` for all players immediately after economy events on DRAFT entry | `ShopRefreshNeeded { player }` event; Card Pool subscriber; F2 ordering (step 2 after step 1) |
| `round-state-machine.md` | Rule 7 — DRAFT_AUCTION behavior | RSM sends `StartAuction(round_number)` to the Auction System on DRAFT_AUCTION entry | `AuctionPhaseEntered { round }` event; Auction System [M2] subscriber; F2 step 3 (before broadcast) |
| `round-state-machine.md` | Rule 10 — RESOLUTION | RSM signals Combat Resolution System to execute all six sub-steps | `ResolutionPhaseEntered { round }` event; Combat Resolution [M2] subscriber |
| `round-state-machine.md` | Rule 11 — GAME_OVER detection | After RESOLUTION completes, RSM evaluates real_objectives_destroyed | `ResolutionPhaseEntered` read by Objective System; `ResolutionComplete` inbound event triggers RSM evaluation |
| `round-state-machine.md` | Rule 14 — Phase broadcast | Every state transition broadcasts `S2CPhaseChanged` after new state is entered and all entry actions have fired | `BroadcastPhaseChanged` event; network dispatch subscriber; F2 ordering (ALWAYS LAST) |
| `round-state-machine.md` | Formula F2 — Phase Entry Sequence | Server fires events in strict order: mana → income → shop → auction (if applicable) → broadcast | Emission ordering in `advance_phase` match arms enforces F2 steps 1–5 by linear code order |
| `round-state-machine.md` | TR-RSM-04 | RSM emits economy signals at phase entry | `DraftStarted` event covers TR-RSM-04 |
| `round-state-machine.md` | TR-RSM-05 | RSM emits shop refresh at phase entry | `ShopRefreshNeeded` event covers TR-RSM-05 |
| `economy-system.md` | Rule 2 — Current-round mana | Economy System sets mana on DRAFT entry | Economy System reads `DraftStarted`; TR-ECO-02 |
| `economy-system.md` | Rule 3/4 — Gold income and interest | Economy System applies baseline + interest on DRAFT entry | Economy System reads `DraftStarted { phase }`; TR-ECO-04, TR-ECO-05 |
| `round-state-machine.md` | TR-BLS-05 — Placement entry | Board/Lane System opens placement buffer on PLACEMENT entry | `PlacementPhaseEntered` event; Board/Lane System subscriber |
| `game-session-system.md` | Session teardown | Game Session System destroys SessionConfig + ServerRng on GAME_OVER | `GameOverEmitted` event; Game Session System subscriber; TR-GSS-07 |

---

## Related

- `docs/architecture/adr-009-rsm-state-machine.md` — Defines the `advance_phase`
  function and RSM state structure. ADR-010 is a companion: ADR-009 defines the
  state machine; this ADR defines what it emits at each transition.
- `docs/architecture/adr-003-cargo-workspace-structure.md` — Defines `server/core/`
  and `server/feature/` module boundaries that this ADR's zero-import constraint relies on.
- `docs/architecture/adr-005-server-side-rng.md` — `ServerRng` is torn down on
  `GameOverEmitted`; ADR-005 specifies the RNG resource lifecycle.
- `docs/architecture/adr-007-placement-buffer.md` — `PlacementPhaseEntered` triggers
  the buffer opening described in ADR-007.
- `docs/architecture/adr-008-lightyear-channel-config.md` — `BroadcastPhaseChanged`
  triggers a `ReliableChannel` broadcast; the channel assignment is defined in ADR-008.
- `design/gdd/round-state-machine.md` — Primary source for F2 emission ordering,
  phase rules, and acceptance criteria (RSM-1 through RSM-38).
- `design/gdd/economy-system.md` — `DraftStarted` subscriber contract; income
  formulas (Rules 2–4) that the Economy System applies on receiving `DraftStarted`.
- `design/gdd/game-session-system.md` — `SessionReady` inbound event source;
  `GameOverEmitted` subscriber (session teardown).
