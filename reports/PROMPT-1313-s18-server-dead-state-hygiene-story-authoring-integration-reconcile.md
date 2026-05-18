# PROMPT 1313 -- S18-SERVER-DEAD-STATE-HYGIENE-STORY-AUTHORING-INTEGRATION-RECONCILE

**Status**: READY_FOR_MAIN_LAND
**Mode**: docs-only integration / reconciliation
**Authored**: 2026-05-19 by PROMPT 1313
**Integration branch**: `integrate/s18-server-dead-state-hygiene-story-authoring-1313`
**Base**: `integrate/s18-story-authoring-waves-mainland-1311` @ `413a1ff3859787421e0e6939a966643674ff8b39` (PROMPT 1311 main-land report on top of PROMPT 1306 refreshed reconcile `00b21667125bdb06e95153b2d898d9397e7c6ff4` on top of `origin/main@3207cb4c` PROMPT 1312 sang-meprise ADR main-land)
**Source branch reconciled**: `origin/work/s18-server-dead-state-hygiene-story-authoring-1305 @ d74f1d828e8b98d6e27f914474d2d61cdc07352d` (PROMPT 1305 authoring run)

---

## 1. Summary

PROMPT 1313 reconciles the PROMPT 1305 Sprint 18 server dead-state hygiene
story-authoring run (`d74f1d8`) onto main after PROMPT 1311. PROMPT 1305 was
authored against `origin/main@6239c9e` and assigned the following story slots
under `production/epics/**`:

- `round-state-machine/story-007-auction-safety-timer-remove.md`
- `lightyear-protocol-verification/story-009-playersnapshot-submitted-disposition.md`
- `class-system/story-011-classchoice-drop.md`

PROMPT 1306 (already main-landed via PROMPT 1311) assigned overlapping slots:

- `round-state-machine/story-007-s18-rsm-submissions-received-clear.md` (from PROMPT 1295)
- `lightyear-protocol-verification/story-009-s18-protocol-snapshot-real-wire-tests.md` (from PROMPT 1295)
- `lightyear-protocol-verification/story-010-s2c-activation-rejected-protocol-register.md` (from PROMPT 1303, renumbered by 1306)

Resolution (PROMPT 1313):

| Conflict | Decision |
|----------|----------|
| `round-state-machine/story-007` | Keep PROMPT 1306's `story-007-s18-rsm-submissions-received-clear.md` (already on main-land via PROMPT 1311). Renumber PROMPT 1305's auction-safety-timer-remove to `story-008-auction-safety-timer-remove.md`. |
| `lightyear-protocol-verification/story-009` | Keep PROMPT 1306's `story-009-s18-protocol-snapshot-real-wire-tests.md` + `story-010-s2c-activation-rejected-protocol-register.md` (already main-landed). Renumber PROMPT 1305's playersnapshot-submitted-disposition to `story-011-playersnapshot-submitted-disposition.md`. |
| `class-system/story-011` | Uncontested -- `class-system` had `story-010-token-passives.md` as its highest slot on main-land. PROMPT 1305's `story-011-classchoice-drop.md` is preserved verbatim. |

All three story bodies are preserved verbatim from PROMPT 1305 except for:

- the `# Story NNN:` heading on line 1 of each renumbered file,
- intra-file references to the story's own evidence-file path
  (`tests/evidence/rsm-story-007-...` -> `tests/evidence/rsm-story-008-...`;
  `tests/evidence/lyp-story-009-...` -> `tests/evidence/lyp-story-011-...`).

All cross-story references inside the bodies are **story-ID-based**
(`S18-RSM-SUBMISSIONS-RECEIVED-CLEAR-001`,
`S18-PROTO-PLAYERSNAPSHOT-SUBMITTED-DISPOSITION-001`,
`S18-RSM-AUCTION-SAFETY-TIMER-REMOVE-001`,
`S18-PROTO-CLASSCHOICE-DROP-001`), not slot-number-based -- so no
cross-reference rewrites are needed inside the story bodies. The renumber is
mechanical.

