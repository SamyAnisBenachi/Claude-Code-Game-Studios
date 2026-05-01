# HUD

> **Status**: Approved — 2026-04-30 (Pass 4: 1 blocker resolved — HUD-01 entity count corrected 16→18 [TextSpan children]; 4 recommended fixes: GoldDisplayState is_populated unified in Rule 1, Rule 11 .before() terminology, OQ-HUD-01 pre-implementation gate note, LOBBY audio row + D.1 type note)
> **Author**: SamyAnisBenachi + Claude Code agents
> **Last Updated**: 2026-05-02
> **Implements Pillar**: No idle spectating · Simple surface

## Overview

The HUD is the client-side persistent readout layer that surfaces the game's economic and tactical state to both players at all times. Where Hand UI owns the hand fan and the PLACEMENT timer, Shop/Auction UI owns the shop slots and the DRAFT/auction timers, and Board Rendering owns the lanes, units, and in-cell objective sprites, the HUD owns what is left over and always-on: the local player's **gold**, **current mana / mana cap**, and **reserve mana**; the opponent's **gold**; the **round number** and a compact **phase label**; and the **objective scoreboard** — five status dots per side summarising which of the ten visible objectives are still standing without revealing which were real or fake. The HUD subscribes to three server-to-client signals — `S2CGoldUpdate` (unicast, own economy), `S2CGoldBroadcast` (broadcast, both players' gold + reserved_gold), and `S2CPhaseChanged` (round + phase) — plus the client-internal Bevy `Message` `HudObjectiveUpdate` written by Board Rendering after it drains `ObjectiveDestroyed` from Lightyear; see Rule 6 and Interactions. It rebuilds its full display from `S2CGameSnapshot` on reconnect. It produces no client-to-server messages: the HUD is read-only, server-authoritative, and never asserts state.

During DRAFT_AUCTION, each gold label switches to an **inline parenthetical format** — `11g (4r)` — showing total gold and reserved gold on the same line. The player computes free gold (`11 − 4 = 7`) from one glance at one label without refocusing. Outside DRAFT_AUCTION, labels read `11g`. The top-right zone is always two lines (own gold on line 1, opponent gold on line 2) in every phase — no sub-labels, no format-switching of zone height.

The HUD is what makes "no idle spectating" enforceable across the whole round, not just during decisions the player owns. During the opponent's auction the local player has no input, but the HUD shows the opponent's gold (and their reserved amount), the local player's own gold (and their own reserved), and the round number — live data points that turn a passive watch into an information read. During RESOLUTION, sister UIs disappear; the HUD persists, and gold/objective deltas tick visibly so the consequences of the round are written into the readout the player carries into the next DRAFT. The HUD's job is to be peripheral but never absent: glanceable, never demanding attention, but always answering the same three questions — *what do I have*, *what do they have*, *how close is anyone to winning*.

## Player Fantasy

**The HUD is what you never look at, and always know.**

If Hand UI is the war map and the Board is the place the opponent cannot lie, the HUD is the corner of your eye that is doing math while you stare at the auction. It serves the fantasy of **peripheral omniscience under an information war** — the four numbers and ten dots that compose this game's economic and tactical truth, broadcast at the screen's edge so quietly that the player learns to trust them without ever taking their gaze off the central decision. By round six, a competent player tallies their opponent's free gold without focusing on it; by round twelve, they always know which round it is, how much reserve mana they hold, and how many dots are still standing on each side — without any of those readings consuming a moment of central attention. The HUD's signature feeling is not surprise or revelation; it is the quiet absence of surprise — every read confirms what was already half-known.

**The anchor moment.** Opponent's auction, round 9. The auction panel is climbing — 7g, 8g, 9g — and your central gaze is locked on the bid number, calculating whether to raise. But you already know, without ever looking down, that their gold reads `9g (3r)` — nine total, three reserved — and yours reads `11g (4r)`. Two labels, top-right, same corner, same glance: free gold is 6 for them, 7 for you. You raise to 10. They fold. Your hunt was complete before you committed — and the HUD never asked for your eyes.

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

*Pillar alignment: **Auction as signature** — the inline parenthetical `Xg (Yr)` format makes the auction's bluffs legible in a single glance; two values, one line, no eye travel. **No idle spectating** — peripheral reading converts every opponent-turn moment into active intelligence gathering. **Simple surface** — five dots per side, alive or dead, one rule; gold is gold; mana is mana — the HUD adds no second-order logic on top of the data it surfaces.*

*Design note — opponent mana omission (intentional):* The HUD shows opponent gold but not opponent mana. This is intentional design: gold visibility is required for the auction pillar (an ascending open-bid auction with hidden gold is a guessing game, not a negotiation). Mana is hidden because deployment is the game's strategic layer — knowing the opponent's mana pool precisely would eliminate the placement bluff. The information asymmetry is deliberate: the auction has no secrets; deployment keeps some.

## Detailed Design

### Core Rules

**Rule 1 — Pre-pooled HUD node tree.**
All HUD elements — phase label, round counter, own gold label, opponent gold label, mana label, reserve mana label, and the 10 scoreboard dot entities (5 opponent + 5 local) — are spawned at session start as children of a single root `Node` and toggled via `Visibility::Visible` / `Visibility::Hidden`. No per-round spawn or despawn; no per-update entity creation. There are **no separate "reserved gold" sub-label entities** — reserved gold is encoded inline in the gold label strings during ECONOMY_AUCTION mode (Rule 3). Each gold label entity carries a backing `GoldDisplayState { gold: f32, reserved_gold: f32, is_populated: bool }` component that holds the canonical numeric values (`is_populated` distinguishes the cold-start `"--g"` placeholder from a legitimate `0g` value — see Edge Cases); the displayed string is re-derived from this component by a change-detection system each frame during a tween. The root `Node` carries `PickingBehavior { should_block_lower: false, is_hoverable: false }` guarded with `#[cfg(feature = "ui_picking")]` (Bevy 0.18 renames the Cargo feature from `bevy_ui_picking_backend` to `ui_picking`; inserting `PickingBehavior` without the feature compiled in causes a runtime panic — component not registered) so HUD never captures click events; sister UIs receive all input. If `ui_picking` is not in `Cargo.toml`, the root `Node` is already non-interactive and no picking component is needed.

**Rule 2 — Screen placement (fixed for the entire match).**
The HUD occupies four screen-edge zones, each anchored with `Val::Px(12.0)` from the screen edges. Layout never reflows during a match:

| Zone | Contents |
|---|---|
| Top-left | Phase label (line 1) + round counter (line 2) |
| Top-center | Scoreboard — 2 rows × 5 dots; top row = opponent; bottom row = local; dots aligned horizontally to lane midpoints 1–5 using the session-scoped `BoardLayout` resource owned by Board Rendering — see Dependencies |
| Top-right | Own gold (line 1); Opponent gold (line 2). Always 2 lines. During DRAFT_AUCTION, each label switches to inline parenthetical format `Xg (Yr)` — no zone height change (see Rule 3) |
| Bottom-left | Local mana (line 1, format `current / mana_cap`); local reserve mana (line 2, hidden when reserve_mana == 0). Vertical height is always reserved for 2 lines to prevent layout shift when the reserve mana label appears |

Bottom-center is reserved for the Hand UI fan and must remain clear of all HUD elements. Center-screen is reserved for Shop/Auction UI panels and the DRAFT_INITIAL grid.

**Rule 3 — Display formats (exact strings).**

| Readout | ECONOMY_BASIC format | ECONOMY_AUCTION format | Example (AUCTION) |
|---|---|---|---|
| Phase label | `<PHASE_STRING>` (see Rule 5) | same | `AUCTION` |
| Round counter | `R<round_number>` | same | `R9` |
| Own gold | `<gold>g` | `<gold>g (<reserved_gold>r)` | `11g (4r)` |
| Own mana | `<current_mana> / <mana_cap>` | same | `6 / 10` |
| Reserve mana | `+<reserve_mana> reserve` (hidden when == 0) | same | `+2 reserve` |
| Opponent gold | `<gold>g` | `<gold>g (<reserved_gold>r)` | `8g (3r)` |

The `(Yr)` parenthetical suffix on gold labels is rendered as a second text span on the same label entity at 65% opacity (see Visual/Audio Requirements). It is present only in ECONOMY_AUCTION mode; in ECONOMY_BASIC the label reads `Xg` with no suffix. When `reserved_gold == 0` during DRAFT_AUCTION, the label reads `Xg (0r)` — the parenthetical is shown even at zero to signal that the mode has changed and that the player can expect the value to change as bids are placed.

Mana label always shows two numbers separated by ` / `, even when `current_mana == mana_cap` (i.e. `10 / 10`, not `MAX`). The cap can change mid-match from fake-objective rewards — a single-number display would create a false impression of cap stability.

**Rule 4 — Update triggers (per-message contract).**
Each HUD readout is updated only by the message(s) below; HUD never redraws the full readout tree, only the affected text/visibility:

| Message / Signal | Updates |
|---|---|
| `S2CGoldUpdate { gold, current_mana, reserve_mana, mana_cap }` | Own gold label `GoldDisplayState.gold` (triggers label re-render); own mana label numerator + denominator; reserve mana label visibility + value |
| `S2CGoldBroadcast { player_id, gold, reserved_gold }` | If `player_id == opponent_id`: opponent gold label `GoldDisplayState { gold, reserved_gold }` (full refresh). If `player_id == local_id`: own gold label `GoldDisplayState.reserved_gold` only (see Rule 11) |
| `S2CPhaseChanged { phase, round_number, timer_duration_ms }` | Phase label, round counter, mode transitions per Rule 5. **`timer_duration_ms` is explicitly ignored** — see Rule 12 |
| `HudObjectiveUpdate { target_player_id, lane }` (client-internal Bevy `Message`) | Single dot at `(target_player_id, lane)` transitions ALIVE → DESTROYED. Note: this is written by Board Rendering after it drains `ObjectiveDestroyed` — see Rule 6 and Interactions |
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
| GAME_OVER | `GAME OVER` (phase label only — no round number appended; round counter element on line 2 remains visible per `round_counter_visible` predicate) | `FROZEN` |

On `S2CPhaseChanged(DRAFT_AUCTION)`: HUD enters `ECONOMY_AUCTION` mode — both gold label entities switch to the `Xg (Yr)` inline parenthetical format. The `(Yr)` suffix is appended to the label string immediately on phase entry; both labels will read `Xg (0r)` until the next `S2CGoldBroadcast` carries non-zero reserved values (server invariant: reserved_gold == 0 at DRAFT_AUCTION entry, so `(0r)` is correct). On any phase exit from DRAFT_AUCTION: both gold labels revert to `Xg` format (parenthetical stripped). Phase label and round counter update atomically on the message — no fade, no animation.

**Rule 6 — Scoreboard layout and state.**
Two horizontal rows of 5 dot entities are pre-spawned. Indexed by `(player_id, lane)` into a fixed `[[Entity; 5]; 2]` array — O(1) lookup, no query needed. Row mapping: index `[0]` = opponent (top row); index `[1]` = local (bottom row). Lane mapping: `dots[player][lane - 1]` for lanes 1..=5. Each dot has two states:

- **ALIVE** — filled circle, full opacity, neutral high-contrast tint (Art Director defines exact hue)
- **DESTROYED** — hollow ring only, reduced opacity; same visual for real and fake

ALIVE → DESTROYED is an instantaneous state flip on `HudObjectiveUpdate` receipt. No tween, no animation. Board Rendering owns the in-cell objective shatter animation; the HUD scoreboard is a permanent record, not a reaction surface.

**ObjectiveDestroyed fanout architecture.** HUD does NOT subscribe to `MessageReceiver<ObjectiveDestroyed>` (Lightyear channel). Board Rendering is the **sole drain** of `ObjectiveDestroyed` via its own `MessageReceiver`. After draining, Board Rendering writes `HudObjectiveUpdate { target_player_id: PlayerId, lane: u8 }` through `MessageWriter<HudObjectiveUpdate>`, stripping `was_fake` before writing and architecturally enforcing Rule 7. `HudPlugin` reads only `MessageReader<HudObjectiveUpdate>` in an explicitly ordered presentation system. `HudObjectiveUpdate` is registered once with `app.add_message::<HudObjectiveUpdate>()` from the client presentation composition layer before Board Rendering writes or HUD reads the message. The Board Rendering write system and HUD read/apply system must be ordered so the dot state transition occurs in the same ECS tick. This eliminates the double-drain bug that would occur if two systems attempted to drain the same Lightyear channel. The `HudObjectiveUpdate` message type lives in the client crate's presentation/UI shared module, accessible to both Board Rendering and HudPlugin (OQ-HUD-05 resolved).

**Rule 7 — Real/fake identity is never shown on the scoreboard.**
The HUD's scoreboard renders all 5 dots identically per side regardless of real/fake assignment. `HudObjectiveUpdate` carries only `target_player_id` and `lane` — it strips `was_fake` at the Board Rendering boundary, so HUD cannot read this field even if future contributors wanted to. The scoreboard's contract with the player is *alive vs destroyed*, not *real vs fake* — that distinction belongs to the destruction reveal animation owned by Board Rendering, and to the destroyed-attacker's awareness in their own UI feedback (handled outside HUD scope).

**Rule 8 — Opponent gold display, adaptive by phase.**
In ECONOMY_BASIC mode, the opponent gold label displays `"<gold>g"` — a single value updated from `S2CGoldBroadcast { player_id: opponent_id, gold, reserved_gold }`. The `reserved_gold` field updates `GoldDisplayState.reserved_gold` on the opponent entity but is not rendered in ECONOMY_BASIC format. On entry to ECONOMY_AUCTION (Rule 5), the label switches to `"<gold>g (<reserved_gold>r)"`. On DRAFT_AUCTION exit, the label reverts to `"<gold>g"` regardless of `reserved_gold` value — the server clears bids as part of auction settlement.

*Design note — opponent mana not shown:* The HUD shows opponent gold but not opponent mana. See Player Fantasy section for rationale (intentional information asymmetry).

**Rule 9 — RESOLUTION: HUD persists, sister UIs hide.**
On `S2CPhaseChanged(RESOLUTION)`: HUD remains fully visible at exactly the same visual weight it held during PLACEMENT (no scaling up, no opacity shift, no contrast change — that would be a "moving target" violation of the player fantasy). Hand UI hides immediately (Hand UI Rule 12); Shop/Auction UI hides its panels. The HUD is the only persistent UI surface during RESOLUTION. Gold deltas (kill rewards, objective rewards, embedded `GoldAwarded` entries from `S2CResolutionEvent`) arrive as `S2CGoldUpdate` and `S2CGoldBroadcast` and update the readouts in real time per Rule 4. Numeric tweens are ≤300ms (see Rule 14). Dot darkenings on `HudObjectiveUpdate` fire instantly (Rule 6).

**Rule 10 — GAME_OVER: HUD freezes, never reveals identity retroactively.**
On `S2CPhaseChanged(GAME_OVER)`: phase label updates to `GAME OVER`. All readouts retain their last received state; the round counter remains visible showing the final round number. The scoreboard does **not** retroactively reveal real/fake on either side — destroyed dots remain "destroyed", alive dots remain "alive", with no identity glyph added. A separate post-game summary screen (not owned by HUD) may reveal the full objective map, but the HUD's contract holds: the dots mean *alive vs destroyed*, end of match included. A win/loss overlay renders above the HUD; the HUD remains visible beneath it as a final-state record.

**`S2CGameSnapshot` bypass for FROZEN mode.** If a snapshot arrives while the HUD is in FROZEN state (e.g., the local player disconnected and reconnected at GAME_OVER), the snapshot rebuild runs per Rule 13 regardless of FROZEN mode. After the rebuild completes, HUD immediately re-enters FROZEN. No incremental updates (`S2CGoldUpdate`, `HudObjectiveUpdate`) are accepted after re-entry. This ensures a reconnecting player sees the correct authoritative final state rather than a blank or stale display. This is the explicit tiebreak: **snapshots always win, then FROZEN re-applies**.

**Rule 11 — Tie-break: `S2CGoldUpdate` vs `S2CGoldBroadcast` for own gold display.**
Both messages contribute to the own gold label. `S2CGoldUpdate` owns the `gold` value; `S2CGoldBroadcast` (when `player_id == local_id`) owns the `reserved_gold` value. They write to separate fields of the `GoldDisplayState` component on the own gold label entity. When both arrive in the same Bevy tick:
1. `handle_gold_broadcast_system` runs first (scheduled `.before(handle_gold_update_system)`) — writes opponent gold + reserved (both fields), and writes own `GoldDisplayState.reserved_gold` only.
2. `handle_gold_update_system` runs second (system order: higher priority) — writes own `GoldDisplayState.gold`. It does NOT touch `GoldDisplayState.reserved_gold`.
3. The label re-renders from the `GoldDisplayState` component after both systems complete, producing `{gold from S2CGoldUpdate}g ({reserved_gold from S2CGoldBroadcast}r)` in ECONOMY_AUCTION mode.

This ordering is enforced via `app.configure_sets` or explicit `.after()` dependency between the two HUD systems and is a code contract, not an optional optimisation.

**Field-split ordering proof:** The Rule 11 result is correct regardless of Lightyear channel drain order. `S2CGoldUpdate` writes only to `GoldDisplayState.gold`; `S2CGoldBroadcast` writes only to `GoldDisplayState.reserved_gold` (for the own label). Because the two messages write to *separate fields*, no ordering between them can produce a corrupt composite value — the `.after()` constraint is belt-and-suspenders, not the primary correctness mechanism. No drain-order verification against Lightyear 0.26 internals is required.

