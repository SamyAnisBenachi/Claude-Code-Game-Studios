# Epic: Round State Machine

> **Layer**: Core
> **GDD**: design/gdd/round-state-machine.md
> **Architecture Module**: `server/core/rsm/` (full module — `state.rs`, `events.rs`, `transitions.rs`, `system.rs`, `plugin.rs`)
> **Status**: Ready
> **Stories**: 6 stories created — see Stories table below

## Overview

Implements the server-authoritative phase orchestrator for Lanes and Lies. This epic owns the `RoundState` resource (single source of truth for phase, round number, all timers, `submissions_received`, and `disconnect_trackers`), the `advance_phase` transition function (the sole writer of `ResMut<RoundState>`), and the full RSM event bus catalog (six outbound phase Messages plus three inbound completion signals). The RSM is the heart of the server tick: every other Core and Feature system gates on `Res<RoundState>.phase`, every C2S handler runs the phase-gate pattern through it, and every DRAFT entry fires the F2 emission sequence (mana → income → shop → optional auction → broadcast) in strict linear code order. The RSM has zero direct imports from `server/feature/` — it communicates exclusively through Bevy buffered Events. This epic delivers all 38 RSM acceptance criteria from the GDD, the GAME_OVER detection path including Draw cases, the safety timeout for stalled RESOLUTION, and the `S2CPhaseChanged` broadcast on `ReliableChannel` after every transition. It does NOT implement individual subscriber systems (Economy, Card Pool, Board/Lane) — those are owned by their respective epics; this epic delivers the events those subscribers will read.

## Governing ADRs

| ADR | Decision Summary | Engine Risk |
|-----|-----------------|-------------|
| ADR-009: RSM Phase State as ECS Resource | `RoundState` is a plain `Resource` (NOT `#[derive(States)]`); single writer (`advance_phase`); phase-gate pattern in every C2S handler; client holds read-only `ClientPhaseView` mirror | HIGH |
| ADR-010: RSM Phase Event Bus | All outbound phase signals are Bevy buffered `Message` types (`#[derive(Message)]`); `MessageWriter::write()` in `advance_phase`; F2 emission ordering enforced by linear code order in `advance_phase` match arms; `BroadcastPhaseChanged` always written last | HIGH |

## Engine Risk: HIGH

Three post-cutoff APIs converge in this epic:

1. **`MessageWriter::write()` — `EventWriter`/`EventReader` no longer exist in Bevy 0.17+.** RSM phase signals use `#[derive(Message)]` + `MessageWriter<T>` + `MessageReader<T>` + `app.add_message::<T>()`. `liv-bevy-018` skill is mandatory to enforce the correct API.
2. **No `#[derive(States)]` for `RoundPhase`** — Bevy States' `OnEnter`/`OnExit` schedules conflict with Lightyear session lifecycle (ADR-009 Alternative 1 rejected). `RoundPhase` is a plain enum stored inside the `RoundState` resource.
3. **Buffered Message vs Observer split** — Bevy 0.17 formalised the split. RSM phase signals are buffered Messages (`MessageReader::read()`); Observers are one-shot lifecycle triggers. `SessionReady` uses Observer (Epic 2 / ADR-012); all recurring phase messages use `#[derive(Message)]`.

`liv-bevy-018` skill is mandatory on every `.rs` file in this epic. Any networking touch (Lightyear broadcast of `S2CPhaseChanged`, `S2CGameOver`) additionally triggers `liv-bevy-lightyear`.

## GDD Requirements

> Note: `docs/architecture/tr-registry.yaml` has not yet been populated. TR-IDs below are informal references from the ADR "GDD Requirements Addressed" sections. Run `/architecture-review` to register stable IDs before stories are written.

