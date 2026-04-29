# ADR-011: Reconnect Flow and Game Snapshot Protocol

## Status

Accepted

## Date

2026-04-29

## Last Verified

2026-04-29

## Decision Makers

User + network-programmer (protocol design), technical-director (authority model
validation), creative-director (secret-information stripping validation)

## Summary

On reconnect, the server unicasts a full `S2CGameSnapshot` to the reconnecting
client before any live game messages are delivered. A `SessionToken` issued at
first connect is the sole identity-mapping mechanism across Lightyear transport
reconnects (Lightyear assigns a new `ClientId` on every WebSocket connect).
The server holds live messages in a per-player queue until snapshot delivery is
confirmed, and re-sends all session-start messages — `S2CObjectiveIdentities`,
`S2CHandshake`, and `S2CPhaseChanged` — in a fixed order before unfreezing the
live message queue.

## Engine Compatibility

| Field | Value |
|-------|-------|
| **Engine** | Bevy 0.18 + Lightyear 0.26 |
| **Domain** | Networking |
| **Knowledge Risk** | HIGH — Lightyear 0.26 (released January 2026) is entirely post-cutoff. `ClientId` reassignment on WebSocket reconnect, `NetworkTarget::Single(ClientId)` unicast shape, reliable channel ordering, and `OnConnected`/`OnDisconnected` event firing semantics must all be verified before implementing reconnect-path code. |
| **References Consulted** | `docs/engine-reference/bevy/VERSION.md`, `design/gdd/network-protocol.md` (Rules 3, 6, 7, 8; Section D.1; NP-9 through NP-22), ADR-001 (objective identity unicast, re-send requirement), ADR-002 (authority model), ADR-008 (channel config — all reconnect messages on `ReliableChannel`) |
| **Post-Cutoff APIs Used** | Lightyear 0.26 `OnConnected` / `OnDisconnected` events; `NetworkTarget::Single(ClientId)` unicast target (exact variant unverified — may be `NetworkTarget::Only(vec![client_id])`); `MessageSender<T>` unicast send from server systems; per-connection reliable channel enqueue ordering |
| **Verification Required** | (1) Confirm `ClientId` is always new on transport reconnect in Lightyear 0.26 (not reused). (2) Confirm `NetworkTarget::Single(ClientId)` is the correct unicast variant name. (3) Confirm reliable channel enqueue order is the delivery order for a given connection. (4) Confirm `OnConnected` fires synchronously in the Bevy `Update` schedule, not deferred. (5) Confirm messages enqueued before `OnConnected` processes are not delivered to the new `ClientId`. |

> **Note**: Knowledge Risk is HIGH. This ADR must be re-validated if Lightyear
> upgrades to 0.27 or beyond. Flag as "Superseded" and write a new ADR on upgrade.

## ADR Dependencies

| Field | Value |
|-------|-------|
| **Depends On** | ADR-001 (objective identity unicast — establishes that `S2CObjectiveIdentities` must be re-sent on reconnect and is never auto-replicated); ADR-002 (client-server authority model — server is sole state authority; client rebuilds from snapshot, no local prediction to reconcile); ADR-008 (channel config — all reconnect messages route on `ReliableChannel`) |
| **Enables** | Full networking implementation for any system that touches player disconnection or reconnection |
| **Blocks** | Any story implementing `OnConnected`, `OnDisconnected`, or the session handshake path; any system that sends live S2C messages to a specific player (must check `snapshot_sent` before enqueuing) |
| **Ordering Note** | ADR-009 (Round State Machine) is referenced but not yet written as an ADR document. This ADR assumes the RSM is the authority on `disconnect_grace_seconds` and the GAME_OVER transition — that assumption must be confirmed when ADR-009 is authored. ADR-002 explicitly lists ADR-011 as enabled by the authority model. |

## Context

### Problem Statement

Lightyear 0.26 assigns a new `ClientId` on every WebSocket transport connection.
When a player's browser tab drops and reconnects — whether from a network hiccup,
a backgrounded WASM tab, or an OS sleep event — the server cannot use the
`ClientId` alone to identify which session slot the reconnecting client belongs to.
Without a session identity mechanism, the server either treats the reconnect as a
new player (denying access to the in-progress game) or has no safe way to resume
the session.

Beyond identity, a reconnecting client has missed an unknown number of live game
messages. Lightyear does not replay reliable messages sent on a previous
connection's channel after transport reconnect — the delivery guarantee applies
only within a single transport session. A client that reconnects during PLACEMENT
after having submitted its cards, for example, must be able to reconstruct its
complete board view without receiving every `S2CGoldUpdate`, `S2CCardAcquired`,
or component replication event it missed.

The bluff mechanic introduces a third constraint: secret information (own
`is_fake` per objective, hand contents, shop slots, pool state) must be re-delivered
privately to the reconnecting player only, with opponent secret data stripped. A
broadcast snapshot would leak the information that makes the bluff game work.

Finally, live game messages in-flight from other server systems — a concurrent
`S2CGoldUpdate` triggered by a kill reward in the same frame as the reconnect —
must not arrive at the client before the snapshot rebuilds client state.
A `S2CGoldUpdate` arriving before the snapshot would be applied to an empty
client ECS world, producing a phantom gold value with no associated player
entity.

### Current State

No reconnect handling exists. This ADR is a greenfield decision. It precedes
any implementation of handshake, snapshot, or session resume logic.

### Constraints

- **Lightyear 0.26 `ClientId` reassignment**: Every WebSocket transport connect
  produces a new `ClientId`. The reconnect identity mechanism cannot rely on
  `ClientId` persistence. An application-layer token is mandatory.
