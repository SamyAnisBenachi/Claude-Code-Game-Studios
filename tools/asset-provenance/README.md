# Asset Provenance Validators

> Tools implementing the release-gate and dev-proxy metadata rules defined
> in [ADR-025](../../docs/architecture/adr-025-asset-pack-provenance-architecture.md)
> and [`design/assets/provenance/schema.md`](../../design/assets/provenance/schema.md).
>
> The tools are invocable today as CLIs. Wiring the release validator into a
> CI release-packaging pipeline is deliberately out of scope for this story.
> When a CI release-packaging job is added, it must call the release validator
> and treat a non-zero exit as a hard failure.

## Release-Scan Validator

Given a release manifest, `check_release.py` fails if any packaged mapping
satisfies any of these rules:

1. `source_class == licensed_krosmaga_dev_proxy`
2. `source_class == unknown_provenance`
3. `release_class != release_allowed`
4. `workflow_status != approved`
5. `approval_evidence` is null while `workflow_status == approved`
6. The resolved concrete `path` begins with `dev-assets/` or any descendant

On success the validator emits nothing on stderr and exits zero. On failure
it emits a JSON list of `{logical_id, rule, value, path}` records on stderr,
then exits 1.

## Files

| File | Purpose |
|------|---------|
| `check_release.py` | Release-scan validator. Pure Python 3 stdlib. |
| `test_check_release.py` | Self-contained unit test for the release validator. |
| `validate_dev_proxy_pack.py` | Dev-only Krosmaga proxy pack metadata validator. Pure Python 3 stdlib. |
| `test_validate_dev_proxy_pack.py` | Self-contained unit test for the dev-proxy pack validator. |
| `fixtures/release-manifest-clean.json` | Example PASS release manifest. |
| `fixtures/release-manifest-krosmaga-leak.json` | Example FAIL release manifest containing a Krosmaga-proxy row. |
| `fixtures/release-manifest-dev-path.json` | Example FAIL release manifest where a `path` begins with `dev-assets/`. |
| `fixtures/release-manifest-unapproved.json` | Example FAIL release manifest with `workflow_status` not `approved`. |
| `fixtures/dev-proxy-pack-clean.json` | Example PASS dev-proxy pack manifest containing no payload assets. |
| `fixtures/dev-proxy-pack-release-claim.json` | Example FAIL manifest where a proxy row claims release-safe metadata. |
| `fixtures/dev-proxy-pack-repo-assets-source.json` | Example FAIL manifest where a proxy source path points into `assets/**`. |

## Release CLI Usage

```
python tools/asset-provenance/check_release.py <release-manifest.json>
```

Exit codes:

- `0`: every row passes all six release rules.
- `1`: at least one row fails; JSON failure report on stderr.
- `2`: manifest could not be parsed or required keys are missing.

Release manifest format:

```json
{
  "logical_assets": [
    {
      "logical_id": "lid_card_frame_common",
      "workflow_status": "approved",
      "source_class": "studio_original",
      "release_class": "release_allowed",
      "approval_evidence": "production/qa/sign-off-2026-05-20-card-frames.md",
      "path": "art/ui/card/ui_card_frame_common_hand.png"
    }
  ]
}
```

Required keys per release row: `logical_id`, `workflow_status`,
`source_class`, `release_class`, `path`. Optional: `approval_evidence`
(required when `workflow_status == approved`).

## Dev-Proxy Pack Validator

The dev-proxy pack validator checks future Krosmaga proxy pack metadata
before any binding or materialization work consumes it. It validates metadata
only. It does not copy Krosmaga payloads, scan extracted source banks, or
write into `assets/**`.

```
python tools/asset-provenance/validate_dev_proxy_pack.py <dev-proxy-pack-manifest.json>
```

Exit codes:

- `0`: every pack row satisfies the dev-only policy.
- `1`: at least one row violates the policy; JSON failure report on stderr.
- `2`: manifest could not be parsed or required keys are missing.

Required top-level shape:

```json
{
  "pack": {
    "pack_id": "krosmaga-proxy-v1",
    "dev_only": true,
    "source_class": "licensed_krosmaga_dev_proxy",
    "release_class": "dev_only"
  },
  "entries": []
}
```

Required keys per entry: `logical_id`, `source_path`, `match_quality`,
`dev_only`, `source_class`, `release_class`, `workflow_status`,
`license_provenance_warning`, and `expected_consumer_surface`.

Allowed `match_quality` values: `exact`, `good`, `needs_conversion`,
`ambiguous`, `missing`, `no_art_needed`.

The validator fails clearly when:

- A row implies release-safe art (`dev_only=false`,
  `source_class!=licensed_krosmaga_dev_proxy`, `release_class!=dev_only`, or
  `workflow_status!=needed`).
- A non-missing row has no `source_path`.
- `source_path` points into repo `assets/**`, which would imply copied
  Krosmaga content.
- An `ambiguous` row lacks `manual_review_required=true` or
  `ambiguity_notes`.
- A `missing` or `no_art_needed` row names a source path or lacks
  `missing_handling`.
- The license/provenance warning does not state that the row is dev-only or
  not release-approved.

## Non-Claims

Running these validators does not approve any asset, does not flip
`PAW-TD-*-a` rows, and does not constitute release sign-off. They are
necessary-but-not-sufficient gates: passing manifests still need the
studio's release-management and art/legal approval process.
