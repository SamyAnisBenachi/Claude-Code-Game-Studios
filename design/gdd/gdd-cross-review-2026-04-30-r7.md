# Cross-GDD Review Report — R7

**Date:** 2026-04-30 (R7 — same-day re-run after R6 in-session revisions)
**GDDs Reviewed:** 20 system GDDs + master GDD + systems-index
**Registry baseline:** entities.yaml v1 (2026-04-30 — 7 entities, 2 items, 22 formulas, 41 constants, 18 network messages)
**Prior review:** `gdd-cross-review-2026-04-30.md` (R6) — verdict FAIL, 9 blockers

---

## Progress History

| Review | Date | Verdict | Blockers |
|---|---|---|---|
| R1–R5 | 2026-04-29 | PASS | 0 (9 GDDs) |
| R6 | 2026-04-30 | FAIL | 9 (20 GDDs, 11 new) |
| **R7** | **2026-04-30** | **FAIL** | **9 (4 prior carryover + 5 new; 2 of original 9 fully resolved)** |

---

## R6 Blocker Disposition

| ID | R6 Status | R7 Status | Proof |
|---|---|---|---|
| C-B1 | 🔴 | ✅ RESOLVED | `network-protocol.md` Section D.2 has 11+ ResolutionEvent variants (CombatDamage, KeywordTriggered, DisplacementEvent, UnitDied, DeathTriggerFired, FinalBlowFired, AppearanceFired, etc.) |
| C-B2 | 🔴 | 🔴 STILL OPEN | `keyword-system.md:166` still uses `ResolutionTimeout`; OQ-NP2 (line 518) still requests new variant. Canonical 3-variant enum in registry/RSM/NP unchanged. KS and RSM-38/CR-41 still disagree. |
| C-B3 | 🔴 | 🟡 PARTIAL | NP enum renamed to `AcquisitionSource` (lines 550–552). **But `auction-system.md:89` still emits `CardSource::AuctionWon`** (stale). Registry note (line 1531) lists only PrismLane1–5 — incomplete variant set. See C-NEW-2, C-NEW-3. |
| C-B4 | 🔴 | 🔴 STILL OPEN | `hand-ui.md:68` Rule 5c still includes `S2CCardAcquired` in activation-lock resolver. NP:174 reinforces the conceptual error (KeywordEffect-grant cards arrive via S2CCardAcquired mid-RESOLUTION, never on C2SActivateCard). |
| C-B5 | 🔴 | 🟡 PARTIAL | Tuning Knobs (lines 337–338) firmly say "Client render config." **Dependencies table line 199 still says "Loaded at session start."** Internal contradiction unresolved. |
| C-B6 | 🔴 | 🔴 STILL OPEN | Zero matches for `S2CSangMepriseReveal` in `entities.yaml` `network_messages` section. Reconnect gap still untracked. |
| D-B1 | 🔴 | 🔴 STILL OPEN | `class-system.md:214` still says `destroy(chosen_enemy_objective)`; `class-system.md:260` still says `destroy(chosen_real_objective)`. Neither routes through `take_damage()`. No per-game Garde-Temps use limit added. |
| D-B2 | 🔴 | 🟡 PARTIAL | `class-system.md:404–406` defines "ascending trigger_index within the batch" as ordering rule + Punition mutual-destruction → Draw. R6 alternative (move Punition to sub-step 3+) rejected. **But ordering rule is class-system-local — RSM/NP/CA do not echo it.** See C-NEW-4. Sang Méprise + Punition lethal-to-2 combo not explicitly called out. |
| D-B3 | 🔴 | 🟡 DEFERRED | `auction-system.md:381` reframed as "Pillar Risk: Acknowledged. Decision deadline: before M3 closes." Treated as in-game escalation path, not a blocker for Architecture **iff** the team accepts an M2 playtest checkpoint. |

**Summary:** 2 of 9 R6 blockers fully resolved (C-B1, C-B3 partial→architectural unblock if completed), 4 fully open, 3 partial.

---

