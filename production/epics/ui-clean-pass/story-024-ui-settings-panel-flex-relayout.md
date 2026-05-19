# Story 024: S18-UI-SETTINGS-PANEL-FLEX-RELAYOUT-001 -- Settings Panel Flex Re-Layout + UI-Scale Invariant

> **Epic**: UI Clean-Pass
> **Story ID**: S18-UI-SETTINGS-PANEL-FLEX-RELAYOUT-001
> **Status**: Done -- closed by PROMPT 1331 on `origin/main@4940a7b` (Sprint 18 Should Have). Implementation: PROMPT 1187 (`8eeb94e`).
> **Layer**: Presentation -- settings UI surface (`client/src/ui/settings/mod.rs` only)
> **Type**: Tech Debt -- per-surface re-layout (root-cause RC-2; per-surface O-01)
> **Sprint**: Sprint 18 Should Have row per `production/sprints/sprint-18.md`. Activated by PROMPT 1301; closed by PROMPT 1331.
> **Authored**: 2026-05-18 by PROMPT 1189
> **Authoring source-of-truth**: `origin/main@efb698e`
> **Closure source-of-truth**: `origin/main@4940a7bdcbf7189a6c1d7adb5cf87edc93022096` (PROMPT 1326 windows launcher main-land tip; strict descendant of PROMPT 1187 implementation tip `8eeb94e`).
> **Estimated effort**: ~0.5d (Sprint 18 activation re-baselined to 0.25d as verify-only)
> **Source audit**: PROMPT 1180 §1.5 O-01, §2 RC-2, §6 Lane F (PROMPT 1195 candidate)
> **Active impl PROMPT**: PROMPT 1187 (`8eeb94e dev-story(s18-ui-settings-panel-flex-relayout): replace absolute child stack with bounded flex layout`); landed first as predicted.

---

## Status / No-Claim Banner

Sprint 18 Should Have row, closed Done by PROMPT 1331 paperwork-only `/story-done`. **No claim** on release readiness, `QA-COND-0005` Standard-tier completion (this row is a **precondition**, not closure), `QA-COND-0006`, `PAW-TD-*-a`, gate-check retry, stage advance, or closure of any audit finding outside Lane F / O-01. PROMPT 761 Polish->Release FAIL preserved with NO retry; `production/stage.txt` NOT modified; stage remains `Polish`.

## Problem Class / Prevention Target

**Defect class** (O-01): settings panel is a hardcoded `760×520 px` block of 8 absolute-positioned children (`settings/mod.rs:1158-1268`). `settings_panel_node(scale_percent)` scales container only, not children. UI-scale 75% shrinks to 570×390 — children clip. UI-scale 150% grows to 1140×780 — children still occupy 760×520, dead zone. **The accessibility UI-scale primitive is broken by design.**

**Prevention target**: replace 8 absolute children with flex rows + columns + scroll body. UI-scale 75% / 100% / 150% must not clip children or leave dead zones. 1280×720 / 1366×768 must keep all primary controls reachable.

## 1180 Lane Coverage

Owns Lane F:

> | **F — Settings panel re-layout (O-01)** | `client/src/ui/settings/mod.rs` only | `tests/integration/settings/ui_scale_invariant_test.rs` (NEW) | **P1** |

## Context

- `client/src/ui/settings/mod.rs:17-18` — `SETTINGS_PANEL_WIDTH_PX=760`, `_HEIGHT_PX=520`.
- `client/src/ui/settings/mod.rs:919-920` — scale percent applied to container only.
- `client/src/ui/settings/mod.rs:1158-1268` — 8 absolute child builders.

**GDD / ADR**: `design/gdd/accessibility-settings.md` if present; no body change. ADR-021 / 023 preserved.

**Engine / skills**: Bevy 0.18; `liv-bevy-018`; `Overflow::scroll_y()` canonical.

### Control Manifest Rules

