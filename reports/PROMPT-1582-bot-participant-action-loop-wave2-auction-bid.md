# PROMPT 1582 — BOT-PARTICIPANT-ACTION-LOOP-WAVE2-AUCTION-BID

**Status:** SHIPPED
**Base:** `origin/main@9be8827fbd22b2a49d973ba585b5d210fdc8a903`
**Branch:** `work/bot-action-loop-wave2-auction-bid-1582`
**Worktree:** `D:/Tmp/wt-1582`

## Scope delivered

Wave-2 deterministic auction bid decision behaviour for the server-side bot
participant action loop. The bot now reads `AuctionState`, `PlayerEconomies`,
and `PlayerHands` (all read-only) during the `DraftAuction` phase and records
one `BotDecisionKind::AuctionBid { card_id, amount, valuation }` per
`(player, round, current_price)` tuple when the heuristic decides to bid, or
one `BotDecisionKind::AuctionPass { reason }` with a precise static-literal
reason otherwise.

PROMPT 1582 is **decision-only** per the task brief: the bot does NOT push
any auction wire message in this wave. Auction-side ingestion (i.e. funnelling
the bot's chosen amount through `process_bid_batch` so the auction actually
clears) is deferred to a future wave that owns the auction module. PROMPT 1582
strictly respected the owned scope below and made no edits inside
`server/src/feature/auction/**`.

Per-phase behaviour after PROMPT 1582:

| Phase           | Bot action                                                                              |
|-----------------|-----------------------------------------------------------------------------------------|
| `Lobby`         | unchanged — owned by PROMPT 1514 `bot_lobby_auto_confirm`                                |
| `DraftInitial`  | writes `DraftReadySignal { player, ready: true }` once (unchanged)                       |
| `DraftAuction`  | **Wave 2** — heuristic emits `AuctionBid { … }` or `AuctionPass { reason }`              |
| `DraftShop`     | writes `DraftReadySignal { player, ready: true }` once (unchanged)                       |
| `Placement`     | writes empty `PlacementSubmissionReceived` once (unchanged)                              |
| `Resolution`    | no-op (unchanged)                                                                       |
| `GameOver`      | no-op (unchanged)                                                                       |

## Heuristic

The Wave-2 auction branch is fully deterministic given the bot's seeded
`ChaCha8Rng` and the observable auction state. Per `(player, round)`:

1. On the first tick the bot sees `AuctionState::phase == LiveBidding` with a
   non-`None` `card_id`, the bot draws **one** `u64` from its private RNG and
   computes a stable **reservation valuation** as
   `valuation = starting_price + 1 + (rng_word % (starting_price / 2 + 1))`.
   The `+1` floor guarantees the bot is willing to outbid the rarity-derived
   minimum legal bid; the noise headroom is capped at half the starting
   price by `BOT_AUCTION_VALUATION_NOISE_DENOMINATOR` so the bot never burns
   its bankroll on one card.
2. The valuation is cached in a Bevy `Local<HashMap<(PlayerId, u32),
   AuctionRoundContext>>` for the rest of the round so the RNG only ever
   advances once per round and the bot's reservation never drifts mid-round.
3. Each tick the bot consults the live `current_price`. If it has already
   produced a decision against this exact `current_price`, it stays silent
   (idempotency). If `current_price` rises (a human raised), the bot
   re-evaluates.
4. The bid/pass decision applies these gates in order:
   - `phase != LiveBidding` → pass `phase_not_live_bidding`
   - `auction.card_id != cached_card_id` → pass `card_changed_mid_round`
   - bot is current_leader → pass `already_leader`
   - hand full (`PlayerHands::hand_len >= MAX_HAND_SIZE`) → pass `hand_full`
   - `timer_remaining_ms < BOT_AUCTION_PASS_THRESHOLD_MS` → pass
     `timer_below_threshold`
   - `valuation < current_price + 1` → pass `valuation_below_min_bid`
   - `gold - reserved_gold < current_price + 1` → pass `insufficient_gold`
   - otherwise → bid `amount = valuation.min(unreserved_gold).max(current_price + 1)`
5. Every decision appends one `BotDecisionEntry` to `BotDecisionLog` with the
   bot's seed and the post-draw `seed_word_counter` snapshot per ADR-005
   audit convention.

When `AuctionState` is absent (test scaffolding that bypasses the auction
plugin) the Wave-2 path falls back to a single `AuctionPass {
auction_state_unavailable }` entry per round so the Wave-1 contract is
preserved — legacy tests continue to compile against the new branch.

## Files

| Path                                                          | Change                                                                                             |
|---------------------------------------------------------------|----------------------------------------------------------------------------------------------------|
| `server/src/feature/bot/state.rs`                              | +9 — new `BOT_AUCTION_VALUATION_NOISE_DENOMINATOR` constant + module doc                            |
| `server/src/feature/bot/action_loop.rs`                        | +388/−46 — Wave-2 heuristic, `AuctionRoundContext`, `draw_valuation`, `decide_auction_action`, `run_auction_branch`, signature widened with `Res<AuctionState>` / `Res<PlayerEconomies>` / `Res<PlayerHands>` |
| `tests/unit/bot/bot_auction_bid_decision_test.rs`              | +315 — **new** unit test file, 9 scenarios                                                          |
| `server/Cargo.toml`                                            | +5 — additive `[[test]] bot_auction_bid_decision_test` block, mirrors PROMPT 1428/1514 pattern      |

