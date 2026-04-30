# HUD

> **Status**: In Design
> **Author**: SamyAnisBenachi + Claude Code agents
> **Last Updated**: 2026-04-30
> **Implements Pillar**: No idle spectating · Simple surface

## Overview

The HUD is the client-side persistent readout layer that surfaces the game's economic and tactical state to both players at all times. Where Hand UI owns the hand fan and the PLACEMENT timer, Shop/Auction UI owns the shop slots and the DRAFT/auction timers, and Board Rendering owns the lanes, units, and in-cell objective sprites, the HUD owns what is left over and always-on: the local player's **gold**, **current mana / mana cap**, and **reserve mana**; the opponent's **gold and free gold** (gold − reserved); the **round number** and a compact **phase label**; and the **objective scoreboard** — five status dots per side summarising which of the ten visible objectives are still standing without revealing which were real or fake. The HUD subscribes to four server-to-client messages — `S2CGoldUpdate` (unicast, own economy), `S2CGoldBroadcast` (broadcast, both players' gold + reserved_gold), `S2CPhaseChanged` (round + phase), and `ObjectiveDestroyed` (per-objective fall) — and rebuilds its full display from `S2CGameSnapshot` on reconnect. It produces no client-to-server messages: the HUD is read-only, server-authoritative, and never asserts state.

The HUD is what makes "no idle spectating" enforceable across the whole round, not just during decisions the player owns. During the opponent's auction the local player has no input, but the HUD shows the opponent's gold dropping, the local player's own gold reserved by their bid, and the round number — three live data points that turn a passive watch into an information read. During RESOLUTION, sister UIs disappear; the HUD persists, and gold/objective deltas tick visibly so the consequences of the round are written into the readout the player carries into the next DRAFT. The HUD's job is to be peripheral but never absent: glanceable, never demanding attention, but always answering the same three questions — *what do I have*, *what do they have*, *how close is anyone to winning*.

## Player Fantasy

**The HUD is what you never look at, and always know.**

If Hand UI is the war map and the Board is the place the opponent cannot lie, the HUD is the corner of your eye that is doing math while you stare at the auction. It serves the fantasy of **peripheral omniscience under an information war** — the four numbers and ten dots that compose this game's economic and tactical truth, broadcast at the screen's edge so quietly that the player learns to trust them without ever taking their gaze off the central decision. By round six, a competent player tallies their opponent's free gold without focusing on it; by round twelve, they always know which round it is, how much reserve mana they hold, and how many dots are still standing on each side — without any of those readings consuming a moment of central attention. The HUD's signature feeling is not surprise or revelation; it is the quiet absence of surprise — every read confirms what was already half-known.

**The anchor moment.** Opponent's auction, round 9. The auction panel is climbing — 7g, 8g, 9g — and your central gaze is locked on the bid number, calculating whether to raise. But you already know, without ever looking down, that they have exactly 3g of free gold left, that you have 11g free of which 4g is now reserved on this bid, and that round 9 means two more shop windows before the long game runs out of runway. The HUD told you all of this peripherally, in the half-second between blinks. You raise to 10. They fold. Your hunt was complete before you committed — and the HUD never asked for your eyes.

**The five feelings the HUD is responsible for delivering:**

1. **"I am peripherally omniscient."** — Central vision belongs to the auction, the hand, and the board; the HUD lives in the corner of the eye and is read without focus. By the third game, the player tracks the opponent's free gold across rounds the way a hunter tracks tracks — never with effort, always with continuity.

2. **"I am never ambushed by a number."** — No gold total, no mana count, no objective fall ever surprises the well-informed player. The HUD has been quietly broadcasting the truth the whole match; the only player who is surprised is the one who stopped reading. Every bid, every placement is made with the full economic picture already loaded.

3. **"I am always reading the closing pages, even on round two."** — The objective scoreboard is the match's whole arc collapsed onto ten dots. A player glancing at "two of mine alive, five of theirs" on round 4 already knows the long curve they are riding. The HUD persists across every phase, so the endgame is never out of view, even when it is twelve rounds away.

4. **"The HUD never lies to flatter me."** — Five dots per side, alive or dead, with no real/fake hint until the moment of destruction. No win-probability bar. No "you're winning!" colour. The HUD's neutrality is what makes its truths trustworthy — when one of your real objectives falls, the dot simply darkens, and the silence is heavier than any animation could be.

5. **"Watching is reading."** — During RESOLUTION, sister UIs vanish; the HUD persists. Gold deltas tick visibly, dots darken, the round number does not yet advance. The watch-the-tape phase is not idle because the HUD is still feeding the next decision.

**What the HUD must NOT feel like:**
- **A focal point** — it must never compete with auction, hand, or board for the central gaze; if the player is *looking at* the HUD instead of *through* it, the design has failed.
- **A warning system** — no flashing, no urgency colours that demand attention away from the central decision; the HUD informs, it does not interrupt.
- **A moving target** — layout, position, and visual hierarchy are fixed for the entire match; muscle memory must hold from round 1 to round 20.
- **A scoreboard from a sports broadcast** — no "ROUND 9 OF 20" framing, no theatrical announcements; this is a tactician's instrument panel, not a stadium screen.
- **A celebration surface** — no number ever pulses to congratulate; no objective dot ever sparkles when the player destroys an opponent's; the HUD reports facts and lets the player feel them.

*Pillar alignment: **Auction as signature** — the HUD is what makes the auction's bluffs legible across the entire table; without persistent opponent-gold readout, the open ascending auction becomes a guessing game instead of a negotiation. **No idle spectating** — peripheral reading converts every opponent-turn moment into active intelligence gathering. **Simple surface** — five dots per side, alive or dead, one rule; gold is gold; mana is mana — the HUD adds no second-order logic on top of the data it surfaces.*

## Detailed Design

### Core Rules

**Rule 1 — Pre-pooled HUD node tree.**
All HUD elements — phase label, round counter, local gold, local mana (numerator + denominator), local reserve mana, opponent gold, opponent reserved sub-label, and the 10 scoreboard dot entities (5 opponent + 5 local) — are spawned at session start as children of a single root `Node` and toggled via `Visibility::Visible` / `Visibility::Hidden`. No per-round spawn or despawn; no per-update entity creation. The root `Node` carries `PointerEvents::None` (or equivalent Bevy 0.18 idiom) so HUD never captures click events; sister UIs receive all input.

**Rule 2 — Screen placement (fixed for the entire match).**
The HUD occupies four screen-edge zones, each anchored with `Val::Px(12.0)` from the screen edges. Layout never reflows during a match:

