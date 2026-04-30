# Story 007: ResolutionPhaseEntered Subscription & RESOLUTION-end Sync

> **Epic**: Objective System
> **Status**: Ready
> **Layer**: Feature
> **Type**: Integration
> **Manifest Version**: 2026-04-29

## Context

**GDD**: `design/gdd/objective-system.md`
**Requirement**: `TR-OBJ-008` — `ObjectiveDestroyed` broadcast at RESOLUTION-end sync (NOT mid-sub-step); 500ms minimum reveal hold (client concern); `Sang Méprise`: `S2CSangMepriseReveal` reliable unicast to opponent only; `ObjectiveDestroyed` fires regardless of Sang Méprise visibility

**ADR Governing Implementation**: [ADR-010: RSM Phase Event Bus](docs/architecture/adr-010-rsm-event-bus.md) (`ResolutionPhaseEntered` subscription pattern); [ADR-001](docs/architecture/adr-001-objective-identity-unicast.md) (`S2CSangMepriseReveal` unicast)

**ADR Decision Summary (ADR-010)**: The Objective System subscribes to `ResolutionPhaseEntered` via `MessageReader<ResolutionPhaseEntered>` — NOT `EventReader`. At RESOLUTION-end (after all sub-steps complete — gated by `ResolutionComplete` signal in M2; for M1 testing, manually triggered), the queued `PendingObjectiveEvents` are broadcast to both clients as `ObjectiveDestroyed` messages in ascending lane order. Sang Méprise is a separate unicast sent during RESOLUTION to the opponent only — the same `NetworkTarget::Single(PeerId)` pattern as ADR-001. `ObjectiveDestroyed` fires at RESOLUTION-end regardless of whether Sang Méprise was active.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**: `MessageReader<ResolutionPhaseEntered>::read()` — `EventReader` does not exist in Bevy 0.17+. Subscribe via `app.add_message::<ResolutionPhaseEntered>()` in the Objective System plugin; schedule the subscriber system `.after(advance_phase)`. `ServerMultiMessageSender` for Sang Méprise unicast (same API as Story 003 — verify checklist items 7 and 9 before implementing). `ObjectiveDestroyed` broadcast: use `NetworkTarget::All`. For M1, `ResolutionComplete` (M2) is not yet available — the integration test triggers RESOLUTION-end manually. `liv-bevy-018` and `liv-bevy-lightyear` both mandatory for this story.

**Control Manifest Rules (Feature layer)**:
- Required: Subscribe to `ResolutionPhaseEntered` via `MessageReader<T>`, never by polling `RoundState` directly (ADR-010)
- Required: Feature systems never import from `server/core/rsm/` directly — subscribe to Messages only (ADR-010)
- Required: `Sang Méprise` reveal sent as one-shot reliable unicast `S2CSangMepriseReveal` to opponent only; reveal persists in client local state for RESOLUTION duration (ADR-001)
- Required: `ObjectiveDestroyed` broadcast fires at RESOLUTION-end regardless of prior Sang Méprise visibility (GDD OS-24)
- Forbidden: Never emit `ObjectiveDestroyed` mid-sub-step — it is queued during sub-steps and broadcast at sync point (GDD Rule 6)
- Guardrail: 500ms minimum reveal hold between HP = 0 and `was_fake` shown is a client/Board Rendering responsibility, not enforced server-side

---

## Acceptance Criteria

*From GDD `design/gdd/objective-system.md`, scoped to this story:*

- [ ] OS-13a (BLOCKING): GIVEN an objective is destroyed by the opponent, WHEN the RESOLUTION-end sync fires, THEN `ObjectiveDestroyed { target_player_id, lane, was_fake: bool }` is broadcast to both clients with the correct payload — NOT emitted during the sub-step when damage was applied. (Integration scope: tests batch emission timing, not just queuing. Unit test in Story 005 covered queuing; this story covers the broadcast moment.)
- [ ] OS-18a (BLOCKING): GIVEN multiple objectives destroyed in the same RESOLUTION, WHEN `ObjectiveDestroyed` events are broadcast at RESOLUTION-end sync, THEN events are broadcast in ascending lane order.
- [ ] OS-18b (ADVISORY): GIVEN two `take_damage()` calls targeting the same objective in sub-step 6, WHEN both calls are processed, THEN only one HP replication update is visible to clients (not two intermediate values). (ADVISORY — Lightyear batching is a transport property, not asserted in ECS unit test. Verified via network integration test once Lightyear session is available.)
- [ ] OS-24 (BLOCKING): GIVEN Sang Méprise is active during a RESOLUTION in which a fake objective is also destroyed, WHEN the consequence path executes and RESOLUTION-end sync fires, THEN `ObjectiveDestroyed { was_fake: true }` is broadcast to both clients exactly once — Sang Méprise visibility does not suppress or duplicate the authoritative destruction event.

---

## Implementation Notes

*Derived from ADR-010 and ADR-001 Implementation Guidelines:*

**ResolutionPhaseEntered subscription** — add to `ObjectivePlugin::build()`:

```rust
app.add_message::<ResolutionPhaseEntered>();  // already registered by RSM — skip if duplicate
// Schedule the objective resolver after advance_phase:
app.add_systems(Update, objective_resolution_ready.after(advance_phase));
```

