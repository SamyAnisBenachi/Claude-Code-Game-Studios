# Control Manifest

> **Engine**: Bevy 0.18 + Lightyear 0.26
> **Last Updated**: 2026-05-01
> **Manifest Version**: 2026-05-01
> **ADRs Covered**: ADR-001, ADR-002, ADR-003, ADR-004, ADR-005, ADR-006, ADR-007, ADR-008, ADR-009, ADR-010, ADR-011, ADR-012, ADR-013, ADR-014, ADR-015, ADR-016, ADR-017, ADR-019, ADR-020, ADR-021, ADR-022
> **Status**: Active — regenerate with `/create-control-manifest update` when ADRs change

`Manifest Version` is the date this manifest was generated. Story files embed
this date when created. `/story-readiness` compares a story's embedded version
to this field to detect stories written against stale rules. Always matches
`Last Updated` — they are the same date, serving different consumers.

This manifest is a programmer's quick-reference extracted from all Accepted ADRs,
technical preferences, and engine reference docs. For the reasoning behind each
rule, see the referenced ADR.

> **Note**: ADR-018 (Keyword System ECS State Architecture) has status **Proposed** and is excluded.
> The `SimpleKeyword` / `Keyword` enum amendments it describes are already embedded in ADR-006 as inline amendments.
> Re-run `/create-control-manifest update` after ADR-018 is Accepted.

---

## Foundation Layer Rules

*Applies to: Cargo workspace layout, asset loading pipeline, Lightyear channel configuration,
Round State Machine phase state, RSM event bus, session lifecycle, reconnect protocol.*

### Required Patterns

- **Three-crate Cargo workspace: `shared/`, `server/`, `client/`.** `shared/` = protocol types only; `server/` = headless binary; `client/` = WASM binary. — source: ADR-003
- **`shared/` Cargo features: `bevy = { default-features = false, features = ["serialize"] }` and Lightyear `shared` feature only.** No `bevy_ecs`, `bevy_render`, `bevy_ui`, `tokio`, or server-only Lightyear features. — source: ADR-003
- **`server/` Cargo features: headless Bevy (`multi_threaded` only), Lightyear `server` + `websocket`.** `rand` and `rand_chacha` server-only. — source: ADR-003
- **`client/` Cargo features: `bevy_ui`, `bevy_sprite`, `bevy_text`, `bevy_asset`, `bevy_winit`, `webgl2`, Lightyear `client` + `websocket`.** — source: ADR-003
- **Lightyear protocol registration lives in one function `pub fn register_protocol(app: &mut App)` in `shared/src/protocol.rs`.** Both `server/main.rs` and `client/main.rs` call it at startup. — source: ADR-003
- **`GameConfig` struct in `shared/config.rs` is a plain serde struct — NO `#[derive(Resource)]`.** Server wraps it via `commands.insert_resource(GameConfig::load(...))`. Client inserts independently if needed for UI. — source: ADR-003, ADR-004
- **Server internal layering (code-review enforced, not compile-enforced): `feature/` may import from `core/`; `core/` may import from `foundation/`. Reverse direction forbidden.** — source: ADR-003
- **Client internal layering: `ui/` → `state/` → `network/` → `shared/`. Reverse direction forbidden.** — source: ADR-003
- **Workspace `Cargo.toml` release profile: `lto = "thin"`, `codegen-units = 1`, `strip = "symbols"`, `panic = "abort"`.** Dev profile: `opt-level = 1`. — source: ADR-003
- **Adding a dependency to `shared/` requires an ADR amendment and technical-director approval.** — source: ADR-003
- **Asset loading: use `bevy_asset_loader` `LoadingState` for `GameConfig` (RON) and `CardCatalog` (JSON).** Both loaded before transitioning to `AppState::Lobby`. — source: ADR-004
- **Every `impl AssetLoader` struct must `#[derive(Default, TypePath)]`.** Missing `TypePath` produces a confusing runtime error. — source: ADR-004
- **`ron` must be a direct crate dependency in `server/Cargo.toml`.** It is no longer re-exported from `bevy_asset` in 0.18. — source: ADR-004
- **On fatal load error call `AppExit::Error(NonZeroU8::MIN)` — not `panic!`.** Lets Bevy shut cleanly and surface a non-zero exit code to the deployment platform. — source: ADR-004
- **Hot-reload for `GameConfig` in debug builds only: gate the `add_systems` call itself behind `#[cfg(debug_assertions)]`.** Not just the function body. — source: ADR-004
- **Exactly 2 Lightyear channels: `ReliableChannel` (ordered, guaranteed) and `UnreliableChannel` (best-effort).** Channel definitions in `shared/src/protocol.rs`. Channel assignment is permanent per message type — never switch at runtime. — source: ADR-008
- **`S2CResolutionEvent` MUST be enqueued before `S2CPhaseChanged(DRAFT_SHOP)` in the same server frame (OQ-D invariant).** Enforced by same-channel enqueue order on `ReliableChannel`. — source: ADR-008
- **Default channel for any new message type: `ReliableChannel`.** Only use `UnreliableChannel` when: dropped packet is immediately superseded, stale arrival causes no state corruption, sent more than once per phase. — source: ADR-008
- **Per-connection `snapshot_sent: bool = false` flag set on `OnConnected`.** Snapshot system scheduled BEFORE all live-game message systems. No live S2C message may be enqueued to a reconnecting client before `snapshot_sent = true`. — source: ADR-008, ADR-011
- **`RoundState` resource is the single source of truth for phase on the server.** Only `rsm_tick_system` holds `ResMut<RoundState>`. All other systems use `Res<RoundState>`. — source: ADR-009
- **`rsm_tick_system` scheduled AFTER `AuctionSystem` and `CombatResolutionSystem` in the `Update` schedule.** — source: ADR-009
- **Phase gate pattern in every C2S handler:** `if round_state.phase != X { return; }` — silent discard, no error to client. — source: ADR-002, ADR-009
- **Use `#[derive(Message)]` + `MessageWriter<T>` / `MessageReader<T>` + `app.add_message::<T>()` for all RSM buffered signals.** `EventWriter`/`EventReader`/`Events<T>` do not exist in Bevy 0.17+. — source: ADR-009, ADR-010
- **`SessionReady` uses `#[derive(Event)]` + `commands.trigger(SessionReady)` (Observer) — NOT `app.add_message::<SessionReady>()`.** It is a one-shot lifecycle trigger, not a recurring game-loop message. — source: ADR-010, ADR-012
- **RSM has zero imports from `server/feature/`.** All phase-reactive logic is triggered by events, not direct function calls. — source: ADR-010
- **F2 emission order in `advance_phase` is strict code order:** 1. `DraftStarted`, 2. `ShopRefreshTriggered` (per player), 3. `AuctionPhaseEntered` (auction rounds only), 4. `BroadcastPhaseChanged` (ALWAYS LAST). — source: ADR-010
- **All subscriber systems scheduled `.after(advance_phase)`.** A subscriber scheduled before `advance_phase` will miss the current frame's messages. — source: ADR-010
- **GSS command order before triggering `SessionReady`:** `commands.insert_resource(SessionConfig)` → `commands.insert_resource(ServerRng)` → `commands.trigger(SessionReady)`. Never reorder. — source: ADR-012
- **Register exactly one Observer for `SessionReady` — the RSM's `on_session_ready` handler.** Other systems react to `DraftStarted` from the RSM, not directly to `SessionReady`. — source: ADR-012
- **`evaluate_session_ready` runs only while in LOBBY state.** Gate on `LobbyState::GameActive` to prevent re-trigger. — source: ADR-012
- **Mandatory reconnect send order (all `ReliableChannel`, all unicast):** 1. `S2CHandshake`, 2. `S2CGameSnapshot`, 3. `S2CObjectiveIdentities`, 4. `S2CPhaseChanged`. Then set `snapshot_sent = true` and flush deferred queue. — source: ADR-011
- **`hello_timeout_ms` watchdog: 5000ms.** Close connection if no `C2SHello` received within window. Do NOT send `S2CHandshakeRejected` on timeout — silence only. — source: ADR-011

