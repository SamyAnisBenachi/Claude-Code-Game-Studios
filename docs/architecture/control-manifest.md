# Control Manifest

> **Engine**: Bevy 0.18 + Lightyear 0.26
> **Last Updated**: 2026-04-29
> **Manifest Version**: 2026-04-29
> **ADRs Covered**: ADR-001, ADR-002, ADR-003, ADR-004, ADR-005, ADR-006, ADR-007, ADR-008, ADR-009, ADR-010, ADR-011, ADR-012
> **Status**: Active — regenerate with `/create-control-manifest` when ADRs change

This manifest is a programmer's quick-reference extracted from all Accepted ADRs,
technical preferences, and engine reference docs. For the reasoning behind each rule,
see the referenced ADR. Where an ADR says *why*, this manifest says *what*.

`Manifest Version` is the date this manifest was generated. Story files embed this
date. `/story-readiness` compares a story's embedded version to this field to detect
stories written against stale rules.

---

## Foundation Layer Rules

*Applies to: GameConfig, ServerRng, Network Protocol types, Lightyear plugin setup,
Cargo workspace structure*

### Required Patterns

- **Load `game_config.ron` via `bevy_asset_loader` at server startup; fatal on missing or malformed file — no fallback, no defaults at runtime.** — ADR-004
- **Abort startup if any dangerous `GameConfig` value is invalid:** `shop_weight_cap ∈ (0.0, 1.0)`, `shop_weight_per_card < shop_weight_cap`, `fake_count ∈ [1, 3]`, `objective_hp >= 1`, `placement_timer_seconds >= 1`. — ADR-004
- **Debug hot-reload of `GameConfig` must re-validate before applying. Reject invalid reload with warning; retain previous config. Never apply an invalid config.** — ADR-004
- **`CardCatalog` is immutable after load. Never mutate card definitions mid-session. Card data changes require server restart.** — ADR-004, ADR-006
- **`pool_copies_override ≤ 0` in `CardData` is a soft error: log warning, use rarity default, continue startup. Never abort.** — ADR-004
- **Epic and Legendary copy counts are compile-time constants (`EPIC_POOL_COPIES = 1`, `LEGENDARY_POOL_COPIES = 1`), never `GameConfig` fields.** — ADR-006
- **`GameConfig` struct lives in `shared/config.rs` without `#[derive(Resource)]`. Server wraps it:** `app.insert_resource(config)` **in** `server/foundation/config.rs`. — ADR-003
- **All game randomness uses a single per-session `ServerRng` resource backed by `ChaCha20Rng` from `rand_chacha 0.3`. Seeded once from `OsRng::from_entropy()` at session start. Never re-seed mid-session.** — ADR-005
- **RNG consumption order is strict and binding (corrupts audit if violated):**
  - DRAFT_INITIAL: (1) AssignFakeObjectives — 2 seeds/player, ascending `player_id`; (2) DrawInitialDraft — per player, ascending `player_id`
  - Each DRAFT_SHOP/AUCTION: (3) DrawShopSlot — 2–3 seeds/slot, ascending `player_id` then `slot_index`
  - RESOLUTION in order: (4) ResolveEcaflip — ascending `lane`; (5) ResolvePrism — ascending `player_id` then `lane`; (6) AwardFakeObjectiveReward — ascending `player_id` then `lane`; (7) DrawFreeCard — only if step 6 awarded free card
  — ADR-005
- **Three-crate Cargo workspace only: `shared/`, `server/`, `client/`. No other crate split.** — ADR-003
- **Within `server/`: dependency direction is `feature/ → core/ → foundation/` only. No reverse imports.** — ADR-003
- **Exactly two Lightyear channels: `ReliableChannel` (all game-state and control messages) and `UnreliableChannel` (heartbeat + auction timer only). Channel assignment is permanent per message type.** — ADR-008
- **All channel definitions live in `shared/src/protocol.rs`. Both server and client compile against identical channel types.** — ADR-008
- **`AssetLoader` impls must `#[derive(Default, TypePath)]` — required as of Bevy 0.18.** — ADR-004
- **Add `ron = "0.8"` as a direct dep in `server/Cargo.toml`. It is no longer re-exported from `bevy_asset`.** — ADR-003, ADR-004

