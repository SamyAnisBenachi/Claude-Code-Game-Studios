# Story 024: S12-UX-LOBBY-LAYOUT-MODAL-001 -- Lobby Layout Modal (First-Impression Surface)

> **Epic**: Playable Client
> **Story ID**: S12-UX-LOBBY-LAYOUT-MODAL-001
> **Status**: Draft -- Sprint 14 candidate (per `docs/ux/ui-clean-pass-roadmap.md` rank 12, Tier 1, Must), NOT activated
> **Layer**: Lobby UI / UX (Client)
> **Type**: Integration -- net-new lobby root layout (replaces top-left 420px column with a producer-chosen modal-panel or full-viewport hero composition); paired with viewport-invariant test + visual evidence
> **Sprint**: Sprint 14 candidate (per `docs/ux/ui-clean-pass-roadmap.md` 14-slug Sprint 14+ MVP sequence rank 12; PROMPT 802 §6 Lane C). **NOT** activated by this authoring run.
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
- Pull this row, or any of the 14 PROMPT 802 candidate slugs, into any
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
conformance (WCAG contrast, ≥44px hit-targets, full keyboard navigation,
screen-reader support, colorblind modes, text scaling). Hit-target work
is delegated to the paired story `S11-UX-LOBBY-BUTTON-HITTARGETS`
(story 026), itself accept-risk against `QA-COND-0005`.

**ADR-002 binding preserved**: server-authoritative client-server
authority is unchanged. This story is composition / hierarchy /
typography / responsive-layout work only. No new client-side state
machine, no client-side optimistic authority, no protocol change, no
server-side change. The lobby state machine (`LobbyViewState`,
`LobbyInputState`, `S2CClassLocked` / `S2CSessionReady` drains, lobby
button intent emission via `LobbyCommand`) is preserved verbatim from
Sprint 12 story 013 + Sprint 13 story 023's text-differentiation work.

---

## Source Finding

- PROMPT 802 audit §3.1 L1 + L4 verdict: lobby is **ROUGH-BORDERING-
  UNACCEPTABLE** as a polished friend-game-product UI. The lobby root
  is an absolute-positioned `420×?? Px` column anchored at top-left
  `Val::Px(24.0), Val::Px(24.0)` on a 1920×1080 viewport
  (`client/src/ui/lobby.rs:828-858`). The remainder of the viewport is
  blank.
- PROMPT 802 §3.1 L4: slot panels and "Confirming..." button render as
  siblings *below* the confirm action, breaking left-to-right read
  order; the slot affordance reads as below-the-fold relative to the
  primary action.
- PROMPT 802 §3.1 §1 verdict driver: "the friend-game first-impression
  surface fails the polish bar before play begins."
- `docs/ux/ui-clean-pass-roadmap.md` ranks this as `S12-UX-LOBBY-LAYOUT-
  MODAL-001` rank 12 (Tier 1, Must, 1.0d) and explicitly names it among
  the 4 highest-impact rows for Sprint 14 Must Have framing.
- PROMPT 802 §9 producer-decision-3 explicitly gates this row on a
  producer pick between two incompatible candidate layouts:
  - **(a)** centered modal panel like the result screen
    (`client/src/presentation/result_screen.rs:488` — the lone surface
    in the audit that does layout correctly: `Display::Flex`, centered,
    `width: 88%`, `max_width: 860 Px`, `GlobalZIndex(100)`).
  - **(b)** full-viewport hero layout with background art.

---

## Problem Class / Prevention Target

**Defect class**: the lobby root is a small absolute-positioned column
hugging the top-left corner of a `1920×1080` viewport with no responsive
composition and no first-impression visual identity. Children below the
primary CTA (slot panels, room-code chip) render in a read order that
contradicts the player's expected flow (create / join -> class pick ->
confirm -> ready). The class-picker row, class-portrait row, and
selectable class buttons are not visually associated (see paired story
025). Button dimensions are not stable across rebuild (see paired story
026).

**Prevention target**: replace the absolute-positioned 420-Px column
with a centred, responsive composition that:

1. Uses the producer-selected layout direction (modal panel or full-
   viewport hero) — see Producer Decision section below.
2. Fits the `1920×1080` viewport without overflow and without leaving
   the viewport visually unframed.
3. Fits the `1366×768` viewport without overflow, without clipping the
   primary CTA, without orphaning the room-code chip, and without
   breaking the read order.
