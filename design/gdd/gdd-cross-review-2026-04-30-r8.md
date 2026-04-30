# Cross-GDD Review Report — R8

**Date:** 2026-04-30 (R8 — delta after auction pass 5, NP R5 lean, HUD pass 2, prism/class/card-animations/SAU revisions)
**GDDs Reviewed:** 20 system GDDs + master GDD + systems-index + entities.yaml registry
**Prior review:** `gdd-cross-review-2026-04-30-r7.md` — Verdict FAIL, 9 blockers
**Mode:** Full (parallel consistency + design theory agents)

---

## Progress History

| Review | Date | Verdict | Blockers |
|---|---|---|---|
| R1–R5 | 2026-04-29 | PASS | 0 (9 GDDs) |
| R6 | 2026-04-30 | FAIL | 9 (20 GDDs, 11 new) |
| R7 | 2026-04-30 | FAIL | 9 (4 prior + 5 new) |
| **R8** | **2026-04-30** | **FAIL** | **13 (8 carryover + 5 new consistency + 3 new design)** |

---

## R7 Blocker Disposition

| ID | R7 → R8 | Proof |
|---|---|---|
| C-B2 | 🔴 ESCALATED | NP unilaterally added `ResolutionTimeout` as 4th `GameOverReason` variant (NP:607–612). RSM:103 + registry:1402 still define 3 variants. KS:166 uses `ResolutionTimeout`. Four sources now disagree → see new C-R8-1. |
| C-B4 | 🔴 STILL OPEN | `hand-ui.md:68` Rule 5c still lists `S2CCardAcquired` in activation-lock resolver. NP:88 remains a live counter-example. |
| C-B5 | ✅ RESOLVED | `hand-ui.md:199` Dependencies + Tuning Knobs (lines 328–339) now internally consistent. "Client render config" throughout. |
| C-B6 | 🔴 STILL OPEN | `entities.yaml` `network_messages`: no `S2CSangMepriseReveal` entry. |
| C-NEW-1 | 🔴 STILL OPEN | No `S2CSingleObjectiveReveal` in entities.yaml or NP. class-system.md NP-4 still flags it open. |
| C-NEW-2 | 🔴 STILL OPEN | `auction-system.md:93` still emits `CardSource::AuctionWon`. Only an "Implementation note: verify…" caveat added. |
| C-NEW-3 | 🟡 PARTIAL | NP:569–580 now defines complete `AcquisitionSource` set. `entities.yaml:1531` note still lists only `PrismLane1..5`. |
| C-NEW-4 | 🟡 PARTIAL | `class-system.md:404–406` anchors ordering; RSM/NP/CA still have zero `trigger_index` mentions. |
| C-NEW-7 | 🔴 STILL OPEN | Single registry pass not done. |
| D-B1 + C-NEW-5 | 🔴 STILL OPEN | `class-system.md:214,260` formula text still uses `destroy(...)`. `objective-system.md:246` uses `take_damage(...)`. No per-game Garde-Temps cap added. |
| D-B3 | ✅ RESOLVED | `auction-system.md:28,400` OQ7 closed; Player Fantasy reframed; M2 monitoring gate (gold gap >20g + no agency → escalate). Falsifiable. |
| D-B4 | 🔴 STILL OPEN | `auction-system.md` Rule 8 + Edge Cases: zero matches for "grace"/"disconnect_grace". Surviving player UX during 30s window undefined. |
| D-B5 | 🔴 STILL OPEN | `class-system.md:435` still "degrades gracefully." NP OQ6 MEDIUM/Open. `S2CGameSnapshot` no `active_sang_meprise_reveals` field. `hud.md` no overlay spec. |
| D-W6 | 🔴 STILL OPEN | `round-state-machine.md`: zero matches for `OQ-PLACEMENT-LOAD`. Class additions pushed concurrent systems to 9+ (see D-R8-2). |
| D-W7 | 🟡 PARTIAL | `class-system.md:404` documents combo ordering. `garde_temps_reserve_cost` and `miss_nuit_cap` knobs exist. No `xelorium_steal_cap` added. |
| D-W8 | 🔴 STILL OPEN | `class-system.md:214,260` still `destroy(...)`. Animation contract undefined for Garde-Temps trigger. Resolved with D-B1. |

