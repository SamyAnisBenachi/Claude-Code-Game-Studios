# Gate Check: Polish → Release (Sprint 10 close-out)

**Date**: 2026-05-11
**Checked by**: `/gate-check` skill (PROMPT 676)
**Source of truth**: `origin/main @ 217428a` (wave 8)
**Stage at run**: `Polish` (`production/stage.txt`)
**Target stage**: `Release`
**Review mode**: `lean` (default — `production/review-mode.txt` absent)
**Scope policy applied**: `project_scope.md` — friend-game ONLY skips accessibility-tier
criteria (QA-COND-0005) + commercial-release artifacts (store metadata, public-release
claims). ALL OTHER criteria (functionality, tests, polish, perf, code-review visibility,
QA pipeline, balance, changelog, smoke) keep normal quality bar and must PASS.

---

## Phase 0 — Precondition Check

| Precondition | Required artifact | Status |
|---|---|---|
| PROMPT 674 — `/smoke-check sprint` Sprint 10 | `production/qa/smoke-sprint-10-*.md` | **MISSING** |
| PROMPT 675 — `/team-qa sprint` Sprint 10 | `production/qa/qa-signoff-sprint-10-*.md` | **MISSING** |
| Sprint 10 QA plan | `production/qa/qa-plan-sprint-10-*.md` | **MISSING** |

Carry-state (`codex-orchestrator-state.md` lines 3010–3018) explicitly queues 674/675/676
as a sequential trio. Only PROMPT 676 fired this session — 674 and 675 were NOT run.

`production/sprint-status.yaml` records `qa_plan_found: false` with note: *"No Sprint 10
QA plan exists at activation time. A Sprint 10 QA plan must be authored via /qa-plan
sprint before any gate-check or sprint close-out claim."* The sprint plan
(`production/sprints/sprint-10.md`:40–42) carries the same rule.

**Precondition gate**: FAIL. The Polish → Release gate cannot honestly evaluate
on artifact-existence checks alone — its primary inputs are missing.

---

## Phase 1 — Required Artifacts ([6/13 present, 7 missing])

### Friend-game-scope APPLICABLE (must PASS)

- [x] **All features from milestone plan are implemented** — Sprint 10 = 6/6 Must + 2/3
      Should integrated. Last Should (`S10-TD-003`) explicitly deferred Sprint 11 per
      carry-state. Bug-fix tail (Finding D `217428a`, V3 Worker A `d9ee107`) integrated;
      user retests pending but not gate-blocking under friend-game scope.
- [x] **Content is complete** — Card pool, lobby/HUD/shop-auction chrome assets present
      per PAW-002..006 closure and Sprint 10 chrome stories.
- [ ] **QA test plan exists** — `production/qa/qa-plan-sprint-10-*.md` **MISSING**.
      `sprint-status.yaml: qa_plan_found: false`.
- [ ] **QA sign-off report exists** (`/team-qa` APPROVED / APPROVED WITH CONDITIONS) —
      `production/qa/qa-signoff-sprint-10-*.md` **MISSING**. PROMPT 675 was queued but
      not fired.
- [x] **All Must Have story test evidence is present** — `production/qa/evidence/`
      contains sprint-10-hud-chrome-evidence.md, sprint-10-lobby-chrome-evidence.md,
      sprint-10-shop-auction-chrome-evidence.md, sprint-10-plugin-registration-audit.md,
      client-tracing-init-fix-evidence.md. Each Sprint 10 Must Have closure paperwork
      ties to one of these. Tests in `tests/integration/hud/`, `tests/integration/lobby/`,
      `tests/integration/shop_auction/`, `tests/integration/hand/` exist for the relevant
      stories.
- [ ] **Smoke check passes cleanly (PASS verdict) on RC build** —
      `production/qa/smoke-sprint-10-*.md` **MISSING**. PROMPT 674 was queued but not
      fired. Last smoke is sprint-8 (`smoke-sprint-8-2026-05-07.md`, PASS WITH WARNINGS).
- [ ] **No test regressions from previous sprint** — UNVERIFIED in this session
      (would be captured by PROMPT 674).
- [ ] **Balance data reviewed (`/balance-check` run)** — **MISSING**. No
      `production/qa/balance-check-*.md` artifact found. ECO-004 (kill/objective reward
      loop, `9fb8e60`) landed in Sprint 10 — balance impact NOT reviewed via skill.
- [ ] **Release / launch checklist completed** — **MISSING**. No
      `production/qa/release-checklist-*.md` or `launch-checklist-*.md`.
- [ ] **Changelog / patch notes drafted** — **MISSING**. No `production/changelog/`
      directory; no `patch-notes-*.md` artifact found.

### Friend-game-scope SKIPPABLE (commercial-release / accessibility — accept-risk)

- [—] **Store metadata prepared** — N/A (friend-game; no commercial release).
      Accept-risk per `project_scope.md`.
