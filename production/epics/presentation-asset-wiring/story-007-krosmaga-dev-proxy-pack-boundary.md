# Story 007: S18-PAW-KROSMAGA-DEV-PROXY-PACK-BOUNDARY-001 -- Dev-Only Krosmaga Proxy Pack + Provenance Boundary

> **Epic**: Presentation Asset Wiring
> **Story ID**: `S18-PAW-KROSMAGA-DEV-PROXY-PACK-BOUNDARY-001`
> **Status**: Implementation complete on `work/s18-paw-krosmaga-dev-proxy-boundary-1369` (PROMPT 1369, 2026-05-19). NOT a sprint-row activation, NOT a story-done closure, NOT a release approval — see Status / No-Claim Banner below.
> **Layer**: Presentation asset wiring / provenance governance
> **Type**: Docs + tooling + dev-only pack boundary
> **Sprint**: Future Sprint 18 candidate; lands before any story wires Krosmaga-derived proxy rows
> **Authored**: 2026-05-18 by PROMPT 1280
> **Implemented**: 2026-05-19 by PROMPT 1369 from `origin/main@daa7759`
> **Authoring source-of-truth**: local `HEAD@051b59d`; workspace had unrelated dirty code/runtime files that were not touched
> **Implementation source-of-truth**: `origin/main@daa7759` fetched and branched cleanly
> **Source reports**: PROMPT 1257, 1258, 1265, 1266, 1267
> **Estimated effort**: ~0.5d

---

## Status / No-Claim Banner

PROMPT 1280 performed story-authoring. PROMPT 1369 performed the implementation
on `work/s18-paw-krosmaga-dev-proxy-boundary-1369`. Neither prompt activated
Sprint 18, modified `production/sprint-status.yaml`, modified
`production/sprints/**`, modified `production/stage.txt`, modified
`production/qa/**`, modified `production/gate-checks/**`, ran `/story-done`,
ran smoke/gate-check, created a release approval, or copied a Krosmaga file
into `assets/**` or anywhere else in the repo.

PROMPT 1369's implementation preserves every claim below; any future story
that consumes the boundary must continue to preserve them:

- Krosmaga proxy rows remain `source_class=licensed_krosmaga_dev_proxy`.
- Krosmaga proxy rows remain `workflow_status=needed`.
- Krosmaga proxy rows remain `release_class=dev_only`.
- No Krosmaga file is promoted into `assets/**`.
- No public/final package may include a Krosmaga proxy path.
- No final-art / asset-production accept-risk row (`PAW-TD-*-a`) is closed.
- No public release, RC readiness, full-game completion, playtest validation, or
  Standard-tier accessibility completion is claimed.
- No Sprint 18 Must Have row is expanded; Sprint 19 is not activated.
- No Polish → Release retry is claimed.

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

- [x] **AC1 -- ADR/provenance doc exists**: `docs/architecture/adr-025-asset-pack-provenance-architecture.md`
  defines `workflow_status`, `source_class`, `release_class`, logical asset IDs,
  pack selection/materialization, and release-gate rules. (PROMPT 1369 — the
  original story text suggested ADR-022 but that slot is already
  `adr-022-keyword-observer-architecture.md` and ADR-023/024 are also taken;
  the next free slot ADR-025 is used.)
- [x] **AC2 -- Asset-manifest appendix exists**: `design/assets/asset-manifest.md`
  carries `Appendix A — Provenance Fields (added 2026-05-19, PROMPT 1369)`
  introducing the three-axis taxonomy and the defaults for the existing 296
  rows without rewriting any individual row.
- [x] **AC3 -- Logical asset schema/index exists**: `design/assets/provenance/`
  contains `README.md`, `schema.md`, `logical-id-index.md`, and
  `dev-pack-example.toml`. The logical-ID index covers card, hand stat/cost,
  HUD class figurine, HUD objective dot, board cell, board unit base, overlay
  targeting marker, result panel chrome, and shared placeholder surfaces.
- [x] **AC4 -- Krosmaga proxy classification is explicit**: ADR-025 §1,
  `design/assets/provenance/schema.md` § Three-Axis Taxonomy,
  `design/assets/provenance/dev-pack-example.toml` `[pack]`, and the manifest
  appendix all state Krosmaga proxy rows carry exactly
  `source_class=licensed_krosmaga_dev_proxy`, `workflow_status=needed`,
  `release_class=dev_only`. No other combination is permitted.