**Resolved this cycle:** C-B5 (hand-ui internal contradiction), D-B3 (auction wealth gap + Player Fantasy reframing).

---

## Consistency Issues

### Blocking

🔴 **C-R8-1 — `GameOverReason` enum split: 3 variants vs 4 variants across 4 sources**
GDDs: `network-protocol.md:607–612` (4 variants: ObjectivesDestroyed, Disconnection, Draw, **ResolutionTimeout**) vs `round-state-machine.md:103` (3 variants) + `keyword-system.md:166` (uses ResolutionTimeout) + `entities.yaml:1402` (3 variants). NP added 4th variant unilaterally; two implementers will produce different enum sizes.
→ Pick one canonical set (recommend: 4-variant NP/KS; update RSM Rule 14 + RSM-38 + registry). Resolves carryover C-B2.

🔴 **C-R8-2 — `S2CGoldUpdate` payload mismatch across NP / SAU / HUD / registry**
GDDs: `network-protocol.md:107` (5 fields including `reserved_gold`). `shop-auction-ui.md:205,410` ("does NOT carry `reserved_gold`"). `hud.md:86,167,305` (4 fields, no `reserved_gold`). `entities.yaml:1354` (4-field payload). NP introduced the field unilaterally.
→ Reconcile: either strip `reserved_gold` from `S2CGoldUpdate` in NP (clients use `S2CGoldBroadcast` for reserved — already the case) OR add field to registry + SAU + HUD. Four-document contradiction on a wire payload.

🔴 **C-R8-3 — Auction→Hand UI bidirectional dep missing**
GDDs: `auction-system.md` Dependencies (lines 268–281): lists Shop/Auction UI as downstream-hard but omits Hand UI. `hand-ui.md:68` Rule 5c processes `S2CCardAcquired` from auction wins.
→ Add Hand UI as soft downstream to `auction-system.md` Dependencies, or document NP-mediated routing in `hand-ui.md`.

🔴 **C-R8-4 — `AuctionPhaseEntered` rename incomplete: `round-state-machine.md` still uses `StartAuction`**
GDDs: `auction-system.md`: all 10+ occurrences use `AuctionPhaseEntered` (canonical). `round-state-machine.md:69,151,198,276,374`: still uses `StartAuction(round_number)`. F2 Step 4 still says "StartAuction → Auction System". AU1-a and RSM-32 test different names. Compile mismatch at implementation.
→ Update RSM Rule 7, F2 sequence, Dependencies, Interactions, RSM-32 to `AuctionPhaseEntered`.

🔴 **C-R8-5 — Sang Méprise snapshot field name unlocked across 3 GDDs**
GDDs: `class-system.md:591`: `active_sang_meprise_identities`. `network-protocol.md:797` OQ6: `sang_meprise_active`. R7 report: `active_sang_meprise_reveals`. Three different names for the same unresolved field.
→ Lock one canonical name in `network-protocol.md` D.1 schema before Architecture. Recommend `active_sang_meprise_reveals`.

### Warnings

⚠️ **C-R8-6 — RSM Rule 13 disconnect detection contradicts NP heartbeat**
`round-state-machine.md:91`: Lightyear `OnDisconnected`/`OnConnected` only. `network-protocol.md:37,53`: mandates `C2SHeartbeat` ~5s for browser half-open TCP; NP-24/25 assert RSM resets trackers on heartbeat.
→ Update RSM Rule 13 to acknowledge dual-source detection.

⚠️ **C-R8-7 — `legendary_pool_entry_round` not in entities.yaml**
`auction-system.md:295` defines it (default 6); `game-config.md:73` includes it in struct. Zero hits in entities.yaml constants.
→ Register in entities.yaml constants.

⚠️ **C-R8-8 — `S2CGoldBroadcast` registry missing `economy-system.md` as producer**
`entities.yaml:1492–1504` referenced_by: NP, auction-system, shop-auction-ui, hud. Economy System is the producer (fires on every gold mutation) but absent.
→ Add `economy-system.md` to `S2CGoldBroadcast.referenced_by`.