## Carryover Warnings (R5/R6)

| ID | Title | R7 Status |
|---|---|---|
| C-W1 | BLS F2 missing `fake_objective_spawn_advance` | ✅ RESOLVED |
| C-W2 | 2v2 spawn-range counter ownership | 🔴 Still open |
| C-W3 | `lobby_heartbeat_timeout_seconds` missing from game-config.md | ✅ RESOLVED |
| C-W4 | EC19/EC20 stale ACs in economy-system.md | ✅ RESOLVED |
| C-W5 | RSM stale "GDD not yet written" labels (Auction + CR) | 🔴 Still open (`round-state-machine.md:151,152,276,277`) |
| C-W6 | RSM stale tuning-knob note | ✅ RESOLVED |
| C-W7 | economy-system Interactions table incomplete | 🟡 Partial — `add_reserve()` mentioned but never formalised in Rule 3/4 |
| C-W8 | game-config Objective System interactions row incomplete | 🔴 Still open |
| C-W9 | server-rng.md doesn't list GSS as lifecycle owner | 🔴 Still open |
| C-W10 | Master GDD §3.3 + C9 "in that lane" stale | 🔴 Still open (lanes-and-lies-gdd.md:113, 827 vs §3.2 line 94 — internal contradiction) |
| C-W11 | RSM Rule 15 excludes C2SAcknowledgeResult from GAME_OVER | 🔴 Still open (`round-state-machine.md:119`) |
| C-W-NEW-1 | 10 GameConfig fields missing | 🟡 Partial (2/10 fixed: `prism_strike_damage`, `prism_strike_mana_cost` added). Still missing: `stagger_cadence_ms`, `type_advantage_atk_bonus`, `type_advantage_ar_bonus`, `garde_temps_reserve_cost`, `miss_nuit_cap`, `dé_chateux_reveal_threshold`, `seed_ar_bonus`, `seed_enemy_damage` |
| C-W-NEW-2 | OUTNUMBERED indicator per-lane vs per-unit | 🔴 Still open (`combat-resolution.md:495,554` per-lane) |
| C-W-NEW-3 | DRAFT_INITIAL grid ownership conflict | 🔴 Still open (hand-ui.md:55–61 vs shop-auction-ui.md:36,462,636) |
| C-W-NEW-4 | economy-system bidirectional deps incomplete | 🔴 Still open |
| C-W-NEW-5 | `add_reserve` API not formalised in economy-system.md | 🟡 Partial |
| C-W-NEW-6 | shop-auction-ui overlay durations stale (1.5s/1.0s) | 🔴 Still open |
| C-W-NEW-7 | Keyword/class RNG slots not in server-rng.md | 🔴 Still open |
| C-W-NEW-8 | Reveal-tween cross-reference (BR ↔ CA C-11) | 🔴 Still open |
| C-W-NEW-9 | objective-system.md missing prism/class downstream deps | 🔴 Still open |

**Resolved this cycle:** C-W1 only.

---

## New Consistency Issues (R7)

🔴 **C-NEW-1 — `S2CSingleObjectiveReveal` referenced by class-system but missing from network-protocol.md and registry**
**GDDs:** `class-system.md:597` (NP-4), `network-protocol.md` (no entry), `entities.yaml` (no entry)
class-system.md NP-4 explicitly requires `S2CSingleObjectiveReveal { player_id, lane, is_fake }` for Dé du Chateux's narrow-scope reveal (Ecaflip Krosmic CS-9). Without it, Dé du Chateux cannot replicate. Distinct payload from `S2CSangMepriseReveal` (single-lane vs full-board, opponent-only vs both players).
→ Add to `network-protocol.md` Section D.1 S2C table and `entities.yaml` `network_messages`.

🔴 **C-NEW-2 — Stale `CardSource::AuctionWon` reference in auction-system.md**
**GDDs:** `auction-system.md:89`, `network-protocol.md:550`, `entities.yaml:1529`
auction-system.md Rule 7 still emits `S2CCardAcquired { card_id, source: CardSource::AuctionWon }`. NP renamed the enum to `AcquisitionSource`; registry agrees. The string "CardSource" is dead.
→ Single edit — replace `CardSource::AuctionWon` with `AcquisitionSource::AuctionWon`.