### Forbidden Approaches

- **Never add `client/Cargo.toml` dependency on `server`.** CI must fail any PR that introduces this edge. — source: ADR-003
- **Never use `#[cfg(feature = "server")]` to gate game logic in `protocol/` or `client/`.** If a type is server-only, it lives in `server/`. If shared, it lives in `protocol/`. — source: ADR-002, ADR-003
- **Never add `#[derive(Resource)]`, `Plugin` impls, or `App::add_systems` to `shared/`.** — source: ADR-003
- **Never add `rand` or `rand_chacha` to `client/Cargo.toml` for gameplay purposes.** — source: ADR-003, ADR-005
- **`CardCatalog` is NOT hot-reloaded.** Card data changes require a server restart. — source: ADR-004
- **Never split `S2CResolutionEvent` and `S2CPhaseChanged(DRAFT_SHOP)` onto different channels.** Would require explicit sequence numbers and client-side reorder buffer. — source: ADR-008
- **Never use `EventWriter<T>` / `EventReader<T>` / `Events<T>`.** These do not exist in Bevy 0.17+. Use `MessageWriter<T>` / `MessageReader<T>` (buffered) or Observers (reactive). — source: ADR-009, ADR-010
- **Never use `app.add_message::<SessionReady>()` or `EventReader<SessionReady>`.** `SessionReady` is an Observer event — it will never fire via `MessageReader`. — source: ADR-012
- **Never register more than one Observer for `SessionReady`.** Other systems must subscribe to `DraftStarted` instead. — source: ADR-012
- **Never send `S2CGameSnapshot` as a broadcast.** Always unicast per player with opponent secrets stripped. — source: ADR-011
- **Never skip re-sending `S2CObjectiveIdentities` on reconnect.** Lightyear reliable delivery only applies within one transport session — it is not replayed on reconnect. — source: ADR-011
- **Never use `ResMut<RoundState>` in any system other than `rsm_tick_system`.** — source: ADR-009
- **Never schedule a subscriber system before `advance_phase`.** — source: ADR-010

### Performance Guardrails

