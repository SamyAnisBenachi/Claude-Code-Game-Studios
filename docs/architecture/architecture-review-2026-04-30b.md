# Architecture Review — 2026-04-30 (Run 2 — ADR-013..018 Wave)

## Document Status

| Field | Value |
|---|---|
| **Date** | 2026-04-30 |
| **Engine** | Bevy 0.18 + Lightyear 0.26 |
| **Scope** | Incremental review — 6 new ADRs (013–018) authored since Run 1 |
| **Prior Review** | `docs/architecture/architecture-review-2026-04-30.md` (M1 PASS, M2/M3 FAIL) |
| **GDDs Reviewed** | 20 system GDDs (full set: 9 M1 Approved/Needs Revision, 8 M2, 3 M3) |
| **ADRs Reviewed** | 18 (ADR-001 through ADR-018; ADR-001..012 Accepted; ADR-013..018 Proposed) |
| **Verdict** | **CONCERNS — M1 PASS / Core M2/M3 CONCERNS / Presentation M2/M3 FAIL** |

---

## Phase 1 — Loaded

- **GDDs**: 20 — card-data-pool, game-config, server-rng, economy-system, board-lane-system,
  round-state-machine, network-protocol, game-session-system, objective-system (M1);
  card-acquisition, auction-system, combat-resolution, board-rendering, hand-ui, shop-auction-ui,
  hud (M2); keyword-system, prism-system, class-system, card-animations (M3)
- **ADRs**: 18 total — ADR-001..012 (all Accepted), ADR-013..018 (all Proposed, new this wave)
- **Engine**: Bevy 0.18 + Lightyear 0.26 — HIGH risk; S1-05 spike verified 20 items
- **TR Registry**: `docs/architecture/tr-registry.yaml` — 80 M1 TRs (TR-CDP..TR-OBJ); M2/M3 pending
- **No `docs/consistency-failures.md`** — reflexion log append skipped

**New ADRs since last review:**

| ADR | Filename | H1 Title | Status | System |
|---|---|---|---|---|
| ADR-013 | `adr-013-auction-system-state.md` | ADR-013: Auction System State Machine | Proposed | M2 |
| ADR-014 | `adr-014-class-system-architecture.md` | ADR-014: Class System Architecture | Proposed | M3 |
| ADR-015 | `adr-015-card-acquisition-shop-state.md` | ⚠ **ADR-014** Card Acquisition Shop State | Proposed | M2 |
| ADR-016 | `adr-016-prism-system-architecture.md` | ADR-016: Prism System Architecture | Proposed | M3 |
| ADR-017 | `adr-017-combat-resolution-execution-architecture.md` | ADR-017: Combat Resolution Execution | Proposed | M2 |
| ADR-018 | `adr-018-keyword-system.md` | ADR-018: Keyword System | Proposed | M3 |

---

## Phase 2+3 — Traceability Summary

| Tier | Requirements | Covered | Partial | Gap |
|---|---|---|---|---|
| **M1** (ADR-001..012, 80 TRs) | 80 | 80 ✅ | 0 | 0 |
| **Core M2** (Auction, Combat, Card Acq) | 30 | 23 ✅ | 5 ⚠️ | 2 ❌ |
| **Core M3** (Class, Prism, Keyword) | 28 | 23 ✅ | 4 ⚠️ | 1 ❌ |
| **Presentation M2/M3** (5 systems) | 32 | 0 | 0 | 32 ❌ |
| **Total** | **170** | **126 (74%)** | **9 (5%)** | **35 (21%)** |

### Coverage Gaps — Core Layer (blocking)

| # | TR-ID | GDD | Requirement | Notes |
|---|---|---|---|---|
| 1 | TR-AU-010 | auction-system.md | Release-build `gold < reserved_gold` invariant guard (Rule 7 Case A fatal log) | ADR-013 covers debug_assert only; release-build path missing |
| 2 | TR-CR-010 | combat-resolution.md | Kill-gold attribution (+1g) Economy call-site ownership | ADR-017 silent on this |
| 3 | TR-CS-010 | class-system.md | Sang Méprise reveal-on-reconnect snapshot contract (OQ-CS-2 still open) | ADR-014 does not include re-delivery |

### Coverage Gaps — Partial (soft warnings)

