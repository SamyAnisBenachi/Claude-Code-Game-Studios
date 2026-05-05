# Control Manifest

> **Engine**: Bevy 0.18 + Lightyear 0.26
> **Last Updated**: 2026-05-05
> **Manifest Version**: 2026-05-05
> **ADRs Covered**: ADR-001, ADR-002, ADR-003, ADR-004, ADR-005, ADR-006, ADR-007, ADR-008, ADR-009, ADR-010, ADR-011, ADR-012, ADR-013, ADR-014, ADR-015, ADR-016, ADR-017, ADR-018, ADR-019, ADR-020, ADR-021, ADR-022
> **Status**: Active — regenerate with `/create-control-manifest` when ADRs change

`Manifest Version` is the date this manifest was generated. Story files embed this date when
created. `/story-readiness` compares a story's embedded version to this field to detect stories
written against stale rules. Always matches `Last Updated`.

This manifest is a programmer's quick-reference extracted from all Accepted ADRs, technical
preferences, and engine reference docs. For the reasoning behind each rule, see the referenced ADR.

---

## Foundation Layer Rules

*Applies to: Cargo workspace layout, asset loading pipeline, Lightyear channel configuration,
Round State Machine phase state, RSM event bus, session lifecycle, reconnect protocol.*

### Required Patterns

- **Three-crate Cargo workspace: `shared/`, `server/`, `client/`.** `shared/` = protocol types only; `server/` = headless binary; `client/` = WASM binary. — source: ADR-003
- **`shared/` Cargo features: `bevy = { default-features = false, features = ["serialize"] }` and Lightyear `shared` feature only.** No `bevy_ecs`, `bevy_render`, `bevy_ui`, `tokio`, or server-only Lightyear features. — source: ADR-003
- **`server/` Cargo features: headless Bevy (`multi_threaded` only), Lightyear `server` + `websocket`.** `rand` and `rand_chacha` are server-only crates. — source: ADR-003
- **`client/` Cargo features: `bevy_ui`, `bevy_sprite`, `bevy_text`, `bevy_asset`, `bevy_winit`, `webgl2`, Lightyear `client` + `websocket`.** — source: ADR-003
- **Single `pub fn register_protocol(app: &mut App)` in `shared/src/protocol.rs`.** Both `server/main.rs` and `client/main.rs` call it at startup. Never duplicate. — source: ADR-003
- **`GameConfig` struct in `shared/config.rs` is a plain serde struct — NO `#[derive(Resource)]`.** Server wraps it via `commands.insert_resource(GameConfig::load(...))`. — source: ADR-003, ADR-004
- **Server internal layering (code-review enforced): `feature/` may import from `core/`; `core/` may import from `foundation/`. Reverse direction forbidden.** — source: ADR-003
- **Client internal layering: `ui/` → `state/` → `network/` → `shared/`. Reverse direction forbidden.** — source: ADR-003
- **Workspace `Cargo.toml` release profile: `lto = "thin"`, `codegen-units = 1`, `strip = "symbols"`, `panic = "abort"`.** Dev profile: `opt-level = 1`. — source: ADR-003
- **Adding a dependency to `shared/Cargo.toml` requires an ADR amendment and technical-director approval.** — source: ADR-003
- **Asset loading: use `bevy_asset_loader` `LoadingState` for `GameConfig` (RON) and `CardCatalog` (JSON).** Both loaded before transitioning to `AppState::Lobby`. — source: ADR-004
- **Every `impl AssetLoader` struct must `#[derive(Default, TypePath)]`.** Missing `TypePath` produces a runtime error. — source: ADR-004
- **`ron` must be a direct crate dependency in `server/Cargo.toml`.** Not re-exported from `bevy_asset` in Bevy 0.18. — source: ADR-004
- **Fatal load errors call `AppExit::Error(NonZeroU8::MIN)` — NOT `panic!`.** Lets Bevy shut cleanly with a non-zero exit code. — source: ADR-004
- **Hot-reload for `GameConfig` in debug builds only: gate the `add_systems` call itself behind `#[cfg(debug_assertions)]`.** Not just the function body. — source: ADR-004
- **Exactly 2 Lightyear channels: `ReliableChannel` (ordered, guaranteed) and `UnreliableChannel` (best-effort).** Channel definitions live in `shared/src/protocol.rs`. Assignment is permanent per message type — never switch at runtime. — source: ADR-008
- **Only `C2SHeartbeat` and `S2CAuctionUpdate` use `UnreliableChannel`.** Everything else uses `ReliableChannel`. — source: ADR-008
- **`S2CResolutionEvent` MUST be enqueued before `S2CPhaseChanged(DRAFT_SHOP)` in the same server frame (OQ-D invariant).** Enforced by same-channel FIFO on `ReliableChannel`. — source: ADR-008
- **Default channel for any new message type: `ReliableChannel`.** Use `UnreliableChannel` only when: dropped packet is immediately superseded, stale arrival causes no state corruption, and the message is sent more than once per phase. — source: ADR-008
- **`C2SSubmitPlacement` payload shape is submit-only:** `PlacedCardSubmit { card_id, target, current_mana_spend, reserve_mana_spend }`. `S2CPlacementReveal` uses a distinct reveal type and MUST NOT expose mana-spend fields. — source: network-protocol.md, ADR-007
- **Per-connection `snapshot_sent` flag.** Set to `false` on `OnConnected`. Snapshot system scheduled BEFORE all live-game message systems. No live S2C enqueued to a reconnecting client before `snapshot_sent = true`. — source: ADR-008, ADR-011
- **`RoundState` resource is the single source of truth for phase.** Only `rsm_tick_system` holds `ResMut<RoundState>`. All other systems use `Res<RoundState>`. — source: ADR-009
- **`rsm_tick_system` scheduled AFTER `AuctionSystem` and `CombatResolutionSystem` in `Update`.** — source: ADR-009
- **Phase gate pattern in every C2S handler:** `if round_state.phase != X { return; }` — silent discard, no error to client. — source: ADR-002, ADR-009
- **Use `#[derive(Message)]` + `MessageWriter<T>` / `MessageReader<T>` + `app.add_message::<T>()` for all RSM buffered signals.** `EventWriter`/`EventReader`/`Events<T>` do not exist in Bevy 0.17+. — source: ADR-009, ADR-010
- **`SessionReady` uses `#[derive(Event)]` + `commands.trigger(SessionReady)` (Observer) — NOT `app.add_message::<SessionReady>()`.** One-shot lifecycle trigger, not a recurring message. — source: ADR-010, ADR-012
- **RSM has zero imports from `server/feature/`.** All phase-reactive logic is triggered by events, not direct calls. — source: ADR-010
- **F2 emission order in `advance_phase` is strict code order:** 1. `DraftStarted`, 2. `ShopRefreshTriggered` (per player), 3. `AuctionPhaseEntered` (auction rounds only), 4. `BroadcastPhaseChanged` (ALWAYS LAST). — source: ADR-010
- **All subscriber systems scheduled `.after(advance_phase)`.** A subscriber before `advance_phase` misses the current frame's messages. — source: ADR-010
- **GSS command order before triggering `SessionReady`:** `commands.insert_resource(SessionConfig)` → `commands.insert_resource(ServerRng)` → `commands.trigger(SessionReady)`. Never reorder. — source: ADR-012
- **Exactly one Observer for `SessionReady` — the RSM's `on_session_ready` handler.** Other systems react to `DraftStarted`, not `SessionReady`. — source: ADR-012
- **`evaluate_session_ready` runs only while in LOBBY state.** Gate on `LobbyState::GameActive` to prevent re-trigger. — source: ADR-012
- **`ObjectiveIdentity { is_fake }` is NEVER in the Lightyear replication graph.** Server holds `HiddenObjectives` as a server-only resource. — source: ADR-001
- **At `DRAFT_INITIAL`: send `S2CObjectiveIdentities` via `ServerMultiMessageSender::send::<S2CObjectiveIdentities, ReliableChannel>(&msg, server, &NetworkTarget::Single(peer_id))` — one unicast per player.** Lightyear 0.26 uses `PeerId`, not `ClientId`. — source: ADR-001, Lightyear 0.26 verification
- **Mandatory reconnect send order (all `ReliableChannel`, all unicast):** 1. `S2CHandshake`, 2. `S2CGameSnapshot`, 3. `S2CObjectiveIdentities`, 4. `S2CPhaseChanged`. Then set `snapshot_sent = true` and flush deferred queue. — source: ADR-011
- **`S2CObjectiveIdentities` must be explicitly re-sent on every reconnect.** Lightyear reliable delivery does not replay across transport reconnects. — source: ADR-001, ADR-011
- **`hello_timeout_ms` watchdog: 5000ms default.** Close connection silently if no `C2SHello` within the window. — source: ADR-011

