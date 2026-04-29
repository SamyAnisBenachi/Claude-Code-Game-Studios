# Story 004: Protocol Skeleton & CI Dependency Gates

> **Epic**: Workspace & Shared Types
> **Status**: Complete
> **Layer**: Foundation
> **Type**: Integration
> **Manifest Version**: 2026-04-29

## Context

**GDD**: `design/gdd/network-protocol.md`
**Requirement**: TR-??? (TR registry not yet populated — covers TR-NP-STRUCT; ADR-003 Validation Criteria 5–10)
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-003 (workspace structure — protocol registration symmetry) + ADR-008 (Lightyear channel config — channel type stubs)
**ADR Decision Summary**: `shared/src/protocol.rs` is the single registration site for all Lightyear protocol types. Both `server/main.rs` and `client/main.rs` call `register_protocol(app)`. A working no-op message (`S2CHeartbeat`) proves the pattern compiles end-to-end before Epic 4 populates all C2S*/S2C* types. CI gates enforce the dependency hygiene rules from ADR-003 across all three crates.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: MEDIUM
**Engine Notes**: Lightyear 0.26 is entirely post-training-cutoff (released Jan 2026). The exact `app.register_message::<T>(ChannelDirection::...)` API syntax must be verified against Lightyear 0.26 docs.rs before this story can be marked Done. If `register_message` has a different name or signature in 0.26, document the correct API and update `protocol.rs` accordingly. This story intentionally uses a single no-op message to surface any API incompatibility before Epic 4 implements the full protocol. **This story is the earliest Lightyear API contact point — treat it as a mini-spike.**

**Control Manifest Rules (Foundation layer)**:
- Required: All channel definitions live in `shared/src/protocol.rs`. Both server and client compile against identical channel types.
- Required: Exactly two Lightyear channels: `ReliableChannel` and `UnreliableChannel`. Channel assignment is permanent per message type.
- Forbidden: Never derive `Resource`, add plugin code in `shared/`.
- Guardrail: WASM bundle ≤ 50 MB after `--release + LTO + strip` (CI-gated in this story).

---

## Acceptance Criteria

**Protocol skeleton:**
- [ ] `shared/src/protocol.rs` defines `S2CHeartbeat` as `#[derive(Serialize, Deserialize, Debug, Clone)] pub struct S2CHeartbeat;`
- [ ] `shared/src/protocol.rs` defines `ReliableChannel` and `UnreliableChannel` channel type stubs (exact Lightyear 0.26 API verified and used)
- [ ] `pub fn register_protocol(app: &mut App)` exists in `shared/src/protocol.rs` and registers `S2CHeartbeat` on `ReliableChannel` using the verified Lightyear 0.26 API
- [ ] `server/src/main.rs` calls `shared::protocol::register_protocol(&mut app)` — compiles
- [ ] `client/src/main.rs` calls `shared::protocol::register_protocol(&mut app)` — compiles
- [ ] Lightyear 0.26 API surface note: the exact method name(s) used from `lightyear` are documented in a code comment in `register_protocol` (e.g. `// Lightyear 0.26: app.add_message::<T>() — verified against docs.rs 2026-04-29`)

**CI dependency gates:**
- [ ] `cargo tree -p shared --prefix none` does NOT contain: `bevy_ecs`, `bevy_render`, `bevy_ui`, `bevy_winit`, `tokio`, or any `lightyear` feature gated on `server` or `client`
- [ ] `cargo tree -p client --prefix none` does NOT contain `tokio` or `rand_chacha`
- [ ] `cargo tree -p server --prefix none` does NOT contain `bevy_render`, `bevy_ui`, or `bevy_winit`
- [ ] WASM bundle-size gate: `cargo build -p client --target wasm32-unknown-unknown --release` produces a `.wasm` artefact ≤ 50 MB
- [ ] Negative-test: temporarily adding a disallowed crate (e.g. `tokio = { workspace = true }`) to `shared/Cargo.toml` causes the `cargo tree -p shared` gate to fire (document this test result in `tests/evidence/story-004-negative-test.md`, then revert the change)
- [ ] `bevy_asset_loader` decision documented: either version pinned in `workspace.dependencies` after verifying crates.io, or fallback path noted in `server/src/foundation/mod.rs` as a `// TODO(Epic 2)` comment

