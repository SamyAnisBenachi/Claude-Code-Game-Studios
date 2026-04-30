# Architecture Review — 2026-04-30 (Run 3 — ADR-019/020 Wave + Promotions)

## Document Status

| Field | Value |
|---|---|
| **Date** | 2026-04-30 |
| **Engine** | Bevy 0.18 + Lightyear 0.26 |
| **Scope** | Incremental review — ADR-019 Accepted, ADR-020 Proposed; ADR-013/014/015/016/017 promoted Proposed → Accepted since Run 2 |
| **Prior Reviews** | Run 1 (`architecture-review-2026-04-30.md`); Run 2 (`architecture-review-2026-04-30b.md`) |
| **GDDs Reviewed** | 20 system GDDs (M1: 9, M2: 7, M3: 4) |
| **ADRs Reviewed** | 20 (ADR-001..020) — 17 Accepted, 2 Proposed (018, 020), 0 Superseded |
| **Verdict** | **CONCERNS** — M1 PASS, Core M2/M3 PASS pending stale-ref cleanup, Presentation FAIL |

---

## Phase 1 — Loaded

- **GDDs**: 20 (full list in `design/gdd/systems-index.md`)
- **ADRs**: 20 — ADR-001..017 + ADR-019 Accepted; ADR-018 + ADR-020 Proposed
- **TR Registry**: `docs/architecture/tr-registry.yaml` — 80 M1 + 10 TR-AUC; M2/M3 remainder appended in Phase 8 of this report
- **Engine**: Bevy 0.18 + Lightyear 0.26 — HIGH risk; one new BLOCKING UNVERIFIED item (`ReplicateTo`)
- **No `docs/consistency-failures.md`** — reflexion log append skipped

### Status Changes Since Run 2 (2026-04-30b)

| ADR | Run 2 status | Now | Notes |
|---|---|---|---|
| ADR-013 Auction System | Proposed | **Accepted** | Duplicate "Post-Cutoff APIs Used" row deleted (B-6 ✅) |
| ADR-014 Class System | Proposed | **Accepted** | H1 numbering correct; hand-storage prose stale (HC-1) |
| ADR-015 Card Acquisition | Proposed | **Accepted** | H1 corrected to ADR-015 (B-1 ✅); `EconomyState` references stale (SI-1) |
| ADR-016 Prism System | Proposed | **Accepted** | `HandState` everywhere — conflicts with ADR-015's `PlayerHands` (HC-1) |
| ADR-017 Combat Resolution | Proposed | **Accepted** | Line 76 still references `ResMut<EconomyState>` (SI-1) |
| ADR-018 Keyword System | Proposed | Proposed | Held — ADR-005 + ADR-006 amendments still missing (B-4 unchanged) |
| **ADR-019** Economy Resource Architecture | — | **Accepted (NEW)** | Resolves prior B-3 design decision; defines `PlayerEconomies`, `InterestSnapshots`, `PendingResolutionComplete` |
| **ADR-020** Board/Lane State Architecture | — | **Proposed (NEW)** | Lightyear `ReplicateTo` API marked BLOCKING UNVERIFIED |
| ADR-010 RSM Event Bus | Accepted | Accepted | Prism subscriber row added (B-5 ✅, line 387) |

---

## Phase 2 + 3 — Traceability Matrix

### Coverage Summary (Run 3)

