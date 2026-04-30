# Cross-GDD Review Report — R9

**Date:** 2026-04-30 (R9 — delta after R8 batch fixes, hand-ui R2, card-animations Pass 3, board-rendering R5, NP R5, SAU pass 4, HUD pass 3)
**GDDs Reviewed:** 20 system GDDs + master GDD + systems-index + entities.yaml registry
**Prior review:** `gdd-cross-review-2026-04-30-r8.md` — Verdict FAIL, 13 blockers
**Mode:** Full (parallel consistency + design theory agents)

---

## Progress History

| Review | Date | Verdict | Blockers |
|---|---|---|---|
| R1–R5 | 2026-04-29 | PASS | 0 (9 GDDs) |
| R6 | 2026-04-30 | FAIL | 9 |
| R7 | 2026-04-30 | FAIL | 9 |
| R8 | 2026-04-30 | FAIL | 13 |
| **R9** | **2026-04-30** | **FAIL** | **11 (3 carryover escalated + 8 new)** |

**Net R8 → R9:** 18 R8 issues fully resolved. New blockers expose deeper game-design holism issues (PLACEMENT overload, Xelor dominant strategy, anti-pillar violation in DRAFT_AUCTION).

---

## R8 Blocker Disposition

| ID | R8 Status | R9 Status | Proof |
|---|---|---|---|
| C-R8-1 (GameOverReason 4-source) | ESCALATED | ✅ RESOLVED w/ 1 cosmetic stale | NP:106,638; RSM:103-110,396; KS:518 closed; registry:1416 (4 variants). **Stale**: `entities.yaml:1329` note still says "three variants" → C-R9-4. |
| C-R8-2 (S2CGoldUpdate payload 4-vs-5) | NEW | ✅ RESOLVED | NP:107 excludes `reserved_gold`; registry:1366 4-field; SAU/HUD aligned. |
| C-R8-3 (Auction→Hand UI dep) | NEW | ✅ RESOLVED | `auction-system.md:289` Hand UI listed as Downstream (soft). |
| C-R8-4 (StartAuction rename) | NEW | ✅ RESOLVED w/ 2 cosmetic carryover | RSM:69,155,202,280,378 all `AuctionPhaseEntered`. Stale: `auction-system.md:144,252` still cite `StartAuction` → C-R9-W1. |
| C-R8-5 (Sang Méprise field name) | NEW | ✅ RESOLVED | NP:211 `active_sang_meprise_reveals` canonical; class-system:472 references; OQ closed. |
| C-R8-6 (RSM disconnect vs heartbeat) | NEW | 🔴 STILL OPEN | RSM:94 still says "Lightyear OnDisconnected only"; RSM-23/25/35 ACs use "heartbeat gap"; NP:37 mandates `C2SHeartbeat`. → C-R9-W2. |
| C-R8-7 (legendary_pool_entry_round registry) | NEW | ✅ RESOLVED | `entities.yaml:988-996`. |
| C-R8-8 (S2CGoldBroadcast economy ref) | NEW | ✅ RESOLVED | `entities.yaml:1511`. |
| C-R8-9 (AUCTION_PREPARING false error) | NEW | 🔴 STILL OPEN | SAU:64,415,732 retain "Connection error" copy. → C-R9-W3. |
| C-R8-10 (Prism PrismReward OQ3) | NEW | ✅ RESOLVED | Past-tense; OQ3 closed. |
| C-R8-11 (RSM stale "GDD not yet written") | NEW | ✅ RESOLVED | Zero matches. |
| C-R8-12 (S2CObjectiveIdentities path) | NEW | ✅ RESOLVED | `entities.yaml:1527` → `design/gdd/hud.md` (file exists). |
| C-R8-13 (NP S2CResolutionEvent ordering) | NEW | 🟡 PARTIAL | NP:376-393 added; **CA Rule C-8 contradicts** → C-R9-2. |
| C-B4 (S2CCardAcquired in hand-ui Rule 5c) | STILL OPEN | ✅ RESOLVED | hand-ui.md:68 explicitly excludes it. |
| C-B6 (S2CSangMepriseReveal registry) | STILL OPEN | ✅ RESOLVED | `entities.yaml:1551-1564`. |
| C-NEW-1 (S2CSingleObjectiveReveal) | STILL OPEN | 🟡 PARTIAL | Registry:1565-1574 OK; NP has zero schema → C-R9-1. |
| C-NEW-2 (CardSource→AcquisitionSource) | STILL OPEN | ✅ RESOLVED | auction-system.md:93. |
| C-NEW-3 (AcquisitionSource variants) | PARTIAL | ✅ RESOLVED | Registry:1547 + NP:600 aligned. |
| C-NEW-4 (multi-Krosmic ordering) | PARTIAL | 🟡 PARTIAL | RSM Rule 11a + NP D.2 ✅; **CA still missing** → C-R9-2. |
| C-NEW-7 (registry single pass) | STILL OPEN | ✅ RESOLVED | Done. |
| D-B1 + C-NEW-5 (Garde-Temps interface) | STILL OPEN | ✅ RESOLVED | class-system.md:219,237 `take_damage`; objective:246,333 aligned; per-game cap=1 added. |
| D-B4 (DRAFT_AUCTION disconnect UX) | STILL OPEN | ✅ RESOLVED | auction-system.md:104-117 full grace-pause spec. |
| D-B5 (Sang Méprise reconnect) | STILL OPEN | ✅ RESOLVED | NP:211 snapshot field; class-system:472 cross-ref. |
| D-W6 (OQ-PLACEMENT-LOAD) | STILL OPEN | 🔴 STILL OPEN | RSM Open Questions: zero match → C-R9-6. |
| D-W7 (xelorium_steal_cap) | PARTIAL | 🔴 STILL OPEN | No knob added → C-R9-W13. |
| D-W8 (Garde-Temps animation contract) | STILL OPEN | 🔴 STILL OPEN | CA zero matches for Garde-Temps → C-R9-W11. |
| D-R8-1 (Xelorium worked example) | NEW | 🔴 STILL OPEN | No round-3-to-8 reserve curve → C-R9-W6. |
| D-R8-2 (PLACEMENT cognitive load) | NEW | 🔴 STILL OPEN | → D-R9-1. |
| D-R8-3 (Mummy passive cap) | NEW | 🔴 STILL OPEN | class-system:74,116,498 — no `mummy_damage_reserve_cap` → C-R9-7 / D-R9-2. |
| D-R8-4 (M2 escalation 2nd trigger) | NEW | 🟡 PARTIAL | Only gold-gap trigger; no zero-bid trigger. |
| D-R8-5 (Hand-full lockout 20s idle) | NEW | 🔴 STILL OPEN | No Instant activation in DRAFT_AUCTION → D-R9-3. |
| D-R8-6 (Sadida seed density cap) | NEW | 🔴 STILL OPEN | No `max_total_seeds_per_player` → C-R9-W7 / D-R9-5. |
| D-R8-7 (Player Fantasy reconciliation) | NEW | 🔴 STILL OPEN | Master GDD §2 unchanged → C-R9-W9 / D-R9-4. |
| C-W-NEW-1 (game-config missing fields) | CARRYOVER | 🟡 PARTIAL | Only `stagger_cadence_ms` added; 7 more fields missing → C-R9-W4. |
| C-W-NEW-3 (DRAFT_INITIAL grid ownership) | CARRYOVER | 🔴 STILL OPEN | Both hand-ui + SAU claim authority → C-R9-5. |
| C-NEW-8 (Class System ↔ Hand UI dep) | CARRYOVER | 🔴 STILL OPEN | Asymmetric dependency → C-R9-W5. |

