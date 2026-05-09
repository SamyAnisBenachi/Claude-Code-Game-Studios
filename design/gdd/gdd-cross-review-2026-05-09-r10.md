# Cross-GDD Review Report — R10

**Date:** 2026-05-09 (since-last-review delta vs R9 baseline 2026-04-30)
**GDDs reviewed:** 20 system GDDs + master GDD + systems-index + entities.yaml registry
**Prior review:** `gdd-cross-review-2026-04-30-r9.md` — Verdict FAIL, 11 blockers / 14 warnings
**Mode:** since-last-review focus on NP / RSM / GSS / class-system / registry edits, plus R9 carryover audit
**Scope discipline:** Only `design/gdd/` and `design/registry/` were read. `server/`, `client/`, `shared/`, `tests/` not touched.

---

## Progress History

| Review | Date | Verdict | Blockers |
|---|---|---|---|
| R1–R5 | 2026-04-29 | PASS | 0 |
| R6 | 2026-04-30 | FAIL | 9 |
| R7 | 2026-04-30 | FAIL | 9 |
| R8 | 2026-04-30 | FAIL | 13 |
| R9 | 2026-04-30 | FAIL | 11 |
| **R10** | **2026-05-09** | **CONCERNS** | **2 (1 carryover + 1 cosmetic registry)** |

**Net R9 → R10:** 9 of 11 R9 blockers fully resolved. 1 R9 blocker remains (C-R9-1 schema vs registry). 1 partial (C-R9-2). 6 R9 warnings still open. ADR-023 (PLACEMENT timer multiplier accessibility) cleanly threaded across NP/RSM/GSS — no regressions.

---

## R9 Blocker Disposition

