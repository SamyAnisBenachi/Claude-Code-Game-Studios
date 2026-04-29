# Round State Machine

> **Status**: Approved
> **Author**: User + Agents
> **Last Updated**: 2026-04-29
> **Implements Pillar**: No idle spectating · Auction as signature · Simple surface

## Overview

The Round State Machine is the server-side phase orchestrator that drives all game activity in Lanes and Lies. It owns the current phase, the round counter, and both timing mechanisms that bound the PLACEMENT phase. The game begins with a one-time **DRAFT_INITIAL** phase (pre-game card selection with 5 starting gold). Thereafter, each round advances through three phases in fixed order: **DRAFT** (personal shop + optional auction), **PLACEMENT** (10-second simultaneous secret card selection), and **RESOLUTION** (simultaneous reveal and effect resolution). The RSM fires all phase-boundary events that other systems depend on — mana reset and gold income at DRAFT start, economy snapshot at RESOLUTION end, shop refresh at DRAFT start. It detects auction rounds by formula and signals the Auction System within DRAFT accordingly. PLACEMENT ends when all players submit or the 10-second timer expires, whichever comes first. After RESOLUTION, it checks the loss condition; if the threshold is reached, the machine transitions to the terminal **GAME_OVER** state rather than starting a new round. All phase transitions are broadcast to clients via Lightyear S2C messages; the client holds a read-only phase mirror for UI display only. The RSM does not execute game logic — it sequences and signals; the systems it triggers own their own logic.

## Player Fantasy

The Round State Machine is not something the player interacts with — it is the tempo they live inside. Its presence is felt most sharply in one moment: the hand hovering over submit during PLACEMENT. The player has read the gold counts, watched the auction tells, counted the cards the opponent could hold — and now they must commit, blind, on a clock. No take-back. No reactive adjustment after the reveal begins. The RSM's hard phase boundaries make every placement a decision the player owns completely. When the reveal vindicates the read, it feels earned. When it punishes them, they know exactly which assumption was wrong.

## Detailed Design

### Core Rules

**The RSM owns:**
- `phase: RoundPhase` — LOBBY · DRAFT_INITIAL · DRAFT_AUCTION · DRAFT_SHOP · PLACEMENT · RESOLUTION · GAME_OVER
- `round_number: u32` — starts at 1, increments at each new DRAFT
- `placement_timer` and `draft_shop_timer` — server-side countdown in seconds
- `submissions_received: Set<PlayerId>` — which players have submitted placement this round
- `disconnect_trackers: Map<PlayerId, f32>` — seconds since last heartbeat per player

The RSM does **not** own: card state, gold/mana balances, auction bid state, combat logic, or objective HP. It signals those systems; they own their own state.

---

**Rule 1 — Phase sequence:**
```
LOBBY
  └─► DRAFT_INITIAL (round 1, once only)
        └─► PLACEMENT (round 1)
              └─► RESOLUTION (round 1)
                    ├─► DRAFT_AUCTION (round 3, 6, 9…) ─► DRAFT_SHOP ─► PLACEMENT ─► RESOLUTION ─► …
                    ├─► DRAFT_SHOP (round 2, 4, 5, 7…) ─► PLACEMENT ─► RESOLUTION ─► …
                    └─► GAME_OVER (terminal)
```
DRAFT_INITIAL is the round 1 draft phase. It transitions directly to PLACEMENT — there is no DRAFT_SHOP for round 1.

**Rule 2 — Round counter increment:**
`round_number` is set to 1 when DRAFT_INITIAL begins. It increments by 1 at the moment the RSM transitions out of RESOLUTION into DRAFT_AUCTION or DRAFT_SHOP. Economy events at the new DRAFT entry (Rule 3) use the already-incremented value.

**Rule 3 — Economy events at DRAFT entry:**
On entry into DRAFT_INITIAL, DRAFT_AUCTION, or DRAFT_SHOP (from RESOLUTION or game start), the RSM fires these for all players in order, before accepting player input:
1. `apply_mana_ramp(player)` → `current_mana = min(round_number, mana_cap)`
2. `apply_gold_income(player)` → `gold += baseline + interest` (interest from prior RESOLUTION snapshot; for round 1: starting gold of 5 is granted instead)

**Rule 4 — Interest snapshot:**
At RESOLUTION end — after all combat, kill rewards, and objective rewards have fired — the RSM instructs the Economy System to snapshot each player's gold. The Economy System holds this snapshot and applies it in Rule 3 at the next DRAFT entry.

**Rule 5 — Shop refresh timing:**
On entry into any DRAFT phase (DRAFT_INITIAL, DRAFT_AUCTION, or DRAFT_SHOP), the RSM fires `refresh_shop(player)` for all players immediately after economy events. For DRAFT_INITIAL this populates the initial 9-card selection; for DRAFT_AUCTION and DRAFT_SHOP it refreshes the personal shop. During DRAFT_AUCTION the shop is **visible but not interactable** — players see their upcoming cards while the auction runs (a deliberate design choice: players can make informed bid decisions knowing what their shop holds). Shop purchases and manual refresh are accepted only during DRAFT_SHOP.

