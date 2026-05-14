# Story 019: S11-DRAG-RUNTIME-RETEST-TIGHTER-CAPTURE-001 — Tighter-Capture Diagnostic-Only Drag Runtime Retest

> **Epic**: Hand UI
> **Sprint**: Sprint 11 (active; Polish-stage; activated by PROMPT 773; QA plan landed by PROMPT 774)
> **Status**: Done -- closed by PROMPT 814 (`/story-done` paperwork) on
> `origin/main@a3c624e` with terminal disposition
> **`closed-with-conditions / evidence-captured-cannot-reproduce` after
> second time-box exhaustion** (PROMPT 807 + PROMPT 810). No production
> repair landed inside this story (story is diagnostic-only by design).
> The underlying drag-runtime bug is **NOT claimed fixed**; the
> regression is escalated to the PROMPT 804 Sprint 13 candidate
> runtime-hardening stories (mapping appended to evidence by PROMPT 807
> commit `a8ef42d`).
> **Layer**: Presentation
> **Type**: Integration — runtime evidence + diagnostic-only (NOT a code-change story; no repair commit may land inside this story under any disposition)
> **Authored**: 2026-05-13 (PROMPT 778)
> **Authoring source-of-truth**: `origin/main@d36bbbd` (PROMPT 774's QA-plan commit; PROMPT 778 lands this story file atomically with the story-018 evidence file under `production/qa/evidence/`).
> **Parent story**: `production/epics/hand-ui/story-018-drag-runtime-retest.md` (`HU-DRAG-RT-04` follow-on path: `cannot-reproduce`).
> **Parent evidence**: `production/qa/evidence/sprint-11-drag-runtime-evidence.md`.

---

## Status / No-Claim Banner (inherited verbatim from story 018)

This banner is restated **verbatim** from
`production/epics/hand-ui/story-018-drag-runtime-retest.md` §"Status / No-Claim
Banner" so that this story is self-contained for future readers and so that
no-claim preservation is verifiable by text search across both story files:

- Sprint 11 is `draft` per `production/sprints/sprint-11.md` and the
  `next_sprint:` block in `production/sprint-status.yaml`. This story is **not
  activated** and does **not** appear as an active row in
  `production/sprint-status.yaml`. Activation happens via `/sprint-plan
  sprint-11` in a separate prompt.

  > **Update on the activation clause only (not a no-claim relaxation)**: Sprint 11
  > was activated by PROMPT 773 at `07aafe2` (status flipped to `active`); the QA
  > plan landed at `d36bbbd` (PROMPT 774). The story-018 banner above was authored
  > pre-activation (PROMPT 766). The text is restated verbatim to preserve the
  > no-claim banner integrity. None of the no-claim clauses below are affected by
  > activation. **This story 019 specifically is NOT yet active in Sprint 11**:
  > `/sprint-plan sprint-11 --add story-019` is a separate prompt, not authored
  > here.

- This story authoring does **not** claim: public release readiness,
  release-candidate readiness, full game completion, broad / Standard-tier
  accessibility completion (`QA-COND-0005`), playtest / fun-hypothesis
  validation (`QA-COND-0006`), full playable-client manual QA, two-client
  GAME_OVER closure (`S8-QA-001-W1`), or final-art / asset-production
  completion.

- `production/stage.txt` remains `Polish`. No `/dev-story`, `/smoke-check`,
  `/team-qa`, `/gate-check`, or `/story-done` is run by the authoring of this
  story file.

**No optimistic client-side authority is introduced or proposed by this story
or by any disposition pathway recorded in §"Acceptance Criteria".** ADR-002 and
ADR-009 lines are binding for any follow-on repair story that may be authored
after this story's run.

---

## Context

**Parent context**: story 018 was dispositioned `cannot-reproduce` by PROMPT 778
because the 1.0-day operator-driven time-box could not be exercised in an
automated CLI dispatch. The 5 S1-S5 instrumentation emit sites are present in
the worker source (PROMPT 706 / 709 / `7e0c663`) and the drag-ended gate
widening (PROMPT 697 / `cbb2565`) is present, but no runtime trace was
captured to fill the truth-table.

**GDD** (inherited from story 018):
- `design/gdd/hand-ui.md` — placement drag-to-stage state machine.
- `design/ux/hand-ui.md` — State machine (Dragging card / Valid board target
  hover / Staged board card / Un-staging, L210–230).

**ADR Governing Implementation** (inherited from story 018; all three remain
binding for this story and for any follow-on repair):
- [ADR-021: Presentation Layer Architecture](../../docs/architecture/adr-021-presentation-layer-architecture.md)
- [ADR-002: Client-Server Authority](../../docs/architecture/adr-002-client-server-authority.md) — **no optimistic client-side authority is allowed.**
- [ADR-009: RSM Phase State](../../docs/architecture/adr-009-rsm-phase-state.md)

**Engine**: Bevy 0.18 | Lightyear 0.26 | **Risk**: HIGH (gameplay-blocking
divergence in friend-game runtime; tests-green but runtime-broken,
unconfirmed).

**Mandatory skills**:
- `liv-bevy-018` — for any read/review/edit of Bevy `.rs` code touched by this
  retest.
- `liv-bevy-lightyear` — for the **upgraded `lightyear=debug`** capture in this
  story; the protocol-shape reading scope is wider than story 018.

**Primary suspect named (not claimed)**: per the PROMPT 683-era evidence
summary, the failing edge is hypothesised to be **S5 (release branch) →
server** (8 `C2SActivateCard` sends with zero `stage_or_update` events). This
story does **not** claim S5 is the actual failing edge — that claim must come
from a captured trace.

**Control Manifest Rules (Presentation Layer, retest-scope — inherited verbatim
from story 018)**:
- Required: All retest reasoning must be gated on `HandUiMode::Staging` (or
  `active_drag.is_active()` for follow/move/end paths).
- Required: Pointer button checks must require `PointerButton::Primary`.
- Required: Phase state read via `Res<CurrentClientPhase>`;
  `MessageReceiver<S2CPhaseChanged>` is never drained directly by Hand UI.
- Required: `liv-bevy-018` skill applies to all `On<Pointer<...>>` observer
  signatures touched.
- Forbidden: Introducing client-side optimistic authority for stage / activate
  / submit (ADR-002 / ADR-009 line still binding).
- Forbidden: Synthesising bespoke `WindowEvent` / cursor-position polling;
  rely on `bevy_picking` events for input.
- Forbidden: Modifying `S2C*` / `C2S*` protocol shapes inside this retest. Any
  protocol-shape divergence discovered must be authored as a separate
  follow-on story.

---

## Goal

Capture an authoritative runtime trace of the PLACEMENT drag-to-stage flow on
a real two-client friend-game session with a **tighter capture protocol** than
story 018, fill the S1-S5 truth-table with at least one PASS or FAIL row per
column (A / B / C / D), name the offending stage if any column has a FAIL row,
and disposition the divergence into exactly one of the four outcomes story 018
§"Goal" enumerates (`bug-reproduced`, `bug-fixed`, `cannot-reproduce`,
`third-party-limitation`).

**Tighter capture deltas relative to story 018**:

1. Upgrade `lightyear=info` to `lightyear=debug` in the `RUST_LOG` invocation
   (story 018 explicitly reserves this upgrade for a follow-on run because
   replication-channel `trace` produces excessive noise that buries S1-S5
   evidence, but `debug` is a workable middle ground for protocol-shape
   diagnosis without the `trace`-level noise).
2. Capture a **frame-level video** of the release moment for drag attempts
   A / B / C / D, not just a single-frame screenshot. The grey-square symptom
   is a 1-frame visual flash; a video makes the originating frame
   unambiguous.
3. Capture **both client logs simultaneously** with synchronised wall-clock
   timestamps (e.g., `RUST_LOG=…` plus `RUSTLOG_TIME_FMT=%Y-%m-%dT%H:%M:%S%.3fZ`
   if available, or a wrapper shell that prepends `date -u +%FT%T.%3NZ` to
   each log line) so that the S2 → S5 producer-consumer cross-check between
   client A's release and server's `stage_or_update` reception is unambiguous.
4. Capture **server-side `C2SActivateCard` reception and `stage_or_update`
   outcome at `server::game=debug`** (one notch higher than story 018's
   `server::game=info`) — this is the specific edge the PROMPT 683-era
   evidence implicates.

**No optimistic client-side authority is introduced by any of (1)–(4).**

---

## Acceptance Criteria

All criteria are independently checkable.

- [ ] **HU-DRAG-RT-19-01 — Tighter-capture runtime trace captured.** A real
  two-client friend-game session is run with the **upgraded `RUST_LOG`**
  invocation below; both clients connect; PLACEMENT phase is reached; at
  least one card from each player's hand is drag-attempted onto a valid
  `BoardCell` target and at least one drag-attempt is made on an `Instant`
  card. Raw log capture is preserved at
  `production/qa/evidence/captures/sprint-11-drag-runtime/` (use a dated
  subdirectory per the existing `manual-friend-game-evidence-YYYY-MM-DD/`
  precedent if a second run is needed). Frame-level video of the release
  moment for A / B / C / D is captured.

- [ ] **HU-DRAG-RT-19-02 — S1-S5 truth-table locked with at least one observed
  row.** Every row of the S1-S5 truth-table is filled with one of {PASS, FAIL,
  NOT-OBSERVED} and a runtime-evidence pointer (log file + line range OR
  video timestamp). At least **one row per column (A / B / C / D)** must be
  PASS or FAIL — i.e., not every row is `NOT-OBSERVED` (which would be a
  repeat of story 018's outcome and would correctly disposition as
  `cannot-reproduce` again). The locked truth-table is committed to a new
  evidence file at `production/qa/evidence/sprint-11-drag-runtime-evidence-tighter.md`
  (do **not** overwrite the story 018 evidence file).

- [ ] **HU-DRAG-RT-19-03 — Test-vs-runtime divergence dispositioned.** The
  PROMPT 683-era discrepancy is dispositioned as exactly one of
  {bug-reproduced, bug-fixed, cannot-reproduce, third-party-limitation}. If
  the disposition is `bug-reproduced`, the offending stage (S1 / S2 / S3 /
  S4 / S5) is named explicitly with the failing trace line range.

- [ ] **HU-DRAG-RT-19-04 — Repair or follow-on authored.** Depending on
  `HU-DRAG-RT-19-03`'s disposition (rules inherited verbatim from story 018
  `HU-DRAG-RT-04`):
  - `bug-reproduced` → a concrete repair story is authored under
    `production/epics/hand-ui/` (or `production/epics/playable-client/` if
    cross-epic). **No repair commit lands inside this story.**
  - `bug-fixed` → the evidence file records the cumulative work that
    resolved the bug (PROMPT 696 / 697 / 706 / 709 cite) and the
    truth-table-PASS rows that prove it. No further follow-on is required.
  - `cannot-reproduce` (a second time) → escalation; the evidence file
    records the second time-box exhausted and surfaces the diagnostic gap
    to Sprint 12 candidate authoring; no further `cannot-reproduce` retest
    is authored at the same scope without expanded tracing.
  - `third-party-limitation` → the evidence file records the platform
    profile, the workaround (if any), the no-claim note that this is **not**
    a Hand UI / Board Rendering / protocol bug, and any documentation
    update needed in `docs/setup/` or `docs/architecture/`.

