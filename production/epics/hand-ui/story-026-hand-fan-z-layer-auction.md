# Story 026: S18-HAND-FAN-Z-LAYER-AUCTION-001 -- Hand Fan Visibility / Z-Layer During DraftAuction

> **Epic**: Hand UI
> **Story ID**: `S18-HAND-FAN-Z-LAYER-AUCTION-001`
> **Status**: Draft -- future Sprint 18 candidate; NOT activated
> **Layer**: Presentation / Hand UI (auction-phase fan z-order / visibility)
> **Type**: UI + Integration test (z-layer / visibility / QA-snapshot assertions)
> **Sprint**: Sprint 18 Wave-A candidate per PROMPT 1287 §5 (NOT activated)
> **Authored**: 2026-05-18 by PROMPT 1294
> **Authoring worktree**: `D:\_DEV\claude-code-game-studios-worktrees\s18-story-authoring-wave-a-1294`
> **Authoring branch**: `work/s18-story-authoring-wave-a-1294`
> **Authoring source-of-truth**: `origin/main@1345c6b8b1cbd543dbd63d279186c93924ca54db` (PROMPT 1285 Sprint 18 plan draft)
> **Source reports**: PROMPT 1201 HUNT-1201-09; PROMPT 1180 H-05; PROMPT 1287 §5 Wave-A

---

## Status / No-Claim Banner

PROMPT 1294 authors this story as a **future Sprint 18 Wave-A
candidate**. Sprint 18 is `draft` on `origin/main`
(`production/sprints/sprint-18.md`, authored by PROMPT 1285) and
is **NOT activated** by PROMPT 1294.

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
- closure of the R1 drag-pipeline-dead bug (PROMPT 1127 §R1)

ADR-002 + ADR-021 binding preserved: this story is composition /
z-order / visibility only. No client-side authority, no protocol
shape change, no server-side change. The shop-auction-ui surface
(`client/src/ui/shop_auction/`) may be touched **only if** the
implementing prompt explicitly chooses an auction-modal z-override
path (see Acceptance Criteria AC9 / AC10 — worker discretion);
the default path is a `HandFanRoot` visibility-only edit confined
to `client/src/ui/hand/mod.rs`.

---

## Source Findings

PROMPT 1201 (multiplayer hunt audit) records the hand-fan /
auction z-layer collision as a P0 hunt row:

- **HUNT-1201-09** (P0): in `RoundPhase::DraftAuction`
  (`HandUiMode::PassiveLocked`), the hand fan
  (`HandFanRoot` at `z_layers::UI_BASE = 300`) remains
  rendered alongside the auction panel
  (`ShopAuctionRoot` also at `z_layers::UI_BASE = 300`).
  Both surfaces compete for the same z-layer, the hand
  fan visually occludes the auction featured-card area
  and bid controls in narrower viewports, and the
  auction surface is the phase-dominant UI by design.
  The collision causes player confusion: the auction
  cards / bid affordance read as half-hidden behind the
  bottom hand band.

PROMPT 1180 (lobby + UI hunt audit) corroborates:

- **H-05**: hand-fan visibility during DraftAuction is
  ambiguous — players expect either a clean
  "hand hidden" state OR a clearly-subordinate
  presentation. The current state (visible + same
  z-layer as the auction panel) is neither.

PROMPT 1287 §5 Wave-A row pins this story slug:
`S18-HAND-FAN-Z-LAYER-AUCTION-001`, owner epic
`hand-ui`, likely implementation files
`client/src/ui/hand/mod.rs` and **possibly**
`client/src/ui/shop_auction/` (only if the worker
explicitly chooses a modal-z override on the auction
side).

---

## Problem Class / Prevention Target

**Defect class**: during `RoundPhase::DraftAuction`, the
hand fan and the auction panel paint at the same
`z_layers::UI_BASE` layer with overlapping screen-space
bounds. The hand fan does not occupy a "subordinate"
visual rank; the player cannot tell which surface is
phase-dominant without reading copy.

**Prevention target**: during `RoundPhase::DraftAuction`,
the auction panel MUST be the visually-dominant UI
surface AND the hand fan MUST NOT occlude the auction
panel's featured-card / bid-controls regions. The chosen
mechanism is worker discretion within one of:

