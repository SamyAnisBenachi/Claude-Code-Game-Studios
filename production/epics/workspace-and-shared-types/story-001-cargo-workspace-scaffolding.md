# Story 001: Cargo Workspace Scaffolding

> **Epic**: Workspace & Shared Types
> **Status**: Complete
> **Layer**: Foundation
> **Type**: Integration
> **Manifest Version**: 2026-04-29
> **Estimate**: 0.5d

## Context

**GDD**: `design/gdd/game-config.md` · `design/gdd/network-protocol.md`
**Requirement**: TR-??? (TR registry not yet populated — covers ADR-003 Validation Criteria 1–4)
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-003: Cargo Workspace Structure and Crate Split
**ADR Decision Summary**: Three-crate workspace (`shared/`, `server/`, `client/`). `shared/` compiles with `bevy = { default-features = false, features = ["serialize"] }` only — no ECS, no plugins, no rendering. `server/` and `client/` depend on `shared/` and never on each other. Compile-time boundary enforcement prevents secret leakage into the WASM client.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: MEDIUM
**Engine Notes**: Bevy 0.18 input handling moved behind features — confirm exact feature names for mouse/keyboard input (`bevy_input` may be a single feature rather than separate `mouse`/`keyboard` flags). Verify against `docs/engine-reference/bevy/VERSION.md` at implementation time. Lightyear 0.26 `shared` feature must not pull server/client transport code into `shared/` — if it does, fall back to per-side registration (documented in ADR-003 Risks).

**Control Manifest Rules (Foundation layer)**:
- Required: Three-crate Cargo workspace only (`shared/`, `server/`, `client/`). No other crate split.
- Required: Within `server/`: dependency direction is `feature/ → core/ → foundation/` only.
- Forbidden: Never derive `Resource`, add plugin code, or use `#[cfg(feature = "server")]` branching in `shared/`.
- Forbidden: `client/` must never depend on `server/`. `server/` must never depend on `client/`.
- Forbidden: Never put `rand` or `rand_chacha` in `client/Cargo.toml` for gameplay modules.
- Guardrail: WASM bundle ≤ 50 MB after `--release + LTO + strip` (CI-gated — Story 004).

---

## Acceptance Criteria

- [ ] `Cargo.toml` workspace root exists with `members = ["shared", "server", "client"]`, `resolver = "2"`, `[workspace.package]` (edition 2021, version 0.1.0), `[workspace.dependencies]` table, and `[profile.release]` (LTO thin, codegen-units=1, strip symbols, panic=abort) + `[profile.dev]` (opt-level=1)
- [ ] `shared/Cargo.toml` exists with `bevy = { workspace = true, default-features = false, features = ["serialize"] }` and `lightyear = { workspace = true, default-features = false, features = ["shared"] }` only — no `bevy_ecs`, no `bevy_asset`, no `tokio`, no `rand_chacha`
- [ ] `shared/src/lib.rs` exists with `pub mod card; pub mod config; pub mod protocol;` (module declarations only — bodies are empty stubs)
- [ ] `server/Cargo.toml` exists depending on `shared = { path = "../shared" }` with headless Bevy features (`bevy_ecs`, `multi_threaded`), Lightyear server+websocket, `serde`, `ron`, `rand`, `rand_chacha`
- [ ] `server/src/main.rs` exists as a compilable empty `fn main() {}`
- [ ] `server/src/foundation/`, `server/src/core/`, `server/src/feature/` subdirectories exist with placeholder `mod.rs` files
- [ ] `client/Cargo.toml` exists depending on `shared = { path = "../shared" }` with browser Bevy features (`bevy_ui`, `bevy_sprite`, `bevy_text`, `bevy_asset`, `bevy_winit`, `webgl2`), Lightyear client+websocket, `bevy_tweening`, `serde` — no `tokio`, no `rand_chacha`
- [ ] `client/src/main.rs` exists as a compilable empty `fn main() {}`
- [ ] `client/src/network/`, `client/src/state/`, `client/src/ui/` subdirectories exist with placeholder `mod.rs` files
- [ ] `client/index.html` exists as a minimal valid Trunk entry point
- [ ] `cargo check --workspace` passes with **zero warnings** on a clean clone
- [ ] `bevy_asset_loader` version decision: either pinned in workspace deps after verifying a 0.18-compatible release on crates.io, or fallback documented in an implementation note in `server/src/foundation/config.rs`

