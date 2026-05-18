# Story 020: S18-UI-PLAY-AREA-CONTAINER-001 -- PlayArea Flex Container + Strip-Budget Contract

> **Epic**: UI Clean-Pass
> **Story ID**: S18-UI-PLAY-AREA-CONTAINER-001
> **Status**: Draft -- future Sprint 18 candidate; NOT activated by this authoring run
> **Layer**: Presentation -- design-token + per-surface consumer migration (shop_auction + hand)
> **Type**: Tech Debt -- structural refactor (root-cause RC-1)
> **Sprint**: Future Sprint 18 candidate per `reports/PROMPT-1180-global-ui-layout-system-deep-audit.md` §6 Lane A.
> **Authored**: 2026-05-18 by PROMPT 1189
> **Authoring source-of-truth**: `origin/main@efb698e`
> **Estimated effort**: ~1.0d
> **Source audit**: PROMPT 1180 §2 RC-1, §6 Lane A, §7 (PROMPT 1190 candidate)

---

## Status / No-Claim Banner

Future Sprint 18 candidate. **No sprint activated by this authoring run.** PROMPT 1189 does NOT modify sprint / stage / session-state / QA / code / Cargo / CI files.

No claim on release readiness, `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a`, gate-check retry, stage advance, or closure of any audit finding outside RC-1 / Lane A. Sprint 17 and earlier dispositions preserved verbatim.

## Problem Class / Prevention Target

**Defect class** (RC-1): the four major surface families (HUD top, shop / auction / draft panels, hand fan + placement action panel, FooterBar) all paint at `z_layers::UI_BASE` with `position_type: Absolute` and hand-computed `(top, bottom)` literals. No container enforces a vertical budget. Per-surface offsets disagree (shop `bottom: 0, height: 260`; auction `top: 80, bottom: 140`; footer `bottom: 100, height: 96`) producing the catalogued overlaps S-01, S-02, S-06, F-01, F-03.

**Prevention target**: introduce a `PlayArea` flex parent owning `viewport_height − HEADER_BAR_HEIGHT − FOOTER_BAR_HEIGHT − HAND_BAR_HEIGHT` middle band. Every in-session panel parents into `PlayArea`; strip primitives stay viewport-edge anchored.

## 1180 Lane Coverage

Owns Lane A:

> | **A — PlayArea container + strip-budget contract** | `client/src/ui/design_tokens/play_area.rs` (NEW); patch consumers `shop_auction/mod.rs::{bottom_panel_node, auction_panel_node, footer_node, toast_node}`, `hand/mod.rs::placement_action_panel_node` | `tests/integration/ui_clean_pass/play_area_budget_test.rs` (NEW) | **P0** |

Pre-condition for Lane J (story 026).

## Context

- `client/src/ui/shop_auction/mod.rs:5075-5084` — `bottom_panel_node()` (S-01).
- `client/src/ui/shop_auction/mod.rs:5364-5374` — `auction_panel_node()` (S-02).
- `client/src/ui/shop_auction/mod.rs:5642-5651` — `footer_node()` (S-06).
- `client/src/ui/shop_auction/mod.rs:5665-5674` — `toast_node()` (S-07).
- `client/src/ui/hand/mod.rs::placement_action_panel_node` — no `height/max_height/overflow` (F-03).
- `client/src/ui/design_tokens/strips.rs` — strip primitives module; `PlayArea` is a sibling abstraction.

**ADR-021 / ADR-002**: no system-set / authority change. Layout-only.

**Engine / skills**: Bevy 0.18; `liv-bevy-018` on every `.rs` edit.

### Control Manifest Rules

