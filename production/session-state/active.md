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

## Current Stage: Polish
`production/stage.txt` = `Polish`

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

## Session Extract - /story-done 2026-05-02
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/prism-system/story-002-deterministic-lanes.md` - Story 002: Deterministic Lane Rewards - Lanes 1/2/4/5
- Criteria: 4/4 passing; PS-01, PS-02, PS-07, and PS-12 covered by `tests/unit/prism/deterministic_lanes_test.rs`.
- Test Evidence: `cargo test -p server --test prism_deterministic_lanes_test` passed 6/6. `cargo check -p server` passed.
- Verification: `resolve_prism_draws` sorts pending `PrismCollected` messages by ascending player id and lane, routes Lanes 1/5 to `prism_strike`, routes Lanes 2/4 to `prism_reserve`, marks `PrismPresence` collected, and records stale duplicates in `DiscardLog` without reward or RNG consumption.
- Notes: Advisory only - story manifest version is 2026-04-30 while current control manifest is 2026-05-01. Advisory only - the exact ADR-016 schedule slot `.after(resolve_ecaflip_triggers).before(award_fake_objective_rewards)` is documented but not yet represented by concrete system labels in code because those neighboring systems are not present. Lean mode skipped external QA/code-review gates.
- Tech debt logged: None
- Sprint status: Unchanged per user instruction; no PRISM-002 row exists in `production/sprint-status.yaml`.
- Next recommended: Prism Story 003 Lane 3 RNG Draw Pipeline and Audit Log Ordering (`production/epics/prism-system/story-003-lane3-rng.md`) after readiness check.

## Session Extract - /story-done 2026-05-02
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/hud/story-004-scoreboard-dot-observer.md` - Story 004: Scoreboard Dot Message and State Machine
- Criteria: 6/6 passing; HUD-02, HUD-06, HUD-07, HUD-12b dot portion, HUD-26, and HUD-30 covered by `tests/integration/hud/scoreboard_dot_message_test.rs`.
- Test Evidence: `cargo test -p client --test scoreboard_dot_message_test` passed 4/4. `cargo check -p client` passed.
- Verification: `HudObjectiveUpdate` is a Bevy `Message` defined in `client/src/ui/shared.rs`, registered with `app.add_message::<HudObjectiveUpdate>()`, consumed by `MessageReader<HudObjectiveUpdate>`, and applied to `ScoreboardDotState` with a lane bounds guard before indexing. Dot alignment is derived from `BoardLayout::scoreboard_lane_center_x`.
- Notes: Advisory only - `TR-HUD-004` registry text still describes stale Hidden/Real/Fake/Destroyed wording while the current GDD and story require ALIVE/DESTROYED only. Advisory only - `BoardRenderingPlugin` / the real `ObjectiveDestroyed` drain is not present in `client/src` yet, so cross-plugin fanout is verified through the ordered Bevy `MessageWriter<HudObjectiveUpdate>` integration harness. Lean mode skipped external QA/code-review gates.
- Tech debt logged: None
- Sprint status: Unchanged per user instruction; no `HUD-004` row exists in `production/sprint-status.yaml`.
- Next recommended: HUD Story 006 ECONOMY_AUCTION Inline Gold Format (`production/epics/hud/story-006-economy-auction-inline-gold.md`) or HUD Story 007 GAME_OVER Freeze Mode (`production/epics/hud/story-007-game-over-freeze.md`) after readiness check.

## Session Extract - /story-done 2026-05-02
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/card-animations/story-007-damage-number-lifecycle.md` - Story 007: Damage number lifecycle (F2 despawn timer)
- Criteria: 3/3 passing; CA-8, CA-9, and CA-25 covered by `tests/unit/card-animations/damage_number_test.rs`.
- Test Evidence: `cargo test -p client --test card_animations_damage_number_test` passed 5/5.
- Verification: `client/src/card_animations/damage_numbers.rs` spawns world-space `Text2d` damage numbers with `DamageNumber`, `DespawnAfter(Timer)`, deterministic F3 jitter, and separate `TweenAnim` controllers targeting `Transform` and `TextColor`; `client/src/card_animations/queue.rs` computes F2 via `max(float_tween_duration_ms, fade_tween_duration_ms)` and enforces the strict sub-step budget.
- Notes: No blocking GDD or ADR deviation found. Advisory only - `TR-CAN-007` registry/epic wording still contains broader/stale `bevy_tweening 0.18` language, while the current GDD compatibility note and implementation use `bevy_tweening v0.15.0` with `TweenAnim`, `PlaybackState`, and `TweenState`. Lean mode skipped external QA/code-review gates.
- Tech debt logged: None
- Sprint status: Unchanged per user instruction; no `CARD-ANIM-007` row exists in `production/sprint-status.yaml`.
- Next recommended: Card Animations Story 009 CI boundary enforcement (`production/epics/card-animations/story-009-ci-boundary-enforcement.md`) after readiness check, or close the remaining implemented card-animation stories in the serialized story-done queue.

## Session Extract - /story-done 2026-05-02
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/hud/story-006-economy-auction-inline-gold.md` - Story 006: ECONOMY_AUCTION Inline Gold Format and TextSpan Rendering
- Criteria: 4/4 passing; HUD-17, HUD-08, HUD-29, and HUD-28 covered by `tests/unit/hud/economy_auction_inline_gold_test.rs`.
- Test Evidence: `production/qa/evidence/economy-auction-inline-gold-evidence.md`; `cargo test -p client --test hud_economy_auction_inline_gold_test` passed 4/4.
- Verification: `client/src/ui/hud/mod.rs` switches to `HudMode::EconomyAuction` on DRAFT_AUCTION, renders reserved gold through the existing child `TextSpan`, clears the span text on basic-mode return without despawning, and clamps reserved gold to total gold for display.
- Notes: Advisory only - manual visual walkthrough remains pending until a playable auction UI flow exists. Lean mode skipped external QA/code-review gates.
- Tech debt logged: None
- Sprint status: Unchanged per user instruction; no explicit `HUD-006` row exists in `production/sprint-status.yaml`.
- Next recommended: HUD Story 007 GAME_OVER Freeze Mode (`production/epics/hud/story-007-game-over-freeze.md`) after readiness check, or continue the serialized closure queue for implemented presentation stories.

## Session Extract - /story-done 2026-05-02
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/card-animations/story-005-placement-reveal-parallelism.md` - Story 005: Placement-reveal parallelism + PLACEMENT 250ms budget + PlacementCancelAllAnimsRequested
- Criteria: 3/4 automated criteria passing; CA-3, CA-12, and CA-21 covered by `tests/integration/card-animations/placement_reveal_test.rs`. CA-4b remains advisory manual visual evidence.
- Test Evidence: `cargo test -p client --test card_animations_placement_reveal_test` passed 9/9. `cargo check -p client` passed.
- Verification: `client/src/card_animations/placement.rs` starts all reveal entries from one message pass, clamps PLACEMENT animation durations through `placement_phase_duration`, and cancels `PlacementPhaseAnimator` controllers before snapping targets to `BoardLayout.cell_to_world` while preserving Z.
- Notes: No blocking GDD or ADR deviation found. Advisory only - no `production/qa/evidence/placement-reveal-evidence.md` screenshot/sign-off exists for CA-4b. Lean mode skipped external QA/code-review gates.
- Tech debt logged: None
- Sprint status: Unchanged per user instruction; no `CARD-ANIM-005` row exists in `production/sprint-status.yaml`.
- Next recommended: Add CA-4b visual evidence when a playable placement-to-resolution flow is available, or continue the serialized closure queue for implemented presentation stories.

## Session Extract - /story-done 2026-05-02
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/auction-system/story-004-bid-validation-gate.md` - Story 004: Bid Validation - 5-Condition Rejection Gate
- Criteria: 7/7 passing; AU2, AU3, AU16, AU17, AU12, AU18, and AU13 covered by `tests/unit/auction/bid_validation_gate_test.rs`.
- Test Evidence: `cargo test -p server --test auction_bid_validation_gate_test` passed 9/9. Adjacent auction/economy regression bundle passed 14 executable tests with 1 ignored future settlement guard. `cargo fmt -p server -- --check` and `cargo check -p server` passed.
- Verification: `process_bid_batch` rejects expired, too-low, current-leader, insufficient-gold, and hand-full bids with the expected `BidRejectedReason`, silently discards non-`LiveBidding` bids, and preserves same-tick arrival order so duplicate amount bids accept first and reject second.
- Notes: Advisory only - story manifest v2026-04-30 is older than current control manifest v2026-05-01. Advisory only - story/TR/ADR wording still references `C2SAuctionBid`; current GDD/protocol/code use `C2SPlaceBid`. Lean mode skipped external QA/code-review gates.
- Tech debt logged: None
- Sprint status: Unchanged per user instruction; no explicit `AUC-004` / Auction Story 004 row exists in `production/sprint-status.yaml`.
- Next recommended: Auction Story 005 Accepted Bid (`production/epics/auction-system/story-005-accepted-bid-reservation.md`) after readiness check, or continue the serialized closure queue for already implemented stories.

## Session Extract - /story-done 2026-05-02
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/round-state-machine/story-006-network-dispatch-wiring.md` - Story 006: Network Dispatch Wiring
- Criteria: 11/11 passing; RSM-26 dispatch and RSM-38 timeout coverage verified by `tests/integration/rsm/rsm_network_dispatch_test.rs`.
- Test Evidence: `cargo test -p server --test rsm_network_dispatch_test` passed 3/3. `cargo check --workspace` passed. `tests/evidence/rsm-story-006-tests.md` exists.
- Verification: `server/src/network/rsm_dispatch.rs` converts each `BroadcastPhaseChanged` into one `S2CPhaseChanged`, sends it with `ServerMultiMessageSender` on `ReliableChannel` to `NetworkTarget::All`, and preserves the test outbox seam. `server/src/network/mod.rs` schedules dispatch `.after(advance_phase)`. No Lightyear sender or legacy Bevy event API usage exists in `server/src/core/rsm/`.
- Notes: Advisory only - story manifest version is `2026-04-29` while current control manifest is `2026-05-01`. Advisory only - story-era timeout wording references `Draw`, while current GDD and implementation use `ResolutionTimeout`. Advisory only - broader `S2CPhaseChanged.timer_duration_ms` `u32` vs `Option<u32>` design drift remains outside this dispatch-wiring closure. Lean mode skipped external QA/code-review gates.
- Tech debt logged: None
- Sprint status: Unchanged per user instruction; no `RSM-006` row exists in `production/sprint-status.yaml`.
- Next recommended: Card Pool Story 004 (`production/epics/card-data-pool/story-004-shop-refresh-subscriber-session-ready.md`) or GSS Story 006 (`production/epics/game-session-system/story-006-game-over-teardown.md`) after readiness check.

## Session Extract - /story-done 2026-05-02
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/class-system/story-003-xelor-reserve-formulas.md` - Story 003: Xelor Reserve Formulas - Gelure, Xelorium, Rollback
- Criteria: 6/6 passing; CS-AC-04, CS-AC-05, CS-AC-05b, CS-AC-06, CS-AC-07, and CS-AC-08 covered by `tests/unit/class/xelor_reserve_test.rs`.
- Test Evidence: `cargo test -p server --test xelor_reserve_test` passed 6/6. `cargo check -p server` passed. `cargo fmt -p server -- --check` passed.
- Verification: `server/src/feature/class/resolution/effects.rs` implements `apply_gelure`, `pay_xelorium_cost`, `apply_xelorium`, and `apply_rollback` as synchronous helper functions over `PlayerEconomies` and board ECS state. Rollback consumes reserve through economy API, moves only friendly Minion/token units, clamps cells through supplied movement rules, and skips STUNned units.
- Notes: Advisory only - story manifest version is `2026-04-30` while current control manifest is `2026-05-01`. Advisory only - story notes point to `server/src/core/resolution/effects.rs`, while implementation lives in `server/src/feature/class/resolution/effects.rs` under the current feature-layer organization. Advisory only - no separate `tests/evidence/class-story-003-tests.md` exists; the required Logic evidence is the unit test file itself. Lean mode skipped external QA/code-review gates.
- Tech debt logged: None
- Sprint status: Unchanged per user instruction; no `CS-003` row exists in `production/sprint-status.yaml`.
- Next recommended: Class System Story 004 Garde-Temps Reserve Gate (`production/epics/class-system/story-004-garde-temps-reserve-gate.md`) after readiness check, or Class System Story 005 Miss Nuit per-round trigger (`production/epics/class-system/story-005-miss-nuit-trigger.md`) if its dependencies are ready.

## Session Extract - /story-done 2026-05-02
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/hud/story-007-game-over-freeze.md` - Story 007: GAME_OVER Freeze Mode
- Criteria: 4/4 passing; HUD-10, HUD-19, HUD-23, and GAME_OVER snap covered by `tests/unit/hud/game_over_freeze_test.rs` plus adjacent HUD regression checks.
- Test Evidence: `cargo test -p client --test hud_game_over_freeze_test` passed 2/2. Adjacent HUD regression bundle passed 14/14: `hud_phase_transitions_test`, `same_tick_tie_break_test`, and `scoreboard_dot_message_test`. `cargo check -p client` passed.
- Verification: `client/src/ui/hud/mod.rs` enters `HudMode::Frozen` on `RoundPhase::GameOver`, drains gold update/broadcast and objective messages without applying them while Frozen, keeps the round counter visible, writes `"GAME OVER"` through the phase label system, snaps numeric tween targets to authoritative state, and removes active HUD `TweenAnim` controllers on FROZEN entry.
- Notes: Advisory only - the required Frozen guards for post-GAME_OVER `S2CGoldUpdate` and `S2CGoldBroadcast` are implemented and verified by code review; the current unit test does not explicitly emit the story's literal `999`/`888` gold messages after GAME_OVER. Lean mode skipped external QA/code-review gates.
- Tech debt logged: None
- Sprint status: Unchanged per user instruction; no explicit `HUD-007` row exists in `production/sprint-status.yaml`.
- Next recommended: HUD Story 008 Reconnect Snapshot Rebuild (`production/epics/hud/story-008-reconnect-snapshot-rebuild.md`) after readiness check.

## Session Extract - /story-done 2026-05-02
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/board-lane-system/story-007-trap-trigger-mechanics.md` - Story 007: Trap Trigger Mechanics
- Criteria: 4/4 passing; BL-16, BL-17, BL-31, and NEW-007a covered by `tests/integration/board-lane-system/trap_trigger_test.rs`.
- Test Evidence: `cargo test -p server --test trap_trigger_test` passed 4/4. `cargo fmt -p server -- --check` passed.
- Verification: `server/src/feature/board/movement.rs` emits buffered `TrapTrigger` messages via `MessageWriter`, removes triggered Trap occupancy, despawns triggered Trap entities with `despawn()`, and commits standard movement, CHARGE movement, displacement destinations, and lane-change destinations through final-cell trap checks. `server/src/feature/board/plugin.rs` registers `TrapTrigger` with `app.add_message::<TrapTrigger>()`.
- Notes: No blocking GDD or ADR deviation found. Advisory only - BL-31 is verified with `World::run_system_once` against `commit_lane_change_destinations` rather than the story note's suggested `App::new()` + `BoardPlugin` harness. Lean mode skipped external QA/code-review gates.
- Tech debt logged: None
- Sprint status: Unchanged per user instruction; no `BOARD-007` row exists in `production/sprint-status.yaml`.
- Next recommended: Board Story 008 Objective Cell Detection (`production/epics/board-lane-system/story-008-objective-cell-detection.md`) after readiness check.

## Session Extract - /story-done 2026-05-02
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/prism-system/story-003-lane3-rng.md` - Story 003: Lane 3 RNG Draw Pipeline and Audit Log Ordering
- Criteria: 4/4 passing; PS-03, PS-10, PS-11, and PS-17 covered by `tests/unit/prism/lane3_rng_test.rs`.
- Test Evidence: `cargo test -p server --test prism_lane3_rng_test` passed 4/4. `cargo check -p server` passed.
- Verification: `server/src/feature/prism/system.rs` hand-full pre-check returns before `ServerRng::resolve_prism`, successful Lane 3 draws use a Minion/Spell `PoolFilter`, `PrismAuditEntry` records `Some(card_id)` or `None`, and pending `PrismCollected` messages are sorted by ascending player id and lane before reward processing.
- Notes: Advisory only - `TR-PRI-004` in `docs/architecture/tr-registry.yaml` still says Lane 3 hand-full emits `S2CPrismRewardDropped`; current `design/gdd/prism-system.md` Rule 7 and Story 004 say Lane 3 hand-full does not emit that message, and implementation follows the current GDD. Advisory only - story-required Prism `AuditLog` records `Some(card_id)` / `None`; broader ADR-005 `ServerRng.audit_log()` still records stub `result: None` for `resolve_prism`, so reconcile if the central RNG log becomes the post-game authority. Lean mode skipped external QA/code-review gates.
- Tech debt logged: None
- Sprint status: Unchanged per user instruction; no `PRISM-003` row exists in `production/sprint-status.yaml`.
- Next recommended: Prism Story 004 Hand-Full Rejection and Network Message Staging (`production/epics/prism-system/story-004-hand-full-network.md`) after readiness check, or Prism Story 005 Full-Set Respawn Cycle (`production/epics/prism-system/story-005-respawn-cycle.md`) after Story 004 closure.

## Session Extract - /story-done 2026-05-02
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/hud/story-009-same-tick-tie-break.md` - Story 009: Same-Tick Gold Tie-Break (Plugin-Level Integration)
- Criteria: 2/2 passing; HUD-20 and own reserved_gold isolation covered by `tests/integration/hud/same_tick_tie_break_test.rs`.
- Test Evidence: `cargo test -p client --test same_tick_tie_break_test` passed 3/3. `cargo check -p client` passed. `cargo fmt -p client -- --check` passed.
- Verification: `HudPlugin::build()` registers `handle_gold_broadcast_system.before(handle_gold_update_system)` in `HudSystemSet::MessageDrain`; local broadcasts update only `GoldDisplayState.reserved_gold`, while `S2CGoldUpdate` owns local `GoldDisplayState.gold`.
- Notes: Advisory only - the test injects `HudGoldBroadcastMessage` / `HudGoldUpdateMessage` directly into Bevy messages after the Lightyear drain seam. It verifies plugin-level handler ordering and field ownership, but does not instantiate real Lightyear receiver entities. Lean mode skipped external QA/code-review gates.
- Tech debt logged: None
- Sprint status: Unchanged per user instruction; no `HUD-009` row exists in `production/sprint-status.yaml`.
- Next recommended: HUD Story 008 Reconnect Snapshot Rebuild (`production/epics/hud/story-008-reconnect-snapshot-rebuild.md`) after readiness check, or continue the serialized closure queue for HUD Story 010 Numeric Tween Animation if its implementation is ready for closure.

