# Sprint 10 -- 2026-05-21 to 2026-06-03

> **Status**: Active (activated 2026-05-10, PROMPT 591).
> **Drafted**: 2026-05-09 by `/sprint-plan` (producer agent).
> **Activation**: Not active. Activation requires Sprint 9 close-out
> (S9-QA-001 closed via the in-flight `DRAFT_INITIAL` fix wave evidence
> consolidation, or explicit accepted-risk close-out per PROMPT 460).
> **Source-of-truth assumption**: `origin/main` post `d7211f1`
> (PROMPT 545/556 `CardPoolPlugin` + `KeywordPlugin` plugin-registration
> fix that unblocked DRAFT_INITIAL display).

Sprint 10 turns the freshly unblocked DRAFT_INITIAL flow into a verifiable
Polish slice by closing out the integrated-but-not-`/story-done` Presentation
Asset Wiring batch (PAW-002 to PAW-006), draining the test-fixtures and
plugin-registration tech debt that the DRAFT_INITIAL breakthrough surfaced,
and starting on the targeted visual-UI gap so the friend-game build is
visually playable rather than raw/unstyled.

This sprint is docs/planning only at draft time. It does not run
`/dev-story`, `/story-done`, `/smoke-check`, `/team-qa`, `/gate-check`,
implementation, or CI.

## Planning Notes

- Current stage is `Polish`. `production/stage.txt` reads `Polish`.
- Sprint 9 is currently `active` per `production/sprint-status.yaml`. 7 of 8
  Must Have stories are `done`. Only `S9-QA-001` remains `in-progress`,
  blocked on `MANUAL-FG-001` (human-operator two-client GAME_OVER GUI route).
- Sprint 10 assumes Sprint 9 closes by S9-QA-001 reaching either real manual
  evidence or accepted-risk friend-game-lite QA evidence per the PROMPT 459 +
  manual screenshots + PROMPT 460 hybrid plan recorded in
  `production/session-state/codex-orchestrator-state.md`.
- The `DRAFT_INITIAL` breakthrough (`d7211f1`, PROMPT 556) registered
  `CardPoolPlugin` + `KeywordPlugin` in `server/src/main.rs` and is the
  reason PAW-002 to PAW-006 are now ready for `/story-done` verification.
  All five PAW stories were integrated previously but were held pending
  DRAFT_INITIAL display confirmation.
- PR-SPRINT skipped — Lean mode. `production/review-mode.txt` is not
  present, so the sprint-plan workflow defaults to `lean`.
- No Sprint 10 QA plan exists. A Sprint 10 QA plan must be authored via
  `/qa-plan sprint` before any Production → Polish gate-check or sprint
  close-out claim.
- Sprint 10 must keep all carried Sprint 8 conditions and Sprint 9 no-claims
  intact. It does not claim public release readiness, broad Standard-tier
  accessibility completion, playtest/fun-hypothesis validation, full
  playable-client manual QA, or full game completion.

## Entry Conditions (must be true at activation)

- Sprint 9 is `closed` or `closed-with-conditions`.
- `S9-QA-001` is `done` via real manual evidence OR explicitly recorded as
  accepted-risk friend-game-lite QA evidence with `S8-QA-001-W1` disposition
  preserved (open or explicitly carried).
- `S9-AUDIO-001` worker commit `db7f1a9` is integrated to `main` or formally
  deferred into Sprint 10.
- DRAFT_INITIAL displays 9 cards on `main` (verified at `d7211f1` by user
  screenshot 2026-05-09); no regression has reintroduced the silent failure.
- `production/sprint-status.yaml` Sprint 9 row reads `closed` /
  `closed-with-conditions`.

If any entry condition fails, Sprint 10 does not activate; the producer
must revise scope before activation.

## Sprint Goal

