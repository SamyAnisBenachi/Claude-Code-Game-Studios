# Story 001: ServerRng Type Definitions & Audit Infrastructure

> **Epic**: Server-side RNG
> **Status**: Ready
> **Layer**: Foundation
> **Type**: Logic
> **Manifest Version**: 2026-04-29

## Context

**GDD**: `design/gdd/server-rng.md`
**Requirement**: TR-??? (covers TR-RNG-01: one ChaCha20Rng per session; TR-RNG-05: audit log at every call site; RNG1, RNG5, RNG11)

**ADR Governing Implementation**: ADR-005: Server-side RNG — ChaCha20 Seeding, Audit Log, and Consumption Order
**ADR Decision Summary**: Single `ServerRng` resource per session backed by `ChaCha20Rng`. First audit entry is always `session_init` at `seed_index = 0`. Root seed is never logged or transmitted. All `ChaCha20Rng` access is private to `server/src/foundation/rng.rs`.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: LOW
**Engine Notes**: `rand` and `rand_chacha` are pure-Rust crates with no Bevy API surface — no post-cutoff risk. Verify `rand_chacha 0.3` + `rand 0.9` version compatibility on crates.io before implementation (GDD OQ2). `#[derive(Resource)]` on `ServerRng` requires `bevy_ecs` in scope — server crate only, never `shared/`.

**Control Manifest Rules (Foundation layer)**:
- Required: All game randomness uses a single per-session `ServerRng` resource backed by `ChaCha20Rng` from `rand_chacha 0.3`. Seeded once from `OsRng::from_entropy()` at session start. Never re-seed mid-session.
- Forbidden: Never use `rand::thread_rng()`, `StdRng`, or `SmallRng` in server game logic.
- Forbidden: Never transmit RNG seeds to clients in any S2C message.
- Guardrail: `ServerRng` state ~136 bytes. Audit log < 32 KB per session.

---

## Acceptance Criteria

- [ ] `ServerRng` struct exists in `server/src/foundation/rng.rs` with:
  - `rng: ChaCha20Rng` — private field
  - `seed_index: u32` — private, monotonically incrementing
  - `audit_log: Vec<AuditEntry>` — private
  - `#[derive(Resource)]`
- [ ] `AuditEntry` struct exists with fields: `event_type: RngEvent`, `seed_index: u32`, `result: Option<String>`
- [ ] `RngEvent` enum has exactly 8 variants: `SessionInit`, `AssignFakeObjectives { player_id: PlayerId }`, `DrawInitialDraft { player_id: PlayerId }`, `DrawShopSlot { player_id: PlayerId, slot_index: u8 }`, `ResolveEcaflip { lane: u8 }`, `ResolvePrism { player_id: PlayerId, lane: u8 }`, `AwardFakeObjectiveReward { player_id: PlayerId, lane: u8 }`, `DrawFreeCard { player_id: PlayerId }`
- [ ] `ServerRng::new() -> Self` production constructor: calls `ChaCha20Rng::from_entropy()`, pushes `SessionInit` sentinel at `seed_index = 0`, sets internal counter to 1 before returning
- [ ] `ServerRng::from_seed(seed: u64) -> Self` test-only constructor: uses `ChaCha20Rng::seed_from_u64(seed)`, same sentinel behaviour — marked `#[cfg(test)]` or `pub(crate)` to prevent accidental production use
- [ ] Public accessor `audit_log(&self) -> &[AuditEntry]`
- [ ] Public accessor `current_seed_index(&self) -> u32`
- [ ] **RNG1**: After `ServerRng::new()`, `current_seed_index() == 1` (index 0 is consumed by `session_init`)
- [ ] **RNG5**: After `N` gameplay calls to `next_seed()` (internal), `audit_log().len() == N + 1` (the +1 is `SessionInit`)
- [ ] **RNG11**: `audit_log()[0].event_type == RngEvent::SessionInit` and `audit_log()[0].result == None`; no raw seed bytes appear in any `AuditEntry.result` field
- [ ] `ChaCha20Rng` and `RngCore` are not re-exported; `rng` field is private; no public method exposes raw `u64` seeds directly (only via intent-named API in Story 002)
- [ ] `cargo check -p server` passes after adding this file

