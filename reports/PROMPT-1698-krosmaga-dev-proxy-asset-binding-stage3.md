# PROMPT 1698 — KROSMAGA-DEV-PROXY-ASSET-BINDING-STAGE3-RECOVERY

- Worktree: `D:/Tmp/wt-1698`
- Branch: `work/krosmaga-dev-proxy-stage3-binding-1698`
- Base: `origin/main@f9324431`
- Status: SHIPPED (local commit; push attempted — see Push section)

## Context Recovery

PROMPT 1689 crashed before producing output. This prompt advances Stage 3 asset
binding readiness from the current `origin/main` baseline.

### Confirmed baseline state (all in main before this prompt)

| Component | Commit | Status |
|---|---|---|
| Stage 1 logical-id map | `a9b54eda` (via PROMPT 1576) | ✅ in main |
| Stage 2 validator + candidate manifest | `a9b54eda` (via PROMPT 1576) | ✅ in main |
| Stage 3 validator infrastructure (PROMPT 1585) | `0eba48d9` | ✅ in main |
| Stage 3 demo fixture (2-entry iop/cra subset) | `0eba48d9` | ✅ in main |

The `validate_dev_proxy_pack.py` already had 62 tests covering all Stage 3 optional
fields (`atlas_binding`, `sprite_sheets`, `license_provenance`, `stage_readiness`).
The `dev-proxy-pack-stage3-candidate.json` fixture had 5 entries (iop+cra figurines,
card_frame_common, board_cell_idle, hud_obj_dot) — a thin demonstration slice.

## Work Done

### Gap reconciliation against 4 UI lanes

The `design/assets/provenance/logical-id-index.md` defines 37 logical IDs across
all active UI lanes. Cross-referenced against both the Stage 1 Krosmaga map
(`krosmaga-proxy-logical-id-map-stage1.md`) and the Stage 2 candidate manifest:

| Lane | Total LIDs | Stage3-bindable (good/exact/needs_conv proxy) | Stage2 (ambiguous) | Stage1 (missing/blocked) |
|------|---:|---:|---:|---:|
| Hand / Card frames | 9 | 1 (`lid_card_frame_common`) | 0 | 8 (no Krosmaga proxies for rare/epic/legendary/badge/icon rows) |
| HUD | 12 | 6 (class figurines iop→sadida) | 1 (`lid_hud_objective_dot_real_revealed`) | 5 (unknown figurine + 3 obj dots + phase timer) |
| Shop / Auction | 6 | 0 | 1 (`lid_auction_panel_chrome`) | 5 (shop chrome + 3 bid button states + slot well) |
| Board | 9 | 0 | 0 | 9 (all missing — CCGS-original art path) |
| Overlays | 5 | 0 | 1 (`lid_overlay_targeting_marker_real`) | 4 (fake marker + 3 result chromes blocked on UX) |

**Total bindable at stage3:** 7 entries (6 HUD figurines + 1 card frame common).
All other LIDs block at stage2 (ambiguous, needs art-lead review) or stage1
(no proxy identified or UX spec blocked).

### Implementation: expand `dev-proxy-pack-stage3-candidate.json`

Expanded the fixture from 5 entries to **17 entries** covering all 4 UI lanes:

**Newly added stage3_binding entries (atlas_binding declared):**
- `lid_hud_class_figurine_sacrier` — `god_medallion_sacri_500.png` → `hud_class_figurines_v1` frame 2
- `lid_hud_class_figurine_xelor` — `god_medallion_xelor_500.png` → frame 3
- `lid_hud_class_figurine_ecaflip` — `god_medallion_eca_500.png` → frame 4
- `lid_hud_class_figurine_sadida` — `god_medallion_sadida_500.png` → frame 5

All four have `match_quality: good` in Stage 1 map; source paths carried from
`dev-proxy-pack-stage2-candidate.json`. The full 6-class HUD figurine set
(iop, cra, sacrier, xelor, ecaflip, sadida) is now stage3_binding in the fixture.

**Newly added stage2_candidate entries (ambiguous — manual review required):**
- `lid_auction_panel_chrome` — `panel_black2.png` candidate; nine-patch split unknown
- `lid_overlay_targeting_marker_real` — Flash_11 candidate; ASSET-129/205 ambiguity
- `lid_hud_objective_dot_real_revealed` — retained from prior fixture

