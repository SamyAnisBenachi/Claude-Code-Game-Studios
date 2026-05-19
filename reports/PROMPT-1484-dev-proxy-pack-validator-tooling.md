# PROMPT 1484 - Dev Proxy Pack Validator Tooling

Status: COMPLETE

## Scope

Implemented a tooling-only validator for future dev-only Krosmaga proxy pack
metadata. No Krosmaga payload assets were copied, imported, inspected, or
materialized. No files under `client/**`, `server/**`, `shared/**`,
`assets/**`, sprint status, or session state were edited.

## Context Read

- `reports/PROMPT-1257-asset-pack-provenance-architecture-proposal.md`
- `reports/PROMPT-1258-krosmaga-placeholder-asset-mapping-report.md`
- `reports/PROMPT-1267-krosmaga-asset-bank-to-ccgs-ui-binding-audit.md`
- `reports/PROMPT-1479-krosmaga-asset-binding-rollout-plan.md`
- `docs/architecture/adr-025-asset-pack-provenance-architecture.md`
- `design/assets/provenance/schema.md`
- Existing `tools/asset-provenance/check_release.py` and tests

## Files

- `tools/asset-provenance/validate_dev_proxy_pack.py`
- `tools/asset-provenance/test_validate_dev_proxy_pack.py`
- `tools/asset-provenance/fixtures/dev-proxy-pack-clean.json`
- `tools/asset-provenance/fixtures/dev-proxy-pack-release-claim.json`
- `tools/asset-provenance/fixtures/dev-proxy-pack-repo-assets-source.json`
- `tools/asset-provenance/README.md`
- `reports/PROMPT-1484-dev-proxy-pack-validator-tooling.md`

## Validator Behavior

The new validator is a pure Python 3 stdlib CLI:

```text
python tools/asset-provenance/validate_dev_proxy_pack.py <dev-proxy-pack-manifest.json>
```

It validates a lightweight JSON manifest with top-level `pack` metadata and
an `entries` list. Each row must include `logical_id`, `source_path`,
`match_quality`, `dev_only`, `source_class`, `release_class`,
`workflow_status`, `license_provenance_warning`, and
`expected_consumer_surface`.

The validator fails clearly when a row would imply release-safe art, including
`dev_only=false`, `source_class!=licensed_krosmaga_dev_proxy`,
`release_class!=dev_only`, or `workflow_status!=needed`.

It also fails when a non-missing row lacks a `source_path`, or when
`source_path` points inside repo `assets/**`, which would indicate copied
Krosmaga payload content in the runtime asset tree.

`match_quality` is constrained to `exact`, `good`, `needs_conversion`,
`ambiguous`, `missing`, or `no_art_needed`. Ambiguous rows must set
`manual_review_required=true` and include `ambiguity_notes`. Missing or
no-art-needed rows must omit `source_path` and document `missing_handling`.

The license/provenance warning must explicitly state that the row is dev-only
or not release-approved.

## Fixtures

The clean fixture demonstrates a valid metadata-only pack with:

- A good match row.
- A needs-conversion row.
- A missing row with explicit handling.

Failure fixtures cover:

- A row that claims release-safe metadata.
- A row whose `source_path` points into repo `assets/**`.

These fixtures are synthetic metadata only and contain no Krosmaga payload.

## Validation Performed

Ran the lightweight Python unittest suite only:

```text
python -m unittest tools/asset-provenance/test_validate_dev_proxy_pack.py
python -m unittest tools/asset-provenance/test_check_release.py tools/asset-provenance/test_validate_dev_proxy_pack.py
```

Result: both commands passed. The combined run executed 41 tests.

No Cargo or broad test suites were run.

Confirmed `git diff -- assets` is empty.

## Handoff To Future Asset Binding

Future asset-binding or materialization work can use this validator as a
preflight check for local metadata before resolving any Krosmaga proxy pack.
The validator deliberately does not require PROMPT 1483 to land first and does
not prescribe a runtime resolver. It establishes the minimum metadata contract
needed to keep dev-only Krosmaga proxies separate from release-safe art claims
and from canonical runtime `assets/**` payloads.

1484: DEV-PROXY-PACK-VALIDATOR-TOOLING: COMPLETE