| Zone | Contents |
|---|---|
| Top-left | Phase label (line 1) + round counter (line 2) |
| Top-center | Scoreboard — 2 rows × 5 dots; top row = opponent; bottom row = local; dots aligned horizontally to lane midpoints 1–5 |
| Top-right | Opponent gold (line 1); opponent reserved sub-label (line 2, DRAFT_AUCTION only) |
| Bottom-left | Local gold (line 1); local reserved-gold sub-label (line 2, DRAFT_AUCTION only) |
| Bottom-right | Local mana (line 1, format `current / mana_cap`); local reserve mana (line 2, hidden when reserve_mana == 0) |

Bottom-center is reserved for the Hand UI fan and must remain clear of all HUD elements. Center-screen is reserved for Shop/Auction UI panels and the DRAFT_INITIAL grid.

**Rule 3 — Display formats (exact strings).**

| Readout | Format | Example |
|---|---|---|
| Phase label | `<PHASE_STRING>` (see Rule 5) | `AUCTION` |
| Round counter | `R<round_number>` | `R9` |
| Local gold | `<gold>g` | `8g` |
| Local mana | `<current_mana> / <mana_cap>` | `6 / 10` |
| Local reserve mana | `+<reserve_mana> reserve` (hidden when reserve_mana == 0) | `+2 reserve` |
| Local reserved gold (DRAFT_AUCTION only) | `<reserved_gold>g reserved` | `4g reserved` |
| Opponent gold | `<gold>g` | `11g` |
| Opponent reserved gold (DRAFT_AUCTION only) | `<reserved_gold>g reserved` | `5g reserved` |

Mana label always shows two numbers separated by ` / `, even when `current_mana == mana_cap` (i.e. `10 / 10`, not `MAX`). The cap can change mid-match from fake-objective rewards — a single-number display would create a false impression of cap stability.

**Rule 4 — Update triggers (per-message contract).**
Each HUD readout is updated only by the message(s) below; HUD never redraws the full readout tree, only the affected text/visibility:

| Message | Updates |
|---|---|
| `S2CGoldUpdate { gold, current_mana, reserve_mana, mana_cap }` | Local gold, local mana numerator + denominator, local reserve mana visibility + value |
| `S2CGoldBroadcast { player_id, gold, reserved_gold }` | If `player_id == opponent_id`: opponent gold + opponent reserved sub-label. If `player_id == local_id`: local reserved-gold sub-label only (local gold itself is owned by `S2CGoldUpdate` — see Rule 11) |
| `S2CPhaseChanged { phase, round_number, timer_duration_ms }` | Phase label, round counter, mode transitions per Rule 5. **`timer_duration_ms` is explicitly ignored** — see Rule 12 |
| `ObjectiveDestroyed { target_player_id, lane, was_fake }` | Single dot at `(target_player_id, lane)` transitions ALIVE → DESTROYED. **`was_fake` is ignored by HUD** — see Rule 7 |
| `S2CGameSnapshot` | Full HUD rebuild (Rule 13) |

**Rule 5 — Phase label strings + mode transitions.**

| RSM Phase | Phase label | Mode |
|---|---|---|
| LOBBY | (HUD hidden) | `HIDDEN` |
| DRAFT_INITIAL | `DRAFT INITIAL` | `ECONOMY_BASIC` |
| DRAFT_SHOP | `DRAFT` | `ECONOMY_BASIC` |
| DRAFT_AUCTION | `AUCTION` | `ECONOMY_AUCTION` |
| PLACEMENT | `PLACEMENT` | `ECONOMY_BASIC` |
| RESOLUTION | `RESOLUTION` | `ECONOMY_BASIC` |
| GAME_OVER | `GAME OVER` (no round suffix) | `FROZEN` |

On `S2CPhaseChanged(DRAFT_AUCTION)`: HUD enters `ECONOMY_AUCTION` mode — opponent reserved sub-label and local reserved-gold sub-label become visible (their values populate from the next `S2CGoldBroadcast` payloads). On any phase exit from DRAFT_AUCTION: both reserved sub-labels hide. Phase label and round counter update atomically on the message — no fade, no animation.

**Rule 6 — Scoreboard layout and state.**
Two horizontal rows of 5 dot entities are pre-spawned. Indexed by `(player_id, lane)` into a fixed `[[Entity; 5]; 2]` array — O(1) lookup, no query needed. Row mapping: index `[0]` = opponent (top row); index `[1]` = local (bottom row). Lane mapping: `dots[player][lane - 1]` for lanes 1..=5. Each dot has two states:

- **ALIVE** — filled circle, full opacity, neutral high-contrast tint (Art Director defines exact hue)
- **DESTROYED** — darkened/hollow, reduced opacity; same visual for real and fake

ALIVE → DESTROYED is an instantaneous state flip on `ObjectiveDestroyed` receipt. No tween, no animation. Board Rendering owns the in-cell objective shatter animation; the HUD scoreboard is a permanent record, not a reaction surface.

**Rule 7 — Real/fake identity is never shown on the scoreboard.**
The HUD's scoreboard renders all 5 dots identically per side regardless of real/fake assignment, even on the local player's own row (where the client knows the assignment from `S2CObjectiveIdentities` per ADR-001). Two reasons: (a) a persistent on-screen real/fake indicator becomes a tell readable via screen-share, video capture, or shoulder surfing; (b) the scoreboard's contract with the player is *alive vs destroyed*, not *real vs fake* — that distinction belongs to the destruction reveal animation owned by Board Rendering, and to the destroyed-attacker's awareness of `was_fake` in their own UI feedback (handled outside HUD scope). The `was_fake` field on `ObjectiveDestroyed` is forwarded to Board Rendering via Board Rendering's own `MessageReader`; HUD ignores it entirely.

**Rule 8 — Opponent gold display, adaptive by phase.**
Outside DRAFT_AUCTION (i.e. `ECONOMY_BASIC` mode), the opponent gold readout shows a single value: `"<gold>g"` updated from `S2CGoldBroadcast { player_id: opponent_id, gold, reserved_gold }`. The `reserved_gold` field is read but its value is invariant 0 outside DRAFT_AUCTION (server invariant: bids only exist during DRAFT_AUCTION) — the opponent reserved sub-label is `Visibility::Hidden`. On entry to DRAFT_AUCTION (Rule 5), the sub-label becomes visible and populates from the next `S2CGoldBroadcast`. On DRAFT_AUCTION exit, the sub-label hides regardless of `reserved_gold` value — the server clears bids as part of auction settlement.

