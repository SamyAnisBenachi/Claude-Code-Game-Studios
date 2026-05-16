# Story 025: S11-UX-LOBBY-CLASS-PICKER -- Lobby Class-Picker Layout & Hierarchy

> **Epic**: Playable Client
> **Story ID**: S11-UX-LOBBY-CLASS-PICKER
> **Status**: Done -- Sprint 14 /story-done closure by PROMPT 962 on `origin/main@fed5fb9be135db274310c363151a073056927b92`
> **Layer**: Lobby UI / UX (Client)
> **Type**: Integration -- targeted lobby UI composition edit (class-picker region: portrait row + button row + class label hierarchy) + viewport-invariant + visual capture
> **Sprint**: Sprint 14 candidate (per `docs/ux/ui-clean-pass-roadmap.md` rank 11; PROMPT 802 §6 Lane C; PROMPT 685 row 5 class-picker slice, `subsumed-by` PROMPT 802 §3.1 L2 + L3, §4 Tier 1.8). **NOT** activated by this authoring run.
> **Authored**: 2026-05-14 by PROMPT 880
> **Authoring source-of-truth**: `origin/main@51e6228` (PROMPT 871 `qa(s13): /story-done S13-TWO-CLIENT-RUNTIME-HARNESS-001`)
> **Authoring worktree**: `D:\_DEV\claude-code-game-studios-worktrees\s14-lobby-layout-story-authoring`
> **Authoring branch**: `story/s14-lobby-layout-story-authoring`

---

## Status / No-Claim Banner

This story is authored as a **Sprint 14 candidate** by PROMPT 880. Sprint
14 is **NOT** activated by PROMPT 880. Sprint 13 disposition (`active` per
PROMPT 826) is preserved unchanged. Sprint 12 disposition
(`closed-with-conditions` per PROMPT 817) is preserved unchanged. Sprint
11 / Sprint 10 closeouts preserved unchanged.

PROMPT 880 (this authoring run) does **NOT**:

- Activate Sprint 14.
- Modify `production/sprint-status.yaml`.
- Modify any file under `production/sprints/`, `production/qa/`,
  `production/session-state/`, or `production/stage.txt`.
- Modify or retry the PROMPT 761 `Polish->Release` gate-check artifact.
- Modify any code under `client/`, `server/`, `shared/`, or `tests/`.
- Run `/dev-story`, `/story-readiness`, `/story-done`, `/smoke-check`,
  `/team-qa`, `/gate-check`, `/release-check`, or `/qa-plan`.
- Invoke any `cargo` / `trunk` build or test command.
- Pull this row or any of the 14 PROMPT 802 candidate slugs into any
  active sprint.

This story does **not** claim: public release readiness, release-candidate
readiness, full game completion, broad / Standard-tier accessibility
completion (`QA-COND-0005`), playtest / fun-hypothesis validation
(`QA-COND-0006`), full playable-client manual QA, two-client `GAME_OVER`
closure (`S8-QA-001-W1`), final-art / asset-production completion
(`PAW-TD-*-a`), or `Polish->Release` gate-check retry. PROMPT 761
`Polish->Release` `FAIL` evidence preserved.

**Friend-game scope only.** Per the friend-game-scope vs Standard-tier-
accessibility scope boundary in `docs/ux/ui-clean-pass-roadmap.md`, this
story is friend-game visual polish only. It does **not** advance
`QA-COND-0005` and does **not** commit to Standard-tier accessibility
conformance.

**ADR-002 binding preserved**: server-authoritative client-server
authority is unchanged. Class-lock state remains server-authoritative
via `S2CClassLocked`. This story is composition / hierarchy /
typography only. No new client-side state machine, no client-side
optimistic authority, no protocol change, no server-side change.

---

## Source Finding

- PROMPT 802 audit §3.1 L2: "Class-picker is a `lobby_wrap_row_node()`
  of 7 buttons at 92px each — wraps at ~3-4 per row inside a 420-wide
  column. Not a grid, no visual class affordance."
  (`client/src/ui/lobby.rs:935-955`).
