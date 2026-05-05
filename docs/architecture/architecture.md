# Lanes and Lies — Master Architecture

## Document Status

| Field | Value |
|---|---|
| **Version** | 1.0 |
| **Last Updated** | 2026-04-29 |
| **Engine** | Bevy 0.18 + Lightyear 0.26 |
| **GDDs Covered** | card-data-pool, game-config, server-rng, economy-system, board-lane-system, round-state-machine, network-protocol, game-session-system, objective-system (9 M1 GDDs) |
| **ADRs Referenced** | ADR-001 through ADR-012 (12 total — all M1 decisions recorded) |
| **TR Coverage** | 74/74 M1 requirements covered |
| **Technical Director Sign-Off** | APPROVED 2026-04-29 (self-review — see Phase 7b below) |
| **Lead Programmer Feasibility** | Skipped — Lean mode (PHASE-GATE only) |

---

## Engine Knowledge Gap Summary

| Risk | Domain | Key Implication |
|---|---|---|
| HIGH | ECS / Spawning | Required Components (0.15) — no Bundles; `Query::single()` returns `Result` (0.16); `despawn()` replaces `despawn_recursive()` (0.16) |
| HIGH | Messages / Observers | `EventWriter`/`EventReader` removed (0.17) → `MessageWriter`/`MessageReader` + `app.add_message::<T>()` for buffered messages; `#[derive(Event)]` + Observer for one-shot triggers; `SessionReady` uses Observer per ADR-012 |
| HIGH | Lightyear 0.26 | Entire networking API is post-cutoff — unicast target shape, ReplicationState, channel ordering must be verified against docs.rs before implementing |
| HIGH | bevy_ui | `ImageNode` not `UiImage` (0.16); `LineHeight` as required component (0.18); `BorderRadius` inside `Node` field (0.18) |
| MEDIUM | Asset Loading | `AssetLoader` requires `#[derive(TypePath)]` (0.18); `ron` must be direct dep (0.18) |
| MEDIUM | Reflect | `#[reflect(..)]` parentheses only (0.18) |

**Mitigation:** `liv-bevy-018` skill enforces 0.18 ECS/UI/event patterns on every `.rs` file. `liv-bevy-lightyear` skill enforces Lightyear 0.26 patterns. All HIGH-risk Lightyear decisions carry "VERIFY BEFORE IMPLEMENTING" flags in their ADRs.

---

## 1. System Layer Map

### Cargo Workspace Structure

```
lanes-and-lies/
├── Cargo.toml              (workspace root)
├── shared/                 Foundation protocol types — zero Bevy plugin deps
│   └── src/
│       ├── lib.rs
│       ├── protocol.rs     C2S/S2C message types, channel definitions
│       ├── card.rs         CardData, CardId, Rarity, ClassId, CardType
│       └── config.rs       GameConfig struct (no #[derive(Resource)] here)
├── server/                 Headless Bevy app → Railway (Docker)
│   └── src/
│       ├── main.rs
│       ├── foundation/     Plugin setup, asset loading, RNG
│       │   ├── config.rs   Wraps GameConfig as Bevy Resource
│       │   └── rng.rs      ServerRng Resource + audit log
│       ├── core/           Phase orchestration + session
│       │   ├── rsm/        Round State Machine
│       │   ├── session/    Game Session System
│       │   ├── economy/    Economy System
│       │   └── pool/       Card Data & Pool
│       └── feature/        Game mechanics
│           ├── board/      Board/Lane System
│           ├── objective/  Objective System
│           ├── auction/    Auction System        [M2]
│           ├── acquisition/Card Acquisition      [M2]
│           ├── combat/     Combat Resolution     [M2]
│           ├── keyword/    Keyword System        [M3]
│           ├── prism/      Prism System          [M3]
│           └── class/      Class System          [M3]
└── client/                 WASM Bevy app → Vercel (Trunk)
    └── src/
        ├── main.rs
        ├── network/        Lightyear client plugin, message dispatch
        ├── state/          Client-side state mirror (read-only view)
        └── ui/             Presentation layer
            ├── board/      Board Rendering        [M2]
            ├── hand/       Hand UI                [M2]
            ├── shop/       Shop/Auction UI         [M2]
            ├── hud/        HUD                    [M2]
            └── anim/       Card Animations        [M3]
```

### Layer Diagram

