# ADR-0002: Client-Server Authority Model

## Status

Accepted

## Date

2026-04-29

## Last Verified

2026-04-29

## Decision Makers

User (final authority) + technical-director (architecture) + network-programmer (Lightyear feasibility) + creative-director (bluff-game integrity)

## Summary

Defines authority for all game state in Lanes and Lies. The headless server is the sole authority over phase, economy, RNG, combat, and validation; the client is a read-only view that consumes S2C projections and emits C2S input intents. No client-side prediction, no client-side game logic, no shared simulation — the client/server boundary is enforced by separate Cargo crates with disjoint dependency graphs, not by feature flags.

## Engine Compatibility

| Field | Value |
|-------|-------|
| **Engine** | Bevy 0.18 + Lightyear 0.26 |
| **Domain** | Networking / Core |
| **Knowledge Risk** | HIGH — Lightyear 0.26 (Jan 2026) and Bevy 0.18 are post-cutoff (training data ends ~Bevy 0.14, Lightyear pre-0.20) |
| **References Consulted** | `docs/engine-reference/bevy/VERSION.md`, ADR-001 (spike findings on Lightyear 0.26 visibility primitives), `design/gdd/network-protocol.md`, `.claude/docs/technical-preferences.md` |
| **Post-Cutoff APIs Used** | Lightyear 0.26 `Server::send_message_to_target` with `NetworkTarget::Single(ClientId)` (unicast), Lightyear 0.26 component replication (`Replicate`, `NetworkVisibility`), Lightyear 0.26 `OnConnected` / `OnDisconnected` events, Bevy 0.18 `Resource` derive on server-only state |
| **Verification Required** | (1) C2S messages addressed to a `ClientId` not currently registered as a session participant must be ignored, not panic. (2) Server-only `Resource` types (e.g. `HiddenObjectives`, `ServerRng`) must not appear in any type included in the `protocol` crate's serialization graph — confirm by inspecting the linker output for the client crate. (3) Lightyear `Server::send_message_to_target` must remain the canonical unicast entry point in 0.26.x patch releases. |

> **Note**: Knowledge Risk is HIGH. This ADR must be re-validated if Lightyear or Bevy versions change. Flag as "Superseded" and write a new ADR if either dependency moves.

## ADR Dependencies

| Field | Value |
|-------|-------|
| **Depends On** | None (foundational) |
| **Enables** | ADR-003 (Cargo workspace layout: `client/`, `server/`, `protocol/`), ADR-009 (Round State Machine — phase ownership), ADR-011 (Reconnect & snapshot delivery) |
| **Blocks** | All networking implementation; LOBBY, DRAFT, AUCTION, PLACEMENT, RESOLUTION system stories |
| **Ordering Note** | Must be Accepted before any wire-format protocol story or any Lightyear plugin scaffolding is opened. ADR-001 (objective identity unicast) was accepted first as a focused spike but logically depends on the authority model formalized here — both are now jointly load-bearing. |

## Context

### Problem Statement

Lanes and Lies has multiple state classes with different visibility rules: public state (board positions, objective HP, current phase), per-player private state (hand, gold, mana pool), and per-player secrets the opponent must never see (`is_fake` per objective, RNG seed). The wrong authority model lets a malicious or buggy client desynchronize, leak secrets, or unilaterally advance phase. The auction subsystem in particular requires trusted bid arbitration — first-valid-wins ordering across two clients cannot be resolved on either client.

The decision must be made now because every system below — game session, round state machine, economy, combat, network protocol — assumes a specific answer. Deferring forces those systems to defensively design for both server-authoritative and shared-simulation worlds, which is expensive and produces unprincipled code.

### Current State

No code yet. The GDD (`network-protocol.md` Rule 1) declares server authority and `technical-preferences.md` lists "No game state on client" as a Forbidden Pattern, but neither document formalizes the boundary at the crate level, the message dispatch contract, or the reconnection protocol. ADR-001 has accepted unicast-message authority for the specific case of `ObjectiveIdentity`. This ADR generalizes that posture to all game state.

### Constraints

