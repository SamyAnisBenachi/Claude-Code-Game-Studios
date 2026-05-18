# Story 027: S18-LOBBY-CONFIRM-CTA-VISIBLE-001 -- Lobby Confirm CTA Reachability & Room-Browser Layout

> **Epic**: Playable Client
> **Story ID**: `S18-LOBBY-CONFIRM-CTA-VISIBLE-001`
> **Status**: Draft -- future Sprint 18 candidate; NOT activated
> **Layer**: Lobby UI (Client)
> **Type**: UI + Integration (viewport-invariant node-size assertions + two-client reachability)
> **Sprint**: Sprint 18 Wave-A candidate per PROMPT 1287 §5 (NOT activated)
> **Authored**: 2026-05-18 by PROMPT 1294
> **Authoring worktree**: `D:\_DEV\claude-code-game-studios-worktrees\s18-story-authoring-wave-a-1294`
> **Authoring branch**: `work/s18-story-authoring-wave-a-1294`
> **Authoring source-of-truth**: `origin/main@1345c6b8b1cbd543dbd63d279186c93924ca54db` (PROMPT 1285 Sprint 18 plan draft)
> **Source reports**: PROMPT 1201 HUNT-1201-01 / HUNT-1201-02 / HUNT-1201-20; PROMPT 1180 L-01 / L-03; PROMPT 1287 §5 Wave-A

---

## Status / No-Claim Banner

PROMPT 1294 authors this story as a **future Sprint 18 Wave-A candidate**.
Sprint 18 is `draft` on `origin/main` (`production/sprints/sprint-18.md`,
authored by PROMPT 1285) and is **NOT activated** by PROMPT 1294.

PROMPT 1294 (this authoring run) does **NOT**:

- Activate Sprint 18.
- Modify `production/sprint-status.yaml`.
- Modify `production/sprints/sprint-18.md` (or any other sprint plan file).
- Modify `production/stage.txt` (remains `Polish`).
- Modify any file under `production/session-state/**`,
  `production/qa/**`, or `production/gate-checks/**`.
- Run `/story-readiness`, `/dev-story`, `/story-done`, `/smoke-check`,
  `/team-qa`, `/gate-check`, `/release-check`, or `/qa-plan`.
- Modify any code under `client/`, `server/`, `shared/`, or `tests/`.
- Modify any `Cargo.toml` / `Cargo.lock` / `.cargo/` / `.github/` file.
- Push to `origin/main`.

This story does **not** claim:

- public release readiness
- release-candidate readiness
- full game completion
- broad / Standard-tier accessibility completion (`QA-COND-0005`)
- playtest / fun-hypothesis validation (`QA-COND-0006`)
- closure of `S8-QA-001-W1`
- final-art / asset-production completion (`PAW-TD-*-a`)
- `Polish->Release` gate-check retry (PROMPT 761 FAIL preserved)
- advance of stage from `Polish` to `Release`

ADR-002 binding preserved: this story is composition / layout only. No
client-side authority, no protocol shape change, no server-side change.

The `QA-COND-0005` accept-risk carve-out on the lobby button
hit-target defect (Story 026 `S11-UX-LOBBY-BUTTON-HITTARGETS`) is
preserved verbatim. This story does **not** raise button heights to
the Standard-tier 44 Px minimum; it only asserts CTA *reachability*
across canonical viewports.

---

## Source Findings

PROMPT 1201 (multiplayer hunt audit) records the lobby Confirm CTA
reachability and room-browser layout defects as P0 hunt rows:

- **HUNT-1201-01** (P0): at 1366x768 (and at smaller viewport heights),
  the lobby modal panel's `max_height: 92%` clamp pushed the
  `LobbyConfirmClassButton` past the visible viewport once both
  players had selected a class. PROMPT 985 partially remediated by
  shortening the status banner to two lines; the residual gap is
  that the assertion is not re-evidenced across the full canonical
  viewport set (1280x720 / 1366x768 / 1920x1080) after subsequent
  lobby content additions.
- **HUNT-1201-02** (P0): the room-list browser
  (`LobbyRoomListContainer`) overflow behaviour is not bounded —
  when more than ~5 joinable rooms exist the list can grow past the
  modal panel body and push the Confirm CTA below the viewport
  again, defeating the PROMPT 985 fix.