| Tier | Requirements | ✅ Covered | ⚠️ Partial | ❌ Gap | Coverage |
|---|---|---|---|---|---|
| M1 (9 GDDs) | 80 | 80 | 0 | 0 | **100%** |
| M2 Auction (TR-AUC) | 10 | 10 | 0 | 0 | **100%** |
| M2 Combat Resolution (TR-CR) | 12 | 8 | 4 | 0 | 67% |
| M2 Card Acquisition (TR-CA) | 10 | 10 | 0 | 0 | **100%** |
| M3 Class System (TR-CS) | 12 | 12 | 0 | 0 | **100%** |
| M3 Prism System (TR-PRI) | 8 | 8 | 0 | 0 | **100%** |
| M3 Keyword System (TR-KW) | 12 | 0 | 12 | 0 | 0% (ADR-018 Proposed) |
| M3 Card Animations (TR-CAN) | 7 | 0 | 1 | 6 | 0% (no ADR) |
| Presentation: Board Rendering (TR-BR) | 7 | 0 | 3 | 4 | 0% |
| Presentation: Hand UI (TR-HU) | 8 | 5 | 2 | 1 | 62% |
| Presentation: Shop/Auction UI (TR-SAU) | 6 | 4 | 1 | 1 | 67% |
| Presentation: HUD (TR-HUD) | 10 | 3 | 4 | 3 | 30% |
| **Total** | **182** | **140** | **27** | **15** | **77%** |

**Run 2 → Run 3 deltas**: ADR-019 closed 8 ECO-related partial gaps. ADR-020 Proposed brings 9 BLS/BR coverage entries that depend on the Lightyear API verification. M3 Class + Prism reached 100% with ADR-014/016 Accepted. Card Animations remains zero coverage.

### Per-System TR Highlights (M2/M3 — newly registered in Phase 8)

The full TR list per system is appended to `docs/architecture/tr-registry.yaml`. Below are the key gap rows.

#### Combat Resolution — partial (4)
- TR-CR-006 COUNTERATTACK after-damage timing (melee only) → ADR-018 Proposed
- TR-CR-007 FINAL BLOW kill sub-step routing → ADR-018 Proposed
- TR-CR-010 INJURED activates at sub-step boundary → ADR-018 Proposed
- TR-CR-011 OUTNUMBERED per-player global count → ADR-018 Proposed (no explicit ADR-018 coverage)

#### Card Animations — gap (6)
- TR-CAN-001..004 Decoration Test, animation budgets, input gating, parallelism → no ADR
- TR-CAN-006 Sub-step pause gates → no ADR
- TR-CAN-007 `bevy_tweening` integration + custom lenses → no version-pinning ADR

#### HUD — gap (3)
- TR-HUD-004 Objective scoreboard state machine (Hidden→Real→Fake→Destroyed) → ADR-020 Proposed (presentation portion)
- TR-HUD-008 Objective dot ALIVE→DESTROYED with no animation → no presentation ADR
- TR-HUD-009 FROZEN mode on GAME_OVER (no incremental updates) → no presentation ADR
- TR-HUD-010 Numeric tween 300ms cap → no presentation ADR

#### Board Rendering — gap (4)
- TR-BR-001 RESOLUTION sub-step visual separation + pause gates → ADR-020 Proposed (server-side only)
- TR-BR-003 Unit position replication; cell occupancy tracking → ADR-020 Proposed (server-side only; client mirror missing)
- TR-BR-005 `BoardLayout` cell-to-world resource → no client-side ADR
- TR-BR-006 OUTNUMBERED lane indicator (legacy per-lane vs new per-unit) → ADR-018 reconciliation pending

---

## Phase 4 — Cross-ADR Conflicts

### Prior Blockers — Resolution Status

| Prior | Issue | Status |
|---|---|---|
| B-1 | ADR-015 H1 says "ADR-014" | ✅ FIXED |
| B-5 | ADR-010 missing Prism subscriber row | ✅ FIXED (line 387) |
| B-6 | ADR-013 duplicate "Post-Cutoff APIs Used" row | ✅ FIXED |
| B-2 | Hand storage: PlayerHands vs PlayerSessionData.hand vs HandState | ❌ **STILL OPEN — ESCALATED to CRITICAL** |
| B-3 | EconomyState resource undefined | ⚠️ PARTIAL — ADR-019 Accepted, but stale `EconomyState` text remains in 4 ADRs |
| B-4 | ADR-005/006 amendments for ADR-018 | ❌ STILL OPEN |

### 🔴 HARD CONFLICT — HC-1: Hand Storage Triple-Named (escalated from B-2)

