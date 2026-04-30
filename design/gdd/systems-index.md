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
| 7 | Network Protocol | `gdd/network-protocol.md` | Approved | /design-review R5 lean 2026-04-30: APPROVED. 2 blockers resolved: Rule 8 C2SHeartbeat "reliable channel" contradiction fixed (→ unreliable, see Rule 2); SeedBoardState owner: PlayerId added (consistency with TrapBoardState/SinistroState). Remaining recommended: BodyguardBondBroken AC missing; AR cap undocumented; activate_timeout_ms not in config. Cross-GDD open: keyword-system.md silenced_until_round u8→u32; board-rendering.md OQ-BR-06 gate should be unlocked. 3 HIGH Lightyear OQs (OQ-1/2/3) and 2 MEDIUM OQs (OQ-6/7) remain open — network programmer verification required before implementation. |
| 8 | Game Session System | `gdd/game-session-system.md` | Approved | /design-review R3 2026-04-29: APPROVED. Fixed inline: game-config.md GSS row corrected (disconnect_grace_seconds removed; lobby_heartbeat_timeout_seconds noted pending); transitions table: "RSM fires SessionReady" → "GSS fires SessionReady"; ServerRng init failure → LOBBY_CANCELLED row added. 41 ACs. All modes in scope (1v1, 2v2, 3v3). lobby_heartbeat_timeout_seconds added to game-config.md + entities.yaml (OQ8 resolved). ADR needed for SessionReady Observer vs Events<T> (OQ7). |
| 9 | Objective System | `gdd/objective-system.md` | Approved | /design-review R3 2026-04-29: APPROVED. Fixed inline: game-config.md fake_count upper bound tightened ≤4→≤3 + GC8c AC added; OS-17 conditional language cleaned up (ADR-001 resolved). XL scope — 9 cross-system deps, 5 formulas. Sang Méprise reconnect gap forwarded to NP backlog (active_sang_meprise_identities missing from S2CGameSnapshot). |

### M2 — Playable Game
*Goal: Complete 1v1 game with auction, combat, shop, and win condition — visually playable.*

