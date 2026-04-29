# Prompt — Agent d'audit Bevy 0.18 / Lightyear 0.26

> Copie ce prompt dans un nouvel agent pour corriger les violations Bevy 0.18
> découvertes le 2026-04-29.

## Skills à lire EN PREMIER (obligatoire)

1. `C:\Users\Sam\.claude\skills\liv-bevy-018\SKILL.md`
2. `C:\Users\Sam\.claude\skills\liv-bevy-018\REFERENCE.md`
3. `C:\Users\Sam\.claude\skills\liv-bevy-lightyear\SKILL.md`
4. `C:\Users\Sam\.claude\skills\liv-bevy-lightyear\api_patterns.md`
5. `C:\Users\Sam\.claude\skills\liv-bevy-lightyear\architecture.md`

## Violations connues à corriger

### Bevy 0.18 — EventWriter n'existe plus

`EventWriter<T>` / `EventReader<T>` / `Events<T>` n'existent plus en Bevy 0.18.
- Buffered (pull) → `MessageWriter<T>` / `MessageReader<T>` + `app.add_message::<T>()`
- Observer (push) → `#[derive(Event)]` + `commands.trigger()` + `Observer`

### Fichiers à auditer et corriger

**Code Rust :**
- `server/src/foundation/rng.rs`
- `server/src/main.rs`
- `client/src/main.rs`
- `tests/unit/foundation/server_rng_types_test.rs`

**Cargo.toml :**
- `server/Cargo.toml` — vérifier features Bevy 0.18 valides
- `client/Cargo.toml` — vérifier features (envisager `"2d"` collection)
- `shared/Cargo.toml` — vérifier qu'il n'y a plus de bevy ni lightyear

**Stories Core :**
- Tous `production/epics/round-state-machine/story-*.md`
- Tous `production/epics/game-session-system/story-*.md`
- Tous `production/epics/economy-system/story-*.md`
- Tous `production/epics/card-data-pool/story-*.md`
- Chercher `EventWriter`, `EventReader`, `add_event` → corriger

**ADRs :**
- `docs/architecture/adr-009-rsm-phase-state.md`
- `docs/architecture/adr-010-rsm-event-bus.md`
- `docs/architecture/control-manifest.md` (section Core Layer Rules)
- ADRs Lightyear (008, 011, 012) → ajouter note "⚠️ API Verification Required (S1-05)"

## Règles

- Lire le skill avant d'écrire — sans exception
- Ne jamais inventer une API Lightyear 0.26 → marquer `// TODO(S1-05)`
- Lire chaque fichier avant de l'éditer
- Ne pas toucher `shared/src/card.rs`, `shared/src/config.rs` (pure serde)
