# ADR-017: Combat Resolution Execution Architecture

## Status

Accepted

## Date

2026-04-30

## Engine Compatibility

| Field | Value |
|-------|-------|
| **Engine** | Bevy 0.18 |
| **Domain** | Core / Game Logic |
| **Knowledge Risk** | HIGH — Bevy 0.18 is post-LLM-cutoff; exclusive system API and Message patterns have changed since 0.14 |
| **References Consulted** | `docs/engine-reference/bevy/VERSION.md`, `docs/architecture/adr-010-rsm-event-bus.md`, `docs/architecture/adr-009-rsm-phase-state.md`, `docs/architecture/adr-002-client-server-authority.md` |
| **Post-Cutoff APIs Used** | Bevy 0.18 exclusive system (`fn f(world: &mut World)`), Bevy 0.16+ `#[derive(Message)]` / `MessageWriter<T>` / `MessageReader<T>` for Bevy-internal buffered messages (Event→Message split), Lightyear 0.26 `MessageSender<T>` / `MessageReceiver<T>` for network protocol messages, Bevy 0.18 `Commands::entity(e).despawn()` (replaces `despawn_recursive`) |
| **Verification Required** | (1) **RESOLVED**: Exclusive system registration via `fn f(world: &mut World)` is stable Bevy API since before 0.14; auto-detection of `&mut World` as exclusive system has not changed through 0.18. `add_systems(Update, resolve_combat)` is correct. The system holds the World lock for its full duration — no concurrent systems run in the same schedule frame. (2) **RESOLVED**: `World::resource_mut::<T>()` and `World::resource::<T>()` are stable World accessor APIs since Bevy 0.12; no breaking changes documented through 0.18 in `docs/engine-reference/bevy/breaking-changes.md`. (3) **RESOLVED**: API disambiguation is documented in the Engine Compatibility table and the Risks section. `MessageWriter<BeginResolution>` / `MessageWriter<ResolutionComplete>` are Bevy-internal buffered messages (registered via `app.add_message::<T>()`); `S2CResolutionEvent` uses Lightyear's `MessageSender<T>` (registered via Lightyear's protocol plugin). Code review checklist must enforce this boundary. |

## ADR Dependencies

| Field | Value |
|-------|-------|
| **Depends On** | ADR-002 (Accepted — server authority model; resolution is server-only), ADR-009 (Accepted — RSM phase state; `BeginResolution` trigger and `ResolutionComplete` exit are RSM-owned), ADR-010 (Accepted — RSM event bus; both messages travel on the buffered Message bus) |
| **Enables** | Board/Lane System epic (M1) — the `BoardState` API this ADR specifies as the interface contract lets M1 finalize its spatial query surface; Combat Resolution epic (M2) — stories cannot be written until execution model is Accepted |
| **Blocks** | Combat Resolution epic (M2) — all stories in this epic depend on the exclusive system boundary and the `ResolutionEvent` enum defined here |
| **Ordering Note** | ADR-009 and ADR-010 must be Accepted before any Combat Resolution story is opened. This ADR resolves OQ1 from `combat-resolution.md` — the ADR must be Accepted before the Combat Resolution epic is started. |

## Context

### Problem Statement

Combat Resolution is the most algorithmically complex system in the game. Before any implementation story can be opened, three architectural decisions need explicit documentation:

1. **Execution model** — Should the 6-step deterministic algorithm run inside a single Bevy frame (exclusive system) or be spread across multiple frames via a state machine?
2. **Collision detection boundary (OQ1)** — Sub-step 5 defines step-by-step collision detection for enemy unit interactions. The Board/Lane GDD (BL-27, BL-27b) defines a "skip intermediate cells" rule for movement. These two rules govern different things and must be reconciled in one place.
3. **Event delivery model** — Should clients receive resolution events as a single batch (`S2CResolutionEvent`) or as a stream of per-sub-step messages?

OQ1 in `combat-resolution.md` explicitly flags that this ADR must exist before the Combat Resolution epic begins.

### Constraints

- **Server-only** (ADR-002): no client observes intermediate sub-step ECS state; authority belongs entirely to the server crate.
- **Frame budget**: ≤ 15 ms for the full resolution batch in one Bevy frame (ADR-002 performance table; a worst-case 5-lane contested round).
- **60-second safety timeout** (RSM Rule 14): the RSM monitors wall-clock time from `BeginResolution`; if `ResolutionComplete` has not arrived in 60 seconds, the RSM fires a Draw. The execution must be synchronous — an async or multi-frame model would complicate the timeout contract.
- **Pre-computed RNG**: server-side RNG for Ecaflip dice (`ServerRng` per ADR-002) must be consumed before sub-step 1 executes. No RNG is called mid-algorithm.
- **Bevy 0.18 exclusive systems**: an exclusive system receives `&mut World` and cannot simultaneously use standard Bevy system params (Query, Res, etc.) — all ECS access is through raw World API.

