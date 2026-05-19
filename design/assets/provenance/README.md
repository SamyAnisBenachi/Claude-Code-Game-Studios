# Asset Provenance — Index and Boundaries

> Introduced: 2026-05-19 by Story 007 (`S18-PAW-KROSMAGA-DEV-PROXY-PACK-BOUNDARY-001`,
> PROMPT 1369) for the Presentation Asset Wiring epic.
> Governing ADR: [ADR-025: Asset-Pack Provenance Architecture and Dev-Only
> Krosmaga Proxy Boundary](../../../docs/architecture/adr-025-asset-pack-provenance-architecture.md).

This folder is the structural boundary between studio-owned art, third-party
dev-only proxies (Krosmaga), and release-eligible packaged content. It does
**not** approve any Krosmaga material for release, does not close
`PAW-TD-*-a` final-art accept-risk rows, and does not change any
`production/sprint-status.yaml`, `production/sprints/**`, `production/stage.txt`,
`production/qa/**`, or `production/gate-checks/**` entry.

## Files

| File | Purpose |
|------|---------|
| [`schema.md`](./schema.md) | Canonical schema for the three-axis provenance taxonomy (`workflow_status`, `source_class`, `release_class`), logical asset IDs, pack manifest format, and release-scan rules. |
| [`logical-id-index.md`](./logical-id-index.md) | Minimal current set of logical asset IDs across card, hand, board, HUD, overlay, and result surfaces, with their default classification and which PAW story owns them. |
| [`dev-pack-example.toml`](./dev-pack-example.toml) | Example dev-only pack manifest demonstrating the pack TOML schema. Contains no Krosmaga payload. Real dev-packs live outside the repo at `dev-assets/krosmaga-proxy/pack.toml`. |

## How a Logical Asset Resolves

```
Logical ID (e.g., lid_card_frame_common)
         │
         ▼
Pack selection order:
  1. Release build → only release_allowed packs eligible
  2. Dev workstation + dev-assets/krosmaga-proxy/ present → may resolve
  3. Studio placeholder / generated placeholder
  4. Fallback: lid_ui_placeholder_1x1_white
         │
         ▼
Concrete file (assets/... or dev-assets/...)
         │
         ▼
Release-scan validator (tools/asset-provenance/check_release.py):
  fail if source_class ∈ {licensed_krosmaga_dev_proxy, unknown_provenance}
  fail if release_class ≠ release_allowed
  fail if workflow_status ≠ approved
  fail if path begins with dev-assets/
```

## Krosmaga Proxy Boundary (Hard Rules)

- No Krosmaga file may enter `assets/**`.
- No Krosmaga file may enter the repo (even outside `assets/**`).
- The dev-only pack lives at `dev-assets/krosmaga-proxy/` on a developer's
  workstation. The `dev-assets/` tree is gitignored.
- Every Krosmaga proxy row in the logical-ID index carries exactly:
  `source_class = licensed_krosmaga_dev_proxy`,
  `workflow_status = needed`,
  `release_class = dev_only`.
- The release-scan validator hard-fails any packaged build that resolves
  through a Krosmaga proxy.

## Non-Claims

This boundary work does **not** claim or imply:

- Release readiness, RC closure, certification, or store submission.
- Final-art / asset-clearance completion (`PAW-TD-*-a` rows remain open).
- Sprint 18 Must Have row expansion or Sprint 19 activation.
- Standard-tier accessibility coverage.
- Playtest validation for any Krosmaga-style chrome.
- That `production/sprint-status.yaml`, `production/sprints/**`,
  `production/stage.txt`, `production/qa/**`, or
  `production/gate-checks/**` have been updated.

Any presentation-asset-wiring story that adopts the logical-ID layer must
preserve every item in this list.
