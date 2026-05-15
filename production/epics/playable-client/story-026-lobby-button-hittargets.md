# Story 026: S11-UX-LOBBY-BUTTON-HITTARGETS -- Lobby Button Dimensions & Hit-Target Stability (Friend-Game Scope)

> **Epic**: Playable Client
> **Story ID**: S11-UX-LOBBY-BUTTON-HITTARGETS
> **Status**: Draft -- Sprint 14 candidate (per `docs/ux/ui-clean-pass-roadmap.md` Tier 1 Should-priority adjacent row, paired with rank 11; PROMPT 685 row 5 button-hittargets slice), NOT activated
> **Layer**: Lobby UI / UX (Client)
> **Type**: Integration -- targeted lobby UI dimension edit (canonical button width/height constants + dimension-stability invariant) + assertion test + visual capture
> **Sprint**: Sprint 14 candidate (per `docs/ux/ui-clean-pass-roadmap.md` Tier 1 Should-priority adjacent rows table; PROMPT 802 §3.1 L5, §4 Tier 1.9; PROMPT 685 row 5 button-hittargets slice, `subsumed-by` PROMPT 802). **NOT** activated by this authoring run.
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

### `QA-COND-0005` Friend-Game-Scope Carve-Out

`docs/ux/ui-clean-pass-roadmap.md` explicitly preserves
`QA-COND-0005` accept-risk on the `LOBBY_BUTTON_HEIGHT = 30.0` defect
(PROMPT 802 §3.1 L5). This story does **not** advance `QA-COND-0005`.
Standard-tier accessibility conformance — including but not limited to
**≥44 Px hit-target minimums** — is **out of scope**.

This story is the friend-game-scope-only repair: it locks canonical
button dimensions and asserts dimension stability across rebuild
(no shifting button widths or heights between spawns) so the lobby
button hit-zones do not surprise the player. It does **not** raise the
button heights to Standard-tier 44 Px. The chosen friend-game-tier
literals MAY remain at or near the current `30 Px` height with
ux-designer sign-off, OR may be modestly increased (e.g. `32-40 Px`)
purely for visual hierarchy purposes, but in either case
`QA-COND-0005` remains accept-risk and is not flipped to `closed` by
this story.

The activation prompt that pulls this row **MUST** re-state the
`QA-COND-0005` accept-risk carve-out on the activation artifact and
**MUST NOT** silently fold this story into Standard-tier accessibility
completion.

**ADR-002 binding preserved**: server-authoritative client-server
authority is unchanged. This story is composition / dimensions only.
No client-side state, no protocol change, no server-side change.

---

## Source Finding

- PROMPT 802 audit §3.1 L5: "Hit-target sizes: `LOBBY_BUTTON_HEIGHT =
  30.0px`, room-code chip `40px`, portraits `64×80`, slot panels
  `48px` tall. 30-40px is below the Standard-tier 44px minimum and
  friend-game-acceptable but is a known accepted-risk via
  `QA-COND-0005`. Calling out so it does not silently regress."
  (`client/src/ui/lobby.rs:22, 24, 983-985, 1024-1025`).
- `docs/ux/ui-clean-pass-roadmap.md` Tier 1 Should-priority adjacent
  rows table:
  > `S11-UX-LOBBY-BUTTON-HITTARGETS` | 1 | Should | 0.25d | Pair with
  > rank 11; note `QA-COND-0005` accept-risk preserved on the L5
  > hit-target defect.
- `docs/ux/ui-clean-pass-roadmap.md` PROMPT 685 -> PROMPT 802
  reconciliation row 5:
  > `subsumed-by S11-UX-LOBBY-CLASS-PICKER (Tier 1.8) + S11-UX-LOBBY-
  > BUTTON-HITTARGETS (Tier 1.9) (re-validated by PROMPT 802 §3.1 L2,
  > L3, L5, §4); §3.1 L5 hit-target ≥44px scope remains QA-COND-0005
  > accept-risk per friend-game scope boundary above.
- PROMPT 685 row 5 button-hittargets slice was named as part of the
  8-story UI-clean-pass milestone in 2026-05-11 and was never authored
  into a story file. This story authors that slice with the
  `QA-COND-0005` accept-risk carve-out preserved verbatim.

