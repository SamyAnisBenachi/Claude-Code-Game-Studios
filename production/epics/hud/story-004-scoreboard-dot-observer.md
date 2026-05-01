# Story 004: Scoreboard Dot Observer and State Machine

> **Epic**: HUD
> **Status**: Blocked
> **Layer**: Presentation
> **Type**: Integration
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/hud.md`
**Requirement**: `TR-HUD-004`, `TR-HUD-005`, `TR-HUD-008`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

> **Note on TR-HUD-004**: The registry description ("Hidden → Real → Fake → Destroyed") is stale.
> The GDD specifies exactly two dot states: **ALIVE** and **DESTROYED**. No real/fake identity
> is ever surfaced on the scoreboard. Implement per the GDD (ALIVE/DESTROYED), not the registry text.

**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](docs/architecture/adr-021-presentation-layer-architecture.md), [ADR-001: Objective Identity Unicast](docs/architecture/adr-001-objective-identity-unicast.md)
**ADR Decision Summary**: `HudPlugin` registers a Bevy Observer via `app.observe(handle_hud_objective_update)`. Board Rendering is the sole `MessageReceiver<ObjectiveDestroyed>` drain; after draining it triggers `HudObjectiveUpdate { target_player_id, lane }` via `commands.trigger(..)`, stripping `was_fake`. Observer guarantees same-frame delivery with no 2-frame event buffer window. `HudObjectiveUpdate` type crosses plugin boundaries — crate location is OQ-HUD-05 (open).

**Engine**: Bevy 0.18 | **Risk**: HIGH
**Engine Notes**: `app.observe(handle_hud_objective_update)` — Observer handler signature `fn(trigger: Trigger<HudObjectiveUpdate>, ...)`. NOT `EventReader<HudObjectiveUpdate>` (removed in Bevy 0.17+). NOT `MessageReader<ObjectiveDestroyed>` (Board Rendering is the sole drain). `Trigger<T>` is the correct handler param — NOT `On<T>`.

**Control Manifest Rules (Presentation Layer + Feature Layer)**:
- Required: `app.observe(handler)` for `HudObjectiveUpdate`. Observer handler MUST check bounds as its first operation (`if !(1..=5).contains(&lane)`). `was_fake` field never reaches HUD (stripped by Board Rendering).
- Forbidden: Never use `EventReader<HudObjectiveUpdate>`. Never use `MessageReader<ObjectiveDestroyed>`. Never use `On<T>` as observer handler parameter.
- Guardrail: Dot state flip must complete within the same ECS tick as the trigger — no deferred state.

---

## BLOCKED — Pre-Implementation Gate

**OQ-HUD-05** — `HudObjectiveUpdate` trigger type crate location is unresolved. The type crosses plugin boundaries: Board Rendering calls `commands.trigger(HudObjectiveUpdate{..})` and `HudPlugin` calls `app.observe(handle_hud_objective_update)`. Both crates must import the same type. Until the crate location is decided (shared crate, Board Rendering crate, or HudPlugin crate), neither the Observer registration nor any test can be compiled.

**Do not begin implementation until OQ-HUD-05 is resolved and this story's status is changed to Ready.**

Also blocked by **OQ-HUD-04** for dot horizontal alignment: `LANE_MIDPOINT_X: [f32; 5]` sharing mechanism between Board Rendering and HudPlugin is unresolved. Dot horizontal position cannot be verified until this is resolved. The state machine itself (ALIVE/DESTROYED) can be implemented once OQ-HUD-05 resolves; alignment verification requires OQ-HUD-04.

---

## Acceptance Criteria

*From GDD `design/gdd/hud.md`, scoped to this story — testable once OQ-HUD-05 resolves:*

- [ ] **HUD-06** (BLOCKING): GIVEN all 10 dots ALIVE, WHEN `HudObjectiveUpdate{target_player_id=opponent, lane=3}` fires, THEN opponent dot index 2 (0-indexed) transitions to DESTROYED; all other 9 dots remain ALIVE; no real/fake identifier applied to any dot.
- [ ] **HUD-07** (BLOCKING): GIVEN HUD initialized and any message/phase sequence processed (including GAME_OVER), WHEN HUD entity subtree inspected, THEN: (a) no `Text`/`TextSpan` content contains `"REAL"`, `"FAKE"`, or any `ObjectiveIdentity` discriminant; (b) no entity carries `ObjectiveIdentity` or equivalent real/fake marker; (c) only valid dot-state flag values are `ALIVE (false)` and `DESTROYED (true)`.
- [ ] **HUD-12b — dot portion** (BLOCKING): `HudObjectiveUpdate` fires → dot visual state reflects new value within the same ECS tick. No `Animator<T>` component on dot entities.
- [ ] **HUD-26** (BLOCKING): GIVEN `destroyed[opponent][2]` already `true`, WHEN `HudObjectiveUpdate{target_player_id=opponent, lane=3}` fires again, THEN dot entity state component has same value as before; no panic, error, or spurious output.
- [ ] **HUD-30** (BLOCKING): GIVEN HUD in any visible mode, WHEN `HudObjectiveUpdate{lane=0}` or `HudObjectiveUpdate{lane=6}` fires, THEN no dot entity state changes, no array index access is performed, no panic occurs, and a warning is logged.

---

## Implementation Notes

*Derived from ADR-021 and ADR-001 Implementation Guidelines:*

- Register Observer in `HudPlugin::build()`: `app.observe(handle_hud_objective_update)`.
- Observer handler signature:
  ```rust
  fn handle_hud_objective_update(
      trigger: Trigger<HudObjectiveUpdate>,
      hud_entities: Res<HudEntities>,
      mut dot_states: Query<(&mut DotState, &mut BackgroundColor, &mut BorderColor)>,
      phase: Res<CurrentClientPhase>,
  ) {
      let event = trigger.event();
      if !(1..=5).contains(&event.lane) {
          warn!("HUD: OOB lane {} in HudObjectiveUpdate — ignored", event.lane);
          return;
      }
      // FROZEN mode: reject after GAME_OVER (Story 007 handles FROZEN gate)
      // ...
      let entity = hud_entities.dots[player_index][event.lane as usize - 1];
      // transition ALIVE → DESTROYED only (idempotent)
  }
  ```
- Dot state: use a `DotState` component with `enum DotState { Alive, Destroyed }` (or `bool destroyed`).
- DESTROYED visual: `BackgroundColor(Color::NONE)`, `BorderColor` shifted to low-luminance. ALIVE visual: `BackgroundColor` filled, `BorderColor` normal.
- Bounds guard MUST precede any array index access — `lane - 1` on a `u8` of 0 underflows in debug.
- `HudObjectiveUpdate` type location: follow whatever decision is made for OQ-HUD-05.
- Dot horizontal alignment (`LANE_MIDPOINT_X`): implement once OQ-HUD-04 resolves. Placeholder: uniform spacing.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- [Story 007]: FROZEN mode gate (GAME_OVER rejects HudObjectiveUpdate)
- [Story 008]: Snapshot rebuild of dot states on reconnect

---

## QA Test Cases

*Written by qa-lead at story creation. Test specs ready — blocked until OQ-HUD-05 resolves.*

**HUD-06**: ALIVE → DESTROYED transition
  - Given: All 10 dots `DotState::Alive`; `HudEntities.dots` populated
  - When: `app.trigger(HudObjectiveUpdate { target_player_id: opponent_id, lane: 3 })` in test
  - Then: `dots[0][2]` entity has `DotState::Destroyed`; all other 9 entities have `DotState::Alive`; `BackgroundColor(Color::NONE)` on destroyed dot
  - Edge cases: Lane 1 → `dots[0][0]`; Lane 5 → `dots[0][4]`; local player → `dots[1][lane-1]`

**HUD-07**: Real/fake identity never stored
  - Given: HUD initialized; full phase sequence including GAME_OVER processed
  - When: Query all entities in HUD subtree for `ObjectiveIdentity` component; query all `Text`/`TextSpan` for "REAL" or "FAKE"
  - Then: Zero entities with `ObjectiveIdentity`; zero text matches
  - Edge cases: `HudObjectiveUpdate` with no `was_fake` field (it was stripped) — confirm field is absent from the trigger type

**HUD-26**: Idempotent on duplicate trigger
  - Given: `dots[0][2]` already `DotState::Destroyed`; record component value before
  - When: `app.trigger(HudObjectiveUpdate { target_player_id: opponent_id, lane: 3 })`
  - Then: `dots[0][2]` component unchanged; no panic; no error output
  - Edge cases: Three consecutive duplicate triggers → still no change, no panic

**HUD-30**: OOB lane guard
  - Given: All 10 dots `DotState::Alive`
  - When: `app.trigger(HudObjectiveUpdate { target_player_id: opponent_id, lane: 0 })`; then same with `lane: 6`
  - Then: All dot states unchanged; `warn!` logged for each; no panic
  - Edge cases: `lane: 255` (u8 max) → same guard catches it

**HUD-12b (dot portion)**: Same-tick state flip
  - Given: Dot starts `DotState::Alive`
  - When: Observer triggers and `Update` schedule runs once
  - Then: Dot is `DotState::Destroyed` within the same schedule run; no `Animator<T>` on dot entity

---

## Test Evidence

**Story Type**: Integration
**Required evidence**: `tests/integration/hud/scoreboard_dot_observer_test.rs` — must exist and pass

**Status**: [ ] Not yet created — blocked on OQ-HUD-05

---

## Dependencies

- Depends on: Story 001 (entity pool, `HudEntities.dots` array); OQ-HUD-05 resolved (trigger type crate location); OQ-HUD-04 resolved (for alignment verification)
- Unlocks: Story 007 (FROZEN mode gates the dot updates), Story 008 (snapshot rebuilds dot states)
