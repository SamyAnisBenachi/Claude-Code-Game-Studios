# Story 001: HandUiPlugin Scaffold — Pre-Pooled Entity Spawning

> **Epic**: Hand UI
> **Status**: In Progress
> **Layer**: Presentation
> **Type**: Logic
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/hand-ui.md`
**Requirement**: `TR-HU-001`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](../../docs/architecture/adr-021-presentation-layer-architecture.md)
**ADR Decision Summary**: `HandUiPlugin` composes into `PresentationPlugin` as the third sub-plugin (after `CardAnimationsPlugin` and `BoardRenderingPlugin`). All fan slot, grid slot, and drag sprite entities are pre-pooled on `OnEnter(ClientState::InSession)` and despawned on `OnExit`. In steady state, only `Visibility` is toggled — no per-round spawn/despawn.

**Engine**: Bevy 0.18 | **Risk**: HIGH
**Engine Notes**: Required Components API (no `NodeBundle` — spawn `Node { .. }` directly). `ChildOf(parent_entity)` replaces `set_parent()`. `commands.entity(e).despawn()` is recursive by default. `OnEnter`/`OnExit` schedule hooks for session-scoped lifecycle. All systems reading `Res<CardAtlas>` or `Res<BoardLayout>` must be `in_state(ClientState::InSession)`.

**Control Manifest Rules (Presentation Layer)**:
- Required: `PresentationPlugin` registration order — `HandUiPlugin` is 3rd. Reordering causes runtime panics (`Res<CardAtlas>` / `Res<BoardLayout>` not yet inserted).
- Required: Pre-pool all hand fan entities at session start; toggle `Visibility` only — never spawn/despawn mid-session.
- Required: `BoardLayout` and `CardAtlas` are session-scoped resources; systems reading them must be `in_state(ClientState::InSession)`.
- Required: `PickingBehavior` component only inside `#[cfg(feature = "ui_picking")]` guard.
- Forbidden: `NodeBundle`, `SpriteBundle`, any `*Bundle` type.
- Forbidden: `commands.entity(e).set_parent(p)` — use `commands.entity(e).insert(ChildOf(p))`.

---

## Acceptance Criteria

*From GDD `design/gdd/hand-ui.md`, scoped to this story:*

- [ ] **HU-01**: GIVEN the game session starts with 0 cards in hand, WHEN Hand UI initializes (on `OnEnter(ClientState::InSession)`), THEN 10 pre-pooled fan card slot entities and 9 pre-pooled DRAFT_INITIAL grid slot entities exist in the scene (all `Visibility::Hidden`), AND the pre-pooled drag sprite entity exists (`Visibility::Hidden`), with no runtime spawn or despawn occurring during a normal session. (Reconnect rebuild per Story HU-013 may despawn-and-rebuild the drag sprite state, but NOT the fan or grid slots.)

---

## Implementation Notes

*Derived from ADR-021 Implementation Guidelines:*

1. **Plugin registration**: `HandUiPlugin::build()` must be called after `BoardRenderingPlugin::build()` has already run — `Res<CardAtlas>` and `Res<BoardLayout>` are inserted by `BoardRenderingPlugin::build()`. This is guaranteed by the `PresentationPlugin` registration order contract (ADR-021 Guideline 1). Do not call `app.init_resource::<CardAtlas>()` — the resource is NOT available until `OnEnter(ClientState::InSession)`.

2. **Spawn hook**: All 20 entities (10 fan slots + 9 grid slots + 1 drag sprite) are spawned in a system registered on `OnEnter(ClientState::InSession)`. Each entity starts with `Visibility::Hidden`. No entity is spawned in `Update` during normal play.

3. **Despawn hook**: All Hand UI entities are despawned in a system registered on `OnExit(ClientState::InSession)` via `commands.entity(e).despawn()` (recursive by default in Bevy 0.16+; do not use `despawn_recursive()`).

4. **Fan slot marker**: Each fan slot entity should carry a `FanSlotIndex(u8)` component (index 0–9) so systems can query them by index without global entity lookups.

5. **Grid slot marker**: Each DRAFT_INITIAL grid slot entity should carry a `GridSlotIndex(u8)` component (index 0–8).

6. **Drag sprite**: One pre-pooled drag sprite entity; hidden at spawn; toggled visible only during active PLACEMENT drag.

7. **No `PickingBehavior` at spawn unless `ui_picking` feature is active** — gate any `PickingBehavior` insertion inside `#[cfg(feature = "ui_picking")]` (ADR-021 Guideline 4).

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- [Story 002]: Fan layout formula (positions, rotations) applied to spawned slots
- [Story 003]: Phase-driven visibility transitions (HIDDEN/GRID/PASSIVE/STAGING etc.)
- [Story 004]: DRAFT_INITIAL grid display, purchase flow

---

## QA Test Cases

*Written by qa-lead at story creation. The developer implements against these — do not invent new test cases during implementation.*

- **HU-01**: Pre-pooled entity count on session entry
  - Given: `App` with `HandUiPlugin` registered; `ClientState` set to `InSession`
  - When: `App::update()` runs (triggering `OnEnter(ClientState::InSession)`)
  - Then: Query entities with `FanSlotIndex` marker → count == 10; all have `Visibility::Hidden`
  - Then: Query entities with `GridSlotIndex` marker → count == 9; all have `Visibility::Hidden`
  - Then: Query pre-pooled drag sprite entity → count == 1; has `Visibility::Hidden`
  - Edge cases:
    - Record all entity IDs after first session entry; advance through simulated PLACEMENT and DRAFT phases (toggling Visibility only); assert same entity IDs returned — no new `FanSlotIndex` or `GridSlotIndex` entities created
    - Session exit then re-entry: assert old entities despawned; assert new set of 10+9+1 entities spawned

---

## Test Evidence

**Story Type**: Logic
**Required evidence**:
- `tests/unit/hand-ui/plugin_scaffold_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: None — Foundation story; this is the first story in the epic
- Unlocks: Story 002 (fan layout), Story 003 (phase state machine), Story 004 (DRAFT_INITIAL grid)