- [—] **Localization externalized** — N/A (friend-game; single-locale, no public release).
- [—] **EULA / privacy policy / age ratings** — N/A (friend-game; no public release).

---

## Phase 2 — Quality Checks ([1/8 verified PASS, 5 unverified, 2 accept-risk])

- [ ] **Full QA pass signed off by qa-lead** — NOT DONE (PROMPT 675 not run).
- [?] **All tests passing** — UNVERIFIED in this session. `liv-bevy-018` integration
      tests at `tests/integration/hud/`, `tests/integration/hand/` PASS per
      Sprint 10 closure paperwork. Sprint 11-preview tech-debt batch (PROMPTs 677-681
      in flight) explicitly addresses known compilation/test failures: 12 × E0596 in
      `lobby_asset_wiring_test`, broken `*_harness.rs` bins, 8 `Messages<...>`
      fixture files, HUD test-fixture cascade (hud_asset_wiring_test 0/6,
      hud_plugin_scaffold_test 3/4). Without a fresh `/smoke-check` these regressions
      are NOT contained to "deferred Sprint 11" — they may indicate active test-suite
      drift on `main`.
- [ ] **Performance targets met** — UNVERIFIED. No `/perf-profile` artifact for
      Sprint 10 found. `S11-TD-SERVER-LOG-SPAM-001` notes server log 396k+ lines per
      session as a perf concern.
- [x] **No known critical/high/medium-severity bugs (under friend-game scope)** —
      Open QA-COND-0001..0007 are accept-risk for friend-game scope per existing
      sprint-status carried-conditions. Finding D (`217428a`) and V3 Worker A
      (`d9ee107`) repair commits landed; user retests still pending but not blocking
      paperwork close-out under accept-risk.
- [—] **Accessibility basics covered** — N/A under `project_scope.md` friend-game
      carve-out (QA-COND-0005 accept-risk for Standard-tier accessibility gaps).
- [—] **Localization verified** — N/A (friend-game, single-locale).
- [—] **Legal requirements met** — N/A (no public release).
- [?] **Build compiles and packages cleanly** — UNVERIFIED in this session (would be
      captured by PROMPT 674 smoke). `cargo check` not run as part of this gate.

---

## Phase 3 — Director Panel Assessment

**Skipped this run.** The gate already fails on Phase 0 + Phase 1 artifact-existence
checks for non-skippable items (3 missing required artifacts + 4 missing
recommended artifacts in the must-PASS column). Spawning 4 directors would not
change the verdict, since the missing artifacts are factual gaps that require
the corresponding skills to be run (`/qa-plan`, `/smoke-check`, `/team-qa`,
`/balance-check`, `/launch-checklist`, `/changelog`) — not a judgement call.

Directors should be re-engaged once PROMPTs 674 + 675 + the auxiliary artifacts
exist, so they can read real reports rather than rule on absence.

---

## Phase 5a — Chain-of-Verification (FAIL draft)

1. **Have I accurately separated hard blockers from strong recommendations?** —
   Yes. Missing `/team-qa` and `/smoke-check` reports are skill-required prerequisites
   per the gate-check skill definition itself ("QA sign-off report exists (`/team-qa`
   output)" and "Smoke check passes cleanly … report exists in `production/qa/`").
   These are hard blockers, not preferences.
2. **Are there any PASS items I was too lenient about?** — No. All x-marked items
   are verified by filesystem evidence (story closures, evidence docs, integration
   commits on `main`).
3. **Am I missing any additional blockers the user should know about?** — Yes, two:
   (a) Sprint 11-preview tech-debt batch (677-681) contains test-suite breakages
   that touch `main` test runtime — a fresh smoke-check would surface these and may
   re-classify them as Sprint 10 blockers; (b) user retests for Finding D and V3
   Worker A are still in flight, and the Polish → Release gate explicitly requires
   no critical/high bugs.
4. **Can I provide a minimal path to PASS — the specific 3 things that must
   change?** — Yes:
   - Run `/qa-plan sprint` for Sprint 10 → produce `qa-plan-sprint-10-*.md`
   - Run PROMPT 674 (`/smoke-check sprint`) → produce `smoke-sprint-10-*.md` with
     PASS or PASS WITH WARNINGS
   - Run PROMPT 675 (`/team-qa sprint`) → produce `qa-signoff-sprint-10-*.md` with
     APPROVED or APPROVED WITH CONDITIONS
   - Then re-run PROMPT 676 with directors enabled, plus `/balance-check`,
     `/changelog` (sprint-10), `/launch-checklist` (friend-game-lean variant)
5. **Is the fail condition resolvable, or does it indicate a deeper design problem?** —
   Resolvable. No design problem; this is a pipeline-ordering issue. The user fired
   676 before 674 + 675 ran. The fix is to fire those first and then re-fire 676.

**Chain-of-Verification: 5 questions checked — verdict unchanged (FAIL).**

