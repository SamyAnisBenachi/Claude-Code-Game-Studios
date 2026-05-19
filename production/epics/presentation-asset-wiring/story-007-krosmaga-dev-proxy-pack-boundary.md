# Story 007: S18-PAW-KROSMAGA-DEV-PROXY-PACK-BOUNDARY-001 -- Dev-Only Krosmaga Proxy Pack + Provenance Boundary

> **Epic**: Presentation Asset Wiring
> **Story ID**: `S18-PAW-KROSMAGA-DEV-PROXY-PACK-BOUNDARY-001`
> **Status**: Draft -- future Sprint 18 prerequisite candidate; NOT activated
> **Layer**: Presentation asset wiring / provenance governance
> **Type**: Docs + tooling + dev-only pack boundary
> **Sprint**: Future Sprint 18 candidate; should land before any story wires Krosmaga-derived proxy rows
> **Authored**: 2026-05-18 by PROMPT 1280
> **Authoring source-of-truth**: local `HEAD@051b59d`; workspace had unrelated dirty code/runtime files that were not touched
> **Source reports**: PROMPT 1257, 1258, 1265, 1266, 1267
> **Estimated effort**: ~0.5d

---

## Status / No-Claim Banner

This story is authoring-only. It does not activate Sprint 18, does not modify
`production/sprint-status.yaml`, does not create a release approval, and does
not copy or import Krosmaga assets.

The future implementation of this story must preserve these claims:

- Krosmaga proxy rows remain `source_class=licensed_krosmaga_dev_proxy`.
- Krosmaga proxy rows remain `workflow_status=needed`.
- Krosmaga proxy rows remain `release_class=dev_only`.
- No Krosmaga file is promoted into `assets/**`.
- No public/final package may include a Krosmaga proxy path.
- No final-art / asset-production accept-risk row (`PAW-TD-*-a`) is closed.
- No public release, RC readiness, full-game completion, playtest validation, or
  Standard-tier accessibility completion is claimed.

PROMPT 1280 performed story-authoring only. The implementation described below
is future work.

---

## Source Findings

PROMPT 1257 proposed the three-axis provenance taxonomy that this story turns
into a repo boundary:

- `workflow_status`: production readiness.
- `source_class`: file provenance.
- `release_class`: packaging permission.

PROMPT 1258 and PROMPT 1267 both require all Krosmaga proxy candidates to stay
dev-only and outside release packaging. PROMPT 1267 also recommends building a
dev-only proxy pack and loader aliases before wiring card, board, result, or
overlay chrome candidates.

PROMPT 1265 / 1266 establish that Krosmaga is a state-feedback and composition
reference for CCGS, not a release asset source. This story exists so later UI
implementation stories can use local development proxies without blurring that
boundary.

---

## Scope

### In Scope

- Author or ratify a provenance architecture document, preferably
  `docs/architecture/adr-022-asset-pack-provenance-architecture.md`, defining:
  `workflow_status`, `source_class`, `release_class`, logical asset IDs,
  dev-pack selection, pack materialization, and release-scan failure rules.
- Add a concise appendix to `design/assets/asset-manifest.md` defining the same
  fields without rewriting every asset row.
- Add a minimal provenance schema/index under `design/assets/provenance/`
  covering logical asset IDs used by current card, hand, board, shop/auction,
  result, and shared-overlay primitives.
- Add a dev-only pack manifest contract under a gitignored path such as
  `dev-assets/krosmaga-proxy/pack.toml`, or document the exact local path if the
  repo intentionally keeps the local pack untracked.
- Add validation tooling or tests that fail a release/package scan when any
  packaged asset resolves to:
  `source_class=licensed_krosmaga_dev_proxy`,
  `release_class!=release_allowed`,
  `workflow_status!=approved`, a missing approval record, or a path under
  `dev-assets/`.
- Amend `docs/architecture/adr-021-presentation-layer-architecture.md` only with
  a small related-decision link if ADR-022 is authored.
- Update `production/epics/presentation-asset-wiring/EPIC.md` to state the
  logical-ID/source-swap invariant.

### Out Of Scope

- Copying Krosmaga assets into the repo.
- Moving current runtime assets.
- Rewriting every `design/assets/asset-manifest.md` row.
- Replacing placeholders with Krosmaga art.
- Wiring any UI code to Krosmaga paths.
- Packaging, releasing, or approving Krosmaga proxy material.
- Any code under `server/` or `shared/`.

---

## Dependencies And Sequencing

| Dependency | Required posture |
|---|---|
| PROMPT 1257 provenance proposal | Accepted as the governing taxonomy input. |
| PROMPT 1258 placeholder mapping | Used only for dev-proxy classification and release warning. |
| PROMPT 1267 binding audit | Used only for logical asset IDs and implementation ordering. |
| ADR-021 | Presentation code still binds logical slots; asset source choice is outside gameplay behavior. |

This story should run before the Sprint 19 Krosmaga-style UI implementation wave
that consumes card frames, board cells, overlay chrome, result chrome, or
targeting-marker proxy candidates.

---

## Acceptance Criteria

- [ ] **AC1 -- ADR/provenance doc exists**: A repository document defines
  `workflow_status`, `source_class`, `release_class`, logical asset IDs, pack
  selection/materialization, and release-gate rules.
- [ ] **AC2 -- Asset-manifest appendix exists**: `design/assets/asset-manifest.md`
  has a concise appendix introducing the provenance fields without a noisy
  whole-file rewrite.
- [ ] **AC3 -- Logical asset schema/index exists**: `design/assets/provenance/`
  contains a schema and minimal logical ID index for current presentation
  surfaces.
- [ ] **AC4 -- Krosmaga proxy classification is explicit**: Any example or
  seed Krosmaga row uses exactly `source_class=licensed_krosmaga_dev_proxy`,
  `workflow_status=needed`, and `release_class=dev_only`.
- [ ] **AC5 -- Dev pack stays outside runtime assets**: The story implementation
  either creates no Krosmaga pack files or creates only untracked/gitignored
  `dev-assets/krosmaga-proxy/**` metadata. It does not write Krosmaga material
  under `assets/**`.
- [ ] **AC6 -- Release scan blocks dev proxies**: A test/tool fails if a release
  package resolves a logical asset to a Krosmaga proxy, non-approved workflow
  status, non-release-allowed release class, missing approval evidence, or a
  `dev-assets/**` path.
- [ ] **AC7 -- Presentation boundary preserved**: ADR-021 remains the Bevy UI
  / world-space rendering boundary; the new pack layer does not change gameplay
  logic or UI call-site ownership.
- [ ] **AC8 -- No source asset copy**: `git diff -- assets/` is empty except for
  any intentional non-Krosmaga metadata explicitly approved in the story branch.
- [ ] **AC9 -- No release claim**: Completion notes repeat that Krosmaga proxy
  assets are not release-approved and do not close `PAW-TD-*-a`.

---

## Worker Contract

1. Worktree slug: `work/s18-paw-krosmaga-dev-proxy-pack-boundary`.
2. Read PROMPT 1257, 1258, 1265, 1266, and 1267 before implementation.
3. Keep the change docs/tooling only; do not copy Krosmaga assets.
4. If a validation test touches Rust/Bevy code, activate `liv-bevy-018`.
5. Run only targeted validation/tooling checks needed for AC6.
6. Push the worker branch only; do not commit to `main`.