**Rule 9 — RESOLUTION: HUD persists, sister UIs hide.**
On `S2CPhaseChanged(RESOLUTION)`: HUD remains fully visible at exactly the same visual weight it held during PLACEMENT (no scaling up, no opacity shift, no contrast change — that would be a "moving target" violation of the player fantasy). Hand UI hides immediately (Hand UI Rule 12); Shop/Auction UI hides its panels. The HUD is the only persistent UI surface during RESOLUTION. Gold deltas (kill rewards, objective rewards, embedded `GoldAwarded` entries from `S2CResolutionEvent`) arrive as `S2CGoldUpdate` and `S2CGoldBroadcast` and update the readouts in real time per Rule 4. Numeric tweens are ≤300ms (see Rule 14). Dot darkenings on `ObjectiveDestroyed` fire instantly (Rule 6).

**Rule 10 — GAME_OVER: HUD freezes, never reveals identity retroactively.**
On `S2CPhaseChanged(GAME_OVER)`: phase label updates to `GAME OVER` (no round suffix). All readouts retain their last received state. The scoreboard does **not** retroactively reveal real/fake on either side — destroyed dots remain "destroyed", alive dots remain "alive", with no identity glyph added. A separate post-game summary screen (not owned by HUD) may reveal the full objective map, but the HUD's contract holds: the dots mean *alive vs destroyed*, end of match included. A win/loss overlay renders above the HUD; the HUD remains visible beneath it as a final-state record.

**Rule 11 — Tie-break: `S2CGoldUpdate` vs `S2CGoldBroadcast` for local gold.**
`S2CGoldUpdate` is the authoritative unicast update for the local player's economy (network-protocol.md). `S2CGoldBroadcast` is broadcast to all players for cross-player visibility. When both arrive in the same Bevy tick:
1. The HUD system processes `S2CGoldBroadcast` first (system order: lower priority).
2. The HUD system processes `S2CGoldUpdate` second (system order: higher priority).
3. The final value written to the local gold label is the value from `S2CGoldUpdate`.
This ordering is enforced via `app.configure_sets` or explicit `.after()` dependency between two HUD systems and is a code contract, not an optional optimisation. The local reserved-gold sub-label is owned exclusively by `S2CGoldBroadcast` (Rule 4) — `S2CGoldUpdate` does not carry `reserved_gold` and never overwrites it.

**Rule 12 — HUD never displays a timer.**
The PLACEMENT timer is owned by Hand UI (hand-ui.md Rule 11). The DRAFT_INITIAL, DRAFT_SHOP, and DRAFT_AUCTION timers are owned by Shop/Auction UI (shop-auction-ui.md Rules 5 [DRAFT_INITIAL], DRAFT_SHOP Rule 5, Auction Panel Rule 3). The `timer_duration_ms` field of `S2CPhaseChanged` is read by those systems, not by HUD. The HUD's `MessageReader<S2CPhaseChanged>` discards the timer field — a comment in the system documents this to prevent future contributors from wiring a HUD timer.

**Rule 13 — Reconnect: `S2CGameSnapshot` rebuild.**
On receipt of `S2CGameSnapshot`, the HUD reads the embedded economy and scoreboard state in a single synchronous pass and writes to all label entities and all dot entities before the next frame renders. Because all HUD entities are pre-pooled (Rule 1), the rebuild is a series of `Text` and `Visibility` writes with no spawn latency, no flicker. The snapshot is sufficient and authoritative — the HUD does not wait for subsequent `S2CGoldUpdate` or `S2CGoldBroadcast` to populate state after a reconnect. Phase label and round counter populate from the snapshot's `phase` and `round_number` fields, which advance the HUD into the correct mode (Rule 5) atomically.

**Rule 14 — Animation budget.**
Numeric value updates (gold, mana, reserve, reserved) tween over ≤300ms via `bevy_tweening` from previous to new value. Phase label, round counter, and dot state changes are **not** animated — text replaces in place, dot darkens instantly. No flashing, no pulsing, no urgency colours, no scale tweens larger than ±1 pixel. Animations that obscure or compete with the central decision (auction panel, hand fan, board) are forbidden.

### States and Transitions

| RSM Phase | HUD Mode | Visibility | Transitions in / out |
|---|---|---|---|
| `LOBBY` | `HIDDEN` | All elements `Visibility::Hidden` | Out: on `S2CPhaseChanged(DRAFT_INITIAL)` — all elements made visible, populated by next snapshot or `S2CGoldUpdate` |
| `DRAFT_INITIAL` | `ECONOMY_BASIC` | All readouts visible; both reserved sub-labels hidden; all 10 dots ALIVE | In: from `LOBBY` on first `S2CPhaseChanged`. Out: to `PLACEMENT` |
| `DRAFT_SHOP` | `ECONOMY_BASIC` | Same as above | In: from `RESOLUTION` (non-auction rounds) or from `DRAFT_AUCTION` |
| `DRAFT_AUCTION` | `ECONOMY_AUCTION` | All readouts visible; **opponent reserved + local reserved-gold sub-labels become visible** | In: from `RESOLUTION` (auction rounds 3, 6, 9…). Out: to `DRAFT_SHOP` — both reserved sub-labels hide |
| `PLACEMENT` | `ECONOMY_BASIC` | Same as DRAFT_SHOP; sister Hand UI shows placement timer (HUD does not) | In: from `DRAFT_INITIAL` or `DRAFT_SHOP`. Out: to `RESOLUTION` |
| `RESOLUTION` | `ECONOMY_BASIC` | HUD remains fully visible at unchanged weight; sister UIs hide | In: from `PLACEMENT`. Out: to `DRAFT_AUCTION`, `DRAFT_SHOP`, or `GAME_OVER` |
| `GAME_OVER` | `FROZEN` | All readouts retain last state; phase label = `GAME OVER`; no further updates accepted | In: from `RESOLUTION`. Terminal — no out |

### Interactions with Other Systems

**`S2CGoldUpdate` (own economy — network-protocol.md, source: economy-system.md).**
Single subscriber: `HudPlugin`'s `MessageReader<S2CGoldUpdate>`. Update granularity: writes only to the four local economy label entities; no full-tree redraw. Reserve label visibility toggles on `reserve_mana == 0` boundary. Authority: this is the local player's authoritative gold/mana state — wins ties against `S2CGoldBroadcast` per Rule 11.

