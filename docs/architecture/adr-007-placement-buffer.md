# ADR-007: Placement Buffer and Simultaneous Reveal Architecture

## Status

Accepted

## Date

2026-04-29

## Last Verified

2026-04-29

## Decision Makers

User + gameplay-programmer + network-programmer + lead-programmer

## Summary

During PLACEMENT, each player's submitted cards are held in a plain Rust data structure (`PendingPlacements` resource) rather than spawned as ECS entities. All submissions are committed atomically and broadcast to both clients as a single `S2CPlacementReveal` message only after PLACEMENT closes. This prevents Lightyear component replication from leaking opponent placement data before the reveal, which would destroy the hidden-information mechanic that is central to the game's bluff system.

## Engine Compatibility

| Field | Value |
|-------|-------|
| **Engine** | Bevy 0.18 + Lightyear 0.26 |
| **Domain** | Networking / Core / Gameplay |
| **Knowledge Risk** | HIGH — Bevy 0.15–0.18 and Lightyear 0.26 are all post-cutoff |
| **References Consulted** | `docs/engine-reference/bevy/VERSION.md`, `design/gdd/network-protocol.md`, `design/gdd/board-lane-system.md`, `design/gdd/round-state-machine.md` |
| **Post-Cutoff APIs Used** | `MessageWriter::write()` / `MessageReader::read()` (Bevy 0.17+ — `EventWriter`/`EventReader` no longer exist); `Commands::spawn()` without Bundle (Bevy 0.15+ Required Components API); Lightyear 0.26 `ConnectionManager` unicast and `ReplicateTo` entity replication |
| **Verification Required** | (1) Verify `MessageWriter<T>` system param name in Bevy 0.18 — `EventWriter` was removed in 0.17. (2) Verify Lightyear 0.26 entity replication is NOT triggered until `ReplicateTo` / replication group is explicitly added — confirm no auto-replication occurs on `Commands::spawn`. (3) Verify `ConnectionManager` broadcast API shape for `S2CPlacementReveal`. |

> **Note**: Knowledge Risk is HIGH. This ADR must be re-validated if the project upgrades engine versions. Flag as "Superseded" and write a new ADR.

## ADR Dependencies

| Field | Value |
|-------|-------|
| **Depends On** | ADR-002 (server authority model — all game state is server-side; clients are read-only views); ADR-003 (workspace / crate layout — placement buffer lives in the server crate); ADR-009 (RSM phase events — `PlacementPhaseEntered` is the trigger that opens the buffer) |
| **Enables** | Combat Resolution system [M2] (sub-step 1 receives committed placements); Board Rendering [M2] (clients render from `S2CPlacementReveal`, not from pre-arrived replication) |
| **Blocks** | Board / Lane System implementation — the placement submission path cannot be built without this architecture settled |
| **Ordering Note** | ADR-009 (RSM phase event contracts) must be Accepted before the buffer's open/close lifecycle can be implemented. ADR-002 must be Accepted to confirm that no client-side placement state is permissible. |

## Context

### Problem Statement

Lanes and Lies requires both players to place cards simultaneously and secretly, with full reveal happening at a single atomic moment when PLACEMENT closes. This is not a cosmetic requirement — it is the primary hidden-information mechanic. If either player can observe the opponent's placement before the reveal, the bluff is broken.

Bevy ECS is the natural home for persistent game entities, and Lightyear replicates ECS components to clients automatically. This creates a direct conflict: if placed units are spawned as ECS entities during PLACEMENT, Lightyear's replication will propagate their `BoardPosition` and `CardOwner` components to both clients as soon as they appear in the world. There is no Lightyear 0.26 API to scope component replication to a single client at component granularity (see ADR-001 spike findings — the same constraint applies here). The opponent would silently receive the other player's placement data before the reveal fires, destroying the mechanic with no observable error.

The decision must be made now because it governs the entire placement submission pipeline. Every downstream system that touches placement — board rendering, combat resolution, network protocol — depends on this architecture being settled.

### Current State

This is a greenfield system. No placement code exists. The board-lane-system GDD (Rule 6, Architecture Note) specifies the constraint: "The buffer is a server-only data structure, not Bevy entities. Unit entities are spawned only at sub-step 1 commit, at which point they are added to the Lightyear replication group. Buffer contents never exist in the ECS world as entities, preventing any accidental replication to clients before the reveal."

This ADR documents, formalises, and provides detailed implementation guidance for that design decision.

### Constraints

