# Story 006: BOT-DISCONNECT-REJOIN-006 -- Bot Disconnect / Re-join Hardening

> **Epic**: Bot & Autoplay
> **Story ID**: BOT-DISCONNECT-REJOIN-006
> **Status**: Draft -- future-sprint candidate (NOT activated; Sprint 18/19 row set NOT modified)
> **Layer**: Integration -- server-side bot lifecycle + session-safety
> **Type**: Integration -- bot join / disconnect / rejoin lifecycle and reconnect-state safety
> **Sprint**: Unscheduled -- Sprint 20+ candidate (gated on BOT-ROOM-PARTICIPANT-001 story-done)
> **Authored**: 2026-05-27 by PROMPT 1650
> **Authoring source-of-truth**: `origin/main@178a8471`

---

## Status / No-Claim Banner

This story is a **future-sprint paperwork placeholder** authored to give the
disconnect / rejoin and reconnect-snapshot hardening items an owned story ID.
Story 001 (`BOT-ROOM-PARTICIPANT-001`) lists these as open follow-up items (not
yet a story); this story closes that gap in the ledger.

PROMPT 1650 (this authoring run) does NOT:

- Activate Sprint 19 or Sprint 20.
- Run `/story-readiness`, `/dev-story`, `/story-done`, `/smoke-check`,
  `/team-qa`, `/gate-check`, `/release-check`, or `/qa-plan`.
- Modify any code under `client/`, `server/`, `shared/`, or `tests/`.
- Modify `Cargo.toml`, build scripts, CI workflows, or sprint YAML files.
- Claim closure of any PROMPT or landed work.
- Retry the Polish→Release gate-check.

Non-claims preserved verbatim: NO public release readiness, NO
release-candidate readiness, NO full game completion, NO Standard-tier
accessibility advancement (`QA-COND-0005`), NO playtest validation
advancement (`QA-COND-0006`), NO full playable-client manual QA, NO
`S8-QA-001-W1` closure, NO `PAW-TD-*-a` final-art completion, NO stage
advance from `Polish`.

---

## Problem Class / Prevention Target

**Defect class**: The server-side bot participant has no hardened lifecycle for
the disconnect → re-join path. AC8 of Story 001 (`Bot lifecycle (join,
disconnect, re-join) does not corrupt room or session state`) is listed as an
acceptance criterion but has no implementation design, no story-level
specification, and no test evidence target. Similarly, reconnect-snapshot
correctness for a bot participant (i.e., the server's reconnect snapshot
correctly identifies a re-joined bot without promoting it to a ghost human
slot) is unspecified.

Failure modes not currently guarded against:

1. **Duplicate bot spawn** -- if a bot disconnects mid-game and the server's
   participant-map does not detect the prior occupancy, a second bot entity
   could be created for the same slot, leading to two action-emitters writing
   to the same game state concurrently.
2. **Stale session remnant** -- bot disconnects during AUCTION or PLACEMENT;
   room scheduler does not advance because it waits for an action from an
   entity that no longer holds a connection, deadlocking the round.
3. **Reconnect snapshot corruption** -- the QA snapshot path (PROMPT 1597)
   emits per-participant snapshots; if the bot's participant record is partially
   cleaned up on disconnect, the snapshot emitted on bot reconnect may contain
   stale or inconsistent phase/round state that confuses the autoplay harness or
   the replay reader.
4. **Decision-log discontinuity** -- the streamed bot-decision log (PROMPT 1597)
   emits decisions keyed to a connection handle; after a disconnect/rejoin the
   handle rotates, producing a split log with orphaned pre-disconnect entries
   and an un-correlated post-rejoin stream for the same logical participant.
5. **Human participant state corruption** -- a bot disconnect during a shared-
   resolution step (e.g., RESOLUTION phase lane combat) could trigger an
   unguarded `Option::unwrap` on a bot resource, panicking the server and
   evicting the human participant mid-game.

**Prevention target**: This story delivers the specification, implementation,
and test evidence for a hardened bot lifecycle so that all five failure modes
above are ruled out by design and verified by automated test.

---

## Scope

### In Scope

