# ADR-016: Prism System Architecture — State Ownership, Schedule Slot, and Hand-Write API

## Status

Accepted

## Date

2026-04-30

## Engine Compatibility

| Field | Value |
|-------|-------|
| **Engine** | Bevy 0.18 |
| **Domain** | Core / Feature State Management |
| **Knowledge Risk** | HIGH — Bevy 0.15–0.18 all post-cutoff; Message/Event split in 0.17+ is critical; Lightyear 0.26 server→client targeted-send API verified for Prism on 2026-05-02 |
| **References Consulted** | `docs/engine-reference/bevy/VERSION.md`, `docs/engine-reference/bevy/current-best-practices.md`, ADR-010 (Message/Event split patterns), ADR-005 (RESOLUTION schedule slot), ADR-008 (Lightyear channel config), `tests/evidence/lightyear-026-verification.md`, local Lightyear 0.26.4 crate source |
| **Post-Cutoff APIs Used** | `MessageReader<T>` / `MessageWriter<T>` (`#[derive(Message)]`) for server-internal messages; `ResMut<T>` for PrismState; Lightyear 0.26 component replication via `Replicate::to_clients(NetworkTarget::All)` for public PrismPresence; Lightyear server→client targeted send via `ServerMultiMessageSender::send::<Message, ReliableChannel>(&msg, server, &NetworkTarget::...)` |
| **Verification Required** | (1) **RESOLVED 2026-05-02**: Lightyear 0.26.4 server→client targeted send uses `ServerMultiMessageSender`, not a `ConnectionManager` or server-handle method. Generic order is `<Message, Channel>`; unicast target is `NetworkTarget::Single(peer_id)` where the identifier type is `PeerId`, not `ClientId`; all-player delivery uses `NetworkTarget::All`. (2) **RESOLVED for PrismPresence**: Prism visibility is public board state, so use `Replicate::to_clients(NetworkTarget::All)`; no owner-only per-entity scoping is required for PrismPresence. (3) **RESOLVED**: `MessageReader<T>::read()` confirmed — `for msg in reader.read()` is the correct drain iterator form, per `docs/engine-reference/bevy/current-best-practices.md` Bevy 0.17+ Message patterns. |

> **Engine Specialist Note (2026-04-30)**: `PrismCollected` is a server-internal Bevy `#[derive(Message)]` emitted by the Board/Lane System server code. It is NOT a Lightyear C2S message from a client. `MessageReader<PrismCollected>` is therefore the correct system parameter — this is the Bevy-layer message API, not Lightyear's `MessageReceiver<T>`. The two APIs must not be conflated: `MessageReader<T>` is for server-internal Bevy messages; `MessageReceiver<T>` (Lightyear) is for C2S network messages.

---

## ADR Dependencies

| Field | Value |
|-------|-------|
| **Depends On** | ADR-005 (defines `resolve_prism_draws` by name in the RESOLUTION schedule — this ADR formalizes its system signature and resource parameters); ADR-010 (RSM event bus — `ResolutionPhaseEntered` gates RESOLUTION systems; Prism subscriber contract must be added to ADR-010 Subscriber Contracts table before any Prism story is marked Done); ADR-008 (Lightyear channel config — `S2CCardAcquired` reliable unicast channel assignment; `PrismPresence` unreliable channel); ADR-006 (card data schema — static `prism_strike` and `prism_reserve` definitions in `assets/data/cards.json`) |
| **Enables** | Prism System epic implementation stories; HUD epic (prism progress / respawn counter per `hud.md`); Card Animations epic (prism collection burst VFX, PrismPresence state change); Board Rendering epic (PrismPresence component replication to clients) |
| **Blocks** | Prism System epic implementation stories — none may start until this ADR is Accepted |
| **Ordering Note** | ADR-005 must be Accepted (already is). ADR-010 Subscriber Contracts table must be updated to add the `ResolutionPhaseEntered → Prism System` row before any Prism story is marked Done. Lightyear targeted-send Verification Required item 1 is resolved; `resolve_prism_draws` must use the verified `ServerMultiMessageSender` API. |

