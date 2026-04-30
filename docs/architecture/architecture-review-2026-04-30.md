# Architecture Review — 2026-04-30

## Document Status

| Field | Value |
|---|---|
| **Date** | 2026-04-30 |
| **Engine** | Bevy 0.18 + Lightyear 0.26 |
| **GDDs Reviewed** | 22 system GDDs (9 M1 Approved, 8 M2 Approved/InReview/Designed, 4 M3 Designed) |
| **ADRs Reviewed** | 12 (ADR-001 through ADR-012, all Accepted) |
| **Verdict** | **CONCERNS — M1 PASS / M2+M3 FAIL (ADRs not yet authored)** |

---

## Phase 1 — Loaded

- **GDDs**: 22 (card-data-pool, game-config, server-rng, economy-system, board-lane-system, round-state-machine, network-protocol, game-session-system, objective-system, card-acquisition, auction-system, combat-resolution, board-rendering, hand-ui, shop-auction-ui, hud, keyword-system, prism-system, class-system, card-animations, plus master GDD + systems-index)
- **ADRs**: 12 (ADR-001 through ADR-012, all Accepted as of 2026-04-29)
- **Engine**: Bevy 0.18 + Lightyear 0.26 — HIGH risk; verified partially by S1-05 spike
- **Engine reference docs**: `docs/engine-reference/bevy/` — VERSION.md, breaking-changes.md, deprecated-apis.md, current-best-practices.md

No `docs/consistency-failures.md` found — skip reflexion log append.

---

## Phase 2+3 — Traceability Summary

| Tier | Requirements | Covered | Partial | Gaps |
|---|---|---|---|---|
| **M1** (9 GDDs, 12 ADRs) | 74 (claimed in architecture.md) | 74 ✅ | 0 | 0 |
| **M2 Feature** (auction, combat, card-acq) | ~33 | ~7 ⚠️ | ~6 | ~20 ❌ |
| **M2 Presentation** (board-render, hand-ui, shop-ui, hud) | ~63 | ~5 ⚠️ | ~30 | ~28 ❌ |
| **M3 Feature** (keyword, prism, class) | ~34 | ~9 ⚠️ | ~12 | ~13 ❌ |
| **M3 Presentation** (card-animations) | 13 | 0 | ~3 | ~10 ❌ |
| **Total** | **~217** | **~95** | **~51** | **~71** |

**M1 note**: Architecture.md Phase 5 claims 74/74 TRs covered across ADR-001..012. The per-system matrix is structurally correct — each TR range maps cleanly to the stated ADRs. However, **M1 GDDs use AC-IDs** (`RSM-1..38`, `NP-1..29`, `GSS-1..41`, `BL-1..33`, etc.) not TR-IDs. Architecture.md's `TR-RSM-NN` convention has no mirror in the GDD bodies. This is a traceability gap: the registry is currently an empty stub.

**M2/M3 note**: GDDs are now authored but governing ADRs have not yet been written. This was anticipated — architecture.md §6 ("Required ADRs — M2/M3") explicitly deferred them until GDDs were authored. That trigger is now reached.

---

## Phase 3 — Coverage Gaps (Top 10, foundational first)

