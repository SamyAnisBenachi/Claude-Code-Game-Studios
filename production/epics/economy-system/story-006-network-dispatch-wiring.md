# Story 006: Network Dispatch Wiring

> **Epic**: Economy System
> **Status**: Ready
> **Layer**: Core
> **Type**: Integration
> **Manifest Version**: 2026-04-29

## Context

**GDD**: `design/gdd/economy-system.md`
**Requirement**: TR-??? (covers TR-ECO-01 partial — client state mirror for gold/mana; GDD Rule 6 gold visibility; GDD Rule 7 auction broadcast)

**ADR Governing Implementation**: ADR-010: RSM Phase Event Bus + ADR-008: Lightyear Channel Configuration
**ADR Decision Summary (ADR-010)**: Economy emits `S2CGoldUpdate` (unicast to owner) and `S2CGoldBroadcast` (broadcast all) as internal Bevy events. A dedicated dispatch system in `server/src/network/` reads these events and sends them via Lightyear's `MessageSender`.
**ADR Decision Summary (ADR-008)**: Both messages are sent on `ReliableChannel`. `S2CGoldUpdate` carries the full private state (`gold`, `current_mana`, `reserve_mana`, `mana_cap`) to the owning player only. `S2CGoldBroadcast` carries only `{ player: PlayerId, gold: u32 }` to all players — gold is publicly visible per GDD Rule 6. Both are sent on `ReliableChannel` because currency desync is a game-breaking bug; reliability is mandatory.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: MEDIUM
**Engine Notes**: `MessageSender<S2CGoldUpdate>` unicast requires specifying `NetworkTarget::Single(player_connection_id)`. `MessageSender<S2CGoldBroadcast>` uses `NetworkTarget::All`. Lightyear 0.26 API patterns must follow `liv-bevy-lightyear` skill. Bevy-side: `MessageWriter::write()` / `MessageReader::read()` — `EventWriter`/`EventReader` do not exist in Bevy 0.18. The dispatch system reads Bevy messages (internal) and calls Lightyear send (external) — both API families are active in the same file; BOTH skills are mandatory.

> **liv-bevy-lightyear skill is MANDATORY on the implementing agent for this story.** Any `.rs` file that imports `lightyear` must activate this skill. Failure to do so risks pre-0.26 Lightyear API patterns that will fail to compile.

**Control Manifest Rules (Core layer)**:
- Required: `S2CGoldUpdate` is unicast — sent ONLY to the player who owns the economy state. Private currency information (current_mana, reserve_mana, mana_cap) must not be broadcast.
- Required: `S2CGoldBroadcast` is broadcast (`NetworkTarget::All`) — gold totals are publicly visible. This is the only Economy message that reaches the opponent.
- Required: Both messages sent on `ReliableChannel` per ADR-008.
- Required: Dispatch system lives in `server/src/network/economy_dispatch.rs` — NOT in `server/src/core/economy/`. Economy core does not import Lightyear; the network module does.
- Forbidden: Economy core (`server/src/core/economy/`) must NOT import `lightyear`. The dispatch system is the only file with Lightyear imports for Economy events.
- Guardrail: If `ConnectionId` for a player is not found (player disconnected), log a debug warning and skip the unicast — do not panic. Reconnect/resync is handled by GSS epic.

---

## Acceptance Criteria

- [ ] `server/src/network/economy_dispatch.rs` exists with two systems:
  - `dispatch_gold_update(mut reader: MessageReader<S2CGoldUpdate>, sender: MessageSender<S2CGoldUpdate>, connection_map: Res<PlayerConnectionMap>)` — unicasts each message to the owning player's `ConnectionId` on `ReliableChannel` — TODO(liv-bevy-018): verify MessageReader<T> type name in Bevy 0.18
  - `dispatch_gold_broadcast(mut reader: MessageReader<S2CGoldBroadcast>, sender: MessageSender<S2CGoldBroadcast>)` — broadcasts each message to `NetworkTarget::All` on `ReliableChannel`
- [ ] `PlayerConnectionMap` resource (or equivalent Lightyear API for player→connection lookup) is used to resolve `PlayerId` → `ConnectionId` for unicast
- [ ] If `ConnectionId` not found for a player in `dispatch_gold_update`, a `warn!` (not `error!`, not panic) is emitted and the event is skipped
- [ ] Both systems are registered in `EconomyPlugin` or a new `EconomyNetworkPlugin` in `server/src/network/`; scheduled in `Update` `.after(on_draft_started)` and `.after(handle_kill_award)` and `.after(handle_objective_award)` (dispatch runs after all economy mutations that enqueue events in the same frame)
- [ ] `S2CGoldUpdate` and `S2CGoldBroadcast` types are defined in `shared/src/protocol.rs` with `#[derive(Serialize, Deserialize, Clone, Debug)]` and the Lightyear message derive macro (per `liv-bevy-lightyear` skill for Lightyear 0.26)
- [ ] Integration test: GIVEN two players initialised, WHEN `DraftStarted { round: 1, phase: Initial }` processed end-to-end (init → on_draft_started → dispatch), THEN `S2CGoldUpdate` unicast fires exactly once per player (player A receives their own; player B receives their own; neither receives the other's private update); `S2CGoldBroadcast` fires twice in total (once per player's gold, broadcast to all)
- [ ] Integration test: GIVEN player A wins a kill award, WHEN `handle_kill_award` fires → `S2CGoldUpdate` enqueued → dispatch runs, THEN `S2CGoldUpdate` for player A is sent on `ReliableChannel`; `S2CGoldBroadcast` for player A is sent to all
- [ ] `cargo check --workspace` passes
- [ ] CI channel-type gate: grep for `UnreliableChannel` in `server/src/network/economy_dispatch.rs` returns zero matches — economy messages must not accidentally be sent on the unreliable channel

