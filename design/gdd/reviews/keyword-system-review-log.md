# Keyword System — Review Log

## Review — 2026-04-30 — Verdict: NEEDS REVISION → revised in-session → Approved
Scope signal: L
Specialists: game-designer, systems-designer, network-programmer, qa-lead, creative-director
Blocking items: 11 | Recommended: 9 | Nice-to-have: 5
Summary: First /design-review pass on this GDD. Adversarial review uncovered: (1) KW-024 SHIELD AC has sub-step assignments reversed and contradicts KW-017 — pure AC text bug, design rule was correct; (2) OQ-KS1 "one seed slot covers RANGE+TELEPORT" claim was false and would create non-determinism (split into 3 named slots: range_equidistant_select, teleport_random_dest, strich_change_lane_select); (3) 7 persistent states (SHIELD, STUN, SILENCE, INJURED, LEADER bonus, OUTNUMBERED, BODYGUARD bond) had no replication contract — added new Replication Contract subsection with replication path / lifetime / reconnect recovery per state; (4) two pillar-violation risks flagged (HASTE rename doesn't self-describe; COUNTERATTACK same-cell-OR-adjacent rule cannot fit on a card) — user adjudicated to keep HASTE + add design-lever note, keep COUNTERATTACK rule + add tooltip note; (5) Tuning Knobs missing Dangerous Combinations guidance for round-1 objective damage stacks (CHARGE 5–6+HASTE+MP, RANGE 5–6+FS+HASTE, WALL+SHIELD+IRREMOVABLE+RESISTANCE); (6) qa-lead identified KW-036–KW-040 incorrectly tagged ADVISORY (all are pure Logic) — promoted to BLOCKING; 11 ACs needed splitting; KW-035 used fragile event-ordering — rewritten to component-state-at-boundary. Senior verdict (creative-director): NEEDS REVISION (not MAJOR — bones are right, ~half-day revision). User chose to revise in-session and accept as Approved without re-review. All 11 blockers + 9 recommended items applied; 5 new Network Protocol OQs (NP1–NP5) and 1 new Combat Resolution OQ (KS5 OUTNUMBERED visual) generated for downstream propagation.
Prior verdict resolved: First review

### User-adjudicated design decisions
- **HASTE rename**: Keep + design-lever note (revert deferred to post-vertical-slice if playtest confirms confusion).
- **COUNTERATTACK proximity rule**: Keep current (same-cell OR collision-halted adjacent) + ship client tooltip on first encounter.
- **KW-036–KW-040 reclassification**: Promoted ADVISORY → BLOCKING per qa-lead + creative-director recommendation.
- **CHARGE X / RANGE X safe range**: Keep max 6 + Dangerous Combinations note (card-pool authors are responsible for not shipping degenerate combos).

### Downstream propagation required
- `server-rng.md` Rule 5 RESOLUTION chain: register 3 new seed slots (OQ-KS1).
- `combat-resolution.md`: update OQ4 to Resolved (OQ-KS3); update OUTNUMBERED visual indicator from per-lane to per-unit (OQ-KS5).
- `card-data-pool.md`: HASTE schema field + Extension=1 audit (OQ-KS2).
- `network-protocol.md`: 5 new OQs (NP1 DisplacementEvent expanded; NP2 ResolutionTimeout variant; NP3 DEATH chain order encoding; NP4 UnitBoardState field additions; NP5 KeywordTriggered variant + KeywordPayload enum).
- `card-data-pool.md` Trap design: traversal trigger semantics + non-lethal/non-STUN continuation rule (OQ-KS4).

### Files revised
- `design/gdd/keyword-system.md` — 41 ACs → ~55 after splits/promotions; 5 OQs → 10 (5 OQ-KS + 5 OQ-NP); new Replication Contract subsection in Detailed Design; new Dangerous Combinations + COUNTERATTACK tooltip + HASTE design-lever subsections in Tuning Knobs; 5 new Edge Cases (INJURED classification, BODYGUARD bond storage, LEADER placed-this-round, REPEL Trap traversal parity, Trap continuation rule).
- `design/gdd/systems-index.md` — Status: Designed → Approved (entry updated with revision summary).