**Rule 12 — HUD never displays a timer.**
The PLACEMENT timer is owned by Hand UI (hand-ui.md Rule 11). The DRAFT_INITIAL, DRAFT_SHOP, and DRAFT_AUCTION timers are owned by Shop/Auction UI. The `timer_duration_ms` field of `S2CPhaseChanged` is read by those systems, not by HUD. The HUD's `MessageReader<S2CPhaseChanged>` handler must destructure the message with `let S2CPhaseChanged { phase, round_number, .. } = msg;` — the `..` pattern discards `timer_duration_ms` without binding it. This produces an unused-field compile-time note (not an error in Rust unless `#[deny(unused_variables)]` is set), but more importantly documents the boundary for code reviewers.

**Rule 13 — Reconnect: `S2CGameSnapshot` rebuild.**
On receipt of `S2CGameSnapshot`, the HUD reads the embedded economy and scoreboard state in a single synchronous pass and writes to all label entities and all dot entities before the next frame renders. Because all HUD entities are pre-pooled (Rule 1), the rebuild is a series of `GoldDisplayState`, `Text`, and `Visibility` writes with no spawn latency, no flicker. The snapshot is sufficient and authoritative — the HUD does not wait for subsequent messages to populate state after a reconnect. **HIDDEN-mode exception:** if the snapshot arrives while `HudMode == HIDDEN` (i.e., `S2CPhaseChanged(DRAFT_INITIAL)` has not yet been received), the snapshot rebuild applies all values AND transitions the HUD out of HIDDEN mode using `snapshot.phase` to determine the correct visible mode — treating the snapshot as an implicit phase-change signal. This ensures a cold-start client that receives a snapshot before any `S2CPhaseChanged` still renders the HUD correctly. Phase label and round counter populate from the snapshot's `phase` and `round_number` fields, which advance the HUD into the correct mode (Rule 5) atomically. **FROZEN mode exception:** if the snapshot arrives while HUD is in FROZEN state (GAME_OVER reconnect), the rebuild runs to completion, then HUD immediately re-enters FROZEN — see Rule 10.

**Rule 14 — Animation budget.**
Numeric value updates (gold and reserved_gold, mana, reserve mana) tween over ≤300ms via `bevy_tweening` using a **backing numeric resource**: each tweened label carries a `GoldDisplayState { gold: f32, reserved_gold: f32 }` component (or equivalent per-readout `f32` backing field). The tween animates the `f32` value; a change-detection system reads the current backing value each frame and writes the formatted string to the `Text` component. **When multiple updates for the same readout arrive in the same Bevy tick** (e.g., a lag burst delivering 3 `S2CGoldUpdate` messages at once), collapse to the **last value only** before initiating a single tween — do not start a tween for each message in sequence. Phase label, round counter, and dot state changes are **not** animated — text replaces in place, dot darkens instantly. No flashing, no pulsing, no urgency colours, no scale tweens larger than ±1 pixel. Animations that obscure or compete with the central decision (auction panel, hand fan, board) are forbidden.

**In-flight tween cancellation:** If a new authoritative value arrives while a tween is in progress, cancel the current tween and start a new tween from the current backing `f32` value (wherever the interpolation has reached) to the new authoritative value. Duration remains ≤300ms. This is trivially safe because the canonical value lives in `GoldDisplayState.gold`, not in the string — no string-parsing is needed.

### States and Transitions

| RSM Phase | HUD Mode | Visibility | Transitions in / out |
|---|---|---|---|
| `LOBBY` | `HIDDEN` | All elements `Visibility::Hidden` | Out: on `S2CPhaseChanged(DRAFT_INITIAL)` — all elements made visible, populated by next snapshot or `S2CGoldUpdate` |
| `DRAFT_INITIAL` | `ECONOMY_BASIC` | All readouts visible; both gold labels show `Xg` format; all 10 dots ALIVE | In: from `LOBBY` on `S2CPhaseChanged(DRAFT_INITIAL)`. **Occurs exactly once per match (round 1 only — per RSM Rule 1, RESOLUTION never transitions back to DRAFT_INITIAL; subsequent rounds use DRAFT_SHOP or DRAFT_AUCTION).** Out: to `PLACEMENT` |
| `DRAFT_SHOP` | `ECONOMY_BASIC` | Same as above | In: from `RESOLUTION` (non-auction rounds) or from `DRAFT_AUCTION` |
| `DRAFT_AUCTION` | `ECONOMY_AUCTION` | All readouts visible; **both gold labels switch to `Xg (Yr)` inline parenthetical format** (zone height unchanged — always 2 lines) | In: from `RESOLUTION` (auction rounds 3, 6, 9…). Out: to `DRAFT_SHOP` — both gold labels revert to `Xg` format |
| `PLACEMENT` | `ECONOMY_BASIC` | Same as DRAFT_SHOP; sister Hand UI shows placement timer (HUD does not) | In: from `DRAFT_INITIAL` or `DRAFT_SHOP`. Out: to `RESOLUTION` |
| `RESOLUTION` | `ECONOMY_BASIC` | HUD remains fully visible at unchanged weight; sister UIs hide | In: from `PLACEMENT`. Out: to `DRAFT_AUCTION`, `DRAFT_SHOP`, or `GAME_OVER` |
| `GAME_OVER` | `FROZEN` | All readouts retain last state; phase label = `GAME OVER`; round counter remains visible showing final round number; no incremental updates (`S2CGoldUpdate`, `HudObjectiveUpdate`) accepted. `S2CGameSnapshot` bypasses FROZEN and triggers a full rebuild per Rule 10/13 tiebreak, then HUD immediately re-enters FROZEN. | In: from `RESOLUTION`. Terminal — no out |

### Interactions with Other Systems

**`S2CGoldUpdate` (own economy — network-protocol.md, source: economy-system.md).**
Single subscriber: `HudPlugin`'s `MessageReader<S2CGoldUpdate>`. Update granularity: writes `GoldDisplayState.gold` on own gold label entity (triggering format refresh); updates mana label numerator + denominator; toggles reserve label visibility on `reserve_mana == 0` boundary. Authority: this is the local player's authoritative gold/mana state — wins ties against `S2CGoldBroadcast` per Rule 11.

**`S2CGoldBroadcast` (both players — network-protocol.md).**
Single subscriber: `HudPlugin`'s `MessageReader<S2CGoldBroadcast>`. Update granularity: if `player_id == opponent_id`: writes `GoldDisplayState { gold, reserved_gold }` on opponent gold label entity (full refresh). If `player_id == local_id`: writes `GoldDisplayState.reserved_gold` on own gold label entity only (`GoldDisplayState.gold` is owned by `S2CGoldUpdate` — see Rule 11). **Mode-independence contract:** `handle_gold_broadcast_system` always writes to `GoldDisplayState` fields unconditionally regardless of current `HudMode` — only the rendered string output is mode-gated. This ensures that a broadcast arriving one tick before `S2CPhaseChanged(DRAFT_AUCTION)` preserves `reserved_gold` in the struct and renders correctly once the phase transition sets `gold_label_format = AUCTION_FORMAT`. The `local_free_gold` formula registered in `entities.yaml` (`gold - reserved_gold`, source: shop-auction-ui.md) is referenced *implicitly* by HUD: the player reads free gold by mentally subtracting the `r` value from the `g` value in the inline label.

**`S2CPhaseChanged` (phase + round — network-protocol.md, source: round-state-machine.md).**
Single subscriber: `HudPlugin`'s `MessageReader<S2CPhaseChanged>`. Update granularity: phase label text + round counter text + mode transition (Rule 5) + gold label format switch (ECONOMY_BASIC ↔ ECONOMY_AUCTION). The `timer_duration_ms` field is discarded (Rule 12).

**`HudObjectiveUpdate { target_player_id: PlayerId, lane: u8 }` (client-internal Bevy `Message`, written by Board Rendering).**
Registration: the client presentation composition layer calls `app.add_message::<HudObjectiveUpdate>()` once before Board Rendering writes or HUD reads the message. Board Rendering is the sole `MessageReceiver<ObjectiveDestroyed>` drain on the Lightyear channel. After draining, Board Rendering writes `HudObjectiveUpdate` via `MessageWriter<HudObjectiveUpdate>` (stripping `was_fake`). `HudPlugin` reads it through `MessageReader<HudObjectiveUpdate>`. Update granularity in HUD: one dot entity, indexed `dots[target_player_id][lane - 1]` after bounds validation. The `was_fake` field never reaches HUD — Rule 7 is architecturally enforced. See Rule 6 for message fanout architecture detail.

**`S2CGameSnapshot` (reconnect — network-protocol.md).**
Single subscriber: `HudPlugin`'s `MessageReader<S2CGameSnapshot>`. Triggers full HUD state rebuild per Rule 13. After rebuild, the HUD waits for the next phase/economy/objective messages to drive incremental updates as normal.

**Strict non-ownership boundary.**
HUD MUST NOT:
- Display any countdown timer (Rule 12)
- Subscribe to `C2S*` messages (HUD produces no input messages)
- Subscribe directly to `MessageReceiver<ObjectiveDestroyed>` (Board Rendering is the sole Lightyear drain — see Rule 6)
- Write to any client state other than its own labels and dots
- Compute derived values that could disagree with server (e.g., HUD must not track gold as a delta accumulator — it sets the value from each authoritative message)
- Animate beyond Rule 14's budget

