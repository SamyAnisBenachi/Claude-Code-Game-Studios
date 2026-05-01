# Keyword System — Review Log

## Review — 2026-05-01 (R6 lean re-review) — Verdict: APPROVED
Scope signal: XL
Specialists: none (lean mode — single-session analysis)
Blocking items: 0 | Recommended: 3 found + applied
Summary: R6 lean pass on an already-APPROVED document. No new blocking items found. Formula boundary analysis (F1/F2/F3), AC coverage scan (all 82 ACs), replication contract, and dependency graph all pass clean. Three documentation hygiene items applied inline: (1) KW-041 struck-through row added between KW-040 and KW-042 — removed in R3 per D7 (1-cell-apart collision rule / ATTRACT enemy branching) but had no annotation, identical pattern to the KW-046 fix in R5; (2) OQ-NP2 header column formatted with strikethrough to match all other resolved OQs (body already said RESOLVED 2026-04-30); (3) OQ-KS3 R6 verification note added — combat-resolution.md is now Approved, verify OQ4 was struck through before COUNTERATTACK implementation. No design decisions. Implementation-ready.
Prior verdict resolved: Yes — R5 APPROVED status confirmed; R6 found no regressions.

## Review — 2026-05-01 (R5 lean re-review) — Verdict: APPROVED
Scope signal: XL
Specialists: none (lean mode — single-session analysis)
Blocking items: 1 found + applied | Recommended: 3 found + applied
Summary: R5 lean re-review of post-R5 file. R5 had closed OQ-KS6 (STUN/DEATH), OQ-KS7, OQ-KS8. One blocker remained: SILENCE+HASTE same-SS1 timing rule was unspecified — mirrors the existing explicit STUN+HASTE rule, same timing context, required an Edge Case entry and a BLOCKING AC (KW-082). Three recommended items also applied: (1) Formula 2 had a redundant post-formula collision check that was unreachable given the enemy-branch effective_pull formula — clarified to remove dead code; (2) KW-046 was removed in R3 per D8 but had no annotation — struck-through row added with rationale; (3) INJURED-granted keywords restoration after SILENCE expiry was implicit but unstated — Edge Case added. All creative decisions settled across R1–R5; no new design questions raised.
Prior verdict resolved: Yes — R5 closed all R4 blockers; R5 lean re-review verified and closed the final 4 items.

## Review — 2026-05-01 (R4 full + R5 inline) — Verdict: NEEDS REVISION → Revised Inline
Scope signal: XL
Specialists: game-designer, systems-designer, qa-lead, network-programmer, creative-director
Blocking items: 19 found | Recommended: 8 | False positives dismissed: 2 (GD-1 D3 already applied; GD-5 re-opens confirmed D2)
Summary: R4 found 19 real blockers across 4 tiers. All 9 R3 design decisions (D2–D9) correctly verified. Three tiers of issues: (1) 7 hard implementation traps — Formula 2 piecewise, REPEL lane precondition, outnumbered_cache contract, snapshot write-time, SilenceApplied payload, CounterattackFired target_id; (2) 5 design-rule gaps — COUNTERATTACK chain HP, DEATH trigger kill-gold, TELEPORT co-occupation, STUN+DEATH exception, Player Fantasy SILENCE test achievability; (3) 7 test-plan defects — duplicate AC, blocked ACs carrying BLOCKING gate, undefined fixture, 7 coverage gaps. R5 inline pass applied all 19 blockers. New design ruling: STUN does NOT suppress DEATH (DEATH is not an action-based hook). OQ-KS6, KS7, KS8 closed. 7 new BLOCKING ACs (KW-075–KW-081). Creative-director verdict: R5 re-review recommended in fresh session — all creative decisions settled, remaining work is verification pass only.
Prior verdict resolved: Yes — R3 closed design-rule gaps; R4+R5 closed protocol contract gaps and test plan integrity.

## Review — 2026-05-01 (R3 full) — Verdict: MAJOR REVISION NEEDED → Revised Inline
Scope signal: XL
Specialists: game-designer, systems-designer, qa-lead, network-programmer, creative-director
Blocking items: 25 found | Recommended: 12 | 9 design decisions applied inline
Summary: R3 surfaced two pillar violations (SILENCE anticipation test is unachievable — accepted as known risk D1; OUTNUMBERED mid-RESOLUTION flip — kept by design D2) and foundational protocol/schema gaps. Key design decisions: COUNTERATTACK simplified to "any non-RANGE attack" removing the proximity condition (D3); STUN = full shutdown including reactive hooks so COUNTERATTACK does not fire when stunned (D4); RANGE targets WALL as nearest enemy (D5); FIRST STRIKE can kill WALL in SS3 (D6); KW-041 removed — fundamental 1-cell-apart collision rule confirmed (enemy units can never be on same cell as opposing player unit, ATTRACT enemy cap is 1 cell short of caster, Formula 2 updated with branching for friendly vs enemy targets) (D7); LEADER snapshot moved to post-SS1 so LEADER placed this round grants bonus this round (D8); BODYGUARD with no target enters with None bond (D9). Protocol: silenced_until_round formula corrected to current_round+N-1 (was producing 2-round SILENCE); HASTE suppression note added; SILENCE client-clear note added. 5 new OQs (KS6–KS10) for propagation to NP GDD and board-lane-system.md. Creative-director verdict: systemic pattern of GDD-authored-top-down without protocol/fantasy authoring constraints — R4 should be final revision pass. R4 re-review recommended (fresh session).
Prior verdict resolved: Partially — R2 closed replication contract gaps; R3 closed design-rule and formula gaps. R4 will verify.

## Review — 2026-04-30 (R2 full) — Verdict: NEEDS REVISION → Revised Inline
Scope signal: L
Specialists: game-designer, systems-designer, qa-lead, network-programmer, creative-director
Blocking items: 31 addressed inline | Recommended: 10
Summary: First full specialist panel since R1 (which was accepted as Approved too early). Dominant issue: Replication Contract was never synced after NP GDD evolved through R3–R6 — 3 field type errors (STUN renamed, SILENCE type u8→u32, HASTE field absent), 2 stale OQs (NP4/NP5 resolved in NP GDD), 1 undefined type (KeywordKind→GrantedKeyword), and a broken INJURED derivation (UnitStats.max_hp doesn't exist in UnitStats). Additional fixes: LEADER stacking rule (no stack, earliest-placed); INJURED exhaustive grant list (4 variants: FS/CA/RANGE/SHIELD); SILENCE design intent framed as totalizing counter in Player Fantasy; SILENCE VFX + COUNTERATTACK tooltip exception in UI Requirements; SILENCE structured duration tracked as OQ-KS-new; REPEL X=0 authoring rule; ATTRACT lane-local precondition; OUTNUMBERED Fields excluded (confirmed); 16 new ACs (KW-042–KW-057). Creative-director adjudications: SILENCE scope intentional (keep rule, surface better); LEADER no-stack (earliest-placed); INJURED 4-variant closed list. R3 re-review recommended in fresh session.
Prior verdict resolved: Yes — R1 accepted-as-Approved was premature; R2 found 31 additional blockers from NP GDD drift and AC coverage gaps.

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
