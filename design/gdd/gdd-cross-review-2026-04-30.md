# Cross-GDD Review Report

**Date:** 2026-04-30
**GDDs Reviewed:** 20 system GDDs + master GDD + systems-index
**New since last review (R5 2026-04-29):** 11 (card-acquisition, auction-system, combat-resolution, board-rendering, hand-ui, shop-auction-ui, hud, keyword-system, prism-system, class-system, card-animations)
**Registry baseline:** entities.yaml v1 (2026-04-30)

---

## Progress History

| Review | Date | Verdict | Blockers |
|---|---|---|---|
| R1–R5 | 2026-04-29 | PASS | 0 (9 GDDs) |
| R6 | 2026-04-30 | **FAIL** | 9 (20 GDDs, 11 new) |

---

## Consistency Issues

### Blocking — must resolve before architecture begins

🔴 **C-B1 — `S2CResolutionEvent` missing 3 required variant families**
**GDDs:** `combat-resolution.md` OQ5, `keyword-system.md` OQ-NP1/OQ-NP5, `card-animations.md` Rule C-10, `board-rendering.md`

`S2CResolutionEvent` is the primary wire signal for all RESOLUTION animation replay. Three variant families are explicitly required across multiple GDDs but do not exist in `network-protocol.md` (zero matches verified):

- `CombatDamage { attacker_id, defender_id, damage_amount, was_blocked_by_shield, sub_step }` — required by CR OQ5, Board Rendering damage-flash AC, Card Animations domain-event indirection
- `KeywordTriggered { unit_id, keyword, sub_step, ... LEADER/BODYGUARD/OUTNUMBERED snapshot }` — required by KS OQ-NP5 Replication Contract; 11 ACs depend on it
- `DisplacementEvent { unit_id, attacker_id, keyword: DisplacementKind, from_cell, to_cell, sub_step, was_blocked }` — required by KS OQ-NP1 for REPEL/ATTRACT/TELEPORT animations

Without these, Board Rendering can display no damage numbers, no SHIELD-absorption feedback, no displacement animations — all BLOCKING ACs across CR and KS fail.
→ **Add all three to `network-protocol.md` Section D.2. Register in `entities.yaml` `network_messages` section.**

---

🔴 **C-B2 — `GameOverReason::ResolutionTimeout` claimed by Keyword System; not in canonical enum**
**GDDs:** `keyword-system.md` Replication Contract, `combat-resolution.md` CR-41, `round-state-machine.md` Rule 14 + RSM-38, `entities.yaml`, `network-protocol.md`

Canonical `GameOverReason` enum: `{ ObjectivesDestroyed, Disconnection, Draw }` (3 variants — RSM Rule 14, registry, NP). Keyword System Replication Contract asserts `S2CGameOver { reason: ResolutionTimeout }` as a required new variant for the 60s safety timeout. CR-41 and RSM-38 both already specify the same event as `reason: Draw`. Two documents disagree on which variant represents a RESOLUTION timeout.
→ **Decision required before architecture:** Either (a) KS Replication Contract aligns to `Draw` (consistent with RSM-38 and CR-41 — the simpler fix, recommended), or (b) add `ResolutionTimeout` as a 4th variant to all 5 sources simultaneously. Choice (a): `ResolutionTimeout` is semantically a Draw with no winning player — update KS Replication Contract and close KS OQ-NP2.

---

🔴 **C-B3 — `S2CCardAcquired` payload has two different enum names and an incomplete variant set**
**GDDs:** `network-protocol.md` line 105 (`CardSource`), `auction-system.md` Rule 7 (`CardSource::AuctionWon`), `prism-system.md` Rule 4 (`PrismLane{L}`), `entities.yaml` line 1489 (`AcquisitionSource`), `card-acquisition.md`, `objective-system.md`