### Requirements

- Must execute all 6 sub-steps in a single invocation triggered by `BeginResolution`.
- Must accumulate a structured event log sufficient for complete client animation replay (CR-32).
- Must produce a `ResolutionEvent` enum that closes OQ5 (adds `CombatDamage` and `KeywordTriggered` variants to the schema in `network-protocol.md`).
- Must formally resolve the boundary between the Board/Lane "skip intermediate cells" rule and the step-by-step collision detection in sub-step 5.
- The combat modifier stack must be a pure function (no World access) to satisfy testability requirements for all BLOCKING CRs.

## Decision

Three decisions are made together and are mutually reinforcing.

### Decision 1 — Exclusive System Execution

The 6-step algorithm runs inside a single Bevy exclusive system:

```rust
pub fn resolve_combat(world: &mut World) { ... }
```

Registered in the server `App` as a normal system — Bevy 0.18 auto-detects `&mut World` as exclusive. The system:

1. Reads `MessageReader<BeginResolution>` — exits immediately if no message is present (idling outside RESOLUTION phase).
2. Takes a stat snapshot (unit ATK, HP, AR, MP, keywords; LEADER bonuses) at resolution entry. This snapshot is immutable for the duration of the run.
3. Executes sub-steps 1–6 as sequential function calls, accumulating a `ResolutionLog`.
4. Applies all ECS mutations at their natural points (units move, die, gold updates via `ResMut<EconomyState>`).
5. Broadcasts `S2CResolutionEvent { round, events: log.into_vec() }` via Lightyear.
6. Writes `MessageWriter<ResolutionComplete>` to notify the RSM.

All ECS mutations and event logging happen **inline** within one frame. No intermediate sub-step state is observable by other Bevy systems.

### Decision 2 — Movement Collision Boundary (Resolves OQ1)

Two movement rules coexist in sub-step 5. They govern **different aspects** of the same movement phase and do not conflict:

**Rule A — Destination Rule (Board/Lane GDD — unchanged)**

The `unit_movement` formula F1 computes each unit's **intended destination** once at sub-step 5 entry:

```
destination = clamp(current_cell + direction × MP, 1, 8)
```

This destination is the only cell that triggers Traps (BL-27) and Prism collection (board-lane-system.md Rule 11). Intermediate cells are skipped for all non-collision purposes. Friendly units never block movement. This rule is unchanged from the Board/Lane GDD.

**Rule B — Enemy Collision Detection (Combat Resolution — additional layer)**

A per-tick advance loop inside `fn execute_movement()` determines the **actual final position** after enemy obstruction. This is a logical simulation (not Bevy frames):

```rust
fn execute_movement(board: &mut BoardState, log: &mut ResolutionLog) {
    // 1. Compute each non-STUNned unit's destination via F1 (one-time).
    // 2. Run per-tick loop until all units reach destination or halt:
    //    - Each tick: advance all units by 1 cell toward destination.
    //    - WALL halt: unit stops when its next step would land on an enemy WALL cell.
    //    - Path-cross halt: if A and B would swap cells in the same tick,
    //      both halt at their previous-tick cells (adjacent, fight in sub-step 6).
    //    - Same-cell: two enemies landing on the same cell both remain (fight in sub-step 6).
    //    - Friendly units, Traps, Prisms: not checked in this loop (destination rule applies).
}
```

The precise boundary: **the destination rule applies to everything; the collision loop applies only to enemy unit obstruction within that same sub-step**. A unit can skip over enemy units in principle (destination rule) but cannot skip through them in practice (collision loop catches it tick by tick). These are not contradictory — the collision loop is a filter on top of the destination rule.

This matches CRs CR-8 (WALL halt), CR-9 (path-crossing halt), and the design note in combat-resolution.md sub-step 5.

### Decision 3 — Batch Event Delivery

`S2CResolutionEvent` is a single reliable-broadcast Lightyear message sent **after all 6 sub-steps complete**. Clients receive the complete log before any animation frame begins, then replay it locally at animation tempo (governed by card-animations.md).

