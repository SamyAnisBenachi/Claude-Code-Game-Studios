# PROMPT 1614 — BOT-DEBUG-OVERLAY-IMPLEMENTATION

**Lane:** implementation worker (debug-only observability).
**Source-of-truth tip:** `origin/main @ 26ef46e1` (rebased onto latest after
fetching; PROMPT 1613 + 26ef46e1 session-state were the 3 intervening
commits, all in disjoint scopes).
**Worktree / branch:** `D:/Tmp/wt-1614` / `work/bot-debug-overlay-1614`.
**Data contract:** `reports/PROMPT-1604-bot-flow-debug-overlay-data-contract.md`
(verbatim §4.1–§4.3; no semantic deviation).

## 1. Summary

Implemented the first end-to-end slice of the bot debug overlay:

- Server emits `S2CDebugBotStatePush` to every human peer in a session at
  500 ms cadence, gated by `CCGS_BOT_DEBUG_UI=1` (mirrors
  `BotQaSnapshotConfig::from_env_values` parsing rules byte-for-byte).
- Client renders the latest payload inside an F8-toggled corner panel,
  gated by an independent `CCGS_DEBUG_UI=1` env var (so the panel never
  surfaces unless QA explicitly opted in).
- Carries the three data points PROMPT 1604 §1 flagged as not
  client-visible today: the bot's **hand** (god-mode), the **tail of its
  decision log** (with auction valuations), and a per-bot
  **last_bid_valuation** rolled up from the decision log.

No release behaviour changes when either env var is unset / `0`.
No semantic gameplay mutation endpoints were touched.

## 2. Files changed (allowlist-clean)

| Path | Status | Owned by PROMPT 1614? |
|---|---|---|
| `shared/src/protocol.rs` | modified — added `S2CDebugBotStatePush` + `DebugBotStateEntry` + `DebugBotDecisionEntry` and their registration | yes (§Owned scope) |
| `server/src/feature/bot/debug_push.rs` | NEW — `BotDebugPushPlugin`, `BotDebugPushConfig`, `BotDebugPushState`, `bot_debug_push_system`, pure `assemble_debug_bot_state_push` helper, 9 inline unit tests | yes |
| `server/src/feature/bot/mod.rs` | modified — `pub mod debug_push;` + re-exports | yes |
| `server/src/main.rs` | modified — one-line `app.add_plugins(BotDebugPushPlugin)` after the existing `BotQaSnapshotPlugin` registration | yes |
| `client/src/presentation/debug_bot_overlay.rs` | NEW — `DebugBotOverlayPlugin`, `DebugBotOverlayConfig`, `DebugBotOverlayState`, F8 toggle system, `S2CDebugBotStatePush` drain system, `render_overlay_body` pure helper, 7 inline unit tests | yes |
| `client/src/presentation/mod.rs` | modified — `pub mod debug_bot_overlay;`, import + `app.add_plugins(DebugBotOverlayPlugin)` after the existing `QASnapshotPlugin` registration | yes |

Forbidden paths NOT touched: `tools/autoplay/**`, `docs/autoplay.md`,
`production/sprint-status.yaml`, `production/session-state/**`,
`production/sprints/**`, `production/stage.txt`. No broad UI polish.

`git diff --check` is clean.

## 3. Debug gating

### Server side (`server/src/feature/bot/debug_push.rs`)

- Env var: `CCGS_BOT_DEBUG_UI`. Parsing rules:
  `"1"` → enabled, `"0"` → disabled, unset / empty → `cfg!(debug_assertions)`,
  any other value → `tracing::warn!` + disabled (never panics).
- `bot_debug_push_system` early-returns when:
  1. config disabled, **or**
  2. `BotPlayers` resource missing or empty, **or**
  3. `now < state.next_push_ms` (500 ms rate limit), **or**
  4. `BotDecisionLog` resource missing.
- Reads observable resources only (`BotPlayers`, `BotDecisionLog`,
  `PlayerHands`, `PlayerEconomies`, `SessionConfig`, `PlayerConnectionMap`,
  `RoundState`); no edits to existing bot files.
- Recipients = every peer in `PlayerConnectionMap` whose `PlayerId` is not
  in `BotPlayers`. Bots never receive the debug push (saves bandwidth on
  AI vs AI soaks).
- Sends via `ServerMultiMessageSender::send::<S2CDebugBotStatePush,
  ReliableChannel>` with `NetworkTarget::Only(recipient_peers)`.
- If recipients empty (e.g. lobby pre-handshake) the push is still
  assembled and `next_push_ms` is bumped so the rate limit holds; the
  wire send is just skipped.

### Client side (`client/src/presentation/debug_bot_overlay.rs`)

- Env var: `CCGS_DEBUG_UI`. **Independent** of `CCGS_BOT_DEBUG_UI` so a
  developer running a debug-server can stay on a non-debug client.
- When disabled, the plugin spawns no UI and registers zero systems —
  identical no-cost-when-off discipline used by `QASnapshotPlugin`.
