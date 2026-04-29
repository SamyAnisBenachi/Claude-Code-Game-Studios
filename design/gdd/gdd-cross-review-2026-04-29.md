# Cross-GDD Review Report

**Date:** 2026-04-29 (Revision 2 — post game-config.md revision)
**GDDs Reviewed:** 7 (6 system GDDs + master GDD)
**Systems Covered:** Card Data & Pool, Game Config, Server-side RNG, Economy System, Board/Lane System, Round State Machine, Master GDD
**Registry baseline:** entities.yaml v1

**Progress vs. prior review (2026-04-29 R1):** 6 of 10 prior blockers resolved (C-B1 ✓, C-B2 ✓, C-B3 ✓, D-B2 ✓, D-B3 ✓, D-B4 ✓). 3 new blockers surfaced from resolution decisions. Net: 10 → 7 blockers.

---

## Consistency Issues

### Blocking (must resolve before architecture begins)

🔴 **C-B4 — Auto-refresh policy undefined in card-data-pool.md** *(carried from R1)*
`round-state-machine.md` Rule 5 fires `refresh_shop(player)` on every DRAFT phase entry. `card-data-pool.md` has no definition of what `refresh_shop()` does to pool state — no §3.4, no named operation, no policy on whether unselected cards return to the pool or are discarded. A programmer implementing Card Data & Pool cannot resolve this without guessing.
→ Fix: add Rule 3.4 defining: (a) auto-refresh discards the current 3-slot offering; (b) does NOT call `distribute()` — unselected cards disappear without affecting pool copy counts; (c) a new server seed generates fresh slots. Update RSM interaction row.

🔴 **C-B5 cluster — Interest formula hardcodes `/5` in 3 locations despite `interest_threshold_gold` added to GameConfig**
`game-config.md` added `interest_threshold_gold: u32` (default 5) today. Three documents still hardcode the literal `5`:
- `economy-system.md` Formula 2 (line ~211): `min(floor(g / 5), interest_max_bonus)` + "hardcoded" note at line ~338
- `lanes-and-lies-gdd.md` §7 Tuning Knobs (line 749): `interest_per_5g | 1 | 1 | Do not change...` — stale confusing row
- `entities.yaml` interest formula (line 135): expression `"min(floor(g / 5), interest_max_bonus)"` + hardcoded notes
→ Fix all three: update formula expression to `/ GameConfig.interest_threshold_gold`, remove "hardcoded" notes, update registry variables list and expression, replace master GDD row with `interest_threshold_gold | 5 | 3–10`.

🔴 **C-B6 — `S2CGameOver` and `GameOverReason` unregistered; master GDD has no reference** *(carried from R1)*
`round-state-machine.md` Rule 14 defines `S2CGameOver { loser, round, reason }` and `GameOverReason` enum. Neither exists in `entities.yaml`. `lanes-and-lies-gdd.md` §8 ACs C6/C7 reference the win condition but never name the network message types.
→ Fix: add `S2CGameOver` and `GameOverReason` to registry; add cross-reference in master GDD §8 C7.

🔴 **N-B1 (NEW) — `refresh_base_cost` referenced in economy-system.md but missing from game-config.md**
`economy-system.md` lines ~148, ~311, ~336, ~342 reference `GameConfig.refresh_base_cost` (default 1g, escalating refresh costs). `economy-system.md` line ~342 explicitly states "GameConfig updates required: `reserve_mana_cap` and `refresh_base_cost`…" — `reserve_mana_cap` was added today, `refresh_base_cost` was not. The struct, Tuning Knobs, Interactions table, and ACs GC1/GCN-DEFAULTS all lack this field.
→ Fix: add `pub refresh_base_cost: u32` to game-config.md struct (Economy section, default 1, safe range 1–3), add Tuning Knobs row, update Interactions/Dependencies tables, update ACs GC1 fixture and GCN-DEFAULTS.

🔴 **N-B2 (NEW) — Master GDD §3.9 Xelor and §4.1 still say "no reserve cap" — contradicts `reserve_mana_cap` addition**
`lanes-and-lies-gdd.md` line 377 (§3.9): "Reserve has no stated maximum cap for Xelor (balance tuning TBD)". Line 518 (§4.1): "// No maximum cap on reserve (balance TBD)". Both contradict `economy-system.md` Rule 3 which states `reserve_mana_cap = 10` universally. Implementers reading the master GDD would implement Xelor with uncapped reserve.
→ Fix: update §3.9 line 377 and §4.1 line 518 to state the universal `GameConfig.reserve_mana_cap` cap.

---

### Warnings

⚠️ **C-W1 — economy-system.md Interactions table missing `interest_threshold_gold` and `reserve_mana_cap`**
Interactions table (line ~173) and Dependencies table (line ~311) both omit `interest_threshold_gold`. The Interactions table also omits `reserve_mana_cap`. The two tables within economy-system.md are inconsistent with each other and with game-config.md.