### Forbidden Approaches

- **Never add `server` as a dependency in `client/Cargo.toml`.** CI must fail any PR introducing this edge. — source: ADR-003
- **Never use `#[cfg(feature = "server")]` to gate game logic in `protocol/` or `client/`.** Server-only types live in `server/`. — source: ADR-002, ADR-003
- **Never add `#[derive(Resource)]`, `Plugin` impls, or `App::add_systems` to `shared/`.** — source: ADR-003
- **Never add `rand` or `rand_chacha` to `client/Cargo.toml` for gameplay purposes.** — source: ADR-003, ADR-005
- **Never lazy-load `GameConfig` or `CardCatalog`.** Must load before `AppState::Lobby`. — source: ADR-004
- **Never hot-reload `CardCatalog`.** Card data changes require a server restart. — source: ADR-004
- **Never split `S2CResolutionEvent` and `S2CPhaseChanged(DRAFT_SHOP)` onto different channels.** Cross-channel ordering is not guaranteed. — source: ADR-008
- **Never use `EventWriter<T>` / `EventReader<T>` / `Events<T>`.** Do not exist in Bevy 0.17+. Use `MessageWriter`/`MessageReader` (buffered) or Observers (reactive). — source: ADR-009, ADR-010
- **Never use `app.add_message::<SessionReady>()` or `MessageReader<SessionReady>`.** `SessionReady` is an Observer event — `MessageReader` will never fire for it. — source: ADR-012
- **Never register more than one Observer for `SessionReady`.** Other systems subscribe to `DraftStarted`. — source: ADR-012
- **Never send `S2CGameSnapshot` as a broadcast.** Always unicast per player with opponent secrets stripped. — source: ADR-011
- **Never use `ResMut<RoundState>` in any system other than `rsm_tick_system`.** — source: ADR-009
- **Never schedule a subscriber system before `advance_phase`.** — source: ADR-010
- **Never use direct function calls from `advance_phase` into feature modules.** Violates Core→Feature dependency direction. — source: ADR-010
- **Never replicate `ObjectiveIdentity` as a Lightyear ECS component.** Entity-split + `ReplicationGroup` workaround also rejected — same silent-leak risk. — source: ADR-001

### Performance Guardrails

- **WASM bundle size**: < 50 MB. CI gate fails above threshold. — source: ADR-003, technical-preferences.md
- **Server steady-state tick**: ≤ 5 ms. During RESOLUTION: ≤ 15 ms. — source: ADR-002
- **Per-round message budget**: < 1 KB/round/player. Reconnect snapshot: < 16 KB (hard ceiling 32 KB). — source: ADR-002, ADR-011

---

## Core Layer Rules

*Applies to: client-server authority, objective identity, server RNG, card data schema,
auction state, class system, card acquisition shop state, economy system, board/lane state.*

### Required Patterns

