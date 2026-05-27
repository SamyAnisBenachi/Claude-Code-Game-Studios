# Story 022: S18-UI-CARD-ART-AND-LABEL-STRIP-001 -- Card-Art `NodeImageMode::Fit` + Opaque Label-Strip Primitive

> **Epic**: UI Clean-Pass
> **Story ID**: S18-UI-CARD-ART-AND-LABEL-STRIP-001
> **Status**: Done — closed by PROMPT 1715 on origin/main@ac55edfc (PROMPT 1726 tip)
> **Layer**: Presentation -- card-slot primitive extension + per-surface consumer migration (hand + shop_auction)
> **Type**: Tech Debt -- structural primitive extension (root-cause RC-3)
> **Sprint**: Sprint 18 (Should Have)
> **Authored**: 2026-05-18 by PROMPT 1189
> **Authoring source-of-truth**: `origin/main@efb698e`
> **Completed**: 2026-05-28 by PROMPT 1715
> **Estimated effort**: ~0.5d
> **Source audit**: PROMPT 1180 §2 RC-3, §6 Lane C (PROMPT 1192 candidate); cross-refs F-02, S-04, UI-1129-05 / -09 / -02.

---

## Status / No-Claim Banner

**Done.** Closed by PROMPT 1715 paperwork on origin/main@ac55edfc. Implementation on main via PROMPT 1348 worker (26bc1204) + PROMPT 1373 integration (fbb1c697) + PROMPT 1381/1389 main-land. AC1–AC14 all PASS. No claim on release readiness, `QA-COND-*`, `PAW-TD-*-a`, gate-check retry, stage advance, or closure of any audit finding outside RC-3 / Lane C.

## Problem Class / Prevention Target

**Defect class** (RC-3): every card-art `ImageNode` is wired without `image_mode: NodeImageMode::Auto` or `Fit`. Bevy 0.18 default `NodeImageMode::Stretch` produces horizontally banner-stretched portraits (UI-1129-05). Story 009's `card_slot_node` defines outer dimensions but not inner image-fit. Auction featured card overlays title / rarity / cost / stat directly on chrome (UI-1129-02 / S-04) because no opaque label strip primitive exists.

**Prevention target**: extend `card_slot_node` to spawn `CardSlotArtImage` (`NodeImageMode::Fit`) + sibling `CardSlotLabelStrip` (opaque background). Promote PROMPT 1117 chrome-preservation rule from behavioural to structural.

## 1180 Lane Coverage

Owns Lane C:

> | **C — Card-art image-mode policy + label-strip primitive** | `client/src/ui/design_tokens/card_slot.rs` (extend); `client/src/asset_wiring.rs` (READ-only); patch consumers `hand/mod.rs::sync_hand_fan_card_art_system`, `shop_auction/mod.rs::handle_draft_offering_system`, `shop_auction/mod.rs::auction_featured_card_node` | `tests/integration/ui_clean_pass/card_art_aspect_fit_test.rs` (NEW), `tests/integration/shop_auction_ui/auction_featured_art_binding_test.rs` (NEW) | **P0** |

## Context

- `client/src/ui/hand/mod.rs:3377-3384` — `HandCardFrame` spawn; F-02.
- `client/src/ui/shop_auction/mod.rs:3484` — `auction_featured_card_node`; S-04.
- `client/src/asset_wiring.rs:594` — `apply_card_display_art`; PROMPT 1117 chrome preservation behavioural.
- `client/src/ui/design_tokens/card_slot.rs` — story 009 primitive.

**GDD / ADR**: `design/gdd/card-data-pool.md`, `design/gdd/shop-auction-ui.md` cross-cut; no body change. ADR-021 / 013 / 019 preserved.

**Engine / skills**: Bevy 0.18; `liv-bevy-018`; `NodeImageMode` canonical.

### Control Manifest Rules

- Required: extend `card_slot_node` to spawn `CardSlotArtImage` (Fit) + `CardSlotLabelStrip` (opaque from `SURFACE_ELEVATED`).
- Required: migrate three consumer spawn sites.
- Required: label strip satisfies §5 C-6 (`min_width`, `Overflow::clip_x()` OR ellipsis).
- Forbidden: changing `CardSlotKind` variants or outer dimensions.
- Forbidden: editing lobby, HUD, settings, server, shared.

