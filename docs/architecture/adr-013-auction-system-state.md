# ADR-013: Auction System State Machine and Bid Processing Architecture

## Status

Accepted

## Date

2026-04-30

## Decision Makers

User + gameplay-programmer + engine specialist

## Summary

The Auction System's authoritative state is stored in a single `AuctionState` resource on the
server (parallel to `RoundState` in ADR-009). A single system — `auction_tick_system` — is the
sole writer. Per-frame execution order is enforced by code order within that system: inbound
control messages first, then bid drain (using Lightyear's `MessageReceiver<C2SAuctionBid>`),
then timer decrement, then RESOLVING transition and settlement. Economy System gold operations
(reserve, release, spend) are invoked via `api.rs` functions on `ResMut<PlayerEconomies>` to enforce the
release-before-reserve invariant atomically within one system run. The `auction_snapshot()`
function is a pure function on `&AuctionState`.

---

## Engine Compatibility

| Field | Value |
|-------|-------|
| **Engine** | Bevy 0.18 + Lightyear 0.26 |
| **Domain** | Core |
| **Knowledge Risk** | HIGH — Bevy 0.15–0.18 are all post-cutoff |
| **References Consulted** | `docs/engine-reference/bevy/VERSION.md`, `docs/engine-reference/bevy/breaking-changes.md`, `docs/engine-reference/bevy/deprecated-apis.md`, ADR-009, ADR-010 |
| **Post-Cutoff APIs Used** | `#[derive(Resource)]`, `Res<T>` / `ResMut<T>` system params (stable); `#[derive(Message)]` + `MessageReader<T>` + `MessageWriter<T>` + `app.add_message::<T>()` (Bevy 0.17+ Message/Event split — `EventReader`/`EventWriter` removed); Lightyear 0.26 `MessageReceiver<T>` for C2S inbound messages (distinct from Bevy's `MessageReader<T>`); `Time::delta().as_millis()` for timer ticking |
| **Verification Required** | (1) **RESOLVED**: Lightyear 0.26 C2S receiver system param is `MessageReceiver<T>` with `receiver.receive_messages()` — confirmed in `docs/engine-reference/bevy/current-best-practices.md` (Lightyear 0.26 pattern). `MessageReceiver<C2SAuctionBid>` is correct; Bevy's `MessageReader<T>` is a distinct API and must not be used for Lightyear network messages. (2) **RESOLVED (implementation-time)**: Economy income/interest systems run at `DraftStarted` (DRAFT entry) — mutually exclusive with LIVE_BIDDING. `ResMut<PlayerEconomies>` held by `auction_tick_system` during DRAFT_AUCTION does not conflict. Verify via Bevy schedule graph dump when registering systems. (3) **RESOLVED**: `u32::try_from(time.delta().as_millis()).unwrap_or(u32::MAX)` is the form specified in the Decision text — no `as u32` cast. Implementation follows the specified form. |

> **Critical API boundary — Lightyear vs Bevy messages:**
> Bevy's `MessageReader<T>` / `MessageWriter<T>` are for Bevy's internal buffered message bus
> (registered via `app.add_message::<T>()`). Lightyear's network messages use a separate
> `MessageReceiver<T>` / `MessageSender<T>` system param pair (registered via Lightyear's
> `ProtocolPlugin`). **Do not use `MessageReader<C2SAuctionBid>` for a Lightyear C2S message** —
> it will compile only if the type is also registered on Bevy's bus, which is incorrect.
> All bid processing must go through Lightyear's `MessageReceiver<C2SAuctionBid>`.

---

## ADR Dependencies

| Field | Value |
|-------|-------|
| **Depends On** | ADR-009 (RoundState resource pattern — `AuctionState` mirrors this; single-resource, single-writer, sole `ResMut`); ADR-010 (message bus — `AuctionPhaseEntered` inbound trigger and `AuctionSettled` outbound signal are defined in that ADR's event catalog); ADR-002 (server authority — `AuctionState` is server-only); ADR-008 (Lightyear channel config — all S2C auction messages use reliable broadcast or unicast) |
| **Enables** | Economy System stories involving gold reservation/release; Card Data & Pool stories for `draw_auction_card()`; Network Protocol stories for C2SAuctionBid, S2CAuctionBidAccepted, S2CAuctionSettled wire types; Reconnect snapshot handler (`auction_snapshot()`) |
| **Blocks** | Any story involving auction bid validation, auction timer, or auction settlement — no phase-conditional auction system may be implemented until this ADR is Accepted and `AuctionState` resource shape is settled |
| **Ordering Note** | ADR-009 and ADR-010 must be Accepted. ADR-010 is the canonical source for `AuctionPhaseEntered` name. `auction-system.md` GDD and ADR-009 Key Interfaces used the old name `StartAuction` — both have been updated as part of this ADR's authoring pass. `AbortAuction` has been added to ADR-010's event catalog as part of this same pass. |

---

## Context

### Problem Statement

The Auction System is Lanes and Lies' signature mechanic — a live ascending auction running each
DRAFT_AUCTION phase. Its Bevy 0.18 implementation requires decisions on: (1) how to store the
four-state machine (IDLE → SELECTING → LIVE_BIDDING → RESOLVING → IDLE) in a Bevy resource,
(2) how to process inbound `C2SAuctionBid` Lightyear messages within a single server frame while
enforcing GDD Rule 6's ordering (drain bids before decrementing timer), (3) how to execute the
atomic gold reservation handoff (release prev leader → reserve new leader) without a cross-frame
race, and (4) how to expose a read-only snapshot for the reconnect handler. Without this decision,
the Auction System, Economy System, and reconnect handler will be implemented with incompatible
state-access patterns.

### Constraints

- **Server authority**: `AuctionState` is server-only; clients hold no auction state (ADR-002).
- **Rule 6 ordering**: Within each server tick, bid messages must be drained and processed in
  arrival order BEFORE the timer is decremented. A bid arriving in the same tick as timer-zero
  is processed before RESOLVING fires.
- **Release-before-reserve invariant**: `release_gold_reservation(prev_leader)` must complete
  before `reserve_gold(new_leader, amount)` is called. These two operations must be atomic within
  a single system run — no other system may observe the "two players reserved" intermediate state.
- **Single bid reader**: Lightyear's `MessageReceiver<C2SAuctionBid>` can only be drained once
  per frame — whichever system runs first consumes the messages. There must be exactly one system
  reading C2S auction bids. No standalone phase-gate handler may also read this receiver.
- **Reconnect snapshot**: `auction_snapshot()` must be callable at any point during LIVE_BIDDING
  (including in the same frame a bid was processed) and must not require a Bevy ECS query.
- **System scheduling**: `auction_tick_system` must be scheduled BEFORE `rsm_tick_system` so
  that `AuctionSettled` is readable by the RSM in the same frame (ADR-009 ordering contract:
  AuctionSystem → CombatResolutionSystem → rsm_tick_system).
- **Bevy 0.18**: No bundles, no `EventWriter`/`EventReader`, `Query::single()` returns `Result`.
- **Timer precision**: GDD Rule 6 specifies `u32` milliseconds with saturating subtraction.

### Requirements

- `AuctionState` is the single source of truth for all auction state on the server.
- Only `auction_tick_system` writes to `AuctionState` and drains C2S auction bids.
- Economy calls (`api::reserve_gold`, `api::release_gold_reservation`, `api::spend_gold`) execute via
  `api.rs` functions on `ResMut<PlayerEconomies>` — not cross-frame messages — to enforce atomicity.
- `auction_snapshot()` is a pure function on `&AuctionState`. No ECS query needed.
- `AuctionState` must be testable with `World::new()` — no live Lightyear session required.

---

## Decision

The Auction System's authoritative state lives in a single `AuctionState` resource on the server,
parallel to `RoundState` (ADR-009). A single system — `auction_tick_system` — is the sole
`ResMut<AuctionState>` writer and the sole drainer of Lightyear's `MessageReceiver<C2SAuctionBid>`.
Within each server frame, `auction_tick_system` executes in strict code order:

1. Read `MessageReader<AuctionPhaseEntered>` — if IDLE, draw auction card, initialise state, broadcast `S2CAuctionCard`, transition to LIVE_BIDDING.
2. Read `MessageReader<AbortAuction>` — if non-IDLE, release any reservation, return to IDLE silently (no `AuctionSettled` written).
3. If LIVE_BIDDING: drain Lightyear `MessageReceiver<C2SAuctionBid>` in arrival order. For each bid: validate 5 conditions; on accept, call `economy.release_gold_reservation(prev_leader)` then `economy.reserve_gold(new_leader, amount)` sequentially, update state, reset timer with `min(remaining + reset_ms, cap_ms)`, broadcast `S2CAuctionBidAccepted`; on reject, unicast `S2CAuctionBidRejected`.
4. If LIVE_BIDDING: `timer_remaining_ms = timer_remaining_ms.saturating_sub(u32::try_from(time.delta().as_millis()).unwrap_or(u32::MAX))`.
5. If LIVE_BIDDING and `timer_remaining_ms == 0`: transition to RESOLVING. Execute settlement synchronously (Case A: winner exists; Case B: no bids). Write `MessageWriter<AuctionSettled>`. Return to IDLE.

Economy System interactions use direct `ResMut<PlayerEconomies>` — not message-passing — so the
release-before-reserve invariant is enforced by the sequential call order within one system body,
with no system boundary between steps 1 and 2 of the handoff.

### Architecture Diagram

```
SERVER WORLD
┌──────────────────────────────────────────────────────────────┐
│  AuctionState (Resource — server only)                       │
│  ├── phase: AuctionPhase  (Idle|Selecting|LiveBidding|Resolving)│
│  ├── card_id: Option<CardId>                                 │
│  ├── current_price: u32   (starting_price until first bid)   │
│  ├── current_leader: Option<PlayerId>                        │
│  └── timer_remaining_ms: u32                                 │
│                                                              │
│  auction_tick_system  (scheduled BEFORE rsm_tick_system)     │
│                                                              │
│    Bevy-side readers (internal message bus):                 │
│      MessageReader<AuctionPhaseEntered>  → start auction     │
│      MessageReader<AbortAuction>         → abort/cleanup     │
│      Res<RoundState>                     → phase gate        │
│      Res<GameConfig>                     → timer constants   │
│      Res<Time>                           → tick delta        │
│                                                              │
│    Lightyear-side readers (network):                         │
│      MessageReceiver<C2SAuctionBid>      → bid input         │
│      (Only this system drains this receiver — sole reader)   │
│                                                              │
│    Writes:                                                   │
│      ResMut<AuctionState>               (sole writer)        │
│      ResMut<PlayerEconomies>            (reserve/release/spend via api.rs)│
│                                                              │
│    Lightyear-side senders (network):                         │
│      S2CAuctionCard         (reliable broadcast)             │
│      S2CAuctionBidAccepted  (reliable broadcast)             │
│      S2CAuctionBidRejected  (reliable unicast → bidder)      │
│      S2CAuctionSettled      (reliable broadcast)             │
│      S2CCardAcquired        (reliable unicast → winner)      │
│                                                              │
│    Bevy-side writer (internal message bus):                  │
│      MessageWriter<AuctionSettled>   → RSM reads this        │
│                                                              │
│  Per-frame execution order (code order, not system order):   │
│    1. Handle AuctionPhaseEntered (IDLE→SELECTING→LIVE_BIDDING)│
│    2. Handle AbortAuction (cleanup → IDLE, no AuctionSettled)│
│    3. Drain MessageReceiver<C2SAuctionBid> (bid validation)  │
│       └─ on accept: release prev, reserve new (atomic)       │
│    4. saturating_sub timer                                   │
│    5. if timer==0: RESOLVING → settlement → IDLE             │
│       └─ write MessageWriter<AuctionSettled>                 │
└──────────────────────────────────────────────────────────────┘

                     │ AuctionSettled (Bevy MessageWriter)
                     ▼
          rsm_tick_system reads → DRAFT_AUCTION → DRAFT_SHOP
```

### Key Interfaces

```rust
// server/src/auction/state.rs

use bevy::prelude::*;
use shared::protocol::{CardId, PlayerId};

/// Server-authoritative auction state machine state.
/// Only `auction_tick_system` may hold `ResMut<AuctionState>`.
#[derive(Resource)]
pub struct AuctionState {
    pub phase: AuctionPhase,
    /// None in Idle; set at Selecting entry from draw_auction_card().
    pub card_id: Option<CardId>,
    /// Starting price (rarity-based) until first accepted bid; last accepted bid thereafter.
    pub current_price: u32,
    /// None if no bid has been placed yet this auction.
    pub current_leader: Option<PlayerId>,
    /// Milliseconds remaining. u32 + saturating_sub — never underflows.
    pub timer_remaining_ms: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuctionPhase {
    Idle,
    Selecting,
    LiveBidding,
    Resolving,
}

impl Default for AuctionState {
    fn default() -> Self {
        Self {
            phase: AuctionPhase::Idle,
            card_id: None,
            current_price: 0,
            current_leader: None,
            timer_remaining_ms: 0,
        }
    }
}
```

```rust
// server/src/auction/snapshot.rs

use shared::protocol::{CardId, PlayerId};
use crate::auction::state::AuctionState;

pub struct AuctionSnapshot {
    pub card_id: CardId,
    /// Starting price if no bids placed; last accepted bid amount otherwise.
    pub last_accepted_bid: u32,
    pub current_leader: Option<PlayerId>,
    pub timer_remaining_ms: u32,
}

/// Returns None when AuctionPhase::Idle.
/// Pure function — no ECS query. Reconnect handler takes Res<AuctionState> and calls this.
/// Must be scheduled before auction_tick_system (GDD Rule 9 reconnect ordering requirement).
pub fn auction_snapshot(state: &AuctionState) -> Option<AuctionSnapshot> {
    if state.phase == AuctionPhase::Idle {
        return None;
    }
    state.card_id.map(|card_id| AuctionSnapshot {
        card_id,
        last_accepted_bid: state.current_price,
        current_leader: state.current_leader,
        timer_remaining_ms: state.timer_remaining_ms,
    })
}
```

```rust
// server/src/auction/system.rs — parameter list sketch
// (Lightyear receiver type to be confirmed per Verification Required item 1)

fn auction_tick_system(
    mut auction: ResMut<AuctionState>,
    mut economies: ResMut<PlayerEconomies>,
    round_state: Res<RoundState>,
    game_config: Res<GameConfig>,
    time: Res<Time>,
    // Bevy internal message bus:
    mut phase_entered: MessageReader<AuctionPhaseEntered>,
    mut abort: MessageReader<AbortAuction>,
    mut settled_writer: MessageWriter<AuctionSettled>,
    // Lightyear network (exact type pending Verification Required item 1):
    mut bids: MessageReceiver<C2SAuctionBid>,
    mut s2c_sender: MessageSender</* S2C auction message types */>,
    mut card_pool: ResMut<CardPool>,
) {
    // Step 1: handle AuctionPhaseEntered
    // Step 2: handle AbortAuction
    // Step 3: drain bids (if LiveBidding)
    //   timer_remaining_ms = timer_remaining_ms
    //       .saturating_sub(u32::try_from(time.delta().as_millis()).unwrap_or(u32::MAX));
    // Step 4: decrement timer
    // Step 5: settle if timer == 0
}
```

```rust
// server/src/auction/plugin.rs — scheduling

app.configure_sets(Update, AuctionSet::Tick.before(RsmSet::Tick));
// Also: reconnect_snapshot_system.before(auction_tick_system) per GDD Rule 9
```

### Economy Interface Used Within `auction_tick_system`

```rust
// Called in this exact sequence on an accepted bid (Rule 5 atomicity).
// Both api.rs calls are sequential in the same function body — no system boundary between them.
// prev_leader price is passed as the amount originally reserved.
if let Some(prev_leader) = auction.current_leader {
    if let Some(econ) = economies.0.get_mut(&prev_leader) {
        api::release_gold_reservation(econ, auction.current_price); // Step 1: release prev
    }
}
if let Some(econ) = economies.0.get_mut(&new_leader) {
    let _ = api::reserve_gold(econ, bid_amount); // Step 2: reserve new (pre-validated by can_afford_bid)
}
```

---

## Alternatives Considered

### Alternative 1: ECS Components on a Dedicated Auction Entity

- **Description**: Spawn an `AuctionEntity` at game start and attach `AuctionPhaseComponent`,
  `AuctionTimerComponent`, `AuctionLeaderComponent`, etc. Systems query
  `Query<(&AuctionPhaseComponent, ...), With<AuctionEntity>>`.
- **Pros**: Idiomatic Bevy ECS for per-entity state. Bevy change detection works automatically.
- **Cons**: `Query::single()` returns `Result` in Bevy 0.16+ — adds unwrap/error-handling
  boilerplate at every read site. Auction state is singleton global state, not per-entity state.
  Semantic mismatch. Resources are the idiomatic Bevy pattern for global singletons.
- **Rejection Reason**: Mirrors ADR-009's justified rejection of component-based RSM state.
  `Query::single()` boilerplate is unnecessary overhead for inherently singleton data.

### Alternative 2: Multiple Fine-Grained Resources

- **Description**: Split into `AuctionPhaseResource`, `AuctionTimerResource`,
  `AuctionBidStateResource` (leader + price). Each concern in a separate resource.
- **Pros**: Narrower `ResMut` borrows — read-only timer systems could avoid blocking bid writers.
- **Cons**: The release-before-reserve invariant (GDD Rule 5) requires `current_leader` and
  `reserved_gold` to update atomically. Splitting leader state across resources makes this
  impossible to enforce at the type level. `auction_tick_system` is the sole writer anyway,
  so the narrower-borrow benefit is unused.
- **Rejection Reason**: Splits cohesive invariant-carrying state without any concurrency benefit.
  Increases the risk of intermediate-state observation bugs.

### Alternative 3: Message-Passing for Economy Calls

- **Description**: Instead of direct `ResMut<PlayerEconomies>` access, emit messages like
  `ReserveGold { player, amount }` consumed by the Economy System in its own system.
- **Pros**: Decouples Auction System from Economy System at the Rust module boundary.
- **Cons**: Breaks the release-before-reserve invariant. If `ReleaseGoldReservation` and
  `ReserveGold` are processed by the Economy System in the next frame, there is a one-frame
  window where two players simultaneously have non-zero `reserved_gold`. GDD Rule 5 explicitly
  states: "Steps 1–2 are atomic. There is never a state where two players simultaneously have
  an active reservation." Cross-frame messaging cannot provide this guarantee.
- **Rejection Reason**: Violates the GDD's explicit reservation invariant.

---

## Consequences

### Positive

- `Res<AuctionState>` is O(1) to read from any system — single pointer dereference, no query.
- The release-before-reserve invariant (Rule 5) is enforced by sequential code within
  `auction_tick_system` — `release_gold_reservation` call precedes `reserve_gold` call in the
  same function body, with no await, yield, or system boundary between them.
- `auction_snapshot()` is a pure function on `&AuctionState` — trivially unit-testable, zero
  Bevy API surface, no ECS query overhead.
- Rule 6's bid-before-timer ordering is enforced by code order within `auction_tick_system` —
  the bid drain loop runs before `saturating_sub` — not by system ordering constraints that can
  silently drift.
- `AuctionState::default()` starts in `AuctionPhase::Idle` — insert it as a resource at plugin
  init and the system is safe regardless of whether `AuctionPhaseEntered` has arrived yet.
- `World::new()` tests can insert `AuctionState` and `PlayerEconomies` as resources and run
  `auction_tick_system` to test all 20 BLOCKING acceptance criteria without a live Lightyear
  session (once the Lightyear C2S receiver is abstracted behind a testable interface).

### Negative

- `auction_tick_system` has a wide parameter list: two `ResMut` (AuctionState, PlayerEconomies),
  four `Res` (RoundState, GameConfig, Time, CardPool), Bevy message readers/writers, and
  Lightyear senders/receivers. This is the cost of keeping the release-before-reserve invariant
  within a single system — the function cannot be decomposed further without breaking it.
- Direct `ResMut<PlayerEconomies>` access creates a compile-time coupling between
  `server/feature/auction` and `server/core/economy`. An interface trait could decouple them,
  but adds complexity outside hackathon scope.
- `AbortAuction` was absent from ADR-010's event catalog (despite being defined in ADR-009).
  This has been corrected as part of this ADR's authoring pass.

### Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Lightyear 0.26 C2S receiver type is not `MessageReceiver<T>` | MEDIUM | Compilation failure | Verify exact system param name from Lightyear 0.26 docs before implementing bid drain. Isolate C2S reading behind a helper function returning `impl Iterator<Item = C2SAuctionBid>` to keep bid-validation logic testable regardless of Lightyear API shape. |
| A second system accidentally drains `MessageReceiver<C2SAuctionBid>` | LOW | Silently lost bids — some players' bids are never validated | `auction_tick_system` is the sole system in the auction plugin. Code review gate: `MessageReceiver<C2SAuctionBid>` appears in exactly one system. |
| `ResMut<PlayerEconomies>` scheduling conflict with economy income systems | LOW | Bevy runtime panic | Economy income/interest systems run at DRAFT entry (mutually exclusive with LIVE_BIDDING). Verify via Bevy schedule graph dump. `AuctionSet::Tick.before(EconomySet::Tick)` ordering within `Update` provides compile-time enforcement. |
| `as u32` truncation lint on `time.delta().as_millis()` | LOW | Clippy CI failure | Use `u32::try_from(time.delta().as_millis()).unwrap_or(u32::MAX)` to be explicit. No runtime impact (frame delta never exceeds ~100 ms). |

---

## GDD Requirements Addressed

| GDD System | Requirement | How This ADR Addresses It |
|------------|-------------|--------------------------|
| `auction-system.md` | Rule 1 — `AuctionPhaseEntered` triggers auction; guard: non-IDLE receipt is discarded | `auction_tick_system` reads `MessageReader<AuctionPhaseEntered>`; guards on `auction.phase == AuctionPhase::Idle` |
| `auction-system.md` | Rule 4 — Bid validation: 5 conditions must all pass | All 5 checks run inside bid drain loop in `auction_tick_system`, in the same code block |
| `auction-system.md` | Rule 5 — Steps 1–2 (release → reserve) are atomic; reservation invariant | Sequential `api::release_gold_reservation` → `api::reserve_gold` within one function body; no system boundary between them; both called on `economies.0.get_mut(&player_id)` entries |
| `auction-system.md` | Rule 6 — Timer tick order: drain bids BEFORE decrement | Code order: bid drain loop at step 3, `saturating_sub` at step 4 |
| `auction-system.md` | Rule 7 — Resolution: spend, add card, broadcast S2CAuctionSettled, fire AuctionSettled | RESOLVING branch in `auction_tick_system`; all operations within same system run; `MessageWriter<AuctionSettled>` written to trigger RSM transition |
| `auction-system.md` | Rule 8 — AbortAuction: cancel timer, release reservation, IDLE, no AuctionSettled | `MessageReader<AbortAuction>` handling at step 2; release reservation, `phase = Idle`, no `AuctionSettled` write |
| `auction-system.md` | Rule 9 — `auction_snapshot()` for reconnect; reconnect system before auction tick | Pure `&AuctionState` function; reconnect handler scheduled before `auction_tick_system` |
| `auction-system.md` | Rule 10 — Timer in u32 ms; saturating subtraction | `timer_remaining_ms: u32`; `saturating_sub` + `u32::try_from(time.delta().as_millis()).unwrap_or(u32::MAX)` |
| `auction-system.md` | AU14 — `reserved_gold == 0` for all players at SELECTING entry | Interest snapshot invariant (GDD Edge Cases) guarantees no outstanding reservations at DRAFT entry; no reservation carry-over from prior phases |
| `round-state-machine.md` | Rule 7 — RSM scheduled after Auction System | `AuctionSet::Tick.before(RsmSet::Tick)` in Bevy `SystemSet`; `AuctionSettled` readable by rsm_tick_system in same frame |

---

## Performance Implications

- **CPU**: In `AuctionPhase::Idle` (most frames), `auction_tick_system` reads two empty Bevy
  message queues and one empty Lightyear receiver, then returns — < 1 µs. In LIVE_BIDDING: bid
  drain is O(n bids) where n ≤ 2 in 1v1; timer decrement is O(1). Total budget: < 0.1 ms.
- **Memory**: `AuctionState` is a fixed-size struct — < 64 bytes. No allocations during operation.
- **Network**: At most one S2CAuctionBidAccepted and one S2CAuctionBidRejected per server tick.
  S2CAuctionSettled and S2CCardAcquired fire once per auction. Negligible bandwidth impact.
- **Load Time**: `AuctionState` inserted as a resource at plugin setup. No asset loading.

---

## Migration Plan

Greenfield — no existing auction code in the codebase.

1. Define `AuctionState`, `AuctionPhase`, `AuctionSnapshot` in `server/src/auction/state.rs`
   and `server/src/auction/snapshot.rs`.
2. Implement `auction_snapshot()` as a free function.
3. Implement `auction_tick_system` in `server/src/auction/system.rs`.
4. Register `AuctionState` as a `Resource` and `auction_tick_system` in
   `server/src/auction/plugin.rs`.
5. Configure `AuctionSet::Tick.before(RsmSet::Tick)` in the server's `Update` schedule.
   Configure reconnect snapshot system before `auction_tick_system`.
6. Verify Lightyear 0.26 C2S receiver API (Verification Required item 1) before implementing
   the bid drain loop. Abstract behind a helper to preserve testability.
7. Use `u32::try_from(time.delta().as_millis()).unwrap_or(u32::MAX)` for the timer cast.

---

## Validation Criteria

- [ ] `AuctionState` resource inserts cleanly into `World::new()` with `AuctionPhase::Idle`
  and zero values.
- [ ] All 20 BLOCKING acceptance criteria (AU1-a through AU20, M7-a) have corresponding unit
  tests in `tests/unit/auction/` that pass using `World::new()` + message injection — no
  Lightyear session required (abstract Lightyear C2S receiver per Verification Required item 1).
- [ ] `ResMut<AuctionState>` appears in exactly one system (`auction_tick_system`) — code review
  gate on every auction PR.
- [ ] Lightyear `MessageReceiver<C2SAuctionBid>` (or confirmed equivalent) appears in exactly
  one system — code review gate.
- [ ] `auction_tick_system` is scheduled before `rsm_tick_system` — verified by Bevy schedule
  graph dump.
- [ ] Bid drain runs before timer decrement in every invocation — verified by AU13 test
  (same-tick duplicate bid → only first accepted).
- [ ] `release_gold_reservation(prev_leader)` call site always precedes `reserve_gold(new_leader)`
  in source — verified by AU20 test (release-before-reserve invariant).
- [ ] `auction_snapshot()` returns `None` exactly when `phase == AuctionPhase::Idle` — verified
  by AU10 test.

---

## Related Decisions

- `docs/architecture/adr-009-rsm-phase-state.md` — `RoundState` resource pattern that
  `AuctionState` mirrors; scheduling contract `AuctionSystem → rsm_tick_system`.
- `docs/architecture/adr-010-rsm-event-bus.md` — Event catalog defining `AuctionPhaseEntered`
  (inbound trigger) and `AuctionSettled` (outbound signal); `AbortAuction` added to catalog
  as part of this ADR's authoring pass.
- `docs/architecture/adr-002-client-server-authority.md` — Server authority; `AuctionState`
  is server-only.
- `docs/architecture/adr-008-lightyear-channel-config.md` — Reliable channel for all S2C
  auction messages.
- `design/gdd/auction-system.md` — Complete auction specification; all BLOCKING acceptance
  criteria this ADR enables. Updated as part of this pass: `StartAuction` → `AuctionPhaseEntered`;
  `AuctionSettled` field names updated.
- `design/gdd/round-state-machine.md` — RSM phase sequence; DRAFT_AUCTION entry/exit;
  `AbortAuction` on disconnect.
- `design/gdd/economy-system.md` — `api::reserve_gold`, `api::release_gold_reservation`,
  `api::spend_gold` API contracts used within `auction_tick_system` via `ResMut<PlayerEconomies>`.