- PROMPT 802 audit §3.1 L3: "Class portraits row is a *separate
  sibling* below the class buttons row — portraits and selectable
  class buttons are not visually associated."
  (`client/src/ui/lobby.rs:977-990`).
- PROMPT 802 audit §3.1 L6 (orthogonal but adjacent): inverted
  typography hierarchy — "Class" label `13px` is smaller than status
  text `18px`; class buttons `13px` are smaller than CTA buttons.
  Friend-game-tier acceptable but flagged so it does not silently
  regress.
- `docs/ux/ui-clean-pass-roadmap.md` ranks this as
  `S11-UX-LOBBY-CLASS-PICKER` rank 11 (Tier 1, Must, 1.0d).
- PROMPT 685 row 5 partial: `S11-UX-LOBBY-CLASS-PICKER` was named as
  an 8-story UI-clean-pass milestone row in 2026-05-11 and was never
  authored into a story file. This story authors that row, with a
  re-validation note that the PROMPT 685 disposition remains
  `subsumed-by` PROMPT 802 §3.1 L2 + L3, §4 Tier 1.8.

---

## Problem Class / Prevention Target

**Defect class**: the class-picker region in the lobby is composed
as three independent siblings — a `"Class"` label (`13 Px`), a wrap-row
of 7 selectable class buttons (each `92 Px` wide at `13 Px` font), and
a wrap-row of 7 class portraits (`64×80 Px`) — with no visual
hierarchy, no spatial association between portrait and button for the
same `ClassId`, and a wrap pattern that produces 3-4 buttons per row
inside the parent 420-Px column with no grid alignment.

Symptom: the player cannot tell which portrait corresponds to which
class button (especially for classes whose name and silhouette do not
trivially map, e.g. Sram / Ecaflip / Sang Méprise). The two rows wrap
independently of each other.

**Prevention target**: compose the class-picker region as a single
hierarchical block:

1. A single `"Class"` heading that sets the region scope.
2. A class-picker grid where each cell pairs a portrait + a label /
   button for the same `ClassId`, so portrait and selection control
   sit in the same visual unit.
3. Predictable grid columns (e.g. 4-column at narrow viewports,
   7-column at wide viewports if the parent has the room) so the
   portrait↔button pairing does not wrap apart.
4. Stable cell dimensions across rebuild (paired with story 026's
   button-dimension invariant).
5. Clear selection affordance: the currently-selected `ClassId`'s cell
   is visually highlighted (border, background, or scale tweak — final
   visual language owned by `ux-designer` consultation per AC8).

This story does **not** add a new selection mechanism, a new class, or
a new client-side state. The existing `LobbyClassButton { class_id }`
interaction and `LobbyClassPortrait { class_id }` markers are preserved.

---

## Context

### Existing surface (read-only at authoring time)

- **`client/src/ui/lobby.rs:935-955`** (class button wrap row): current
  shape is `lobby_wrap_row_node()` parent with 7 `LobbyClassButton`
  children, each `lobby_button_node(Val::Px(92.0))` at `lobby_text_font
  (13.0)`. Sourced from `lobby_class_options()`.
- **`client/src/ui/lobby.rs:977-990`** (portrait wrap row): current
  shape is `lobby_wrap_row_node()` parent with 7 `LobbyClassPortrait`
  children, each `Node { width: 64.0 Px, height: 80.0 Px, .. }` with an
  `ImageNode` loading `lobby_portrait_asset(class_id)`. Sourced from
  `lobby_all_class_ids()` (includes `Neutral`; class buttons use
  `lobby_class_options()` which may not). The two iterators MUST be
  reconciled by the implementing prompt if the cell pairing assumes
  1-to-1 correspondence (see AC2).
- **`client/src/ui/lobby.rs:935`** (`"Class"` label): `Text::new("Class
  "), lobby_text_font(13.0)`.