### Forbidden Approaches

- **Never derive `Resource`, add plugin code, or use `#[cfg(feature = "server")]` branching in the `shared/` crate.** `shared/` must compile with `bevy = { default-features = false, features = ["serialize"] }` only. — ADR-003
- **Never transmit RNG seeds to clients in any S2C message.** Seeds are server-only. — ADR-005
- **Never use `rand::thread_rng()`, `StdRng`, or `SmallRng` in server game logic.** All game randomness goes through `ServerRng`. — ADR-005
- **Never use any RNG on the client for gameplay purposes.** Client crate has no `rand`/`rand_chacha` game logic dependency. — ADR-005
- **`client/` must never depend on `server/`. `server/` must never depend on `client/`.** Compiler enforces this. — ADR-003
- **`foundation/` within `server/` must never import from `core/` or `feature/`.** Code-review enforced. — ADR-003
- **Never put `rand` or `rand_chacha` in `client/Cargo.toml` for gameplay modules.** CI-gated. — ADR-003
- **Never send `S2CAuctionUpdate` (timer/price) or `C2SHeartbeat` on `ReliableChannel`.** These are the only two message types that belong on `UnreliableChannel`. — ADR-008

### Performance Guardrails

- **`GameConfig` + `CardCatalog` load time: < 100ms total at expected card count (~298 cards).** — ADR-004
- **WASM bundle size: ≤ 50 MB after `--release + LTO + strip`.** CI-gated. — ADR-003
- **`ServerRng` state: ~136 bytes. Audit log: < 32 KB per session. Zero network cost (never transmitted).** — ADR-005
- **O(1) `CardCatalog` lookup by `CardId` via `HashMap`.** — ADR-006

---

## Core Layer Rules

*Applies to: Round State Machine, Game Session System, Economy System, Card Data & Pool*

### Required Patterns

- **`RoundState` resource is the server's single source of truth for game phase. All systems read via `Res<RoundState>`.** — ADR-009
- **Only `advance_phase` (in `server/core/rsm/transitions.rs`) may hold `ResMut<RoundState>`. No other system writes phase.** Enforced by CI grep: `grep -r "ResMut<RoundState>" server/src/ | grep -v transitions.rs` must return zero results. — ADR-009
- **Use `MessageWriter::write()` to emit RSM phase messages. `EventWriter`/`EventReader` no longer exist in Bevy 0.17+.** — ADR-009, ADR-010
- **Use `MessageReader::read()` to consume RSM phase messages. Register with `app.add_message::<T>()`. Do NOT use `app.add_event::<T>()` for buffered messages.** — ADR-010
- **RSM emits all phase transitions as Bevy buffered Messages (`#[derive(Message)]`). `advance_phase` is the sole emitter. RSM has zero direct imports from `server/feature/`.** — ADR-010
- **Emission ordering on any DRAFT entry is strict (GDD F2):**
  1. `DraftStarted` (Economy reads — mana ramp + gold income)
  2. `ShopRefreshNeeded { player }` per player (Card Pool reads — draw shop slots)
  3. `AuctionPhaseEntered { round }` (if auction round — Auction System reads)
  4. `BroadcastPhaseChanged` **← always last** (clients notified only after server state is ready)
  — ADR-010