- **No reliable message replay across reconnects**: Lightyear's reliability
  guarantee covers in-session delivery only. Messages sent on a previous
  connection's channel are not replayed when a new transport connection opens.
  All session-state messages — including `S2CObjectiveIdentities` — must be
  explicitly re-sent.
- **Bluff game secret isolation**: `is_fake` must never appear in the opponent's
  snapshot. Secret stripping must be server-enforced before unicast send, not
  post-hoc filtered on the client.
- **WASM/WebSocket half-open TCP**: Browser tabs that are backgrounded can enter
  half-open TCP states where `OnDisconnected` does not fire for 2–7 minutes.
  `C2SHeartbeat` (sent on `UnreliableChannel` every ~5s) provides a fallback
  disconnect signal. The reconnect flow must remain correct whether the disconnect
  was detected via `OnDisconnected` or via heartbeat gap.
- **Same-frame race with live message systems**: Server systems that send live
  S2C messages run in the same Bevy `Update` schedule as the reconnect handler.
  Without explicit gating, a `S2CGoldUpdate` can be enqueued to the reconnecting
  client's new `ClientId` in the same frame the snapshot is enqueued — with no
  ordering guarantee between them unless the snapshot system is scheduled first.
- **LOBBY reconnect not in scope**: Game Session System Rule 9 specifies that
  any disconnect during LOBBY immediately cancels the session. This ADR covers
  only in-game reconnect (after DRAFT_INITIAL has begun). LOBBY has no
  reconnect-with-grace-window at MVP.

### Requirements

- TR-NP-01: A reconnecting player is identified by a `SessionToken` issued at
  first connect, presented in `C2SHello` on reconnect. The token maps to the
  existing session slot regardless of the new `ClientId`.
- TR-NP-02: On reconnect, `S2CGameSnapshot` is the first S2C message delivered
  to the reconnecting client. No live game message precedes it.
- TR-NP-03: The snapshot is unicast per reconnecting player with opponent secret
  fields (hand, shop slots, pool, `is_fake`) stripped. No broadcast snapshot
  is ever sent.
- TR-NP-04: Live messages destined for the reconnecting player that are generated
  while the snapshot is being assembled and delivered are queued server-side and
  flushed only after snapshot delivery is confirmed.
- TR-NP-07: Disconnect detection uses both `OnDisconnected` (primary) and
  `C2SHeartbeat` gap (fallback). The reconnect flow is identical regardless of
  which signal detected the disconnect.
- TR-NP-08: `S2COpponentReconnected` is broadcast to all other connected players
  when a player completes the reconnect handshake.

## Decision

On any new transport connection, the server requires a `C2SHello` message before
processing any other C2S input. If `session_token` is `Some(token)`, the server
maps the token to an active session and executes the reconnect flow below. If
`session_token` is `None`, this is a fresh connect and the session creation or
join flow applies.

**Session token identity is the sole reconnect identity bridge.** The server
maintains a `HashMap<SessionToken, (SessionId, PlayerId)>` mapping. The old
`ClientId` is discarded when the transport drops; the new `ClientId` is bound
to the session slot via this map.

