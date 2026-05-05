# Game Session System

> **Status**: In Review — MAJOR REVISION NEEDED (R2, see design-review 2026-04-29), revised inline — pending R3
> **Author**: User + Agents
> **Last Updated**: 2026-04-29
> **Implements Pillar**: Simple surface · No idle spectating
## Overview

The Game Session System manages the full lifecycle of a multiplayer game session in Lanes and Lies — from room creation through class selection and lobby readiness, to the handoff that starts the first round. It serves two roles simultaneously: as infrastructure, it creates and maintains the Lightyear server-side session room, tracks connected players, validates player count and mode configuration against the expected composition, and signals the Round State Machine when all LOBBY entry conditions are satisfied; as player experience, it is the lobby — the brief window where players select their class, see their opponent's class choice (a public revelation), and confirm readiness before the economic clock starts ticking. If the expected player count is not reached within `lobby_timeout_seconds`, the session is cancelled and no game begins. Once DRAFT_INITIAL starts, the Game Session System becomes passive infrastructure: it holds the session's mode and class configuration that other systems read throughout the game, but all active phase control passes to the Round State Machine.

## Player Fantasy

The Game Session System itself is invisible to the player — they never think "I'm interacting with session management." What they feel is something sharper: the instant their opponent's class appears on screen, the information war has begun.

Class selection is public and simultaneous. You pick Xelor and your opponent immediately recalibrates — *they'll accumulate reserve mana, play slow, save for a game-ending Garde-Temps*. They pick Sacrier and you recalibrate — *BODYGUARD threats, self-damage synergies, careful about Punition on auction rounds*. Neither player has bid on a card or placed a unit. Both have already started playing.

The lobby is brief by design — it should feel like the moment two players sit down across a table and lock eyes, not a waiting screen. The "No idle spectating" pillar applies here too: class reveal is the first piece of live information in the game, and the best players start reading it immediately.

## Detailed Rules

### Core Rules

**The Game Session System owns:**
- `session_id: Uuid` — server-generated unique identifier
- `mode: GameMode` — `OneVOne | TwoVTwo | ThreeVThree | OneVOneVOne | TwoVTwoVTwo`
- `slots: Vec<SessionSlot>` — ordered player slots (see Rule 4)
- `class_selections: Map<PlayerId, ClassId>` — finalized class per player
- `team_assignments: Map<PlayerId, TeamId>` — derived from slot index at creation
- `lobby_deadline: f32` — server clock time at which lobby timeout fires if unfilled
- `placement_timer_multiplier_requests: Map<PlayerId, PlacementTimerMultiplier>` — lobby/session accessibility requests for the PLACEMENT timer multiplier before `SessionReady`
- `placement_timer_multiplier_effective: PlacementTimerMultiplier` — neutral effective room/session value, computed from requests and frozen into `SessionConfig` at `SessionReady`

The Game Session System does **not** own: round number, phase state, gold/mana, card hands, board state, or any in-game economy. All of those belong to the RSM and downstream systems. After firing `SessionReady`, the GSS becomes a read-only configuration store.

---

**Rule 1 — Room creation:**
A room is created when a player issues `C2SCreateRoom { mode: GameMode }`. The server assigns a `session_id`, generates a 6-character alphanumeric room code, initializes slots based on mode, sets `lobby_deadline = now + lobby_timeout_seconds`, and returns `S2CRoomCreated { room_code, mode, slots }` to the creator. The creator is automatically assigned to slot 0 (Team A, position 0).

| Mode | Total slots | Team A slots | Team B slots | Team C slots |
|---|---|---|---|---|
| `OneVOne` | 2 | 1 (index 0) | 1 (index 1) | — |
| `TwoVTwo` | 4 | 2 (indices 0–1) | 2 (indices 2–3) | — |
| `ThreeVThree` | 6 | 3 (indices 0–2) | 3 (indices 3–5) | — |
| `OneVOneVOne` | 3 | 1 (index 0) | 1 (index 1) | 1 (index 2) |
| `TwoVTwoVTwo` | 6 | 2 (indices 0–1) | 2 (indices 2–3) | 2 (indices 4–5) |

**Rule 2 — Joining a room:**
A player joins by sending `C2SJoinRoom { room_code: String, requested_slot: u8 }`. If the slot is unoccupied and the session is in `LOBBY_WAITING`, the server:
1. Assigns the player to the requested slot.
2. Returns `S2CJoinAck { mode, slots }` to the joining player — the full current slot state, so the joiner can render which slots are already taken.
3. Broadcasts `S2CSlotUpdated { slots: Vec<SessionSlot> }` — the **full** current slot vector (not a delta) — to all **other** connected players already in the session, so they see the newly filled slot immediately.

The joiner receives only `S2CJoinAck` (which already contains full slot state). Other players receive only `S2CSlotUpdated`. No player receives both.

If the slot is occupied, the server returns `S2CJoinRejected { reason: SlotOccupied }`. If the session has already started, the server returns `S2CJoinRejected { reason: SessionInProgress }`.

**Rule 3 — Slot data structure:**
```
SessionSlot {
    index: u8,
    team: TeamId,            // Fixed at creation; derived from index
    player: Option<PlayerId>,
    class: Option<ClassId>,  // None until confirmed (Rule 7)
}
```

**Rule 4 — Team assignment is slot-derived:**
A player's team is fixed by their slot index at the time they join, and never changes. Players choose their team by choosing their slot. Teammates on the same team are determined before the game starts and are visible in the lobby UI.

**Rule 5 — Lobby timeout:**
The `lobby_timeout_seconds` countdown starts the moment the room is created, not when the first joiner arrives. If the session has not reached `LOBBY_READY` before the deadline, the server cancels the session, broadcasts `S2CSessionCancelled { reason: LobbyTimeout }` to all connected clients, and destroys the session resource. No `S2CGameOver` is emitted — the session never started. No player is awarded a win.

**Rule 6 — Class selection opens immediately:**
A player may begin selecting (browsing) a class as soon as they occupy a slot. `C2SSelectClass { class_id }` updates the player's preview selection and is reversible at any time before confirmation. The selected-but-unconfirmed class is visible only to that player's own client — not broadcast to others.

**Rule 7 — Class lock and deferred simultaneous reveal:**
`C2SConfirmClass { class_id: ClassId }` locks the player's class choice. The server:
1. Writes `SessionSlot[player].class = Some(class_id)` AND `class_selections[player_id] = class_id` in the same exclusive-access system, sequentially, such that no other system can observe a partially-written state. (Bevy ECS does not provide transactional rollback — "same system" is the sequencing guarantee, not database atomicity.)
2. Sends `S2CClassLocked { class_id }` **point-to-point to the locking player only** — confirms the lock and shows the player their own class. No other player receives this message.
3. Evaluates whether all slots now have `class: Some(_)`.
4. If all slots are locked: broadcasts `S2CClassesRevealed { player_class_map: Map<PlayerId, ClassId> }` to **all players simultaneously**.

No player sees their opponent's class until all players in the session have locked. A confirmed class cannot be changed. Duplicate classes across players in the same session are permitted.