---

## Context

### Problem Statement

The Prism System GDD (`prism-system.md`) is complete and reviewed, but three architectural questions block story authoring:

1. **State ownership**: Where does `PrismState` live as a Bevy type, and which system holds exclusive `ResMut` access?
2. **Schedule slot formalization**: ADR-005 names `resolve_prism_draws` in the RESOLUTION schedule but does not define its system signature, resource parameters, or how it reads `PrismCollected` messages.
3. **Hand-write API (GDD OQ1)**: The GDD explicitly bypasses Card Acquisition for prism rewards (`card-acquisition.md` line 80), yet both the Prism System and Card Acquisition must write to the same `PlayerHands`. Without a resolved API boundary, either a `ResMut<PlayerHands>` conflict occurs (Bevy scheduler panic) or both systems implement duplicate hand logic (drift risk).

A fourth concern — `PrismPresence` client replication for Board Rendering — is cross-cutting with the network layer and needs a documented component replication pattern.

### Constraints

- `resolve_prism_draws` must run in the RESOLUTION `Update` set, after `resolve_ecaflip_triggers` and before `award_fake_objective_rewards` (ADR-005 contract, locked).
- Only one Bevy system may hold `ResMut<PlayerHands>` at a time — Bevy's scheduler prevents two systems with overlapping mutable access from running concurrently.
- `PrismState` mutations happen exclusively in `resolve_prism_draws`; no other system may write to it.
- Bevy 0.18: `EventWriter`/`EventReader` do not exist. `PrismCollected` is `#[derive(Message)]` (server-internal), consumed via `MessageReader<PrismCollected>`.
- No client-side RNG — all randomness (Lane 3 draw) is server-side via `ServerRng::next_seed()` per ADR-005.
- Server is authoritative; `PrismState` lives only on the server. Clients read prism state via `PrismPresence` component replication only.

### Requirements

- Must own `PrismState` as a single-writer resource with no access conflicts.
- Must provide a `hand_push()` API callable by both Prism System and Card Acquisition without concurrent `ResMut<PlayerHands>` conflicts.
- Must slot `resolve_prism_draws` into the ADR-005 RESOLUTION schedule with a concrete, correct system signature.
- Must define how `PrismPresence` components are replicated to clients for board rendering.
- Must support `S2CCardAcquired` reliable unicast to the owning player on each successful hand add, `S2CPrismRewardDropped` reliable unicast to the owning player on Lanes 1/2/4/5 hand-full, and `S2CPrismRespawned` reliable all-player delivery on full-set respawn.
- Must handle the `PrismCollected` message buffer correctly across multiple collections per RESOLUTION.

---

## Decision

### 1. PrismState Resource Ownership

`PrismState` is registered as a Bevy `Resource` by `PrismPlugin::build()` at session start. `ResMut<PrismState>` appears exclusively in `resolve_prism_draws`. No other system may access `PrismState` mutably. External systems (Board Rendering, HUD) read prism state via `PrismPresence` component replication on the client — they do not query `PrismState` directly.

```rust
// server/feature/prism/state.rs

#[derive(Resource, Default)]
pub struct PrismState {
    /// [player_index][lane_index (0-based, lane 1 = index 0)] — true = collected
    pub collected: [[bool; 5]; MAX_PLAYERS],
    /// Transient per-RESOLUTION flag; set in Rule 8, cleared after Rule 9 fires
    pub pending_respawn: [bool; MAX_PLAYERS],
}
```

`PrismState` is inserted by `PrismPlugin` on session start and removed on `GameOverEmitted` (same lifecycle as `ServerRng` per ADR-005).

### 2. Hand-Write API — Shared Module Function