4. Composes children via flex (not absolute) so that resize, font-scale,
   or text-length changes do not visibly shift z-order or read order.
5. Establishes a clear visual hierarchy: room-code + status banner ->
   create/join controls -> class-picker (paired with portraits) ->
   confirm CTA -> waiting state. Slot panels and room-code chip do
   **not** render *below* the confirm CTA.
6. Reserves `GlobalZIndex` (rank 1 / `S11-TD-UI-ZINDEX-LAYERS`) for the
   lobby modal layer, even if that rank ships first — this story does
   not author the layer enum but is a downstream consumer of it. If
   the lobby surface is activated before `S11-TD-UI-ZINDEX-LAYERS`
   lands, the implementing prompt MUST flag the dependency and either
   wait or use a local placeholder `GlobalZIndex` with a `TODO`
   reference to the canonical layer enum (re-fix story).

---

## Producer Decision -- Modal Panel vs Full-Viewport Hero

> **Status**: Open. Must be resolved by the producer (with ux-designer
> + art-director consultation) **before** the implementing prompt
> enters `/dev-story` on this row. The decision is the same gate
> recorded as `PROMPT 802 §9 producer-decision-3` and is restated here
> as a non-derivable input to this story.

### Option A -- Centred Modal Panel (analogous to `result_screen.rs`)

| Aspect | Value |
|---|---|
| Root composition | `Display::Flex`, `align_items: Center`, `justify_content: Center` over a full-viewport parent that owns a dim background and `GlobalZIndex`. |
| Panel sizing | `width: Val::Percent(88.0)`, `max_width: Val::Px(860.0)`, `max_height: Val::Percent(92.0)` (mirrors result-screen literals; final numbers locked by ux-designer per S12-UX-GLOBAL-UI-DESIGN-SPEC-001 once authored, friend-game-tier numbers acceptable in absence of that spec). |
| Background | Dim overlay (alpha owned by `S12-TD-UI-OVERLAY-ALPHA-TOKEN-001` once authored; friend-game-tier `0.45..0.58` literal acceptable in absence of that token, with `TODO`). |
| Pros | Reuses the only surface in the audit that already does layout correctly. Smallest visual-design surface area to lock. Composes cleanly with future modal stacking (e.g. settings modal). |
| Cons | Less first-impression "wow" than a hero layout. No background art slot. |
| Affects | `client/src/ui/lobby.rs` root node + immediate children. Minimal asset work. |

### Option B -- Full-Viewport Hero Layout (with background art)

| Aspect | Value |
|---|---|
| Root composition | Full-viewport flex container with an `ImageNode` background art layer (size: `Val::Percent(100.0)`), foreground content composed in centred / right-aligned / left-aligned regions per art-director spec. |
| Panel sizing | Foreground form fields use a centred or right-aligned column at `width: clamp(420..640 Px, ~40% viewport)`; class-picker uses a wider grid (paired story 025). |
| Background | Background art slot (asset to be wired by art-director, likely a new `PAW-TD-006-c` slot; friend-game-tier placeholder PNG acceptable in absence of final art, with `TODO` reference to `PAW-TD-*-a` accept-risk). |
| Pros | Larger first-impression visual identity. Room for class portraits as hero figures (couples cleanly with paired story 025 class-picker hierarchy). |
| Cons | Larger visual-design surface area (background art slot, art-director spec dependency). Higher coupling to `PAW-TD-*-a` placeholder-art accept-risk. |
| Affects | `client/src/ui/lobby.rs` root + likely new background art wiring (asset wiring is **not** authored here; this story is layout-only). |

### Decision Capture

The implementing prompt MUST:

- Record the producer's chosen option (A or B) in the evidence document
  before opening the worktree, with the producer agent / human name and
  the date.
- If Option B is chosen, also record the art-director's stance on
  whether a placeholder background asset is acceptable for friend-game
  scope (or whether the surface must wait for a final-art asset). A
  placeholder is acceptable per `PAW-TD-*-a` accept-risk; the story
  does not regress this disposition.
- If the producer chooses an option **outside A or B** (e.g. a hybrid
  modal-over-hero, a tabbed lobby, a split-screen lobby), the
  implementing prompt MUST stop and reroute to PROMPT 802 §9 producer-
  decision-3 for re-scope. This story file is not authoritative for
  any layout outside Option A / Option B.

---