The integration commit is docs-only and confined to `production/epics/**` +
this report under `reports/`. No source code, no Cargo, no tests, no sprint
state, no session state, no stage, no QA, no gate-checks, no ADR edits, no
sprint-status / `production/sprints/sprint-18.md` edits. Stage remains
`Polish`; Sprint 18 is NOT activated; Sprint 17 remains `closed-with-conditions`.

---

## 2. Conflict Analysis

### 2.1 `round-state-machine/story-007`

| Branch | File | Purpose |
|--------|------|---------|
| 1306 main-land (from 1295) | `story-007-s18-rsm-submissions-received-clear.md` | Sprint 18 candidate Logic fix: clear `submissions_received` on Placement -> Resolution exit (one-line transition fix + regression test). Source: PROMPT 1202 §2 F-07 / PROMPT 1287 §3.10 Lane W9. |
| 1305 (`d74f1d8`) | `story-007-auction-safety-timer-remove.md` | Sprint 18 candidate Config/Data + docs sync: remove dead `RoundState.auction_safety_timer` field + tick coverage + `rsm_scaffold_test.rs:19` assertion. Source: PROMPT 1298 §3 F-09. |

**Decision**: Keep 1306's slot-007. Renumber 1305 to slot-008.

**Rationale**: 1306 is already on main-land via PROMPT 1311. Re-renumbering
1306 (which is a "live" reconcile already shipped) is more disruptive than
shifting the still-pending 1305 author work up by one slot. 1305's F-09 dead
field removal is independent of the F-07 SUBMISSIONS_RECEIVED clear (different
fields, different code paths, no shared invariant); the relative ordering
007 -> 008 within the round-state-machine epic carries no semantic constraint.
Both stories cite PROMPT 1298 / PROMPT 1287 audit lineage; the chronology
(1295 authored 2026-05-18 ahead of 1305 authored 2026-05-18 later in the day)
matches the 007 -> 008 ordering.

### 2.2 `lightyear-protocol-verification/story-009`

| Branch | File | Purpose |
|--------|------|---------|
| 1306 main-land (from 1295) | `story-009-s18-protocol-snapshot-real-wire-tests.md` | Sprint 18 candidate Logic (test-infrastructure): real-wire snapshot helper + 4 test migrations. Source: PROMPT 1202 §2 F-08. |
| 1306 main-land (from 1303, renumbered) | `story-010-s2c-activation-rejected-protocol-register.md` | Sprint 18 candidate Config/Data: register `S2CActivationRejected` + `ActivationRejectedReason` in `shared/src/protocol.rs`. Source: PROMPT 1297 audit. |
| 1305 (`d74f1d8`) | `story-009-playersnapshot-submitted-disposition.md` | Sprint 18 candidate decision-first (Path A drop / Path B wire / Path C defer) for `PlayerSnapshot.submitted` field. Source: PROMPT 1298 §3 F-05. |

**Decision**: Keep 1306's slot-009 + slot-010. Renumber 1305 to slot-011.

**Rationale**: 1306's slot-009 (real-wire-tests) and slot-010 (s2c-activation-
rejected-register) are both already on main-land via PROMPT 1311. The 1306
sequence note documents the 007 -> 008 -> 009 "hardening rows from audit
reports" continuity and the 010 Config/Data adjacency. 1305's drop-vs-wire
disposition is the next natural slot (011) -- it follows the same
"decision-first" pattern established by `story-008-protocol-orphan-drain.md`
in this same epic, and its Path B is explicitly HARD-BLOCKED on
`S18-RSM-SUBMISSIONS-RECEIVED-CLEAR-001` (the F-07 fix sitting at the new
round-state-machine slot-007), so slot 011 sequencing follows the dependency
chain naturally.

### 2.3 `class-system/story-011`

