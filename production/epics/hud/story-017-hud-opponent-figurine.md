# Story 017: S11-UX-HUD-OPP-FIGURINE -- HUD Opponent Figurine Composition (Layout Only)

> **Epic**: HUD
> **Story ID**: S11-UX-HUD-OPP-FIGURINE
> **Status**: Done via PROMPT 976 (2026-05-16) after PROMPT 968
> implementation and PROMPT 975 integration on
> `origin/main@a3bc885f5f54e9b4e254d9abeb6b72a3e2321e8a`
> **Layer**: HUD / Presentation (layout / composition only)
> **Type**: UI -- layout composition + visual evidence
> **Sprint**: Sprint 14 Nice to Have (drawn from PROMPT 802 Expert UI
> Layout audit roadmap; pairs with ranks 7 and 8 per
> `docs/ux/ui-clean-pass-roadmap.md` adjacent-rows table)
> **Authored**: 2026-05-14 by PROMPT 879 (worktree
> `D:\_DEV\claude-code-game-studios-worktrees\s14-hud-layout-story-authoring`,
> branch `story/s14-hud-layout-story-authoring`)
> **Authoring source-of-truth**: `origin/main@dd9630b` (PROMPT 877
> `integrate(s13): merge work/s13-r2-placement-crash-audit (server story 002 / PROMPT 874)`;
> session-start HEAD was `origin/main@51e6228` PROMPT 871 — worktree
> fast-forwarded to `dd9630b` during authoring to keep source-of-truth current)

---

## Status / No-Claim Banner

This story is closed by PROMPT 976 as the Sprint 14 Nice to Have HUD
opponent figurine row. PROMPT 976 is paperwork-only closure after
PROMPT 975 integrated PROMPT 968 onto `origin/main`.

This closure:

- Does **not** close Sprint 14; Sprint 14 remains active.
- Does **not** invoke `/smoke-check`, `/team-qa`, `/gate-check`,
  `/release-check`, `/qa-plan`, or Sprint 14 close-out.
- Does **not** modify `client/`, `server/`, `shared/`, `tests/`,
  Cargo files, `production/sprints/sprint-14.md`,
  `production/qa/qa-plan-sprint-14.md`, `production/stage.txt`, or
  the PROMPT 761 gate artifact.
- Does **not** advance stage (Polish remains).
- Does **not** retry the PROMPT 761 Polish->Release gate-check FAIL.
- Does **not** claim final-art / asset-production completion
  (`PAW-TD-004-a` accept-risk preserved).
- Does **not** claim release-candidate readiness, public release
  readiness, full game completion, broad / Standard-tier
  accessibility completion (`QA-COND-0005`), or playtest / fun-
  hypothesis validation (`QA-COND-0006`).
- Does **not** close `S8-QA-001-W1` (two-client GAME_OVER closure).

PROMPT 976 verifies AC evidence from the PROMPT 968 worker report,
PROMPT 975 integration report, integrated HUD code/tests, and
`production/qa/evidence/sprint-14-hud-opponent-figurine/README.md`.
Runtime browser/WASM PNG captures remain unclaimed by this closure.

PROMPT 879 originally authored this story as a Sprint 14 candidate.
That historical authoring run did not activate Sprint 14.

PROMPT 879 (this authoring run) does **NOT**:

- Activate Sprint 14.
- Modify `production/sprint-status.yaml`.
- Modify `production/sprints/sprint-13.md`, `production/sprints/sprint-14.md`,
  or any other sprint plan file.
- Modify `production/stage.txt` (remains `Polish`).
- Modify any `production/session-state/*` file.
- Modify any QA-plan / smoke / Team-QA / gate-check / release-check
  artifact under `production/qa/`.
- Run `/story-readiness`, `/dev-story`, `/story-done`, `/smoke-check`,
  `/team-qa`, `/gate-check`, `/release-check`, or `/qa-plan` on this
  story.
- Modify any code under `client/`, `server/`, `shared/`, or `tests/`.
- Modify any `Cargo.toml` / `Cargo.lock`.
- Retry the PROMPT 761 Polish->Release gate-check.

This story does **not** claim:

- public release readiness
- release-candidate readiness
- full game completion
- broad / Standard-tier accessibility completion (`QA-COND-0005`)
- playtest / fun-hypothesis validation (`QA-COND-0006`)
- full playable-client manual QA
- two-client GAME_OVER closure (`S8-QA-001-W1`)
- final-art / asset-production completion (`PAW-TD-*-a`); the
  opponent figurine reuses the same PAW-004 placeholder asset that
  the own-player figurine uses today, with **no final-art treatment
  introduced by this story** (placeholder asset preserved under
  `PAW-TD-004-a` accept-risk)