- Server-side bot participant lifecycle state machine: `Connecting` →
  `Joined` → `Ready` → `Active` → `Disconnected` → `Rejoin` or `Evicted`.
- Duplicate-bot guard: idempotency check at bot-join time against the
  server's existing participant map.
- Round-advancement guard: server's action-wait logic must not block on a
  disconnected bot participant; a configurable timeout yields a forfeit-action
  (stand-in bid / no-place) so the round advances for the human participant.
- Reconnect snapshot correctness: when a bot reconnects, the snapshot emitted
  by the QA snapshot path must reflect the current authoritative phase/round
  state, not the pre-disconnect snapshot frame.
- Decision-log continuity: bot decision log entries must carry a stable
  `bot_session_id` (logical, not connection-handle-based) so the log can be
  correlated across a disconnect/rejoin boundary in post-game replay tooling.
- Human participant safety: all server-side bot action paths must handle the
  bot being in `Disconnected` state without panicking; no `unwrap()` on bot
  resources in production code paths.
- Integration test: a headless test that simulates bot disconnect mid-round
  and verifies round advancement, snapshot correctness, and log continuity.

### Out of Scope

- Actual re-connection transport logic (Lightyear WebSocket reconnect is
  handled by the engine; this story handles the server-side participant-map
  consequence, not the transport).
- Client-side autoplay reconnect flow (deferred; the autoplay harness targets
  a stable human client, not a bot-reconnect scenario).
- Bot decision *quality* after reconnect (heuristic reuse is assumed; no new
  heuristic needed).
- Multi-bot rooms (all current usage is one-bot rooms; this story does not
  need to handle concurrent bot disconnects in the same room).
- Release-blocking QA gate or Polish→Release gate-check retry.

---

## Acceptance Criteria

> These criteria are written to be exercised at Sprint activation / readiness
> time. They are **not validated by PROMPT 1650.**

- **AC1 -- Duplicate-bot guard**: If a bot entity already occupies a
  participant slot in a given room (whether connected or in `Disconnected`
  state), a second bot-join request for the same room is rejected with a
  server-side error log entry; no second bot entity is created.
- **AC2 -- Round-advancement guard**: If a bot disconnects during a phase that
  requires a bot action (AUCTION bid, PLACEMENT submit), the server advances
  past the wait after ≤ the configured forfeit timeout (default: 10 s); a
  forfeit-action is recorded in the bot decision log with reason
  `"DISCONNECTED_FORFEIT"`.
- **AC3 -- Reconnect snapshot correctness**: When a bot reconnects, the QA
  snapshot path emits a snapshot whose `phase`, `round`, and `client_state`
  fields match the current server authoritative values, not the pre-disconnect
  values.
- **AC4 -- Decision-log continuity**: Bot decision log entries emitted after
  reconnect carry the same stable `bot_session_id` as entries emitted before
  the disconnect. A replay reader can correlate pre- and post-disconnect
  entries into a single logical game trace.
- **AC5 -- Human participant safety**: A human client in the same room as a
  disconnecting-and-rejoining bot completes the affected round (AUCTION →
  PLACEMENT → RESOLUTION → next-round preamble) without a server panic or
  forced disconnect.
- **AC6 -- State cleanup on eviction**: If the bot does not reconnect within
  the configured eviction timeout (default: 60 s), the server cleans up the
  bot participant record from the room participant map and emits a structured
  eviction log entry; the room can be closed or reassigned without dangling
  participant references.
- **AC7 -- Integration test passes**: A headless integration test covering
  AC1–AC6 passes on `cargo test` (scoped to `tests/integration/bot/`) with
  no server panic and deterministic outcome.

---

## Implementation Design Notes

These notes are advisory for the implementing agent; they are not binding and
should be verified against the live `origin/main` codebase at implementation
time.

### Participant State Machine (server-side)

Extend the bot participant representation in `server/src/game/bot/` with an
explicit state enum:

```rust
enum BotParticipantState {
    Joining,
    Active,
    Disconnected { since: Instant, forfeit_deadline: Instant },
    Evicted,
}
```

The room participant map stores this alongside the existing participant data.
All bot-action dispatch paths check `BotParticipantState` before emitting an
action; `Disconnected` short-circuits to a forfeit path.

