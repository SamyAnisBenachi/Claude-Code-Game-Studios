# PROMPT 1289 — Sprint 17 Closeout Evidence Reconcile

## Status line

1289: SPRINT-17-CLOSEOUT-EVIDENCE-RECONCILE: APPROVED-WITH-CONDITIONS

## Verdict

APPROVED-WITH-CONDITIONS. This reconcile corrects the **smoke-evidence basis**
of the PROMPT 1279 Sprint 17 closeout without reopening Sprint 17, without
flipping any row status, without modifying the existing `sprint_17_closeout`
block in `production/sprint-status.yaml`, and without changing Sprint 17 stage
or disposition. The single documentation/evidence condition recorded by
PROMPT 1279 — the missing durable PROMPT 1277 smoke artifact — is **discharged**
by the PROMPT 1284 durable post-fmt smoke + PROMPT 1288 Team-QA refresh chain.
All other carried conditions remain open. All non-claims are preserved verbatim.

This reconcile is **not** Sprint 17 reopening, **not** Sprint 18 activation,
**not** a release-readiness claim, **not** a release-candidate claim, **not**
a Polish → Release gate retry, and **not** a stage advance.

## Why this reconcile exists

| # | Event | Outcome |
|---|---|---|
| 1 | PROMPT 1278 Team-QA at `origin/main@946ca39` | `APPROVED-WITH-CONDITIONS`; accepted prompt-provided PROMPT 1277 disposition because no durable tracked smoke report existed; recorded the missing PROMPT 1277 artifact as a documentation/evidence condition. |
| 2 | PROMPT 1279 closeout at `origin/main@946ca39` | Sprint 17 closed `closed-with-conditions`; carried the PROMPT 1278 "missing durable smoke artifact" gap as `sprint_17_closeout.smoke_evidence.missing_artifact_condition`. |
| 3 | PROMPT 1277 durable smoke report (post-hoc) | FAIL on `cargo fmt --check`; other smoke segments PASS / PASS-WITH-WARNINGS. |
| 4 | PROMPT 1281 | Repaired the rustfmt drift surfaced by (3). |
| 5 | PROMPT 1282 | Verified the fmt repair locally on top of the PROMPT 1279 closeout. |
| 6 | PROMPT 1283 | Landed the PROMPT 1281 fmt repair on `origin/main` as `d73e25e49519a214f8fb0fefa1e78351ccd74795`. |
| 7 | PROMPT 1284 | Reran Sprint 17 smoke from `origin/main@d73e25e` in a clean worktree under the Windows/MSVC Cargo resource policy. Result: PASS across the board (1861/1861 tests, 0 failed, 0 ignored). |
| 8 | PROMPT 1288 | Team-QA refresh against `origin/main@d73e25e`; verdict `APPROVED-WITH-CONDITIONS`; refresh artifact `production/qa/team-qa-sprint-17-2026-05-18-post-fmt-refresh.md`; PROMPT 1278 documentation/evidence condition discharged by PROMPT 1284 durable post-fmt smoke. Worker branch tip: `origin/work/s17-post-fmt-team-qa-refresh-1288@ec6e5da4fd6a69ed2e99caa991b825e3ad02ac8c`. |

PROMPT 1279's smoke-evidence basis was therefore **stale**: it cited PROMPT 1264
FAIL + PROMPT 1272/1274/1275/1276 repair-chain landed on main + prompt-provided
PROMPT 1277 disposition without a durable tracked artifact, and predated the
PROMPT 1281 fmt repair that PROMPT 1277 surfaced. PROMPT 1284 is now the
durable post-fmt smoke of record and PROMPT 1288 is the corresponding refreshed
Team-QA of record.

## Source of truth

| Field | Value |
|---|---|
| Reconcile date | 2026-05-18 |
| Reconcile prompt | 1289 |
| Reconcile basis branch | `origin/work/s17-post-fmt-team-qa-refresh-1288@ec6e5da4fd6a69ed2e99caa991b825e3ad02ac8c` |
| Reconcile basis ancestor on `origin/main` | `d73e25e49519a214f8fb0fefa1e78351ccd74795` (PROMPT-1281: repair cargo fmt drift) |
| Reconcile worktree | `D:/Tmp/gcs-prompt-1289-closeout-reconcile` |
| Reconcile branch | `paperwork/sprint-17-closeout-evidence-reconcile-1289` (new; based on `ec6e5da`) |
| Sprint 17 disposition | `closed-with-conditions` (UNCHANGED — preserved verbatim from PROMPT 1279) |
| Top-level `production/sprint-status.yaml#status` | `closed-with-conditions` (UNCHANGED) |
| Stage | `Polish` (UNCHANGED) |
| `production/stage.txt` | UNTOUCHED |
| Cargo by PROMPT 1289 | **None.** PROMPT 1284 is the smoke evidence of record; this reconcile is paperwork-only. |

## What this reconcile discharges

The single condition discharged by this reconcile is the smoke-evidence
documentation gap. In `production/sprint-status.yaml`, the existing
`sprint_17_closeout[date=2026-05-18, prompt=1279].smoke_evidence` block records:

> `missing_artifact_condition: "No durable tracked reports/PROMPT-1277* artifact and no tracked production/qa/smoke-sprint-17* report were found on origin/main. Team-QA accepted the prompt-provided PASS/PASS-WITH-WARNINGS disposition with local rerun artifacts, but the missing durable smoke report remains a documentation/evidence condition."`

That documentation/evidence condition is **discharged** by:

- `reports/PROMPT-1284-sprint-17-post-fmt-smoke-rerun.md` (local durable post-fmt
  smoke against `origin/main@d73e25e`): `cargo fmt --check` PASS, targeted client
  + dev-launcher tests PASS, `cargo check --workspace --all-targets` PASS,
  `cargo test --workspace --tests --no-fail-fast` PASS at 263 suites · 1861
  tests · 0 failed · 0 ignored, active-`#[ignore]` scan PASS at 0 matches.
- `production/qa/team-qa-sprint-17-2026-05-18-post-fmt-refresh.md` (PROMPT 1288
  Team-QA refresh) verdict `APPROVED-WITH-CONDITIONS` integrating PROMPT 1284
  as the durable post-fmt smoke of record.

The PROMPT 1278 Team-QA review of record at
`production/qa/team-qa-sprint-17-2026-05-18.md` is **unmodified**. The
PROMPT 1279 closeout block at `production/sprint-status.yaml#sprint_17_closeout`
is **unmodified**. The PROMPT 1279 closeout banner at
`production/sprints/sprint-17.md` is **unmodified**.

## What this reconcile does NOT discharge

All conditions other than the smoke-evidence documentation gap remain open and
carried forward exactly as recorded by PROMPT 1278, PROMPT 1279, and PROMPT 1288:

1. **Sprint 17 disposition** remains `closed-with-conditions` per PROMPT 1279.
   This reconcile does not reopen Sprint 17, does not flip the top-level
   `status` field, and does not edit the `sprint_17_closeout` block.
2. **`S11-HUD-TIMER-EYEBALL-VISUAL-001`** remains `ready` and
   **human-operator-blocked**; no LLM `/dev-story` or `/story-done` is
   authorised. HUD timer human screenshot evidence across the `DraftInitial`
   45 s, `DraftShop` 30 s, and `Placement` 10–12 s timer phases remains
   required and unclosed.
3. **`S17-UI-HUD-OPP-MANA-CLEANUP-001`** remains `in_progress`. AC3 source
   repair is on `origin/main@c842668`, but no final `/story-done` paperwork
   closed the Sprint 17 parent row; PROMPT 1279 explicitly did not mark it
   `done`; this reconcile preserves that explicit non-closure.
4. **PROMPT 761 Polish → Release gate-check FAIL** preserved verbatim at
   `production/gate-checks/gate-polish-release-2026-05-12.md`. No retry
   attempted; no retry in scope for Sprint 17.
5. **Stage** remains `Polish`. No Sprint 18 activation, no release-readiness
   claim, no release-candidate claim, no full-game claim.
6. **`S8-QA-001-W1`** remains OPEN (manual / browser two-client GAME_OVER gap).
7. **`QA-COND-0005`** remains accepted-risk / friend-game scope only.
8. **`QA-COND-0006`** remains accepted-risk / deferred.
9. **`PAW-TD-*-a`** placeholder-art accepted-risk across PAW-002..PAW-006
   preserved verbatim.
10. **`TQ-S12-C1..C7`** preserved verbatim; `TQ-S12-C7` AppCompat informational
    condition not closed.
11. **PROMPT 683-era runtime divergence** preserved (folded into Sprint 12
    story 019 `closed-with-conditions / cannot-reproduce`); underlying
    drag-runtime bug not claimed fixed.
12. **PROMPT 1054 P1 UI snapshot visual retest** remains
    `BLOCKED-HUMAN-OPERATOR`.
13. **All prior Sprint 10 / 11 / 12 / 13 / 14 / 15 / 16 dispositions**
    (`closed-with-conditions` per PROMPT 763 / 792 / 817 / 894 / 987 /
    1056 / 1082+1088) preserved unchanged.
14. **24 PROMPT 1022 QA-snapshot audit findings** preserved as report-only
    inputs to future story authoring; none claimed closed.
15. **Long-tail PROMPT 1076 findings** (AUDIT-1076-05 / 08 / 11) remain
    Sprint 18+ candidates.
16. **Long-tail PROMPT 1077 findings** (SOURCE-1077-05 / 07 / 11 / 12 / 13
    / 14 / 15) deferred to Sprint 18+.
17. **PROMPT 1278 / PROMPT 1279 smoke-warning carries** preserved as historical
    Sprint 18+ runtime-behaviour candidates: the local PROMPT 1277-era reruns
    logged `hand_ui_phase_transition_auto_submit_short_circuit` /
    `invalid_submit_state` at one round-1 `Placement → Resolution` transition,
    and `RSM disconnect timer breach: grace window exceeded` after a later
    client disconnect in `DraftShop`. These did not recur in the PROMPT 1284
    automated smoke (which exercises tests + `cargo check`, not a multi-client
    browser session); they remain Sprint 18+ candidates.
