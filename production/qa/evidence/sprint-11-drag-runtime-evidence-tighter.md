# Sprint 11 Drag Runtime Retest — Tighter-Capture Evidence + Disposition (S11-DRAG-RUNTIME-RETEST-TIGHTER-CAPTURE-001)

> **Story**: `production/epics/hand-ui/story-019-drag-runtime-retest-tighter-capture.md`
> **Story ID**: S11-DRAG-RUNTIME-RETEST-TIGHTER-CAPTURE-001
> **Sprint**: Sprint 12 (active; Polish-stage; activated by PROMPT 798; QA plan landed by PROMPT 799). Story 019 was authored under Sprint 11 (PROMPT 778) and carried forward into Sprint 12 scope by the PROMPT 799 QA plan (`production/qa/qa-plan-sprint-12.md`).
> **Authored**: 2026-05-14 (PROMPT 807)
> **Worker branch / commit**: `work/s11-drag-runtime-retest-tighter-capture` (this commit is the evidence-authoring commit; see §"Verification" for hash)
> **Base source-of-truth**: `origin/main@d8d0196` (PROMPT 801 `dev(s12): un-ignore lobby ConfirmClass intent-chain test`). PROMPT 807 lands paperwork-only commits on top under `production/qa/evidence/`.
> **Parent story**: `production/epics/hand-ui/story-018-drag-runtime-retest.md`
> **Parent evidence**: `production/qa/evidence/sprint-11-drag-runtime-evidence.md`
> **Disposition (lock)**: **`cannot-reproduce`** — **second time-box exhaustion**. See §"Disposition" below for justification and §"Follow-on artefact" for the Sprint 13 expanded-tracing escalation recommendation.

---

## Status / No-Claim Banner (restated verbatim per `HU-DRAG-RT-19-07`)

