# Review Log — Combat Resolution GDD

## Review — 2026-04-30 — Verdict: MAJOR REVISION NEEDED → Revised In-Session → APPROVED

Scope signal: XL
Specialists: game-designer, systems-designer, qa-lead, network-programmer, gameplay-programmer, creative-director
Blocking items: 9 | Recommended: 9
Summary: Full panel identified three P0 issues: (1) SHIELD was specified in three mutually contradictory positions — pre-check in SS6, step 10 in modifier stack, and "after COUNTERATTACK" in SS3. Resolved by canonicalizing as pre-check, removing from modifier stack step 10. (2) COUNTERATTACK had no retaliation formula anywhere in either GDD, and its timing contradicted keyword-system.md. Resolved by defining the full formula (full modifier stack, FINAL BLOW eligible, chains once, multi-attacker simultaneous retaliation) and fixing timing to "once after all damage applied." (3) RANGE equidistant RNG seed slot was missing from server-rng.md, which would have corrupted audit-log sequences. Added `range_equidistant_select` to the RESOLUTION caller table. P1 fixes: bilateral+multi-source overlap rule specified; INJURED/OUTNUMBERED/death ordering at sub-step boundaries fixed; kill_log attribution mechanism added; internal 10k iteration budget added as deadlock guard. OQ1 and OQ5 closed. CR-44/45 promoted to BLOCKING. All OQs resolved.
Prior verdict resolved: Yes — addressed all 25 blockers from 2026-04-29 pass

## Review — 2026-04-29 — Verdict: MAJOR REVISION NEEDED → Revised In-Session

Scope signal: XL
Specialists: game-designer, systems-designer, qa-lead, network-programmer, gameplay-programmer, creative-director
Blocking items: 25 | Recommended: 14
Summary: Five specialist agents identified three foundational structural issues: (1) the simultaneous-vs-sequential combat model was undefined, leaving AR_attacker_combat with no algorithm and the SHIELD/multi-source interaction contradictory; (2) the document contradicted itself in three places about INJURED sub-step activation (sub-step 4 vs 5); (3) the network event schema was missing CombatDamage and KeywordTriggered variants, making the animation contract undeliverable. All 25 blocking items were resolved in-session: two-pass simultaneous combat algorithm formalized, u8/i32 arithmetic spec added, CR-26 corrected, OQ2 (type advantage → GameConfig) and OQ4 (COUNTERATTACK adjacency) resolved, 11 new ACs added (CR-35–CR-45), CR-31–34 reclassified BLOCKING, SHIELD pre-check algorithm specified, gold authority clarified as batch-only. Remaining open: OQ1 (WALL ADR), OQ3 (RANGE equidistant RNG seed), OQ5 (network-protocol.md needs CombatDamage/KeywordTriggered event variants).
Prior verdict resolved: No — first review
