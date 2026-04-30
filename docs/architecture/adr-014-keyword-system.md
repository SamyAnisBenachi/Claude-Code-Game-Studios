# ADR-014: Keyword System — ECS State Architecture, Module Boundary, and Protocol Schema

## Status
Proposed

## Date
2026-04-30

## Engine Compatibility

| Field | Value |
|-------|-------|
| **Engine** | Bevy 0.18 + Lightyear 0.26 |
| **Domain** | Core / Gameplay Logic (Keyword Resolution) |
| **Knowledge Risk** | HIGH — 4 versions of breaking changes post-LLM-cutoff (0.15–0.18) |
| **References Consulted** | `docs/engine-reference/bevy/VERSION.md`, ADR-002, ADR-005, ADR-006, ADR-009, ADR-010, `design/gdd/keyword-system.md` |
| **Post-Cutoff APIs Used** | `#[derive(Message)]` + `MessageWriter<T>` / `MessageReader<T>` (Bevy 0.17+ Message/Event split); `#[derive(Component)]` with `Option<Entity>` field (Entity handle semantics Bevy 0.18); component lifecycle hooks (`on_remove`) — NOT used here; manual query scan used instead |
| **Verification Required** | (1) Confirm `app.add_message::<KeywordTriggered>()` is registered in `KeywordPlugin` — missing registration panics at first write. (2) Confirm `#[serde(tag = "kw", content = "val")]` round-trips correctly for all `Keyword` variants in a unit test before merging ADR-006 amendment. (3) Verify `&Entities` system param API in Bevy 0.18 for the BODYGUARD stale-reference cleanup system. |

## ADR Dependencies

| Field | Value |
|-------|-------|
| **Depends On** | ADR-002 (client-server authority — Accepted); ADR-005 (server-side RNG — Accepted; 3 new seed slots required before keyword implementation, see Decision Part 7); ADR-006 (card data schema — Accepted; amendment required to extend `SimpleKeyword` enum); ADR-009 (RSM phase state — Accepted); ADR-010 (RSM event bus — Accepted; `ResolutionPhaseEntered` triggers keyword system init) |
| **Enables** | Combat Resolution M2 (needs `UnitKeywordState` component and `server/feature/keyword/effects.rs` interface); Board Rendering (keyword state glyphs read `UnitKeywordState`); Card Animations (`KeywordTriggered` and `DisplacementEvent` protocol types); Network Protocol OQ-NP1/NP4/NP5 resolution |
| **Blocks** | Any story implementing keyword resolution, combat sub-step execution, or board rendering of keyword state glyphs. ADR-006 amendment must be merged before any keyword card data encoding. ADR-005 seed slot additions must be registered before KW-033b can be implemented. |
| **Ordering Note** | ADR-006 amendment and ADR-005 seed slot additions are **pre-implementation gates** — they must be done before any keyword story is opened, but this ADR can be Accepted and stories authored against it immediately. |

## Context

### Problem Statement

The `keyword-system.md` GDD defines ~28 keywords with 41 BLOCKING acceptance criteria,
6 persistent runtime states, and a full replication contract. No ADR specifies:

1. How keyword runtime states (SHIELD, STUN, SILENCE, LEADER, BODYGUARD, OUTNUMBERED)
   are stored as ECS components on board unit entities
2. How keyword resolution integrates with Combat Resolution's 6-sub-step structure —
   what the interface between the two modules looks like
3. Where keyword logic lives in the crate hierarchy (`server/feature/keyword/` vs. inline
   in combat)
