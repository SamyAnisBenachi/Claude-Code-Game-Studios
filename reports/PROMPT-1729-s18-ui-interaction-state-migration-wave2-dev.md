# PROMPT-1729 — S18-UI-INTERACTION-STATE-MIGRATION-WAVE2-DEV

**Status**: SHIPPED
**Branch**: `work/s18-ui-interaction-state-migration-wave-2`
**Commit**: `32572a04`
**Base**: `origin/main@cbf4479d`
**Story**: `production/epics/ui-clean-pass/story-025-ui-interaction-state-migration-wave-2.md`

---

## AC Coverage

| AC | Description | Status |
|----|-------------|--------|
| AC1 | Lobby Confirm CTA Hovered/Pressed bands use `HOVER_BG_TINT_ALPHA` / `PRESSED_BG_TINT_ALPHA`; InFlight/Waiting/Confirmed grandfathered | DONE |
| AC2 | `LobbyCreateRoomButton` + `LobbyJoinRoomButton` 4-state overlay via `lobby_create_join_interaction_overlay_system` + named constants | DONE |
| AC3 | `ShopReadyButton` + `ShopRefreshButton` covered by `shop_auction_primary_button_interaction_overlay_system` | DONE |
| AC4 | `AuctionBidButton`×3 per-frame overlay after `sync_auction_panel_system`; `AuctionPassButton` via primary-button overlay; `AUCTION_PASS_BUTTON_BG/BORDER` constants replace inline literals | DONE |
| AC5 | `HandSubmitButton` overlay via `hand_submit_button_interaction_overlay_system`; `HandSubmitInteractionState::Inactive` → disabled tint; `Active` → hover/pressed tokens | DONE |
| AC6 | Status chips carry no `Interaction` — regression guard satisfied (pre-existing, not touched) | DONE |
| AC7 | `CursorIcon::System(SystemCursorIcon::Pointer)` on all P1 button spawns: lobby Create/Join/Confirm, draft-initial Ready/Dismiss/Retrieval, ShopReady, ShopRefresh, AuctionPass, AuctionBid×3, HandSubmit | DONE |
| AC8 | `client/src/ui/design_tokens/interaction_states.rs` zero diff | DONE |
| AC9 | New `tests/integration/ui_clean_pass/interaction_state_consumer_coverage_test.rs` + `client/Cargo.toml` entry | DONE |
| AC10 | No new RGB literals at consumer spawn sites (Create/Join use named constants; AuctionPass uses `AUCTION_PASS_BUTTON_BG/BORDER`) | DONE |

---

## Files Changed

| File | Change |
|------|--------|
| `client/src/ui/lobby.rs` | `CursorIcon` import; LOBBY_CREATE/JOIN constants; Hovered/Pressed bands → tokens; guard consts removed; CursorIcon on Create/Join/Confirm spawns; `apply_interaction_tint` helper; `lobby_create_join_interaction_overlay_system`; registered in `LobbyUiPlugin::StateSync` |
| `client/src/ui/shop_auction/mod.rs` | `CursorIcon` import; `AUCTION_PASS_BUTTON_BG/BORDER` constants; `HOVER/PRESSED` token imports; CursorIcon on DraftInitialReady/Dismiss/Retrieval/ShopRefresh/ShopReady/AuctionPass/AuctionBid×3 spawns; `apply_interaction_tint` helper; `shop_auction_primary_button_interaction_overlay_system`; `auction_bid_button_interaction_overlay_system`; both registered in `ShopAuctionUiPlugin::StateSync` chain |
| `client/src/ui/hand/mod.rs` | `CursorIcon` import; `HOVER/PRESSED/DISABLED` token imports; CursorIcon on `HandSubmitButton` spawn; `hand_submit_button_interaction_overlay_system`; registered in `HandUiPlugin::StateSync` chain |
| `tests/integration/ui_clean_pass/interaction_state_consumer_coverage_test.rs` | NEW — AC9 structural guard: CursorIcon presence, token import, overlay registration, AC10 literal regression |
| `client/Cargo.toml` | `[[test]]` entry for `ui_clean_pass_interaction_state_consumer_coverage_test` |

---

## Key Design Decisions

- **AuctionBidButton ordering**: `sync_auction_panel_system` writes `BackgroundColor` every frame; `auction_bid_button_interaction_overlay_system` runs immediately after it in the `.chain()` so hover tints survive the per-frame overwrite. Only `AuctionBidButtonState::Enabled` buttons receive tints — disabled/in-flight states keep sync colors.
- **AuctionPassButton**: same base colors as `primary_action_button_*` helpers → handled by the primary button overlay, not a separate system.
- **HandSubmitInteractionState**: does NOT drive `BackgroundColor` (only text), so `Changed<Interaction> | Changed<HandSubmitInteractionState>` overlay works without sync conflicts.
- **Bundle limit**: AuctionBidButton spawn hit Bevy's 15-element tuple Bundle limit with 16 items; `CursorIcon` moved to chained `.insert()` call alongside the existing `ImageNode` insert.
- **AC8 zero-diff**: `interaction_states.rs` not touched.

---

## Build Gate

`cargo check -p client` passes with zero errors. All warnings are pre-existing `#[deprecated]` markers on universal entity markers (`HudEntity`, `HandUiEntity`, `ShopAuctionUiEntity`).

1729: S18-UI-INTERACTION-STATE-MIGRATION-WAVE2-DEV: SHIPPED
