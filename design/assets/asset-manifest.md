# Asset Manifest

> Last updated: 2026-05-09

## Progress Summary

| Total | Needed | In Progress | Done | Approved |
|-------|--------|-------------|------|----------|
| 296 | 296 | 0 | 0 | 0 |

Generated Placeholder and File Present Placeholder rows remain counted under Needed until production approval is recorded.

## Status Taxonomy

- Needed: no usable delivery file is tracked yet.
- Placeholder: ownership placeholder only; no delivered-file credit.
- File Present Placeholder: delivery file exists and matches required name/technical dimensions, but is not production approved and still counts as Needed.
- Generated Placeholder: generated art/VFX file exists and matches required name/technical dimensions, but is not production approved and still counts as Needed.
- In Progress: production work underway.
- Done: final delivery complete with supporting evidence.
- Approved: production approval/sign-off complete.
- Blocked: tracked but waiting on unresolved design/UX/dependency.

---

## 2026-05-04 Expansion Notes

- Full-game coverage now includes combat resolution, keyword system assets, auction-system audio/material states, prism rewards, game session / lobby / reconnect / outcome ownership, shared fonts / materials / shaders, and illustration specs for the current `cards.json` art IDs.
- Current per-card rows are illustration-only. Runtime card composition owns frames, badges, text, type / rarity labels, hover, ghost, drag, and state overlays.
- The full ~315-card catalog remains deferred until roster IDs, card IDs, and art IDs are reconciled.
- GAME_OVER / outcome rows are placeholder ownership only and blocked pending result-screen UX.
- Optional bid confirmation remains an unresolved accessibility design decision; no confirmation-step production assets are tracked in this manifest yet.

---

## 2026-05-09 Expansion — Card Animations, Objective System, Round State Machine

Three new per-system spec files added covering the remaining GDDs that had no asset spec:
- `design/assets/specs/card-animations-assets.md` — ASSET-243 through ASSET-266 (24 assets): custom lens types, timing constants config block, damage-number text/lifecycle components, PLACEMENT marker, domain message types, and the `StagedObjectiveRevealQueue` / `GroupDrainedSignal` pipeline types.
- `design/assets/specs/objective-system-assets.md` — ASSET-267 through ASSET-280 (14 assets): reveal-moment two-beat VFX, owner identity indicator glyphs, Sang Méprise temporary slot tints, ADR-001 unicast message types, and the identity reveal audio sting.
- `design/assets/specs/round-state-machine-assets.md` — ASSET-281 through ASSET-296 (16 assets): phase announcement banners, round number badge, GAME_OVER result panel and text styles (4 outcome variants), `RoundState` resource / `GameOverReason` enum, and PLACEMENT begin tension sting.
- Assets that overlap with existing spec systems (objective destruction VFX, board sprites, HUD timer bar, auction/session audio) are cross-referenced by existing IDs and not re-minted.
- New manifest total: 296 (was 242, +54).

---

## 2026-05-07 Card Illustration Reconciliation

ASSET-227 through ASSET-242 now reflect generated placeholder files for the current `assets/data/cards.json` `art_id` set. This is file tracking only: no row is Done or Approved, and none of these illustrations are cleared for public release until production approval/sign-off is documented.

| Asset ID | art_id | Display file | Zoom file | Reconciled status |
|----------|--------|--------------|-----------|-------------------|
| ASSET-227 | iop_knight_001 | `assets/art/cards/display/card_iop_knight_001_art_display.png` (120x180) | `assets/art/cards/zoom/card_iop_knight_001_art_zoom.png` (240x360) | Generated Placeholder |
| ASSET-228 | sacrier_foot_002 | `assets/art/cards/display/card_sacrier_foot_002_art_display.png` (120x180) | `assets/art/cards/zoom/card_sacrier_foot_002_art_zoom.png` (240x360) | Generated Placeholder |
| ASSET-229 | cra_piercing_shot_003 | `assets/art/cards/display/card_cra_piercing_shot_003_art_display.png` (120x180) | `assets/art/cards/zoom/card_cra_piercing_shot_003_art_zoom.png` (240x360) | Generated Placeholder |
| ASSET-230 | xelor_time_trap_004 | `assets/art/cards/display/card_xelor_time_trap_004_art_display.png` (120x180) | `assets/art/cards/zoom/card_xelor_time_trap_004_art_zoom.png` (240x360) | Generated Placeholder |
| ASSET-231 | gobball_sturdy_005 | `assets/art/cards/display/card_gobball_sturdy_005_art_display.png` (120x180) | `assets/art/cards/zoom/card_gobball_sturdy_005_art_zoom.png` (240x360) | Generated Placeholder |
| ASSET-232 | sadida_rose_field_006 | `assets/art/cards/display/card_sadida_rose_field_006_art_display.png` (120x180) | `assets/art/cards/zoom/card_sadida_rose_field_006_art_zoom.png` (240x360) | Generated Placeholder |
| ASSET-233 | ecaflip_decree_007 | `assets/art/cards/display/card_ecaflip_decree_007_art_display.png` (120x180) | `assets/art/cards/zoom/card_ecaflip_decree_007_art_zoom.png` (240x360) | Generated Placeholder |
| ASSET-234 | iop_double_face_008 | `assets/art/cards/display/card_iop_double_face_008_art_display.png` (120x180) | `assets/art/cards/zoom/card_iop_double_face_008_art_zoom.png` (240x360) | Generated Placeholder |
| ASSET-235 | neutral_tofu_scout_101 | `assets/art/cards/display/card_neutral_tofu_scout_101_art_display.png` (120x180) | `assets/art/cards/zoom/card_neutral_tofu_scout_101_art_zoom.png` (240x360) | Generated Placeholder |
| ASSET-236 | neutral_wabbit_guard_102 | `assets/art/cards/display/card_neutral_wabbit_guard_102_art_display.png` (120x180) | `assets/art/cards/zoom/card_neutral_wabbit_guard_102_art_zoom.png` (240x360) | Generated Placeholder |
| ASSET-237 | neutral_market_runner_103 | `assets/art/cards/display/card_neutral_market_runner_103_art_display.png` (120x180) | `assets/art/cards/zoom/card_neutral_market_runner_103_art_zoom.png` (240x360) | Generated Placeholder |
| ASSET-238 | neutral_training_banner_104 | `assets/art/cards/display/card_neutral_training_banner_104_art_display.png` (120x180) | `assets/art/cards/zoom/card_neutral_training_banner_104_art_zoom.png` (240x360) | Generated Placeholder |
| ASSET-239 | neutral_guild_errand_105 | `assets/art/cards/display/card_neutral_guild_errand_105_art_display.png` (120x180) | `assets/art/cards/zoom/card_neutral_guild_errand_105_art_zoom.png` (240x360) | Generated Placeholder |
| ASSET-240 | neutral_paddock_bruiser_106 | `assets/art/cards/display/card_neutral_paddock_bruiser_106_art_display.png` (120x180) | `assets/art/cards/zoom/card_neutral_paddock_bruiser_106_art_zoom.png` (240x360) | Generated Placeholder |
| ASSET-241 | neutral_vault_sentry_107 | `assets/art/cards/display/card_neutral_vault_sentry_107_art_display.png` (120x180) | `assets/art/cards/zoom/card_neutral_vault_sentry_107_art_zoom.png` (240x360) | Generated Placeholder |
| ASSET-242 | neutral_crowned_mercenary_108 | `assets/art/cards/display/card_neutral_crowned_mercenary_108_art_display.png` (120x180) | `assets/art/cards/zoom/card_neutral_crowned_mercenary_108_art_zoom.png` (240x360) | Generated Placeholder |

