# PROMPT 2038 — Card Asset Shop Placeholder Binding Repair

Branch: `work/PROMPT-2038`
Worktree: `D:\_DEV\Work\gcs-app-worktrees\lanesandlies\PROMPT-2038`
Source-of-truth at start: `origin/main@8f7d3502`

## Scope

Audit + repair card-art / asset-binding for first-round shop, draft-initial
9-card grid, auction featured card, and the shared card-display-art helper
chain. Touch only `client/src/asset_wiring.rs`, `client/src/ui/shop_auction/`,
and focused asset-binding tests.

Explicitly out of scope: hand-fan layout, drag/drop, server gameplay,
autoplay, sprint/session-state docs.

## Asset Inventory (audit results)

### Card display art — full coverage

All 16 cards in `assets/data/cards.json` (ids 1–8 + 101–108) have both the
`display` and `zoom` variants on disk:

```
assets/art/cards/display/card_<art_id>_art_display.png   — 16 / 16
assets/art/cards/zoom/card_<art_id>_art_zoom.png         — 16 / 16
```

`client/src/asset_wiring.rs::resolve_card_display_art` constructs the path
`art/cards/display/card_{art_id}_art_display.png` for every non-empty,
non-sentinel `art_id`, which matches the on-disk layout 1-for-1. **No card
art is silently missing.** The `probe_card_display_art_paths` startup probe
will emit no `warn!`s for the current catalog.

### Shop / draft-initial / auction featured surfaces — binding verified

- **Draft-initial 9-card grid** (`spawn_draft_initial_grid` →
  `handle_draft_offering_system`): per-slot `CardSlotArtImage` child is
  spawned with `card_slot_art_image_component()` and bound through
  `apply_card_display_art(art_entity, …)` (`client/src/ui/shop_auction/mod.rs:2645`).
  Chrome preserved on the Err / clear path per the
  `chrome-preservation` contract.
- **Mid-round shop slots** (`spawn_shop_slots` → `apply_shop_slot_card` /
  `clear_shop_slot`): card art binds on the slot root, replacing the
  spawn-time `SHOP_SLOT_WELL_IDLE_ASSET` chrome when a card is present
  (`client/src/ui/shop_auction/mod.rs:7026, 7041`).
- **Shop footer (locked carry-over slots)** (`apply_shop_footer_slot`):
  binds via the same helper at line 7075.
- **Auction featured card** (`spawn_auction_contents` →
  `S2CAuctionCard` handler at `:4199, :4221`): canonical
  `CardSlotArtImage` child sized to `CardSlotKind::AuctionFeatured`;
  `apply_card_display_art` binds on that child, not the slot root.

All four surfaces route through the single chokepoint
`apply_card_display_art` in `client/src/asset_wiring.rs`. The binding
pipeline is correct.

### Placeholder bindings where a real asset (or sane non-`?` fallback) exists

The constants block in `asset_wiring.rs` annotates each repointed asset
with a `NO ANALOGUE on disk` comment. In owned-scope review, only one
constant is shop / auction domain:

| Constant                       | Previous target                                            | New target                            | Rationale |
|--------------------------------|------------------------------------------------------------|---------------------------------------|-----------|
| `BID_BUTTON_HOVER_ASSET`       | `art/characters/ui_unit_placeholder_default_board.png` (`?`) | `BID_BUTTON_NORMAL_ASSET` (`ui_bid_button_active.png`) | Hover variant PNG never landed; using the active-state PNG keeps the chrome stable on hover instead of flashing a question-mark glyph. Visual no-op vs Normal until a real hover asset is authored. |

Out-of-scope `NO ANALOGUE` constants (not touched — owned by HUD / hand /
character pipelines):

- `STAT_BADGE_AR_ASSET` (hand chrome — armor badge).
- `HUD_PHASE_TIMER_BAR_ASSET` (HUD phase timer bar).
- `HUD_OBJECTIVE_DOT_DESTROYED_ASSET` (HUD scoreboard objective dot).

These continue to render the universal `?` glyph; flagged below for
follow-up PROMPTs.

## Fixed Surfaces

1. **`client/src/asset_wiring.rs:46-53`** — `BID_BUTTON_HOVER_ASSET`
   re-pointed from the universal placeholder PNG to
   `BID_BUTTON_NORMAL_ASSET`. Hovering the bid button no longer flashes
   `?` chrome. The constant remains a single chokepoint that can be
   re-pointed once a real hover PNG lands; no consumer code change
   needed at that point.

2. **`tests/integration/presentation/asset_wiring_foundation_test.rs:274-303`**
   — `test_bid_button_selector_covers_all_variants` renamed and rewritten
   as `test_bid_button_selector_never_routes_to_placeholder`. The prior
   "distinct path per variant" guard tacitly required the placeholder
   re-point to stay in place (otherwise Normal/Hover would collapse to
   the same path). The new assertion expresses the intent directly: no
   `BidButtonChromeState` variant may resolve to
   `PLACEHOLDER_FALLBACK_ASSET`. When a real hover PNG lands the test
   stays green automatically.

## Validation

- **Path allow-list**: only `client/src/asset_wiring.rs` and
  `tests/integration/presentation/asset_wiring_foundation_test.rs`
  modified (plus this report). All within the owned scope.
- **`git diff --check`** on owned files: clean (no trailing whitespace,
  no conflict markers). Pre-existing `.claude/settings.json` whitespace
  is from session-start hook scaffolding, not authored here.
- **Broad cargo suites not run** per the PROMPT validation policy.
- **Focused test**: rewritten asset-foundation test covers the new
  invariant.

## Remaining Missing Assets (report-only — not in this PROMPT's scope)

Assets explicitly annotated `NO ANALOGUE on disk` in `asset_wiring.rs`
that still resolve to the universal `?` placeholder and would benefit
from a follow-up PROMPT in their owning surface:

1. `STAT_BADGE_AR_ASSET` — armor stat badge. Owned by hand-chrome / HUD.
   Cards 2, 5, 8 carry `ar > 0`; their AR badge currently shows `?` in
   the hand fan and zoom panel. Either author
   `art/ui/hand/ui_badge_ar_default_hud.png` or fall back to the HP
   badge with a recolor.
2. `HUD_PHASE_TIMER_BAR_ASSET` — HUD phase-timer fill bar. Owned by HUD.
   Either author `art/ui/hud/ui_phase_timer_bar_fill.png` or render the
   bar as a styled UI node (no image needed).
3. `HUD_OBJECTIVE_DOT_DESTROYED_ASSET` — destroyed-objective scoreboard
   dot. Owned by HUD. `env_objective_fake_crack_board.png` already
   exists on disk and is a reasonable visual approximation
   ("cracked / broken") that would avoid the `?` glyph until a
   dedicated `env_objective_destroyed_board.png` lands.
4. `ui_bid_button_hover.png` — bid-button hover variant. The temporary
   fallback to the active PNG ships in this PROMPT; the real hover
   variant remains to be authored.

These are visual identity issues, not binding bugs — the binding
pipeline is correct; the underlying PNGs are simply absent.

## Files Touched

```
client/src/asset_wiring.rs                                       (+10 −2)
tests/integration/presentation/asset_wiring_foundation_test.rs   (+19 −9)
reports/PROMPT-2038-card-asset-shop-placeholder-binding-repair.md (new)
```

2038: CARD-ASSET-SHOP-PLACEHOLDER-BINDING-REPAIR: SHIPPED
