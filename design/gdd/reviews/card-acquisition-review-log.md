# Review Log — Card Acquisition

## Review — 2026-04-29 — Verdict: APPROVED (post-revision)
Scope signal: L
Specialists: game-designer · systems-designer · economy-designer · qa-lead · network-programmer · creative-director
Blocking items: 6 | Recommended: 8
Summary: Six blocking items resolved in-session: RSM Rule 12 gold forfeiture contradiction resolved (gold always carries over); S2CShopSlots schema corrected to Vec<Option<CardId>> in NP GDD; ERR_WRONG_PHASE references removed from CA GDD (NP Rule 4 silent discard is authoritative); double auto-refresh in auction rounds resolved (single fire at DRAFT_AUCTION entry, same slots persist to DRAFT_SHOP); CA18 rollback mechanism specified (refund_gold call); commitment fantasy addressed via shop_weight_per_card tuning note. Creative director flagged the commitment mechanic's statistical invisibility as the central design risk — recommend raising shop_weight_per_card to 0.20–0.25 during playtesting. Three new ACs added (CA20–CA22). OQs 1–2 resolved; OQs 3–5 remain open.
Prior verdict resolved: N/A — first review