---

## Blockers (must resolve before re-running)

1. **Missing Sprint 10 QA plan** — Run `/qa-plan sprint` to author
   `production/qa/qa-plan-sprint-10-*.md`. Sprint-status.yaml and sprint-10.md both
   require this before any close-out claim.
2. **Missing Sprint 10 smoke-check** — Fire PROMPT 674 (`/smoke-check sprint`) and
   produce `production/qa/smoke-sprint-10-*.md` with PASS or PASS WITH WARNINGS.
3. **Missing Sprint 10 `/team-qa` sign-off** — Fire PROMPT 675 (`/team-qa sprint`) and
   produce `production/qa/qa-signoff-sprint-10-*.md` with APPROVED or APPROVED WITH
   CONDITIONS verdict (the user-stated precondition for this 676 run).

## Strong Recommendations (would convert CONCERNS → PASS but are not hard blockers
under friend-game scope)

4. **No `/balance-check` for ECO-004 reward-loop tuning** — Recommend running
   `/balance-check` to capture the economy delta from kill/objective reward wiring.
5. **No `/changelog` or `/patch-notes` for Sprint 10** — Recommend running
   `/changelog` to produce a sprint-10 changelog and `/patch-notes` for a friend-game
   patch summary.
6. **No `/launch-checklist` (friend-game lean variant)** — Even under friend-game
   scope, a lean launch-checklist establishes that the build artifact is shippable.
7. **User retests for Finding D + V3 Worker A still pending** — Should be confirmed
   before claiming "no critical bugs".
8. **Sprint 11-preview tech-debt batch in flight** — PROMPTs 677-681 surface known
   test-suite drift on `main`. Land their closures first; otherwise PROMPT 674 will
   re-discover the breakages.

## Accept-Risk Items (documented per `project_scope.md` friend-game carve-out)

- **QA-COND-0005** — Standard-tier accessibility gaps (carried).
- **QA-COND-0006** — Playtest fun-hypothesis evidence deferred.
- **QA-COND-0007** — Deferred manual visual evidence.
- **No store metadata** — N/A for friend-game (no commercial release).
- **No localization / EULA / age ratings** — N/A for friend-game.
- **`S8-QA-001-W1`** — Manual browser GAME_OVER evidence gap carried.

Carrying these items is consistent with all prior sprint closures and matches the
`sprint-status.yaml` `carried_conditions` block. Their accept-risk status does
NOT extend to Blockers 1-3 above, which are non-accessibility, non-commercial
quality gates.

---

## Verdict: **FAIL**

The gate fails on hard precondition + required-artifact checks: PROMPTs 674
(`/smoke-check`) and 675 (`/team-qa`) — both prerequisites for this gate per the
gate-check skill spec AND per the explicit text of PROMPT 676 ("Gate: 675 must be
APPROVED or APPROVED WITH CONDITIONS before this fires") — were not run before
this 676 invocation. Three other required-but-recommended artifacts (`balance-check`,
`changelog`, `launch-checklist`) are also absent.

Per `project_scope.md`, the gate **cannot** auto-PASS under "friend-game accept-risk"
on these items. They are quality-pipeline artifacts, not accessibility or
commercial-release artifacts. Project scope requires functionality + tests + polish +
QA visibility to pass at normal bar.

Sprint 10 work itself (6/6 Must + 2/3 Should done) is substantively complete on
`main`. **The blocker is the close-out paperwork pipeline, not the implementation.**

`production/stage.txt` is NOT advanced from `Polish` to `Release`.

---

## Follow-Up Actions

1. Land Sprint 11-preview tech-debt batch (PROMPTs 677-681 in flight) → clean up
   test-suite drift before `/smoke-check`.
2. Run `/qa-plan sprint` to author Sprint 10 QA plan.
3. Fire PROMPT 674 (`/smoke-check sprint` Sprint 10 scope).
4. Fire PROMPT 675 (`/team-qa sprint`) — requires the QA plan from step 2 and
   the smoke from step 3.
5. Run `/balance-check`, `/changelog`, `/launch-checklist` (friend-game lean).
6. Re-fire PROMPT 676 (`/gate-check Polish→Release`) — directors will spawn this
   time since artifacts will exist.

---

## Carry-State Update (to be added to `production/session-state/codex-orchestrator-state.md`)

- **PROMPT 676 verdict**: FAIL (precondition gap — 674/675 not run; balance-check /
  changelog / launch-checklist also missing).
- **Sprint 10 status**: substantively done, close-out paperwork incomplete.
- **`production/stage.txt`**: unchanged (`Polish`).
- **Sprint 11 planning**: still blocked on Sprint 10 close-out.
- **Accept-risk items carried**: QA-COND-0005/0006/0007, S8-QA-001-W1,
  no public-release / no release-candidate-readiness / no full-game-completion
  claims (unchanged from prior sprint).
