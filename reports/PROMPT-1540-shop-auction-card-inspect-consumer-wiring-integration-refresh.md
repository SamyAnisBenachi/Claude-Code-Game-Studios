# PROMPT 1540 — Shop/Auction Card Inspect Consumer Wiring Integration Refresh

## Status

READY_FOR_MAINLAND_ENQUEUE

## Source / Target

- Source branch: `worker/prompt-1530-shop-auction-card-inspect-consumer-wiring` @ `b3743828151e70c2bdd35387a44512bfc9c01d17`
- Integration base: `origin/main` @ `f341d6c5156eb22544a05c1834d7179f560bf317`
- Integration branch: `integrate/shop-auction-card-inspect-1540`
- Integration commit: `be159aec6c87a095ea695160e5e3dd20ef288c97`

## Method

Created a fresh worktree from `origin/main` and cherry-picked the single PROMPT 1530
payload commit (`b3743828`) cleanly. No conflicts; no manual fixups required.

## Payload (delta vs origin/main)

```
client/src/ui/shop_auction/inspect.rs    | 275 +++++++++++++++++ (new)
client/src/ui/shop_auction/mod.rs        |  37 +++
reports/PROMPT-1530-shop-auction-card-inspect-consumer-wiring.md | 87 +++ (new)
3 files changed, 399 insertions(+)
```

The payload wires the shared `card_inspect` primitive (PROMPT 1482) into the three
shop/auction card surfaces (`ShopSlotCard`, `DraftInitialSlotCard`,
`AuctionFeaturedCard`) so the local player can right-click any shop offer, keep-9
slot, or live featured-auction card to open the enlarged inspect overlay. Dismiss
paths: Escape, secondary button on same card, backdrop click. Primary-button
presses (purchase/bid/pass) are untouched. Three unit tests cover the
target-fold state machine (open + repeat-close, dismiss, direct switch).

## Path allowlist review

All three files are within the allowed scope (shop_auction module + this PROMPT's
integration report passenger). No production/, sprint, QA, stage, Cargo, or
unrelated source modules were touched. PASS.

## Validation

- `git diff --check origin/main..HEAD` — PASS (no whitespace errors).
- Path allowlist review — PASS (see above).
- Broad Cargo verification — explicitly deferred per task contract (VERIFY lane
  will cover Cargo builds/tests).
- `liv-bevy-018` review — payload uses existing `mod.rs` registration shape that
  mirrors the PROMPT 1520 hand/draft consumer; no Bundle / pre-0.15 API patterns
  introduced. No deviations flagged on review of the diff.

## Push / Branch state

Local integration branch `integrate/shop-auction-card-inspect-1540` exists at
`be159aec`. Pushing is not required by this PROMPT; the orchestrator will
decide enqueue/push.

## Final line

1540: SHOP-AUCTION-CARD-INSPECT-CONSUMER-WIRING-INTEGRATION-REFRESH: READY_FOR_MAINLAND_ENQUEUE
