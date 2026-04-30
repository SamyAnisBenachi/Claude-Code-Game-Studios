# Systems Index — Lanes and Lies

**Source GDD:** `lanes-and-lies-gdd.md`
**Last updated:** 2026-04-29
**Total systems:** 20
**Design order:** Foundation → Core → Networking → Feature → Presentation

---

## Dependency Layers

```
FOUNDATION            CORE                  NETWORKING
Card Data & Pool ──►  Economy System ──►    Network Protocol ──►  (all Feature systems)
Game Config      ──►  Board/Lane System ──► Game Session System
Server-side RNG  ──►  Objective System
                      Round State Machine

FEATURE (depend on Core + Networking)
Card Acquisition · Auction System · Prism System
Combat Resolution · Keyword System · Class System

PRESENTATION (depend on Feature systems they display)
Board Rendering · Hand UI · Shop/Auction UI · HUD · Card Animations
```

**Bottleneck systems** (most others depend on these):
- `Card Data & Pool` — every system reads card definitions
- `Economy System` — shop, auction, prisms, combat rewards
- `Network Protocol` — auction, placement, resolution sync
- `Round State Machine` — all phase transitions

---

## Progress Tracker

### M1 — Core Loop
*Goal: Two players connect, play one full round, and the round resolves correctly.*

| # | System | File | Status | Notes |
|---|---|---|---|---|
| 1 | Card Data & Pool | `gdd/card-data-pool.md` | Approved | /design-review 2026-04-29 (Pass 3). 7 blockers resolved: draw_shop_slot split into draw_class_card/draw_neutral_family/draw_family_card (Option C, aligns with server-rng.md seed table); SlotType enum defined; OQ7 resolved (auction is shared/common pool, separate from per-player shop pools — no collision); C-B4 resolved (refresh policy documented, Card Acquisition owns display-slot state and fallback logic); fallback contract clarified (pool returns None cleanly, caller routes); CP7 rewritten (behavioral test); 12 new ACs (CP-IC, CP-SHC/SHN, CP-NW, CP3c/d, CP-C2/C3, CP5c, CP7b); CP9 reclassified BLOCKING; edge cases added (late-game class exhaustion, eligible_types=1); Economy→Pool dependency direction corrected. N-B1 (refresh_base_cost) confirmed resolved in game-config.md. |
| 2 | Game Config | `gdd/game-config.md` | Approved | /design-review 2026-04-29 + cross-review fixes: refresh_base_cost added (N-B1 ✓), interest_threshold_gold added, reserve_mana_cap removed (design decision: no cap), 5 RSM timer fields, epic/legendary moved to const, 13 ACs rewritten. |
| 3 | Server-side RNG | `gdd/server-rng.md` | Approved | /design-review 2026-04-29. 8 blockers resolved: Rule 5 rewritten as 3 per-phase chains (DRAFT_INITIAL/RESOLUTION/SHOP), fake-objective initial assignment added to seed table (2 seeds/player), shop slot seed count corrected (2–3/slot), Rule 6 inter-player ordering added (ascending player_id→lane→position), Formula 1b added for CDF weighted selection, Formula 1 preconditions added, AuditEntry type corrected to Option<String> with encoding table, 4 invalid ACs removed and 7 new ACs added (RNG11–15). |
| 4 | Economy System | `gdd/economy-system.md` | Approved | /design-review 2026-04-29 + cross-review fixes: interest formula updated to use GameConfig.interest_threshold_gold (C-B5a ✓), reserve_mana_cap removed (design decision: no cap, organic pressure is limiter), free pick uncapped per design decision (B-3 ✓), refresh_base_cost knob documented. Warning C-W1 remains: Interactions table needs interest_threshold_gold added. |
| 5 | Board / Lane System | `gdd/board-lane-system.md` | Approved | /design-review 2026-04-28. All 14 blocking items resolved: player fantasy rewritten (forensic prediction, not live read), prism respawn confirmed per-player independent (OQ1 closed), F1/F2/F3 formula type safety + direction tables fixed, pending buffer architecture specified (Resource not entities), S2CPlacementReveal + replay log specified, 12 ACs rewritten + 5 new ACs (BL-27b/30/31/32/33). WALL farming and global spawn range kept by design. |
| 6 | Round State Machine | `gdd/round-state-machine.md` | Needs Revision | /design-review 2026-04-29. 13 blockers resolved: disconnect grace 5s→30s, RESOLUTION safety timeout added (60s→Draw), Rule 5/F2 shop refresh inconsistency fixed, StartAuction added to F2 sequence, S2CGameOver+GameOverReason enum defined, ready signal now retractable, RSM-31 BLOCKING, RSM-32–38 added. 4 open questions remain (lobby_timeout, DRAFT_INITIAL gold forfeiture, multiplayer auction, late-joiner sync). |
| 7 | Network Protocol | `gdd/network-protocol.md` | Needs Revision | /design-review R6 full 2026-04-30: NEEDS REVISION (14 blockers addressed inline). Key fixes: Player Fantasy→rule table added; OQ-7 RESOLVED (haste_active: bool, Option A); AR cap = 20; stun_active: bool → stunned_until_round: Option<u32>; timer_remaining_ms → Option<u32>; shop_slots → Vec<Option<CardId>>; sub_step authoritative source declared; Bevy scheduling invariant note; Silence=Pass heartbeat-uncertain indicator; activate_timeout_ms Tuning Knobs; C2SActivateCard dispatcher enforcement + NP-55; NP-7/12/17 rewritten; NP-31h/46–55 added (12 new ACs). Cross-GDD: silenced_until_round Option<u8>→Option<u32> in keyword-system.md (owner: first keyword implementer). Remaining: OQ-1/2/3 HIGH Lightyear verification + OQ-6 limitation accepted. Re-review recommended (R7). |
| 8 | Game Session System | `gdd/game-session-system.md` | Approved | /design-review R3 2026-04-29: APPROVED. Fixed inline: game-config.md GSS row corrected (disconnect_grace_seconds removed; lobby_heartbeat_timeout_seconds noted pending); transitions table: "RSM fires SessionReady" → "GSS fires SessionReady"; ServerRng init failure → LOBBY_CANCELLED row added. 41 ACs. All modes in scope (1v1, 2v2, 3v3). lobby_heartbeat_timeout_seconds added to game-config.md + entities.yaml (OQ8 resolved). ADR needed for SessionReady Observer vs Events<T> (OQ7). |
| 9 | Objective System | `gdd/objective-system.md` | Approved | /design-review R3 2026-04-29: APPROVED. Fixed inline: game-config.md fake_count upper bound tightened ≤4→≤3 + GC8c AC added; OS-17 conditional language cleaned up (ADR-001 resolved). XL scope — 9 cross-system deps, 5 formulas. Sang Méprise reconnect gap forwarded to NP backlog (active_sang_meprise_identities missing from S2CGameSnapshot). |