- **`BroadcastPhaseChanged` must always be the last event emitted in any phase transition arm.** — ADR-010
- **Phase-gate pattern is required in every C2S message handler: `if round_state.phase != expected_phase { return; }`. Invalid phase → silently discard, `debug!` log only, zero S2C response.** — ADR-009, ADR-002
- **`SessionReady` is delivered via Bevy Observer trigger (same-frame). GSS must insert `SessionConfig` and `ServerRng` via `Commands` BEFORE calling `Commands::trigger(SessionReady)`.** — ADR-012
- **GSS `check_lobby_ready` system must be scheduled `.before(advance_phase)` via `.chain()` in `RsmPlugin::build()`.** — ADR-012
- **`SessionConfig` is inserted once at `SessionReady` and never mutated. All Feature systems read it as `Res<SessionConfig>`.** — ADR-012
- **If `ServerRng::from_entropy()` fails, do NOT emit `SessionReady`. Transition to `LOBBY_CANCELLED` and broadcast `S2CSessionCancelled`.** — ADR-012
- **`SessionReady` fires at most once per session. Guard with `session_ready_fired: bool` flag.** — ADR-012
- **Future systems reacting to session start must subscribe to `DraftStarted` (emitted by RSM), not to `SessionReady` directly.** — ADR-012
- **`CardCatalog` is server-lifetime, immutable `Res<CardCatalog>`. `PlayerPool` is session-scoped per player, mutable, in `PlayerPools: HashMap<PlayerId, PlayerPool>`.** — ADR-006
- **`distribute()` is the sole pool mutation function. `copies_remaining` never goes below 0.** — ADR-006
- **All pool draw functions return `Option<T>`. Never panic on empty pool — return `None` and let caller handle it.** — ADR-006
- **`total_acquired(id)` is derived: `initial_count[id] - copies_remaining[id]`. No separate stored field.** — ADR-006
- **Timers tick only for the active phase. Reset the relevant timer immediately on phase entry before ticking.** — ADR-009
- **`round_number` increments BEFORE economy events fire on RESOLUTION → DRAFT transition.** — ADR-009

### Forbidden Approaches

- **Never use `#[derive(States)]` for `RoundPhase`.** Bevy States' `OnEnter`/`OnExit` schedules conflict with Lightyear's session lifecycle. — ADR-009
- **Never use buffered `Events<T>` for `SessionReady`.** Observer is required for same-frame resource visibility. — ADR-012
- **Never use `EventReader<SessionReady>` to consume session start.** Must use Observer (`app.observe(on_session_ready)`). — ADR-012

### Performance Guardrails

- **RSM tick budget: ≤ 5ms steady state; ≤ 15ms during RESOLUTION batch.** — ADR-009
- **Server tick budget: ≤ 5ms steady state total on single Railway dyno.** — ADR-002
- **`Res<RoundState>` phase check is O(1). Phase-gate pattern adds no measurable overhead.** — ADR-009

---

## Feature Layer Rules

*Applies to: Board/Lane System, Objective System (M1); Auction System, Combat Resolution, Card Acquisition, Keyword System, Prism System, Class System (M2/M3)*

### Required Patterns

- **During PLACEMENT, submitted cards are buffered in `PendingPlacements` resource (plain Rust data, NOT ECS entities).** — ADR-007
- **Unit ECS entities may ONLY be spawned AFTER `S2CPlacementReveal` is enqueued on `ReliableChannel`. This is a load-bearing invariant — violation leaks opponent placements.** — ADR-007
- **`S2CPlacementReveal` and entity spawning happen in the same system invocation, in this order: (1) enqueue `S2CPlacementReveal`, (2) spawn entities.** — ADR-007
- **Placement validation is all-or-nothing per player: if any card in the batch fails, silently discard the entire submission. No partial acceptance.** — ADR-007
- **Invalid placement submissions produce no S2C response to the client.** — ADR-007
- **Mana deduction happens at PLACEMENT close, not at submission receipt.** — ADR-007
- **`PendingPlacements` is fully cleared on entry to each new PLACEMENT phase.** — ADR-007
- **Spawn range validation (Formula F2): Minions only; Structures and Traps bypass range entirely. Process concurrent events: ascending `player_id` → ascending `lane_index` → ascending `cell`.** — ADR-007
- **`ObjectiveIdentity { is_fake: bool }` is held in server-only `HiddenObjectives` resource and never replicated as an ECS component.** — ADR-001
- **`ObjectiveHp { hp: u32 }` is a replicated ECS component, broadcast to both clients on every change.** — ADR-001
- **Send `S2CObjectiveIdentities` as reliable unicast per player at `DRAFT_INITIAL` after fake lane assignment.** — ADR-001
- **Re-send `S2CObjectiveIdentities` on every reconnect. Reliable delivery is not guaranteed across transport reconnects.** — ADR-001
- **`Sang Méprise` reveal: send one-shot reliable unicast `S2CSangMepriseReveal` to opponent only. Reveal persists in client local state for RESOLUTION duration only.** — ADR-001
- **`ObjectiveCounters { real_destroyed, fake_destroyed }` is a server-side Resource. RSM reads it at RESOLUTION end for GAME_OVER evaluation. RSM never imports from `feature/objective/`.** — Architecture Phase 4
- **[M2/M3 Feature systems] Every new Feature system that reacts to phase changes must subscribe to the relevant RSM event (e.g., `AuctionPhaseEntered`, `ResolutionPhaseEntered`). Never observe `RoundState` directly.** — ADR-010

