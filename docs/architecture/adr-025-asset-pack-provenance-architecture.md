# ADR-025: Asset-Pack Provenance Architecture and Dev-Only Krosmaga Proxy Boundary

## Status

Accepted

## Date

2026-05-19

## Last Verified

2026-05-19

## Summary

Defines a three-axis provenance taxonomy (`workflow_status`, `source_class`,
`release_class`), introduces a logical asset ID layer, formalises a dev-only
"Krosmaga proxy" asset pack outside the runtime `assets/` tree, and specifies
the release-scan rules that must fail any package containing dev-only,
non-approved, or licensed-proxy material. This boundary lets later Krosmaga-
style Sprint 18/19 UI stories use Krosmaga-derived visual references on a
developer workstation without copying Krosmaga files into the repo, into
release artifacts, or into any claim of final-art / asset-clearance closure.

This ADR does **not** approve any Krosmaga asset for release, does not close
any `PAW-TD-*-a` final-art accept-risk row, does not activate Sprint 18 or
Sprint 19 rows, does not modify shared status trackers, and does not promote
any presentation surface from `Needed` to `Approved`. It is the docs/tooling
boundary that makes those later steps possible without leaking dev-only
material into a packaged build.

## Engine Compatibility

| Field | Value |
|-------|-------|
| **Engine** | Bevy 0.18 (no engine code changes in this ADR) |
| **Domain** | Asset pipeline / governance / release packaging |
| **Knowledge Risk** | LOW — the change set is documentation, manifest appendix, schema markdown, validator script, and `.gitignore` rules. No `bevy`/`lightyear` API surface is touched. |
| **References Consulted** | `docs/architecture/adr-021-presentation-layer-architecture.md` (presentation rendering boundary), `production/epics/presentation-asset-wiring/EPIC.md` (TR-PAW-001 through TR-PAW-007), `design/assets/asset-manifest.md` (current 296-row catalog and status taxonomy), `production/epics/presentation-asset-wiring/story-007-krosmaga-dev-proxy-pack-boundary.md` (acceptance criteria), `production/epics/presentation-asset-wiring/story-001-asset-wiring-foundation.md` through `story-006-lobby-portraits.md` (logical-slot consumers). |
| **Post-Cutoff APIs Used** | None. ADR introduces no Bevy/Lightyear API usage. |
| **Verification Required** | (1) Release-scan validator must fail any packaged asset that resolves to `source_class=licensed_krosmaga_dev_proxy`, `release_class != release_allowed`, `workflow_status != approved`, missing approval evidence, or a `dev-assets/**` path. (2) `git diff -- assets/` empty after this ADR's implementation. (3) No file under `dev-assets/krosmaga-proxy/**` is tracked by git. |

## ADR Dependencies

| Field | Value |
|-------|-------|
| **Depends On** | ADR-021 (presentation layer architecture — logical UI surfaces and the bevy_ui / world-space rendering boundary that the logical asset IDs map onto). |
| **Enables** | Sprint 18/19 Krosmaga-style UI implementation stories that read frames, board cells, overlay chrome, result chrome, targeting markers from dev-only proxies without claiming release readiness. Future `PAW-TD-*-a` final-art rows when production art is delivered through the same logical-ID layer. |
| **Blocks** | Any release package, RC build, certification submission, or "PAW-TD final-art closed" claim while a Krosmaga-source path resolves through a logical asset ID — the release scan fails such a build. |
| **Ordering Note** | This ADR is docs/tooling only. It is safe to Accept before any Krosmaga-style UI implementation story opens. Story 007 implementation lands the manifest appendix, schema, validator, and `.gitignore` rules. Each later Krosmaga-style story references this ADR for the provenance contract. |

## Context

### Problem Statement