Close out the integrated Presentation Asset Wiring batch with
`/story-done` verification, drain the test-fixture and plugin-registration
tech debt surfaced by the DRAFT_INITIAL breakthrough, and start the
targeted visual-UI chrome work needed for the friend-game build to look
playable rather than raw — without expanding into broad production, public
release readiness, broad accessibility completion, full playable-client
manual QA, playtest validation, or full game completion.

## Capacity

- Total workdays: 10
- Buffer (20%): 2 days reserved for integration friction, fixture
  cascade-fail recovery, and evidence capture
- Available: **8 effective planned days**
- Planned Must Have scope: **6.5 estimated days**
- Should Have scope is conditional and must not displace Must Have closure.

---

## Tasks

### Must Have (Critical Path)

| ID | Task | Agent/Owner | Est. Days | Dependencies | Acceptance Criteria |
|----|------|-------------|-----------|--------------|---------------------|
| S10-PAW-001 | PAW-002..PAW-006 `/story-done` close-out batch | UI/client programmer + orchestrator | 1.50 | DRAFT_INITIAL verified working on `main` (already at `d7211f1`); presentation-asset-wiring story files exist in `production/epics/presentation-asset-wiring/` | Each of PAW-002 (Hand UI card frames/badges), PAW-003 (Shop/Auction chrome), PAW-004 (HUD chrome), PAW-005 (board unit sprites — story file exists at `production/epics/presentation-asset-wiring/story-005-board-unit-sprites.md`), and PAW-006 (Lobby class portraits/slots) reaches `done` in `production/sprint-status.yaml` via `/story-done` against the integrated commit; each story-done verifies acceptance criteria from `docs/architecture/tr-registry.yaml` (TR-PAW-002..006); no `ImageNode` is used for board content (ADR-021); no inline asset path strings outside `asset_wiring.rs`; missing PAW-002/003/004/006 story files are authored before `/story-done` runs. |
| S10-TD-001 | Test-fixture cascade-fail repair (14 fixtures) | server gameplay programmer | 1.50 | `add_message` dedup Wave 1 + Wave 2 are integrated to `main` (already at `200d2d9` and `6f77d4b`); pattern established at `tests/integration/auction/pool_integration_test.rs` | All 14 partial-App test fixtures under `tests/integration/` add explicit `.add_message::<T>()` for each message type they consume; `cargo test -p server` passes without `Messages<T>` resource panics; no production code in `server/`, `client/`, or `shared/` is modified by this story (test-only changes); evidence document at `production/qa/evidence/sprint-10-test-fixture-repair.md` records each fixture file, the message types added, and the before/after pass count. |
| S10-TD-002 | Plugin-registration audit and dead-plugin sweep | server programmer + orchestrator | 0.75 | `d7211f1` (CardPoolPlugin + KeywordPlugin registration fix) is on `main` | Audit grep enumerates every `pub struct *Plugin` under `server/src/feature/*` and `server/src/core/*` and diffs it against `add_plugins(...)` calls in `server/src/main.rs`; any plugin defined but not registered is either added to `App` or formally documented as intentional dead code with a `#[allow(dead_code)]` and a comment pointing at the decision; the audit doc lives at `production/qa/evidence/sprint-10-plugin-registration-audit.md`; the same audit is run on `client/src/main.rs` for `client/src/*/plugin.rs` and the result documented; no silent dead-plugin paths remain in either binary. |
| S10-CARRY-001 | Sprint 9 carry-over consolidation | orchestrator + QA tester | 0.75 | Sprint 9 close-out complete; `S9-AUDIO-001` worker commit `db7f1a9` integration decision made | Any Sprint 9 work that did not reach `done` at Sprint 9 close (S9-AUDIO-001 if not yet integrated, or any deferred S9-QA-001 tail evidence) is either integrated and `/story-done`-d in Sprint 10, or formally deferred with a written reason; `production/sprint-status.yaml` Sprint 10 row reflects the carry decisions; no Sprint 9 condition is silently dropped. |
| S10-POLISH-001 | HUD visual chrome — timer + class figurines + RESOLUTION dim | UI/client programmer | 1.25 | S10-PAW-001 PAW-004 (HUD chrome) verified done; HUD plugin remains within ADR-021 presentation boundaries | `client/src/ui/hud/` HUD shows the phase timer bar with the wired sprite from PAW-004, opponent + own class figurines from PAW-004 sprites, and the RESOLUTION-phase dim/freeze overlay specified by `design/gdd/hud.md`; no client-side optimistic phase authority is added; existing `S2CPhaseChanged` drain remains the single source of phase truth; integration test asserts the dim overlay renders only while `Phase::Resolution`; manual evidence screenshot recorded at `production/qa/evidence/sprint-10-hud-chrome-evidence.md`. No broad Standard-tier accessibility completion is claimed. |
| S10-POLISH-002 | Shop/Auction panel chrome wiring | UI/client programmer | 1.25 | S10-PAW-001 PAW-003 (Shop/Auction chrome) verified done; SAU-007 settlement and SAU-008 reconnect work remains intact | `client/src/ui/shop_auction/` panels (slot wells, auction panel background, border ramp tiles, bid button chrome) consume the `asset_wiring.rs` constants wired by PAW-003; no inline asset paths or `ImageNode` usage for board content; the active friend-game route through DRAFT_SHOP, DRAFT_AUCTION, settlement, and post-auction DRAFT_SHOP visibly uses the wired chrome; integration test asserts panel root entities have non-default `ImageNode.image` after `OnEnter(ClientState::InSession)`; manual evidence screenshot recorded at `production/qa/evidence/sprint-10-shop-auction-chrome-evidence.md`. No claim of full asset approval or final visual polish. |

