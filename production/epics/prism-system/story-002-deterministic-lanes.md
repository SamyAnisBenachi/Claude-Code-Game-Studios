# Story 002: Deterministic Lane Rewards — Lanes 1/2/4/5

> **Epic**: Prism System
> **Status**: Ready
> **Layer**: Feature (M3)
> **Type**: Logic
> **Manifest Version**: 2026-04-30

## Context

**GDD**: `design/gdd/prism-system.md`
**Requirements**: `TR-PRI-001`, `TR-PRI-002`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-016: Prism System Architecture — State Ownership, Schedule Slot, and Hand-Write API
**ADR Decision Summary**: `resolve_prism_draws` drains `MessageReader<PrismCollected>` and processes each message in ascending `player_id` → ascending `lane` order. For Lanes 1/2/4/5, reward routing is deterministic (no RNG): Lane 1/5 → `hand_push(player, prism_strike)`, Lane 2/4 → `hand_push(player, prism_reserve)`. Stale duplicates (`collected[lane][player] == true`) are silently discarded with a `warn!` log and a `DiscardLog` entry. Lanes 1/2/4/5 consume **zero** `ServerRng` seeds.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**:
- `MessageReader<PrismCollected>`: Bevy-layer server-internal message bus (NOT Lightyear C2S). Drain via `for msg in reader.read()` — confirmed in `docs/engine-reference/bevy/current-best-practices.md`
- `hand_push()` is a shared module function from `server/feature/hand/mod.rs` — takes `&mut PlayerHands`, not `ResMut<PlayerHands>` directly (the system holds the `ResMut`)
- Static card IDs (`prism_strike_card_id`, `prism_reserve_card_id`) are read from `Res<CardCatalog>` by looking up card definitions in `assets/data/cards.json` — no hardcoded `u32` values in system code
- `resolve_prism_draws` is registered: `.after(resolve_ecaflip_triggers).before(award_fake_objective_rewards)` (ADR-005 RESOLUTION schedule slot)

**Control Manifest Rules (Feature layer — from ADR-016):**
- Required: Process `PrismCollected` messages in ascending `player_id` → ascending `lane` order (GDD Rule 6 / TR-PRI-008 determinism requirement)
- Required: Stale duplicate guard — check `collected[lane][player]` before any reward; if already `true`, add `DiscardLog` entry and `warn!`; discard silently
- Forbidden: Any `ServerRng::next_seed()` call for Lanes 1/2/4/5
- Forbidden: Economy System (`spend_gold`, `add_reserve`, any gold mutation) in the reward delivery path for prism collection
- Forbidden: `EventWriter<T>` / `EventReader<T>` — Bevy 0.17+ removed these; use `MessageWriter<T>` / `MessageReader<T>`

---

## Acceptance Criteria

*From GDD `design/gdd/prism-system.md`, scoped to this story:*

- [ ] **PS-01** — GIVEN a player's unit occupies their own spawn cell in Lane 1 or Lane 5 at the end of RESOLUTION sub-step 5 AND the prism token for that lane is present, WHEN `resolve_prism_draws` runs, THEN exactly one `prism_strike` spell card is added to that player's hand and the prism token is marked collected.
- [ ] **PS-02** — GIVEN a player's unit occupies their own spawn cell in Lane 2 or Lane 4 at the end of RESOLUTION sub-step 5 AND the prism token is present, WHEN `resolve_prism_draws` runs, THEN exactly one `prism_reserve` spell card is added to that player's hand and the prism token is marked collected.
- [ ] **PS-07** — GIVEN a WALL unit (MP=0) is parked at a player's own spawn cell in any lane AND the prism token for that lane is present, WHEN RESOLUTION sub-step 5 completes (zero movement; unit remains at spawn cell), THEN the prism token is collected and the lane reward is granted.
- [ ] **PS-12** — GIVEN `resolve_prism_draws` receives a `PrismCollected` message for a (player, lane) pair whose token is already marked collected (stale duplicate), WHEN the message is evaluated, THEN: (a) no reward granted, (b) no seed consumed, (c) no client message sent, AND (d) `world.resource::<DiscardLog>().entries` contains exactly one entry `(player_id, lane)`.

---

## Implementation Notes

*Derived from ADR-016 Decision Section 3 (system signature) and GDD Rules 3–4:*