**Registry references (cross-system facts this section consumes):**
- `S2CGoldUpdate`, `S2CGoldBroadcast`, `S2CPhaseChanged`, `S2CGameSnapshot`, `ObjectiveDestroyed` — all registered in `network_messages` section of `entities.yaml`
- `HudObjectiveUpdate` — client-internal Bevy `Message` defined in the client crate's presentation/UI shared module; not a Lightyear channel message
- `local_free_gold` formula (shop-auction-ui.md) — referenced implicitly via inline gold label
- `mana_cap`, `objective_hp`, `lane_count`, `fake_count` — registered constants from game-config.md / board-lane-system.md / objective-system.md
- `BoardLayout` — session-scoped layout resource inserted by `BoardRenderingPlugin` and read by HUD for scoreboard dot horizontal alignment

## Formulas

The HUD performs no game-logic computation — all economic, combat, and objective state arrives pre-computed from the server. This section documents the display formula HUD references and the visibility predicates that drive HUD element show/hide behaviour. No new formulas are introduced; no new constants are defined.

### D.1 — Display formula (implicit, referenced from sibling system)

The HUD renders free gold *implicitly* — the player computes it from the inline parenthetical: `gold = 11, reserved = 4 → free = 7`. The underlying formula:

`local_free_gold = gold - reserved_gold`

**Variables:**

| Variable | Symbol | Type | Range | Description |
|----------|--------|------|-------|-------------|
| gold | `g` | u32 | 0..=u32::MAX (practical: 0..=99) | Player's total gold from `S2CGoldBroadcast.gold` |
| reserved_gold | `r` | u32 | 0..=g (server invariant) | Player's gold reserved on active auction bid. Always 0 outside DRAFT_AUCTION. |

**Output Range:** 0 to gold (always non-negative under server invariant `reserved_gold ≤ gold`).
**Example:** `gold = 11, reserved_gold = 4` ⇒ `local_free_gold = 7`. HUD displays `11g (4r)` in ECONOMY_AUCTION; player reads `free = 7` from the inline label.
**Type note:** The server sends `gold` and `reserved_gold` as `u32` in network messages. The HUD converts to `f32` at the message handler boundary (stored in `GoldDisplayState.gold: f32` and `GoldDisplayState.reserved_gold: f32`) for `bevy_tweening` compatibility.

**Authority:** Owned by `shop-auction-ui.md`, registered in `entities.yaml` formulas section as `local_free_gold`. HUD does not duplicate ownership — it renders both values inline per Rule 3.

### D.2 — Visibility predicates

Each toggleable HUD element has one boolean predicate that determines whether it is rendered or formatted with its expanded form. Predicates are evaluated on every relevant message arrival; no polling.

| Element | Predicate | Source data |
|---------|-----------|-------------|
| Local reserve mana label | `reserve_label_visible := reserve_mana > 0` | `S2CGoldUpdate.reserve_mana` |
| Gold label format | `gold_label_format := if phase == DRAFT_AUCTION then AUCTION_FORMAT else BASIC_FORMAT` | `S2CPhaseChanged.phase` |
| All HUD elements (root visibility) | `hud_visible := phase != LOBBY` | `S2CPhaseChanged.phase` |
| Round counter | `round_counter_visible := phase != LOBBY` | `S2CPhaseChanged.phase` |

**Notes:**
- There are no separate "reserved gold" label entities. Reserved gold appears as the `(Yr)` parenthetical in the gold label string when `gold_label_format == AUCTION_FORMAT`; it does not have its own visibility predicate.
- `gold_label_format` is re-evaluated on every `S2CPhaseChanged` and on `S2CGameSnapshot` (Rule 13). When switching to `BASIC_FORMAT`, gold labels render `Xg`; when switching to `AUCTION_FORMAT`, gold labels render `Xg (Yr)` — both using the current `GoldDisplayState` values.
- `reserve_label_visible` is re-evaluated on every `S2CGoldUpdate`.
- No predicate combines data from two messages — each is a single-field test.
- `round_counter_visible` evaluates to `true` during `GAME_OVER` — the round counter remains visible in FROZEN mode as part of the final-state record.

### D.3 — Dot state mapping

Each scoreboard dot has a state derived from a single per-player-per-lane boolean stored in HUD-local state (initialised to ALIVE for all 10 dots at session start, then updated by `HudObjectiveUpdate` per Rule 6):

`dot_state(player, lane) := if destroyed[player][lane] then DESTROYED else ALIVE`

**Variables:**

| Variable | Type | Source |
|----------|------|--------|
| `destroyed[player][lane]` | `[[bool; 5]; 2]` (HUD-local state) | Set true on `HudObjectiveUpdate { target_player_id: player, lane }`; reset to false on `S2CGameSnapshot` rebuild per the snapshot's objective state |

**Output Range:** `{ ALIVE, DESTROYED }`.
**Note:** No real/fake distinction enters this formula. The `was_fake` field of `ObjectiveDestroyed` is stripped by Board Rendering before writing `HudObjectiveUpdate` (Rule 7).

## Edge Cases

The HUD is a read-only display layer; most "edge cases" reduce to defensive rendering against malformed messages or boundary states. Each case names the exact condition and the exact resolution; under no condition does the HUD modify game state, abort the session, or panic.

### Cold start and first-message timing

- **If `S2CPhaseChanged(DRAFT_INITIAL)` arrives before any `S2CGoldUpdate`**: display `--g` for own gold and `-- / --` for mana with the `+N reserve` label hidden. Do not show `0g` or `0 / 10` — those are valid runtime values and would teach a wrong mental model. On first `S2CGoldUpdate` receipt, replace placeholders by updating `GoldDisplayState.gold` and re-rendering (no tween — there is no "old value" to animate from).
- **If `S2CPhaseChanged(DRAFT_INITIAL)` arrives before any `S2CGoldBroadcast` for the opponent**: display `--g` for opponent gold. Same rationale as own gold — `0g` is a valid runtime value and must not appear before the server has confirmed opponent economy state. The `GoldDisplayState.gold` on the opponent entity should be initialised to a sentinel value (e.g. `f32::NAN` or a separate `is_populated: bool` flag) so the formatter knows to render `--g` instead.
- **If `S2CGoldBroadcast` arrives before `local_id` has been established by the LOBBY handshake**: discard the message. Do not attempt to render either row — displaying the wrong player's gold in the wrong zone is a worse failure than a momentary blank. The next `S2CGameSnapshot` or post-handshake broadcast will populate the readouts authoritatively.
- **If `S2CGameSnapshot` arrives as the first message after a fresh load (cold start)**: this is the canonical recovery path. Pre-pooled Node tree (Rule 1) is always present at startup; the snapshot writes phase, round, both economies, and the `destroyed[][]` state in one synchronous pass per Rule 13.

### Reconnect and replay

- **If `S2CGameSnapshot` arrives while phase is currently DRAFT_AUCTION**: rebuild HUD from snapshot. The snapshot's `PlayerSnapshot` will include `gold` and `reserved_gold` for both players once the pending NP GDD amendment is applied (see Dependencies — bidirectional consistency note). Write both fields to `GoldDisplayState` for both players; the format is AUCTION_FORMAT (phase == DRAFT_AUCTION from snapshot). Note: `reserved_gold` values from the snapshot may be stale by seconds in an active auction; subsequent `S2CGoldBroadcast` messages will carry live values and correct the display.
- **If `S2CGameSnapshot` arrives while phase is currently GAME_OVER** (player reconnected at end-of-game): the snapshot rebuild runs per Rule 13, overriding FROZEN mode. This is the explicit tiebreak — **snapshots always win, then FROZEN re-applies**. After the rebuild completes, HUD immediately re-enters FROZEN; no incremental updates are accepted afterward.
- **If `HudObjectiveUpdate` arrives for `(player, lane)` whose `destroyed[player][lane]` is already `true`**: no-op. Idempotent. Log a warning (likely a server replay or reconnect artifact). Do not re-trigger the dot transition.
- **If two `S2CPhaseChanged` are processed in the same Bevy tick**: last-write-wins — the second message overwrites the first. RSM is a strict sequence; two phases in one tick implies a reconnect artifact, and the later message is authoritative.

### Same-tick message arrival

- **If `S2CPhaseChanged(DRAFT_AUCTION)` and `S2CGoldBroadcast` arrive in the same Bevy tick**: the phase system writes `gold_label_format = AUCTION_FORMAT` (gold labels now render `Xg (Yr)`). The broadcast updates `GoldDisplayState.reserved_gold`. Both contribute to the final rendered label. If `S2CPhaseChanged` runs before `S2CGoldBroadcast` in the same tick, the label renders `Xg (0r)` momentarily — then immediately re-renders to `Xg (Nr)` when the broadcast updates the `GoldDisplayState`. This is a within-tick re-render, not a visible flicker, because both systems run before the frame is rendered to screen.
- **If `S2CGoldUpdate` and `S2CGoldBroadcast` for the local player arrive same tick with different `gold` values**: `S2CGoldUpdate` wins for `GoldDisplayState.gold` (Rule 11). `S2CGoldBroadcast` updates only `GoldDisplayState.reserved_gold`; its `gold` field is ignored for the own label.
- **Multiple `S2CGoldUpdate` messages in the same tick** (lag burst): apply only the last value per Rule 14 — collapse to the final message before initiating a single tween.

