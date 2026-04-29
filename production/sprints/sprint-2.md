# Sprint 2 — 2026-05-14 to 2026-05-27

## Sprint Goal

Implement the Core layer's pure-function foundation — RSM state + event bus, Economy API, and Card Pool weighted-draw — so that all subscriber stories can begin in parallel in Sprint 3.

## Capacity

- Total working days: 10 (2-week sprint)
- Buffer (20%): 2 days reserved for unplanned work / verification surprises
- Available: **8 effective days**

---

## Pre-Sprint Housekeeping (Complete Before Starting Any Story)

| Action | Item | Why |
|--------|------|-----|
| Run `/story-done` | `production/epics/game-config-pipeline/story-002-asset-loading-pipeline.md` (S1-07) | Implemented 2026-04-29; never formally closed |
| Run `/story-done` | `production/epics/server-rng/story-003-determinism-session-reset.md` (S1-12) | Implemented 2026-04-29; never formally closed |
| Run `/story-done` | `production/epics/card-data-pool/story-001-pool-state-core-api.md` (pool-001) | Implemented 2026-04-29; outside S1 scope — needs formal close |
| Fix `sprint-status.yaml` | S1-02 + S1-03 | Session state: ✅ Done; YAML still shows `ready-for-dev` |
| Resolve duplicates | `production/epics/card-data-pool/` | Two parallel story sets exist. Set B (`*-pool-state-core-api`, `*-weighted-draw-functions`, `*-refresh-shop-slot-variants`, `*-shop-refresh-subscriber-session-ready`, `*-manual-refresh-cost-escalation`, `*-network-dispatch-wiring`) is canonical — Story 1 was implemented against it. Delete Set A files. |

---

## Tasks

### Must Have — Critical Path (3.5 days)

| ID | Story | Epic | Est. | Dependencies | Acceptance Criteria |
|----|-------|------|------|-------------|---------------------|
| S2-01 | [RSM Story 1: State + Events Scaffold](../epics/round-state-machine/story-001-state-and-events-scaffold.md) | round-state-machine | 1.5d | S1-05 Done | `RoundState` enum defined; all 6 ADR-010 messages (`DraftStarted`, `ShopRefreshNeeded`, `PlacementPhaseEntered`, `ResolutionPhaseEntered`, `GameOverEmitted`, `BroadcastPhaseChanged`) compile with `#[derive(Message)]`; `RsmPlugin` registered; `cargo check -p server` green |
| S2-02 | [Economy Story 1: State + Pure API Scaffold](../epics/economy-system/story-001-state-and-pure-api-scaffold.md) | economy-system | 1.0d | S1-09 Done | `PlayerEconomy` struct with all GDD fields; `interest()`, `income_for_round()`, `award_kill()`, `award_objective()` pure functions; no Bevy subscribers yet; unit tests on all formulas pass |
| S2-03 | [Card Pool Story 2: Weighted Draw Functions](../epics/card-data-pool/story-002-weighted-draw-functions.md) | card-data-pool | 1.0d | pool-001 Done | `weighted_draw()` + `initial_draw_offering()` pure functions; rarity weight × class match × `copies_remaining` formula; unit tests cover empty pool (returns `None`), class-match filter, weight distribution |

### Should Have (2.0 days)

| ID | Story | Epic | Est. | Dependencies | Acceptance Criteria |
|----|-------|------|------|-------------|---------------------|
| S2-04 | [Card Pool Story 3: Refresh Shop Slot Variants](../epics/card-data-pool/story-003-refresh-shop-slot-variants.md) | card-data-pool | 1.0d | S2-03 | `refresh_shop()` with `SlotVariant` enum; DRAFT_INITIAL 9-card vs SHOP_REFRESH 3-card logic; edge-case tests: duplicate card, full-pool, empty-pool all handled |
| S2-05 | [S1 Carryover: Startup Validation Gate](../epics/game-config-pipeline/story-003-startup-validation-gate.md) | game-config-pipeline | 1.0d | S1-07 Done | 10 dangerous-value checks with passing+failing unit tests; fatal exit on CRITICAL values; soft error on WARNING; `cargo check -p server` green |

### Nice to Have — defer to Sprint 3 if capacity consumed (4.5 days)