### Should Have

| ID | Task | Agent/Owner | Est. Days | Dependencies | Acceptance Criteria |
|----|------|-------------|-----------|--------------|---------------------|
| S10-POLISH-003 | Lobby visual chrome — class carousel + portraits + slot indicators | UI/client programmer | 1.00 | S10-PAW-001 PAW-006 (Lobby chrome) verified done; SAU-008 reconnect snapshot intact | Lobby UI shows class carousel portraits, player slot panel state visuals, and room code chip from PAW-006 wired sprites; existing class-confirm and re-ack behavior from `be8b37d` is preserved; no client-side authority over class lock; integration test asserts portrait `ImageNode.image` non-default for each `ClassId`; manual evidence at `production/qa/evidence/sprint-10-lobby-chrome-evidence.md`. |
| S10-TD-003 | Doc hygiene tech debt sweep | orchestrator + architect | 0.50 | None | `ADR-011` cross-references using `TR-NP-04` are corrected to `TR-NP-006` where applicable; Network Protocol Rule 7 gains an `ADR-011` breadcrumb; the `add_message` "systemic duplicate bug" entry in `codex-orchestrator-state.md` is corrected to reflect that `App::add_message` is idempotent in Bevy 0.18 (per `bevy_app-0.18.1/src/sub_app.rs:358`); evidence is the diff itself plus a one-paragraph note in `production/session-state/codex-orchestrator-state.md`. No protocol or architecture decision is changed. |
| ECO-004 | Kill and Objective Awards reward-loop polish | gameplay programmer | 1.00 | Pull only if S10 evidence shows a concrete reward-loop gameplay issue | Reward changes preserve current event contracts, avoid duplicate gold awards, land before interest snapshot, and do not expand into broad economy rebalance. Conditional backlog — do not displace Must Have. |

### Nice to Have

| ID | Task | Agent/Owner | Est. Days | Dependencies | Acceptance Criteria |
|----|------|-------------|-----------|--------------|---------------------|
| S10-N1 | Sprint 10 evidence index | orchestrator | 0.25 | Must Have evidence captured | One concise `production/qa/evidence/sprint-10-evidence-index.md` records each story's evidence path, build/commit, manual route status, no-claim language, and any deferred items. |
| S10-N2 | Friend-game route readability notes | UI programmer + orchestrator | 0.25 | Must Have evidence captured | Small captured rough-edge notes for HUD, hand, shop, auction, board readability are prioritised for later stories; fixes only proceed if they directly improve the active friend-game loop and do not expand into broad Standard-tier accessibility. |

