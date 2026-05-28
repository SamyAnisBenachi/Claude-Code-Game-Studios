# PROMPT 2019 — S18-OBS-SNAPSHOT-LAYOUT-FIELDS-001 Story-Done Paperwork

> **Date**: 2026-05-28
> **Source-of-truth at closure**: `origin/main@0501437355c8b963489bc349968ca7fc1d4b7345` (PROMPT 2018 tip)
> **Story**: S18-OBS-SNAPSHOT-LAYOUT-FIELDS-001 — QA Snapshot Layout-Debug Field Enrichment (Q-01..Q-10)
> **Story file**: `production/epics/ui-clean-pass/story-023-obs-snapshot-layout-fields.md`
> **Type**: Paperwork-only /story-done (implementation pre-landed on origin/main before Sprint 18 activation)

---

## 1. Implementation Evidence

### Primary Implementation

**PROMPT 1186 commit `d75db1af`** — `dev-story(s18-obs-snapshot-layout-fields): add Q-01..Q-10 layout-debug fields to QA snapshot`

Files modified:
- `client/src/presentation/qa_snapshot.rs` (+605 lines) — new `LayoutSnapshot` struct with:
  - `ViewportLayoutSnapshot` (Q-01: width_px, height_px, ui_scale, window_scale_factor)
  - `Vec<SurfaceLayoutSnapshot>` for 19 canonical surface root markers (Q-02: bounds, Q-03: overflow_clipped, Q-04: children_count, Q-08: z_layer_resolved)
  - `Vec<ButtonAffordanceSnapshot>` (Q-07: entity, name, interaction)
  - `LayoutCollisionsSnapshot` (Q-09: placement_action_panel_overlaps, Q-10: shop_panel_bottom_edge_y / hand_bar_top_edge_y / overlap_px)
  - `limitations: Vec<String>` documenting Q-05/Q-06/Q-07 partial gaps
- `tests/integration/qa_snapshot/layout_field_coverage_test.rs` (NEW, 682 lines, 14 tests)
- `client/Cargo.toml` — `[[test]]` entry for new coverage test

**Tests (PROMPT 1186)**: `cargo test -p client --test qa_snapshot_layout_field_coverage_test` → **14/14 PASS**

### Follow-up Implementation

**PROMPT 1533 commit `03342873`** — `PROMPT-1533 qa_snapshot: ACK lifecycle + hover provenance + label roles`
- Q-05 partial closure: added semantic `role` token (stable Name-derived identifier) to `ui_text_markers[]` entries.

### Note on PROMPT 1287/1229 Inventory Reference

The Sprint 18 plan activation inventory (PROMPT 1287 §2) cited `e68ac4f` / PROMPT 1229 for this story. Clarification:
- PROMPT 1229 (`e68ac4f`) adds `placement_state` / `auction_state` / `current_phase.timer_remaining_ms` — separate feature, not Q-01..Q-10.
- PROMPT 1229 also updated `layout_field_coverage_test.rs` struct constructions to accommodate the new `QASnapshotData` fields.
- The actual Q-01..Q-10 layout field implementation is PROMPT 1186 (`d75db1af`), which is an ancestor of `origin/main@05014373`.

---

## 2. AC-Level Story Readiness Audit

AC outcomes against `origin/main@05014373`:

| AC | Outcome | Evidence |
|---|---|---|
| AC1 Q-01 | PASS | `ViewportLayoutSnapshot.{width_px,height_px,ui_scale,window_scale_factor}` from `PrimaryWindow` |
| AC2 Q-02 | PASS | `SurfaceLayoutSnapshot.bounds: Option<SurfaceBoundsRect>` {x,y,w,h} in logical px via `ComputedNode` + `GlobalTransform` |
| AC3 Q-03 | PASS | `SurfaceLayoutSnapshot.overflow_clipped: Option<bool>` via `ComputedNode::content_size` vs `size` |
| AC4 Q-04 | PASS | `SurfaceLayoutSnapshot.children_count: Option<usize>` via `Children` component |
| AC5 Q-05 | PASS-WITH-LIMITATIONS | text/Name/bounds/font_px/clipped/overflow_px + role token emitted; per-glyph `clipped_chars` NOT computable (qa_snapshot.rs:3585-3590) |
| AC6 Q-06 | PASS-WITH-LIMITATIONS | null-emitted (not computable without per-image-marker components + `Assets<Image>` read; forbidden write scope); documented (qa_snapshot.rs:3591-3597) |
| AC7 Q-07 | PASS-WITH-LIMITATION | `button_affordances[].interaction` = default/hover/pressed; no `disabled` (Bevy 0.18 `Interaction` has no Disabled variant; qa_snapshot.rs:3598-3601) |
| AC8 Q-08 | PASS | `SurfaceLayoutSnapshot.z_layer_resolved: Option<i32>` via `GlobalZIndex` |
| AC9 Q-09 | PASS | `LayoutCollisionsSnapshot.placement_action_panel_overlaps: Vec<String>` via `build_layout_collisions` helper |
| AC10 Q-10 | PASS | `LayoutCollisionsSnapshot.{shop_panel_bottom_edge_y,hand_bar_top_edge_y,shop_panel_vs_hand_bar_overlap_px}` |
| AC11 | PASS | `tests/integration/qa_snapshot/layout_field_coverage_test.rs` — 14 tests, 14/14 PASS at PROMPT 1186 |
| AC12 | PASS | Additive schema extension; `CCGS_QA_SNAPSHOT=1` contract preserved |
| AC13 | PASS | `production/qa/evidence/sprint-18-snapshot-layout-fields/evidence.md` created by PROMPT 2019; Q-05/Q-06/Q-07 gaps with file:line |
| AC14 | PASS | `liv-bevy-018` activated by PROMPT 1186 (ComputedNode, GlobalTransform, GlobalZIndex, Children) |
| AC15 | PASS | PROMPT 1186 commit: CARGO_TARGET_DIR + DEBUG=0 + INCREMENTAL=0 + RUSTFLAGS confirmed |
| AC16 | PASS | No accept-risk closure; 24 PROMPT 1022 findings preserved |
| AC17 | PASS | Sprint 18 active / stage Polish UNCHANGED |
| AC18 | PASS | Branch `s18-obs-snapshot-layout-fields` confirmed in `git log --all` |

**Overall story verdict: DONE — AC1..AC18 all PASS (AC5/AC6/Q-07 PASS-WITH-LIMITATIONS per documented gap contract)**

---

## 3. Files Changed by PROMPT 2019

| File | Change |
|---|---|
| `production/epics/ui-clean-pass/story-023-obs-snapshot-layout-fields.md` | Status Draft → Done; Sprint/Completed/Impl PROMPT lines updated; AC1..AC18 [x]; Completion Notes section added |
| `production/qa/evidence/sprint-18-snapshot-layout-fields/evidence.md` | NEW — AC13 limitation notes (Q-05/Q-06/Q-07 with file:line) |
| `production/sprint-status.yaml` | S18-OBS-SNAPSHOT-LAYOUT-FIELDS-001 row flipped `status: ready → done`; `sprint_18_activation.active_set.should_have` row annotated with `status_post_closure`; `rows_not_closed_by_prompt_1718` OBS entry updated; PROMPT 2019 block appended as 8th `sprint_18_story_done:` entry |
| `reports/PROMPT-2019-s18-obs-snapshot-layout-fields-story-done-paperwork.md` | This report |

---

## 4. Sprint Coverage Update

Sprint 18 active row coverage: **9 of 12 DONE** (was 8 of 12 after PROMPT 1718).

Remaining open rows:
- `S11-HUD-TIMER-EYEBALL-VISUAL-001` — must-have, human-operator-blocked carry, no LLM /story-done authorised
- `S18-UI-CARD-ART-AND-LABEL-STRIP-001` — should-have, ready
- `S18-UI-INTERACTION-STATE-MIGRATION-WAVE-2-001` — nice-to-have, ready

---

## 5. Non-Claims

- Stage advance, Sprint 18 close-out, Sprint 19 activation: NOT claimed
- Polish→Release gate-check retry: NOT claimed (PROMPT 761 FAIL preserved)
- S8-QA-001-W1 / QA-COND-0005 / QA-COND-0006 / PAW-TD-*-a: NOT claimed
- Q-05 per-glyph `clipped_chars` closure: NOT claimed
- Q-06 image aspect ratio closure: NOT claimed
- Any PROMPT 1022 / 1076 / 1077 finding closure: NOT claimed

---

## 6. Forbidden Changes Observed

- `client/**` / `server/**` / `shared/**` / `tests/**` NOT modified
- `Cargo.toml` / `Cargo.lock` / `Trunk.toml` NOT modified
- `production/stage.txt` NOT modified (remains Polish)
- `production/sprints/sprint-18.md` NOT modified
- `production/gate-checks/gate-polish-release-2026-05-12.md` NOT modified
- Sprint 17 / 16 / 15 / 14 / 13 / 12 / 11 / 10 sprint-status blocks NOT modified
- `sprint_18_story_done:` PROMPT 1337 + 1331 + 1357 + 1713 + 1716 + 1717 + 1718 entries preserved verbatim
- No /smoke-check, /team-qa, /gate-check, /release-check run; no Cargo / trunk invocation; no Polish→Release retry

---

2019: S18-OBS-SNAPSHOT-LAYOUT-FIELDS-001: SHIPPED
