# Story 016: Fan Card Slot Chrome Composition Layout

> **Epic**: Hand UI
> **Status**: In Progress
> **Layer**: Presentation
> **Type**: Integration
> **Manifest Version**: 2026-05-11

## Context

**GDD**: `design/ux/hand-ui.md` (Component Inventory L149–164 — "Fan card slot" enumerates: card art, cost, stats, name, type, slot state)
**Requirement**: `HU-card-slot-chrome-layout`
*(No matching `TR-HU-*` requirement exists in `docs/architecture/tr-registry.yaml`. The component inventory in `design/ux/hand-ui.md` is the authoritative source; a follow-up TR row can be filed by the orchestrator if a registry entry is required for full traceability. `design/ux/visual-art.md` does not exist at the time of writing — measured layout below is a reasonable default, not an authored visual-art spec.)*

**ADR Governing Implementation**: ADR-021 Presentation Layer Architecture (implicit — bevy_ui `Node` only; no world-space sprites; required-components API)

**Engine**: Bevy 0.18 | **Risk**: LOW
**Engine Notes**:
- Each fan slot is an `Absolute`-positioned bevy_ui `Node` parented to `HandFanRoot`. Slot intrinsic box (when promoted to `FanSlotState::Active` by `apply_fan_layout_system`) is `HAND_CARD_DISPLAY_WIDTH_PX × HAND_CARD_DISPLAY_HEIGHT_PX = 96 × 136`.
- Chrome children must declare their own non-zero `Node.width`/`Node.height` AND `position_type: Absolute`. The pre-fix spawn sites used `Node::default()`, which is `width: Val::Auto, height: Val::Auto, position_type: Relative` — at runtime this collapses the chrome to a 0×0 row inside the slot, leaving every glyph invisible even when `apply_fan_layout_system` correctly promotes the slot.
- Percent values resolve against the slot's intrinsic box (positioned ancestor with non-zero `width`/`height`), per the standard CSS / Taffy absolute-positioning rule — same rule that Worker A relied on for the slot's local-to-`fan_root` coord-space fix.

**Control Manifest Rules (Presentation Layer)**:
- Required: UI always bevy_ui `Node` — no world-space sprites for chrome elements.
- Required: All chrome systems remain `in_state(ClientState::InSession)` (no change — chrome is already spawned inside the gated `spawn_hand_ui_entities` system).
- Forbidden: `NodeBundle` — use `Node { .. }` Required Components.
- Forbidden: any logic change in this story — Worker B touches only spawn-site geometry.

---

## Verdict B context (PROMPT 669 diagnostic)

`Finding B v2 V3 Verdict B PROVEN`: 7 chrome children at `client/src/ui/hand/mod.rs` L2566–2618 spawn with `Node::default()` and therefore render at 0×0:

| Marker | Source line (pre-fix) | Asset handle |
|---|---|---|
| `HandCardFrame` | L2569 (spawn block) | `placeholder.card_frame_common` |
| `StatBadgeAtk` | L2577 | `placeholder.stat_badge_atk` |
| `StatBadgeHp` | L2585 | `placeholder.stat_badge_hp` |
| `StatBadgeMp` | L2593 | `placeholder.stat_badge_mp` |
| `StatBadgeAr` | L2601 | `placeholder.stat_badge_ar` |
| `HandRarityIcon` | L2609 | `placeholder.rarity_icon_common` |
| `HandTypeIcon` | L2617 | `placeholder.class_type_icon_neutral` |

PROMPT 669 explicitly deferred Verdict B until Worker A's coord-space fix (`HU-02`) landed so the slot is on-screen first; Worker A integrated at `d9ee107` (per `production/session-state/codex-orchestrator-state.md` wave 8). This story closes Verdict B.

---

## Chosen layout (reasonable defaults — no VA-9 authored spec exists)

All values are percent of the fan slot's local box (`96 × 136`). Stat badges occupy the four corners at `20% × 20%`; rarity / type icons sit centered horizontally at `15% × 15%`. Specific positions chosen to avoid overlap and match standard CCG conventions (mana cost top-left, attack bottom-left, health bottom-right, armor top-right):

| Marker | `position_type` | left | right | top | bottom | width × height |
|---|---|---|---|---|---|---|
| `HandCardFrame` | Absolute | `0%` | — | `0%` | — | `100% × 100%` |
| `StatBadgeMp` | Absolute | `0%` | — | `0%` | — | `20% × 20%` |
| `StatBadgeAr` | Absolute | — | `0%` | `0%` | — | `20% × 20%` |
| `StatBadgeAtk` | Absolute | `0%` | — | — | `0%` | `20% × 20%` |
| `StatBadgeHp` | Absolute | — | `0%` | — | `0%` | `20% × 20%` |
| `HandRarityIcon` | Absolute | `42.5%` | — | `0%` | — | `15% × 15%` |
| `HandTypeIcon` | Absolute | `42.5%` | — | — | `0%` | `15% × 15%` |

