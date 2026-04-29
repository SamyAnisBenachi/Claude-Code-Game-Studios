# Story 006: Network Dispatch Wiring

> **Epic**: Round State Machine
> **Status**: Ready
> **Layer**: Core
> **Type**: Integration
> **Manifest Version**: 2026-04-29

## Context

**GDD**: `design/gdd/round-state-machine.md`
**Requirement**: TR-RSM-09
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-009 (RSM Phase State as ECS Resource), ADR-010 (RSM Phase Event Bus), ADR-008 (Lightyear Channel Config)
**ADR Decision Summary**: Network dispatch lives in `server/src/network/` — NOT in `server/core/rsm/`. A system in `server/src/network/` reads `EventReader<BroadcastPhaseChanged>` and sends `S2CPhaseChanged { phase, round_number, timer_duration_ms }` via `MessageSender<S2CPhaseChanged>` on `ReliableChannel` to `NetworkTarget::All`. This preserves the rule that `server/core/rsm/` does not import Lightyear send code. A resolution safety timeout (ADVISORY) is also wired in this story.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**: `MessageSender<T>` and `NetworkTarget::All` are Lightyear 0.26 API — verify exact type names against `liv-bevy-lightyear` skill before implementation. `ReliableChannel` assignment from ADR-008. `EventReader::read()` not `.iter()`. `liv-bevy-lightyear` skill is MANDATORY on all files in this story. `liv-bevy-018` skill also mandatory.

**Control Manifest Rules (Core layer / Network layer)**:
- Required: Network dispatch system lives in `server/src/network/` — never in `server/src/core/rsm/`.
- Required: `BroadcastPhaseChanged` → `S2CPhaseChanged` send uses `ReliableChannel` (ADR-008).
- Required: `NetworkTarget::All` — phase changes are broadcast to all connected clients.
- Required: Exactly one `S2CPhaseChanged` sent per `BroadcastPhaseChanged` event — no double-send, no missing send.
- Required: `RsmPlugin::build()` wires full system scheduling: `rsm_input_reader → advance_phase → [all subscriber systems via .after(advance_phase)]`; network dispatch system scheduled `.after(advance_phase)`.
- Forbidden: Any Lightyear `MessageSender` usage in `server/src/core/rsm/`.
- CI grep gate: `grep -r "MessageSender" server/src/core/rsm/` must return zero matches.

---

## Acceptance Criteria

- [ ] `server/src/network/rsm_dispatch.rs` (or `server/src/network/mod.rs`) defines `dispatch_phase_changed` system: reads `EventReader<BroadcastPhaseChanged>`; for each event, sends `S2CPhaseChanged { phase: event.phase, round_number: event.round, timer_duration_ms: event.timer_ms }` via `MessageSender<S2CPhaseChanged>` on `ReliableChannel` to `NetworkTarget::All`
- [ ] `dispatch_phase_changed` is registered in the network plugin (or `RsmPlugin`) with `.after(advance_phase)` scheduling constraint
- [ ] `S2CPhaseChanged { phase, round_number, timer_duration_ms }` is defined in `shared/src/protocol.rs` with `#[derive(Serialize, Deserialize, Clone, Debug)]` and registered as a Lightyear message type on `ReliableChannel` (verify registration pattern with `liv-bevy-lightyear`)
- [ ] No Lightyear `MessageSender` usage exists in `server/src/core/rsm/` — the RSM emits `BroadcastPhaseChanged` (a Bevy buffered Event) and the network dispatch system converts it to a Lightyear message
- [ ] Resolution safety timeout (ADVISORY): if `phase == Resolution` for longer than `config.resolution_max_duration_seconds` (default: 60s), `rsm_input_reader` or the timer tick system emits `GameOverEmitted { reason: Draw, loser: None }` and transitions to GAME_OVER; this timeout must never fire in normal play — its presence protects against a Combat Resolution crash or infinite keyword chain
- [ ] `resolution_max_duration_seconds = 60.0` is a `GameConfig` field in `assets/config/game_config.ron` — not hardcoded
- [ ] `RsmPlugin::build()` documents full system scheduling in a comment: `AuctionSystem → CombatResolutionSystem → rsm_input_reader → advance_phase → [subscriber systems] → dispatch_phase_changed`; system ordering constraints are applied in the plugin
- [ ] CI grep gate: `grep -r "MessageSender" server/src/core/rsm/` returns zero matches
- [ ] CI grep gate: `grep -rE "EventWriter::send|\.send\(.*\)" server/src/core/rsm/` returns zero matches (re-verified at integration completion)
- [ ] `cargo check --workspace` clean with zero warnings across all RSM and network files
- [ ] `tests/integration/rsm/rsm_network_dispatch_test.rs` passes tests RSM-26 and RSM-38 (see QA Test Cases)

---

## Implementation Notes

*Derived from ADR-009, ADR-010, ADR-008:*

**Why dispatch lives in `server/src/network/` not `server/src/core/rsm/`:** The RSM (`server/core/`) must not import Lightyear send code. Adding `MessageSender` as a system parameter in `advance_phase` would tie the core RSM to the networking layer — violating the dependency direction rule from ADR-003. The `BroadcastPhaseChanged` Bevy event is the decoupling boundary: RSM emits it, the network layer reads it and converts to a Lightyear message.

