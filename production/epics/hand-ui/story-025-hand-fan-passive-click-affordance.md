# Story 025: S18-HAND-FAN-PASSIVE-CLICK-AFFORDANCE-001 -- Passive Hand Click Inspect / Selection Feedback

> **Epic**: Hand UI
> **Story ID**: `S18-HAND-FAN-PASSIVE-CLICK-AFFORDANCE-001`
> **Status**: Draft -- future Sprint 18 candidate; NOT activated
> **Layer**: Presentation / Hand UI (passive click feedback outside Placement)
> **Type**: UI + Integration test (ECS marker + outbound-message assertions)
> **Sprint**: Sprint 18 Wave-A candidate per PROMPT 1287 §5 (NOT activated)
> **Authored**: 2026-05-18 by PROMPT 1294
> **Authoring worktree**: `D:\_DEV\claude-code-game-studios-worktrees\s18-story-authoring-wave-a-1294`
> **Authoring branch**: `work/s18-story-authoring-wave-a-1294`
> **Authoring source-of-truth**: `origin/main@1345c6b8b1cbd543dbd63d279186c93924ca54db` (PROMPT 1285 Sprint 18 plan draft)
> **Source reports**: PROMPT 1201 HUNT-1201-06; PROMPT 1203 B-1203-PLA-08; PROMPT 1287 §5 Wave-A

---

## Status / No-Claim Banner

PROMPT 1294 authors this story as a **future Sprint 18 Wave-A
candidate**. Sprint 18 is `draft` on `origin/main`
(`production/sprints/sprint-18.md`, authored by PROMPT 1285) and is
**NOT activated** by PROMPT 1294.

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
- repair of the R1 drag-pipeline-dead bug (PROMPT 1127 §R1) — separate
  prompt
- closure of Sprint 12 story 019 underlying drag-runtime question —
  disposition preserved verbatim

ADR-002 + ADR-021 binding preserved: this story is **read-only over
client-side mirrors of authoritative state**
(`Res<HandCardCatalog>`, `Res<HandUiMode>`,
`Res<CurrentClientPhase>`, `Res<HandUiOutboundMessages>`,
per-slot `HandSlotCard`, `FanSlotIndex`). It introduces no new
server-authoritative state, no new Lightyear message, no new
protocol shape, and no client-side authority over
stage / activate / submit. The existing `C2SActivateCard`
emission path (Story 012 `S18-UI-HAND-ACTIVATION-LOCK-001`)
remains the only authoritative client-to-server card
activation channel.

---

## Source Findings

PROMPT 1201 (multiplayer hunt audit) records the passive-hand
click affordance gap as a P0 hunt row:

- **HUNT-1201-06** (P0): in `RoundPhase::DraftShop`
  (`HandUiMode::Passive`), clicking a hand card silently emits
  `C2SActivateCard` with no preceding visible inspect /
  selection feedback. The player has no on-screen indication
  that a click will cause card activation; mis-clicks therefore
  produce server-acknowledged activations the player did not
  intend.
- **HUNT-1201-06** also flags that in `RoundPhase::DraftAuction`
  (`HandUiMode::PassiveLocked`), the hand fan visually accepts
  pointer hover and pressed states (existing
  `interaction_states::HOVER_*` / `PRESSED_*` tokens fire) even
  though `allows_activation()` is `false`. The player reads the
  visual feedback as "clickable" and is confused when the click
  produces no observable result.

PROMPT 1203 (placement audit, B-class bug taxonomy):

- **B-1203-PLA-08**: a single passive click during DraftShop
  triggers `C2SActivateCard` immediately on the first press,
  with no inspect / confirmation step. The activation lock
  (Story 012) prevents *follow-up* activations during the
  server round-trip, but it does not prevent the *initial*
  unintended activation. PROMPT 1203 classifies this as a
  missing-feature (designed-out) defect.

PROMPT 1287 §5 Wave-A row pins this story slug:
`S18-HAND-FAN-PASSIVE-CLICK-AFFORDANCE-001`, owner epic
`hand-ui`, likely implementation file
`client/src/ui/hand/mod.rs`.