⚠️ **C-W2 — Registry `epic_pool_copies` and `legendary_pool_copies` still list `game-config.md` in `referenced_by`**
Both are now Rust consts, not config fields. Registry `referenced_by` for both should remove `game-config.md` and add a note explaining they are now hardcoded constants.

⚠️ **C-W3 — Registry `mana_ramp` output_range hardcodes `[1, 12]`**
`game-config.md` safe range for `mana_cap` is now 6–14. If `mana_cap` is configured above 12, the registry output_range is stale. Should be `[1, mana_cap]` with symbolic notation.

⚠️ **C-W4 — economy-system.md Tuning Knobs note "interest_per_5g hardcoded" directly contradicts today's addition**
Part of C-B5 fix but listed separately for traceability.

---

## Game Design Issues

### Blocking

🔴 **D-B1 — Fake-first is still strictly dominant over all strategies** *(carried from R1)*
No changes to §3.3 objective rewards or §4.7 damage formula. Fake destruction gives +3 gold + global spawn range expansion + 50/50 bonus; real destruction gives +3 gold only. Spawn range expansion is a permanent throughput multiplier — units placed from Cell 3 reach objectives ~3 rounds faster than from Cell 1. There is no round count at which real-first beats fake-first. Violates "Deep emergence" pillar.
→ Fix: decouple spawn expansion from fake destruction (tie to kills/time/alternate axis), OR give real objective destruction a competing reward. Design decision required.

🔴 **B-2 (NEW) — Garde-Temps is permanently unplayable at `reserve_mana_cap = 10`**
`lanes-and-lies-gdd.md` §3.9: "Garde-Temps (20 reserve mana!) — destroys a dofus." `economy-system.md` Rule 3: `reserve_mana_cap = 10`. A player can never hold more than 10 reserve; Garde-Temps costs 20. This card can never legally be played at default configuration. Introduced by today's D-B2 fix. Garde-Temps defines Xelor's late-game identity — rendering it permanently unplayable undermines the class pillar.
→ Design decision required: (a) raise `reserve_mana_cap` above 20 and find another snowball control mechanism, OR (b) reduce Garde-Temps cost to ≤ 10, OR (c) make Garde-Temps draw from combined current + reserve, OR (d) add per-card "from reserve" mechanics separate from the cap. Must resolve before Xelor class architecture is designed.

🔴 **B-3 (NEW) — Free card pick from fake destruction can yield a Legendary, bypassing "Auction as signature"**
`economy-system.md` OQ1 (Resolved 2026-04-29): "Free card pick draws from the shared auction pool. Any rarity may be drawn, subject to pool availability." There is 1 Legendary copy per pool. It appears only at auction. The free card pick fires at DRAFT entry (before the auction round fires) — confirmed by RSM/Economy ordering. A player who destroys a fake before the auction round can draw the Legendary for free, bypassing the 5g+ bid and the entire auction drama. This directly violates "Auction as signature."
→ Fix: cap free card pick at Rare or Epic. Exclude Legendaries from the free pick rarity pool. Update `economy-system.md` OQ1 resolution and relevant GDDs.

---

### Warnings

⚠️ **D-W1 — Manual shop refresh escalation reduces but does not eliminate the Rare-without-auction loop** *(carried from R1)*
Two refreshes (3g total) + purchase (3g) = 6g guaranteed targeted Rare. Auction cost for the same Rare may exceed 6g in competitive bidding. Shop path still has positive EV over auction for Rares. Auction identity is strongest for Legendaries; weaker for Rares/Epics. Monitor in playtesting; cap manual refreshes at 2 per DRAFT if this proves dominant.

⚠️ **D-W2 — RESOLUTION phase has zero player decisions** *(carried from R1)*
RSM Rule 10: "No player input accepted." Violates "No idle spectating" for the duration of RESOLUTION. Low severity at short RESOLUTION duration; becomes serious above 10s animation time. Consider at least one optional micro-decision during RESOLUTION (reaction card, keyword activation between sub-steps).

⚠️ **D-W3 (NEW) — `interest_threshold_gold = 3` removes the miser/gambler tension**
At threshold=3: starting gold (5g) already exceeds the maximum-interest bracket (6g) by round 2. Holding gold becomes always-optimal from round 2 onward — the core economic decision evaporates. Safe range minimum of 3 is too permissive.
→ Fix: update `game-config.md` safe range for `interest_threshold_gold` from `3–10` to `5–10` (default is already 5).

⚠️ **D-W4 (NEW) — Reserve cap creates asymmetric Xelor experience; compounds with B-2**
Non-Xelor classes reach `reserve_mana_cap = 10` slowly (10+ prism cycles). Xelor reaches it in 2–3 rounds via Gelure, then repeatedly wastes excess transfers. The cap is invisible to non-Xelor and a constant ceiling Xelor bumps against. With Garde-Temps now unplayable (B-2), Xelor's late-game reserve accumulation serves no purpose beyond card cost supplementing. Both B-2 and D-W4 must be resolved together as a design unit.

