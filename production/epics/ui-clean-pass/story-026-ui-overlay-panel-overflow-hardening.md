# Story 026: S18-UI-OVERLAY-PANEL-OVERFLOW-HARDENING-001 -- Photosensitivity + Connection-Lost + Draft-Initial Modal Overflow Hardening

> **Epic**: UI Clean-Pass
> **Story ID**: S18-UI-OVERLAY-PANEL-OVERFLOW-HARDENING-001
> **Status**: Draft -- future Sprint 18 candidate; NOT activated by this authoring run
> **Layer**: Presentation -- overlay panels (`photosensitivity_warning.rs`, `connection_lost_overlay.rs`, `shop_auction/mod.rs::draft_initial_*`)
> **Type**: Tech Debt -- per-surface overflow hardening (root-cause RC-2; per-surface O-02 / O-03 / S-08 / S-09)
> **Sprint**: Future Sprint 18 candidate per PROMPT 1180 §6 Lane J.
> **Authored**: 2026-05-18 by PROMPT 1189
> **Authoring source-of-truth**: `origin/main@efb698e`
> **Estimated effort**: ~0.4d
> **Source audit**: PROMPT 1180 §1.5 O-02 + O-03, §1.4 S-08 + S-09, §2 RC-2, §6 Lane J (PROMPT 1199 candidate)

---

## Status / No-Claim Banner

Future Sprint 18 candidate. **No sprint activated.** No claim on release readiness, `QA-COND-*`, `PAW-TD-*-a`, gate-check retry, stage advance, or closure of any audit finding outside Lane J / RC-2 / O-02 / O-03 / S-08 / S-09.

## Problem Class / Prevention Target

**Defect class**: three overlay panels carry the same overflow bug class.

- **O-02 — photosensitivity warning**: `width: 560 px, max_width: 92%`, no `max_height`, no overflow. At 1280×600 overflows top edge; Acknowledge button can sit off-screen.
- **O-03 — connection-lost overlay**: `width: 60%, max_width: 520, row_gap: 12, padding: 22`, no `max_height`, no overflow. Brittle to body expansion.
- **S-08 — draft-initial modal**: `width: 88%, max_width: 860, height: 360px, max_height: 92%` — fixed pixel + percent max conflict. 360 px at 1366×768 AND 3840×2160.
- **S-09 — draft-initial grid**: absolute 3×3 grid at `(96, 28)` offset with `left: 96 + col*132, top: 28 + row*66`.

**Prevention target**: every modal declares `max_height: Val::Percent(92.0)` + `overflow: Overflow::scroll_y()`. Draft-initial grid uses `Display::Grid` or `FlexWrap::Wrap`.

## 1180 Lane Coverage

Owns Lane J:

> | **J — Photosensitivity / connection-lost / draft-modal overflow hardening** | `client/src/ui/photosensitivity_warning.rs`, `client/src/presentation/connection_lost_overlay.rs`, `client/src/ui/shop_auction/mod.rs::{draft_initial_modal_panel_node, draft_initial_slot_node, draft_initial_grid_node}` | Lane A's live-spawn harness | **P2** | A, B, serialise with H on `shop_auction/mod.rs` |

## Context

- `client/src/ui/photosensitivity_warning.rs:226-263` — panel root.
- `client/src/presentation/connection_lost_overlay.rs:188-269` — panel root.
- `client/src/ui/shop_auction/mod.rs:5101-5114` — draft-initial modal.
- `client/src/ui/shop_auction/mod.rs:5116-5143` — draft-initial grid + slots.

**Reference**: `client/src/presentation/result_screen.rs:502-549` — PROMPT 1180 §1.5 O-04 "the only surface that does layout correctly". Use as template.

**GDD / ADR**: no body change.

**Engine / skills**: Bevy 0.18; `liv-bevy-018`; `Overflow::scroll_y()`, `Display::Grid` canonical.

### Control Manifest Rules

