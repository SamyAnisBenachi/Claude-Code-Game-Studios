# Network Protocol

> **Status**: Approved — revised 2026-04-30 (R5 lean: 2 blockers resolved — Rule 8 C2SHeartbeat channel contradiction fixed; SeedBoardState owner: PlayerId added. R4 in-session: 23 blockers resolved — SubStepMarker 1-7 contract, reserved_gold in PlayerSnapshot+S2CGoldUpdate, GameOverReason/GrantedKeyword/HandshakeRejectedReason enums defined, PlacedCard renamed, C2SHeartbeat→unreliable channel, ObjectiveHp/BoardPosition RESOLUTION suppression rules, resistance_x/vulnerability_x in UnitBoardState, C2SRequestSnapshot defined, duplicate card_id validation, SessionToken entropy+localStorage, terminated-session handshake behavior, AuctionSnapshot bid-pending spec, HANDSHAKING rendering spec, simultaneous-reveal unreliable interaction, NP-6/NP-34 contradiction fixed, NP-31 split into per-keyword ACs, 10 new ACs NP-36–NP-45, SinistroState.damage field, OQ-6 priority→MEDIUM, OQ-7 HASTE reconnect added)
> **Author**: User + Agents
> **Last Updated**: 2026-04-30
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
- **Unreliable (liveness-only):** `C2SHeartbeat` — **MUST be unreliable, NOT on the reliable channel.** Using the reliable channel causes head-of-line blocking behind large `S2CResolutionEvent` batches, delaying heartbeat delivery and triggering false-positive disconnect-timer decrements.
- **Unreliable / Component Replication** (best-effort, self-correcting): live unit positions (`BoardPosition`), unit stats for display (`UnitStats`), objective HP ticks (`ObjectiveHp`), prism presence (`PrismPresence`) — state that self-corrects on the next frame if a packet is dropped

**Rule 3 — Protocol versioning and reconnect identity.** After the transport connection is established, the client sends `C2SHello` before any other message. On a fresh connect, `session_token` is `None`. On a reconnect, the client includes the `SessionToken` received in `S2CHandshake` at first connect — this is the sole mechanism by which the server maps the new `ClientId` (assigned by Lightyear on every WebSocket connect) back to the existing session slot. The server responds with `S2CHandshake` on version match, or `S2CHandshakeRejected { reason: HandshakeRejectedReason }` and closes the connection on version mismatch, an unrecognised session token, or a token that maps to a terminated session. If the token is structurally valid but maps to a session already past GAME_OVER, the server sends `S2CHandshakeRejected { reason: HandshakeRejectedReason::SessionExpired }` — the client must clear its persisted `session_token` (`localStorage` key `'lanes_session_token'`) on receiving this reason. WASM clients MUST persist `session_token` in `localStorage` under key `'lanes_session_token'`; clear on `S2CGameOver` receipt or on `S2CHandshake` for a new `session_id`. No other C2S messages are processed until the handshake completes. If `C2SHello` is not received within `hello_timeout_ms` (default: 5000ms), the server closes the connection. `HANDSHAKING` is a protocol-layer state that precedes any RSM phase — the RSM does not observe it.

**Rule 4 — Phase-gated message acceptance.** The server silently discards any C2S message not valid in the current RSM phase (see Valid C2S Messages table). Discards are logged server-side for diagnostics. No error message is sent to the client — this avoids timing attacks based on error response latency.

**Rule 5 — Submission atomicity.** A C2S submission (placement, bid, purchase) is not acknowledged until the server has written it to authoritative state. The client must not advance its local view until the corresponding S2C acknowledgement or S2C broadcast confirms the action.

**Rule 6 — Replication scope.** Lightyear component replication delivers shared public board state to all players. Private per-player state (hand, gold, mana, pool) is unicast via targeted reliable S2C messages and never broadcast. The server sends one `S2CGameSnapshot` per connecting player, with opponent secret fields stripped — a single broadcast snapshot is forbidden.

**Rule 7 — Reconnect snapshot.** On any client connection (initial join or reconnect), before any other messages, the server sends `S2CGameSnapshot` containing complete authoritative game state. The client discards locally buffered messages received before snapshot processing completes and rebuilds its view from scratch.

**Rule 8 — Disconnect detection.** Disconnect detection uses Lightyear's built-in `OnDisconnected` / `OnConnected` events **and** a mandatory application-layer `C2SHeartbeat`. WebSocket connections from WASM browsers can enter half-open TCP states (tab backgrounded, OS sleep, mobile radio switch) where `OnDisconnected` does not fire for 2–7 minutes. The client sends `C2SHeartbeat {}` every ~5 seconds on the unreliable channel (see Rule 2); the server resets `disconnect_trackers[player]` on each heartbeat received. The RSM owns the `disconnect_grace_seconds = 30s` threshold and the GAME_OVER transition. The protocol broadcasts `S2COpponentDisconnected { grace_remaining_ms }` on `OnDisconnected` so the remaining player sees the countdown.

---

### C2S Messages

| Message | Payload | Valid Phase(s) | Notes |
|---|---|---|---|
| `C2SHello` | `{ protocol_version: u32, session_token: Option<SessionToken> }` | HANDSHAKING | Client sends first. `None` = fresh connect. `Some(token)` = reconnect; server uses token to map the new Lightyear `ClientId` to the existing session slot. |
| `C2SPurchaseCard` | `{ card_id: CardId }` | DRAFT_INITIAL, DRAFT_SHOP | Server validates: gold ≥ cost, hand < 10, card in shop |
| `C2SRefreshShop` | `{}` | DRAFT_SHOP | Cost validated server-side via refresh_cost formula (1g first refresh, 2g all subsequent per refresh_cap); not valid in DRAFT_INITIAL |
| `C2SActivateCard` | `{ card_id: CardId }` | DRAFT_INITIAL, DRAFT_SHOP | Play instant-effect card from hand during DRAFT (reserve spells, Gelure, etc.); no board target |
| `C2SSignalReady` | `{ retract: bool }` | DRAFT_INITIAL, DRAFT_SHOP | `false` = signal ready; `true` = retract ready signal |
| `C2SPlaceBid` | `{ amount: u32 }` | DRAFT_AUCTION | Server validates: `amount ≥ last_accepted_bid + 1`, `player.gold ≥ amount` |
| `C2SSubmitPlacement` | `{ placements: Vec<PlacedCardSubmit> }` | PLACEMENT | Batch submit; marks player as submitted in RSM; empty list = zero new cards this round |
| `C2SAcknowledgeResult` | `{}` | GAME_OVER | Client confirms result rendered; server may clean up session |
| `C2SHeartbeat` | `{}` | All phases | **Channel: Unreliable** (see Rule 2). Sent every `heartbeat_interval_ms` (default 5000ms); server resets disconnect timer on receipt. Mandatory for WASM/WebSocket — `OnDisconnected` is unreliable in browser half-open TCP states. **Note:** WASM browsers stop `requestAnimationFrame` when the tab is backgrounded — the heartbeat cannot fire while the tab is invisible. Players who background the tab for >`disconnect_grace_seconds` will forfeit. Surface a "return to tab" warning on tab-restore via the Page Visibility API. |
| `C2SRequestSnapshot` | `{}` | All IN_GAME phases | Client-initiated desync recovery. Server responds with `S2CGameSnapshot` unicast (same path as reconnect). Rate-limited: server ignores if a snapshot was sent to this client within the last `snapshot_cooldown_ms` (default 5000ms). This is a recovery tool — clients must not poll for snapshots. Resolves OQ-BR-06 (cross-reference: `board-rendering.md`). |

**LOBBY phase messages** — defined in `game-session-system.md`; listed here for phase-gating (Rule 4):

