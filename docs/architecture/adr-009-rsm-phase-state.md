# ADR-009: Round State Machine Phase Representation as ECS Resource

## Status

Accepted

## Date

2026-04-29

## Last Verified

2026-04-29

## Decision Makers

User + Lead Programmer + technical-director

## Summary

The Round State Machine's authoritative phase state (`RoundPhase`, `round_number`,
timers, submission tracking, and disconnect trackers) is stored in a single plain
Rust `RoundState` resource on the server. Phase transitions are driven by a
dedicated RSM system that reads events and writes this resource. No phase state
lives in ECS components on entities. The resource is the single source of truth
for all other systems that gate behaviour on the current phase; those systems read
`Res<RoundState>` directly — they do not replicate phase state as Lightyear
components.

---

## Engine Compatibility

| Field | Value |
|-------|-------|
| **Engine** | Bevy 0.18 + Lightyear 0.26 |
| **Domain** | Core / Networking |
| **Knowledge Risk** | HIGH — Bevy 0.15–0.18 are all post-cutoff; Required Components API (0.15), `Query::single()` returning `Result` (0.16), and ECS resource APIs must be verified |
| **References Consulted** | `docs/engine-reference/bevy/VERSION.md`, `design/gdd/round-state-machine.md`, ADR-002, ADR-008 |
| **Post-Cutoff APIs Used** | `Resource` derive macro (stable across versions but 0.18 patterns should be verified); `Res<T>` / `ResMut<T>` system params; `MessageReader<T>` / `MessageWriter<T>` (Bevy 0.17+ Message/Event split — `EventReader`/`EventWriter` no longer exist); `#[derive(Message)]` for buffered inter-system signals |
| **Verification Required** | (1) Confirm `Res<T>` / `ResMut<T>` system param API is unchanged in 0.18. (2) Confirm Bevy 0.18 buffered Message pattern: `AuctionSettled` and `ResolutionComplete` use `#[derive(Message)]` + `MessageReader<T>` + `app.add_message::<T>()`; `SessionReady` uses `#[derive(Event)]` + `Observer` per ADR-012. (3) Confirm `SystemSet` ordering API for scheduling the RSM after dependent systems. |

> **Note**: Knowledge Risk is HIGH. Any Bevy upgrade from 0.18 requires re-validating
> the event/observer split and resource API patterns before touching the RSM system.

## ADR Dependencies

| Field | Value |
|-------|-------|
| **Depends On** | ADR-002 (client-server authority model — server is authoritative; `RoundState` lives only on the server); ADR-008 (channel config — `S2CPhaseChanged` routes on `ReliableChannel`) |
| **Enables** | All RSM-dependent system implementations: Economy System (reads phase for income timing), Auction System (waits for `StartAuction` event), Combat Resolution System (waits for `BeginResolution` event), Objective System (read-only at RESOLUTION end), Board/Lane System (listens for `OnResolutionEnd`) |
| **Blocks** | Any story that gates gameplay behaviour on the current round phase. No phase-conditional system may be implemented until this ADR is Accepted and the `RoundState` resource shape is settled. |
| **Ordering Note** | ADR-002 must be Accepted first (or provisionally accepted as in the current state of the registry) because this ADR's decision that `RoundState` lives only on the server presupposes an authoritative-server model. ADR-008 must be Accepted before any `S2CPhaseChanged` send code is written. |

## Context

### Problem Statement

The Round State Machine drives all phase-gated behaviour in Lanes and Lies. Every
gameplay system — Economy, Auction, Combat Resolution, Objective, Board/Lane, and
Network Protocol — must know the current phase to decide whether to accept input,
fire effects, or stay dormant. A decision is needed on how this shared phase state
is represented in Bevy 0.18: as an ECS resource, as components on a dedicated
entity, as a replicated Lightyear component, or as a global static. Without this
decision, individual programmers will implement incompatible phase-read patterns,
and the RSM will have no stable interface for other systems to depend on.

### Constraints

