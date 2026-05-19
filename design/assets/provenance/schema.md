# Provenance Schema

> Canonical schema for the three-axis asset provenance taxonomy defined in
> [ADR-025](../../../docs/architecture/adr-025-asset-pack-provenance-architecture.md).
> The taxonomy is enforced by the release-scan validator under
> `tools/asset-provenance/`.

## Three-Axis Taxonomy

Each logical asset (`lid_*`) and each concrete file resolved through a
logical asset carries three independent fields. None of the three implies
either of the others.

### `workflow_status`

Production-pipeline state. Mirrors the existing `Status` column in
`design/assets/asset-manifest.md` with explicit value names.

| Value | Meaning | Release-eligible? |
|-------|---------|-------------------|
| `needed` | No usable delivery file is tracked yet, or only an ownership placeholder exists. Covers manifest categories `Needed`, `Placeholder`, `File Present Placeholder`, `Generated Placeholder`. | No |
| `in_progress` | Production work is underway. A delivery file may exist but no final sign-off has been recorded. | No |
| `done` | Final delivery is complete with supporting evidence — file in the right location, correct technical dimensions, art lead has reviewed the file. | No — `approved` is the release gate. |
| `approved` | Production approval/sign-off is complete and recorded. | Yes, **if and only if** `source_class` and `release_class` also permit. |
| `blocked` | Tracked but waiting on unresolved design/UX/dependency. | No |

### `source_class`

Where the bits originated.

| Value | Meaning | Release-eligible? |
|-------|---------|-------------------|
| `studio_original` | Produced inside the studio (in-house art, audio, generated placeholder, code asset). Default for any new manifest row. | Yes, when `workflow_status=approved` and `release_class=release_allowed`. |
| `licensed_external_release` | Sourced from an external party under a license that allows redistribution in the studio's released build (signed font, licensed SFX pack, etc.). Reserved — no row currently uses this. | Yes, under the same rule as `studio_original`. |
| `licensed_krosmaga_dev_proxy` | Derived from Krosmaga (Ankama) game material. Used in-studio only as a visual reference for state-feedback and composition prototyping during early Sprint 18/19 Krosmaga-style work. Owned by Ankama. | **Never.** Release-scan validator hard-fails on this value. |
| `unknown_provenance` | Sentinel for any logical ID whose source has not been classified. | **Never.** Same hard-fail as `licensed_krosmaga_dev_proxy`. |

### `release_class`

Whether the row may appear in a packaged build at all, independent of where
it currently is in production.

| Value | Meaning | Release-eligible? |
|-------|---------|-------------------|
| `release_allowed` | May be included in a packaged build once `workflow_status=approved` and `source_class` permits it. | Yes (subject to the other axes). |
| `dev_only` | Used during development only. Includes Krosmaga proxies plus internal debug assets (debug fonts, debug grid overlays, dev console PNG). | **Never.** Release-scan validator hard-fails. |
| `internal_only` | Used inside the studio (production-handoff packets, art-bible exports, etc.) but never shipped in a player-facing build. | **Never.** Same hard-fail as `dev_only`. |

### Release-Eligibility Matrix

A logical-asset → concrete-file mapping is release-eligible **if and only if**:

```
workflow_status == approved
AND source_class ∈ {studio_original, licensed_external_release}
AND release_class == release_allowed
AND approval_evidence is present (or null is acceptable when status=approved
    came from a row already approved before this schema was introduced)
AND resolved path does not begin with dev-assets/
```

Any other combination fails the release-scan validator.

## Logical Asset ID Layer

A **logical asset ID** is a stable, source-agnostic identifier for what a UI
surface needs. Format:

```
lid_<surface>_<element>[_<variant>][_<dimensions>]
```

Examples:

- `lid_card_frame_common` — card frame chrome (common rarity).
- `lid_card_frame_legendary` — card frame chrome (legendary rarity).
- `lid_card_stat_badge_atk` — ATK stat badge for the hand card composition.
- `lid_hud_class_figurine_iop` — HUD class figurine for the Iop class.
- `lid_hud_objective_dot_real_revealed` — HUD objective dot, real-revealed
  state.