**The server holds live messages** for a reconnecting player (those with
`snapshot_sent[player] == false`) in a per-player queue. Messages are not
dropped — they are dequeued and flushed in their original order immediately
after the snapshot system marks `snapshot_sent[player] = true`. This preserves
economy consistency (a kill reward that was computed during the drop window
still arrives; it just arrives after the snapshot rebuilds the player's world).

**Re-sent messages on reconnect are mandatory and ordered.** These four messages
are always sent on reconnect, in this exact sequence, before any queued live
messages are flushed:

1. `S2CHandshake` (session token reissued, same token value)
2. `S2CGameSnapshot` (full authoritative state, secrets stripped per player)
3. `S2CObjectiveIdentities` (per ADR-001 — reliable delivery not guaranteed across transport reconnect)
4. `S2CPhaseChanged` (current phase, current round, current `timer_remaining_ms`)

After step 4 is enqueued, `snapshot_sent[player] = true` is written. The
queued live message flush runs in the next system scheduled after the snapshot
system in the same `Update` frame.

### Architecture

```
TRANSPORT LAYER                 SERVER                          CLIENT
                                                                (new connection)
new WebSocket connect  -------> OnConnected fires
                                new ClientId assigned
                                                <-------------- C2SHello {
                                                                  session_token: Some(token)
                                                                }
                                hello_timeout_ms watchdog
                                starts on OnConnected
                                (default 5000ms; close
                                connection if no C2SHello)

                                token lookup:
                                  HashMap<SessionToken, (SessionId, PlayerId)>
                                  found -> session slot identified
                                  not found -> S2CHandshakeRejected + close

                                snapshot_sent[player] = false
                                live message queue frozen for player

  [MANDATORY SEND ORDER — all on ReliableChannel, all unicast to new ClientId]

                                1. S2CHandshake { session_token (same value) }
                                                --------------> client persists token
                                2. S2CGameSnapshot (see payload below)
                                                --------------> client rebuilds world
                                3. S2CObjectiveIdentities (own is_fake per lane)
                                                --------------> client updates cache
                                4. S2CPhaseChanged { phase, round, timer_ms }
                                                --------------> client enters correct phase UI

                                snapshot_sent[player] = true

  [QUEUED LIVE MESSAGES FLUSHED — in original order]

                                flush deferred queue for player
                                (S2CGoldUpdate, S2CCardAcquired, etc.
                                 computed during the reconnect window)
                                                --------------> client applies sequentially

  [BROADCAST TO OTHER PLAYERS]

                                S2COpponentReconnected { player_id }
                                                ========> all other connected players

  [SESSION RESUMES]

                                Live game message flow resumes normally.
                                All systems that check snapshot_sent[player]
                                now see true and enqueue directly.
```

### Key Interfaces

```rust
// shared/src/protocol.rs — wire types

/// Issued by server at first connect. Client persists and presents on reconnect.
/// [u8; 16] = UUID v4 (128-bit), server-generated, cryptographically random.
pub type SessionToken = [u8; 16];

/// Client sends first on any transport connection (fresh or reconnect).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct C2SHello {
    pub protocol_version: u32,
    /// None = fresh connect. Some(token) = reconnect identity claim.
    pub session_token: Option<SessionToken>,
}

/// Server sends in response to a successful handshake (fresh or reconnect).
/// On reconnect the same token value is reissued — client refreshes its stored copy.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct S2CHandshake {
    pub protocol_version: u32,
    pub session_id: SessionId,
    pub session_token: SessionToken,
}

/// Full authoritative game state. Unicast per player; secrets stripped before send.
/// Sent as first game message on every connect and reconnect (NP-9).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct S2CGameSnapshot {
    pub protocol_version: u32,
    pub round_number: u32,
    pub phase: RoundPhase,
    /// Milliseconds remaining in the current phase timer. 0 if no active timer.
    pub timer_remaining_ms: u32,
    /// One entry per player in the session (always 2 for OneVOne).
    pub players: Vec<PlayerSnapshot>,
    pub board: BoardSnapshot,
    /// Non-None only during DRAFT_AUCTION. Entire field omitted in all other phases.
    pub auction_state: Option<AuctionSnapshot>,
}

/// Per-player projection within S2CGameSnapshot.
///
/// SECRET STRIPPING RULE (enforced by server before unicast send):
///   For the RECIPIENT'S OWN entry: all fields populated.
///   For the OPPONENT'S entry:
///     - hand:          always empty Vec
///     - shop_slots:    always empty Vec
///     - pool_snapshot: always empty Vec
///     - objectives:    ObjectiveSnapshot.is_real = false for all entries
///                      (opponent never knows own fake/real assignment)
///   PUBLIC fields present in BOTH entries (not stripped):
///     - player_id, gold, spawn_range_cells, mana_cap, submitted
///     - current_mana, reserve_mana  (opponent mana visibility: confirm with GDD)
///     - opponent_objectives (hp + destruction state, is_fake only after destruction)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerSnapshot {
    pub player_id: PlayerId,
    /// Gold is public — included for both own and opponent entry. See NP-19.
    pub gold: u32,
    pub current_mana: u32,
    pub reserve_mana: u32,
    /// Cells available for Minion placement (1–3). Public — opponent's value IS transmitted.
    pub spawn_range_cells: u8,
    /// Current mana cap (default 10; can reach 11–12 via fake objective reward).
    pub mana_cap: u8,
    /// True if this player has submitted their PLACEMENT this round. Public.
    pub submitted: bool,

    // --- Own player only (STRIPPED from opponent's copy) ---

    /// Own hand contents. Empty Vec in opponent's copy.
    pub hand: Vec<CardId>,
    /// Current personal shop offering (3 slots). Empty Vec in opponent's copy.
    pub shop_slots: Vec<CardId>,
    /// Full pool state: (card_id, copies_remaining). Empty Vec in opponent's copy.
    pub pool_snapshot: Vec<(CardId, u8)>,

    // --- Objective data (secret-stripped differently for own vs. opponent) ---

    /// Own objectives (5 entries, one per lane).
    /// ObjectiveSnapshot.is_real is populated truthfully for own entry.
    /// In opponent's copy: is_real = false for ALL entries (always — not conditionally).
    pub objectives: Vec<ObjectiveSnapshot>,
    /// Opponent's objectives as seen by this player (hp + destruction state only).
    /// was_fake: None until destroyed; Some(v) after ObjectiveDestroyed reveals it.
    pub opponent_objectives: Vec<OpponentObjectiveSnapshot>,
}

/// Own objective as seen by the owning player.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ObjectiveSnapshot {
    pub lane: u8,
    pub hp: u8,
    /// Owner knows real/fake. In opponent's copy of PlayerSnapshot: always false.
    /// SERVER INVARIANT: never set is_real = true in the opponent's PlayerSnapshot copy.
    pub is_real: bool,
    pub is_destroyed: bool,
}

/// Opponent objective as seen by the non-owning player.
/// is_fake is deliberately absent — opponent does not know until destruction.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OpponentObjectiveSnapshot {
    pub lane: u8,
    pub hp: u8,
    pub is_destroyed: bool,
    /// None = not yet destroyed (identity still hidden).
    /// Some(v) = destroyed; was_fake revealed via S2CResolutionEvent::ObjectiveDestroyed.
    /// SERVER INVARIANT: if is_destroyed = true then hp MUST = 0.
    pub was_fake: Option<bool>,
}

/// Board state for reconnect. All fields required — component replication is NOT
/// re-sent on reconnect; the snapshot is the authoritative board source on join.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BoardSnapshot {
    pub units: Vec<UnitBoardState>,
    pub traps: Vec<TrapBoardState>,
    pub structures: Vec<StructureBoardState>,
    pub fields: Vec<FieldBoardState>,
    /// Prism collection state per lane. Required: PrismPresence component
    /// replication does not replay on reconnect.
    pub prisms: Vec<PrismBoardState>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UnitBoardState {
    /// server-assigned NetworkId.id — client maps this to a local ECS Entity
    pub unit_id: EntityId,
    pub card_id: CardId,
    pub owner: PlayerId,
    pub lane: u8,
    pub cell: u8,
    pub current_hp: u8,
    pub atk: u8,
    /// Armor rating (damage reduction class).
    pub ar: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TrapBoardState {
    pub trap_id: EntityId,
    pub owner: PlayerId,
    pub lane: u8,
    pub cell: u8,
    /// SECRET STRIPPING RULE:
    ///   Own trap: Some(card_id) — owner knows what card was placed.
    ///   Opponent's trap: None — identity hidden until triggered or destroyed.
    pub card_id: Option<CardId>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StructureBoardState {
    pub structure_id: EntityId,
    pub card_id: CardId,
    pub owner: PlayerId,
    pub lane: u8,
    pub cell: u8,
    pub current_hp: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FieldBoardState {
    pub field_id: EntityId,
    pub card_id: CardId,
    pub owner: PlayerId,
    pub lane: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PrismBoardState {
    pub lane: u8,
    /// true = prism collected this game; false = still available on board.
    pub collected: bool,
}

/// Auction state — included only when phase == DRAFT_AUCTION. None otherwise.
/// S2CAuctionCard is NOT re-sent on reconnect; this field is the sole reconnect source.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuctionSnapshot {
    pub card_id: CardId,
    /// Last accepted bid amount. Minimum valid next bid = last_accepted_bid + 1.
    pub last_accepted_bid: u32,
    /// None = no bids placed yet in this auction round.
    pub current_leader: Option<PlayerId>,
    pub timer_remaining_ms: u32,
}

// --- Server-side reconnect state (not on the wire) ---

/// Per-session server resource. Tracks reconnect status per player.
/// Systems that send live S2C messages MUST check this before enqueuing.
pub struct ReconnectTracker {
    /// Keyed by PlayerId. false = snapshot not yet sent; live messages must be queued.
    pub snapshot_sent: HashMap<PlayerId, bool>,
    /// Deferred messages for each player awaiting snapshot delivery.
    /// Flushed in order immediately after snapshot_sent[player] is set to true.
    pub deferred_queue: HashMap<PlayerId, Vec<DeferredMessage>>,
    /// Token → (SessionId, PlayerId) mapping. Populated at session start.
    /// Entry removed when session ends (GAME_OVER + ack).
    pub token_map: HashMap<SessionToken, (SessionId, PlayerId)>,
}
```

### Implementation Guidelines

**System scheduling discipline (critical):**

The reconnect snapshot system must be scheduled BEFORE all live-game message
systems in the same Bevy `Update` schedule. Use Bevy system ordering (`.before()`
or system sets) to enforce this. If the snapshot system and a `S2CGoldUpdate`
system run in the same frame with no ordering constraint, Bevy may execute them
in any order, producing a race where `S2CGoldUpdate` is enqueued before
`S2CGameSnapshot` for the new `ClientId`.

Recommended system set structure (to be confirmed in the RSM ADR):

```
// Bevy Update schedule ordering for reconnect path:
//
// SystemSet::ReconnectHandshake      <-- runs first
//   handle_reconnect_system           (snapshot assembly, snapshot send, snapshot_sent = true)
//
// SystemSet::LiveMessages            <-- runs after ReconnectHandshake
//   flush_deferred_queue_system       (checks snapshot_sent; flushes if true)
//   all other S2C-sending systems     (check snapshot_sent before enqueue)
```

**Per-system gating contract:**

Every server system that sends an S2C message targeting a specific player MUST
check `ReconnectTracker.snapshot_sent[player]` before enqueuing. If `false`,
push the message to `deferred_queue[player]` instead of sending directly. This
is a coding convention — it is not enforced by the channel or the engine. Violating
it produces the corrupt-state-before-snapshot failure mode (NP-9).

The check is NOT required for broadcast messages (e.g., `S2CPhaseChanged`,
`S2CPlacementReveal`) — broadcast messages are received by all currently connected
players; a reconnecting player is not yet connected when the broadcast fires, so
they will never receive the missed broadcast. The snapshot covers their state.

**`hello_timeout_ms` watchdog:**

On `OnConnected`, start a `hello_timeout_ms` (default 5000ms) countdown for that
connection. If `C2SHello` is not received within the window, close the transport
connection. Do NOT send `S2CHandshakeRejected` on timeout — silence is the
response to a non-speaking client (Rule 3 in `network-protocol.md`).

**Token validation failure:**

If `C2SHello.session_token = Some(token)` and the token is not found in
`ReconnectTracker.token_map` (expired, unknown, or session already ended), send
`S2CHandshakeRejected { server_version, client_version }` and close the connection.
Do not leak session existence information in the rejection message — use the same
rejection payload regardless of failure reason.

**`S2CObjectiveIdentities` re-send invariant:**

`S2CObjectiveIdentities` was originally sent at `DRAFT_INITIAL`. Lightyear's
reliable delivery guarantee applied only to the original transport session — it
is not replayed on reconnect. The reconnect path MUST explicitly re-send
`S2CObjectiveIdentities` from `HiddenObjectives` server resource. This is step 3
in the mandatory send order. Skipping it produces a client with stale or missing
`is_fake` data, breaking the bluff mechanic silently (the client would show
incorrect objective identity on hover for the rest of the game).

**`S2CPhaseChanged` re-send carries live timer:**

The `timer_remaining_ms` field in the re-sent `S2CPhaseChanged` must reflect the
remaining time in the current phase at the moment the message is assembled — not
the original phase duration. The server must read the current phase timer value
from the RSM at snapshot assembly time. A stale `timer_remaining_ms = 0` would
cause the client to display an expired timer or skip a phase UI.

**Sang Meprise interaction:**

If `S2CSangMepriseReveal` was sent to the reconnecting player's opponent before
the disconnect, it was delivered on the old transport connection. After reconnect,
the opponent already has the reveal cached in their local client state — no
re-send is needed for the opponent. However, if the reconnecting player themselves
was the target of the reveal (i.e., they were shown the opponent's objectives),
that reveal must be re-sent on reconnect. The server must track whether
`S2CSangMepriseReveal` was sent to a given player in the current round and
re-send if so. This is an edge case — the common path requires no special handling.

**Session end cleanup:**

When the session reaches GAME_OVER and `C2SAcknowledgeResult` is received from
all players (or `ack_timeout_ms` expires), remove the session's entries from
`ReconnectTracker.token_map`. Stale tokens in the map are a memory leak in
long-running server processes and would cause false positive reconnect matches if
`SessionToken` space is exhausted (extremely unlikely with 128-bit tokens, but
the cleanup is good hygiene).

## Alternatives Considered

### Alternative 1: Lightyear Room/Entity State Re-subscription

- **Description**: Use Lightyear's `NetworkVisibility` or Rooms API to automatically
  re-subscribe the reconnecting `ClientId` to all replicated entities. Rely on
  Lightyear's replication engine to re-send all component data to the new
  connection rather than assembling a manual snapshot.
- **Pros**: No manual snapshot assembly code. Component additions and removals are
  tracked by the replication engine automatically.
- **Cons**: As established by ADR-001's spike, Lightyear 0.26 replication operates
  at entity granularity. It cannot selectively strip per-component secret data
  (`is_fake`, hand contents) for the reconnecting player. Any automatic re-subscription
  would send the same data to both players. Additionally, the order in which
  replicated components arrive at the client is not guaranteed relative to reliable
  channel messages — the snapshot → live-message ordering invariant (NP-9) would
  be broken. Rejected because it is architecturally incompatible with the bluff
  game's privacy requirements.
- **Estimated Effort**: Low to implement, but unworkable given privacy constraints.
- **Rejection Reason**: Lightyear 0.26 has no per-component-per-client replication
  scope. Automatic re-subscription leaks secret data. Rejected.

### Alternative 2: Delta-Replay — Re-send All Missed Messages

- **Description**: The server logs every S2C message sent during a session.
  On reconnect, replay all messages sent after the player's disconnect timestamp
  in order.
- **Pros**: No special snapshot assembly logic. Clients reconstruct state by
  replaying the event stream. Conceptually clean for an event-sourced system.
- **Cons**: Message log storage grows unboundedly over a game session. Replaying
  a large backlog (e.g., a player disconnects at DRAFT_INITIAL and reconnects at
  RESOLUTION round 8) produces a burst of hundreds of messages with no guarantee
  the client processes them in finite time before the next phase begins. Secret
  data (hand, gold updates) was previously unicast correctly — replay must
  preserve that routing per message, adding significant log complexity. If any
  message was never sent (e.g., `S2CObjectiveIdentities` pre-dates the log
  window), the delta is incomplete.
- **Estimated Effort**: High — log infrastructure, replay engine, and per-message
  routing metadata add substantial complexity.
- **Rejection Reason**: Unbounded memory cost, burst delivery risk, and the
  complexity of maintaining per-message routing metadata in the log. The full
  snapshot approach is simpler, has bounded size, and is always correct
  regardless of how long the player was disconnected. Rejected.

### Alternative 3: Client-Side State Persistence (LocalStorage / IndexedDB)

- **Description**: The WASM client serializes game state to browser LocalStorage
  or IndexedDB on every state change. On reconnect, the client loads its own
  saved state and sends a `C2SStateChecksum` to the server for validation.
  If the checksum matches, the server skips the snapshot and sends only a delta.
- **Pros**: Reduces reconnect bandwidth for the common case (short disconnect,
  state unchanged).
- **Cons**: Violates ADR-002's authority model — the client presents its own
  state as authoritative, which enables state manipulation attacks. Even with
  checksum validation, a malicious client can forge a checksum against a known
  good state and then mutate local storage. Browser storage APIs are unreliable
  in WASM (storage quota exceeded, private browsing mode, iOS Safari quirks).
  The delta computation on the server requires diffing two full game states,
  which is not simpler than assembling the full snapshot. The bluff game's
  privacy requirements still require the server to strip secrets on the delta
  regardless.
- **Estimated Effort**: High — client storage infrastructure, checksum protocol,
  server-side diff engine, and all the secret-stripping logic still required.
- **Rejection Reason**: Contradicts server-authority model. Adds attack surface.
  Browser storage is unreliable in WASM contexts. No meaningful simplification
  over the full snapshot approach. Rejected.

### Alternative 4: Single Broadcast Snapshot (Strip Nothing, Rely on Client)

- **Description**: The server sends one `S2CGameSnapshot` broadcast to all players
  on reconnect (or caches a single pre-computed snapshot). The client ignores
  opponent secret fields it "shouldn't" read.
- **Pros**: One snapshot computation, one send. Simpler server code.
- **Cons**: The client has the opponent's `is_fake` values, hand contents, and shop
  slots in memory. Any client-side modification (browser devtools, WASM memory
  inspection, custom JS) can read them. This is the highest-severity failure mode
  for a bluff card game — the entire game mechanic collapses if either player can
  read the opponent's secret during a disconnect. There is no recovery if this leaks.
- **Estimated Effort**: Lowest of all alternatives.
- **Rejection Reason**: Catastrophic privacy failure for the core game mechanic.
  Rejected unconditionally.

## Consequences

### Positive

- Identity mapping via `SessionToken` is robust to `ClientId` churn —
  Lightyear's internal ID reassignment is fully transparent to the game layer.
- The snapshot approach is always correct regardless of disconnect duration —
  a player who reconnects after 25 seconds and one who reconnects after 28
  seconds (within `disconnect_grace_seconds = 30`) receive the same quality
  of state reconstruction.
- Secret information privacy is enforced at the message-routing boundary,
  not by client convention. A mis-route (sending the wrong snapshot to the
  wrong `ClientId`) produces an observable bug; the data is never silently
  broadcast to everyone.
- The `deferred_queue` ensures no game events are lost during the reconnect
  window. A kill reward computed while the player was reconnecting still
  arrives and is applied correctly.
- `S2CObjectiveIdentities` re-send is explicit and mandatory in the protocol
  definition — the bluff mechanic cannot silently break from a missed re-send.

### Negative

- Every server system that sends unicast S2C messages must check
  `snapshot_sent[player]` and implement the deferred-queue path. This is a
  coding convention, not engine-enforced — omitting it in a new system produces
  a hard-to-reproduce bug that only manifests during active reconnects.
- Snapshot assembly reads from multiple server resources (`HiddenObjectives`,
  board state, economy state, RSM state, auction state) in a single system.
  This produces a wide fan-out of resource reads, which in Bevy's ECS scheduler
  prevents parallelism with any system that writes those resources. Schedule
  the snapshot system carefully to avoid stalling the frame.
- The `ReconnectTracker.deferred_queue` is an unbounded queue per player. In
  pathological cases (player reconnects at the start of RESOLUTION; server
  sends ~20 resolution events and a phase transition before snapshot delivery
  confirms), the queue may hold dozens of messages. This is bounded by session
  length and phase message count — not by time — and is not a practical concern
  at MVP, but should be monitored.

### Neutral

- The mandatory re-send order (Handshake → Snapshot → ObjectiveIdentities →
  PhaseChanged) is a fixed protocol contract. Any future session-start message
  added to the protocol must be assigned a position in this order, documented
  here, and updated in the implementation.
- `AuctionSnapshot` inside `S2CGameSnapshot` is the sole reconnect source for
  auction state. `S2CAuctionCard` is not re-sent after reconnect. This is a
  deliberate simplification — the snapshot carries everything needed to render
  the auction panel.

## Risks

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|-----------|
| Lightyear 0.26 does not guarantee per-connection reliable enqueue order across message types | MEDIUM | Critical — live messages could arrive before snapshot on reconnect even when enqueued after | Verify checklist item 3 (Engine Compatibility) before implementation. If unguaranteed, add an explicit `snapshot_sequence_id: u32` field to all S2C messages and a client-side reorder buffer. |
| A new system developer omits the `snapshot_sent` check when adding a new unicast S2C message | HIGH | Medium — reconnecting client receives a live message before snapshot; local state corrupted | Code review gate: PR template must include "Does this system send unicast S2C? Does it check `snapshot_sent`?" Lint rule or doc comment on `ReconnectTracker` to warn at usage sites. |
| `OnConnected` fires before the old connection's messages are fully flushed | LOW | High — snapshot sent on new `ClientId` while old messages still in flight on old `ClientId` | Verify engine behavior (checklist item 4 and 5). If old-connection messages can still arrive after `OnConnected`, add a one-frame delay before beginning snapshot assembly. |
| `hello_timeout_ms` fires on legitimate slow WASM cold-start connects | MEDIUM | Low — player must reconnect again; second attempt usually succeeds within timeout | `hello_timeout_ms` default is 5000ms (5 seconds), well above typical WASM boot time. Monitor client-side `C2SHello` send latency post-launch; increase default in `GameConfig` if legitimate cold-start timeouts are observed. |
| `ReconnectTracker.token_map` not cleaned up on session end | LOW | Low — memory leak in long-running server; no correctness impact (128-bit token space makes collision astronomically unlikely) | Explicit cleanup in `C2SAcknowledgeResult` handler and `ack_timeout_ms` expiry path. Integration test: assert `token_map.len() == 0` after session teardown. |
| Sang Meprise reveal re-send logic is omitted (edge case) | LOW | Medium — reconnecting player loses their opponent-objective visibility granted by the spell for the rest of the round | Track `sang_meprise_sent_to: HashSet<PlayerId>` per session per round. Re-send on reconnect if player is in the set. Covered by a dedicated acceptance criterion (see Validation Criteria). |

## Performance Implications

| Metric | Before | Expected After | Budget |
|--------|--------|----------------|--------|
| CPU (snapshot assembly) | N/A | < 1ms per reconnect event (reads 5 resources; no compute) | Not in the 16.67ms frame budget hot path — reconnect is a rare event |
| Snapshot message size | N/A | ~900–4000 bytes (varies by board state; late-game boards have more units) | `snapshot_max_bytes = 16384` (configurable in `GameConfig`) |
| Deferred queue memory | N/A | < 2 KB per player during reconnect window (bounded by message count per round) | Acceptable; queue is ephemeral and flushed within 1–2 frames of snapshot delivery |
| `snapshot_sent` check per live message system | N/A | O(1) `HashMap` lookup per system per player per frame | Negligible; `HashMap<PlayerId, bool>` lookup is ~5ns |
| Reconnect end-to-end latency | N/A | `hello_timeout_ms` watchdog + 1 RTT for `C2SHello` + snapshot assembly + 1 RTT for delivery | Target < 500ms total for a player on 100ms RTT; acceptable for a 30s grace window |

## Migration Plan

This is a greenfield decision — no existing reconnect handling exists in the
codebase. No migration of existing code is required.

Implementation order:

1. Verify all Engine Compatibility checklist items (1–5) against
   `docs.rs/lightyear/0.26` before writing any reconnect-path code.
2. Add `SessionToken`, `ReconnectTracker`, and all snapshot structs to
   `shared/src/protocol.rs`.
3. Implement the `handle_reconnect_system` in `server/src/session/reconnect.rs`:
   token lookup, snapshot assembly with secret stripping, mandatory send sequence,
   `snapshot_sent = true`.
4. Implement `flush_deferred_queue_system` in `server/src/session/reconnect.rs`:
   drains `deferred_queue[player]` after `snapshot_sent = true`.
5. Add the `snapshot_sent` check to all existing unicast S2C message systems
   before any live game system stories are marked Done.
6. Write an integration test covering NP-9, NP-16, NP-17, NP-20, NP-21, NP-22
   (see Validation Criteria). These tests require a live Lightyear session —
   `World::new()` unit tests cannot verify channel delivery order.
7. Write a unit test for snapshot assembly secret-stripping (NP-16) using
   `World::new()` — no live Lightyear session needed; assert on the assembled
   `S2CGameSnapshot` struct fields directly.

**Rollback plan**: If the Lightyear 0.26 `ClientId` reassignment behavior
differs from the documented assumption (e.g., the same `ClientId` is preserved
across transport reconnects on the same IP), remove `SessionToken` from
`C2SHello` and use `ClientId` as the identity key in `ReconnectTracker.token_map`.
The rest of the reconnect flow is unchanged. Update this ADR to Superseded
and write a new ADR documenting the simplified identity mechanism.

## Validation Criteria

- [ ] **NP-9**: A client that drops and reconnects receives `S2CGameSnapshot` as
  the first S2C message on the new connection — no `S2CPhaseChanged`,
  `S2CGoldUpdate`, or any other game message precedes it in the reliable channel
  stream. (Integration test — requires a live Lightyear session.)
- [ ] **NP-16**: The `S2CGameSnapshot` produced for Player B does NOT contain
  Player A's `hand`, `shop_slots`, `pool_snapshot`, or any `ObjectiveSnapshot`
  with `is_real = true` for Player A's objectives. (Unit test — assert on
  assembled struct; no live session required.)
- [ ] **NP-16 inverse**: The `S2CGameSnapshot` produced for Player A DOES contain
  Player A's `hand`, `shop_slots`, `pool_snapshot`, and correct `is_real` values
  for Player A's own objectives. (Unit test — same test as above, opposite side.)
- [ ] **NP-17**: After a player reconnects within `disconnect_grace_seconds`,
  `S2COpponentReconnected { player_id }` is received by all other connected
  players. (Integration test.)
- [ ] **NP-18**: When phase is `DRAFT_AUCTION` at reconnect time, the received
  `S2CGameSnapshot.auction_state` is non-null and contains `card_id`,
  `last_accepted_bid`, `current_leader` (or `None`), and `timer_remaining_ms`.
  (Integration test.)
- [ ] **NP-20**: A player reconnecting during PLACEMENT receives a snapshot with
  `round_number > 0`, `phase = PLACEMENT`, all board units in `BoardSnapshot.units`,
  and their own `hand` populated if they hold cards. (Integration test.)
- [ ] **NP-21**: A player reconnecting during PLACEMENT before submitting receives
  `PlayerSnapshot.submitted = false` and `timer_remaining_ms` reflects the live
  countdown (not the original phase duration). (Integration test.)
- [ ] **NP-22**: A player reconnecting during PLACEMENT after submitting receives
  `PlayerSnapshot.submitted = true` and the placement UI is not re-presented.
  (Integration test.)
- [ ] **Deferred queue correctness**: A `S2CGoldUpdate` generated during the
  reconnect window (between `OnConnected` and `snapshot_sent = true`) arrives
  at the reconnecting client AFTER the snapshot is processed, not before.
  (Integration test — requires instrumenting the deferred queue flush and
  asserting message order on the client.)
- [ ] **`S2CObjectiveIdentities` re-send**: A reconnecting client receives
  `S2CObjectiveIdentities` as step 3 of the mandatory sequence and its
  `is_fake` cache is correctly populated with the server's `HiddenObjectives`
  values for that player. (Integration test.)
- [ ] **Sang Meprise re-send**: If `S2CSangMepriseReveal` was sent to the
  reconnecting player in the current round before the disconnect, the server
  re-sends it on reconnect and the client's local reveal state is restored.
  (Integration test — requires a round where Sang Meprise is active.)
- [ ] **Token expiry rejection**: A client presenting an unknown or expired
  `SessionToken` in `C2SHello` receives `S2CHandshakeRejected` and the
  transport connection is closed. No session state is modified. (Integration test.)
- [ ] **`hello_timeout_ms` watchdog**: A transport connection that sends no
  `C2SHello` within 5000ms is closed by the server with no S2C message sent.
  (Integration test.)

## GDD Requirements Addressed

| GDD Document | System | Requirement | How This ADR Satisfies It |
|-------------|--------|-------------|--------------------------|
| `design/gdd/network-protocol.md` | Network Protocol | Rule 3 — Session token identity: `session_token` in `C2SHello` is the sole mechanism for mapping new `ClientId` to existing session slot on reconnect | Defines `SessionToken`, `ReconnectTracker.token_map`, and token lookup flow |
| `design/gdd/network-protocol.md` | Network Protocol | Rule 6 — Replication scope: per-player snapshot with secrets stripped; single broadcast snapshot is forbidden | Defines the per-player secret-stripping rules for `PlayerSnapshot`, `ObjectiveSnapshot`, `TrapBoardState`; broadcast snapshot explicitly rejected as Alternative 4 |
| `design/gdd/network-protocol.md` | Network Protocol | Rule 7 — Reconnect snapshot: `S2CGameSnapshot` sent before any other S2C message on connect or reconnect | Documents mandatory send order and `deferred_queue` mechanism to enforce NP-9 |
| `design/gdd/network-protocol.md` | Network Protocol | Rule 8 — Disconnect detection: `OnDisconnected` + `C2SHeartbeat` gap; reconnect flow identical regardless of which signal fires | Reconnect flow is triggered by `OnConnected` regardless of how the disconnect was detected |
| `design/gdd/network-protocol.md` | Network Protocol | TR-NP-01 (NP-1) — `S2CHandshake` is the first S2C message on fresh connect | Handshake is step 1 of the mandatory reconnect sequence |
| `design/gdd/network-protocol.md` | Network Protocol | TR-NP-02 (NP-9) — `S2CGameSnapshot` first game message after reconnect | Enforced by `deferred_queue` and snapshot system scheduling before live-message systems |
| `design/gdd/network-protocol.md` | Network Protocol | TR-NP-03 (NP-16) — Snapshot does not contain opponent secrets | Enforced by secret-stripping rules on `PlayerSnapshot` and `TrapBoardState` |
| `design/gdd/network-protocol.md` | Network Protocol | TR-NP-04 — Live messages held until snapshot delivered | Defined via `ReconnectTracker.deferred_queue` and `snapshot_sent` flag |
| `design/gdd/network-protocol.md` | Network Protocol | TR-NP-07 (NP-12, NP-13, NP-24, NP-25) — Disconnect detection and grace window | Token map and reconnect tracker preserve session state for `disconnect_grace_seconds`; reconnect flow resumes within the grace window |
| `design/gdd/network-protocol.md` | Network Protocol | TR-NP-08 (NP-17) — `S2COpponentReconnected` broadcast on reconnect | Defined as the terminal step of the mandatory reconnect sequence |
| `design/gdd/network-protocol.md` | Network Protocol | Section D.1 — `S2CGameSnapshot` schema | Struct definitions in Key Interfaces are the authoritative Rust-typed implementation of the schema in Section D.1 |
| `design/gdd/network-protocol.md` | Network Protocol | NP-18 — Auction state in snapshot during `DRAFT_AUCTION` | `AuctionSnapshot` embedded in `S2CGameSnapshot`; `S2CAuctionCard` not re-sent |
| `design/gdd/network-protocol.md` | Network Protocol | NP-20, NP-21, NP-22 — Reconnect during PLACEMENT carries correct `submitted` and `timer_remaining_ms` | `PlayerSnapshot.submitted` and `S2CGameSnapshot.timer_remaining_ms` defined with exact semantics |
| `design/gdd/game-session-system.md` | Game Session System | Rule 9 — LOBBY disconnect is immediate cancel, not reconnect-with-grace | This ADR explicitly scopes to post-`DRAFT_INITIAL` reconnect; LOBBY case is out of scope and handled by GSS Rule 9 |
| `design/gdd/game-session-system.md` | Game Session System | Rule 11 — `SessionConfig` is read-only after `SessionReady`; RSM owns phase after GSS handoff | Reconnect snapshot reads `SessionConfig` as a read-only source for `mode` and `class_map`; does not mutate it |

> This ADR covers a foundational networking mechanism that enables all in-game
> systems to be resilient to player disconnection without data loss.

## Related

- `docs/architecture/adr-001-objective-identity-unicast.md` — Establishes that
  `S2CObjectiveIdentities` is never auto-replicated and must be explicitly re-sent
  on reconnect. Step 3 of the mandatory reconnect send order is a direct
  consequence of ADR-001's decision.
- `docs/architecture/adr-002-client-server-authority.md` — Authority model
  foundational to this ADR. Client is a read-only view; the snapshot is the sole
  mechanism for rebuilding that view. ADR-002 explicitly lists ADR-011 as enabled.
- `docs/architecture/adr-008-lightyear-channel-config.md` — All reconnect
  messages (`S2CHandshake`, `S2CGameSnapshot`, `S2CObjectiveIdentities`,
  `S2CPhaseChanged`) are on `ReliableChannel` per the channel assignment table.
  The `snapshot_sent` mechanism described in ADR-008 Implementation Guidelines
  (Reconnect Invariant section) is the same mechanism formalized in this ADR.
- ADR-009 (pending — Round State Machine) — The RSM owns `disconnect_grace_seconds`
  and the GAME_OVER transition when a player does not reconnect within the grace
  window. This ADR defers to ADR-009 for those decisions; ADR-009 must reference
  this ADR for the reconnect flow that fires when a player does reconnect.
- `design/gdd/network-protocol.md` — Source GDD for all wire message types,
  acceptance criteria (NP-9 through NP-22), and the `S2CGameSnapshot` schema
  (Section D.1) that this ADR formalizes in Rust types.
- `design/gdd/game-session-system.md` — LOBBY disconnect behavior (Rule 9)
  is explicitly out of scope for this ADR.