- (a) **Hide the hand fan** during DraftAuction — set
  `HandFanRoot.Visibility = Hidden` (currently the fan
  is `Inherited` while `HandUiMode::PassiveLocked`
  shows the fan slots). The cleanest path; matches
  the `HandUiMode::Hidden` precedent. Worker
  discretion within ux-designer sign-off.
- (b) **Demote the hand fan** to a subordinate z-layer
  (introduce a `z_layers::UI_SUBORDINATE` constant
  below `UI_BASE`, OR keep `UI_BASE` and shift the
  auction panel up to `UI_OVERLAY = 400`). Requires
  shop-auction-ui edit.
- (c) **Shrink and dim** the hand fan to a thin
  bottom-band acknowledgment strip during
  DraftAuction (visible but visually subordinate).
  Requires hand-ui layout edit + shop-auction-ui
  z-coordination.

The BLOCKING **observable** contract is that the
`hand_fan_visible` QA-snapshot field reads `0` (or
equivalent zero-bound) while the auction panel
QA-snapshot fields confirm the auction surface is
present and on-screen. Worker may instead expose a new
boolean snapshot field (e.g.
`hand_fan_occludes_auction = false`) if the chosen
mechanism preserves a visible-but-subordinate fan;
either path is acceptable as long as the BLOCKING
contract is provable from QA snapshot evidence (AC4).

---

## Context

### Existing surface (read-only at authoring time)

- **`client/src/ui/hand/mod.rs:183-220`**: `HandUiMode`
  enum. `HandUiMode::PassiveLocked` (DraftAuction)
  currently sets the hand fan / fan slots to visible
  (`shows_fan_root() == true`,
  `shows_fan_slots() == true`).
- **`client/src/ui/hand/mod.rs:3632-3651`**:
  `HandFanRoot` spawn. The fan root is a child of
  `HandBarRoot` which carries
  `z_layers::UI_BASE = 300` (`mod.rs:3628`). The fan
  inherits z-order from the bar.
- **`client/src/ui/hand/mod.rs:3842`**: a drag-time
  overlay child paints at `z_layers::UI_OVERLAY`.
- **`client/src/ui/design_tokens/z_layers.rs`**:
  `UI_BASE = 300`, `UI_OVERLAY = 400`, `MODAL = 500`.
  Each adjacent layer is separated by 100 integer
  units, "leaving headroom for future intermediate
  layers without re-ordering existing constants".
- **`client/src/ui/shop_auction/mod.rs`**: auction
  panel `ShopAuctionRoot` lives at `UI_BASE` per
  `card_slot.rs:30` (`AuctionFeatured` documented z
  is `UI_BASE`). The auction featured card is
  `380 × 280 px landscape`.
- **`client/src/presentation/qa_snapshot.rs:686`**:
  `pub hand_fan_visible: usize` — count of visible
  hand-fan slot entities in the current snapshot.
  Existing QA-snapshot field consumed by
  Wave-A evidence.
- **`client/src/presentation/qa_snapshot.rs:417,
  426`**: `pub placement_state` and
  `pub auction_state` top-level snapshot keys
  (lifted by PROMPT 1229 / story 023 in
  `ui-clean-pass`). The auction state surface is
  the BLOCKING evidence channel for AC4 / AC5.
- **`design/gdd/hand-ui.md` Rule 3**: documents
  phase-driven hand visibility but does not pin
  the DraftAuction state. This story optionally
  pins it (worker discretion via the
  `/dev-story`).

### GDD / ADR / TR trace

- **GDD**: `design/gdd/hand-ui.md` Rule 3 (Phase
  Behavior). Light addition: one row pinning the
  DraftAuction fan disposition. Worker discretion.
- **GDD**: `design/gdd/shop-auction-ui.md` — auction
  panel phase dominance (existing).
- **ADR-002** (Client-Server Authority): preserved.
- **ADR-021** (Presentation Layer Architecture):
  preserved. The hand fan stays inside bevy_ui
  composition order. If a new `UI_SUBORDINATE`
  z-layer constant is added, it goes alongside
  existing constants in `z_layers.rs`.