**Locked-waiting state (one locked, one not):** After a player receives `S2CClassLocked`, their client enters the locked-waiting state. The client renders the player's own confirmed class selection and displays a visible animated indicator showing that the opponent is actively in the class selection process (e.g., a pulsing cursor or "choosing..." animation). The indicator signals presence and active engagement — not class identity. The client derives this state purely from slot occupancy (`opponent slot is occupied`) and the absence of `S2CClassesRevealed` — no new server message is needed. The opponent's browsing activity (`C2SSelectClass`) is explicitly NOT broadcast (Rule 6) and must not be inferred from message timing. The client must not render a lobby countdown timer during the locked-waiting state — the lobby deadline is a server-side cancellation mechanism, not a player-visible pressure display.

This produces the Player Fantasy's "simultaneous reveal" moment: both players commit blind, then both classes appear at once. It mirrors the auction pillar — a sealed bid followed by a simultaneous outcome.

**Rule 8 — Session ready condition:**
`SessionReady` fires when all of the following are simultaneously true:
1. All slots are occupied (`player != None` for every slot)
2. All players have confirmed their class (`class_selections` has an entry for every `PlayerId`)
3. `lobby_deadline` has not expired

This is an internal server-side event. The RSM listens for it to trigger LOBBY → DRAFT_INITIAL. The GSS does not transition game phases itself.

**Rule 9 — Disconnection during LOBBY (MVP behavior: immediate cancel):**
Any disconnection detected during LOBBY (before DRAFT_INITIAL begins) immediately cancels the session. The server broadcasts `S2CSessionCancelled { reason: PlayerDisconnected }` to all remaining connected clients and destroys the session resource. No `S2CGameOver` is emitted — the session never officially started. No player is awarded a win.

**Dual-signal disconnect detection during LOBBY.** The GSS uses two independent signals:

1. **Lightyear `OnDisconnected` (primary):** When Lightyear fires this event, the GSS cancels the session immediately in the same server tick.
2. **`C2SHeartbeat` gap (fallback):** WASM/WebSocket connections can enter half-open TCP states where `OnDisconnected` does not fire for 2–7 minutes. The GSS therefore also tracks heartbeat gaps for each connected player. If a player's last `C2SHeartbeat` was received more than `lobby_heartbeat_timeout_seconds` ago (default: 15s, loaded from `GameConfig`), the GSS treats this as a disconnect and cancels the session — identical behavior to an `OnDisconnected` event.

The session cancels on the **first** of these two signals. The heartbeat gap threshold (`lobby_heartbeat_timeout_seconds`) is distinct from the RSM's `disconnect_grace_seconds` (30s): the LOBBY threshold is intentionally shorter because LOBBY has no grace window and no in-progress game to protect. After `SessionReady` fires, heartbeat tracking passes to the RSM, which owns `disconnect_grace_seconds`.

**Heartbeat tracker ownership:** The GSS initializes and owns the heartbeat tracker from the time a player occupies a slot until `SessionReady` fires. On `SessionReady`, the RSM takes over. The two trackers must not overlap.

**MVP scope:** Reconnect-with-grace-window during LOBBY is deferred to post-hackathon. A functional reconnect mechanism requires a reconnect credential (session token) issued at handshake time — this is not in scope and not designed. See Open Questions. In-game disconnect handling (after DRAFT_INITIAL) is governed by the RSM's `disconnect_grace_seconds = 30` rule and is unaffected by this decision.

**Rule 10 — Late join is not permitted:**
Once `SessionReady` has fired and DRAFT_INITIAL has begun, the session is closed. Any client connecting to an in-progress session receives `S2CJoinRejected { reason: SessionInProgress }` and may only observe as a spectator (per RSM edge case handling).

**Rule 11 — Session configuration handoff and canonical data sources:**
When `SessionReady` fires, the GSS writes the following into a read-only `SessionConfig` resource that persists for the game's lifetime:
- `mode: GameMode`
- `player_count: u8`
- `team_map: Map<PlayerId, TeamId>` — derived from `SessionSlot.team` for each occupied slot; RSM uses for team-forfeiture logic
- `class_map: Map<PlayerId, ClassId>` — derived from `SessionSlot.class` for each occupied slot; Class System uses for class-specific rule activation
- `placement_timer_multiplier_effective: PlacementTimerMultiplier` — neutral effective PLACEMENT room/session timer multiplier from Rule 14; RSM reads it when starting PLACEMENT timers

The GSS also initializes the `ServerRng` resource as part of the `SessionReady` handoff, satisfying the RNG GDD's contract: "Initialized once at game-session start."

**Canonical class source:** `SessionSlot.class` is the authoritative class store. `class_map` in `SessionConfig` is derived from it once at `SessionReady` time and is never independently mutated. The `class_selections: Map<PlayerId, ClassId>` field in the session root is a derived index populated in the same system as `SessionSlot.class` when `C2SConfirmClass` is processed (see Rule 7) — it exists only to serve the F4 predicate without iterating slots. `SessionSlot.class` is always the tie-breaker if these diverge.

**SessionConfig None invariant:** When building `class_map` at `SessionReady` time, every occupied slot must have `SessionSlot.class = Some(_)`. If any occupied slot has `SessionSlot.class = None`, this is a programmer invariant violation — the code must `panic!` with an explicit message (e.g., `"SessionConfig build: occupied slot {index} has no confirmed class — invariant violated"`). This path is unreachable if F4 is correctly implemented (F4 requires all classes confirmed before `is_ready = true`), but the defensive check must exist. Do not silently proceed with a `None` entry in `class_map`.

**Implementation note (belongs in ADR):** Whether `SessionReady` uses a Bevy `Observer` (same-frame delivery) or buffered `Events<T>` (next-frame delivery) determines when the RSM can read `SessionConfig`. This ordering guarantee is an implementation concern, not a GDD design rule — the behavior requirement is that `SessionConfig` and `ServerRng` are available to all systems that handle `SessionReady`. The specific Bevy mechanism is documented in the session system ADR.

**Rule 12 — Server restart during LOBBY:**
If the server process terminates while a session is in `LOBBY_WAITING` or `LOBBY_READY`, the session is destroyed. There is no reconnect-and-resume. All players must create a new session after the server restarts. This is consistent with MVP disconnect behavior (Rule 9): lobby state is ephemeral and not persisted.

**Rule 13 — Session membership: one active session per player:**
A `PlayerId` may occupy a slot in at most one active session at a time.

- **`C2SCreateRoom` when already in a session:**
  - If the player owns a session currently in `LOBBY_WAITING` (i.e., they created it and it has not yet started): the server returns `S2CRoomCreated` with the **existing** room code and current slot state. No new session is created. This is the idempotent create path — designed for the case where the creator sends `C2SCreateRoom` twice before the first response arrives (slow-network retry).
  - If the player is in any other session state (`LOBBY_READY`, `GAME_ACTIVE`, or a session they joined but did not create): the server returns `S2CCreateRoomRejected { reason: AlreadyInSession }`. No new session is created.

