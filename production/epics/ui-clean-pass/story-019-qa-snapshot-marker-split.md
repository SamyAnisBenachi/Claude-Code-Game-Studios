# Story 019: S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001 -- QA Snapshot Marker Split + Visibility-Aware Counts + Session ID Prefix

> **Epic**: UI Clean-Pass
> **Story ID**: S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001
> **Status**: Draft -- Sprint 17 Should Have candidate (SOURCE-1077-08 + SOURCE-1077-09 + SOURCE-1077-16 bundle); NOT activated by this authoring run
> **Layer**: Presentation -- QA snapshot tooling + per-surface UI markers (cross-cut: hud + hand + shop_auction + presentation overlays)
> **Type**: Tech Debt -- structural refactor + tooling correctness
> **Sprint**: Sprint 17 Should Have row per `production/sprints/sprint-17.md` §"Should Have". Activation is a separate explicit prompt (PROMPT 1093 pattern).
> **Authored**: 2026-05-18 by PROMPT 1095
> **Authoring source-of-truth**: `origin/main@7d36191fe94adf99d3448a58185d8079d828c29e`
> **Estimated effort**: ~0.25d (bundled SOURCE-1077-08 + SOURCE-1077-09 + SOURCE-1077-16; cross-module marker rename + visibility filter + snapshot ID prefix)
> **Source audit**: PROMPT 1077 `reports/PROMPT-1077-ui-state-source-consistency-deep-audit.md` §"Per-finding evidence" SOURCE-1077-08 (P2), SOURCE-1077-09 (P2), SOURCE-1077-16 (P3)

---

## Target Epic Justification

This story is filed under `production/epics/ui-clean-pass/` rather
than `production/epics/presentation-layer/`. Justification:

- The repo's UI Clean-Pass epic is the documented home for cross-
  cutting UI structural / consistency refactors (story 010 shop /
  auction modsplit, story 011 hand modsplit, story 015 architecture
  sequencing, story 016 dead-code warning cleanup — all cross-cut
  hand + shop_auction + qa_snapshot files).
- The change in this row spans `client/src/presentation/qa_snapshot.rs`
  AND marker definitions across `client/src/ui/hud/`,
  `client/src/ui/hand/`, `client/src/ui/shop_auction/`,
  `client/src/presentation/connection_lost_overlay.rs`, and
  `client/src/presentation/result_screen.rs` (per
  `client/src/presentation/board_rendering.rs` for any board-marker
  rename). It is **not** owned by any one surface.
- The Presentation Layer epic (`production/epics/presentation-layer/`)
  scopes ADR-021 plugin / phase-sink / accessibility-control /
  result-screen ownership — not QA-snapshot tooling correctness or
  per-surface marker granularity. Per the Presentation Layer
  EPIC.md §Overview: "This epic exists because ADR-021 defines
  cross-epic infrastructure that should not be owned by Board
  Rendering 001 or Shop/Auction UI 001." The qa_snapshot tool is
  presentation-layer code but its consumer is QA tooling, not the
  ADR-021 plugin composition.
