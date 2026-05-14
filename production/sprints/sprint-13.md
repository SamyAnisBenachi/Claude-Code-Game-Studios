# Sprint 13 -- ACTIVATED (Polish stage)

> **PROMPT 833 `/story-done` (2026-05-14)**: First Sprint 13 row closed.
> Sprint 13 Should Have row `S11-SERVER-POOL-INIT-LOG-GUARD-001`
> (`production/epics/server/story-001-init-pool-log-guard.md`) flipped
> Draft -> Done on `origin/main@7983f5c` (worker `c6f6325` PROMPT 829;
> integration `7983f5c` PROMPT 832). W5 `ee27fb6` `acquisition_tick`
> pattern applied: server entry log downgraded `info!` -> `debug!`; new
> `info!` emitted only after the `DraftPhase::Initial` continue-guard.
> AC1-AC9 closed against evidence at
> `production/qa/evidence/sprint-13-init-pool-log-guard-evidence.md`
> (NEW on main via PROMPT 832). Targeted regression
> `cargo test -p server --lib` 98/0/0 at both worker and integration;
> full-workspace gate deferred to Sprint 13 end-of-sprint orchestrator
> integration per QA-plan-sprint-13. AC4 cold-path bound closes on
> static analysis; runtime smoke deferred to end-of-sprint integration
> smoke per QA-plan-sprint-13 serialization policy.
>
> Sprint 13 progress after PROMPT 833: **1 of 6 Should Have done; 0 of
> 6 Must Have done; 0 of 7 Nice to Have done.** Sprint 13 disposition
> UNCHANGED (`active`). Stage UNCHANGED (`Polish`). PROMPT 761
> Polish->Release gate-check `FAIL` preserved (no retry). S8-QA-001-W1
> OPEN preserved. QA-COND-0005 + QA-COND-0006 accepted-risk preserved.
> PAW-TD-*-a accept-risk preserved. PROMPT 683-era runtime divergence
> question preserved. TQ-S12-C1..C7 preserved verbatim. Sprint 13 is
> NOT closed-out by PROMPT 833. No `/smoke-check`, `/team-qa`,
> `/gate-check`, `/release-check`, `/dev-story`, `/story-readiness`,
> `/qa-plan` run by PROMPT 833. No `client/`, `server/`, `shared/`,
> `tests/` touched.

---

> **PROMPT 826 activation (2026-05-14)**: Sprint 13 flipped from DRAFT
> to ACTIVE. `production/sprint-status.yaml` top-level `sprint:` flipped
> 12 -> 13 and `status:` flipped `closed-with-conditions` -> `active`.
> `next_sprint:` draft block removed (now superseded by
> `sprint_13_activation:` block appended at end of file). `stories:`
> block content replaced with Sprint 13 row set (6 Must Have + 6 Should
> Have + 7 Nice to Have = 19 rows, all `ready`). Stage remains
> `Polish`. Sprint 12 disposition (`closed-with-conditions` per
> PROMPT 817) preserved unchanged under `sprint_12_closeout:` block.
> Sprint 11 / Sprint 10 closeouts preserved unchanged. PROMPT 761
> Polish->Release gate-check `FAIL` preserved at
> `production/gate-checks/gate-polish-release-2026-05-12.md` (no
> retry). TQ-S12-C1..C7 preserved verbatim. S8-QA-001-W1 OPEN
> preserved. QA-COND-0005 + QA-COND-0006 accepted-risk preserved.
> PAW-TD-*-a accept-risk preserved. PROMPT 683-era runtime divergence
> question preserved (folded into Sprint 12 story 019 cannot-reproduce
> closure). Story 019 underlying drag-runtime bug NOT claimed fixed.
> Sprint 13 QA plan does NOT exist at activation; must be authored via
> `/qa-plan sprint` before any `/dev-story`. PROMPT 826 paperwork-only
> activation: **NO** `/dev-story`, **NO** `/story-readiness` rerun,
> **NO** `/story-done`, **NO** `/smoke-check`, **NO** `/team-qa`,
> **NO** `/gate-check`, **NO** `/release-check`, **NO** `/qa-plan`,
> **NO** implementation, **NO** CI runs, **NO** cargo/trunk build or
> test runs were performed by this activation.
>
> **Status**: `active` (flipped from `draft` by PROMPT 826).
> **Source-of-truth at activation**: `origin/main@6fbbe86`
> (PROMPT 825 commit `paperwork(s13): refresh draft pointers to actual
> story paths (PROMPT 825)`).
> **Worktree**: `D:\_DEV\claude-code-game-studios-worktrees\sprint-13-activation`.
> **Branch**: `sprint-plan/sprint-13-activation`.
> **Start / end**: 2026-07-02 -> 2026-07-15 (10 workdays).
>
> **Release scope**: still explicitly OUT of Sprint 13. PROMPT 761
> Polish->Release gate-check `FAIL` evidence preserved at
> `production/gate-checks/gate-polish-release-2026-05-12.md`. Do **not**
> retry the Polish->Release gate-check until release-scope artifacts
> (final art, manual-QA sign-off, accessibility completion, playtest
> evidence) actually exist on `main`. Sprint 13 does **not** advance
> stage. Sprint 13 does **not** claim S8-QA-001-W1 closure.

---

> **PROMPT 825 pointer refresh (2026-05-14)**: Updated 11 story pointers
> from placeholder `story-XXX` slugs to actual story file paths now
> present on `main` (PROMPT 822 authored the missing files; PROMPT 823
> verified all 12 newly reviewed stories READY). Sprint 13 remains
> **DRAFT and NOT activated**. Stage remains `Polish`. No code change.
> No QA / smoke / gate-check / activation run by PROMPT 825.
>
> **Status**: `draft` -- authored 2026-05-14 by PROMPT 818
> (`/sprint-plan sprint-13` draft on worktree
> `D:\_DEV\claude-code-game-studios-worktrees\sprint-13-plan-draft` on
> branch `sprint-plan/sprint-13-draft`). Source-of-truth at draft:
> `origin/main@d09f9fe` (`docs(octogent): Codex orchestrator via
> app-server (Phase 4 of migration)`); the substantive Sprint 12
> close-out disposition lives one commit prior at `origin/main@c9d22f6`
> (PROMPT 817 commit `close-out(s12): Sprint 12 close-out disposition
> closed-with-conditions (PROMPT 817)`); the intervening
> `docs(octogent)` commit `d09f9fe` does not change Sprint 12 / Sprint
> 13 planning content. Stage remains `Polish`
> (NOT Release). Sprint 12 disposition preserved unchanged:
> `closed-with-conditions` per PROMPT 817. Sprint 11 disposition
> preserved unchanged: `closed-with-conditions` per PROMPT 792.
> Sprint 10 disposition preserved unchanged: `closed-with-conditions`
> per PROMPT 763. PROMPT 818 paperwork-only draft: **NO** `/dev-story`,
> **NO** `/story-readiness`, **NO** `/story-done`, **NO** `/smoke-check`,
> **NO** `/team-qa`, **NO** `/gate-check`, **NO** `/release-check`,
> **NO** `/qa-plan`, **NO** implementation, **NO** CI runs, **NO**
> cargo/trunk build or test runs were performed by this draft.
> **Sprint 13 is NOT activated** by this draft; activation happens via
> `/sprint-plan sprint-13` in a separate prompt after this draft is
> reviewed.
>
> **Start / end (provisional; locked at activation)**: 2026-07-02 ->
> 2026-07-15 (10 workdays). Continuous follow-on to Sprint 12
> (2026-06-18 -> 2026-07-01).
>
> **Release scope**: explicitly OUT of Sprint 13. PROMPT 761
> Polish->Release gate-check `FAIL` evidence remains preserved at
> `production/gate-checks/gate-polish-release-2026-05-12.md`. Do **not**
> retry the Polish->Release gate-check until release-scope artifacts
> (final art, manual-QA sign-off, accessibility completion, playtest
> evidence) actually exist on `main`. Sprint 13 does **not** advance
> stage. Sprint 13 does **not** claim S8-QA-001-W1 closure.

