# ADR-001: Hidden Objective Identity via Targeted Unicast, Not Component Replication

| Field | Value |
|---|---|
| **Status** | Accepted |
| **Date** | 2026-04-29 |
| **Deciders** | User + technical-director (spike), creative-director (fantasy validation) |
| **Affects** | Objective System, Network Protocol, Game Session System |

---

## Context

Lanes and Lies has a per-player secret: `is_fake` (whether each objective is real or counterfeit). The owning player must see their own fake/real assignments; the opponent must not — until an objective is destroyed. Both players always see current HP on all objectives.

The initial design proposed two replicated ECS components on the same entity:
- `ObjectiveHp { hp: u32 }` — replicated to both clients
- `ObjectiveIdentity { is_fake: bool }` — replicated to the owning player only

This required Lightyear 0.26 to support per-component replication scope (different components of the same entity delivered to different client sets). A technical spike was required before implementation because silent failure — Lightyear replicating `ObjectiveIdentity` to both clients with no error — would delete the bluff mechanic with no observable symptom.

**Spike findings (2026-04-29):**
- Lightyear 0.26's visibility primitives (`NetworkVisibility`, `SenderNetworkVisibility`, `ReplicateTo(NetworkTarget)`, Rooms) all operate at **entity granularity**, not component granularity.
- `DisabledComponents` is a per-entity-global filter (disables replication for all clients), not a per-client-per-component scope.
- No first-class per-component-per-client replication scope API exists in 0.26. The 0.26 release focused on Bevy 0.18 upgrade and `LocalTimeline` refactor — no new component-level visibility was introduced.
- Workaround (split secret onto a separate entity + `ReplicationGroup`) is technically possible but reintroduces the silent-leak failure mode and adds cross-entity coupling that every contributor must maintain.
- `Server::send_message_to_target` + `NetworkTarget::Single(client_id)` (targeted unicast) is the documented, idiomatic library path for "data intended for exactly one client."

---

## Decision

**Do NOT replicate `ObjectiveIdentity` as an ECS component.**

Instead:

1. `ObjectiveHp { hp: u32 }` remains a replicated ECS component — broadcast to both clients on every change. This is visible public state.

2. `ObjectiveIdentity { is_fake: bool }` is **never inserted into the replication graph**. The server holds it in a non-replicated server-side resource (`HiddenObjectives` or equivalent).

3. At `DRAFT_INITIAL`, after fake lane assignment, the server sends a targeted reliable unicast per player:

```rust
// Shared protocol
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct S2CObjectiveIdentities {
    pub identities: Vec<(LaneId, bool)>,  // (lane_id, is_fake)
}

// Server-side dispatch — once per player at DRAFT_INITIAL
server.send_message_to_target::<ReliableChannel, S2CObjectiveIdentities>(
    S2CObjectiveIdentities { identities: player_identities },
    NetworkTarget::Single(owner_client_id),
);
```

4. On reconnect / late-join, the server re-sends `S2CObjectiveIdentities` to the reconnecting client as part of the session resume handshake. This must be explicitly handled — reliable message delivery guarantees in-session delivery but not reconnect replay.

5. `Sang Méprise` (Sacrier Krosmic spell): when active, the server sends an additional targeted unicast `S2CSangMepriseReveal { identities: Vec<(LaneId, bool)> }` to the **opponent** only. This is a one-shot reveal that persists in client local state for the RESOLUTION duration. The same unicast pattern; no replication scope changes. See Objective System GDD OQ5 and Sang Méprise edge case.

---

## Consequences

**Positive:**
- Privacy is enforced at the message-routing boundary (`NetworkTarget::Single`), not via fragile per-component visibility flags. A mis-route produces the wrong recipient — it does not silently broadcast to everyone.
- Single source of truth on the server: `HiddenObjectives` resource holds authoritative identity. Clients hold view-only projections received at session start.
- Zero per-tick replication cost for identity: sent once at `DRAFT_INITIAL` and on reconnect.
- Testable without two-client network infrastructure: assert at the protocol level that `S2CObjectiveIdentities` was dispatched to client A's ID only, not client B's.
- Self-documenting: "send the secret to the owner" reads as intent. The entity-split workaround does not.

**Negative / Tradeoffs:**
- Two pathways for objective data exist: replicated HP (continuous, unreliable) and unicast identity (one-shot, reliable). This must be documented clearly in the Network Protocol GDD so contributors do not conflate them.
- Reconnect handling must explicitly re-send `S2CObjectiveIdentities` — it is not automatic. The Network Protocol GDD must specify this in the reconnect/snapshot path.
- `ObjectiveIdentity` cannot be queried via ECS on the client. Client code must cache the received message in a local resource. All client systems that need fake/real identity read from this cache, not from a component.

---

## Alternatives Considered

**A — Per-component replication scope on a single entity (rejected):**
Not supported as a first-class Lightyear 0.26 API. The entity-split workaround (separate secret entity + `ReplicationGroup`) risks a silent leak on any missed visibility update — the highest-severity failure for a bluff game. Rejected.

**B — Targeted unicast message (chosen):**
Library-idiomatic, matches card/bluff game precedent (Hearthstone, Among Us), fails loudly if mis-routed. Negligible network cost.

**C — Encrypted payload broadcast (rejected):**
Broadcast all identity data encrypted with the owner's session key; only the owner can decrypt. Adds key-management complexity, session key distribution, and WASM crypto overhead for zero win over server-authoritative message routing. Rejected.

---

## Implementation Notes

- Exact symbol for single-client target: confirm `NetworkTarget::Single(ClientId)` vs `NetworkTarget::Only(vec![client_id])` against `docs.rs/lightyear/0.26.x` before implementing. Both shapes have appeared across Lightyear versions.
- Channel type: use a reliable, ordered channel. Identity data must not be dropped and must arrive before `DRAFT_INITIAL` placement input is accepted.
- `S2CObjectiveIdentities` payload is tiny (~6 bytes per player at 5 lanes + header). No bandwidth concern.
- `HiddenObjectives` server resource should be wiped and re-populated at each new session, not carried across sessions.

---

## GDD Impact

| Document | Required Update |
|---|---|
| `design/gdd/objective-system.md` | OQ4 resolved — replace BLOCKING spike note with reference to this ADR |
| `design/gdd/network-protocol.md` | Add `S2CObjectiveIdentities` message to protocol definition; add reconnect re-send requirement |
| `design/gdd/game-session-system.md` | Note that session resume must trigger `S2CObjectiveIdentities` re-send |