- **HUNT-1201-20** (P0): row-click join target labels
  (`format_room_list_row_label`) read as a single compressed line
  at narrow widths and are not visibly distinguishable from
  non-interactive informational rows.

PROMPT 1180 L-class lobby defects (cross-referenced by PROMPT 1287
§5 Wave-A as the Wave-A authoring source):

- **L-01**: class confirmation can be triggered before the player
  has visually confirmed which class chip was selected — the
  picker chips read as ambiguous "selected" state for the local
  player vs. the server-locked state.
- **L-03**: two-client reach-to-`DraftInitial` smoke is not
  asserted by an automated harness at the supported viewport
  matrix; manual evidence is the only proof and is fragile under
  layout drift.

PROMPT 1287 §5 Wave-A row pins this story slug:
`S18-LOBBY-CONFIRM-CTA-VISIBLE-001`, owner epic
`playable-client` (preferred), likely implementation file
`client/src/ui/lobby.rs`.

---

## Problem Class / Prevention Target

**Defect class**: the lobby Confirm CTA (`LobbyConfirmClassButton`)
and the room-browser surface (`LobbyRoomListContainer`) lack
viewport-invariant placement guarantees. Layout drift in either
surface can silently hide the primary CTA below the visible
viewport, blocking the two-client reach-to-`DraftInitial` path
without producing any error log.

**Prevention target**: make Confirm-CTA visibility and the
room-browser layout a **first-class viewport invariant** asserted
by an automated test at the supported viewport matrix
(1280x720, 1366x768, 1920x1080), and prove the two-client
reach-to-`DraftInitial` path completes from both clients without
relying on manual evidence. Row-click join rows must be visibly
distinguished from non-interactive informational rows; chip-only
class confirmation paths must be removed or explicitly gated.

---

## Context

### Existing surface (read-only at authoring time)

- **`client/src/ui/lobby.rs:48-78`**: lobby button / chip / slot /
  portrait dimension constants
  (`LOBBY_BUTTON_HEIGHT_PX = 30.0`,
  `LOBBY_CONFIRM_BUTTON_WIDTH_PERCENT = 100.0`,
  `LOBBY_CONFIRM_BUTTON_HEIGHT_PX = LOBBY_BUTTON_HEIGHT_PX`,
  `LOBBY_SLOT_PANEL_*_PX`, `LOBBY_ROOM_CODE_CHIP_*_PX`,
  `LOBBY_CLASS_PICKER_*_PX`). `QA-COND-0005` accept-risk
  preserved.
- **`client/src/ui/lobby.rs:37-45`**: lobby modal panel
  size literals (`LOBBY_PANEL_WIDTH_PERCENT = 88.0`,
  `LOBBY_PANEL_MAX_WIDTH_PX = 860.0`,
  `LOBBY_PANEL_MAX_HEIGHT_PERCENT = 92.0`).
- **`client/src/ui/lobby.rs:1678-1700`**: `LobbyConfirmClassButton`
  spawn site inside the lobby modal panel. The button is the
  bottom-most child of the panel content column.
- **`client/src/ui/lobby.rs:1715, 1749-1808`**:
  `LobbyRoomListContainer` and `rebuild_room_list_rows`. Rows are
  rebuilt on every `S2CRoomList` drain; there is no `max_height`
  / overflow clamp on the container.
- **`client/src/ui/lobby.rs:1820-1830`**: PROMPT 985 status-banner
  two-line truncation rationale (preserves
  `lobby_confirm_button_reachable_test.rs` and
  `class_confirmations_are_server_confirmed`).
- **`tests/integration/playable_client/lobby_confirm_button_reachable_test.rs`**:
  PROMPT 985 viewport reachability assertion (existing baseline;
  this story extends the matrix and adds the room-browser
  overflow case).
- **`tests/integration/playable_client/lobby_button_dimensions_test.rs`**:
  Story 026 dimension-stability test (existing baseline;
  preserved).

### GDD / ADR / TR trace

- **GDD**: `design/gdd/game-session-system.md` (lobby flow → DraftInitial).
- **ADR-002** (Client-Server Authority): preserved; no new
  client-side authority.