The `hand` module exposes a pure function `hand_push` that both the Prism System and Card Acquisition call. The function takes `&mut PlayerHands` (not `ResMut<PlayerHands>` — it is called from within a system that holds the `ResMut`). Because Bevy's scheduler runs systems serially when they share a mutable resource, Card Acquisition and `resolve_prism_draws` never run in the same frame step simultaneously.

```rust
// server/feature/hand/mod.rs

pub fn hand_push(
    hand: &mut PlayerHands,
    player: PlayerId,
    card_id: CardId,
) -> Result<(), HandFullError> {
    if hand.len(player) >= HAND_SIZE_MAX {
        return Err(HandFullError);
    }
    hand.cards[player].push(card_id);
    Ok(())
}
```

**Scheduling guarantee**: `resolve_prism_draws` is the only RESOLUTION-phase system that holds `ResMut<PlayerHands>`. Card Acquisition systems run in DRAFT phase only (they do not run during RESOLUTION). Bevy enforces `ResMut` exclusivity at the system scheduler level — no concurrent write access is possible.

### 3. resolve_prism_draws System Signature

```rust
// server/feature/prism/systems.rs

pub fn resolve_prism_draws(
    mut prism_state: ResMut<PrismState>,
    mut hand: ResMut<PlayerHands>,
    server_rng: Res<ServerRng>,
    card_pool: Res<CardDataPool>,
    mut s2c_sender: ServerMultiMessageSender,
    server: Query<&Server>,
    // Use server.single() before sends. Owner-only sends use:
    //   s2c_sender.send::<S2CCardAcquired, ReliableChannel>(
    //       &msg, server, &NetworkTarget::Single(owner_peer_id),
    //   )
    // Respawn all-player delivery uses NetworkTarget::All.
    mut prism_presence: Query<(&PrismLaneKey, &mut PrismPresence)>,
    mut prism_collected: MessageReader<PrismCollected>,
    phase: Res<CurrentPhase>,
) {
    // Guard: only run during RESOLUTION (secondary — primary gate is .run_if scheduling)
    if phase.current != RoundPhase::Resolution {
        return;
    }
    // GDD Rules 3–9 implementation here
}
```

System registration in `PrismPlugin::build()`:

```rust
app.add_systems(
    Update,
    resolve_prism_draws
        .after(resolve_ecaflip_triggers)
        .before(award_fake_objective_rewards)
        .run_if(in_state(AppState::InGame)),
);
```

### 4. PrismPresence Component Replication

Ten `PrismPresence` entities are spawned at session start — one per `(player, lane)` pair. Each carries a `Replicate` component targeting the relevant clients via `UnreliableChannel`. The `collected: bool` field is the only replicated field.

```rust
// server/feature/prism/components.rs

#[derive(Component, Clone, Debug)]
pub struct PrismLaneKey {
    pub player: PlayerId,
    pub lane: u8,   // 1–5
}

// Lightyear-replicated to clients — verify Serialize/Deserialize derive requirement
#[derive(Component, Clone, Serialize, Deserialize)]
pub struct PrismPresence {
    pub collected: bool,
}
```

`resolve_prism_draws` updates `PrismPresence.collected` on the matching entity after each `PrismState` mutation. Lightyear picks up the component change and delivers it to clients on the next frame via `UnreliableChannel`. Board Rendering reads `PrismPresence` on the client to control prism token visibility.

> **Replication note (Verification Required item 2 resolved for Prism)**: PrismPresence is public board state. Spawn it with `Replicate::to_clients(NetworkTarget::All)`. Owner-only per-entity scoping is not needed for PrismPresence.

### Architecture Diagram