- **Server is the sole authority over all game state. Client is a read-only view.** Invalid C2S inputs are silently discarded — no error response to client. — source: ADR-002
- **All C2S handlers follow this pattern:** 1. Resolve `ClientId` → `PlayerId` (unknown sender → log + drop, never panic). 2. Phase gate (silent discard). 3. Domain validation (silent discard). 4. Apply to authoritative state. 5. Broadcast/unicast S2C. — source: ADR-002
- **No optimistic client updates.** `ClientState` mutates only from inbound S2C — never from local user input. — source: ADR-002
- **On `OnConnected`: server sends `S2CGameSnapshot` before any other S2C.** `snapshot_sent` flag mechanism (Foundation layer). — source: ADR-002
- **Server tick is the wall clock.** Client timer display is derived from `S2CPhaseChanged.deadline_server_ms` for presentation only. — source: ADR-002
- **Single `ServerRng` resource backed by `ChaCha20Rng` from `rand_chacha 0.3`.** Seeded once from `OsRng` at session start; removed on `GameOverEmitted`. — source: ADR-005
- **All RNG access via intent-named methods on `ServerRng` — never raw `next_u32`/`gen`.** Every consumption must push an `AuditEntry` in the same call as the draw. — source: ADR-005
- **Strict RNG consumption order (ADR-005 §4).** RESOLUTION chain Orders 4–10: `RangeEquidistantSelect` → `TeleportRandomDest` → `StrichChangeLaneSelect` → `ResolveEcaflip` → `ResolvePrism` → `AwardFakeObjectiveReward` → `DrawFreeCard`. Any new consumer requires a new `RngEvent` variant + ADR-005 amendment before the story opens. — source: ADR-005
- **Inter-player ordering for concurrent RNG events:** ascending `player_id` → ascending `lane_index` → ascending `cell`. — source: ADR-005
- **`CardCatalog` (immutable, server-lifetime) = `HashMap<CardId, CardData>` from `assets/data/cards.json`.** `PlayerPool` (mutable, session-scoped) = per-player copy counts. Two separate concerns, two separate types. — source: ADR-006
- **`EPIC_POOL_COPIES = 1` and `LEGENDARY_POOL_COPIES = 1` are compile-time `const` — NOT `GameConfig` fields.** — source: ADR-006
- **`PlayerPool::distribute()` is the sole pool mutation.** Returns `Err(DistributeError::Exhausted)` at 0 copies — never underflows. — source: ADR-006
- **`total_acquired = initial_count - copies_remaining`.** No separate tracking field needed. — source: ADR-006
- **All draw functions return `Option<T>`.** Never panic on exhausted pool. — source: ADR-006
- **`Keyword` enum: use `#[serde(tag = "kw", content = "val")]` (adjacent tagging).** Serializes `Simple(Shield)` as `{ "kw": "Simple", "val": "Shield" }`. DO NOT use `#[serde(untagged)]` — fails at runtime for newtype variants with scalar inner type. — source: ADR-006, ADR-018
- **`SimpleKeyword::Haste` (not `Charge`) is the combat keyword removing summoning sickness.** `cards.json` must use `"Haste"`. Any fixture using `Charge` must be updated. — source: ADR-006, ADR-018
- **`AuctionState` resource: `AuctionState::default()` starts in `AuctionPhase::Idle`.** Only `auction_tick_system` holds `ResMut<AuctionState>`. — source: ADR-013
- **Per-frame code order in `auction_tick_system`:** 1. Handle `AuctionPhaseEntered` (IDLE → LIVE_BIDDING). 2. Handle `AbortAuction` (cleanup → IDLE, no `AuctionSettled`). 3. Drain Lightyear `MessageReceiver<C2SAuctionBid>`. 4. `saturating_sub` timer (`u32::try_from(time.delta().as_millis()).unwrap_or(u32::MAX)` — NOT `as u32`). 5. If `timer == 0`: settle → write `MessageWriter<AuctionSettled>` → IDLE. — source: ADR-013
- **Release-before-reserve invariant (atomic within one system body):** `api::release_gold_reservation(prev_leader)` THEN `api::reserve_gold(new_leader, amount)` — sequential lines, no system boundary between them. — source: ADR-013
- **`auction_tick_system` scheduled BEFORE `rsm_tick_system`:** `AuctionSet::Tick.before(RsmSet::Tick)`. — source: ADR-013
- **Lightyear `MessageReceiver<T>` (C2S network) ≠ Bevy `MessageReader<T>` (internal bus).** Do NOT use `MessageReader<C2SAuctionBid>` for a Lightyear network message. — source: ADR-013
- **`PlayerSessions` resource owns session-identity state only: `class: ClassId` and `class_locked: bool`.** Economy state → `PlayerEconomies`. Hand state → `PlayerHands`. Do NOT accumulate other fields. — source: ADR-014
- **Import `ClassId` from `shared::card::ClassId` — NEVER redefine it.** — source: ADR-014
- **`C2SClassChoice` derives Lightyear's `Message` trait (`lightyear::prelude::Message`) — NOT Bevy's `Message` trait.** Both exist in this project simultaneously. — source: ADR-014
- **LOBBY→DRAFT_INITIAL gate: `all_classes_chosen()` must return `true`.** `ClassId::Neutral` as player class → silent discard. `class_locked == true` → silent discard. — source: ADR-014
- **`SourceClass(ClassId)` component on token entities: set at spawn, NEVER mutated.** Absent on non-token units. — source: ADR-014
- **Class effects in RESOLUTION are plain Rust functions called from within the RESOLUTION system body — NOT standalone Bevy systems, NOT buffered Messages within RESOLUTION.** Frame-delayed messages violate sub-step ordering. — source: ADR-014
- **`ShopStates` resource sole writer: `card_acquisition_tick_system`.** `PlayerHands` sole DRAFT-phase writer: also `card_acquisition_tick_system`. Prism/Objective write `PlayerHands` in RESOLUTION only — exclusive by phase. — source: ADR-015
- **CA18 mandatory rollback:** `spend_gold` → `distribute` → on `Err(Exhausted)`: `refund_gold` immediately. Sequential in one system body. Gold must never remain deducted after a failed distribute. — source: ADR-015
- **`displayed_this_draft` NOT cleared on `ShopUnlock` trigger.** Dedup accumulates across DRAFT_AUCTION + DRAFT_SHOP. Reset only on new DRAFT_INITIAL entry. — source: ADR-015
- **`card_acquisition_tick_system` scheduled AFTER `rsm_tick_system`:** `CardAcquisitionSet::Tick.after(RsmSet::Tick)`. — source: ADR-015
- **All `PlayerEconomy` field mutations go through `server/src/core/economy/api.rs` functions.** Direct field assignment (`.gold = x`, `.current_mana = x`) outside `api.rs` is forbidden. — source: ADR-019
- **Placement mana spends use explicit split API:** Board/Lane calls `validate_explicit_mana_split` before pending write and `apply_explicit_mana_split` at PLACEMENT close. Normal non-placement spends keep auto-split `validate_spend` / `apply_spend`. — source: ADR-019
- **Interest snapshot triggered by `MessageReader<ResolutionComplete>` — NOT `ResolutionPhaseEntered`.** Snapshot must occur after all kill/objective gold awards in `resolve_combat`. — source: ADR-019
- **`on_resolution_complete` scheduled `.before(rsm_input_reader)`.** Snapshot must precede RSM DRAFT transition. — source: ADR-019
- **`PendingResolutionComplete(bool)` bridge resource.** `resolve_combat` (exclusive system) sets `world.resource_mut::<PendingResolutionComplete>().0 = true`. `drain_pending_resolution_complete` (regular system, `CombatSystemSet::PostResolution`) emits `MessageWriter<ResolutionComplete>`. `MessageWriter<T>` cannot be used from an exclusive system. — source: ADR-019
- **`ResMut<PlayerEconomies>` write access restricted to five systems:** `initialise_player_economies`, `on_draft_started`, `on_resolution_complete`, `auction_tick_system`, `resolve_combat`. — source: ADR-019
- **Board state is a hybrid: ECS entities (Lightyear-replicated) + `BoardState` resource (O(1) spatial index).** All board mutations through `board/api.rs` — entity components AND `BoardState` index updated atomically. — source: ADR-020
- **Unit entities spawn WITHOUT `Replicate` component.** Add `Replicate::to_clients(NetworkTarget::All)` ONLY AFTER `S2CPlacementReveal` is enqueued. — source: ADR-020
- **Correct Lightyear 0.26 replication API: `Replicate::to_clients(NetworkTarget::All)`.** `ReplicateTo` component does NOT exist in Lightyear 0.26.0. — source: ADR-020
- **Movement formula F1:** `new_cell = clamp(current_cell + direction × mp, 1, 8)` using `i16` intermediate arithmetic to prevent u8 overflow/underflow. — source: ADR-020
- **`remove_unit_from_board` removes from `BoardState` index but does NOT despawn entity.** Caller explicitly calls `commands.entity(e).despawn()`. — source: ADR-020
- **`expand_spawn_range(state, player)` clamped at 2.** Maximum fakes destroyed = 2. — source: ADR-020