Three distinct names in active use for the same enum: `CardSource` (NP + auction-system), `AcquisitionSource` (registry), unnamed (prism-system). NP enum lines 393–401 list only `PrismLane1..5` — missing: `AuctionWon` (auction-system.md Rule 7), `FreeCardPick` (objective reward path), `KeywordEffect` (NP line 477 itself). Any system consuming this message (hand-ui.md, board-rendering.md) gets an underspecified enum with missing variants.
→ **Pick one name** (recommend `AcquisitionSource` to match registry). Define complete variant set in `network-protocol.md` and update `entities.yaml` to match. Variants needed: `PrismLane1–5`, `AuctionWon`, `ShopPurchase`, `DraftInitial`, `FreeCardPick`, `KeywordEffect`.

---

🔴 **C-B4 — Hand UI activation lock cannot reliably resolve: `S2CCardAcquired` never fires on card activation**
**GDDs:** `hand-ui.md` Rule 5c + HU-28 + HU-29, `network-protocol.md`, `prism-system.md`

`hand-ui.md` Rule 5c: on `C2SActivateCard` (Instant card play), the slot enters a locked state until "(a) `S2CCardAcquired` / `S2CGoldUpdate` confirms activation OR (b) `activate_timeout_ms` (3000ms) elapses." But `S2CCardAcquired` is the message for *acquiring* a new card — it is never sent when *playing* a card already in hand. For Instant cards whose only effect is on reserve mana (e.g., `prism_reserve`), `S2CGoldUpdate` fires (reserve_mana field) and the lock resolves correctly. For any hypothetical zero-side-effect Instant, neither message fires and the slot locks for 3 full seconds. Additionally, `S2CCardAcquired` appearing in the resolver list is a conceptual error that will confuse implementers.
→ **Remove `S2CCardAcquired` from Rule 5c's resolver list.** The lock resolves on `S2CGoldUpdate` only (or a new `S2CCardActivated` confirmation if zero-side-effect instants exist). Clarify in HU-28/29.

---

🔴 **C-B5 — `activate_timeout_ms` and `purchase_timeout_ms` missing from `game-config.md`**
**GDDs:** `hand-ui.md` Tuning Knobs + Dependencies table

`hand-ui.md` Dependencies table line 199 lists both fields as "Loaded at session start from Game Config." Tuning Knobs table tags them "Client render config" — a contradiction within the same GDD. Verified: zero matches in `game-config.md`.
→ **Pick one source.** If client-only: remove from Dependencies table, mark "client compile-time constant" in Tuning Knobs. If server-driven: add both to `game-config.md` struct + Tuning Knobs, and to `entities.yaml` constants section. Recommended: client compile-time constants (they govern client-side timeout UX only).

---

🔴 **C-B6 — `S2CSangMepriseReveal` missing from `entities.yaml` registry**
**GDDs:** `class-system.md` CS-AC-13 + OQ-CS-2, `network-protocol.md` line 125, `objective-system.md` OQ6, `hud.md` (implicit reconnect-gap)

`S2CSangMepriseReveal { identities: Vec<(lane, is_fake)> }` is defined in `network-protocol.md` and referenced by 3+ GDDs but is not registered in `entities.yaml` `network_messages` section. The registry is the cross-system source of truth; this omission means the reconnect gap (OQ-CS-2) is not formally tracked, and `consistency-check` will not detect future deviations.
→ **Add to `entities.yaml`:** `source: objective-system.md`, `referenced_by: [class-system, network-protocol, objective-system]`. Also confirm reconnect gap is formally tracked (OQ-CS-2).

---

### Warnings — should resolve but won't block architecture

⚠️ **C-W-NEW-1 — 10 new GameConfig fields registered in `entities.yaml` but absent from `game-config.md`**
`stagger_cadence_ms`, `type_advantage_atk_bonus`, `type_advantage_ar_bonus`, `garde_temps_reserve_cost`, `miss_nuit_cap`, `dé_chateux_reveal_threshold`, `seed_ar_bonus`, `seed_enemy_damage`, `prism_strike_damage`, `prism_strike_mana_cost` — all registered with `referenced_by: design/gdd/game-config.md` but absent from `game-config.md` struct, Interactions table, and Tuning Knobs section. Must be added before M3 implementation epics begin.

