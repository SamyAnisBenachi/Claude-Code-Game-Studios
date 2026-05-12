# Gate Check: Polish -> Release (RETRY)

| Field | Value |
|---|---|
| Date | 2026-05-12 |
| Run by | `/gate-check` skill (lean review mode) |
| Target gate | Polish -> Release |
| Sprint context | Sprint 10 close-out window (post-smoke retry-7) |
| HEAD at gate time | `83bd8e5` (wave-12 state update post-smoke-retry-7) |
| Last code-bearing HEAD | `6b54eda` S11-TD-FIXTURE-CLASS-D-001 |
| Smoke report under review | `production/qa/smoke-sprint-10-2026-05-12-retry-7.md` (PASS WITH WARNINGS @ `bc96700`) |
| Triggering orchestrator prompt | PROMPT 761 |
| Review mode resolved | `lean` (default; no `production/review-mode.txt` override) |
| Verdict | **FAIL** |
| Stage disposition | **Stay in Polish.** `production/stage.txt` NOT modified. |

---

## Executive Summary

This retry attempted to advance the project from **Polish** to **Release** on the back
of the Sprint 10 smoke retry-7 PASS WITH WARNINGS verdict (1123/1123 tests pass, 11
documented `#[ignore]` markers). All four phase-gate directors (CD, TD, PR, AD)
returned **NOT READY** independently. The gate-check skill therefore returns **FAIL**.

The verdict is not a quality complaint against the Sprint 10 work itself — the smoke
verdict, the per-story `/story-done` closures, and the test suite are all consistent
with the work the sprint committed to deliver. The verdict is that **Sprint 10 was
scoped as "friend-game-lite," not as release-candidate preparation**, and the
sprint plan itself explicitly disclaims release scope. Promoting to Release would
contradict the team's own signed scope.

---

## Polish -> Release Required Artifact Check

| Status | Artifact / Check | Evidence |
|---|---|---|
| FAIL | Smoke check passes cleanly (PASS verdict) on RC build | Latest smoke (`smoke-sprint-10-2026-05-12-retry-7.md`) is **PASS WITH WARNINGS**, not clean PASS. Gate requires clean PASS for Release. |
| FAIL | QA sign-off report exists (`/team-qa` — APPROVED or APPROVED WITH CONDITIONS) | Last QA sign-off: `qa-signoff-sprint-8-2026-05-07.md`. No sign-off for Sprint 9 or Sprint 10. |
| FAIL | Release checklist completed (`/release-checklist` or `/launch-checklist`) | No file matches under `production/`. Only template files exist in `.claude/docs/templates/`. |
| FAIL | Changelog / patch notes drafted | No `production/changelog*.md` or `production/patch-notes*.md`. |
| FAIL | Balance data reviewed (`/balance-check`) | No balance-check artifact found. |
| FAIL | All features from milestone plan implemented at release quality | Sprint 10 carry block declares `release_candidate_readiness: false`, `public_release_readiness: false`, `full_game_completion_claimed: false`. |
| FAIL | Content complete (all levels, assets, dialogue) | All PAW-002..PAW-006 assets are tagged `accept-risk -- placeholder PNGs, not final art (friend-game scope)`. No final art exists. |
| CONCERNS | All Must Have story test evidence present | Logic/Integration tests pass; UI/Visual evidence is mostly `manual screenshot DEFERRED per friend-game-lite paperwork pattern`. Acceptable for Polish; insufficient for Release. |
| CONCERNS | No test regressions from previous sprint | 0 regressions in retry-7, but 11 new `#[ignore]` markers added (D-5 owner review pending). |
| PARTIAL | Localization strings externalized | Not scanned this run; project has no committed i18n scope. Note as open question for Release scope. |
| ACCEPTED RISK | Accessibility basics covered | QA-COND-0005 is accepted-risk friend-game scope only; Standard-tier completion not verified. Incompatible with public-release framing. |
| FAIL | Full QA pass signed off by `qa-lead` (release scope) | No `qa-signoff-release-*.md` exists; Sprint 10 has no sign-off at all. |
| CONCERNS | Performance targets met across target platforms | No `/perf-profile` artifact at release scope. WASM bundle size not measured against the <50 MB budget for this build. |
| OPEN | No known critical/high/medium-severity bugs | S8-QA-001-W1 manual/browser two-client GAME_OVER gap is still **open** carried forward from Sprint 8. |

**Required artifacts present: 0 / 13.**

---

## Quality Check Summary