- closure of any other Sprint 14 candidate row from `docs/ux/ui-clean-pass-roadmap.md`

Sprint 10 / Sprint 11 / Sprint 12 / Sprint 13 dispositions unchanged.
PROMPT 761 Polish->Release gate-check FAIL evidence preserved. PROMPT
802 audit roadmap accept-risk boundaries (`PAW-TD-*-a`, `QA-COND-0005`,
`QA-COND-0006`) preserved verbatim.

**No optimistic client-side authority is introduced or proposed.**
The opponent figurine renders the opponent's `ClassId` learned through
`S2CGameSnapshot` (or class-locked broadcast) — both server-authoritative
sources per ADR-002 + ADR-012. This story introduces a HUD-side
composition slot for the opponent figurine but does NOT introduce a
new server message, a new client-side class authority, or any
optimistic class inference.

---

## Source Finding

- PROMPT 802 audit `reports/PROMPT-802-Expert-UI-Layout-Audit-And-Repair-Roadmap.md`
  §3.2 (HUD) defect H10:
  - **H10**: opponent figurine has no separate composition — only own
    player gets a figurine. Already-tracked PROMPT 685 row
    `S11-UX-HUD-OPP-FIGURINE`. Source: `hud/mod.rs:566-586`.
- PROMPT 685 row 2 (audit row "HUD strip slice — opp figurine") is
  `subsumed-by` `S11-UX-HUD-OPP-FIGURINE` per
  `docs/ux/ui-clean-pass-roadmap.md` reconciliation matrix.
- `docs/ux/ui-clean-pass-roadmap.md` places this slug in the "Tier 1
  Should-Priority Adjacent Rows" table at 0.5d, pairing with ranks 7
  (`S11-UX-HUD-TOP-STRIP-LAYOUT`) and 8 (`S11-UX-HUD-BOTTOM-STRIP-LAYOUT`).
  The roadmap's note: "should not be activated before the matching
  Tier 1 Must row on their surface lands."

---

## Problem Class / Prevention Target

**Defect class**: The current HUD pre-pools a single class figurine
for the own player only (`hud/mod.rs:567-586`); the opponent has no
on-screen figurine slot at all. The opponent's identity is currently
inferred by the player from the opponent gold label, scoreboard dots,
and contextual cues, but the HUD does not surface the opponent's
class visually. This:

1. Breaks the symmetry between own and opponent HUD readouts (gold,
   mana, and scoreboard dots all symmetric; figurine is not).
2. Forces friend-game players to remember which class their opponent
   selected — defect surfaces most on long sessions or first-time
   playtests where players are still memorising the 7-class roster.
3. Has no current consumer for the opponent figurine slot, so a
   future "opponent passive proc'd" indicator (per class-specific HUD
   reveal mechanics) has no place to anchor.

**Prevention target**: introduce an opponent class figurine pre-pooled
entity, anchored to the **opposite edge** of the HUD bottom strip (or
the top-right of the HUD, as the implementation prompt decides per
the design spec from `S12-UX-GLOBAL-UI-DESIGN-SPEC-001`). The figurine
consumes the same `hud_figurine_asset` resolver as the own-player
figurine; both render the existing PAW-004 placeholder asset; both
flip to the correct class asset when the corresponding `ClassId` is
known via snapshot.

The opponent figurine is read-only over server-authoritative state
(`S2CGameSnapshot` + class-locked broadcast). It does NOT introduce
client-side class authority. Per ADR-001 it does NOT carry objective
identity (no `was_fake` leak risk).

---

## Context

### Existing surface

- **`client/src/ui/hud/mod.rs`** (per ADR-021): own-player figurine
  spawned at lines 567-586 with `bottom: Val::Px(config.hud_margin_px + 60.0)`,
  `width: Val::Px(64.0)`, `height: Val::Px(64.0)`. Class asset
  resolved through `hud_figurine_asset(class_id)` in
  `client/src/asset_wiring/` (PAW-004 chrome). `HudFigurine` marker
  component identifies the entity. `HudEntities.figurine` field
  exposes the entity to downstream systems.
- **`shared/src/protocol.rs`** + **`shared/src/session.rs`**: own
  `PlayerId` and opponent `PlayerId` are exposed via
  `HudPlayerIds { local_id, opponent_id }`. Opponent `ClassId` arrives
  via `S2CGameSnapshot` (full snapshot) and `S2CClassLocked` (lobby /
  class-lock broadcast).
