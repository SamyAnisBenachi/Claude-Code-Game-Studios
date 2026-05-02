# Session State — Lanes and Lies

> Lis ce fichier EN PREMIER dans toute nouvelle session.
> Il contient l'état complet du projet au 2026-04-30.
>
> **✅ SESSION 2026-05-01 (continued) — /design-review network-protocol.md (R7) + round-state-machine.md (R2) — COMPLETE**
>
> **network-protocol.md** — R7 lean: APPROVED (revisions accepted inline)
> 4 blockers resolved: S2CPrismRespawned + S2CPrismRewardDropped added to S2C table + NP-56/57 ACs; EndOfTurnFired removed from inner-sub_step invariant list; NP-52/NP-53 reclassified BLOCKING. Systems-index updated. Review log appended.
>
> **round-state-machine.md** — R2 full (5 specialists): APPROVED (revisions accepted inline)
> 13 blockers resolved: Rule 13 hybrid heartbeat model; F2 Bevy scheduling table; Rule 14 Option<u32> timer; Rule 7 IDLE invariant; auction_max_duration 120→300; RSM-34/38 BLOCKING; auction_followup_placement_timer_seconds Tuning Knob; 6 new ACs (RSM-11b, 29b, 29c, 33a, 35b, rewritten RSM-9). OQ-4 and OQ-6 resolved. Systems-index updated. Review log appended.
>
> **⚠️ Pending cross-GDD item:** NP GDD S2CPhaseChanged.timer_duration_ms must change u32→Option<u32> (flagged in RSM Rule 14 cross-GDD note). Owner: first RSM implementer.
>
> **All M1 Foundation GDDs are now Approved.**
>
> Next recommended: /consistency-check (to apply S2CPhaseChanged type fix + surface any cross-GDD drift) OR /review-all-gdds (holistic M1+M2+M3 review)
>
> **✅ SESSION 2026-05-01 — /ux-design class-picker — COMPLETE**
> UX spec written section-by-section: design/ux/class-picker.md
> Status: Complete — pending /ux-review
> OQ-MM-1 and OQ-MM-2 from main-menu.md resolved.
> 2 new patterns flagged for interaction-patterns.md: Card Frame Container, Dot Position Indicator.
> Next: /ux-review design/ux/class-picker.md — OR — /design-review design/gdd/keyword-system.md (R4)
>
## /design-review keyword-system.md R3 — Session Interrupt State
> Reprendre: lire les decisions ci-dessous, editer design/gdd/keyword-system.md, puis systems-index + review-log.

### Verdict R3: MAJOR REVISION NEEDED (worse than R2)
Specialists: game-designer, systems-designer, qa-lead, network-programmer, creative-director
25 blocking items, 12 recommended. GDD edits NOT YET WRITTEN.

### 9 Design Decisions (CONFIRMED BY USER — apply all to GDD):

**D1 — SILENCE Player Fantasy**: Keep as-is (accepted risk). Design test stays. SILENCE VFX animation is the mitigation. No info mechanism needed.

**D2 — OUTNUMBERED timing**: Keep current per-sub-step re-evaluation. Document the "flip risk" (OUTNUMBERED unit that kills its way to parity loses the bonus) in Tuning Knobs as a known design tension.

**D3 — COUNTERATTACK rule**: SIMPLIFY. Fires on any non-RANGE attack. Remove "same-cell OR collision-halted adjacent" proximity condition entirely. Remove the tooltip exception — rule is now card-text-derivable from the name alone. Update keyword definition + all references.

**D4 — STUN + COUNTERATTACK**: COUNTERATTACK does NOT fire when stunned. STUN = full shutdown including reactive hooks. Add to Edge Cases and STUN keyword definition.

**D5 — RANGE + WALL**: RANGE targets nearest enemy normally. WALL is a valid RANGE target. RANGE cannot "shoot through" a WALL — if WALL is the nearest enemy, RANGE attacks WALL. WALL's blocking behavior (movement halt) does not affect RANGE targeting. Add to Edge Cases under both RANGE and WALL keywords.

**D6 — FIRST STRIKE + WALL**: FIRST STRIKE CAN attack a WALL in SS3. If FS kills WALL in SS3, WALL is removed in SS4 and no longer blocks movement in SS5. Intentional counter-play (FIRST STRIKE + CHARGE X counters WALL lane anchors). Add to Edge Cases.

