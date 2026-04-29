# Session State

## Current Stage: Pre-Production ✅

## Sprint 1 Progress

| ID | Story | Priority | Status |
|---|---|---|---|
| **S1-01** | Cargo Workspace Scaffolding | Must Have | ✅ done (2026-04-29) |
| **S1-02** | Shared Card Types | Must Have | ✅ done (2026-04-29) |
| **S1-03** | GameConfig POD Struct | Must Have | ✅ done (2026-04-29) |
| **S1-04** | Protocol Skeleton + CI Gates | Must Have | ⚠️ impl-complete — needs local `cargo check` |
| S1-05 ⭐ | Lightyear 0.26 Verification Spike | Must Have | backlog (needs S1-04 Done) |
| **S1-09** | ServerRng Type Definitions | Should Have | ⚠️ impl-complete — needs `cargo test -p server` |

## Pending User Actions
1. `cargo check --workspace` → paste into `tests/evidence/story-004-workspace-check.md`
2. If S1-04 Option A fails → move `register_protocol` out of `shared/` (ADR-003 fallback)
3. `cargo test -p server --verbose` → confirms 6 RNG tests pass → close S1-09
4. Run `/story-done` on S1-04 and S1-09 once evidence collected
5. Note: `cards.json` uses `"id": [1]` format (serde newtype serialization) — may need `#[serde(transparent)]` on CardId in Story 002 (loader)

## Wave 1 — DONE
- ✅ server-rng/story-001: ServerRng + AuditEntry + RngEvent + 6 tests
- ✅ create-epics core: RSM, GSS, Economy, Pool epics + index
- ✅ qa-plan: production/qa/sprint-1-qa-plan.md

## Wave 2 — DONE (2026-04-29)
- ✅ game-config-pipeline/story-001: assets/config/game_config.ron + assets/data/cards.json
- ✅ create-stories RSM: 6 stories (001–006)
- ✅ create-stories GSS: 7 stories (001–007, story-004 Blocked on ADR-012)
- ✅ create-stories Economy: 6 stories (001–006)
- ✅ create-stories Card-Data-Pool: 6 stories (001–006)

## Stories Backlog — Ready for Development
Foundation:
- production/epics/game-config-pipeline/story-001 (asset data) — impl done, needs /story-done
- production/epics/server-rng/story-002-intent-named-api-invariants.md (needs S1-09 Done)
- production/epics/game-config-pipeline/story-002 and story-003 (needs S1-03 Done ✅)

Core (sequenced — must do RSM before GSS before Economy/Pool):
- RSM story-001 → RSM story-002 → ... → RSM story-006
- GSS story-001 (lobby scaffold) — Ready
- Economy story-001 (pure API scaffold) — Ready  
- Pool story-001 (state & API) — Ready

## Next After S1-04/S1-09 Close
Wave 3 → Wave 4 (S1-05 ⭐ Lightyear spike — GATE STRICT SÉQUENTIEL)
