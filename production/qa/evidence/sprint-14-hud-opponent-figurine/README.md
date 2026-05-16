# Sprint 14 HUD Opponent Figurine Evidence

Story: `production/epics/hud/story-017-hud-opponent-figurine.md`

Prompt: `PROMPT-968`

Worker branch: `work/s14-hud-opponent-figurine`

Implementation commit: recorded in the PROMPT-968 final worker report. This
README is part of that same worker commit.

## No-Claim Restatement

This evidence records HUD opponent-figurine layout composition and snapshot
asset-sync work only. It does not claim public release readiness,
release-candidate readiness, full game completion, full playable-client manual
QA, final-art completion, Sprint 14 close-out, stage advance, or a
Polish-to-Release gate retry.

Carried non-claims preserved:

- `S8-QA-001-W1` remains open.
- `QA-COND-0005` Standard-tier accessibility remains accepted-risk.
- `QA-COND-0006` playtest validation remains accepted-risk / deferred.
- `PAW-TD-004-a` placeholder figurine art remains accepted-risk.
- PROMPT 761 Polish-to-Release gate-check remains FAIL and is not retried.

No final-art replacement is introduced. No Standard-tier accessibility
hit-target claim is made for the passive figurine indicator. No objective
identity or `was_fake` data is surfaced through the figurine path.

## Cross-Links

| Source | Relevance |
|--------|-----------|
| `reports/PROMPT-802-Expert-UI-Layout-Audit-And-Repair-Roadmap.md` section 3.2 H10 | Original finding: opponent figurine has no separate HUD composition slot. |
| `docs/ux/ui-clean-pass-roadmap.md` Tier 1 Should-Priority Adjacent Rows | Sprint 14 sequencing note for the HUD opponent figurine row. |
| `docs/ux/global-ui-design-spec.md` sections 3 and 9 | Z-layer and strip-composition inputs consumed by the bottom-strip placement. |
| `docs/architecture/adr-021-presentation-layer-architecture.md` | HUD pre-pooled entity and PresentationSet schedule context. |
| `docs/architecture/adr-002-client-server-authority.md` | Opponent class identity is read from server-authoritative snapshots only. |
| `docs/architecture/adr-012-session-lifecycle.md` | Lobby class-lock authority context; no new HUD class-lock drain is added. |

## Automated Evidence

| AC | Evidence | Status |
|----|----------|--------|
| AC1, AC3 | `hud_opp_figurine_test::ac1_ac3_opponent_figurine_is_prepooled_and_exposed` verifies `HUD_ENTITY_COUNT == 23`, two `HudFigurine` entities, exactly one `OpponentFigurineMarker`, distinct own/opponent entities, and `HudEntities.opponent_figurine`. | PASS |
| AC2, AC12, AC15 | `hud_opp_figurine_test::ac2_ac12_ac15_opponent_figurine_composes_through_bottom_strip` verifies the opponent figurine is a child of `HudBottomStrip`, has no direct `GlobalZIndex`, has non-absolute positioning, and uses fixed 64 x 64 px dimensions. | PASS |
| AC4, AC5 | `hud_opp_figurine_test::ac4_ac5_opponent_figurine_updates_from_snapshot_class_id` sends authoritative snapshots and verifies own and opponent `ImageNode` handles resolve through `hud_figurine_asset(class_id)`. | PASS |
| AC6 | `hud_opp_figurine_test::ac5_ac6_game_over_snapshot_updates_but_incremental_paths_do_not_exist` verifies a GAME_OVER snapshot still rebuilds the opponent figurine and source-greps that HUD does not add a parallel `S2CClassLocked` incremental drain. | PASS |
| AC7, AC8 | `hud_opp_figurine_test::ac7_ac8_opponent_figurine_path_has_no_objective_or_client_inference_input` source-greps the sync path for `snapshot_hud_players`, `opponent.class_id`, no objective/unit/lane/board inference, and no `MessageReceiver<S2CClassLocked>`. | PASS |
| AC9 | `client/src/ui/hud/mod.rs` keeps `sync_figurine_image_system` in the existing `HudSystemSet::StateSync` schedule slot; no new HUD schedule set is introduced. | PASS |
| AC11 | No opponent figurine caption is rendered by this story. Text-fitting criteria are trivially satisfied for this passive image-only indicator. | PASS |
| AC14 | No opponent-figurine caption exists, and the implementation adds no HUD font-size viewport scaling. | PASS |
| AC16, AC17 | This worker scope excludes sprint trackers, session-state, sprint plans, QA plan, `stage.txt`, server, and shared files. Final diff verification is recorded in the PROMPT-968 report. | PASS |
| AC18 | Targeted HUD and workspace verification commands are recorded in the PROMPT-968 final worker report. | RECORDED IN REPORT |
| AC19 | This README reserves the evidence slot and records automated evidence plus visual-capture limitations. | PASS |

## Class-Swap Observation

Automated ECS snapshot coverage exercises these class states:

| Run | Own class | Opponent class | Expected opponent figurine |
|-----|-----------|----------------|----------------------------|
| Initial snapshot | `Iop` | `Cra` | `hud_figurine_asset(ClassId::Cra)` |
| Later snapshot | `Iop` | `Ecaflip` | `hud_figurine_asset(ClassId::Ecaflip)` |
| GAME_OVER rebuild | `Sacrier` | `Xelor` | `hud_figurine_asset(ClassId::Xelor)` |

The test compares Bevy `ImageNode.image` handles against `AssetServer::load`
for the same resolver path, proving the opponent figurine consumes the same
asset resolver as the own-player figurine.

## Dimension Table

| Element | 1920x1080 intended dimensions | 1366x768 intended dimensions | Status |
|---------|-------------------------------|------------------------------|--------|
| Own figurine | 64 x 64 px fixed Node intent | 64 x 64 px fixed Node intent | Covered by existing bottom-strip tests |
| Opponent figurine | 64 x 64 px fixed Node intent | 64 x 64 px fixed Node intent | PASS by `hud_opp_figurine_test` |

Rendered pixel screenshots are deferred; the ECS tests assert the fixed Node
intent that the renderer consumes.

## Overlap Audit

| Check | Evidence | Status |
|-------|----------|--------|
| Opponent figurine does not overlap own figurine in hierarchy | Own and opponent figurines are distinct direct children of `HudBottomStrip`; both are flex children with non-absolute `Node` positioning. | PASS by ECS Node intent |
| Opponent figurine does not overlap top-strip children | Opponent figurine is hosted under `HudBottomStrip`; mana, reserve mana, gold, phase, round, and timer remain top-strip children. | PASS by hierarchy |
| Opponent figurine z slot does not override strip order | Opponent figurine has no direct `GlobalZIndex` and inherits the bottom strip's `z_layers::UI_BASE` slot. | PASS |
| Browser-rendered overlap at DRAFT_SHOP and DRAFT_AUCTION | Requires runtime WASM/browser screenshots in a later visual pass. | Manual capture pending |

## Visual Capture Status

No PNG screenshots are claimed by this worker. The worker environment verified
the layout through Bevy ECS tests but did not run two browser clients for
runtime capture.

Expected filenames for a later manual or browser-capture pass:

- `opp-figurine-1920x1080-draft-shop.png`
- `opp-figurine-1366x768-draft-shop.png`
- `opp-figurine-1920x1080-draft-auction.png`
- `opp-figurine-1366x768-draft-auction.png`

The captures must not be treated as present until those PNG files are committed
or attached by a later capture worker.
