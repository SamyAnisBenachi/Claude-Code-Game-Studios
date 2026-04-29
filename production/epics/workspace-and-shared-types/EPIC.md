# Epic: Workspace & Shared Types

> **Layer**: Foundation
> **GDD**: design/gdd/game-config.md · design/gdd/card-data-pool.md · design/gdd/network-protocol.md
> **Architecture Module**: `shared/` crate (full)
> **Status**: Ready
> **Stories**: 4 stories created — see table below

## Overview

Establishes the three-crate Cargo workspace (`shared/`, `server/`, `client/`) and populates `shared/` with all cross-cutting data types: the `GameConfig` POD struct, the full `CardData`/`CardId`/`Rarity`/`ClassId`/`CardType`/`Keyword` schema, and the `register_protocol` entry point with one working no-op message (`S2CHeartbeat`). Both `server/main.rs` and `client/main.rs` call `register_protocol` at startup to prove end-to-end protocol symmetry at the type level. CI gates enforce that `shared/` never pulls `bevy_ecs`, `tokio`, or `rand_chacha`; that the client WASM bundle stays ≤ 50 MB; and that the server binary carries no rendering or UI crates. No gameplay code lives in this epic — it is the compile-time foundation that makes every subsequent epic possible.

## Governing ADRs

| ADR | Decision Summary | Engine Risk |
|-----|-----------------|-------------|
| ADR-003: Cargo Workspace Structure | Three-crate workspace; `shared/` with `bevy serialize` only; no cross-target deps; compile-enforced authority boundary | MEDIUM |
| ADR-006: Card Data Schema and Pool Architecture | `CardData`, `CardId`, and rarity types in `shared/src/card.rs`; `CardCatalog` as type alias; Epic/Legendary copy counts as consts | LOW |

## GDD Requirements

> Note: `docs/architecture/tr-registry.yaml` has not yet been populated. TR-IDs below are informal references from the ADR "GDD Requirements Addressed" sections. Run `/architecture-review` to register stable IDs before stories are written.

| Informal TR-ID | Requirement | ADR Coverage |
|----------------|-------------|--------------|
| TR-GC-01 | All tuning knobs loaded from external config (`game_config.ron`) | ADR-003 ✅ (GameConfig POD in shared/) |
| TR-CDP-01 | Card data types defined in shared crate; consumed identically by server and client | ADR-006 ✅ |
| TR-CDP-03 | Card definition schema with all required fields (id, name, rarity, class, type, stats, keywords, art_id) | ADR-006 ✅ |
| TR-NP-STRUCT | Protocol message types and channel definitions live in one shared crate consumed by both sides | ADR-003 ✅ |

## Scope

### Deliverables

**Workspace root**
- `Cargo.toml` — workspace members, `[workspace.dependencies]` version table, `[profile.release]` (LTO thin, codegen-units=1, strip symbols, panic=abort), `[profile.dev]` (opt-level=1)

**`shared/` crate**
- `shared/Cargo.toml` — `bevy = { default-features = false, features = ["serialize"] }` only; no `bevy_asset`, no `bevy_ecs`, no `tokio`, no `rand_chacha`
- `shared/src/lib.rs` — `pub mod card; pub mod config; pub mod protocol;`
- `shared/src/config.rs` — `GameConfig` as pure serde POD: `#[derive(Serialize, Deserialize, Debug, Clone, Default)]`. **No `Resource`. No `Asset`. No `TypePath`.** These Bevy-specific derives belong to the server-side loading code (Epic 2).
- `shared/src/card.rs` — `CardId(u32)`, `Rarity`, `ClassId`, `CardType`, `UnitType`, `Keyword`, `SimpleKeyword`, `CardData`, `CardCatalog` type alias, `EPIC_POOL_COPIES: u32 = 1`, `LEGENDARY_POOL_COPIES: u32 = 1`
- `shared/src/protocol.rs` — `S2CHeartbeat` no-op message struct; `pub fn register_protocol(app: &mut App)` that registers it; stub module structure for C2S*/S2C* types to be populated by Epic 4