The canonical `ResolutionEvent` enum (defined in `protocol/` crate — this closes OQ5):

```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ResolutionEvent {
    SubStepBegin { sub_step: u8 },
    UnitPlaced    { unit_id: UnitId, player: PlayerId, lane: LaneId, cell: CellIndex },
    UnitMoved     { unit_id: UnitId, from_cell: CellIndex, to_cell: CellIndex, sub_step: u8 },
    UnitChangedLane { unit_id: UnitId, from_lane: LaneId, to_lane: LaneId, sub_step: u8 },
    CombatDamage  { attacker_id: UnitId, defender_id: UnitId, damage_amount: u8,
                    was_blocked_by_shield: bool, sub_step: u8 },
    ObjectiveDamage { attacker_id: UnitId, lane: LaneId, damage_amount: u8, objective_hp_after: u8 },
    UnitRemoved   { unit_id: UnitId, lane: LaneId, cell: CellIndex },
    KeywordTriggered { unit_id: UnitId, keyword: KeywordKind, sub_step: u8 },
    GoldAwarded   { player: PlayerId, amount: u8, reason: GoldReason },
    ObjectiveDestroyed { lane: LaneId, owner: PlayerId, is_fake: bool },
    GameOver      { loser: Option<PlayerId>, reason: GameOverReason },
}
```

`network-protocol.md` must be updated to reference this enum as the canonical `ResolutionEvent` schema, closing OQ5.

### Architecture Diagram

```
                  Combat Resolution — Execution Flow

  RSM (server/)                resolve_combat (server/ exclusive system)
  ───────────────────────────────────────────────────────────────────────
  BeginResolution ──────────►  read trigger
                               snapshot unit stats + LEADER bonuses
                               consume ServerRng (Ecaflip pre-computation)

                               sub-step 1: apply_placements()
                               │  ECS: spawn units, fire APPEARANCE triggers
                               │  Log: UnitPlaced, KeywordTriggered (APPEARANCE)

                               sub-step 2: execute_charge_x()
                               │  Destination rule + collision loop (enemy only)
                               │  Log: UnitMoved, KeywordTriggered (CHARGE X)

                               sub-step 3: execute_first_strike()
                               │  apply_combat_modifier_stack() — pure fn
                               │  ECS: mutate HP (not removed yet)
                               │  Log: CombatDamage, KeywordTriggered (FIRST STRIKE, COUNTERATTACK)

                               sub-step 4: remove_dead()
                               │  ECS: despawn units, award kill gold
                               │  Log: UnitRemoved, GoldAwarded (kill), KeywordTriggered (DEATH)

                               sub-step 5: execute_movement()
                               │  Destination rule (F1) + enemy collision loop (tick-by-tick)
                               │  ECS: mutate positions
                               │  Log: UnitMoved, UnitChangedLane (cross-lane triggers)

                               sub-step 6: execute_combat()
                               │  apply_combat_modifier_stack() — pure fn (bilateral 2-pass)
                               │  ECS: mutate HP, despawn dead, award kill gold, check objective
                               │  Log: CombatDamage, UnitRemoved, GoldAwarded, ObjectiveDamage,
                               │       ObjectiveDestroyed, GameOver, KeywordTriggered

                               broadcast S2CResolutionEvent (full log) via Lightyear
  ◄──────────── ResolutionComplete  write ResolutionComplete (MessageWriter)
```

### Key Interfaces

```rust
// ── server/ crate ──────────────────────────────────────────────────────────

/// Exclusive Bevy system. Registered with add_systems(Update, resolve_combat).
/// Bevy 0.18 auto-detects &mut World as exclusive.
pub fn resolve_combat(world: &mut World);

/// Pure function — no ECS access. Takes snapshots, returns result.
/// Called for every individual attack (one attacker / one defender pair).
pub fn apply_combat_modifier_stack(
    attacker: &UnitSnapshot,
    defender: &UnitSnapshot,
) -> CombatResult;

pub struct CombatResult {
    pub net_damage: u8,
    pub ar_attacker_combat: u8,  // type-advantage AR bonus for two-pass bilateral algorithm
}

/// Unit stats frozen at RESOLUTION entry. Immutable during algorithm run.
pub struct UnitSnapshot {
    pub unit_id: UnitId,
    pub player: PlayerId,
    pub lane: LaneId,
    pub cell: CellIndex,
    pub atk: u8,
    pub hp: u8,   // live HP (mutated inline; snapshot is per-field, not a frozen copy)
    pub ar: u8,
    pub mp: u8,
    pub unit_type: UnitType,
    pub keywords: KeywordSet,
    pub leader_atk_bonus: u8,  // snapshotted at resolution entry; 0 if no LEADER
}

/// Accumulates all events during algorithm execution.
#[derive(Default)]
pub struct ResolutionLog {
    events: Vec<ResolutionEvent>,
}

impl ResolutionLog {
    pub fn push(&mut self, event: ResolutionEvent);
    pub fn into_vec(self) -> Vec<ResolutionEvent>;
}

// ── protocol/ crate ────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug, Message)]
pub struct S2CResolutionEvent {
    pub round: u32,
    pub events: Vec<ResolutionEvent>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ResolutionEvent { /* as defined in Decision 3 */ }

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum GoldReason { Kill, ObjectiveDestroyed }

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum GameOverReason { ObjectivesDestroyed, ResolutionTimeout, Draw }
```