**Resolved this cycle:** C-B4, C-B6, C-NEW-2, C-NEW-3, C-NEW-7, C-R8-2, C-R8-3, C-R8-4 (mostly), C-R8-5, C-R8-7, C-R8-8, C-R8-10, C-R8-11, C-R8-12, D-B1+C-NEW-5, D-B4, D-B5 — 18 items fully resolved.

---

## Consistency Issues

### Blocking

🔴 **C-R9-1 — `S2CSingleObjectiveReveal` registered in entities.yaml but undefined in network-protocol.md**
`entities.yaml:1565-1574` (full payload `{ player_id: PlayerId, lane: LaneId, is_fake: bool }`, source declared as `design/gdd/network-protocol.md`). `network-protocol.md:102-130` message table: no entry. D.1: no struct definition. `class-system.md:657` NP-4 still flagged Open. Registry declares NP as source-of-truth, yet NP has no schema. Implementers cannot build the wire format.
→ Add `S2CSingleObjectiveReveal` row to NP message table (Reliable, Unicast to opponent), add struct definition to D.1, close class-system NP-4.

🔴 **C-R9-2 — Card Animations Rule C-8 contradicts NP D.2 trigger_index ordering contract**
`network-protocol.md:376-393` D.2: "PRIMARY ORDERING KEY for events within the same sub_step… CLIENT CONTRACT: render events strictly in array order; do NOT re-sort." `card-animations.md:176` Rule C-8: "Within a group: all `ResolutionEvent`s in `AnimGroup.events` spawn their tweens in a single frame. **No ordering within a group**." CA has zero `trigger_index` mentions. AnimGroup partitions by `sub_step` only (CA:178), so all same-sub-step events fire as one unordered batch — directly negating the NP D.2 contract. Multi-Krosmic batches (Xelorium then Gelure same sub-step) cannot animate deterministically. Carryover C-NEW-4/C-R8-13.
→ Update CA Rule C-8: spawn order = ascending `trigger_index` within group. Add CA AC asserting deterministic spawn-order for same-sub-step events. Add explicit cite of NP D.2 contract.