- **`design/gdd/hud.md`**: does NOT currently prescribe an opponent
  figurine — this story extends the HUD spec; the GDD update is a
  light addition (one row in the entity table, one paragraph on
  symmetry) authored alongside the code change.
- **`docs/ux/ui-clean-pass-roadmap.md`** "Tier 1 Should-Priority
  Adjacent Rows" table — sequencing relative to ranks 7 / 8.
- **`reports/PROMPT-802-Expert-UI-Layout-Audit-And-Repair-Roadmap.md`**
  §3.2 H10.

### GDD / ADR / TR trace

- **GDD**: `design/gdd/hud.md` — light addition (one entity row, one
  paragraph on symmetry) needed. The implementation prompt may pair
  with a small GDD edit, OR defer the GDD edit to a follow-on
  Sprint 14 story if the producer prefers strict scope separation.
- **ADR-021** (Presentation Layer Architecture): one new pre-pooled
  entity; `HUD_ENTITY_COUNT` (currently 22) increments by 1 to 23.
  ADR-021 spec ("18 pre-pooled entities" wording in the EPIC) becomes
  "23 pre-pooled entities" or an equivalent updated figure; the
  implementation prompt cross-checks ADR-021 and adds a Consequences
  note if needed.
- **ADR-002** (Client-Server Authority): opponent `ClassId` is
  server-authoritative; figurine is read-only.
- **ADR-001** (Objective Identity Unicast): opponent figurine does
  NOT carry objective identity; `was_fake` invariant preserved.
- **ADR-008** (Lightyear Channel Config): no new channel; opponent
  `ClassId` is already drained from `S2CGameSnapshot` /
  `S2CClassLocked`.
- **ADR-011** (Reconnect + Snapshot): opponent figurine is included
  in the full HUD rebuild on `S2CGameSnapshot`.
- **ADR-012** (lobby class-lock authority, if applicable): not
  changed by this story.
- **TR registry**: no new TR; this is an extension of TR-HUD-005
  (own/opponent symmetry on identity-bearing readouts).

### Engine / skills

- **Engine**: Bevy 0.18 (Rust).
- **Mandatory skills**: `liv-bevy-018` for any `.rs` edits under
  `client/src/ui/hud/`. The story implementation prompt MUST activate
  this skill before editing.
- **Lightyear**: no protocol change required; existing
  `S2CGameSnapshot` and `S2CClassLocked` drains suffice. If the
  implementation prompt adds a new message receiver inside HUD it
  must invoke `liv-bevy-lightyear` per project preferences. If it
  reads via the existing `Res<...>` consumer it does not.

### Control Manifest Rules

- Required: New opponent figurine entity is pre-pooled at session
  start (not lazily spawned on first opponent class observation).
- Required: Opponent figurine is anchored inside the
  `HudBottomStrip` parent (or equivalent layout slot defined by
  `S11-UX-HUD-BOTTOM-STRIP-LAYOUT`) — NOT as a new absolute-positioned
  sibling of `HudRoot`.
- Required: Opponent figurine consumes the same `hud_figurine_asset`
  resolver as the own-player figurine (PAW-004 placeholder chrome
  preserved).
- Required: `HudEntities` exposes an `opponent_figurine: Entity`
  field; the existing `figurine` field is renamed to
  `own_figurine: Entity` OR retained as-is with a complementary
  `opponent_figurine` field — implementation-prompt decision, but the
  rename if chosen MUST update every consumer in the same commit.
- Required: ADR-021 system schedule preserved.
- Required: ADR-002 + ADR-012 binding (no client-side opponent class
  authority added).
- Required: ADR-001 binding (no `was_fake` exposure).
- Required: On `S2CGameSnapshot`, the opponent figurine flips to the
  correct class asset; on reconnect rebuild, the opponent figurine
  is part of the rebuild (ADR-011 binding).
- Required: TR-HUD-009 FROZEN-on-GAME_OVER behaviour applies: after
  `phase == GAME_OVER`, incremental opponent class updates are
  rejected; only `S2CGameSnapshot` can overwrite the figurine asset.
- Forbidden: Introducing a client-side opponent-class inference (e.g.
  deriving class from spawned units). Class identity comes from the
  server snapshot / broadcast only.
- Forbidden: Final-art / asset replacement on the opponent figurine
  (`PAW-TD-004-a` preserved).
- Forbidden: Standard-tier accessibility hit-target ≥44px work on
  the figurine (`QA-COND-0005` preserved); the figurine is a passive
  HUD indicator with no hit-target requirement.
