# Story 004: Scoreboard Dot Message and State Machine

> **Epic**: HUD
> **Status**: Blocked (OQ-HUD-04)
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
**ADR Decision Summary**: `HudObjectiveUpdate` is a client-internal Bevy `Message` registered with `app.add_message::<HudObjectiveUpdate>()`, not a Lightyear replicated component and not a direct HUD read from `MessageReceiver<ObjectiveDestroyed>`. Board Rendering remains the sole `MessageReceiver<ObjectiveDestroyed>` drain; after draining it writes `HudObjectiveUpdate { target_player_id, lane }` with `MessageWriter`, stripping `was_fake`. `HudPlugin` consumes `MessageReader<HudObjectiveUpdate>` in an explicitly ordered client presentation system so the dot state flips in the same ECS tick as the Board Rendering drain. `HudObjectiveUpdate` is defined in the client crate's presentation/UI shared module, accessible to both Board Rendering and HUD; OQ-HUD-05 is resolved.

**Engine**: Bevy 0.18 | **Risk**: HIGH
**Engine Notes**: `HudObjectiveUpdate` uses `#[derive(Message)]`, `app.add_message::<HudObjectiveUpdate>()`, `MessageWriter<HudObjectiveUpdate>`, and `MessageReader<HudObjectiveUpdate>`. NOT `EventReader<HudObjectiveUpdate>` / `EventWriter<HudObjectiveUpdate>` (removed in Bevy 0.17+). NOT `MessageReceiver<ObjectiveDestroyed>` in HUD (Board Rendering is the sole Lightyear drain). NOT a Lightyear replicated component.

**Control Manifest Rules (Presentation Layer + Feature Layer)**:
- Required: `app.add_message::<HudObjectiveUpdate>()` for the client-internal signal. HUD's `MessageReader<HudObjectiveUpdate>` system MUST check bounds before any array index access (`if !(1..=5).contains(&lane)`). `was_fake` field never reaches HUD (stripped by Board Rendering).
- Forbidden: Never use `EventReader<HudObjectiveUpdate>`. Never use `EventWriter<HudObjectiveUpdate>`. Never use `MessageReceiver<ObjectiveDestroyed>` in HUD. Never read a Lightyear replicated component directly for this scoreboard update.
- Guardrail: Dot state flip must complete within the same ECS tick as the `HudObjectiveUpdate` message write via explicit system ordering — no deferred state.

---

## BLOCKED - Pre-Implementation Gate

**OQ-HUD-05 - RESOLVED (2026-05-01)** - `HudObjectiveUpdate` is a client-internal Bevy `Message`, not a Lightyear replicated component and not a direct HUD read from Lightyear state. The type lives in the client crate's presentation/UI shared module (the shared presentation module imported by both Board Rendering and HudPlugin), not in the workspace `shared/` crate. This keeps the dependency-light protocol/shared crate free of Bevy presentation concerns while giving both client sub-plugins one canonical type.

Registration and flow:
- Register once with `app.add_message::<HudObjectiveUpdate>()` from the client presentation composition layer before Board Rendering writes or HUD reads the message.
- Board Rendering is the sole Lightyear `MessageReceiver<ObjectiveDestroyed>` drain. After it drains `ObjectiveDestroyed`, it writes `HudObjectiveUpdate { target_player_id, lane }` with `MessageWriter`, stripping `was_fake`.
- HudPlugin reads only `MessageReader<HudObjectiveUpdate>` and never reads `ObjectiveDestroyed`, a replicated component, or any `was_fake`/identity-bearing payload.
- The Board Rendering write system and HUD read/apply system must be ordered so the dot state transition occurs in the same ECS tick.

Rationale: ADR-021 separates Lightyear network drains from client-internal presentation signals and warns that Lightyear receivers are single-drain. A Bevy `Message` preserves Board Rendering as the sole network/replication boundary, keeps the HUD read-only, and prevents scoreboard code from depending on identity-bearing network state.

Still blocked by **OQ-HUD-04** for dot horizontal alignment: `LANE_MIDPOINT_X: [f32; 5]` sharing mechanism between Board Rendering and HudPlugin is unresolved. Dot horizontal position cannot be verified until this is resolved. The state machine itself (ALIVE/DESTROYED) can be implemented with OQ-HUD-05 resolved; alignment verification requires OQ-HUD-04.

---

## Acceptance Criteria

*From GDD `design/gdd/hud.md`, scoped to this story — OQ-HUD-05 is resolved; alignment verification remains blocked by OQ-HUD-04:*

- [ ] **HUD-06** (BLOCKING): GIVEN all 10 dots ALIVE, WHEN `HudObjectiveUpdate{target_player_id=opponent, lane=3}` is written, THEN opponent dot index 2 (0-indexed) transitions to DESTROYED; all other 9 dots remain ALIVE; no real/fake identifier applied to any dot.
- [ ] **HUD-07** (BLOCKING): GIVEN HUD initialized and any message/phase sequence processed (including GAME_OVER), WHEN HUD entity subtree inspected, THEN: (a) no `Text`/`TextSpan` content contains `"REAL"`, `"FAKE"`, or any `ObjectiveIdentity` discriminant; (b) no entity carries `ObjectiveIdentity` or equivalent real/fake marker; (c) only valid dot-state flag values are `ALIVE (false)` and `DESTROYED (true)`.
- [ ] **HUD-12b — dot portion** (BLOCKING): `HudObjectiveUpdate` is written → dot visual state reflects new value within the same ECS tick. No `Animator<T>` component on dot entities.
- [ ] **HUD-26** (BLOCKING): GIVEN `destroyed[opponent][2]` already `true`, WHEN `HudObjectiveUpdate{target_player_id=opponent, lane=3}` is written again, THEN dot entity state component has same value as before; no panic, error, or spurious output.
- [ ] **HUD-30** (BLOCKING): GIVEN HUD in any visible mode, WHEN `HudObjectiveUpdate{lane=0}` or `HudObjectiveUpdate{lane=6}` is written, THEN no dot entity state changes, no array index access is performed, no panic occurs, and a warning is logged.