```
┌────────────────────────────────────────────────────────────────┐
│  PRESENTATION  (client/ crate only)                            │
│                                                                │
│  Board Rendering · Hand UI · Shop/Auction UI · HUD    [M2]    │
│  Card Animations (bevy_tweening)                      [M3]    │
│                                                                │
│  ⚠️ HIGH: bevy_ui 0.18 — ImageNode, LineHeight component,     │
│     BorderRadius inside Node field, Required Components        │
├────────────────────────────────────────────────────────────────┤
│  FEATURE  (server/feature/ — game mechanics)                   │
│                                                                │
│  Board/Lane System · Objective System             [M1]         │
│  Auction System · Card Acquisition · Combat Res.  [M2]         │
│  Keyword System · Prism System · Class System     [M3]         │
│                                                                │
│  Rule: Feature systems subscribe to Core phase Messages.       │
│  RSM never calls Feature directly.                             │
│  ⚠️ HIGH: Observers for DEATH/APPEARANCE/FINAL BLOW —         │
│     Event/Observer 0.17+ split applies                         │
├────────────────────────────────────────────────────────────────┤
│  CORE  (server/core/ — session + phase orchestration)          │
│                                                                │
│  Round State Machine   Bevy States + phase Messages            │
│  Game Session System   Lobby FSM → SessionConfig Resource      │
│  Economy System        Gold / mana / reserve per player        │
│  Card Data & Pool      CardCatalog (immutable) +               │
│                        PlayerPool (mutable session state)      │
│                                                                │
│  ⚠️ HIGH: Lightyear 0.26 — broadcast, unicast,                 │
│     OnDisconnected, channel ordering                           │
├────────────────────────────────────────────────────────────────┤
│  FOUNDATION  (shared/ + server/foundation/)                    │
│                                                                │
│  GameConfig struct     Zero Bevy deps in shared/;              │
│                        server/ inserts as Bevy Resource        │
│  Server-side RNG       ChaCha20 Resource + audit log           │
│  Network Protocol      C2S/S2C types + channel defs (shared/)  │
│  Lightyear plugins     Server + client plugin setup            │
│                                                                │
│  ⚠️ MEDIUM: AssetLoader TypePath (0.18), ron direct dep        │
├────────────────────────────────────────────────────────────────┤
│  PLATFORM  (engine + runtime — not in source)                  │
│                                                                │
│  Bevy 0.18 · Lightyear 0.26 · WASM/WebSocket                  │
│  Trunk (client build) · Cargo (server build)                   │
│  Vercel (client deploy) · Railway/Docker (server deploy)       │
└────────────────────────────────────────────────────────────────┘
```

### RSM Communication Pattern (Architectural Rule)

The RSM is a pure orchestrator. It never holds direct references to Feature
systems and never calls them. Instead it emits Bevy Messages at phase
boundaries. Feature and Core systems subscribe:

```
RSM emits →                     Subscriber
─────────────────────────────────────────────────────
DraftStarted { round, phase }   Economy (mana ramp + gold income)
ShopRefreshNeeded { player }    Card Pool (draw shop slots)
AuctionPhaseEntered { round }   Auction System (start 20s timer)
ResolutionPhaseEntered          Combat Resolution (execute sub-steps)
PlacementPhaseEntered           Board/Lane System (open placement window)
GameOverEmitted { reason }      Game Session System (teardown)
```

This pattern means Core layer modules are dependency-ordered by data flow,
not by direct function calls. The RSM has zero `use feature::*` imports.

### Crate Dependency Rules

```
shared   ←  server   (server depends on shared)
shared   ←  client   (client depends on shared)
server   ✗  client   (never — no cross-binary dep)
client   ✗  server   (never)

Within server/:
  foundation  ←  core     (core depends on foundation)
  foundation  ←  feature  (feature depends on foundation)
  core        ←  feature  (feature depends on core)
  feature     ✗  core     (feature must not import core modules directly
                            — communicate via Messages only)
```

---

## 2. Module Ownership

### Foundation Layer

| Module | Owns | Exposes | Consumes |
|---|---|---|---|
| **GameConfig** | `assets/config/game_config.ron` parsed struct; validation rules; debug hot-reload watcher | `Res<GameConfig>` (read by all systems) | `bevy_asset_loader` (⚠️ TypePath derive required — 0.18); `ron` direct dep |
| **ServerRng** | `ChaCha20Rng` instance; `seed_index: u32`; `AuditLog: Vec<AuditEntry>` | `fn next_seed() → u64`; `Res<ServerRng>` | `rand_chacha 0.3`; seeded from `OsRng` at session start |
| **Network Protocol** | C2S/S2C message type definitions; channel type definitions; `SessionToken`; `PlayerId`, `LaneId`, `CardId` newtypes | All message types via `shared::protocol` (imported by both server + client) | Zero — pure data definitions; no Bevy plugins, no game logic |
| **Lightyear Plugins** | `LightyearServerPlugin` config (WebSocket port, transport); `LightyearClientPlugin` config (WebSocket URL) | Lightyear `Server`/`Client` resources; `OnConnected`, `OnDisconnected`; `MessageReceiver`, `MessageSender` | `GameConfig.protocol_version`, `GameConfig.hello_timeout_ms` |