4. The complete `Keyword`/`SimpleKeyword` enum for card declarations (ADR-006 defines only
   8 of ~28 keywords; `SimpleKeyword::Charge` conflicts with the GDD's HASTE rename — OQ-KS2)
5. The `KeywordKind`, `KeywordPayload`, and `DisplacementEvent` types needed for network
   protocol messages (OQ-NP1, OQ-NP5)
6. The 3 RNG seed slots required for RANGE equidistant selection, TELEPORT random
   destination, and Strich CHANGE LANE selection (OQ-KS1)

### Constraints

- All keyword resolution is server-authoritative (ADR-002). `UnitKeywordState` lives in
  `server/` only; clients receive keyword state via `UnitBoardState` fields in
  `S2CGameSnapshot` (OQ-NP4).
- `Entity` handles in `UnitKeywordState.bodyguard_protects` are valid within one session's
  ECS World. They must NEVER be serialized into `protocol/` types — the network protocol
  uses a stable `EntityId` (session-scoped u32).
- The GDD explicitly forbids lane-scoped storage of the BODYGUARD bond: "BODYGUARD
  protection MUST be stored as a unit-to-unit entity bond (`Option<EntityId>` on the
  BODYGUARD's component), NOT as a lane-scoped attribute."
- Bevy 0.18 does not provide automatic reference invalidation for `Option<Entity>` fields
  when a referenced entity despawns — manual cleanup is required.

### Requirements

- One Bevy `Component` per board unit entity holds all 6 persistent keyword states
- BODYGUARD bond stored as `Option<Entity>` (typed Bevy handle) on the BODYGUARD entity —
  stable across CHANGE LANE
- Keyword effects module is separate from combat resolution module — enforces GDD ownership
  boundary: "Combat Resolution owns *when*; Keyword System owns *what*"
- Complete `SimpleKeyword` enum (all ~28 keywords, HASTE replaces Charge) defined and
  traceable to an ADR-006 amendment
- `KeywordKind`, `KeywordPayload`, `DisplacementEvent` types defined in `protocol/` crate
- 3 RNG seed slots formally tracked as a pre-implementation gate against ADR-005

## Decision

### Part 1: UnitKeywordState — Monolithic Component

All 6 persistent keyword states are stored as a single `UnitKeywordState` component on
each board unit entity in `server/feature/keyword/components.rs`.

```rust
// server/feature/keyword/components.rs

use bevy::prelude::*;

/// All persistent keyword state for one board unit entity.
///
/// All 6 states co-located to avoid archetype migrations during sub-step
/// processing (up to 10 units × 6 states × 12 sub-step boundaries = up to
/// 720 potential migrations per RESOLUTION round if stored separately).
///
/// BODYGUARD bond: Option<Entity> is a typed Bevy handle, NOT a lane index.
/// Stable across CHANGE LANE — entity ID does not change when position changes.
/// NEVER serialize bodyguard_protects into protocol/ types; use EntityId there.
#[derive(Component, Clone, Debug, Default)]
pub struct UnitKeywordState {
    // ── SHIELD ───────────────────────────────────────────────────────
    pub shield_active: bool,

    // ── STUN ─────────────────────────────────────────────────────────
    pub stun_active: bool,

    // ── SILENCE ──────────────────────────────────────────────────────
    /// Some(r) = silenced until end of round r (inclusive). None = not silenced.
    pub silenced_until_round: Option<u8>,

    // ── LEADER bonus (snapshotted at RESOLUTION entry) ────────────────
    /// ATK bonus granted by a living LEADER's family snapshot. 0 = none.
    pub leader_bonus_atk: u8,
    /// HP bonus granted by a living LEADER's family snapshot. 0 = none.
    pub leader_bonus_hp: u8,

    // ── BODYGUARD bond ────────────────────────────────────────────────
    /// The entity this BODYGUARD unit is protecting. Set in SS1.
    /// Cleared when BODYGUARD despawns (see bodyguard_cleanup_system).
    /// FORBIDDEN: do not use a lane/cell index — bond must survive CHANGE LANE.
    pub bodyguard_protects: Option<Entity>,

    // ── OUTNUMBERED (derived, cached per sub-step boundary) ───────────
    /// Cached result of outnumbered(player) for the current sub-step.
    /// Re-evaluated at each sub-step boundary per keyword-system.md Formula 3.
    pub outnumbered_active: bool,
}
```

### Part 2: Module Structure

```
server/feature/keyword/
  mod.rs            ← KeywordPlugin (component registration, system scheduling)
  components.rs     ← UnitKeywordState
  effects.rs        ← keyword effect functions (called BY combat resolution)
  state_eval.rs     ← leader_snapshot_system, eval_outnumbered_system,
                       bodyguard_cleanup_system
  movement.rs       ← repel_destination(), attract_destination() formula impls
```

**Interface boundary:** `server/feature/combat/` calls into `server/feature/keyword::effects::*`
functions. The keyword module does NOT schedule its own systems against the combat sub-step
timeline — that timeline is owned entirely by the combat resolution system. Keyword effect
functions are called by combat resolution as plain function calls with query references.

**System scheduling (within KeywordPlugin):**

```
Update set:
  leader_snapshot_system      runs on ResolutionPhaseEntered (before SS1)
  eval_outnumbered_system     called by combat resolution at each sub-step boundary
  bodyguard_cleanup_system    runs in PostUpdate (after despawn commands flush)
```

`bodyguard_cleanup_system` runs in `PostUpdate` to guarantee it executes after any unit
despawn commands have been applied. It scans all entities with `bodyguard_protects.is_some()`
and clears the field if the referenced entity no longer exists:

```rust
// server/feature/keyword/state_eval.rs

pub fn bodyguard_cleanup_system(
    mut units: Query<&mut UnitKeywordState>,
    entities: &Entities,   // bevy::ecs::entity::Entities — O(1) alive check
) {
    for mut kw_state in units.iter_mut() {
        if let Some(bond_target) = kw_state.bodyguard_protects {
            if !entities.contains(bond_target) {
                kw_state.bodyguard_protects = None;
            }
        }
    }
}
```

`&Entities` is the correct Bevy 0.18 system param for an alive entity check without a
full `Query`. Verify exact symbol path against Bevy 0.18 docs before merging.

### Part 3: Extended Keyword Enum (ADR-006 Amendment Required)

The following supersedes the partial `SimpleKeyword` definition in ADR-006 and must be
applied as an amendment to `shared/src/card.rs` before any keyword story is implemented.

```rust
// shared/src/card.rs  (ADR-006 amendment)

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SimpleKeyword {
    // ── Timing triggers ──────────────────────────────────────────────
    Appearance,
    Death,
    FinalBlow,
    Counterattack,
    StartOfTurn,
    EndOfTurn,
    // ── Combat keywords ──────────────────────────────────────────────
    FirstStrike,
    Haste,           // Renamed from Charge (OQ-KS2). Removes summoning sickness.
    Wall,
    Bodyguard,
    Irremovable,
    Untargetable,
    Shield,
    Leader,
    Outnumbered,
    ArmorPiercing,
    Silence,
    Stun,
    // ── Movement keywords (no-parameter) ─────────────────────────────
    Teleport,
    ChangeLane,
    // Note: CHARGE X (movement), RANGE 1-X, RESISTANCE X, VULNERABILITY X,
    // REPEL X, ATTRACT X are parameterized variants in Keyword below.
}

/// Adjacently tagged to support unit-variant newtypes (Simple(SimpleKeyword))
/// alongside struct variants (RangeX { max_range: u8 }).
///
/// #[serde(tag = "kw", content = "val")] serializes as:
///   Simple(Shield)          → { "kw": "Simple", "val": "Shield" }
///   RangeX { max_range: 3 } → { "kw": "RangeX", "val": { "max_range": 3 } }
///
/// DO NOT use #[serde(tag = "kw")] (internally tagged) — it fails at runtime
/// for newtype variants whose inner type serializes as a scalar (not a map).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "kw", content = "val")]
pub enum Keyword {
    Simple(SimpleKeyword),
    RangeX         { max_range: u8 },   // RANGE 1-X
    ChargeXMove    { cells: u8 },       // CHARGE X (movement keyword)
    ResistanceX    { value: u8 },
    VulnerabilityX { value: u8 },       // NEW
    RepelX         { distance: u8 },    // NEW
    AttractX       { distance: u8 },    // NEW
}
```

**Migration note:** `SimpleKeyword::Charge` (ADR-006) is removed; `SimpleKeyword::Haste`
replaces it. Any existing fixtures or code using `Keyword::Simple(SimpleKeyword::Charge)`
must be updated. `cards.json` must use `"Haste"` for the combat keyword. A round-trip
JSON test for all 7 `Keyword` variants must pass before the amendment merges. This
resolves OQ-KS2.

### Part 4: KeywordKind, KeywordPayload, DisplacementEvent (protocol/ crate)

Resolves OQ-NP1 and OQ-NP5 from `keyword-system.md`.

```rust
// protocol/src/keyword.rs  (new file)

use serde::{Serialize, Deserialize};
use crate::{EntityId, SubStep};

/// Identifies which keyword triggered a network event.
/// Used in S2CResolutionEvent::KeywordTriggered { unit_id, keyword, sub_step, payload }.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum KeywordKind {
    ShieldConsumed,
    StunApplied,
    SilenceApplied,
    InjuredBonusActive,
    LeaderSnapshotTaken,
    BodyguardBondCreated,
    BodyguardBondBroken,
    OutnumberedFlipped,
}

/// Per-keyword payload carried in KeywordTriggered events.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum KeywordPayload {
    ShieldConsumed,
    StunApplied,
    SilenceApplied       { duration_rounds: u8 },
    InjuredBonusActive   { granted_keyword: InjuredGrantedKeyword },
    LeaderSnapshotTaken  { leader_unit_id: EntityId },
    BodyguardBondCreated { bodyguard_id: EntityId, protected_id: EntityId },
    BodyguardBondBroken  { bodyguard_id: EntityId },
    OutnumberedFlipped   { player_id: u8, active: bool },
}

/// Which keyword the INJURED state granted as a bonus.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum InjuredGrantedKeyword {
    FirstStrike,
    // Additional per-card INJURED-bonus keywords added as cards are authored.
}

/// Displacement event for REPEL / ATTRACT / TELEPORT animations on the client.
///
/// to_cell reflects the *actual* final position (after Trap interruption or
/// IRREMOVABLE block), not the formula-computed destination.
/// was_blocked = true when IRREMOVABLE rejected displacement — client plays
/// Void flat flash instead of a slide animation.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DisplacementEvent {
    pub unit_id:     EntityId,
    pub attacker_id: Option<EntityId>,
    pub keyword:     DisplacementKind,
    pub from_cell:   u8,
    pub to_cell:     u8,
    pub sub_step:    SubStep,
    pub was_blocked: bool,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplacementKind {
    Repel,
    Attract,
    Teleport,
}
```

### Part 5: LEADER Snapshot System

```rust
// server/feature/keyword/state_eval.rs

/// Runs at RESOLUTION entry (on ResolutionPhaseEntered, before SS1).
/// Snapshots LEADER bonuses onto all eligible family units.
/// A silenced LEADER (silenced_until_round covers the current round) grants no bonus.
///
/// IMPORTANT: app.add_message::<KeywordTriggered>() must be registered in
/// KeywordPlugin or this MessageWriter will panic at first write.
pub fn leader_snapshot_system(
    mut units: Query<(Entity, &CardId, &UnitBoardOwner, &UnitKeywordState)>,
    mut family_members: Query<(&CardId, &UnitBoardOwner, &mut UnitKeywordState)>,
    card_catalog: Res<CardCatalog>,
    current_round: Res<CurrentRound>,
    mut keyword_triggered: MessageWriter<KeywordTriggered>,
) {
    // 1. Collect all living LEADER units not silenced this round.
    // 2. For each LEADER, find all entities of the same family + owner.
    // 3. Write leader_bonus_atk / leader_bonus_hp to each family member's UnitKeywordState.
    // 4. Emit KeywordTriggered { keyword: LeaderSnapshotTaken, leader_unit_id } for client tint.
    //
    // Snapshot semantics: LEADER units that entered play in SS1 of THIS round are
    // NOT included (not on board at RESOLUTION entry). LEADER killed in SS4 does NOT
    // revoke the snapshot; fields persist until RESOLUTION ends and are cleared at
    // the start of the next round's leader_snapshot_system run.
}
```

### Part 6: OUTNUMBERED Evaluation

```rust
// server/feature/keyword/state_eval.rs

/// Called by combat resolution at the start of each sub-step boundary.
/// Re-evaluates outnumbered(player) from global board counts.
/// Emits OutnumberedFlipped ONLY when the boolean transitions (bandwidth-efficient).
///
/// Formula 3: outnumbered(player) = count(alive_units(player)) < count(alive_units(opponent))
/// Traps excluded from board count (face-down, not participating in counts).
pub fn eval_outnumbered_system(
    mut units: Query<(&UnitBoardOwner, &mut UnitKeywordState)>,
    board_counts: Res<BoardUnitCounts>,
    mut keyword_triggered: MessageWriter<KeywordTriggered>,
) {
    // Update outnumbered_active on each unit; emit OutnumberedFlipped on transition.
}
```

### Part 7: RNG Seed Slots Gate (OQ-KS1)

Three RESOLUTION-chain seed slots must be added to ADR-005's consumption order table
before any keyword story opens. KW-033b is formally BLOCKED until this is done.

| Slot name | Fires when | Sub-step(s) | Ordering rule |
|-----------|-----------|-------------|---------------|
| `range_equidistant_select` | Multiple targets equidistant from a RANGE attacker | SS3 (RANGE+FIRST STRIKE), SS6 (standard RANGE) | Ascending player_id, then ascending lane (ADR-005 Rule 6) |
| `teleport_random_dest` | TELEPORT card text specifies a random destination | Within triggering sub-step | Ascending player_id, then ascending lane |
| `strich_change_lane_select` | Strich auto-CHANGE LANE fires; both adjacent lanes valid | After triggering sub-step | Ascending player_id |

### Architecture Diagram

```
server/feature/keyword/               server/feature/combat/
  components.rs                         sub_steps.rs
    UnitKeywordState ◄── reads/writes ── execute_ss3()
    (Component on                           ↳ calls keyword::effects::apply_first_strike()
     board unit entities)                   ↳ calls keyword::effects::check_shield_absorb()
                                            ↳ calls keyword::state_eval::eval_outnumbered()
  effects.rs
    apply_first_strike(...)             sub_steps.rs: execute_ss1()
    apply_shield_absorb(...)              ↳ calls keyword::effects::apply_appearance()
    apply_bodyguard_bond(...)             ↳ calls keyword::state_eval::leader_snapshot_system
    apply_movement_keyword(...)

  movement.rs
    repel_destination(target_cell, owner, x) -> u8
    attract_destination(caster_cell, target_cell, x) -> u8
    (Pure functions — i32 intermediate arithmetic, clamp to u8)

  state_eval.rs
    leader_snapshot_system    (system — runs before SS1)
    eval_outnumbered_system   (called at each sub-step boundary)
    bodyguard_cleanup_system  (PostUpdate — clears stale Option<Entity> refs)

protocol/src/keyword.rs
  KeywordKind, KeywordPayload, DisplacementEvent, DisplacementKind
  (used in S2CResolutionEvent — owned by network-protocol.md)
```

### Key Interfaces

**Keyword effects interface (called by combat resolution):**

```rust
// server/feature/keyword/effects.rs
pub fn apply_first_strike(attacker: Entity, target: Entity, world: &mut World) -> DamageResult;
pub fn check_shield_absorb(kw_state: &UnitKeywordState, sub_step: SubStep) -> bool;
pub fn apply_bodyguard_bond(bodyguard: Entity, protected: Entity, world: &mut World);
pub fn apply_repel(target: Entity, distance: u8, owner: PlayerSide, world: &mut World) -> u8;
pub fn apply_attract(caster_cell: u8, target: Entity, distance: u8, world: &mut World) -> u8;
// ... one function per keyword effect category
```

**Movement formulas (pure functions — no world access, no panic on valid input):**

```rust
// server/feature/keyword/movement.rs

/// Formula 1 from keyword-system.md.
/// Uses i32 intermediate to prevent u8 underflow; result clamped to [1, 8].
pub fn repel_destination(target_cell: u8, owner: PlayerSide, x: u8) -> u8;

/// Formula 2 from keyword-system.md.
/// effective_pull = min(x, |caster_cell - target_cell|); result in [1, 8].
pub fn attract_destination(caster_cell: u8, target_cell: u8, x: u8) -> u8;
```

## Alternatives Considered

### Alternative A (Chosen): Monolithic UnitKeywordState

See Decision Part 1. Chosen for co-location of frequently co-read states, avoidance of
archetype migrations, and direct mapping to `UnitBoardState` network fields.

### Alternative B: Individual Components per Keyword State

- **Description:** `ShieldActive`, `StunState`, `SilencedState`, `LeaderBonus`,
  `BodyguardBond` as separate optional components added/removed via `Commands`.
- **Pros:** Idiomatic Bevy flag pattern; query filters by presence at zero archetype cost.
- **Cons:** Up to 6 `Commands::insert/remove` calls per unit per sub-step = up to 720
  archetype migrations per RESOLUTION round for 10 units. Snapshot serialization requires
  querying 6 separate component types per unit. Harder to map to the flat `UnitBoardState`
  network struct.
- **Rejection Reason:** Archetype migration cost at RESOLUTION frequency is
  disproportionate to the query ergonomics benefit. At most 10 units on board, individual
  components are not worth the structural overhead.

### Alternative C: Hybrid — Monolithic State + Separate BodyguardBond Component

- **Description:** `UnitKeywordState` holds the 5 non-bond states; `BodyguardBond
  { protects: Entity }` is a separate optional component added only to BODYGUARD units.
- **Pros:** "Find all units with active BODYGUARD bond" is a zero-cost archetype query.
- **Cons:** Reverse-lookup ("does entity X have a BODYGUARD?") is cheap with ≤10 BODYGUARD
  units. The replication contract still requires `bodyguard_protects: Option<EntityId>` in
  `UnitBoardState` — the server derives it from `UnitKeywordState` anyway, making the
  dual-representation redundant.
- **Rejection Reason:** The bond is unidirectional (BODYGUARD → protected unit). The
  marginal query ergonomics of a separate component do not justify the added component
  type and dual-write complexity.

## Consequences

### Positive

- Single component access per unit per sub-step — no multi-query joins in the hot
  RESOLUTION path
- GDD's "CR owns when; Keyword owns what" boundary enforced at the module level
- Both enum types (card declaration + network events) fully specified — no implementer
  ambiguity
- BODYGUARD bond is `Option<Entity>` — entity-stable by construction; CHANGE LANE cannot
  orphan it
- OQ-NP1, OQ-NP5, OQ-KS2 resolved by this ADR; OQ-KS1 formally tracked as a
  pre-implementation gate; OQ-NP4 field set fully specified

### Negative

- `UnitKeywordState` will grow if more persistent states are added in future expansions —
  a refactor milestone should be planned if the struct exceeds ~10 fields
- ADR-006 requires an amendment before any keyword implementation; this is a blocking
  sequencing constraint
- Three new RNG seed slots in ADR-005 add ordering constraints to the RESOLUTION chain

### Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| `bodyguard_protects: Option<Entity>` becomes dangling if BODYGUARD despawns without cleanup | MEDIUM | Silent incorrect protection state | `bodyguard_cleanup_system` in PostUpdate; integration test asserts bond cleared within one frame of BODYGUARD despawn |
| `#[serde(tag = "kw", content = "val")]` round-trip fails for a new `Keyword` variant | LOW | JSON parse error → server startup abort | Mandatory round-trip test for all 7 `Keyword` variants before ADR-006 amendment merges; CI gate |
| `SimpleKeyword::Charge` references not updated after rename | MEDIUM | Runtime deserialization error on old `cards.json` | One-time audit of all fixtures and Rust source at ADR-006 amendment time; resolves OQ-KS2 |
| `app.add_message::<KeywordTriggered>()` not registered in `KeywordPlugin` | LOW | Runtime panic on first `MessageWriter::write()` | Verification criterion in Engine Compatibility table; integration test smoke-checks message delivery |
| 3 RNG seed slots not registered before keyword implementation stories open | MEDIUM | Non-deterministic RANGE target selection breaks audit log invariant (ADR-005) | KW-033b explicitly marked BLOCKED; pre-implementation gate in ADR Dependencies |

## GDD Requirements Addressed

| GDD System | Requirement | How This ADR Addresses It |
|---|---|---|
| `keyword-system.md` Replication Contract | 6 persistent states in `UnitBoardState` (OQ-NP4) | `UnitKeywordState` fields map 1:1: `shield_active`, `stun_active`, `silenced_until_round`, `leader_bonus_atk`, `leader_bonus_hp`, `bodyguard_protects` |
| `keyword-system.md` Edge Case: BODYGUARD bond storage | "MUST be stored as unit-to-unit entity bond, NOT lane-scoped attribute" | `bodyguard_protects: Option<Entity>` on BODYGUARD entity; stable across CHANGE LANE |
| `keyword-system.md` LEADER snapshot semantics | Snapshotted at RESOLUTION entry; persists after LEADER dies in SS4 | `leader_snapshot_system` runs before SS1; fields persist until RESOLUTION ends regardless of LEADER death |
| `keyword-system.md` OUTNUMBERED Formula 3 | Global board count, re-evaluated at each sub-step boundary | `eval_outnumbered_system` called at each sub-step boundary; `outnumbered_active` cached in `UnitKeywordState` |
| `keyword-system.md` OQ-NP1 | `DisplacementEvent { unit_id, attacker_id, keyword, from_cell, to_cell, sub_step, was_blocked }` | `DisplacementEvent` struct + `DisplacementKind` enum in `protocol/src/keyword.rs` |
| `keyword-system.md` OQ-NP5 | `KeywordTriggered { unit_id, keyword: KeywordKind, sub_step, payload: KeywordPayload }` | `KeywordKind` + `KeywordPayload` + `InjuredGrantedKeyword` in `protocol/src/keyword.rs` |
| `keyword-system.md` OQ-KS1 | 3 distinct RNG seed slots in RESOLUTION chain | Formally tracked as pre-implementation gate; must be added to ADR-005 before keyword stories open |
| `keyword-system.md` OQ-KS2 | HASTE rename: `SimpleKeyword::Charge` → `SimpleKeyword::Haste` | `SimpleKeyword::Haste` defined; `Charge` removed; serde tag updated |
| `card-data-pool.md` / ADR-006 | `Keyword`/`SimpleKeyword` covers all ~28 keywords | Extended `SimpleKeyword` (20 variants) + `Keyword` with 7 variants (3 new: `VulnerabilityX`, `RepelX`, `AttractX`); serde changed to adjacent tag |

## Performance Implications

- **CPU:** `UnitKeywordState` is a flat struct (~40 bytes). One component access per unit
  per sub-step at 10 units per player = 20 accesses per sub-step — negligible
- **Memory:** 20 units × ~40 bytes = ~800 bytes of keyword state per session
- **Network:** `KeywordTriggered` emitted only on state transitions (e.g.,
  `OutnumberedFlipped` only when boolean changes). Estimated ≤ 20 keyword events per
  round × ~24 bytes = ≤ 480 bytes/round — within the 1 KB/round budget (ADR-008)
- **Load Time:** `SimpleKeyword` enum extension (+12 variants) adds negligible compile
  time

## Migration Plan

Greenfield — no keyword implementation exists. Sequence:

1. **ADR-006 amendment:** Extend `SimpleKeyword` to 20 variants; rename `Charge` →
   `Haste`; add `VulnerabilityX`, `RepelX`, `AttractX`; change serde to
   `#[serde(tag = "kw", content = "val")]`. Write round-trip test for all 7 variants.
2. **ADR-005 seed slots:** Add `range_equidistant_select`, `teleport_random_dest`,
   `strich_change_lane_select` to the RESOLUTION consumption order table. Unblock KW-033b.
3. **`server/feature/keyword/` scaffold:** Create module tree; define `UnitKeywordState`;
   register in `KeywordPlugin` with `app.add_message::<KeywordTriggered>()`.
4. **`protocol/src/keyword.rs`:** Define `KeywordKind`, `KeywordPayload`,
   `InjuredGrantedKeyword`, `DisplacementEvent`, `DisplacementKind`.
5. **State eval systems:** Implement `leader_snapshot_system`, `eval_outnumbered_system`,
   `bodyguard_cleanup_system`.
6. **Effects stubs:** Stub all keyword effect functions in `effects.rs` and `movement.rs`
   with `todo!()` bodies so combat resolution stories can compile against the interface.
7. **Formula tests:** Write tests for `repel_destination()` and `attract_destination()`
   covering KW-029a, KW-029b, KW-030 before any combat resolution story opens.

## Validation Criteria

All BLOCKING acceptance criteria in `keyword-system.md` must pass (KW-001 through KW-041,
excluding KW-033b which is formally BLOCKED pending OQ-KS1 resolution).

Pre-implementation gates:
- [ ] ADR-006 amendment merged; `cards.json` round-trip test passes for all 7 `Keyword`
      variants
- [ ] ADR-005 updated with 3 new RESOLUTION seed slots (`range_equidistant_select`,
      `teleport_random_dest`, `strich_change_lane_select`)
- [ ] `app.add_message::<KeywordTriggered>()` registration confirmed in `KeywordPlugin`
- [ ] `&Entities` system param verified against Bevy 0.18 docs before `bodyguard_cleanup_system` merges

## Related Decisions

- ADR-002: Client-server authority — `UnitKeywordState` is server-only; never in `protocol/`
- ADR-005: Server-side RNG — 3 new seed slots (OQ-KS1) are a pre-implementation gate
- ADR-006: Card data schema — amendment required; this ADR supersedes ADR-006's partial
  `SimpleKeyword` definition
- ADR-009: RSM phase state — LEADER snapshot runs on `ResolutionPhaseEntered`
- ADR-010: RSM event bus — `ResolutionPhaseEntered` triggers keyword system RESOLUTION init
- `design/gdd/keyword-system.md` — primary GDD; all 41 ACs, formulas, replication contract
- `design/gdd/combat-resolution.md` — owns sub-step timing; calls keyword effects functions
- `design/gdd/network-protocol.md` — must be updated with `KeywordTriggered` +
  `DisplacementEvent` variants (OQ-NP1, OQ-NP5)