🔴 **C-NEW-3 — `AcquisitionSource` enum variant set undefined**
**GDDs:** `network-protocol.md:550–552`, `entities.yaml:1531`, multiple consumers
C-B3 renamed the type but did not complete the variant set. Five variants are referenced across the codebase but registered nowhere: `AuctionWon`, `KeywordEffect`, `ShopPurchase`, `DraftInitial`, `FreeCardPick`. Only `PrismLane1..5` are formally declared.
→ Define complete variant set inline at `network-protocol.md:550–552` and update registry note.

⚠️ **C-NEW-4 — Multi-Krosmic batch ordering rule lives only in class-system.md; RSM/NP/CA do not echo it**
**GDDs:** `class-system.md:404–406`, `round-state-machine.md` (zero `trigger_index` matches), `network-protocol.md` (zero matches), `card-animations.md` (zero matches)
The Sang Méprise + Punition + Xelorium + Gelure determinism contract is anchored in a single sentence in class-system.md edge cases. RSM Rule 11 (PLACEMENT-commit pipeline), NP `S2CResolutionEvent` ordering invariants, and CA replay sequencer all need to know this — none mention "trigger_index" or any equivalent. D-B2 is unresolved at the contract level.
→ RSM Rule 11 (or new 11a) should document the multi-effect-same-sub-step ordering. NP `S2CResolutionEvent` should specify event emission order `(player_id, lane, trigger_index)` ascending within a sub-step. CA can then cite the rule.

⚠️ **C-NEW-5 — Reinforces D-B1: `destroy(...)` literal still in formula pseudocode while objective-system.md updated**
**GDDs:** `class-system.md:214,260`, `objective-system.md:50,246`
objective-system.md:246 has been updated to interpret Garde-Temps as `take_damage(lane, attacker, objective_hp)` parenthetically. class-system.md authoritative formula text still uses `destroy(...)`. Two sides of the contract use different verbs in primary specification.
→ Edit class-system.md CS-4 + CS-6 to call `take_damage()` directly.

⚠️ **C-NEW-6 — prism-system.md Rule 11 references removed `GoldAwardReason::PrismReward` variant**
**GDDs:** `prism-system.md:71` (says "should be removed"), `prism-system.md:283` (OQ3 "MUST FIX"), `network-protocol.md:503` (variant deliberately omitted; fix already made)
NP already removed the variant. prism-system.md still anticipates the removal as future work. Stale anticipatory text.
→ Update prism-system.md Rule 11 + OQ3 to past tense / closed.

⚠️ **C-NEW-7 — Registry single-pass needed: C-B6 + C-NEW-1 + C-NEW-3 all clear in one edit**
A single registry update (entities.yaml) can register `S2CSangMepriseReveal`, `S2CSingleObjectiveReveal`, and complete `AcquisitionSource` variants together. Currently 3 separate consistency gaps; 1 edit closes all.

⚠️ **C-NEW-8 — class-system.md downstream deps omit hand-ui.md and shop-auction-ui.md**
**GDDs:** `class-system.md:443–452` (only card-animations downstream listed)
Garde-Temps and Punition require player to "choose" an objective. Hand UI owns PLACEMENT staging and target resolution; class-aware target resolution must be declared. Bidirectional dependency gap.
→ Add Hand UI as Soft downstream in class-system.md Dependencies. Add class-system.md as upstream in hand-ui.md.

⚠️ **C-NEW-9 — Reinforces C-B4 with concrete counter-example**
NP:174 explicitly says keyword-effect cards arrive via `S2CCardAcquired { source: KeywordEffect }` mid-RESOLUTION, not on `C2SActivateCard`. So `S2CCardAcquired` will never resolve a DRAFT_SHOP Instant activation lock — it's only the message for new card acquisitions. Hand UI Rule 5c's resolver list is conceptually wrong AND now has an active counter-example in NP that will confuse implementers.