| # | Suggested ADR | Covers | Impact |
|---|---|---|---|
| 1 | **ADR-013 — Auction System State Machine + S2CGoldBroadcast Schema Extension** | `auction-system.md` TRs 1–10 | Signature mechanic; IDLE/SELECTING/LIVE_BIDDING/RESOLVING enum; atomic release-then-reserve gold; `reserved_gold` field in `S2CGoldBroadcast` |
| 2 | **ADR-014 — Combat Resolution Sub-Step Scheduling + S2CResolutionEvent Schema** | `combat-resolution.md` TRs 1–2, 8 | 6 deterministic sub-steps + 5-step internal sequence; `S2CResolutionEvent::TaggedEvent` variant taxonomy; RESOLUTION safety timeout integration with RSM |
| 3 | **ADR-015 — Keyword Trigger Observer Architecture** | `keyword-system.md` TRs 1–3, `combat-resolution.md` TR-3 | DEATH/APPEARANCE/FINAL BLOW/COUNTERATTACK Bevy 0.17+ Observer chaining semantics; `UnitBoardState` keyword-state replication contract |
| 4 | **ADR-016 — Lightyear Per-Entity Replication Visibility (PLACEMENT Commitment Hiding)** | `board-rendering.md` TRs 8–9 | Reveal-tween fires only on `S2CPlacementReveal`-frame entities; verifies Lightyear 0.26 entity visibility scoping post-commit |
| 5 | **ADR-017 — C2SRequestSnapshot Recovery Message + Server Rate-Limit** | `board-rendering.md` TR-15 | Stuck-state recovery for RESOLUTION reveal timeout, sub_step OOR, S2CObjectiveIdentities timeout; currently undefined in network-protocol.md |
| 6 | **ADR-018 — bevy_tweening 0.18 Custom Lens Library + CardAnimationsPlugin Architecture** | `card-animations.md` TRs 1, 3, 7, 10; `board-rendering.md` TR-6; `shop-auction-ui.md` TR-4 | 5 custom Lens types; `#[derive(Message)]` domain-event indirection; bevy_tweening 0.18 crates.io availability (BLOCKING — PR #264 still draft per evidence doc) |
| 7 | **ADR-019 — Client UI Entity Pre-Pooling + Texture Atlas Sharing Strategy** | `hand-ui.md` TRs 1–3, `hud.md` TR-1, `board-rendering.md` TRs 3–4 | All UI entities pre-spawned at session start; `TextureAtlasLayout` sharing between Hand UI and Board Rendering |
| 8 | **ADR-020 — Class Token Entity Model** | `class-system.md` TR-4 | 7 token types (Mummy, Sinistro, Chacha Noir, Seed, Madoll, La Gonflable, La Sacrifiée); spawn paths, replication, Miranda-stolen `source_class` retention |
| 9 | **ADR-021 — Sang Méprise Mid-RESOLUTION Unicast Reveal + Reconnect Re-delivery** | `class-system.md` TR-6, `objective-system.md` | Extends ADR-001 unicast pattern; reconnect re-delivery via `S2CGameSnapshot.active_sang_meprise_identities` or explicit re-send |
| 10 | **ADR-022 — Card Acquisition Transactional Rollback (spend/refund pair)** | `card-acquisition.md` TR-7 | TOCTOU rollback: if `distribute()` fails after `spend_gold()`, atomic `refund_gold(player, cost)` — Economy API extension |

**Additional secondary gaps** (lower priority, after top-10): HUD system tie-break ordering (`S2CGoldBroadcast` before `S2CGoldUpdate`), Client `BoardRenderState` machine + phase-buffering, `BoardLayout` server resource + coordinate system, `S2CSessionPaused`/`S2CSessionResumed` messages for disconnect-grace UI, Persistent keyword state component model (SHIELD/STUN/SILENCE cross-round), RESOLUTION per-round snapshot resources (LEADER bonus, OUTNUMBERED count), Client persistent preferences storage (WASM localStorage), `ClientCardDataPlugin` asset pipeline.

---

## Phase 4 — Cross-ADR Conflict Detection

**Zero hard conflicts detected.** Architecture.md self-attests "12/12 pass. Zero conflicts" and independent re-check confirms it.

### Soft inconsistencies

🟡 **ADR-005 stale dependency reference**: `Depends On: ADR-007 (pending, after ADR-009) — Game Session lifecycle`. The actual Game Session lifecycle ADR is **ADR-012** (SessionReady Delivery), not ADR-007 (Placement Buffer). ADR-005 was authored before ADR-012 was numbered. Resolution: update ADR-005 §"Depends On" to reference ADR-012.

🟡 **ADR-009 architecture note vs ADR-010 event naming**: ADR-009 Key Interfaces list `#[derive(Message)] pub struct AuctionSettled`, `ResolutionComplete`, `OnResolutionEnd`, `StartAuction`, `AbortAuction`, `BeginResolution` etc. ADR-010 defines the canonical catalog with slightly different field names on some types (e.g. `AuctionSettled { winner, final_price, card_id }` in ADR-010 vs `AuctionSettled { winner: Option<PlayerId>, winning_bid: u32 }` in ADR-009 Key Interfaces). Not blocking — ADR-010 is authoritative; ADR-009 code samples are illustrative only.

### ADR Dependency Ordering (topologically sorted)

```
Foundation — no external deps:
  1. ADR-001  Objective Identity Unicast
  2. ADR-002  Client-Server Authority

Foundation — depends on Authority:
  3. ADR-003  Cargo Workspace Structure (→ ADR-002)

Foundation — depends on Workspace:
  4. ADR-004  Asset Loading Pipeline (→ ADR-003)
  5. ADR-008  Lightyear Channel Config (→ ADR-002, ADR-003)
  6. ADR-006  Card Data Schema + PlayerPool (→ ADR-003, ADR-004)
  7. ADR-005  Server-side RNG (→ ADR-003; lifecycle owner = ADR-012)

Core:
  8. ADR-009  RSM Phase State (→ ADR-002, ADR-008)
  9. ADR-010  RSM Event Bus (→ ADR-009, ADR-003, ADR-008)
 10. ADR-012  SessionReady Delivery (→ ADR-009, ADR-005)
 11. ADR-011  Reconnect Snapshot (→ ADR-001, ADR-002, ADR-008)

Feature:
 12. ADR-007  Placement Buffer (→ ADR-002, ADR-003, ADR-009)
```

No cycles. No ADR depends on a Proposed or missing ADR.

---

## Phase 5 — Engine Compatibility Audit

**ADRs with Engine Compatibility section**: 12/12

### 🔴 CRITICAL — ADR-004 Code Samples Invalid for Bevy 0.18

[adr-004-asset-loading-pipeline.md](adr-004-asset-loading-pipeline.md) contains two code samples using Bevy types that do not exist in 0.17+:

**Issue 1** (validate_and_promote, ~line 426):
```rust
mut exit: EventWriter<AppExit>,  // ❌ EventWriter does not exist in Bevy 0.17+
```
The TODO annotation was added (session audit 2026-04-29) but the invalid code was not replaced.

**Correct Bevy 0.18 pattern**: `AppExit` is dispatched via `MessageWriter<AppExit>` in 0.17+ (Message/Event split), or via `commands.trigger(AppExit::error())` if AppExit was converted to an Observer event. The `liv-bevy-018` skill `references/api_patterns.md` has the verified pattern. **This fix is applied below in Phase 5b.**

**Issue 2** (hot_reload_game_config, ~line 525):
```rust
mut events: EventReader<AssetEvent<GameConfig>>,  // ❌ EventReader does not exist in Bevy 0.17+
```
In Bevy 0.17+, asset change detection uses the Observer pattern: `app.observe(|_trigger: On<AssetEvent<GameConfig>>, ...| { ... })`. **This fix is applied below.**

### 🟡 MEDIUM — ADR-008 Lightyear Channel Syntax Unverified

S1-05 spike found 7 API differences from ADR-008 assumptions (channel definition syntax, `ChannelDirection` model, send/receive method names, `NetworkTarget` variant names, server send API, connection event naming). Resolutions are documented in `tests/evidence/lightyear-026-verification.md` and annotated in `docs/architecture/control-manifest.md`. ADR-008 code samples still show pre-spike syntax. Not blocking — the verification checklist in ADR-008 §Implementation Guidelines already flags each item as ⬜ until confirmed.

### 🟡 MEDIUM — ADR-011 Connection Event Semantics Unverified

`OnConnected`/`OnDisconnected` event types unverified for Lightyear 0.26 (may be Observer triggers with `Connect`/`Disconnect` + marker components `Connected`/`Disconnected`/`Connecting`). ADR-011 §"⚠️ API Verification Required" already documents this. S1-05 spike evidence covers the connection lifecycle findings.

### 🟡 MEDIUM — ADR-012 Observer Handler Signature

ADR-012 uses `Trigger<SessionReady>` as handler parameter; `liv-bevy-018` skill uses `On<E>` as the trigger type. ADR-012 §"⚠️ API Verification Required" item (3) explicitly flags this. No action needed until implementation.

### 🟢 CLEAN — ADR-009, ADR-010

`MessageWriter::write()` / `MessageReader::read()` + `app.add_message::<T>()` + `#[derive(Message)]` pattern is consistent with verified Bevy 0.17/0.18 Message/Event split. `EventWriter`/`EventReader` do not appear.

### 🟢 CLEAN — ADR-006, ADR-005

No Bevy API churn risk. Pure Rust serde/rand/HashMap patterns.

### 🟢 CLEAN — Control Manifest

`docs/architecture/control-manifest.md` correctly enforces all deprecated-API prohibitions (`SpriteBundle`, `NodeBundle`, `set_parent`, `despawn_recursive`, `EventWriter::send`, `UiImage::new`, `EventReader`, `EventWriter` for game events). Presentation Layer Rules section is current.

### Deprecated API References in ADRs

Only the two ADR-004 samples above use removed APIs. All other ADRs are clean per the `docs/engine-reference/bevy/deprecated-apis.md` checklist.

### Engine Specialist Findings (liv-bevy-018 pattern, applied in review)

The `liv-bevy-018` skill enforces 0.18 patterns on every `.rs` file. The skill's mandatory activation rule (documented in `docs/engine-reference/bevy/VERSION.md`) is the primary guard against API drift in implementation. ADR-level code samples that are illustrative only (not compiled) fall outside the skill's compile-time enforcement and must be manually verified.

---

## Phase 5b — GDD Revision Flags (Architecture → Design Feedback)

**No GDD revision flags required.** GDDs operate at the design layer and do not make specific Bevy/Lightyear API claims that verified engine reality contradicts.

Two tracked risks:
- `card-animations.md` BLOCKING dependency on `bevy_tweening 0.18` crates.io availability (TR-anim-010). If no 0.18-compatible release ships, the Custom Lens architecture must be redesigned. Track: re-check crates.io before ADR-018 is authored.
- `network-protocol.md` OQ-3 (Reliable channel cross-message-type FIFO ordering). ADR-008 OQ-D invariant verifies this assumption (checklist item 10). Mark OQ-3 as resolved-pending-verification in the NP GDD.

---

## Phase 6 — Architecture Document Coverage

`docs/architecture/architecture.md` (authored 2026-04-29) explicitly scopes to 9 M1 GDDs + 12 M1 ADRs. As of 2026-04-30, the following sections are stale:

| Section | Stale because |
|---|---|
| §1 Layer Map / Module Ownership | M2 Feature modules (Auction, Card Acquisition, Combat) are placeholders only |
| §2 M2/M3 Feature modules table | "will be fully specified in their governing ADRs" — now 8 GDDs ready |
| §5 ADR Audit table | Covers ADR-001..012 only; ADR-013..022 will extend it |
| §5 Traceability Matrix | 74 M1 TRs only; ~143 M2/M3 TRs not yet mapped |
| §6 Required ADRs — M2 | Auction, Combat, Card Acquisition ADRs unwritten |
| §6 Required ADRs — M3 | Keyword, Prism, Class ADRs unwritten |

Action: regenerate architecture.md after each batch of M2 ADRs lands.

**Orphaned systems** (in architecture.md but no GDD authored yet): none — all 20 systems in systems-index.md have corresponding GDDs.

---

## Phase 7 — Verdict

### CONCERNS

**M1: PASS.** 74/74 TRs covered; 12/12 ADRs accepted; no conflicts; clean dependency DAG; engine compatibility documented with S1-05 spike resolutions.

**M2/M3: FAIL for implementation readiness.** ~71 architectural gaps flagged; ~10 governing ADRs needed. This was anticipated — architecture.md §6 deferred M2/M3 ADRs until GDDs were authored. That trigger is now reached for 11 GDDs.

### Blocking issues (must resolve before `/gate-check pre-production`)

| # | Issue | Resolution |
|---|---|---|
| B-1 | ADR-004 invalid Bevy code samples (`EventWriter<AppExit>`, `EventReader<AssetEvent<T>>`) | **Fixed in this review** — see edit to ADR-004 |
| B-2 | ADR-005 stale dependency reference (ADR-007 → ADR-012) | **Fixed in this review** — see edit to ADR-005 |
| B-3 | `tr-registry.yaml` is an empty stub — 74 M1 TRs not materialized | **Populated in this review** |
| B-4 | M2 governing ADRs not authored (Auction, Combat, Card Acquisition at minimum) | Run `/architecture-decision [system]` for each |

### Required new ADRs (priority order)

| Priority | ADR | Milestone |
|---|---|---|
| 1 | ADR-013 — Auction System State Machine + S2CGoldBroadcast Schema | M2 |
| 2 | ADR-014 — Combat Resolution Sub-Step Scheduling + S2CResolutionEvent Schema | M2 |
| 3 | ADR-015 — Keyword Trigger Observer Architecture | M3 (also needed for M2 Combat triggers) |
| 4 | ADR-016 — Lightyear Per-Entity Replication Visibility (PLACEMENT) | M2 |
| 5 | ADR-017 — C2SRequestSnapshot Recovery + Server Rate-Limit | M2 |
| 6 | ADR-018 — bevy_tweening 0.18 Custom Lens Library | M2/M3 |
| 7 | ADR-019 — Client UI Pre-Pooling + Atlas Sharing | M2 |
| 8 | ADR-020 — Class Token Entity Model | M3 |
| 9 | ADR-021 — Sang Méprise Unicast Reveal + Reconnect Re-delivery | M3 |
| 10 | ADR-022 — Card Acquisition Transactional Rollback | M2 |

---

## Handoff

**Immediate actions (top 3):**
1. Run `/architecture-decision auction-system` — write ADR-013 (highest-impact gap; signature mechanic)
2. Run `/architecture-decision combat-resolution` — write ADR-014 (blocks all M2 Combat stories)
3. Run `/architecture-decision keyword-system` — write ADR-015 (needed for M2 Combat triggers even before M3)

**Gate guidance**: When blocking issues B-3 and B-4 (top-3 ADRs at minimum) are resolved, run `/gate-check pre-production` to evaluate advancement.

**Rerun trigger**: Re-run `/architecture-review` after each batch of ADRs is written to confirm coverage improves.
