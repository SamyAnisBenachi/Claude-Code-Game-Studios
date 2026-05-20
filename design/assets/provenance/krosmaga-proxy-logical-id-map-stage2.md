# Krosmaga Proxy Logical ID Map - Stage 2

> Dev-only provenance map produced for PROMPT 1512. Extends
> `krosmaga-proxy-logical-id-map-stage1.md` beyond its 50 Stage 1 P1
> `USE_DEV_PROXY` rows. Documentation-only. No Krosmaga payload is copied,
> imported, approved, or released by this file. No `assets/**`, no source
> code, no Cargo, no sprint/session/QA paperwork is touched.

## Legal And Runtime Boundary (Reaffirmed)

- Every Stage 2 mapped candidate carries `source_class=licensed_krosmaga_dev_proxy`,
  `workflow_status=needed`, `release_class=dev_only` -- identical to Stage 1.
- Candidate paths remain read-only references into the local Krosmaga bank
  at `D:\_GAMES\Ankama\Krosmaga\_extracted\` (or the nested
  `_extracted\_extracted\AssetStudio_All\...` mirror used by the source
  manifests). No Krosmaga file is copied into the repo.
- Dev-pack materialization may only occur under a future gitignored
  `dev-assets/krosmaga-proxy/` and must stay excluded from release packaging.
- Release-scan validator remains the enforcement boundary; this map is
  documentation, not enforcement. None of these rows advance any logical ID
  to Done, Approved, or release-allowed.

## Inputs Used (Stage 2)

- `design/assets/provenance/krosmaga-proxy-logical-id-map-stage1.md`
  (Stage 1 mapped rows, ambiguous list, missing list).
- `design/assets/provenance/logical-id-index.md` (`lid_*` reserved IDs).
- Krosmaga catalogs already cited by Stage 1:
  - `krosmaga_ui_asset_catalog.csv`
  - `krosmaga_vfx_particle_gaf_asset_catalog.csv`
  - `krosmaga_fonts_models_misc_asset_catalog.csv`
  - `krosmaga_card_visual_to_board_model_mapping.csv`
  - `converted_audio_by_use_manifest.csv`
  - `rebuilt_board_sprites_manifest.csv`
  - `rebuilt_board_animations_chains.csv`
  - `krosmaga_full_nonprefab_asset_inventory.csv` (only for the prism token
    lookup; no recursive scan was performed beyond targeted grep).

Source manifests were read first; the Krosmaga bank tree itself was only
touched through targeted greps against the catalogs above.

## Stage 2 Coverage Totals

| Metric                                                    | Count |
|-----------------------------------------------------------|------:|
| Stage 2 newly mapped rows (this file)                     |    13 |
| Of which: AMBIGUOUS-promoted (best-candidate, manual review still required) | 12 |
| Of which: MISSING-promoted (single plausible candidate)   |     1 |
| Stage 2 still-ambiguous source rows                       |    92 |
| Stage 2 still-missing source rows (no-art-needed CCGS originals) |    80 |
| `lid_*` index rows cross-walked to existing Stage 1 paths |    16 |
| Manual-review-required Stage 2 rows                       |    13 |

Stage 1 totals are unchanged. Stage 2 promotes a subset of Stage 1's
"Top Ambiguous Rows" (12 rows) and one Stage 1 "Top Missing Row" to
mapped entries with explicit lower confidence; the remaining 92 source
ambiguous rows and 80 source missing rows are left unmapped intentionally
for the reasons documented below.

## A. Stage 2 Mapped (AMBIGUOUS-promoted)

Each row below was listed under Stage 1 "Top Ambiguous Rows" with a
best-candidate path already identified. Stage 2 promotes them to mapped
entries with `match quality = AMBIGUOUS-PROMOTED / score <= 60` so dev
tooling can see them without misreading them as approved bindings.

| logical_id | CCGS surface/use | Krosmaga candidate path | source report/manifest | match quality | conversion need | dev-only/legal note | consumer priority |
|---|---|---|---|---|---|---|---|
| `ui.shop_auction.rarity.gem.rare` | rarity gem chrome (`ASSET-002`) | `D:\_GAMES\Ankama\Krosmaga\_extracted\_extracted\AssetStudio_All\assets\art\ui\textures\collection\deck\manacurve\manacurveblue\manaCurveBlue @1904368712017522139.png` | PROMPT-1258; `design/assets/specs/shop-auction-ui-assets.md`; `krosmaga_ui_asset_catalog.csv` | AMBIGUOUS-PROMOTED / score 55 | Manual review required: Krosmaga mana-curve gem is a deck-builder UI element, not a card rarity gem. Use only as colour/silhouette anchor; resize/crop/atlas repack and Bevy material wiring required. | `source_class=licensed_krosmaga_dev_proxy`; `workflow_status=needed`; `release_class=dev_only`; do not copy to `assets/**`; not release-approved. | P1 polish unblocker |
| `ui.shop_auction.rarity.gem.epic` | rarity gem chrome (`ASSET-003`) | `D:\_GAMES\Ankama\Krosmaga\_extracted\_extracted\AssetStudio_All\assets\art\ui\textures\collection\deck\manacurve\manacurvepurple\manaCurvePurple @1481286478496480667.png` | PROMPT-1258; `design/assets/specs/shop-auction-ui-assets.md`; `krosmaga_ui_asset_catalog.csv` | AMBIGUOUS-PROMOTED / score 55 | Same caveat as `rare` -- mana-curve element, colour anchor only. | `source_class=licensed_krosmaga_dev_proxy`; `workflow_status=needed`; `release_class=dev_only`; do not copy to `assets/**`; not release-approved. | P1 polish unblocker |
| `ui.shop_auction.rarity.gem.legendary` | rarity gem chrome (`ASSET-004`) | `D:\_GAMES\Ankama\Krosmaga\_extracted\_extracted\AssetStudio_All\assets\art\ui\textures\collection\deck\card_mini_infinite\card_mini_Infinite @3264540028981720921.png` | PROMPT-1258; `design/assets/specs/shop-auction-ui-assets.md`; `krosmaga_ui_asset_catalog.csv` | AMBIGUOUS-PROMOTED / score 50 | Manual review required: Krosmaga "Infinite" badge is a deck-collection marker, not a rarity gem. Use only as silhouette/shape anchor. | `source_class=licensed_krosmaga_dev_proxy`; `workflow_status=needed`; `release_class=dev_only`; do not copy to `assets/**`; not release-approved. | P1 polish unblocker |
| `ui.shop_auction.draft.shop.slot.well.highlight.strip` | shop slot highlight (`ASSET-005`) | `D:\_GAMES\Ankama\Krosmaga\_extracted\_extracted\AssetStudio_All\assets\art\particles\materials\lights\particleadd_glow_04\Glow_04_GrayA @-377674893314949650.png` | PROMPT-1258; `design/assets/specs/shop-auction-ui-assets.md`; `krosmaga_vfx_particle_gaf_asset_catalog.csv` | AMBIGUOUS-PROMOTED / score 58 | Re-uses the same `Glow_04` source already mapped Stage 1 for `vfx.shop_auction.gold.bloom.glow`. Manual review required to keep the two surfaces visually distinct; resize/crop/atlas repack. | `source_class=licensed_krosmaga_dev_proxy`; `workflow_status=needed`; `release_class=dev_only`; do not copy to `assets/**`; not release-approved. | P1 polish unblocker |
| `ui.shop_auction.auction.panel.background` | auction panel background (`ASSET-006`) | `D:\_GAMES\Ankama\Krosmaga\_extracted\_extracted\AssetStudio_All\assets\art\ui\textures\common\panel_black2\panel_black2 @2374755845677634530.png` | PROMPT-1258; `design/assets/specs/shop-auction-ui-assets.md`; `krosmaga_ui_asset_catalog.csv` | AMBIGUOUS-PROMOTED / score 60 | Generic dark panel; manual review for nine-slice intent and grain compatibility before any dev-pack materialization. | `source_class=licensed_krosmaga_dev_proxy`; `workflow_status=needed`; `release_class=dev_only`; do not copy to `assets/**`; not release-approved. | P1 polish unblocker |
| `ui.shop_auction.auction.panel.border.ramp.tiles.tiles` | auction panel border ramp (`ASSET-007`) | `D:\_GAMES\Ankama\Krosmaga\_extracted\_extracted\AssetStudio_All\assets\resources\art\ui\menus\duplicated\optionui\panel_black_dropdown @-4658742064776812214.png` | PROMPT-1258; `design/assets/specs/shop-auction-ui-assets.md`; `krosmaga_ui_asset_catalog.csv` | AMBIGUOUS-PROMOTED / score 55 | Krosmaga dropdown chrome; only the outer border ramp is potentially useful. Manual review of nine-slice corners required. | `source_class=licensed_krosmaga_dev_proxy`; `workflow_status=needed`; `release_class=dev_only`; do not copy to `assets/**`; not release-approved. | P1 polish unblocker |
| `vfx.shop_auction.gold.particle.glow` | gold particle glow (`ASSET-008`) | `D:\_GAMES\Ankama\Krosmaga\_extracted\_extracted\AssetStudio_All\assets\art\particles\textures\fire\firesparkles_01_dist_rgba_2x2\FireSparkles_01_Dist_RGBA_2x2 @928712014495333451.png` | PROMPT-1258; `design/assets/specs/shop-auction-ui-assets.md`; `krosmaga_vfx_particle_gaf_asset_catalog.csv` | AMBIGUOUS-PROMOTED / score 58 | Fire-sparkles atlas; manual review to retint to gold; resize/crop/atlas repack and Bevy material wiring required. | `source_class=licensed_krosmaga_dev_proxy`; `workflow_status=needed`; `release_class=dev_only`; do not copy to `assets/**`; not release-approved. | P1 polish unblocker |
| `audio.shop_auction.draft.initial.entry.sting` | DRAFT_INITIAL entry sting (`ASSET-012`) | `D:\_GAMES\Ankama\Krosmaga\_extracted\_extracted\converted_audio_by_use_v4_final\03_ui_gameplay\ui\main_bank\039_gui_dry_draft_menu_back.wav` | PROMPT-1258; `design/assets/specs/shop-auction-ui-assets.md`; `converted_audio_by_use_manifest.csv` | AMBIGUOUS-PROMOTED / score 50 | Same `039_gui_dry_draft_menu_back.wav` is the best Stage 1 candidate for several adjacent shop/auction stings. Manual review needed to disambiguate per-event timing; OGG conversion + loudness pass required. Dev-only. | `source_class=licensed_krosmaga_dev_proxy`; `workflow_status=needed`; `release_class=dev_only`; do not copy to `assets/**`; not release-approved. | P2 timing-only proxy |
| `audio.shop_auction.draft.initial.purchase.chime` | DRAFT_INITIAL purchase chime (`ASSET-013`) | `D:\_GAMES\Ankama\Krosmaga\_extracted\_extracted\converted_audio_by_use_v4_final\03_ui_gameplay\ui\main_bank\039_gui_dry_draft_menu_back.wav` | PROMPT-1258; `design/assets/specs/shop-auction-ui-assets.md`; `converted_audio_by_use_manifest.csv` | AMBIGUOUS-PROMOTED / score 50 | Same caveat as `entry.sting`; same source WAV; manual disambiguation required. | `source_class=licensed_krosmaga_dev_proxy`; `workflow_status=needed`; `release_class=dev_only`; do not copy to `assets/**`; not release-approved. | P2 timing-only proxy |
| `audio.shop_auction.draft.initial.budget.depleted.bell` | DRAFT_INITIAL budget depleted bell (`ASSET-014`) | `D:\_GAMES\Ankama\Krosmaga\_extracted\_extracted\converted_audio_by_use_v4_final\03_ui_gameplay\ui\main_bank\039_gui_dry_draft_menu_back.wav` | PROMPT-1258; `design/assets/specs/shop-auction-ui-assets.md`; `converted_audio_by_use_manifest.csv` | AMBIGUOUS-PROMOTED / score 48 | Same caveat; bell semantics are weakly matched by `draft_menu_back`. Manual review strongly recommended before any dev-pack use. | `source_class=licensed_krosmaga_dev_proxy`; `workflow_status=needed`; `release_class=dev_only`; do not copy to `assets/**`; not release-approved. | P2 timing-only proxy |
| `audio.shop_auction.draft.shop.entry.phrase` | DRAFT_SHOP entry phrase (`ASSET-015`) | `D:\_GAMES\Ankama\Krosmaga\_extracted\_extracted\converted_audio_by_use_v4_final\03_ui_gameplay\ui\main_bank\039_gui_dry_draft_menu_back.wav` | PROMPT-1258; `design/assets/specs/shop-auction-ui-assets.md`; `converted_audio_by_use_manifest.csv` | AMBIGUOUS-PROMOTED / score 50 | Shared source with sibling rows. Manual review required for per-event distinction. | `source_class=licensed_krosmaga_dev_proxy`; `workflow_status=needed`; `release_class=dev_only`; do not copy to `assets/**`; not release-approved. | P2 timing-only proxy |
| `audio.shop_auction.draft.shop.purchase.chime` | DRAFT_SHOP purchase chime (`ASSET-016`) | `D:\_GAMES\Ankama\Krosmaga\_extracted\_extracted\converted_audio_by_use_v4_final\03_ui_gameplay\ui\main_bank\039_gui_dry_draft_menu_back.wav` | PROMPT-1258; `design/assets/specs/shop-auction-ui-assets.md`; `converted_audio_by_use_manifest.csv` | AMBIGUOUS-PROMOTED / score 50 | Shared source with sibling rows. Manual review required. | `source_class=licensed_krosmaga_dev_proxy`; `workflow_status=needed`; `release_class=dev_only`; do not copy to `assets/**`; not release-approved. | P2 timing-only proxy |

## B. Stage 2 Mapped (MISSING-promoted)

One Stage 1 "Top Missing" row has a defensible single-candidate match in
the Krosmaga bank; it is promoted to a Stage 2 mapping with explicit low
confidence. The other nine Stage 1 missing rows are CCGS-original board
chrome that has no 1:1 Krosmaga analogue and remain MISSING (see
Section C).

| logical_id | CCGS surface/use | Krosmaga candidate path | source report/manifest | match quality | conversion need | dev-only/legal note | consumer priority |
|---|---|---|---|---|---|---|---|
| `board.env.prism.idle.32x32` | board prism idle marker (`ASSET-032` env_prism_idle_32x32) | `D:\_GAMES\Ankama\Krosmaga\_extracted\_extracted\AssetStudio_All\assets\art\cards\illustrations\spells\prisme_de_pioche_token\prisme_de_pioche_token @1871156936751072541.png` | PROMPT-1258; `design/assets/specs/board-rendering-assets.md`; `krosmaga_full_nonprefab_asset_inventory.csv` | AMBIGUOUS-PROMOTED / score 45 | Manual review required: source is a card-illustration token, not a board-cell sprite; significant downscale (~430->32 px) + alpha cleanup needed; conditional fallback only. Resize/crop/atlas repack and Bevy material wiring required. | `source_class=licensed_krosmaga_dev_proxy`; `workflow_status=needed`; `release_class=dev_only`; do not copy to `assets/**`; not release-approved. | P1 polish unblocker (board prism) |

## C. Rows Explicitly Left MISSING (CCGS originals)

These Stage 1 "Top Missing" rows have no 1:1 Krosmaga analogue and must be
produced by studio art. Krosmaga uses a different board model (5 lanes with
end-of-lane prisms framed by per-class chrome), so per-cell tiles, lane
dividers/labels, abstract objective-state sprites, generic unit bases,
trap-facedown tiles, and the lane field-wash strip are CCGS-mechanical
inventions. Forcing a Krosmaga proxy would mislead downstream consumers.

| source logical_id (Stage 1 MISSING) | asset | reason still MISSING |
|---|---|---|
| `board.env.lane.divider.64x80` | `ASSET-023` env_lane_divider_64x80 | Krosmaga lanes are framed by per-class board chrome, not a reusable 64x80 divider tile. No bank entry matches this rasterised slice. |
| `board.env.lane.number.label.sprites` | `ASSET-024` env_lane_number_label_01..05 | Krosmaga does not number its lanes UI-side; no per-lane numeric sprite exists in `krosmaga_ui_asset_catalog.csv`. |
| `board.env.objective.unknown.64x96` | `ASSET-029` env_objective_unknown_64x96 | CCGS hidden-objective state has no Krosmaga analogue (Krosmaga objectives are always class-revealed prism statues). |
| `board.env.objective.real.reveal.64x96` | `ASSET-030` env_objective_real_reveal_64x96 | Same: per-state objective sprite is a CCGS-original mechanic. Class figurines from Stage 1 are not a substitute (different intent / silhouette). |
| `board.env.objective.fake.crack.64x96` | `ASSET-031` env_objective_fake_crack_64x96 | "Fake-objective crack" is CCGS-original (Sang Méprise); no parallel in the bank. |
| `board.unit.base.player.48x16` | `ASSET-033` ui_unit_base_player_a_48x16 | Krosmaga uses class-tinted unit shadows baked into each sprite; no separate 48x16 base atlas tile exists. |
| `board.unit.base.player.48x16` | `ASSET-034` ui_unit_base_player_b_48x16 | Same reason as `ASSET-033`. |
| `board.trap.tile.facedown.32x32` | `ASSET-035` ui_trap_tile_facedown_32x32 | Krosmaga has no facedown trap mechanic on the board grid; no source candidate matches the intent. |
| `board.field.wash.lane.512x80` | `ASSET-037` ui_field_wash_lane_512x80 | Lane-wash overlay (a CCGS visual tell) has no Krosmaga analogue; bank flashes (`Flash_10/11`) are point-frame effects, not lane-length strips. |

These nine rows are flagged `no-art-needed-from-Krosmaga / needs-studio-original`
and remain unmapped intentionally.

## D. `lid_*` Index Cross-Walk (Stage 2)

`design/assets/provenance/logical-id-index.md` reserves a Krosmaga-proxy
section that is intentionally empty in the initial index. The rows below
cross-walk those reserved `lid_*` IDs to Stage 1 candidate paths already
documented in the bank, so a future dev-pack materialization can find the
single source-of-truth in one place. **No dev-pack entry is added here**:
the schema requires the entry to live under `dev_pack_entries:` in the
index itself once the surface adopts the layer per ADR-025. This table
is provenance documentation only.

| `lid_*` (logical-id-index) | Cross-walks to Stage 1 row | Bank candidate path (already documented Stage 1) |
|---|---|---|
| `lid_hud_class_figurine_iop` | Stage 1 `ui.class.figurine.iop` | `rebuilt_board_animations\godiopally` |
| `lid_hud_class_figurine_cra` | Stage 1 `ui.class.figurine.cra` | `rebuilt_board_animations\godcraally` |
| `lid_hud_class_figurine_sacrier` | Stage 1 `ui.class.figurine.sacrier` | `rebuilt_board_animations\godsacrieurally` |
| `lid_hud_class_figurine_xelor` | Stage 1 `ui.class.figurine.xelor` | `rebuilt_board_animations\godxelorally` |
| `lid_hud_class_figurine_ecaflip` | Stage 1 `ui.class.figurine.ecaflip` | `rebuilt_board_animations\godecaflipally` |
| `lid_hud_class_figurine_sadida` | Stage 1 `ui.class.figurine.sadida` | `rebuilt_board_animations\godsadidaally` |
| `lid_hud_class_figurine_unknown` | -- (no Stage 1 row) | MISSING -- no neutral/unknown class figurine in the Krosmaga bank; CCGS-original placeholder required. |
| `lid_shop_panel_chrome` | Stage 2 `ui.shop_auction.auction.panel.background` (section A) | `assets\art\ui\textures\common\panel_black2\panel_black2 @2374755845677634530.png` (AMBIGUOUS-PROMOTED) |
| `lid_auction_panel_chrome` | Stage 2 `ui.shop_auction.auction.panel.background` | Same path; manual review for nine-slice differences. |
| `lid_auction_bid_button_idle` | -- (no Stage 1 row) | Still AMBIGUOUS in PROMPT-1258 source; no Stage 2 promotion. |
| `lid_auction_bid_button_hover` | -- | Still AMBIGUOUS; no Stage 2 promotion. |
| `lid_auction_bid_button_locked` | -- | Still AMBIGUOUS; no Stage 2 promotion. |
| `lid_board_objective_unknown_64x96` | -- (Section C MISSING) | MISSING per Section C; do not bind. |
| `lid_board_objective_real_reveal_64x96` | -- (Section C MISSING) | MISSING per Section C; do not bind. |
| `lid_overlay_targeting_marker_real` | -- (no Stage 1 row; Sang Méprise surface) | Still AMBIGUOUS in PROMPT-1258 source; CCGS-original strongly preferred. |
| `lid_overlay_targeting_marker_fake` | -- | Same. CCGS-original strongly preferred. |

The 16 `lid_*` cross-walks above do not add any new candidate path beyond
what Stage 1 and Section A/B/C already cite; they only state which existing
documented candidate a future dev-pack adoption would point to.

## Why The Remaining 92 + 80 Rows Stay Unmapped

- **Custom Bevy UI materials and bid/result chrome.** Most still-ambiguous
  rows in PROMPT-1258 require a UX-locked panel shape, nine-slice geometry,
  or button-state set that no single Krosmaga sprite delivers. Promoting
  them would couple downstream consumers to a misleading silhouette.
- **Result-screen chrome (`ASSET-211..214`).** Still blocked behind the
  result-screen UX wave per `logical-id-index.md`. Until the UX lock is in,
  no Krosmaga panel is the right shape.
- **Per-card illustrations beyond Stage 1's P1 set.** Each per-card row
  needs a 1:1 source-art match by character/spell name; Stage 1 covered the
  P1 wave (7 cards). Stage 2 deliberately does not opportunistically bind
  P2/P3 cards because manual review per card is required and the cost of
  miswiring per-card silhouettes is high.
- **CCGS-original board mechanics (Section C).** Lanes, abstract objective
  states, unit-base shadow strips, facedown traps, lane-wash overlays --
  these are CCGS mechanical inventions with no Krosmaga 1:1.
- **Most audio rows beyond Stage 1's four timing proxies and the four
  Stage 2 shared-source promotions.** Per-event SFX disambiguation requires
  listening to the WAVs; static index scan cannot promote them safely.

## Validation Notes

- Static file/path review only; no Cargo, no asset copy, no Bevy build.
- Existing manifests/indexes were preferred over recursive bank scans.
  Targeted greps were used only against the catalogs cited above.
- No Krosmaga source file was copied or modified.
- No runtime asset, Bevy code, server/shared code, sprint status, session
  state, or production plan file was edited.
- Markdown table rows parse cleanly (no embedded `|` outside code spans).

1512: KROSMAGA-PROXY-LOGICAL-ID-MAP-STAGE-2: COMPLETE