- **WASM bundle size**: < 50 MB (hard ceiling). CI gate fails above threshold. — source: ADR-003, technical-preferences.md
- **Server steady-state tick**: ≤ 5 ms. During RESOLUTION batch: ≤ 15 ms. — source: ADR-002
- **Per-round message budget**: < 1 KB/round/player including replication deltas. Snapshot on reconnect: < 16 KB (hard ceiling 32 KB). — source: ADR-002, ADR-011

---

## Core Layer Rules

*Applies to: client-server authority, objective identity, server RNG, card data schema,
auction state, class system, card acquisition, economy, board/lane state.*

### Required Patterns

- **Server is the sole authority over all game state. Client is a read-only view.** All C2S inputs are validated server-side. Invalid inputs are silently discarded — no error response to client. — source: ADR-002
- **All C2S handlers route through a single entry point.** Pattern: 1. Resolve `ClientId` → `PlayerId` (unknown sender → log + drop, never panic). 2. Phase gate (silent discard). 3. Domain validation (silent discard). 4. Apply to authoritative state. 5. Broadcast/unicast S2C as required. — source: ADR-002
- **No optimistic client updates.** `ClientState` mutates only via inbound S2C messages — never from local user input. — source: ADR-002
- **On `OnConnected`: server sends `S2CGameSnapshot` before any other S2C.** Use the `snapshot_sent` flag mechanism (see Foundation). — source: ADR-002
- **Server tick is the wall clock.** Client timer display is derived from `S2CPhaseChanged.deadline_server_ms` for presentation only. — source: ADR-002
- **`ObjectiveIdentity { is_fake: bool }` is NEVER inserted into the Lightyear replication graph.** Server holds `HiddenObjectives` as a server-only resource, never replicated. — source: ADR-001
- **At `DRAFT_INITIAL`: send `S2CObjectiveIdentities` via `NetworkTarget::Single(owner_client_id)` on `ReliableChannel` — one unicast per player.** — source: ADR-001
- **`Sang Méprise` reveal: targeted unicast `S2CSangMepriseReveal` to the opponent only.** Never broadcast. — source: ADR-001
- **`HiddenObjectives` wiped and re-populated at each new session.** Never carried across sessions. — source: ADR-001
- **Single `ServerRng` resource backed by `ChaCha20Rng` from `rand_chacha 0.3`.** Seeded once from `OsRng` at session start. Inserted immediately before `SessionReady`; removed on `GameOverEmitted`. — source: ADR-005
- **All RNG access via intent-named methods on `ServerRng` — never raw `next_u32`/`gen` access.** Every consumption must push an `AuditEntry` in the same call. — source: ADR-005
- **Strict RNG consumption order in RESOLUTION chain (ADR-005 §4):** Orders 4–10: `RangeEquidistantSelect` → `TeleportRandomDest` → `StrichChangeLaneSelect` → `ResolveEcaflip` → `ResolvePrism` → `AwardFakeObjectiveReward` → `DrawFreeCard`. Any new consumer requires a new `RngEvent` variant + ADR amendment before implementation. — source: ADR-005
- **Inter-player ordering for concurrent RNG events:** ascending `player_id` → ascending `lane_index` → ascending `cell`. — source: ADR-005
- **`CardCatalog` (immutable, server-lifetime) = `HashMap<CardId, CardData>` from `assets/data/cards.json`.** `PlayerPool` (mutable, session-scoped) = per-player copy counts and shop slots. — source: ADR-006
- **`EPIC_POOL_COPIES = 1` and `LEGENDARY_POOL_COPIES = 1` are compile-time constants — NOT `GameConfig` fields.** — source: ADR-006
- **`PlayerPool::distribute()` is the sole pool mutation.** Returns `Err(DistributeError::Exhausted)` at 0 copies — never underflows below 0. — source: ADR-006
- **`total_acquired = initial_count - copies_remaining`.** No separate tracking field needed. — source: ADR-006
- **All draw functions return `Option<T>`.** Never panic on exhausted pool. — source: ADR-006
- **`Keyword` enum: use `#[serde(tag = "kw", content = "val")]` (adjacent tagging).** Serializes as `{ "kw": "Simple", "val": "Shield" }` for `Simple(SimpleKeyword)`. Do NOT use `#[serde(untagged)]` — fails at runtime for newtype variants with scalar inner type. — source: ADR-006
- **`SimpleKeyword::Haste` (not `Charge`) is the combat keyword removing summoning sickness.** `cards.json` must use `"Haste"`. Any fixture using `Charge` must be updated. — source: ADR-006
- **`AuctionState` resource: `AuctionState::default()` starts in `AuctionPhase::Idle`.** Only `auction_tick_system` holds `ResMut<AuctionState>`. — source: ADR-013
- **Per-frame code order in `auction_tick_system`:** 1. Handle `AuctionPhaseEntered` (IDLE → LIVE_BIDDING). 2. Handle `AbortAuction` (cleanup → IDLE, no `AuctionSettled`). 3. Drain `MessageReceiver<C2SAuctionBid>`. 4. `saturating_sub` timer. 5. If `timer == 0`: settle → write `MessageWriter<AuctionSettled>` → IDLE. — source: ADR-013
- **Release-before-reserve invariant (atomic):** `api::release_gold_reservation(prev_leader)` THEN `api::reserve_gold(new_leader, amount)` — sequential in same function body, no system boundary between them. — source: ADR-013
- **Timer cast: use `u32::try_from(time.delta().as_millis()).unwrap_or(u32::MAX)` — NOT `as u32`.** — source: ADR-013
- **`auction_tick_system` scheduled BEFORE `rsm_tick_system`.** `AuctionSet::Tick.before(RsmSet::Tick)`. — source: ADR-013
- **`MessageReceiver<T>` (Lightyear C2S network) vs `MessageReader<T>` (Bevy internal bus) are distinct APIs.** Do NOT use `MessageReader<C2SAuctionBid>` for a Lightyear C2S message. — source: ADR-013
- **`PlayerSessions` resource owns session-identity state only: `class: ClassId` and `class_locked: bool` per player.** Economy state → `PlayerEconomies`. Hand state → `PlayerHands`. Do NOT add other fields. — source: ADR-014
- **Import `ClassId` from `shared::card::ClassId` — NEVER redefine it.** — source: ADR-014
- **`C2SClassChoice` derives Lightyear's `Message` trait (`lightyear::prelude::Message`), NOT Bevy's `Message` trait (`bevy::prelude::Message`).** Both exist in this project simultaneously. — source: ADR-014
- **LOBBY→DRAFT_INITIAL gate: `all_classes_chosen()` must return true before `lock_all_classes()` is called.** `ClassId::Neutral` as player class → silent discard. `class_locked == true` → silent discard. — source: ADR-014
- **`SourceClass(ClassId)` component is set at token entity spawn time and NEVER mutated.** Absent on non-token units. — source: ADR-014
- **Class effects in RESOLUTION are plain Rust functions called from within the RESOLUTION system body.** NOT standalone Bevy systems. NOT buffered Messages within a RESOLUTION tick (frame-delayed → violates sub-step ordering). — source: ADR-014
- **`ShopStates` resource sole writer: `card_acquisition_tick_system`.** `PlayerHands` resource: written by CA in DRAFT, Prism/Objective in RESOLUTION — exclusive by phase, no concurrent conflict. — source: ADR-015
- **CA18 mandatory rollback:** `spend_gold` → `distribute` → on `Err(Exhausted)`: `refund_gold` — sequential, no system boundary. Gold must never remain deducted after a failed distribute. — source: ADR-015
- **`displayed_this_draft` NOT cleared on `ShopUnlock` trigger.** Dedup history accumulates across DRAFT_AUCTION + DRAFT_SHOP. Reset on new DRAFT_INITIAL entry only. — source: ADR-015
- **`card_acquisition_tick_system` scheduled AFTER `rsm_tick_system`.** `CardAcquisitionSet::Tick.after(RsmSet::Tick)`. — source: ADR-015
- **All `PlayerEconomy` field mutations go through `server/src/core/economy/api.rs` functions.** Direct field assignment (`.gold =`, `.current_mana =`, etc.) outside `api.rs` is forbidden. — source: ADR-019
- **`on_resolution_complete` reads `MessageReader<ResolutionComplete>` (NOT `ResolutionPhaseEntered`).** Snapshot must be taken AFTER all kill/objective gold awards in `resolve_combat`. — source: ADR-019
- **`on_resolution_complete` scheduled `.before(rsm_input_reader)`.** Interest snapshot must precede RSM DRAFT transition. — source: ADR-019
- **`PendingResolutionComplete(bool)` bridge resource.** `resolve_combat` (exclusive system) sets `world.resource_mut::<PendingResolutionComplete>().0 = true`. `drain_pending_resolution_complete` (regular system, `CombatSystemSet::PostResolution`) reads it and emits `MessageWriter<ResolutionComplete>`. `MessageWriter<T>` cannot be accessed from an exclusive system via `world.resource_mut()`. — source: ADR-019
- **`ResMut<PlayerEconomies>` write access restricted to five designated systems:** `initialise_player_economies`, `on_draft_started`, `on_resolution_complete`, `auction_tick_system`, `resolve_combat`. — source: ADR-019
- **Board state is a hybrid: ECS entities (Lightyear-replicated) + `BoardState` resource (spatial index).** All board mutations through `board/api.rs` — both entity components AND `BoardState` index updated atomically. — source: ADR-020
- **Unit entities spawned WITHOUT `Replicate` component at first.** Add `Replicate::to_clients(NetworkTarget::All)` ONLY AFTER `S2CPlacementReveal` is enqueued (simultaneous-reveal invariant from ADR-007). — source: ADR-020
- **Correct Lightyear 0.26 replication API: `Replicate::to_clients(NetworkTarget::All)`.** `ReplicateTo` does NOT exist in Lightyear 0.26.0. — source: ADR-020
- **Movement formula F1:** `new_cell = clamp(current_cell + direction × mp, 1, 8)` using `i16` intermediate arithmetic to prevent u8 overflow. — source: ADR-020
- **`remove_unit_from_board` removes from `BoardState` index but does NOT despawn the entity.** Caller explicitly despawns via `commands.entity(e).despawn()`. — source: ADR-020
- **`expand_spawn_range(state, player)` clamped at 2.** Maximum fakes destroyed = 2. — source: ADR-020