- **Engine**: Lightyear 0.26 provides component replication and per-target reliable messages. It does not provide shared deterministic simulation or rollback (those are out of scope for the engine and the project).
- **Platform**: WASM client must stay under the 50 MB bundle budget — the client crate cannot pull server-only dependencies (server RNG, server-side validators, hidden state).
- **Pacing**: Game is turn-based with placement windows on the order of 10 seconds and an auction phase with multi-second bid windows. Per-frame input latency is not a competitive factor.
- **Hackathon scope**: No client-side prediction infrastructure available; building it would consume 1–2 weeks with no player-visible benefit at the chosen pacing.
- **Integrity**: Bluff mechanics require server-only secrets (ADR-001). A shared-simulation model would require the client to hold the secret in memory in some form — unacceptable.

### Requirements

- **R1 — Single source of truth**: For every piece of game state, there is exactly one authoritative writer (the server). Clients never mutate authoritative state directly.
- **R2 — Validation is server-side and silent**: Invalid C2S input is discarded server-side with no error response sent to the client (per network-protocol.md Rule 4 — prevents timing attacks). Diagnostic logging is internal.
- **R3 — Crate-level enforcement**: The boundary is enforced by the Rust type system / linker, not by `#[cfg(...)]` flags. A programmer cannot accidentally import a server-only type into client code.
- **R4 — Reconnect determinism**: A client that reconnects mid-game receives a complete projected snapshot and rebuilds its view from scratch. No state is reconstructed from incremental message replay.
- **R5 — Secret privacy**: Server-only secrets (objective identity, RNG seed, opponent's private hand) never enter the client crate's binary at any point.
- **R6 — Performance**: Server tick budget ≤ 5ms in the steady state on a single Railway dyno. Client per-frame budget ≤ 16.67 ms (60 FPS) including network message processing.

## Decision

The server is the sole authority over all game state. The client is a view. The boundary is enforced by three separate Cargo crates with disjoint dependency graphs:

- `protocol/` — message types, IDs, enums, and shared data schemas. No game logic. Depended on by both `client/` and `server/`.
- `server/` — headless Rust binary. Owns all game logic, RNG, validation, hidden state. Depends on `protocol/` only. Never compiled into the client bundle.
- `client/` — WASM binary. Owns rendering, input capture, view-state mirror. Depends on `protocol/` only. Cannot reach into `server/`.

There is no `#[cfg(feature = "server")]` conditional compilation in shared code. Server and client are physically separate crates. A type that must not appear on the client lives in `server/` and is therefore unreachable from `client/` code.

### Architecture

```
                        Lanes and Lies — Authority Model

  +-----------------------------------+        +-----------------------------------+
  |          CLIENT  (WASM)           |        |        SERVER  (Railway)          |
  |  crate: client                    |        |  crate: server                    |
  |  depends-on: protocol             |        |  depends-on: protocol             |
  +-----------------------------------+        +-----------------------------------+
  |  ClientState (read-only mirror)   |        |  Authoritative ECS World          |
  |    - PhaseView                    |        |    - RoundState (RSM)             |
  |    - SelfHandView                 |        |    - Players { gold, hand, mana } |
  |    - PublicBoardView              |        |    - Board { units, objectives }  |
  |    - LocalEffectsCache            |        |    - AuctionState                 |
  |                                   |        |                                   |
  |  Server-only Resources: NONE      |        |  Server-only Resources:           |
  |                                   |        |    - HiddenObjectives  (ADR-001)  |
  |                                   |        |    - ServerRng (ChaCha, seeded)   |
  |                                   |        |    - SessionConfig                |
  |                                   |        |    - DisconnectTrackers           |
  +------------------+----------------+        +------------------+----------------+
                     |                                            |
                     |  C2S input intents (Lightyear messages)    |
                     |  reliable / phase-gated / validated        |
                     +------------------------------------------> |
                     |                                            |
                     |                                            |  validate_phase()
                     |                                            |  validate_resource()
                     |                                            |  apply_to_world()
                     |                                            |
                     |  S2C projections                           |
                     |  - replicated components (public state)    |
                     |  - reliable unicast messages (private)     |
                     |  - reliable broadcast messages (events)    |
                     | <------------------------------------------+
                     |                                            |
                     |  S2CGameSnapshot (on connect / reconnect)  |
                     |  unicast, opponent secrets stripped        |
                     | <------------------------------------------+


  Data flow legend:
    --->  C2S: client expresses INTENT, server is free to reject silently
    <---  S2C: server expresses TRUTH, client must accept

  Forbidden:
    X     Client mutates ClientState from local input (no prediction)
    X     Client derives game logic outcomes (no shared simulation)
    X     Server-only Resource appears in `protocol/` or `client/` crate
    X     `cfg(feature = "server")` guarding game logic in shared modules
```

### Key Interfaces

```rust
// ---------- protocol/ crate ----------
// Public message contract. NO logic, NO secrets, NO server-only types.

pub mod protocol {
    use serde::{Serialize, Deserialize};

    /// Identifies a player's session slot (stable across reconnect).
    /// Distinct from Lightyear's transient ClientId.
    #[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct PlayerId(pub u8);

    #[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
    pub struct SessionToken(pub u128);

    /// Every client-bound projection type lives here.
    /// Each is either a replicated component or a Lightyear message.
    pub enum S2C {
        Handshake(S2CHandshake),
        GameSnapshot(S2CGameSnapshot),
        PhaseChanged(S2CPhaseChanged),
        ObjectiveIdentities(S2CObjectiveIdentities), // ADR-001 unicast
        // ... see network-protocol.md for exhaustive list
    }

    pub enum C2S {
        Hello(C2SHello),
        Heartbeat(C2SHeartbeat),
        PurchaseCard(C2SPurchaseCard),
        PlaceBid(C2SPlaceBid),
        SubmitPlacement(C2SSubmitPlacement),
        // ... see network-protocol.md
    }
}


// ---------- server/ crate ----------
// Authoritative Resources. NEVER reachable from client.

#[derive(Resource)]      // Bevy 0.18 Resource
struct HiddenObjectives { /* per-player is_fake map; ADR-001 */ }

#[derive(Resource)]
struct ServerRng(rand_chacha::ChaCha20Rng);

/// Single canonical entry point for every C2S handler.
/// Pattern enforced by code review and the `liv-bevy-lightyear` skill.
fn handle_c2s_message(
    msg: protocol::C2S,
    sender: ClientId,
    world: &mut World,
) {
    // 1. Resolve sender ClientId -> PlayerId via SessionRegistry.
    //    Unknown sender -> log + drop (NEVER panic).
    let Some(player_id) = world.resource::<SessionRegistry>().player_for(sender) else {
        warn!(?sender, "C2S from unregistered ClientId — dropped");
        return;
    };

    // 2. Phase-gate. Silent discard per network-protocol.md Rule 4.
    let phase = world.resource::<RoundState>().phase();
    if !msg.is_valid_in_phase(phase) {
        debug!(?msg, ?phase, ?player_id, "C2S rejected: phase-gated");
        return;
    }

    // 3. Domain validation (gold, hand size, bid amount, etc.).
    //    Silent discard on any failure.
    if let Err(reason) = validate(&msg, player_id, world) {
        debug!(?msg, ?reason, ?player_id, "C2S rejected: validation");
        return;
    }

    // 4. Apply to authoritative state. Atomic per Rule 5.
    apply(msg, player_id, world);

    // 5. Broadcast / unicast S2C as required by the message family.
}


// ---------- client/ crate ----------
// Read-only view. NO game logic, NO mutation paths from input.

#[derive(Resource, Default)]
struct ClientState {
    phase: PhaseView,
    self_hand: SelfHandView,
    public_board: PublicBoardView,
    // ...
}

/// All ClientState mutation flows through THIS function.
/// Inputs to it are always inbound S2C messages or replicated components.
/// User input never calls this.
fn apply_s2c_to_client_state(
    msg: protocol::S2C,
    state: &mut ClientState,
) {
    match msg {
        protocol::S2C::GameSnapshot(snap) => state.replace_from(snap),
        protocol::S2C::PhaseChanged(p)    => state.phase = p.into(),
        protocol::S2C::ObjectiveIdentities(ids) => state.cache_identities(ids),
        // ...
    }
}

/// User input -> outbound C2S only. Never touches ClientState directly.
fn on_user_clicked_buy(card_id: CardId, net: &mut LightyearClient) {
    net.send_reliable(protocol::C2S::PurchaseCard(C2SPurchaseCard { card_id }));
    // No optimistic update. Wait for S2C confirmation.
}
```

### Implementation Guidelines

1. **Cargo workspace layout** (formalized in ADR-003):
   ```
   Cargo.toml          # workspace
   protocol/           # shared types, no logic
   server/             # headless binary, depends on protocol
   client/             # WASM binary, depends on protocol
   ```
   `client/Cargo.toml` MUST NOT list `server` as a dependency. CI MUST fail any PR that introduces such an edge.

2. **No `cfg(feature = "...")` for authority gating.** If a type is server-only, it lives in `server/`. If shared, it lives in `protocol/`. There is no third option.

3. **C2S handlers are total functions over invalid input.** Every handler validates phase, sender identity, and domain rules. On any failure, the handler returns; it does not error to the client. Use `tracing::debug!` for rejection logs; never `panic!`.

4. **No optimistic client updates.** When the player clicks "Buy Card", the client emits `C2SPurchaseCard` and waits. The hand UI updates only when `S2CCardAcquired` (or the corresponding replicated component change) arrives. This is acceptable because every confirmation is one round-trip on a reliable channel; auction and placement phases tolerate this latency.

5. **Snapshot-driven reconnect.** On `OnConnected` (Lightyear 0.26), the server unicasts `S2CGameSnapshot` with opponent secret fields stripped (per network-protocol.md Rule 6). The client treats the snapshot as a full reset: discard all locally buffered state, rebuild `ClientState` from scratch. Any S2C messages received before snapshot processing completes are dropped (Rule 7).

6. **Server tick is the wall clock.** All time-based logic (auction timer, placement timer, disconnect grace) reads server time. The client's view of remaining time is derived from `S2CPhaseChanged.deadline_server_ms` minus its own clock with drift compensation; this is presentation only and never feeds back into authoritative state.

7. **RNG never leaves the server.** `ServerRng` is seeded at session start from OS entropy. The seed is never serialized into any `protocol/` type. RNG outputs (shop rolls, Ecaflip dice, fake-objective placement) are revealed to clients only as concrete results in S2C messages.

8. **Lightyear specifics.** Use `Server::send_message_to_target::<ReliableChannel, _>(..., NetworkTarget::Single(client_id))` for unicast (per ADR-001). Public state replicated via Lightyear `Replicate` components (e.g. `BoardPosition`, `ObjectiveHp`). Confirm exact symbol names against `docs.rs/lightyear/0.26.x` before merge — this is a HIGH-risk post-cutoff API.

## Alternatives Considered

### Alternative 1: Client-side prediction

- **Description**: Client speculatively applies the predicted result of its own inputs (e.g. mark hand as -1 card on `PurchaseCard`), then reconciles when the server response arrives. Standard pattern for action games.
- **Pros**: Hides round-trip latency from the player on their own actions.
- **Cons**: Adds a reconciliation system, divergence detection, rollback handling, and a parallel "predicted state" mirror on the client. Each adds bug surface area (especially around bluff mechanics — a mispredicted reveal would briefly leak server-only state). Engine support: not provided by Lightyear 0.26 out of the box.
- **Estimated Effort**: 1–2 weeks of network-programmer time + ongoing maintenance burden.
- **Rejection Reason**: The game's pacing makes prediction unnecessary. Auction bids are evaluated server-side over multi-second windows; placement is a 10-second batch submission; combat resolution is a scripted replay. There is no per-frame input latency to hide. Spending effort here trades budget against player-visible value with no return.

### Alternative 2: Shared deterministic simulation (client mirrors server logic)

- **Description**: Client and server run the same game logic against the same inputs in lockstep. Server is canonical, but client locally executes the same code path so its view advances without waiting for S2C messages.
- **Pros**: Lowest perceived latency; rich client-side reactive UI.
- **Cons**: Game logic must live in `protocol/` or a shared crate, which means server-only secrets (objective identity, RNG seed) would either need to be fed to the client (security failure) or branched away with `cfg` (re-introducing the conditional-compilation footgun this ADR is designed to eliminate). Bluff-game integrity is incompatible with the client running real game logic. `technical-preferences.md` explicitly forbids "game state on client".
- **Estimated Effort**: Comparable to authoritative-only, but with much higher long-term maintenance and a critical security failure mode.
- **Rejection Reason**: Directly violates the bluff pillar — the moment any objective-identity-dependent code runs on the client, `is_fake` must be reachable from client memory in some form, and any bug in the visibility filter leaks the bluff. Rejected as architecturally hostile to the game's signature mechanic.

### Alternative 3: Peer-to-peer (P2P)

- **Description**: Two clients communicate directly; no headless server. One client elected as "host" arbitrates contested actions.
- **Pros**: No server hosting cost (Railway dyno saved); simplest deploy.
- **Cons**: Auction system requires trusted ordering of bids placed under multi-second pressure — a P2P host can trivially cheat by reordering its own bid relative to the opponent's. Bluff-game secrets ("is this objective fake?") cannot be hidden from the host. Reconnect is fragile (host dropping = game ending). NAT traversal in browsers is non-trivial.
- **Estimated Effort**: Comparable but with worse cheat surface and worse reliability.
- **Rejection Reason**: Auction integrity and bluff secrecy both require a trusted third party. P2P fundamentally cannot provide it. The tiny operational cost of a Railway dyno is negligible compared to the design integrity it buys.

## Consequences

### Positive

- **Single source of truth.** Every state mutation has exactly one writer. Debugging desync questions become trivial: ask the server.
- **Bluff integrity by construction.** Server-only types live in the `server/` crate. The client crate cannot import them. ADR-001's unicast pattern generalizes cleanly: any future per-player secret follows the same dispatch path.
- **Crate-level enforcement.** Boundary violations are link-time errors, not runtime bugs.
- **Cheat resistance.** No client-side authority means no client-side cheats. The worst a malicious client can do is spam C2S messages, which the server rate-limits and silently discards.
- **Simpler reconnect.** Snapshot-driven recovery has one path: replace `ClientState` from `S2CGameSnapshot`. No incremental replay, no causal ordering of buffered events.
- **Clean test boundary.** Server logic can be tested headlessly with no Lightyear dependency by feeding `protocol::C2S` values directly into `handle_c2s_message`. Client view logic can be tested by feeding `protocol::S2C` values into `apply_s2c_to_client_state`. Each side tests against the protocol contract, not against the network.

### Negative

- **One round-trip latency on every player-initiated action.** No optimistic UI. A player clicking "Buy Card" on a 100ms RTT link sees the hand update ~100ms later. Acceptable at chosen pacing; surfaces as a UI feedback design problem (loading spinners, disabled buttons, animation timings) rather than a networking problem.
- **Heavier reliance on reliable channels.** Most signals (purchases, bids, placements) need ordered reliable delivery. Reliable bandwidth is more expensive than unreliable replication; total bandwidth budget (`< 1 KB / round`) must be re-validated after first integration test.
- **Server CPU cost.** All validation, RNG, and combat resolution run on the server. Within budget for two-player sessions on a single Railway dyno; revisit if the project ever scales to N>2 or to multiple concurrent sessions per dyno.
- **No P2P/LAN fallback.** A server outage means no game. Mitigated by Railway uptime guarantees and a clean reconnect path.

### Neutral

- The Cargo workspace gets three crates instead of one. Build times marginally longer; cleaner dependency graph.
- Client and server can be deployed and updated independently as long as `protocol/` versioning (per network-protocol.md Rule 3) is respected. Mismatched versions are detected at handshake and rejected (`S2CHandshakeRejected`).

## Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Lightyear 0.26 API used here (`send_message_to_target`, `NetworkTarget::Single`, `OnConnected`) shifts in a 0.26.x patch release. | Medium | Medium | Pin exact patch version in `Cargo.toml`. Wrap Lightyear calls behind a thin `server::net` adapter so an API change touches one file. |
| A contributor adds a `cfg(feature = "server")` branch to `protocol/` for "convenience", reintroducing the conditional-compilation footgun. | Medium | High | Codify in `coding-standards.md` and as a `liv-bevy-lightyear` skill rule. Add a CI grep that fails on `cfg(feature = "server")` outside `server/`. |
| Round-trip latency on player input becomes a feel problem despite the turn-based pacing. | Low | Medium | Validate during first playable milestone; if a specific interaction feels bad, address it with UI feedback (spinner, disabled state) rather than reintroducing prediction. |
| `S2CGameSnapshot` payload exceeds the per-message budget on reconnect mid-late-round (large board state). | Low | Medium | Define snapshot size budget (target < 16 KB). Audit during first integration test. If exceeded, chunk into multiple reliable messages with an explicit completion marker before client begins `ClientState` rebuild. |
| Server tick budget exceeded under combat resolution load. | Low | Medium | Profile during first vertical slice; combat resolution can be batched across ticks if needed. |
| `S2CGameSnapshot` accidentally ships an opponent secret because a new field is added to a shared type and the strip-on-send code is forgotten. | Medium | Critical | Encode the strip logic as a method on `S2CGameSnapshot` itself (`for_player(player_id)` constructor). Any future field added must pass through that constructor. Add a unit test that constructs a snapshot with both players' secrets present, then asserts the projected snapshot for player A contains nothing of player B's secret state. |

## Performance Implications

| Metric | Before | Expected After | Budget |
|--------|--------|---------------|--------|
| CPU (client frame time) | n/a (no code) | ≤ 2 ms for S2C processing + view update | 16.67 ms total (60 FPS) |
| CPU (server tick) | n/a | ≤ 5 ms steady state; ≤ 15 ms during RESOLUTION batch | 16.67 ms (server ticks at 60 Hz; headroom needed for combat resolution) |
| Memory (client) | n/a | `ClientState` < 1 MB; total WASM heap < 256 MB | 256 MB |
| Memory (server) | n/a | < 32 MB per session | Railway dyno default |
| Load Time (client) | n/a | First snapshot received within 500 ms of WS connect on a 100 ms RTT link | < 1s after handshake |
| Network (steady-state) | n/a | < 1 KB / round / player including replication deltas | 1 KB / round (per technical-preferences.md) |
| Network (snapshot on reconnect) | n/a | < 16 KB unicast | 32 KB hard ceiling |

## Migration Plan

This ADR is foundational; there is no existing implementation to migrate from. Adoption sequence:

1. **Workspace split** (ADR-003 — to be authored): create `protocol/`, `server/`, `client/` crates. Move the existing `Cargo.toml` content and any prototype code into the appropriate crate. Verify `client` does not depend on `server` via `cargo tree`.
2. **CI guard**: add a CI step that fails if `client/Cargo.toml` lists `server` as a dependency, or if any file under `client/` or `protocol/` contains `cfg(feature = "server")`.
3. **Protocol scaffold**: stub `protocol/src/lib.rs` with empty `S2C` and `C2S` enums and the foundational types (`PlayerId`, `SessionToken`).
4. **Server scaffold**: stub `server/src/main.rs` with the Lightyear server plugin, an empty `handle_c2s_message`, and the server-only resources (`HiddenObjectives` placeholder, `ServerRng`).
5. **Client scaffold**: stub `client/src/main.rs` with the Lightyear client plugin and the empty `ClientState` resource.
6. **Handshake first**: implement `C2SHello` -> `S2CHandshake` end-to-end before any other message. This validates the wire format and the authority pattern in one slice.
7. **Snapshot path**: implement `S2CGameSnapshot` (empty body initially) sent on `OnConnected`. This reserves the reconnect path before any state exists to snapshot.
8. **Cascade per system**: each subsequent system (lobby, draft, auction, placement, resolution) follows the same pattern — define C2S/S2C in `protocol/`, implement server handler in `server/`, implement view in `client/`.

**Rollback plan**: This is a foundational decision. Reverting it would require rewriting every networked system. There is no incremental rollback. If after the first vertical slice the model proves unworkable, write a superseding ADR before any further system code is built — do not patch around the model.

## Validation Criteria

- [ ] `client/Cargo.toml` does not depend on `server`. Verified by `cargo tree -p client` showing no `server` node.
- [ ] No `cfg(feature = "server")` exists anywhere in `protocol/` or `client/`. Verified by CI grep.
- [ ] All C2S handlers route through a single `handle_c2s_message` entry point in `server/`. Verified by code review during first network-protocol story landing.
- [ ] An invalid C2S message (wrong phase, insufficient gold, unknown sender) results in `tracing::debug!` log on the server and zero S2C response to the client. Verified by integration test driving the server with `protocol::C2S` values directly and asserting no outbound message.
- [ ] On client `OnConnected`, the server emits `S2CGameSnapshot` before any other S2C. Verified by ordering test in the network-protocol integration suite.
- [ ] `S2CGameSnapshot::for_player(p)` excludes any field marked server-only or belonging to player ≠ p. Verified by a unit test that constructs a fully-populated snapshot and asserts the projected variant for player A has no player-B secret state.
- [ ] `ServerRng` and `HiddenObjectives` are not present in any type reachable from `protocol/`. Verified by `cargo tree` and a review of `protocol/src/lib.rs`'s exported types.
- [ ] Steady-state server tick under a 2-player session stays within 5 ms on the target Railway dyno. Verified by `tracing` spans in the first vertical slice.

## GDD Requirements Addressed

| GDD Document | System | Requirement | How This ADR Satisfies It |
|-------------|--------|-------------|--------------------------|
| `design/gdd/network-protocol.md` | Network Protocol | Rule 1 — "The server is the sole source of game truth. Clients hold read-only mirrors and express intent via C2S messages only." | This ADR formalizes the rule at the crate level: server-only types live in the `server/` crate and are unreachable from the `client/` crate by construction. |
| `design/gdd/network-protocol.md` | Network Protocol | Rule 3 (TR-NP-01) — handshake protocol with versioning and reconnect identity. | C2S/S2C handshake messages defined in `protocol/`; handler enforces phase-gate (HANDSHAKING) before any other message accepted. |
| `design/gdd/network-protocol.md` | Network Protocol | Rule 7 (TR-NP-02) — `S2CGameSnapshot` delivered on every connection before any other S2C. | Implementation guideline 5 makes snapshot-on-`OnConnected` the canonical reconnect path; client discards in-flight messages until snapshot processed. |
| `design/gdd/network-protocol.md` | Network Protocol | Rule 4 (TR-NP-03) — phase-gated C2S, silent discard. | `handle_c2s_message` Step 2 silently rejects out-of-phase messages with `debug!` log only. |
| `design/gdd/network-protocol.md` | Network Protocol | Rule 5 (TR-NP-06) — submission atomicity; client must not advance until S2C confirmation. | Implementation guideline 4: no optimistic client updates. `ClientState` mutates only via inbound S2C. |
| `design/gdd/round-state-machine.md` | Round State Machine | TR-RSM-09 — `S2CPhaseChanged` broadcast on every phase transition. | Phase ownership lives on the server (`RoundState` Resource); transitions emit `S2CPhaseChanged` as the canonical client signal. |
| `design/gdd/game-session-system.md` | Game Session System | TR-GSS-01 — room creation. | LOBBY-phase C2S messages (`C2SCreateRoom`, `C2SJoinRoom`) handled server-side under the same authority model; session registry maps Lightyear `ClientId` to stable `PlayerId`. |
| `design/gdd/game-session-system.md` | Game Session System | Rule 11 — `SessionConfig` handoff at `SessionReady`. | `SessionConfig` is a server-only `Resource`; values relevant to clients are projected via S2C messages. |
| `design/gdd/game-session-system.md` | Game Session System | Rule 12 — server restart during LOBBY destroys session, no resume. | Authority model places all session state on the server; if the server process dies, no client holds recoverable state. Consistent with this ADR's snapshot-driven reconnect semantics: there is nothing to reconnect to. |
| `.claude/docs/technical-preferences.md` | Forbidden Patterns | "No game state on client" / "No client-side RNG" | Server-only crate isolation enforces both at link time, not at code-review time. |

## Related

- [ADR-001 — Hidden Objective Identity via Targeted Unicast](./adr-001-objective-identity-unicast.md) — first concrete instance of the authority pattern this ADR generalizes. ADR-001 remains accepted; this ADR provides the umbrella architecture under which it sits.
- [ADR-003 — Cargo workspace layout (pending)](./adr-003-cargo-workspace.md) — operationalizes the three-crate split prescribed here.
- [ADR-009 — Round State Machine ownership (pending)](./adr-009-round-state-machine.md) — phase authority is a direct consequence of this ADR.
- [ADR-011 — Reconnect & snapshot delivery (pending)](./adr-011-reconnect-snapshot.md) — refines the snapshot path sketched in Implementation Guideline 5.
- `design/gdd/network-protocol.md` — wire-level message catalog.
- `design/gdd/game-session-system.md` — session lifecycle and `SessionConfig` semantics.
- `.claude/docs/technical-preferences.md` — Forbidden Patterns section.
- `docs/engine-reference/bevy/VERSION.md` — engine version pin and knowledge-risk notes.