| Branch | File | Purpose |
|--------|------|---------|
| 1305 (`d74f1d8`) | `story-011-classchoice-drop.md` | Sprint 18 candidate decision-first + Config/Data + docs sync (ADR-014 + control-manifest): drop dead `C2SClassChoice` protocol path. Source: PROMPT 1298 §3 F-06. Supersedes PROMPT 1202 placeholder `S14-PROTO-CLASSCHOICE-DISPOSITION-001` and closes the `lightyear-protocol-verification/story-008` orphan-drain allowlist row for `C2SClassChoice`. |

**Decision**: Preserve slot-011 verbatim. No collision -- class-system
highest slot on main-land was `story-010-token-passives.md`, so 011 is the
next free slot.

---

## 3. Files Touched by the Integration Commit

All paths under `production/epics/**` (allowed) plus this report under
`reports/` (allowed by task statement).

| File | Source | Operation |
|------|--------|-----------|
| `production/epics/round-state-machine/EPIC.md` | merged from 1305 | M -- append slot-008 row (auction-safety-timer-remove) + extend the post-table prose to cover the 008 row provenance (PROMPT 1305 + PROMPT 1313 renumber 007 -> 008) |
| `production/epics/round-state-machine/story-008-auction-safety-timer-remove.md` | 1305 | A -- body verbatim from `d74f1d8` `story-007-auction-safety-timer-remove.md`; only changes are `# Story 007:` -> `# Story 008:` (line 1) and one intra-file evidence-file reference `tests/evidence/rsm-story-007-...` -> `tests/evidence/rsm-story-008-...` |
| `production/epics/lightyear-protocol-verification/EPIC.md` | merged from 1305 | M -- append slot-011 row (playersnapshot-submitted-disposition) + extend the sequence note to cover the 011 row provenance (PROMPT 1305 + PROMPT 1313 renumber 009 -> 011) |
| `production/epics/lightyear-protocol-verification/story-011-playersnapshot-submitted-disposition.md` | 1305 | A -- body verbatim from `d74f1d8` `story-009-playersnapshot-submitted-disposition.md`; only changes are `# Story 009:` -> `# Story 011:` (line 1) and intra-file evidence-file references `tests/evidence/lyp-story-009-...` -> `tests/evidence/lyp-story-011-...` (drop / wire / defer flavours, 3 occurrences) |
| `production/epics/class-system/EPIC.md` | 1305 | M -- append slot-011 row (classchoice-drop) verbatim from 1305; row text amended only to add the PROMPT 1313 integration-by reference |
| `production/epics/class-system/story-011-classchoice-drop.md` | 1305 | A -- verbatim from `d74f1d8` (no renumber needed; uncontested slot) |
| `reports/PROMPT-1313-s18-server-dead-state-hygiene-story-authoring-integration-reconcile.md` | new | A -- this report |

---

## 4. Allowlist / Forbidden-List Compliance

**Allowed (touched)**: `production/epics/round-state-machine/**`,
`production/epics/lightyear-protocol-verification/**`,
`production/epics/class-system/**`, `reports/PROMPT-1313-*.md`.

**Forbidden (NOT touched in this integration commit)**:

- `production/sprint-status.yaml` -- NOT touched
- `production/sprints/sprint-18.md` and all other `production/sprints/**` -- NOT touched
- `production/stage.txt` -- NOT touched
- `production/session-state/**` -- NOT touched
- `production/qa/**` -- NOT touched
- `production/gate-checks/**` -- NOT touched
- `client/**`, `server/**`, `shared/**`, `tests/**` -- NOT touched
- `Cargo.toml`, `Cargo.lock` -- NOT touched
- `docs/architecture/**` -- NOT touched (ADR-009 + ADR-014 amendments are deliverables of the implementation prompts that land each story; the authoring run only records the *planned* ADR edits)
- `docs/engine-reference/**` -- NOT touched
- `design/**` -- NOT touched
- `.claude/**` -- NOT touched
- `production/epics/**/EPIC.md` rows for stories OTHER than the three new 1305-authored rows -- NOT touched (verified by `git diff --stat`)
- Existing story files in `round-state-machine/`, `lightyear-protocol-verification/`, `class-system/` -- NOT touched

