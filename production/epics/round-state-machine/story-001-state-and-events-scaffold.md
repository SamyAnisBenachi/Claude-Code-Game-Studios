# Story 001: State and Events Scaffold

> **Epic**: Round State Machine
> **Status**: Ready
> **Layer**: Core
> **Type**: Config/Data
> **Manifest Version**: 2026-04-29

## Context

**GDD**: `design/gdd/round-state-machine.md`
**Requirement**: TR-RSM-01, TR-RSM-02, TR-RSM-03, TR-RSM-04, TR-RSM-05, TR-RSM-06
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-009 (RSM Phase State as ECS Resource), ADR-010 (RSM Phase Event Bus)
**ADR Decision Summary**: `RoundState` is a plain `Resource` (NOT `#[derive(States)]`); `RoundPhase` is a plain enum stored inside `RoundState`, never a Bevy `State`. All outbound RSM message types derive `Message`, `Clone`, `Debug`. Messages are registered via `app.add_message::<T>()` in `RsmPlugin`. `SessionReady` is an Observer Event (`#[derive(Event)]`) registered via `app.observe(on_session_ready)` — NOT via `add_message`. Re-exported through `server/core/rsm/mod.rs` so feature systems import from `server::core::rsm::events::*`.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**: RSM phase messages use `#[derive(Message)]` + `MessageWriter<T>`/`MessageReader<T>` + `app.add_message::<T>()`. `EventWriter`/`EventReader`/`Events<T>` do NOT exist in Bevy 0.17+. `SessionReady` is the sole exception — it uses `#[derive(Event)]` + Observer. `#[derive(States)]` must NOT be applied to `RoundPhase` — Bevy States' `OnEnter`/`OnExit` schedules conflict with Lightyear session lifecycle (ADR-009 Alternative 1 rejected). `#[derive(Resource)]` pattern is stable but must be verified against 0.18. `liv-bevy-018` skill mandatory on all files in this story.

**Control Manifest Rules (Core layer)**:
- Required: `RoundPhase` is a plain enum with no `#[derive(States)]`. Only `#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]`.
- Required: `RoundState` derives `Resource` (not `Component`, not `States`).
- Required: All RSM phase message types use `#[derive(Message)]` and `MessageWriter::write()`. `SessionReady` uses `#[derive(Event)]` (Observer only).
- Required: Register messages via `app.add_message::<T>()`, NOT `app.add_event::<T>()`.
- Forbidden: No `#[derive(States)]` on `RoundPhase` anywhere in `server/src/core/rsm/`.
- Forbidden: No `EventWriter`, `EventReader`, or `Events<T>` usage anywhere in `server/src/core/rsm/`.
- Guardrail: Re-export all types through `server/core/rsm/mod.rs` — feature systems must not import from internal paths.

---

## Acceptance Criteria