## Context

### Existing surface (read-only at authoring time)

- **`client/src/ui/lobby.rs:828-1050`** (`spawn_lobby_ui_system`): the
  authoritative lobby spawn function. Current shape: `LobbyRoot` node
  at `PositionType::Absolute`, `left: Val::Px(24.0)`,
  `top: Val::Px(24.0)`, `width: Val::Px(420)`, `max_width: 92%`,
  `flex_direction: Column`. Children spawn in order: status banner,
  room-code chip, create/join row, slot row, class label + class
  picker (wrap row of 7 buttons), confirm button, portraits row, slot
  panels row, room-code chip image. Per PROMPT 802 §3.1 L4, the
  portraits / slot panels / room-code chip image render *below* the
  confirm CTA — read order is broken.
- **`client/src/presentation/result_screen.rs:488-608`**: the reference
  layout that this story emulates if Option A is selected. `Display::
  Flex`, centred, `width: 88%`, `max_width: 860 Px`, `GlobalZIndex(100)`.
- **Lobby state**: `LobbyViewState`, `LobbyInputState`, `S2CClassLocked`
  / `S2CClassesRevealed` / `S2CSessionReady` drains, `LobbyCommand`
  intent emitter. All preserved verbatim by this story.

### GDD / ADR / TR trace

- **GDD**: `design/gdd/game-session-system.md` (lobby flow);
  `design/gdd/hand-ui.md` (UX text owned by ux-designer; the lobby
  inherits the same friend-game-tier text-style discipline pending
  `S12-UX-GLOBAL-UI-DESIGN-SPEC-001`).
- **ADR-002** (Client-Server Authority): no client-side state-machine
  authority added. Composition / hierarchy / typography only.
- **ADR-008** (Lightyear Channel Configuration): no channel change.
- **ADR-012** (SessionReady Delivery): no change to the SessionReady
  Observer.
- **ADR-021** (Presentation Layer Architecture): preserved. Lobby
  remains a read-only presentation of `LobbyViewState` projected from
  server-authoritative messages.
- **TR registry**: no new TR (UX composition only).

### Engine / skills

- **Engine**: Bevy 0.18 (Rust).
- **Mandatory skills**: `liv-bevy-018` (any `.rs` edit in the lobby UI
  module).
- **Mandatory skills**: `liv-bevy-lightyear` (only if the touched lobby
  code imports `lightyear` directly; the current `spawn_lobby_ui_system`
  does not, but adjacent drain systems do).

### Control Manifest Rules

- Required: lobby root composes children via flex (not absolute) so
  that resize and font-scale changes do not break z-order or read
  order, except for an outer full-viewport parent that owns the modal
  layer or the hero background.
- Required: lobby renders without overflow at `1920×1080` and at
  `1366×768`.
