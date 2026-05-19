# Story 027: S19-UI-HAND-RESERVE-STRIP-CLEANUP-001 -- Hand Reserve-Strip Cleanup / Microbadge Follow-Up

> **Epic**: Hand UI
> **Story ID**: `S19-UI-HAND-RESERVE-STRIP-CLEANUP-001`
> **Status**: Draft -- future Sprint 19 candidate; NOT activated
> **Layer**: Presentation / Hand UI (reserve-strip per-card allocation widget under `client/src/ui/hand/`)
> **Type**: UI + Integration test (regression-lock of PROMPT 1175 source repair plus narrow visual / a11y-label polish)
> **Sprint**: Sprint 19 candidate (re-evaluated at Sprint 19 planning per PROMPT 1301 `sprint_18_activation.dropped_rows` disposition). Sprint 18 dropped this row at activation because no story file existed on `origin/main`; this authoring run lands the missing story file but does NOT activate Sprint 19 and does NOT pull the row back into Sprint 18.
> **Authored**: 2026-05-19 by PROMPT 1351
> **Authoring worktree**: `D:\_DEV\Work\Claude-Code-Game-Studios\.claude\worktrees\prompt-1351-s19-hand-reserve-strip-cleanup-story-authoring`
> **Authoring branch**: `work/s19-hand-reserve-strip-cleanup-story-authoring-1351`
> **Authoring source-of-truth**: `origin/main@1e9548f23f7f19d3f8e14591b731cdfbbdd57874`
> **Predecessor candidate slug** (recorded historically; superseded by this story's `S19-` slug): `S18-UI-HAND-RESERVE-STRIP-CLEANUP-001` (PROMPT 1263 §3 candidate slug; PROMPT 1285 §2.3 plan row; PROMPT 1301 dropped at activation)
> **Source reports**: PROMPT 1076 AUDIT-1076-17 (P3); PROMPT 1112 partial disposition of `S17-UI-HUD-OPP-MANA-CLEANUP-001` (AC3 carried); PROMPT 1175 `c842668` AC3 source repair main-land; PROMPT 1263 §3 candidate-slug pin; PROMPT 1279 Sprint 17 closeout `rows_carried_forward[1]`; PROMPT 1285 Sprint 18 plan draft §2.3; PROMPT 1301 Sprint 18 activation `dropped_rows` disposition

---

## Status / No-Claim Banner

PROMPT 1351 authors this story as a **future Sprint 19 candidate**.
Sprint 18 is `active` on `origin/main` (PROMPT 1301 activation against
`origin/main@1345c6b8b1cbd543dbd63d279186c93924ca54db`) and is **NOT
re-opened, re-scoped, or otherwise modified** by PROMPT 1351.

PROMPT 1351 (this authoring run) does **NOT**:

- Activate Sprint 19.
- Re-open or modify Sprint 18 status / scope.
- Modify `production/sprint-status.yaml`.
- Modify any file under `production/sprints/**`.
- Modify `production/stage.txt` (remains `Polish`).
- Modify any file under `production/session-state/**`,
  `production/qa/**`, or `production/gate-checks/**`.
- Run `/story-readiness`, `/dev-story`, `/story-done`, `/smoke-check`,
  `/team-qa`, `/gate-check`, `/release-check`, or `/qa-plan`.
- Modify any code under `client/`, `server/`, `shared/`, or `tests/`.
- Modify any `Cargo.toml` / `Cargo.lock` / `.cargo/` / `.github/`
  / `Trunk.toml` file.
- Push to `origin/main`.
- Claim closure of `S17-UI-HUD-OPP-MANA-CLEANUP-001` (its AC3 source
  repair landed on `origin/main` via PROMPT 1175 `c842668`, but the
  parent row remains `in_progress` per PROMPT 1279
  `sprint_17_closeout.rows_carried_forward[1]`; this authoring run
  does **NOT** close that parent row — it only authors the
  follow-up story whose future `/dev-story` + `/story-done`
  paperwork is the producer-preferred discharge path).
- Claim discharge of `AUDIT-1076-17` (the AC3 source repair has
  landed on `origin/main`, but the discharge claim against the
  parent row is reserved for the future `/story-done` paperwork on
  `S17-UI-HUD-OPP-MANA-CLEANUP-001` once this story lands, OR for
  the producer's explicit accept-into-Sprint-19-discharge
  disposition — whichever the producer chooses; either path is
  outside this authoring run's scope).

This story does **not** claim:

- public release readiness
- release-candidate readiness
- full game completion
- broad / Standard-tier accessibility completion (`QA-COND-0005`
  preserved; the new a11y-label addition under AC5 is friend-game-
  scope only and does NOT pursue Standard-tier conformance)
- playtest / fun-hypothesis validation (`QA-COND-0006`)
- closure of `S8-QA-001-W1`
- final-art / asset-production completion (`PAW-TD-*-a`)
- `Polish->Release` gate-check retry (PROMPT 761 FAIL preserved)
- advance of stage from `Polish` to `Release`
- closure of `S11-HUD-TIMER-EYEBALL-VISUAL-001` (human-operator-
  blocked carry preserved verbatim)
- closure of any AUDIT-1076-* / SOURCE-1077-* / PROMPT 1022 finding
  outside the AUDIT-1076-17 regression-lock asserted by AC3 / AC4
- closure of Sprint 12 story 019 underlying drag-runtime question —
  disposition preserved verbatim
- repair of PROMPT 1127 §R1 drag-pipeline-dead bug — separate prompt

ADR-002 + ADR-021 binding preserved. This story is composition-only
(reserve-strip child node text values + optional visual / a11y-label
polish on the existing `[ - ]` / value / `[ + ]` child entities). No
new server-authoritative state, no new Lightyear message, no
protocol shape change, no client-side authority over reserve-mana
spend (which remains driven by the existing
`handle_reserve_strip_button_interactions_system` per Story 011
HU-25 / HU-26 / HU-27 contract).

---

## Source Findings

### AUDIT-1076-17 (P3) — Duplicate / unanchored mana microbadge (closed source-side; parent-row paperwork still open)

- **Audit location**:
  `reports/PROMPT-1076-latest-user-test-log-snapshot-deep-audit.md`
  §"Per-finding evidence" AUDIT-1076-17.
- **Original behaviour** (pre-PROMPT 1175): the per-staged-card
  reserve allocation widget spawned by `spawn_reserve_strip` (under
  `client/src/ui/hand/mod.rs`) read **"Reserve N + / Current N"**.
  Snapshot 000020 captured this widget floating above the canonical
  HUD `MANA n / N` strip; the duplicated "Reserve / Current"
  wording read to the player as a second mana display.
- **AC3 source repair** (PROMPT 1175 `c842668` on `origin/main`):
  - `client/src/ui/hand/mod.rs:4082-4099` (`spawn_reserve_strip`):
    the value text now spawns with `Text::new("")` and is populated
    by `set_reserve_value_text` with a bare reserve-allocation
    integer (e.g. `"2"`) when a card is staged. Doc-comment cites
    PROMPT 1175 + AUDIT-1076-17 verbatim.
  - `client/src/ui/hand/mod.rs:4988-5007` (`set_reserve_value_text`):
    no "Reserve" / "Current" wording; empty when `cost == 0`;
    otherwise `reserve_amount.to_string()`.
  - `tests/unit/hand-ui/reserve_mana_strip_test.rs`: HU-25
    assertion updated +
    `audit_1076_17_reserve_strip_text_has_no_reserve_or_current_wording_when_staged`
    regression added.
  - `tests/integration/hand-ui/placement_staged_disclosure_accessibility_test.rs`:
    two reserve-text assertions updated +
    `assert_no_reserve_strip_microbadge_wording` helper added.
- **Parent-row paperwork gap**:
  `S17-UI-HUD-OPP-MANA-CLEANUP-001` AC3 source repair landed via
  PROMPT 1175, but per PROMPT 1112 paragraph "AC3 carry-forward
  classification and follow-up candidate" and PROMPT 1279
  `sprint_17_closeout.rows_carried_forward[1]`, the parent row's
  final `/story-done` was deliberately gated on either (a) the
  AC3 source repair landing under a separate hand-ui owner row
  (which PROMPT 1175 satisfied), OR (b) producer explicit
  accept-into-Sprint-N-discharge disposition. The producer-
  preferred path is (a) executed via a discrete follow-up story
  per PROMPT 1111 recommendation. **This story IS that follow-up
  candidate** (`S19-UI-HAND-RESERVE-STRIP-CLEANUP-001`,
  predecessor candidate slug `S18-UI-HAND-RESERVE-STRIP-CLEANUP-001`).

### PROMPT 1263 §3 candidate-slug pin

PROMPT 1263 (Sprint 18 readiness verdict) records the slug
`S18-UI-HAND-RESERVE-STRIP-CLEANUP-001` as a "candidate slug only,
not an existing story file" and explicitly forbade Sprint 18
activation from including this row "until a story file is authored
under `production/epics/hand-ui/` (most likely slot) and landed on
`origin/main`".

### PROMPT 1285 §2.3 Sprint 18 plan draft

PROMPT 1285 (Sprint 18 plan draft, on `origin/main` at PROMPT 1292
land tip `1345c6b`) listed the row in its Nice to Have tier (0.2d)
with `story-authoring-needed` as its blocking gate.

### PROMPT 1301 dropped-row disposition

PROMPT 1301 (Sprint 18 activation paperwork, `origin/main`) dropped
the row at activation per the PROMPT 1285 §2.3 explicit constraint
and recorded it under
`sprint_18_activation.dropped_rows[*] (reason: story-authoring-
needed; re-evaluation deferred to Sprint 19 planning)`. The
Sprint 18 plan banner (`production/sprints/sprint-18.md` lines
49-61) cites this disposition verbatim.

---

## Problem Class / Prevention Target

**Defect class**: a per-card cleanup surface remains where future
edits to `spawn_reserve_strip` / `set_reserve_value_text` could
silently re-introduce the verbose "Reserve N + / Current N" wording
that AUDIT-1076-17 originally flagged, OR introduce a new floating
sibling node carrying duplicate-mana wording. The current
regression test bins
(`tests/unit/hand-ui/reserve_mana_strip_test.rs` and
`tests/integration/hand-ui/placement_staged_disclosure_accessibility_test.rs`)
assert against the specific `ReserveStripValueText` entity, but
they do not assert the broader invariant "no `Text` node anywhere
under the reserve strip subtree contains the substrings `Reserve`
/ `Current` / `mana`" at any staged-card lifecycle stage.

**Prevention target**:

1. **Regression-lock the PROMPT 1175 AC3 source repair against
   broader subtree coverage**: a single integration test bin
   asserts that across the full reserve-strip subtree (parent
   `ReserveStripForFanSlot` + all `ChildOf` descendants), no
   `Text` node contains the wording `"Reserve "`, `"Current "`,
   or `"mana"` (case-insensitive on the latter), at every
   meaningful staged-card lifecycle state (empty / staged-with-
   cost-0 / staged-with-cost-positive / +-clicked / unstaged).
2. **Optional narrow visual / a11y-label polish** under worker
   discretion within `client/src/ui/hand/mod.rs` reserve-strip
   spawn / sync code paths only. The bare integer (e.g. `"2"`)
   reads ambiguously in isolation; the implementing prompt MAY
   add a screen-reader-only `Name` / accessible-label helper
   that decorates the `ReserveStripValueText` entity with a
   non-rendered label like `"Reserve allocation: 2 of 5"` IF
   the worker chooses the a11y-label path. This is **friend-
   game-scope only**; `QA-COND-0005` Standard-tier conformance
   is NOT pursued and the visible canonical HUD `MANA n / N`
   strip remains the single visible mana display.
3. **Document the parent-row paperwork discharge contract**: the
   evidence document recorded by the future `/dev-story` worker
   explicitly records the discharge claim against
   `S17-UI-HUD-OPP-MANA-CLEANUP-001` AC3 / `AUDIT-1076-17`. The
   producer's subsequent `/story-done` paperwork on the parent
   row (a separate prompt, not this story's `/dev-story`) is the
   authoritative closure trigger.

---

## Context

### Existing surface (read-only at authoring time)

- **`client/src/ui/hand/mod.rs:837`**:
  `pub struct ReserveStripValueText(pub u8);` — marker on the
  per-slot value-text entity that PROMPT 1175 made bare-integer-
  only.
- **`client/src/ui/hand/mod.rs:849`**: `HandLayout::reserve_strips`
  pre-pooled array (`[Entity; HAND_FAN_SLOT_COUNT]`).
- **`client/src/ui/hand/mod.rs:1177-1208`**: `HandUiPlugin`
  schedule entries for
  `handle_reserve_strip_button_interactions_system` +
  `apply_reserve_strip_layout_system` +
  `sync_reserve_strip_state_system`.
- **`client/src/ui/hand/mod.rs:1327-1380`**:
  `apply_reserve_strip_layout_system` and
  `sync_reserve_strip_state_system` (per-frame `Visibility` +
  layout sync, hand-staged-card-driven).
- **`client/src/ui/hand/mod.rs:3209-3308`**:
  `set_reserve_strip_visibility` + the
  `handle_reserve_strip_button_interactions_system`'s downstream
  `sync_reserve_strip_entities` call.
- **`client/src/ui/hand/mod.rs:3847-3848`**: pre-pool init at
  `spawn_hand_ui` time (one strip per fan slot).
- **`client/src/ui/hand/mod.rs:4061-4112`**: `spawn_reserve_strip`
  — the spawn site PROMPT 1175 cleaned (value text now empty;
  doc-comment cites PROMPT 1175 + AUDIT-1076-17). The `[ - ]`
  button spawn (`spawn_reserve_strip_button` with `"-"` label,
  `0.0` left) precedes the value text; the value text precedes
  the `[ + ]` button spawn (with `"+"` label, `156.0` left) which
  starts disabled (`ReserveStripButtonDisabled`).
- **`client/src/ui/hand/mod.rs:4114-4136`**:
  `spawn_reserve_strip_button` — common spawn helper for the
  `[ - ]` / `[ + ]` buttons.
- **`client/src/ui/hand/mod.rs:4988-5007`**: `set_reserve_value_text`
  — empty when `cost == 0`; bare `reserve_amount.to_string()`
  otherwise. PROMPT 1175 doc-comment cites AUDIT-1076-17.
- **`client/src/ui/hand/mod.rs:5009-5033`**:
  `set_reserve_button_disabled` — toggles the
  `ReserveStripButtonDisabled` marker insertion / removal.
- **`tests/unit/hand-ui/reserve_mana_strip_test.rs:10`**:
  imports `ReserveStripValueText` + helpers.
- **`tests/unit/hand-ui/reserve_mana_strip_test.rs:39-77`**: HU-25
  assertion update + the
  `audit_1076_17_reserve_strip_text_has_no_reserve_or_current_wording_when_staged`
  unit-level regression + `assert_no_reserve_strip_microbadge_wording`
  helper (queries `ReserveStripValueText` only).
- **`tests/integration/hand-ui/placement_staged_disclosure_accessibility_test.rs:168,231-234,428-429`**:
  parallel integration-level regression + helper (also queries
  `ReserveStripValueText` only).
- **`design/gdd/hand-ui.md` Rule 13**: documents the reserve-mana
  split control behaviour (Story 011 HU-25 / HU-26 / HU-27). No
  visible-text wording is pinned in the GDD; the bare-integer
  contract lives in code + tests only.
- **`docs/architecture/tr-registry.yaml:1810-1817`**: `TR-HU-004`
  ("Reserve mana split control: UI shows current_mana / mana_cap
  with reserve_mana iff > 0") — this story does NOT amend
  `TR-HU-004`; it proposes a sibling `TR-HU-013` (see below).

### Predecessor / sibling stories (status preserved verbatim)

- **Story 011 (`reserve-mana-strip`)** -- Complete. Story 011
  owns the reserve-strip ceiling formula / `[ + ]` disable logic
  / per-card `reserve_amount` state. This story does NOT re-author
  Story 011 and does NOT change its acceptance criteria
  HU-25 / HU-26 / HU-27. Its file is `production/epics/hand-ui/
  story-011-reserve-mana-strip.md`.
- **Story 005 (`placement-submit-core`)** -- Complete. Owns the
  strip visibility-toggle lifecycle (Visible on stage / Hidden on
  un-stage). No edit.
- **Story 010 (`submit-prevalidation`)** -- Complete. Sums
  `reserve_amount` as the pre-submit gate. No edit.
- **Story 014 (`placement-staged-disclosure-accessibility`)** --
  Complete. Owns the parallel integration-test bin that
  PROMPT 1175 amended; this story's new integration test bin
  is a NEW file under `tests/integration/hand-ui/`, not an edit
  of Story 014's bin.
- **Story 022 / 023 / 025 / 026** -- Draft, future Sprint 18
  candidates. Disjoint from the reserve-strip surface (they own
  drag-preview / idle-affordance / passive-click-intent /
  draft-auction z-layer surfaces respectively). No edit.

### Active Sprint 18 lane awareness

PROMPT 1347 (`S18-AUCTION-WON-CARD-DISPOSITION-001`),
PROMPT 1348 (`S18-UI-CARD-ART-AND-LABEL-STRIP-001`), and
PROMPT 1349 (`S18-UI-OVERLAY-PANEL-OVERFLOW-HARDENING-001`) are
the in-flight Sprint 18 Must / Should / Nice lanes at PROMPT 1351
authoring time. **None of them touch `client/src/ui/hand/mod.rs`**
(card-display owners are `client/src/ui/shop_auction/` and
`client/src/ui/ui_clean_pass/` per their respective story files).
The reserve-strip surface this story scopes is therefore
file-disjoint from every in-flight Sprint 18 lane; no file-level
serialisation is required at Sprint 19 `/dev-story` time.

### GDD / ADR / TR trace

- **GDD**: `design/gdd/hand-ui.md` Rule 13 (reserve-mana split
  control). No visible-text wording pinned. Worker may add a
  one-line note pinning "bare per-card allocation integer; no
  duplicate-mana wording" but is not required to.
- **ADR-002** (Client-Server Authority): preserved unchanged.
  Reserve-mana spend is driven by the existing
  `handle_reserve_strip_button_interactions_system` which writes
  to a local-mirror resource; the server validates at submit.
- **ADR-021** (Presentation Layer Architecture): preserved
  unchanged. The new (or amended) systems remain inside the
  existing `HandUiPlugin` schedule sets; no new schedule wiring.
- **TR-HU-004** (existing): preserved verbatim. The bare-integer
  cleanup does NOT change the requirement text.
- **TR registry**: this story proposes a new sibling
  `TR-HU-013 — Reserve strip cleanup: bare per-card allocation
  integer; no duplicate-mana wording across the strip subtree`.
  Authoring under `/dev-story` time at worker discretion; the
  authoring run (PROMPT 1351) does NOT add this row to
  `docs/architecture/tr-registry.yaml`. (TR-HU-009 / -010 / -011 /
  -012 are reserved by Stories 022 / 023 / 025 / 026
  respectively at their own `/dev-story` time, per their story
  files; this story takes -013 to avoid any clash.)

### Engine / skills

- **Engine**: Bevy 0.18 (Rust).
- **Mandatory skills**: `liv-bevy-018` for any `.rs` edit (the
  reserve-strip spawn site uses Bevy 0.18 Required Components API
  — `Text::new("")` + `Node` + `ChildOf` + `Visibility` —
  exclusively).
- **Lightyear**: NOT applicable. No `lightyear` import is
  introduced; `liv-bevy-lightyear` is NOT activated.

### Control Manifest Rules

- **Required**: across the full reserve-strip subtree (parent
  `ReserveStripForFanSlot` + every `ChildOf` descendant), no
  `Text` node may contain the substrings `"Reserve "`,
  `"Current "`, or `"mana"` (case-insensitive on `"mana"`), at
  any staged-card lifecycle state. The canonical HUD
  `MANA n / N` strip continues to render unchanged in a
  different subtree.
- **Required**: the bare-integer contract from PROMPT 1175 is
  preserved (`set_reserve_value_text` writes
  `reserve_amount.to_string()` when `cost > 0`; writes `""`
  when `cost == 0`). The future worker may not introduce any
  visible text decoration on `ReserveStripValueText` (no
  prefix, no suffix, no separator).
- **Required**: if the worker chooses the optional a11y-label
  polish path (AC5), the screen-reader-only label entity / label
  attribute is **non-rendered** (no visible UI change) and is
  scoped to the reserve-strip subtree only. The bare-integer
  `Text` content remains the only visible glyph.
- **Required**: the `[ - ]` / `[ + ]` button visual states (the
  `BackgroundColor` on the `ReserveStripButton` entity and the
  `ReserveStripButtonDisabled` marker insertion / removal
  semantics) preserve the existing Story 011 HU-25 / HU-26
  contracts. Any visual chrome polish (AC6 worker-discretion path)
  is purely additive (e.g. a `BorderColor` / `BorderRadius`
  child node, or a `TextColor` tint) and does NOT alter the
  click-disable semantics.
- **Required**: `liv-bevy-018` skill activated on every `.rs`
  edit.
- **Required**: the future `/dev-story` worker records, in its
  evidence document, an explicit "discharges `AUDIT-1076-17`
  parent-row paperwork gap from `S17-UI-HUD-OPP-MANA-CLEANUP-001`
  AC3 carry-forward (PROMPT 1112 partial disposition;
  PROMPT 1175 source repair already on `origin/main`)" claim.
  This claim is for the future `/story-done` paperwork's use; the
  `/dev-story` worker does NOT itself close the parent row.
- **Forbidden**: Mutating server-authoritative state.
- **Forbidden**: Adding a new Lightyear message.
- **Forbidden**: Editing `shared/src/protocol.rs`, the server-side
  reserve / placement / submission code, or any QA / sprint /
  session-state tracker.
- **Forbidden**: Touching `production/sprint-status.yaml`,
  `production/sprints/`, `production/qa/`,
  `production/stage.txt`, `production/session-state/`, the
  PROMPT 761 gate-check artifact, or any `Cargo.toml` /
  `Cargo.lock` / `.cargo/` / `.github/` / `Trunk.toml` file.
- **Forbidden**: Re-authoring Story 011 / 005 / 010 / 014. Their
  files are preserved verbatim by this story's `/dev-story`.
- **Forbidden**: Re-introducing the verbose "Reserve N + /
  Current N" wording or any equivalent multi-token mana display
  in the reserve-strip subtree.
- **Forbidden**: Modifying the canonical HUD `MANA n / N` strip
  under `client/src/ui/hud/` (different subtree, different owner;
  out of scope).
- **Forbidden**: Claiming `QA-COND-0005`, `QA-COND-0006`,
  `PAW-TD-*-a`, `S8-QA-001-W1`,
  `S11-HUD-TIMER-EYEBALL-VISUAL-001`, PROMPT 761 `Polish->Release`
  retry, or stage-advance.
- **Forbidden**: Closing the parent row
  `S17-UI-HUD-OPP-MANA-CLEANUP-001` from this story's
  `/dev-story` worker. The parent-row `/story-done` paperwork is
  a separate prompt (the producer's call).

---

## Story Classification

**Story type**: **UI + Integration test** — narrow visual /
a11y-label polish on the existing reserve-strip subtree plus a
broader subtree-scoped integration test that regression-locks the
PROMPT 1175 AC3 source repair against future drift.

Per `.claude/docs/coding-standards.md` "Test Evidence by Story
Type" matrix, the integration-test bin under AC3 / AC4 is the
BLOCKING evidence gate; the optional visual polish (AC6) is
ADVISORY with screenshot + lead sign-off if elected.

This is **NOT** a:

- Networking / protocol story.
- Final-art story.
- Accessibility story claiming Standard-tier conformance
  (`QA-COND-0005` preserved; a11y-label addition under AC5 is
  friend-game-scope only).
- Re-author of Story 011 reserve-mana-strip-formula contracts.
- Re-author of the canonical HUD `MANA n / N` strip under
  `client/src/ui/hud/`.
- Repair of the R1 drag-pipeline-dead bug.
- Repair of any underlying drag-runtime question (Sprint 12
  story 019 disposition preserved verbatim).

---

## Dependencies

| Dependency | Required posture | Why blocking |
|---|---|---|
| Sprint 19 activation | Required before `/dev-story` | This story is a Sprint 19 candidate; do not run `/dev-story` until top-level `sprint:` reads `19` and this row is `ready`. |
| Story 011 (Reserve mana split strip) | Complete on `origin/main` | Owns the reserve-strip ceiling formula / button-disable logic / `[ - ]` / `[ + ]` interactions that AC1 / AC2 / AC6 preserve. |
| PROMPT 1175 `c842668` AC3 source repair | On `origin/main` | The bare-integer contract this story regression-locks. |
| `S17-UI-HUD-OPP-MANA-CLEANUP-001` parent row | `in_progress` on `origin/main`; `/story-done` NOT closed by this story's `/dev-story`; closure is the **separate** producer prompt that uses this story's evidence document as the discharge claim | The parent-row paperwork gap that this story discharges (after its own `/story-done`) but does NOT itself close. |
| `tests/integration/hand-ui/placement_staged_disclosure_accessibility_test.rs` | Existing on `origin/main` (Story 014); PROMPT 1175 amended with two reserve_text assertions + `assert_no_reserve_strip_microbadge_wording` helper | The future `/dev-story` worker does NOT edit this bin. AC4 introduces a NEW bin (file-disjoint). |
| `tests/unit/hand-ui/reserve_mana_strip_test.rs` | Existing on `origin/main`; PROMPT 1175 amended with HU-25 update + `audit_1076_17_reserve_strip_text_has_no_reserve_or_current_wording_when_staged` | The future `/dev-story` worker MAY append additional regression tests; MUST NOT regress existing tests. |

This story touches `client/src/ui/hand/mod.rs` only (under the
reserve-strip spawn / sync surface). At Sprint 19 `/dev-story` time
the worker MUST serialise against any concurrent worker editing the
same module (PROMPT 1351 records that at authoring time no in-flight
Sprint 18 lane edits `client/src/ui/hand/mod.rs`; this is informational
only and the orchestrator re-verifies at `/dev-story` dispatch time).

---

## Acceptance Criteria

All criteria are independently checkable.

- [ ] **AC1 — Bare-integer contract preserved across all
  lifecycle states**: GIVEN a Bevy 0.18 `App` running
  `HandUiPlugin` in `ClientState::InSession` with
  `HandUiMode::Staging` and a populated
  `Res<HandCardCatalog>` (cards with cost in {0, 1, 5}), WHEN
  the lifecycle is driven (no card staged → cost-0 card staged →
  cost-positive card staged at `reserve_amount = 0` → `[ + ]`
  clicked → `[ - ]` clicked → card un-staged), THEN for every
  `(ReserveStripValueText, &Text)` query result at every step:
  - cost-0 staged: `text.0.is_empty()` is `true`;
  - cost-positive staged at `reserve_amount = N`: `text.0 ==
    N.to_string()` (bare integer, no prefix, no suffix);
  - un-staged: `text.0.is_empty()` is `true`.
  Verified by integration-test ECS query + lifecycle driver.

- [ ] **AC2 — `[ - ]` / `[ + ]` button-state contracts
  preserved (Story 011 HU-25 / HU-26)**: GIVEN the same
  `App`, WHEN the lifecycle from AC1 is driven, THEN the
  `ReserveStripButton` entities preserve the Story 011
  HU-25 / HU-26 contracts (ceiling = `min(card_cost,
  player.reserve_mana - sum(other_staged.reserve_amount))`;
  `ReserveStripButtonDisabled` inserted / removed exactly per
  the existing `set_reserve_button_disabled` semantics; no
  auto-decrement of other staged cards). Verified by the
  existing Story 011 unit-test bin
  (`tests/unit/hand-ui/reserve_mana_strip_test.rs`) under
  `cargo test -p client --test hand_ui_reserve_mana_strip_test`
  PASS unchanged.

- [ ] **AC3 — Subtree-scoped regression-lock of PROMPT 1175
  AC3 source repair**: GIVEN the same `App` in AC1's
  cost-positive-staged state, WHEN every `Text` node in the
  reserve-strip subtree (rooted at the
  `ReserveStripForFanSlot` parent entity, traversing all
  `ChildOf` descendants) is scanned, THEN none of those
  `Text` nodes contains:
  - the literal substring `"Reserve "` (case-sensitive,
    trailing space included);
  - the literal substring `"Current "` (case-sensitive,
    trailing space included);
  - the substring `"mana"` (case-insensitive — matches both
    `"mana"`, `"Mana"`, `"MANA"`, etc.).
  The assertion runs at every lifecycle step from AC1
  (no card staged, cost-0 staged, cost-positive staged at
  `reserve_amount = 0`, after `[ + ]` click, after `[ - ]`
  click, un-staged). Verified by a NEW integration-test
  helper that recursively walks the subtree via the Bevy
  0.18 `Children` query (NOT via the existing
  `assert_no_reserve_strip_microbadge_wording` helper, which
  only queries the `ReserveStripValueText` entity directly).

- [ ] **AC4 — NEW integration-test bin under
  `tests/integration/hand-ui/`**: GIVEN the post-
  implementation build, WHEN
  `cargo test -p client --test hand_ui_reserve_strip_cleanup_test`
  (or canonical-equivalent path chosen by `/dev-story`)
  runs, THEN it PASSES with at minimum 7 sub-tests covering
  AC1 (3 lifecycle states minimum), AC2 (selective spot-
  check via the existing button-state helpers), AC3
  (subtree walk at 3 lifecycle states minimum). The test
  drives state via direct resource insertion / message
  emission (set `HandUiMode`, set `CurrentClientPhase`,
  emit `HandStagingChanged` or equivalent); does NOT depend
  on the PROMPT 1127 §R1 bevy_picking drag path.

- [ ] **AC5 — OPTIONAL a11y-label polish (worker
  discretion)**: IF the worker chooses to add a screen-
  reader-only label / accessible-label attribute decorating
  the `ReserveStripValueText` entity, GIVEN the post-
  implementation build, WHEN the entity is inspected, THEN
  the visible `Text` content remains the bare integer (per
  AC1) AND the screen-reader label entity / attribute
  contains a phrase like `"Reserve allocation: N of cost"`
  (worker-named exact wording) that is NOT rendered on the
  visible UI surface (verified by absence in any visible
  `Text` glyph during the bevy_ui pipeline). The evidence
  document records the chosen wording and the worker's
  rationale. **If the worker elects NOT to add this label,
  this AC is N/A and the evidence document records the
  N/A disposition; AC1..AC4 + AC6..AC15 alone close the
  story.**

- [ ] **AC6 — OPTIONAL narrow visual chrome polish (worker
  discretion)**: IF the worker chooses to add minor visual
  chrome to the reserve-strip subtree (e.g. a `BorderColor`
  or `BorderRadius` child node on the `[ - ]` / `[ + ]`
  buttons, OR a `TextColor` tint on the
  `ReserveStripValueText` entity), GIVEN the post-
  implementation build, WHEN inspected, THEN the chrome is
  additive (no replacement of existing entities, no
  removal of existing entities, no edit to existing
  `BackgroundColor` semantics on the buttons), the
  click-disable semantics from Story 011 HU-25 / HU-26 are
  preserved, and the `HAND_UI_ENTITY_COUNT` constant is
  bumped exactly by the number of newly pre-pooled
  entities (if any new pre-pooled entities are added). **If
  the worker elects NOT to add any visual chrome, this AC
  is N/A and the evidence document records the N/A
  disposition; AC1..AC5 + AC7..AC15 alone close the story.**

- [ ] **AC7 — Hand-ui regression suite PASS**: GIVEN the
  post-implementation build, WHEN run, THEN all PASS:
  - `cargo test -p client --test hand_ui_reserve_mana_strip_test`
    (Story 011 unit bin; PROMPT 1175
    `audit_1076_17_reserve_strip_text_has_no_reserve_or_current_wording_when_staged`
    regression PASSES unchanged)
  - `cargo test -p client --test hand_ui_placement_staged_disclosure_accessibility_test`
    (Story 014 integration bin; PROMPT 1175
    `assert_no_reserve_strip_microbadge_wording` helper
    PASSES unchanged)
  - `cargo test -p client --test hand_ui_drag_state_visuals_test`
  - `cargo test -p client --test hand_ui_drag_to_board_cell_test`
  - `cargo test -p client --test hand_ui_submit_prevalidation_test`
  - `cargo test -p client --test hand_ui_placement_timer_test`
  - `cargo test -p client --test hand_ui_placement_unstaging_test`
  - `cargo test -p client --test hand_ui_plugin_scaffold_test`
    (entity-count assertion updated if AC6 adds new
    pre-pooled entities)

- [ ] **AC8 — HUD canonical mana strip unchanged**: GIVEN
  the implementation commit, WHEN
  `git diff origin/main..HEAD -- client/src/ui/hud/` is
  inspected, THEN the diff is empty. The canonical HUD
  `MANA n / N` strip lives in a different subtree and a
  different owner module; this story does NOT touch it.

- [ ] **AC9 — Cross-epic story files unchanged**: GIVEN
  the implementation commit, WHEN
  `git diff origin/main..HEAD -- production/epics/` is
  inspected, THEN the only modified files under
  `production/epics/` are this story file
  (`production/epics/hand-ui/story-027-hand-reserve-strip-cleanup.md`)
  and `production/epics/hand-ui/EPIC.md` (story-list row
  refresh at `/story-done` time only, not at `/dev-story`
  time). Story 011 / 005 / 010 / 014 files NOT modified.
  Story 018 (`opp-figurine-mana-cleanup`) parent-row file
  NOT modified by this story's `/dev-story` worker; the
  parent-row `/story-done` paperwork is a separate prompt.

- [ ] **AC10 — ADR-002 + ADR-021 binding preserved**:
  GIVEN the implementation commit, WHEN inspected, THEN:
  - The new systems (if any) read only client-local mirror
    resources (`Res<HandCardCatalog>`, `Res<HandUiMode>`,
    `Res<CurrentClientPhase>`, per-slot
    `&ReserveStripForFanSlot`, per-slot `&FanSlotIndex`)
    and write only `Text` content, `Visibility`,
    `BackgroundColor` / `BorderColor` / `TextColor`, or
    the optional a11y-label entity / attribute.
  - No new `S2C*` / `C2S*` message; `shared/src/protocol.rs`
    diff is empty.
  - `liv-bevy-lightyear` NOT activated.

- [ ] **AC11 — No `production/` shared-tracker edits by
  `/dev-story`**: GIVEN the implementation commit, WHEN
  `production/sprint-status.yaml`, `production/sprints/`,
  `production/qa/`, `production/stage.txt`,
  `production/session-state/`, and the PROMPT 761
  gate-check artifact are diffed, THEN none is modified
  by this story's `/dev-story` worker. (The parallel
  `/story-done` paperwork prompt — which is a separate
  worker — does edit `production/sprint-status.yaml` and
  related trackers per the standard `/story-done`
  precedent; that's outside this AC's scope.)

- [ ] **AC12 — Carried conditions preserved**: GIVEN the
  evidence and the implementation commit, WHEN inspected,
  THEN no claim is made against any of: `S8-QA-001-W1`,
  `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a`,
  `S11-HUD-TIMER-EYEBALL-VISUAL-001` (human-blocked),
  PROMPT 761 `Polish->Release` gate-check retry, stage
  advance from `Polish`, R1 drag-pipeline-dead bug,
  Sprint 12 story 019 underlying drag-runtime question,
  TQ-S12-C1..C7, any AUDIT-1076-* finding outside
  AUDIT-1076-17 regression-lock, any SOURCE-1077-*
  finding, or any of the 24 PROMPT 1022 audit findings.

- [ ] **AC13 — Parent-row paperwork discharge claim
  recorded in evidence**: GIVEN the evidence document at
  `production/qa/evidence/sprint-19-hand-reserve-strip-cleanup/README.md`
  (or canonical-equivalent path), WHEN read, THEN it
  records the explicit claim that this story's
  implementation discharges
  `S17-UI-HUD-OPP-MANA-CLEANUP-001` AC3 / `AUDIT-1076-17`
  parent-row paperwork gap (citing PROMPT 1112 partial
  disposition + PROMPT 1175 `c842668` AC3 source repair +
  PROMPT 1279 `sprint_17_closeout.rows_carried_forward[1]`
  in the discharge basis). **The evidence document does
  NOT itself close the parent row**; closure is the
  producer's separate `/story-done` prompt on
  `S17-UI-HUD-OPP-MANA-CLEANUP-001`, which uses this
  evidence as its discharge basis.

- [ ] **AC14 — Visual evidence captured**: GIVEN the
  post-implementation build, WHEN browser / WASM captures
  (or documented ECS node-text-content samples) are taken
  for a staged-card-with-cost-positive scenario, THEN the
  evidence records one capture / sample showing the bare
  integer in the reserve strip and the canonical HUD
  `MANA n / N` strip rendering side-by-side without any
  duplicate-mana wording in the strip subtree. Per Story
  026 precedent, ECS text-content sampling is acceptable
  if pixel captures are infeasible.

- [ ] **AC15 — Friend-game-scope no-claim restated in
  evidence**: GIVEN the evidence document, WHEN read at
  the bottom, THEN it verbatim restates the friend-game-
  scope-only disposition and preserves all carried
  `QA-COND-0005` / `QA-COND-0006` / `PAW-TD-*-a` /
  `S8-QA-001-W1` accept-risk language. The optional AC5
  a11y-label addition is **friend-game-scope only**; the
  evidence document explicitly states "Standard-tier
  accessibility conformance NOT pursued by this row;
  `QA-COND-0005` accept-risk preserved verbatim".

---

## Likely Files (for the future /dev-story — DO NOT EDIT IN THIS AUTHORING RUN)

| Path | Anticipated change |
|------|--------------------|
| `client/src/ui/hand/mod.rs` | Narrow edits inside `spawn_reserve_strip` (line ~4061) and/or `set_reserve_value_text` (line ~4988) and/or the strip-subtree systems (lines ~1327, 1350, 3209). If AC5 chosen: insert a non-rendered a11y-label child / attribute on `ReserveStripValueText`. If AC6 chosen: additive `BorderColor` / `BorderRadius` / `TextColor` only. **No** change to the bare-integer contract (AC1) or the click-disable semantics (AC2). |
| `tests/integration/hand-ui/hand_ui_reserve_strip_cleanup_test.rs` (NEW) | Integration test per AC3 / AC4. ECS-query-driven subtree walk; lifecycle driver via direct resource insertion. |
| `tests/integration/hand-ui/hand_ui_plugin_scaffold_test.rs` (existing) | Update `HAND_UI_ENTITY_COUNT` expectation only if AC6 adds new pre-pooled entities. |
| `tests/unit/hand-ui/reserve_mana_strip_test.rs` (existing) | OPTIONAL: append additional bare-integer / subtree regression tests if AC3 coverage is split between unit + integration. The existing PROMPT 1175 `audit_1076_17_reserve_strip_text_has_no_reserve_or_current_wording_when_staged` test PASSES unchanged. |
| `design/gdd/hand-ui.md` | OPTIONAL: one-line addition to Rule 13 pinning "bare per-card allocation integer; no duplicate-mana wording across the strip subtree". Worker discretion. |
| `docs/architecture/tr-registry.yaml` | OPTIONAL: add `TR-HU-013 — Reserve strip cleanup: bare per-card allocation integer; no duplicate-mana wording across the strip subtree`. Worker discretion. |
| `production/qa/evidence/sprint-19-hand-reserve-strip-cleanup/README.md` (NEW) | Evidence document; AC3 / AC4 / AC5 / AC6 dispositions; AC13 parent-row discharge claim; AC14 visual evidence; AC15 friend-game-scope no-claim restatement. |
| This story file | Status flipped Draft → Ready by `/story-readiness` post Sprint 19 activation; Ready → Done by `/story-done`. |
| `production/epics/hand-ui/EPIC.md` | `/story-done` paperwork: refresh story row. PROMPT 1351 authoring adds the row in `Draft`. |

**Explicitly out of scope for the `/dev-story` worker**:

- `production/epics/hand-ui/story-011-reserve-mana-strip.md`
  (Story 011 Complete; NO EDIT).
- `production/epics/hand-ui/story-005-placement-submit-core.md`
  (Story 005 Complete; NO EDIT).
- `production/epics/hand-ui/story-010-submit-prevalidation.md`
  (Story 010 Complete; NO EDIT).
- `production/epics/hand-ui/story-014-placement-staged-disclosure-accessibility.md`
  (Story 014 Complete; NO EDIT).
- `production/epics/hud/story-018-opp-figurine-mana-cleanup.md`
  (parent row; status preserved; **the parent-row `/story-done`
  paperwork is a SEPARATE prompt — this story's `/dev-story`
  worker does NOT touch this file**).
- `client/src/ui/hud/` (canonical HUD mana strip; different
  owner module; NO EDIT).
- `shared/src/protocol.rs`, `server/`, `client/src/network/`.
- `production/sprints/*`, `production/sprint-status.yaml`,
  `production/stage.txt`, PROMPT 761 gate-check artifact.
- `Cargo.toml`, `Cargo.lock`, `.cargo/`, `.github/`,
  `Trunk.toml`.
- R1 drag-pipeline-dead repair (separate prompt).

---

## Out of Scope

- Server-side change.
- Protocol change (`shared/src/protocol.rs`).
- New Lightyear message.
- New client-side authority or optimistic mutations over
  reserve-mana spend.
- Standard-tier accessibility conformance
  (`QA-COND-0005` preserved; AC5 a11y-label is friend-game-
  scope only).
- Final-art (`PAW-TD-*-a`).
- Re-author of the canonical HUD `MANA n / N` strip under
  `client/src/ui/hud/`.
- Re-author of Story 011 reserve-mana-strip ceiling / disable
  logic (HU-25 / HU-26 / HU-27 contracts preserved verbatim).
- Reserve-strip layout / position / button-spacing changes
  beyond the additive `BorderColor` / `BorderRadius` /
  `TextColor` polish allowed under AC6.
- Closing the parent row `S17-UI-HUD-OPP-MANA-CLEANUP-001`
  (separate `/story-done` prompt; producer-owned).
- Sprint 19 activation.
- `/qa-plan sprint-19` authoring.
- `/dev-story`, `/story-readiness`, `/story-done` on this story
  under the authoring prompt.
- Polish → Release gate-check retry.
- Stage advance from Polish.

---

## QA Test Cases

*Drafted by qa-lead at story creation. The developer implements
against these — do not invent new test cases during
implementation.*

- **AC1 — Bare-integer contract across lifecycle**:
  - Given: `App` running `HandUiPlugin` in
    `ClientState::InSession`; populated `HandCardCatalog`
    with cards of cost {0, 1, 5}; `HandUiMode::Staging`.
  - When: lifecycle driven: empty → cost-0 staged →
    cost-positive staged at `reserve_amount=0` → `[ + ]`
    clicked twice → `[ - ]` clicked once → un-staged.
  - Then: at each step, every
    `(ReserveStripValueText, &Text)` result matches the
    AC1 bare-integer / empty contract.

- **AC2 — Button-state contracts preserved**:
  - When: `cargo test -p client
    --test hand_ui_reserve_mana_strip_test` runs.
  - Then: PASSES unchanged (Story 011 HU-25 / HU-26 / HU-27
    + PROMPT 1175 `audit_1076_17_...` regression all PASS).

- **AC3 — Subtree-scoped regression-lock**:
  - Given: AC1 cost-positive-staged state.
  - When: every `Text` node descendant of the
    `ReserveStripForFanSlot` root entity is collected via
    a `Children`-traversal helper.
  - Then: none contains `"Reserve "`, `"Current "`, or
    `"mana"` (case-insensitive on `"mana"`).
  - Repeat: at every lifecycle step from AC1.

- **AC4 — NEW integration-test bin**:
  - When: `cargo test -p client
    --test hand_ui_reserve_strip_cleanup_test` runs.
  - Then: PASSES with ≥7 sub-tests covering AC1 / AC2 /
    AC3 lifecycle states.

- **AC5 — OPTIONAL a11y-label polish**:
  - Given: AC5 path chosen by worker.
  - When: the screen-reader-only label entity / attribute
    is inspected.
  - Then: visible `Text` glyph remains bare integer
    (no visible UI change); non-rendered label carries
    accessible descriptor (worker-named exact wording).
  - If: AC5 path NOT chosen, the test bin records the
    N/A disposition and AC1..AC4 + AC6..AC15 close the
    story.

- **AC6 — OPTIONAL visual chrome polish**:
  - Given: AC6 path chosen by worker.
  - When: the implementation diff is inspected.
  - Then: chrome additions are additive only (no
    replacement / removal); click-disable semantics
    preserved; `HAND_UI_ENTITY_COUNT` bumped exactly by
    new pre-pooled entity count (if any).
  - If: AC6 path NOT chosen, test bin records N/A.

- **AC7 — Hand-ui regression suite**:
  - When: each listed test bin runs.
  - Then: all PASS.

- **AC14 — Visual evidence**:
  - Given: post-implementation build.
  - When: captures / samples at any canonical viewport with
    a staged cost-positive card.
  - Then: bare integer renders in strip; canonical HUD
    `MANA n / N` renders unchanged; no duplicate-mana
    wording in strip subtree.

---

## Performance Budget

Per ADR-021 Presentation steady-state budget of `< 1 ms` per
frame. The reserve-strip already runs per-frame layout / sync
systems (`apply_reserve_strip_layout_system`,
`sync_reserve_strip_state_system`); this story adds at most:

- One subtree-walk on the integration-test path only (test
  bin, NOT in the runtime hot path).
- Zero new runtime systems if AC5 / AC6 paths are NOT chosen.
- One new constant-time visibility-toggle if AC5 a11y-label
  path is chosen.
- One new constant-time per-frame `BorderColor` / `TextColor`
  toggle if AC6 visual-chrome path is chosen.

Expected per-frame runtime cost change: negligible (<0.05 ms
per HandBar instance).

---

## Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Worker re-introduces verbose "Reserve N + / Current N" wording while attempting AC6 visual chrome polish. | Low | High | AC3 BLOCKING subtree-walk regression-lock; AC7 Story 011 + Story 014 regression bins PASS gates. |
| Worker edits Story 011 / 005 / 010 / 014 to "amend reserve-strip semantics". | Low | High | Out-of-scope rule explicit; reviewer checks unrelated story files unchanged (AC9). |
| Worker closes the parent row `S17-UI-HUD-OPP-MANA-CLEANUP-001` from this story's `/dev-story`. | Low | High | Control Manifest forbidden rule; AC13 explicit (evidence records discharge claim; closure is the separate producer prompt). |
| Worker edits `client/src/ui/hud/` while attempting to "harmonise" the strip with the HUD mana strip. | Low | High | AC8 explicit empty-diff gate; out-of-scope rule. |
| Worker introduces a new Lightyear message to "broadcast reserve-strip allocation". | Low | High | AC10; `liv-bevy-lightyear` NOT activated. |
| Worker bumps `HAND_UI_ENTITY_COUNT` without matching new pre-pooled entity count. | Low | Medium | AC6 explicit "exactly by the number of newly pre-pooled entities"; reviewer counts. |
| Worker chooses AC5 a11y-label path and claims `QA-COND-0005` Standard-tier conformance. | Low | High | AC15 friend-game-scope no-claim restated; Control Manifest forbidden rule. |
| Worker activates Sprint 19 as a side effect of `/dev-story` paperwork. | Low | Medium | No-Claim Banner forbids; orchestrator dispatch precedence (Sprint 19 activation is a separate prompt). |

---

## Verification (orchestrator-side, before worker dispatch)

These are sanity checks for the orchestrator that emits the
`/dev-story` prompt, NOT for PROMPT 1351 itself:

- `production/sprint-status.yaml` top-level `sprint:` field
  reads `19` after Sprint 19 activation; the row for this story
  is `ready`.
- `production/stage.txt` reads `Polish` and is unchanged.
- `production/sprints/sprint-19.md` (when authored) shows the
  ACTIVATED banner and includes this row.
- PROMPT 761 `Polish->Release` gate-check FAIL preserved.
- `production/qa/qa-plan-sprint-19.md` (when authored)
  references this story.
- `/story-readiness` on this story returns READY.
- Story 011 / 005 / 010 / 014 unchanged on `origin/main`.
- Story 018 (`opp-figurine-mana-cleanup`) parent-row file
  unchanged on `origin/main` (still in `Partial / In Progress`
  per PROMPT 1112).
- PROMPT 1175 `c842668` is an ancestor of the Sprint 19
  activation tip; `client/src/ui/hand/mod.rs:4082-4099` and
  `client/src/ui/hand/mod.rs:4988-5007` still carry the
  PROMPT 1175 doc-comment / bare-integer contract.
- In-flight Sprint 18 lanes (PROMPT 1347 / 1348 / 1349) have
  landed or been closed before Sprint 19 activation; no
  in-flight worker is editing `client/src/ui/hand/mod.rs` at
  `/dev-story` dispatch time for this row.

---

## Parent-Row Discharge Contract

This section documents the relationship between this story
(`S19-UI-HAND-RESERVE-STRIP-CLEANUP-001`) and the parent row
`S17-UI-HUD-OPP-MANA-CLEANUP-001` whose AC3 paperwork gap it
discharges after implementation.

### Current state on `origin/main`

- `S17-UI-HUD-OPP-MANA-CLEANUP-001` -- **Partial / In
  Progress**. Per PROMPT 1112 partial-disposition paperwork
  (recorded under
  `sprint_17_partial_disposition` in
  `production/sprint-status.yaml`), AC1 + AC2 + AC4..AC15 are
  DELIVERED via PROMPT 1105 worker + PROMPT 1111 integration;
  AC3 is explicitly carried.
- AC3 source repair has **landed** on `origin/main` via
  PROMPT 1175 `c842668` (`dev-story(s17-hand-reserve-strip-
  microbadge-cleanup): drop verbose mana microbadge wording
  from reserve strip`). The verbose "Reserve N + / Current N"
  wording is removed; the value text is now a bare integer.
- The parent row's `/story-done` has **NOT** been run. Per
  PROMPT 1279 Sprint 17 closeout
  `rows_carried_forward[1]`, the parent row is carried into
  Sprint 18 as `in_progress` with the explicit non-claim
  "no silent done closure".
- PROMPT 1285 §2.3 + PROMPT 1301 `dropped_rows` recorded the
  preferred discharge path as the (then-named)
  `S18-UI-HAND-RESERVE-STRIP-CLEANUP-001` follow-up story;
  PROMPT 1351 authors that story under the renamed Sprint 19
  slug `S19-UI-HAND-RESERVE-STRIP-CLEANUP-001`.

### Discharge path (post-implementation)

When this story is implemented in Sprint 19 and its
`/story-done` paperwork closes it as `Done`:

1. The evidence document records an explicit AC13 discharge
   claim against `AUDIT-1076-17` / `S17-UI-HUD-OPP-MANA-
   CLEANUP-001` AC3 carry-forward.
2. A **separate** producer-owned `/story-done` prompt on the
   parent row `S17-UI-HUD-OPP-MANA-CLEANUP-001` uses this
   story's evidence document as its discharge basis. That
   prompt's scope is parent-row paperwork only — it does NOT
   touch `client/src/ui/hand/mod.rs` or any test bin under
   `tests/`, and it does NOT activate any sprint.
3. After both `/story-done` prompts land, the
   `sprint_17_partial_disposition` block in
   `production/sprint-status.yaml` is amended to record AC3
   as DELIVERED (citing the chained PROMPT 1175 `c842668`
   source repair + Sprint 19 follow-up + parent-row
   `/story-done` closure), and the AC16 HUD epic story-count
   refresh deferred by PROMPT 1112 lands.

### Explicit non-discharge by this authoring run

PROMPT 1351 (this authoring run) discharges **nothing**. It
authors the story file and the EPIC.md row only. All discharge
claims are reserved for the future `/dev-story` worker's
evidence document (AC13) and the future parent-row `/story-
done` paperwork prompt (out of scope here).

---

## Authoring Trail

- 2026-05-19 — PROMPT 1351 — Story file authored as future
  Sprint 19 candidate
  `S19-UI-HAND-RESERVE-STRIP-CLEANUP-001`. Worktree
  `D:\_DEV\Work\Claude-Code-Game-Studios\.claude\worktrees\
  prompt-1351-s19-hand-reserve-strip-cleanup-story-authoring`,
  branch `work/s19-hand-reserve-strip-cleanup-story-
  authoring-1351`, base
  `origin/main@1e9548f23f7f19d3f8e14591b731cdfbbdd57874`
  (latest origin/main at authoring time). Files touched by this
  authoring run: this file (NEW) and
  `production/epics/hand-ui/EPIC.md` (story-list row added).
  Predecessor candidate slug `S18-UI-HAND-RESERVE-STRIP-
  CLEANUP-001` (PROMPT 1263 §3, PROMPT 1285 §2.3, PROMPT 1301
  dropped) explicitly superseded by the Sprint 19 slug; no
  retroactive Sprint 18 pull-in is performed. AC3 source
  repair `c842668` (PROMPT 1175) is on `origin/main` and is
  the bare-integer contract this story regression-locks.

---

`027: S19-UI-HAND-RESERVE-STRIP-CLEANUP-001: DRAFT`
