# Review Log — Auction System

## Review — 2026-04-30 (pass 5, lean) — Verdict: APPROVED

Scope signal: L
Specialists: none (lean mode)
Blocking items: 0 | Recommended: 3
Summary: Fresh-session re-review confirming pass-4 in-session revisions. All 11 pass-4 blockers verified resolved. Three pre-implementation gates identified: (1) CardSource::AuctionWon vs AcquisitionSource enum name must be reconciled against network-protocol.md before story implementation (compilation risk); (2) RSM GDD requires a targeted update to document auction_max_duration_seconds as a second AbortAuction trigger per Rule 8; (3) OQ9 (AuctionExpired reachability) must be resolved by Gameplay Programmer before AU12 can be implemented. Recommended cleanups: migrate misplaced AU11 to Economy System GDD; promote CardSource naming inconsistency to a formal OQ; close OQ6 Card Data & Pool cross-GDD update. Document is complete, internally consistent, and fully implementable.
Prior verdict resolved: Yes (pass 4 MAJOR REVISION NEEDED resolved in-session; this pass confirms resolution)

---

## Review — 2026-04-30 (pass 4) — Verdict: MAJOR REVISION NEEDED → Resolved in-session

Scope signal: L
Specialists: game-designer · systems-designer · economy-designer · network-programmer · ux-designer · qa-lead · creative-director (senior, Opus)
Blocking items: 11 | Recommended: 8
Summary: Pass 4 surfaced three categories of issues. (1) Pillar-threatening design: the interest formula rewards hoarding over spending, inverting "No idle spectating"; OQ7's only proposed mitigation ("max bid capped at opponent's free gold") creates "never bid first" dominant strategy worse than the original problem — both resolved by accepting economic dominance as valid win condition alongside bidding skill, with Player Fantasy reframed and M2 monitoring gate added. (2) Correctness bugs: client minimum bid off-by-one (snapshot derived `starting_price` vs server-required `starting_price + 1`), `spend_reserved_gold` annotation describing wrong failure mode (overflow vs. free-card bug), `AuctionPhaseEntered` vs `StartAuction` naming inconsistency throughout, `S2CCardAcquired` enum name conflict, `auction_max_duration_seconds` with no enforcement path, contradictory hand-full messages — all corrected. (3) AC coverage gaps: AU1-b split into server-side (testable now) + network integration (gate ADR-008), four new ACs added (AU12 AuctionExpired pending OQ9, AU21 Legendary stratification, AU22 1000ms clamp, AU23 duplicate guard), subtypes added throughout, AU19-a reframed as defensive regression guard. Creative-director verdict: MAJOR REVISION NEEDED (first time a pass 4 review hit MAJOR vs NEEDS REVISION — driven by pillar contradiction, not specification gaps).
Prior verdict resolved: Yes (pass 3 in-session revision + pass 4 re-review condition met)

---

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