---

## Problem Class / Prevention Target

**Defect class**: hand fan clicks outside `RoundPhase::Placement`
lack visible inspect / selection feedback and the click
semantics are not explicitly gated. The two visible failure modes
are:

1. **DraftShop (HandUiMode::Passive)**: the first click on a hand
   card silently dispatches `C2SActivateCard`. The player has
   no inspect step in which to confirm intent. Mis-clicks
   produce server-acknowledged activations.
2. **DraftAuction (HandUiMode::PassiveLocked)**: the hand fan
   accepts pointer hover / press visuals but `allows_activation`
   returns `false`, so the click has no effect. The visual
   feedback is misleading.

**Prevention target**: introduce explicit click-intent affordances
on the hand fan during `HandUiMode::Passive` and
`HandUiMode::PassiveLocked` so that:

- A click produces a **visible** inspect / selection state on the
  clicked card BEFORE any `C2SActivateCard` is dispatched, OR
- The click is explicitly gated (no `C2SActivateCard` written,
  visual feedback says "locked / not clickable", existing tween
  states reflect the gate).

The exact mechanism is implementation-prompt discretion within
the existing design-token surface (e.g. a single-click "selected"
state with a follow-up confirm / dismiss; OR a long-press
activation gesture; OR an explicit `[Activate]` mini-CTA that
appears on hover / focus). The BLOCKING contract is the
*observable* property: clicks outside PLACEMENT MUST NOT produce
unintended `C2SActivateCard` emissions, AND every click MUST
produce visible feedback within one tick. Worker discretion may
narrow the BLOCKING scope to `Passive` only (with `PassiveLocked`
gated by the existing `allows_activation()` guard) if
ux-designer signs off — the **observable BLOCKING contract** is
that no unintended activation occurs in either mode AND that
every click produces visible feedback.

---

## Context

### Existing surface (read-only at authoring time)

- **`client/src/ui/hand/mod.rs:183-220`**: `HandUiMode` enum and
  `allows_activation()` — `Passive == true`, `PassiveLocked ==
  false`, all others `false`. The `DraftShop`-time activation
  path goes through `Passive` and currently has no
  click-intent gate.
- **`client/src/ui/hand/mod.rs:2731-2768`**:
  `handle_hand_fan_activate_click_system`. Reads
  `HandFanCardClicked` events; if `mode.allows_activation()`,
  dispatches `C2SActivateCard` via the
  `MessageSender<C2SActivateCard>` query AND appends to
  `HandUiOutboundMessages.activate_cards`. There is no
  intermediate inspect / selection step.
- **`client/src/ui/hand/mod.rs:222-227`**:
  `HandUiOutboundMessages.activate_cards: Vec<C2SActivateCard>`.
  Test-visible outbound-message buffer. Story 012 / Story 020
  integration tests already drain this buffer.
- **`client/src/ui/hand/mod.rs:1340-...`**: existing
  `HandFanCardClicked` event producer (pointer-click
  observer / Bevy picking wiring). Out of scope for editing
  unless the worker chooses an explicit click-intent system
  insertion.
- **Story 012 (`hand-ui/story-012-activation-lock.md`)**:
  activation-lock contract — `OQ8 S2CActivationRejected`
  was registered; the lock prevents follow-up activations
  during the server round-trip. The lock does not prevent
  the *first* unintended click. This story (025)
  complements Story 012 by adding the missing pre-click
  intent gate.
- **`client/src/ui/design_tokens/interaction_states.rs`**:
  `HOVER_BG_TINT_ALPHA`, `HOVER_BORDER_ALPHA`,
  `PRESSED_BG_TINT_ALPHA`, `DISABLED_*` tokens. The
  inspect / selection treatment in this story reuses
  these tokens (no new design tokens are authored).

### GDD / ADR / TR trace

- **GDD**: `design/gdd/hand-ui.md` Rule 3 (Phase Behavior) —
  the DraftShop / DraftAuction passive click semantics
  are an underspecified row. Light addition by `/dev-story`
  optional.