- **ADR-021** (Presentation Layer Architecture): preserved;
  lobby UI stays inside bevy_ui composition order.
- **TR registry**: no new TR (composition / layout invariant only).

### Engine / skills

- **Engine**: Bevy 0.18 (Rust).
- **Mandatory skills**: `liv-bevy-018` for any `.rs` edit under
  `client/src/ui/lobby.rs` and for the integration test.
- **Lightyear**: NOT applicable; `liv-bevy-lightyear` not required.

### Control Manifest Rules

- **Required**: At each viewport in {1280x720, 1366x768, 1920x1080},
  after both players select a class, the
  `LobbyConfirmClassButton` node's computed on-screen bounds are
  fully inside the visible viewport (top >= 0, bottom <= viewport
  height) AND the button is `Visibility::Visible` AND
  `Interaction != Disabled` for the player whose class is
  confirmable.
- **Required**: The `LobbyRoomListContainer` has a bounded
  `max_height` / `overflow: Scroll`-equivalent layout policy so
  that an arbitrarily long room list does NOT push the
  Confirm CTA past the viewport. The exact mechanism is
  implementation-prompt discretion (bounded height + scroll, or
  bounded height + truncation with "+N more rooms" indicator)
  subject to ux-designer sign-off.
- **Required**: Row-click join rows
  (`LobbyRoomListRow`-marked spawns) are visibly distinct from
  non-interactive informational rows — chosen mechanism is
  worker discretion within the existing
  `interaction_states::HOVER_*` / `PRESSED_*` token set; the
  BLOCKING assertion is that an integration test can
  distinguish row-click rows from informational rows by ECS
  marker / colour query without inspecting raw pixels.
- **Required**: Class confirmation MUST be gated behind the
  existing `LobbyConfirmClassButton` press path. Chip-only
  confirmation (e.g. accidental keyboard / pointer activation
  on a `LobbyClassPickerCell` immediately triggering
  `LobbyCommand::ConfirmClass`) MUST NOT be possible. The
  picker cell's `Interaction` state may select the class; only
  the explicit Confirm CTA press dispatches `C2SConfirmClass`.
- **Required**: Two-client reach-to-`DraftInitial` evidence:
  either an integration test that drives two simulated clients
  through `Create → Join → SelectClass(both) → Confirm(both)
  → S2CPhaseChanged(DraftInitial)` OR a documented two-client
  manual evidence walkthrough at the canonical viewports.
- **Required**: `liv-bevy-018` skill applied to all `.rs` edits.
- **Forbidden**: Mutating server-authoritative state from the
  lobby UI; the only client-side mutation paths remain the
  existing C2S message senders (`C2SCreateRoom`,
  `C2SJoinRoom`, `C2SListRooms`, `C2SSelectClass`,
  `C2SConfirmClass`).
- **Forbidden**: Editing `shared/src/protocol.rs`, the server
  lobby code, or any QA / sprint / session-state tracker.
- **Forbidden**: Claiming Standard-tier accessibility
  conformance, claiming ≥44 Px hit-target completion, claiming
  `QA-COND-0005` advancement.
- **Forbidden**: Touching `production/sprint-status.yaml`,
  `production/sprints/`, `production/qa/`, `production/stage.txt`,
  `production/session-state/`, the PROMPT 761 gate-check
  artifact, or any `Cargo` file.

---

## Story Classification

**Story type**: UI + Integration test — viewport-invariant lobby
layout assertions + two-client reach-to-`DraftInitial` evidence.
Friend-game scope only.

This is **NOT** a:

- Pure UX-spec story (real client UI lands).
- Server-side change.
- Protocol change.
- Accessibility-tier (Standard-tier) repair.
- Final-art asset wiring.

---

## Acceptance Criteria

All criteria are independently checkable.

- [ ] **AC1 — Confirm CTA visible at 1280x720 after both classes
  selected**: GIVEN a synthesised lobby with two `SessionSlot`
  entries, each holding a `selected_class`, AND the viewport sized
  to 1280x720, WHEN the lobby is rendered for one tick, THEN the
  `LobbyConfirmClassButton` node's computed on-screen bounds are
  fully inside `0..720` on the Y axis (top >= 0, bottom <= 720)
  AND `Visibility::Visible`. Verified by integration-test ECS
  query + bevy_ui `ComputedNode` inspection.