Remaining work: production art review, final delivery evidence, approval/sign-off, and public-release clearance.

---

## Assets by Context

### System: Shop / Auction UI

| Asset ID | Name | Category | Status | Spec File |
|----------|------|----------|--------|-----------|
| ASSET-001 | Gold Coin Icon | UI Icon | Needed | design/assets/specs/shop-auction-ui-assets.md |
| ASSET-002 | Rarity Gem — Rare | UI Badge | Needed | design/assets/specs/shop-auction-ui-assets.md |
| ASSET-003 | Rarity Gem — Epic | UI Badge | Needed | design/assets/specs/shop-auction-ui-assets.md |
| ASSET-004 | Rarity Gem — Legendary | UI Badge | Needed | design/assets/specs/shop-auction-ui-assets.md |
| ASSET-005 | DRAFT_SHOP Slot Well Highlight Strip | UI Background | Needed | design/assets/specs/shop-auction-ui-assets.md |
| ASSET-006 | Auction Panel Background | UI Background | Needed | design/assets/specs/shop-auction-ui-assets.md |
| ASSET-007 | Auction Panel Border Ramp Tiles (×4) | UI Border | Needed | design/assets/specs/shop-auction-ui-assets.md |
| ASSET-008 | Gold Particle Glow Sprite | VFX / Particle | Needed | design/assets/specs/shop-auction-ui-assets.md |
| ASSET-009 | Gold Bloom Glow Sprite | VFX / Overlay | Needed | design/assets/specs/shop-auction-ui-assets.md |
| ASSET-010 | Prism White Flash Sprite | VFX / UI Overlay | Needed | design/assets/specs/shop-auction-ui-assets.md |
| ASSET-011 | Bid Pulse Ring Frames | VFX / Animation | Needed | design/assets/specs/shop-auction-ui-assets.md |
| ASSET-012 | DRAFT_INITIAL Entry Sting | Audio | Needed | design/assets/specs/shop-auction-ui-assets.md |
| ASSET-013 | DRAFT_INITIAL Purchase Chime | Audio | Needed | design/assets/specs/shop-auction-ui-assets.md |
| ASSET-014 | DRAFT_INITIAL Budget Depleted Bell | Audio | Needed | design/assets/specs/shop-auction-ui-assets.md |
| ASSET-015 | DRAFT_SHOP Entry Phrase | Audio | Needed | design/assets/specs/shop-auction-ui-assets.md |
| ASSET-016 | DRAFT_SHOP Purchase Chime | Audio | Needed | design/assets/specs/shop-auction-ui-assets.md |
| ASSET-017 | Shop Refresh Swoosh | Audio | Needed | design/assets/specs/shop-auction-ui-assets.md |
| ASSET-018 | Shop Refresh Failed | Audio | Needed | design/assets/specs/shop-auction-ui-assets.md |
| ASSET-019 | Ready Signal | Audio | Needed | design/assets/specs/shop-auction-ui-assets.md |
| ASSET-020 | Ready Retracted | Audio | Needed | design/assets/specs/shop-auction-ui-assets.md |
| ASSET-021 | Countdown Tick Loop | Audio | Needed | design/assets/specs/shop-auction-ui-assets.md |

### System: board-rendering