- Required: replace each absolute child with flex; panel becomes `display: Flex, flex_direction: Column` with header / category column / content pane / footer sub-regions.
- Required: `max_height: Val::Percent(92.0)` + `overflow: Overflow::scroll_y()` on content pane (§5 C-5).
- Required: UI-scale 75% / 100% / 150% must not clip children.
- Required: 1280×720 / 1366×768 keep all primary controls reachable (in viewport OR via scroll).
- Required: visual hierarchy preserved (back top-left, category column left, content pane right, footer bottom).
- Forbidden: new `PositionType::Absolute` child offsets (panel root may stay absolute as centered-overlay).
- Forbidden: editing other UI modules.

## Story Classification

**Integration**.

## Dependencies and Parallelism

| Sibling | Parallel-safe? | Notes |
|---|---|---|
| Stories 020 / 021 / 022 / 023 / 025 / 026 / 027 | YES | Disjoint. |
| Active PROMPTs 1178 / 1182 / 1183 / 1188 | YES | Disjoint. |
| Active PROMPT 1181 (foundation primitives) | PARTIAL | Serialise if 1181 in flight on `design_tokens/`. |
| Active PROMPT 1187 | DUPLICATE | Impl worker; may land first. |

## Acceptance Criteria

- [x] AC1 -- No absolute-positioned child offsets: `grep -n "PositionType::Absolute" client/src/ui/settings/mod.rs` reports at most ONE occurrence (panel root).
  **VERDICT: PASS** -- verified on `origin/main@4940a7b`: `grep -c "PositionType::Absolute" client/src/ui/settings/mod.rs` returns `1` (panel root only). All 8 absolute child offsets replaced by the bounded flex hierarchy authored by PROMPT 1187 (`header_row` / `body_row` / `category_column` / `content_pane` / `footer_row`).
- [x] AC2 -- Panel scroll + max-height: `max_height: Val::Percent(92.0)` + `overflow: Overflow::scroll_y()`.
  **VERDICT: PASS** -- verified on `origin/main@4940a7b`: `client/src/ui/settings/mod.rs:1218-1222` sets `max_width: Val::Percent(92.0)` + `max_height: Val::Percent(92.0)` + `overflow: Overflow::scroll_y()` on the panel root; `client/src/ui/settings/mod.rs:1328` repeats `overflow: Overflow::scroll_y()` on the inner `content_pane`. `sync_settings_shell_visibility_system` rewrites `width = Px(BASE * factor)`, `min_width = Px(SETTINGS_PANEL_MIN_WIDTH_PX)`, `max_width = Percent(92)`, `height = Auto`, `max_height = Percent(92)` every frame (`client/src/ui/settings/mod.rs:961-969`).
- [x] AC3 -- UI-scale 75% keeps children inside panel bounds.
  **VERDICT: PASS** -- covered by `tests/integration/accessibility_settings/ui_scale_invariant_test.rs` AC3 assertion. New `SETTINGS_PANEL_MIN_WIDTH_PX = 540.0` floor preserves the `category_column` (170 px) + `content_pane` room at 75% scale (570 px ceiling for 760 base × 0.75); the flex children no longer carry literal pixel offsets, so they shrink with the panel bounds.
- [x] AC4 -- UI-scale 150% no dead zones: `content_size` ≈ `inner_size` − padding within 4 px.
  **VERDICT: PASS** -- covered by `tests/integration/accessibility_settings/ui_scale_invariant_test.rs` AC4 assertion. `body_row` carries `flex_grow: 1` and `content_pane` carries `flex_grow: 1` so the children expand to fill the grown panel rather than leaving a 760×520 island inside a 1140×780 container.
- [x] AC5 -- 1280×720 keeps primary controls reachable (viewport OR scroll).
  **VERDICT: PASS** -- covered by `tests/integration/accessibility_settings/ui_scale_invariant_test.rs` AC5 assertion. `max_height: Val::Percent(92.0)` ceiling on the panel root + `overflow: Overflow::scroll_y()` on the panel and `content_pane` guarantee reachability via viewport-or-scroll at 1280×720.
- [x] AC6 -- 1366×768 keeps primary controls reachable.
  **VERDICT: PASS** -- covered by `tests/integration/accessibility_settings/ui_scale_invariant_test.rs` AC6 assertion. Same `max_height` + `overflow: Overflow::scroll_y()` contract carries 1366×768.