- Required: lobby read order top-to-bottom (and / or left-to-right per
  Option B's art-director spec) is: status / room-code -> create-join
  -> class picker (paired with portraits per story 025) -> confirm CTA
  -> waiting / slot status. Slot panels and room-code chip image MUST
  NOT render *below* the confirm CTA.
- Required: no overlap between siblings at either viewport. Text fits
  inside its parent button at the canonical font sizes (current friend-
  game-tier literals in `client/src/ui/lobby.rs` may be adjusted by the
  implementing prompt only with ux-designer sign-off).
- Required: button dimensions are stable across rebuild (paired story
  026 owns the dimension-stability invariant; this story consumes it).
- Forbidden: new client-side lobby state authority, new optimistic
  client mutations of class lock / slot assignment, or new protocol
  shape.
- Forbidden: any edit to `shared/`, `server/`, `tests/`,
  `production/sprint-status.yaml`, `production/sprints/`,
  `production/qa/`, `production/session-state/`,
  `production/stage.txt`, the PROMPT 761 gate-check artifact, or any
  `Cargo` file.
- Forbidden: claiming Standard-tier accessibility conformance, claiming
  `QA-COND-0005` advanced, claiming `PAW-TD-*-a` resolved, or claiming
  `Polish->Release` gate-check retry.

---

## Story Classification

**Story type**: Integration -- targeted lobby UI composition edit +
viewport-invariant test + visual capture. Friend-game scope only.

This is **NOT** a:

- Pure UX-spec story (real client UI lands).
- Server-side change.
- Protocol change.
- Accessibility-tier (Standard-tier) repair.
- Final-art asset wiring.

---

## Acceptance Criteria

All criteria are independently checkable. ACs marked **Producer-gated**
are only resolvable once the Producer Decision section above is
recorded.

- [ ] **AC1 -- Producer decision recorded (Producer-gated)**: GIVEN
  the implementing prompt's evidence document, WHEN read at the top,
  THEN it records the chosen layout option (A: centred modal panel, B:
  full-viewport hero) with the producer name + date + ux-designer +
  art-director consultation note. Reroute applies if neither A nor B
  was chosen.

- [ ] **AC2 -- Lobby root composes via flex (not absolute)**: GIVEN
  `client/src/ui/lobby.rs` `spawn_lobby_ui_system` after the
  implementation, WHEN inspected, THEN the lobby root inhabits a full-
  viewport flex container (`Display::Flex`, viewport-anchored width /
  height), and the inner lobby panel (Option A) or hero foreground
  (Option B) is composed via flex children with no `PositionType::
  Absolute` on the primary form column. The current top-left
  `Val::Px(24.0)` anchor pattern is removed.

- [ ] **AC3 -- 1920×1080 responsive fit**: GIVEN the lobby UI rendered
  at viewport size `1920×1080`, WHEN captured, THEN (a) no sibling
  overlaps another sibling; (b) all text fits inside its parent button
  without truncation, ellipsis insertion, or single-line overflow at
  the canonical font size; (c) the lobby panel (A) is visibly centred
  or the hero composition (B) fills the viewport without seams;
  (d) class portraits are visually associated with selectable class
  buttons per paired story 025; (e) the read order top-to-bottom is
  status / room-code -> create-join -> class picker -> confirm ->
  waiting / slot status.

- [ ] **AC4 -- 1366×768 responsive fit**: GIVEN the lobby UI rendered
  at viewport size `1366×768`, WHEN captured, THEN (a)-(e) of AC3 hold;
  (f) no clipping of the primary CTA, room-code chip, or class picker;
  (g) Option-A panel scales via `max_width` / `Percent` rules; Option-B
  hero scales without orphaning foreground content off-viewport.

- [ ] **AC5 -- Read order preserved across resize**: GIVEN the lobby
  UI at any viewport in `[1366×768, 1920×1080]`, WHEN resized between
  the two endpoints during a single session, THEN the read order from
  AC3(e) is preserved; no child re-renders below the confirm CTA; no
  z-order flip is observable.

- [ ] **AC6 -- Stable button dimensions across rebuild**: GIVEN repeat
  spawns of the lobby root (e.g. exit-and-re-enter the `ClientState::
  Lobby` state), WHEN button widths and heights are sampled across the
  rebuild, THEN they match the canonical dimensions from paired story
  026 (`S11-UX-LOBBY-BUTTON-HITTARGETS`) within 1 Px tolerance. Paired
  story 026 owns the canonical dimensions; this story consumes them.

- [ ] **AC7 -- Class-picker hierarchy preserved**: GIVEN paired story
  025 (`S11-UX-LOBBY-CLASS-PICKER`) has landed (or lands in the same
  sprint), WHEN this story's repair lands, THEN the class portrait
  row + class button row + class label are composed as a single
  hierarchical region per story 025's spec (not as three independent
  siblings). If story 025 has not landed first, this story MUST flag
  the dependency and either wait or stage the hierarchy with a `TODO`
  reference to story 025.

- [ ] **AC8 -- Viewport-invariant test added**: GIVEN a new or extended
  test under `tests/integration/playable_client/` (canonical path
  chosen by the implementing prompt, e.g.
  `lobby_layout_viewport_invariant_test.rs`), WHEN the test asserts
  lobby children fit within `1366×768` and `1920×1080` viewports
  without overflow and without sibling overlap, THEN the test passes.
  This is a layout invariant only; "looks right" judgement belongs in
  the visual evidence capture. (Foundational viewport-invariant test
  bin authoring is owned by `S11-TD-UI-VIEWPORT-INVARIANT-TESTS` rank
  4; this story may consume that bin if it has landed, or may author a
  standalone test if not.)

- [ ] **AC9 -- No client-side authority added (ADR-002)**: GIVEN the
  implementation diff, WHEN reviewed, THEN no client-side mutation of
  class-lock, slot-assignment, or session-ready state is introduced
  outside the existing `S2CClassLocked` / `S2CSlotUpdated` /
  `S2CSessionReady` drain paths. No protocol shape change in
  `shared/src/protocol.rs`. No server-side change.