### Forbidden Approaches

- **Never spawn ECS entity for a pending placement before `S2CPlacementReveal` is enqueued.** Violation leaks hidden placement data via Lightyear replication. — ADR-007
- **Never replicate `ObjectiveIdentity` as an ECS component. Never use per-component Lightyear visibility workarounds.** — ADR-001
- **Never send opponent `is_fake` values in any broadcast message.** Objective identity is owner-only. — ADR-001
- **Never let Feature systems call Core/Foundation systems directly.** Feature layer communicates upward via Events (emitting `ObjectiveDestroyed`, `AwardGold`, etc.). — Architecture Phase 4
- **Never let Feature systems import from `server/core/rsm/` directly.** Subscribe to RSM events only. — ADR-010

### Performance Guardrails

- **`PendingPlacements` validation: O(N) where N = cards in submission. Must complete within single frame.** — ADR-007
- **`S2CObjectiveIdentities` payload: ~6 bytes per player at 5 lanes + header. Zero bandwidth concern.** — ADR-001

---

## Presentation Layer Rules

*Applies to: Board Rendering, Hand UI, Shop/Auction UI, HUD, Card Animations (client/ crate) [M2+]*

### Required Patterns

- **All Presentation code lives in `client/` crate only. Zero game logic. Zero server state.** — ADR-002, ADR-003
- **Spawn sprites using Required Components pattern (Bevy 0.18): `Sprite::from_image(handle)` + `Transform`. Never use `SpriteBundle`.** — Engine reference `deprecated-apis.md`
- **Spawn UI using `Node { .. }` with inline `border_radius` field. Never use `NodeBundle`.** — Engine reference `deprecated-apis.md`
- **`LineHeight` is a required component for `Text`, `Text2d`, and `TextSpan` as of Bevy 0.18. Insert explicitly if non-default value needed.** — Engine reference `breaking-changes.md`
- **Use `ImageNode::new(handle)` not `UiImage::new(handle)` for UI images.** — Engine reference `deprecated-apis.md`
- **`despawn()` replaces `despawn_recursive()` as of Bevy 0.16. Use `despawn_related::<Children>()` for children-only despawn.** — Engine reference `deprecated-apis.md`
- **All Presentation reads go through client `ClientState` resources only. Never derive state from local simulation.** — ADR-002
- **`liv-bevy-018` skill is mandatory on every file in `client/ui/`.** — Architecture Phase 2

### Forbidden Approaches

- **Never use `SpriteBundle`, `Camera2dBundle`, `NodeBundle`, `TransformBundle`, `SpatialBundle`. All Bundles are deprecated as of Bevy 0.15.** — Engine reference `deprecated-apis.md`
- **Never use `UiImage` (use `ImageNode`), `UiImageSize` (use `ImageNodeSize`), `TextFont { line_height }` (use `LineHeight` component).** — Engine reference `deprecated-apis.md`
- **Never modify game state from the client crate. Client sends C2S inputs; server applies them.** — ADR-002
- **Never reflect with brackets or braces: `#[reflect[..]]` or `#[reflect{..}]`. Use parentheses only: `#[reflect(..)]`.** — Engine reference `breaking-changes.md`

---

## Global Rules (All Layers)

### Naming Conventions