- **Lightyear 0.26 replicates at entity granularity**: No per-component, per-client replication scope exists (confirmed in ADR-001 spike). Any ECS entity added to a replication group is replicated to all targeted clients for all replicated components.
- **Bevy 0.18 Required Components**: Spawning an entity with any replicated component will trigger replication. There is no "draft entity" concept in the Lightyear model.
- **All game logic is server-side**: Clients are views only (ADR-002). Placement validation and mana deduction happen exclusively on the server.
- **All-or-nothing submission**: The network protocol defines `C2SSubmitPlacement` as a single batch message. Partial acceptance is not supported (network-protocol.md, `C2SSubmitPlacement` notes).
- **Silent discard on validation failure**: Invalid submissions are silently discarded in their entirety — no error message is sent to the client (network-protocol.md Rule 4).

### Requirements

- Player A's placement must be invisible to Player B until `S2CPlacementReveal` fires, and vice versa.
- Both players must receive `S2CPlacementReveal` simultaneously (same message, broadcast, containing both players' placements in one payload).
- Validation (mana budget, spawn range, card ownership, hand size, occupancy) must run entirely server-side before writing to the buffer.
- Mana must be deducted at buffer commit time (on PLACEMENT close), not at ECS entity spawn time — this matches the binding-commitment model defined in board-lane-system.md Rule 6.
- A player who does not submit before the timer expires is treated as playing zero cards; existing board state is unchanged.
- Duplicate submissions from the same player in the same phase are silently discarded (the first accepted submission is final).
- The buffer must be fully cleared on each new PLACEMENT phase entry.
- ECS unit entities may only be spawned and added to Lightyear replication after `S2CPlacementReveal` has been enqueued on the reliable channel.

## Decision

Placement submissions are stored as plain Rust data in a `PendingPlacements` Bevy `Resource` on the server. No ECS entities are created for pending placements. At PLACEMENT close, the server:

1. Collects all valid submissions from `PendingPlacements`.
2. Deducts mana from the Economy System for each accepted placement.
3. Enqueues a single `S2CPlacementReveal` broadcast on the reliable channel to all clients.
4. Spawns ECS unit entities and adds them to the Lightyear replication group.
5. Emits the `PlacementCommitted` internal event to trigger Combat Resolution sub-step 1.
6. Clears `PendingPlacements`.

**2026-05-05 amendment - explicit placement mana split:** accepted pending data preserves the protocol submit split as `current_mana_spend` and `reserve_mana_spend`. The legacy `reserve_amount` shorthand is superseded. Board/Lane validates the explicit split through ADR-019 Economy APIs at submission time and applies the exact split at PLACEMENT close in step 2.

The ordering in step 3 before step 4 is the invariant that enforces the simultaneous reveal: clients receive the reveal message before any replicated ECS component from newly spawned units can arrive, because the entities do not exist until after the message is enqueued.

### Architecture

```
PLACEMENT phase open
        |
        v
+----------------------------+
|    C2SSubmitPlacement      |  (from either client, any time during PLACEMENT)
+----------------------------+
        |
        v
+-----------------------------------+
|   validate_placement_submission   |  (server-side, full batch)
|   - mana budget check             |
|   - spawn range check (F2)        |
|   - card ownership + hand check   |
|   - occupancy check               |
|   - duplicate submission check    |
+-----------------------------------+
        |                   |
   valid                 invalid
        |                   |
        v                   v
+------------------+   silently discarded
| PendingPlacements|   (no S2C sent)
|   Resource       |
| [PlayerA: Some]  |
| [PlayerB: None]  |
+------------------+
        |
        | (all submitted OR placement_timer expires)
        v
+--------------------------------------+
|   close_placement_phase (server)     |
|                                      |
|  1. Collect PendingPlacements        |
|  2. Deduct mana (Economy System)     |
|  3. Enqueue S2CPlacementReveal       |  <-- BOTH players receive this simultaneously
|     (broadcast, reliable channel)    |
|  4. Spawn ECS unit entities          |  <-- Lightyear replication begins HERE
|     + add to replication group       |
|  5. Emit PlacementCommitted event    |
|  6. Clear PendingPlacements          |
+--------------------------------------+
        |
        v
   RSM -> RESOLUTION
   Combat Resolution sub-step 1 fires
```

```
Client A                    Server                      Client B
    |                          |                             |
    |-- C2SSubmitPlacement --->|                             |
    |                          |--(buffer write, no ECS)--   |
    |                          |                             |
    |              (timer OR both submitted)                 |
    |                          |                             |
    |<--- S2CPlacementReveal --|--- S2CPlacementReveal ----->|
    |    (both players' cards) |    (both players' cards)    |
    |                          |                             |
    |       (ECS entities spawned + Lightyear replication begins)
    |<-- BoardPosition (repl) -|-- BoardPosition (repl) ---->|
```

### Key Interfaces

```rust
// server/src/feature/board/placement.rs

/// Server-side buffer holding validated placement submissions for the current
/// PLACEMENT phase. Cleared on PlacementPhaseEntered; populated by
/// handle_placement_submission; consumed by close_placement_phase.
///
/// Implements: board-lane-system.md Rule 6 (pending placement buffer)
/// ADR: ADR-007
#[derive(Resource, Default)]
pub struct PendingPlacements {
    pub submissions: HashMap<PlayerId, PlayerSubmission>,
}

pub struct PlayerSubmission {
    /// Validated placements for this player, in submission order.
    pub placements: Vec<PlacedCard>,
    /// Server time at submission receipt (f32 seconds). For audit log only.
    pub submitted_at: f32,
    /// True once C2SSubmitPlacement has been accepted for this player this phase.
    /// Second submissions arriving while is_final = true are silently discarded.
    pub is_final: bool,
}

pub struct PlacedCard {
    pub card_id: CardId,
    /// Absolute cell target for Minions, Traps, Structures.
    /// PlayTarget::LaneWide for Fields. PlayTarget::Instant for Orders.
    pub target: PlayTarget,
    /// Mana drawn from current-round mana for this card.
    pub current_mana_spend: u32,
    /// Mana drawn from reserve mana for this card.
    pub reserve_mana_spend: u32,
}

// Internal event — emitted after S2CPlacementReveal is enqueued and ECS entities
// are spawned. Consumed by Combat Resolution to begin sub-step 1.
#[derive(Event)]
pub struct PlacementCommitted {
    pub round_number: u32,
    pub committed_placements: HashMap<PlayerId, Vec<PlacedCard>>,
}
```

```rust
// Spawn range validation — implements board-lane-system.md Formula F2.
// Must be called for every PlacedCard whose target is PlayTarget::BoardCell
// before writing to PendingPlacements. Structures and Traps bypass this check.
fn validate_spawn_range(
    target_cell: u8,
    player: PlayerId,
    fakes_destroyed: u8,   // read from SpawnRangeState resource
) -> bool {
    match player {
        PlayerId::A => {
            let spawn_cell_a: u8 = 1;
            target_cell >= spawn_cell_a && target_cell <= spawn_cell_a + fakes_destroyed
        }
        PlayerId::B => {
            let spawn_cell_b: u8 = 8;
            target_cell >= spawn_cell_b - fakes_destroyed && target_cell <= spawn_cell_b
        }
    }
}
// NOTE: spawn_cell_A (1) and spawn_cell_B (8) are structural constants, not
// GameConfig fields. They reflect the physical board layout defined in
// board-lane-system.md Rule 1. Do not load them from config.
```

```rust
// Mana validation — full batch check before writing to PendingPlacements.
// Validates both current_mana and reserve_mana constraints simultaneously.
// On failure, the entire batch is silently discarded (all-or-nothing per
// network-protocol.md C2SSubmitPlacement notes).
fn validate_mana_budget(
    placements: &[PlacedCard],
    card_costs: &[(CardId, u32)],  // (card_id, cost) looked up from CardPool
    player_mana: &PlayerMana,
) -> bool {
    let total_reserve: u32 = placements.iter().map(|p| p.reserve_mana_spend).sum();
    let total_current: u32 = placements.iter().map(|p| p.current_mana_spend).sum();
    let all_splits_match_cost = placements
        .iter()
        .zip(card_costs.iter())
        .all(|(p, (_, cost))| p.current_mana_spend + p.reserve_mana_spend == *cost);
    all_splits_match_cost
        && total_reserve <= player_mana.reserve_mana
        && total_current <= player_mana.current_mana
}
```

### Implementation Guidelines

#### The Simultaneous Reveal Invariant

**This is the load-bearing constraint of this ADR. It must never be violated.**

> No ECS entity representing a placement may be spawned, and no Lightyear replication group may be modified, until `S2CPlacementReveal` has been enqueued on the reliable channel in the same system invocation that closes the PLACEMENT phase.

In practice: the system that calls `close_placement_phase` must:
1. Build the `S2CPlacementReveal` payload from `PendingPlacements`.
2. Call `connection_manager.send_message_to_target::<ReliableChannel, S2CPlacementReveal>(...)` (or equivalent Lightyear 0.26 broadcast API — verify symbol before implementing).
3. Only then call `commands.spawn(...)` for each placed unit.
4. Only then add the newly spawned entities to the Lightyear replication group (e.g., `ReplicateTo` component or equivalent 0.26 API — verify before implementing).

Steps 3 and 4 must not occur in a separate Bevy system scheduled in the same frame before step 2. If the reveal broadcast and entity spawn are split across two systems, they must be ordered with an explicit `before`/`after` constraint so that the broadcast system always runs first.

**Do not rely on network timing to save you.** Even if ECS replication and the broadcast message happened to arrive at clients in the "right" order by accident in testing, this would be undefined behavior. The ordering guarantee must be structural (enqueue before spawn) not temporal.

#### Buffer Lifecycle

```
PlacementPhaseEntered message received
    -> placement_buffer_open system runs
    -> PendingPlacements.submissions.clear()
    -> Phase is now open for C2SSubmitPlacement

C2SSubmitPlacement received (any time during PLACEMENT)
    -> handle_placement_submission system runs
    -> If player.is_final = true: silently discard (NP-14)
    -> Validate full batch (mana, spawn range, ownership, occupancy, hand size)
    -> If invalid: silently discard entire batch (not partial; NP-4, NP-5)
    -> If valid: write PlayerSubmission { placements, submitted_at, is_final: true }
    -> Do NOT modify ECS, do NOT spawn entities, do NOT send S2C

PLACEMENT close condition (either):
    (a) Both players have is_final = true in PendingPlacements
    (b) placement_timer reaches 0 (read from GameConfig.placement_timer_seconds)
    -> close_placement_phase system runs
    -> For each player without is_final: treat as empty submission (zero cards)
    -> Deduct mana via Economy System events
    -> Enqueue S2CPlacementReveal broadcast (reliable channel) -- MUST be first
    -> Spawn ECS entities for all placed cards
    -> Add entities to Lightyear replication group
    -> Emit PlacementCommitted { round_number, committed_placements }
    -> PendingPlacements.submissions.clear()
    -> RSM advances to RESOLUTION (via PlacementCommitted or direct RSM event)
```

#### Validation Rules (All-or-Nothing Per Player)

The server validates the complete batch atomically before writing to `PendingPlacements`. If any single card in the batch fails any check, the entire batch is discarded:

| Check | Failure condition | Source |
|---|---|---|
| Phase gate | Received outside PLACEMENT | network-protocol.md Rule 4 |
| Duplicate submission | `player.is_final = true` already | network-protocol.md NP-14 |
| Card in hand | `card_id` not in `player.hand` | network-protocol.md `C2SSubmitPlacement` notes |
| Reserve mana | `sum(reserve_mana_spend) > player.reserve_mana` | network-protocol.md `C2SSubmitPlacement` notes |
| Current mana | `sum(current_mana_spend) > player.current_mana` | network-protocol.md `C2SSubmitPlacement` notes |
| Split sum | `current_mana_spend + reserve_mana_spend != card.cost` | network-protocol.md `C2SSubmitPlacement` notes |
| Lane range | `lane` outside 1–5 | network-protocol.md lane/cell validation contract |
| Cell range | `cell` outside 1–8 | network-protocol.md lane/cell validation contract |
| Spawn range (Minions) | F2 validation fails for `target_cell` | board-lane-system.md Formula F2 |
| Minion slot | Player already has Minion in target lane this round | board-lane-system.md Rule 3 |
| Trap occupancy | Player already has Trap at `(lane, cell)` | board-lane-system.md Rule 5 |
| Structure occupancy | Player already has Structure at `(lane, cell)` | board-lane-system.md Rule 5 |
| Field occupancy | Player already has Field in `lane` | board-lane-system.md Rule 5 |
| Hand size | `placements.len() > player.hand.len()` | implied by card-in-hand check |

Structures and Traps bypass spawn range validation — they may be placed on any of the player's 20 home cells (board-lane-system.md Rule 4, Exceptions).

#### Mana Deduction Timing

Mana is deducted at PLACEMENT close (step 2 of `close_placement_phase`), not at submission receipt. This matches the binding-commitment model: the placement is binding when the phase closes, and so is the mana cost. If GAME_OVER fires before sub-step 1 (e.g., both players disconnect simultaneously), the session ends and no refund is issued.

The mana deduction fires before `S2CPlacementReveal` is broadcast in the close sequence. Economy System emits `S2CGoldUpdate` as a side effect; this message will arrive at clients after `S2CPlacementReveal` on the reliable channel (enqueue order = receive order within the same channel). No special ordering is required — gold update arriving after the reveal is the expected and correct behavior.

#### Bevy System Scheduling

All placement systems live in the `Update` schedule on the server app. Required ordering:

```
handle_placement_submission
    (no ordering constraint relative to other systems — just reads C2S messages)

placement_timer_tick
    .before(close_placement_phase)

close_placement_phase
    .after(placement_timer_tick)
    .after(handle_placement_submission)
    -- this system enqueues S2CPlacementReveal AND spawns entities in sequence
    -- no split across two systems

Combat Resolution sub-step 1
    .after(close_placement_phase)
    -- reads PlacementCommitted event; must not run before it is emitted
```

#### Reconnect Handling

If a player reconnects during PLACEMENT after having already submitted (`is_final = true`), the `S2CGameSnapshot` carries `PlayerSnapshot.submitted = true`. The client skips the placement UI and renders "waiting for opponent." The buffer is not re-sent — the snapshot is the sole reconnect source (network-protocol.md NP-10, NP-22, edge case 2).

If `S2CPlacementReveal` arrives at a reconnected client before or at the same time as the snapshot (race on reconnect mid-close), receipt of `S2CPlacementReveal` is definitive — both placements are closed. The client renders the reveal payload regardless of local `submitted` state (network-protocol.md edge case 8).

#### File Location

```
server/src/feature/board/
    placement.rs          -- PendingPlacements resource, handle_placement_submission,
                             close_placement_phase, validate_spawn_range,
                             validate_mana_budget, PlacedCard, PlayerSubmission
    placement_tests.rs    -- unit tests (see Validation Criteria)
```

The `liv-bevy-018` and `liv-bevy-lightyear` skills are mandatory on all files in `server/src/feature/board/`. Do not write or review board code without both skills active.

## Alternatives Considered

### Alternative 1: Spawn ECS Entities with a "Hidden" Flag Component

- **Description**: Spawn unit entities immediately on `C2SSubmitPlacement`, but mark them with a `PlacementHidden` component. Lightyear visibility rules (Rooms, `NetworkVisibility`) are configured to suppress replication of entities with `PlacementHidden` to all clients. At reveal time, remove `PlacementHidden`, which triggers replication to both clients.
- **Pros**: Placement data lives in ECS from the start, which is idiomatic for Bevy. Queries over pending placements use the same API as queries over board entities.
- **Cons**: Lightyear 0.26 visibility primitives operate at entity granularity. Suppressing replication of an entity prevents all clients from seeing it — but removing the flag reveals it to all clients simultaneously, which is what we want. The fatal flaw is the silent-failure mode: if any system, plugin, or future Lightyear upgrade inadvertently changes the visibility scope of `PlacementHidden` entities, replication silently leaks to the opponent with no error, no log, and no observable symptom until the mechanic is noticed to be broken. For a bluff game, this is the highest-severity failure mode. Rejected for the same reason ADR-001 rejected the per-component scoping workaround.
- **Estimated Effort**: Similar to chosen approach, minus the `PendingPlacements` resource design. Plus ongoing maintenance cost of ensuring visibility suppression is never accidentally lifted.
- **Rejection Reason**: Silent-failure risk on the mechanic that is the entire point of the game. The buffer approach fails loudly — if an entity is not in the replication group, it simply does not exist on any client.

### Alternative 2: Separate "Placement" Replication Group per Player, Revealed at Close

- **Description**: Spawn entities immediately into a per-player `ReplicationGroup` that targets only that player's `ClientId`. At reveal time, modify the group to target both clients simultaneously.
- **Pros**: Still ECS-native. Player A can query their own pending placements from ECS.
- **Cons**: Modifying a Lightyear replication group at PLACEMENT close requires knowing the exact API for group retargeting in Lightyear 0.26 (post-cutoff — unverified). More importantly, retargeting a group fires delta replication (sending only the diff since last known state for the new target), not a fresh snapshot. The new target (the opponent) would receive incremental component updates for units they have never seen — this is undefined behavior in Lightyear and could produce desync. The guarantee of simultaneous atomic reveal cannot be achieved via replication group retargeting; it requires a single explicit message. This approach is more complex, less verifiable, and still requires `S2CPlacementReveal` for the guarantee, making the replication group work redundant.
- **Estimated Effort**: Significantly higher — requires deep Lightyear 0.26 internals knowledge not available in training data.
- **Rejection Reason**: Cannot provide the simultaneous-reveal guarantee via replication alone. Would require `S2CPlacementReveal` regardless, making it strictly more complex than the chosen approach with no benefit.

### Alternative 3: Client-Side Placement State with Server Validation Echo

- **Description**: Clients maintain their own pending placement state. The server only validates and acknowledges. Both clients submit independently and receive an "opponent ready" signal.
- **Pros**: Reduced server-to-client message traffic during PLACEMENT.
- **Cons**: Directly violates the server-authority model (ADR-002). Clients are not permitted to hold authoritative game state. This would also require the server to maintain a shadow copy for reconnect snapshots anyway.
- **Estimated Effort**: Equivalent, but adds ongoing violation of the architecture that will require correction later.
- **Rejection Reason**: Violates ADR-002. Not considered further.

## Consequences

### Positive

- The simultaneous reveal invariant is enforced structurally: placement entities cannot be replicated before the reveal because they do not exist in the ECS world until after the broadcast is enqueued. This is not a convention or a flag — it is a physical impossibility.
- `PendingPlacements` is a plain Rust struct — trivially testable with `World::new()` without a full Lightyear session. All validation logic (mana, spawn range, occupancy) can be unit-tested without network infrastructure.
- The reveal boundary is explicit and locatable: one function (`close_placement_phase`) owns the entire close sequence in a defined order. Debugging a reveal issue has one place to look.
- Reconnect handling is clean: the buffer persists on the server for the duration of PLACEMENT. If a player reconnects mid-phase, their `submitted` state and any buffered placement is intact. `S2CGameSnapshot` reflects `submitted: true` correctly without any additional logic.
- No ongoing per-frame cost: `PendingPlacements` is a resource read only on submission and on phase close. It does not participate in any query that runs every tick.

### Negative

- Placement data lives in two representations: first as `PendingPlacements` (plain Rust), then as ECS entities (post-reveal). Any system that needs to query "what was placed this round" before sub-step 1 commits must read from `PendingPlacements`, not from ECS. This requires contributors to understand the two-phase lifecycle.
- Mana validation must call the ADR-019 Economy explicit split API. The buffer validates the mana budget at submission time; the Economy System deducts at close time. These must stay in sync by sharing `validate_explicit_mana_split` / `apply_explicit_mana_split` rather than duplicating arithmetic.
- The `close_placement_phase` system is an ordering-critical singleton: the broadcast-before-spawn invariant must be maintained if the system is ever refactored. This risk is mitigated by the Implementation Guidelines above and by the BLS-01 unit test suite.

### Neutral

- Placement data is not ECS-queryable during PLACEMENT. This is a deliberate constraint — it is what enforces the invariant. Systems that are curious about "what is the opponent placing" should not exist. If a future system (e.g., a debug overlay) needs to query pending placements, it must read from `PendingPlacements` directly, not from ECS.
- The buffer is cleared on `PlacementPhaseEntered`, not on `PlacementPhaseExited`. This means the buffer is empty at phase entry (ready for new submissions) rather than retaining the previous round's data until the new phase begins. Either choice is valid; entry-clear is chosen because it makes the "buffer is the current round's data" invariant unambiguous.

## Risks

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|-----------|
| `close_placement_phase` is split into two systems by a future refactor, with entity spawn preceding broadcast | Low | Critical — breaks simultaneous reveal invariant | BLS-01 integration test asserts no `BoardPosition` component replication precedes `S2CPlacementReveal` delivery. Code comment on system marks the ordering constraint explicitly. |
| Lightyear 0.26 auto-replicates entities on spawn before `ReplicateTo` is added | Low | Critical — same failure mode | Verify `Commands::spawn` without explicit replication opt-in produces no replication (see Engine Compatibility Verification Required). Add test asserting entity spawned without `ReplicateTo` component does not appear in client world. |
| Mana validation in buffer becomes stale relative to Economy System model | Medium | High — invalid placements accepted | Board/Lane calls Economy's explicit split validation API. No independent placement mana arithmetic is allowed outside tests. |
| `placement_timer` and "all submitted" condition race in the same frame | Low | Medium — double-close | `close_placement_phase` is guarded by a phase flag; the RSM transitions away from PLACEMENT after close fires, making a second invocation a no-op. |
| Validation occupancy check diverges from ECS board state (stale reads) | Low | Medium — placements accepted that conflict with live board | Occupancy validation reads from `BoardOccupancy` resource (owned by Board/Lane System), which is updated synchronously after each sub-step commit. Validation runs before the next PLACEMENT phase, by which time occupancy reflects the previous round's committed state. |

## Performance Implications

| Metric | Before | Expected After | Budget |
|--------|--------|---------------|--------|
| CPU (per-frame, PLACEMENT phase) | N/A (new system) | < 0.1ms — HashMap read/write on C2SSubmitPlacement (at most 2 submissions per phase) | < 2ms total game logic budget |
| CPU (close_placement_phase) | N/A | < 0.5ms — validate, deduct, broadcast, spawn, emit. Executes once per PLACEMENT phase (~once per 10s). Not in hot path. | < 2ms |
| Memory | N/A | ~200 bytes per `PlayerSubmission` (at most 10 cards × ~16 bytes each + metadata). Two players = ~400 bytes peak. | Negligible |
| Network (S2CPlacementReveal) | N/A | < 200 bytes (2 players × up to 5 cards × ~16 bytes per PlacedCard + message header). Sent once per PLACEMENT phase on reliable channel. | < 1KB per round message budget |

`PendingPlacements` does not participate in any per-frame ECS query. Performance impact is negligible.

## Migration Plan

This is a new system with no existing implementation to migrate. No migration steps required.

If a future decision supersedes this ADR (e.g., Lightyear gains first-class per-client entity visibility that makes the buffer unnecessary):

1. Write a new ADR documenting the new approach and its rationale.
2. Mark this ADR as `Superseded by ADR-XXX`.
3. Migrate `PendingPlacements` data to the new representation.
4. Update `close_placement_phase` to use the new reveal mechanism.
5. Re-run BLS-01 through BLS-07 test suite to confirm simultaneous-reveal invariant is preserved under the new approach.

**Rollback plan**: If this implementation is found to have an unfixable bug before shipping, the fallback is Alternative 1 (ECS entities with `PlacementHidden` flag) with explicit integration tests that assert no component replication occurs before `S2CPlacementReveal`. This fallback carries the silent-failure risk documented above and should only be used as a last resort.

## Validation Criteria

- [ ] **BLS-01**: GIVEN Player A submits valid placements during PLACEMENT, WHEN the submission is accepted, THEN no ECS entity with `BoardPosition` or `CardOwner` component exists in the server `World` for those placements before `close_placement_phase` runs. (Unit test — `World::new()`, no Lightyear session required.)
- [ ] **BLS-02**: GIVEN Player A submits valid placements and PLACEMENT closes, WHEN `close_placement_phase` runs, THEN `S2CPlacementReveal` is enqueued on the reliable channel AND ECS unit entities are present in the `World` — in that system invocation, in that order. (Integration test — requires a live Lightyear session to inspect message enqueue order relative to entity spawn.)
- [ ] **BLS-03**: GIVEN both players have submitted, WHEN `S2CPlacementReveal` is delivered to each client, THEN the `placements` field contains entries for BOTH players' cards in a single message. No partial reveal (one player's cards arriving before the other's) is possible. (Integration test — NP-28.)
- [ ] **BLS-04**: GIVEN a placement batch where `sum(placements[i].reserve_mana_spend) > player.reserve_mana`, WHEN `C2SSubmitPlacement` is processed, THEN the entire batch is silently discarded, `PendingPlacements[player].is_final` remains `false`, and no S2C message is sent. (Unit test.)
- [ ] **BLS-05**: GIVEN Player A has 0 fakes destroyed and submits a Minion to `(lane: 1, cell: 2)`, WHEN `validate_spawn_range` is called, THEN it returns `false` and the batch is discarded. (Unit test — board-lane-system.md BL-5.)
- [ ] **BLS-06**: GIVEN Player A submits a valid `C2SSubmitPlacement` and then submits a second `C2SSubmitPlacement` in the same PLACEMENT phase, WHEN the second message is processed, THEN `PendingPlacements[Player_A].placements` is unchanged (first submission retained), `is_final` remains `true`, and no S2C message is sent. (Unit test — NP-14.)
- [ ] **BLS-07**: GIVEN `PlacementPhaseEntered` fires, WHEN `placement_buffer_open` runs, THEN `PendingPlacements.submissions` is empty for all players. (Unit test.)
- [ ] **TR-NP-06 (NP-6)**: GIVEN a player sends valid `C2SSubmitPlacement` during PLACEMENT, WHEN the server accepts it, THEN `player.submitted = true` in server state. No S2C message is sent to any player. (Unit test — acceptance criterion NP-6.)
- [ ] **TR-NP-10 (NP-10)**: GIVEN a client reconnects during PLACEMENT after having already submitted, WHEN `S2CGameSnapshot` is processed, THEN the reconnecting player's `PlayerSnapshot.submitted = true` and the placement UI is not re-presented. (Integration test — NP-10.)
- [ ] **TR-BLS-01 through TR-BLS-07**: All referenced TRs from the network-protocol.md and board-lane-system.md acceptance criteria pass under this architecture.

## GDD Requirements Addressed

| GDD Document | System | Requirement | How This ADR Satisfies It |
|-------------|--------|-------------|--------------------------|
| `design/gdd/board-lane-system.md` | Board / Lane System | Rule 6: "During PLACEMENT, submitted placements are validated and held in a per-player pending buffer — they are not immediately committed to the board." | `PendingPlacements` resource is the pending buffer. ECS entity spawn is deferred to `close_placement_phase`. |
| `design/gdd/board-lane-system.md` | Board / Lane System | Rule 6, Architecture Note: "The buffer is a server-only data structure (`PlacementBuffer` resource), not Bevy entities. Unit entities are spawned only at sub-step 1 commit, at which point they are added to the Lightyear replication group." | `PendingPlacements` resource holds plain Rust structs. `commands.spawn` is called in `close_placement_phase` after `S2CPlacementReveal` is enqueued. Replication group assignment follows immediately after spawn. |
| `design/gdd/board-lane-system.md` | Board / Lane System | Formula F2: Spawn range validation formula for Minion placement. | `validate_spawn_range` implements F2 exactly, reading `fakes_destroyed` from `SpawnRangeState` resource. Structures and Traps bypass this check per Rule 4 Exception. |
| `design/gdd/network-protocol.md` | Network Protocol | `S2CPlacementReveal`: "atomic simultaneous reveal; both players receive this as the sole signal that placement is closed. Client MUST render from this payload, not from pre-arrived component replication, to honour the simultaneous-reveal guarantee." | The invariant (broadcast before spawn) structurally prevents any component replication from arriving before `S2CPlacementReveal`. The guarantee is architectural, not advisory. |
| `design/gdd/network-protocol.md` | Network Protocol | NP-6: Silent submission — no S2C acknowledgement until `S2CPlacementReveal`. | `handle_placement_submission` writes to `PendingPlacements` only. No S2C message is sent on acceptance. |
| `design/gdd/network-protocol.md` | Network Protocol | NP-14: Duplicate submission discarded silently. | `is_final` flag on `PlayerSubmission` gates all subsequent submissions for the same player in the same phase. |
| `design/gdd/network-protocol.md` | Network Protocol | NP-28: Both players' placements in one `S2CPlacementReveal` payload. | `close_placement_phase` collects all `PendingPlacements` entries into a single `S2CPlacementReveal { placements: Vec<PlacedCard> }` before spawning any entity. |
| `design/gdd/network-protocol.md` | Network Protocol | `C2SSubmitPlacement` server validation: all-or-nothing batch discard on any validation failure. | `validate_placement_submission` checks the full batch atomically. Partial acceptance is not implemented. |
| `design/gdd/network-protocol.md` | Network Protocol | TR-NP-10: Reconnect during PLACEMENT with submitted = true. | `PlayerSubmission.is_final` persists on the server for the PLACEMENT duration. Snapshot reads from this flag to set `PlayerSnapshot.submitted`. |

## Related

- [ADR-001: Hidden Objective Identity via Targeted Unicast, Not Component Replication](adr-001-objective-identity-unicast.md) — Established that Lightyear 0.26 has no per-component, per-client replication scope. The same constraint motivates this ADR's buffer-before-spawn invariant.
- ADR-002 (server authority model) — Must be Accepted. Establishes that clients are read-only views; no client-side placement state is permissible.
- ADR-003 (workspace / crate layout) — Must be Accepted. Determines which crate owns `server/src/feature/board/placement.rs`.
- ADR-009 (RSM phase event contracts) — Must be Accepted. `PlacementPhaseEntered` message is the trigger that opens the buffer; its exact type and channel must be defined before the buffer lifecycle can be implemented.
- `design/gdd/board-lane-system.md` — Primary GDD source. Rule 6 and Formula F2 are directly implemented here.
- `design/gdd/network-protocol.md` — Defines `C2SSubmitPlacement`, `S2CPlacementReveal`, and all acceptance criteria referenced in Validation Criteria above.
- `design/gdd/round-state-machine.md` — Rule 9 defines PLACEMENT timer semantics (`placement_timer_seconds`, all-submitted early exit) consumed by `close_placement_phase`.
- `design/gdd/game-config.md` — `placement_timer_seconds` is read from `GameConfig` resource, not hardcoded.