- **ADR-002** (Client-Server Authority): preserved. The
  click-intent gate is a read-only derivation of
  `HandUiMode` / `HandSlotCard` and a write to the
  per-slot inspect-state marker; no S2C / C2S edit.
- **ADR-021** (Presentation Layer Architecture):
  preserved. The inspect / selection state marker is a
  per-slot component (or child node) on existing
  pre-pooled fan-slot entities.
- **TR registry**: may add a new row (`TR-HU-011` —
  *"Passive-hand click intent gate / inspect feedback"*).
  Worker discretion; performed by `/dev-story`, not by
  this authoring run.

### Engine / skills

- **Engine**: Bevy 0.18 (Rust).
- **Mandatory skills**: `liv-bevy-018` for any `.rs` edits.
- **Lightyear**: NOT applicable; `liv-bevy-lightyear`
  NOT activated.

### Control Manifest Rules

- **Required**: A passive hand click in
  `HandUiMode::Passive` (DraftShop) MUST NOT immediately
  produce a `C2SActivateCard` emission. The first click
  must produce a **visible** inspect / selection state on
  the clicked card; activation requires a distinct
  follow-up step (second click on the same slot, or an
  explicit confirm CTA, or a long-press gesture — worker
  discretion within ux-designer sign-off).
- **Required**: A passive hand click in
  `HandUiMode::PassiveLocked` (DraftAuction) MUST NOT
  produce any `C2SActivateCard` emission AND MUST
  produce visible "locked / not clickable" feedback OR
  the click must be explicitly suppressed at the
  picking layer (no hover / press visuals that imply
  clickability).
- **Required**: `HandUiOutboundMessages.activate_cards`
  is the authoritative integration-test surface. The
  BLOCKING assertion is that a single click in
  `Passive` mode produces **zero** new entries in
  `activate_cards` for that tick; a follow-up confirm
  step produces exactly **one** new entry.
- **Required**: The Story 012 `S2CActivationRejected`
  contract (activation lock) is preserved. The
  click-intent gate is **independent of** the activation
  lock — they operate at different layers of the
  pipeline.
- **Required**: Outside PLACEMENT, every click on a
  visible fan-slot entity with `HandSlotCard` produces
  visible feedback within one tick (inspect state
  marker visible, or explicit-gated feedback visible).
  Verified by integration-test ECS query.
- **Required**: The existing `RoundPhase::Placement`
  drag-to-stage flow (`HandUiMode::Staging`) MUST NOT
  regress. Click semantics in `Staging` mode are
  out-of-scope and unchanged.
- **Required**: `liv-bevy-018` skill applied to all
  `.rs` edits.
- **Forbidden**: Mutating server-authoritative state
  from the click-intent gate.
- **Forbidden**: Adding a new Lightyear message
  (`liv-bevy-lightyear` NOT activated).
- **Forbidden**: Editing `shared/src/protocol.rs`,
  the server-side `S2CActivate*` handlers, or any QA /
  sprint / session-state tracker.
- **Forbidden**: Touching `production/sprint-status.yaml`,
  `production/sprints/`, `production/qa/`,
  `production/stage.txt`, `production/session-state/`,
  the PROMPT 761 gate-check artifact, or any `Cargo`
  file.
- **Forbidden**: Editing Story 012
  (`hand-ui/story-012-activation-lock.md`) or Story 020
  (`hand-ui/story-020-hand-drag-state-visuals.md`).
  Both are Complete; this story is **additive** on a
  distinct surface.

---

## Story Classification

**Story type**: UI + Integration test — pre-click
intent-gate state machine + ECS marker assertions +
outbound-message drain assertions.

This is **NOT** a:

- Networking / protocol story (no new C2S / S2C message).
- Final-art story (`PAW-TD-*-a` preserved).
- Accessibility story (`QA-COND-0005` preserved).
- Drag-state visual differentiation re-author (Story 020
  Complete).
- Activation-lock re-author (Story 012 contract
  preserved).
- Runtime-bug-repair story for the R1 drag-pipeline-dead
  classification (PROMPT 1127 §R1 is a separate prompt).

---

## Dependencies

