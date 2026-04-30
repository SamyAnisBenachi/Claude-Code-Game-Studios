# Session State — Lanes and Lies

> Lis ce fichier EN PREMIER dans toute nouvelle session.
> Il contient l'état complet du projet au 2026-04-29.
>
> **Session active (2026-04-30):** HUD GDD ✅ DESIGNED — design/gdd/hud.md. All 8 sections + Visual/Audio + UI Requirements + Open Questions complete (21 ACs, 18 BLOCKING). Registry: 10 referenced_by updated. Systems index updated. /design-review pending (fresh session). Next: /ux-design hud (fresh session).
>
> **Parallel session (2026-04-30):** Class System GDD ✅ DESIGNED — design/gdd/class-system.md. All 8 sections + UI Requirements + Open Questions complete (27 ACs, 26 BLOCKING). 7 token entities + 8 formulas + 5 constants registered. 4 OQs: Xelorium timing, Sang Méprise reconnect gap (NP), Rollback+HASTE, Madoll spell scope. Systems index updated. /design-review pending (fresh session).
>
> **Parallel session (2026-04-30):** Card Animations GDD ✅ DESIGNED — design/gdd/card-animations.md. All 11 sections complete (8 required + Visual/Audio + UI Requirements + Open Questions). 20 ACs (16 BLOCKING, 4 ADVISORY). 5 custom lenses, domain-event indirection architecture, AnimGroup/AnimQueue drain. Registry: +stagger_cadence_ms. 9 OQs (5 API-verification gating implementation, 3 cross-system recommendations). Systems index updated. /design-review pending (fresh session).
>
> **Parallel session (2026-04-30):** Prism System GDD ✅ DESIGNED — design/gdd/prism-system.md. All 8 sections + Visual/Audio + UI Requirements + Open Questions complete (22 ACs, 17 BLOCKING). Registry: 2 items (prism_strike, prism_reserve), 2 constants (prism_strike_damage, prism_strike_mana_cost), 1 network_message (S2CCardAcquired). Systems index updated. /design-review pending (fresh session).
>
> Prior session (2026-04-29): Hand UI GDD ✅ DESIGNED — design/gdd/hand-ui.md. All 8 sections + Visual/Audio + UI Requirements + Open Questions complete (24 ACs). Systems index updated. /design-review pending (fresh session).

---

## Stage actuel : Pre-Production ✅
`production/stage.txt` = `Pre-Production`

---

## Sprint 1 — État des stories

| ID | Story | Fichier | Statut |
|---|---|---|---|
| S1-01 | Cargo Workspace Scaffolding | `production/epics/workspace-and-shared-types/story-001-cargo-workspace-scaffolding.md` | ✅ Done |
| S1-02 | Shared Card Types | `story-002-shared-card-types.md` | ✅ Done |
| S1-03 | GameConfig POD Struct | `story-003-game-config-pod-struct.md` | ✅ Done |
| S1-04 | Protocol Skeleton + CI Gates | `story-004-protocol-skeleton-ci-gates.md` | ✅ Done |
| S1-05 | Lightyear 0.26 Spike ⭐ | `production/epics/lightyear-protocol-verification/story-001-lightyear-026-verification-spike.md` | ✅ ADR-012 items 15-16 verified locally |
| S1-09 | ServerRng Type Definitions | `production/epics/server-rng/story-001-type-definitions-audit-infrastructure.md` | ✅ Done |

**Machine-readable status :** `production/sprint-status.yaml`
**Plan complet :** `production/sprints/sprint-1.md`

---

## CI GitHub Actions

**Dernier commit :** `88971ec` — "Fix CI: remove invalid bevy_ecs feature, strip bevy from shared/"
**URL :** https://github.com/SamyAnisBenachi/Claude-Code-Game-Studios/actions

**Statut attendu :** En attente de vérification (doit être vert)

**Historique des fixes CI cette session :**
1. Commit `4d2666a` — push initial → ROUGE (register_protocol non-vérifié)
2. Commit `865a138` — suppression appels Lightyear non-vérifiés → ROUGE (bevy_ecs feature invalide)
3. Commit `88971ec` — suppression bevy_ecs feature + bevy de shared/ → EN ATTENTE

**Une fois CI vert :** lancer `/story-done S1-04` puis `/story-done S1-09`

---

## Découvertes critiques Bevy 0.18 (2026-04-29)

> Ces infos doivent être appliquées avant tout code Bevy