### Server invariant violations (defensive)

- **If `S2CGoldBroadcast.reserved_gold > gold`**: clamp `GoldDisplayState.reserved_gold` to `gold` for display. Log a warning. The server invariant forbids this; the guard prevents `"11g (13r)"` (free gold appearing negative).
- **If `S2CGoldUpdate.mana_cap == 0`**: display `current_mana / 0` as received and log a warning. The HUD does no division — only string concatenation. The server invariant forbids `mana_cap == 0`; the guard prevents a panic, not a malformed display.
- **If `S2CGoldUpdate.mana_cap < current_mana` (overfull mana)**: display `current_mana / mana_cap` as received (e.g. `"3 / 2"`). Overfull mana is a transient legal state during economy recalculation that resolves within 1–2 frames when the server sends a correcting update. The HUD is authoritative-display-only; it shows the server value exactly. A player reading `"3 / 2"` during this transient sees an unusual number that confirms the server is processing a recalculation — this is the truthful read. Do not clamp the numerator; clamping would display a corrected-looking `"2 / 2"` that hides the real state.
- **If `HudObjectiveUpdate.lane` is outside `1..=5` or `target_player_id` is unknown**: validate bounds **before** computing the index. In Rust, `lane - 1` on a `u8` value of 0 underflows (panics in debug builds). The required guard is: `if !(1..=5).contains(&lane) { /* log warning, return */ }` — this check must precede any `dots[player][lane - 1]` access.
- **If `S2CGoldBroadcast.player_id` matches neither `local_id` nor `opponent_id`**: ignore and log. Cannot occur in 1v1 mode under current GDD rules.

### Animation and visual transitions

- **If a numeric tween is in-flight when a new value arrives**: cancel the tween, start a new tween from the current `f32` value in `GoldDisplayState` (wherever the interpolation reached) to the new authoritative value. Duration remains ≤300ms. No string parsing needed — the backing resource holds the canonical float.
- **If `S2CPhaseChanged(GAME_OVER)` fires while a numeric tween is in-flight**: snap immediately to the authoritative final value (write directly to `GoldDisplayState`), cancelling the tween. A frozen HUD (Rule 10) must show the final authoritative number, not a mid-interpolation value.
- **Multiple same-readout updates in one tick**: collapse to last value only, then start one tween (Rule 14). Do not start and immediately cancel multiple tweens for intermediate values.

### Layout and viewport

- **If the browser window is resized mid-match**: bevy_ui's `Val::Px` anchors and layout system reflow automatically. HUD layout is fixed *relative to screen edges* (Rule 2), not in absolute world coordinates. Verified at 1280×720, 1920×1080, and the project's WASM target viewport.

### Cross-system gap (flagged for Open Questions)

- **Disconnect grace / paused-game indicator**: When a player is in the `disconnect_grace_seconds = 30` window, the game is paused server-side. `S2CSessionPaused` / `S2CSessionResumed` are not in the network registry. **Without a paused-state message, the HUD cannot show a "Waiting for opponent…" overlay.** This is forwarded to Open Questions for Network Protocol / Game Session System resolution.

## Dependencies

### Upstream — hard (HUD cannot function without these)

| System | GDD | Interface | Direction |
|---|---|---|---|
| Economy System | `economy-system.md` | `S2CGoldUpdate { gold, current_mana, reserve_mana, mana_cap }` — unicast to local player | → HUD reads |
| Round State Machine | `round-state-machine.md` | `S2CPhaseChanged { phase, round_number, timer_duration_ms }` — broadcast | → HUD reads |
| Objective System | `objective-system.md` | `ObjectiveDestroyed` (drained by Board Rendering, then written as `HudObjectiveUpdate`) | → HUD reads via Board Rendering's client-internal message |
| Network Protocol | `network-protocol.md` | Defines all four message envelopes above; `S2CGameSnapshot` for reconnect | → HUD reads |
| Game Session System | `game-session-system.md` | Provides `local_id` from LOBBY handshake; sends `S2CGameSnapshot` on reconnect | → HUD reads |
| Game Config | `game-config.md` | `lane_count` (5) — **compile-time hard dependency.** The `[[Entity; 5]; 2]` dot entity array and `[[bool; 5]; 2]` dot state array are sized at compile time to `lane_count`. If `lane_count` changes from 5 to 6, both arrays must be resized in code — a runtime config change alone will silently truncate the scoreboard. `mana_cap` is learned at runtime via `S2CGoldUpdate`. | → HUD reads |
| Board Rendering | `board-rendering.md` | `BoardLayout` — session-scoped resource inserted by `BoardRenderingPlugin` and read by HudPlugin for scoreboard dot horizontal alignment. Board Rendering owns the lane/cell coordinate model; HUD must derive dot positions from `Res<BoardLayout>` rather than a duplicated `LANE_MIDPOINT_X` array or local uniform-spacing fallback. `HudObjectiveUpdate` client-internal Bevy `Message` written by Board Rendering after draining `ObjectiveDestroyed`. | → HUD reads |

### Downstream — none

HUD is a terminal presentation layer. No other system reads HUD state. HUD produces no output messages and no shared resources.

### Bidirectional consistency note

If any of the upstream systems changes a message payload or phase name, this GDD must be updated in the same PR. Specifically:
- Any new `S2CGoldUpdate` field that affects mana display (e.g., `max_mana_override`) requires HUD Rule 3 + Rule 4 update.
- Any new RSM phase requires a row in the Phase label strings table (Rule 5) and a row in the States and Transitions table.
- Any new objective-tier message (e.g., partial-destruction) must be evaluated for HUD scoreboard impact.
- **`BoardLayout` lane alignment**: if Board Rendering changes lane layout, HUD dot positions must be re-verified against the `BoardLayout`-derived lane midpoint helper.

**RESOLVED (2026-04-30):** `S2CGameSnapshot.PlayerSnapshot` now includes `reserved_gold: u32` (added in NP GDD Pass 4). The HUD DRAFT_AUCTION reconnect path is implementable.

## Tuning Knobs

HUD is a display-only system; its tuning knobs are all visual/timing parameters, not game balance levers. All defaults are safe starting values.

| Knob | Location | Default | Safe Range | Effect of going too high / too low |
|---|---|---|---|---|
| `hud_tween_duration_ms` | **Client-side `HudConfig` constant** (not `GameConfig` — this is a cosmetic UI preference, not an authoritative game parameter) | 300 ms | 100–500 ms | Too high: gold/mana lags behind real state, player reads stale numbers mid-auction. Too low: snapping feels jarring and visually busy. |
| `hud_margin_px` | Client layout constant | 12 px | 8–24 px | Too small: content near browser chrome clips on tight viewports. Too large: zones crowd inward and risk overlapping sister-UI elements. |
| `hud_bottom_clearance_px` | Client layout constant | 80 px | 60–120 px | Must remain ≥ Hand UI fan height (currently ~72 px). Too small: HUD zones overlap the hand fan. |
| `hud_dot_diameter_px` | Art Director / client constant | ~16 px | 10–24 px | Too small: DESTROYED dots unreadable at a glance. Too large: scoreboard competes with board and sister UIs. |
| `hud_opponent_row_opacity_destroyed` | Art Director / theme | ~0.30–0.40 luminance of background | 0.15–0.55 | Too dark: destroyed dots disappear. Too light: DESTROYED and ALIVE are indistinguishable. |
| `hud_reserved_suffix_opacity` | Art Director / theme | 0.65 | 0.50–0.80 | Too low: `(4r)` suffix unreadable inline. Too high: suffix competes with the main gold number for salience. |

**Knobs that are NOT here (delegated to other systems):**
- Timer display values — owned by Hand UI and Shop/Auction UI
- Mana cap max value — owned by Economy System / Game Config
- Objective HP — owned by Objective System / Game Config
- Scoreboard dot hue families — owned by Art Director / Art Bible

## Visual/Audio Requirements

*Art Director direction integrated from consultation. Hue families are descriptive intent — exact values are Art Bible's authority.*

### Typography

Font style: **bold, slightly rounded sans-serif** — the Wakfu "clean cartoon stencil" register. Confident stroke weight, no serifs, corners softened just enough to avoid a mechanical feel.

Relative scale hierarchy (base = own gold / own mana numerals):

| Readout | Relative size | Weight / style |
|---|---|---|
| Own gold `Xg` | 1.0× (base) | Bold upright |
| Own mana `current / mana_cap` | 1.0× | Bold upright |
| Opponent gold `Xg` | 0.85× | Bold upright — symmetry, not subordination |
| Phase label + round counter | 0.65×, 65% opacity | Regular upright — caption, not title |
| Parenthetical reserved suffix `(Yr)` | 0.65×, 65% opacity | Regular upright — on same line as parent gold value, visually subordinate |
| Reserve mana label | 0.65× | Regular upright |