🔴 **C-R9-3 — `S2CActivationRejected` referenced by Hand UI but unregistered in NP and entities.yaml**
`hand-ui.md:68` Rule 5c declares it as a resolver. `hand-ui.md:200,580,606` Interactions + HU-28b + OQ8 all cite it; OQ8 marks NP registration as a BLOCKING gate. NP: zero matches. entities.yaml: zero matches. HU-28b (BLOCKING) is unimplementable until NP and registry register the message.
→ Add `S2CActivationRejected` to NP message table + D.2 with payload `{ card_id, reason: ActivationRejectedReason }` on Reliable, Unicast; register in entities.yaml; close hand-ui OQ8.

🔴 **C-R9-4 — `entities.yaml` `S2CGameOver` note stale ("three variants") AND YAML duplicate `notes:` keys**
`entities.yaml:1326-1330`: payload says 4 variants; line 1329 note says "three variants: ObjectivesDestroyed, Disconnection, Draw." Lines 1327 and 1329 are two consecutive `notes:` keys in the same map entry (YAML structural error — permissive parsers silently keep the last value, which is the stale 3-variant string overwriting the correct 4-variant note at 1327). Compounds C-R8-1 carryover.
→ Merge the two `notes:` blocks into a single string and update to 4 variants including ResolutionTimeout.

🔴 **C-R9-5 — DRAFT_INITIAL 3×3 grid: dual-ownership between Hand UI and Shop/Auction UI**
`hand-ui.md:55-61` Rule 4 defines grid overlay (click-to-buy, hand-full lockout, ready signal). `shop-auction-ui.md:30-51` DRAFT_INITIAL Panel Rules 1–7 define identical scope. `hand-ui.md:498` HU-01 pre-pools 9 grid slots in Hand UI scene. Two systems simultaneously claim authority over the same 9 slot entities. Two spawners = conflict at implementation. Carryover from R7/R8.
→ Pick one owner. Recommend: SAU owns slot rendering; Hand UI owns fan animation only. Defer side with an explicit "display owned by [other GDD]" sentence.

🔴 **C-R9-6 — `OQ-PLACEMENT-LOAD` not filed in RSM despite R8 Required Action #12**
`round-state-machine.md:386-396` lists 5 Open Questions (lobby_timeout, resolved, multi-card auction, late-joiner, resolved). Zero matches for `PLACEMENT-LOAD` or cognitive-load in any GDD. M2 telemetry gate undocumented. Scenario H (auction-followup PLACEMENT overload — 10 active systems, 10s budget) structurally unaddressed.
→ File `OQ-PLACEMENT-LOAD` in RSM Open Questions with M2 telemetry trigger: `% missed placements on auction-followup rounds > 25%` → raise `placement_timer_seconds` to 12s OR collapse class-state overlays to togglable side panel.

🔴 **C-R9-7 — `mummy_damage_reserve_cap` absent across class-system, game-config, and registry**
`class-system.md:74` Mummy: "passive: gains +1 reserve whenever it suffers damage." `class-system.md:116`: "(no cap — each hit adds 1 reserve)." `class-system.md:498` Tuning Knobs lists `garde_temps_per_game_cap`, `miss_nuit_cap`, but no `mummy_damage_reserve_cap`. Tuning Knobs at line 498 even flags "dominant strategy risk if Mummy passive cap is also absent" and then never adds the knob. `game-config.md` and entities.yaml: no entry. Carryover D-R8-3 (see also D-R9-2 for holistic consequences).
→ Add `mummy_damage_reserve_cap` (default 1/round per Mummy, safe range 1–3) to class-system Tuning Knobs + game-config struct + registry constants.

🔴 **C-R9-8 — `keyword-system.md:166` stale OQ reference after OQ-NP2 closed**
`keyword-system.md:166` Rule 1 prose: "the server broadcasts `S2CGameOver { loser: None, reason: ResolutionTimeout }` (new variant — see OQ2 below)". `keyword-system.md:518` OQ-NP2 RESOLVED 2026-04-30. Rule 1 still calls the variant "new" and points to a closed OQ — dangling reference for any future reader.
→ Strip "(new variant — see OQ2 below)" from line 166.

### Warnings

⚠️ **C-R9-W1 — auction-system.md:144,252 still cite `StartAuction` (cosmetic carryover from C-R8-4)**
All other 10+ occurrences use `AuctionPhaseEntered`. Two stale prose lines remain.
→ Replace both with `AuctionPhaseEntered`.

