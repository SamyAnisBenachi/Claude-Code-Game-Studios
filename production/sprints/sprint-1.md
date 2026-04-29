# Sprint 1 — 2026-04-30 to 2026-05-13

## Sprint Goal

Establish the compilable three-crate workspace, all shared Foundation types, and a verified Lightyear 0.26 API surface — unblocking every Core layer epic and making the Pre-Production gate-check pass.

## Capacity

- Total working days: 10 (2-week sprint)
- Buffer (20%): 2 days reserved for unplanned work / verification surprises
- Available: **8 effective days**

---

## Tasks

### Must Have — Critical Path (3.5 days)

| ID | Story | Epic | Est. | Dependencies | Acceptance Criteria |
|----|-------|------|------|-------------|---------------------|
| S1-01 | [Cargo Workspace Scaffolding](../epics/workspace-and-shared-types/story-001-cargo-workspace-scaffolding.md) | workspace-and-shared-types | 0.5d | None | `cargo check --workspace` green, zero warnings on clean clone |
| S1-02 | [Shared Card Types](../epics/workspace-and-shared-types/story-002-shared-card-types.md) | workspace-and-shared-types | 0.5d | S1-01 | All `CardData` types in `shared/src/card.rs`; `cargo check -p shared` green |
| S1-03 | [GameConfig POD Struct](../epics/workspace-and-shared-types/story-003-game-config-pod-struct.md) | workspace-and-shared-types | 0.5d | S1-01 | All GDD Section G fields present; serde only; `Default` impl encodes design-intent values |
| S1-04 | [Protocol Skeleton + CI Gates](../epics/workspace-and-shared-types/story-004-protocol-skeleton-ci-gates.md) | workspace-and-shared-types | 1.0d | S1-01, S1-02, S1-03 | All 4 `cargo tree` gates pass; WASM ≤ 50 MB; negative gate test fires; `register_protocol` wired in both entry points |
| **S1-05** ⭐ | **[Lightyear 0.26 Verification Spike](../epics/lightyear-protocol-verification/story-001-lightyear-026-verification-spike.md)** | lightyear-protocol-verification | **1.0d** | S1-04 | All 20 checklist items annotated CONFIRMED/DIFFERS; ADR-012 test written; `control-manifest.md` updated; zero `⬜` items remain |

### Should Have — Foundation Layer Complete (4.5 days)

| ID | Story | Epic | Est. | Dependencies | Acceptance Criteria |
|----|-------|------|------|-------------|---------------------|
| S1-06 | [Asset Data Files](../epics/game-config-pipeline/story-001-asset-data-files.md) | game-config-pipeline | 0.5d | S1-03 | `game_config.ron` + `cards.json` fixture parse without error; all GDD Section G defaults present |
| S1-07 | [Asset Loading Pipeline](../epics/game-config-pipeline/story-002-asset-loading-pipeline.md) | game-config-pipeline | 1.0d | S1-06, S1-04 | Server reaches `AppState::Lobby` with `Res<GameConfig>` and `Res<CardCatalog>` present |
| S1-08 | [Startup Validation Gate](../epics/game-config-pipeline/story-003-startup-validation-gate.md) | game-config-pipeline | 1.0d | S1-07 | All 10 dangerous-value checks have passing+failing unit tests; fatal exit verified; soft error verified |
| S1-09 | [ServerRng Type Definitions](../epics/server-rng/story-001-type-definitions-audit-infrastructure.md) | server-rng | 0.5d | S1-01 | `ServerRng` compiles; session_init sentinel at index 0; RNG1/RNG5/RNG11 tests pass |
| S1-10 | [Intent-Named API & Invariants](../epics/server-rng/story-002-intent-named-api-invariants.md) | server-rng | 1.0d | S1-09 | All 7 intent-named methods present; RNG2/RNG6/RNG7/RNG12 tests pass |

### Nice to Have (3.5 days — defer to Sprint 2 if buffer consumed)