## Session Extract - /story-done 2026-05-02
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/hud/story-010-numeric-tween-animation.md` - Story 010: Numeric Tween Animation
- Criteria: 2/3 covered and passing; HUD-12 numeric tween duration and cancel-and-replace covered by `tests/unit/hud/numeric_tween_animation_test.rs`. Layout screenshot evidence remains advisory and untested.
- Test Evidence: `cargo test -p client --test hud_numeric_tween_animation_test` passed 4/4. Adjacent HUD regression bundle passed 21/21: `hud_gold_mana_display_test`, `hud_economy_auction_inline_gold_test`, `hud_game_over_freeze_test`, `hud_phase_transitions_test`, and `hud_numeric_tween_animation_test`. `cargo check -p client`, `cargo fmt -p client -- --check`, and `git diff --check HEAD~1..HEAD` passed.
- Verification: `client/src/ui/hud/mod.rs` implements separate `GoldTweenTarget` and `ManaTweenTarget` backing components, lens-driven `f32` interpolation, `TweenAnim::set_tweenable()` cancel-and-replace, `StateSync` text rendering from tween targets, and `HudConfig.hud_tween_duration_ms` clamped to `<= 300ms`.
- Notes: Advisory only - no manual screenshot/sign-off evidence exists at `production/qa/evidence/numeric-tween-evidence.md`, so the 1280x720 and 1920x1080 layout-legibility criterion remains untested. Lean mode skipped external QA/code-review gates. Story wording references `Animator<T>` / `bevy_tweening 0.18`; current implementation uses the Bevy 0.18-compatible `bevy_tweening 0.15` API with `TweenAnim`.
- Tech debt logged: None
- Sprint status: Unchanged per user instruction; no `HUD-010` row exists in `production/sprint-status.yaml`.
- Next recommended: Continue the serialized closure queue with Hand UI Story 004 (`production/epics/hand-ui/story-004-draft-initial-grid.md`) if its implementation is ready for closure.

## Session Extract - /story-done 2026-05-02
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/hand-ui/story-004-draft-initial-grid.md` - Story 004: DRAFT_INITIAL Grid - Display & Purchase Flow
- Criteria: 6/6 passing; HU-07, HU-08, HU-09, HU-10, HU-10c, and HU-30 covered by `tests/integration/hand-ui/draft_initial_grid_test.rs`.
- Test Evidence: `cargo test -p client --test hand_ui_draft_initial_grid_test` passed 5/5. `cargo check -p client` passed.
- Verification: `client/src/ui/hand/mod.rs` populates pre-pooled grid slots from draft offerings, sends local purchase intents through `HandUiOutboundMessages`, applies pending purchase timeouts with `Time<Virtual>`, hides confirmed grid slots, animates acquired cards into fan slots with `TweenAnim`, locks visible grid slots at hand-full, and toggles the pre-pooled hand-full notification through `NotificationTimer`.
- Notes: Advisory only - live Lightyear wiring is not verified here; the implementation uses local Bevy messages/outbox rather than real `MessageReceiver<S2CDraftOffering>` / `MessageSender<C2SPurchaseCard>`. Advisory only - `CardAtlas` art/frame lookup is not implemented; HU-07 marks art rendering advisory. Advisory only - current `TR-HU-005` registry text also mentions the 45s timer and 5g budget via `S2CGoldBroadcast`; this story's acceptance criteria cover the grid display and purchase flow only. Lean mode skipped external QA/code-review gates.
- Tech debt logged: None
- Sprint status: Unchanged per user instruction; no explicit `HAND-UI-004` / `Hand UI Story 004` row exists in `production/sprint-status.yaml`.
- Next recommended: Hand UI Story 005 PLACEMENT Entry - Submit Button & Core Stage/Unstage (`production/epics/hand-ui/story-005-placement-submit-core.md`) after readiness check.

## Session Extract - /story-done 2026-05-02
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/game-session-system/story-005-lobby-disconnect-dual-signal.md` - Story 005: Lobby Disconnect - Dual-Signal Cancel
- Criteria: 9/9 top-level acceptance criteria passing; disconnect observer, heartbeat fallback, lobby timeout, shared cancel cleanup, first-signal-wins, plugin registration, and verification commands all passed.
- Test Evidence: `cargo fmt -p server -- --check` passed; `cargo test -p server --test dual_signal_disconnect_test --test lobby_timeout_test` passed 8/8; `cargo check -p server` passed.
- Verification: `lobby_timeout_check` now evaluates the room-session timeout path as `now > lobby_deadline.0 && !f4_session_ready(...)`; regression coverage proves a fully filled/class-locked lobby still cancels after the deadline because F4 is false once the deadline has passed.
- Notes: Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates. `production/sprint-status.yaml` has no matching row for this story; Sprint 4 lists S4-11 in the markdown plan only.
- Tech debt logged: None
- Next recommended: GSS Story 006 Game-Over Teardown (`production/epics/game-session-system/story-006-game-over-teardown.md`) or GSS Story 007 Reconnect Snapshot (`production/epics/game-session-system/story-007-reconnect-snapshot.md`) after readiness check.

## Session Extract - /story-done 2026-05-02
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/game-session-system/story-006-game-over-teardown.md` - Story 006: Game-Over Teardown
- Criteria: 7/7 acceptance criteria passing; teardown subscriber, reliable game-over broadcast, session resource removal, `ActiveSessions` cleanup, `ReconnectTracker` cleanup, idempotency, plugin scheduling, and verification commands all passed.
- Test Evidence: `cargo test -p server --test game_over_teardown_test` passed 4/4. `cargo check -p server` passed with zero warnings.
- Verification: `handle_game_over_teardown` consumes `MessageReader<GameOverEmitted>`, broadcasts `S2CGameOver` through `ServerMultiMessageSender` on `ReliableChannel`, removes `SessionConfig`, `ServerRng`, and remaining session resources after broadcast, transitions `LobbyState` to `GameOver`, and is scheduled `.after(advance_phase)`.
- Notes: Advisory only - story-era wording says `S2CGameOver.loser: PlayerId`, while current ADR/protocol source uses `Option<PlayerId>` for draw outcomes; implementation follows the current source of truth. Advisory only - live Lightyear peer delivery is code-verified, while the integration test verifies the outbox path rather than a full transport session. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates.
- Tech debt logged: None
- Sprint status: Unchanged per user instruction; no matching row exists in `production/sprint-status.yaml`.
- Next recommended: GSS Story 007 Reconnect Snapshot (`production/epics/game-session-system/story-007-reconnect-snapshot.md`) after readiness check.

## Session Extract - /story-done 2026-05-03
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/objective-system/story-002-fake-assignment-and-config-guards.md` - Story 002: Fake Assignment & Config Guards
- Criteria: 4/4 passing; OS-2, OS-23a, OS-23b, and OS-28 covered by `tests/unit/objective/fake_assignment_test.rs`.
- Test Evidence: `cargo test -p server --test fake_assignment_test` passed 5/5. Supporting config guard verification via `cargo test -p server game_config` passed.
- Verification: `assign_fake_objectives` clears and repopulates `HiddenObjectives`, sorts players by ascending `player_id`, consumes `ServerRng::assign_fake_objectives` in that order, assigns distinct fake lanes, and records all five lanes per player. `validate_objective_config` rejects `fake_count = 0`, `fake_count > lane_count - loss_threshold`, and `objective_hp = 0`; global `validate_game_config` also rejects those values before Lobby promotion.
- Notes: Advisory only - story wording mentions a `LOBBY_CANCELLED` / `S2CSessionCancelled` path for invalid objective config, but current implementation splits responsibility between pre-Lobby config validation and the Objective System DRAFT_INITIAL guard. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates.
- Tech debt logged: None
- Sprint status: Unchanged per user instruction; no matching row exists in `production/sprint-status.yaml`.
- Next recommended: Objective Story 003 identity unicast (`production/epics/objective-system/story-003-identity-unicast-delivery.md`) after readiness check, or continue the serialized closure queue with PRISM-004 per user direction.

## Session Extract - /story-done 2026-05-03
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/prism-system/story-004-hand-full-network.md` - Story 004: Hand-Full Rejection and Network Message Staging
- Criteria: 3/3 passing; PS-09, PS-20, and PS-23 covered by `tests/integration/prism/hand_full_network_test.rs`.
- Test Evidence: `cargo test -p server --test prism_hand_full_network_test` passed 7/7. `cargo check -p server` passed. `cargo fmt --all -- --check` passed.
- Verification: `resolve_prism_draws` uses `hand_push()` for deterministic and Lane 3 success paths, stages `S2CCardAcquired` and `S2CPrismRewardDropped` through `PrismNetworkOutbox`, defers owner messages through `ReconnectTracker.deferred_queue` when `snapshot_sent[player]` is false, and sends live owner-only messages via `ServerMultiMessageSender` on `ReliableChannel` to `NetworkTarget::Single(peer_id)`.
- Notes: Advisory only - `TR-PRI-004` in `docs/architecture/tr-registry.yaml` still says Lane 3 hand-full emits `S2CPrismRewardDropped`; current `design/gdd/prism-system.md` Rule 7 and Story 004 say Lane 3 hand-full does not emit that message, and implementation follows the current GDD. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates.
- Tech debt logged: None
- Sprint status: Unchanged per user instruction; no `PRISM-004` row exists in `production/sprint-status.yaml`.
- Next recommended: Hand UI Story 005 PLACEMENT Entry - Submit Button & Core Stage/Unstage (`production/epics/hand-ui/story-005-placement-submit-core.md`) after tracker update and queue handoff.

## Session Extract - /story-done 2026-05-03
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/hand-ui/story-005-placement-submit-core.md` - Story 005: PLACEMENT Entry - Submit Button & Core Stage/Unstage
- Criteria: 5/5 passing; HU-11, HU-13, HU-14, HU-16, and HU-17 covered by `tests/unit/hand-ui/placement_submit_core_test.rs`.
- Test Evidence: `cargo test -p client --test hand_ui_placement_submit_core_test` passed 5/5. `cargo check -p client` passed. `cargo fmt -p client -- --check` passed.
- Verification: `client/src/ui/hand/mod.rs` registers `GhostPlacementChanged` as a Bevy-internal message, resets `PendingPlacements` and activates the Submit button on PLACEMENT entry, stages valid drops into `PendingPlacements`, emits ghost messages, restores invalid drops without ghost messages, and locks the Submit button after the first `C2SSubmitPlacement` send.
- Notes: Advisory only - the unit test verifies the Hand UI local message/outbox seam and optional `MessageSender<C2SSubmitPlacement>` path, not a full live Lightyear transport session. Advisory only - current `TR-HU-002` also mentions cursor-to-cell mapping via `BoardLayout`; this story intentionally scopes that work to core stage/submit behavior, while drag highlight and target mapping are deferred to Story 006. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates.
- Tech debt logged: None
- Sprint status: Unchanged per user instruction; no matching Hand UI Story 005 row exists in `production/sprint-status.yaml`.
- Next recommended: Hand UI Story 006 PLACEMENT Drag Highlights (`production/epics/hand-ui/story-006-placement-drag-highlights.md`) after readiness check.

## Session Extract - /story-done 2026-05-03
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/game-session-system/story-007-reconnect-snapshot.md` - Story 007: Reconnect and Game Snapshot
- Criteria: 12/12 acceptance criteria passing; ADR-011 reconnect handshake, snapshot ordering, timeout/rejection, deferred queue flush, opponent reconnect broadcast, cleanup, and unicast guard requirements verified.
- Test Evidence: `cargo fmt --all -- --check`; `cargo test -p server --test snapshot_secret_strip_test`; `cargo test -p server --test reconnect_snapshot_test`; `cargo test -p server --test game_over_teardown_test`; `cargo test -p server --test prism_hand_full_network_test`; `cargo check -p server`; `cargo check --workspace`; `git diff --check`.
- Verification: `server/src/core/session/reconnect.rs` implements the `C2SHello` reconnect token path, `S2CHandshake` -> `S2CGameSnapshot` -> `S2CObjectiveIdentities` -> `S2CPhaseChanged` dispatch order, silent hello timeout closure, rejection without session existence leakage, and deferred queue flush after `snapshot_sent=true`. Acquisition, auction, and prism unicast paths now defer reconnecting player messages until snapshot completion. Sang Meprise reveal state is restored in the reconnect snapshot and re-sent when tracked.
- Notes: Advisory only - Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates. Live Lightyear peer delivery is code-verified through the server send paths and covered by outbox/guard integration tests rather than a full transport reconnect session.
- Tech debt logged: None
- Sprint status: Unchanged; no matching GSS-007 row exists in `production/sprint-status.yaml`.
- Next recommended: Continue the serialized closure queue with the next ready story after sprint/status review.

## Session Extract - /story-done 2026-05-03
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/hud/story-008-reconnect-snapshot-rebuild.md` - Story 008: Reconnect Snapshot Rebuild
- Criteria: 2/3 directly covered and passing; HUD-13 and HUD-27 covered by `tests/integration/hud/reconnect_snapshot_rebuild_test.rs`. HUD-14 screenshot evidence remains advisory and untested.
- Test Evidence: `cargo test -p client --test reconnect_snapshot_rebuild_test` passed 3/3. `cargo check -p client` passed. `cargo fmt -p client -- --check` passed.
- Verification: `client/src/ui/hud/mod.rs` drains `S2CGameSnapshot` through `MessageReceiver<S2CGameSnapshot>`, processes the last snapshot in `HudSystemSet::MessageDrain`, updates cached HUD entity handles without respawn, writes phase/round/gold/mana/dots, reapplies FROZEN for GAME_OVER snapshots, and runs before incremental gold/objective handlers.
- Notes: Advisory only - no manual screenshot/sign-off evidence exists at `production/qa/evidence/reconnect-snapshot-evidence.md`. Advisory only - current `TR-HUD-009` registry text is narrower than the story, but the broader rebuild requirement remains present in current HUD GDD Rule 13 and the story acceptance criteria. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates.
- Tech debt logged: None
- Sprint status: Unchanged per user instruction; no matching HUD Story 008 row exists in `production/sprint-status.yaml`.
- Next recommended: Create `production/qa/evidence/reconnect-snapshot-evidence.md` for HUD-14 visual sign-off, then continue the serialized closure queue with Hand UI Story 006 PLACEMENT Drag Highlights (`production/epics/hand-ui/story-006-placement-drag-highlights.md`) after readiness check.

## Session Extract - /story-done 2026-05-03
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/auction-system/story-005-accepted-bid-reservation.md` - Story 005: Accepted Bid - Gold Reservation Handoff & Timer Reset
- Criteria: 7/7 passing; AU4, AU20, M7-a, M7-c, AU5, AU6, and AU22 covered by `tests/integration/auction/accepted_bid_reservation_test.rs`.
- Test Evidence: `cargo test -p server --test accepted_bid_reservation_test` passed 5/5. `cargo check -p server` passed.
- Verification: `server/src/feature/auction/system.rs` performs release-before-reserve ordering, applies timer reset with `saturating_add(...).min(cap)`, clamps raw tick delta to 1000ms before `saturating_sub`, and dispatches accepted bid messages through `ReliableChannel` with `NetworkTarget::All` when reconnect deferral does not apply.
- Notes: Advisory only - the integration evidence tests the extracted `process_bid_batch` and timer seams rather than an `App::new()` harness with both plugins. Advisory only - implementation enqueues `S2CGoldBroadcast` from the auction acceptance path after Economy API calls; behavior matches AU/M7 outcomes, but story wording says the Economy System/API should fire those broadcasts. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates.
- Tech debt logged: None
- Sprint status: Unchanged per user instruction; no matching `AUC-005` row exists in `production/sprint-status.yaml`.
- Next recommended: Continue the serialized closure queue with Prism Story 005 Respawn Cycle (`production/epics/prism-system/story-005-respawn-cycle.md`).

## Session Extract - /story-done 2026-05-03
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/prism-system/story-005-respawn-cycle.md` - Story 005: Full-Set Respawn Cycle and Multi-Player Independence
- Criteria: 6/6 passing; PS-05, PS-13, PS-14, PS-16, PS-21, and PS-24 covered by `tests/unit/prism/respawn_cycle_test.rs`.
- Test Evidence: `cargo test -p server --test prism_respawn_cycle_test` passed 5/5. Prism regressions passed 23/23. `cargo check -p server` and `cargo check --workspace` passed.
- Verification: `resolve_prism_draws` runs respawn at the end after reward staging, clears `pending_respawn` after reset, resets `PrismPresence` for the respawned player, and sends `S2CPrismRespawned` to `NetworkTarget::All`.
- Notes: Advisory only - `TR-PRI-006` still mentions a `collected_this_round_grace` flag, while current GDD, ADR-016, and the story specify same-round prevention as a structural timing guarantee. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates.
- Tech debt logged: None
- Sprint status: Unchanged per user instruction; no matching `PRISM-005` row exists in `production/sprint-status.yaml`.
- Next recommended: Continue the serialized closure queue with Objective Story 003 Identity Unicast Delivery (`production/epics/objective-system/story-003-identity-unicast-delivery.md`).

## Session Extract - /story-done 2026-05-03
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/objective-system/story-003-identity-unicast-delivery.md` - Story 003: Identity Unicast Delivery
- Criteria: 5/5 passing; OS-17 advisory, OS-17a, OS-17b, OS-17c, and OS-17d covered by `tests/integration/objective/identity_unicast_test.rs` plus reconnect regression coverage.
- Test Evidence: `cargo test -p server --test objective_identity_unicast_test --test reconnect_snapshot_test` passed 10/10. `cargo test -p server --test objective_state_test` passed 4/4. `cargo check -p server` passed.
- Verification: Objective identity delivery sends owner-only reliable `S2CObjectiveIdentities` at DRAFT_INITIAL, builds payloads only from each owning player's `HiddenObjectives`, defers through `ReconnectTracker.snapshot_sent`, and keeps `ObjectiveIdentity` server-only with no replicated ECS component.
- Notes: Advisory only - `design/gdd/objective-system.md` still has older prose saying `ObjectiveIdentity` is replicated to the owner. Current OS-17/OQ4 text, `TR-OBJ-007`, and ADR-001 require reliable unicast and never-replication, which the implementation follows. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates.
- Tech debt logged: None
- Sprint status: Unchanged per user instruction; no matching `OBJECTIVE-003` row exists in `production/sprint-status.yaml`.
- Next recommended: Continue the serialized closure queue with CDP Story 004 Shop Refresh Subscriber SessionReady (`production/epics/card-data-pool/story-004-shop-refresh-subscriber-session-ready.md`).

## Session Extract - /story-done 2026-05-03
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/card-data-pool/story-004-shop-refresh-subscriber-session-ready.md` - Story 004: PlayerPools Draft Entry Init + ShopRefreshTriggered Handoff
- Criteria: 6/6 passing; AC-1 through AC-6 covered by `tests/integration/pool/session_ready_test.rs`.
- Test Evidence: `cargo test -p server --test pool_session_ready_test` passed 5/5. `cargo check -p server` passed.
- Verification: Card Pool initializes `PlayerPools` from `DraftStarted { phase: Initial }`, clears pool-owned session resources on `GameOverEmitted`, and schedules Card Acquisition after `CardPoolSet::Lifecycle` so same-frame `ShopRefreshTriggered` reads initialized pools.
- Notes: No blocking deviation found. Implementation uses Bevy 0.18 `MessageReader` / `MessageWriter`; lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates.
- Tech debt logged: None
- Sprint status: Updated `S3-07` to `done`, `completed: "2026-05-03"`, and top-level `updated: "2026-05-03"`.
- Next recommended: Continue the serialized closure queue with Hand UI Story 006 Placement Drag Highlights (`production/epics/hand-ui/story-006-placement-drag-highlights.md`).

## Session Extract - /story-done 2026-05-03
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/hand-ui/story-006-placement-drag-highlights.md` - Story 006: PLACEMENT Drag - Highlight Sets & TargetUnit
- Criteria: 5/5 passing; HU-12, HU-12b, HU-12c, HU-12d, and HU-20 covered by `tests/unit/hand-ui/placement_drag_highlights_test.rs`.
- Test Evidence: `cargo test -p client --test hand_ui_placement_drag_highlights_test` passed 5/5. `cargo check -p client` passed. `cargo fmt -p client -- --check` passed.
- Verification: Hand UI placement drag now manages board-cell highlight markers for Minion, TargetObj, and LaneWide cards; uses `TargetUnitHover` for unit-targeting without cell highlights; displays `NoValidTargetsOverlay` when TargetUnit has no valid targets; and cleans highlight/hover/overlay state on drag end or invalid drop.
- Notes: No blocking GDD/ADR deviation found. Visual overlay and outline pulse rendering sign-off remains advisory. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates.
- Tech debt logged: None
- Sprint status: Unchanged per user instruction; no matching `HAND-UI-006` row exists in `production/sprint-status.yaml`.
- Next recommended: Run a fresh sprint/status scan to choose the next ready implementation batch.