### Forbidden Approaches

- **Never `panic!` on invalid C2S input.** Use `tracing::debug!` for rejection logs. Return silently. — source: ADR-002
- **Never replicate `ObjectiveIdentity` as an ECS component.** — source: ADR-001
- **Never use `rand::thread_rng()`, `StdRng`, or `SmallRng` in server game logic.** — source: ADR-005
- **Never transmit seed bytes, `seed_index`, or `audit_log` entries in any production S2C message.** — source: ADR-005
- **Never add a new RNG consumer without first registering it in ADR-005 §4 consumption order table.** — source: ADR-005
- **Never mutate `CardCatalog` after server startup.** — source: ADR-006
- **Never use `SimpleKeyword::Charge` in code or `cards.json`.** Renamed to `Haste`. — source: ADR-006, ADR-018
- **Never use `#[serde(untagged)]` or `#[serde(tag = "kw")]` alone on `Keyword` enum.** Internal-tagging fails for newtype variants with scalar inner type. Use adjacent tagging: `#[serde(tag = "kw", content = "val")]`. — source: ADR-006, ADR-018
- **Never use `ResMut<AuctionState>` in any system other than `auction_tick_system`.** — source: ADR-013
- **Never drain `MessageReceiver<C2SAuctionBid>` in more than one system.** First drain consumes all. — source: ADR-013
- **Never use cross-frame messaging for the release/reserve gold pair.** Breaks the one-frame simultaneous-reservation invariant. — source: ADR-013
- **Never add economy or hand fields to `PlayerSessionData`.** — source: ADR-014
- **Never use `ClassId::Neutral` as a valid player class in DRAFT or later phases.** — source: ADR-014
- **Never drain `MessageReceiver<C2SClassChoice>` in more than one system.** — source: ADR-014
- **Never clear `displayed_this_draft` on `ShopUnlock` trigger.** — source: ADR-015
- **Never use cross-frame messaging for the spend/refund pair (CA18).** — source: ADR-015
- **Never use `ResMut<ShopStates>` in any system other than `card_acquisition_tick_system`.** — source: ADR-015
- **Never use `ResolutionPhaseEntered` as the interest snapshot trigger.** Fires too early — kills/objective gold not yet awarded. — source: ADR-019
- **Never use `MessageWriter<T>` as a system param inside an exclusive system.** Use `PendingResolutionComplete` bridge resource instead. — source: ADR-019
- **Never assign to `PlayerEconomy` fields directly outside `economy/api.rs`** (`.gold =`, `.current_mana =`, etc.). — source: ADR-019
- **Never mutate `BoardPosition` component or `BoardState` index outside `board/api.rs`.** — source: ADR-020
- **Never add `Replicate` to a unit entity before `S2CPlacementReveal` is enqueued.** — source: ADR-020
- **Never use `ReplicateTo` component.** Does not exist in Lightyear 0.26. — source: ADR-020
- **Never use any `*Bundle` type (`SpriteBundle`, `Camera2dBundle`, `NodeBundle`, etc.).** Deprecated in Bevy 0.15. Use Required Components API. — source: engine-reference

### Performance Guardrails

- **Client S2C processing + view update**: ≤ 2 ms per frame. — source: ADR-002
- **Server steady-state game logic**: ≤ 5 ms per frame tick. — source: ADR-002
- **`RoundState` lookup**: O(1) — single resource dereference. — source: ADR-009
- **Board spatial query `get_units_at_cell`**: O(1) HashMap lookup. — source: ADR-020

---

## Feature Layer Rules

*Applies to: placement buffer, prism system, combat resolution, keyword system (ECS state + observer architecture).*

### Required Patterns