---

## Required Sprint 10 Story Docs

`/sprint-plan` did not author new story files. Before `/dev-story` begins,
the following story files must exist or be created in a separate docs-only
prompt and pass `/story-readiness`:

| Planned ID | Required story file |
|------------|---------------------|
| S10-PAW-001 | `production/epics/presentation-asset-wiring/story-002-hand-ui-card-frames-badges.md` (NEW), `story-003-shop-auction-chrome.md` (NEW), `story-004-hud-chrome.md` (NEW), `story-005-board-unit-sprites.md` (EXISTS), `story-006-lobby-portraits-slots.md` (NEW) |
| S10-TD-001 | `production/epics/playable-client/story-009-test-fixture-cascade-fail-repair.md` (NEW) |
| S10-TD-002 | `production/epics/playable-client/story-010-plugin-registration-audit.md` (NEW) |
| S10-CARRY-001 | No new story file required — orchestrator updates `sprint-status.yaml` and references existing S9 story files |
| S10-POLISH-001 | `production/epics/hud/story-011-hud-visual-chrome-mvp.md` (NEW) |
| S10-POLISH-002 | `production/epics/shop-auction-ui/story-014-panel-chrome-mvp.md` (NEW) |
| S10-POLISH-003 | `production/epics/game-session-system/story-011-lobby-visual-chrome-mvp.md` (NEW) |
| S10-TD-003 | No new story file required — doc-only sweep |

Until those files exist and pass `/story-readiness`, the corresponding rows
in `production/sprint-status.yaml` are tracked as blocked by missing story
docs. The story-readiness package is a Sprint 10 prerequisite; it is not
counted against Must Have capacity.

## Carryover from Sprint 9

| Task | Reason | New Estimate |
|------|--------|--------------|
| S9-QA-001 close-out tail (if any) | If Sprint 9 closes with accepted-risk friend-game-lite evidence rather than full manual route, any documentation tail (carrying `S8-QA-001-W1` disposition forward) lands in S10-CARRY-001 | included in S10-CARRY-001 (0.75d) |
| S9-AUDIO-001 audio bootstrap + timer urgency cue | Worker commit `db7f1a9` was pushed but had not integrated to `main` at Sprint 9 close per orchestrator state. Decision: integrate-and-`/story-done` in S10-CARRY-001, or formally defer | included in S10-CARRY-001 (0.75d) |

## Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Sprint 9 entry condition (S9-QA-001 close) does not land before S10 activation | Medium | High | Hold S10 activation until Sprint 9 is `closed` or `closed-with-conditions`. Producer reverifies entry conditions before any `/dev-story` runs. |
| Test-fixture cascade-fail (S10-TD-001) reveals additional hidden fixtures beyond the 14 enumerated | Medium | Medium | Run the full `cargo test -p server` suite first, enumerate every panicking fixture, and treat the count as a Sprint 10 evidence artefact rather than a hard scope cap. |
| Plugin-registration audit (S10-TD-002) discovers further unregistered plugins requiring functional changes | Low | High | Treat any newly discovered unregistered plugin like the `CardPoolPlugin` + `KeywordPlugin` finding: register if behavior is required, document as accepted dead code if not, and surface the finding for separate-prompt review before scope expansion. |
| Visual chrome stories (S10-POLISH-001/002/003) expand into broad asset production | Medium | High | Constrain each to wiring already-approved sprites from `asset_wiring.rs`; no new asset authoring is in scope. New asset requests are caught by `/scope-check`. |
| Visual chrome work introduces client-side optimistic authority | Medium | High | Each polish story explicitly preserves the existing `S2CPhaseChanged` and economy view drains; integration tests assert no new local-source phase or economy mutation. |
| `QA-COND-0005` (Standard-tier accessibility) is misrepresented as closed | Medium | High | Polish stories preserve the accepted-risk language explicitly. Visual chrome wiring is friend-game-lite scope, not Standard-tier accessibility completion. |
| `QA-COND-0006` (playtest/fun-hypothesis validation) is misrepresented as closed | Medium | High | Sprint 10 evidence is friend-game internal evidence, never playtest validation. |
| `S8-QA-001-W1` is silently dropped if carry-over consolidation misses it | Medium | High | S10-CARRY-001 explicitly forces a written disposition for `S8-QA-001-W1`. The disposition does not change based on Sprint 10 work alone. |
| Sprint 10 expands beyond friend-game scope into public release readiness | Medium | High | Keep all six Must Have stories within friend-game friend-game-lite scope. Producer feasibility note (Phase 4) remains attached to this draft. |

