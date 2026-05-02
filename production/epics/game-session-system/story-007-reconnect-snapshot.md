# Story 007: Reconnect and Game Snapshot

> **Epic**: Game Session System
> **Status**: Ready
> **Layer**: Core
> **Type**: Integration
> **Manifest Version**: 2026-05-01

## Pre-conditions

**All 14 ADR-011 Engine Compatibility checklist items must be verified before implementation begins.** This story has the widest dependency set in the epic and the highest combined engine risk. The lightyear-protocol-verification spike (S1-05) must be complete.

Key items that directly gate this story:
- Item 1: `ClientId` is always new on transport reconnect in Lightyear 0.26 (not reused).
- Item 2: `NetworkTarget::Single(ClientId)` is the correct unicast variant name (or documented alternative).
- Item 3: Reliable channel enqueue order is the delivery order for a given connection.
- Item 4: `OnConnected` fires synchronously in the Bevy `Update` schedule, not deferred.
- Item 5: Messages enqueued before `OnConnected` processes are not delivered to the new `ClientId`.
- Items 12–14: `OnDisconnected` API shape and semantics (also required by Story 005).

**2026-05-02 API alignment**: Current compiled Lightyear 0.26.4 usage in this repo observes connection state through `Connected` / `Disconnected` marker components with Bevy observers (`On<Add, Connected>` and `On<Add, Disconnected>`). Implement the "OnConnected"/"OnDisconnected" acceptance wording via that verified marker-component observer path, not via legacy `EventReader`/`EventWriter` APIs.

---

## Context

**GDD**: `design/gdd/game-session-system.md`, `design/gdd/network-protocol.md`
**Requirement**: TR-GSS-08 (`SessionToken` enables reconnect across new `ClientId`), TR-NP-01 through TR-NP-04, TR-NP-07, TR-NP-08

**ADR Governing Implementation**: ADR-011 (Reconnect Flow and Game Snapshot Protocol), ADR-001 (Objective Identity Unicast — `S2CObjectiveIdentities` must be re-sent), ADR-008 (Lightyear Channel Config — all reconnect messages on `ReliableChannel`), ADR-002 (Client-Server Authority — client rebuilds from snapshot, no prediction to reconcile)
**ADR Decision Summary**: On reconnect, server sends four messages in mandatory order: `S2CHandshake → S2CGameSnapshot → S2CObjectiveIdentities → S2CPhaseChanged`. Live messages destined for the reconnecting player are held in `ReconnectTracker.deferred_queue` until `snapshot_sent[player] = true`. A `hello_timeout_ms` watchdog closes connections that send no `C2SHello` within 5s.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**: `OnConnected`, unicast `MessageSender`, reliable channel ordering semantics are all Lightyear 0.26 post-cutoff APIs. `liv-bevy-018` and `liv-bevy-lightyear` skills are mandatory on all `.rs` files. Schedule the snapshot system in `SystemSet::ReconnectHandshake` before `SystemSet::LiveMessages` (per ADR-011 §Implementation Guidelines).

**Control Manifest Rules (Core layer)**:
- Required: `S2CGameSnapshot` is the first game message sent to a reconnecting client after `S2CHandshake`. No live game message may precede it on `ReliableChannel`.
- Required: Every server system that sends unicast S2C messages targeting a specific player MUST check `ReconnectTracker.snapshot_sent[player]` before enqueuing. If `false`, push to `deferred_queue[player]` instead.
- Required: Secret stripping is enforced server-side before unicast send — never post-hoc on the client.
- Forbidden: A single broadcast snapshot. `S2CGameSnapshot` is always unicast, always per-player, always secrets-stripped.
- Forbidden: `S2CAuctionCard` re-sent on reconnect. `AuctionSnapshot` inside `S2CGameSnapshot` is the sole reconnect source for auction state.

---

## Acceptance Criteria

- [ ] `handle_reconnect` system exists in `server/src/core/session/system.rs` (or `reconnect.rs`) and:
  - Triggers on `OnConnected` (or equivalent Lightyear 0.26 API)
  - Starts a `hello_timeout_ms` (default 5000ms from `GameConfig`) watchdog countdown for the new `ClientId`
  - On receiving `C2SHello { session_token: Some(token) }`:
    - Looks up token in `ReconnectTracker.token_map`; on miss sends `S2CHandshakeRejected` and closes connection; does not leak session existence
    - On hit: maps new `ClientId` to existing `PlayerId` in `SessionParticipants`
    - Sets `ReconnectTracker.snapshot_sent[player] = false`
    - Freezes live message delivery for player (deferred_queue path now active)
    - Sends the mandatory sequence on `ReliableChannel` in this exact order:
      1. `S2CHandshake { protocol_version, session_id, session_token }` (same token value reissued)
      2. `S2CGameSnapshot` (built by `build_game_snapshot`, secrets stripped per player)
      3. `S2CObjectiveIdentities` (read from `Res<HiddenObjectives>` — ADR-001 re-send requirement)
      4. `S2CPhaseChanged { phase, round, timer_remaining_ms }` (live timer value, not original duration)
    - Sets `ReconnectTracker.snapshot_sent[player] = true` after step 4 is enqueued
  - On receiving `C2SHello { session_token: None }`: this is a fresh connect — defer to room create/join flow (Story 002); this system does not handle fresh connects beyond routing them