- **`PendingPlacements` resource holds pending submissions as plain Rust data — NO ECS entity spawn during PLACEMENT.** Unit entities only exist after sub-step 1 commit. — source: ADR-007
- **Placement validation is all-or-nothing per player batch.** Any single card failure → silent discard of entire batch. No partial acceptance. — source: ADR-007
- **Placement submit authority validation covers sender, phase, hand ownership, duplicate card IDs, target legality, spawn/occupancy, and explicit current/reserve mana split before pending write.** — source: ADR-007, ADR-019
- **Buffer cleared on `PlacementPhaseEntered` (not `PlacementPhaseExited`).** — source: ADR-007
- **`close_placement_phase` strict order (all in ONE system — never split across two):** 1. Collect `PendingPlacements`. 2. Deduct mana. 3. Enqueue `S2CPlacementReveal` on `ReliableChannel`. 4. Spawn ECS unit entities. 5. Add `Replicate::to_clients(NetworkTarget::All)`. 6. Emit `PlacementCommitted`. 7. Clear buffer. — source: ADR-007
- **Mana deducted at PLACEMENT close, not at submission receipt.** — source: ADR-007
- **Spawn range validation (Formula F2) applies to Minions only.** Structures and Traps bypass it. — source: ADR-007
- **`PrismState` resource sole writer: `resolve_prism_draws`.** External systems read prism state via `PrismPresence` component replication — never via `PrismState` directly. — source: ADR-016
- **`PrismCollected` is a server-internal Bevy `#[derive(Message)]` — NOT a Lightyear C2S message.** Use `MessageReader<PrismCollected>`, NOT Lightyear's `MessageReceiver<PrismCollected>`. — source: ADR-016
- **`hand_push()` is a shared module function.** Both Prism System and Card Acquisition call it. CA is DRAFT-only; Prism is RESOLUTION-only — no concurrent write conflict. — source: ADR-016
- **`resolve_prism_draws` scheduled:** `.after(resolve_ecaflip_triggers).before(award_fake_objective_rewards)` within the RESOLUTION `Update` set. — source: ADR-016
- **Prism S2C sends use Lightyear 0.26 `ServerMultiMessageSender`.** Owner-only sends (`S2CCardAcquired`, `S2CPrismRewardDropped`) target `NetworkTarget::Single(owner_peer_id)`; all-player respawn sends target `NetworkTarget::All`. — source: ADR-016, ADR-008
- **`resolve_combat` is an exclusive Bevy system: `fn resolve_combat(world: &mut World)`.** Bevy 0.18 auto-detects `&mut World`. All 6 sub-steps execute in a single frame invocation. — source: ADR-017
- **Base stats snapshot taken at RESOLUTION entry; LEADER bonuses snapshot post-SS1/pre-SS2 after all SS1 APPEARANCE effects resolve.** — source: ADR-017, ADR-018
- **`apply_combat_modifier_stack` is a pure function — no World access.** All modifier-stack CRs testable without Bevy context. — source: ADR-017
- **Movement boundary (resolves OQ1):** Destination rule (Formula F1) governs Trap/Prism triggering — intermediate cells skipped. Enemy collision tick-by-tick loop governs obstruction (WALL halt, path-crossing). Complementary layers, not contradictions. — source: ADR-017
- **`S2CResolutionEvent` is a single reliable broadcast sent AFTER all 6 sub-steps complete.** Clients receive the full log and replay at animation tempo. — source: ADR-017
- **HP mutations use `saturating_sub`.** Never raw `u8 -= u8` (debug panic / release wrap). — source: ADR-017
- **`MessageWriter<BeginResolution>` / `MessageWriter<ResolutionComplete>` are Bevy-internal messages.** `S2CResolutionEvent` uses Lightyear's `MessageSender<T>`. Do not confuse them. — source: ADR-017
- **All 6 persistent keyword states in ONE `UnitKeywordState` component (monolithic).** Co-location avoids archetype migrations — up to 720 per RESOLUTION round if stored separately. — source: ADR-018
- **`bodyguard_protects: Option<Entity>` is a typed Bevy entity handle — NOT a lane/cell index.** Stable across CHANGE LANE. Entity ID never changes when position changes. — source: ADR-018
- **NEVER serialize `bodyguard_protects: Option<Entity>` into `protocol/` types.** Use `EntityId` (session-scoped u32) in network types. — source: ADR-018
- **`bodyguard_cleanup_system` runs in `PostUpdate`** using `&Entities` (Bevy 0.18 system param) for O(1) alive check (`entities.contains(entity)`). Clears stale `Option<Entity>` after despawns. — source: ADR-018
- **Keyword effects module (`server/feature/keyword/`) is separate from combat module.** Combat calls `keyword::effects::*` as plain function calls. Combat owns *when*; Keyword owns *what*. — source: ADR-018
- **`leader_snapshot_system` runs after SS1 fully drains, before SS2 begins.** LEADER units entering during SS1 are included if alive at snapshot time; the bonus persists through SS5/SS6 even if the LEADER dies in SS4 and recomputes fresh post-SS1 next round. — source: ADR-018
- **`eval_outnumbered_system` called at each sub-step boundary.** After SS4, it runs only after `ChainDeathBuffer` fully drains and all resulting removals are processed; it emits `OutnumberedFlipped` only on boolean transition (bandwidth-efficient). — source: ADR-018
- **Three RNG seed slots must be registered in ADR-005 §4 BEFORE any keyword story opens:** `range_equidistant_select`, `teleport_random_dest`, `strich_change_lane_select`. KW-033b is formally BLOCKED until done. — source: ADR-018
- **`Keyword`/`SimpleKeyword` round-trip test for all 7 variants must pass before the ADR-006 amendment merges.** — source: ADR-018
- **5 global Observers registered in `KeywordPlugin::build()`:** `on_unit_appeared`, `on_unit_died`, `on_final_blow_dealt`, `on_start_of_turn`, `on_end_of_turn`. — source: ADR-022
- **Every Observer handler MUST check keyword presence as its FIRST operation and `return` early if absent.** Global observers fire for ALL entities — guard is mandatory, not optional. — source: ADR-022
- **DEATH chain managed via `ChainDeathBuffer` VecDeque — NOT recursive `world.trigger_targets()` inside `on_unit_died`.** `ChainDeathBuffer` cleared at SS4 start. Initial deaths seeded in lane order (Lane 1 first). — source: ADR-022
- **`world.trigger_targets(event, entity)` is the correct call within the exclusive system.** Fires observers synchronously. Confirmed valid `World` method in Bevy 0.18. — source: ADR-022
- **`On<T>` is the correct observer handler parameter type (Bevy 0.17+).** NOT `Trigger<T>` (pre-0.17 form). Confirmed by `breaking-changes.md` line 140. — source: ADR-022
- **COUNTERATTACK and INJURED use inline dispatch — NOT Observers.** COUNTERATTACK requires proximity check before dispatch. INJURED is a state re-evaluation scan at sub-step boundaries, not an event. — source: ADR-022
- **START OF TURN: normal Bevy system reads `MessageReader<DraftPhaseEntered>` → `commands.trigger_targets(StartOfTurnTriggered, entity)`.** `app.add_message::<DraftPhaseEntered>()` registered in RSM plugin (the emitter), NOT in `KeywordPlugin`. — source: ADR-022
- **`app.add_message::<KeywordTriggered>()` MUST be registered in `KeywordPlugin`.** Missing registration panics at first `MessageWriter::write()`. — source: ADR-022
- **`ResMut<T>` and `MessageWriter<T>` are usable inside Observer handlers** (standard system params work in observers). Sequential borrow pattern in drain loop is safe. — source: ADR-022

