# ADR-005: Server-side RNG — ChaCha20 Seeding, Audit Log, and Consumption Order

## Status

Accepted

## Date

2026-04-29

## Last Verified

2026-05-01

## Amendments

| Date | Source | Change |
|------|--------|--------|
| 2026-05-01 | ADR-018 keyword-system amendment | Added 3 keyword combat RNG seed slots to `RngEvent`: `RangeEquidistantSelect`, `TeleportRandomDest`, `StrichChangeLaneSelect`. Inserted as Orders 4–6 in the RESOLUTION consumption table; former Orders 4–7 renumbered to 7–10. Migration Plan steps 5–6 updated to reflect new order numbers. |

## Decision Makers

- User (final authority)
- technical-director (architecture, audit-log design, consumption-order specification)
- network-programmer (consulted: server-authoritative randomness boundary)
- gameplay-programmer (consulted: consumption-site enumeration across systems)

## Summary

All gameplay randomness in Lanes and Lies is computed server-side using a single per-session `ServerRng` resource backed by `ChaCha20Rng` (from `rand_chacha 0.3`), seeded once from `OsRng` at session start. Seeds are NEVER transmitted to clients; only outcomes are broadcast. A monotonically-incrementing `seed_index` and a structured `audit_log` provide post-game determinism replay and anti-cheat traceability. Consumption order across all RNG sites is fixed and strictly enforced: violating the order corrupts audit replay even when the seed is identical.

## Engine Compatibility

- **Bevy**: 0.18 (Resource API stable across 0.15-0.18; no migration risk)
- **Knowledge Risk**: LOW — `rand` and `rand_chacha` are pure-Rust crates outside the Bevy API surface and unaffected by the post-0.14 Bevy knowledge gap.
- **Crate versions**:
  - `rand = "0.9"` (server crate only)
  - `rand_chacha = "0.3"` (server crate only)
- **WASM constraint**: Neither crate is added to the client crate. Client `Cargo.toml` MUST NOT depend on `rand` or `rand_chacha` for game logic. (Cosmetic-only RNG on client — e.g. particle jitter — is permitted but must use a separate, clearly-scoped dependency and MUST NOT influence any gameplay state.)

## ADR Dependencies

**Depends On:**
- ADR-003 — Workspace layout. `ServerRng` lives in `server/src/foundation/rng.rs`; the client crate has no path to construct or consume it.
- ADR-012 — SessionReady Delivery. The Game Session System owns `ServerRng` creation (inserted via `Commands::insert_resource` immediately before `Commands::trigger(SessionReady)`) and destruction on `GameOverEmitted`. (Note: originally referenced as "ADR-007 pending" when this ADR was authored; ADR-012 is the accepted ADR that formalizes this contract.)

**Enables:**
- Objective System — fake-objective lane assignment at `DRAFT_INITIAL`
- Card Pool / Shop — initial draft draw and per-phase shop slot rolls
- Combat (Milestone 2) — Ecaflip dice resolution
- Prism activation resolution
- Fake-objective-on-destroy reward roll (incl. conditional free card draw)

**Blocks:**
- Any system that calls `ServerRng::next_seed()` or any RNG-consuming method. No system may merge until its consumption site is registered in the order table below.

## Context

Lanes and Lies is a competitive bluff/auction card game where the integrity of randomness is a core trust contract between the players and the server:

1. **Bluff fairness** — Fake objectives are assigned by RNG. If a client could predict, observe, or influence the seed, the bluff layer collapses (the entire game falls apart, since `Sang Méprise` and other reveal mechanics presume hidden state is genuinely hidden).
2. **Economy fairness** — Shop rolls determine which cards each player can buy. Client-side or shared-seed RNG would let a player predict opponent shop offerings and counter-pick.
3. **Combat fairness** — Ecaflip dice and Prism procs decide round outcomes. These must be unspoofable.
4. **Reproducibility** — Post-game replay, desync diagnosis, and anti-cheat investigations require that, given the same seed and the same input event ordering, the server can reproduce every random outcome exactly. This requires both seed determinism AND consumption-order determinism.
5. **Auditability** — When a player disputes an outcome ("the shop never offered me X across 5 rounds"), we need a structured log to verify the actual roll sequence without re-running the entire match.

