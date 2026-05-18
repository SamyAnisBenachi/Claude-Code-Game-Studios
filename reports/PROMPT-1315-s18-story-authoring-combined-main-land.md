# PROMPT 1315 -- S18-STORY-AUTHORING-COMBINED-MAIN-LAND (refresh-only; main NOT pushed)

Date: 2026-05-19
Author: PROMPT 1315 worker (Claude Code, opus-tier main-land sub-agent)
Mode: refresh-only on the combined 1313 integrate branch. NO push to
`origin/main`. Per mid-run orchestrator correction
(`D/NEEDS_RELAUNCH with exact refs and do not push main` +
`il doit utiliser la branche combinée origin/integrate/...-1313@cd4d059,
pas 1311 seul, et la refresh sur origin/main@3be6c25`).

## Outcome

`1315: S18-STORY-AUTHORING-COMBINED-MAIN-LAND: USE_1313_BRANCH_ON_3BE6C25`

Disposition: **NEEDS_RELAUNCH**. This prompt refreshes the combined
1313 integrate branch onto the post-1314 `origin/main` tip and
force-pushes the refreshed tip back to
`origin/integrate/s18-server-dead-state-hygiene-story-authoring-1313`.
No advancement of `origin/main`. A follow-up main-land prompt is
required to fast-forward `origin/main` to the refreshed tip.

## Exact refs

| Field | Value |
|-------|-------|
| Pre-refresh source branch (origin) | `origin/integrate/s18-server-dead-state-hygiene-story-authoring-1313` |
| Pre-refresh source tip | `cd4d05901438e94c2eab43d9839614394036f6fd` |
| Pre-refresh source parent (= old `origin/main` at 1313 authoring) | `3207cb4fc855332a26efaea1ac6bee2d0b1802ef` (PROMPT 1312 sang-meprise ADR main-land) |
| New `origin/main` tip used as refresh parent | `3be6c25064993f29a6b3eaf524f1999260405fac` (PROMPT 1314 Sprint 18 activation main-land) |
| Refresh interval landed on main between authoring and refresh | `3207cb4..3be6c25` = `5930f7d` (PROMPT 1301 activate s18) + `04546ee` (PROMPT 1310 report) + `3be6c25` (PROMPT 1314 report) |
| Post-refresh source-branch tip (this run) | `<final-integrate-tip-sha>` (this PROMPT 1315 report commit on top of `6046340`) |
| Post-refresh parent chain (most recent first) | this report commit -> `6046340` -> `6e4bc91` -> `075cb54` -> `3be6c25` -> `04546ee` -> `5930f7d` -> `3207cb4` |
| `git merge-base --is-ancestor 3be6c25 cd4d059` (pre-refresh) | FALSE -- main was NOT an ancestor of pre-refresh source tip. Refresh required. |
| `git merge-base --is-ancestor 3be6c25 <post-refresh-tip>` | TRUE -- strict-ff eligible from `origin/main` after refresh. |

## Cherry-pick mapping (PROMPT 1306 + 1311 + 1313 refreshed onto 3be6c25)

```
old SHA                                       new SHA            subject
00b21667125bdb06e95153b2d898d9397e7c6ff4  ->  075cb54a00a8e42b   story-authoring-integrate(s18-waves): reconcile PROMPT 1295 + PROMPT 1303 ... (PROMPT 1306)
413a1ff??? (PROMPT 1311 report)           ->  6e4bc9130139ffef   report(prompt-1311): s18 story-authoring waves main-land
cd4d05901438e94c2eab43d9839614394036f6fd  ->  6046340bba59d782   story-authoring-integrate(s18-server-dead-state-hygiene): ... (PROMPT 1313)
```

Cherry-pick was clean: zero conflicts, zero whitespace errors, no
manual merge resolution required. Verified by
`git status` = clean and `git diff --check origin/main..HEAD` = silent.

## Method

1. `git fetch origin --prune`. `origin/main` advanced from `3207cb4`
   (PROMPT 1312) to `3be6c25` (PROMPT 1314 activation main-land report)
   during the gap between PROMPT 1313 authoring and PROMPT 1315 launch.
   `origin/integrate/s18-server-dead-state-hygiene-story-authoring-1313`
   tip = `cd4d059` (unchanged since PROMPT 1313 push).
2. `git merge-base --is-ancestor 3be6c25 cd4d059` -> FALSE. Direct
   fast-forward of `origin/main` to `cd4d059` is NOT viable; refresh
   onto post-1314 tip required.
