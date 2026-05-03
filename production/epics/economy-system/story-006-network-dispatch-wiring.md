# Story 006: Network Dispatch Wiring

> **Epic**: Economy System
> **Status**: Complete
> **Layer**: Core
> **Type**: Integration
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/economy-system.md`
**Requirement**: TR-NP-010 + TR-HUD-007 (client state mirror for gold/mana, public gold visibility, and HUD tie-break: own private economy state comes from `S2CGoldUpdate`; public gold/reservation state comes from `S2CGoldBroadcast`). GDD trace: `design/gdd/economy-system.md` Rule 6 "Gold visibility" states both players can always see each other's current `gold` total; Rule 7 defines `reserved_gold` as server-authoritative auction reservation state used for shop/bid affordability.

**ADR Governing Implementation**: ADR-010: RSM Phase Event Bus + ADR-008: Lightyear Channel Configuration
**ADR Decision Summary (ADR-010)**: Economy emits internal Bevy messages named `server::core::economy::S2CGoldUpdate` (unicast intent) and `server::core::economy::S2CGoldBroadcast` (broadcast intent). A dedicated dispatch system in `server/src/network/` reads those internal messages, converts them to `shared::protocol` S2C payloads, and sends them through Lightyear.
**ADR Decision Summary (ADR-008)**: Both wire messages are sent on `ReliableChannel`. `shared::protocol::S2CGoldUpdate` carries the full private owner state (`gold`, `current_mana`, `reserve_mana`, `mana_cap`) to the owning player only. `shared::protocol::S2CGoldBroadcast` carries `{ player_id: PlayerId, gold: u32, reserved_gold: u32 }` to all players so public gold and auction-reservation UI state stay synchronized. Both are sent on `ReliableChannel` because currency desync is a game-breaking bug; reliability is mandatory.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: MEDIUM
**Engine Notes**: Bevy-side dispatch uses `MessageReader<T>::read()` for internal messages; `EventWriter`/`EventReader` do not exist in Bevy 0.18. Lightyear 0.26 server S2C send uses `ServerMultiMessageSender::send::<Message, ReliableChannel>(&msg, server, &NetworkTarget::Single(peer_id))` for unicast and `NetworkTarget::All` for broadcast; `PeerId` is the connection identifier, not `ClientId` or `ConnectionId`. The dispatch system reads Bevy messages (internal) and calls Lightyear send (external), so BOTH `liv-bevy-018` and `liv-bevy-lightyear` are mandatory.

> **liv-bevy-lightyear skill is MANDATORY on the implementing agent for this story.** Any `.rs` file that imports `lightyear` must activate this skill. Failure to do so risks pre-0.26 Lightyear API patterns that will fail to compile.

**Control Manifest Rules (Core layer)**:
- Required: `S2CGoldUpdate` is unicast — sent ONLY to the player who owns the economy state. Private currency information (current_mana, reserve_mana, mana_cap) must not be broadcast.
- Required: `S2CGoldBroadcast` is broadcast (`NetworkTarget::All`) — gold totals and `reserved_gold` are publicly visible for auction/shop affordance. This is the only Economy message that reaches the opponent.
- Required: Both messages sent on `ReliableChannel` per ADR-008.
- Required: Dispatch system lives in `server/src/network/economy_dispatch.rs` — NOT in `server/src/core/economy/`. Economy core does not import Lightyear; the network module does.
- Forbidden: Economy core (`server/src/core/economy/`) must NOT import `lightyear`. The dispatch system is the only file with Lightyear imports for Economy events.
- Guardrail: If `PeerId` for a player is not found (player disconnected), log a warning and skip the unicast — do not panic. Reconnect/resync is handled by GSS epic.
- Performance: no measurable gameplay-loop impact expected; dispatch work is O(number of queued economy messages), normally at most one update/broadcast per affected player per economy mutation, and uses the existing Lightyear ReliableChannel budget.

---

## Acceptance Criteria

- [ ] `server/src/network/economy_dispatch.rs` exists with two systems:
  - `dispatch_gold_update(mut reader: MessageReader<server::core::economy::S2CGoldUpdate>, sender: ServerMultiMessageSender, server: Query<&Server>, connection_map: Res<PlayerConnectionMap>)` — converts each internal message to `shared::protocol::S2CGoldUpdate` and unicasts it to the owning player's `PeerId` on `ReliableChannel`
  - `dispatch_gold_broadcast(mut reader: MessageReader<server::core::economy::S2CGoldBroadcast>, sender: ServerMultiMessageSender, server: Query<&Server>)` — converts each internal message to `shared::protocol::S2CGoldBroadcast` and broadcasts it to `NetworkTarget::All` on `ReliableChannel`
- [ ] `PlayerConnectionMap` resource is used to resolve `PlayerId` → `PeerId` for unicast; the current map shape is `HashMap<PeerId, PlayerId>`, so reverse lookup is expected
- [ ] If `PeerId` is not found for a player in `dispatch_gold_update`, a `warn!` (not `error!`, not panic) is emitted and the message is skipped
- [ ] Both systems are registered in a new `EconomyNetworkPlugin` in `server/src/network/`; scheduled in `Update` after `on_draft_started` and in a named network dispatch set so future award handlers can run before dispatch in the same frame
- [ ] `shared/src/protocol.rs` already defines `S2CGoldUpdate` and `S2CGoldBroadcast` with `#[derive(Serialize, Deserialize, Debug, Clone)]`; this story must use those wire types without duplicating them
- [ ] Integration test: GIVEN two players initialised, WHEN `DraftStarted { round: 1, phase: Initial }` processed end-to-end (init → on_draft_started → dispatch), THEN `S2CGoldUpdate` unicast fires exactly once per player (player A receives their own; player B receives their own; neither receives the other's private update); `S2CGoldBroadcast` fires twice in total (once per player's gold, broadcast to all)
- [ ] Integration test: GIVEN an economy award path has enqueued `S2CGoldUpdate` and `S2CGoldBroadcast` for player A, WHEN dispatch runs, THEN `S2CGoldUpdate` for player A is sent on `ReliableChannel` to A's `PeerId`; `S2CGoldBroadcast` for player A is sent to all
- [ ] `cargo check --workspace` passes
- [ ] CI channel-type gate: grep for `UnreliableChannel` in `server/src/network/economy_dispatch.rs` returns zero matches — economy messages must not accidentally be sent on the unreliable channel