## Planning Notes

- Current stage is `Polish`. `production/stage.txt` reads `Polish`.
  Sprint 13 does NOT advance stage.
- Sprint 12 is `closed-with-conditions` (PROMPT 817); 5/5 Must Have
  done; 0/4 Should Have done; 0/5 Nice to Have done. All 4 Should Have
  rows and all 5 Nice to Have rows were explicitly deferred forward
  to Sprint 13 planning by the Sprint 12 close-out (see
  `sprint_12_closeout.deferred_into_sprint_13_planning` in
  `production/sprint-status.yaml`). This draft pulls those deferrals
  plus the 8 PROMPT 804 / PROMPT 808 Sprint 13 candidate
  runtime-hardening stories already authored on `main` under
  `production/epics/lightyear-protocol-verification/` and
  `production/epics/playable-client/`, plus a devops candidate for
  the Windows AppCompat smoke-warning workaround (TQ-S12-C7
  informational), plus a small UI-layout audit-roadmap planning row
  drawn from PROMPT 802 (without attempting a full UI overhaul in
  one sprint).
- This draft pulls candidates from:
  - Sprint 12 close-out deferred items (4 Should Have rows + 5 Nice
    to Have rows from
    `sprint_12_closeout.deferred_into_sprint_13_planning`).
  - The 8 PROMPT 804 Sprint 13 candidate runtime-hardening story
    files authored on `main` at commit `55b25be` (PROMPT 808
    integration): `story-007-protocol-completeness-invariant.md`,
    `story-008-protocol-orphan-drain.md`, `story-016-fixture-factory.md`,
    `story-017-two-client-runtime-harness.md`,
    `story-018-obs-tracing-targets.md`,
    `story-019-obs-wallclock-timestamps.md`,
    `story-020-late-msg-dedupe.md`,
    `story-021-conn-lost-ux.md`.
  - PROMPT 803 Multiplayer Runtime Hardening Audit roadmap §3 + §5
    Must rows (the source of the 8 PROMPT 804 stories above).
  - PROMPT 802 Expert UI Layout Audit roadmap (scoped to audit/story-
    authoring prep only -- no full UI overhaul in this sprint).
  - Sprint 12 sub-prompt PROMPT 815 disk-pressure invocation +
    PROMPT 815/816/817 Windows AppCompat warning (TQ-S12-C7) as
    devops candidate input.
  - Wider Sprint 12 backlog "not yet pulled" list
    (`production/sprints/sprint-12.md` historical DRAFT body).
- PR-SPRINT skipped -- Lean mode (no `production/review-mode.txt`).
- No Sprint 13 QA plan exists at draft time. A Sprint 13 QA plan must
  be authored via `/qa-plan sprint` after Sprint 13 story files exist
  for the new candidates and pass `/story-readiness`, and before any
  Polish gate-check or sprint close-out claim for Sprint 13.
- Sprint 13 explicitly does NOT claim public release readiness,
  release-candidate readiness, full game completion, broad
  Standard-tier accessibility completion, full playable-client manual
  QA, playtest / fun-hypothesis validation, final-art /
  asset-production completion, `S8-QA-001-W1` closure, or a
  Polish->Release retry. None of these can be added to Sprint 13 by
  activation; they require their own scope and gate evidence.

## Entry Conditions (must be true at activation)

- Sprint 12 row in `production/sprint-status.yaml` reads
  `closed-with-conditions` (already true at draft time per PROMPT
  817).
- `production/stage.txt` still reads `Polish`.
- PROMPT 761 Polish->Release gate-check `FAIL` evidence is preserved
  at `production/gate-checks/gate-polish-release-2026-05-12.md`.
- `S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a`
  dispositions intact.
- Sprint 12 disposition (`closed-with-conditions` per PROMPT 817) is
  preserved unchanged.
- Sprint 13 story files referenced below have either been authored
  and passed `/story-readiness`, or are explicitly held with a written
  blocker.
- The 8 PROMPT 804 candidate stories already exist on `main` (at
  `production/epics/lightyear-protocol-verification/story-007*.md`,
  `story-008*.md`, and `production/epics/playable-client/story-016*.md`
  through `story-021*.md`); each still requires its own
  `/story-readiness` pass before `/dev-story` against it.

If any entry condition fails, Sprint 13 does NOT activate; producer
must revise scope before activation.

## Sprint Goal

Sprint 13 is a **runtime hardening + Sprint 12 cleanup + UI-layout
audit-roadmap-prep** sprint, NOT a release sprint. The goal is:

1. Harden the multiplayer runtime per the PROMPT 803 audit by landing
   protocol-completeness invariants (story 007), draining or pruning
   the 8 S2C orphans + 1 C2S orphan (story 008), authoring a
   production-faithful test app factory (story 016), authoring a
   non-interactive two-client runtime harness ready for Story 019
   tighter-capture rerun and for S8-QA-001-W1 evidence (story 017),
   adding module-path-scoped tracing targets so the Story 019
   diagnostic invocation actually captures something (story 018), and
   adding wall-clock ISO-8601 timestamps to every tracing subscriber
   (story 019).
2. Drain Sprint 12 deferred Should Have rows: HUD timer eyeball
   visual check, client `phase_changed=true` 60Hz idempotency,
   server `init_pool` log emits before guard, lobby "Confirming..."
   text differentiation.