| Element | Convention | Example |
|---|---|---|
| Structs / Enums / Components / Events / Plugins | `PascalCase` | `CardUnit`, `AuctionBidEvent`, `GamePlugin` |
| Functions / Systems / Variables / Fields | `snake_case` | `resolve_combat`, `current_gold` |
| Constants / Statics | `SCREAMING_SNAKE_CASE` | `OBJECTIVE_HP`, `MAX_HAND_SIZE` |
| Files / Modules | `snake_case.rs` | `auction_system.rs`, `card_pool.rs` |
| Plugins | suffix `Plugin` | `CombatPlugin`, `AuctionPlugin` |
| Systems | verb_noun | `spawn_unit`, `resolve_lane_combat`, `apply_interest` |
| Resources | noun, PascalCase | `GameConfig`, `CardPool`, `RoundState` |
| Lightyear C2S messages | prefix `C2S` | `C2SPlaceUnit`, `C2SAuctionBid` |
| Lightyear S2C messages | prefix `S2C` | `S2CRoundResolved`, `S2CPhaseChanged` |

### Performance Budgets

| Target | Value |
|---|---|
| Framerate | 60 FPS (browser/WASM) |
| Frame budget total | 16.67ms |
| Game logic budget | < 2ms |
| Render budget | < 12ms |
| Server tick budget (steady state) | ≤ 5ms |
| Network per round | < 1 KB |
| WASM bundle (release) | ≤ 50 MB |
| WASM heap | < 256 MB |
| S2C snapshot size | < 16 KB unicast (32 KB max if chunked) |

### Client-Server Authority (All Layers)

- **Server is sole authority over all game state. Client is a read-only view.** — ADR-002
- **No client-side prediction. No shared simulation. No optimistic UI updates.** — ADR-002
- **All game logic (phase transitions, combat, economy, RNG, validation) runs on the headless server binary only.** — ADR-002
- **On reconnect: server sends `S2CGameSnapshot` before any live messages. Client rebuilds state from snapshot.** — ADR-011
- **Snapshot secret-stripping rules (enforced server-side before unicast send):**
  - Own player: all fields populated (hand, shop_slots, mana, reserve, objectives with `is_fake`)
  - Opponent: hand = empty, shop_slots = empty — gold is visible (public by design)
  - Own objectives: `hp` + `is_fake` (from `HiddenObjectives`)
  - Opponent objectives: `hp` only — `is_fake` field absent entirely
  — ADR-011
- **Live messages to reconnecting player are queued server-side until `snapshot_sent[player] = true`.** Systems sending unicast S2C must check `ReconnectTracker.snapshot_sent[player]` before enqueuing. — ADR-011

### Forbidden Patterns (All Layers)

- **No client-side RNG for gameplay.** All randomness is server-side via `ServerRng`. — technical-preferences.md, ADR-005
- **No game state on client.** Clients are views. All authoritative state lives on the Lightyear server. — technical-preferences.md, ADR-002
- **No `unwrap()` in production paths.** Use `?` propagation or `expect("message")` with a diagnostic string. — technical-preferences.md
- **No `bevy_egui` in shipped build.** `egui` is dev/debug only. All shipped UI uses `bevy_ui`. — technical-preferences.md
- **No hardcoded balance values in systems.** Every tuning knob goes through `GameConfig` loaded from `assets/config/game_config.ron`. — technical-preferences.md
- **No `cfg(feature = "server")` for authority gating.** Server-only types live in the `server/` crate; compiler enforces the boundary. — ADR-002

### Approved Libraries

| Crate | Version | Purpose |
|---|---|---|
| `bevy` | 0.18 | Core engine |
| `lightyear` (`bevy_lightyear`) | 0.26 | Multiplayer networking |
| `bevy_tweening` | 0.18 | UI and movement animations |
| `bevy_asset_loader` | latest 0.18-compatible | Typed asset loading |
| `rand` + `rand_chacha` | `0.9` / `0.3` | Server-side seeded RNG |
| `serde` + `serde_json` | latest | Card data serialisation |
| `ron` | `0.8` | Config files (`GameConfig`) |
| `trunk` | latest | WASM build + dev server |
| `wasm-bindgen` | latest | WASM/JS boundary |