**Rule 6 — Auction round detection:**
```
is_auction_round(R) = (R mod 3 == 0)
```
Evaluated after `round_number` increments on RESOLUTION exit. Round 3 → true. Round 4 → false. Round 6 → true. Round 9 → true.

**Rule 7 — DRAFT_AUCTION behavior:**
On entry, the RSM sends `StartAuction(round_number)` to the Auction System. The Auction System owns the 20-second timer and all bid state. `AuctionSettled` is a Bevy buffered Event; the RSM system must be scheduled after the Auction System so the event is readable in the same or next frame. The RSM waits for `AuctionSettled` (winner or no-bid timeout), then transitions to DRAFT_SHOP.

**Rule 8 — DRAFT_SHOP timer:**
`draft_shop_timer` starts on DRAFT_SHOP entry. Default: `draft_shop_timer_seconds = 30`. When timer reaches 0, RSM transitions to PLACEMENT. If all players signal ready before 0, RSM transitions immediately. Ready signal is available from t=0. Players may retract their ready signal at any time until the all-ready condition fires — retraction returns them to the normal shopping state with no penalty.

**Rule 9 — PLACEMENT timer:**
`placement_timer` starts on PLACEMENT entry. Default: `placement_timer_seconds = 10`. When timer reaches 0, PLACEMENT ends. Players who did not submit are treated as playing zero cards (existing board state unchanged, no refund). If all players submit early, RSM transitions to RESOLUTION immediately.

**Rule 10 — RESOLUTION:**
On entry, RSM signals the Combat Resolution System to execute all six global sub-steps. No player input is accepted. `ResolutionComplete` is a Bevy buffered Event; the RSM system must be scheduled after Combat Resolution in the app schedule so the event is readable in the same or next frame.

**Safety timeout:** If `ResolutionComplete` is not received within `resolution_max_duration_seconds` (default: 60s), the RSM broadcasts `OnResolutionEnd` then transitions to GAME_OVER as a Draw. This timeout must never fire in normal play — its presence protects sessions from a Combat Resolution crash or infinite keyword chain.

**`OnResolutionEnd`:** This event is broadcast on all RESOLUTION exits, including transitions to GAME_OVER. Board/Lane System listens to it for unit cleanup; it must fire regardless of whether a win condition follows. The interest snapshot (Rule 4) fires before `OnResolutionEnd`.

**Rule 11 — GAME_OVER detection:**
After RESOLUTION completes and after the interest snapshot (Rule 4) is taken, the RSM evaluates: for each player, if `real_objectives_destroyed(player) >= 2`. If any player meets the condition, transition to GAME_OVER. If multiple players meet the condition simultaneously (mutual destruction in the same RESOLUTION), the result is a **Draw** — all qualifying players are declared losers; no winner is announced.

**Rule 12 — DRAFT_INITIAL termination:**
Ends when all players submit (early exit) OR `draft_initial_timer_seconds` expires. Non-submitting players forfeit their starting gold (use-it-or-lose-it). RSM then transitions to PLACEMENT with round_number = 1.

**Rule 13 — Disconnection:**
The RSM updates `disconnect_trackers` using Lightyear's `OnDisconnected` and `OnConnected` connection events (not a custom heartbeat message). If time since last `OnConnected` event > `disconnect_grace_seconds` (default: **30s**), RSM immediately transitions to GAME_OVER, declaring that player the loser. In team modes: that player's team loses. If reconnected within the grace period, game continues normally.

**Browser note:** 30s is intentional for a WASM/browser target. OS interrupts, antivirus scans, Windows Update prompts, and tab switches routinely cause 3–6s Lightyear connection gaps with no player action; a 5s window would produce false forfeits. 30s is hard to abuse in a 10–15 minute game while still catching genuine disconnections.

**Disconnection during RESOLUTION:** The RSM defers the GAME_OVER transition until RESOLUTION exits naturally. The current combat sub-step completes, `OnResolutionEnd` fires, the interest snapshot fires (if applicable), and only then is GAME_OVER set. This preserves a clean RESOLUTION→GAME_OVER transition and prevents Board/Lane state from leaking.

**Rule 14 — Phase broadcast:**
Every state transition broadcasts `S2CPhaseChanged` to all connected clients after the new state is entered and all entry actions have fired. Sent on the Lightyear **reliable** channel — phase changes must not be dropped. Payload: `{ phase, round_number, timer_duration_secs }`. For DRAFT_AUCTION, `timer_duration_secs = 0` (the Auction System drives its own countdown; clients must not render an RSM-owned timer for DRAFT_AUCTION). Clients hold a read-only `ClientPhaseView` resource for UI only and have no authority to trigger transitions.

**GAME_OVER message:** In addition to `S2CPhaseChanged(GAME_OVER)`, the RSM also broadcasts a separate `S2CGameOver` message on the reliable channel. Payload: `{ loser: PlayerId, round: u32, reason: GameOverReason }`. The `GameOverReason` enum is defined here (server-side type, rendered by HUD):