- **TR registry**: may add `TR-HU-012 — DraftAuction
  hand-fan subordinate / hidden disposition`.
  Worker discretion.

### Engine / skills

- **Engine**: Bevy 0.18 (Rust).
- **Mandatory skills**: `liv-bevy-018` for any
  `.rs` edits.
- **Lightyear**: NOT applicable;
  `liv-bevy-lightyear` NOT activated.

### Control Manifest Rules

- **Required**: During `RoundPhase::DraftAuction`
  (`HandUiMode::PassiveLocked`), the auction panel
  is visually dominant. Verified by QA-snapshot
  evidence (AC4) AND viewport capture (AC10).
- **Required**: The hand fan MUST NOT occlude the
  auction featured-card region OR the auction
  bid-controls region. The verification path is
  one of:
  - (a) `hand_fan_visible = 0` in the QA snapshot
    while `auction_state` is `Some(_)` and
    contains a featured card.
  - (b) The hand fan is visible but its bevy_ui
    bounds do not overlap the auction panel's
    bevy_ui bounds (verified by `ComputedNode`
    inspection at the canonical viewports).
  Worker discretion between (a) and (b); the
  BLOCKING assertion is that one of these
  conditions holds at every canonical viewport.
- **Required**: The disposition is reversible —
  on transition out of `RoundPhase::DraftAuction`
  (into `Resolution` / `Placement` / etc.), the
  hand fan returns to its pre-DraftAuction
  visibility / layout. Verified by integration
  test driving the phase transition.
- **Required**: The Story 017 / 020 / 022 / 023
  hand-ui contracts (drag MVP, drag-state
  visuals, mana preview during drag, idle
  affordance) MUST NOT regress. None of those
  stories operates during `PassiveLocked`, so
  the change should be naturally isolated;
  AC8 / AC9 BLOCKING gate verifies.
- **Required**: QA snapshot remains the
  authoritative evidence surface. `hand_fan_visible`
  (or the new `hand_fan_occludes_auction` boolean,
  worker discretion) MUST be lifted to a top-level
  snapshot key consumed by automated assertions.
- **Required**: `liv-bevy-018` skill applied to
  all `.rs` edits.
- **Forbidden**: Mutating server-authoritative
  state.
- **Forbidden**: Adding a new Lightyear message.
- **Forbidden**: Editing `shared/src/protocol.rs`,
  the server-side auction / resolution code, or
  any QA / sprint / session-state tracker.
- **Forbidden**: Touching `production/sprint-status.yaml`,
  `production/sprints/`, `production/qa/`,
  `production/stage.txt`,
  `production/session-state/`, the PROMPT 761
  gate-check artifact, or any `Cargo` file.
- **Forbidden**: Re-ordering existing constants in
  `client/src/ui/design_tokens/z_layers.rs`
  (`UI_BASE = 300`, etc. are stable). A new
  `UI_SUBORDINATE` constant MAY be added at a
  free numeric value below `UI_BASE` (e.g. 250)
  per the module's "leaving headroom" rationale.
- **Forbidden**: Editing Story 005 / 010 / 020 /
  022 / 023 file content. Those stories are
  Complete / on-main; this story is **additive**
  on a distinct phase / surface.
- **Forbidden**: Claiming `QA-COND-0005`,
  `QA-COND-0006`, `PAW-TD-*-a`, or
  `S8-QA-001-W1` advancement.

---

## Story Classification

**Story type**: UI + Integration test — phase-driven
visibility / z-order change on `HandFanRoot` (and
optionally `ShopAuctionRoot`) + QA-snapshot evidence
assertion.

This is **NOT** a:

- Networking / protocol story.
- Final-art story.
- Accessibility story (`QA-COND-0005` preserved).
- Drag-state visual differentiation re-author.
- Activation-lock re-author.
- Repair of R1 drag-pipeline-dead bug.

---

## Dependencies

