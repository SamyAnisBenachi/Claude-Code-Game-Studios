# Epic: Server-side RNG

> **Layer**: Foundation
> **GDD**: design/gdd/server-rng.md
> **Architecture Module**: `server/foundation/rng.rs`
> **Status**: Ready
> **Stories**: 3 stories created — see table below

## Overview

Implements the single source of randomness for all gameplay events in Lanes and Lies: a `ServerRng` Bevy resource backed by `ChaCha20Rng` (from `rand_chacha 0.3`), initialized once per game session from OS entropy, and exposed exclusively through intent-named API methods that enforce the audit log contract and the strict consumption order defined in ADR-005. No system may hold its own RNG source; all consume a seed from `ServerRng` and perform their computation locally. Seeds are never transmitted — only outcomes. The audit log records every random event with its seed index and outcome encoding for post-game replay and dispute resolution.

**Lifecycle note:** `ServerRng` creation at `SessionReady` and destruction on `GameOverEmitted` are owned by the **Game Session System** (Core layer, ADR-007). This epic implements the resource definition, API, and audit machinery only — lifecycle wiring is a Core epic dependency.

## Governing ADRs

| ADR | Decision Summary | Engine Risk |
|-----|-----------------|-------------|
| ADR-005: Server-side RNG | ChaCha20Rng per session; intent-named API only; audit log at every call site; strict §4 consumption order | LOW |

## GDD Requirements

> Note: `docs/architecture/tr-registry.yaml` has not yet been populated. TR-IDs below are informal references from ADR-005 "GDD Requirements Addressed". Run `/architecture-review` to register stable IDs before stories are written.

| Informal TR-ID | Requirement | ADR Coverage |
|----------------|-------------|--------------|
| TR-RNG-01 | One RNG per game session; ChaCha20Rng from rand_chacha 0.3; seeded from OsRng | ADR-005 ✅ |
| TR-RNG-02 | Single public operation: `next_seed() → u64`; callers seed their own local computation | ADR-005 ✅ |
| TR-RNG-03 | Broadcast rule: results only, never seeds. Seeds and RNG state never in any S2C message | ADR-005 ✅ |
| TR-RNG-04 | Fixed execution order across all phases (§4 table); any reorder corrupts audit replay | ADR-005 ✅ |
| TR-RNG-05 | Audit log: every call captures event_type, seed_index, and encoded result | ADR-005 ✅ |
| TR-RNG-06 | API surface: intent-named methods only; no raw RngCore access outside rng.rs | ADR-005 ✅ |

## Scope

### Deliverables

**`server/src/foundation/rng.rs`**

- `ServerRng` resource struct: private `ChaCha20Rng`; `seed_index: u32` (monotonically incrementing, starts at 1 after `session_init` sentinel); `audit_log: Vec<AuditEntry>`
- `AuditEntry` struct: `event_type: RngEvent`, `seed_index: u32`, `result: Option<String>`
- `RngEvent` enum — all 7 variants (exact names from ADR-005 §1):
  - `AssignFakeObjectives { player_id: PlayerId }`
  - `DrawInitialDraft { player_id: PlayerId }`
  - `DrawShopSlot { player_id: PlayerId, slot_index: u8 }`
  - `ResolveEcaflip { lane: u8 }`
  - `ResolvePrism { player_id: PlayerId, lane: u8 }`
  - `AwardFakeObjectiveReward { player_id: PlayerId, lane: u8 }`
  - `DrawFreeCard { player_id: PlayerId }`
- `session_init` sentinel: at construction, the first `AuditEntry` pushed is `{ event_type: SessionInit, seed_index: 0, result: None }` where `SessionInit` is a special variant marking the session boundary. Required by GDD RNG11/RNG5.
- Public constructor `ServerRng::new() -> Self` — calls `ChaCha20Rng::from_entropy()`; `seed_index = 0`; pushes `session_init` sentinel; sets `seed_index = 1` before any gameplay call
- Intent-named API methods — one per `RngEvent` variant (return placeholder outcomes initially; implementations filled in by consuming epics):
  - `fn assign_fake_objectives(&mut self, player_id: PlayerId) -> (u8, u8)` — seeds 0..5 pick, then 0..4 pick
  - `fn draw_initial_draft(&mut self, player_id: PlayerId) -> u64` — passes seed to Card Pool
  - `fn draw_shop_slot(&mut self, player_id: PlayerId, slot_index: u8) -> (u64, u64)` — or `(u64, u64, u64)` for neutral path; Card Pool decides how many seeds it consumes
  - `fn resolve_ecaflip(&mut self, lane: u8) -> u64`
  - `fn resolve_prism(&mut self, player_id: PlayerId, lane: u8) -> u64`
  - `fn award_fake_objective_reward(&mut self, player_id: PlayerId, lane: u8) -> u64`
  - `fn draw_free_card(&mut self, player_id: PlayerId) -> u64`
- `ChaCha20Rng` and all `RngCore` access is **private to this module**. No `pub use rand` re-exports.

**Forbidden** (enforced by module privacy):
- No `rand::thread_rng()` anywhere in server game logic
- No `StdRng`, `SmallRng`, or direct `ChaCha20Rng` construction outside this file
- No transmission of seeds, `seed_index`, or `audit_log` entries in any S2C message

**Deferred CI checks** (not in this epic's file scope — owned by CI/devops setup):
- VC3: Module-boundary grep CI check — confirms zero usages of `rand::thread_rng`, `StdRng`, `SmallRng`, or direct `ChaCha20Rng` construction outside `server/src/foundation/rng.rs`
- VC4: Client dep-tree audit — `rand`/`rand_chacha` not reachable from client gameplay modules (covered by Epic 1 CI gates)

**Unit tests** (`server/tests/` or `tests/unit/foundation/`)
- Determinism test: construct `ServerRng` with a fixed seed (test-only constructor that bypasses `from_entropy()`), call the same sequence of intent-named methods, assert `audit_log` byte-for-byte identical across runs
- Consumption-order test: same scripted session produces `audit_log` with `(event_type, seed_index)` sequence exactly matching the §4 order table in ADR-005
- `session_init` sentinel test: `audit_log[0]` is always `SessionInit` with `seed_index = 0`

## Definition of Done

- `ServerRng`, `AuditEntry`, `RngEvent` (7 variants + `SessionInit`) compile in `server/`
- All intent-named API methods present (stub return values acceptable for this epic)
- `session_init` sentinel pushed at construction; `seed_index` starts at 1 for first gameplay call
- Determinism test passes: same fixed seed → same `audit_log` byte-for-byte
- Consumption-order test passes: `(event_type, seed_index)` sequence matches ADR-005 §4 table for a scripted session
- No public `ChaCha20Rng` or `RngCore` exports from the module
- VC3/VC4 CI checks noted as deferred (not blocking this epic)

## Stories

| # | Story | Type | Status | ADR |
|---|-------|------|--------|-----|
| 001 | [Type Definitions & Audit Infrastructure](story-001-type-definitions-audit-infrastructure.md) | Logic | Ready | ADR-005 |
| 002 | [Intent-Named API & Consumption Invariants](story-002-intent-named-api-invariants.md) | Logic | Ready | ADR-005 |
| 003 | [Determinism Proof & Session Reset](story-003-determinism-session-reset.md) | Logic | Ready | ADR-005 |

> Story sequence: 001 → 002 → 003 (linear chain).

## Next Step

Run `/story-readiness production/epics/server-rng/story-001-type-definitions-audit-infrastructure.md` before starting implementation.