⚠️ **C-R9-W2 — RSM Rule 13 disconnect detection describes Lightyear-only path while RSM ACs use heartbeat language**
`round-state-machine.md:94`: "Lightyear's `OnDisconnected`/`OnConnected` (not a custom heartbeat message)." RSM-23/25/35/369/371/381: "heartbeat gap" / "seconds since last heartbeat." `network-protocol.md:37,53`: `C2SHeartbeat` ~5s mandated; NP-24/25 assert RSM resets trackers on heartbeat. Carryover C-R8-6.
→ Update RSM Rule 13 to "Lightyear `OnDisconnected`/`OnConnected` events AND `C2SHeartbeat` receipt reset the timer."

⚠️ **C-R9-W3 — AUCTION_PREPARING 10s timeout shows "Connection error" while server is healthy**
`shop-auction-ui.md:64,415,732` retain "Connection error — awaiting server…". Server-side `auction_max_duration_seconds = 120s` (`auction-system.md:117`) — server is healthy; client surface is false alarm. Carryover C-R8-9.
→ Change to "Awaiting auction card…" OR trigger `C2SRequestSnapshot`.

⚠️ **C-R9-W4 — `game-config.md` missing 7 of 8 class-system fields registered/referenced elsewhere**
Only `stagger_cadence_ms` (line 110) was added. Missing from `GameConfig` struct: `type_advantage_atk_bonus`, `type_advantage_ar_bonus` (registry:1147,1157 — must be added before Combat Resolution story), `garde_temps_reserve_cost` (registry:1204), `miss_nuit_cap` (registry:1216), `dé_chateux_reveal_threshold` (class-system:500), `seed_ar_bonus` (class-system:501), `seed_enemy_damage` (class-system:502).
→ Add all 7 fields to game-config.md struct + Tuning Knobs section.

⚠️ **C-R9-W5 — Class System Dependencies missing Hand UI (asymmetric dep)**
`class-system.md:478-489` Dependencies: no Hand UI row. `hand-ui.md:316-324` Interactions: no Class System row. Yet `class-system.md:530` instructs "Coordinate with Hand UI GDD" for reserve-insufficient visuals; `hand-ui.md:161,606` reciprocally cite Class System. design-docs.md rule: "if A depends on B, B's doc must mention A."
→ Add Hand UI as Downstream (soft) in class-system Dependencies; add Class System row to hand-ui.md Interactions.

⚠️ **C-R9-W6 — D-R8-1 Xelorium worked example not added (commitment unmet)**
`class-system.md:497` Tuning Knobs references "calibrated tradeoff — see CS-3 strategic tradeoff note" but no rounds-3-through-8 reserve accumulation curve exists. R8 Required Action #1 for class-system unmet.
→ Add worked example: typical-vs-worst-case Xelor reserve accumulation rounds 3–8, validating `garde_temps_reserve_cost = 20` against realistic sources.

⚠️ **C-R9-W7 — Sadida seed density: no global cap (D-R8-6 carryover)**
`class-system.md:308` max 1 seed per cell; 5 lanes × 4 cells = up to 20 seeds per player by round 5. No `max_total_seeds_per_player` knob anywhere. Gate on M2 playtest.
→ Add `max_total_seeds_per_player` (default unlimited; safe range 12–20) to class-system Tuning Knobs.

⚠️ **C-R9-W8 — Sang Méprise + objective destruction reveal-suppression contract undefined in card-animations.md**
`objective-system.md:97-103`: 500ms reveal hold + suppression of surprise animation when Sang Méprise active. `board-rendering.md:75`: ObjectiveIdentityCache suppression branch noted. `card-animations.md`: zero matches for "Sang Méprise" or "surprise." Scenario I from R8 still unresolved.
→ Add Edge Case to card-animations.md: "If Sang Méprise active during ObjectiveDestroyed → suppress surprise animation; standard reveal tween still plays."

⚠️ **C-R9-W9 — Master GDD §2 Player Fantasy multi-system reconciliation missing (D-R8-7 carryover)**
`lanes-and-lies-gdd.md:17-26` §2: four-feeling list unchanged. No layering statement reconciling auction ("predatory patience"), class ("4 rhythm archetypes"), and prism ("standing income stream") sub-fantasies.
→ Add 1-paragraph reconciliation: "auction = singular tense moment; class = silent rhythm filling all rounds; prism = standing income that funds both."

⚠️ **C-R9-W10 — `garde_temps_reserve_cost` registered (registry:571,1204) but absent from `GameConfig` struct**
`class-system.md:497` Tuning Knobs asserts GameConfig ownership. `game-config.md` struct: no field. Loader will not populate it.
→ Add to game-config.md struct (part of C-R9-W4 batch).

⚠️ **C-R9-W11 — Garde-Temps card animation contract undefined (D-W8 carryover)**
`class-system.md:117,219`: Garde-Temps fires full-HP lethal damage via `take_damage()` in one event. `card-animations.md`: zero matches for "Garde-Temps." Spell-driven lethal animation budget undefined; falls into general 500ms staged_objective_reveal window without an explicit animation spec.
→ Add Garde-Temps row to card-animations.md keyword/spell animation registry (durations + assets).

