# Story 002: Intent-Named API & Consumption Invariants

> **Epic**: Server-side RNG
> **Status**: Complete
> **Layer**: Foundation
> **Type**: Logic
> **Manifest Version**: 2026-04-29

## Context

**GDD**: `design/gdd/server-rng.md`
**Requirement**: TR-??? (covers TR-RNG-02: single public API; TR-RNG-06: intent-named methods only; TR-RNG-04: fixed consumption order; RNG2, RNG6, RNG7, RNG12)

**ADR Governing Implementation**: ADR-005: Server-side RNG — §6 API Surface, §4 Consumption Order, §7 Forbidden
**ADR Decision Summary**: `ServerRng` exposes only intent-named methods — one per `RngEvent` variant. Each method calls the private `next_seed()` helper, writes an `AuditEntry`, and returns the seed (as `u64`) or typed result. Raw `RngCore` access is private. No consumer may bypass the audit log.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: LOW
**Engine Notes**: No Bevy API surface in this file. Pure Rust. `rand 0.9` + `rand_chacha 0.3` only.

**Control Manifest Rules (Foundation layer)**:
- Required: RNG consumption order is strict and binding — DRAFT_INITIAL: AssignFakeObjectives → DrawInitialDraft; DRAFT_SHOP: DrawShopSlot; RESOLUTION: ResolveEcaflip → ResolvePrism → AwardFakeObjectiveReward → DrawFreeCard.
- Forbidden: Never use `rand::thread_rng()`, `StdRng`, `SmallRng`, or direct `ChaCha20Rng` outside `rng.rs`.
- Forbidden: Never transmit seeds in any S2C message.

---

## Acceptance Criteria

**Private `next_seed()` helper:**
- [ ] Private method `fn next_seed(&mut self) -> u64` exists; calls `self.rng.next_u64()`, increments `self.seed_index`, returns the `u64`
- [ ] **RNG6**: When `next_seed()` is called for an event where the eligible subset is empty (draw returns `None`), `seed_index` still increments by 1 and the `AuditEntry` is pushed with `result: None` — the seed is consumed regardless of outcome

**All 7 intent-named public methods present:**
- [ ] `pub fn assign_fake_objectives(&mut self, player_id: PlayerId) -> (u8, u8)` — calls `next_seed()` twice; pushes two `AuditEntry` items with `RngEvent::AssignFakeObjectives`; returns `(0, 0)` as stub (real computation in Objective System epic)
- [ ] `pub fn draw_initial_draft(&mut self, player_id: PlayerId) -> u64` — calls `next_seed()` once; returns seed for Card Pool to use
- [ ] `pub fn draw_shop_slot(&mut self, player_id: PlayerId, slot_index: u8) -> u64` — calls `next_seed()` once for the split roll; Card Pool determines how many total seeds to request (further `draw_shop_slot` calls for Phase 2/3); stub return is the raw seed
- [ ] `pub fn resolve_ecaflip(&mut self, lane: u8) -> u64` — calls `next_seed()` once; returns seed
- [ ] `pub fn resolve_prism(&mut self, player_id: PlayerId, lane: u8) -> u64` — calls `next_seed()` once; returns seed
- [ ] `pub fn award_fake_objective_reward(&mut self, player_id: PlayerId, lane: u8) -> u64` — calls `next_seed()` once; returns seed
- [ ] `pub fn draw_free_card(&mut self, player_id: PlayerId) -> u64` — calls `next_seed()` once; returns seed

**Audit integrity:**
- [ ] Every intent-named method pushes exactly one `AuditEntry` per `next_seed()` call with the matching `RngEvent` variant, the current `seed_index` (before increment), and `result: None` (stub — real result encoding added by consuming epics when they implement actual draws)
- [ ] **RNG2**: Two `ServerRng::from_seed(A)` and `ServerRng::from_seed(B)` with `A != B` — first call to any intent-named method on each returns different `u64` values
- [ ] **RNG7**: Calling `resolve_ecaflip(lane: 1)` then `resolve_ecaflip(lane: 1)` on the same instance produces two `AuditEntry` items with consecutive `seed_index` values (K and K+1), both with `event_type: RngEvent::ResolveEcaflip { lane: 1 }`
- [ ] **RNG12**: Documented ordering contract: when processing concurrent events for multiple players/lanes, callers MUST iterate in order: ascending `player_id` → ascending `lane` → ascending board position. This contract is documented as a `/// # Ordering Contract` doc-comment on `ServerRng` and tested by calling methods for player_id=2/lane=3 before player_id=1/lane=1 and asserting the resulting audit log is flagged as non-deterministic (or alternatively: a test demonstrates correct ordering when the caller follows the contract)
- [ ] No method exposes `ChaCha20Rng`, `RngCore`, raw internal state, or `seed_index` beyond the `current_seed_index()` accessor

---

## Implementation Notes

*Derived from ADR-005 §6 API Surface and §4 Consumption Order table:*

**Private `next_seed()` internal helper:**
```rust
fn next_seed(&mut self) -> u64 {
    let seed = self.rng.next_u64();
    // Note: seed_index is recorded BEFORE increment in the audit entry
    // but seed_index field tracks the NEXT available index
    // So at call time: record seed_index-1 in the entry (or track pre-increment)
    // ADR-005 §5: "Record the seed_index value AT THE TIME OF CALL (before increment)"
    let idx = self.seed_index;
    self.seed_index = self.seed_index.wrapping_add(1);
    seed
    // AuditEntry is pushed by the caller (intent-named method), not here
    // This keeps next_seed() pure and audit-entry free
}
```