| Message | Payload | Valid Phase(s) | Notes |
|---|---|---|---|
| `C2SCreateRoom` | `{ mode: GameMode }` | LOBBY (pre-session) | Processed before session exists; creates a new session |
| `C2SJoinRoom` | `{ room_code: String, requested_slot: u8 }` | LOBBY | Joins an existing session by room code and slot index |
| `C2SSelectClass` | `{ class_id: ClassId }` | LOBBY | Preview selection — reversible; not broadcast to others |
| `C2SConfirmClass` | `{ class_id: ClassId }` | LOBBY | Irrevocable class lock — triggers reveal when all locked |

The LOBBY phase is a protocol-layer state entered after `IN_GAME` handshake completes and before RSM transitions to `DRAFT_INITIAL`. All LOBBY C2S messages are phase-gated: messages arriving after LOBBY ends are silently discarded per Rule 4. Semantics owned by `game-session-system.md`.

```
struct PlacedCardSubmit {  // C2S payload — used in C2SSubmitPlacement.placements
    card_id:        CardId,
    target:         PlayTarget,
    reserve_amount: u32,  // mana drawn from reserve pool; card.cost - reserve_amount drawn from current-round mana
                          // 0 = pay all from current; card.cost = pay all from reserve; n = split
}
// NOTE: The S2C counterpart in S2CPlacementReveal.placements is PlacedCardReveal (see D.3).
// They share the same name root but have different fields — keep in distinct modules (c2s:: / s2c::).

enum PlayTarget {
    BoardCell  { lane: u8, cell: u8 },            // Minion (cell ≤ spawn_range), Trap, Structure
    TargetUnit { lane: u8, unit_id: EntityId },   // Targeted spell — target must be on board from prior rounds
    TargetObj  { player_id: PlayerId, lane: u8 }, // Objective-targeting spell (prism Lane 1/5, direct spells)
    LaneWide   { lane: u8 },                      // Field — lane-wide effect
    Instant,                                      // Untargeted spell, Order
}
```

**Placement model:** `C2SSubmitPlacement` is a single batch — the player finalizes their entire selection in one message. Once sent, the submission cannot be retracted. Client UI manages local selection state; the server only sees the final batch.

**C2SActivateCard acknowledgement model:** There is no dedicated `S2CCardActivated` acknowledgement. Per Rule 5, the client must not advance its local view until S2C confirms the action. For `C2SActivateCard`, confirmation arrives through the existing economy/hand messages: a successful instant card play produces `S2CGoldUpdate` as side effects (reserve_mana changes for reserve spells like `prism_reserve`; mana changes for mana-cost instants). If `S2CGoldUpdate` does not arrive within `activate_timeout_ms` (client-side constant, default 3000ms), the client treats the activation as rejected and reverts local optimistic state. **Design decision:** Cards with no observable side effects (e.g., a no-cost, no-draw Order) MUST produce a `S2CGoldUpdate` with unchanged values as a no-op confirmation signal. This is an intentional architectural choice — the alternative (a dedicated `S2CCardActivated` message) was evaluated and rejected to keep the message surface small. Implementers must ensure every server-side instant-card handler sends `S2CGoldUpdate` even when values do not change.

**`C2SSubmitPlacement` server validation:** The server validates the full batch before accepting: `sum(placements[i].reserve_amount) ≤ player.reserve_mana` AND `sum(card[i].cost - placements[i].reserve_amount) ≤ player.current_mana`. If any card is not in the player's hand, or any `lane`/`cell` value is out of range (1–5 / 1–8), the entire batch is silently discarded (Rule 4). **If any `card_id` appears more than once in the batch, the entire batch is silently discarded** — validation runs against the original hand snapshot before applying any changes; a duplicated card_id passes the "in hand" check on first pass but must be caught by explicit dedup check. Partial acceptance is not supported.

**Silence = pass at auction:** No `C2SPassBid` message. If a player does not bid, the timer runs to zero and the last bidder wins. **Design note:** During DRAFT_AUCTION, a silent opponent is protocol-indistinguishable from a half-open connection — heartbeats stop firing on backgrounded WASM tabs, and the 30-second grace window exceeds the 20-second auction timer. This ambiguity is accepted at friend-game scope. See `shop-auction-ui.md` for the client rendering contract (no presence indicator).

**Targeted spell provisional assumption:** `PlayTarget::TargetUnit` can only target units already on the board from prior rounds — not units in the current PLACEMENT buffer (unrevealed). If the Keyword System GDD requires targeting newly placed units, `PlayTarget` must be extended.

---

### S2C Messages

| Message | Channel | Scope | Payload |
|---|---|---|---|
| `S2CHandshake` | Reliable | Unicast | `{ protocol_version: u32, session_id: SessionId, session_token: SessionToken }` — WASM clients MUST persist `session_token` in `localStorage` under key `'lanes_session_token'`; clear on `S2CGameOver` or on a new `S2CHandshake` for a different `session_id`. Session token MUST be generated from OS entropy (`rand::thread_rng()`), NOT the seeded gameplay RNG. |
| `S2CHandshakeRejected` | Reliable | Unicast | `{ reason: HandshakeRejectedReason, server_version: u32, client_version: u32 }` — see `HandshakeRejectedReason` enum in D.3. `server_version`/`client_version` populated for `VersionMismatch` only; 0 for other reasons. |
| `S2CGameSnapshot` | Reliable | Unicast | Full game state — see Section D |
| `S2CPhaseChanged` | Reliable | Broadcast | `{ phase: RoundPhase, round_number: u32, timer_duration_ms: u32 }` |
| `S2CGameOver` | Reliable | Broadcast | `{ loser: Option<PlayerId>, round: u32, reason: GameOverReason }` — `loser=None` when no single player lost: Draw (mutual destruction or mutual disconnection) or ResolutionTimeout (60s safety timeout). See `GameOverReason` enum in round-state-machine.md Rule 14 (4 variants). |
| `S2CGoldUpdate` | Reliable | Unicast | `{ gold: u32, current_mana: u32, reserve_mana: u32, mana_cap: u8 }` — `reserved_gold` is NOT included; use `S2CGoldBroadcast` for `reserved_gold`. (`S2CGoldBroadcast` is already the authoritative source for auction-panel affordability logic and HUD bidding-headroom — having `reserved_gold` in both messages creates a dual-source conflict.) |
| `S2CGoldBroadcast` | Reliable | Broadcast | `{ player_id: PlayerId, gold: u32, reserved_gold: u32 }` — opponent gold always visible; `gold - reserved_gold` = free gold (required for auction panel to display correct bidding headroom). **RULE: fires on any mutation of `gold` OR `reserved_gold` — including bid acceptance (which changes `reserved_gold` but not `gold`). Without this, the auction panel's free-gold display is stale after each bid. Fires ONCE per bid-acceptance event, after all reservation mutations for that bid complete (release previous leader's reservation + reserve new leader's gold are treated as a single atomic change). RESOLUTION gold changes flow through `S2CResolutionEvent::GoldAwarded` — a post-batch `S2CGoldBroadcast` fires after `S2CResolutionEvent` delivery and before `S2CPhaseChanged(DRAFT_SHOP)` to sync totals.** |
| `S2CCardAcquired` | Reliable | Unicast | `{ card_id: CardId, source: AcquisitionSource }` |
| `S2COpponentSubmitted` | Reliable | Unicast (non-submitting player) | `{ player_id: PlayerId }` — sent when one player submits their placement; the other player receives this immediately so they can show "waiting for opponent" state. NOT sent to the player who submitted. |
| `S2CShopSlots` | Reliable | Unicast | `{ slots: Vec<Option<CardId>> }` — `None` = empty slot (dedup exhaustion or pool exhaustion for that slot type) |
| `S2CDraftOffering` | Reliable | Unicast | `{ card_ids: Vec<CardId> }` — exactly 9 cards at DRAFT_INITIAL (fewer only in stripped test fixtures) |
| `S2CPoolUpdate` | Reliable | Unicast | `{ updates: Vec<(CardId, u8)> }` — delta `copies_remaining` |
| `S2CPlacementReveal` | Reliable | Broadcast | `{ placements: Vec<PlacedCardReveal> }` — atomic simultaneous reveal; both players receive this as the sole signal that placement is closed. Client MUST render from this payload, not from pre-arrived component replication. **Pre-arrived unreliable `BoardPosition` replication that contradicts placement positions MUST be discarded — the reveal payload is authoritative for all placement-related board positions.** Cross-reference: `board-rendering.md` Rule 7 (collect-then-reveal buffer). |
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
| `S2CObjectiveIdentities` | Reliable | Unicast (owner) | `{ identities: Vec<(LaneId, bool)> }` — `(lane_id, is_fake)`. Owned by ADR-001. Dispatched once at DRAFT_INITIAL to each player with their own objective identity assignments; **MUST be re-sent on reconnect** as part of the session resume sequence (after `S2CGameSnapshot`, before any actionable phase). Reliable delivery guarantees in-session arrival but does not auto-replay across reconnects — the server explicitly re-dispatches. Payload is tiny (~6 bytes per player). Cross-references: `docs/architecture/adr-001-objective-identity-unicast.md` (source); `design/gdd/board-rendering.md` Rule 11 + ObjectiveIdentityCache (consumer); `design/ux/hud.md` (own-objective dots). |

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

