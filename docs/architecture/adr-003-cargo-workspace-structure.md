# ADR-003: Cargo Workspace Structure and Crate Split

## Status

Accepted

## Date

2026-04-29

## Last Verified

2026-04-29

## Decision Makers

User + technical-director (architecture), with input from network-programmer
(protocol crate boundary), devops-engineer (build/deploy implications), and
lead-programmer (module ownership within `server/` and `client/`).

## Summary

Lanes and Lies needs a project structure that compile-enforces the
client/server authority model (ADR-002) and prevents server-only secrets from
leaking into the WASM client bundle. Decision: a three-crate Cargo workspace
(`shared/`, `server/`, `client/`) where `shared/` holds protocol types only
(no Bevy plugins, no `Resource` derives), `server/` and `client/` both depend
on `shared/` but never on each other, and both targets register the same
Lightyear protocol from `shared/` to guarantee message-format symmetry.

## Engine Compatibility

| Field | Value |
|-------|-------|
| **Engine** | Bevy 0.18 + Lightyear 0.26 |
| **Domain** | Core / Networking / Build |
| **Knowledge Risk** | MEDIUM — Bevy 0.18 feature flags and Lightyear 0.26 feature gates verified against `docs/engine-reference/bevy/VERSION.md` and `liv-bevy-018` skill notes (2026-04-28); minor drift possible if 0.18.x patches change default feature sets |
| **References Consulted** | `docs/engine-reference/bevy/VERSION.md`, `.claude/docs/technical-preferences.md`, Lightyear 0.26 release notes (Jan 2026), Bevy 0.18 cargo features matrix |
| **Post-Cutoff APIs Used** | Bevy 0.18 `default-features = false` + selective feature flags (`bevy_ecs`, `bevy_ui`, `bevy_sprite`, `bevy_text`, `bevy_asset`, `multi_threaded`, `webgl2`, `mouse`, `keyboard`, `serialize`); Lightyear 0.26 `shared` / `server` / `client` feature gates |
| **Verification Required** | (a) `cargo build -p shared` succeeds without pulling `bevy_ecs`; (b) `cargo build -p client --target wasm32-unknown-unknown` succeeds and bundle stays ≤ 50 MB after `--release` + LTO; (c) `cargo build -p server` produces a headless binary with no windowing/rendering deps; (d) confirm `bevy_asset_loader` 0.18-compatible version on crates.io before pinning |

> **Note**: Knowledge Risk is MEDIUM. If the project upgrades to Bevy 0.19+ or
> Lightyear 0.27+, this ADR must be re-validated — feature flag names and
> default sets are the most common breakage point across Bevy minor versions.

## ADR Dependencies

| Field | Value |
|-------|-------|
| **Depends On** | ADR-002 (client-server authority model — the crate split is the compile-time enforcement of that runtime authority decision) |
| **Enables** | ADR-004 (asset loading — `client/` foundation), ADR-005 (server RNG seeding — `server/foundation/rng.rs`), ADR-009 (Round State Machine — lives in `server/core/rsm/`) |
| **Blocks** | All implementation epics. No `.rs` file outside throwaway `prototypes/` should be written until this workspace exists |
| **Ordering Note** | This ADR must be implemented (workspace scaffolded, all three crates compile empty) before the first gameplay story (`STORY-001`) is opened. ADR-002 should be Accepted first; if ADR-002 changes the authority model, this ADR's three-crate split must be re-evaluated |

## Context

### Problem Statement

We need to start writing code for Lanes and Lies. Before any module is created
we must answer: how is the codebase physically partitioned?

This is a hidden-information multiplayer card game. Two failure modes are
catastrophic and silent:

1. **Secret leakage**: server-only data (e.g. `HiddenObjectives` resource from
   ADR-001, RNG seed, opponent hand contents) accidentally compiles into the
   WASM client bundle. A reverse-engineer could extract it.
2. **Protocol divergence**: server and client diverge on the wire format of a
   message. Lightyear deserialisation fails (or worse, succeeds with garbage)
   only at runtime, on the deployed build, in production.

Both must be prevented at **compile time**, not by code-review discipline.
Convention alone is insufficient for a project with 48 agents touching the
codebase asynchronously.

The cost of deciding wrong: a single-crate or poorly-bounded split would
require an invasive workspace refactor mid-project, touching every `use`
statement and every `Cargo.toml`. Deciding now is cheap; deciding in M2 is
weeks of work.

### Current State

The repository contains no Rust crates yet. `Cargo.toml` does not exist at
the workspace root. All design work to date is in `design/gdd/` and
`docs/architecture/`. This is a greenfield decision.