---

## New Design Issues (R7)

🔴 **D-B4 — Disconnect during DRAFT_AUCTION with active reservation: 30s grace UX undefined**
**Systems:** `auction-system.md` Rule 8 (line 100), `game-session-system.md:286`, `round-state-machine.md` Rule 13
**Pillar impact:** Auction-as-signature.
auction-system.md cleanly defines RSM-initiated `AbortAuction` cleanup (release reservation → IDLE → no AuctionSettled). But there is no spec for what the surviving player sees during the 30s grace if their opponent disconnects mid-bid. Does the timer keep running? Does the surviving player's reservation stay locked? Is the auction "frozen"? Nowhere defined. If the auction settles during grace with the disconnected player as still-leader, gold is spent on an unwinnable card.
→ Add to auction-system.md Rule 8: behavior during `disconnect_grace_seconds`. Recommended: freeze timer + lock panel ("Opponent disconnected — 30s grace") with reservation preserved.

🔴 **D-B5 — Sang Méprise reconnect gap is asymmetric and silent**
**Systems:** `class-system.md:251,435`, `network-protocol.md:730`, `objective-system.md`, `hud.md`
**Pillar impact:** No idle spectating.
class-system.md:435: reconnected client "degrades gracefully: objectives appear hidden for the rest of that RESOLUTION." But the Sacrier paid spell cost; opponent retains full reveal information. No client-side messaging tells the player "you missed your reveal due to disconnect." `S2CGameSnapshot` does not carry `active_sang_meprise_identities`. R6 self-flagged this; nothing changed.
→ Minimum viable: add a HUD overlay "Reveal lost on reconnect — Sacrier ability information unavailable for this RESOLUTION." Preferred: add `active_sang_meprise_reveals: Vec<(player, identities)>` to `S2CGameSnapshot`. Lock before Architecture.

⚠️ **D-W6 — Player attention budget during PLACEMENT now exceeds 4 active systems**
**Systems:** hand-ui, board-rendering, hud, card-animations, prism-system, class-system
PLACEMENT (10s) requires the player to track simultaneously: hand fan + Instant plate, drag-stage to board cells, timer ticks, gold/mana/reserve display, lane/objective HP, staged-card overlay, plus class-specific overlays (Xelor reserve readout, Sadida seed-position highlights). 7 layers concurrent. R6's D-W5 noted 5; class-system additions push to 7. Auto-stage on timer expiry addresses only the cursor-at-expiry edge.
→ File OQ-PLACEMENT-LOAD in round-state-machine.md: if playtests show >25% missed placements on auction-followup rounds OR class-specific decisions abandoned in <5s, raise `placement_timer_seconds` from 10 to 12 OR collapse class overlays to a togglable info panel.

⚠️ **D-W7 — Xelorium + Gelure same PLACEMENT batch creates uncounter-able mana drain**
**Systems:** `class-system.md:404`, `economy-system.md`, `combat-resolution.md`
class-system.md:404 declares the combo "legal high-burst combo." But: opponent's full `current_mana` AND any Xelor residual mana both transfer to Xelor reserve in one PLACEMENT. Worst case: opponent at 8 mana → Xelor reserve +8 in one round. Combined with Miss Nuit (cap 2/round) and Lane 2/4 prism farming, Xelor reaches Garde-Temps (cost 20) by round 4–5 with no board commitment. Few counters in pool besides "play Xelorium yourself" or SILENCE the Mummy.
→ (a) document in class-system.md Edge Cases as "monitor playtest"; (b) add tuning knob `xelorium_steal_cap` (default unbounded, max 5); (c) gate Garde-Temps if reserve would exceed 20 same-tick. Pick (a) for now; (b)/(c) reserved as post-playtest fixes.