The technical preferences document (`.claude/docs/technical-preferences.md`) already encodes the principle "No client-side RNG — all randomness must be seeded and computed server-side, result broadcast to clients." This ADR operationalises that principle into a concrete architecture: which crate, which RNG type, where it lives in the ECS, how it is seeded and torn down, what its audit contract is, and — most importantly — the strict consumption order across all gameplay phases.

A single `ChaCha20Rng` was chosen over `StdRng` or `SmallRng` because:
- `ChaCha20Rng` is cryptographically strong, deterministic across platforms (no SIMD path divergence between server hosts), and serializable. The latter two properties are non-negotiable for cross-machine replay and pause/resume.
- `StdRng` is currently `ChaCha12Rng`-backed but the underlying type is **not** part of `rand`'s stability contract — a future `rand` version could change it and silently break replay. `ChaCha20Rng` from `rand_chacha` is explicitly versioned.
- `SmallRng` is non-deterministic across platforms and forbidden for any audit-relevant role.

`OsRng::from_entropy()` is used at session start because the server is the sole consumer; there is no replay-from-fixed-seed requirement at session creation, only within a session once it begins.

## Decision

### 1. Resource Definition

```rust
// server/src/foundation/rng.rs
use bevy::prelude::*;
use rand_chacha::ChaCha20Rng;
use rand::{SeedableRng, RngCore};

#[derive(Resource)]
pub struct ServerRng {
    rng: ChaCha20Rng,
    seed_index: u32,         // monotonically incrementing, starts at 0
    audit_log: Vec<AuditEntry>,
}

pub struct AuditEntry {
    pub event_type: RngEvent,
    pub seed_index: u32,
    pub result: Option<String>,  // encoded outcome string for post-game audit
}

pub enum RngEvent {
    AssignFakeObjectives    { player_id: PlayerId },
    DrawInitialDraft        { player_id: PlayerId },
    DrawShopSlot            { player_id: PlayerId, slot_index: u8 },
    // ── Keyword combat RNG (added by ADR-018 amendment) ──────────────────
    /// RANGE equidistant target selection (SS3 and SS6 RESOLUTION sub-steps).
    RangeEquidistantSelect  { player_id: PlayerId, lane: u8 },
    /// TELEPORT random destination lane selection within triggering sub-step.
    TeleportRandomDest      { player_id: PlayerId, lane: u8 },
    /// Strich auto-CHANGE LANE: selects one of the two valid adjacent lanes.
    StrichChangeLaneSelect  { player_id: PlayerId },
    // ─────────────────────────────────────────────────────────────────────
    ResolveEcaflip          { lane: u8 },
    ResolvePrism            { player_id: PlayerId, lane: u8 },
    AwardFakeObjectiveReward{ player_id: PlayerId, lane: u8 },
    DrawFreeCard            { player_id: PlayerId },
}
```

### 2. Lifecycle

- **Creation**: The `Game Session System` constructs `ServerRng` with `ChaCha20Rng::from_entropy()`, `seed_index: 0`, and an empty `audit_log`, immediately before emitting `SessionReady`.
- **Insertion**: Inserted as a `Resource` into the server `World`. Owned by the session; no other system constructs it.
- **Destruction**: The Game Session System removes the resource on `GameOverEmitted`.
- **No persistence**: `ServerRng` state is never written to disk, never replicated, never serialised across sessions. Session boundaries are hard reset boundaries.
- **No mid-session re-seed**: Once a session begins, the seed is fixed. There is no "re-roll" or "reseed on dispute" path.

### 3. Seed Transmission Policy

- Seeds (the 32-byte ChaCha20 state) are **never** included in any S2C message.
- `seed_index` MAY appear in S2C debug/admin messages but MUST NOT appear in any production-shipped client-visible payload. Treat `seed_index` as server-internal.
- The `audit_log` is server-only. It MAY be persisted off-session to a server-side log store for later inspection; it MUST NOT be sent to clients during a live session.

### 4. Consumption Order (STRICT — must not be violated for audit determinism)

The following order is binding. Any system that consumes RNG must do so at the prescribed phase, in the prescribed sub-order, and must not interleave with foreign RNG consumers.

#### DRAFT_INITIAL phase

| Order | Event                       | Per-call seed count | Iteration order                                |
|-------|-----------------------------|---------------------|------------------------------------------------|
| 1     | `AssignFakeObjectives`      | 2 seeds per player  | ascending `player_id`                          |
| 2     | `DrawInitialDraft`          | per player          | ascending `player_id`                          |