### Forbidden APIs (Bevy 0.14 → 0.18 — do not use)

The following APIs are deprecated or removed in Bevy 0.15–0.18. Using them produces a compile error on Bevy 0.18.

| Forbidden | Use Instead | Since |
|---|---|---|
| `SpriteBundle` | `Sprite::from_image(..)` + `Transform` | 0.15 |
| `Camera2dBundle` | `Camera2d` + `Transform` | 0.15 |
| `NodeBundle` | `Node { .. }` | 0.15 |
| `TransformBundle` | `Transform` alone | 0.15 |
| `SpatialBundle` | `Transform` + `Visibility` | 0.15 |
| Manual `GlobalTransform` insert | Don't — auto-inserted by `Transform` | 0.15 |
| `query.single()` panicking | `query.single()?` or `let Ok(x) = query.single()` | 0.16 |
| `EventWriter<T>` / `EventReader<T>` / `Events<T>` | `MessageWriter<T>` / `MessageReader<T>` + `app.add_message::<T>()` for buffered messages; `#[derive(Event)]` + `Observer` + `commands.trigger()` for one-shot triggers | 0.17 |
| `commands.entity(e).set_parent(p)` | `commands.entity(e).insert(ChildOf(p))` | 0.16 |
| `Parent` component | `ChildOf` component | 0.16 |
| `commands.entity(e).despawn_recursive()` | `commands.entity(e).despawn()` | 0.16 |
| `UiImage::new(handle)` | `ImageNode::new(handle)` | 0.16 |
| `TextFont { line_height: .. }` | `LineHeight` as separate required component | 0.18 |
| `BorderRadius` as separate component | `Node { border_radius: .. }` field | 0.18 |
| `entity.row()` | `entity.index()` | 0.18 |
| `ron` from `bevy_scene`/`bevy_asset` | Add `ron = "0.8"` directly to `Cargo.toml` | 0.18 |
| `AssetLoader` without `TypePath` | `#[derive(Default, TypePath)]` on loader struct | 0.18 |
| `#[reflect[..]]` or `#[reflect{..}]` | `#[reflect(..)]` parentheses only | 0.18 |
| `AnimationTarget { id, player }` | `AnimationTargetId` + `AnimatedBy` | 0.18 |

Source: `docs/engine-reference/bevy/deprecated-apis.md`

---

## Lightyear 0.26 Verification Checklist

**These items must be verified against `docs.rs/lightyear/0.26` before any networking code is written. Lightyear 0.26 is post-LLM-cutoff.**