- [x] AC7 -- Visual hierarchy preserved; before/after screenshots in `production/qa/evidence/sprint-18-settings-flex-relayout/`.
  **VERDICT: PASS-STRUCTURAL + ADVISORY-EVIDENCE-DEFERRED** -- structural hierarchy preserved verbatim in the PROMPT 1187 dev-story commit message and source (header `back_close_button` top-left → `category_column.category_accessibility` left → `content_pane` (colorblind / reduced-motion / timer-row / effective-timer / menu-scale / hud-scale) right → footer `status_footer` + `footer_close_button` bottom). Optional per-row screenshot evidence directory `production/qa/evidence/sprint-18-settings-flex-relayout/` was not authored at activation; the story ACs marked it as optional ("optional AC7 screenshots" per story `Owned files`); the structural test in `ui_scale_invariant_test.rs` is the binding gate. ADVISORY recorded; not hidden.
- [x] AC8 -- Settings functionality preserved (round-trip identical).
  **VERDICT: PASS** -- covered by `tests/integration/accessibility_settings/ui_scale_invariant_test.rs` AC8 assertion (marker counts unchanged; focus-order traversal preserved). Sibling tests `tests/integration/accessibility_settings/settings_shell_test.rs` + `timer_selector_test.rs` continue to pass under the new flex hierarchy per PROMPT 1187 worker evidence.
- [x] AC9 -- `liv-bevy-018` activated.
  **VERDICT: PASS** -- PROMPT 1187 worker activated `liv-bevy-018` for every `.rs` edit (per PROMPT 1324 readiness audit row 6; commit message documents 0.18-idiom usage including `FlexWrap::Wrap`, `JustifyContent::SpaceBetween`, `Overflow::scroll_y()`, `Val::Percent` / `Val::Px` ratios; no deprecated `Bundle` types; no `set_parent` / `despawn_recursive`; no pre-0.15 patterns).
- [x] AC10 -- Cargo resource policy applied.
  **VERDICT: PASS** -- PROMPT 1187 worker applied the 5 Windows/MSVC Cargo resource policy env vars before every Cargo invocation per the commit's "test coverage" section. PROMPT 1331 itself is paperwork-only and does NOT invoke Cargo.
- [x] AC11 -- No accept-risk closure; `QA-COND-0005` row-precondition note explicit.
  **VERDICT: PASS** -- this row is a `QA-COND-0005` Standard-tier accessibility **precondition**, not closure. `QA-COND-0005` remains accepted-risk after PROMPT 1331. `QA-COND-0006`, `PAW-TD-*-a`, `S8-QA-001-W1`, `TQ-S12-C1..C7` (TQ-S12-C7 explicitly NOT closed), PROMPT 683-era runtime divergence, PROMPT 1054 BLOCKED-HUMAN, and PROMPT 761 Polish->Release FAIL all preserved verbatim.
- [x] AC12 -- Sprint disposition preserved.
  **VERDICT: PASS** -- Sprint 18 disposition remains `active` after PROMPT 1331 (Sprint 18 NOT closed-out by this row). Stage `Polish` unchanged. `production/stage.txt` NOT modified.
- [x] AC13 -- Worker branch scope contained; slug `work/s18-ui-settings-panel-flex-relayout`.
  **VERDICT: PASS** -- PROMPT 1187 worker landed via dev-story commit `8eeb94e` on origin/main directly (slug `s18-ui-settings-panel-flex-relayout` honoured in the commit subject). Worker branch did not push main; integration was main-land via the activation tip lineage rather than a separate `integrate/*` branch (impl landed pre-Sprint-18 activation per PROMPT 1287 §2 inventory and was therefore inherited at the Sprint 18 activation tip `1345c6b` per PROMPT 1301).

## Implementation Notes

### Owned files

| Path | Expected change |
|------|-----------------|
| `client/src/ui/settings/mod.rs` | Replace 8 absolute child builders with flex. |
| `tests/integration/settings/ui_scale_invariant_test.rs` (NEW) | AC3..AC8. |
| `production/qa/evidence/sprint-18-settings-flex-relayout/` (NEW) | Optional AC7 screenshots. |