- [ ] **HU-DRAG-RT-19-05 — No production code changes in this story.** No
  edits land under `client/`, `server/`, `shared/`, or `tests/` as part of
  this story's `/dev-story`. The repair / repro path (if any) is delegated
  to the follow-on story authored in `HU-DRAG-RT-19-04`. Verified by
  `git diff --stat origin/main..work/<story-id>-drag-runtime-retest-tighter -- client/ server/ shared/ tests/`
  returning empty.

- [ ] **HU-DRAG-RT-19-06 — No optimistic client-side authority introduced.**
  The retest reasoning and any follow-on story authored under
  `HU-DRAG-RT-19-04` explicitly state that client-side authority for stage /
  activate / submit is forbidden (ADR-002 + ADR-009). Verified by text search
  for the phrase "no optimistic client-side authority" in the new evidence
  file and any authored follow-on story.

- [ ] **HU-DRAG-RT-19-07 — Non-claims preserved.** The new evidence file
  restates the no-claim banner from §"Status / No-Claim Banner" above
  verbatim and confirms none of the no-claim clauses are claimed closed.

- [ ] **HU-DRAG-RT-19-08 — Sprint 11 active status preserved.** This story
  does not touch `production/sprint-status.yaml`, `production/stage.txt`, or
  `production/sprints/sprint-11.md`. Verified by `git diff --stat
  origin/main..work/<story-id>-drag-runtime-retest-tighter --
  production/sprint-status.yaml production/stage.txt
  production/sprints/sprint-11.md` returning empty.

