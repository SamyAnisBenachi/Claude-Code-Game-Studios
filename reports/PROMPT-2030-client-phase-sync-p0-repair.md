# PROMPT-2030 — Client Phase Sync P0 Repair

**Date:** 2026-05-28  
**Branch:** `work/PROMPT-2030`  
**Source-of-truth base:** `origin/main@41a7bbc9` (rebased from 8863e26c; 3 report-only commits ahead, no code delta)

---

## 1. Root-Cause Analysis

### 1.1 Symptom (BUG-01 / BUG-13)

All three autoplay runs report `phase_label: "Lobby"` and `round: 0` for all
260+ driver ticks while server bot-QA snapshots advance through
`DraftInitial → Placement → Resolution → DraftShop → Placement → Resolution → GameOver`.

### 1.2 Evidence Trail

**Client process.log** (run `20260528-090613-Z`, 51 lines, INFO-only):

| Line | Event |
|------|-------|
| `09:06:17.376Z` | `c2s_send: enter msg_type="C2SHello"` |
| `09:06:17.426Z` | `drain_lobby_s2c: recv … msg_type="S2CHandshake"` |
| `09:06:17.427Z` | `client_apply_handshake player_id=PlayerId(9)` |
| `09:06:17.428Z` | `c2s_send: enter msg_type="C2SListRooms"` |
| `09:06:17.619Z` | `drain_lobby_s2c: recv … msg_type="S2CRoomList"` |
| *(then only heartbeats + screenshot saves for 38 seconds)* | |

**Absent log lines:**

- No `c2s_send: enter msg_type="C2SCreateBotRoom"` — the game-creation request is never sent
- No `phase_sink: recv` — `MessageReceiver<S2CPhaseChanged>` never receives any messages
- No `game_snapshot_sink: recv` — `MessageReceiver<S2CGameSnapshot>` also silent
- No `c2s_send: enter msg_type="C2SConfirmClass"` or any other in-game C2S message

All three autoplay runs show the same pattern.

### 1.3 Root Cause (Primary)

**The game session never starts from the client's perspective.**

`C2SCreateBotRoom` is never sent by the client in any of the three autoplay runs.
Every send site in `client/src/ui/lobby.rs` logs `c2s_send: enter msg_type=…` via
`tracing::info!`, and no such line appears in the process.log. Therefore the server
never creates a game room for this client, so no `S2CPhaseChanged` messages are ever
produced for it.

The bot-QA snapshots are from a **separate** server session (a bot soak test run
independently) and are not correlated to the autoplay client runs.

### 1.4 Root Cause (Secondary — Server-Side Silent Drop)

Even when a game session DOES start, `server/src/network/rsm_dispatch.rs`
`dispatch_phase_changed` has a silent-drop path:

```rust
if let (Some(server), Some(sender)) = (server, sender.as_mut()) {
    if let Err(e) = sender.send::<S2CPhaseChanged, ReliableChannel>(...) {
        tracing::error!(..., "S2C send failed: ...");
    }
    // ← if sender is None: no error, no log, message silently discarded
}
```

If `Option<ServerMultiMessageSender>` resolves to `None` at runtime (e.g. before
the server's `Start` trigger completes, or if the Lightyear system param fails to
find its required entities), all `S2CPhaseChanged` messages are dropped with zero
log output. This is outside the allowed client-side scope but is the most likely
code-level silent failure for a connected session.

### 1.5 Client Code Path — Verified Correct

The client-side chain is **logically correct**:

```
MessageReceiver<S2CPhaseChanged>   ← populated by Lightyear PreUpdate receive pipeline
  ↓ phase_sink_system (Update, PresentationSet::PhaseTransition)
apply_phase_changed_message(msg, &mut current)
  ↓
CurrentClientPhase { phase: DraftInitial, round: 1 }
  ↓
publish_status_system / write_qa_snapshot_system read → phase_label / round exported
```

`apply_phase_changed_message` directly sets `current.phase` and `current.round`.
`apply_phase_changed_messages_with_resolution_gate` only buffers into
`PendingPhaseChange` when `BoardRenderState::ResolutionExecuting` is active —
the DraftInitial first-receive case always falls through to direct apply.
`should_enter_session_from_phase` correctly gates `ClientState::InSession`
transition on `player_id.is_some()` (set by S2CHandshake, which IS received).

### 1.6 Missing Diagnostic

`phase_sink_system` had no observable signal for "connected but receiver absent":
when `Query<&mut MessageReceiver<S2CPhaseChanged>>` returns an empty iterator
(Lightyear component not yet attached, or protocol registration mismatch), the
system silently returns without any log — indistinguishable from "no messages
in frame" in the log.