```
RESOLUTION Update Set (ADR-005 schedule)
│
├─ apply_placement_effects       [Board/Lane System]
│       │ emits MessageWriter<PrismCollected> (server-internal Bevy Message)
│       ▼
├─ resolve_ecaflip_triggers      [Ecaflip System, M2]
│
├─ resolve_prism_draws           ← THIS ADR
│   │
│   ├─ IN:  MessageReader<PrismCollected>    server-internal, from Board/Lane System
│   ├─ IN:  Res<ServerRng>                  next_seed() for Lane 3 only
│   ├─ IN:  Res<CardDataPool>               draw_random() for Lane 3 only
│   ├─ IN:  Res<CurrentPhase>               phase guard
│   │
│   ├─ OWN: ResMut<PrismState>              sole writer — no other system writes this
│   ├─ OWN: ResMut<PlayerHands>               via hand_push() shared fn
│   ├─ OWN: Query<&mut PrismPresence>       update collected bool for replication
│   │
│   ├─ OUT: S2CCardAcquired                 reliable unicast to owning player
│   ├─ OUT: S2CPrismRewardDropped           reliable unicast to owning player
│   └─ OUT: S2CPrismRespawned               reliable all-player delivery
│           via ServerMultiMessageSender::send::<M, ReliableChannel>(...)
│
└─ award_fake_objective_rewards  [Objective System]
```

### Key Interfaces

```rust
// ── Hand module (shared API — stable internal contract) ──────────────────────
// Called by: resolve_prism_draws (Prism System), Card Acquisition systems
pub fn hand_push(
    hand: &mut PlayerHands,
    player: PlayerId,
    card_id: CardId,
) -> Result<(), HandFullError>;

// ── PrismState access contract ───────────────────────────────────────────────
// Writer:  resolve_prism_draws only (ResMut<PrismState>)
// Readers: none server-side (external systems read PrismPresence via Lightyear)

// ── Network output ───────────────────────────────────────────────────────────
// S2CCardAcquired { card_id: CardId, source: AcquisitionSource::PrismLane(u8) }
// Reliable unicast to owning player:
//   ServerMultiMessageSender::send::<S2CCardAcquired, ReliableChannel>(
//       &msg, server, &NetworkTarget::Single(owner_peer_id),
//   )
// S2CPrismRewardDropped follows the same owner-only target.
// S2CPrismRespawned uses NetworkTarget::All on ReliableChannel.
// PrismPresence { collected: bool } — unreliable component replication per (player, lane).
```

---

## Alternatives Considered

### Alternative 1: Message-Based Hand Write

- **Description**: `resolve_prism_draws` writes `AddCardToHand { player, card_id }` Messages. A centralized hand-writer system (owned by Card Acquisition) drains the buffer and writes to `PlayerHands`.
- **Pros**: Prism System never holds `ResMut<PlayerHands>` — clean separation at the system level.
- **Cons**: Requires scheduling the hand-writer after `resolve_prism_draws` in the RESOLUTION set. The hand-full pre-check (GDD Rule 7) must happen at message-write time or drain time. If at write time, Prism needs read access to `PlayerHands` anyway (partially defeats isolation). If at drain time, the pre-check lives in Card Acquisition code — coupling two systems that should not know about each other. Adds a system and an ordering dependency for no gain in safety (Bevy already enforces `ResMut` exclusivity).
- **Rejection Reason**: Added scheduling complexity and hand-full coupling outweigh the isolation benefit. `hand_push()` as a shared function achieves equivalent safety via Bevy's scheduler.

### Alternative 2: Route Through Card Acquisition

- **Description**: Prism System emits `AcquireCard { player, card_id, source: PrismLane(u8), bypass_ca: true }` Messages. Card Acquisition processes all AcquireCard messages and writes `PlayerHands`.
- **Pros**: Card Acquisition remains the sole system holding `ResMut<PlayerHands>`.
- **Cons**: Card Acquisition must distinguish bypass sources and skip its own validation for prism-sourced cards. This inverts the bypass contract (GDD: "Prism rewards bypass Card Acquisition entirely") — Card Acquisition becomes aware of the Prism System. The `bypass_ca: true` flag means CA contains conditional paths based on source, not capability.
- **Rejection Reason**: Violates the explicit bypass contract in `card-acquisition.md` line 80 and the GDD. Creates implicit coupling between two feature systems that should not know about each other.