**`shared/` purity constraint:** `GameConfig` and all protocol types in `shared/` compile with `bevy = { default-features = false, features = ["serialize"] }` only. No `Resource` derive in shared. Server wraps: `app.insert_resource(config)`.

---

### Core Layer

| Module | Owns | Exposes | Consumes |
|---|---|---|---|
| **Round State Machine** | `RoundPhase` (Bevy State); `round_number: u32`; `placement_timer`; `draft_shop_timer`; `disconnect_trackers: Map<PlayerId, f32>`; `submissions_received: Set<PlayerId>` | Phase Messages: `DraftStarted`, `AuctionPhaseEntered`, `PlacementPhaseEntered`, `ResolutionPhaseEntered`, `ShopRefreshNeeded`, `GameOverEmitted`; broadcasts `S2CPhaseChanged` with effective timer duration | `Res<GameConfig>` (base timers); `Res<SessionConfig>` (frozen placement timer multiplier); Lightyear `OnDisconnected`; `C2SSubmitPlacement`, `C2SHeartbeat` |
| **Game Session System** | `SessionSlot` vec; `session_id`, `room_code`; `lobby_deadline: f64`; `class_selections`; `heartbeat_trackers` (LOBBY only); placement timer multiplier requests before `SessionReady` | `Res<SessionConfig> { mode, player_count, team_map, class_map, placement_timer_multiplier_effective }` (read-only after `SessionReady`); `SessionReady` Message; `S2CSessionSettingsUpdated` neutral timer setting | `Res<GameConfig>` (lobby timeouts); Lightyear `OnConnected`, `OnDisconnected`; `C2SCreateRoom`, `C2SJoinRoom`, `C2SSelectClass`, `C2SConfirmClass`, `C2SSetPlacementTimerMultiplier`, `C2SHeartbeat` |
| **Economy System** | `PlayerEconomy { gold, current_mana, reserve_mana, mana_cap }` per player; `gold_snapshot` (interest base); `reserved_gold` per active bid | `fn validate_spend()`; `fn apply_spend()`; `fn apply_gold_award()`; `S2CGoldUpdate` (unicast); `S2CGoldBroadcast` (broadcast) | `Res<GameConfig>`; `DraftStarted` Message (mana ramp + income); `ResolutionPhaseEntered` (snapshot); `AuctionBidPlaced` event (reservation) |
| **Card Data & Pool** | `CardCatalog: HashMap<CardId, CardData>` (immutable, loaded at startup); `PlayerPool { copies_remaining: HashMap<CardId, u32> }` per player; `ShopSlots` per player | `fn draw_class_card()`; `fn draw_neutral()`; `fn draw_family_card()`; `fn acquire_card()`; `S2CShopSlots` (unicast); `S2CDraftOffering` (unicast) | `Res<GameConfig>` (pool counts, weights); `Res<ServerRng>`; `ShopRefreshNeeded` Message; `CardAcquired` event (from Feature) |

---

### Feature Layer (M1 systems)

| Module | Owns | Exposes | Consumes |
|---|---|---|---|
| **Board/Lane System** | `BoardGrid: [[Option<BoardCell>; 8]; 5]` per player; `PendingPlacements` buffer (not ECS entities until sub-step 1 commit); `SpawnRange` per player; `PrismState` per lane/player | `fn validate_placement()`; `S2CPlacementReveal` (broadcast); `OnResolutionEnd` Message; `PrismCollected` event | `Res<GameConfig>`; `PlacementPhaseEntered` + `ResolutionPhaseEntered` Messages; `C2SSubmitPlacement`; `fake_objectives_destroyed` (from Objective System) |
| **Objective System** | `ObjectiveState { hp, ar }` per lane/player (replicated via Lightyear); `HiddenObjectives { is_fake }` per lane/player (server-only Resource, **never** replicated — ADR-001); `real_objectives_destroyed`, `fake_objectives_destroyed` counters | `fn take_damage()`; `ObjectiveDestroyed` event; `S2CObjectiveIdentities` unicast (ADR-001); `fake_objectives_destroyed` counter | `Res<GameConfig>`; `Res<ServerRng>` (fake assignment + free-card draw); `ResolutionPhaseEntered`; emits `AwardGold` to Economy |

**M2/M3 Feature modules** (Auction, Card Acquisition, Combat Resolution, Keywords, Prisms, Class System) follow the same ownership template and will be fully specified in their governing ADRs when those GDDs are authored.

---

### Presentation Layer (client/ crate — M2+)