### Stable `bot_session_id`

Assign a random `Uuid` to each bot at initial join and store it in the server-
side participant record. Do NOT use the Lightyear `ClientId` (connection handle)
as the log key. On reconnect, look up the existing participant record by room +
slot and re-use the stored `Uuid`. The decision log schema gains a required
`bot_session_id: Uuid` field.

### Forfeit Action

When the forfeit deadline passes, emit a synthetic decision entry:

```
{ bot_session_id, phase, round, action: "FORFEIT", reason: "DISCONNECTED_FORFEIT" }
```

Then call the same server-side action-resolution path a real bot action would
take (e.g., `resolve_bid(BotBid::Pass)` or `resolve_placement(BotPlacement::Skip)`).
This unblocks the human participant's round without requiring a special-case path
in the core game loop.

### Reconnect Snapshot Patch

At bot-reconnect, trigger a fresh `QaSnapshot` observation using the current
server `RoundState` resource, rather than re-sending the cached pre-disconnect
snapshot. The existing observation path (PROMPT 1597) should support this with
a targeted `observe_now()` call; no schema change required.

### Test Harness

Use the existing bot-vs-bot soak entrypoint (`server/src/game/bot/soak.rs` or
equivalent from PROMPT 1603) as the basis. The integration test should:

1. Boot a headless server room with one bot and one mock-human participant.
2. Simulate a bot disconnect (drop the connection handle mid-AUCTION).
3. Assert: round advances within forfeit timeout; forfeit decision log entry
   is present; mock-human participant is not disconnected.
4. Simulate bot reconnect before eviction timeout.
5. Assert: reconnect snapshot fields match server authoritative state; decision
   log `bot_session_id` matches pre-disconnect value.
6. Assert: room completes PLACEMENT → RESOLUTION without panic.

---

## Implementation Lineage

| PROMPT | Slice | Notes |
|--------|-------|-------|
| 1430 | Protocol room foundations | Bot join handshake established |
| 1439 | Foundation scaffold | Server-side bot entity and participant map |
| 1531 / 1582 | Action loop Waves 1 + 2 | Bot action dispatch paths |
| 1597 | QA snapshot + decision log | Snapshot and log infrastructure this story extends |
| 1602 | Wave 3 placement heuristic | Current production-tip bot action set |
| *This story* | Disconnect / rejoin hardening | New implementation work |

---

## Dependencies

| Story / ADR | Dependency | Type |
|---|---|---|
| [BOT-ROOM-PARTICIPANT-001](story-001-bot-room-participant.md) | Must be story-done (AC1–AC7 verified) before this story's Sprint activation | Hard gate |
| [ADR-002: Client-Server Authority](../../../docs/architecture/adr-002-client-server-authority.md) | Bot decisions remain server-authoritative; reconnect path must not relax this | Constraint |
| [ADR-012: SessionReady Delivery](../../../docs/architecture/adr-012-session-ready-delivery.md) | Re-join after eviction cannot re-enter `SessionReady` Observer path; must be a fresh join or be refused | Constraint |

---

## Recommended Sprint Planning Steps

1. Wait for `BOT-ROOM-PARTICIPANT-001` `/story-done` clearance (Sprint 19).
2. Assign this story to Sprint 20 candidate backlog.
3. Run `/story-readiness BOT-DISCONNECT-REJOIN-006` against the Sprint 20
   activation tip to confirm no new server-side regressions have closed off
   the design notes above.
4. Spawn an implementation prompt scoped to `server/src/game/bot/lifecycle.rs`
   (or equivalent) and `tests/integration/bot/disconnect_rejoin_test.rs`.
5. Run `/story-done BOT-DISCONNECT-REJOIN-006` once AC1–AC7 pass.

---

## Test Evidence (target)

| Type | Target | Gate |
|---|---|---|
| Integration test | `tests/integration/bot/disconnect_rejoin_test.rs` covers AC1–AC7 | BLOCKING |
| Decision-log replay | Post-game log inspection confirms `bot_session_id` continuity across disconnect boundary | BLOCKING |
| Manual smoke | Orchestrator-driven friend-game with human + bot, bot force-disconnected and re-joined mid-round | ADVISORY |