⚠️ **C-R9-W12 — Hand-full DRAFT_AUCTION 20s forced idle (D-R8-5 carryover — see also D-R9-3)**
`auction-system.md:256` bid button disabled; no Instant card activation path during DRAFT_AUCTION. Reachable via Lane 3 prism random-draw on prior round. Player has zero agency for full 20s. Pillar 2 ("No idle spectating") violation.
→ Resolve via D-R9-3 fix (add DRAFT_AUCTION to `C2SActivateCard` valid phases).

⚠️ **C-R9-W13 — `xelorium_steal_cap` not added (D-W7 carryover)**
`class-system.md:441` Xelorium + Gelure same-batch combo documented as legal. Reserve uncapped (`economy-system.md:69`). No cap knob on steal amount.
→ Add `xelorium_steal_cap` (default unbounded, max 5) to class-system Tuning Knobs OR document explicit "accept and monitor" decision in Edge Cases.

⚠️ **C-R9-W14 — `entities.yaml` `S2CGameOver` entry has duplicate YAML `notes:` keys**
`entities.yaml:1326-1330`: lines 1327 and 1329 are both `notes:`. YAML duplicate-key — permissive parsers keep the last value (the stale 3-variant string), silently discarding the correct 4-variant note. Compounds C-R9-4.
→ Merge both `notes:` blocks into one. Fix alongside C-R9-4.

---

## Design Issues

### Blocking

🔴 **D-R9-1 — Auction-followup PLACEMENT cognitive overload: 10 concurrent active systems vs 4-system comfort threshold (10s budget)**

GDDs: `round-state-machine.md:74` (`placement_timer_seconds = 10`); `hand-ui.md:140-161` (per-card reserve strips at up to 10 cards, overlapping at high count per line 144-145); `class-system.md:524-550` (Xelor reserve readout, Sinistro indicator, Sang Méprise overlay, Xelorium drain feedback — overlapping per match-up); `auction-system.md:208` (master GDD A9 — hand 9→10 on auction win).

Active systems during PLACEMENT on auction-followup rounds: (1) hand fan + Instant plate (10 cards), (2) per-card reserve strips, (3) drag-stage cells + spawn highlights, (4) timer, (5) gold/mana/reserve HUD, (6) lane/objective HP, (7) staged-card overlay, (8) class-specific overlays, (9) 7 token types on board, (10) opponent gold ticker (`S2CGoldBroadcast`). **10 total** (comfortable threshold: 4). The 200ms timer-expiry grace window (hand-ui.md:102-107) covers 1 in-flight drag, not 9 remaining cards.

**Holistic break:** Pillar 3 ("No idle spectating") inverts at the same moment Pillar 4 ("Auction as signature") fires — the auction win produces forced overload, not active engagement.

→ (a) File `OQ-PLACEMENT-LOAD` in RSM (closes C-R9-6). (b) Add `auction_followup_placement_timer_seconds = 12` knob to game-config.md. (c) Collapse class-state overlays to a togglable side panel during PLACEMENT.

🔴 **D-R9-2 — Xelor reserve loop has no monotone source cap; Garde-Temps lethal by R6 without auction wins (dominant strategy)**

GDDs: `class-system.md:74,116` (Mummy uncapped — multiple Mummies multiply); `class-system.md:121` (Miss Nuit capped +2/round); `class-system.md:165-181` (Xelorium +up to opponent.current_mana, max +12 at mana_cap=12); `class-system.md:498` (Tuning Knobs has `garde_temps_per_game_cap=1`, no Mummy knob).

Typical R6 Xelor reserve stacking: Miss Nuit +2 + Lane 2/4 prism +1-2 + Mummy passive (uncapped) +2-4 + Xelorium one-shot +6-12 + Gelure +6. Garde-Temps cost 20 reachable by R6 without any auction wins. With a Sacrier opponent (high INJURED-interaction rate), Mummy passive farming is fastest — **Sacrier's identity (controlled self-damage) accelerates the opponent's win condition**, inverting the intended matchup. Two-of-three real objectives can be erased before any Legendary enters the auction pool (round 6 threshold).

**Holistic break:** Violates Pillar 2 ("10+ viable strategies"). Xelor becomes optimal first-pick against Sacrier; reserve-loop is optimal path independent of auction outcomes; class diversity collapses. Carryover D-R8-3.

→ (a) Add `mummy_damage_reserve_cap` knob (default 1/round per Mummy, safe 1–3) to class-system Tuning Knobs + game-config + registry (closes C-R9-7). (b) Document worked reserve curve R3-R8 (closes D-R8-1). (c) Option: raise `garde_temps_reserve_cost` to 24-25 OR cap Mummy passive to once-per-round-per-Mummy (mirroring Miss Nuit pattern). Monitor in M2 playtest.

🔴 **D-R9-3 — DRAFT_AUCTION hand-full lockout violates "No idle spectating" anti-pillar**

