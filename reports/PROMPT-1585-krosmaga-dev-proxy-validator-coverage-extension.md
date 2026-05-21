# PROMPT 1585 -- KROSMAGA-DEV-PROXY-VALIDATOR-COVERAGE-EXTENSION

- Worktree: `D:/Tmp/wt-1585`
- Branch: `work/krosmaga-dev-proxy-validator-coverage-1585`
- Base: `origin/main@9be8827fbd22b2a49d973ba585b5d210fdc8a903`
- Status: SHIPPED (local commit; push attempted — see Push section)

## Scope chosen

Stage 1 (PROMPT 1483) produced the human-readable Krosmaga proxy logical-id
map. Stage 2 (PROMPT 1534) materialized the JSON dev-proxy pack validator
plus an active-UI-lane candidate manifest fixture. This Stage 3 readiness
work extends the validator with additive, optional metadata for the next
step toward atlas binding, without copying or materializing any Krosmaga
payload and without changing any release-class semantics.

Four optional fields were added. All are no-ops when omitted — every
pre-existing fixture and test continues to pass unchanged:

1. `atlas_binding` (per-entry object) — sprite-sheet frame coordinates.
2. `pack.sprite_sheets` (pack-level list) — atlas registry, cross-referenced
   from `atlas_binding.atlas_id`.
3. `pack.license_provenance` (pack-level object) — structured rights
   statement complementing the per-row `license_provenance_warning`.
4. `stage_readiness` (per-entry string) — opt-in marker gating Stage 3
   binding claims.

No runtime rendering code, no asset copy, no release-class metadata change,
no sprint/session/QA paperwork edits, no Cargo invocation.

## Validator coverage added

`tools/asset-provenance/validate_dev_proxy_pack.py` (new rule names):

### `atlas_binding` (per-entry, optional)
- `atlas_binding_shape` — must be an object when present.
- `atlas_binding_atlas_id_required` — non-empty string.
- `atlas_binding_atlas_id_format` — lowercase letters, digits, underscores only.
- `atlas_binding_atlas_id_unknown` — when `pack.sprite_sheets` is provided,
  the atlas_id must reference a registered sheet.
- `atlas_binding_frame_index_non_negative` — integer >= 0 (rejects booleans).
- `atlas_binding_frame_size_px_shape` — `[width, height]` pair of positive integers.
- `atlas_binding_frame_origin_px_shape` — optional; when present, `[x, y]`
  pair of non-negative integers.
- `atlas_binding_forbidden_for_missing_or_no_art` — `missing` / `no_art_needed`
  rows must not declare a binding (no concrete frame to bind).
- `atlas_binding_forbidden_for_ambiguous` — `ambiguous` rows must clear
  manual review before declaring a binding.

### `pack.sprite_sheets` (pack-level, optional)
- `pack_sprite_sheets_shape` — must be a list when present.
- `pack_sprite_sheet_entry_shape` — each entry must be an object.
- `pack_sprite_sheet_sheet_id_required` — non-empty string.
- `pack_sprite_sheet_sheet_id_format` — lowercase token charset.
- `pack_sprite_sheet_sheet_id_duplicate` — sheet_ids unique within the pack.
- `pack_sprite_sheet_dimensions_px_shape` — `[w, h]` pair of positive integers.
- `pack_sprite_sheet_frame_count_positive` — positive integer.

### `pack.license_provenance` (pack-level, optional)
- `pack_license_provenance_shape` — must be an object when present.
- `pack_license_provenance_holder_required` — non-empty string.
- `pack_license_provenance_kind_value` — one of
  `{licensed_krosmaga_dev_proxy, licensed_external_release}`.
- `pack_license_provenance_kind_must_match_source_class` — kind must match
  `pack.source_class` when both are present.
- `pack_license_provenance_dev_only_statement_required` — non-empty string.
- `pack_license_provenance_dev_only_statement_must_block_release_claim` —
  must contain dev-only / not-release-approved language (same lexical check
  as the existing per-row license warning).