| Module | Owns | Exposes | Consumes |
|---|---|---|---|
| **Board Rendering** | Sprite entities for units, objectives, lanes, cells; health bar UI nodes | None (render only) | Client state mirror; `S2CPlacementReveal`, `S2CResolutionEvent` |
| **Hand UI** | Card fan layout; selected card state; play confirmation | `C2SSubmitPlacement` (sends to server) | Client state mirror; `S2CShopSlots`, `S2CDraftOffering` |
| **Shop/Auction UI** | Shop slot display; auction price/timer/leader display | `C2SShopRefresh`, `C2SAuctionBid` (sends to server) | Client state mirror; `S2CAuctionUpdate`, `S2CGoldUpdate` |
| **HUD** | Gold, mana, reserve display; round number; objective status dots | None | Client state mirror |
| **Card Animations** | `bevy_tweening` animation sequences for card draw, play, movement | None (visual only) | Board Rendering events; phase Messages |

⚠️ **HIGH RISK — entire Presentation layer:** All bevy_ui spawning must use 0.18 patterns. `liv-bevy-018` skill is mandatory on every client UI file.

---

### Engine API Risk Summary by Module

| Module | Engine APIs | Risk |
|---|---|---|
| RSM | `#[derive(States)]` forbidden; `MessageWriter::write()` (not EventWriter); Lightyear broadcast | HIGH |
| Game Session System | Lightyear `OnConnected`/`OnDisconnected`, `MessageReceiver` | HIGH |
| Economy System | `MessageWriter::write()`, `MessageReader::read()` (not EventWriter/EventReader) | MEDIUM |
| Card Data & Pool | `bevy_asset_loader`, `#[derive(TypePath)]`, `ron` direct dep | MEDIUM |
| Board/Lane System | `MessageWriter::write()` (not EventWriter), Lightyear broadcast | HIGH |
| Objective System | Lightyear unicast (ADR-001 pattern), `MessageWriter::write()` (not EventWriter) | HIGH |
| Presentation (all) | `Node`, `ImageNode`, `LineHeight`, `Text`, `TextFont` | HIGH |

---

## 3. Data Flow

### Flow 1: Startup / Initialisation Order

```
Server startup
│
├─1─ bevy_asset_loader loads game_config.ron
│    → GameConfig inserted as Res<GameConfig>
│    → Validation runs; fatal on failure
│
├─2─ cards.json loaded
│    → CardCatalog built (immutable HashMap)
│    → PlayerPool NOT yet created (session-scoped)
│
├─3─ Lightyear server plugin starts
│    → WebSocket listener open on configured port
│    → Awaiting C2SHello within hello_timeout_ms
│
└─4─ Server enters LOBBY state; RSM waits for SessionReady
     Game Session System active; all other Core/Feature systems idle
```

### Flow 2: Session Start (LOBBY → DRAFT_INITIAL)

```
Client A                    Server                      Client B
   │                           │                           │
   ├──C2SCreateRoom──────────►│                           │
   │                    creates session,                  │
   │                    room_code, lobby_deadline         │
   │◄─S2CRoomCreated──────────┤                           │
   │                           │◄────────C2SJoinRoom──────┤
   │                           │  validates slot           │
   │◄─S2CSlotUpdated───────────┤────S2CJoinAck────────────►│
   │                           │                           │
   ├──C2SConfirmClass──────────►│◄────C2SConfirmClass──────┤
   │◄─S2CClassLocked(own)──────┤────S2CClassLocked(own)───►│
   │                           │  all classes confirmed    │
   │◄─S2CClassesRevealed───────┤────S2CClassesRevealed────►│
   │                           │                           │
   │                    GSS fires SessionReady             │
   │                    ServerRng initialized              │
   │                    SessionConfig published            │
   │                    RSM enters DRAFT_INITIAL           │
   │                    ObjectiveSystem assigns fakes      │
   │◄─S2CObjectiveIdentities───┤  (unicast per player, ADR-001)
   │                           ├────S2CObjectiveIdentities►│
   │◄─S2CDraftOffering─────────┤────S2CDraftOffering──────►│
   │◄─S2CPhaseChanged──────────┤────S2CPhaseChanged───────►│
```

### Flow 3: Round Loop (Core game tick)