⚠️ **D-W8 — Garde-Temps direct destroy bypasses ObjectiveDestroyed animation choreography**
**Systems:** `class-system.md:214`, `objective-system.md` Rule 5, `card-animations.md`, `board-rendering.md`
Same root cause as D-B1. Two destruction code paths = two animation contracts. Card Animations AnimQueue has no source of truth on whether to play the 500ms reveal hold + ObjectiveDestroyed sequence on a Garde-Temps trigger.
→ Resolved with D-B1 fix.

---

## Cross-System Scenario Walkthroughs (Phase 4 — 6 new scenarios)

| # | Scenario | Severity | Maps to |
|---|---|---|---|
| A | Reconnect during active Sang Méprise | ⚠️ WARNING | D-B5 |
| B | Auction settle race vs hand-full (S2CGoldBroadcast / S2CAuctionSettled / S2CCardAcquired same-tick FIFO) | ⚠️ WARNING | shop-auction-ui OQ8 |
| C | Xelorium + Gelure same PLACEMENT batch (sub-step 1 ascending trigger_index) | ⚠️ WARNING | D-W7 |
| D | Mass-token DEATH chain (Bow Meow / Chafer / Sadida Pollinisation) | 🔴 BLOCKER | KS OQ-NP3 + UnitDied missing chain-position |
| E | Disconnect during DRAFT_AUCTION with active reservation | 🔴 BLOCKER | D-B4 |
| F | Punition + opponent's active Sang Méprise mirror | ℹ️ INFO | (clean — design tension only) |

### Scenario detail — Scenario D (BLOCKER, new)
**Trigger:** Sadida casts Pollinisation at sub-step 1 → seed creation. Bow Meow dies → Chafer DEATH-trigger spawns Decrepit Chafer → chain length 3–5 deaths in sub-step 4.
**Failure:** `keyword-system.md` OQ-NP3 still open; `UnitDied` does not encode chain position. Animation queue cannot deterministically order pulses; seeds placed at chain-link cells trigger walk-overs immediately on next round; CA `total_duration_ms` balloons past 5s ceiling. Maps to existing C-B1 follow-up + KS OQ-NP3.

### Scenario detail — Scenario E (BLOCKER, new)
**Trigger:** Player A bids 5g, becomes leader, reserved_gold=5. Disconnects at second 4 of 8. RSM behaviour during 30s grace ambiguous.
**Failure:** auction-system Rule 8 only covers RSM-initiated `AbortAuction`. If auction times out during grace with disconnected player as still-leader, `S2CAuctionSettled` fires; reserved_gold spent; on grace expiry the player loses; unwinnable card given to disconnected player. Maps to D-B4.

---

## GDDs Flagged for Revision

| GDD | Issues | Priority |
|---|---|---|
| `network-protocol.md` | C-B2 (ResolutionTimeout decision), C-NEW-1 (S2CSingleObjectiveReveal), C-NEW-3 (AcquisitionSource variants), C-NEW-4 (echo trigger_index ordering) | **Blocking** |
| `keyword-system.md` | C-B2 (align ResolutionTimeout → Draw, close OQ-NP2) | **Blocking** |
| `class-system.md` | D-B1 + C-NEW-5 (Garde-Temps + Punition `take_damage()` routing), D-W7 (Xelorium+Gelure note), C-NEW-8 (hand-ui dep) | **Blocking** |
| `objective-system.md` | D-B1 (interface contract), C-W-NEW-9 (bidirectional deps to prism + class) | **Blocking** |
| `hand-ui.md` | C-B4 (remove S2CCardAcquired from Rule 5c resolver), C-B5 (resolve Tuning Knobs vs Dependencies internal contradiction), C-NEW-8 (class-system upstream dep), C-NEW-9 | **Blocking** |
| `auction-system.md` | C-NEW-2 (stale CardSource::AuctionWon), D-B4 (disconnect grace UX), D-B3 (M3 deadline tracked) | **Blocking** |
| `entities.yaml` | C-B6, C-NEW-1, C-NEW-3, C-NEW-7 — all closed by single registry pass | **Blocking** |
| `round-state-machine.md` | C-NEW-4 (echo trigger_index ordering Rule 11a), C-W5 (stale "GDD not yet written" labels), C-W11 (Rule 15 wording), D-W6 (OQ-PLACEMENT-LOAD) | Warning + Blocking |
| `prism-system.md` | C-NEW-6 (close PrismReward OQ3), D-W-NEW-4 (WALL+prism farming OQ) | Warning |
| `game-config.md` | C-W-NEW-1 (8 fields still missing) | Warning |
| `combat-resolution.md` | C-W-NEW-2 (OUTNUMBERED per-unit), C-NEW-4 echo | Warning |
| `economy-system.md` | C-W-NEW-4 (S2CGoldBroadcast + SAU/HUI deps), C-W-NEW-5 (`add_reserve` API), D-W-NEW-3 dependency note | Warning |
| `card-animations.md` | C-W-NEW-6 (overlay duration sync), C-W-NEW-8 (reveal-tween catalog), C-NEW-4 echo | Warning |
| `board-rendering.md` | C-W-NEW-2 (OUTNUMBERED), C-W-NEW-8 (reveal-tween cross-ref) | Warning |
| `server-rng.md` | C-W9 (GSS lifecycle), C-W-NEW-7 (4 new RNG slots) | Warning |
| `shop-auction-ui.md` | C-W-NEW-3 (DRAFT_INITIAL ownership), C-W-NEW-6 (overlay durations) | Warning |
| `lanes-and-lies-gdd.md` | C-W10 ("in that lane" stale text vs §3.2 global rule) | Warning |

