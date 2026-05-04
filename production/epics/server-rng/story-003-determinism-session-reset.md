# Story 003: Determinism Proof & Session Reset

> **Epic**: Server-side RNG
> **Status**: Ready
> **Layer**: Foundation
> **Type**: Logic
> **Manifest Version**: 2026-05-01

## Context

**GDD**: `design/gdd/server-rng.md`
**Requirement**: TR-RNG-004 (fixed execution order/RNG8), TR-RNG-001 (session reset/RNG13), TR-RNG-007 (overflow wrap/RNG15 ADVISORY)

**ADR Governing Implementation**: ADR-005: Server-side RNG — §2 Lifecycle, §4 Consumption Order, Validation Criteria VC1 + VC2
**ADR Decision Summary**: Given the same fixed seed and the same sequence of intent-named method calls, the `audit_log` must be byte-for-byte identical across runs and across host machines. Session boundaries are hard reset boundaries — a new `ServerRng` always starts with `session_init` at index 0, regardless of prior session state.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: LOW
**Engine Notes**: `ChaCha20Rng` is deterministic across platforms by construction (no SIMD divergence). `rand_chacha 0.3` + `rand 0.9` version pairing must be confirmed (Story 001 OQ2) — if the crate changes its stream algorithm in a patch version, this test will catch it.

**Control Manifest Rules (Foundation layer)**:
- Required: All game randomness uses a single per-session `ServerRng` resource backed by `ChaCha20Rng` from `rand_chacha 0.3`. Seeded once from `OsRng::from_entropy()` at session start. Never re-seed mid-session.
- Guardrail: `ServerRng` state ~136 bytes. Audit log < 32 KB per session.

---

## Acceptance Criteria

**Determinism (ADR-005 VC1 + VC2):**
- [ ] A scripted call sequence of all 7 intent-named methods (in ADR-005 §4 order: `assign_fake_objectives` × 2 players, `draw_initial_draft` × 2 players, `draw_shop_slot` × 2 players × 3 slots, `resolve_ecaflip` × 1, `resolve_prism` × 1, `award_fake_objective_reward` × 1, `draw_free_card` × 1) produces the same `audit_log` on two independent `ServerRng::from_seed(FIXED_SEED)` instances
- [ ] The `audit_log` comparison is structural equality: `seed_index` values and `event_type` variants match exactly — not just the count
- [ ] The same scripted sequence run twice in the same test process produces identical results (guards against state leakage between test runs)

**Session reset (RNG13):**
- [ ] Creating a second `ServerRng::from_seed(x)` after a first has been used produces a fresh `audit_log` starting with `SessionInit` at index 0
- [ ] `current_seed_index()` on the new instance returns `1` (reset, not continuing from prior session)
- [ ] The second session's first non-sentinel `seed_index` is `1`, not the prior session's final `seed_index + 1` — sessions are fully independent

**Overflow (RNG15 — ADVISORY):**
- [ ] A `ServerRng` constructed via a test-only `ServerRng::at_max_seed_index()` helper (sets `seed_index = u32::MAX`) does not panic when `next_seed()` is called
- [ ] After the overflow call, `current_seed_index()` returns `0` (wrapping_add behaviour)
- [ ] The audit log records the overflow entry with `seed_index = u32::MAX`, not a panicked or corrupted state

**Deferred ACs — documented (not implemented in this story):**
- [ ] A code comment in `rng.rs` lists: "RNG8 (RESOLUTION ordering): deferred to RSM epic — requires full system chain"; "RNG9 (run condition): each consuming system adds its own guard — see ADR-005 §2 lifecycle"; "RNG10 (chi-squared): deferred to QA/Polish phase"; "RNG14 (no seeds in S2C): deferred to Epic 4 network integration tests"

---

## Implementation Notes

*Derived from ADR-005 §2 Lifecycle and Validation Criteria VC1–VC2:*