**D7 — ATTRACT backfire (CRITICAL RULE CLARIFICATION)**: KW-041 is INVALID and must be REMOVED. Fundamental rule confirmed by user: "1 player unit and 1 enemy unit can NEVER be on the same cell. When they make contact they are always 1 cell apart." Therefore:
- An enemy unit can never be ATTRACTed to the caster's own cell (1-cell-apart rule applies)
- KW-041 premise was wrong — enemy cannot reach Cell 1 (Player A's objective)
- Formula 2 description "stops at caster's cell" must change to "stops 1 cell short of caster's cell for enemy targets (collision rule applies)"
- Remove the Edge Case: "Pulling an enemy to your own objective cell is valid"
- Add to Edge Cases: "Collision rule applies to ATTRACT for enemy targets: an enemy unit pulled by ATTRACT stops 1 cell short of the caster's cell, never sharing the caster's cell. For friendly targets, the unit can stop at the caster's cell."
- Add note: "PROPAGATE TO board-lane-system.md: the 1-cell-apart collision rule between opposing units must be formally defined there and referenced here."
- TELEPORT is the only exception: "Co-occupation is allowed" (already stated) — TELEPORT bypasses collision rules.

**D8 — LEADER snapshot timing**: CHANGE to post-SS1. Snapshot taken after all SS1 APPEARANCE effects resolve, before SS2 begins. A LEADER placed in SS1 of round R grants its bonus in round R (not deferred to R+1). Update LEADER keyword definition, States table, and Edge Cases. Remove KW-046 (which tested the now-incorrect "no bonus this round" behavior). Update KW-047 (still valid — two same-family LEADERs). Note: Combat Resolution GDD also needs updating (snapshot timing change). File a propagation note.

**D9 — BODYGUARD no valid target**: Enters with no bond (bodyguard_protects = None). BODYGUARD always enters successfully. If no other friendly unit exists, bond is None, no protection provided. Add to Edge Cases.

### GDD Edits Required (design/gdd/keyword-system.md):

**Rules/definitions to change:**
- COUNTERATTACK: Remove "same-cell OR collision-halted adjacent" → "any non-RANGE attack". Remove COUNTERATTACK tooltip note from UI Requirements.
- STUN: Add "STUN suppresses all keyword hooks including reactive triggers (COUNTERATTACK does not fire when stunned)."
- RANGE: Add RANGE+WALL interaction rule.
- LEADER: Change snapshot timing from "RESOLUTION entry" to "after SS1 completes, before SS2 begins."
- ATTRACT formula: Update cap description for enemy targets (1 cell short of caster cell).

**Edge Cases to add:**
- RANGE + WALL (D5)
- FIRST STRIKE + WALL (D6)
- STUN + COUNTERATTACK = no COUNTERATTACK (D4)
- ATTRACT collision for enemy targets / "1-cell-apart" rule (D7)
- BODYGUARD no valid target at entry → None bond (D9)
- OUTNUMBERED flip risk (D2, in Tuning Knobs)
- SILENCE + IRREMOVABLE (silenced IRREMOVABLE = displaceable)
- SILENCE + UNTARGETABLE (silenced UNTARGETABLE = Spell/Order-targetable)
- BODYGUARD+UNTARGETABLE immune to SILENCE (add to Dangerous Combinations)
- LEADER grants itself its own bonus
- DEATH chain bound wrong if DEATH triggers spawn units (remove "structural 9-link" certainty)
- INJURED via APPEARANCE grants bonuses from SS2 (powerful synergy note for card authors)
- BODYGUARD CHANGE LANE (BODYGUARD itself): bond persists

**ACs to change/add/remove:**
- Remove KW-041 (ATTRACT backfire — invalid premise)
- Remove KW-046 (LEADER placed this round no bonus — now incorrect after D8)
- Update KW-007 (INJURED timing still valid but LEADER snapshot language changes)
- Add KW-029c: REPEL X=6 at Cell=1 (zero traversal boundary)
- Add KW-029d: REPEL X=6 at Cell=8 for Player B (zero traversal boundary)
- Add: SHIELD persisting across rounds
- Add: DEATH chain re-entry prevention (already-dead set)
- Add: IRREMOVABLE + CHANGE LANE (own movement allowed)
- Add: STUN+COUNTERATTACK = does not fire
- Add: FIRST STRIKE + WALL kills in SS3
- Fix KW-035a: replace "GoldLedger" with actual resource name (leave as TODO for implementation)
- Fix KW-054 inline comment "(2 > 1)" → "count(A)=2, count(B)=1, `2 < 1 = false`"

**Protocol/schema fixes (Replication Contract):**
- Fix SILENCE `silenced_until_round` formula: server computes `current_round + N - 1` (expiry-inclusive), client renders while `current_round <= silenced_until_round`. Add worked example for N=1.
- Add to HASTE row: "SERVER MUST NOT emit HasteActivated for STUNned HASTE units."
- Add to SILENCE row: "On SilenceApplied, client MUST clear all runtime INJURED-granted keyword state."
- Add to COUNTERATTACK event note: "CounterattackFired payload must include target_id: EntityId." (NP GDD update required — add as OQ)
- Add note: INJURED-RANGE GrantedKeyword::Range must carry {max_range: u8}. (NP GDD D.3 update required)

**Formula updates:**
- Formula 2 (ATTRACT): Add i32 intermediate arithmetic safety note (matching Formula 1). Update cap description for enemy targets.
- OUTNUMBERED formula: Replace "evaluated at the start of each sub-step" → "evaluated at each sub-step boundary — after the preceding sub-step fully completes, before the current sub-step begins."

**OQs to add:**
- OQ-KS6: STUN suppresses ALL keyword hooks including reactive triggers (CONFIRMED: COUNTERATTACK does not fire while stunned). Update with ruling.
- OQ-KS7: SilenceApplied event payload must include stripped_keywords list (NP GDD update required before SILENCE implementation).
- OQ-KS8: CounterattackFired must include target_id for multi-attacker scenarios (NP GDD update required).
- OQ-KS9: LEADER snapshot timing change (post-SS1) must propagate to Combat Resolution GDD.
- OQ-KS10: Board-lane-system.md must formally define the "1-cell-apart" collision rule for opposing units (referenced in ATTRACT formula fix).

**Status to update:** Change header from "Needs Revision" to "Needs Revision — R3 MAJOR REVISION, decisions collected, edits in progress (R4 pending)"

---

> **Session active (2026-04-30):** HUD GDD ✅ DESIGNED — design/gdd/hud.md. All 8 sections + Visual/Audio + UI Requirements + Open Questions complete (21 ACs, 18 BLOCKING). Registry: 10 referenced_by updated. Systems index updated. /design-review pending (fresh session). Next: /ux-design hud (fresh session).
>
> **Parallel session (2026-04-30):** Class System GDD ✅ DESIGNED — design/gdd/class-system.md. All 8 sections + UI Requirements + Open Questions complete (27 ACs, 26 BLOCKING). 7 token entities + 8 formulas + 5 constants registered. 4 OQs: Xelorium timing, Sang Méprise reconnect gap (NP), Rollback+HASTE, Madoll spell scope. Systems index updated. /design-review pending (fresh session).
>
> **Parallel session (2026-04-30):** Card Animations GDD ✅ DESIGNED — design/gdd/card-animations.md. All 11 sections complete (8 required + Visual/Audio + UI Requirements + Open Questions). 20 ACs (16 BLOCKING, 4 ADVISORY). 5 custom lenses, domain-event indirection architecture, AnimGroup/AnimQueue drain. Registry: +stagger_cadence_ms. 9 OQs (5 API-verification gating implementation, 3 cross-system recommendations). Systems index updated. /design-review pending (fresh session).
>
> **Parallel session (2026-04-30):** Prism System GDD ✅ DESIGNED — design/gdd/prism-system.md. All 8 sections + Visual/Audio + UI Requirements + Open Questions complete (22 ACs, 17 BLOCKING). Registry: 2 items (prism_strike, prism_reserve), 2 constants (prism_strike_damage, prism_strike_mana_cost), 1 network_message (S2CCardAcquired). Systems index updated. /design-review pending (fresh session).
>
> Prior session (2026-04-29): Hand UI GDD ✅ DESIGNED — design/gdd/hand-ui.md. All 8 sections + Visual/Audio + UI Requirements + Open Questions complete (24 ACs). Systems index updated. /design-review pending (fresh session).
>
> **Session (2026-04-30):** /dev-story lyv-002 (All Protocol Message Types) — story already fully implemented in commit 759bd4a. All ACs verified against shared/src/protocol.rs. Evidence at tests/evidence/story-lyv-002-types-check.md. Next: /story-done production/epics/lightyear-protocol-verification/story-002-all-protocol-message-types.md

## Session Extract — /architecture-review 2026-05-01 (Run 4)
- Verdict: **PASS** — first PASS verdict in project history
- ADRs reviewed: 22 (ADR-001..022). Status: **22/22 Accepted** (Run 3: 18+2 Proposed)
- New ADRs since Run 3: ADR-021 (Presentation Layer, Accepted), ADR-022 (Keyword Observer, Accepted)
- ADR promotions: ADR-018 Proposed→Accepted (ADR-005/006 amendments landed), ADR-020 Proposed→Accepted (ReplicateTo verified)
- All Run 3 blockers resolved: B-1'(HC-1) ✅ B-2'(HC-2) ✅ B-3'(SI-1) ✅ B-4'(HC-3) ✅ B-5'(B-4) ✅ B-6'(ADR-021) ✅ B-7' partial (control-manifest fresh; architecture.md still stale)
- TR registry: version 4, 181 active TRs; coverage ~100% (was 77% Run 3)
- Open items: SC-1 `Trigger<T>` vs `On<T>` in observer param (MEDIUM — pre-impl gate before KW-001); SC-2 architecture.md stale (MEDIUM); OQ-KS9 LEADER timing → combat-resolution.md (LOW)
- Report: docs/architecture/architecture-review-2026-05-01.md
- Next: SC-1 verification (cargo check Trigger<T> stub before first keyword story); architecture.md refresh; /gate-check pre-production when ready

## Session Extract — /architecture-review 2026-04-30 (Run 3)
- Verdict: **CONCERNS** — improving (M1 PASS unchanged; Core M2/M3 mostly PASS pending stale-ref cleanup; Presentation FAIL)
- ADRs reviewed: 20 (ADR-001..020). Status: 18 Accepted, 2 Proposed (ADR-018 Keyword, ADR-020 Board/Lane)
- Status changes since Run 2 (2026-04-30b): ADR-013/014/015/016/017 Proposed→Accepted; ADR-019 Economy NEW Accepted; ADR-020 Board/Lane NEW Proposed
- Prior blockers status: B-1 ✅ (ADR-015 H1), B-5 ✅ (ADR-010 Prism row), B-6 ✅ (ADR-013 dup row); **B-2 ❌ ESCALATED** (hand storage triple-named now CRITICAL — three Accepted ADRs conflict); B-3 ⚠️ partial (ADR-019 Accepted but stale `EconomyState` in 4 ADRs); B-4 ❌ (ADR-005/006 amendments still missing)
- New TR-IDs registered: 80 (TR-CR×12, TR-CA×10, TR-PRI×8, TR-KW×12, TR-CAN×7, TR-BR×7, TR-HU×8, TR-SAU×6, TR-HUD×10). TR-CS pre-populated by another agent (8 entries). Total active: 178 TRs
- Coverage: 140/182 covered (77%), 27 partial, 15 gaps. Run 2→3 delta: +71 newly registered TRs, +14% coverage
- Top blocking issues: B-1' HC-1 hand storage (CRITICAL); B-3' SI-1 stale EconomyState (HIGH, ADR-015 line 300 load-bearing); B-4' HC-3 ADR-020 Lightyear `ReplicateTo` UNVERIFIED (HIGH); B-5' ADR-005/006 amendments for ADR-018 (HIGH); B-6' Presentation Layer ADR-021 missing (MEDIUM)
- GDD revision flags: None new (R8/R9 flags still active)
- Stale docs: architecture.md (last updated 2026-04-29, covers ADR-001..012 only); control-manifest.md (covers ADR-001..012 only, lists ADR-013..018 as pending)
- Required new ADRs: (1) ADR-005 amendment seed slots; (2) ADR-006 amendment SimpleKeyword 20 variants; (3) ADR-021 Presentation Layer
- Report: docs/architecture/architecture-review-2026-04-30c.md
- Next: fix HC-1 (PlayerHands rename in ADR-014/016) → fix SI-1 (ripple `EconomyState`→`PlayerEconomies`) → WebFetch Lightyear 0.26 for ReplicateTo → land ADR-005/006 amendments → author ADR-021

## Session Extract — /review-all-gdds 2026-04-30 (R9)
- Verdict: FAIL
- GDDs reviewed: 20
- Flagged for revision: network-protocol.md, card-animations.md, class-system.md, round-state-machine.md, hand-ui.md, shop-auction-ui.md, keyword-system.md, entities.yaml
- Blocking issues: 11 — (C-R9-1) S2CSingleObjectiveReveal unregistered in NP; (C-R9-2) CA Rule C-8 contradicts NP D.2 trigger_index ordering; (C-R9-3) S2CActivationRejected unregistered in NP/registry; (C-R9-4) entities.yaml S2CGameOver duplicate notes + 3-vs-4 stale; (C-R9-5) DRAFT_INITIAL grid dual-ownership hand-ui vs SAU; (C-R9-6) OQ-PLACEMENT-LOAD not filed in RSM; (C-R9-7) mummy_damage_reserve_cap missing; (C-R9-8) keyword-system:166 stale OQ ref; (D-R9-1) PLACEMENT cognitive overload 10 active systems; (D-R9-2) Xelor reserve dominant strategy (Mummy uncapped); (D-R9-3) DRAFT_AUCTION hand-full lockout = anti-pillar violation
- Resolved this cycle (R8→R9): 18 items including C-B4, C-B6, C-NEW-2/3/7, C-R8-2/3/4/5/7/8/10/11/12/13, D-B1+C-NEW-5, D-B4, D-B5
- New design concerns: No-idle-spectating pillar FAIL (D-R9-3); Deep emergence CONCERN (D-R9-2 Xelor reserve loop, D-R9-5 Sadida seed PIERCE unverified)
- Quick-fixes applied (2026-04-30): C-R9-4+W14 (entities.yaml S2CGameOver merged notes + 4 variants), C-R9-8 (keyword-system:166 stale OQ ref stripped), C-R9-6 (RSM OQ-PLACEMENT-LOAD filed as OQ6), C-R9-5 (DRAFT_INITIAL ownership: SAU=rendering, hand-ui=fan animation — both GDDs annotated), D-R9-4+C-R9-W9 (master GDD §2 Player Fantasy layering paragraph added)
- Recommended next: 3-file coordinated edit — (1) NP: register S2CSingleObjectiveReveal + S2CActivationRejected + add DRAFT_AUCTION to C2SActivateCard; (2) CA: Rule C-8 trigger_index update; (3) class-system: Mummy cap + Xelorium worked example + Hand UI dep
- Report: design/gdd/gdd-cross-review-2026-04-30-r9.md

## Session Extract — /review-all-gdds 2026-04-30 (R8)
- Verdict: FAIL
- GDDs reviewed: 20
- Flagged for revision: round-state-machine.md, network-protocol.md, class-system.md, auction-system.md, hand-ui.md, keyword-system.md, objective-system.md
- Blocking issues: 13 — (1) GameOverReason 3-vs-4 split across RSM/NP/KS/registry; (2) S2CGoldUpdate payload 4-vs-5 fields; (3) auction→hand-ui dep gap; (4) StartAuction→AuctionPhaseEntered rename incomplete in RSM; (5) Sang Méprise snapshot field name unlocked; (6-8 carryover) S2CCardAcquired in hand-ui Rule 5c, S2CSangMepriseReveal + S2CSingleObjectiveReveal not in registry, C-NEW-4 trigger_index echo; (9-10) Garde-Temps destroy() vs take_damage(); (11) D-B4 disconnect-grace UX; (12) D-B5 Sang Méprise reconnect; (13) Xelor reserve 4-stacking-sources (Mummy passive no cap)
- New design concerns: cognitive load 9+ systems during PLACEMENT (overloaded); Xelor dominant-strategy risk; Sadida seed AR stacking; three Player Fantasies unreconciled in master GDD
- Resolved this cycle: C-B5 (hand-ui internal contradiction), D-B3 (auction wealth gap reframing)
- Recommended next: 3-coordinated-edit batch: (1) registry single pass, (2) GameOverReason + S2CGoldUpdate reconciliation, (3) StartAuction rename in RSM
- Report: design/gdd/gdd-cross-review-2026-04-30-r8.md

---

## Stage actuel : Pre-Production ✅
`production/stage.txt` = `Pre-Production`

## Session Extract — /story-done 2026-05-01
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/auction-system/story-001-auction-state-scaffold.md` — AuctionState Types & Snapshot Scaffold
- Criteria: 5/5 passing (AU10-a/b/c/d/e all covered by unit tests)
- Test evidence: `tests/unit/auction/auction_state_scaffold_test.rs` — exists at required path; 7 tests; CI confirmed passing
- Advisory deviations: `starting_price` field added to `AuctionState`/`AuctionSnapshot` per story's own verification note; manifest version stale by 1 day (no blocking rules delta)
- Tech debt logged: None
- Sprint status: `AUC-001` already `done` in `production/sprint-status.yaml`
- Next recommended: S3-02 GSS Story 2 (Room Create and Join), S3-03 GSS Story 3 (Class Selection and Reveal), S3-05 RSM Story 4 (Win Condition and Game Over)

---

## Sprint 1 — État des stories

| ID | Story | Fichier | Statut |
|---|---|---|---|
| S1-01 | Cargo Workspace Scaffolding | `production/epics/workspace-and-shared-types/story-001-cargo-workspace-scaffolding.md` | ✅ Done |
| S1-02 | Shared Card Types | `story-002-shared-card-types.md` | ✅ Done |
| S1-03 | GameConfig POD Struct | `story-003-game-config-pod-struct.md` | ✅ Done |
| S1-04 | Protocol Skeleton + CI Gates | `story-004-protocol-skeleton-ci-gates.md` | ✅ Done |
| S1-05 | Lightyear 0.26 Spike ⭐ | `production/epics/lightyear-protocol-verification/story-001-lightyear-026-verification-spike.md` | ✅ ADR-012 items 15-16 verified locally |
| S1-09 | ServerRng Type Definitions | `production/epics/server-rng/story-001-type-definitions-audit-infrastructure.md` | ✅ Done |

**Machine-readable status :** `production/sprint-status.yaml`
**Plan complet :** `production/sprints/sprint-1.md`

---

## CI GitHub Actions

**Dernier commit :** `88971ec` — "Fix CI: remove invalid bevy_ecs feature, strip bevy from shared/"
**URL :** https://github.com/SamyAnisBenachi/Claude-Code-Game-Studios/actions

**Statut attendu :** En attente de vérification (doit être vert)

**Historique des fixes CI cette session :**
1. Commit `4d2666a` — push initial → ROUGE (register_protocol non-vérifié)
2. Commit `865a138` — suppression appels Lightyear non-vérifiés → ROUGE (bevy_ecs feature invalide)
3. Commit `88971ec` — suppression bevy_ecs feature + bevy de shared/ → EN ATTENTE

**Une fois CI vert :** lancer `/story-done S1-04` puis `/story-done S1-09`

---

## Découvertes critiques Bevy 0.18 (2026-04-29)

> Ces infos doivent être appliquées avant tout code Bevy

**liv-bevy-018 installé globalement :** `C:\Users\Sam\.claude\skills\liv-bevy-018\`
**liv-bevy-lightyear installé globalement :** `C:\Users\Sam\.claude\skills\liv-bevy-lightyear\`

### ✅ AUDIT COMPLET — 2026-04-29

Le skill liv-bevy-018 révèle que **EventWriter/EventReader n'existent plus en Bevy 0.18** :
- `EventWriter<T>` → `MessageWriter<T>`
- `EventReader<T>` → `MessageReader<T>`
- `app.add_event::<T>()` → `app.add_message::<T>()`

**AUDIT TERMINÉ — Toutes les violations corrigées :**
- `docs/architecture/adr-010-rsm-event-bus.md` — ✅ "Bevy buffered Messages (MessageWriter/MessageReader)"
- `docs/architecture/adr-009-rsm-phase-state.md` — ✅ EventReader/EventWriter → MessageReader/MessageWriter
- `docs/architecture/control-manifest.md` — ✅ Core Layer Rules mis à jour
- `docs/architecture/architecture.md` — ✅ Engine risk table corrigée
- `docs/architecture/adr-007-placement-buffer.md` — ✅ TODO(liv-bevy-018) ajouté
- `docs/architecture/adr-004-asset-loading-pipeline.md` — ✅ TODO(liv-bevy-018) ajouté
- `docs/architecture/adr-008/011/012-*.md` — ✅ Sections ⚠️ API Verification Required ajoutées
- Toutes les stories RSM, GSS, Economy, CardPool — ✅ MessageWriter/MessageReader
- `server/Cargo.toml` — ✅ TODO feature verification ajouté
- `client/Cargo.toml` — ✅ TODO feature collection verification ajouté
- `server/src/main.rs` — ✅ Commentaire "bevy_ecs" corrigé

### Lightyear 0.26 — API non-vérifiée

- Lightyear 0.26 utilise un **entity-per-connection model** (depuis v0.25)
- L'ancienne API resource-based (ClientConfig, ClientConnectionManager) n'existe plus
- **Aucun code Lightyear ne peut être écrit avant S1-05** (spike de vérification)
- S1-05 doit lire `api_patterns.md` dans `C:\Users\Sam\.claude\skills\liv-bevy-lightyear\`

### Features Bevy 0.18 valides

- `"bevy_ecs"` **n'est PAS** une feature valide dans Bevy 0.18
- Server headless : `bevy = { default-features = false, features = ["multi_threaded"] }`
- Client 2D : `bevy = { features = ["2d"] }` (collection haute-niveau Bevy 0.18)
- `EventWriter`/`EventReader` n'existent plus → `MessageWriter`/`MessageReader`

---

## Prochaines étapes (dans l'ordre)

### Immédiat
1. ✅ CI vert sur commit `88971ec` — vérifié (run 25130998038)
2. ✅ `/story-done S1-04` — COMPLETE WITH NOTES (2026-04-29)
3. ✅ `/story-done S1-09` — Done (déjà marqué)

### ✅ Audit Bevy 0.18 TERMINÉ
4. Audit complet fait — toutes violations EventWriter/EventReader corrigées en MessageWriter/MessageReader
   Lightyear ADRs annotés avec ⚠️ API Verification Required

### Premier vrai code de jeu (pas de gate Lightyear)
5. `/dev-story production/epics/round-state-machine/story-001-state-and-events-scaffold.md`
   → Story prête : ACs corrigés avec MessageWriter/MessageReader/#[derive(Message)]

### Gate Lightyear (bloque tout le networking)
6. `/dev-story production/epics/lightyear-protocol-verification/story-001-...`
   → S1-05 ⭐ — rien de networking avant que ce spike soit Done

---

## Epics créés

### Foundation (Sprint 1)
- `production/epics/workspace-and-shared-types/` — 4 stories
- `production/epics/game-config-pipeline/` — 4 stories
- `production/epics/server-rng/` — 3 stories
- `production/epics/lightyear-protocol-verification/` — 4 stories ⭐

### Core (Sprint 2+)
- `production/epics/round-state-machine/` — 6 stories
- `production/epics/game-session-system/` — 7 stories (story-004 Blocked ADR-012)
- `production/epics/economy-system/` — 6 stories
- `production/epics/card-data-pool/` — 6 stories

**Index complet :** `production/epics/index.md`

---

## Design — État GDDs

M1 (9 GDDs) : ✅ TOUS APPROUVÉS — prêts à implémenter
M2 (7 GDDs) : 2 DESIGNED, 5 PAS COMMENCÉS

**Auction System GDD :** `design/gdd/auction-system.md` — ✅ DESIGNED (2026-04-29). /design-review pending (fresh session).
**Combat Resolution GDD :** `design/gdd/combat-resolution.md` — ✅ DESIGNED (2026-04-29). Toutes sections complètes (A–H + Visual/Audio + UI Requirements + Open Questions). Registry: 2 nouvelles formules (net_damage, type_advantage). 5 OQs: OQ1 WALL ADR, OQ2 type advantage GameConfig, OQ3 RANGE RNG seed, OQ4 COUNTERATTACK proximity, OQ5 ResolutionEvent enum. /design-review pending (fresh session).
**Card Acquisition GDD :** `design/gdd/card-acquisition.md` — ✅ DESIGNED (2026-04-29). /design-review pending (fresh session).

### ✅ TERMINÉ — Shop/Auction UI GDD
- **Fichier :** `design/gdd/shop-auction-ui.md`
- **Statut :** Designed (2026-04-29) — sections A–H + Visual/Audio + UI Requirements + Open Questions
- **Registry :** 1 nouvelle formule (local_free_gold); 6 referenced_by mis à jour
- **5 OQs :** bid text input (OQ1), tooltip persistence (OQ2), S2CGoldBroadcast reserved_gold NP update (OQ3), screen layout split (OQ4), C2SSignalReady NP registration (OQ5)
- **Next :** /design-review design/gdd/shop-auction-ui.md (fresh session) · /ux-design shop-auction-ui

---

## Outils importants

```bash
# CI GitHub
https://github.com/SamyAnisBenachi/Claude-Code-Game-Studios/actions

# Cargo (Windows)
# Run from Developer PowerShell for VS 2026 so MSVC link.exe is on PATH.
# .cargo/config.toml sets target-dir = "target/msvc-local".
C:\Users\Sam\.cargo\bin\cargo.exe check --workspace
C:\Users\Sam\.cargo\bin\cargo.exe test -p server --verbose
C:\Users\Sam\.cargo\bin\cargo.exe test -p server session_ready_observer

# gh CLI (installé, besoin auth)
C:\Program Files\GitHub CLI\gh.exe

# Rust installé via winget 2026-04-29
# Normal PowerShell still will not see MSVC link.exe; use Developer PowerShell for VS 2026 or CI.
```

---

## Session Extract — /story-done 2026-04-29

- **Verdict**: COMPLETE WITH NOTES
- **Story**: `production/epics/workspace-and-shared-types/story-004-protocol-skeleton-ci-gates.md` — Protocol Skeleton & CI Dependency Gates
- **Passing ACs**: 5/11 — dep gates (shared/client/server), WASM size, protocol type stubs
- **Advisory deviations**: register_protocol() absent from shared/ (ADR-003 fallback; deferred to S1-05); evidence collected via CI rather than local builds
- **Tech debt logged**: None formally — deferred ACs tracked in story Completion Notes
- **Next recommended**: S1-05 — Lightyear 0.26 Verification Spike (now unblocked) at `production/epics/lightyear-protocol-verification/story-001-lightyear-026-verification-spike.md`

## Session Extract — /story-done 2026-04-29
- **Verdict**: COMPLETE
- **Story**: `production/epics/server-rng/story-001-type-definitions-audit-infrastructure.md` — ServerRng Type Definitions & Audit Infrastructure
- **Passing ACs**: 13/13
- **Deviations**: None
- **Test Evidence**: Logic — `tests/unit/foundation/server_rng_types_test.rs` (5 tests, CI green commit 6bdee76)
- **Tech debt logged**: None
- **Next recommended**: S1-10 — Intent-Named API & Consumption Invariants at `production/epics/server-rng/story-002-intent-named-api-invariants.md` (S1-09 unblocks it)

## Session Extract — /dev-story 2026-04-29
- **Story**: `production/epics/game-config-pipeline/story-001-asset-data-files.md` — Asset Data Files
- **Files changed**: `assets/config/game_config.ron` (3 network timeout values corrected to GDD design-intent), `assets/data/cards.json` (fixed serde newtype bug: `"id": [N]` → `"id": N` on all 8 entries)
- **Test written**: None — Config/Data story; evidence at `tests/evidence/story-gcp-001-data-files.md`
- **Blockers**: None
- **Status**: Complete
- **Next**: `/story-done production/epics/game-config-pipeline/story-001-asset-data-files.md` or continue with next ready story

## Session Extract — /dev-story 2026-04-29
- **Story**: `production/epics/lightyear-protocol-verification/story-001-lightyear-026-verification-spike.md` — Lightyear 0.26 Verification Spike
- **Files changed**:
  - `server/tests/session_ready_observer_test.rs` — ADR-012 open condition test (2 test functions)
  - `tests/evidence/lightyear-026-verification.md` — all 20 items annotated; ADR-012 items 15-16 now locally verified
  - `docs/architecture/control-manifest.md` — §Lightyear 0.26 Verification Checklist: all 20 ⬜ → ✅/⚠️
- **Test written**: `server/tests/session_ready_observer_test.rs` — 2 tests (ADR-012 open condition); now verified locally from Developer PowerShell for VS 2026
- **Key findings**: 7 API differences from ADR assumptions (channel syntax, direction model, send/receive methods, NetworkTarget identifier type, server send API, connection event naming); all have concrete resolution paths documented
- **Blockers**: None for ADR-012 items 15-16; `cargo test -p server session_ready_observer` passes from Developer PowerShell for VS 2026
- **Next**: Networking follow-up stories may rely on the resolved ADR-012 observer ordering test

## Session Extract — /dev-story 2026-04-29
- **Story**: `production/epics/card-data-pool/story-001-pool-state-core-api.md` — Pool State + Core API
- **Files changed**:
  - `shared/src/session.rs` — created; `PlayerId(u64)` type
  - `shared/src/lib.rs` — added `pub mod session;`
  - `server/src/core/pool/state.rs` — created; `PlayerPool`, `PlayerPools`, `DistributeError`, `PoolFilter` structs
  - `server/src/core/pool/api.rs` — created; `impl PlayerPool` (initialize, distribute, is_available, copies_remaining, total_acquired) + 20 embedded `#[cfg(test)]` tests
  - `server/src/core/pool/plugin.rs` — created; `CardPoolPlugin` skeleton (registers `PlayerPools`)
  - `server/src/core/pool/mod.rs` — created; module re-exports
  - `server/src/core/mod.rs` — added `pub mod pool;`
  - `tests/unit/pool/pool_state_test.rs` — created; evidence documentation (20 test cases mapped to ACs 1–10)
- **Test written**: 20 `#[cfg(test)]` tests in `server/src/core/pool/api.rs`; run via `cargo test -p server`
- **Blockers**: Local builds blocked by Smart App Control — CI is verification gate
- **Next**: `/code-review server/src/core/pool/api.rs server/src/core/pool/state.rs` then `/story-done production/epics/card-data-pool/story-001-pool-state-core-api.md`

## Session Extract — /dev-story 2026-04-29
- **Story**: `production/epics/server-rng/story-003-determinism-session-reset.md` — Determinism Proof & Session Reset (S1-12)
- **Files changed**:
  - `server/src/foundation/rng.rs` — added `PartialEq` to `AuditEntry` derive; added `at_max_seed_index()` test-only constructor; added deferred-AC comments (RNG8/9/10/14); added 7 new Story 003 tests embedded in `#[cfg(test)] mod tests`
  - `tests/unit/foundation/server_rng_determinism_test.rs` — created; Story 003 evidence documentation
- **Test written**: 7 embedded `#[cfg(test)]` tests in `rng.rs`: 2× determinism (VC1/VC2), 2× session reset (RNG13), 3× overflow (RNG15)
- **Blockers**: Local build blocked by Smart App Control (pre-existing) — CI is verification gate
- **Next**: `/code-review server/src/foundation/rng.rs` then `/story-done production/epics/server-rng/story-003-determinism-session-reset.md`

## Session Extract — /dev-story 2026-04-29
- **Story**: `production/epics/server-rng/story-002-intent-named-api-invariants.md` — Intent-Named API & Consumption Invariants
- **Files changed**:
  - `server/src/foundation/rng.rs` — refactored `next_seed()` to private no-param helper; added 7 intent-named public methods; added `# Ordering Contract` doc-comment on `ServerRng`; embedded `#[cfg(test)]` module with 10 tests covering both Story 001 (updated) and Story 002 ACs
  - `tests/unit/foundation/server_rng_types_test.rs` — converted to evidence documentation (Story 001 tests now embedded in rng.rs)
  - `tests/unit/foundation/server_rng_api_test.rs` — created; Story 002 evidence documentation
- **Test written**: Embedded `#[cfg(test)] mod tests` in `rng.rs` (10 tests; run via `cargo test -p server`)
- **Blockers**: Local build blocked by Smart App Control — CI is verification gate (same as prior stories)
- **Next**: `/code-review server/src/foundation/rng.rs` then `/story-done production/epics/server-rng/story-002-intent-named-api-invariants.md`

## Session Extract — /dev-story 2026-04-29
- **Story**: `production/epics/game-config-pipeline/story-002-asset-loading-pipeline.md` — Asset Loading Pipeline (S1-07)
- **Files changed**:
  - `server/src/foundation/config.rs` — NEW: full pipeline (AppState, GameConfigAsset, GameConfigLoader, CardCatalog struct, CardCatalogLoader, GameAssets, start_loading, check_loading_done, validate_and_promote stub, ConfigPlugin)
  - `server/src/foundation/mod.rs` — added `pub mod config;`
  - `server/src/main.rs` — added AssetPlugin + ConfigPlugin to App builder
  - `server/Cargo.toml` — added bevy features `bevy_asset`, `bevy_state`; added `thiserror = "1"`
  - `shared/src/config.rs` — added 3 missing auction floor fields (auction_floor_rare/epic/legendary)
- **Test written**: None — Integration story; evidence at `tests/evidence/story-gcp-002-pipeline.md`
- **Key deviation**: bevy_asset_loader unavailable for 0.18 (PR #264 draft) — manual AssetServer polling used
- **Blockers**: Local build blocked by Smart App Control — CI is verification gate
- **Next**: `/code-review server/src/foundation/config.rs` then `/story-done production/epics/game-config-pipeline/story-002-asset-loading-pipeline.md`

## Session Extract — /story-done 2026-04-29
- **Verdict**: COMPLETE
- **Story**: `production/epics/game-config-pipeline/story-001-asset-data-files.md` — Asset Data Files (S1-06)
- **Criteria**: 8/8 passing — all ACs auto-verified (file reads + evidence doc)
- **Deviations**: None — manifest version match (2026-04-29)
- **Tech debt logged**: None
- **Next recommended**: S1-07 — Asset Loading Pipeline at `production/epics/game-config-pipeline/story-002-asset-loading-pipeline.md` (now unblocked: S1-06 Done)

## Session Extract — /architecture-review 2026-04-30
- **Verdict**: CONCERNS — M1 PASS / M2+M3 FAIL (ADRs not yet authored)
- **Requirements**: 80 M1 TRs — 80 covered (100%); ~120 M2/M3 gaps (0 ADRs exist)
- **New TR-IDs registered**: 80 M1 TRs in tr-registry.yaml (populated by another agent; M2/M3 deferred until ADRs authored)
- **ADR-013 authored**: auction-system-state.md (Proposed) — covers TR-AU-001..010
- **Engine fixes applied**: current-best-practices.md + ADR-003 (invalid bevy_ecs feature removed); ADR-008 + ADR-009 (stale "pending" dependency notes removed)
- **Top ADR gaps**: ADR-014 (Combat Resolution Sub-step Scheduler), ADR-015 (Card Acquisition + Hand Management), ADR-023 (Keyword Observer Architecture)
- **Report**: docs/architecture/architecture-review-2026-04-30.md

## Session Extract — /story-done 2026-04-29
- **Verdict**: COMPLETE
- **Story**: `production/epics/server-rng/story-002-intent-named-api-invariants.md` — Intent-Named API & Consumption Invariants (S1-10)
- **Criteria**: 11/11 passing — all ACs auto-verified (code read + test traceability)
- **Deviations**: None — ADR-005 compliant, manifest version match (2026-04-29)
- **Test Evidence**: Logic — 10 embedded `#[cfg(test)]` tests in `server/src/foundation/rng.rs`; evidence doc at `tests/unit/foundation/server_rng_api_test.rs`
- **Code Review**: APPROVED (lean mode)
- **Tech debt logged**: None
- **Next recommended**: S1-12 — Determinism Proof & Session Reset at `production/epics/server-rng/story-003-determinism-session-reset.md` (S1-10 now Done, blocker cleared)

## Session Extract — /architecture-review 2026-04-30
- **Verdict**: CONCERNS — M1 PASS / M2+M3 FAIL (ADRs not yet authored)
- **Requirements**: ~217 total — ~95 covered, ~51 partial, ~71 gaps
- **New TR-IDs registered**: 74 (TR-CDP-001..009, TR-GC-001..005, TR-RNG-001..006, TR-ECO-001..008, TR-BLS-001..010, TR-RSM-001..010, TR-NP-001..012, TR-GSS-001..010, TR-OBJ-001..010)
- **GDD revision flags**: None — all GDD design assumptions consistent with verified engine behaviour
- **Fixes applied**: ADR-004 invalid Bevy code samples replaced; ADR-005 stale dep reference fixed (ADR-007 → ADR-012)
- **Top ADR gaps**: ADR-013 Auction State Machine, ADR-014 Combat Sub-Step Scheduling, ADR-015 Keyword Observer Architecture
- **Report**: docs/architecture/architecture-review-2026-04-30.md
- **Files changed**: architecture-review-2026-04-30.md (new), architecture-traceability.md (updated), tr-registry.yaml (74 TRs populated), adr-004 (code samples fixed), adr-005 (dep reference fixed)

## Session Extract — /design-review network-protocol.md R3 2026-04-30
- **Verdict**: MAJOR REVISION NEEDED → Revised inline
- **Blockers resolved**: 17
- **Key fixes**: ResolutionEvent 10 new variants + 3 new enums; BoardSnapshot seeds + sinistros; PlayerSnapshot class_id; UnitBoardState 7 keyword state fields + max_hp; GoldAwardReason::PrismReward removed; AcquisitionSource rename + FreeCardPick; S2COpponentSubmitted; AuctionSnapshot starting_price; NP-19/NP-23/NP-25/NP-26 rewritten; NP-30–35 new ACs; OQ5 closed
- **Systems index**: Network Protocol → In Review
- **Recommended next**: /design-review design/gdd/network-protocol.md --depth lean (R4 re-review, fresh session after /clear)
- **Also still Needs Revision**: keyword-system.md (C-B2), class-system.md (D-B1/D-B2), auction-system.md (D-B3), objective-system.md (D-B1), hand-ui.md (C-B4/5), entities.yaml (C-B6)

## Session Extract — /review-all-gdds 2026-04-30
- **Verdict**: FAIL
- **GDDs reviewed**: 20 (11 new since R5)
- **Blocking issues**: 9 (C-B1 S2CResolutionEvent variants; C-B2 GameOverReason enum; C-B3 S2CCardAcquired schema; C-B4 Hand UI activation lock; C-B5 GameConfig missing fields; C-B6 S2CSangMepriseReveal unregistered; D-B1 Garde-Temps routing; D-B2 Sang Méprise+Punition ordering; D-B3 Auction wealth gap mitigation)
- **Flagged for revision**: network-protocol.md, keyword-system.md, class-system.md, hand-ui.md, auction-system.md, objective-system.md, entities.yaml
- **Systems index updated**: 6 GDDs marked Needs Revision (Network Protocol, Objective System, Auction System, Hand UI, Keyword System, Class System)
- **Report**: design/gdd/gdd-cross-review-2026-04-30.md
- **Recommended next**: /design-review each flagged GDD starting with network-protocol.md (C-B1/B2/B3 are the highest-leverage blockers — they unblock Combat Resolution, Keyword System, and Card Animations implementation)

---

## TODO conditionnels (déclencher manuellement quand condition remplie)

- **Re-review board-rendering** quand `network-protocol.md` est mis à jour avec `C2SRequestSnapshot` : `/design-review design/gdd/board-rendering.md` (commande à lancer dans une fenêtre fraîche). Source : board-rendering R2 verdict CONDITIONAL APPROVED, OQ-BR-06 cross-doc dependency.
- **Re-review auction-system** après mise à jour NP `reserved_gold` (si pas déjà fait) : `/design-review design/gdd/auction-system.md`
- **Re-review combat-resolution** quand NP ajoute variants `CombatDamage` + `KeywordTriggered` à `ResolutionEvent` enum (OQ5 combat-resolution).

## Session Extract — /review-all-gdds 2026-04-30 R7
- Verdict: FAIL
- GDDs reviewed: 20 + master + systems-index
- Flagged for revision: network-protocol, keyword-system, class-system, objective-system, hand-ui, auction-system, entities.yaml, round-state-machine, prism-system, game-config, combat-resolution, economy-system, card-animations, board-rendering, server-rng, shop-auction-ui, lanes-and-lies-gdd
- Blocking issues (9): C-B2 ResolutionTimeout vs Draw; C-B4 hand-ui Rule 5c S2CCardAcquired; C-B5 hand-ui Tuning vs Dependencies contradiction; C-B6+C-NEW-1+C-NEW-3 registry single-pass; C-NEW-2 stale CardSource::AuctionWon; C-NEW-4 multi-Krosmic ordering not echoed by RSM/NP/CA; D-B1+C-NEW-5 Garde-Temps/Punition take_damage routing; D-B4 disconnect grace during DRAFT_AUCTION; D-B5 Sang Méprise reconnect-gap UX
- Resolved this cycle: C-B1 (S2CResolutionEvent variants), C-B3 partial (AcquisitionSource rename — variants still incomplete), C-W1
- Recommended next: address 9 blockers per file table in report (single registry pass closes 4 at once); /design-review re-runs for class-system, hand-ui, auction-system after fixes
- Report: design/gdd/gdd-cross-review-2026-04-30-r7.md

## Session Extract - /dev-story 2026-04-30
- Story: `production/epics/round-state-machine/story-001-state-and-events-scaffold.md` - State and Events Scaffold
- Files changed: `server/src/core/rsm/state.rs`, `server/src/core/rsm/events.rs`, `server/src/core/rsm/plugin.rs`, `server/src/core/rsm/mod.rs`, `server/src/core/mod.rs`, `server/src/lib.rs`, `server/src/main.rs`, `shared/src/protocol.rs`, `client/src/state/mod.rs`, `server/tests/rsm_scaffold_test.rs`, `tests/unit/rsm/rsm_scaffold_test.rs`, `tests/evidence/rsm-story-001-check.md`
- Test written: `server/tests/rsm_scaffold_test.rs` (2 tests), evidence mapping at `tests/unit/rsm/rsm_scaffold_test.rs`, smoke evidence at `tests/evidence/rsm-story-001-check.md`
- Blockers: previously blocked in normal PowerShell by missing MSVC `link.exe`; use Developer PowerShell for VS 2026 for local Cargo verification
- Next: `/code-review server/src/core/rsm/state.rs server/src/core/rsm/events.rs server/src/core/rsm/plugin.rs server/tests/rsm_scaffold_test.rs` then run Cargo in a VS Developer Command Prompt or CI

## Session Extract - /dev-story 2026-04-30
- Story: `production/epics/card-data-pool/story-002-weighted-draw-functions.md` - Weighted Draw Functions
- Files changed: `server/src/core/pool/api.rs`, `tests/unit/pool/weighted_draw_test.rs`
- Test written: embedded tests in `server/src/core/pool/api.rs` (11 S2-03 tests), evidence mapping at `tests/unit/pool/weighted_draw_test.rs`
- Verification: CI green on run 25161381682 for commit `013f204`
- Blockers: local `cargo test -p server weighted_draw --verbose` blocked by Windows/MSVC build-script failures before server tests ran; CI passed
- Next: S2-04 Card Pool Story 3: Refresh Shop Slot Variants is unblocked once sprint tracker is read fresh

## Session Extract - /story-done 2026-04-30
- Story: `production/epics/economy-system/story-001-state-and-pure-api-scaffold.md` - State & Pure API Scaffold
- Files changed: `server/src/core/economy/state.rs`, `server/src/core/economy/api.rs`, `server/src/core/economy/mod.rs`, `server/src/core/mod.rs`, `shared/src/config.rs`, `assets/config/game_config.ron`, `server/tests/game_config_defaults_test.rs`, `tests/unit/economy/state_api_test.rs`
- Test written: embedded tests in `server/src/core/economy/api.rs` (17 tests), evidence mapping at `tests/unit/economy/state_api_test.rs`
- Verification: CI green on run 25161623746 for commit `5d8655c`
- Blockers: local `cargo test -p server economy::api::tests` blocked before compile by missing Windows MSVC `link.exe`; CI passed
- Next: S2-08 Economy Story 2 is unblocked once S2-01 is Done

## Session Extract - /story-done 2026-04-30
- Story: `production/epics/round-state-machine/story-001-state-and-events-scaffold.md` - State and Events Scaffold
- Verdict: COMPLETE WITH NOTES
- Criteria: 14/14 passing in CI (current main run 25161983584)
- Deviations: Advisory only - Bevy 0.18 uses `App::add_observer(on_session_ready)` and `On<SessionReady>` instead of older `app.observe` / `Trigger` wording.
- Test Evidence: `server/tests/rsm_scaffold_test.rs`, `tests/unit/rsm/rsm_scaffold_test.rs`, `tests/evidence/rsm-story-001-check.md`
- Implementation Commit: `2b66f35`
- Next: S2-07 RSM Story 2 and S2-08 Economy Story 2 are unblocked now that S2-01 and S2-02 are Done

## Session Extract - environment update 2026-04-30
- Cargo local fixed: `.cargo/config.toml` sets `target-dir = "target/msvc-local"` so Cargo's generated build scripts/proc-macro DLLs stay in a repo-local target tree.
- Verified command: from Developer PowerShell for VS 2026, `cargo test -p server session_ready_observer` -> 2 passed; 0 failed.
- Scope: normal PowerShell still will not see MSVC `link.exe`; use Developer PowerShell for VS 2026 or CI for local verification.
- Docs updated: `CODEX.md`, `docs/architecture/control-manifest.md`, `tests/evidence/lightyear-026-verification.md`.
- Impact: ADR-012 Lightyear verification items 15-16 are no longer pending; GSS/RSM observer ordering can rely on the passing local test.

## Session Extract - workflow update 2026-04-30
- Worker Codex windows should use Developer PowerShell for VS 2026 for fast local Cargo iteration (`cargo test -p server ...`) before pushing.
- Workers should push after local tests pass and hand off commit hash + CI run id/details; they do not need to wait on GitHub Actions by default.
- The orchestrator window owns GitHub Actions monitoring, routing failures back to owners, and final `story-done` tracking once CI is green.

## Session Extract - /story-done 2026-04-30
- Verdict: COMPLETE
- Story: `production/epics/lightyear-protocol-verification/story-002-all-protocol-message-types.md` - All Protocol Message Types
- Criteria: Protocol catalogue implemented in `shared/src/protocol.rs`; worker local checks passed (`cargo fmt --check`, `cargo check -p shared`, `cargo test -p shared`, extra `cargo check -p server`).
- Implementation Commit: `759bd4a`; CI run `25169319842` passed.
- Notes: `shared/` remains dependency-pure; server/client plugin story owns adapting the protocol manifest to concrete Lightyear registration calls.
- Tech debt logged: None
- Next recommended: S2-05 Startup Validation Gate and S2-09 Server & Client Network Plugins are now ready-for-dev in `production/sprint-status.yaml`.

## Session Extract - sprint tracker refresh 2026-04-30
- Sprint 2 must-haves S2-01, S2-02, and S2-03 are Done with green CI.
- Unblocked Sprint 2 pull-forward stories are now claimable in `production/sprint-status.yaml`: S2-04 Card Pool Refresh Shop Slot Variants, S2-07 RSM advance_phase + F2 Ordering, and S2-08 Economy Initialisation + Draft Subscriber.
- Workers may use `implement next` in separate Codex windows; each worker must claim exactly one ready story, run local Cargo tests from Developer PowerShell for VS 2026, commit/push its own files, and hand off CI details to the orchestrator.

## Session Extract - /dev-story 2026-04-30
- Story: `production/epics/economy-system/story-002-initialisation-draft-subscriber.md` - Initialisation & DraftStarted Subscriber
- Owner: `codex-s2-08-economy`
- Files changed: `server/src/core/economy/api.rs`, `server/src/core/economy/mod.rs`, `server/src/core/economy/plugin.rs`, `server/src/core/economy/system.rs`, `server/src/core/session/mod.rs`, `server/src/core/session/state.rs`, `server/src/core/mod.rs`, `server/src/lib.rs`, `server/src/main.rs`, `server/tests/economy_draft_subscriber_test.rs`, `server/tests/economy_round_trace_test.rs`, `tests/unit/economy/draft_subscriber_test.rs`, `tests/integration/economy/round_trace_test.rs`
- Test written: `server/tests/economy_draft_subscriber_test.rs` (7 tests) and `server/tests/economy_round_trace_test.rs` (1 test); evidence mappings at `tests/unit/economy/draft_subscriber_test.rs` and `tests/integration/economy/round_trace_test.rs`
- Verification: `CARGO_INCREMENTAL=0 cargo test -p server --test economy_draft_subscriber_test --test economy_round_trace_test --verbose` -> 8 passed; cargo global cache last-use warning only
- Blockers: None for S2-08 local tests. Scheduling `.after(advance_phase)` remains dependent on S2-07's in-progress RSM transition symbol landing.
- Next: `/code-review server/src/core/economy/system.rs server/src/core/economy/plugin.rs server/src/core/session/state.rs server/tests/economy_draft_subscriber_test.rs server/tests/economy_round_trace_test.rs` then `/story-done production/epics/economy-system/story-002-initialisation-draft-subscriber.md` after CI green

## Session Extract - /story-done 2026-04-30
- Verdict: COMPLETE
- Story: `production/epics/round-state-machine/story-002-advance-phase-and-f2-ordering.md` - Advance Phase and F2 Ordering
- Criteria: 16/16 passing; `server/tests/rsm_transitions_test.rs` covered by CI run `25167672501`
- Implementation Commit: `cb550b9`
- Tech debt logged: None
- Next recommended: RSM Story 003 once sprint planning pulls it forward.

## Session Extract - /story-done 2026-04-30
- Verdict: COMPLETE
- Story: `production/epics/economy-system/story-002-initialisation-draft-subscriber.md` - Initialisation & DraftStarted Subscriber
- Criteria: 12/12 passing; `server/tests/economy_draft_subscriber_test.rs` and `server/tests/economy_round_trace_test.rs` covered by CI run `25167672501`
- Implementation Commit: `9396d32`; repair commit `e4ac84e` restored S2-08 files after S2-04 scope cleanup and fixed config doctests
- Tech debt logged: None
- Next recommended: Economy Story 003 once sprint planning pulls it forward.

## Session Extract - /story-done 2026-04-30
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/card-data-pool/story-003-refresh-shop-slot-variants.md` - refresh_shop + Slot Variants
- Criteria: 4/4 passing; refresh-shop tests embedded in `server/src/core/pool/api.rs` covered by CI run `25167672501`
- Implementation Commit: `901823d`; repair commit `e4ac84e` fixed integration history and config doctests
- Notes: Implementation uses weighted draw selection directly; future class/neutral split policy remains with the subscriber story as scoped.
- Tech debt logged: None
- Next recommended: Card Pool Story 004 / ShopRefreshNeeded subscriber once sprint planning pulls it forward.

## Session Extract — /story-done 2026-04-30
- Verdict: COMPLETE (re-verification pass — story was already marked Complete by Codex)
- Story: `production/epics/round-state-machine/story-002-advance-phase-and-f2-ordering.md` — advance_phase + F2 Ordering
- Action: AC checkboxes checked off (16/16); Status and Completion Notes already correct; sprint-status already `done`
- Tech debt logged: None
- Next recommended: All Must Have stories done. Sprint close-out sequence: `/smoke-check sprint` → `/team-qa sprint` → `/gate-check`

## Session Extract — /architecture-review 2026-04-30 Run 2
- Verdict: CONCERNS — M1 PASS / Core M2/M3 CONCERNS / Presentation FAIL
- ADRs reviewed: 18 (ADR-001..012 Accepted, ADR-013..018 Proposed)
- Requirements: 170 total — 126 covered (74%), 9 partial (5%), 35 gaps (21%)
- Blocking issues: 6 — B-1 ADR-015 H1 duplicate "ADR-014"; B-2 hand storage triple-named; B-3 EconomyState undefined (needs ADR-019); B-4 ADR-005/006 amendments missing; B-5 ADR-010 Prism row missing; B-6 ADR-013 template error
- New TR-IDs registered: 0 (M2/M3 TRs documented but not yet appended to tr-registry.yaml; pending ADR Acceptance)
- GDD revision flags: None (engine compatibility clean on new ADRs)
- Top required ADRs: ADR-019 (Economy Resource), ADR-020 (Presentation Layer), ADR-006 amendment, ADR-005 amendment
- Report: docs/architecture/architecture-review-2026-04-30b.md
- Traceability index updated: docs/architecture/architecture-traceability.md

## Session Extract - /story-done 2026-04-30
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/lightyear-protocol-verification/story-003-server-client-network-plugins.md` - Server & Client Network Plugins
- Criteria: 16/16 passing; server/client network plugins, shared protocol registration, C2S receiver stubs, client sender stubs, and ADR-001 unicast compile-proof verified against implementation and CI.
- Test Evidence: `tests/evidence/story-lyv-003-server-check.md`, `tests/evidence/story-lyv-003-client-check.md`, and GitHub Actions run `25176947506` passed for commit `215253e4eb1c234233459a9e742e06fd429ad4bb`.
- Local Verification: `cargo check -p server` passed locally with pre-existing warnings outside S2-09 scope. `cargo check -p client` was attempted locally and blocked by Windows dependency compilation/OOM/timeout before reaching the client crate; CI is the authoritative green build.
- Tech debt logged: None
- Next recommended: S2-10 E2E WebSocket Round-Trip at `production/epics/lightyear-protocol-verification/story-004-e2e-websocket-roundtrip.md` is unblocked by S2-09 completion.

## Session Extract - /story-done 2026-04-30
- Verdict: COMPLETE
- Story: `production/epics/game-config-pipeline/story-003-startup-validation-gate.md` - Startup Validation Gate
- Criteria: 22/22 passing; validation tests mapped in `tests/unit/foundation/game_config_validation_test.rs` and embedded in `server/src/foundation/config.rs`
- Verification: CI green on main run `25176947506`; local `cargo test -p server game_config` attempted from normal PowerShell but failed before story tests due Windows resource/toolchain metadata errors
- Tech debt logged: None
- Next recommended: S2-09 Server & Client Network Plugins is in progress; after it completes, S2-10 E2E WebSocket Round-Trip is blocked on S2-09 completion

## Session Extract - /dev-story 2026-04-30
- Story: `production/epics/auction-system/story-001-auction-state-scaffold.md` - AuctionState Types & Snapshot Scaffold
- Owner: `claude-auc-001-auction-state`
- Files changed: `server/src/feature/auction/state.rs`, `server/src/feature/auction/snapshot.rs`, `server/src/feature/auction/mod.rs`, `server/src/feature/mod.rs`, `server/src/lib.rs`, `server/Cargo.toml`, `tests/unit/auction/auction_state_scaffold_test.rs`, `production/sprint-status.yaml`
- Test written: `tests/unit/auction/auction_state_scaffold_test.rs` (7 executable tests; Cargo-wired as `auction_state_scaffold_test`)
- Verification: Visual Studio developer environment via `VsDevCmd.bat`, `cargo test -p server --test auction_state_scaffold_test` -> 7 passed; `rustfmt --check` passed for AUC-001 files. Workspace `cargo fmt --check` is blocked by unrelated formatting drift in `server/src/core/rsm/transitions.rs`, `server/src/lobby/handler.rs`, and `server/src/network/mod.rs`.
- Notes: Snapshot scaffold follows current `network-protocol.md` by carrying `starting_price` in addition to ADR-013's original fields.
- Blockers: None
- Next: `/code-review server/src/feature/auction/state.rs server/src/feature/auction/snapshot.rs tests/unit/auction/auction_state_scaffold_test.rs` then `/story-done production/epics/auction-system/story-001-auction-state-scaffold.md` after CI green

## Session Extract - /dev-story 2026-04-30
- Story: `production/epics/class-system/story-001-class-lifecycle.md` - Class Lifecycle / PlayerSessions Scaffold
- Owner: `claude-cs-001-class-lifecycle`
- Files changed: `shared/src/protocol.rs`, `server/src/core/session/state.rs`, `server/src/core/session/plugin.rs`, `server/src/core/session/snapshot.rs`, `server/src/core/session/mod.rs`, `server/src/core/rsm/events.rs`, `server/src/core/rsm/plugin.rs`, `server/src/core/rsm/transitions.rs`, `server/src/lobby/handler.rs`, `server/src/lobby/mod.rs`, `server/src/network/mod.rs`, `server/src/main.rs`, `server/src/lib.rs`, `server/tests/class_lifecycle_test.rs`, `server/tests/rsm_scaffold_test.rs`, `server/tests/rsm_transitions_test.rs`, `tests/unit/class/class_lifecycle_test.rs`, `production/sprint-status.yaml`
- Test written: `server/tests/class_lifecycle_test.rs` (9 tests); evidence mapping at `tests/unit/class/class_lifecycle_test.rs`
- Verification: `cargo test -p server --test class_lifecycle_test` -> 9 passed; `cargo test -p server --test rsm_transitions_test --test rsm_scaffold_test` -> 16 passed; `cargo check --workspace` passed with pre-existing warnings.
- Notes: `systems-index` still reports Class System as In Review, and `control-manifest.md` still lists ADR-014 as pending despite ADR-014 and the epic/story being Ready/Accepted. Shared protocol remains dependency-light, so `C2SClassChoice` follows the existing serde-only Lightyear registration manifest rather than adding a `lightyear` dependency to `shared/`.
- Blockers: None
- Next: `/code-review server/src/core/session/state.rs server/src/lobby/handler.rs server/src/core/rsm/transitions.rs server/tests/class_lifecycle_test.rs` then `/story-done production/epics/class-system/story-001-class-lifecycle.md` after CI green

## Session Extract - /story-done 2026-05-01
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/class-system/story-001-class-lifecycle.md` - Class Lifecycle / PlayerSessions Scaffold
- Criteria: 3/3 passing; class choice, lobby gate locking, and snapshot `class_id` verified against current code.
- Test Evidence: `tests/unit/class/class_lifecycle_test.rs`; executable coverage in `server/tests/class_lifecycle_test.rs`.
- Verification: `cargo test -p server --test class_lifecycle_test` -> 9 passed; `cargo test -p server --test rsm_transitions_test --test rsm_scaffold_test` -> 16 passed. GitHub Actions run `25194696023` passed on `main` at `17e3fc352ad1f843daafba4fa8ac484847311f9e`; implementation commit `91539e1` is an ancestor.
- Notes: Advisory only - story manifest `2026-04-30` is older than current control manifest `2026-05-01`; shared protocol remains serde-only/dependency-light rather than deriving Lightyear `Message` directly in `shared/`.
- Tech debt logged: None
- Next recommended: Continue in-progress Sprint 3 must-haves (S3-04, S3-06, AUC-001) before pulling dependent ready-for-dev stories.

## Session Extract - /story-done 2026-05-01
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/auction-system/story-001-auction-state-scaffold.md` - AuctionState Types & Snapshot Scaffold
- Criteria: 5/5 passing; `tests/unit/auction/auction_state_scaffold_test.rs` passed locally with `cargo test -p server --test auction_state_scaffold_test` (7/7 tests).
- Verification: implementation commit `b7180f6` is included in green main CI run `25194696023` (`17e3fc352ad1f843daafba4fa8ac484847311f9e`, success).
- Notes: Advisory only - story manifest v2026-04-30 is older than current control manifest v2026-05-01; no blocking drift found. Snapshot includes `starting_price` to match current `network-protocol.md`.
- Tech debt logged: None
- Next recommended: AUC-002 Auction Phase Entry (`production/epics/auction-system/story-002-auction-phase-entry.md`) or AUC-003 AbortAuction Handler (`production/epics/auction-system/story-003-auction-abort-handler.md`) are ready.

## Session Extract -- /story-done 2026-05-01
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/game-session-system/story-001-lobby-scaffold.md` -- Lobby Scaffold
- Criteria: 10/10 passing; `server/tests/session_scaffold_test.rs` passed locally (9/9), with evidence pointer at `tests/unit/session/scaffold_test.rs`
- Verification: `cargo check -p server` passed locally with pre-existing warnings outside S3-01; GitHub Actions run `25194696023` passed for commit `17e3fc352ad1f843daafba4fa8ac484847311f9e`
- Notes: Story manifest 2026-04-29 is older than control manifest 2026-05-01; no blocking deviations found
- Tech debt logged: None
- Next recommended: S3-02 Room Create and Join at `production/epics/game-session-system/story-002-room-create-join.md`

## Session Extract - /story-done 2026-05-01
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/auction-system/story-002-auction-phase-entry.md` - Auction Phase Entry
- Criteria: 3/3 passing; AU1-a/AU1-b-server/AU23 verified against `auction_tick_system` and `AuctionPhaseEntered` message handling.
- Test Evidence: `tests/unit/auction/auction_phase_entry_test.rs`; `cargo test -p server --test auction_phase_entry_test` -> 4 passed, 0 failed.
- Notes: Advisory only - story manifest v2026-04-30 is older than current control manifest v2026-05-01; AU23 logging is implemented via `tracing::error!` but the current unit test does not capture/assert the log event directly; `S2CAuctionCard` uses the server-side Bevy message shim until later Lightyear dispatch stories.
- Tech debt logged: None
- Sprint status: `AUC-002` set to `done` in `production/sprint-status.yaml`; existing in-progress claims for `S3-02` and `KW-001` preserved.
- Next recommended: AUC-003 AbortAuction Handler (`production/epics/auction-system/story-003-auction-abort-handler.md`) and AUC-004 Bid Validation Gate (`production/epics/auction-system/story-004-bid-validation-gate.md`) are ready candidates.

## Session Extract - /story-done 2026-05-01
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/keyword-system/story-001-module-scaffold.md` - Keyword System Module Scaffold
- Criteria: 11/11 passing; scaffold files, keyword component, observer/message/resource registration, protocol keyword payload types, stubs, and smoke evidence verified.
- Test Evidence: `tests/integration/keyword/plugin_smoke_test.rs`; `cargo test -p server --test keyword_plugin_smoke_test` -> 2 passed, 0 failed. `cargo check --workspace` passed with warnings only.
- Notes: Advisory only - protocol keyword types live in `shared/src/keyword.rs` in this three-crate workspace; `docs/architecture/tr-registry.yaml` has stale wording for TR-KW-006 and TR-KW-012 and was intentionally not edited.
- Tech debt logged: None
- Sprint status: `KW-001` set to `done` in `production/sprint-status.yaml`; existing in-progress claims for `S3-02`, `S3-04`, `S3-06`, and `S3-08` preserved.
- Next recommended: Keyword Story 002 Movement Formulas at `production/epics/keyword-system/story-002-movement-formulas.md` after readiness check.

## Session Extract - /story-done 2026-05-01
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/game-session-system/story-002-room-create-join.md` - Room Create and Join
- Criteria: 7/7 passing; create room, join room, ActiveSessions guard, room code generation, plugin registration, `cargo check -p server`, and integration coverage verified.
- Test Evidence: `tests/integration/session/room_create_join_test.rs`; `cargo test --test room_create_join_test` -> 7 passed, 0 failed; `cargo check -p server` passed with zero warnings.
- Notes: Advisory only - story manifest v2026-04-29 is older than current control manifest v2026-05-01. Current GDD/registry supersedes the story wording for slot updates: joiner receives only `S2CJoinAck`; existing occupants receive `S2CSlotUpdated`; implementation matches the current rule.
- Tech debt logged: None
- Sprint status: `S3-02` set to `done` in `production/sprint-status.yaml`; existing in-progress claims for `S3-04`, `S3-06`, and `S3-08` preserved; `S3-03` preserved as `ready-for-dev`.
- Next recommended: S3-03 Class Selection and Reveal at `production/epics/game-session-system/story-003-class-selection-reveal.md` after readiness check.

## Session Extract - /story-done 2026-05-01
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/game-session-system/story-003-class-selection-reveal.md` - Class Selection and Reveal
- Criteria: 6/6 passing; select preview, confirm lock, deferred reveal, idempotence/re-lock handling, plugin wiring, and class preview resource verified.
- Test Evidence: `tests/unit/session/class_reveal_test.rs`; `cargo test -p server --test class_reveal_test` -> 8 passed, 0 failed. `cargo check -p server` passed with zero warnings.
- Notes: Advisory only - story manifest v2026-04-29 is older than current control manifest v2026-05-01. Story rejection-message wording is stale; current implementation follows current GDD/protocol with `S2CConfirmClassRejected { reason: ClassAlreadyConfirmed }` and silent same-class duplicate confirms.
- Tech debt logged: None
- Sprint status: `S3-03` set to `done` in `production/sprint-status.yaml`; existing in-progress claims for `S3-04`, `S3-06`, and `S3-08` preserved.
- Next recommended: S3-05 RSM Story 4: Win Condition and Game Over at `production/epics/round-state-machine/story-004-win-condition-and-game-over.md`; S3-04 and S3-06 are still in progress.

## Session Extract - /story-done 2026-05-01
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/card-acquisition/story-001-state-scaffold.md` - State Scaffold: ShopStates, PlayerHands, Phase Machine
- Criteria: 3/3 passing; CA1, CA2, and CA7 verified against `process_purchase_card`, `process_refresh_shop_request`, `ShopStates`, and `PlayerHands`.
- Test Evidence: `tests/unit/card_acquisition/state_scaffold_test.rs`; `cargo test -p server --test card_acquisition_state_scaffold_test` -> 6 passed, 0 failed.
- Notes: Advisory only - story manifest v2026-04-30 is older than current control manifest v2026-05-01. Current implementation schedules CA after concrete systems `advance_phase` and `auction_tick_system`; named `RsmSet::Tick` and `AuctionSet::Tick` sets are not present in the current codebase, but behavior matches the intended order.
- Tech debt logged: None
- Sprint status: `CA-001` set to `done` in `production/sprint-status.yaml`; existing in-progress claims preserved.
- Next recommended: S3-05 RSM Story 4: Win Condition and Game Over at `production/epics/round-state-machine/story-004-win-condition-and-game-over.md`; Card Acquisition Story 002 is also unlocked at `production/epics/card-acquisition/story-002-draft-initial.md` if pulling CA work forward.

## Session Extract - /story-done 2026-05-01
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/auction-system/story-003-auction-abort-handler.md` - Auction Abort Handler
- Criteria: 2/3 passing; AU9 and AU19-b covered. AU19-a deferred until Story 006 settlement implementation; current handler no-ops in RESOLVING so abort does not interrupt settlement.
- Test Evidence: `tests/unit/auction/auction_abort_handler_test.rs`; `cargo test -p server --test auction_abort_handler_test` -> 3 passed, 0 failed, 1 ignored.
- Notes: Advisory only - story manifest v2026-04-30 is older than current control manifest v2026-05-01; RSM GDD has conflicting `auction_max_duration_seconds` language versus Auction GDD/TR-AUC-008.
- Tech debt logged: None
- Sprint status: `AUC-003` set to `done` in `production/sprint-status.yaml`; existing in-progress claims preserved.
- Next recommended: AUC-004 Bid Validation Gate at `production/epics/auction-system/story-004-bid-validation-gate.md`; enable the AU19-a settlement guard after AUC-006 lands.

## Session Extract - /story-done 2026-05-01
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/class-system/story-002-token-spawn-scaffold.md` - Token Spawn Scaffold / SourceClass Component
- Criteria: 6/6 passing; all token `SourceClass` mappings, `TokenUnit`, standard-unit `None`, snapshot derivation, and Miranda-style owner transfer preservation verified.
- Test Evidence: `tests/unit/class/token_spawn_test.rs`; executable suite `server/tests/token_spawn_test.rs`; `cargo test -p server --test token_spawn_test` -> 5 passed, 0 failed.
- Notes: Advisory only - story manifest v2026-04-30 is older than current control manifest v2026-05-01. Current `TR-CS-009` also includes token passive behaviors; this story closes only the spawn/snapshot scaffold portion and leaves passives to Story 010.
- Tech debt logged: None
- Sprint status: `CS-002` set to `done` in `production/sprint-status.yaml`; existing in-progress claims preserved.
- Next recommended: S3-05 RSM Story 4: Win Condition and Game Over at `production/epics/round-state-machine/story-004-win-condition-and-game-over.md`; S3-04, S3-06, S3-08, KW-002, and CARD-ANIM-001 remain in progress.

## Session Extract - /story-done 2026-05-01
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/keyword-system/story-002-movement-formulas.md` - Movement Formulas - repel_destination + attract_destination
- Criteria: 8/8 passing; REPEL clamp behavior, signed intermediate arithmetic, ATTRACT friendly co-location, ATTRACT enemy one-cell-short rule, no overshoot, board bounds, and determinism verified.
- Test Evidence: `tests/unit/keyword/movement_formulas_test.rs`; executable suite `server/tests/movement_formulas_test.rs`; `cargo test -p server --test movement_formulas_test` -> 11 passed, 0 failed.
- Notes: Advisory only - story manifest v2026-04-30 is older than current control manifest v2026-05-01. Story traceability was patched from stale `TR-KW-002` to `TR-BLS-010`; the old TR maps to CHARGE/HASTE, while this story matches KW-029/KW-030 REPEL/ATTRACT formulas and ADR-018.
- Tech debt logged: None
- Sprint status: `KW-002` set to `done` in `production/sprint-status.yaml`; existing in-progress claims for `S3-04`, `S3-06`, `S3-08`, and `CARD-ANIM-001` preserved.
- Next recommended: S3-05 RSM Story 4: Win Condition and Game Over at `production/epics/round-state-machine/story-004-win-condition-and-game-over.md`; S3-04 and S3-06 remain in progress.

## Session Extract - /story-done 2026-05-01
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/card-animations/story-001-plugin-scaffold-custom-lenses.md` - CardAnimationsPlugin scaffold + 5 custom lenses + cargo-check gates
- Criteria: 2/2 passing; CA-1 covered by plugin/lens unit tests, CA-20 covered for scaffolded registered domain messages.
- Test Evidence: `tests/unit/card-animations/plugin_scaffold_test.rs`; `cargo test -p client --test card_animations_plugin_scaffold_test --target-dir target\codex-card-animations-test` -> 8 passed, 0 failed.
- Notes: Advisory only - current GDD also names `TimerColorZoneRequested` and `NoBidsTransitionRequested`, but those message stubs are not present in the Story 001 scaffold. Advisory only - story/GDD/TR text says `bevy_tweening 0.18`, while the workspace pins `bevy_tweening = "0.15"` as the Bevy 0.18-compatible release.
- Tech debt logged: None
- Sprint status: `CARD-ANIM-001` set to `done` in `production/sprint-status.yaml`; existing in-progress claims preserved.
- Next recommended: Continue in-progress Sprint 3 must-haves S3-04 and S3-06 before opening S3-05, because S3-05's story file depends on S3-04 being Done.

## Session Extract - /story-done 2026-05-01
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/lightyear-protocol-verification/story-004-e2e-websocket-roundtrip.md` - End-to-End WebSocket Round-Trip Test
- Criteria: 15/15 passing; E2E WebSocket heartbeat round-trip, ReliableChannel proof, release WASM build, bundle size, DIFFERS resolution documentation, and ADR-012 final status verified.
- Test Evidence: `tests/integration/network/e2e_websocket_test.rs`; `tests/evidence/story-lyv-004-wasm-size.md`; `tests/evidence/lightyear-026-verification.md`.
- Verification: `cargo test -p server --test e2e_websocket_test e2e_websocket_heartbeat_roundtrip_and_reliable_channel --verbose` -> 1 passed, 0 failed. `cargo build -p client --target wasm32-unknown-unknown --release` -> passed; local `client.wasm` measured 20,905,154 bytes.
- Notes: Advisory only - story manifest v2026-04-29 is older than current control manifest v2026-05-01; no blocking drift found.
- Tech debt logged: None
- Sprint status: `S3-06` set to `done` in `production/sprint-status.yaml`; existing in-progress claims preserved.
- Next recommended: Continue S3-04 RSM Story 3 (`production/epics/round-state-machine/story-003-timers-and-input-reader.md`) before opening S3-05, because S3-05 depends on S3-04 being Done.

## Session Extract - /story-done 2026-05-01
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/round-state-machine/story-003-timers-and-input-reader.md` - Timers and Input Reader
- Criteria: 10/10 passing; timer activation, ready/submission early exits, stale inbound guards, SessionReady entry, and timer defaults verified.
- Test Evidence: `tests/unit/rsm/rsm_timers_test.rs`; executable suite `server/tests/rsm_timers_test.rs`; `cargo test -p server rsm_timers` passed with 10/10 RSM timer tests plus the timer default test.
- Verification: `cargo test -p server --test rsm_timers_test` passed 10/10; `cargo test -p server rsm_timers` passed the 10 RSM timer tests plus the timer default test; `cargo test -p server --test rsm_timers_test --test rsm_transitions_test` passed 24/24; `cargo check -p server` passed. `ShopRefreshNeeded` search under `server` and `tests` returned no matches; RSM emits `ShopRefreshTriggered`.
- Notes: Advisory only - story manifest v2026-04-29 is older than current control manifest v2026-05-01, and story text has stale observer wording (`Trigger`/`observe`) while implementation uses verified Bevy 0.18 `On`/`add_observer`.
- Tech debt logged: None
- Sprint status: `S3-04` set to `done` in `production/sprint-status.yaml`; existing in-progress claims preserved.
- Next recommended: S3-05 RSM Story 4: Win Condition and Game Over at `production/epics/round-state-machine/story-004-win-condition-and-game-over.md`.

## Session Extract - /story-done 2026-05-01
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/round-state-machine/story-004-win-condition-and-game-over.md` - Win Condition and Game Over
- Criteria: 9/9 accepted for this RSM story; objective counters, ResolutionComplete win evaluation, single-loser/draw/no-loss paths, auction-round guard, GAME_OVER event ordering, grep gate, F2 ordering test, and unit evidence verified.
- Test Evidence: `tests/unit/rsm/rsm_win_condition_test.rs`; `tests/integration/rsm/rsm_f2_ordering_test.rs`; evidence pointer at `tests/evidence/rsm-story-004-tests.md`.
- Verification: `cargo test -p server --test rsm_win_condition_test --test rsm_f2_ordering_test` passed 8/8 tests; `cargo check -p server` passed; `server/src/core/rsm/` has no `use server::feature` imports.
- Notes: Advisory only - story manifest v2026-04-29 is older than current control manifest v2026-05-01. Story text references `TR-RSM-08`; current registry uses `TR-RSM-008`. The downstream Game Session draw subscriber item is out of scope here and remains owned by GSS Story 006.
- Tech debt logged: None
- Sprint status: `S3-05` set to `done` in `production/sprint-status.yaml`; existing in-progress claims preserved.
- Next recommended: all Sprint 3 must-have stories are complete. Continue in-progress S3-08 if pulling should-have work forward, otherwise run `/smoke-check sprint` then `/team-qa sprint`.

## Session Extract - /story-done 2026-05-01
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/card-acquisition/story-002-draft-initial.md` - Draft Initial - 9-Card Offering
- Criteria: 4/4 passing; CA3, CA4, CA5, and CA21 verified against current `main` implementation. Implementation commit `2c6c65b` is included in current HEAD.
- Test Evidence: `tests/integration/card_acquisition/draft_initial_test.rs`; `cargo test -p server --test card_acquisition_draft_initial_test` passed 5/5 tests.
- Notes: Advisory only - story manifest v2026-04-30 is older than current control manifest v2026-05-01; no blocking drift found. Current `TR-CA-002` maps only CA3/CA4, while the current GDD also contains the CA5 and CA21 criteria closed by this story.
- Tech debt logged: None
- Sprint status: No `CA-002` entry exists in `production/sprint-status.yaml`; file left unchanged.
- Next recommended: Card Acquisition Story 003 (`production/epics/card-acquisition/story-003-draw-pipeline.md`) is ready and unlocks Stories 004 and 005.

## Session Extract - /story-done 2026-05-01
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/keyword-system/story-003-first-strike-haste.md` - FIRST STRIKE + HASTE Keywords
- Criteria: 6/6 passing; FIRST STRIKE standard-target timing, simultaneous FIRST STRIKE snapshots, HASTE same-round movement/attack, STUN-over-HASTE, HASTE+FIRST STRIKE, and HASTE+CHARGE X covered.
- Test Evidence: `tests/unit/keyword/first_strike_haste_test.rs`; executable suite `server/tests/first_strike_haste_test.rs`; `cargo test -p server --test first_strike_haste_test` passed 7/7.
- Verification: implementation commit `874d86b545bade516e6ef46d7bd07c143d34f7e9` is included in `main`.
- Notes: Advisory only - story text says ADR-018 is Proposed/BLOCKED and uses manifest v2026-04-30; current control manifest is v2026-05-01 and ADR-018 is Accepted, so this was stale text rather than a blocker.
- Tech debt logged: None
- Sprint status: No `KW-003` entry exists in `production/sprint-status.yaml`; file left unchanged.
- Next recommended: Keyword Story 004 STUN State (`production/epics/keyword-system/story-004-stun-state.md`) after readiness refresh, because its story text also contains stale ADR-018 Proposed/BLOCKED wording.

## Session Extract - /story-done 2026-05-01
- Verdict: COMPLETE
- Story: `production/epics/hud/story-001-hud-plugin-scaffold.md` - HUD Plugin Scaffold and Pre-Pooled Entity Tree
- Criteria: 4/4 passing; HUD-01, HUD-24, HUD-11, and the no-`ui_picking` client build gate verified.
- Test Evidence: `tests/unit/hud/hud_plugin_scaffold_test.rs`; `cargo test -p client --test hud_plugin_scaffold_test --target-dir target\codex-hud-test` passed 4/4.
- Verification: `cargo check -p client --target-dir target\codex-hud-test`, `cargo fmt -- --check`, `git diff --check`, and `cargo build -p client` passed.
- Notes: Implementation commits `b04748b`, `cbce522`, and test harness fix `95b58ae` are included in `main` at `c8a8210`; no blocking deviations found.
- Tech debt logged: None
- Sprint status: No `HUD-001` entry exists in `production/sprint-status.yaml`; file left unchanged.
- Next recommended: HUD-002 Gold and Mana Display (`production/epics/hud/story-002-gold-mana-display.md`) or HUD-003 Phase Label/Round Counter (`production/epics/hud/story-003-phase-label-round-counter.md`) after readiness check.

## Session Extract - /story-done 2026-05-01
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/economy-system/story-003-interest-snapshot-resolution.md` - Interest Snapshot & Resolution End
- Criteria: 10/10 accepted against current architecture; snapshot, max-interest, mana discard, stale overwrite, zero-gold baseline, kill-threshold crossing, and same-frame RSM handoff are covered.
- Test Evidence: `tests/unit/economy/interest_snapshot_test.rs`; executable suite `server/tests/economy_interest_snapshot_test.rs`; `cargo test -p server --test economy_interest_snapshot_test` passed 7/7.
- Verification: `cargo check -p server` passed; `cargo fmt --check` passed; implementation commit `db61102` is included in current `main`.
- Notes: Advisory only - story manifest v2026-04-29 is older than current control manifest v2026-05-01. Current ADR-019/control-manifest requires `MessageReader<ResolutionComplete>` and `on_resolution_complete.before(rsm_input_reader)`, superseding the story's stale `ResolutionPhaseEntered` trigger wording.
- Tech debt logged: None
- Sprint status: `S3-08` set to `done` in `production/sprint-status.yaml`; existing claims preserved.
- Next recommended: Sprint 3 Must Have and pulled-forward S3-08 are complete; run `/smoke-check sprint` then `/team-qa sprint`, or pick S3-07 Card Pool Story 4 if continuing Should Have work.

## Session Extract - /story-done 2026-05-01
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/card-animations/story-002-tween-cancel-replace-lifecycle.md` - Tween cancel-replace lifecycle
- Criteria: 5/5 passing; CA-2, CA-4, CA-6, CA-7, and CA-19 verified against the merged CARD-ANIM-002 implementation.
- Test Evidence: `tests/unit/card-animations/tween_lifecycle_test.rs`; `cargo test -p client --test card_animations_tween_lifecycle_test --target-dir target\codex-card-anim-002-test` passed 5/5.
- Verification: paired scaffold+lifecycle tests passed 8/8 and 5/5; `cargo check -p client --target-dir target\codex-card-anim-002-test` passed. Implementation commit `1354d5a` and merge commit `e9103d9` are included in current `main`.
- Notes: Advisory only - story/GDD/ADR wording still says `Animator<T>` / `AnimatorState` and `bevy_tweening 0.18`; the compiled workspace API is `TweenAnim` with `PlaybackState`/`TweenState` from `bevy_tweening v0.15.0`. Advisory only - current `TR-CAN-007` registry text maps to CA-25/scaffold-style lifecycle wording, while this story closes CA-2, CA-4, CA-6, CA-7, and CA-19.
- Tech debt logged: None
- Sprint status: No `CARD-ANIM-002` entry exists in `production/sprint-status.yaml`; file left unchanged.
- Next recommended: Continue the serialized story-done queue with BOARD-001, or close CARD-ANIM-004 if staying in Card Animations.

## Session Extract - /story-done 2026-05-01
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/board-lane-system/story-001-board-grid-initialization.md` - Board Grid Initialization
- Criteria: 3/3 passing; BL-21, NEW-001a, and NEW-001b covered by `tests/unit/board-lane-system/board_grid_initialization_test.rs`.
- Test Evidence: `tests/unit/board-lane-system/board_grid_initialization_test.rs`; `cargo test -p server --test board_grid_initialization_test` passed 4/4.
- Notes: Advisory only - story manifest v2026-04-29 is older than current control manifest v2026-05-01. Current `TR-BLS-001` registry text maps primarily to grid and direction requirements, while BL-21 remains covered by the GDD acceptance criteria and story-scoped unit test.
- Tech debt logged: None
- Sprint status: No `BOARD-001` entry exists in `production/sprint-status.yaml`; file left unchanged.
- Next recommended: Board Story 002 Standard Unit Movement at `production/epics/board-lane-system/story-002-standard-unit-movement.md` after readiness check.

## Session Extract - /story-done 2026-05-01
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/card-acquisition/story-003-draw-pipeline.md` - Shop Draw Pipeline - Auto-Refresh, Dedup, and 50/50 Split
- Criteria: 4/4 passing; CA6, CA12, CA16, and CA19 verified against the merged CA-003 implementation.
- Test Evidence: `tests/unit/card_acquisition/draw_pipeline_test.rs`; `cargo test -p server --test card_acquisition_draw_pipeline_test` passed 5/5 tests.
- Verification: `cargo check -p server` passed. Implementation commit `c6200f0` and merge commit `98cb52a` are included in current `main`.
- Notes: Advisory only - story manifest v2026-04-30 is older than current control manifest v2026-05-01; no blocking GDD/ADR drift found.
- Tech debt logged: None
- Sprint status: No `CA-003` entry exists in `production/sprint-status.yaml`; file left unchanged per user instruction.
- Next recommended: Card Acquisition Story 004 (`production/epics/card-acquisition/story-004-refresh-cost.md`) and Story 005 (`production/epics/card-acquisition/story-005-purchase-flow.md`) are unblocked by CA-003.

## Session Extract - /story-done 2026-05-01
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/card-acquisition/story-006-external-bypass.md` - External Bypasses - PlayerHands Shared API
- Criteria: 1/1 passing; CA17 covered by direct `PlayerHands::push_card()` boundary test with unchanged gold and no `C2SPurchaseCard` receiver path.
- Test Evidence: `tests/integration/card_acquisition/external_bypass_test.rs`; `cargo test -p server --test card_acquisition_external_bypass_test` passed 1/1 test.
- Verification: implementation commit `6af1137` is included in current `main`.
- Notes: Advisory only - story manifest v2026-04-30 is older than current control manifest v2026-05-01. Current registry/control-manifest wording uses `hand_push()` in places while implementation uses `PlayerHands::push_card()`; CA17 behavior matches the GDD/ADR intent.
- Tech debt logged: None
- Sprint status: No `CA-006` entry exists in `production/sprint-status.yaml`; file left unchanged per user instruction.
- Next recommended: Continue the serialized story-done queue with CARD-ANIM-004, or run readiness on Card Acquisition Story 004/005 if continuing that epic.

## Session Extract - /story-done 2026-05-01
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/hand-ui/story-001-plugin-scaffold.md` - HandUiPlugin Scaffold - Pre-Pooled Entity Spawning
- Criteria: 1/1 passing; HU-01 covered by `tests/unit/hand-ui/plugin_scaffold_test.rs`.
- Test Evidence: `tests/unit/hand-ui/plugin_scaffold_test.rs`; `cargo test -p client --test hand_ui_plugin_scaffold_test` passed 3/3.
- Verification: `cargo check -p client` passed; client source grep found no `NodeBundle`, `SpriteBundle`, `set_parent`, `despawn_recursive`, or ungated `PickingBehavior`.
- Notes: Advisory only - `PresentationPlugin` / `BoardRenderingPlugin` composition is not present in `client/src/` yet, so ADR-021's final registration-order contract is not verifiable in this story. HU-01 scoped pre-pooling behavior is verified.
- Tech debt logged: None
- Sprint status: No Hand UI entry exists in `production/sprint-status.yaml`; file left unchanged.
- Next recommended: Hand UI Story 002 Fan Layout Formula (`production/epics/hand-ui/story-002-fan-layout-formula.md`) is already In Progress and has passing evidence available; continue its story-done queue or run `/story-readiness` on Story 003 Phase State Machine.

## Session Extract - /story-done 2026-05-01
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/card-animations/story-004-anim-queue-resolution-drain.md` - AnimQueue RESOLUTION drain + GAME_OVER skip path + empty-queue handling
- Criteria: 4/4 passing; GAME_OVER skip, objective reveal on skip, 599/600 ms boundary behavior, and empty queue pending-phase drain verified.
- Test Evidence: `tests/unit/card-animations/anim_queue_test.rs`; `cargo test -p client --test card_animations_anim_queue_test` passed 4/4.
- Verification: `client/src/card_animations/queue.rs` implements `resolution_executing_system`, GAME_OVER skip staging, `GroupDrainedSignal`, and empty-queue drain; `client/src/card_animations/mod.rs` registers `GroupDrainedSignal` and orders objective reveal after queue drain. No direct S2C readers or old Bevy `EventWriter`/`EventReader` patterns found in `client/src/card_animations`.
- Notes: Advisory only - story/GDD wording still says `Animator<T>` / `AnimatorState` and `bevy_tweening 0.18`, while the compiled workspace uses `TweenAnim` / `PlaybackState` / `TweenState` from `bevy_tweening 0.15`. Advisory only - `TR-CAN-006` registry text says 100 ms pause, while current GDD/code use 150 ms inter-step pause; this did not affect the four scoped ACs.
- Tech debt logged: None
- Sprint status: No `CARD-ANIM-004` entry exists in `production/sprint-status.yaml`; file left unchanged per user instruction.
- Next recommended: Card Animations Story 006 Multi-objective stagger reveal (`production/epics/card-animations/story-006-objective-stagger-reveal.md`) is unlocked by this story.

## Session Extract - /story-done 2026-05-01
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/card-animations/story-009-ci-boundary-enforcement.md` - CI boundary enforcement (no direct S2C subscription in card_animations/)
- Criteria: 1/1 passing; CA-14 covered by the `Check Card Animations S2C boundary` step in `.github/workflows/tests.yml`.
- Test Evidence: Integration CI config in `.github/workflows/tests.yml`; local grep found no `Message(Reader|Receiver)<S2C` in `client/src/card_animations/`. CI was not waited on per instruction.
- Verification: main integration commit `75e11ea` is included in `main`; worker commit `55b5331` is not an ancestor of `HEAD`, but its `.github/workflows/tests.yml` content is identical to `75e11ea`.
- Notes: Advisory only - story references `TR-CAN-001`, which currently maps to CA-1 rather than CA-14. Advisory only - GDD text still mentions legacy `EventReader<S2C` and `src/card_animations/`; implemented CI targets actual path `client/src/card_animations/` and checks both `MessageReader` and Lightyear `MessageReceiver` S2C subscriptions.
- Tech debt logged: None
- Sprint status: No `CARD-ANIM-009` entry exists in `production/sprint-status.yaml`; file left unchanged per user instruction.
- Next recommended: Card Animations Story 006 Multi-objective stagger reveal (`production/epics/card-animations/story-006-objective-stagger-reveal.md`) or Story 008 Input-gating (`production/epics/card-animations/story-008-input-gating.md`) after readiness check.

## Session Extract - /story-done 2026-05-01
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/board-lane-system/story-002-standard-unit-movement.md` - Standard Unit Movement Formula (F1)
- Criteria: 4/4 passing; BL-1, BL-2, BL-3, and BL-4 covered by `tests/unit/board-lane-system/standard_movement_test.rs`.
- Test Evidence: `tests/unit/board-lane-system/standard_movement_test.rs`; `cargo test -p server --test standard_movement_test` passed 5/5.
- Verification: `cargo check -p server` passed. Worker commit `4a76028` has the same BOARD-002 implementation diff as main integration commit `0d8e41c`; `0d8e41c` is included in current `main`.
- Notes: Advisory only - story manifest v2026-04-29 is older than current control manifest v2026-05-01. Advisory only - implementation uses current project ECS components (`BoardPosition`, `UnitStats`, `UnitOwner`) instead of the story's older placeholder component names.
- Tech debt logged: None
- Sprint status: No `BOARD-002` row exists in `production/sprint-status.yaml`; file left unchanged per user instruction.
- Next recommended: Continue in-progress Board Story 003 Spawn Range Validation, or use the now-unlocked Board Story 006 CHARGE X bonus movement after readiness check.

## Session Extract - /story-done 2026-05-01
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/keyword-system/story-004-stun-state.md` - STUN State Management
- Criteria: 3/3 passing; KW-015a, KW-015b, and KW-034 covered by `tests/unit/keyword/stun_test.rs`.
- Test Evidence: `tests/unit/keyword/stun_test.rs`; executable suite `server/tests/stun_test.rs`; `cargo test -p server --test stun_test` passed 4/4.
- Verification: `cargo check -p server` passed; `cargo check -p shared` passed. Worker commit `7543293` exists on `work/kw-004-stun-state`; main integration commit `b8b1287` is included in current `main`.
- Notes: Advisory only - story manifest v2026-04-30 is older than current control manifest v2026-05-01. Story text still said ADR-018 was Proposed/BLOCKED, while current architecture has ADR-018 Accepted. Current `TR-KW-007` registry wording says "one-sub-step scope", while the GDD acceptance criteria, ADR-018, and tests close the one-RESOLUTION STUN lifecycle.
- Tech debt logged: None
- Sprint status: No `KW-004` row exists in `production/sprint-status.yaml`; file left unchanged per user instruction.
- Next recommended: Keyword Story 005 SHIELD Scope (`production/epics/keyword-system/story-005-shield-scope.md`) after readiness check, or continue the serialized story-done queue already in progress.

## Session Extract - /story-done 2026-05-01
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/card-animations/story-008-input-gating.md` - Input-gating: timer bar, drag latency, bid button state, de-hover cancel-replace
- Criteria: 3/3 blocking passing (CA-13, CA-23, CA-24); advisory manual evidence for CA-13b and CA-22 deferred until bid-button UI and DRAFT_INITIAL animation sequencing UI exist.
- Test Evidence: `tests/integration/card-animations/input_gating_test.rs`; `cargo test -p client --test card_animations_input_gating_test` passed 6/6. Scaffold regression `cargo test -p client --test card_animations_plugin_scaffold_test` passed 8/8. `cargo check -p client` passed.
- Verification: worker branch `work/card-anim-008-input-gating` contains `0d75fb0`; main integration commit `9308bf3` is included in current `main`. `0d75fb0` is not an ancestor of `HEAD`, but it has the same stable patch-id as `9308bf3`.
- Notes: Manual CA-13b and CA-22 evidence remains pending in `production/qa/evidence/input-gating-evidence.md` because the required UI surfaces do not exist yet. Advisory only - story/GDD wording still says `Animator<T>` / `AnimatorState` and `bevy_tweening 0.18`, while the compiled workspace uses `TweenAnim` / `PlaybackState` / `TweenState` from `bevy_tweening 0.15`.
- Tech debt logged: None
- Sprint status: No `CARD-ANIM-008` row exists in `production/sprint-status.yaml`; file left unchanged per user instruction.
- Next recommended: Card Animations Story 006 Multi-objective stagger reveal (`production/epics/card-animations/story-006-objective-stagger-reveal.md`) after readiness check, or continue the serialized story-done queue.

## Session Extract - /story-done 2026-05-01
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/hud/story-002-gold-mana-display.md` - Gold and Mana Display (ECONOMY_BASIC)
- Criteria: 6/6 passing; HUD-03, HUD-04, HUD-21, HUD-25, HUD-31, and Multi-update collapse covered by `tests/unit/hud/gold_mana_display_test.rs`.
- Test Evidence: `tests/unit/hud/gold_mana_display_test.rs`; `cargo test -p client --test hud_gold_mana_display_test` passed 6/6. Scaffold regression `cargo test -p client --test hud_plugin_scaffold_test` passed 4/4. `cargo check -p client` passed.
- Verification: worker branch `work/hud-002-gold-mana-display` contains worker commit `0c00a44`; main integration commit `3eaf578` is included in current `main`. `0c00a44` is not an ancestor of `HEAD`, but the integrated HUD-002 implementation is present through `3eaf578`.
- Notes: Advisory only - `TR-HUD-001` / `TR-HUD-002` registry entries map to `HUD-01` / `HUD-02`, while this story verifies story-scoped GDD criteria `HUD-03`, `HUD-04`, `HUD-21`, `HUD-25`, and `HUD-31`; current GDD behavior is covered by tests.
- Tech debt logged: None
- Sprint status: No `HUD-002` row exists in `production/sprint-status.yaml`; file left unchanged per user instruction.
- Next recommended: `S3-07` Card Pool Story 4 remains the active Sprint 3 should-have backlog item, or continue the serialized story-done queue.

## Session Extract - /story-done 2026-05-01
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/card-animations/story-006-objective-stagger-reveal.md` - Multi-objective stagger reveal (F1 formula)
- Criteria: 3/3 passing; CA-10, CA-11, and CA-16 covered by `tests/unit/card-animations/objective_stagger_test.rs`.
- Test Evidence: `tests/unit/card-animations/objective_stagger_test.rs`; `cargo test -p client --test card_animations_objective_stagger_test` passed 3/3. Queue regression `cargo test -p client --test card_animations_anim_queue_test` passed 4/4. Scaffold regression `cargo test -p client --test card_animations_plugin_scaffold_test` passed 8/8. `cargo check -p client` passed.
- Verification: worker branch `work/card-anim-006-objective-stagger-reveal` contains worker commit `effcef2`; main integration commit `8d641b9` is included in current `main` and has no diff against `effcef2`.
- Notes: Advisory only - `TR-CAN-001` and `TR-CAN-004` registry entries map to older CA-1/CA-4 wording, while this story verifies story-scoped GDD criteria CA-10, CA-11, and CA-16. Advisory only - story/GDD wording still says `Animator<T>` / `AnimatorState` and `bevy_tweening 0.18`, while the compiled workspace uses `TweenAnim` / `PlaybackState` / `TweenState`.
- Tech debt logged: None
- Sprint status: No `CARD-ANIM-006` row exists in `production/sprint-status.yaml`; file left unchanged per user instruction.
- Next recommended: Continue the user-directed serialized story-done queue, or run readiness on Sprint 4 `S4-01` before starting new implementation work.

## Session Extract - /story-done 2026-05-01
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/hud/story-003-phase-label-round-counter.md` - Phase Label, Round Counter, and Instantaneous Transitions
- Criteria: 4/4 passing; HUD-05, HUD-22, HUD-12b label/round portion, and HUD-11 covered by `tests/unit/hud/phase_label_round_counter_test.rs`.
- Test Evidence: `tests/unit/hud/phase_label_round_counter_test.rs`; `cargo test -p client --test hud_phase_label_round_counter_test` passed 6/6. HUD regression `cargo test -p client --test hud_plugin_scaffold_test --test hud_gold_mana_display_test --test hud_phase_label_round_counter_test` passed 16/16. `cargo check -p client` passed.
- Verification: worker branch `work/hud-003-phase-label-round-counter` contains worker commit `52a3605`; main integration commit `ce76a88` is included in current `main`. `52a3605` is not an ancestor of `HEAD`, but the HUD-003 source/test implementation is integrated through `ce76a88`.
- Notes: Advisory only - `TR-HUD-003` registry metadata points at older HUD AC numbering, while this story closes current story-scoped GDD criteria HUD-05, HUD-22, HUD-12b, and HUD-11. Advisory only - shared `PresentationPlugin`/`phase_sink_system` wiring is not present yet; HUD-003 verifies the HUD sub-plugin reads `Res<CurrentClientPhase>` and does not drain `MessageReceiver<S2CPhaseChanged>` directly.
- Tech debt logged: None
- Sprint status: No `HUD-003` row exists in `production/sprint-status.yaml`; file left unchanged per user instruction.
- Next recommended: Continue the user-directed serialized story-done queue, or run readiness on Sprint 4 `S4-01` before starting new implementation work.

## Session Extract - /story-done 2026-05-01
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/keyword-system/story-005-shield-scope.md` - SHIELD Sub-Step Scope
- Criteria: 7/7 passing; KW-024, KW-037, same-sub-step simultaneous absorption, persistence until triggered, SS3-to-SS6 consumption, SS6-to-next-round consumption, and `ShieldConsumed` event emission covered by `tests/unit/keyword/shield_test.rs`.
- Test Evidence: `tests/unit/keyword/shield_test.rs`; `cargo test -p server --test shield_test` passed 7/7. `cargo test -p server --test keyword_plugin_smoke_test` passed 2/2. `cargo check -p server` passed.
- Verification: worker branch `work/kw-005-shield-scope` contains worker commit `a1a824b`; main integration commit `0b610fd` is included in current `main` and has the same stable patch-id as `a1a824b`.
- Notes: Advisory only - story manifest v2026-04-30 is older than current control manifest v2026-05-01. Advisory only - story text still says ADR-018 was Proposed/BLOCKED, while current ADR-018 is Accepted. Advisory only - ADR-018 contains an older immutable `check_shield_absorb` signature snippet; implementation uses the mutable signature required to consume SHIELD and matches the story/GDD behavior.
- Tech debt logged: None
- Sprint status: No `KW-005` row exists in `production/sprint-status.yaml`; file left unchanged per user instruction.
- Next recommended: Keyword Story 013 FINAL BLOW + COUNTERATTACK Dispatch (`production/epics/keyword-system/story-013-final-blow-counterattack.md`) after readiness check, or continue the serialized story-done queue.

## Session Extract - /story-done 2026-05-01
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/board-lane-system/story-003-spawn-range-validation.md` - Spawn Range Validation (F2)
- Criteria: 5/5 passing; BL-5, BL-5b, BL-6, BL-6b, and BL-7 covered by `tests/unit/board-lane-system/spawn_range_validation_test.rs`.
- Test Evidence: `tests/unit/board-lane-system/spawn_range_validation_test.rs`; `cargo test -p server --test spawn_range_validation_test` passed 7/7. `cargo check -p server` passed.
- Verification: worker branch `work/BOARD-003-spawn-range-validation` contains worker commit `bf39342`; main integration commit `9c38083` is included in current `main`. `bf39342` is not an ancestor of `main`, but the BOARD-003 source/test/story diff matches `9c38083`.
- Notes: No blocking GDD or ADR deviation found. Implementation uses current `SessionConfig.team_map` plus structural `BoardConfig` spawn cells instead of the story's simplified `PlayerId::A/B` sample; defaults remain Player A cell 1 and Player B cell 8, and tests cover the required F2 behavior.
- Tech debt logged: None
- Sprint status: No `BOARD-003` row exists in `production/sprint-status.yaml`; file left unchanged per user instruction.
- Next recommended: Board Story 004 placement occupancy validation (`production/epics/board-lane-system/story-004-placement-occupancy.md`) after readiness check, or continue the user-directed serialized story-done queue.

## Session Extract - /story-done 2026-05-01
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/hand-ui/story-002-fan-layout-formula.md` - Story 002: Fan Layout Formula - Card Position & Rotation
- Criteria: 4/4 passing; HU-02, HU-02b, HU-03, and HU-03b covered by `tests/unit/hand-ui/fan_layout_formula_test.rs`.
- Evidence: `cargo test -p client --test hand_ui_fan_layout_formula_test` passed 5/5; `cargo test -p client --test hand_ui_plugin_scaffold_test` passed 3/3; `cargo check -p client` passed.
- Deviations: Advisory docs/config notes only; no blockers.
- Tech debt logged: None
- Sprint status: Unchanged; no HAND-UI-002 row exists in `production/sprint-status.yaml`.
- Next recommended: Hand UI Story 003 Phase State Machine (`production/epics/hand-ui/story-003-phase-state-machine.md`).

## Session Extract - /story-done 2026-05-01
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/card-acquisition/story-004-refresh-cost.md` - Manual Refresh Cost Formula and Counter Reset
- Criteria: 6/6 passing; CA8, CA9, CA10, CA11, CA15, and CA22 covered by `tests/unit/card_acquisition/refresh_cost_test.rs`.
- Test Evidence: `cargo test -p server --test card_acquisition_refresh_cost_test` passed 6/6; CA regression bundle passed 17/17; `cargo test -p server --test game_config_defaults_test` passed 7/7; `cargo check -p server` passed.
- Verification: worker branch `work/ca-004-refresh-cost` contains worker commit `f26f738`; main integration commit `5cb53a8` is included in current `main`.
- Notes: No blocking GDD or ADR deviation found. Scope note only: implementation also updated GameConfig defaults, `assets/config/game_config.ron`, `server/Cargo.toml`, and test wiring required for the refresh-cost evidence.
- Tech debt logged: None
- Sprint status: No `CA-004` row exists in `production/sprint-status.yaml`; file left unchanged per user instruction.
- Next recommended: Card Acquisition Story 005 Purchase Flow (`production/epics/card-acquisition/story-005-purchase-flow.md`) after readiness check, or continue the user-directed serialized story-done queue.

## Session Extract - /story-done 2026-05-01
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/board-lane-system/story-004-placement-occupancy.md` - Placement Occupancy Enforcement
- Criteria: 5/5 passing; BL-8, BL-9, BL-29, BL-32, and BL-33 covered by `tests/unit/board-lane-system/placement_occupancy_test.rs`.
- Test Evidence: `cargo test -p server --test placement_occupancy_test` passed 10/10; board regression tests passed (`board_grid_initialization_test` 4/4, `spawn_range_validation_test` 7/7, `standard_movement_test` 5/5); session regressions passed (`room_create_join_test` 7/7, `class_reveal_test` 8/8); `cargo check -p server` passed.
- Verification: worker branch `work/BOARD-004-placement-occupancy` contains worker commit `224708d`; main integration commit `0c69612` is included in current `main`. The worker and integration commits have the same stable patch-id.
- Notes: No blocking GDD or ADR deviation found. Scope note only: implementation also updated board occupancy storage/exports, `SessionConfig`/`GameMode` support, `server/Cargo.toml`, and the board initialization regression test to support the occupancy API and 2v2 team-capacity evidence.
- Tech debt logged: None
- Sprint status: No `BOARD-004` row exists in `production/sprint-status.yaml`; file left unchanged per user instruction.
- Next recommended: Board Story 005 placement buffer phase integration (`production/epics/board-lane-system/story-005-placement-buffer-phase-integration.md`) after readiness check, or continue the user-directed serialized story-done queue.

## Session Extract - /story-done 2026-05-01
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/card-acquisition/story-005-purchase-flow.md` - Story 005: Purchase Flow, Dead Slot, and CA18 Atomicity
- Criteria: 4/4 passing; CA13, CA14, CA18, and CA20 covered by `tests/integration/card_acquisition/purchase_atomicity_test.rs`.
- Test Evidence: `cargo test -p server --test card_acquisition_purchase_atomicity_test` passed 4/4; CA regression bundle passed 22/22; `cargo check -p server` passed.
- Verification: worker branch `work/ca-005-purchase-flow` contains worker commit `415384a`; main integration commit `c6141bc` is included in current `main`; the worker and integration commits have identical trees for the CA-005 changed files.
- Notes: No blocking GDD or ADR deviation found. Advisory only: `TR-CA-009` maps to CA20 but its registry requirement text describes dead-slot/draw-None fallback rather than the current GDD CA20 phase-transition criterion; story/GDD CA20 is covered, and CA13 covers dead-slot purchase rejection.
- Tech debt logged: None
- Sprint status: No `CA-005` row exists in `production/sprint-status.yaml`; file left unchanged per user instruction.
- Next recommended: Card Acquisition epic close-out evidence is ready; continue with sprint/epic smoke or the user-directed serialized story-done queue.

## Session Extract - /story-done 2026-05-01
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/combat-resolution/story-001-resolve-combat-scaffold.md` - resolve_combat Scaffold + Safety Timeout
- Criteria: 4/4 passing; CR-30 placement reveal ordering, CR-32 resolution-event ordering, CR-41 iteration budget overflow, and idle exit behavior covered by `tests/integration/combat/resolve_combat_scaffold_test.rs`.
- Test Evidence: `cargo test -p server --test resolve_combat_scaffold_test` passed 3/3. `cargo check -p server` passed.
- Verification: `server/src/feature/combat/mod.rs` implements the exclusive `resolve_combat` scaffold, pending completion bridge, iteration budget guard, and combat network outbox trace. RSM wiring reads `ResolutionComplete` before advancing from RESOLUTION.
- Notes: Advisory only - scaffold uses `CombatNetworkOutbox` rather than direct Lightyear `MessageSender`; `S2CPlacementReveal` and `S2CResolutionEvent` are registered on `ReliableChannel`, and actual network dispatch remains for later wiring stories.
- Tech debt logged: None
- Sprint status: Unchanged; no `COMBAT-001` row exists in `production/sprint-status.yaml`.
- Next recommended: Combat Resolution Story 002 modifier stack (`production/epics/combat-resolution/story-002-combat-modifier-stack.md`) or Story 003 sub-step 1 placement/appearance (`production/epics/combat-resolution/story-003-substep1-placement-appearance.md`) after readiness check.

## Session Extract - /story-done 2026-05-01
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/objective-system/story-001-objective-state-model.md` - Objective State Model
- Criteria: 3/3 passing; OS-1a, OS-1b, and OS-1c covered by `tests/unit/objective/objective_state_test.rs`.
- Test Evidence: `cargo test -p server --test objective_state_test` passed 4/4. `cargo check -p server` passed.
- Verification: `server/src/feature/objective/` defines and registers `ObjectiveHp`, `ObjectiveSlot`, and `HiddenObjectives`; objective spawn creates five replicated HP slots per player at configured HP; `ObjectiveCounters` is exposed through `server/src/core/objective_contract.rs` for RSM-safe reads.
- Notes: Advisory only - `ObjectiveCounters` lives in core and is re-exported by the objective feature to satisfy the RSM no-Feature-import layering rule. Advisory only - Objective System GDD Rule 4 still contains older replicated-identity wording; current TR-OBJ-007 and ADR-001 are followed.
- Tech debt logged: None
- Sprint status: Unchanged; no Objective Story 1 row exists in `production/sprint-status.yaml`.
- Next recommended: Objective Story 002 Fake Assignment & Config Guards (`production/epics/objective-system/story-002-fake-assignment-and-config-guards.md`) after readiness check.

## Session Extract - /story-done 2026-05-01
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/hud/story-005-phase-transitions.md` - Phase Transitions and RESOLUTION Persistence
- Criteria: 4/4 passing; HUD-15, HUD-16, HUD-18, and HUD-09 covered by `tests/unit/hud/phase_transitions_test.rs`.
- Test Evidence: `cargo test -p client --test hud_phase_transitions_test` passed 5/5. HUD regression slice passed 21/21. `cargo check -p client` passed.
- Verification: `client/src/ui/hud/mod.rs` keeps phase transitions in `HudSystemSet::PhaseTransition`, reads `Res<CurrentClientPhase>`, uses `HudMode` as the mode resource, mutates `Visibility` directly, and leaves `InheritedVisibility` untouched.
- Notes: Advisory only - full sister-UI hiding is explicitly out of scope for isolated `HudPlugin` verification, and lean mode skipped the external QA/code-review gates.
- Tech debt logged: None
- Sprint status: Unchanged; no `HUD-005` row exists in `production/sprint-status.yaml`.
- Next recommended: HUD Story 006 ECONOMY_AUCTION Inline Gold Format (`production/epics/hud/story-006-economy-auction-inline-gold.md`) after readiness check.

## Session Extract - /story-done 2026-05-01
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/hand-ui/story-003-phase-state-machine.md` - Story 003: Phase State Machine - Visibility & Input Gating
- Criteria: 3/3 passing; HU-04, HU-05, and HU-06 covered by `tests/unit/hand-ui/phase_state_machine_test.rs`.
- Test Evidence: `cargo test -p client --test hand_ui_phase_state_machine_test` passed 3/3. `cargo check -p client` passed.
- Verification: `client/src/ui/hand/mod.rs` implements `HandUiMode`, phase-driven visibility changes, tween cancellation via the current `TweenAnim` abstraction, and PASSIVE_LOCKED click suppression. Hand UI reads `Res<CurrentClientPhase>` and does not drain `MessageReceiver<S2CPhaseChanged>` directly.
- Notes: Advisory only - `TR-HU-001` maps to HU-01 pre-pooling while this story closes current GDD criteria HU-04/HU-05/HU-06. Advisory only - global ADR-021 `PresentationSet` / `phase_sink_system` wiring is outside this story; local Hand UI scheduling is used. HU-06 is verified through `HandUiOutboundMessages`, not a real Lightyear outbound sender.
- Tech debt logged: None
- Sprint status: Unchanged; no `HAND-UI-003` row exists in `production/sprint-status.yaml`.
- Next recommended: Hand UI Story 004 DRAFT_INITIAL Grid (`production/epics/hand-ui/story-004-draft-initial-grid.md`) or Story 005 PLACEMENT Entry (`production/epics/hand-ui/story-005-placement-submit-core.md`) after readiness check.

## Session Extract - /story-done 2026-05-02
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/board-lane-system/story-005-placement-buffer-phase-integration.md` - Placement Buffer and Phase Integration
- Criteria: 4/4 passing; BL-14, ADR-007-LC-1, ADR-007-LC-2, and ADR-007-LC-3 covered by `tests/integration/board-lane-system/placement_buffer_test.rs`.
- Test Evidence: `cargo test -p server --test placement_buffer_test` passed 3/3; `cargo check -p server` passed; `cargo test -p server --test placement_occupancy_test --test spawn_range_validation_test` passed 17/17; `cargo fmt -p server -- --check` passed.
- Verification: `close_placement_phase` sends `S2CPlacementReveal` through `ServerMultiMessageSender` on `ReliableChannel` before spawning entities with `Replicate::to_clients(NetworkTarget::All)`; the live WebSocket test receives the reveal on a client before asserting replicated spawn trace order.
- Notes: Advisory only - `production/qa/evidence/placement-buffer-evidence.md` records automated evidence only; no manual lead sign-off is claimed. Lean mode skipped external QA/code-review gates.
- Tech debt logged: None
- Sprint status: Unchanged; no `BOARD-005` row exists in `production/sprint-status.yaml`.
- Next recommended: Continue the user-directed serialized story-done queue with ECO-005 or BOARD-006, or run readiness on Board Story 007 after Board Story 006 closes.

## Session Extract - /story-done 2026-05-02
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/economy-system/story-005-auction-reservation-bid-validation.md` - Auction Reservation & Bid Validation
- Criteria: 13/13 passing; auction bid validation, reservation lifecycle, shop affordability under active reservation, outbid release, auction win spend, release overflow guard, and server check covered.
- Test Evidence: `cargo test -p server --test auction_reservation_test` passed. `cargo check -p server` passed.
- Verification: Closure-only update from the already reviewed result; no implementation or test files changed in this closure commit.
- Notes: Prior review found Verdict: COMPLETE WITH NOTES with 13/13 acceptance criteria passing.
- Tech debt logged: None
- Sprint status: Unchanged; no S4/ECO-005 row exists in `production/sprint-status.yaml`.
- Next recommended: Continue the user-directed serialized story-done queue or run readiness on the next Sprint 4 economy/auction story.

## Session Extract - /story-done 2026-05-02
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/board-lane-system/story-006-charge-bonus-movement.md` - Story 006: CHARGE X Bonus Movement and Intermediate Cell Skip
- Criteria: 3/3 passing; BL-22 and BL-27b covered by `tests/unit/board-lane-system/charge_movement_test.rs`, BL-27 covered by `tests/unit/board-lane-system/standard_movement_test.rs`.
- Test Evidence: `cargo test -p server --test charge_movement_test --test standard_movement_test` passed 10/10. `cargo check -p server` passed.
- Verification: `server/src/feature/board/movement.rs` defines `ChargeBonus`, reuses `apply_f1` for CHARGE and standard movement, reads owner direction from `SessionConfig`, and updates only the final `BoardPosition.cell` for each movement sub-step.
- Notes: No blocking GDD or ADR deviation found. Advisory only - trap "no trigger" behavior is verified by unchanged trap occupancy; explicit `TrapTrigger` event absence is not asserted because positive trap trigger mechanics are Story 007 scope. Lean mode skipped external QA/code-review gates.
- Tech debt logged: None
- Sprint status: Unchanged; no Board Story 006 row exists in `production/sprint-status.yaml`.
- Next recommended: Board Story 007 Trap Trigger Mechanics (`production/epics/board-lane-system/story-007-trap-trigger-mechanics.md`) after readiness check, or Board Story 009 Prism Collection if trap integration is deferred.

## Session Extract - /story-done 2026-05-02
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/round-state-machine/story-005-disconnect-handling.md` - Story 005: Disconnect Handling
- Criteria: 10/10 passing against current TR-RSM-010 and GDD Rule 13; the implementation follows the current heartbeat-driven millisecond countdown model rather than the stale story-era elapsed-seconds text.
- Test Evidence: `cargo test -p server rsm_disconnect` passed 11/11; evidence recorded in `tests/evidence/rsm-story-005-tests.md`. `cargo fmt -p server -- --check`, `cargo check -p server`, the WebSocket heartbeat regression, and `cargo test -p server rsm_` all passed.
- Verification: `RoundState.disconnect_trackers` is `HashMap<PlayerId, u32>`; `C2SHeartbeat` is resolved through `PlayerConnectionMap` and forwarded to RSM as `PlayerHeartbeat`; Lightyear 0.26 `Connected`/`Disconnected` marker observers are preserved; RSM-006 network dispatch remains untouched.
- Notes: Advisory only - story text still references the older elapsed `f32`/remove-on-reconnect model, while the current GDD/TR requires countdown reset behavior. Lean mode skipped external QA/code-review gates.
- Tech debt logged: None
- Sprint status: Unchanged; no RSM Story 005 row exists in `production/sprint-status.yaml`.
- Next recommended: RSM Story 006 Network Dispatch Wiring (`production/epics/round-state-machine/story-006-network-dispatch-wiring.md`) after readiness check.

## Session Extract - /story-done 2026-05-02
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/card-animations/story-003-simultaneous-track-animation.md` - Story 003: Simultaneous Transform controller animation
- Criteria: 2/2 passing; CA-15 and CA-18 covered by `tests/unit/card-animations/tracks_animation_test.rs`.
- Test Evidence: `cargo test -p client --test card_animations_tracks_animation_test` passed 4/4. `cargo fmt -p client -- --check` passed. `cargo check -p client` passed.
- Verification: `client/src/card_animations/animators.rs` spawns separate `TweenAnim` controller entities with `AnimTarget::component::<Transform>` and `AnimTarget::component::<Sprite>`; `client/src/card_animations/lenses.rs` provides X-only and Y-only translation lenses for same-`Transform` parallelism.
- Notes: No blocking GDD or ADR deviation found. Advisory only - `TR-CAN-004` registry text is broader than this story's scoped CA-15/CA-18 checks. Lean mode skipped external QA/code-review gates.
- Tech debt logged: None
- Sprint status: Unchanged; no `CARD-ANIM-003` row exists in `production/sprint-status.yaml`.
- Next recommended: Card Animations Story 005 placement reveal parallelism (`production/epics/card-animations/story-005-placement-reveal-parallelism.md`) or Story 007 damage number lifecycle (`production/epics/card-animations/story-007-damage-number-lifecycle.md`) if those implemented branches need formal closure.

## Session Extract - /story-done 2026-05-02
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/game-session-system/story-004-f4-session-ready.md` - Story 004: F4 Predicate and SessionReady Trigger
- Criteria: 12/12 passing; F4 exact-deadline behavior is covered by `tests/unit/session/session_ready_test.rs::test_session_ready_f4_true_on_exact_lobby_deadline`, and the SessionReady trigger path is covered by the required GSS-004 unit/integration bundle.
- Test Evidence: `cargo fmt -p server -- --check` passed; `cargo test -p server --test session_ready_test --test single_fire_test --test rng_init_failure_test --test lobby_to_draft_initial_test` passed; `cargo check -p server` passed.
- Verification: Repair commit `3c64b84` changes F4 to `server_clock_now <= lobby_deadline`, preserves Bevy 0.18 `add_observer(on_session_ready)`, and adds no second `SessionReady` observer.
- Notes: Advisory only - story acceptance text still contains older `<` wording, while current TR-GSS-003/GDD require `<=`. Advisory only - story/ADR wording still references `app.observe(on_session_ready)`, while current implementation uses Bevy 0.18 `add_observer(on_session_ready)` as required.
- Tech debt logged: None
- Sprint status: `S3-09` in `production/sprint-status.yaml` marked `done` with completed date `2026-05-02`.
- Next recommended: Continue the user-directed serialized story-done queue or run readiness on the next queued Sprint 4 story.

## Session Extract - /story-done 2026-05-02
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/prism-system/story-001-state-scaffold.md` - Story 001: PrismState Scaffold and Session Lifecycle
- Criteria: 3/3 passing; PS-08, PS-07b, and PS-15 covered by `tests/unit/prism/state_scaffold_test.rs`.
- Test Evidence: `cargo test -p server --test prism_state_scaffold_test` passed 6/6. Supplemental `cargo test -p server --test prism_deterministic_lanes_test` passed 6/6.
- Verification: `server/src/feature/prism/` defines `PrismState`, `DiscardLog`, `AuditLog`, `PrismPresence`, `PrismCollected`, lifecycle init/cleanup, Bevy `MessageReader`/`add_message` usage, and public `Replicate::to_clients(NetworkTarget::All)` presence replication.
- Notes: No blocking GDD or ADR deviation found. Advisory only - story manifest version is 2026-04-30 while current control manifest is 2026-05-01. Advisory only - deterministic lane reward implementation and tests are already present but belong to Story 002 closure. Lean mode skipped external QA/code-review gates.
- Tech debt logged: None
- Sprint status: Unchanged; no Prism Story 001 row exists in `production/sprint-status.yaml`.
- Next recommended: Prism Story 002 Deterministic Lane Rewards (`production/epics/prism-system/story-002-deterministic-lanes.md`) after readiness check, or run `/story-done` on it if the already-present implementation is ready for closure.