| Status | Check | Evidence |
|---|---|---|
| PASS | Tests passing | 1123/1123 effective pass in retry-7. |
| PASS | `cargo fmt --check` clean | Per smoke report header. |
| WARN | 11 `#[ignore]`d tests pending D-5 owner review | Documented blockers per ignored test; not addressed yet. |
| FAIL | Full QA pass signed off | Not signed off for S9 or S10; no release-scope sign-off attempt. |
| FAIL | Build compiles and packages cleanly across release targets | WASM release-build artifact not produced this gate. |
| FAIL | Sprint-cursor discipline | `sprint-status.yaml` status: active for Sprint 10; HEAD already carries S11-* commits with `next_sprint.status: not_planned`. |
| FAIL | Fun hypothesis validated | QA-COND-0006 explicitly carried as accepted-risk / deferred. No playtest evidence for Sprints 7-10. |

---

## Director Panel Assessment

Spawned in parallel per `.claude/docs/director-gates.md` parallel-gate protocol.

### Creative Director: **NOT READY**

> Fun hypothesis is unvalidated by playtest evidence (zero playtest reports for
> Sprints 7-10; QA-COND-0006 deferred). Sprint 10 explicitly disclaims
> release-candidate scope -- phase advancement would be a process-integrity
> violation. Release-gate artifacts entirely absent (no launch-checklist,
> release-checklist, changelog, patch-notes). One manual visual playtest still
> outstanding per smoke retry-7 itself. Core fantasy is intact at friend-game-lite
> quality; not yet demonstrated at release-candidate quality. **Hold the line.**

### Technical Director: **NOT READY**

> Foundation ADRs incomplete (5 pending per CLAUDE.md: client-server authority,
> card data schema, round state machine, auction event flow, Sang Mepris reveal).
> Sprint discipline broken (sprint-status.yaml lists S10 active while HEAD carries
> S11 commits). No release-candidate performance evidence (no `/perf-profile`, no
> WASM <50 MB measurement, no 60 FPS capture). 11 ignored D-5 markers are
> unresolved architectural risk (state-transition class, not cosmetic). HUD timer
> bar lacks live playtest evidence -- ADVISORY in Polish, BLOCKING in Release.
> Open ops tickets (orchestrator-root concurrent-session lock, cargo workspace disk
> usage) touch build reproducibility.

### Producer: **NOT READY**

> Sprint 10 is not closed (status: active; no S9 or S10 QA sign-off). Carried
> conditions explicitly forbid release framing. Sprint 11 work in flight with
> `next_sprint.status: not_planned` -- dependencies not ordered. S8-QA-001-W1
> two-client GAME_OVER gap still open since Sprint 8. Zero release artifacts
> exist. Friend-game accepted-risk scope is incompatible with "Release" framing
> unless redefined in writing. **Recommend staying in Polish for at least one
> more planned sprint.**

### Art Director: **NOT READY**

> ALL shipped art is explicitly tagged placeholder PNGs (PAW-TD-002-a through
> PAW-TD-006-a). Zero final-quality art exists. Visual Identity Anchor section
> not located in `design/gdd/game-concept.md` (file existence to be verified).
> HUD timer bar (`112ac83`) has no post-cherry-pick live visual playtest.
> Art Bible sign-off (2026-05-01) was scoped to friend-game / lean mode -- not
> Release-grade. Final art production must complete before this gate can pass.

**Director panel verdict:** 0/4 READY, 0/4 CONCERNS, **4/4 NOT READY**.

Per `.claude/docs/director-gates.md` escalation rule: any NOT READY -> overall
verdict minimum FAIL. **All four** NOT READY removes any ambiguity.

---

## Blockers (Hard, in Priority Order)

1. **Sprint 10 is not closed.** `sprint-status.yaml.status: active`. No QA sign-off
   for Sprint 9 or Sprint 10. Sprint plan itself explicitly disclaims release
   scope. The sprint close-out workflow (`/team-qa`, `/qa-signoff`, sprint
   `/story-done` for remaining stories, sprint-status flip to `closed-with-...`)
   must complete before any release-tier gate can run.

2. **No release-tier artifacts exist.** `/release-checklist`, `/launch-checklist`,
   changelog, player-facing patch notes, release-scope QA plan, release-scope
   smoke build, and balance-check are all absent. Release phase entry requires
   a release-phase artifact baseline; the project does not have one.

3. **All art is placeholder.** Every PAW deliverable carries an explicit
   `accept-risk -- placeholder PNGs, not final art (friend-game scope)` tech-debt
   marker. Final art production is a Release-blocking dependency that has not yet
   begun in any visible sprint plan.

4. **Fun hypothesis unvalidated.** QA-COND-0006 has been carried as accepted-risk
   since Sprint 6. No playtest report exists for Sprints 7, 8, 9, or 10. Releasing
   without playtest evidence answering the central design question is unsupported.

5. **Foundation ADRs incomplete.** Five pending ADRs (client-server authority,
   card data schema, round state machine, auction event flow, Sang Mepris reveal)
   per CLAUDE.md "Pending ADRs needed" list. Architecture contracts cannot ship
   undocumented.

6. **Carried open defect.** S8-QA-001-W1 two-client GAME_OVER manual/browser gap
   has remained OPEN since Sprint 8 close-out. Multiplayer end-state defect of
   unknown severity is incompatible with Release.