## Session Extract - /story-done 2026-05-03
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/card-data-pool/story-005-manual-refresh-cost-escalation.md` - Story 005: Manual Refresh Cost Escalation Integration
- Criteria: 7/7 passing; AC-1 through AC-7 covered by `tests/integration/pool/manual_refresh_test.rs`.
- Test Evidence: `cargo test -p server --test pool_manual_refresh_test` passed 7/7.
- Verification: `process_manual_refresh_shop_request` computes cost from `GameConfig.refresh_base_cost + min(refresh_count_this_draft, refresh_cap)`, spends through Economy API before drawing, refunds on draw failure, emits slots only on success, and increments `refresh_count_this_draft` only after a successful refresh. `apply_shop_refresh_trigger` resets the counter on `DraftInitial`, `AuctionLock`, `ShopOpen`, and `ShopUnlock`.
- Notes: No blocking GDD, ADR, Bevy 0.18, or Lightyear deviation found. AC-5 directly tests `AuctionLock`; code inspection confirms the same non-`ShopActive` gate covers `Inactive` and `DraftInitial`. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates.
- Tech debt logged: None
- Sprint status: Updated `S3-10` to `done`, `completed: "2026-05-03"`, and top-level `updated: "2026-05-03"`.
- Next recommended: Run a fresh sprint/status scan to choose the next ready implementation batch.

## Session Extract - /story-done 2026-05-03
- Verdict: COMPLETE
- Story: `production/epics/economy-system/story-006-network-dispatch-wiring.md` - Story 006: Network Dispatch Wiring
- Criteria: 9/9 passing; AC-1 through AC-9 covered by static verification, `tests/integration/economy/network_dispatch_test.rs`, `cargo check --workspace`, and the explicit channel grep.
- Test Evidence: `cargo test -p server --test economy_network_dispatch_test` passed 4/4. `cargo check --workspace` passed. `Select-String` found zero `UnreliableChannel` matches in `server/src/network/economy_dispatch.rs`.
- Verification: Economy network dispatch converts internal `server::core::economy` messages into shared protocol payloads, resolves `PlayerId` to `PeerId` through `PlayerConnectionMap`, unicasts private `S2CGoldUpdate` on `ReliableChannel`, broadcasts public `S2CGoldBroadcast` on `ReliableChannel`, warns and skips missing unicast peers, and keeps Lightyear imports out of economy core.
- Notes: No blocking GDD, ADR, Bevy 0.18, or Lightyear deviation found. `EconomyNetworkOutbox` is an intentional headless test adapter while live server sends still use `ServerMultiMessageSender::send::<M, ReliableChannel>`. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates.
- Tech debt logged: None
- Sprint status: Unchanged; no explicit ECO-006 or S4-14 entry exists in `production/sprint-status.yaml`.
- Next recommended: Run a fresh sprint/status scan to choose the next ready implementation batch.

## Session Extract - /story-done 2026-05-03
- Verdict: COMPLETE
- Story: `production/epics/board-lane-system/story-008-objective-cell-detection.md` - Story 008: Objective Cell Detection (F3)
- Criteria: 3/3 passing; BL-10, BL-11, and BL-25 covered by `tests/unit/board-lane-system/objective_detection_test.rs`.
- Test Evidence: `cargo test -p server --test objective_detection_test` passed 5/5. Repair evidence `cargo test -p server --test resolve_combat_scaffold_test resolve_combat_ss6_emits_unit_at_objective_messages` passed 1/1. `cargo check -p server` passed.
- Verification: `UnitAtObjective` derives `Message`, `BoardPlugin` registers it with `app.add_message::<UnitAtObjective>()`, `detect_objective_presence` writes objective hits through `MessageWriter`, and repair commit `b27db7d` wires objective detection into production combat sub-step 6.
- Notes: No blocking GDD, ADR, Bevy 0.18, or Lightyear deviation found. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates.
- Tech debt logged: None
- Sprint status: Unchanged per user instruction; no matching BOARD-008 row exists in `production/sprint-status.yaml`.
- Next recommended: Continue the serialized closure queue after sprint/status review; Board Story 009 Prism Collection remains ready but should be readiness-checked before implementation.

## Session Extract - /story-done 2026-05-03
- Verdict: COMPLETE
- Story: `production/epics/hand-ui/story-007-placement-instant-staging.md` - Story 007: PLACEMENT Instant Card Staging
- Criteria: 2/2 passing; HU-18 and HU-19 covered by `tests/unit/hand-ui/placement_instant_staging_test.rs`.
- Test Evidence: `cargo test -p client --test hand_ui_placement_instant_staging_test` passed 3/3 locally. Main integration at `d3a16d1` also passed `cargo fmt -p client -- --check`, `cargo test -p client --test hand_ui_placement_submit_core_test`, `cargo test -p client --test hand_ui_placement_drag_highlights_test`, `cargo check -p client`, and `cargo check -p client --features ui_picking`.
- Verification: Hand UI Instant placement drags show the drag sprite, clear board-cell highlights, add `FanPlateHighlighted` to the fan plate, stage valid plate drops as `PlayTarget::Instant`, emit `GhostPlacementChanged { target: Some(PlayTarget::Instant), card_id: Some(card_id) }`, increment Submit text, and cancel outside-plate drops without ghost messages.
- Notes: No blocking GDD, ADR, Bevy 0.18, or `ui_picking` guard deviation found. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates.
- Tech debt logged: None.
- Sprint status: Unchanged per user instruction; no matching `HAND-UI-007` row exists in `production/sprint-status.yaml`.
- Next recommended: Hand UI Story 008 PLACEMENT Un-staging (`production/epics/hand-ui/story-008-placement-unstaging.md`) after readiness check.

## Session Extract - /story-done 2026-05-03
- Verdict: COMPLETE
- Story: `production/epics/objective-system/story-004-damage-interface.md` - Story 004: Damage Interface
- Criteria: 7/7 passing; OS-3, OS-4, OS-5, OS-6, OS-16, OS-20, and OS-25 covered by `tests/unit/objective/damage_interface_test.rs`.
- Test Evidence: `cargo test -p server --test damage_interface_test` passed 7/7. `cargo fmt -p server -- --check`, `cargo check -p server`, and `git diff --check 033c212^..033c212` also passed.
- Verification: `take_damage(world, lane, attacker_player, amount)` is exported as the Objective System damage interface, uses unsigned `u32` damage, short-circuits `amount == 0`, resolves the opposing objective by lane, applies `saturating_sub`, marks the objective destroyed before queuing `ObjectiveDestroyed`, and repeated lethal calls no-op after the first destruction.
- Notes: No blocking GDD, ADR, Bevy 0.18, or Lightyear deviation found. Advisory only - story manifest `2026-04-29` is older than current control manifest `2026-05-01`; no applicable rule conflict found in lean review. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates.
- Tech debt logged: None.
- Sprint status: Unchanged per user instruction; no matching `OBJECTIVE-004` row exists in `production/sprint-status.yaml`.
- Next recommended: Continue the serialized closure queue after a fresh sprint/status scan.

## Session Extract - /story-done 2026-05-03
- Verdict: COMPLETE
- Story: `production/epics/combat-resolution/story-002-combat-modifier-stack.md` - Story 002: Combat Modifier Stack - Pure Function
- Criteria: 6/6 passing; CR-12, CR-13, CR-14, CR-15, CR-42, and CR-43 covered by `tests/unit/combat/modifier_stack_test.rs`.
- Test Evidence: `cargo test -p server --test modifier_stack_test` passed 7/7. `cargo test -p server --test game_config_defaults_test` passed 8/8. `cargo test -p server --test resolve_combat_scaffold_test` passed 4/4. `cargo fmt --all -- --check`, `cargo check -p server`, and `git diff --check 0e5ac46^..0e5ac46` passed.
- Verification: `apply_combat_modifier_stack` is pure Rust with no Bevy `World` access, computes intermediate arithmetic in `i32`, reads type-advantage bonuses from `GameConfig`, floors damage at zero, applies RESISTANCE/VULNERABILITY/ARMOR-PIERCING/SILENCE/type advantage in the specified order, and leaves base attacker stats unaffected.
- Notes: No blocking GDD, ADR, Bevy 0.18, or Lightyear deviation found. Advisory only - story manifest `2026-04-30` is older than current control manifest `2026-05-01`; no applicable rule conflict found in lean review. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates.
- Tech debt logged: None.
- Sprint status: Unchanged per user instruction.
- Next recommended: Run `/story-readiness` for AUC-006 before implementation.

## Session Extract - /story-done 2026-05-03
- Verdict: COMPLETE
- Story: `production/epics/board-lane-system/story-009-prism-collection.md` - Story 009: Prism Collection
- Criteria: 4/4 passing; BL-12, BL-13, BL-18, and BL-30 covered by `tests/unit/board-lane-system/prism_collection_test.rs`.
- Test Evidence: `cargo test -p server --test prism_collection_test` passed 6/6. `cargo test -p server --test standard_movement_test --test charge_movement_test --test trap_trigger_test` passed 14/14. `cargo check -p server` passed.
- Verification: `apply_standard_movement` emits `PrismCollected` only from the sub-step 5 endpoint check, reads `PrismPresence` to suppress already-collected prism duplicates, respects each player's own spawn-side prism cell, and leaves CHARGE-only and TELEPORT/displacement arrivals without prism collection.
- Notes: No blocking GDD, ADR, Bevy 0.18, or Lightyear deviation found. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates.
- Tech debt logged: None.
- Sprint status: Unchanged per user instruction; no matching `BOARD-009` row exists in `production/sprint-status.yaml`.
- Next recommended: Continue the serialized closure queue after a fresh sprint/status scan.

## Session Extract - /story-done 2026-05-03
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/auction-system/story-006-resolution-settlement.md` - Story 006: Resolution & Settlement - Case A/B & Post-Settlement Invariants
- Criteria: 4/4 passing; AU7-a, AU7-b, AU8, and AU14 covered by `tests/unit/auction/resolution_settlement_test.rs` and `tests/integration/auction/resolution_settlement_test.rs`.
- Test Evidence: `cargo test -p server --test auction_resolution_settlement_test` passed 3/3. `cargo test -p server --test auction_resolution_settlement_integration_test` passed 1/1. `cargo check -p server` passed.
- Verification: `settle_expired_auction` completes settlement synchronously, releases the winner reservation before `spend_gold`, writes `AuctionSettled` for winner and no-bid paths, queues `S2CAuctionSettled`, sends `S2CCardAcquired` only when `hand_push` succeeds, and resets `AuctionState` to `Idle`.
- Notes: Advisory only - AU14's integration test uses direct `App` setup with `auction_tick_system` and economy resources rather than `AuctionPlugin + EconomyPlugin`, because Auction plugin registration is explicitly Story 007 and no `AuctionPlugin` exists yet. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates.
- Tech debt logged: None.
- Sprint status: Unchanged per user instruction; no matching `AUC-006` row exists in `production/sprint-status.yaml`.
- Next recommended: Auction Story 007 Plugin Registration & System Scheduling (`production/epics/auction-system/story-007-auction-plugin-scheduling.md`) after readiness check.

## Session Extract - /story-done 2026-05-03
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/hand-ui/story-008-placement-unstaging.md` - Story 008: PLACEMENT Un-Staging - Board Ghosts & Instant Fan Slot
- Criteria: 3/3 passing; HU-21, HU-21b, and HU-21c covered by `tests/integration/hand-ui/placement_unstaging_test.rs`.
- Test Evidence: `cargo test -p client --test hand_ui_placement_unstaging_test` passed 4/4. `cargo check -p client` passed.
- Verification: Hand UI drains board ghost click and drag-start Bevy messages, exposes testable `FanZoneBounds`, un-stages Instant fan-slot ghosts through the fan click path, removes staged cards from `PendingPlacements`, emits `GhostPlacementChanged { target: None, card_id: Some(card_id) }`, restores `FanSlotState::Active`, updates Submit count text, and hides the reserve strip.
- Notes: Advisory only - `TR-HU-002` in `docs/architecture/tr-registry.yaml` still maps to HU-12/HU-13 drag-to-stage text, not HU-21/HU-21b/HU-21c un-staging. Current `design/gdd/hand-ui.md` Rule 8 contains the verified HU-21 criteria, so this is a registry traceability gap, not an implementation blocker. Advisory only - implementation uses the existing local `HandUiSystemSet::MessageDrain`; ADR-021's global `PresentationSet` wiring is not present in the current client architecture. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates.
- Tech debt logged: None.
- Sprint status: Unchanged per user instruction; no matching `HAND-UI-008` row exists in `production/sprint-status.yaml`.
- Next recommended: Hand UI Story 009 PLACEMENT Timer (`production/epics/hand-ui/story-009-placement-timer.md`) after readiness check.

## Session Extract - /story-done 2026-05-03
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/hand-ui/story-009-placement-timer.md` - Story 009: PLACEMENT Timer - Urgency, Grace Window & Submit Checkmark
- Criteria: 4/4 passing; HU-15, HU-15b, HU-22, and HU-23 covered by `tests/integration/hand-ui/placement_timer_test.rs`.
- Test Evidence: `cargo test -p client --test hand_ui_placement_timer_test` passed 4/4. `cargo check -p client` passed.
- Verification: Hand UI maintains `PlacementTimer` with remaining time, urgency single-shot state, grace-window state, and submitted state; registers `TimerUrgencyAudio` as a Bevy-internal message; decrements via `Time<Virtual>`; emits urgency once when crossing the 5s threshold; resolves the 200ms grace window by either staging a valid drop before submit or cancelling the active drag before submit; sends `C2SSubmitPlacement` through the existing submit path; shows `TimerSubmittedCheckmark`; and prevents duplicate timer-expiry submits after manual submit.
- Notes: Advisory only - `TR-HU-007` in `docs/architecture/tr-registry.yaml` maps only to HU-22 timer urgency, while this story also closes HU-15, HU-15b, and HU-23 from the current Hand UI GDD. Advisory only - HU-22 Amber color rendering and pulse animation remain lead-signoff concerns; the blocking state component and single-shot Bevy message are automated. Advisory only - implementation uses the current local `HandUiSystemSet::StateSync`; ADR-021's global `PresentationSet` wiring is not present in the current client architecture. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates.
- Tech debt logged: None.
- Sprint status: Unchanged per user instruction; no matching `HAND-UI-009` row exists in `production/sprint-status.yaml`.
- Next recommended: Continue the serialized closure queue with Combat Resolution Story 003 (`production/epics/combat-resolution/story-003-substep1-placement-appearance.md`) after readiness/status review.

## Session Extract - /story-done 2026-05-04
- Verdict: COMPLETE
- Story: `production/epics/combat-resolution/story-003-substep1-placement-appearance.md` - Story 003: Sub-step 1 - Placement Commit + APPEARANCE Triggers
- Criteria: 4/4 passing; CR-24, CR-38, CR-39, and CR-40 covered by `tests/unit/combat/substep1_placement_test.rs`.
- Test Evidence: `cargo test -p server --test substep1_placement_test` passed 4/4. `cargo check -p server` passed.
- Verification: Current `main` after integrated commit `7fdf4fd` keeps `server/src/feature/combat/mod.rs` unchanged from the COMBAT-003 implementation; SS1 enqueues `S2CPlacementReveal` before placement spawns, fires APPEARANCE before SS2, defers DEATH triggers until all SS1 APPEARANCE effects finish, applies queued CHANGE LANE before SS2, and applies STUN immediately so CHARGE X and standard movement helpers suppress movement.
- Notes: No blocking GDD, ADR, Bevy 0.18, or Lightyear deviation found. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates.
- Tech debt logged: None.
- Sprint status: Unchanged per user instruction; no matching COMBAT-003 row exists in `production/sprint-status.yaml`.
- Next recommended: Continue the serialized closure queue with Objective Story 005 (`production/epics/objective-system/story-005-destruction-consequence-path.md`).

## Session Extract - /story-done 2026-05-04
- Verdict: COMPLETE
- Story: `production/epics/objective-system/story-005-destruction-consequence-path.md` - Story 005: Destruction Consequence Path
- Criteria: 7/7 passing; OS-7, OS-9, OS-10, OS-13a, OS-14, OS-18a, and OS-21 covered by `tests/unit/objective/consequence_path_test.rs`.
- Test Evidence: `cargo test -p server --test consequence_path_test` passed 7/7.
- Verification: Current `main` after integrated commit `cf93a8d` queues `ObjectiveDestroyed` with `target_player_id`, lane, and `was_fake`; awards objective gold once on opponent destruction; increments real destroyed counters for real objectives including self-destruction; preserves fake counters/rewards for self-destroyed fakes; and preserves lane-ascending queue order through `take_damage`.
- Notes: No blocking GDD, ADR, Bevy 0.18, or Lightyear deviation found. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates.
- Tech debt logged: None.
- Sprint status: Unchanged per user instruction; no matching OBJECTIVE-005 row exists in `production/sprint-status.yaml`.
- Next recommended: Objective Story 006 D4 Fake Reward Draw (`production/epics/objective-system/story-006-d4-fake-reward-draw.md`) after readiness check.

## Session Extract - /story-done 2026-05-04
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/auction-system/story-007-auction-plugin-scheduling.md` - Story 007: Plugin Registration & System Scheduling
- Criteria: 5/5 plugin/CI criteria passing; AU1-b-network deferred/open pending ADR-008 Lightyear FIFO integration test and remains a sprint-review note, not a blocker.
- Test Evidence: CI grep gate exists in `.github/workflows/tests.yml`. Local checks passed: no deprecated Bevy event API in `server/src/feature/auction/`, exactly one `ResMut<AuctionState>`, targeted auction/economy test set, and `cargo check -p server`.
- Verification: Current `main` after integrated commit `ea5d88d` registers `AuctionPlugin` in `server/src/main.rs`, inserts `AuctionState::default()`, registers required auction Bevy messages, configures `AuctionSet::Tick.before(RsmSet::Tick)`, and schedules `auction_tick_system.after(reconnect_snapshot_system)`.
- Notes: No blocking GDD, ADR, Bevy 0.18, or Lightyear deviation found. Advisory only - available smoke report `production/qa/smoke-2026-04-30.md` predates AUC-007, so closure used local CI-equivalent checks. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates.
- Tech debt logged: None.
- Sprint status: Unchanged per approval boundary; no matching `AUC-007` row exists in `production/sprint-status.yaml`.
- Next recommended: Auction Story 008 Pool Integration (`production/epics/auction-system/story-008-pool-integration.md`) after readiness check.

## Session Extract - /story-done 2026-05-04
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/hand-ui/story-011-reserve-mana-strip.md` - Story 011: Reserve Mana Split Strip - Per-Staged-Card Controls
- Criteria: 3/3 passing; HU-25, HU-26, and HU-27 covered by `tests/unit/hand-ui/reserve_mana_strip_test.rs`.
- Test Evidence: `cargo test -p client --test hand_ui_reserve_mana_strip_test` passed 3/3. `cargo check -p client` passed.
- Verification: Current `main` includes integrated commit `bd8c15b`; reserve strip plus clicks increment to the per-card ceiling, disabled plus buttons ignore further clicks, other staged cards reduce the ceiling without auto-decrementing, and cost-0 cards keep the reserve strip hidden with no click effect.
- Notes: Advisory only - `TR-HU-004` in `docs/architecture/tr-registry.yaml` does not currently describe the reserve strip behavior covered by HU-25/HU-26/HU-27; current `design/gdd/hand-ui.md` Rule 13 is the verified behavior source. Advisory only - VA-9 specifies a 96 px strip width while the implementation uses 104 px; logic criteria pass and visual sizing can be reconciled in UI polish if needed. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates.
- Tech debt logged: None.
- Sprint status: Unchanged per approval boundary; no matching `HAND-UI-011` row exists in `production/sprint-status.yaml`.
- Next recommended: Hand UI Story 010 Submit Pre-Validation (`production/epics/hand-ui/story-010-submit-prevalidation.md`) after readiness check, or continue the Sprint 5 Must Have combat spine if that remains the priority.

