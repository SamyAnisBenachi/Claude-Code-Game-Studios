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
| 1 | Card Data & Pool | `gdd/card-data-pool.md` | Needs Revision | /review-all-gdds 2026-04-29 R2: C-B4 still open — no §3.4 auto-refresh policy (who calls refresh_shop, what happens to pool state). Must resolve before architecture. |
| 2 | Game Config | `gdd/game-config.md` | Needs Revision | /review-all-gdds 2026-04-29 R2: N-B1 — refresh_base_cost referenced in economy-system.md but not in struct, Tuning Knobs, or ACs. Add field before architecture. |
| 3 | Server-side RNG | `gdd/server-rng.md` | Approved | /design-review 2026-04-29. 8 blockers resolved: Rule 5 rewritten as 3 per-phase chains (DRAFT_INITIAL/RESOLUTION/SHOP), fake-objective initial assignment added to seed table (2 seeds/player), shop slot seed count corrected (2–3/slot), Rule 6 inter-player ordering added (ascending player_id→lane→position), Formula 1b added for CDF weighted selection, Formula 1 preconditions added, AuditEntry type corrected to Option<String> with encoding table, 4 invalid ACs removed and 7 new ACs added (RNG11–15). |
| 4 | Economy System | `gdd/economy-system.md` | Needs Revision | /review-all-gdds 2026-04-29 R2: C-B5a — interest formula still hardcodes /5, must use GameConfig.interest_threshold_gold; B-2 — Garde-Temps costs 20 reserve but reserve_mana_cap=10 makes it permanently unplayable (design decision needed); B-3 — OQ1 resolution allows free Legendary pick, undermining Auction as signature (cap at Rare/Epic). |
| 5 | Board / Lane System | `gdd/board-lane-system.md` | Approved | /design-review 2026-04-28. All 14 blocking items resolved: player fantasy rewritten (forensic prediction, not live read), prism respawn confirmed per-player independent (OQ1 closed), F1/F2/F3 formula type safety + direction tables fixed, pending buffer architecture specified (Resource not entities), S2CPlacementReveal + replay log specified, 12 ACs rewritten + 5 new ACs (BL-27b/30/31/32/33). WALL farming and global spawn range kept by design. |
| 6 | Round State Machine | `gdd/round-state-machine.md` | Approved | /design-review 2026-04-29. 13 blockers resolved: disconnect grace 5s→30s, RESOLUTION safety timeout added (60s→Draw), Rule 5/F2 shop refresh inconsistency fixed, StartAuction added to F2 sequence, S2CGameOver+GameOverReason enum defined, ready signal now retractable, RSM-31 BLOCKING, RSM-32–38 added. 4 open questions remain (lobby_timeout, DRAFT_INITIAL gold forfeiture, multiplayer auction, late-joiner sync). |
| 7 | Network Protocol | `gdd/network-protocol.md` | Not Started | Lightyear 0.26 message types (C2S/S2C), replication components, channel definitions |
| 8 | Game Session System | `gdd/game-session-system.md` | Not Started | Room creation, player join, class selection, mode (1v1/2v2/…), reconnection grace period |
| 9 | Objective System | `gdd/objective-system.md` | Not Started | 5 objectives (3 real + 2 fake); HP tracking; loss condition (2 REAL destroyed); fake rewards |

### M2 — Playable Game
*Goal: Complete 1v1 game with auction, combat, shop, and win condition — visually playable.*

| # | System | File | Status | Notes |
|---|---|---|---|---|
| 10 | Card Acquisition | `gdd/card-acquisition.md` | Not Started | Personal shop (3 cards/round, TFT weighting), initial draft (9 cards/5g), hand management |
| 11 | Auction System | `gdd/auction-system.md` | Not Started | **Signature mechanic** — open ascending, visible price/leader, +5s timer reset, lightyear sync |
| 12 | Combat Resolution | `gdd/combat-resolution.md` | Not Started | Simultaneous reveal, sub-step order, damage formula, objective damage, RPS type advantage |
| 13 | Board Rendering | `gdd/board-rendering.md` | Not Started | Visual 5-lane grid, unit sprites at cells, health bars, Ankama art style |
| 14 | Hand UI | `gdd/hand-ui.md` | Not Started | Card fan display, card selection, play confirmation during PLACEMENT |
| 15 | Shop / Auction UI | `gdd/shop-auction-ui.md` | Not Started | 3-card shop slots; auction panel with live price/timer/leader display |
| 16 | HUD | `gdd/hud.md` | Not Started | Gold, mana, reserve display; round number; objective status dots (5 per side) |

### M3 — Full Feature
*Goal: All GDD mechanics working — keywords, prisms, class rules, and animations.*

| # | System | File | Status | Notes |
|---|---|---|---|---|
| 17 | Keyword System | `gdd/keyword-system.md` | Not Started | All triggers: APPEARANCE, DEATH, FINAL BLOW, INJURED, START/END OF TURN; all movement/combat keywords |
| 18 | Prism System | `gdd/prism-system.md` | Not Started | 5 prisms; Lane 1/5 → obj-damage spell; Lane 2/4 → reserve spell card; Lane 3 → draw; respawn |
| 19 | Class System | `gdd/class-system.md` | Not Started | Xelor reserve spells (Gelure, Rollback, etc.); other class-specific rules; interaction matrix |
| 20 | Card Animations | `gdd/card-animations.md` | Not Started | bevy_tweening for card draw, play, unit movement per round, combat resolution reveal |

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
