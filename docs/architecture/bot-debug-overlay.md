# Bot Debug Overlay — Architecture

**Status:** Implemented (PROMPT 1614), verified (PROMPT 1618)  
**Commit:** `37306162` on `origin/main`  
**Story AC:** Story 005 AC7  
**References:** [PROMPT 1604 data-contract audit](../../reports/PROMPT-1604-bot-flow-debug-overlay-data-contract.md) · [PROMPT 1614 implementation](../../reports/PROMPT-1614-bot-debug-overlay-implementation.md) · [PROMPT 1618 focused verify](../../reports/PROMPT-1618-bot-debug-overlay-focused-verify.md)

---

## Overview

The bot debug overlay is a dev/QA-only observability surface that exposes the
server's internal bot state — hand contents, economy, and decision-log tail —
to a human observer in real time. It is delivered via a new debug-only S2C
protocol message, rendered as a non-interactive corner panel in the client,
and gated by independent environment variables on both sides so it can never
appear in a production session unless both the server operator and the client
operator have explicitly opted in.

---

## 1. Data Contract

### 1.1 Why a new protocol message is required

PROMPT 1604 audited three data categories required by the overlay:

| # | Data | Server resource | Already client-visible? |
|---|---|---|---|
| 1 | Bot hand (god-mode) | `PlayerHands` (redacted for non-recipient in `snapshot.rs:111-119`) | **No** — intentionally emptied |
| 2 | Decision-log tail + auction valuation | `BotDecisionLog` (`Resource`, never replicated) | **No** — server-only |
| 3 | Class / gold / mana / submitted / objectives | `PlayerSnapshot` inside `S2CGameSnapshot` (unredacted for all players) | **Yes** — no new plumbing needed |

Items 1 and 2 cannot reach the client without new protocol. A single
`S2CDebugBotStatePush` message carries all three, keeping the implementation
as one atomic protocol addition.

### 1.2 `S2CDebugBotStatePush`

Defined in `shared/src/protocol.rs`. Sent at most every 500 ms per session on
the `Reliable` channel. Carries a god-mode snapshot of every bot player.

```rust
pub struct S2CDebugBotStatePush {
    /// One entry per bot in the session.
    pub bots: Vec<DebugBotStateEntry>,
    /// Total entries in the server-side BotDecisionLog (shows "N of M").
    pub decision_log_total: u32,
    /// Server wallclock ms when this push was assembled (stale-indicator).
    pub assembled_at_ms: u64,
}
```

Registration: `register_s2c::<S2CDebugBotStatePush>(registry, ProtocolChannel::Reliable)` in `shared/src/protocol.rs`.

### 1.3 `DebugBotStateEntry`

One entry per bot in the session. Carries god-mode data that normal
`PlayerSnapshot` redacts.

```rust
pub struct DebugBotStateEntry {
    pub player_id:          PlayerId,
    pub class_id:           Option<ClassId>,
    pub gold:               u32,
    pub current_mana:       u32,
    pub mana_cap:           u8,
    pub submitted:          bool,
    /// God-mode full hand. Never sent in any other message.
    pub hand:               Vec<CardId>,
    /// Tail of the bot's decision log, oldest→newest, capped at 12 entries.
    pub decision_tail:      Vec<DebugBotDecisionEntry>,
    /// Valuation from the most recent AuctionBid decision, or None.
    pub last_bid_valuation: Option<u32>,
}
```

`class_id` is `Option` because the bot may not have chosen a class yet; the
assembly helper falls back to the session `ClassMap` before yielding `None`.

### 1.4 `DebugBotDecisionEntry`

A serialisable mirror of `BotDecisionEntry` from
`server/src/feature/bot/state.rs`. Decoupled so the shared crate does not
import server-internal types.

```rust
pub struct DebugBotDecisionEntry {
    pub round_number:  u32,
    pub phase:         RoundPhase,
    pub timestamp_ms:  u64,
    pub kind_label:    String,   // snake_case variant name, e.g. "auction_bid"
    pub detail:        Option<String>, // human-readable, e.g. "card=42 amt=4 val=5"
}
```