| Dependency | Required posture | Why blocking |
|---|---|---|
| Sprint 18 activation | Required before `/dev-story` | This story is a Sprint 18 candidate. |
| Story 012 (S18-UI-HAND-ACTIVATION-LOCK-001) | Implemented / activation-lock contract preserved | This story complements Story 012 — the click-intent gate is the upstream layer; activation lock is the in-flight layer. |
| Story 020 (S12-UX-HAND-DRAG-STATE-VISUALS-001) | Complete on `origin/main` | The inspect / selection marker MUST use a distinct marker from `DragStateOverlay` so Story 020 AC2 query semantics remain valid. |
| Story 023 (S18-UI-HAND-IDLE-PLAYABLE-AFFORDANCE-001) | Implementation landed on `origin/main` per PROMPT 1287 §2 inventory | The idle-affordance overlay markers (`FanSlotPlayableAffordanceOverlay` / `FanSlotPlayableAffordanceUnaffordableOverlay`) are distinct from the inspect / selection state in this story; both are sibling layers on the same fan-slot pre-pool. Conflict-resolution rule: the inspect / selection marker may temporarily occlude the idle-affordance overlay on a focused slot; idle-affordance treatment resumes on dismiss / blur. |
| Existing `HandUiOutboundMessages` resource | Established | The BLOCKING integration-test surface. |
| Bevy picking / `HandFanCardClicked` event producer | Established | Source of click events; modifications optional. |

This story touches `client/src/ui/hand/mod.rs` (and
possibly a new sibling submodule like
`client/src/ui/hand/passive_click_affordance.rs`). Serialise
against any concurrent worker editing the same module.

---

## Acceptance Criteria

All criteria are independently checkable.

- [ ] **AC1 — Passive click does NOT immediately activate
  (DraftShop)**: GIVEN `Res<HandUiMode> == Passive` AND
  `Res<CurrentClientPhase> == DraftShop` AND a fan-slot has
  `HandSlotCard(card_id)`, WHEN a `HandFanCardClicked`
  event for that slot is consumed (or synthesised in the
  test fixture) AND one `App::update()` tick runs, THEN:
  - `HandUiOutboundMessages.activate_cards` is empty for
    this tick (no new `C2SActivateCard` entry).
  - The clicked slot entity carries a visible inspect /
    selection state marker (worker-named, e.g.
    `HandFanSlotInspectState::Selected` or
    equivalent).
  - Visible feedback is present (worker discretion within
    `interaction_states` tokens).
  Verified by integration-test ECS query + outbound-message
  drain.

- [ ] **AC2 — Follow-up confirm step DOES activate
  (DraftShop)**: GIVEN AC1 state (one slot in inspect /
  selected state), WHEN the worker-defined follow-up
  confirm step is executed (second click on the same slot,
  or an explicit confirm CTA press, or completion of a
  long-press gesture), THEN exactly one new
  `C2SActivateCard { card_id }` entry is appended to
  `HandUiOutboundMessages.activate_cards` for that tick.
  The follow-up mechanism is worker discretion within
  ux-designer sign-off; the **BLOCKING** assertion is that
  a distinct second user action is required.

- [ ] **AC3 — Click in DraftAuction does NOT activate**:
  GIVEN `Res<HandUiMode> == PassiveLocked` AND
  `Res<CurrentClientPhase> == DraftAuction` AND a fan-slot
  has `HandSlotCard(card_id)`, WHEN a `HandFanCardClicked`
  event for that slot is consumed AND one tick runs, THEN
  `HandUiOutboundMessages.activate_cards` is empty for
  this tick AND either:
  - (a) visible "locked / not clickable" feedback is
    present on the clicked slot, OR
  - (b) the click is suppressed at the picking layer (no
    `HandFanCardClicked` event is produced — verified by
    inserting a `Pointer<Click>` and observing zero
    `HandFanCardClicked` reads, if the worker chooses
    this path).
  Worker discretion between (a) and (b); the BLOCKING
  assertion is that no unintended activation occurs.