### Constraints

- **Engine**: Bevy 0.18 (Required Components API, post-0.15 patterns only).
  Bevy 0.18 supports `default-features = false` and selective feature flags,
  which is required to keep `shared/` minimal.
- **Networking**: Lightyear 0.26 exposes `client`, `server`, and `shared`
  feature gates. Protocol/channel/message registration is meant to be done
  once and shared — this is the library's idiomatic split.
- **Targets**: client is `wasm32-unknown-unknown` deployed to Vercel via
  Trunk; server is native (likely `x86_64-unknown-linux-gnu`) deployed to
  Railway via Docker. Two distinct toolchain invocations.
- **Bundle budget**: WASM client ≤ 50 MB (release + LTO + strip). Pulling
  server-only deps into the client would blow this. `tokio`, `ron`,
  `rand_chacha`, server-side Lightyear features must NOT be reachable from
  `client/`.
- **Team workflow**: 48 agents, file-extension-based routing. The crate a
  file lives in must be unambiguous from its path.
- **Reversibility cost**: Changing the workspace layout after M1 is
  expensive — every import path breaks. Decide once, decide well.

### Requirements

- **Compile-enforced authority boundary**: it must be a compile error for
  `client/` code to import a server-only type, and vice versa.
- **Single source of truth for protocol**: every `C2S*` and `S2C*` message
  type, every channel definition, and every Lightyear protocol registration
  must live in exactly one crate, consumed unchanged by both sides.
- **Lean shared crate**: `shared/` must compile with the smallest possible
  Bevy feature set. It should not pull `bevy_ecs`, `bevy_render`, `bevy_ui`,
  or any plugin-related code. Adding a heavy dep to `shared/` should require
  an explicit ADR amendment.
- **No circular dependencies**: `server/` and `client/` must not depend on
  each other. Both depend on `shared/`. Within `server/`, layers must form
  a DAG.
- **Build performance**: changes to `client/` must not trigger `server/`
  rebuilds and vice versa. `cargo check -p client` must be fast (< 30s
  cold, < 5s incremental on a modern machine).
- **Trunk compatibility**: `client/` must be buildable as the Trunk entry
  point with `index.html` co-located (or referenced) at the crate root.

## Decision

A **three-crate Cargo workspace** at the repository root:

```
lanes-and-lies/
├── Cargo.toml              workspace root (members declaration only)
│
├── shared/                 PROTOCOL TYPES ONLY — zero Bevy plugin deps
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── protocol.rs     C2S/S2C message types, channel defs,
│       │                   Lightyear protocol registration fn
│       ├── card.rs         CardData, CardId, Rarity, ClassId, CardType
│       └── config.rs       GameConfig struct (NO #[derive(Resource)])
│
├── server/                 HEADLESS BEVY APP → Railway (Docker)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── foundation/     Plugin setup, asset loading, RNG seeding,
│       │                   GameConfig→Resource wrapping
│       ├── core/           rsm/      (Round State Machine)
│       │                   session/  (game session lifecycle)
│       │                   economy/  (gold, interest, banking)
│       │                   pool/     (card pool, shop generation)
│       └── feature/        board/    (lanes, units, placement)
│                           objective/ (HP, identity, Sang Méprise)
│                           [M2+] auction/, combat/, draft/
│
└── client/                 WASM BEVY APP → Vercel (Trunk)
    ├── Cargo.toml
    ├── index.html          Trunk entry point
    └── src/
        ├── main.rs
        ├── network/        Lightyear client plugin, message dispatch,
        │                   reconnect/snapshot handling
        ├── state/          Client-side state mirror (read-only views;
        │                   caches received unicast data, e.g.
        │                   HiddenObjectives projection from ADR-001)
        └── ui/             [M2+] board/, hand/, shop/, hud/, anim/
```

Hard rules:

1. **`shared/` ban list**: NO `#[derive(Resource)]`, NO `Plugin` impls, NO
   `App::add_systems`, NO Bevy queries. Pure data + serde + Lightyear
   protocol registration. The `bevy` dep in `shared/Cargo.toml` is
   `default-features = false, features = ["serialize"]` ONLY.
2. **`GameConfig` lives in `shared/config.rs`** as a plain serde struct
   without `#[derive(Resource)]`. The server-side wrapper at
   `server/foundation/config.rs` does:
   `app.insert_resource(shared::config::GameConfig::load(...))`. The client
   may also insert it as a resource on its own side if needed for UI display
   formulas — same struct, two independent resource registrations.
