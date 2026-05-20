# PROMPT 1531 — BOT-PARTICIPANT-ACTION-LOOP-WAVE1

**Status:** SHIPPED
**Base:** `origin/main@5358aed1a6075aca621936fd14f561be8fb854d3`
**Branch:** `worker/prompt-1531-bot-participant-action-loop-wave1`
**Worktree:** `D:/Tmp/wt-1531`

## Scope delivered

Server-side deterministic bot action loop for the per-round phases that
otherwise stall when one occupant is a bot. Wave 1 is intentionally flow-only:
the bot does not bid, does not buy from shop, and does not place units. It
emits the minimum set of internal server signals required for the RSM to
advance past Draft / Auction / Placement so a human can play a complete round
against a bot opponent.

Per phase behavior (gate-by-gate):

| Phase           | Bot action                                                                              |
|-----------------|-----------------------------------------------------------------------------------------|
| `Lobby`         | unchanged — owned by PROMPT 1514 `bot_lobby_auto_confirm`                                |
| `DraftInitial`  | writes `DraftReadySignal { player, ready: true }` once (idempotent via `draft_ready_players`) |
| `DraftAuction`  | intentional pass — logs `BotDecisionKind::AuctionPass` once per `(player, round)`; safety timer settles |
| `DraftShop`     | writes `DraftReadySignal { player, ready: true }` once                                  |
| `Placement`     | writes empty `PlacementSubmissionReceived` once (idempotent via `submissions_received`) |
| `Resolution`    | no-op                                                                                   |
| `GameOver`      | no-op                                                                                   |

Every decision branch appends a single audit entry to `BotDecisionLog` with
seed + word-counter snapshot per ADR-005 audit-log convention. Wave-1
intentionally consumes zero RNG entropy; the seed counter stays at 0.

## Files

Owned-scope only — no protocol/client/launcher edits.

| Path                                   | Change                                                                                      |
|----------------------------------------|---------------------------------------------------------------------------------------------|
| `server/src/feature/bot/action_loop.rs`| **new** — `bot_action_loop` system, `BotActionLoopPlugin`, 7 unit tests                     |
| `server/src/feature/bot/mod.rs`        | re-export `BotActionLoopPlugin` and the system                                              |
| `server/src/main.rs`                   | register `feature::bot::BotActionLoopPlugin` next to `BotLobbyPlugin`                       |
| `reports/PROMPT-1531-bot-participant-action-loop-wave1.md` | this report                                                              |

No protocol change. No `Cargo.toml` change. No CI change.

## Design notes

- **No new protocol surface.** The bot writes the *internal*
  `DraftReadySignal` and `PlacementSubmissionReceived` messages — the same
  messages the existing network drain systems produce after resolving a real
  C2S payload to a `PlayerId`. The contract is reused, not extended. This
  satisfies the prompt instruction to stop and report instead of broadening
  the protocol on a flow-completion wave.
- **Idempotency via RSM-owned state.** `rsm.draft_ready_players` and
  `rsm.submissions_received` are already cleared on phase entry, so a per-bot
  per-phase bookkeeping field is unnecessary for those two phases. The
  auction phase has no analogous server set; the system uses a Bevy `Local`
  `HashSet<(PlayerId, u32)>` keyed on `(player, round_number)` so the log
  gets exactly one entry per bot per auction round.
- **Determinism.** Inputs are `RoundState.phase`, `RoundState.round_number`,
  and `BotPlayers` membership. No wall-clock comparisons, no RNG draws. Two
  replays of the same session produce the same `BotDecisionLog` modulo
  `Time::elapsed` (recorded in the entry but not used for control flow).
- **Phase-transition latency.** The system uses unordered `MessageWriter`s,
  which means the signal lands on the next Bevy tick. That is the same
  latency human signals experience and is well below any phase timer.

## Tests

`cargo test -p server --lib feature::bot::action_loop` — 7/7 pass:

- `draft_initial_emits_ready_for_bot_only`
- `draft_ready_idempotent_after_rsm_records_signal`
- `draft_shop_also_emits_ready`
- `placement_emits_empty_failsafe_once`
- `auction_logs_pass_once_per_round`
- `idle_phases_emit_nothing`
- `humans_only_session_is_a_noop`

Coverage targets the system's gating logic in isolation. No broad cargo
suites were run (per prompt validation instructions).

## Deferred (explicit, not in scope)

- **Shop buy heuristic.** Bot does not consume the shop. A later wave should
  add a deterministic gold-threshold buy rule (e.g., buy the cheapest legal
  card if `gold > threshold`, otherwise pass + ready).
- **Auction bid heuristic.** Bot never bids. A later wave should derive a
  reservation price from card-value heuristics and place a single bid per
  round.
- **Placement heuristic.** Bot submits empty placement. A later wave should
  pick a legal lane/cell pair if the hand has any playable cards.
- **Result-screen acknowledgement.** Not part of wave-1 scope; the human
  acknowledges and the bot relies on safety/grace timers.
- **Live two-client smoke verification.** Deferred to later live QA per the
  prompt's "Heavy smoke/two-client verification deferred" instruction.

## Validation summary

- `cargo check -p server` — clean (only pre-existing client warnings remain,
  none in the touched files).
- `cargo test -p server --lib feature::bot::action_loop` — 7/7 pass.
- `git diff --check` — clean.
- Path allowlist: `server/src/feature/bot/**` + `server/src/main.rs` (one
  plugin registration) + `reports/**`. No client, launcher, asset, sprint, or
  CI files touched.

---

1531: BOT-PARTICIPANT-ACTION-LOOP-WAVE1: SHIPPED
