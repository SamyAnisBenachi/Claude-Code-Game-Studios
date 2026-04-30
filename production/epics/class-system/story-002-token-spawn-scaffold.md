# Story 002: Token Spawn Scaffold — SourceClass Component

> **Epic**: Class System
> **Status**: Ready
> **Layer**: Feature (M3)
> **Type**: Logic
> **Manifest Version**: 2026-04-30

## Context

**GDD**: `design/gdd/class-system.md`
**Requirement**: `TR-CS-009` (partial — spawn scaffold and snapshot; passive behaviors in Story 010)
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-014: Class System Architecture — PlayerSessionState, SourceClass Component, and Direct Effect Dispatch
**ADR Decision Summary**: All 7 class-specific token types (Mummy, Chacha Noir, Seed, Madoll, La Gonflable, La Sacrifiée, Sinistro) are spawned with a `SourceClass(ClassId)` ECS component set at spawn time, never mutated. A `TokenUnit` marker component is present on all tokens. `UnitBoardState.source_class: Option<ClassId>` is derived from the `SourceClass` component at snapshot build time using a hand-written builder (`world.get::<SourceClass>(entity).map(|sc| sc.0)`); no `#[derive(Reflect)]` required.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: MEDIUM
**Engine Notes**:
- `#[derive(Component)]` on both `SourceClass` and `TokenUnit` — standard Bevy 0.18 component derivation.
- `Reflect` intentionally NOT derived on `SourceClass`: server-only component; no scene serialisation or Bevy inspector usage in the headless server build (ADR-014 §3 explicit opt-out).
- Bevy 0.18 Required Components: spawn tokens using direct component tuples — never `SpriteBundle` or any Bundle pattern (deprecated since 0.15). These are server-side logical entities with no visual representation.
- `commands.spawn((UnitStats { .. }, BoardPosition { .. }, UnitOwner(owner), SourceClass(ClassId::Xelor), TokenUnit))` — the canonical spawn pattern.
- ADR-014 is NOT yet in the control manifest. Apply Feature Layer rules generically; spawn rules from ADR-014 §3 are authoritative.

**Control Manifest Rules (Feature Layer)**:
- Required: Feature systems subscribe to Core phase Messages — ADR-010
- Forbidden: Never spawn ECS entity for a pending placement before `S2CPlacementReveal` is enqueued — ADR-007 (applies to Minion placement; token spawns during RESOLUTION are exempt from placement-buffer ordering but must still use `commands.spawn`)
- Guardrail: Server tick budget ≤ 5ms steady state — ADR-002

---

## Acceptance Criteria

*From GDD `design/gdd/class-system.md`, token registry section and NP-2 resolution:*

- [ ] Each of the 7 token types spawns with a `SourceClass(ClassId::*)` component matching its class: Mummy→Xelor, Chacha Noir→Ecaflip, Seed→Sadida, Madoll→Sadida, La Gonflable→Sadida, La Sacrifiée→Sadida, Sinistro→Xelor.
- [ ] All 7 token types also carry the `TokenUnit` marker component.
- [ ] Standard (non-token) class and neutral card units have NO `SourceClass` component.
- [ ] `UnitBoardState.source_class` is `Some(ClassId::*)` for each token in a built snapshot — value matches the `SourceClass` component at spawn time.
- [ ] `UnitBoardState.source_class` is `None` for standard (non-token) units in the snapshot.
- [ ] A Miranda-stolen token retains its original `SourceClass(ClassId::*)` component — it is never overwritten by the new controller's class. `UnitBoardState.source_class` is unchanged after control transfer.

---

## Implementation Notes

*Derived from ADR-014 Decision §3:*

**Component definitions** — file: `server/src/core/board/components.rs`

```rust
/// Identifies the class that spawned this token entity.
/// Set at spawn time. Never mutated.
/// Absent on non-token units (standard class and neutral cards).
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceClass(pub ClassId);

/// Marker present on all token entities.
/// Used for Sacrifice Poupesque and Miranda filters alongside SourceClass.
#[derive(Component, Default)]
pub struct TokenUnit;
```

**Token spawn functions** — file: `server/src/core/board/spawn.rs`

One function per token type; each hard-codes its `ClassId::*` variant:

```rust
pub fn spawn_mummy(commands: &mut Commands, owner: PlayerId, lane: u8, cell: u8) {
    commands.spawn((
        UnitStats { hp: 2, atk: 2, mp: 3, ar: 0 },
        BoardPosition { lane, cell },
        UnitOwner(owner),
        SourceClass(ClassId::Xelor),
        TokenUnit,
    ));
}

pub fn spawn_chacha_noir(commands: &mut Commands, owner: PlayerId, lane: u8, cell: u8) {
    commands.spawn((
        UnitStats { hp: 2, atk: 2, mp: 6, ar: 0 },
        BoardPosition { lane, cell },
        UnitOwner(owner),
        SourceClass(ClassId::Ecaflip),
        TokenUnit,
    ));
}

pub fn spawn_seed(commands: &mut Commands, owner: PlayerId, lane: u8, cell: u8) {
    commands.spawn((
        SeedMarker,          // cell-attached marker (not a unit — no UnitStats/UnitOwner)
        BoardPosition { lane, cell },
        SeedOwner(owner),
        SourceClass(ClassId::Sadida),
        TokenUnit,
    ));
}

pub fn spawn_madoll(commands: &mut Commands, owner: PlayerId, lane: u8, cell: u8) { ... }
pub fn spawn_la_gonflable(commands: &mut Commands, owner: PlayerId, lane: u8, cell: u8) { ... }
pub fn spawn_la_sacrifiee(commands: &mut Commands, owner: PlayerId, lane: u8, cell: u8) { ... }
pub fn spawn_sinistro(commands: &mut Commands, owner: PlayerId, objective_lane: u8) { ... }
```

Stats per GDD token registry: Madoll HP=3/ATK=1/MP=3; La Gonflable HP=3/ATK=2/MP=3; La Sacrifiée HP=2/ATK=2/MP=3; Sinistro is spell-attached (no UnitStats).

**Snapshot derivation** — in snapshot builder:
```rust
let source_class: Option<ClassId> = world.get::<SourceClass>(unit_entity).map(|sc| sc.0);
// → Some(ClassId::Xelor) for Mummy; None for a standard Iop Minion
```

**Miranda invariant**: `SourceClass` component is never mutated by the Miranda control-transfer system. Miranda changes `UnitOwner`, not `SourceClass`. This invariant should be enforced by making the Miranda handler only write `UnitOwner` — a code review check.

---

## Out of Scope

*Handled by neighbouring stories:*

- Story 001: `PlayerSessions` Resource and class lifecycle — must be DONE before this story
- Story 006: Sang Méprise and Punition behaviors (uses `TokenUnit` marker for Sacrifice filter)
- Story 007: Sadida Seed walk-over and Graines de Folie conversion (uses Seed spawn from this story)
- Story 010: Sinistro/La Gonflable/La Sacrifiée PASSIVE behaviors — spawn functions created here; passive logic implemented there

---

## QA Test Cases

*Logic story — automated test specs using `World::new()`.*

- **Token SourceClass correctness**: For each of the 7 token types
  - Given: empty `World` with `Commands`
  - When: `spawn_[token](commands, owner, lane, cell)` called and commands flushed
  - Then: exactly one entity has `SourceClass(ClassId::[ExpectedClass])` and `TokenUnit`; stats match GDD token registry
  - Run for: spawn_mummy→Xelor, spawn_chacha_noir→Ecaflip, spawn_seed→Sadida, spawn_madoll→Sadida, spawn_la_gonflable→Sadida, spawn_la_sacrifiee→Sadida, spawn_sinistro→Xelor

- **Standard unit has no SourceClass**:
  - Given: a standard Iop Minion spawned via normal placement path (no `SourceClass` inserted)
  - When: `world.get::<SourceClass>(entity)` called
  - Then: returns `None`

- **Snapshot derivation — token**:
  - Given: a Madoll entity with `SourceClass(ClassId::Sadida)` in the world
  - When: `build_unit_board_state(entity, &world)` called
  - Then: `UnitBoardState.source_class == Some(ClassId::Sadida)`

- **Snapshot derivation — standard unit**:
  - Given: a standard unit entity with no `SourceClass` component
  - When: `build_unit_board_state(entity, &world)` called
  - Then: `UnitBoardState.source_class == None`

- **Miranda invariant — SourceClass not overwritten**:
  - Given: a Madoll entity with `SourceClass(ClassId::Sadida)`, `UnitOwner(player_a)`
  - When: Miranda control-transfer writes `UnitOwner(player_b)` to the entity
  - Then: entity still has `SourceClass(ClassId::Sadida)` (unchanged); snapshot `source_class == Some(ClassId::Sadida)`

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/class/token_spawn_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 (PlayerSessions, `class_of()` API) — must be DONE
- Depends on: `workspace-and-shared-types` story (ClassId enum in shared/src/card.rs) — must be DONE
- Unlocks: Story 007 (Sadida Seeds — uses `spawn_seed`, `spawn_madoll`); Story 010 (Token passives — uses all 7 spawn fns + SourceClass queries)
