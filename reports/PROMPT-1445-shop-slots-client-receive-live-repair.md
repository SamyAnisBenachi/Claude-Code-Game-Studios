# PROMPT 1445 -- Shop Slots Client Receive Live Repair

Status: REPAIRED

## Root Cause

The live evidence showed the server sending `S2CShopSlots` as `DraftShop` opened, while the client still had the previous `Placement` phase locally. `handle_shop_slots_system` buffered slots during `DraftInitial`, `DraftAuction`, and `Resolution`, but not during `Placement`, so an early next-shop slot payload could be discarded before `DraftShop` state became active. The UI then remained in the `Waiting for shop offers...` placeholder with `ShopAuctionShopState.slots_loaded == false`.

The protocol receive bridge itself was already covered by `playable_client_protocol_receiver_drain_test`: `S2CShopSlots` can drain through `draft_shop_hand_bridge_fanout_system` into `ShopAuctionShopSlotsReceived`.

## Changed Files

- `client/src/ui/shop_auction/mod.rs`
  - Added `RoundPhase::Placement` to `should_buffer_shop_slots`.
  - This preserves early `S2CShopSlots` payloads across the `Placement -> DraftShop` client phase boundary and lets the existing `take_buffered_slots` path apply them once the shop UI is active.
- `tests/integration/shop_auction_ui/reconnect_late_message_test.rs`
  - Added a regression for slots arriving during `Placement`, then entering `DraftShop`.
  - The test proves UI state is populated, the shop panel is visible, all three offer card slots are visible, and the slots render as available card offers rather than the placeholder.

## Tests

Cargo policy applied: yes.

Environment used before Cargo commands:

```powershell
$env:CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'
$env:CARGO_PROFILE_DEV_DEBUG='0'
$env:CARGO_PROFILE_TEST_DEBUG='0'
$env:CARGO_INCREMENTAL='0'
$env:RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'
```

Targeted tests run:

```powershell
cargo test -p client --test shop_auction_ui_reconnect_late_message_test shop_slots_buffered_during_placement_apply_on_next_draft_shop
cargo test -p client --test shop_auction_ui_reconnect_late_message_test
cargo test -p client --test playable_client_protocol_receiver_drain_test s2c_shop_slots_drains_to_shop_auction_shop_slots_received
```

Results:

- `shop_slots_buffered_during_placement_apply_on_next_draft_shop`: passed.
- `shop_auction_ui_reconnect_late_message_test`: 7 passed.
- `s2c_shop_slots_drains_to_shop_auction_shop_slots_received`: passed.

Existing warnings: deprecation warnings for broad HUD/hand/shop QA markers and one dead-code warning in the protocol drain test helper. No new warnings were introduced by this repair.

## Live Retest

Live two-client retest remains required. This repair is source- and focused-test-verified, but it has not rerun the full native two-client QA scenario that produced snapshots `2-000004/005/006/016/017/020`.

1445: SHOP-SLOTS-CLIENT-RECEIVE-LIVE-REPAIR: REPAIRED