---

## Verdict: **FAIL**

9 blocking issues remain. Net change from R6: 2 fully resolved (C-B1, partial C-B3), 5 still fully open (C-B2, C-B4, C-B6, D-B1, plus 2 partials), 5 NEW blockers introduced by R6 in-session revisions (C-NEW-1, C-NEW-2, C-NEW-3, D-B4, D-B5).

### Required actions before Architecture
| # | Issue | GDDs to Update |
|---|---|---|
| 1 | C-B2: lock `ResolutionTimeout` → `Draw` in keyword-system.md (close OQ-NP2) | `keyword-system.md` |
| 2 | C-B4: remove `S2CCardAcquired` from hand-ui.md Rule 5c resolver | `hand-ui.md` |
| 3 | C-B5: pick "Client render config" — strip "Loaded at session start" from Dependencies table line 199 | `hand-ui.md` |
| 4 | C-B6 + C-NEW-1 + C-NEW-3 + C-NEW-7: single registry pass — register `S2CSangMepriseReveal`, `S2CSingleObjectiveReveal`, complete `AcquisitionSource` variants | `entities.yaml`, `network-protocol.md` |
| 5 | C-NEW-2: replace `CardSource::AuctionWon` with `AcquisitionSource::AuctionWon` in auction-system.md:89 | `auction-system.md` |
| 6 | C-NEW-4: echo multi-Krosmic ordering rule in RSM Rule 11a + NP S2CResolutionEvent emission contract | `round-state-machine.md`, `network-protocol.md` |
| 7 | D-B1 + C-NEW-5: Garde-Temps + Punition route through `take_damage()`; per-game Garde-Temps use limit (recommended 1) | `class-system.md` |
| 8 | D-B4: define DRAFT_AUCTION + 30s disconnect grace UX (recommended: freeze timer, preserve reservation) | `auction-system.md` |
| 9 | D-B5: minimum HUD overlay for Sang Méprise reveal-lost-on-reconnect; preferred: add to S2CGameSnapshot | `class-system.md` or `network-protocol.md` |

### Notes for follow-up review
- D-B3 (auction wealth gap) is now treated as deferred-with-escalation (M2 playtesting → M3 deadline). If team agrees this is non-blocking for Architecture, drop from blocker list.
- C-W-NEW-1 (8 GameConfig fields) is mechanical busywork; should clear before M3 implementation epics begin.
- 6 new Phase 4 scenarios walked; 2 are BLOCKERS (D, E) — both surface the same cross-system contract gaps already flagged in C-NEW-4 / D-B4.