### Forbidden Approaches

- **Never `panic!` on invalid C2S input.** Use `tracing::debug!` for rejection logs. Return silently. — source: ADR-002
- **Never replicate `ObjectiveIdentity` as an ECS component.** Entity-split + `ReplicationGroup` workaround also rejected — same silent-leak risk. — source: ADR-001
- **Never use `rand::thread_rng()`, `StdRng`, or `SmallRng` in server game logic.** — source: ADR-005
- **Never transmit seed bytes, `seed_index`, or `audit_log` entries in any production S2C message.** — source: ADR-005
- **Never add a new RNG consumer without registering it in ADR-005 §4 consumption order table first.** — source: ADR-005
- **Never mutate `CardCatalog` after server startup.** — source: ADR-006
- **Never use `SimpleKeyword::Charge` in code or `cards.json`.** Renamed to `Haste`. — source: ADR-006
- **Never use `ResMut<AuctionState>` in any system other than `auction_tick_system`.** — source: ADR-013
- **Never drain `MessageReceiver<C2SAuctionBid>` in more than one system.** First drain consumes all messages. — source: ADR-013
- **Never use cross-frame message passing for release/reserve gold.** Breaks the simultaneous-reservation invariant. — source: ADR-013
- **Never add economy or hand fields to `PlayerSessionData`.** — source: ADR-014
- **Never use `ClassId::Neutral` as a valid player class in DRAFT or later phases.** — source: ADR-014
- **Never drain `MessageReceiver<C2SClassChoice>` in more than one system.** — source: ADR-014
- **Never clear `displayed_this_draft` on `ShopUnlock` trigger.** — source: ADR-015
- **Never use cross-frame message passing for the spend/refund pair (CA18).** — source: ADR-015
- **Never use `ResMut<ShopStates>` in any system other than `card_acquisition_tick_system`.** — source: ADR-015
- **Never use `on_resolution_phase_entered` as the interest snapshot trigger.** Fires too early — kills/objective gold not yet awarded. — source: ADR-019
- **Never use `MessageWriter<T>` as a system param inside an exclusive system.** Use the `PendingResolutionComplete` bridge resource instead. — source: ADR-019
- **Never mutate `BoardPosition` component or `BoardState` index entries directly outside `board/api.rs`.** — source: ADR-020
- **Never add `Replicate` component to a unit entity before `S2CPlacementReveal` is enqueued.** Breaks simultaneous-reveal invariant. — source: ADR-020
- **Never use `ReplicateTo` component.** It does not exist in Lightyear 0.26. — source: ADR-020
- **Never use `SpriteBundle`, `Camera2dBundle`, `NodeBundle`, `TransformBundle`, or any `*Bundle` type.** Deprecated in Bevy 0.15. Use Required Components API. — source: engine-reference