`kind_label` and `detail` are assembled from the `BotDecisionKind` enum in
`server/src/feature/bot/state.rs:184-205` by `decision_kind_to_label_and_detail`
in `debug_push.rs`. The conversion is deterministic and covered by unit tests
(`decision_kind_label_is_snake_case`, `decision_detail_formats_auction_bid`).

---

## 2. Server Assembly (`BotDebugPushPlugin`)

**File:** `server/src/feature/bot/debug_push.rs` (NEW, additive only — no
edits to `state.rs`, `action_loop.rs`, or `lobby_loop.rs`)

### 2.1 Resources

| Resource | Role |
|---|---|
| `BotDebugPushConfig` | Parsed once at startup from env var. Holds `enabled: bool`, `interval_ms: u64` (500), `tail_cap: usize` (12). |
| `BotDebugPushState` | Runtime state. Holds `next_push_ms: u64` for rate-limiting. |

`BotDebugPushConfig::from_env_values` follows the `QASnapshotConfig` parsing
rules byte-for-byte: `"1"` → enabled, `"0"` → disabled, unset/empty →
`cfg!(debug_assertions)`, any other value → `tracing::warn!` + disabled. This
ensures the binary's env-var behaviour is consistent across all debug tooling.

### 2.2 Rate limit and gating order

`bot_debug_push_system` (registered in `Update`) early-returns in this order:

1. `if !config.enabled { return; }` — production servers pay zero cost.
2. `if bot_players.is_empty() { return; }` — no bots, no push.
3. `if now_ms < state.next_push_ms { return; }` — 500 ms rate gate.
4. `if decision_log is missing { return; }` — resource not yet inserted.

After a push is assembled, `next_push_ms` is bumped unconditionally even if
the recipients list is empty (lobby pre-handshake guard).

### 2.3 Source resources read

`bot_debug_push_system` reads only these existing server resources (no writes):

- `BotPlayers` — identifies which `PlayerId`s are bots.
- `BotDecisionLog` — decision entries to tail.
- `PlayerHands` — god-mode hand per bot.
- `PlayerEconomies` — gold / mana / mana_cap / submitted per bot.
- `SessionConfig` / `ClassMap` — class resolution fallback.
- `PlayerConnectionMap` — identifies human recipients.
- `RoundState` — `assembled_at_ms` timestamp.

### 2.4 Recipients

`NetworkTarget::Only(recipient_peers)` where recipient peers are every
`PlayerId` in `PlayerConnectionMap` that is **not** in `BotPlayers`. Bots
never receive the debug push (saves bandwidth during AI-vs-AI soaks).

### 2.5 Plugin registration

`app.add_plugins(BotDebugPushPlugin)` in `server/src/main.rs`, placed
immediately after the existing `BotQaSnapshotPlugin` registration line.

---

## 3. Activation and Gating

### 3.1 Server gate (`CCGS_BOT_DEBUG_UI`)

Set `CCGS_BOT_DEBUG_UI=1` in the server's environment. The env var is read
once at plugin startup; no hot-reload. When unset in a non-debug build, the
gate is closed and `bot_debug_push_system` is a no-op with no allocations or
wire traffic.

| Value | Behaviour |
|---|---|
| `1` | Enabled |
| `0` | Disabled |
| unset / empty | `cfg!(debug_assertions)` — enabled in debug builds, disabled in release |
| any other | `tracing::warn!` + disabled |

### 3.2 Client gate (`CCGS_DEBUG_UI`)

Set `CCGS_DEBUG_UI=1` in the client's environment. Independent of the server
gate: a debug-server operator can use a non-debug client and vice versa. Same
parsing rules as `CCGS_BOT_DEBUG_UI`.

When disabled, `DebugBotOverlayPlugin` spawns no UI nodes and registers zero
Bevy systems — identical no-cost discipline used by `QASnapshotPlugin`.