| Asset ID | Name | Category | M2 Priority | Status | Spec File |
|----------|------|----------|-------------|--------|-----------|
| ASSET-022 | env_board_background_default | Environment | PLACEHOLDER | Needed | design/assets/specs/board-rendering-assets.md |
| ASSET-023 | env_lane_divider_64x80 | Environment | PLACEHOLDER | Needed | design/assets/specs/board-rendering-assets.md |
| ASSET-024 | env_lane_number_label_01–05 (×5) | Environment / UI | BLOCKING | Generated Placeholder | design/assets/specs/board-rendering-assets.md |
| ASSET-025 | env_cell_node_idle_32x32 | Environment | BLOCKING | Generated Placeholder | design/assets/specs/board-rendering-assets.md |
| ASSET-026 | env_cell_node_spawn_active_32x32 | Environment | BLOCKING | Generated Placeholder | design/assets/specs/board-rendering-assets.md |
| ASSET-027 | env_cell_node_spawn_inactive_32x32 | Environment | PLACEHOLDER | Needed | design/assets/specs/board-rendering-assets.md |
| ASSET-028 | env_cell_node_invalid_32x32 | Environment | PLACEHOLDER | Needed | design/assets/specs/board-rendering-assets.md |
| ASSET-029 | env_objective_unknown_64x96 | Environment | BLOCKING | Generated Placeholder | design/assets/specs/board-rendering-assets.md |
| ASSET-030 | env_objective_real_reveal_64x96 | Environment | BLOCKING | Generated Placeholder | design/assets/specs/board-rendering-assets.md |
| ASSET-031 | env_objective_fake_crack_64x96 | Environment | PLACEHOLDER | Needed | design/assets/specs/board-rendering-assets.md |
| ASSET-032 | env_prism_idle_32x32 | Environment | PLACEHOLDER | Needed | design/assets/specs/board-rendering-assets.md |
| ASSET-033 | ui_unit_base_player_a_48x16 | UI | BLOCKING | Generated Placeholder | design/assets/specs/board-rendering-assets.md |
| ASSET-034 | ui_unit_base_player_b_48x16 | UI | BLOCKING | Generated Placeholder | design/assets/specs/board-rendering-assets.md |
| ASSET-035 | ui_trap_tile_facedown_32x32 | Environment (world-space) | BLOCKING | Generated Placeholder | design/assets/specs/board-rendering-assets.md |
| ASSET-036 | ui_structure_token_32x32 | Environment (world-space) | PLACEHOLDER | Needed | design/assets/specs/board-rendering-assets.md |
| ASSET-037 | ui_field_wash_lane_512x80 | Environment (world-space) | PLACEHOLDER | Needed | design/assets/specs/board-rendering-assets.md |
| ASSET-038 | ui_field_badge_icon_24x24 | UI | PLACEHOLDER | Needed | design/assets/specs/board-rendering-assets.md |
| ASSET-039 | ui_unit_placeholder_48x64 | UI / Error State | BLOCKING | Generated Placeholder | design/assets/specs/board-rendering-assets.md |
| ASSET-040 | vfx_objective_real_flash (×3 frames) | VFX | BLOCKING | Generated Placeholder | design/assets/specs/board-rendering-assets.md |
| ASSET-041 | vfx_objective_attack_ring | VFX | PLACEHOLDER (M3) | Needed | design/assets/specs/board-rendering-assets.md |
| ASSET-042 | vfx_spawn_range_pulse | VFX | PLACEHOLDER | Needed | design/assets/specs/board-rendering-assets.md |
| ASSET-043 | vfx_prism_collect_shimmer | VFX | PLACEHOLDER | Needed | design/assets/specs/board-rendering-assets.md |
| ASSET-044 | hp_bar_white_pixel_1x2 | Reserved Atlas Frame | BLOCKING | File Present Placeholder | design/assets/specs/board-rendering-assets.md |
| ASSET-045 | snd_reveal_sting | Audio | BLOCKING | Needed | design/assets/specs/board-rendering-assets.md |
| ASSET-046 | snd_unit_advance | Audio | BLOCKING | Needed | design/assets/specs/board-rendering-assets.md |
| ASSET-047 | snd_objective_destroy_real | Audio | BLOCKING | Needed | design/assets/specs/board-rendering-assets.md |
| ASSET-048 | snd_objective_destroy_fake | Audio | BLOCKING | Needed | design/assets/specs/board-rendering-assets.md |
| ASSET-049 | snd_prism_collect | Audio | ADVISORY | Needed | design/assets/specs/board-rendering-assets.md |
| ASSET-050 | snd_objective_attack | Audio | ADVISORY | Needed | design/assets/specs/board-rendering-assets.md |
| ASSET-051 | snd_trap_trigger | Audio | ADVISORY | Needed | design/assets/specs/board-rendering-assets.md |

### System: hand-ui

| Asset ID | Name | Category | Status | Spec File |
|----------|------|----------|--------|-----------|
| ASSET-052 | Card Display Composition Template | Runtime Card Composition | Needed | design/assets/specs/hand-ui-assets.md |
| ASSET-053 | Card Zoom Composition Template | Runtime Card Composition | Needed | design/assets/specs/hand-ui-assets.md |
| ASSET-054 | Drag Card Composition | Runtime Card Composition | Needed | design/assets/specs/hand-ui-assets.md |
| ASSET-055 | Card Frame Chrome (Hand) | UI | Needed | design/assets/specs/hand-ui-assets.md |
| ASSET-056 | Mana Cost Diamond Badge | UI | Needed | design/assets/specs/hand-ui-assets.md |
| ASSET-057 | ATK Stat Badge | UI | Needed | design/assets/specs/hand-ui-assets.md |
| ASSET-058 | HP Stat Badge | UI | Needed | design/assets/specs/hand-ui-assets.md |
| ASSET-059 | Type/Rarity Icon | UI | Needed | design/assets/specs/hand-ui-assets.md |
| ASSET-060 | DRAFT_INITIAL Grid Panel | UI | Needed | design/assets/specs/hand-ui-assets.md |
| ASSET-061 | Grid Slot Cell | UI | Needed | design/assets/specs/hand-ui-assets.md |
| ASSET-062 | Grid Slot Empty Checkmark | UI | Needed | design/assets/specs/hand-ui-assets.md |
| ASSET-063 | Submit Button | UI | Needed | design/assets/specs/hand-ui-assets.md |
| ASSET-064 | Submit Pre-Validation Error Label | UI | Needed | design/assets/specs/hand-ui-assets.md |
| ASSET-065 | Placement Timer Panel | UI | Needed | design/assets/specs/hand-ui-assets.md |
| ASSET-066 | Timer Checkmark Glyph | UI | Needed | design/assets/specs/hand-ui-assets.md |
| ASSET-067 | Reserve Mana Split Strip | UI | Needed | design/assets/specs/hand-ui-assets.md |
| ASSET-068 | "Auction in Progress" Label | UI | Needed | design/assets/specs/hand-ui-assets.md |
| ASSET-069 | Hand Full Notification | UI | Needed | design/assets/specs/hand-ui-assets.md |
| ASSET-070 | "No Valid Targets" Overlay | UI | Needed | design/assets/specs/hand-ui-assets.md |
| ASSET-071 | Fan Ghost Slot (Staged Card) | UI | Needed | design/assets/specs/hand-ui-assets.md |
| ASSET-072 | Hand-Full Grid Lock Overlay | UI | Needed | design/assets/specs/hand-ui-assets.md |
| ASSET-073 | Card Purchase Bloom Flash | VFX | Needed | design/assets/specs/hand-ui-assets.md |
| ASSET-074 | Card Hover Gold Outline Pulse | VFX | Needed | design/assets/specs/hand-ui-assets.md |
| ASSET-075 | Fan Plate Prism White Border Glow | VFX | Needed | design/assets/specs/hand-ui-assets.md |
| ASSET-076 | Fan Plate Staged Gold Flash | VFX | Needed | design/assets/specs/hand-ui-assets.md |
| ASSET-077 | TargetUnit Hover Outline | VFX | Needed | design/assets/specs/hand-ui-assets.md |
| ASSET-078 | Card Lift SFX | Audio | Needed | design/assets/specs/hand-ui-assets.md |
| ASSET-079 | Valid Targets Appear SFX | Audio | Needed | design/assets/specs/hand-ui-assets.md |
| ASSET-080 | Successful Stage SFX | Audio | Needed | design/assets/specs/hand-ui-assets.md |
| ASSET-081 | Snap-Back / Invalid Drop SFX | Audio | Needed | design/assets/specs/hand-ui-assets.md |
| ASSET-082 | Instant Card Staged SFX | Audio | Needed | design/assets/specs/hand-ui-assets.md |
| ASSET-083 | Submit SFX | Audio | Needed | design/assets/specs/hand-ui-assets.md |
| ASSET-084 | Timer Urgency SFX | Audio | Needed | design/assets/specs/hand-ui-assets.md |
| ASSET-085 | Card Acquired SFX | Audio | Needed | design/assets/specs/hand-ui-assets.md |
| ASSET-086 | Hand Full SFX | Audio | Needed | design/assets/specs/hand-ui-assets.md |
| ASSET-087 | Reserve Adjust Click SFX | Audio | Needed | design/assets/specs/hand-ui-assets.md |