| # | System | File | Status | Notes |
|---|---|---|---|---|
| 10 | Card Acquisition | `gdd/card-acquisition.md` | Approved | /design-review 2026-04-29: 6 blockers resolved. Single-fire auction refresh (DRAFT_AUCTION entry only; same slots persist to DRAFT_SHOP). RSM Rule 12 gold carry-over confirmed. ERR_WRONG_PHASE → silent discard (NP wins). CA18 rollback: refund_gold() specified. shop_weight_per_card design note (raise to 0.20–0.25). 22 ACs (CA1–CA22). refresh_cap added to game-config.md. S2CShopSlots/S2CDraftOffering registered in NP GDD. OQs 1–2 resolved. OQs 3–5 open. |
| 11 | Auction System | `gdd/auction-system.md` | Approved | **Signature mechanic** — /design-review 2026-04-30 (pass 5, lean): APPROVED. All 11 pass-4 blockers confirmed resolved. Pre-implementation gates: (1) reconcile CardSource::AuctionWon vs AcquisitionSource enum name in NP GDD before story impl; (2) update RSM GDD for auction_max_duration_seconds AbortAuction trigger; (3) resolve OQ9 (AuctionExpired reachability) before AU12; (4) verify spend_reserved_gold API name (OQ4). Recommended: migrate AU11 to Economy System GDD; promote CardSource naming to OQ10; close OQ6 cross-GDD update. 33 ACs (BLOCKING + ADVISORY). Scope: L. |
| 12 | Combat Resolution | `gdd/combat-resolution.md` | In Review | /design-review 2026-04-29: MAJOR REVISION NEEDED → revised in-session (25 blockers addressed). Key fixes: two-pass AR_attacker_combat algorithm + worked example, u8/i32 arithmetic spec, CR-26 fixed (sub-step 4), OQ4 ✓ (COUNTERATTACK fires for collision-halt adjacency), OQ2 ✓ (type advantage → GameConfig), sub-step 2 STUN check added, sub-step 5 tick loop formalized, SHIELD pre-check specified, gold authority batch-only, 11 new ACs (CR-35–CR-45), CR-31–34 reclassified BLOCKING. Remaining: OQ1 (WALL ADR), OQ3 (RANGE RNG seed), OQ5 (NP GDD needs CombatDamage/KeywordTriggered variants). Re-review after NP GDD updates. |
| 13 | Board Rendering | `gdd/board-rendering.md` | In Review | /design-review R3 2026-04-30: NEEDS REVISION → revised in-session. 14 blockers resolved: game-config.md synced (fog fields removed, 5 reveal-tween fields added); AnimQueue E0201 compile error fixed (field renamed `total_duration_ms_cached`); Rule 1 amended (C2SRequestSnapshot exception); F4 ceiling corrected (12.6s, not 11.5s); Player Fantasy ceiling raised to 12.6s; Rule 7 rewritten to collect-then-reveal buffer (1-frame delay guarantees simultaneous tween start regardless of Lightyear replication batching); PendingResolutionScript inverse-stuck EC added (EC-PLACEMENT-STUCK + GATED AC); HP bar write-conflict invariant added to Rule 6 (no Animator<Transform> on fill axis); draw-call breakdown table added to Rule 5 (worst-case 12–17 calls; ceiling may be exceeded under tinting); `UNIT_SPRITE_WIDTH` added to Internal Constants; OQ-BR-04 RESOLVED (replicated SpawnRange component); BR-7 rewritten (collect-then-reveal, no "or equivalent"); BR-17 apply_deferred requirement explicit; BR-19 poison-entity step added; BR-SYSTEMSET-ORDER app.update() pre-run required; BR-18c/BR-EC-STUCK OQ-BR-06 gates labeled inline; BR-HP-INVARIANT AC added. Remaining: OQ-BR-06 (C2SRequestSnapshot not in NP GDD — gates 4 ACs). Re-review recommended after OQ-BR-06 lands. |
| 14 | Hand UI | `gdd/hand-ui.md` | Needs Revision | /design-review 2026-04-30: NEEDS REVISION → revised in-session (4 P0 + 6 P1 blockers resolved). GhostPlacementChanged payload extended to `target: Option<PlayTarget>` (BoardCell/TargetUnit/TargetObj/LaneWide/Instant variants); GhostClickedEvent + GhostDragStartEvent reverse interfaces added (Board Rendering → Hand UI); Reserve Mana Split spec'd in Rule 13 + VA-9 + 3 ACs; un-stage matrix complete (board ghost click / drag-back / Instant fan ghost); Rule 10 client-side pre-validation with inline error label; Formula 1 count=2 clamp bug fixed + Y-direction corrected (bevy_ui screen-space) + count=0 guard; GRID→PASSIVE state transition added; Rule 9 auto-stage on timer expiry (anti-pillar fix); 8 FAIL ACs rewritten (state vs visual convention); 9 new ACs (HU-02b/03b/12b/12c/12d/15b/17b/17c/21b/21c/25/26/27/28/29/30); OQ1/OQ2/OQ4 closed; OQ6/OQ7 opened (atlas sharing, zoom resolution). Board Rendering GDD updated (Rule 8, ACs, Interactions). |
| 15 | Shop / Auction UI | `gdd/shop-auction-ui.md` | In Review | /design-review 2026-04-30 (pass 3): MAJOR REVISION NEEDED → revised in-session (28 BLOCKING items resolved). Key fixes — Axis 1 (cross-GDD reconciliation): leading-state buttons HIDDEN (not disabled, matches auction-system.md); bid labels → total-commitment-primary ("8g (+1)"); Player Fantasy updated to match preset system; S2CGoldUpdate payload corrected (carries reserved_gold per NP GDD line 107). Axis 2 (protocol): Rule 6 two-message gate made symmetric (pending_bid_accepted + pending_gold_broadcast_seen flags handle both arrival orderings); player_id==local_player filter added; Rule 9 now discards post-settlement S2CAuctionBidAccepted. Axis 3 (Bevy 0.18): BorderRadius → Node field (not separate component); PositionType::Absolute required for AuctionPanel slide-down; Overflow::Clip required for ShopPanel expand-up; TweenCompleted mechanism specified for DRAFT_SHOP timer start. Other: DRAFT_INITIAL dual-message buffer rule added; state machine hidden sub-state documented; Formula D.3 boundary note corrected (f32 >10000); Tuning Knobs auction_timer_reset_seconds description fixed; Formula D.6 tier 0–3g changed to Pale Ink Blue #2A4D8A (visible border); footer card costs specified as visible; audio tick double-ownership resolved (this GDD: DRAFT_INITIAL/SHOP; auction-system.md: DRAFT_AUCTION); duck rule 50ms holdoff added; SAU-V1 regression fixed; 13 new ACs (SAU-DS9/10/11/12, SAU-DI11, SAU-DA14/15, SAU-EG6a/b, SAU-F20, SAU-DA11a/b, SAU-SET1a/b); OQ9 reclassified HIGH RISK. Re-review recommended in fresh session (/design-review --depth lean). |
| 16 | HUD | `gdd/hud.md` | Approved | /design-review Pass 4 2026-04-30: APPROVED. 1 blocker resolved: HUD-01 entity count corrected 16→18 (TextSpan children for gold labels were not counted). Recommended fixes: `GoldDisplayState` struct unified with `is_populated: bool` in Rule 1; Rule 11 `.before()` terminology; OQ-HUD-01 pre-implementation gate note added; LOBBY audio silent row + D.1 `u32`→`f32` type note. |

### M3 — Full Feature
*Goal: All GDD mechanics working — keywords, prisms, class rules, and animations.*