## Session Extract - /story-done 2026-05-04
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/board-lane-system/story-010-displacement-keywords.md` - Story 010: Displacement Keywords and Spawn Range Expansion
- Criteria: 10/10 passing; BL-15, BL-19, BL-20, BL-23, BL-24, BL-26, BL-28, NEW-010a, NEW-010b, and NEW-010c covered by `tests/unit/board-lane-system/displacement_keywords_test.rs`.
- Test Evidence: `cargo test -p server --test displacement_keywords_test` passed 9/9. `cargo check -p server` passed.
- Verification: Current `main` includes repair commit `61e7042`; REPEL clamps at each player's spawn boundary, CHANGE LANE silently no-ops at lane boundaries and same-owner full slots, IRREMOVABLE units ignore displacement, fake objective destruction expands `SpawnRangeState` with clamp-to-2 behavior for the next PLACEMENT validation, and enemy ATTRACT stops one cell short of the caster per current GDD BL-24.
- Notes: Advisory only - the story's inline BL-24 acceptance text and QA wording are stale; implementation and tests follow current GDD BL-24 enemy ATTRACT one-cell-short behavior. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates.
- Tech debt logged: None.
- Sprint status: Unchanged per approval boundary; no matching `BOARD-010` row exists in `production/sprint-status.yaml`.
- Next recommended: Continue the Sprint 5 queue from `production/sprints/sprint-5.md`, or run a fresh sprint/status scan before pulling the next story.

## Session Extract - /story-done 2026-05-04
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/card-data-pool/story-006-network-dispatch-wiring.md` - Story 006: Network Dispatch Wiring
- Criteria: 4/4 passing; AC-1 through AC-4 covered by `tests/unit/network/shop_dispatch_test.rs`, with AC-4 also supported by the reconnect snapshot regression.
- Test Evidence: `cargo test -p server --test shop_dispatch_test` passed 5/5. `cargo test -p server --test reconnect_snapshot_test acquisition_unicast_helpers_defer_while_snapshot_pending` passed 1/1 filtered test. `cargo check -p server` passed.
- Verification: Current `main` contains integrated commit `4d21482`; `S2CShopSlots` and `S2CDraftOffering` remain reliable server-to-client protocol messages, dispatch helpers resolve only the owning player's `PeerId`, partial shop slots preserve `None`, and reconnect-pending acquisition messages queue through `ReconnectTracker.deferred_queue`.
- Notes: No blocking GDD, ADR, Bevy 0.18, or Lightyear deviation found. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates.
- Tech debt logged: None.
- Sprint status: Unchanged per user instruction; `production/sprint-status.yaml` was not touched and has no matching CDP-006/S5-04 row.
- Next recommended: Continue the Sprint 5 queue from `production/sprints/sprint-5.md`; likely next Must Have is Combat Story 004 Movement + Collision after readiness/status review.

## Session Extract - /story-done 2026-05-04
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/combat-resolution/story-004-movement-collision.md` - Story 004: Sub-steps 2 & 5 - Movement + Collision
- Criteria: 5/5 passing; CR-5, CR-8, CR-9, CR-31, and CR-44 movement/collision clauses covered by `tests/unit/combat/movement_collision_test.rs`.
- Test Evidence: `cargo test -p server --test movement_collision_test` passed 5/5. `cargo check -p server` passed.
- Verification: Current `main` after integrated commit `408c34a` executes CHARGE X movement in SS2 and standard movement in SS5 through the same bounded tick loop, suppresses STUNned unit movement, uses `apply_f1` with `i16` intermediate arithmetic, halts on enemy WALL cells, halts path-crossing units at pre-crossing cells, logs separate SS2/SS5 `UnitMoved` trace entries, and preserves the RANGE-vs-WALL SS5 movement exemption.
- Notes: Advisory only - CR-5, CR-8, and CR-44 include later SS3/SS6 attack, damage, and CombatDamage clauses in the current GDD text; those are explicitly out of scope for COMBAT-004 and remain owned by Combat Stories 005, 007, and 008. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates.
- Tech debt logged: None.
- Sprint status: Updated S5-06 in `production/sprint-status.yaml` to `done` with completion date 2026-05-04.
- Next recommended: Combat Story 5 FIRST STRIKE Attacks (`production/epics/combat-resolution/story-005-substep3-first-strike.md`) after readiness check.

## Session Extract - /story-done 2026-05-04
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/combat-resolution/story-005-substep3-first-strike.md` - Story 005: Sub-step 3 - FIRST STRIKE Attacks
- Criteria: 5/5 passing; CR-1, CR-2, CR-4, CR-22, and CR-37 covered by `tests/unit/combat/substep3_first_strike_test.rs`.
- Test Evidence: `cargo test -p server --test substep3_first_strike_test` passed 5/5. `cargo check -p server` passed.
- Verification: Current `main` after integrated commit `e1733be` collects SS3 FIRST STRIKE attacks from snapshot data, uses `apply_combat_modifier_stack` per attack, applies HP loss with `saturating_sub`, emits SS3/SS6 RANGE+FIRST STRIKE `CombatDamage` trace entries, records lethal SS3 kill credit, and emits FINAL BLOW in SS3 while leaving killed units present for SS4.
- Notes: Advisory only - CR-2 and CR-37 include later SS4 DEATH/gold-award effects in current GDD wording. COMBAT-005 seeds lethal state/kill credit and leaves dead units present; COMBAT-006 owns SS4 removal, DEATH chains, and kill-gold emission. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates.
- Tech debt logged: None.
- Sprint status: Updated S5-07 in `production/sprint-status.yaml` to `done` with completion date 2026-05-04.
- Next recommended: Combat Story 6 Dead Removal + DEATH Chains + Kill Gold (`production/epics/combat-resolution/story-006-substep4-dead-removal.md`) after readiness check.

## Session Extract - /story-done 2026-05-04
- Verdict: COMPLETE
- Story: `production/epics/auction-system/story-008-pool-integration.md` - Story 008: Pool Integration - draw_auction_card, distribute, Legendary Stratification
- Criteria: 3/3 passing; AU8-pool, AU15, and AU21 covered by `tests/integration/auction/pool_integration_test.rs`.
- Test Evidence: `cargo test -p server --test auction_pool_integration_test` passed 5/5. `cargo check --workspace` passed.
- Verification: Current `main` after integrated commit `c6a1d86` calls `PlayerPool::draw_auction_card()` at SELECTING entry through a round-eligible auction-pool view, calls `distribute(card_id)` immediately after a successful draw, handles empty eligible pools with same-invocation `AuctionSettled { winner: None, final_price: 0, card_id: CardId(0) }` and no `S2CAuctionCard`, and excludes Legendary cards before `GameConfig.legendary_pool_entry_round`.
- Notes: No blocking GDD, ADR, Bevy 0.18, or Lightyear deviation found. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates.
- Tech debt logged: None.
- Sprint status: Updated S5-14 in `production/sprint-status.yaml` to `done` with completion date 2026-05-04.
- Next recommended: Objective Story 6 D4 Fake Reward Draw (`production/epics/objective-system/story-006-d4-fake-reward-draw.md`) after readiness check.

## Session Extract - /story-done 2026-05-04
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/server-rng/story-003-determinism-session-reset.md` - Story 003: Determinism Proof & Session Reset
- Criteria: 10/10 passing; determinism, RNG13 session reset, RNG15 overflow wrap, and deferred RNG8/RNG9/RNG10/RNG14 documentation covered by `server/src/foundation/rng.rs` tests and comments.
- Test Evidence: `tests/unit/foundation/server_rng_determinism_test.rs` exists; runnable tests are embedded in `server/src/foundation/rng.rs`. `cargo test -p server foundation::rng::tests` passed 19/19. `cargo check -p server` passed.
- Verification: `ServerRng::from_seed` creates a fresh `SessionInit` sentinel at seed index 0, starts gameplay at seed index 1, keeps first gameplay seed index independent from prior sessions, uses `wrapping_add` for overflow, and records the overflow audit entry with `seed_index = u32::MAX`.
- Notes: Advisory only - RNG source comments still use shorthand `TR-RNG-04`, `RNG13`, and `RNG15` while the story/registry trace is correct (`TR-RNG-004`, `TR-RNG-001`, `TR-RNG-007`). Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates.
- Tech debt logged: None.
- Sprint status: Updated S5-18 in `production/sprint-status.yaml` to `done` with completion date 2026-05-04.
- Next recommended: Continue Sprint 5 Must Have combat spine with Combat Story 6 Dead Removal + DEATH Chains + Kill Gold (`production/epics/combat-resolution/story-006-substep4-dead-removal.md`) after readiness check.

## Session Extract - /story-done 2026-05-04
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/combat-resolution/story-006-substep4-dead-removal.md` - Story 006: Sub-step 4 - Dead Removal + DEATH Chains + Kill Gold
- Criteria: 3/3 passing; CR-16, CR-23, and CR-25 covered by `tests/unit/combat/substep4_dead_removal_test.rs`.
- Test Evidence: `cargo test -p server --test substep4_dead_removal_test` passed 3/3. `cargo check -p server` passed. `git diff --check` passed.
- Verification: Current `main` after integrated commit `ea43240` removes SS4 dead units, drains SS3 kill records into `GoldAwarded { amount: 1, reason: Kill }` entries and economy gold, keeps FINAL BLOW in the kill sub-step with no SS4 FINAL BLOW trace, and processes DEATH chains sequentially through `ChainDeathBuffer` without recursive DEATH dispatch.
- Notes: Advisory only - implementation dispatches `UnitDied` with Bevy's targeted `EntityEvent` path via `world.trigger(...)` rather than the manifest wording `world.trigger_targets(...)`; compile and CR-25 coverage confirm equivalent synchronous targeted behavior for this story. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates.
- Tech debt logged: None.
- Sprint status: Updated S5-08 in `production/sprint-status.yaml` to `done` with completion date 2026-05-04.
- Next recommended: Combat Story 7 Standard Combat + SHIELD + COUNTERATTACK (`production/epics/combat-resolution/story-007-substep6-combat-shield-counterattack.md`) after readiness check.

## Session Extract - /story-done 2026-05-04
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/objective-system/story-006-d4-fake-reward-draw.md` - Story 006: D4 Fake Reward Draw
- Criteria: 8/8 passing; OS-8, OS-11, OS-12, OS-15, OS-19, OS-22, OS-26, and OS-27 covered by `tests/unit/objective/fake_reward_test.rs`.
- Test Evidence: `cargo test -p server --test fake_reward_test` passed 8/8. `cargo test -p server --test consequence_path_test --test damage_interface_test` passed 14/14. `cargo check -p server` passed. `git diff --check` passed.
- Verification: Current `main` includes integrated commit `d46e812`; fake opponent destruction increments `fake_objectives_destroyed`, consumes `AwardFakeObjectiveReward`, emits exactly one D4 outcome, emits two mana-cap messages for two mana rolls, skips the draw seed and emits +1 gold when the hand is full, treats pool exhaustion as a silent no-op after consuming the draw seed, and passes `FAKE_REWARD_POOL_FILTER` into `draw_random`.
- Notes: Advisory only - current `TR-OBJ-005` in `docs/architecture/tr-registry.yaml` is narrower than the current story/GDD coverage; OS-15, OS-22, and OS-27 are verified against `design/gdd/objective-system.md` and the story evidence. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates.
- Tech debt logged: None.
- Sprint status: Updated S5-15 in `production/sprint-status.yaml` to `done` with completion date 2026-05-04.
- Next recommended: Combat Story 7 Standard Combat + SHIELD + COUNTERATTACK (`production/epics/combat-resolution/story-007-substep6-combat-shield-counterattack.md`) after readiness check.

## Session Extract - /story-done 2026-05-04
- Verdict: COMPLETE
- Story: `production/epics/presentation-layer/story-001-presentation-plugin-set-and-phase-sink.md` - Story 001: PresentationPlugin, PresentationSet, and Phase Sink
- Criteria: 15/15 passing; ADR-021 R1/R5 and control-manifest presentation rules accepted as sufficient traceability for this ADR-only infrastructure.
- Test Evidence: `tests/integration/presentation/presentation_plugin_scaffold_test.rs`; `cargo test -p client --test presentation_plugin_scaffold_test` passed 3/3. `cargo test -p client --test hud_phase_transitions_test --test hand_ui_phase_state_machine_test --test card_animations_plugin_scaffold_test` passed 16/16. `cargo check -p client` passed.
- Verification: Current `main` includes integrated commit `1c5c40f`; `PresentationPlugin` exports and registers the shared presentation layer, configures `PresentationSet` order `PhaseTransition -> MessageDrain -> StateSync -> AnimationTick`, owns the single `MessageReceiver<S2CPhaseChanged>` phase sink, applies last-write-wins phase updates while ignoring timer display data, bridges HUD/Hand UI/Card Animations into ADR-021 scheduling, and documents Board Rendering and Shop/Auction UI slots.
- Notes: No blocking GDD, ADR, Bevy 0.18, or Lightyear deviation found. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates.
- Tech debt logged: None.
- Sprint status: Unchanged per user instruction; no matching `PRESENTATION-001` row exists in `production/sprint-status.yaml`.
- Next recommended: Combat Story 7 Standard Combat + SHIELD + COUNTERATTACK (`production/epics/combat-resolution/story-007-substep6-combat-shield-counterattack.md`) after readiness check.

## Session Extract - /story-done 2026-05-04
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/combat-resolution/story-007-substep6-combat-shield-counterattack.md` - Story 007: Sub-step 6 - Standard Combat + SHIELD + COUNTERATTACK
- Criteria: 7/7 passing; CR-6, CR-7, CR-20, CR-21, CR-29, CR-35, and CR-36 covered by `tests/unit/combat/substep6_combat_shield_counterattack_test.rs`.
- Test Evidence: `cargo test -p server --test substep6_combat_shield_counterattack_test` passed 7/7. `cargo test -p server --test movement_collision_test --test substep3_first_strike_test --test substep4_dead_removal_test` passed 13/13. `cargo check -p server` passed. `git diff --check` passed.
- Verification: Current `main` includes integrated commit `2c8f752`; SS6 standard unit-vs-unit combat runs through `execute_standard_combat`, SHIELD absorbs grouped incoming damage once per sub-step and persists until consumed, COUNTERATTACK fires for same-cell and collision-halt adjacent melee contact after incoming damage or SHIELD absorption, and RANGE attackers at distance do not trigger COUNTERATTACK.
- Notes: Advisory only - current GDD CR-21 wording still says COUNTERATTACK fires before the SHIELD absorption check, while active `TR-CR-006`, this story, and ADR-017 specify after incoming damage or SHIELD absorption. Implementation follows the active TR/story contract. No Story 008 RANGE targeting scope creep found; this closure only verifies the narrow RANGE + FIRST STRIKE support needed by CR-20 and CR-29. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates.
- Tech debt logged: None.
- Sprint status: Updated S5-09 in `production/sprint-status.yaml` to `done` with completion date 2026-05-04.
- Next recommended: Combat Story 8 RANGE Targeting (`production/epics/combat-resolution/story-008-range-targeting.md`) after readiness check.

## Session Extract - /story-done 2026-05-04
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/board-rendering/story-001-plugin-scaffold-board-layout-card-atlas.md` - Story 001: Plugin Scaffold, BoardLayout, and CardAtlas
- Criteria: 8/8 passing; plugin registration, session resource lifecycle, CardAtlas handles, BoardLayout coordinate formula, invalid-input asserts, phase receiver ownership, and PresentationPlugin dependency verified by `tests/unit/board_rendering/plugin_scaffold_test.rs`.
- Test Evidence: `cargo test -p client --test board_rendering_plugin_scaffold_test` passed 9/9. `cargo test -p client --test hand_ui_placement_drag_highlights_test --test card_animations_placement_reveal_test` passed 14/14. `cargo check -p client` passed. `git diff --check` passed.
- Verification: Current `main` includes integrated commit `b5abcd5`; `BoardRenderingPlugin` inserts/removes `BoardLayout` and `CardAtlas` on `ClientState::InSession` boundaries, `CardAtlas` carries `Handle<Image>` plus `Handle<TextureAtlasLayout>`, and Presentation registration uses the completed `PresentationPlugin`, `PresentationSet`, `phase_sink_system`, and `CurrentClientPhase` path from PRESENTATION-001.
- Notes: Advisory only - `TR-BR-002` in `docs/architecture/tr-registry.yaml` says `cell_to_world(lane, cell) -> Vec3`, while the current Board Rendering GDD F1 formula and ADR-021 specify `Vec2`; implementation follows the current GDD/ADR and existing Presentation consumers. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates.
- Scope: No scope creep found into snapshots, unit spawning, new tweens, resolution animation queue, shop/auction UI, or visual asset generation.
- Tech debt logged: None.
- Sprint status: Unchanged; no matching BOARD-RENDERING-001 row exists in `production/sprint-status.yaml`.
- Next recommended: Board Rendering Story 002 (`production/epics/board-rendering/story-002-board-grid-camera-and-layer-constants.md`) after readiness check, or continue Sprint 5 Must Have Combat Story 8 if the current sprint priority remains unchanged.

## Session Extract - /story-done 2026-05-04
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/objective-system/story-007-resolution-phase-subscription.md` - Story 007: ResolutionPhaseEntered Subscription & RESOLUTION-end Sync
- Criteria: 4/4 passing for blocking server scope; OS-13a, OS-18a, ResolutionPhaseEntered subscription, and OS-24 covered by `tests/integration/objective/resolution_sync_test.rs`; OS-18b ECS-side no-duplicate consequence path covered by `tests/unit/objective/damage_interface_test.rs`.
- Test Evidence: `cargo test -p server --test objective_resolution_sync_test` passed 5/5. `cargo test -p server --test fake_reward_test --test consequence_path_test --test damage_interface_test` passed 22/22. `cargo check -p server` passed. `git diff --check` passed.
- Verification: Current `main` includes integrated commit `1c8ef2a`; `ResolutionPhaseEntered` is consumed through `MessageReader`, `ResolutionComplete` drains `PendingObjectiveEvents` only at RESOLUTION-end, sorted by ascending lane, writes internal `ObjectiveDestroyed`, sends one reliable `S2CResolutionEvent` batch to `NetworkTarget::All`, and leaves Sang Meprise visibility state outside the suppression path.
- Notes: Advisory only - OS-18b's single visible HP replication update is a live Lightyear transport/session assertion and remains deferred per story text. No combat objective damage, GAME_OVER, UI/result screen, design asset, or unrelated scope creep found. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates.
- Tech debt logged: None.
- Sprint status: Updated S5-16 in `production/sprint-status.yaml` to `done` with completion date 2026-05-04.
- Next recommended: Hand UI Story 10 Submit Pre-Validation (`production/epics/hand-ui/story-010-submit-prevalidation.md`) after readiness check, or continue Sprint 5 Nice to Have combat work if Sprint 5 should stay server-focused.