All three ADRs are now **Accepted** — three incompatible implementations are now equally authoritative.

| ADR | Status | Hand storage claim |
|---|---|---|
| ADR-014 Class System | Accepted | Lines 70, 87 — `hand: Vec<CardId>` is a planned field in `PlayerSessionData` (inside `PlayerSessions` resource) |
| ADR-015 Card Acquisition | Accepted | Lines 250, 254, 299 — `PlayerHands` is the canonical Resource (concrete struct + impl) |
| ADR-016 Prism System | Accepted | Lines 45, 52, 91, 97, 109, 118, 192, 207, 327 — `HandState` resource (distinct third name) |

**Impact**: Stories opened against any of the three ADRs would write incompatible code. Foundational risk — the next epic (Card Acquisition or Prism) will codify the wrong name.

**Resolution (recommended — Option 1)**: Adopt `PlayerHands` (ADR-015 has the only concrete struct + impl). Required edits:
- ADR-014 lines 70, 85–87, 437, 450 — remove the `hand: Vec<CardId>` promise from `PlayerSessionData`; cross-reference ADR-015 ownership
- ADR-016 — rename `HandState` → `PlayerHands` (~10 occurrences), update `hand_push(&mut PlayerHands, …)` signature

### 🔴 HARD CONFLICT — HC-2: ADR-014 PlayerSessions Extension Plan Now Stale

ADR-014 line 70 promises: *"future Economy ADRs extend `PlayerSessionData` with `gold`, `current_mana`, `reserve`, and `hand` fields"*.

ADR-019 (Accepted) chose **Alternative 2** — separate `PlayerEconomies` resource. ADR-014's risk row line 450 even predicted exactly this risk (without a mitigation), and the risk has now materialized.

**Impact**: Future readers of ADR-014 will assume `PlayerSessionData` accumulates fields it never will. Code reviews will lose the cross-reference.

**Resolution**: Edit ADR-014 lines 70, 85–87, 437, 450:
- Remove the "Economy/Card Acq fields" promise from `PlayerSessionData`
- Reference ADR-019 (`PlayerEconomies` is the canonical owner)
- Reference ADR-015 (`PlayerHands` is the canonical owner)
- Update the line 450 risk row to "Resolved by ADR-019" with disposition note

### 🔴 HARD CONFLICT — HC-3: ADR-020 Lightyear `ReplicateTo` UNVERIFIED (BLOCKING)

ADR-020 Verification Required item 1 (line 21): *"`ReplicateTo(NetworkTarget::All)` as the Lightyear 0.26 API for entity replication scope. **No project-held engine-reference confirms this name**. The correct component may be named differently in the actual 0.26.0 release."* Marked **BLOCKING — UNVERIFIED**.

**Impact**: ADR-020 cannot move from Proposed → Accepted. Board/Lane state stories are blocked. Combat Resolution implementation blocked because `resolve_combat` cannot spawn entities into the replication group with confidence.

**Resolution**:
1. Verify against Lightyear 0.26 release notes: `https://github.com/cBournhonesque/lightyear/releases` tag v0.26.0
2. Update ADR-020 Verification Required item 1 with confirmed API name
3. Append the verified pattern to `docs/engine-reference/bevy/current-best-practices.md`
4. Promote ADR-020 to Accepted

### 🟡 SOFT — SI-1: Stale `EconomyState` References After ADR-019 Adoption (carry-over from B-3)

ADR-019 canonicalized the resource as `PlayerEconomies`. Stale `EconomyState` references in four Accepted ADRs:

| ADR | Lines | Severity | Context |
|---|---|---|---|
| ADR-013 | 118, 332, 359, 365, 369 | Low | Prose / Alternatives Considered / Risk text |
| ADR-014 | 450 | Low | Risk row text predicting an outcome that already happened |
| **ADR-015** | **90, 164, 300, 419** | **High** | Decision text + diagram + system signature + dependency list — **load-bearing** |
| ADR-017 | 76 | Medium | Process narrative: "gold updates via `ResMut<EconomyState>`" |