| Informal TR-ID | Requirement | ADR Coverage |
|----------------|-------------|--------------|
| TR-RSM-01 | Phase enum drives all phase-gated behaviour; RSM is sole writer | ADR-009 ✅ |
| TR-RSM-02 | Round counter increments at RESOLUTION → DRAFT transition; never reaches 0 | ADR-009 ✅ |
| TR-RSM-03 | Timers (`placement_timer`, `draft_shop_timer`, `draft_initial_timer`, safety timers) live on `RoundState` | ADR-009 ✅ |
| TR-RSM-04 | RSM emits economy signals at phase entry | ADR-010 ✅ (`DraftStarted`) |
| TR-RSM-05 | RSM emits shop refresh at phase entry | ADR-010 ✅ (`ShopRefreshNeeded`) |
| TR-RSM-06 | F2 emission ordering: mana → income → shop → auction → broadcast (always last) | ADR-010 ✅ (linear code order in match arms) |
| TR-RSM-07 | Phase-gate pattern in every C2S handler | ADR-009 ✅ |
| TR-RSM-08 | Win condition (≥2 real objectives destroyed) evaluated at RESOLUTION end | ADR-009, ADR-010 ✅ (RSM reads `Res<ObjectiveCounters>`) |
| TR-RSM-09 | `S2CPhaseChanged` broadcast on `ReliableChannel` after every transition | ADR-010 ✅ (`BroadcastPhaseChanged` event drives network dispatch) |
| TR-RSM-10 | Disconnect grace timer (30s) handled per Rule 13; mutual disconnection = Draw | ADR-009 ✅ |

## Scope

### Deliverables

**`server/src/core/rsm/state.rs`**
- `RoundPhase` enum: `Lobby | DraftInitial | DraftAuction | DraftShop | Placement | Resolution | GameOver`
- `DraftPhase` enum (in `shared/src/protocol.rs` per ADR-010): `Initial | Auction | Shop` — used as field on `DraftStarted` event
- `GameOverReason` enum (in `shared/src/protocol.rs`): `ObjectivesDestroyed | Disconnection | Draw`
- `RoundState` resource: `phase`, `round_number: u32`, `placement_timer: Option<Timer>`, `draft_shop_timer: Option<Timer>`, `draft_initial_timer: Option<Timer>`, `auction_safety_timer: Option<Timer>`, `resolution_safety_timer: Option<Timer>`, `submissions_received: HashSet<PlayerId>`, `disconnect_trackers: HashMap<PlayerId, f32>`
- `ClientPhaseView` resource (client-side mirror): `phase`, `round_number`, `timer_duration_ms`. Updated only by `S2CPhaseChanged` handler in `client/src/network/`. Read-only — never drives transitions.

**`server/src/core/rsm/events.rs`** (the full ADR-010 catalog)
- Outbound: `DraftStarted { round, phase: DraftPhase }`, `ShopRefreshNeeded { player }`, `AuctionPhaseEntered { round }`, `PlacementPhaseEntered { round }`, `ResolutionPhaseEntered { round }`, `GameOverEmitted { reason, loser: Option<PlayerId> }`, `BroadcastPhaseChanged { phase, round, timer_ms }`
- Inbound buffered messages: `AuctionSettled { winner, final_price, card_id }` [M2], `ResolutionComplete` [M2]
- Observer event (NOT a message): `SessionReady` — delivery via Observer per ADR-012. Do NOT register with `app.add_message::<SessionReady>()`. Do NOT read via `MessageReader<SessionReady>`.
- All outbound and inbound messages derive `Message`, `Clone`, `Debug`. `SessionReady` derives `Event`. All re-exported through `server/core/rsm/mod.rs`.

**`server/src/core/rsm/transitions.rs`**
- `advance_phase` system — the SOLE writer of `ResMut<RoundState>`. Match arm per source phase. Inside each arm, F2 emission order is enforced by linear `MessageWriter::write()` call order. `BroadcastPhaseChanged` is always the last `.write()` call.
- CI grep gate: `grep -r "ResMut<RoundState>" server/src/ | grep -v transitions.rs` must return zero matches.

**`server/src/core/rsm/system.rs`**
- `rsm_input_reader` system: reads `MessageReader<AuctionSettled>` [M2], `MessageReader<ResolutionComplete>` [M2]; updates RSM state with the inbound-message guard pattern (`if rsm_state.phase != expected { continue; }`); schedules `.before(advance_phase)`. `SessionReady` is handled by the RSM Observer (`on_session_ready`), NOT by this system.
- Timer tick system: ticks only the active phase's timer. Resets the relevant timer immediately on phase entry before ticking. Calls `advance_phase` when a timer reaches 0.
- Submission tracking system: handles `MessageReader<C2SSubmitPlacement>` (validates phase via `Res<RoundState>.phase`); updates `submissions_received`; triggers `advance_phase` when set is full.
- Disconnect tracking system: subscribes to Lightyear `OnDisconnected` / `OnConnected`; iterates `disconnect_trackers` in a single pass per tick (RSM-37 mutual-disconnection invariant); fires `GameOverEmitted { reason: Disconnection | Draw, loser: ... }` on threshold breach.
- Win-condition evaluation system: at RESOLUTION end, reads `Res<ObjectiveCounters>` (defined by Epic 4 / Objective epic — this epic forward-declares the contract); routes RSM to GAME_OVER (single loser), GAME_OVER (Draw), or next DRAFT.