**Parenthetical reserved suffix rendering:** The `(Yr)` portion is a child `TextSpan` entity (Bevy 0.18 API) within the gold label's entity tree. The parent entity carries `Text` + `TextFont` + `TextColor` for the `Xg` primary span (full weight, full opacity); a child entity carries `TextSpan` + `TextFont` + `TextColor` for the ` (Yr)` secondary span (0.65× scale, 65% opacity). In ECONOMY_BASIC mode the child TextSpan's text is set to `""` (empty string — not despawned). Visually: `**11g** (4r)` where the bold number is full weight/opacity and the parenthetical is recessed. This distinction allows the player to parse "total gold" at a glance and "reserved" as a modifier without needing to shift gaze to a second line.

**Gold vs. mana hue distinction (peripheral-legibility requirement):** Gold readout uses **warm yellow-white** (Arcane Gold's lightened ivory-warm register). Mana readout uses **cool blue-white** (Prism White's cool register). The distinction is warm vs. cool luminance — both remain legible at low attention.

### Scoreboard Dots

Dot diameter: approximately **1.5× the cap-height of the round counter numeral**.

**Implementation:** `Node` with `border_radius: BorderRadius::all(Val::Px(dot_diameter_px / 2.0))`, explicit `width: Val::Px(dot_diameter_px)`, and `height: Val::Px(dot_diameter_px)` (Bevy 0.18: `BorderRadius` is a field inside `Node`, not a standalone component; explicit dimensions required or the flexbox child collapses to zero). `BackgroundColor` (fill) and `BorderColor` (outline) are standalone components on the same entity. Border width ~2px. This is pure bevy_ui with no Sprite overhead or world-space coordinate management.

**ALIVE state:** filled circle, full opacity, neutral high-contrast tint. No glow, no rim light.

**DESTROYED state:** fill removed (`BackgroundColor(Color::NONE)`), border color shifted to approximately 30–40% luminance of the background. The circle shape (border) persists; only the fill is removed. This structural continuity — ring present, fill absent — makes an instantaneous flip read as intentional.

**Player identity tint:**
- Opponent row (top): **Terracotta / enemy identity** family
- Local row (bottom): **Sky Blue / ally identity** family
- On destruction: both go to the same neutral dark

**Transition:** ALIVE → DESTROYED is an **instantaneous state flip**, no tween.

### Dot Alignment

Scoreboard dots are positioned so each dot's horizontal center aligns with the lane midpoint derived from `Res<BoardLayout>`, the session-scoped layout resource inserted by `BoardRenderingPlugin` per ADR-021. `BoardLayout` is the single source of truth; HudPlugin must not define a separate `LANE_MIDPOINT_X: [f32; 5]` constant or fall back to local uniform spacing. If the concrete UI positioning path needs screen-space pixels, the projection/conversion helper belongs beside `BoardLayout` in the client presentation shared module so Board Rendering and HUD use the same coordinate model. Without this alignment, the scoreboard dot for lane 3 will not appear above the lane 3 column on screen.

### Phase Label

Phase label recedes via size and opacity alone (0.65×, 65% opacity). No background chip, no drop shadow. When `GAME OVER` appears: **same typographic treatment** as all other phase labels. The visual drama of GAME_OVER belongs to the board (objective burst, reveal wipe, win/loss grade). The HUD label stays consistent.

### Audio Events

| Event | Sound feel | Notes |
|---|---|---|
| Phase label change (any transition except RESOLUTION or GAME_OVER) | Single dry medium-pitched wood-block tick — one hit, no reverb tail | Marks the moment without announcing it. **Exceptions: (1) no tick fires on `S2CPhaseChanged(RESOLUTION)`** — board audio owns all audio space during RESOLUTION; **(2) no tick fires on `S2CPhaseChanged(GAME_OVER)`** — a resolved chord fires instead (see row below). |
| Objective dot darkening (DESTROYED, via `HudObjectiveUpdate`) | Short low stone-thud. **Silent during RESOLUTION** — board animation track owns all audio space. Stone-thud fires only when `phase ≠ RESOLUTION`. | Confirms permanent removal. Not mournful, not alarming. |
| Gold tween during RESOLUTION | **Silent** | Board animation track owns all audio space during RESOLUTION |
| GAME_OVER phase transition | Single resolved chord — two or three notes settling to tonic; no fanfare, no sting. The typography refuses drama (same visual register as all phase labels); the chord provides the singular audio confirmation of finality. This intentional contrast — quiet label, resolved chord — gives the moment weight without spectacle. | Confirms finality without editorialising win/loss; win/loss musical differentiation belongs to the outcome screen |
| All other HUD events (mana update, reserve mana change, phase label other than GAME_OVER, gold label format switch) | **Silent** | Peripheral UI layer must not add audio noise to active decision phases |
| All HUD events while in HIDDEN mode (LOBBY phase) | **Silent** | HUD root is `Visibility::Hidden` during LOBBY; no audio events fire |

📌 **Asset Spec** — Visual/Audio requirements are defined. After the art bible is approved, run `/asset-spec system:hud` to produce per-asset visual descriptions, dimensions, and generation prompts from this section.

## UI Requirements

The HUD's UI architecture is fully specified in Section C (Detailed Design). This section summarises the implementation-facing requirements.

### Implementation requirements (bevy_ui)

- HUD is a single root `Node` tree at screen-edge. `PickingBehavior { should_block_lower: false, is_hoverable: false }` on the root, guarded with `#[cfg(feature = "ui_picking")]` (Bevy 0.18 renames the Cargo feature from `bevy_ui_picking_backend` to `ui_picking`; inserting `PickingBehavior` without the feature compiled in causes a runtime panic — component not registered). If `ui_picking` is not in `Cargo.toml`, the root `Node` is already non-interactive and no picking component is needed.
- All 10 scoreboard dot entities and all 6 label entities (own gold, opponent gold, mana, reserve mana, phase label, round counter) are pre-spawned at session start (`Visibility::Hidden` initially). No per-round spawning. **No separate reserved-gold sub-label entities** — reserved gold is encoded inline in the gold label string.
- Each gold label entity carries a `GoldDisplayState { gold: f32, reserved_gold: f32, is_populated: bool }` component. The `is_populated` flag distinguishes the cold-start placeholder (`"--g"`) from a legitimate `0g` value. A change-detection system reads `GoldDisplayState` each frame and writes the formatted string to the `Text` component.
- Numeric tweening uses `bevy_tweening` on the `f32` fields of `GoldDisplayState` via the **separate-backing-field pattern**: a dedicated tween target drives the `f32` value toward the authoritative target each frame, and a change-detection system reads the current value and writes the formatted string to the `Text`/`TextSpan` components. **Do not implement as a `Lens<GoldDisplayState>` directly** — the `Tweening<GoldDisplayState>` animator + `handle_gold_update_system` + `handle_gold_broadcast_system` would create three simultaneous mutable writers on the same `GoldDisplayState` component in the same tick, which Bevy's scheduler cannot safely schedule.
- Layout uses `Val::Px` anchors for screen-edge positioning. Bottom-left zone always reserves vertical space for 2 lines (mana + reserve mana) to prevent layout shift when the reserve mana label appears.
- Scoreboard dots rendered as `Node` with `BackgroundColor` + `BorderColor` + `BorderRadius` (see Visual/Audio Requirements — Scoreboard Dots). Do not use `Sprite`-backed entities for dots — keeps the entire HUD in the UI hierarchy.
- Text labels use `Text` + `TextFont` + `TextColor` + `Node` (Bevy 0.18 required-component tuple). Gold labels use a **parent entity** (`Text` + `TextFont` + `TextColor` for the `Xg` primary span, full weight and opacity) plus a **child `TextSpan` entity** (`TextSpan` + `TextFont` + `TextColor` for the ` (Yr)` secondary span, 65% opacity, 0.65× scale). In ECONOMY_BASIC mode the child `TextSpan` entity's text is set to `""` (empty string — not despawned or hidden). The "no separate reserved-gold sub-label entities" rule in Rule 1 refers to top-level logical HUD label entities — the `TextSpan` child is an implementation detail of the gold label entity tree, not a separate logical label.
- `LineHeight` is auto-required in Bevy 0.18; override explicitly if line spacing needs adjustment.
- System order: `handle_gold_broadcast_system` runs **before** `handle_gold_update_system` in the HUD plugin's system set, enforcing the Rule 11 tie-break.
- `timer_duration_ms` from `S2CPhaseChanged` is discarded using `let S2CPhaseChanged { phase, round_number, .. } = msg;` — document with a code comment explaining the boundary.
- HUD reads `HudObjectiveUpdate` through `MessageReader<HudObjectiveUpdate>`, NOT `EventReader<HudObjectiveUpdate>`, NOT `EventWriter<HudObjectiveUpdate>`, and NOT `MessageReceiver<ObjectiveDestroyed>`. Board Rendering writes `HudObjectiveUpdate` through `MessageWriter<HudObjectiveUpdate>` after draining `ObjectiveDestroyed`. Explicit presentation-system ordering guarantees the HUD dot state flip in the same ECS tick as the Board Rendering write.
- `hud_tween_duration_ms` lives in a client-side `HudConfig` struct (not in `GameConfig` — it is a cosmetic preference, not a server-authoritative game parameter).
- Dot horizontal position: read `Res<BoardLayout>` and use the BoardLayout-owned lane midpoint/projection helper to align dots over lane columns. Do not introduce a separate `LANE_MIDPOINT_X` constant, local array, or uniform-spacing fallback in HudPlugin.

