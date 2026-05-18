# Story 024: S18-UI-SETTINGS-PANEL-FLEX-RELAYOUT-001 -- Settings Panel Flex Re-Layout + UI-Scale Invariant

> **Epic**: UI Clean-Pass
> **Story ID**: S18-UI-SETTINGS-PANEL-FLEX-RELAYOUT-001
> **Status**: Draft -- future Sprint 18 candidate; NOT activated by this authoring run
> **Layer**: Presentation -- settings UI surface (`client/src/ui/settings/mod.rs` only)
> **Type**: Tech Debt -- per-surface re-layout (root-cause RC-2; per-surface O-01)
> **Sprint**: Future Sprint 18 candidate per PROMPT 1180 §6 Lane F.
> **Authored**: 2026-05-18 by PROMPT 1189
> **Authoring source-of-truth**: `origin/main@efb698e`
> **Estimated effort**: ~0.5d
> **Source audit**: PROMPT 1180 §1.5 O-01, §2 RC-2, §6 Lane F (PROMPT 1195 candidate)
> **Active impl PROMPT**: PROMPT 1187. If 1187 lands first, this story may close via `/story-done`.

---

## Status / No-Claim Banner

Future Sprint 18 candidate. **No sprint activated.** No claim on release readiness, `QA-COND-0005` Standard-tier completion (this row is a **precondition**, not closure), `QA-COND-0006`, `PAW-TD-*-a`, gate-check retry, stage advance, or closure of any audit finding outside Lane F / O-01.

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

- [ ] AC1 -- No absolute-positioned child offsets: `grep -n "PositionType::Absolute" client/src/ui/settings/mod.rs` reports at most ONE occurrence (panel root).
- [ ] AC2 -- Panel scroll + max-height: `max_height: Val::Percent(92.0)` + `overflow: Overflow::scroll_y()`.
- [ ] AC3 -- UI-scale 75% keeps children inside panel bounds.
- [ ] AC4 -- UI-scale 150% no dead zones: `content_size` ≈ `inner_size` − padding within 4 px.
- [ ] AC5 -- 1280×720 keeps primary controls reachable (viewport OR scroll).
- [ ] AC6 -- 1366×768 keeps primary controls reachable.
- [ ] AC7 -- Visual hierarchy preserved; before/after screenshots in `production/qa/evidence/sprint-18-settings-flex-relayout/`.
- [ ] AC8 -- Settings functionality preserved (round-trip identical).
- [ ] AC9 -- `liv-bevy-018` activated.
- [ ] AC10 -- Cargo resource policy applied.
- [ ] AC11 -- No accept-risk closure; `QA-COND-0005` row-precondition note explicit.
- [ ] AC12 -- Sprint disposition preserved.
- [ ] AC13 -- Worker branch scope contained; slug `work/s18-ui-settings-panel-flex-relayout`.

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