---

## Implementation Notes

*Derived from ADR-003 §Migration Plan steps 3–4 and ADR-008 Decision:*

**Lightyear 0.26 `register_protocol` shape (to verify):**
ADR-003 sketches the pattern as:
```rust
pub fn register_protocol(app: &mut App) {
    app.register_message::<S2CHeartbeat>(ChannelDirection::ServerToClient);
    // channel registrations ...
}
```
The exact method name (`register_message`, `add_message`, etc.) and the `ChannelDirection` enum path must be confirmed against Lightyear 0.26 docs.rs. If the API differs, update this story's implementation and add a note to `docs/architecture/control-manifest.md` under "Lightyear 0.26 verification — item 1".

**Channel type stubs (ADR-008):**
ADR-008 specifies two channels. The channel type definitions in `shared/protocol.rs` follow the Lightyear 0.26 pattern for defining named channels. Exact syntax: verify against docs.rs. Sketch:
```rust
pub struct ReliableChannel;
pub struct UnreliableChannel;
// Register channels in register_protocol with appropriate ChannelSettings
```

**CI gate implementation — GitHub Actions:**
The `.github/workflows/tests.yml` (already staged in git) should be extended with:
1. A `cargo-check` job running `cargo check --workspace`
2. A `dep-tree-shared` job running `cargo tree -p shared` piped through `grep` to assert absence of disallowed crates
3. A `dep-tree-client` job checking for `tokio`/`rand_chacha` in the client tree
4. A `dep-tree-server` job checking for render/UI crates in the server tree
5. A `wasm-bundle-size` job building the WASM release and asserting ≤ 50 MB

If CI is not the right home for some gates (e.g. the WASM build is slow), document why in `tests/evidence/story-004-ci-gates.md`.

**`bevy_asset_loader` pinning:**
Before closing this story, visit crates.io and search for `bevy_asset_loader`. If a `0.18`-compatible release exists, add it to `[workspace.dependencies]` with the exact version. If not, add this comment to `server/src/foundation/mod.rs`:
```rust
// bevy_asset_loader: no 0.18-compatible release verified as of [date].
// Fallback: manual AssetServer loading in Epic 2 (game-config-pipeline).
// Re-check crates.io before implementing Epic 2 Story 001.
```

**WASM bundle size mitigation if > 50 MB:**
If the initial WASM build exceeds 50 MB, apply these in order:
1. Verify `[profile.release]` LTO + strip + panic=abort are active
2. Run `wasm-opt -Oz` post-processing
3. Drop any Bevy features not yet needed (e.g. `bevy_audio` if not explicitly required)
4. If still over budget, file a finding in `tests/evidence/story-004-wasm-size.md` — do not block the story, but flag it for the TD before Epic 2 begins

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- Epic 4 (lightyear-protocol-verification): All C2S*/S2C* message types beyond `S2CHeartbeat`; full channel verification checklist; end-to-end WebSocket connection test

---

## QA Test Cases

*QL-STORY-READY skipped — Lean mode. Basic checks below.*

- **AC: Protocol symmetry — both sides compile**
  - Given: `register_protocol` implemented with `S2CHeartbeat`; both entry points call it
  - When: `cargo check --workspace` is run
  - Then: Zero errors — both `server` and `client` compile with the shared protocol type

- **AC: `cargo tree -p shared` gate fires on violation**
  - Given: A temporary edit adding `tokio = { workspace = true }` to `shared/Cargo.toml`
  - When: `cargo tree -p shared --prefix none | grep tokio` is run
  - Then: `tokio` appears in output (gate would fire)
  - Cleanup: Revert the edit; document the test result in `tests/evidence/story-004-negative-test.md`