⚠️ **C-W-NEW-2 — OUTNUMBERED indicator: Combat Resolution says per-lane; Keyword System says per-unit (global count)**
`combat-resolution.md` Visual section: "Crimson Slate arrow-down on player's lane-line side (per lane)." `keyword-system.md` Replication Contract + Formula 3: count is global across all lanes; the indicator is per-unit. Already self-flagged as KS OQ-KS5. `board-rendering.md` Rule 14 aligns with KS (per-unit). CR Visual section needs updating.

⚠️ **C-W-NEW-3 — Ownership conflict: DRAFT_INITIAL 3×3 grid click handler claimed by both Hand UI and Shop/Auction UI**
`hand-ui.md` Rule 4 and `shop-auction-ui.md` Rule 3 both describe clicking the DRAFT_INITIAL 3×3 card grid and sending `C2SPurchaseCard`. Two presentation systems own the same UI surface. Recommended: Shop/Auction UI owns the DRAFT_INITIAL panel entirely; Hand UI Rule 4 applies only to DRAFT_SHOP hand management.

⚠️ **C-W-NEW-4 — `economy-system.md` not updated to list `S2CGoldBroadcast`, `shop-auction-ui.md`, `hand-ui.md` as dependents**
`shop-auction-ui.md` OQ7 self-flags this gap. `hand-ui.md` Rule 13 (reserve-split) also reads reserve from economy state. Bidirectional dependency audit required.

⚠️ **C-W-NEW-5 — `add_reserve(player, n)` API name referenced by 3 GDDs but never formally defined in `economy-system.md`**
`prism-system.md` Rule 4 and `class-system.md` Interactions table reference `add_reserve()` by name. `economy-system.md` describes the mechanic abstractly without naming the API. Add a formal API entry in economy-system.md Rule 3/4.

⚠️ **C-W-NEW-6 — Card Animations OQ-CA-07/08 propose Shop/Auction UI overlay duration changes (1.5s → 0.8s; 1.0s → 0.25s); SAU still encodes old values**
`shop-auction-ui.md` Rule 9 + SAU-V10 encode 1.5s/1.0s. Must be resolved in one of the two GDDs.

⚠️ **C-W-NEW-7 — New keyword and class RNG seed slots not registered in `server-rng.md`**
`keyword-system.md` OQ-KS1: 3 named slots (`range_equidistant_select`, `teleport_random_dest`, `strich_change_lane_select`). `class-system.md` CS-AC-22: `de_chateux_roll` (implied 4th slot). None in `server-rng.md` seed table. Must be registered before implementation.

⚠️ **C-W-NEW-8 — Reveal-tween spec split between Board Rendering and Card Animations with no cross-reference**
`board-rendering.md` Rule 7 defines the placement-reveal tween (scale 0.4→1.0 + alpha 0→1, 250ms). `card-animations.md` Easing Catalog (Rule C-11) has no matching entry. Add a row to C-11 or a cross-reference in BR Rule 7 to CA C-11.

⚠️ **C-W-NEW-9 — `objective-system.md` not updated to list `prism-system.md` or `class-system.md` as downstream dependents**
`prism-system.md` consumes `objective_damage` formula. `class-system.md` Garde-Temps + Punition + Sang Méprise all route through Objective System. Bidirectional deps required per design-docs.md rules.

**Carryover status (from R5 2026-04-29):**

| Warning | Status |
|---|---|
| C-W1: BLS F2 missing `fake_objective_spawn_advance` | Still open |
| C-W2: 2v2 spawn-range counter ownership | Still open |
| C-W3: `lobby_heartbeat_timeout_seconds` missing from game-config.md | ✅ RESOLVED |
| C-W4: EC19/EC20 stale ACs in economy-system.md | ✅ RESOLVED |
| C-W5: RSM labels authored GDDs "not yet written" | Still open (Auction + CR still labeled) |
| C-W6: RSM stale tuning-knob note | ✅ RESOLVED |
| C-W7: economy-system.md Interactions table incomplete | Partially resolved (Dependencies fixed; Interactions still missing fields) |
| C-W8: game-config.md Objective System interactions row incomplete | Still open |
| C-W9: server-rng.md doesn't list GSS as lifecycle owner | Still open |
| C-W10: Master GDD §3.3 + C9 "in that lane" stale | Still open |
| C-W11: RSM Rule 15 excludes C2SAcknowledgeResult from GAME_OVER | Still open |