### Alternative 3: Component-Based Per-Player State

- **Description**: Prism state and hand state colocated on player entities as components (`PrismStateComponent`, `HandComponent`). Systems query by player entity, avoiding `Resource`-level contention.
- **Pros**: More idiomatic ECS design for per-player state.
- **Cons**: The established architecture (ADR-002, ADR-013, ADR-019) uses `Resource` for centralized authoritative server state (`AuctionState`, `PlayerEconomies`). Mixing component-based state for prism/hand while keeping resource-based for auction/economy creates architectural inconsistency. `PlayerHands` is already defined as a Resource in card-acquisition architecture (ADR-015) — migrating it to a Component would cascade to Card Acquisition, Economy, and Objective System.
- **Rejection Reason**: Inconsistent with established server-side state ownership patterns. Cascading refactor across multiple systems not in scope.

---

## Consequences

### Positive

- `PrismState` has a single writer. No concurrent mutation is possible — Bevy enforces `ResMut` exclusivity at the scheduler level.
- `hand_push()` is a pure function with no Bevy dependencies — unit-testable directly with a constructed `PlayerHands` without a full Bevy `World`.
- The RESOLUTION schedule slot is unambiguous: `resolve_prism_draws` sits between `resolve_ecaflip_triggers` and `award_fake_objective_rewards` with explicit `.after()` / `.before()` constraints.
- External systems (Board Rendering, HUD) depend only on `PrismPresence` component replication — they have no direct dependency on `PrismState`. The Prism System can be refactored without touching rendering or HUD code.
- `PrismPresence` entities are first-class Lightyear-replicated objects — Board Rendering gets automatic change delivery without Prism System managing client subscriptions.

### Negative

- `hand_push()` as a shared API means both Prism and Card Acquisition break simultaneously if `PlayerHands` struct changes shape. The function must be treated as a stable internal API boundary. Changes to `PlayerHands` require updating all callers.
- Ten `PrismPresence` entities spawned at session start must be despawned on `GameOverEmitted` — session cleanup must include these entities explicitly.

### Risks

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| Player-to-peer lookup is stale after reconnect | MEDIUM | Owner-only Prism messages target an old `PeerId` or are lost | Use the ADR-011 session-token identity bridge and current connection map at send time; if `snapshot_sent[player]` is false or no current `PeerId` exists, use the deferred queue path instead of sending immediately. |
| `PrismPresence` replication target is accidentally narrowed | LOW | Opponent does not see a public prism respawn/state change | PrismPresence is public board state; spawn with `Replicate::to_clients(NetworkTarget::All)`. Owner-only targeting applies to `S2CCardAcquired` and `S2CPrismRewardDropped`, not PrismPresence. |
| `MessageReader<PrismCollected>` drained by another system before `resolve_prism_draws` | LOW | Silent loss of prism collection events | Lightyear's `MessageReceiver<T>` (C2S) and Bevy's `MessageReader<T>` (internal) are distinct APIs. `PrismCollected` is server-internal — only `resolve_prism_draws` registers a `MessageReader<PrismCollected>`. Forbidden pattern below documents this. |
| Card Acquisition systems scheduled into RESOLUTION `Update` set in future M2 | LOW | Concurrent `ResMut<PlayerHands>` → Bevy panic | Document as scheduling invariant: Card Acquisition systems are DRAFT-only. Enforce via `.run_if(in_state(DraftPhase))` conditions on CA systems. |

---

## GDD Requirements Addressed