- **`C2SJoinRoom` when already in a session:** The server returns `S2CJoinRejected { reason: AlreadyInSession }`. The player's existing slot is undisturbed. (See GSS-15 for the same-session self-join case.)

`S2CCreateRoomRejected` is a new message type distinct from `S2CJoinRejected` — the contexts are different and should not share an error type.

**Rule 14 — PLACEMENT timer multiplier negotiation:**
During LOBBY, before `SessionReady`, each occupied player slot may request a
multiplayer-safe PLACEMENT timer multiplier via the Network Protocol message
`C2SSetPlacementTimerMultiplier`.

Allowed multiplayer Standard-tier values are:

- `1x`
- `1.5x`
- `2x`
- `3x`

The effective room/session value is neutral and server-authoritative:

`placement_timer_multiplier_effective = min(max(requests), 3x)`

Players who do not send a request contribute `1x`. Requests below `1x`,
including `0.5x`, are not multiplayer-safe and do not lower the effective
multiplayer value. Requests above `3x` cannot raise the effective value above
`3x`.

The GSS broadcasts `S2CSessionSettingsUpdated {
placement_timer_multiplier_effective }` whenever the effective value changes and
unicasts the current value to a joining player as part of lobby state recovery.
This message is a neutral room/session setting display. It must not identify
which player requested the extension.

When `SessionReady` fires, the current effective value is written into
`SessionConfig.placement_timer_multiplier_effective` and frozen for the active
match. Later Settings changes may update local preferences for the next session
but must not mutate the active session config.

---

### States and Transitions

| From | To | Trigger | Guard |
|---|---|---|---|
| — | `LOBBY_WAITING` | `C2SCreateRoom` received | Mode is valid |
| `LOBBY_WAITING` | `LOBBY_WAITING` | Player joins, selects, or confirms class | Slot unoccupied; session not yet ready |
| `LOBBY_WAITING` | `LOBBY_READY` | All slots filled AND all classes confirmed | `lobby_deadline` not expired |
| `LOBBY_WAITING` | `LOBBY_CANCELLED` | `lobby_deadline` expires | At least one slot empty or class unconfirmed |
| `LOBBY_WAITING` | `LOBBY_CANCELLED` | Any player disconnects (MVP: immediate cancel) | — |
| `LOBBY_READY` | `GAME_ACTIVE` | GSS fires `SessionReady`; RSM enters DRAFT_INITIAL; GSS observes RSM transition | All LOBBY conditions met; `ServerRng` init succeeds |
| `LOBBY_WAITING` | `LOBBY_CANCELLED` | `ServerRng` initialization fails when `SessionReady` would fire | All other LOBBY conditions met |
| `LOBBY_READY` | `LOBBY_CANCELLED` | Any player disconnects (MVP: immediate cancel) | — |
| `GAME_ACTIVE` | `GAME_OVER` | RSM emits `S2CGameOver` | GSS observes for session teardown only |
| `LOBBY_CANCELLED` | — | Session destroyed | Terminal |
| `GAME_OVER` | — | Session destroyed after teardown | Terminal |

> **Note on state naming:** GSS states (`LOBBY_WAITING`, `LOBBY_READY`, `GAME_ACTIVE`) are the session lifecycle view. The RSM's single `LOBBY` phase corresponds to the GSS `LOBBY_WAITING → LOBBY_READY` window. When the GSS reaches `LOBBY_READY` and fires `SessionReady`, the RSM's LOBBY → DRAFT_INITIAL guard is satisfied. See `round-state-machine.md` Rule 1 for the RSM-side LOBBY definition.
>
> **MVP note on reconnect:** The `RECONNECTING` protocol state (defined in the Network Protocol GDD) does not apply during LOBBY for the hackathon MVP. Disconnect during LOBBY = immediate session cancellation (Rule 9).

---

### Interactions with Other Systems

| System | Direction | What this system does |
|---|---|---|
| **Round State Machine** | GSS → RSM | Fires `SessionReady` when all slots filled and all classes confirmed. Publishes `SessionConfig` (mode, player_count, team_map, class_map, `placement_timer_multiplier_effective`) that RSM reads for player-count checks, team-forfeit logic, and PLACEMENT timer duration. |
| **Round State Machine** | RSM → GSS | GSS observes `S2CGameOver` to trigger session teardown (destroy `SessionConfig`, remove `ServerRng`, close audit log). |
| **Game Config** | GSS reads | Reads `lobby_timeout_seconds` at room creation. Reads `disconnect_grace_seconds` for pre-game disconnect grace (same value as RSM — no separate field). |
| **Server-side RNG** | GSS → RNG | Initializes `ServerRng` resource immediately before `SessionReady` fires. Destroys `ServerRng` resource on `GAME_OVER` teardown. |
| **Class System** *(GDD not yet written)* | GSS publishes | Writes `class_map: Map<PlayerId, ClassId>` into `SessionConfig` at session start. Class System reads this to activate class-specific rule sets. |
| **Network Protocol / Lightyear** | GSS → clients | During LOBBY: sends `S2CRoomCreated` (unicast to creator), `S2CJoinAck` (unicast to joiner), `S2CSlotUpdated` (broadcast on any slot change), `S2CClassLocked` (unicast to locking player), `S2CClassesRevealed` (broadcast when all locked), `S2CSessionSettingsUpdated` (neutral timer setting), `S2CSessionCancelled` (broadcast on cancel). All on reliable channel. After DRAFT_INITIAL, phase-state broadcasts transfer to RSM. |
| **Network Protocol / Lightyear** | Lightyear → GSS | GSS listens to Lightyear `OnDisconnected` events to trigger immediate session cancellation (Rule 9, MVP behavior). |

## Formulas

### F1 — Expected Player Count

The expected player count formula is defined as:

`expected_count(mode) = lookup(mode)`

**Variables:**

| Variable | Symbol | Type | Range | Description |
|---|---|---|---|---|
| Game mode | `mode` | `GameMode` | `{OneVOne, TwoVTwo, ThreeVThree, OneVOneVOne, TwoVTwoVTwo}` | The mode set by the room creator at session creation |
| Expected count | `expected_count` | `u8` | 2–6 | Number of player slots that must be filled before `SessionReady` can fire |

| Mode | `expected_count` |
|---|---|
| `OneVOne` | 2 |
| `OneVOneVOne` | 3 |
| `TwoVTwo` | 4 |
| `ThreeVThree` | 6 |
| `TwoVTwoVTwo` | 6 |

**Output Range:** 2 to 6.
**Example:** Mode = `TwoVTwo` → `expected_count = 4`. All 4 slots must be filled and all 4 classes confirmed before `SessionReady` fires.

---

### F2 — Team Assignment from Slot Index

The team assignment formula is defined as:

`team(slot_index, mode) = lookup(slot_index, team_slot_table[mode])`

**Variables:**

| Variable | Symbol | Type | Range | Description |
|---|---|---|---|---|
| Slot index | `i` | `u8` | 0 to `expected_count - 1` | Zero-based position in the slot list |
| Mode | `mode` | `GameMode` | see F1 | Determines total slot count and team boundaries |
| Result | `team` | `TeamId` | `{A, B, C}` | Team assigned to this slot |