The presentation-asset-wiring epic (`production/epics/presentation-asset-wiring/EPIC.md`)
defines per-surface path constants and fallback chains so that final art
delivery is a single path swap in `asset_wiring.rs`. That mechanism assumes a
flat trust model: every asset resolved by `asset_wiring.rs` is producible,
ownable, and shippable. It works for placeholder PNGs the studio owns and for
final art delivered through the production pipeline. It does **not** model
"this PNG is a developer-local visual reference derived from a third-party
licensed game (Krosmaga), used to validate state-feedback and composition
ideas during early-Sprint UI prototyping, but cannot ever appear in a
packaged build."

Without an explicit boundary, three failure modes are possible:

1. A developer drops a Krosmaga screenshot crop into `assets/art/ui/<surface>/`
   to make a layout look right while iterating on Hand UI or HUD chrome. A
   subsequent commit pushes the file to `origin/main`. The studio now ships
   third-party material it has no license to redistribute.

2. A status sweep flips a Krosmaga-proxy row in `design/assets/asset-manifest.md`
   from `Generated Placeholder` to `Done` or `Approved`. Release tooling
   accepts the row as packaged-ready. Same outcome as (1) on the next build.

3. The `PAW-TD-*-a` final-art accept-risk rows close on the strength of "all
   slots show art" without distinguishing real-final-art delivery from
   dev-only proxy substitution. A milestone gate or playtest sign-off then
   claims completion the asset pipeline has not actually achieved.

The Krosmaga-style implementation wave described in the source reports
(PROMPT 1257, 1258, 1265, 1266, 1267, 1353) intentionally wants
developer-local proxies during prototyping for state-feedback and
composition reference. The boundary must therefore be **structural** (a
separate, untracked dev-pack path; a logical-ID layer; a validator
gate), not procedural ("please don't commit it"). Procedural rules erode
under sprint pressure; structural rules do not.

### Constraints

- **No Krosmaga file may enter `assets/**`.** The runtime `assets/` tree is
  the source of every packaged build. Anything inside `assets/` is assumed
  release-eligible by the existing pipeline.
- **No Krosmaga file may enter the repo at all** — even outside `assets/`,
  a tracked Krosmaga file remains a redistribution risk. Dev-only metadata
  about the local proxy pack may be tracked; the proxy PNG/JPG/WAV payload
  itself stays in a gitignored directory on the developer's workstation.
- **No code change to `client/`, `server/`, or `shared/` gameplay/source
  unless required by the validator test.** This story is explicitly scoped
  as docs/tooling.
- **No change to shared status trackers.** `production/sprint-status.yaml`,
  `production/sprints/**`, `production/stage.txt`, `production/qa/**`, and
  `production/gate-checks/**` are untouched. This ADR carries no claim of
  release readiness, RC closure, Standard-tier accessibility completion,
  PAW-TD closure, or playtest validation.
- **ADR-021 rendering boundary must hold.** Logical asset IDs describe what
  art a UI surface needs; they do not move the bevy_ui / world-space sprite
  boundary, they do not change ownership of any UI plugin, and they do not
  reshape gameplay logic.

### Requirements

- **R1**: Every asset reference resolvable by future `asset_wiring.rs` (or
  any equivalent loader) carries enough metadata to decide release
  eligibility independently of how the file was named or where it lives in
  `assets/`.
- **R2**: A logical asset ID layer separates "what the UI needs" (`card_frame_common`,
  `hud_objective_dot_real_revealed`, `board_cell_idle_32x32`) from "which
  concrete file is in front of it today" so the studio can swap the
  underlying source from placeholder → dev-proxy → final art without
  rewriting call sites.
- **R3**: A dev-only pack convention exists, with a documented, gitignored
  workstation path, so developers can resolve Krosmaga-derived proxies
  during prototyping without those files entering the repository.
- **R4**: A release-time validator deterministically fails when any packaged
  asset resolves through a Krosmaga-proxy source, a non-approved workflow
  status, a non-release-allowed release class, missing approval evidence, or
  a `dev-assets/**` path.
- **R5**: The asset-manifest catalog (296 rows today) is not rewritten
  row-by-row; the three-axis provenance fields are introduced via a small
  appendix and a separate provenance index file. Implicit defaults for
  existing rows are documented so the appendix doesn't silently change the
  release eligibility of in-flight work.