| Dependency | Required posture | Why blocking |
|---|---|---|
| Sprint 18 activation | Required before `/dev-story` | This story is a Sprint 18 candidate. |
| Story 003 (Hand UI phase state machine) | Complete on `origin/main` | The `HandUiMode::PassiveLocked` transition is owned by Story 003; this story narrows the rendering disposition under that mode. |
| Story 023 (S18-OBS-SNAPSHOT-LAYOUT-FIELDS-001) | Implementation landed on `origin/main` per PROMPT 1287 §2 inventory (commit `e68ac4f`, PROMPT 1229) | `auction_state` and `placement_state` are now top-level QA-snapshot keys; this story consumes that surface for AC4 assertions. |
| `hand_fan_visible` QA-snapshot field | Existing (`qa_snapshot.rs:686`) | The BLOCKING evidence channel. |
| Story 021 (`S17-UI-HAND-B0004-CLEANUP-001`, Draft) | Optional / advisory | Hierarchy cleanup may simplify visibility wiring but is not required. |

This story touches `client/src/ui/hand/mod.rs` (and
optionally `client/src/ui/shop_auction/mod.rs` if the
worker chooses option (b) z-override). Serialise against
any concurrent worker editing the same modules.

---

## Acceptance Criteria

All criteria are independently checkable.

- [ ] **AC1 — DraftAuction hand-fan disposition is
  subordinate or hidden**: GIVEN `Res<HandUiMode> ==
  PassiveLocked` AND `Res<CurrentClientPhase> ==
  DraftAuction`, WHEN one `App::update()` tick runs,
  THEN one of the following holds:
  - (a) `HandFanRoot.Visibility == Hidden` AND every
    `FanSlotIndex` entity's effective visibility is
    `Hidden`, OR
  - (b) `HandFanRoot` is visible but its bevy_ui
    `ComputedNode.bounds` do NOT overlap the
    auction panel's bevy_ui `ComputedNode.bounds`
    at the canonical viewports (1280x720 / 1366x768
    / 1920x1080), OR
  - (c) `HandFanRoot` is visible at a z-layer below
    the auction panel (e.g. `UI_SUBORDINATE = 250`
    while `ShopAuctionRoot` remains `UI_BASE = 300`)
    AND visible-but-subordinate; in this case AC5
    BLOCKING assertion ensures the auction panel
    still wins the painter test.
  Worker discretion between (a) / (b) / (c) within
  ux-designer sign-off. Verified by integration-test
  ECS query + `ComputedNode` inspection.

- [ ] **AC2 — Auction panel is visually dominant**:
  GIVEN the same preconditions as AC1, WHEN the
  auction panel is inspected, THEN it is
  `Visibility::Visible`, its
  `ComputedNode.bounds` are fully inside the
  viewport at every canonical viewport in
  {1280x720, 1366x768, 1920x1080}, and (in
  combination with AC1) it is not occluded by
  `HandFanRoot`.

- [ ] **AC3 — Disposition is reversible on phase
  transition**: GIVEN AC1 post-state (hand fan
  subordinate / hidden during DraftAuction), WHEN
  `Res<CurrentClientPhase>` transitions to
  `RoundPhase::Placement` AND
  `HandUiMode::Staging` is entered, THEN
  `HandFanRoot.Visibility == Inherited` (visible)
  AND fan slots are reachable for drag interaction
  per existing Story 005 / 006 / 007 / 020 /
  022 / 023 contracts. Verified by integration-
  test phase-transition driver.

- [ ] **AC4 — QA snapshot evidence**: GIVEN the
  post-implementation build with the test fixture
  in AC1 state, WHEN the QA snapshot is written
  (`write_qa_snapshot_system`), THEN one of:
  - (a) `snapshot.hand_fan_visible == 0` AND
    `snapshot.auction_state` is `Some(_)` with a
    populated featured-card field, OR
  - (b) a new boolean snapshot field
    `hand_fan_occludes_auction == false`
    (worker-named) is present and `false`
    while `auction_state` is `Some(_)`.
  The chosen path determines AC4's exact assertion
  shape; either path is acceptable. The snapshot
  field choice is recorded in the evidence
  document.

- [ ] **AC5 — Auction panel wins the painter test**:
  GIVEN AC1 state, WHEN a synthetic
  ECS-level "what entity wins at pixel (cx, cy)"
  inspection runs at the centre of the auction
  featured-card region, THEN the answer is an
  entity that is a descendant of
  `ShopAuctionRoot`, NOT a descendant of
  `HandFanRoot`. The check uses bevy_ui
  `ComputedNode` z-order + bounds; pixel rendering
  is not required. (Worker discretion: this AC
  may be folded into AC1 if AC1 chooses path (a)
  full-hide.)