⚠️ **C-R8-9 — Client AUCTION_PREPARING 10s timeout shows false error vs healthy server**
`shop-auction-ui.md:64`: 10s timeout shows "Connection error." `auction-system.md:107`: RSM safety timeout is 120s server-side. Server is healthy; client surfaces a false error.
→ Change message to "Awaiting auction card…" OR trigger `C2SRequestSnapshot` instead.

⚠️ **C-R8-10 — `prism-system.md` Rule 11 + OQ3 anticipate `GoldAwardReason::PrismReward` removal as future work**
`prism-system.md:71,283`: anticipate removal. `network-protocol.md:520–523`: already removed the variant.
→ Update prism-system.md Rule 11 + close OQ3 with past-tense resolution.

⚠️ **C-R8-11 — RSM still labels Auction System and Combat Resolution as "GDD not yet written"**
`round-state-machine.md:151,152,276,277`: stale labels. Both GDDs exist and are Approved.
→ Remove stale labels.

⚠️ **C-R8-12 — `S2CObjectiveIdentities` registry referenced_by lists `design/ux/hud.md` (non-existent path)**
`entities.yaml:1512`: path `design/ux/hud.md` does not exist in scope.
→ Replace with `design/gdd/hud.md`.

⚠️ **C-R8-13 — NP `S2CResolutionEvent` still missing multi-Krosmic ordering contract**
`network-protocol.md` lines 352–562: no `trigger_index` field, no ascending-`(player_id, lane, trigger_index)` emission contract. CA replay sequencer cannot deterministically order same-sub-step effects. Carryover C-NEW-4.
→ Add "Ordering invariants for same-sub-step events" subsection to NP D.2.

---

## Design Issues

### Blocking

🔴 **D-R8-1 — Xelorium output range claim contradicts uncapped reserve**
`class-system.md:175` states output range `[self.reserve, self.reserve + mana_cap]` (max +12 per event). `economy-system.md:69` + `class-system.md:421`: reserve has no cap. Formula is correct per-event but is repeatedly cited as if per-event cap = cumulative ceiling. Feeds D-R8-3.
→ Add a worked example in class-system.md showing typical-vs-worst-case Xelor reserve curve rounds 3–8, validating `garde_temps_reserve_cost = 20` against realistic accumulation rates.

🔴 **D-R8-2 — Cognitive load during PLACEMENT now 9+ concurrent systems (comfortable threshold: 4)**
PLACEMENT (10s) requires: (1) hand fan + Instant plate, (2) drag-stage cells + spawn-range highlights, (3) timer, (4) gold/mana/reserve display, (5) lane/objective HP, (6) staged-card overlay, (7) per-card reserve strip (hand-ui.md Rule 13), (8) class-specific overlays (Xelor reserve readout, Sadida seed highlights, Sinistro attachments), (9) 7 token types from class-system. Auction-followup rounds with 10-card hand push this past any reasonable 10s budget.
→ File `OQ-PLACEMENT-LOAD` in round-state-machine.md. M2 telemetry gate: `% cards staged in final 2s > 30%` OR `% missed placements on auction-followup rounds > 25%` → raise `placement_timer_seconds` to 12s OR collapse class overlays to a togglable side panel.

🔴 **D-R8-3 — Xelor reserve economy: 4 independent stacking sources, no Mummy passive cap**
Independent reserve streams per round: (a) Mummy passive (`class-system.md:115`) "+1 reserve whenever it suffers damage" — **no cap stated**; multiple Mummies = multiplicative; (b) Miss Nuit +2/round (capped); (c) Lane 2+4 prism ~+2 over 5 rounds; (d) Xelorium one-shot up to +opponent.current_mana. Combined: reserve 20 (Garde-Temps cost) reachable by round 6–7 without auction wins. Garde-Temps can fire by rounds 6–8, destroying 2 real objectives in 2 rounds. Candidate dominant early-win strategy.
→ Add `mummy_damage_reserve_cap` tuning knob to class-system.md (default 1/round, safe range 1–3) AND/OR raise `garde_temps_reserve_cost` to 25. Monitor in M2 playtest. Document in Tuning Knobs.