**`server/src/core/rsm/plugin.rs`**
- `RsmPlugin`: registers all message types via `app.add_message::<T>()`; registers `SessionReady` Observer via `app.observe(on_session_ready)`; inserts `RoundState` resource; wires system scheduling: `rsm_input_reader → advance_phase → [all subscriber systems via .after(advance_phase)]`. Auction System and Combat Resolution System are scheduled `.before(rsm_input_reader)` per RSM GDD Rules 7 and 10.

**Network dispatch wiring**
- A system in `server/src/network/` reads `MessageReader<BroadcastPhaseChanged>` and sends `S2CPhaseChanged { phase, round_number, timer_duration_ms }` via `MessageSender<S2CPhaseChanged>` on `ReliableChannel` to `NetworkTarget::All`. Lives in network crate, NOT in `server/core/rsm/` (preserves the rule that `core/` does not import Lightyear send code).
- `S2CGameOver { loser, round, reason }` send is wired to `MessageReader<GameOverEmitted>` in the Game Session System teardown subscriber (Epic 2). This epic only writes the message; Epic 2 owns the broadcast.

**Tests**
- `tests/unit/rsm/` — all 38 RSM acceptance criteria (RSM-1 through RSM-38) testable with `World::new()` + event injection. No live Lightyear session required.
- Targeted tests for double-transition guards (RSM-31, RSM-34): two simultaneous triggers in the same tick produce exactly one transition.
- F2 ordering tests (RSM-32): assert that no subscriber's effect is visible before the prior step's effect.
- Mutual disconnection test (RSM-37): both trackers exceed grace on the same tick → single `GameOverEmitted { reason: Draw, loser: None }`.
- Resolution safety timeout test (RSM-38): synthetic test where `ResolutionComplete` is never written; advances simulated time past `resolution_max_duration_seconds`; asserts `GameOverEmitted { reason: Draw }`.

### Out of Scope (owned by other epics)

- Economy formulas (mana ramp, gold income, interest snapshot): Epic 3 — Economy System, subscribes to `DraftStarted`.
- Shop weighted draw (Formula 2): Epic 4 — Card Data & Pool, subscribes to `ShopRefreshNeeded`.
- Combat sub-step execution: M2 (Combat Resolution epic), subscribes to `ResolutionPhaseEntered`, emits `ResolutionComplete`.
- Auction state machine + 20s timer: M2 (Auction System epic), subscribes to `AuctionPhaseEntered`, emits `AuctionSettled`.
- Lobby state machine, class selection, `SessionReady` emission: Epic 2 — Game Session System.
- Objective destruction counting (`real_objectives_destroyed`): Objective System epic, exposes `Res<ObjectiveCounters>` that this epic reads.
- `S2CGameSnapshot` reconnect delivery (ADR-011): Network epic.

### Implementation Notes

**Resource visibility ordering for `SessionReady`** — When the GSS (Epic 2) inserts `SessionConfig` and `ServerRng` and triggers `SessionReady`, Epic 2's Observer handler is the LOBBY → DRAFT_INITIAL transition. ADR-012 places that handler in `server/core/rsm/system.rs` as `on_session_ready` (a `Trigger<SessionReady>` observer registered via `app.observe(on_session_ready)`). This epic owns `on_session_ready` even though the `SessionReady` event itself is emitted by Epic 2. Inside `on_session_ready`: set `phase = DraftInitial`, `round_number = 1`, then queue `advance_phase` for the same tick (so F2 emission for DRAFT_INITIAL fires from the standard match arm, not duplicated in the observer).

**No `unwrap()` in production paths** — RSM systems use `?` propagation or `expect("diagnostic message")` per coding standards. Timer ticks return `Option<Duration>` — handle the `None` case explicitly.

**Single-tick double-transition prevention (RSM-31, RSM-34)** — Inside `advance_phase`, check `phase != expected_source_phase` at entry and return early if violated. The second of two simultaneous triggers will find the phase already changed and silently no-op.

## Definition of Done