### 3.3 F8 toggle

With `CCGS_DEBUG_UI=1`, the overlay is visible by default on first show.
Pressing **F8** toggles `DebugBotOverlayState.visible` which syncs to the
root node's `Visibility`. F8 was confirmed free (F9 is used by QA snapshot).
The keyboard handler is gated by `config.enabled` and follows the same
`Option<Res<ButtonInput<KeyCode>>>` shape as `qa_snapshot_keyboard_shortcut_system`
to avoid panics under `MinimalPlugins`.

---

## 4. Client Overlay (`DebugBotOverlayPlugin`)

**File:** `client/src/presentation/debug_bot_overlay.rs` (NEW)  
**Registration:** `client/src/presentation/mod.rs` — `app.add_plugins(DebugBotOverlayPlugin)` after `QASnapshotPlugin`.

### 4.1 UI layout

- Top-right corner, absolute position, 360 px wide, ≤70 % viewport height.
- Header: `"Bot Debug (F8)"`.
- Body lines (regenerated on each `DebugBotOverlayState` change):
  - `Bots: N  |  Decisions: M  |  @ <assembled_at_ms> ms`
  - Per bot: `[PlayerId(x)] class=Cra gold=N mana=C/M submitted=B hand=H last_bid_val=V`
  - Per decision tail entry (oldest→newest, max 12): `r<R>.<Phase> <ts> <kind_label> <detail>`
- Non-interactive: `Pickable { should_block_lower: false, is_hoverable: false }` so gameplay input passes through the overlay unblocked.

### 4.2 Z-layer

`z_layers::DEBUG` (700) — above MODAL (500). The documented layer for diagnostic dev tooling. This ensures the overlay is never obscured by any gameplay UI, even modal overlays.

### 4.3 State resources

| Resource | Role |
|---|---|
| `DebugBotOverlayConfig` | Parsed from `CCGS_DEBUG_UI`; holds `enabled: bool`. |
| `DebugBotOverlayState` | Holds `latest: Option<S2CDebugBotStatePush>` and `visible: bool`. Default: hidden + empty. |

### 4.4 Systems

| System | Schedule | Purpose |
|---|---|---|
| Startup spawn | `Startup` | Spawns root node at `Visibility::Hidden`. |
| `debug_bot_overlay_f8_system` | `Update` | Toggles `visible` on F8 keypress. |
| Receive drain | `Update` | Drains `MessageReceiver<S2CDebugBotStatePush>` into `DebugBotOverlayState.latest`. |
| Render body | `Update` | Calls `render_overlay_body` from `DebugBotOverlayState`; re-layouts text children. |
| Sync visibility | `Update` | Writes `Visibility` on root node from `DebugBotOverlayState.visible`. |

---

## 5. Operator Workflow

### 5.1 Bot-vs-bot soak

```bash
# Terminal 1 — server
CCGS_BOT_DEBUG_UI=1 cargo run -p server

# Terminal 2 — human observer client
CCGS_DEBUG_UI=1 trunk serve
```

Press **F8** to show the overlay once the session is running. The panel
updates every ~500 ms. Watch `kind_label` and `detail` in the decision tail
to verify the bot is taking expected actions each phase. The `last_bid_val`
column shows the bot's reservation price for the current auction item.

### 5.2 Autoplay smoke

During autoplay (`tools/autoplay/`) the server is launched headless. Set
`CCGS_BOT_DEBUG_UI=1` in the server's environment. If a human observer
client is attached (e.g. for manual spot-checks), set `CCGS_DEBUG_UI=1` on
that client. The JSONL decision log (PROMPT 1597) runs in parallel and is the
primary machine-readable audit trail; the overlay is the live human-readable
companion.

---

## 6. Release Safety

The overlay is dev-only by design. In production:

- Neither `CCGS_BOT_DEBUG_UI` nor `CCGS_DEBUG_UI` is set.
- Server: `bot_debug_push_system` returns immediately at the first gating
  check; no allocations, no wire traffic, no `S2CDebugBotStatePush` messages.