- [ ] **AC6 — Hand-ui regression suite PASS**:
  GIVEN the post-implementation build, WHEN run,
  THEN all PASS:
  - `cargo test -p client --test hand_ui_drag_state_visuals_test`
  - `cargo test -p client --test hand_ui_drag_to_board_cell_test`
  - `cargo test -p client --test hand_ui_submit_prevalidation_test`
  - `cargo test -p client --test hand_ui_reserve_mana_strip_test`
  - `cargo test -p client --test hand_ui_placement_staged_disclosure_accessibility_test`
  - `cargo test -p client --test hand_ui_placement_timer_test`
  - `cargo test -p client --test hand_ui_placement_unstaging_test`
  - `cargo test -p client --test hand_ui_plugin_scaffold_test`
    (entity-count assertion updated if a new
    sibling overlay / marker is added).

- [ ] **AC7 — Shop-auction-ui regression suite
  PASS**: GIVEN the post-implementation build,
  WHEN the existing shop-auction-ui integration
  tests run, THEN all PASS unchanged. The
  auction panel behaviour MUST NOT regress as a
  side effect of any z-layer / visibility
  coordination.

- [ ] **AC8 — Integration test in
  `tests/integration/hand-ui/`**: GIVEN the
  post-implementation build, WHEN
  `cargo test -p client --test hand_ui_fan_z_layer_auction_test`
  (or canonical-equivalent path chosen by
  `/dev-story`) runs, THEN it PASSES with at
  minimum 7 assertions covering AC1, AC2, AC3,
  AC4, AC5, AC6 (selective spot-check), AC7
  (selective spot-check). The test drives state
  via direct resource insertion (set
  `HandUiMode`, set `CurrentClientPhase`).

- [ ] **AC9 — Optional z-layer constant
  addition**: IF the worker chooses path (c)
  z-demote, GIVEN the post-implementation
  build, WHEN
  `client/src/ui/design_tokens/z_layers.rs`
  is inspected, THEN a new
  `pub const UI_SUBORDINATE: GlobalZIndex`
  (or canonical-equivalent name) is present at
  a value strictly less than `UI_BASE = 300`
  (e.g. 250) AND existing
  `UI_BASE` / `UI_OVERLAY` / `MODAL` / `TOAST`
  / `DEBUG` constants are unchanged. If the
  worker chooses path (a) or (b), this AC is
  N/A and the evidence document records the
  N/A disposition.

- [ ] **AC10 — Optional shop-auction-ui edit**:
  IF the worker chooses path (b) z-override on
  the auction side, GIVEN the
  post-implementation build, WHEN
  `client/src/ui/shop_auction/mod.rs` (and any
  related shop-auction modules) are diffed,
  THEN the diff is narrowly scoped to a z-layer
  promotion (e.g. `ShopAuctionRoot` from
  `UI_BASE` to `UI_OVERLAY`) and to no other
  composition change. If the worker chooses
  path (a) hide or path (c) demote-fan-only,
  this AC is N/A.

- [ ] **AC11 — ADR-002 + ADR-021 binding
  preserved**: GIVEN the post-implementation
  build, WHEN inspected, THEN:
  - The new systems read `Res<HandUiMode>`
    (immutable),
    `Res<CurrentClientPhase>` (immutable),
    per-slot `&FanSlotIndex` (immutable); they
    write only `Visibility` / `GlobalZIndex` /
    the optional `hand_fan_occludes_auction`
    snapshot boolean.
  - No new `S2C*` / `C2S*` message;
    `shared/src/protocol.rs` diff is empty.
  - `liv-bevy-lightyear` NOT activated.

- [ ] **AC12 — No `production/` shared-tracker
  edits**: GIVEN the implementation commit,
  WHEN `production/sprint-status.yaml`,
  `production/sprints/`, `production/qa/`,
  `production/stage.txt`,
  `production/session-state/`, and the PROMPT
  761 gate-check artifact are diffed, THEN
  none is modified by this story's implementing
  prompt except in the `/story-done` paperwork
  commit.