**Note on `server/Cargo.toml`**: the task brief lists "unrelated Cargo/CI
files" as forbidden. The 5-line additive `[[test]]` block here is **directly
related** to the owned test file (without it, the test binary is not built)
and follows the exact pattern PROMPT 1428 used for `bot_foundation_state_test`
and PROMPT 1514 used for `bot_lobby_loop_test`. No other Cargo or CI surface
was touched.

No protocol change. No client edit. No production/* edit. No auction-module
edit. No `.claude/settings.json` edit.

## Determinism notes

- Inputs to the heuristic are `RoundState.phase`, `RoundState.round_number`,
  `BotPlayers` membership, the observable `AuctionState` fields, the bot's
  unreserved gold, the bot's hand size, and the bot's private `ChaCha8Rng`.
  Wall-clock time is captured into the audit log but never gates a decision.
- The RNG is advanced **once per `(player, round)`**, not once per tick — so
  a replay with the same observable inputs produces the same valuation and
  the same bid sequence.
- All `Pass` reasons are `&'static str` literals (per
  `BotDecisionKind::AuctionPass`'s shape) so the decision log is
  allocation-free and grep-stable.

## Validation summary

Targeted only, per the prompt's "no broad Cargo suites" instruction.

- `cargo check -p server` — clean (worker uses `CARGO_TARGET_DIR=D:/_cargo/PROMPT-1582`).
- `cargo test -p server --lib feature::bot` — **7/7 pass** (all PROMPT 1531
  Wave-1 tests still pass; the previously named `auction_logs_pass_once_per_round`
  test exercises the new fallback path with reason `auction_state_unavailable`).
- `cargo test -p server --test bot_auction_bid_decision_test` — **9/9 pass**:
  - `test_bot_auction_emits_bid_when_gold_and_valuation_clear_gate`
  - `test_bot_auction_passes_when_unreserved_gold_is_insufficient`
  - `test_bot_auction_passes_when_hand_is_full`
  - `test_bot_auction_passes_when_bot_is_already_leader`
  - `test_bot_auction_passes_when_timer_below_threshold`
  - `test_bot_auction_decision_is_idempotent_at_same_current_price`
  - `test_bot_auction_reevaluates_when_current_price_rises`
  - `test_bot_auction_decision_is_deterministic_for_same_seed`
  - `test_bot_auction_fallback_when_auction_state_resource_is_absent`
- `cargo test -p server --test bot_lobby_loop_test --test bot_foundation_state_test`
  — **8 + 6 pass** (no regression).
- `git diff --check` — clean (no whitespace issues).
- Path allowlist: only the four files above. No edits outside
  `server/src/feature/bot/` plus the test file plus the additive
  `[[test]]` block.

Broad workspace `cargo test` deferred to a dedicated VERIFY lane per the
post-1471 orchestrator override ("broad Cargo out of implementation workers,
dedicated VERIFY prompts only" — see PROMPT 1581 §H reservation
`VERIFY-1594` for the post-1582/1583 bot test coverage refresh).

## Deferred (explicit, not in scope)

- **Auction-side wire ingestion.** The bot's chosen `amount` is recorded in
  the audit log only. A later wave must funnel the bot's bid into
  `process_bid_batch` (either by exposing a server-side `AuctionBid`
  `Message` queue that the auction module drains, or by giving the bot
  module a `&mut AuctionState`/`&mut AuctionNetworkOutbox` SystemParam).
  Touching `server/src/feature/auction/**` is explicitly outside PROMPT
  1582's owned scope.
- **Shop buy heuristic.** Unchanged from Wave 1 — bot still falls through
  `DraftShop` via the ready signal alone.
- **Placement heuristic.** Unchanged — bot still submits empty placement
  fail-safe.
- **Multiple-counter-bid valuation.** Wave 2 evaluates a single bid amount.
  `legal_action_count` is set to `Some(1)` for bid entries and `Some(0)` for
  pass entries; a future heuristic that considers multiple counter-bids per
  tick should report the true evaluated count there.
- **Live two-client smoke verification.** Deferred to a VERIFY lane chained
  off PROMPT 1580 per PROMPT 1581 §C.

## Non-claims

- No claim that the bot's bid currently reaches the auction settlement. The
  auction safety timer still settles the round without a winner unless a
  human bids; that wiring is a future wave.
- No Sprint 18/19 disposition flip; no `production/*` edit.
- No PROMPT 761 retry, no QA-COND advancement, no `S8-QA-001-W1` closure.
- No Krosmaga release/legal claim — PROMPT 1582 does not touch any Krosmaga
  surface.

---

1582: BOT-PARTICIPANT-ACTION-LOOP-WAVE2-AUCTION-BID: SHIPPED