- [ ] **AC2 — Confirm CTA visible at 1366x768**: same as AC1 at
  1366x768.
- [ ] **AC3 — Confirm CTA visible at 1920x1080**: same as AC1 at
  1920x1080.
- [ ] **AC4 — Room-browser overflow does not hide Confirm CTA**:
  GIVEN a synthesised `LobbyViewState.room_list` with at least 12
  entries (more than fit unconstrained in the panel body), at the
  smallest viewport in the matrix (1280x720), WHEN the lobby is
  rendered for one tick, THEN the Confirm CTA bounds are still
  inside the viewport (AC1 holds) AND the room list applies a
  bounded layout policy (`max_height` clamp OR `overflow: Scroll`
  OR explicit truncation marker). Verified by integration-test
  ECS query.
- [ ] **AC5 — Row-click rows visibly distinct from informational
  rows**: GIVEN a `LobbyRoomListRow`-marked row entity AND a
  non-`LobbyRoomListRow` informational fallback entity in the
  same container, WHEN inspected, THEN at least one of the
  following differs between the two:
  - `Button` component presence,
  - `Interaction` component presence,
  - `BackgroundColor` value,
  - `BorderColor` value,
  - text colour (`TextColor`).
  Verified by integration-test ECS query (no pixel inspection).
- [ ] **AC6 — Class confirmation requires explicit Confirm-CTA
  press**: GIVEN a `LobbyClassPickerCell` receives an
  `Interaction::Pressed`, WHEN the lobby command dispatcher runs
  for one tick, THEN NO `LobbyCommand::ConfirmClass` is written
  by that picker-cell press alone. A subsequent
  `LobbyConfirmClassButton` press IS required to write
  `LobbyCommand::ConfirmClass`. Verified by integration-test
  command-buffer drain.
- [ ] **AC7 — Two-client reach-to-`DraftInitial`**: GIVEN two
  simulated clients (`tests/integration/playable_client/` style
  harness OR documented manual walkthrough with screen captures),
  WHEN both clients execute Create / Join / SelectClass / Confirm
  in order, THEN both clients observe an
  `S2CPhaseChanged { phase: RoundPhase::DraftInitial }` and
  transition into the `InSession` client state. Each canonical
  viewport (AC1-3) must show at least one captured frame of the
  Confirm CTA at the moment of press; a single integration test
  that drives the two-client harness across the matrix is
  acceptable.
- [ ] **AC8 — Existing PROMPT 985 reachability test continues to
  PASS**: GIVEN the post-implementation build, WHEN
  `cargo test -p client --test lobby_confirm_button_reachable_test`
  runs, THEN it PASSES. Any extension to that test file (new
  viewport rows / new room-list overflow row) is acceptable; the
  pre-existing assertions MUST NOT regress.
- [ ] **AC9 — Story 026 dimension-stability test continues to
  PASS**: GIVEN the post-implementation build, WHEN
  `cargo test -p client --test lobby_button_dimensions_test`
  runs, THEN it PASSES. The `QA-COND-0005` accept-risk
  carve-out is preserved verbatim.
- [ ] **AC10 — No client-side authority added (ADR-002)**:
  GIVEN the implementation diff, WHEN reviewed, THEN no
  client-side state mutation, no protocol change in
  `shared/src/protocol.rs`, and no server-side change is
  present. The diff is composition / layout / interaction
  gating only.
- [ ] **AC11 — Visual evidence at all three canonical
  viewports**: GIVEN the post-implementation build, WHEN
  browser / WASM captures are taken at 1280x720 / 1366x768 /
  1920x1080 after both classes are selected, THEN the
  evidence document records one capture per viewport showing
  the Confirm CTA visible and not occluded by the room list.
  Per Story 026 / Story 022 precedent, a documented manual
  walkthrough with ECS node-dimension sampling is acceptable
  if pixel captures are infeasible.
- [ ] **AC12 — No `production/` shared-tracker edits**: GIVEN
  the implementation commit, WHEN
  `production/sprint-status.yaml`, `production/sprints/`,
  `production/qa/`, `production/stage.txt`,
  `production/session-state/`, and the PROMPT 761
  gate-check artifact are diffed, THEN none is modified by
  this story's implementing prompt except in the
  `/story-done` paperwork commit (epic index + story-status
  refresh).