---

## Implementation Notes

*Derived from EPIC.md §Network dispatch wiring and ADR-008:*

**Lightyear 0.26 unicast pattern (verify against `liv-bevy-lightyear` skill before implementing):**
```rust
// Verified Lightyear 0.26 shape from tests/evidence/lightyear-026-verification.md items 7-9.
fn dispatch_gold_update(
    mut events: MessageReader<economy::S2CGoldUpdate>,
    mut sender: ServerMultiMessageSender,
    server: Query<&Server>,
    connection_map: Res<PlayerConnectionMap>,
) {
    let Ok(server) = server.single() else { return; };
    for event in events.read() {
        match peer_for_player(&connection_map, event.player) {
            Some(peer_id) => {
                let msg = shared::protocol::S2CGoldUpdate {
                    gold: event.gold,
                    current_mana: event.current_mana,
                    reserve_mana: event.reserve_mana,
                    mana_cap: event.mana_cap,
                };
                let _ = sender.send::<shared::protocol::S2CGoldUpdate, ReliableChannel>(
                    &msg,
                    server,
                    &NetworkTarget::Single(peer_id),
                );
            }
            None => {
                warn!("dispatch_gold_update: no PeerId for player {} — skipping unicast", event.player.0);
            }
        }
    }
}
```
The exact send API was verified by the Lightyear 0.26 evidence report: server S2C sends use `ServerMultiMessageSender`; channel selection is the generic `ReliableChannel`; unicast uses `NetworkTarget::Single(peer_id)`.

**`PlayerConnectionMap` resource:** This resource maps Lightyear `PeerId` → `PlayerId` and is maintained by the GSS epic (Game Session System). Economy does not own it. Import it as `Res<PlayerConnectionMap>` and reverse-lookup the `PeerId` for a given `PlayerId`. Do not create a duplicate connection map.

**Why dispatch is separate from economy core:** Economy core (`server/core/economy/`) has no Lightyear dependency. This keeps the economy module independently testable (Stories 001–005 use no Lightyear at all). The dispatch module is the only place Economy-related Lightyear code lives. This matches the ADR-003 workspace structure: core logic is pure, network is a separate module.

**`S2CGoldUpdate` message fields:**
```rust
// shared/src/protocol.rs
#[derive(Serialize, Deserialize, Clone, Debug, /* Lightyear message derive */)]
pub struct S2CGoldUpdate {
    pub gold: u32,
    pub current_mana: u32,
    pub reserve_mana: u32,
    pub mana_cap: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, /* Lightyear message derive */)]
pub struct S2CGoldBroadcast {
    pub player_id: PlayerId,
    pub gold: u32,
    pub reserved_gold: u32,
}
```
These are the current wire message shapes from `shared/src/protocol.rs`. The internal Bevy messages in `server::core::economy` include routing context; the wire `S2CGoldUpdate` omits `player` because the unicast target identifies the owner.

**Integration test scope:** The integration test for this story wires the Economy internal message stack with the dispatch conversion/targeting logic. It does NOT require a live Lightyear WebSocket. Use a dispatch outbox/test adapter if direct inspection of `ServerMultiMessageSender` is not ergonomic in a headless `App`; document the chosen test approach in `tests/evidence/story-eco-006-dispatch-test-approach.md`.

