# Network Protocol

> **Status**: In Review — revised 2026-04-29 (29 blockers resolved from /design-review R2)
> **Author**: User + Agents
> **Last Updated**: 2026-04-29
> **Implements Pillar**: No idle spectating · Auction as signature

## Overview

The Network Protocol owns the complete wire layer between the Lanes and Lies server and its clients. It defines every client-to-server (C2S) message type used to express player intent (bid placements, card purchases, placement submissions, ready signals), every server-to-client (S2C) message type used to broadcast authoritative outcomes (phase transitions, card reveals, economy updates, resolution replays), the Lightyear channel tier assigned to each message (reliable vs. unreliable), and the Lightyear component replication strategy used to keep client board views synchronized between explicit events. It is not a game system — it carries no logic, owns no game state, and makes no game decisions. Its sole responsibility is accurate, ordered, and versioned delivery of the signals that all other systems depend on to function correctly across the network. This GDD also specifies the late-joiner and reconnection full-state sync protocol — the strategy for restoring a complete game snapshot to a client connecting mid-game, which phase-transition broadcasts alone cannot achieve.

## Player Fantasy

Players never see the Network Protocol, but they feel it the instant it fails. Three moments make this load-bearing: (1) the closing seconds of an auction, where bid order and timer authority decide who pays what — desync here turns "I read them" into "the server cheated me"; (2) the simultaneous lane reveal, which must be atomic — a staggered reveal leaks information and breaks "I fooled them"; (3) the always-on information stream that powers "Zero idle time" — dropped or stale state turns active watching into confused waiting. The protocol has no fantasy of its own; it is the substrate that lets the other four player fantasies survive contact with the network.

## Detailed Rules

### Core Rules

**Rule 1 — Authority model.** The server is the sole source of game truth. Clients hold read-only mirrors and express intent via C2S messages only — they never assert state directly. All game logic, validation, and state mutation happens server-side.

**Rule 2 — Channel assignment.** Every message uses exactly one channel tier:
- **Reliable** (ordered, guaranteed): phase transitions, game-over, purchases, bids, placement submissions, resolution events, card acquisitions, economy updates, connection events, snapshot delivery
- **Unreliable / Component Replication** (best-effort, self-correcting): live unit positions (`BoardPosition`), unit stats for display (`UnitStats`), objective HP ticks (`ObjectiveHp`), prism presence (`PrismPresence`) — state that self-corrects on the next frame if a packet is dropped

**Rule 3 — Protocol versioning and reconnect identity.** After the transport connection is established, the client sends `C2SHello` before any other message. On a fresh connect, `session_token` is `None`. On a reconnect, the client includes the `SessionToken` received in `S2CHandshake` at first connect — this is the sole mechanism by which the server maps the new `ClientId` (assigned by Lightyear on every WebSocket connect) back to the existing session slot. The server responds with `S2CHandshake` on version match, or `S2CHandshakeRejected` and closes the connection on mismatch or an unrecognised session token. No other C2S messages are processed until the handshake completes. If `C2SHello` is not received within `hello_timeout_ms` (default: 5000ms), the server closes the connection. `HANDSHAKING` is a protocol-layer state that precedes any RSM phase — the RSM does not observe it.

**Rule 4 — Phase-gated message acceptance.** The server silently discards any C2S message not valid in the current RSM phase (see Valid C2S Messages table). Discards are logged server-side for diagnostics. No error message is sent to the client — this avoids timing attacks based on error response latency.

**Rule 5 — Submission atomicity.** A C2S submission (placement, bid, purchase) is not acknowledged until the server has written it to authoritative state. The client must not advance its local view until the corresponding S2C acknowledgement or S2C broadcast confirms the action.

**Rule 6 — Replication scope.** Lightyear component replication delivers shared public board state to all players. Private per-player state (hand, gold, mana, pool) is unicast via targeted reliable S2C messages and never broadcast. The server sends one `S2CGameSnapshot` per connecting player, with opponent secret fields stripped — a single broadcast snapshot is forbidden.

**Rule 7 — Reconnect snapshot.** On any client connection (initial join or reconnect), before any other messages, the server sends `S2CGameSnapshot` containing complete authoritative game state. The client discards locally buffered messages received before snapshot processing completes and rebuilds its view from scratch.

**Rule 8 — Disconnect detection.** Disconnect detection uses Lightyear's built-in `OnDisconnected` / `OnConnected` events **and** a mandatory application-layer `C2SHeartbeat`. WebSocket connections from WASM browsers can enter half-open TCP states (tab backgrounded, OS sleep, mobile radio switch) where `OnDisconnected` does not fire for 2–7 minutes. The client sends `C2SHeartbeat {}` every ~5 seconds on the reliable channel; the server resets `disconnect_trackers[player]` on each heartbeat received. The RSM owns the `disconnect_grace_seconds = 30s` threshold and the GAME_OVER transition. The protocol broadcasts `S2COpponentDisconnected { grace_remaining_ms }` on `OnDisconnected` so the remaining player sees the countdown.

---

### C2S Messages

| Message | Payload | Valid Phase(s) | Notes |
|---|---|---|---|
| `C2SHello` | `{ protocol_version: u32, session_token: Option<SessionToken> }` | HANDSHAKING | Client sends first. `None` = fresh connect. `Some(token)` = reconnect; server uses token to map the new Lightyear `ClientId` to the existing session slot. |
| `C2SPurchaseCard` | `{ card_id: CardId }` | DRAFT_INITIAL, DRAFT_SHOP | Server validates: gold ≥ cost, hand < 10, card in shop |
| `C2SRefreshShop` | `{}` | DRAFT_SHOP | Costs 1g server-side; not valid in DRAFT_INITIAL |
| `C2SActivateCard` | `{ card_id: CardId }` | DRAFT_INITIAL, DRAFT_SHOP | Play instant-effect card from hand during DRAFT (reserve spells, Gelure, etc.); no board target |
| `C2SSignalReady` | `{ retract: bool }` | DRAFT_INITIAL, DRAFT_SHOP | `false` = signal ready; `true` = retract ready signal |
| `C2SPlaceBid` | `{ amount: u32 }` | DRAFT_AUCTION | Server validates: `amount ≥ last_accepted_bid + 1`, `player.gold ≥ amount` |
| `C2SSubmitPlacement` | `{ placements: Vec<PlacedCard> }` | PLACEMENT | Batch submit; marks player as submitted in RSM; empty list = zero new cards this round |
| `C2SAcknowledgeResult` | `{}` | GAME_OVER | Client confirms result rendered; server may clean up session |
| `C2SHeartbeat` | `{}` | All phases | Sent every `heartbeat_interval_ms` (default 5000ms); server resets disconnect timer on receipt. Mandatory for WASM/WebSocket — `OnDisconnected` is unreliable in browser half-open TCP states. **Note:** WASM browsers stop `requestAnimationFrame` when the tab is backgrounded — the heartbeat cannot fire while the tab is invisible. Players who background the tab for >`disconnect_grace_seconds` will forfeit. Surface a "return to tab" warning on tab-restore via the Page Visibility API. |