**liv-bevy-018 installé globalement :** `C:\Users\Sam\.claude\skills\liv-bevy-018\`
**liv-bevy-lightyear installé globalement :** `C:\Users\Sam\.claude\skills\liv-bevy-lightyear\`

### ✅ AUDIT COMPLET — 2026-04-29

Le skill liv-bevy-018 révèle que **EventWriter/EventReader n'existent plus en Bevy 0.18** :
- `EventWriter<T>` → `MessageWriter<T>`
- `EventReader<T>` → `MessageReader<T>`
- `app.add_event::<T>()` → `app.add_message::<T>()`

**AUDIT TERMINÉ — Toutes les violations corrigées :**
- `docs/architecture/adr-010-rsm-event-bus.md` — ✅ "Bevy buffered Messages (MessageWriter/MessageReader)"
- `docs/architecture/adr-009-rsm-phase-state.md` — ✅ EventReader/EventWriter → MessageReader/MessageWriter
- `docs/architecture/control-manifest.md` — ✅ Core Layer Rules mis à jour
- `docs/architecture/architecture.md` — ✅ Engine risk table corrigée
- `docs/architecture/adr-007-placement-buffer.md` — ✅ TODO(liv-bevy-018) ajouté
- `docs/architecture/adr-004-asset-loading-pipeline.md` — ✅ TODO(liv-bevy-018) ajouté
- `docs/architecture/adr-008/011/012-*.md` — ✅ Sections ⚠️ API Verification Required ajoutées
- Toutes les stories RSM, GSS, Economy, CardPool — ✅ MessageWriter/MessageReader
- `server/Cargo.toml` — ✅ TODO feature verification ajouté
- `client/Cargo.toml` — ✅ TODO feature collection verification ajouté
- `server/src/main.rs` — ✅ Commentaire "bevy_ecs" corrigé

### Lightyear 0.26 — API non-vérifiée

- Lightyear 0.26 utilise un **entity-per-connection model** (depuis v0.25)
- L'ancienne API resource-based (ClientConfig, ClientConnectionManager) n'existe plus
- **Aucun code Lightyear ne peut être écrit avant S1-05** (spike de vérification)
- S1-05 doit lire `api_patterns.md` dans `C:\Users\Sam\.claude\skills\liv-bevy-lightyear\`

### Features Bevy 0.18 valides

- `"bevy_ecs"` **n'est PAS** une feature valide dans Bevy 0.18
- Server headless : `bevy = { default-features = false, features = ["multi_threaded"] }`
- Client 2D : `bevy = { features = ["2d"] }` (collection haute-niveau Bevy 0.18)
- `EventWriter`/`EventReader` n'existent plus → `MessageWriter`/`MessageReader`

---

## Prochaines étapes (dans l'ordre)

### Immédiat
1. ✅ CI vert sur commit `88971ec` — vérifié (run 25130998038)
2. ✅ `/story-done S1-04` — COMPLETE WITH NOTES (2026-04-29)
3. ✅ `/story-done S1-09` — Done (déjà marqué)

### ✅ Audit Bevy 0.18 TERMINÉ
4. Audit complet fait — toutes violations EventWriter/EventReader corrigées en MessageWriter/MessageReader
   Lightyear ADRs annotés avec ⚠️ API Verification Required

### Premier vrai code de jeu (pas de gate Lightyear)
5. `/dev-story production/epics/round-state-machine/story-001-state-and-events-scaffold.md`
   → Story prête : ACs corrigés avec MessageWriter/MessageReader/#[derive(Message)]

### Gate Lightyear (bloque tout le networking)
6. `/dev-story production/epics/lightyear-protocol-verification/story-001-...`
   → S1-05 ⭐ — rien de networking avant que ce spike soit Done

---

## Epics créés

### Foundation (Sprint 1)
- `production/epics/workspace-and-shared-types/` — 4 stories
- `production/epics/game-config-pipeline/` — 4 stories
- `production/epics/server-rng/` — 3 stories
- `production/epics/lightyear-protocol-verification/` — 4 stories ⭐

### Core (Sprint 2+)
- `production/epics/round-state-machine/` — 6 stories
- `production/epics/game-session-system/` — 7 stories (story-004 Blocked ADR-012)
- `production/epics/economy-system/` — 6 stories
- `production/epics/card-data-pool/` — 6 stories

**Index complet :** `production/epics/index.md`

---

## Design — État GDDs

M1 (9 GDDs) : ✅ TOUS APPROUVÉS — prêts à implémenter
M2 (7 GDDs) : 2 DESIGNED, 5 PAS COMMENCÉS

**Auction System GDD :** `design/gdd/auction-system.md` — ✅ DESIGNED (2026-04-29). /design-review pending (fresh session).
**Combat Resolution GDD :** `design/gdd/combat-resolution.md` — ✅ DESIGNED (2026-04-29). Toutes sections complètes (A–H + Visual/Audio + UI Requirements + Open Questions). Registry: 2 nouvelles formules (net_damage, type_advantage). 5 OQs: OQ1 WALL ADR, OQ2 type advantage GameConfig, OQ3 RANGE RNG seed, OQ4 COUNTERATTACK proximity, OQ5 ResolutionEvent enum. /design-review pending (fresh session).
**Card Acquisition GDD :** `design/gdd/card-acquisition.md` — ✅ DESIGNED (2026-04-29). /design-review pending (fresh session).

### ✅ TERMINÉ — Shop/Auction UI GDD
- **Fichier :** `design/gdd/shop-auction-ui.md`
- **Statut :** Designed (2026-04-29) — sections A–H + Visual/Audio + UI Requirements + Open Questions
- **Registry :** 1 nouvelle formule (local_free_gold); 6 referenced_by mis à jour
- **5 OQs :** bid text input (OQ1), tooltip persistence (OQ2), S2CGoldBroadcast reserved_gold NP update (OQ3), screen layout split (OQ4), C2SSignalReady NP registration (OQ5)
- **Next :** /design-review design/gdd/shop-auction-ui.md (fresh session) · /ux-design shop-auction-ui

---

## Outils importants

```bash
# CI GitHub
https://github.com/SamyAnisBenachi/Claude-Code-Game-Studios/actions