### Forbidden files

- Other UI surfaces, `design_tokens/**` (1181 if active).
- Server, shared, sprint / stage / QA / gate-check (except AC7 evidence), ADRs.

## Worker Contract

1. Worktree slug `work/s18-ui-settings-panel-flex-relayout`.
2. Read story + PROMPT 1180 §1.5 O-01 + §5 C-3 / C-5 + §6 Lane F.
3. Activate `liv-bevy-018`.
4. Cargo resource policy env vars.
5. Targeted tests only.
6. Push worker branch only.

---

## Completion Notes (PROMPT 1331)

PROMPT 1331 is the paperwork-only Sprint 18 `/story-done` closure for `S18-UI-SETTINGS-PANEL-FLEX-RELAYOUT-001` on the strength of:

- **PROMPT 1187 worker** (`8eeb94e3244245850b044e83ffcfff4df0da835f`):
  Replaced the 8-child absolute-positioned 760×520 settings panel with a bounded flex hierarchy:
  `panel (flex column, padding SPACING_LG, row_gap SPACING_MD, Overflow::scroll_y, max_width Percent(92), max_height Percent(92))` →
  `header_row (flex row) → back_close_button`,
  `body_row (flex row, flex_grow 1) → category_column (flex column, width 170 px) + content_pane (flex column, flex_grow 1, Overflow::scroll_y)`,
  `footer_row (flex row, JustifyContent::SpaceBetween) → status_footer + footer_close_button`.
  `sync_settings_shell_visibility_system` rewrites width/min_width/max_width/max_height every frame so UI-scale changes keep the panel bounded inside the viewport at 75 / 100 / 125 / 150 percent. New `SETTINGS_PANEL_MIN_WIDTH_PX = 540.0` floor preserves the category column + content pane room at 75% scale (570 px ceiling for 760 base × 0.75).
  Test coverage: `tests/integration/accessibility_settings/ui_scale_invariant_test.rs` (NEW, 324 lines, 8 `#[test]` declarations + 2 helpers) covering AC1..AC8 + marker counts unchanged + focus-order traversal. All existing settings tests (`accessibility_settings_shell_test`, `accessibility_settings_timer_selector_test`) continue to pass under the new flex hierarchy.
  Files touched by PROMPT 1187: `client/Cargo.toml` (+4 lines), `client/src/ui/settings/mod.rs` (+218 / -49), `tests/integration/accessibility_settings/ui_scale_invariant_test.rs` (NEW +325). Total: 3 files, 498 insertions, 49 deletions.
- **Integration**: PROMPT 1187 landed directly via dev-story commit `8eeb94e` on `origin/main`; no separate `/integrate` prompt was authored. PROMPT 1232 / PROMPT 1263 inventories listed this row as "effectively implemented on `origin/main`"; PROMPT 1287 §2 inventory recorded "no explicit commit captured" (paperwork mis-attribution). PROMPT 1324 readiness audit row 6 identified the implementation commit as `8eeb94e` and produced verdict `READY_FOR_STORY_DONE`.

### PROMPT 1324 test-path mismatch advisory

PROMPT 1324 row 6 surfaced a **paperwork-only** mismatch between the story / QA-plan spec and the landed test file location:

- **Spec path** (story `Owned files` + QA plan row 10 §"Required automated tests"): `tests/integration/settings/ui_scale_invariant_test.rs`.
- **Actual landed path** (`origin/main@4940a7b` per PROMPT 1187 commit `8eeb94e`): `tests/integration/accessibility_settings/ui_scale_invariant_test.rs`.

Same surface coverage; the actual directory is **consistent with the existing pattern** (sibling tests `tests/integration/accessibility_settings/settings_shell_test.rs` + `timer_selector_test.rs` already live under `accessibility_settings/`). PROMPT 1324 verdict: minor paperwork mismatch; not a hard AC gap. Two discharge options proposed: (a) accept the actual path and annotate; (b) move the file to match the spec.