## Session Extract - /story-done 2026-05-04
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/combat-resolution/story-008-range-targeting.md` - Story 008: Sub-step 6 - RANGE Targeting
- Criteria: 5/5 passing; CR-3, CR-4, CR-28, CR-44, and CR-45 covered by `tests/unit/combat/range_targeting_test.rs`.
- Test Evidence: `cargo test -p server --test range_targeting_test` passed 6/6. Adjacent regression command passed 20/20 across `movement_collision_test`, `substep3_first_strike_test`, `substep4_dead_removal_test`, and `substep6_combat_shield_counterattack_test`. `cargo check -p server` passed. `git diff --check` passed.
- Verification: Current `main` includes integrated implementation commit `dd7bd50`; RANGE targeting selects the nearest forward valid target without movement or RNG for single-nearest cases, orders equidistant targets by `(distance, target_cell, target_unit_id)`, consumes exactly one `RangeEquidistantSelect` seed for equidistant nearest cases, emits distinct SS3 and SS6 damage for RANGE + FIRST STRIKE, excludes behind-attacker enemies, attacks WALL from the current cell, and reacquires a fresh SS6 target after SS4 removal.
- Notes: No blocking GDD, ADR, Bevy 0.18, or ServerRng deviation found. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates.
- Tech debt logged: None.
- Sprint status: Updated S5-10 in `production/sprint-status.yaml` to `done` with completion date 2026-05-04.
- Next recommended: Combat Story 9 Objective Damage + GAME_OVER (`production/epics/combat-resolution/story-009-objective-damage-gameover.md`) after readiness check.

## Session Extract - /story-done 2026-05-04
- Verdict: COMPLETE
- Story: `production/epics/combat-resolution/story-009-objective-damage-gameover.md` - Story 009: Sub-step 6 - Objective Damage + GAME_OVER
- Criteria: 6/6 passing; CR-10, CR-11, CR-17, CR-18, CR-19, and CR-27 covered by `tests/unit/combat/objective_damage_gameover_test.rs`.
- Test Evidence: `cargo test -p server --test objective_damage_gameover_test` passed 6/6. Requested adjacent regression command passed 36/36 across RANGE targeting, SS6 shield/counterattack, SS4 dead removal, objective consequence path, fake rewards, and objective resolution sync. `cargo check -p server` passed. `git diff --check` passed.
- Verification: Current `main` includes integrated implementation commit `16398c6`; surviving Cell 8 attackers damage objectives and remain, units removed before objective damage do not damage objectives, objective destruction awards +3 objective gold with no kill gold, second real objective destruction reaches RSM-owned `GameOverEmitted` after normal `ResolutionComplete`, mutual second real objective destruction resolves Draw after all destructions, and objective HP saturates to zero.
- Notes: No blocking GDD, ADR, Bevy 0.18, or economy/RSM ownership deviation found. Combat Resolution does not directly broadcast `S2CGameOver`; Game Session/network dispatch owns that after `GameOverEmitted`. Full `S2CResolutionEvent` completeness/order verification remains COMBAT-011 scope. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates.
- Tech debt logged: None.
- Sprint status: Updated S5-11 in `production/sprint-status.yaml` to `done` with completion date 2026-05-04.
- Next recommended: Review COMBAT-010 Persistent Keyword States and COMBAT-011 ResolutionEvent Log Completeness readiness before starting either.

## Session Extract - /story-done 2026-05-05
- Verdict: COMPLETE
- Story: `production/epics/combat-resolution/story-010-persistent-keyword-states.md` - Story 010: Persistent Keyword States - INJURED, LEADER, OUTNUMBERED
- Criteria: 4/4 passing; CR-26, CR-33, CR-34, and KW-040 covered by `tests/unit/combat/persistent_states_test.rs`.
- Test Evidence: `cargo test -p server --test persistent_states_test` passed 5/5. Requested adjacent regression command passed 19/19 across `objective_damage_gameover_test`, `range_targeting_test`, `substep4_dead_removal_test`, and `substep1_placement_test`. `cargo check -p server` passed. `git diff --check` passed.
- Verification: Current `main` includes integrated implementation commit `7ab44a1`; INJURED activates at the completed-sub-step boundary and persists into the next round, LEADER bonuses snapshot post-SS1/pre-SS2 after SS1 APPEARANCE effects and persist through SS5/SS6 even if the LEADER dies in SS4, and OUTNUMBERED recomputes only after the SS4 `ChainDeathBuffer` drain before SS5.
- Notes: No blocking GDD, ADR, Bevy 0.18, or keyword-state deviation found. Full ordered `S2CResolutionEvent` completeness remains COMBAT-011 scope. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates.
- Tech debt logged: None.
- Sprint status: Updated S5-19 in `production/sprint-status.yaml` to `done` with completion date 2026-05-05.
- Next recommended: Combat Story 11 ResolutionEvent Log Completeness (`production/epics/combat-resolution/story-011-resolution-event-log.md`) after readiness check.

## Session Extract - /story-done 2026-05-05
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/shop-auction-ui/story-001-plugin-scaffold-panel-tree-and-formulas.md` - Story 001: Plugin Scaffold, Panel Tree, and Formulas
- Criteria: 8/8 passing; plugin registration, stable bevy_ui panel roots, phase-resource usage, PresentationPlugin dependency, free-gold formula, bid labels, and auction border tiers covered by `tests/unit/shop_auction_ui/plugin_scaffold_formulas_test.rs`.
- Test Evidence: `cargo test -p client --test shop_auction_ui_plugin_scaffold_formulas_test` passed 8/8. `cargo fmt -p client -- --check`, `cargo check -p client`, `git diff --check 2dbc988^..2dbc988`, and `git diff --check` passed.
- Verification: Current `main` includes worker commit `733158c`, main integration commit `2dbc988`, and current `origin/main`/`HEAD` before closure commit `f527247`; `ShopAuctionUiPlugin` is registered fifth through `PresentationPlugin`, reads phase through `Res<CurrentClientPhase>`, creates six hidden bevy_ui panel roots for session lifecycle, and exposes pure formulas for free gold, bid button text, and auction border tiers.
- Notes: Advisory only - story manifest version is 2026-05-01 while current control manifest is 2026-05-05. No blocking GDD, ADR-019, ADR-021, or Bevy 0.18 deviation found. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates.
- Tech debt logged: None.
- Sprint status: Unchanged; no matching `SHOP-AUCTION-UI-001` / `SAU-001` row exists in `production/sprint-status.yaml`.
- Next recommended: Shop/Auction UI Story 002 (`production/epics/shop-auction-ui/story-002-draft-initial-grid-purchase-ready.md`) after readiness check, or Sprint 5 Combat Story 11 (`production/epics/combat-resolution/story-011-resolution-event-log.md`) if staying on the current sprint path.

## Session Extract - /story-done 2026-05-05
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/board-rendering/story-002-board-grid-camera-and-z-layers.md` - Story 002: Board Grid, Camera, and Z Layers
- Criteria: 7/7 passing; camera singleton/projection, 40 LaneCell sprite nodes, named Z constants, world-space sprite usage, and tint-based spawn highlight state covered by `tests/unit/board_rendering/board_grid_camera_test.rs`.
- Test Evidence: `cargo test -p client --test board_rendering_grid_camera_test` passed 6/6. `cargo test -p client --test board_rendering_plugin_scaffold_test` passed 9/9. `cargo fmt -p client -- --check` and `cargo check -p client` passed.
- Verification: Current `main` contains worker commit `6ae82d7` and main integration commit `ce3658b`; `BoardRenderingPlugin` spawns a fixed orthographic `Camera2d`, creates 40 `BoardCellNode` entities with `LaneCell` markers, uses named constants from `rendering_constants.rs`, and encodes spawn highlight state through `Sprite.color` tint instead of a separate Z layer.
- Notes: Advisory only - story manifest version is 2026-05-01 while current control manifest is 2026-05-05. Active TR-BR-003 also mentions BoardPosition replication; this remains outside the declared Story 002 grid/camera/Z-layer scope. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates.
- Tech debt logged: None.
- Sprint status: Unchanged; no matching BOARD-RENDERING-002 row exists in `production/sprint-status.yaml`.
- Next recommended: Board Rendering Story 003 (`production/epics/board-rendering/story-003-snapshot-spawn-units-objectives-and-hp-bars.md`) after readiness check, or Sprint 5 Combat Story 11 (`production/epics/combat-resolution/story-011-resolution-event-log.md`) if staying on the current sprint path.

## Session Extract - /story-done 2026-05-05
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/combat-resolution/story-011-resolution-event-log.md` - Story 011: ResolutionEvent Log Completeness
- Criteria: 5/5 passing; CR-30 and CR-32 are covered by `tests/integration/combat/resolution_event_log_test.rs`.
- Test Evidence: `cargo test -p server --test resolution_event_log_test` passed 3/3. Requested adjacent regression command passed 27/27 across `objective_damage_gameover_test`, `objective_resolution_sync_test`, `range_targeting_test`, `substep4_dead_removal_test`, and `substep6_combat_shield_counterattack_test`. `cargo check --workspace` passed. `git diff --check` passed.
- Verification: Current `main` includes worker commit `06d5b1744c39e3f6be97ffedb11c3dd99e489c12` and integrated commit `73ad695`; `S2CPlacementReveal` is atomic and precedes SS1 effects, `S2CResolutionEvent` carries the full typed CR-32 log in chronological order, objective events are consolidated into the single combat-owned batch, and `S2CPhaseChanged(DRAFT_SHOP)` is not observable before the resolution batch.
- Notes: Advisory only - story manifest version is 2026-05-01 while current control manifest is 2026-05-05; no conflicting current GDD, ADR-017, ADR-008, Bevy 0.18, or Lightyear 0.26 rule found. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates.
- Tech debt logged: None.
- Sprint status: Updated S5-20 in `production/sprint-status.yaml` to `done` with completion date 2026-05-05.
- Next recommended: Sprint 5 Must Have scope is complete; either run sprint close-out (`/smoke-check sprint`, `/team-qa sprint`, then `/gate-check`) or pick up S5-17 Hand UI Story 10 Submit Pre-Validation if the remaining Should Have item stays in scope.

## Session Extract - /story-done 2026-05-05
- Verdict: COMPLETE
- Story: `production/epics/lightyear-protocol-verification/story-005-placement-payload-shape-split.md` - Story 005: Placement Payload Shape Split
- Criteria: 5/5 passing; NP-58 / TR-NP-013 submit payload, reveal payload, type separation, split-payment invariant documentation, and reliable protocol registration verified against `shared/src/protocol.rs`.
- Test Evidence: `production/qa/evidence/placement-payload-shape-split-evidence.md`; fresh local verification passed `cargo test -p shared` (5/5), `cargo check -p shared`, `cargo check -p server`, `cargo check -p client`, and `git diff --check`.
- Verification: Current `main` includes worker commit `103d27d3260bc26a74536115a60a2e83c34ab1e5`, main integration commit `e65fcfe`, and additional integration fix `48b4b5f`; client submission code stages/sends `PlacedCardSubmit`, server call sites convert through internal `AcceptedPlacement`, and reveal messages expose `PlacedCardReveal` without mana-spend fields.
- Notes: No blocking GDD, ADR-003, ADR-008, ADR-002, ADR-007, Bevy 0.18, or Lightyear 0.26 deviation found. Story manifest version `2026-05-05` matches the current control manifest. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates.
- Tech debt logged: None.
- Sprint status: Unchanged; no matching NP-005/story row exists in `production/sprint-status.yaml`.
- Next recommended: HAND-UI-010 Submit Pre-Validation (`production/epics/hand-ui/story-010-submit-prevalidation.md`) is unblocked on the NP-005 protocol prerequisite, but still depends on ECO-007, BLS-011, and PRES-002 per the current Sprint 5 row.

## Session Extract - /story-done 2026-05-05
- Verdict: COMPLETE
- Story: `production/epics/economy-system/story-007-explicit-placement-mana-split-api.md` - Story 007: Explicit Placement Mana Split API
- Criteria: 6/6 passing; EC27 / TR-ECO-009 explicit split validation, pool-specific rejection, invalid-sum rejection, exact deduction, zero-cost split behavior, and existing auto-split regression coverage verified.
- Test Evidence: `cargo test -p server --test explicit_placement_mana_split_test` passed 6/6. `cargo test -p server economy::api::tests` passed for the matching economy API tests. Adjacent economy regressions passed 26/26 across `auction_reservation_test`, `economy_draft_subscriber_test`, `economy_interest_snapshot_test`, `economy_round_trace_test`, and `economy_network_dispatch_test`. `cargo check -p server` passed. `git diff --check` passed.
- Verification: Current `main` includes worker commit `b1c678f3400f4f7c8047adffd3f6d0b775bb5c29` and integration commit `dacc7d38bc300f89ac13b689e5abbb17f69a1fed`; `validate_explicit_mana_split` preserves submitted placement current/reserve allocation and rejects invalid requests without mutation, while `apply_explicit_mana_split` deducts the exact validated amounts without auto-split recomputation.
- Notes: No blocking GDD, ADR-019, ADR-002, Bevy 0.18, or scope deviation found. Story manifest version `2026-05-05` matches the current control manifest. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates because `production/review-mode.txt` is absent.
- Tech debt logged: None.
- Sprint status: Unchanged; no matching ECO-007/story row exists in `production/sprint-status.yaml`.
- Next recommended: Board/Lane Story 011 Placement Submit Authority Validation (`production/epics/board-lane-system/story-011-placement-submit-authority-validation.md`) after readiness check. HAND-UI-010 still depends on BLS-011 and PRES-002; PRES-002 story-done remains untouched per instruction.

## Session Extract - /story-done 2026-05-05
- Verdict: COMPLETE
- Story: `production/epics/presentation-layer/story-002-shared-economy-view.md` - Story 002: Shared Economy View
- Criteria: 6/6 passing; resource shape, single `S2CGoldUpdate` drain, reconnect snapshot seed, no local optimism, Hand UI consumption path, and single production drain grep guard verified.
- Test Evidence: `cargo test -p client --test shared_economy_view_test` passed 3/3. Affected HUD/Hand/PRES regression bundle passed: `hud_gold_mana_display_test` 6/6, `same_tick_tie_break_test` 3/3, `reconnect_snapshot_rebuild_test` 3/3, `hand_ui_reserve_mana_strip_test` 3/3, `hand_ui_draft_initial_grid_test` 5/5, and `hud_numeric_tween_animation_test` 4/4. `cargo check -p client`, `cargo fmt -p client -- --check`, and `git diff --check` passed.
- Verification: Current `main` includes worker commit `58afb3b2ba321bbea4bd5331a13bc228f952e6ed`, main integration commit `8587fa9`, and integration fix commit `e14feb6`; `PlayerEconomyView` is the shared presentation read model for own gold/current mana/reserve mana/mana cap, is updated only from authoritative `S2CGoldUpdate` or local reconnect snapshot data, and is consumed by Hand UI/HUD without independent `S2CGoldUpdate` drains.
- Notes: No blocking GDD, ADR-021, ADR-002, ADR-008, ADR-019, Bevy 0.18, or Lightyear 0.26 deviation found. Story manifest version `2026-05-05` matches the current control manifest. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates.
- Tech debt logged: None.
- Sprint status: Unchanged; no matching PRES-002 row exists in `production/sprint-status.yaml`.
- Next recommended: PRES-002 no longer blocks HAND-UI-010; remaining HAND-UI-010 prerequisite is BLS-011 (`production/epics/board-lane-system/story-011-placement-submit-authority-validation.md`) before readiness can be rechecked.

## Session Extract - /dev-story 2026-05-05
- Story: `production/epics/shop-auction-ui/story-002-draft-initial-grid-purchase-ready.md` - Story 002: Draft Initial Grid Purchase Ready
- Files changed: `client/Cargo.toml`, `client/src/presentation/mod.rs`, `client/src/ui/shop_auction/mod.rs`, `tests/integration/shop_auction_ui/draft_initial_grid_test.rs`, `tests/unit/shop_auction_ui/plugin_scaffold_formulas_test.rs`
- Test written: `tests/integration/shop_auction_ui/draft_initial_grid_test.rs`
- Blockers: None
- Next: `/code-review client/src/ui/shop_auction/mod.rs client/src/presentation/mod.rs tests/integration/shop_auction_ui/draft_initial_grid_test.rs tests/unit/shop_auction_ui/plugin_scaffold_formulas_test.rs` then `/story-done production/epics/shop-auction-ui/story-002-draft-initial-grid-purchase-ready.md`

## Session Extract - /story-done 2026-05-05
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/board-rendering/story-004-ghost-preview-hand-ui-bridge.md` - Story 004: Ghost Preview and Hand UI Bridge
- Criteria: 14/14 passing; ghost message drain, BoardCell lifecycle, replacement, sprite constraints, variant matrix, clear/no-op behavior, reveal cleanup, reverse ghost interaction messages, no spawn-highlight ownership, and no submit-message ownership verified.
- Test Evidence: `cargo test -p client --test board_rendering_ghost_preview_bridge_test` passed 4/4. Adjacent Hand UI tests passed: `hand_ui_placement_drag_highlights_test`, `hand_ui_placement_submit_core_test`, `hand_ui_placement_timer_test`, and `hand_ui_placement_unstaging_test`. `cargo fmt -p client -- --check`, `cargo check -p client`, and `git diff --check` passed.
- Verification: Current `main` includes worker commit `1e87793ddcc6fffdcc8ac284b18e842735911d2a` via integration commit `699ac16`; Board Rendering owns only presentation ghost previews and reverse ghost interaction messages, while Hand UI remains staging/submit owner.
- Notes: Advisory only - manual screenshot/UI evidence deferred until interactive UI evidence exists. Known unrelated `hand_ui_placement_instant_staging_test` stale `pending[0].owner_id` compile issue is not a BOARD-004 blocker.
- Tech debt logged: None.
- Sprint status: Unchanged; no matching BOARD-004 row exists in `production/sprint-status.yaml`.
- Next recommended: Board Rendering Story 005 (`production/epics/board-rendering/story-005-placement-reveal-animation-handoff.md`) after readiness check.

## Session Extract - /story-done 2026-05-05
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/shop-auction-ui/story-002-draft-initial-grid-purchase-ready.md` - Story 002: Draft Initial Grid Purchase Ready
- Criteria: 9/9 passing; activation buffering, sort order, valid purchase sends, insufficient-gold lockout, hand-full lockout, purchase confirmation overlay, Ready/Retract Ready signaling, ready-state purchase interactivity, and PLACEMENT dismissal were verified.
- Test Evidence: `cargo test -p client --test shop_auction_ui_draft_initial_grid_test` passed 9/9. `cargo test -p client --test shop_auction_ui_plugin_scaffold_formulas_test` passed 8/8. `cargo check -p client`, `cargo fmt -p client -- --check`, and `git diff --check` passed.
- Verification: Current `main` includes worker commit `b52510adba1f60fb2c740c0636e4ebaed6c5b862` via main integration commit `21c09f5`; DRAFT_INITIAL reads shared phase/economy state, owns only `S2CDraftOffering` and `S2CCardAcquired` drains, sends purchase/ready intents over `ReliableChannel`, and uses Bevy UI Required Components without forbidden bundle patterns.
- Notes: Advisory only - manual visual evidence for overlay/tooltip remains deferred to Story 009 or a separately scoped tooltip/visual evidence story. Story manifest version `2026-05-05` matches the current control manifest. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates because `production/review-mode.txt` is absent.
- Tech debt logged: None.
- Sprint status: Unchanged; no matching SAU-002/story row exists in `production/sprint-status.yaml`.
- Next recommended: Shop/Auction UI Story 003 (`production/epics/shop-auction-ui/story-003-draft-shop-refresh-and-purchase.md`) after readiness check, or continue the current sprint close-out sequence if no additional Should Have scope is being pulled in.

## Session Extract - /story-done 2026-05-05
- Verdict: COMPLETE
- Story: `production/epics/board-lane-system/story-011-placement-submit-authority-validation.md` - Story 011: Placement Submit Authority Validation
- Criteria: 8/8 passing; sender authority/phase gate, hand ownership, duplicate card rejection, target legality, spawn/occupancy legality, explicit mana validation, accepted pending write, and close-phase deduction ordering verified.
- Test Evidence: `cargo test -p server --test placement_submit_authority_validation_test` passed 8/8. Requested adjacent regression commands passed: `placement_buffer_test` 3/3 and `explicit_placement_mana_split_test` 6/6. `cargo fmt -p server -- --check`, `cargo check -p server`, `cargo check --workspace`, and `git diff --check` passed.
- Verification: Current `main` includes worker commit `d2d16312db93205d81613387e3aade18cbebd732` via main integration commit `7f034b3`; sender identity resolves from `PlayerConnectionMap`, invalid submissions are silently discarded all-or-nothing before `PendingPlacements` writes, accepted submissions preserve explicit current/reserve spend, and `close_placement_phase` deducts explicit mana before reveal enqueue and unit spawn.
- Notes: No blocking GDD, ADR-007, ADR-019, ADR-002, ADR-008, Bevy 0.18, or Lightyear 0.26 deviation found. Story manifest version `2026-05-05` matches the current control manifest. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates.
- Tech debt logged: None.
- Sprint status: Unchanged; no matching BLS-011 row exists in `production/sprint-status.yaml`.
- Next recommended: HAND-UI-010 can be rechecked now that BLS-011 is complete.