**RESOLUTION rendering contract for unreliable components:** While `S2CResolutionEvent` animation is draining (`isResolutionAnimating = true` in client state):
1. **`ObjectiveHp` replication is suppressed** — objective HP bars are driven exclusively by `ObjectiveDamaged.hp_after` at AnimQueue playback time. Applying replication simultaneously causes HP bar flicker (jump to final value, then decrement during animation).
2. **`BoardPosition` replication is suppressed** — unit positions are driven exclusively by `UnitMoved` and `DisplacementEvent` events in the batch. Applying replication simultaneously causes units to teleport before their movement animations play.
3. **`UnitStats` replication is advisory** — apply only after the unit's sub-step events have been consumed by the AnimQueue, not at batch receipt. These rules are client-side contracts but are specified here because they arise from the dual-delivery architecture.

**`TrapIdentity` visibility:** Trap card identity must be hidden from the opponent. Proposed component split: `TrapPresence { has_trap: bool }` (broadcast) + `TrapIdentity { card_id: CardId }` (owner-only). See Open Question 2 for Lightyear 0.26 verification requirement.

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
| `HANDSHAKING` | Transport connected; `C2SHello` version check pending. No game state visible. **Client renders a "reconnecting…" loading indicator during this window.** WASM cold-start can consume 2–3 seconds before `C2SHello` fires; `hello_timeout_ms` safe minimum for WASM is ~4000ms (not the 2000ms listed in the safe range — 2000ms risks kicking slow cold-start clients). | Transport connect | `IN_GAME` on `S2CHandshake`; connection close on version mismatch or `hello_timeout_ms` |
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
| **Auction System** | Bid accepted events, auction settled event, auction card selection | `S2CAuctionCard`, `S2CAuctionBidAccepted`, `S2CAuctionSettled` (reliable broadcast). Gold reservation changes on bid acceptance trigger `S2CGoldBroadcast` (reserved_gold rule). |
| **Combat Resolution** | Sub-step events, kill/objective gold-award events (embedded as `GoldAwarded` entries in batch) | `S2CResolutionEvent` (reliable broadcast). Gold awards embedded in batch — **no standalone `S2CGoldUpdate` during RESOLUTION**. After batch delivery, server sends `S2CGoldBroadcast` to sync totals. `S2CGoldUpdate` fires for all non-RESOLUTION gold events only. |
| **Keyword System** | Keyword trigger events, displacement events, keyword-effect card grants | `S2CResolutionEvent` variants: `KeywordTriggered`, `DisplacementEvent`, `AppearanceFired`, `DeathTriggerFired`, `FinalBlowFired`, `EndOfTurnFired`. Keyword-granted cards: `S2CCardAcquired { source: AcquisitionSource::KeywordEffect }`. |
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
    class_id:          ClassId,      // PUBLIC — required for reconnect; S2CClassesRevealed is NOT re-sent on reconnect; without this field the reconnecting client cannot render class tokens, class-specific board state, or keyword interactions
    gold:              u32,
    reserved_gold:     u32,          // PUBLIC — gold committed to current auction bid (free gold = gold - reserved_gold). Required for HUD bidding-headroom display on reconnect.
    current_mana:      u32,
    reserve_mana:      u32,
    spawn_range_cells: u8,           // cells available for Minion placement: 1–3
    mana_cap:          u8,           // current mana cap (default 10; can increase to 11–12 via fake reward). SERVER INVARIANT: must not exceed 12.
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
    seeds:      Vec<SeedBoardState>,    // Sadida Seed cell-markers; absent = reconnecting client sees no Seeds, board state desyncs
    sinistros:  Vec<SinistroState>,     // Xelor Sinistro attachments on objectives; absent = reconnecting client misses ongoing damage per RESOLUTION
}

SeedBoardState {
    owner: PlayerId,  // Sadida player who placed this seed; required for ownership-based rendering (consistent with TrapBoardState/SinistroState)
    lane: u8,
    cell: u8,
    // Seed persists indefinitely until consumed by Graines de Folie or destroyed. Max 1 per cell.
}

SinistroState {
    owner:  PlayerId,  // the Xelor player who played Sinistro
    lane:   u8,        // objective lane this Sinistro is attached to
    damage: u8,        // damage dealt per RESOLUTION tick (default 1). See class-system.md Xelor Sinistro entity. Destroyed if its parent objective takes damage.
}

PrismBoardState {
    player_id: PlayerId,  // identifies which player's prism; keyed on (player_id, lane)
    lane:      u8,
    collected: bool,      // true = prism collected; false = still available
}
// BoardSnapshot.prisms = Vec<PrismBoardState>: one entry per (player_id, lane) pair.
// 1v1: 10 entries (2 players × 5 lanes). 2v2: 20 entries (4 players × 5 lanes).
// player_id field is required — without it the snapshot cannot reconstruct per-player prism state.