```rust
pub enum GameOverReason {
    ObjectivesDestroyed,  // Normal win condition
    Disconnection,        // A player exceeded disconnect_grace_seconds
    Draw,                 // Mutual destruction or mutual disconnection same tick
}

**Rule 15 — Valid player actions per state:**

| State | Accepted player actions |
|---|---|
| LOBBY | Connect; set name/class |
| DRAFT_INITIAL | Purchase cards (up to starting gold); signal ready |
| DRAFT_AUCTION | Place auction bids; view (not interact with) personal shop |
| DRAFT_SHOP | Purchase cards from shop; manual refresh (1g); signal ready; retract ready signal |
| PLACEMENT | Submit card selection for this round |
| RESOLUTION | None (read-only) |
| GAME_OVER | None |

---

### States and Transitions

| From | To | Trigger | Guard Condition |
|---|---|---|---|
| LOBBY | DRAFT_INITIAL | All expected players connected | Player count matches mode config |
| DRAFT_INITIAL | PLACEMENT | All players submit, OR `draft_initial_timer` expires | ≥1 player still connected |
| DRAFT_INITIAL | GAME_OVER | Player disconnects > `disconnect_grace_seconds` | — |
| PLACEMENT | RESOLUTION | All players submit, OR `placement_timer` reaches 0 | — |
| PLACEMENT | GAME_OVER | Player disconnects > `disconnect_grace_seconds` | — |
| RESOLUTION | DRAFT_AUCTION | Resolution completes; no loss condition; `is_auction_round(round_number)` after increment = true | round_number incremented |
| RESOLUTION | DRAFT_SHOP | Resolution completes; no loss condition; `is_auction_round(round_number)` after increment = false | round_number incremented |
| RESOLUTION | GAME_OVER | Resolution completes; `real_objectives_destroyed(any_player) >= 2` | Draw if multiple players qualify simultaneously |
| RESOLUTION | GAME_OVER | Player disconnects > `disconnect_grace_seconds` | — |
| DRAFT_AUCTION | DRAFT_SHOP | Auction System signals settlement | — |
| DRAFT_AUCTION | GAME_OVER | Player disconnects > `disconnect_grace_seconds` | — |
| DRAFT_SHOP | PLACEMENT | `draft_shop_timer` reaches 0, OR all players signal ready | — |
| DRAFT_SHOP | GAME_OVER | Player disconnects > `disconnect_grace_seconds` | — |

GAME_OVER is terminal — no transitions out.

---

### Interactions with Other Systems

| System | Interface direction | What the RSM does |
|---|---|---|
| **Economy System** | RSM → Economy | Fires `apply_mana_ramp` and `apply_gold_income` on DRAFT entry; fires `interest_snapshot` at RESOLUTION end |
| **Card Data & Pool** | RSM → Pool | Fires `refresh_shop(player)` on DRAFT_AUCTION or DRAFT_SHOP entry (after economy events) |
| **Auction System** *(GDD not yet written)* | RSM ↔ Auction | Sends `StartAuction(round_number)` on DRAFT_AUCTION entry; waits for `AuctionSettled` signal before transitioning to DRAFT_SHOP |
| **Combat Resolution** *(GDD not yet written)* | RSM ↔ Combat | Sends `BeginResolution` on RESOLUTION entry; waits for `ResolutionComplete` signal; kill and objective rewards fire inside Combat Resolution, not RSM |
| **Objective System** *(GDD not yet written)* | RSM reads | Reads `real_objectives_destroyed(player)` at RESOLUTION end to evaluate GAME_OVER condition |
| **Server-side RNG** | Combat/Objective call directly | RSM ensures random operations only occur inside RESOLUTION state; does not call RNG itself |
| **Network Protocol / Lightyear** *(GDD not yet written)* | RSM → all clients | Broadcasts `S2CPhaseChanged` on every transition; relies on Lightyear for delivery to all connected clients |
| **Board/Lane System** *(GDD not yet written — provisional)* | RSM → Board | Fires `OnResolutionEnd` event; Board/Lane System listens to clean up dead units and carry over board state. Interface to be finalized when Board/Lane GDD is written. |

## Formulas

### F1 — Auction Round Detection

The `is_auction_round` formula is defined as:

```
is_auction_round(R) = (R mod 3 == 0)
```

**Variables:**

| Variable | Symbol | Type | Range | Description |
|---|---|---|---|---|
| Round number | R | u32 | 1–∞ | The round number being evaluated (1-indexed; DRAFT_INITIAL = round 1) |
| Result | — | bool | {true, false} | Whether this round includes an auction phase |

**Output:** Boolean. `true` if R is divisible by 3; `false` otherwise.

**Worked examples:**
- R=1 (DRAFT_INITIAL): formula not evaluated — DRAFT_INITIAL never has an auction
- R=2: 2 mod 3 = 2 → false
- R=3: 3 mod 3 = 0 → **true** (first auction)
- R=4: 4 mod 3 = 1 → false
- R=6: 6 mod 3 = 0 → **true**
- R=9: 9 mod 3 = 0 → **true**

**Density note:** One auction every 3 rounds. In a typical 9–12 round game: 3–4 auctions total.

---

### F2 — Phase Entry Sequence

On entry into DRAFT_INITIAL, DRAFT_AUCTION, or DRAFT_SHOP (from RESOLUTION or game start), the server fires these events in this exact order before accepting player input:

| Step | Event | Applies to | Dependency reason |
|---|---|---|---|
| 1 | `apply_mana_ramp(player)` — `current_mana = min(R, mana_cap)` | All DRAFT phases | Players need mana set before they can evaluate card costs |
| 2 | `apply_gold_income(player)` — `gold += baseline + interest` | All DRAFT phases | Players need gold credited before purchasing |
| 3 | `refresh_shop(player)` | All DRAFT phases | Shop is populated after gold is known |
| 4 | `StartAuction(round_number)` → Auction System | DRAFT_AUCTION only | Auction System must be ready before clients are notified of the phase; bids arriving before StartAuction is processed would reach an uninitialised Auction System |
| 5 | Broadcast `S2CPhaseChanged` (+ `S2CGameOver` if applicable) | All DRAFT phases | Clients are notified only after all server state is correct and the Auction System is ready |

Exception for round 1 (DRAFT_INITIAL): Step 2 grants `starting_gold = 5` instead of `baseline + interest`. Steps 3–5 follow normally (no step 4 since DRAFT_INITIAL has no auction).

---

### F3 — Round Timing Reference

Per-round wall-clock time as seen by players. `T_res` is the server-determined RESOLUTION duration — variable, not player-controllable.

| Round type | Phases | Minimum duration |
|---|---|---|
| Round 1 (DRAFT_INITIAL) | DRAFT_INITIAL + PLACEMENT + RESOLUTION | 45s + 10s + T_res |
| Non-auction round (R mod 3 ≠ 0) | DRAFT_SHOP + PLACEMENT + RESOLUTION | 30s + 10s + T_res |
| Auction round (R mod 3 = 0) | DRAFT_AUCTION + DRAFT_SHOP + PLACEMENT + RESOLUTION | ≥20s + 30s + 10s + T_res |

**Timer values owned by the RSM:**
- `draft_initial_timer_seconds` = 45 (new constant; early exit expected at ~25–30s via all-submit)
- `draft_shop_timer_seconds` = 30

**Timer values owned by other systems (cross-reference only):**
- `placement_timer_seconds` = 10 (owned by game-config.md)
- `auction_timer_seconds` = 20 base, up to resets (owned by game-config.md; Auction System drives it)

**UI planning note:** The timer display must handle 45s, 30s, and 10s countdowns (RSM-driven). DRAFT_AUCTION's countdown is driven by the Auction System, not the RSM.

## Edge Cases

**If the final player's submit arrives on the same server tick the placement_timer reaches 0**: all-submit path takes priority — the player is not treated as having played zero cards. The submit causal event fires first; the timer expiry is the fallback.

**If the final ready signal in DRAFT_SHOP arrives on the same tick the draft_shop_timer reaches 0**: all-ready path takes priority, by the same rule. Both paths lead to PLACEMENT; the earliest causal event is honored.

**If a card purchase message from a player is in-flight when draft_initial_timer expires**: the purchase is rejected. The server's timer is authoritative; the S2CPhaseChanged broadcast triggers client shop lockout. No grace window for in-flight messages.

**If two concurrent triggers both attempt to advance the RSM in the same frame** (e.g., all-submit AND timer expiry during PLACEMENT): the RSM processes the first and discards the second. The second trigger finds `phase ≠ PLACEMENT` and is ignored. The RSM must not transition twice from the same state.

**If a GAME_OVER condition and a normal RESOLUTION→DRAFT transition evaluate simultaneously at RESOLUTION end**: GAME_OVER takes precedence. The transition table evaluates the loss condition before computing next-DRAFT routing.

**If AuctionSettled arrives after the RSM has already advanced to GAME_OVER** (e.g., a disconnect fired mid-auction): the signal is silently discarded. The RSM validates `phase == DRAFT_AUCTION` before acting on AuctionSettled.

**If a player disconnects during DRAFT_AUCTION**: GAME_OVER fires per Rule 13. The RSM sends `AbortAuction` to the Auction System before transitioning to GAME_OVER. The Auction System must not be left in an active state that would later fire a stale AuctionSettled.

**If AuctionSettled never arrives** (Auction System deadlock or crash): the RSM waits up to `auction_max_duration_seconds` (safety timeout — see Tuning Knobs), then fires `AuctionAborted`, treats the round as a no-bid outcome, and transitions to DRAFT_SHOP. This timeout must never fire in normal play.

**If `round_number = 0` is evaluated**: this is a bug — the counter is set to 1 at DRAFT_INITIAL entry and never decremented. `0 mod 3 = 0` would incorrectly trigger an auction. Guard: `round_number >= 1` must hold before any `is_auction_round` evaluation.

**If a player reconnects at exactly `disconnect_grace_seconds`**: the player survives. The condition is `time_since_heartbeat > disconnect_grace_seconds` (strict greater-than). A heartbeat received at exactly t=5.000 is valid.

**If both players disconnect simultaneously** (e.g., server-side network partition): both exceed the grace period on the same tick. Result is a **Draw** — no winner is declared. The RSM must evaluate all disconnect trackers in a single pass before deciding outcome; sequential processing would incorrectly declare one player the winner.

**If one team member disconnects in a team mode**: that player's entire team loses immediately. The RSM maps player → team and applies GAME_OVER to all players on that team. If the losing team had already met the win condition in the same RESOLUTION, the disconnect GAME_OVER takes precedence.

**If a loss condition trigger (gameplay win) and a disconnection occur simultaneously at RESOLUTION end**: evaluate in this order — (1) if the disconnecting player is the loser, the gameplay outcome stands (they lost by objectives); (2) if the disconnecting player is the winning player, the disconnect outcome applies (opponent wins by forfeit). Disconnect overrides victory.

**If GAME_OVER fires mid-RESOLUTION due to disconnection** (not at RESOLUTION end): the current combat sub-step completes. The interest snapshot does NOT fire — RESOLUTION never reached its end. Economy state is not updated for a game that is already over.

**If the expected player count for the game session is never reached in LOBBY**: the session is cancelled after `lobby_timeout_seconds` (see Open Questions). Result is "match not started" — no GAME_OVER, no winner. The waiting player is not awarded a win.

**If a player connects to a session already past LOBBY** (late join): the session is no longer accepting players. The late-connecting client receives a spectator view of the current state via S2CPhaseChanged and cannot participate. The RSM does not re-evaluate player count after DRAFT_INITIAL has begun.

**If a player signals ready in DRAFT_SHOP and then sends a purchase message before the server processes the all-ready transition**: the purchase is accepted if the server is still in `phase == DRAFT_SHOP` when it is processed. The transition is atomic at the server tick it fires — messages in the same pre-transition tick are accepted; messages arriving after the transition executes are rejected.

## Dependencies

### Upstream Dependencies

| System | Type | Interface | Notes |
|---|---|---|---|
| **Game Config** | Hard | Read-only — RSM reads `placement_timer_seconds`, `draft_shop_timer_seconds`, `draft_initial_timer_seconds`, `disconnect_grace_seconds` at startup | All RSM timer values come from GameConfig; changing them requires no RSM code change |
| **Economy System** | Hard | RSM calls `apply_mana_ramp`, `apply_gold_income`, `interest_snapshot` at phase boundaries | Economy System does not self-trigger; RSM is the sole authority on when economy events fire |
| **Card Data & Pool** | Hard | RSM calls `refresh_shop(player)` on DRAFT_AUCTION and DRAFT_SHOP entry | Card Data & Pool executes the weighted draw; RSM only fires the signal |
| **Server-side RNG** | Soft | RSM does not call RNG directly; it provides the RESOLUTION state during which other systems call RNG | If RNG were unavailable, combat resolution would fail — but the RSM itself would still function |

### Downstream Dependents

| System | Type | Interface | Notes |
|---|---|---|---|
| **Auction System** *(GDD not yet written)* | Hard | RSM → Auction: `StartAuction(round_number)`, `AbortAuction`; Auction → RSM: `AuctionSettled` | Bidirectional — RSM drives and waits; Auction System must support AbortAuction for disconnection edge case |
| **Combat Resolution** *(GDD not yet written)* | Hard | RSM → Combat: `BeginResolution`; Combat → RSM: `ResolutionComplete` | Kill and objective rewards fire inside Combat Resolution; RSM receives only the completion signal |
| **Objective System** *(GDD not yet written)* | Hard | RSM reads `real_objectives_destroyed(player)` at RESOLUTION end | RSM does not mutate objective state; read-only dependency |
| **Network Protocol / Lightyear** *(GDD not yet written)* | Hard | RSM emits `S2CPhaseChanged` via Lightyear broadcast on every transition | All clients depend on this for phase mirror; late-joiner sync is an open question (see Open Questions) |
| **Game Session System** *(GDD not yet written)* | Hard | Game Session System manages the LOBBY state and triggers DRAFT_INITIAL start; RSM takes over from DRAFT_INITIAL onward | Game Session System sets up player count, class selection, and mode config that RSM reads for LOBBY guard conditions |
| **Board/Lane System** *(GDD not yet written — provisional)* | Soft | RSM fires `OnResolutionEnd`; Board/Lane System listens for board cleanup | Provisional — interface to be finalized when Board/Lane GDD is authored |
| **All Feature systems** | Soft | Feature systems gate their logic on `phase == X` (e.g., Auction only active in DRAFT_AUCTION) | RSM doesn't push to feature systems beyond phase signals; each system reads current phase independently |

## Tuning Knobs

All RSM timer constants are loaded from `GameConfig` (`assets/config/game_config.ron`) at startup. No code change is required to tune them. **Note:** `auction_max_duration_seconds` and `resolution_max_duration_seconds` must be explicitly added to `game-config.md` when that GDD is reviewed.

| Knob | Default | Safe Range | Too Low | Too High | Interacts With |
|---|---|---|---|---|---|
| `draft_initial_timer_seconds` | 45 | 30–60 | New players can't evaluate 9 cards + 5g budget before time expires | Experienced players sit idle; kills pacing on round 1 | `draft_shop_timer_seconds` — round 1 total = this + 10s placement |
| `draft_shop_timer_seconds` | 30 | 20–45 | Players miss optimal purchases; especially punishing on auction rounds (less shop time after auction) | Pacing sags; experienced players finish in ~10s and wait | `placement_timer_seconds` — total round length = this + 10s; on auction rounds add ≥20s |
| `disconnect_grace_seconds` | 30 | 15–60 | Browser OS events (tab switches, antivirus, Windows Update) cause 3–6s gaps; values below 15s produce false forfeits on the target platform | Window is abusable for intentional lag stalling — keep ≤60s | None — independent timer |
| `resolution_max_duration_seconds` | 60 | 30–180 | Aborts combat resolution mid-execution; must be set well above the realistic max for a full board resolution | Server hangs in RESOLUTION longer before safety Draw fires | Combat Resolution sub-step count — realistic max depends on board complexity; must never fire in normal play |
| `auction_max_duration_seconds` | 120 | 60–300 | Aborts a legitimately long auction if bids keep arriving; must be set well above max realistic auction duration | Deadlock hangs a session longer before safety recovery | `auction_timer_seconds` + `auction_timer_reset_seconds` — realistic max is `auction_timer + (est. max bids × auction_timer_reset)` |

**Cross-referenced constants (owned by Game Config — not tunable here):**

| Constant | Value | Source |
|---|---|---|
| `placement_timer_seconds` | 10 | `game-config.md` |
| `auction_timer_seconds` | 20 | `game-config.md` |
| `auction_timer_reset_seconds` | 5 | `game-config.md` |

**Knob interactions:**
- `draft_shop_timer_seconds` + `placement_timer_seconds` determine minimum non-auction round length (default: 40s). Shortening both aggressively removes deliberation.
- On auction rounds, reducing `draft_shop_timer_seconds` is more impactful because players have already spent time bidding — less time remains for shopping.
- `auction_max_duration_seconds` should always be ≥ `auction_timer_seconds + (20 × auction_timer_reset_seconds) = 120s` to avoid aborting a game-deciding late auction.

## Visual/Audio Requirements

The RSM owns no art assets. It generates phase-change events that all visual and audio output hooks into. The HUD GDD, Shop/Auction UI GDD, and audio system own all asset specifications.

| RSM event | Expected trigger (owned by HUD/Audio GDD) |
|---|---|
| DRAFT_INITIAL entry | Phase announcement overlay; shop card deal animation |
| DRAFT_AUCTION entry | Auction panel appears; ambient urgency audio begins |
| DRAFT_SHOP entry | Shop panel active; calm ambient audio |
| PLACEMENT entry | "PLACEMENT" overlay; tension audio; timer begins ticking |
| PLACEMENT: all-submit early exit | Immediate reveal audio sting |
| RESOLUTION entry | Reveal overlay; dramatic audio sting; combat resolves |
| GAME_OVER entry | Win/Loss/Draw screen; corresponding fanfare or defeat audio |

## UI Requirements

The RSM drives the following UI elements via `S2CPhaseChanged` broadcasts. Each element is owned by the HUD GDD — this section establishes the data the RSM provides.

| UI element | Phase | RSM data provided |
|---|---|---|
| Phase indicator | All | `phase` enum value from `S2CPhaseChanged` |
| Round number | All | `round_number` from `S2CPhaseChanged` |
| DRAFT_INITIAL countdown | DRAFT_INITIAL | `timer_duration_secs = 45` on entry; client counts down locally |
| DRAFT_SHOP countdown | DRAFT_SHOP | `timer_duration_secs = 30` on entry |
| PLACEMENT countdown | PLACEMENT | `timer_duration_secs = 10` on entry; must stop (not hide) on early-exit |
| GAME_OVER result screen | GAME_OVER | `loser: PlayerId`, `round: u32`, `reason: GameOverReason` |
| Auction UI trigger | DRAFT_AUCTION | RSM enters DRAFT_AUCTION state; Auction System drives auction timer shown in Auction UI |

📌 **UX Flag — Round State Machine**: This system has UI requirements. In Pre-Production, run `/ux-design` to create a UX spec for the HUD and phase-transition overlay before writing epics. Stories referencing phase display should cite `design/ux/hud.md`, not this GDD directly.

## Acceptance Criteria

| # | Criterion | Type |
|---|---|---|
| RSM-1 | GIVEN all expected players connect to the session, WHEN the RSM evaluates the LOBBY guard, THEN it transitions to DRAFT_INITIAL and broadcasts `S2CPhaseChanged(DRAFT_INITIAL, round=1)`. | BLOCKING |
| RSM-2 | GIVEN DRAFT_INITIAL ends (all players submit OR timer expires), WHEN the RSM transitions, THEN the next state is PLACEMENT and round_number = 1. The RSM does NOT transition to DRAFT_SHOP or DRAFT_AUCTION. | BLOCKING |
| RSM-3 | GIVEN RESOLUTION completes with no loss condition and round_number increments to 3, WHEN the RSM evaluates `is_auction_round`, THEN it transitions to DRAFT_AUCTION (3 mod 3 = 0). | BLOCKING |
| RSM-4 | GIVEN RESOLUTION completes and round_number increments to 4, WHEN the RSM evaluates `is_auction_round`, THEN it transitions to DRAFT_SHOP (4 mod 3 ≠ 0). | BLOCKING |
| RSM-5 | GIVEN round_number increments to 9, WHEN the RSM evaluates `is_auction_round`, THEN it transitions to DRAFT_AUCTION (9 mod 3 = 0). | BLOCKING |
| RSM-6 | GIVEN DRAFT_INITIAL begins, WHEN entry actions complete, THEN each player's `current_mana = 1` and gold = 5, and each player's shop is populated with 9 cards before S2CPhaseChanged is broadcast. | BLOCKING |
| RSM-7 | GIVEN DRAFT_SHOP entry for round_number = 3 and mana_cap ≥ 3, WHEN economy events fire, THEN `current_mana = 3`. If mana_cap = 2, `current_mana = 2`. Formula is `min(round_number, mana_cap)` — not a flat value. | BLOCKING |
| RSM-8 | GIVEN RESOLUTION completes for round N with no loss condition, WHEN the RSM transitions to the next DRAFT, THEN round_number = N+1 at the moment `apply_mana_ramp` is called. Evidence: `current_mana = min(N+1, mana_cap)`, not `min(N, mana_cap)`. | BLOCKING |
| RSM-9 | GIVEN a player holds 8 gold at RESOLUTION end, WHEN the next DRAFT begins, THEN that player receives gold income including interest = 1 (floor(8/5) = 1), applied before any shop purchases are accepted. | BLOCKING |
| RSM-10 | GIVEN DRAFT_SHOP begins at any round_number, WHEN a player attempts a purchase, THEN the purchase is accepted only after `apply_mana_ramp`, `apply_gold_income`, and `refresh_shop` have all fired — a purchase message arriving before S2CPhaseChanged is delivered must be rejected. | BLOCKING |
| RSM-11 | GIVEN RESOLUTION completes and round_number mod 3 = 0, WHEN the RSM enters DRAFT_AUCTION, THEN each player's personal shop is populated with a fresh card set, and shop purchases and manual refresh are rejected until DRAFT_SHOP begins. | BLOCKING |
| RSM-12 | GIVEN DRAFT_AUCTION is active and the Auction System emits `AuctionSettled`, WHEN the RSM receives the signal, THEN it transitions to DRAFT_SHOP and broadcasts `S2CPhaseChanged(DRAFT_SHOP)`. | BLOCKING |
| RSM-13 | GIVEN DRAFT_SHOP is active and a player's gold ≥ the card's cost, WHEN the player submits a purchase, THEN the server accepts it and deducts the cost from the player's gold. | BLOCKING |
| RSM-14 | GIVEN DRAFT_SHOP is active and a player's gold < the card's cost, WHEN the player submits a purchase, THEN the server rejects it and the player's gold is unchanged. | BLOCKING |
| RSM-15 | GIVEN PLACEMENT is active with 2 players and one player submits at t=5s, WHEN the server processes the submission, THEN RESOLUTION does not begin until the second player also submits or the 10s timer expires. | BLOCKING |
| RSM-16 | GIVEN PLACEMENT is active and all players submit before the timer expires, WHEN the final submission is processed, THEN RESOLUTION begins immediately — not at timer=0. | BLOCKING |
| RSM-17 | GIVEN PLACEMENT timer reaches 0 and Player A has not submitted, WHEN RESOLUTION begins, THEN: (a) no new units are placed in any lane for Player A, (b) units present from prior rounds remain in their lane positions, and (c) Player A's hand contains the same cards as before PLACEMENT began. | BLOCKING |
| RSM-18 | GIVEN DRAFT_SHOP timer reaches 0, WHEN the RSM transitions, THEN PLACEMENT begins and `S2CPhaseChanged(PLACEMENT)` is broadcast. | BLOCKING |
| RSM-19 | GIVEN DRAFT_SHOP is active and all players signal ready at t=15s, WHEN the RSM processes the ready signals, THEN PLACEMENT begins at t=15s — the 30s timer does not run to completion. | BLOCKING |
| RSM-20 | GIVEN a player has had 1 real objective destroyed, WHEN their second real objective is destroyed during RESOLUTION, THEN after all six sub-steps complete the RSM transitions to GAME_OVER with that player as the loser. | BLOCKING |
| RSM-21 | GIVEN 2 fake objectives are destroyed (not real) during RESOLUTION, WHEN RESOLUTION completes, THEN the RSM does NOT transition to GAME_OVER. | BLOCKING |
| RSM-22 | GIVEN both players each have their second real objective destroyed in the same RESOLUTION, WHEN GAME_OVER is triggered, THEN the result is a Draw — no winner is announced. | BLOCKING |
| RSM-23 | GIVEN a player's heartbeat gap exceeds 30 seconds (strictly greater than disconnect_grace_seconds), WHEN the RSM evaluates disconnect trackers, THEN the RSM transitions to GAME_OVER with that player declared the loser and S2CGameOver(reason=Disconnection) broadcast. | BLOCKING |
| RSM-24 | GIVEN a player loses connection at t=5s and reconnects at t=20s (within disconnect_grace_seconds), WHEN the RSM evaluates, THEN GAME_OVER is not triggered and the game continues. | BLOCKING |
| RSM-25 | GIVEN a player's heartbeat gap is set to exactly `disconnect_grace_seconds` (test by directly writing `disconnect_tracker[player] = disconnect_grace_seconds`), WHEN the RSM evaluates, THEN the player survives — the condition is `> disconnect_grace_seconds`, not `>=`. | BLOCKING |
| RSM-26 | GIVEN any RSM state transition occurs, WHEN all server-side entry actions have completed, THEN an `S2CPhaseChanged` message is broadcast containing: `phase` matching the new state, `round_number` matching the post-increment counter, and `timer_duration_secs` matching the timer started in the new state (0 for RESOLUTION and GAME_OVER). | BLOCKING |
| RSM-27 | GIVEN PLACEMENT is active, WHEN a player sends a shop purchase or manual refresh message, THEN the server rejects it. | BLOCKING |
| RSM-28 | GIVEN DRAFT_AUCTION is active, WHEN a player sends a shop purchase or manual refresh message, THEN the server rejects it. | BLOCKING |
| RSM-29 | GIVEN RESOLUTION is active, WHEN a player sends a placement submission, THEN the server rejects it. | BLOCKING |
| RSM-30 | GIVEN DRAFT_INITIAL timer expires and Player A spent 3g of the 5g budget, WHEN PLACEMENT begins, THEN Player A's gold = 0 (unspent starting gold is forfeited) and Player A has only the cards they purchased in hand. | BLOCKING |
| RSM-31 | GIVEN PLACEMENT is active and both an all-submit event and a timer-expiry event arrive on the same server tick, WHEN the RSM processes them, THEN the RSM transitions to RESOLUTION exactly once — the second trigger is discarded. Double-transition would execute combat twice; this is a data-corruption scenario. | BLOCKING |
| RSM-32 | GIVEN the RSM enters any DRAFT state, WHEN entry actions fire, THEN the execution sequence is strictly: (1) apply_mana_ramp, (2) apply_gold_income, (3) refresh_shop, (4) StartAuction if DRAFT_AUCTION, (5) S2CPhaseChanged — a purchase or bid message arriving before S2CPhaseChanged is broadcast is rejected by the phase guard. | BLOCKING |
| RSM-33 | GIVEN LOBBY transitions to DRAFT_INITIAL, WHEN the RSM initialises its state, THEN round_number = 1. The value 0 must be unreachable at any is_auction_round call site. | BLOCKING |
| RSM-34 | GIVEN DRAFT_SHOP is active and all players signal ready on the same server tick the draft_shop_timer reaches 0, WHEN the RSM processes both events, THEN PLACEMENT begins exactly once — the second trigger is discarded. | ADVISORY |
| RSM-35 | GIVEN a player's heartbeat gap exceeds disconnect_grace_seconds during RESOLUTION, WHEN the RSM evaluates, THEN GAME_OVER is deferred until RESOLUTION exits naturally: the current combat sub-step completes, OnResolutionEnd fires, and GAME_OVER fires on that same RESOLUTION exit — not mid-sub-step. | BLOCKING |
| RSM-36 | GIVEN GAME_OVER fires for any reason, WHEN S2CGameOver is broadcast, THEN: (a) reason = ObjectivesDestroyed if loss condition was met, (b) reason = Disconnection if a player exceeded disconnect_grace_seconds, (c) reason = Draw if mutual destruction or mutual disconnection on the same tick. The reason field must match the actual cause. | BLOCKING |
| RSM-37 | GIVEN both players' Lightyear connections drop on the same server tick (mutual disconnection), WHEN the RSM evaluates disconnect trackers in a single pass, THEN S2CGameOver(reason=Draw) is broadcast — no single player is declared loser. | BLOCKING |
| RSM-38 | GIVEN RESOLUTION is active and ResolutionComplete is not received within resolution_max_duration_seconds, WHEN the safety timeout fires, THEN OnResolutionEnd broadcasts, S2CGameOver(reason=Draw) broadcasts, and the RSM transitions to GAME_OVER. | ADVISORY |

## Open Questions

1. **`lobby_timeout_seconds`** — How long does LOBBY wait before cancelling the session if the expected player count is never reached? Referenced in Edge Cases but no default value set. Proposed default: **90s**. Must be added to GameConfig and the entity registry before the RSM can be implemented. Owner: Game Session System GDD when authored.

2. **DRAFT_INITIAL gold forfeiture** — AC RSM-30 assumes unspent starting gold is zeroed at DRAFT_INITIAL end ("use it or lose it"). Economy System GDD does not explicitly confirm the zeroing. Confirm before either system is implemented. If gold carries forward, RSM-30 must be revised.

3. **Auction card count in multiplayer modes** — Master GDD Open Question 3: 1 card per auction in 1v1; what about 2v2/3v3? If multiple auction cards run sequentially, DRAFT_AUCTION becomes a multi-step loop. RSM design is sound for 1v1. Extension to multi-card auctions must be handled in the Auction System GDD.

4. **Late-joiner sync strategy** — Clients connecting mid-game receive `S2CPhaseChanged` for the current phase but need full game state (gold, round, board, hand). The RSM's per-transition broadcast is insufficient for a full state restore. Resolution belongs in the Network Protocol GDD; the RSM's `RoundState` resource must be structured to support full-snapshot delivery.

5. ~~**`GAME_OVER` payload `GameOverReason` enum**~~ — **Resolved.** `GameOverReason` is defined in Rule 14 of this GDD: `ObjectivesDestroyed | Disconnection | Draw`. HUD GDD may render it; the type is owned here.