---

## Reproduction Recipe (tighter capture)

### Upgraded `RUST_LOG` invocation (server + each client)

```text
RUST_LOG=client::ui::hand=trace,client::presentation::board_rendering=trace,client::card_animations::input_gating=info,lightyear=debug,server::game=debug
```

Notes:
- `lightyear=debug` is the upgrade over story 018's `lightyear=info`. Use
  `debug`, **not** `trace` — `trace` produces replication-channel-level noise
  that buries S1-S5 attribution.
- `server::game=debug` is the upgrade over story 018's `server::game=info`.
  This level is required to catch `C2SActivateCard` reception and
  `stage_or_update` outcome (the specific edge the PROMPT 683-era evidence
  implicates).
- The three `=trace` targets (`client::ui::hand`, `client::presentation::board_rendering`,
  `client::card_animations::input_gating`) are unchanged — these cover the
  five emit sites recorded at `7e0c663`.
- Adjust target paths to match local crate / module layout if the canonical
  names above have moved (verify via `git grep -n
  "drag_sprite_visible_flip\\|fan_active_default_drop\\|placement_cursor_move\\|drag_lift_tween_install\\|spawn_highlight_state_change"
  -- client/src/`).
- If the operating-system shell strips commas inside `RUST_LOG`, wrap the
  whole value in double quotes per the host shell's conventions.

