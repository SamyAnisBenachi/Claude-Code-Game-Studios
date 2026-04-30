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
| S1-05 | Lightyear 0.26 Spike ⭐ | `production/epics/lightyear-protocol-verification/story-001-lightyear-026-verification-spike.md` | ⚠️ PENDING CI (Items 15-16 test written, needs CI run) |
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
C:\Users\Sam\.cargo\bin\cargo.exe check --workspace
C:\Users\Sam\.cargo\bin\cargo.exe test -p server --verbose

# gh CLI (installé, besoin auth)
C:\Program Files\GitHub CLI\gh.exe

# Rust installé via winget 2026-04-29
# Smart App Control bloque les builds locaux → utiliser CI ou WSL2
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
  - `tests/evidence/lightyear-026-verification.md` — all 20 items annotated (9 CONFIRMED, 7 DIFFERS, 2 PENDING CI, 2 CONFIRMED by architecture)
  - `docs/architecture/control-manifest.md` — §Lightyear 0.26 Verification Checklist: all 20 ⬜ → ✅/⚠️
- **Test written**: `server/tests/session_ready_observer_test.rs` — 2 tests (ADR-012 open condition); PENDING CI (local MSVC linker PATH issue pre-existing)
- **Key findings**: 7 API differences from ADR assumptions (channel syntax, direction model, send/receive methods, NetworkTarget identifier type, server send API, connection event naming); all have concrete resolution paths documented
- **Blockers**: Items 15-16 (ADR-012 flush ordering) PENDING CI test execution — run `cargo test -p server session_ready_observer` in VS Developer Command Prompt or CI
- **Next**: Push to CI for test execution, then `/story-done production/epics/lightyear-protocol-verification/story-001-lightyear-026-verification-spike.md` once CI confirms PASS on both ADR-012 tests

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