---

## Game Design Issues

### Blocking — must resolve before architecture begins

🔴 **D-B1 — Garde-Temps routing ambiguity: direct destroy vs. `take_damage()` interface conflict**
**Systems:** `class-system.md` CS-4, `objective-system.md` Rule 5 + Rule 7, `combat-resolution.md`

`objective-system.md` Rule 5 states all damage flows through `take_damage()` — the authoritative interface that enforces the 500ms reveal hold, fake-reveal drama beat, and consequence audit trail. `class-system.md` CS-4 says "chosen enemy objective HP → 0" but does not specify whether this calls `take_damage(objective_hp)` or a direct destroy. These are not equivalent: a direct destroy path bypasses the fake-reveal beat, ObjectiveDestroyed animation sequence, and replay-log entry.

Architecture cannot specify the Objective System's API surface without this being locked. If Garde-Temps uses a direct destroy path, it contradicts `objective-system.md` Rule 5 and creates a second destruction code path that bypasses the audit trail.

→ **Lock routing:** Garde-Temps MUST call `take_damage(lane, attacker_player, amount = objective_hp)` — lethal damage through the standard interface. State explicitly in `class-system.md` CS-4 Detailed Rules. Additionally: impose a per-game Garde-Temps use limit (recommended: 1) in `class-system.md` to cap the uncapped-reserve exploitation path from D-W4.

---

🔴 **D-B2 — Sacrier Sang Méprise + Punition combo: sub-step 1 ordering undefined; potential same-round 2-objective deletion**
**Systems:** `class-system.md` CS-5, CS-6, `objective-system.md` Rule 7, `round-state-machine.md` Rule 11, `card-animations.md`

Both Sang Méprise and Punition resolve at sub-step 1. No GDD specifies ordering when two Krosmic spells from the same player fire at the same sub-step. If both are in the same PLACEMENT batch: Sang Méprise reveals alive objectives → Punition deals 3 damage to each alive opponent objective (untargeted AOE). On a board where 2 real objectives are at HP ≤ 3, Punition delivers lethal damage to both in a single round — a same-round game-ending sequence with no positional requirement and no economic sacrifice beyond having both cards in hand.

The ordering ambiguity also creates non-deterministic animation sequences in `card-animations.md` S2CResolutionEvent replay (event ordering at source must be deterministic for client replay to work).

→ **Two options:** (a) Add an explicit sub-step ordering rule for multi-Krosmic batches in `class-system.md` and `round-state-machine.md` (alphabetical by card_id, or by player-submitted order); (b) Restrict Punition to sub-step 3+ (after movement) to require board presence rather than pure spell-casting — this is the stronger design fix as it gates the AOE behind positional commitment. Lock this decision in `class-system.md` + `round-state-machine.md` before Architecture.

---

🔴 **D-B3 — Auction wealth gap: trailing-player non-participation violates the signature mechanic pillar**
**Systems:** `auction-system.md` OQ7, `economy-system.md` Rule 6, `hud.md`, `game-config.md`

`auction-system.md` OQ7 explicitly acknowledges: "At 35g vs 10g, the wealthy player can open at opponent_gold + 1g and win without a real contest." The interest formula compounds the gap (+2g/round for the wealthy, +0g for the broke) — a 25g deficit grows by ~2g/round. By round 9, the trailing player's auction participation has degraded from negotiation to watching. This directly violates the "auction as a lie detector with a price tag" pillar and the "no passive spectating" anti-pillar.