---

## Problem Class / Prevention Target

**Defect class**: lobby button dimensions are inconsistent and
unstable across rebuild.

- `LOBBY_BUTTON_HEIGHT = 30.0` Px (lines 22-24) governs only the
  buttons that explicitly use `lobby_button_node(_)`. The slot panels
  use a separate `Val::Px(48.0)` height; the room-code chip image
  uses `Val::Px(40.0)`; the portraits use `64×80 Px`. No shared
  dimension token.
- The lobby is respawned on `OnEnter(ClientState::Lobby)` and
  despawned on `OnExit(ClientState::Lobby)`. There is no test
  asserting that repeat spawns produce identical button dimensions;
  a future per-site dimension edit could silently shift hit-zones.
- The 30-40 Px range is below the Standard-tier 44 Px minimum;
  preserved as `QA-COND-0005` accept-risk per the carve-out above.

**Prevention target**: lock canonical lobby button dimensions
(width / height per button-class: room-code chip, create / join,
slot button, class button, confirm) in a small set of named
constants under `client/src/ui/lobby.rs` (or under a shared lobby
sub-module if the implementing prompt prefers), and assert
dimension stability across rebuild via an integration test. The
chosen dimensions are friend-game-tier and may be near the current
literals; the goal is **stability + named-constants** rather than
absolute-tier conformance.

Paired stories 024 (`S12-UX-LOBBY-LAYOUT-MODAL-001`) and 025
(`S11-UX-LOBBY-CLASS-PICKER`) consume these canonical dimensions:
story 024 AC6 ("stable button dimensions across rebuild") and story
025 AC5 ("stable cell dimensions across rebuild") both refer back
to this story for the canonical numbers.

---

## Context

### Existing surface (read-only at authoring time)

- **`client/src/ui/lobby.rs:22`**: `const LOBBY_BUTTON_HEIGHT: f32 =
  30.0;`. Sole shared button-height constant.
- **`client/src/ui/lobby.rs:895, 907, 928, 950, 969`**: `lobby_button_
  node(Val::Px(128.0))` / `lobby_button_node(Val::Px(72.0))` /
  `lobby_button_node(Val::Px(92.0))` / `lobby_button_node(Val::Percent
  (100.0))`. Per-button-class widths inlined as magic literals at each
  call site.
- **`client/src/ui/lobby.rs:983-985`**: `Node { width: Val::Px(64.0),
  height: Val::Px(80.0), .. }` for portrait cells.
- **`client/src/ui/lobby.rs:998-1000, 1008-1010`**: `Node { width: Val::
  Px(160.0), height: Val::Px(48.0), .. }` for slot panels.
- **`client/src/ui/lobby.rs:1024-1025`**: `Node { width: Val::Px(200.0),
  height: Val::Px(40.0), .. }` for room-code chip.
- **`lobby_button_node` helper**: existing fn in `client/src/ui/lobby.rs`
  that wraps a `Node { width, height: LOBBY_BUTTON_HEIGHT, .. }`. The
  implementing prompt may extend this helper, add named per-button-class
  helpers, or introduce a small `LobbyButtonDimensions` constant block.
  Pattern selection is owned by the implementing prompt with ux-designer
  consultation per AC4.

### GDD / ADR / TR trace

- **GDD**: `design/gdd/game-session-system.md` (lobby flow).
- **ADR-002** (Client-Server Authority): no client-side authority
  added.
- **ADR-021** (Presentation Layer Architecture): preserved.
- **TR registry**: no new TR (dimension constants only).

### Engine / skills

- **Engine**: Bevy 0.18 (Rust).
- **Mandatory skills**: `liv-bevy-018` for any `.rs` edit.

### Control Manifest Rules

- Required: lobby button dimensions are declared as named constants
  (per-button-class), not inlined as magic literals at each call
  site.
- Required: lobby button dimensions are stable across rebuild
  (repeat `OnEnter(ClientState::Lobby)` spawns produce identical
  `width` and `height` per button-class).
- Required: lobby button text fits inside the button at the
  canonical font size at the locked dimensions, without silent
  ellipsis or single-line overflow.