**Resolution priority**: ADR-015 line 300 is highest because it is a function signature that would compile-fail in code review. Ripple replace `EconomyState` → `PlayerEconomies` in all four ADRs.

### 🟡 SOFT — SI-2: ADR-018 Pre-Implementation Amendments Still Pending (B-4 unchanged)

ADR-018 line 524–528 documents two required pre-implementation gates:

| Gate | Required In | Status |
|---|---|---|
| Add 3 RESOLUTION seed slots: `range_equidistant_select`, `teleport_random_dest`, `strich_change_lane_select` | ADR-005 | ❌ MISSING — 0 grep matches |
| Extend `SimpleKeyword` to 20 variants; rename `Charge`→`Haste`; switch to adjacent serde tag | ADR-006 | ❌ MISSING — ADR-006 still has 7-variant enum with `Charge` |

ADR-018 is correctly held in Proposed status until both amendments land.

### 🟢 RESOLVED — Prism subscriber row in ADR-010 (B-5 fixed)

ADR-010 line 387 now contains: *"`ResolutionPhaseEntered { round }` | Prism System | `resolve_prism_draws()` — for each prism cell traversed this round, draw `prism_strike` card into the traversing player's hand (overflow → `prism_reserve`); emit `S2CCardAcquired` unicast per draw; emit `S2CPrismRespawned` after repositioning; emit `S2CPrismRewardDropped` when hand is full and reserve is at cap. Per ADR-016. **[M3 — not yet implemented]** | M3"*. ✅

### ADR Dependency Order (topologically sorted; no cycles)

```
Foundation (Accepted):
  002 Client/Server Authority
  001 Objective Identity Unicast      → 002
  003 Cargo Workspace Structure       → 002

Infrastructure (Accepted):
  004 Asset Loading Pipeline          → 003
  008 Lightyear Channel Config        → 002, 003
  006 Card Data Schema + PlayerPool   → 003, 004
  005 Server-Side RNG                 → 003 (lifecycle: 012)

Core Phase Orchestration (Accepted):
  009 RSM Phase State                 → 002, 008
  010 RSM Event Bus                   → 009, 003, 008
  012 SessionReady Delivery           → 009, 005
  011 Reconnect Snapshot              → 001, 002, 008
  007 Placement Buffer                → 002, 003, 009

M2/M3 Wave 1 (Accepted):
  013 Auction State Machine           → 009, 010, 002, 008
  014 Class System                    → 002, 003, 005, 006, 009, 010, 012
  017 Combat Resolution               → 002, 009, 010, 005
  019 Economy Resource Architecture   → 002, 009, 010, 013, 017

M2/M3 Wave 2 (Accepted):
  015 Card Acquisition / Shop State   → 009, 010, 013, 005, 006, 008
  016 Prism System                    → 005, 010, 008, 006

Pending (Proposed):
  018 Keyword System                  → 002, 005⚠, 006⚠, 009, 010
  020 Board/Lane State Architecture   → 002, 003, 007, 009, 017, 018, Lightyear‑0.26⚠

Missing (no ADR):
  021 Presentation Layer (board rendering · hand UI · shop UI · HUD · card animations)
```

No cycles detected. ADR-018 cannot be Accepted until ADR-005 + ADR-006 amendments land. ADR-020 cannot be Accepted until Lightyear `ReplicateTo` is verified.

---

## Phase 5 — Engine Compatibility Audit

**ADRs with Engine Compatibility section**: 20/20 ✅

### New ADRs Engine Audit

| ADR | Verdict | Key finding |
|---|---|---|
| ADR-019 Economy Resource Architecture | 🟢 CLEAN | Correctly forbids `EventWriter`/`EventReader`; `PendingResolutionComplete` flag-buffer pattern documented as exclusive-system → MessageWriter bridge; engine specialist validated 2026-04-30 |
| ADR-020 Board/Lane State Architecture | 🟡 BLOCKING | Required Components API (no Bundles) ✅; `world.query()` in exclusive system confirmed ✅; **Lightyear `ReplicateTo(NetworkTarget::All)` UNVERIFIED — must verify before Accepted** |