---

## Implementation Notes

*Derived from ADR-005 §1 Resource Definition and §5 Audit Log Contract:*

**Struct layout:**
```rust
use bevy::prelude::Resource;
use rand_chacha::ChaCha20Rng;
use rand::{SeedableRng, RngCore};

#[derive(Resource)]
pub struct ServerRng {
    rng: ChaCha20Rng,       // private — never exposed
    seed_index: u32,         // starts at 0; entry 0 = session_init; gameplay starts at 1
    audit_log: Vec<AuditEntry>,
}
```

**Constructor and sentinel:**
```rust
impl ServerRng {
    pub fn new() -> Self {
        let mut rng = ChaCha20Rng::from_entropy();
        let mut audit_log = Vec::new();
        // Entry 0 is always session_init — seed_index = 0
        audit_log.push(AuditEntry {
            event_type: RngEvent::SessionInit,
            seed_index: 0,
            result: None,
        });
        Self { rng, seed_index: 1, audit_log }
    }

    #[cfg(test)]
    pub fn from_seed(seed: u64) -> Self {
        let rng = ChaCha20Rng::seed_from_u64(seed);
        let mut audit_log = Vec::new();
        audit_log.push(AuditEntry {
            event_type: RngEvent::SessionInit,
            seed_index: 0,
            result: None,
        });
        Self { rng, seed_index: 1, audit_log }
    }
}
```

**`PlayerId` type**: Use a newtype `PlayerId(u32)` or import from `shared/` if already defined there. If not yet defined in `shared/`, define a local placeholder `type PlayerId = u32` in this module with a `// TODO: import from shared/` comment — do not block this story on it.

**`AuditEntry.result` encoding**: Per GDD Rule 8, the `result` field contains a human-readable encoded outcome string. The encoding per event type is defined in the GDD Rule 8 table. This story does not implement the encoding — it just establishes the `Option<String>` field. Story 002 fills in real result strings when intent-named methods are implemented.

**Verify crates.io version pairing (GDD OQ2):** Before closing this story, confirm on crates.io that `rand_chacha = "0.3"` and `rand = "0.9"` are compatible (i.e., `rand_chacha 0.3.x` uses `rand_core` compatible with `rand 0.9`). Document the verified version pair in a comment at the top of `rng.rs`.

---

## Out of Scope

- Story 002: Intent-named API methods (`assign_fake_objectives`, etc.) — only the scaffolding types are defined here
- Core layer (Game Session System): Inserting/removing `ServerRng` from the world — lifecycle wiring is not part of this epic

---

## QA Test Cases

*QL-STORY-READY skipped — Lean mode.*

- **RNG1: Resource initialises with seed_index = 1**
  - Given: `ServerRng::new()` constructed
  - When: `rng.current_seed_index()` is read
  - Then: Returns `1` (index 0 consumed by `SessionInit`)

- **RNG5: N gameplay calls → N+1 audit entries**
  - Given: `ServerRng::from_seed(42)` constructed
  - When: Internal `next_seed()` is called 3 times (via Story 002 methods or directly in test)
  - Then: `audit_log().len() == 4` (1 sentinel + 3 calls)
  - Edge cases: 0 calls → len == 1 (sentinel only)

- **RNG11: session_init sentinel correct, no raw seed in log**
  - Given: `ServerRng::from_seed(12345)` constructed
  - When: `audit_log()[0]` is inspected
  - Then: `event_type == RngEvent::SessionInit`, `seed_index == 0`, `result == None`
  - Edge cases: Confirm no `AuditEntry.result` contains the string "12345" (raw seed not logged)

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/foundation/server_rng_types_test.rs` — all test cases passing
**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: `workspace-and-shared-types` Story 001 (workspace scaffolding must exist)
- Unlocks: Story 002 (intent-named API)