```
RESOLUTION ends
│
Server (RSM)
├─ RSM emits ResolutionPhaseEntered → Combat cleans up, Board emits OnResolutionEnd
├─ Economy snapshots gold for interest (before income)
├─ RSM evaluates win condition → GAME_OVER if ≥2 real objectives destroyed
│
├─ [if not GAME_OVER] RSM increments round_number
├─ RSM routes: is_auction_round(R)? → DRAFT_AUCTION : DRAFT_SHOP
├─ RSM emits DraftStarted { round, phase }
│   ├─ Economy receives → apply_mana_ramp + apply_gold_income
│   ├─ RSM emits ShopRefreshNeeded { player } → Card Pool draws shop slots
│   └─ [if auction round] RSM emits AuctionPhaseEntered { round }
│       └─ Auction System starts 20s timer, awaits C2SAuctionBid
│
├─ RSM broadcasts S2CPhaseChanged to all clients
│
DRAFT phase (player decisions)
│
├─ [DRAFT_SHOP] RSM starts draft_shop_timer (30s)
├─ All C2SReadySignal received OR timer expires → RSM emits PlacementPhaseEntered
│
PLACEMENT (10s base, extended by frozen session multiplier if applicable)
│
├─ Board/Lane validates and buffers placements (PendingPlacements — not ECS entities)
├─ All C2SSubmitPlacement received OR timer expires
├─ Board/Lane atomically commits buffer → S2CPlacementReveal broadcast to ALL
│
RESOLUTION (server-authoritative, no player input)
│
├─ Combat executes 6 sub-steps
├─ Each sub-step: S2CResolutionEvent broadcast (ordered, reliable channel)
├─ Objective damage processed → ObjectiveDestroyed events → AwardGold to Economy
├─ Combat emits ResolutionComplete → RSM transitions (loops back to top)
```

### Flow 4: Reconnect Flow

```
Client (reconnecting)           Server
│                                  │
├──transport connect───────────►│  │
├──C2SHello { session_token }──►│  token lookup: active session found?
│◄─S2CHandshake ────────────────┤
│◄─S2CGameSnapshot (unicast) ───┤  full state: phase, round, gold, board, hand
│◄─S2CObjectiveIdentities ──────┤  re-sent (reliable delivery not guaranteed across reconnect)
│◄─S2CPhaseChanged (current) ───┤
│                         Server unblocks: enqueues live messages
│                         only AFTER snapshot system completes
│◄─S2COpponentReconnected ──────┤──────────────────────────► broadcast to opponent
```

### Flow 5: C2S Message Routing (Phase-Gate Pattern)

```
Any C2S message arrives at server
│
├─ Phase gate: is this message valid for current RoundPhase?
│   valid   → route to owning system
│   invalid → silently discard (no error response to client)
│
├─ Player auth: does sender's ClientId map to an active session slot?
│   valid   → proceed
│   invalid → silently discard
│
└─ Owning system processes: Economy / Board / Auction / GSS
```

### Flow 6: Client State Mirror

The client never simulates. All state flows one direction: server → client.

```
S2CPhaseChanged ──────────────────► ClientPhase Resource
S2CGoldUpdate (unicast) ──────────► OwnPlayerState { gold, mana, reserve }
S2CGoldBroadcast ─────────────────► OpponentState { gold }
S2CShopSlots (unicast) ───────────► HandState { cards, shop_slots }
S2CPlacementReveal ───────────────► BoardState { units_by_lane }
S2CResolutionEvent (stream) ──────► BoardState updates (animated by Presentation)
S2CObjectiveIdentities (unicast) ─► OwnObjectiveCache { is_fake: [bool; 5] }
S2CGameSnapshot (on reconnect) ──► Full ClientState rebuild

Rules:
  - Client NEVER derives state from local simulation
  - Client NEVER sends state to server
  - All Presentation reads go through ClientState resources only
```

---

## 4. API Boundaries

### Boundary 1: Foundation → All (GameConfig + ServerRng)

```rust
// server/foundation/config.rs
// GameConfig loaded once, inserted as Bevy Resource after LoadingState.
// All systems read via Res<GameConfig>. No system caches field values.
// Invariant: Res<GameConfig> always present after LoadingState completes.

// server/foundation/rng.rs
#[derive(Resource)]
pub struct ServerRng {
    rng: ChaCha20Rng,
    seed_index: u32,
    audit_log: Vec<AuditEntry>,
}

impl ServerRng {
    /// Called ONLY in the consumption order from server-rng.md:
    /// DRAFT_INITIAL: assign_fake_objectives → draw_initial_draft
    /// RESOLUTION:    apply_placement_effects → resolve_ecaflip
    ///                → resolve_prism → award_fakes
    pub fn next_seed(&mut self, event_type: RngEvent) -> u64;
}

pub struct AuditEntry {
    pub event_type: RngEvent,
    pub seed_index: u32,
    pub result: Option<String>,
}

// Invariant: next_seed() is ONLY called server-side. Seeds NEVER sent to clients.
// Invariant: ServerRng destroyed on session teardown (GSS owns lifecycle).
```

### Boundary 2: Foundation → Core (Network Protocol types)