- **Server authority**: All game state is server-authoritative (ADR-002). The
  client holds a read-only phase mirror for UI only — it must never drive
  transitions.
- **Lightyear replication scope**: ADR-001 establishes that Lightyear 0.26 has
  no per-component replication scope. Replicating `RoundPhase` as a Lightyear
  component would broadcast it to all clients with no ability to scope it — which
  is acceptable for phase (all clients should see it), but the approach requires
  explicit documentation of this choice.
- **Bevy 0.18 Required Components**: Bundles are deprecated. Any component-based
  approach must use the 0.15+ Required Components API.
- **Single writer**: The RSM system is the sole writer of phase state. Economy,
  Auction, Combat, and other systems are readers only.
- **Phase-gated input rejection**: Every C2S message handler must be able to
  check the current phase cheaply — O(1) — and reject messages that arrive in
  the wrong phase (RSM Rules 15, 27–29 ACs).
- **Test isolation**: The RSM must be testable with `World::new()` — a real
  Lightyear session is not required to test phase transitions.

### Requirements

- `RoundState` is the single source of truth for phase on the server.
- Other systems access phase via `Res<RoundState>` — no phase state is duplicated
  across resources.
- Phase transitions are driven by a single dedicated RSM system, not by multiple
  systems each mutating phase independently.
- `S2CPhaseChanged` is emitted on every transition; the channel and ordering rules
  from ADR-008 apply.
- The `RoundState` resource must support full-snapshot delivery on reconnect
  (Network Protocol GDD open question 4 — the resource shape must include all
  fields needed for `S2CGameSnapshot`).
- The client holds `ClientPhaseView` — a separate, non-authoritative mirror
  resource populated only from `S2CPhaseChanged` messages — to drive UI.

## Decision

The RSM's authoritative state lives in a single `RoundState` resource on the
server. Phase transitions are executed by one dedicated system (`rsm_tick_system`)
that reads Bevy buffered events from other systems and writes `ResMut<RoundState>`.
No phase state is stored in ECS components on entities or replicated as a Lightyear
component. The client holds a separate `ClientPhaseView` resource that is updated
only by the `S2CPhaseChanged` message handler.

### Architecture Diagram

```
SERVER WORLD
┌──────────────────────────────────────────────────────────┐
│  RoundState (Resource — server only)                     │
│  ├── phase: RoundPhase                                   │
│  ├── round_number: u32                                   │
│  ├── placement_timer: Option<Timer>                      │
│  ├── draft_shop_timer: Option<Timer>                     │
│  ├── draft_initial_timer: Option<Timer>                  │
│  ├── auction_safety_timer: Option<Timer>                 │
│  ├── resolution_safety_timer: Option<Timer>              │
│  ├── submissions_received: HashSet<PlayerId>             │
│  └── disconnect_trackers: HashMap<PlayerId, f32>         │
│                                                          │
│  rsm_tick_system                                         │
│    reads:  MessageReader<AuctionSettled>  (see ADR-013)  │
│            MessageReader<ResolutionComplete>             │
│            Lightyear OnConnected / OnDisconnected        │
│    writes: ResMut<RoundState>                            │
│    sends:  MessageSender<S2CPhaseChanged>  (ReliableChannel) │
│            MessageSender<S2CGameOver>      (ReliableChannel) │
│    fires:  MessageWriter<OnResolutionEnd>                │
│            MessageWriter<AuctionPhaseEntered>            │
│            MessageWriter<BeginResolution>                │
│            MessageWriter<ApplyManaRamp>                  │
│            MessageWriter<ApplyGoldIncome>                │
│            MessageWriter<RefreshShop>                    │
│            MessageWriter<InterestSnapshot>               │
│                                                          │
│  All other systems gate on: Res<RoundState>.phase        │
│  (Economy, Auction, Combat, Objective, Board/Lane, C2S   │
│   message handlers)                                      │
└──────────────────────────────────────────────────────────┘

                        │  S2CPhaseChanged (ReliableChannel)
                        ▼

CLIENT WORLD
┌──────────────────────────────────────────────────────────┐
│  ClientPhaseView (Resource — client only)                │
│  ├── phase: RoundPhase                                   │
│  ├── round_number: u32                                   │
│  └── timer_duration_ms: u32                              │
│                                                          │
│  Updated only by: S2CPhaseChanged message handler        │
│  Read by: HUD system, shop UI, placement UI, timer UI    │
│  No authority — never drives transitions                 │
└──────────────────────────────────────────────────────────┘
```

