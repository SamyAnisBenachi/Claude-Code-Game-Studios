# PROMPT 1320 -- Sprint 18 QA Plan Integration Refresh + Main-Land Report

Date: 2026-05-19
Author: PROMPT 1320 worker (Claude Code, opus-tier integration-refresh sub-agent)
Mode: serialized main-land prompt, explicit orchestrator authorization.

## Outcome

`1320: SPRINT-18-QA-PLAN-INTEGRATION-REFRESH: LANDED`

`origin/main` advanced via strict fast-forward from
`3be6c25064993f29a6b3eaf524f1999260405fac` (PROMPT 1314 Sprint 18
activation main-land tip) to the PROMPT 1318 QA-plan branch tip plus
this PROMPT 1320 final report commit. No `--force`, no
`--force-with-lease`, no rebase, no merge commit.

This prompt is paperwork-only. It carries forward the PROMPT 1318
Sprint 18 QA plan author commit onto `origin/main`. It introduces no
source/test/Cargo changes and asserts no release/QA/gate sign-off
claims; the QA plan file itself is a paperwork artifact that binds
QA workflow sequencing per its own `Plan Prerequisite Note` section.

## "Refresh" verdict vs PROMPT 1319

PROMPT 1320 was authored as a possible refresh on top of a PROMPT
1319 main-land that did not occur (no `reports/PROMPT-1319-*.md`
artifact, no `1319` reference in the `git log --all` history at this
worker's fetch time, no `main`-tip advancement past
`3be6c25` since the PROMPT 1314 land). The PROMPT 1318 work branch
`origin/work/qa-plan-sprint-18-1318@8eedaf6` was therefore already
strict-fast-forward-eligible against the current `origin/main` tip
without any rebase. **No refresh diff was required.** The PROMPT 1320
land is a direct fast-forward of the PROMPT 1318 branch tip.

## Source refs

- QA plan work branch: `origin/work/qa-plan-sprint-18-1318`
- Branch tip at fetch: `8eedaf6` (`qa-plan(s18): author Sprint 18 QA plan (PROMPT 1318)`)
- Parent chain verified:
  - `8eedaf6` qa-plan(s18): author Sprint 18 QA plan (PROMPT 1318)
  - `3be6c25` report(prompt-1314): s18 activation main-land  ← `origin/main` at PROMPT 1320 fetch
  - `04546ee` report(prompt-1310): s18 activation refresh main-land
  - `5930f7d` activate(s18): flip sprint 17 -> 18 and status closed-with-conditions -> active (PROMPT 1301)
- `git merge-base --is-ancestor 3be6c25 8eedaf6` = TRUE (strict-FF eligible).

## Pre-push origin/main

- `origin/main` at fetch: `3be6c25064993f29a6b3eaf524f1999260405fac`.
- Identical to the parent recorded on the QA-plan work branch.
- No `origin/main` advancement since the PROMPT 1318 author commit.
- No PROMPT 1319 land observed; no overlap inspection triggered.

## Worktree used