| ID | Story | Epic | Est. | Dependencies | Acceptance Criteria |
|----|-------|------|------|-------------|---------------------|
| S1-11 | [Debug Hot-Reload + Release Verify](../epics/game-config-pipeline/story-004-debug-hot-reload.md) | game-config-pipeline | 0.5d | S1-08 | Hot-reload accept/reject tested; release binary symbol check documented |
| S1-12 | [Determinism Proof + Session Reset](../epics/server-rng/story-003-determinism-session-reset.md) | server-rng | 0.5d | S1-10 | Fixed-seed determinism test passes; RNG13 + RNG15 advisory pass |
| S1-13 | [All Protocol Message Types](../epics/lightyear-protocol-verification/story-002-all-protocol-message-types.md) | lightyear-protocol-verification | 1.0d | S1-05 | All C2S*/S2C* message types defined; `cargo check -p shared` green |
| S1-14 | [Server & Client Network Plugins](../epics/lightyear-protocol-verification/story-003-server-client-network-plugins.md) | lightyear-protocol-verification | 1.0d | S1-13 | Unicast compile-proof compiles; both crates check clean |
| S1-15 | [End-to-End WebSocket Round-Trip](../epics/lightyear-protocol-verification/story-004-e2e-websocket-roundtrip.md) | lightyear-protocol-verification | 1.0d | S1-14 | Heartbeat round-trip integration test passes; WASM bundle size documented |

---

## Carryover from Previous Sprint

None — this is Sprint 1.

---

## Risks

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| Lightyear 0.26 API differs significantly from ADR assumptions | HIGH | HIGH | S1-05 is the spike — all DIFFERS items get resolution paths before any dependent code is written |
| `bevy_asset_loader` has no 0.18-compatible release on crates.io | MEDIUM | MEDIUM | S1-04 checks this; fallback to manual `AssetServer` loading documented in story |
| WASM bundle exceeds 50 MB after Bevy + Lightyear + tweening | MEDIUM | HIGH | S1-04 measures early; feature pruning + `wasm-opt` are the mitigation path |
| Bevy 0.18 input feature flag names differ from ADR-003 draft | MEDIUM | LOW | S1-01 reconciles against `docs/engine-reference/bevy/VERSION.md` at first `cargo check` |
| S1-05 verification spike takes longer than 1 day (20 items + ADR-012 test) | MEDIUM | MEDIUM | 2-day buffer absorbs overflow; Nice to Have stories defer to Sprint 2 first |

---

## Dependencies on External Factors

- `docs.rs/lightyear/0.26` must be accessible to complete S1-05
- `crates.io` check for `bevy_asset_loader` 0.18-compatible release (S1-04, S1-07)

---

## Definition of Done for Sprint 1

- [ ] All Must Have stories (S1-01 through S1-05) completed and evidence documented
- [ ] S1-05 checklist fully annotated — zero `⬜` items remain in `docs/architecture/control-manifest.md §Lightyear 0.26 Verification Checklist`
- [ ] All Logic/Integration stories have passing unit tests in `tests/unit/foundation/` or `server/tests/`
- [ ] `cargo check --workspace` green; all 4 `cargo tree` CI gates passing
- [ ] No S1 or S2 bugs in delivered code
- [ ] QA plan created before last story is implemented (`production/qa/qa-plan-sprint-1.md`)

---

## Gate-Check Impact

Completing Must Have (S1-01–S1-05) satisfies both remaining gate blockers:
1. ~~`design/ux/interaction-patterns.md` missing~~ ✅ Done prior session
2. ~~Epics + Sprint plan missing~~ ✅ Done this session

Run `/gate-check` after Sprint 1 Must Have is complete — verdict should be **PASS**.

---

> ⚠️ **No QA Plan**: This sprint was started without a QA plan. Run `/qa-plan sprint`
> before the last story is implemented. The Production → Polish gate requires a QA
> sign-off report, which requires a QA plan.