- Sprint 17 plan row source allows either epic ("Target epic:
  production/epics/ui-clean-pass/ or production/epics/presentation-
  layer/ if the repo's existing ownership pattern points there").
  The UI Clean-Pass epic's existing story set demonstrates the
  cross-cutting-UI-refactor ownership pattern; choosing it keeps
  the SOURCE-1077-* bundle (stories 017 / 018 / 019 under this
  epic) co-located.

---

## Status / No-Claim Banner

This story is a Sprint 17 Should Have **candidate** authored by PROMPT
1095. **No sprint is activated by this authoring run.** PROMPT 1095
does NOT modify `production/sprint-status.yaml`,
`production/sprints/sprint-17.md`, `production/sprints/sprint-16.md`,
`production/stage.txt`, any `production/session-state/*` file, any
QA-plan / smoke / Team-QA / gate-check / release-check artifact
under `production/qa/`, any code under `client/`, `server/`,
`shared/`, `tests/`, `Cargo.toml`, `Cargo.lock`, `.cargo/`,
`.github/`, or `Trunk.toml`. PROMPT 1095 does NOT run
`/story-readiness`, `/dev-story`, `/story-done`, `/smoke-check`,
`/team-qa`, `/gate-check`, `/release-check`, `/qa-plan`, `cargo`,
`trunk`, or any CI command.

This story does **not** claim: public release readiness, release-
candidate readiness, full game completion, broad / Standard-tier
accessibility completion (`QA-COND-0005`), Standard-tier hit-target
conformance, playtest / fun-hypothesis validation (`QA-COND-0006`),
full playable-client manual QA, two-client GAME_OVER closure
(`S8-QA-001-W1`), final-art / asset-production completion
(`PAW-TD-*-a`), `Polish->Release` gate-check retry, stage advance,
closure of the Sprint 12 story 019 underlying drag-runtime bug,
closure of `S11-HUD-TIMER-EYEBALL-VISUAL-001`, closure of any of
the 24 PROMPT 1022 audit findings (this row improves the QA
snapshot pipeline that 1022 used; it does NOT close 1022 findings),
closure of any AUDIT-1076-* finding, or closure of any SOURCE-1077-*
finding outside the three bundled here (-08 / -09 / -16).

**No optimistic client-side authority is introduced or proposed.**
No protocol shape change. No new server-authoritative state. No new
C2S / S2C message.

Sprint 16 disposition preserved unchanged. Sprint 15 / 14 / 13 / 12
/ 11 / 10 dispositions preserved unchanged. PROMPT 761 Polish->Release
gate-check `FAIL` evidence preserved. `PAW-TD-*-a`, `QA-COND-0005`,
`QA-COND-0006`, `TQ-S12-C1..C7` preserved verbatim.

---

## Source Findings

This story bundles three PROMPT 1077 findings because they share
`client/src/presentation/qa_snapshot.rs` and are cheaper to land
together than as separate prompts.

### SOURCE-1077-08 (P2) — UI marker components are too coarse for the QA snapshot

- **Audit location**:
  `reports/PROMPT-1077-ui-state-source-consistency-deep-audit.md`
  §"Per-finding evidence" SOURCE-1077-08 (P2).
- **Affected file lines at audit time**:
  - `client/src/presentation/qa_snapshot.rs:403-435`
    (`UiCountQueries`).
  - Markers: `crate::ui::shop_auction::ShopAuctionUiEntity`
    (line 408), `crate::ui::hand::HandUiEntity` (line 407),
    `crate::ui::hud::HudEntity` (line 405).
- **Symptom**: every snapshot in the PROMPT 1022 / 1034 / 1036
  captures shows `shop_auction_entities = 78` regardless of phase
  because the counter measures the spawned tree, not the visible
  sub-surface. The QA snapshot pipeline cannot machine-detect "shop
  has no UI in DraftShop" because the counter is constant.

### SOURCE-1077-09 (P2) — Persistent overlay roots inflate marker counts

- **Audit location**: same report §"Per-finding evidence"
  SOURCE-1077-09 (P2).
- **Affected file lines at audit time**:
  - `client/src/presentation/connection_lost_overlay.rs:66`
    (`spawn_connection_lost_overlay_system` registered on
    `Startup`).
  - `client/src/presentation/result_screen.rs` analogous Startup
    spawn.
  - `client/src/presentation/qa_snapshot.rs:411-419` queries the
    marker with no `Visibility::Visible` filter, returning 1 even
    when the root is `Visibility::Hidden`.
- **Symptom**: QA snapshot reports
  `connection_lost_overlay_roots = 1` and `result_screen_roots = 1`
  across every captured frame regardless of whether the overlay is
  visible.

### SOURCE-1077-16 (P3) — Snapshot ID prefix aliases across concurrent clients

- **Audit location**: same report §"Per-finding evidence"
  SOURCE-1077-16 (P3).
- **Affected file lines at audit time**:
  `client/src/presentation/qa_snapshot.rs:1010-1012`
  (cited by PROMPT 1036 M11; confirmed by PROMPT 1077 audit).
- **Format at audit**: `{counter:06}-{unix_millis}`. The `counter`
  is an `AtomicU64` initialised at 0 per process.
- **Symptom**: two clients running in parallel produce colliding
  `000000-*` prefixes that differ only by millisecond-precision
  `unix_millis`. Reviewer cannot tell from the directory name alone
  which client produced which frame.

---

## Problem Class / Prevention Target

**Defect class**: a QA snapshot pipeline that misreports per-surface
visibility because:

1. Universal marker components (`HudEntity`, `HandUiEntity`,
   `ShopAuctionUiEntity`) lump multiple sub-surfaces under one
   query result.
2. Marker counts ignore `Visibility::Hidden` so pre-spawned overlays
   show up as if they were active.
3. Per-client snapshot directories collide on the counter prefix
   when two clients run concurrently.

The downstream consequence is that PROMPT 1022 / 1034 / 1036 (and
any future PROMPT 1076 / 1077-style snapshot audit) cannot machine-
detect visible defects: every snapshot looks the same because the
counter is constant.

**Prevention target**:

1. **Split universal markers into per-sub-surface markers** so the
   snapshot can distinguish "shop slots visible" from "auction
   modal visible" from "settlement overlay visible". Per the PROMPT
   1077 minimal-repair surface: introduce per-root markers
   `DraftInitialModalRoot`, `ShopPanelRoot`, `AuctionPanelRoot`,
   `SettlementOverlayRoot`, `ShopFooterRoot`, `HandFanRoot` (already
   exists), `HandDraftGridRoot`, `PlacementActionPanelRoot`,
   `HudTopStripRoot`, `HudBottomStripRoot`, `ScoreboardDotRowRoot`.
   Concrete marker list TBD by implementing worker.
2. **Add `Visibility::Inherited`/`Visible` filter to marker
   queries** in `UiCountQueries.snapshot()` so hidden pre-spawned
   overlay roots stop double-counting.
3. **Prefix snapshot directory names with `session_id`** once
   `ClientSessionIdentity.session_id` is known, so concurrent-client
   captures sort by client. Pre-session captures get a
   `pre-session-` prefix.

---

## Context

### Existing surface

- **`client/src/presentation/qa_snapshot.rs`** — QA snapshot tool;
  owns `UiCountQueries` (line 403-435 at audit time) and
  `format_snapshot_id` (line 1010-1012 at audit time). Reads marker
  query results, writes JSON snapshot per client.
- **`client/src/ui/hud/`** — defines `HudEntity` universal marker.
  Sub-surface root entities (`HudTopStripRoot`, `HudBottomStripRoot`,
  `ScoreboardDotRowRoot`) may already exist or may need to be
  introduced; implementing worker re-verifies at activation HEAD.
  Story 017 (HUD opponent figurine), story 015 (HUD top strip),
  story 016 (HUD bottom strip) all already separate HUD sub-strips
  conceptually; the marker split formalises that.
- **`client/src/ui/hand/`** — defines `HandUiEntity` universal
  marker. `HandFanRoot` already exists per PROMPT 1077 audit; need
  per-sub-surface markers for `HandDraftGridRoot`,
  `PlacementActionPanelRoot`, drag-state visuals.
- **`client/src/ui/shop_auction/`** — defines `ShopAuctionUiEntity`
  universal marker. Per-sub-surface markers needed:
  `DraftInitialModalRoot`, `ShopPanelRoot`, `AuctionPanelRoot`,
  `SettlementOverlayRoot`, `ShopFooterRoot`.
- **`client/src/presentation/connection_lost_overlay.rs:66`** —
  `spawn_connection_lost_overlay_system` on `Startup`. Marker:
  `ConnectionLostOverlayRoot` (already exists).
- **`client/src/presentation/result_screen.rs`** — analogous Startup
  spawn. Marker: `ResultScreenRoot` (already exists per PROMPT
  1036 M6 / PROMPT 1077).
- **`client/src/state/mod.rs`** or `client/src/network/`
  `ClientSessionIdentity` resource. Provides `session_id` for AC3
  snapshot ID prefix.

### GDD / ADR / TR trace

- **GDD**: no GDD update in scope. QA snapshot tooling is a dev /
  QA capability not specified in `design/gdd/`.
- **ADR-021** (Presentation Layer Architecture): no system-set or
  schedule change. Marker renames / additions slot into existing
  spawn sites.
- **ADR-002** (Client-Server Authority): no change. QA snapshot is
  read-only over ECS state.
- **ADR-001** (Objective Identity Unicast): no change. Marker
  granularity doesn't affect `was_fake` exposure.
- **TR registry**: no new TR. QA snapshot tooling correctness is
  not GDD-spec'd.

### Engine / skills

- **Engine**: Bevy 0.18 (Rust).
- **Mandatory skills**: `liv-bevy-018` on every `.rs` edit. No
  Lightyear edits — `liv-bevy-lightyear` NOT required.

### Control Manifest Rules

- Required: split each universal marker (`HudEntity`, `HandUiEntity`,
  `ShopAuctionUiEntity`) into per-sub-surface markers. Sub-surface
  marker list TBD by implementing worker against the live module
  shape at activation HEAD; reference list per PROMPT 1077
  minimal-repair surface.
- Required: add a `Visibility::Inherited`/`Visible` filter in
  `UiCountQueries.snapshot()` for marker counts. The filter must
  match Bevy 0.18's `Visibility` API (re-verify under
  `liv-bevy-018`).
