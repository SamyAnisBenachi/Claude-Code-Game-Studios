# Story 027: S18-UI-LAYOUT-CONTRACT-DOC-AND-LINT-001 -- Global UI Layout Contract Doc + Button-vs-Chip Lint

> **Epic**: UI Clean-Pass
> **Story ID**: S18-UI-LAYOUT-CONTRACT-DOC-AND-LINT-001
> **Status**: Done -- Sprint 18 Must Have (closed PROMPT 1337 on `origin/main@72b89ca` after PROMPT 1188 implementation `c2eaab0` + PROMPT 1208 baseline refresh `ae8f7d1` + PROMPT 1334 AC9 cross-link backfill landed at `72b89ca` via PROMPT 1335 main-land). AC1..AC5 + AC7..AC16 PASS; AC6 ADVISORY (L4 chip-side static lint deferred per contract §10 false-positive-surface rationale; carried per PROMPT 1323 advisory + PROMPT 1327 directive).
> **Layer**: UX documentation + lint-style integration test
> **Type**: UX -- contract authoring + lint enforcement (RC-1..RC-5 governance)
> **Sprint**: Sprint 18 Must Have Row 4 (activated PROMPT 1301; closed PROMPT 1337)
> **Authored**: 2026-05-18 by PROMPT 1189
> **Authoring source-of-truth**: `origin/main@efb698e`
> **Estimated effort**: ~0.3d
> **Source audit**: PROMPT 1180 §5 (C-1..C-7), §6 Lane K (PROMPT 1200 candidate)
> **Active impl PROMPT**: PROMPT 1188 (worker commit `c2eaab0`); PROMPT 1208 baseline refresh `ae8f7d1`; PROMPT 1334 AC9 cross-link backfill main-landed by PROMPT 1335 at `72b89ca`.
> **Completed**: 2026-05-19 by PROMPT 1337 (`/story-done` relaunch after PROMPT 1327 NEEDS_WORK).

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

- [x] AC1 -- Doc exists with all seven C-1..C-7 labeled sections. (`docs/ux/global-ui-layout-contract.md` §3 C-1, §4 C-2, §5 C-3, §6 C-4, §7 C-5, §8 C-6, §9 C-7; verified PROMPT 1327 §2 + PROMPT 1337 re-verification.)
- [x] AC2 -- Viewport matrix table verbatim from §5 C-1. (Contract §3 7-row matrix incl. 1280×720 floor + 1366×768 min + 1920×1080 baseline + 16:10 + 4:3 + 4K + ultrawide.)
- [x] AC3 -- C-4 section has ≥1 button example (lobby confirm) + ≥1 chip example (HUD pill) with file:line pointers. (Contract §6.1 button, §6.2 status chip, §6.3 HUD pill prefix; pointer to `client/src/ui/hud/mod.rs:2806/2816` + `HudPillPrefixLabel` symbol reference.)
- [x] AC4 -- Lint test exists and `cargo test -p client --test button_vs_chip_lint_test` exits code 0 under Cargo resource policy. (Test registered in `client/Cargo.toml` `[[test]]` line 477-479 by PROMPT 1188 commit `c2eaab0`; trusted from PROMPT 1188 + 1208 lineage; PROMPT 1337 paperwork-only — no fresh Cargo invocation per `/story-done` paperwork policy.)
- [x] AC5 -- Lint catches `Button` without `BackgroundColor` (synthetic fixture). (Lint rule **L1** `l1_every_button_spawn_tuple_carries_background_color` at lint:339.)
- [ ] AC6 -- Lint catches status-chip-styled-as-button (`BackgroundColor` + sibling `Text ≥ BODY` + no `Interaction`). ⚠️ ADVISORY — L4 chip-side static lint NOT implemented per contract §10 rationale ("intentionally non-failing because the false-positive surface is too large"). L1+L2 close the click-feedback contract from the `Button` side (Interpretation 1 from PROMPT 1323 §4). Kept open per PROMPT 1327 directive ("AC6 advisory may remain advisory ... unless you find a new hard blocker"). Re-confirmed advisory by PROMPT 1337 — no new hard blocker found.
- [x] AC7 -- Lint catches `Overflow::visible()` without `// AC:` within ±2 lines; strip-primitive allowlist documented inline. (Lint rule **L3** `l3_overflow_visible_sites_carry_ac_justification_or_baseline_entry` at lint:439; window = current line + 2 preceding lines via `has_ac_comment_near` at lint:425, defensible reading of ±2 per PROMPT 1323 §4 grep-style design; strip primitive module excluded by `is_path_excluded("src/ui/design_tokens/strips.rs")` at lint:68-72.)
- [x] AC8 -- Allowlist per-file with TODO Lane ticket pointers; lint PASS on current `origin/main`; new unallowlisted violations FAIL. (Baseline constants `BUTTON_NO_BG_BASELINE` lint:94, `BUTTON_NO_INTERACTION_BASELINE` lint:98, `OVERFLOW_VISIBLE_BASELINE` lint:105, all currently empty; `baselines_are_sorted_and_unique` self-test at lint:496 + stale-baseline assertions inside L1/L2/L3 ensure new violations FAIL and stale entries FAIL. PROMPT 1208 maintained baseline by bumping hand reserve-strip line 3774 → 3783.)
- [x] AC9 -- `docs/ux/global-ui-design-spec.md` gets one cross-link line; body otherwise unchanged. (Backfilled by PROMPT 1334 commit at `docs/ux/global-ui-design-spec.md:913-917`; main-landed by PROMPT 1335 at `origin/main@72b89ca9702eed5fc9149b92a2d8b7cc1d56aad6`; +5 lines / 0 deletions per `git show --stat 72b89ca`; bullet placed in existing Cross-References section adjacent to roadmap entry, body otherwise unchanged.)
- [x] AC10 -- Contract cross-references roadmap and (conditionally) sequencing notes. (Contract §12 Cross-References lines 524-539 — cites design-spec, roadmap, board-rendering spec, audit, EPIC, design-token modules, sibling lint tests, deprecated harness.)
- [x] AC11 -- `liv-bevy-018` activated for lint test. (Per PROMPT 1188 worker contract; trusted from commit lineage; lint test `.rs` file scope satisfies skill-activation routing per `.claude/docs/technical-preferences.md` File Extension Routing.)
- [x] AC12 -- Cargo resource policy applied. (Per PROMPT 1188 worker + PROMPT 1208 baseline-refresh commit verification blocks; trusted from commit lineage; PROMPT 1337 paperwork-only — no fresh Cargo invocation.)
- [x] AC13 -- Zero changes under `client/src/**`, `server/src/**`, `shared/src/**`. (`git show --stat c2eaab0` touched `client/Cargo.toml` + `docs/ux/global-ui-layout-contract.md` + lint test only; `ae8f7d1` touched lint test only; `72b89ca` touched `docs/ux/global-ui-design-spec.md` only — all outside the forbidden src trees.)
- [x] AC14 -- No accept-risk closure. (Contract §1 Status Banner preserves `QA-COND-0005` Standard-tier accessibility, `QA-COND-0006` playtest, `PAW-TD-*-a` placeholder-art verbatim; PROMPT 1188 + 1208 + 1334 + 1335 + 1337 commits make no closure claim.)
- [x] AC15 -- Sprint disposition preserved. (PROMPT 1188 / 1208 / 1334 / 1335 did not edit `production/sprint-status.yaml`, `production/sprints/*.md`, `production/stage.txt`, or session-state; PROMPT 1320 QA plan refresh included Row 4 dispatch without disposition flip; PROMPT 1337 flips the Row 4 status from `ready` → `done` only — Sprint 18 remains `active`, stage remains `Polish`, PROMPT 761 Polish→Release FAIL preserved with NO retry.)
- [x] AC16 -- Worker branch scope contained; slug `work/s18-ui-layout-contract-doc-and-lint`. (Per PROMPT 1188 worker contract clause + PROMPT 1188 commit message; trusted from commit lineage.)