- [ ] **AC4 — Inspect / selection marker is distinct from
  Story 020 `DragStateOverlay`**: GIVEN the
  post-implementation build, WHEN the inspect /
  selection state marker components are inspected, THEN
  they do NOT include `DragStateOverlay` as a component.
  Story 020 AC2 query
  (`Query<&FanSlotIndex, Without<DragStateOverlay>>`)
  remains semantically intact.

- [ ] **AC5 — Inspect / selection marker is distinct from
  Story 023 `FanSlotPlayableAffordanceOverlay`**: GIVEN
  the post-implementation build, WHEN the inspect /
  selection state marker components are inspected, THEN
  they do NOT match the Story 023 idle-affordance
  markers. The two layers MAY coexist on the same slot
  (the inspect marker takes visual precedence on a
  focused slot). Verified by integration-test ECS query
  + visual sign-off note in the evidence document.

- [ ] **AC6 — PLACEMENT click semantics unchanged**:
  GIVEN `Res<HandUiMode> == Staging` AND
  `Res<CurrentClientPhase> == Placement`, WHEN the
  existing drag-to-stage flow is exercised by the
  pre-existing Story 005 / Story 006 / Story 007 /
  Story 008 integration tests, THEN they all PASS
  unchanged. The click-intent gate is gated on
  `HandUiMode != Staging`.

- [ ] **AC7 — Story 012 activation-lock contract
  preserved**: GIVEN the post-implementation build,
  WHEN the existing activation-lock tests
  (`hand_ui_activation_lock_test.rs`) run, THEN they
  PASS unchanged. The click-intent gate operates
  *before* the activation-lock layer; once a click is
  confirmed (AC2) the existing lock semantics apply.

- [ ] **AC8 — Inspect / selection state is clearable**:
  GIVEN a slot in inspect / selection state, WHEN one
  of the following occurs:
  - the player presses Escape / clicks outside the fan,
  - the player clicks a different slot (inspect moves to
    the new slot; old slot clears),
  - `HandUiMode` transitions away from `Passive`,
  - `RoundPhase` transitions away from `DraftShop`,
  THEN the inspect / selection state marker is removed
  from the previously-inspected slot within one tick.
  Verified by integration-test ECS query.

- [ ] **AC9 — Integration test in
  `tests/integration/hand-ui/`**: GIVEN the
  post-implementation build, WHEN
  `cargo test -p client --test hand_ui_passive_click_affordance_test`
  (or canonical-equivalent path chosen by
  `/dev-story`) runs, THEN it PASSES with at minimum
  9 assertions covering AC1, AC2, AC3, AC4, AC5, AC6,
  AC7, AC8. The test drives state via direct resource
  insertion (set `HandUiMode`, set
  `CurrentClientPhase`, write `HandSlotCard`,
  synthesise `HandFanCardClicked`). No raw
  `Pointer<*>` event synthesis is required.

- [ ] **AC10 — Targeted regressions pass**: GIVEN the
  post-implementation build, WHEN run, THEN all PASS:
  - `cargo test -p client --lib`
  - `cargo test -p client --test hand_ui_activation_lock_test`
  - `cargo test -p client --test hand_ui_drag_state_visuals_test`
  - `cargo test -p client --test hand_ui_drag_to_board_cell_test`
  - `cargo test -p client --test hand_ui_submit_prevalidation_test`
  - `cargo test -p client --test hand_ui_reserve_mana_strip_test`
  - `cargo test -p client --test hand_ui_placement_staged_disclosure_accessibility_test`
  - `cargo test -p client --test hand_ui_placement_timer_test`
  - `cargo test -p client --test hand_ui_placement_unstaging_test`
  - `cargo test -p client --test hand_ui_plugin_scaffold_test`
    (entity-count assertion updated if marker / overlay
    children are added).

- [ ] **AC11 — ADR-002 + ADR-021 binding preserved**:
  GIVEN the post-implementation build, WHEN inspected,
  THEN:
  - The new system reads
    `Res<HandUiMode>` (immutable),
    `Res<CurrentClientPhase>` (immutable), per-slot
    `&FanSlotIndex` / `&HandSlotCard` (immutable);
    writes only the inspect / selection state markers
    and `HandUiOutboundMessages` (existing surface,
    write-through preserved for the AC2 follow-up
    case).
  - No new `S2C*` / `C2S*` message;
    `shared/src/protocol.rs` diff is empty.
  - `liv-bevy-lightyear` NOT activated.