- [ ] `build_game_snapshot` function exists (in `session/reconnect.rs` or `session/system.rs`) and:
  - Reads: `Res<RoundState>`, `Res<SessionConfig>`, `Res<PlayerEconomy>`, `Res<HiddenObjectives>`, `Res<BoardGrid>` (all board entity state), `Res<PlayerPool>`
  - Constructs `S2CGameSnapshot` with `round_number`, `phase`, `timer_remaining_ms` (live), `players` (two `PlayerSnapshot` entries), `board` (`BoardSnapshot`), `auction_state` (`Some(AuctionSnapshot)` only when phase is `DRAFT_AUCTION`, else `None`)
  - Applies secret stripping for the recipient player per ADR-011 §Key Interfaces `PlayerSnapshot` rules:
    - Own entry: all fields populated (`hand`, `shop_slots`, `pool_snapshot`, `objectives` with correct `is_real`)
    - Opponent entry: `hand`, `shop_slots`, `pool_snapshot` are empty `Vec`; all `ObjectiveSnapshot.is_real = false`
    - `TrapBoardState.card_id`: `Some(card_id)` for own traps, `None` for opponent traps
- [ ] `flush_deferred_queue` system exists and:
  - Runs in `SystemSet::LiveMessages` (after `SystemSet::ReconnectHandshake`)
  - For each player where `snapshot_sent[player] == true` and `deferred_queue[player]` is non-empty: sends all queued messages in their original enqueue order, then clears the queue
- [ ] `hello_timeout_watchdog` system exists and:
  - Tracks pending `ClientId`s that have sent no `C2SHello` within `hello_timeout_ms`
  - Closes the transport connection silently (no S2C message sent) on timeout
- [ ] `S2COpponentReconnected { player_id: PlayerId }` is broadcast to all other connected session participants after `snapshot_sent[player] = true`
- [ ] `ReconnectTracker` resource is initialised at session start (Story 004, `on_session_ready`) with token entries for all session players, all `snapshot_sent = true` (initial connects are not reconnects), empty `deferred_queues`
- [ ] `ReconnectTracker.token_map` entries for this session are cleaned up in `handle_game_over_teardown` (Story 006) — this story's integration test must verify the cleanup happens
- [ ] `SystemSet::ReconnectHandshake` is defined and scheduled before `SystemSet::LiveMessages` in the server's `Update` schedule
- [ ] All existing unicast S2C message systems have been audited and patched to check `snapshot_sent[player]` before enqueuing (with a comment referencing this requirement)
- [ ] `cargo check -p server` passes with zero warnings
- [ ] Unit test `tests/unit/session/snapshot_secret_strip_test.rs` passes — NP-16
- [ ] Integration test `tests/integration/session/reconnect_snapshot_test.rs` passes — NP-9, NP-17, deferred queue correctness

---

## Implementation Notes

*Derived from ADR-011 §Decision, §Architecture, §Implementation Guidelines:*

**Snapshot assembly is the widest resource read in the codebase**: `build_game_snapshot` reads `RoundState`, `SessionConfig`, `PlayerEconomy`, `HiddenObjectives`, `BoardGrid`, `PlayerPool`. This prevents Bevy from parallelizing any write to these resources with the snapshot system. Schedule the snapshot system carefully — it is not on the hot path (reconnect is rare) but the resource contention must be documented in the system's comments.

**`snapshot_sent` default for initial connects**: When a player first connects (Story 002 — fresh connect via `C2SHello { session_token: None }`), `snapshot_sent[player]` must be `true` immediately after the initial `S2CHandshake` + `S2CGameSnapshot` is sent at session start. All live systems must not queue messages for players who have not yet received their initial snapshot. For the MVP (LOBBY does not have a snapshot), initialize `snapshot_sent[player] = true` in `on_session_ready` — at DRAFT_INITIAL entry, all players are freshly connected and have received the initial phase broadcast.

**`timer_remaining_ms` in re-sent `S2CPhaseChanged`**: This MUST reflect the remaining time in the current phase at snapshot assembly time — not the original phase duration. Read the live countdown from `RoundState.phase_timer_remaining_ms` (or equivalent RSM field). A value of `0` would cause the client to display an expired timer.

**`S2CObjectiveIdentities` re-send**: Read from `Res<HiddenObjectives>` — this resource is defined and owned by the objective-system epic. The reconnect path re-sends the same data that was unicast at `DRAFT_INITIAL`. Do not read from `SessionConfig` for this — `HiddenObjectives` is the authoritative source for per-player fake/real assignments.