- [ ] **AC13 — Workspace test pass**: GIVEN
  `cargo test --workspace --tests --no-fail-fast` at the
  implementation commit, WHEN compared to the Sprint 18
  activation-tip baseline, THEN no new `#[ignore]` markers
  are introduced; the new viewport / overflow / row-click /
  chip-gating assertions pass; previously-passing tests
  continue to pass.
- [ ] **AC14 — No new Lightyear / protocol message**: GIVEN the
  post-implementation build, WHEN `git diff` is inspected for
  `shared/src/protocol.rs` / `shared/src/network/` /
  `client/src/network/` / `server/src/network/`, THEN no diff
  is present. `liv-bevy-lightyear` is NOT activated.
- [ ] **AC15 — Friend-game-scope no-claim restated in
  evidence**: GIVEN the evidence document, WHEN read, THEN
  it verbatim restates the friend-game-scope-only
  disposition and the `QA-COND-0005` accept-risk carve-out
  from this story's Status / No-Claim Banner.
- [ ] **AC16 — Carried conditions preserved**: GIVEN the
  evidence and the implementation commit, WHEN inspected,
  THEN no claim is made against any of:
  `S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006`,
  `PAW-TD-*-a`, `S11-HUD-TIMER-EYEBALL-VISUAL-001` (human-
  blocked), PROMPT 761 `Polish->Release` gate-check retry,
  or stage advance from `Polish`.

---

## Dependencies

| Dependency | Required posture | Why blocking |
|---|---|---|
| Sprint 18 activation | Required before `/dev-story` | This story is a Sprint 18 candidate. |
| Story 026 (S11-UX-LOBBY-BUTTON-HITTARGETS) | Complete on `origin/main` (PROMPT 972) | Dimension stability + `QA-COND-0005` carve-out is the source baseline for the AC9 regression assertion. |
| Story 024 (S12-UX-LOBBY-LAYOUT-MODAL-001) | Complete on `origin/main` | Lobby modal panel composition is the host surface. AC1-3 viewport invariants extend its `max_height: 92%` clamp. |
| `lobby_confirm_button_reachable_test.rs` | Existing baseline (PROMPT 985) | Extension target for AC1-3 / AC8. |
| `S18-UI-VIEWPORT-INVARIANT-LIVE-HARNESS-001` (Should Have) | On `origin/main` per PROMPT 1287 §2 inventory (PROMPT 1185 / commit `671c677`) | Useful but not strictly blocking; viewport assertions may live in the existing reachability test if the live harness is not yet promoted to a shared dependency. |

This story touches `client/src/ui/lobby.rs` and should not run
concurrently with another worker editing the same file (Story 024
/ 025 / 026 are Complete on `origin/main`; verify at activation
time).

---

## Likely Files

| Path | Anticipated change |
|------|--------------------|
| `client/src/ui/lobby.rs` | Bounded room-list overflow policy (`max_height` or `overflow: Scroll` on `LobbyRoomListContainer`); row-click row visual differentiation tightened; chip-only confirmation guard (defensive — verify `LobbyClassPickerCell` press never dispatches `LobbyCommand::ConfirmClass`); minor copy / spacing tweaks to support narrowest viewport. |
| `tests/integration/playable_client/lobby_confirm_button_reachable_test.rs` (extended) OR `lobby_confirm_cta_visible_test.rs` (NEW, worker discretion) | Viewport matrix expansion (1280x720 / 1366x768 / 1920x1080); long-room-list overflow case; row-click vs informational row distinction; chip-only confirmation guard. |
| `tests/integration/playable_client/lobby_two_client_reach_draft_initial_test.rs` (NEW, optional — may be folded into existing two-client harness) | Two-client reach assertion AC7. May be deferred to the live two-client harness (`S13-TWO-CLIENT-RUNTIME-HARNESS-001`) if `/dev-story` chooses. |
| `production/qa/evidence/sprint-18-lobby-confirm-cta-visible-evidence.md` (NEW) | ux-designer consultation note; integration-test pass output; viewport captures (AC11); two-client reach evidence (AC7); friend-game-scope no-claim restatement (AC15). |
| This story file | Status flip Draft → Implemented / Done on `/story-done` paperwork. |
| `production/epics/playable-client/EPIC.md` | Status flip on `/story-done` paperwork. |