### Warnings

⚠️ **D-R8-4 — M2 escalation gate has falsifiability gap for extreme gold deficit**
`auction-system.md:28`: "trailing player can always afford the starting floor in typical rounds." At round 9 with trailing player at 4g and Legendary floor at 6g, floor is unreachable. Current M2 gate (gold gap >20g + no reported agency) doesn't catch.
→ Add second telemetry trigger: `% of auctions where trailing player placed zero bids > 25%` → escalate.

⚠️ **D-R8-5 — Hand-full lockout during DRAFT_AUCTION forces 20s forced idle**
`auction-system.md:94` says unreachable under RSM enforcement; `auction-system.md:246` + `shop-auction-ui.md:101` still specify the UI. Contradiction signals reachability via Lane 3 prism random draw on the prior round. No Instant card activation relief during DRAFT_AUCTION.
→ Either: (a) allow Instant card activation during DRAFT_AUCTION (requires RSM Rule 15 + NP C2SActivateCard phase-gate update), or (b) add to master GDD anti-pillar exceptions list. Recommend (a).

⚠️ **D-R8-6 — Sadida seed density: no global cap; potential +4 AR per friendly walk-forward by round 7**
`class-system.md:425`: per-movement AR gain explicitly uncapped. Up to 20 seeds across the board (5 lanes × 4 cells) by round 5. A 4 MP unit traversing 4 seeded cells gains +4 AR permanently — unkillable by most 1–3 ATK units.
→ Add `max_total_seeds_per_player` tuning knob to class-system.md (default unlimited; safe range 12–20). Defer hard cap to M2 playtest.

⚠️ **D-R8-7 — Three Player Fantasies coexist without master GDD reconciliation**
Auction ("predatory patience"), Class ("4 rhythm archetypes"), Prism ("colonial-economic income stream"). Compatible but cognitively distinct. No layering statement in `lanes-and-lies-gdd.md` §2.
→ Add 1-paragraph reconciliation to `lanes-and-lies-gdd.md` §2: "auction = singular tense moment; class = silent rhythm; prism = standing income that funds both."

---

## Cross-System Scenario Walkthroughs

### R7 Scenario Re-Walk

| # | Scenario | R7 Status | R8 Status |
|---|---|---|---|
| A | Sang Méprise reconnect | ⚠️ WARNING | 🔴 STILL BROKEN — D-B5 unresolved |
| B | Auction settle race vs hand-full | ⚠️ WARNING | ✅ RESOLVED — NP:108 + SAU OQ8 close the gap |
| C | Xelorium + Gelure same batch | ⚠️ WARNING | 🟡 PARTIAL — class-system documents ordering; C-NEW-4 not echoed in RSM/NP/CA |
| D | Mass-token DEATH chain | 🔴 BLOCKER | 🔴 STILL BROKEN — KS OQ-NP3 open; `UnitDied` no chain-position |
| E | Disconnect during DRAFT_AUCTION | 🔴 BLOCKER | 🔴 STILL BROKEN — D-B4 unresolved |
| F | Punition + Sang Méprise mirror | ℹ️ INFO | ✅ RESOLVED — class-system.md:406 documents mutual-destruction → Draw |

### New Scenarios (R8)

🔴 **Scenario G — Multi-class Krosmic same RESOLUTION batch** (BLOCKER)
**Trigger:** Player A submits Sang Méprise + Punition; Player B submits Garde-Temps + Xelorium in same PLACEMENT batch.
**Failure:** (1) C-NEW-4 still open — NP `S2CResolutionEvent` has no `trigger_index` field; two clients may render ordering non-deterministically. (2) D-B5 unresolved — reconnect during multi-trigger batch loses Sang Méprise reveal. (3) D-W8 unresolved — Garde-Temps `destroy()` vs `take_damage()` animation contract undefined.
→ Maps to: C-NEW-4, D-B5, D-W8. Cannot be well-defined until all three are resolved.