- [ ] **AC13 — Carried conditions preserved**:
  GIVEN the evidence and the implementation
  commit, WHEN inspected, THEN no claim is
  made against any of: `S8-QA-001-W1`,
  `QA-COND-0005`, `QA-COND-0006`,
  `PAW-TD-*-a`, `S11-HUD-TIMER-EYEBALL-VISUAL-001`
  (human-blocked), PROMPT 761 `Polish->Release`
  gate-check retry, stage advance from
  `Polish`, R1 drag-pipeline-dead bug, or
  Sprint 12 story 019 underlying drag-runtime
  question.

- [ ] **AC14 — Visual evidence captured**:
  GIVEN the post-implementation build, WHEN
  browser / WASM captures (or documented ECS
  node-bounds samples) are taken at 1280x720
  / 1366x768 / 1920x1080 during
  `RoundPhase::DraftAuction`, THEN the
  evidence records one capture / sample per
  viewport showing the auction panel visually
  dominant and the hand fan
  subordinate-or-hidden. Per Story 026
  precedent, ECS node-dimension sampling is
  acceptable if pixel captures are infeasible.

- [ ] **AC15 — Friend-game-scope no-claim
  restated in evidence**: GIVEN the
  evidence document, WHEN read at the bottom,
  THEN it verbatim restates the
  friend-game-scope-only disposition and
  preserves all carried `QA-COND-0005` /
  `QA-COND-0006` / `PAW-TD-*-a` /
  `S8-QA-001-W1` accept-risk language.

---

## Likely Files (for the future /dev-story — DO NOT EDIT IN THIS AUTHORING RUN)

| Path | Anticipated change |
|------|--------------------|
| `client/src/ui/hand/mod.rs` | Phase-driven `HandFanRoot.Visibility` (or `GlobalZIndex`) edit. If path (a) chosen: `HandUiMode::PassiveLocked` flips fan-root visibility to `Hidden`. If path (c) chosen: `HandFanRoot.GlobalZIndex` is set to `UI_SUBORDINATE` during DraftAuction and restored on phase transition out. If path (b) chosen: no edit here; edit in `shop_auction/mod.rs`. |
| `client/src/ui/shop_auction/mod.rs` | OPTIONAL (path (b) only): promote `ShopAuctionRoot.GlobalZIndex` from `UI_BASE` to `UI_OVERLAY` during `RoundPhase::DraftAuction`. |
| `client/src/ui/design_tokens/z_layers.rs` | OPTIONAL (path (c) only): add `pub const UI_SUBORDINATE: GlobalZIndex = GlobalZIndex(250);` plus doc comment. Existing constants unchanged. |
| `client/src/presentation/qa_snapshot.rs` | OPTIONAL: add `pub hand_fan_occludes_auction: bool` (or canonical-named boolean) to the top-level snapshot struct. Otherwise rely on existing `hand_fan_visible` field. |
| `tests/integration/hand-ui/hand_ui_fan_z_layer_auction_test.rs` (NEW) | Integration test per AC8. ECS-query-driven; drives state via direct resource insertion. |
| `tests/integration/hand-ui/hand_ui_plugin_scaffold_test.rs` (existing) | Update `HAND_UI_ENTITY_COUNT` expectation if applicable. |
| `design/gdd/hand-ui.md` | Optional addition: pin the DraftAuction hand-fan disposition in Rule 3. Worker discretion. |
| `docs/architecture/tr-registry.yaml` | Optional: `TR-HU-012 — DraftAuction hand-fan subordinate / hidden disposition`. Worker discretion. |
| `production/qa/evidence/sprint-18-hand-fan-z-layer-auction/README.md` (NEW) | ux-designer consultation note (chosen path); integration-test pass output; QA-snapshot diff; viewport captures (AC14); carried-conditions no-claim restatement (AC15). |
| This story file | Status flipped Draft → Ready by `/story-readiness` post Sprint 18 activation; Ready → Done by `/story-done`. |
| `production/epics/hand-ui/EPIC.md` | `/story-done` paperwork: refresh story row. PROMPT 1294 authoring adds the row in `Draft`. |

**Explicitly out of scope for the `/dev-story` worker**:

- `production/epics/hand-ui/story-020-hand-drag-state-visuals.md`
  (Story 020 Complete; NO EDIT).
- `production/epics/hand-ui/story-022-hand-mana-preview-during-drag.md`
  (Implementation landed; NO EDIT).
- `production/epics/hand-ui/story-023-hand-idle-playable-affordance.md`
  (Implementation landed; NO EDIT).
- `production/epics/shop-auction-ui/story-020-auction-won-card-disposition.md`
  (Sprint 18 Must Have; coordinate sequencing but do NOT edit
  this story's text).
- `shared/src/protocol.rs`, `server/`, `client/src/network/`.
- `production/sprints/*`, `production/sprint-status.yaml`,
  `production/stage.txt`, PROMPT 761 gate-check artifact.
- `Cargo.toml`, `Cargo.lock`, `.cargo/`, `.github/`.
- R1 drag-pipeline-dead repair (separate prompt).

---

## Out of Scope

- Server-side change.
- Protocol change (`shared/src/protocol.rs`).
- New Lightyear message.
- New client-side authority or optimistic mutations.
- Standard-tier accessibility of auction UI
  (`QA-COND-0005` preserved).
- Final-art (`PAW-TD-*-a`).
- Auction panel content / bid-controls / featured-card
  composition changes (out of host module).
- DraftShop / DraftInitial / Placement / Resolution
  hand visibility (covered by Story 003 phase state
  machine; this story narrows ONLY the
  `PassiveLocked` / DraftAuction disposition).
- Opponent-hand visibility (local hand only).
- Repair of R1 drag-pipeline-dead bug.
- Sprint 18 activation.
- `/qa-plan sprint-18` authoring.
- `/dev-story`, `/story-readiness`, `/story-done` on
  this story under the authoring prompt.
- Polish → Release gate-check retry.
- Stage advance from Polish.

---

## QA Test Cases

*Drafted by qa-lead at story creation. The developer
implements against these — do not invent new test
cases during implementation.*

- **AC1 — DraftAuction disposition**:
  - Given: `HandUiMode::PassiveLocked`,
    `CurrentClientPhase::DraftAuction`.
  - When: one tick.
  - Then: chosen path's invariant holds (hidden /
    non-overlapping / subordinate-z).

- **AC2 — Auction panel dominant**:
  - Given: AC1 preconditions; viewport in
    {1280x720, 1366x768, 1920x1080}.
  - When: one tick.
  - Then: auction panel `ComputedNode.bounds`
    fully inside viewport; auction panel
    `Visibility::Visible`; no `HandFanRoot`
    descendant overlaps the auction featured-
    card / bid-controls regions.

- **AC3 — Reversibility on phase exit**:
  - Given: AC1 post-state.
  - When: phase transitions to `Placement`
    (`HandUiMode::Staging`).
  - Then: `HandFanRoot.Visibility == Inherited`
    AND fan slots reachable.

- **AC4 — QA snapshot evidence**:
  - Given: AC1 state.
  - When: `write_qa_snapshot_system` runs.
  - Then: `snapshot.hand_fan_visible == 0` AND
    `snapshot.auction_state` is `Some(_)`, OR
    the new boolean snapshot field
    `hand_fan_occludes_auction == false` is
    present and `false`.

- **AC5 — Painter ordering**:
  - Given: AC1 state.
  - When: ECS-level inspection at the auction
    featured-card centre.
  - Then: winning entity is a descendant of
    `ShopAuctionRoot`.

- **AC6 — Hand-ui regression suite**:
  - When: each listed test runs.
  - Then: all PASS.

- **AC7 — Shop-auction-ui regression suite**:
  - When: existing shop-auction tests run.
  - Then: all PASS.

- **AC14 — Visual evidence**:
  - Given: post-implementation build.
  - When: captures / samples at each canonical
    viewport during DraftAuction.
  - Then: one capture / sample per viewport;
    auction dominant; hand subordinate or
    hidden.

---

## Performance Budget

Per ADR-021 Presentation steady-state budget of
`< 1 ms` per frame. The new visibility-toggle /
z-layer-flip system is `O(1)` per phase transition.
The optional QA-snapshot boolean is `O(1)` per
snapshot write. Expected per-frame cost change:
negligible.