The story 019 no-claim banner (which itself restates story 018's banner verbatim) is
restated here word-for-word so that this evidence file is self-contained for future
readers and so that `HU-DRAG-RT-19-07` is verifiable by text search:

- Sprint 11 is `draft` per `production/sprints/sprint-11.md` and the `next_sprint:`
  block in `production/sprint-status.yaml`. This story is **not activated** and
  does **not** appear as an active row in `production/sprint-status.yaml`.
  Activation happens via `/sprint-plan sprint-11` in a separate prompt.

  > **Update on the activation clause only (not a no-claim relaxation)**: Sprint 11
  > was activated by PROMPT 773 at `07aafe2` (status flipped to `active`); the QA
  > plan landed at `d36bbbd` (PROMPT 774). The story-018 banner above was authored
  > pre-activation (PROMPT 766). Sprint 11 has since been `closed-with-conditions`
  > per PROMPT 792, and Sprint 12 is now active per PROMPT 798. The text is restated
  > verbatim to preserve no-claim banner integrity. None of the no-claim clauses
  > below are affected by activation or by the Sprint 11 closeout.

- This story authoring does **not** claim: public release readiness,
  release-candidate readiness, full game completion, broad / Standard-tier
  accessibility completion (`QA-COND-0005`), playtest / fun-hypothesis
  validation (`QA-COND-0006`), full playable-client manual QA, two-client
  GAME_OVER closure (`S8-QA-001-W1`), or final-art / asset-production
  completion.

- `production/stage.txt` remains `Polish`. No `/dev-story`, `/smoke-check`,
  `/team-qa`, `/gate-check`, or `/story-done` is run by the authoring of this
  story file.

**No optimistic client-side authority is introduced or proposed by this evidence
file or by any follow-on path referenced in §"Follow-on artefact".** ADR-002 and
ADR-009 lines are binding for any follow-on repair story that may be authored
after this run.

**This evidence file does NOT claim any of the above either.** It records:

1. The 5 S1-S5 tracing emit sites are still present in the worker source on the
   tighter-capture base (`origin/main@d8d0196`) at the same file:line locations
   recorded in the parent evidence (`d36bbbd` era). Static-code-only verification —
   no runtime trace was captured.
2. The drag-ended gate widening (PROMPT 697 / commit `cbb2565`) is present in the
   worker source.
3. The runtime divergence question opened by the PROMPT 683 diagnostic (8
   `C2SActivateCard` sends, zero `stage_or_update` events) **still cannot be
   closed by static-code analysis alone**; it requires the operator-driven
   two-client friend-game route with frame-level video, synchronised wall-clock
   timestamps, and `lightyear=debug` / `server::game=debug` log capture — none
   of which an automated CLI dispatch worker can exercise.

---

## Scope of this evidence pass

This evidence file is the deliverable for `HU-DRAG-RT-19-01` … `HU-DRAG-RT-19-08`
of story 019. Per the story's own §"QA Test Cases", the test type is Integration
but the evidence form is **manual runtime trace, not automated `cargo test`**.

This worker (PROMPT 807) is an automated dispatch in a non-interactive CLI session.
It **structurally cannot**:

- Launch two browser tabs and interact with them via real mouse clicks.
- Press, drag, and release a `bevy_picking` pointer across two real client
  surfaces observed by a human eye.
- Record a frame-level video (30 fps minimum, 60 fps preferred for the 1-frame
  grey-square flash) of the release moment for drag attempts A / B / C / D.
- Wire synchronised UTC wall-clock-millisecond timestamping across three
  log producers (server, client A, client B) running on three real shell
  sessions on real wall-clock hardware.

These constraints are identical to those that forced story 018 to disposition as
`cannot-reproduce` under PROMPT 778; the tighter capture protocol does not
remove them — it tightens what an operator-driven session must capture, but
the structural unavailability of an interactive session inside a CLI dispatch
is unchanged.

What this worker **can** do, and **has** done:

- Read the worker source at `work/s11-drag-runtime-retest-tighter-capture`
  branched from `origin/main@d8d0196` and confirm all five S1-S5 emit sites
  are present at the **same file:line locations** recorded in the parent
  evidence (`d36bbbd` era). Note: the parent evidence base was `d36bbbd`
  (PROMPT 774); current base is `d8d0196` (PROMPT 801). No drift in the
  drag-runtime code area between these two commits.
- Re-verify the drag-ended gate widening (PROMPT 697 / commit `cbb2565`) is
  on this branch.
- Author this evidence file as the **paperwork** half of story 019 with the
  `cannot-reproduce` (second-time) disposition, the truth-table locked as
  `NOT-OBSERVED` on every row (with code-evidence pointers, not runtime-evidence
  pointers), and the Sprint 13 expanded-tracing escalation recommendation per
  `HU-DRAG-RT-19-04` rule for second-time `cannot-reproduce`.

The evidence form below is deliberately conservative: every row of the S1-S5
truth-table is recorded as `NOT-OBSERVED` because **no runtime trace was captured
in this run, exactly as in story 018**. Static-code presence of an emit site
still does not prove that site fires under live friend-game pointer events.

---

## Static-code re-verification on tighter-capture base

Static verification of the 5 S1-S5 instrumentation sites + 3 sibling-line
`spawn_highlight_caller` callers as of
`work/s11-drag-runtime-retest-tighter-capture` (branched from
`origin/main@d8d0196`). Identical file:line locations to the parent evidence
file (which was authored on base `d36bbbd`) — **no drift between `d36bbbd` and
`d8d0196` in this code area**.

| S# | Target string | File | Line | Function | Level | Presence |
|----|---------------|------|------|----------|-------|----------|
| S1 | `drag_sprite_visible_flip` | `client/src/ui/hand/mod.rs` | 2020 | `handle_placement_drag_started_system` | `info!` | ✅ Present |
| S2 | `fan_active_default_drop` | `client/src/ui/hand/mod.rs` | 1901 | `handle_hand_fan_card_click_system` (Active-slot branch) | `info!` | ✅ Present |
| S3 | `placement_cursor_move` | `client/src/ui/hand/mod.rs` | 2049 | `handle_placement_cursor_moved_system` | `debug!` | ✅ Present |
| S4 | `drag_lift_tween_install` | `client/src/card_animations/input_gating.rs` | 163 | `hand_card_drag_start_system` | `info!` | ✅ Present |
| S5 | `spawn_highlight_state_change` | `client/src/presentation/board_rendering.rs` | 1709 | `set_spawn_highlight_state` | `info!` | ✅ Present |
| S5-callers | `spawn_highlight_caller` | `client/src/presentation/board_rendering.rs` | 1640, 1685, 2622 | `apply_snapshot_spawn_highlights_clear`, `apply_player_spawn_highlight`, `spawn_cell_node_default` | `info!` | ✅ Present (3 of 3) |

Verification command (executed via `Grep`):

```text
rg -n "drag_sprite_visible_flip|fan_active_default_drop|placement_cursor_move|drag_lift_tween_install|spawn_highlight_state_change|spawn_highlight_caller" client/src/
```

All 9 hits are at the exact lines recorded above; no code drift since the parent
evidence file.

---

## S1-S5 Truth Table (locked as `NOT-OBSERVED`, second time)

Per story 019 §"Time-box": if S1-S5 cannot be filled with at least one PASS/FAIL
row per column within 1.5 days, lock the truth-table as best-effort with
`NOT-OBSERVED` rows explicitly named and disposition `HU-DRAG-RT-19-03` as
`cannot-reproduce` (second time). This run records every row as `NOT-OBSERVED`
because no runtime trace was captured.

The "evidence pointer" column points to the **code-evidence** for the emit
site's existence (static), not the runtime trace for the emit site's firing
(which is the gap this second-time disposition records).

| Stage | Description | Drag A (BoardCell) | Drag B (Instant fan-plate) | Drag C (cancel empty) | Drag D (invalid cell) | Code-evidence pointer |
|-------|-------------|--------------------|-----------------------------|----------------------|------------------------|----------------------|
| **S1** | Pointer Press observer on `FanSlotIndex` (Primary, Staging) | `NOT-OBSERVED` | `NOT-OBSERVED` | `NOT-OBSERVED` | `NOT-OBSERVED` | `client/src/ui/hand/mod.rs:2020` (`target: "drag_sprite_visible_flip"`) |
| **S2** | `HandUiPlacementDragStarted { card, owner_id }` emit | `NOT-OBSERVED` | `NOT-OBSERVED` | `NOT-OBSERVED` | `NOT-OBSERVED` | `client/src/ui/hand/mod.rs:1901` (`target: "fan_active_default_drop"`) |
| **S3** | Drag-start consumer flips `HandDragSprite::Visibility` to Visible + board ghost engages | `NOT-OBSERVED` | `NOT-OBSERVED` (no board ghost expected; `NOT-OBSERVED` is the correct lock per story 019 row B) | `NOT-OBSERVED` (cancel — ghost cleared expected) | `NOT-OBSERVED` (invalid target — ghost cleared expected) | `client/src/ui/hand/mod.rs:2049` + `client/src/presentation/board_rendering.rs:1709` |
| **S4** | `Pointer<Move>` → `HandUiPlacementCursorMoved` + drag-sprite `Node.left` / `Node.top` track cursor; input gating confirms Staging | `NOT-OBSERVED` | `NOT-OBSERVED` | `NOT-OBSERVED` | `NOT-OBSERVED` | `client/src/card_animations/input_gating.rs:163` |
| **S5** | **PRIMARY SUSPECT** — `Pointer<Release>` (Primary) → `HandUiPlacementDragEnded` → correct `PlacementTargetKind` branch → `active_drag.clear()` → server-authoritative `C2SActivateCard` (**no client-side optimism**) | `NOT-OBSERVED` | `NOT-OBSERVED` | `NOT-OBSERVED` (no `C2SActivateCard` send expected — `NOT-OBSERVED` is correct per story 019 row C) | `NOT-OBSERVED` (snap-back; no `C2SActivateCard` send expected — `NOT-OBSERVED` is correct per story 019 row D) | `client/src/presentation/board_rendering.rs:1640,1685,2622` + `client/src/ui/hand/mod.rs:2065` (`handle_placement_drag_ended_system`) |

**Grey-square attribution lock**: per story 019, the grey-square symptom (the
visual 1-frame flash described by PROMPT 698) originates at the **first row in
A/B/C/D that transitions to FAIL**. Because every row is `NOT-OBSERVED`, the
attribution remains **deferred** — and per `HU-DRAG-RT-19-04` second-time
`cannot-reproduce` rule, the deferral now escalates to Sprint 13 expanded-tracing
authoring (see §"Follow-on artefact").

**Special call-out (the 683-era runtime evidence, preserved unchanged)**:
PROMPT 683 reported 8 `C2SActivateCard` sends with zero `stage_or_update`
events, implying the failing edge is **S5 (release branch) → server**. This
second-time evidence file **does not confirm or refute** that claim either —
confirming it still requires the operator-driven two-client friend-game route
with the tighter capture protocol (frame-level video + synchronised wall-clock
timestamps + `lightyear=debug` + `server::game=debug`). The Sprint 13
expanded-tracing follow-on (see below) preserves this specific question as
the primary capture target and, per `HU-DRAG-RT-19-04`, must broaden the
tracing scope (e.g., per-channel selective `lightyear` debug) rather than
re-run the same-scope capture a third time.

---

## Disposition (`HU-DRAG-RT-19-03`)

**Disposition**: `cannot-reproduce` (**second time** since PROMPT 778 / story 018)
— strictly per story 019 §"Time-box" ("If S1-S5 cannot be filled with at least
one PASS/FAIL row per column within 1.5 days: lock the truth-table as best-effort
with `NOT-OBSERVED` rows explicitly named. Disposition `HU-DRAG-RT-19-03` as
`cannot-reproduce` (second time). Escalate the diagnostic gap to Sprint 12 [now
Sprint 13 — see §"Follow-on artefact"] candidate authoring; do not author a third
same-scope retest without expanded tracing scope.").

**Justification (second time)**:

- The 1.5-day retest time-box presumes an operator-driven session with two
  browser tabs, manual mouse drag/release inputs, **frame-level video capture**,
  **synchronised UTC-millisecond log prefixing**, and `RUST_LOG=…=trace`
  log capture from server + both clients at the upgraded `lightyear=debug` /
  `server::game=debug` levels. PROMPT 807 is an automated CLI dispatch that
  cannot manipulate browsers, capture user-visible drag/release frames, or
  run three synchronised real-time shell sessions with millisecond UTC
  prefixing. The time-box was therefore not exercised in operator mode at all —
  it was structurally unavailable to this worker, **identically to PROMPT 778
  for story 018**.
- The tighter capture protocol (`lightyear=debug` / `server::game=debug` /
  frame-level video / synchronised wall-clock prefix) does not remove this
  structural constraint — it tightens what the operator must capture once a
  real operator-driven session is available.
- Static-code re-verification (above) confirms the 5 instrumentation sites and
  the drag-ended gate widening remain present in the worker source on the
  current base `d8d0196`. This is necessary but not sufficient to disposition
  the runtime divergence as `bug-fixed`: a `bug-fixed` disposition would
  require positive S1-S5 PASS rows from a real tighter-capture trace, and none
  exist in this run.
- A `bug-reproduced` disposition would require a captured trace showing the
  failing edge. None exists.
- A `third-party / platform limitation` disposition would require evidence
  of browser- or OS- or input-device-specific behaviour. None exists.

`cannot-reproduce` (second time) is therefore the **only honest disposition**
available to this run, and per story 019 §"Time-box" it is explicitly the
prescribed disposition when the 1.5-day time-box cannot be exercised — with
the additional explicit constraint that **no third same-scope retest is
authored**. The follow-on path is expanded-tracing scope (see below).

**Offending stage named in evidence file** (per `HU-DRAG-RT-19-03`): not named —
no row transitioned to FAIL in this run because no row was OBSERVED. The
Sprint 13 expanded-tracing follow-on preserves S5 as the **primary suspect**
per the PROMPT 683-era evidence summary, without claiming S5 is the actual
failing edge.

---

## Follow-on artefact (`HU-DRAG-RT-19-04`) — Sprint 13 expanded-tracing escalation

Because the disposition is `cannot-reproduce` **for the second time**, the
follow-on per story 019 `HU-DRAG-RT-19-04` is **NOT a third same-scope retest**.
Story 019 §"Time-box" and `HU-DRAG-RT-19-04` `cannot-reproduce` rule both
require: "no further `cannot-reproduce` retest is authored at the same scope
without expanded tracing." This evidence file therefore **recommends authoring
a Sprint 13 expanded-tracing story** at a new file path; this evidence file
does NOT author the story itself (that is a separate orchestrator prompt
operating on Sprint 13 scope).

**Recommended Sprint 13 follow-on story path**:
`production/epics/hand-ui/story-NNN-drag-runtime-expanded-tracing.md`
(where `NNN` is the next available story number in the hand-ui epic at the time
of Sprint 13 authoring; the producer / orchestrator picks the number).

**Recommended expanded-tracing scope (advisory; final scope is the
follow-on story's author's call)**:

1. **Per-channel selective `lightyear` debug logging.** Instead of
   `lightyear=debug` (story 019's setting), enable per-channel debug logging
   only on the channels relevant to the `C2SActivateCard` → `stage_or_update`
   path (e.g., the reliable ordered channel carrying `C2SActivateCard`). This
   keeps the protocol-shape signal high while reducing replication-channel
   noise that may bury S5 attribution.
2. **Persistent in-process millisecond-UTC tracing init.** Either (a) land a
   small Bevy `tracing_subscriber::fmt().with_timer(...)` change scoped to a
   feature flag (e.g., `tracing-utc-millis`) gated off by default, or (b)
   land a small documented shell-wrapper script in `tools/` that prepends
   UTC-millisecond timestamps to each log producer's output. Whichever path
   the follow-on takes, it must remain a separable patch and not entangle
   with gameplay code.
3. **Operator workflow embed.** Land a runbook section in
   `production/qa/evidence/manual-friend-game-evidence-runbook.md` (or a new
   sibling runbook) that documents the exact two-client friend-game route
   for tighter capture: server invocation, client-A invocation, client-B
   invocation, the upgraded `RUST_LOG` string, frame-level video capture
   tool (OBS / ShareX / native OS screen recorder) with target FPS (60 fps
   preferred for the 1-frame grey-square flash), and the synchronised
   timestamp protocol.
4. **No protocol-shape modification.** The follow-on remains diagnostic-only
   and must NOT modify `S2C*` / `C2S*` shapes. Any protocol divergence
   discovered must be authored as a further separate follow-on story.
5. **No optimistic client-side authority.** Verbatim restatement of the
   ADR-002 / ADR-009 prohibition is required in the follow-on story's
   no-claim banner. (This evidence file already restates it; see §"Status /
   No-Claim Banner".)
6. **No `S8-QA-001-W1` closure claims.** The follow-on must explicitly carry
   forward the open `S8-QA-001-W1` two-client GAME_OVER condition.

**What the Sprint 13 follow-on does NOT do**:

- It does NOT land any repair commit under `client/` / `server/` / `shared/`
  / `tests/`. Any repair commit (if needed after the expanded-tracing
  capture dispositions `bug-reproduced`) is delegated to a further follow-on
  repair story.
- It does NOT close `S8-QA-001-W1`.
- It does NOT close `QA-COND-0005` or `QA-COND-0006`.
- It does NOT retry the Polish → Release gate-check.
- It does NOT advance `production/stage.txt` from `Polish`.

**Open question preserved unchanged for the Sprint 13 follow-on**:

> Does the PLACEMENT drag-to-stage flow on a real two-client friend-game
> emit `HandUiPlacementDragEnded` and route to the correct
> `PlacementTargetKind` branch (matching the static-code path on
> `origin/main@d8d0196`), or does the S5 release-branch emit `C2SActivateCard`
> without the server's `stage_or_update` advancing (the PROMPT 683-era
> 8-sends / 0-events hypothesis)?

This question is **unanswered** after two CLI-dispatch retest attempts
(PROMPT 778 / PROMPT 807). Both were structurally unable to exercise the
operator-driven route. The Sprint 13 expanded-tracing follow-on must
be authored in a context where a real operator session is available, or
the diagnostic gap must be narrowed by other means (e.g., a deterministic
integration test that exercises the `Pointer<Release>` observer with a
synthesised `bevy_picking` event sequence on a headless `App`, if such a
test is feasible without violating ADR-002 / ADR-009).

---

## Acceptance criteria check (`HU-DRAG-RT-19-01` … `HU-DRAG-RT-19-08`)

| AC | Status | Justification |
|----|--------|----------------|
| **HU-DRAG-RT-19-01** — Tighter-capture runtime trace captured | ❌ deferred (disposition `cannot-reproduce`, second time) | No two-client friend-game session was executed by this automated dispatch (identical structural unavailability to PROMPT 778 / story 018). Per story 019 §"Time-box", `cannot-reproduce` (second time) is the prescribed disposition when the 1.5-day time-box cannot be exercised, and the truth-table is locked as best-effort `NOT-OBSERVED`. Follow-on is Sprint 13 expanded-tracing per §"Follow-on artefact". |
| **HU-DRAG-RT-19-02** — S1-S5 truth-table locked with at least one observed row | ❌ deferred → `cannot-reproduce` (second time) | The acceptance criterion's "at least one PASS or FAIL row per column" target was not met — every row is `NOT-OBSERVED`. Per story 019 §"Time-box", this is the explicit second-time disposition trigger. The truth-table is locked above with `NOT-OBSERVED` on every row and code-evidence pointers for each emit site. |
| **HU-DRAG-RT-19-03** — Test-vs-runtime divergence dispositioned | ✅ `cannot-reproduce` (second time) | §"Disposition" above records the disposition and justifies it. The PROMPT 683-era discrepancy is preserved as the primary suspect (S5 release-branch → server edge) for the Sprint 13 expanded-tracing follow-on without being claimed as confirmed. |
| **HU-DRAG-RT-19-04** — Repair or follow-on authored | ✅ recommendation authored (no follow-on story file in this commit) | §"Follow-on artefact" above records the Sprint 13 expanded-tracing escalation recommendation per the `cannot-reproduce` (second time) rule. **The follow-on story file itself is NOT authored by this commit** — story 019 explicitly requires expanded-tracing scope (not same-scope), which is a separate orchestrator prompt against Sprint 13's planning context. **No repair commit lands in this story.** |
| **HU-DRAG-RT-19-05** — No production code changes | ✅ verified | This commit changes only paperwork: `production/qa/evidence/sprint-11-drag-runtime-evidence-tighter.md` (this file) and `production/qa/evidence/captures/sprint-11-drag-runtime/2026-05-14-cli-dispatch-second-timebox/README.md` (capture-directory placeholder for the second-time `cannot-reproduce`). No edits land under `client/` / `server/` / `shared/` / `tests/`. Verified by `git diff --stat origin/main..HEAD -- client/ server/ shared/ tests/` in §"Verification". |
| **HU-DRAG-RT-19-06** — No optimistic client-side authority introduced | ✅ verified | This evidence file states the prohibition (§"Status / No-Claim Banner" above) and the Sprint 13 expanded-tracing recommendation (§"Follow-on artefact") explicitly requires restating it. ADR-002 + ADR-009 lines preserved. Text search confirms the phrase "no optimistic client-side authority" appears in this file. |
| **HU-DRAG-RT-19-07** — Non-claims preserved | ✅ verified | §"Status / No-Claim Banner" above restates the story 019 banner verbatim, including public release, full manual QA, Standard-tier accessibility (`QA-COND-0005`), playtest / fun-hypothesis (`QA-COND-0006`), full game completion, `S8-QA-001-W1` two-client GAME_OVER closure. **None are claimed closed by this retest.** |
| **HU-DRAG-RT-19-08** — Sprint 11 active status preserved (adapted for Sprint 12 active) | ✅ adapted (see note) | Per the story 019 banner clause, no edits land under `production/sprint-status.yaml`, `production/stage.txt`, `production/sprints/sprint-11.md`, or `production/sprints/sprint-12.md` in this commit. Verified by `git diff --stat origin/main..HEAD -- production/sprint-status.yaml production/stage.txt production/sprints/sprint-11.md production/sprints/sprint-12.md` in §"Verification". Sprint 11 has been `closed-with-conditions` per PROMPT 792 (separate prompt) and Sprint 12 is now `active` per PROMPT 798 (separate prompt); neither sprint state is touched by this commit. |

---

## Verification

Commands run in the worker (`work/s11-drag-runtime-retest-tighter-capture`,
branched from `origin/main@d8d0196`):

| Command | Purpose | Result |
|---|---|---|
| `git fetch origin` | Sync remote refs | Clean (HEAD == `origin/main@d8d0196`) |
| `git worktree add D:/_DEV/claude-code-game-studios-worktrees/s11-drag-runtime-retest-tighter-capture -b work/s11-drag-runtime-retest-tighter-capture origin/main` | Create isolated worktree | Worktree created at branch `work/s11-drag-runtime-retest-tighter-capture` tracking `origin/main` |
| `rg -n "drag_sprite_visible_flip\|fan_active_default_drop\|placement_cursor_move\|drag_lift_tween_install\|spawn_highlight_state_change\|spawn_highlight_caller" client/src/` (executed via `Grep` tool) | Confirm 5 S1-S5 emit-site target strings + 3 sibling callers exist in worker source | All 9 hits present at the lines recorded in §"Static-code re-verification" |
| `cargo test -p client --no-fail-fast` | Required verification per PROMPT 807 dispatch | See §"`cargo test -p client` result" below for full result |
| `git diff --check origin/main...HEAD` | Verify no whitespace / conflict-marker damage | CLEAN |
| `git diff --stat origin/main..HEAD -- client/ server/ shared/ tests/` | Verify `HU-DRAG-RT-19-05` (no production code changes) | EMPTY (paperwork-only commit) |
| `git diff --stat origin/main..HEAD -- production/sprint-status.yaml production/stage.txt production/sprints/sprint-11.md production/sprints/sprint-12.md` | Verify `HU-DRAG-RT-19-08` (sprint state untouched) | EMPTY |

### `cargo test -p client` result

Executed during PROMPT 807 verification phase. Recorded outcome (see
worker terminal log for raw output): **see §"Verification — actual results"
appendix at bottom of file** (appended after the test run completes; this
section reserved at authoring time).

**`cargo` policy**: per the orchestrator dispatch ("Verification: `cargo test
-p client --no-fail-fast`"), `cargo test` is invoked **once** by this worker
as a verification-only check. The expectation is that pre-existing tests on
`work/s11-drag-runtime-retest-tighter-capture` (branched from `origin/main@d8d0196`)
pass identically to `main` because this commit changes zero code. Any non-pass
outcome surfaces as an out-of-scope finding (see §"Out-of-scope findings" below)
and is reported back, but does not flip the disposition — `cannot-reproduce`
remains the recorded outcome for this story's runtime-divergence question.

**Commit hash for this evidence-authoring commit**: recorded in
`reports/PROMPT-807-S11-DRAG-RUNTIME-RETEST-TIGHTER-CAPTURE.md` (final commit
hash after rebase on `origin/main`).

**Push**: worker branch `work/s11-drag-runtime-retest-tighter-capture` is pushed;
`main` is never pushed by this worker per the 2026-05-13 orchestrator override.

---

## Out-of-scope findings encountered during this run

Per story 019 §"Risks", out-of-scope findings are noted here but **not** repaired:

- None encountered at authoring time. Any `cargo test -p client --no-fail-fast`
  finding will be appended below in §"Verification — actual results" if the
  test run surfaces anything unexpected; such findings are NOT in scope for
  this story and are NOT repaired here. They would be authored as separate
  follow-on stories.

---

## Verification — actual results (appended after worker execution)

### `cargo test -p client --no-fail-fast` — 2026-05-14, PROMPT 807

**Exit code**: 0 (PASS)
**Aggregate**: **59 passed / 0 failed / 1 ignored** across 9 client test binaries + 0 doc-tests.

Per-binary breakdown:

| Binary | Passed | Failed | Ignored |
|--------|--------|--------|---------|
| (client lib unit tests, binary 1) | 2 | 0 | 0 |
| (client lib unit tests, binary 2) | 6 | 0 | 0 |
| (client lib unit tests, binary 3) | 7 | 0 | 0 |
| `shop_auction_ui_card_acquired_test` | 4 | 0 | 0 |
| `shop_auction_ui_draft_initial_grid_test` | 10 | 0 | 0 |
| `shop_auction_ui_draft_initial_objective_overlay_test` | 8 | 0 | 0 |
| `shop_auction_ui_plugin_scaffold_formulas_test` | 7 | 0 | 1 |
| `shop_auction_ui_reconnect_late_message_test` | 6 | 0 | 0 |
| `shop_auction_ui_shop_panel_test` | 9 | 0 | 0 |
| Doc-tests `client` | 0 | 0 | 0 |
| **Total** | **59** | **0** | **1** |

The 1 ignored test is `shop_auction_ui_prepooled_panel_roots_are_bevy_ui_nodes`
with the well-known **PROMPT 750 D-5** ignore reason ("ShopAuctionUiEntity
count drift — actual=66, formula expects=57 (9 entity delta); needs scaffold
owner to either update formula or trim spawn"). This ignore is **carried
forward unchanged** from `origin/main@d8d0196` and is **NOT** introduced by
this commit. Per Sprint 11 closeout (PROMPT 792) and the Sprint 12 QA plan
(PROMPT 799), the 5 retained D-5 `#[ignore]` tests are handled by Sprint 12
stories 012 / 013 / 014 / 015; this scaffold-formula D-5 is the cluster
addressed by Sprint 12 story 015 (umbrella vs split path; out of scope for
PROMPT 807).

**Out-of-scope findings**: none. No new failures, no new ignores, no panic,
no compilation warning beyond baseline.

### `git diff --check origin/main...HEAD` — 2026-05-14, PROMPT 807

**Result**: CLEAN (no whitespace damage, no conflict markers).

### `git diff --stat origin/main..HEAD -- client/ server/ shared/ tests/`

**Result**: EMPTY — `HU-DRAG-RT-19-05` verified (zero production code changes).

### `git diff --stat origin/main..HEAD -- production/sprint-status.yaml production/stage.txt production/sprints/sprint-11.md production/sprints/sprint-12.md`

**Result**: EMPTY — `HU-DRAG-RT-19-08` verified (sprint state and stage untouched).

---

## Authoring trail

- 2026-05-14 — PROMPT 807 — Evidence file authored as the paperwork half of
  story 019. Disposition: `cannot-reproduce` (**second time**; first time was
  story 018 / PROMPT 778). The 1.5-day time-box could not be exercised in the
  automated CLI dispatch — identical structural constraint to PROMPT 778.
  Truth-table locked as `NOT-OBSERVED` on every row with code-evidence
  pointers. Per story 019 §"Time-box" and `HU-DRAG-RT-19-04` rule for
  second-time `cannot-reproduce`: no third same-scope retest is authored;
  the follow-on path is **Sprint 13 expanded-tracing scope** recommended at
  §"Follow-on artefact". No production code changes. No release / RC /
  accessibility / playtest / full-game / GAME_OVER claims. Sprint 11
  closed-with-conditions state, Sprint 12 active state, QA plan, and
  `production/stage.txt` `Polish` value untouched.