### M2 — Playable Game
*Goal: Complete 1v1 game with auction, combat, shop, and win condition — visually playable.*

| # | System | File | Status | Notes |
|---|---|---|---|---|
| 10 | Card Acquisition | `gdd/card-acquisition.md` | Approved | /design-review 2026-04-29: 6 blockers resolved. Single-fire auction refresh (DRAFT_AUCTION entry only; same slots persist to DRAFT_SHOP). RSM Rule 12 gold carry-over confirmed. ERR_WRONG_PHASE → silent discard (NP wins). CA18 rollback: refund_gold() specified. shop_weight_per_card design note (raise to 0.20–0.25). 22 ACs (CA1–CA22). refresh_cap added to game-config.md. S2CShopSlots/S2CDraftOffering registered in NP GDD. OQs 1–2 resolved. OQs 3–5 open. |
| 11 | Auction System | `gdd/auction-system.md` | Approved | **Signature mechanic** — /design-review 2026-04-30 (pass 5, lean): APPROVED. All 11 pass-4 blockers confirmed resolved. Pre-implementation gates: (1) reconcile CardSource::AuctionWon vs AcquisitionSource enum name in NP GDD before story impl; (2) update RSM GDD for auction_max_duration_seconds AbortAuction trigger; (3) resolve OQ9 (AuctionExpired reachability) before AU12; (4) verify spend_reserved_gold API name (OQ4). Recommended: migrate AU11 to Economy System GDD; promote CardSource naming to OQ10; close OQ6 cross-GDD update. 33 ACs (BLOCKING + ADVISORY). Scope: L. |
| 12 | Combat Resolution | `gdd/combat-resolution.md` | Approved | /design-review 2026-04-30 (pass 2, full panel): MAJOR REVISION NEEDED → revised in-session. P0 fixes: SHIELD canonical pre-check rule (removed from modifier stack step 10); COUNTERATTACK formula defined (full stack, FINAL BLOW eligible, chains once, multi-attacker simultaneous retaliation); OQ3 ✓ (`range_equidistant_select` added to server-rng.md); bilateral+multi-source overlap rule specified; INJURED/OUTNUMBERED/death boundary ordering fixed; kill_log attribution mechanism added; 10k iteration budget added; OQ1 ✓ (ADR-017), OQ5 ✓ (NP D.2+ADR-017); CR-44/45 → BLOCKING; CR-32 rewritten; STUN+SHIELD edge case added. Pre-impl gates: S2CPlacementReveal/S2CResolutionEvent Lightyear same-frame ordering (ADR-011); UnitId vs EntityId reconciliation (NP D.2/ADR-017); type_advantage fields in game-config.md. |
| 13 | Board Rendering | `gdd/board-rendering.md` | Approved | /design-review R5 2026-04-30: APPROVED. 2 blockers resolved: (1) EC-RESOLUTION-REVEAL-STUCK stale OQ-BR-06 sentence removed (missed in R4); rate-limit note added; (2) BR-STATUS-TIER AC added (BLOCKING) — covers Tier-1 display priority ordering introduced in R4 but lacking test coverage. Recommended open items: OQ-BR-02 should be marked RESOLVED; Lightyear detection API gap (BR-7 NOTE) should become OQ-BR-11; stale dependency labels (Combat Resolution/HUD); BR-RECONNECT-TIME hardware spec. |
| 14 | Hand UI | `gdd/hand-ui.md` | Approved | /design-review R3 lean 2026-04-30: APPROVED. 4 recommended items resolved in-session: HU-10/10b merged (duplicate ACs collapsed — all non-arrival paths unified in one AC); S2CGoldUpdate assumption documented in Rule 5c (instant cards always cost ≥1 mana; gold-neutral future cards require S2CActivationConfirmed); GRID→PASSIVE_LOCKED gap closed (DRAFT_INITIAL→DRAFT_AUCTION is not a valid RSM path — note added to state machine); BoardLayout init constraint added to Dependencies table. OQ5–OQ8 remain open (pre-implementation gates, not design gaps). Implementation-gated: HU-28/HU-28b await OQ8 (S2CActivationRejected in NP GDD); OQ5/OQ6 require ADRs before asset pipeline work. |
| 15 | Shop / Auction UI | `gdd/shop-auction-ui.md` | Approved | /design-review 2026-04-30 (pass 4, lean): APPROVED. No new blocking items. 6 recommended: (1) auction_timer_reset_seconds knob wording ("ceiling" → "value the server sends"); (2) state machine sub-states LocallyExpired/Settling/PrepareTimeout not in diagram — add note or expand; (3) auction-system.md Player Fantasy still references squeeze-bid expressivity (OQ1 cross-GDD gap — pre-impl gate); (4) OQ7 bidirectionality updates pending (economy-system.md + game-config.md); (5) SAU-DA7 test fixture missing explicit S2CGoldBroadcast payload; (6) DRAFT_SHOP affordability source not explicit in Rules 3/4. OQ9 (YOU-ARE-LEADING idle window) remains HIGH RISK — validate in first playtest. |
| 16 | HUD | `gdd/hud.md` | Approved | /design-review Pass 4 2026-04-30: APPROVED. 1 blocker resolved: HUD-01 entity count corrected 16→18 (TextSpan children for gold labels were not counted). Recommended fixes: `GoldDisplayState` struct unified with `is_populated: bool` in Rule 1; Rule 11 `.before()` terminology; OQ-HUD-01 pre-implementation gate note added; LOBBY audio silent row + D.1 `u32`→`f32` type note. |