This table is a planning estimate. The implementing prompt is
authoritative for the realised set.

---

## Required Skills

- **`liv-bevy-018`** — mandatory for `.rs` edits to
  `client/src/ui/lobby.rs`.
- **`ux-designer` agent** — recommended consultation on the
  row-browser overflow policy choice and on row-click
  visual differentiation.
- **`accessibility-specialist` agent** — recommended
  consultation to re-confirm the `QA-COND-0005`
  accept-risk disposition (re-confirmation does NOT
  advance `QA-COND-0005`; it preserves the accept-risk
  disposition).

---

## Evidence Path

`production/qa/evidence/sprint-18-lobby-confirm-cta-visible/`
(populated by the implementing prompt).

**Required evidence content**:

- Diff summary for `client/src/ui/lobby.rs` and the test
  files.
- ux-designer consultation note with the chosen overflow
  policy and row-distinction mechanism.
- Integration-test pass output (AC1-AC9 / AC13).
- Viewport captures at 1280x720 / 1366x768 / 1920x1080
  (AC11), OR documented manual walkthrough with ECS
  node-dimension sampling (Story 026 precedent).
- Two-client reach-to-`DraftInitial` evidence (AC7).
- `QA-COND-0005` accept-risk carve-out re-statement
  (AC15).
- Carried-conditions no-claim restatement (AC16).
- Cross-link to PROMPT 1201 HUNT-1201-01 / -02 / -20 and
  PROMPT 1180 L-01 / L-03.
- ADR-002 / ADR-021 preservation note (AC10).

---

## Out of Scope

- Server-side change.
- Protocol change (`shared/src/protocol.rs`).
- New client-side authority or optimistic mutations.
- Standard-tier accessibility of the lobby UI —
  `QA-COND-0005` accept-risk preserved.
- Full keyboard navigation, screen-reader support,
  colourblind modes, text scaling, WCAG contrast for
  lobby buttons.
- Final-art asset wiring (`PAW-TD-*-a` accept-risk
  preserved).
- Sprint 18 activation. Sprint 17 close-out reopen.
  `S8-QA-001-W1` closure. `Polish->Release` gate-check
  retry. Stage advance from `Polish`.
- No `/dev-story`, `/story-done`, `/story-readiness`,
  `/smoke-check`, `/team-qa`, `/gate-check`,
  `/release-check`, or `/qa-plan` under the authoring
  prompt.

---

## QA Test Cases

*Drafted by qa-lead at story creation. The developer
implements against these — do not invent new test cases
during implementation.*

- **AC1-3 — Confirm CTA visible at each canonical
  viewport after both classes selected**:
  - Given: lobby modal panel rendered with two
    `SessionSlot`s each carrying a `selected_class`;
    viewport = (W, H) ∈ {(1280, 720), (1366, 768),
    (1920, 1080)}.
  - When: one tick.
  - Then: `LobbyConfirmClassButton` `ComputedNode.bounds`
    fully inside `0..H` on the Y axis; node is
    `Visibility::Visible`.

- **AC4 — Long room-list does not occlude Confirm CTA**:
  - Given: `LobbyViewState.room_list` with 12+ entries;
    viewport = (1280, 720).
  - When: one tick.
  - Then: Confirm CTA bounds still inside the viewport AND
    `LobbyRoomListContainer` applies a bounded layout
    policy.

- **AC5 — Row-click rows visibly distinct**:
  - Given: one `LobbyRoomListRow`-marked entity and one
    informational row entity in the same container.
  - When: one tick.
  - Then: at least one differentiator
    (`Button` / `Interaction` / `BackgroundColor` /
    `BorderColor` / `TextColor`) differs.

- **AC6 — Chip-only confirmation guard**:
  - Given: a `LobbyClassPickerCell` receives
    `Interaction::Pressed`.
  - When: the lobby command dispatcher runs.
  - Then: no `LobbyCommand::ConfirmClass` written by the
    cell press alone.