- Required: every in-scope modal declares `max_height: Val::Percent(92.0)` + `overflow: Overflow::scroll_y()` (§5 C-5).
- Required: `draft_initial_grid_node` uses `Display::Grid` (3×3) OR `FlexWrap::Wrap`; absolute offsets removed.
- Required: draft-initial modal `height: 360px` literal removed.
- Required: photosensitivity Acknowledge button anchored at panel bottom regardless of body length (footer slot pattern).
- Forbidden: editing UI modules outside the three listed.
- Forbidden: serialisation conflict with PROMPT 1182 on `draft_initial_*`.

## Story Classification

**Integration**.

## Dependencies and Parallelism

| Sibling | Parallel-safe? | Notes |
|---|---|---|
| Story 020 (Lane A) | PARTIAL | Consumes `PlayArea` if 020 lands first. |
| Stories 021 / 022 / 023 / 024 / 025 / 027 | YES | Disjoint. |
| Active PROMPT 1178 (lobby) | YES | Different files. |
| Active PROMPT 1182 (shop/auction) | NO | Same `draft_initial_*`; serialise. |
| Active PROMPT 1183 (HUD + overlays) | PARTIAL | May own `connection_lost_overlay.rs`; serialise on that file. |
| Active PROMPTs 1187 / 1188 | YES | Disjoint. |

## Acceptance Criteria

- [ ] AC1 -- Photosensitivity panel declares max-height + scroll; Acknowledge in footer slot.
- [ ] AC2 -- Connection-lost panel declares max-height + scroll.
- [ ] AC3 -- Draft-initial modal removes fixed `height: 360px`; adds `max_height: 92%` + scroll.
- [ ] AC4 -- Draft-initial grid uses `Display::Grid` OR `FlexWrap::Wrap`; absolute offsets removed.
- [ ] AC5 -- 1280×600 sub-floor: Acknowledge fully on-screen OR scroll-reachable. Lane B harness asserts if landed, else `overlay_overflow_hardening_test.rs` (NEW).
- [ ] AC6 -- 1366×768 primary controls reachable for all three overlays.
- [ ] AC7 -- 3840×2160 draft-initial modal scales to `max_height: 92%` (~1987 px), not fixed 360 px.
- [ ] AC8 -- `result_screen.rs:502-549` unchanged (reference template).
- [ ] AC9 -- `liv-bevy-018` activated.
- [ ] AC10 -- Cargo resource policy applied.
- [ ] AC11 -- No accept-risk closure.
- [ ] AC12 -- Sprint disposition preserved.
- [ ] AC13 -- Worker branch scope contained; slug `work/s18-ui-overlay-panel-overflow-hardening`.

## Implementation Notes

### Owned files

| Path | Expected change |
|------|-----------------|
| `client/src/ui/photosensitivity_warning.rs` | Max-height + scroll + footer-slot Acknowledge. |
| `client/src/presentation/connection_lost_overlay.rs` | Same overflow hardening. |
| `client/src/ui/shop_auction/mod.rs` | Refactor `draft_initial_modal_panel_node` + grid + slot. |
| `tests/integration/ui_clean_pass/overlay_overflow_hardening_test.rs` (NEW, conditional) | AC5..AC7 if Lane B not landed. |

### Forbidden files

- Other UI surfaces.
- `result_screen.rs` (AC8); `qa_snapshot.rs`.
- Server, shared, ADRs, sprint / state / QA / Cargo files.

## Worker Contract

1. Worktree slug `work/s18-ui-overlay-panel-overflow-hardening`.
2. Read story + PROMPT 1180 §1.5 O-02 / O-03 + §1.4 S-08 / S-09 + §2 RC-2 + §5 C-5 + §6 Lane J.
3. Activate `liv-bevy-018`.
4. Cargo resource policy env vars.
5. Targeted tests only.
6. Push worker branch only.
7. Verify serialisation with PROMPT 1182 on `draft_initial_*`; BLOCK + relay if 1182 in flight on the same function set.
