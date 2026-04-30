# Story 004: Manual Refresh Cost Formula and Counter Reset

> **Epic**: Card Acquisition
> **Status**: Ready
> **Layer**: Feature (M2)
> **Type**: Logic
> **Manifest Version**: 2026-04-30

## Context

**GDD**: `design/gdd/card-acquisition.md`
**Requirement**: `TR-CA-004`
*(Requirement text lives in `docs/architecture/tr-registry.yaml` — read fresh at review time)*

**ADR Governing Implementation**: ADR-015: Card Acquisition Shop State Machine Architecture
**ADR Decision Summary**: Manual refresh cost is computed as `refresh_base_cost + min(refresh_count_this_draft, refresh_cap)` (GDD Formula 1). `refresh_count_this_draft` lives in `PlayerShopState` and resets to 0 on every `DraftInitial`, `AuctionLock`, and `ShopOpen` trigger (fresh phase entry). A rejected refresh (insufficient gold) does NOT increment the counter.

**Engine**: Bevy 0.18 + Lightyear 0.26 | **Risk**: HIGH
**Engine Notes**:
- `Res<GameConfig>` access for `refresh_base_cost` and `refresh_cap` — no caching of field values; read from config each call
- `MessageReceiver<C2SRefreshShop>` is the Lightyear C2S receiver — confirm API with `liv-bevy-lightyear` skill

**Control Manifest Rules (Feature layer — from ADR-015):**
- Required: `refresh_count_this_draft` resets to 0 on every new DRAFT phase entry (`DraftInitial`, `AuctionLock`, `ShopOpen`)
- Required: `ShopUnlock` (DRAFT_AUCTION → DRAFT_SHOP) also resets `refresh_count_this_draft` to 0 (per ADR-015 ShopUnlock branch)
- Required: Rejected refresh (insufficient gold) must NOT increment `refresh_count_this_draft`
- Required: Read `refresh_base_cost` and `refresh_cap` from `Res<GameConfig>` — no hardcoded values

---

## Acceptance Criteria

*From GDD `design/gdd/card-acquisition.md`, scoped to this story:*

- [ ] **CA8** — GIVEN `refresh_base_cost=1`, `refresh_cap=1`, `refresh_count_this_draft=0`, WHEN manual refresh fires, THEN gold decrements by 1g and `refresh_count_this_draft` becomes 1.
- [ ] **CA9** — GIVEN `refresh_base_cost=1`, `refresh_cap=1`, `refresh_count_this_draft=1`, WHEN second manual refresh fires, THEN gold decrements by 2g (`1 + min(1,1) = 2`).
- [ ] **CA10** — GIVEN `refresh_base_cost=1`, `refresh_cap=1`, `refresh_count_this_draft=5`, WHEN refresh fires, THEN gold decrements by 2g (`1 + min(5,1) = 2`) — cap confirmed regardless of count.
- [ ] **CA11** — GIVEN `gold < refresh_cost` for the next refresh, WHEN `C2SRefreshShop` arrives, THEN rejected, gold unchanged, `refresh_count_this_draft` unchanged.
- [ ] **CA15** — GIVEN round N DRAFT_SHOP saw 3 refreshes, WHEN round N+1 DRAFT_SHOP begins and the player triggers their first manual refresh, THEN gold decrements by `refresh_base_cost` (1g at default) and `refresh_count_this_draft` is 1 — confirming it reset to 0 at the new phase entry.
- [ ] **CA22** — GIVEN round N DRAFT_AUCTION saw auto-refresh fire (auction round) followed by round N DRAFT_SHOP (no second refresh), WHEN round N+1 DRAFT_AUCTION entry fires, THEN the first manual refresh in round N+1 DRAFT_SHOP decrements gold by `refresh_base_cost` and `refresh_count_this_draft` is 1 — confirming the counter was 0 at the new DRAFT_AUCTION entry, not carried from round N.

---

## Implementation Notes

*Derived from ADR-015 Decision:*

**Formula 1** implementation within `card_acquisition_tick_system` manual refresh handler (step 2, `ShopActive` only):