- [ ] **AC12 — No `production/` shared-tracker edits**:
  GIVEN the implementation commit, WHEN
  `production/sprint-status.yaml`,
  `production/sprints/`, `production/qa/`,
  `production/stage.txt`,
  `production/session-state/`, and the PROMPT 761
  gate-check artifact are diffed, THEN none is
  modified by this story's implementing prompt
  except in the `/story-done` paperwork commit.

- [ ] **AC13 — Carried conditions preserved**: GIVEN
  the evidence and the implementation commit, WHEN
  inspected, THEN no claim is made against any of:
  `S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006`,
  `PAW-TD-*-a`, `S11-HUD-TIMER-EYEBALL-VISUAL-001`
  (human-blocked), PROMPT 761 `Polish->Release`
  gate-check retry, stage advance from `Polish`, R1
  (drag-pipeline-dead bug), or Sprint 12 story 019
  underlying drag-runtime question.

- [ ] **AC14 — Story 023 idle-affordance overlay
  coexistence**: GIVEN a fan slot that is BOTH (a)
  Story 023 idle-Playable AND (b) inspected /
  selected by this story, WHEN one tick runs, THEN
  the inspect / selection visual takes precedence on
  the focused slot AND the idle-affordance overlay
  Visibility is preserved (worker discretion: the
  idle overlay may remain Visible *under* the
  inspect overlay, OR may be Hidden while the slot
  is inspected; the BLOCKING assertion is that no
  state corruption occurs on dismiss / blur —
  AC8 verifies clearability).

---

## Likely Files (for the future /dev-story — DO NOT EDIT IN THIS AUTHORING RUN)

| Path | Anticipated change |
|------|--------------------|
| `client/src/ui/hand/mod.rs` | Insert the click-intent gate system upstream of `handle_hand_fan_activate_click_system`; or wrap that system in an explicit `if mode.allows_immediate_activation()` branch. Add the inspect / selection state marker components. Bump `HAND_UI_ENTITY_COUNT` if the inspect overlay is realised as a child node per slot. |
| `client/src/ui/hand/passive_click_affordance.rs` (NEW, recommended) | Host the new state-machine + marker components. Re-exported from `mod.rs`. |
| `tests/integration/hand-ui/hand_ui_passive_click_affordance_test.rs` (NEW) | Integration test per AC9. ECS-query-driven; drives state via direct resource insertion + synthesised `HandFanCardClicked`. |
| `tests/integration/hand-ui/hand_ui_plugin_scaffold_test.rs` (existing) | Update `HAND_UI_ENTITY_COUNT` expectation if applicable (AC10). |
| `design/gdd/hand-ui.md` | Optional light addition: one row documenting the passive click intent gate. Worker discretion. |
| `docs/architecture/tr-registry.yaml` | Optional addition: `TR-HU-011 — Passive-hand click intent gate / inspect feedback`. Worker discretion. |
| `production/qa/evidence/sprint-18-hand-passive-click-affordance/README.md` (NEW) | ux-designer consultation note (chosen confirm mechanism); integration-test pass output; carried-conditions no-claim restatement. |
| This story file | Status flipped Draft → Ready by `/story-readiness` post Sprint 18 activation; Ready → Done by `/story-done`. |
| `production/epics/hand-ui/EPIC.md` | `/story-done` paperwork: refresh story row. PROMPT 1294 authoring adds the row in `Draft`. |

**Explicitly out of scope for the `/dev-story` worker**:

- `production/epics/hand-ui/story-012-activation-lock.md`
  (Story 012 complete or in flight; preserve contract).
- `production/epics/hand-ui/story-020-hand-drag-state-visuals.md`
  (Story 020 Complete; NO EDIT).
- `production/epics/hand-ui/story-023-hand-idle-playable-affordance.md`
  (Story 023 implementation landed; NO EDIT).