- `lid_board_cell_idle_32x32` — board cell node, idle state.
- `lid_overlay_targeting_marker_real` — Sang Méprise reveal marker, real
  variant.
- `lid_result_panel_chrome_win` — result panel chrome, WIN variant.
- `lid_ui_placeholder_1x1_white` — universal fallback per ADR-021.

Logical IDs are **declarative metadata**, not new runtime code. The
`design/assets/provenance/logical-id-index.md` file enumerates the current
set with their default classification and which PAW story owns them.
`asset_wiring.rs` (Story 001 onwards) may adopt the logical-ID layer when
convenient; until adopted, surfaces continue to use the per-story path
constants defined in stories 001–006.

### YAML Encoding (Recommended)

When a tool needs to reason about a logical-asset row programmatically, the
canonical encoding is:

```yaml
- logical_id: lid_card_frame_common
  description: "Hand UI card frame, common rarity"
  owner_story: production/epics/presentation-asset-wiring/story-002-hand-ui-card-frames.md
  workflow_status: needed
  source_class: studio_original
  release_class: release_allowed
  approval_evidence: null
  studio_paths:
    - art/ui/card/ui_card_frame_common_hand.png
  dev_pack_entries:
    krosmaga-proxy-v1: frames/card_frame_common.png  # optional, dev-only
```

Required keys: `logical_id`, `workflow_status`, `source_class`,
`release_class`. Optional keys: `description`, `owner_story`,
`approval_evidence`, `studio_paths`, `dev_pack_entries`.

`dev_pack_entries` is a map from pack ID to relative path inside that
pack. A logical ID with a `dev_pack_entries` value is **not** thereby
release-eligible — that mapping is consulted only on developer
workstations.

## Pack Manifest Format

A pack manifest declares the pack's identity, source class, release class,
and the logical-ID → relative-path map for the pack. Pack manifests live
inside their pack directory:

```
dev-assets/krosmaga-proxy/pack.toml          # dev-only Krosmaga pack
                                             # (gitignored payload)
design/assets/provenance/dev-pack-example.toml  # safe-to-commit example
```

The TOML schema:

```toml
[pack]
pack_id = "krosmaga-proxy-v1"
description = "Dev-only Krosmaga visual reference pack"
source_class = "licensed_krosmaga_dev_proxy"
release_class = "dev_only"
workflow_status = "needed"   # never overridden upward by a Krosmaga pack

[entries]
lid_card_frame_common      = "frames/card_frame_common.png"
lid_card_frame_rare        = "frames/card_frame_rare.png"
lid_board_cell_idle_32x32  = "board/cell_idle_32x32.png"
# … etc
```

Required pack fields: `pack_id`, `source_class`, `release_class`.
Optional pack fields: `description`, `workflow_status`.

## Release-Scan Failure Modes

The validator at `tools/asset-provenance/check_release.py` walks the set of
packaged assets (as declared by a release manifest input file) and the
provenance metadata. The scan fails (non-zero exit) when any resolution
satisfies any of:

1. `source_class == licensed_krosmaga_dev_proxy`.
2. `source_class == unknown_provenance`.
3. `release_class != release_allowed`.
4. `workflow_status != approved`.
5. `approval_evidence` is null **and** `workflow_status == approved` (a row
   cannot self-report `approved` without an evidence reference).
6. The resolved concrete path begins with `dev-assets/` or any descendant.

The validator emits a structured JSON error report on stderr listing each
failing row, the failing rule, and the offending value. On success it emits
nothing on stderr and exits zero.

## Cross-References

- [ADR-025](../../../docs/architecture/adr-025-asset-pack-provenance-architecture.md)
  — full architectural decision.
- [`logical-id-index.md`](./logical-id-index.md) — current logical-asset
  ID set.
- [`dev-pack-example.toml`](./dev-pack-example.toml) — example pack
  manifest.
- [`../asset-manifest.md`](../asset-manifest.md) Appendix A — quick
  reference to the three-axis taxonomy for readers of the per-row catalog.
- `tools/asset-provenance/check_release.py` — release-scan validator
  implementing the rules above.