---

## Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Worker hides the hand fan during DraftAuction AND breaks the Placement-entry transition (fan stays hidden into Placement). | Medium | High | AC3 explicit; integration-test phase-transition driver asserts. |
| Worker promotes `ShopAuctionRoot` to a z-layer that conflicts with the result-screen modal or the photosensitivity-warning modal. | Low | High | `MODAL = 500` is unchanged; promotion to `UI_OVERLAY = 400` keeps modals on top. AC2 spot-checks. |
| Worker reorders existing z-layer constants. | Low | High | Control Manifest forbids; reviewer checks `z_layers.rs` diff. |
| Worker edits Story 005 / 020 / 022 / 023 to "amend phase visibility". | Low | High | Out-of-scope rule + reviewer checks unrelated story files unchanged. |
| Worker breaks Story 020 / 022 / 023 by side-effect of fan-root visibility / z change. | Medium | High | AC6 BLOCKING regression gate. |
| Worker introduces a new Lightyear message to "ack phase change". | Low | High | AC11; `liv-bevy-lightyear` NOT activated. |
| Worker activates Sprint 18 as a side effect of `/dev-story` paperwork. | Low | Medium | No-Claim Banner forbids. |
| Worker conflates this story with the Sprint 18 Must Have `S18-AUCTION-WON-CARD-DISPOSITION-001` row. | Low | Medium | Two different surfaces (auction-won card disposition is server-side resolution wiring; this story is hand-fan layout). Out-of-scope rule disjoins. |

---

## Verification (orchestrator-side, before worker dispatch)

These are sanity checks for the orchestrator that emits the
`/dev-story` prompt, NOT for PROMPT 1294 itself:

- `production/sprint-status.yaml` top-level `sprint:` field reads
  `18` after Sprint 18 activation; row for this story is `ready`.
- `production/stage.txt` reads `Polish` and is unchanged.
- `production/sprints/sprint-18.md` shows the ACTIVATED banner and
  includes this row.
- PROMPT 761 `Polish->Release` gate-check FAIL preserved.
- `production/qa/qa-plan-sprint-18.md` references this story.
- `/story-readiness` on this story returns READY.
- Stories 003 / 020 / 022 / 023 unchanged on `origin/main`.
- Sprint 18 Must Have row `S18-AUCTION-WON-CARD-DISPOSITION-001`
  sequencing reviewed: this story does NOT depend on that row's
  landing.
- R1 drag-pipeline-dead repair status: this story is independent.

---

## Authoring Trail

- 2026-05-18 — PROMPT 1294 — Story file authored as future
  Sprint 18 Wave-A candidate
  `S18-HAND-FAN-Z-LAYER-AUCTION-001`. Worktree
  `D:\_DEV\claude-code-game-studios-worktrees\s18-story-authoring-wave-a-1294`,
  branch `work/s18-story-authoring-wave-a-1294`, base
  `origin/main@1345c6b8b1cbd543dbd63d279186c93924ca54db`
  (PROMPT 1285 Sprint 18 plan draft). Files touched by this
  authoring run: this file (NEW) and
  `production/epics/hand-ui/EPIC.md` (story-list row added).
  Sibling Wave-A stories
  `S18-LOBBY-CONFIRM-CTA-VISIBLE-001` and
  `S18-HAND-FAN-PASSIVE-CLICK-AFFORDANCE-001` authored in the
  same run (the lobby row under
  `production/epics/playable-client/`, the passive-click row
  under `production/epics/hand-ui/`). Sprint 18 NOT activated.
  No code change. No `/dev-story`, `/story-readiness`,
  `/story-done`, `/smoke-check`, `/team-qa`, `/gate-check`,
  `/release-check`, `/qa-plan`, `cargo`, or `trunk` command
  run. ADR-002 + ADR-021 binding preserved; Sprint 12 story
  019 disposition preserved; PROMPT 761 gate-check FAIL
  preserved; `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a`,
  `S8-QA-001-W1`, `TQ-S12-C1..C7`,
  `S11-HUD-TIMER-EYEBALL-VISUAL-001` human-blocked state, all
  preserved verbatim. R1 drag-pipeline-dead repair status
  unchanged (separate prompt).