**Sang Meprise edge case**: If `S2CSangMepriseReveal` was sent to the reconnecting player in the current round (they were the target of the reveal), it must be re-sent on reconnect. Track this in `sang_meprise_sent_to: HashSet<PlayerId>` per session per round (cleared at round start). Re-send if player is in the set. This is an edge case — the NP-9 integration test does not need to cover it, but the acceptance criterion exists in ADR-011 and must be addressed before the story is marked Done.

**Token validation failure**: If `C2SHello.session_token = Some(token)` and the token is not in `token_map`, send `S2CHandshakeRejected { server_version, client_version }` with the same payload regardless of failure reason. Do not include session-existence information in the rejection.

**`deferred_queue` is an in-memory Vec per player**: It is bounded by the number of S2C messages generated during the reconnect window (typically < 20). It is not persisted. If the server restarts, the queue is lost and the player must reconnect to a fresh session (or the session is dead). This is acceptable at MVP.

---

## Out of Scope

- LOBBY reconnect (explicitly out of scope per ADR-011 — LOBBY disconnect is immediate cancel, Story 005)
- Sang Meprise re-send tracking initialisation (the `sang_meprise_sent_to` set must be defined here as part of the session state, but the population logic lives in the narrative/spell system epic)
- `C2SAcknowledgeResult` post-game acknowledgment handler (post-MVP)
- Client-side reconnect UI (client epic)
- `snapshot_max_bytes` enforcement / snapshot size monitoring (post-MVP)

---

## QA Test Cases

*QL-STORY-READY skipped — Lean mode.*

- **NP-16: secret stripping — own vs opponent**
  - Given: `S2CGameSnapshot` assembled for Player B (recipient); Player A's `PlayerSnapshot` entry is the opponent entry
  - When: `build_game_snapshot(&world, recipient_player_id = B)` is called
  - Then: `players[A].hand == []`; `players[A].shop_slots == []`; `players[A].pool_snapshot == []`; all `players[A].objectives[*].is_real == false`
  - And: `players[B].hand` is populated; `players[B].objectives[*].is_real` reflects server truth

- **NP-16 inverse:**
  - Given: Same snapshot assembled for Player A (recipient)
  - When: `build_game_snapshot(&world, recipient_player_id = A)` is called
  - Then: `players[A].hand` populated; `players[A].objectives[*].is_real` correct; opponent (B) fields stripped

- **NP-9: snapshot-first delivery (integration)**
  - Given: Player A disconnects mid-game; live game systems generate `S2CGoldUpdate` for A during the reconnect window
  - When: A reconnects and `handle_reconnect` processes `C2SHello { session_token: Some(t) }`
  - Then: `S2CGameSnapshot` is in the outbound queue before `S2CGoldUpdate` for A's new `ClientId`; deferred queue is flushed after snapshot; delivery order asserted

- **NP-17: opponent reconnected broadcast (integration)**
  - Given: Player A reconnects; Player B is connected
  - When: `snapshot_sent[A] = true` is written
  - Then: `S2COpponentReconnected { player_id: A }` in outbound queue targeting B

- **Deferred queue correctness (integration)**
  - Given: `snapshot_sent[A] = false`; 3 unicast S2C messages enqueued to A via deferred path
  - When: `flush_deferred_queue` runs after `snapshot_sent[A]` is set to `true`
  - Then: All 3 messages in the outbound queue in original enqueue order; `deferred_queue[A]` is empty

- **Hello timeout watchdog**
  - Given: New `ClientId` connected; no `C2SHello` received within 5000ms
  - When: `hello_timeout_watchdog` runs
  - Then: Transport connection closed for that `ClientId`; no S2C message sent; session state not modified

---

## Test Evidence

**Story Type**: Integration
**Required evidence**:
- `tests/unit/session/snapshot_secret_strip_test.rs` (NP-16) — passing
- `tests/integration/session/reconnect_snapshot_test.rs` (NP-9, NP-17, deferred queue) — passing
- ADR-011 Engine Compatibility checklist items 1–14 all documented with verified results in `docs/architecture/adr-011-reconnect-snapshot.md`
**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 001 (session types — `SessionToken`, `LobbyState`, `SessionId`)
- Depends on: Story 002 (room create/join — `SessionParticipants`, `ActiveSessions`)
- Depends on: Story 003 (class selection — `ClassSelections` populated for `SessionConfig`)
- Depends on: Story 004 (F4 + SessionReady — `ReconnectTracker` initialised in `on_session_ready`)
- Depends on: Story 005 (lobby disconnect — dual-signal disconnect detection must be in place; reconnect shares `OnConnected`/`OnDisconnected` Lightyear infrastructure)
- Depends on: Story 006 (game-over teardown — `ReconnectTracker.token_map` cleanup happens there; reconnect must not conflict with teardown)
- Depends on: round-state-machine epic (`RoundState`, `RoundPhase`, phase timer fields read by `build_game_snapshot`)
- Depends on: objective-system epic (`HiddenObjectives` resource read for `S2CObjectiveIdentities` re-send)
- Depends on: lightyear-protocol-verification epic — all 14 ADR-011 checklist items confirmed (S1-05 spike complete)