3. Drain Sprint 12 deferred Nice to Have rows: Cargo workspace
   disk-usage reduction strategy note (re-affirmed by the PROMPT 815
   disk-pressure cleanup), Cargo PDB-size pressure investigation
   note, orchestrator-root concurrent-session lock pattern doc,
   `gh` CLI installation note, intermittent R2 Placement runtime
   crash audit doc.
4. Land a small devops candidate for the Windows AppCompat
   smoke-warning workaround at `docs/setup/dev-environment.md`
   (TQ-S12-C7 informational).
5. Land an audit/story-authoring-only roadmap row for the PROMPT 802
   Expert UI Layout audit (UI clean-pass). **Sprint 13 does NOT
   attempt the full UI overhaul**; it produces a sequenced story
   index for Sprint 14+ pull-in.

Sprint 13 does not claim release readiness, broad accessibility
completion, full playable-client manual QA, playtest validation,
final-art / asset-production completion, S8-QA-001-W1 closure, full
game completion, two-client GAME_OVER closure, or a Polish->Release
retry.

## Capacity (provisional)

- Total workdays: 10 (assumes 2-week sprint same as Sprint 10/11/12)
- Buffer (20%): 2 days reserved for runtime-hardening integration
  friction (story 008 protocol orphan-drain may surface product
  decisions per PROMPT 803 §9), two-client harness friction (story
  017 manual-QA prep), fixture-factory migration (story 016 partial
  parallel-safety), and devops onboarding for the AppCompat workaround
- Available: **8 effective planned days**
- Planned Must Have scope: **~5.50 estimated days** (4 PROMPT 804
  Must-tier hardening rows + Sprint 12 deferred Should Have rows)
- Should Have scope is conditional and must not displace Must Have
  closure.
- Nice to Have scope is documentation-tier and lands only when
  Should Have closure is on track.

---

## Tasks

> All IDs below are **draft S13-* / S11-*** tickets. They are NOT yet
> active `sprint-status.yaml` rows. Slugs prefixed `S13-` are net-new
> hardening / observability candidates surfaced in PROMPT 803 and
> already authored as story files by PROMPT 804 / PROMPT 808 on `main`.
> Slugs prefixed `S11-` are carried forward unchanged from Sprint 11 /
> Sprint 12 close-out deferrals to preserve traceability (e.g.,
> evidence cross-links in
> `sprint_12_closeout.deferred_into_sprint_13_planning`). Slugs
> prefixed `S13-OPS-` or `S13-UI-` are net-new devops or UI-prep rows
> for Sprint 13. Promotion to active rows happens at activation via
> `/sprint-plan sprint-13`.

### Must Have (Critical Path)

| ID | Task | Agent/Owner | Est. Days | Source | Acceptance Criteria (draft) |
|----|------|-------------|-----------|--------|------------------------------|
| S13-PROTO-INVARIANT-001 | Protocol completeness invariant test (every defined C2S/S2C has >=1 send-site and >=1 drain-site) -- gates DC-1/DC-2 recurrences | qa-lead + network-programmer | 0.75 | PROMPT 803 §3 DC-1 + DC-15, §4 Lane A, §5 Must row 1; story file already authored at `production/epics/lightyear-protocol-verification/story-007-protocol-completeness-invariant.md` (PROMPT 804 worker `d334499`, integrated by PROMPT 808 as `55b25be`) | Story 007 `/story-readiness` passes against Sprint 13 activation HEAD. Invariant test at `tests/invariants/protocol_completeness_test.rs` (NEW) introduces a workspace-level cross-reference of every C2S/S2C in `shared/src/protocol.rs:60-110` against MessageSender / MessageReceiver call sites in `client/src/network/` and `server/src/network/`. Test is allowed to be `#[ignore]`d behind a feature gate ONLY if Story 008 is sequenced same-wave; otherwise test must pass under default `cargo test --workspace`. ADR-002 + ADR-008 + ADR-009 binding (no client-side optimistic state added by this invariant test). |
| S13-PROTO-ORPHAN-DRAIN-001 | Add `MessageReceiver` / `MessageSender` for the 8 S2C orphans + the 1 C2S orphan (or delete them from the protocol with rationale per row) | network-programmer + technical-director | 1.50 | PROMPT 803 §3 DC-1, §4 Lane A (8 S2C orphans + 1 C2S orphan), §5 Must row 2, §9 product-decision #3/#4; story file already authored at `production/epics/lightyear-protocol-verification/story-008-protocol-orphan-drain.md` | Story 008 `/story-readiness` passes. Per-row disposition recorded in story file BEFORE code change (drain-or-delete-with-rationale) for: `C2SRequestSnapshot`, `S2CHeartbeat`, `S2COpponentDisconnected`, `S2COpponentReconnected`, `S2CPoolUpdate`, `S2CPrismRespawned`, `S2CPrismRewardDropped`, `S2CSangMepriseReveal`, `S2CSessionCancelled`. Each disposition cites file:line evidence from PROMPT 803 §4. Story 007 invariant test passes after Story 008 lands. No optimistic client-side authority added (ADR-002 binding). |
| S13-FIXTURE-FACTORY-001 | Canonical production-faithful test app factory at `tests/helpers/production_app_factory.rs` (NEW); migrate B1 / B2 / lobby_app / shop_app onto it | qa-lead + test-infra owner | 1.00 | PROMPT 803 §3 DC-7 + DC-8, §4 Lane D, §5 Must row 5; story file already authored at `production/epics/playable-client/story-016-fixture-factory.md` | Story 016 `/story-readiness` passes. Factory mirrors `client::main` / `server::main` plugin sets and is wired into B1 (`tests/integration/board_rendering/ghost_preview_bridge_test.rs`), B2 (`tests/integration/board_rendering/snapshot_spawn_test.rs`), `lobby_app()` and `shop_app()` (in `tests/integration/playable_client/native_operator_controls_test.rs`). Tests still pass post-migration; assertions remain semantically equivalent. **No production-source diff under `client/src/` / `server/src/` / `shared/src/`** -- pure test infra. Decision-first on whether to migrate B5 in the same wave or defer to Sprint 14. |
| S13-TWO-CLIENT-RUNTIME-HARNESS-001 | Non-interactive scripted two-client harness (cargo bin) that drives the full friend-game route end-to-end against the real server, with structured log capture | network-programmer + devops-engineer + qa-lead | 1.25 | PROMPT 803 §3 DC-14, §4 Lane E, §5 Must row 6; story file already authored at `production/epics/playable-client/story-017-two-client-runtime-harness.md` | Story 017 `/story-readiness` passes. New Cargo workspace member under `tools/two-client-runtime/` (NEW) compiles and runs deterministically against the existing `server` binary. Produces snapshot + log evidence under `production/qa/evidence/captures/sprint-13-two-client-harness/`. **AC12 forbid-auto-closure**: story 017 explicitly does NOT close `S8-QA-001-W1`; closure remains gated on a separate `/story-done` prompt with qa-lead sign-off in a later sprint. No release-readiness claimed by story 017. |
| S13-OBS-TRACING-TARGETS-001 | Add `target: "client::ui::hand"`, `target: "client::presentation::board_rendering"`, `target: "client::card_animations::input_gating"`, `target: "server::game"` to all relevant emission sites so the Story 019 `RUST_LOG` invocation actually captures something | observability + each module owner | 0.75 | PROMPT 803 §3 DC-11, §5 Must row 7; story file already authored at `production/epics/playable-client/story-018-obs-tracing-targets.md` | Story 018 `/story-readiness` passes. Verification by re-running the Story 019 invocation pattern (`RUST_LOG=client::ui::hand=trace,client::presentation::board_rendering=trace,client::card_animations::input_gating=info,lightyear=debug,server::game=debug`) against a smoke test and asserting non-empty per-target output. Does NOT itself re-attempt Story 019 capture (no third same-scope retest authorised per TQ-S12-C2). Story 018 may collide on `client/src/presentation/board_rendering.rs` with Sprint 12 Story 014 history -- worker must rebase / re-check on `main` at Sprint 13 activation HEAD. |
| S13-OBS-WALLCLOCK-TIMESTAMPS-001 | Configure `tracing_subscriber::fmt().with_timer(UtcTime::rfc_3339())` (or equivalent) in server, client, and tests so multi-process logs can be aligned at ms precision | observability + devops-engineer | 0.25 | PROMPT 803 §3 DC-12, §5 Must row 8; story file already authored at `production/epics/playable-client/story-019-obs-wallclock-timestamps.md` | Story 019 (Sprint 13 candidate, NOT Sprint 12 story 019) `/story-readiness` passes. Spot check by running `server` + two clients and asserting ISO-8601 UTC prefix on every emitted line. Three files touched: `server/src/main.rs:87`, `client/src/main.rs:36`, `tests/test_helpers.rs:52`. Parallel-safe with the other Sprint 13 Must rows. |