### Forbidden Approaches

- **Never spawn ECS entities for pending placements during PLACEMENT phase.** — source: ADR-007
- **Never split `S2CPlacementReveal` enqueue and entity spawn into two systems.** — source: ADR-007
- **Never accept partial batch placement.** All-or-nothing per player only. — source: ADR-007
- **Never deduct mana at submission receipt.** Deduct at PLACEMENT close. — source: ADR-007
- **Never use Lightyear `MessageReceiver<PrismCollected>`.** It is a Bevy-internal message; use `MessageReader<PrismCollected>`. — source: ADR-016
- **Never add `ResMut<PrismState>` to any system other than `resolve_prism_draws`.** — source: ADR-016
- **Never schedule Card Acquisition systems during RESOLUTION phase.** — source: ADR-016
- **Never break RESOLUTION into a multi-frame state machine.** Exclusive system ensures atomicity and satisfies the 60-second RSM safety timeout. — source: ADR-017
- **Never stream per-sub-step S2C events to clients.** One batch message only (`S2CResolutionEvent`). — source: ADR-017
- **Never apply raw `u8 -= u8` for HP mutations.** Use `saturating_sub`. — source: ADR-017
- **Never use individual `Component` flags per keyword state.** Archetype migration cost at RESOLUTION frequency is prohibitive. — source: ADR-018
- **Never store BODYGUARD bond as a lane/cell index.** Must be `Option<Entity>` to survive CHANGE LANE. — source: ADR-018
- **Never serialize `bodyguard_protects: Option<Entity>` into protocol types.** — source: ADR-018
- **Never omit the guard check in an Observer handler.** Effects silently fire for units without the keyword. — source: ADR-022
- **Never use recursive `world.trigger_targets()` inside `on_unit_died`.** Use `ChainDeathBuffer` explicit queue. — source: ADR-022
- **Never use Observers for COUNTERATTACK or INJURED.** Inline dispatch only. — source: ADR-022
- **Never use `Trigger<T>` as observer handler parameter type.** Use `On<T>` (Bevy 0.17+). — source: ADR-022

### Performance Guardrails

- **Combat resolution (worst-case 5-lane contested round)**: ≤ 15 ms in a single exclusive system frame. — source: ADR-017
- **`resolve_combat` idle (no `BeginResolution`)**: exits immediately — < 1 µs. — source: ADR-017

---

## Presentation Layer Rules

*Applies to: client-side rendering, UI, animations, asset sharing, phase synchronization.*

### Required Patterns

- **`PresentationPlugin` registration order is a contract — DO NOT reorder:** 1. `CardAnimationsPlugin`, 2. `BoardRenderingPlugin`, 3. `HandUiPlugin`, 4. `HudPlugin`, 5. `ShopAuctionUiPlugin`. Reordering causes runtime panics (Resource not yet inserted). — source: ADR-021
- **`PresentationSet` execution order:** `PhaseTransition` → `MessageDrain` → `StateSync` → `AnimationTick` (chained via `.chain()`). — source: ADR-021
- **Single `phase_sink_system` drains `MessageReceiver<S2CPhaseChanged>` (Lightyear).** All sub-plugins read `Res<CurrentClientPhase>` — never `MessageReceiver<S2CPhaseChanged>` directly. — source: ADR-021
- **Single shared economy-view system drains `MessageReceiver<S2CGoldUpdate>` and seeds `PlayerEconomyView` from `S2CGameSnapshot`.** Hand UI, HUD, and Shop/Auction UI read `Res<PlayerEconomyView>` instead of each draining economy messages. — source: ADR-021
- **Board content ALWAYS world-space:** `Sprite` + `Transform` with `Camera2d`. Board entities cannot appear above bevy_ui without custom render layers. — source: ADR-021
- **UI ALWAYS bevy_ui:** `Node` for HUD, hand fan, shop panels, auction bid box. — source: ADR-021
- **Hand drag-sprite preview is a bevy_ui `Node` — NOT a world-space `Sprite`.** Preserves correct z-ordering above board during drag. — source: ADR-021
- **`CardAtlas` shared Resource: `Handle<Image>` + `Handle<TextureAtlasLayout>`.** Atlas sprite spawn pattern: `Sprite { texture_atlas: Some(TextureAtlas { layout, index }), .. }`. `Handle<TextureAtlas>` asset type does not exist in Bevy 0.18. — source: ADR-021
- **Tween cancel-and-replace: `Animator<T>::set_tweenable(new_tween)`.** Never despawn+respawn. Never write `Transform` while `Animator<Transform>` is active (animator overwrites next frame). — source: ADR-021
- **Child entity Z: use LOCAL offset — NOT absolute world Z.** `local_z = target_world_z − parent_world_z`. `GlobalTransform` adds parent + child values. — source: ADR-021
- **`PickingBehavior` component only inside `#[cfg(feature = "ui_picking")]` guard.** Inserting without the feature compiled panics at runtime. CI must include a build without `ui_picking`. — source: ADR-021
- **Pre-pool all HUD and hand fan entities at session start.** Toggle `Visibility` only — never spawn/despawn in steady state. — source: ADR-021
- **`BoardLayout` and `CardAtlas` are session-scoped Resources** (inserted on `OnEnter(ClientState::InSession)`, removed on `OnExit`). Systems reading them must be `in_state(ClientState::InSession)`. — source: ADR-021
- **`Color::srgba` / `Color::srgb` constructors.** `Color::rgba` was renamed in Bevy 0.15. — source: ADR-021, engine-reference
- **`SpriteAlphaLens::lerp()`: use `target.color.with_alpha(alpha)`.** `Color` has no mutating `set_alpha()` in Bevy 0.18. — source: ADR-021