### Key Interfaces

```rust
// server/src/rsm/state.rs

use bevy::prelude::*;
use std::collections::{HashSet, HashMap};

/// The server-authoritative round state machine state.
/// This is the single source of truth for the current phase.
/// Only `rsm_tick_system` may write to this resource.
#[derive(Resource)]
pub struct RoundState {
    pub phase: RoundPhase,
    pub round_number: u32,
    pub placement_timer: Option<Timer>,
    pub draft_shop_timer: Option<Timer>,
    pub draft_initial_timer: Option<Timer>,
    /// Safety timeout — must never fire in normal play.
    pub auction_safety_timer: Option<Timer>,
    /// Safety timeout — must never fire in normal play.
    pub resolution_safety_timer: Option<Timer>,
    pub submissions_received: HashSet<PlayerId>,
    pub disconnect_trackers: HashMap<PlayerId, f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RoundPhase {
    Lobby,
    DraftInitial,
    DraftAuction,
    DraftShop,
    Placement,
    Resolution,
    GameOver,
}

// client/src/rsm/view.rs

/// The client's read-only phase mirror. Populated from S2CPhaseChanged only.
/// Never used to drive server-side decisions.
#[derive(Resource, Default)]
pub struct ClientPhaseView {
    pub phase: RoundPhase,
    pub round_number: u32,
    pub timer_duration_ms: u32,
}
```

```rust
// Bevy buffered Messages used for inter-system signalling (server-side)
// All defined in server/src/core/rsm/events.rs
// NOTE: These use #[derive(Message)] not #[derive(Event)] — see ADR-010 and
// the Bevy 0.17+ Message/Event split. Register via app.add_message::<T>().

// AuctionSettled: canonical shape per ADR-013 (field names updated from winning_bid)
#[derive(Message)] pub struct AuctionSettled { pub winner: Option<PlayerId>, pub final_price: u32, pub card_id: CardId }
#[derive(Message)] pub struct ResolutionComplete;
#[derive(Message)] pub struct OnResolutionEnd;
// AuctionPhaseEntered: canonical name per ADR-013 (renamed from StartAuction per ADR-010)
#[derive(Message)] pub struct AuctionPhaseEntered { pub round: u32 }
#[derive(Message)] pub struct AbortAuction;
#[derive(Message)] pub struct BeginResolution;
#[derive(Message)] pub struct ApplyManaRamp { pub player: PlayerId }
#[derive(Message)] pub struct ApplyGoldIncome { pub player: PlayerId }
#[derive(Message)] pub struct RefreshShop { pub player: PlayerId }
#[derive(Message)] pub struct InterestSnapshot { pub player: PlayerId }
```

```rust
// Phase-gate pattern used by all C2S message handlers

fn handle_c2s_auction_bid(
    mut bids: MessageReader<C2SAuctionBid>,
    round_state: Res<RoundState>,
    // ...
) {
    if round_state.phase != RoundPhase::DraftAuction {
        // Reject silently — wrong phase
        return;
    }
    // process bid...
}
```

### System Scheduling

The `rsm_tick_system` must be scheduled AFTER the Auction System and Combat
Resolution System within the same `Update` schedule, so that events they fire
(`AuctionSettled`, `ResolutionComplete`) are readable in the same frame:

```
SystemSet ordering (server Update schedule):
  AuctionSystem → CombatResolutionSystem → rsm_tick_system → MessageSendSystems
```

This follows RSM GDD Rules 7 and 10, which explicitly state that the RSM system
must be scheduled after the Auction and Combat Resolution systems.