### Should Have

| ID | Task | Agent/Owner | Est. Days | Source | Acceptance Criteria (draft) |
|----|------|-------------|-----------|--------|------------------------------|
| S11-HUD-TIMER-EYEBALL-VISUAL-001 | HUD timer eyeball visual check (W2 carry from Sprint 10 smoke retry-7) | UI programmer | 0.25 | Sprint 12 close-out deferral (Should Have, `blocked`; story file authored by PROMPT 822, /story-readiness READY per PROMPT 823); originally a Sprint 10 W2 carry | Story file exists at `production/epics/hud/story-014-hud-timer-eyeball-visual-check.md` and passes `/story-readiness` (PROMPT 822 / PROMPT 823). Manual 2-client run validates timer countdown renders correctly for `DraftInitial` 45s, `DraftShop` 30s, `Placement` 10-12s phases. Evidence: screenshot capture in `production/qa/evidence/sprint-13-hud-timer-visual-check/` (NEW). Cosmetic verification only; no production-code change unless an actual visual regression is found and a follow-on story is authored. Does NOT claim Standard-tier accessibility completion. |
| S11-HU-PHASE-IDEMPOTENCY-001 | Client `phase_changed=true` 60Hz idempotency | client gameplay programmer | 0.75 | Sprint 12 close-out deferral (Should Have, `blocked`; story file authored by PROMPT 822, /story-readiness READY per PROMPT 823); PROMPT 803 §3 DC-5 same-class candidate | Story file exists at `production/epics/playable-client/story-022-client-phase-changed-idempotency.md` and passes `/story-readiness` (PROMPT 822 / PROMPT 823). Spurious `phase_changed=true` on every frame reduced to actual phase transitions only. Existing `S2CPhaseChanged` drain remains the single source of phase truth. Integration test asserts no `phase_changed=true` outside actual phase transition frames. **No client-side optimistic phase authority added** (ADR-002 + ADR-009 binding). |
| S11-SERVER-POOL-INIT-LOG-GUARD-001 **(DONE 2026-05-14 PROMPT 833)** | Server `init_pool` log emits before guard | server gameplay programmer | 0.25 | Sprint 12 close-out deferral (Should Have, `blocked`; story file authored by PROMPT 822, /story-readiness READY per PROMPT 823); Wave 12 backlog parallel to `ee27fb6` acquisition_tick fix | Story file at `production/epics/server/story-001-init-pool-log-guard.md` flipped Status Draft -> Done by PROMPT 833 on `origin/main@7983f5c` (worker `c6f6325` PROMPT 829; integration `7983f5c` PROMPT 832). W5 `ee27fb6` pattern applied: entry log downgraded `info!` -> `debug!`; new `info!` emitted only after `DraftPhase::Initial` continue-guard. `cargo test -p server --lib` 98/0/0 at worker + integration. Evidence: `production/qa/evidence/sprint-13-init-pool-log-guard-evidence.md`. AC4 cold-path bound closes on static analysis (N_info <= session restarts << 50); runtime smoke <50 emitted-line confirmation deferred to Sprint 13 end-of-sprint integration smoke per QA-plan-sprint-13 serialization policy. |
| S11-LOBBY-UX-CONFIRM-STATE-001 | Lobby "Confirming..." text differentiation (own-confirm-acked vs waiting-opponent) | UI programmer + ux-designer | 0.50 | Sprint 12 close-out deferral (Should Have, `blocked`; story file authored by PROMPT 822, /story-readiness READY per PROMPT 823); Sprint 11 promotion from Nice to Have to batch with Sprint 12 Cluster B3 lobby work (story 013 already landed) | Story file exists at `production/epics/playable-client/story-023-lobby-confirm-state.md` and passes `/story-readiness` (PROMPT 822 / PROMPT 823). Lobby UI text distinguishes the two states. **No client-side class-lock authority added** (ADR-002 binding; reinforced by Sprint 12 story 013 fallback path that preserved ADR-002 + ADR-008 + ADR-012). Integration test asserts text differentiation across the two states. |
| S13-LATE-MSG-DEDUPE-001 | Add `(round, message-id)` dedupe set on client drains for `S2CGameOver`, `S2CClassLocked`, `S2CPlaceUnit` so duplicate reliable redelivery is idempotent | client gameplay programmer + network-programmer | 0.75 | PROMPT 803 §3 DC-6, §5 Should row 1; story file already authored at `production/epics/playable-client/story-020-late-msg-dedupe.md` | Story 020 `/story-readiness` passes. Late-message matrix test (NEW) asserts duplicate redelivery is dedupe-guarded for the three S2C surfaces. ADR-002 + ADR-009 binding. Sequences AFTER Sprint 12 close-out per story-020 dependency notes (file-scope conflict on `client/src/ui/lobby.rs` historically with Sprint 12 story 013, `client/src/presentation/board_rendering.rs` historically with Sprint 12 story 014; both now closed). |
| S13-CONN-LOST-UX-001 | Proactive "Reconnecting..." / "Connection Lost" overlay between transport drop and reconnect-window-expiry; closes backlog `S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001` | UI programmer + ux-designer | 1.00 | PROMPT 803 §3 DC-13, §5 Should row 2; story file already authored at `production/epics/playable-client/story-021-conn-lost-ux.md` | Story 021 `/story-readiness` passes. New overlay module under `client/src/presentation/` (NEW). Manual smoke evidence in `production/qa/evidence/sprint-13-conn-lost-ux/`. Sequences AFTER Sprint 12 close-out per story-021 dependency note on `client/src/presentation/mod.rs` (historically Sprint 12 story 012; now closed). Does NOT claim full S8-QA-001-W1 closure; sets up the visible-UX building block. |