### Existing ADRs (013–018) — No Regressions

All Run 2 audit verdicts hold. Promotions from Proposed → Accepted did not introduce new engine concerns. The MessageWriter (Bevy buffered) vs MessageReceiver/MessageSender (Lightyear network) split is consistent across all 20 ADRs.

### Deprecated API Check

Grep of all 20 ADRs for `EventWriter`, `EventReader`, `SpriteBundle`, `NodeBundle`, `Camera2dBundle`, `set_parent`, `despawn_recursive`, `UiImage`: **all matches are in negative/forbidden context** (e.g. "Post-Cutoff APIs NOT Used"). Zero positive references. ✅

### Engine Specialist Consultation

Skipped this round — all engine items either inherited from Run 2 audit (validated then) or covered by ADR-019's documented engine-specialist validation note (line 21). The single new BLOCKING item (ADR-020 Lightyear API) requires WebFetch verification, not engine-specialist prose review.

---

## Phase 5b — GDD Revision Flags

No new revision flags emerge from this incremental review.

Existing flags from `/review-all-gdds R8` and R9 remain active in `systems-index.md`:
- network-protocol.md, card-animations.md, class-system.md, round-state-machine.md, hand-ui.md, shop-auction-ui.md, keyword-system.md — all marked "Needs Revision"

These are tracked separately by the design pipeline (`/review-all-gdds`), not architecture-driven.

---

## Phase 6 — Architecture Document Coverage

Both `docs/architecture/architecture.md` and `docs/architecture/control-manifest.md` are **stale** with respect to ADR-013..020.

| Document | Stated | Actual | Action |
|---|---|---|---|
| architecture.md `Last Updated` | 2026-04-29 | 2026-04-30 | Refresh after HC-1/HC-2/HC-3 land |
| architecture.md `ADRs Referenced` | ADR-001..012 | ADR-001..020 | Add Layer Map / module ownership entries for Auction/Combat/CA/Class/Prism/Keyword/Board/Economy |
| architecture.md `TR Coverage` | 74/74 M1 | 90/90 (M1 + Auction) covered + 71 M2/M3 newly registered | Refresh |
| control-manifest.md `ADRs Covered` | ADR-001..012 | should be ADR-001..017 + ADR-019 | Regenerate via `/create-control-manifest` |
| control-manifest.md `ADRs Pending` | ADR-013..018 (all Proposed) | only ADR-018 + ADR-020 Proposed | Regenerate |

Stories embedding the current `Manifest Version: 2026-04-30` will inherit guardrails missing all of ADR-013..017, ADR-019, ADR-020. Rerun `/create-control-manifest` after blocking issues land.

No orphaned systems detected — all 20 systems have GDDs. All Feature/Core/Foundation modules have at least one Accepted ADR (Card Animations is the sole Feature/Presentation gap).

---

## Phase 7 — Verdict

### **CONCERNS** — improving from Run 2

| Tier | Verdict | Rationale |
|---|---|---|
| **M1 Foundation+Core** | **PASS** | 80/80 TRs covered; 12 ADRs Accepted; engine clean (unchanged from Run 2) |
| **M2 Auction + Card Acquisition + Combat** | **PASS** with stale-ref fixes | All ADRs Accepted; HC-1, HC-2, SI-1 must clean up; story tooling will inherit broken type names without these |
| **M2 Economy** | **PASS** | ADR-019 Accepted with full GDD coverage and engine-specialist validation |
| **M3 Class System + Prism** | **PASS** with HC-1 fix | ADR-014/016 Accepted; both reference incorrect hand-storage type |
| **M3 Keyword System** | **CONCERNS** | ADR-018 correctly held Proposed; 2 amendments missing; 12 TRs partial |
| **M3 Card Animations** | **FAIL** | Zero ADR coverage; bevy_tweening pin missing; no AnimQueue/Z-order/budget rules |
| **M2/M3 Presentation (Board Rendering / Hand UI / Shop UI / HUD)** | **CONCERNS → FAIL** | ADR-020 covers Board state but not BoardLayout client-side, Z-order, HUD persistence; no presentation-layer ADR |
| **Board/Lane State (server)** | **CONCERNS** | ADR-020 Proposed; Lightyear `ReplicateTo` UNVERIFIED is BLOCKING |