- **`lobby_class_options()` / `lobby_all_class_ids()`**: existing
  helpers in `client/src/ui/lobby.rs`. The implementing prompt MUST
  reconcile their ordering and `Neutral` handling (read-only at
  authoring; the reconciliation may produce a single shared iterator
  or a documented diff). No client-side authority is added by
  reconciliation.
- **`shared::card::ClassId`**: class enumeration. **Not modified** by
  this story.

### GDD / ADR / TR trace

- **GDD**: `design/gdd/game-session-system.md` (lobby flow and class
  selection); `design/gdd/hand-ui.md` (UX text owned by ux-designer).
- **ADR-002** (Client-Server Authority): no client-side class-lock
  authority added.
- **ADR-008** (Lightyear Channel Configuration): no channel change.
- **ADR-012** (SessionReady Delivery): no change.
- **ADR-021** (Presentation Layer Architecture): preserved.
- **TR registry**: no new TR (UX composition only).

### Engine / skills

- **Engine**: Bevy 0.18 (Rust).
- **Mandatory skills**: `liv-bevy-018` (any `.rs` edit in the lobby UI
  module).
- **Mandatory skills**: `liv-bevy-lightyear` only if the touched code
  imports `lightyear` directly (the class-picker composition does
  not).

### Control Manifest Rules

- Required: class portraits and class selection controls for the same
  `ClassId` are visually paired (same cell, adjacent cells, or shared
  background — exact pairing pattern owned by ux-designer
  consultation per AC8).
- Required: class-picker grid columns are predictable across resize;
  the portrait↔button pairing does not wrap apart.
- Required: typography hierarchy in the class-picker region is not
  inverted — the `"Class"` heading sits at or above the per-cell
  label / button font size (ux-designer locks the final literal).
- Required: no overlap between class-picker cells at `1366×768` or
  `1920×1080`.
- Required: text fits inside each class-button cell at the canonical
  font size; if the canonical font size does not accommodate the
  longest class name, the implementing prompt either (a) increases
  the cell width with ux-designer sign-off, (b) reduces the font size
  with ux-designer sign-off (subject to friend-game-tier readability),
  or (c) wraps the label to two lines with ux-designer sign-off. No
  silent ellipsis insertion.
- Required: selection affordance for the currently-selected `ClassId`
  cell is visually distinct from non-selected cells.
- Forbidden: new client-side class-lock authority. Class-lock state
  remains server-authoritative via `S2CClassLocked`.
- Forbidden: edits to `shared/`, `server/`, `tests/`,
  `production/sprint-status.yaml`, `production/sprints/`,
  `production/qa/`, `production/session-state/`,
  `production/stage.txt`, the PROMPT 761 gate-check artifact, or any
  `Cargo` file.
- Forbidden: claiming Standard-tier accessibility conformance,
  claiming `QA-COND-0005` advanced, claiming `PAW-TD-*-a` resolved.

---

## Story Classification

**Story type**: Integration -- targeted lobby UI composition edit
(class-picker region only) + viewport-invariant test + visual
capture. Friend-game scope only.

This is **NOT** a:

- Pure UX-spec story (real client UI lands).
- Server-side change.
- Protocol change.
- Accessibility-tier (Standard-tier) repair.
- Final-art asset wiring (portrait assets owned by `PAW-006`).

---

## Acceptance Criteria

All criteria are independently checkable.

- [x] **AC1 -- Single hierarchical class-picker block**: GIVEN
  `client/src/ui/lobby.rs` `spawn_lobby_ui_system` after the
  implementation, WHEN inspected, THEN the class-picker region is
  composed as a single hierarchical block (one `"Class"` heading +
  one grid or aligned-row composition pairing portraits with
  selection controls for the same `ClassId`). The current three-
  independent-siblings shape (label + button wrap-row + portrait
  wrap-row) is replaced.