### `stage_readiness` (per-entry, optional)
- `stage_readiness_value` — must be one of
  `{stage1_logical, stage2_candidate, stage3_binding}` when present.
- `stage3_binding_requires_atlas_binding` — a `stage3_binding` claim must
  carry an `atlas_binding` block.
- `stage3_binding_requires_concrete_match` — a `stage3_binding` claim
  requires `match_quality ∈ {exact, good, needs_conversion}`.

None of the new rules relax existing checks; they are strictly additive.
Every prior fixture continues to validate.

## Fixtures added

- `tools/asset-provenance/fixtures/dev-proxy-pack-stage3-candidate.json` —
  PASS example. Demonstrates the full Stage 3 shape: atlas_binding on two
  hand-card and two HUD figurine rows, a `pack.sprite_sheets` registry with
  two sheets, a `pack.license_provenance` block, and `stage_readiness`
  values covering all three valid markers (including a `stage1_logical`
  missing row and a `stage2_candidate` ambiguous row).
- `tools/asset-provenance/fixtures/dev-proxy-pack-atlas-binding-bad.json` —
  deliberate FAIL example exercising both
  `stage3_binding_requires_atlas_binding` and
  `atlas_binding_atlas_id_unknown` in a single manifest.

No Krosmaga payload is committed in either fixture; they are JSON manifests
with absolute Windows source paths pointing at the user's local Ankama
extraction tree, mirroring the Stage 2 candidate fixture convention.

## Tests / validation

- `python -m unittest tools/asset-provenance/test_validate_dev_proxy_pack.py`
  — 62 tests pass (was 25 in Stage 2; added 37 new tests covering each new
  rule plus the two new CLI fixtures).
- `python -m pytest tools/asset-provenance/test_validate_dev_proxy_pack.py -q`
  — 62 passed.
- `python -m unittest tools/asset-provenance/test_check_release.py` — 27
  tests pass (release validator unaffected; regression check).
- Path allowlist review (manual): all edits stay under
  `tools/asset-provenance/**` (validator + tests + fixtures + README) — no
  `assets/**`, no `dev-assets/**`, no client/server/shared runtime code, no
  `production/**` paperwork, no Cargo invocation. No `assets/dev-proxy/`
  manifest edits were required because every new field is optional and
  every pre-existing fixture still validates without modification.
- `git diff --check` clean.

## Files touched

```
M tools/asset-provenance/README.md
M tools/asset-provenance/test_validate_dev_proxy_pack.py
M tools/asset-provenance/validate_dev_proxy_pack.py
A tools/asset-provenance/fixtures/dev-proxy-pack-atlas-binding-bad.json
A tools/asset-provenance/fixtures/dev-proxy-pack-stage3-candidate.json
A reports/PROMPT-1585-krosmaga-dev-proxy-validator-coverage-extension.md
```

Lines: +1027 / −2 across the five tool files (pre-report). The
`validate_dev_proxy_pack.py` change is split between four new helper
functions (`_check_atlas_binding`, `_check_stage_readiness`,
`_check_pack_sprite_sheets`, `_check_pack_license_provenance`) and wiring
them into the existing `_check_entry` / `validate_manifest` flow.

## Non-claims

- No Krosmaga source bytes were copied, generated, materialized, or
  imported into the repo.
- No row was promoted to `approved`, `release_allowed`, or `studio_original`.
- No client UI lane was rewired; the Stage 3 candidate manifest is data,
  not a runtime binding.
- This work is necessary-but-not-sufficient: passing the validator does not
  approve any asset for release. The release-scan validator still hard-fails
  any packaged build resolving a logical asset through a Krosmaga proxy.

## Push

<filled in after the push attempt>

1585: KROSMAGA-DEV-PROXY-VALIDATOR-COVERAGE-EXTENSION: SHIPPED