- [x] **AC5 -- Dev pack stays outside runtime assets**: `.gitignore` excludes
  `dev-assets/`. No Krosmaga file is committed to the repo. The only
  Krosmaga-mentioning artefacts are documentation (ADR-025, schema,
  logical-ID index, `dev-pack-example.toml`) and the validator fixtures
  (which contain only string identifiers, no payload).
- [x] **AC6 -- Release scan blocks dev proxies**: `tools/asset-provenance/check_release.py`
  implements the six release-gate rules. `tools/asset-provenance/test_check_release.py`
  exercises every rule (27 tests; passing case + 6 failure modes covered).
  Run from worktree root: `python -m unittest tools/asset-provenance/test_check_release.py`.
- [x] **AC7 -- Presentation boundary preserved**: ADR-021 is unmodified except
  for a single Related-Decisions link to ADR-025. No code under `client/**`,
  `server/**`, or `shared/**` is touched. The bevy_ui / world-space rendering
  boundary, `CardAtlas` / `BoardLayout` resource ownership, and PresentationSet
  ordering are unaffected.
- [x] **AC8 -- No source asset copy**: `git diff origin/main..HEAD -- assets/`
  is empty for the PROMPT 1369 branch. No file under `assets/**` is added,
  removed, modified, or renamed.
- [x] **AC9 -- No release claim**: This story file's Status / No-Claim Banner
  and ADR-025's Non-Claims section both state that Krosmaga proxy assets are
  not release-approved and that no `PAW-TD-*-a` row is closed by this
  implementation.

---

## Worker Contract

1. Worktree slug: `work/s18-paw-krosmaga-dev-proxy-pack-boundary` (PROMPT
   1369 used `work/s18-paw-krosmaga-dev-proxy-boundary-1369` to keep the
   branch name PROMPT-tagged; this satisfies the slug intent).
2. Read PROMPT 1257, 1258, 1265, 1266, and 1267 before implementation.
3. Keep the change docs/tooling only; do not copy Krosmaga assets.
4. If a validation test touches Rust/Bevy code, activate `liv-bevy-018`.
5. Run only targeted validation/tooling checks needed for AC6.
6. Push the worker branch only; do not commit to `main`.

## PROMPT 1369 Implementation Notes

- Branch: `work/s18-paw-krosmaga-dev-proxy-boundary-1369` off
  `origin/main@daa7759`.
- Worktree: `D:/_DEV/claude-code-game-studios-worktrees/paw-krosmaga-dev-proxy-1369`.
- No Cargo was invoked. Cargo policy was therefore not applied. The
  validator is pure Python 3 stdlib (no third-party deps), invoked via
  `python -m unittest`.
- Files added: `docs/architecture/adr-025-asset-pack-provenance-architecture.md`,
  `design/assets/provenance/README.md`,
  `design/assets/provenance/schema.md`,
  `design/assets/provenance/logical-id-index.md`,
  `design/assets/provenance/dev-pack-example.toml`,
  `tools/asset-provenance/README.md`,
  `tools/asset-provenance/check_release.py`,
  `tools/asset-provenance/test_check_release.py`,
  `tools/asset-provenance/fixtures/release-manifest-{clean,krosmaga-leak,dev-path,unapproved}.json`.
- Files edited (small additions only): `.gitignore`,
  `design/assets/asset-manifest.md`,
  `docs/architecture/adr-021-presentation-layer-architecture.md`
  (one Related-Decisions bullet — no rendering-boundary content changed),
  `production/epics/presentation-asset-wiring/EPIC.md`,
  this story file.
- Validator unit test result: 27/27 passing.
- `git diff origin/main -- assets/` is empty.
- No file under `client/`, `server/`, or `shared/` was touched.
- No `production/sprint-status.yaml`, `production/sprints/**`,
  `production/stage.txt`, `production/qa/**`, or
  `production/gate-checks/**` file was touched.
- No story-done paperwork, smoke check, gate-check, or QA sign-off was
  performed by this prompt.

