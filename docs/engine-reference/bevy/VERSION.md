# Bevy Engine — Version Reference

| Field | Value |
|-------|-------|
| **Engine Version** | Bevy 0.18 |
| **Language** | Rust (stable, edition 2021) |
| **Project Pinned** | 2026-04-28 |
| **Last Docs Verified** | 2026-04-28 |
| **LLM Knowledge Cutoff** | ~Bevy 0.14 (August 2025 training cutoff) |
| **Risk Level** | HIGH — 4 versions of breaking changes post-cutoff |

## Knowledge Gap Warning

The LLM's training data covers Bevy up to approximately **0.14**. Versions 0.15, 0.16,
0.17, and 0.18 all introduced significant breaking changes. Do NOT suggest pre-0.15 API
patterns without cross-referencing these docs first.

## MANDATORY Skill Activation

**Every agent writing, modifying, or reviewing Bevy code MUST activate these skills:**

| File type | Skill to activate | Why |
|---|---|---|
| Any `.rs` file importing `bevy` | `liv-bevy-018` | Enforces 0.18 API patterns; prevents deprecated Bundle/pre-0.15 patterns |
| Any `.rs` file importing `lightyear` | `liv-bevy-lightyear` | Lightyear 0.26 API; WebSocket/replication patterns for Bevy 0.18 |
| Both in same file | Activate **both** skills | Networking code uses both APIs simultaneously |

These skills are **not optional**. Without them, agents will generate code using
pre-0.15 Bundle patterns (e.g. `SpriteBundle`, `Camera2dBundle`, `NodeBundle`),
deprecated hierarchy APIs (`set_parent`, `despawn_recursive`), and outdated event
patterns (`send()` instead of `write()`) — all of which will fail to compile on
Bevy 0.18.

## Post-Cutoff Version Timeline

| Version | Risk | Key Theme |
|---------|------|-----------|
| 0.15 | HIGH | Required Components (bundles deprecated), Transform auto-GlobalTransform |
| 0.16 | HIGH | Query::single() returns Result, Event→Message split, ChildOf replaces Parent, UiImage→ImageNode |
| 0.17 | HIGH | Event/Observer split formalized, bevy_render reorganized, wgpu 25 |
| 0.18 | HIGH | LineHeight required component, RenderTarget required component, Entities API overhaul, Input behind features |

## Verified Dependency Versions (Bevy 0.18 compatible)

| Crate | Version | Notes |
|-------|---------|-------|
| `bevy` | `0.18` | Core engine |
| `lightyear` | `0.26.0` | Bevy 0.18 compatible; released Jan 2026 |
| `bevy_tweening` | `0.18` | Follows Bevy version numbering exactly |
| `bevy_asset_loader` | verify on crates.io | Check for 0.18-compatible release |
| `rand` | `0.9` | Server-side RNG |
| `rand_chacha` | `0.3` | Deterministic seeded RNG |

## Verified Sources

- Bevy 0.18 release notes: https://bevy.org/news/bevy-0-18/
- 0.17 → 0.18 migration guide: https://bevy.org/learn/migration-guides/0-17-to-0-18/
- 0.16 → 0.17 migration guide: https://bevy.org/learn/migration-guides/0-16-to-0-17/
- 0.15 → 0.16 migration guide: https://bevy.org/learn/migration-guides/0-15-to-0-16/
- 0.14 → 0.15 migration guide: https://bevy.org/learn/migration-guides/0-14-to-0-15/
- Lightyear releases: https://github.com/cBournhonesque/lightyear/releases
- bevy_tweening: https://github.com/djeedai/bevy_tweening
