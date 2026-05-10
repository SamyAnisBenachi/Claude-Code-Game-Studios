# Sprint 10 Shop / Auction Panel Chrome MVP — Evidence (S10-POLISH-002)

> **Status**: Automated wiring evidence captured. Manual two-client friend-game
> route screenshot capture is **pending** (friend-game-lite paperwork pattern,
> per S10-TD-001 precedent — no live two-client run was performed in this
> implementation prompt because the worker environment has no display).
>
> **Story**: `production/epics/shop-auction-ui/story-014-panel-chrome-mvp.md`
> **Branch**: `work/s10-polish-002-shop-auction-panel-chrome`
> **Worktree**: `D:\_DEV\claude-code-game-studios-worktrees\s10-polish-002-shop-auction-panel-chrome`
> **Audit base commit**: `811de8a` (`origin/main` HEAD at branch creation).
> **Implementation commits**: see `git log work/s10-polish-002-shop-auction-panel-chrome..origin/main` in reverse.

## Friend-Game No-Claims Language

This evidence document does NOT claim:

- Public release readiness.
- Full asset approval (placeholder chrome PNGs from PAW-003 remain in use; final
  art replacement is accept-risk and carried by PAW-TD-003-a).
- Full playable-client manual QA.
- Broad Standard-tier accessibility completion.
- Playtest / fun-hypothesis validation.
- Closure of S8-QA-001-W1, QA-COND-0005, or QA-COND-0006.

Auction panel root reuses `SHOP_PANEL_CHROME_ASSET` as a placeholder until an
auction-specific chrome constant is added in a later story (PAW-TD-003-a is
accept-risk for friend-game scope). Auction border ramp tiles are not wired —
they have no spawn site in `client/src/ui/shop_auction/` and no asset constant
in `client/src/asset_wiring.rs`. Wiring those is out of scope for this MVP
verification story.

## AC Mapping

| AC  | Evidence | Status |
|---|---|---|
| AC-1 Panels consume `asset_wiring.rs` constants | `grep -rE '\.png\|\.jpg\|\.svg\|"assets/' client/src/ui/shop_auction/` returns zero hits. | PASS |
| AC-2 No `ImageNode` for board content | All `ImageNode` use is inside `client/src/ui/shop_auction/mod.rs` on `bevy_ui` `Node` entities (panel chrome, slot wells, bid buttons). No `Sprite::` use found in the same tree. | PASS |
| AC-3 Friend-game route visibly uses wired chrome | **Pending manual capture** (see "Manual Walkthrough Pending" below). Wiring is asserted by the integration test in AC-4. | PENDING |
| AC-4 Integration test asserts non-default `ImageNode.image` | New test `tests/integration/shop_auction_ui/chrome_wiring_test.rs` — 4/4 pass. Each test fixture transitions through `OnEnter(ClientState::InSession)` and asserts `ImageNode.image != Handle::<Image>::default()` on shop panel root, auction panel root, all three bid buttons, and all three shop slot wells. | PASS |
| AC-5 No new phase or economy drainer | `grep -rn 'MessageReceiver<S2CPhaseChanged>\|MessageReceiver<S2CGoldUpdate>' client/src/ui/shop_auction/` returns zero hits. | PASS |
| AC-6 SAU-007 + SAU-008 behaviour preserved | `cargo test --test shop_auction_ui_auction_settlement_test` 7/7 pass. `cargo test --test shop_auction_ui_reconnect_late_message_test` 6/6 pass. (One initial wall-clock-flaky run on the SAU-008 timer-tick assertion resolved on re-run; not caused by this story's wiring change.) | PASS |
| AC-7 Manual evidence document recorded | This document. Manual screenshots pending live two-client run (friend-game-lite paperwork). | PARTIAL |

## Automated Evidence

### New integration test

`tests/integration/shop_auction_ui/chrome_wiring_test.rs` — 4 tests:

- `shop_auction_panel_root_carries_non_default_image_node_after_on_enter_in_session`
- `shop_auction_auction_panel_root_carries_non_default_image_node_after_on_enter_in_session`
- `shop_auction_bid_buttons_carry_non_default_image_node_after_on_enter_in_session`
- `shop_auction_shop_slots_carry_non_default_image_node_after_on_enter_in_session`

Fixture follows the canonical partial-App pattern from
`tests/integration/shop_auction_ui/shop_panel_test.rs::app_in_session()`:
`MinimalPlugins` + `AssetPlugin::default()` + `init_asset::<Image>()` +
`StatesPlugin` + `init_state::<ClientState>()` + `ShopAuctionUiPlugin` +
`ShopAuctionCardCatalog` + `PlayerEconomyView` +
`NextState::<ClientState>::set(InSession)` + one `app.update()`.

### Implementation patch

`client/src/ui/shop_auction/mod.rs` — three-line patch after the
`spawn_panel_root(... ShopAuctionPanelRoot::Auction ...)` call to
`commands.entity(auction_panel).insert(ImageNode::new(asset_server.load(SHOP_PANEL_CHROME_ASSET)))`.

`client/Cargo.toml` — one new `[[test]]` block for
`shop_auction_ui_chrome_wiring_test`.

## Verification Run (local)

| Command | Result |
|---|---|
| `cargo fmt --manifest-path client/Cargo.toml -- --check` | PASS (exit 0) |
| `cargo check --manifest-path client/Cargo.toml --lib` | PASS |
| `cargo build --manifest-path client/Cargo.toml --lib` | PASS |
| `cargo test --manifest-path client/Cargo.toml --test shop_auction_ui_chrome_wiring_test --jobs 2` | 4/4 pass |
| `cargo test --manifest-path client/Cargo.toml --test shop_auction_ui_auction_settlement_test --jobs 2` | 7/7 pass |
| `cargo test --manifest-path client/Cargo.toml --test shop_auction_ui_reconnect_late_message_test --jobs 2` | 6/6 pass |

`CARGO_TARGET_DIR=target/msvc-local` (per project memory `project_tech_stack.md`
guidance for local Windows builds; CI remains authoritative).

## Manual Walkthrough Pending

- **Capture target**: one screenshot per phase (DRAFT_SHOP, DRAFT_AUCTION,
  auction settlement, post-auction DRAFT_SHOP) showing the wired chrome.
- **Blocker**: this implementation prompt ran in a non-display worker
  environment. A subsequent operator-led friend-game route capture is
  required before AC-3 / AC-7 close fully.
- **Recommended runner**: native two-client friend-game route per
  `production/qa/evidence/native-friend-game-operator-controls-evidence.md`
  conventions, or browser/WASM equivalent.
- **Storage**: place captured PNGs under
  `production/qa/evidence/captures/sprint-10-shop-auction-chrome/` and link
  them inline in the AC-3 / AC-7 rows above when available.

Lead sign-off: Pending manual walkthrough.

## References

- Story: `production/epics/shop-auction-ui/story-014-panel-chrome-mvp.md`
- PAW-003 (asset wiring substrate this story consumes):
  `production/epics/presentation-asset-wiring/story-003-shop-auction-chrome.md`
- ADR-021 (Presentation Layer Architecture):
  `docs/architecture/adr-021-presentation-layer-architecture.md`
- Sprint plan: `production/sprints/sprint-10.md` (S10-POLISH-002 row).