Architecture cannot begin without a mitigation choice because the fix has cross-system implementation implications:
- **Option A (recommended): Free-bid floor** — trailing player's first bid of each auction costs 0 gold if the gold deficit exceeds `auction_bid_floor_deficit` (new GameConfig field). Preserves negotiation fantasy (trailing player can probe intent without gold cost).
- **Option B: Trailing-player interest multiplier** — player more than X gold behind gets +1 interest/round. Requires economy-system.md change.
- **Option C: Gold consolation on lost auction** — losing bidder receives 1g back per bid round. Requires auction-system.md + economy-system.md change.

→ **Pick one option and update `auction-system.md` OQ7 to RESOLVED before Architecture.**

---

### Warnings — should resolve but won't block architecture

⚠️ **D-W1 (Modified) — Double-fake-same-RESOLUTION: combined reward is potentially game-ending**
Design explicitly blesses interest-bracket capture on single fake destruction (objective-system.md design note). Double-fake same-RESOLUTION rewards: +6g, +2 spawn range, +1 mana cap + free card pick (or two of either). The 10-configuration deduction requirement (C(5,2)=10) provides the intended skill gate. Document as an accepted design risk with a "monitor first-playtest" note. If games consistently end the round both fakes are destroyed, add a deferred-reward rule for same-round double-fake.

⚠️ **D-W2 (Mitigated) — RESOLUTION passive ceiling inadequate for complex keyword chains**
Card Animations enforces a 5s ceiling; Board Rendering accepts veteran passive-watch trade-off. Residual: DEATH chains + multi-keyword triggers could exceed 5s. Recommend adding to `card-animations.md`: "If AnimQueue `total_duration_ms` exceeds 7,000ms, compress by reducing `board_sub_step_duration_ms` proportionally to fit within 7,000ms."

⚠️ **D-W3 (Mitigated) — Manual refresh as only Rare-without-auction path**
Acknowledged as intentional miser/gambler tension. Document in `economy-system.md` Edge Cases: effective Rare acquisition cost via refresh = 5g (bracket lost + refresh + card cost) for players at exactly 5g.

⚠️ **D-W4 — Xelor reserve loop rewards passivity (perverse incentive vs. "no idle spectating")**
Xelor's optimal Gelure play is to leave some mana unspent and transfer to reserve — a strategic incentive to play fewer units than optimal. Board pressure is the stated organic limiter but requires playtesting to validate. Document in `class-system.md` as a design tension; monitor Xelor win-rate vs. passive-play correlation.

⚠️ **D-W5 (Partially addressed) — PLACEMENT cognitive load: 5 active systems in 10 seconds**
Hand UI Rule 9 (auto-stage on timer expiry) is the primary mitigation. Remaining: PLACEMENT starts immediately on phase transition, panel animation overlaps first 350ms. Document in `round-state-machine.md`: "Post-auction PLACEMENT start: timer begins on phase transition, not after panel animation completes. If >20% of playtests show missed placements on auction rounds, add `post_auction_placement_buffer_ms` GameConfig field (default 0)."

⚠️ **D-W-NEW-1 — Ecaflip Craps 8-damage variant: skill-independent multi-objective deletion**
Craps (coin-flip 0) deals 8 damage spread among alive objectives. On a late-game board where 2+ objectives are at HP≤4, this can delete them without any skill, deduction, or economic sacrifice. Document as accepted design risk in `class-system.md` CS-10 — Ecaflip's value proposition is intentional high-variance. If playtesting shows opponents feel they lost to luck rather than Ecaflip skill, reduce 8-damage to 6 or restrict to unit targets.

⚠️ **D-W-NEW-2 — Sadida Graines de Folie: spawn rejection is silent and punishing without pre-cast UI warning**
If 6 of 8 Seeds are in occupied lanes, Graines de Folie costs full mana for 2 Madolls. Players expect 8. Add to `class-system.md` CS-8 + `hand-ui.md` / `shop-auction-ui.md` UI Requirements: "Pre-cast: show eligible spawn count. If 0 eligible, disable or warn before submission."

