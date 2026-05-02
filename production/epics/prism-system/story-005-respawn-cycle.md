# Story 005: Full-Set Respawn Cycle and Multi-Player Independence

> **Epic**: Prism System
> **Status**: Ready
> **Layer**: Feature (M3)
> **Type**: Logic
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/prism-system.md`
**Requirements**: `TR-PRI-005`, `TR-PRI-006`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-016: Prism System Architecture — State Ownership, Schedule Slot, and Hand-Write API
**ADR Decision Summary**: After all `PrismCollected` events are processed in Rule 6 order, `resolve_prism_draws` runs a respawn check (GDD Rule 8): for each player, if all 5 `collected[lane][player]` are `true`, set `pending_respawn[player] = true`. After all reward messages are complete, fire respawn (Rule 9): reset all 5 lanes to `false` and emit `S2CPrismRespawned { player_id }` on `ReliableChannel` to all connected players via `NetworkTarget::All`. Respawn fires AFTER all reward messages — newly respawned prisms cannot be collected in the same RESOLUTION. Each player's respawn cycle is fully independent (keyed on `player_id`, not team or session).

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**:
- `S2CPrismRespawned { player_id: PlayerId }` — reliable all-player delivery via `ServerMultiMessageSender::send::<S2CPrismRespawned, ReliableChannel>(&msg, server, &NetworkTarget::All)` (per GDD Rule 9: both players receive the notification, unlike `S2CCardAcquired` which is owning-player-only)
- Lightyear 0.26.4 `NetworkTarget::All` verified; `S2CPrismRespawned` is registered in `network-protocol.md` and covered by NP-56
- `PrismPresence` component entities for all 5 lanes of the respawned player must have their `collected` field set to `false` after `PrismState` reset — Lightyear picks up the component change for client replication on next frame

**Control Manifest Rules (Feature layer — from ADR-016):**
- Required: Rule 8 (respawn detection) runs AFTER all reward messages in the same RESOLUTION — pending_respawn is a transient flag, not a persisted state
- Required: Rule 9 (respawn fire) clears `pending_respawn[player]` to `false` after mutating `collected[]`
- Required: `S2CPrismRespawned` sent to BOTH players (not unicast to respawning player only)
- Required: Each player's respawn cycle checked independently — `count(collected[lane][p] == true) == 5` per player
- Forbidden: Any resource reward (card, gold, mana) emitted during respawn — respawn is a pure state reset

---

## Acceptance Criteria

*From GDD `design/gdd/prism-system.md`, scoped to this story:*

- [ ] **PS-05** — GIVEN a player has collected all 5 of their own prism tokens (in same or across multiple RESOLUTIONs), WHEN `resolve_prism_draws` finishes delivering all reward messages for that RESOLUTION, THEN all 5 prism tokens for that player reset to `collected = false` at end of `resolve_prism_draws`, and the opponent's prism state is unchanged.
- [ ] **PS-13** — GIVEN a player collects their 5th prism token in RESOLUTION N, WHEN `resolve_prism_draws` completes all reward messages for RESOLUTION N, THEN the full respawn (all 5 tokens reset to `collected = false`) occurs AFTER the last reward message — any unit at a spawn cell in that same RESOLUTION does NOT collect the freshly respawned token within RESOLUTION N.
- [ ] **PS-14** — GIVEN a player's prism set respawns after full collection, WHEN the respawn state is inspected, THEN no additional reward (card, gold, mana, or otherwise) is granted by the respawn event itself — it is a state reset only.
- [ ] **PS-16** — GIVEN a 2v2 game where Player A and Player B are on the same team, WHEN Player A collects the Lane 3 prism keyed on `(player_A_id, lane_3)`, THEN Player B's prism token at `(player_B_id, lane_3)` is unaffected, and Player A's respawn cycle runs on Player A's individual count (0–5) independently.
- [ ] **PS-21** — GIVEN Player A collects their 5th prism (triggering full respawn) AND Player B has collected 3 of 5 in the same RESOLUTION, WHEN `resolve_prism_draws` completes, THEN Player A's prisms all reset to uncollected, Player B retains 3 collected, and Player B's respawn does not trigger.
- [ ] **PS-24** — GIVEN a 2v2 game where Player A and Player B are on the same team AND both have units that end sub-step 5 at their shared spawn cell of the same lane, WHEN RESOLUTION sub-step 5 completes, THEN two distinct `PrismCollected` messages are emitted (one per player), two distinct rewards are delivered (each unicast to its owning player), and both `collected[lane][player_A]` and `collected[lane][player_B]` are set to `true` independently.

---

## Implementation Notes

*Derived from ADR-016 and GDD Rules 8–9:*

Add to the end of `resolve_prism_draws`, after the reward delivery loop:

```rust
// Rule 8: respawn detection — runs AFTER all reward messages
for player in all_players() {
    let idx = player_idx(player);
    if prism_state.collected[idx].iter().all(|&c| c) {
        prism_state.pending_respawn[idx] = true;
    }
}

