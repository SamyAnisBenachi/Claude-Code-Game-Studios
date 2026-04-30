# Review Log — Auction System

## Review — 2026-04-30 (pass 3) — Verdict: NEEDS REVISION → Resolved in-session

Scope signal: L
Specialists: game-designer · systems-designer · economy-designer · network-programmer · ux-designer · creative-director (senior, Opus)
Blocking items: 14 | Recommended: 12 | Nice-to-have: 2
Summary: 14 specification defects resolved in-session — no design rework required. Key fixes: AuctionSnapshot `last_accepted_bid` sentinel harmonised to 0 (NP source of truth), `starting_price` field added to match NP R3; Rule 5 timer reset hardcode replaced with config reference (knob was non-functional); tick delta clamp added against lag-spike premature resolution; `saturating_sub` guards in `spend_reserved_gold` and bid validation; RESOLVING declared uninterruptible; M7-c added for outbid-player gold broadcast; AU8 split; AU20 reformulated for testability; bid buttons now require total-amount primary label; timer ease-out target clarified as fixed snapshot; leading-state UI three-way inconsistency resolved (buttons hidden). OQ7 deadline moved forward to before M2 implementation begins — all three current mitigation candidates found structurally flawed by economy-designer; creative-director accepted the finding. Design-level critiques (preset buttons, dead-zone wait loop) maintained as RECOMMENDED with M2 telemetry gates per creative-director adjudication — Pass 1 preset-button decision stands pending empirical data.
Prior verdict resolved: Yes (pass 2 in-session revision + re-review condition met)

---

## Review — 2026-04-29 (pass 2) — Verdict: NEEDS REVISION → Resolved in-session

Scope signal: L
Specialists: game-designer · systems-designer · economy-designer · network-programmer · ux-designer · qa-lead · creative-director (senior, Opus)
Blocking items: 14 | Recommended: 12 | Nice-to-have: 8
Summary: Prior approval condition (`reserved_gold` in `S2CGoldBroadcast`) was still unmet — added in NP GDD and auction interactions table. Three new config gaps closed: `legendary_pool_entry_round` and `auction_max_duration_seconds` added to game-config.md. Four new ACs added (AU20 Rule 5 atomicity, AU7-a reservation-zero assertion, AU19 split note, AU8 integration reclassification). UI Requirements overhauled: preset bid buttons primary, rarity text labels required (accessibility), shop lock affordance specified, hand-full reactive state added. OQ7 rewritten with Pillar Risk callout and M3 deadline; OQ8 first-bid bonus mitigation stricken (corrupts signal). Creative-director adjudicated: OQ7 not blocking GDD approval if time-boxed; bid-padding is RECOMMENDED not BLOCKING; AU7-b stays BLOCKING. OQ2/OQ3 closed as resolved.
Prior verdict resolved: Yes (14 blockers from pass 1 were already resolved; this pass addressed the re-review condition + 14 new items).

---

## Review — 2026-04-29 — Verdict: MAJOR REVISION NEEDED → Resolved in-session

Scope signal: L  
Specialists: game-designer · systems-designer · economy-designer · network-programmer · qa-lead · creative-director  
Blocking items: 14 | Recommended: 12  
Summary: Review identified two design-level blockers (shop interactivity three-document conflict and 1g bid increment not delivering the stated "predatory patience" fantasy) plus twelve specification defects (u32 timer underflow, missing reserved_gold in broadcast payload, no-op not specified on first bid, etc.). All 14 blocking items and all recommended fixes were resolved in-session. Creative-director adjudicated: shop NOT interactable during DRAFT_AUCTION; variable bid sizing as primary mechanic; Legendary pool stratification to Round 6+; pool state intentionally hidden. Re-review needed after network-protocol.md is updated to add `reserved_gold` to `S2CGoldBroadcast` payload (NP GDD update required before auction implementation).  
Prior verdict resolved: N/A — first review.