### Performance Guardrails

- **Client S2C processing + view update**: ≤ 2 ms per frame. — source: ADR-002
- **Server steady-state game logic**: ≤ 5 ms per frame tick. — source: ADR-002
- **`RoundState` lookup**: O(1) — single resource dereference, no query. — source: ADR-009
- **Board spatial query `get_units_at_cell`**: O(1) HashMap lookup regardless of unit count. — source: ADR-020

---

## Feature Layer Rules

*Applies to: placement buffer, prism system, combat resolution, keyword observer architecture.*

### Required Patterns

- **`PendingPlacements` resource holds pending submissions as plain Rust data — NO ECS entity spawn during PLACEMENT phase.** Unit entities only exist after sub-step 1 commit in `resolve_combat`. — source: ADR-007
- **Placement validation is all-or-nothing per player batch.** Any single card failure → discard entire batch silently. No partial acceptance. — source: ADR-007
- **Buffer cleared on `PlacementPhaseEntered` (not `PlacementPhaseExited`).** — source: ADR-007
- **`close_placement_phase` strict order (all in ONE system — never split):** 1. Collect `PendingPlacements`. 2. Deduct mana (Economy API). 3. Enqueue `S2CPlacementReveal` broadcast (`ReliableChannel`). 4. Spawn ECS unit entities. 5. Add `Replicate::to_clients(NetworkTarget::All)` to spawned entities. 6. Emit `PlacementCommitted`. 7. Clear buffer. — source: ADR-007
- **Mana deducted at PLACEMENT close time, not at submission receipt.** — source: ADR-007
- **Spawn range validation for Minions uses Formula F2.** Structures and Traps bypass spawn range check. — source: ADR-007
- **`PrismState` resource sole writer: `resolve_prism_draws`.** External systems read prism state via `PrismPresence` component replication — not via `PrismState` directly. — source: ADR-016
- **`PrismCollected` is a server-internal Bevy `#[derive(Message)]` — NOT a Lightyear C2S message.** Use `MessageReader<PrismCollected>`, NOT `MessageReceiver<PrismCollected>`. — source: ADR-016
- **`hand_push()` is a shared module function.** Both Prism System and Card Acquisition call it. Neither system holds `ResMut<PlayerHands>` simultaneously — CA is DRAFT-only, Prism is RESOLUTION-only. — source: ADR-016
- **`resolve_prism_draws` system registration:** `.after(resolve_ecaflip_triggers).before(award_fake_objective_rewards)`. — source: ADR-016
- **`resolve_combat` is an exclusive Bevy system: `fn resolve_combat(world: &mut World)`.** Bevy 0.18 auto-detects `&mut World`. All 6 sub-steps execute in a single frame invocation. — source: ADR-017
- **Stat snapshot taken at RESOLUTION entry; immutable for the algorithm run.** — source: ADR-017
- **`apply_combat_modifier_stack` is a pure function — no World access.** Testable without Bevy context. — source: ADR-017
- **Movement boundary:** Destination rule (Formula F1) governs Trap/Prism triggering — skips intermediate cells. Enemy collision tick-by-tick loop governs obstruction (WALL halt, path-crossing). These are complementary layers on sub-step 5. — source: ADR-017
- **`S2CResolutionEvent` is a single reliable broadcast sent AFTER all 6 sub-steps complete.** Clients receive the complete log and replay at animation tempo. — source: ADR-017
- **HP mutations use `saturating_sub`.** Never raw `u8 -= u8`. — source: ADR-017
- **`MessageWriter<BeginResolution>` / `MessageWriter<ResolutionComplete>` are Bevy-internal.** `S2CResolutionEvent` uses Lightyear's `MessageSender<T>`. Do not confuse them. — source: ADR-017
- **5 global Observers registered in `KeywordPlugin::build()`:** `on_unit_appeared`, `on_unit_died`, `on_final_blow_dealt`, `on_start_of_turn`, `on_end_of_turn`. — source: ADR-022
- **Every Observer handler MUST check keyword presence as its first operation (guard pattern) and `return` early if absent.** Global observers fire for ALL entities — guard is mandatory. — source: ADR-022
- **DEATH chain managed via `ChainDeathBuffer` VecDeque — NOT recursive `world.trigger_targets()` inside `on_unit_died`.** `ChainDeathBuffer` cleared at SS4 start (defensive: also cleared at SS4 end). Initial deaths seeded in lane order (Lane 1 first). — source: ADR-022
- **`world.trigger_targets(event, entity)` confirmed valid `World` method in Bevy 0.18.** Fires observers synchronously within the exclusive system call. — source: ADR-022
- **`Trigger<T>` is the correct observer handler parameter type.** NOT `On<T>` (doc inconsistency in breaking-changes.md — `Trigger<T>` is the stable canonical form). — source: ADR-022
- **COUNTERATTACK and INJURED use inline dispatch from `resolve_combat` — NOT Observers.** COUNTERATTACK requires proximity check before dispatch; INJURED is a state re-evaluation scan, not an event. — source: ADR-022
- **START OF TURN: normal Bevy system reads `MessageReader<DraftPhaseEntered>` → `commands.trigger_targets(StartOfTurnTriggered, entity)`.** `app.add_message::<DraftPhaseEntered>()` registered in RSM plugin (emitter), NOT in `KeywordPlugin`. — source: ADR-022
- **`app.add_message::<KeywordTriggered>()` MUST be registered in `KeywordPlugin`.** Missing registration panics at first `MessageWriter::write()`. — source: ADR-022