---

## Implementation Notes

*Derived from EPIC.md §Network dispatch wiring and ADR-008:*

**Lightyear 0.26 unicast pattern (verify against `liv-bevy-lightyear` skill before implementing):**
```rust
// Approximate pattern — exact Lightyear 0.26 API must be verified via the skill
fn dispatch_gold_update(
    mut events: MessageReader<S2CGoldUpdate>,  // TODO(liv-bevy-018): verify MessageReader type name
    mut sender: MessageSender<S2CGoldUpdate>,
    connection_map: Res<PlayerConnectionMap>,
) {
    for event in events.read() {
        match connection_map.get(&event.player) {
            Some(conn_id) => {
                sender.send_to(*conn_id, event.clone(), NetworkTarget::Single(*conn_id));
            }
            None => {
                warn!("dispatch_gold_update: no ConnectionId for {:?} — skipping unicast", event.player);
            }
        }
    }
}
```
The exact `MessageSender` API, channel parameter passing, and `NetworkTarget` enum variants must be confirmed against the Lightyear 0.26 verified patterns from the `lightyear-protocol-verification` Foundation epic. Do not use the `liv-bevy-lightyear` skill's suggested code as-is without first cross-referencing with `tests/evidence/lightyear-026-verification.md`.

**`PlayerConnectionMap` resource:** This resource maps `PlayerId` → Lightyear `ConnectionId`. It is maintained by the GSS epic (Game Session System) — Economy does not own it. Import it as `Res<PlayerConnectionMap>`. If not yet defined when this story is implemented, use a stub `HashMap<PlayerId, ConnectionId>` with a `// TODO: import from shared or GSS` comment.

**Why dispatch is separate from economy core:** Economy core (`server/core/economy/`) has no Lightyear dependency. This keeps the economy module independently testable (Stories 001–005 use no Lightyear at all). The dispatch module is the only place Economy-related Lightyear code lives. This matches the ADR-003 workspace structure: core logic is pure, network is a separate module.

**`S2CGoldUpdate` message fields:**
```rust
// shared/src/protocol.rs
#[derive(Serialize, Deserialize, Clone, Debug, /* Lightyear message derive */)]
pub struct S2CGoldUpdate {
    pub player: PlayerId,
    pub gold: u32,
    pub current_mana: u32,
    pub reserve_mana: u32,
    pub mana_cap: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, /* Lightyear message derive */)]
pub struct S2CGoldBroadcast {
    pub player: PlayerId,
    pub gold: u32,
}
```
These are the full message shapes. `reserved_gold` is intentionally omitted from both — it is an internal server accounting field only; clients do not need it.

**Integration test scope:** The integration test for this story wires the full Economy stack (Stories 001–005 logic) with the dispatch systems in a single Bevy `App`. It does NOT require a live Lightyear WebSocket — use Lightyear's in-process test client/server pattern (verified in `lightyear-protocol-verification` Story 004) or verify that `MessageSender` can be replaced with a test spy that records sent messages. Document the chosen test approach in `tests/evidence/story-eco-006-dispatch-test-approach.md`.

---

## Out of Scope

- `S2CPhaseChanged` dispatch — owned by RSM epic's network dispatch system
- Full reconnect resync — GSS epic re-sends full state including Economy state on reconnect
- Client-side rendering of gold/mana bars — HUD GDD (`design/ux/hud.md`) and UI programmer
- `S2CAuctionBidRejected` — Auction System (M2) network message; not Economy's

---

## QA Test Cases

*QL-STORY-READY skipped — Lean mode.*

- **Unicast isolation: player A's update does not reach player B**
  - Given: Two-player test app; `S2CGoldUpdate { player: A, gold: 8, .. }` enqueued
  - When: `dispatch_gold_update` runs
  - Then: Player A's message receiver receives the update; Player B's receiver does NOT receive `S2CGoldUpdate` for A

- **Broadcast reaches all players**
  - Given: `S2CGoldBroadcast { player: A, gold: 8 }` enqueued
  - When: `dispatch_gold_broadcast` runs
  - Then: Both player A's and player B's receivers receive the broadcast

- **Missing connection: no panic, warn logged**
  - Given: `S2CGoldUpdate { player: C, .. }` enqueued but player C has no `ConnectionId` in `PlayerConnectionMap`
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
**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 002 (`S2CGoldUpdate` and `S2CGoldBroadcast` Bevy events defined and enqueued by economy systems)
- Depends on: S1-04 (`workspace-and-shared-types` Story 004 — protocol skeleton; `shared/src/protocol.rs` must exist for message type definitions)
- Depends on: S1-05 (`lightyear-protocol-verification` Story 005 / Story 003 — server + client network plugins must be wired; verified Lightyear 0.26 patterns must be available in `tests/evidence/lightyear-026-verification.md`)
- Note: `liv-bevy-lightyear` skill is MANDATORY on the implementing agent — this is the first Economy story that touches Lightyear code directly
- Unlocks: Economy System epic complete; full Economy → Client sync path proven end-to-end; Board/Lane System and Auction System can rely on Economy state being mirrored to clients correctly