## Alternatives Considered

### Alternative 1: Phase State as ECS Components on a Singleton Entity

- **Description**: Spawn a single `RsmEntity` at game start and attach
  `RoundPhase`, `RoundNumber`, timer components, etc. to it. Systems query
  `Query<&RoundPhase, With<RsmEntity>>` to read phase.
- **Pros**: Idiomatic Bevy ECS pattern. Components can be queried with standard
  Bevy query filters. Would align with any future entity-based RSM extensions.
- **Cons**: `Query::single()` returns `Result` in Bevy 0.16+, adding unwrap/error
  handling boilerplate at every call site. Resources are semantically cleaner for
  singleton global state — they communicate "there is exactly one of these" without
  any query complexity. Phase is global state, not per-entity state. Component
  access is slightly slower than resource access for a singleton.
- **Rejection Reason**: Resources are the idiomatic Bevy pattern for global
  singleton state. The component approach adds boilerplate with no benefit for
  data that is inherently not per-entity.

### Alternative 2: Replicate RoundPhase as a Lightyear Component

- **Description**: Mark `RoundPhase` with Lightyear's replication trait so it
  syncs automatically to all clients without explicit `S2CPhaseChanged` sends.
- **Pros**: Eliminates manual `MessageSender<S2CPhaseChanged>` calls. Reconnect
  sync is handled automatically by Lightyear's component snapshot system.
- **Cons**: ADR-001 documents that Lightyear 0.26 has no per-component replication
  scope — a replicated component broadcasts to all clients. This is acceptable for
  phase, but the implicit broadcast makes the data flow harder to reason about and
  audit. More critically, the ordered delivery guarantee from ADR-008 (OQ-D
  invariant: `S2CResolutionEvent` before `S2CPhaseChanged(DRAFT_SHOP)`) is
  impossible to enforce with component replication — Lightyear replication does not
  guarantee ordering relative to explicit `MessageSender` messages on `ReliableChannel`.
- **Rejection Reason**: Breaks the OQ-D ordering invariant documented in ADR-008.
  Phase transition must be sent as an explicit message on `ReliableChannel` after
  `S2CResolutionEvent`, which requires explicit `MessageSender` usage.

### Alternative 3: Global Static / Thread-Local State

- **Description**: Store phase in a global `static AtomicU8` or similar.
- **Pros**: Zero system param overhead. Accessible from any context.
- **Cons**: Violates the project's "no static singletons for game state" coding
  standard. Untestable in isolation (`World::new()` tests cannot reset global
  statics without UB risk). Incompatible with future multi-session server
  scenarios. Rejected categorically.
- **Rejection Reason**: Explicitly forbidden by project coding standards.

## Consequences

### Positive

- `Res<RoundState>` is the cheapest possible phase read — a direct resource
  lookup, O(1), no query, no unwrap.
- Phase transitions are auditable in one place (`rsm_tick_system`). No system
  can mutate phase without going through the RSM system.
- `RoundState` can be directly included in `S2CGameSnapshot` for reconnect state
  delivery — the fields are plain Rust data, not ECS component handles.
- `World::new()` tests can insert `RoundState` as a resource and assert phase
  transitions without a live Lightyear session. All 38 RSM acceptance criteria
  are testable this way.
- The client's `ClientPhaseView` resource mirrors the server shape, making
  client-side phase-conditional logic easy to write and reason about.
- System scheduling is explicit and enforced by Bevy's `SystemSet` ordering —
  the RSM is guaranteed to see `AuctionSettled` and `ResolutionComplete` events
  from the same frame.

### Negative

- All inter-system signalling from the RSM (economy events, shop refresh, etc.)
  uses Bevy buffered `Event` types. Each event type must be defined and
  registered. This is more boilerplate than direct system-to-system function calls.
- `disconnect_trackers: HashMap<PlayerId, f32>` inside `RoundState` grows with
  player count; in 1v1 this is negligible, but in future team modes the map
  must be iterated in a single pass per RSM tick (RSM GDD Rule 13 edge case:
  mutual disconnection requires evaluating all trackers before deciding outcome).
