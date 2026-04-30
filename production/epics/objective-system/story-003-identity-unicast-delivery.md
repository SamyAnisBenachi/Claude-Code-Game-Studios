# Story 003: Identity Unicast Delivery

> **Epic**: Objective System
> **Status**: Ready
> **Layer**: Feature
> **Type**: Integration
> **Manifest Version**: 2026-04-29

## Context

**GDD**: `design/gdd/objective-system.md`
**Requirement**: `TR-OBJ-007` — `ObjectiveHp` is a replicated ECS component broadcast to both clients; `ObjectiveIdentity` is server-only in `HiddenObjectives` Resource and NEVER replicated; identity is delivered via reliable unicast `S2CObjectiveIdentities` at DRAFT_INITIAL and re-sent on every reconnect.

**ADR Governing Implementation**: [ADR-001: Hidden Objective Identity via Targeted Unicast, Not Component Replication](docs/architecture/adr-001-objective-identity-unicast.md)

**ADR Decision Summary**: After fake lane assignment at DRAFT_INITIAL, the server sends `S2CObjectiveIdentities { identities: Vec<(LaneId, bool)> }` as a reliable unicast per player — only to the owning player's `PeerId`, never broadcast. On reconnect, the server re-sends this message as part of the session resume handshake. The `HiddenObjectives` Resource is the single source of truth; the client caches the received message in a local resource.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**: Lightyear 0.26 unicast API (checklist item 9, ⚠️ DIFFERS from older patterns): `ServerMultiMessageSender` system param — `send::<M, C>(&msg, &server, &NetworkTarget)` where M = message type, C = channel type. `NetworkTarget::Single(PeerId)` confirmed (item 7, note: uses `PeerId` not the older `ClientId`). Channel: `ReliableChannel` — identity data must not be dropped and must arrive before any placement input is accepted. Verify `ServerMultiMessageSender` import path against `docs.rs/lightyear/0.26` before implementing. Items 15 and 16 (PENDING CI): verify `Res<T>` visibility through Commands before implementing the reconnect path.

**Control Manifest Rules (Feature layer)**:
- Required: Send `S2CObjectiveIdentities` as reliable unicast to each player immediately after fake assignment at DRAFT_INITIAL (ADR-001)
- Required: Re-send `S2CObjectiveIdentities` on every reconnect; reliable delivery not guaranteed across transport reconnects (ADR-001)
- Required: Live messages to reconnecting player queued until `snapshot_sent[player] = true` — check `ReconnectTracker.snapshot_sent[player]` before enqueuing unicast (ADR-011)
- Forbidden: Never replicate `ObjectiveIdentity` as an ECS component (ADR-001)
- Forbidden: Never send opponent `is_fake` values in any broadcast message (ADR-001)

---

## Acceptance Criteria

*From GDD `design/gdd/objective-system.md`, scoped to this story:*

- [ ] OS-17 (ADVISORY): GIVEN two connected clients (owner and attacker), WHEN the attacker's state for an opponent's intact objective is queried, THEN `ObjectiveHp` is present AND the attacker does NOT receive `S2CObjectiveIdentities` for the opponent's lanes. (ADVISORY — two-client integration test requiring a live Lightyear session, not a `World::new()` unit test. Pre-sprint decision required: if a pure-function unit test asserting message dispatch per-PeerId is feasible, it becomes BLOCKING; otherwise ADVISORY with manual evidence document.)

---

## Implementation Notes

*Derived from ADR-001 Implementation Guidelines:*

Define `S2CObjectiveIdentities` in `shared/src/protocol.rs`:

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct S2CObjectiveIdentities {
    pub identities: Vec<(LaneId, bool)>,  // (lane_id, is_fake)
}
```

Register the message type: `app.register_message::<S2CObjectiveIdentities>().add_direction(NetworkDirection::ServerToClient)`.

Send after fake lane assignment completes at DRAFT_INITIAL (one send per player, in ascending `player_id` order, immediately after `assign_fake_objectives` returns):

```rust
// For each player, build their own identity view from HiddenObjectives
let identities = (1..=5).map(|lane| {
    let lane_id = LaneId(lane);
    let is_fake = hidden.identities[&(player, lane_id)];
    (lane_id, is_fake)
}).collect();

multi_sender.send::<S2CObjectiveIdentities, ReliableChannel>(
    &S2CObjectiveIdentities { identities },
    &server,
    &NetworkTarget::Single(player_peer_id),
);
```

On reconnect: after `S2CGameSnapshot` is sent and `snapshot_sent[player] = true`, re-send `S2CObjectiveIdentities` for the reconnecting player's own lanes from `HiddenObjectives`. Per ADR-011: systems sending unicast S2C must check `ReconnectTracker.snapshot_sent[player]` before enqueuing.

Confirm exact `NetworkTarget::Single` vs `NetworkTarget::Only` spelling against `docs/engine-reference/lightyear/` before implementing — checklist item 7 shows a DIFFERS flag on this item.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- [Story 002]: Fake lane assignment (populating `HiddenObjectives`)
- [Story 007]: `S2CSangMepriseReveal` — a separate unicast using the same pattern but for the Sang Méprise spell during RESOLUTION

---

## QA Test Cases

- **OS-17** (ADVISORY): Owner receives their own fake/real assignments; attacker does not
  - Setup: Start a two-client Lightyear test session; complete DRAFT_INITIAL
  - Verify: Player A's client has received `S2CObjectiveIdentities` containing 5 entries (their own lanes); Player B has NOT received any message that discloses Player A's `is_fake` values
  - Pass condition: Capture outbound messages from server per-PeerId; assert exactly one `S2CObjectiveIdentities` per player sent to their own PeerId only; assert no broadcast `S2CObjectiveIdentities` message exists

---

## Test Evidence

**Story Type**: Integration
**Required evidence**: `tests/integration/objective/identity_unicast_test.rs` OR `production/qa/evidence/identity-unicast-evidence.md` with manual walkthrough

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 002 must be DONE (`HiddenObjectives` must be populated before delivery)
- Unlocks: Story 007 (Sang Méprise uses the same unicast pattern)