18. **Sprint 17 carried/partial rows** (`S11-HUD-TIMER-EYEBALL-VISUAL-001`
    ready/human-operator-blocked carry; `S17-UI-HUD-OPP-MANA-CLEANUP-001`
    in_progress carry) remain in their original carry posture.

## Non-claims (explicit; preserved verbatim)

PROMPT 1289 does NOT claim:

- Sprint 17 reopening.
- Sprint 17 status flip (no change to `closed-with-conditions`).
- Sprint 17 row-status flip on any row.
- Sprint 18 activation.
- release readiness or release-candidate readiness.
- full game completion.
- broad / Standard-tier accessibility completion.
- playtest / fun-hypothesis validation.
- full playable-client manual QA.
- two-client GAME_OVER closure.
- final-art completion.
- Polish → Release gate retry.
- stage advance from `Polish`.
- closure of `S11-HUD-TIMER-EYEBALL-VISUAL-001` or any HUD timer human
  evidence acceptance.
- silent `done` closure of `S17-UI-HUD-OPP-MANA-CLEANUP-001`.
- closure of `S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a`,
  `TQ-S12-C1..C7` (any of them).
- closure of any remaining PROMPT 1022, PROMPT 1076, or PROMPT 1077 finding
  outside concrete repairs already on `origin/main`.

## Files changed by PROMPT 1289

- `production/qa/closeout-evidence-reconcile-sprint-17-2026-05-18-prompt-1289.md`
  (this file; new) — reconcile narrative of record.
- `production/sprints/sprint-17.md` — prepended a small `PROMPT 1289 reconcile`
  banner ABOVE the existing `PROMPT 1279 closeout` banner. The PROMPT 1279
  closeout banner and the PROMPT 1099 activation banner are preserved verbatim
  below.
- `production/sprint-status.yaml` — appended a new
  `sprint_17_closeout_evidence_reconcile` block referencing this artifact and
  noting that PROMPT 1284 + PROMPT 1288 discharge the
  `missing_artifact_condition` from the existing `sprint_17_closeout` block.
  The existing `sprint_17_closeout` block is **not edited**. The top-level
  `sprint`, `stage`, and `status` fields are **not edited**.
- `reports/PROMPT-1289-sprint-17-closeout-evidence-reconcile.md` (overwritten
  by this relaunch; previous BLOCKED report content replaced by the
  APPROVED-WITH-CONDITIONS final report).
- `reports/PROMPT-1289-sprint-17-closeout-evidence-reconcile.summary.txt`
  (overwritten with the new DONE summary line).

**Forbidden writes confirmed absent:** `production/session-state/**`,
`production/stage.txt`, `production/gate-checks/**`, `production/qa/team-qa-sprint-17-2026-05-18.md`
(PROMPT 1278 review of record), `production/qa/team-qa-sprint-17-2026-05-18-post-fmt-refresh.md`
(PROMPT 1288 refresh of record), any source code, tests, Cargo files, CI files,
assets, Sprint 18 activation file, or release artifact — none modified.

## Worker branch / commit / push

- Branch: `paperwork/sprint-17-closeout-evidence-reconcile-1289` (new).
- Base: `ec6e5da4fd6a69ed2e99caa991b825e3ad02ac8c` (PROMPT 1288 worker tip,
  which is a strict descendant of `origin/main@d73e25e`).
- Commit subject pattern: `closeout-evidence-reconcile(s17): discharge
  PROMPT 1279 smoke-evidence documentation gap via PROMPT 1284 + PROMPT 1288
  (PROMPT 1289)`.
- Push target: `origin/paperwork/sprint-17-closeout-evidence-reconcile-1289`
  only.
- `main` not pushed by this prompt.

## Recommendation

The PROMPT 1278 Team-QA verdict (`APPROVED-WITH-CONDITIONS`), the PROMPT 1279
closeout disposition (`closed-with-conditions`), and the PROMPT 1288 Team-QA
refresh (`APPROVED-WITH-CONDITIONS`) for Sprint 17 **stand**. The single
documentation/evidence condition originating from PROMPT 1278 and carried
forward by PROMPT 1279 is **discharged** by PROMPT 1284 + PROMPT 1288. All
other carried conditions remain open and must continue to be honored by
downstream prompts.

The next orchestrator-launchable prompt is the Sprint 18 plan draft (PROMPT
1285) followed by Sprint 18 activation readiness and the Sprint 18 QA plan,
in that order, per `sprint_17_closeout.next_launchable_prompts`. **No Sprint 18
activation, no Polish → Release retry, no release claim, no stage advance, and
no closure of any other carried condition is unlocked by this reconcile.**

1289: SPRINT-17-CLOSEOUT-EVIDENCE-RECONCILE: APPROVED-WITH-CONDITIONS