---

## Implementation Notes

*Derived from ADR-021 and ADR-001 Implementation Guidelines, with OQ-HUD-05 resolved:*

- Define `HudObjectiveUpdate` in the client crate's presentation/UI shared module:
  ```rust
  #[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
  pub struct HudObjectiveUpdate {
      pub target_player_id: PlayerId,
      pub lane: u8,
  }
  ```
- Register the message once from the client presentation composition layer: `app.add_message::<HudObjectiveUpdate>()`.
- Board Rendering writes the message after draining `ObjectiveDestroyed`:
  ```rust
  writer.write(HudObjectiveUpdate {
      target_player_id,
      lane,
  });
  ```
- HUD reader/apply system signature:
  ```rust
  fn handle_hud_objective_update_system(
      mut updates: MessageReader<HudObjectiveUpdate>,
      hud_entities: Res<HudEntities>,
      mut dot_states: Query<(&mut DotState, &mut BackgroundColor, &mut BorderColor)>,
      phase: Res<CurrentClientPhase>,
  ) {
      for event in updates.read() {
          if !(1..=5).contains(&event.lane) {
              warn!("HUD: OOB lane {} in HudObjectiveUpdate - ignored", event.lane);
              continue;
          }
          // FROZEN mode: reject after GAME_OVER (Story 007 handles FROZEN gate)
          // ...
          let entity = hud_entities.dots[player_index][event.lane as usize - 1];
          // transition ALIVE -> DESTROYED only (idempotent)
      }
  }
  ```
- Dot state: use a `DotState` component with `enum DotState { Alive, Destroyed }` (or `bool destroyed`).
- DESTROYED visual: `BackgroundColor(Color::NONE)`, `BorderColor` shifted to low-luminance. ALIVE visual: `BackgroundColor` filled, `BorderColor` normal.
- Bounds guard MUST precede any array index access — `lane - 1` on a `u8` of 0 underflows in debug.
- `HudObjectiveUpdate` type location: OQ-HUD-05 resolved to client crate presentation/UI shared module, not workspace `shared/`.
- Dot horizontal alignment (`LANE_MIDPOINT_X`): implement once OQ-HUD-04 resolves. Placeholder: uniform spacing.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- [Story 007]: FROZEN mode gate (GAME_OVER rejects HudObjectiveUpdate)
- [Story 008]: Snapshot rebuild of dot states on reconnect

---

## QA Test Cases

*Written by qa-lead at story creation. OQ-HUD-05 is resolved; tests should use Bevy Message registration/writes.*

**HUD-06**: ALIVE → DESTROYED transition
  - Given: All 10 dots `DotState::Alive`; `HudEntities.dots` populated
  - When: `HudObjectiveUpdate { target_player_id: opponent_id, lane: 3 }` is written through Bevy's `MessageWriter` path in test and `app.update()` runs
  - Then: `dots[0][2]` entity has `DotState::Destroyed`; all other 9 entities have `DotState::Alive`; `BackgroundColor(Color::NONE)` on destroyed dot
  - Edge cases: Lane 1 → `dots[0][0]`; Lane 5 → `dots[0][4]`; local player → `dots[1][lane-1]`

**HUD-07**: Real/fake identity never stored
  - Given: HUD initialized; full phase sequence including GAME_OVER processed
  - When: Query all entities in HUD subtree for `ObjectiveIdentity` component; query all `Text`/`TextSpan` for "REAL" or "FAKE"
  - Then: Zero entities with `ObjectiveIdentity`; zero text matches
  - Edge cases: `HudObjectiveUpdate` with no `was_fake` field (it was stripped) — confirm field is absent from the message type

**HUD-26**: Idempotent on duplicate message
  - Given: `dots[0][2]` already `DotState::Destroyed`; record component value before
  - When: `HudObjectiveUpdate { target_player_id: opponent_id, lane: 3 }` is written through Bevy's `MessageWriter` path
  - Then: `dots[0][2]` component unchanged; no panic; no error output
  - Edge cases: Three consecutive duplicate messages → still no change, no panic

**HUD-30**: OOB lane guard
  - Given: All 10 dots `DotState::Alive`
  - When: `HudObjectiveUpdate { target_player_id: opponent_id, lane: 0 }` is written through Bevy's `MessageWriter` path; then same with `lane: 6`
  - Then: All dot states unchanged; `warn!` logged for each; no panic
  - Edge cases: `lane: 255` (u8 max) → same guard catches it

**HUD-12b (dot portion)**: Same-tick state flip
  - Given: Dot starts `DotState::Alive`
  - When: Board Rendering writes `HudObjectiveUpdate` and the ordered `Update` schedule runs once
  - Then: Dot is `DotState::Destroyed` within the same schedule run; no `Animator<T>` on dot entity

---

## Test Evidence

**Story Type**: Integration
**Required evidence**: `tests/integration/hud/scoreboard_dot_message_test.rs` — must exist and pass

**Status**: [ ] Not yet created — OQ-HUD-05 resolved; story remains blocked on OQ-HUD-04 alignment verification

---

## Dependencies

- Depends on: Story 001 (entity pool, `HudEntities.dots` array); OQ-HUD-05 resolved (client-internal Bevy `Message` in client presentation/UI shared module); OQ-HUD-04 resolved (for alignment verification)
- Unlocks: Story 007 (FROZEN mode gates the dot updates), Story 008 (snapshot rebuilds dot states)