## Decision

### 1. Three-Axis Provenance Taxonomy

Every logical asset and every concrete asset file is described by three
independent axes. They are independent because a developer can hold a
production-status file that came from an unsafe source, or hold a safe-source
file that is still in production work-in-progress — the release gate must
reject either case.

**`workflow_status`** — *Production readiness.* What stage of the studio's
production pipeline the row is in. Mirrors the existing manifest status
column but with explicit names so the release gate can compare.

- `needed` — no usable delivery file is tracked yet, or only an ownership
  placeholder exists. Includes the current manifest categories `Needed`,
  `Placeholder`, `File Present Placeholder`, `Generated Placeholder`. Not
  release-eligible.
- `in_progress` — production work is underway; a delivery file may exist
  but no final sign-off has been recorded. Not release-eligible.
- `done` — final delivery is complete with supporting evidence (file in
  the right location, correct technical dimensions, art lead has reviewed
  the file). Not yet release-eligible — `approved` is the release gate.
- `approved` — production approval/sign-off is complete and recorded.
  Release-eligible if (and only if) `source_class` and `release_class` also
  allow release.
- `blocked` — tracked but waiting on unresolved design/UX/dependency. Not
  release-eligible.

**`source_class`** — *File provenance.* Where the bits originated.

- `studio_original` — produced inside the studio (in-house art, in-house
  audio, in-house generated placeholder, in-house code asset). The default
  for any new manifest row. Release-eligible if `workflow_status=approved`
  and `release_class=release_allowed`.
- `licensed_external_release` — sourced from an external party under a
  license that allows redistribution in the studio's released build (a
  signed font, a licensed SFX pack, etc.). Release-eligible under the same
  rules as `studio_original`. *No row currently uses this; reserved for
  future expansions.*
- `licensed_krosmaga_dev_proxy` — derived from Krosmaga (Ankama) game
  material. Owned by Ankama. Used in-studio only as a visual reference for
  state-feedback and composition prototyping during early Sprint 18/19
  Krosmaga-style work. **Never release-eligible.** The release scan must
  hard-fail any packaged asset that resolves through this source class.
- `unknown_provenance` — sentinel for any logical ID where the source has
  not been classified. Treated as not release-eligible until classified.
  The release scan fails on this value the same way it fails on
  `licensed_krosmaga_dev_proxy`.

**`release_class`** — *Packaging permission.* Whether the row is allowed in
a packaged build at all, independent of where it currently is in production.

- `release_allowed` — may be included in a packaged build once
  `workflow_status=approved` and `source_class` permits it.
- `dev_only` — used during development only. The release-scan validator
  fails any packaged build that contains a row with this value. Krosmaga
  proxies are always `dev_only`. Internal debug assets (debug font, debug
  grid overlay, dev console PNG, etc.) may also be `dev_only`.
- `internal_only` — used inside the studio (e.g., production handoff
  packets, art bible exports) but never shipped in a player-facing build.
  Same release-gate behaviour as `dev_only`.

### 2. Logical Asset ID Layer

Every UI surface that needs art references a **logical asset ID**, not a
concrete file path. Examples (the canonical list lives in
`design/assets/provenance/logical-id-index.md`):

- `lid_card_frame_common`, `lid_card_frame_rare`, `lid_card_frame_epic`,
  `lid_card_frame_legendary` — Hand UI card frame chrome per rarity.
- `lid_card_stat_badge_atk`, `lid_card_stat_badge_hp`, `lid_card_cost_badge_mana`
  — Hand UI stat/cost badges.
- `lid_hud_class_figurine_{class_id}` — HUD class figurine per class.
- `lid_hud_objective_dot_{state}` — HUD objective dot per state (`unknown`,
  `real_revealed`, `fake_revealed`, `destroyed`).
- `lid_board_cell_idle_32x32`, `lid_board_cell_spawn_active_32x32`,
  `lid_board_cell_invalid_32x32` — Board cell node states.