**`server/` crate**
- `server/Cargo.toml` — depends on `shared`; headless Bevy features only (bevy_ecs, multi_threaded); Lightyear server + websocket; serde, ron, rand, rand_chacha
- `server/src/main.rs` — empty `fn main()` that calls `shared::protocol::register_protocol(&mut app)`

**`client/` crate**
- `client/Cargo.toml` — depends on `shared`; browser Bevy features (bevy_ui, bevy_sprite, bevy_text, bevy_asset, webgl2); Lightyear client + websocket; bevy_tweening; serde. No `tokio`. No `rand_chacha`.
- `client/src/main.rs` — empty `fn main()` that calls `shared::protocol::register_protocol(&mut app)`
- `client/index.html` — Trunk entry point

**CI gates (all required)**
- `cargo check --workspace` passes with zero warnings
- `cargo tree -p shared` → fails if `bevy_ecs`, `bevy_render`, `bevy_ui`, `bevy_winit`, `tokio`, `rand_chacha`, or Lightyear server/client features found
- `cargo tree -p client` → fails if `tokio` or `rand_chacha` found
- `cargo tree -p server` → fails if `bevy_render`, `bevy_ui`, or `bevy_winit` found
- WASM bundle-size gate: `cargo build -p client --target wasm32-unknown-unknown --release` produces `.wasm` ≤ 50 MB
- Negative-test: confirm dependency-tree gate demonstrably fails when a disallowed crate is manually added to `shared/Cargo.toml` (one throwaway commit to verify gate fires)

**`bevy_asset_loader` pinning** — verify the 0.18-compatible release on crates.io at implementation time. If unavailable, document the manual `AssetServer` fallback in an implementation note; do not block this epic on it.

### Implementation Note: GameConfig and Asset Derive

`GameConfig` in `shared/src/config.rs` intentionally omits `#[derive(Asset, TypePath)]` to preserve the lean-shared-crate constraint (ADR-003 Rule 1). ADR-004 shows these derives on the shared struct — this is a tension to resolve in Epic 2 at implementation time. Two valid paths: (a) add `bevy_asset` feature to `shared/Cargo.toml` as a deliberate, narrowly-scoped exception with ADR-004 as justification; (b) create a server-side wrapper struct in `server/foundation/config.rs` that adds `Asset + TypePath` without touching `shared/`. The CI gate for `shared/` will reveal which path is required.

## Definition of Done

- All deliverables above implemented and passing
- `cargo check --workspace` green on a clean clone
- All four `cargo tree` gates pass (three positive, one negative)
- WASM bundle ≤ 50 MB in release build
- Both `server/main.rs` and `client/main.rs` compile and call `register_protocol` — removing the call from one side produces a protocol-mismatch indication
- `bevy_asset_loader` version decision documented (pinned or fallback noted)

## Stories

| # | Story | Type | Status | ADR |
|---|-------|------|--------|-----|
| 001 | [Cargo Workspace Scaffolding](story-001-cargo-workspace-scaffolding.md) | Integration | Ready | ADR-003 |
| 002 | [Shared Card Types](story-002-shared-card-types.md) | Config/Data | Ready | ADR-006 |
| 003 | [GameConfig POD Struct](story-003-game-config-pod-struct.md) | Config/Data | Ready | ADR-003 |
| 004 | [Protocol Skeleton & CI Dependency Gates](story-004-protocol-skeleton-ci-gates.md) | Integration | Ready | ADR-003, ADR-008 |

> Story sequence: 001 first → 002 and 003 in parallel → 004 last (depends on all three).

## Next Step

Run `/story-readiness production/epics/workspace-and-shared-types/story-001-cargo-workspace-scaffolding.md` to validate the first story before implementation, then `/dev-story` to begin.