**`S2CGoldBroadcast` (both players — network-protocol.md).**
Single subscriber: `HudPlugin`'s `MessageReader<S2CGoldBroadcast>`. Update granularity: writes only to opponent gold + opponent reserved sub-label OR to local reserved-gold sub-label, depending on `player_id`. Local gold itself is *not* updated by this message (Rule 11). The `local_free_gold` formula registered in `entities.yaml` (`gold - reserved_gold`, source: shop-auction-ui.md) is referenced *implicitly* by HUD: the HUD does not display free gold as a separate readout, but the player computes it mentally from the two visible numbers during DRAFT_AUCTION.

**`S2CPhaseChanged` (phase + round — network-protocol.md, source: round-state-machine.md).**
Single subscriber: `HudPlugin`'s `MessageReader<S2CPhaseChanged>`. Update granularity: phase label text + round counter text + mode transition (Rule 5). The `timer_duration_ms` field is discarded (Rule 12).

**`ObjectiveDestroyed` (per-objective fall — network-protocol.md, source: objective-system.md).**
Single subscriber: `HudPlugin`'s `MessageReader<ObjectiveDestroyed>`. Update granularity: one dot entity, indexed `dots[player_id][lane - 1]`. The `was_fake` field is ignored by HUD; Board Rendering subscribes to the same message via its own reader for the in-cell shatter animation. No coupling between HUD and Board Rendering on this path.

**`S2CGameSnapshot` (reconnect — network-protocol.md).**
Single subscriber: `HudPlugin`'s `MessageReader<S2CGameSnapshot>`. Triggers full HUD state rebuild per Rule 13. After rebuild, the HUD waits for the next phase/economy/objective messages to drive incremental updates as normal.

**Strict non-ownership boundary.**
HUD MUST NOT:
- Display any countdown timer (Rule 12)
- Subscribe to `C2S*` messages (HUD produces no input messages)
- Write to any client state other than its own labels and dots
- Compute derived values that could disagree with server (e.g., HUD must not track gold as a delta accumulator — it sets the value from each authoritative message)
- Animate beyond Rule 14's budget

These constraints will be enforced in code review. A future contributor adding a timer widget, a `MessageReader<C2SPlaceBid>`, or a flashing pulse on the gold label must be rejected.

**Registry references (cross-system facts this section consumes):**
- `S2CGoldUpdate`, `S2CGoldBroadcast`, `S2CPhaseChanged`, `S2CGameSnapshot`, `ObjectiveDestroyed` — all registered in `network_messages` section of `entities.yaml`
- `local_free_gold` formula (shop-auction-ui.md) — referenced implicitly via the gold + reserved display
- `mana_cap`, `objective_hp`, `lane_count`, `fake_count` — registered constants from game-config.md / board-lane-system.md / objective-system.md

## Formulas

The HUD performs no game-logic computation — all economic, combat, and objective state arrives pre-computed from the server. This section documents the one display-time formula HUD references and the visibility predicates that drive HUD element show/hide behaviour. No new formulas are introduced; no new constants are defined.

### D.1 — Display formula (implicit, referenced from sibling system)

The HUD does not compute or display *free gold* as a separate readout (Rule 8 — the player computes it mentally from the two visible numbers during DRAFT_AUCTION). The underlying formula is:

`local_free_gold = gold - reserved_gold`

**Variables:**

| Variable | Symbol | Type | Range | Description |
|----------|--------|------|-------|-------------|
| gold | `g` | u32 | 0..=u32::MAX (practical: 0..=99) | Player's total gold from `S2CGoldBroadcast.gold` |
| reserved_gold | `r` | u32 | 0..=g (server invariant) | Player's gold reserved on active auction bid from `S2CGoldBroadcast.reserved_gold`. Always 0 outside DRAFT_AUCTION. |

**Output Range:** 0 to gold (always non-negative under server invariant `reserved_gold ≤ gold`).
**Example:** `gold = 11, reserved_gold = 4` ⇒ `local_free_gold = 7`. The HUD displays `11g` and `4g reserved`; the player reads `free = 7`.

**Authority:** Owned by `shop-auction-ui.md`, registered in `entities.yaml` formulas section as `local_free_gold`. HUD does not duplicate ownership — it consumes both `gold` and `reserved_gold` from the same `S2CGoldBroadcast` payload and renders them as two adjacent labels per Rule 3.

### D.2 — Visibility predicates

Each toggleable HUD element has one boolean predicate that determines whether it is rendered. Predicates are evaluated on every relevant message arrival; no polling.

| Element | Predicate | Source data |
|---------|-----------|-------------|
| Local reserve mana label | `reserve_label_visible := reserve_mana > 0` | `S2CGoldUpdate.reserve_mana` |
| Local reserved-gold sub-label | `local_reserved_visible := phase == DRAFT_AUCTION` | `S2CPhaseChanged.phase` |
| Opponent reserved-gold sub-label | `opp_reserved_visible := phase == DRAFT_AUCTION` | `S2CPhaseChanged.phase` |
| All HUD elements (root visibility) | `hud_visible := phase != LOBBY` | `S2CPhaseChanged.phase` |
| Phase label round suffix | `round_suffix_visible := phase != LOBBY AND phase != GAME_OVER` | `S2CPhaseChanged.phase` |

**Output Range:** all predicates evaluate to `bool`.
**Notes:**
- Predicates that depend on `phase` are re-evaluated on every `S2CPhaseChanged` message and on `S2CGameSnapshot` (Rule 13).
- The `reserve_label_visible` predicate is re-evaluated on every `S2CGoldUpdate` (the only message that carries `reserve_mana`).
- No predicate combines data from two messages — each is a single-field test, ensuring deterministic visibility transitions.

### D.3 — Dot state mapping

Each scoreboard dot has a state derived from a single per-player-per-lane boolean stored in HUD-local state (initialised to ALIVE for all 10 dots at session start, then updated by `ObjectiveDestroyed` per Rule 6):

`dot_state(player, lane) := if destroyed[player][lane] then DESTROYED else ALIVE`

**Variables:**

| Variable | Type | Source |
|----------|------|--------|
| `destroyed[player][lane]` | `[[bool; 5]; 2]` (HUD-local state) | Set true on `ObjectiveDestroyed { target_player_id: player, lane }`; reset to false on `S2CGameSnapshot` rebuild per the snapshot's objective state |

**Output Range:** `{ ALIVE, DESTROYED }`.
**Note:** No real/fake distinction enters this formula. The `was_fake` field of `ObjectiveDestroyed` is never read by HUD (Rule 7).

## Edge Cases

The HUD is a read-only display layer; most "edge cases" reduce to defensive rendering against malformed messages or boundary states. Each case names the exact condition and the exact resolution; under no condition does the HUD modify game state, abort the session, or panic.