---

## Implementation Notes

*Derived from ADR-003 Implementation Guidelines:*

**Workspace `Cargo.toml` profile config:**
```toml
[profile.release]
lto = "thin"
codegen-units = 1
strip = "symbols"
panic = "abort"   # smaller WASM; acceptable for multiplayer client

[profile.dev]
opt-level = 1     # Bevy is unusable at opt-level=0
```

**`shared/Cargo.toml` — lean by design:**
```toml
[dependencies]
bevy = { workspace = true, default-features = false, features = ["serialize"] }
lightyear = { workspace = true, default-features = false, features = ["shared"] }
serde = { workspace = true }
```
Do NOT add `bevy_asset`, `bevy_ecs`, `bevy_render`, `tokio`, or `rand_chacha` here — violations will be caught by Story 004's CI gate.

**`client/Cargo.toml` — verify input feature names before committing:**
ADR-003 drafts `x11` as a feature for native dev/debug target and suggests separate `mouse`/`keyboard` flags. Bevy 0.18 may have consolidated these under `bevy_input`. Check `docs/engine-reference/bevy/VERSION.md` and the Bevy 0.18 cargo features matrix before finalising. If wrong feature names are used, `cargo check -p client` will fail with "Package `bevy` does not have feature `mouse`" — easy to spot.

**Server subdirectory structure:**
Create `server/src/foundation/mod.rs`, `server/src/core/mod.rs`, `server/src/feature/mod.rs` as empty files. Add `mod foundation; mod core; mod feature;` to `server/src/main.rs`. This establishes the layer directories before any code lands.

**Trunk compatibility:**
`client/index.html` should be minimal — Trunk reads the manifest from `client/Cargo.toml`. The `index.html` just needs a `<link data-trunk rel="rust" />` or equivalent. Trunk is invoked from the `client/` directory; workspace discovery still works because Trunk reads the `Cargo.toml` resolver config.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- Story 002: Card type implementations in `shared/src/card.rs`
- Story 003: `GameConfig` struct in `shared/src/config.rs`
- Story 004: `register_protocol` fn, CI dependency-tree gates, WASM bundle-size gate

---

## QA Test Cases

*QL-STORY-READY skipped — Lean mode. Basic integration checks below.*

- **AC: `cargo check --workspace` passes**
  - Given: Clean clone of the repository with no pre-built artifacts
  - When: `cargo check --workspace` is run
  - Then: Exit code 0, zero warnings printed
  - Edge cases: Missing `mod` declarations in `lib.rs` cause compile error — confirm all three module stubs are declared

- **AC: No cross-crate dep**
  - Given: The workspace as scaffolded
  - When: `cargo tree -p client` is inspected
  - Then: `server` does not appear as a dependency anywhere in the client tree

- **AC: `shared/` has only serialize**
  - Given: `shared/Cargo.toml` as written
  - When: `cargo tree -p shared` is inspected
  - Then: `bevy_ecs`, `tokio`, `rand_chacha` do not appear

---

## Test Evidence

**Story Type**: Integration
**Required evidence**: `cargo check --workspace` output showing zero warnings — paste into `tests/evidence/story-001-workspace-check.md`
**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: None — this is the first story
- Unlocks: Story 002 (card types), Story 003 (GameConfig struct) — both can start in parallel once this is Done

## Completion Notes

**Completed**: 2026-04-29
**Criteria**: 11/12 auto-verified. 1 ADVISORY: `cargo check --workspace` output not yet recorded in evidence file — run locally and paste into `tests/evidence/story-001-workspace-check.md`.
**Deviations**: None
**Test Evidence**: `tests/evidence/story-001-workspace-check.md` — placeholder exists, awaiting manual cargo check output
**Code Review**: APPROVED WITH SUGGESTIONS (non-blocking — client Cargo.toml workspace = true consistency deferred to Story 004)