### System: hud

| Asset ID | Name | Category | Status | Spec File |
|----------|------|----------|--------|-----------|
| ASSET-088 | HUD Zone Chip Background | Sprite / 2D Art | Needed | design/assets/specs/hud-assets.md |
| ASSET-089 | Reserve Mana Diamond Icon | Sprite / 2D Art | Needed | design/assets/specs/hud-assets.md |
| ASSET-090 | Project Display Font Style Anchor (split to ASSET-215/216) | Font Direction / Style Anchor | Needed | design/assets/specs/hud-assets.md |
| ASSET-091 | Phase Transition Tick SFX | Audio | Needed | design/assets/specs/hud-assets.md |
| ASSET-092 | Scoreboard Dot Darkening Thud SFX | Audio | Needed | design/assets/specs/hud-assets.md |
| ASSET-093 | GAME_OVER Resolved Chord SFX | Audio | Needed | design/assets/specs/hud-assets.md |

### System: class-system

| Asset ID | Name | Category | Status | Spec File |
|----------|------|----------|--------|-----------|
| ASSET-094 | Figurine: Iop | Sprite / 2D Art | Needed | design/assets/specs/class-system-assets.md |
| ASSET-095 | Figurine: Cra | Sprite / 2D Art | Needed | design/assets/specs/class-system-assets.md |
| ASSET-096 | Figurine: Sacrier | Sprite / 2D Art | Needed | design/assets/specs/class-system-assets.md |
| ASSET-097 | Figurine: Xelor | Sprite / 2D Art | Needed | design/assets/specs/class-system-assets.md |
| ASSET-098 | Figurine: Ecaflip ⚠️ F-CS-1 | Sprite / 2D Art | Needed | design/assets/specs/class-system-assets.md |
| ASSET-099 | Figurine: Sadida ⚠️ F-CS-1 | Sprite / 2D Art | Needed | design/assets/specs/class-system-assets.md |
| ASSET-100 | Token Sprite: Mummy / Momie (Xelor) | Sprite / 2D Art | Needed | design/assets/specs/class-system-assets.md |
| ASSET-101 | Token Sprite: Chacha Noir (Ecaflip) ⚠️ F-CS-1 | Sprite / 2D Art | Needed | design/assets/specs/class-system-assets.md |
| ASSET-102 | Token Sprite: Madoll / La Folle (Sadida) ⚠️ F-CS-1 | Sprite / 2D Art | Needed | design/assets/specs/class-system-assets.md |
| ASSET-103 | Token Sprite: La Gonflable (Sadida) ⚠️ F-CS-1 | Sprite / 2D Art | Needed | design/assets/specs/class-system-assets.md |
| ASSET-104 | Token Sprite: La Sacrifiée (Sadida) ⚠️ F-CS-1 | Sprite / 2D Art | Needed | design/assets/specs/class-system-assets.md |
| ASSET-105 | Graine / Seed Cell Floor Marker (Sadida) ⚠️ F-CS-1 | Environment | Needed | design/assets/specs/class-system-assets.md |
| ASSET-106 | Class Icon: Iop | UI | Needed | design/assets/specs/class-system-assets.md |
| ASSET-107 | Class Icon: Cra | UI | Needed | design/assets/specs/class-system-assets.md |
| ASSET-108 | Class Icon: Sacrier | UI | Needed | design/assets/specs/class-system-assets.md |
| ASSET-109 | Class Icon: Xelor | UI | Needed | design/assets/specs/class-system-assets.md |
| ASSET-110 | Class Icon: Ecaflip ⚠️ F-CS-1 | UI | Needed | design/assets/specs/class-system-assets.md |
| ASSET-111 | Class Icon: Sadida ⚠️ F-CS-1 | UI | Needed | design/assets/specs/class-system-assets.md |
| ASSET-112 | Class Picker Panel Background ⚠️ F-CS-3 | UI | Needed | design/assets/specs/class-system-assets.md |
| ASSET-113 | Class Option Tile (reusable frame) | UI | Needed | design/assets/specs/class-system-assets.md |
| ASSET-114 | Class Locked Indicator Badge | UI | Needed | design/assets/specs/class-system-assets.md |
| ASSET-115 | "Waiting for Opponent" Placeholder Tile | UI | Needed | design/assets/specs/class-system-assets.md |
| ASSET-116 | Sinistro Objective Indicator Icon | UI | Needed | design/assets/specs/class-system-assets.md |
| ASSET-117 | Garde-Temps Exhausted Badge | UI | Needed | design/assets/specs/class-system-assets.md |
| ASSET-118 | Reserve Insufficient Indicator Glyph | UI | Needed | design/assets/specs/class-system-assets.md |
| ASSET-119 | Rollback Zero-Reserve Inline Warning ⚠️ F-CS-4 | UI | Needed | design/assets/specs/class-system-assets.md |
| ASSET-120 | Sang Méprise Reveal Marker — Real variant | VFX / Particles | Needed | design/assets/specs/class-system-assets.md |
| ASSET-121 | Sang Méprise Reveal Marker — Fake variant | VFX / Particles | Needed | design/assets/specs/class-system-assets.md |
| ASSET-122 | Xelorium Drain Flash | VFX / Particles | Needed | design/assets/specs/class-system-assets.md |
| ASSET-123 | Class Select Hover SFX | Audio | Needed | design/assets/specs/class-system-assets.md |
| ASSET-124 | Class Confirm / Ready SFX | Audio | Needed | design/assets/specs/class-system-assets.md |
| ASSET-125 | Opponent Class Reveal SFX | Audio | Needed | design/assets/specs/class-system-assets.md |
| ASSET-126 | Reserve Gain SFX | Audio | Needed | design/assets/specs/class-system-assets.md |
| ASSET-127 | Ready Retract SFX | Audio | Needed | design/assets/specs/class-system-assets.md |