3. **Lightyear protocol registration in `shared/`**: a single
   `pub fn register_protocol(app: &mut App)` function in
   `shared/src/protocol.rs` registers all channels and messages. Both
   `server/main.rs` and `client/main.rs` call it. There is no second
   registration site.
4. **No cross-target deps**: `server/Cargo.toml` MUST NOT list `client` as
   a dependency. `client/Cargo.toml` MUST NOT list `server`. Workspace
   `members` is the only place both names appear.
5. **Server internal layering**: within `server/src/`, `feature/` may import
   from `core/`, and `core/` may import from `foundation/`. Reverse
   directions are forbidden (`foundation/` MUST NOT import from `core/` or
   `feature/`). This is a code-review rule, not yet a compile rule —
   crate-internal modules cannot enforce DAG layering at compile time
   without further splitting. If violations recur, escalate to splitting
   `server/` into sub-crates.
6. **Client internal layering**: `ui/` may import from `state/`, `state/`
   may import from `network/`, `network/` may import from `shared/`.
   Reverse directions forbidden.
7. **Forbidden in `client/`**: any use of `rand` outside deterministic
   visual jitter (e.g. particle wobble seeded from frame count). All
   gameplay randomness comes from the server. `rand_chacha` should not
   appear in `client/Cargo.toml` at all.

### Architecture

```
                    +----------------------+
                    |     shared/          |
                    |  (no Bevy plugins)   |
                    |                      |
                    |  - protocol.rs       |
                    |    C2S* / S2C* msgs  |
                    |    register_protocol |
                    |  - card.rs           |
                    |    CardData, CardId  |
                    |  - config.rs         |
                    |    GameConfig (POD)  |
                    +----------+-----------+
                               |
                  depends on   |   depends on
            +------------------+------------------+
            |                                     |
            v                                     v
  +-------------------+                 +-------------------+
  |     server/       |                 |     client/       |
  |  (headless Bevy)  |                 |   (WASM Bevy)     |
  |  → Railway/Docker |    NO DIRECT    |   → Vercel/Trunk  |
  |                   |    DEP EITHER   |                   |
  |  feature/         |    DIRECTION    |  ui/   [M2+]      |
  |     ↑            |                  |    ↑              |
  |  core/            |                 |  state/           |
  |     ↑            |                  |    ↑              |
  |  foundation/      |                 |  network/         |
  +-------------------+                 +-------------------+
            |                                     |
            +------------ both call --------------+
                          shared::protocol
                          ::register_protocol(app)
                          at startup

Lightyear wire:
  client  --[C2S messages over WebSocket]-->  server
  client  <--[S2C messages over WebSocket]--  server
  (both sides serialize/deserialize using identical types from shared/)
```

### Key Interfaces

**Workspace `Cargo.toml`** (root):

```toml
[workspace]
resolver = "2"
members = ["shared", "server", "client"]

[workspace.package]
edition = "2021"
version = "0.1.0"
license = "PROPRIETARY"

# Centralised dependency versions — child crates inherit via `workspace = true`
[workspace.dependencies]
bevy = "0.18"
lightyear = "0.26"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
ron = "0.8"
rand = "0.9"
rand_chacha = "0.3"
bevy_tweening = "0.18"
# bevy_asset_loader: pin after verifying 0.18-compatible release on crates.io
```

**`shared/Cargo.toml`**:

```toml
[package]
name = "shared"
edition.workspace = true
version.workspace = true

[dependencies]
# Bevy with serialize ONLY — no ECS, no plugins, no rendering.
bevy = { workspace = true, default-features = false, features = ["serialize"] }
# Lightyear's `shared` feature: protocol/channel/message types, no transport.
lightyear = { workspace = true, default-features = false, features = ["shared"] }
serde = { workspace = true }
```

**`server/Cargo.toml`**:

```toml
[package]
name = "server"
edition.workspace = true
version.workspace = true

[dependencies]
shared = { path = "../shared" }
# Headless Bevy: ECS + multi-threading, no rendering / windowing / UI.
bevy = { workspace = true, default-features = false, features = [
    "multi_threaded",
    # Note: "bevy_ecs" is NOT a valid Bevy 0.18 feature. Removed 2026-04-30.
    # Verified by CI commit 88971ec. Headless server only needs "multi_threaded".
] }
# Lightyear server transport over WebSocket.
lightyear = { workspace = true, default-features = false, features = [
    "server",
    "websocket",
] }
serde = { workspace = true }
serde_json = { workspace = true }
ron = { workspace = true }
rand = { workspace = true }
rand_chacha = { workspace = true }

[[bin]]
name = "lanes-and-lies-server"
path = "src/main.rs"
```