### Forbidden Approaches

- **Never use any `*Bundle` type** (`SpriteBundle`, `Camera2dBundle`, `NodeBundle`, etc.). Deprecated in Bevy 0.15. Use Required Components API. — source: ADR-021, engine-reference
- **Never use `Handle<TextureAtlas>`.** Asset type removed in Bevy 0.18. Use `Handle<TextureAtlasLayout>` with `TextureAtlas` as a component field. — source: ADR-021, engine-reference
- **Never register `MessageReceiver<S2CPhaseChanged>` in more than one system.** First drain consumes all — other systems silently miss messages. — source: ADR-021
- **Never register `MessageReceiver<S2CGoldUpdate>` in Hand UI, HUD, or Shop/Auction UI.** The shared economy-view system is the only production drain. — source: ADR-021
- **Never despawn+respawn entities for tween cancel-and-replace.** Discards game-state components. — source: ADR-021
- **Never assign a child entity's `Transform.translation.z` to the intended world Z directly.** Renders at `parent_z + intended_world_z`. Use local offset. — source: ADR-021
- **Never use `UiImage::new()`.** Use `ImageNode::new()` (Bevy 0.16+). — source: engine-reference
- **Never use `commands.entity(e).set_parent(p)`.** Use `commands.entity(e).insert(ChildOf(p))` (Bevy 0.16+). — source: engine-reference
- **Never use `Parent` component.** Use `ChildOf` (Bevy 0.16+). — source: engine-reference
- **Never use `commands.entity(e).despawn_recursive()`.** Use `commands.entity(e).despawn()` — recursive by default in Bevy 0.16+. — source: engine-reference
- **Never use `Color::rgba()`.** Use `Color::srgba()`. — source: engine-reference
- **Never render board units/objectives/prisms/HP bars/spawn range as bevy_ui nodes.** World-space sprites only. — source: ADR-021

### Performance Guardrails

- **Presentation steady-state**: < 1 ms per frame. Phase-boundary frame (toggle ~50 entities, cancel tweens): < 3 ms spike. — source: ADR-021

---

## Global Rules (All Layers)

### Naming Conventions

| Element | Convention | Example |
|---------|-----------|---------|
| Structs / Enums / Components / Events / Plugins | `PascalCase` | `CardUnit`, `AuctionBidEvent`, `GamePlugin` |
| Functions / Systems / Variables / Fields | `snake_case` | `resolve_combat`, `current_gold` |
| Constants / Statics | `SCREAMING_SNAKE_CASE` | `OBJECTIVE_HP`, `MAX_HAND_SIZE` |
| Files / Modules | `snake_case.rs` | `auction_system.rs`, `card_pool.rs` |
| Plugins | Suffix `Plugin` | `CombatPlugin`, `AuctionPlugin` |
| Systems | Verb\_noun pattern | `spawn_unit`, `resolve_lane_combat` |
| Resources | Noun, PascalCase | `GameConfig`, `CardPool`, `RoundState` |
| Lightyear C2S protocol types | Prefix `C2S` | `C2SPlaceUnit`, `C2SAuctionBid` |
| Lightyear S2C protocol types | Prefix `S2C` | `S2CRoundResolved`, `S2CPhaseChanged` |

### Performance Budgets

| Target | Value |
|--------|-------|
| Framerate | 60 FPS (browser/WASM) |
| Total frame budget | 16.67 ms |
| Client S2C processing + view update | ≤ 2 ms |
| Server steady-state game logic | ≤ 5 ms |
| Server RESOLUTION batch (worst case) | ≤ 15 ms |
| WASM bundle size (release + LTO + strip) | < 50 MB |
| WASM heap | < 256 MB |
| Per-round message (including replication deltas) | < 1 KB/round/player |
| Reconnect snapshot | < 16 KB (hard ceiling: 32 KB) |

### Approved Libraries / Addons

| Crate | Version | Purpose |
|-------|---------|---------|
| `bevy` | `0.18` | Core engine |
| `lightyear` | `0.26.0` | Multiplayer networking (Bevy 0.18 compatible) |
| `bevy_tweening` | `0.18` | UI and movement animations |
| `bevy_asset_loader` | verify on crates.io | Typed asset loading / loading states |
| `rand` | `0.9` | Server-side seeded RNG — **server crate only** |
| `rand_chacha` | `0.3` | Deterministic seeded RNG (`ChaCha20Rng`) — **server crate only** |
| `serde` + `serde_json` | latest | Card data serialisation (JSON card pool files) |
| `ron` | `0.8` | Config files (`GameConfig`, card definitions) |
| `trunk` | latest | WASM build + dev server |
| `wasm-bindgen` | latest | WASM/JS boundary (if needed for browser APIs) |

> `rand` and `rand_chacha` must NOT appear in `client/Cargo.toml` for gameplay. `bevy_egui` is dev/debug only — never in shipped builds.

### Forbidden APIs (Bevy 0.18 + Lightyear 0.26)

These APIs are deprecated, removed, or do not exist. Using them produces a compile error or silent runtime failure.