### System: combat-resolution

| Asset ID | Name | Category | Status | Spec File |
|----------|------|----------|--------|-----------|
| ASSET-128 | Placement Reveal Card-Back Silhouette | Sprite / Overlay | Needed | design/assets/specs/combat-resolution-assets.md |
| ASSET-129 | Placement Reveal Prism Edge Flash | VFX | Needed | design/assets/specs/combat-resolution-assets.md |
| ASSET-130 | Player-Side Base Ring Reveal Flash | VFX | Needed | design/assets/specs/combat-resolution-assets.md |
| ASSET-131 | CHARGE X Motion Trail Copy Material | Material / Runtime Tint | Needed | design/assets/specs/combat-resolution-assets.md |
| ASSET-132 | FIRST STRIKE Impact Flash | VFX | Needed | design/assets/specs/combat-resolution-assets.md |
| ASSET-133 | Standard Combat Impact Flash | VFX | Needed | design/assets/specs/combat-resolution-assets.md |
| ASSET-134 | Damage Number Text Style | Runtime Text / Material | Needed | design/assets/specs/combat-resolution-assets.md |
| ASSET-135 | SHIELD Active Hex Glyph | UI / Unit Indicator | Needed | design/assets/specs/combat-resolution-assets.md |
| ASSET-136 | SHIELD Absorb Burst Particles | VFX / Particle | Needed | design/assets/specs/combat-resolution-assets.md |
| ASSET-137 | STUN Orbit Star Glyph | UI / Unit Indicator | Needed | design/assets/specs/combat-resolution-assets.md |
| ASSET-138 | INJURED Outline Pulse Material | Shader / Material | Needed | design/assets/specs/combat-resolution-assets.md |
| ASSET-139 | LEADER Crown Glyph | UI / Unit Indicator | Needed | design/assets/specs/combat-resolution-assets.md |
| ASSET-140 | LEADER Family Buff Ring Tint | Material / Runtime Tint | Needed | design/assets/specs/combat-resolution-assets.md |
| ASSET-141 | SILENCE Desaturation Outline Material | Shader / Material | Needed | design/assets/specs/combat-resolution-assets.md |
| ASSET-142 | OUTNUMBERED Arrow-Down Glyph | UI / Unit Indicator | Needed | design/assets/specs/combat-resolution-assets.md |
| ASSET-143 | Death Squash / Crimson Tint Material | Shader / Material | Needed | design/assets/specs/combat-resolution-assets.md |
| ASSET-144 | Death Crimson Particle Burst | VFX / Particle | Needed | design/assets/specs/combat-resolution-assets.md |
| ASSET-145 | Trigger Gold Pulse Ring | VFX | Needed | design/assets/specs/combat-resolution-assets.md |
| ASSET-146 | Ranged Bolt Projectile - Blade | VFX / Projectile | Needed | design/assets/specs/combat-resolution-assets.md |
| ASSET-147 | Ranged Bolt Projectile - Arcane | VFX / Projectile | Needed | design/assets/specs/combat-resolution-assets.md |
| ASSET-148 | Ranged Bolt Projectile - Neutral | VFX / Projectile | Needed | design/assets/specs/combat-resolution-assets.md |
| ASSET-149 | Objective HP Pip Damage Flash | Material / UI Overlay | Needed | design/assets/specs/combat-resolution-assets.md |
| ASSET-150 | Objective Destruction Prism Overlay Frames | VFX / Screen Overlay | Needed | design/assets/specs/combat-resolution-assets.md |
| ASSET-151 | Real Objective Lane Gold Flood | VFX / Lane Overlay | Needed | design/assets/specs/combat-resolution-assets.md |
| ASSET-152 | Fake Objective Question Dissolve | VFX / Glyph Animation | Needed | design/assets/specs/combat-resolution-assets.md |
| ASSET-153 | Kill Gold +1 Float Text Style | Runtime Text / Material | Needed | design/assets/specs/combat-resolution-assets.md |
| ASSET-154 | Objective Gold +3 Float Text Style | Runtime Text / Material | Needed | design/assets/specs/combat-resolution-assets.md |
| ASSET-155 | Placement Reveal Flip SFX | Audio | Needed | design/assets/specs/combat-resolution-assets.md |
| ASSET-156 | FIRST STRIKE Impact SFX | Audio | Needed | design/assets/specs/combat-resolution-assets.md |
| ASSET-157 | Standard Combat Impact SFX | Audio | Needed | design/assets/specs/combat-resolution-assets.md |
| ASSET-158 | Unit Death SFX | Audio | Needed | design/assets/specs/combat-resolution-assets.md |
| ASSET-159 | SHIELD Absorb SFX | Audio | Needed | design/assets/specs/combat-resolution-assets.md |
| ASSET-160 | SHIELD Break SFX | Audio | Needed | design/assets/specs/combat-resolution-assets.md |
| ASSET-161 | Kill Gold Reward SFX | Audio | Needed | design/assets/specs/combat-resolution-assets.md |
| ASSET-162 | COUNTERATTACK Response SFX | Audio | Needed | design/assets/specs/combat-resolution-assets.md |

### System: keyword-system

| Asset ID | Name | Category | Status | Spec File |
|----------|------|----------|--------|-----------|
| ASSET-163 | BODYGUARD Shield-Arc Glyph | UI / Unit Indicator | Needed | design/assets/specs/keyword-system-assets.md |
| ASSET-164 | BODYGUARD Bond Break Shards | VFX / Particle | Needed | design/assets/specs/keyword-system-assets.md |
| ASSET-165 | IRREMOVABLE Chain-Link Glyph | UI / Unit Indicator | Needed | design/assets/specs/keyword-system-assets.md |
| ASSET-166 | IRREMOVABLE Block Flash | VFX / Overlay | Needed | design/assets/specs/keyword-system-assets.md |
| ASSET-167 | UNTARGETABLE Diamond-Cross Glyph | UI / Unit Indicator | Needed | design/assets/specs/keyword-system-assets.md |
| ASSET-168 | REPEL Push Flash | VFX / Overlay | Needed | design/assets/specs/keyword-system-assets.md |
| ASSET-169 | ATTRACT Pull Flash | VFX / Overlay | Needed | design/assets/specs/keyword-system-assets.md |
| ASSET-170 | TELEPORT Exit Bar | VFX | Needed | design/assets/specs/keyword-system-assets.md |
| ASSET-171 | TELEPORT Entry Bar | VFX | Needed | design/assets/specs/keyword-system-assets.md |
| ASSET-172 | START OF TURN Floating Label Style | Runtime Text / Material | Needed | design/assets/specs/keyword-system-assets.md |
| ASSET-173 | END OF TURN Trigger Pulse Hook | VFX / Reuse | Needed | design/assets/specs/keyword-system-assets.md |
| ASSET-174 | SILENCE Stripped-Keyword Dissolve | VFX / Particle / Material | Needed | design/assets/specs/keyword-system-assets.md |