- [ ] `server/src/core/rsm/state.rs` defines `RoundPhase` as a plain enum with exactly 7 variants: `Lobby`, `DraftInitial`, `DraftAuction`, `DraftShop`, `Placement`, `Resolution`, `GameOver`; derives `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash` only — no `#[derive(States)]`
- [ ] `RoundState` struct derives `Resource` and contains: `phase: RoundPhase`, `round_number: u32`, `placement_timer: Option<Timer>`, `draft_shop_timer: Option<Timer>`, `draft_initial_timer: Option<Timer>`, `auction_safety_timer: Option<Timer>`, `resolution_safety_timer: Option<Timer>`, `submissions_received: HashSet<PlayerId>`, `disconnect_trackers: HashMap<PlayerId, f32>`
- [ ] `RoundState::default()` or `RoundState::new()` initialises `phase = RoundPhase::Lobby`, `round_number = 0`, all `Option<Timer>` fields as `None`, both collections empty
- [ ] `shared/src/protocol.rs` (or `shared/src/lib.rs`) defines `DraftPhase` enum with variants `Initial`, `Auction`, `Shop`; derives `Serialize`, `Deserialize`, `Clone`, `Debug`, `PartialEq`, `Eq`
- [ ] `shared/src/protocol.rs` defines `GameOverReason` enum with variants `ObjectivesDestroyed`, `Disconnection`, `Draw`; derives `Serialize`, `Deserialize`, `Clone`, `Debug`, `PartialEq`, `Eq`
- [ ] `client/src/state/mod.rs` (or `client/src/rsm/view.rs`) defines `ClientPhaseView` resource with fields: `phase: RoundPhase`, `round_number: u32`, `timer_duration_ms: u32`; derives `Resource`, `Default`; doc comment states "Updated only by S2CPhaseChanged handler — never drives server transitions"
- [ ] `server/src/core/rsm/events.rs` defines all 7 outbound message types: `DraftStarted { round: u32, phase: DraftPhase }`, `ShopRefreshNeeded { player: PlayerId }`, `AuctionPhaseEntered { round: u32 }`, `PlacementPhaseEntered { round: u32 }`, `ResolutionPhaseEntered { round: u32 }`, `GameOverEmitted { reason: GameOverReason, loser: Option<PlayerId> }`, `BroadcastPhaseChanged { phase: RoundPhase, round: u32, timer_ms: u32 }` — all derive `Message`, `Clone`, `Debug`
- [ ] `server/src/core/rsm/events.rs` defines: `SessionReady` (marker struct — `#[derive(Event)]` — doc comment states "DELIVERY: Observer trigger per ADR-012, NOT a buffered Message — do not read via MessageReader; subscribe via app.observe(on_session_ready)"), `AuctionSettled { winner: Option<PlayerId>, final_price: u32, card_id: CardId }` (`#[derive(Message, Clone, Debug)]`), `ResolutionComplete` (marker struct — `#[derive(Message, Clone, Debug)]`)
- [ ] `server/src/core/rsm/plugin.rs` defines `RsmPlugin` struct implementing Bevy `Plugin`; `build()` registers all 9 message types (all except `SessionReady`) via `app.add_message::<T>()`; inserts `RoundState` resource via `app.insert_resource(RoundState::new())`; `SessionReady` is registered via `app.observe(on_session_ready)` — NOT via `app.add_message::<SessionReady>()`
- [ ] `server/src/core/rsm/mod.rs` re-exports: `pub use events::*;`, `pub use state::{RoundPhase, RoundState};`, `pub use plugin::RsmPlugin;`
- [ ] CI grep gate: `grep -rE "EventWriter|EventReader|Events<|add_event" server/src/core/rsm/` returns zero matches
- [ ] CI grep gate: `grep -r "derive(States)" server/src/core/rsm/` returns zero matches
- [ ] `cargo check --workspace` clean with zero warnings on `server/src/core/rsm/**`
- [ ] `tests/unit/rsm/rsm_scaffold_test.rs` passes: `World::new()` + `app.init_resource::<RoundState>()` succeeds without panic; `world.resource::<RoundState>().phase == RoundPhase::Lobby`; `world.resource::<RoundState>().round_number == 0`; all 9 message types (excluding `SessionReady`) can be added via `app.add_message::<T>()` and read back from the world without panic

---

## Implementation Notes

*Derived from ADR-009 and ADR-010:*

**Why `RoundPhase` is NOT `#[derive(States)]`:** Bevy States' `OnEnter`/`OnExit` schedules fire based on Bevy's own state transition system, which is incompatible with Lightyear's session lifecycle. Session start (`SessionReady` Observer) and session end (`GameOverEmitted`) must be controlled by the RSM at precisely defined points — not by Bevy's state machinery. ADR-009 Alternative 1 was explicitly rejected for this reason.