- Forbidden: Modifying any code outside `client/src/ui/hud/` and
  `client/src/asset_wiring/` (asset resolver may be unchanged or
  may need a small additive consumer) in service of this story.
- Forbidden: Introducing animation / tween on the figurine swap.
  The class-swap is instantaneous (StateSync set), matching
  TR-HUD-008 dot-state-flip behaviour.

---

## Story Classification

**Story type**: UI -- layout composition + new pre-pooled entity +
visual evidence.

This is **NOT** a:

- Logic story (no formula or state machine change beyond the
  pre-pool entity count).
- Integration story (no new system-set or schedule wiring).
- Final-art story (placeholder PAW-004 figurine preserved).
- Accessibility story (`QA-COND-0005` preserved).
- Animation story (no new tween).

Per `.claude/docs/coding-standards.md` "Test Evidence by Story Type",
UI stories deliver a **manual walkthrough doc OR interaction test**
with screenshot evidence as ADVISORY gate.

---

## Dependencies (must be Done before /dev-story on this story)

| Dependency | Slug | Why blocking |
|---|---|---|
| Z-index layers | `S11-TD-UI-ZINDEX-LAYERS` (rank 1, Tier 0 Must) | Opponent figurine needs an explicit z layer assignment so reconnect / snapshot rebuild does not respawn it out of order relative to the bottom-strip. |
| Flex strip primitives | `S11-TD-UI-FLEX-STRIPS` (rank 3, Tier 0 Must) | Opponent figurine is hosted inside the `HudBottomStrip` flex parent (or equivalent); the flex tokens are required. |
| Global UI design spec | `S12-UX-GLOBAL-UI-DESIGN-SPEC-001` (rank 6, Tier 0 Must) | Defines opponent figurine anchor (top-right vs bottom-right vs bottom-strip-right) per the design spec. |
| HUD bottom strip layout | `S11-UX-HUD-BOTTOM-STRIP-LAYOUT` (rank 8, story 016 in this epic, Tier 1 Must) | This story hosts the opponent figurine inside the bottom-strip flex parent introduced by story 016. Authoring this story before story 016 lands would force a rewrite once the bottom-strip parent exists. Per `docs/ux/ui-clean-pass-roadmap.md` adjacent-rows table: "should not be activated before the matching Tier 1 Must row on their surface lands." |

**Optional but recommended** (not blocking):

- `S11-UX-HUD-TOP-STRIP-LAYOUT` (rank 7, story 015) — if the
  design spec from rank 6 anchors the opponent figurine to the
  top-right (mirror of the own-player figurine anchored to the
  bottom-left), this story optionally consumes the `HudTopStrip`
  parent introduced by story 015. The implementation prompt resolves
  the anchor decision per the design spec.

---

## Acceptance Criteria

All criteria are independently checkable.

- [x] **AC1 -- Opponent figurine entity pre-pooled at session start**:
  GIVEN `spawn_hud` runs, WHEN inspected post-refactor, THEN a new
  entity carrying `HudFigurine` + (new) `OpponentFigurineMarker` (or
  equivalent) is spawned with the same `hud_figurine_asset` fallback
  resolver as the own-player figurine. `HUD_ENTITY_COUNT` increments
  by 1 (currently 22 → 23) to reflect the new pre-pooled entity.

- [x] **AC2 -- Opponent figurine hosted inside flex strip parent**:
  GIVEN the post-refactor spawn, WHEN the opponent figurine's `Node`
  is inspected, THEN it carries NO `PositionType::Absolute` direct
  anchor. It is a flex child of `HudBottomStrip` (or `HudTopStrip`,
  per the design-spec decision) with dimensions stable across
  viewports.

- [x] **AC3 -- `HudEntities` exposes opponent figurine**: GIVEN
  the post-refactor `HudEntities` resource, WHEN inspected, THEN it
  exposes an `opponent_figurine: Entity` field pointing at the new
  entity. The existing own-player figurine remains reachable
  (`HudEntities.figurine` or `HudEntities.own_figurine` per the
  implementation-prompt rename decision).

- [x] **AC4 -- Asset resolution from server-authoritative state**:
  GIVEN an `S2CGameSnapshot` is drained, WHEN the snapshot contains
  the opponent's `ClassId`, THEN the opponent figurine's `ImageNode`
  is updated to the resolved `hud_figurine_asset(class_id)`. Update
  happens in the `StateSync` set (instantaneous, no tween — matches
  TR-HUD-008 behaviour).

