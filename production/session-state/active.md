# Session State

## Current Task
/review-all-gdds — FAIL verdict (2026-04-29 R2)

## Status
Cross-GDD review R2 complete. 7 blocking issues prevent architecture start.
Progress: 6 prior blockers resolved this session, 3 new blockers surfaced.

## Session Extract — /design-review game-config.md 2026-04-29
- Verdict: APPROVED (post-revision, then NEEDS REVISION again per /review-all-gdds R2)
- Key changes: 5 RSM timer fields added, reserve_mana_cap + interest_threshold_gold added, epic/legendary moved to const, 6 new validation rules, 13 ACs rewritten, Formula 3 in card-data-pool.md updated to use fake_objective_spawn_advance
- N-B1 discovered post-approval: refresh_base_cost missing from struct (economy-system.md references it)
- Review log: design/gdd/reviews/game-config-review-log.md

## Session Extract — /review-all-gdds 2026-04-29 R2
- Verdict: FAIL
- GDDs reviewed: 7
- Flagged for revision: game-config.md, economy-system.md, card-data-pool.md, lanes-and-lies-gdd.md, entities.yaml
- Report: design/gdd/gdd-cross-review-2026-04-29.md (overwritten with R2)

## Remaining Blocking Issues (7)

**Consistency cluster:**
- C-B4: card-data-pool.md missing refresh_shop() auto-refresh policy (§3.4)
- C-B5: interest formula hardcodes /5 in economy-system.md, master GDD §7, entities.yaml
- C-B6: S2CGameOver + GameOverReason unregistered in entities.yaml; not in master GDD
- N-B1 (NEW): refresh_base_cost missing from game-config.md (economy-system.md references it)
- N-B2 (NEW): master GDD §3.9 + §4.1 still say "no reserve cap" — contradicts reserve_mana_cap

**Design cluster:**
- D-B1: Fake-first still strictly dominant (unchanged)
- B-2 (NEW): Garde-Temps costs 20 reserve but reserve_mana_cap=10 — card permanently unplayable. Introduced by D-B2 fix. Design decision required: raise cap, lower cost, or change spending model.
- B-3 (NEW): Free card pick from fake destruction can draw Legendary from auction pool — bypasses "Auction as signature." Fix: cap free pick at Rare/Epic.

## Resolved This Session (6)
- C-B1: disconnect_grace_seconds — master GDD updated (OQ6 resolved, §5 = 30s, M4 = 30s)
- C-B2/C-B3: 5 RSM timer fields added to game-config.md
- D-B2: reserve_mana_cap=10 added to game-config.md
- D-B3: OQ1 resolved in economy-system.md (though B-3 shows the resolution has a design flaw)
- D-B4: server-rng.md Approved (RNG execution order rewritten as per-phase chains)

## Recommended Fix Order (before re-running /review-all-gdds)

1. **Design decisions first (block the mechanical fixes):**
   - D-B1: Choose fake reward asymmetry fix (spawn decoupled, or real objectives get bonus)
   - B-2: Choose Garde-Temps path (raise reserve_mana_cap, lower cost, or combined pool)
   - B-3: Confirm free pick capped at Rare/Epic — update economy OQ1 and relevant GDDs

2. **Mechanical fixes (no design decisions needed):**
   - N-B1: Add refresh_base_cost to game-config.md (struct + Tuning Knobs + ACs)
   - N-B2: Update master GDD §3.9 (line 377) and §4.1 (line 518) reserve cap claims
   - C-B5: Update economy-system.md formula, master GDD §7 row, entities.yaml expression
   - C-B6: Add S2CGameOver + GameOverReason to entities.yaml; add master GDD §8 stub
   - C-B4: Run /design-review card-data-pool.md (needs §3.4 auto-refresh policy)

3. **Re-run /review-all-gdds**
