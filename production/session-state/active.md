# Session State — Lanes and Lies

> Lis ce fichier EN PREMIER dans toute nouvelle session.
> Il contient l'état complet du projet au 2026-04-29.

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
| S1-04 | Protocol Skeleton + CI Gates | `story-004-protocol-skeleton-ci-gates.md` | ⚠️ Impl — attendre CI vert |
| S1-05 | Lightyear 0.26 Spike ⭐ | `production/epics/lightyear-protocol-verification/story-001-lightyear-026-verification-spike.md` | 🔒 Bloqué sur S1-04 |
| S1-09 | ServerRng Type Definitions | `production/epics/server-rng/story-001-type-definitions-audit-infrastructure.md` | ⚠️ Impl — attendre CI vert |

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
1. Vérifier CI vert sur commit `88971ec`
2. `/story-done production/epics/workspace-and-shared-types/story-004-protocol-skeleton-ci-gates.md`
3. `/story-done production/epics/server-rng/story-001-type-definitions-audit-infrastructure.md`

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
M2 (7 GDDs) : ❌ PAS COMMENCÉS — à designer pendant implémentation M1

**M2 priorité :** Auction System (signature mécanique) → Card Acquisition → Combat Resolution
**Commande :** `/design [system-name]` ou vérifier quel skill existe

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
