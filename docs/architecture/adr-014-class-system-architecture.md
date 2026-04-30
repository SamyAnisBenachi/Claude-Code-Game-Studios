# ADR-014: Class System Architecture — PlayerSessionState, SourceClass Component, and Direct Effect Dispatch

## Status

Accepted

## Date

2026-04-30

## Engine Compatibility

| Field | Value |
|-------|-------|
| **Engine** | Bevy 0.18 + Lightyear 0.26 |
| **Domain** | Core / Session State |
| **Knowledge Risk** | MEDIUM — Bevy 0.17 replaced `EventWriter`/`EventReader` with `MessageWriter`/`MessageReader`; Bevy 0.16 changed `Query::single()` to return `Result`; `ClassId` already defined in `shared/src/card.rs` (ADR-006) and must NOT be redefined |
| **References Consulted** | `docs/engine-reference/bevy/VERSION.md`, `docs/engine-reference/bevy/breaking-changes.md`, `docs/engine-reference/bevy/current-best-practices.md`, `docs/architecture/adr-002-client-server-authority.md`, `docs/architecture/adr-006-card-data-schema.md`, `docs/architecture/adr-010-rsm-event-bus.md` |
| **Post-Cutoff APIs Used** | `#[derive(Message, Serialize, Deserialize, Clone)]` for `C2SClassChoice` — this is **Lightyear's** `Message` trait from `lightyear::prelude`, NOT Bevy's `Message` trait from `bevy::prelude`; both exist in this project simultaneously and must not be confused. Lightyear 0.26 `MessageReceiver<C2SClassChoice>` with `.receive_messages()` for server-side handler (exact method name verified against `docs/engine-reference/bevy/current-best-practices.md` 2026-04-28). `#[derive(Resource)]` for `PlayerSessions` (stable since Bevy 0.12). |
| **Verification Required** | (1) **RESOLVED**: `MessageReceiver<C2SClassChoice>.receive_messages()` confirmed — same pattern as ADR-013 item 1 and `docs/engine-reference/bevy/current-best-practices.md` (Lightyear 0.26). Pin exact Lightyear patch version in `Cargo.toml`; wrap receiver call in `server/src/lobby/handler.rs` — one file to update if API shifts. (2) **RESOLVED**: `SourceClass` component does NOT require `#[derive(Reflect)]`. The ADR Decision §3 explicitly opts out: "Reflect intentionally NOT derived: server-only component; no scene serialisation or Bevy inspector usage in the headless server build." Snapshot serialisation uses a hand-written builder (`world.get::<SourceClass>(entity).map(|sc| sc.0)`) — no Reflect required. (3) **RESOLVED (implementation-time)**: `PlayerSessions` inserted immediately before `SessionReady` per ADR-012 lifecycle contract. Verified by lifecycle integration test (ADR-012 pattern) in Validation Criteria. |

## ADR Dependencies

| Field | Value |
|-------|-------|
| **Depends On** | ADR-002 (Accepted — authority model; `PlayerSessions` is a server-only Resource, never replicated or visible to client crate); ADR-003 (Accepted — workspace; `PlayerSessions` lives in `server/src/core/session/state.rs`); ADR-005 (Accepted — server RNG; Ecaflip dice effects consume from the RESOLUTION chain per §4 consumption order — no new `RngEvent` variants are introduced by this ADR, but class-effect stories will add them); ADR-006 (Accepted — `ClassId` enum lives in `shared/src/card.rs`; this ADR uses it and does not redefine it); ADR-009 (Accepted — RSM phases; `C2SClassChoice` is LOBBY-phase gated; `LobbyComplete` Message triggers class-locking); ADR-010 (Accepted — event bus; `LobbyComplete` follows the `#[derive(Message)]` bus pattern); ADR-012 (Accepted — session lifecycle; `PlayerSessions` is inserted at `SessionReady` and removed at `GameOverEmitted`) |
| **Enables** | Economy System ADR (gold/mana/reserve fields added to `PlayerSessionData`; Xelor reserve formulas use `ResMut<PlayerSessions>`); Objective System ADR (`sang_meprise_active` lives in Objective resource, not here, but Sang Méprise class effect reads `PlayerSessions` to identify the caster); Combat Resolution ADR (Rollback movement, Seed walk-over, Sacrier Fulgurance — all read `player.class` via `Res<PlayerSessions>`); Card Acquisition ADR (`class_of()` provides the `ClassId` parameter to `PlayerPool::draw_class_card()` from ADR-006); all Class System epic stories (LOBBY-CS, CLASS-EFFECTS-*, TOKEN-SPAWN-*) |
| **Blocks** | No story implementing a class-specific card effect may merge until this ADR is Accepted. LOBBY class selection story blocked until `C2SClassChoice` handler and `PlayerSessions` are defined. Any story that reads `player.class` is blocked. |
| **Ordering Note** | Must be Accepted before any class effect story (Xelor reserve, Sacrier reveals, Sadida seeds, Ecaflip dice, Miranda) is opened for implementation. ADR-006 provides the `ClassId` type this ADR depends on and must remain Accepted. |