- `lid_board_unit_base_player_a_48x16`, `lid_board_unit_base_player_b_48x16`
  — Board unit base sprites per player.
- `lid_overlay_targeting_marker_real`, `lid_overlay_targeting_marker_fake`
  — Sang Méprise reveal markers.
- `lid_result_panel_chrome_win`, `lid_result_panel_chrome_loss`,
  `lid_result_panel_chrome_draw` — Result chrome (overlay-only until
  result-screen UX is unblocked).
- `lid_ui_placeholder_1x1_white` — Universal fallback per ADR-021 path
  convention.

A **logical ID resolves to a concrete pack entry** through a pack selection
order:

1. If the active build is a release build, only `release_allowed` packs are
   eligible.
2. If the active build is a developer workstation build *and* the dev-only
   Krosmaga proxy pack exists at `dev-assets/krosmaga-proxy/`, that pack
   may resolve logical IDs.
3. Otherwise the studio placeholder PNG / generated placeholder is
   resolved.
4. If no entry exists, fallback to `lid_ui_placeholder_1x1_white`.

The logical-ID layer is **declarative metadata, not new runtime code**.
Story 007's implementation lands the index markdown plus the validator;
later stories adapt `asset_wiring.rs` if/when they choose to consume the
logical-ID layer. ADR-021's bevy_ui vs world-space boundary is not affected
— a logical ID like `lid_board_cell_idle_32x32` still resolves to a
world-space `Sprite`, and `lid_card_frame_common` still resolves to a
bevy_ui `ImageNode`.

### 3. Dev-Only Krosmaga Proxy Pack

A developer workstation may carry a local Krosmaga-derived proxy pack at:

```
dev-assets/krosmaga-proxy/
├── pack.toml              # pack metadata (gitignored; example committed at design/assets/provenance/dev-pack-example.toml)
├── frames/                # card frame proxies per rarity
├── board/                 # board cell / lane / objective proxies
├── overlays/              # reveal, hover, target chrome proxies
├── result/                # result panel chrome proxies
└── audio/                 # state-feedback sound proxies (if applicable)
```

The entire `dev-assets/` tree is gitignored. The pack manifest `pack.toml`
declares:

- `pack_id` (e.g., `krosmaga-proxy-v1`).
- `source_class` (must be `licensed_krosmaga_dev_proxy` for this pack).
- `release_class` (must be `dev_only` for this pack).
- A map of logical ID → relative path within the pack.

A safe-to-commit **example** pack manifest is checked in at
`design/assets/provenance/dev-pack-example.toml`. The example lists logical
IDs and placeholder relative paths but contains no Krosmaga payload. The
example is documentation, not a usable pack — it shows the schema developers
must follow for their local pack.

### 4. Release-Scan Failure Rules

The release-scan validator (Story 007 implementation lands it under
`tools/asset-provenance/`) walks the set of packaged assets and the
provenance metadata. It **fails the scan** (non-zero exit, structured error
report) if any packaged-asset resolution satisfies any of:

- `source_class == licensed_krosmaga_dev_proxy`.
- `source_class == unknown_provenance`.
- `release_class != release_allowed`.
- `workflow_status != approved`.
- Logical-ID lookup is missing `approval_evidence` and `workflow_status` is
  not `approved`.
- The resolved concrete path begins with `dev-assets/` or any descendant.

The scan must run as part of any future release-packaging pipeline and as
part of any CI matrix entry that produces a shippable artifact. Until those
pipelines exist, the validator is invocable manually and via the validator
unit test that this story lands.

### 5. ADR-021 Cross-Link

ADR-021 (presentation layer architecture) gains a small Related-Decisions
entry pointing to this ADR. ADR-021 remains the authority on
plugin composition, SystemSet ordering, the bevy_ui / world-space rendering
boundary, and the `CardAtlas` / `BoardLayout` resource ownership.

## Alternatives Considered

### Alternative 1: Rely on a per-row `Public Release Clearance` column in `asset-manifest.md`