## Session Extract - /story-done 2026-05-05
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/board-rendering/story-003-snapshot-spawn-units-objectives-and-hp-bars.md` - Story 003: Snapshot Spawn, Units, Objectives, and HP Bars
- Criteria: 7/7 passing; snapshot rebuild, visible unit components, HP fill thresholds and epsilon handling, HP child local Z, standing objective identity isolation, missing-card placeholder fallback, and stale pending visual state cleanup verified.
- Test Evidence: `cargo test -p client --test board_rendering_snapshot_spawn_test` passed 4/4. Requested adjacent commands passed: `board_rendering_grid_camera_test` 6/6, `board_rendering_plugin_scaffold_test` 9/9, `presentation_plugin_scaffold_test` 3/3, and `reconnect_snapshot_rebuild_test` 3/3. `cargo fmt -p client -- --check`, `cargo check -p client`, and `git diff --check` passed.
- Verification: Current `main` includes worker commit `fd6f6fd121dd32a945fbfcd276c3921fb159f7e7` via main integration commit `c0dc500ea675342fd0b17dedf7a85039f16bc9f7`; `S2CGameSnapshot` rebuild consumes the local snapshot message, despawns stale `BoardSnapshotEntity` nodes, spawns atlas-backed unit/objective sprites with HP bar children, clears `AnimQueue`, `PendingPhaseChange`, `PendingObjectiveDestroyedEvents`, `StagedObjectiveRevealQueue`, and `ObjectiveIdentityCache`, and pauses stale `TweenAnim` components.
- Notes: Advisory only - current client code has no concrete `PendingResolutionScript` resource; the implemented pending visual/script resources are cleared and covered by `board_rendering_snapshot_spawn_test`. Story manifest version `2026-05-05` matches the current control manifest. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates because `production/review-mode.txt` is absent.
- Tech debt logged: None.
- Sprint status: Unchanged; no matching BOARD-003/story row exists in `production/sprint-status.yaml`.
- Next recommended: Board Rendering Story 005 (`production/epics/board-rendering/story-005-placement-reveal-animation-handoff.md`) after readiness check.

## Session Extract - /story-done 2026-05-05
- Verdict: COMPLETE
- Story: `production/epics/hand-ui/story-010-submit-prevalidation.md` - Story 010: Submit Pre-Validation -- Mana & Reserve Checks
- Criteria: 3/3 passing; HU-17b reserve overdraw, TR-HU-008 current mana overdraw, and HU-17c correction success verified.
- Test Evidence: `cargo test -p client --test hand_ui_submit_prevalidation_test` passed 8/8. Adjacent client regression command passed 15/15 across `hand_ui_placement_timer_test`, `hand_ui_placement_submit_core_test`, `hand_ui_placement_instant_staging_test`, and `hand_ui_reserve_mana_strip_test`. `cargo test -p server --test placement_submit_authority_validation_test` passed 8/8. `cargo fmt -p client -- --check`, `cargo check -p client`, and `git diff --check` passed.
- Verification: Current `main` includes worker branch `work/hand-ui-010-submit-prevalidation` commit `ccecb7c17157e0ace1c34ba93438c46dd8371ff6` through main integration commit `8ee2860`; submit pre-validation reads `PlayerEconomyView`, blocks overdrawn reserve/current spends before `C2SSubmitPlacement`, keeps Submit active on failure, attaches and clears `SubmitValidationError`, and follows the existing successful-send inactive path after correction.
- Notes: No blocking GDD, ADR-021, ADR-002, Bevy 0.18, or Lightyear 0.26 deviation found. Broader non-economy Rule 10 checks remain server-authoritative under BLS-011 and out of HAND-UI-010 scope. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates.
- Tech debt logged: None.
- Sprint status: Updated S5-17 in `production/sprint-status.yaml` to `done` with completion date 2026-05-05.
- Next recommended: Sprint 5 Must Have and Should Have tracked story rows are complete; run sprint close-out (`/smoke-check sprint`, `/team-qa sprint`, then `/gate-check`) if no more pull-forward work is being added.

## Session Extract - /story-done 2026-05-05
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/shop-auction-ui/story-003-shop-panel-slots-refresh-purchase-ready.md` - Story 003: Shop Panel Slots Refresh Purchase Ready
- Criteria: 9/9 passing; DRAFT_SHOP phase+slot activation, three-slot rendering, valid multi-purchase sends, same-frame refresh disable, confirmed-only refresh count changes, hand-full slot lockout with refresh affordance, Ready/Retract Ready interactivity, PLACEMENT dismissal/late confirmation handling, and DRAFT_AUCTION slot buffering verified.
- Test Evidence: `cargo test -p client --test shop_auction_ui_shop_panel_test` passed 8/8. Requested adjacent regressions passed: `shop_auction_ui_plugin_scaffold_formulas_test` 8/8 and `shop_auction_ui_draft_initial_grid_test` 9/9. `cargo fmt -p client -- --check`, `cargo check -p client`, and `git diff --check` passed.
- Verification: Current `main` includes worker branch `work/sau-003-shop-panel-slots-refresh-purchase-ready` commit `2be9e0af39dbd03fb1782aaf4d95cf4c74646feb` through main integration commit `27e077a`; DRAFT_SHOP reads shared phase/economy state, owns shop slot/acquisition handling, sends purchase/refresh/ready intents over `ReliableChannel`, and buffers auction-phase `S2CShopSlots` until the shop phase.
- Notes: Advisory only - manual visual evidence remains deferred to Story 009. No blocking GDD, ADR-015, ADR-021, Bevy 0.18, or Lightyear 0.26 deviation found. Story manifest version `2026-05-05` matches the current control manifest. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates.
- Tech debt logged: None.
- Sprint status: Unchanged; no matching SAU-003/story row exists in `production/sprint-status.yaml`.
- Next recommended: Shop/Auction UI Story 004 (`production/epics/shop-auction-ui/story-004-auction-locked-footer-and-card-buffer.md`) after readiness check, or continue sprint close-out if no more pull-forward work is being added.

## Session Extract - /story-done 2026-05-05
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/board-rendering/story-005-placement-reveal-collect-and-tween.md` - Story 005: Placement Reveal Collect and Tween
- Criteria: 7/7 passing; one-frame opponent-only reveal collection, one sorted `PlacementRevealAnimReady` batch, same-pass Card Animations reveal start, `unit_reveal_tween_duration_ms` use, `ResolutionReveal stuck` recovery, and `PendingResolutionScript stuck` recovery verified.
- Test Evidence: `cargo test -p client --test board_rendering_placement_reveal_test` passed 3/3; `board_rendering_snapshot_spawn_test` passed 4/4; `board_rendering_ghost_preview_bridge_test` passed 4/4; `card_animations_placement_reveal_test` passed 9/9; `cargo test -p shared` passed 5/5. `cargo fmt -p client -- --check`, `cargo fmt -p shared -- --check`, `cargo check -p client`, and `git diff --check` passed.
- Verification: Worker branch `work/board-rendering-005-placement-reveal-collect-tween` commit `a7d792e092d380ec15c7cabcac7effec0c52839a` integrated. `C2SRequestSnapshot` is a narrow empty payload C2S reliable message, registered in `shared/src/protocol.rs`, exposed through the client Lightyear sender surface, and used only by Board Rendering reveal/stuck recovery.
- Notes: Advisory only - manual Visual/Feel sign-off file `production/qa/evidence/board-rendering-placement-reveal-evidence.md` is not present yet. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates because `production/review-mode.txt` is absent. No reconnect rebuild, full resolution playback, Shop/Auction UI, Hand UI, server placement authority, design, asset, or sprint-status scope creep found.
- Tech debt logged: None.
- Sprint status: Unchanged; no matching BOARD-005 row exists in `production/sprint-status.yaml`.
- Next recommended: Board Rendering Story 006 (`production/epics/board-rendering/story-006-resolution-anim-queue-and-phase-buffering.md`) after its blocker is rechecked against the now-complete Combat Resolution event log work.

## Session Extract - /story-done 2026-05-05
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/board-rendering/story-010-performance-evidence-and-ci-guards.md` - Story 010: Performance Evidence and CI Guards
- Criteria: 5/8 passing, 3/8 deferred with accepted blocker notes for the narrowed baseline scope. Z-literal CI guard, single phase-drain guard, 20-unit native ECS baseline fixture, two-atlased-image handle evidence, approved standalone/non-counted batch notes, and BOARD-009 status-icon atlas deferral verified.
- Test Evidence: `production/qa/evidence/board-rendering-performance-evidence.md` created. `cargo test -p client --test board_rendering_grid_camera_test --test board_rendering_plugin_scaffold_test --test board_rendering_snapshot_spawn_test` passed 20/20. `cargo fmt -p client -- --check`, `cargo check -p client`, and `git diff --check` passed.
- Verification: Worker branch `work/board-rendering-010-baseline-ci-perf-guards` commit `f51a3f7f92634e33ae4e96fac787fefb35e990f9` integrated. CI now runs focused Board Rendering source guards for named Z constants and single `MessageReceiver<S2CPhaseChanged>` drain ownership.
- Notes: Advisory/deferred only - browser/WASM 1920x1080 frame-time screenshot capture still needs a harness that can seed the 20-unit baseline fixture and record timing; BOARD-009 status-icon atlas evidence remains deferred and is not claimed by this narrowed baseline closure. Story manifest version `2026-05-05` matches the current control manifest. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates because `production/review-mode.txt` is absent.
- Tech debt logged: None.
- Sprint status: Unchanged; no matching BOARD-010 row exists in `production/sprint-status.yaml`.
- Next recommended: Board Rendering final epic closure should wait for the browser/WASM capture harness and BOARD-009 status-icon atlas evidence, or continue sprint close-out if no more pull-forward work is being added.

## Session Extract - /story-done 2026-05-05
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/shop-auction-ui/story-004-auction-panel-activation-and-preparing-state.md` - Story 004: Auction Panel Activation and Preparing State
- Criteria: 7/7 passing; card-first preparing, phase-first inactive buffer, both-order activation with countdown, 10-second preparing timeout, non-auction dismiss/clear, locked DRAFT_AUCTION footer, hidden refresh, and no purchase/refresh sends during auction verified.
- Test Evidence: `cargo test -p client --test shop_auction_ui_auction_activation_test` passed 6/6. Requested regressions passed: `shop_auction_ui_plugin_scaffold_formulas_test` 8/8, `shop_auction_ui_draft_initial_grid_test` 9/9, `shop_auction_ui_shop_panel_test` 8/8, and `presentation_plugin_scaffold_test` 3/3. `cargo fmt -p client -- --check`, `cargo check -p client`, and `git diff --check` passed on final `main`.
- Verification: Worker branch `work/sau-004-auction-panel-activation-preparing` commit `e63cfc4c6f86349d6760ed42e416a13fcb1096b7` is integrated through main implementation commit `c671acac51357e40460869a1b2daebbe09a0e2c1`. Local `main` is based on the newer `AGENTS.md` footer commit, so no SAU-004 closure commit includes `AGENTS.md`.
- Notes: Advisory only - manual visual evidence remains deferred to Story 009 / `production/qa/evidence/shop-auction-ui-auction-preparing-evidence.md`. No blocking GDD, ADR-013, ADR-021, Bevy 0.18, or Lightyear 0.26 deviation found. Story manifest version `2026-05-05` matches the current control manifest. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates because `production/review-mode.txt` is absent.
- Tech debt logged: None.
- Sprint status: Unchanged; no matching SAU-004/story row exists in `production/sprint-status.yaml`.
- Next recommended: Shop/Auction UI Story 005 (`production/epics/shop-auction-ui/story-005-auction-bid-buttons-affordability-and-inflight.md`) after `/story-readiness`, or continue sprint close-out if no more pull-forward work is being added.

## Session Extract - /story-done 2026-05-05
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/board-rendering/story-009-status-icons-cooccupancy-and-spawn-range.md` - Story 009: Status Icons and Co-Occupancy Visuals
- Criteria: 10/10 passing for narrowed status-icon and co-occupancy visuals; status icon count, overflow badge, tier/duration/key ordering, local stack transforms, `ChildOf` inheritance, board-elements atlas use, per-unit OUTNUMBERED key rendering, F3 two-unit offsets, and F3 index-2 assert verified.
- Test Evidence: `cargo test -p client --test board_rendering_status_icons_test` passed 5/5. Requested adjacent regressions passed: `board_rendering_snapshot_spawn_test` 5/5, `board_rendering_grid_camera_test` 6/6, `board_rendering_plugin_scaffold_test` 9/9, and `board_rendering_placement_reveal_test` 3/3. `cargo fmt -p client -- --check`, `cargo check -p client`, and `git diff --check` passed.
- Verification: Worker commit `9693bab086e946b6908fde7c2ee537dfa19eba91` was integrated onto current `main` as implementation commit `ea8783d38c1a9f1aa5133b3d11607bffaa3f6ad7`; closure notes were added after verification.
- Notes: No blocking GDD, ADR-018, ADR-021, Bevy 0.18, or Lightyear 0.26 deviation found for the narrowed renderer scope. Final visual/browser evidence remains open, and spawn range source/replication/highlight closure remains out of scope and not closed. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates because `production/review-mode.txt` is absent.
- Tech debt logged: None.
- Sprint status: Unchanged; no matching BOARD-009 row exists in `production/sprint-status.yaml`.
- Next recommended: Final Board Rendering visual/evidence closure should wait until spawn range source/replication is resolved and status-icon atlas/browser evidence is captured.

## Session Extract - /story-done 2026-05-05
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/shop-auction-ui/story-005-auction-bid-buttons-affordability-and-inflight.md` - Story 005: Auction Bid Buttons, Affordability, and In-Flight
- Criteria: 8/8 passing; total-commitment bid labels, local-free-gold affordability, hand-full lockout, leading badge, exact one-send click behavior, in-flight visual lockout, duplicate-send suppression, and locally expired timer messaging verified.
- Test Evidence: `cargo test -p client --test shop_auction_ui_auction_bid_buttons_test` passed 5/5. Requested regressions passed: `shop_auction_ui_auction_activation_test` 6/6, `shop_auction_ui_shop_panel_test` 8/8, `shop_auction_ui_draft_initial_grid_test` 9/9, and `presentation_plugin_scaffold_test` 3/3. Additional changed-file check `shop_auction_ui_plugin_scaffold_formulas_test` passed 8/8. `cargo fmt -p client -- --check`, `cargo check -p client`, and `git diff --check` passed.
- Verification: Worker branch `work/sau-005-auction-bid-buttons-affordability-inflight` commit `b118dcabcb755e606eb212b55010baf05e8228eb` was applied onto current `main` after Sprint 5 smoke gate commit `38f613a`. Integration fixes updated the scaffold entity-count test and moved bid-status text below the button row to avoid overlap.
- Notes: Advisory only - manual visual/accessibility evidence remains deferred to Story 009 / `production/qa/evidence/shop-auction-ui-bid-buttons-evidence.md`. No blocking GDD, ADR-013, ADR-019, ADR-021, Bevy 0.18, or Lightyear 0.26 deviation found. Story manifest version `2026-05-05` matches the current control manifest. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates because `production/review-mode.txt` is absent.
- Tech debt logged: None.
- Sprint status: Unchanged; no matching SAU-005/story row exists in `production/sprint-status.yaml`.
- Next recommended: Shop/Auction UI Story 006 (`production/epics/shop-auction-ui/story-006-auction-accepted-rejected-feedback.md`) after readiness check, or continue sprint close-out if no more pull-forward work is being added.

## Session Extract - /story-done 2026-05-05
- Verdict: COMPLETE
- Story: `production/epics/lightyear-protocol-verification/story-006-spawn-range-live-update-contract.md` - Story 006: Spawn Range Live Update Contract
- Criteria: 5/5 passing; `ResolutionEvent::SpawnRangeChanged` schema, reliable `S2CResolutionEvent` transport, post-`ObjectiveDestroyed` ordering, snapshot recovery role, and no replicated `SpawnRange` component verified.
- Test Evidence: `cargo fmt -p shared -- --check`, `cargo test -p shared`, `cargo test -p shared --test spawn_range_live_update_contract`, `cargo check -p shared`, `cargo check -p server`, `cargo check -p client`, and `git diff --check` passed in the NP-006 worker worktree.
- Verification: Worker branch `work/np-006-spawn-range-live-update-contract` commit `02267ebfe65ab31f943b8b209dd51236970c2068` was fast-forwarded onto `main`; the commit touches only `shared/src/protocol.rs`, `shared/Cargo.toml`, and `tests/unit/protocol/spawn_range_live_update_contract_test.rs`.
- Notes: No blocking GDD, ADR-003, ADR-008, ADR-011, ADR-020, Bevy 0.18, or Lightyear 0.26 deviation found. BLS-012, BR-011, and spawn highlight visuals remain out of scope and were not implemented. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates.
- Tech debt logged: None.
- Sprint status: Unchanged; no matching NP-006/story row exists in `production/sprint-status.yaml`.
- Next recommended: Board/Lane Story 012 (`production/epics/board-lane-system/story-012-spawn-range-authoritative-projection.md`) after readiness check; do not implement it as part of NP-006 closure.

## Session Extract - /story-done 2026-05-05
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/shop-auction-ui/story-006-auction-accepted-rejected-feedback.md` - Story 006: Auction Accepted/Rejected Feedback
- Criteria: 9/9 passing; local accepted bid leader/button/timer target, opponent accepted bid two-message local-gold gate in both arrival orders, opponent gold non-satisfaction, rejection affordability reset, exact rejection toast mapping, toast replacement/timer behavior, and phase-exit settlement guard cleanup verified.
- Test Evidence: `cargo test -p client --test shop_auction_ui_auction_feedback_test` passed 6/6. Requested regressions passed: `shop_auction_ui_auction_bid_buttons_test` 5/5, `shop_auction_ui_auction_activation_test` 6/6, and `shop_auction_ui_shop_panel_test` 8/8. `cargo fmt -p client -- --check`, `cargo check -p client`, and `git diff --check` passed after freeing generated Cargo artifacts from `target\msvc-local`.
- Verification: Worker branch `work/sau-006-auction-accepted-rejected-feedback` commit `abbbe0f1498d7a949e19e2377244b40e83ad5c91` was integrated. Shop/Auction UI handles `S2CAuctionBidAccepted` and `S2CAuctionBidRejected`, consumes HUD's local `S2CGoldBroadcast` bridge for the re-enable gate, writes `AuctionTimerTargetFill`, uses pre-pooled toast entities, and keeps late rejected messages from reviving controls after phase exit.
- Notes: Advisory only - toast visual polish remains Story 009 scope. No blocking GDD, ADR-013, ADR-019, ADR-021, Bevy 0.18, or Lightyear 0.26 deviation found. Story manifest version `2026-05-05` matches the current control manifest. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates because `production/review-mode.txt` is absent.
- Tech debt logged: None.
- Sprint status: Unchanged; no matching SAU-006/story row exists in `production/sprint-status.yaml`.
- Next recommended: Shop/Auction UI Story 007 (`production/epics/shop-auction-ui/story-007-auction-settlement-and-shop-transition.md`) after readiness check; do not implement it as part of SAU-006 closure.