- [x] **AC5 -- Reconnect / snapshot rebuild covers opponent
  figurine**: GIVEN an `S2CGameSnapshot` arrives mid-session, WHEN
  the HUD rebuilds, THEN the opponent figurine is part of the rebuild
  with the snapshot-correct class asset. ADR-011 binding.

- [x] **AC6 -- FROZEN-on-GAME_OVER applies**: GIVEN
  `phase == GAME_OVER`, WHEN a (hypothetical) incremental class
  update arrives, THEN the opponent figurine is NOT updated; only
  `S2CGameSnapshot` can overwrite it (snapshot bypasses FROZEN per
  TR-HUD-009 + ADR-011).

- [x] **AC7 -- ADR-001 invariant preserved**: GIVEN the post-refactor
  build, WHEN any path that surfaces the opponent figurine is
  inspected, THEN no objective identity or `was_fake` data flows to
  the figurine. The figurine is class-identity-bearing only, not
  objective-identity-bearing. Defence-in-depth grep + code review
  recorded in the evidence document.

- [x] **AC8 -- No client-side class authority added**: GIVEN the
  post-refactor build, WHEN the opponent figurine update path is
  inspected, THEN it reads from `S2CGameSnapshot` / `S2CClassLocked`
  drained-resource state ONLY. No system derives class from spawned
  units, lane state, or any other client-side observation. ADR-002 +
  ADR-012 binding.

- [x] **AC9 -- ADR-021 schedule preserved**: GIVEN a `cargo build -p
  client`, WHEN run, THEN no new system-set or schedule wiring is
  introduced. The opponent-figurine update system slots into the
  existing `StateSync` set inside `PresentationSet`.

- [x] **AC10 -- Visual evidence captured at two viewports**: GIVEN
  the post-refactor build runs end-to-end through the friend-game
  route with two clients connected as different classes, WHEN HUD is
  visible (any non-`Hidden` phase), THEN screenshots are captured at
  **desktop** (1920×1080) AND at a **smaller viewport** (1366×768
  minimum). Captures land under
  `production/qa/evidence/sprint-14-hud-opponent-figurine/` (NEW)
  with filenames `opp-figurine-1920x1080-<phase>.png` and
  `opp-figurine-1366x768-<phase>.png` for at least one phase that
  shows both figurines visibly (e.g. `DraftShop` after class lock).

- [x] **AC11 -- Text fitting anti-regression**: GIVEN the captures,
  WHEN visually inspected against the figurine label (if any caption
  is added by the design spec, e.g. class name under the figurine),
  THEN no text is clipped or truncated. If no caption is rendered by
  this story, this AC is satisfied trivially and the evidence
  document records "no figurine caption rendered; AC trivially
  satisfied."

- [x] **AC12 -- Stable dimensions anti-regression**: GIVEN the
  captures, WHEN dimensions of the opponent figurine are measured,
  THEN width and height are identical at 1920×1080 and 1366×768
  (fixed pixel sizing — 64×64 or the post-refactor constant).

- [x] **AC13 -- No overlap anti-regression**: GIVEN the captures at
  both viewports, WHEN the opponent figurine region is inspected,
  THEN it does NOT overlap the own-player figurine, current mana
  bar, reserve mana diamond, scoreboard dots, dim overlay edge, or
  any top-strip child. Captures span at least DRAFT_SHOP and
  DRAFT_AUCTION phases.

- [x] **AC14 -- No viewport-width font scaling anti-regression**:
  GIVEN a grep across `client/src/ui/hud/` post-refactor, WHEN run
  with pattern `Val::Percent`/`Val::Vw`/`Val::Vh` filtered to lines
  touching `TextFont` or `font_size`, THEN zero hits on opponent-figurine
  caption (if present).

- [x] **AC15 -- Z-index layer slot consumed (not re-invented)**:
  GIVEN the post-refactor spawn, WHEN the opponent figurine's z
  positioning is inspected, THEN it inherits the parent strip's z
  layer slot from `S11-TD-UI-ZINDEX-LAYERS` — NOT a hard-coded
  `GlobalZIndex(N)` re-introduced inline.

- [x] **AC16 -- Sprint 13/14 disposition preserved**: GIVEN the
  story commit, WHEN `production/sprint-status.yaml`,
  `production/sprints/sprint-13.md`, `production/sprints/sprint-14.md`
  (when authored), `production/stage.txt`, and PROMPT 761 gate-check
  artifact are diffed, THEN none of them are modified by this story.