---

## 5. No-Claim Banner (mirror of PROMPT 1305 + PROMPT 1306 + PROMPT 1311)

PROMPT 1313 (this integration run) does **NOT**:

- Activate Sprint 18. Sprint 18 is NOT activated by this integration.
- Modify `production/sprint-status.yaml`.
- Modify `production/sprints/sprint-18.md` or any other sprint file.
- Modify `production/stage.txt`. Stage remains `Polish`.
- Modify any `production/session-state/*` file.
- Modify `production/qa/**` or `production/gate-checks/**`.
- Run `/story-readiness`, `/dev-story`, `/story-done`, `/smoke-check`,
  `/team-qa`, `/gate-check`, or `/qa-plan` on any of the three integrated stories.
- Modify any code under `client/`, `server/`, `shared/`, or `tests/`.
- Modify any `Cargo.toml` / `Cargo.lock`.
- Modify any file under `docs/architecture/**` (planned ADR-009 + ADR-014 +
  control-manifest amendments remain deliverables of the implementation prompts
  that eventually land each story).
- Claim release readiness, RC readiness, full-game claim, sprint close-out,
  Polish -> Release advance, gate-check pass, or any production state advance.
- Claim that the three integrated stories are Ready, Implemented, Tested,
  or Done. They remain `Draft -- Sprint 18 candidate, NOT activated`.

Sprint 17 remains `closed-with-conditions` per PROMPT 1279. The PROMPT 1276
source-of-truth and the Sprint 17 evidence conditions carried by PROMPT
1284 + PROMPT 1288 + PROMPT 1289 remain in force.

---

## 6. Verification Notes

- No-duplicate-story-ID verification:
  - `round-state-machine/`: story-007 = S18-RSM-SUBMISSIONS-RECEIVED-CLEAR-001
    (from 1306); story-008 = S18-RSM-AUCTION-SAFETY-TIMER-REMOVE-001 (from
    1305 renumbered). Distinct story IDs, distinct files.
  - `lightyear-protocol-verification/`: story-009 = S18-PROTOCOL-SNAPSHOT-
    REAL-WIRE-TESTS-001 (from 1306); story-010 =
    S18-PROTOCOL-S2CACTIVATIONREJECTED-REGISTER-001 (from 1306); story-011 =
    S18-PROTO-PLAYERSNAPSHOT-SUBMITTED-DISPOSITION-001 (from 1305 renumbered).
    Distinct story IDs, distinct files.
  - `class-system/`: story-011 = S18-PROTO-CLASSCHOICE-DROP-001 (from 1305).
    No collision -- previous highest slot was 010-token-passives.
- Cross-reference scan inside 1305 story bodies (`S18-RSM-...`,
  `S18-PROTO-PLAYERSNAPSHOT-...`, `S18-PROTO-CLASSCHOICE-...`): all
  references are story-ID-based, not slot-number-based, and remain valid.
- No drift between the 4d339f5 (initial work base) and 413a1ff (PROMPT 1311
  HEAD) for the EPIC.md / story files touched by this integration: verified
  by `git diff 4d339f5 413a1ff -- production/epics/round-state-machine/EPIC.md
  production/epics/lightyear-protocol-verification/EPIC.md
  production/epics/class-system/EPIC.md` (empty output).

---

## 7. Worker Branch & Push Plan

- Branch: `integrate/s18-server-dead-state-hygiene-story-authoring-1313`
- Base: `integrate/s18-story-authoring-waves-mainland-1311` @ `413a1ff`
- Push target: same branch name on `origin` (worker branch only; main is NOT pushed by this worker).
- Main-land step is handled by a separate orchestrator prompt (PROMPT N+1 main-land), not by this worker.