| TR-ID | Issue |
|---|---|
| TR-AU-008 | Pre-S2CPhaseChanged ordering invariant not mechanically encoded in ADR-013 |
| TR-AU-009 | `S2CGoldBroadcast` broadcast ownership route to Economy missing explicit cross-reference |
| TR-CR-008 | SHIELD sub-step pre-check distinct from per-attacker modifier stack — ADR-017 does not call it out explicitly |
| TR-CA-009 | Class-filter wiring for DRAFT_INITIAL draw lacks explicit cross-link ADR-015→ADR-014→ADR-006 |
| TR-KW-008 | INJURED state-flag persistence under SILENCE — ADR-018 covers `silenced_until_round` but not state-flag separation |
| TR-KW-009 | RNG seed slots pre-implementation gate flagged but ADR-005 amendment not yet landed |
| TR-PRI-008 | `PrismBoardState` reconnect schema deferred to "GDD pre-impl fixes" — not architecturally locked |
| TR-PRI-004 | Lightyear `Replicate` per-entity scoping API unverified (ADR-008 checklist item 2 open) |
| TR-CA-009 | DRAFT_INITIAL class-filter dependency chain ADR-015→ADR-014→ADR-006 not cross-linked |

### Coverage Gaps — Presentation Layer (32 TRs, zero ADR coverage)

Systems with no governing ADR: board-rendering, hand-ui, shop-auction-ui, hud, card-animations.

Representative high-priority gaps requiring a Presentation Layer ADR (or one ADR per surface):

| TR-ID | Requirement |
|---|---|
| TR-BR-001 | `AnimQueue` Resource + `Time<Virtual>`-driven Timer for sub-step playback |
| TR-BR-002 | `BoardLayout` resource — single coordinate authority (`cell_to_world(lane, cell)`) |
| TR-BR-003 | Z-order layer constants in `rendering_constants.rs`; no inline literals |
| TR-BR-006 | `PendingPhaseChange` + `PendingResolutionScript` resources buffer premature messages |
| TR-BR-007 | Custom `SpriteAlphaLens` for `bevy_tweening` (no built-in alpha lens in 0.18) |
| TR-HU-002 | Card drag/stage state machine: `Idle → Dragging → Staged → Committed` |
| TR-HU-003 | `Res<BoardLayout>` consumed for cursor→cell mapping |
| TR-SAU-001 | `local_free_gold = gold − reserved_gold` formula (D.1) from `S2CGoldBroadcast` |
| TR-SAU-005 | Optimistic bid pending state — reversal on `S2CAuctionBidRejected` |
| TR-HUD-002 | Scoreboard objective dot state machine (`Hidden → Real → Fake → Destroyed`) |
| TR-CAN-001 | `bevy_tweening` hard dependency — no version-pinning ADR exists |
| TR-CAN-002 | Per-phase animation budget enforced (PLACEMENT hard cap 250 ms) |
| TR-CAN-004 | Mandatory simultaneous-start parallelism for placement reveal tweens |

**Suggested ADR**: `/architecture-decision presentation-layer` — covers Z-order, draw-call budget, `bevy_tweening` version pin, `BoardLayout` authority, `AnimQueue` pattern, and client state-mirror architecture.

---

## Phase 4 — Cross-ADR Conflict Detection

### 🔴 HARD CONFLICT — HC-1: ADR Numbering Duplicate

**`adr-015-card-acquisition-shop-state.md` line 1 reads:**
> `# ADR-014: Card Acquisition Shop State Machine Architecture`

**`adr-014-class-system-architecture.md` line 1 reads:**
> `# ADR-014: Class System Architecture — PlayerSessionState, SourceClass Component, and Direct Effect Dispatch`

Two files have identical H1 ADR numbers. Filename `adr-015-…` correctly encodes the intended number but the internal title does not. All internal cross-references in ADR-016 to "ADR-014 item 2" are ambiguous.

**Resolution**: Edit `adr-015-card-acquisition-shop-state.md` line 1 to `# ADR-015: Card Acquisition Shop State Machine Architecture`. Audit ADR-016 for stale internal references.

---

### 🔴 HARD CONFLICT — HC-2: Player Hand Storage Triple-Named

| ADR | Claim |
|---|---|
| ADR-014 (Class) line 87 | "Card Acq ADR → `hand: Vec<CardId>`" lives in `PlayerSessionData` |
| ADR-015 (Card Acquisition) | Defines separate `PlayerHands: Resource` with `HashMap<PlayerId, Vec<CardId>>` |
| ADR-016 (Prism) | References `HandState` (distinct third name) and `ResMut<HandState>` |

**Impact**: Three mutually incompatible implementations will be written if each story follows its governing ADR.

