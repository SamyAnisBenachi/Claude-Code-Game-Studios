# Sprint 14 Auction Free-Gold Counters Evidence

Story: `S11-UX-AUCTION-FREE-GOLD-COUNTERS`
Prompt: `PROMPT-958`
Date: 2026-05-16
Branch: `work/s14-auction-free-gold-counters-958`

## No-Claim Banner

Story 017 implements auction free-gold counter layout, composition,
typography, and readability only. It does not advance `QA-COND-0005`
(Standard-tier accessibility), `QA-COND-0006` (playtest / fun-hypothesis
validation), `PAW-TD-002-a`, `PAW-TD-003-a`, `S8-QA-001-W1`, the PROMPT 761
Polish->Release gate-check, public release readiness, release-candidate
readiness, full game completion, broad accessibility completion, playtest
validation, final-art completion, Sprint 14 close-out, or a stage advance.
All listed conditions remain accepted-risk / open per their existing
dispositions.

## UX Consultation / Local Fallback

The worker prompt requested CCGS UI programmer + UX-designer consultation if
available. Those named CCGS subagents were not available in this Codex worker
session, so implementation used the local fallback path.

Local UX decision:

- Grouping pattern: one shared `AuctionFreeGoldCounterGroup` flex row with two
  sibling `AuctionFreeGoldCounter` readouts.
- Counter semantics: `Interest` and `Bid Refund` source readouts both consume
  the existing `local_free_gold = gold - reserved_gold` UI/economy view path.
  No provenance-specific economy state exists in this presentation module, and
  adding one would exceed the layout-only story scope.
- Adjacency: group anchors from the +5 bid-button x-position plus one
  `SPACING_MD` gap, so the counters read as part of the bid decision cluster.
- Spacing: shared group padding is `SPACING_XS + 2px`; inter-counter gap is
  `SPACING_MD`.
- Typography hierarchy: labels use `typography::CAPTION`; numeric values use
  `typography::H2`, preserving value font size > label font size.

## Implementation Evidence

- Code: `client/src/ui/shop_auction/mod.rs`
- Test: `tests/integration/shop_auction_ui/auction_free_gold_counters_layout_test.rs`
- Test target registration: `client/Cargo.toml`
- Cargo target addition justification: the repository registers integration
  tests explicitly in `client/Cargo.toml`; adding this test target is required
  for `cargo test -p client --test shop_auction_ui_auction_free_gold_counters_layout_test`.

## Acceptance Evidence

- AC1: automated test asserts one shared `AuctionFreeGoldCounterGroup`, two
  sibling counters, two labels, two numeric values, stable marker components,
  and direct `ChildOf` relationships.
- AC2: automated test asserts panel-relative group offset adjacent to the bid
  cluster at 1366x768 and 1920x1080 with `0.01px` tolerance.
- AC3: automated test asserts numeric value font size (`H2`) is greater than
  label font size (`CAPTION`).
- AC4: automated test computes panel-relative rectangles for 1366x768 and
  1920x1080 and asserts no clipping / overlap with bid cluster, featured card,
  timer, bid status, or settlement text.
- AC5: automated test drives local gold broadcasts and asserts counter text and
  marker state match `gold - reserved_gold` on every tested frame, including
  saturating subtraction when `reserved_gold > gold`.
- AC6: regression targets are run separately per final report; new layout code
  does not change bid button state calculation, click send, focus state, or
  accepted/rejected handling.
- AC7: local fallback decision is recorded above.
- AC8: no `server/`, `shared/`, protocol, economy-authority, or new
  `MessageReceiver<S2CGoldUpdate>` / `MessageReceiver<S2CGoldBroadcast>` drain
  changes were made.
- AC9: no-claim banner recorded above.
- AC10: verification commands and results are recorded in the final report.