- `shared/src/protocol.rs`, `server/`, `client/src/network/`.
- `production/sprints/*`, `production/sprint-status.yaml`,
  `production/stage.txt`, PROMPT 761 gate-check artifact.
- `Cargo.toml`, `Cargo.lock`, `.cargo/`, `.github/`.
- R1 drag-pipeline-dead repair, R2 mana-preview, R3
  idle-affordance authoring (separate prompts /
  separate stories).

---

## Out of Scope

- Server-side change.
- Protocol change (`shared/src/protocol.rs`).
- New Lightyear message.
- New client-side authority or optimistic mutations.
- Standard-tier accessibility (`QA-COND-0005`).
- Final-art (`PAW-TD-*-a`).
- Drag-time click semantics (covered by Story 020 +
  Story 017 + Story 022).
- DraftInitial grid click semantics (covered by
  Story 004 — `HandUiMode::Grid` has its own
  purchase flow).
- Opponent-hand click affordance (local-only scope).
- Repair of R1 drag-pipeline-dead bug (separate
  prompt).
- Sprint 18 activation.
- `/qa-plan sprint-18` authoring.
- `/dev-story`, `/story-readiness`, `/story-done`
  on this story under the authoring prompt.
- Polish → Release gate-check retry.
- Stage advance from Polish.

---

## QA Test Cases

*Drafted by qa-lead at story creation. The developer
implements against these — do not invent new test
cases during implementation.*

- **AC1 — Passive click does not immediately activate**:
  - Given: PLACEMENT-disjoint state — `HandUiMode::Passive`,
    `CurrentClientPhase::DraftShop`, slot 0 holds an
    Instant-typed card.
  - When: `HandFanCardClicked { card: slot_0_entity }` is
    written; one tick.
  - Then: `HandUiOutboundMessages.activate_cards.is_empty()`;
    slot 0 carries the inspect / selection state marker.

- **AC2 — Follow-up confirm DOES activate**:
  - Given: AC1 post-state.
  - When: the worker-defined follow-up confirm step is
    triggered (e.g. a second
    `HandFanCardClicked { card: slot_0_entity }` event, OR
    a synthesised `HandFanConfirmActivation` event); one
    tick.
  - Then: `HandUiOutboundMessages.activate_cards.len() == 1`
    with `card_id == slot_0_card_id`.

- **AC3 — DraftAuction click does NOT activate**:
  - Given: `HandUiMode::PassiveLocked`,
    `CurrentClientPhase::DraftAuction`, slot 0 holds a
    card.
  - When: `HandFanCardClicked` for slot 0; one tick.
  - Then: `activate_cards.is_empty()`; either the inspect
    overlay shows locked feedback OR the click is
    suppressed at the picking layer.

- **AC4 — Marker disjoint from `DragStateOverlay`**:
  - Given: post-implementation build.
  - When: ECS query
    `Query<&FanSlotIndex, (With<HandFanSlotInspectState>,
    With<DragStateOverlay>)>`.
  - Then: empty result (no overlap).

- **AC5 — Marker disjoint from
  `FanSlotPlayableAffordanceOverlay`**:
  - Given: post-implementation build.
  - When: ECS query asserts the inspect-state marker is
    not on entities carrying the affordance overlay
    markers.
  - Then: distinct markers verified.

- **AC6 — PLACEMENT unchanged**:
  - Given: `HandUiMode::Staging`,
    `CurrentClientPhase::Placement`.
  - When: existing PLACEMENT integration tests run.
  - Then: all PASS unchanged.

- **AC7 — Activation-lock test unchanged**:
  - When: `hand_ui_activation_lock_test` runs.
  - Then: PASS unchanged.

- **AC8 — Inspect state clearable**:
  - Given: slot 0 inspected; click on slot 1 (or Escape).
  - When: one tick.
  - Then: slot 0 inspect marker removed; slot 1 inspect
    marker present (if click target).

---

## Performance Budget

Per ADR-021 Presentation steady-state budget of `< 1 ms` per
frame. The click-intent gate is `O(1)` per click event; the
inspect-state marker management is `O(fan slots) = O(10)`
per tick. Expected per-frame cost: `< 20 µs`. Well within
budget.