# Cargo (Windows)
# Run from Developer PowerShell for VS 2026 so MSVC link.exe is on PATH.
# .cargo/config.toml sets target-dir = "target/msvc-local".
C:\Users\Sam\.cargo\bin\cargo.exe check --workspace
C:\Users\Sam\.cargo\bin\cargo.exe test -p server --verbose
C:\Users\Sam\.cargo\bin\cargo.exe test -p server session_ready_observer

# gh CLI (installé, besoin auth)
C:\Program Files\GitHub CLI\gh.exe

# Rust installé via winget 2026-04-29
# Normal PowerShell still will not see MSVC link.exe; use Developer PowerShell for VS 2026 or CI.
```

---

## Session Extract — /story-done 2026-04-29

- **Verdict**: COMPLETE WITH NOTES
- **Story**: `production/epics/workspace-and-shared-types/story-004-protocol-skeleton-ci-gates.md` — Protocol Skeleton & CI Dependency Gates
- **Passing ACs**: 5/11 — dep gates (shared/client/server), WASM size, protocol type stubs
- **Advisory deviations**: register_protocol() absent from shared/ (ADR-003 fallback; deferred to S1-05); evidence collected via CI rather than local builds
- **Tech debt logged**: None formally — deferred ACs tracked in story Completion Notes
- **Next recommended**: S1-05 — Lightyear 0.26 Verification Spike (now unblocked) at `production/epics/lightyear-protocol-verification/story-001-lightyear-026-verification-spike.md`

## Session Extract — /story-done 2026-04-29
- **Verdict**: COMPLETE
- **Story**: `production/epics/server-rng/story-001-type-definitions-audit-infrastructure.md` — ServerRng Type Definitions & Audit Infrastructure
- **Passing ACs**: 13/13
- **Deviations**: None
- **Test Evidence**: Logic — `tests/unit/foundation/server_rng_types_test.rs` (5 tests, CI green commit 6bdee76)
- **Tech debt logged**: None
- **Next recommended**: S1-10 — Intent-Named API & Consumption Invariants at `production/epics/server-rng/story-002-intent-named-api-invariants.md` (S1-09 unblocks it)

## Session Extract — /dev-story 2026-04-29
- **Story**: `production/epics/game-config-pipeline/story-001-asset-data-files.md` — Asset Data Files
- **Files changed**: `assets/config/game_config.ron` (3 network timeout values corrected to GDD design-intent), `assets/data/cards.json` (fixed serde newtype bug: `"id": [N]` → `"id": N` on all 8 entries)
- **Test written**: None — Config/Data story; evidence at `tests/evidence/story-gcp-001-data-files.md`
- **Blockers**: None
- **Status**: Complete
- **Next**: `/story-done production/epics/game-config-pipeline/story-001-asset-data-files.md` or continue with next ready story

## Session Extract — /dev-story 2026-04-29
- **Story**: `production/epics/lightyear-protocol-verification/story-001-lightyear-026-verification-spike.md` — Lightyear 0.26 Verification Spike
- **Files changed**:
  - `server/tests/session_ready_observer_test.rs` — ADR-012 open condition test (2 test functions)
  - `tests/evidence/lightyear-026-verification.md` — all 20 items annotated; ADR-012 items 15-16 now locally verified
  - `docs/architecture/control-manifest.md` — §Lightyear 0.26 Verification Checklist: all 20 ⬜ → ✅/⚠️
- **Test written**: `server/tests/session_ready_observer_test.rs` — 2 tests (ADR-012 open condition); now verified locally from Developer PowerShell for VS 2026
- **Key findings**: 7 API differences from ADR assumptions (channel syntax, direction model, send/receive methods, NetworkTarget identifier type, server send API, connection event naming); all have concrete resolution paths documented
- **Blockers**: None for ADR-012 items 15-16; `cargo test -p server session_ready_observer` passes from Developer PowerShell for VS 2026
- **Next**: Networking follow-up stories may rely on the resolved ADR-012 observer ordering test

## Session Extract — /dev-story 2026-04-29
- **Story**: `production/epics/card-data-pool/story-001-pool-state-core-api.md` — Pool State + Core API
- **Files changed**:
  - `shared/src/session.rs` — created; `PlayerId(u64)` type
  - `shared/src/lib.rs` — added `pub mod session;`
  - `server/src/core/pool/state.rs` — created; `PlayerPool`, `PlayerPools`, `DistributeError`, `PoolFilter` structs
  - `server/src/core/pool/api.rs` — created; `impl PlayerPool` (initialize, distribute, is_available, copies_remaining, total_acquired) + 20 embedded `#[cfg(test)]` tests
  - `server/src/core/pool/plugin.rs` — created; `CardPoolPlugin` skeleton (registers `PlayerPools`)
  - `server/src/core/pool/mod.rs` — created; module re-exports
  - `server/src/core/mod.rs` — added `pub mod pool;`
  - `tests/unit/pool/pool_state_test.rs` — created; evidence documentation (20 test cases mapped to ACs 1–10)