The `objective_resolution_ready` system reads `ResolutionPhaseEntered` and prepares the objective system to accept `take_damage()` calls during RESOLUTION sub-steps. In M1 (without Combat Resolution), this system just ensures the Objective System is in the correct state.

**RESOLUTION-end broadcast** — implement `broadcast_objective_events` system:

```rust
fn broadcast_objective_events(
    mut pending: ResMut<PendingObjectiveEvents>,
    mut sender: ServerMultiMessageSender,
    server: Res<Server>,
) {
    // Sort by lane ascending before broadcasting
    pending.queue.sort_by_key(|e| e.lane.0);

    for event in pending.queue.drain(..) {
        sender.send::<ObjectiveDestroyed, ReliableChannel>(
            &event,
            &server,
            &NetworkTarget::All,
        );
    }
}
```

This system runs at RESOLUTION-end — in M2, gated by `ResolutionComplete`. In M1 integration tests, trigger it manually after all `take_damage()` calls are complete.

**Sang Méprise unicast** — `S2CSangMepriseReveal` follows the same ADR-001 pattern as `S2CObjectiveIdentities`:

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct S2CSangMepriseReveal {
    pub identities: Vec<(LaneId, bool)>,  // all 5 lanes: (lane, is_fake) for defending player
}

// Sent to opponent only when Sang Méprise spell is triggered:
sender.send::<S2CSangMepriseReveal, ReliableChannel>(
    &S2CSangMepriseReveal { identities },
    &server,
    &NetworkTarget::Single(opponent_peer_id),
);
```

`ObjectiveDestroyed` fires at RESOLUTION-end regardless of Sang Méprise state (OS-24). The client's Board Rendering GDD is responsible for suppressing the "surprise reveal" animation if Sang Méprise was active — the server makes no behavioral change.

Clear `PendingObjectiveEvents.queue` after each RESOLUTION broadcast, before the next PLACEMENT phase opens.

Reconnect gap for Sang Méprise (open issue from GDD OQ5): `S2CSangMepriseReveal` is a one-shot and is not included in `S2CGameSnapshot`. A player reconnecting mid-RESOLUTION after this message fired will not receive the revealed identities. This is a known limitation documented in the GDD — do not attempt to patch it in this story.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- [Story 005]: Queuing `ObjectiveDestroyed` events during sub-steps (this story broadcasts the already-queued events)
- [Story 003]: `S2CObjectiveIdentities` at DRAFT_INITIAL (Sang Méprise uses the same pattern but is a different message type)

---

## QA Test Cases

*Written by qa-lead at story creation. Implement against these — do not invent new test cases.*

- **OS-13a** (broadcast timing): `ObjectiveDestroyed` broadcast only at RESOLUTION-end, not during sub-step
  - Given: Minimal integration test: `World` with `PendingObjectiveEvents` containing 1 queued `ObjectiveDestroyed`; `broadcast_objective_events` system not yet run
  - When: System runs (simulating RESOLUTION-end)
  - Then: `assert_eq!(pending_events.queue.len(), 0)` (queue drained); `assert_eq!(captured_broadcasts.len(), 1)` (one message dispatched); message has correct `target_player_id`, `lane`, `was_fake`
  - Edge cases: Empty queue (no destructions this RESOLUTION) → zero broadcasts

- **OS-18a** (broadcast order): Multiple `ObjectiveDestroyed` broadcast in ascending lane order
  - Given: `PendingObjectiveEvents` queue contains events for lanes 3, 1, 5 (insertion order arbitrary)
  - When: `broadcast_objective_events` runs
  - Then: Broadcast order: lane 1 first, lane 3 second, lane 5 third (`assert_eq!(broadcasts[0].lane, LaneId(1))`)

- **OS-18b** (ADVISORY): Single HP replication update per objective per sub-step
  - Setup: Start two-client Lightyear session; apply two sequential `take_damage(5)` to same objective (HP: 3 → 0)
  - Verify: Client B receives one `ObjectiveHp` update (HP = 0), not an intermediate update (HP = 0 twice or HP = -2)
  - Pass condition: Client B's `ObjectiveHp` for that entity changes exactly once (3 → 0)

- **OS-24**: Sang Méprise + destruction → `ObjectiveDestroyed` fires exactly once
  - Given: `PendingObjectiveEvents` queue contains one `ObjectiveDestroyed { was_fake: true }` event; a `SangMepriseActive` resource is set to true (simulating active Sang Méprise)
  - When: `broadcast_objective_events` runs
  - Then: `assert_eq!(captured_broadcasts.len(), 1)`; `assert_eq!(captured_broadcasts[0].was_fake, true)`; no duplicate events emitted
  - Edge cases: No active Sang Méprise → same result (always fires once)

---

## Test Evidence

**Story Type**: Integration
**Required evidence**: `tests/integration/objective/resolution_sync_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 005 must be DONE (`PendingObjectiveEvents` is populated by the consequence path); Story 006 should be DONE (complete reward draw logic before RESOLUTION-end sync is implemented); Story 003 DONE (Sang Méprise uses the same unicast pattern — familiarity with the Lightyear API)
- Unlocks: This is the last story in the Objective System epic — all TR-OBJ-001 through TR-OBJ-010 requirements are satisfied when this story is Done