### Cold start and first-message timing

- **If `S2CPhaseChanged(DRAFT_INITIAL)` arrives before any `S2CGoldUpdate`**: display `--g` for local gold and `-- / --` for mana with the `+N reserve` label hidden. Do not show `0g` or `0 / 10` — those are valid runtime values and would teach a wrong mental model. On first `S2CGoldUpdate` receipt, replace placeholders by direct text replacement (no tween — there is no "old value" to animate from).
- **If `S2CGoldBroadcast` arrives before `local_id` has been established by the LOBBY handshake**: discard the message. Do not attempt to render either row — displaying the wrong player's gold in the wrong zone is a worse failure than a momentary blank. The next `S2CGameSnapshot` or post-handshake broadcast will populate the readouts authoritatively.
- **If `S2CGameSnapshot` arrives as the first message after a fresh load (cold start)**: this is the canonical recovery path, not an anomaly. Pre-pooled Node tree (Rule 1) is always present at startup; the snapshot writes phase, round, both economies, and the destroyed[][] state in one synchronous pass per Rule 13.

### Reconnect and replay

- **If `S2CGameSnapshot` arrives while phase is currently DRAFT_AUCTION**: rebuild HUD from snapshot. The snapshot's `PlayerSnapshot` includes `gold` and `reserved_gold` for both players. Treat the snapshot as a full authoritative overwrite — set both reserved sub-labels visible (per D.2 predicates evaluated against the snapshot's `phase`) and populate them from the snapshot. No reconciliation needed against pre-snapshot state.
- **If `ObjectiveDestroyed` arrives for `(player, lane)` whose `destroyed[player][lane]` is already `true`**: no-op. Idempotent. Log a warning (likely a server replay or reconnect artifact). Do not re-trigger the dot transition.
- **If two `S2CPhaseChanged` are processed in the same Bevy tick**: last-write-wins — the second message overwrites the first. RSM is a strict sequence; two phases in one tick implies a reconnect artifact, and the later message is authoritative.

### Same-tick message arrival

- **If `S2CPhaseChanged`, `S2CGoldUpdate`, and `S2CGoldBroadcast` all arrive in the same Bevy tick** (canonical at DRAFT_INITIAL entry): process per Rule 11's system order — `S2CGoldBroadcast` runs first (writes opponent gold + opponent reserved sub-label), then `S2CGoldUpdate` runs (overwrites local gold/mana). `S2CPhaseChanged` may run in either order relative to the gold messages — it only writes the phase label and round, neither of which conflicts with gold.
- **If `S2CGoldUpdate` and `S2CGoldBroadcast` for the local player arrive same tick with different `gold` values**: `S2CGoldUpdate` wins for the local gold display (Rule 11). `S2CGoldBroadcast` updates only the local reserved-gold sub-label; its `gold` field is not displayed.

### Server invariant violations (defensive)

- **If `S2CGoldBroadcast.reserved_gold > gold`**: clamp `local_free_gold` mental computation to 0 by displaying `gold = X, reserved = X` (saturating: clamp `reserved` to `gold` for display). Log a warning. The server invariant forbids this; the guard prevents misleading "negative free gold" implications.
- **If `S2CGoldUpdate.mana_cap == 0`**: display `current_mana / 0` as received and log a warning. The HUD does no division — only string concatenation. The server invariant forbids `mana_cap == 0`; the guard prevents a crash, not a malformed display.
- **If `S2CGoldUpdate.mana_cap < current_mana`**: display `current_mana / mana_cap` as-is. Overfull mana is a transient legal state during economy recalculation; HUD does not clamp the displayed `current_mana`. The server is authoritative.
- **If `ObjectiveDestroyed.lane` is outside `1..=5` or `target_player_id` is unknown**: ignore the message; do not update `destroyed[][]`. Log a warning. HUD's read-only contract forbids reactive logic that could mask a server bug.
- **If `S2CGoldBroadcast.player_id` matches neither `local_id` nor `opponent_id`**: ignore and log. Cannot occur in 1v1 mode under current GDD rules.

### Animation and visual transitions

- **If a numeric tween (e.g., gold `8g → 11g`) is in-flight when a new value arrives** (e.g., `S2CGoldUpdate` with `gold = 5`): cancel the tween, start a new tween from the *currently displayed* value (whatever fractional position the in-flight tween reached) to the new authoritative value. Do not snap to either old or in-flight target. Tween duration remains ≤300ms (Rule 14).
- **If `S2CPhaseChanged(GAME_OVER)` fires while a numeric tween is in-flight**: snap immediately to the authoritative final value, cancelling the tween. A frozen HUD (Rule 10) must show the final authoritative number, not a mid-interpolation value.
- **If `S2CAuctionSettled { winner: None }` arrives**: this is owned by Shop/Auction UI, not HUD. HUD reacts only to the subsequent `S2CPhaseChanged(DRAFT_SHOP)` per Rule 5 — both reserved sub-labels hide on phase exit regardless of `reserved_gold` value. Server clears bids as part of auction settlement, so subsequent broadcasts will carry `reserved_gold = 0`.

### Layout and viewport

- **If the browser window is resized mid-match**: bevy_ui's `Val::Px` anchors and layout system reflow automatically; no explicit handler in HUD systems. Verified at 1280×720, 1920×1080, and the project's WASM target viewport. HUD layout is fixed *relative to screen edges* (Rule 2), not in absolute world coordinates.

### Cross-system gap (flagged for Open Questions)

- **Disconnect grace / paused-game indicator**: When a player is in the `disconnect_grace_seconds = 30` window (registered constant from `round-state-machine.md`), the game is paused server-side and the surviving client has nothing to do. HUD currently has no message to consume for "session paused" state — `S2CSessionPaused` / `S2CSessionResumed` are not in the network registry. **Without a paused-state message, the HUD cannot show a "Waiting for opponent…" overlay.** This is forwarded to Open Questions for Network Protocol / Game Session System resolution.

## Dependencies

### Upstream — hard (HUD cannot function without these)

| System | GDD | Interface | Direction |
|---|---|---|---|
| Economy System | `economy-system.md` | `S2CGoldUpdate { gold, current_mana, reserve_mana, mana_cap }` — unicast to local player | → HUD reads |
| Round State Machine | `round-state-machine.md` | `S2CPhaseChanged { phase, round_number, timer_duration_ms }` — broadcast | → HUD reads |
| Objective System | `objective-system.md` | `ObjectiveDestroyed { target_player_id, lane, was_fake }` — broadcast | → HUD reads |
| Network Protocol | `network-protocol.md` | Defines all four message envelopes above; `S2CGameSnapshot` for reconnect | → HUD reads |
| Game Session System | `game-session-system.md` | Provides `local_id` from LOBBY handshake; sends `S2CGameSnapshot` on reconnect | → HUD reads |