**LOBBY phase messages** — defined in `game-session-system.md`; listed here for phase-gating (Rule 4):

| Message | Payload | Valid Phase(s) | Notes |
|---|---|---|---|
| `C2SCreateRoom` | `{ mode: GameMode }` | LOBBY (pre-session) | Processed before session exists; creates a new session |
| `C2SJoinRoom` | `{ room_code: String, requested_slot: u8 }` | LOBBY | Joins an existing session by room code and slot index |
| `C2SSelectClass` | `{ class_id: ClassId }` | LOBBY | Preview selection — reversible; not broadcast to others |
| `C2SConfirmClass` | `{ class_id: ClassId }` | LOBBY | Irrevocable class lock — triggers reveal when all locked |

The LOBBY phase is a protocol-layer state entered after `IN_GAME` handshake completes and before RSM transitions to `DRAFT_INITIAL`. All LOBBY C2S messages are phase-gated: messages arriving after LOBBY ends are silently discarded per Rule 4. Semantics owned by `game-session-system.md`.

```
struct PlacedCard {
    card_id:        CardId,
    target:         PlayTarget,
    reserve_amount: u32,  // mana drawn from reserve pool; card.cost - reserve_amount drawn from current-round mana
                          // 0 = pay all from current; card.cost = pay all from reserve; n = split
}

enum PlayTarget {
    BoardCell  { lane: u8, cell: u8 },            // Minion (cell ≤ spawn_range), Trap, Structure
    TargetUnit { lane: u8, unit_id: EntityId },   // Targeted spell — target must be on board from prior rounds
    TargetObj  { player_id: PlayerId, lane: u8 }, // Objective-targeting spell (prism Lane 1/5, direct spells)
    LaneWide   { lane: u8 },                      // Field — lane-wide effect
    Instant,                                      // Untargeted spell, Order
}
```

**Placement model:** `C2SSubmitPlacement` is a single batch — the player finalizes their entire selection in one message. Once sent, the submission cannot be retracted. Client UI manages local selection state; the server only sees the final batch.

**C2SActivateCard acknowledgement model:** There is no dedicated `S2CCardActivated` acknowledgement. Per Rule 5, the client must not advance its local view until S2C confirms the action. For `C2SActivateCard`, confirmation arrives through the existing economy/hand messages: a successful instant card play produces `S2CGoldUpdate` and/or `S2CCardAcquired` as side effects. If neither arrives within a reasonable timeout, the client must treat the activation as rejected and revert local optimistic state. Cards with no observable side effects (e.g., a no-cost, no-draw Order) should include a `S2CGoldUpdate` (even if gold is unchanged — confirm as a no-op) to provide the required confirmation signal.

**`C2SSubmitPlacement` server validation:** The server validates the full batch before accepting: `sum(placements[i].reserve_amount) ≤ player.reserve_mana` AND `sum(card[i].cost - placements[i].reserve_amount) ≤ player.current_mana`. If any card is not in the player's hand, or any `lane`/`cell` value is out of range (1–5 / 1–8), the entire batch is silently discarded (Rule 4). Partial acceptance is not supported.

**Silence = pass at auction:** No `C2SPassBid` message. If a player does not bid, the timer runs to zero and the last bidder wins.

**Targeted spell provisional assumption:** `PlayTarget::TargetUnit` can only target units already on the board from prior rounds — not units in the current PLACEMENT buffer (unrevealed). If the Keyword System GDD requires targeting newly placed units, `PlayTarget` must be extended.

---

### S2C Messages

| Message | Channel | Scope | Payload |
|---|---|---|---|
| `S2CHandshake` | Reliable | Unicast | `{ protocol_version: u32, session_id: SessionId, session_token: SessionToken }` — client must persist `session_token` for reconnect |
| `S2CHandshakeRejected` | Reliable | Unicast | `{ server_version: u32, client_version: u32 }` |
| `S2CGameSnapshot` | Reliable | Unicast | Full game state — see Section D |
| `S2CPhaseChanged` | Reliable | Broadcast | `{ phase: RoundPhase, round_number: u32, timer_duration_ms: u32 }` |
| `S2CGameOver` | Reliable | Broadcast | `{ loser: Option<PlayerId>, round: u32, reason: GameOverReason }` — `None` = Draw (both players lose; reason = `Draw` for mutual destruction/disconnection/resolution timeout) |
| `S2CGoldUpdate` | Reliable | Unicast | `{ gold: u32, current_mana: u32, reserve_mana: u32, mana_cap: u8 }` |
| `S2CGoldBroadcast` | Reliable | Broadcast | `{ player_id: PlayerId, gold: u32 }` — satisfies AC M7; opponent gold always visible |
| `S2CCardAcquired` | Reliable | Unicast | `{ card_id: CardId, source: CardSource }` |
| `S2CShopSlots` | Reliable | Unicast | `{ slots: Vec<Option<CardId>> }` — `None` = empty slot (dedup exhaustion or pool exhaustion for that slot type) |
| `S2CDraftOffering` | Reliable | Unicast | `{ card_ids: Vec<CardId> }` — exactly 9 cards at DRAFT_INITIAL (fewer only in stripped test fixtures) |
| `S2CPoolUpdate` | Reliable | Unicast | `{ updates: Vec<(CardId, u8)> }` — delta `copies_remaining` |
| `S2CPlacementReveal` | Reliable | Broadcast | `{ placements: Vec<PlacedCard> }` — atomic simultaneous reveal; both players receive this as the sole signal that placement is closed. Client MUST render from this payload, not from pre-arrived component replication, to honour the simultaneous-reveal guarantee. |
| `S2CResolutionEvent` | Reliable | Broadcast | `{ events: Vec<TaggedEvent> }` — ordered sub-step replay log |
| `S2CAuctionCard` | Reliable | Broadcast | `{ card_id: CardId, starting_price: u32 }` |
| `S2CAuctionBidAccepted` | Reliable | Broadcast | `{ bidder: PlayerId, amount: u32, new_timer_ms: u32 }` |
| `S2CAuctionSettled` | Reliable | Broadcast | `{ winner: Option<PlayerId>, amount: u32 }` |
| `S2CAuctionBidRejected` | Reliable | Unicast (bidder) | `{ reason: BidRejectedReason }` — sent when a valid-phase `C2SPlaceBid` fails server validation. Never sent if the auction timer has already expired (`S2CAuctionSettled` is the terminal signal). |
| `S2COpponentDisconnected` | Reliable | Unicast (remaining player) | `{ player_id: PlayerId, grace_remaining_ms: u32 }` — sent to the remaining connected player only; NOT to the disconnecting player's connection. |
| `S2COpponentReconnected` | Reliable | Broadcast | `{ player_id: PlayerId }` |
| `S2CRoomCreated` | Reliable | Unicast (creator) | `{ room_code: String, mode: GameMode, slots: Vec<SessionSlot> }` |
| `S2CJoinAck` | Reliable | Unicast (joiner) | `{ mode: GameMode, slots: Vec<SessionSlot> }` — full slot state so joiner can render lobby |
| `S2CJoinRejected` | Reliable | Unicast | `{ reason: JoinRejectedReason }` — enum: `SlotOccupied`, `SessionFull`, `RoomNotFound`, `InvalidSlot`, `AlreadyInSession`, `SessionInProgress`, `InvalidMode` |
| `S2CSlotUpdated` | Reliable | Broadcast | `{ slots: Vec<SessionSlot> }` — broadcast on any slot state change (join, disconnect, free) |
| `S2CClassLocked` | Reliable | Unicast (locking player) | `{ class_id: ClassId }` — confirms player's own lock; not visible to others until `S2CClassesRevealed` |
| `S2CClassesRevealed` | Reliable | Broadcast | `{ player_class_map: Map<PlayerId, ClassId> }` — sent only when ALL slots have locked |
| `S2CConfirmClassRejected` | Reliable | Unicast | `{ reason: ConfirmClassRejectedReason }` — enum: `ClassAlreadyConfirmed` |
| `S2CSessionCancelled` | Reliable | Broadcast | `{ reason: SessionCancelledReason }` — enum: `LobbyTimeout`, `PlayerDisconnected` |
| `S2CSangMepriseReveal` | Reliable | Unicast (opponent) | `{ identities: Vec<(lane: u8, is_fake: bool)> }` — reveals selected objective identities to the opponent. Decided in `objective-system.md` Open Question 6 (Option B: targeted unicast). Sent by Objective System; protocol delivers. |