- Required: the `QA-COND-0005` accept-risk carve-out is preserved
  verbatim in the evidence document and in the implementation diff
  rationale.
- Forbidden: claiming Standard-tier accessibility conformance,
  claiming ≥44 Px hit-target completion, claiming `QA-COND-0005`
  advanced.
- Forbidden: new client-side authority, server-side change,
  protocol change.
- Forbidden: edits to `shared/`, `server/`, `tests/`,
  `production/sprint-status.yaml`, `production/sprints/`,
  `production/qa/`, `production/session-state/`,
  `production/stage.txt`, the PROMPT 761 gate-check artifact, or any
  `Cargo` file.

---

## Story Classification

**Story type**: Integration -- targeted lobby UI dimension edit
(canonical button-dimension constants) + dimension-stability
integration test + visual capture. Friend-game scope only.

This is **NOT** a:

- Pure UX-spec story (real client UI lands).
- Server-side change.
- Protocol change.
- Accessibility-tier (Standard-tier) repair. The `QA-COND-0005`
  accept-risk is preserved verbatim.
- Final-art asset wiring.

---

## Acceptance Criteria

All criteria are independently checkable.

- [ ] **AC1 -- Named per-button-class dimension constants**: GIVEN
  `client/src/ui/lobby.rs` after the implementation, WHEN inspected,
  THEN per-button-class width and height are declared as named
  constants (e.g. `LOBBY_CREATE_BUTTON_WIDTH`, `LOBBY_SLOT_BUTTON_
  WIDTH`, `LOBBY_CLASS_BUTTON_WIDTH`, `LOBBY_CONFIRM_BUTTON_WIDTH`,
  `LOBBY_ROOM_CODE_CHIP_DIMENSIONS`, `LOBBY_SLOT_PANEL_DIMENSIONS`,
  `LOBBY_PORTRAIT_DIMENSIONS`; exact names locked by implementing
  prompt + ux-designer). No magic `Val::Px(N)` literal in
  `spawn_lobby_ui_system` for button-class dimensions; literals are
  resolved through the named constants.

- [ ] **AC2 -- Dimension stability across rebuild**: GIVEN a new
  integration test (e.g.
  `tests/integration/playable_client/lobby_button_dimensions_test.rs`),
  WHEN the test drives `OnExit(ClientState::Lobby)` followed by
  `OnEnter(ClientState::Lobby)` and samples per-button-class width
  and height across the two spawns, THEN the dimensions match within
  1 Px tolerance for each button class. The test asserts the
  per-button-class width / height values match the named constants
  introduced by AC1.

- [ ] **AC3 -- Text fit at canonical dimensions**: GIVEN the lobby
  UI rendered at `1920×1080` and at `1366×768`, WHEN button text is
  inspected, THEN the canonical font size renders each button label
  fully inside the button without silent ellipsis insertion or
  single-line overflow. If a label does not fit at the chosen
  dimension, AC3 is satisfied via either (a) dimension increase
  with ux-designer sign-off, (b) font-size decrease with ux-designer
  sign-off subject to friend-game-tier readability, or (c) label
  wrap to two lines with ux-designer sign-off.

- [ ] **AC4 -- ux-designer consultation recorded**: GIVEN the
  implementation prompt's first ux-designer interaction, WHEN the
  final per-button-class dimensions are locked, THEN the consultation
  note and chosen literals are recorded in the evidence document.
  The note MUST explicitly call out the `QA-COND-0005` accept-risk
  carve-out and confirm that the chosen literals remain in
  friend-game scope (no claim of ≥44 Px Standard-tier compliance).

- [ ] **AC5 -- `QA-COND-0005` carve-out preserved**: GIVEN the
  evidence document and the implementation diff rationale, WHEN
  read, THEN the friend-game-scope-only carve-out is restated
  verbatim from this story's Status / No-Claim Banner including
  the explicit "no claim of ≥44 Px Standard-tier compliance" line.
  No flip of `QA-COND-0005` status; no edit of
  `production/qa/accept-risk-registry.md` (or canonical equivalent)
  is permitted.

