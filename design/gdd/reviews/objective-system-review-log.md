# Review Log — Objective System

---

## Review — 2026-04-29 R2 — Verdict: MAJOR REVISION NEEDED → Revised Inline

**Scope signal:** XL (9 dependencies, 5 formulas, 3 ADRs required, Lightyear technical spike blocking)
**Specialists:** game-designer, systems-designer, network-programmer, qa-lead, creative-director
**Blocking items:** 16 identified | 16 resolved inline
**Recommended items:** 14 | Prior verdict: MAJOR REVISION NEEDED (R1)

**Summary (creative-director):** Three existential risks identified: (1) Lightyear per-component replication scope unverified — silent identity leak would delete the bluff mechanic; (2) Sang Méprise edge case contradicted OQ6 with Option A language; (3) the reveal moment had no specification despite being the system's primary emotional payload. Resolved: reveal moment subsection added (500ms mandatory gap, batch delivery, Board Rendering constraints specified); Sang Méprise edge case rewritten to Option B unicast + reconnect gap flagged in both edge case and OQ5; OQ1 removed; OQ4 (formerly OQ5) strengthened to BLOCKING technical spike. Formula cluster (D2-1, D5-1, D5-4, D5-5, D4-3, D4-4) fully fixed: loss_threshold corrected as fixed constant with explicit "do not derive in code" note; LaneId defined as 1-indexed throughout; seed count corrected per path; fake_count bounds invariants added (>=1 and <=3); prose/assertion contradiction at fake_count=3 fixed. AC set restructured: OS-10/12 scoped to Objective System outputs only; OS-17/18b reclassified ADVISORY pending OQ4 ADR; OS-13/OS-23 split; OS-24–28 added covering Sang Méprise + ObjectiveDestroyed, Garde-Temps path, mixed rewards, PoolFilter contract, and fake_count=1 boundary.

**Key blocker remaining for R3:** OQ4 technical spike — verify Lightyear 0.26 per-component replication scope or adopt unicast architecture. All other blockers resolved.

**Prior verdict resolved:** Yes — all 16 R2 blockers resolved inline.

---

## Review — 2026-04-29 — Verdict: MAJOR REVISION NEEDED → Revised Inline

**Scope signal:** XL (9 dependencies, 5 formulas, new ADRs required for Lightyear replication and interest timing)
**Specialists:** game-designer, systems-designer, network-programmer, economy-designer, qa-lead, creative-director
**Blocking items:** 12 identified across specialists | 6 resolved inline | 3 deferred to ADR
**Recommended items:** 8 | Prior verdict: First review

**Summary (creative-director):** The HP visibility model was the dominant finding — exact integers risk collapsing the bluff into arithmetic. User overrode this as a deliberate design decision (retained exact HP integers with documented rationale). Remaining blockers resolved: ObjectiveDestroyed payload corrected to include `target_player_id`; config invariant guards added for malformed `fake_count` and `objective_hp`; fake reward asymmetry and interest bracket capture documented as intentional design; full-hand fallback clarified as non-exploitable; pool-exhausted FreeCardPick AC added. Sang Méprise replication resolved to Option B (unicast event, not dynamic scope change). Lightyear replication strategy flagged for ADR.

**Blockers resolved inline:**
1. `ObjectiveDestroyed` missing `target_player_id` → payload updated in Rule 7, AC OS-13, registry
2. Config invariant (fake_count ≥ 4 = unwinnable) → Edge Cases + Tuning Knobs guard + AC OS-23
3. Config invariant (objective_hp = 0 = spawn destroyed) → Edge Cases + AC OS-23
4. Fake reward asymmetry undocumented → design-intent note added after Rule 10
5. Interest bracket capture undocumented → design-intent note added after Rule 10
6. Full-hand fallback non-exploitable → Edge Case clarification added
7. Pool-exhausted FreeCardPick no AC → AC OS-22 added

**ACs restructured:**
- OS-9: narrowed to count-only (RSM transition owned by RSM integration tests)
- OS-18: split into OS-18a (ordering) and OS-18b (batching/transport)
- OS-21 added: self-destruction of a fake (distinct branch from OS-14)
- OS-17: manual evidence path reclassified as ADVISORY

**Deferred to ADR:**
- OQ5: Lightyear replication strategy for `ObjectiveIdentity` (unicast at session start recommended)
- OQ6: Sang Méprise safe mechanism (Option B unicast — authoritative; do not implement Option A)
- Interest snapshot timing: resolved via RSM Rule 4 cross-reference (fires after objective rewards)

**Design override:** HP display kept as exact integers (not qualitative states) despite game-designer + creative-director recommendation. Documented as deliberate in Rule 4 design note. Monitor in playtesting.

**Prior verdict resolved:** N/A — first /design-review for this GDD. (/review-all-gdds R4 previously fixed 2 issues inline: Rule 10 mana cap ceiling, draw_random interface.)