### Upstream — soft (HUD functions without these, but accuracy is limited)

| System | GDD | Interface | Notes |
|---|---|---|---|
| Game Config | `game-config.md` | `lane_count` (5), `fake_count` (2), `mana_cap` (10), `objective_hp` (5) — constants | Used to size the pre-pooled `[[Entity; 5]; 2]` dot array and mana display denominator defaults at session start. If any constant changes at runtime, HUD learns through `S2CGoldUpdate` (mana_cap) or `S2CGameSnapshot` — not directly from config. |

### Sibling — horizontal (same Presentation layer; ownership boundaries must be respected)

| System | GDD | Boundary |
|---|---|---|
| Hand UI | `hand-ui.md` | Owns the PLACEMENT timer (Rule 11) and the hand fan. HUD must NOT duplicate any timer display. Hand UI's bottom-center zone must remain clear of HUD elements. |
| Shop / Auction UI | `shop-auction-ui.md` | Owns DRAFT_INITIAL / DRAFT_SHOP / DRAFT_AUCTION timers and the `local_free_gold` formula (registered). HUD references `local_free_gold` implicitly — see Formulas D.1 — but does not display it as a separate readout. |
| Board Rendering | `board-rendering.md` | Owns in-cell objective sprite rendering and the `was_fake` reveal animation on `ObjectiveDestroyed`. HUD ignores `was_fake` entirely (Section C Rule 7). The two systems share the `ObjectiveDestroyed` message via independent `MessageReader` instances with no coupling. |

### Downstream — none

HUD is a terminal presentation layer. No other system reads HUD state. HUD produces no output messages and no shared resources.

### Bidirectional consistency note

If any of the upstream systems changes a message payload or phase name, this GDD must be updated in the same PR. Specifically:
- Any new `S2CGoldUpdate` field that affects mana display (e.g., `max_mana_override`) requires HUD Rule 3 + Rule 4 update.
- Any new RSM phase requires a row in the Phase label strings table (Rule 5) and a row in the States and Transitions table.
- Any new objective-tier message (e.g., partial-destruction) must be evaluated for HUD scoreboard impact.

## Tuning Knobs

HUD is a display-only system; its tuning knobs are all visual/timing parameters, not game balance levers. All defaults are safe starting values; adjustments affect readability and peripheral feel, not gameplay outcome.

| Knob | Location | Default | Safe Range | Effect of going too high / too low |
|---|---|---|---|---|
| `hud_tween_duration_ms` | `GameConfig` or client constant | 300 ms | 100–500 ms | Too high: gold/mana lags behind real state, player reads stale numbers mid-auction. Too low: snapping feels jarring and visually busy. |
| `hud_margin_px` | Client layout constant | 12 px | 8–24 px | Too small: content near browser chrome clips on tight viewports. Too large: zones crowd inward and risk overlapping sister-UI elements. |
| `hud_bottom_clearance_px` | Client layout constant | 80 px | 60–120 px | Must remain ≥ Hand UI fan height (currently ~72 px). Too small: HUD zones overlap the hand fan. If Hand UI fan height changes, update this in tandem. |
| `hud_dot_diameter_px` | Art Director / client constant | ~16 px | 10–24 px | Too small: DESTROYED dots unreadable at a glance. Too large: scoreboard competes with board and sister UIs for visual weight. |
| `hud_opponent_row_opacity_destroyed` | Art Director / theme | ~0.30–0.40 luminance of background | 0.15–0.55 | Too dark: destroyed dots disappear entirely, losing the "shape still present" quality. Too light: DESTROYED and ALIVE are indistinguishable. |
| `hud_reserved_sublabel_opacity` | Art Director / theme | 0.55–0.60 | 0.40–0.75 | Too low: reserved sub-label disappears under the main number. Too high: reserved competes with main gold for attention. |

**Knobs that are NOT here (delegated to other systems):**
- Timer display values — owned by Hand UI (`placement_timer_seconds`) and Shop/Auction UI (`draft_shop_timer_seconds`, `auction_timer_seconds`)
- Mana cap max value — owned by Economy System / Game Config (`mana_cap`)
- Objective HP — owned by Objective System / Game Config (`objective_hp`)
- Scoreboard dot hue families — owned by Art Director / Art Bible; not a `GameConfig` knob

## Visual/Audio Requirements

*Art Director direction integrated from consultation. Hue families are descriptive intent — exact values are Art Bible's authority.*

### Typography

Font style: **bold, slightly rounded sans-serif** — the Wakfu "clean cartoon stencil" register. Confident stroke weight, no serifs, corners softened just enough to avoid a mechanical feel. Italic used only on the reserved sub-label.

Relative scale hierarchy (base = local gold / local mana numerals):

| Readout | Relative size | Weight / style |
|---|---|---|
| Local gold | 1.0× (base) | Bold upright |
| Local mana `current / mana_cap` | 1.0× | Bold upright |
| Opponent gold | 0.85× | Bold upright — symmetry, not subordination |
| Phase label + round counter | 0.65×, 65% opacity | Regular upright — caption, not title |
| Reserved sub-label (both) | 0.55×, 55–60% opacity | Regular italic |
| Reserve mana label | 0.65× | Regular upright |

**Gold vs. mana hue distinction (peripheral-legibility requirement):** Gold readout uses **warm yellow-white** (Arcane Gold's lightened ivory-warm register, not full saturated gold). Mana readout uses **cool blue-white** (Prism White's cool register, consistent with the Reserve Mana diamond motif). The distinction is warm vs. cool luminance — both remain legible at low attention; the player never needs to read the label to know which resource they're seeing.

### Scoreboard Dots

Dot diameter: approximately **1.5× the cap-height of the round counter numeral** — present but not domineering.

**ALIVE state:** filled circle with a bold, clean outline (Void near-black, ~2px at target resolution). No glow, no rim light. Feels like a printed board-game piece — solid, matter-of-fact.

**DESTROYED state:** hollow ring only (outline preserved, fill removed), tinted to approximately **30–40% luminance of the background panel**. Not pure black — dark enough to read as "gone," not as an empty slot. The outline stays on destruction: same circle, same position, shape evacuated. This structural continuity — ring present, fill absent — is what makes an instantaneous flip read as intentional rather than glitchy.