| ID | Story | Epic | Est. | Dependencies | Acceptance Criteria |
|----|-------|------|------|-------------|---------------------|
| S2-06 | [S1 Carryover: All Protocol Message Types](../epics/lightyear-protocol-verification/story-002-all-protocol-message-types.md) | lightyear-protocol-verification | 1.0d | S1-05 Done | All C2S\*/S2C\* message types defined in `shared/src/protocol.rs`; `cargo check -p shared` green; registry entries added |
| S2-07 | [RSM Story 2: advance_phase + F2 Ordering](../epics/round-state-machine/story-002-advance-phase-and-f2-ordering.md) | round-state-machine | 1.0d | S2-01 | `advance_phase()` handles all phase transitions; F2 drain order (Economy before Card Pool); `MessageWriter<ShopRefreshNeeded>` emitted per player |
| S2-08 | [Economy Story 2: Initialisation + Draft Subscriber](../epics/economy-system/story-002-initialisation-draft-subscriber.md) | economy-system | 0.5d | S2-01, S2-02 | `on_draft_started` subscriber system; economy initialised at session start; `DraftStarted` triggers gold award; unit tests pass |
| S2-09 | [S1 Carryover: Server & Client Network Plugins](../epics/lightyear-protocol-verification/story-003-server-client-network-plugins.md) | lightyear-protocol-verification | 1.0d | S2-06 | Unicast compile-proof passes; server plugin handles WebSocket; WASM client plugin compiles; `cargo check` green on both crates |
| S2-10 | [S1 Carryover: E2E WebSocket Round-Trip](../epics/lightyear-protocol-verification/story-004-e2e-websocket-roundtrip.md) | lightyear-protocol-verification | 1.0d | S2-09 | Heartbeat round-trip integration test passes; WASM bundle size documented |

---

## Carryover from Sprint 1

| Task | Reason | New Estimate |
|------|--------|-------------|
| S1-08 Startup Validation Gate → S2-05 | Unblocked once S1-07 `/story-done` runs; not yet implemented | 1.0d (Should Have) |
| S1-13 Protocol Message Types → S2-06 | Pure additive; unblocked since S1-05 Done | 1.0d (Nice to Have) |
| S1-14 Server & Client Network Plugins → S2-09 | Depends S1-13 | 1.0d (Nice to Have) |
| S1-15 E2E WebSocket Round-Trip → S2-10 | Depends S1-14; lower priority than Core layer | 1.0d (Nice to Have) |
| S1-11 Debug Hot-Reload | Depends S1-08; deferred to Sprint 3 | 0.5d |

---

## Risks

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| RSM Story 1 `#[derive(Message)]` API differs from control-manifest spec | MEDIUM | HIGH | `liv-bevy-018` mandatory on every `.rs` file; S1-05 spike annotated 7 DIFFERS items for Lightyear messages — apply same vigilance to Bevy Messages |
| card-data-pool duplicate story files cause confusion during implementation | HIGH | LOW | Pre-sprint cleanup: delete Set A files before any story is picked up |
| GSS Story 4 (SessionReady Observer — ADR-012 high-risk) surfaces Bevy 0.18 Observer breaking change | HIGH | HIGH | Explicitly deferred to Sprint 3; S1-05 CI tests for ADR-012 open conditions serve as early warning |
| S1-08 dangerous-value tests require Bevy `App` setup rather than plain Rust unit tests | MEDIUM | MEDIUM | Confirm correct test pattern in `liv-bevy-018` before starting S2-05 |
| Sprint 1 YAML discrepancy (S1-02, S1-03 show `ready-for-dev`) causes `/sprint-status` to misreport velocity | HIGH | LOW | Fix as pre-sprint housekeeping |

---

## Dependencies on External Factors

- CI must confirm green on `/story-done` closures for S1-07 and S1-12 before their dependents start
- Bevy 0.18 migration guide + `docs.rs/lightyear/0.26` must remain accessible for ADR clarifications

---

## Design Parallel Track (outside sprint story capacity)

These design tasks run concurrently with sprint implementation sessions:

- `/design-review` on 3 GDDs pending review: Auction System, Combat Resolution, Card Acquisition
- Complete Board Rendering GDD (`design/gdd/board-rendering.md` — skeleton exists, Section A in progress)
- Begin Hand UI GDD (M2 system, not yet started)

---

## Definition of Done for Sprint 2

- [ ] Pre-sprint housekeeping complete: `/story-done` on S1-07, S1-12, pool-001; S1-02/S1-03 YAML corrected to `done`; card-data-pool Set A duplicates deleted
- [ ] All Must Have stories (S2-01 through S2-03) completed and evidence documented
- [ ] RSM event bus verified: all 6 message types compile and registered in `RsmPlugin`
- [ ] All Logic/Integration stories have passing unit tests in `tests/unit/` or embedded `#[cfg(test)]`
- [ ] `cargo check --workspace` green; CI passing
- [ ] No S1 or S2 bugs in delivered features
- [ ] QA plan exists: `production/qa/qa-plan-sprint-2.md`

---

> ⚠️ **No QA Plan**: Run `/qa-plan sprint` before the first story is implemented.
> The Production → Polish gate requires a QA sign-off report, which requires a QA plan.