**Component Replication (Lightyear replicated ECS components — unreliable, all players):**

| Component | Fields | Notes |
|---|---|---|
| `NetworkId` | `id: u64` | Server-assigned monotonic u64 identifying this entity on the wire. Present on every replicated entity. Client maps `NetworkId → Entity` via a `HashMap<u64, Entity>` populated from `S2CGameSnapshot` + replication. This is `EntityId` in all protocol message fields. |
| `BoardPosition` | `lane: u8, cell: u8` | Unit positions for board rendering |
| `UnitStats` | `hp: u8, atk: u8, ar: u8` | Health bars and combat preview. `ar` = armor rating (damage reduction class) |
| `CardOwner` | `player_id: PlayerId` | Team color / ownership display |
| `ObjectiveHp` | `hp: u8` | Real/fake identity never replicated — revealed only via `S2CResolutionEvent::ObjectiveDestroyed` |
| `PrismPresence` | `collected: bool` | Prism availability on board |

**Not replicated via components:** `gold`, `current_mana`, `reserve_mana`, hand, pool — delivered via unicast reliable S2C messages (`S2CGoldUpdate`, `S2CCardAcquired`, etc.) rather than Lightyear component replication. However, **opponent gold IS public** and included in `S2CGameSnapshot.PlayerSnapshot` and `S2CGoldBroadcast` — "not replicated via components" does not mean "private." Only hand, shop slots, pool, and real/fake objective identity are secret.

**Lane and cell validation contract:** All `lane` fields are 1-indexed (valid range 1–5). All `cell` fields are 1-indexed absolute (valid range 1–8). The server rejects any C2S message containing out-of-range lane or cell values with a silent discard (Rule 4). Clients must not send values outside these ranges.

**`S2CPoolSnapshot` superseded:** `card-data-pool.md` references `S2CPoolSnapshot` as a standalone reconnect message. This GDD supersedes that reference — pool state is delivered within `S2CGameSnapshot.PlayerSnapshot.pool_snapshot`. There is no separate `S2CPoolSnapshot` message. The card-data-pool.md cross-reference should be updated accordingly.

> ⚠️ **HIGH-RISK — Lightyear 0.26 verification required before implementation:**
> 1. **Per-client unicast API** — required for `S2CGameSnapshot`, `S2CGoldUpdate`, `S2CCardAcquired`. Verify `ConnectionManager` unicast API in Lightyear 0.26 docs.
> 2. **Component visibility filtering** — Trap card identity must be hidden from the opponent. Proposed solution: two separate components — `TrapPresence { has_trap: bool }` (broadcast to all players) + `TrapIdentity { card_id: CardId }` (owner-only). Verify whether Lightyear 0.26 supports per-client entity visibility, or whether this requires explicit interest-management scoping.
> 3. **Reliable channel ordering** — `S2CResolutionEvent` must arrive before `S2CPhaseChanged(DRAFT_SHOP)`. Verify in-order delivery guarantee across message types on the same reliable channel.

---

### States and Transitions

| Protocol State | Description | Enters from | Exits to |
|---|---|---|---|
| `HANDSHAKING` | Transport connected; `C2SHello` version check pending. No game state visible. | Transport connect | `IN_GAME` on `S2CHandshake`; connection close on version mismatch or `hello_timeout_ms` |
| `IN_GAME` | Handshake complete; all C2S/S2C traffic active. Covers all RSM phases from LOBBY → GAME_OVER. | `HANDSHAKING` success; `RECONNECTING` success | `RECONNECTING` on transport drop; terminal on `C2SAcknowledgeResult` after GAME_OVER |
| `RECONNECTING` | Transport dropped. Server holds session for `disconnect_grace_seconds`. | `IN_GAME` on transport drop | `IN_GAME` on reconnect + snapshot delivery; terminal on grace period expiry → RSM GAME_OVER |

---

### Interactions with Other Systems