## Session Extract - /story-done 2026-05-05
- Verdict: COMPLETE
- Story: `production/epics/board-lane-system/story-012-spawn-range-authoritative-projection.md` - Story 012: Spawn Range Authoritative Projection
- Criteria: 6/6 passing; `SpawnRangeState` is the live Board/Lane source for placement validation, fake objective facts expand and clamp the projection, snapshots read `SpawnRangeState` instead of `ObjectiveCounters`, `SpawnRangeChanged` is emitted for live changes, event ordering follows `ObjectiveDestroyed`, and no replicated `SpawnRange` component was introduced.
- Test Evidence: `cargo test -p server --test spawn_range_authoritative_projection_test --test resolution_event_log_test --test spawn_range_validation_test --test displacement_keywords_test --test placement_submit_authority_validation_test --test objective_damage_gameover_test --test consequence_path_test --test fake_reward_test --test snapshot_secret_strip_test --test reconnect_snapshot_test` passed. `cargo test -p shared --test spawn_range_live_update_contract`, `cargo fmt`, `cargo check -p server`, and `git diff --check` passed.
- Verification: Worker branch `work/bls-012-spawn-range-authoritative-projection` commit `cfc44ac888297782c439be9963d877fa4497dae1` was integrated onto `main` as implementation commit `4418aae`. Combat now traces ordered `ResolutionEvent::SpawnRangeChanged` after fake `ObjectiveDestroyed`, and snapshot assembly uses the Board/Lane `SpawnRangeState` projection.
- Notes: No blocking GDD, ADR-008, ADR-011, ADR-020, Bevy 0.18, or Lightyear 0.26 deviation found. Story manifest version `2026-05-05` matches the current control manifest. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates because `production/review-mode.txt` is absent. BR-011 spawn range highlights, final board visual/browser evidence, status icon/co-occupancy evidence, and design/assets work were not implemented or closed.
- Tech debt logged: None.
- Sprint status: `production/sprint-status.yaml` matching BLS-012 row set to `done` with `completed: "2026-05-05"`.
- Next recommended: Keep BR-011 (`production/epics/board-rendering/story-011-spawn-range-highlights.md`) as a separate readiness/implementation decision; do not treat BLS-012 closure as BR-011 closure.

## Session Extract - /story-done 2026-05-05
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/board-rendering/story-011-spawn-range-highlights.md` - Story 011: Spawn Range Highlights
- Criteria: 6/6 passing; snapshot `PlayerSnapshot.spawn_range_cells` rebuild seeding, ordered live `ResolutionEvent::SpawnRangeChanged` consumption, DRAFT/PLACEMENT persistence, objective-event separation, OBJMISS behavior, and no replicated `SpawnRange` component verified.
- Test Evidence: `cargo test -p client --test board_rendering_spawn_range_highlights_test --test board_rendering_grid_camera_test --test board_rendering_plugin_scaffold_test --test board_rendering_snapshot_spawn_test --test board_rendering_status_icons_test --test board_rendering_placement_reveal_test --test board_rendering_ghost_preview_bridge_test` passed 36/36. `cargo fmt -p client -- --check`, `cargo check -p client`, and `git diff --check` passed after closure updates.
- Verification: Worker branch `work/board-rendering-011-spawn-range-highlights` commit `ec7d0abc46e0d1a840b7347400cb9d6487bc2cf0` was fast-forwarded onto `main`. Board Rendering now owns persistent board-cell spawn highlight state separately from Hand UI drag-time ghost/highlight behavior.
- Notes: No blocking GDD, ADR-011, ADR-020, ADR-021, Bevy 0.18, or Lightyear transport deviation found. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates because `production/review-mode.txt` is absent. BOARD-012 browser/WASM perf evidence, final Board Rendering visual evidence, traps, final VFX, and server spawn range authority remain separate and were not implemented or closed.
- Tech debt logged: None.
- Sprint status: `production/sprint-status.yaml` matching BR-011 row set to `done` with `completed: "2026-05-05"`.
- Next recommended: Keep BOARD-012 browser/WASM perf evidence and final Board Rendering visual/evidence closure separate from BR-011.

## Session Extract - /story-done 2026-05-05
- Verdict: COMPLETE
- Story: `production/epics/objective-system/story-008-os18b-two-client-objective-hp-visibility.md` - Story 008: OS-18b Two-Client Objective HP Visibility Evidence
- Criteria: 7/7 passing; final authoritative server `ObjectiveHp` saturated to 0 after same-sub-step double damage, consequence path queued exactly once, RESOLUTION-end destruction emitted exactly once, both clients observed `[0]`, intermediate HP 1 was not observed, duplicate final HP was not observed, and the evidence file records the command plus client observation sequences.
- Test Evidence: `cargo test -p server --test os18b_two_client_objective_hp_visibility_test -- --nocapture` passed 1/1. Required evidence exists at `production/qa/evidence/os-18b-two-client-objective-hp-visibility-2026-05-05.md`.
- Verification: Worker branch `work/objective-system-008-os18b-two-client-objective-hp-visibility` commit `3394127e2f7badecdc491d4b6be8cfcca61b3e4c` was cherry-picked onto current `main` as integration commit `f097606`. Requested regressions passed: `objective_resolution_sync_test` 5/5, `objective_identity_unicast_test` 4/4, `damage_interface_test` 7/7, `consequence_path_test` 7/7, `e2e_websocket_test` 1/1, and `cargo check -p server`.
- Notes: No blocking GDD, ADR-001, ADR-010, Bevy 0.18, or Lightyear 0.26 deviation found. The harness exposed and fixed a public ObjectiveHp replication bug by changing objective initialization to `Replicate::to_clients(NetworkTarget::All)`. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates because `production/review-mode.txt` is absent.
- QA condition: `QA-COND-0003` closed in `production/qa/bugs/QA-COND-0003-os-18b-two-client-objective-hp-visibility.md`.
- Tech debt logged: None.
- Sprint status: `production/sprint-status.yaml` matching S6-05 row set to `done` with `completed: "2026-05-05"`.
- Post-gate note: Superseded by the 2026-05-06 S6-06 gate pass; OS-18b evidence contributed to S6-05 closure.

## Session Extract - /story-done 2026-05-05
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/game-session-system/story-008-placement-timer-multiplier-authority.md` - Story 008: PLACEMENT Timer Multiplier Authority
- Criteria: 24/24 verified; protocol values are extension-only, reliable C2S/S2C settings messages are registered, no requester identity is exposed, GSS computes and freezes highest-request-wins before `SessionReady`, RSM applies the frozen multiplier to standard and auction-followup PLACEMENT timers, `S2CPhaseChanged.timer_duration_ms` carries the effective duration, reconnect snapshot carries the frozen neutral value, and Hand UI uses server-provided phase duration.
- Test Evidence: `cargo fmt -p shared -- --check`, `cargo fmt -p server -- --check`, `cargo fmt -p client -- --check`, `cargo check -p shared`, `cargo check -p server`, `cargo check -p client`, `cargo test -p shared`, the requested server tests, the requested client tests, and `git diff --check` all passed.
- Verification: Worker branch `work/game-session-system-008-placement-timer-multiplier-authority` commit `d31b98d60b0921f01017b4427b9193c5e7383ed8` was cherry-picked onto current `main` as integration commit `4b505af`.
- Notes: No blocking GDD, ADR-002, ADR-009, ADR-012, ADR-021, ADR-023, Bevy 0.18, or Lightyear 0.26 deviation found. Equivalent Hand UI server-duration coverage lives in `tests/integration/hand-ui/placement_timer_test.rs::hu_24_placement_timer_uses_server_phase_duration`; no separate `hand_ui_server_timer_duration_test.rs` was created. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates because `production/review-mode.txt` is absent.
- QA condition: `QA-COND-0005` remains Open. Only the PLACEMENT timer-extension sub-gap is recorded as implemented/verified in `production/qa/evidence/gss-008-placement-timer-multiplier-authority-2026-05-05.md`.
- Tech debt logged: None.
- Sprint status: Unchanged; S6-04 tracks the broader QA-COND-0005 remediation condition, so this sub-gap verification does not legitimately close that sprint row.
- Post-gate note: QA-COND-0005 is friend-game-only accepted risk for the Sprint 6 gate and remains future accessibility debt before any public/external release.