## Context

### Problem Statement

The Class System GDD specifies six playable classes with session-scoped identity: class is selected during LOBBY, locked at LOBBY→DRAFT_INITIAL, and never changes for the remainder of the session. Each player's class determines:

1. Which class card library is sampled during personal shop slot generation — `PlayerPool::draw_class_card(class, ...)` from ADR-006 requires the `ClassId` at draw time.
2. Which class-specific token entities a player can spawn — Mummy, Chacha Noir, Seeds, Madolls, La Gonflable, La Sacrifiée (7 token types, each needing a `source_class` tag).
3. Which class-specific keyword extensions and spell effects apply — 11 cross-system formulas (CS-1 through CS-11), 27 acceptance criteria, effects dispatching to Economy, Objective, and Combat systems.

No existing ADR specifies:
- Where `player.class: ClassId` lives in the server ECS
- How class-locking is enforced architecturally at LOBBY→DRAFT_INITIAL
- How token `source_class` is stored and queried (required for Miranda-stolen token integrity and LEADER bonus checks)
- What communication pattern class effects use when mutating Economy, Objective, and Combat state within a RESOLUTION tick

Without these decisions, every class-effect story would make independent architectural choices, leading to inconsistent patterns and potential single-writer violations.

### Constraints

- **`ClassId` source**: `ClassId` is already defined in `shared/src/card.rs` (ADR-006) with variants `{ Iop, Cra, Sacrier, Xelor, Ecaflip, Sadida, Neutral }`. It MUST NOT be redefined in this ADR or any downstream system.
- **Server authority** (ADR-002): `PlayerSessions` is a server-only `Resource`. It lives in `server/` and must NEVER appear in `protocol/` or `client/`. The `ClassId` enum itself lives in `shared/` because both client (for display) and server (for logic) reference it.
- **LOBBY phase gate** (ADR-009): `C2SClassChoice` is silently discarded outside LOBBY phase. The class-locking invariant (`class_locked = true` for all players) must be enforced before LOBBY→DRAFT_INITIAL fires.
- **Bevy 0.17+ Message API**: Cross-system signals use `#[derive(Message)]` + `MessageWriter`/`MessageReader` (Bevy's intra-ECS pattern). `EventWriter`/`EventReader` do not exist in Bevy 0.17+.
- **RESOLUTION sub-step ordering**: Class effects must resolve within the same server tick as the RESOLUTION phase, in the prescribed sub-step order (Xelorium at sub-step 1, Rollback at sub-step 2, etc.). A buffered-Message approach with 1-frame delivery latency would violate this ordering — class effect functions must be called synchronously from within the RESOLUTION system body, not via registered Message handlers in a subsequent frame.

### Requirements

- **R1** — Class is a session-scoped property: `player.class` must be readable in every phase ≥ DRAFT_INITIAL without additional computation.
- **R2** — Class is locked immutably: once `class_locked = true`, no path may change `player.class`. Violations must be prevented structurally (server-side validation), not by convention.
- **R3** — Token `source_class` is immutable after spawn: the `SourceClass(ClassId)` component is set at token entity creation and never mutated.
- **R4** — Class effect sub-step ordering: cross-system mutations (Economy, Objective, Combat) are applied within the same RESOLUTION system run, not in a subsequent frame.
- **R5** — Snapshot completeness: `S2CGameSnapshot.PlayerSnapshot` must include `class_id: ClassId` (CS-AC-03, NP-1). `UnitBoardState` must include `source_class: Option<ClassId>` (NP-2).
- **R6** — Single `ClassId` source: both client and server use the identical `ClassId` enum from `shared/src/card.rs` with no duplication.

## Decision

### 1. PlayerSessions Resource

`PlayerSessions` is a server-only `Resource` in `server/src/core/session/state.rs`. It is the single authoritative store for per-player session-identity state. `class: ClassId` and `class_locked: bool` are the first fields; future Economy, Card Acquisition, and Combat ADRs extend `PlayerSessionData` with `gold`, `current_mana`, `reserve`, and `hand` fields.

```rust
// server/src/core/session/state.rs

use bevy::prelude::*;
use shared::card::ClassId;
use shared::session::PlayerId;
use std::collections::HashMap;

/// Per-player session-identity state. Server-only — never replicated.
///
/// Inserted by the session lifecycle system immediately before SessionReady.
/// Removed by the session lifecycle system on GameOverEmitted (ADR-012 contract).
///
/// Future ADRs add fields to PlayerSessionData:
///   Economy ADR  → gold: u32, current_mana: u32, reserve: u32
///   Card Acq ADR → hand: Vec<CardId>
/// Each subsystem owns only its declared fields; other fields are read-only.
#[derive(Resource, Default)]
pub struct PlayerSessions {
    pub players: HashMap<PlayerId, PlayerSessionData>,
}

#[derive(Default, Clone, Debug)]
pub struct PlayerSessionData {
    /// Class selected at LOBBY. ClassId::Neutral is the "not yet chosen" sentinel.
    /// Changed by C2SClassChoice handler while class_locked == false.
    /// ClassId::Neutral is architecturally unreachable in any phase >= DRAFT_INITIAL.
    pub class: ClassId,

    /// True after the LOBBY->DRAFT_INITIAL transition. Never reset to false.
    /// C2SClassChoice messages received while class_locked == true are silently discarded.
    pub class_locked: bool,
}

impl PlayerSessions {
    /// Returns the locked class for a player.
    ///
    /// Panics (with message) only if called before the player is registered in the session.
    /// Systems at phase >= DRAFT_INITIAL are guaranteed the player is in the map
    /// (invariant from GSS session lifecycle).
    pub fn class_of(&self, player_id: PlayerId) -> ClassId {
        self.players
            .get(&player_id)
            .expect("class_of: player not registered in PlayerSessions")
            .class
    }

    /// True if the player's class is locked (phase >= DRAFT_INITIAL invariant).
    pub fn is_locked(&self, player_id: PlayerId) -> bool {
        self.players.get(&player_id).map(|p| p.class_locked).unwrap_or(false)
    }

    /// Gate check for LOBBY->DRAFT_INITIAL.
    /// Returns true only when all registered players have chosen a non-Neutral class.
    pub fn all_classes_chosen(&self) -> bool {
        self.players.values().all(|p| p.class != ClassId::Neutral)
    }

    /// Lock all classes atomically. Called by the RSM LOBBY->DRAFT_INITIAL gate
    /// system immediately before emitting LobbyComplete. Never called again.
    pub fn lock_all_classes(&mut self) {
        for p in self.players.values_mut() {
            debug_assert!(
                p.class != ClassId::Neutral,
                "lock_all_classes: player has Neutral class — gate should have blocked this"
            );
            p.class_locked = true;
        }
    }
}
```

### 2. Class Selection and Locking Lifecycle

```
LOBBY entry (GSS — ADR-012):
  PlayerSessions inserted with all players having class = ClassId::Neutral, class_locked = false.

C2SClassChoice received (Lightyear MessageReceiver<C2SClassChoice>, LOBBY phase only):
  if player.class_locked == true   → silently discard (ADR-002 Rule 2 — no error response)
  if msg.class == ClassId::Neutral → silently discard (Neutral is not a valid player class)
  else → player.class = msg.class
       → broadcast S2COpponentClassSelected to other players (public visibility rule)

RSM LOBBY->DRAFT_INITIAL gate (ADR-009):
  Invariant enforced: sessions.all_classes_chosen() must return true.
  If false → refuse transition; lobby remains in LOBBY phase.
  On gate pass:
    sessions.lock_all_classes()      ← atomic; all class_locked = true
    emit LobbyComplete Message       ← triggers DRAFT_INITIAL entry
    (C2SClassChoice from this point is silently discarded regardless of class_locked,
     because the RSM phase gate rejects it at phase != LOBBY)

Post-LOBBY invariant:
  All players in PlayerSessions have class_locked == true and class != ClassId::Neutral.
  This invariant holds for the remainder of the session. No system may set class_locked = false.
```

### 3. SourceClass Component

Token entities (Mummy, Chacha Noir, Seeds, Madolls, La Gonflable, La Sacrifiée) are spawned with a `SourceClass(ClassId)` ECS component. Standard class and neutral card units receive no `SourceClass` component.

```rust
// server/src/core/board/components.rs

/// Identifies the class that spawned this token entity.
///
/// Set at spawn time by the token spawn function. Never mutated.
///
/// Absent on non-token units (standard class and neutral cards).
///
/// Used for:
///   LEADER bonus checks: filter With<SourceClass> + source_class == player's class
///   Miranda-stolen token integrity: stolen tokens retain their SourceClass;
///     the new controller's class does not override it.
///
/// Reflect intentionally NOT derived: server-only component; no scene
/// serialisation or Bevy inspector usage in the headless server build.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceClass(pub ClassId);

/// Marker component present on all token entities (Mummy, Madoll, Seed, etc.).
/// Used for Sacrifice Poupesque and Miranda filters alongside SourceClass.
#[derive(Component, Default)]
pub struct TokenUnit;
```

**Token spawn pattern (server-only):**

```rust
// All token spawn functions live in server/src/core/board/spawn.rs.
// They are never called from client code.

fn spawn_mummy(
    commands: &mut Commands,
    owner: PlayerId,
    lane: u8,
    cell: u8,
) {
    commands.spawn((
        UnitStats { hp: 2, atk: 2, mp: 3, ar: 0 },
        BoardPosition { lane, cell },
        UnitOwner(owner),
        SourceClass(ClassId::Xelor),   // set at spawn; never mutated
        TokenUnit,
        // No Transform — server-only logical entity; client renders via snapshot
    ));
}

fn spawn_madoll(commands: &mut Commands, owner: PlayerId, lane: u8, cell: u8) {
    commands.spawn((
        UnitStats { hp: 3, atk: 1, mp: 3, ar: 0 },
        BoardPosition { lane, cell },
        UnitOwner(owner),
        SourceClass(ClassId::Sadida),
        TokenUnit,
    ));
}
// ... one spawn function per token type; each hard-codes its ClassId::* variant
```

**Snapshot derivation:** When building `S2CGameSnapshot`, the snapshot system derives `UnitBoardState.source_class` from the presence of the `SourceClass` component:

```rust
// In snapshot builder:
let source_class: Option<ClassId> = world.get::<SourceClass>(unit_entity).map(|sc| sc.0);
// → Some(ClassId::Xelor) for a Mummy; None for a standard Iop Minion
```

### 4. Cross-System Effect Dispatch

Class-specific card effects execute as **plain Rust functions called from within the RESOLUTION system body**. They are NOT registered as standalone Bevy systems and do NOT communicate via buffered Messages within a RESOLUTION tick.

**Why not standalone systems or Messages:** Bevy systems take `SystemParam` types (Resources, Queries, MessageReaders). `PlayerId` is plain data — it cannot be passed as a system parameter. More importantly, buffered `MessageWriter/MessageReader` in Bevy 0.17+ delivers messages in the *next frame*, which would decouple RESOLUTION sub-steps that must execute in the same tick. Class effects are synchronous game logic, not cross-frame signals.

**Pattern:**

```rust
// server/src/core/resolution/system.rs

fn resolve_resolution(
    mut sessions:    ResMut<PlayerSessions>,
    mut objectives:  ResMut<ObjectiveState>,
    mut board:       ResMut<BoardState>,
    mut rng:         ResMut<ServerRng>,
    mut placements:  MessageReader<PlacementsCommitted>,
    // ... other resources
) {
    for placement_batch in placements.read() {
        let player_id = placement_batch.player_id;

        // Sub-step 1: apply spells in placement order
        for card_play in &placement_batch.cards {
            match card_play.effect {
                CardEffect::Gelure        => apply_gelure(&mut sessions, player_id),
                CardEffect::Xelorium      => apply_xelorium(&mut sessions, player_id),
                CardEffect::SangMeprise   => apply_sang_meprise(&mut sessions, &mut objectives, player_id),
                CardEffect::Punition(tgt) => apply_punition(&mut sessions, &mut objectives, player_id, tgt),
                // ...
            }
        }

        // Sub-step 2: charge movement (Rollback)
        apply_rollback_if_played(&mut sessions, &mut board, player_id, &placement_batch);

        // ... remaining sub-steps
    }
}

/// CS-1 formula: reserve_new = reserve + current_mana; current_mana_new = 0
fn apply_gelure(sessions: &mut PlayerSessions, player_id: PlayerId) {
    let p = sessions.players.get_mut(&player_id)
        .expect("apply_gelure: player not in session");
    p.reserve += p.current_mana;   // reserve/current_mana added by Economy ADR
    p.current_mana = 0;
}

/// CS-2 formula: self.reserve += opponent.current_mana; opponent.current_mana = 0
fn apply_xelorium(sessions: &mut PlayerSessions, caster_id: PlayerId) {
    // Must resolve after Xelorium's own mana cost is deducted (Economy Rule 4).
    // Caster's cost deduction happens in the cost-deduction pass before this call.
    let opponent_id = sessions.opponent_of(caster_id);
    let stolen = sessions.players[&opponent_id].current_mana;
    sessions.players.get_mut(&caster_id).expect("caster").reserve += stolen;
    sessions.players.get_mut(&opponent_id).expect("opponent").current_mana = 0;
}
```

**System ordering constraint:** All systems within the RESOLUTION `SystemSet` that take `ResMut<PlayerSessions>` (or any other mutable resource) MUST be scheduled with explicit `.before()` / `.after()` constraints. Bevy's multi-threaded executor panics in debug builds when two systems share a `ResMut<T>` without ordering. In practice the RESOLUTION batch is a single system (`resolve_resolution`), so contention risk is low — but any future split of RESOLUTION into sub-step systems must carry explicit ordering annotations.

### Architecture Diagram

```
                    Class System Architecture — Lanes and Lies

  ┌─────────────────────────────────────────────────────────────────────┐
  │  shared/src/card.rs  (ADR-006 — no Bevy deps)                       │
  │  ClassId { Iop, Cra, Sacrier, Xelor, Ecaflip, Sadida, Neutral }     │
  │  ← imported by protocol/, server/, client/                          │
  └───────────────────────┬─────────────────────────────────────────────┘
                          │
          ┌───────────────┴─────────────────────┐
          │                                       │
  ┌───────▼──────────────┐             ┌──────────▼──────────────────────────┐
  │  protocol/            │             │  server/                             │
  │                        │             │                                     │
  │  C2SClassChoice        │             │  core/session/state.rs:             │
  │    class: ClassId      │             │    PlayerSessions (Resource)        │
  │    ← Lightyear Message │             │      players: HashMap<PlayerId,     │
  │      (lightyear::      │             │        PlayerSessionData {          │
  │       prelude::Message)│             │          class: ClassId,            │
  │      NOT bevy::prelude │             │          class_locked: bool,        │
  │                        │             │          // future: gold, mana, ... │
  │  PlayerSnapshot        │             │        }>                           │
  │    class_id: ClassId   │             │                                     │
  │    (resolves NP-1)     │             │  core/board/components.rs:          │
  │                        │             │    SourceClass(ClassId) component   │
  │  UnitBoardState        │             │    (on token entities only;         │
  │    source_class:       │             │     never mutated after spawn)      │
  │      Option<ClassId>   │             │                                     │
  │    (resolves NP-2)     │             │  LOBBY handler (Lightyear server):  │
  └────────────────────────┘             │    MessageReceiver<C2SClassChoice>  │
                                         │    → update PlayerSessions.class    │
                                         │      if !class_locked               │
                                         │                                     │
                                         │  RSM LOBBY→DRAFT_INITIAL gate:      │
                                         │    all_classes_chosen() check       │
                                         │    lock_all_classes() on pass       │
                                         │                                     │
                                         │  RESOLUTION system body:            │
                                         │    class effects = plain Rust fns   │
                                         │    taking &mut PlayerSessions        │
                                         │    (not standalone Bevy systems)    │
                                         └─────────────────────────────────────┘
```

### Key Interfaces

```rust
// ── protocol/ additions ────────────────────────────────────────────────────────

/// Client sends during LOBBY to select or change class.
/// Derives Lightyear's Message trait (lightyear::prelude::Message),
/// NOT Bevy's Message trait (bevy::prelude::Message) — both exist in this project.
#[derive(lightyear::prelude::Message, Serialize, Deserialize, Clone, Debug)]
pub struct C2SClassChoice {
    pub class: ClassId,  // ClassId::Neutral is rejected server-side
}

/// Addition to existing PlayerSnapshot in S2CGameSnapshot (resolves NP-1).
pub struct PlayerSnapshot {
    pub class_id: ClassId,   // never ClassId::Neutral in any phase >= DRAFT_INITIAL
    // ... existing fields
}

/// Addition to existing UnitBoardState (resolves NP-2).
pub struct UnitBoardState {
    pub source_class: Option<ClassId>,  // Some for tokens; None for standard units
    // ... existing fields
}


// ── server/ public API ─────────────────────────────────────────────────────────

impl PlayerSessions {
    /// Returns locked class for a player. Panics with message if player not registered.
    pub fn class_of(&self, player_id: PlayerId) -> ClassId { ... }

    /// Returns true if all registered players have chosen a non-Neutral class.
    pub fn all_classes_chosen(&self) -> bool { ... }

    /// Sets class_locked = true for all players. Called once at LOBBY->DRAFT_INITIAL.
    pub fn lock_all_classes(&mut self) { ... }
}

/// LOBBY-phase C2SClassChoice handler.
fn handle_class_choice(
    mut receiver: MessageReceiver<C2SClassChoice>,
    mut sessions: ResMut<PlayerSessions>,
    session_reg:  Res<SessionRegistry>,
) {
    for (client_id, msg) in receiver.receive_messages() {
        let Some(player_id) = session_reg.player_for(client_id) else { continue; };
        let Some(player) = sessions.players.get_mut(&player_id) else { continue; };
        if player.class_locked { continue; }            // silent discard
        if msg.class == ClassId::Neutral { continue; }  // silent discard
        player.class = msg.class;
        // TODO: broadcast S2COpponentClassSelected to other players
    }
}
```

## Alternatives Considered

### Alternative 1: Standalone PlayerClasses Resource (`HashMap<PlayerId, ClassId>`)

- **Description**: Separate minimal `Res<PlayerClasses>` holding only class data, with a companion `PlayerClassLocked(HashSet<PlayerId>)` resource for lock state.
- **Pros**: Smaller resource scope; systems that only need class need not take the full `PlayerSessions`.
- **Cons**: Adds two resources for one concept (class identity + lock). When Economy ADR adds gold/mana/reserve and Card Acquisition adds hand, the server World ends up with 4+ independent per-player resources, all accessed in most RESOLUTION systems. A unified `PlayerSessions` (consistent with `PlayerPools` from ADR-006) is the cleaner single-lookup pattern.
- **Rejection Reason**: User selected unified `PlayerSessionState` pattern for consistency with established project resource conventions.

### Alternative 2: ClassAssignment Component on PlayerEntity

- **Description**: Each player gets a persistent ECS entity at LOBBY entry with `ClassAssignment { class: Option<ClassId>, locked: bool }`. Systems query `Query<&ClassAssignment, With<ClassLocked>>`.
- **Pros**: Native ECS query patterns; `ClassLocked` marker provides type-level enforcement.
- **Cons**: Player is a session concept, not a spatial game-world entity. Spawning/despawning player entities adds lifecycle overhead. `Query::single()` (returns `Result` in Bevy 0.16+) adds error handling everywhere class is read. Inconsistent with `PlayerPools` and `AuctionState` (both Resources, not entities).
- **Rejection Reason**: Resource pattern preferred; entity lifecycle for session-scoped data is unnecessary overhead.

### Alternative 3: Buffered Messages for Class Effect Dispatch

- **Description**: Class effect handlers emit typed `Message` types (e.g., `ClassEffectGelure { player_id }`) consumed by Economy/Objective systems via `MessageReader<T>` in the next frame.
- **Pros**: Clean separation between class intent and system effect; each system owns its mutation path.
- **Cons**: Bevy 0.17+ buffered `Message` delivery is frame-delayed. RESOLUTION sub-steps must resolve within a single server tick to maintain sub-step ordering (Xelorium at sub-step 1 before Rollback at sub-step 2). A 1-frame delay decouples sub-steps, breaking the ordering contract in the Class System GDD. Additionally, `PlayerId` cannot be passed as a Bevy `SystemParam`, requiring an additional message carry structure.
- **Rejection Reason**: RESOLUTION sub-step ordering requires same-frame execution. Class effect functions are plain Rust helpers called from within the RESOLUTION system body — not standalone systems and not Message producers within a RESOLUTION tick.

## Consequences

### Positive

- `class_of()` is an O(1) HashMap lookup on a u8 key. No overhead in any class effect path.
- `class_locked: bool` is an explicit, readable invariant. No implicit phase-check required — the field is the contract.
- `SourceClass(ClassId)` is queryable via Bevy's standard ECS filters (`With<SourceClass>`, `Without<SourceClass>`) — LEADER checks and Miranda filters compose naturally without auxiliary data structures.
- `UnitBoardState.source_class` resolves NP-2 (Miranda-stolen token source tracking on reconnect).
- `PlayerSnapshot.class_id` resolves NP-1 (opponent class display throughout the game, CS-AC-03).
- Class effects as plain Rust functions preserve RESOLUTION sub-step ordering with no frame-delay risk.
- `PlayerSessions` is extensible: Economy and Card Acquisition ADRs add fields to `PlayerSessionData` without changing the storage architecture.
- `#[derive(Default)]` on both `PlayerSessions` and `PlayerSessionData` enables `app.init_resource::<PlayerSessions>()` and simplifies test setup.

### Negative

- Systems that mutate any field in `PlayerSessionData` must take `ResMut<PlayerSessions>`. In Bevy's multi-threaded executor, two systems sharing `ResMut<PlayerSessions>` without explicit ordering will panic in debug builds. All RESOLUTION sub-step systems that write `PlayerSessions` must be explicitly ordered (see Implementation Guidelines §5).
- `PlayerSessions` grows over time as Economy, Card Acquisition, and Combat ADRs add fields. The struct becomes the canonical per-player session store. Mitigated by documenting field ownership per-ADR and enforcing it in code review.
- `ClassId::Neutral` as the "not yet chosen" sentinel overloads the enum variant with lifecycle meaning beyond card classification. Any system that operates on `player.class` at phase ≥ DRAFT_INITIAL must not encounter `Neutral` — protected by the `lock_all_classes()` + RSM gate, and by a `debug_assert` in `lock_all_classes`.

### Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Economy ADR introduces a separate `EconomyState` resource instead of adding fields to `PlayerSessions`, creating split per-player state | Medium | Medium | This ADR explicitly documents that Economy fields (`gold`, `current_mana`, `reserve`) are added to `PlayerSessionData`. Economy ADR author must read this ADR and extend the existing struct. Registry entry for `player_class` state ownership enforces the single-resource pattern. |
| Two RESOLUTION sub-step systems both take `ResMut<PlayerSessions>` without ordering, causing Bevy debug panic | Low | Medium | All RESOLUTION systems sharing `ResMut<PlayerSessions>` must be in an explicit `.before()`/`.after()` chain. Document in control manifest. |
| `ClassId::Neutral` sentinel leaks into in-game code as a valid player class | Medium | Low | `all_classes_chosen()` gate refuses LOBBY→DRAFT_INITIAL if any player has `class == Neutral`. `debug_assert` in `lock_all_classes` fires if this invariant is violated. |
| `SourceClass` component missing from a new token type's spawn function | Medium | Medium | Unit test spawning one of each token type, building the snapshot, and asserting `source_class.is_some()` for each. Add to regression suite. |
| `MessageReceiver<C2SClassChoice>.receive_messages()` method name shifts in a Lightyear 0.26.x patch | Low | Low | Pin exact Lightyear patch version in `Cargo.toml`. Wrap the receiver call in `server/src/lobby/handler.rs` — one file to update if the name changes. |
| Second system drains `MessageReceiver<C2SClassChoice>`, silently starving the LOBBY handler | Low | High | Register as forbidden pattern in architecture registry: only one system may drain `MessageReceiver<C2SClassChoice>` (same pattern as ADR-013 for `MessageReceiver<C2SAuctionBid>`). |

## GDD Requirements Addressed

| GDD System | Requirement | How This ADR Addresses It |
|------------|-------------|--------------------------|
| `class-system.md` | Class lifecycle: LOBBY selection, locked at DRAFT_INITIAL, immutable thereafter (CS-AC-01, CS-AC-02) | `class_locked: bool` in `PlayerSessionData`; `all_classes_chosen()` gate; `lock_all_classes()` called atomically by RSM handler before `LobbyComplete` |
| `class-system.md` | Class publicly visible to opponent throughout game (CS-AC-03) | `PlayerSnapshot.class_id: ClassId` added to `S2CGameSnapshot` (resolves NP-1) |
| `class-system.md` | Token `source_class` tag for LEADER bonus checks and Miranda-stolen token integrity (Detailed Rules §Token registry) | `SourceClass(ClassId)` component on token entities; `UnitBoardState.source_class: Option<ClassId>` in snapshot (resolves NP-2) |
| `class-system.md` | All 11 cross-system formulas (CS-1 through CS-11) must resolve within RESOLUTION sub-step ordering | Plain Rust helper functions called from within the RESOLUTION system body — same-frame, sub-step-ordered execution |
| `class-system.md` | Class card shop filter: class slot rolls from player's class library only (CS-AC-26) | `sessions.class_of(player_id)` provides `ClassId` to `PlayerPool::draw_class_card(class, ...)` (ADR-006 interface) |
| `class-system.md` | Cross-class draw legality: Drheller/uniform draws bypass class filter (CS-AC-27, CS-AC-27b) | `PlayerPool::draw_random(filter: &PoolFilter)` with `filter.class = None` — class filter not applied; no play-time class restriction gate |
| `class-system.md` | Token SILENCE non-propagation: spawned tokens are independent entities | Tokens have no component linking them to their parent unit after spawn. `SourceClass` is a permanent identity tag, not a propagation channel. |
| `game-session-system.md` | Class Ready gate: LOBBY→DRAFT_INITIAL refused if any player has `class = None` | `all_classes_chosen()` enforces `class != ClassId::Neutral` for all players; called by RSM before `LobbyComplete` |
| `network-protocol.md` | NP-1: `PlayerSnapshot` missing `class_id` field | `PlayerSnapshot.class_id: ClassId` added to `protocol/src/snapshot.rs` |
| `network-protocol.md` | NP-2: `UnitBoardState` missing `source_class` field | `UnitBoardState.source_class: Option<ClassId>` added; derived from `SourceClass` component at snapshot build time |

## Performance Implications

- **CPU**: `PlayerSessions` HashMap lookup is O(1) with `PlayerId` (u8 key — tiny, cache-friendly). Class effect functions are inlined plain Rust calls in the RESOLUTION loop — no system dispatch overhead. `SourceClass` is 1 byte (enum stored as u8 in Bevy's archetype table); at ≤ 20 total token entities per board the archetype cost is negligible.
- **Memory**: `PlayerSessionData` starts at 2 bytes (u8 enum + bool). Grows as Economy fields are added (estimated ~48 bytes at full definition). `HashMap<PlayerId, PlayerSessionData>` for 2 players is trivially small (~200 bytes including HashMap overhead).
- **Network**: `PlayerSnapshot.class_id` adds 1 byte per player to `S2CGameSnapshot` (2 bytes total, 2-player game) — well within the 16 KB snapshot budget (ADR-002). `UnitBoardState.source_class: Option<ClassId>` adds 2 bytes per unit in the snapshot; at ≤ 40 units total, 80 bytes added — negligible.
- **Load Time**: No impact. `PlayerSessions` is inserted at `SessionReady`, not at server startup.

## Migration Plan

This is a greenfield system — no existing class implementation to migrate from.

Adoption sequence:
1. Add `PlayerSessions` Resource to `server/src/core/session/state.rs` with class-only fields (this ADR's scope). Wire insertion before `SessionReady` and removal on `GameOverEmitted` (ADR-012 contract).
2. Add `C2SClassChoice` to `protocol/src/messages.rs`. Add `class_id: ClassId` to `PlayerSnapshot`. Add `source_class: Option<ClassId>` to `UnitBoardState`.
3. Implement LOBBY-phase handler for `C2SClassChoice` — update `player.class`, reject `ClassId::Neutral`, reject if `class_locked`.
4. Implement RSM LOBBY→DRAFT_INITIAL gate calling `all_classes_chosen()` then `lock_all_classes()`.
5. Add `SourceClass(ClassId)` component to `server/src/core/board/components.rs`. Add it to each of the 7 token spawn functions.
6. Implement snapshot builder extension: derive `UnitBoardState.source_class` from `SourceClass` component.
7. Write unit tests for CS-AC-01, CS-AC-02, CS-AC-03 (class lifecycle and snapshot fields).
8. Each class-effect story (Xelor reserve, Sacrier reveals, Sadida seeds, Ecaflip dice, Miranda) adds a plain Rust helper function called from within the RESOLUTION system body, following the `apply_gelure` pattern above.

**Implementation Guidelines:**
1. Import `ClassId` from `shared::card::ClassId` — never redefine it.
2. All class effect functions take `&mut PlayerSessions` (and other `&mut` state) as plain parameters — NOT as Bevy system params.
3. All RESOLUTION systems sharing `ResMut<PlayerSessions>` must be in an explicit ordering chain (`.before()`/`.after()`) within their `SystemSet`.
4. `MessageReceiver<C2SClassChoice>` must be drained by exactly one system (the LOBBY handler). Register this as a forbidden-pattern in the architecture registry.
5. Never set `class_locked = false` after `lock_all_classes()` has been called.

## Validation Criteria

- [ ] `PlayerSessions` present in server World between `SessionReady` and `GameOverEmitted`; absent before/after. Verified by lifecycle integration test (ADR-012 pattern).
- [ ] `C2SClassChoice` with a valid `ClassId` (non-Neutral) while in LOBBY phase updates `player.class`. Verified by unit test (CS-AC-01).
- [ ] `C2SClassChoice` with `ClassId::Neutral` is silently rejected; `player.class` unchanged. Verified by unit test.
- [ ] `C2SClassChoice` received after `class_locked == true` is silently discarded; `player.class` unchanged. Verified by unit test (CS-AC-01).
- [ ] LOBBY→DRAFT_INITIAL transition refused when any player has `class == ClassId::Neutral`. Verified by unit test (CS-AC-02).
- [ ] After gate pass, all players have `class_locked == true`. Verified by unit test (CS-AC-02).
- [ ] `S2CGameSnapshot.PlayerSnapshot.class_id` equals each player's locked class. Verified by snapshot unit test (CS-AC-03, NP-1).
- [ ] Each of the 7 token types spawns with a `SourceClass` component. `UnitBoardState.source_class` is `Some(class)` for each in the snapshot. Verified by token spawn integration test (NP-2).
- [ ] Non-token units have no `SourceClass` component; `UnitBoardState.source_class` is `None`. Verified alongside token test.
- [ ] Miranda-stolen token retains its original `SourceClass` component; `source_class` in snapshot is unchanged after control transfer. Verified by Miranda integration test.

## Related Decisions

- [ADR-002 — Client-Server Authority Model](./adr-002-client-server-authority.md) — `PlayerSessions` is a server-only Resource; the crate-level isolation pattern enforces it.
- [ADR-005 — Server-side RNG](./adr-005-server-side-rng.md) — Ecaflip dice effects (CS-9, CS-10) consume from the RESOLUTION RNG chain; class-effect stories will add `ResolveEcaflip` RngEvent variants per ADR-005 §4 ordering.
- [ADR-006 — Card Data Schema](./adr-006-card-data-schema.md) — `ClassId` enum defined there; `PlayerPool::draw_class_card(class: ClassId, ...)` receives the class from `PlayerSessions.class_of()`.
- [ADR-009 — RSM Phase State](./adr-009-rsm-phase-state.md) — LOBBY phase gate; `all_classes_chosen()` is the gate predicate; `LobbyComplete` Message triggers DRAFT_INITIAL.
- [ADR-010 — RSM Event Bus](./adr-010-rsm-event-bus.md) — `LobbyComplete` follows the `#[derive(Message)]` bus pattern established by ADR-010.
- [ADR-012 — Session Ready Delivery](./adr-012-session-ready-delivery.md) — `PlayerSessions` lifecycle (insert before `SessionReady`, remove on `GameOverEmitted`) mirrors the `ServerRng` and `PlayerPools` lifecycle contracts.
- [ADR-013 — Auction System State](./adr-013-auction-system-state.md) — Establishes the `MessageReceiver<T>` single-drain forbidden pattern that ADR-014 extends to `MessageReceiver<C2SClassChoice>`.
- `design/gdd/class-system.md` — Authoritative design reference for all 6 classes, 7 token types, 11 formulas, 27 ACs.
- `design/gdd/network-protocol.md` — Open questions NP-1 through NP-5; this ADR resolves NP-1 and NP-2.
