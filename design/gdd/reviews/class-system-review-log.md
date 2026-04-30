# Review Log — Class System GDD

## Review — 2026-04-30 (Pass 3) — Verdict: NEEDS REVISION → Revised In-Session

Scope signal: M
Specialists: game-designer, systems-designer, qa-lead, economy-designer, network-programmer, ux-designer, creative-director
Blocking items: 11 GDD-local | Recommended: 15+ | Cross-doc BLOCKING tracked as NP/entities.yaml/Hand-UI debt
Summary: Pass 3 surfaced issues requiring cross-document synthesis that Pass 2 missed. Key fixes: (1) PIERCE keyword replaced with ARMOR-PIERCING (PIERCE was never defined in keyword-system.md); (2) Mummy passive "no cap" invalidates the Rollback↔Garde-Temps tradeoff — CS-3 language revised to "binding in Mummy-light games" with Mummy caveat; (3) CS-4/CS-6 both referenced Rule 9 (display-only) instead of Rule 7 (consequence path) — fixed; (4) CS-4 `chosen_enemy_objective.hp` was undefined in variable table — added as GameConfig.objective_hp (max HP constant); (5) Sinistro orphan state on parent destruction was unspecified — edge case added; (6) CS-12 Cra was a stub — added binding tempo rule and melee-push ruling; (7) CS-13 Iop scope declared; (8) Xelorium+Gelure burst ceiling added with design confirmation; (9) 12 new ACs added including CS-AC-31/32 (Miranda), CS-AC-34 (Madoll passive), CS-AC-35 (Craps alive=0 crash guard), CS-AC-38–41 (Cra/Iop), CS-AC-08b (HASTE+Rollback). NP-1 closed; NP-5 updated (4th mutation site: Garde-Temps); NP-9 registered (SeedPlaced/SeedConsumed). UX BLOCKING items (Hand UI per-card states, class-picker UX spec, Garde-Temps exhausted NP message) tracked as downstream debt — not GDD-local.
Prior verdict resolved: Yes — Pass 2 APPROVED confirmed; Pass 3 surfaced cross-doc synthesis issues not visible in isolated review.

## Review — 2026-04-30 (Pass 2) — Verdict: APPROVED

Scope signal: XL
Specialists: game-designer, systems-designer, qa-lead, economy-designer, network-programmer, ux-designer, creative-director
Blocking items: 30 | Recommended: 14
Summary: 30 new blocking items surfaced in re-review: Cra missing CS-12 RANGE anchor formula, Sadida AR ceiling resolved via PIERCE counter documentation, Rollback↔Garde-Temps strategic tradeoff made explicit, garde_temps_used_this_game ownership declared (Game Session System), OQ-CS-2 closed matching NP GDD, three new NP contracts registered (NP-6/7/8 for Sinistro/Miranda/Chacha Noir), CS-11 naming standardized, Craps division guard added, six UI requirements added (class picker state, Xelorium drain, Sinistro display, Garde-Temps permanent disable). Shava Shavien tails-to-opponent kept by design decision — CD's criterion #3 not met by design choice. All 30 blockers resolved in-session.
Prior verdict resolved: Yes — all 18 prior blockers confirmed resolved from Pass 1.

## Review — 2026-04-30 — Verdict: MAJOR REVISION NEEDED → Revised In-Session

Scope signal: XL
Specialists: game-designer, systems-designer, qa-lead, network-programmer, economy-designer, ux-designer, creative-director
Blocking items: 18 | Recommended: 12
Summary: The GDD had one formula that contradicted its own worked example (CS-5 Sang Méprise, resolved by correcting the example to match the formula for full mutual reveal), four token entities with no stat blocks (resolved by adding HP/ATK/MP from Krosmaga reference), and two classes (Sadida, Ecaflip) whose mechanics required a pillar reframe to clarify that "authorship-via-class-rhythm" encompasses four distinct modes — authored outcome, authored sacrifice, prepared battlefield, and authored risk exposure. Four open questions were closed (OQ-CS-1/3/4 and a new one added for CS-AC-27b). Five mandatory Network Protocol GDD changes were identified and documented as NP-1 through NP-5. All 18 blocking items were addressed in-session.
Prior verdict resolved: No — first review