## Story Classification

**Integration**.

## Dependencies and Parallelism

| Sibling | Parallel-safe? | Notes |
|---|---|---|
| Stories 020 / 021 / 023 / 024 / 026 / 027 | YES | Disjoint. |
| Story 025 (Lane I) | NO | After Wave 2. |
| Active PROMPT 1178 | YES | Distinct class portrait fix. |
| Active PROMPT 1182 | PARTIAL | Same file; serialise. |
| Active PROMPTs 1183 / 1187 | YES | Different surfaces. |

Prerequisites: story 009 (Done); 017 / 018 (Draft S17) preferred first.

## Acceptance Criteria

- [x] AC1 -- Markers `CardSlotArtImage` + `CardSlotLabelStrip` defined and exported; `card_slot_node` spawns both as children.
- [x] AC2 -- Card-art `ImageNode` carries `image_mode: NodeImageMode::Auto` (Bevy 0.18 has no `NodeImageMode::Fit`; `Auto` is the canonical equivalent per PROMPT 1348 commit message); default `Stretch` forbidden.
- [x] AC3 -- Label strip carries opaque `BackgroundColor` (alpha 0.92 ≥ 0.85 floor), `min_width` = 24 px clamp, `Overflow::clip_x()`.
- [x] AC4 -- `sync_hand_fan_card_art_system` migrated (F-02 / UI-1129-05 resolved).
- [x] AC5 -- `handle_draft_offering_system` migrated.
- [x] AC6 -- `auction_featured_card_node` / `spawn_auction_contents` migrated (S-04 / UI-1129-02 resolved).
- [x] AC7 -- Chrome-preservation rule structural (carried by primitive); `asset_wiring.rs` stays READ-only.
- [x] AC8 -- `card_art_aspect_fit_test.rs` (NEW) 8 / 8 PASS (PROMPT 1373 + 1381 + 1389 verification).
- [x] AC9 -- `auction_featured_art_binding_test.rs` (NEW) 8 / 8 PASS (PROMPT 1373 + 1381 + 1389 verification).
- [x] AC10 -- No accept-risk closure.
- [x] AC11 -- `liv-bevy-018` activated (per PROMPT 1348 worker contract).
- [x] AC12 -- Cargo resource policy applied (PROMPT 1373 + 1381 verification sections).
- [x] AC13 -- Sprint 18 active + stage Polish unchanged (preserved across PROMPT 1348 / 1373 / 1381 / 1389).
- [x] AC14 -- Worker branch slug `work/s18-ui-card-art-and-label-strip` used.

## Implementation Notes

### Owned files

| Path | Expected change |
|------|-----------------|
| `client/src/ui/design_tokens/card_slot.rs` | Extend primitive — Fit art + opaque label strip. |
| `client/src/ui/hand/mod.rs` | Migrate `sync_hand_fan_card_art_system`. |
| `client/src/ui/shop_auction/mod.rs` | Migrate `handle_draft_offering_system` + `auction_featured_card_node`. |
| `client/src/asset_wiring.rs` | READ-only unless AC7 requires moving the chrome rule. |
| `tests/integration/ui_clean_pass/card_art_aspect_fit_test.rs` (NEW) | AC8. |
| `tests/integration/shop_auction_ui/auction_featured_art_binding_test.rs` (NEW) | AC9. |

### Forbidden files

- `client/src/ui/lobby.rs`, `client/src/ui/hud/**`, `client/src/ui/settings/**`.
- `server/`, `shared/`, `tests/integration/server/`.
- Sprint / stage / session-state / QA / gate-check files, ADRs.

## Worker Contract

1. Worktree slug `work/s18-ui-card-art-and-label-strip`.
2. Read story + PROMPT 1180 §2 RC-3 + §6 Lane C.
3. Activate `liv-bevy-018`.
4. Cargo resource policy env vars.
5. Targeted tests only.
6. Push worker branch only.