### M3 — Full Feature
*Goal: All GDD mechanics working — keywords, prisms, class rules, and animations.*

| # | System | File | Status | Notes |
|---|---|---|---|---|
| 17 | Keyword System | `gdd/keyword-system.md` | Needs Revision | /design-review R3 full 2026-05-01: MAJOR REVISION NEEDED → revised inline. 9 design decisions applied: D3 COUNTERATTACK simplified (any non-RANGE attack, tooltip removed); D4 STUN = full shutdown (no COUNTERATTACK); D5 RANGE+WALL specified; D6 FIRST STRIKE+WALL confirmed; D7 KW-041 removed (1-cell-apart collision rule — ATTRACT enemy cap corrected, Formula 2 updated); D8 LEADER snapshot post-SS1 (bonus now in same round); D9 BODYGUARD no-target → None bond. Protocol fixes: silenced_until_round formula corrected (current_round+N-1), HASTE suppression rule, SILENCE client-clear rule. 12 new ACs (KW-058–KW-069, KW-029c/d, KW-030a/b replacing KW-030). 5 new OQs (KS6–KS10). R4 re-review recommended. |
| 18 | Prism System | `gdd/prism-system.md` | Approved | /design-review Pass 3 2026-04-30 (lean): APPROVED (revisions accepted, no re-review). 1 blocker resolved: PS-09 "silently dropped" contradicted Rule 7's `S2CPrismRewardDropped` requirement — AC rewritten with explicit notification staging assertion. 3 recommended fixes: Rule 7 now explicitly states `S2CPrismRewardDropped` NOT sent for Lane 3 hand-full; PS-12 rewritten with `DiscardLog` resource test mechanism; OQ4 tagged as pre-implementation gate matching OQ1. Pre-implementation gates: OQ1 (hand-write API) + OQ4 (server-rng.md hand-full note) + NP OQ1 (Lightyear unicast) must all resolve before Prism epic starts. 2 messages pending NP GDD registration: `S2CPrismRespawned`, `S2CPrismRewardDropped`. 25 ACs (23 BLOCKING, 1 ADVISORY). |
| 19 | Class System | `gdd/class-system.md` | Approved | /design-review pass 3 2026-04-30: NEEDS REVISION → revised in-session (11 GDD-local blockers resolved). PIERCE→ARMOR-PIERCING; CS-3/CS-4 Mummy caveat (tradeoff "binding in Mummy-light games"); CS-4/CS-6 Rule 9→Rule 7; CS-4 `chosen_enemy_objective.hp` variable defined (max HP constant, not current); Sinistro orphan edge case; CS-12 Cra binding tempo rule + melee-push ruling; CS-13 Iop scope declaration; Xelorium+Gelure burst ceiling confirmed by design; CS-AC-31–41 + CS-AC-08b added (Miranda, Madoll passive, Craps alive=0, Sinistro destroy, Seed stacking, Punition Draw, Cra/Iop/HASTE ACs); NP-1 closed; NP-5 +Garde-Temps mutation site; NP-9 SeedPlaced/SeedConsumed registered. Re-review recommended in fresh session. |
| 20 | Card Animations | `gdd/card-animations.md` | Approved | /design-review Pass 5 2026-04-30 (lean): APPROVED. 0 new blockers. 1 fix applied inline: V.3 stale "1.5 s" → "400 ms" (OQ-CA-07 was resolved in Pass 4). 3 cross-doc verifications deferred to implementation: board-rendering.md F4 concurrent formula, game-config.md sub-step minimum 451 ms, shop-auction-ui.md settlement overlay 400 ms. Standing pre-impl gates: OQ-CA-01/05/06/13 + CA-3/CA-21/CA-25 remain correctly gated. PRE-LAUNCH BLOCKING: GAME_OVER 599 ms mitigation deferred per friend-game scope. 25 ACs (20 BLOCKING, 5 ADVISORY). |