```rust
pub fn resolve_prism_draws(
    mut prism_state: ResMut<PrismState>,
    mut hands: ResMut<PlayerHands>,
    card_catalog: Res<CardCatalog>,
    mut discard_log: ResMut<DiscardLog>,
    mut audit_log: ResMut<AuditLog>,
    mut prism_collected: MessageReader<PrismCollected>,
    phase: Res<CurrentPhase>,
    // ... server_rng, card_pool for Lane 3 (Story 003)
    // ... network sender for S2CCardAcquired, S2CPrismRewardDropped (Story 004)
) {
    if phase.current != RoundPhase::Resolution {
        return;
    }

    // Collect all pending messages, sort by (player_id, lane) — ascending both
    let mut events: Vec<PrismCollected> = prism_collected.read().collect();
    events.sort_by_key(|e| (e.player_id, e.lane));

    for event in events {
        let (player, lane) = (event.player_id, event.lane);
        let lane_idx = (lane - 1) as usize;

        // Stale duplicate guard (Rule 3 / PS-12)
        if prism_state.collected[player_idx(player)][lane_idx] {
            warn!("Stale PrismCollected for ({:?}, lane {})", player, lane);
            discard_log.entries.push((player, lane));
            continue;
        }

        // Mark collected (always, even on hand-full paths)
        prism_state.collected[player_idx(player)][lane_idx] = true;

        match lane {
            1 | 5 => {
                let card_id = card_catalog.id_for("prism_strike");
                match hand_push(&mut hands, player, card_id) {
                    Ok(()) => { /* stage S2CCardAcquired — Story 004 */ }
                    Err(HandFullError) => { /* stage S2CPrismRewardDropped — Story 004 */ }
                }
            }
            2 | 4 => {
                let card_id = card_catalog.id_for("prism_reserve");
                match hand_push(&mut hands, player, card_id) {
                    Ok(()) => { /* stage S2CCardAcquired — Story 004 */ }
                    Err(HandFullError) => { /* stage S2CPrismRewardDropped — Story 004 */ }
                }
            }
            3 => { /* Lane 3 RNG path — Story 003 */ }
            _ => { warn!("Unknown prism lane {}", lane); }
        }
    }

    // Rule 8: respawn check — Story 005
    // Rule 9: respawn fire — Story 005
}
```

**PS-07 note**: The Prism System trusts `PrismCollected` events from Board/Lane System. WALL units (MP=0) at the spawn cell produce a `PrismCollected` message from Board/Lane System exactly as any other unit; `resolve_prism_draws` processes it identically. No special-case logic for WALL units in the Prism System. The Board/Lane System edge case is tested in `board-lane-system` stories (BL-12, BL-18).

**Static card lookup**: `card_catalog.id_for("prism_strike")` / `card_catalog.id_for("prism_reserve")` — these are `CardId` lookups by name string from the immutable `CardCatalog`. The actual mana cost and damage values live in `CardData` and are not read here (they're relevant at play time — Story 006).

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- Story 001: PrismState, DiscardLog, AuditLog resource definitions and session lifecycle
- Story 003: Lane 3 RNG draw pipeline (draw_random, seed consumption, AuditLog population)
- Story 004: Hand-full rejection for Lanes 1/2/4/5 (S2CPrismRewardDropped staging); S2CCardAcquired network staging
- Story 005: Full-set respawn cycle (Rules 8–9)
- Story 006: prism_strike / prism_reserve play path (effect when card is played from hand)

---

## QA Test Cases

- **PS-01**: Lane 1/5 → prism_strike granted
  - Given: `collected[0][player_a] = false`; `hands.len(player_a) < 10`; `PrismCollected(player_a, lane=1)` in buffer
  - When: `resolve_prism_draws` runs
  - Then: `collected[0][player_a] == true`; `hands.len(player_a)` increased by 1; hand contains `prism_strike` card; `ServerRng.seed_index` unchanged; gold unchanged
  - Edge cases: Lane 5 (mirror of Lane 1 — same prism_strike card); both Lane 1 and Lane 5 in same RESOLUTION for same player (two prism_strike cards added in lane order)

- **PS-02**: Lane 2/4 → prism_reserve granted
  - Given: `collected[1][player_a] = false`; `hands.len(player_a) < 10`; `PrismCollected(player_a, lane=2)` in buffer
  - When: `resolve_prism_draws` runs
  - Then: `collected[1][player_a] == true`; hand contains `prism_reserve` card; no seed consumed; gold unchanged
  - Edge cases: Lane 4 (mirror — same prism_reserve card); hand at 9 cards (adds to 10, confirming boundary)

- **PS-07**: WALL unit triggers collection normally
  - Given: `PrismCollected(player_a, lane=N)` in buffer (originating from WALL unit — Prism System does not distinguish); `collected[N-1][player_a] = false`; `hands.len(player_a) < 10`
  - When: `resolve_prism_draws` runs
  - Then: `collected[N-1][player_a] == true`; appropriate reward card added to hand; no special WALL handling
  - Edge cases: WALL at Lane 3 spawn (produces Lane 3 PrismCollected — tested in Story 003); WALL collected in prior round (`collected = true` already — stale duplicate path, PS-12)

- **PS-12**: Stale duplicate → DiscardLog entry
  - Given: `collected[0][player_a] = true` (Lane 1 already collected); `PrismCollected(player_a, lane=1)` in buffer (stale duplicate)
  - When: `resolve_prism_draws` evaluates
  - Then: `hands.len(player_a)` unchanged; no `S2CCardAcquired` staged; `ServerRng.seed_index` unchanged; `DiscardLog.entries` contains exactly `[(player_a, 1)]`
  - Edge cases: two stale duplicates in same RESOLUTION for same player (two DiscardLog entries); stale for one player, valid for another (only stale is discarded, valid proceeds normally)

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/prism/deterministic_lanes_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 (`state-scaffold`) must be Done — `PrismState`, `DiscardLog`, `PlayerHands`, `hand_push()` must be defined
- Depends on (external): Card Acquisition Story 001 must be Done — `hand_push()` shared API must exist
- Unlocks: Story 003 (Lane 3 builds on top of `resolve_prism_draws`), Story 004 (hand-full path uses same system)