3. File-overlap audit between the three PROMPT 1306+1311+1313 commits
   on the source branch and the three PROMPT 1301+1310+1314 commits
   newly on `origin/main`:
   - 1306+1311+1313 surface (13 files): `production/epics/class-system/EPIC.md`,
     `production/epics/class-system/story-011-classchoice-drop.md`,
     `production/epics/hand-ui/story-012-activation-lock.md`,
     `production/epics/lightyear-protocol-verification/EPIC.md`,
     `production/epics/lightyear-protocol-verification/story-009-s18-protocol-snapshot-real-wire-tests.md`,
     `production/epics/lightyear-protocol-verification/story-010-s2c-activation-rejected-protocol-register.md`,
     `production/epics/lightyear-protocol-verification/story-011-playersnapshot-submitted-disposition.md`,
     `production/epics/round-state-machine/EPIC.md`,
     `production/epics/round-state-machine/story-007-s18-rsm-submissions-received-clear.md`,
     `production/epics/round-state-machine/story-008-auction-safety-timer-remove.md`,
     `reports/PROMPT-1306-...md`, `reports/PROMPT-1311-...md`,
     `reports/PROMPT-1313-...md`.
   - 1301+1310+1314 surface on main (6 files):
     `production/session-state/active.md`,
     `production/session-state/codex-orchestrator-state.md`,
     `production/sprint-status.yaml`,
     `production/sprints/sprint-18.md`,
     `reports/PROMPT-1310-...md`, `reports/PROMPT-1314-...md`.
   - **Zero file-path overlap**. Clean refresh expected and confirmed.
4. Local refresh branch: `mainland/s18-server-dead-state-hygiene-1315`
   created off `origin/main` (= `3be6c25`).
5. `git cherry-pick 00b2166 413a1ff cd4d059` -> three clean
   applications; resulting tip `6046340`.
6. `git diff --stat origin/main..HEAD` matches the union expected
   (3296 insertions / 8 deletions across the 13-file 1306+1311+1313
   surface; no edits to sprint-status / session-state /
   sprints / qa / gate-checks / source / tests / Cargo /
   docs/architecture / stage.txt / production/qa).
7. Added this PROMPT 1315 paperwork report on top of `6046340`.
8. Force-pushed the refreshed tip back to
   `origin/integrate/s18-server-dead-state-hygiene-story-authoring-1313`
   with `--force-with-lease` (lease anchored to pre-refresh tip
   `cd4d05901438e94c2eab43d9839614394036f6fd`) so any concurrent
   worker advance would fail safely.
9. **`origin/main` was NOT pushed.** The Sprint 18 activation tip on
   main is unchanged at `3be6c25064993f29a6b3eaf524f1999260405fac`.

## Files changed on refresh surface (post-refresh tip vs origin/main @ 3be6c25)

From the three refreshed 1306+1311+1313 commits (13 files; verbatim
trees from `cd4d059`):

```
production/epics/class-system/EPIC.md                                                         | +1
production/epics/class-system/story-011-classchoice-drop.md                                   | new (PROMPT 1305 verbatim)
production/epics/hand-ui/story-012-activation-lock.md                                         | modified (PROMPT 1303 stale-OQ8 + xref renumber)
production/epics/lightyear-protocol-verification/EPIC.md                                      | extended (slot-009/010/011 rows)
production/epics/lightyear-protocol-verification/story-009-s18-protocol-snapshot-real-wire-tests.md      | new (PROMPT 1306)
production/epics/lightyear-protocol-verification/story-010-s2c-activation-rejected-protocol-register.md  | new (PROMPT 1306)
production/epics/lightyear-protocol-verification/story-011-playersnapshot-submitted-disposition.md       | new (PROMPT 1305 -> renumbered to slot 011)
production/epics/round-state-machine/EPIC.md                                                  | extended (slot-007/008 rows)
production/epics/round-state-machine/story-007-s18-rsm-submissions-received-clear.md          | new (PROMPT 1306)
production/epics/round-state-machine/story-008-auction-safety-timer-remove.md                 | new (PROMPT 1305 -> renumbered to slot 008)
reports/PROMPT-1306-s18-story-authoring-waves-integration-reconcile.md                        | new
reports/PROMPT-1311-s18-story-authoring-waves-main-land.md                                    | new
reports/PROMPT-1313-s18-server-dead-state-hygiene-story-authoring-integration-reconcile.md    | new
```

Plus this report (as a follow-up commit):

```
reports/PROMPT-1315-s18-story-authoring-combined-main-land.md
```

All within the allowed file list for a docs-only story-authoring
integration refresh. No forbidden surfaces touched (`client/**`,
`server/**`, `shared/**`, `tests/**`, `Cargo.toml`, `Cargo.lock`,
`production/stage.txt`, `production/qa/**`,
`production/gate-checks/**`, `production/sprint-status.yaml`,
`production/sprints/**`, `production/session-state/**`,
`docs/architecture/**`).

