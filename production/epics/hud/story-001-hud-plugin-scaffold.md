# Story 001: HUD Plugin Scaffold and Pre-Pooled Entity Tree

> **Epic**: HUD
> **Status**: Ready
> **Layer**: Presentation
> **Type**: Logic
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/hud.md`
**Requirement**: `TR-HUD-001`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: [ADR-021: Presentation Layer Architecture](docs/architecture/adr-021-presentation-layer-architecture.md)
**ADR Decision Summary**: `HudPlugin` is the 4th sub-plugin registered in `PresentationPlugin`. All 18 HUD entities are pre-pooled at session entry (`OnEnter(ClientState::InSession)`); only `Visibility` is toggled in steady state. `PickingBehavior` on root `Node` only inside `#[cfg(feature = "ui_picking")]`.

**Engine**: Bevy 0.18 | **Risk**: HIGH
**Engine Notes**: Required Components API — no `NodeBundle`. Use `commands.spawn((Node { .. }, Visibility::Hidden, ..))`. TextSpan children: parent entity carries `Text + TextFont + TextColor + Node`; child entity carries `TextSpan + TextFont + TextColor`, inserted with `commands.entity(child).insert(ChildOf(parent_entity))`. Scoreboard dots: `BorderRadius` is a **field inside `Node`** in Bevy 0.18 — NOT a standalone component. `PickingBehavior` is only registered when `ui_picking` feature is compiled in — inserting without feature panics at runtime. `Color::srgba`/`Color::srgb` — NOT `Color::rgba`.

**Control Manifest Rules (Presentation Layer)**:
- Required: Pre-pool all HUD entities at session start; toggle `Visibility` only — never spawn/despawn in steady state. `PickingBehavior` only inside `#[cfg(feature = "ui_picking")]`.
- Forbidden: Any `*Bundle` type (`NodeBundle`, etc.). Never `Color::rgba()`. Never `commands.entity(e).set_parent(p)` — use `insert(ChildOf(p))`.
- Guardrail: Presentation steady-state < 1 ms/frame; phase-boundary frame < 3 ms spike.

---

## Acceptance Criteria

*From GDD `design/gdd/hud.md`, scoped to this story:*

- [ ] **HUD-01** (BLOCKING): Exactly 18 HUD entities exist in the `World` after `HudPlugin` initialises, before any S2C message is received: 1 phase label, 1 round counter, 1 own gold label parent, 1 own gold `TextSpan` child (text `""`), 1 opponent gold label parent, 1 opponent gold `TextSpan` child (text `""`), 1 mana label, 1 reserve mana label, and 10 scoreboard dot entities. No new HUD entities are spawned when subsequent update messages arrive.
- [ ] **HUD-24** (BLOCKING): HUD root `Node` entity has `Visibility::Hidden` when no `S2CPhaseChanged` has been received (LOBBY state).
- [ ] **HUD-11** (BLOCKING): No `bevy::time::Timer` component on any HUD entity at any time; no `Text` or `TextSpan` content matching the pattern `\d+(ms|s|sec)` appears in any HUD entity.
- [ ] **CI gate** (ADVISORY): `cargo build -p client` without `ui_picking` feature compiles without panic — enforced by CI build matrix, not a unit test.

---

## Implementation Notes

*Derived from ADR-021 Implementation Guidelines:*

- Spawn all 18 entities in a startup system registered on `OnEnter(ClientState::InSession)`. Despawn on `OnExit`.
- All entities start with `Visibility::Hidden` — root `Node` propagates to children via `InheritedVisibility`.
- Store all 18 entity handles in a `HudEntities` resource for O(1) lookup by downstream systems. Include: `root`, `phase_label`, `round_counter`, `own_gold_parent`, `own_gold_span`, `opponent_gold_parent`, `opponent_gold_span`, `mana_label`, `reserve_label`, `dots: [[Entity; 5]; 2]` (index `[0]` = opponent top row, `[1]` = local bottom row).
- Gold label structure: parent = `(Text::new("--g"), TextFont { .. }, TextColor(..), Node { .. })`; child = `(TextSpan::new(""), TextFont { .. }, TextColor(..), ChildOf(parent_entity))`.
- Scoreboard dots: `(Node { border_radius: BorderRadius::all(Val::Px(8.0)), width: Val::Px(16.0), height: Val::Px(16.0), .. }, BackgroundColor(Color::srgba(..)), BorderColor(Color::srgba(..)))`.
- Root `Node` picking guard:
  ```rust
  #[cfg(feature = "ui_picking")]
  commands.entity(root).insert(PickingBehavior { should_block_lower: false, is_hoverable: false });
  ```
- `HudConfig` (client-side only, not `GameConfig`): `hud_margin_px: f32`, `hud_dot_diameter_px: f32`, `hud_tween_duration_ms: u32`.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- [Story 002]: Gold/mana label text population from S2C messages
- [Story 003]: Phase label + round counter text population
- [Story 004]: Dot state machine and Observer registration
- [Story 005]: Phase transition visibility toggling (LOBBY → visible)

---

## QA Test Cases

*Written by qa-lead at story creation.*

**HUD-01**: Exactly 18 HUD entities after init
  - Given: `App::new()` with minimal Bevy plugins + `HudPlugin`, `ClientState::InSession` entered
  - When: Init system runs
  - Then: Query all entities with `HudEntity` marker component → count == 18; query `GoldDisplayState` entities → count == 2; query dot-state marker entities → count == 10
  - Edge cases: Run app for 3 frames — count must remain 18 (no respawning)

**HUD-24**: Root hidden at session start
  - Given: `HudPlugin` initialized, no `S2CPhaseChanged` received
  - When: Query `HudRoot` entity for `Visibility` component
  - Then: `Visibility::Hidden`
  - Edge cases: Receive `S2CGoldUpdate` but no `S2CPhaseChanged` → root still `Visibility::Hidden`

**HUD-11**: No timer data on any HUD entity
  - Given: Plugin initialized; `S2CGoldUpdate` and `S2CPhaseChanged` with `timer_duration_ms=99999` processed
  - When: Query all HUD entities for `bevy::time::Timer`; query all `Text`/`TextSpan` for content matching `\d+(ms|s|sec)`
  - Then: Zero entities with `Timer`; zero text content matches
  - Edge cases: `timer_duration_ms` field present in `S2CPhaseChanged` — value must NOT appear in any HUD text

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/hud/hud_plugin_scaffold_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: None (foundation story — no prior HUD story required). `ClientState::InSession` AppState must exist.
- Unlocks: Stories 002, 003, 004, 005, 006, 007, 008, 009, 010 (all require the entity pool)