GDDs: `lanes-and-lies-gdd.md:48` Anti-pillar: "No phase exists where a player has nothing to do or observe." `auction-system.md:256`: bid button disabled when `BidRejectedReason::HandFull`. `network-protocol.md:48`: `C2SActivateCard` valid in DRAFT_INITIAL and DRAFT_SHOP only — NOT DRAFT_AUCTION. Player at hand=10 entering DRAFT_AUCTION (reachable via Lane 3 prism random-draw on prior round, or via auction win at hand=9) is structurally locked out for 20s with zero actions available.

**Holistic break:** The R8 fix surfaced a warning text — the warning IS the violation, not the resolution. This is an explicit breach of the master GDD anti-pillar as written.

→ **Recommended:** Add DRAFT_AUCTION to `C2SActivateCard` valid phases in NP:48; update RSM Rule 15 valid-actions table; update hand-ui.md `PASSIVE_LOCKED` state to allow Instant clicks. Restores agency: locked-out player can play reserve spells or Instants while watching the auction. Alternative: explicit anti-pillar exception in master GDD (not recommended — pillar takes priority).

### Warnings

⚠️ **D-R9-4 — Three player fantasies coexist without master-GDD layering paragraph (D-R8-7 carryover)**
`lanes-and-lies-gdd.md:17-27`: four-feeling list. Auction ("predatory patience"), class ("4 rhythm archetypes"), prism ("standing income stream") assert independent fantasies. Without a master-GDD paragraph explicitly layering them, designers risk drift (e.g., class designer adding Krosmic-at-auction effect that double-loads the auction moment).
→ Add 1-paragraph reconciliation to `lanes-and-lies-gdd.md` §2.

⚠️ **D-R9-5 — Sadida seed AR uncapped + PIERCE coverage unverified → potential binary counter matchup**
`class-system.md:308,425,462`: max 1 seed per cell; +1 AR per walk-over (uncapped per-unit); PIERCE is "the design ceiling — pre-implementation gate: verify PIERCE availability." Up to 20 seeds across board by round 5. 4-MP unit traversing 4 seeded cells gains +4 AR permanently. PIERCE is the only counter; if Iop/Bow Meow archetypes lack PIERCE in M1 pool, Sadida becomes unkillable for rush classes. +4 AR vs 1-ATK tokens = 0 damage.
→ Move PIERCE pre-impl gate from advisory to BLOCKING on Class System stories. Audit card-data-pool.md PIERCE distribution. Add `max_total_seeds_per_player` knob (C-R9-W7).

⚠️ **D-R9-6 — R3 first-ever auction signal-value at low-gold totals may falsify Auction-as-signature pillar at first instance**
`auction-system.md:30,355`: preset +1/+3/+5 buttons. At round 3, both players have ~5-10g. All R3 bid sequences look similar — three button inputs cannot distinguish aggressive from conservative bidder at low gold totals. Signal-to-cost ratio may be noise at the game's first signature moment.
→ Add R3 worked example to auction-system.md Player Fantasy. Add M2 telemetry gate: "% R3 auctions where both players reach +5 button before settling > 50% → escalate bid-design sprint."

⚠️ **D-R9-7 — Trailing player bid-vocabulary collapses to {silence, +1} at low free-gold tiers**
`auction-system.md:30` (+1/+3/+5 buttons). Trailing player with free_gold < 3 cannot use +3 or +5. Two ranges ([0,2], [3,∞]) reduce signal to {silence, +1}. Long-term: trailing player's silence is overdetermined; auction reads degrade. Compounds D-R9-6.
→ M2 monitoring: bid-button distribution as function of `free_gold` tier. If trailing players use only +1 for >30% of rounds, escalate.

---

## Cross-System Scenario Walkthroughs

**Scenarios walked: 7**

### Scenario 1 — Auction win → Hand=10 → R+1 PLACEMENT

🔴 **BLOCKER** (maps to D-R9-1)

**Trigger:** Player A wins R3 auction at hand=9 → hand becomes 10. R4 PLACEMENT begins.

1. Auction settles: `spend_reserved_gold(leader)`, `S2CCardAcquired { source: AcquisitionSource::AuctionWon }` (`auction-system.md:91-96`).
2. RSM: DRAFT_AUCTION → DRAFT_SHOP (`round-state-machine.md:140`). Shop purchases rejected — hand=10 (economy-system E9).
3. RSM: DRAFT_SHOP → PLACEMENT. `placement_timer = 10s` (RSM:75). No timer extension for auction-followup.
4. Hand UI enters STAGING with 10 cards + 10 reserve-split strips (`hand-ui.md:140-161,175`).
5. Timer expiry: 200ms grace covers 1 in-flight drag (hand-ui.md:102-107). Remaining 9 cards stay unplaced.