## Invariant verification (post-refresh, against origin/main @ 3be6c25)

1. `production/stage.txt` = `Polish`, UNCHANGED (not in diff). VERIFIED.
2. `production/sprint-status.yaml` UNCHANGED from main (not in diff);
   `sprint:18, status:active, stage:Polish,
   sprint_18_activation:` block preserved verbatim from PROMPT 1314.
   VERIFIED.
3. `production/sprints/sprint-18.md` ACTIVATED banner above DRAFT
   banner UNCHANGED (not in diff). VERIFIED.
4. `production/session-state/active.md` +
   `production/session-state/codex-orchestrator-state.md` UNCHANGED
   from main (not in diff). VERIFIED.
5. `production/qa/qa-plan-sprint-18.md` absent. VERIFIED.
6. `production/gate-checks/gate-polish-release-2026-05-12.md` FAIL
   verdict preserved (file not in diff). VERIFIED.
7. `git diff --check origin/main..HEAD` clean. VERIFIED.
8. ADR-024 (`docs/architecture/adr-024-sang-meprise-reveal-mechanism.md`)
   preserved verbatim (file not in diff). VERIFIED.
9. Story-007/008 numbering on round-state-machine, and story-009/010/011
   numbering on lightyear-protocol-verification, both match PROMPT 1313
   reconcile decisions: PROMPT 1306 keeps the lower slots; PROMPT 1305
   renumbered to story-008 (RSM) and story-011 (LYP). VERIFIED via
   per-file content read.

## Cargo policy

Cargo policy N/A. No Cargo commands were issued and no Cargo-relevant
files exist in the refresh diff. Windows/MSVC Cargo resource policy
not applied (not required).

## Push

- `origin/integrate/s18-server-dead-state-hygiene-story-authoring-1313`:
  force-pushed (`--force-with-lease=cd4d05901438e94c2eab43d9839614394036f6fd`)
  to the post-refresh tip.
- `origin/main`: **NOT pushed.** Remains at
  `3be6c25064993f29a6b3eaf524f1999260405fac` (PROMPT 1314 activation
  main-land tip).

## Non-claims (explicit)

This prompt does NOT make and MUST NOT be read as making any of the
following claims:

- No `origin/main` advancement. `origin/main` remains at PROMPT 1314
  activation main-land tip `3be6c25`.
- No Polish -> Release retry. PROMPT 761 `Polish -> Release`
  gate-check FAIL verdict at
  `production/gate-checks/gate-polish-release-2026-05-12.md` remains
  in force with NO retry attempted. `production/stage.txt` remains
  `Polish`.
- No release readiness claim.
- No RC (release candidate) readiness claim.
- No full-game completion claim.
- No final-art completion claim.
- No broad accessibility advancement claim.
- No playtest validation claim.
- No `S11-HUD-TIMER-EYEBALL-VISUAL-001` closure claim.
- No `S17-UI-HUD-OPP-MANA-CLEANUP-001` parent-row closure claim.
- No `/story-readiness`, `/dev-story`, `/story-done`, `/smoke-check`,
  `/gate-check`, `/qa-plan`, or release-readiness work was performed.
- No source code, tests, or build artifacts were modified. No
  `src/**`, `client/**`, `server/**`, `shared/**`, `tests/**`,
  `Cargo.toml`, `Cargo.lock`, `assets/**`, or shader changes.
- No new sprint authored. Sprint 18 activation state on main carried
  forward from PROMPT 1314 verbatim.
- No story Draft -> Ready promotion. PROMPT 1306 + 1305 integrated
  candidate stories remain Draft Sprint 18 candidates.
- No PROMPT 1296 absorption. PROMPT 1296 reconcile remains a separate
  follow-up if/when the orchestrator schedules it.

## Next steps for orchestrator

Disposition is **NEEDS_RELAUNCH**: a follow-up serialized main-land
prompt must be launched to fast-forward `origin/main` from
`3be6c25` to the refreshed integrate-branch tip. The refreshed branch
is now strict-ff-eligible from current `origin/main` (verified by
`git merge-base --is-ancestor 3be6c25 <post-refresh-tip>` = TRUE).

Suggested next-prompt scope (paperwork-only, single push):

1. Fetch and verify `origin/main` still at `3be6c25` and integrate
   branch tip at the post-refresh SHA recorded above.
2. Confirm strict-ff ancestry holds.
3. `git push origin <post-refresh-tip>:refs/heads/main` (no force,
   no rebase, no merge commit).
4. Add a paperwork-only `reports/PROMPT-NNNN-...-main-land.md`
   follow-up commit and push as a strict-ff continuation.
5. Optionally `git push origin :integrate/s18-server-dead-state-hygiene-story-authoring-1313`
   to retire the integrate branch after land.