### Screen compatibility targets

| Viewport | Requirement |
|---|---|
| 1280 × 720 | All four zones legible; no overlap between top-right (2-line gold zone) and top-center scoreboard |
| 1920 × 1080 | Primary design target |
| 2560 × 1440 | Scale correctly; no layout reflow |
| Mobile / narrow (≤900 wide) | Out of scope for hackathon; advisory only |

### UX Spec flag

📌 **UX Flag — HUD**: This system has UI requirements. In Phase 4 (Pre-Production), run `/ux-design` to create a UX spec for the HUD screen (`design/ux/hud.md`) **before** writing epics. Stories that reference HUD layout should cite `design/ux/hud.md`, not the GDD directly.

## Acceptance Criteria

*28 BLOCKING (automatable, ECS World-based) · 4 ADVISORY (visual/manual check)*

**Core structure**

```
HUD-01: Pre-pooled node tree — BLOCKING
GIVEN the session has started,
WHEN HUD is initialized,
THEN exactly 18 HUD entities exist in the World before any S2C message is
received: 1 phase label, 1 round counter, 1 own gold label parent entity,
1 own gold TextSpan child entity (text ""), 1 opponent gold label parent
entity, 1 opponent gold TextSpan child entity (text ""), 1 mana label,
1 reserve mana label, and 10 scoreboard dot entities (= 6 top-level label
entities + 2 pre-spawned TextSpan child entities + 10 dots = 18).
No new HUD entities are spawned when subsequent update messages arrive.

HUD-02: Fixed four-zone layout — ADVISORY
GIVEN a running match,
WHEN the HUD is visible in ECONOMY_BASIC mode,
THEN top-left, top-center, top-right, and bottom-left zones render in their
designated screen-edge positions with no overlap with board or sister UIs;
confirmed by screenshot at 1280×720 and 1920×1080.

HUD-03: Display format correctness — BLOCKING
GIVEN HUD is in ECONOMY_BASIC mode,
WHEN S2CGoldUpdate arrives with gold=8, current_mana=6, mana_cap=10,
reserve_mana=2, and S2CGoldBroadcast arrives with player_id=opponent_id,
gold=6, reserved_gold=0,
THEN own gold label reads "8g", opponent gold label reads "6g",
mana label reads "6 / 10", and reserve label reads "+2 reserve".

HUD-04: Per-message update isolation — BLOCKING
GIVEN HUD is in any visible mode,
WHEN only S2CGoldUpdate arrives and no other messages are processed,
THEN only own gold/mana/reserve label component values change; all other
HUD label components retain their prior values.

HUD-05: Phase label strings — BLOCKING
GIVEN HUD is visible,
WHEN S2CPhaseChanged fires for each RSM phase in sequence,
THEN phase label reads: DRAFT_INITIAL→"DRAFT INITIAL", DRAFT_SHOP→"DRAFT",
DRAFT_AUCTION→"AUCTION", PLACEMENT→"PLACEMENT", RESOLUTION→"RESOLUTION",
GAME_OVER→"GAME OVER"; LOBBY produces no visible phase label (HUD hidden).

HUD-06: Scoreboard dot alive→destroyed — BLOCKING
GIVEN both players start with 5 objectives each (all 10 dots ALIVE),
WHEN HudObjectiveUpdate is written for opponent at lane 3,
THEN opponent dot index 2 (0-indexed) changes to DESTROYED; all other 9
dots remain ALIVE; no real/fake identifier is applied to any dot.

HUD-07: Real/fake identity never shown — BLOCKING
GIVEN the HUD has been initialized and any sequence of messages and phase
transitions has been processed (including GAME_OVER),
WHEN the HUD entity subtree is inspected,
THEN: (a) no Text or TextSpan component in the HUD entity subtree contains
the strings "REAL", "FAKE", or any string encoding an ObjectiveIdentity
discriminant; (b) no entity in the HUD subtree carries a component of type
ObjectiveIdentity or any equivalent real/fake identity marker; (c) the only
valid values of the dot-state flag stored in HUD-local state are ALIVE (false)
and DESTROYED (true) — no third state encoding real/fake exists.

HUD-08: Opponent gold adaptive — BLOCKING
GIVEN HUD is in ECONOMY_BASIC mode,
WHEN S2CGoldBroadcast arrives with player_id=opponent_id, gold=7,
reserved_gold=0,
THEN opponent gold label reads "7g" with no parenthetical suffix.
GIVEN HUD enters ECONOMY_AUCTION mode via S2CPhaseChanged(DRAFT_AUCTION),
AND S2CGoldBroadcast then arrives with player_id=opponent_id, gold=7,
reserved_gold=3,
THEN the opponent gold label entity has a parent Text entity reading "7g"
and exactly one child TextSpan entity reading " (3r)" (Bevy 0.18 multi-span
API); no separate top-level HUD entity outside this entity tree represents
opponent reserved gold.

HUD-09: RESOLUTION persistence — BLOCKING
GIVEN HUD is in any visible mode,
WHEN S2CPhaseChanged fires for RESOLUTION,
THEN the HUD root Node entity has Visibility::Visible; the HudMode resource
reads ECONOMY_BASIC; and gold label values update when S2CGoldUpdate messages
arrive during resolution (verified by sending a S2CGoldUpdate and reading the
updated GoldDisplayState.gold value from the World).
(Sister-UI hiding requires an integration test across all presentation plugins —
not verifiable in HudPlugin isolation.)

HUD-10: GAME_OVER freeze — BLOCKING
GIVEN HUD has own gold label showing 12g and 7 dots in their current states,
WHEN S2CPhaseChanged fires for GAME_OVER,
THEN no subsequent S2CGoldUpdate or HudObjectiveUpdate changes any HUD
component; phase label reads "GAME OVER"; no real/fake data appears anywhere.

HUD-11: No timer displayed — BLOCKING
GIVEN HUD is in any mode,
WHEN any S2C message arrives,
THEN no HUD entity holds a component or text value representing a countdown
or elapsed time.

HUD-12: Numeric tween duration — ADVISORY
GIVEN HUD is in ECONOMY_BASIC mode,
WHEN a gold or mana value changes,
THEN the displayed value animates to the new number within 300ms; confirmed
by elapsed-time measurement between message receipt and final GoldDisplayState
value stabilising (manual tick-step or elapsed-timer measurement).

HUD-12b: Phase label, round counter, and dot transitions are instantaneous — BLOCKING
GIVEN HUD is in any visible mode,
WHEN S2CPhaseChanged fires (any phase) or HudObjectiveUpdate is written,
THEN the phase label text, round counter text, and affected dot visual state
all reflect the new values within the same ECS tick — no tween, no deferred
update, no animation component attached to these entities.

HUD-13: Snapshot rebuild — BLOCKING
GIVEN the HUD is in any state (including partially updated or stale),
WHEN S2CGameSnapshot is received,
THEN every HUD zone reflects the values encoded in the snapshot within the
same frame; no entity is despawned or re-spawned (pre-pooled entities reused).

HUD-14: Snapshot rebuild — no flicker — ADVISORY
GIVEN a player reconnects mid-match,
WHEN S2CGameSnapshot is received,
THEN no frame is observed where any zone shows a blank or stale value;
confirmed by screenshot at the reconnect moment (best-effort evidence —
screenshots cannot prove absence across all frames, but capture the
reconnect-instant state).
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
THEN phase label reads "DRAFT", both gold labels read "Xg" format (no
parenthetical suffix), and HUD mode remains ECONOMY_BASIC.

HUD-17: RESOLUTION → DRAFT_AUCTION — BLOCKING
GIVEN HUD is in ECONOMY_BASIC mode with own GoldDisplayState.gold=11 and
opponent GoldDisplayState.gold=8,
WHEN S2CPhaseChanged fires for DRAFT_AUCTION,
THEN HUD enters ECONOMY_AUCTION, phase label reads "AUCTION",
own gold label reads "11g (0r)" and opponent gold label reads "8g (0r)"
(parenthetical shows 0r at auction entry per server invariant that no bids
exist at auction start).

HUD-18: PLACEMENT → RESOLUTION — BLOCKING
GIVEN HUD is in ECONOMY_BASIC mode with phase label "PLACEMENT",
WHEN S2CPhaseChanged fires for RESOLUTION,
THEN phase label reads "RESOLUTION", HUD remains fully visible,
and no zone is hidden; gold-delta ticks continue to be received.

HUD-19: RESOLUTION → GAME_OVER — BLOCKING
GIVEN HUD is in ECONOMY_BASIC mode with own GoldDisplayState.gold=12.0
and phase label "RESOLUTION",
WHEN S2CPhaseChanged fires for GAME_OVER,
AND the following are subsequently emitted and processed in the same test world:
  S2CGoldUpdate { gold: 999, current_mana: 0, reserve_mana: 0, mana_cap: 10 },
  S2CGoldBroadcast { player_id: local_id, gold: 888, reserved_gold: 0 },
  a `HudObjectiveUpdate { target_player_id: opponent, lane: 1 }` message written through the Bevy Message path,
THEN: (a) phase label reads "GAME OVER"; (b) HudMode resource reads FROZEN;
(c) own GoldDisplayState.gold remains 12.0 (not 999 or 888); (d) opponent
dot for lane 1 retains its pre-GAME_OVER dot state.

HUD-29: ECONOMY_AUCTION → ECONOMY_BASIC transition — BLOCKING
GIVEN HUD is in ECONOMY_AUCTION mode with own gold label "11g (4r)" and
opponent gold label "8g (2r)",
WHEN S2CPhaseChanged fires for DRAFT_SHOP,
THEN HUD enters ECONOMY_BASIC; own gold label parent Text entity reads "11g"
and child TextSpan entity text is "" (empty, no parenthetical); opponent gold
label parent reads "8g" with child TextSpan text ""; HudMode resource reads
ECONOMY_BASIC.
```