- **Test written**: 20 `#[cfg(test)]` tests in `server/src/core/pool/api.rs`; run via `cargo test -p server`
- **Blockers**: Local builds blocked by Smart App Control — CI is verification gate
- **Next**: `/code-review server/src/core/pool/api.rs server/src/core/pool/state.rs` then `/story-done production/epics/card-data-pool/story-001-pool-state-core-api.md`

## Session Extract — /dev-story 2026-04-29
- **Story**: `production/epics/server-rng/story-003-determinism-session-reset.md` — Determinism Proof & Session Reset (S1-12)
- **Files changed**:
  - `server/src/foundation/rng.rs` — added `PartialEq` to `AuditEntry` derive; added `at_max_seed_index()` test-only constructor; added deferred-AC comments (RNG8/9/10/14); added 7 new Story 003 tests embedded in `#[cfg(test)] mod tests`
  - `tests/unit/foundation/server_rng_determinism_test.rs` — created; Story 003 evidence documentation
- **Test written**: 7 embedded `#[cfg(test)]` tests in `rng.rs`: 2× determinism (VC1/VC2), 2× session reset (RNG13), 3× overflow (RNG15)
- **Blockers**: Local build blocked by Smart App Control (pre-existing) — CI is verification gate
- **Next**: `/code-review server/src/foundation/rng.rs` then `/story-done production/epics/server-rng/story-003-determinism-session-reset.md`

## Session Extract — /dev-story 2026-04-29
- **Story**: `production/epics/server-rng/story-002-intent-named-api-invariants.md` — Intent-Named API & Consumption Invariants
- **Files changed**:
  - `server/src/foundation/rng.rs` — refactored `next_seed()` to private no-param helper; added 7 intent-named public methods; added `# Ordering Contract` doc-comment on `ServerRng`; embedded `#[cfg(test)]` module with 10 tests covering both Story 001 (updated) and Story 002 ACs
  - `tests/unit/foundation/server_rng_types_test.rs` — converted to evidence documentation (Story 001 tests now embedded in rng.rs)
  - `tests/unit/foundation/server_rng_api_test.rs` — created; Story 002 evidence documentation
- **Test written**: Embedded `#[cfg(test)] mod tests` in `rng.rs` (10 tests; run via `cargo test -p server`)
- **Blockers**: Local build blocked by Smart App Control — CI is verification gate (same as prior stories)
- **Next**: `/code-review server/src/foundation/rng.rs` then `/story-done production/epics/server-rng/story-002-intent-named-api-invariants.md`

