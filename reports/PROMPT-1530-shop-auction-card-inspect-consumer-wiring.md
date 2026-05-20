# PROMPT 1530 — SHOP-AUCTION-CARD-INSPECT-CONSUMER-WIRING

**Base**: `origin/main@5358aed1a6075aca621936fd14f561be8fb854d3`
**Branch**: `worker/prompt-1530-shop-auction-card-inspect-consumer-wiring`
**Worktree**: `D:/Tmp/wt-1530`

## Summary

Wired the shared `card_inspect` primitive (PROMPT 1482, adopted by hand /
DRAFT_INITIAL in PROMPT 1520) into the shop/auction card surfaces so the
local player can right-click any shop offer, DRAFT_INITIAL keep-9 slot, or
the live featured-auction card to open the enlarged inspect overlay,
dismissable with Escape, secondary-button-on-same-card, or a click on the
dim backdrop. Mirrors the hand/draft consumer registration shape exactly
so behaviour is identical across surfaces.

## Files changed (owned scope only)

- `client/src/ui/shop_auction/inspect.rs` — new module (added)
  - `ShopAuctionCardInspectTarget` `Resource` (current overlay target).
  - `ShopAuctionCardInspectRequested` / `ShopAuctionCardInspectDismissed`
    `Message` types.
  - `ShopAuctionCardInspectOverlayRoot` `Component` (overlay root marker).
  - `produce_shop_auction_card_inspect_requests_system` — reads
    `Pointer<Press>` (Secondary button), emits a request when the press
    lands on `ShopSlotCard`, `DraftInitialSlotCard`, or
    `AuctionFeaturedCard`. The featured-card branch reads the active
    `card_id` from `ShopAuctionAuctionState` (the entity itself does not
    carry the id).
  - `apply_shop_auction_card_inspect_target_system` — folds latest request
    + dismiss (message or Escape) into the target resource; toggle on
    same-card re-request; direct switch on different-card request.
  - `sync_shop_auction_card_inspect_overlay_system` — spawns/despawns the
    overlay tree on resource change using
    `spawn_card_inspect(parent, view)`; reuses
    `crate::ui::hand::inspect::build_card_inspect_view_from_card` as a
    pure read-only mapping (the only intentional cross-module reference;
    no behavioural coupling, hand state untouched).
  - `handle_shop_auction_card_inspect_backdrop_dismiss_system` — emits
    dismiss when the dim backdrop is pressed; `FocusPolicy::Block` on the
    inner card prevents bubbling.
  - 3 unit tests (open → repeat-close, dismiss message, card switch).

- `client/src/ui/shop_auction/mod.rs` — plugin wiring
  - Declared `pub mod inspect;`.
  - Plugin: `init_resource::<ShopAuctionCardInspectTarget>()`,
    `add_message::<ShopAuctionCardInspectRequested>()`,
    `add_message::<ShopAuctionCardInspectDismissed>()`,
    `add_message::<Pointer<Press>>()` (idempotent — PROMPT 696 mirror
    pattern for tests on `MinimalPlugins`).
  - Producer + backdrop-dismiss + target-fold systems chained at the end
    of the `Input` set so all signals settle in the same tick.
  - Overlay-sync system added at the end of the `StateSync` set.

- `reports/PROMPT-1530-shop-auction-card-inspect-consumer-wiring.md` —
  this report.

## Out of scope (not touched)

- `client/src/ui/hand/**` — hand inspect is unchanged; the single use of
  `build_card_inspect_view_from_card` is a read-only function import.
- `client/src/ui/draft/**` — no such directory; DRAFT_INITIAL lives under
  `shop_auction`.
- `shared/`, `server/`, sprint/session paperwork — not touched.

## Validation

- `cargo check -p client --lib` — clean (only pre-existing
  `ShopAuctionUiEntity` / `HandUiEntity` deprecation warnings in code I
  did not touch).
- `cargo test -p client --lib ui::shop_auction::inspect` —
  `test result: ok. 3 passed; 0 failed; 0 ignored`.
- `git diff --check` — clean (no whitespace errors).
- Path allowlist: every modified path lies inside the owned scope.

Broad Cargo verification is deferred to the VERIFY lane per the prompt's
implementation rules.

## Commits

Single commit on `worker/prompt-1530-shop-auction-card-inspect-consumer-wiring`
authored locally; pushed to origin if reachable, otherwise local-only with
the branch name relayed.

---

1530: SHOP-AUCTION-CARD-INSPECT-CONSUMER-WIRING: SHIPPED