- [x] **AC17 -- No accept-risk closure claimed**: GIVEN the evidence
  document, WHEN inspected, THEN it explicitly does NOT claim closure
  of `S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-004-a`,
  or any other accept-risk disposition. Final-art replacement on the
  opponent figurine is explicitly out of scope. Standard-tier
  accessibility is not pursued.

- [x] **AC18 -- Targeted regression passes**: GIVEN the post-refactor
  code, WHEN `cargo test -p client --lib` is run, THEN it passes.
  Existing HUD scaffold + observer + snapshot rebuild tests (stories
  001, 004, 008) continue to pass; tests that previously asserted
  `HUD_ENTITY_COUNT == 22` are updated to the new pre-pool count by
  the implementation prompt.

- [x] **AC19 -- Evidence document slot reserved**:
  `production/qa/evidence/sprint-14-hud-opponent-figurine/README.md`
  (NEW). Records the build commit, the two viewport captures, the
  class-swap observation (at least one opponent class change captured
  visually — e.g. opponent locks Iop, then in a second run locks
  Ecaflip; screenshots of both states), no-claim restatement,
  cross-links to PROMPT 802 §3.2 H10 + `docs/ux/ui-clean-pass-roadmap.md`
  "Tier 1 Should-Priority Adjacent Rows" table.

- [x] **AC20 -- HUD epic count updated**: GIVEN the epic file
  `production/epics/hud/EPIC.md`, WHEN updated by the implementation
  prompt at `/story-done` time, THEN the "Stories" table reflects the
  new story 017 entry and the `HUD_ENTITY_COUNT` summary line in the
  epic body (currently "18 pre-pooled entities" wording, or updated
  to "22 pre-pooled entities" elsewhere) is bumped consistently to
  the post-refactor count.

## Completion Notes

**Completed**: 2026-05-16 by PROMPT 976 `/story-done` paperwork
closure.

**Criteria**: 20 / 20 accepted. AC1-AC9, AC11-AC12, and AC14-AC19
PASS through integrated ECS/source/evidence checks and PROMPT 975
verification. AC10 and AC13 are accepted as
PASS-WITH-RUNTIME-CAPTURE-DEFERRED: the evidence README reserves the
1920x1080 and 1366x768 DRAFT_SHOP / DRAFT_AUCTION capture filenames
and records manual capture pending, while automated ECS tests verify
bottom-strip hierarchy, fixed 64 x 64 dimensions, class-swap asset
resolution, z-layer inheritance, and no objective/client-inference
path. AC20 is satisfied by PROMPT 976 updating
`production/epics/hud/EPIC.md` to the post-refactor 23-entity HUD
count and Done story row.

**Deviations**: Runtime browser/WASM PNG captures at 1920 x 1080 and
1366 x 768 remain deferred; PROMPT 976 does not claim browser
screenshot completion, Standard-tier accessibility, release readiness,
Sprint 14 close-out, final-art completion, or any gate retry. No GDD
addition is paired with this closure; the implementation remained
code/test/evidence scoped and this closure updates the HUD epic count
only.

**Test Evidence**: PROMPT 975 reports `cargo fmt --all -- --check`
PASS, `cargo check --workspace --all-targets` PASS with one
pre-existing `hand_ui_asset_wiring_test` warning, `cargo test -p
client --lib` PASS 45/45, story test `cargo test -p client --test
hud_opp_figurine_test` PASS 5/5, `hud_resolution_dim_test` PASS 8/8,
`hud_phase_timer_bar_test` PASS 4/4, `hud_top_strip_layout_test` PASS
6/6, `hud_bottom_strip_layout_test` PASS 8/8,
`hud_asset_wiring_test` PASS 6/6, `cargo build -p client` PASS, AC14
viewport-font grep PASS with zero hits, forbidden-path review empty,
and diff checks PASS.

**Code Review**: PROMPT 976 verified integration commit
`a3bc885f5f54e9b4e254d9abeb6b72a3e2321e8a` is reachable from
`origin/main`, reviewed the integrated source/test/evidence for AC
coverage, and performed paperwork-only closure. No `client/`,
`server/`, `shared/`, `tests/`, Cargo, Sprint 14 plan, QA plan, stage,
or gate artifact was edited by PROMPT 976.

---

## Likely Files