**Player identity tint:**
- Opponent row (top): **Terracotta / enemy identity** family — warm, adversarial
- Local row (bottom): **Sky Blue / ally identity** family
- Both rows use identical dot size and style; tint alone anchors ownership
- On destruction: both go to the same neutral dark — identity collapses when an objective is lost, which is appropriate

**Transition:** ALIVE → DESTROYED is an **instantaneous state flip**, no tween. The ring-persists-fill-evacuates technique makes the instant transition feel weighty rather than abrupt.

### Reserved-Gold Sub-label

Main gold number: full opacity, base size, bold upright. Reserved sub-label: 55–60% opacity, 0.55× size, italic. Visual contract: *bold upright number = current truth; smaller italic = constraint footnote*. Player reads the big number first, registers the modifier without needing to shift focus.

Optional separator: a faint horizontal divider or bracket notation (`4g reserved`) between the two values to prevent vertical ambiguity.

### Phase Label

Phase label recedes via size and opacity alone (0.65×, 65% opacity). No background chip, no drop shadow, no chip. It reads as a caption. When `GAME OVER` appears: **same typographic treatment** as all other phase labels — no size increase, no dramatic colour shift, no weight change. The visual drama of GAME_OVER belongs to the board (objective burst, reveal wipe, win/loss grade). The HUD label stays consistent; the board carries the finality.

### Audio Events

| Event | Sound feel | Notes |
|---|---|---|
| Phase label change (any transition) | Single dry medium-pitched wood-block tick — one hit, no reverb tail | Marks the moment without announcing it; silent for players not listening |
| Objective dot darkening (DESTROYED) | Short low stone-thud — weight being removed from the table | Not mournful, not alarming; confirms permanent removal |
| Gold tween during RESOLUTION | **Silent** | Board animation track owns all audio space during RESOLUTION; coin ticks would compete with combat sounds |
| GAME_OVER phase transition | Single resolved chord — two or three notes settling to tonic; no fanfare, no sting | Confirms finality without editorialising win/loss; win/loss musical differentiation belongs to the outcome screen, not HUD |
| All other HUD events (mana update, reserve mana, phase label other than GAME_OVER) | **Silent** | Peripheral UI layer must not add audio noise to active decision phases |

📌 **Asset Spec** — Visual/Audio requirements are defined. After the art bible is approved, run `/asset-spec system:hud` to produce per-asset visual descriptions, dimensions, and generation prompts from this section.

## UI Requirements

The HUD's UI architecture is fully specified in Section C (Detailed Design). This section summarises the implementation-facing requirements and flags the UX spec.

### Implementation requirements (bevy_ui)

- HUD is a single root `Node` tree at screen-edge. `PointerEvents::None` on the root so all click events pass through to sister UIs.
- All 10 scoreboard dot entities and all label entities are pre-spawned at session start (`Visibility::Hidden` initially). No per-round spawning.
- Layout uses `Val::Px` anchors for screen-edge positioning. Zones must not use world-space coordinates.
- Text labels use `Text` + `TextFont` + `TextColor` + `Node` (Bevy 0.18 required-component tuple). `LineHeight` is auto-required; override explicitly if needed.
- Scoreboard dots rendered as `Node` with `BackgroundColor` or as small `Sprite`-backed entities — implementation decides, but visual quality must match Art Director spec.
- Numeric tween via `bevy_tweening` on the label's text component or a backing numeric resource; implementation must not tween by spawning new entities.
- System order: `handle_gold_broadcast_system` runs **before** `handle_gold_update_system` in the HUD plugin's system set, enforcing the Rule 11 tie-break.
- `timer_duration_ms` from `S2CPhaseChanged` is explicitly discarded in the HUD system — document with a code comment.

### Screen compatibility targets

| Viewport | Requirement |
|---|---|
| 1280 × 720 | All four zones legible; no overlap with board or sister UIs |
| 1920 × 1080 | Primary design target |
| 2560 × 1440 | Scale correctly; no layout reflow |
| Mobile / narrow (≤900 wide) | Out of scope for hackathon; advisory only |

### UX Spec flag

📌 **UX Flag — HUD**: This system has UI requirements. In Phase 4 (Pre-Production), run `/ux-design` to create a UX spec for the HUD screen (`design/ux/hud.md`) **before** writing epics. Stories that reference HUD layout should cite `design/ux/hud.md`, not the GDD directly.

Note this in the systems index for HUD when it is updated.

## Acceptance Criteria

*18 BLOCKING (automatable, ECS World-based) · 3 ADVISORY (visual/manual check)*

**Core structure**