Notes:
- `HandTypeIcon` is anchored bottom-center rather than the prompt's literal "top-corner" suggestion because both top corners are already occupied by `StatBadgeMp` / `StatBadgeAr`. Bottom-center is the closest non-overlapping default; future visual-art authoring (VA-9 or successor) may relocate it without changing the AC contract below.
- `HandRarityIcon` left offset `42.5%` = `(100 − 15) / 2` — centers the 15 %-wide icon horizontally inside the slot.
- All positions use `Val::Percent`; the four-corner stat badges leave the unused side as `Val::Auto` so the badge is anchored against the explicit side only.

The three helper functions (`fan_slot_card_frame_node`, `fan_slot_stat_badge_node`, `fan_slot_icon_node`) plus the two anchor enums (`StatBadgeCorner`, `SlotIconAnchor`) live alongside the existing slot-node helpers in `client/src/ui/hand/mod.rs` so each spawn site reads as one labelled call rather than seven inline `Node` literals.

---

## Acceptance Criteria

*Drives the regression test in Phase 4 — `tests/integration/hand-ui/hand_ui_chrome_composition_test.rs`.*

- [ ] **HU-CHROME-01**: GIVEN the client is `InSession` at viewport `1280 × 720` AND two `HandUiCardAcquiredReceived` events have been drained AND the phase has transitioned to `Placement` (so `apply_fan_layout_system` promotes the first two fan slots to `FanSlotState::Active` with the standard `96 × 136` intrinsic box), THEN for each occupied fan slot:
  - (a) `HandCardFrame` child has `Node.position_type == PositionType::Absolute` AND `Node.width.is_percent()` (resolves to `100%`).
  - (b) Each of the four stat-badge children (`StatBadgeAtk`, `StatBadgeHp`, `StatBadgeMp`, `StatBadgeAr`) has `Node.position_type == PositionType::Absolute` AND `Node.width` is a non-zero `Val::Percent` (`20%`) AND has exactly one of `(Node.left, Node.right)` set to a `Val::Percent(0.0)` anchor with the other `Val::Auto`, and likewise for `(Node.top, Node.bottom)`.
  - (c) `HandRarityIcon` and `HandTypeIcon` children have `Node.position_type == PositionType::Absolute` AND `Node.width == Val::Percent(15.0)` AND `Node.left == Val::Percent(42.5)`.

- [ ] **HU-CHROME-02**: GIVEN the same scenario as HU-CHROME-01, THEN every chrome child of every occupied slot has `Node.width` that is **not** `Val::Auto` AND **not** `Val::Px(0.0)` (proves the `Node::default()` regression is closed). This is the canary that fires if a future refactor reverts any of the seven spawn sites to `Node::default()`.

- [ ] **HU-CHROME-03**: GIVEN the same scenario, THEN every chrome child of every occupied slot has `Node.position_type == PositionType::Absolute` (proves chrome layout is not in normal flow — children render at their declared corner / center inside the slot's positioned containing block, not as a vertical stack of zero-height siblings).

---

## Out of Scope

*Handled by other stories — do not implement here:*

- **Worker A territory (HU-02)**: Fan-slot coord-space alignment (`fan_base_y` LOCAL to `fan_root`). Already integrated at `d9ee107`. Do not touch `metrics_for_viewport` or `apply_fan_layout_system`.
- **Asset content**: This story preserves the existing `PlaceholderAssets` references for the seven chrome handles. Real art delivery is a separate stream owned by `art-director` / `S10-POLISH-*` lines.
- **Card-art `ImageNode` sizing**: The fan slot's own card-art image is set by `apply_fan_layout_system` and is not one of the seven chrome children.
- **Visibility lifecycle**: Reserve-strip `Visibility::Inherited` fix landed at `dc664c8` (Story 011 Verdict 2 reconciliation).

---

## Test Evidence

**Story Type**: Integration
**Required evidence**:
- `tests/integration/hand-ui/hand_ui_chrome_composition_test.rs` — must exist and pass at least one happy-path assertion per AC.

**Status**: [ ] Created and passing (Phase 4 of PROMPT 682).

---

## Dependencies

- Depends on: HU-02 fan-slot on-screen coord-space fix (integrated at `d9ee107`) — without it, the chrome would render at the correct local position inside a slot that itself renders off-screen.
- Depends on: `PlaceholderAssets` (story-001 plugin scaffold + PAW-002 chrome handles) — provides the seven `ImageNode` handles the spawn sites reference.
- Unlocks: future visual-art authoring (VA-9 or successor) that can refine the chosen percent layout without re-introducing the 0×0 regression.

## Completion Notes

To be filled by `/story-done` after orchestrator integration.
