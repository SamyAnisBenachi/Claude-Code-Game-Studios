# PLAYABLE-002 Live Draft/Shop/Hand Bridge Evidence

## Scope

Friend-game draft/shop/hand bridge evidence only. This is not public release
readiness, not playtest validation, not broad accessibility completion, not full
playable-client manual QA, and not PLAYABLE-003 two-primary-client end-to-end
loop evidence.

## Build Target

- Worker branch: `work/playable-002-live-draft-shop-hand-bridge`
- Worktree: `D:\_DEV\claude-code-game-studios-worktrees\PLAYABLE-002`
- Base: `origin/main` at `f81a3f9bfc2be883606e8222aa783e9119151a2c`
- Implementation commit: `5077839bfc606dd46deea6baccdedc04e6ea75f0`

## Automated Evidence

The implementation was verified with focused server/client tests and relevant
regressions:

- `cargo test -p server --test playable_client_draft_ready_bridge_test` - PASS
- `cargo test -p client --test playable_client_draft_shop_hand_bridge_test` - PASS
- `cargo test -p server --test card_acquisition_purchase_atomicity_test` - PASS
- `cargo test -p server --test rsm_timers_test` - PASS
- `cargo test -p server --test rsm_network_dispatch_test` - PASS
- `cargo test -p server --test card_acquisition_draft_initial_test` - PASS
- `cargo test -p server --test economy_network_dispatch_test` - PASS
- `cargo test -p client --test shop_auction_ui_draft_initial_grid_test` - PASS
- `cargo test -p client --test shop_auction_ui_shop_panel_test` - PASS
- `cargo test -p client --test hand_ui_draft_initial_grid_test` - PASS
- `cargo test -p client --test shared_economy_view_test` - PASS
- `cargo fmt -p client -p server -- --check` - PASS
- `cargo check -p client` - PASS
- `cargo check -p server` - PASS
- `git diff --check origin/main...HEAD` - PASS

## Message Path Coverage

- `C2SSignalReady` now maps from the client connection peer to
  `DraftReadySignal { player, ready: !retract }`, preserving the existing RSM
  ready/retract/all-ready path.
- Successful DRAFT_INITIAL purchases produce owner-only `S2CCardAcquired` with
  `CardSource::DraftInitial` plus an owner `S2CGoldUpdate`.
- Successful DRAFT_SHOP purchases produce owner-only `S2CCardAcquired` with
  `CardSource::ShopPurchase`, `S2CShopSlots`, and an owner `S2CGoldUpdate`.
- Rejected purchases do not build acquisition or gold-update network events.
- A single production presentation fanout drains live `S2CDraftOffering`,
  `S2CShopSlots`, and `S2CCardAcquired`; Hand UI and Shop/Auction UI consume
  Bevy messages from that shared fanout instead of double-draining Lightyear
  receivers.
- `S2CGameSnapshot` rebuilds Hand UI hand contents and DRAFT_SHOP slots before
  additional live presentation messages are applied.

## Manual Capture Status

No two-real-client manual capture or screenshot set was taken for this story.
PLAYABLE-003 owns real two-primary-client loop evidence and full friend-game
path verification.