**Resolution options**:
1. ✅ Adopt ADR-015's `PlayerHands` as canonical (cleanest — hand lifecycle is orthogonal to session identity). Amend ADR-014 to remove the `hand` promise from `PlayerSessionData`; amend ADR-016 to rename `HandState → PlayerHands`.
2. Adopt ADR-014's unified `PlayerSessionData.hand`: refactor ADR-015 and ADR-016.

---

### 🔴 HARD CONFLICT — HC-3: `EconomyState` Resource Undefined

ADR-013 (Auction), ADR-015 (Card Acquisition) both take `ResMut<EconomyState>` as a system param, invoking `reserve_gold`, `release_gold_reservation`, `spend_gold`, `refund_gold`. ADR-014 says Economy fields (`gold`, `current_mana`, `reserve`) live inside `PlayerSessionData`, meaning `ResMut<PlayerSessions>` is the correct param.

No Economy ADR has been authored. The existing implementation in `server/src/core/economy/` predates the ADR wave — its Resource type name has not been canonically locked by an ADR.

**Impact**: ADR-013 and ADR-015 cannot both be Accepted until the Economy resource shape is decided.

**Resolution**: Author the Economy System ADR (proposed number: **ADR-019**) before accepting ADR-013 or ADR-015. ADR-019 must decide: unified `PlayerSessions` with economy fields, or separate `EconomyState` resource. Then amend ADR-013/015 parameter lists to match.

---

### 🔴 HARD CONFLICT — HC-4: Pre-Implementation Gates Not Landed

| Gate | Required By | Target ADR | Status |
|---|---|---|---|
| `SimpleKeyword` extended to 20 variants + adjacent serde tag + Charge→Haste rename | ADR-018 | ADR-006 | **MISSING** — ADR-006 still has 7-variant `SimpleKeyword` + `#[serde(untagged)]` |
| 3 new RESOLUTION seed slots (`range_equidistant_select`, `teleport_random_dest`, `strich_change_lane_select`) + `RngEvent` variants | ADR-018 | ADR-005 | **MISSING** — grep confirms zero matches in ADR-005 §4 |
| Subscriber Contracts row: `ResolutionPhaseEntered → Prism System` | ADR-016 | ADR-010 | **MISSING** — Prism not present in ADR-010 subscribers table |

Until these amendments land, ADR-018 and ADR-016 cannot safely advance to Accepted, and stories that implement keyword resolution or prism draws will silently reference a stale ADR-005/ADR-006.

**Note (landed gates)**: ADR-010 already has `AbortAuction` (for ADR-013) and `ShopRefreshTriggered` (for ADR-015) — both landed in ADR-010.

---

### 🟡 Soft Inconsistency — SI-1: ADR-013 Template Error (Duplicate Row)

`adr-013-auction-system-state.md` Engine Compatibility table has two `Post-Cutoff APIs Used` rows. Row 1 (line 36) is the correct content. Row 2 (line 37) says "None — `EventWriter`/`EventReader` not used" — this contradicts row 1 and was a template paste error.

**Resolution**: Delete line 37.

---

### 🟡 Soft Inconsistency — SI-2: ADR-010 Stale `StartAuction` Reference

ADR-010 line 738 prose still references old message name `StartAuction`. This was renamed to `AuctionPhaseEntered` (as part of ADR-013 authoring pass). The message catalog entries were updated but one prose line was missed.

---

### 🟡 Soft Inconsistency — SI-3: ADR-016 Internal Cross-Reference Bug

ADR-016 line 172 reads `"Verification required (ADR-014 item 2)"`. The intended referent is ADR-016's own Verification Required item 2 (Lightyear `Replicate` per-entity scoping). Should read `"Verification Required item 2"` (self-reference, no ADR prefix).

---

### ADR Dependency Ordering (Topologically Sorted)

No cycles detected across all 18 ADRs.

