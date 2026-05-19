# Asset Provenance — Release-Scan Validator

> Tool implementing the release-gate rules defined in
> [ADR-025](../../docs/architecture/adr-025-asset-pack-provenance-architecture.md)
> and [`design/assets/provenance/schema.md`](../../design/assets/provenance/schema.md).
>
> The tool is invocable today as a CLI. Wiring it into a CI release-packaging
> pipeline is **deliberately out of scope** for this story — see ADR-025
> "Consequences → Negative". When a CI release-packaging job is added, it
> must call this validator and treat a non-zero exit as a hard failure.

## What the validator checks

Given a release manifest (a YAML file describing the set of logical-asset
mappings the release will package), the validator fails (non-zero exit) if
**any** mapping satisfies any of these rules:

1. `source_class == licensed_krosmaga_dev_proxy`
2. `source_class == unknown_provenance`
3. `release_class != release_allowed`
4. `workflow_status != approved`
5. `approval_evidence` is null while `workflow_status == approved`
6. The resolved concrete `path` begins with `dev-assets/` or any descendant

On success the validator emits nothing on stderr and exits zero. On failure
the validator emits a JSON list of `{logical_id, rule, value, path}`
records on stderr, then exits 1.

## Files

| File | Purpose |
|------|---------|
| `check_release.py` | The validator. Pure Python 3 stdlib (no third-party deps). |
| `test_check_release.py` | Self-contained unit test. Run with `python -m unittest`. |
| `fixtures/release-manifest-clean.json` | Example PASS manifest (all rows release-eligible). |
| `fixtures/release-manifest-krosmaga-leak.json` | Example FAIL manifest containing a Krosmaga-proxy row. |
| `fixtures/release-manifest-dev-path.json` | Example FAIL manifest where a `path` begins with `dev-assets/`. |
| `fixtures/release-manifest-unapproved.json` | Example FAIL manifest with `workflow_status` not `approved`. |

## CLI usage

```
python tools/asset-provenance/check_release.py <release-manifest.json>
```

The release manifest is JSON so the validator has no third-party
dependencies. The human-authored `design/assets/provenance/logical-id-index.md`
encoding is YAML for readability; a separate step (out of scope for this
story) translates it to the JSON manifest format the validator consumes.

Exit codes:

- `0` — every row passes all six rules.
- `1` — at least one row fails; JSON failure report on stderr.
- `2` — manifest could not be parsed or required keys are missing.

## Release manifest format

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

Required keys per row: `logical_id`, `workflow_status`, `source_class`,
`release_class`, `path`. Optional: `approval_evidence` (required when
`workflow_status == approved`).

## Non-claims

Running this validator does **not** approve any asset, does not flip
`PAW-TD-*-a` rows, and does not constitute release sign-off. It is a
necessary-but-not-sufficient gate: a manifest that passes the validator
still needs the studio's release-management process (currently undefined).