UnitBoardState {
    unit_id:    EntityId,   // server-assigned NetworkId.id — use to look up local ECS Entity
    card_id:    CardId,
    owner:      PlayerId,
    lane:       u8,
    cell:       u8,
    max_hp:     u8,         // base HP + any permanent HP bonuses; required for INJURED state derivation (INJURED = current_hp < max_hp)
    current_hp: u8,
    atk:        u8,
    ar:             u8,  // current AR including permanent Seed bonuses. SERVER INVARIANT: must not overflow u8 — enforce AR cap in server logic (document cap value in Edge Cases when determined)
    resistance_x:   u8,  // RESISTANCE X keyword value; 0 = keyword not present. Required for reconnect damage-preview and keyword-badge display.
    vulnerability_x: u8, // VULNERABILITY X keyword value; 0 = keyword not present. Required for reconnect damage-preview.

    // Keyword state — required by Keyword System Replication Contract for correct reconnect rendering
    // NOTE: Keyword System Replication Contract uses `silenced_until_round: Option<u8>` — this is INCORRECT.
    // The authoritative type is Option<u32> here (matches round_number). Keyword GDD must be updated to match.
    shield_active:           bool,              // SHIELD hex glyph; false after consumption
    stun_active:             bool,              // STUN stars glyph; false if not stunned this round
    silenced_until_round:    Option<u32>,       // SILENCE outline; Some(R) = silenced until round R; u32 matches round_number type
    leader_bonus_atk:        u8,               // ATK bonus from LEADER snapshot (0 if no allied LEADER)
    leader_bonus_hp:         u8,               // HP bonus from LEADER snapshot (0 if no allied LEADER)
    bodyguard_protects:      Option<EntityId>, // BODYGUARD bond target; None = no active bond. Survives CHANGE LANE — cannot be derived from position alone.
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
    max_hp:       u8,         // required for HP bar fractional fill rendering
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
    starting_price:      u32,           // auction floor for this card's rarity (3g/4g/5g for Rare/Epic/Legendary)
    last_accepted_bid:   u32,           // last bid amount accepted; minimum valid next bid = last_accepted_bid + 1.
                                        // At zero bids: last_accepted_bid = 0. Use starting_price (not last_accepted_bid + 1) as
                                        // the display floor — client MUST check: if last_accepted_bid == 0, minimum bid = starting_price.
    current_leader:      Option<PlayerId>,
    timer_remaining_ms:  u32,           // milliseconds remaining on the auction timer. NOTE: stale by network RTT by the time client renders it — this is a known accepted limitation for friend-game scope.
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
    sub_step: u8,       // 1–7: sub-steps 1–6 = in-turn resolution; 7 = post-SS6 END OF TURN bucket (EndOfTurnFired events).
                        // SERVER CONTRACT: MUST NOT emit sub_step outside 1–7.
                        // CLIENT CONTRACT: treat 8+ as fatal desync — send C2SRequestSnapshot immediately.
    event:    ResolutionEvent,
}

enum ResolutionEvent {
    // ── Sub-step delimiters ──────────────────────────────────────────────────
    SubStepMarker {
        sub_step: u8,   // 1–6 at sub-step start; 7 = post-SS6 (END OF TURN triggers).
                        // REQUIRED even for empty sub-steps: AnimQueue uses this to insert mandatory inter-step pauses.
    },

    // ── Movement ────────────────────────────────────────────────────────────
    UnitMoved {
        unit_id:   EntityId,
        from_lane: u8,
        to_lane:   u8,   // same as from_lane for in-lane movement; differs for CHANGE LANE keyword
        from_cell: u8,
        to_cell:   u8,
    },

    // ── Forced displacement (distinct from voluntary movement) ───────────────
    DisplacementEvent {
        unit_id:      EntityId,
        attacker_id:  Option<EntityId>,  // None = spell effect with no board unit as caster
        from_lane:    u8,
        from_cell:    u8,
        to_lane:      u8,
        to_cell:      u8,               // actual final position after IRREMOVABLE block or board-edge clamp
        kind:         DisplacementKind,
        block_reason: Option<DisplacementBlockReason>,  // None = displacement completed normally
        sub_step:     u8,
        // INVARIANT: if block_reason = Some(IrremovableKeyword), to_cell == from_cell.
        // If block_reason = Some(BoardEdgeClamped), to_cell is 1 or 8 (clamped, not original destination).
        // A TrapTriggered event for the same unit may immediately follow if the unit lands on a trap cell mid-displacement.
    },

    // ── Combat damage ────────────────────────────────────────────────────────
    CombatDamage {
        attacker_id:        EntityId,
        target_id:          EntityId,
        lane:               u8,
        damage:             u8,         // 0 = attack blocked by SHIELD
        was_lethal:         bool,       // true = a UnitDied event for target_id follows immediately within this sub_step
        is_counterattack:   bool,       // true = this damage is the return-strike (COUNTERATTACK keyword)
        damage_source_kind: DamageKind,
        sub_step:           u8,
        // ORDERING INVARIANT: if was_lethal = true, the next event in the array for the same sub_step MUST be
        // UnitDied { unit_id: target_id }. Client can rely on this for animation sequencing.
    },

    // ── Deaths ───────────────────────────────────────────────────────────────
    UnitDied {
        unit_id:   EntityId,
        lane:      u8,
        cell:      u8,
        killer_id: Option<EntityId>,  // None = damage-over-time, structure effect, or spell
    },
    DeathTriggerFired {
        unit_id:  EntityId,   // the unit whose DEATH keyword is triggering
        sub_step: u8,
        // Fired immediately before the DEATH keyword's secondary effects. Drives Arcane Gold pulse animation.
        // ORDERING: SubStep → … → UnitDied → DeathTriggerFired → [secondary effects] within same sub_step.
    },

    // ── Keyword activations ──────────────────────────────────────────────────
    KeywordTriggered {
        source_unit_id: Option<EntityId>,  // None for board-global events (e.g., OutnumberedFlipped)
        sub_step:       u8,
        payload:        KeywordPayload,
    },
    AppearanceFired {
        unit_id:  EntityId,
        sub_step: u8,         // Drives Arcane Gold aura pulse when unit first acts on the board
    },
    FinalBlowFired {
        killer_id: EntityId,
        victim_id: EntityId,
        sub_step:  u8,        // Fired in the sub-step where the kill occurs, BEFORE the corresponding UnitDied
    },
    EndOfTurnFired {
        unit_id:  EntityId,   // sub_step 7 (post-SS6 sentinel)
    },

    // ── Economy events (embedded in batch) ───────────────────────────────────
    GoldAwarded {
        player_id: PlayerId,
        amount:    u32,
        reason:    GoldAwardReason,
        // NOTE: RESOLUTION gold changes flow through GoldAwarded entries here, NOT through standalone S2CGoldBroadcast.
        // After S2CResolutionEvent is fully delivered (before S2CPhaseChanged), the server sends S2CGoldBroadcast
        // to sync both players' gold totals to their authoritative post-RESOLUTION values.
    },
    ManaCapIncreased {
        player_id: PlayerId,
        new_cap:   u8,        // SERVER INVARIANT: must not exceed 12
    },
    SpawnRangeChanged {
        player_id:            PlayerId,
        new_spawn_range_cells: u8,   // 1–3; increases when attacker destroys a fake objective
        // This event is the ONLY delivery mechanism for spawn range changes for the connected player.
        // Reconnecting clients receive spawn_range_cells in PlayerSnapshot instead.
    },

    // ── Traps ────────────────────────────────────────────────────────────────
    TrapTriggered {
        trap_id:            EntityId,
        triggering_unit_id: EntityId,
        lane:               u8,
        cell:               u8,
    },
    TrapDestroyed {
        trap_id:     EntityId,
        lane:        u8,
        cell:        u8,
        card_id:     Option<CardId>,   // Some = own trap (identity known to owner); None = opponent's trap
        attacker_id: Option<EntityId>, // None = destroyed by non-combat effect
    },

    // ── Structures ───────────────────────────────────────────────────────────
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

    // ── Objectives ───────────────────────────────────────────────────────────
    ObjectiveDamaged {
        target_player_id: PlayerId,
        lane:             u8,
        hp_before:        u8,
        hp_after:         u8,
        attacker_id:      Option<EntityId>,
        // RENDERING CONTRACT: client MUST apply hp_after to the HP bar at AnimQueue playback time,
        // not at batch receipt. HP bars decrement as the event is played back, not all at once.
    },
    ObjectiveDestroyed {
        target_player_id: PlayerId,
        lane:             u8,
        was_fake:         bool,   // both players learn real/fake status simultaneously at destruction
    },

    // ── Prisms ───────────────────────────────────────────────────────────────
    PrismCollected {
        player_id: PlayerId,
        lane:      u8,
    },
}