// Rule 9: respawn fire — after all reward messages
for player in all_players() {
    let idx = player_idx(player);
    if prism_state.pending_respawn[idx] {
        // Reset all 5 lanes
        prism_state.collected[idx] = [false; 5];
        prism_state.pending_respawn[idx] = false;

        // Update PrismPresence entities for this player (Lightyear replication)
        for (key, mut presence) in prism_presence.iter_mut() {
            if key.player == player {
                presence.collected = false;
            }
        }

        // Emit S2CPrismRespawned to all connected players (not owning player only)
        s2c_sender.send::<S2CPrismRespawned, ReliableChannel>(
            &S2CPrismRespawned { player_id: player },
            server,
            &NetworkTarget::All,
        );
    }
}
```

**PS-13 timing guarantee**: `pending_respawn` is set AFTER the reward loop completes. Respawn mutation (`collected = false`) fires AFTER that. Board/Lane System's sub-step 5 has already run and all `PrismCollected` messages for this RESOLUTION have been drained — no new `PrismCollected` can arrive for the just-respawned prisms within this RESOLUTION. This is structural, not a runtime guard.

**PS-24 multi-player note**: In 2v2, each player has their own prism state keyed on `(player_id, lane)`. When two teammates have units at the same lane's spawn cells simultaneously, Board/Lane System emits `PrismCollected(player_a, lane)` and `PrismCollected(player_b, lane)` as distinct events. `resolve_prism_draws` processes them independently (sorted by ascending `player_id`). Each player's `collected[lane-1][player]` is set to `true` independently. This requires verifying that Board/Lane System actually emits two distinct events — the test exercises the Prism System side only.

**`S2CPrismRespawned` registration**: Registered in `network-protocol.md`; the NP GDD pre-implementation gate is resolved.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- Story 002: Initial `collected` state mutations from lane rewards
- Story 003: Lane 3 RNG draws (prerequisite for full respawn cycles in tests)
- Story 004: `S2CCardAcquired` and `S2CPrismRewardDropped` network staging (rewards that precede respawn)
- Story 006: prism_strike / prism_reserve play path — respawn has no reward to play

---

## QA Test Cases

- **PS-05**: Full-set respawn, opponent unchanged
  - Given: `player_a.collected = [true, true, true, true, false]` from prior rounds; this RESOLUTION processes `PrismCollected(player_a, lane=5)` → sets 5th to true; `player_b.collected = [true, true, false, false, false]`
  - When: `resolve_prism_draws` completes reward delivery and runs respawn check
  - Then: `player_a.collected == [false, false, false, false, false]`; `player_b.collected == [true, true, false, false, false]` (unchanged); `S2CPrismRespawned { player_id: player_a }` staged for both players
  - Edge cases: both players complete their full set in the same RESOLUTION (two independent respawns fire; two `S2CPrismRespawned` staged)

- **PS-13**: Respawn fires AFTER reward messages (timing)
  - Given: `player_a` has 4 lanes collected; this RESOLUTION `player_a` collects Lane 5 (5th); no new `PrismCollected` for respawned lanes arrives (Board/Lane System already ran sub-step 5)
  - When: `resolve_prism_draws` processes the Lane 5 event, delivers the reward message, then fires respawn
  - Then: `player_a.collected` resets to all false AFTER the reward message is staged; reward loop completes before `pending_respawn` check runs; order of operations verifiable by inspecting state at each step
  - Edge cases: both respawn and reward for the 5th lane complete in same `resolve_prism_draws` call — reward staged first, then respawn

- **PS-14**: Respawn = no reward
  - Given: respawn fires for `player_a`
  - When: all 5 lanes reset and `S2CPrismRespawned` staged
  - Then: `player_a.gold` unchanged; `player_a.hand.len()` unchanged; `player_a.mana` unchanged; no `S2CCardAcquired` staged by the respawn code path; no `S2CGoldUpdate` staged
  - Edge cases: respawn fires for player who also collected a prism this RESOLUTION — reward is from collection, not from respawn (assert no extra card beyond the collection reward)

- **PS-16**: 2v2 per-player prism independence
  - Given: `player_a.collected[2] = true` (Lane 3); `player_b.collected[2] = false`; no new events in buffer for either player this RESOLUTION
  - When: `resolve_prism_draws` runs
  - Then: `player_a.collected[2] == true` (unchanged); `player_b.collected[2] == false` (unchanged); no rewards granted; independence confirmed at data structure level (separate per-player arrays)

- **PS-21**: Independent respawn counts
  - Given: `player_a.collected = [true, true, true, true, false]`; `player_b.collected = [true, true, true, false, false]`; this RESOLUTION `PrismCollected(player_a, lane=5)` and `PrismCollected(player_b, lane=4)` both in buffer
  - When: `resolve_prism_draws` processes both (player_a first — lower player_id)
  - Then: `player_a.collected == [false, false, false, false, false]` (full respawn); `player_b.collected == [true, true, true, true, false]` (4 of 5, no respawn); only one `S2CPrismRespawned` staged (for player_a)

- **PS-24**: 2v2 same-team both collect same lane
  - Given: `player_a.collected[2] = false`; `player_b.collected[2] = false`; both `PrismCollected(player_a, lane=3)` and `PrismCollected(player_b, lane=3)` in buffer (from distinct prism tokens per player)
  - When: `resolve_prism_draws` processes (ascending player_id: player_a first, then player_b)
  - Then: `player_a.collected[2] == true`; `player_b.collected[2] == true`; two distinct reward events (one for each player); each reward unicast to its owning player only; `AuditLog` has two entries if both collected Lane 3
  - Edge cases: player_a and player_b have different hand sizes — hand-full check applied independently per player

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/prism/respawn_cycle_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 (`state-scaffold`) must be Done — `PrismState.pending_respawn` field must be defined
- Depends on: Story 002 (`deterministic-lanes`) must be Done — reward delivery loop in `resolve_prism_draws` must exist (respawn appends to the end of that function)
- Depends on: Story 003 (`lane3-rng`) must be Done — full `resolve_prism_draws` function body complete
- Depends on: Pre-implementation gate NP GDD — resolved; `S2CPrismRespawned` registered in `network-protocol.md`
- Unlocks: None — this is a leaf story in the Prism epic dependency chain; Story 006 (spell play path) has no dependency on respawn