PROMPT 1331 selects **option (a)**: accept the actual landed path under `tests/integration/accessibility_settings/` as the binding test artifact. Rationale: (i) the test file is already on `origin/main` providing the same AC3..AC8 + marker-count + focus-order coverage the spec required; (ii) `accessibility_settings/` is the established directory for this surface (3 of 3 settings integration test files live there on `origin/main`); (iii) moving the file would burn a fresh worker round and produce an integration commit with zero functional change, contrary to the paperwork-only `/story-done` charter; (iv) PROMPT 1331 is forbidden from touching `tests/**` anyway by the allowed-files scope. The story-024 + QA plan row 10 paperwork wording (`tests/integration/settings/...`) is preserved unchanged in the story narrative; this Completion Notes section is the single point of record acknowledging the directory mismatch, mirroring the PROMPT 1110 "PROMPT 1106 evidence-file trailing-whitespace advisory" precedent.

### Test Evidence

- **Story type**: Integration (per `.claude/docs/coding-standards.md` "Test Evidence by Story Type" matrix; BLOCKING gate satisfied by automated integration test).
- **Required evidence per matrix**: integration test OR documented playtest (BLOCKING).
- **Worker evidence on `origin/main@4940a7b`**:
  - `tests/integration/accessibility_settings/ui_scale_invariant_test.rs` (NEW, 324 lines, 8 `#[test]` declarations) covers AC1 (`PositionType::Absolute` count = 1), AC2 (`max_height` + `Overflow::scroll_y` on both panel and content pane), AC3..AC6 (UI-scale 75 / 100 / 125 / 150 percent + 1280×720 / 1366×768 reachability), AC7 (visual-hierarchy structural assertions), AC8 (marker counts unchanged; focus-order traversal).
  - `client/src/ui/settings/mod.rs` carries the bounded flex hierarchy + `SETTINGS_PANEL_MIN_WIDTH_PX` floor + `sync_settings_shell_visibility_system` width/min_width/max_width/max_height rewrite.
  - Optional `production/qa/evidence/sprint-18-settings-flex-relayout/` screenshot directory was not authored at activation; the structural integration test is the binding AC7 gate (visual-hierarchy assertions in the test file).
- **PROMPT 1187 worker Cargo gate**: applied the 5 Windows/MSVC Cargo resource policy env vars per the commit message; all existing settings tests + new `ui_scale_invariant_test` declarations PASS at worker.
- **PROMPT 1331 itself does NOT invoke Cargo** (paperwork-only closure).

---

## Closure Trail

| Prompt | Date | Source-of-truth | Commit | Disposition |
|---|---|---|---|---|
| PROMPT 1180 | 2026-05-18 | `origin/main@efb698e` | (audit report) | Lane F / O-01 / RC-2 identified; this row reserved for PROMPT 1195 candidate |
| PROMPT 1187 | 2026-05-18 | (pre-Sprint-18 activation) | `8eeb94e` | `/dev-story` worker: 8-child absolute panel → bounded flex hierarchy; +218 / -49 in `client/src/ui/settings/mod.rs`; NEW 325-line integration test (landed under `tests/integration/accessibility_settings/`); Cargo gate pass under policy |
| PROMPT 1189 | 2026-05-18 | `origin/main@efb698e` | (story authoring batch tip) | Story 024 authored as Sprint 18 candidate |
| PROMPT 1232 / 1263 | 2026-05-18 | (Sprint 18 plan inventory) | n/a | Row listed as "effectively implemented on origin/main" pre-activation |
| PROMPT 1287 | 2026-05-18 | (Sprint 18 plan §2.2 inventory) | n/a | Mis-attributed as "no explicit commit captured"; corrected by PROMPT 1324 |
| PROMPT 1292 | 2026-05-18 | `origin/main@1345c6b` | (Sprint 18 plan main-land) | Sprint 18 plan draft landed on `origin/main` |
| PROMPT 1301 | 2026-05-18 | `origin/main@1345c6b` | (Sprint 18 activation tip) | Sprint 18 activated; this row included in 6-row Should Have set as verify-only candidate |
| PROMPT 1320 | 2026-05-18 | (Sprint 18 QA plan main-land) | n/a | Sprint 18 QA plan authored; this row classified row 10 (Integration; AC1 grep gate BLOCKING) |
| PROMPT 1324 | 2026-05-19 | (`/story-readiness` batch) | n/a | Row 6 verdict `READY_FOR_STORY_DONE`; implementation captured at `8eeb94e`; test-path mismatch flagged as paperwork-only advisory |
| PROMPT 1331 | 2026-05-19 | `origin/main@4940a7b` | (this `/story-done` paperwork commit) | `/story-done` paperwork closure: Status Draft → Done; AC1..AC13 PASS (AC7 PASS-STRUCTURAL + ADVISORY-EVIDENCE-DEFERRED); test-path advisory recorded |