- When enabled:
  - F8 toggles `DebugBotOverlayState.visible`.
  - Receive system updates `DebugBotOverlayState.latest` on every
    incoming `S2CDebugBotStatePush`.
  - Body text re-rendered from `render_overlay_body` whenever
    `DebugBotOverlayState` changes.
  - Visibility synced from `DebugBotOverlayState.visible` to the root
    node's `Visibility`.
  - Root node: `position_type: Absolute`, top-right corner,
    `Pickable { should_block_lower: false, is_hoverable: false }` so
    gameplay input passes through.
  - Z-layer: `z_layers::DEBUG` (700), above MODAL (500) — the documented
    layer for diagnostic dev tooling.

## 4. Protocol / UI surface

### `S2CDebugBotStatePush`

```rust
pub struct S2CDebugBotStatePush {
    pub bots: Vec<DebugBotStateEntry>,
    pub decision_log_total: u32,
    pub assembled_at_ms: u64,
}

pub struct DebugBotStateEntry {
    pub player_id: PlayerId,
    pub class_id: Option<ClassId>,
    pub gold: u32,
    pub current_mana: u32,
    pub mana_cap: u8,
    pub submitted: bool,
    pub hand: Vec<CardId>,
    pub decision_tail: Vec<DebugBotDecisionEntry>,
    pub last_bid_valuation: Option<u32>,
}

pub struct DebugBotDecisionEntry {
    pub round_number: u32,
    pub phase: RoundPhase,
    pub timestamp_ms: u64,
    pub kind_label: String,   // snake_case variant name
    pub detail: Option<String>,
}
```

Registered on the `Reliable` channel via `register_s2c::<S2CDebugBotStatePush>`.
Naming follows the existing convention; no other `S2CDebug*` message
exists yet, so this slice sets the precedent for the family.

### UI surface

Corner panel (top-right, 360 px wide, ≤70 % viewport height, scrollable).
Header `"Bot Debug (F8)"`. Body lines:

- `Bots: N  |  Decisions: M  |  @ <assembled_at_ms> ms`
- For each bot: `[PlayerId(x)] class=Cra gold=N mana=C/M submitted=B hand=H last_bid_val=V`
- For each tail entry (oldest→newest, capped at 12): `r<R>.<Phase> <ts> <kind_label> <detail>`

Non-interactive. Drains every frame when the overlay is enabled.

## 5. Validation

Focused (per task rules — broad workspace verification deferred to a
VERIFY lane):

| Check | Command | Result |
|---|---|---|
| Path allowlist review | `git status` | All paths inside owned scope; no forbidden touches. |
| Whitespace cleanliness | `git diff --check` | Clean. |
| shared crate compiles | `cargo check -p shared` | OK (no new warnings). |
| server crate compiles | `cargo check -p server` | OK (no new warnings; pre-existing deprecation warnings in unrelated UI files are unchanged). |
| client crate compiles | `cargo check -p client` | OK (pre-existing deprecation warnings only). |
| server inline unit tests for `debug_push` | `cargo test -p server --lib feature::bot::debug_push` | 9/9 PASS. |
| client inline unit tests for `debug_bot_overlay` | `cargo test -p client --lib presentation::debug_bot_overlay` | 7/7 PASS. |
| shared crate tests | `cargo test -p shared` | All pre-existing tests still pass. |

Deferred (per "no broad workspace cargo" rule): full
`cargo test --workspace`, smoke check, integration-test sweep.

## 6. Next VERIFY lane suggestion

A short focused verify lane should:

1. Run `cargo check --workspace` and `cargo test -p server` /
   `cargo test -p client` for the bot test family.
2. Spin up the headless server with `CCGS_BOT_DEBUG_UI=1` (e.g. via the
   PROMPT 1603 two-bot soak entrypoint) + a `trunk serve` client with
   `CCGS_DEBUG_UI=1`, confirm F8 surfaces the overlay and the body
   updates as the bot bids in DRAFT_AUCTION.
3. Confirm the gameplay UI behind the overlay still receives clicks
   (Pickable is non-blocking).
4. Confirm leaving both env vars unset in a release build leaves zero
   on-screen debug surface and no `S2CDebugBotStatePush` traffic on the
   wire.

## 7. Test evidence by story type

- **Logic** (env parsing, kind labels, assembly correctness, tail cap,
  last_bid_valuation roll-up, hand/economy pass-through, class
  fallback chain): covered by 9 inline server tests + 7 inline client
  tests, all green.
- **Visual / Feel** (corner placement, F8 toggle in-game, scroll
  behaviour with long decision tails): ADVISORY — defer to the VERIFY
  lane's live capture step.
- **Integration** (server send → client receive round-trip): ADVISORY —
  the receive-drain system is unit-tested via the pure
  `apply_overlay_toggle` + state-mutation pattern; an end-to-end
  Lightyear scaffold is heavier than the task's "focused tests if
  cheap" rule allowed.

## 8. Out-of-scope check

- Only the six allowlisted files were touched.
- `production/**` untouched.
- `tools/autoplay/**` and `docs/autoplay.md` untouched (PROMPT 1613 lane
  remains disjoint).
- No edits to `state.rs`, `action_loop.rs`, `lobby_loop.rs`, or
  `qa_snapshot.rs` inside the bot module — `debug_push.rs` is purely
  additive.
- No `Cargo.toml` / `Cargo.lock` edits required: the new file is exposed
  to the crate via `mod.rs`, no new test wiring, no new deps.

---

1614: BOT-DEBUG-OVERLAY-IMPLEMENTATION: SHIPPED