- All deliverables above implemented and passing.
- All 38 RSM acceptance criteria (RSM-1 through RSM-38) have passing unit tests in `tests/unit/rsm/` using `World::new()` + event injection. BLOCKING criteria: 36 tests must pass; ADVISORY criteria (RSM-34, RSM-38): 2 tests should pass but a documented deferral is acceptable if a fundamental Bevy-time test harness limitation is encountered.
- `cargo check --workspace` green; zero warnings on `server/src/core/rsm/**`.
- CI grep gate: `grep -r "ResMut<RoundState>" server/src/ | grep -v transitions.rs` returns zero matches.
- CI grep gate: `grep -r "use server::feature" server/src/core/rsm/` returns zero matches (RSM has zero feature/ imports — ADR-010 invariant).
- CI grep gate: `grep -rE "EventWriter|EventReader|Events<|add_event" server/src/core/rsm/` returns zero matches (`EventWriter`/`EventReader` do not exist in Bevy 0.17+; use `MessageWriter`/`MessageReader`).
- An integration test demonstrates that `BroadcastPhaseChanged` is emitted strictly after `DraftStarted`, `ShopRefreshNeeded`, and `AuctionPhaseEntered` in any DRAFT entry transition (assert via order-recorded mock subscribers).
- An integration test demonstrates that a C2S message arriving in the wrong phase is silently discarded with no S2C response (ADR-009 phase-gate pattern verification).
- `RsmPlugin` registers cleanly in a headless Bevy `App` startup test; resource and event registration succeed without panic.

## Stories

| # | Story | Type | Status | ADR |
|---|-------|------|--------|-----|
| 001 | [State and Events Scaffold](story-001-state-and-events-scaffold.md) | Config/Data | Ready | ADR-009, ADR-010 |
| 002 | [Advance Phase and F2 Ordering](story-002-advance-phase-and-f2-ordering.md) | Logic | Ready | ADR-009, ADR-010 |
| 003 | [Timers and Input Reader](story-003-timers-and-input-reader.md) | Logic | Ready | ADR-009, ADR-010, ADR-012 |
| 004 | [Win Condition and Game Over](story-004-win-condition-and-game-over.md) | Logic | Ready | ADR-009, ADR-010 |
| 005 | [Disconnect Handling](story-005-disconnect-handling.md) | Logic | Ready | ADR-009 |
| 006 | [Network Dispatch Wiring](story-006-network-dispatch-wiring.md) | Integration | Ready | ADR-009, ADR-010, ADR-008 |

Work through stories in order — each story's `Depends on:` field tells you what must be Done before you can start it.

---

## Story Breakdown Hint

Suggested decomposition (final story list to be authored via `/create-stories`):

1. **State + Events scaffold** (Config/Data) — `state.rs` with `RoundState` + `RoundPhase`; `events.rs` with the full ADR-010 catalog; plugin event registration; `World::new()` smoke test that inserts the resource.
2. **`advance_phase` and F2 ordering** (Logic) — `transitions.rs` match arms for all 7 source phases; F2 linear emission order; double-transition guard; tests RSM-1 through RSM-12, RSM-31, RSM-32, RSM-33.
3. **Timers** (Logic) — DRAFT_INITIAL (45s), DRAFT_SHOP (30s), PLACEMENT (10s), auction safety (120s), resolution safety (60s); tests RSM-15 through RSM-19, RSM-30, RSM-38.
4. **Win condition + GAME_OVER** (Logic) — Read `Res<ObjectiveCounters>` (forward-declared); single-loser, mutual-destruction Draw, `S2CGameOver` payload via `GameOverEmitted`; tests RSM-20 through RSM-22, RSM-36.
5. **Disconnect handling** (Logic / Integration) — Subscribe to Lightyear `OnDisconnected`/`OnConnected`; mutual-disconnect single-pass evaluation; mid-RESOLUTION deferral (RSM-35); tests RSM-23 through RSM-25, RSM-35, RSM-37.
6. **Network dispatch wiring** (Integration) — `BroadcastPhaseChanged` → `S2CPhaseChanged` send; `liv-bevy-lightyear` skill mandatory; integration test verifies broadcast on `ReliableChannel` to `NetworkTarget::All`; test RSM-26.

## Next Step

Run `/create-stories production/epics/round-state-machine/EPIC.md` to author the story files, then `/story-readiness` on Story 001 before implementation begins.