### Forbidden Approaches

- **Never spawn ECS entities for pending placements during PLACEMENT phase.** Lightyear replication would immediately broadcast opponent placement — breaks simultaneous-reveal mechanic. — source: ADR-007
- **Never split `S2CPlacementReveal` enqueue and entity spawn into two separate systems.** Broadcast-before-spawn is the structural invariant. — source: ADR-007
- **Never accept partial batch placement.** All-or-nothing per player only. — source: ADR-007
- **Never deduct mana at submission receipt time.** Deduct at PLACEMENT close. — source: ADR-007
- **Never use `MessageReceiver<PrismCollected>` (Lightyear).** It is a Bevy-internal message. Use `MessageReader<PrismCollected>`. — source: ADR-016
- **Never add `ResMut<PrismState>` to any system other than `resolve_prism_draws`.** — source: ADR-016
- **Never run Card Acquisition systems during RESOLUTION phase.** — source: ADR-016
- **Never break RESOLUTION into a multi-frame state machine.** — source: ADR-017
- **Never stream per-sub-step S2C events to clients.** One batch message only (`S2CResolutionEvent`). — source: ADR-017
- **Never apply raw `u8 -= u8` for HP mutations.** Use `saturating_sub`. — source: ADR-017
- **Never omit the guard check in an Observer handler.** Effects will silently fire for units that do not have the keyword. — source: ADR-022
- **Never use recursive `world.trigger_targets()` inside `on_unit_died`.** Use `ChainDeathBuffer` explicit queue. — source: ADR-022
- **Never use Observers for COUNTERATTACK or INJURED.** Inline dispatch only. — source: ADR-022
- **Never use `On<T>` as observer handler parameter type.** Use `Trigger<T>`. — source: ADR-022

### Performance Guardrails

- **Combat resolution (worst-case 5-lane contested round)**: ≤ 15 ms in a single exclusive system frame. Profile with `tracing::instrument` in the first vertical slice. — source: ADR-017
- **`resolve_combat` idle (no `BeginResolution` message)**: exits immediately — < 1 µs. — source: ADR-017

---

## Presentation Layer Rules

*Applies to: client-side rendering, UI, animations, asset sharing, phase synchronization.*

### Required Patterns