### Blocking Issues (must resolve before next gate)

| ID | Issue | Severity | Resolution |
|---|---|---|---|
| **B-1'** (=HC-1) | Hand storage triple-named — three Accepted ADRs with conflicting names | **CRITICAL** | Adopt `PlayerHands` (ADR-015); edit ADR-014 + ADR-016 |
| **B-2'** (=HC-2) | ADR-014 PlayerSessionData economy-extension plan superseded by ADR-019 | HIGH | Edit ADR-014 lines 70, 85–87, 437, 450 — cross-reference ADR-019 ownership |
| **B-3'** (=SI-1) | Stale `EconomyState` references in ADR-013/014/015/017; ADR-015 line 300 is load-bearing | HIGH | Ripple replace `EconomyState` → `PlayerEconomies` |
| **B-4'** (=HC-3) | ADR-020 BLOCKING UNVERIFIED on Lightyear `ReplicateTo` | HIGH | WebFetch Lightyear 0.26 release; update ADR-020 + bevy module doc |
| **B-5'** (=B-4) | ADR-005 + ADR-006 amendments for ADR-018 still missing | HIGH | Land both amendments before promoting ADR-018 |
| **B-6'** | Presentation Layer ADR (planned ADR-021) does not exist; Card Animations fully uncovered | MEDIUM | Author `/architecture-decision presentation-layer` |
| **B-7'** | architecture.md + control-manifest.md stale | MEDIUM | Refresh after B-1' through B-3' land |

### Required New ADRs (priority order)

| Priority | ADR | System | Purpose |
|---|---|---|---|
| 1 | ADR-005 amendment | RNG seed slots — required by ADR-018 | Add 3 RESOLUTION seed slots + RngEvent variants |
| 2 | ADR-006 amendment | Card data schema — required by ADR-018 | Extend `SimpleKeyword` to 20 variants; rename Charge→Haste; adjacent serde tag |
| 3 | **ADR-021 Presentation Layer** | Board Rendering, Hand UI, Shop UI, HUD, Card Animations | Z-order constants, `BoardLayout` client resource, AnimQueue pattern, `bevy_tweening` 0.18 version pin, HUD persistence sequence |

---

## Phase 8 — Handoff

### Immediate actions (top 3)

1. **Fix B-1' (HC-1)** — 30-minute multi-edit on ADR-014 + ADR-016 to adopt `PlayerHands`. Highest leverage: unblocks consistent story tooling for all of M2/M3.
2. **Fix B-3' (SI-1)** — ripple-replace `EconomyState` → `PlayerEconomies` in 4 ADRs (ADR-015 line 300 is the only load-bearing site).
3. **WebFetch Lightyear 0.26 release notes** — verify `ReplicateTo` API name, land in ADR-020 Verification Required item 1, promote ADR-020 to Accepted.

### Gate guidance

When B-1' through B-5' resolve and ADR-021 is Accepted, run `/gate-check pre-production` for a PASS verdict. Card Animations will remain a flagged risk until either ADR-021 covers it or a dedicated ADR is authored.

### Rerun trigger

Re-run `/architecture-review` after:
- ADR-014 + ADR-016 hand-storage edits land (HC-1 fix)
- ADR-005 + ADR-006 amendments land (B-5' fix)
- ADR-020 Lightyear API verified and promoted to Accepted

---

## Session Extract — appended to `production/session-state/active.md`

(See append in same commit.)