- Required: snapshot ID format extended to
  `{session_id}-{counter:06}-{unix_millis}`. Pre-session captures
  (before `ClientSessionIdentity.session_id` is known) use the
  prefix `pre-session-`.
- Required: emit per-sub-surface counts in the JSON snapshot
  (`hud_top_strip_visible`, `shop_panel_visible`,
  `auction_panel_visible`, etc.). Exact JSON shape TBD; PROMPT
  1036 §5.4 outlined the addition.
- Required: legacy universal counts may be kept (deprecated) OR
  removed; the implementing worker chooses and justifies. Default:
  keep deprecated for one Sprint cycle to avoid breaking PROMPT
  1022 / 1034 / 1036 historical comparison.
- Required: `PAW-TD-*-a`, `QA-COND-0005`, `QA-COND-0006` preserved.
- Forbidden: changing the snapshot JSON top-level schema in a way
  that breaks the existing CCGS_QA_SNAPSHOT button env-var
  contract (`CCGS_QA_SNAPSHOT=1`).
- Forbidden: changing the snapshot output directory layout
  outside the directory-name prefix (the prefix change is in
  scope; deeper layout changes are NOT).
- Forbidden: modifying `shared/`, `server/`, or any test under
  `tests/integration/server/` or `tests/unit/server/`.
- Forbidden: closing any of the 24 PROMPT 1022 findings. This
  story improves the tool that captured them; it does not retest
  or close them.

---

## Story Classification

**Story type**: **Integration** (multi-module marker rename + a
cross-cutting query change in qa_snapshot + a new
ClientSessionIdentity consumer).

Per `.claude/docs/coding-standards.md` "Test Evidence by Story
Type" matrix, Integration stories require integration test OR
documented playtest (BLOCKING gate). This row delivers an
integration test bin.

This is **NOT** a:

- Logic-only story (touches multiple modules + qa_snapshot tool).
- Visual / feel story (no shader, VFX, or animation curve change;
  no consumer-visible behaviour change beyond the snapshot folder
  name).
- UI layout story (no menu / HUD / screen flow change).
- Final-art story.
- Accessibility story.

---

## Dependencies and Parallelism

### Prerequisites

- None on `origin/main` at activation HEAD. This row stands alone
  on the qa_snapshot module + marker definitions.

### Parallelism summary

| Sibling story | Parallel-safe? | Notes |
|---|---|---|
| `S17-UI-CARD-DISPLAY-ART-HELPER-001` (Must) | **PARTIAL** | both edit `client/src/ui/hand/mod.rs` and `client/src/ui/shop_auction/mod.rs`. Serialise — this row needs to add markers; the helper bundle deletes helper bodies and adds `use`. File overlap on those two files. The orchestrator MUST sequence (helper bundle first OR this row first, then rebase). |
| `S17-UI-CARD-SLOT-INSET-WIRING-001` (Should) | **YES** | disjoint (`design_tokens/card_slot.rs` only). |
| `S17-UI-HUD-OPP-MANA-CLEANUP-001` (Should) | **PARTIAL** | both edit `client/src/ui/hud/`. Serialise. |
| `S17-UI-BID-BUTTON-PHASE-RACE-001` (Should) | **PARTIAL** | both edit `client/src/ui/shop_auction/`. Serialise. |
| `S17-UI-MODAL-BLACK-SLAB-001` (conditional Must) | **PARTIAL** | both edit `client/src/ui/shop_auction/mod.rs`. Serialise. |
| `S17-UI-SHOP-AUCTION-SURFACE-PAINT-001` (conditional Must) | **PARTIAL** | both edit `client/src/ui/shop_auction/`. Serialise. |
| `S17-UI-HAND-B0004-CLEANUP-001` (Nice) | **PARTIAL** | both edit `client/src/ui/hand/`. Serialise. |
| `S17-OPS-VULKAN-VALIDATION-GATING-001` (Nice) | **YES** | disjoint. |
| `S17-SERVER-START-OF-TURN-DEBUG-001` (Nice) | **YES** | disjoint. |

This row is the **most parallel-conflict-prone Sprint 17 row**
because it spans hud + hand + shop_auction + qa_snapshot. The
Sprint 17 producer SHOULD schedule it after the conditional Must
Have rows (modal + shop-auction surface paint) land, and after the
PROMPT 1077 P0 card-display-art bundle (which also edits
hand + shop_auction).

---

## Acceptance Criteria

All criteria are independently checkable.

