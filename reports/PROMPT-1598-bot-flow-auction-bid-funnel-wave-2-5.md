# PROMPT 1598 — BOT-FLOW-AUCTION-BID-FUNNEL-WAVE-2-5

**Source-of-truth tip:** `origin/main @ 3a4603af`
**Branch:** `work/bot-flow-auction-bid-funnel-wave-2-5-1598`
**Worktree:** `D:/Tmp/wt-1598`
**Owner:** server-only worker
**Companion items in PROMPT 1594 inventory:** item 4 / ordering C — auction-bid
funnel Wave 2.5.

## Goal

Plumb the bot's chosen bid amount (decided by PROMPT 1582's Wave-2 heuristic)
into the authoritative server auction processing path so bot auctions can
actually clear with the bot as winner when the heuristic says to bid. Strictly
server-side; no protocol change; no rule shortcut.

## Approach

Introduced a single server-internal `Resource`, `PendingBotBids`, that lives
in the auction module. The bot action loop pushes one `AuctionBid` per
heuristic `Bid` decision into the queue; `auction_tick_system` drains the
queue every frame and merges the bot bids with the network `C2SPlaceBid`
batch before handing the combined batch to the existing `process_bid_batch`.
The same validation pipeline (price floor, leader gate, gold reservation,
hand-full gate, expiry gate) therefore applies uniformly to bots and humans.

No new wire types. Bots have no `PeerId`; queued bids carry `peer_id: None`
and the outbox layer's existing no-peer handling (broadcasts of acceptance
go to all connected peers; the per-player unicast for `S2CCardAcquired`
silently drops when no peer can be resolved) is the contract.

## Files changed

| File | Change |
|---|---|
| `server/src/feature/auction/system.rs` | Added `PendingBotBids` resource + `AuctionBidIntake` `SystemParam` bundle; `auction_tick_system` now drains the queue and chains it after network bids in `process_bid_batch`. Bundling is required because the system was at Bevy's 16-arg `IntoSystem` ceiling. |
| `server/src/feature/auction/plugin.rs` | `init_resource::<PendingBotBids>()` so any plugin that needs to fan a bid through the auction validator finds the queue ready. |
| `server/src/feature/auction/mod.rs` | Re-export `PendingBotBids`. |
| `server/src/feature/bot/action_loop.rs` | `bot_action_loop` and `run_auction_branch` take `Option<ResMut<PendingBotBids>>`; on `AuctionDecision::Bid` the bot now pushes the `AuctionBid` into the queue in addition to the existing decision-log entry. Tracing line updated from "not wired to auction yet" to "funnelled into PendingBotBids". |
| `server/Cargo.toml` | Registered new `[[test]]` entry for the funnel regression test. |
| `tests/unit/bot/bot_auction_bid_funnel_wave_2_5_test.rs` | New focused regression test: 4 cases proving the funnel enqueues on `Bid`, does NOT enqueue on `Pass`, the funnelled bid clears `process_bid_batch` and the bot becomes `current_leader` with the correct reservation, and the funnel is idempotent across re-ticks at unchanged `current_price`. |

## Tests

```
cargo test -p server --test bot_auction_bid_funnel_wave_2_5_test
   running 4 tests
   test test_bot_auction_bid_clears_through_process_bid_batch_and_becomes_leader ... ok
   test test_bot_auction_bid_funnel_idempotent_at_same_current_price ... ok
   test test_bot_auction_pass_does_not_enqueue_into_pending_bot_bids ... ok
   test test_bot_auction_bid_funnel_enqueues_decision_into_pending_bot_bids ... ok
   test result: ok. 4 passed; 0 failed.

cargo test -p server --test bot_auction_bid_decision_test
   running 9 tests
   test result: ok. 9 passed; 0 failed.

cargo test -p server --test auction_bid_validation_gate_test
   running 9 tests
   test result: ok. 9 passed; 0 failed.

cargo test -p server --lib feature::bot
   running 7 tests
   test feature::bot::action_loop::tests::auction_logs_pass_once_per_round ... ok
   test feature::bot::action_loop::tests::placement_emits_empty_failsafe_once ... ok
   test feature::bot::action_loop::tests::humans_only_session_is_a_noop ... ok
   test feature::bot::action_loop::tests::draft_initial_emits_ready_for_bot_only ... ok
   test feature::bot::action_loop::tests::draft_shop_also_emits_ready ... ok
   test feature::bot::action_loop::tests::draft_ready_idempotent_after_rsm_records_signal ... ok
   test feature::bot::action_loop::tests::idle_phases_emit_nothing ... ok
   test result: ok. 7 passed; 0 failed.
```

Broad verification (workspace test) deferred to the VERIFY lane per PROMPT
owned-scope.

## Compile gate

`cargo check -p server` finished `dev` profile clean; the previously failing
"`fn auction_tick_system` is not a system set" / "no method named `in_set`"
errors were caused by exceeding Bevy 0.18's 16-arg `IntoSystem` impl ceiling
when adding the bot-bid resource as a 17th param. The `AuctionBidIntake`
`SystemParam` bundle (network receivers + bot queue, both bid-intake sources)
restored the system to a registrable signature without splitting it further.

## Coordination notes

- **PROMPT 1597 (server QA snapshot):** does not touch `BotDecisionLog` public
  shape from this side — the auction-bid funnel writes the same
  `BotDecisionKind::AuctionBid { card_id, amount, valuation }` variant that
  PROMPT 1582 introduced, with the wave-2 decision-log entry preserved and a
  new push into `PendingBotBids` running alongside it. Safe to land in
  either order with PROMPT 1597.
- **No protocol shape changed.** `shared/src/protocol.rs` was only read for
  `C2SPlaceBid` reference, never modified.
- **No client edits.** Per owned-scope.
- **No placement Wave 3.** Bot still emits the empty placement fail-safe;
  the funnel is auction-only.

## Out-of-scope follow-ups

- Bot reservation/gold accounting under high-velocity counter-bids: the
  queue is per-frame; if a human raises the price between the bot's
  decision tick and the auction drain on the same frame, the bot's stale
  bid is rejected as `AmountTooLow` (validated by
  `auction_bid_validation_gate_test::at_price_and_below_price_rejected_as_amount_too_low`).
  Re-evaluation on the next tick is already handled by the
  `last_decision_price != current_price` re-evaluation gate from PROMPT
  1582.
- Bot-side card-acquired unicast: today the outbox attempts the bot's
  `S2CCardAcquired` unicast, fails peer resolution (`peer_id = None`), and
  logs a `DROPPED — peer_id unresolved` warning. State mutation is
  unaffected. PROMPT 1597 / a later wave can suppress the warning if it
  pollutes the QA snapshot.

## Status

1598: BOT-FLOW-AUCTION-BID-FUNNEL-WAVE-2-5: SHIPPED
