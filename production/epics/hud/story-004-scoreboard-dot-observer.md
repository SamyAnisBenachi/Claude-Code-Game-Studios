# Story 004: Scoreboard Dot Message and State Machine

> **Epic**: HUD
> **Status**: Complete
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
**ADR Decision Summary**: `HudObjectiveUpdate` is a client-internal Bevy `Message` registered with `app.add_message::<HudObjectiveUpdate>()`, not a Lightyear replicated component and not a direct HUD read from `MessageReceiver<ObjectiveDestroyed>`. Board Rendering remains the sole `MessageReceiver<ObjectiveDestroyed>` drain; after draining it writes `HudObjectiveUpdate { target_player_id, lane }` with `MessageWriter`, stripping `was_fake`. `HudPlugin` consumes `MessageReader<HudObjectiveUpdate>` in an explicitly ordered client presentation system so the dot state flips in the same ECS tick as the Board Rendering drain. `HudObjectiveUpdate` is defined in the client crate's presentation/UI shared module, accessible to both Board Rendering and HUD; OQ-HUD-05 is resolved. Scoreboard dot horizontal alignment uses `Res<BoardLayout>` inserted by `BoardRenderingPlugin`; OQ-HUD-04 is resolved without introducing a duplicate `LANE_MIDPOINT_X` constant.

**Engine**: Bevy 0.18 | **Risk**: HIGH
**Engine Notes**: `HudObjectiveUpdate` uses `#[derive(Message)]`, `app.add_message::<HudObjectiveUpdate>()`, `MessageWriter<HudObjectiveUpdate>`, and `MessageReader<HudObjectiveUpdate>`. NOT `EventReader<HudObjectiveUpdate>` / `EventWriter<HudObjectiveUpdate>` (removed in Bevy 0.17+). NOT `MessageReceiver<ObjectiveDestroyed>` in HUD (Board Rendering is the sole Lightyear drain). NOT a Lightyear replicated component.

**Control Manifest Rules (Presentation Layer + Feature Layer)**:
- Required: `app.add_message::<HudObjectiveUpdate>()` for the client-internal signal. HUD's `MessageReader<HudObjectiveUpdate>` system MUST check bounds before any array index access (`if !(1..=5).contains(&lane)`). `was_fake` field never reaches HUD (stripped by Board Rendering).
- Required: scoreboard dot horizontal placement MUST derive from the session-scoped `Res<BoardLayout>` inserted by `BoardRenderingPlugin`. Any lane midpoint/projection helper belongs with the shared presentation `BoardLayout` module so Board Rendering and HUD use one coordinate source.
- Forbidden: Never use `EventReader<HudObjectiveUpdate>`. Never use `EventWriter<HudObjectiveUpdate>`. Never use `MessageReceiver<ObjectiveDestroyed>` in HUD. Never read a Lightyear replicated component directly for this scoreboard update.
- Forbidden: Never define a separate `LANE_MIDPOINT_X` constant, local `[f32; 5]`, or uniform-spacing fallback in HudPlugin for scoreboard alignment.
- Guardrail: Dot state flip must complete within the same ECS tick as the `HudObjectiveUpdate` message write via explicit system ordering — no deferred state.

---

## Resolved Pre-Implementation Gate

**OQ-HUD-05 - RESOLVED (2026-05-01)** - `HudObjectiveUpdate` is a client-internal Bevy `Message`, not a Lightyear replicated component and not a direct HUD read from Lightyear state. The type lives in the client crate's presentation/UI shared module (the shared presentation module imported by both Board Rendering and HudPlugin), not in the workspace `shared/` crate. This keeps the dependency-light protocol/shared crate free of Bevy presentation concerns while giving both client sub-plugins one canonical type.

Registration and flow:
- Register once with `app.add_message::<HudObjectiveUpdate>()` from the client presentation composition layer before Board Rendering writes or HUD reads the message.
- Board Rendering is the sole Lightyear `MessageReceiver<ObjectiveDestroyed>` drain. After it drains `ObjectiveDestroyed`, it writes `HudObjectiveUpdate { target_player_id, lane }` with `MessageWriter`, stripping `was_fake`.
- HudPlugin reads only `MessageReader<HudObjectiveUpdate>` and never reads `ObjectiveDestroyed`, a replicated component, or any `was_fake`/identity-bearing payload.
- The Board Rendering write system and HUD read/apply system must be ordered so the dot state transition occurs in the same ECS tick.

Rationale: ADR-021 separates Lightyear network drains from client-internal presentation signals and warns that Lightyear receivers are single-drain. A Bevy `Message` preserves Board Rendering as the sole network/replication boundary, keeps the HUD read-only, and prevents scoreboard code from depending on identity-bearing network state.

**OQ-HUD-04 - RESOLVED (2026-05-02)** - HudPlugin reads the session-scoped `BoardLayout` resource inserted by `BoardRenderingPlugin` for scoreboard dot horizontal alignment. Board Rendering remains the owner of the lane/cell coordinate model. If HUD needs screen-space pixel positions, the lane midpoint/projection helper is implemented beside `BoardLayout` in the client presentation shared module and reused by HUD; HudPlugin must not define a separate `LANE_MIDPOINT_X: [f32; 5]`, local array, or uniform-spacing fallback.