- [x] **AC1 -- Universal markers split into per-sub-surface markers**:
  GIVEN the post-implementation client build, WHEN
  `grep -rn "pub struct HudEntity\b" client/src/ shared/src/` is
  run, THEN the universal `HudEntity` marker is either removed or
  marked `#[deprecated]`. Per-sub-surface HUD markers
  (`HudTopStripRoot`, `HudBottomStripRoot`, `ScoreboardDotRowRoot`,
  or equivalents per the implementing worker's discovery) are
  introduced. Equivalent splits land for `HandUiEntity` (sub-
  surface markers: `HandFanRoot` already exists + at least
  `HandDraftGridRoot`, `PlacementActionPanelRoot`) and
  `ShopAuctionUiEntity` (sub-surface markers per PROMPT 1077
  reference list: `DraftInitialModalRoot`, `ShopPanelRoot`,
  `AuctionPanelRoot`, `SettlementOverlayRoot`, `ShopFooterRoot`).

- [x] **AC2 -- `UiCountQueries` consumes per-sub-surface markers**:
  GIVEN `client/src/presentation/qa_snapshot.rs` post-refactor,
  WHEN inspected, THEN `UiCountQueries` and the JSON snapshot
  output include per-sub-surface visible counts (one per sub-
  surface marker introduced in AC1). The implementing worker
  chooses the JSON field names; PROMPT 1036 §5.4 reference list:
  `hud_top_strip_visible`, `shop_panel_visible`,
  `auction_panel_visible`, etc.

- [x] **AC3 -- Visibility filter in `UiCountQueries.snapshot()`**:
  GIVEN the post-refactor `UiCountQueries.snapshot()`, WHEN
  inspected, THEN every per-sub-surface marker query carries a
  `Visibility::Visible` (or `Visibility::Inherited` resolved to
  `Visible`) filter. Marker queries return 0 for any pre-spawned
  but `Visibility::Hidden` root.

- [x] **AC4 -- Connection-lost overlay visible flag honours
  Visibility**: GIVEN the post-refactor snapshot, WHEN the
  connection-lost overlay is `Visibility::Hidden`, THEN
  `connection_lost_overlay_visible` (or equivalent JSON field) is
  `false`. WHEN the overlay is `Visibility::Visible`, THEN the
  flag is `true`. The legacy `connection_lost_overlay_roots = 1`
  (marker count regardless of visibility) is either removed OR
  preserved alongside the new `_visible` flag for backwards
  compatibility — the implementing worker chooses and documents.

- [x] **AC5 -- Result-screen overlay visible flag honours
  Visibility**: same shape as AC4 for `ResultScreenRoot`.

- [x] **AC6 -- Snapshot ID prefix includes session_id (post-
  session) or `pre-session-` (before)**: GIVEN the post-refactor
  `format_snapshot_id` (or equivalent function), WHEN
  `ClientSessionIdentity.session_id` is `Some(id)`, THEN the
  returned snapshot ID format is `{id}-{counter:06}-{unix_millis}`
  (or equivalent that prefixes by session_id). WHEN
  `ClientSessionIdentity.session_id` is `None`, THEN the format is
  `pre-session-{counter:06}-{unix_millis}`. The directory name in
  the captured output reflects the new format.

- [x] **AC7 -- Two-client capture does not alias snapshot
  directories**: GIVEN two clients running in parallel each
  emitting at least one snapshot under post-session state, WHEN
  the snapshot directories are listed, THEN their names differ in
  the session_id prefix. The implementing worker captures a smoke-
  level evidence run (two-client launch via PROMPT 1075-style
  smoke harness — out of scope to author here; the worker reuses
  the existing harness) and the captured directory names confirm
  AC7. Where running two clients is not feasible during
  `/dev-story`, the worker MAY satisfy this AC by injecting a
  fixture `ClientSessionIdentity` with two distinct `session_id`
  values into a single-client test bin and asserting the resulting
  directory-name pair.

- [x] **AC8 -- Legacy universal counts not silently lost**: GIVEN
  the post-refactor snapshot JSON, WHEN compared against a
  pre-refactor PROMPT 1022 / 1034 / 1036 snapshot JSON, THEN
  either (a) the universal counts (`hud_entities`,
  `hand_ui_entities`, `shop_auction_entities`) are preserved as
  `#[deprecated]` legacy fields alongside the new per-sub-surface
  fields; OR (b) the universal counts are explicitly removed AND
  a one-line note is added to a release-note document
  (`docs/qa/qa-snapshot-changelog.md` or equivalent — NEW; or to
  the worker's evidence file) explaining the removal. The
  implementing worker chooses and justifies.

- [x] **AC9 -- Integration test covers marker split**: GIVEN
  `tests/integration/ui_clean_pass/qa_snapshot_marker_split_test.rs`
  (NEW), WHEN run, THEN it asserts:
  (a) For each per-sub-surface marker introduced in AC1, spawning
  an entity carrying that marker AND `Visibility::Visible`
  contributes to the corresponding JSON count.
  (b) Spawning an entity carrying the marker AND
  `Visibility::Hidden` does NOT contribute to the count (AC3).
  (c) `pre-session-` prefix fires when `ClientSessionIdentity`
  has no session_id (AC6).
  (d) `{session_id}-` prefix fires when `ClientSessionIdentity`
  has a session_id (AC6).

- [x] **AC10 -- `CCGS_QA_SNAPSHOT=1` env-var contract preserved**:
  GIVEN the existing CCGS_QA_SNAPSHOT button env-var contract
  (Snapshot button surfaced when `CCGS_QA_SNAPSHOT=1` on client
  launch, per the user's memory note
  `reference_ccgs_qa_snapshot_button`), WHEN the post-refactor
  client is launched with `CCGS_QA_SNAPSHOT=1`, THEN the Snapshot
  button continues to surface and the captured snapshot follows
  the new directory-name prefix. The implementing worker confirms
  by reading the existing button-surface code path and asserting
  the env-var check is unchanged.

- [x] **AC11 -- No protocol or server change**: GIVEN
  `git diff <activation HEAD>..HEAD`, WHEN inspected, THEN there
  are zero changes under `server/`, `shared/`, or
  `tests/integration/server/`. The implementation is client-side
  only.

- [x] **AC12 -- ADR-021 schedule preserved**: GIVEN `cargo build
  -p client`, WHEN run under the Cargo resource policy, THEN no
  new system-set or schedule wiring is introduced. Marker spawn
  sites remain in their existing schedule slots; new markers are
  inserted at the same spawn sites alongside (or replacing) the
  universal markers.

- [x] **AC13 -- No accept-risk closure claimed**: GIVEN the
  commit message and any evidence document, WHEN inspected, THEN
  they explicitly do NOT claim closure of `S8-QA-001-W1`,
  `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a`, or any other
  accept-risk disposition. Standard-tier accessibility is NOT
  pursued; playtest validation is NOT pursued; final-art
  replacement is NOT pursued. **Closure of the 24 PROMPT 1022
  findings is explicitly NOT claimed** — this row improves the
  tool that captured them; it does not retest or close them.

- [x] **AC14 -- Sprint 17 disposition preserved**: GIVEN the
  implementation commit(s), WHEN
  `production/sprint-status.yaml`, `production/sprints/sprint-17.md`
  (and earlier), `production/stage.txt`,
  `production/session-state/*`, `production/qa/*`,
  `production/gate-checks/*`, and `docs/architecture/adr-*.md`
  are diffed, THEN none are modified by this story's `/dev-story`
  worker.

- [x] **AC15 -- Worker branch scope contained**: GIVEN the worker
  branch (slug recommendation: `work/s17-qa-snapshot-marker-split`),
  WHEN inspected, THEN it pushes only the worker branch — never
  `main`.

- [x] **AC16 -- Cargo resource policy applied for every Cargo
  command**: future implementation MUST set the Cargo resource
  policy env vars (`CARGO_TARGET_DIR=
  D:\_DEV\cargo-target\ccgs-msvc`, `CARGO_PROFILE_DEV_DEBUG=0`,
  `CARGO_PROFILE_TEST_DEBUG=0`, `CARGO_INCREMENTAL=0`,
  `RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'`) before
  every `cargo check` / `cargo test` invocation on Windows / MSVC.
  Disk preflight (~>= 50 GB free on D:) recorded in the evidence
  file. Story authoring (PROMPT 1095) does NOT invoke Cargo.

---

## Implementation Notes

### Owned files (likely change set)

| Path | Expected change |
|------|-----------------|
| `client/src/presentation/qa_snapshot.rs` | Extend `UiCountQueries` to consume per-sub-surface markers + Visibility filter; extend JSON snapshot output; extend `format_snapshot_id` (or equivalent) with `session_id` prefix. |
| `client/src/ui/hud/` (multiple files; re-verify at activation HEAD) | Introduce per-sub-surface marker components (`HudTopStripRoot`, `HudBottomStripRoot`, `ScoreboardDotRowRoot`); deprecate or remove universal `HudEntity` marker. |
| `client/src/ui/hand/` (multiple files) | Per-sub-surface markers (`HandFanRoot` already exists, add others). |
| `client/src/ui/shop_auction/` (multiple files) | Per-sub-surface markers (`DraftInitialModalRoot`, `ShopPanelRoot`, `AuctionPanelRoot`, `SettlementOverlayRoot`, `ShopFooterRoot`). |
| `client/src/presentation/connection_lost_overlay.rs` | No marker change (marker `ConnectionLostOverlayRoot` already exists); only the qa_snapshot query change. |
| `client/src/presentation/result_screen.rs` | Same as above for `ResultScreenRoot`. |
| `client/src/presentation/board_rendering.rs` | Only touched IF the implementing worker discovers a board sub-surface marker that should be split. Reading-only otherwise. |
| `tests/integration/ui_clean_pass/qa_snapshot_marker_split_test.rs` (NEW) | AC9 integration test. |
| `production/qa/evidence/sprint-17-qa-snapshot-marker-split/evidence.md` (NEW, by `/dev-story` worker) | Evidence document; NOT authored by PROMPT 1095. |
| `docs/qa/qa-snapshot-changelog.md` (optional NEW; or worker's evidence note) | Release note for the snapshot JSON shape change. |

### Forbidden files

- Everything under `server/`, `shared/`.
- Everything under `tests/integration/server/`,
  `tests/unit/server/`, `tests/integration/lightyear*`,
  `tests/unit/lightyear*`.
- `Cargo.toml`, `Cargo.lock`, `.cargo/`, `.github/`, `Trunk.toml`.
- `production/sprint-status.yaml`, `production/sprints/*`,
  `production/stage.txt`, `production/session-state/*`,
  `production/qa/qa-plan-*.md`, `production/qa/smoke-*.md`,
  `production/qa/team-qa-*.md`, `production/gate-checks/*`.
- All other `production/epics/*` story files (no cross-epic edit).
- `docs/architecture/adr-*.md` (no ADR amendment in scope).
- `.claude/`, `AGENTS.md`, `CLAUDE.md`, `CODEX.md`.

### Cargo resource policy

Per the binding Sprint 15+ QA plan precedent, every `cargo`
invocation on Windows / MSVC MUST set the five env vars under AC16.

### Target citations

- Sprint 17 plan row source:
  `production/sprints/sprint-17.md` §"Should Have" row
  `S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001`.
- Source audit:
  `reports/PROMPT-1077-ui-state-source-consistency-deep-audit.md`
  §"Per-finding evidence" SOURCE-1077-08 / 09 / 16.
- Cross-reference: PROMPT 1036 M6 / M7 / M8 / M11 (earlier audit
  iteration that surfaced the same defects).

---

## Worker Contract (for future `/dev-story`)

The future `/dev-story` worker MUST:

1. Run `git checkout` against Sprint 17 activation HEAD on a
   fresh worktree (suggested slug
   `work/s17-qa-snapshot-marker-split`).
2. Read this story file end-to-end before any code change.
3. Re-verify the audit-time module shapes by reading the current
   `qa_snapshot.rs`, hud / hand / shop_auction module markers,
   `connection_lost_overlay.rs`, and `result_screen.rs`. The
   Sprint 17 conditional Must Have rows (modal + shop / auction
   surface paint + helper bundle) may have shifted line numbers
   and may have introduced or removed sub-surface root entities.
4. Pick the final per-sub-surface marker list (informed by the
   PROMPT 1077 reference list AND the current module shape).
5. Pick the JSON field naming convention (preserve existing
   universal counts as `#[deprecated]` legacy fields OR remove
   them outright — document the choice).
6. Activate `liv-bevy-018` skill before any `.rs` edit. Do NOT
   activate `liv-bevy-lightyear`.
7. Set the Cargo resource policy env vars per AC16 before every
   `cargo check` / `cargo test` invocation.
8. Run `cargo check -p client` and the targeted `cargo test -p
   client --test qa_snapshot_marker_split_test` under the Cargo
   resource policy.
9. Push the worker branch (never `main`).
10. Stop. Closure paperwork is later prompts' scope.

The worker MUST NOT:

- Modify `server/`, `shared/`, or anything under
  `tests/integration/server/` / `tests/unit/server/`.
- Modify the snapshot output directory layout outside the
  directory-name prefix.
- Touch the `CCGS_QA_SNAPSHOT=1` env-var contract beyond what AC10
  requires (the contract is preserved; the worker only changes
  the directory name shape downstream).
- Close any of the 24 PROMPT 1022 findings (AC13). Those remain
  preserved as report-only.
- Modify Cargo / Trunk / CI files.
- Modify `production/sprint-status.yaml`, `production/sprints/`,
  `production/stage.txt`, `production/session-state/`,
  `production/qa/qa-plan-*.md`, `production/qa/smoke-*.md`,
  `production/qa/team-qa-*.md`, `production/gate-checks/`.
- Run `/story-done`, `/smoke-check`, `/team-qa`, `/gate-check`,
  `/release-check`, or `/qa-plan` on this story.
- Run the full workspace `cargo test --workspace` invocation.
- Run `trunk` or any CI command.
- Push to `main`.
- Claim closure of any AUDIT-1076-* finding or any SOURCE-1077-*
  finding outside the three bundled here.

### Build gate scope (parallel-agent isolation)

The build gate for this story MUST be scoped to the files this
worker owns plus the new test bin. Because this row spans multiple
sub-surface modules (hud + hand + shop_auction + presentation),
file-overlap risk with other Sprint 17 rows is high; the worker
MUST NOT block on workspace-wide compilation errors introduced by
other in-flight workers' branches. The orchestrator schedules
file-overlap rows serially per the Parallelism summary above.

### Relay / reporting expectation for future workers

Final status line:

```
N: S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001: STATUS
```

where `N` is the prompt number that ran `/dev-story`.

---

## Completion Notes

Closed by PROMPT 1124 `/story-done` paperwork on 2026-05-18 against
source-of-truth `origin/main@74c25b65fbb08da966c2a4e9607812ec34ae610e`
(PROMPT 1123 integration tip `integrate(s17): merge PROMPT 1122
qa-snapshot marker-split into main (PROMPT 1123)` merging PROMPT 1122
worker `f4d0fda dev-story(s17-qa-snapshot-marker-split): per-sub-surface
markers + visibility-aware counts + session-id snapshot prefix
(PROMPT 1122)` onto `origin/main` via `--no-ff` merge, rebased mid-run
onto `origin/main@0cab942` so PROMPT 1109 Vulkan validation gating is
preserved verbatim alongside the marker-split work). `74c25b6` is a
strict fast-forward descendant of `0cab942` (PROMPT 1109 Vulkan
gating integration tip) and of `d35d24d` (PROMPT 1119 bid-button
phase-race integration tip).

### PROMPT 1122 worker + PROMPT 1123 integration outcome

- **Per-sub-surface markers introduced** (AC1):
  - HUD: `HudTopStripRoot`, `HudBottomStripRoot`,
    `HudScoreboardDotRoot`, `HudDimOverlayRoot` added at the
    existing spawn sites in `client/src/ui/hud/mod.rs`.
  - Hand: `HandBarRoot`, `HandDraftGridSlotRoot`,
    `PlacementActionPanelRoot` added at `hand_bar`, every
    `grid_slot`, and `placement_action_panel` in
    `client/src/ui/hand/mod.rs`; pre-existing `HandFanRoot`
    preserved verbatim.
  - Shop / Auction: pre-existing `ShopAuctionPanelRoot` enum
    declared canonical per-sub-surface marker and consumed
    directly by `UiCountQueries`; no new entities, no spawn-site
    relocation.
  - Universal markers `HudEntity` / `HandUiEntity` /
    `ShopAuctionUiEntity` carry
    `#[deprecated(since = "S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001",
    note = …)]` (AC1 + AC8 option (a)). 82 intentional
    deprecation warnings surface at `cargo check -p client`.
- **`UiCountQueries` extended** (AC2 / AC3): per-sub-surface
  visibility queries via `HandVisibilityQueries` +
  `ShopAuctionVisibilityQueries` sub-`SystemParam`s (under the
  16-field ceiling); `UiCounts` gains 17 new `*_visible` fields
  (`hud_root_visible`, `hud_top_strip_visible`,
  `hud_bottom_strip_visible`, `hud_scoreboard_dot_visible`,
  `hud_dim_overlay_visible`, `hand_bar_visible`,
  `hand_fan_visible`, `hand_draft_grid_slot_visible`,
  `placement_action_panel_visible`, `shop_draft_offering_visible`,
  `shop_panel_visible`, `auction_panel_visible`,
  `shop_footer_visible`, `auction_toast_visible`,
  `settlement_overlay_visible`, `connection_lost_overlay_visible`,
  `result_screen_visible`). `is_visibility_visible` (true when
  `Visibility != Hidden`) is applied per-count.
- **`format_snapshot_id` extended** (AC6 / AC7): new signature
  `format_snapshot_id(counter, unix_millis, session_id:
  Option<u64>)`; post-handshake emits
  `{session_id}-{counter:06}-{unix_millis}`; before handshake
  emits `pre-session-{counter:06}-{unix_millis}` via new
  `QA_SNAPSHOT_PRE_SESSION_PREFIX = "pre-session"` constant.
  `short_id` updated to surface the counter token under the new
  format.
- **`CCGS_QA_SNAPSHOT=1` env-var contract preserved verbatim**
  (AC10): `QA_SNAPSHOT_ENV_VAR` constant unchanged;
  `from_env_values` behaviour unchanged. Snapshot button surface
  path untouched.
- **Legacy universal counts retained as `#[deprecated]`** (AC8
  option (a)): `hud_entities`, `hand_ui_entities`,
  `shop_auction_entities`, `connection_lost_overlay_roots`,
  `result_screen_roots` populated alongside the new `*_visible`
  fields so PROMPT 1022 / 1034 / 1036 historical snapshot
  comparisons resolve.

### Test evidence

- `tests/integration/ui_clean_pass/qa_snapshot_marker_split_test.rs`
  (NEW; AC9) -- **11/11 PASS** at the integration tip
  (`74c25b6`):
  - `each_per_sub_surface_marker_with_visible_visibility_contributes_to_count`
  - `hidden_visibility_excludes_marker_from_per_sub_surface_counts`
  - `inherited_visibility_counts_as_visible`
  - `repeated_marker_spawns_accumulate_into_visible_count`
  - `connection_lost_overlay_visible_honours_own_visibility`
  - `result_screen_visible_honours_own_visibility`
  - `legacy_universal_counts_remain_populated_alongside_new_fields`
  - `pre_session_prefix_used_when_session_id_is_none`
  - `session_id_prefix_used_when_session_id_is_some`
  - `two_clients_with_distinct_session_ids_do_not_alias`
  - `ccgs_qa_snapshot_env_contract_preserved`
- `tests/integration/qa_snapshot/qa_snapshot_overlay_test.rs`
  (existing bin, extended with `..UiCounts::default()` + worker-
  side `#[allow(deprecated)]`) -- **20/20 PASS** at integration
  tip.
- Adjacent surface focused bins at integration tip (build-gate
  isolation per story §"Build gate scope") all PASS:
  `hud_plugin_scaffold_test 4/4`,
  `hand_ui_plugin_scaffold_test 3/3`,
  `hud_top_strip_layout_test 8/8`,
  `hud_bottom_strip_layout_test 8/8`,
  `shop_auction_ui_shop_panel_test 10/10`,
  `shop_auction_ui_auction_activation_test 8/8`,
  `shop_auction_ui_draft_initial_grid_test 10/10`,
  `hand_fan_root_b0004_hierarchy_test 1/1`.
- `cargo check -p client` -- PASS at integration tip
  (`Finished \`dev\` profile [optimized]`, 82 intentional
  deprecation warnings, zero errors, zero new non-deprecation
  warnings).
- `git diff --check origin/main...HEAD` at integration tip --
  clean.
- Evidence file:
  `production/qa/evidence/sprint-17-qa-snapshot-marker-split/evidence.md`.
- Worker report: `reports/PROMPT-1122-s17-ui-qa-snapshot-marker-split.md`.
- Integration report:
  `reports/PROMPT-1123-s17-qa-snapshot-marker-split-integration.md`.

### Cargo resource policy (AC16)

- **PROMPT 1122 worker** applied all 5 Cargo resource policy env
  vars (`CARGO_TARGET_DIR=D:\_DEV\cargo-target\ccgs-msvc` +
  `CARGO_PROFILE_DEV_DEBUG=0` + `CARGO_PROFILE_TEST_DEBUG=0` +
  `CARGO_INCREMENTAL=0` + `RUSTFLAGS='-C debuginfo=0 -C
  link-arg=/DEBUG:NONE'`) before every cargo invocation; D: free
  space ≈ 772 GB at worker session start.
- **PROMPT 1123 integration** encountered a PowerShell / Bash
  env-var propagation gap on the very first `cargo check -p
  client` invocation (`powershell -NoProfile -Command` wrapper
  ate the `$env:VAR=...` block at the bash shell layer); cargo
  built into the worktree-local `target/` directory once
  (~1m42s, `[optimized + debuginfo]`). All subsequent cargo
  invocations used the correct env vars (visible via
  `[optimized]` profile note and the shared
  `D:\_DEV\cargo-target\ccgs-msvc\` build path). Worktree-local
  `target/` is self-contained and cleaned with the worktree.
  Recorded explicitly as a process / policy advisory note in
  PROMPT 1123's integration report and in this Completion Notes
  section -- NOT hidden. D: free remained > 718 GB throughout;
  build correctness gate unaffected (all 87/87 targeted sub-
  tests across 10 bins + 2 cargo check invocations PASS at
  integration tip). PROMPT 1124 itself does NOT invoke Cargo
  (paperwork-only closure).

### Per-AC outcome

- AC1 universal markers split into per-sub-surface markers --
  **PASS**. New HUD / Hand markers added; `ShopAuctionPanelRoot`
  declared canonical per-sub-surface marker; universal markers
  carry `#[deprecated(since = "S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001",
  …)]`. Asserted via
  `each_per_sub_surface_marker_with_visible_visibility_contributes_to_count`.
- AC2 `UiCountQueries` consumes per-sub-surface markers + JSON
  emits per-sub-surface counts -- **PASS**. 17 new `*_visible`
  fields on `UiCounts`. Asserted via
  `each_per_sub_surface_marker_with_visible_visibility_contributes_to_count`
  + `repeated_marker_spawns_accumulate_into_visible_count`.
- AC3 visibility filter in `UiCountQueries.snapshot()` --
  **PASS**. `is_visibility_visible` applied per-count. Asserted
  via `hidden_visibility_excludes_marker_from_per_sub_surface_counts`
  + `inherited_visibility_counts_as_visible`.
- AC4 connection-lost overlay visible flag honours Visibility --
  **PASS**. New `connection_lost_overlay_visible` field; legacy
  `connection_lost_overlay_roots` retained. Asserted via
  `connection_lost_overlay_visible_honours_own_visibility`.
- AC5 result-screen overlay visible flag honours Visibility --
  **PASS**. New `result_screen_visible` field; legacy
  `result_screen_roots` retained. Asserted via
  `result_screen_visible_honours_own_visibility`.
- AC6 snapshot ID prefix includes session_id / `pre-session-` --
  **PASS**. `format_snapshot_id` signature accepts
  `session_id: Option<u64>`. Asserted via
  `pre_session_prefix_used_when_session_id_is_none` +
  `session_id_prefix_used_when_session_id_is_some`.
- AC7 two-client capture does not alias snapshot directories --
  **PASS**. Worker satisfied AC7 via the worker-allowable
  fallback (fixture-injected `ClientSessionIdentity` with two
  distinct `session_id` values in a single-client test bin).
  Asserted via `two_clients_with_distinct_session_ids_do_not_alias`.
- AC8 legacy universal counts preserved as `#[deprecated]` --
  **PASS (option (a))**. `hud_entities`, `hand_ui_entities`,
  `shop_auction_entities`, `connection_lost_overlay_roots`,
  `result_screen_roots` carry deprecation attributes and remain
  populated. Asserted via
  `legacy_universal_counts_remain_populated_alongside_new_fields`.
- AC9 integration test covers marker split -- **PASS**. NEW
  `tests/integration/ui_clean_pass/qa_snapshot_marker_split_test.rs`;
  11/11 PASS at integration tip; registered as
  `[[test]] name = "ui_clean_pass_qa_snapshot_marker_split_test"`
  in `client/Cargo.toml`.
- AC10 `CCGS_QA_SNAPSHOT=1` env-var contract preserved --
  **PASS**. `QA_SNAPSHOT_ENV_VAR` constant + `from_env_values`
  behaviour unchanged. Asserted via
  `ccgs_qa_snapshot_env_contract_preserved`.
- AC11 no protocol or server change -- **PASS**. PROMPT 1123
  integration diff vs `origin/main` touched 8 paths exactly:
  `client/Cargo.toml` (test-registration-only),
  `client/src/presentation/qa_snapshot.rs`,
  `client/src/ui/hand/mod.rs`, `client/src/ui/hud/mod.rs`,
  `client/src/ui/shop_auction/mod.rs`,
  `production/qa/evidence/sprint-17-qa-snapshot-marker-split/evidence.md`,
  `tests/integration/qa_snapshot/qa_snapshot_overlay_test.rs`,
  and the new test bin. Zero changes under `server/`, `shared/`,
  `tests/integration/server/`, `tests/unit/server/`.
- AC12 ADR-021 schedule preserved -- **PASS**. No new system set,
  no schedule wiring, no spawn-site relocation; new markers
  inserted alongside existing universal markers on the same
  entities. `cargo check -p client` PASS at integration tip.
- AC13 no accept-risk closure -- **PASS**. PROMPT 1122 worker
  commit + evidence.md + PROMPT 1123 integration merge commit +
  this PROMPT 1124 paperwork all explicitly preserve
  `S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a`,
  `TQ-S12-C1..C7`, PROMPT 761 `Polish->Release` FAIL,
  `S11-HUD-TIMER-EYEBALL-VISUAL-001` carry, all `AUDIT-1076-*`
  findings, all `SOURCE-1077-*` findings outside SOURCE-1077-08 /
  09 / 16, the 24 PROMPT 1022 audit findings, and the PROMPT
  1112 AC3 hand reserve-strip carry (preserved OPEN; this row
  does NOT close it).
- AC14 Sprint 17 disposition preserved by worker + integration
  -- **PASS**. PROMPT 1122 worker + PROMPT 1123 integration
  diffs touched zero files under `production/sprint-status.yaml`,
  `production/sprints/sprint-17.md`, `production/stage.txt`,
  `production/session-state/*`, `production/qa/qa-plan-sprint-17.md`,
  `production/qa/smoke-*.md`, `production/qa/team-qa-*.md`,
  `production/gate-checks/*`, `docs/architecture/adr-*.md`.
  PROMPT 1124 is the first authorised modifier of
  `production/sprint-status.yaml` + `production/session-state/*`
  for this row.
- AC15 worker branch scope contained -- **PASS**. PROMPT 1122
  worker pushed `work/s17-qa-snapshot-marker-split` (`f4d0fda`)
  only -- never `main`. PROMPT 1123 integration onto
  `origin/main` performed separately via
  `integrate/s17-qa-snapshot-marker-split-1123` -> `74c25b6`.
- AC16 Cargo resource policy applied -- **PASS-WORKER +
  ADVISORY-INTEGRATION**. PROMPT 1122 worker applied all 5 env
  vars before every cargo invocation; PROMPT 1123 integration
  encountered a one-call env-var propagation gap on the first
  `cargo check`; recorded explicitly above and in PROMPT 1123's
  integration report. PROMPT 1124 itself does NOT invoke Cargo.

### External / baseline drift (advisory; out of scope)

`shop_auction_ui_plugin_scaffold_formulas_test::shop_auction_ui_prepooled_panel_roots_are_bevy_ui_nodes`
fails with `left: 87, right: 82` on the unmodified
`origin/main@5345164` baseline before any PROMPT 1122 edit. The
hand-tuned arithmetic in
`tests/unit/shop_auction_ui/plugin_scaffold_formulas_test.rs:50`
drifted from the actual `ShopAuctionUiEntity`-tagged spawn count
introduced by an upstream landing. Per the story §"Build gate
scope" -- *only owned files and directly related tests* -- this
row does not block on unrelated in-flight worker drift. PROMPT
1122 makes no edit to that formula; PROMPT 1123 integration
preserved the drift verbatim; PROMPT 1124 paperwork does NOT
silently fix the counts and does NOT advance closure on the
drift. Recommend a separate follow-up story for counter
reconciliation against the post-marker-split spawn tree.

---

## Closure Trail

### Closure trail (commits)

1. **PROMPT 1095** -- net-new Sprint 17 story authoring batch
   (story 019 drafted; integrated by PROMPT 1097 paperwork-only
   main integration into `origin/main@bc3db29`).
2. **PROMPT 1097** -- paperwork-only main integration of the
   Sprint 17 story authoring batch (`bc3db29`).
3. **PROMPT 1099** -- Sprint 17 activation (`cb62a9e`).
4. **PROMPT 1100** -- `/qa-plan sprint-17` authoring (`ff47075`).
5. **PROMPT 1122** -- `/dev-story` worker
   (`f4d0fda dev-story(s17-qa-snapshot-marker-split):
   per-sub-surface markers + visibility-aware counts + session-id
   snapshot prefix (PROMPT 1122)`) on branch
   `work/s17-qa-snapshot-marker-split`.
6. **PROMPT 1123** -- integration of PROMPT 1122 onto
   `origin/main` via `--no-ff` merge
   (`74c25b6 integrate(s17): merge PROMPT 1122 qa-snapshot
   marker-split into main (PROMPT 1123)`) on branch
   `integrate/s17-qa-snapshot-marker-split-1123`; rebased mid-run
   onto `origin/main@0cab942` (PROMPT 1109 Vulkan validation
   gating integration tip) when concurrent integration landed,
   so `wgpu-validation` feature + S17-OPS comment block at
   `client/Cargo.toml:34-40` are preserved verbatim alongside
   the marker-split test registration.
7. **PROMPT 1124** -- this `/story-done` paperwork (flips AC1
   .. AC16 to `[x]`; appends Completion Notes; records closure
   tip; updates `production/sprint-status.yaml`,
   `production/session-state/active.md`,
   `production/session-state/codex-orchestrator-state.md`).

### Conditions carried forward unchanged

- Sprint 16 disposition `closed-with-conditions` (UNCHANGED).
- Sprint 17 stage `Polish` (UNCHANGED).
- PROMPT 761 Polish->Release gate-check `FAIL` preserved.
- `S8-QA-001-W1` OPEN preserved.
- `QA-COND-0005` + `QA-COND-0006` accepted-risk preserved.
- `PAW-TD-*-a` placeholder-art accept-risk preserved.
- `TQ-S12-C1..C7` preserved verbatim.
- Sprint 15 / 14 / 13 / 12 / 11 / 10 dispositions preserved
  unchanged.
- HUD timer row `S11-HUD-TIMER-EYEBALL-VISUAL-001` human-operator-
  blocked carry preserved; NOT closed by this row.
- 24 PROMPT 1022 audit findings preserved as report-only; NOT
  closed by this row.
- PROMPT 1108 (S17-SERVER-START-OF-TURN-DEBUG-001), PROMPT 1110
  (S17-UI-CARD-SLOT-INSET-WIRING-001), PROMPT 1112
  (S17-UI-HUD-OPP-MANA-CLEANUP-001 PARTIAL disposition with AC3
  carried), PROMPT 1117 (S17-UI-CARD-DISPLAY-ART-HELPER-001),
  PROMPT 1120 (S17-UI-HAND-B0004-CLEANUP-001), and PROMPT 1121
  (S17-UI-BID-BUTTON-PHASE-RACE-001) closures all preserved
  verbatim above this PROMPT 1124 entry.
- PROMPT 1109 Vulkan validation gating (`wgpu-validation`
  feature in `client/Cargo.toml:34-40` and `request_features`
  integration in `client/src/main.rs`) preserved verbatim by
  PROMPT 1123 mid-run rebase onto `origin/main@0cab942`.
- PROMPT 1112 AC3 hand reserve-strip carry remains OPEN; this
  row does NOT close it.
- SOURCE-1077-08 + SOURCE-1077-09 + SOURCE-1077-16 discharged on
  `origin/main` by PROMPT 1123 integration; the seven remaining
  un-bundled SOURCE-1077-* findings (05 / 07 / 11 / 12 / 13 /
  14 / 15) remain deferred to Sprint 18+. SOURCE-1077-01/02/03/04
  discharged by PROMPT 1114/1117; SOURCE-1077-06 by PROMPT
  1106/1110; SOURCE-1077-10 by PROMPT 1119/1121.
- All AUDIT-1076-* findings preserved as open / report-only
  outside AUDIT-1076-14 (discharged PROMPT 1118/1120),
  AUDIT-1076-15 (discharged PROMPT 1107/1108), AUDIT-1076-18
  (discharged PROMPT 1103/1109), and AUDIT-1076-10 +
  AUDIT-1076-16 (discharged PROMPT 1111; AUDIT-1076-17 remains
  OPEN carried with AC3 of `S17-UI-HUD-OPP-MANA-CLEANUP-001`).
- External `shop_auction_ui_plugin_scaffold_formulas_test::shop_auction_ui_prepooled_panel_roots_are_bevy_ui_nodes`
  baseline drift (`87 vs 82`) preserved verbatim by PROMPT 1123
  integration; not silently fixed by PROMPT 1124 paperwork.

### Explicitly NOT claimed by this story or its `/dev-story` worker

- Closure of any of the 24 PROMPT 1022 audit findings (this row
  improves the tool that captured them; it does not retest or
  close them).
- Closure of any AUDIT-1076-* finding.
- Closure of any SOURCE-1077-* finding outside SOURCE-1077-08 / 09
  / 16.
- Sprint 17 close-out.
- Public release readiness; release-candidate readiness; full
  game completion.
- Broad / Standard-tier accessibility completion; playtest /
  fun-hypothesis validation; full playable-client manual QA;
  two-client GAME_OVER closure; final-art completion;
  Polish->Release gate-check retry; stage advance.

`019: S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001: DONE`