## Alternatives Considered

### Alternative 1: State Machine Across Frames

- **Description**: Each sub-step is a separate Bevy system scheduled via a `ResolutionSubStep` state enum (`ApplyingPlacements` → `ExecutingChargeX` → … → `ExecutingCombat`). Board ECS state is live and observable between sub-steps.
- **Pros**: More idiomatic ECS design. Each sub-step can be inspected by debug tools mid-resolution.
- **Cons**: The RSM's 60-second safety timeout (RSM Rule 14) requires that `ResolutionComplete` be emitted after all sub-steps finish. A state machine introduces frame boundaries between sub-steps, requiring the timeout monitor to traverse those states. Cross-frame execution also means other systems could observe intermediate board state (e.g., a dead unit's components still present between sub-step 3 damage and sub-step 4 removal), creating subtle consistency bugs. The GDD describes RESOLUTION_EXECUTING as a single internal state with sub-steps as "internal" sequencing — not as externally observable RSM states.
- **Rejection Reason**: The synchronous exclusive system is simpler, matches the GDD's atomicity framing, and eliminates the intermediate-state observation problem. The per-sub-step state labels in the GDD's state table are logging identifiers for the 60-second timeout, not ECS schedule states.

### Alternative 2: Skip-Intermediate-Cells for Enemy Collision Too

- **Description**: Apply the Board/Lane "skip intermediate cells" rule uniformly to all movement, including enemy collision. Units teleport to their F1 destination. Enemy-vs-enemy collision is checked only at final positions.
- **Pros**: One unified movement rule; simpler implementation.
- **Cons**: Allows units to pass through each other and through WALLs if both units' destinations are past each other. Path-crossing (CR-9) and WALL-halting (CR-8) cannot be correctly detected at destination-only resolution — by the time positions are checked, both units have already swapped. The GDD explicitly designates step-by-step as a deliberate design choice ("the board tells the truth"; WALL must stop the advancing unit).
- **Rejection Reason**: Directly breaks CR-8 and CR-9. The collision model is a stated design choice, not an implementation detail.

### Alternative 3: Streaming Per-Sub-Step Events

- **Description**: After each sub-step completes server-side, broadcast a partial `S2CResolutionSubStep` message. Clients apply the partial board mutations and animate incrementally.
- **Pros**: Clients can begin animating sub-step 1 while sub-step 2 is still computing; lower perceived latency.
- **Cons**: The server completes all 6 sub-steps in a single frame (≤ 15 ms). The client can only animate one frame per 16.67 ms anyway. Streaming sub-step messages would arrive in the same Lightyear batch and be processed in the same client frame — there is no perceived latency benefit. Additionally, clients would need to track partial board state mid-resolution, complicating the `apply_s2c_to_client_state` function (ADR-002) and the reconnect snapshot path. Card Animations GDD already specifies that the client replays the log at its own animation tempo.
- **Rejection Reason**: No latency benefit at this timescale; adds client complexity for zero player-visible gain.

## Consequences

### Positive

- **Testable modifier stack**: `apply_combat_modifier_stack` is a pure function. All 45 BLOCKING CRs that depend on damage calculation can be unit-tested without any Bevy context.
- **Atomic resolution**: other Bevy systems never observe a partially-resolved board. No sub-step interleaving bugs.
- **Simple timeout contract**: the RSM monitors one message (`ResolutionComplete`). If it never arrives within 60 seconds, the RSM fires Draw — no sub-step heartbeats needed.
- **Animation decoupled from execution**: the client replays the `ResolutionLog` at animation tempo. Server execution speed is independent of animation speed.
- **OQ1 formally closed**: the movement rule boundary is specified in this ADR. The board-lane-system GDD deviation note now has an authoritative reference.
- **OQ5 substantially closed**: `CombatDamage` and `KeywordTriggered` variants are defined. `network-protocol.md` must be updated to reference this enum.