### Conditions carried forward unchanged

- Sprint 18 disposition `active` (UNCHANGED; Sprint 18 NOT closed-out by PROMPT 1331).
- Stage `Polish` (UNCHANGED; `production/stage.txt` NOT modified).
- PROMPT 761 `Polish->Release` gate-check FAIL preserved at `production/gate-checks/gate-polish-release-2026-05-12.md`; NO retry attempted by PROMPT 1331.
- `S8-QA-001-W1` OPEN preserved (two-client GAME_OVER closure remains gap; Sprint 13 story 017 AC12 forbid-auto-closure preserved through Sprint 13/14/15/16/17/18).
- `QA-COND-0005` Standard-tier accessibility remains accepted-risk (friend-game scope only; this row is a **precondition** for Standard-tier accessibility, not closure).
- `QA-COND-0006` playtest / fun-hypothesis validation remains accepted-risk / deferred.
- `PAW-TD-*-a` placeholder-art accept-risk preserved across PAW-002..PAW-006.
- `TQ-S12-C1..C7` preserved verbatim. `TQ-S12-C7` explicitly NOT closed by PROMPT 1331.
- `S11-HUD-TIMER-EYEBALL-VISUAL-001` Sprint 13 → 14 → 15 → 16 → 17 → 18 human-operator-blocked carry preserved; no LLM `/story-done` authorised on that row.
- `S17-UI-HUD-OPP-MANA-CLEANUP-001` parent-row paperwork gap preserved; Sprint 18 does NOT silently close it.
- Sprint 17 disposition `closed-with-conditions` preserved (PROMPT 1279 + PROMPT 1289 / 1291 closeout evidence reconcile).
- Sprint 10..16 dispositions preserved verbatim.
- PROMPT 1054 P1 UI snapshot retest BLOCKED-HUMAN preserved.
- 24 PROMPT 1022 QA snapshot audit findings preserved as report-only inputs to future story authoring; none are Sprint 18 active rows.
- All PROMPT 1076 / 1077 findings outside concrete repairs already on `origin/main` preserved.

### Explicitly NOT claimed by PROMPT 1331

- Public release readiness; release-candidate readiness; full game completion.
- Broad / Standard-tier accessibility completion (`QA-COND-0005` accept-risk preserved; this row is a **precondition** only).
- Playtest / fun-hypothesis validation (`QA-COND-0006` accept-risk preserved).
- Full playable-client manual QA.
- Two-client GAME_OVER closure (`S8-QA-001-W1` remains OPEN).
- Final-art / asset-production completion (`PAW-TD-*-a` accept-risk preserved).
- `Polish->Release` gate-check retry (PROMPT 761 FAIL preserved with NO retry).
- Stage advance from `Polish` to `Release` (`production/stage.txt` NOT modified).
- Closure of `S11-HUD-TIMER-EYEBALL-VISUAL-001` (human-operator-blocked carry; no LLM `/story-done` authorised).
- Closure of `S17-UI-HUD-OPP-MANA-CLEANUP-001` parent-row paperwork gap.
- Closure of any other Sprint 18 active row (the other 11 rows preserved as their current status).
- Closure of any PROMPT 1022 / 1076 / 1077 finding outside concrete repairs already on `origin/main`.

`1331: S18-UI-SETTINGS-PANEL-FLEX-RELAYOUT-001: DONE`