#### Each DRAFT_SHOP / DRAFT_AUCTION phase

| Order | Event                       | Per-call seed count | Iteration order                                |
|-------|-----------------------------|---------------------|------------------------------------------------|
| 3     | `DrawShopSlot`              | 2-3 seeds per slot  | ascending `player_id` → ascending `slot_index` |

#### RESOLUTION phase (in this exact order)

<!-- ADR-018 amendment: Orders 4-6 added for keyword combat RNG (range_equidistant_select,
     teleport_random_dest, strich_change_lane_select). Former Orders 4-7 renumbered to 7-10. -->

| Order | Event                          | Per-call seed count | Iteration order                                            |
|-------|--------------------------------|---------------------|------------------------------------------------------------|
| 4     | `RangeEquidistantSelect`       | 1 per RANGE attack with equidistant targets | ascending `player_id` → ascending `lane` (SS3 RANGE+FIRST STRIKE, SS6 standard RANGE) |
| 5     | `TeleportRandomDest`           | 1 per TELEPORT activation | ascending `player_id` → ascending `lane` (within triggering sub-step) |
| 6     | `StrichChangeLaneSelect`       | 1 per Strich CHANGE LANE auto-activation | ascending `player_id` (after triggering sub-step) |
| 7     | `ResolveEcaflip`               | per Ecaflip card    | ascending `lane`                                           |
| 8     | `ResolvePrism`                 | per prism activation| ascending `player_id` → ascending `lane`                   |
| 9     | `AwardFakeObjectiveReward`     | per destroyed fake  | ascending `player_id` → ascending `lane`                   |
| 10    | `DrawFreeCard`                 | only if reward = free card | triggered conditionally by Order 9 (50/50 outcome) |

**Inter-player ordering for concurrent events** (used wherever the table above says "ascending player_id then ..."):

> ascending `player_id` → ascending `lane_index` → ascending `cell`

This three-level total order is the canonical tiebreak for any RNG site that processes multiple players' simultaneous events in a single phase. No system may invent an alternative ordering.

### 5. Audit Log Contract

Every RNG consumption MUST:
1. Capture `event_type` matching the call site (no generic "Misc" variant).
2. Record the `seed_index` value AT THE TIME OF CALL (before increment).
3. Record an encoded `result: Option<String>` representing the outcome. `None` is permitted only for outcomes that are themselves zero-information (e.g. an internal shuffle step whose output feeds another logged consumption).
4. Push the entry to `audit_log` in the same call as the RNG draw — never asynchronously, never best-effort.

The `result` encoding format is per-event-type and stable for the lifetime of the protocol version (e.g. `"lane=2,is_fake=true"` for `AssignFakeObjectives`). Format changes require an ADR amendment.

### 6. API Surface

`ServerRng` exposes only intent-named methods (one per `RngEvent` variant), never raw `next_u32` / `gen` access. Each method:
- accepts the parameters needed to construct the matching `RngEvent`
- performs the draw
- writes the audit entry
- increments `seed_index`
- returns the typed outcome (e.g. `bool` for fake assignment, `CardId` for shop, `EcaflipFace` for combat)

Raw `RngCore` access is private to the module. This prevents new consumers from skipping the audit log or violating the consumption order.

### 7. Forbidden

- No `rand::thread_rng()` anywhere in server game logic.
- No `StdRng`, `SmallRng`, or any non-`ServerRng` RNG type in server game logic.
- No RNG of any kind on the client for gameplay purposes.
- No transmission of seed bytes, `seed_index`, or `audit_log` entries in any production S2C message.
- No mid-session reseed.
- No interleaving of consumption sites that would violate the order table in §4.

## Alternatives Considered