---

## Out of Scope

- `S2CPhaseChanged` dispatch — owned by RSM epic's network dispatch system
- Full reconnect resync — GSS epic re-sends full state including Economy state on reconnect
- Client-side rendering of gold/mana bars — HUD GDD (`design/ux/hud.md`) and UI programmer
- `S2CAuctionBidRejected` — Auction System (M2) network message; not Economy's
- Implementing `handle_kill_award` / `handle_objective_award` — Economy Story 004 owns award mutation and enqueue behavior. This story only dispatches economy messages already enqueued by core Economy systems.

---

## QA Test Cases

*QL-STORY-READY skipped — Lean mode.*

- **Unicast isolation: player A's update does not reach player B**
  - Given: Two-player test app; `S2CGoldUpdate { player: A, gold: 8, .. }` enqueued
  - When: `dispatch_gold_update` runs
  - Then: Player A's message receiver receives the update; Player B's receiver does NOT receive `S2CGoldUpdate` for A

- **Broadcast reaches all players**
  - Given: `S2CGoldBroadcast { player_id: A, gold: 8, reserved_gold: 2 }` enqueued
  - When: `dispatch_gold_broadcast` runs
  - Then: Both player A's and player B's receivers receive the broadcast

- **Missing connection: no panic, warn logged**
  - Given: `S2CGoldUpdate { player: C, .. }` enqueued but player C has no `PeerId` in `PlayerConnectionMap`
  - When: `dispatch_gold_update` runs
  - Then: `warn!` logged; no panic; other queued events unaffected

- **ReliableChannel enforcement**
  - Given: `economy_dispatch.rs` as implemented
  - When: `grep "UnreliableChannel" server/src/network/economy_dispatch.rs`
  - Then: Zero matches (CI gate)

---

## Test Evidence

**Story Type**: Integration
**Required evidence**:
- `tests/integration/economy/network_dispatch_test.rs` — unicast isolation, broadcast delivery, missing-connection safety
- `tests/evidence/story-eco-006-dispatch-test-approach.md` — documents chosen Lightyear in-process test pattern (MessageSender spy vs. live Lightyear)
**Status**: [x] Complete - integration evidence created and verified

---

## Dependencies

- Depends on: Story 002 (`S2CGoldUpdate` and `S2CGoldBroadcast` Bevy events defined and enqueued by economy systems)
- Depends on: S1-04 (`workspace-and-shared-types` Story 004 — protocol skeleton; `shared/src/protocol.rs` must exist for message type definitions)
- Depends on: `lightyear-protocol-verification` Story 001 DONE (verified Lightyear 0.26 API patterns available in `tests/evidence/lightyear-026-verification.md`)
- Depends on: `lightyear-protocol-verification` Story 003 DONE (server + client network plugins wired; server S2C `ServerMultiMessageSender` pattern proven)
- Depends on: `lightyear-protocol-verification` Story 004 DONE (broadcast pattern and final DIFFERS resolutions documented)
- Note: `liv-bevy-lightyear` skill is MANDATORY on the implementing agent — this is the first Economy story that touches Lightyear code directly
- Unlocks: Economy System epic complete; full Economy → Client sync path proven end-to-end; Board/Lane System and Auction System can rely on Economy state being mirrored to clients correctly

## Readiness Refresh Notes

**Refreshed**: 2026-05-03 against Control Manifest Version 2026-05-01.
**Changes**: Replaced placeholder TR wording with active TR-NP-010/TR-HUD-007 trace, updated stale `MessageSender`/`ConnectionId` wording to verified `ServerMultiMessageSender`/`PeerId`, clarified internal Bevy messages vs. shared wire payloads, added the reserved-gold field present in `shared/src/protocol.rs`, and documented the no-live-WebSocket test approach.

## Completion Notes

**Completed**: 2026-05-03
**Verdict**: COMPLETE
**Criteria**: 9/9 passing; all acceptance criteria covered by static verification, `cargo check --workspace`, and `tests/integration/economy/network_dispatch_test.rs`.
**Deviations**: None blocking. Note: `EconomyNetworkOutbox` is present as a headless test adapter while live server sends still use `ServerMultiMessageSender::send::<M, ReliableChannel>`.
**Test Evidence**: `cargo test -p server --test economy_network_dispatch_test` passed 4/4. `cargo check --workspace` passed. `Select-String` channel gate found zero `UnreliableChannel` matches in `server/src/network/economy_dispatch.rs`. Evidence doc exists at `tests/evidence/story-eco-006-dispatch-test-approach.md`.
**Code Review**: Skipped by lean review mode.
**Sprint Status**: Unchanged; no explicit ECO-006 or S4-14 entry exists in `production/sprint-status.yaml`.