enum GoldAwardReason {
    KillReward,       // +1g per kill
    ObjectiveReward,  // +3g per objective destroyed (real or fake)
    // NOTE: PrismReward variant deliberately omitted. Prism System Rule 11 and BLOCKING AC PS-15
    // confirm that prisms grant zero gold. This variant would be dead code — removed to prevent
    // implementer confusion. See prism-system.md Rule 11.
}

enum DamageKind {
    Melee,
    Range,            // RANGE keyword — projectile impact VFX
    FirstStrike,      // FIRST STRIKE — fires before standard melee
    Spell,            // Spell card damage (prism_strike, etc.)
    StructureEffect,  // Damage from a Structure's passive ability
    SeedTrap,         // Sadida Seed damage on enemy walk-over
}

enum DisplacementKind {
    Repel(u8),        // REPEL X — push target X cells away from caster
    Attract(u8),      // ATTRACT X — pull target X cells toward caster
    Teleport { dest_lane: u8, dest_cell: u8 },   // TELEPORT — direct repositioning; cross-lane allowed if card specifies
}

enum DisplacementBlockReason {
    IrremovableKeyword,   // target has IRREMOVABLE; to_cell == from_cell
    BoardEdgeClamped,     // displacement clamped at board edge (cell 1 or 8); to_cell is the edge cell
}

// KeywordPayload — per-keyword trigger payloads for KeywordTriggered events.
// Defined in full by keyword-system.md Replication Contract (OQ-NP5).
// Implementer MUST cross-reference keyword-system.md for the authoritative payload per keyword.
enum KeywordPayload {
    ShieldConsumed,
    StunApplied { duration_rounds: u8 },
    SilenceApplied { duration_rounds: u8 },
    InjuredBonusActive { granted_keyword: GrantedKeyword },  // typed subset enum (see D.3) — avoids recursive KeywordPayload type without raw discriminants
    LeaderSnapshotTaken { leader_unit_id: EntityId },
    OutnumberedFlipped { player_id: PlayerId, active: bool },  // board-global — source_unit_id is None
    BodyguardBondCreated { bodyguard_id: EntityId, protected_id: EntityId },
    BodyguardBondBroken { bodyguard_id: EntityId },
    CounterattackFired,   // COUNTERATTACK reactive strike — paired with a CombatDamage(is_counterattack: true)
    HasteActivated,       // HASTE — unit moves at SS2 in addition to SS5
}
```

---

### D.3 — Supporting Enum Definitions

```
// Renamed from CardSource to AcquisitionSource to match entities.yaml registry.
// All consumers (hand-ui, auction-system, prism-system, objective-system) must use this name.
enum AcquisitionSource {
    ShopPurchase,
    DraftInitial,     // 9-card pick at DRAFT_INITIAL (clarified from previous DraftSelection)
    AuctionWon,
    FreeCardPick,     // random draw from pool as fake-objective destruction reward (objective-system.md Rule 10)
    PrismLane1,       // "1 damage to a chosen objective" spell added to hand
    PrismLane2,       // "+1 reserve mana" spell added to hand
    PrismLane3,       // random draw from shop pool
    PrismLane4,       // "+1 reserve mana" spell added to hand (same reward as Lane 2)
    PrismLane5,       // "1 damage to a chosen objective" spell added to hand
    KeywordEffect,    // card added to hand by a keyword trigger (DEATH, FINAL BLOW, class ability)
}

struct PlacedCardReveal {  // element of S2CPlacementReveal.placements — S2C only; distinct from PlacedCardSubmit (C2S)
    card_id:  CardId,
    owner_id: PlayerId,
    target:   PlayTarget,
    // reserve_amount omitted: mana accounting is server-side; clients never see the split
}

// Wire identity types
type SessionToken = [u8; 16];   // UUID v4 (128-bit), server-generated from OS entropy; MUST NOT use gameplay RNG
type EntityId = u64;            // server-assigned monotonic counter per session; mapped to local ECS Entity via HashMap<EntityId, Entity> on each client

enum BidRejectedReason {
    InsufficientGold,   // player.gold - reserved_gold < amount
    AmountTooLow,       // amount <= current bid (must be >= last_accepted_bid + 1)
    AuctionExpired,     // server timer fired between client send and server receive — S2CAuctionSettled follows immediately
    AlreadyLeader,      // bidder == current_leader; self-bids are rejected (auction-system.md Rule 4)
    HandFull,           // bidder.hand_size == 10; must play a card before bidding (auction-system.md Rule 4)
}

enum HandshakeRejectedReason {
    VersionMismatch,    // protocol_version mismatch — server_version and client_version fields populated
    UnrecognisedToken,  // session_token not found in active sessions
    SessionExpired,     // session_token recognised but session already past GAME_OVER — client must clear localStorage token
}

enum GameOverReason {
    ObjectivesDestroyed,  // at least one player's real objectives destroyed; loser = Some(player) or None if simultaneous
    Disconnection,        // player's disconnect_grace_seconds expired; loser = Some(player)
    Draw,                 // mutual simultaneous objective destruction OR both players disconnected simultaneously
    ResolutionTimeout,    // RESOLUTION 60s safety timeout (RSM Rule 10); loser = None. Distinct from Draw — client shows "round timed out" not "draw"
}