---

## Recommended Design Order

Write GDDs in this order (each one depends on those before it being designed first):

```
1  → Card Data & Pool       (everything reads this)
2  → Game Config            (all numbers in one place)
3  → Server-side RNG        (simple, needed early)
4  → Economy System         (mana + gold model)
5  → Round State Machine    (phase flow — context for all other GDDs)
6  → Network Protocol       (what travels over the wire)
7  → Board / Lane System    (spatial model)
8  → Objective System       (win condition)
9  → Card Acquisition       (shop + draft + hand)
10 → Auction System         (signature mechanic — design this carefully)
11 → Combat Resolution      (damage + movement)
12 → Game Session System    (room + lobby)
13 → Board Rendering        (visual board)
14 → Hand UI
15 → Shop / Auction UI      (UX for signature mechanic)
16 → HUD
17 → Keyword System         (extends combat)
18 → Prism System
19 → Class System
20 → Card Animations
```

---

## High-Risk Systems

| System | Risk | Reason |
|---|---|---|
| Auction System | HIGH | Real-time multiplayer + lightyear sync + signature mechanic feel |
| Network Protocol | HIGH | Blocks 8+ other systems; Lightyear 0.26 is post-training-cutoff |
| Combat Resolution | HIGH | Most complex logic; interacts with 5 other systems |
| Keyword System | HIGH | ~20 keywords, many interactions; untested edge cases |

---

## Reference Docs (not systems)

| Doc | File | Purpose |
|---|---|---|
| Master GDD | `lanes-and-lies-gdd.md` | Complete design rules, formulas, ACs |
| Krosmaga Card Reference | `krosmaga-cards-reference.md` | Source card data, Extension=1 |