**A — Per-system independent RNGs (rejected):**
Each system owns its own `ChaCha20Rng` seeded from a master seed. Cleaner ownership, but requires either (a) deriving sub-seeds deterministically (adds a layer of seed-derivation logic to audit) or (b) accepting that each sub-RNG diverges from any global ordering (breaks replay determinism unless every sub-RNG's consumption order is also documented). The audit log would have to be merged from N sources with timestamps, reintroducing the ordering problem we are trying to eliminate. Rejected.

**B — Per-event hash-based randomness (rejected):**
Compute outcomes by hashing `(session_seed, event_descriptor)`. Stateless, embarrassingly parallel, naturally deterministic. Rejected because (1) it makes "draw N cards from a deck without replacement" awkward — every shop slot needs a synthetic descriptor that encodes prior draws — and (2) the `event_descriptor` design becomes the new fragile contract, with the same risk of ordering bugs but spread across every call site rather than enforced at one resource.

**C — Client-seeded RNG with server verification (rejected):**
Client picks a seed, server verifies outcomes against a commitment scheme. Used in some peer-to-peer card games. Rejected outright: violates the "no client-side RNG" principle in `technical-preferences.md`, adds cryptographic complexity, and offers zero benefit over a server-authoritative model when the server is the sole authority anyway.

**D — `StdRng` with documented version pinning (rejected):**
`StdRng` currently wraps `ChaCha12Rng` but its backing type is explicitly not part of `rand`'s stability contract. A `rand` minor version bump could silently change replay outcomes. The cost of using `ChaCha20Rng` directly (one extra crate, two extra rounds per draw — negligible) is far below the cost of a future invisible replay-breaking change. Rejected.

**E — `SmallRng` for performance (rejected):**
Non-deterministic across platforms (different SIMD paths produce different streams). Rejected; cross-host replay is required.

**F — `ServerRng` chosen (this ADR):**
Single-resource, explicit consumption order, audit log at every site, intent-named API. Provides strong fairness guarantees, replayability, and a clear cheating-investigation path with negligible runtime cost.

## Consequences

**Positive:**
- Fairness is guaranteed by construction: clients cannot influence, observe, or predict outcomes.
- Determinism: `(seed, consumption order, audit log)` is sufficient to fully replay a session for debugging or dispute resolution.
- Centralised ownership: one resource, one module, one set of code paths to review for randomness correctness.
- Cheating investigation is tractable: the audit log linearises every random event with its seed index and outcome.
- Refactor safety: intent-named API methods make grep/audit ("who calls `ResolveEcaflip`?") trivial.

**Negative / Tradeoffs:**
- Strict consumption order is a global invariant, not enforceable by the type system. Reordering systems unintentionally breaks replay determinism without breaking gameplay correctness — a silent failure mode the test suite must catch (see Validation Criteria §1, §2).
- The audit log grows linearly with session length. At expected rates (~hundreds of entries per match) this is bounded; we do not need ring-buffering for v1, but persistence-to-disk policy must be defined before audit logs are retained off-session.
- Adding any new RNG consumer requires (a) a new `RngEvent` variant, (b) a new entry in the §4 order table, (c) an ADR amendment. This is intentional friction to prevent ad-hoc RNG creeping into the codebase.

## Risks

| Risk | Severity | Mitigation |
|---|---|---|
| A new system adds RNG consumption without registering in §4 ordering | HIGH | (1) Make `ChaCha20Rng` and `RngCore` access private to the module; force all consumers through intent-named API. (2) CI-level grep for forbidden imports of `rand::thread_rng`, `StdRng`, `SmallRng` outside the rng module. (3) Architecture review gate flags new RNG usage. |
| Consumption order silently violated by system schedule reorder | HIGH | Integration test that runs a full session with a fixed seed and asserts the exact audit-log sequence (event types + seed indices). Any reorder breaks the test. |
| Client crate accidentally pulls `rand` via transitive dependency for gameplay | MEDIUM | CI check: client `Cargo.lock` audit excludes `rand` / `rand_chacha` from the client crate's dependency tree for gameplay modules. Cosmetic-only modules that need jitter use an explicitly named alternative crate. |
| Audit log encoding format drift between game versions makes old logs unreadable | LOW | Encoding format is documented per-event in the `RngEvent` impl. Version the audit log header if format changes. |
| `OsRng::from_entropy()` panics on a constrained server platform | LOW | Railway hosts have `/dev/urandom`. If we ever target a platform without it, this becomes blocking — out of scope for current deployment target. |
| Future Lightyear update changes server-side `Resource` insertion timing relative to `SessionReady` | LOW | Session lifecycle ADR (ADR-007) owns the precise insertion point; this ADR depends on that contract. |

## Performance Implications

- **Frame cost**: ChaCha20 produces 64 random bytes per block; a single `next_u64` is sub-microsecond. Even at peak (RESOLUTION phase processing every lane × every player) total RNG work per round is well below 100 microseconds — far inside the < 2 ms server-frame game-logic budget from `technical-preferences.md`.
- **Memory cost**: `ChaCha20Rng` state is ~136 bytes. `AuditEntry` averages ~32-64 bytes; at hundreds of entries per match the audit log is < 32 KB per session. Bounded and trivial.
- **Network cost**: Zero. No RNG state is ever transmitted. Only outcomes (which would be transmitted regardless of how randomness is computed) appear on the wire.
- **WASM bundle**: Zero. `rand`/`rand_chacha` are server-only crates; the client WASM bundle is unaffected, preserving the < 50 MB budget.

## Migration Plan

This is a greenfield decision; no migration from a prior implementation is required.

**Adoption sequence:**
1. Land `server/src/foundation/rng.rs` with `ServerRng`, `AuditEntry`, `RngEvent`, intent-named API stubs returning placeholder outcomes.
2. Wire `ServerRng` insertion/removal into the Game Session lifecycle (blocked on ADR-007).
3. Implement Objective System fake assignment (Order 1) — first real consumer; validates the API and audit log end-to-end.
4. Implement Card Pool draft and shop draws (Orders 2, 3).
5. Implement keyword combat RNG consumers (Orders 4–6): `RangeEquidistantSelect`, `TeleportRandomDest`, `StrichChangeLaneSelect`.
6. Implement Ecaflip and Prism consumers (Orders 7, 8).
7. Implement fake-objective reward + conditional free card draw (Orders 9, 10).
8. Land the integration test that asserts a fixed-seed session produces a fixed audit-log sequence.

Each step requires a passing test before the next is unlocked.

## Validation Criteria

This ADR is correct if and only if all of the following hold:

1. **Determinism test**: Running a scripted session with a fixed seed (injected via a test-only constructor) produces the same `audit_log` byte-for-byte across runs and across host machines (Linux x86_64 and the Railway production target).
2. **Consumption-order test**: The same scripted session produces an `audit_log` whose `(event_type, seed_index)` sequence matches the order specified in §4 exactly. Any system reorder that perturbs this sequence fails the test.
3. **Module boundary test**: A static-analysis or grep-based CI check confirms zero usages of `rand::thread_rng`, `StdRng`, `SmallRng`, or direct `ChaCha20Rng` construction outside `server/src/foundation/rng.rs`.
4. **Client isolation test**: The client crate's resolved dependency graph contains no `rand` or `rand_chacha` reachable from gameplay modules.
5. **Seed non-transmission test**: Protocol-level test asserts no S2C message variant contains a field of type `[u8; 32]`, `Seed`, or any name matching `seed`/`rng` in production builds.
6. **Lifecycle test**: `ServerRng` is present in the server `World` between `SessionReady` and `GameOverEmitted`, and absent before/after.

We will know this ADR was the right call if: (a) every gameplay RNG bug discovered during development is reproducible from the audit log alone, without re-running the full match, and (b) no playtest or production session reports an "unfair shop" or "predictable fake assignment" issue traceable to RNG architecture.

## GDD Requirements Addressed

- `design/gdd/server-rng.md` — Rules 1 through 6 (server-only authority, ChaCha20 backing, no transmission, audit log, lifecycle, ordering)
- `design/gdd/server-rng.md` — Formulas F1 (seed advancement) and F2 (per-event consumption count)
- `design/gdd/server-rng.md` — Seed table (the §4 consumption order in this ADR is the canonical source for that table)

**Technical Requirements covered:** TR-RNG-01 through TR-RNG-06.

## Related

- **ADR-001** — Hidden Objective Identity via Targeted Unicast. Consumes `AssignFakeObjectives` outcomes (Order 1) to populate `HiddenObjectives`; the unicast in ADR-001 carries those outcomes to the owning client.
- **ADR-003** (workspace) — Defines the server crate where `ServerRng` lives.
- **ADR-007** (pending) — Game Session lifecycle owns `ServerRng` creation/destruction.
- `design/gdd/objective-system.md` — Consumer of Order 1 (`AssignFakeObjectives`) and Order 9 (`AwardFakeObjectiveReward`).
- `design/gdd/card-data-pool.md` — Consumer of Orders 2 and 3 (draft and shop draws) and Order 10 (free card draw).
- `design/gdd/round-state-machine.md` — Defines the phase transitions (DRAFT_INITIAL, DRAFT_SHOP, DRAFT_AUCTION, RESOLUTION) referenced by the consumption order table.
- `.claude/docs/technical-preferences.md` — "Forbidden Patterns" section: this ADR is the operational realisation of the "no client-side RNG" principle.