**`S2CPhaseChanged` Lightyear message registration:** In Lightyear 0.26, messages are registered on channels. Verify the exact registration call with `liv-bevy-lightyear` — it is likely something like:
```rust
app.add_message::<S2CPhaseChanged>(ChannelDirection::ServerToClient)
   .add_channel::<ReliableChannel>(); // if not already registered
```
The exact API may differ — do NOT guess from pre-0.26 Lightyear examples.

**`MessageSender<S2CPhaseChanged>` usage in dispatch system:**
```rust
fn dispatch_phase_changed(
    mut events: EventReader<BroadcastPhaseChanged>,
    mut sender: MessageSender<S2CPhaseChanged>,  // Lightyear 0.26 — verify type
) {
    for event in events.read() {
        sender.send_to_all(  // or send(NetworkTarget::All) — verify API
            &S2CPhaseChanged {
                phase: event.phase,
                round_number: event.round,
                timer_duration_ms: event.timer_ms,
            }
        );
    }
}
```
The `MessageSender` API, method name, and target syntax must be verified against Lightyear 0.26 docs via `liv-bevy-lightyear` skill.

**Resolution safety timeout implementation:** Add `auction_safety_timer` and `resolution_safety_timer` fields to `RoundState` (they are already declared in the Story 001 scaffold). The timer tick system (Story 003) should tick `resolution_safety_timer` when `phase == Resolution`. When it fires: emit `GameOverEmitted { reason: Draw, loser: None }`. This is ADVISORY — the safety timeout must never fire in normal play, so its test is documented but a deferred deferral is acceptable if a fundamental Bevy-time test harness limitation is encountered (per EPIC.md Definition of Done).

**Full system scheduling in `RsmPlugin`:** The plugin's `build()` function must register all system ordering constraints:
```rust
app.add_systems(Update, (
    rsm_input_reader,
    advance_phase,
    tick_disconnect_timers,
    tick_rsm_timers,
    dispatch_phase_changed,
)
.chain()  // or explicit .before()/.after() constraints
);
// Plus: AuctionSystem and CombatResolutionSystem scheduled .before(rsm_input_reader)
// (those plugins register their own ordering constraints relative to RsmPlugin systems)
```

**`S2CGameOver` broadcast:** The RSM emits `GameOverEmitted`. The Game Session System (Epic 2) subscribes to it and sends `S2CGameOver`. This story does NOT wire `S2CGameOver` — that belongs to Epic 2.

---

## Out of Scope

*Handled by neighbouring stories or other epics — do not implement here:*

- Story 001-005: All prior RSM infrastructure (must be Done before this story)
- Epic 2 (Game Session System): `S2CGameOver` broadcast, session teardown on `GameOverEmitted`
- ADR-011 (reconnect snapshot): `S2CGameSnapshot` delivery to reconnecting clients — Network epic
- Client-side `ClientPhaseView` update system: owned by client-side UI/network epic; reads `S2CPhaseChanged` and updates `ClientPhaseView` resource — out of scope for the server-side RSM epic

---

## QA Test Cases

Integration tests that verify the full `BroadcastPhaseChanged` → `S2CPhaseChanged` pipeline. These tests may require a partial Lightyear app setup to exercise the message send path — consult `liv-bevy-lightyear` for the correct test harness pattern.

- **RSM-26**: Network dispatch sends exactly one `S2CPhaseChanged` per `BroadcastPhaseChanged` event, on `ReliableChannel`, to `NetworkTarget::All`; no `S2CPhaseChanged` is sent without a preceding `BroadcastPhaseChanged` in the same frame; verify by injecting a `BroadcastPhaseChanged` event and asserting the outbound Lightyear message queue contains exactly one `S2CPhaseChanged` with matching fields

- **RSM-38** (ADVISORY): Resolution safety timeout — synthetic test where `ResolutionComplete` is never written; advance simulated time past `config.resolution_max_duration_seconds` (60s); assert `GameOverEmitted { reason: Draw, loser: None }` is emitted; assert `BroadcastPhaseChanged { phase: GameOver }` follows; assert `rsm.phase == GameOver`; this test is ADVISORY — if a fundamental Bevy-time test harness limitation prevents advancing `Time` in a headless World, document the deferral in `tests/evidence/rsm-story-006-rsm38-deferral.md` with the specific limitation

**Additional integration checks:**
- Phase changed C2S reject (phase-gate pattern): inject a `C2SSubmitPlacement` while `phase == DraftShop`; assert it is silently discarded (no state change, no `S2CPhaseChanged` triggered); this verifies ADR-009 phase-gate invariant across the full pipeline
- `RsmPlugin` clean startup: a headless Bevy `App` with only `RsmPlugin` added starts without panic; all event types registered; resource initialized; system graph has no ordering conflicts

---

## Test Evidence

**Story Type**: Integration
**Required evidence**: Integration tests — `tests/integration/rsm/rsm_network_dispatch_test.rs` must pass; paste `cargo test -p server rsm_network_dispatch` output into `tests/evidence/rsm-story-006-tests.md`; for RSM-38 if deferred, paste deferral explanation into `tests/evidence/rsm-story-006-rsm38-deferral.md`
**Gate Level**: BLOCKING for RSM-26; ADVISORY for RSM-38
**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 005 (disconnect handling) must be Done — full RSM system chain must be complete before network dispatch integration can be validated end-to-end
- Unlocks: Epic Definition of Done — all 38 RSM acceptance criteria covered; downstream epics (Economy System, Card Data & Pool, Board/Lane System) can begin implementing their `DraftStarted` / `ShopRefreshNeeded` / `PlacementPhaseEntered` / `ResolutionPhaseEntered` subscribers