## Session Extract - /story-done 2026-05-05
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/board-rendering/story-012-browser-wasm-board-performance-evidence.md` - Story 012: Browser/WASM Board Performance Evidence
- Criteria: 10/10 satisfied by current browser/WASM evidence; harness seed, fixture counts, 1920x1080 capture, nonblank 5-lane board, browser RAF total-frame timing, ADR-021 steady-state presentation timing, corrected snapshot rebuild timing, evidence update, scope guard, and failure handling are covered.
- Test Evidence: `production/qa/evidence/board-rendering-performance-evidence.md`; `production/qa/evidence/captures/board-rendering-baseline-1920x1080.png`; `production/qa/evidence/captures/board-rendering-baseline-timing.json`. The trace reports browser RAF max 6.0 ms <=16.67 ms, steady-state presentation max 0.2 ms <1 ms, seeded full snapshot rebuild 3.3 ms <=16.67 ms, and `board012BudgetPass=true`.
- QA condition: `QA-COND-0004` is Closed / N/A - Closed in both the bug file and taxonomy.
- Notes: Evidence correction commit `7d64cd766d02b0c2888124d7e23debf29fd53d16` corrected the timing classification. ADR-021's <3 ms phase-boundary presentation spike budget remains documented for true hide/show/cancel-tween phase work and is not claimed as newly sampled by BOARD-012. No board rendering behavior, `design/assets/**`, `AGENTS.md`, `production/session-state/codex-orchestrator-state.md`, or unrelated QA-COND files were touched. No full Board Rendering epic closure is claimed.
- Tech debt logged: None.
- Sprint status: `production/sprint-status.yaml` matching S6-03 row set to `done` with `completed: "2026-05-05"`.
- Post-gate note: Superseded by the 2026-05-06 S6-06 gate pass; BOARD-012 closed QA-COND-0004 for the Sprint 6 gate.

## Session Extract - /story-done 2026-05-06
- Verdict: COMPLETE
- Story: `production/epics/hud/story-011-current-reserve-mana-shapes.md` - Story 011: Current and Reserve Mana Shape Distinction
- Criteria: 11/11 passing; A11Y-ST-13 current/reserve mana shape distinction, current/cap text, reserve text/visibility, non-color component geometry, pre-pooled entity stability, focused/regression tests, browser/WASM evidence, two viewport captures, QA-COND-0005 impact statement, and whitespace gate verified.
- Test Evidence: `production/qa/evidence/hud-011-mana-shapes-evidence.md`; capture artifacts under `production/qa/evidence/captures/hud-011-mana-shapes/`; `cargo test -p client --test hud_mana_shape_distinction_test` passed 3/3; `cargo test -p client --test hud_gold_mana_display_test` passed 6/6; `cargo fmt -p client -- --check`, `cargo check -p client`, and `git diff --check` passed.
- Verification: Current `main` is aligned with `origin/main` and includes the integrated HUD-011 implementation. The HUD exposes current mana as bar geometry and reserve mana as diamond geometry through Bevy UI component/layout state, hides the reserve diamond and label together at zero reserve, and preserves existing economy display behavior.
- Notes: No blocking GDD, ADR-002, ADR-021, Bevy 0.18, or control-manifest deviation found. Story manifest version `2026-05-05` matches the current control manifest. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates because `production/review-mode.txt` is absent. No new code was implemented during closure.
- QA condition: `QA-COND-0005` remains Open. HUD-011 closes only the A11Y-ST-13 sub-gap in the Standard-tier accessibility disposition register.
- Tech debt logged: None.
- Sprint status: `production/sprint-status.yaml` updated to record A11Y-ST-13 evidence under S6-04 while leaving S6-04/QA-COND-0005 open.
- Post-gate note: Remaining Standard-tier accessibility rows are Polish/public-release debt under the QA-COND-0005 friend-game-only accepted-risk disposition.

## Session Extract - /story-done 2026-05-06
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/accessibility-settings/story-001-settings-accessibility-foundation-and-preferences.md` - Story 001: Settings / Accessibility Foundation and Preferences
- Criteria: 24/24 passing; Settings / Accessibility plugin registration, preference defaults and validation, versioned storage abstraction, storage failure fallback, native/debug in-memory backend, safe/unsafe settings entry, stable shell markers, keyboard focus baseline, independent menu/HUD scale fields, colorblind/reduced-motion preference updates, and PLACEMENT timer selector authority boundary verified.
- Test Evidence: `production/qa/evidence/accessibility-settings-foundation-2026-05-05.md`; `cargo test -p client --test accessibility_settings_preferences_test` passed 4/4; `cargo test -p client --test accessibility_settings_shell_test` passed 4/4; `cargo test -p client --test accessibility_settings_timer_selector_test` passed 4/4; `cargo test -p client --test presentation_plugin_scaffold_test` passed 5/5; `cargo fmt -p client -- --check`, `cargo check -p client`, and `git diff --check` passed.
- Verification: Current root `main` was clean and aligned with `origin/main` after `git fetch origin` before closure edits. No implementation code was changed during closure.
- Notes: No blocking GDD, ADR-002, ADR-021, ADR-023, Bevy 0.18, or control-manifest deviation found. Story manifest version `2026-05-05` matches the current control manifest. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates because `production/review-mode.txt` is absent.
- QA condition: `QA-COND-0005` remains Open. A11Y Settings 001 clears only the Settings foundation and preference persistence dependency evidence for future Standard-tier rows; it does not close colorblind palette application, reduced-motion consumers, full UI-scale evidence, pause-anywhere behavior, input remapping, tutorial persistence, brightness/gamma, or audio-control rows.
- Tech debt logged: None.
- Sprint status: `production/sprint-status.yaml` updated to record A11Y Settings 001 evidence under S6-04 while leaving S6-04/QA-COND-0005 open.
- Post-gate note: Remaining Standard-tier accessibility rows are Polish/public-release debt under the QA-COND-0005 friend-game-only accepted-risk disposition.

## Session Extract - /story-done 2026-05-06
- Verdict: COMPLETE
- Story: `production/epics/hud/story-012-text-size-and-contrast-accessibility.md` - Story 012: HUD Text Size and Contrast Accessibility Evidence
- Criteria: 19/19 passing; HUD text-size fixture, gold floors, reserved suffix floors, mana/reserve floors, phase/round floors, cold-start placeholders, HUD contrast pairs, DRAFT_AUCTION and RESOLUTION contrast, string preservation, reserve visibility, pre-pooled entity stability, focused/regression tests, browser/WASM evidence, two viewport captures, A11Y-ST-01/A11Y-ST-03 impact statements, QA-COND-0005 open statement, and whitespace gate verified.
- Test Evidence: `production/qa/evidence/hud-012-text-size-contrast-accessibility.md`; capture artifacts under `production/qa/evidence/captures/hud-012-text-size-contrast/`; `cargo test -p client --test hud_text_size_contrast_accessibility_test` passed 4/4; `cargo test -p client --test hud_gold_mana_display_test` passed 6/6; `cargo test -p client --test hud_phase_label_round_counter_test` passed 6/6; `cargo test -p client --test hud_economy_auction_inline_gold_test` passed 4/4; `cargo fmt -p client -- --check`, `cargo check -p client`, and `git diff --check` passed.
- Verification: Current root `main` was clean and aligned with `origin/main` after `git fetch origin` before closure edits. No implementation code was changed during closure.
- Notes: No blocking GDD, ADR-002, ADR-021, Bevy 0.18, or control-manifest deviation found. Story manifest version `2026-05-05` matches the current control manifest. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates because `production/review-mode.txt` is absent.
- QA condition: `QA-COND-0005` remains Open. HUD-012 closes only the HUD-owned portions of A11Y-ST-01 and A11Y-ST-03 in the Standard-tier accessibility disposition register; actual auction price counter and non-HUD UI/accessibility surfaces remain separate.
- Tech debt logged: None.
- Sprint status: `production/sprint-status.yaml` updated to record HUD-012 evidence under S6-04 while leaving S6-04/QA-COND-0005 open.
- Post-gate note: Remaining Standard-tier accessibility rows are Polish/public-release debt under the QA-COND-0005 friend-game-only accepted-risk disposition.

## Session Extract - /story-done 2026-05-06
- Verdict: COMPLETE
- Story: `production/epics/hand-ui/story-014-placement-staged-disclosure-accessibility.md` - Story 014: PLACEMENT Staged Disclosure Accessibility
- Criteria: 14/14 passing; A11Y-ST-14 entry state, card selection disclosure, lane/cell target guidance, preserved target-kind highlight semantics, invalid-drop recovery, valid board staging, Instant fan-plate staging, reserve/current split disclosure timing, reserve split behavior, submit invalid flow, correction/success path, Browser/WASM evidence completeness, QA-COND-0005 impact statement, and whitespace gate verified.
- Test Evidence: `production/qa/evidence/hand-ui-placement-staged-disclosure-accessibility-2026-05-05.md`; capture artifacts under `production/qa/evidence/captures/hand-ui-placement-staged-disclosure/`; `cargo test -p client --test hand_ui_placement_staged_disclosure_accessibility_test --jobs 1` passed 6/6; requested Hand UI regression groups passed 11/11, 13/13, and 25/25; `cargo fmt -p client -- --check`, `cargo check -p client --jobs 1`, and `git diff --check` passed.
- Verification: Current root `main` was clean and aligned with `origin/main` after `git fetch origin` before closure edits. No implementation code was changed during closure.
- Notes: No blocking GDD, ADR-002, ADR-021, ADR-023, Bevy 0.18, or control-manifest deviation found. Story manifest version `2026-05-05` matches the current control manifest. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates because `production/review-mode.txt` is absent.
- QA condition: `QA-COND-0005` remains Open. HAND-UI-014 closes only the A11Y-ST-14 PLACEMENT staged-disclosure sub-gap in the Standard-tier accessibility disposition register.
- Tech debt logged: None.
- Sprint status: `production/sprint-status.yaml` updated to record HAND-UI-014 evidence under S6-04 while leaving S6-04/QA-COND-0005 open.
- Post-gate note: Remaining Standard-tier accessibility rows are Polish/public-release debt under the QA-COND-0005 friend-game-only accepted-risk disposition.

## Session Extract - /story-done 2026-05-06
- Verdict: COMPLETE
- Story: `production/epics/shop-auction-ui/story-011-auction-bid-target-size-and-focus-evidence.md` - Story 011: Auction Bid Target Size and Focus Evidence
- Criteria: 16/16 passing; A11Y-ST-12 auction bid targets measure 108x44 CSS px, focus order is +1 -> +3 -> +5, focus rings expose 2px state, unaffordable controls skip focus, local leader hides bid controls, `BIDDING...` in-flight state disables further sends, and click/Enter activation preserves exact one-send semantics.
- Test Evidence: `production/qa/evidence/shop-auction-ui-auction-bid-target-focus-2026-05-05.md`; capture artifacts under `production/qa/evidence/captures/shop-auction-ui-auction-bid-target-focus/`; `cargo test -p client --test shop_auction_ui_auction_bid_target_focus_test --jobs 1` passed 4/4; requested SAU-004/005/006 regression group passed 17/17; `cargo test -p client --test shop_auction_ui_draft_initial_objective_overlay_test --jobs 1` passed 8/8; `cargo fmt -p client -- --check`, `cargo check -p client --jobs 1`, and `git diff --check` passed.
- Verification: Current root `main` had no implementation changes during closure. Browser/WASM evidence exists for affordable, focus +1/+3/+5, unaffordable, `BIDDING...`, and local `YOU ARE LEADING` states.
- Notes: No blocking GDD, ADR-013, ADR-019, ADR-021, Bevy 0.18, or control-manifest deviation found. Story manifest version `2026-05-05` matches the current control manifest. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates because `production/review-mode.txt` is absent.
- QA condition: `QA-COND-0005` remains Open. SAU-011 closes only the A11Y-ST-12 auction bid target/focus sub-gap in the Standard-tier accessibility disposition register.
- Tech debt logged: None.
- Sprint status: `production/sprint-status.yaml` updated to record SAU-011 evidence under S6-04 while leaving S6-04/QA-COND-0005 open.
- Post-gate note: Remaining Standard-tier accessibility rows are Polish/public-release debt under the QA-COND-0005 friend-game-only accepted-risk disposition.

## Session Extract - /story-done 2026-05-06
- Verdict: COMPLETE
- Story: `production/epics/shop-auction-ui/story-012-draft-initial-clear-objective-overlay.md` - Story 012: Draft Initial Clear Objective Overlay
- Criteria: 17/17 passing; DRAFT_INITIAL objective overlay activation waits for phase plus offering, exact copy is rendered, dismiss button and Esc dismissal work, non-actionable outside dismissal emits no C2S messages, guarded controls preserve behavior, retrieval reopens the same overlay, phase exit hides overlay/retrieval, purchase/Ready/Retract Ready/PLACEMENT regressions pass, and Browser/WASM evidence records non-occlusion/readability.
- Test Evidence: `production/qa/evidence/shop-auction-ui-draft-initial-clear-objective-overlay-2026-05-05.md`; capture artifacts under `production/qa/evidence/captures/shop-auction-ui-draft-initial-clear-objective-overlay/`; `capture-summary.json` aggregate verdict PASS; `cargo test -p client --test shop_auction_ui_draft_initial_objective_overlay_test --jobs 1` passed 8/8; requested Shop/Auction UI regression groups passed 17/17 and 17/17; `cargo fmt -p client -- --check`, `cargo check -p client --jobs 1`, and `git diff --check` passed.
- Verification: Current root `main` had no implementation changes during closure. Browser/WASM evidence exists for entry overlay, focused dismiss control, Esc dismissal, retrieval affordance, reopened overlay, and required grid, timer, Ready, HUD gold, and hand surface non-occlusion.
- Notes: No blocking GDD, ADR-015, ADR-021, Bevy 0.18, or control-manifest deviation found. Story manifest version `2026-05-05` matches the current control manifest. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates because `production/review-mode.txt` is absent.
- QA condition: `QA-COND-0005` remains Open. SAU-012 closes only the A11Y-ST-18 DRAFT_INITIAL clear-objective sub-gap in the Standard-tier accessibility disposition register.
- Tech debt logged: None.
- Sprint status: `production/sprint-status.yaml` updated to record SAU-012 evidence under S6-04 while leaving S6-04/QA-COND-0005 open.
- Post-gate note: Remaining Standard-tier accessibility rows are Polish/public-release debt under the QA-COND-0005 friend-game-only accepted-risk disposition.

## Session Extract - /story-done 2026-05-06
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/accessibility-settings/story-003-photosensitivity-warning-and-flash-audit.md` - Story 003: Photosensitivity Warning and Flash Audit
- Criteria: 14/14 passing; the evidence file exists, the audit covers the required effects and source documents, each row includes flash count/not-flashing rationale plus exposure and disposition, warning implementation is recorded, single-source warning copy is test-observable, boot-time warning and acknowledgement behavior are verified, reduced-motion scope was not expanded, QA-COND-0005 remains Open, and `git diff --check` passes.
- Test Evidence: `production/qa/evidence/accessibility-photosensitivity-warning-flash-audit-2026-05-05.md`; `cargo test -p client --test accessibility_settings_photosensitivity_warning_test --jobs 1` passed 4/4; `cargo test -p client --test presentation_plugin_scaffold_test --jobs 1` passed 5/5; `cargo fmt -p client -- --check`, `cargo check -p client --jobs 1`, and `git diff --check` passed.
- Verification: Current root `main` was clean and aligned with `origin/main` after `git fetch origin` before closure edits. No implementation code was changed during closure.
- Notes: No blocking GDD, ADR-002, ADR-021, Bevy 0.18, or control-manifest deviation found. Story manifest version `2026-05-05` matches the current control manifest. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates because `production/review-mode.txt` is absent.
- Residual risk: Objective destruction full-screen Prism White is documented as exceeding or unable to prove the local max 3 flashes/sec rule if counted as separate flashes. Story closure is valid as warning plus audit evidence because the evidence assigns the allowed "warning implemented now" disposition and keeps strict-compliance remediation or producer disposition visible before release if warning-only treatment is insufficient.
- QA condition: `QA-COND-0005` remains Open. A11Y-BS-03 closes only the photosensitivity warning/audit sub-gap in the Standard-tier accessibility disposition register.
- Tech debt logged: None.
- Sprint status: `production/sprint-status.yaml` updated to record A11Y-BS-03 evidence under S6-04 while leaving S6-04/QA-COND-0005 open.
- Post-gate note: Remaining Standard-tier accessibility rows are Polish/public-release debt under the QA-COND-0005 friend-game-only accepted-risk disposition.

## Session Extract - Producer Risk Disposition 2026-05-06
- Decision: `QA-COND-0005` / S6-04 is accepted risk for Sprint 6 friend-game scope only.
- Producer wording: "Friend-game scope - Standard-tier accessibility waived by producer. No public release, no obligation."
- Scope: This does not verify Standard-tier accessibility, close the underlying accessibility debt, or apply to any public/external release candidate. Remaining Standard-tier rows must be revisited before a public, external, commercial, or broader release.
- Sprint status: S6-04 is no longer an active P1 blocker. S6-06 later completed final re-smoke, QA sign-off, and the Production -> Polish gate-check, carrying QA-COND-0005 and QA-COND-0006 as explicit accepted-risk conditions rather than verified evidence.

## Session Extract - S6-06 Post-Gate Reconciliation 2026-05-06
- Stage: `production/stage.txt` is `Polish`.
- Gate: `production/gate-checks/gate-production-polish-sprint-6-2026-05-06.md` records `PASS WITH CONDITIONS` at commit `f622605872856ea29014601db5a9996c8b3d1122`.
- Smoke: `production/qa/smoke-2026-05-06.md` records `PASS WITH WARNINGS`.
- QA sign-off: `production/qa/qa-signoff-sprint-6-2026-05-06.md` records `APPROVED WITH CONDITIONS`.
- Sprint status: docs-only reconciliation marks S6-06 done in `production/sprint-status.yaml` without running `/story-done`.
- Sprint 6 plan: `production/sprints/sprint-6.md` records the post-gate state and preserves the carried conditions.
- Current condition summary: QA-COND-0005 remains friend-game-only accepted risk, not verified Standard-tier accessibility completion; QA-COND-0006 remains accepted-risk/deferred, not playtest or fun-hypothesis validation; QA-COND-0001 is Closed / N/A - Closed via integrated two-client FIFO evidence at `8fc30de33526d2f44e53d7e1fa82673ee4f26ccf`; QA-COND-0007 is Closed / N/A - Closed via Hand UI visual evidence and resolution replay readability evidence; full manual playable-client QA is not claimed.

## Session Extract - /story-done 2026-05-06
- Verdict: COMPLETE WITH NOTES
- Story: `production/epics/board-rendering/story-006-resolution-anim-queue-and-phase-buffering.md` - Story 006: Resolution AnimQueue and Phase Buffering
- Criteria: 7/7 passing; `S2CResolutionEvent` groups by sub-step, groups sort ascending, out-of-range sub-steps reject the entire queue and enqueue one snapshot recovery request, `PendingResolutionScript` buffers early scripts, `PendingPhaseChange` buffers RESOLUTION phase changes, buffered phase applies only after queue drain, and same-frame resolution plus DRAFT_SHOP phase preserves replay before DRAFT resumes.
- Test Evidence: `tests/integration/board_rendering/resolution_anim_queue_test.rs`; `cargo test -p client --test board_rendering_resolution_anim_queue_test` passed 5/5; relevant regressions passed for placement reveal, snapshot spawn cleanup, spawn-range highlights, AnimQueue drain/GAME_OVER skip, and objective stagger.
- Verification: Root `main` was clean and aligned with `origin/main` after `git fetch origin` before closure edits. Integrated implementation commit `8caa1a0195fd817b1ce632877db2174a357e8162` is present on main.
- Notes: No blocking GDD, ADR-017, ADR-021, Bevy 0.18, or control-manifest deviation found. Story manifest version `2026-05-05` matches the current control manifest. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW gates because `production/review-mode.txt` is absent.
- QA condition: BR-006 alone did not close `QA-COND-0007`; post-gate reconciliation now records QA-COND-0007 as Closed / N/A - Closed via separate Hand UI visual evidence and resolution replay readability evidence. Full playable-client manual QA is still not claimed.
- Tech debt logged: None.
- Sprint status: `production/sprint-status.yaml` reconciled S6-S4 / QA-COND-0007 as closed after the separate visual/readability evidence, not from BR-006 story completion alone.
- Next recommended: Do not carry QA-COND-0007 into Sprint 7 planning; preserve the evidence boundary that BR-006 infrastructure alone did not claim full playable-client manual QA.

## Session Extract - Prompt 266 Post-Gate Condition Status Reconciliation 2026-05-06
- Scope: docs-only reconciliation of current planning/status references after QA-COND-0001 and QA-COND-0007 closure; no `/dev-story`, `/story-done`, `/smoke-check`, `/team-qa`, or `/gate-check` run.
- Stage: `production/stage.txt` remains `Polish`.
- QA-COND-0001: Closed / N/A - Closed via integrated two-client FIFO evidence at `8fc30de33526d2f44e53d7e1fa82673ee4f26ccf`.
- QA-COND-0007: Closed / N/A - Closed via Hand UI visual evidence and resolution replay readability evidence.
- QA-COND-0005: remains accepted-risk / friend-game-only waiver; this is not verified Standard-tier accessibility completion.
- QA-COND-0006: remains accepted-risk/deferred; this is not playtest evidence or fun-hypothesis validation.
- Planning impact: `production/sprint-status.yaml`, `production/qa/bug-register-taxonomy.md`, and `production/sprints/sprint-6.md` no longer carry QA-COND-0001 or QA-COND-0007 as open active conditions, and QA-COND-0007 should not be pulled into Sprint 7 planning.
- Scope guard: full playable-client manual QA is not claimed.

## Session Extract - Prompt 268 /sprint-plan Sprint 7 2026-05-06
- Stage: `production/stage.txt` is `Polish`.
- Sprint 7 plan created at `production/sprints/sprint-7.md`; `production/sprint-status.yaml` now points to Sprint 7.
- Sprint 7 goal: make the Polish build playable for an internal friend-game session through the real primary client path, without claiming public release readiness, broad accessibility completion, or full playtest validation.
- Must Have: PLAYABLE-001 Primary Client Bootstrap + Fresh Lobby Entry; PLAYABLE-002 Live Draft/Shop/Hand Bridge; PLAYABLE-003 Real End-to-End Loop Verification.
- PLAYABLE story files were not created by `/sprint-plan`; they are listed in Sprint 7 and must be created/readiness reviewed in a separate docs-only prompt before `/dev-story`.
- Scope guard: QA-COND-0005 remains friend-game-only accepted risk, QA-COND-0006 remains accepted-risk/deferred, and full playable-client manual QA is not claimed.
- First recommended prompt: create the Sprint 7 PLAYABLE story docs/readiness package, then run `/qa-plan sprint-7` before implementation.

## Session Extract - /story-done 2026-05-06
- Verdict: COMPLETE
- Story: `production/epics/playable-client/story-001-primary-client-bootstrap-fresh-lobby-entry.md` - Story 001: Primary Client Bootstrap + Fresh Lobby Entry
- Criteria: 10/10 passing; primary client bootstrap, fresh hello identity mapping, create/join/class server-confirmed lobby flow, authoritative session entry gating, no optimistic authority, friend-game lobby readability, regression commands, and evidence document verified for PLAYABLE-001 scope.
- Test Evidence: `production/qa/evidence/playable-client-lobby-entry.md`; `cargo test -p server --test playable_client_lobby_entry_server_test` passed 3/3; `cargo test -p client --test playable_client_lobby_entry_test` passed 5/5; requested server regression bundle passed 22/22; `e2e_websocket_test --no-run`, `presentation_plugin_scaffold_test`, format, client/server checks, and `git diff --check` passed.
- Verification: Root `main` was clean and aligned with `origin/main` at commit `85878d254a569ef19f54cee0e66ada2575b13e50` after `git fetch origin` before closure edits. Integrated implementation commit `85878d254a569ef19f54cee0e66ada2575b13e50` is present on main.
- Notes: No blocking GDD, ADR-002, ADR-003, ADR-008, ADR-011, ADR-012, ADR-021, Bevy 0.18, Lightyear, or control-manifest deviation found. Story manifest version `2026-05-05` matches the current control manifest. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW because `production/review-mode.txt` is absent.
- Scope guard: PLAYABLE-001 closure is lobby/bootstrap only. It does not claim PLAYABLE-002 draft/shop/hand bridge completion, PLAYABLE-003 two-real-client end-to-end evidence, public release readiness, broad accessibility completion, playtest validation, full playable-client manual QA, or a complete primary-client game loop.
- QA conditions: `QA-COND-0005` remains accepted risk for friend-game scope only and is not verified Standard-tier accessibility completion. `QA-COND-0006` remains accepted-risk/deferred and is not playtest evidence or fun-hypothesis validation.
- Tech debt logged: None.
- Sprint status: `production/sprint-status.yaml` marks PLAYABLE-001 done; PLAYABLE-002 remains not started and is unblocked for `/dev-story`; PLAYABLE-003 remains blocked by PLAYABLE-002.
- Next recommended: `/dev-story production/epics/playable-client/story-002-live-draft-shop-hand-bridge.md`.

## Session Extract - /story-done 2026-05-06
- Verdict: COMPLETE
- Story: `production/epics/playable-client/story-002-live-draft-shop-hand-bridge.md` - Story 002: Live Draft/Shop/Hand Bridge
- Criteria: 11/11 passing; live DRAFT_INITIAL offering, authoritative DRAFT_INITIAL purchase, shared economy projection, ready/retract server path, server-owned all-ready progression, live DRAFT_SHOP slots, DRAFT_SHOP purchase/refresh messages, snapshot recovery, regression commands, and evidence document verified for PLAYABLE-002 scope.
- Test Evidence: `production/qa/evidence/playable-client-draft-shop-hand-bridge.md`; `cargo test -p client --test playable_client_draft_shop_hand_bridge_test` passed 4/4; `cargo test -p server --test playable_client_draft_ready_bridge_test` passed 3/3; relevant client regressions passed 52/52; relevant server regressions passed 70/70; format, client/server checks, and `git diff --check` passed.
- Verification: Root `main` was clean and aligned with `origin/main` at commit `1c4d34df0b0c1ee67df7210819c791671c708c81` after `git fetch origin` before closure edits. Integrated implementation commits `5077839bfc606dd46deea6baccdedc04e6ea75f0` and `1c4d34df0b0c1ee67df7210819c791671c708c81` are present on main.
- Notes: No blocking GDD, ADR-002, ADR-008, ADR-010, ADR-015, ADR-019, ADR-021, Bevy 0.18, Lightyear, or control-manifest deviation found. Story manifest version `2026-05-05` matches the current control manifest. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW because `production/review-mode.txt` is absent.
- Scope guard: PLAYABLE-002 closure is friend-game draft/shop/hand bridge only. It does not claim PLAYABLE-003 two-real-client end-to-end evidence, public release readiness, broad accessibility completion, playtest validation, full playable-client manual QA, or a complete primary-client game loop.
- QA conditions: `QA-COND-0005` remains accepted risk for friend-game scope only and is not verified Standard-tier accessibility completion. `QA-COND-0006` remains accepted-risk/deferred and is not playtest evidence or fun-hypothesis validation.
- Tech debt logged: None.
- Sprint status: `production/sprint-status.yaml` marks PLAYABLE-002 done and PLAYABLE-003 ready/unblocked.
- Next recommended: `/dev-story production/epics/playable-client/story-003-real-end-to-end-loop-verification.md`.

## Session Extract - /story-done 2026-05-07
- Verdict: COMPLETE
- Story: `production/epics/playable-client/story-003-real-end-to-end-loop-verification.md` - Story 003: Real End-to-End Loop Verification
- Criteria: 11/11 passing; evidence uses same-commit controlled real-Lightyear server/client paths with no direct World injection, lobby/session entry is covered, draft/shop is covered, auction bid/settlement and AuctionWon acquisition are covered, non-empty placement and both-client `S2CPlacementReveal` are covered, `UnitPlaced` resolution replay and next-loop `DRAFT_SHOP` are covered, nearest endpoint is documented honestly, defects are classified, forbidden public-release/playtest/QA claims are absent, required commands pass, and required evidence exists.
- Reached endpoint: `DRAFT_INITIAL -> PLACEMENT(empty) -> RESOLUTION -> DRAFT_SHOP -> PLACEMENT(non_empty) -> RESOLUTION -> DRAFT_AUCTION -> DRAFT_SHOP -> PLACEMENT(non_empty) -> RESOLUTION -> DRAFT_SHOP`.
- Game-over: Not reached and not claimed. The documented nearest reachable friend-game endpoint is next-loop `DRAFT_SHOP` after post-auction placement/resolution.
- Test Evidence: `production/qa/evidence/playable-client-real-e2e-loop.md`, captures under `production/qa/evidence/captures/playable-client-real-e2e-loop/`, and `tests/integration/playable_client/real_e2e_loop_test.rs`.
- Verification: `cargo test -p server --test playable_client_real_e2e_loop_test` passed 4/4; `cargo test -p client --test playable_client_lobby_entry_test` passed 5/5; `cargo test -p client --test playable_client_draft_shop_hand_bridge_test` passed 4/4; `cargo test -p server --test playable_client_draft_ready_bridge_test` passed 3/3; `cargo test -p server --test e2e_websocket_test` passed 1/1; `cargo check --workspace`, `cargo fmt -p client -p server -- --check`, and `git diff --check` passed.
- Notes: No blocking GDD, ADR, Bevy 0.18, Lightyear, or control-manifest deviation found for serialized story-done closure. Story manifest version `2026-05-05` matches the current control manifest. Lean mode skipped QL-TEST-COVERAGE and LP-CODE-REVIEW because `production/review-mode.txt` is absent.
- Scope guard: PLAYABLE-003 closure is internal friend-game evidence only. It does not claim public release readiness, broad accessibility completion, playtest validation, fun-hypothesis validation, QA sign-off, full playable-client manual QA, live native manual two-client completion, game-over coverage, or full game completion.
- QA conditions: `QA-COND-0005` remains accepted risk for friend-game scope only and is not verified Standard-tier accessibility completion. `QA-COND-0006` remains accepted-risk/deferred and is not playtest evidence or fun-hypothesis validation.
- Tech debt logged: None.
- Sprint status: `production/sprint-status.yaml` marks PLAYABLE-003 done. All Sprint 7 Must Have PLAYABLE stories are complete.
- Next recommended: `S7-N1` friend-game evidence index cleanup is unblocked; after that, choose whether to pull conditional Should Have work or proceed to the Sprint 7 close-out sequence later. No `/smoke-check`, `/team-qa`, `/gate-check`, or Sprint 8 planning was run in this serialized story-done scope.

## Session Extract - Prompt 305 S7-N1 Evidence Index Cleanup 2026-05-07
- Scope: docs-only evidence index cleanup for Sprint 7 friend-game auditability; no `/smoke-check`, `/team-qa`, `/gate-check`, or Sprint 8 planning was run.
- Evidence index: `production/qa/evidence/sprint-7-friend-game-evidence-index.md`.
- Linked story evidence: `production/qa/evidence/playable-client-lobby-entry.md`, `production/qa/evidence/playable-client-draft-shop-hand-bridge.md`, and `production/qa/evidence/playable-client-real-e2e-loop.md`.
- Capture spine: `production/qa/evidence/captures/playable-client-real-e2e-loop/`, especially `prompt-290-room-session-trace.json`, `prompt-296-draft-shop-trace.json`, `prompt-298-auction-placement-resolution-trace.json`, and `phase-captures.md`.
- Verified endpoint: next-loop DRAFT_SHOP after post-auction placement/resolution.
- Forbidden claims preserved exactly: no public release readiness, no broad accessibility completion, no playtest/fun-hypothesis validation, no full playable-client manual QA, no full game completion.
- QA conditions: `QA-COND-0005` remains accepted risk for friend-game scope only and is not verified Standard-tier accessibility completion. `QA-COND-0006` remains accepted-risk/deferred and is not playtest evidence or fun-hypothesis validation.
- Sprint status: `production/sprint-status.yaml` marks `S7-N1` done.
- Next recommended: decide whether to pull conditional Sprint 7 Should Have work (`SAU-007` or `ECO-004`) or begin the later Sprint 7 close-out sequence when requested.