```
Foundation — no external deps (all Accepted):
  1. ADR-002  Client-Server Authority
  2. ADR-001  Objective Identity Unicast (→ ADR-002)
  3. ADR-003  Cargo Workspace Structure (→ ADR-002)

Infrastructure — depends on Foundation:
  4. ADR-004  Asset Loading Pipeline (→ ADR-003)
  5. ADR-008  Lightyear Channel Config (→ ADR-002, ADR-003)
  6. ADR-006  Card Data Schema + PlayerPool (→ ADR-003, ADR-004)
  7. ADR-005  Server-Side RNG (→ ADR-003; lifecycle → ADR-012)

Core Phase Orchestration:
  8. ADR-009  RSM Phase State (→ ADR-002, ADR-008)
  9. ADR-010  RSM Event Bus (→ ADR-009, ADR-003, ADR-008)
 10. ADR-012  SessionReady Delivery (→ ADR-009, ADR-005)
 11. ADR-011  Reconnect Snapshot (→ ADR-001, ADR-002, ADR-008)
 12. ADR-007  Placement Buffer (→ ADR-002, ADR-003, ADR-009)

M2/M3 Feature — first wave (all Proposed):
 13. ADR-013  Auction State Machine (→ ADR-009, ADR-010, ADR-002, ADR-008)
 14. ADR-014  Class System (→ ADR-002, ADR-003, ADR-005, ADR-006, ADR-009, ADR-010, ADR-012)
 15. ADR-017  Combat Resolution (→ ADR-002, ADR-009, ADR-010, ADR-005)

M2/M3 Feature — second wave (gate on ADR-013 or amendments):
 16. ADR-015  Card Acquisition / Shop State (→ ADR-009, ADR-010, ADR-013*, ADR-005, ADR-006, ADR-008)
 17. ADR-016  Prism System (→ ADR-005, ADR-010, ADR-008, ADR-006)
 18. ADR-018  Keyword System (→ ADR-002, ADR-005⚠, ADR-006⚠, ADR-009, ADR-010)

* ADR-015 depends on ADR-013 (Proposed-on-Proposed)
⚠ ADR-018 depends on ADR-005 amendment + ADR-006 amendment — not yet landed
```

**Recommended Accept order** (after fixes land):
ADR-013 (fixes only) → ADR-017 → ADR-014 → (Economy ADR-019) → ADR-015 → (ADR-005/006 amendments) → ADR-016 (ADR-010 row) → ADR-018

---

## Phase 5 — Engine Compatibility Audit

**ADRs with Engine Compatibility section**: 18/18 ✅

### New ADRs (013–018) Engine Audit

| ADR | Verdict | Key Finding |
|---|---|---|
| ADR-013 | 🟡 MEDIUM | Duplicate "Post-Cutoff APIs Used" row (template error). All actual APIs correct (MessageWriter/MessageReader/MessageReceiver, u32::try_from). |
| ADR-014 | 🟢 CLEAN | Correctly distinguishes `lightyear::prelude::Message` from `bevy::prelude::Message`. No deprecated APIs. |
| ADR-015 | 🟢 CLEAN | Correct MessageReader (Bevy) vs MessageReceiver (Lightyear) split. No deprecated APIs. |
| ADR-016 | 🟢 CLEAN | Engine Specialist Note explicitly disambiguates the two Message APIs. `Replicate` API flagged for verification. No deprecated APIs. |
| ADR-017 | 🟢 CLEAN | Correct `fn f(world: &mut World)` exclusive system pattern. `MessageWriter` vs Lightyear `MessageSender` explicitly separated. `despawn()` not `despawn_recursive`. |
| ADR-018 | 🟢 CLEAN | `app.add_message::<KeywordTriggered>()` registration flagged as a verification gate. `&Entities` system param flagged for 0.18 verification. No deprecated APIs. |

**Cross-ADR API consistency**: All 6 new ADRs use the MessageWriter/MessageReader (Bevy internal) vs MessageReceiver/MessageSender (Lightyear network) distinction correctly — consistent with ADR-010's critical API boundary note.

### Existing ADRs — No Regressions

ADR-001..012 engine status unchanged from prior review. Fixes applied in Run 1 (ADR-004 code samples, ADR-005 dep reference) are still in place.

### Deprecated API Check

Grep of ADR-013..018 for `EventWriter`, `EventReader`, `SpriteBundle`, `NodeBundle`, `Camera2dBundle`, `set_parent`, `despawn_recursive`, `UiImage`: **zero matches** (all mentioned only in negative/forbidden context).

### ADR-016 Lightyear Unverified Risk

ADR-016 Verification Required item 2 (Lightyear `Replicate` per-entity scoping, `UnreliableChannel` ChannelMode variants) remains open — ADR-008 checklist item 2 is also unresolved. This is inherited from the S1-05 spike findings. Not a new regression but a continuing risk before M3 Prism stories start.

---

## Phase 5b — GDD Revision Flags

No new GDD revision flags from engine compatibility analysis. Engine reality contradictions are the same as prior review (Lightyear unicast API unverified, `bevy_tweening` 0.18 crates.io availability unconfirmed for card-animations.md TR-CAN-001).

Existing flags still active from `/review-all-gdds R8` (2026-04-30):
- `round-state-machine.md`, `network-protocol.md`, `class-system.md`, `auction-system.md`, `hand-ui.md`, `keyword-system.md`, `objective-system.md` — all marked Needs Revision in systems-index.md