// Subset of keywords that INJURED can grant — avoids recursive KeywordPayload type.
// Implementer MUST cross-reference keyword-system.md for the authoritative INJURED grant list.
// This list is part of the wire protocol — extend only with explicit Keyword GDD approval.
enum GrantedKeyword {
    FirstStrike,
    Counterattack,
    Range,
    Shield,
    // Add here if keyword-system.md adds new INJURED-grantable keywords
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

- **If `S2CDraftOffering` and `S2CPhaseChanged(DRAFT_INITIAL)` are enqueued in the same frame**: `S2CDraftOffering` MUST be enqueued before `S2CPhaseChanged(DRAFT_INITIAL)`. The client must not render the DRAFT_INITIAL shop panel until the 9-card offering is available. Analogous to the `S2CAuctionCard` invariant. An empty shop on the first round of the game is a first-impression failure.

- **If `S2CResolutionEvent` has been delivered and the server is about to send `S2CPhaseChanged(DRAFT_SHOP)`**: The server MUST enqueue `S2CGoldBroadcast` (for both players, if their gold changed) AFTER `S2CResolutionEvent` and BEFORE `S2CPhaseChanged(DRAFT_SHOP)`. RESOLUTION-origin gold changes (kill rewards, objective rewards) travel inside `S2CResolutionEvent` as `GoldAwarded` entries. The post-batch `S2CGoldBroadcast` syncs authoritative totals to both players before the new DRAFT phase begins.

- **If `C2SPlaceBid` arrives at the server after `S2CAuctionSettled` has already been dispatched (race on the last bid)**: The server does not send `S2CAuctionBidRejected` in this case — `S2CAuctionSettled` is the terminal signal and takes precedence. The server silently discards the late bid after settlement is final. The client should display the bid as "pending" until either `S2CAuctionBidAccepted` or `S2CAuctionSettled` resolves it.

- **Bid-pending client state spec:** After `C2SPlaceBid` is sent and before `S2CAuctionBidAccepted` or `S2CAuctionSettled` arrives, the client MUST render a "bid pending" state on the auction panel — the bid amount is shown as in-flight, bid controls are disabled, and neither a "you won" nor "you lost" result is displayed. If the auction timer display reaches zero locally before `S2CAuctionSettled` arrives, the client renders "settling…" and continues waiting — no local result is finalized. This is the client-side spec for the highest-tension moment of the mechanic. See `shop-auction-ui.md` for the visual rendering contract. **Maximum observable latency window:** `2 × RTT_p99` (approximately 200ms for typical WebSocket connections) — this is the upper bound between bid send and confirmation; the UI should not show an error or timeout within this window.

- **If the client reconnects during DRAFT_AUCTION and the auction timer is nearly expired:** `AuctionSnapshot.timer_remaining_ms` is stale by network RTT at the moment the client renders it. The reconnecting client's timer model is always more optimistic than reality (they think they have more time). A bid placed on a stale timer that the server has already expired produces `S2CAuctionBidRejected { reason: BidRejectedReason::AuctionExpired }` — this is correct server behavior. The client renders `AuctionExpired` as equivalent to the "settling…" bid-pending state above (not as an error), since the player's intent was valid at their local time.

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
| `hello_timeout_ms` | 5000 | **4000**–15000 | Legitimate slow-starting clients (WASM cold start) kicked before sending `C2SHello`. **Note: WASM cold-start consumes 2–3s; safe minimum for WASM is ~4000ms, not 2000ms.** | Slow detection of port-scanning / connection-flooding | `disconnect_grace_seconds` — both are connection safety nets on different layers |
| `snapshot_cooldown_ms` | 5000 | 2000–10000 | Clients can flood the server with `C2SRequestSnapshot` requests | Legitimate desync recovery is too slow | `S2CGameSnapshot` — rate-limits client-initiated snapshot requests |
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
| NP-4 | **GIVEN** a client is IN_GAME and sends `C2SPurchaseCard` during PLACEMENT phase, **WHEN** the server processes it, **THEN** no S2C message is sent to either player and the player's gold, hand, and shop_slots are all unchanged (full discard invariant, not just gold). | BLOCKING |
| NP-5 | **GIVEN** a client sends `C2SSubmitPlacement` during DRAFT_SHOP phase AND `player.submitted` was `false` before the message arrived, **WHEN** the server processes it, **THEN** `player.submitted` remains `false` (unchanged) and no S2C message of any kind is sent in response. | BLOCKING |
| NP-6 | **GIVEN** a player sends valid `C2SSubmitPlacement` during PLACEMENT, **WHEN** the server accepts it, **THEN** `player.submitted = true` in server state. No S2C message is sent **to the submitting player** — the submission is recorded silently until `S2CPlacementReveal` fires. (Note: `S2COpponentSubmitted` IS sent to the non-submitting player — see NP-34. NP-6 only asserts the submitting player's receive stream is silent.) | BLOCKING |
| NP-7 | **GIVEN** units are on the board, **WHEN** the server updates `BoardPosition` or `UnitStats`, **THEN** no S2C* message of any type is received by a test client on the reliable channel for that tick — delivery occurs via Lightyear component replication only. (Integration test — requires a live Lightyear session with a test client observing the reliable channel message stream.) | BLOCKING |
| NP-8 | **GIVEN** RESOLUTION completes and the server sends phase-exit messages, **WHEN** messages are captured in the reliable channel stream, **THEN** `S2CResolutionEvent` precedes `S2CPhaseChanged(DRAFT_SHOP)` in the sequence. (Integration test — requires a live Lightyear session to inspect message ordering on the reliable channel.) | BLOCKING |
| NP-9 | **GIVEN** a client reconnects mid-game, **WHEN** the server processes the new transport connection, **THEN** the first S2C message sent is `S2CGameSnapshot` — no `S2CPhaseChanged`, `S2CGoldUpdate`, or other game message precedes it. (Integration test — requires a live Lightyear session to inspect channel message ordering.) | BLOCKING |
| NP-10 | **GIVEN** a client reconnects during PLACEMENT after having already submitted, **WHEN** `S2CGameSnapshot` is processed, **THEN** the reconnecting player's `PlayerSnapshot.submitted = true`. (Server-observable assertion; BLOCKING.) | BLOCKING |
| NP-10-UI | **GIVEN** NP-10 condition — `PlayerSnapshot.submitted = true` on reconnect, **THEN** the placement card-selection UI is not re-presented to the player. (UI rendering expectation — cannot be asserted headlessly; manual walkthrough evidence.) | ADVISORY |
| NP-11 | **GIVEN** both players are IN_GAME, **WHEN** the server sends `S2CGoldUpdate` for Player A, **THEN** the server does not enqueue `S2CGoldUpdate` for Player B's `ClientId`. Gold is unicast to the owning player only. (Integration test — requires a live Lightyear session to verify delivery scope per `ClientId`.) | BLOCKING |
| NP-12 | **GIVEN** a player's transport connection drops, **WHEN** Lightyear's `OnDisconnected` event fires, **THEN** `S2COpponentDisconnected { grace_remaining_ms }` is received by the remaining player before the next `S2CPhaseChanged` or any other phase-affecting message is delivered on the reliable channel. (Integration test — requires a live Lightyear session.) | BLOCKING |
| NP-13 | **GIVEN** a player has been disconnected for strictly more than `disconnect_grace_seconds`, **WHEN** the RSM evaluates disconnect trackers, **THEN** `S2CGameOver` is broadcast on the reliable channel. (Integration test — requires a live Lightyear session or a `disconnect_trackers` resource set to 0 in test setup to avoid real-time sleep.) | BLOCKING |
| NP-14 | **GIVEN** `C2SSubmitPlacement` is sent twice by the same player in the same PLACEMENT phase, **WHEN** the server processes both, **THEN** the second submission is silently discarded — `player.submitted` remains `true` and no S2C message is sent in response to the second message. | BLOCKING |
| NP-15 | **GIVEN** a client is IN_GAME and sends `C2SHello` again on the same connection, **WHEN** the server processes it, **THEN** no `S2CHandshake` is sent in response and the existing session is not disrupted. | BLOCKING |
| NP-16 | **GIVEN** a two-player game is in progress and Player B reconnects, **WHEN** the server produces Player B's `S2CGameSnapshot`, **THEN** the snapshot does NOT contain: Player A's `hand`, `shop_slots`, `pool_snapshot`, any `ObjectiveSnapshot.is_real = true` for Player A's objectives, or `TrapBoardState.card_id: Some(...)` for traps owned by Player A (opponent traps must have `card_id: None`). | BLOCKING |
| NP-17 | **GIVEN** a player disconnects and reconnects within `disconnect_grace_seconds` (simulated via `disconnect_trackers` resource injection with value > 0 to skip real-time countdown), **WHEN** the server processes the reconnect, **THEN** `S2COpponentReconnected` is broadcast to the remaining player. (Integration test — requires two live clients and a transport drop+reconnect sequence.) | BLOCKING |
| NP-18 | **GIVEN** the server is in DRAFT_AUCTION and a player reconnects, **WHEN** `S2CGameSnapshot` is received, **THEN** `auction_state` is non-null and contains: `card_id`, `last_accepted_bid`, `current_leader` (or `None` if no bids yet), and `timer_remaining_ms`. | BLOCKING |
| NP-19 | **GIVEN** Player A's gold changes due to a non-RESOLUTION event (shop purchase, phase income, auction settlement, bid reservation change), **WHEN** the server processes the change, **THEN** `S2CGoldBroadcast { player_id: Player_A, gold: new_amount, reserved_gold: Player_A.reserved_gold }` is delivered to ALL connected players including Player B. NOTE: RESOLUTION-origin gold changes (kill rewards, objective rewards) are carried inside `S2CResolutionEvent::GoldAwarded` entries — NOT via standalone `S2CGoldBroadcast` during RESOLUTION. A post-batch `S2CGoldBroadcast` fires after `S2CResolutionEvent` delivery and before `S2CPhaseChanged(DRAFT_SHOP)` to sync totals. | BLOCKING |
| NP-20 | **GIVEN** a game is in PLACEMENT with at least 2 units on the board, **WHEN** a player reconnects and receives `S2CGameSnapshot`, **THEN** the snapshot contains: `round_number > 0`, `phase = PLACEMENT`, all board units present in `BoardSnapshot.units`, and the reconnecting player's `hand` is populated if they hold cards. (Integration test.) | BLOCKING |
| NP-21 | **GIVEN** a client reconnects during PLACEMENT before having submitted, **WHEN** `S2CGameSnapshot` is processed, **THEN** `PlayerSnapshot.submitted = false` for the reconnecting player AND `snapshot.timer_remaining_ms` is in range `(0, placement_timer_ms)` (non-zero, not already expired). (Server-observable assertions; BLOCKING.) | BLOCKING |
| NP-21-UI | **GIVEN** NP-21 condition, **THEN** the placement card-selection UI is re-presented and the timer display counts down from `timer_remaining_ms`. (UI rendering expectation — ADVISORY; manual walkthrough evidence.) | ADVISORY |
| NP-22 | **GIVEN** a client reconnects during PLACEMENT and the opponent has already submitted, **WHEN** `S2CGameSnapshot` is processed, **THEN** the opponent's `PlayerSnapshot.submitted = true`. The snapshot is the sole authority — no further submission confirmation messages are sent. (Server-observable assertion; BLOCKING.) | BLOCKING |
| NP-22-UI | **GIVEN** NP-22 condition — opponent `submitted = true` on reconnect, **THEN** the client renders "waiting for opponent" state for that player. (UI rendering expectation — ADVISORY.) | ADVISORY |
| NP-23 | **GIVEN** Player A acquires a card (from shop, auction, prism, or keyword effect), **WHEN** the server sends `S2CCardAcquired`, **THEN** it is NOT delivered to Player B's transport connection. Card acquisition is private to the owning player. (Integration test — requires two live clients to verify non-delivery to B.) | BLOCKING |
| NP-24 | **GIVEN** the server receives `C2SHeartbeat` from Player A, **WHEN** the heartbeat is processed, **THEN** `disconnect_trackers[Player_A]` is reset to `disconnect_grace_seconds`. (Unit-testable if `C2SHeartbeat` dispatch is isolated; integration test if live Lightyear session is required for message delivery.) | BLOCKING |
| NP-25 | **GIVEN** `disconnect_trackers[Player_A]` has been decremented to 0 (heartbeat not received; simulated via `Resource` injection in test setup to avoid real-time sleep), **WHEN** the RSM evaluates disconnect trackers, **THEN** `S2CGameOver { loser: Some(Player_A), reason: GameOverReason::Disconnection }` is broadcast on the reliable channel. | BLOCKING |
| NP-26 | **GIVEN** the server is in any RSM phase (DRAFT_INITIAL, DRAFT_SHOP, DRAFT_AUCTION, PLACEMENT, RESOLUTION, GAME_OVER), **WHEN** a `C2SHeartbeat` is received from a player, **THEN** `disconnect_trackers[Player_A]` is reset, no S2C message of any type is sent in response, and the current RSM phase is unchanged. | BLOCKING |
| NP-27 | **GIVEN** the server has advanced past LOBBY (RSM phase is DRAFT_INITIAL or later) and a client sends `C2SCreateRoom`, `C2SJoinRoom`, `C2SSelectClass`, or `C2SConfirmClass`, **WHEN** the server processes it, **THEN** the message is silently discarded per Rule 4 — no S2C response is sent and no state changes. | BLOCKING |
| NP-28 | **GIVEN** both players have submitted their PLACEMENT and the server fires `S2CPlacementReveal`, **WHEN** the message is delivered, **THEN** ALL connected players receive it AND the `placements` field contains entries for BOTH players (both players' cards are in the reveal payload). (Integration test — requires two live clients to verify all-player delivery.) | BLOCKING |
| NP-29 | **GIVEN** a player sends `C2SPlaceBid` during DRAFT_AUCTION with `amount < player.gold` but `amount <= last_accepted_bid` (amount too low), **WHEN** the server processes it, **THEN** `S2CAuctionBidRejected { reason: BidRejectedReason::AmountTooLow }` is sent unicast to the bidding player and no auction state changes. (Integration test — requires two live clients to confirm non-bidding player does NOT receive the rejection.) | BLOCKING |
| NP-30 | **GIVEN** unit A deals non-lethal damage to unit B during RESOLUTION combat (B survives the hit), **WHEN** the server emits `S2CResolutionEvent`, **THEN** the event batch contains a `CombatDamage { attacker_id: A, target_id: B, damage > 0, was_lethal: false }` entry at the correct sub_step, and no `UnitDied` entry follows it for unit B within that sub_step. | BLOCKING |
| NP-31a | **GIVEN** a SHIELD keyword is consumed on a unit during RESOLUTION, **WHEN** the server emits `S2CResolutionEvent`, **THEN** the event batch contains `KeywordTriggered { source_unit_id: Some(unit), sub_step, payload: KeywordPayload::ShieldConsumed }` at the sub_step where the attack was blocked. The preceding `CombatDamage` entry for that attack has `damage: 0`. | BLOCKING |
| NP-31b | **GIVEN** a STUN is applied to a unit during RESOLUTION, **WHEN** the server emits `S2CResolutionEvent`, **THEN** the event batch contains `KeywordTriggered { source_unit_id: Some(attacker), sub_step, payload: KeywordPayload::StunApplied { duration_rounds } }` at the sub_step where the stun is applied. | BLOCKING |
| NP-31c | **GIVEN** a SILENCE is applied to a unit during RESOLUTION, **WHEN** the server emits `S2CResolutionEvent`, **THEN** the event batch contains `KeywordTriggered { source_unit_id: Some(attacker), sub_step, payload: KeywordPayload::SilenceApplied { duration_rounds } }` and the target's `silenced_until_round` is populated in the next `S2CGameSnapshot` snapshot. | BLOCKING |
| NP-31d | **GIVEN** a LEADER unit is present on the board during RESOLUTION sub-step 1, **WHEN** the server emits `S2CResolutionEvent`, **THEN** the event batch contains `KeywordTriggered { source_unit_id: Some(leader), sub_step: 1, payload: KeywordPayload::LeaderSnapshotTaken { leader_unit_id } }`. | BLOCKING |
| NP-31e | **GIVEN** a BODYGUARD bond is established during RESOLUTION, **WHEN** the server emits `S2CResolutionEvent`, **THEN** the event batch contains `KeywordTriggered { source_unit_id: Some(bodyguard), sub_step, payload: KeywordPayload::BodyguardBondCreated { bodyguard_id, protected_id } }` at the sub_step of the attack that triggered the bond. | BLOCKING |
| NP-31f | **GIVEN** the OUTNUMBERED condition flips for a player during RESOLUTION, **WHEN** the server emits `S2CResolutionEvent`, **THEN** the event batch contains `KeywordTriggered { source_unit_id: None, sub_step, payload: KeywordPayload::OutnumberedFlipped { player_id, active } }` at the sub_step where unit counts changed. | BLOCKING |
| NP-31g | **GIVEN** a COUNTERATTACK fires during RESOLUTION, **WHEN** the server emits `S2CResolutionEvent`, **THEN** the event batch contains both `KeywordTriggered { payload: CounterattackFired, sub_step }` AND a `CombatDamage { is_counterattack: true }` entry for the return strike, with `KeywordTriggered` appearing before `CombatDamage` in the array for that sub_step. | BLOCKING |
| NP-32 | **GIVEN** a REPEL, ATTRACT, or TELEPORT keyword displaces a unit during RESOLUTION, **WHEN** the server emits `S2CResolutionEvent`, **THEN** the event batch contains a `DisplacementEvent` entry with correct `from_lane`, `from_cell`, `to_lane`, `to_cell`, and `kind`. (The "BoardPosition converges" assertion is advisory — unreliable replication is suppressed during RESOLUTION animation per the RESOLUTION rendering contract. Test only the event batch fields.) | BLOCKING |
| NP-33 | **GIVEN** a fake objective is destroyed during RESOLUTION (attacker's spawn range expands), **WHEN** the server emits `S2CResolutionEvent`, **THEN** the event batch contains both `ObjectiveDestroyed { was_fake: true }` AND `SpawnRangeChanged { player_id: attacker, new_spawn_range_cells }`, with `SpawnRangeChanged` appearing **after** `ObjectiveDestroyed` in the events array. | BLOCKING |
| NP-34 | **GIVEN** Player A submits their PLACEMENT and Player B has not yet submitted, **WHEN** the server processes Player A's `C2SSubmitPlacement`, **THEN** `S2COpponentSubmitted { player_id: Player_A }` is delivered to Player B's transport connection within the same reliable channel message sequence. | BLOCKING |
| NP-35 | **GIVEN** a reconnecting player's `S2CGameSnapshot` is processed, **WHEN** both `PlayerSnapshot` entries in `players` are inspected, **THEN** `class_id` is non-zero/non-None for both players (server-observable assertion — confirms the field is populated, not that the UI renders correctly). | BLOCKING |

| NP-36 | **GIVEN** a client was previously connected and received a `session_token` in `S2CHandshake`, **WHEN** the client disconnects and reconnects with `C2SHello { session_token: Some(token) }` containing that token, **THEN** the server maps the new `ClientId` to the existing session slot and sends `S2CGameSnapshot` with that player's private data populated (`hand`, `gold`, `pool_snapshot` non-empty if applicable). The server does NOT create a new session slot. (Integration test.) | BLOCKING |
| NP-37 | **GIVEN** a client sends `C2SHello { session_token: Some(token) }` where the token is unrecognised or maps to a terminated session, **WHEN** the server processes it, **THEN** the server sends `S2CHandshakeRejected { reason: UnrecognisedToken }` or `SessionExpired` respectively and closes the transport connection. | BLOCKING |
| NP-38 | **GIVEN** RESOLUTION completes and at least one player's gold changed, **WHEN** the server sends post-RESOLUTION messages, **THEN** `S2CGoldBroadcast` is received on the reliable channel AFTER `S2CResolutionEvent` and BEFORE `S2CPhaseChanged(DRAFT_SHOP)` in the message sequence. (Integration test — capture reliable channel stream.) | BLOCKING |
| NP-39 | **GIVEN** the server is transitioning to DRAFT_AUCTION, **WHEN** messages are captured on the reliable channel, **THEN** `S2CAuctionCard` is received before `S2CPhaseChanged(DRAFT_AUCTION)` in the message sequence. (Integration test.) | BLOCKING |
| NP-40 | **GIVEN** the server is transitioning to DRAFT_INITIAL, **WHEN** messages are captured on the reliable channel, **THEN** `S2CDraftOffering` is received before `S2CPhaseChanged(DRAFT_INITIAL)` in the message sequence. (Integration test.) | BLOCKING |
| NP-41 | **GIVEN** a player reconnects mid-game, **WHEN** the server processes the reconnect, **THEN** `S2CObjectiveIdentities` is sent unicast to the reconnecting player after `S2CGameSnapshot` and before any `S2CPhaseChanged` or other phase-actionable message. (Integration test.) | BLOCKING |
| NP-42 | **GIVEN** a player sends `C2SSubmitPlacement` with a batch where the same `card_id` appears more than once, **WHEN** the server processes it, **THEN** the entire batch is silently discarded, `player.submitted` remains `false`, and no S2C message is sent to either player. | BLOCKING |
| NP-43 | **GIVEN** a client is IN_GAME and sends `C2SRequestSnapshot`, **WHEN** the server processes it AND at least `snapshot_cooldown_ms` has elapsed since the last snapshot was sent to this client, **THEN** `S2CGameSnapshot` is sent unicast to that client with current authoritative state. If `snapshot_cooldown_ms` has NOT elapsed, the request is silently ignored. | BLOCKING |
| NP-44 | **GIVEN** the server is in LOBBY phase and a client sends `C2SCreateRoom` or `C2SJoinRoom`, **WHEN** the server processes it, **THEN** the message is accepted and processed (not discarded) — positive phase-gate test confirming LOBBY messages are valid in LOBBY. | BLOCKING |
| NP-45 | **GIVEN** a player reconnects during PLACEMENT, DRAFT_SHOP, DRAFT_INITIAL, or RESOLUTION (any non-DRAFT_AUCTION phase), **WHEN** `S2CGameSnapshot` is received, **THEN** `auction_state` is `None`. | BLOCKING |

## Open Questions

1. **Lightyear 0.26 per-client unicast API** — `S2CGameSnapshot`, `S2CGoldUpdate`, and `S2CCardAcquired` require unicast delivery to a specific `ClientId`. Verify `ConnectionManager` unicast API in Lightyear 0.26 docs before implementing. If it doesn't exist, the secret-information model breaks and the snapshot architecture must be redesigned. **Owner:** Network programmer. **Priority: HIGH — blocks implementation start.**

2. **Lightyear 0.26 component visibility filtering** — `TrapBoardState.card_id` requires per-client scoping (identity hidden from opponent). Verify whether Lightyear 0.26 supports field-level or entity-level replication filtering per client. If not, use two separate components (`TrapPresence` broadcast + `TrapIdentity` owner-only). **Owner:** Network programmer. **Priority: HIGH.**

3. **Reliable channel in-order delivery across message types** — `S2CResolutionEvent` must precede `S2CPhaseChanged(DRAFT_SHOP)` (NP-8). Verify whether Lightyear's reliable channel guarantees strict FIFO ordering across all message types on the same channel, or whether an explicit sequence number field is needed. **Owner:** Network programmer. **Priority: HIGH.**

4. ~~**New constants in `game-config.md`**~~ — **RESOLVED (R2 2026-04-29):** `protocol_version`, `hello_timeout_ms`, `ack_timeout_ms`, `heartbeat_interval_ms` added to `game-config.md` struct, Tuning Knobs, Interactions table, and GCN-DEFAULTS AC. Registry updated. ✓

5. ~~**`PlayTarget::TargetUnit` extension**~~ — **RESOLVED (R3 2026-04-30):** Keyword System GDD (approved) confirms no keyword requires targeting current-PLACEMENT units. `PlayTarget` enum is stable. No extension needed.

6. **Sang Méprise reconnect gap** — `S2CGameSnapshot` has no `sang_meprise_active` field. A Sacrier player who reconnects mid-RESOLUTION while Sang Méprise is active loses the reveal information they paid a card for. This is tracked as OQ-CS-2 in class-system.md. **Owner:** Network Protocol + Objective System. **Priority: MEDIUM** (upgraded from LOW — card expenditure + information loss is a fairness issue, not just polish; must resolve before M3 implementation).

7. **Mid-RESOLUTION reconnect + HASTE policy** — `UnitBoardState` has no `haste_active` field. A client reconnecting mid-RESOLUTION after sub-step 1 cannot determine which units had HASTE (and already acted in SS1) vs. units subject to summoning sickness. The AnimQueue cannot correctly fast-forward without this. **Two options:** (A) Add `haste_active: bool` to `UnitBoardState` — set to `true` for all units whose `card_id` has the HASTE keyword; (B) Accept that mid-RESOLUTION reconnect replays the entire RESOLUTION from SS1, requiring the server to buffer the full current RESOLUTION event log until phase exits. **Owner:** Network Protocol + Keyword System. **Priority: MEDIUM — must decide before RESOLUTION implementation begins.**