| # | System | File | Status | Notes |
|---|---|---|---|---|
| 17 | Keyword System | `gdd/keyword-system.md` | Needs Revision | /design-review 2026-04-30: NEEDS REVISION → revised in-session. 11 blockers resolved: KW-024 SHIELD logic fixed; OQ-KS1 split into 3 named seed slots (range_equidistant_select, teleport_random_dest, strich_change_lane_select); new Replication Contract subsection (7 persistent states with replication paths + reconnect recovery); OQ-NP1 expanded with attacker_id + was_blocked; KW-035 rewritten to component-state-at-boundary; KW-036–KW-040 promoted ADVISORY→BLOCKING; 11 ACs split (KW-004/008/009/010/015/020/027/029/031/033/035); REPEL i32 intermediate note; Dangerous Combinations table; COUNTERATTACK tooltip note; HASTE design-lever note. New OQs: NP2 (ResolutionTimeout), NP3 (DEATH chain order), NP4 (UnitBoardState fields), NP5 (KeywordTriggered variant), KS5 (OUTNUMBERED visual). 41 ACs → ~55 after splits. |
| 18 | Prism System | `gdd/prism-system.md` | In Review (Pass 2 revised) | /design-review pass 2 2026-04-30: 13 blockers resolved. Key fixes: `collected_this_round` ghost removed; OQ3→OQ1 cross-ref fixed; RESOLUTION invariant typo fixed; `S2CPrismRespawned` reliable message added (Rules 9/10); `S2CPrismRewardDropped` notification added (Rule 7); dual-WALL numeric model added (travel-time budget + pre-impl gate for Combat Resolution GDD); Lane 3 "ramp" framing corrected; server-rng.md Lane 3 hand-full conditional added (OQ4 resolved); OQ2 CLOSED (`S2CCardAcquired` confirmed implemented in protocol.rs); PS-07b added; PS-17 test artifact specified; PS-18/19 ADVISORY→BLOCKING; PS-20 updated; PS-23 relocated to card-acquisition.md. 25 ACs (23 BLOCKING, 1 ADVISORY). Still open: NP OQ1 (Lightyear unicast API — blocks epic start), OQ1 (hand-write API — blocks epic start). New messages (`S2CPrismRespawned`, `S2CPrismRewardDropped`) pending NP GDD registration. Re-review recommended in fresh session. |
| 19 | Class System | `gdd/class-system.md` | Needs Revision | /design-review 2026-04-30: MAJOR REVISION NEEDED → revised in-session (18 blockers resolved). Token stat blocks added (Mummy/Chacha Noir/La Gonflable/La Sacrifiée); CS-5 formula/example contradiction fixed; Player Fantasy 4-rhythm-archetype clarification; Rollback Minion-scope; Garde-Temps alive precondition; Punition Draw case; Seed walk-over = every traversed cell; CS-AC-03 BLOCKING; CS-AC-14 split; CS-AC-28/29/30 added; OQ-CS-1/3/4 closed; NP-1–5 opened (required NP changes). Open: OQ-CS-2 (Sang Méprise reconnect — NP backlog, mandated). Re-review recommended in fresh session. |
| 20 | Card Animations | `gdd/card-animations.md` | Needs Revision | /design-review Pass 2 2026-04-30: NEEDS REVISION → revised in-session (24 new blockers resolved across 6 domains). Key fixes Pass 2: OQ-CA-06 reopened (wrong API — `add_event` → `add_message`/`MessageReader` for all domain events); `PlacementCancelAllAnimsRequested` added to Rule C-10 domain event table + SystemSet ordering spec; F2 safe range corrected (400–550 ms, not 400–600 ms), "50 ms margin" (not 100 ms), sub-step floor documented (≥450 ms), post-deserialization assert ordering; F1 concurrent-stagger model confirmed + sort ownership moved to Board Rendering + F4 incompatibility noted; `impact_flash_audio_offset_ms` phantom config fixed (17 ms, not 67 ms); settlement overlay audio ownership gap flagged; placement reveal polyphony spec (1 batch trigger); GAME_OVER framing corrected (lands on climax, not rare); de-hover cancel replacement tween specified; AnimQueue lifecycle (clear on completion + reset on rebuild); DamageNumber marker component + StagedObjectiveRevealQueue stagger timer spec; CA-14→BLOCKING, CA-12 runtime-clamp fix, CA-16 domain corrected (i max=3), CA-18 strengthened, CA-21 board-cell source specified, CA-22 demoted ADVISORY, CA-10 advance_by fix, CA-3 sync claim dropped, CA-2 component-presence assertion, CA-5 split to CA-5a/5b/5c; CA-24 added (de-hover cancel-replace); OQ-CA-07 RESOLVED (1.5 s justified as affective beat); OQ-CA-11/12 added. 24 ACs (20 BLOCKING, 4 ADVISORY). 12 OQs. Re-review recommended in fresh session. |

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