- [ ] **AC10 -- ux-designer consultation recorded**: GIVEN the
  implementation prompt's first ux-designer interaction, WHEN final
  literals (panel max-width, padding, gap, font sizes if changed) are
  locked, THEN the consultation note and chosen literals are recorded
  in the evidence document. Friend-game-tier literals are acceptable;
  Standard-tier accessibility conformance is **not** claimed.

- [ ] **AC11 -- Workspace test pass**: GIVEN
  `cargo test --workspace --tests --no-fail-fast` at the
  implementation commit, WHEN compared to the post-Sprint-13 baseline,
  THEN no new `#[ignore]` markers are introduced; the new
  viewport-invariant test passes; previously-passing tests continue
  to pass.

- [ ] **AC12 -- No `production/` shared-tracker edits, no sprint
  advance**: GIVEN the implementation commit, WHEN
  `production/sprint-status.yaml`, `production/sprints/sprint-13.md`,
  `production/sprints/sprint-14.md` (if it exists by then),
  `production/stage.txt`, `production/session-state/`,
  `production/qa/`, and the PROMPT 761 gate-check artifact are diffed,
  THEN none of them is modified by this story's implementing prompt
  except in the `/story-done` paperwork commit (which is a separate
  prompt scope).

- [ ] **AC13 -- Friend-game-scope no-claim restated in evidence**:
  GIVEN the evidence document, WHEN read at the bottom, THEN it
  verbatim restates the friend-game-scope-only disposition and the
  non-claims list from this story's Status / No-Claim Banner above:
  no public release readiness, no Standard-tier accessibility
  completion, no `QA-COND-0005` advancement, no `QA-COND-0006`
  advancement, no `S8-QA-001-W1` closure, no `PAW-TD-*-a` resolution,
  no `Polish->Release` gate-check retry.

---

## Likely Files

| Path | Anticipated change |
|------|--------------------|
| `client/src/ui/lobby.rs` (canonical path verified by implementing worker) | Edited: `spawn_lobby_ui_system` lobby root composition replaced with the producer-selected modal-panel (Option A) or full-viewport hero (Option B) shape. Inline `Val::Px(24.0)` top-left anchor removed. Children rearranged so portraits + slot panels + room-code chip do not render below the confirm CTA. |
| `tests/integration/playable_client/lobby_layout_viewport_invariant_test.rs` (or canonical equivalent under the rank-4 viewport-invariant test bin) | NEW integration test asserting AC3 / AC4 / AC8. |
| `production/qa/evidence/sprint-14-lobby-layout-modal-evidence.md` (slot reserved; sprint number may be adjusted at activation time) | NEW evidence document: producer decision capture (AC1), ux-designer consultation (AC10), 1920×1080 + 1366×768 captures (AC3 / AC4), viewport-invariant test pass output, no-claim restatement. |
| This story file | Status flip Draft -> Implemented / Done on `/story-done` paperwork. |
| `production/epics/playable-client/EPIC.md` | Status flip on `/story-done` paperwork. |

This table is a planning estimate. The implementing prompt is
authoritative for the realised set.

---

## Required Skills

- **`liv-bevy-018`** -- mandatory for `.rs` edits to `client/src/ui/lobby.rs`.
- **`liv-bevy-lightyear`** -- only if the implementation touches a
  `MessageReceiver<T>` / `MessageSender<T>` drain in the lobby module.
- **`ux-designer` agent** -- mandatory consultation per AC10.
- **`art-director` agent** -- mandatory consultation if Option B is
  chosen (background art slot stance per Producer Decision).

---

## Evidence Path

`production/qa/evidence/sprint-14-lobby-layout-modal-evidence.md`
(slot reserved; sprint number may be adjusted at activation time;
populated by the implementing prompt).

**Required evidence content**:

- Producer decision (Option A or Option B), producer agent / human
  name, date, ux-designer + art-director consultation note (AC1, AC10).
- Diff summary for `client/src/ui/lobby.rs`.
- New viewport-invariant integration-test pass output (AC8 / AC11).
- 1920×1080 visual capture (AC3) and 1366×768 visual capture (AC4),
  saved under
  `production/qa/evidence/captures/sprint-14-lobby-layout-modal/`.