- Required: `client/src/ui/design_tokens/play_area.rs` (NEW) — `PlayArea` marker + builder `play_area_node()` producing `Node { position_type: Absolute, top: HEADER_BAR_HEIGHT_PX, left: 0, right: 0, bottom: HAND_BAR_HEIGHT_PX + FOOTER_BAR_HEIGHT_PX, display: Flex, flex_direction: Column }`.
- Required: migrate the five consumer functions to parent into `PlayArea` with inner-flex sizing.
- Required: strip primitives unchanged (sibling, not child).
- Forbidden: `Overflow::visible()` on `PlayArea`.
- Forbidden: editing sprint / stage / session-state / QA / gate-check files.

## Story Classification

**Integration** (cross-module + consumer migration + new integration test).

## Dependencies and Parallelism

| Sibling | Parallel-safe? | Notes |
|---|---|---|
| Story 021 (Lane B) | YES | Test-only. |
| Stories 022 / 023 / 024 / 027 | YES | Disjoint. |
| Story 025 (Lane I) | NO | After Wave 2. |
| Story 026 (Lane J) | NO | Consumes `PlayArea`. |
| Active PROMPT 1182 (shop/auction) | PARTIAL | Same file; serialise. |
| Active PROMPTs 1178 / 1183 / 1187 | YES | Disjoint. |

## Acceptance Criteria

- [ ] AC1 -- `PlayArea` module exists with marker + builder per Control Manifest.
- [ ] AC2 -- Shop panel parents into `PlayArea`; viewport-anchored literal removed.
- [ ] AC3 -- Auction panel parents into `PlayArea` (was `top: 80, bottom: 140`).
- [ ] AC4 -- Shop footer parents into `PlayArea` (was `bottom: 100, height: 96`).
- [ ] AC5 -- Shop toast parents into `PlayArea` (may stay absolute within `PlayArea`).
- [ ] AC6 -- Placement action panel parents into `PlayArea` with `max_height` + `overflow::scroll_y()` OR `flex_wrap` OR pagination per §5 C-3.
- [ ] AC7 -- Integration test `play_area_budget_test.rs` asserts budget edges + consumer containment at 1280×720 / 1366×768 / 1920×1080.
- [ ] AC8 -- Strip primitives unchanged.
- [ ] AC9 -- Zero changes under `server/`, `shared/`, `tests/integration/server/`.
- [ ] AC10 -- `liv-bevy-018` activated.
- [ ] AC11 -- Cargo resource policy applied (`CARGO_TARGET_DIR=D:\_DEV\cargo-target\ccgs-msvc`, `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0`, `CARGO_INCREMENTAL=0`, `RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'`).
- [ ] AC12 -- No accept-risk closure.
- [ ] AC13 -- Sprint disposition preserved.
- [ ] AC14 -- Worker branch scope contained; slug `work/s18-ui-play-area-container`.

## Implementation Notes

### Owned files

| Path | Expected change |
|------|-----------------|
| `client/src/ui/design_tokens/play_area.rs` (NEW) | `PlayArea` marker + builder. |
| `client/src/ui/design_tokens/mod.rs` | Add module + re-export. |
| `client/src/ui/design_tokens/strips.rs` | Re-export strip-height constants if not already public. |
| `client/src/ui/shop_auction/mod.rs` | Migrate 4 consumers. |
| `client/src/ui/hand/mod.rs` | Migrate placement action panel. |
| `client/src/ui/mod.rs` | Spawn `PlayArea` on `OnEnter(InSession)`. |
| `tests/integration/ui_clean_pass/play_area_budget_test.rs` (NEW) | AC7. |

### Forbidden files

- `client/src/ui/lobby.rs` (PROMPT 1178), `client/src/ui/hud/**` (PROMPT 1183).
- `server/`, `shared/`, sprint / stage / session-state / QA / gate-check files, ADRs.

## Worker Contract

1. Worktree slug `work/s18-ui-play-area-container`.
2. Read this story + PROMPT 1180 §2 RC-1 + §6 Lane A.
3. Activate `liv-bevy-018`.
4. Cargo resource policy before every Cargo command.
5. Targeted tests only.
6. Push worker branch only.

Build gate scoped to owned files + new test bin per the user's `feedback_build_gate_isolated_files_only` rule.