**Failure:** Player cannot physically stage all 10 cards in 10s under current PLACEMENT load. `OQ-PLACEMENT-LOAD` not filed. **No telemetry gate exists.**

### Scenario 2 — Multi-class Krosmic same RESOLUTION (SM + Punition + Garde-Temps + Xelorium)

⚠️ **WARNING** (CA trigger_index still missing — C-R9-2)

Trigger_index assignment RSM Rule 11a works (RSM:87-88). NP D.2 ordering contract in place (NP:376-393). D-B1 fix held (Garde-Temps uses `take_damage` — class-system:219,237). Loss evaluation deferred to RESOLUTION end (RSM Rule 11). **But:** CA Rule C-8 "No ordering within a group" contradicts NP D.2 — multi-Krosmic batch animated non-deterministically client-side. See C-R9-2.

Additional edge case not documented: Punition (self-sacrifice real-2) + opponent Garde-Temps (destroys real-1) in same sub-step → Sacrier may lose even though Punition fired, with lethal sub-step sourced from both effects. Loss attribution in `S2CGameOver.loser` and `kill_log` not specified for this combination.
→ Add edge case to class-system.md: "Punition + opponent Garde-Temps same sub-step — Sacrier loss evaluated at RESOLUTION end counting both events; attribution per kill_log."

### Scenario 3 — Disconnect during DRAFT_AUCTION

⚠️ **WARNING** (acceptable at friend-game scope)

R8 D-B4 fix held: auction-system.md:104-117 Rule 8 specifies grace-pause, timer frozen, bid buttons disabled, `S2COpponentDisconnected` to surviving player, reservation preserved, `AbortAuction` on grace expiry. **Surviving player still has 30s grace + remaining timer of zero-agency** (no Instant activation). Accepted at friend-game scope but must be documented as known limitation in master GDD or auction-system Edge Cases.

### Scenario 4 — Reconnect during multi-trigger batch (Sang Méprise reveal)

⚠️ **WARNING** (NP OQ7 unresolved)

R8 D-B5 fix held: `S2CGameSnapshot.active_sang_meprise_reveals` (NP:211) allows state rebuild on reconnect. In-flight animation narrative (DEATH chains, Punition damage) still lost per CA:14. Acknowledged — non-blocking for Architecture.

### Scenario 5 — Prism collect + objective destruction same RESOLUTION

⚠️ **WARNING** (new — no CA contract)

If A's hand=10 when Lane 3 prism fires, spell card is dropped. If loss condition fires same RESOLUTION end, `S2CCardAcquired` may still reach client and animate to hand simultaneously with `S2CGameOver`. No CA Edge Case specifying "RESOLUTION events post-loss-condition: drain animation queue; delay game-over screen by `card_animation_max_ms`."
→ Add to card-animations.md Edge Cases.

### Scenario 6 — R3 first-ever auction, all-in vs silent

⚠️ **WARNING** (maps to D-R9-6)

Player A bids 4g (floor 3g Rare, current_price=3+1=4). Player B silent. Timer reaches 0; A pays 4g. A learned nothing. B learned A bids on Rares at minimum. Signal value at R3 low-gold totals underdetermined. Maps to D-R9-6.

### Scenario 7 — Late-game R9 Xelor vs Sacrier: Garde-Temps + Punition collision

⚠️ **WARNING** (maps to D-R9-2)

Xelor player A: reserve=22, stages Garde-Temps (R8 per-game cap=1, not yet used). Sacrier player B: 2 alive reals, stages Punition. Sub-step 1: Garde-Temps (trigger_index 0) destroys Sacrier real-1 → real_destroyed=1. Punition (trigger_index 1): B sacrifices real-2 → real_destroyed=2 → B loses. B chose Punition intending to deal damage; did not anticipate Garde-Temps stacking with Punition's own self-sacrifice. No documented warning in class-system for "Punition into Xelor with reserve ≥ 20 is self-lethal." Maps to Sacrier matchup identity inversion (D-R9-2).

---

## GDDs Flagged for Revision

