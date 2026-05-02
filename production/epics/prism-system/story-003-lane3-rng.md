# Story 003: Lane 3 RNG Draw Pipeline and Audit Log Ordering

> **Epic**: Prism System
> **Status**: Ready
> **Layer**: Feature (M3)
> **Type**: Logic
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/prism-system.md`
**Requirements**: `TR-PRI-003`, `TR-PRI-004`, `TR-PRI-008`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-016: Prism System Architecture — State Ownership, Schedule Slot, and Hand-Write API; ADR-005: Server-side RNG
**ADR Decision Summary**: Lane 3 collection invokes `ServerRng::next_seed()` (event_type `"draw_random"`) then `CardDataPool::draw_random(PoolFilter { card_type: Some(Minion | Spell), .. }, seed)`. Hand-full pre-check short-circuits BEFORE the seed call — no seed consumed on hand-full. Pool-exhausted draw (`None`) DOES consume a seed and writes an audit entry with `result: None`. The `AuditLog` resource captures all Lane 3 RNG events in ascending `player_id` → ascending `lane` order (enforced by the same processing order as Story 002).

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**:
- `ServerRng::next_seed()` — synchronous call; returns `u64`; increments `seed_index` monotonically; must only be called from systems consuming the `Res<ServerRng>` in the ADR-005 RESOLUTION schedule slot
- `CardDataPool::draw_random(filter, seed) -> Option<CardId>` — pure function on `&PlayerPool`; never panics on empty pool
- `distribute(card_id)` — mutates `copies_remaining` in the player's pool; owned by `CardDataPool`; must be called only on `Some(card_id)` result
- `AuditLog` resource is test-inspectable via `world.resource::<AuditLog>()` — no tracing dependency needed for test assertions

**Control Manifest Rules (Feature layer — from ADR-016, ADR-005):**
- Required: Hand-full check precedes `next_seed()` — seed is NOT consumed on hand-full Lane 3
- Required: `distribute(card_id)` called on `Some` result ONLY — never on `None`
- Required: `AuditLog` entry written in same call body as `next_seed()` (per ADR-005: every RNG draw MUST write an `AuditEntry` in the same call as the draw — never async, never best-effort)
- Required: Processing order ascending `player_id` → ascending `lane` (same sort as Story 002) — ensures inter-player audit log ordering is deterministic
- Forbidden: Any `ServerRng` consumption for Lane 3 when `hand.len(player) >= 10`

---

## Acceptance Criteria

*From GDD `design/gdd/prism-system.md`, scoped to this story:*

- [ ] **PS-03** — GIVEN a player's unit occupies their own spawn cell in Lane 3 at end of sub-step 5 AND the prism token is present AND the player's pool contains at least one Minion or Spell card, WHEN `resolve_prism_draws` runs, THEN exactly 1 card is drawn via `draw_random(filter=Minion|Spell, seed)` and added to that player's hand, and exactly 1 server RNG seed is consumed.
- [ ] **PS-10** — GIVEN a player's hand contains exactly 10 cards AND the player's unit collects a prism in Lane 3, WHEN `resolve_prism_draws` evaluates the collection, THEN `draw_random` is NOT called, no server RNG seed is consumed, no card is added to hand, and the prism token IS still marked collected.
- [ ] **PS-11** — GIVEN a player's unit collects a prism in Lane 3 AND `draw_random(filter=Minion|Spell, seed)` returns `None` (pool exhausted), WHEN `resolve_prism_draws` processes the result, THEN no card is added to hand, exactly 1 server RNG seed IS consumed, the audit log entry has `result: None`, and the prism token is marked collected.
- [ ] **PS-17** — GIVEN multiple players each have a unit eligible for prism collection in the same RESOLUTION (including at least one Lane 3 collection to generate audit log entries), WHEN `resolve_prism_draws` processes all collections, THEN the `AuditLog` resource's `entries` Vec for that RESOLUTION contains entries in ascending `player_id` order, and within the same `player_id`, in ascending lane index order.

---

## Implementation Notes

*Derived from ADR-016 Decision Section 3 and GDD Rule 5:*

Lane 3 branch inside the `resolve_prism_draws` message loop (extending Story 002's `match lane` block):

```rust
3 => {
    // Rule 5 step 1: hand-full pre-check (PS-10)
    if hands.len(player) >= HAND_SIZE_MAX {
        // prism consumed (collected already set = true above), no seed, no card, no broadcast
        // NOTE: S2CPrismRewardDropped is NOT emitted for Lane 3 hand-full (GDD Rule 7)
        continue;
    }

    // Rule 5 step 2: consume one seed (PS-03)
    let seed = server_rng.next_seed();  // event_type = "draw_random"
    let filter = PoolFilter {
        card_type: Some(CardTypeFilter::MinionOrSpell),
        class: None, rarity: None, max_cost: None,
    };

    let result = card_pool.draw_random(&pools[player], filter, seed);

    // Audit log — MUST be written in same call body (ADR-005 mandate)
    audit_log.entries.push(PrismAuditEntry {
        player_id: player,
        lane: 3,
        seed_index: server_rng.seed_index - 1,  // already incremented
        result,
    });

    match result {
        Some(card_id) => {
            card_pool.distribute(player, card_id);  // mutates copies_remaining
            match hand_push(&mut hands, player, card_id) {
                Ok(()) => { /* stage S2CCardAcquired — Story 004 */ }
                Err(HandFullError) => {
                    // TOCTOU: hand was not full at pre-check but became full
                    // (only possible in concurrent system — not applicable here,
                    // single system body. Defensive: treat as silent failure.)
                }
            }
        }
        None => {
            // PS-11: pool exhausted — seed consumed, audit entry written, no hand add
        }
    }
}
```

**Ordering guarantee** (PS-17): The message sort by `(player_id, lane)` from Story 002's event collection ensures `AuditLog` entries are appended in the correct order. The Lane 3 audit entry is written during the loop iteration for that (player, lane) pair, so order is preserved automatically.

**`server-rng.md` Rule 3 conditional note** (pre-implementation gate OQ4): Resolved 2026-05-02. `design/gdd/server-rng.md` Rule 3 documents that Lane 3 consumes 0 seeds if the collecting player's hand is full at collection time and warns replay tools not to assume a fixed 1 seed per Lane 3 `PrismCollected` event.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- Story 001: AuditLog resource definition and session lifecycle
- Story 002: Lanes 1/2/4/5 deterministic reward routing; stale duplicate handling
- Story 004: S2CCardAcquired network staging for successful Lane 3 draw
- Story 005: Full-set respawn cycle triggered after all lane events processed

---

## QA Test Cases

- **PS-03**: Lane 3 successful draw
  - Given: `collected[2][player_a] = false`; `hands.len(player_a) < 10`; pool has ≥1 Minion/Spell card; `PrismCollected(player_a, lane=3)` in buffer
  - When: `resolve_prism_draws` runs
  - Then: `ServerRng.seed_index` incremented by exactly 1; `AuditLog.entries` has 1 new entry with `result: Some(card_id)`; `hands.len(player_a) == prior_len + 1`; `collected[2][player_a] == true`; `distribute` called on the returned card_id
  - Edge cases: player's pool has exactly 1 card (succeeds, pool now empty)

- **PS-10**: Hand full → no seed consumed
  - Given: `collected[2][player_a] = false`; `hands.len(player_a) == 10` (exactly full); `PrismCollected(player_a, lane=3)` in buffer
  - When: `resolve_prism_draws` evaluates
  - Then: `ServerRng.seed_index` unchanged (no seed consumed); `AuditLog.entries` unchanged; `hands.len(player_a) == 10`; `collected[2][player_a] == true`; no `S2CPrismRewardDropped` staged (Lane 3 hand-full does NOT emit this message)
  - Edge cases: hand exactly at 10 (not 9 — boundary matters)

- **PS-11**: Pool exhausted → seed consumed, None logged
  - Given: `collected[2][player_a] = false`; `hands.len(player_a) < 10`; pool configured to return `None` for all draws; `PrismCollected(player_a, lane=3)` in buffer
  - When: `resolve_prism_draws` runs
  - Then: `ServerRng.seed_index` incremented by exactly 1; `AuditLog.entries` has 1 new entry with `result: None`; `hands.len(player_a)` unchanged; `collected[2][player_a] == true`; `distribute` NOT called
  - Edge cases: pool has 0 copies of every eligible card (same as None); pool exhausted mid-session (after prior distributes reduced copies_remaining to 0)

- **PS-17**: Audit log ordering across players and lanes
  - Given: `PrismCollected(player_b, lane=5)`, `PrismCollected(player_a, lane=3)`, `PrismCollected(player_a, lane=1)` in buffer (in arbitrary insertion order; player_a has lower player_id); pool has cards for player_a Lane 3
  - When: `resolve_prism_draws` processes all (sort by (player_id, lane))
  - Then: processing order is player_a-lane_1 → player_a-lane_3 → player_b-lane_5; `AuditLog.entries` has exactly 1 entry (only Lane 3 writes audit entries) at position corresponding to player_a; `ServerRng.seed_index` incremented by 1 total (only 1 seed consumed, for player_a Lane 3)
  - Edge cases: two players both collect Lane 3 — player_a seed index N, player_b seed index N+1; entries in ascending player_id order

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/prism/lane3_rng_test.rs` — must exist and pass
*(Testable via `World::new()` with injected `PrismCollected` messages and a mock `CardDataPool` returning `None` or `Some(card_id)`)* 

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 (`state-scaffold`) must be Done — `AuditLog`, `PrismState` must be defined
- Depends on: Story 002 (`deterministic-lanes`) must be Done — `resolve_prism_draws` message loop structure exists; this story adds the Lane 3 branch
- Depends on: Pre-implementation gate OQ4 — resolved 2026-05-02; `server-rng.md` Rule 3 conditional note exists
- Unlocks: Story 004 (`hand-full-network`) — S2CCardAcquired staging for Lane 3 successful draws; Story 005 (`respawn-cycle`) — uses full `resolve_prism_draws` function