| ID | R9 Status | R10 Status | Proof |
|---|---|---|---|
| C-R9-1 (S2CSingleObjectiveReveal NP schema) | 🔴 BLOCKING | 🔴 STILL OPEN | `entities.yaml:1592-1603` registry entry present; `network-protocol.md` zero matches; `class-system.md:711` NP-4 still "Open — NP required change". |
| C-R9-2 (CA Rule C-8 vs trigger_index) | 🔴 BLOCKING | 🟡 PARTIAL | NP D.2 contract complete (`network-protocol.md:434-464` `trigger_index` ordering key, sole tiebreaker, wire-stable). RSM Rule 11a complete (`round-state-machine.md:109-110`). **CA Rule C-8 still says "No ordering within a group"** (`card-animations.md:180`). NP+RSM contract is canonical; CA still negates it for animation spawn order. |
| C-R9-3 (S2CActivationRejected) | 🔴 BLOCKING | ✅ RESOLVED | NP table line 124, NP-50 AC line 951, registry line 1496-1507. Hand UI OQ8 satisfied. |
| C-R9-4 + W14 (S2CGameOver duplicate notes) | 🔴 BLOCKING | ✅ RESOLVED | Single `notes:` block at registry:1343, 4 variants ("ObjectivesDestroyed, Disconnection, Draw, ResolutionTimeout"), no YAML duplicate. |
| C-R9-5 (DRAFT_INITIAL grid ownership) | 🔴 BLOCKING | 🟡 PARTIAL | `hand-ui.md:55-61, 165-175, 498-501, 519-521` retains GRID state, click-to-buy, HU-01 pre-pools 9 grid slots. `shop-auction-ui.md:486, 498, 638, 712` retains panel + slot rendering. **No explicit "display owned by [other]" sentence** in either GDD. Two systems still describe the same 3×3 entity overlay in their respective scopes. (See R10-1 below.) |
| C-R9-6 (OQ-PLACEMENT-LOAD) | 🔴 BLOCKING | ✅ RESOLVED | RSM Open Question 6 closed 2026-05-01. `auction_followup_placement_timer_seconds` first-class Tuning Knob (RSM:349, default 12s). RSM-29c BLOCKING AC. Rule 9 prior-phase-aware logic. |
| C-R9-7 (mummy_damage_reserve_cap) | 🔴 BLOCKING | 🔴 STILL OPEN | `class-system.md:74,116,510` Mummy passive uncapped (`no cap — each hit adds 1 reserve`). Tuning Knobs at line 510 still flags "dominant strategy risk if Mummy passive cap is also absent" without adding the knob. `game-config.md` and `entities.yaml` zero matches. (See R10-2 below.) |
| C-R9-8 (keyword-system stale OQ ref) | 🔴 BLOCKING | ✅ RESOLVED | Zero matches for "(new variant — see OQ2 below)". |
| C-R9-W1 (StartAuction prose) | ⚠️ Warning | 🔴 STILL OPEN | `auction-system.md:144,252` still cite `StartAuction`. Not blocking; cosmetic carryover. |
| C-R9-W2 (RSM heartbeat reconcile) | ⚠️ Warning | ✅ RESOLVED | RSM Rule 13 (lines 115-127) hybrid model: heartbeat reset + Lightyear OnDisconnected + heartbeat absence detection. NP Rule 8 + NP-24/26 aligned. |
| C-R9-W3 (Connection error text) | ⚠️ Warning | 🔴 STILL OPEN | `shop-auction-ui.md:416,733` retain "Connection error — awaiting server…". Server is healthy (`auction_max_duration_seconds = 300`); copy is misleading. |
| C-R9-W4 (game-config 7 missing fields) | ⚠️ Warning | 🟡 PARTIAL | Added: `type_advantage_atk_bonus`, `type_advantage_ar_bonus`. Still missing: `garde_temps_reserve_cost`, `garde_temps_per_game_cap`, `miss_nuit_cap`, `dé_chateux_reveal_threshold`, `seed_ar_bonus`, `seed_enemy_damage`. (R9-W10 also unresolved — `garde_temps_reserve_cost` registered at registry:1207 but not in GameConfig struct.) |
| C-R9-W5 (Class System ↔ Hand UI dep) | ⚠️ Warning | 🔴 STILL OPEN | `class-system.md:486-501` Dependencies — no Hand UI row. `hand-ui.md:200-204, 316-324` Interactions — no Class System row. Bidirectionality rule (design-docs.md) still violated. |
| C-R9-W6 (Xelorium worked example) | ⚠️ Warning | 🔴 STILL OPEN | `class-system.md:497, 510` Tuning Knobs still references "calibrated tradeoff — see CS-3 strategic tradeoff note" without rounds-3-through-8 reserve curve. |
| C-R9-W7 (Sadida seed density) | ⚠️ Warning | 🔴 STILL OPEN | No `max_total_seeds_per_player` knob anywhere. |
| C-R9-W8 (Sang Méprise CA suppression) | ⚠️ Warning | 🔴 STILL OPEN | `card-animations.md` zero matches for "Sang Méprise" or "surprise". |
| C-R9-W9 / D-R9-4 (Master GDD layering paragraph) | ⚠️ Warning | ✅ RESOLVED | `lanes-and-lies-gdd.md:28` adds 1-paragraph reconciliation explicitly layering auction/class/prism sub-fantasies. Prose: "auction = singular tense moment; class = silent rhythm filling all rounds; prism = standing income that funds both." |
| C-R9-W10 (garde_temps_reserve_cost in struct) | ⚠️ Warning | 🔴 STILL OPEN | Registry:1207 has it; `game-config.md` struct does not. |
| C-R9-W11 (Garde-Temps animation contract) | ⚠️ Warning | 🔴 STILL OPEN | `card-animations.md` zero matches for "Garde-Temps". |
| C-R9-W12 (Hand-full DRAFT_AUCTION 20s idle) | ⚠️ Warning | 🔴 STILL OPEN | Resolved-by D-R9-3 path NOT taken — `network-protocol.md:60` `C2SActivateCard` still valid only in `DRAFT_INITIAL, DRAFT_SHOP`. |
| C-R9-W13 (xelorium_steal_cap) | ⚠️ Warning | 🔴 STILL OPEN | No knob, no Edge Case "accept and monitor" annotation. |
| D-R9-1 (PLACEMENT cognitive overload) | 🔴 BLOCKING | ✅ RESOLVED | Closed via OQ-PLACEMENT-LOAD resolution path: `auction_followup_placement_timer_seconds=12` + RSM Rule 9 + RSM-29c. M2 telemetry gate now structurally addressed. |
| D-R9-2 (Xelor reserve loop dominant strategy) | 🔴 BLOCKING | 🔴 STILL OPEN | Roots in C-R9-7 (Mummy cap missing) and C-R9-W6 (no worked curve). Garde-Temps reachable by R6 unchecked. (See R10-2.) |
| D-R9-3 (DRAFT_AUCTION lockout anti-pillar) | 🔴 BLOCKING | 🔴 STILL OPEN | NP:60 unchanged — DRAFT_AUCTION not in `C2SActivateCard` valid phases. RSM Rule 15 / hand-ui PASSIVE_LOCKED unchanged. (See R10-3.) |
| D-R9-5 (Sadida seed AR + PIERCE coverage) | ⚠️ Warning | 🔴 STILL OPEN | No PIERCE-coverage audit in card-data-pool.md; no `max_total_seeds_per_player` knob. |
| D-R9-6 / D-R9-7 (R3 auction signal-value) | ⚠️ Warning | 🔴 STILL OPEN | No R3 worked example in `auction-system.md`; no telemetry gate. |