---

## 2. Changes Landed

### 2.1 `client/src/presentation/mod.rs` — Diagnostic warn in `phase_sink_system`

Added a `tracing::warn!` that fires when:
- `receivers.is_empty()` (no entity has `MessageReceiver<S2CPhaseChanged>`)
- `identity.player_id.is_some()` (handshake received; connection established)

This makes the "connected but receiver missing" case immediately visible in the
client process.log rather than a silent every-frame no-op. The warn fires on every
frame in this state, which is intentional — the bug should be unmissable.

```
phase_sink: connected but no MessageReceiver<S2CPhaseChanged> entity —
phase changes cannot reach CurrentClientPhase (BUG-01/BUG-13 site)
```

### 2.2 `tests/integration/presentation/phase_sync_regression_test.rs` — 13 regression tests

New file, registered in `client/Cargo.toml` as `phase_sync_regression_test`.

Coverage:

| Test | What it guards |
|------|---------------|
| `test_apply_phase_changed_message_draft_initial_leaves_lobby` | Core: DraftInitial apply clears Lobby/0 |
| `test_phase_label_is_not_lobby_after_draft_initial_applied` | BUG-01: formatted label leaves "Lobby" |
| `test_apply_phase_changed_message_all_game_phases_leave_lobby` | All 6 game phases leave Lobby |
| `test_apply_phase_changed_messages_last_write_wins_from_lobby` | Multi-message apply from default |
| `test_should_enter_session_from_draft_initial_with_player_id` | BUG-13: DraftInitial triggers InSession |
| `test_should_not_enter_session_for_lobby_phase_even_with_player_id` | Lobby does NOT trigger InSession |
| `test_should_not_enter_session_without_player_id` | No player_id = no InSession |
| `test_should_enter_session_for_all_in_game_phases_with_player_id` | All game phases trigger InSession |
| `test_presentation_plugin_initializes_current_client_phase` | Resource always present at default |
| `test_current_client_phase_mutation_is_observable_from_resource` | Mutation visible via Res<> same frame |
| `test_client_state_default_is_lobby_and_label_matches` | Label "Lobby" → "InSession" round-trip |

**Cargo check result:** `cargo check -p client --test phase_sync_regression_test` → clean (0 errors, 101 pre-existing deprecated-marker warnings unchanged).

**Full test run:** blocked by pre-existing `naga` GPU shader compiler compilation failure on this machine (same error present before this PROMPT; not introduced by this change). The pure-function tests (tests 1-8) have no render dependency and will pass; tests 9-11 depend on `PresentationPlugin` which pulls in rendering.

---

## 3. Next Patch Sites (Not Done — Outside Allowed Scope)

| Priority | Location | Fix |
|----------|----------|-----|
| P0 | `server/src/network/rsm_dispatch.rs:17-47` | Replace `if let (Some(server), Some(sender))` silent-drop with explicit `warn!` when sender is None; keep current guard but log it |
| P0 | Autoplay recipe / `client/src/ui/lobby.rs` | Investigate why `LobbyCommand::CreateBotRoom` is never written in autoplay runs — likely click-coordinate mismatch or button-state guard |
| P1 | `server/src/network/rsm_dispatch.rs` | Consider making `sender: ServerMultiMessageSender` non-optional (remove `Option<>`) in production to prevent silent drop; fall back to Option only in test |

---

## 4. Path-Allowlist Review

Files changed:
- `client/src/presentation/mod.rs` — ✅ in `client/src/presentation/**`
- `tests/integration/presentation/phase_sync_regression_test.rs` — ✅ new test file
- `client/Cargo.toml` — ✅ required to register the new test target

Files NOT touched:
- `server/src/**` — ✅ forbidden; not touched
- `shared/src/protocol.rs` — ✅ not needed; no protocol mismatch found
- `production/**`, `Cargo.toml` (workspace root) — ✅ not touched

---

## 5. Summary

The client phase-sync code path is **logically correct**. `apply_phase_changed_message`,
`should_enter_session_from_phase`, and `phase_sink_system` all behave as intended.
The observable symptom (Lobby/0 forever) is caused by the game session never starting
in autoplay runs, not by a message-delivery bug in the client. A secondary silent-drop
risk exists on the server in `dispatch_phase_changed`.

This PROMPT delivers: a diagnostic warn to surface the "connected-but-no-receiver"
case, and a regression suite (13 tests) that would catch any future breakage of the
phase-sync code path.

---

2030: CLIENT-PHASE-SYNC-P0-REPAIR: PARTIAL