⚠️ **D-W5 (NEW) — 8 simultaneous active tracking requirements during 10-second PLACEMENT timer**
Players track simultaneously: (1) gold budget, (2) mana budget, (3) reserve balance, (4) own 5 objectives, (5) spawn range, (6) opponent gold, (7) 5-lane board state, (8) opponent card prediction. 8 concurrent active requirements exceeds the comfortable 4–6 range for skilled players, especially under 10-second timer pressure. Risk: players make errors from overwhelm rather than the intended "I read them" skill expression. Mitigate with UI surfacing of object-level information.

---

## Cross-System Scenario Issues

**Scenarios walked:** 3

🔴 **Garde-Temps acquisition scenario** — Systems: Xelor Class, Economy
Player buys Garde-Temps (4g, Xelor shop). Spends rounds accumulating reserve via Gelure + Miss Nuit. At round 10: reserve = 10 (at cap). Attempts to play Garde-Temps (cost: 20 reserve). Server validates `10 >= 20` → FALSE. Card play rejected. Player holds an unplayable signature card for the rest of the game with no recourse. Scenario is reachable in normal 1v1 gameplay. GDD documents no warning that the card may be unplayable.

ℹ️ **interest_threshold_gold config mismatch** — Systems: Economy, GameConfig
Config set to `interest_threshold_gold = 4`. Economy formula still hardcodes `/5`. Config change has zero gameplay effect. Feature silently broken at any non-default value until C-B5 is resolved.

ℹ️ **Free pick + auction ordering confirmation** — Systems: Economy (OQ1), Card Data & Pool, RSM
Free card pick bonus fires at DRAFT_INITIAL/DRAFT_SHOP entry (from fake destruction resolved during prior RESOLUTION). Auction fires during DRAFT_AUCTION phase. Since DRAFT_INITIAL/SHOP precede DRAFT_AUCTION in phase ordering, the free pick draws from the pool before the auction card is offered. This confirms B-3: if the Legendary was in the pool, the free pick can claim it before the auction round fires.

---

## GDDs Flagged for Revision

| GDD | Reason | Type | Priority |
|---|---|---|---|
| `game-config.md` | Missing `refresh_base_cost` field (N-B1) | Consistency | Blocking |
| `economy-system.md` | Interest formula hardcodes `/5` (C-B5a); Garde-Temps/reserve conflict (B-2); free pick Legendary exclusion (B-3); Interactions table gaps (C-W1) | Consistency + Design | Blocking |
| `card-data-pool.md` | `refresh_shop()` auto-refresh policy missing (C-B4) | Consistency | Blocking |
| `lanes-and-lies-gdd.md` | `interest_per_5g` tuning row (C-B5b); §3.9/§4.1 reserve cap claims (N-B2); S2CGameOver reference (C-B6) | Consistency | Blocking |
| `entities.yaml` | Interest formula expression (C-B5c); S2CGameOver/GameOverReason entries (C-B6); epic/legendary referenced_by (C-W2); mana_ramp output_range (C-W3) | Consistency | Blocking/Warning |

---

## Verdict: FAIL

7 blocking issues prevent architecture from starting.

**Root cause clusters:**

**Cluster 1 — Incomplete propagation of game-config.md revision (N-B1, C-B5 cluster, N-B2):** `refresh_base_cost` was in the economy GDD's pending list but missed in today's revision. `interest_threshold_gold` was added to config but not propagated to economy formula, master GDD, or registry. `reserve_mana_cap` addition not reflected in master GDD reserve claims.

**Cluster 2 — Reserve cap creates new design conflicts (B-2, D-W4):** Fixing D-B2 (snowball) by capping reserve at 10 made Garde-Temps (cost 20) permanently unplayable. The fix must be revisited as a design unit.

**Cluster 3 — Unresolved carry-overs (C-B4, C-B6, D-B1, B-3):** Shop auto-refresh policy, S2CGameOver registry, fake dominant strategy, and Legendary free-pick bypass are unchanged or newly identified.

### Required actions before re-running /review-all-gdds

1. **C-B4**: Add `refresh_shop()` auto-refresh policy to `card-data-pool.md`
2. **C-B5**: Update interest formula in `economy-system.md`, `lanes-and-lies-gdd.md` §7, and `entities.yaml`
3. **C-B6**: Register `S2CGameOver` + `GameOverReason` in `entities.yaml`; add master GDD stub
4. **N-B1**: Add `refresh_base_cost` to `game-config.md` struct, Tuning Knobs, and ACs
5. **N-B2**: Update master GDD §3.9 and §4.1 reserve cap claims
6. **D-B1**: Design decision — reduce fake vs. real objective reward asymmetry
7. **B-2**: Design decision — resolve Garde-Temps (cost 20) vs. reserve_mana_cap (cap 10) conflict
8. **B-3**: Cap free card pick rarity at Rare/Epic; exclude Legendaries from free pick pool