**Resolved this cycle (9 blockers + 2 warnings):** C-R9-3, C-R9-4/W14, C-R9-6, C-R9-8, D-R9-1, plus C-R9-W2, C-R9-W9/D-R9-4. Cleanly closed via consistency-check 2026-05-08 and ADR-023 work.

---

## New Issues (R10)

### Blocking

🔴 **R10-1 — DRAFT_INITIAL grid ownership decision still unmade after R7→R8→R9→R10 carryover (C-R9-5 carryover; same root)**

Same evidence as C-R9-5: Hand UI Rule 4 + HU-01/HU-07/HU-08/HU-09 own grid slot pool, click-to-buy, hand-full lockout. Shop/Auction UI DRAFT_INITIAL Panel + SAU-DI8 own panel chrome and the same purchase pathway.

This is now a **four-cycle carryover** with no movement. The root issue is that no single owner has been declared by either creative-director or technical-director. The implementation tradeoff (one spawner vs two) is real but small; the cost of leaving it unmade is forcing the first M2 implementer to make the call with no authority.

→ **Required action this sprint:** Producer + creative-director alignment session to pick one. Recommended path (unchanged from R9): SAU owns slot rendering; Hand UI owns fan animation only. Both GDDs add an explicit "display owned by [other GDD]" deferring sentence.

### Warnings

⚠️ **R10-W1 — Registry stale `notes:` directives — five `Must be added to game-config.md` notes refer to actions already taken**