| # | Item | ADR | Status |
|---|---|---|---|
| 1 | Channel definition syntax: plain structs + `app.add_channel::<T>(ChannelSettings { mode, send_frequency, priority })` — no `#[derive(Channel)]` macro | ADR-008 | ⚠️ DIFFERS |
| 2 | `ChannelMode` enum variants: `OrderedReliable(ReliableSettings)` ✅, `UnorderedUnreliable` ✅ (also: `UnorderedReliable`, `SequencedReliable`, `SequencedUnreliable`, `UnorderedUnreliableWithAcks`) | ADR-008 | ✅ CONFIRMED |
| 3 | Direction is on message registration, NOT channel: `app.register_message::<T>().add_direction(NetworkDirection::...)` — `NetworkDirection` enum: `ServerToClient`, `ClientToServer`, `Bidirectional` | ADR-008 | ⚠️ DIFFERS |
| 4 | `MessageSender<M>` and `MessageReceiver<M>` type names confirmed in prelude; both are **components** on entities (not standalone system params) | ADR-008 | ✅ CONFIRMED |
| 5 | Client send: `sender.send::<Channel>(message)` — channel via generic type, no target param, no `send_to_server()` method | ADR-008 | ⚠️ DIFFERS |
| 6 | Server receive: `receiver.receive() -> impl Iterator<Item = M>` — no `receive_messages()` method; also `receive_with_tick()`, `has_messages()`, `num_messages()` | ADR-008 | ⚠️ DIFFERS |
| 7 | `NetworkTarget` = `type alias Target<PeerId>`. Unicast: `NetworkTarget::Single(PeerId)` — identifier is `PeerId` not `ClientId` | ADR-001, ADR-008 | ⚠️ DIFFERS |
| 8 | `NetworkTarget::All` ✅ confirmed; also `AllExceptSingle(PeerId)`, `AllExcept(Vec<PeerId>)`, `Only(Vec<PeerId>)`, `None` | ADR-008 | ✅ CONFIRMED |
| 9 | Server send API: `ServerMultiMessageSender` system param — `send::<M, C>(&msg, &server, &NetworkTarget)` (generics: Message first, Channel second; not `send_message_to_target`) | ADR-001, ADR-011 | ⚠️ DIFFERS |
| 10 | `OrderedReliable` channel guarantees FIFO across all message types on the channel by definition; OQ-D invariant upheld by same-channel enqueue order | ADR-008 | ✅ CONFIRMED |
| 11 | No built-in snapshot guarantee — application-level concern: enqueue snapshot first in `Update` tick + `snapshot_sent` flag per ADR-011 design | ADR-011 | ✅ CONFIRMED |
| 12 | On reconnect, new `LinkOf` entity spawns with new `PeerId` (not `ClientId` — renamed); old entity despawned; `SessionToken` is cross-reconnect identity bridge | ADR-011 | ⚠️ DIFFERS |
| 13 | No `OnConnected` event — connection state uses marker components (`Connected`); detect via `Trigger<OnAdd, Connected>` observer on client entities | ADR-011 | ⚠️ DIFFERS |
| 14 | Pre-connect messages NOT delivered to new `PeerId`: confirmed by entity-per-connection model — new entity starts with empty message queue | ADR-011 | ✅ CONFIRMED |
| 15 | `Commands::trigger(SessionReady)` fires Observer in same `Update` frame — confirmed by `cargo test -p server session_ready_observer` from Developer PowerShell for VS 2026 | ADR-012 | ✅ CONFIRMED |
| 16 | `Res<T>` inserted via `Commands::insert_resource()` before `Commands::trigger()` is visible to Observer — confirmed by `cargo test -p server session_ready_observer` from Developer PowerShell for VS 2026 | ADR-012 | ✅ CONFIRMED |
| 17 | `Trigger<T>` is correct Observer parameter type in Bevy 0.18 — confirmed from Bevy 0.18 api_patterns | ADR-012 | ✅ CONFIRMED |
| 18 | Component replication is opt-in: entity must have `Replicate::default()` AND component must be registered via `app.register_component::<T>()` | ADR-007 | ✅ CONFIRMED |
| 19 | `ReplicationGroup` struct confirmed in prelude; `ReplicationGroup::new_id(id)` syntax confirmed | ADR-001 | ✅ CONFIRMED |
| 20 | `LocalTimeline` is a struct in `lightyear::core::prelude` ("local timeline matching Time<Virtual>"); accessible as `Res<LocalTimeline>` | Engine reference | ✅ CONFIRMED |

**Legend:** ✅ CONFIRMED — API exists as assumed | ⚠️ DIFFERS — API differs, resolution path documented in `tests/evidence/lightyear-026-verification.md`

Items 15 and 16 are resolved. Local Windows verification requires Developer PowerShell for VS 2026 because normal PowerShell does not load MSVC `link.exe`.

**Do not merge any networking story with unverified or unresolved DIFFERS items.**

---

## Skill Activation Rules

These skills are non-optional gates on specific file types:

| File type | Skill | Why |
|---|---|---|
| Any `.rs` importing `bevy` | `liv-bevy-018` | Enforces 0.18 API patterns; prevents deprecated Bundle/pre-0.15 patterns |
| Any `.rs` importing `lightyear` | `liv-bevy-lightyear` | Lightyear 0.26 API; verification patterns for post-cutoff networking |
| Both in same file | Activate **both** | Networking code uses both APIs simultaneously |