---

## Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Worker conflates this story with the R1 drag-pipeline-dead bug repair. | Low | High | "No-Claim Banner" and "Out of Scope" disjoin R1; AC9 drives state via direct resource insertion. |
| Worker edits Story 012 to "amend activation-lock semantics" instead of adding the upstream click-intent gate. | Medium | High | "Forbidden Control Manifest Rules" and AC7 BLOCKING gate verifies activation-lock regression PASSES. |
| Worker adds `DragStateOverlay` to the new inspect-state marker (contaminating Story 020 query). | Low | High | AC4 BLOCKING; ECS query gate catches at test runtime. |
| Worker introduces a new Lightyear message to "ack inspect intent". | Low | High | AC11 + AC9 forbid; `liv-bevy-lightyear` NOT activated. |
| Worker activates Sprint 18 as a side effect of `/dev-story` paperwork. | Low | Medium | No-Claim Banner forbids. |
| Worker breaks the existing PLACEMENT drag-to-stage flow when adding the click-intent gate. | Medium | High | AC6 BLOCKING; full hand-ui regression suite (AC10) must PASS. |
| Worker chooses an "explicit confirm CTA" that overlaps the Confirm-class CTA in DraftShop UI. | Medium | Medium | ux-designer consultation BLOCKING in AC2; visual capture sign-off in evidence. |

---

## Verification (orchestrator-side, before worker dispatch)

These are sanity checks for the orchestrator that emits the
`/dev-story` prompt, NOT for PROMPT 1294 itself:

- `production/sprint-status.yaml` top-level `sprint:` field reads
  `18` after Sprint 18 activation; row for this story is `ready`.
- `production/stage.txt` reads `Polish` and is unchanged.
- `production/sprints/sprint-18.md` shows the ACTIVATED banner
  and includes this row.
- PROMPT 761 `Polish->Release` gate-check FAIL preserved.
- `production/qa/qa-plan-sprint-18.md` references this story.
- `/story-readiness` on this story returns READY.
- Story 012 / Story 020 / Story 023 unchanged on `origin/main`.
- R1 drag-pipeline-dead bug repair status: this story is
  independent (AC9 drives state via direct resource insertion).

---

## Authoring Trail

- 2026-05-18 — PROMPT 1294 — Story file authored as future
  Sprint 18 Wave-A candidate
  `S18-HAND-FAN-PASSIVE-CLICK-AFFORDANCE-001`. Worktree
  `D:\_DEV\claude-code-game-studios-worktrees\s18-story-authoring-wave-a-1294`,
  branch `work/s18-story-authoring-wave-a-1294`, base
  `origin/main@1345c6b8b1cbd543dbd63d279186c93924ca54db`
  (PROMPT 1285 Sprint 18 plan draft). Files touched by this
  authoring run: this file (NEW) and
  `production/epics/hand-ui/EPIC.md` (story-list row added).
  Sibling Wave-A stories
  `S18-LOBBY-CONFIRM-CTA-VISIBLE-001` and
  `S18-HAND-FAN-Z-LAYER-AUCTION-001` authored in the same run
  (the lobby row under `production/epics/playable-client/`,
  the z-layer row under `production/epics/hand-ui/`). Sprint
  18 NOT activated. No code change. No `/dev-story`,
  `/story-readiness`, `/story-done`, `/smoke-check`,
  `/team-qa`, `/gate-check`, `/release-check`, `/qa-plan`,
  `cargo`, or `trunk` command run. ADR-002 + ADR-021 binding
  preserved; Sprint 12 story 019 disposition preserved;
  PROMPT 761 gate-check FAIL preserved; `QA-COND-0005`,
  `QA-COND-0006`, `PAW-TD-*-a`, `S8-QA-001-W1`,
  `TQ-S12-C1..C7`,
  `S11-HUD-TIMER-EYEBALL-VISUAL-001` human-blocked state,
  all preserved verbatim. R1 drag-pipeline-dead repair
  status unchanged (separate prompt).