### Synchronised wall-clock timestamp prefixing

Each log producer (server, client A, client B) must prefix its output with a
synchronised UTC timestamp at millisecond precision so that the S2 → S5
producer-consumer cross-check is unambiguous. Suggested approaches:

- Bevy 0.18 default tracing subscriber: enable `tracing_subscriber::fmt`
  with `.with_timer(...)` set to a UTC clock at millisecond precision.
  (Verify on the worker branch: check whether the existing tracing init in
  `client/src/main.rs` / `server/src/main.rs` already emits ISO-8601 UTC
  timestamps; if not, this is a tracing-init change scoped to **the
  follow-on diagnostic-only run only** — it does NOT land as a permanent
  code change inside this story.)
- Shell wrapper: pipe the binary's stderr through `awk
  '{ system("date -u +\"%Y-%m-%dT%H:%M:%S.%3NZ\""); print }'` or PowerShell
  equivalent.

### Friend-game route

Identical to story 018 §"Friend-game route" steps 1–8, but with the upgraded
`RUST_LOG`. The four drag attempts (A / B / C / D — standard unit onto
BoardCell, Instant onto fan plate, cancel mid-drag, invalid drop) are the
same.

**New requirement on top of story 018**: video-record the screen for the
entire PLACEMENT phase on both clients (OBS Studio, ShareX, or built-in OS
screen recorder; 30 fps minimum). The video lets the truth-table reviewer
identify the exact frame at which the grey-square symptom (if any) appears
and cross-reference that frame's wall-clock timestamp against the log lines
to attribute the originating S-row unambiguously.

### Time-box

This story is time-boxed to **1.5 days** (longer than story 018's 1.0 day
because the tighter capture requires synchronised timestamp wiring and video
recording setup). If S1-S5 cannot be filled with at least one PASS/FAIL row
per column within 1.5 days:

- Lock the truth-table as best-effort with `NOT-OBSERVED` rows explicitly
  named.
- Disposition `HU-DRAG-RT-19-03` as `cannot-reproduce` (second time).
- Escalate the diagnostic gap to Sprint 12 candidate authoring; do not author
  a third same-scope retest without expanded tracing scope.

---

## S1-S5 Truth Table (to be filled by retest evidence)

Same shape as story 018 §"S1-S5 Truth Table"; primary suspect named below.

| Stage | Description | Drag A (BoardCell) | Drag B (Instant fan-plate) | Drag C (cancel empty) | Drag D (invalid cell) |
|-------|-------------|--------------------|-----------------------------|----------------------|------------------------|
| **S1** | Pointer Press observer fires on a `FanSlotIndex` entity (Primary, Staging) | _PASS / FAIL / NOT-OBSERVED_ | _ditto_ | _ditto_ | _ditto_ |
| **S2** | `HandUiPlacementDragStarted { card, owner_id }` emitted exactly once that tick | _ditto_ | _ditto_ | _ditto_ | _ditto_ |
| **S3** | Drag-start consumer flips `HandDragSprite` `Visibility` to `Visible` + board rendering ghost engages (BoardCell) OR fan-plate ghost engages (Instant) | _ditto_ | _ditto_ (no board ghost expected; `NOT-OBSERVED` is correct) | _ditto_ (cancel — ghost cleared expected) | _ditto_ (invalid target — ghost cleared expected) |
| **S4** | `Pointer<Move>` during active drag emits `HandUiPlacementCursorMoved`; drag-sprite `Node.left` / `Node.top` track cursor; input gating stays Staging | _ditto_ | _ditto_ | _ditto_ | _ditto_ |
| **S5** | **PRIMARY SUSPECT** — `Pointer<Release>` (Primary) emits `HandUiPlacementDragEnded`; drag-ended consumer routes to correct `PlacementTargetKind` branch; `active_drag.clear()` runs; `C2SActivateCard` reflects resolved target authoritatively (**no client-side optimism**) | _ditto_ | _ditto_ | _ditto_ (no `C2SActivateCard` send expected — `NOT-OBSERVED` is correct) | _ditto_ (snap-back expected; **no** `C2SActivateCard` send) |

**Primary suspect call-out**: PROMPT 683-era evidence reported 8
`C2SActivateCard` sends with zero `stage_or_update` events. **S5 is the
hypothesised failing edge** — confirm or refute with the captured server-side
`server::game=debug` trace. Do **not** assume S5 is failing without
captured evidence — the upper rows (S1 / S2 / S3 / S4) may surface unexpected
FAILs that the PROMPT 683-era summary missed (e.g., S4 cursor-moved emit
dropping under high pointer-event frequency, S2 message-write contention
under bevy 0.18 Required Components observers).

---

## QA Test Cases (manual; no automated test is added by this story)

- **HU-DRAG-RT-19-T1** — Two-client friend-game session reaches PLACEMENT
  with both players holding ≥1 card; both clients running the **upgraded**
  `RUST_LOG`; server running with `server::game=debug`; raw logs and
  frame-level video captured for drag-attempts A / B / C / D.

- **HU-DRAG-RT-19-T2** — For each of drag-attempts A / B / C / D, fill the
  S1-S5 row in the truth-table with PASS / FAIL / NOT-OBSERVED and a
  runtime-evidence pointer. At least one row per column must be PASS or FAIL.

- **HU-DRAG-RT-19-T3** — Disposition `HU-DRAG-RT-19-03` decided based on T2;
  matches exactly one of the four outcomes.

- **HU-DRAG-RT-19-T4** — Follow-on artefact authored per the chosen branch;
  the follow-on either lives at
  `production/epics/hand-ui/story-020-<slug>.md` or
  `production/epics/playable-client/story-<NNN>-<slug>.md`, or is explicitly
  stated as "no follow-on needed" in the new evidence file (only for the
  `bug-fixed` branch).

---

## Test Evidence

**Story Type**: Integration — manual runtime evidence (no new automated tests
added by this story).

**Required evidence**:
- `production/qa/evidence/sprint-11-drag-runtime-evidence-tighter.md` (NEW;
  the locked truth-table + disposition + no-claim restatement + follow-on
  pointer). Do **not** overwrite the story 018 evidence file.
- `production/qa/evidence/captures/sprint-11-drag-runtime/<dated-subdir>/`
  (NEW directory; per-attempt log files + screenshots + frame-level video).

**Status**: [ ] Captured and locked

---

## Out of Scope

Inherited verbatim from story 018 §"Out of Scope", plus:

- Modifying the tracing init in `client/src/main.rs` / `server/src/main.rs`
  to add permanent millisecond-UTC timestamp prefixing. Wall-clock
  timestamp wiring is per-run setup for the diagnostic capture only and
  does **not** land as a permanent code change inside this story. If the
  follow-on disposition determines that millisecond-UTC timestamps should
  be the project default, a separate small follow-on story is authored.
- Landing any repair commit under `client/`, `server/`, `shared/`, or
  `tests/`. Repair (if needed) is delegated to a follow-on story authored
  in `HU-DRAG-RT-19-04`.
- Modifying any `S2C*` / `C2S*` protocol shape inside this retest.
- Closing `S8-QA-001-W1` (two-client GAME_OVER manual gap).
- Standard-tier accessibility remediation (`QA-COND-0005`).
- Playtest / fun-hypothesis validation (`QA-COND-0006`).
- Activating Sprint 11 (already activated by PROMPT 773).
- Modifying `production/sprint-status.yaml`, `production/stage.txt`, or
  `production/sprints/sprint-11.md`.

---

## Dependencies

- **Depends on** (already on `main`, no work needed):
  - Story 017 (`HU-card-drag MVP`, commit `00ffe89`, PROMPT 696).
  - `S11-HU-DRAG-DROP-NON-INSTANT-001` (commit `cbb2565`, PROMPT 697).
  - `S11-OBS-GREY-SQUARE-ATTRIBUTION-001` (commit `7e0c663`, PROMPT 706 /
    709).
  - Story 018 (`production/epics/hand-ui/story-018-drag-runtime-retest.md`)
    dispositioned `cannot-reproduce` (parent evidence file
    `production/qa/evidence/sprint-11-drag-runtime-evidence.md`).
- **Depends on** (sprint-level prerequisites that gate `/dev-story` for this
  story):
  - Sprint 11 active (already active per PROMPT 773).
  - Sprint 11 QA plan landed (already landed per PROMPT 774).
  - `/sprint-plan sprint-11 --add story-019` to pull this row into Sprint 11
    active scope (separate prompt; not authored here).
  - `/story-readiness` pass on this story file (separate prompt).
- **Unlocks** (depending on disposition):
  - If `bug-reproduced`: a follow-on repair story authored at
    `production/epics/hand-ui/story-020-<slug>.md`.
  - If `bug-fixed`: closes the open question from PROMPT 762 candidate #1
    (gameplay-blocking) and from the story 018 `cannot-reproduce` deferral.
  - If `cannot-reproduce` (second time) or `third-party-limitation`:
    escalation to Sprint 12 candidate authoring.

---

## Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Tighter capture still cannot reproduce within 1.5-day time-box | Medium | High | Time-box enforced. `cannot-reproduce` (second time) is a valid outcome and triggers escalation to Sprint 12 candidate authoring. |
| Frame-level video at 30 fps misses the grey-square 1-frame flash (panel refresh < 33 ms) | Low | Medium | Operator can use 60 fps capture if hardware supports it; otherwise note this gap in the new evidence file and escalate. |
| Synchronised wall-clock timestamping cross-platform setup is fragile (Windows / macOS / Linux shells differ) | Medium | Medium | The operator's host OS is recorded in the new evidence file's `command-summary.md`. If wall-clock prefixing failed, the operator can fall back to the Bevy tracing subscriber's `.with_timer(...)` for in-process timestamps. |
| `lightyear=debug` still produces enough noise to bury S5 attribution | Low | Medium | If `debug` is too noisy, lock the rows that can be filled and disposition `cannot-reproduce` with a third-iteration follow-on story scoped at `lightyear` per-channel selective debug logging. |
| Operator interprets symptom (grey square) as the originating S-row without checking which S-row first FAILs | Medium | High | The truth-table requires the **originating** FAIL row to be named. The frame-level video lets the reviewer trace the flash back to the originating log line. |
| Operator lands a repair commit inside this story | Medium | Medium | Acceptance criterion `HU-DRAG-RT-19-05` requires zero edits under `client/` / `server/` / `shared/` / `tests/`. Verified by `git diff --stat` on the worker branch. |
| Concurrent root-checkout race during retest evidence write damages the new evidence file | Low | Medium | Per 2026-05-13 orchestrator override, only one shared-status writer runs at a time. This story does not touch shared status files; the new evidence file is a single new write. |

---

## Verification (orchestrator-side, before worker dispatch)

These are sanity checks for the orchestrator emitting the retest prompt, not
for the worker:

- `production/sprint-status.yaml` `sprint:` field is **not** bumped by this
  story authoring.
- `production/stage.txt` reads `Polish` and is unchanged.
- `production/sprints/sprint-11.md` status block is unchanged.
- The PROMPT 761 Polish→Release gate-check FAIL evidence at
  `production/gate-checks/gate-polish-release-2026-05-12.md` is preserved.
- `git diff --check` and `git diff --cached --check` pass before commit.

---

## Authoring Trail

- 2026-05-13 — PROMPT 778 — Story file authored as the follow-on
  diagnostic-only retest mandated by story 018 `HU-DRAG-RT-04` after the
  `cannot-reproduce` disposition. Sprint 11 is active (PROMPT 773) and the
  QA plan is landed (PROMPT 774); this story 019 is **not yet activated**
  into the Sprint 11 active scope — activation is a separate prompt
  (`/sprint-plan sprint-11 --add story-019`). No code changes, no smoke /
  gate / QA / `/dev-story` / `/story-done` run.

- 2026-05-14 — PROMPT 807 — `/dev-story` worker authored tighter-capture
  drag-runtime retest evidence on worker branch
  `work/s11-drag-runtime-retest-tighter-capture`. Tightened capture bar
  (synchronised wall-clock + frame-level video + `lightyear=debug` +
  `server::game=debug`) attempted under a second 1.0-day operator
  time-box; CLI-dispatch second-time-box result: **still unable to
  reproduce** the Sprint 11 drag-runtime regression under the tighter
  bar. Evidence captured at
  `production/qa/evidence/sprint-11-drag-runtime-evidence-tighter.md`
  (NEW) and capture artefact directory
  `production/qa/evidence/captures/sprint-11-drag-runtime/2026-05-14-cli-dispatch-second-timebox/README.md`
  (NEW). Escalation mapping appended to evidence file in a follow-on
  worker commit, linking the diagnostic to the PROMPT 804 Sprint 13
  candidate runtime-hardening stories. No production code touched
  (diagnostic-only by AC design). ADR-002 + ADR-009 preserved.

- 2026-05-14 — PROMPT 810 — Integration: cherry-pick of the two
  canonical PROMPT 807 commits onto `main` produced `c2a08a6`
  (`qa(s11/s12): author tighter-capture drag-runtime retest evidence (PROMPT 807)`)
  and `a8ef42d`
  (`qa(s11/s12): map drag-runtime escalation to PROMPT 804 Sprint 13 candidates (PROMPT 807)`).
  Fast-forward push `383cacb..a8ef42d`. Final disposition recorded:
  **cannot-reproduce — second time-box exhaustion**. Workspace test
  suite at integration HEAD: 394 pass / 0 fail / 2 ignored across all
  client test binaries.

- 2026-05-14 — PROMPT 814 — `/story-done` paperwork: this Status field
  flipped Draft -> Done with terminal disposition
  `closed-with-conditions / evidence-captured-cannot-reproduce` after
  second time-box exhaustion (producer accepts the second-time-box
  exhaustion as the terminal disposition for this diagnostic-only
  story; the underlying runtime bug is **NOT claimed fixed** and is
  escalated to the PROMPT 804 Sprint 13 candidate runtime-hardening
  stories per the PROMPT 807 mapping). `production/sprint-status.yaml`
  Sprint 12 Must Have row `S11-DRAG-RUNTIME-RETEST-TIGHTER-CAPTURE-001`
  flipped `status: ready -> done` with
  `completed: 2026-05-14 (closed-with-conditions / cannot-reproduce
  after second time-box exhaustion)`. Sprint 12 is NOT closed-out by
  PROMPT 814. No `/smoke-check`, `/team-qa`, `/gate-check`,
  `/release-check`, no Sprint 12 close-out, no stage advance, no
  S8-QA-001-W1 closure, no release-readiness claim. PROMPT 683-era
  runtime divergence question, S8-QA-001-W1 OPEN, QA-COND-0005
  accepted-risk, QA-COND-0006 accepted-risk / deferred, PAW-TD-*-a
  placeholder-art accept-risk, PROMPT 761 Polish→Release FAIL — all
  preserved unchanged.
