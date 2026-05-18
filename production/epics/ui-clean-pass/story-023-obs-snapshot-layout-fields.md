# Story 023: S18-OBS-SNAPSHOT-LAYOUT-FIELDS-001 -- QA Snapshot Layout-Debug Field Enrichment (Q-01..Q-10)

> **Epic**: UI Clean-Pass
> **Story ID**: S18-OBS-SNAPSHOT-LAYOUT-FIELDS-001
> **Status**: Draft -- future Sprint 18 candidate; NOT activated by this authoring run
> **Layer**: Presentation -- QA snapshot tooling (`client/src/presentation/qa_snapshot.rs` only)
> **Type**: Tech Debt -- observability extension
> **Sprint**: Future Sprint 18 candidate per PROMPT 1180 §6 Lane D.
> **Authored**: 2026-05-18 by PROMPT 1189
> **Authoring source-of-truth**: `origin/main@efb698e`
> **Estimated effort**: ~0.4d
> **Source audit**: PROMPT 1180 §4 (Q-01..Q-10), §6 Lane D (PROMPT 1193 candidate)
> **Active impl PROMPT**: PROMPT 1186. If 1186 lands first, this story may close via `/story-done`.

---

## Status / No-Claim Banner

Future Sprint 18 candidate. **No sprint activated.** No claim on release readiness, `QA-COND-*`, `PAW-TD-*-a`, 24 PROMPT 1022 findings, or any audit finding outside Lane D.

## Problem Class / Prevention Target

**Defect class** (§4): QA snapshot cannot prove failure modes from JSON alone. Ten layout-debug fields (Q-01..Q-10) would let next audit verify RC-1..RC-5 from JSON without source dives.

**Prevention target**: emit all ten Q-* fields. Where ECS data missing, emit `null` and document — do NOT invent values.

## 1180 Lane Coverage

Owns Lane D:

> | **D — Snapshot field enrichment (Q-01..Q-10)** | `client/src/presentation/qa_snapshot.rs` only | `tests/integration/qa_snapshot/layout_field_coverage_test.rs` (NEW) | **P1** |

Q-* enumeration:

| # | Field | Catches |
|---|---|---|
| Q-01 | `viewport.width_px / height_px` | Single normalisation. |
| Q-02 | `surface.<name>.bounds = { x, y, w, h }` per `*_root` | RC-1 / RC-2 / RC-5. |
| Q-03 | `surface.<name>.overflow_clipped: bool` | RC-2. |
| Q-04 | `surface.<name>.children_count: u32` | Dynamic-panel drift. |
| Q-05 | `text.<marker>.fits + clipped_chars` | UI-1129-05 / -09. |
| Q-06 | `image.<marker>.aspect_ratio_src / _rendered` | F-02. |
| Q-07 | `button.<marker>.affordance_state` | RC-4. |
| Q-08 | `panel.<name>.z_layer_resolved` | RC-1 same-z. |
| Q-09 | `placement_action_panel.collisions: [<surface>...]` | F-03. |
| Q-10 | `shop_panel.bottom_edge_y / hand_bar.top_edge_y` | S-01. |

## Context

- `client/src/presentation/qa_snapshot.rs` — `UiCountQueries`, JSON emit.
- Story 019 (Draft S17) extends marker split + visibility filter + session ID prefix; this row layers Q-* on top.

**GDD / ADR**: no change.

**Engine / skills**: Bevy 0.18; `liv-bevy-018`; `ComputedNode` / `GlobalTransform` / `Visibility` / `ImageNode` / `TextLayout` canonical.

### Control Manifest Rules

- Required: emit all ten Q-* fields. Schema additive; `CCGS_QA_SNAPSHOT=1` env-var contract preserved.
- Required: missing-data fallback `null` + documented gap.
- Forbidden: editing UI surfaces (lobby, hud, hand, shop_auction, settings).
- Forbidden: protocol / server / shared change.

## Story Classification

**Integration**.

## Dependencies and Parallelism

| Sibling | Parallel-safe? | Notes |
|---|---|---|
| Stories 020 / 021 / 022 / 024 / 025 / 026 / 027 | YES | Disjoint. |
| Story 019 (marker split) | PARTIAL | Prefer 019 first. |
| Active PROMPTs 1178 / 1182 / 1183 / 1187 / 1188 | YES | Don't own this file for layout fields. |
| Active PROMPT 1186 | DUPLICATE | Impl worker; may land first. |

## Acceptance Criteria

- [ ] AC1..AC10 -- Each Q-01..Q-10 field emitted per the enumeration above; `null` allowed where ECS data missing.
- [ ] AC11 -- `layout_field_coverage_test.rs` (NEW) spawns minimal scene per marker family, drives ≥3 frames, asserts presence of every Q-* field (null-emission path exercised where applicable).
- [ ] AC12 -- `CCGS_QA_SNAPSHOT=1` contract preserved.
- [ ] AC13 -- No invented values; evidence note lists every `null`-emitted field + missing-query file:line.
- [ ] AC14 -- `liv-bevy-018` activated.
- [ ] AC15 -- Cargo resource policy applied.
- [ ] AC16 -- No accept-risk closure; 24 PROMPT 1022 findings preserved.
- [ ] AC17 -- Sprint disposition preserved.
- [ ] AC18 -- Worker branch scope contained; slug `work/s18-obs-snapshot-layout-fields`.

## Implementation Notes

### Owned files

| Path | Expected change |
|------|-----------------|
| `client/src/presentation/qa_snapshot.rs` | Emit Q-01..Q-10 + helper queries. |
| `tests/integration/qa_snapshot/layout_field_coverage_test.rs` (NEW) | AC11. |
| `production/qa/evidence/sprint-18-snapshot-layout-fields/evidence.md` (NEW) | AC13 limitation notes. |

### Forbidden files

- All UI surfaces, server, shared.
- Sprint / state / QA / gate-check files (except AC13 evidence).
- ADRs.

## Worker Contract

1. Worktree slug `work/s18-obs-snapshot-layout-fields`.
2. Read story + PROMPT 1180 §4 + Lane D.
3. Re-verify `qa_snapshot.rs` shape + story 019 marker list if landed.
4. Activate `liv-bevy-018`.
5. Cargo resource policy env vars.
6. Targeted tests only.
7. Push worker branch only.