### System: auction-system

| Asset ID | Name | Category | Status | Spec File |
|----------|------|----------|--------|-----------|
| ASSET-175 | Auction Ambient Urgency Tone Loop | Audio | Needed | design/assets/specs/auction-system-assets.md |
| ASSET-176 | Accepted Bid Ascending SFX | Audio | Needed | design/assets/specs/auction-system-assets.md |
| ASSET-177 | Auction Red-Zone Countdown Tick Cue | Audio / Reuse | Needed / Reuse | design/assets/specs/auction-system-assets.md |
| ASSET-178 | Timer Reset Reverse-Tick SFX | Audio | Needed | design/assets/specs/auction-system-assets.md |
| ASSET-179 | Auction Won By Self Sting | Audio | Needed | design/assets/specs/auction-system-assets.md |
| ASSET-180 | Auction Won By Opponent Sting | Audio | Needed | design/assets/specs/auction-system-assets.md |
| ASSET-181 | No-Bid Card Gone SFX | Audio | Needed | design/assets/specs/auction-system-assets.md |
| ASSET-182 | Auction Timer Bar Material Exception | UI Material | Needed | design/assets/specs/auction-system-assets.md |
| ASSET-183 | Local Expiry Awaiting-Settlement Pulse | UI Material / Animation | Needed | design/assets/specs/auction-system-assets.md |
| ASSET-184 | Auction Disconnect Grace Overlay | UI | Placeholder | design/assets/specs/auction-system-assets.md |

### System: prism-system

| Asset ID | Name | Category | Status | Spec File |
|----------|------|----------|--------|-----------|
| ASSET-185 | Card Illustration: prism_strike | Card Illustration | Needed | design/assets/specs/prism-system-assets.md |
| ASSET-186 | Card Illustration: prism_reserve | Card Illustration | Needed | design/assets/specs/prism-system-assets.md |
| ASSET-187 | Prism Reward Card-Acquired Shimmer | VFX / Reuse | Needed / Reuse | design/assets/specs/prism-system-assets.md |
| ASSET-188 | Prism Reward Dropped Indicator | UI / Toast | Needed | design/assets/specs/prism-system-assets.md |
| ASSET-189 | Prism Set Respawn Pulse | VFX | Placeholder | design/assets/specs/prism-system-assets.md |
| ASSET-190 | Prism Strike Projectile | VFX / Projectile | Needed | design/assets/specs/prism-system-assets.md |
| ASSET-191 | Prism Reserve Bar Ping | VFX / UI Overlay | Needed | design/assets/specs/prism-system-assets.md |
| ASSET-192 | Prism Strike Reward Icon | UI Icon | Needed | design/assets/specs/prism-system-assets.md |
| ASSET-193 | Prism Reserve Reward Icon | UI Icon | Needed | design/assets/specs/prism-system-assets.md |
| ASSET-194 | Prism Random Draw Reward Icon | UI Icon | Needed | design/assets/specs/prism-system-assets.md |

### System: game-session-system

| Asset ID | Name | Category | Status | Spec File |
|----------|------|----------|--------|-----------|
| ASSET-195 | Title / Lobby Backdrop | UI Background | Placeholder | design/assets/specs/game-session-system-assets.md |
| ASSET-196 | Room Code Display Chip | UI | Needed | design/assets/specs/game-session-system-assets.md |
| ASSET-197 | Room Code Copy Icon | UI Icon | Needed | design/assets/specs/game-session-system-assets.md |
| ASSET-198 | Join Room Input Frame | UI | Needed | design/assets/specs/game-session-system-assets.md |
| ASSET-199 | Lobby Player Slot Panel States | UI | Needed | design/assets/specs/game-session-system-assets.md |
| ASSET-200 | Class Browser Carousel Arrows | UI Icon | Needed | design/assets/specs/game-session-system-assets.md |
| ASSET-201 | Lobby Timer Progress Bar Material | UI Material | Needed | design/assets/specs/game-session-system-assets.md |
| ASSET-202 | Lobby Cancel Confirmation Overlay | UI Overlay | Placeholder | design/assets/specs/game-session-system-assets.md |
| ASSET-203 | Lobby Inline Error Flash Material | UI Material | Needed | design/assets/specs/game-session-system-assets.md |
| ASSET-204 | Button Loading Spinner | UI Icon / Animation | Needed | design/assets/specs/game-session-system-assets.md |
| ASSET-205 | Simultaneous Class Reveal Flash | VFX / UI Material | Needed | design/assets/specs/game-session-system-assets.md |
| ASSET-206 | Session Cancelled - Opponent Left Overlay | UI Overlay | Placeholder | design/assets/specs/game-session-system-assets.md |
| ASSET-207 | Session Cancelled - Timeout Overlay | UI Overlay | Placeholder | design/assets/specs/game-session-system-assets.md |
| ASSET-208 | Reconnect Snapshot Rebuild Overlay | UI Overlay | Placeholder | design/assets/specs/game-session-system-assets.md |
| ASSET-209 | Opponent Disconnected Grace Overlay | UI Overlay | Placeholder | design/assets/specs/game-session-system-assets.md |
| ASSET-210 | Opponent Reconnected Toast | UI / Toast | Placeholder | design/assets/specs/game-session-system-assets.md |
| ASSET-211 | GAME_OVER Result Panel Placeholder - UX not designed | UI Overlay | Blocked | design/assets/specs/game-session-system-assets.md |
| ASSET-212 | Outcome Badge Placeholder Set - blocked pending result-screen UX | UI | Blocked | design/assets/specs/game-session-system-assets.md |
| ASSET-213 | Post-Match Action Button Placeholder - UX not designed | UI | Blocked | design/assets/specs/game-session-system-assets.md |
| ASSET-214 | Post-Match Objective Reveal Placeholder - blocked pending result-screen UX | UI / VFX | Blocked | design/assets/specs/game-session-system-assets.md |