Team boundaries are derived from the slot table in Rule 1. For 2-team modes: slots in the first half → Team A; slots in the second half → Team B. For 3-team modes: slots are divided into thirds.

| Mode | Team A slots | Team B slots | Team C slots |
|---|---|---|---|
| `OneVOne` | 0 | 1 | — |
| `TwoVTwo` | 0, 1 | 2, 3 | — |
| `ThreeVThree` | 0, 1, 2 | 3, 4, 5 | — |
| `OneVOneVOne` | 0 | 1 | 2 |
| `TwoVTwoVTwo` | 0, 1 | 2, 3 | 4, 5 |

**Output Range:** One of `{A, B, C}`. Assignment is fixed at session creation and never changes.
**Example:** Mode = `TwoVTwo`, slot_index = 2 → Team B.

---

### F3 — Lobby Deadline

The lobby deadline formula is defined as:

`lobby_deadline = session_created_at + lobby_timeout_seconds`

**Variables:**

| Variable | Symbol | Type | Range | Description |
|---|---|---|---|---|
| Session creation time | `session_created_at` | `f64` | 0.0 – ∞ | Server clock time (seconds, f64) when `C2SCreateRoom` is processed. **Must be `f64`, not `f32`:** at ~6 days uptime `f32` precision degrades below tick granularity (16.67ms at 60 Hz), and at ~97 days loses sub-second precision entirely, producing incorrect deadline evaluation. |
| Lobby timeout | `lobby_timeout_seconds` | `u32` | 30–300 | From `GameConfig`; default 90. Cast to `f64` before addition. |
| Result | `lobby_deadline` | `f64` | `session_created_at + 30.0` to `session_created_at + 300.0` | Server clock time at which session cancellation triggers |

**Output Range:** `session_created_at + 30.0` to `session_created_at + 300.0` (within safe range of `lobby_timeout_seconds`).
**Evaluation:** The GSS checks `server_clock_now >= lobby_deadline` every server tick. On the first tick where this is true and `LOBBY_READY` has not been reached, the session transitions to `LOBBY_CANCELLED`. The deadline evaluation system does NOT run once the GSS state is `LOBBY_READY` — cancellation by timeout is only possible from `LOBBY_WAITING`.

---

### F4 — Session Ready Predicate

The session ready predicate is defined as:

`is_ready = all_slots_filled(slots) AND all_classes_confirmed(class_selections, slots) AND (server_clock_now <= lobby_deadline)`

**Variables:**

| Variable | Symbol | Type | Range | Description |
|---|---|---|---|---|
| Slot occupancy | `all_slots_filled` | `bool` | `{true, false}` | True when `player != None` for every entry in `slots` |
| Class confirmation | `all_classes_confirmed` | `bool` | `{true, false}` | True when `class_selections` contains an entry for every `PlayerId` currently in `slots` |
| Time check | `not_expired` | `bool` | `{true, false}` | True when `server_clock_now <= lobby_deadline` (non-strict: the session proceeds if the last class is confirmed on the exact deadline tick) |
| Result | `is_ready` | `bool` | `{true, false}` | True only when all three conditions hold simultaneously |

**Output Range:** Boolean.
**Example:** 4-player 2v2 game. Players in slots 0, 1, 2 have confirmed classes. Slot 3 is occupied but class is unconfirmed. `all_classes_confirmed = false` → `is_ready = false`. Player in slot 3 confirms → `all_classes_confirmed = true` → `is_ready = true` → `SessionReady` fires.

---

### F5 — Effective PLACEMENT Timer Multiplier

The effective multiplayer PLACEMENT timer multiplier is defined as:

`effective = min(max(multiplayer_safe_requests), 3x)`

**Variables:**

| Variable | Symbol | Type | Range | Description |
|---|---|---|---|---|
| Player request | `request[player]` | `PlacementTimerMultiplier` | `{1x, 1.5x, 2x, 3x}` | Multiplayer-safe request submitted before `SessionReady`; absent request contributes `1x` |
| Effective multiplier | `effective` | `PlacementTimerMultiplier` | `{1x, 1.5x, 2x, 3x}` | Highest multiplayer-safe request across players, capped at 3x |

**Output Range:** `1x` to `3x`.
**Example:** Player A requests `1.5x`; Player B requests `3x`. The server
broadcasts neutral room/session setting `3x` and freezes `3x` into
`SessionConfig` at `SessionReady`.

`0.5x` is not a multiplayer-safe request. It must not be exposed in multiplayer
Standard-tier Settings and cannot reduce the effective multiplayer value below
`1x`.

## Edge Cases

**If an invalid room code is entered:** Server returns `S2CJoinRejected { reason: RoomNotFound }`. Client shows "Room not found" and lets the player re-enter.

**If a room code collision occurs during generation:** Before confirming a new `session_id`, the server checks for any active session with the same 6-character code. If a collision is detected, it regenerates until unique. The character set is uppercase alphanumeric minus `{0, O, 1, I, L}` = 31 characters; code space = 31^6 ≈ 887 million codes. At typical scale this branch is extremely rare but must be handled.

**If all slots are occupied when a player attempts to join:** Server returns `S2CJoinRejected { reason: SessionFull }`. Distinct from `SlotOccupied` — `SessionFull` means all slots are taken; `SlotOccupied` means one specific slot is taken but others may be free.

**If `requested_slot` is out of range for the session mode:** For example, requesting slot 4 in a `OneVOne` session (which has only slots 0–1). Server validates `requested_slot < expected_count(mode)` and returns `S2CJoinRejected { reason: InvalidSlot }`. Other slots may still be available, so this is not a `SessionFull` condition.

**If the last class is confirmed on the same server tick that `lobby_deadline` equals `server_clock_now`:** The session proceeds. F4 uses `<=` in its deadline check, so `server_clock_now == lobby_deadline` evaluates to `is_ready = true` (see F4 formula). `SessionReady` fires; `S2CSessionCancelled` is NOT sent. No system ordering trick is required — the formula handles this by definition. The deadline cancellation system gates on `LOBBY_READY` not having been reached: once `is_ready = true` is evaluated and the GSS transitions to `LOBBY_READY`, the deadline cancellation path is blocked.

**If a player disconnects during LOBBY (before DRAFT_INITIAL):** Session is immediately cancelled (Rule 9, MVP behavior). Server broadcasts `S2CSessionCancelled { reason: PlayerDisconnected }`. This applies at all sub-states of LOBBY: `LOBBY_WAITING`, `LOBBY_READY`. There is no grace window during LOBBY for the hackathon MVP.

**If `SessionReady` fires and a player disconnects on a strictly later tick (RSM has entered `GAME_ACTIVE`):** The GSS defers to the RSM. This is an in-game disconnection governed by RSM Rule 13 (`disconnect_grace_seconds = 30`).

**If all players disconnect simultaneously during LOBBY:** Session is immediately cancelled. `S2CSessionCancelled { reason: PlayerDisconnected }` is broadcast to any clients still connected at the moment of cancellation. No winner declared.