| System | Protocol receives from it | Protocol delivers on its behalf |
|---|---|---|
| **Round State Machine** | Phase change events, GAME_OVER data | `S2CPhaseChanged`, `S2CGameOver` (reliable broadcast) |
| **Economy System** | Per-player gold/mana change events | `S2CGoldUpdate` (reliable unicast) |
| **Card Data & Pool** | Shop slots per player, draft offering, pool delta per purchase | `S2CShopSlots`, `S2CDraftOffering`, `S2CPoolUpdate` (reliable unicast) |
| **Board / Lane System** | Placement reveal data, resolution replay log, unit position / objective HP updates | `S2CPlacementReveal`, `S2CResolutionEvent` (reliable broadcast); `BoardPosition`, `ObjectiveHp` (component replication) |
| **Server-side RNG** | RNG results via consuming systems | No direct protocol messages — consuming systems broadcast results after reading from RNG |
| **Auction System** *(GDD not yet written)* | Bid accepted events, auction settled event, auction card selection | `S2CAuctionCard`, `S2CAuctionBidAccepted`, `S2CAuctionSettled` (reliable broadcast) |
| **Combat Resolution** *(GDD not yet written)* | Sub-step events, kill/objective/gold-award events | `S2CResolutionEvent`, `S2CGoldUpdate` (reliable) |
| **Game Session System** | Lobby phase messages, session metadata, opponent status changes | `S2CRoomCreated`, `S2CJoinAck`, `S2CJoinRejected`, `S2CSlotUpdated`, `S2CClassLocked`, `S2CClassesRevealed`, `S2CConfirmClassRejected`, `S2CSessionCancelled` (all reliable); `S2CHandshake`, `S2COpponentDisconnected`, `S2COpponentReconnected` (reliable) |

## Formulas

*The Network Protocol has no game-math formulas. This section defines the two complex data structures that serve the same load-bearing role: `S2CGameSnapshot` (the complete reconnect/late-joiner state schema) and `S2CResolutionEvent` (the combat replay log).*

### D.1 — S2CGameSnapshot Schema

Sent unicast per player on every connect and reconnect, before any other S2C messages. The server produces one snapshot per player — opponent secret fields (hand, pool, real/fake objective identity) are stripped before sending.

```
S2CGameSnapshot {
    protocol_version:       u32,
    round_number:           u32,
    phase:                  RoundPhase,
    timer_remaining_ms:     u32,     // milliseconds left in the current phase timer; 0 if no active timer

    players: Vec<PlayerSnapshot>,    // one entry per player in the session
    board:   BoardSnapshot,
}

PlayerSnapshot {
    player_id:         PlayerId,
    gold:              u32,
    current_mana:      u32,
    reserve_mana:      u32,
    spawn_range_cells: u8,           // cells available for Minion placement: 1–3
    mana_cap:          u8,           // current mana cap (default 10; can increase to 11–12 via fake reward)
    submitted:         bool,         // true if this player has submitted their PLACEMENT this round

    // Own player only (secret — stripped from opponent's copy):
    hand:          Vec<CardId>,
    shop_slots:    Vec<CardId>,             // current personal shop offering (3 cards)
    pool_snapshot: Vec<(CardId, u8)>,       // (card_id, copies_remaining) — full pool state

    // Own player objectives (opponent's copy has is_real = false for all):
    objectives:          Vec<ObjectiveSnapshot>,           // 5 entries, one per lane
    opponent_objectives: Vec<OpponentObjectiveSnapshot>,   // 5 entries
}

ObjectiveSnapshot {
    lane:         u8,
    hp:           u8,
    is_real:      bool,   // owner knows; opponent always receives false — revealed only at destruction
    is_destroyed: bool,
}

OpponentObjectiveSnapshot {
    lane:         u8,
    hp:           u8,
    is_destroyed: bool,
    was_fake:     Option<bool>,  // None = not yet destroyed; Some(true/false) = revealed at destruction
    // SERVER INVARIANT: if is_destroyed = true then hp MUST = 0. Client may assert this; violation is a server bug.
}

BoardSnapshot {
    units:      Vec<UnitBoardState>,
    traps:      Vec<TrapBoardState>,
    structures: Vec<StructureBoardState>,
    fields:     Vec<FieldBoardState>,
    prisms:     Vec<PrismBoardState>,   // required: component replication not re-sent on reconnect
}

PrismBoardState {
    lane:      u8,
    collected: bool,   // true = prism collected this game; false = still available
}

UnitBoardState {
    unit_id:    EntityId,   // server-assigned NetworkId.id — use to look up local ECS Entity
    card_id:    CardId,
    owner:      PlayerId,
    lane:       u8,
    cell:       u8,
    current_hp: u8,
    atk:        u8,
    ar:         u8,         // armor rating
}

TrapBoardState {
    trap_id: EntityId,
    owner:   PlayerId,
    lane:    u8,
    cell:    u8,
    card_id: Option<CardId>,   // None if this trap belongs to the opponent (identity hidden)
}

StructureBoardState {
    structure_id: EntityId,
    card_id:      CardId,
    owner:        PlayerId,
    lane:         u8,
    cell:         u8,
    current_hp:   u8,
}

FieldBoardState {
    field_id: EntityId,
    card_id:  CardId,
    owner:    PlayerId,
    lane:     u8,
}
```

**Secret information rule:** The server MUST NOT send a broadcast `S2CGameSnapshot`. Each player receives exactly one snapshot with their own private data intact and the opponent's private data stripped. This is the enforcement mechanism for the "no client-side game state" architecture.

**Public vs. secret in `PlayerSnapshot`:** The `players` Vec contains a `PlayerSnapshot` for each player in the session. For the recipient's own entry, all fields are populated. For the opponent's entry, the "Own player only" fields (hand, shop_slots, pool_snapshot, objectives, opponent_objectives) are stripped. All other fields — including `gold`, `spawn_range_cells`, `mana_cap`, `submitted` — are present in BOTH entries and are therefore **public information**. `spawn_range_cells` for the opponent IS transmitted and must be used to correctly render the opponent's placement zone.

**Post-reconnect message sequencing:** After `S2CGameSnapshot` is enqueued on the reliable channel, the server MUST NOT enqueue any additional S2C messages until the same system has finished processing the reconnect event. This prevents live-game messages (e.g., a concurrent `S2CGoldUpdate` from another system in the same frame) from arriving before the snapshot in a separate system ordering. All post-snapshot live messages must be enqueued in a system scheduled AFTER the snapshot system in the same `Update` schedule.

```
// Auction state during DRAFT_AUCTION (None in all other phases):
auction_state: Option<AuctionSnapshot>,

AuctionSnapshot {
    card_id:             CardId,
    last_accepted_bid:   u32,           // last bid amount accepted; minimum valid next bid = last_accepted_bid + 1
    current_leader:      Option<PlayerId>,
    timer_remaining_ms:  u32,           // milliseconds remaining on the auction timer
}
```

---