**`SessionReady` registration:** `app.observe(on_session_ready)` not `app.add_event::<SessionReady>()`. The function `on_session_ready` is a stub in this story (it can be `fn on_session_ready(_: Trigger<SessionReady>) {}`) — the real implementation is in Story 003. The important thing is that the Observer is registered in `RsmPlugin::build()` so the plugin compiles and the system graph is correct.

**`DraftPhase` and `GameOverReason` location:** These belong in `shared/src/protocol.rs` per ADR-010, because they cross the client/server boundary as fields on Lightyear messages (`S2CPhaseChanged`, `S2CGameOver`). They must derive `Serialize` and `Deserialize` in addition to the standard derives.

**`PlayerId` and `CardId` types:** Imported from `shared::protocol`. Do not redefine in `server/core/rsm/`. If `PlayerId` or `CardId` are not yet defined in `shared/src/protocol.rs`, add stub newtypes there as part of this story's scope — they are required for the event type signatures to compile.

**`RoundState::new()` vs `Default`:** Prefer a named constructor `RoundState::new()` over `Default` so the initialization intent is explicit. If `Default` is used, document which fields deviate from Rust's default (e.g., `round_number: 0` is fine, but `phase: RoundPhase::Lobby` requires an explicit impl since `RoundPhase` has no natural default).

**Event re-export discipline:** Feature systems (Economy, Card Pool, Board/Lane) import event types as `use server::core::rsm::events::*`. They must never use internal paths like `server::core::rsm::events::DraftStarted` directly — only the re-exported path. This is enforced by re-exporting everything through `mod.rs`.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- Story 002: `advance_phase` transition logic and F2 emission ordering
- Story 003: Timer tick systems, `rsm_input_reader`, `on_session_ready` real implementation
- Story 004: Win condition evaluation, GAME_OVER path
- Story 005: Disconnect tracking systems
- Story 006: Network dispatch wiring (`BroadcastPhaseChanged` → `S2CPhaseChanged` send)
- `ClientPhaseView` update system in `client/src/` — client-side RSM wiring is out of scope for the Core RSM epic; it is owned by the client-side UI/network epic

---

## QA Test Cases

- **AC: Resource inserts cleanly into World**
  - Given: A fresh `World::new()` with `RsmPlugin` applied
  - When: `world.resource::<RoundState>()` is called
  - Then: Returns without panic; `phase == RoundPhase::Lobby`; `round_number == 0`; all timers `None`; both collections empty

- **AC: All message types compile and register**
  - Given: A fresh `App::new()` with `RsmPlugin` added
  - When: `app.finish()` and `app.cleanup()` are called
  - Then: No panic; `Messages<DraftStarted>`, `Messages<ShopRefreshNeeded>`, `Messages<AuctionPhaseEntered>`, `Messages<PlacementPhaseEntered>`, `Messages<ResolutionPhaseEntered>`, `Messages<GameOverEmitted>`, `Messages<BroadcastPhaseChanged>`, `Messages<AuctionSettled>`, `Messages<ResolutionComplete>` all exist as resources in the world

- **AC: No EventWriter/EventReader usage in rsm module**
  - Given: CI grep on `server/src/core/rsm/`
  - When: `grep -rE "EventWriter|EventReader|Events<|add_event"` runs
  - Then: Zero matches

- **AC: No `derive(States)` on RoundPhase**
  - Given: CI grep on `server/src/core/rsm/`
  - When: `grep -r "derive(States)"` runs
  - Then: Zero matches

---

## Test Evidence

**Story Type**: Config/Data
**Required evidence**: Smoke check — `cargo check --workspace` output showing zero warnings on `server/src/core/rsm/**` — paste into `tests/evidence/rsm-story-001-check.md`
**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: workspace-and-shared-types Story 004 (protocol skeleton + CI gates) must be Done — `PlayerId`, `CardId`, and the shared protocol types must exist before `server/src/core/rsm/events.rs` can compile
- Unlocks: Story 002 (advance_phase and F2 ordering)
