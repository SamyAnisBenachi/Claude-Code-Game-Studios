# Story 004: Hand-Full Rejection and Network Message Staging

> **Epic**: Prism System
> **Status**: Ready
> **Layer**: Feature (M3)
> **Type**: Integration
> **Manifest Version**: 2026-04-30

## Context

**GDD**: `design/gdd/prism-system.md`
**Requirement**: `TR-PRI-004`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-016: Prism System Architecture — State Ownership, Schedule Slot, and Hand-Write API; ADR-008: Lightyear Channel Config
**ADR Decision Summary**: When `hand_push()` returns `Err(HandFullError)` for Lanes 1/2/4/5, stage `S2CPrismRewardDropped { player_id, lane }` as a reliable unicast to the owning player — NOT a broadcast. When `hand_push()` succeeds for any lane, stage `S2CCardAcquired { card_id, source: PrismLane(L) }` as a reliable unicast to the owning player. Both messages are on `ReliableChannel`. They are independently staged — one successful lane and one hand-full lane in the same RESOLUTION produces one of each. Lane 3 hand-full does NOT emit `S2CPrismRewardDropped` (GDD Rule 7: reward was never materialized).

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**:
- Lightyear 0.26 server→client unicast API: `server.send_message_to_target::<ReliableChannel, T>(msg, NetworkTarget::Single(peer_id))` — exact API must be verified against `docs.rs/lightyear/0.26` before implementation (ADR-016 Verification Required item 1; ADR-008 checklist item 9)
- `NetworkTarget::Single(PeerId)` — confirmed in ADR-008 checklist item 7 (⚠️ DIFFERS — `PeerId` not `ClientId`; resolve via `liv-bevy-lightyear` skill)
- `snapshot_sent[player]` gate: before any unicast S2C, check if snapshot has been sent; if not, push to `deferred_queue[player]` instead (ADR-008, ADR-011 requirement)
- If Lightyear 0.26 does not expose a unit-testable outbound buffer in `World::new()` tests, this story's ACs reclassify as Integration requiring `App::new()` test — confirm approach per OQ1 resolution in prism-system.md

**Control Manifest Rules (Feature layer — from ADR-008, ADR-011):**
- Required: `S2CCardAcquired` → `ReliableChannel`, reliable unicast to owning player only
- Required: `S2CPrismRewardDropped` → `ReliableChannel`, reliable unicast to owning player only (Lanes 1/2/4/5 only — not Lane 3 hand-full)
- Required: `snapshot_sent[player]` check before every unicast S2C enqueue
- Forbidden: Broadcast for `S2CCardAcquired` or `S2CPrismRewardDropped` — unicast only
- Forbidden: `S2CPrismRewardDropped` emitted for Lane 3 hand-full (GDD Rule 7: Lane 3 hand-full is silent)

---

## Acceptance Criteria

*From GDD `design/gdd/prism-system.md`, scoped to this story:*

- [ ] **PS-09** — GIVEN a player's hand contains exactly 10 cards AND the player's unit collects a prism in Lane 1, 2, 4, or 5, WHEN `resolve_prism_draws` attempts to add the spell card, THEN: (a) the spell card is NOT added to hand, (b) the prism token IS marked collected (`collected[lane][player] == true`), (c) no replacement reward is granted, AND (d) exactly one `S2CPrismRewardDropped { player_id, lane }` message is staged for reliable unicast to the owning player.
- [ ] **PS-20** — GIVEN a player's unit collects a valid prism in any lane (hand not full), WHEN `resolve_prism_draws` processes the collection, THEN exactly one `S2CCardAcquired { card_id: [lane-appropriate card ID], source: PrismLane{L} }` message is staged for reliable unicast to the owning player. (Note: if Lightyear 0.26 outbound buffer is not accessible in `World`-based tests, this AC reclassifies as Integration requiring `App::new()` — see Implementation Notes.)
- [ ] **PS-23** — GIVEN a player collects two prisms in the same RESOLUTION: one where the hand is not full (reward succeeds) and one where the hand is full (Lanes 1/2/4/5 only), WHEN `resolve_prism_draws` processes both, THEN the reliable-unicast staging buffer holds one `S2CCardAcquired` entry and one `S2CPrismRewardDropped` entry for that player — confirming both message types are independently staged and not conflated.

---

## Implementation Notes

*Derived from ADR-016 Decision and GDD Rule 7:*

Replace stub comments in Story 002's `hand_push()` call sites with actual staging:

```rust
// Lanes 1/2/4/5 hand_push result handling:
match hand_push(&mut hands, player, card_id) {
    Ok(()) => {
        // PS-20: S2CCardAcquired — reliable unicast to owning player
        // Check snapshot_sent gate before enqueuing (ADR-008 / ADR-011)
        if snapshot_sent[player] {
            server.send_message_to_target::<ReliableChannel, S2CCardAcquired>(
                S2CCardAcquired { card_id, source: AcquisitionSource::PrismLane(lane) },
                NetworkTarget::Single(peer_id_of(player)),
            );
        } else {
            deferred_queue[player].push(DeferredMessage::CardAcquired { card_id, source: ... });
        }
    }
    Err(HandFullError) => {
        // PS-09: S2CPrismRewardDropped — reliable unicast, Lanes 1/2/4/5 ONLY
        // collected[lane][player] is already true (set before hand_push call)
        if snapshot_sent[player] {
            server.send_message_to_target::<ReliableChannel, S2CPrismRewardDropped>(
                S2CPrismRewardDropped { player_id: player, lane },
                NetworkTarget::Single(peer_id_of(player)),
            );
        } else {
            deferred_queue[player].push(DeferredMessage::PrismRewardDropped { player_id: player, lane });
        }
    }
}

// Lane 3 hand_push result (only on Some(card_id) path from Story 003):
match hand_push(&mut hands, player, card_id) {
    Ok(()) => {
        // S2CCardAcquired — same as above
    }
    Err(HandFullError) => {
        // TOCTOU edge: no S2CPrismRewardDropped for Lane 3 (GDD Rule 7)
        // This path is theoretically unreachable if the pre-check at step 1 is correct,
        // but handle defensively: log warn, no message staged.
    }
}
```

**`S2CPrismRespawned` registration note**: Both `S2CPrismRewardDropped` and `S2CPrismRespawned` must be registered in `network-protocol.md` before this story is marked Done (pre-implementation gate — EPIC.md NP GDD row). Registration is a documentation task; the messages can be implemented before the doc update but not marked Done until the doc is updated.

**Lightyear server→client send API**: The exact server-side send API (likely `ServerMultiMessageSender` system param per ADR-008 checklist item 9) must be verified against `docs.rs/lightyear/0.26` before writing this code. Use `liv-bevy-lightyear` skill when implementing.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- Story 002: The `hand_push()` call itself and `collected` state mutation
- Story 003: Lane 3 hand-full (no seed, no card, no message)
- Story 005: `S2CPrismRespawned` staging (respawn network message)
- Story 001: `snapshot_sent` / `deferred_queue` infrastructure (these live in the reconnect/network layer established by ADR-011 — use existing infrastructure)

---

## QA Test Cases

- **PS-09**: Hand full (Lanes 1/2/4/5) → S2CPrismRewardDropped, no card
  - Given: `hands.len(player_a) == 10`; `collected[0][player_a] = false`; `PrismCollected(player_a, lane=1)` in buffer
  - When: `resolve_prism_draws` processes
  - Then: `hands.len(player_a) == 10` (unchanged); `collected[0][player_a] == true`; outbound reliable buffer for `player_a` contains `S2CPrismRewardDropped { player_id: player_a, lane: 1 }`; no `S2CCardAcquired` for this collection
  - Edge cases: all lanes 1/2/4/5 simultaneously hand-full for a player → 4 `S2CPrismRewardDropped` entries staged (one per lane); Lane 3 hand-full in same RESOLUTION → NO `S2CPrismRewardDropped` for Lane 3 (confirmed absent in buffer)

- **PS-20**: Successful collection → S2CCardAcquired unicast staged
  - Given: `hands.len(player_a) < 10`; valid `PrismCollected(player_a, lane=2)` in buffer
  - When: `resolve_prism_draws` processes
  - Then: outbound reliable message buffer for `player_a` contains `S2CCardAcquired { card_id: prism_reserve_id, source: PrismLane(2) }`; no broadcast (unicast only — player_b's outbound buffer does NOT contain this message)
  - Edge cases: `player_b` also collects a prism → each player's `S2CCardAcquired` is in their own respective outbound buffer only

- **PS-23**: Both message types staged independently in same RESOLUTION
  - Given: `player_a` has 9 cards; two `PrismCollected` events for `player_a`: `lane=1` (processed first → hand 9→10) and `lane=5` (processed second → hand now at 10 → full)
  - When: `resolve_prism_draws` processes lane_1 then lane_5 (ascending lane order)
  - Then: outbound buffer for `player_a` contains: `S2CCardAcquired { source: PrismLane(1) }` AND `S2CPrismRewardDropped { lane: 5 }`; both are independently present; neither is missing or overwritten; buffer has exactly 2 entries for `player_a`

---

## Test Evidence

**Story Type**: Integration
**Required evidence**: `tests/integration/prism/hand_full_network_test.rs` — must exist and pass
*(If Lightyear 0.26 outbound buffer is accessible in `World::new()`, test can use that. Otherwise, use `App::new()` with the Lightyear plugin; use `liv-bevy-lightyear` skill to confirm the correct test approach for outbound message inspection.)*

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 002 (`deterministic-lanes`) must be Done — `hand_push()` call sites in `resolve_prism_draws` must exist
- Depends on: Story 003 (`lane3-rng`) must be Done — Lane 3 `hand_push()` call site in `resolve_prism_draws` must exist
- Depends on: Pre-implementation gate NP OQ1 — Lightyear 0.26 server→client unicast API verified (ADR-016)
- Depends on: Pre-implementation gate NP GDD — `S2CPrismRewardDropped` registered in `network-protocol.md`
- Unlocks: None from within Prism epic — this is the networking completion story; Story 005 (respawn) can proceed independently
