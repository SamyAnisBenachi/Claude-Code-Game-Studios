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
| 6 | Round State Machine | `gdd/round-state-machine.md` | Approved | /design-review 2026-04-29. 13 blockers resolved: disconnect grace 5s→30s, RESOLUTION safety timeout added (60s→Draw), Rule 5/F2 shop refresh inconsistency fixed, StartAuction added to F2 sequence, S2CGameOver+GameOverReason enum defined, ready signal now retractable, RSM-31 BLOCKING, RSM-32–38 added. 4 open questions remain (lobby_timeout, DRAFT_INITIAL gold forfeiture, multiplayer auction, late-joiner sync). |
| 7 | Network Protocol | `gdd/network-protocol.md` | Approved | /design-review R3 2026-04-29: APPROVED. Fixed inline: timer_duration_secs/new_timer_secs/grace_remaining_secs → _ms throughout (NP + RSM + entities.yaml). 29 ACs (28 BLOCKING). Open: Lightyear 0.26 unicast API, component visibility filtering, reliable channel ordering (OQ1–3). |
| 8 | Game Session System | `gdd/game-session-system.md` | Approved | /design-review R3 2026-04-29: APPROVED. Fixed inline: game-config.md GSS row corrected (disconnect_grace_seconds removed; lobby_heartbeat_timeout_seconds noted pending); transitions table: "RSM fires SessionReady" → "GSS fires SessionReady"; ServerRng init failure → LOBBY_CANCELLED row added. 41 ACs. All modes in scope (1v1, 2v2, 3v3). lobby_heartbeat_timeout_seconds added to game-config.md + entities.yaml (OQ8 resolved). ADR needed for SessionReady Observer vs Events<T> (OQ7). |
| 9 | Objective System | `gdd/objective-system.md` | Approved | /design-review R3 2026-04-29: APPROVED. Fixed inline: game-config.md fake_count upper bound tightened ≤4→≤3 + GC8c AC added; OS-17 conditional language cleaned up (ADR-001 resolved). XL scope — 9 cross-system deps, 5 formulas. Sang Méprise reconnect gap forwarded to NP backlog (active_sang_meprise_identities missing from S2CGameSnapshot). |

### M2 — Playable Game
*Goal: Complete 1v1 game with auction, combat, shop, and win condition — visually playable.*