### D.2 — S2CResolutionEvent Schema

Sent once per RESOLUTION on the reliable channel, after `S2CPlacementReveal` and before `S2CPhaseChanged(DRAFT_SHOP)`. The client replays `events` in array order to drive animations; `sub_step` is the animation grouping key.

```
S2CResolutionEvent {
    events: Vec<TaggedEvent>,
}

TaggedEvent {
    sub_step: u8,       // 1–6, matches the 6 RESOLUTION sub-steps
                        // SERVER CONTRACT: MUST NOT emit sub_step outside 1–6.
                        // CLIENT CONTRACT: treat any out-of-range sub_step as a fatal desync — request a full snapshot immediately.
    event:    ResolutionEvent,
}

enum ResolutionEvent {
    UnitMoved {
        unit_id:   EntityId,
        from_lane: u8,
        to_lane:   u8,   // same as from_lane for in-lane movement; differs for CHANGE LANE keyword
        from_cell: u8,
        to_cell:   u8,
    },
    UnitDied {
        unit_id:   EntityId,
        lane:      u8,
        cell:      u8,
        killer_id: Option<EntityId>,  // None = damage-over-time, structure effect, or spell
    },
    TrapTriggered {
        trap_id:            EntityId,
        triggering_unit_id: EntityId,
        lane:               u8,
        cell:               u8,
    },
    ObjectiveDamaged {
        target_player_id: PlayerId,
        lane:             u8,
        hp_before:        u8,
        hp_after:         u8,
        attacker_id:      Option<EntityId>,
    },
    ObjectiveDestroyed {
        target_player_id: PlayerId,
        lane:             u8,
        was_fake:         bool,   // both players learn real/fake status simultaneously at destruction
    },
    PrismCollected {
        player_id: PlayerId,
        lane:      u8,
    },
    GoldAwarded {
        player_id: PlayerId,
        amount:    u32,
        reason:    GoldAwardReason,
    },
    ManaCapIncreased {
        player_id: PlayerId,
        new_cap:   u8,   // the player's mana_cap after the fake objective reward
    },
    StructureDamaged {
        structure_id: EntityId,
        lane:         u8,
        cell:         u8,
        hp_before:    u8,
        hp_after:     u8,
        attacker_id:  Option<EntityId>,
    },
    StructureDestroyed {
        structure_id: EntityId,
        lane:         u8,
        cell:         u8,
        attacker_id:  Option<EntityId>,
    },
    TrapDestroyed {
        trap_id:     EntityId,
        lane:        u8,
        cell:        u8,
        card_id:     Option<CardId>,   // Some = own trap (identity known to owner); None = opponent's trap (identity not revealed on non-trigger destruction)
        attacker_id: Option<EntityId>, // None = destroyed by non-combat effect (spell, structure)
    },
}

enum GoldAwardReason {
    KillReward,       // +1g per kill
    ObjectiveReward,  // +3g per objective destroyed (real or fake)
    PrismReward,      // gold awarded on prism collection (if any — amount per prism defined in Prism System GDD)
}
```

---

### D.3 — Supporting Enum Definitions

```
enum CardSource {
    ShopPurchase,
    DraftSelection,
    AuctionWon,
    PrismLane1,      // "1 damage to a chosen objective" spell added to hand
    PrismLane2,      // "+1 reserve mana" spell added to hand
    PrismLane3,      // random draw from shop pool
    PrismLane4,      // "+1 reserve mana" spell added to hand (same reward as Lane 2)
    PrismLane5,      // "1 damage to a chosen objective" spell added to hand
    KeywordEffect,   // card added to hand by a keyword trigger (DEATH, FINAL BLOW, class ability)
}

struct PlacedCard {   // element of S2CPlacementReveal.placements — same name as C2S payload struct; C2S has reserve_amount, S2C omits it (mana accounting is server-side)
    card_id:  CardId,
    owner_id: PlayerId,
    target:   PlayTarget,
}

// Wire identity types
type SessionToken = [u8; 16];   // UUID v4 (128-bit), server-generated at first connect, included in S2CHandshake
type EntityId = u64;            // server-assigned monotonic counter per session; mapped to local ECS Entity via HashMap<EntityId, Entity> on each client

enum BidRejectedReason {
    InsufficientGold,   // player.gold - reserved_gold < amount
    AmountTooLow,       // amount <= current bid (must be >= last_accepted_bid + 1)
    AuctionExpired,     // server timer fired between client send and server receive — S2CAuctionSettled follows immediately
    AlreadyLeader,      // bidder == current_leader; self-bids are rejected (auction-system.md Rule 4)
    HandFull,           // bidder.hand_size == 10; must play a card before bidding (auction-system.md Rule 4)
}
```

## Edge Cases

- **If `S2CResolutionEvent` and `S2CPhaseChanged` are sent on the same reliable channel, `S2CResolutionEvent` MUST be enqueued first**: The client must not render the phase change until the resolution replay is complete. If these are ever moved to separate channels, an explicit sequence number is required — this is an implementation invariant, not enforced by the wire protocol itself.

- **If a client reconnects during PLACEMENT after having already submitted**: The `PlayerSnapshot.submitted` field carries `true`. The client skips the placement UI and renders "waiting for opponent" immediately — it does not re-present card selection.

- **If a client reconnects during PLACEMENT before having submitted**: The snapshot delivers `timer_remaining_ms` with the live countdown already running (in milliseconds). The client re-presents the placement UI. No special case needed, provided `timer_remaining_ms` is accurate in the snapshot.

- **If `C2SHello` is sent twice on the same connection**: Server discards the second silently if handshake is already complete. `C2SHello` is NOT a retry mechanism — to retry, the client must close and reopen the transport connection.

- **If `C2SHello` is never received within `hello_timeout_ms` (5000ms)**: The server closes the transport connection. No `S2CHandshakeRejected` is sent — silence is the response to a non-speaking client.

- **If `C2SPurchaseCard` is in-flight when the transport drops and the client reconnects**: The in-flight message was on the dead connection and was never received by the server. The snapshot reflects authoritative state without the purchase. The client discards any optimistic local UI and rebuilds from snapshot. The player must re-issue the intent manually — no automatic retry.