- [ ] **AC6 -- Consumed by paired stories 024 / 025**: GIVEN this
  story's named constants land, WHEN paired stories 024
  (`S12-UX-LOBBY-LAYOUT-MODAL-001`) and 025 (`S11-UX-LOBBY-CLASS-
  PICKER`) reference dimension stability (story 024 AC6, story 025
  AC5), THEN those references resolve to the constants defined by
  this story (or to an equivalent surface chosen by the implementing
  prompt and ux-designer; in either case the constants are visible
  to a reader of `client/src/ui/lobby.rs` without grep'ing for magic
  literals).

- [ ] **AC7 -- No client-side authority added (ADR-002)**: GIVEN
  the implementation diff, WHEN reviewed, THEN no client-side
  state mutation, no protocol change in `shared/src/protocol.rs`,
  no server-side change is present. The diff is composition /
  dimensions only.

- [ ] **AC8 -- Workspace test pass**: GIVEN
  `cargo test --workspace --tests --no-fail-fast` at the
  implementation commit, WHEN compared to the post-Sprint-13
  baseline, THEN no new `#[ignore]` markers are introduced; the new
  dimension-stability test passes; previously-passing tests continue
  to pass.

- [ ] **AC9 -- No `production/` shared-tracker edits**: GIVEN the
  implementation commit, WHEN `production/sprint-status.yaml`,
  `production/sprints/`, `production/qa/`, `production/stage.txt`,
  `production/session-state/`, and the PROMPT 761 gate-check
  artifact are diffed, THEN none is modified by this story's
  implementing prompt except in the `/story-done` paperwork
  commit. Critically, no `QA-COND-0005` status flip lands.

- [ ] **AC10 -- Visual capture at 1920×1080 and 1366×768**: GIVEN
  the lobby UI rendered at both viewports, WHEN captured, THEN
  no button text overflows its container; no button overlaps
  another button (paired with story 024 AC3 / AC4 and story 025
  AC4); per-button-class dimensions look visually consistent within
  each class.

- [ ] **AC11 -- Friend-game-scope no-claim restated in evidence**:
  GIVEN the evidence document, WHEN read at the bottom, THEN it
  verbatim restates the friend-game-scope-only disposition and the
  `QA-COND-0005` accept-risk carve-out from this story's Status /
  No-Claim Banner.

---

## Likely Files

| Path | Anticipated change |
|------|--------------------|
| `client/src/ui/lobby.rs` (canonical path verified by implementing worker) | Edited: named per-button-class dimension constants introduced; `spawn_lobby_ui_system` (and helper fns like `lobby_button_node`) refactored to consume the constants instead of inline `Val::Px(N)` literals. |
| `tests/integration/playable_client/lobby_button_dimensions_test.rs` (or canonical equivalent under the rank-4 viewport-invariant test bin) | NEW integration test asserting AC2 (dimension stability across rebuild) and AC1 (constants match call sites). |
| `production/qa/evidence/sprint-14-lobby-button-hittargets-evidence.md` (slot reserved; sprint number may be adjusted at activation time) | NEW evidence document: ux-designer consultation (AC4) with explicit `QA-COND-0005` carve-out; integration-test pass output; 1920×1080 + 1366×768 captures (AC10); friend-game-scope no-claim restatement (AC11). |
| This story file | Status flip Draft -> Implemented / Done on `/story-done` paperwork. |
| `production/epics/playable-client/EPIC.md` | Status flip on `/story-done` paperwork. |

This table is a planning estimate. The implementing prompt is
authoritative for the realised set.

---

## Required Skills

- **`liv-bevy-018`** -- mandatory for `.rs` edits to
  `client/src/ui/lobby.rs`.
- **`ux-designer` agent** -- mandatory consultation per AC4.
- **`accessibility-specialist` agent** -- recommended consultation to
  re-confirm the `QA-COND-0005` accept-risk disposition for the
  chosen friend-game-tier literals (note: re-confirmation does NOT
  advance `QA-COND-0005`; it preserves the accept-risk disposition).

---

## Evidence Path

`production/qa/evidence/sprint-14-lobby-button-hittargets-evidence.md`
(slot reserved; sprint number may be adjusted at activation time;
populated by the implementing prompt).

**Required evidence content**:

- Diff summary for `client/src/ui/lobby.rs` (dimension-constant
  introduction + call-site refactor).
