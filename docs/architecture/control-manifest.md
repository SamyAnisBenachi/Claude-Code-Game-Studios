# Control Manifest

> **Engine**: Bevy 0.18 + Lightyear 0.26
> **Last Updated**: 2026-05-01
> **Manifest Version**: 2026-05-01
> **ADRs Covered**: ADR-001, ADR-002, ADR-003, ADR-004, ADR-005, ADR-006, ADR-007, ADR-008, ADR-009, ADR-010, ADR-011, ADR-012, ADR-017
> **ADRs Pending (Proposed — not yet covered)**: ADR-013, ADR-014, ADR-015, ADR-016, ADR-018
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
Cargo workspace structure, asset loading, session lifecycle*

### Required Patterns

**Cargo workspace**

- **Three-crate Cargo workspace only: `shared/`, `server/`, `client/`. No other crate split.** — ADR-003
- **`shared/` uses `bevy = { default-features = false, features = ["serialize"] }` only.** — ADR-003
- **`shared/` ban list: NO `#[derive(Resource)]`, NO Plugin impls, NO `App::add_systems`, NO Bevy queries.** — ADR-003
- **`GameConfig` struct lives in `shared/config.rs` WITHOUT `#[derive(Resource)]`. Server wraps it:** `app.insert_resource(config)` **in** `server/foundation/config.rs`. — ADR-003
- **Single `pub fn register_protocol(app: &mut App)` in `shared/src/protocol.rs`; called by BOTH `server/main.rs` and `client/main.rs` at startup.** — ADR-003
- **No `pub use` shortcuts in `shared/` — keep module paths explicit.** — ADR-003
- **Within `server/`: dependency direction is `feature/ → core/ → foundation/` only. No reverse imports.** — ADR-003
- **Within `client/`: `ui/ → state/ → network/ → shared/` only. No reverse imports.** — ADR-003
- **Release profile:** `lto = "thin"`, `codegen-units = 1`, `strip = "symbols"`, `panic = "abort"`. — ADR-003
- **Dev profile:** `opt-level = 1` (Bevy is unusable at 0). — ADR-003

**Asset loading**

- **Server uses `MinimalPlugins` (headless binary) — NEVER `DefaultPlugins`.** — ADR-004
- **App state sequence: `AppState::Loading` → `AppState::ConfigValidation` → `AppState::Lobby` → `AppState::InSession`.** — ADR-004
- **Load `game_config.ron` via `bevy_asset_loader` `LoadingState` at server startup; fatal on missing or malformed file — no fallback, no defaults at runtime.** — ADR-004
- **Abort startup if any dangerous `GameConfig` value is invalid:** `shop_weight_cap ∈ (0.0, 1.0)`, `shop_weight_per_card < shop_weight_cap`, `fake_count ∈ [1, 3]`, `objective_hp >= 1`, `placement_timer_seconds >= 1`. Call `app_exit.write(AppExit::Error(NonZeroU8::MIN))` — NEVER `panic!`. — ADR-004
- **`pool_copies_override ≤ 0` in `CardData` is a soft error: log `warn!`, use rarity default, continue startup. Never abort.** — ADR-004
- **Debug hot-reload of `GameConfig` must re-validate before applying. Reject invalid reload with warning; retain previous config.** Gate the `add_systems` call itself behind `#[cfg(debug_assertions)]`. — ADR-004
- **`CardCatalog` is immutable after load. Card data changes require server restart.** — ADR-004, ADR-006
- **Every `impl AssetLoader` struct must `#[derive(Default, TypePath)]` — required in Bevy 0.18.** — ADR-004
- **Add `ron = "0.8"` as a direct dep in `server/Cargo.toml`. It is no longer re-exported from `bevy_asset`.** — ADR-003, ADR-004
- **Test fixtures construct `GameConfig` directly via struct literal and insert via `world.insert_resource(cfg)` — no asset loader needed in tests.** — ADR-004

**Server-side RNG**

- **All game randomness uses a single per-session `ServerRng` resource backed by `ChaCha20Rng` from `rand_chacha 0.3`. Seeded once from `OsRng::from_entropy()` at session start. Never re-seed mid-session.** — ADR-005
- **`ServerRng` lives only in `server/src/foundation/rng.rs`. Expose intent-named methods only — never raw `next_u32`/`gen`.** — ADR-005
- **Every RNG draw MUST write an `AuditEntry` in the same call as the draw (never async, never best-effort).** — ADR-005
- **`ServerRng` inserted immediately before `commands.trigger(SessionReady)` (ADR-012); removed on `GameOverEmitted`.** — ADR-005
- **RNG consumption order is strict and binding (corrupts audit if violated):**
  - DRAFT_INITIAL: (1) `AssignFakeObjectives` — ascending `player_id`; (2) `DrawInitialDraft` — ascending `player_id`
  - Each DRAFT_SHOP/AUCTION: (3) `DrawShopSlot` — ascending `player_id` → ascending `slot_index`
  - RESOLUTION in order: (4) `ResolveEcaflip` — ascending `lane`; (5) `ResolvePrism` — ascending `player_id` → ascending `lane`; (6) `AwardFakeObjectiveReward` — ascending `player_id` → ascending `lane`; (7) `DrawFreeCard` — only if step 6 awarded free card
  — ADR-005