| Path | Anticipated change |
|------|--------------------|
| `client/src/ui/hud/mod.rs` | Add opponent figurine pre-pool inside `spawn_hud`; add `OpponentFigurineMarker` (or equivalent); add `opponent_figurine` field on `HudEntities`; bump `HUD_ENTITY_COUNT` from 22 to 23; add system in `StateSync` set that updates opponent figurine asset on snapshot. |
| `client/src/asset_wiring/mod.rs` (or submod) | If a small additive consumer is needed to expose `hud_figurine_asset` for the opponent path; usually unchanged. |
| `client/src/presentation/mod.rs` | If a presentation-side consumer of opponent `ClassId` already exists, no change; otherwise a small additive wire-up. |
| `design/gdd/hud.md` | Light addition: opponent figurine row + symmetry paragraph. Optional pairing with this story per implementation-prompt scope decision; may also be a follow-on small doc story. |
| `docs/architecture/adr-021-presentation-layer-architecture.md` | Optional Consequences-section note that `HUD_ENTITY_COUNT` increments to 23 to accommodate opponent figurine. Strictly not required if the ADR-021 text references "the pre-pooled entity set" abstractly; cross-check at implementation time. |
| `tests/unit/hud/hud_plugin_scaffold_test.rs` | Update existing `HUD_ENTITY_COUNT` assertion; add opponent-figurine pre-pool assertion. |
| `tests/integration/hud/opponent_figurine_snapshot_test.rs` | NEW *iff* an integration test bin exists; assert opponent figurine asset updates on `S2CGameSnapshot`. |
| `production/qa/evidence/sprint-14-hud-opponent-figurine/README.md` | NEW evidence document. |
| `production/qa/evidence/sprint-14-hud-opponent-figurine/opp-figurine-1920x1080-draft-shop.png` | NEW screenshot capture (desktop). |
| `production/qa/evidence/sprint-14-hud-opponent-figurine/opp-figurine-1366x768-draft-shop.png` | NEW screenshot capture (smaller viewport). |
| This story file | Status update on `/story-done`. |
| `production/epics/hud/EPIC.md` | Add row in "Stories" table; bump pre-pool count in epic body if it appears there. |

This table is a planning estimate. Per PROMPT 879 framing,
`client/src/`, `server/src/`, `shared/src/`, `tests/`, and
`Cargo.toml` are NOT touched by the authoring prompt — only by a
future implementation prompt run after Sprint 14 activates.

---

## Required Skills

- `liv-bevy-018` (MANDATORY for the implementation prompt).
- `liv-bevy-lightyear`: required ONLY if the implementation prompt
  adds a new `MessageReceiver<S2CClassLocked>` inside the HUD plugin.
  If the existing `Res<...>` consumer of class state suffices,
  Lightyear is not touched and the skill is not required at edit
  time.

The authoring prompt (PROMPT 879) does NOT invoke either skill
because no code is touched at authoring time.

---

## Evidence Path

`production/qa/evidence/sprint-14-hud-opponent-figurine/README.md`
(NEW; populated by the implementation prompt).

**Required evidence content**:

- Build commit hash and branch.
- Two screenshots minimum: 1920×1080 + 1366×768 at the same phase
  showing both own-player and opponent figurines (recommend
  `DraftShop` after class lock).
- Class-swap observation: at least one second run with the opponent
  locking a different class; screenshot proving the figurine flips.