| # | System | File | Status | Notes |
|---|---|---|---|---|
| 10 | Card Acquisition | `gdd/card-acquisition.md` | Approved | /design-review 2026-04-29: 6 blockers resolved. Single-fire auction refresh (DRAFT_AUCTION entry only; same slots persist to DRAFT_SHOP). RSM Rule 12 gold carry-over confirmed. ERR_WRONG_PHASE → silent discard (NP wins). CA18 rollback: refund_gold() specified. shop_weight_per_card design note (raise to 0.20–0.25). 22 ACs (CA1–CA22). refresh_cap added to game-config.md. S2CShopSlots/S2CDraftOffering registered in NP GDD. OQs 1–2 resolved. OQs 3–5 open. |
| 11 | Auction System | `gdd/auction-system.md` | In Review | **Signature mechanic** — /design-review 2026-04-29 (pass 2): NEEDS REVISION → revised in-session (14 blockers resolved). S2CGoldBroadcast updated (reserved_gold added to NP GDD); legendary_pool_entry_round + auction_max_duration_seconds registered in game-config.md; bid input changed to preset-buttons primary; rarity accessibility text labels required; 3 new ACs (AU20, AU8 integration note, AU19 split note); OQ7 Pillar Risk callout added (M3 deadline); OQ8 first-bid bonus stricken; OQ2/OQ3 closed. Re-review recommended in fresh session. |
| 12 | Combat Resolution | `gdd/combat-resolution.md` | In Review | /design-review 2026-04-29: MAJOR REVISION NEEDED → revised in-session (25 blockers addressed). Key fixes: two-pass AR_attacker_combat algorithm + worked example, u8/i32 arithmetic spec, CR-26 fixed (sub-step 4), OQ4 ✓ (COUNTERATTACK fires for collision-halt adjacency), OQ2 ✓ (type advantage → GameConfig), sub-step 2 STUN check added, sub-step 5 tick loop formalized, SHIELD pre-check specified, gold authority batch-only, 11 new ACs (CR-35–CR-45), CR-31–34 reclassified BLOCKING. Remaining: OQ1 (WALL ADR), OQ3 (RANGE RNG seed), OQ5 (NP GDD needs CombatDamage/KeywordTriggered variants). Re-review after NP GDD updates. |
| 13 | Board Rendering | `gdd/board-rendering.md` | In Review | /design-review 2026-04-30 MAJOR REVISION → 24 blockers resolved in-session. Player Fantasy reframed (watching IS reading); animation defaults cut (5s default / 8.5s ceiling); AnimGroup/AnimQueue Data Structures subsection added; Rule 10/11 buffer logic fixed (PhaseChanged buffered in any RESOLUTION sub-state, reconnect always→DraftShop); ADR-001 reconnect path with S2CObjectiveIdentities re-send + Reconnecting sub-state; F1/F2/F3 preconditions + release-loud assertions; Bevy 0.18 API contract subsection (despawn, ChildOf, MessageReader, q.single() Result, custom SpriteAlphaLens, set_tweenable, local-Z); BR-3 split into BR-3a/BR-3b BLOCKING proxies; BR-7/BR-20 reclassified BLOCKING with Time<Virtual> injection; BR-19 strengthened (ComponentId set equality); 14 new ACs (BR-2b, 3a, 3b, 22b, Z-LOCAL, EC-EARLY, CARDMISS, OBJMISS, STUCK, 2-ATLAS, FOG-OPACITY, FRAME-TIME, RECONNECT-TIME, FIRST-LOOK); BR-25 moved to CI lint rule; sub_step OOR aligned with NP (fatal desync + RequestSnapshot). 10 OQs (5 carryover with recommended resolutions; 5 new: OQ-BR-06 RequestSnapshot, OQ-BR-07 trap NP-OQ-2, OQ-BR-08 Visual Identity Anchor, OQ-BR-09 dud-thud audio gate, OQ-BR-10 HP poll vs tween). Re-review recommended in fresh session. |
| 14 | Hand UI | `gdd/hand-ui.md` | Approved | /design-review 2026-04-30: NEEDS REVISION → revised in-session (4 P0 + 6 P1 blockers resolved). GhostPlacementChanged payload extended to `target: Option<PlayTarget>` (BoardCell/TargetUnit/TargetObj/LaneWide/Instant variants); GhostClickedEvent + GhostDragStartEvent reverse interfaces added (Board Rendering → Hand UI); Reserve Mana Split spec'd in Rule 13 + VA-9 + 3 ACs; un-stage matrix complete (board ghost click / drag-back / Instant fan ghost); Rule 10 client-side pre-validation with inline error label; Formula 1 count=2 clamp bug fixed + Y-direction corrected (bevy_ui screen-space) + count=0 guard; GRID→PASSIVE state transition added; Rule 9 auto-stage on timer expiry (anti-pillar fix); 8 FAIL ACs rewritten (state vs visual convention); 9 new ACs (HU-02b/03b/12b/12c/12d/15b/17b/17c/21b/21c/25/26/27/28/29/30); OQ1/OQ2/OQ4 closed; OQ6/OQ7 opened (atlas sharing, zoom resolution). Board Rendering GDD updated (Rule 8, ACs, Interactions). |
| 15 | Shop / Auction UI | `gdd/shop-auction-ui.md` | In Review | /design-review 2026-04-30 (pass 1): NEEDS REVISION → revised in-session (11 BLOCKING items resolved). Key fixes: Bevy 0.18 API errors (Style.width → Node, Tween<Transform> → custom Lens, stale text-input bullet); cross-type FIFO buffer-as-primary path (NP OQ3-independent); same-tick bid+settle race AC; C2SSignalReady DRAFT_INITIAL Ready button added (matches RSM); border temperature ramp promoted to Detailed Design + Formula D.6; refresh disable-on-click explicit; DRAFT_AUCTION prominence tiers (PRIMARY/SECONDARY/TERTIARY); "BIDDING..." in-flight label; Player Fantasy scoped down (gold-as-tell mechanic gap honest). 12 new ACs (SAU-EG3/EG4, DI7/8/9, DA11/12, DS8, F16-19). 3 new OQs (OQ6 NP FIFO informational, OQ7 bidirectional updates, OQ8 buffer timing). Re-review recommended in fresh session. |
| 16 | HUD | `gdd/hud.md` | Designed | Persistent screen-edge readout: gold/mana/reserve (local + opponent), round + phase label, 10 objective dots (alive/destroyed, real/fake hidden). 21 ACs (18 BLOCKING). 3 OQs: disconnect-pause message (NP/GSS), local real/fake opt-in (M3), GAME_OVER summary screen (M3). /design-review pending (fresh session). UX spec needed: /ux-design hud. |

### M3 — Full Feature
*Goal: All GDD mechanics working — keywords, prisms, class rules, and animations.*

| # | System | File | Status | Notes |
|---|---|---|---|---|
| 17 | Keyword System | `gdd/keyword-system.md` | Approved | /design-review 2026-04-30: NEEDS REVISION → revised in-session. 11 blockers resolved: KW-024 SHIELD logic fixed; OQ-KS1 split into 3 named seed slots (range_equidistant_select, teleport_random_dest, strich_change_lane_select); new Replication Contract subsection (7 persistent states with replication paths + reconnect recovery); OQ-NP1 expanded with attacker_id + was_blocked; KW-035 rewritten to component-state-at-boundary; KW-036–KW-040 promoted ADVISORY→BLOCKING; 11 ACs split (KW-004/008/009/010/015/020/027/029/031/033/035); REPEL i32 intermediate note; Dangerous Combinations table; COUNTERATTACK tooltip note; HASTE design-lever note. New OQs: NP2 (ResolutionTimeout), NP3 (DEATH chain order), NP4 (UnitBoardState fields), NP5 (KeywordTriggered variant), KS5 (OUTNUMBERED visual). 41 ACs → ~55 after splits. |
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