```rust
// Drain C2SRefreshShop messages when phase == ShopActive
for msg in refresh_messages.receive_messages() {
    let player_id = msg.client_id; // mapped to PlayerId
    let shop = shop_states.players.get_mut(player_id)?;
    let refresh_cost = game_config.refresh_base_cost
        + shop.refresh_count_this_draft.min(game_config.refresh_cap);

    if economy.spend_gold(player_id, refresh_cost).is_err() {
        // CA11: reject silently — counter NOT incremented
        continue;
    }
    // Gold spent — proceed with draw (Story 003 pipeline)
    shop.refresh_count_this_draft += 1;
    // ... draw 3 slots, update displayed_this_draft, send S2CShopSlots
}
```

**Counter reset** — in `ShopRefreshTrigger` processing (step 1):
- `DraftInitial` → `refresh_count_this_draft = 0`
- `AuctionLock` → `refresh_count_this_draft = 0`
- `ShopOpen` → `refresh_count_this_draft = 0`
- `ShopUnlock` → `refresh_count_this_draft = 0` ← also resets on auction→shop unlock

**CA15 / CA22 test setup**: Simulate a full DRAFT phase with N refreshes, then inject a `ShopOpen` or `AuctionLock` trigger to start the next phase, then verify counter is 0 before the next manual refresh attempt.

**GameConfig fields**: `refresh_base_cost: u32` (default 1), `refresh_cap: u32` (default 1). Both already in `game-config.md` and `entities.yaml` per OQ1 resolution.

---

## Out of Scope

*Handled by neighbouring stories — do not implement here:*

- Story 003: The draw pipeline itself (slot fill, dedup, 50/50 split) — this story only covers the cost formula and counter; the draw execution delegates to Story 003's logic
- Story 005: Full purchase flow atomicity (CA18)

---

## QA Test Cases

- **CA8**: First refresh costs refresh_base_cost
  - Given: `refresh_base_cost=1`, `refresh_cap=1`, `refresh_count_this_draft=0`; economy has ≥1g
  - When: `C2SRefreshShop` processed in `ShopActive`
  - Then: economy.gold decremented by 1; `refresh_count_this_draft == 1`
  - Edge cases: exact gold match (1g, costs 1g → 0g remaining)

- **CA9**: Second refresh costs base+1
  - Given: `refresh_count_this_draft=1`; economy has ≥2g
  - When: `C2SRefreshShop` processed
  - Then: gold decremented by 2 (`1 + min(1,1)`); `refresh_count_this_draft == 2`

- **CA10**: Cap holds at count=5
  - Given: `refresh_count_this_draft=5`; economy has ≥2g
  - When: `C2SRefreshShop` processed
  - Then: gold decremented by 2 (`1 + min(5,1) = 2`); count increments to 6
  - Edge cases: very large count (u32::MAX would overflow `min` — confirm `min` saturates correctly)

- **CA11**: Insufficient gold → no state change
  - Given: economy.gold = 0; refresh_cost = 1
  - When: `C2SRefreshShop` processed
  - Then: economy.gold == 0 (unchanged); `refresh_count_this_draft` unchanged; no draw executed; no `S2CShopSlots` sent
  - Edge cases: gold exactly 1 less than cost; cost = 0 (not a valid GameConfig value per safe range)

- **CA15**: Counter resets between rounds
  - Given: `refresh_count_this_draft=3` after round N DRAFT_SHOP
  - When: `ShopRefreshTriggered { trigger: ShopOpen }` fires for round N+1
  - Then: `refresh_count_this_draft == 0`; next manual refresh costs `refresh_base_cost`

- **CA22**: Counter resets on AuctionLock entry (auction round)
  - Given: round N had `refresh_count_this_draft=2` at DRAFT_SHOP end; round N+1 fires `AuctionLock`
  - When: `ShopUnlock` fires (DRAFT_AUCTION → DRAFT_SHOP)
  - Then: `refresh_count_this_draft == 0`; first manual refresh in round N+1 DRAFT_SHOP costs `refresh_base_cost`

---

## Test Evidence

**Story Type**: Logic
**Required evidence**: `tests/unit/card_acquisition/refresh_cost_test.rs` — must exist and pass

**Status**: [ ] Not yet created

---

## Dependencies

- Depends on: Story 003 (`draw-pipeline`) must be Done — manual refresh delegates to the draw pipeline; counter increments after successful gold spend + draw
- Unlocks: None — this story is a leaf dependency; purchase flow (Story 005) does not depend on manual refresh cost
