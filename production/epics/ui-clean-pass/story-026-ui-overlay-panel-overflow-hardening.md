# Story 026: S18-UI-OVERLAY-PANEL-OVERFLOW-HARDENING-001 -- Photosensitivity + Connection-Lost + Draft-Initial Modal Overflow Hardening

> **Epic**: UI Clean-Pass
> **Story ID**: S18-UI-OVERLAY-PANEL-OVERFLOW-HARDENING-001
> **Status**: Done — closed by PROMPT 1718 on origin/main@6db48a9a (2026-05-28)
> **Layer**: Presentation -- overlay panels (`photosensitivity_warning.rs`, `connection_lost_overlay.rs`, `shop_auction/mod.rs::draft_initial_*`)
> **Type**: Tech Debt -- per-surface overflow hardening (root-cause RC-2; per-surface O-02 / O-03 / S-08 / S-09)
> **Sprint**: Sprint 18 (active)
> **Authored**: 2026-05-18 by PROMPT 1189
> **Authoring source-of-truth**: `origin/main@efb698e`
> **Estimated effort**: ~0.4d
> **Completed**: 2026-05-28
> **Impl PROMPT**: PROMPT 1349 (f7cfa422) — max-height + scroll + footer-slot pattern on photosensitivity, connection-lost, draft-initial
> **Source audit**: PROMPT 1180 §1.5 O-02 + O-03, §1.4 S-08 + S-09, §2 RC-2, §6 Lane J (PROMPT 1199 candidate)

---

## Status / No-Claim Banner

**DONE** — closed by PROMPT 1718 on `origin/main@6db48a9a` (2026-05-28). Sprint 18 active / stage Polish UNCHANGED. No claim on release readiness, `QA-COND-*`, `PAW-TD-*-a`, gate-check retry, stage advance, or closure of any audit finding outside Lane J / RC-2 / O-02 / O-03 / S-08 / S-09.

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

- [x] AC1 -- Photosensitivity panel declares max-height + scroll; Acknowledge in footer slot. **PASS**: `test_overlay_overflow_ac1_photosensitivity_panel_declares_max_height_and_scroll` + `test_overlay_overflow_ac1_photosensitivity_acknowledge_in_footer_slot` ok (PROMPT 1349).
- [x] AC2 -- Connection-lost panel declares max-height + scroll. **PASS**: `test_overlay_overflow_ac2_connection_lost_panel_declares_max_height_and_scroll` ok (PROMPT 1349).
- [x] AC3 -- Draft-initial modal removes fixed `height: 360px`; adds `max_height: 92%` + scroll. **PASS**: `test_overlay_overflow_ac3_draft_initial_modal_drops_height_literal` ok (PROMPT 1349).
- [x] AC4 -- Draft-initial grid uses `Display::Grid` OR `FlexWrap::Wrap`; absolute offsets removed. **PASS**: `test_overlay_overflow_ac4_draft_initial_grid_uses_display_grid` + `test_overlay_overflow_ac4_draft_initial_slot_drops_absolute_offsets` ok (PROMPT 1349).
- [x] AC5 -- 1280×600 sub-floor: Acknowledge fully on-screen OR scroll-reachable. Lane B harness asserts if landed, else `overlay_overflow_hardening_test.rs` (NEW). **PASS**: `test_overlay_overflow_ac5_sub_floor_viewport_keeps_acknowledge_reachable` ok; NEW `overlay_overflow_hardening_test.rs` (433 lines) (PROMPT 1349).
- [x] AC6 -- 1366×768 primary controls reachable for all three overlays. **PASS**: `test_overlay_overflow_ac6_primary_viewport_controls_reachable` ok (PROMPT 1349).
- [x] AC7 -- 3840×2160 draft-initial modal scales to `max_height: 92%` (~1987 px), not fixed 360 px. **PASS**: `test_overlay_overflow_ac7_4k_viewport_modal_scales_to_max_height` ok (PROMPT 1349).
- [x] AC8 -- `result_screen.rs:502-549` unchanged (reference template). **PASS**: `test_overlay_overflow_ac8_result_screen_reference_template_unchanged` ok (PROMPT 1349).
- [x] AC9 -- `liv-bevy-018` activated. **PASS**: cited in commit message + Bevy 0.18 patterns (`Overflow::scroll_y()`, `Display::Grid`, `Val::Percent`) used throughout (PROMPT 1349).
- [x] AC10 -- Cargo resource policy applied. **PASS**: PROMPT 1371 integration §4.1 confirms `CARGO_TARGET_DIR=D:/_DEV/cargo-target/ccgs-msvc` + debuginfo off + incremental off on every invocation.
- [x] AC11 -- No accept-risk closure. **PASS**: commit message + PROMPT 1371 non-claims section confirm; no carried condition closed.
- [x] AC12 -- Sprint disposition preserved. **PASS**: Sprint 18 active / stage Polish UNCHANGED; production/stage.txt NOT modified (PROMPT 1375 verification).
- [x] AC13 -- Worker branch scope contained; slug `work/s18-ui-overlay-panel-overflow-hardening`. **PASS**: PROMPT 1349 worker used slug `work/s18-ui-overlay-panel-overflow-hardening` per git log --all (branches `work/s18-ui-overlay-panel-overflow-hardening-1349` and `work/s18-ui-overlay-panel-overflow-hardening-main-land-1375`).

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