- **`PresentationPlugin` registration order is a contract — DO NOT reorder:** 1. `CardAnimationsPlugin`, 2. `BoardRenderingPlugin`, 3. `HandUiPlugin`, 4. `HudPlugin`, 5. `ShopAuctionUiPlugin`. Reordering causes runtime panics (Resource not yet inserted). — source: ADR-021
- **`PresentationSet` execution order:** `PhaseTransition` → `MessageDrain` → `StateSync` → `AnimationTick` (chained via `.chain()`). — source: ADR-021
- **Single `phase_sink_system` drains `MessageReceiver<S2CPhaseChanged>` (Lightyear).** All sub-plugins read `Res<CurrentClientPhase>` — never `MessageReceiver<S2CPhaseChanged>` directly. — source: ADR-021
- **Board content ALWAYS world-space:** `Sprite` + `Transform` with `Camera2d`. Board entities CANNOT appear above bevy_ui without custom render layers. — source: ADR-021
- **UI ALWAYS bevy_ui:** `Node` for HUD, hand fan, shop panels, auction. — source: ADR-021
- **Hand drag preview is a bevy_ui `Node`, NOT a world-space `Sprite`.** Preserves correct z-ordering above board during drag. — source: ADR-021
- **`CardAtlas` shared Resource: `Handle<Image>` + `Handle<TextureAtlasLayout>`.** Atlas sprite spawn pattern: `Sprite { texture_atlas: Some(TextureAtlas { layout, index }), .. }`. `Handle<TextureAtlas>` asset type does not exist in Bevy 0.18. — source: ADR-021
- **Tween cancel-and-replace via `Animator<T>::set_tweenable(new_tween)`.** Never despawn+respawn game-state entities mid-animation. Never write to `Transform` while `Animator<Transform>` is active (animator overwrites next frame). — source: ADR-021
- **Child entity Z: use local Z offset — NOT absolute world Z.** `local_z = target_world_z − parent_world_z`. `GlobalTransform` adds parent + child values. — source: ADR-021
- **`PickingBehavior` component only inside `#[cfg(feature = "ui_picking")]` guard.** Inserting without feature flag compiled panics at runtime. CI must include a build without `ui_picking`. — source: ADR-021
- **Pre-pool all HUD and hand fan entities at session start.** Toggle `Visibility` only — no mid-round spawn/despawn of these entities. — source: ADR-021
- **`BoardLayout` and `CardAtlas` are session-scoped Resources** (inserted on `OnEnter(ClientState::InSession)`, removed on `OnExit`). All systems reading them must be scoped to `in_state(ClientState::InSession)`. — source: ADR-021
- **`Color::srgba` / `Color::srgb` constructors.** `Color::rgba` was renamed with the linear/sRGB split in Bevy 0.15. — source: ADR-021, engine-reference
- **`SpriteAlphaLens::lerp()`: use `target.color.with_alpha(alpha)`.** `Color` has no mutating `set_alpha()` in Bevy 0.18. — source: ADR-021

### Forbidden Approaches

- **Never use any `*Bundle` type (`SpriteBundle`, `Camera2dBundle`, `NodeBundle`, etc.).** All deprecated in Bevy 0.15. Use Required Components API. — source: ADR-021, engine-reference
- **Never use `Handle<TextureAtlas>`.** Asset type removed in Bevy 0.18. Use `Handle<TextureAtlasLayout>` with `TextureAtlas` as a component field. — source: ADR-021, engine-reference
- **Never register `MessageReceiver<S2CPhaseChanged>` in more than one system.** First drain consumes all messages — other systems would silently miss them. — source: ADR-021
- **Never despawn+respawn entities for tween cancel-and-replace.** Discards `BoardPosition`, `UnitOwner`, `UnitKeywordState`, and all game-state components. — source: ADR-021
- **Never assign a child entity's `Transform.translation.z` to the intended world Z directly.** It will render at `parent_z + intended_world_z`. Use local offset. — source: ADR-021
- **Never use `UiImage::new()`.** Use `ImageNode::new()` (Bevy 0.16+). — source: engine-reference
- **Never use `commands.entity(e).set_parent(p)`.** Use `commands.entity(e).insert(ChildOf(p))` (Bevy 0.16+). — source: engine-reference
- **Never use `Parent` component.** Use `ChildOf` (Bevy 0.16+). — source: engine-reference
- **Never use `commands.entity(e).despawn_recursive()`.** Use `commands.entity(e).despawn()` — recursive by default in Bevy 0.16+. — source: engine-reference
- **Never use `Color::rgba()`.** Use `Color::srgba()`. — source: engine-reference
- **Never render board units/objectives/prisms/HP bars/spawn range as bevy_ui nodes.** World-space sprites only. — source: ADR-021

### Performance Guardrails

- **Presentation layer steady-state**: < 1 ms per frame. Phase-boundary frame (toggle ~50 entities, cancel tweens): < 3 ms spike. — source: ADR-021
- **Card atlas loaded once.** No per-system reload or duplicate GPU texture upload. `CardAtlas` shared via `Res<CardAtlas>`. — source: ADR-021

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
| Lightyear protocol types | Prefix `C2S` or `S2C` | `C2SPlaceUnit`, `S2CRoundResolved` |

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