| GDD System | Requirement | How This ADR Addresses It |
|---|---|---|
| `prism-system.md` | Rule 1 — `PrismState` resource ownership | Defines `PrismState` as `#[derive(Resource, Default)]`; `ResMut<PrismState>` exclusive to `resolve_prism_draws` |
| `prism-system.md` | Rule 3 — `PrismCollected` as Bevy Message, consumed by `MessageReader` | System signature includes `MessageReader<PrismCollected>`; confirmed server-internal (not Lightyear C2S) |
| `prism-system.md` | OQ1 — Hand-write API resolution | Resolves to `hand_push()` shared module function; both Prism and Card Acquisition call it; no dual-ResMut conflict |
| `prism-system.md` | Rules 8–9 — Respawn timing (after all reward messages, within `resolve_prism_draws`) | Single-system ownership: respawn fires at end of `resolve_prism_draws` function body, after reward-message emission, per GDD Rule 9 |
| `prism-system.md` | States and Transitions — `PrismPresence` client replication | Defines 10 `PrismPresence` entities + Lightyear `Replicate` for Board Rendering |
| `prism-system.md` | AC PS-20 — `S2CCardAcquired` reliable unicast | Lightyear server reliable send via `ServerMultiMessageSender::send::<S2CCardAcquired, ReliableChannel>(&msg, server, &NetworkTarget::Single(owner_peer_id))`; channel assignment per ADR-008 |
| `server-rng.md` | Rule 5 — Schedule slot between `resolve_ecaflip_triggers` and `award_fake_objective_rewards` | Formalizes `.after(resolve_ecaflip_triggers).before(award_fake_objective_rewards)` system registration |
| `round-state-machine.md` | Rule 10 — RESOLUTION phase structure | `resolve_prism_draws` gated on `ResolutionPhaseEntered` (phase guard + scheduling); ADR-010 subscriber row to be added |
| `card-acquisition.md` | Line 80 — Prism bypasses Card Acquisition | `hand_push()` is shared module fn; Prism calls it directly without routing through CA |
| `network-protocol.md` | `PrismBoardState` in reconnect snapshot | `PrismPresence` component replication carries `collected: bool` per `(player, lane)` — reconnect snapshot can read this state (see GDD pre-implementation required fixes) |

---

## Performance Implications

- **CPU**: `resolve_prism_draws` processes at most 10 `PrismCollected` messages per RESOLUTION (5 lanes × 2 players in 1v1). Max 4 `hand_push()` calls and 2 `ServerRng::next_seed()` calls per RESOLUTION. O(n_players × lanes_collected) — negligible.
- **Memory**: `PrismState` ≈ 20 bytes (10 bools + 2 bools). 10 `PrismPresence` component entities ≈ 1 byte replicated payload each. No memory concern.
- **Load Time**: 10 entity spawns at session start — immeasurable overhead.
- **Network**: At most 10 `S2CCardAcquired` / `S2CPrismRewardDropped` messages per RESOLUTION (reliable owner-only, ~32 bytes each) + at most 2 `S2CPrismRespawned` reliable all-player messages + 10 `PrismPresence` unreliable updates. Well within the < 1 KB per-round budget (technical-preferences.md).

---

## Migration Plan

Greenfield implementation — no existing Prism System code. Implementation sequence:

1. Define `PrismState` in `server/feature/prism/state.rs`; insert in `PrismPlugin::build()`.
2. Define `hand_push()` in `server/feature/hand/mod.rs`; update Card Acquisition to call it instead of writing `PlayerHands` directly (if CA already writes directly, this is a refactor of CA — coordinate with CA epic story).
3. Spawn 10 `PrismPresence` entities at session start; add `Replicate::to_clients(NetworkTarget::All)` for public board-state replication.
4. Confirm `app.add_message::<PrismCollected>()` registration — ownership of this call belongs to the producer (Board/Lane System plugin).
5. Use the resolved Lightyear server→client targeted send API (`ServerMultiMessageSender::send::<M, ReliableChannel>(&msg, server, &NetworkTarget::...)`) when implementing `resolve_prism_draws`.
6. Implement `resolve_prism_draws` following GDD Rules 3–9, using `hand_push()`, `ServerRng::next_seed()`, and `CardDataPool::draw_random()`.
7. Register `resolve_prism_draws` with `.after(resolve_ecaflip_triggers).before(award_fake_objective_rewards)`.
8. Update ADR-010 Subscriber Contracts table: add `ResolutionPhaseEntered → Prism System` row.
9. Despawn `PrismPresence` entities on `GameOverEmitted`.