**Scripted determinism test sequence:**
```rust
#[test]
fn test_determinism() {
    const SEED: u64 = 0xDEAD_BEEF_CAFE_1234;
    const P1: PlayerId = PlayerId(1);
    const P2: PlayerId = PlayerId(2);

    fn run_scripted_session(seed: u64) -> Vec<AuditEntry> {
        let mut rng = ServerRng::from_seed(seed);
        // ADR-005 §4 DRAFT_INITIAL order
        rng.assign_fake_objectives(P1);
        rng.assign_fake_objectives(P2);
        rng.draw_initial_draft(P1);
        rng.draw_initial_draft(P2);
        // DRAFT_SHOP
        for slot in 0..3u8 {
            rng.draw_shop_slot(P1, slot);
            rng.draw_shop_slot(P2, slot);
        }
        // RESOLUTION
        rng.resolve_ecaflip(1);
        rng.resolve_prism(P1, 2);
        rng.award_fake_objective_reward(P1, 3);
        rng.draw_free_card(P1);
        rng.audit_log().to_vec()
    }

    let log_a = run_scripted_session(SEED);
    let log_b = run_scripted_session(SEED);

    assert_eq!(log_a.len(), log_b.len());
    for (a, b) in log_a.iter().zip(log_b.iter()) {
        assert_eq!(a.seed_index, b.seed_index);
        // Compare event_type variant discriminants (requires PartialEq on RngEvent)
        assert_eq!(std::mem::discriminant(&a.event_type), std::mem::discriminant(&b.event_type));
    }
}
```

**`AuditEntry` and `RngEvent` must derive `Clone` and `PartialEq`** for this test to work. Add those derives in Story 001 if not already present.

**RNG13 session reset test:**
```rust
#[test]
fn test_session_reset() {
    let mut session_a = ServerRng::from_seed(1);
    session_a.resolve_ecaflip(0);
    session_a.resolve_ecaflip(1);
    // session_a.current_seed_index() == 3 here

    // New session — completely independent
    let session_b = ServerRng::from_seed(1);
    assert_eq!(session_b.current_seed_index(), 1); // reset to 1 (after sentinel)
    assert_eq!(session_b.audit_log().len(), 1);    // sentinel only
    assert!(matches!(session_b.audit_log()[0].event_type, RngEvent::SessionInit));
}
```

**RNG15 overflow test helper:**
The test-only `at_max_seed_index()` constructor needs to advance the internal counter to `u32::MAX`. Since we can't efficiently call `next_seed()` ~4.3 billion times, expose a test-only constructor that sets the counter directly:
```rust
#[cfg(test)]
pub fn at_max_seed_index() -> Self {
    let mut s = Self::from_seed(42);
    s.seed_index = u32::MAX;
    s
}
```

**`wrapping_add` for overflow:** In `next_seed()`, use:
```rust
self.seed_index = self.seed_index.wrapping_add(1);
```
This ensures RNG15 doesn't panic. The GDD Edge Cases section documents this: "If it occurs, `seed_index` wraps to 0. The session log must note the wrap-around to prevent audit confusion." Adding a `// seed_index wrapped` note to the audit result is optional but recommended.

---

## Out of Scope

- Lifecycle wiring (insertion/removal from world) — owned by Game Session System epic (Core layer)
- RNG8, RNG9, RNG10, RNG14 — deferred (documented in code comments per ACs above)

---

## QA Test Cases

*QL-STORY-READY skipped — Lean mode.*

- **Determinism: same seed → same audit_log structure**
  - Given: Two `ServerRng::from_seed(0xDEAD_BEEF_CAFE_1234)` instances
  - When: The same scripted call sequence (all 7 methods in §4 order) is applied to each
  - Then: Both `audit_log()` slices have identical length; all `(seed_index, event_type discriminant)` pairs match

- **RNG13: New session starts clean**
  - Given: Session A `ServerRng::from_seed(1)` after multiple calls (seed_index = 5)
  - When: Session B `ServerRng::from_seed(1)` is created independently
  - Then: B's `current_seed_index() == 1`; B's `audit_log().len() == 1`; B's `audit_log()[0]` is `SessionInit`

- **RNG15: Overflow wraps without panic**
  - Given: `ServerRng::at_max_seed_index()` (seed_index = u32::MAX)
  - When: Any intent-named method is called
  - Then: No panic; `current_seed_index()` returns 0 (wrapped); audit_log has one more entry

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/foundation/server_rng_determinism_test.rs` — all test cases passing
**Status**: [x] `tests/unit/foundation/server_rng_determinism_test.rs` evidence file present; runnable tests embedded in `server/src/foundation/rng.rs` (run via `cargo test -p server`)

---

## Dependencies

- Depends on: Story 002 (intent-named API must exist for scripted sequence test)
- Unlocks: Epic `server-rng` **complete** → all three Foundation epics (workspace, game-config-pipeline, server-rng) done