```rust
// shared/src/protocol.rs
// Must compile with bevy default-features=false, features=["serialize"] only.
// No Resource derive, no game logic, no Bevy plugins.

#[derive(Message, Serialize, Deserialize, Clone, Debug)]
pub struct C2SHello { pub protocol_version: u32, pub session_token: Option<SessionToken> }

#[derive(Message, Serialize, Deserialize, Clone, Debug)]
pub struct S2CPhaseChanged { pub phase: RoundPhase, pub round_number: u32, pub timer_duration_ms: Option<u32> }

#[derive(Message, Serialize, Deserialize, Clone, Debug)]
pub struct S2CGameSnapshot {
    pub phase: RoundPhase, pub round: u32,
    pub placement_timer_multiplier_effective: PlacementTimerMultiplier,
    pub own: PlayerSnapshot, pub opponent: OpponentSnapshot,
    pub board: BoardSnapshot,
}

// Channel definitions — registered in shared, used by both server and client plugins
pub struct ReliableChannel;    // ordered, guaranteed delivery
pub struct UnreliableChannel;  // best-effort, for high-frequency position updates

// ⚠️ HIGH RISK: `Message` trait is Lightyear 0.26. Verify derive macro name
//    and import path against docs.rs/lightyear/0.26 before implementing (OQ-A).
```

### Boundary 3: Core internal (RSM → phase Message bus)

```rust
// server/core/rsm/events.rs
// Bevy Messages (buffered MessageWriter/MessageReader — #[derive(Message)]).
// RSM writes; Core and Feature systems subscribe via MessageReader.
// SessionReady is the sole exception: Observer trigger per ADR-012.
// RSM has ZERO imports from feature/ or other core/ modules.

#[derive(Event)] pub struct DraftStarted       { pub round: u32, pub phase: DraftPhase }
#[derive(Event)] pub struct ShopRefreshNeeded  { pub player: PlayerId }
#[derive(Event)] pub struct AuctionPhaseEntered { pub round: u32 }
#[derive(Event)] pub struct PlacementPhaseEntered { pub round: u32 }
#[derive(Event)] pub struct ResolutionPhaseEntered { pub round: u32 }
#[derive(Event)] pub struct GameOverEmitted    { pub reason: GameOverReason,
                                                 pub loser: Option<PlayerId> }

// Subscriber contract:
//   DraftStarted          → Economy (mana ramp + gold income)
//   ShopRefreshNeeded     → Card Pool (draw shop slots per player)
//   AuctionPhaseEntered   → Auction System [M2]
//   PlacementPhaseEntered → Board/Lane System
//   ResolutionPhaseEntered → Combat Resolution [M2], Objective System, Board/Lane
//   GameOverEmitted       → Game Session System (teardown)
```

### Boundary 4: Core → Feature (Economy public API)

```rust
// server/core/economy/api.rs

impl EconomySystem {
    /// Returns Err if player cannot afford cost. Called by Board/Lane and Card Pool.
    pub fn validate_spend(economy: &PlayerEconomy, cost: u32,
        from_reserve: bool) -> Result<(), SpendError>;

    /// Deducts cost. Call only after validate_spend returns Ok.
    pub fn apply_spend(economy: &mut PlayerEconomy, cost: u32, from_reserve: bool);

    /// Awards gold (kill reward, objective reward).
    /// Called by Objective System and Combat Resolution [M2].
    pub fn apply_gold_award(economy: &mut PlayerEconomy, amount: u32);

    /// Reserves gold for an active auction bid (prevents double-spend). [M2]
    pub fn reserve_gold(economy: &mut PlayerEconomy,
        amount: u32) -> Result<(), SpendError>;

    /// Releases reservation (bid retracted or auction ended). [M2]
    pub fn release_gold_reservation(economy: &mut PlayerEconomy, amount: u32);
}

// Invariant: PlayerEconomy mutated ONLY through this API.
// Invariant: reserve_mana has NO cap (Economy OQ2 design decision).
```

### Boundary 5: Core → Feature (Card Pool public API)

```rust
// server/core/pool/api.rs

impl CardPool {
    /// Draws shop slots for one player. Called on ShopRefreshNeeded Message.
    pub fn refresh_shop(pool: &mut PlayerPool, catalog: &CardCatalog,
        rng: &mut ServerRng, config: &GameConfig) -> Vec<CardId>;

    /// Removes card from pool on acquisition (shop purchase, auction win, free pick).
    pub fn acquire_card(pool: &mut PlayerPool, card_id: CardId) -> Result<(), PoolError>;

    /// Weighted draw with optional class/family filter.
    /// Returns None (never panics) when filtered pool is empty.
    pub fn draw(pool: &PlayerPool, catalog: &CardCatalog,
        filter: PoolFilter, rng: &mut ServerRng) -> Option<CardId>;
}

// Invariant: copies_remaining never below 0.
// Invariant: CardCatalog is read-only after startup — never mutated.
```

### Boundary 6: Core → Feature (SessionConfig read contract)