- Phase state is not automatically synced to clients — every transition requires
  an explicit `MessageSender<S2CPhaseChanged>` call. Missing a send is a silent
  bug (client stays in stale phase). Mitigated by: the RSM is the only system
  that transitions phase, so the send is co-located with the transition.

### Risks

- **Risk**: Bevy 0.17+ `Message`/`Event` split renamed `EventReader`/`EventWriter`
  to `MessageReader`/`MessageWriter` for buffered signals. Verification Required
  item (2) must be completed before implementing the RSM system.
  **Mitigation**: `AuctionSettled` and `ResolutionComplete` use `#[derive(Message)]`
  + `MessageReader<T>`. `SessionReady` uses `#[derive(Event)]` + Observer per ADR-012.
  No EventWriter/EventReader usage anywhere in RSM code — these no longer exist.

- **Risk**: A second system accidentally acquires `ResMut<RoundState>` and
  mutates phase outside the RSM tick, creating a split-brain state.
  **Mitigation**: Code review gate — `ResMut<RoundState>` may only appear in
  `rsm_tick_system`. All other systems must use `Res<RoundState>` (read-only).
  Lint or comment at resource definition site.

- **Risk**: The RSM tick system and a message handler both read `Res<RoundState>`
  in the same frame and the handler processes a message that should have been
  rejected (phase mismatch) because the RSM hasn't ticked yet.
  **Mitigation**: Message handlers run BEFORE `rsm_tick_system` in the schedule.
  Phase at message-receipt time is the correct phase to gate on — the RSM has
  not yet transitioned, so the reject is valid. Document this ordering contract.

## GDD Requirements Addressed

| GDD Document | Requirement | How This ADR Addresses It |
|---|---|---|
| `design/gdd/round-state-machine.md` | Rule 1 — Phase sequence: LOBBY → DRAFT_INITIAL → PLACEMENT → RESOLUTION → … | `RoundPhase` enum defines all valid phases; `rsm_tick_system` is the sole writer that enforces the transition graph |
| `design/gdd/round-state-machine.md` | Rule 14 — Phase broadcast: every transition broadcasts `S2CPhaseChanged` on reliable channel after entry actions complete | Explicit `MessageSender<S2CPhaseChanged>` in `rsm_tick_system`; ADR-008 `ReliableChannel` assignment enforced |
| `design/gdd/round-state-machine.md` | Rules 8–9 — DRAFT_SHOP and PLACEMENT timers; Rule 12 — DRAFT_INITIAL timer | `placement_timer`, `draft_shop_timer`, `draft_initial_timer` fields on `RoundState`; ticked by `rsm_tick_system` |
| `design/gdd/round-state-machine.md` | Rule 13 — Disconnection: `disconnect_trackers` map iterated in single pass | `disconnect_trackers: HashMap<PlayerId, f32>` on `RoundState`; iterated once per RSM tick to catch mutual disconnect |
| `design/gdd/round-state-machine.md` | Rule 9 — PLACEMENT: `submissions_received` tracks which players submitted | `submissions_received: HashSet<PlayerId>` on `RoundState` |
| `design/gdd/round-state-machine.md` | Rule 15 — Valid player actions per state: C2S messages rejected in wrong phase | Phase-gate pattern using `Res<RoundState>.phase` in every C2S handler |
| `design/gdd/round-state-machine.md` | RSM-31, RSM-34 — Double-transition guard: RSM must not transition twice from same state | Single `rsm_tick_system` writer; phase check at transition entry prevents double-fire |
| `design/gdd/round-state-machine.md` | RSM-33 — `round_number = 0` unreachable at any `is_auction_round` call site | `round_number` initialised to 1 at `RoundState` construction; set explicitly on DRAFT_INITIAL entry |
| `design/gdd/network-protocol.md` | Open Question 4 — Late-joiner / reconnect full state restore | `RoundState` is plain Rust data; all fields can be included in `S2CGameSnapshot` directly |
| `design/gdd/round-state-machine.md` | Rule 7 — RSM scheduled after Auction System; Rule 10 — RSM scheduled after Combat Resolution | `SystemSet` ordering constraint documents and enforces the required scheduling relationship |