**`client/Cargo.toml`**:

```toml
[package]
name = "client"
edition.workspace = true
version.workspace = true

[dependencies]
shared = { path = "../shared" }
# Full Bevy minus default-features so we can pick web-friendly subset.
# Note: precise feature names verified against Bevy 0.18 cargo features matrix —
# adjust if a feature has been renamed in the 0.18 patch series.
bevy = { workspace = true, default-features = false, features = [
    "bevy_ui",
    "bevy_sprite",
    "bevy_text",
    "bevy_asset",
    "bevy_winit",
    "webgl2",
    "x11",         # harmless on wasm; required for native dev/debug target
] }
# Lightyear client transport over WebSocket (browser-compatible).
lightyear = { workspace = true, default-features = false, features = [
    "client",
    "websocket",
] }
bevy_tweening = { workspace = true }
# bevy_asset_loader = { version = "<verify>", default-features = false }
serde = { workspace = true }

[[bin]]
name = "lanes-and-lies-client"
path = "src/main.rs"
```

> **Verify before merge**: Bevy 0.18 input handling moved behind features
> (per `VERSION.md`). Confirm the exact feature names for mouse/keyboard
> input on 0.18 — they may be `bevy_input` (single feature) rather than
> separate `mouse` / `keyboard` flags as drafted. Update this Cargo.toml
> in the implementation PR; the technical-director gate must verify
> against `docs/engine-reference/bevy/VERSION.md` at that point.

**`shared/src/protocol.rs` interface (sketch):**

```rust
use bevy::app::App;
use serde::{Deserialize, Serialize};
use lightyear::prelude::*;

// All wire types live here.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct C2SPlaceUnit { /* ... */ }

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct S2CRoundResolved { /* ... */ }

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct S2CObjectiveIdentities { /* per ADR-001 */ }

// Single registration entry point. Called by BOTH server and client at startup.
pub fn register_protocol(app: &mut App) {
    // app.register_message::<C2SPlaceUnit>(ChannelDirection::ClientToServer);
    // app.register_message::<S2CRoundResolved>(ChannelDirection::ServerToClient);
    // app.register_message::<S2CObjectiveIdentities>(ChannelDirection::ServerToClient);
    // ... channel registrations ...
    // (Exact Lightyear 0.26 API symbols verified at implementation time
    //  via liv-bevy-lightyear skill.)
}
```