**If `C2SConfirmClass` is received without any prior `C2SSelectClass` from that player:** `C2SConfirmClass` is self-contained — it carries `class_id` in its payload. No prior `C2SSelectClass` is a server-side precondition. The preview step (`C2SSelectClass`) is a client-side convenience; the server does not require it before accepting a confirmation. Implementers must not add a "must have previewed" server guard — this is explicitly not in the specification.

**If `C2SConfirmClass` is sent again by a player who has already confirmed:** If the new `class_id` matches the already-confirmed class: server silently discards the duplicate (idempotent). No `S2CClassLocked` is re-sent; no `S2CClassesRevealed` is triggered again. If the new `class_id` is different (attempted re-lock after confirmation): server returns `S2CConfirmClassRejected { reason: ClassAlreadyConfirmed }`. Rule 7 stands — confirmed class cannot be changed. (Note: a dedicated `S2CConfirmClassRejected` message is cleaner than reusing `S2CJoinRejected` for a class-locking error.)

**If all occupied slots have confirmed classes but not all slots are filled:** Example: `TwoVTwo` session; slots 0 and 1 are occupied and both have confirmed classes; slots 2 and 3 are empty. `all_classes_confirmed` must be evaluated as `∀ PlayerId p in slots where p != None: class_selections.contains_key(p)` — not as `class_selections.len() == expected_count`. Using the count-based evaluation would silently produce a false `is_ready = false` now but could produce `is_ready = true` prematurely if the predicate is later refactored to compare against occupied count rather than expected count.

**If a player attempts to rejoin a session after it was cancelled due to their disconnect:** The session no longer exists. Server returns `S2CJoinRejected { reason: RoomNotFound }`. The player must create or join a new session. (Post-hackathon: reconnect-with-grace requires a session token issued at handshake time — see Open Questions.)

**If `C2SCreateRoom` carries an invalid or unrecognized `GameMode` variant:** Server validates the mode is within `{OneVOne, TwoVTwo, ThreeVThree, OneVOneVOne, TwoVTwoVTwo}` before processing. If invalid, no room is created and no `S2CRoomCreated` is sent.

**If `ServerRng` initialization fails when `SessionReady` would fire:** Rule 11 requires `ServerRng` to be initialized immediately before `SessionReady`. If initialization fails, the session transitions to `LOBBY_CANCELLED` (internal reason). `SessionReady` is NOT fired. No game may begin without a valid `ServerRng`.

**If a player sends `C2SJoinRoom` for a session they already occupy a slot in:** Server detects the `PlayerId` already exists in `slots` and returns `S2CJoinRejected { reason: AlreadyInSession }`. Players cannot occupy two slots.

**Room code character set — ambiguous characters:** Room codes use uppercase alphanumeric characters only. Excluded to prevent optical ambiguity: `0` (zero), `O` (letter O), `1` (one), `I` (letter I), `L` (letter L). Characters `S`/`5` and `Z`/`2` are included — accepted ambiguity risk at typical display sizes. Server normalizes all lowercase input to uppercase before lookup.

## Dependencies

### Upstream Dependencies

| System | Type | Interface | Notes |
|---|---|---|---|
| **Game Config** | Hard | GSS reads `lobby_timeout_seconds` at room creation; reads `lobby_heartbeat_timeout_seconds` for heartbeat-based disconnect detection during LOBBY | These are the only two GameConfig values the GSS consumes during LOBBY. `disconnect_grace_seconds` is owned by the RSM and applies only after `DRAFT_INITIAL` begins — it does not apply to LOBBY. |
| **Network Protocol / Lightyear** | Hard | GSS uses Lightyear's `OnDisconnected` events (primary) and `C2SHeartbeat` gaps (fallback) for LOBBY disconnect tracking; uses Lightyear's reliable channel for all lobby broadcasts | GSS assumes Lightyear is initialized and accepting connections before any `C2SCreateRoom` can be processed. See Rule 9 for the dual-signal disconnect model. The specific Lightyear 0.26 API for connection events must be verified before implementation. |

---

### Downstream Dependents

| System | Type | Interface | Notes |
|---|---|---|---|
| **Round State Machine** | Hard | GSS fires `SessionReady` (internal Bevy Event) when LOBBY conditions are met; RSM's LOBBY→DRAFT_INITIAL transition is gated on this event. GSS publishes `SessionConfig` (mode, player_count, team_map, class_map, `placement_timer_multiplier_effective`) that RSM reads throughout the game. RSM observes `S2CGameOver` to trigger GSS teardown. | Bidirectional. See `round-state-machine.md` Rule 1 and ADR-023. |
| **Server-side RNG** | Hard | GSS initializes the `ServerRng` resource immediately before firing `SessionReady`. GSS destroys `ServerRng` during `GAME_OVER` teardown. The RNG GDD states "initialized once at game-session start" — this GDD is that boundary. | The RNG GDD should note this GDD as the lifecycle owner. |
| **Class System** *(GDD not yet written)* | Hard | GSS writes `class_map: Map<PlayerId, ClassId>` into `SessionConfig`. The Class System reads this at game start to activate class-specific rule sets for each player. | Class System GDD must list Game Session System as an upstream dependency when authored. |
| **Auction System** *(GDD not yet written)* | Soft | Auction System reads `player_count` from `SessionConfig` for multi-player auction card count decisions (RSM Open Question 3). | Indirect — consumed via `SessionConfig`, not a direct GSS interface. |
| **All in-game feature systems** | Soft | Systems that need to know player count, team assignments, or class per player read `SessionConfig`. The GSS is the only authoritative source of this data at game start. | These systems should list Game Session System as an upstream dependency when their GDDs are authored. |

---

### Cross-Reference Updates Required

The following upstream GDDs should add Game Session System to their downstream dependency tables (flagged for the next `/consistency-check` pass — not modifying approved GDDs mid-session):

- **`game-config.md`** — Should add: "Game Session System | Downstream (hard) | Reads `lobby_timeout_seconds` and `lobby_heartbeat_timeout_seconds`"
- **`server-rng.md`** — Should note: "Game Session System is the session lifecycle owner — it initializes and destroys the `ServerRng` resource"

## Tuning Knobs

| Knob | Default | Safe Range | Too Low | Too High | Interacts With |
|---|---|---|---|---|---|
| `lobby_timeout_seconds` | 90 | 60–300 | Players can't coordinate fast enough to fill the lobby before cancellation; frequent false cancellations for friend groups on slow connections or using external voice chat to synchronize | Players wait a very long time if one player drops or is delayed; lobby feels abandoned before it officially cancels | `lobby_heartbeat_timeout_seconds` — if a player ghosts (no heartbeat) before the deadline, the session cancels early via heartbeat timeout rather than waiting for the deadline |
| `lobby_heartbeat_timeout_seconds` | 15 | 10–30 | Legitimate WASM/browser latency spikes (tab switch, antivirus, OS interrupts) could cause 3–6s gaps — values below 10s risk false forfeits in normal browser conditions | Detection of a ghosted player is delayed; in the worst case a ghost slot is held for up to `lobby_timeout_seconds` before the deadline fires | `lobby_timeout_seconds` — the heartbeat timeout is the fast-path detection; the deadline is the slow-path fallback. Set `lobby_heartbeat_timeout_seconds` significantly below `lobby_timeout_seconds` so a ghost is detected well before the deadline fires. Default 15s ≪ 90s. |