**Pre-implementation required fixes** (GDD OQ2 / OQ3 / OQ4 — must complete before Prism epic stories start):

- **Network Protocol GDD**: Add `player_id: PlayerId` to `PrismBoardState` reconnect snapshot schema (current schema cannot distinguish per-player prism state).
- **Network Protocol GDD**: Remove `GoldAwardReason::PrismReward` enum variant (prisms grant zero gold — this variant is a footgun).
- **`server-rng.md` Rule 3**: Add conditional note — "0 seeds consumed for Lane 3 if player hand is full at collection time."
- **`game-config.md` GameConfig struct**: Add `prism_strike_damage` and `prism_strike_mana_cost` fields (currently absent).

---

## Validation Criteria

- [ ] `PrismState` compiles as `#[derive(Resource, Default)]` in Bevy 0.18 with no deprecated derives.
- [ ] `hand_push()` unit test: `PlayerHands` with 9 cards → `Ok(())`; 10 cards → `Err(HandFullError)`. No Bevy `World` required.
- [ ] `resolve_prism_draws` compiles with all listed system parameters in Bevy 0.18; no `EventReader`/`EventWriter` usage; `MessageReader<PrismCollected>` drains the buffer (confirmed server-internal Bevy Message).
- [ ] Lightyear server→client targeted send API implemented with `ServerMultiMessageSender::send::<M, ReliableChannel>(&msg, server, &NetworkTarget::...)` (Verification Required item 1 resolved).
- [ ] `PrismPresence` entities replicated to correct clients: Board Rendering reads `collected: bool` for the right `(player, lane)` on the client.
- [ ] `resolve_prism_draws` does not execute outside RESOLUTION phase (AC PS-08 baseline — phase guard + run_if scheduling).
- [ ] `S2CCardAcquired` delivered on reliable channel to owning player only (not broadcast) — verifiable via Lightyear outbound queue inspection.
- [ ] `PrismPresence` entities despawned on `GameOverEmitted` — World contains 0 `PrismPresence` entities after session teardown.
- [ ] ADR-010 Subscriber Contracts table updated with `ResolutionPhaseEntered → Prism System` row before first Prism story is marked Done.

---

## Related Decisions

- [ADR-005](adr-005-server-side-rng.md) — Defines `resolve_prism_draws` by name in the RESOLUTION schedule; this ADR formalizes what that slot contains.
- [ADR-010](adr-010-rsm-event-bus.md) — `ResolutionPhaseEntered` message; Prism subscriber contract must be added to ADR-010's Subscriber Contracts table.
- [ADR-008](adr-008-lightyear-channel-config.md) — Channel assignments for `S2CCardAcquired` (reliable) and `PrismPresence` (unreliable); server→client send API reference.
- [ADR-006](adr-006-card-data-schema.md) — Static card definitions for `prism_strike` and `prism_reserve` from `assets/data/cards.json`.
- [ADR-002](adr-002-client-server-authority.md) — Server-authoritative state model; `PrismState` lives only on the server.
- `design/gdd/prism-system.md` — Primary GDD; all rules, formulas, edge cases, and acceptance criteria.
- `design/gdd/card-acquisition.md` — Bypass contract (line 80); shared `hand_push()` API.
- `design/gdd/server-rng.md` — RNG schedule slot; `draw_random` audit log format.