| GDD | Issues | Priority |
|---|---|---|
| `network-protocol.md` | C-R9-1 (S2CSingleObjectiveReveal schema), C-R9-3 (S2CActivationRejected), D-R9-3 (DRAFT_AUCTION + C2SActivateCard valid phase) | **Blocking** |
| `card-animations.md` | C-R9-2 (Rule C-8 vs trigger_index), C-R9-W8 (Sang Méprise suppression), C-R9-W11 (Garde-Temps animation row) | **Blocking + Warning** |
| `entities.yaml` | C-R9-4 + C-R9-W14 (S2CGameOver duplicate notes + 3-vs-4 stale), C-R9-3 (S2CActivationRejected register) | **Blocking** |
| `class-system.md` | C-R9-7 (Mummy cap), D-R9-2 (Xelor reserve loop), C-R9-W5 (Hand UI dep), C-R9-W6 (Xelorium worked example), C-R9-W7 (Sadida seed density), C-R9-W13 (xelorium_steal_cap) | **Blocking + Warning** |
| `round-state-machine.md` | C-R9-6 (OQ-PLACEMENT-LOAD), C-R9-W2 (heartbeat reconcile), D-R9-3 (Rule 15 add Instant in DRAFT_AUCTION), D-R9-1 (auction-followup timer knob) | **Blocking + Warning** |
| `hand-ui.md` | C-R9-5 (DRAFT_INITIAL grid ownership), C-R9-W5 (Class System dep), D-R9-3 (PASSIVE_LOCKED state update) | **Blocking + Warning** |
| `shop-auction-ui.md` | C-R9-5 (DRAFT_INITIAL grid ownership), C-R9-W3 (AUCTION_PREPARING text) | **Blocking + Warning** |
| `keyword-system.md` | C-R9-8 (line 166 stale OQ ref) | Blocking |
| `auction-system.md` | C-R9-W1 (StartAuction prose carryover), C-R9-W12 (hand-full DRAFT_AUCTION idle — resolved by D-R9-3) | Warning |
| `game-config.md` | C-R9-W4 (7 missing fields), C-R9-W10 (garde_temps_reserve_cost in struct) | Warning |
| `lanes-and-lies-gdd.md` | C-R9-W9 / D-R9-4 (Player Fantasy layering paragraph), D-R9-3 (anti-pillar exception if fix declined) | Warning |
| `objective-system.md` | C-R9-W8 (Sang Méprise suppression cross-ref) | Warning |
| `card-data-pool.md` | D-R9-5 (PIERCE coverage audit vs Sadida seed dominant strategy) | Warning |

---

## Verdict: FAIL

**11 blockers (8 consistency + 3 design) + 14 warnings.**

**Pillar status:**
- **Auction-as-signature: ✅ OK** — D-B3 reframing held; R3 signal-value risk is monitoring concern (D-R9-6/7), not breakage.
- **No-idle-spectating: 🔴 FAIL** — D-R9-3 (DRAFT_AUCTION hand-full lockout) is an explicit anti-pillar violation per lanes-and-lies-gdd.md:48. D-R9-1 (PLACEMENT overload on auction-followup rounds) inverts the pillar at the signature moment.
- **Deep emergence: ⚠️ CONCERN** — D-R9-2 (Xelor Mummy uncapped → dominant-strategy risk by R6) and D-R9-5 (Sadida seed AR + unverified PIERCE coverage) are candidate dominant-strategy/binary-counter risks.

---

## Required Actions Before Architecture

| # | Issue | Primary Files | Closes |
|---|---|---|---|
| 1 | NP message-table pass: register `S2CSingleObjectiveReveal` + `S2CActivationRejected` | network-protocol.md, entities.yaml | C-R9-1, C-R9-3, hand-ui OQ8, class-system NP-4 |
| 2 | CA Rule C-8 update: ascending trigger_index spawn order + NP D.2 cross-ref + AC | card-animations.md | C-R9-2, C-NEW-4, C-R8-13 |
| 3 | Add DRAFT_AUCTION to `C2SActivateCard` valid phases (NP + RSM Rule 15 + hand-ui PASSIVE_LOCKED) | network-protocol.md, round-state-machine.md, hand-ui.md | D-R9-3, D-R8-5, C-R9-W12 |
| 4 | Class System balance pass: add Mummy cap + worked Xelorium curve + Sadida density knob + Hand UI dep | class-system.md, game-config.md, entities.yaml | C-R9-7, D-R9-2, D-R8-1..3, D-R8-6, C-R9-W5..7, C-R9-W13 |
| 5 | DRAFT_INITIAL grid ownership decision (pick one owner, other defers) | hand-ui.md, shop-auction-ui.md | C-R9-5 |
| 6 | File `OQ-PLACEMENT-LOAD` in RSM + auction-followup timer knob | round-state-machine.md, game-config.md | C-R9-6, D-R9-1, D-R8-2, D-W6 |
| 7 | entities.yaml `S2CGameOver` registry fix: merge duplicate notes + update to 4 variants | entities.yaml | C-R9-4, C-R9-W14 |
| 8 | Strip "(new variant — see OQ2 below)" from keyword-system.md:166 | keyword-system.md | C-R9-8 |
| 9 | Master GDD §2 Player Fantasy layering paragraph | lanes-and-lies-gdd.md | D-R9-4, D-R8-7 |
| 10 | game-config.md struct: add 7 missing class-system fields | game-config.md | C-R9-W4, C-R9-W10 |

**Highest-leverage coordinated edit:** Items 1 + 3 (NP message pass + DRAFT_AUCTION valid-phase fix) are both NP edits that can be batched. Items 4 + 10 (class-system balance pass + game-config fields) are the highest design-risk reduction — Mummy cap + Xelorium curve closes 9 issues in two files.