**`shared/src/config.rs` (POD struct, no Resource):**

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameConfig {
    pub max_hand_size: u32,
    pub objective_hp: u32,
    pub starting_gold: u32,
    // ... see design/gdd/game-config.md for full schema ...
}
// NO #[derive(Resource)] here — that lives in server/foundation/config.rs
// where Bevy's Resource trait is in scope.
```

### Implementation Guidelines

- **Workspace scaffolding story**: the first implementation story
  (`STORY-001` or equivalent) creates the workspace root, all three
  `Cargo.toml` files, empty `src/lib.rs` / `src/main.rs` per crate, and
  proves `cargo check --workspace` succeeds with zero warnings on a clean
  clone. No gameplay code in this story.
- **CI gate**: add a CI job that runs `cargo check -p shared` in isolation
  and verifies the resulting dependency tree (via `cargo tree -p shared`)
  contains NO `bevy_ecs`, `bevy_render`, `bevy_ui`, `tokio`, or any
  server-only crate. If `shared/` ever pulls these, the build fails. This
  is the compile-time guard against accidental scope creep.
- **CI gate (bundle size)**: measure WASM bundle size on every PR; fail if
  > 50 MB. This catches accidental client-side bloat early.
- **No `pub use` shortcuts in `shared/`**: keep module paths explicit
  (`shared::protocol::C2SPlaceUnit`) so it is obvious where types live.
- **Adding a dependency to `shared/`** requires an ADR amendment and
  technical-director approval. Server-only and client-only deps may be
  added by lead-programmer alone via standard PR review.
- **Module ownership** (matches file-extension routing in
  `.claude/docs/technical-preferences.md`):
  - `shared/src/**.rs` → `network-programmer` (protocol stewardship) +
    `gameplay-programmer` (card/config data shape)
  - `server/src/**.rs` → `gameplay-programmer` + `network-programmer`
    (network module only)
  - `client/src/network/**.rs` → `network-programmer`
  - `client/src/ui/**.rs` → `ui-programmer`
  - `client/src/state/**.rs` → `gameplay-programmer`
- **Trunk config**: `client/Trunk.toml` (or `client/index.html` with Trunk
  defaults) lives inside `client/`. Trunk is invoked from the `client/`
  directory; `cargo` workspace discovery still works because Trunk reads
  `Cargo.toml` resolver config.
- **Profile config in workspace root**:
  ```toml
  [profile.release]
  lto = "thin"
  codegen-units = 1
  strip = "symbols"
  panic = "abort"   # smaller WASM, acceptable for a multiplayer client

  [profile.dev]
  opt-level = 1     # Bevy is unusable at opt-level=0
  ```

## Alternatives Considered

### Alternative 1: Single Crate with `cfg` Flags

- **Description**: One crate with `#[cfg(feature = "server")]` and
  `#[cfg(feature = "client")]` gating. Build twice with different
  feature sets.
- **Pros**: Simpler workspace; no path manipulation; one `Cargo.toml`;
  trivially shared types (everything is in the same module tree).
- **Cons**: The compiler does NOT enforce the client/server boundary —
  a misplaced `cfg` attribute (or an item with no `cfg` at all) compiles
  into both targets. Server-only secrets like `HiddenObjectives` resource
  contents could silently land in the WASM bundle. Code review is the only
  guard, and 48 agents touching the codebase makes review-as-guard
  unreliable. Additionally, IDE tooling (rust-analyzer) struggles with
  multi-cfg crates: the same file shows different errors depending on which
  feature set is active. `cargo check` covers only one feature set per
  invocation, doubling CI time and creating combinatorial holes.
- **Estimated Effort**: ~30% less initial setup than the chosen approach.
- **Rejection Reason**: For a hidden-information game where secret leakage
  is silent and catastrophic, "the compiler enforces the boundary" is
  worth more than the setup savings. ADR-001 already established that we
  treat hidden-information failures as code-red. This decision is
  consistent with that posture.

### Alternative 2: Two Crates (No Shared)

- **Description**: `server/` and `client/` only. Protocol types defined in
  one (say, `server/protocol.rs`) and **copied** into the other, with a
  CI lint or proc-macro to detect divergence.
- **Pros**: Maximum isolation — server and client are entirely independent
  build trees. Easiest to reason about per-target dependency hygiene.
- **Cons**: Hand-copying serde types is the canonical recipe for production
  outages in client/server games. Any change to a `C2S*` or `S2C*` message
  must be made in two places, and the failure mode of forgetting the
  second place is silent at compile time and only manifests when a
  serialised message hits a deserialiser with a different schema. CI lints
  detecting "diff between two files" are brittle (formatting changes
  trigger false positives; sub-struct refactors are missed). Lightyear
  protocol registration would need to be duplicated, which violates the
  library's intended idiom.
- **Estimated Effort**: ~10% less than chosen approach (one fewer crate
  to scaffold), but ~3x more long-term maintenance burden.
- **Rejection Reason**: The whole reason `shared/` exists in
  client/server architectures is to make protocol divergence a compile
  error. Discarding it forfeits the most valuable compile-time guarantee
  this layout provides.

### Alternative 3: Four Crates (`shared`, `server`, `client`, `protocol`)

- **Description**: Split protocol/channels/messages into a dedicated
  `protocol/` crate, leaving `shared/` for non-network common types
  (cards, config). `server` and `client` depend on both.
- **Pros**: Even tighter scope on `protocol/` — Lightyear bumps only
  rebuild that crate. Clearer ownership boundary for `network-programmer`.
- **Cons**: Marginal benefit at our scale. `shared/` is already small
  enough that splitting it again creates `Cargo.toml` overhead without
  meaningful build-time or clarity wins. Card data types and protocol
  message types are tightly coupled (a `C2SPlaceUnit` references a
  `CardId`), so the split would either re-introduce a dependency edge
  (`protocol → shared`) or duplicate `CardId`. Neither is appealing.
- **Estimated Effort**: ~15% more than chosen approach.
- **Rejection Reason**: Premature subdivision. Revisit if `shared/`
  exceeds ~5000 LOC or builds become a bottleneck. Easy to extract later
  — moving modules from `shared/` to a new `protocol/` crate is a
  mechanical refactor.

## Consequences

### Positive

- **Compile-enforced authority boundary**: `client/` cannot accidentally
  see server-only state. Secret leakage is no longer a code-review concern;
  it is a compile error.
- **Single source of truth for wire format**: protocol types are defined
  once in `shared/` and consumed identically by both sides. Lightyear
  protocol registration runs from the same function on both ends —
  divergence is impossible by construction.
- **Independent target builds**: server and client compile, test, and
  deploy independently. CI can fan out: `cargo check -p server` and
  `cargo build -p client --target wasm32-unknown-unknown` run in parallel.
- **Bundle hygiene**: WASM client cannot accidentally pull `tokio`,
  `rand_chacha`, or server-only Lightyear features. The 50 MB bundle
  budget is structurally easier to defend.
- **Clear file-routing**: agents writing `.rs` files know exactly which
  crate it belongs in based on the path — `shared/src/`, `server/src/`,
  or `client/src/`. Matches the file-extension routing table in
  `.claude/docs/technical-preferences.md`.
- **Future-proof against more targets**: adding a `bot/` crate (CPU
  opponent), a `replay/` crate (offline replay viewer), or a `tools/`
  crate (balance simulator) is a clean extension — they all depend on
  `shared/` alone.

### Negative

- **More boilerplate**: three `Cargo.toml` files instead of one. Cross-crate
  imports are slightly more verbose.
- **`GameConfig` Resource registration is split**: the struct lives in
  `shared/config.rs` without `Resource` derive; the server (and possibly
  client) wraps it locally. Mildly awkward, but unavoidable given the
  goal of keeping `shared/` plugin-free.
- **Cold build time**: three separate compilation units add ~10-20s to a
  cold `cargo build --workspace` compared to a single crate. Incremental
  builds are unaffected (and arguably faster, since changes localise).
- **Trunk integration**: Trunk operates on a single crate. Running it from
  the `client/` directory is straightforward but means any tooling that
  expects the `Cargo.toml` to be at the repo root must be configured.
  Acceptable cost.

### Neutral

- **Server-internal layering (`foundation` → `core` → `feature`) is a
  convention, not a compile rule**. Within a single crate, modules can
  import each other freely. We accept this for now; if violations recur,
  the next step is splitting `server/` into sub-crates (`server-core`,
  `server-feature`). That escalation is intentionally deferred.
- **Client `state/` module mirrors some server state shapes**. This is by
  design — the client needs read-only views of, e.g., `HiddenObjectives`
  data delivered via the unicast in ADR-001. The mirror lives in `client/`
  and depends on `shared/` types; it is not a duplication concern.

## Risks

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|-----------|
| `shared/` accumulates Bevy plugin deps over time, defeating the lean-protocol intent | MEDIUM | HIGH | CI job runs `cargo tree -p shared` and fails if disallowed crates appear; dependency additions to `shared/` require ADR amendment |
| Lightyear 0.26 `shared` feature does not cleanly separate from `server`/`client` features (i.e. registering protocol pulls transport code) | MEDIUM | MEDIUM | Verified via `liv-bevy-lightyear` skill at implementation time; if confirmed, fall back to defining message types in `shared/` and doing per-side registration in `server/` and `client/` separately. Worst case: protocol structs live in `shared/`, registration calls duplicate — still better than full divergence |
| Bevy 0.18 input feature flag names drift from drafted Cargo.toml (`mouse` / `keyboard` may not exist as separate features) | MEDIUM | LOW | Reconcile against `docs/engine-reference/bevy/VERSION.md` and Bevy 0.18 cargo features matrix in the workspace-scaffolding implementation PR; technical-director gate verifies before merge |
| `bevy_asset_loader` does not yet have a 0.18-compatible release | LOW | MEDIUM | Verify on crates.io before pinning; if missing, fall back to manual `AssetServer` loading in `client/network/` and `server/foundation/` until upstream catches up |
| Server-internal layering (`foundation`→`core`→`feature`) is silently violated because compiler does not enforce intra-crate DAG | MEDIUM | MEDIUM | Code-review checklist; if violations recur in practice, split `server/` into `server-foundation`, `server-core`, `server-feature` sub-crates (deferred ADR) |
| WASM bundle exceeds 50 MB after Bevy 0.18 + Lightyear 0.26 + tweening + asset_loader are all included | MEDIUM | HIGH | CI bundle-size gate fails the build at > 50 MB; mitigation path is selective Bevy feature pruning (drop `bevy_pbr`, `bevy_render` 3D modules, audio if unused) and `wasm-opt` post-processing |
| Workspace root `Cargo.toml` profile settings interact badly with Trunk's own optimisation passes | LOW | LOW | Validate Trunk release build matches expected size and behaviour during workspace scaffolding story |

## Performance Implications

| Metric | Before | Expected After | Budget |
|--------|--------|---------------|--------|
| CPU (frame time) | N/A (no code) | No change vs. single-crate; workspace is build-time only | 16.67ms total / < 2ms game logic / < 12ms render |
| Memory (server) | N/A | < 100 MB resident per game session | (no formal budget yet) |
| Memory (client WASM heap) | N/A | < 256 MB | 256 MB |
| Load Time (WASM cold load) | N/A | Target < 5s on broadband | (no formal budget yet) |
| Network | N/A | < 1 KB per round message | < 1 KB per round message |
| **Build time (cold, full workspace)** | N/A | ~60-90s on a modern dev machine | (no hard budget) |
| **Build time (incremental, single crate touched)** | N/A | < 5s `cargo check -p <crate>` | < 30s |
| **WASM bundle size (release + LTO + strip)** | N/A | Target < 40 MB; budget 50 MB | < 50 MB |

The workspace structure itself has zero runtime performance cost — it is
purely a compile-time organisation. Build-time costs are the only meaningful
performance dimension affected, and the three-way split is expected to
*improve* iterative build times for typical single-crate edits.

## Migration Plan

This is a greenfield decision; there is no existing codebase to migrate.
The plan below is the **scaffolding** plan.

1. **Create workspace skeleton**:
   - Create `Cargo.toml` at `d:\_DEV\claude-code-game-studios\Cargo.toml`
     with the workspace `members` declaration and shared profile config.
   - Create `shared/Cargo.toml`, `shared/src/lib.rs` (empty `pub mod`s).
   - Create `server/Cargo.toml`, `server/src/main.rs` (empty `fn main()`),
     and the `foundation/`, `core/`, `feature/` subdirectories with
     placeholder `mod.rs` files.
   - Create `client/Cargo.toml`, `client/src/main.rs` (empty `fn main()`),
     and the `network/`, `state/`, `ui/` subdirectories.
   - Add `client/index.html` for Trunk.
   - Verify: `cargo check --workspace` succeeds with zero warnings.

2. **Verify dependency hygiene**:
   - Run `cargo tree -p shared` and confirm no `bevy_ecs`, no
     `tokio`, no server-only Lightyear features in the tree.
   - Run `cargo tree -p client` and confirm no `tokio`, no
     `rand_chacha`.
   - Run `cargo tree -p server` and confirm no rendering / UI / windowing
     crates.

3. **Add CI gates**:
   - GitHub Actions job: `cargo fmt --check`, `cargo clippy --workspace
     --all-targets -- -D warnings`, `cargo check --workspace`.
   - Dependency-tree gate: a script asserts `cargo tree -p shared --prefix
     none` does not contain disallowed crate names.
   - WASM bundle-size gate: build `client` for `wasm32-unknown-unknown`,
     measure the `.wasm` artefact, fail if > 50 MB.

4. **Wire in Lightyear protocol skeleton**:
   - Define one no-op message in `shared/protocol.rs` (e.g.
     `S2CHeartbeat`) and the `register_protocol` function.
   - Both `server/main.rs` and `client/main.rs` call
     `shared::protocol::register_protocol(&mut app)` and verify the build
     succeeds. This proves the pattern works end-to-end before the first
     real message is added.

5. **Open the gameplay backlog**:
   - Once steps 1-4 are green on CI, mark this ADR's blocked epics as
     unblocked. Story authors may begin writing implementation stories
     against the now-existing crate structure.

**Rollback plan**: if a fundamental incompatibility emerges (e.g. Lightyear
0.26's `shared` feature pulls server transport code, breaking the WASM
build), the fallback is Alternative 2 (two crates, duplicated protocol
types). Migration cost from this state to that one is mechanical: delete
`shared/`, copy its contents into both `server/` and `client/`, add a CI
diff-check between the two copies. Not desirable, but bounded. We would
write a superseding ADR documenting the failure mode before doing so.

## Validation Criteria

- [ ] `cargo check --workspace` succeeds on a clean clone with zero
      warnings.
- [ ] `cargo tree -p shared --prefix none` does NOT contain `bevy_ecs`,
      `bevy_render`, `bevy_ui`, `bevy_winit`, `tokio`, or any
      `lightyear` feature gated on `server` or `client`.
- [ ] `cargo tree -p client --prefix none` does NOT contain `tokio` or
      `rand_chacha`.
- [ ] `cargo tree -p server --prefix none` does NOT contain `bevy_render`,
      `bevy_ui`, or `bevy_winit`.
- [ ] `cargo build -p client --target wasm32-unknown-unknown --release`
      produces a `.wasm` artefact ≤ 50 MB.
- [ ] Both `server/main.rs` and `client/main.rs` call
      `shared::protocol::register_protocol(&mut app)`; removing the call
      from one side and adding a real message causes a runtime
      protocol-mismatch error (proves protocol registration is
      symmetry-required).
- [ ] Attempting `use server::...` from any file in `client/src/`
      produces a compile error ("unresolved import").
- [ ] CI dependency-tree gate is in place and demonstrably fails when a
      disallowed crate is added to `shared/Cargo.toml` (negative test
      via a throwaway PR).
- [ ] Incremental `cargo check -p client` after touching one file in
      `client/src/` completes in < 5 seconds on a dev machine.
- [ ] `bevy_asset_loader` 0.18-compatible version pinned in
      `client/Cargo.toml`, OR documented fallback to manual
      `AssetServer` loading.

## GDD Requirements Addressed

| GDD Document | System | Requirement | How This ADR Satisfies It |
|-------------|--------|-------------|--------------------------|
| `design/gdd/network-protocol.md` | Networking | "Server is authoritative; client is a view. All randomness server-side." | Compile-enforced separation: `client/` cannot reach server state; `rand_chacha` not in `client/Cargo.toml` |
| `design/gdd/objective-system.md` | Hidden Information | "Per-player secret `is_fake`; opponent must not see until reveal" (per ADR-001) | `HiddenObjectives` resource lives in `server/`; `client/state/` only stores received unicast projections; compile-time guarantees no leak path |
| `design/gdd/game-config.md` | Configuration | "All tuning knobs in `GameConfig` loaded from `assets/config/game_config.ron`" | `GameConfig` struct in `shared/config.rs` (POD); server loads RON and inserts as Resource via `server/foundation/config.rs`; client may insert independently for UI math |
| `design/gdd/card-data-pool.md` | Card Data | "Card definitions are JSON; types deserialise via serde on both sides" | `CardData`, `CardId`, `Rarity`, `ClassId`, `CardType` defined in `shared/card.rs` with `serde::{Serialize, Deserialize}`; consumed identically by server (pool generation) and client (display) |
| `design/gdd/round-state-machine.md` | RSM | "Server-authoritative round phase transitions" | RSM lives in `server/core/rsm/`; client receives phase changes via `S2C*` messages from `shared/protocol.rs`; no RSM code reachable from `client/` |
| Foundational technical decision | All | Enables the full implementation phase by establishing the physical layout of the codebase | Without this ADR, no `.rs` file outside `prototypes/` can be written without arbitrary placement choices that would later require an invasive refactor |

## Related

- **ADR-001** (`docs/architecture/adr-001-objective-identity-unicast.md`)
  — Hidden objective identity uses targeted unicast, not component
  replication. The unicast pattern produces the `S2CObjectiveIdentities`
  message that lives in `shared/protocol.rs`; the server-only
  `HiddenObjectives` resource lives in `server/feature/objective/`. This
  ADR provides the structural enforcement that `HiddenObjectives` cannot
  be reached from `client/`.
- **ADR-002** (Client-Server Authority Model) — *Pending / expected*. This
  ADR is the compile-time enforcement of the runtime authority decision
  established in ADR-002. If ADR-002 is not yet Accepted at the time of
  reading, that is a sequencing flag — this ADR is consistent with the
  authority model documented in `design/gdd/network-protocol.md` and
  `.claude/docs/technical-preferences.md`, but the formal ADR-002 should
  precede the workspace scaffolding implementation story.
- **Future ADR-004** (Asset Loading) — Will live in `client/foundation/`
  (or equivalent) and `server/foundation/`, both depending on
  `shared/card.rs` types. This ADR establishes the crate structure
  ADR-004 will populate.
- **Future ADR-005** (Server RNG) — `rand_chacha`-based deterministic RNG
  lives in `server/foundation/rng.rs`. This ADR's `client/Cargo.toml`
  ban list is the structural guarantee that no client-side RNG path
  can exist.
- **Future ADR-009** (Round State Machine) — RSM implementation lives in
  `server/core/rsm/`; this ADR defines the directory it occupies.
- `.claude/docs/technical-preferences.md` — Engine, language, naming, and
  performance budget reference. The Cargo.toml templates in this ADR
  reflect the allowed-libraries table from that document.
- `docs/engine-reference/bevy/VERSION.md` — Pinned Bevy 0.18 version
  notes. Bevy feature flag names in the Cargo.toml templates must be
  reconciled against this document at implementation time.
- `design/gdd/network-protocol.md` — Protocol message catalogue. Every
  `C2S*` and `S2C*` message listed there must be defined in
  `shared/src/protocol.rs`.
