# ADR-015: Card Acquisition Shop State Machine Architecture

## Status

Accepted

## Date

2026-04-30

## Decision Makers

User + gameplay-programmer + engine specialist

## Summary

The Card Acquisition system's authoritative per-player state is stored in two server-only
Resources: `ShopStates` (shop phase machine, displayed_this_draft dedup set, current slot
display, refresh counter) and `PlayerHands` (each player's card hand). A single system —
`card_acquisition_tick_system` — is the sole `ResMut<ShopStates>` writer and the sole drainer
of `MessageReceiver<C2SPurchaseCard>` and `MessageReceiver<C2SRefreshShop>`. The RSM emits a
`ShopRefreshTriggered` Bevy Message on each relevant phase entry; CA consumes it to execute
auto-refresh draws. The transactional spend+distribute+refund-on-fail pair (CA18) executes as
sequential calls within one system run — no cross-frame message path between the two steps.

---

## Engine Compatibility

| Field | Value |
|-------|-------|
| **Engine** | Bevy 0.18 + Lightyear 0.26 |
| **Domain** | Core |
| **Knowledge Risk** | HIGH — Bevy 0.15–0.18 are all post-cutoff |
| **References Consulted** | `docs/engine-reference/bevy/VERSION.md`, ADR-009, ADR-010, ADR-013 |
| **Post-Cutoff APIs Used** | `#[derive(Resource)]`, `Res<T>` / `ResMut<T>` system params (stable); `#[derive(Message)]` + `MessageReader<T>` + `app.add_message::<T>()` (Bevy 0.17+ Message/Event split — `EventReader`/`EventWriter` removed); Lightyear 0.26 `MessageReceiver<T>` for C2S inbound messages; `HashSet<CardId>` inside Resource (std::collections, no Bevy version dependency) |
| **Verification Required** | (1) **RESOLVED**: Lightyear 0.26 C2S receiver type confirmed — `MessageReceiver<C2SPurchaseCard>` and `MessageReceiver<C2SRefreshShop>` with `.receive_messages()` are the correct system params. Confirmed via ADR-013 item 1 resolution and `docs/engine-reference/bevy/current-best-practices.md` (Lightyear 0.26 pattern). `MessageReceiver<T>` is Lightyear's C2S network API; Bevy's `MessageReader<T>` is the server-internal bus — do not confuse them. (2) **RESOLVED (implementation-time)**: DRAFT and RESOLUTION phases are mutually exclusive RSM phases — `card_acquisition_tick_system` runs in DRAFT only; Prism/Objective systems run in RESOLUTION only. No concurrent `ResMut<PlayerHands>` write conflict is possible. Verify via Bevy schedule graph dump when registering systems; add explicit `CardAcquisitionSet::Tick.before(PrismSet::Tick)` ordering as a compile-time guard. (3) **RESOLVED**: `Resource` containing `HashSet<CardId>` compiles without `#[derive(Reflect)]` or `#[derive(Clone)]` — these are not required for `Resource` registration. `#[derive(Reflect)]` is only needed if the resource is explicitly registered with Bevy's reflection system (not used for server-only logic resources in this project). |

---

## ADR Dependencies

| Field | Value |
|-------|-------|
| **Depends On** | ADR-009 (RoundState resource pattern — `ShopStates` mirrors this; single-resource per-domain, single-writer system); ADR-010 (message bus — `ShopRefreshTriggered` is added to its event catalog as part of this ADR); ADR-013 (AuctionState resource pattern; spend/refund atomicity pattern — this ADR applies the same approach to card purchase); ADR-005 (ServerRng — all shop draw operations consume deterministic seeds); ADR-006 (CardPool — `draw_class_card`, `draw_neutral_family`, `draw_family_card`, `distribute`, `is_available` interfaces); ADR-008 (Lightyear channel config — `S2CDraftOffering` and `S2CShopSlots` route on reliable unicast channel) |
| **Enables** | Any story involving the personal shop, draft initial display, card purchase, manual refresh, hand-size enforcement, or dedup behaviour; `PlayerHands` resource definition enables Prism System and Objective System stories that write to player hands |
| **Blocks** | Any story involving CA1–CA22 acceptance criteria — no card purchase or shop display may be implemented until `ShopStates` and `PlayerHands` resource shapes are settled |
| **Ordering Note** | ADR-009, ADR-010, ADR-013 must be Accepted before implementation begins. ADR-010 event catalog must be updated to include `ShopRefreshTriggered` (done as part of this ADR authoring pass). `card_acquisition_tick_system` is scheduled AFTER `rsm_tick_system` so the `ShopRefreshTriggered` message produced by RSM is readable in the same frame. |

---

## Context

### Problem Statement

The Card Acquisition system manages per-player shop state across three mutually exclusive
sub-phases (DRAFT_INITIAL, DRAFT_AUCTION lock, DRAFT_SHOP). Its Bevy 0.18 implementation
requires decisions on: (1) how to store per-player shop state (hand, dedup set, slot display,
refresh counter) in server Resources without ECS component queries, (2) how the RSM signals
auto-refresh events to the CA system at the right phase transitions, (3) how to implement the
spend+distribute+refund-on-fail atomic pair required by CA18 without a cross-frame race,
and (4) how `PlayerHands` is shared with Prism System and Objective System (which bypass CA
entirely). Without this decision, CA, Prism, and Objective will be implemented with
incompatible hand-access patterns.

### Constraints

- **Server authority**: `ShopStates` and `PlayerHands` are server-only; clients receive updates
  via `S2CShopSlots`, `S2CDraftOffering`, and hand-update messages (ADR-002).
- **Single C2S reader per message type**: Lightyear's `MessageReceiver<T>` can only be drained
  once per frame. No system other than `card_acquisition_tick_system` may drain
  `MessageReceiver<C2SPurchaseCard>` or `MessageReceiver<C2SRefreshShop>`.
- **CA18 atomicity**: If `distribute()` returns `Err(DistributeError::Exhausted)` after
  `spend_gold()` succeeded, `refund_gold()` must be called before the system returns.
  Gold must never remain deducted after a failed distribute. No cross-frame messaging allowed
  between the spend and the refund — they must be sequential lines in the same function body.
- **Phase exclusion**: CA is active only during DRAFT sub-phases; Prism/Objective write hands
  only during RESOLUTION. These phases never overlap, so `ResMut<PlayerHands>` is safe for
  multiple writers as long as scheduling is explicit.
- **Dedup continuity (auction rounds)**: In auction rounds, `displayed_this_draft` is NOT
  cleared on the DRAFT_AUCTION → DRAFT_SHOP transition. Dedup history accumulates across
  the full DRAFT phase (DRAFT_AUCTION + DRAFT_SHOP).
- **Bevy 0.18**: No bundles, no `EventWriter`/`EventReader`, `Query::single()` returns `Result`.

### Requirements

- `ShopStates` is the single source of truth for all shop state per player on the server.
- `PlayerHands` is the single source of truth for each player's card hand on the server.
- Only `card_acquisition_tick_system` writes `ResMut<ShopStates>`.
- Economy calls (`spend_gold`, `refund_gold`) execute via direct `ResMut<PlayerEconomies>` access
  within `card_acquisition_tick_system` — not cross-frame messages — to enforce CA18 atomicity.
- `ShopStates` must be testable with `World::new()` — no live Lightyear session required.

---

## Decision

The Card Acquisition system's per-player state lives in two Bevy Resources on the server.
`ShopStates` holds the shop-specific machine state for each player: current phase, the
`displayed_this_draft` dedup set, the three-slot display, and the refresh counter. `PlayerHands`
holds each player's card hand (a `Vec<CardId>` per player). A single system —
`card_acquisition_tick_system` — is the sole `ResMut<ShopStates>` writer and the sole drainer
of the two Lightyear C2S receivers.

**RSM → CA trigger**: The RSM emits `ShopRefreshTriggered { player_id: PlayerId, trigger:
ShopRefreshTrigger }` as a Bevy buffered Message via `MessageWriter<ShopRefreshTriggered>` on
the relevant phase entries. CA consumes it via `MessageReader<ShopRefreshTriggered>`. This
preserves the ADR-010 event-bus pattern and ensures the refresh fires exactly once per phase
entry, without CA needing to detect phase transitions from `Res<RoundState>` directly.

**ShopRefreshTrigger variants**:
- `DraftInitial` — on DRAFT_INITIAL entry: draw 9 cards via `draw_initial_draft()`.
- `AuctionLock` — on DRAFT_AUCTION entry (auction round): draw 3 slots, enter AuctionLock phase.
- `ShopOpen` — on DRAFT_SHOP entry (non-auction round): draw 3 slots, enter ShopActive phase.
- `ShopUnlock` — on DRAFT_AUCTION → DRAFT_SHOP (auction round): no new draw; same slots become
  purchasable. Transition from AuctionLock → ShopActive. Reset `refresh_count_this_draft`.
  **Do not** clear `displayed_this_draft` — dedup history carries through.

**Purchase flow** (DRAFT_SHOP only, ShopActive phase):
1. Phase gate: reject if `player_shop.phase != ShopActive`.
2. `hand.len() < 10` check (CA1, CA2).
3. `pool.is_available(card_id)` check (CA13 — TOCTOU guard).
4. `economy.spend_gold(player_id, card_cost)` — Economy validates gold sufficiency.
5. `card_pool.distribute(card_id)`:
   - `Ok(())` → push card to `player_hand`, remove slot, send `S2CShopSlots`.
   - `Err(DistributeError::Exhausted)` → call `economy.refund_gold(player_id, card_cost)`,
     log error, leave slot displayed, return (CA18 mandatory rollback).

**Scheduling order** (code-order within Update schedule):
`AuctionSet::Tick` → `RsmSet::Tick` → `CardAcquisitionSet::Tick`

RSM produces `ShopRefreshTriggered` before CA consumes it. CA runs after RSM in every frame.

### Architecture Diagram

```
SERVER WORLD
┌──────────────────────────────────────────────────────────────────────┐
│  ShopStates (Resource — server only)                                 │
│  └── players: HashMap<PlayerId, PlayerShopState>                     │
│        ├── phase: ShopPhase   (Inactive|DraftInitial|AuctionLock|    │
│        │                        ShopActive)                          │
│        ├── displayed_this_draft: HashSet<CardId>  (dedup set)        │
│        ├── current_slots: [Option<CardId>; 3]    (visible slots)     │
│        └── refresh_count_this_draft: u32         (Formula 1 counter) │
│                                                                      │
│  PlayerHands (Resource — server only)                                │
│  └── hands: HashMap<PlayerId, Vec<CardId>>                           │
│      (written by CA in DRAFT, Prism/Objective in RESOLUTION)         │
│                                                                      │
│  card_acquisition_tick_system  (after RsmSet::Tick)                  │
│                                                                      │
│    Bevy-side readers (internal message bus):                         │
│      MessageReader<ShopRefreshTriggered>  → auto-refresh trigger     │
│                                                                      │
│    Lightyear-side readers (network):                                 │
│      MessageReceiver<C2SPurchaseCard>    → purchase input            │
│      MessageReceiver<C2SRefreshShop>    → manual refresh input       │
│      (Only this system drains these receivers — sole readers)        │
│                                                                      │
│    Writes:                                                           │
│      ResMut<ShopStates>     (sole writer)                            │
│      ResMut<PlayerHands>    (in DRAFT only; Prism/Obj write in RES)  │
│      ResMut<PlayerEconomies>   (spend_gold / refund_gold via api.rs) │
│      ResMut<CardPool>       (distribute, is_available)               │
│                                                                      │
│    Per-frame execution order (code order):                           │
│      1. Drain MessageReader<ShopRefreshTriggered>                    │
│         ├── DraftInitial → draw_initial_draft(), send S2CDraftOffering│
│         ├── AuctionLock  → draw 3 slots, phase=AuctionLock, send S2C │
│         ├── ShopOpen     → draw 3 slots, phase=ShopActive, send S2C  │
│         └── ShopUnlock   → phase=ShopActive, reset refresh_count     │
│      2. If ShopActive: drain MessageReceiver<C2SRefreshShop>         │
│         └── validate gold → draw 3 slots → update displayed → S2C   │
│      3. If ShopActive or DraftInitial: drain C2SPurchaseCard         │
│         └── phase gate → hand check → avail check → spend → distrib │
│             └── on Exhausted: refund_gold (CA18 mandatory rollback)  │
│      4. If AuctionLock or Inactive: drain + discard both receivers   │
│                                                                      │
│    Lightyear-side senders (network):                                 │
│      S2CDraftOffering  (reliable unicast → player)  DraftInitial     │
│      S2CShopSlots      (reliable unicast → player)  every refresh    │
└──────────────────────────────────────────────────────────────────────┘

         ┌─────────────────────────────────────────────────────┐
         │  rsm_tick_system  (RsmSet::Tick)                    │
         │  On DRAFT_INITIAL entry:                            │
         │    MessageWriter<ShopRefreshTriggered>              │
         │      { player_id, trigger: DraftInitial }  × 2     │
         │  On DRAFT_AUCTION entry (auction round):            │
         │    MessageWriter<ShopRefreshTriggered>              │
         │      { player_id, trigger: AuctionLock }   × 2     │
         │  On DRAFT_AUCTION→DRAFT_SHOP (auction round):       │
         │    MessageWriter<ShopRefreshTriggered>              │
         │      { player_id, trigger: ShopUnlock }    × 2     │
         │  On DRAFT_SHOP entry (non-auction round):           │
         │    MessageWriter<ShopRefreshTriggered>              │
         │      { player_id, trigger: ShopOpen }      × 2     │
         └─────────────────────────────────────────────────────┘
```

### Key Interfaces

```rust
// server/src/card_acquisition/state.rs

use std::collections::{HashMap, HashSet};
use bevy::prelude::*;
use shared::protocol::{CardId, PlayerId};

/// Server-authoritative per-player shop state machine.
/// Only `card_acquisition_tick_system` may hold `ResMut<ShopStates>`.
#[derive(Resource, Default)]
pub struct ShopStates {
    pub players: HashMap<PlayerId, PlayerShopState>,
}

#[derive(Default)]
pub struct PlayerShopState {
    pub phase: ShopPhase,
    /// Cards shown this DRAFT phase (not cleared until next DRAFT entry).
    pub displayed_this_draft: HashSet<CardId>,
    pub current_slots: [Option<CardId>; 3],
    /// Resets to 0 on each DRAFT_SHOP (or DRAFT_INITIAL) entry. See GDD Formula 1.
    pub refresh_count_this_draft: u32,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShopPhase {
    #[default]
    Inactive,
    DraftInitial,   // 9-card purchase window, no manual refresh
    AuctionLock,    // 3 slots visible but read-only (during DRAFT_AUCTION)
    ShopActive,     // purchases + manual refresh available (during DRAFT_SHOP)
}
```

```rust
// server/src/card_acquisition/hands.rs

use std::collections::HashMap;
use bevy::prelude::*;
use shared::protocol::{CardId, PlayerId};

/// Server-authoritative player hand state.
/// Written by card_acquisition_tick_system (DRAFT phases).
/// Also written by prism_tick_system and objective_tick_system (RESOLUTION phase).
/// Phases are mutually exclusive — no concurrent-write conflict possible.
#[derive(Resource, Default)]
pub struct PlayerHands {
    pub hands: HashMap<PlayerId, Vec<CardId>>,
}

impl PlayerHands {
    pub fn hand_len(&self, player: PlayerId) -> usize {
        self.hands.get(&player).map_or(0, |h| h.len())
    }

    pub fn push_card(&mut self, player: PlayerId, card_id: CardId) {
        self.hands.entry(player).or_default().push(card_id);
    }
}
```

```rust
// server/src/card_acquisition/messages.rs

use bevy::prelude::*;
use shared::protocol::PlayerId;

/// Emitted by rsm_tick_system on each relevant phase entry.
/// Consumed by card_acquisition_tick_system to execute auto-refresh.
/// Registered on Bevy's internal message bus (not a Lightyear network message).
#[derive(Message, Clone)]
pub struct ShopRefreshTriggered {
    pub player_id: PlayerId,
    pub trigger: ShopRefreshTrigger,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShopRefreshTrigger {
    /// DRAFT_INITIAL entry: draw 9 cards via draw_initial_draft().
    DraftInitial,
    /// DRAFT_AUCTION entry (auction round): draw 3 slots, lock shop.
    AuctionLock,
    /// DRAFT_SHOP entry (non-auction round): draw 3 slots, open shop.
    ShopOpen,
    /// DRAFT_AUCTION → DRAFT_SHOP transition: same slots, unlock shop.
    /// Does NOT clear displayed_this_draft. Resets refresh_count_this_draft.
    ShopUnlock,
}
```

```rust
// server/src/card_acquisition/system.rs — parameter list sketch

fn card_acquisition_tick_system(
    mut shop_states: ResMut<ShopStates>,
    mut hands: ResMut<PlayerHands>,
    mut economy: ResMut<PlayerEconomies>,
    mut card_pool: ResMut<CardPool>,
    mut server_rng: ResMut<ServerRng>,
    game_config: Res<GameConfig>,
    // Bevy internal message bus:
    mut refresh_triggered: MessageReader<ShopRefreshTriggered>,
    // Lightyear network (exact type pending Verification Required item 1):
    mut purchase_messages: MessageReceiver<C2SPurchaseCard>,
    mut refresh_messages: MessageReceiver<C2SRefreshShop>,
    // Lightyear S2C senders:
    mut s2c_sender: MessageSender</* S2CDraftOffering, S2CShopSlots */>,
) {
    // Step 1: consume ShopRefreshTriggered messages from RSM
    // Step 2: if ShopActive — drain C2SRefreshShop (manual refresh)
    //         if AuctionLock or Inactive — drain + discard C2SRefreshShop
    // Step 3: if ShopActive or DraftInitial — drain C2SPurchaseCard
    //         └─ on Exhausted: economy.refund_gold(player_id, cost);
    //         if AuctionLock or Inactive — drain + discard C2SPurchaseCard
}
```

```rust
// server/src/card_acquisition/plugin.rs — scheduling

// CA runs after RSM (RSM produces ShopRefreshTriggered; CA consumes it)
app.configure_sets(Update, CardAcquisitionSet::Tick.after(RsmSet::Tick));
// CA runs after Auction (consistent with auction → rsm → ca ordering)
app.configure_sets(Update, CardAcquisitionSet::Tick.after(AuctionSet::Tick));
```

### Purchase Atomicity (CA18) — Mandatory Rollback Pattern

```rust
// Within card_acquisition_tick_system purchase handler — sequential, same function body:
if economy.spend_gold(player_id, card_cost).is_ok() {
    match card_pool.distribute(card_id) {
        Ok(()) => {
            hands.push_card(player_id, card_id);
            shop_state.current_slots[slot_idx] = None;
            // send S2CShopSlots update
        }
        Err(DistributeError::Exhausted) => {
            // TOCTOU recovery: spend succeeded but distribute failed.
            // refund_gold call is MANDATORY before returning — gold must not remain deducted.
            economy.refund_gold(player_id, card_cost);
            // Slot remains displayed (dead slot). Card NOT added to hand.
            error!("card_acquisition: distribute TOCTOU for {:?} — gold refunded", card_id);
        }
    }
}
// No await, no yield, no system boundary between spend_gold and the refund_gold path.
```

---

## Alternatives Considered

### Alternative 1: ECS Components on Player Entities

- **Description**: Attach `ShopPhaseComponent`, `DisplayedThisDraftComponent`,
  `CurrentSlotsComponent`, `RefreshCountComponent` to player entities. Systems query
  `Query<(&ShopPhaseComponent, ...), With<PlayerTag>>`.
- **Pros**: Idiomatic Bevy ECS for per-entity data. Bevy change detection works automatically.
- **Cons**: `Query::single()` returns `Result` in Bevy 0.16+ — adds unwrap/error-handling
  at every access. The game is strictly 1v1; component queries over 2 known entities add
  unnecessary indirection. `HashSet<CardId>` inside a component creates awkward
  `HashMap`-in-component patterns. Mirrors the same rejection made in ADR-009 and ADR-013.
- **Rejection Reason**: Resources are the idiomatic Bevy pattern for global/per-session singleton
  state. `ShopStates` is session-scoped data, not per-entity spatial data.

### Alternative 2: CA Reads Res\<RoundState\> Directly for Phase Detection

- **Description**: `card_acquisition_tick_system` inspects `Res<RoundState>` each frame,
  comparing `current_phase` to the previous frame's cached phase to detect transitions.
  No `ShopRefreshTriggered` message needed.
- **Pros**: Removes one message type from the event bus. Fewer moving parts.
- **Cons**: CA must maintain "previous phase" state inside `ShopStates` or a local cache.
  Transition detection via diff is fragile — if CA misses a frame (scheduling hiccup), it
  silently misses the refresh trigger. The RSM already knows exactly when transitions occur;
  emitting a message is the established ADR-010 pattern for broadcasting phase-entry events.
- **Rejection Reason**: Violates the ADR-010 event-bus contract. Makes CA depend on transition
  diffing logic that is RSM's responsibility. `ShopRefreshTriggered` is a clean, testable
  trigger — CA can be unit-tested by injecting it via `World::new()` without a live RSM.

### Alternative 3: Message-Passing for spend/refund Pair (CA18)

- **Description**: On purchase: emit `SpendGold { player, amount }` message; Economy System
  handles it. If distribute fails: emit `RefundGold { player, amount }`. Economy processes
  both in subsequent frames.
- **Pros**: Decouples CA from Economy at the Rust module boundary.
- **Cons**: Breaks CA18's mandatory atomicity. A one-frame window exists where `spend_gold`
  was processed but `refund_gold` has not yet arrived — the player's gold is temporarily
  deducted with no card in hand. The GDD explicitly requires the rollback to be mandatory and
  immediate. Cross-frame messaging cannot provide this guarantee. Mirrors the same rejection
  made in ADR-013 for the release-before-reserve invariant.
- **Rejection Reason**: Violates the GDD CA18 requirement: "Call `refund_gold(player, cost)`
  immediately. Gold must never remain deducted after a failed distribute."

---

## Consequences

### Positive

- `Res<ShopStates>` is O(1) lookup per player (HashMap over 2 known entries). No ECS query,
  no `Query::single()` boilerplate.
- The CA18 rollback invariant is enforced by sequential call order within one system body
  — `spend_gold` → `distribute` → `refund_gold` on failure — with no system boundary between them.
- `ShopRefreshTriggered` decouples the RSM phase-entry trigger from the CA draw pipeline.
  CA can be unit-tested by injecting `ShopRefreshTriggered` messages directly into a `World::new()`
  test without a live RSM or Lightyear session.
- `PlayerHands` is a separately testable resource. Prism and Objective stories can be written
  and tested against `PlayerHands` without depending on `ShopStates`.
- `ShopPhase` provides an explicit phase gate: any incoming C2S message received in the wrong
  phase is silently discarded, satisfying CA5, CA7, and CA20 without complex RSM coupling.

### Negative

- `card_acquisition_tick_system` has a wide parameter list: two domain `ResMut` (ShopStates,
  PlayerHands), `ResMut<PlayerEconomies>`, `ResMut<CardPool>`, `ResMut<ServerRng>`, `Res<GameConfig>`,
  Bevy message readers, and two Lightyear C2S receivers. This is the cost of keeping CA18
  atomicity within a single system — the function cannot be split further without breaking it.
- `PlayerHands` is a shared resource written by three systems across two mutually exclusive
  phases. Future engineers must not add a fourth writer in the same phase as an existing writer
  without explicit scheduling. The resource name alone does not communicate this constraint
  — this ADR and the control manifest must do so.
- `displayed_this_draft` as a `HashSet<CardId>` inside `ShopStates` is heap-allocated per
  player per session. With ~298 cards and 3 slots per refresh, worst-case set size is small
  (< 100 entries), but the allocation exists. Acceptable for hackathon scope.

### Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Lightyear 0.26 C2S receiver type is wrong (`MessageReceiver<T>` vs another type) | MEDIUM | Compilation failure | Same verification as ADR-013 item 1. Abstract behind a helper returning `impl Iterator` for testability. |
| A second system accidentally drains `MessageReceiver<C2SPurchaseCard>` | LOW | Silently lost purchases — player clicks buy, nothing happens | `card_acquisition_tick_system` is the sole system in the CA plugin. Code review gate: `MessageReceiver<C2SPurchaseCard>` appears in exactly one system. |
| `ResMut<PlayerHands>` scheduling conflict between CA and future Prism/Objective systems | LOW | Bevy runtime panic if both hold `ResMut<PlayerHands>` in the same frame | Prism/Objective run in RESOLUTION phase; CA in DRAFT phase. RSM phase exclusion makes same-frame conflict impossible. Verify via Bevy schedule graph dump. Explicit `CardAcquisitionSet::Tick.before(PrismSet::Tick)` ordering provides compile-time enforcement if sets ever overlap. |
| `displayed_this_draft` not cleared correctly in auction rounds | MEDIUM | Dedup leaks between phases; player sees duplicate cards | CA22 acceptance criterion directly tests this. Test fixture must simulate DRAFT_AUCTION → DRAFT_SHOP without clearing dedup, then verify new DRAFT_AUCTION entry does clear it. |
| `ShopRefreshTriggered` emitted for wrong phase variants (RSM bug) | LOW | Shop draws at wrong time, slots populated during RESOLUTION | `ShopPhase` gate in CA rejects all draws outside the expected phase. CA self-protects — it will not execute a draw if its internal phase disagrees with the trigger. |

---

## GDD Requirements Addressed

| GDD System | Requirement | How This ADR Addresses It |
|------------|-------------|--------------------------|
| `card-acquisition.md` | Rule 1 — `hand: Vec<CardId>` capped at 10, server-authoritative | `PlayerHands` resource on server; `hand_len()` check in purchase handler (CA1, CA2) |
| `card-acquisition.md` | Rule 2 — DRAFT_INITIAL: `draw_initial_draft(class, 9, seed)`, send `S2CDraftOffering` | `ShopRefreshTriggered { trigger: DraftInitial }` → DraftInitial branch in step 1; `phase = ShopPhase::DraftInitial` (CA3, CA4, CA5) |
| `card-acquisition.md` | Rule 3 — Auto-refresh: RSM fires once per phase entry via `refresh_shop(player)` | `MessageReader<ShopRefreshTriggered>` consumed at step 1; variant determines draw type (CA6, CA15, CA22) |
| `card-acquisition.md` | Rule 4 — DRAFT_AUCTION: shop locked, C2S messages silently discarded | `ShopPhase::AuctionLock` gate: both receivers drained and discarded at step 4 (CA7) |
| `card-acquisition.md` | Rule 5 — Manual refresh cost Formula 1: `refresh_base_cost + min(refresh_count, refresh_cap)` | `refresh_count_this_draft` field in `PlayerShopState`; validated against Economy before draw (CA8–CA11) |
| `card-acquisition.md` | Rule 5 — Dedup: `displayed_this_draft` accumulates across all refreshes in a DRAFT phase | `HashSet<CardId>` in `PlayerShopState`; never cleared mid-phase; checked before every slot assignment (CA6, CA12, CA16, CA19) |
| `card-acquisition.md` | Rule 6 — Purchase: 3 pre-purchase checks in order | Phase gate → `hand_len < 10` → `pool.is_available()` → `spend_gold` → `distribute` (CA13, CA14, CA2) |
| `card-acquisition.md` | Rule 7 — External bypasses (Prism, Objective) bypass CA entirely | `PlayerHands` resource is separately accessible by Prism/Objective via `ResMut<PlayerHands>`; CA is not in that call chain (CA17) |
| `card-acquisition.md` | CA18 — `distribute()` failure after `spend_gold()` succeeds: mandatory `refund_gold` | Sequential `spend_gold → distribute → refund_gold on Err` within one system body; no cross-frame path (CA18) |
| `card-acquisition.md` | CA20 — Phase transition wins if timer expires mid-message | `ShopPhase` gate checked per message; `ShopUnlock`/`ShopOpen` resets phase before next C2S drain iteration (CA20) |
| `card-acquisition.md` | CA21 — `S2CDraftOffering` unicast to target player only | Lightyear unicast sender used for `S2CDraftOffering` (reliable unicast channel per ADR-008) (CA21) |
| `round-state-machine.md` | Rule 5 — RSM fires `refresh_shop(player)` on DRAFT phase entries | `rsm_tick_system` emits `ShopRefreshTriggered` with correct trigger variant on each phase entry |
| `network-protocol.md` | `S2CDraftOffering` + `S2CShopSlots` unicast, reliable | Both messages sent via Lightyear reliable unicast channel (ADR-008); schemas already registered in network-protocol.md (OQ2 resolved) |

---

## Performance Implications

- **CPU**: When `ShopPhase::Inactive` (most frames — PLACEMENT and RESOLUTION), CA drains two
  empty Lightyear receivers and one empty Bevy message reader, then returns — < 1 µs. During
  DRAFT phases: slot draw is O(3 × retry_budget) where retry_budget ≤ 20; dedup check is
  O(1) `HashSet` lookup. Total budget: < 0.1 ms per DRAFT phase tick.
- **Memory**: `PlayerShopState` per player: `HashSet<CardId>` (max ~100 entries × 4 bytes =
  ~400 bytes); `[Option<CardId>; 3]` = 12 bytes; `u32` counter. Under 1 KB per player, 2 KB total.
  `PlayerHands`: `Vec<CardId>` max 10 entries × 4 bytes = 40 bytes per player. Negligible.
- **Network**: `S2CDraftOffering` sends once per game (9 `CardId`s = ~36 bytes). `S2CShopSlots`
  sends per refresh (3 `Option<CardId>`s = ~12 bytes). Both unicast. Well within the 1 KB/round
  budget (ADR-008).
- **Load Time**: Both resources inserted at plugin setup. No asset loading.

---

## Migration Plan

Greenfield — no existing card acquisition code in the codebase.

1. Define `ShopStates`, `PlayerShopState`, `ShopPhase` in `server/src/card_acquisition/state.rs`.
2. Define `PlayerHands` in `server/src/card_acquisition/hands.rs`.
3. Define `ShopRefreshTriggered` and `ShopRefreshTrigger` in
   `server/src/card_acquisition/messages.rs`.
4. Register both resources and the Bevy Message in `server/src/card_acquisition/plugin.rs`.
5. Implement `card_acquisition_tick_system` in `server/src/card_acquisition/system.rs`.
6. Configure `CardAcquisitionSet::Tick.after(RsmSet::Tick)` in the server's `Update` schedule.
7. Add `MessageWriter<ShopRefreshTriggered>` to `rsm_tick_system`'s parameter list. Emit the
   correct trigger variant on each relevant phase entry.
8. Verify Lightyear 0.26 C2S receiver API before implementing message drain loops (Verification
   Required item 1). Abstract behind a helper for testability.
9. Add explicit scheduling between `CardAcquisitionSet::Tick` and future `PrismSet::Tick` /
   `ObjectiveSet::Tick` when those systems are implemented (Verification Required item 2).

---

## Validation Criteria

- [ ] `ShopStates` and `PlayerHands` insert cleanly into `World::new()` with default values.
- [ ] All 22 BLOCKING acceptance criteria (CA1–CA22) have corresponding unit or integration
  tests in `tests/unit/card_acquisition/` and `tests/integration/card_acquisition/` that pass
  using `World::new()` + message injection — no Lightyear session required.
- [ ] `ResMut<ShopStates>` appears in exactly one system (`card_acquisition_tick_system`) —
  code review gate on every CA PR.
- [ ] `MessageReceiver<C2SPurchaseCard>` and `MessageReceiver<C2SRefreshShop>` each appear in
  exactly one system — code review gate.
- [ ] `economy.refund_gold()` is called before any return path following a `spend_gold()` +
  failed `distribute()` — verified by CA18 integration test with fault injection.
- [ ] `displayed_this_draft` is NOT cleared on `ShopUnlock` trigger — verified by CA16 and
  CA22 tests.
- [ ] `card_acquisition_tick_system` is scheduled after `rsm_tick_system` — verified by Bevy
  schedule graph dump.

---

## Related Decisions

- `docs/architecture/adr-009-rsm-phase-state.md` — `RoundState` resource pattern that
  `ShopStates` mirrors; phase exclusion that makes `PlayerHands` multi-writer safe.
- `docs/architecture/adr-010-rsm-event-bus.md` — Event catalog; `ShopRefreshTriggered`
  added to this catalog as part of this ADR's authoring pass.
- `docs/architecture/adr-013-auction-system-state.md` — `AuctionState` resource pattern;
  spend/refund atomicity pattern (CA18 mirrors release-before-reserve from ADR-013 CA18).
- `docs/architecture/adr-005-server-side-rng.md` — ServerRng seeds consumed by draw pipeline.
- `docs/architecture/adr-006-card-data-schema.md` — CardPool interface (`draw_*`, `distribute`,
  `is_available`).
- `docs/architecture/adr-008-lightyear-channel-config.md` — Reliable unicast channel for
  `S2CDraftOffering` and `S2CShopSlots`.
- `design/gdd/card-acquisition.md` — Complete card acquisition specification; all BLOCKING
  acceptance criteria this ADR enables.
- `design/gdd/round-state-machine.md` — RSM phase sequence; DRAFT sub-phase entries that
  emit `ShopRefreshTriggered`.
- `design/gdd/economy-system.md` — `spend_gold`, `refund_gold` API contracts.
- `design/gdd/network-protocol.md` — `S2CDraftOffering`, `S2CShopSlots` wire schemas.