## Performance Implications

- **CPU**: `Res<RoundState>` lookup is a single pointer dereference. Phase checks
  in C2S handlers add ~0 ns per frame outside PLACEMENT/RESOLUTION (zero contention).
  Timer tick in `rsm_tick_system` is ≤ 5 field updates per frame. Total RSM tick
  budget: < 0.05 ms.
- **Memory**: `RoundState` is a fixed-size struct plus two small collections
  (`HashSet<PlayerId>`, `HashMap<PlayerId, f32>`) — in 1v1, both hold exactly 2
  entries. Total: < 1 KB.
- **Network**: `S2CPhaseChanged` is sent at most once per phase transition — 7
  transitions per game in the typical case. Not a bandwidth concern.
- **Load Time**: `RoundState` is inserted as a resource at plugin setup; no
  asset loading required.

## Migration Plan

This is a greenfield decision — no existing RSM code exists in the codebase.

1. Define `RoundPhase` enum and `RoundState` resource in `server/src/rsm/state.rs`.
2. Define all inter-system event types in `server/src/rsm/events.rs`.
3. Define `ClientPhaseView` in `client/src/rsm/view.rs`.
4. Implement `rsm_tick_system` in `server/src/rsm/system.rs`.
5. Register `RoundState` as a resource and `rsm_tick_system` in the server's RSM
   plugin (`server/src/rsm/plugin.rs`).
6. Implement `ClientPhaseView` update system in `client/src/rsm/plugin.rs`.
7. Verify `MessageReader`/`MessageWriter` API in Bevy 0.18 before writing any
   messaging code (see Engine Compatibility Verification Required). Confirm
   `app.add_message::<T>()` registration pattern. `EventReader`/`EventWriter`
   no longer exist — do not use them.

## Validation Criteria

- [ ] `RoundState` resource compiles and inserts cleanly into a `World::new()`
  test without a live Lightyear session.
- [ ] All 38 RSM acceptance criteria (RSM-1 through RSM-38) have corresponding
  unit tests in `tests/unit/rsm/` that pass using only `World::new()` + event
  injection — no Lightyear session required.
- [ ] `ResMut<RoundState>` appears in exactly one system (`rsm_tick_system`) in
  the server codebase — verified by code review gate on every RSM PR.
- [ ] Phase-gate pattern (`if round_state.phase != X { return; }`) is present in
  every C2S message handler; verified by the first integration test that sends a
  message in the wrong phase and confirms rejection.
- [ ] `rsm_tick_system` is scheduled after `AuctionSystem` and
  `CombatResolutionSystem` in the `Update` schedule; verified by Bevy's
  system graph debug output (`bevy/dynamic_linking` + schedule graph dump).
- [ ] `S2CPhaseChanged` is sent exactly once per transition in all 38 RSM ACs
  test suite — no double-send, no missing send.

## Related Decisions

- `docs/architecture/adr-001-objective-identity-unicast.md` — Establishes
  `NetworkTarget::Single(ClientId)` unicast pattern used by `S2CGameOver` and
  future unicast RSM messages.
- `docs/architecture/adr-002-client-server-authority.md` (pending) — Establishes
  the authoritative-server model that this ADR presupposes.
- `docs/architecture/adr-008-lightyear-channel-config.md` — Defines
  `ReliableChannel` used by `S2CPhaseChanged` and `S2CGameOver`; establishes the
  OQ-D ordering invariant that ruled out Lightyear component replication for phase.
- `design/gdd/round-state-machine.md` — The complete RSM specification; all 38
  acceptance criteria this ADR must enable.
- `design/gdd/network-protocol.md` — Full message catalogue; `S2CPhaseChanged`
  and `S2CGameOver` payload schemas.