🔴 **Scenario H — Auction win → Hand = 10 → PLACEMENT overload on auction-followup round** (BLOCKER)
**Trigger:** Player A wins Round 3 auction (hand 9→10). Round 4 PLACEMENT begins.
**Failure:** With 10 cards + reserve strip (hand-ui.md Rule 13) + class overlays + 7 token types + 10s timer, auto-stage on timer expiry (hand-ui.md Rule 9) covers 1 card; remaining 9 cards have no recovery path within 10s. Empirical case of D-R8-2.
→ Maps to: D-R8-2, D-W6. Recommend: `placement_timer_seconds = 12` for round immediately following any auction win.

⚠️ **Scenario I — Prism collect + objective destruction + Sang Méprise active same RESOLUTION** (WARNING)
**Failure:** `objective-system.md:97` mandates 500ms reveal hold before `was_fake` shown. Sang Méprise reveals `is_fake` to both players this RESOLUTION — `objective-system.md:103` says suppress surprise animation. `card-animations.md` has no contract for "Sang Méprise active → suppress reveal animation."
→ Maps to: C-W-NEW-8. Add to card-animations.md Edge Cases.

✅ **Scenario J — Auction during last round vs same-tick objective destruction** (RESOLVED AT ARCHITECTURE)
RSM Rule 11: GAME_OVER evaluated at RESOLUTION end, before DRAFT_AUCTION entry. Race condition cannot occur.

⚠️ **Scenario K — Reconnect mid-RESOLUTION DEATH chain animation** (WARNING)
**Failure:** NP OQ7 (unresolved): no RESOLUTION event buffer policy. Reconnecting client gets correct board state from `S2CGameSnapshot` but loses the animated narrative of the DEATH chain. Per `card-animations.md:14`: in-progress tweens discarded, no replay.
→ Maps to: NP OQ7. Non-blocking for Architecture; resolve before RESOLUTION implementation.

---

## GDDs Flagged for Revision

| GDD | Issues | Priority |
|---|---|---|
| `entities.yaml` | C-B6, C-NEW-1, C-NEW-3 (partial), C-NEW-7, C-R8-7, C-R8-8, C-R8-12 | **Blocking** — single pass closes 7 |
| `round-state-machine.md` | C-R8-4 (StartAuction rename), C-R8-1 (GameOverReason 4th variant), C-NEW-4 (Rule 11a), C-W5, C-W11, D-W6 (OQ-PLACEMENT-LOAD), C-R8-11 | **Blocking + Warning** |
| `network-protocol.md` | C-R8-1 (GameOverReason), C-R8-2 (S2CGoldUpdate reconcile), C-NEW-4 (trigger_index), C-R8-5 (Sang Méprise field name), C-R8-13 | **Blocking** |
| `class-system.md` | D-B1+C-NEW-5 (destroy→take_damage + Garde-Temps cap), D-R8-3 (Mummy passive cap), D-R8-1 (Xelorium worked example), D-R8-6 (seed density knob), C-NEW-8 (Hand UI dep) | **Blocking + Warning** |
| `auction-system.md` | C-NEW-2 (CardSource::AuctionWon), D-B4 (disconnect grace UX), C-R8-3 (Hand UI dep) | **Blocking** |
| `hand-ui.md` | C-B4 (S2CCardAcquired resolver), C-W-NEW-3 (DRAFT_INITIAL grid ownership) | **Blocking + Warning** |
| `keyword-system.md` | C-B2 + C-R8-1 (align GameOverReason, close OQ-NP2) | **Blocking** |
| `objective-system.md` | D-B1 (interface contract align with class-system), C-W-NEW-9 | **Blocking + Warning** |
| `game-config.md` | C-W-NEW-1 (8 fields missing: stagger_cadence_ms, type_advantage_atk_bonus/ar_bonus, garde_temps_reserve_cost, miss_nuit_cap, dé_chateux_reveal_threshold, seed_ar_bonus, seed_enemy_damage) | Warning |
| `prism-system.md` | C-R8-10 (PrismReward OQ3 close) | Warning |
| `shop-auction-ui.md` | C-W-NEW-3 (DRAFT_INITIAL ownership), C-R8-9 (preparing-state timeout) | Warning |
| `card-animations.md` | C-W-NEW-8 (reveal-tween cross-ref), C-NEW-4 echo, Scenario I contract | Warning |
| `lanes-and-lies-gdd.md` | D-R8-7 (Player Fantasy reconciliation), C-W10 | Warning |
| `economy-system.md` | C-W-NEW-4 (SAU/HUD bidirectional deps), C-R8-8 | Warning |
| `server-rng.md` | C-W9 (GSS lifecycle), C-W-NEW-7 (keyword/class RNG slots) | Warning |
| `combat-resolution.md` | C-W-NEW-2 (OUTNUMBERED per-unit vs per-lane) | Warning |