- **Description**: Add a single column to every row of `asset-manifest.md`
  recording redistribution permission. No logical-ID layer, no separate
  validator, no dev-pack convention.
- **Pros**: Minimal new infrastructure. One file to read.
- **Cons**: Forces a 296-row rewrite when introducing the new column.
  Single-axis encoding muddles three distinct concepts (`workflow_status`,
  `source_class`, `release_class`) so the release-gate question ("is this
  shippable today?") can't be answered without recomputing per-row. No
  developer-local pack story. Procedural-only enforcement; nothing
  structurally prevents a Krosmaga PNG from landing in `assets/`.
- **Rejection Reason**: The three-axis taxonomy is necessary precisely
  because a single status column collapses release independence into
  production-progress. Story 007's acceptance criteria require a
  release-scan tool, which only a multi-axis schema can drive.

### Alternative 2: Allow Krosmaga proxies in `assets/` under a special `assets/.dev/` subfolder

- **Description**: Place Krosmaga proxies inside the `assets/` tree but
  under a marked sub-path. Have the release packager exclude `assets/.dev/`.
- **Pros**: No new top-level directory. No `.gitignore` change.
- **Cons**: The proxies still enter the git history if anyone forgets the
  `.dev/` subfolder convention. The runtime asset pipeline still has the
  files indexed. A future contributor who renames `.dev/` or moves files
  out of it breaks the boundary silently. The packager exclusion is a
  procedural rule on a shared tree.
- **Rejection Reason**: The boundary must be structural. A separate
  top-level, fully gitignored `dev-assets/` tree cannot accidentally enter
  the runtime asset bundle and cannot accidentally appear in a `git diff
  -- assets/`. A single allowlisted dev sub-folder under `assets/` does
  not survive the next refactor.

### Alternative 3: Reuse a generic "asset registry" pattern with per-asset license fields

- **Description**: Build a heavier asset-registry abstraction with full
  license metadata per asset (license name, license URL, attribution
  string, expiration date, etc.). Krosmaga-proxy is one license type
  among many.
- **Pros**: Future-proof for any licensed asset class.
- **Cons**: Speculative complexity. Today the only third-party material on
  the table is the Krosmaga dev proxy; no other licensed source is in
  scope. Designing a full license-management schema now would consume
  Sprint 18 capacity for a need that has not appeared.
- **Rejection Reason**: YAGNI. The three-axis taxonomy is the minimum
  schema that draws the Krosmaga boundary. A heavier license registry can
  be added later by extending `source_class` and `release_class` value sets
  — non-breaking.

### Alternative 4: Document the rule in `CLAUDE.md` only

- **Description**: Add a paragraph to `CLAUDE.md` (or `coding-standards.md`)
  telling contributors not to copy Krosmaga material into the repo. No
  schema, no logical-ID layer, no validator.
- **Pros**: Zero infrastructure cost.
- **Cons**: Procedural-only. The next sprint pressure to "just make the
  board look right" defeats the rule. No release-gate enforcement. No
  audit trail.
- **Rejection Reason**: Story 007 explicitly asks for a release-scan
  validator (AC6) and a logical-ID schema (AC3). A documentation-only rule
  does not satisfy the acceptance criteria and does not produce a
  structural boundary.

## Consequences

### Positive

- Krosmaga-style Sprint 18/19 work can use Krosmaga visual references
  during prototyping without the licensing or release-integrity risk that
  would otherwise block the work or force per-PR review.
- The logical-ID layer aligns with TR-PAW-001's stated invariant: final art
  delivery is a single swap (now: a single pack-entry remap) — no
  call-site change.
- The release-scan validator gives the studio an explicit, automatable
  go/no-go check for packaging, instead of a per-PR human review of every
  asset path.
- The existing 296-row `asset-manifest.md` is not rewritten. The appendix
  documents the implicit default (`workflow_status` derived from current
  status column; `source_class=studio_original`; `release_class=release_allowed`)
  so in-flight rows keep their semantics.
- Sprint 18 row authoring is unblocked for stories that want to use
  Krosmaga proxies on a developer workstation. No status-tracker change is
  required to satisfy this ADR's contract.

### Negative

- A second source of truth exists for "what art a UI surface needs": the
  per-surface story files (e.g., `story-002-hand-ui-card-frames.md`) and
  the logical-ID index (`design/assets/provenance/logical-id-index.md`).
  Mitigated by treating the logical-ID index as the canonical layer and
  cross-linking from each PAW story when it adopts the layer.
- The dev-only Krosmaga proxy pack is per-workstation. Onboarding a new
  developer to the Krosmaga-style implementation wave requires giving them
  the pack out-of-band (the studio's internal asset share, not via the
  repo). Mitigated by documenting the expected pack layout in
  `design/assets/provenance/dev-pack-example.toml` and the README in
  `design/assets/provenance/`.
- The release-scan validator must be wired into any future CI release-
  packaging job. Not wired in this story. Acknowledged in the validator
  README: the tool exists and is unit-tested, but the broader CI integration
  is a separate piece of work.

### Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| A developer puts a Krosmaga PNG inside `assets/` despite the rule. | Medium | High | The validator's test fixture explicitly covers this case; once wired into CI it becomes a hard gate. Until CI wires it, code-review checklist references this ADR. |
| Existing manifest rows are misinterpreted as `licensed_krosmaga_dev_proxy` after the appendix lands. | Low | Medium | The appendix explicitly states the **default** for any existing row is `studio_original` / `workflow_status` derived from current status column / `release_class=release_allowed`. Krosmaga proxy classification only applies to rows that *opt in* via the logical-ID index. |
| The logical-ID index drifts from the per-surface stories. | Medium | Low | The provenance README lists which PAW stories own which logical-ID slices and treats the index as the canonical mapping. Each PAW story that adopts the layer must reference the index entry. |
| A future release-packaging tool packages assets directly out of the source tree without invoking the validator. | Medium | High | Validator README notes the requirement; downstream story (when CI release packaging is wired) must take this ADR as the gating dependency. Validator is invocable as a CLI for any pipeline. |

## GDD Requirements Addressed

| Requirement | How This ADR Addresses It |
|-------------|--------------------------|
| Source reports PROMPT 1257 (three-axis provenance taxonomy proposal) | Ratified as `workflow_status` / `source_class` / `release_class` with explicit value sets and release-gate semantics. |
| Source reports PROMPT 1258 + 1267 (Krosmaga proxies stay dev-only and outside release packaging) | `source_class=licensed_krosmaga_dev_proxy` enforced via release-scan validator; pack lives under gitignored `dev-assets/`. |
| Source reports PROMPT 1265 + 1266 (Krosmaga is state-feedback and composition reference, not a release asset source) | Logical-ID layer separates "what the UI needs" from "which concrete file resolves it" so dev-only proxy substitution does not blur into release-eligible delivery. |
| Source report PROMPT 1353 (Krosmaga-style implementation wave readiness map) | This ADR is the prerequisite the wave readiness map identifies for any story that wires a Krosmaga-derived proxy. |
| Story 007 AC1–AC9 | AC1 ADR/provenance doc → this file. AC2 manifest appendix → `design/assets/asset-manifest.md` appendix. AC3 logical-ID schema/index → `design/assets/provenance/`. AC4 explicit Krosmaga classification → enumerated above. AC5 dev-pack stays untracked → gitignored `dev-assets/krosmaga-proxy/`. AC6 release scan blocks dev proxies → validator under `tools/asset-provenance/`. AC7 presentation boundary preserved → ADR-021 untouched on rendering boundary. AC8 no asset copy → `git diff -- assets/` empty. AC9 no release claim → repeated in the no-claim banner and in this ADR's Non-Claims section. |

## Non-Claims

This ADR explicitly does **not** establish or imply any of the following:

- That any Krosmaga-derived asset is approved for release.
- That `PAW-TD-*-a` final-art accept-risk rows are closed.
- That Sprint 18 Must Have rows have been expanded.
- That Sprint 19 has been activated.
- That release readiness, RC closure, certification, or store submission
  has been achieved or attempted.
- That Standard-tier accessibility coverage is complete.
- That playtest validation has occurred for Krosmaga-style chrome.
- That `production/sprint-status.yaml`, `production/sprints/**`,
  `production/stage.txt`, `production/qa/**`, or `production/gate-checks/**`
  have been updated by Story 007's implementation. They have not.

## Performance Implications

None — this ADR introduces no runtime code, no Bevy systems, no Lightyear
message types, no `Cargo.toml` dependency. The validator is a Python script
invoked outside the build/runtime hot path.

## Migration Plan

1. **Land the ADR (this file).** No code touched.
2. **Land the manifest appendix.** A new section at the bottom of
   `design/assets/asset-manifest.md` defines the three-axis fields and the
   implicit defaults for existing rows.
3. **Land the provenance schema + logical-ID index.** New files under
   `design/assets/provenance/`: `README.md`, `schema.md`,
   `logical-id-index.md`, `dev-pack-example.toml`.
4. **Land the validator.** New tool under `tools/asset-provenance/` with a
   self-contained unit test covering the failure modes in §4.
5. **Add `dev-assets/` to `.gitignore`.** Already gitignored implicitly
   only via per-file rules — make it explicit.
6. **Cross-link ADR-021.** Small Related-Decisions edit. No content change
   to ADR-021's rendering-boundary decision.
7. **Annotate EPIC.md.** Add the logical-ID / source-swap invariant
   to the presentation-asset-wiring EPIC.

No data migration. No rerun of `cargo` or any Bevy system. No story
status flip required outside Story 007 itself.

## Validation Criteria

- [ ] `design/assets/provenance/schema.md` defines the three-axis taxonomy
  with explicit value enumerations matching this ADR's §1.
- [ ] `design/assets/provenance/logical-id-index.md` lists at least the
  card frame, hand stat/cost badge, HUD class figurine, HUD objective dot,
  board cell, board unit base, Sang Méprise reveal marker, result panel
  chrome, and shared placeholder logical IDs.
- [ ] `design/assets/provenance/dev-pack-example.toml` is a syntactically
  valid TOML file demonstrating the pack schema without including any
  Krosmaga payload.
- [ ] `design/assets/asset-manifest.md` carries a clearly delimited
  provenance appendix referencing this ADR.
- [ ] `.gitignore` excludes `dev-assets/`.
- [ ] `tools/asset-provenance/check_release.py` (or equivalent) exits
  non-zero with a structured error when any of §4's failure modes apply,
  and exits zero on a clean studio-original / approved / release-allowed
  manifest.
- [ ] A unit test (`tools/asset-provenance/test_check_release.py` or a Rust
  test under `tests/`) covers each of the six failure modes plus the
  passing case.
- [ ] `git diff origin/main..HEAD -- assets/` is empty for the Story 007
  branch.
- [ ] `ADR-021` `Related Decisions` references this ADR.
- [ ] `production/epics/presentation-asset-wiring/EPIC.md` carries a note
  about the logical-ID / source-swap invariant under TR-PAW-007.

## Related Decisions

- [ADR-021 — Presentation Layer Architecture](./adr-021-presentation-layer-architecture.md) —
  rendering boundary and shared resource ownership; the logical-ID layer
  in this ADR maps onto ADR-021's bevy_ui / world-space split without
  changing it.
- [ADR-004 — Asset Loading Pipeline](./adr-004-asset-loading-pipeline.md) —
  `bevy_asset_loader` LoadingState that ultimately consumes the resolved
  pack entries. Unaffected by this ADR; the pack selection is upstream of
  the LoadingState handle list.
- `production/epics/presentation-asset-wiring/EPIC.md` — TR-PAW-001
  through TR-PAW-007 (Story 007 traces to this ADR).
- `design/assets/asset-manifest.md` — existing 296-row catalog; the
  provenance appendix extends the catalog without rewriting any row.