### Nice to Have

| ID | Task | Agent/Owner | Est. Days | Source | Acceptance Criteria (draft) |
|----|------|-------------|-----------|--------|------------------------------|
| S11-TD-CARGO-DISK-USAGE-001 | Cargo workspace disk-usage reduction strategy (investigation note only) | devops-engineer | 0.50 | Sprint 12 close-out deferral (Nice to Have, `blocked`; story file authored by PROMPT 822, /story-readiness READY per PROMPT 823); re-affirmed by PROMPT 815 disk-pressure invocation (cleaned 25 GB + ~200 GB worker `target/` directories) | Story file exists at `production/epics/devops/story-001-cargo-workspace-disk-usage.md` and passes `/story-readiness` (PROMPT 822 / PROMPT 823). Investigation note at `docs/architecture/cargo-workspace-disk-usage.md` (NEW) documents current `target/` footprint per worktree, identifies trim candidates (shared target dir, prune debug symbols, sccache, etc.), recommends a single change to land in a follow-on story. **No build-script changes land in this story.** |
| S11-TD-CARGO-PDB-LIMIT-001 | Cargo PDB-size pressure investigation (no profile changes) | devops-engineer | 0.25 | Sprint 12 close-out deferral (Nice to Have, `blocked`; story file authored by PROMPT 822, /story-readiness READY per PROMPT 823); Wave 12 backlog | Story file exists at `production/epics/devops/story-002-cargo-pdb-limit.md` and passes `/story-readiness` (PROMPT 822 / PROMPT 823). Document PDB-size impact on disk usage and CI runtime. Recommend Windows-side `split-debuginfo` / `strip` profile knobs for `[profile.dev]` or `[profile.test]`. **No profile changes land in this story.** |
| S11-OPS-ORCHESTRATOR-LOCK-001 | Orchestrator-root concurrent-session lock pattern (documented only) | orchestrator | 0.25 | Sprint 12 close-out deferral (Nice to Have, `blocked`; story file authored by PROMPT 822, /story-readiness READY per PROMPT 823); Wave 12 backlog (2x sessions mutating main HEAD concurrently); reinforced by 2026-05-13 override rule "only one shared-status writer at a time" | Story file exists at `production/epics/devops/story-003-orchestrator-lock.md` and passes `/story-readiness` (PROMPT 822 / PROMPT 823). Lock-file or convention documented at `.octogent/orchestrator-lock.md` (or appended to existing orchestrator docs) describing how to detect / avoid concurrent root-checkout writes. **No code lands; pattern is documented only.** |
| S11-OPS-GH-CLI-001 | `gh` CLI installation note for dev machine | orchestrator + devops-engineer | 0.10 | Sprint 12 close-out deferral (Nice to Have, `blocked`; story file authored by PROMPT 822, /story-readiness READY per PROMPT 823); Wave 12 backlog (`gh` absent 3+ times) | Story file exists at `production/epics/devops/story-004-gh-cli-setup.md` and passes `/story-readiness` (PROMPT 822 / PROMPT 823). One paragraph in repo onboarding doc (or `docs/setup/dev-environment.md`) names `gh` as required, with install command. **No tooling changes land.** |
| S11-SERVER-R2-PLACEMENT-CRASH-AUDIT-001 | Intermittent R2 Placement runtime crash audit (no fix lands) | server gameplay programmer | 0.50 | Sprint 12 close-out deferral (Nice to Have, `blocked`; story file authored by PROMPT 822, /story-readiness READY per PROMPT 823); Wave 12 backlog (12:07 capture; not reproduced 13:28) | Story file exists at `production/epics/server/story-002-r2-placement-crash-audit.md` and passes `/story-readiness` (PROMPT 822 / PROMPT 823). Audit log emits enriched diagnostics around `Phase::Placement` round-2 transition. If a repro is captured during Sprint 13, a follow-on story is authored with the precise repro. **No fix is implemented in this story.** |
| S13-OPS-WIN-APPCOMPAT-NOTE-001 | Windows AppCompat heuristic + manifest/rename workaround note at `docs/setup/dev-environment.md` (informational from TQ-S12-C7) | devops-engineer | 0.25 | TQ-S12-C7 informational; PROMPT 815 / PROMPT 816 / PROMPT 817 evidence pointing to the substring `update` in `spawn_range_live_update_contract-*.exe` triggering the Windows installer-detection heuristic | Story file exists at `production/epics/devops/story-005-win-appcompat-note.md` and passes `/story-readiness` (PROMPT 822 / PROMPT 823). One paragraph note at `docs/setup/dev-environment.md` documenting the AppCompat heuristic and either (a) the binary-rename workaround used during PROMPT 815, or (b) a small embedded manifest (`level="asInvoker"`) decision documented for a follow-on story. **No production-source change lands**; doc-only. **NOT a Sprint 12 close-out blocker** (already accepted-risk per TQ-S12-C7); landing here is purely informational so the next smoke check is not surprised by the same warning. |
| S13-UI-AUDIT-ROADMAP-PREP-001 | PROMPT 802 Expert UI Layout audit-roadmap-prep (story index for Sprint 14+ pull-in; **no UI overhaul attempted in Sprint 13**) | producer + ux-designer | 0.50 | PROMPT 802 Expert UI Layout audit (§3 per-surface verdicts; §6 sequenced repair plan; §11 backlog-vs-recommendation matrix) | Story file exists at `production/epics/ui-clean-pass/story-001-prompt-802-audit-roadmap-prep.md` and passes `/story-readiness` (PROMPT 822 / PROMPT 823). Story authors an audit-roadmap-prep note at `docs/ux/ui-clean-pass-roadmap.md` (NEW) that (a) reconciles the 14 PROMPT 802 candidate slugs against the existing PROMPT 685 8-story milestone backlog, (b) sequences them into the right Sprint 14+ pull-in order, and (c) names the 3-4 highest-impact "must land before any polished friend-game-product showcase" rows for Sprint 14 Must Have framing. **Sprint 13 does NOT activate any of the 14 PROMPT 802 candidate slugs**; this is paperwork only. Existing accepted-risk for `PAW-TD-*-a` placeholder PNGs, `QA-COND-0005` Standard-tier accessibility, and `QA-COND-0006` playtest validation remain preserved. |

