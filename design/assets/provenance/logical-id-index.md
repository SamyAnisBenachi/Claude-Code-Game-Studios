# Logical Asset ID Index

> Introduced 2026-05-19 by Story 007 (PROMPT 1369). The schema for the
> three-axis fields lives in [`schema.md`](./schema.md); the governing ADR
> is [ADR-025](../../../docs/architecture/adr-025-asset-pack-provenance-architecture.md).
>
> **Defaults**: every row below carries
> `workflow_status=needed`, `source_class=studio_original`,
> `release_class=release_allowed` unless explicitly overridden in the
> "Notes" column. No row in this initial index is a Krosmaga proxy —
> Krosmaga proxy mappings are added as the Sprint 18/19 Krosmaga-style
> stories adopt the layer, and each such mapping carries the exact triplet
> `source_class=licensed_krosmaga_dev_proxy / workflow_status=needed /
> release_class=dev_only` per the schema.

## Hand / Card Composition

Owner story: `production/epics/presentation-asset-wiring/story-002-hand-ui-card-frames.md`.

| Logical ID | Description | Default studio path (Story 002 wiring) | Notes |
|------------|-------------|----------------------------------------|-------|
| `lid_card_frame_common` | Card frame chrome — common rarity | `art/ui/card/ui_card_frame_common_hand.png` | |
| `lid_card_frame_rare` | Card frame chrome — rare rarity | `art/ui/card/ui_card_frame_rare_hand.png` | |
| `lid_card_frame_epic` | Card frame chrome — epic rarity | `art/ui/card/ui_card_frame_epic_hand.png` | |
| `lid_card_frame_legendary` | Card frame chrome — legendary rarity | `art/ui/card/ui_card_frame_legendary_hand.png` | |
| `lid_card_cost_badge_mana` | Mana cost diamond badge | `art/ui/card/ui_card_badge_cost.png` | |
| `lid_card_stat_badge_atk` | ATK stat badge | `art/ui/card/ui_card_badge_atk.png` | |
| `lid_card_stat_badge_hp` | HP stat badge | `art/ui/card/ui_card_badge_hp.png` | |
| `lid_card_typerarity_icon_unit` | Type / rarity icon — unit | `art/ui/card/ui_card_icon_unit.png` | |
| `lid_card_typerarity_icon_spell` | Type / rarity icon — spell | `art/ui/card/ui_card_icon_spell.png` | |

## Shop / Auction Chrome

Owner story: `production/epics/presentation-asset-wiring/story-003-shop-auction-chrome.md`.

| Logical ID | Description | Default studio path | Notes |
|------------|-------------|---------------------|-------|
| `lid_shop_panel_chrome` | DRAFT_SHOP panel chrome | `art/ui/shop/ui_shop_panel_chrome.png` | |
| `lid_shop_slot_well_idle` | Shop slot well background — idle | `art/ui/shop/ui_shop_slot_well_idle.png` | |
| `lid_auction_panel_chrome` | Auction panel chrome | `art/ui/auction/ui_auction_panel_chrome.png` | |
| `lid_auction_bid_button_idle` | Bid button chrome — idle | `art/ui/auction/ui_auction_bid_button_idle.png` | |
| `lid_auction_bid_button_hover` | Bid button chrome — hover | `art/ui/auction/ui_auction_bid_button_hover.png` | |
| `lid_auction_bid_button_locked` | Bid button chrome — locked | `art/ui/auction/ui_auction_bid_button_locked.png` | |

## HUD

Owner story: `production/epics/presentation-asset-wiring/story-004-hud-figurines-timer-dots.md`.

| Logical ID | Description | Default studio path | Notes |
|------------|-------------|---------------------|-------|
| `lid_hud_class_figurine_iop` | HUD class figurine — Iop | `art/ui/hud/ui_class_figurine_iop.png` | |
| `lid_hud_class_figurine_cra` | HUD class figurine — Cra | `art/ui/hud/ui_class_figurine_cra.png` | |
| `lid_hud_class_figurine_sacrier` | HUD class figurine — Sacrier | `art/ui/hud/ui_class_figurine_sacrier.png` | |
| `lid_hud_class_figurine_xelor` | HUD class figurine — Xelor | `art/ui/hud/ui_class_figurine_xelor.png` | |
| `lid_hud_class_figurine_ecaflip` | HUD class figurine — Ecaflip | `art/ui/hud/ui_class_figurine_ecaflip.png` | |
| `lid_hud_class_figurine_sadida` | HUD class figurine — Sadida | `art/ui/hud/ui_class_figurine_sadida.png` | |
| `lid_hud_class_figurine_unknown` | HUD class figurine — unknown / not selected | `art/ui/hud/ui_class_figurine_unknown.png` | |
| `lid_hud_phase_timer_bar` | Phase timer bar chrome | `art/ui/hud/ui_phase_timer_bar.png` | |
| `lid_hud_objective_dot_unknown` | Objective dot — unknown state | `art/ui/hud/ui_objective_dot_unknown.png` | |
| `lid_hud_objective_dot_real_revealed` | Objective dot — real revealed | `art/ui/hud/ui_objective_dot_real.png` | |
| `lid_hud_objective_dot_fake_revealed` | Objective dot — fake revealed | `art/ui/hud/ui_objective_dot_fake.png` | |
| `lid_hud_objective_dot_destroyed` | Objective dot — destroyed | `art/ui/hud/ui_objective_dot_destroyed.png` | |