- Client: `DebugBotOverlayPlugin` registers no UI nodes and no systems.
- The god-mode `hand` field of `DebugBotStateEntry` is never transmitted.
  It does not shadow or interfere with the normal `PlayerSnapshot` redaction
  in `server/src/core/session/snapshot.rs:111-119`.

The dual-gate design means a misconfigured server (env var accidentally set)
cannot surface the overlay on a release client, and vice versa. Both gates
must be open simultaneously for any debug data to appear.

---

## 7. Relationship to Adjacent Systems

### 7.1 Bot Decision JSONL (PROMPT 1597)

`BotQaSnapshotPlugin` streams `BotDecisionLog` entries to a `.jsonl` file on
the server's local disk. This is the primary machine-readable audit trail for
offline analysis (grep-able, appendable, survives session end).

`BotDebugPushPlugin` reads the same `BotDecisionLog` resource and broadcasts
a tail to connected human clients in near-real-time. The two plugins are
independent: JSONL runs always when its env var is set; the debug push runs
only with `CCGS_BOT_DEBUG_UI=1`. Neither depends on the other.

### 7.2 QA Snapshots (PROMPT 1597 / `qa_snapshot.rs`)

`QASnapshotPlugin` serialises full game state on demand (F9 or
`CCGS_QA_SNAPSHOT=1`). The debug overlay and QA snapshot share:

- The same env-var parsing pattern (`from_env_values` shape).
- The same F-key discipline (F9 for snapshot, F8 for overlay — no collision).
- The same plugin registration location (`presentation/mod.rs`).

They are independent systems at runtime.

### 7.3 Future headless / autoplay evidence

The JSONL log is already the canonical evidence source for CI and autoplay.
The debug push overlay is a human-observer tool. A future headless-evidence
pipeline (if needed) should read from JSONL, not from the push message, since
WASM clients and headless runners have no GUI.

---

## 8. Test Coverage

### 8.1 Server tests (9/9 passing — `debug_push.rs`)

| Test | What it verifies |
|---|---|
| `from_env_values_respects_explicit_enable` | `"1"` → enabled |
| `from_env_values_invalid_is_disabled` | non-`"0"`/`"1"` → warn + disabled |
| `from_env_values_uses_dev_default_when_unset` | unset → `cfg!(debug_assertions)` |
| `decision_kind_label_is_snake_case` | all `BotDecisionKind` variants → snake_case labels |
| `decision_detail_formats_auction_bid` | `AuctionBid` → `"card=N amt=N val=N"` |
| `assemble_handles_missing_resources_gracefully` | missing `PlayerHands` / `PlayerEconomies` → no panic |
| `assemble_pulls_class_from_bot_then_falls_back_to_session_class_map` | class resolution fallback chain |
| `assemble_includes_hand_and_economy_when_available` | hand + gold/mana pass-through |
| `assemble_orders_bots_by_player_id_and_caps_tail` | tail cap at 12; deterministic ordering |

### 8.2 Client tests (7/7 passing — `debug_bot_overlay.rs`)

| Test | What it verifies |
|---|---|
| `from_env_values_respects_explicit_enable` | `"1"` → enabled |
| `from_env_values_invalid_is_disabled` | non-`"0"`/`"1"` → warn + disabled |
| `from_env_values_uses_dev_default_when_unset` | unset → `cfg!(debug_assertions)` |
| `debug_bot_overlay_state_default_is_hidden_and_empty` | initial state: hidden, no payload |
| `apply_overlay_toggle_flips_visibility` | F8 toggles `visible` |
| `render_overlay_body_with_no_bots_includes_assembled_at` | header line with timestamp |
| `render_overlay_body_includes_class_gold_mana_submitted_hand_and_decision` | full body format |

### 8.3 Compile verification (PROMPT 1618)

`cargo check -p shared`, `cargo check -p server`, `cargo check -p client` —
all pass clean at commit `37306162`.