| Forbidden | Use instead | Since |
|-----------|-------------|-------|
| `SpriteBundle { .. }` | `Sprite::from_image(..)` + `Transform` | Bevy 0.15 |
| `Camera2dBundle { .. }` | `Camera2d` + `Transform` | Bevy 0.15 |
| `NodeBundle { .. }` | `Node { .. }` | Bevy 0.15 |
| `TransformBundle { .. }` / `SpatialBundle { .. }` | `Transform` / `Transform` + `Visibility` | Bevy 0.15 |
| Manual `GlobalTransform` insert | Auto-inserted by `Transform` Required Component | Bevy 0.15 |
| `Handle<TextureAtlas>` | `Handle<TextureAtlasLayout>` inside `Sprite.texture_atlas` | Bevy 0.15 |
| `Color::rgba(..)` | `Color::srgba(..)` | Bevy 0.15 |
| `query.single()` panicking form | `let Ok(x) = query.single()` (returns `Result`) | Bevy 0.16 |
| `query.get_single()` | `query.single()` | Bevy 0.16 |
| `event_writer.send(e)` | `event_writer.write(e)` | Bevy 0.16 |
| `UiImage::new(handle)` | `ImageNode::new(handle)` | Bevy 0.16 |
| `commands.entity(e).set_parent(p)` | `commands.entity(e).insert(ChildOf(p))` | Bevy 0.16 |
| `Parent` component | `ChildOf` component | Bevy 0.16 |
| `commands.entity(e).despawn_recursive()` | `commands.entity(e).despawn()` | Bevy 0.16 |
| `commands.entity(e).despawn_descendants()` | `commands.entity(e).despawn_related::<Children>()` | Bevy 0.16 |
| `EventWriter<T>` / `EventReader<T>` / `Events<T>` | `MessageWriter<T>` / `MessageReader<T>` (buffered) or Observer (reactive) | Bevy 0.17 |
| `app.add_event::<T>()` | `app.add_message::<T>()` (buffered) or `app.observe(handler)` (reactive) | Bevy 0.17 |
| `TextFont { line_height: .. }` | `LineHeight` as separate Required Component | Bevy 0.18 |
| `entity.row()` | `entity.index()` | Bevy 0.18 |
| `Entities::flush()` | Removed — use `World::spawn()` | Bevy 0.18 |
| `ron` from `bevy_asset`/`bevy_scene` | Add `ron = "0.8"` directly to `Cargo.toml` | Bevy 0.18 |
| `AssetLoader` without `TypePath` | Add `#[derive(TypePath)]` to loader struct | Bevy 0.18 |
| `#[reflect[..]]` or `#[reflect{..}]` | `#[reflect(..)]` parentheses only | Bevy 0.18 |
| Cargo feature `bevy_ui_picking_backend` | `ui_picking` | Bevy 0.18 |
| Cargo feature `bevy_sprite_picking_backend` | `sprite_picking` | Bevy 0.18 |
| `ReplicateTo` component (Lightyear) | `Replicate::to_clients(NetworkTarget::All)` | Lightyear 0.26 |

Source: `docs/engine-reference/bevy/deprecated-apis.md`, `docs/engine-reference/bevy/current-best-practices.md`

### Cross-Cutting Constraints

- **No `unwrap()` in production paths.** Use `?` propagation or `expect("descriptive message")`. — source: technical-preferences.md
- **No hardcoded balance values in systems.** All tuning knobs go through `Res<GameConfig>` loaded from `assets/config/game_config.ron`. — source: technical-preferences.md, ADR-004
- **No `bevy_egui` in shipped builds.** Egui is dev/debug only. All shipped UI uses bevy_ui. — source: technical-preferences.md
- **No game state on client.** Client is a read-only view; `ClientState` mutates only via inbound S2C. — source: ADR-002
- **No client-side RNG for gameplay.** All randomness server-side via `ServerRng`; `rand_chacha` must not appear in `client/Cargo.toml`. — source: ADR-005
- **`Lightyear MessageReceiver<T>` can only be drained once per frame.** First system to drain consumes all messages. Register exactly one drainer per C2S message type (server) and per S2C message type (client). — source: ADR-013, ADR-021
- **Bevy Required Components API (0.15+): never use Bundle structs.** Spawn with `commands.spawn((Component1, Component2, ...))`. Required components (`GlobalTransform`, `Visibility`, `InheritedVisibility`) are auto-inserted. — source: ADR-020, ADR-021, engine-reference
- **All tests against Bevy systems use `World::new()`.** No live Lightyear session required for unit and most integration tests. No mocks — test against real ECS `World` state. — source: technical-preferences.md
- **Activate `liv-bevy-018` skill on every `.rs` file importing `bevy`.** Prevents pre-0.15 API patterns. — source: `docs/engine-reference/bevy/VERSION.md`
- **Activate `liv-bevy-lightyear` skill on every `.rs` file importing `lightyear`.** Enforces Lightyear 0.26 patterns. — source: `docs/engine-reference/bevy/VERSION.md`

### Bevy 0.18 Message vs Observer — Quick Reference

| Need | Pattern | Registration |
|------|---------|-------------|
| Buffered inter-system signal (game loop, recurring) | `#[derive(Message)]` + `MessageWriter<T>` / `MessageReader<T>` | `app.add_message::<T>()` |
| One-shot reactive lifecycle trigger | `#[derive(Event)]` + `world.trigger_targets(event, entity)` + observer handler `fn(trigger: On<T>, ...)` | `app.observe(handler_fn)` |
| Lightyear C2S inbound (server receives from client) | `MessageReceiver<T>` with `.receive()` | Via Lightyear protocol registration |
| Lightyear C2S outbound (client sends to server) | `MessageSender<T>` with `.send::<ReliableChannel>(msg)` or `.send::<UnreliableChannel>(msg)` | Via Lightyear protocol registration |
| Lightyear S2C outbound (server sends to clients) | `ServerMultiMessageSender::send::<M, C>(&msg, server, &NetworkTarget::Single(peer_id) / NetworkTarget::All)` | Via Lightyear protocol registration |

> **Critical boundary**: Bevy's `MessageReader<T>` (internal bus, registered via `app.add_message`) and Lightyear's `MessageReceiver<T>` (network layer, registered via Lightyear protocol) are DISTINCT APIs. Using the wrong one compiles but produces no messages at runtime.