### System: shared-fonts-materials-shaders

| Asset ID | Name | Category | Status | Spec File |
|----------|------|----------|--------|-----------|
| ASSET-215 | Project Display Font - Regular | Font | Needed | design/assets/specs/shared-fonts-materials-shaders-assets.md |
| ASSET-216 | Project Display Font - Bold | Font | Needed | design/assets/specs/shared-fonts-materials-shaders-assets.md |
| ASSET-217 | Keyboard Focus Ring Material | UI Material | Needed | design/assets/specs/shared-fonts-materials-shaders-assets.md |
| ASSET-218 | Button Chrome Material Set | UI Material | Needed | design/assets/specs/shared-fonts-materials-shaders-assets.md |
| ASSET-219 | Shared Timer Bar Material Set | UI Material | Needed | design/assets/specs/shared-fonts-materials-shaders-assets.md |
| ASSET-220 | Card Ghost / Lock Desaturation Shader | WGSL Shader | Needed | design/assets/specs/shared-fonts-materials-shaders-assets.md |
| ASSET-221 | Gold Selection Outline Shader | WGSL Shader | Needed | design/assets/specs/shared-fonts-materials-shaders-assets.md |
| ASSET-222 | Unit Target Outline Material2D | WGSL Shader | Needed | design/assets/specs/shared-fonts-materials-shaders-assets.md |
| ASSET-223 | Colorblind Palette Override Materials | Accessibility Material Set | Placeholder | design/assets/specs/shared-fonts-materials-shaders-assets.md |
| ASSET-224 | Reduced-Motion Animation Policy Map | Accessibility Data Asset | Placeholder | design/assets/specs/shared-fonts-materials-shaders-assets.md |
| ASSET-225 | Audio Bus Settings UI Controls | UI / Settings | Placeholder | design/assets/specs/shared-fonts-materials-shaders-assets.md |
| ASSET-226 | Brightness / Gamma Overlay Material | Accessibility Material | Placeholder | design/assets/specs/shared-fonts-materials-shaders-assets.md |

### System: card-animations

| Asset ID | Name | Category | Status | Spec File |
|----------|------|----------|--------|-----------|
| ASSET-243 | `SpriteAlphaLens` Custom Lens | Rust / Animation Lens | Needed | design/assets/specs/card-animations-assets.md |
| ASSET-244 | `BackgroundColorAlphaLens` Custom Lens | Rust / Animation Lens | Needed | design/assets/specs/card-animations-assets.md |
| ASSET-245 | `SpriteColorLens` Custom Lens | Rust / Animation Lens | Needed | design/assets/specs/card-animations-assets.md |
| ASSET-246 | `TransformScaleXLens` Custom Lens | Rust / Animation Lens | Needed | design/assets/specs/card-animations-assets.md |
| ASSET-247 | `TextColorLens` Custom Lens | Rust / Animation Lens | Needed | design/assets/specs/card-animations-assets.md |
| ASSET-248 | Card Animations Timing Constants Block | Config Data | Needed | design/assets/specs/card-animations-assets.md |
| ASSET-249 | Damage Number Jitter Table | Static Data / Rust Const | Needed | design/assets/specs/card-animations-assets.md |
| ASSET-250 | Damage Number Text Style | Runtime Text / Material | Needed | design/assets/specs/card-animations-assets.md |
| ASSET-251 | Damage Number `DespawnAfter` Timer Component | ECS Component / Data | Needed | design/assets/specs/card-animations-assets.md |
| ASSET-252 | `DamageNumber` Marker Component | ECS Component | Needed | design/assets/specs/card-animations-assets.md |
| ASSET-253 | `PlacementPhaseAnimator` Marker Component | ECS Component | Needed | design/assets/specs/card-animations-assets.md |
| ASSET-254 | `StagedObjectiveRevealQueue` Resource | ECS Resource | Needed | design/assets/specs/card-animations-assets.md |
| ASSET-255 | `GroupDrainedSignal` Message | Rust / ECS Message Type | Needed | design/assets/specs/card-animations-assets.md |
| ASSET-256 | `PlacementRevealAnimReady` Message | Rust / Message Type | Needed | design/assets/specs/card-animations-assets.md |
| ASSET-257 | `ObjectiveDestroyedAnimReady` Message | Rust / Message Type | Needed | design/assets/specs/card-animations-assets.md |
| ASSET-258 | `BoardRebuildRequested` Message | Rust / Message Type | Needed | design/assets/specs/card-animations-assets.md |
| ASSET-259 | `PlacementCancelAllAnimsRequested` Message | Rust / Message Type | Needed | design/assets/specs/card-animations-assets.md |
| ASSET-260 | `DamageNumberSpawnRequested` Message | Rust / Message Type | Needed | design/assets/specs/card-animations-assets.md |
| ASSET-261 | `CardAcquiredAnimReady` Message | Rust / Message Type | Needed | design/assets/specs/card-animations-assets.md |
| ASSET-262 | `SnapBackRequested` Message | Rust / Message Type | Needed | design/assets/specs/card-animations-assets.md |
| ASSET-263 | `HandHideRequested` / `HandShowRequested` Messages | Rust / Message Type | Needed | design/assets/specs/card-animations-assets.md |
| ASSET-264 | Shop/Auction UI Animation Messages (×6) | Rust / Message Type | Needed | design/assets/specs/card-animations-assets.md |
| ASSET-265 | Keyword System Animation Messages (×3) | Rust / Message Type | Needed | design/assets/specs/card-animations-assets.md |
| ASSET-266 | Reserved — card-animations system | — | Needed | design/assets/specs/card-animations-assets.md |

### System: objective-system