⚠️ **D-W-NEW-3 — Xelorium timing OQ must be resolved before Economy System integration ADR**
If steal captures mana "at sub-step 1 commit" (opponent spent all mana on placed cards), steal = 0 and the 4-mana cost is wasted. Recommended: steal captures `opponent.current_mana` as it stood at PLACEMENT submission time (server-side snapshot before PlacementBuffer commit). Document in `class-system.md` CS-2 and `economy-system.md`.

⚠️ **D-W-NEW-4 — WALL + Lane 2/4 prism farming accelerates Xelor Garde-Temps beyond intended timeline**
WALL-park in Lanes 2 and 4 simultaneously generates +10 reserve from prisms alone over 5 rounds — on top of Gelure + Miss Nuit. Round-6 Garde-Temps becomes achievable without meaningful board commitment. Add OQ to `prism-system.md`: "If WALL-park + Xelor reserve proves too strong, consider: prism_reserve only grants reserve if collecting unit is not a WALL; OR prism respawn requires unit to reach opponent's half first."

---

## Cross-System Scenario Issues

**Scenarios walked: 5**

### Scenario 1 — Garde-Temps fires: Objective routing chain

**Trigger:** Xelor submits Garde-Temps (Instant) in PLACEMENT. Sub-step 1 fires.
**Systems:** class-system.md + objective-system.md + combat-resolution.md + network-protocol.md
**Failure mode:** 🔴 BLOCKER — "HP → 0" routing is UNDEFINED. Direct destroy bypasses `objective-system.md` Rule 5 `take_damage()` interface, 500ms reveal hold, fake-reveal beat, and ObjectiveDestroyed animation sequence. Maps to D-B1.
→ Routing must be locked before Architecture.

---

### Scenario 2 — Sang Méprise + Punition same PLACEMENT batch

**Trigger:** Sacrier submits both Krosmics in same PLACEMENT. Sub-step 1 fires.
**Systems:** class-system.md + objective-system.md + round-state-machine.md + card-animations.md
**Failure mode:** 🔴 BLOCKER — No ordering rule exists for multi-Krosmic same-sub-step batches. Non-deterministic animation replay in `card-animations.md`. On a wounded board, combo delivers lethal to 2 real objectives in one round. Maps to D-B2.
→ Ordering rule must be defined before Architecture.

---

### Scenario 3 — Auction settle → S2CCardAcquired → Hand UI

**Trigger:** Auction timer expires, winner receives card via `S2CCardAcquired { source: AuctionWon }`.
**Systems:** auction-system.md + network-protocol.md + hand-ui.md
**Failure mode:** 🔴 via C-B3 — `AuctionWon` variant not in NP enum (only PrismLane1–5 listed). Client cannot parse the source field; enum variant undefined. Maps to C-B3.
→ S2CCardAcquired enum must be complete before any acquisition code is written.

---

### Scenario 4 — Double-fake same RESOLUTION

**Trigger:** Both opponent fakes destroyed in same sub-step 5.
**Systems:** objective-system.md + economy-system.md + round-state-machine.md + board-lane-system.md
**Failure mode:** ℹ️ INFO — Rules are consistent. Each fake fires independent reward; RSM checks real_destroyed ≥ 2 after all objectives processed; no GAME_OVER from fake destruction. Economy swing (+6g + 2 reward rolls) is a design concern (D-W1), not a system fault.

---

### Scenario 5 — WALL-park prism → hand-full → DRAFT_SHOP

**Trigger:** WALL collects Lane 2 prism; player at 9 cards → hand becomes 10 via `S2CCardAcquired`.
**Systems:** prism-system.md + economy-system.md + hand-ui.md
**Failure mode:** ℹ️ INFO — `prism_reserve` is an Instant; `hand-ui.md` Rule 5 allows Instant plays during DRAFT_SHOP regardless of HandFullLocked state. Player plays prism_reserve → `S2CGoldUpdate` (reserve_mana += 1) → lock resolves → hand drops to 9. System interaction is clean. Reserve accumulation concern is D-W-NEW-4 (design warning).