Fresh worktree at
`D:/_DEV/claude-code-game-studios-worktrees/sprint-18-qa-plan-integration-refresh-1320`,
created via `git worktree add ... -b integrate/sprint-18-qa-plan-1320 3be6c25`,
then strict-FF merged to `8eedaf6` (`git merge --ff-only 8eedaf6`).
Root checkout (`D:/_DEV/Work/Claude-Code-Game-Studios`, branch
`mainland/s18-server-dead-state-hygiene-1315`) is dirty with a stale
PROMPT 1317 `production/qa/qa-plan-sprint-18.md` staged file from a
prior session and was NOT used for the main-land push, per the
established main-land convention (see PROMPT 1314 report §"Worktree
used").

## Files changed on main-land surface (vs origin/main @ 3be6c25)

From the PROMPT 1318 work branch (1 file):

```
production/qa/qa-plan-sprint-18.md   | 847 ++++++++++++++++++++++++++++
```

Plus this report (added as a follow-up commit on top of `8eedaf6`):

```
reports/PROMPT-1320-sprint-18-qa-plan-integration-refresh.md
```

All within the allowed file list for a QA-plan paperwork land. No
forbidden surfaces touched (`client/**`, `server/**`, `shared/**`,
`tests/**`, `Cargo.toml`, `Cargo.lock`, `production/stage.txt`,
`production/sprint-status.yaml`, `production/sprints/sprint-18.md`,
`production/sprints/sprint-17.md`, any earlier sprint plan, any
`production/session-state/*` file, any `production/gate-checks/*`
file, any story file under `production/epics/**`, any
release/gate/QA sign-off artifact).

## Invariant verification (executed in the fresh worktree at integration tip)

1. `production/stage.txt` UNCHANGED vs `origin/main` (`git diff
   --name-only 3be6c25..HEAD` does NOT list `production/stage.txt`).
   VERIFIED.
2. `production/sprint-status.yaml` UNCHANGED vs `origin/main` (`git
   diff --name-only 3be6c25..HEAD` does NOT list
   `production/sprint-status.yaml`). VERIFIED. Top-level fields
   `sprint: 18`, `status: active`, `stage: Polish` and the
   `sprint_18_activation:` block remain at the PROMPT 1314 main-land
   values; no row status flips occurred.
3. `production/sprints/sprint-18.md` UNCHANGED vs `origin/main`
   (likewise excluded from `git diff --name-only`). VERIFIED. The
   PROMPT 1301 `ACTIVATED` banner above the PROMPT 1285 `DRAFT`
   banner is preserved.
4. `production/qa/qa-plan-sprint-18.md` was ABSENT on `origin/main@3be6c25`
   and IS PRESENT on the integration tip at 847 lines (`git ls-tree
   HEAD -- production/qa/qa-plan-sprint-18.md` returns the blob). The
   blob hash matches `origin/work/qa-plan-sprint-18-1318@8eedaf6:production/qa/qa-plan-sprint-18.md`
   (no in-flight edits, no formatting changes). VERIFIED.
5. `production/gate-checks/gate-polish-release-2026-05-12.md` FAIL
   verdict preserved (file untouched in this diff). VERIFIED by
   `git diff --name-only` exclusion. No PROMPT 761 retry attempted
   or recorded.
6. `git diff --check 3be6c25..HEAD` = clean (no whitespace errors,
   no conflict markers). VERIFIED.
7. The QA-plan body's `Plan Prerequisite Note` section verbatim
   restates the binding sequencing: this plan binds in full only
   once Sprint 18 activation is on `origin/main` AND the PROMPT 1284
   Sprint 17 post-fmt smoke rerun is reconciled into the Sprint 17
   closeout block AND this QA plan is on `origin/main` at or after
   the activation tip. The first and third conditions are now satisfied
   by this main-land; the second condition (PROMPT 1284 smoke
   reconcile) is owned by a separate prompt sequence and is out of
   PROMPT 1320 scope. VERIFIED by file content read.
8. No `/dev-story`, `/story-done`, `/story-readiness`, `/smoke-check`,
   `/team-qa`, `/gate-check`, `/release-check`, or QA sign-off was
   triggered by this prompt. VERIFIED (no row-status writes, no
   story file edits, no `production/qa/evidence/**` writes, no
   `production/qa/smoke-*.md` writes).
9. No source files under `client/**`, `server/**`, `shared/**`,
   or `tests/**` touched. VERIFIED.

## Cargo policy

Cargo policy N/A. No Cargo commands were issued and no Cargo-relevant
files exist in the main-land diff. Windows/MSVC Cargo resource policy
not applicable; the Bevy 0.18 / Lightyear 0.26 stack was not loaded.

## Strict-FF push verification

1. Fetched `origin/main` at integration start: `3be6c25` (unchanged
   from the PROMPT 1314 main-land tip).
2. Created `integrate/sprint-18-qa-plan-1320` from `3be6c25` in the
   fresh worktree.
3. Strict-FF-merged `8eedaf6` (`git merge --ff-only`) — output
   `Updating 3be6c25..8eedaf6`, single-file change confirmed.
4. Pushed `integrate/sprint-18-qa-plan-1320` to `origin` —
   `* [new branch]      integrate/sprint-18-qa-plan-1320 -> integrate/sprint-18-qa-plan-1320`.
5. Committed this report on top of `8eedaf6`.
6. Re-pushed `integrate/sprint-18-qa-plan-1320`.
7. Strict-FF-pushed `integrate/sprint-18-qa-plan-1320:main` with no
   `--force`, no `--force-with-lease`. `origin/main` advanced from
   `3be6c25` to the integration-branch tip.

## Allowed-surface summary

| Surface | Touched? | Notes |
|---|---|---|
| `production/qa/qa-plan-sprint-18.md` | NEW (847 lines) | The PROMPT 1318 QA plan author commit. |
| `reports/PROMPT-1320-sprint-18-qa-plan-integration-refresh.md` | NEW | This report. |
| `production/stage.txt` | UNTOUCHED | `Polish` preserved. |
| `production/sprint-status.yaml` | UNTOUCHED | Sprint 18 activation block preserved. |
| `production/sprints/sprint-18.md` | UNTOUCHED | ACTIVATED+DRAFT banner stack preserved. |
| `production/gate-checks/**` | UNTOUCHED | PROMPT 761 FAIL preserved; no retry. |
| `production/epics/**` (story files) | UNTOUCHED | No row status changes. |
| `production/session-state/**` | UNTOUCHED | Ephemeral state untouched. |
| `client/**`, `server/**`, `shared/**`, `tests/**` | UNTOUCHED | No source/test edits. |
| `Cargo.toml`, `Cargo.lock` | UNTOUCHED | No dependency changes. |

## Carried-condition list (preserved verbatim from QA plan body)

PROMPT 1320 does NOT close any of the following carried conditions:

- `S8-QA-001-W1`
- `QA-COND-0005`
- `QA-COND-0006`
- `PAW-TD-*-a`
- `TQ-S12-C1..C7` (including `TQ-S12-C7` AppCompat informational condition)
- PROMPT 683-era runtime divergence question
- Sprint 12 story 019 disposition
- PROMPT 1054 P1 UI snapshot retest `BLOCKED-HUMAN-OPERATOR` verdict
- 24 PROMPT 1022 QA snapshot audit findings
- PROMPT 1076 / 1077 long-tail findings deferred to Sprint 18+
- `S18-UI-HAND-RESERVE-STRIP-CLEANUP-001` story-authoring-needed row
  (out of scope per Sprint 18 plan draft §2.3; excluded from the
  PROMPT 1318 QA plan body)

## Status line

`1320: SPRINT-18-QA-PLAN-INTEGRATION-REFRESH: LANDED`