### Negative

- **Exclusive system limits composability**: `resolve_combat` cannot use Query, Res, etc. — all ECS access is via raw `World` API. This is more verbose. Mitigation: extract pure functions (modifier stack, collision loop) that do not need `World` at all.
- **One 15ms spike per resolution**: the frame that runs `resolve_combat` takes up to 15 ms. This is acceptable for a turn-based game (clients are in non-interactive RESOLUTION observation mode) but must be profiled.
- **`ResolutionLog` allocation**: a contested 5-lane round may produce 50–100 events. At ~100 bytes per event, this is ~10 KB heap per resolution. Acceptable for a two-player session; revisit if session count per server process scales.

### Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Exclusive system takes > 15 ms on Railway hardware during a 5-lane contested round with many keywords. | Low | Medium | Profile in the first vertical slice with `tracing::instrument` on `resolve_combat`. If needed, pre-compute the collision loop results before entering the exclusive system. |
| `ResolutionEvent` enum grows large as keywords are added (M3+), bloating `S2CResolutionEvent`. | Medium | Low | Define a max event count guard (e.g., 256 events per resolution). Keyword implementations must stay within this budget. |
| `network-protocol.md` OQ5 update is forgotten, leaving the schema inconsistent. | Low | Medium | `/architecture-review` will flag the mismatch. Include as a blocking task in the Combat Resolution epic. |
| Bevy 0.18 exclusive system registration API differs from training-data assumptions. | Medium | Medium | Verified via `docs/engine-reference/bevy/VERSION.md`; add as a first-story verification task in the epic. |
| `MessageWriter`/`MessageSender` confusion: developer uses Lightyear `MessageSender` for a Bevy-internal message or vice versa, causing a compile error or silent message loss. | Medium | Medium | Engine Compatibility table disambiguates the two APIs. Code review checklist: `BeginResolution` and `ResolutionComplete` use `MessageWriter`; `S2CResolutionEvent` uses Lightyear `MessageSender`. |
| `UnitSnapshot::hp` mutated with raw `u8 -= u8` subtraction, causing panic in debug or silent wrap in release when `damage > hp`. | Low | Medium | All HP mutations must use `hp = hp.saturating_sub(damage)` or the `i32`-clamp path. The `apply_combat_modifier_stack` function's note (compute in `i32`) applies to ATK; the same discipline applies to every HP mutation site inside `resolve_combat`. |

## GDD Requirements Addressed

| GDD System | Requirement | How This ADR Addresses It |
|------------|-------------|--------------------------|
| `combat-resolution.md` | OQ1 — "An ADR will document this deviation" (step-by-step movement vs. skip-intermediate-cells) | Decision 2 defines the exact boundary: destination rule (F1) governs Trap/Prism; collision loop governs enemy obstruction. |
| `combat-resolution.md` | OQ5 — `S2CResolutionEvent` missing `CombatDamage` and `KeywordTriggered` variants | Decision 3 defines the canonical `ResolutionEvent` enum with both variants. `network-protocol.md` must be updated. |
| `combat-resolution.md` | CR-32 — "S2CResolutionEvent containing a sequenced log of all sub-step events" | Decision 3 + the `ResolutionEvent` enum satisfy the full CR-32 content requirement. |
| `combat-resolution.md` | States/Transitions table — RESOLUTION_EXECUTING is a single internal state; sub-steps are internal sequencing | Decision 1 (exclusive system) matches this framing: sub-steps are function calls, not RSM states. |
| `combat-resolution.md` | Formula 1 (Combat Damage) + CR-12 through CR-15, CR-42, CR-43 — modifier stack correctness | Decision 1 key interface: `apply_combat_modifier_stack` is a pure function satisfying all modifier-stack CRs. |
| `board-lane-system.md` | BL-27, BL-27b — "skip intermediate cells" rule for movement and CHARGE X | Decision 2 preserves this rule for Trap/Prism triggering while introducing the collision loop as an additional layer for enemy obstruction only. |
| `network-protocol.md` | D.2 — `S2CResolutionEvent` schema definition | This ADR extends the schema with the canonical `ResolutionEvent` enum. `network-protocol.md` D.2 must reference this ADR as the source of truth for the enum. |
| `round-state-machine.md` | RSM Rule 14 — 60-second safety timeout fires Draw if RESOLUTION hangs | Decision 1 ensures `ResolutionComplete` is written synchronously. If `resolve_combat` crashes or hangs, the RSM timeout fires Draw without needing sub-step granularity. |