- [x] **AC2 -- Portrait↔button pairing**: GIVEN the class-picker
  region rendered at runtime, WHEN inspected, THEN for each `ClassId`
  in `lobby_class_options()`, the portrait image and the selectable
  button are visually paired (same cell, adjacent cells, or shared
  background) such that a player cannot reasonably mis-identify which
  portrait belongs to which class. The implementing prompt MUST
  reconcile any difference between `lobby_class_options()` and
  `lobby_all_class_ids()` (`Neutral` handling) and record the
  reconciliation in the evidence document.

- [x] **AC3 -- Predictable grid columns**: GIVEN the class-picker
  region, WHEN rendered at `1920×1080` and at `1366×768`, THEN the
  grid columns are predictable (e.g. 4-column or 7-column;
  implementing-prompt-locked with ux-designer sign-off) and do not
  wrap portrait-row vs button-row apart. Cells of the same row align
  vertically.

- [x] **AC4 -- No overlap, text fit**: GIVEN the class-picker region
  at either viewport, WHEN inspected, THEN (a) no cell overlaps an
  adjacent cell; (b) the class name text fits inside its parent cell
  at the canonical font size without silent ellipsis insertion; if
  the canonical font size does not accommodate the longest class
  name, AC4 is satisfied only via cell width increase, font-size
  decrease, or two-line wrap (ux-designer sign-off required for
  whichever option).

- [x] **AC5 -- Stable cell dimensions across rebuild**: GIVEN repeat
  spawns of the lobby root (e.g. exit-and-re-enter `ClientState::
  Lobby`), WHEN class-picker cell widths and heights are sampled
  across the rebuild, THEN they match the canonical dimensions
  declared by this story (or by paired story 026 if applied to the
  class buttons) within 1 Px tolerance.

- [x] **AC6 -- Selection affordance**: GIVEN the class-picker region,
  WHEN the player's `LobbyInputState.selected_class` is set to a
  given `ClassId`, THEN that `ClassId`'s cell is visually distinct
  from non-selected cells (border, background, scale, or other ux-
  designer-locked visual language). The selection affordance
  re-renders on `LobbyInputState` change without requiring a full
  spawn.

- [x] **AC7 -- Viewport-invariant test**: GIVEN a new or extended
  test under `tests/integration/playable_client/` (canonical path
  chosen by the implementing prompt), WHEN the test asserts that
  class-picker cells fit within `1366×768` and `1920×1080` viewports
  without overflow and without portrait↔button row-wrap divergence,
  THEN the test passes. Foundational viewport-invariant test bin
  authoring is owned by `S11-TD-UI-VIEWPORT-INVARIANT-TESTS`; this
  story may consume that bin or author standalone.

- [x] **AC8 -- ux-designer consultation recorded**: GIVEN the
  implementation prompt's first ux-designer interaction, WHEN the
  final pairing pattern (cell shape, selection affordance literal,
  grid column count, font sizes if changed) is locked, THEN the
  consultation note and chosen literals are recorded in the evidence
  document. Friend-game-tier literals are acceptable; Standard-tier
  accessibility conformance is **not** claimed.

- [x] **AC9 -- No client-side class-lock authority (ADR-002)**:
  GIVEN the implementation diff, WHEN reviewed, THEN no client-side
  mutation of class-lock state is introduced outside the existing
  `S2CClassLocked` drain path. The class-button interaction handler
  still emits `LobbyCommand::ConfirmClass`-equivalent intents; it
  does not synthesise `S2CClassLocked` locally. No protocol shape
  change in `shared/src/protocol.rs`. No server-side change.

- [x] **AC10 -- Workspace test pass**: GIVEN
  `cargo test --workspace --tests --no-fail-fast` at the
  implementation commit, WHEN compared to the post-Sprint-13
  baseline, THEN no new `#[ignore]` markers are introduced; the new
  viewport-invariant test passes; previously-passing tests continue
  to pass.

- [x] **AC11 -- No `production/` shared-tracker edits**: GIVEN the
  implementation commit, WHEN `production/sprint-status.yaml`,
  `production/sprints/`, `production/qa/`, `production/stage.txt`,
  `production/session-state/`, and the PROMPT 761 gate-check
  artifact are diffed, THEN none is modified by this story's
  implementing prompt except in the `/story-done` paperwork commit.