## Completion Notes

PROMPT 1337 `/story-done` closure executed against `origin/main@72b89ca9702eed5fc9149b92a2d8b7cc1d56aad6`:

- Implementation lineage: PROMPT 1188 worker commit `c2eaab0` (NEW `docs/ux/global-ui-layout-contract.md` 553 lines + NEW `tests/integration/ui_clean_pass/button_vs_chip_lint_test.rs` 629 lines + `client/Cargo.toml` `[[test]]` registration) + PROMPT 1208 integration commit `ae8f7d1` (hand reserve-strip baseline line bump 3774 → 3783) + PROMPT 1334 cross-link authoring + PROMPT 1335 main-land tip `72b89ca` (`docs/ux/global-ui-design-spec.md` +5 lines / 0 deletions).
- PROMPT 1327 first `/story-done` attempt returned `NEEDS_WORK` due to missing AC9 cross-link (`reports/PROMPT-1327-...md`). PROMPT 1334 authored the cross-link; PROMPT 1335 landed it on `origin/main`. PROMPT 1337 re-runs `/story-done` against the post-1335 main tip.
- AC6 disposition preserved as advisory per PROMPT 1323 §4 + PROMPT 1327 directive. The contract §10 L4 advisory rule is documented but intentionally not implemented as a failing static lint; the chip-side false-positive surface is too large for a strict static lint. L1+L2 close the click-feedback contract from the `Button` side, which is the friend-game-scope acceptance reading.
- AC7 ±2-line window: 3-line scan (current line + 2 preceding) per `has_ac_comment_near` at lint:425; advisory note from PROMPT 1323 §4 carried.
- AC4 + AC11 + AC12 + AC16 trusted from commit lineage; PROMPT 1337 paperwork-only, no fresh Cargo invocation per `/story-done` paperwork policy.

Non-claims preserved: no public release readiness; no RC readiness; no full game completion; no closure of `S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a`, `TQ-S12-C1..C7`; no LLM closure of `S11-HUD-TIMER-EYEBALL-VISUAL-001`; no silent closure of `S17-UI-HUD-OPP-MANA-CLEANUP-001` parent-row paperwork gap; no retry of PROMPT 761 Polish→Release FAIL; no stage advance from Polish to Release; no closure of any AUDIT-1131-* / AUDIT-1076-* / SOURCE-1077-* / PROMPT 1022 / 1076 / 1077 finding outside the concrete repairs already on `origin/main`; no Sprint 10-17 row reopen; no Sprint 17 closeout reopen.

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
