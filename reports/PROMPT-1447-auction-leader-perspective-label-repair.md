# PROMPT 1447 -- Auction Leader Perspective Label Repair

Status: REPAIRED

## Root Cause

The auction UI had two local-player identity sources. `handle_auction_snapshot_system`
correctly stored the snapshot recipient in `ShopAuctionLocalGoldView.player_id`, but
auction follow-up systems preferred `HudPlayerIds.local_id` when it existed. In the
latest two-client evidence, the local client was `PlayerId(2)` while the HUD identity
path could still report `PlayerId(1)`. That stale HUD value caused a local `PlayerId(2)`
leader to be classified as the opponent, rendering `OPPONENT LEADING`.

## Repair

- Added a scoped `auction_local_player_id` helper in `client/src/ui/shop_auction/mod.rs`.
- Updated auction gold broadcast, bid accepted, and settlement systems to prefer the
  snapshot-owned `ShopAuctionLocalGoldView.player_id` and use `HudPlayerIds` only as a
  fallback when the auction-local identity has not been initialized.
- Left the leader render path's local/opponent comparison driven by
  `ShopAuctionLocalGoldView.player_id`.

## Changed Files

- `client/src/ui/shop_auction/mod.rs`
- `tests/integration/shop_auction_ui/auction_lead_loss_state_test.rs`
- `reports/PROMPT-1447-auction-leader-perspective-label-repair.md`

## Tests

Cargo policy applied: yes.

Policy used:

```powershell
$env:CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'
$env:CARGO_PROFILE_DEV_DEBUG='0'
$env:CARGO_PROFILE_TEST_DEBUG='0'
$env:CARGO_INCREMENTAL='0'
$env:RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'
```

Ran:

```powershell
cargo test -p client --test shop_auction_ui_auction_lead_loss_state_test
cargo test -p client --test shop_auction_ui_reconnect_late_message_test
```

Results:

- `shop_auction_ui_auction_lead_loss_state_test`: 7 passed.
- `shop_auction_ui_reconnect_late_message_test`: 7 passed.

An initial attempt used the source-file basename as the Cargo test target and failed
before compilation because the registered target name is
`shop_auction_ui_auction_lead_loss_state_test`.

## Coverage Added

- Local client is `PlayerId(2)`, HUD local id is stale `PlayerId(1)`, and local
  `PlayerId(2)` leads -> `YOU ARE LEADING`.
- Local client is `PlayerId(2)`, HUD local id is stale `PlayerId(1)`, and opponent
  `PlayerId(1)` leads -> `OPPONENT LEADING`.
- Local client is `PlayerId(2)`, HUD local id is stale `PlayerId(1)`, and no leader
  exists at auction start -> no misleading opponent-leading label.

## Reset Regression

The adjacent reconnect/snapshot test target was run because it covers auction snapshot
rebuild and stale leader/transient clearing. It passed, including the no-leader reset
cases.

## Live Retest

Live two-client retest remains required. This repair is covered by focused UI tests but
does not produce new live screenshots for the original QA snapshots
`2-000010-1779192904344` and `2-000024-1779193056562`.

Final line: `1447: AUCTION-LEADER-PERSPECTIVE-LABEL-REPAIR: REPAIRED`