- **If `C2SSubmitPlacement` is received twice for the same player in the same PLACEMENT phase**: The server processes the first submission, marks the player as submitted in server state (preserving the first submission's card list), and silently discards the second without any S2C response. No `S2CPlacementAcknowledged` exists — submissions are recorded silently per Rule 5 (NP-6). The client must use `S2CPlacementReveal` or the snapshot's `submitted` field as the sole confirmation signal.

- **If `S2CGameSnapshot` arrives during PLACEMENT and the opponent has already submitted**: The snapshot's opponent entry carries `submitted: true`. The reconnecting client renders "opponent ready" immediately — the snapshot is the sole source of truth; missed events are never re-sent.

- **If `S2CAuctionBidAccepted` arrives at the same moment the client renders a locally-computed timer-expired state**: The server is authoritative. The client must not finalize the auction locally until `S2CAuctionSettled` arrives on the reliable channel. Any local "time's up" display is advisory only.

- **If `S2CPlacementReveal` arrives before the reconnecting player has processed their own submission state (reconnect mid-PLACEMENT)**: Receipt of `S2CPlacementReveal` is definitive — both players' submissions are now closed and both placements are revealed. The client renders the reveal payload directly regardless of local `submitted` state. No acknowledgement message exists; `S2CPlacementReveal` is the canonical closure signal.

- **If the client reconnects during DRAFT_AUCTION**: The snapshot's `auction_state` field (see Section D) delivers the current auction card, price, leader, and timer. Without this, the reconnecting client cannot render the auction panel correctly. `S2CAuctionCard` is not re-sent after reconnect — the snapshot is the sole reconnect source.

- **If `C2SAcknowledgeResult` is never received after GAME_OVER**: The server waits up to `ack_timeout_ms` (default: 10000ms) before cleaning up the session. The game result is persisted regardless — the acknowledgement is a UI handshake only, not a data commit.

- **If both players disconnect simultaneously (same server tick)**: Neither player can receive `S2COpponentDisconnected`. The server enters RECONNECTING for both players. Grace windows run independently from each player's disconnect time. If either player reconnects, normal flow resumes — the reconnected player receives a snapshot, and `S2COpponentDisconnected { grace_remaining_ms }` is sent immediately to inform them of the other player's state. If neither reconnects within `disconnect_grace_seconds` from the later disconnect, the RSM declares the session a Draw and fires `S2CGameOver { loser: None, reason: GameOverReason::Draw }`.

- **If `S2CAuctionCard` and `S2CPhaseChanged(DRAFT_AUCTION)` are enqueued in the same frame**: `S2CAuctionCard` MUST be enqueued on the reliable channel before `S2CPhaseChanged(DRAFT_AUCTION)`. The client must not enter the DRAFT_AUCTION UI state until it knows which card is being auctioned. This is an enqueue-order invariant enforced by the Auction System + Protocol integration, not by the channel itself.

- **If `C2SPlaceBid` arrives at the server after `S2CAuctionSettled` has already been dispatched (race on the last bid)**: The server does not send `S2CAuctionBidRejected` in this case — `S2CAuctionSettled` is the terminal signal and takes precedence. The server silently discards the late bid after settlement is final. The client should display the bid as "pending" until either `S2CAuctionBidAccepted` or `S2CAuctionSettled` resolves it.

## Dependencies

### Upstream Dependencies

| System | Type | Interface | Notes |
|---|---|---|---|
| **Game Config** | Hard | Reads `protocol_version`, `hello_timeout_ms`, `ack_timeout_ms`, `heartbeat_interval_ms`, `disconnect_grace_seconds` at startup | All protocol constants now in `game-config.md` ✓ |
| **Round State Machine** | Hard | RSM phase transition events drive `S2CPhaseChanged` broadcasts; RSM fires GAME_OVER data | Network Protocol has no phase logic — it is a delivery layer for RSM signals |
| **Economy System** | Hard | Economy fires per-player gold/mana change events; protocol delivers via `S2CGoldUpdate` | Economy System emits events; protocol handles delivery — no direct coupling |
| **Card Data & Pool** | Hard | Pool fires shop-refresh and draft-offering events; pool delta updates on purchase | `S2CShopSlots`, `S2CDraftOffering`, `S2CPoolUpdate` all sourced from Card Data & Pool |
| **Board / Lane System** | Hard | Board fires placement reveal data and resolution replay events; provides unit positions and objective HP for component replication | `S2CPlacementReveal` and `S2CResolutionEvent` are Board/Lane-sourced |
| **Server-side RNG** | Soft | RNG results are broadcast by consuming systems after reading from RNG — protocol never calls RNG directly | Indirect dependency only |

### Downstream Dependents

| System | Type | Interface | Notes |
|---|---|---|---|
| **Game Session System** *(Not Started)* | Hard | Session manages LOBBY and connection state; protocol delivers `S2CHandshake`, `S2COpponentDisconnected`, `S2COpponentReconnected` | Session GDD must define the LOBBY flow and `C2SReadyToStart` handling |
| **Auction System** *(Not Started)* | Hard | Auction System owns all bid state; protocol delivers `S2CAuctionCard`, `S2CAuctionBidAccepted`, `S2CAuctionSettled` | Auction System is the authority on bid acceptance; protocol carries results only |
| **Combat Resolution** *(Not Started)* | Hard | Combat produces all `ResolutionEvent` entries; protocol wraps and delivers in `S2CResolutionEvent` | Combat GDD must confirm `ResolutionEvent` enum variants are sufficient |
| **Objective System** | Hard | Objective System owns real/fake HP state; `ObjectiveDestroyed.was_fake` is sourced from Objective System; Sang Méprise ability triggers `S2CSangMepriseReveal` | Protocol delivers `S2CSangMepriseReveal` unicast to opponent per `objective-system.md` OQ6 (Option B) |
| **Keyword System** *(Not Started)* | Soft | Keywords that give cards to hand produce `S2CCardAcquired(source: KeywordEffect)`; targeted spell keywords may require `PlayTarget` extension | If any keyword requires targeted selection at play-time, `PlayTarget` enum must be extended |
| **All Presentation systems** | Soft | Board Rendering, Hand UI, Shop UI, HUD consume S2C messages and replicated components as their sole data source | No direct interface — presentation reads what the protocol delivers |

### Cross-system bidirectionality

- Board/Lane GDD confirms `protocol_version: u32` validated at LOBBY handshake. ✓
- Card Data Pool GDD lists `S2CPoolUpdate`, `S2CPoolSnapshot`, `S2CDraftOffering`, `S2CShopSlots`, `S2CAuctionCard` as requiring Network Protocol GDD for full definition. ✓
- RSM GDD lists Network Protocol as a hard downstream dependent. ✓

## Tuning Knobs

| Knob | Default | Safe Range | Too Low | Too High | Interacts With |
|---|---|---|---|---|---|
| `protocol_version` | 1 | N/A | — | — | Must match client and server exactly; any mismatch → `S2CHandshakeRejected`. Increment on any breaking wire change. This is a compatibility gate, not a balance knob. |
| `hello_timeout_ms` | 5000 | 2000–15000 | Legitimate slow-starting clients (WASM cold start) kicked before sending `C2SHello` | Slow detection of port-scanning / connection-flooding | `disconnect_grace_seconds` — both are connection safety nets on different layers |
| `ack_timeout_ms` | 10000 | 5000–30000 | Session cleans up before result screen finishes rendering | Dead sessions accumulate server-side memory | None — independent cleanup timer |
| `snapshot_max_bytes` | 16384 | 8192–65536 | Truncation risk — pool snapshot alone is ~900 bytes; full late-game board adds several hundred more; 4096 would be exceeded in normal play | No practical concern — snapshot is sent once per connect, not in the hot path | Server safety limit only; not related to WASM bundle budget |
| `heartbeat_interval_ms` | 5000 | 2000–15000 | Target client heartbeat send interval. Must be ≪ `disconnect_grace_seconds × 1000` (default: ≪ 30000ms). Too low: heartbeat traffic adds bandwidth. Too high: half-open connection detected late. | `disconnect_grace_seconds` — both are disconnect detection layers |

**Protocol constants now in `game-config.md`:** `protocol_version`, `hello_timeout_ms`, `ack_timeout_ms`, `heartbeat_interval_ms` ✓ (added in R2 revision).

**Cross-referenced constants (owned by Game Config):**

| Constant | Value | Source |
|---|---|---|
| `disconnect_grace_seconds` | 30 | `game-config.md` (via RSM) |
| `placement_timer_seconds` | 10 | `game-config.md` |
| `auction_timer_seconds` | 20 | `game-config.md` |

## Visual/Audio Requirements

N/A — This system is pure server-side infrastructure. It has no visual or audio output. All visual and audio feedback from the systems it carries is specified in their respective GDDs.

## UI Requirements

N/A — This system renders nothing. The protocol delivers data consumed by UI systems (HUD, Shop/Auction UI, Hand UI, Board Rendering); it does not own any UI elements.

## Acceptance Criteria

| # | Criterion | Type |
|---|---|---|
| NP-1 | **GIVEN** a client connects and sends `C2SHello` with a matching `protocol_version`, **WHEN** the server processes it, **THEN** the server sends `S2CHandshake` as the first S2C message on that connection. No other S2C message is sent before the handshake completes. | BLOCKING |
| NP-2 | **GIVEN** a client sends `C2SHello` with a mismatched `protocol_version`, **WHEN** the server processes it, **THEN** the server sends `S2CHandshakeRejected { server_version, client_version }` and closes the transport connection. | BLOCKING |
| NP-3 | **GIVEN** a client connects and sends no `C2SHello` within `hello_timeout_ms` (5000ms), **WHEN** the timeout fires, **THEN** the server closes the transport connection without sending any S2C message. | BLOCKING |
| NP-4 | **GIVEN** a client is IN_GAME and sends `C2SPurchaseCard` during PLACEMENT phase, **WHEN** the server processes it, **THEN** no S2C message is sent to that client in response and the player's gold is unchanged. | BLOCKING |
| NP-5 | **GIVEN** a client sends `C2SSubmitPlacement` during DRAFT_SHOP phase AND `player.submitted` was `false` before the message arrived, **WHEN** the server processes it, **THEN** `player.submitted` remains `false` (unchanged) and no S2C message of any kind is sent in response. | BLOCKING |
| NP-6 | **GIVEN** a player sends valid `C2SSubmitPlacement` during PLACEMENT, **WHEN** the server accepts it, **THEN** `player.submitted = true` in server state. No S2C message is sent to any player — the submission is recorded silently until `S2CPlacementReveal` fires. | BLOCKING |
| NP-7 | **GIVEN** units are on the board, **WHEN** the server updates `BoardPosition` or `UnitStats`, **THEN** no reliable S2C message is generated for those updates — delivery occurs via Lightyear component replication only. (Integration test — requires a live Lightyear session; `World::new()` unit tests cannot verify channel assignment.) | BLOCKING |
| NP-8 | **GIVEN** RESOLUTION completes and the server sends phase-exit messages, **WHEN** messages are captured in the reliable channel stream, **THEN** `S2CResolutionEvent` precedes `S2CPhaseChanged(DRAFT_SHOP)` in the sequence. (Integration test — requires a live Lightyear session to inspect message ordering on the reliable channel.) | BLOCKING |
| NP-9 | **GIVEN** a client reconnects mid-game, **WHEN** the server processes the new transport connection, **THEN** the first S2C message sent is `S2CGameSnapshot` — no `S2CPhaseChanged`, `S2CGoldUpdate`, or other game message precedes it. (Integration test — requires a live Lightyear session to inspect channel message ordering.) | BLOCKING |
| NP-10 | **GIVEN** a client reconnects during PLACEMENT after having already submitted, **WHEN** `S2CGameSnapshot` is processed, **THEN** the reconnecting player's `PlayerSnapshot.submitted = true` and the placement UI is not re-presented. | BLOCKING |
| NP-11 | **GIVEN** both players are IN_GAME, **WHEN** the server sends `S2CGoldUpdate` for Player A, **THEN** the server does not enqueue `S2CGoldUpdate` for Player B's `ClientId`. Gold is unicast to the owning player only. (Integration test — requires a live Lightyear session to verify delivery scope per `ClientId`.) | BLOCKING |
| NP-12 | **GIVEN** a player's transport connection drops, **WHEN** Lightyear's `OnDisconnected` event fires, **THEN** `S2COpponentDisconnected` is sent to the remaining player within the same `Update` schedule run. (Integration test — requires a live Lightyear session. "Same `Update` schedule run" is the granularity; a one-tick delay is not acceptable.) | BLOCKING |
| NP-13 | **GIVEN** a player has been disconnected for strictly more than `disconnect_grace_seconds`, **WHEN** the RSM evaluates disconnect trackers, **THEN** `S2CGameOver` is broadcast on the reliable channel. | BLOCKING |
| NP-14 | **GIVEN** `C2SSubmitPlacement` is sent twice by the same player in the same PLACEMENT phase, **WHEN** the server processes both, **THEN** the second submission is silently discarded — `player.submitted` remains `true` and no S2C message is sent in response to the second message. | BLOCKING |
| NP-15 | **GIVEN** a client is IN_GAME and sends `C2SHello` again on the same connection, **WHEN** the server processes it, **THEN** no `S2CHandshake` is sent in response and the existing session is not disrupted. | BLOCKING |
| NP-16 | **GIVEN** a two-player game is in progress and Player B reconnects, **WHEN** the server produces Player B's `S2CGameSnapshot`, **THEN** the snapshot does NOT contain: Player A's `hand`, `shop_slots`, `pool_snapshot`, or any `ObjectiveSnapshot.is_real = true` for Player A's objectives. | BLOCKING |
| NP-17 | **GIVEN** a player disconnects and reconnects within `disconnect_grace_seconds`, **WHEN** the server processes the reconnect, **THEN** `S2COpponentReconnected` is broadcast to the remaining player. | BLOCKING |
| NP-18 | **GIVEN** the server is in DRAFT_AUCTION and a player reconnects, **WHEN** `S2CGameSnapshot` is received, **THEN** `auction_state` is non-null and contains: `card_id`, `last_accepted_bid`, `current_leader` (or `None` if no bids yet), and `timer_remaining_ms`. | BLOCKING |
| NP-19 | **GIVEN** Player A's gold changes for any reason (purchase, kill reward, objective reward, prism reward, phase income — baseline + interest applied at end of RESOLUTION), **WHEN** the server processes the change, **THEN** `S2CGoldBroadcast { player_id: Player_A, gold: new_amount }` is delivered to ALL connected players including Player B. (Satisfies AC M7 — opponent gold always visible.) | BLOCKING |
| NP-20 | **GIVEN** a game is in PLACEMENT with at least 2 units on the board, **WHEN** a player reconnects and receives `S2CGameSnapshot`, **THEN** the snapshot contains: `round_number > 0`, `phase = PLACEMENT`, all board units present in `BoardSnapshot.units`, and the reconnecting player's `hand` is populated if they hold cards. (Integration test.) | BLOCKING |
| NP-21 | **GIVEN** a client reconnects during PLACEMENT before having submitted, **WHEN** `S2CGameSnapshot` is processed, **THEN** `PlayerSnapshot.submitted = false` for the reconnecting player, and the placement UI is re-presented with `timer_remaining_ms` already counting down from its current value (in milliseconds). | BLOCKING |
| NP-22 | **GIVEN** a client reconnects during PLACEMENT and the opponent has already submitted, **WHEN** `S2CGameSnapshot` is processed, **THEN** the opponent's `PlayerSnapshot.submitted = true` and the reconnecting client renders the "waiting for opponent" state. The snapshot is the sole authority — no further submission confirmation messages are sent. | BLOCKING |
| NP-23 | **GIVEN** Player A acquires a card (from shop, auction, prism, or keyword effect), **WHEN** the server sends `S2CCardAcquired`, **THEN** it is NOT delivered to Player B's transport connection. Card acquisition is private to the owning player. | ADVISORY |
| NP-24 | **GIVEN** the server receives `C2SHeartbeat` from Player A, **WHEN** the heartbeat is processed, **THEN** `disconnect_trackers[Player_A]` is reset to `disconnect_grace_seconds`. | BLOCKING |
| NP-25 | **GIVEN** Player A is IN_GAME and has not sent `C2SHeartbeat` for strictly more than `disconnect_grace_seconds` (server-side tracking via `disconnect_trackers`), **WHEN** the RSM evaluates disconnect trackers, **THEN** `S2CGameOver { loser: Some(Player_A), reason: GameOverReason::Disconnection }` is broadcast. This covers the WASM half-open TCP case that `OnDisconnected` does not reliably detect. | BLOCKING |
| NP-26 | **GIVEN** the server is in any RSM phase (DRAFT_INITIAL, DRAFT_SHOP, DRAFT_AUCTION, PLACEMENT, RESOLUTION, GAME_OVER), **WHEN** a `C2SHeartbeat` is received from a player, **THEN** the heartbeat is processed (timer reset) without triggering any other server state change. | BLOCKING |
| NP-27 | **GIVEN** the server has advanced past LOBBY (RSM phase is DRAFT_INITIAL or later) and a client sends `C2SCreateRoom`, `C2SJoinRoom`, `C2SSelectClass`, or `C2SConfirmClass`, **WHEN** the server processes it, **THEN** the message is silently discarded per Rule 4 — no S2C response is sent and no state changes. | BLOCKING |
| NP-28 | **GIVEN** both players have submitted their PLACEMENT and the server fires `S2CPlacementReveal`, **WHEN** the message is delivered, **THEN** ALL connected players receive it AND the `placements` field contains entries for BOTH players (both players' cards are in the reveal payload). | BLOCKING |
| NP-29 | **GIVEN** a player sends `C2SPlaceBid` during DRAFT_AUCTION with `amount < player.gold` but `amount <= last_accepted_bid` (amount too low), **WHEN** the server processes it, **THEN** `S2CAuctionBidRejected { reason: BidRejectedReason::AmountTooLow }` is sent unicast to the bidding player and no auction state changes. | BLOCKING |

## Open Questions

1. **Lightyear 0.26 per-client unicast API** — `S2CGameSnapshot`, `S2CGoldUpdate`, and `S2CCardAcquired` require unicast delivery to a specific `ClientId`. Verify `ConnectionManager` unicast API in Lightyear 0.26 docs before implementing. If it doesn't exist, the secret-information model breaks and the snapshot architecture must be redesigned. **Owner:** Network programmer. **Priority: HIGH — blocks implementation start.**

2. **Lightyear 0.26 component visibility filtering** — `TrapBoardState.card_id` requires per-client scoping (identity hidden from opponent). Verify whether Lightyear 0.26 supports field-level or entity-level replication filtering per client. If not, use two separate components (`TrapPresence` broadcast + `TrapIdentity` owner-only). **Owner:** Network programmer. **Priority: HIGH.**

3. **Reliable channel in-order delivery across message types** — `S2CResolutionEvent` must precede `S2CPhaseChanged(DRAFT_SHOP)` (NP-8). Verify whether Lightyear's reliable channel guarantees strict FIFO ordering across all message types on the same channel, or whether an explicit sequence number field is needed. **Owner:** Network programmer. **Priority: HIGH.**

4. ~~**New constants in `game-config.md`**~~ — **RESOLVED (R2 2026-04-29):** `protocol_version`, `hello_timeout_ms`, `ack_timeout_ms`, `heartbeat_interval_ms` added to `game-config.md` struct, Tuning Knobs, Interactions table, and GCN-DEFAULTS AC. Registry updated. ✓

5. **`PlayTarget::TargetUnit` extension** — If any Keyword System effect requires targeting a unit placed in the current PLACEMENT (not from prior rounds), `PlayTarget` must be extended. Provisional assumption: targeted spells can only reference prior-round units. **Owner:** Keyword System GDD. **Priority: MEDIUM — revisit when Keyword GDD is designed.**