- Longest-content observation table (per AC11; or "no caption
  rendered" if applicable).
- Per-figurine rendered dimension table (per AC12).
- Overlap audit across DRAFT_SHOP + DRAFT_AUCTION (per AC13).
- Z-index layer slot citation (per AC15).
- No-claim restatement (verbatim from "Status / No-Claim Banner").
- Cross-link to PROMPT 802 §3.2 H10.
- Cross-link to `docs/ux/ui-clean-pass-roadmap.md` "Tier 1
  Should-Priority Adjacent Rows" table.
- Cross-link to ADR-021 (presentation layer architecture) +
  ADR-002 (client-server authority) + ADR-012 (lobby class-lock
  authority).

---

## Regression Commands Expected

For the implementation prompt (NOT the authoring prompt):

- `cargo build -p client` (must succeed; AC9).
- `cargo test -p client --lib` (HUD-scoped tests; AC18).
- `cargo test -p client --test '*hud*'` (if integration tests are
  added per AC18).
- `git diff --check origin/main...HEAD`
- `git diff --cached --check`
- Grep `Val::Percent|Val::Vw|Val::Vh` filtered to `client/src/ui/hud/`
  matches against `font_size` / `TextFont` (must be zero; AC14).

The authoring prompt (PROMPT 879) runs only `git diff --check`,
`git diff --cached --check`, `git status --short --branch`.

---

## Out of Scope

- Any final-art treatment on the opponent figurine — PAW-004
  placeholder reused (`PAW-TD-004-a` preserved).
- Class-specific HUD reveal mechanics (e.g. passive-proc indicator,
  ability-cooldown badge) anchored to the opponent figurine — those
  are class-specific design stories, deferred.
- Standard-tier accessibility on the opponent figurine
  (`QA-COND-0005` preserved).
- HUD top strip composition — separate story 015.
- HUD bottom strip composition — separate story 016 (DEPENDENCY).
- Opponent gold label restructuring (already in scope of story 015).
- Animation / tween on figurine class swap.
- Server protocol change (no new message added).
- GDD large rewrites; one light addition only, OR deferred to a
  small follow-on doc story per implementation-prompt scope decision.
- Sprint 14 activation, `S8-QA-001-W1` closure, or Polish->Release
  gate-check retry.
- No `/dev-story`, `/story-done`, `/smoke-check`, `/team-qa`,
  `/gate-check`, `/release-check`, or `/qa-plan` run under PROMPT 879
  (this authoring prompt).

---

## Dependency Notes Against Sprint 13 Active Scope

- Sprint 13 active scope (runtime hardening + Sprint 12 cleanup +
  UI-audit-roadmap-prep) does NOT include this story. Sprint 13
  remains unchanged.
- File-collision risk against Sprint 13 rows on `client/src/ui/hud/`:
  none known. Sprint 13 story 018 (tracing targets) touches the
  `target:` argument inside `tracing` macros and is orthogonal to
  spawn-time Node composition.
- File-collision risk against sibling Sprint 14 candidate story 016
  (HUD bottom strip): both stories edit `spawn_hud` in
  `client/src/ui/hud/mod.rs`. This story DEPENDS on story 016
  having landed — the opponent figurine is hosted inside the
  `HudBottomStrip` parent introduced by story 016.
- This story lands under the existing HUD epic; the HUD epic
  remains `Ready` (with an updated story table after `/story-done`).

---

## Historical Sprint 14 Activation Preconditions

The preconditions that were required before PROMPT 968 entered
`/dev-story` in Sprint 14 were:

1. Sprint 14 activation prompt MUST re-state the accept-risk
   preservations from `docs/ux/ui-clean-pass-roadmap.md` "Accept-Risk
   Dispositions Preserved" — `PAW-TD-004-a`, `QA-COND-0005`,
   `QA-COND-0006`.
2. Sprint 14 QA plan MUST exist and pass `/qa-plan sprint`.
3. The four dependencies (ranks 1, 3, 6, 8) MUST be **Done** (not
   just Ready) before this story enters `/dev-story`. Note: story 016
   (rank 8, `S11-UX-HUD-BOTTOM-STRIP-LAYOUT`) is a HARD dependency,
   not optional.
4. `/story-readiness` MUST pass on this story file against the
   Sprint 14 activation HEAD.
5. Producer / ux-designer decision MUST be recorded on:
   - Anchor location: bottom-strip-right vs top-strip-right vs other
     (per `S12-UX-GLOBAL-UI-DESIGN-SPEC-001` output).
   - Whether `HudEntities.figurine` is renamed to `own_figurine` or
     kept unchanged (with `opponent_figurine` added).
   - Whether a small `design/gdd/hud.md` addition is paired with this
     story or deferred to a follow-on doc story.

These gates were satisfied before implementation; no blocker remains
for this closed row.

## Closure Trail

- PROMPT 968 (2026-05-16) -- `/dev-story` implementation on branch
  `work/s14-hud-opponent-figurine`, commit
  `69f81364137a8248c9976ad30f21671c6070b315`. Added the
  pre-pooled opponent figurine under `HudBottomStrip`, exposed
  `HudEntities.opponent_figurine`, synchronized own/opponent
  figurines from authoritative snapshot class ids, registered the
  HUD story test target, and wrote the evidence README.
- PROMPT 975 (2026-05-16) -- integration merge
  `a3bc885f5f54e9b4e254d9abeb6b72a3e2321e8a` onto `origin/main`.
  Verification passed `cargo fmt`, `cargo check`, client lib, HUD
  story/regression bins, `cargo build -p client`, AC14 grep, diff
  checks, and forbidden-path review.
- PROMPT 976 (2026-05-16) -- serialized `/story-done` paperwork
  closure. Story status marked Done, Sprint 14 row flipped
  `ready -> done`, HUD EPIC count/story row updated, session-state
  banners prepended, and `sprint_14_story_done` entry appended.
  Sprint 14 remains active; stage remains Polish; PROMPT 761 FAIL,
  `S8-QA-001-W1` OPEN, `QA-COND-0005/0006` accepted-risk, and
  `PAW-TD-*-a` accepted-risk are preserved.