- [x] **AC12 -- Friend-game-scope no-claim restated in evidence**:
  GIVEN the evidence document, WHEN read at the bottom, THEN it
  verbatim restates the friend-game-scope-only disposition from the
  Status / No-Claim Banner: no public release readiness, no
  Standard-tier accessibility completion, no `QA-COND-0005`
  advancement, no `QA-COND-0006` advancement, no `S8-QA-001-W1`
  closure, no `PAW-TD-*-a` resolution, no `Polish->Release`
  gate-check retry.

---

## Likely Files

| Path | Anticipated change |
|------|--------------------|
| `client/src/ui/lobby.rs` (canonical path verified by implementing worker) | Edited: class-picker region (current lines 935-990) replaced with a single hierarchical block pairing portraits and selectable buttons per `ClassId`. `LobbyClassButton` / `LobbyClassPortrait` marker components preserved. |
| `tests/integration/playable_client/lobby_class_picker_layout_test.rs` (or canonical equivalent under the rank-4 viewport-invariant test bin) | NEW integration test asserting AC3 / AC4 / AC7. |
| `production/qa/evidence/sprint-14-lobby-class-picker-evidence.md` (slot reserved; sprint number may be adjusted at activation time) | NEW evidence document: ux-designer consultation (AC8), 1920×1080 + 1366×768 captures (AC4 / AC7), viewport-invariant test pass output, pairing pattern rationale, no-claim restatement. |
| This story file | Status flip Draft -> Implemented / Done on `/story-done` paperwork. |
| `production/epics/playable-client/EPIC.md` | Status flip on `/story-done` paperwork. |

This table is a planning estimate. The implementing prompt is
authoritative for the realised set.

---

## Required Skills

- **`liv-bevy-018`** -- mandatory for `.rs` edits to `client/src/ui/lobby.rs`.
- **`liv-bevy-lightyear`** -- only if the implementation touches a
  `MessageReceiver<T>` / `MessageSender<T>` drain in the lobby
  module.
- **`ux-designer` agent** -- mandatory consultation per AC8.
- **`art-director` agent** -- consultation recommended on portrait /
  cell visual treatment if portrait sizing changes.

---

## Evidence Path

`production/qa/evidence/sprint-14-lobby-class-picker-evidence.md`
(slot reserved; sprint number may be adjusted at activation time;
populated by the implementing prompt).

**Required evidence content**:

- Diff summary for `client/src/ui/lobby.rs` (class-picker region
  only).
- ux-designer consultation note (AC8) with chosen pairing pattern,
  selection affordance, grid column count, font sizes.
- `lobby_class_options()` vs `lobby_all_class_ids()` reconciliation
  note (AC2).
- New viewport-invariant integration-test pass output (AC7 / AC10).
- 1920×1080 visual capture (AC3) and 1366×768 visual capture (AC4),
  saved under
  `production/qa/evidence/captures/sprint-14-lobby-class-picker/`.
- Selection affordance capture (AC6).
- Cross-link to paired stories 024 (`S12-UX-LOBBY-LAYOUT-MODAL-001`)
  and 026 (`S11-UX-LOBBY-BUTTON-HITTARGETS`).
- Cross-link to `docs/ux/ui-clean-pass-roadmap.md` (rank 11) and to
  PROMPT 685 row 5 + PROMPT 802 §3.1 L2 / L3.
- Verbatim no-claim restatement (AC12).
- ADR-002 / ADR-008 / ADR-012 / ADR-021 preservation note (AC9).

---

## Regression Commands Expected

For the implementing prompt (not run by PROMPT 880):

- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets`
- `cargo test --workspace --tests --no-fail-fast`
- `cargo test -p client --test lobby_class_picker_layout -- --nocapture`
  (or the new test name)
- `git diff <pre-impl-sha>..<impl-sha> -- 'shared/src/**' 'server/src/**'`
  (verifies AC9: zero protocol-shape change, zero server-side change)
- `git diff --check origin/main...HEAD`
- `git diff --cached --check`

---

## Dependency Notes Against Sprint 14 Pull-In Sequence

Per `docs/ux/ui-clean-pass-roadmap.md` rank-11 dependency line, this
story depends on:

- Rank 1 (`S11-TD-UI-ZINDEX-LAYERS`): for the lobby z-layer
  composition. Placeholder acceptable if rank 1 has not landed.
- Rank 3 (`S11-TD-UI-FLEX-STRIPS`): for the grid / aligned-row
  composition primitive. Inline flex acceptable if rank 3 has not
  landed.
- Rank 4 (`S11-TD-UI-VIEWPORT-INVARIANT-TESTS`): for the viewport-
  invariant test bin. Standalone test acceptable if rank 4 has not
  landed.
- Rank 6 (`S12-UX-GLOBAL-UI-DESIGN-SPEC-001`): for canonical literals
  (grid column count, gap, cell padding, font sizes). Friend-game-
  tier placeholder literals acceptable.

This story is **parallel-safe** within Tier 1 with paired stories 024
(`S12-UX-LOBBY-LAYOUT-MODAL-001`) and 026 (`S11-UX-LOBBY-BUTTON-
HITTARGETS`) because all three touch the same surface module
(`client/src/ui/lobby.rs`); the Sprint 14 activation prompt must
serialise them on file-scope contention. If activated together,
sequence 025 before 024 (so the class-picker hierarchy is a stable
input to 024's root composition) and parallel-safe with 026.

This story does **not** advance: `S8-QA-001-W1`, `QA-COND-0005`,
`QA-COND-0006`, `PAW-TD-*-a`, or PROMPT 761 `Polish->Release` retry.

---

## Out of Scope

- Server-side change.
- Protocol change (`shared/src/protocol.rs`).
- New client-side class-lock authority or optimistic mutations.
- Standard-tier accessibility of the lobby UI (friend-game scope
  only).
- Final-art asset wiring for portraits (`PAW-006` / `PAW-TD-006-a`
  preserved as accept-risk).
- Authoring the global UI design spec
  (`S12-UX-GLOBAL-UI-DESIGN-SPEC-001`), the z-index layer enum
  (`S11-TD-UI-ZINDEX-LAYERS`), the viewport-invariant test bin
  (`S11-TD-UI-VIEWPORT-INVARIANT-TESTS`), or the flex-strips
  primitive (`S11-TD-UI-FLEX-STRIPS`). Separate Sprint 14+ rows.
- Sprint 14 activation. Sprint 13 close-out. `S8-QA-001-W1` closure.
  `Polish->Release` gate-check retry.
- No `/dev-story`, `/story-done`, `/story-readiness`, `/smoke-check`,
  `/team-qa`, `/gate-check`, `/release-check`, or `/qa-plan` under the
  authoring prompt.

---

## Completion Notes -- PROMPT 962 /story-done 2026-05-16

Verdict: PASS. PROMPT 962 accepted AC1-AC12 against the integrated
commit `origin/main@fed5fb9be135db274310c363151a073056927b92`
(PROMPT 961 merge of worker commit
`334434822fbea125d0ebe402611a0ed03212162b` from
`origin/work/s14-lobby-class-picker-957`).

Evidence used:

- PROMPT 961 integration report:
  `reports/PROMPT-961-S14-Lobby-Class-Picker-Integration.md`.
- On-main evidence:
  `production/qa/evidence/sprint-14-lobby-class-picker-evidence.md`.
- On-main test:
  `tests/integration/playable_client/lobby_class_picker_layout_test.rs`
  (5/5 PASS per PROMPT 961).
- Adjacent lobby regressions PASS per PROMPT 961:
  `playable_client_lobby_layout_viewport_invariant_test` 12/12,
  `playable_client_lobby_entry_test` 6/6,
  `playable_client_lobby_confirm_state_text_test` 5/5,
  `lobby_chrome_wiring_test` 5/5, and
  `lobby_asset_wiring_test` 7/7.

AC summary:

- AC1 PASS: `LobbyClassPickerBlock` owns one `LobbyClassPickerHeading`
  and one `LobbyClassPickerGrid`, replacing the prior label plus
  independent wrap rows.
- AC2 PASS: each selectable `ClassId` pairs its `LobbyClassPortrait`
  and `LobbyClassButton` in one `LobbyClassPickerCell`; `Neutral`
  remains portrait-only and non-selectable, documented in evidence.
- AC3 PASS: grid is locked to seven no-wrap columns and fits
  `1366x768` and `1920x1080` panel widths per automated test/evidence.
- AC4 PASS: fixed `108x132` cells, 96px buttons, and label-width
  estimate prevent overlap and silent ellipsis in the story test.
- AC5 PASS: repeat lobby spawns preserve cell dimensions within 1px.
- AC6 PASS: selected-cell affordance exists on first spawn and refreshes
  from `LobbyInputState.selected_class` without respawning the grid.
- AC7 PASS: new integration test covers hierarchy, bounds intent,
  viewport fit, and row-wrap divergence prevention.
- AC8 PASS: UX choices are recorded in the evidence document; no
  Standard-tier accessibility completion is claimed.
- AC9 PASS: forbidden-path review and integrated diff show no server,
  shared, protocol, or class-lock authority change.
- AC10 PASS-WITHIN-STORY-PRESCRIBED-TARGETED-CHECKS: PROMPT 961
  targeted and adjacent lobby checks passed; full-workspace cargo
  smoke remains deferred to Sprint 14 close-out policy.
- AC11 PASS: implementation/integration did not edit shared sprint
  trackers; PROMPT 962 is the separate authorized paperwork edit.
- AC12 PASS: evidence restates friend-game scope and all non-claims.

Runtime browser PNG captures were not produced by the worker or
integration prompts and are not claimed by PROMPT 962. Automated ECS
geometry is the accepted closure evidence for viewport fit and
non-overlap.

No public release readiness, release-candidate readiness, full game
completion, broad accessibility completion, playtest validation,
final-art completion, Sprint 14 close-out, `S8-QA-001-W1` closure,
Polish->Release retry, stage advance, server/shared/protocol change, or
client-side class-lock authority is claimed by this closure.

---

## Authoring / Implementation / Closure Trail

- 2026-05-14 -- PROMPT 880 -- Story file authored (Draft) by Sprint 14
  candidate authoring run on worktree `D:\_DEV\claude-code-game-
  studios-worktrees\s14-lobby-layout-story-authoring`, branch
  `story/s14-lobby-layout-story-authoring`. Source-of-truth:
  `origin/main@51e6228`. No code, no sprint-status flip, no QA, no
  smoke, no gate-check, no `cargo` / `trunk` invocation, no Sprint 14
  activation, no claim against `QA-COND-0005` / `QA-COND-0006` /
  `S8-QA-001-W1` / `PAW-TD-*-a` / PROMPT 761 `Polish->Release` retry.
- 2026-05-16 -- PROMPT 957 -- Dev-story worker commit
  `334434822fbea125d0ebe402611a0ed03212162b` on
  `origin/work/s14-lobby-class-picker-957` implemented the class-picker
  hierarchy, selection affordance refresh, evidence, and integration
  test target.
- 2026-05-16 -- PROMPT 961 -- Integration commit
  `fed5fb9be135db274310c363151a073056927b92` merged the worker branch
  into `origin/main` with targeted and adjacent lobby verification
  PASS.
- 2026-05-16 -- PROMPT 962 -- `/story-done` paperwork closure: Status
  flipped to Done, AC1-AC12 marked complete, Sprint 14 row
  `S11-UX-LOBBY-CLASS-PICKER` flipped `ready -> done`, and shared state
  banners updated. Sprint 14 remains active; stage remains Polish; all
  carried non-claims preserved.