**Cross-referenced constants (owned by Game Config — not tunable here):**

| Constant | Value | Source |
|---|---|---|
| `disconnect_grace_seconds` | 30s | `game-config.md` (shared with Round State Machine — applies after DRAFT_INITIAL, not in LOBBY) |

**Status:** `lobby_timeout_seconds` has been added to `game-config.md` (field `pub lobby_timeout_seconds: u32`, default 90) and registered in `design/registry/entities.yaml`. **`lobby_heartbeat_timeout_seconds` must be added to `game-config.md` and `design/registry/entities.yaml` before implementation** — flagged for next config authoring session (Open Question 8).

## Visual/Audio Requirements

[To be designed]

## UI Requirements

[To be designed]

## Acceptance Criteria

| # | Criterion | Type |
|---|---|---|
| GSS-1 | GIVEN a connected client, WHEN they send `C2SCreateRoom { mode: OneVOne }`, THEN the server responds with `S2CRoomCreated` containing a `room_code` that is exactly 6 characters, uppercase alphanumeric only, and contains none of the excluded characters `0`, `O`, `1`, `I`, `L`. | BLOCKING |
| GSS-2 | GIVEN a connected client, WHEN they send `C2SCreateRoom { mode: OneVOne }`, THEN `S2CRoomCreated.slots` shows the sender's `PlayerId` in slot index 0 with `team: A`, and slot index 1 with `player: None`. | BLOCKING |
| GSS-3 | GIVEN `GameConfig.lobby_timeout_seconds = 90` and server clock time T, WHEN a client sends `C2SCreateRoom`, THEN the server records `lobby_deadline = T + 90`. The deadline is not affected by subsequent join events, class selections, or confirmations. | BLOCKING |
| GSS-4 | GIVEN a session created with `mode: OneVOne`, WHEN the server initializes slots, THEN the slot list has exactly 2 entries: slot 0 (Team A) and slot 1 (Team B). `SessionReady` cannot fire until both slots are occupied and both classes are confirmed. | BLOCKING |
| GSS-5 | GIVEN a session created with `mode: TwoVTwo`, WHEN the server initializes slots, THEN the slot list has exactly 4 entries. Slots 0–1 are Team A; slots 2–3 are Team B. `SessionReady` cannot fire until all 4 slots are occupied and all 4 classes are confirmed. | BLOCKING |
| GSS-6 | GIVEN a session created with `mode: ThreeVThree`, WHEN the server initializes slots, THEN the slot list has exactly 6 entries. Slots 0–2 are Team A; slots 3–5 are Team B. | BLOCKING |
| GSS-7 | GIVEN a session created with `mode: OneVOneVOne`, WHEN the server initializes slots, THEN the slot list has exactly 3 entries. Slot 0 is Team A; slot 1 is Team B; slot 2 is Team C. | BLOCKING |
| GSS-8 | GIVEN a session created with `mode: TwoVTwoVTwo`, WHEN the server initializes slots, THEN the slot list has exactly 6 entries. Slots 0–1 are Team A; slots 2–3 are Team B; slots 4–5 are Team C. | BLOCKING |
| GSS-9 | GIVEN a `TwoVTwo` session, WHEN a player joins slot 2, THEN their `team` field is `B` in both the server's slot state and in the `S2CJoinAck.slots` payload. The team field does not change for the lifetime of the session. | BLOCKING |
| GSS-10 | GIVEN a `TwoVTwo` session with slot 0 occupied and slots 1–3 empty, WHEN a second player sends `C2SJoinRoom { room_code, requested_slot: 1 }`, THEN the server responds with `S2CJoinAck { mode, slots }` where all 4 slot entries are present — slot 0 shows the first player's `PlayerId`, slot 1 shows the joining player's `PlayerId`, and slots 2–3 show `player: None`. | BLOCKING |
| GSS-11 | GIVEN a `OneVOne` session with slot 1 already occupied by Player B, WHEN Player C sends `C2SJoinRoom { room_code, requested_slot: 1 }`, THEN the server returns `S2CJoinRejected { reason: SlotOccupied }`. Player C is not added to the session. Slot 0 remains available. | BLOCKING |
| GSS-12 | GIVEN a `OneVOne` session with slots 0 and 1 both occupied, WHEN a third player sends `C2SJoinRoom { room_code, requested_slot: 0 }`, THEN the server returns `S2CJoinRejected { reason: SessionFull }`. | BLOCKING |
| GSS-13 | GIVEN no active session exists with code `"XXXXXX"`, WHEN a client sends `C2SJoinRoom { room_code: "XXXXXX", requested_slot: 0 }`, THEN the server returns `S2CJoinRejected { reason: RoomNotFound }`. No session is created. | BLOCKING |
| GSS-14 | GIVEN a `OneVOne` session (slots 0 and 1 only), WHEN a client sends `C2SJoinRoom { room_code, requested_slot: 4 }`, THEN the server returns `S2CJoinRejected { reason: InvalidSlot }`. The response is not `SessionFull` — valid slots 0 and 1 may still be available. | BLOCKING |
| GSS-15 | GIVEN Player A is already in slot 0 of an active session, WHEN Player A sends `C2SJoinRoom { room_code, requested_slot: 1 }` for the same session, THEN the server returns `S2CJoinRejected { reason: AlreadyInSession }`. Player A's slot 0 is not disturbed. | BLOCKING |
| GSS-16 | GIVEN Player A and Player B are both in a `OneVOne` session lobby, WHEN Player A sends `C2SSelectClass { class_id: Xelor }`, THEN the server does not enqueue any S2C message targeting Player B's connection in that tick. Player A's own client may reflect the preview state; no outbound message is added to any other player's send queue. *(Unit test: verify server message dispatch log contains zero entries for Player B after processing C2SSelectClass. Integration test with live Lightyear session confirms no frame arrives at Player B.)* | BLOCKING |
| GSS-17a | GIVEN Player A and Player B are both in a `OneVOne` session lobby and Player B has NOT yet confirmed, WHEN Player A sends `C2SConfirmClass { class_id: Xelor }`, THEN: (a) Player A receives `S2CClassLocked { class_id: Xelor }` point-to-point; (b) the server enqueues no S2C message for Player B's connection in that tick; (c) `S2CClassesRevealed` is NOT broadcast. After the system runs: `SessionSlot[A].class = Some(Xelor)` AND `class_selections[A] = Xelor` — both fields hold correct values (verify as invariant assertion in unit test; the "same system" scheduling guarantee is verified via code review and documented in the session ADR). | BLOCKING |
| GSS-17b | GIVEN Player A has already confirmed Xelor, WHEN Player B confirms Iop (the final lock in a OneVOne session), THEN the server broadcasts `S2CClassesRevealed { A: Xelor, B: Iop }` to all players simultaneously. No player sees the other's class before this broadcast. | BLOCKING |
| GSS-18 | GIVEN Player A has already confirmed `class_id: Xelor`, WHEN Player A sends `C2SConfirmClass { class_id: Sacrier }` (a different class), THEN the server returns `S2CConfirmClassRejected { reason: ClassAlreadyConfirmed }`. `SessionSlot[A].class` and `class_selections[A]` still hold `Xelor`. No broadcast is sent. | BLOCKING |
| GSS-19 | GIVEN Player A has confirmed `class_id: Xelor`, WHEN Player B sends `C2SConfirmClass { class_id: Xelor }` (duplicate class), THEN the server accepts the confirmation, writes `SessionSlot[B].class = Some(Xelor)` and `class_selections[B] = Xelor`, and broadcasts `S2CClassesRevealed { A: Xelor, B: Xelor }` (since both are now locked). No rejection is issued for the duplicate. | BLOCKING |
| GSS-20a | GIVEN a `OneVOne` session where slot 1 is occupied and Player B has confirmed their class and slot 0 is empty, WHEN Player A joins slot 0 and sends `C2SConfirmClass`, THEN `SessionReady` fires exactly once. `SessionReady` must not fire a second time if any subsequent message is processed in the same session. | BLOCKING |
| GSS-20b | GIVEN the same precondition as GSS-20a, WHEN Player A joins slot 0 but does NOT send `C2SConfirmClass`, THEN `SessionReady` does not fire. `is_ready` evaluates false because `all_classes_confirmed = false`. | BLOCKING |
| GSS-21 | GIVEN a `TwoVTwo` session where slots 0 and 1 are occupied and both have confirmed classes, but slots 2 and 3 are empty, WHEN the server evaluates `is_ready`, THEN `is_ready = false`. The fact that `class_selections.len() == occupied_slot_count` (2 == 2) is not sufficient to fire `SessionReady`. All 4 slots must be occupied and all 4 classes confirmed. | BLOCKING |
| GSS-22 | GIVEN a `OneVOne` session with only slot 0 occupied and `lobby_timeout_seconds = 90`, WHEN 90 seconds elapse from room creation without `SessionReady` firing, THEN the server broadcasts `S2CSessionCancelled { reason: LobbyTimeout }` to all connected clients. No `S2CGameOver` message is sent. No player is assigned a win. The session resource is destroyed. | BLOCKING |
| GSS-23 | GIVEN a `OneVOne` session where Player A is in slot 0 and Player B is in slot 1, and Player B's last heartbeat was received more than `lobby_heartbeat_timeout_seconds` ago, WHEN the GSS evaluates disconnect trackers, THEN the server immediately broadcasts `S2CSessionCancelled { reason: PlayerDisconnected }` to Player A's connection and destroys the session. The cancellation reason is `PlayerDisconnected`, not `LobbyTimeout`, regardless of how close the `lobby_deadline` was. The session is destroyed without waiting for the deadline. | BLOCKING |
| GSS-24 | GIVEN a `OneVOne` session where both players have confirmed classes and all slots are filled, and `server_clock_now == lobby_deadline` on the evaluation tick, WHEN the server evaluates F4, THEN `is_ready = true` (F4 uses `<=` — see formula) and `SessionReady` fires. `S2CSessionCancelled` is NOT sent. *Note: the `<=` in F4 ensures this case is correct by formula; no system-ordering trick is required. Testable with a controlled clock that sets server_clock_now = lobby_deadline.* | ADVISORY |
| GSS-25 | GIVEN Player A is in a `OneVOne` lobby and Player A disconnects at any point during LOBBY (before DRAFT_INITIAL), WHEN Lightyear fires `OnDisconnected` for Player A's connection, THEN the server immediately broadcasts `S2CSessionCancelled { reason: PlayerDisconnected }` to all connected clients and destroys the session resource. No `S2CGameOver` is emitted. | BLOCKING |
| GSS-26 | GIVEN `SessionReady` has fired and the RSM has transitioned to `DRAFT_INITIAL`, WHEN any client sends `C2SJoinRoom` with the session's room code, THEN the server returns `S2CJoinRejected { reason: SessionInProgress }`. No slot is assigned. | BLOCKING |
| GSS-27 | GIVEN a `TwoVTwoVTwo` session where all 6 slots are filled and 5 of 6 players have confirmed classes, WHEN the 6th player confirms their class, THEN the server broadcasts `S2CClassesRevealed { player_class_map }` to all 6 players simultaneously. The reveal is not sent to the 5 earlier lockers before the 6th lock — all 6 receive it at once in the same broadcast cycle. | BLOCKING |
| GSS-28 | GIVEN `lobby_timeout_seconds = 60` (minimum safe value) and a session created at server time T=0, WHEN `server_clock_now = T + 60.0` (f64) and the session has not reached `LOBBY_READY`, THEN the server broadcasts `S2CSessionCancelled { reason: LobbyTimeout }`. Tests the minimum F3 boundary. | BLOCKING |
| GSS-29 | GIVEN all lobby conditions are met (all slots filled, all classes confirmed, deadline not expired), WHEN `ServerRng` initialization fails at the moment `SessionReady` would fire, THEN `SessionReady` is not fired. The session transitions to `LOBBY_CANCELLED`. No `S2CGameOver` is emitted. *Note: requires dependency injection on the RNG init path — raise with lead programmer before implementation.* | BLOCKING |
| GSS-30 | GIVEN all lobby conditions are met and `ServerRng` initializes successfully, WHEN `SessionReady` fires, THEN the `SessionConfig` resource exists in the ECS world containing `mode`, `player_count`, `team_map` (every `PlayerId` → `TeamId` derived from `SessionSlot.team`), `class_map` (every `PlayerId` → confirmed `ClassId` derived from `SessionSlot.class`), and `placement_timer_multiplier_effective` (the frozen neutral room/session value from Rule 14). The behavioral requirement: `SessionConfig` and `ServerRng` are available when the RSM handles `SessionReady`. The specific tick/frame ordering depends on Observer vs Events<T> — see session ADR. | BLOCKING |
| GSS-31 | GIVEN `lobby_timeout_seconds = 90` and a session created at T=0 with no players joining until T=30, WHEN the deadline is evaluated, THEN it fires at T=90, not T=120. The countdown begins at room creation, not at first slot occupancy. | BLOCKING |
| GSS-32 | GIVEN Player A occupies a slot and has not sent any `C2SSelectClass` message, WHEN Player A sends `C2SConfirmClass { class_id: Sacrier }`, THEN the server accepts the confirmation, writes `SessionSlot[A].class = Some(Sacrier)` and `class_selections[A] = Sacrier` in the same system, and sends `S2CClassLocked { class_id: Sacrier }` to Player A. No prior `C2SSelectClass` is required. | BLOCKING |
| GSS-33 | GIVEN Player A has already confirmed `class_id: Xelor`, WHEN Player A sends `C2SConfirmClass { class_id: Xelor }` again (same class), THEN the server silently discards the duplicate. No `S2CClassLocked` is re-sent. No `S2CClassesRevealed` is triggered. `SessionSlot[A].class` and `class_selections[A]` remain `Xelor`. | BLOCKING |
| GSS-34 | GIVEN an active session with code `"ABCDEF"` already exists, WHEN the server generates a new room code that collides with `"ABCDEF"`, THEN the server regenerates until it produces a unique code. The new session's `room_code` in `S2CRoomCreated` is never `"ABCDEF"`. *(Unit test: mock the code generator to return `"ABCDEF"` on first call and a unique code on second; verify the returned room_code is the unique one.)* | BLOCKING |
| GSS-35 | GIVEN an active session with code `"ABCDEF"`, WHEN a client sends `C2SJoinRoom { room_code: "abcdef", requested_slot: 0 }`, THEN the server normalizes `"abcdef"` to `"ABCDEF"` and locates the session correctly. The join is not rejected as `RoomNotFound`. | BLOCKING |
| GSS-36 | GIVEN a connected client, WHEN they send `C2SCreateRoom` with a `mode` value not in `{OneVOne, TwoVTwo, ThreeVThree, OneVOneVOne, TwoVTwoVTwo}`, THEN no room is created, no `S2CRoomCreated` is sent, and the server responds with `S2CCreateRoomRejected { reason: InvalidMode }`. The client connection is not closed. | ADVISORY |
| GSS-37 | GIVEN slot 0 is occupied by Player A, WHEN Player B joins slot 1 successfully, THEN Player A's connection receives `S2CSlotUpdated { slots: Vec<SessionSlot> }` containing the full current slot vector (both slots populated). Player B receives `S2CJoinAck` (not `S2CSlotUpdated`). Player A does NOT receive a second copy of `S2CJoinAck`. | BLOCKING |
| GSS-38 | GIVEN Player A is already in slot 0 of session `"ABCDEF"` (which is in `LOBBY_WAITING`), WHEN Player A sends `C2SCreateRoom { mode: OneVOne }` again (retry), THEN the server returns `S2CRoomCreated` with room_code `"ABCDEF"` and the current slot state. No new session is created. Player A's existing slot 0 is undisturbed. | BLOCKING |
| GSS-39 | GIVEN Player A is already in slot 0 of session `"ABCDEF"` (which is in `LOBBY_READY`), WHEN Player A sends `C2SCreateRoom { mode: OneVOne }`, THEN the server returns `S2CCreateRoomRejected { reason: AlreadyInSession }`. No new session is created. | BLOCKING |
| GSS-40 | GIVEN Player A is already in slot 2 of session `"XYZABC"` (a session they joined, not created), WHEN Player A sends `C2SJoinRoom { room_code: "QRSTUV", requested_slot: 0 }` for a different session, THEN the server returns `S2CJoinRejected { reason: AlreadyInSession }`. Player A's existing slot in `"XYZABC"` is undisturbed. | BLOCKING |
| GSS-41 | GIVEN Player A is in slot 0 of a `OneVOne` session and Player B is in slot 1, and Player B has not sent `C2SHeartbeat` for longer than `lobby_heartbeat_timeout_seconds`, WHEN the GSS heartbeat tracker evaluates the gap, THEN `S2CSessionCancelled { reason: PlayerDisconnected }` is broadcast to Player A's connection and the session is destroyed. This fires even if Lightyear `OnDisconnected` has not yet fired for Player B. | BLOCKING |
| GSS-42 | GIVEN Player A requests `1.5x` and Player B requests `3x` via `C2SSetPlacementTimerMultiplier` before `SessionReady`, WHEN the GSS recomputes the room/session timer setting, THEN `placement_timer_multiplier_effective = 3x` and `S2CSessionSettingsUpdated { placement_timer_multiplier_effective: 3x }` is sent without identifying which player requested it. | BLOCKING |
| GSS-43 | GIVEN no player has requested a PLACEMENT timer multiplier before `SessionReady`, WHEN the GSS builds `SessionConfig`, THEN `placement_timer_multiplier_effective = 1x`. | BLOCKING |
| GSS-44 | GIVEN `SessionReady` has fired with `placement_timer_multiplier_effective = 2x`, WHEN any player changes their local Settings timer preference during DRAFT_INITIAL, DRAFT_SHOP, PLACEMENT, RESOLUTION, or GAME_OVER, THEN the active `SessionConfig.placement_timer_multiplier_effective` remains `2x`; the change can only affect a future lobby/session. | BLOCKING |

