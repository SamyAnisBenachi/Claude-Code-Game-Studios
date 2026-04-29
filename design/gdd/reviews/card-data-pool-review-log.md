# Review Log — card-data-pool.md

---

## Review — 2026-04-29 (Pass 3) — Verdict: MAJOR REVISION NEEDED → APPROVED (post-revision)

Scope signal: M (implementation) / L (revision cost — cross-GDD audit required)
Specialists: game-designer, systems-designer, economy-designer, qa-lead, creative-director
Blocking items: 7 | Recommended: 7
Summary: Pass 3 applied cross-GDD pressure for the first time and found a direct contradiction between two Approved GDDs (server-rng.md specifies 2–3 seeds per shop slot; draw_shop_slot accepted only 1). Resolved via Option C — draw_shop_slot split into three phase-specific pool functions (draw_class_card, draw_neutral_family, draw_family_card) matching server-rng.md's audit log event_type design. OQ7 (shop/auction pool collision) resolved by user clarification: the auction is a shared/common pool per-game (one card per auction round, all players bid on the same card), entirely separate from each player's personal shop pool — no collision exists. C-B4 resolved: refresh policy documented (Card Acquisition owns display-slot state and fallback; pool documents no-tracking policy). 12 new ACs added, CP9 reclassified BLOCKING. Economy System dependency direction in Interactions table corrected.
Prior verdict resolved: Yes — all 7 blockers addressed in same session. OQ7 closed by design decision.

---

## Review — 2026-04-28 (Re-Review Pass 2) — Verdict: NEEDS REVISION → APPROVED

Scope signal: M (implementation cost)
Specialists: game-designer, systems-designer, economy-designer, network-programmer, qa-lead, creative-director
Blocking items: 6 | Recommended: 14
Summary: Re-review surfaced a new layer of issues not visible in the first cycle: (1) `count_owned` measured snapshot hand+board rather than commitment — replaced with `total_acquired` (monotonically increasing); (2) "card type" for class slot weighting was undefined — now explicitly `card_id`; (3) three startup validation scenarios (duplicate id, missing rarity, SHOP_WEIGHT_CAP=0) required hard-fail assertions; (4) Epic-at-auction contradicted class-specific Epic definition — removed Epic from draw_auction_card(); (5) no reconnection recovery path — S2CPoolSnapshot added; (6) §3.5 auction starting price contradicted §4.4 — fixed to Rare=3g, Epic=4g, Legendary=5g. All 6 resolved in same session. OQ7 (shop/auction pool division) remains open — hard gate before Auction System GDD.
Prior verdict resolved: Yes — all 6 blockers addressed in same session.

---

## Review — 2026-04-28 (Revision Pass) — Verdict: NEEDS REVISION → Revised

Scope signal: L (decision cost) / M (implementation cost)
Specialists: game-designer, systems-designer, economy-designer, network-programmer, qa-lead, creative-director
Blocking items resolved: 12/12 | Recommended items resolved: ~12
Prior verdict resolved: Yes — all 12 blockers addressed in same session

Summary: All 12 blockers were resolved after 4 design decisions made by user: (1) formula values locked at 0.10/0.65; (2) auction pool architecture confirmed as per-player for all rarities; (3) initial draft algorithm specified as fully random from catalog (any rarity); (4) archetype pivot deferred to playtesting. Formula bugs fixed in both GDDs. PoolFilter type defined. 3 missing S2C message types added. 5 missing ACs added (CP-A, CP-B, CP-C, CP5b, CP16b). Naming normalized to "Field" throughout. Open questions OQ6 (DoubleFace schema) and OQ7 (auction draw mechanics) added. Recommend re-review in fresh session before implementation begins.

---

## Review — 2026-04-28 — Verdict: MAJOR REVISION NEEDED

Scope signal: L
Specialists: game-designer, systems-designer, economy-designer, network-programmer, qa-lead, creative-director
Blocking items: 12 | Recommended: 12
Prior verdict resolved: N/A — first review

Summary: All 8 sections present. Core design is sound (per-player pool, archetype weighting formula, rarity ladder). 12 blocking items are primarily unresolved decisions and formula bugs rather than structural design failures. Key decisions needed before fixes can be applied:
1. Auction pool architecture: per-player or shared for Epic/Legendary? (creative-director recommends shared for Epic+Legendary)
2. draw_initial_draft rarity distribution: no Epics/Legendaries in opening 5g draft?
3. Formula 3 (Spawn Range) relocation: move to Board/Lane System GDD?

Formula bugs to fix (unambiguous):
- Formula 1: condition `≠ 0` should be `> 0` (negative values bypass validation)
- Formula 2: SHOP_WEIGHT_CAP = 0 causes division by zero; safe range must exclude 0.0; add |eligible_types| > 0 precondition
- Formula 3: add max(0, fakes_destroyed) guard
- §4.8 of master GDD still shows old values (0.05/0.80); must match card-data-pool.md (0.10/0.65)

Naming inconsistencies to fix:
- "Field" / "Passive/Aura" / "Passive" → normalize to "Field" everywhere
- "Row" vs "Cell" in ACs → normalize to "Row"
- Overview still mentions "Order and Double-Face" card types (both were cut)
- Economy System dependency: "copies_remaining for shop cost display" is wrong — cost is rarity-based and static

Missing ACs to add (all BLOCKING):
- CP-NEW-A: draw_auction_card() returns None when all Epic/Legendary neutral copies exhausted
- CP-NEW-B: draw_random(filter, seed) returns None when filter-matching subset exhausted
- CP-NEW-C: draw_initial_draft() returns exactly 9 distinct card IDs
- CP-NEW-D: pool_copies_override: -1 triggers validation error
- CP-NEW-E: pool_copies_override: 2 on Rare → copies_remaining = 2, not 4

Missing definitions to add:
- PoolFilter type schema (used by draw_random() but never defined)
- draw_initial_draft() algorithm spec (rarity distribution, class/neutral ratio)
- draw_auction_card() algorithm spec (rarity weighting, shared vs per-player)
- S2C message types: S2CShopSlots, S2CDraftOffering, S2CAuctionCard

To continue: open a fresh session and run:
  /design-system retrofit design/gdd/card-data-pool.md