```rust
// server/core/session/config.rs

#[derive(Resource, Clone)]
pub struct SessionConfig {
    pub mode: GameMode,
    pub player_count: u8,
    pub team_map: HashMap<PlayerId, TeamId>,
    pub class_map: HashMap<PlayerId, ClassId>,
    pub placement_timer_multiplier_effective: PlacementTimerMultiplier,
}

// Invariant: Inserted ONCE at SessionReady. NEVER mutated after insertion.
// Invariant: All fields populated at insertion — GSS panics if any slot has class=None.
// Invariant: Removed by GSS on GameOverEmitted.
// Feature systems: Res<SessionConfig> (read-only, no write access).
```

### Boundary 7: Feature → Core (Objective events + counters)

```rust
// server/feature/objective/events.rs

#[derive(Event)]
pub struct ObjectiveDestroyed {
    pub target_player: PlayerId,
    pub lane: u8,
    pub was_fake: bool,
    pub attacker: PlayerId,
}

// Subscribers:
//   Economy → apply_gold_award(attacker, objective_gold_reward) if attacker ≠ target
//   Board/Lane → expand spawn_range if was_fake

#[derive(Resource)]
pub struct ObjectiveCounters {
    pub real_destroyed: HashMap<PlayerId, u8>,
    pub fake_destroyed: HashMap<PlayerId, u8>,
}
// RSM reads Res<ObjectiveCounters> at RESOLUTION end for GAME_OVER check.
// RSM never imports from feature/objective/ directly.
```

---

## 5. ADR Audit

### Quality Check (all 12 ADRs)

| ADR | Engine Compat | Version Stamped | GDD Linkage | Conflicts | Valid |
|---|---|---|---|---|---|
| ADR-001: Objective Identity Unicast | ✅ | ✅ 2026-04-29 | ✅ objective-system, network-protocol, game-session-system | None | ✅ |
| ADR-002: Client-Server Authority | ✅ HIGH flagged | ✅ 2026-04-29 | ✅ network-protocol, technical-preferences | None | ✅ |
| ADR-003: Cargo Workspace Structure | ✅ MEDIUM flagged | ✅ 2026-04-29 | Foundational — enables all other ADRs | None | ✅ |
| ADR-004: Asset Loading Pipeline | ✅ MEDIUM flagged | ✅ 2026-04-29 | ✅ game-config, card-data-pool | None | ✅ |
| ADR-005: Server-side RNG | ✅ LOW risk | ✅ 2026-04-29 | ✅ server-rng | None | ✅ |
| ADR-006: Card Data Schema + Pool | ✅ LOW risk | ✅ 2026-04-29 | ✅ card-data-pool | None | ✅ |
| ADR-007: Placement Buffer | ✅ HIGH flagged | ✅ 2026-04-29 | ✅ board-lane-system, network-protocol | None | ✅ |
| ADR-008: Lightyear Channel Config | ✅ HIGH flagged | ✅ 2026-04-29 | ✅ network-protocol | None | ✅ |
| ADR-009: RSM Phase State | ✅ HIGH flagged | ✅ 2026-04-29 | ✅ round-state-machine | None | ✅ |
| ADR-010: RSM Event Bus | ✅ HIGH flagged | ✅ 2026-04-29 | ✅ round-state-machine, economy-system | None | ✅ |
| ADR-011: Reconnect + Snapshot | ✅ HIGH flagged | ✅ 2026-04-29 | ✅ network-protocol, game-session-system | None | ✅ |
| ADR-012: SessionReady Delivery | ✅ HIGH flagged | ✅ 2026-04-29 | ✅ game-session-system, round-state-machine | None | ✅ |

**12/12 pass. Zero conflicts across all ADRs.**

---

### Traceability Matrix (74 TRs → 12 ADRs)

| System | TR Range | Count | ADR Coverage | Status |
|---|---|---|---|---|
| Card Data & Pool | TR-CDP-01–09 | 9 | ADR-004 (loading), ADR-006 (schema + pool state) | ✅ Full |
| Game Config | TR-GC-01–05 | 5 | ADR-004 (loading + validation), ADR-003 (shared/ purity) | ✅ Full |
| Server-side RNG | TR-RNG-01–06 | 6 | ADR-005 | ✅ Full |
| Economy System | TR-ECO-01–08 | 8 | ADR-010 (DraftStarted subscriber contract) | ✅ Full |
| Board/Lane System | TR-BLS-01–10 | 10 | ADR-007 (placement buffer + reveal), ADR-010 (PlacementPhaseEntered) | ✅ Full |
| Round State Machine | TR-RSM-01–10 | 10 | ADR-009 (phase Resource), ADR-010 (event bus + emission order) | ✅ Full |
| Network Protocol | TR-NP-01–12 | 12 | ADR-002 (authority + phase-gate), ADR-008 (channels), ADR-011 (reconnect) | ✅ Full |
| Game Session System | TR-GSS-01–10 | 10 | ADR-011 (reconnect + snapshot), ADR-012 (SessionReady delivery) | ✅ Full |
| Objective System | TR-OBJ-01–10 | 10 | ADR-001 (identity unicast), ADR-010 (ResolutionPhaseEntered subscriber) | ✅ Full |