**Session lifecycle**

- **`SessionReady` is a Bevy Observer trigger (`#[derive(Event)]`). Registered via `app.observe(on_session_ready)`. NOT registered with `app.add_message::<SessionReady>()`.** — ADR-012
- **GSS Commands sequence (MUST be in this order in one Commands call):** `insert_resource(SessionConfig)` → `insert_resource(ServerRng)` → `trigger(SessionReady)`. — ADR-012
- **Only ONE Observer registered for `SessionReady`** — the RSM's LOBBY→DRAFT_INITIAL handler. Downstream systems react to `DraftStarted`, not to `SessionReady`. — ADR-012
- **`evaluate_session_ready` gated on `LobbyState::GameActive`** to prevent re-triggers after session start. — ADR-012
- **If `ServerRng::from_entropy()` fails, do NOT emit `SessionReady`. Transition to `LOBBY_CANCELLED`.** — ADR-012
- **`SessionConfig` is inserted once at `SessionReady` and never mutated. All Feature systems read it as `Res<SessionConfig>`.** — ADR-012

### Forbidden Approaches

- **Never** have `client/Cargo.toml` depend on `server` crate. Compiler enforces this. — ADR-002, ADR-003
- **Never** use `cfg(feature = "server")` in `protocol/` or `client/` to gate authority. — ADR-002, ADR-003
- **Never** put a server-only `Resource` (`HiddenObjectives`, `ServerRng`, etc.) into `protocol/` or `client/` types. — ADR-002
- **Never** add a dep to `shared/` without ADR amendment + technical-director approval. — ADR-003
- **Never** derive `Resource`, add plugin code, or add heavy Bevy deps in the `shared/` crate. — ADR-003
- **Never** put `rand_chacha` in `client/Cargo.toml`. — ADR-003, ADR-005
- **Never** put `rand` in `client/Cargo.toml` for gameplay purposes. — ADR-003
- **Never** use `rand::thread_rng()`, `StdRng`, or `SmallRng` in server game logic. — ADR-005
- **Never** transmit RNG seeds, `seed_index`, or `audit_log` in any production S2C message. — ADR-005
- **Never** re-seed `ServerRng` mid-session. — ADR-005
- **Never** use `std::fs::read_to_string` in production server paths for config. — ADR-004
- **Never** use `include_bytes!` for config/balance data (violates data-driven standard). — ADR-004
- **Never** read `SessionReady` via `MessageReader<SessionReady>` — it fires as an Observer. — ADR-012
- **Never** register a second Observer for `SessionReady`. — ADR-012
- **Never** call `commands.trigger(SessionReady)` before both `insert_resource` calls in the same Commands queue. — ADR-012

### Performance Guardrails

- **`GameConfig` + `CardCatalog` load time: < 100ms total at expected card count (~298 cards).** — ADR-004
- **WASM bundle size: ≤ 50 MB after `--release + LTO + strip`.** CI-gated. — ADR-003
- **`cargo check -p client` incremental: < 5s.** — ADR-003
- **`cargo tree -p shared` must NOT contain `bevy_ecs`, `bevy_render`, `bevy_ui`, `tokio`, or server-only Lightyear features.** CI-gated. — ADR-003
- **`ServerRng` state: ~136 bytes. Audit log: < 32 KB per session. Zero network cost.** — ADR-005

---

## Core Layer Rules

*Applies to: Round State Machine, event bus, reconnect protocol, card data & pool, authority model, Lightyear channel config*

### Required Patterns

**Client-server authority**

- **Server is sole authority over all game state. Client is a read-only view that emits C2S input intents.** — ADR-002
- **All `ClientState` mutation flows through `apply_s2c_to_client_state` — user input NEVER directly mutates `ClientState`.** — ADR-002
- **Every C2S handler: (1) validate phase, (2) validate sender identity, (3) validate domain rules, (4) apply, (5) broadcast S2C. On any failure: `tracing::debug!` log + silent return. Zero S2C response on reject.** — ADR-002
- **Snapshot-driven reconnect: on `OnConnected`, unicast `S2CGameSnapshot` before any other S2C; client treats it as full reset.** — ADR-002
- **Server tick is the wall clock. Client `timer_remaining_ms` is presentation only — never feeds back to server.** — ADR-002

**Lightyear channel configuration**