---

## Phase 6 — Architecture Document Coverage

`docs/architecture/architecture.md` (last updated 2026-04-29) is stale — it covers ADR-001..012 only. Sections that are now outdated:

| Section | Stale because |
|---|---|
| §1 Layer Map / Module Ownership | ADR-013..018 feature modules not reflected |
| §5 ADR Audit table | Covers ADR-001..012 only |
| §5 Traceability Matrix | 80 M1 TRs; 90 M2/M3 TRs not yet mapped |
| §6 Required ADRs | Auction/Combat/CA ADRs are now authored; Presentation ADRs still outstanding |

No orphaned systems detected — all 20 systems have GDDs. No GDD system lacks a corresponding crate module in the Layer Map.

---

## Phase 7 — Verdict

### CONCERNS (M1 PASS / Core M2/M3 CONCERNS / Presentation FAIL)

**M1: PASS.** 80/80 TRs covered; 12/12 ADRs Accepted; no conflicts; engine compat clean.

**Core M2/M3: CONCERNS.** 6 ADRs authored (good progress); 46/58 Core TRs covered (79%). Blocked from PASS by HC-1 (numbering conflict), HC-2 (hand storage), HC-3 (EconomyState gap), HC-4 (missing amendments to ADR-005/006/010). All ADRs remain Proposed — no stories auto-blocked today, but gate closes when stories are opened.

**Presentation M2/M3: FAIL.** 32/32 Presentation TRs are architectural gaps — no governing ADR for board-rendering, hand-ui, shop-auction-ui, hud, or card-animations. The `bevy_tweening` dependency (TR-CAN-001) has no version-pinning decision, and `BoardLayout` coordinate authority (TR-BR-002) is unresolved.

---

### Blocking Issues (resolve before Core M2/M3 ADRs can be Accepted)

| ID | Issue | Resolution |
|---|---|---|
| **B-1** | `adr-015-card-acquisition-shop-state.md` H1 says "ADR-014" — duplicate numbering | Edit line 1 to `# ADR-015: Card Acquisition Shop State Machine Architecture` |
| **B-2** | Hand storage: `PlayerHands` vs `PlayerSessionData.hand` vs `HandState` across ADR-014/015/016 | Adopt `PlayerHands` (ADR-015); amend ADR-014 + ADR-016 |
| **B-3** | `EconomyState` resource undefined; ADR-013 and ADR-015 assume it | Author ADR-019 (Economy System ADR) before accepting ADR-013/015 |
| **B-4** | ADR-018 depends on ADR-005 amendment (3 seed slots) + ADR-006 amendment (SimpleKeyword 20 variants) — neither landed | Land amendments before accepting ADR-018 |
| **B-5** | ADR-016 depends on ADR-010 Prism subscriber row — not in ADR-010 | Append row to ADR-010 Subscriber Contracts table |
| **B-6** | ADR-013 duplicate "Post-Cutoff APIs Used" row (template error line 37) | Delete line 37 |

### Required New ADRs (priority order)

| Priority | ADR | System | Milestone |
|---|---|---|---|
| 1 | **ADR-019 — Economy System Resource Architecture** | Economy (gold/mana/reserve ownership, EconomyState vs PlayerSessions) | M2 gate |
| 2 | **ADR-020 — Presentation Layer Architecture** | Board Rendering, Hand UI, Shop/Auction UI, HUD, Card Animations | M2 |
| 3 | **ADR-006 amendment** | Extend `SimpleKeyword` to 20 variants; Charge→Haste; adjacent serde | M3 gate |
| 4 | **ADR-005 amendment** | Add 3 RESOLUTION seed slots + RngEvent variants | M3 gate |

---

## Phase 8 — Handoff

**Immediate actions (top 3):**
1. Fix B-1 (3-minute edit — highest risk-to-fix ratio, blocks story tooling)
2. Author `/architecture-decision economy-system` → ADR-019 (unblocks ADR-013 and ADR-015 Acceptance)
3. Append Prism row to ADR-010 + delete ADR-013 duplicate table row (5-minute edits)

**Gate guidance**: When B-1 through B-6 are resolved and ADR-019 is Accepted, run `/gate-check pre-production` for a PASS verdict on Core M2. Presentation ADR-020 is required before M2 Presentation stories can open.

**Rerun trigger**: Re-run `/architecture-review` after ADR-019 is authored and the ADR-005/006 amendments land.

---

## Session Extract Placeholder

*(Updated by Phase 8 session-state update)*