- ux-designer consultation note (AC4) with chosen per-button-class
  literals and explicit `QA-COND-0005` accept-risk carve-out
  re-statement.
- New dimension-stability integration-test pass output (AC2 / AC8).
- 1920×1080 visual capture (AC10) and 1366×768 visual capture (AC10),
  saved under
  `production/qa/evidence/captures/sprint-14-lobby-button-hittargets/`.
- Cross-link to paired stories 024 (`S12-UX-LOBBY-LAYOUT-MODAL-001`)
  and 025 (`S11-UX-LOBBY-CLASS-PICKER`).
- Cross-link to `docs/ux/ui-clean-pass-roadmap.md` (Tier 1 Should-
  priority adjacent rows) and to PROMPT 685 row 5 + PROMPT 802
  §3.1 L5.
- accessibility-specialist re-confirmation note (recommended).
- Verbatim no-claim restatement (AC11).
- ADR-002 / ADR-021 preservation note (AC7).

---

## Regression Commands Expected

For the implementing prompt (not run by PROMPT 880):

- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets`
- `cargo test --workspace --tests --no-fail-fast`
- `cargo test -p client --test lobby_button_dimensions -- --nocapture`
  (or the new test name)
- `git diff <pre-impl-sha>..<impl-sha> -- 'shared/src/**' 'server/src/**'`
  (verifies AC7: zero protocol-shape change, zero server-side change)
- `git diff --check origin/main...HEAD`
- `git diff --cached --check`

---

## Dependency Notes Against Sprint 14 Pull-In Sequence

Per `docs/ux/ui-clean-pass-roadmap.md` Tier 1 Should-priority adjacent
rows table, this row pairs with rank 11 (`S11-UX-LOBBY-CLASS-PICKER`,
story 025). Pull-in order:

- Rank 4 (`S11-TD-UI-VIEWPORT-INVARIANT-TESTS`): for the test bin.
  Standalone test acceptable if rank 4 has not landed.
- Rank 6 (`S12-UX-GLOBAL-UI-DESIGN-SPEC-001`): for canonical
  per-button-class dimensions if a shared spec table exists.
  Friend-game-tier placeholder literals acceptable.

This story is **parallel-safe** within Tier 1 with paired stories 024
(`S12-UX-LOBBY-LAYOUT-MODAL-001`) and 025 (`S11-UX-LOBBY-CLASS-
PICKER`) because all three touch the same surface module
(`client/src/ui/lobby.rs`); the Sprint 14 activation prompt must
serialise them on file-scope contention. If activated together,
sequence 026 first or in parallel with 025 (so dimensions are a
stable input to 024's root composition AC6 and 025's cell-stability
AC5).

This story does **not** advance: `S8-QA-001-W1`, `QA-COND-0005`,
`QA-COND-0006`, `PAW-TD-*-a`, or PROMPT 761 `Polish->Release` retry.

---

## Out of Scope

- Server-side change.
- Protocol change (`shared/src/protocol.rs`).
- New client-side authority or optimistic mutations.
- Standard-tier accessibility of the lobby UI -- explicitly out of
  scope per the `QA-COND-0005` carve-out above. ≥44 Px hit-target
  conformance is **not** claimed; friend-game-tier 30-40 Px
  literals MAY remain.
- Full keyboard navigation, screen-reader support, colorblind modes,
  text scaling, WCAG contrast for lobby buttons (`QA-COND-0005`
  accept-risk preserved).
- Final-art asset wiring (`PAW-006` / `PAW-TD-006-a` accept-risk
  preserved).
- Authoring the global UI design spec
  (`S12-UX-GLOBAL-UI-DESIGN-SPEC-001`) or the viewport-invariant
  test bin (`S11-TD-UI-VIEWPORT-INVARIANT-TESTS`).
- Sprint 14 activation. Sprint 13 close-out. `S8-QA-001-W1` closure.
  `Polish->Release` gate-check retry.
- No `/dev-story`, `/story-done`, `/story-readiness`, `/smoke-check`,
  `/team-qa`, `/gate-check`, `/release-check`, or `/qa-plan` under
  the authoring prompt.

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
  `QA-COND-0005` accept-risk carve-out preserved verbatim.