---

## Carryover from Sprint 12

| Source row (Sprint 12) | Disposition into Sprint 13 |
|------------------------|----------------------------|
| `S11-HUD-TIMER-EYEBALL-VISUAL-001` (Sprint 12 Should Have, `blocked`; W2 carry from Sprint 10 smoke retry-7) | Pulled forward as Sprint 13 Should Have. |
| `S11-HU-PHASE-IDEMPOTENCY-001` (Sprint 12 Should Have, `blocked`) | Pulled forward as Sprint 13 Should Have. PROMPT 803 §3 DC-5 reinforces the same problem class; landing here also satisfies that audit row. |
| `S11-SERVER-POOL-INIT-LOG-GUARD-001` (Sprint 12 Should Have, `blocked`) | Pulled forward as Sprint 13 Should Have. |
| `S11-LOBBY-UX-CONFIRM-STATE-001` (Sprint 12 Should Have, `blocked`) | Pulled forward as Sprint 13 Should Have (no further re-promotion needed). |
| `S11-TD-CARGO-DISK-USAGE-001` (Sprint 12 Nice to Have, `blocked`) | Pulled forward as Sprint 13 Nice to Have. Re-affirmed by PROMPT 815 disk-pressure cleanup of 25 GB + ~200 GB worker `target/` directories. |
| `S11-TD-CARGO-PDB-LIMIT-001` (Sprint 12 Nice to Have, `blocked`) | Pulled forward as Sprint 13 Nice to Have. |
| `S11-OPS-ORCHESTRATOR-LOCK-001` (Sprint 12 Nice to Have, `blocked`) | Pulled forward as Sprint 13 Nice to Have. |
| `S11-OPS-GH-CLI-001` (Sprint 12 Nice to Have, `blocked`) | Pulled forward as Sprint 13 Nice to Have. |
| `S11-SERVER-R2-PLACEMENT-CRASH-AUDIT-001` (Sprint 12 Nice to Have, `blocked`) | Pulled forward as Sprint 13 Nice to Have. |
| Sprint 13 candidate runtime-hardening stories per PROMPT 804 mapping (8 stories) | Pulled forward as Sprint 13 Must Have (stories 007/008/016/017/018/019) and Sprint 13 Should Have (stories 020/021). All 8 story files already exist on `main` at PROMPT 808 integration commit `55b25be`. Each still requires its own `/story-readiness` pass before `/dev-story`. |
| Sprint 13 devops candidate for Windows AppCompat (TQ-S12-C7 informational) | Pulled forward as Sprint 13 Nice to Have `S13-OPS-WIN-APPCOMPAT-NOTE-001`. |

## Conditions Carried Forward Unchanged (NOT closed by Sprint 13)

Sprint 13 explicitly preserves and does NOT claim closure for any of:

- **`S8-QA-001-W1`** -- manual / browser two-client GAME_OVER gap
  remains OPEN. Story 017 (two-client runtime harness) **explicitly
  does NOT close** `S8-QA-001-W1` by itself (per its `AC12
  forbid-auto-closure` in `production/epics/playable-client/story-017-two-client-runtime-harness.md`).
- **`QA-COND-0005`** -- Standard-tier accessibility remains
  accepted-risk (friend-game scope only); Sprint 13 does NOT pursue
  Standard-tier accessibility completion.
- **`QA-COND-0006`** -- playtest / fun-hypothesis validation remains
  accepted-risk / deferred; Sprint 13 does NOT pursue playtest
  evidence.
- **Placeholder / friend-game art scope** -- `PAW-TD-*-a`
  accept-risk on placeholder PNGs across PAW-002..PAW-006 remains in
  place; no final-art / asset-production completion is pursued.
  Sprint 13 UI-audit-roadmap-prep row (`S13-UI-AUDIT-ROADMAP-PREP-001`)
  is paperwork-only and does NOT activate or implement any UI repair
  story.
- **PROMPT 683-era runtime divergence question** -- folded into
  Sprint 12 story 019 (`closed-with-conditions / cannot-reproduce`
  after second time-box exhaustion). Sprint 13 does NOT claim this
  question closed; it provides expanded tracing instrumentation (story
  018 + story 019 (S13 candidate) + story 016 + story 017) so a
  **third same-scope retest is not authorised** per TQ-S12-C2.
- **PROMPT 761 Polish->Release gate-check FAIL** -- preserved at
  `production/gate-checks/gate-polish-release-2026-05-12.md`; **NO
  retry** is in scope for Sprint 13.
- **Story 019 underlying drag-runtime bug** -- NOT claimed fixed by
  Sprint 13. The Sprint 13 runtime-hardening row set (stories
  017/018/019(S13)) makes the diagnostic invocation actually
  productive but does NOT claim to have reproduced or fixed the
  underlying behaviour.
- **TQ-S12-C1..C7** -- all 7 Sprint 12 Team-QA conditions preserved
  verbatim (per `sprint_12_closeout.conditions_carried_forward_unchanged`
  in `production/sprint-status.yaml`). TQ-S12-C2 in particular: no
  third same-scope retest of Sprint 12 story 019 is authorised.

If any condition above changes during Sprint 13, it requires its own
separate story file and explicit disposition -- it cannot be silently
folded into another story.

## Wider Sprint 13 Backlog (not yet pulled into this draft)

The following candidates remain in the broader backlog and are **NOT
scheduled** into this Sprint 13 draft. They may be pulled by a
producer revision before activation, or deferred to Sprint 14:

- PROMPT 803 §5 Should-tier rows beyond `S13-LATE-MSG-DEDUPE-001` +
  `S13-CONN-LOST-UX-001`:
  - `S13-LOBBY-CONFIRMCLASS-SENDER-001` (DC-10; sequences after
    Sprint 12 story 013 lobby work landed at commit `d8d0196`).
  - `S13-COOCCUPANCY-INVARIANT-001` (DC-9; rationale-doc fold candidate
    against Sprint 12 story 014 history).
  - `S13-PHASE-IDEMPOTENCY-CLIENT-001` (DC-5; fold candidate with
    `S11-HU-PHASE-IDEMPOTENCY-001`).
  - `S13-ADR012-LOBBY-OPTIMISM-001` (DC-4; product decision per
    PROMPT 803 §9 #1).
  - `S13-S2C-SUCCESS-LOG-001` (DC-3).
  - `S13-OBSERVABLE-PRODUCER-AUDIT-001` (DC-8; test-only).
- PROMPT 803 §5 Nice-tier rows:
  - `S13-PLUGIN-REGISTRATION-INVARIANT-001` (DC-2).
  - `S13-IGNORE-ATTRIBUTE-DRIFT-001` (CI ignore-attr drift gate).
  - `S13-MANUAL-RUNBOOK-AUTOMATION-001` (S8-QA-001-W1 partial
    automation candidate; gated on story 017 outcome).
  - `S13-PROTO-MESSAGE-ID-001` (sequence-id field across reliable
    S2C).
- PROMPT 802 14 UI-clean-pass candidate slugs (kept under the
  Sprint 13 audit-roadmap-prep row `S13-UI-AUDIT-ROADMAP-PREP-001`;
  not individually activated):
  - `S12-UX-LOBBY-LAYOUT-MODAL-001`, `S11-UX-LOBBY-CLASS-PICKER`,
    `S11-UX-LOBBY-BUTTON-HITTARGETS`,
    `S11-UX-LOBBY-ROOM-CODE-EYEBALL-001`,
    `S11-UX-LOBBY-OPP-SLOT-DISAMBIGUATION-001`,
    `S11-UX-HUD-TOP-STRIP-LAYOUT`, `S11-UX-HUD-BOTTOM-STRIP-LAYOUT`,
    `S11-UX-HUD-OPP-FIGURINE`, `S12-TD-UI-OVERLAY-ALPHA-TOKEN-001`,
    `S12-UX-HAND-DRAG-STATE-VISUALS-001`,
    `S11-UX-DRAFT-GRID-CENTERED-MODAL`,
    `S11-UX-AUCTION-FEATURED-CARD`,
    `S11-UX-AUCTION-FREE-GOLD-COUNTERS`,
    `S12-UX-AUCTION-LEAD-LOSS-STATE-001`,
    `S11-UX-BOARD-RENDERING-SPEC`, `S11-TD-UI-FONT-CONSTANTS`,
    `S11-TD-UI-VIEWPORT-INVARIANT-TESTS`,
    `S11-TD-UI-ZINDEX-LAYERS`, `S11-TD-UI-FLEX-STRIPS`,
    `S12-TD-UI-CARD-SLOT-PRIMITIVE-001`.
- Server hardening test parity from Sprint 11/12 backlog:
  `S11-TD-NET-001`, `S11-TD-NET-002`, `S11-TD-NET-003`.
- `S11-TD-PRISM-COV-001` -- Cluster 2C advisory coverage gap on
  `S2CPrismRewardDropped` + `S2CPrismRespawned` (overlaps with
  Sprint 13 story 008 orphan-drain disposition).
- `S11-TD-HARNESS-MESSAGES-001` -- 4 harness bins downstream from
  PROMPT 690 needing `add_message::<PlayerTeamMapUpdated>`.
- `S11-TD-HARNESS-HANDUI-ENTITIES-001` -- 2 harness bins downstream
  from PROMPT 690 needing `HandUiEntities`.
- `S11-TD-BOARD-RENDERING-SNAPSHOT-PHASE-COUPLING-001` (split from
  PROMPT 680 PARTIAL closure).
- `S11-TD-FIXTURE-MESSAGES-002` (wider exhaustive `add_message` sweep
  -- Option B from PROMPT 708).
- `S11-TD-CI-NORMALIZE-COMMENTS-001` (teach `normalize_source()` to
  strip Rust comments -- Option B from PROMPT 674 FAIL report).
- `S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001` (closed by Sprint 13
  Should Have `S13-CONN-LOST-UX-001` if that row activates).
- Route-readability future-story candidates from
  `production/qa/evidence/sprint-10-route-readability-notes.md`
  (kept under the UI-audit-roadmap-prep row).

## Required Sprint 13 Story Docs

PROMPT 818 (this draft) did NOT author any new story files. The 8
PROMPT 804 candidate stories already exist on `main` from PROMPT
808 integration (`55b25be`). PROMPT 822 authored the 11 previously
missing Sprint 13 story files; PROMPT 823 verified all 12 newly
reviewed stories READY. PROMPT 825 refreshed the pointers below from
the placeholder `story-XXX` slugs to the actual story file paths.
Before `/dev-story` begins on any Must Have / Should Have / Nice to
Have row, each story file below must continue to pass `/story-readiness`
against Sprint 13 activation HEAD.

| Planned ID | Required story file | Status at PROMPT 825 |
|------------|---------------------|----------------------|
| S13-PROTO-INVARIANT-001 | `production/epics/lightyear-protocol-verification/story-007-protocol-completeness-invariant.md` | EXISTS on `main` (PROMPT 804 / PROMPT 808); `/story-readiness` READY per PROMPT 823 |
| S13-PROTO-ORPHAN-DRAIN-001 | `production/epics/lightyear-protocol-verification/story-008-protocol-orphan-drain.md` | EXISTS on `main`; `/story-readiness` READY per PROMPT 823 (producer decisions filled by PROMPT 821) |
| S13-FIXTURE-FACTORY-001 | `production/epics/playable-client/story-016-fixture-factory.md` | EXISTS on `main`; `/story-readiness` READY per PROMPT 823 |
| S13-TWO-CLIENT-RUNTIME-HARNESS-001 | `production/epics/playable-client/story-017-two-client-runtime-harness.md` | EXISTS on `main`; `/story-readiness` READY per PROMPT 823 |
| S13-OBS-TRACING-TARGETS-001 | `production/epics/playable-client/story-018-obs-tracing-targets.md` | EXISTS on `main`; `/story-readiness` READY per PROMPT 823 |
| S13-OBS-WALLCLOCK-TIMESTAMPS-001 | `production/epics/playable-client/story-019-obs-wallclock-timestamps.md` | EXISTS on `main`; `/story-readiness` READY per PROMPT 823. **Note**: This is a Sprint 13 candidate story-019 file in the `playable-client/` epic; it is **distinct from** the Sprint 12 hand-ui story-019 (`production/epics/hand-ui/story-019-drag-runtime-retest-tighter-capture.md`) which is already `Done` with `closed-with-conditions / cannot-reproduce` disposition. |
| S13-LATE-MSG-DEDUPE-001 | `production/epics/playable-client/story-020-late-msg-dedupe.md` | EXISTS on `main`; `/story-readiness` READY per PROMPT 823 |
| S13-CONN-LOST-UX-001 | `production/epics/playable-client/story-021-conn-lost-ux.md` | EXISTS on `main`; `/story-readiness` READY per PROMPT 823 |
| S11-HUD-TIMER-EYEBALL-VISUAL-001 | `production/epics/hud/story-014-hud-timer-eyeball-visual-check.md` | EXISTS on `main` (authored by PROMPT 822); `/story-readiness` READY per PROMPT 823 |
| S11-HU-PHASE-IDEMPOTENCY-001 | `production/epics/playable-client/story-022-client-phase-changed-idempotency.md` | EXISTS on `main` (authored by PROMPT 822); `/story-readiness` READY per PROMPT 823 |
| S11-SERVER-POOL-INIT-LOG-GUARD-001 | `production/epics/server/story-001-init-pool-log-guard.md` | EXISTS on `main` (authored by PROMPT 822); `/story-readiness` READY per PROMPT 823 |
| S11-LOBBY-UX-CONFIRM-STATE-001 | `production/epics/playable-client/story-023-lobby-confirm-state.md` | EXISTS on `main` (authored by PROMPT 822); `/story-readiness` READY per PROMPT 823 |
| S11-TD-CARGO-DISK-USAGE-001 | `production/epics/devops/story-001-cargo-workspace-disk-usage.md` | EXISTS on `main` (authored by PROMPT 822); `/story-readiness` READY per PROMPT 823 |
| S11-TD-CARGO-PDB-LIMIT-001 | `production/epics/devops/story-002-cargo-pdb-limit.md` | EXISTS on `main` (authored by PROMPT 822); `/story-readiness` READY per PROMPT 823 |
| S11-OPS-ORCHESTRATOR-LOCK-001 | `production/epics/devops/story-003-orchestrator-lock.md` | EXISTS on `main` (authored by PROMPT 822); `/story-readiness` READY per PROMPT 823 |
| S11-OPS-GH-CLI-001 | `production/epics/devops/story-004-gh-cli-setup.md` | EXISTS on `main` (authored by PROMPT 822); `/story-readiness` READY per PROMPT 823 |
| S11-SERVER-R2-PLACEMENT-CRASH-AUDIT-001 | `production/epics/server/story-002-r2-placement-crash-audit.md` | EXISTS on `main` (authored by PROMPT 822); `/story-readiness` READY per PROMPT 823 |
| S13-OPS-WIN-APPCOMPAT-NOTE-001 | `production/epics/devops/story-005-win-appcompat-note.md` | EXISTS on `main` (authored by PROMPT 822); `/story-readiness` READY per PROMPT 823 |
| S13-UI-AUDIT-ROADMAP-PREP-001 | `production/epics/ui-clean-pass/story-001-prompt-802-audit-roadmap-prep.md` | EXISTS on `main` (authored by PROMPT 822); `/story-readiness` READY per PROMPT 823 |

All 20 Sprint 13 story files now exist on `main` and have a current
`/story-readiness` verdict of READY. Sprint 13 is still DRAFT and
NOT activated; activation happens via `/sprint-plan sprint-13` in a
separate prompt.

## Explicitly NOT Claimed by Sprint 13 Draft

PROMPT 818 (this draft) does NOT claim, and Sprint 13 activation will
NOT claim, any of the following:

- public release readiness
- release-candidate readiness
- full game completion
- broad / Standard-tier accessibility completion
- playtest / fun-hypothesis validation
- full playable-client manual QA
- two-client GAME_OVER closure (`S8-QA-001-W1`)
- final-art / asset-production completion
- Polish->Release gate-check retry
- stage advance from Polish to Release
- **Sprint 13 activation** (this is a draft, not an activation)
- underlying drag-runtime bug fix (Sprint 12 story 019 remains
  `closed-with-conditions / cannot-reproduce`)
- closure of `S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001` outside
  the bounds of Sprint 13 Should Have `S13-CONN-LOST-UX-001`
- full UI clean-pass repair (Sprint 13 UI work is audit/roadmap-prep
  only via `S13-UI-AUDIT-ROADMAP-PREP-001`)

## Sequencing Notes

Per PROMPT 803 §6 + §7 + §8 and PROMPT 804 §"Notes for the Orchestrator":

- Stories 007 + 008 (lightyear-protocol-verification) are designed
  to land together in the same Sprint 13 wave. Story 008 flips the
  Story 007 invariant test from FAIL-with-9-orphans to PASS. If
  scheduled in different waves, Story 007 may need a temporary
  `#[ignore]` until Story 008 lands; either way, the decision is
  recorded before code change.
- Story 017 (two-client runtime harness) explicitly does NOT close
  `S8-QA-001-W1` by itself (per its AC12 forbid-auto-closure).
- Story 018 (tracing targets) historically collided on
  `client/src/presentation/board_rendering.rs` with Sprint 12 Story
  014; Sprint 12 Story 014 is now closed (decision commit `d5053fe`
  + code-change commit `ae6635d`). Worker still must rebase / re-check
  on Sprint 13 activation HEAD before starting.
- Story 020 (late-msg dedupe) historically collided on
  `client/src/ui/lobby.rs` with Sprint 12 Story 013 and on
  `client/src/presentation/board_rendering.rs` with Sprint 12 Story
  014; Sprint 12 Stories 013 + 014 are now closed. Worker still must
  rebase / re-check on Sprint 13 activation HEAD before starting.
- Story 021 (conn-lost UX) historically collided on
  `client/src/presentation/mod.rs` with Sprint 12 Story 012; Sprint
  12 Story 012 is now closed (commit `c1eef10`). Worker still must
  rebase / re-check on Sprint 13 activation HEAD before starting.
- The UI-audit-roadmap-prep row `S13-UI-AUDIT-ROADMAP-PREP-001` is
  parallel-safe with all other rows (touches `docs/ux/` and
  `production/epics/ui-clean-pass/` only).

## Cargo Resource Policy (this draft)

**Not applied** -- PROMPT 818 is a paperwork-only draft. No `cargo`
command was invoked. `$env:CARGO_TARGET_DIR`,
`$env:CARGO_PROFILE_DEV_DEBUG`, `$env:CARGO_PROFILE_TEST_DEBUG`,
`$env:CARGO_INCREMENTAL`, `$env:RUSTFLAGS` were not set. Cargo
resource policy was not applied because no Cargo command was needed.