- **AC7 — Two-client reach**:
  - Given: two simulated clients in the existing
    two-client harness (or documented manual walkthrough).
  - When: Create / Join / SelectClass / Confirm executed
    on both.
  - Then: both clients observe
    `S2CPhaseChanged { phase: DraftInitial }`.

---

## Performance Budget

Per ADR-021 Presentation steady-state budget of `< 1 ms` per
frame. The new bounded-room-list layout policy adds at most
`O(rows)` flex computation; viewport-assertion test code
executes once per fixture. Expected per-frame cost change:
negligible.

---

## Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Worker conflates this row with a `QA-COND-0005` advancement. | Medium | High | "Status / No-Claim Banner" and AC9 / AC15 forbid; reviewer checks `production/qa/accept-risk-registry.md` (or canonical equivalent) is untouched. |
| Worker overflows the room-list with `overflow: Hidden` and silently truncates without indicator. | Medium | Medium | AC4 requires a *bounded* policy; ux-designer consultation chooses scroll vs explicit truncation marker. |
| Worker introduces a chip-only confirmation path inadvertently while refactoring picker-cell interaction. | Low | High | AC6 explicit; integration-test command-buffer drain asserts. |
| Worker edits Story 024 / 025 / 026 files thinking they're "related". | Low | Medium | Out-of-scope rule + reviewer checks unrelated story files unchanged. |
| Worker bumps lobby button height to ≥44 Px to "fix" hit-targets. | Low | High | `QA-COND-0005` accept-risk carve-out preserved verbatim; AC9 + AC15 forbid. |
| Worker activates Sprint 18 as a side effect of `/dev-story` paperwork. | Low | Medium | No-Claim Banner forbids; activation is a separate, explicit prompt. |

---

## Verification (orchestrator-side, before worker dispatch)

These are sanity checks for the orchestrator that emits the
`/dev-story` prompt, NOT for PROMPT 1294 itself (which is
paperwork-only):

- `production/sprint-status.yaml` top-level `sprint:` field reads
  `18` (after Sprint 18 activation) and the row for
  `S18-LOBBY-CONFIRM-CTA-VISIBLE-001` is `ready` at the time
  `/dev-story` is dispatched.
- `production/stage.txt` reads `Polish` and is unchanged.
- `production/sprints/sprint-18.md` shows the ACTIVATED banner
  and includes this row.
- PROMPT 761 `Polish->Release` gate-check FAIL evidence
  preserved.
- `production/qa/qa-plan-sprint-18.md` references this story.
- `/story-readiness` on this story file returns READY against
  the Sprint 18 activation HEAD.
- Stories 024 / 025 / 026 remain Complete on `origin/main`.

---

## Authoring Trail

- 2026-05-18 — PROMPT 1294 — Story file authored as future
  Sprint 18 Wave-A candidate `S18-LOBBY-CONFIRM-CTA-VISIBLE-001`.
  Worktree
  `D:\_DEV\claude-code-game-studios-worktrees\s18-story-authoring-wave-a-1294`,
  branch `work/s18-story-authoring-wave-a-1294`, base
  `origin/main@1345c6b8b1cbd543dbd63d279186c93924ca54db`
  (PROMPT 1285 Sprint 18 plan draft). Files touched by this
  authoring run: this file (NEW), and
  `production/epics/playable-client/EPIC.md` (story-list row
  added). Sibling Wave-A stories
  `S18-HAND-FAN-PASSIVE-CLICK-AFFORDANCE-001` and
  `S18-HAND-FAN-Z-LAYER-AUCTION-001` authored in the same run
  under `production/epics/hand-ui/`. Sprint 18 NOT activated.
  No code change. No `/dev-story`, `/story-readiness`,
  `/story-done`, `/smoke-check`, `/team-qa`, `/gate-check`,
  `/release-check`, `/qa-plan`, `cargo`, or `trunk` command
  run. ADR-002 + ADR-021 binding preserved; Sprint 12 story
  019 disposition preserved; PROMPT 761 gate-check FAIL
  preserved; `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a`,
  `S8-QA-001-W1`, `TQ-S12-C1..C7`,
  `S11-HUD-TIMER-EYEBALL-VISUAL-001` human-blocked state, all
  preserved verbatim.