## Performance Implications

| Metric | Expected | Budget | Notes |
|--------|----------|--------|-------|
| CPU (server frame — RESOLUTION) | ≤ 15 ms for worst-case 5-lane round | 16.67 ms (60 Hz server tick) | Single-frame spike acceptable; clients are non-interactive during RESOLUTION |
| CPU (server frame — idle) | < 1 ms | 5 ms steady state (ADR-002) | `resolve_combat` exits immediately when no `BeginResolution` message present |
| Memory (`ResolutionLog`) | ~10 KB per resolution (100 events × 100 bytes) | < 1 MB per session | Reset each resolution; no accumulation across rounds |
| Network (`S2CResolutionEvent`) | < 8 KB per resolution (worst case) | 1 KB / round steady state — this message is a one-off burst, not steady state | Acceptable spike on reliable channel; does not count against per-round budget |

## Migration Plan

No existing implementation to migrate from. Adoption sequence:

1. Define `ResolutionEvent` enum in `protocol/` (closes OQ5 in `network-protocol.md`).
2. Scaffold `server/src/combat/mod.rs` with the `resolve_combat` exclusive system stub — trigger on `BeginResolution`, write `ResolutionComplete`, broadcast empty `S2CResolutionEvent`. Integration test: RSM transitions correctly through RESOLUTION_EXECUTING → next phase via this stub.
3. Implement `apply_combat_modifier_stack` as a pure function with full unit test coverage (CR-12 through CR-15, CR-42, CR-43 can be tested immediately).
4. Implement sub-steps in order: 1 (placements), 4 (dead removal), 6 (standard combat), 3 (first strike), 5 (movement + collision loop), 2 (charge X). Sub-step 4 and 6 together cover the majority of BLOCKING CRs.
5. Update `network-protocol.md` D.2 to reference this ADR's `ResolutionEvent` enum as canonical (OQ5 close).

## Validation Criteria

- [ ] `fn resolve_combat(world: &mut World)` compiles and is registered without error in Bevy 0.18. Verified in the first Combat Resolution epic story.
- [ ] `fn apply_combat_modifier_stack` has unit tests covering CR-12, CR-13, CR-14, CR-15, CR-42, CR-43 — all pass without Bevy context.
- [ ] An integration test drives `resolve_combat` with a `BeginResolution` message, asserts `ResolutionComplete` is written, and asserts `S2CResolutionEvent` is emitted. No Lightyear runtime required (use a headless World with stubbed message transport).
- [ ] `S2CResolutionEvent.events` for a one-unit, one-lane round contains at minimum: `SubStepBegin(1)`, `UnitPlaced`, `SubStepBegin(5)`, `UnitMoved`, `SubStepBegin(6)`, `CombatDamage` or `ObjectiveDamage`. Verified by asserting log contents in integration test.
- [ ] CR-8 (WALL halt) and CR-9 (path-crossing halt) are covered by unit tests that exercise `execute_movement` directly with a synthetic `BoardState`.
- [ ] `network-protocol.md` D.2 references `ResolutionEvent` from this ADR. Verified by `/architecture-review` cross-reference pass.

## Related Decisions

- [ADR-002 — Client-Server Authority Model](./adr-002-client-server-authority.md) — server-only execution; `resolve_combat` lives in `server/` crate
- [ADR-009 — RSM Phase State](./adr-009-rsm-phase-state.md) — `BeginResolution` trigger and `ResolutionComplete` exit are RSM-owned phase boundary messages
- [ADR-010 — RSM Phase Event Bus](./adr-010-rsm-event-bus.md) — both messages travel on the buffered Message bus; this ADR adds Combat Resolution to the M2 subscriber table
- [ADR-005 — Server-Side RNG](./adr-005-server-side-rng.md) — `ServerRng` consumed before sub-step 1; Ecaflip dice pre-computed at resolution entry
- `design/gdd/combat-resolution.md` — primary GDD; this ADR resolves OQ1 and OQ5
- `design/gdd/board-lane-system.md` — `unit_movement` formula F1 and the "skip intermediate cells" rule this ADR extends
- `design/gdd/network-protocol.md` — `S2CResolutionEvent` schema; D.2 must be updated to reference the `ResolutionEvent` enum defined here