- **Exactly two Lightyear channels: `ReliableChannel` (all game-state and control messages) and `UnreliableChannel` (heartbeat + auction timer only). Channel assignment is permanent per message type.** — ADR-008
- **`C2SHeartbeat` and `S2CAuctionUpdate` → `UnreliableChannel`. All other messages → `ReliableChannel`.** — ADR-008
- **OQ-D invariant: `S2CResolutionEvent` MUST be enqueued before `S2CPhaseChanged` on `ReliableChannel`. Enforce via Bevy system ordering `.before()`.** — ADR-008
- **All channel definitions live in `shared/src/protocol.rs`.** — ADR-008
- **`snapshot_sent` gate: on `OnConnected`, set `snapshot_sent[player] = false`. Snapshot system sends `S2CGameSnapshot` and sets it to `true`. Every unicast S2C system MUST check `snapshot_sent[player]` before enqueuing; if false → push to `deferred_queue[player]` instead.** — ADR-008, ADR-011
- **Broadcast messages skip the `snapshot_sent` check** (reconnecting player isn't connected yet; snapshot covers their state). — ADR-011
- **Verify before implementing any networking story:** `NetworkTarget` unicast variant, channel definition syntax, server receive API shape — against `docs.rs/lightyear/0.26`. See Lightyear Verification Checklist below. — ADR-008

**Round State Machine (RSM) phase state**

- **`RoundState` resource is the server's single source of truth for game phase. All systems read via `Res<RoundState>`.** — ADR-009
- **Only `advance_phase` (in `server/core/rsm/transitions.rs`) may hold `ResMut<RoundState>`. No other system writes phase.** — ADR-009
- **Use `MessageWriter::write()` to emit RSM phase messages. Use `MessageReader::read()` to consume them. Register with `app.add_message::<T>()`. `EventWriter`/`EventReader` no longer exist in Bevy 0.17+.** — ADR-009, ADR-010
- **Phase-gate pattern is required in every C2S message handler: `if round_state.phase != expected_phase { return; }`. Invalid phase → silent discard, `debug!` log only, zero S2C response.** — ADR-009, ADR-002
- **System schedule order: `AuctionSystem` → `CombatResolutionSystem` → `rsm_tick_system` → `MessageSendSystems`.** — ADR-009
- **`ClientPhaseView` resource on client — updated ONLY from `S2CPhaseChanged` messages, never drives transitions.** — ADR-009
- **`AuctionSettled` and `ResolutionComplete` use `#[derive(Message)]` — NOT `#[derive(Event)]`.** — ADR-009
- **`SessionReady` uses `#[derive(Event)]` + Observer — NOT a buffered Message.** — ADR-009, ADR-012

**RSM event bus**

- **RSM has zero direct imports from `server/feature/`. All phase-reactive logic triggered by event.** — ADR-010
- **`advance_phase` never calls feature module functions directly.** — ADR-010
- **Emission ordering on any DRAFT entry (STRICT — linear code order in `advance_phase` match arm, NOT Bevy system order):**
  1. `DraftStarted` (Economy: mana ramp + gold income)
  2. `ShopRefreshTriggered { player_id, trigger }` per player (replaces deprecated `ShopRefreshNeeded`)
  3. `AuctionPhaseEntered { round }` (DRAFT_AUCTION rounds only)
  4. `BroadcastPhaseChanged` ← **ALWAYS LAST — clients notified only after server state is ready**
  — ADR-010
- **PLACEMENT entry: `PlacementPhaseEntered` → `BroadcastPhaseChanged`.** — ADR-010
- **RESOLUTION entry: `ResolutionPhaseEntered` → `BroadcastPhaseChanged`.** — ADR-010
- **GAME_OVER entry: `GameOverEmitted` → `BroadcastPhaseChanged`.** — ADR-010
- **`BroadcastPhaseChanged` must ALWAYS be the last event emitted in any phase transition arm.** — ADR-010
- **Subscriber systems must be scheduled `.after(advance_phase)` to see current-frame messages.** — ADR-010
- **Guard pattern for inbound RSM messages: validate `phase == expected_phase` before acting; stale → silent discard.** — ADR-010
- **`ShopRefreshTriggered` (not `ShopRefreshNeeded`) is the canonical shop draw trigger.** Do NOT implement `ShopRefreshNeeded`. — ADR-010
- **`AbortAuction` is in the event catalog for auction cleanup when GAME_OVER fires during DRAFT_AUCTION.** — ADR-010
- **New phase-reactive systems MUST add their subscriber contract to ADR-010 before story opens.** — ADR-010

**Reconnect protocol**

- **`SessionToken = [u8; 16]` is the sole identity bridge across Lightyear transport reconnects (new `ClientId`/`PeerId` on every WebSocket connect).** — ADR-011
- **`C2SHello` must be first message on any connection; hello timeout: 5000ms then close silently.** — ADR-011
- **Mandatory reconnect send order (all `ReliableChannel`, all unicast):**
  1. `S2CHandshake` (same token value)
  2. `S2CGameSnapshot` (full state, secrets stripped per player)
  3. `S2CObjectiveIdentities` (must explicitly re-send — NOT auto-replicated across transport reconnect)
  4. `S2CPhaseChanged` (with live `timer_remaining_ms`, not the original phase duration)
- **After step 4: set `snapshot_sent[player] = true`; flush `deferred_queue[player]` in enqueue order.** — ADR-011
- **Reconnect snapshot system scheduled BEFORE all live-game message systems in `Update`.** — ADR-011
- **Secret stripping rules (enforced server-side before unicast send):**
  - Own player: all fields populated (hand, shop_slots, mana, reserve, objectives with `is_fake`)
  - Opponent: `hand` = empty, `shop_slots` = empty; gold is visible (public by design)
  - Own objectives: `hp` + `is_fake` from `HiddenObjectives`
  - Opponent objectives: `hp` only — `is_fake` absent entirely
  - Own trap: `card_id = Some(card_id)`; Opponent trap: `card_id = None`
  — ADR-011
- **`S2CGameSnapshot::for_player(player_id)` constructor handles stripping. Add a unit test asserting no player-B secrets appear in player-A's snapshot.** — ADR-011
- **Session cleanup: remove `ReconnectTracker.token_map` entries on `C2SAcknowledgeResult` or `ack_timeout_ms` expiry.** — ADR-011

**Card data and pool**

- **`CardId` is a newtype `pub struct CardId(pub u32)` — no raw integer arithmetic on IDs.** — ADR-006
- **`CardCatalog = HashMap<CardId, CardData>` — immutable for server lifetime; never mutated after initial load.** — ADR-006
- **`PlayerPool` is session-scoped per player; `distribute()` is the SOLE pool mutation. `copies_remaining` never goes below 0.** — ADR-006
- **All pool draw functions return `Option<T>`. Never panic on empty pool — return `None` and let caller handle it.** — ADR-006
- **`total_acquired(id)` is derived: `initial_count[id] - copies_remaining[id]`. No separate stored field.** — ADR-006
- **`EPIC_POOL_COPIES = 1` and `LEGENDARY_POOL_COPIES = 1` are Rust `const` — NOT `GameConfig` fields.** — ADR-006
- **Pool draw functions accept explicit seeds from `ServerRng` — pool owns NO randomness source.** — ADR-006
- **`FamilyIndex: HashMap<String, Vec<CardId>>` is server-only derived structure — NOT in `shared/`.** — ADR-006

### Forbidden Approaches

- **Never** use `#[derive(States)]` for `RoundPhase`. Bevy States' `OnEnter`/`OnExit` schedules conflict with Lightyear's session lifecycle. — ADR-009
- **Never** store phase state in ECS components on entities. — ADR-009
- **Never** replicate `RoundPhase` as a Lightyear component (breaks OQ-D ordering invariant). — ADR-009
- **`ResMut<RoundState>` must appear in exactly one system (`advance_phase`/`rsm_tick_system`).** Code-review enforced. — ADR-009
- **`EventWriter<T>` / `EventReader<T>` / `Events<T>` do not exist in Bevy 0.17+. Never use them.** — ADR-009, ADR-010
- **Never** call feature module functions directly from `advance_phase`. — ADR-010
- **Never** use Observer for recurring RSM phase messages (`SessionReady` is the sole Observer exception). — ADR-010
- **Never** split `S2CResolutionEvent` and `S2CPhaseChanged` onto different channels. — ADR-008
- **Never** enqueue live unicast S2C to a reconnecting client before `snapshot_sent[player] = true`. — ADR-008, ADR-011
- **Never** broadcast `S2CGameSnapshot` — always unicast per player with secrets stripped. — ADR-011
- **Never** use delta-replay (resend all missed messages) for reconnect — full snapshot only. — ADR-011
- **Never** use `EventReader<SessionReady>` to consume session start. Must use Observer. — ADR-012
- **Never** allow optimistic client updates — `ClientState` mutates only on inbound S2C. — ADR-002
- **Never** put client-side RNG for gameplay in the `client/` crate. — ADR-002, ADR-005
- **Never** mutate `CardCatalog` after initial load. — ADR-006
- **Never** use a shared global pool (TFT model) — each player's pool is independent. — ADR-006
- **Never** use ECS components for pool state — use `Resource`-based `HashMap`. — ADR-006
- **Never** send `S2CAuctionUpdate` or `C2SHeartbeat` on `ReliableChannel`. These are the only two `UnreliableChannel` message types. — ADR-008

### Performance Guardrails

- **Server tick budget: ≤ 5ms steady state; ≤ 15ms during RESOLUTION batch.** — ADR-002, ADR-009
- **Client S2C processing + view update: ≤ 2ms per frame.** — ADR-002
- **Network: < 1 KB per round per player including replication deltas.** — ADR-002
- **Reconnect snapshot: target < 4 KB; hard limit < 16 KB unicast.** — ADR-002, ADR-011
- **`snapshot_sent` check: O(1) HashMap lookup, ~5ns.** — ADR-011

---

## Feature Layer Rules

*Applies to: Board/Lane System, Objective System (M1); Auction System, Combat Resolution, Card Acquisition, Keyword System, Prism System, Class System (M2/M3)*

### Required Patterns

**Placement buffer (ADR-007)**

- **During PLACEMENT, submitted cards are buffered in `PendingPlacements` resource (plain Rust data, NOT ECS entities).** — ADR-007
- **`close_placement_phase` MUST execute in this exact order:**
  1. Build `S2CPlacementReveal` payload from `PendingPlacements`
  2. Enqueue `S2CPlacementReveal` broadcast on `ReliableChannel` ← **THIS MUST BE FIRST**
  3. `commands.spawn(...)` for placed units ← ONLY AFTER broadcast enqueued
  4. Add spawned entities to Lightyear replication group
  5. Emit `PlacementCommitted`
  6. `PendingPlacements.submissions.clear()`
  — ADR-007
- **Mana deduction happens at PLACEMENT close, NOT at submission receipt.** — ADR-007
- **`is_final: bool` on `PlayerSubmission`: set `true` after first accepted submission; subsequent submissions → silent discard.** — ADR-007
- **`PendingPlacements` fully cleared on `PlacementPhaseEntered` (not on exit).** — ADR-007
- **Placement validation is all-or-nothing per player: if any card fails any check, silently discard the entire submission. No partial acceptance.** — ADR-007
- **Invalid placement submissions produce no S2C response to the client.** — ADR-007
- **`validate_spawn_range` (Formula F2): Minions only; Structures and Traps bypass range. Process concurrent events: ascending `player_id` → ascending `lane_index` → ascending `cell`.** — ADR-007
- **`liv-bevy-018` and `liv-bevy-lightyear` skills mandatory on all files in `server/src/feature/board/`.** — ADR-007

**Objective identity (ADR-001)**

- **`ObjectiveHp { hp: u32 }` is a replicated ECS component, broadcast to both clients on every change.** — ADR-001
- **`ObjectiveIdentity { is_fake: bool }` is held in server-only `HiddenObjectives` resource and NEVER replicated as an ECS component.** — ADR-001
- **Send `S2CObjectiveIdentities` as reliable unicast per player at `DRAFT_INITIAL` after fake lane assignment.** — ADR-001
- **Re-send `S2CObjectiveIdentities` on every reconnect (step 3 of mandatory reconnect sequence). Reliable delivery does not persist across transport reconnects.** — ADR-001, ADR-011
- **`Sang Méprise` reveal: send one-shot reliable unicast `S2CSangMepriseReveal` to opponent only. Reveal persists in client local state for RESOLUTION duration only.** — ADR-001
- **Client caches `S2CObjectiveIdentities` in a local resource — NOT an ECS component.** — ADR-001
- **`ObjectiveCounters { real_destroyed, fake_destroyed }` is a server-side Resource. RSM reads it at RESOLUTION end for GAME_OVER evaluation.** — Architecture
- **`[M2/M3]` Every new Feature system reacting to phase changes must subscribe to the relevant RSM event. Never observe `RoundState` directly.** — ADR-010

**Combat Resolution (ADR-017)**

- **`resolve_combat` is a Bevy exclusive system declared as `pub fn resolve_combat(world: &mut World)` and registered via `add_systems(Update, resolve_combat)`. Bevy 0.18 auto-detects `&mut World` as exclusive — no annotation needed.** — ADR-017
- **On every invocation: read `MessageReader<BeginResolution>`. If no message is present, return immediately — zero ECS mutations, zero S2C sends.** — ADR-017
- **S2CPlacementReveal MUST be enqueued via Lightyear BEFORE any sub-step 1 entity spawn or ECS mutation executes.** — ADR-017
- **After all 6 sub-steps complete: enqueue `S2CResolutionEvent` via Lightyear FIRST, then write `ResolutionComplete` via `MessageWriter` SECOND. This ordering enforces the OQ-D delivery invariant (clients receive resolution log before phase change).** — ADR-017, ADR-008
- **Internal iteration budget: maintain a monotonically increasing counter across all sub-step loops. If it exceeds 10,000, abort — emit `GameOver { loser: None, reason: Draw }` to the RSM event bus — and do NOT write `ResolutionComplete`.** — ADR-017
- **`apply_combat_modifier_stack` is a pure function with signature `fn(attacker: &UnitSnapshot, defender: &UnitSnapshot) -> CombatResult`. No World access, no ECS queries. All BLOCKING modifier-stack CRs (CR-12 through CR-15, CR-42, CR-43) are unit-testable without any Bevy context.** — ADR-017
- **API disambiguation: `BeginResolution` and `ResolutionComplete` use Bevy-internal `MessageWriter<T>`/`MessageReader<T>` (registered via `app.add_message::<T>()`). `S2CResolutionEvent` uses Lightyear `MessageSender<T>` (registered via Lightyear protocol plugin). Never confuse the two.** — ADR-017, ADR-008
- **System schedule order: `resolve_combat` runs AFTER `placement_buffer_close_system` and AFTER the frame that enqueues `BeginResolution`, so `MessageReader<BeginResolution>` is populated when `resolve_combat` reads it.** — ADR-017, ADR-009

### Forbidden Approaches

- **Never** call RSM functions directly from `resolve_combat` — communicate exclusively via `ResolutionComplete` (MessageWriter). — ADR-017
- **Never** use streaming per-sub-step S2C delivery — `S2CResolutionEvent` is always a single batch sent after all 6 sub-steps complete. — ADR-017
- **Never** write `ResolutionComplete` before `S2CResolutionEvent` is enqueued. — ADR-017
- **Never** use `EventWriter<T>`/`EventReader<T>` inside `resolve_combat` — removed in Bevy 0.17+. — ADR-017, ADR-009
- **Never** spawn ECS entity for a pending placement before `S2CPlacementReveal` is enqueued. Violation leaks hidden placement data via Lightyear replication. — ADR-007
- **Never** split `S2CPlacementReveal` broadcast and entity spawn across two Bevy systems without an explicit `.before()` ordering constraint. — ADR-007
- **Never** use `PlacementHidden` flag component workaround (silent-failure risk on visibility change). — ADR-007
- **Never** rely on network timing to enforce simultaneous reveal ordering — must be structural. — ADR-007
- **Never** replicate `ObjectiveIdentity` as an ECS component. — ADR-001
- **Never** send opponent `is_fake` values in any broadcast message. — ADR-001
- **Never** let Feature systems call Core/Foundation systems directly — communicate upward via events. — ADR-010

### Performance Guardrails

- **`resolve_combat` idle exit (no `BeginResolution`): < 1ms.** — ADR-017
- **`resolve_combat` full RESOLUTION batch (worst-case 5-lane contested round with keywords): ≤ 15ms.** — ADR-017, ADR-002
- **`ResolutionLog` heap allocation: ~10 KB per resolution (100 events × 100 bytes). Reset each resolution — no accumulation across rounds.** — ADR-017
- **`PendingPlacements` per-frame: < 0.1ms (at most 2 submissions per phase).** — ADR-007
- **`close_placement_phase`: < 0.5ms (runs once per PLACEMENT phase, ~once per 10s).** — ADR-007
- **`S2CObjectiveIdentities` payload: ~6 bytes per player at 5 lanes + header. Zero bandwidth concern.** — ADR-001

---

## Presentation Layer Rules

*Applies to: Board Rendering, Hand UI, Shop/Auction UI, HUD, Card Animations (client/ crate) [M2+]*

### Required Patterns

- **All Presentation code lives in `client/` crate only. Zero game logic. Zero server state.** — ADR-002, ADR-003
- **Spawn sprites using Required Components pattern (Bevy 0.18): `Sprite::from_image(handle)` + `Transform`. Never use `SpriteBundle`.** — deprecated-apis.md
- **Spawn UI using `Node { .. }` with inline `border_radius` field. Never use `NodeBundle`.** — deprecated-apis.md
- **Spawn camera: `commands.spawn((Camera2d, Transform::from_xyz(0., 0., 999.)))`. Never use `Camera2dBundle`.** — deprecated-apis.md
- **`LineHeight` is a required component for `Text` in Bevy 0.18. Insert explicitly if non-default value needed.** — deprecated-apis.md
- **Use `ImageNode::new(handle)` — NOT `UiImage::new(handle)` — for UI images.** — deprecated-apis.md
- **`despawn()` replaces `despawn_recursive()` since Bevy 0.16.** — deprecated-apis.md
- **`query.single()` returns `Result` since 0.16. Use `query.single()?` or `let Ok(x) = query.single()`.** — deprecated-apis.md
- **All Presentation reads go through `ClientState` resources only. Never derive state from local simulation.** — ADR-002
- **Reactive keyword triggers (APPEARANCE, DEATH, FINAL BLOW): use `#[derive(Event)]` + `commands.entity(unit).observe(...)`.** — current-best-practices.md
- **`liv-bevy-018` skill is mandatory on every file in `client/ui/`.** — Architecture

### Forbidden Approaches

- **Never** use `SpriteBundle`, `Camera2dBundle`, `NodeBundle`, `TransformBundle`, `SpatialBundle`. All Bundles deprecated since Bevy 0.15. — deprecated-apis.md
- **Never** use `UiImage` (use `ImageNode`), `UiImageSize` (use `ImageNodeSize`), `TextFont { line_height }` (use `LineHeight` required component). — deprecated-apis.md
- **Never** use `commands.entity(e).set_parent(p)` — use `commands.entity(e).insert(ChildOf(p))`. — deprecated-apis.md
- **Never** modify game state from the client crate. Client sends C2S inputs; server applies them. — ADR-002
- **Never** reflect with brackets or braces: `#[reflect[..]]` / `#[reflect{..}]`. Use parentheses only: `#[reflect(..)]`. — deprecated-apis.md
- **Never** use `bevy_egui` in shipped build. All shipped UI uses `bevy_ui` only. — technical-preferences.md

---

## Global Rules (All Layers)

### Naming Conventions

| Element | Convention | Example |
|---------|-----------|---------|
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

| Target | Value | Source |
|--------|-------|--------|
| Framerate | 60 FPS (browser/WASM) | technical-preferences.md |
| Frame budget total | 16.67ms | technical-preferences.md |
| Game logic budget | < 2ms per frame | technical-preferences.md |
| Render budget | < 12ms per frame | technical-preferences.md |
| Server steady state | ≤ 5ms per tick | ADR-002 |
| Server RESOLUTION batch | ≤ 15ms | ADR-002 |
| WASM bundle (release + LTO + strip) | ≤ 50 MB | ADR-003 |
| WASM heap | < 256 MB | technical-preferences.md |
| Network per round | < 1 KB per round per player | technical-preferences.md |
| Reconnect snapshot | < 16 KB unicast | ADR-002, ADR-011 |

### Cross-Cutting Constraints

These apply everywhere, regardless of layer:

1. **No `unwrap()` in production paths** — use `?` propagation or `expect("descriptive message")`. — technical-preferences.md
2. **All balance values through `GameConfig`** loaded from `assets/config/game_config.ron` — never hardcode tuning numbers in systems. — technical-preferences.md
3. **No `bevy_egui` in shipped build** — all shipped UI uses `bevy_ui` only. — technical-preferences.md
4. **`EventWriter`/`EventReader` are gone in Bevy 0.17+**: use `MessageWriter<T>`/`MessageReader<T>` + `app.add_message::<T>()` for buffered game-loop signals; use `#[derive(Event)]` + `app.observe(..)` for one-shot reactive lifecycle triggers. — ADR-009, ADR-010
5. **Every unicast S2C system checks `snapshot_sent[player]`** before enqueuing; if `false`, push to `deferred_queue[player]` instead. — ADR-008, ADR-011
6. **Bevy `MessageWriter<T>` vs Lightyear `MessageSender<T>`** — these are from different crates and must not be confused. Bevy's `MessageWriter`/`MessageReader` is for server-internal bus messages registered via `app.add_message::<T>()`. Lightyear's `MessageSender`/`MessageReceiver` is for network C2S/S2C messages registered via Lightyear's `ProtocolPlugin`. — ADR-008, ADR-009
7. **`NetworkTarget` unicast variant must be verified** against `docs.rs/lightyear/0.26.x` before any unicast implementation. See Lightyear Verification Checklist item 7. — ADR-001, ADR-008

### Approved Libraries

| Crate | Version | Purpose |
|-------|---------|---------|
| `bevy` | 0.18 | Core engine |
| `lightyear` (`bevy_lightyear`) | 0.26 | Multiplayer networking |
| `bevy_tweening` | 0.18 | UI and movement animations |
| `bevy_asset_loader` | latest 0.18-compatible | Typed asset loading |
| `rand` + `rand_chacha` | `0.9` / `0.3` | Server-side seeded RNG (server crate only) |
| `serde` + `serde_json` | latest | Card data serialisation |
| `ron` | `0.8` | Config files (`GameConfig`) |
| `trunk` | latest | WASM build + dev server |
| `wasm-bindgen` | latest | WASM/JS boundary |

### Forbidden APIs — Bevy 0.14 → 0.18

The following APIs are deprecated or removed in Bevy 0.15–0.18. Using them produces a compile error on Bevy 0.18.

| Forbidden | Use instead | Since |
|-----------|-------------|-------|
| `SpriteBundle` | `Sprite::from_image(..)` + `Transform` | 0.15 |
| `Camera2dBundle` | `Camera2d` + `Transform` | 0.15 |
| `NodeBundle` | `Node { .. }` | 0.15 |
| `TransformBundle` | `Transform` alone | 0.15 |
| `SpatialBundle` | `Transform` + `Visibility` | 0.15 |
| Manual `GlobalTransform` insert | Auto-inserted by `Transform` — do NOT add it | 0.15 |
| `query.single()` panicking form | `query.single()?` or `let Ok(x) = query.single()` | 0.16 |
| `EventWriter<T>` / `EventReader<T>` / `Events<T>` | `MessageWriter<T>` / `MessageReader<T>` + `app.add_message::<T>()` for buffered; `#[derive(Event)]` + `Observer` for one-shot | 0.17 |
| `app.add_event::<T>()` | `app.add_message::<T>()` for buffered; `app.observe(..)` for reactive | 0.17 |
| `commands.entity(e).set_parent(p)` | `commands.entity(e).insert(ChildOf(p))` | 0.16 |
| `Parent` component | `ChildOf` component | 0.16 |
| `commands.entity(e).despawn_recursive()` | `commands.entity(e).despawn()` | 0.16 |
| `commands.entity(e).despawn_descendants()` | `commands.entity(e).despawn_related::<Children>()` | 0.16 |
| `UiImage::new(handle)` | `ImageNode::new(handle)` | 0.16 |
| `UiImageSize` | `ImageNodeSize` | 0.16 |
| `TextFont { line_height: .. }` | `LineHeight` as separate required component | 0.18 |
| `BorderRadius` as separate component | `Node { border_radius: .. }` field | 0.18 |
| `entity.row()` | `entity.index()` | 0.18 |
| `ron` from `bevy_scene`/`bevy_asset` | Add `ron = "0.8"` directly to `Cargo.toml` | 0.18 |
| `AssetLoader` without `TypePath` | `#[derive(Default, TypePath)]` on loader struct | 0.18 |
| `#[reflect[..]]` or `#[reflect{..}]` | `#[reflect(..)]` parentheses only | 0.18 |
| `AnimationTarget { id, player }` | `AnimationTargetId` + `AnimatedBy` components | 0.18 |
| `FontAtlasSets` | Removed — font atlasing is internal | 0.18 |
| Cargo feature `animation` | `gltf_animation` | 0.18 |
| Cargo feature `bevy_sprite_picking_backend` | `sprite_picking` | 0.18 |
| Cargo feature `bevy_ui_picking_backend` | `ui_picking` | 0.18 |

Source: `docs/engine-reference/bevy/deprecated-apis.md`

---

## Lightyear 0.26 Verification Checklist

**These items must be verified against `docs.rs/lightyear/0.26` before any networking code is written. Lightyear 0.26 is post-LLM-cutoff.**

| # | Item | ADR | Status |
|---|---|---|---|
| 1 | Channel definition syntax: plain structs + `app.add_channel::<T>(ChannelSettings { mode, send_frequency, priority })` — no `#[derive(Channel)]` macro | ADR-008 | ⚠️ DIFFERS |
| 2 | `ChannelMode` enum variants: `OrderedReliable(ReliableSettings)` ✅, `UnorderedUnreliable` ✅ | ADR-008 | ✅ CONFIRMED |
| 3 | Direction is on message registration, NOT channel: `app.register_message::<T>().add_direction(NetworkDirection::...)` | ADR-008 | ⚠️ DIFFERS |
| 4 | `MessageSender<M>` and `MessageReceiver<M>` type names confirmed in prelude; both are **components** on entities (not standalone system params) | ADR-008 | ✅ CONFIRMED |
| 5 | Client send: `sender.send::<Channel>(message)` — channel via generic type, no `send_to_server()` method | ADR-008 | ⚠️ DIFFERS |
| 6 | Server receive: `receiver.receive() -> impl Iterator<Item = M>` — no `receive_messages()` method | ADR-008 | ⚠️ DIFFERS |
| 7 | `NetworkTarget` = `type alias Target<PeerId>`. Unicast: `NetworkTarget::Single(PeerId)` — identifier is `PeerId` not `ClientId` | ADR-001, ADR-008 | ⚠️ DIFFERS |
| 8 | `NetworkTarget::All` ✅; also `AllExceptSingle(PeerId)`, `AllExcept(Vec<PeerId>)`, `Only(Vec<PeerId>)`, `None` | ADR-008 | ✅ CONFIRMED |
| 9 | Server send API: `ServerMultiMessageSender` system param — `send::<M, C>(&msg, &server, &NetworkTarget)` (generics: Message first, Channel second) | ADR-001, ADR-011 | ⚠️ DIFFERS |
| 10 | `OrderedReliable` channel guarantees FIFO across all message types — OQ-D invariant upheld by same-channel enqueue order | ADR-008 | ✅ CONFIRMED |
| 11 | No built-in snapshot guarantee — application-level concern: enqueue snapshot first in `Update` tick + `snapshot_sent` flag per ADR-011 | ADR-011 | ✅ CONFIRMED |
| 12 | On reconnect, new `LinkOf` entity spawns with new `PeerId`; `SessionToken` is cross-reconnect identity bridge | ADR-011 | ⚠️ DIFFERS |
| 13 | No `OnConnected` event — connection state uses marker components (`Connected`); detect via `Trigger<OnAdd, Connected>` observer | ADR-011 | ⚠️ DIFFERS |
| 14 | Pre-connect messages NOT delivered to new `PeerId`: confirmed by entity-per-connection model | ADR-011 | ✅ CONFIRMED |
| 15 | `Commands::trigger(SessionReady)` fires Observer in same `Update` frame — confirmed by `cargo test -p server session_ready_observer` | ADR-012 | ✅ CONFIRMED |
| 16 | `Res<T>` inserted via `Commands::insert_resource()` before `Commands::trigger()` is visible to Observer — confirmed by `cargo test -p server session_ready_observer` | ADR-012 | ✅ CONFIRMED |
| 17 | `Trigger<T>` is correct Observer parameter type in Bevy 0.18 — confirmed from Bevy 0.18 api_patterns | ADR-012 | ✅ CONFIRMED |
| 18 | Component replication is opt-in: entity must have `Replicate::default()` AND component must be registered via `app.register_component::<T>()` | ADR-007 | ✅ CONFIRMED |
| 19 | `ReplicationGroup` struct confirmed in prelude; `ReplicationGroup::new_id(id)` syntax confirmed | ADR-001 | ✅ CONFIRMED |
| 20 | `LocalTimeline` is a struct in `lightyear::core::prelude`; accessible as `Res<LocalTimeline>` | Engine reference | ✅ CONFIRMED |

**Legend:** ✅ CONFIRMED — API exists as assumed | ⚠️ DIFFERS — API differs, resolution path documented in `tests/evidence/lightyear-026-verification.md`

Items 15 and 16 are resolved. Local Windows verification requires Developer PowerShell for VS 2026 because normal PowerShell does not load MSVC `link.exe`.

**Do not merge any networking story with unverified or unresolved DIFFERS items.**

---

## Skill Activation Rules

These skills are non-optional gates on specific file types:

| File type | Skill | Why |
|-----------|-------|-----|
| Any `.rs` importing `bevy` | `liv-bevy-018` | Enforces 0.18 API patterns; prevents deprecated Bundle/pre-0.15 patterns |
| Any `.rs` importing `lightyear` | `liv-bevy-lightyear` | Lightyear 0.26 API; verification patterns for post-cutoff networking |
| Both in same file | Activate **both** | Networking code uses both APIs simultaneously |