- `bevy 0.18` — core engine
- `lightyear 0.26` — multiplayer networking (Bevy 0.18 compatible)
- `bevy_tweening 0.18` — UI and movement animations
- `bevy_asset_loader` — typed asset loading; verify 0.18-compatible version on crates.io before pinning
- `rand 0.9` — server-side RNG (server crate only)
- `rand_chacha 0.3` — deterministic seeded RNG (server crate only)
- `serde + serde_json` — card data serialisation
- `ron 0.8` — config files (`GameConfig`, card definitions)
- `trunk` — WASM build + dev server
- `wasm-bindgen` — WASM/JS boundary (if needed for browser APIs)

### Forbidden APIs (Bevy 0.18 + Lightyear 0.26)

These APIs are deprecated, removed, or do not exist. Using them produces a compile error or silent runtime failure.

| Forbidden | Use instead | Since |
|-----------|-------------|-------|
| `SpriteBundle { .. }` | `Sprite::from_image(..)` + `Transform` | Bevy 0.15 |
| `Camera2dBundle { .. }` | `Camera2d` + `Transform` | Bevy 0.15 |
| `NodeBundle { .. }` | `Node { .. }` | Bevy 0.15 |
| `TransformBundle { .. }` / `SpatialBundle { .. }` | `Transform` alone / `Transform` + `Visibility` | Bevy 0.15 |
| Manual `GlobalTransform` insert | Don't — auto-inserted by `Transform` Required Component | Bevy 0.15 |
| `query.single()` panicking form | `let Ok(x) = query.single()` (returns `Result`) | Bevy 0.16 |
| `query.get_single()` | `query.single()` | Bevy 0.16 |
| `event_writer.send(e)` | `event_writer.write(e)` | Bevy 0.16 |
| `EventWriter<T>` / `EventReader<T>` / `Events<T>` | `MessageWriter<T>` / `MessageReader<T>` (buffered) or `Observer` (reactive) | Bevy 0.17 |
| `commands.entity(e).set_parent(p)` | `commands.entity(e).insert(ChildOf(p))` | Bevy 0.16 |
| `Parent` component | `ChildOf` component | Bevy 0.16 |
| `commands.entity(e).despawn_recursive()` | `commands.entity(e).despawn()` (recursive by default) | Bevy 0.16 |
| `commands.entity(e).despawn_descendants()` | `commands.entity(e).despawn_related::<Children>()` | Bevy 0.16 |
| `UiImage::new(handle)` | `ImageNode::new(handle)` | Bevy 0.16 |
| `Handle<TextureAtlas>` | `Handle<TextureAtlasLayout>` with `TextureAtlas { layout, index }` as component field | Bevy 0.15 |
| `Color::rgba(..)` | `Color::srgba(..)` | Bevy 0.15 |
| `ron` from `bevy_asset` / `bevy_scene` | Add `ron = "0.8"` directly to `Cargo.toml` | Bevy 0.18 |
| `AssetLoader` without `TypePath` | Add `#[derive(TypePath)]` to loader struct | Bevy 0.18 |
| `TextFont { line_height: .. }` | `LineHeight` as separate Required Component | Bevy 0.18 |
| `entity.row()` | `entity.index()` | Bevy 0.18 |
| `#[reflect[..]]` or `#[reflect{..}]` | `#[reflect(..)]` parentheses only | Bevy 0.18 |
| Cargo feature `bevy_ui_picking_backend` | `ui_picking` | Bevy 0.18 |
| Cargo feature `bevy_sprite_picking_backend` | `sprite_picking` | Bevy 0.18 |
| `ReplicateTo` component (Lightyear) | `Replicate::to_clients(NetworkTarget::All)` | Lightyear 0.26 |

Source: `docs/engine-reference/bevy/deprecated-apis.md`, `docs/engine-reference/bevy/current-best-practices.md`

### Cross-Cutting Constraints

- **No `unwrap()` in production paths.** Use `?` propagation or explicit `expect()` with a descriptive message. — source: technical-preferences.md
- **No hardcoded balance values in systems.** All tuning knobs go through `GameConfig` resource loaded from `assets/config/game_config.ron`. — source: technical-preferences.md, ADR-004
- **No `bevy_egui` in shipped build.** Egui is dev/debug only. All shipped UI uses bevy_ui. — source: technical-preferences.md
- **No game state on client.** Clients are views. All authoritative state lives on the server. — source: ADR-002, technical-preferences.md
- **No client-side RNG for gameplay.** All randomness seeded and computed server-side; only results broadcast to clients. — source: ADR-005, technical-preferences.md
- **`Lightyear MessageReceiver<T>` can only be drained once per frame.** The first system to drain it consumes all messages. Register exactly one system per message type as the sole drainer. Applies on both server (C2S) and client (S2C). — source: ADR-013, ADR-021
- **Bevy Required Components API (0.15+): never use Bundle structs.** Spawn with `commands.spawn((Component1, Component2, ...))`. Required components (`GlobalTransform`, `Visibility`, `InheritedVisibility`) are auto-inserted. — source: ADR-020, ADR-021, engine-reference
- **All tests against Bevy systems use `World::new()`.** No live Lightyear session required for unit and most integration tests. — source: ADR-009, ADR-013, ADR-019