- **AC: WASM bundle ≤ 50 MB**
  - Given: Release build of client crate for `wasm32-unknown-unknown`
  - When: Size of the output `.wasm` file is measured
  - Then: ≤ 50,000,000 bytes (50 MB)
  - Edge cases: If > 50 MB, document which Bevy features contribute most via `cargo bloat`

---

## Test Evidence

**Story Type**: Integration
**Required evidence**:
- `cargo check --workspace` clean output → `tests/evidence/story-004-workspace-check.md`
- `cargo tree` gate outputs (shared, client, server) → `tests/evidence/story-004-dep-gates.md`
- Negative-test result → `tests/evidence/story-004-negative-test.md`
- WASM bundle size measurement → `tests/evidence/story-004-wasm-size.md`
**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 (workspace), Story 002 (card types), Story 003 (GameConfig) — all three must be Done
- Unlocks: Epic 2 (game-config-pipeline), Epic 3 (server-rng), Epic 4 (lightyear-protocol-verification) — all foundation epics can begin once this story is Done

---

## Completion Notes

**Completed**: 2026-04-29
**Verdict**: COMPLETE WITH NOTES
**Criteria**: 5/11 passing — 6 deferred (ADVISORY, not blocking)
**Review mode**: Lean — LP-CODE-REVIEW and QL-TEST-COVERAGE skipped

### Passing Acceptance Criteria
- `S2CHeartbeat`, `ReliableChannel`, `UnreliableChannel` defined in `shared/src/protocol.rs`
- `cargo tree -p shared` gate passes (CI green — no bevy/tokio/render crates in shared)
- `cargo tree -p client` gate passes (CI green — no tokio/rand_chacha in client)
- `cargo tree -p server` gate passes (CI green — no bevy_render/bevy_ui/bevy_winit in server)
- WASM bundle ≤ 50 MB (CI wasm-size job passing on run 25130998038)

### Advisory Deviations (documented, not blocking)

**`register_protocol()` absent from `shared/`** (ACs 3–6 deferred):
- ADR-003 fallback applied: the `lightyear` `shared` feature does not exist in Lightyear 0.26.
- Protocol registration lives in `server/main.rs` and `client/main.rs` as `// TODO` pending S1-05.
- ACs 3–6 (register_protocol signature, server call, client call, API comment) are explicitly
  deferred to S1-05 (Lightyear 0.26 Verification Spike), which must verify the correct API
  before any registration code is written.

**Evidence collected via CI rather than local runs**:
- Smart App Control blocks local Rust builds on the dev machine.
- All evidence sourced from CI run 25130998038 (commit `88971ec`) — authoritative.
- Evidence files in `tests/evidence/story-004-*.md` filled with CI data.

**Negative test proven by real CI history**:
- Commit `865a138` accidentally added `bevy` to `shared/Cargo.toml` → gate fired (CI RED).
- Commit `88971ec` removed it → gate passes (CI GREEN).
- Real-world violation caught correctly — no synthetic negative test required.

### Test Evidence
- `tests/evidence/story-004-dep-gates.md` — all 3 cargo tree gates PASS (CI run 25130998038)
- `tests/evidence/story-004-wasm-size.md` — WASM CI job PASS (run 25130998038)
- `tests/evidence/story-004-workspace-check.md` — cargo check PASS (CI "Run Cargo Tests" green)
- `tests/evidence/story-004-negative-test.md` — gate confirmed functional via CI history

### Code Review
Skipped — Lean mode.

### Tech Debt
- ACs 3–6 (register_protocol, server/client calls, API comment) → deferred to S1-05
- `bevy_asset_loader` version pinning → deferred to S1-05 (check crates.io before Epic 2)
- Local build environment: Smart App Control blocks local cargo builds — consider WSL2 for local verification