Rationale: ADR-021 already requires `BoardLayout` to be available to all presentation sub-systems as `Res<BoardLayout>`. Reusing that resource keeps board and HUD alignment under one coordinate source and avoids a second layout token that can drift when Board Rendering changes the board geometry.

---

## Acceptance Criteria

*From GDD `design/gdd/hud.md`, scoped to this story — OQ-HUD-04 and OQ-HUD-05 are resolved:*

- [x] **HUD-02 — dot alignment slice** (BLOCKING for this story): GIVEN `HudPlugin` spawns the 10 scoreboard dot entities and `BoardLayout` is present, WHEN the HUD layout sync runs, THEN each dot's horizontal center is derived from `Res<BoardLayout>` for its lane; HudPlugin defines no duplicate `LANE_MIDPOINT_X` array and has no uniform-spacing fallback.
- [x] **HUD-06** (BLOCKING): GIVEN all 10 dots ALIVE, WHEN `HudObjectiveUpdate{target_player_id=opponent, lane=3}` is written, THEN opponent dot index 2 (0-indexed) transitions to DESTROYED; all other 9 dots remain ALIVE; no real/fake identifier applied to any dot.
- [x] **HUD-07** (BLOCKING): GIVEN HUD initialized and any message/phase sequence processed (including GAME_OVER), WHEN HUD entity subtree inspected, THEN: (a) no `Text`/`TextSpan` content contains `"REAL"`, `"FAKE"`, or any `ObjectiveIdentity` discriminant; (b) no entity carries `ObjectiveIdentity` or equivalent real/fake marker; (c) only valid dot-state flag values are `ALIVE (false)` and `DESTROYED (true)`.
- [x] **HUD-12b — dot portion** (BLOCKING): `HudObjectiveUpdate` is written → dot visual state reflects new value within the same ECS tick. No `Animator<T>` component on dot entities.
- [x] **HUD-26** (BLOCKING): GIVEN `destroyed[opponent][2]` already `true`, WHEN `HudObjectiveUpdate{target_player_id=opponent, lane=3}` is written again, THEN dot entity state component has same value as before; no panic, error, or spurious output.
- [x] **HUD-30** (BLOCKING): GIVEN HUD in any visible mode, WHEN `HudObjectiveUpdate{lane=0}` or `HudObjectiveUpdate{lane=6}` is written, THEN no dot entity state changes, no array index access is performed, no panic occurs, and a warning is logged.

---

## Implementation Notes

*Derived from ADR-021 and ADR-001 Implementation Guidelines, with OQ-HUD-04 and OQ-HUD-05 resolved:*

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
- Dot horizontal alignment: read `Res<BoardLayout>` inserted by `BoardRenderingPlugin` and use the BoardLayout-owned lane midpoint/projection helper for dot horizontal centers. Do not add `LANE_MIDPOINT_X`, a local `[f32; 5]`, or a uniform-spacing fallback to HudPlugin.

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

**HUD-02 (dot alignment slice)**: BoardLayout source of truth
  - Given: `BoardLayout` is inserted by `BoardRenderingPlugin` and HUD scoreboard dots are pre-spawned
  - When: the HUD layout sync system positions the two 5-dot rows
  - Then: every dot horizontal center is derived from the BoardLayout-owned lane midpoint/projection helper for its lane; no HudPlugin-local `LANE_MIDPOINT_X` constant, `[f32; 5]` coordinate table, or uniform-spacing fallback is used
  - Edge cases: Missing `BoardLayout` in session state should be treated as a setup error for the presentation composition layer, not silently replaced with local spacing

---

## Test Evidence

**Story Type**: Integration
**Required evidence**: `tests/integration/hud/scoreboard_dot_message_test.rs` — must exist and pass

**Status**: [x] Created and passing

---

## Dependencies

- Depends on: Story 001 (entity pool, `HudEntities.dots` array); OQ-HUD-05 resolved (client-internal Bevy `Message` in client presentation/UI shared module); OQ-HUD-04 resolved (`BoardLayout` source for dot horizontal alignment)
- Unlocks: Story 007 (FROZEN mode gates the dot updates), Story 008 (snapshot rebuilds dot states)

## Completion Notes

**Completed**: 2026-05-02
**Criteria**: 6/6 passing (HUD-02, HUD-06, HUD-07, HUD-12b dot portion, HUD-26, HUD-30).
**Deviations**: Advisory only - `TR-HUD-004` registry text still describes stale Hidden/Real/Fake/Destroyed wording while the current GDD and story require only ALIVE/DESTROYED. Advisory only - `BoardRenderingPlugin` / the real `ObjectiveDestroyed` drain is not present in `client/src` yet, so cross-plugin fanout is verified through the ordered Bevy `MessageWriter<HudObjectiveUpdate>` integration harness.
**Test Evidence**: Integration test file at `tests/integration/hud/scoreboard_dot_message_test.rs`; `cargo test -p client --test scoreboard_dot_message_test` passed 4/4. `cargo check -p client` passed.
**Code Review**: Skipped - Lean mode.