**74/74 TRs covered. Zero gaps for M1 systems.**

---

## 6. Required ADRs

### M1 — All Written ✅

ADR-001 through ADR-012 cover all Foundation, Core, and M1 Feature layer decisions.
Architecture is complete for implementation to begin.

### M2 — Write After M2 GDDs Are Authored

| ADR Title | Write When |
|---|---|
| Auction System Event Architecture | `design/gdd/auction-system.md` approved |
| Combat Resolution Sub-step Scheduling | `design/gdd/combat-resolution.md` approved |
| Card Acquisition + Hand Management | `design/gdd/card-acquisition.md` approved |

### M3 — Write After M3 GDDs Are Authored

| ADR Title | Write When |
|---|---|
| Keyword Observer Architecture (DEATH/APPEARANCE/FINAL BLOW triggers) | `design/gdd/keyword-system.md` approved |
| Prism System Integration | `design/gdd/prism-system.md` approved |
| Class System Rules Engine | `design/gdd/class-system.md` approved |

---

## Architecture Principles

1. **Server is authoritative, client is a view.** All game state lives on the server. Clients receive S2C projections and send C2S inputs. No client-side game logic.
2. **Emit, don't call.** Systems communicate via Bevy Messages and Events. Direct cross-module function calls are forbidden across layer boundaries.
3. **Secrets stay on the server.** Hidden information (objective identity, RNG seeds) never enters the client crate. Server-side resources hold secrets; unicast Messages deliver need-to-know projections.
4. **No hardcoded balance values.** Every tuning knob flows through `GameConfig` loaded from `assets/config/game_config.ron`. Systems read `Res<GameConfig>`; they never hold cached field copies.
5. **liv-bevy-018 + liv-bevy-lightyear on every PR.** Given the 4-version post-cutoff gap, these skills are non-optional gates on any `.rs` file touching Bevy or Lightyear APIs.

---

## Phase 7b — Technical Director Sign-Off

**Gate applied:** TD-ARCHITECTURE (`.claude/docs/director-gates.md`)
**Date:** 2026-04-29
**Review mode:** Lean (LP-FEASIBILITY skipped)

### Criteria Assessment

| Criterion | Status | Notes |
|---|---|---|
| Every TR covered by an architectural decision | ✅ PASS | 74/74 TRs mapped in Phase 5 traceability matrix |
| All HIGH-risk engine domains addressed | ✅ PASS | Lightyear 0.26, bevy_ui 0.18, ECS/Events 0.18 all explicitly handled with "VERIFY BEFORE IMPLEMENTING" flags |
| API boundaries clean, minimal, implementable | ✅ PASS | Phase 4 boundaries are Rust-typed; single-writer invariants enforced; pool API returns Option not panics |
| Foundation layer ADR gaps resolved | ✅ PASS | ADR-002 through ADR-008 cover all Foundation decisions; zero gaps |

**One condition:** ADR-012's `Commands::trigger()` vs `apply_deferred` ordering is an unverified correctness assumption. The verification unit test specified in ADR-012 must pass before the GSS+RSM integration story can be marked Ready. This is a story-level gate, not a architecture-level blocker.

**Sign-Off: APPROVED WITH CONDITIONS**
*Architecture is sound for implementation to begin on all M1 systems except the GSS+RSM integration story, which is gated on ADR-012 verification.*

---

## Open Questions

| # | Question | Blocks | Owner |
|---|---|---|---|
| OQ-A | Lightyear 0.26 exact unicast symbol: `NetworkTarget::Single(ClientId)` vs `NetworkTarget::Only(vec![id])` — verify on docs.rs/lightyear/0.26 | ADR-001 impl, any S2C unicast | Network programmer |
| OQ-B | Bevy States vs manual phase enum for RSM — does `#[derive(States)]` work cleanly with Lightyear session lifecycle, or should phase be a plain `Resource`? | RSM ADR | Lead programmer |
| OQ-C | `SessionReady` delivery: Observer (same-frame) vs buffered `Events<T>` (next-frame) — ordering guarantee needed before RSM reads `SessionConfig` | GSS + RSM ADR | Lead programmer |
| OQ-D | Lightyear 0.26 reliable channel ordering across message types — verify `S2CResolutionEvent` precedes `S2CPhaseChanged(DRAFT_SHOP)` is guaranteed by channel config, not just send order | Network protocol ADR | Network programmer |