| Asset ID | Name | Category | Status | Spec File |
|----------|------|----------|--------|-----------|
| ASSET-267 | Objective Real Identity Reveal Frame | VFX / Overlay | Needed | design/assets/specs/objective-system-assets.md |
| ASSET-268 | Objective Fake Identity Reveal Frame | VFX / Overlay | Needed | design/assets/specs/objective-system-assets.md |
| ASSET-269 | Objective Reveal Hold Backdrop | Material / UI Overlay | Needed | design/assets/specs/objective-system-assets.md |
| ASSET-270 | Objective Owner Real Indicator Glyph | UI / HUD | Needed | design/assets/specs/objective-system-assets.md |
| ASSET-271 | Objective Owner Fake Indicator Glyph | UI / HUD | Needed | design/assets/specs/objective-system-assets.md |
| ASSET-272 | Objective Owner Destroyed Slot Marker | UI / HUD | Needed | design/assets/specs/objective-system-assets.md |
| ASSET-273 | Sang Méprise Slot Reveal Tint — Real | Material / Runtime Tint | Needed | design/assets/specs/objective-system-assets.md |
| ASSET-274 | Sang Méprise Slot Reveal Tint — Fake | Material / Runtime Tint | Needed | design/assets/specs/objective-system-assets.md |
| ASSET-275 | `S2CObjectiveIdentities` Message Type | Rust / Lightyear S2C | Needed | design/assets/specs/objective-system-assets.md |
| ASSET-276 | `S2CSangMepriseReveal` Message Type | Rust / Lightyear S2C | Needed | design/assets/specs/objective-system-assets.md |
| ASSET-277 | `HiddenObjectives` Server Resource | Rust / ECS Resource | Needed | design/assets/specs/objective-system-assets.md |
| ASSET-278 | `ObjectiveDestroyed` Message Type | Rust / Lightyear S2C | Needed | design/assets/specs/objective-system-assets.md |
| ASSET-279 | Objective HP Number Text Style | Runtime Text / Material | Needed | design/assets/specs/objective-system-assets.md |
| ASSET-280 | Objective Identity Reveal Sting | Audio | Needed | design/assets/specs/objective-system-assets.md |

### System: round-state-machine

| Asset ID | Name | Category | Status | Spec File |
|----------|------|----------|--------|-----------|
| ASSET-281 | DRAFT_INITIAL Phase Banner | UI Overlay | Needed | design/assets/specs/round-state-machine-assets.md |
| ASSET-282 | DRAFT_SHOP Phase Banner | UI Overlay | Needed | design/assets/specs/round-state-machine-assets.md |
| ASSET-283 | DRAFT_AUCTION Phase Banner | UI Overlay | Needed | design/assets/specs/round-state-machine-assets.md |
| ASSET-284 | PLACEMENT Phase Banner | UI Overlay | Needed | design/assets/specs/round-state-machine-assets.md |
| ASSET-285 | RESOLUTION Phase Banner | UI Overlay | Needed | design/assets/specs/round-state-machine-assets.md |
| ASSET-286 | Round Number Badge | UI / HUD Element | Needed | design/assets/specs/round-state-machine-assets.md |
| ASSET-287 | GAME_OVER Result Panel Background | UI Background | Needed | design/assets/specs/round-state-machine-assets.md |
| ASSET-288 | WIN Result Text Style | Runtime Text / Material | Needed | design/assets/specs/round-state-machine-assets.md |
| ASSET-289 | LOSS Result Text Style | Runtime Text / Material | Needed | design/assets/specs/round-state-machine-assets.md |
| ASSET-290 | DRAW Result Text Style | Runtime Text / Material | Needed | design/assets/specs/round-state-machine-assets.md |
| ASSET-291 | ResolutionTimeout Result Text Style | Runtime Text / Material | Needed | design/assets/specs/round-state-machine-assets.md |
| ASSET-292 | Disconnection Result Sub-label Text Style | Runtime Text / Material | Needed | design/assets/specs/round-state-machine-assets.md |
| ASSET-293 | Round Number Result Sub-label Text Style | Runtime Text / Material | Needed | design/assets/specs/round-state-machine-assets.md |
| ASSET-294 | `RoundState` Resource | ECS Resource | Needed | design/assets/specs/round-state-machine-assets.md |
| ASSET-295 | `GameOverReason` Enum | Rust Type | Needed | design/assets/specs/round-state-machine-assets.md |
| ASSET-296 | PLACEMENT Begin Tension Sting | Audio | Needed | design/assets/specs/round-state-machine-assets.md |

### Cards: Current cards.json Illustration Specs

| Asset ID | Name | Category | Status | Spec File |
|----------|------|----------|--------|-----------|
| ASSET-227 | Card Illustration: iop_knight_001 | Card Illustration | Generated Placeholder | design/assets/specs/cards/iop_knight_001.md |
| ASSET-228 | Card Illustration: sacrier_foot_002 | Card Illustration | Generated Placeholder | design/assets/specs/cards/sacrier_foot_002.md |
| ASSET-229 | Card Illustration: cra_piercing_shot_003 | Card Illustration | Generated Placeholder | design/assets/specs/cards/cra_piercing_shot_003.md |
| ASSET-230 | Card Illustration: xelor_time_trap_004 | Card Illustration | Generated Placeholder | design/assets/specs/cards/xelor_time_trap_004.md |
| ASSET-231 | Card Illustration: gobball_sturdy_005 | Card Illustration | Generated Placeholder | design/assets/specs/cards/gobball_sturdy_005.md |
| ASSET-232 | Card Illustration: sadida_rose_field_006 | Card Illustration | Generated Placeholder | design/assets/specs/cards/sadida_rose_field_006.md |
| ASSET-233 | Card Illustration: ecaflip_decree_007 | Card Illustration | Generated Placeholder | design/assets/specs/cards/ecaflip_decree_007.md |
| ASSET-234 | Card Illustration: iop_double_face_008 | Card Illustration | Generated Placeholder | design/assets/specs/cards/iop_double_face_008.md |
| ASSET-235 | Card Illustration: neutral_tofu_scout_101 | Card Illustration | Generated Placeholder | design/assets/specs/hand-ui-assets.md |
| ASSET-236 | Card Illustration: neutral_wabbit_guard_102 | Card Illustration | Generated Placeholder | design/assets/specs/hand-ui-assets.md |
| ASSET-237 | Card Illustration: neutral_market_runner_103 | Card Illustration | Generated Placeholder | design/assets/specs/hand-ui-assets.md |
| ASSET-238 | Card Illustration: neutral_training_banner_104 | Card Illustration | Generated Placeholder | design/assets/specs/hand-ui-assets.md |
| ASSET-239 | Card Illustration: neutral_guild_errand_105 | Card Illustration | Generated Placeholder | design/assets/specs/hand-ui-assets.md |
| ASSET-240 | Card Illustration: neutral_paddock_bruiser_106 | Card Illustration | Generated Placeholder | design/assets/specs/hand-ui-assets.md |
| ASSET-241 | Card Illustration: neutral_vault_sentry_107 | Card Illustration | Generated Placeholder | design/assets/specs/hand-ui-assets.md |
| ASSET-242 | Card Illustration: neutral_crowned_mercenary_108 | Card Illustration | Generated Placeholder | design/assets/specs/hand-ui-assets.md |