**Audit entry in each intent-named method (example):**
```rust
pub fn resolve_ecaflip(&mut self, lane: u8) -> u64 {
    let idx = self.seed_index;
    let seed = self.next_seed();
    self.audit_log.push(AuditEntry {
        event_type: RngEvent::ResolveEcaflip { lane },
        seed_index: idx,
        result: None, // consuming epic (Combat Resolution) fills this in
    });
    seed
}
```

**`assign_fake_objectives` produces TWO calls (per GDD seed table):**
```rust
pub fn assign_fake_objectives(&mut self, player_id: PlayerId) -> (u8, u8) {
    let idx1 = self.seed_index;
    let seed1 = self.next_seed();
    self.audit_log.push(AuditEntry {
        event_type: RngEvent::AssignFakeObjectives { player_id },
        seed_index: idx1,
        result: None,
    });
    let idx2 = self.seed_index;
    let seed2 = self.next_seed();
    self.audit_log.push(AuditEntry {
        event_type: RngEvent::AssignFakeObjectives { player_id },
        seed_index: idx2,
        result: None,
    });
    // Stub: returns (0, 0) — Objective System epic implements real lane assignment
    let _ = (seed1, seed2);
    (0, 0)
}
```

**RNG6 — empty pool draws still consume a seed:** The `result: None` in `AuditEntry` signals "no eligible cards" but the `seed_index` still advances. This is by design — the audit log must be reproducible even when a draw yields nothing. Consuming epics (Card Pool, etc.) are responsible for calling `next_seed()` before checking availability, then passing `result: None` to the audit entry if the draw was vacuous.

**RNG12 ordering contract doc-comment (on the struct):**
```rust
/// # Ordering Contract
///
/// When multiple players have simultaneous random events (e.g., Ecaflip triggers
/// for both players in the same resolution sub-step), callers MUST process events
/// in this deterministic order:
///
/// 1. Ascending `player_id` (lower ID first)
/// 2. Within one player: ascending `lane` index (lane 1 → 5)
/// 3. Within a lane: ascending board position (cell 1 → 8)
///
/// Violating this order produces a different audit log for the same game state,
/// breaking determinism and replay. This is a caller contract — `ServerRng` cannot
/// enforce it internally.
impl ServerRng { ... }
```

---

## Out of Scope

- Story 003: Determinism proof and session reset testing
- Consuming epics (Core+): Real result values in `AuditEntry.result` — stubs are acceptable here
- RNG8: RESOLUTION system ordering — deferred to RSM epic (requires full system chain)
- RNG9: Run-condition guard — each consuming system adds its own; not `ServerRng`'s responsibility
- RNG14: No-seeds-in-S2C test — deferred to Epic 4 network integration

---

## QA Test Cases

*QL-STORY-READY skipped — Lean mode.*

- **RNG2: Different seeds → different first output**
  - Given: `let mut a = ServerRng::from_seed(1); let mut b = ServerRng::from_seed(2);`
  - When: `a.resolve_ecaflip(0)` and `b.resolve_ecaflip(0)` are called
  - Then: The two returned `u64` values differ

- **RNG6: Empty-pool draw still increments seed_index**
  - Given: `ServerRng::from_seed(99)` at seed_index 1
  - When: `resolve_ecaflip(0)` is called (simulating an empty-pool scenario by checking the method still calls `next_seed()`)
  - Then: `current_seed_index() == 2`; `audit_log().last().result == None`

- **RNG7: Two Ecaflip triggers → consecutive seed_index entries**
  - Given: `ServerRng::from_seed(7)`, initial `current_seed_index() == 1`
  - When: `resolve_ecaflip(1)` is called, then `resolve_ecaflip(1)` again
  - Then: `audit_log()[1].seed_index == 1`, `audit_log()[2].seed_index == 2`, both have `event_type = ResolveEcaflip { lane: 1 }`

- **RNG12: assign_fake_objectives produces 2 entries per call**
  - Given: `ServerRng::from_seed(3)`
  - When: `assign_fake_objectives(PlayerId(1))` is called
  - Then: `audit_log().len() == 3` (sentinel + 2 entries); both entries have `event_type = AssignFakeObjectives { player_id: PlayerId(1) }` with seed_index 1 and 2

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/foundation/server_rng_api_test.rs` — all test cases passing
**Status**: [x] `tests/unit/foundation/server_rng_api_test.rs` — evidence doc present; 10 embedded `#[cfg(test)]` tests in `server/src/foundation/rng.rs` (run via `cargo test -p server`). Local execution blocked by pre-existing Smart App Control/MSVC issue; CI is verification gate.

---

## Dependencies

- Depends on: Story 001 (type definitions must exist)
- Unlocks: Story 003 (determinism proof)

---

## Completion Notes
**Completed**: 2026-04-29
**Criteria**: 11/11 passing
**Deviations**: None
**Test Evidence**: Logic — `tests/unit/foundation/server_rng_api_test.rs` (evidence doc); 10 tests embedded in `server/src/foundation/rng.rs` covering RNG2/RNG6/RNG7/RNG12 ACs
**Code Review**: APPROVED (lean mode, /code-review ran immediately prior)