**Economy and tie-break**

```
HUD-20: Same-tick S2CGoldUpdate + S2CGoldBroadcast tie-break — BLOCKING
GIVEN S2CGoldUpdate { gold: 15, current_mana: 0, reserve_mana: 0, mana_cap: 10 }
and S2CGoldBroadcast { player_id: local_id, gold: 12, reserved_gold: 0 } arrive
in the same ECS tick,
WHEN all HUD update systems complete,
THEN GoldDisplayState.gold on the own gold label entity reads 15.0, confirming
the system execution order declared in HudPlugin resolves the conflict in favour
of S2CGoldUpdate.
NOTE: This is an Integration story — the test must use App::new() with HudPlugin
registered to verify that the system ordering is correctly declared in the plugin.
A World::new() unit test that manually calls systems in order does not verify
plugin system ordering.

HUD-21: Mana cap denominator update — BLOCKING
GIVEN the mana display reads "4 / 8",
WHEN S2CGoldUpdate arrives with current_mana=4, mana_cap=10,
THEN the mana label updates to "4 / 10"; numerator is unchanged;
reserve label remains hidden if reserve_mana=0.

HUD-22: Round counter format — BLOCKING
GIVEN HUD is in any visible, non-LOBBY phase,
WHEN S2CPhaseChanged arrives with round_number=9,
THEN the round counter label component reads exactly "R9" (not "9", not
"Round 9", not "R09"); verified by querying the Text component on the round
counter entity.

HUD-23: Round counter visible at GAME_OVER — BLOCKING
GIVEN the round counter displayed "R14" during RESOLUTION,
WHEN S2CPhaseChanged fires for GAME_OVER,
THEN: (a) the round counter entity's own `Visibility` component reads
`Visibility::Visible` (verified by direct component query on the round
counter entity specifically, not inferred from HUD root visibility);
(b) the round counter entity's Text component reads "R14"; it is not
hidden, despawned, or overwritten by the phase transition.

HUD-24: HUD root hidden at LOBBY — BLOCKING
GIVEN HUD has been initialized (all entities pre-spawned),
WHEN no S2CPhaseChanged has been received (or the most recent phase was LOBBY),
THEN the HUD root Node entity has Visibility::Hidden.

HUD-25: Cold-start placeholder display — BLOCKING
GIVEN S2CPhaseChanged(DRAFT_INITIAL) has been received in this session,
WHEN no S2CGoldUpdate has yet arrived for the local player
AND no S2CGoldBroadcast for opponent_id has yet arrived,
THEN the own gold label reads "--g", the mana label reads "-- / --", and the
opponent gold label reads "--g". None of these read "0g" or "0 / 0" (which
are valid runtime values and must not appear before the server confirms state).
This test is scoped to the cold-start window; a subsequent legitimate
S2CGoldUpdate { gold: 0 } MUST then produce "0g", not "--g".

HUD-26: Duplicate HudObjectiveUpdate idempotency — BLOCKING
GIVEN destroyed[opponent][2] (lane 3, 0-indexed) is already true,
WHEN HudObjectiveUpdate is written again with target_player_id=opponent, lane=3,
THEN the dot entity's state component has the same value it held before the
second event arrived (verified by reading the component from the World before
and after the second event); no panic, error event, or spurious output is
emitted.

HUD-27: FROZEN snapshot bypass — BLOCKING
GIVEN HUD is in FROZEN mode (S2CPhaseChanged(GAME_OVER) was received and all
labels show their final values),
WHEN S2CGameSnapshot arrives (simulating reconnect at GAME_OVER),
THEN the HUD runs a full rebuild from snapshot per Rule 13, then immediately
re-enters FROZEN; all label values reflect the snapshot state; a subsequent
S2CGoldUpdate with a different gold value does not alter the own gold label's
GoldDisplayState.gold field.

HUD-28: Inline gold format uses parent + TextSpan child — ADVISORY
GIVEN HUD has entered ECONOMY_AUCTION mode and S2CGoldBroadcast arrives
with player_id=opponent_id, gold=7, reserved_gold=3,
THEN querying the opponent gold label entity returns a parent Text entity
whose text reads "7g", and it has exactly one child entity with a TextSpan
component reading " (3r)". No top-level HUD entity outside this entity tree
represents opponent reserved gold.

HUD-30: OOB lane guard in HudObjectiveUpdate — BLOCKING
GIVEN HUD is in any visible mode,
WHEN HudObjectiveUpdate arrives with lane=0 or lane=6 (outside 1..=5),
THEN no dot entity state changes, no index access is performed, no panic
occurs, and a warning is logged; the HUD continues normal operation.

HUD-31: mana_cap=0 guard — BLOCKING
GIVEN HUD is in ECONOMY_BASIC mode,
WHEN S2CGoldUpdate arrives with current_mana=0, mana_cap=0,
THEN the mana label renders "0 / 0" without panic or crash; a warning is
logged. (mana_cap=0 violates the server invariant; this guard prevents panic
on a server bug or test fixture anomaly.)
```

## Open Questions

| ID | Question | Owner | Status |
|---|---|---|---|
| OQ-HUD-01 | **Disconnect/pause indicator — reclassified as gameplay correctness gap.** During the 30-second disconnect grace window, client-side phase timers continue countdown. When PLACEMENT timer expires, Hand UI fires its timeout action locally — but the server is paused. `S2CSessionPaused` / `S2CSessionResumed` must be defined in `network-protocol.md` before any timer-bearing phase can be safely implemented. HUD owns the "Waiting for opponent…" badge. This OQ is **blocking on Network Protocol GDD**. **Pre-implementation gate:** NP GDD must reach Approved status before the pause overlay can be implemented. Until then, implement HUD without the pause overlay badge — treat the disconnect grace window as invisible to the HUD. | Network Protocol GDD (blocking) | Open — NP GDD amendment required |
| OQ-HUD-02 | **Local player real/fake opt-in display — DESIGN REJECTED.** A settings flag would recreate the screen-share leak (the reason Rule 7 exists). | Design | Closed — Rejected |
| OQ-HUD-03 | **GAME_OVER summary screen.** Rule 10 defers retroactive real/fake revelation to a post-game summary screen. When is this GDD'd? Does HUD hand off any state to the summary screen, or does the summary screen rebuild from `S2CGameSnapshot`? | Future GDD (post-game flow / M3) | Open |
| OQ-HUD-04 | **Scoreboard dot lane alignment source.** HudPlugin reads the session-scoped `BoardLayout` resource inserted by `BoardRenderingPlugin` and uses the BoardLayout-owned lane midpoint/projection helper for dot horizontal centers. Do not define a separate `LANE_MIDPOINT_X` constant, local `[f32; 5]`, or uniform-spacing fallback in HudPlugin. | Board Rendering / Tech Lead | Closed — Resolved 2026-05-02 |
| OQ-HUD-05 | **`HudObjectiveUpdate` message type definition location.** `HudObjectiveUpdate { target_player_id, lane }` is a client-internal Bevy `Message` defined in the client crate's presentation/UI shared module. It is registered once with `app.add_message::<HudObjectiveUpdate>()`; Board Rendering writes it with `MessageWriter`, and HudPlugin reads it with `MessageReader`. It is not a Lightyear replicated component, not an Observer trigger, and not defined in the workspace `shared/` crate. | Tech Lead / Lead Programmer | Closed — Resolved 2026-05-01 |
