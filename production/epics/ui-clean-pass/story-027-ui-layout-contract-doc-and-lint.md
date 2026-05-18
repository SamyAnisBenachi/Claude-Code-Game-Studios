# Story 027: S18-UI-LAYOUT-CONTRACT-DOC-AND-LINT-001 -- Global UI Layout Contract Doc + Button-vs-Chip Lint

> **Epic**: UI Clean-Pass
> **Story ID**: S18-UI-LAYOUT-CONTRACT-DOC-AND-LINT-001
> **Status**: Draft -- future Sprint 18 candidate; NOT activated by this authoring run
> **Layer**: UX documentation + lint-style integration test
> **Type**: UX -- contract authoring + lint enforcement (RC-1..RC-5 governance)
> **Sprint**: Future Sprint 18 candidate per PROMPT 1180 §6 Lane K.
> **Authored**: 2026-05-18 by PROMPT 1189
> **Authoring source-of-truth**: `origin/main@efb698e`
> **Estimated effort**: ~0.3d
> **Source audit**: PROMPT 1180 §5 (C-1..C-7), §6 Lane K (PROMPT 1200 candidate)
> **Active impl PROMPT**: PROMPT 1188. If 1188 lands first, this story may close via `/story-done`.

---

## Status / No-Claim Banner

Future Sprint 18 candidate. **No sprint activated.** No claim on release readiness, `QA-COND-*`, `PAW-TD-*-a`, gate-check retry, stage advance, or closure of any audit finding outside Lane K / C-1..C-7.

## Problem Class / Prevention Target

**Defect class** (§5): the audit catalogues 5 RC root causes and 7 contract sections (C-1..C-7) but no enforceable doc pins these invariants. Without a doc, the next UI story regresses; without a lint, the regression ships unnoticed.

**Prevention target**: author `docs/ux/global-ui-layout-contract.md` (NEW) covering C-1..C-7; author `tests/integration/ui_clean_pass/button_vs_chip_lint_test.rs` (NEW) catching common violations. Lint is conservative — per-file allowlist baseline with TODO messaging.

## 1180 Lane Coverage

Owns Lane K:

> | **K — Global UI layout contract doc + lint** | `docs/ux/global-ui-layout-contract.md` (NEW); `tests/integration/ui_clean_pass/button_vs_chip_lint_test.rs` (NEW) | Self | **P2** |

## Context

- `docs/ux/global-ui-design-spec.md` — story 007 (Done; PROMPT 922).
- `docs/ux/ui-clean-pass-roadmap.md` — story 001 (Done; PROMPT 840).
- `docs/ux/ui-architecture-split-sequencing.md` — story 015 (Draft); cross-link if landed.

**GDD / ADR**: no body change; ADR-021 / ADR-002 cross-referenced.

**Engine / skills**: Bevy 0.18; `liv-bevy-018` for the lint test. Doc is Markdown.

### Control Manifest Rules

- Required: doc covers all seven C-1..C-7 sections.
- Required: lint covers (i) `Button` markers without `BackgroundColor`; (ii) `BackgroundColor` markers without `Interaction` with sibling `Text ≥ typography::BODY`; (iii) `Overflow::visible()` sites without `// AC: <ticket>` justification.
- Required: lint conservative — per-file allowlist baseline; lint PASS on current `origin/main`.
- Forbidden: editing production UI code (lint read-only).
- Forbidden: editing `docs/ux/global-ui-design-spec.md` beyond a single cross-link line.

## Story Classification

**Documentation + Integration**.

## Dependencies and Parallelism

| Sibling | Parallel-safe? | Notes |
|---|---|---|
| Stories 020 / 021 / 022 / 023 / 024 / 025 / 026 | YES | Disjoint. |
| Active PROMPTs 1178 / 1182 / 1183 / 1187 | YES | Disjoint. |
| Active PROMPT 1181 (foundation primitives) | YES | Cross-reference 1181's spec note if present. |
| Active PROMPT 1188 | DUPLICATE | Impl worker; may land first. |

## Acceptance Criteria

- [ ] AC1 -- Doc exists with all seven C-1..C-7 labeled sections.
- [ ] AC2 -- Viewport matrix table verbatim from §5 C-1.
- [ ] AC3 -- C-4 section has ≥1 button example (lobby confirm) + ≥1 chip example (HUD pill) with file:line pointers.
- [ ] AC4 -- Lint test exists and `cargo test -p client --test button_vs_chip_lint_test` exits code 0 under Cargo resource policy.
- [ ] AC5 -- Lint catches `Button` without `BackgroundColor` (synthetic fixture).
- [ ] AC6 -- Lint catches status-chip-styled-as-button (`BackgroundColor` + sibling `Text ≥ BODY` + no `Interaction`).
- [ ] AC7 -- Lint catches `Overflow::visible()` without `// AC:` within ±2 lines; strip-primitive allowlist documented inline.
- [ ] AC8 -- Allowlist per-file with TODO Lane ticket pointers; lint PASS on current `origin/main`; new unallowlisted violations FAIL.
- [ ] AC9 -- `docs/ux/global-ui-design-spec.md` gets one cross-link line; body otherwise unchanged.
- [ ] AC10 -- Contract cross-references roadmap and (conditionally) sequencing notes.
- [ ] AC11 -- `liv-bevy-018` activated for lint test.
- [ ] AC12 -- Cargo resource policy applied.
- [ ] AC13 -- Zero changes under `client/src/**`, `server/src/**`, `shared/src/**`.
- [ ] AC14 -- No accept-risk closure.
- [ ] AC15 -- Sprint disposition preserved.
- [ ] AC16 -- Worker branch scope contained; slug `work/s18-ui-layout-contract-doc-and-lint`.

## Implementation Notes

### Owned files

| Path | Expected change |
|------|-----------------|
| `docs/ux/global-ui-layout-contract.md` (NEW) | Seven C-1..C-7 sections + cross-links. |
| `tests/integration/ui_clean_pass/button_vs_chip_lint_test.rs` (NEW) | Lint with per-file allowlist baseline. |
| `docs/ux/global-ui-design-spec.md` | Single cross-link line. |
| `client/Cargo.toml` | Only if a new `[[test]]` entry needed. |

### Forbidden files

- All production UI code, server, shared.
- Other `docs/ux/*` files outside the single cross-link.
- ADRs, sprint / state / QA / gate-check files.

## Worker Contract

1. Worktree slug `work/s18-ui-layout-contract-doc-and-lint`.
2. Read story + PROMPT 1180 §5 (C-1..C-7) + §6 Lane K + design spec.
3. Activate `liv-bevy-018` for lint test.
4. Cargo resource policy env vars.
5. Targeted lint test only.
6. Push worker branch only.