7. **11 ignored tests pending owner review.** D-5 class markers added in retry-7
   to clear the smoke gate are documented blockers, not resolutions. Each needs
   an owner decision (fix / formal accept-risk / re-design) before Release.

8. **Sprint cursor drift.** `sprint-status.yaml` shows Sprint 10 active while
   HEAD carries S11-* commits and `next_sprint.status: not_planned`. Phase gate
   cannot be evaluated against an ambiguous sprint cursor.

---

## Recommendations (Path Forward)

Minimum path to a re-runnable Polish -> Release gate:

1. **Close Sprint 10 properly.**
   - Run `/team-qa sprint-10` -> produce `qa-signoff-sprint-10-*.md`.
   - Run `/story-done` for any remaining ready stories (S10-TD-003 doc hygiene,
     S10-N1 evidence index, S10-N2 readability notes).
   - Flip `sprint-status.yaml.status` to `closed-...` with appropriate disposition.

2. **Plan Sprint 11 formally.**
   - Run `/sprint-plan sprint-11` to give the in-flight S11 commits a home.
   - Decide explicitly: is Sprint 11 the last Polish sprint, or the first Release
     sprint? Record the choice in the sprint plan header.

3. **Define "Release" scope in writing.**
   - Is this a public release or a permanent friend-game release? The current
     accepted-risk conditions (QA-COND-0005 accessibility, QA-COND-0006 fun
     hypothesis) are incompatible with public release framing. Record the
     decision in `production/release/scope.md`.

4. **Generate release-tier artifacts.**
   - `/release-checklist` and/or `/launch-checklist` -> `production/release/`.
   - `/changelog` and `/patch-notes` -> `production/`.
   - `/balance-check` -> `production/qa/`.
   - `/perf-profile` against a release-build WASM bundle, measured against
     the <50 MB / 60 FPS budgets in `technical-preferences.md`.

5. **Close Foundation ADR gaps.**
   - Author the five pending ADRs (client-server authority, card data schema,
     round state machine, auction event flow, Sang Mepris reveal).

6. **Resolve open defects and ignored tests.**
   - Close S8-QA-001-W1 with a verdict (fix or explicit waiver).
   - Disposition the 11 D-5 ignored tests (fix / accept-risk per test / waive
     in batch via ADR).

7. **Final art or scope decision.**
   - Either commit to final art production for the assets currently tagged
     placeholder, OR formally accept friend-game-only release with a written
     scope statement.

8. **Run a fun-hypothesis playtest.**
   - At minimum one playtest closing QA-COND-0006 before any release gate retry.

Once items 1-4 exist as artifacts, re-run `/gate-check polish-release`.

---

## Chain-of-Verification (FAIL draft)

5 questions checked. Verdict **unchanged**.

| Question | Finding |
|---|---|
| Have I accurately separated hard blockers from strong recommendations? | Yes. All eight blockers are concrete file-existence checks or written-record contradictions, not subjective judgements. |
| Are there any PASS items I was too lenient about? | The smoke verdict (PASS WITH WARNINGS) is treated as FAIL for this gate, consistent with the gate's "clean PASS" requirement. No leniency applied. |
| Am I missing any additional blockers the user should know about? | One open question: localization strings not scanned this run (no committed i18n scope). Flagged as PARTIAL, not promoted to a blocker because the project has not committed to a localized release. |
| Can I provide a minimal path to PASS? | Yes -- 8 numbered recommendations above. Items 1-4 are the minimum to re-run the gate at all. |
| Is the fail condition resolvable, or does it indicate a deeper design problem? | Fully resolvable. The work is well-tracked. The verdict reflects scope discipline, not design failure. The project is healthy for its actual stage (Polish, friend-game-lite). |

---

## What This Gate Did NOT Do

- Did not run `/smoke-check`, `/team-qa`, `/story-done`, or `/dev-story`.
- Did not edit `production/sprint-status.yaml`.
- Did not modify `production/stage.txt` (verdict was not PASS).
- Did not write changelog or release artifacts; it identified their absence.

---

## Reference

- Smoke retry-7 (gate input): `production/qa/smoke-sprint-10-2026-05-12-retry-7.md` (commit `bc96700`)
- Sprint plan: `production/sprints/sprint-10.md` (status: active per `sprint-status.yaml`)
- Director-gate definitions: `.claude/docs/director-gates.md` (CD/TD/PR/AD-PHASE-GATE)
- Previous Production -> Polish gate: `production/gate-checks/gate-production-polish-sprint-6-2026-05-06.md` (PASS WITH CONDITIONS, friend-game scope)

---

## Final Verdict

**FAIL** -- Polish -> Release transition is **NOT** approved.

Stay in Polish. Address the eight numbered blockers above (or formally redefine
"Release" as friend-game-only and re-derive the artifact baseline accordingly).

`production/stage.txt` remains `Polish`.