---

## GDDs Flagged for Revision

| GDD | Issues | Priority |
|---|---|---|
| `network-protocol.md` | C-B1 (S2CResolutionEvent variants), C-B2 (GameOverReason), C-B3 (S2CCardAcquired schema) | **Blocking** |
| `keyword-system.md` | C-B2 (claims ResolutionTimeout not in canonical enum) | **Blocking** |
| `class-system.md` | D-B1 (Garde-Temps routing + use limit), D-B2 (Sang Méprise + Punition ordering), D-W-NEW-2/3/4 | **Blocking** |
| `hand-ui.md` | C-B4 (activation lock), C-B5 (GameConfig fields), C-W-NEW-3 (DRAFT_INITIAL ownership) | **Blocking** |
| `auction-system.md` | D-B3 (OQ7 mitigation required before architecture) | **Blocking** |
| `objective-system.md` | D-B1 (Garde-Temps must route through take_damage), C-W-NEW-9 (bidirectional deps) | **Blocking** |
| `entities.yaml` | C-B6 (S2CSangMepriseReveal unregistered), C-B3 (AcquisitionSource naming) | **Blocking** |
| `game-config.md` | C-W-NEW-1 (10 missing fields from class/prism/CR/animations) | Warning |
| `round-state-machine.md` | C-W5 (stale "not yet written" labels), C-W11 (Rule 15 wording), D-B2 (Krosmic ordering rule) | Warning / Blocking |
| `combat-resolution.md` | C-W-NEW-2 (OUTNUMBERED per-lane vs global), D-W-NEW-3 context | Warning |
| `economy-system.md` | C-W-NEW-4 (S2CGoldBroadcast + SAU/HUI deps missing), C-W-NEW-5 (add_reserve API) | Warning |
| `board-rendering.md` | C-W-NEW-2 (OUTNUMBERED), C-W-NEW-8 (reveal-tween cross-reference) | Warning |
| `card-animations.md` | C-W-NEW-6 (overlay duration OQs), C-W-NEW-8 (reveal tween catalog gap) | Warning |
| `server-rng.md` | C-W9 (GSS lifecycle), C-W-NEW-7 (keyword/class RNG slots unregistered) | Warning |
| `prism-system.md` | D-W-NEW-4 (WALL farming OQ needed) | Warning |
| `shop-auction-ui.md` | C-W-NEW-6 (overlay duration), C-W-NEW-3 (DRAFT_INITIAL ownership) | Warning |

---

## Verdict: **FAIL**

9 blocking issues must be resolved before architecture begins:

| # | Issue | GDDs to Update |
|---|---|---|
| C-B1 | Add 3 missing `S2CResolutionEvent` variant families | `network-protocol.md` |
| C-B2 | Resolve `GameOverReason::ResolutionTimeout` vs `Draw` — update KS to use `Draw` | `keyword-system.md` |
| C-B3 | Unify `S2CCardAcquired` to single enum name (`AcquisitionSource`) with complete variant set | `network-protocol.md`, `entities.yaml` |
| C-B4 | Remove `S2CCardAcquired` from Hand UI Rule 5c activation-lock resolver | `hand-ui.md` |
| C-B5 | Lock `activate_timeout_ms` / `purchase_timeout_ms` as client-only constants or add to game-config.md | `hand-ui.md` |
| C-B6 | Register `S2CSangMepriseReveal` in `entities.yaml` | `entities.yaml` |
| D-B1 | Lock Garde-Temps routing to `take_damage()` + add per-game use limit | `class-system.md`, `objective-system.md` |
| D-B2 | Define multi-Krosmic sub-step 1 ordering rule (or move Punition to sub-step 3+) | `class-system.md`, `round-state-machine.md` |
| D-B3 | Commit to auction wealth-gap mitigation (Option A: free-bid floor recommended) | `auction-system.md`, `economy-system.md` |
