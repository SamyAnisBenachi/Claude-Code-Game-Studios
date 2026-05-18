# Story 021: S18-UI-VIEWPORT-INVARIANT-LIVE-HARNESS-001 -- Live-Spawn Viewport Invariant Test Harness

> **Epic**: UI Clean-Pass
> **Story ID**: S18-UI-VIEWPORT-INVARIANT-LIVE-HARNESS-001
> **Status**: Draft -- future Sprint 18 candidate; NOT activated by this authoring run
> **Layer**: Test infrastructure -- viewport invariant harness rewrite
> **Type**: Tech Debt -- structural refactor (root-cause RC-5)
> **Sprint**: Future Sprint 18 candidate per PROMPT 1180 §6 Lane B.
> **Authored**: 2026-05-18 by PROMPT 1189
> **Authoring source-of-truth**: `origin/main@efb698e`
> **Estimated effort**: ~1.0d
> **Source audit**: PROMPT 1180 §2 RC-5, §3, §5 C-7, §6 Lane B (PROMPT 1191 candidate)
> **Active impl PROMPT**: PROMPT 1185. If 1185 lands first, this story may close via `/story-done`.

---

## Status / No-Claim Banner

Future Sprint 18 candidate. **No sprint activated.** No claim on release readiness, `QA-COND-*`, `PAW-TD-*-a`, gate-check retry, stage advance, or closure of any audit finding outside RC-5 / Lane B. Earlier-sprint dispositions preserved.

## Problem Class / Prevention Target

**Defect class** (RC-5): existing `ui_viewport_invariants_test.rs` + `helpers/ui_viewport.rs` + `fixtures/ui_viewport_baseline.rs` harness asserts hand-authored tuples rather than live `Node` instances. Fixture "Provisional shop panel dimensions" diverge from runtime; harness passes while live UI has six catalogued overlaps. `assert_anchor_stability` walks the baseline (tautology).

**Prevention target**: live-spawn harness instantiating real UI plugin roots, driving ≥3 frames per viewport, querying actual `(GlobalTransform, ComputedNode)` against the camera viewport.

## 1180 Lane Coverage

Owns Lane B:

> | **B — Live-spawn viewport invariant harness (RC-5 fix)** | `tests/integration/helpers/ui_viewport.rs` (REWRITE); new `tests/integration/ui_viewport_live_test.rs`; delete `fixtures/ui_viewport_baseline.rs` (or mark obsolete) | Self | **P0** |

Single highest-leverage fix.

## Context

### Existing surface

- `tests/integration/ui_viewport_invariants_test.rs` — fixture harness.
- `tests/integration/helpers/ui_viewport.rs:253-270` — writes fixture tuple verbatim.
- `tests/integration/fixtures/ui_viewport_baseline.rs:80-94` — provisional shop dimensions diverge from runtime.

### Viewport matrix (PROMPT 1180 §5 C-1)

1280×720, 1366×768, 1920×1080, 1920×1200, 1280×960, 3840×2160, 2560×1080.

### Engine / skills

- Bevy 0.18. `liv-bevy-018`. `ComputedNode::stack_index` / `content_size`; `ui_layout_system` in `PostUpdate`.

### Control Manifest Rules

- Required: spawn real UI plugin roots (or smallest production-faithful subset; document blockers).
- Required: drive ≥3 frames; query `(GlobalTransform, ComputedNode)`.
- Required: assert (i) primary-CTA bounds on-screen; (ii) no overlap/clip outside declared allowances; (iii) strip heights match token contract; (iv) anchor stability walks the World.
- Required: blocker reporting if production plugin spawn fails — `production/qa/evidence/sprint-18-ui-viewport-live-harness/blockers.md` with file:line evidence. Hand-authored fallback bounds forbidden.
- Forbidden: editing production UI code.

## Story Classification

**Integration** (test infrastructure).

## Dependencies and Parallelism

| Sibling | Parallel-safe? | Notes |
|---|---|---|
| Story 020 (Lane A) | YES | Test-only edits. |
| Stories 022 / 023 / 024 / 025 / 026 / 027 | YES | Disjoint. |
| Active PROMPTs 1178 / 1182 / 1183 / 1187 / 1188 | YES | Don't own this test file. |
| Active PROMPT 1185 | DUPLICATE | Impl worker; may land first. |

## Acceptance Criteria

- [ ] AC1 -- Canonical 7-viewport matrix per §5 C-1.
- [ ] AC2 -- Live `(GlobalTransform, ComputedNode)` query, not fixture tuples. Hand-authored fixture path removed or `#[deprecated]`+unused.
- [ ] AC3 -- Real UI plugin spawn (`LobbyUiPlugin`, `HudPlugin`, `HandUiPlugin`, `ShopAuctionUiPlugin`, `PresentationPlugin` or smallest faithful subset; document blockers).
- [ ] AC4 -- Layout pass driven (≥3 frames) before assert.
- [ ] AC5 -- No-overlap invariant catches ≥4 of catalogued S-01, S-02, S-06, F-01, F-03, O-01 on `origin/main` (or documents which suppress to baseline if A / F / J landed).
- [ ] AC6 -- Primary-CTA on-screen invariant: `0 ≤ x, y, (x+w) ≤ vw, (y+h) ≤ vh` for every cell.
- [ ] AC7 -- Strip-height contract asserted against `design_tokens/strips.rs` constants to within 1 px.
- [ ] AC8 -- `fixtures/ui_viewport_baseline.rs` DELETED OR `#[deprecated]` with documented rationale.
- [ ] AC9 -- Blocker reporting if plugin spawn fails; no hand-authored fallback bounds.
- [ ] AC10 -- `liv-bevy-018` activated.
- [ ] AC11 -- Cargo resource policy applied.
- [ ] AC12 -- Zero changes under `client/src/**`, `server/src/**`, `shared/src/**`.
- [ ] AC13 -- No accept-risk closure.
- [ ] AC14 -- Sprint disposition preserved.
- [ ] AC15 -- Worker branch scope contained; slug `work/s18-ui-viewport-invariant-live-harness`.

## Implementation Notes

### Owned files

| Path | Expected change |
|------|-----------------|
| `tests/integration/helpers/ui_viewport.rs` | REWRITE — live query helpers. |
| `tests/integration/ui_viewport_live_test.rs` (NEW) | Harness driver. |
| `tests/integration/ui_viewport_invariants_test.rs` | DELETE or thin re-export. |
| `tests/integration/fixtures/ui_viewport_baseline.rs` | DELETE or `#[deprecated]`. |
| `client/Cargo.toml` | Only if a new `[[test]]` entry needed. |
| `production/qa/evidence/sprint-18-ui-viewport-live-harness/blockers.md` (NEW, conditional) | AC9. |

### Forbidden files

- All production code, server, shared.
- Sprint / stage / session-state / QA-plan / smoke / gate-check files (except AC9 evidence path).
- ADRs, launcher / tooling.

## Worker Contract

1. Worktree slug `work/s18-ui-viewport-invariant-live-harness`.
2. Read story + PROMPT 1180 §2 RC-5 + §3.
3. Activate `liv-bevy-018`.
4. Cargo resource policy env vars.
5. Targeted viewport harness tests only.
6. Push worker branch only.
