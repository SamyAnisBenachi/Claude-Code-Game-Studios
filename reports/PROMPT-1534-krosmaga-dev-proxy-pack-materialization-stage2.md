# PROMPT 1534 -- KROSMAGA-DEV-PROXY-PACK-MATERIALIZATION-STAGE2

- Worktree: `D:/Tmp/wt-1534`
- Branch: `worker/prompt-1534-krosmaga-dev-proxy-stage2`
- Base: `origin/main@5358aed1a6075aca621936fd14f561be8fb854d3` (after orchestrator SHA-typo correction)
- Status: SHIPPED (local commit; push pending orchestrator disposition)

## Scope chosen

Stage 1 produced the human-readable Krosmaga proxy logical-id map
(`design/assets/provenance/krosmaga-proxy-logical-id-map-stage1.md`) and
PROMPT-1484/1504 produced the JSON dev-proxy pack validator
(`tools/asset-provenance/validate_dev_proxy_pack.py`). Stage 2 here advances
the next safe tooling/data step the prompt called for:

- improve validator coverage for missing/ambiguous proxy bindings;
- add manifest entries representing active UI lanes (hand frame, HUD class
  figurines, board cell, objective dot, Sang Méprise reveal marker) without
  copying or materializing any Krosmaga payload.

No runtime rendering code, no asset copy, no release-class metadata change,
no sprint/session/QA paperwork edits.

## Validator coverage added

`tools/asset-provenance/validate_dev_proxy_pack.py`:

1. `logical_id_prefix_required` — entry `logical_id` must start with `lid_`
   (per schema.md "Logical Asset ID Layer").
2. `logical_id_body_charset` — body after `lid_` must use only `[a-z0-9_]`.
3. `consumer_surface_format` — `expected_consumer_surface` must be a dotted
   lowercase token path (e.g. `hand.card_frame`, `hud.class_figurine_iop`);
   rejects mixed-case, missing dot, leading/trailing dot.
4. `conversion_notes_required` — `needs_conversion` rows must document
   `conversion_notes` (resize/atlas/audio-remux description).
5. `pack_workflow_status_must_remain_needed` — `pack.workflow_status`, when
   present, must equal `needed`. A Krosmaga pack cannot self-advance.
6. `pack_id_required` — `pack.pack_id` must be a non-empty string.

None of the new rules relax existing checks; they are additive.

## Active-UI-lane manifest entries

Added `tools/asset-provenance/fixtures/dev-proxy-pack-stage2-candidate.json`.
It is a documentation/test fixture that exercises the full validator on the
high-value rows from Stage 1, covering:

| logical_id | UI lane | match_quality |
|---|---|---|
| `lid_card_frame_common` | hand card frame | needs_conversion |
| `lid_hud_class_figurine_iop` | HUD class figurine | good |
| `lid_hud_class_figurine_cra` | HUD class figurine | good |
| `lid_hud_class_figurine_sacrier` | HUD class figurine | good |
| `lid_hud_class_figurine_xelor` | HUD class figurine | good |
| `lid_hud_class_figurine_ecaflip` | HUD class figurine | good |
| `lid_hud_class_figurine_sadida` | HUD class figurine | good |
| `lid_board_cell_idle_32x32` | board cell | missing (CCGS-original path) |
| `lid_hud_objective_dot_real_revealed` | objective dot reveal | ambiguous |
| `lid_overlay_targeting_marker_real` | Sang Méprise reveal marker | ambiguous |

All rows carry the mandatory triplet
`source_class=licensed_krosmaga_dev_proxy / release_class=dev_only /
workflow_status=needed`, the dev-only license warning, the conformant
`lid_*` logical-id, and a dotted `expected_consumer_surface`. The
`needs_conversion` row carries `conversion_notes`; the two `ambiguous` rows
carry `manual_review_required=true` plus `ambiguity_notes`; the `missing`
row documents `missing_handling` and explicitly disclaims any Krosmaga proxy
reuse.

The fixture lives under `tools/asset-provenance/fixtures/`, not under
`dev-assets/`, not under `assets/`, and contains no payload — it is the
JSON manifest the validator consumes, parallel to the existing
`dev-proxy-pack-clean.json` fixture, expanded to cover real CCGS UI lanes.

Also added `tools/asset-provenance/fixtures/dev-proxy-pack-bad-logical-id.json`
as a deliberate FAIL fixture exercising the new logical-id prefix rule.

## Tests / validation

- `python -m unittest tools/asset-provenance/test_validate_dev_proxy_pack.py`
  — 25 tests pass (was 15; added 10 new tests covering the new rules,
  including a CLI-level run against the Stage-2 candidate fixture).
- `python -m unittest tools/asset-provenance/test_check_release.py` — 27
  tests pass (unaffected, regression check).
- Direct CLI run:
  `python tools/asset-provenance/validate_dev_proxy_pack.py
   tools/asset-provenance/fixtures/dev-proxy-pack-stage2-candidate.json`
  exits 0.
- Path allowlist review (manual): all edits stay under
  `tools/asset-provenance/**` (validator + tests + fixtures + README) — no
  `assets/**`, no `dev-assets/**`, no client/server/shared runtime code, no
  `production/**` paperwork, no Cargo invocation. The Stage-1 map at
  `design/assets/provenance/krosmaga-proxy-logical-id-map-stage1.md` is
  unchanged.
- `git diff --check` clean.

## Files touched

```
M tools/asset-provenance/README.md
M tools/asset-provenance/fixtures/dev-proxy-pack-clean.json
M tools/asset-provenance/test_validate_dev_proxy_pack.py
M tools/asset-provenance/validate_dev_proxy_pack.py
A tools/asset-provenance/fixtures/dev-proxy-pack-bad-logical-id.json
A tools/asset-provenance/fixtures/dev-proxy-pack-stage2-candidate.json
A reports/PROMPT-1534-krosmaga-dev-proxy-pack-materialization-stage2.md
```

The `dev-proxy-pack-clean.json` edit adds the now-required
`conversion_notes` field to the existing `needs_conversion` row so the
older fixture still passes; it does not introduce a Krosmaga payload or
change the row's logical_id, source_path, or classification.

## Non-claims

- No Krosmaga source bytes were copied, generated, materialized, or
  imported into the repo.
- No row was promoted to `approved`, `release_allowed`, or `studio_original`.
- No client UI lane was rewired; the Stage-2 candidate manifest is data,
  not a runtime binding.
- This work is necessary-but-not-sufficient: passing the validator does not
  approve any asset for release.

1534: KROSMAGA-DEV-PROXY-PACK-MATERIALIZATION-STAGE2: SHIPPED