---

## Verdict: **FAIL**

**Net change R7→R8:** 2 blockers fully resolved (C-B5, D-B3). 5 new consistency blockers (C-R8-1 through C-R8-5). 3 new design blockers (D-R8-1 through D-R8-3). 2 new scenario blockers (G, H). Effective total: 13 blocking issues.

**Pillar status:**
- **Auction-as-signature: ✅ OK** — D-B3 reframing accepted; M2 monitoring gate concrete.
- **No-idle-spectating: ⚠️ CONCERN** — D-R8-2 (PLACEMENT overload), D-R8-5 (hand-full lockout), Scenario H all structurally challenge the pillar. Fix needed before M2.
- **Deep emergence: ⚠️ CONCERN** — D-R8-3 (Xelor reserve loop) and D-R8-6 (Sadida seed density) are candidate dominant strategies. Gate on Mummy passive cap + M2 playtest monitoring.

**Cognitive load assessment: OVERLOADED** — 9+ active systems during PLACEMENT on auction-followup rounds. R7 "stretched" → R8 "overloaded" on rounds 4, 7, 10. `OQ-PLACEMENT-LOAD` must be filed.

---

## Required Actions Before Architecture

| # | Issue | Primary GDD |
|---|---|---|
| 1 | Registry single pass: register S2CSangMepriseReveal, S2CSingleObjectiveReveal, complete AcquisitionSource variants, add legendary_pool_entry_round, fix S2CObjectiveIdentities path, add economy-system to S2CGoldBroadcast.referenced_by | `entities.yaml` |
| 2 | Lock `GameOverReason` at 3 or 4 variants across RSM/NP/KS/registry | `round-state-machine.md`, `network-protocol.md`, `keyword-system.md` |
| 3 | Reconcile `S2CGoldUpdate` payload (4-field vs 5-field) across NP/SAU/HUD/registry | `network-protocol.md` |
| 4 | Rename `StartAuction` → `AuctionPhaseEntered` in RSM (all 5+ occurrences) | `round-state-machine.md` |
| 5 | Remove `S2CCardAcquired` from hand-ui.md Rule 5c activation-lock resolver | `hand-ui.md` |
| 6 | Replace `CardSource::AuctionWon` with `AcquisitionSource::AuctionWon` in auction-system.md:93 | `auction-system.md` |
| 7 | Echo multi-Krosmic trigger_index ordering in RSM Rule 11a + NP S2CResolutionEvent | `round-state-machine.md`, `network-protocol.md` |
| 8 | Garde-Temps + Punition: use `take_damage()` not `destroy()` + add per-game Garde-Temps use cap (recommended: 1) | `class-system.md`, `objective-system.md` |
| 9 | Define DRAFT_AUCTION disconnect-grace UX in auction-system.md Rule 8 (freeze timer, preserve reservation, "Opponent disconnected" panel state) | `auction-system.md` |
| 10 | Lock `active_sang_meprise_reveals` field name + add to S2CGameSnapshot schema OR define HUD overlay for reveal-lost | `network-protocol.md`, `class-system.md` |
| 11 | Add `mummy_damage_reserve_cap` tuning knob to class-system.md Tuning Knobs | `class-system.md` |
| 12 | File `OQ-PLACEMENT-LOAD` in round-state-machine.md with M2 telemetry trigger | `round-state-machine.md` |

**Highest-leverage coordinated edit:** Items 1–4 above (registry pass + GameOverReason + S2CGoldUpdate + StartAuction rename) can be batched in 3 editing passes across 4 files and close 10+ issues simultaneously.