## Completion Notes (PROMPT 1718)

**Implementation lineage:**
- Worker: PROMPT 1349 (`f7cfa422`) — max-height + scroll + footer-slot pattern on photosensitivity, connection-lost, draft-initial; `Display::Grid` on draft-initial grid; `overlay_overflow_hardening_test.rs` NEW (433 lines, 10 tests, AC1..AC8 binding).
- Integration: PROMPT 1371 (`c4748158`) — GREEN; cherry-pick conflict-free onto `origin/main@516b642`; 10/10 + 10/10 tests PASS.
- Main-land: PROMPT 1375 (`f7cfa422` refresh-cherry-pick onto `origin/main@86f61ee`) — LANDED; `origin/main` advanced to `f7cfa42`; tests re-verified 10/10 + 10/10 PASS.

**Files changed by PROMPT 1349:**
- `client/src/ui/photosensitivity_warning.rs` (+80 / -?) — max-height + scroll + footer-slot Acknowledge.
- `client/src/presentation/connection_lost_overlay.rs` (+42 / -?) — max-height + scroll.
- `client/src/ui/shop_auction/mod.rs` (+51 / -?) — draft-initial modal drops 360px literal; `Display::Grid` grid; max-height + scroll.
- `client/Cargo.toml` (+14) — test target registration.
- `tests/integration/ui_clean_pass/overlay_overflow_hardening_test.rs` (NEW, +433) — AC1..AC8 binding.
- `tests/integration/shop_auction_ui/draft_initial_centered_modal_layout_test.rs` (+42 / -?) — adjacent contract refresh (post-1349 contract; 10/10 PASS).

**Conditions Carried Forward (unchanged):**
- Sprint 18 active / stage Polish / `production/stage.txt` NOT modified.
- PROMPT 761 Polish→Release gate-check FAIL preserved; NO retry.
- `S8-QA-001-W1` OPEN preserved.
- `QA-COND-0005` + `QA-COND-0006` accepted-risk preserved.
- `PAW-TD-*-a` placeholder-art accept-risk preserved.
- `TQ-S12-C1..C7` preserved verbatim.
- `S11-HUD-TIMER-EYEBALL-VISUAL-001` human-operator-blocked carry preserved; NOT closed by PROMPT 1718.

**Explicitly NOT Claimed:**
- Stage advance, Sprint 18 close-out, Sprint 19 activation.
- Release readiness, RC readiness, full game completion.
- QA-COND-0005/0006 advancement, PAW-TD-*-a final-art, S8-QA-001-W1 closure.
- Polish→Release gate-check retry.
- Any AUDIT-1076-\* / SOURCE-1077-\* / PROMPT 1022 finding closure.

**Closure Trail:**
- `production/epics/ui-clean-pass/story-026-ui-overlay-panel-overflow-hardening.md` — this file (Status Done; AC1..AC13 [x]).
- `production/sprint-status.yaml` — S18-UI-OVERLAY-PANEL-OVERFLOW-HARDENING-001 row: `status: ready` → `done`; sprint_18_activation nice_to_have entry annotated; PROMPT 1718 sprint_18_story_done block appended as 7th block.
- `reports/PROMPT-1718-s18-overlay-panel-overflow-story-done.md` — mandatory final report.