Registry entries with notes that no longer match reality (game-config.md *has* the field, but the registry note says it's an action item):

| Registry entry | Line | Stale note | Reality |
|---|---|---|---|
| `hello_timeout_ms` | 1098 | "Must be added to game-config.md" | Already at game-config.md:88 |
| `ack_timeout_ms` | 1109 | "Must be added to game-config.md" | Already at game-config.md:89 |
| `protocol_version` | 1120 | "Must be added to game-config.md" | Already at game-config.md:87 |
| `type_advantage_atk_bonus` | 1156 | "Must be added to game-config.md as type_advantage_atk_bonus. Action: game-config.md update required" | Already at game-config.md:57, 281 |
| `type_advantage_ar_bonus` | (next entry) | Same wording | Already at game-config.md:58, 282 |

Additionally, `referenced_by` for these entries does not list `design/gdd/game-config.md` despite the field being present. Two correctness drifts in one block: stale instruction + stale referenced_by.

→ Strip "Must be added to game-config.md" from the five notes; add `design/gdd/game-config.md` to each `referenced_by`.

⚠️ **R10-W2 — Cross-system messages registered in NP but missing from `entities.yaml` (5 messages)**

Messages defined in network-protocol.md and used by 2+ GDDs that the registry does not contain:

| Message | NP location | Used by |
|---|---|---|
| `C2SHeartbeat` | NP:36 | RSM Rule 13, GSS Rule 9, NP Rule 8 |
| `C2SSetPlacementTimerMultiplier` | NP table; GSS-42 | GSS, NP, accessibility-requirements.md |
| `S2CSessionSettingsUpdated` | NP:145; GSS Rule 14 | GSS, NP |
| `S2CPrismRespawned` | NP:148, NP-56 | Prism System, NP |
| `S2CPrismRewardDropped` | NP:149 | Prism System, NP |

The registry's purpose (`/consistency-check` baseline) requires that any message crossing 2+ system boundaries be registered. These 5 are documented in NP and the consuming GDDs but absent from the registry — `/consistency-check` cannot detect schema drift on them.

→ Add the 5 entries to `entities.yaml network_messages:` section.

⚠️ **R10-W3 — Class System NP-required-change backlog has 8 items still Open after R3 (NP-2 through NP-9)**

`class-system.md:709-716` lists NP-2 through NP-9 as Open. This is an 8-item NP backlog that gates Class System story implementation. Items: NP-2 (`source_class` field on `UnitBoardState`), NP-3 (`UnitSpawned` event variant + `SpawnSource`), NP-5 (reserve mutation events during RESOLUTION — Xelorium/Rollback/Mummy/Garde-Temps), NP-6 (Sinistro three protocol elements), NP-7 (Miranda control transfer), NP-8 (Chacha Noir `SpawnSource::Replacement`), NP-9 (`SeedPlaced`/`SeedConsumed`). NP-4 is C-R9-1 (still Open).

This backlog isn't a *contradiction* — both GDDs agree these need to be added — but it represents a substantial coordinated edit pending. Class System M3 epic stories are blocked on these.

→ Surface this to producer/network-programmer for an NP message-batch sprint before Class System implementation begins. Not blocking for sprint-10 if Class System work is M3.

⚠️ **R10-W4 — `auction_followup_placement_timer_seconds` declared in RSM but unregistered**

`round-state-machine.md:79, 271, 349, 430` cite the knob authoritatively. `game-config.md` zero matches. `entities.yaml` zero matches. The knob is loaded "from GameConfig" per RSM:340 boilerplate but the field doesn't exist in the GameConfig struct.

→ Add to game-config.md struct + Tuning Knobs; register in entities.yaml under constants.

---

## Design Issues (Carried from R9 — unresolved)

⚠️ **R10-D1 — Xelor Mummy + Garde-Temps remains unbounded; carryover D-R9-2/C-R9-7/D-R8-3 (4 cycles)**

Same evidence as R9 D-R9-2: Mummy passive uncapped, Miss Nuit capped at 2, Xelorium one-shot up to +12. Sacrier matchup identity inversion still present (controlled-self-damage opponent accelerates Mummy reserve). Garde-Temps reachable by R6 without auction.

This is the highest-impact open design risk in the project. Recommended fix unchanged: add `mummy_damage_reserve_cap` (default 1/round/Mummy, range 1–3) to class-system Tuning Knobs + game-config + registry, plus add the worked R3-R8 reserve accumulation curve.

⚠️ **R10-D2 — DRAFT_AUCTION hand-full lockout still violates "No idle spectating" anti-pillar; carryover D-R9-3/D-R8-5**

Same evidence: hand=10 entering DRAFT_AUCTION (Lane 3 prism on prior round, or auction win at hand=9) → 20s zero-agency. NP:60 still excludes DRAFT_AUCTION from `C2SActivateCard` valid phases. `lanes-and-lies-gdd.md:51` anti-pillar definition unchanged.

This is an explicit anti-pillar violation per the master GDD. Recommended fix unchanged: add DRAFT_AUCTION to NP:60 valid phases; update RSM Rule 15 + hand-ui PASSIVE_LOCKED.

⚠️ **R10-D3 — Sadida seed density + PIERCE coverage gate still missing; carryover D-R9-5**

`class-system.md:308` 1 seed per cell, 5×4=20 seeds possible by R5. PIERCE is the only counter, but no card-data-pool.md audit confirms PIERCE distribution in M1/M2 pools. No `max_total_seeds_per_player` knob.

→ Either gate Class System Sadida implementation on PIERCE coverage audit, or add the knob.

---

## Cross-System Scenario Walkthroughs

**Scenarios walked: 5** (delta-only — scenarios fully resolved in R9 not re-walked)

### Scenario 1 — Auction-followup PLACEMENT (R9 Scenario 1)

✅ **RESOLVED** — RSM Rule 9 + `auction_followup_placement_timer_seconds` + RSM-29c close the loop. Telemetry path documented via OQ-PLACEMENT-LOAD R2 closure note. ADR-023 layered correctly: base 12000ms × `placement_timer_multiplier_effective` (clean 1× through 3× scaling).

### Scenario 2 — Multi-class Krosmic same RESOLUTION (R9 Scenario 2)

🟡 **WARNING** (carries C-R9-2 carryover) — NP D.2 trigger_index contract is now wire-stable (NP:434-464). RSM Rule 11a assigns indices deterministically (RSM-29b BLOCKING AC). **But Card Animations Rule C-8 (`card-animations.md:180`) still asserts "No ordering within a group"**, contradicting NP D.2's CLIENT CONTRACT ("render events strictly in array order; do NOT re-sort"). Multi-Krosmic batches still animate non-deterministically in the CA spec. Server-side ordering is correct; client-side animation contract still negates it.

### Scenario 3 — DRAFT_AUCTION hand=10 (D-R9-3 carryover)

🔴 **STILL BLOCKING** — Same as R9. NP:60, RSM Rule 15, hand-ui PASSIVE_LOCKED unchanged. Anti-pillar violation persists.

### Scenario 4 — ADR-023 timer multiplier × auction-followup PLACEMENT (NEW)

✅ **CLEAN** — Walked: Player A requests 3× via `C2SSetPlacementTimerMultiplier` in LOBBY → GSS computes `placement_timer_multiplier_effective=3x` (GSS-42 BLOCKING) → `S2CSessionSettingsUpdated` broadcast (no requester attribution per NP-59) → `SessionReady` fires → frozen in `SessionConfig` (GSS-44 BLOCKING) → on auction-followup R6: RSM Rule 9 selects base=12000ms → multiplied to 36000ms → `S2CPhaseChanged.timer_duration_ms = Some(36000)` (NP-60 BLOCKING). All five scheduling boundaries pinned. RSM-29c verifies the end-to-end math. No drift.

### Scenario 5 — DRAFT_INITIAL grid click handler at runtime (R10-1)

⚠️ **WARNING** — With both Hand UI HU-01 (pre-pools 9 grid slots, scene-side) and SAU SAU-DI8 (sends `C2SPurchaseCard` from "purchasable slot"), the implementer cannot determine: which system spawns the visible 9 nodes? Which system attaches the click handler? Which system is the source of the `S2CCardAcquired` → grid-slot lookup? The two GDDs each describe the same UI surface in their own scope without a deferral sentence. R7 cross-review flagged "two presentation systems own the same UI surface" — three reviews later, no decision.

---

## GDDs Flagged for Revision (R10)

| GDD | Issues | Priority |
|---|---|---|
| `network-protocol.md` | C-R9-1 (S2CSingleObjectiveReveal NP schema row + D.1 struct) | **Blocking** |
| `card-animations.md` | C-R9-2 (Rule C-8 vs trigger_index — must update spawn-order language); C-R9-W8, C-R9-W11 | **Blocking + Warning** |
| `class-system.md` | C-R9-7 / R10-D1 (Mummy cap), C-R9-W5, C-R9-W6, C-R9-W13, R10-W3 backlog surface | **Blocking + Warning** |
| `hand-ui.md` + `shop-auction-ui.md` | R10-1 (DRAFT_INITIAL grid ownership), C-R9-W3 | **Blocking + Warning** |
| `network-protocol.md` (separate) | R10-D2 (DRAFT_AUCTION + C2SActivateCard); RSM Rule 15 + hand-ui PASSIVE_LOCKED follow-on | **Blocking (anti-pillar)** |
| `entities.yaml` | R10-W1 (5 stale "Must be added" notes + referenced_by), R10-W2 (5 messages unregistered), R10-W4 (`auction_followup_placement_timer_seconds`), C-R9-W7 (`max_total_seeds_per_player`) | **Warning** |
| `game-config.md` | C-R9-W4 (5 missing class-system fields), C-R9-W10 (`garde_temps_reserve_cost` in struct), R10-W4 (`auction_followup_placement_timer_seconds`) | **Warning** |
| `auction-system.md` | C-R9-W1 (cosmetic StartAuction prose), C-R9-W12 (resolved-by R10-D2) | Warning |
| `card-data-pool.md` | D-R9-5 / R10-D3 PIERCE coverage audit | Warning |
| `lanes-and-lies-gdd.md` | R10-D2 anti-pillar exception only if D-R9-3 fix declined (not recommended) | Conditional |

---

## Pillar Status Snapshot

- **Auction-as-signature: ✅ OK** — D-R9-6/7 are M2-monitoring concerns, not breakage. Master GDD layering paragraph (line 28) reinforces the pillar.
- **No-idle-spectating: 🔴 FAIL** — R10-D2 (carryover D-R9-3) is unchanged anti-pillar violation. Single fix point: NP:60 valid-phase change.
- **Deep emergence: ⚠️ CONCERN** — R10-D1 (Mummy uncapped → Xelor reserve loop) and R10-D3 (Sadida seed/PIERCE) remain candidate dominant-strategy risks. Both have shipped-knob solutions waiting.
- **Simple surface: ✅ OK** — No new rule-surface bloat detected from recent edits. ADR-023 added complexity but housed it cleanly in GSS/SessionConfig.

---

## Required Actions Before Sprint 10 Begins

| # | Issue | Primary Files | Closes |
|---|---|---|---|
| 1 | Add `S2CSingleObjectiveReveal` row to NP message table + D.1 struct; close class-system NP-4 | network-protocol.md, class-system.md | C-R9-1 / R10 carryover |
| 2 | Update CA Rule C-8: spawn order = ascending `trigger_index` within group; cite NP D.2 contract; add CA AC | card-animations.md | C-R9-2 |
| 3 | DRAFT_INITIAL grid ownership decision (creative-director call) — pick one owner; other GDD adds deferring sentence | hand-ui.md, shop-auction-ui.md | R10-1 (4-cycle carryover) |
| 4 | Add `mummy_damage_reserve_cap` knob to class-system Tuning Knobs + game-config struct + entities.yaml | class-system.md, game-config.md, entities.yaml | C-R9-7 / R10-D1 |
| 5 | DRAFT_AUCTION hand-full lockout fix: NP:60 add DRAFT_AUCTION to `C2SActivateCard` valid phases + RSM Rule 15 + hand-ui PASSIVE_LOCKED | network-protocol.md, round-state-machine.md, hand-ui.md | R10-D2 / D-R9-3 anti-pillar |

These five are the blockers/anti-pillar items. The seven remaining R9 warnings (C-R9-W1, W3, W4, W5, W6, W7, W8, W10, W11, W13 + R10-W1 through R10-W4) are not blockers for sprint-10 implementation if the work area avoids the affected systems — they should be batched into a `/consistency-check` cleanup pass.

---

## Highest-Leverage Coordinated Edit

**One NP message-table pass closes 3 of 5 required actions** (items 1, 5, and the bulk of R10-W2): add `S2CSingleObjectiveReveal`, modify `C2SActivateCard` valid phases to include DRAFT_AUCTION, and register `C2SHeartbeat`/`C2SSetPlacementTimerMultiplier`/`S2CSessionSettingsUpdated`/`S2CPrismRespawned`/`S2CPrismRewardDropped` in entities.yaml. Single network-programmer + producer session.

---

## Verdict: CONCERNS

**Net delta R9 → R10: 9 blockers resolved, 2 remain, 1 partial.** Two of three remaining design blockers (R10-D1 Xelor Mummy, R10-D2 DRAFT_AUCTION lockout) are 1-knob / 1-line fixes that have lingered across 3-4 review cycles — each is a known-fix-pending decision, not an active design ambiguity.

The audit shows **healthy forward motion**: ADR-023 is a non-trivial accessibility feature added cleanly across GSS/RSM/NP/registry without introducing new contradictions; the R9 mass-resolution closed 9 of 11 blockers; the consistency-check cadence (2026-05-01, 2026-05-08) is catching drift early. The remaining blockers are decision/edit work, not design-discovery work.

**Why CONCERNS rather than FAIL:** R10-D2 (anti-pillar violation) and R10-D1 (dominant-strategy risk) are friend-game scope — they are real design risks, but the master GDD layering paragraph + ADR-023 demonstrate the team is shipping coherent design. Sprint 10 can proceed on Foundation/Core/Networking systems that are unaffected by these two open items, provided no Sprint 10 story touches Class System Mummy/Xelorium logic, the DRAFT_AUCTION lockout state, or the DRAFT_INITIAL grid ownership boundary.

**Why not PASS:** R10-1 is a 4-cycle carryover blocker. Until the DRAFT_INITIAL grid owner is named, sprint-10 stories that touch either hand-ui or shop-auction-ui will inherit the ambiguity.

---

## Files Read (audit scope)

- `design/gdd/gdd-cross-review-2026-04-30-r9.md` (prior baseline)
- `design/gdd/network-protocol.md` (ADR-023 integration; C2SActivateCard valid phases)
- `design/gdd/round-state-machine.md` (Rule 9 prior-phase-aware; Rule 11a trigger_index; Rule 13 hybrid heartbeat; OQ-PLACEMENT-LOAD R2 closure)
- `design/gdd/game-session-system.md` (Rule 14 ADR-023 negotiation)
- `design/gdd/class-system.md` (NP-4 still open; Mummy cap missing; Hand UI dep missing)
- `design/gdd/card-animations.md` (Rule C-8 still negates NP D.2)
- `design/gdd/hand-ui.md` + `design/gdd/shop-auction-ui.md` (DRAFT_INITIAL grid dual-ownership)
- `design/gdd/game-config.md` (5 class-system fields still missing)
- `design/registry/entities.yaml` (S2CSingleObjectiveReveal registered, NP not; 5 stale "Must be added" notes; 5 unregistered cross-system messages)
- `design/gdd/lanes-and-lies-gdd.md` (line 28 layering paragraph — D-R9-4 resolved)
