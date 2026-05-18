# Technical Preferences

<!-- Updated: 2026-04-28 — Bevy 0.18 + Lightyear stack confirmed. -->
<!-- All agents reference this file for project-specific standards and conventions. -->

## Engine & Language

- **Engine**: Bevy 0.18 (Rust)
- **Language**: Rust (stable toolchain, edition 2021)
- **Rendering**: Bevy 2D — sprites, TextureAtlas, bevy_ui (Required Components API)
- **Physics**: None (lane-based game; no physics engine needed)
- **Networking**: Lightyear (bevy_lightyear) over WebSocket (WASM client) / native (server)
- **Animation**: bevy_tweening for UI and unit movement transitions
- **Asset loading**: bevy_asset_loader (typed asset collections, loading states)

## Input & Platform

- **Target Platforms**: Web browser (WASM via Trunk) — primary. Native desktop as dev/debug target.
- **Input Methods**: Mouse + Keyboard (primary). Touch (stretch goal — not hackathon scope).
- **Primary Input**: Mouse click (card selection, bidding, lane targeting)
- **Gamepad Support**: None
- **Touch Support**: None (hackathon scope)
- **Platform Notes**: WASM bundle must stay under 50 MB. Trunk builds with `--release` for production. Server is a headless Rust binary deployed to Railway via Docker.

## Naming Conventions

- **Structs / Enums / Components / Events / Plugins**: `PascalCase` (e.g., `CardUnit`, `AuctionBidEvent`, `GamePlugin`)
- **Functions / Systems / Variables / Fields**: `snake_case` (e.g., `resolve_combat`, `current_gold`)
- **Constants / Statics**: `SCREAMING_SNAKE_CASE` (e.g., `OBJECTIVE_HP`, `MAX_HAND_SIZE`)
- **Files / Modules**: `snake_case.rs` (e.g., `auction_system.rs`, `card_pool.rs`)
- **Plugins**: suffix `Plugin` (e.g., `CombatPlugin`, `AuctionPlugin`, `LightyearPlugin`)
- **Systems**: verb_noun pattern (e.g., `spawn_unit`, `resolve_lane_combat`, `apply_interest`)
- **Resources**: noun, PascalCase (e.g., `GameConfig`, `CardPool`, `RoundState`)
- **Lightyear protocol types**: prefix with `C2S` (client-to-server) or `S2C` (server-to-client) for messages (e.g., `C2SPlaceUnit`, `S2CRoundResolved`)

## Performance Budgets

- **Target Framerate**: 60 FPS (browser/WASM)
- **Frame Budget**: 16.67ms total; game logic < 2ms; render < 12ms
- **Draw Calls**: Minimise via sprite batching — all units of same atlas in one draw call
- **WASM Bundle Size**: < 50 MB (release build with LTO + strip)
- **Memory**: < 256 MB WASM heap
- **Network**: < 1 KB per round message (lightyear delta compression assumed)

## Testing

- **Framework**: Bevy's built-in `World`-based ECS tests (`#[test]` with `World::new()`)
- **Minimum Coverage**: All economy formulas, combat damage formula, auction state machine, win condition check
- **Required Tests**: See GDD Section 8 Acceptance Criteria — all BLOCKING criteria need a test
- **Test location**: `tests/unit/[system]/` for unit; `tests/integration/[system]/` for multi-system
- **No mocks**: Test against real ECS `World` state, not mock systems (see `liv-bevy-018` for patterns)

## Forbidden Patterns

- **No client-side RNG** — all randomness (Ecaflip dice, shop rolls, fake-objective rewards) must be seeded and computed server-side, result broadcast to clients
- **No game state on client** — clients are views; all authoritative state lives on the Lightyear server
- **No `unwrap()` in production paths** — use `?` propagation or explicit `expect()` with a message
- **No `bevy_egui` in shipped build** — egui is dev/debug only; all shipped UI uses bevy_ui
- **No hardcoded balance values in systems** — all tuning knobs go through `GameConfig` resource loaded from `assets/config/game_config.ron`

## Allowed Libraries / Addons

| Crate | Version | Purpose |
|---|---|---|
| `bevy` | 0.18 | Core engine |
| `bevy_lightyear` | latest compatible with 0.18 | Multiplayer networking |
| `bevy_tweening` | latest compatible with 0.18 | UI and movement animations |
| `bevy_asset_loader` | latest compatible with 0.18 | Typed asset loading / loading screens |
| `rand` + `rand_chacha` | latest | Server-side seeded RNG (ChaCha for determinism) |
| `serde` + `serde_json` | latest | Card data serialisation (JSON card pool files) |
| `ron` | latest | Config files (`GameConfig`, card definitions) |
| `trunk` | latest | WASM build + dev server |
| `wasm-bindgen` | latest | WASM/JS boundary (if needed for browser APIs) |

## Architecture Decisions Log

| ADR | File | Status | Summary |
|---|---|---|---|
| ADR-001 | `docs/architecture/adr-001-objective-identity-unicast.md` | Accepted (Sang Méprise §5 sub-clause superseded by ADR-024) | `ObjectiveIdentity` sent as unicast message at DRAFT_INITIAL, not replicated ECS component. Lightyear 0.26 has no per-component replication scope. |
| ADR-024 | `docs/architecture/adr-024-sang-meprise-reveal-mechanism.md` | Accepted | Sang Méprise reveal: parallel unicast of full alive-objective `reveal_set` to both players; server-state mutation contract for `sang_meprise_active` and `ReconnectTracker.sang_meprise_sent_to`; client `ObjectiveIdentityCache` lifecycle; OQ-BR-01 closure (cache IS the audio-suppression signal). |

Pending ADRs needed: client-server authority model, card data schema, round state machine, auction event flow

## Engine Specialists

- **Primary**: `liv-bevy-018` skill (enforces Bevy 0.18 API patterns)
- **Networking**: `liv-bevy-lightyear` skill (all lightyear code)
- **Language/Code Specialist**: `gameplay-programmer` agent (Rust game logic)
- **Shader Specialist**: `technical-artist` agent (Bevy custom shaders if needed)
- **UI Specialist**: `ui-programmer` agent + `liv-bevy-018` skill
- **Additional Specialists**: `network-programmer` agent for protocol/sync design
- **Routing Notes**: Any `.rs` file importing `bevy` triggers `liv-bevy-018`. Any `.rs` file importing `lightyear` also triggers `liv-bevy-lightyear`.

### File Extension Routing

| File Extension / Type | Specialist to Spawn |
|-----------------------|---------------------|
| `*.rs` (game logic, ECS systems) | `gameplay-programmer` + `liv-bevy-018` |
| `*.rs` (lightyear protocol/networking) | `network-programmer` + `liv-bevy-lightyear` + `liv-bevy-018` |
| `*.rs` (UI systems, bevy_ui) | `ui-programmer` + `liv-bevy-018` |
| `*.wgsl` (custom shaders) | `technical-artist` |
| `*.ron` (config, card data) | `gameplay-programmer` |
| `*.json` (card pool data) | `gameplay-programmer` |
| `Trunk.toml` / `Cargo.toml` | `devops-engineer` |
| General architecture review | `lead-programmer` + `liv-bevy-018` |