```
HUD-01: Pre-pooled node tree — BLOCKING
GIVEN the session has started,
WHEN HUD is initialized,
THEN exactly 10 scoreboard dot entities and all label entities exist in the
World before any S2C message is received, and no new HUD entities are spawned
when subsequent update messages arrive.

HUD-02: Fixed four-zone layout — ADVISORY
GIVEN a running match,
WHEN the HUD is visible in ECONOMY_BASIC mode,
THEN top-left, top-center, top-right, and bottom-right zones render in their
designated screen-edge positions with no overlap with board or sister UIs;
confirmed by screenshot at 1280×720 and 1920×1080.

HUD-03: Display format correctness — BLOCKING
GIVEN HUD is in ECONOMY_BASIC mode,
WHEN S2CGoldUpdate arrives with gold=8, current_mana=6, mana_cap=10,
reserve_mana=2, and S2CGoldBroadcast arrives with opponent_gold=6,
THEN local gold label reads "8g", opponent gold label reads "6g",
mana label reads "6 / 10", and reserve label reads "+2 reserve".

HUD-04: Per-message update isolation — BLOCKING
GIVEN HUD is in any visible mode,
WHEN only S2CGoldUpdate arrives and no other messages are processed,
THEN only local gold/mana/reserve label component values change; all other
HUD label components retain their prior values.

HUD-05: Phase label strings — BLOCKING
GIVEN HUD is visible,
WHEN S2CPhaseChanged fires for each RSM phase in sequence,
THEN phase label reads: DRAFT_INITIAL→"DRAFT INITIAL", DRAFT_SHOP→"DRAFT",
DRAFT_AUCTION→"AUCTION", PLACEMENT→"PLACEMENT", RESOLUTION→"RESOLUTION",
GAME_OVER→"GAME OVER"; LOBBY produces no visible phase label (HUD hidden).

HUD-06: Scoreboard dot alive→destroyed — BLOCKING
GIVEN both players start with 5 objectives each (all 10 dots ALIVE),
WHEN ObjectiveDestroyed fires for opponent at lane 3,
THEN opponent dot index 2 (0-indexed) changes to DESTROYED; all other 9
dots remain ALIVE; no real/fake identifier is applied to any dot.

HUD-07: Real/fake identity never shown — BLOCKING
GIVEN any game state including GAME_OVER,
WHEN any scoreboard dot is rendered or any HUD message is processed,
THEN no HUD component holds or displays a value identifying any objective
as real or fake; the only permitted dot states are ALIVE and DESTROYED.

HUD-08: Opponent gold adaptive — BLOCKING
GIVEN HUD is in ECONOMY_BASIC mode,
WHEN S2CGoldBroadcast arrives with opponent_gold=7, opponent_reserved=3,
THEN opponent gold zone displays only "7g" with no reserved sub-label visible.
GIVEN HUD enters ECONOMY_AUCTION mode via S2CPhaseChanged(DRAFT_AUCTION),
THEN the same zone displays "7g" and "3g reserved" as two distinct label values.

HUD-09: RESOLUTION persistence — BLOCKING
GIVEN HUD is visible and sister UIs are visible,
WHEN S2CPhaseChanged fires for RESOLUTION,
THEN HUD remains fully visible at unchanged visual weight, sister UIs hide,
and gold label values update when S2CGoldUpdate messages arrive during resolution.

HUD-10: GAME_OVER freeze — BLOCKING
GIVEN HUD has gold=12g and 7 dots in their current states,
WHEN S2CPhaseChanged fires for GAME_OVER,
THEN no subsequent S2CGoldUpdate or ObjectiveDestroyed changes any HUD
component; phase label reads "GAME OVER"; no real/fake data appears anywhere.

HUD-11: No timer displayed — BLOCKING
GIVEN HUD is in any mode,
WHEN any S2C message arrives,
THEN no HUD entity holds a component or text value representing a countdown
or elapsed time.

HUD-12: Numeric tween duration — ADVISORY
GIVEN HUD is in ECONOMY_BASIC mode,
WHEN a gold or mana value changes,
THEN the displayed value animates to the new number within 300ms; phase label,
round counter, and scoreboard dot transitions are instantaneous (no tween).

HUD-13: Snapshot rebuild — BLOCKING
GIVEN the HUD is in any state (including partially updated or stale),
WHEN S2CGameSnapshot is received,
THEN every HUD zone reflects the values encoded in the snapshot within the
same frame; no entity is despawned or re-spawned (pre-pooled entities reused).

HUD-14: Snapshot rebuild — no flicker — ADVISORY
GIVEN a player reconnects mid-match,
WHEN S2CGameSnapshot is received,
THEN no frame is observed where any zone shows a blank or stale value;
confirmed by screenshot sequence immediately after reconnect.
```

**Phase transitions**

```
HUD-15: LOBBY → DRAFT_INITIAL — BLOCKING
GIVEN HUD is in HIDDEN mode,
WHEN S2CPhaseChanged fires for DRAFT_INITIAL,
THEN HUD becomes visible (ECONOMY_BASIC), phase label reads "DRAFT INITIAL",
round counter shows the current round number, and all 10 dots are ALIVE.

HUD-16: RESOLUTION → DRAFT_SHOP (non-auction) — BLOCKING
GIVEN HUD is in ECONOMY_BASIC mode after a non-auction RESOLUTION,
WHEN S2CPhaseChanged fires for DRAFT_SHOP,
THEN phase label reads "DRAFT", opponent gold zone shows a single gold
value with no reserved sub-label, and HUD mode remains ECONOMY_BASIC.

HUD-17: RESOLUTION → DRAFT_AUCTION — BLOCKING
GIVEN HUD is in ECONOMY_BASIC mode,
WHEN S2CPhaseChanged fires for DRAFT_AUCTION,
THEN HUD enters ECONOMY_AUCTION, phase label reads "AUCTION", and both
opponent gold zone and local gold zone gain their "Xg reserved" sub-labels.

HUD-18: PLACEMENT → RESOLUTION — BLOCKING
GIVEN HUD is in ECONOMY_BASIC mode with phase label "PLACEMENT",
WHEN S2CPhaseChanged fires for RESOLUTION,
THEN phase label reads "RESOLUTION", HUD remains fully visible,
and no zone is hidden; gold-delta ticks continue to be received.

HUD-19: RESOLUTION → GAME_OVER — BLOCKING
GIVEN HUD is in ECONOMY_BASIC mode with phase label "RESOLUTION",
WHEN S2CPhaseChanged fires for GAME_OVER,
THEN phase label reads "GAME OVER", HUD transitions to FROZEN mode,
and no subsequent message alters any displayed value.
```

**Economy and tie-break**

```
HUD-20: Same-tick S2CGoldUpdate + S2CGoldBroadcast tie-break — BLOCKING
GIVEN S2CGoldUpdate with local_gold=15 and S2CGoldBroadcast with a
conflicting local_gold field queue in the same ECS tick,
WHEN all HUD update systems complete,
THEN the local gold label reads "15g", confirming the system execution order
declared in HudPlugin resolves the conflict in favour of S2CGoldUpdate.

HUD-21: Mana cap denominator update — BLOCKING
GIVEN the mana display reads "4 / 8",
WHEN S2CGoldUpdate arrives with current_mana=4, mana_cap=10,
THEN the mana label updates to "4 / 10"; numerator is unchanged;
reserve label remains hidden if reserve_mana=0.
```

## Open Questions

| ID | Question | Owner | Status |
|---|---|---|---|
| OQ-HUD-01 | **Disconnect/pause indicator.** When a player is in the 30-second disconnect grace window, the game is paused server-side. HUD has no message to consume for "session paused" state — `S2CSessionPaused` / `S2CSessionResumed` are not in the network registry. Without such a message, HUD cannot show a "Waiting for opponent…" badge. Should the Network Protocol GDD define this message, and should HUD own it or a separate notification layer? | Network Protocol GDD + Game Session System GDD | Open |
| OQ-HUD-02 | **Local player real/fake opt-in display.** Section C Rule 7 hides real/fake on the local row to prevent screen-share leaks. A potential M3 settings flag could let the player reveal their own real/fake on their own scoreboard row (info they already know). Is this worth designing in M3, or cut entirely? | Design / M3 scope review | Open |
| OQ-HUD-03 | **GAME_OVER summary screen.** Section C Rule 10 defers retroactive real/fake revelation to a post-game summary screen not yet specified. When is this GDD'd? Does HUD hand off any state to the summary screen, or does the summary screen rebuild from `S2CGameSnapshot`? | Future GDD (post-game flow / M3) | Open |