## Board (World-Space Sprites)

Owner story: `production/epics/presentation-asset-wiring/story-005-board-unit-sprites.md`.

| Logical ID | Description | Default studio path | Notes |
|------------|-------------|---------------------|-------|
| `lid_board_chrome_default` | Board background chrome | `art/board/env_board_chrome_default.png` | |
| `lid_board_lane_divider_64x80` | Lane divider tile | `art/board/env_lane_divider_64x80.png` | |
| `lid_board_cell_idle_32x32` | Cell node — idle state | `art/board/env_cell_node_idle_32x32.png` | |
| `lid_board_cell_spawn_active_32x32` | Cell node — spawn active | `art/board/env_cell_node_spawn_active_32x32.png` | |
| `lid_board_cell_invalid_32x32` | Cell node — invalid placement | `art/board/env_cell_node_invalid_32x32.png` | |
| `lid_board_unit_base_player_a_48x16` | Unit base sprite — player A | `art/board/ui_unit_base_player_a_48x16.png` | |
| `lid_board_unit_base_player_b_48x16` | Unit base sprite — player B | `art/board/ui_unit_base_player_b_48x16.png` | |
| `lid_board_objective_unknown_64x96` | Objective sprite — unknown state | `art/board/env_objective_unknown_64x96.png` | |
| `lid_board_objective_real_reveal_64x96` | Objective sprite — real revealed | `art/board/env_objective_real_reveal_64x96.png` | |

## Overlays (Reveal / Targeting / Result)

Owner stories: Sang Méprise reveal markers map to the Class-System
class-feedback wave (see `design/assets/specs/class-system-assets.md`
ASSET-120/121); result chrome remains blocked pending result-screen UX
(see ASSET-211 through ASSET-214) but the logical IDs are reserved.

| Logical ID | Description | Default studio path | Notes |
|------------|-------------|---------------------|-------|
| `lid_overlay_targeting_marker_real` | Sang Méprise reveal marker — real variant | `art/ui/overlay/ui_overlay_target_real.png` | |
| `lid_overlay_targeting_marker_fake` | Sang Méprise reveal marker — fake variant | `art/ui/overlay/ui_overlay_target_fake.png` | |
| `lid_result_panel_chrome_win` | Result panel chrome — WIN | `art/ui/result/ui_result_panel_win.png` | `workflow_status=blocked` until result-screen UX is unblocked (ASSET-211/213). |
| `lid_result_panel_chrome_loss` | Result panel chrome — LOSS | `art/ui/result/ui_result_panel_loss.png` | Same blocked note. |
| `lid_result_panel_chrome_draw` | Result panel chrome — DRAW | `art/ui/result/ui_result_panel_draw.png` | Same blocked note. |

## Shared / Fallback

| Logical ID | Description | Default studio path | Notes |
|------------|-------------|---------------------|-------|
| `lid_ui_placeholder_1x1_white` | Universal fallback per ADR-021 path convention | `art/ui/shared/ui_placeholder_1x1_white.png` | Resolves any logical-ID lookup that has no other entry. |

## Krosmaga Proxy Mappings (Reserved — empty in this initial index)

This section is intentionally empty in the initial index. When a Sprint 18
or Sprint 19 Krosmaga-style implementation story adopts the logical-ID
layer for one of the surfaces above, it adds a row here with **exactly**:

```yaml
- logical_id: <existing lid_*>
  source_class: licensed_krosmaga_dev_proxy
  workflow_status: needed
  release_class: dev_only
  dev_pack_entries:
    krosmaga-proxy-v1: <relative path inside dev-assets/krosmaga-proxy/>
```

No such row may carry any other `source_class`, `workflow_status`, or
`release_class` value. The release-scan validator hard-fails any packaged
build that resolves a logical ID through a Krosmaga proxy regardless of
this index's content — the index is documentation, not the enforcement
boundary.

## Pack Selection (Reminder)

Per [`schema.md`](./schema.md):

1. Release build → only `release_allowed` packs eligible. Krosmaga proxies
   are unreachable in a release build.
2. Dev workstation + `dev-assets/krosmaga-proxy/` present → may resolve a
   logical ID to the Krosmaga proxy entry for that ID (if listed).
3. Otherwise studio placeholder / generated placeholder.
4. Fallback `lid_ui_placeholder_1x1_white`.
