# Sprint 1 QA Plan

> **Sprint**: Sprint 1 — Foundation
> **Date**: 2026-04-29
> **Gate owner**: QA Lead

---

## Scope

Stories covered in this plan:

| ID | Story | Epic | Type | Gate |
|---|---|---|---|---|
| S1-01 | Cargo Workspace Scaffolding | workspace-and-shared-types | Integration | BLOCKING |
| S1-02 | Shared Card Types | workspace-and-shared-types | Config/Data | ADVISORY |
| S1-03 | GameConfig POD Struct | workspace-and-shared-types | Config/Data | ADVISORY |
| S1-04 | Protocol Skeleton + CI Gates | workspace-and-shared-types | Integration | BLOCKING |
| S1-09 | ServerRng Type Definitions + Audit Infrastructure | server-rng | Logic | BLOCKING |

**Not in scope**: S1-05 (Lightyear spike — backlog), S1-06–S1-08, S1-10–S1-15.

---

## Per-Story Test Requirements

### S1-01 — Cargo Workspace Scaffolding (Integration / BLOCKING)

**Test type**: Smoke check — `cargo check --workspace`
**Required evidence**: `tests/evidence/story-001-workspace-check.md`
**Gate**: BLOCKING — story cannot be marked Done without this file populated.

Manual checks:
- [ ] `cargo check --workspace` exits 0 with zero warnings
- [ ] Three crates compile independently: `cargo check -p shared`, `-p server`, `-p client`
- [ ] `client/index.html` exists (Trunk entry point)

---

### S1-02 — Shared Card Types (Config/Data / ADVISORY)

**Test type**: Smoke check — `cargo check -p shared`
**Required evidence**: `tests/evidence/story-002-shared-types-check.md`
**Gate**: ADVISORY — story can be marked Done; evidence is strongly recommended.

Manual checks:
- [ ] `cargo check -p shared` exits 0 with zero warnings
- [ ] `CardId`, `Rarity`, `ClassId`, `CardType`, `UnitType`, `SimpleKeyword`, `Keyword`, `CardData`, `CardCatalog` all present in `shared/src/card.rs`
- [ ] `Copy` derive on all enum types (grep: `derive.*Copy`)
- [ ] `EPIC_POOL_COPIES` and `LEGENDARY_POOL_COPIES` are `const u32 = 1`

---

### S1-03 — GameConfig POD Struct (Config/Data / ADVISORY)

**Test type**: Smoke check — `cargo check -p shared`
**Required evidence**: `tests/evidence/story-003-game-config-check.md`
**Gate**: ADVISORY.

Manual checks:
- [ ] `cargo check -p shared` exits 0 with zero warnings
- [ ] `GameConfig` struct has all GDD Section G fields with `_seconds` suffix for RSM timers
- [ ] No `#[derive(Resource)]` on `GameConfig` (grep: `derive(Resource)`)
- [ ] `TODO(Epic 2): Asset+TypePath decision` comment present in `shared/src/config.rs`
- [ ] `Default` impl encodes non-zero design values (spot check: `starting_gold = 5`)

---

### S1-04 — Protocol Skeleton + CI Gates (Integration / BLOCKING)

**Test type**: Integration — all evidence files BLOCKING.
**Required evidence**:

| File | Command | Assert |
|---|---|---|
| `tests/evidence/story-004-workspace-check.md` | `cargo check --workspace` | Zero errors/warnings |
| `tests/evidence/story-004-dep-gates.md` | `cargo tree -p shared/client/server --prefix none` | No disallowed crates |
| `tests/evidence/story-004-negative-test.md` | Add tokio to shared, grep, revert | tokio appears → gate fires |
| `tests/evidence/story-004-wasm-size.md` | `cargo build -p client --target wasm32-unknown-unknown --release` | Raw artifact ≤ 100 MB |

Manual checks:
- [ ] `S2CHeartbeat` struct present in `shared/src/protocol.rs`
- [ ] `register_protocol(app: &mut App)` present and calls `add_channel` + `register_message`
- [ ] Both `server/src/main.rs` and `client/src/main.rs` call `shared::protocol::register_protocol`
- [ ] CI jobs `dep-gate-shared`, `dep-gate-client`, `dep-gate-server`, `wasm-size` present in `tests.yml`
- [ ] `bevy_asset_loader` fallback comment present in `server/src/foundation/mod.rs`

**Compile verification (critical for Option A)**:
- [ ] `cargo check -p shared` passes — confirms `register_protocol` compiles with `lightyear features=["shared"]`
- [ ] If it fails: activate ADR-003 fallback — move `register_protocol` to `server/main.rs` and `client/main.rs`

---

### S1-09 — ServerRng Type Definitions + Audit Infrastructure (Logic / BLOCKING)

**Test type**: Automated unit tests — BLOCKING.
**Required evidence**: `tests/unit/foundation/server_rng_types_test.rs` (must pass)

Test cases required:

| Test | AC | Assertion |
|---|---|---|
| `test_new_seed_index_is_one` | RNG1 | `current_seed_index() == 1` after construction |
| `test_zero_calls_has_one_audit_entry` | RNG5 | `audit_log().len() == 1` before any calls |
| `test_n_calls_produces_n_plus_one_audit_entries` | RNG5 | 3 calls → 4 entries |
| `test_sentinel_is_session_init_with_no_result` | RNG11 | `audit_log()[0]` is `SessionInit` with `result = None` |
| `test_no_raw_seed_in_audit_log` | RNG11 | Seed value never appears in any `result` string |
| `test_audit_log_seed_indices_are_sequential` | RNG5 | Indices 0, 1, 2, … in order |

Run with: `cargo test -p server foundation::rng --verbose`

---

## Smoke Test Checklist (run in order)

```bash
# 1. Full workspace compiles
cargo check --workspace

# 2. All server and shared tests pass
cargo test -p server --verbose
cargo test -p shared --verbose

# 3. Dep purity gates
cargo tree -p shared --prefix none | grep -E '^(bevy_ecs|bevy_render|bevy_ui|bevy_winit|tokio) ' && echo FAIL || echo PASS
cargo tree -p client --prefix none | grep -E '^(tokio|rand_chacha) ' && echo FAIL || echo PASS
cargo tree -p server --prefix none | grep -E '^(bevy_render|bevy_ui|bevy_winit) ' && echo FAIL || echo PASS

# 4. RSM single-writer invariant (no ResMut<RoundState> outside transitions.rs)
grep -r "ResMut<RoundState>" server/src/ | grep -v "transitions.rs" && echo FAIL || echo PASS
```

---

## Playtest Sign-off

**Not required for Sprint 1.** All stories are Foundation layer (pure Rust types, CI gates, protocol skeleton). No gameplay, UI, or visual content to playtest. Playtest sign-off gates begin in Sprint 2 with Core layer stories.

---

## Sprint 1 QA Sign-off Gate

Sprint 1 is QA-complete when:
- [x] S1-01 evidence populated (story-001-workspace-check.md)
- [ ] S1-04 evidence populated (all 4 files)
- [ ] S1-04 `cargo check --workspace` confirms Option A compiles OR fallback applied
- [ ] S1-09 `cargo test -p server` passes with 6/6 RNG tests green
- [ ] S1-02, S1-03 evidence populated (advisory — best effort)