- Read-order trace at both viewports (AC3(e) / AC4 / AC5).
- Cross-link to paired stories 025 (`S11-UX-LOBBY-CLASS-PICKER`) and
  026 (`S11-UX-LOBBY-BUTTON-HITTARGETS`).
- Cross-link to `docs/ux/ui-clean-pass-roadmap.md` (rank 12).
- Cross-link to PROMPT 802 §3.1 L1 / L4 and §9 producer-decision-3.
- Verbatim no-claim restatement (AC13).
- ADR-002 / ADR-008 / ADR-012 / ADR-021 preservation note (AC9).

---

## Regression Commands Expected

For the implementing prompt (not run by PROMPT 880):

- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets`
- `cargo test --workspace --tests --no-fail-fast`
- `cargo test -p client --test lobby_layout_viewport_invariant -- --nocapture`
  (or the new test name)
- `git diff <pre-impl-sha>..<impl-sha> -- 'shared/src/**' 'server/src/**'`
  (verifies AC9: zero protocol-shape change, zero server-side change)
- `git diff --check origin/main...HEAD`
- `git diff --cached --check`

---

## Dependency Notes Against Sprint 14 Pull-In Sequence

Per `docs/ux/ui-clean-pass-roadmap.md` rank-12 dependency line, this
story depends on:

- Rank 1 (`S11-TD-UI-ZINDEX-LAYERS`): for the lobby modal layer's
  `GlobalZIndex`. If rank 1 has not landed first, the implementing
  prompt may use a placeholder `GlobalZIndex(100)` (matching
  result-screen) with a `TODO` reference.
- Rank 3 (`S11-TD-UI-FLEX-STRIPS`): for the flex composition pattern.
  Placeholder inline-flex acceptable if rank 3 has not landed.
- Rank 4 (`S11-TD-UI-VIEWPORT-INVARIANT-TESTS`): for the viewport-
  invariant test bin. Standalone test acceptable if rank 4 has not
  landed.
- Rank 6 (`S12-UX-GLOBAL-UI-DESIGN-SPEC-001`): for canonical literals
  (panel max-width, padding, gap, font sizes). Friend-game-tier
  placeholder literals acceptable if rank 6 has not landed.

This story is **parallel-safe** within Tier 1 with paired stories 025
(`S11-UX-LOBBY-CLASS-PICKER`) and 026 (`S11-UX-LOBBY-BUTTON-HITTARGETS`)
because all three touch the same surface module (`client/src/ui/
lobby.rs`); the Sprint 14 activation prompt must serialise them on
file-scope contention. If activated together, sequence 024 last (so
025's class-picker hierarchy and 026's button dimensions are stable
inputs to 024's root composition).

This story does **not** advance:

- `S8-QA-001-W1` (two-client `GAME_OVER` closure).
- `QA-COND-0005` (Standard-tier accessibility completion).
- `QA-COND-0006` (playtest / fun-hypothesis validation).
- Any `PAW-TD-*-a` placeholder-art accept-risk.
- PROMPT 761 `Polish->Release` gate-check retry.

---

## Out of Scope

- Server-side change.
- Protocol change (`shared/src/protocol.rs`).
- New client-side lobby state authority or optimistic mutations.
- Standard-tier accessibility of the lobby UI (≥44px hit-targets, full
  keyboard navigation, screen-reader support, colorblind modes, text
  scaling, WCAG contrast). Friend-game scope only.
- Final-art asset wiring for the lobby (`PAW-006-*`, `PAW-TD-006-*`
  preserved as accept-risk).
- Authoring the global UI design spec (`S12-UX-GLOBAL-UI-DESIGN-SPEC-
  001`), the z-index layer enum (`S11-TD-UI-ZINDEX-LAYERS`), the
  viewport-invariant test bin (`S11-TD-UI-VIEWPORT-INVARIANT-TESTS`),
  or the flex-strips primitive (`S11-TD-UI-FLEX-STRIPS`). Those are
  separate Sprint 14+ rows.
- Sprint 14 activation. Sprint 13 close-out. `S8-QA-001-W1` closure.
  `Polish->Release` gate-check retry.
- `QA-COND-0005` / `QA-COND-0006` advancement.
- No `/dev-story`, `/story-done`, `/story-readiness`, `/smoke-check`,
  `/team-qa`, `/gate-check`, `/release-check`, or `/qa-plan` under the
  authoring prompt.

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