**Newly added stage1_logical entries (no proxy available or UX blocked):**
- `lid_card_frame_rare/epic/legendary` — no Krosmaga candidate in Stage 1 map
- `lid_hud_class_figurine_unknown` — no proxy; CCGS-original needed
- `lid_hud_objective_dot_unknown` — no proxy
- `lid_shop_panel_chrome` — ambiguous in Stage 1 map; surfaced as missing here
- `lid_result_panel_chrome_win` — result-screen UX blocked (ASSET-211/213)
- `lid_board_cell_idle_32x32` — retained from prior fixture (no proxy intended)

## Validation

- `python -m pytest tools/asset-provenance/test_validate_dev_proxy_pack.py -q` → **62 passed**
- `python tools/asset-provenance/validate_dev_proxy_pack.py tools/asset-provenance/fixtures/dev-proxy-pack-stage3-candidate.json` → **exit 0 (PASS)**
- `python -m unittest tools/asset-provenance/test_check_release.py -q` → **27 passed** (regression clean)
- `git diff --check` → clean (LF warning only — expected on Windows)
- Path allowlist review: only `tools/asset-provenance/fixtures/dev-proxy-pack-stage3-candidate.json` + `reports/PROMPT-1698-*.md` modified. No `assets/**`, no client/server Rust source, no sprint/session state.

## Files Touched

```
M tools/asset-provenance/fixtures/dev-proxy-pack-stage3-candidate.json
A reports/PROMPT-1698-krosmaga-dev-proxy-asset-binding-stage3.md
```

## Rollout Plan for Remaining Stage 3 Gaps

### Ready to ship immediately (no further research)

Nothing — all 7 bindable entries are now in the fixture with atlas_binding.

### Stage 2 → Stage 3 promotion (requires art-lead review)

| LID | Blocker | Action |
|---|---|---|
| `lid_auction_panel_chrome` | Nine-patch split dimensions unknown | Art-lead reviews `panel_black2.png` dimensions; adds `frame_size_px` + `frame_origin_px` |
| `lid_hud_objective_dot_real_revealed` | Multiple flash variants; art disambiguation needed | Art-lead selects one Flash variant; promotes to stage3_binding with atlas coords |
| `lid_overlay_targeting_marker_real` | ASSET-129/205 share the same sprite file | Art-lead confirms single-source reuse is acceptable; adds atlas_binding |

### Stage 1 → Stage 2 (proxy search needed)

| LID | Gap | Action |
|---|---|---|
| `lid_card_frame_rare/epic/legendary` | No Krosmaga frame variant identified | Search `assets/art/cards/frames/` in bank for `frame_rare/epic/legendary`; if found, add to Stage 1 map |
| `lid_hud_phase_timer_bar` | Not in Stage 1 map | Audit `assets/art/ui/menus/` or `assets/art/ui/hud/` in bank for timer bar chrome |
| `lid_hud_objective_dot_fake_revealed/destroyed` | Not in Stage 1 map | Search particle/flash bank for dot candidates |

### Permanently blocked (no proxy path)

| LID | Reason |
|---|---|
| `lid_result_panel_chrome_win/loss/draw` | Blocked on result-screen UX spec (ASSET-211–214) |
| `lid_board_*` (8 entries) | No Krosmaga reuse intended; CCGS-original art path |
| `lid_shop_slot_well_idle`, `lid_auction_bid_button_*` | No concrete Krosmaga candidate in Stage 1 bank scan |
| `lid_hud_class_figurine_unknown` | No Krosmaga proxy for the unknown/not-selected state |

## Non-claims

- No Krosmaga source bytes copied, generated, or materialized.
- No row promoted to `approved`, `release_allowed`, or `studio_original`.
- No client UI lane rewired; the expanded fixture is data, not a runtime binding.
- The release-scan validator still hard-fails any packaged build resolving a
  logical asset through a Krosmaga proxy.

## Push

- Local commit: see `git log --oneline work/krosmaga-dev-proxy-stage3-binding-1698`
- Pushing to `origin/work/krosmaga-dev-proxy-stage3-binding-1698`.

1698: KROSMAGA-DEV-PROXY-ASSET-BINDING-STAGE3-RECOVERY: SHIPPED