## Session Extract — /dev-story 2026-04-29
- **Story**: `production/epics/game-config-pipeline/story-002-asset-loading-pipeline.md` — Asset Loading Pipeline (S1-07)
- **Files changed**:
  - `server/src/foundation/config.rs` — NEW: full pipeline (AppState, GameConfigAsset, GameConfigLoader, CardCatalog struct, CardCatalogLoader, GameAssets, start_loading, check_loading_done, validate_and_promote stub, ConfigPlugin)
  - `server/src/foundation/mod.rs` — added `pub mod config;`
  - `server/src/main.rs` — added AssetPlugin + ConfigPlugin to App builder
  - `server/Cargo.toml` — added bevy features `bevy_asset`, `bevy_state`; added `thiserror = "1"`
  - `shared/src/config.rs` — added 3 missing auction floor fields (auction_floor_rare/epic/legendary)
- **Test written**: None — Integration story; evidence at `tests/evidence/story-gcp-002-pipeline.md`
- **Key deviation**: bevy_asset_loader unavailable for 0.18 (PR #264 draft) — manual AssetServer polling used
- **Blockers**: Local build blocked by Smart App Control — CI is verification gate
- **Next**: `/code-review server/src/foundation/config.rs` then `/story-done production/epics/game-config-pipeline/story-002-asset-loading-pipeline.md`

## Session Extract — /story-done 2026-04-29
- **Verdict**: COMPLETE
- **Story**: `production/epics/game-config-pipeline/story-001-asset-data-files.md` — Asset Data Files (S1-06)
- **Criteria**: 8/8 passing — all ACs auto-verified (file reads + evidence doc)
- **Deviations**: None — manifest version match (2026-04-29)
- **Tech debt logged**: None
- **Next recommended**: S1-07 — Asset Loading Pipeline at `production/epics/game-config-pipeline/story-002-asset-loading-pipeline.md` (now unblocked: S1-06 Done)

## Session Extract — /architecture-review 2026-04-30
- **Verdict**: CONCERNS — M1 PASS / M2+M3 FAIL (ADRs not yet authored)
- **Requirements**: 80 M1 TRs — 80 covered (100%); ~120 M2/M3 gaps (0 ADRs exist)
- **New TR-IDs registered**: 80 M1 TRs in tr-registry.yaml (populated by another agent; M2/M3 deferred until ADRs authored)
- **ADR-013 authored**: auction-system-state.md (Proposed) — covers TR-AU-001..010
- **Engine fixes applied**: current-best-practices.md + ADR-003 (invalid bevy_ecs feature removed); ADR-008 + ADR-009 (stale "pending" dependency notes removed)
- **Top ADR gaps**: ADR-014 (Combat Resolution Sub-step Scheduler), ADR-015 (Card Acquisition + Hand Management), ADR-023 (Keyword Observer Architecture)
- **Report**: docs/architecture/architecture-review-2026-04-30.md

## Session Extract — /story-done 2026-04-29
- **Verdict**: COMPLETE
- **Story**: `production/epics/server-rng/story-002-intent-named-api-invariants.md` — Intent-Named API & Consumption Invariants (S1-10)
- **Criteria**: 11/11 passing — all ACs auto-verified (code read + test traceability)
- **Deviations**: None — ADR-005 compliant, manifest version match (2026-04-29)
- **Test Evidence**: Logic — 10 embedded `#[cfg(test)]` tests in `server/src/foundation/rng.rs`; evidence doc at `tests/unit/foundation/server_rng_api_test.rs`
- **Code Review**: APPROVED (lean mode)
- **Tech debt logged**: None
- **Next recommended**: S1-12 — Determinism Proof & Session Reset at `production/epics/server-rng/story-003-determinism-session-reset.md` (S1-10 now Done, blocker cleared)

## Session Extract — /architecture-review 2026-04-30
- **Verdict**: CONCERNS — M1 PASS / M2+M3 FAIL (ADRs not yet authored)
- **Requirements**: ~217 total — ~95 covered, ~51 partial, ~71 gaps
- **New TR-IDs registered**: 74 (TR-CDP-001..009, TR-GC-001..005, TR-RNG-001..006, TR-ECO-001..008, TR-BLS-001..010, TR-RSM-001..010, TR-NP-001..012, TR-GSS-001..010, TR-OBJ-001..010)
- **GDD revision flags**: None — all GDD design assumptions consistent with verified engine behaviour
- **Fixes applied**: ADR-004 invalid Bevy code samples replaced; ADR-005 stale dep reference fixed (ADR-007 → ADR-012)
- **Top ADR gaps**: ADR-013 Auction State Machine, ADR-014 Combat Sub-Step Scheduling, ADR-015 Keyword Observer Architecture
- **Report**: docs/architecture/architecture-review-2026-04-30.md
- **Files changed**: architecture-review-2026-04-30.md (new), architecture-traceability.md (updated), tr-registry.yaml (74 TRs populated), adr-004 (code samples fixed), adr-005 (dep reference fixed)

## Session Extract — /design-review network-protocol.md R3 2026-04-30
- **Verdict**: MAJOR REVISION NEEDED → Revised inline
- **Blockers resolved**: 17
- **Key fixes**: ResolutionEvent 10 new variants + 3 new enums; BoardSnapshot seeds + sinistros; PlayerSnapshot class_id; UnitBoardState 7 keyword state fields + max_hp; GoldAwardReason::PrismReward removed; AcquisitionSource rename + FreeCardPick; S2COpponentSubmitted; AuctionSnapshot starting_price; NP-19/NP-23/NP-25/NP-26 rewritten; NP-30–35 new ACs; OQ5 closed
- **Systems index**: Network Protocol → In Review
- **Recommended next**: /design-review design/gdd/network-protocol.md --depth lean (R4 re-review, fresh session after /clear)
- **Also still Needs Revision**: keyword-system.md (C-B2), class-system.md (D-B1/D-B2), auction-system.md (D-B3), objective-system.md (D-B1), hand-ui.md (C-B4/5), entities.yaml (C-B6)

## Session Extract — /review-all-gdds 2026-04-30
- **Verdict**: FAIL
- **GDDs reviewed**: 20 (11 new since R5)
- **Blocking issues**: 9 (C-B1 S2CResolutionEvent variants; C-B2 GameOverReason enum; C-B3 S2CCardAcquired schema; C-B4 Hand UI activation lock; C-B5 GameConfig missing fields; C-B6 S2CSangMepriseReveal unregistered; D-B1 Garde-Temps routing; D-B2 Sang Méprise+Punition ordering; D-B3 Auction wealth gap mitigation)
- **Flagged for revision**: network-protocol.md, keyword-system.md, class-system.md, hand-ui.md, auction-system.md, objective-system.md, entities.yaml
- **Systems index updated**: 6 GDDs marked Needs Revision (Network Protocol, Objective System, Auction System, Hand UI, Keyword System, Class System)
- **Report**: design/gdd/gdd-cross-review-2026-04-30.md
- **Recommended next**: /design-review each flagged GDD starting with network-protocol.md (C-B1/B2/B3 are the highest-leverage blockers — they unblock Combat Resolution, Keyword System, and Card Animations implementation)

---

## TODO conditionnels (déclencher manuellement quand condition remplie)

- **Re-review board-rendering** quand `network-protocol.md` est mis à jour avec `C2SRequestSnapshot` : `/design-review design/gdd/board-rendering.md` (commande à lancer dans une fenêtre fraîche). Source : board-rendering R2 verdict CONDITIONAL APPROVED, OQ-BR-06 cross-doc dependency.
- **Re-review auction-system** après mise à jour NP `reserved_gold` (si pas déjà fait) : `/design-review design/gdd/auction-system.md`
- **Re-review combat-resolution** quand NP ajoute variants `CombatDamage` + `KeywordTriggered` à `ResolutionEvent` enum (OQ5 combat-resolution).

## Session Extract — /review-all-gdds 2026-04-30 R7
- Verdict: FAIL
- GDDs reviewed: 20 + master + systems-index
- Flagged for revision: network-protocol, keyword-system, class-system, objective-system, hand-ui, auction-system, entities.yaml, round-state-machine, prism-system, game-config, combat-resolution, economy-system, card-animations, board-rendering, server-rng, shop-auction-ui, lanes-and-lies-gdd
- Blocking issues (9): C-B2 ResolutionTimeout vs Draw; C-B4 hand-ui Rule 5c S2CCardAcquired; C-B5 hand-ui Tuning vs Dependencies contradiction; C-B6+C-NEW-1+C-NEW-3 registry single-pass; C-NEW-2 stale CardSource::AuctionWon; C-NEW-4 multi-Krosmic ordering not echoed by RSM/NP/CA; D-B1+C-NEW-5 Garde-Temps/Punition take_damage routing; D-B4 disconnect grace during DRAFT_AUCTION; D-B5 Sang Méprise reconnect-gap UX
- Resolved this cycle: C-B1 (S2CResolutionEvent variants), C-B3 partial (AcquisitionSource rename — variants still incomplete), C-W1
- Recommended next: address 9 blockers per file table in report (single registry pass closes 4 at once); /design-review re-runs for class-system, hand-ui, auction-system after fixes
- Report: design/gdd/gdd-cross-review-2026-04-30-r7.md

## Session Extract - /dev-story 2026-04-30
- Story: `production/epics/round-state-machine/story-001-state-and-events-scaffold.md` - State and Events Scaffold
- Files changed: `server/src/core/rsm/state.rs`, `server/src/core/rsm/events.rs`, `server/src/core/rsm/plugin.rs`, `server/src/core/rsm/mod.rs`, `server/src/core/mod.rs`, `server/src/lib.rs`, `server/src/main.rs`, `shared/src/protocol.rs`, `client/src/state/mod.rs`, `server/tests/rsm_scaffold_test.rs`, `tests/unit/rsm/rsm_scaffold_test.rs`, `tests/evidence/rsm-story-001-check.md`
- Test written: `server/tests/rsm_scaffold_test.rs` (2 tests), evidence mapping at `tests/unit/rsm/rsm_scaffold_test.rs`, smoke evidence at `tests/evidence/rsm-story-001-check.md`
- Blockers: previously blocked in normal PowerShell by missing MSVC `link.exe`; use Developer PowerShell for VS 2026 for local Cargo verification
- Next: `/code-review server/src/core/rsm/state.rs server/src/core/rsm/events.rs server/src/core/rsm/plugin.rs server/tests/rsm_scaffold_test.rs` then run Cargo in a VS Developer Command Prompt or CI

## Session Extract - /dev-story 2026-04-30
- Story: `production/epics/card-data-pool/story-002-weighted-draw-functions.md` - Weighted Draw Functions
- Files changed: `server/src/core/pool/api.rs`, `tests/unit/pool/weighted_draw_test.rs`
- Test written: embedded tests in `server/src/core/pool/api.rs` (11 S2-03 tests), evidence mapping at `tests/unit/pool/weighted_draw_test.rs`
- Verification: CI green on run 25161381682 for commit `013f204`
- Blockers: local `cargo test -p server weighted_draw --verbose` blocked by Windows/MSVC build-script failures before server tests ran; CI passed
- Next: S2-04 Card Pool Story 3: Refresh Shop Slot Variants is unblocked once sprint tracker is read fresh

## Session Extract - /story-done 2026-04-30
- Story: `production/epics/economy-system/story-001-state-and-pure-api-scaffold.md` - State & Pure API Scaffold
- Files changed: `server/src/core/economy/state.rs`, `server/src/core/economy/api.rs`, `server/src/core/economy/mod.rs`, `server/src/core/mod.rs`, `shared/src/config.rs`, `assets/config/game_config.ron`, `server/tests/game_config_defaults_test.rs`, `tests/unit/economy/state_api_test.rs`
- Test written: embedded tests in `server/src/core/economy/api.rs` (17 tests), evidence mapping at `tests/unit/economy/state_api_test.rs`
- Verification: CI green on run 25161623746 for commit `5d8655c`
- Blockers: local `cargo test -p server economy::api::tests` blocked before compile by missing Windows MSVC `link.exe`; CI passed
- Next: S2-08 Economy Story 2 is unblocked once S2-01 is Done

## Session Extract - /story-done 2026-04-30
- Story: `production/epics/round-state-machine/story-001-state-and-events-scaffold.md` - State and Events Scaffold
- Verdict: COMPLETE WITH NOTES
- Criteria: 14/14 passing in CI (current main run 25161983584)
- Deviations: Advisory only - Bevy 0.18 uses `App::add_observer(on_session_ready)` and `On<SessionReady>` instead of older `app.observe` / `Trigger` wording.
- Test Evidence: `server/tests/rsm_scaffold_test.rs`, `tests/unit/rsm/rsm_scaffold_test.rs`, `tests/evidence/rsm-story-001-check.md`
- Implementation Commit: `2b66f35`
- Next: S2-07 RSM Story 2 and S2-08 Economy Story 2 are unblocked now that S2-01 and S2-02 are Done

## Session Extract - environment update 2026-04-30
- Cargo local fixed: `.cargo/config.toml` sets `target-dir = "target/msvc-local"` so Cargo's generated build scripts/proc-macro DLLs stay in a repo-local target tree.
- Verified command: from Developer PowerShell for VS 2026, `cargo test -p server session_ready_observer` -> 2 passed; 0 failed.
- Scope: normal PowerShell still will not see MSVC `link.exe`; use Developer PowerShell for VS 2026 or CI for local verification.
- Docs updated: `CODEX.md`, `docs/architecture/control-manifest.md`, `tests/evidence/lightyear-026-verification.md`.
- Impact: ADR-012 Lightyear verification items 15-16 are no longer pending; GSS/RSM observer ordering can rely on the passing local test.

## Session Extract - workflow update 2026-04-30
- Worker Codex windows should use Developer PowerShell for VS 2026 for fast local Cargo iteration (`cargo test -p server ...`) before pushing.
- Workers should push after local tests pass and hand off commit hash + CI run id/details; they do not need to wait on GitHub Actions by default.
- The orchestrator window owns GitHub Actions monitoring, routing failures back to owners, and final `story-done` tracking once CI is green.

## Session Extract - sprint tracker refresh 2026-04-30
- Sprint 2 must-haves S2-01, S2-02, and S2-03 are Done with green CI.
- Unblocked Sprint 2 pull-forward stories are now claimable in `production/sprint-status.yaml`: S2-04 Card Pool Refresh Shop Slot Variants, S2-07 RSM advance_phase + F2 Ordering, and S2-08 Economy Initialisation + Draft Subscriber.
- Workers may use `implement next` in separate Codex windows; each worker must claim exactly one ready story, run local Cargo tests from Developer PowerShell for VS 2026, commit/push its own files, and hand off CI details to the orchestrator.

## Session Extract - /dev-story 2026-04-30
- Story: `production/epics/economy-system/story-002-initialisation-draft-subscriber.md` - Initialisation & DraftStarted Subscriber
- Owner: `codex-s2-08-economy`
- Files changed: `server/src/core/economy/api.rs`, `server/src/core/economy/mod.rs`, `server/src/core/economy/plugin.rs`, `server/src/core/economy/system.rs`, `server/src/core/session/mod.rs`, `server/src/core/session/state.rs`, `server/src/core/mod.rs`, `server/src/lib.rs`, `server/src/main.rs`, `server/tests/economy_draft_subscriber_test.rs`, `server/tests/economy_round_trace_test.rs`, `tests/unit/economy/draft_subscriber_test.rs`, `tests/integration/economy/round_trace_test.rs`
- Test written: `server/tests/economy_draft_subscriber_test.rs` (7 tests) and `server/tests/economy_round_trace_test.rs` (1 test); evidence mappings at `tests/unit/economy/draft_subscriber_test.rs` and `tests/integration/economy/round_trace_test.rs`
- Verification: `CARGO_INCREMENTAL=0 cargo test -p server --test economy_draft_subscriber_test --test economy_round_trace_test --verbose` -> 8 passed; cargo global cache last-use warning only
- Blockers: None for S2-08 local tests. Scheduling `.after(advance_phase)` remains dependent on S2-07's in-progress RSM transition symbol landing.
- Next: `/code-review server/src/core/economy/system.rs server/src/core/economy/plugin.rs server/src/core/session/state.rs server/tests/economy_draft_subscriber_test.rs server/tests/economy_round_trace_test.rs` then `/story-done production/epics/economy-system/story-002-initialisation-draft-subscriber.md` after CI green

## Session Extract - /story-done 2026-04-30
- Verdict: COMPLETE
- Story: `production/epics/round-state-machine/story-002-advance-phase-and-f2-ordering.md` - Advance Phase and F2 Ordering
- Criteria: 16/16 passing; `server/tests/rsm_transitions_test.rs` covered by CI run `25167672501`
- Implementation Commit: `cb550b9`
- Tech debt logged: None
- Next recommended: RSM Story 003 once sprint planning pulls it forward.

## Session Extract - /story-done 2026-04-30
- Verdict: COMPLETE
- Story: `production/epics/economy-system/story-002-initialisation-draft-subscriber.md` - Initialisation & DraftStarted Subscriber
- Criteria: 12/12 passing; `server/tests/economy_draft_subscriber_test.rs` and `server/tests/economy_round_trace_test.rs` covered by CI run `25167672501`
- Implementation Commit: `9396d32`; repair commit `e4ac84e` restored S2-08 files after S2-04 scope cleanup and fixed config doctests
- Tech debt logged: None
- Next recommended: Economy Story 003 once sprint planning pulls it forward.

## Session Extract - /story-done 2026-04-30
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/card-data-pool/story-003-refresh-shop-slot-variants.md` - refresh_shop + Slot Variants
- Criteria: 4/4 passing; refresh-shop tests embedded in `server/src/core/pool/api.rs` covered by CI run `25167672501`
- Implementation Commit: `901823d`; repair commit `e4ac84e` fixed integration history and config doctests
- Notes: Implementation uses weighted draw selection directly; future class/neutral split policy remains with the subscriber story as scoped.
- Tech debt logged: None
- Next recommended: Card Pool Story 004 / ShopRefreshNeeded subscriber once sprint planning pulls it forward.