## Dependencies on External Factors

- `production/main` source-of-truth must include `d7211f1` (DRAFT_INITIAL
  breakthrough) and the integrated PAW-002..PAW-006 commits before
  `/story-done` runs.
- A local server and two real primary clients can be run for visual evidence
  capture (browser/native).
- No new asset authoring depends on this sprint; PAW work consumes
  already-wired sprite paths defined in prior PAW stories' integration
  commits.

## Definition of Done for this Sprint

- [ ] All Must Have tasks completed and integrated.
- [ ] All Must Have tasks pass acceptance criteria.
- [ ] Sprint 10 QA plan exists at `production/qa/qa-plan-sprint-10.md` (this
      is a Sprint 10 prerequisite — author via `/qa-plan sprint` before
      `/dev-story` begins).
- [ ] All Logic/Integration stories have passing unit/integration tests.
- [ ] `cargo test -p server` and `cargo test -p client` pass without
      fixture cascade failures.
- [ ] `/smoke-check sprint` passed for Sprint 10 (or recorded as PASS WITH
      WARNINGS with documented warnings).
- [ ] `/team-qa sprint` produced an `APPROVED` or `APPROVED WITH
      CONDITIONS` sign-off report.
- [ ] No S1 or S2 bugs in delivered Must Have features.
- [ ] `production/sprint-status.yaml` reflects every Must Have story as
      `done` or explicitly `closed-with-conditions`.
- [ ] `S8-QA-001-W1`, `QA-COND-0005`, and `QA-COND-0006` retain their
      pre-Sprint-10 disposition unless separate actual closure evidence
      lands inside Sprint 10 scope.
- [ ] No public release readiness, full playable-client manual QA, full
      game completion, broad Standard-tier accessibility completion,
      playtest/fun-hypothesis validation, or full asset/content production
      is claimed.

## QA Plan

No Sprint 10 QA plan exists at draft time. The `/sprint-plan` workflow
flagged this as a Phase 5 gate. The producer-recommended path is:

> Run `/qa-plan sprint` after Sprint 10 story docs (above) exist and pass
> `/story-readiness`, and before `/dev-story` begins on any Must Have.

A Sprint 10 plan without a QA plan cannot pass the Production → Polish
gate-check.

## Scope-Creep Guard

This sprint pulls work only from already-named epics and existing Sprint 9
carry-over. New asset authoring, broad accessibility completion, public
release readiness, full playtest validation, full playable-client manual
QA, and full game completion are explicitly out of scope.

Run `/scope-check presentation-asset-wiring`, `/scope-check hud`,
`/scope-check shop-auction-ui`, and `/scope-check game-session-system`
before implementation begins to detect scope creep on the polish stories.

## Verification For Activation (deferred)

Activation will require, at minimum:

- `git diff --check` on the activation commit
- `git diff --cached --check` before commit
- Sprint 9 row in `production/sprint-status.yaml` reads `closed` or
  `closed-with-conditions`
- `production/sprint-status.yaml` Sprint 10 row written by `/sprint-plan`
  activation, not manually

This file is a draft. `/sprint-plan` did not write
`production/sprint-status.yaml` for Sprint 10 — that step is held until the
plan is locked and Sprint 9 is closed.