## Open Questions

1. **Room code sharing UX** — How does the room creator communicate the code to Player B? The GDD assumes out-of-band sharing (Discord, voice chat). Should the game include a copy-to-clipboard button or a shareable URL in the lobby UI? Deferred to the Lobby UI design phase; does not affect server-side behavior.

2. **Simultaneous session membership** — RESOLVED. Rule 13 specifies: one active session per player. `C2SCreateRoom` is idempotent for the creator's own pending LOBBY_WAITING session; otherwise rejected with `S2CCreateRoomRejected { reason: AlreadyInSession }`. `C2SJoinRoom` rejected with `S2CJoinRejected { reason: AlreadyInSession }`. ACs: GSS-38, GSS-39, GSS-40.

3. **Spectator mode definition** — The RSM references a "spectator view" for late-joining clients (post-DRAFT_INITIAL). Full specification belongs to the Network Protocol GDD (RSM Open Question 4). Out of scope here.

4. **`lobby_timeout_seconds` in GameConfig** — RESOLVED. Field `pub lobby_timeout_seconds: u32` (default 90) added to `game-config.md` and registered in `design/registry/entities.yaml`.

5. **Mode implementation order** — All 5 modes are specified and in scope. Recommended implementation order: `OneVOne` first (simplest slot config), then `TwoVTwo`, then `ThreeVThree`. `OneVOneVOne` and `TwoVTwoVTwo` are fully specified but may be addressed last. Server code must be mode-driven (not hardcoded for 1v1). ⚠️ C-W2 (spawn-range counter per-player vs per-team for 2v2) must be resolved before `TwoVTwo` implementation begins.

6. **Reconnect-with-grace during LOBBY** — DEFERRED post-hackathon. MVP behavior is disconnect = forfeit (Rule 9). A functional reconnect mechanism requires: (a) a session token issued in `S2CHandshake` and stored by the client, (b) a `C2SReconnect { session_id, slot_index, token }` message, (c) server-side session-to-token mapping. None of this is designed. Track as a post-hackathon feature when network protocol v2 is designed.

7. ~~**ADR needed: Observer vs. buffered Events<T> for SessionReady**~~ — **Resolved by ADR-012.** `SessionReady` uses a Bevy Observer trigger; `SessionConfig`, including the ADR-023 timer multiplier field, must be inserted before the trigger.

8. **`lobby_heartbeat_timeout_seconds` in GameConfig** — Must be added to `game-config.md` (field `pub lobby_heartbeat_timeout_seconds: u32`, default 15) and registered in `design/registry/entities.yaml` before the Game Session System can be implemented. Owner: next `game-config.md` authoring session.
