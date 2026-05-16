# Sprint 14 -- CLOSED-WITH-CONDITIONS (Polish stage)

> **PROMPT 987 close-out disposition (2026-05-16)**: Sprint 14 disposition
> flipped `active -> closed-with-conditions` on
> `origin/main@f6906020074f3d31e6594fb78788596bbac99477` (PROMPT 986 QA
> evidence integration tip).
>
> **Verdict**: `closed-with-conditions` (NOT release-ready, NOT `closed`).
> Must Have track **9/9 done**. Should Have track **3/4 done** (PROMPT
> 956 + PROMPT 953 + PROMPT 962 closures). Nice to Have track **4/4
> done** (PROMPT 976 + PROMPT 960 + PROMPT 972 + PROMPT 974 closures).
> Total **16 of 17 rows closed**. The single open row is
> `S11-HUD-TIMER-EYEBALL-VISUAL-001` (Should Have, story 014, 0.25d),
> which remains `ready` as the **human-operator-blocked carry**:
> cosmetic visual check requires human screenshot capture across
> `DraftInitial 45s` / `DraftShop 30s` / `Placement 10-12s` phases per
> story file ACs and cannot be auto-closed by an LLM `/story-done`.
> **Carried forward into Sprint 15 planning** (Sprint 13 -> Sprint 14
> -> Sprint 15 carry; originally Sprint 10 smoke retry-7 W2).
>
> **Smoke (PROMPT 983)**: `PASS-WITH-WARNINGS` at
> `production/qa/smoke-sprint-14-2026-05-16-rerun.md` (integrated on
> `origin/main` per PROMPT 986). Cargo aggregate 213 binaries /
> 1350 passed / 0 failed / 0 ignored; functional total with renamed
> AppCompat-blocked binary 1355 passed / 0 failed / 0 ignored across
> 214 effective binaries. Single warning is **environment/tool-only**
> (Windows AppCompat heuristic on `update` substring; identical
> classification to PROMPT 815 / 790 / 979 / 982; no code regression).
> PROMPT 978/979 UI drift repair targeted reruns
> `shop_auction_ui_plugin_scaffold_formulas_test` 8/8 PASS and
> `ui_clean_pass_z_layers_test` 6/6 PASS.
>
> **Team-QA (PROMPT 984)**: `APPROVED-WITH-CONDITIONS` at
> `production/qa/team-qa-sprint-14-2026-05-16.md` (integrated on
> `origin/main` per PROMPT 986). All 10 approval conditions are
> existing carry conditions; none closed by Team-QA.
>
> **Stage UNCHANGED**: `Polish`. `production/stage.txt` NOT modified by
> PROMPT 987. PROMPT 761 Polish->Release gate-check `FAIL` preserved at
> `production/gate-checks/gate-polish-release-2026-05-12.md`; **NO retry**
> attempted by Sprint 14 or PROMPT 987. Sprint 15 NOT activated by
> PROMPT 987; `production/sprints/sprint-15.md` NOT created;
> `production/qa/qa-plan-sprint-15.md` NOT created.
>
> **Conditions carried forward unchanged** (none closed by PROMPT 987):
> `S11-HUD-TIMER-EYEBALL-VISUAL-001` carried as human-operator-blocked
> Sprint 13 -> Sprint 14 -> Sprint 15 row; `S8-QA-001-W1` OPEN (Story
> 017 AC12 forbid-auto-closure preserved through Sprint 14);
> `QA-COND-0005` Standard-tier accessibility accepted-risk (L5
> `LOBBY_BUTTON_HEIGHT = 30.0` defect remains accepted-risk; story 026
> was friend-game stability work, not >=44 px hit-target conformance);
> `QA-COND-0006` playtest validation accepted-risk; `PAW-TD-*-a`
> placeholder-art accept-risk across PAW-002..PAW-006 (story 016 / 017 /
> 018 differentiation by layout / token color, NOT final-art
> replacement); PROMPT 683-era runtime divergence question preserved
> (folded into Sprint 12 story 019 cannot-reproduce closure; third
> same-scope retest NOT authorised per `TQ-S12-C2`); Sprint 12 story
> 019 underlying drag-runtime bug NOT claimed fixed; `TQ-S12-C1..C7`
> (all 7 Sprint 12 Team-QA conditions) preserved verbatim;
> `S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001` backlog row NOT
> flipped; Sprint 13 / Sprint 12 / Sprint 11 / Sprint 10 closeouts
> preserved unchanged.
>
> **Explicitly NOT claimed by PROMPT 987**: public release readiness;
> release-candidate readiness; full game completion; broad /
> Standard-tier accessibility completion; playtest / fun-hypothesis
> validation; full playable-client manual QA; two-client GAME_OVER
> closure (`S8-QA-001-W1` remains OPEN); final-art / asset-production
> completion (`PAW-TD-*-a` accept-risk preserved); Polish->Release
> gate-check retry (PROMPT 761 `FAIL` preserved); stage advance from
> Polish to Release; closure of `S11-HUD-TIMER-EYEBALL-VISUAL-001`
> (human-operator-blocked carry); closure of
> `S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001`; TQ-S12-C7 closure;
> underlying drag-runtime bug fix (Sprint 12 story 019 closed
> cannot-reproduce, NOT bug-fixed); full UI clean-pass repair beyond
> the 17 Sprint 14 candidate rows (Tier 2 / Tier 3 ranks remain out of
> scope); Sprint 15 activation; any code change under `client/` /
> `server/` / `shared/` / `tests/` by PROMPT 987 (paperwork-only
> close-out); any change to `production/stage.txt`,
> `production/qa/qa-plan-sprint-14.md`, `production/gate-checks/*`,
> release artifacts, release-checklist, launch-checklist, changelog, or
> patch notes by PROMPT 987.
>
> **PROMPT 987 paperwork-only close-out scope**: changed only
> `production/sprint-status.yaml` (top-level `status:` flip + `updated:`
> refresh + `sprint_14_closeout:` block appended at EOF) +
> `production/sprints/sprint-14.md` (this banner only; plan body NOT
> rewritten) + `production/session-state/active.md` (banner prepended) +
> `production/session-state/codex-orchestrator-state.md` (section
> prepended) + `production/qa/evidence/sprint-14-close-out-disposition.md`
> (NEW). No `cargo` / `trunk` command invoked. The 16 Sprint 14 closed
> rows and their `/story-done` paperwork remain on `origin/main`
> unchanged; PROMPT 987 does not re-edit any story file or any
> previously-closed sprint-status row.
>
> ---
>
> **Prior PROMPT 897 activation (2026-05-15)**: Sprint 14 was **ACTIVATED** as of commit
> on `origin/main` from this paperwork-only activation prompt. Source-of-truth at
> activation: `origin/main@ce8f590` (PROMPT 896 Sprint 14 plan draft tip).
> `production/sprint-status.yaml` top-level `sprint: 13 -> 14` and
> `status: closed-with-conditions -> active` flipped by PROMPT 897. Stage UNCHANGED
> `Polish`. PROMPT 761 Polish->Release gate-check `FAIL` preserved at
> `production/gate-checks/gate-polish-release-2026-05-12.md` (NO retry). Sprint 13
> close-out `closed-with-conditions` (PROMPT 894) preserved unchanged. All carried
> conditions preserved verbatim (S8-QA-001-W1 OPEN, QA-COND-0005 + QA-COND-0006
> accepted-risk, PAW-TD-*-a accept-risk, PROMPT 683-era runtime divergence
> question, TQ-S12-C1..C7, Sprint 12 / 11 / 10 closeouts). Sprint 14 explicitly
> does NOT claim public release readiness, release-candidate readiness, full game
> completion, broad / Standard-tier accessibility completion, playtest validation,
> full playable-client manual QA, two-client GAME_OVER closure (S8-QA-001-W1),
> final-art / asset-production completion, Polish->Release retry, stage advance,
> or underlying drag-runtime bug fix.
>
> **`production/qa/qa-plan-sprint-14.md` was NOT created by PROMPT 897 and MUST be
> authored separately** via `/qa-plan sprint-14` **after** activation and
> **before** any Sprint 14 `/dev-story` runs. Per-story `/story-readiness` must
> re-run against Sprint 14 activation HEAD for each of the 16 UI candidate story
> files before `/dev-story`. The Sprint 13 carry `S11-HUD-TIMER-EYEBALL-VISUAL-001`
> remains human-operator-blocked; no LLM `/story-done` is authorised.
>
> **Worktree**: `D:\_DEV\wt\ccgs-prompt-897-s14-activation` (fresh detached
> worktree on `origin/main@ce8f590` because root checkout was behind origin/main
> by 7 commits and had unrelated dirt — `M .claude/settings.json` +
> `_run_build_server.bat` staged-add + 3 untracked files; root-checkout dirt NOT
> touched by PROMPT 897; pattern matches PROMPT 884 / 885 / 888 / 891 / 894
> precedent).
>
> **Files changed by PROMPT 897 (4 paperwork files + 1 report)**:
> `production/sprint-status.yaml` (top-level flips + stories block replaced +
> `carried_into_sprint_14:` appended + `sprint_14_activation:` block appended) +
> `production/sprints/sprint-14.md` (this banner) +
> `production/session-state/active.md` (PROMPT 897 banner prepended) +
> `production/session-state/codex-orchestrator-state.md` (PROMPT 897 section
> prepended) + `reports/PROMPT-897-Sprint-14-Activation.md` (gitignored).
>
> ---
>
> **Prior PROMPT 896 draft authoring (2026-05-15)**: This was the Sprint 14
> sprint plan **DRAFT**. Sprint 14 was **NOT activated** by that draft.
> Activation happened in PROMPT 897 (this prompt) which flipped
> `production/sprint-status.yaml` top-level `sprint: 13 -> 14` and
> `status: closed-with-conditions -> active` and appended a
> `sprint_14_activation:` block, mirroring the PROMPT 798 (Sprint 12 activation)
> / PROMPT 826 (Sprint 13 activation) pattern.
>
> **Status**: `draft -- authored 2026-05-15 by PROMPT 896` (paperwork-only
> repair of PROMPT 895 NEEDS-WORK verdict: the only Sprint 14 activation blocker
> was that this file did not exist on `origin/main`).
> **Source-of-truth at authoring**: `origin/main@e216a96` (PROMPT 894
> `close-out(s13): Sprint 13 close-out disposition closed-with-conditions`).
> **Worktree**: `D:\_DEV\wt\ccgs-prompt-896-sprint14-plan` (fresh detached
> worktree on `origin/main@e216a96`).
> **Stage**: `Polish` (UNCHANGED; this draft does NOT advance stage).
> **Start / end (provisional; locked at activation)**: 2026-07-16 ->
> 2026-07-29 (10 workdays). Continuous follow-on to Sprint 13 (2026-07-02 ->
> 2026-07-15).
>
> **Sprint 14 does NOT claim Release readiness, RC readiness, or any
> Polish->Release stage advance.** Sprint 14 remains a `Polish`-stage sprint
> focused on UI clean-pass foundation + Tier 1 layout composition + carry
> forward of the Sprint 13 human-blocked HUD timer eyeball visual check. The
> PROMPT 761 `Polish->Release` gate-check `FAIL` evidence preserved at
> `production/gate-checks/gate-polish-release-2026-05-12.md` is NOT retried by
> this draft and MUST NOT be retried by Sprint 14 activation.
>
> **PROMPT 896 paperwork-only draft**: NO `/dev-story`, NO `/story-readiness`,
> NO `/story-done`, NO `/smoke-check`, NO `/team-qa`, NO `/gate-check`, NO
> `/release-check`, NO `/qa-plan`, NO implementation, NO CI run, NO `cargo` /
> `trunk` invocation. NO file touched outside `production/sprints/sprint-14.md`
> by PROMPT 896. **Sprint 14 is NOT activated by this draft.**
>
> **`production/qa/qa-plan-sprint-14.md` is NOT authored by this draft.** Per
> the Sprint 13 precedent (qa-plan-sprint-13.md was authored separately after
> Sprint 13 activation via `/qa-plan sprint`), Sprint 14's QA plan MUST be
> authored via a separate `/qa-plan sprint-14` prompt **after** activation and
> **before** any Sprint 14 `/dev-story` runs.

---

## Planning Notes

- Current stage is `Polish`. `production/stage.txt` reads `Polish`. Sprint 14
  does NOT advance stage. Sprint 14 is NOT a `Polish->Release` sprint.
- Sprint 13 is `closed-with-conditions` per PROMPT 894 (commit `e216a96` on
  `origin/main`); Must Have 6/6 done, Should Have 5/6 done (one row
  human-operator-blocked carry), Nice to Have 7/7 done; 18 of 19 rows closed.
  The single un-closed row is `S11-HUD-TIMER-EYEBALL-VISUAL-001` (Should Have,
  story 014, 0.25d, human-operator-blocked cosmetic visual check); it is
  **carried forward** into Sprint 14 planning here.
- This draft pulls candidates from the 16 Sprint 14 candidate UI story files
  integrated onto `origin/main` by PROMPT 893 across four `--no-ff` integration
  merges (commits `9f36663` + `2d8eaac` + `2bdb277` + `466d3d4`) authored by
  PROMPT 878 (ui-clean-pass Tier 0 foundation), PROMPT 879 (HUD Tier 1
  layout), PROMPT 880 (playable-client lobby Tier 1 layout), and PROMPT 881
  (shop-auction-ui Tier 1 auction/draft layout). The 16 files all carry
  Sprint 14 candidate "NOT activated" banners and have an explicit
  authoring source-of-truth. None is `/story-readiness` READY against current
  HEAD yet; each requires its own `/story-readiness` pass against Sprint 14
  activation HEAD before any `/dev-story`.
- Sequencing is governed by the canonical reconciliation roadmap at
  `docs/ux/ui-clean-pass-roadmap.md` (PROMPT 838 authored under Sprint 13
  Nice to Have row `S13-UI-AUDIT-ROADMAP-PREP-001`, story
  `production/epics/ui-clean-pass/story-001-prompt-802-audit-roadmap-prep.md`,
  closed by PROMPT 856 `/story-done`). That roadmap reconciles the 14 PROMPT
  802 candidate slugs with the PROMPT 685 8-story milestone backlog and
  documents the canonical sequence (Tier 0 ranks 1-6 before Tier 1 ranks
  7-12 before Tier 3 ranks 13-14).
- Per the roadmap §"Recommended Sprint 14 Activation Pattern", Sprint 14 pulls
  (1) the 4 highest-impact rows (Tier 0 rank 1, Tier 1 ranks 7, 10, 12) as the
  Must Have headline, (2) the remaining Tier 0 ranks 2-6 as Must Have
  foundational rows (Tier 0 must land before any Tier 1 row enters
  `/dev-story`), (3) the remaining Tier 1 Must rows (ranks 8, 9, 11) as
  Should Have (deferrable to Sprint 15 if Tier 0 burn-down consumes Sprint 14
  capacity), and (4) Tier 1 Should-priority adjacent rows (HUD opp figurine,
  auction free-gold counters, lobby button hit-targets, auction lead-loss
  state) as Nice to Have. Tier 3 rows (ranks 13, 14) are explicitly deferred
  to Sprint 15.
- The HUD timer eyeball visual check (`S11-HUD-TIMER-EYEBALL-VISUAL-001`,
  story 014) is carried forward from Sprint 13 as a Should Have row.
  Disposition: **human-operator-blocked**; cosmetic visual check requires
  human screenshot capture across `DraftInitial` 45s / `DraftShop` 30s /
  `Placement` 10-12s phases and cannot be auto-closed by an LLM
  `/story-done`. Closure remains gated on human-operator screenshot capture
  per the Sprint 13 closeout (PROMPT 894) carry plan.
- PR-SPRINT skipped -- Lean mode (no `production/review-mode.txt`).
- No Sprint 14 QA plan exists at draft time. A Sprint 14 QA plan
  (`production/qa/qa-plan-sprint-14.md`) MUST be authored via `/qa-plan
  sprint-14` **after** Sprint 14 activation **and after** each Sprint 14 story
  file passes `/story-readiness` against activation HEAD. No `/dev-story` is
  authorised before the QA plan exists.
- Sprint 14 explicitly does NOT claim public release readiness,
  release-candidate readiness, full game completion, broad / Standard-tier
  accessibility completion (`QA-COND-0005`), playtest / fun-hypothesis
  validation (`QA-COND-0006`), full playable-client manual QA, two-client
  GAME_OVER closure (`S8-QA-001-W1`), final-art / asset-production
  completion (`PAW-TD-*-a`), `Polish->Release` gate-check retry, stage
  advance from Polish to Release, or underlying drag-runtime bug fix
  (Sprint 12 story 019 remains `closed-with-conditions / cannot-reproduce`).
  None of these can be added to Sprint 14 by activation; each requires its
  own scope and gate evidence.

## Entry Conditions (must be true at activation)

- `production/sprint-status.yaml` top-level reads `sprint: 13`,
  `status: "closed-with-conditions"` (already true at draft time per
  PROMPT 894).
- `production/stage.txt` reads `Polish` (UNCHANGED).
- PROMPT 761 `Polish->Release` gate-check `FAIL` evidence preserved at
  `production/gate-checks/gate-polish-release-2026-05-12.md`.
- Sprint 13 disposition (`closed-with-conditions` per PROMPT 894) preserved
  unchanged. Sprint 12 / Sprint 11 / Sprint 10 closeouts preserved unchanged.
- `S8-QA-001-W1` OPEN. `QA-COND-0005` + `QA-COND-0006` accepted-risk.
  `PAW-TD-*-a` accept-risk across PAW-002..PAW-006. PROMPT 683-era runtime
  divergence question preserved (no third same-scope retest per
  `TQ-S12-C2`). `TQ-S12-C1..C7` preserved verbatim. `S11-HUD-TIMER-EYEBALL-
  VISUAL-001` (Sprint 13 human-blocked carry) preserved.
- The 16 Sprint 14 candidate story files referenced below already exist on
  `origin/main` via PROMPT 893 (merge tips `9f36663` + `2d8eaac` +
  `2bdb277` + `466d3d4`). Each still requires its own `/story-readiness`
  pass against Sprint 14 activation HEAD before `/dev-story`.
- Story `production/epics/hud/story-014-hud-timer-eyeball-visual-check.md`
  exists on `origin/main` (authored by PROMPT 822; `/story-readiness` READY
  per PROMPT 823); the row carries forward from Sprint 13.
- PROMPT 802 §9 producer-decisions 1-6 are resolved or queued for resolution
  before any `/dev-story` on a row that names them as a blocker
  (decision 2 for ranks 2 / 3 / 5; decision 3 for rank 12; decision 4 for
  the Nice to Have `S12-UX-AUCTION-LEAD-LOSS-STATE-001` row).

If any entry condition fails, Sprint 14 does NOT activate; producer must
revise scope before activation.

## Sprint Goal

Sprint 14 is a **UI clean-pass Tier 0 foundation + Tier 1 layout composition
+ Sprint 13 HUD timer eyeball visual check carry** sprint, NOT a release
sprint. The goal is:

1. Land the Tier 0 foundation (z-layers, typography, flex strips, viewport-
   invariant tests, overlay alpha token, and global UI design spec). Per
   roadmap §3 Sequencing Rule 1, Tier 0 must land first; without these
   primitives, every Tier 1 surface story has to re-author primitives inline
   or skip token integration -- both reintroduce the original PROMPT 802
   defects.
2. Land the four highest-impact Tier 1 layout composition surfaces (HUD top
   strip, lobby layout modal, auction featured card differentiation, plus
   z-layers as the foundational refactor that gates them all) -- the
   "must land before any polished friend-game-product showcase" set named
   by `docs/ux/ui-clean-pass-roadmap.md` §"3-4 Highest-Impact Rows For
   Sprint 14 Must Have Framing".
3. Carry forward `S11-HUD-TIMER-EYEBALL-VISUAL-001` (HUD timer eyeball
   visual check) from Sprint 13 as a Should Have human-operator-blocked
   row. Closure remains gated on human screenshot capture; no LLM
   `/story-done` is authorised.
4. Hold remaining Tier 1 Must rows (HUD bottom strip, draft centered modal,
   lobby class-picker) as Should Have, deferrable to Sprint 15 if Tier 0
   burn-down consumes Sprint 14 capacity.
5. Hold Tier 1 Should-priority adjacent rows (HUD opp figurine, auction
   free-gold counters, lobby button hit-targets, auction lead-loss state)
   as Nice to Have; do not pull them ahead of their paired Tier 1 Must row
   on the same surface.

Sprint 14 does NOT claim release readiness, broad accessibility completion,
full playable-client manual QA, playtest validation, final-art /
asset-production completion, S8-QA-001-W1 closure, full game completion,
two-client GAME_OVER closure, a Polish->Release retry, or closure of the
underlying drag-runtime bug from Sprint 12 story 019.

## Capacity (provisional)

- Total workdays: 10 (assumes 2-week sprint same as Sprint 10/11/12/13)
- Buffer (20%): 2 days reserved for Tier 0 design-token integration friction
  (rank 1 z-layers + rank 3 flex strips + rank 5 overlay alpha all touch a
  shared `client/src/ui/design_tokens/` host module per roadmap §3 Sequencing
  Rule 2 -- mostly serial), Tier 1 visual-evidence capture friction (HUD top
  strip + lobby layout modal + auction featured card each need a baseline
  capture + post-migration capture), PROMPT 802 §9 producer-decision
  resolution (decisions 2 / 3 / 4 are blocking for ranks 2 / 3 / 5 / 12 and
  for the lead-loss-state Nice to Have), and per-story `/story-readiness`
  re-runs against Sprint 14 activation HEAD.
- Available: **8 effective planned days**
- Planned Must Have scope: **~6.25 estimated days** (4 highest-impact rows
  totalling 3.5d + remaining Tier 0 ranks 2-6 totalling 2.75d). Risk: Tier 0
  foundational work is partial-collision-prone on the shared design-token
  host module and may run mostly serial; if Tier 0 burn-down stretches,
  Should Have rows shift to Sprint 15 per roadmap §"Recommended Sprint 14
  Activation Pattern" rule 3.
- Should Have scope is conditional and must not displace Must Have closure.
  Total Should Have effort: ~2.50d.
- Nice to Have scope is layout / cosmetic polish + the Sprint 13 carry +
  adjacent Tier 1 Should rows; it lands only when Should Have closure is on
  track. Total Nice to Have effort: ~2.00d.

---

## Tasks

> All IDs below are **draft Sprint 14 candidate** tickets. They are NOT yet
> active `production/sprint-status.yaml` rows. Promotion to active rows
> happens at activation via `/sprint-plan sprint-14` (or an equivalent
> activation prompt), mirroring the PROMPT 798 / PROMPT 826 pattern.
> All slug provenance and rank references are against
> `docs/ux/ui-clean-pass-roadmap.md`. Each row pulls a story file already
> integrated onto `origin/main` by PROMPT 893 (the four `--no-ff` merge tips
> `9f36663` + `2d8eaac` + `2bdb277` + `466d3d4`) -- except for the Sprint 13
> carry `S11-HUD-TIMER-EYEBALL-VISUAL-001`, which lives at
> `production/epics/hud/story-014-hud-timer-eyeball-visual-check.md`
> (PROMPT 822 author / PROMPT 823 READY).

### Must Have (Critical Path)

| ID | Task | Agent/Owner | Est. Days | Source | Acceptance Criteria (draft) |
|----|------|-------------|-----------|--------|------------------------------|
| S11-TD-UI-ZINDEX-LAYERS | Centralised UI Z-Index Layer Constants -- named layer enum + const module for z-order; migrate every UI root + `client/src/presentation/result_screen.rs:512` `GlobalZIndex(100)` literal | UI programmer + lead-programmer | 1.0 | Roadmap rank 1 (Tier 0, Must, PROMPT 685 row 1, re-validated by PROMPT 802 §3.9 G1 / §6 Lane A); roadmap §"3-4 Highest-Impact Rows" #1; **foundational refactor that must land FIRST per roadmap §3 Sequencing Rule 1**; story file at `production/epics/ui-clean-pass/story-002-ui-zindex-layers.md` (PROMPT 878 / PROMPT 893 `9f36663`) | Story 002 `/story-readiness` passes against Sprint 14 activation HEAD. AC1-AC9 per story-002. Named-layer module (likely `client/src/ui/design_tokens/z_layers.rs`) exports ≥ 8 named `GlobalZIndex` constants (`Background < World < Units < UiBase < UiOverlay < Modal < Toast < Debug`). All UI roots in `client/src/ui/` migrated; `client/src/presentation/result_screen.rs:512` `GlobalZIndex(100)` migrated to the new `Modal` constant. Grep guard prevents reintroduction of inline `ZIndex(N)` / `GlobalZIndex(N)` literals outside the design-token module. Reconnect / snapshot-rebuild invariant test asserts painted layer order under out-of-order respawn. ADR-021 alignment preserved or amended. No optimistic client-side authority added (ADR-002 binding). `QA-COND-0005` + `QA-COND-0006` + `PAW-TD-*-a` accept-risk preserved unchanged. |
| S11-TD-UI-FONT-CONSTANTS | Typography Scale Tokens -- single-source-of-truth typography scale module (Caption / Body / H3 / H2 / H1 / Display + weights + line-height ratio); migrate all `font_size:` literals across `client/src/ui/` + `client/src/presentation/result_screen.rs` | UI programmer + ux-designer | 0.5 | Roadmap rank 2 (Tier 0, Must, PROMPT 685 row 7, re-validated by PROMPT 802 §3.9 G3 / §6 Lane A); story file at `production/epics/ui-clean-pass/story-003-ui-font-constants.md` (PROMPT 878 / PROMPT 893 `9f36663`). **Producer-decision-2 (numeric values per PROMPT 802 §9) blocking before `/dev-story`.** | Story 003 `/story-readiness` passes. Typography module authored with named semantic levels mapped to stable `Val::Px(N)` values. All inline `font_size:` literals under `client/src/ui/` and the three result-screen `font_size` values (`36 / 18 / 15`) migrated to the new module. Lobby typography hierarchy inversion (PROMPT 802 §3.1 L6: labels at `13px` smaller than the data at `15-18px`) corrected. Grep guard prevents reintroduction of inline `font_size: Val::Px(N)` literals. `QA-COND-0005` accept-risk preserved (this story does NOT advance Standard-tier text-size / WCAG conformance). `PAW-TD-*-a` accept-risk preserved (font-asset switching may be deferred to a follow-on story). |
| S11-TD-UI-FLEX-STRIPS | Flex-Based UI Strip Composition Primitives -- HeaderBar / LaneBar / HandBar / FooterBar flex primitives + SPACING_XS..XL constants; migrate HUD top + bottom + hand UI to flex primitives | UI programmer + ux-designer | 1.0 | Roadmap rank 3 (Tier 0, Must, PROMPT 685 row 2 partial, re-validated by PROMPT 802 §3.9 G2 / §6 Lane A); story file at `production/epics/ui-clean-pass/story-004-ui-flex-strips.md` (PROMPT 878 / PROMPT 893 `9f36663`). **Producer-decision-2 blocking before `/dev-story`.** | Story 004 `/story-readiness` passes. Strip-composition primitive module authored. HUD top strip migrated to `HeaderBar` primitive with flex children for gold / mana / phase / timer (eliminates `hud_margin + 48.0` / `+ 60.0` / `HUD_GOLD_ROW_GAP_PX = 48.0` / `HUD_SECONDARY_ROW_GAP_PX = 28.0` magic offsets). HUD bottom strip migrated to `FooterBar`. Hand UI migrated to `HandBar` preserving the `f190cc7` card-fan repair. Per-module `_GAP_PX` constants replaced by `SPACING_*` tokens. No optimistic client-side authority added (read-only over server-authoritative state). `QA-COND-0005` + `PAW-TD-*-a` accept-risk preserved. |
| S11-TD-UI-VIEWPORT-INVARIANT-TESTS | Automated UI Viewport-Invariant Test Bin -- new integration test asserting no-overlap / no-clipping / stable-anchor / deterministic-strip-height across 1366×768, 1920×1080, 1920×1200, 1280×960, 3840×2160, 2560×1080 | qa-lead + UI programmer | 1.0 | Roadmap rank 4 (Tier 0, Must, PROMPT 685 row 8, re-validated by PROMPT 802 §3.9 G5 / §6 Lane A); parallel-safe with ranks 1-3 per roadmap §3 Sequencing Rule 2; story file at `production/epics/ui-clean-pass/story-005-ui-viewport-invariant-tests.md` (PROMPT 878 / PROMPT 893 `9f36663`) | Story 005 `/story-readiness` passes. New integration test bin under `tests/integration/ui_viewport_invariants_test.rs` (NEW) spawns the playable client UI deterministically across the 6-viewport matrix. Invariants asserted per surface: no overlap (excluding intentional overlay layers by named z-layer), no clipping, stable anchor points, deterministic strip heights. Test helper module at `tests/integration/helpers/ui_viewport.rs` (NEW) exposes viewport-spawning + bounding-rect extraction utilities. Test bin is parallel-safe with ranks 1-3 (test-only; no shared host module). `QA-COND-0005` accept-risk preserved (viewport invariants are layout invariants, not Standard-tier accessibility conformance). |
| S12-TD-UI-OVERLAY-ALPHA-TOKEN-001 | Single-Source Overlay Alpha Token -- replace HUD dim `0.45` + settlement overlay `0.58` + result panel backdrop `0.46` with named `OVERLAY_DIM_ALPHA` / `OVERLAY_SCRIM_ALPHA` constants | UI programmer + ux-designer | 0.25 | Roadmap rank 5 (Tier 0, Must, net-new, PROMPT 802 §3.2 H4 / §3.9 G4 / §6 Lane B); story file at `production/epics/ui-clean-pass/story-006-ui-overlay-alpha-token.md` (PROMPT 878 / PROMPT 893 `9f36663`). **Producer-decision-2 blocking before `/dev-story`.** | Story 006 `/story-readiness` passes. Overlay-token entry in design-token module exports `OVERLAY_DIM_ALPHA` + `OVERLAY_SCRIM_ALPHA` (and optionally `OVERLAY_TOAST_ALPHA`). All three scattered alpha values migrated. Visual continuity verified across combat -> settlement -> result-screen transitions. `QA-COND-0005` + `QA-COND-0006` + `PAW-TD-*-a` accept-risk preserved. |
| S12-UX-GLOBAL-UI-DESIGN-SPEC-001 | Canonical Global UI Design Spec -- author `docs/ux/global-ui-design-spec.md` consolidating layers / spacing / typography / alpha / color / responsive rules; producer + UX-designer + art-director ratification gate | ux-designer + art-director + producer | 1.0 | Roadmap rank 6 (Tier 0, Must, net-new, PROMPT 802 §3.9 G6 / §6 Lane B / §9 producer-decision-2); roadmap §3 Sequencing Rule 2 names this row "should be authored first in Phase 1 because Tier 0 token modules need its numeric values as input"; story file at `production/epics/ui-clean-pass/story-007-global-ui-design-spec.md` (PROMPT 878 / PROMPT 893 `9f36663`) | Story 007 `/story-readiness` passes. Global UI design spec at `docs/ux/global-ui-design-spec.md` (NEW) covers Status / No-Claim Banner, scope boundaries (friend-game vs Standard-tier), z-layer constants, typography scale, spacing scale, overlay alpha tokens, color palette, responsive-layout rules. UX-designer + art-director sign off in commit message or evidence doc. Tier 0 token modules (ranks 2 / 3 / 5) consume the spec's numeric values. `QA-COND-0005` + `QA-COND-0006` + `PAW-TD-*-a` accept-risk preserved verbatim in spec §1 Status banner. |
| S11-UX-HUD-TOP-STRIP-LAYOUT | HUD Top Strip Layout (Composition Only) -- migrate gold / mana / phase / timer readouts from absolute positioning + `hud_margin + N` magic offsets to a flex `HeaderBar` parent + flex children | UI programmer + ux-designer | 0.75 | Roadmap rank 7 (Tier 1, Must, PROMPT 685 row 2 HUD-strip slice, re-validated by PROMPT 802 §3.2 H1 / H8 / §6 Lane C); roadmap §"3-4 Highest-Impact Rows" #4; **depends on ranks 1, 3, 6 landing first**; story file at `production/epics/hud/story-015-hud-top-strip-layout.md` (PROMPT 879 / PROMPT 893 `2d8eaac`) | Story 015 (hud) `/story-readiness` passes against Sprint 14 activation HEAD (after Tier 0 ranks 1, 3, 6 land). HUD top strip composed via `HeaderBar` flex primitive from rank 3. Magic offsets (`hud_margin + 48.0` / `+ 60.0` / `HUD_GOLD_ROW_GAP_PX = 48.0` / `HUD_SECONDARY_ROW_GAP_PX = 28.0`) replaced by `SPACING_*` tokens. Z-order declared via named layer from rank 1. Visual capture comparison against pre-migration baseline in `production/qa/evidence/captures/hud-top-strip-baseline-*`. **Read-only over server-authoritative state** (ADR-021 + ADR-002 binding; HUD reads `Res<CurrentClientPhase>` / `GoldDisplayState` / `ManaDisplayState`). No final-art replacement (`PAW-TD-004-a` preserved). `QA-COND-0005` + `QA-COND-0006` preserved. |
| S11-UX-AUCTION-FEATURED-CARD | Auction Featured Card Visual Hierarchy -- differentiate the featured auction-up card from shop slot wells (both currently reuse the same placeholder chrome PNG); layout / composition change only | UI programmer + ux-designer | 0.75 | Roadmap rank 10 (Tier 1, Must, PROMPT 685 row 4 featured slice, re-validated by PROMPT 802 §3.6 A2 / §6 Lane C); roadmap §"3-4 Highest-Impact Rows" #3; **depends on ranks 1, 3, 6 landing first**; story file at `production/epics/shop-auction-ui/story-016-auction-featured-card.md` (PROMPT 881 / PROMPT 893 `466d3d4`) | Story 016 (shop-auction-ui) `/story-readiness` passes against Sprint 14 activation HEAD. Featured-card composition uses flex primitive + named z-layer + typography tokens from Tier 0. Differentiation is achieved by **layout / composition / hierarchy / scale**, NOT by final-art replacement (`PAW-TD-003-a` placeholder PNGs preserved per friend-game scope boundary in roadmap §"Friend-Game Scope vs Standard-Tier-Accessibility Scope Boundary"). Visual capture comparison in `production/qa/evidence/captures/auction-featured-card-baseline-*`. `QA-COND-0005` + `QA-COND-0006` + `PAW-TD-*-a` preserved. |
| S12-UX-LOBBY-LAYOUT-MODAL-001 | Lobby Layout Modal (First-Impression Surface) -- replace the 420×?? top-left absolute-positioned lobby column with a modal-panel or full-viewport hero layout (producer picks per PROMPT 802 §9 producer-decision-3) | UI programmer + ux-designer + producer | 1.0 | Roadmap rank 12 (Tier 1, Must, net-new, PROMPT 802 §3.1 L1 / §3.1 L4 / §6 Lane C / §9 producer-decision-3); roadmap §"3-4 Highest-Impact Rows" #2; **producer-decision-3 (modal-panel vs full-viewport hero) blocking before `/dev-story`**; **depends on ranks 1, 3, 4, 6 landing first**; story file at `production/epics/playable-client/story-024-lobby-layout-modal.md` (PROMPT 880 / PROMPT 893 `2bdb277`) | Story 024 (playable-client) `/story-readiness` passes against Sprint 14 activation HEAD. Producer-decision-3 resolved with documented rationale. Lobby root composed via flex primitive from rank 3 + named layer from rank 1 + typography tokens from rank 2. Viewport-invariant assertion from rank 4 confirms lobby fits 1366×768..3840×2160 + 2560×1080. Visual capture comparison in `production/qa/evidence/captures/lobby-layout-modal-baseline-*`. **No optimistic client-side authority added** (lobby reads server-authoritative `S2CClassLocked` / `S2CRoomReady` only; ADR-002 + ADR-008 + ADR-012 binding). `QA-COND-0005` accept-risk preserved (the L5 hit-target ≥44px defect on `LOBBY_BUTTON_HEIGHT = 30.0` remains friend-game-scope accept-risk; this story does NOT pursue Standard-tier hit-target conformance). `PAW-TD-*-a` preserved. |

**Must Have subtotal**: ~7.25 estimated days. Tier 0 (ranks 1-6) totals
3.75d (z-layers 1.0 + font 0.5 + flex 1.0 + viewport 1.0 + alpha 0.25 +
spec 1.0 minus internal mostly-serial overlap on the design-token host
module). Tier 1 headline (ranks 7, 10, 12) totals 2.5d. Buffer for
sequencing friction and partial-collision serialization within the
design-token host module remains in the 8-day capacity plan (2-day
reserve).

### Should Have

| ID | Task | Agent/Owner | Est. Days | Source | Acceptance Criteria (draft) |
|----|------|-------------|-----------|--------|------------------------------|
| S11-HUD-TIMER-EYEBALL-VISUAL-001 | HUD Timer Eyeball Visual Check (Sprint 13 carry; human-operator-blocked) -- manual 2-client run validating timer countdown renders correctly for `DraftInitial` 45s, `DraftShop` 30s, `Placement` 10-12s phases | UI programmer + human operator | 0.25 | **Sprint 13 carry** per PROMPT 894 close-out `sprint_13_closeout.carried_into_sprint_14_planning`; originally Sprint 10 smoke retry-7 W2 carry into Sprint 11 / Sprint 12 / Sprint 13; story file at `production/epics/hud/story-014-hud-timer-eyeball-visual-check.md` (PROMPT 822 author / PROMPT 823 READY). **Carried forward unchanged from Sprint 13.** | Story 014 `/story-readiness` confirms READY against Sprint 14 activation HEAD (already READY per PROMPT 823 on `origin/main`). Manual 2-client run + screenshot evidence in `production/qa/evidence/sprint-14-hud-timer-visual-check/` (NEW). Cosmetic verification only; no production-code change unless an actual visual regression is found and a follow-on story is authored. **Closure remains gated on human-operator screenshot capture** -- no LLM `/story-done` is authorised. Does NOT claim `QA-COND-0005` Standard-tier accessibility completion. `PAW-TD-*-a` preserved. |
| S11-UX-HUD-BOTTOM-STRIP-LAYOUT | HUD Bottom Strip Layout (Composition Only) -- migrate figurine area + reserve-strip readouts from a single sprite + magic offset to a flex `FooterBar` parent | UI programmer + ux-designer | 0.5 | Roadmap rank 8 (Tier 1, Must, PROMPT 685 row 2 HUD-strip slice, re-validated by PROMPT 802 §3.2 H1 / H9 / §6 Lane C); **Should Have per roadmap §"Recommended Sprint 14 Activation Pattern" rule 3** (defers to Sprint 15 if Tier 0 burn-down consumes capacity); **depends on ranks 1, 3, 6 landing first**; story file at `production/epics/hud/story-016-hud-bottom-strip-layout.md` (PROMPT 879 / PROMPT 893 `2d8eaac`) | Story 016 (hud) `/story-readiness` passes. HUD bottom strip composed via `FooterBar` flex primitive from rank 3 + named z-layer from rank 1 + typography tokens from rank 2. Visual capture comparison. Read-only over server-authoritative state. `QA-COND-0005` + `QA-COND-0006` + `PAW-TD-004-a` preserved. |
| S11-UX-DRAFT-GRID-CENTERED-MODAL | Draft Initial Grid Centered Modal Layout -- migrate the draft-initial grid from its current presentation to a centered modal composition with z-layer / typography / spacing tokens | UI programmer + ux-designer | 0.75 | Roadmap rank 9 (Tier 1, Must, PROMPT 685 row 3, re-validated by PROMPT 802 §3.4 D1 / §6 Lane C); **Should Have per roadmap §"Recommended Sprint 14 Activation Pattern" rule 3**; **depends on ranks 1, 3, 4, 6 landing first**; story file at `production/epics/shop-auction-ui/story-015-draft-grid-centered-modal.md` (PROMPT 881 / PROMPT 893 `466d3d4`) | Story 015 (shop-auction-ui) `/story-readiness` passes. Draft-initial grid composed as centered modal via flex primitive + named z-layer (`Modal`) + typography tokens + viewport-invariant test confirms centering across the 6-viewport matrix. Visual capture comparison. `PAW-TD-003-a` preserved. `QA-COND-0005` + `QA-COND-0006` preserved. |
| S11-UX-LOBBY-CLASS-PICKER | Lobby Class-Picker Layout & Hierarchy -- migrate class-picker portrait row + button row + class label hierarchy to flex primitive + typography hierarchy correction | UI programmer + ux-designer | 1.0 | Roadmap rank 11 (Tier 1, Must, PROMPT 685 row 5 class-picker slice, re-validated by PROMPT 802 §3.1 L2 / §3.1 L3 / §6 Lane C); **Should Have per roadmap §"Recommended Sprint 14 Activation Pattern" rule 3**; **depends on ranks 1, 3, 4, 6 landing first**; story file at `production/epics/playable-client/story-025-lobby-class-picker-layout.md` (PROMPT 880 / PROMPT 893 `2bdb277`) | Story 025 (playable-client) `/story-readiness` passes. Class-picker composed via flex primitive + named z-layer + typography hierarchy correction (labels ≥ data scale). Viewport-invariant assertion confirms class-picker fits the matrix. Visual capture comparison. **No client-side class-lock authority added** (reads `S2CClassLocked` only; ADR-002 + ADR-008 + ADR-012 binding). `QA-COND-0005` accept-risk preserved (hit-target work is the sibling `S11-UX-LOBBY-BUTTON-HITTARGETS` Nice to Have row; this story is layout / hierarchy only). `PAW-TD-*-a` preserved. |

**Should Have subtotal**: ~2.50 estimated days. The Sprint 13 carry (0.25d)
is human-operator-blocked and cannot be auto-closed. The three Tier 1 Must
rows (ranks 8, 9, 11) are Sprint 14 Should Have **and deferrable to
Sprint 15** if Tier 0 burn-down consumes the Sprint 14 capacity, per
roadmap §"Recommended Sprint 14 Activation Pattern" rule 3.

### Nice to Have

| ID | Task | Agent/Owner | Est. Days | Source | Acceptance Criteria (draft) |
|----|------|-------------|-----------|--------|------------------------------|
| S11-UX-HUD-OPP-FIGURINE | HUD Opponent Figurine Composition (Layout Only) -- pair with HUD top + bottom strip composition; layout / composition only | UI programmer + ux-designer | 0.5 | Roadmap "Tier 1 Should-Priority Adjacent Rows" pair with ranks 7 / 8 (PROMPT 685 row 2 opp-figurine slice, PROMPT 802 §3.2 H10); story file at `production/epics/hud/story-017-hud-opponent-figurine.md` (PROMPT 879 / PROMPT 893 `2d8eaac`). **Do not pull ahead of its paired Tier 1 Must row** (HUD top or bottom strip) on the same surface. | Story 017 (hud) `/story-readiness` passes. Opp figurine composed via flex primitive + named z-layer + typography tokens. Visual capture comparison. `PAW-TD-004-a` preserved. `QA-COND-0005` + `QA-COND-0006` preserved. |
| S11-UX-AUCTION-FREE-GOLD-COUNTERS | Auction Free-Gold Counters Layout and Readability -- pair with auction featured card; layout / composition + readability | UI programmer + ux-designer | 0.5 | Roadmap "Tier 1 Should-Priority Adjacent Rows" pair with rank 10 (PROMPT 685 row 4 free-gold slice, PROMPT 802 §3.6 A3); story file at `production/epics/shop-auction-ui/story-017-auction-free-gold-counters.md` (PROMPT 881 / PROMPT 893 `466d3d4`). **Do not pull ahead of its paired Tier 1 Must row** (auction featured card) on the same surface. | Story 017 (shop-auction-ui) `/story-readiness` passes. Free-gold counter layout uses flex primitive + typography tokens; readability invariant asserted across the viewport matrix. Visual capture comparison. `PAW-TD-003-a` preserved. `QA-COND-0005` + `QA-COND-0006` preserved. |
| S11-UX-LOBBY-BUTTON-HITTARGETS | Lobby Button Dimensions & Hit-Target Stability (Friend-Game Scope) -- canonical button width / height constants + dimension-stability invariant; `QA-COND-0005` accept-risk preserved on the L5 ≥44px hit-target gap | UI programmer + ux-designer | 0.25 | Roadmap "Tier 1 Should-Priority Adjacent Rows" pair with rank 11 (PROMPT 685 row 5 button-hittargets slice, PROMPT 802 §3.1 L5); story file at `production/epics/playable-client/story-026-lobby-button-hittargets.md` (PROMPT 880 / PROMPT 893 `2bdb277`). **Friend-game scope only; does NOT advance `QA-COND-0005` Standard-tier hit-target conformance** per roadmap §"Friend-Game Scope vs Standard-Tier-Accessibility Scope Boundary". **Do not pull ahead of its paired Tier 1 Must row** (lobby class-picker) on the same surface. | Story 026 (playable-client) `/story-readiness` passes. Canonical button width / height constants. Dimension-stability invariant test asserts buttons preserve their canonical dimensions across the viewport matrix. **The L5 `LOBBY_BUTTON_HEIGHT = 30.0` defect remains accepted-risk under `QA-COND-0005`** (friend-game scope; the story is layout-stability work, not Standard-tier ≥44px hit-target conformance). `PAW-TD-*-a` preserved. |
| S12-UX-AUCTION-LEAD-LOSS-STATE-001 | Auction Featured Card Leading / Losing State Visual -- visual differentiation for the leading vs losing bid state on the featured card; producer-decision-4 captured by PROMPT 967 as static token-colored border-frame before `/dev-story` | UI programmer + ux-designer + producer | 0.5 | Roadmap "Tier 1 Should-Priority Adjacent Rows" pair with rank 10 (net-new, PROMPT 802 §3.6 A7 / §9 producer-decision-4); story file at `production/epics/shop-auction-ui/story-018-auction-lead-loss-state.md` (PROMPT 881 / PROMPT 893 `466d3d4`). **Producer-decision-4 resolved by PROMPT 967; re-run `/story-readiness` before `/dev-story`.** **Do not pull ahead of its paired Tier 1 Must row** (auction featured card) on the same surface. | Story 018 (shop-auction-ui) `/story-readiness` passes. Producer-decision-4 resolved with documented rationale: extend Story 016 `AuctionFeaturedCardFrame`; leading uses `SEMANTIC_SUCCESS`, losing uses `SEMANTIC_ERROR`, neutral / pre-bid retains `ACCENT`; no pulse, chevron, badge, or new art. Visual capture comparison across leading + losing states. `PAW-TD-003-a` preserved. `QA-COND-0005` + `QA-COND-0006` preserved. |

**Nice to Have subtotal**: ~1.75 estimated days. These rows pair with their
Tier 1 Must row on the same surface (HUD opp figurine ↔ HUD top / bottom
strip; auction free-gold counters + lead-loss state ↔ auction featured
card; lobby button hit-targets ↔ lobby class-picker). Each is small (≤
0.5d) and should be activated only **after** its paired Tier 1 Must row
lands.

---

## Carryover from Sprint 13

| Source row (Sprint 13) | Disposition into Sprint 14 |
|------------------------|----------------------------|
| `S11-HUD-TIMER-EYEBALL-VISUAL-001` (Sprint 13 Should Have, `ready` after PROMPT 894 close-out -- the only un-closed row of Sprint 13; human-operator-blocked cosmetic visual check) | Pulled forward as Sprint 14 **Should Have** human-operator-blocked carry. Closure remains gated on human screenshot capture across `DraftInitial` 45s / `DraftShop` 30s / `Placement` 10-12s phases. No LLM `/story-done` is authorised; PROMPT 822 / PROMPT 823 / PROMPT 894 disposition preserved. Story file unchanged at `production/epics/hud/story-014-hud-timer-eyeball-visual-check.md`. |
| `S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001` backlog row (closed by `S13-CONN-LOST-UX-001` per PROMPT 891 `/story-done`; row-status flip itself remains a separate paperwork prompt per Sprint 13 closeout `conditions_carried_forward_unchanged`) | NOT pulled into Sprint 14 implementation. Row-status paperwork flip is **out of Sprint 14 implementation scope**; if a producer chooses to flip it in `production/sprint-status.yaml` they MUST do so via a separate paperwork prompt at activation time, NOT via Sprint 14 `/dev-story`. |

(All other Sprint 13 rows were closed by Sprint 13 `/story-done` per PROMPT
833 / 835-inline / 840 / 843 / 844 / 850 / 851 / 854 / 856 / 865 / 868 /
869 / 871 / 876 / 884 / 885 / 888 / 891 and the Sprint 13 close-out
disposition PROMPT 894. None require a Sprint 14 carry.)

## Conditions Carried Forward Unchanged (NOT closed by Sprint 14)

Sprint 14 explicitly preserves and does NOT claim closure for any of:

- **`S8-QA-001-W1`** -- manual / browser two-client GAME_OVER gap remains
  **OPEN**. Sprint 13 story 017 AC12 forbid-auto-closure was preserved
  through Sprint 13; Sprint 14 candidate stories (16 UI clean-pass rows + 1
  Sprint 13 HUD timer eyeball carry) do not touch the two-client GAME_OVER
  surface. Sprint 14 activation MUST NOT silently close `S8-QA-001-W1`.
- **`QA-COND-0005`** -- Standard-tier accessibility remains **accepted-risk**
  (friend-game scope only). Sprint 14 UI clean-pass repair is **friend-game
  visual polish only** per roadmap §"Friend-Game Scope vs Standard-Tier-
  Accessibility Scope Boundary". The L5 `LOBBY_BUTTON_HEIGHT = 30.0`
  defect (PROMPT 802 §3.1 L5) remains accepted-risk under `QA-COND-0005`;
  pulling `S11-UX-LOBBY-CLASS-PICKER` or `S11-UX-LOBBY-BUTTON-HITTARGETS`
  does NOT thereby commit to Standard-tier hit-target conformance.
  Sprint 14 does NOT pursue WCAG contrast ratios, ≥44px hit-targets, full
  keyboard navigation, screen reader support, colorblind modes, or text
  scaling.
- **`QA-COND-0006`** -- playtest / fun-hypothesis validation remains
  **accepted-risk / deferred**. Sprint 14 UI clean-pass polish does NOT
  advance playtest validation even when the surface is visibly polished.
- **`PAW-TD-*-a`** -- placeholder-art accept-risk across PAW-002..PAW-006
  remains in place. Sprint 14 layout / composition / hierarchy /
  typography / z-order work does NOT advance placeholder-art resolution;
  PROMPT 802 §7 places final-art work explicitly out of audit scope. The
  auction featured-card differentiation (rank 10) is achieved by **layout
  / composition / scale / hierarchy**, NOT by final-art replacement.
- **PROMPT 683-era runtime divergence question** -- folded into Sprint 12
  story 019 `closed-with-conditions / cannot-reproduce` (after second
  time-box exhaustion). Sprint 14 does NOT claim this question closed.
  **A third same-scope retest is NOT authorised** per `TQ-S12-C2`.
- **PROMPT 761 `Polish->Release` gate-check `FAIL`** -- preserved at
  `production/gate-checks/gate-polish-release-2026-05-12.md`. **NO retry**
  is in scope for Sprint 14. Stage remains `Polish`.
- **Sprint 12 story 019 underlying drag-runtime bug** -- NOT claimed fixed.
  Sprint 14 UI clean-pass work does not reproduce or fix the underlying
  drag-runtime behaviour.
- **`TQ-S12-C1..C7`** -- all 7 Sprint 12 Team-QA conditions preserved
  verbatim. `TQ-S12-C7` explicitly NOT closed by any Sprint 14 row.
- **Sprint 13 / Sprint 12 / Sprint 11 / Sprint 10 closeouts** -- preserved
  unchanged. Sprint 13 disposition `closed-with-conditions` per PROMPT 894;
  Sprint 12 disposition `closed-with-conditions` per PROMPT 817; Sprint 11
  disposition `closed-with-conditions` per PROMPT 792; Sprint 10
  disposition `closed-with-conditions` per PROMPT 763.

If any condition above changes during Sprint 14, it requires its own
separate story file and explicit disposition -- it cannot be silently
folded into another story.

## Wider Sprint 14 Backlog (not yet pulled into this draft)

The following candidates remain in the broader backlog and are **NOT
scheduled** into this Sprint 14 draft. They may be pulled by a producer
revision before activation, or deferred to Sprint 15:

- **Roadmap Tier 3 (ranks 13, 14)** -- explicitly deferred to Sprint 15 per
  roadmap §"Recommended Sprint 14 Activation Pattern" rule 4:
  - `S12-TD-UI-CARD-SLOT-PRIMITIVE-001` (rank 13, Should, 1.5d, net-new,
    PROMPT 802 §3.3 HA1 / §3.3 HA5 / §4 Tier 3.1) -- refactor touches
    hand + shop + auction together; must wait for Tier 1 surfaces stable.
  - `S11-UX-BOARD-RENDERING-SPEC` (rank 14, Should, 0.75d, PROMPT 685 row 6
    re-validated by PROMPT 802 §3.7 B1 / §4 Tier 3.2) -- doc-only spec
    authoring; depends on rank 6 (global design spec parent doc).
- **Roadmap Tier 0 Should-priority adjacent row**:
  - `S12-TD-UI-INTERACTION-STATE-PRIMITIVES-001` (Tier 0, Should, 1.0d,
    net-new, PROMPT 802 §3.9 G7) -- hover / focus / pressed / disabled
    primitive set; pairs with rank 6 (global design spec). Sprint 14
    Tier 1 surfaces tolerable without it but degrade to per-site button
    styling.
- **Roadmap Tier 1 Should-priority adjacent row**:
  - `S12-UX-HAND-DRAG-STATE-VISUALS-001` (Tier 1, Should, 0.5d, net-new,
    PROMPT 802 §3.3 HA3) -- hand drag-state visuals; orthogonal to ranks
    7-12; touches hand UI only.
- **Roadmap Tier 2 cosmetic / eyeball captures** (12 already-tracked
  candidates per PROMPT 802 §9 producer-decision-5) -- bundle as a single
  Sprint 14+ row `S14-UX-CAPTURES-CLEAN-PASS-001` (per roadmap
  §"Tier 2 Cosmetic / Eyeball Captures") if the producer activates them.
  Most are 0.10d-0.25d manual capture work. **Not pulled into this draft.**
  Note that `S11-HUD-TIMER-EYEBALL-VISUAL-001` is already a Sprint 14
  Should Have row (carried from Sprint 13); it is NOT a Tier 2 capture
  in this draft.
- **`S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001` backlog row-status
  paperwork flip** -- closed in evidence by `S13-CONN-LOST-UX-001` per
  PROMPT 891 but the row-status flip remains a separate paperwork prompt
  per Sprint 13 closeout `conditions_carried_forward_unchanged`. NOT a
  Sprint 14 `/dev-story` row; if a producer chooses to flip the row-status
  they do so at activation paperwork, NOT implementation.
- **Server hardening test parity from Sprint 11/12 backlog**:
  `S11-TD-NET-001`, `S11-TD-NET-002`, `S11-TD-NET-003`.
- **`S11-TD-PRISM-COV-001`** -- Cluster 2C advisory coverage gap on
  `S2CPrismRewardDropped` + `S2CPrismRespawned` (per-row drain-or-delete
  disposition was recorded in Sprint 13 story 008 closure; this row is
  the advisory test coverage follow-on).
- **`S11-TD-HARNESS-MESSAGES-001`** -- 4 harness bins downstream from
  PROMPT 690 needing `add_message::<PlayerTeamMapUpdated>`.
- **`S11-TD-HARNESS-HANDUI-ENTITIES-001`** -- 2 harness bins downstream
  from PROMPT 690 needing `HandUiEntities`.
- **`S11-TD-BOARD-RENDERING-SNAPSHOT-PHASE-COUPLING-001`** (split from
  PROMPT 680 PARTIAL closure).
- **`S11-TD-FIXTURE-MESSAGES-002`** (wider exhaustive `add_message` sweep
  -- Option B from PROMPT 708).
- **`S11-TD-CI-NORMALIZE-COMMENTS-001`** (teach `normalize_source()` to
  strip Rust comments -- Option B from PROMPT 674 FAIL report).
- **PROMPT 803 §5 Should/Nice rows** not pulled into Sprint 13:
  `S13-LOBBY-CONFIRMCLASS-SENDER-001`, `S13-COOCCUPANCY-INVARIANT-001`,
  `S13-PHASE-IDEMPOTENCY-CLIENT-001`, `S13-ADR012-LOBBY-OPTIMISM-001`,
  `S13-S2C-SUCCESS-LOG-001`, `S13-OBSERVABLE-PRODUCER-AUDIT-001`,
  `S13-PLUGIN-REGISTRATION-INVARIANT-001`,
  `S13-IGNORE-ATTRIBUTE-DRIFT-001`, `S13-MANUAL-RUNBOOK-AUTOMATION-001`
  (gated on Sprint 13 story 017 outcome; not authorised to advance
  S8-QA-001-W1 in Sprint 14), `S13-PROTO-MESSAGE-ID-001`.

## Required Sprint 14 Story Docs

PROMPT 896 (this draft) does NOT author any new story files. The 16 Sprint
14 candidate story files already exist on `origin/main` from PROMPT 893
integration (`9f36663` + `2d8eaac` + `2bdb277` + `466d3d4`). The Sprint 13
HUD timer eyeball visual check story 014 already exists on `origin/main`
(authored by PROMPT 822; `/story-readiness` READY per PROMPT 823). Before
`/dev-story` begins on any Must Have / Should Have / Nice to Have row,
each story file below must pass `/story-readiness` against Sprint 14
activation HEAD.

| Planned ID | Required story file | Source-of-truth (current) |
|------------|---------------------|---------------------------|
| S11-TD-UI-ZINDEX-LAYERS | `production/epics/ui-clean-pass/story-002-ui-zindex-layers.md` | EXISTS on `main` (PROMPT 878 / PROMPT 893 merge `9f36663`); requires Sprint 14 activation `/story-readiness` |
| S11-TD-UI-FONT-CONSTANTS | `production/epics/ui-clean-pass/story-003-ui-font-constants.md` | EXISTS on `main`; requires Sprint 14 activation `/story-readiness`; PROMPT 802 §9 producer-decision-2 blocking |
| S11-TD-UI-FLEX-STRIPS | `production/epics/ui-clean-pass/story-004-ui-flex-strips.md` | EXISTS on `main`; requires Sprint 14 activation `/story-readiness`; PROMPT 802 §9 producer-decision-2 blocking |
| S11-TD-UI-VIEWPORT-INVARIANT-TESTS | `production/epics/ui-clean-pass/story-005-ui-viewport-invariant-tests.md` | EXISTS on `main`; requires Sprint 14 activation `/story-readiness` |
| S12-TD-UI-OVERLAY-ALPHA-TOKEN-001 | `production/epics/ui-clean-pass/story-006-ui-overlay-alpha-token.md` | EXISTS on `main`; requires Sprint 14 activation `/story-readiness`; PROMPT 802 §9 producer-decision-2 blocking |
| S12-UX-GLOBAL-UI-DESIGN-SPEC-001 | `production/epics/ui-clean-pass/story-007-global-ui-design-spec.md` | EXISTS on `main`; requires Sprint 14 activation `/story-readiness`; PROMPT 802 §9 producer-decision-2 is partially this story's content |
| S11-UX-HUD-TOP-STRIP-LAYOUT | `production/epics/hud/story-015-hud-top-strip-layout.md` | EXISTS on `main` (PROMPT 879 / PROMPT 893 merge `2d8eaac`); requires Sprint 14 activation `/story-readiness` |
| S11-UX-AUCTION-FEATURED-CARD | `production/epics/shop-auction-ui/story-016-auction-featured-card.md` | EXISTS on `main` (PROMPT 881 / PROMPT 893 merge `466d3d4`); requires Sprint 14 activation `/story-readiness` |
| S12-UX-LOBBY-LAYOUT-MODAL-001 | `production/epics/playable-client/story-024-lobby-layout-modal.md` | EXISTS on `main` (PROMPT 880 / PROMPT 893 merge `2bdb277`); requires Sprint 14 activation `/story-readiness`; PROMPT 802 §9 producer-decision-3 blocking |
| S11-HUD-TIMER-EYEBALL-VISUAL-001 | `production/epics/hud/story-014-hud-timer-eyeball-visual-check.md` | EXISTS on `main` (PROMPT 822 author / PROMPT 823 READY); Sprint 13 carry; human-operator-blocked |
| S11-UX-HUD-BOTTOM-STRIP-LAYOUT | `production/epics/hud/story-016-hud-bottom-strip-layout.md` | EXISTS on `main` (PROMPT 879 / PROMPT 893 merge `2d8eaac`); requires Sprint 14 activation `/story-readiness` |
| S11-UX-DRAFT-GRID-CENTERED-MODAL | `production/epics/shop-auction-ui/story-015-draft-grid-centered-modal.md` | EXISTS on `main` (PROMPT 881 / PROMPT 893 merge `466d3d4`); requires Sprint 14 activation `/story-readiness` |
| S11-UX-LOBBY-CLASS-PICKER | `production/epics/playable-client/story-025-lobby-class-picker-layout.md` | EXISTS on `main` (PROMPT 880 / PROMPT 893 merge `2bdb277`); requires Sprint 14 activation `/story-readiness` |
| S11-UX-HUD-OPP-FIGURINE | `production/epics/hud/story-017-hud-opponent-figurine.md` | EXISTS on `main` (PROMPT 879 / PROMPT 893 merge `2d8eaac`); requires Sprint 14 activation `/story-readiness` |
| S11-UX-AUCTION-FREE-GOLD-COUNTERS | `production/epics/shop-auction-ui/story-017-auction-free-gold-counters.md` | EXISTS on `main` (PROMPT 881 / PROMPT 893 merge `466d3d4`); requires Sprint 14 activation `/story-readiness` |
| S11-UX-LOBBY-BUTTON-HITTARGETS | `production/epics/playable-client/story-026-lobby-button-hittargets.md` | EXISTS on `main` (PROMPT 880 / PROMPT 893 merge `2bdb277`); requires Sprint 14 activation `/story-readiness`; **friend-game scope; `QA-COND-0005` accept-risk preserved** |
| S12-UX-AUCTION-LEAD-LOSS-STATE-001 | `production/epics/shop-auction-ui/story-018-auction-lead-loss-state.md` | EXISTS on `main` (PROMPT 881 / PROMPT 893 merge `466d3d4`); producer-decision-4 resolved by PROMPT 967; requires `/story-readiness` re-run before `/dev-story` |

All 17 Sprint 14 story files (16 candidate UI rows + 1 Sprint 13 carry)
exist on `origin/main` at draft time. Sprint 14 is still **DRAFT and NOT
activated**; activation happens via a separate explicit prompt after this
draft is reviewed.

## Explicitly NOT Claimed by Sprint 14 Draft

PROMPT 896 (this draft) does NOT claim, and Sprint 14 activation MUST NOT
claim, any of the following:

- public release readiness
- release-candidate readiness
- full game completion
- broad / Standard-tier accessibility completion (`QA-COND-0005`)
- playtest / fun-hypothesis validation (`QA-COND-0006`)
- full playable-client manual QA
- two-client GAME_OVER closure (`S8-QA-001-W1`)
- final-art / asset-production completion (`PAW-TD-*-a`)
- `Polish->Release` gate-check retry (PROMPT 761 FAIL preserved; NO retry
  authorised per `TQ-S12-C2`-adjacent reasoning)
- stage advance from `Polish` to `Release`
- **Sprint 14 activation** (this is a draft, not an activation)
- underlying drag-runtime bug fix (Sprint 12 story 019 remains
  `closed-with-conditions / cannot-reproduce`; third same-scope retest
  NOT authorised per `TQ-S12-C2`)
- closure of `S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001` backlog row-
  status (the evidence closure landed in Sprint 13 via `S13-CONN-LOST-UX-001`
  per PROMPT 891, but the row-status flip itself is a separate paperwork
  prompt and not a Sprint 14 implementation row)
- full UI clean-pass repair beyond the 17 Sprint 14 story rows above
  (Tier 3 ranks 13 / 14 deferred to Sprint 15; Tier 2 cosmetic captures
  bundled into a separate row if a producer activates them)
- automated closure of `S11-HUD-TIMER-EYEBALL-VISUAL-001` (human-operator-
  blocked carry; no LLM `/story-done` authorised)
- creation of `production/qa/qa-plan-sprint-14.md` -- this is **explicitly
  out of Sprint 14 draft scope**; the QA plan MUST be authored separately
  via `/qa-plan sprint-14` after activation and before any Sprint 14
  `/dev-story`

## Sequencing Notes

Per `docs/ux/ui-clean-pass-roadmap.md` §3 Sequencing Rules and
§"Recommended Sprint 14 Activation Pattern":

1. **Tier 0 (ranks 1-6) lands FIRST.** Without Tier 0, every Tier 1
   surface story has to re-author primitives inline or skip token
   integration -- both reintroduce the original PROMPT 802 defects.
2. **Within Tier 0, rank 6 (`S12-UX-GLOBAL-UI-DESIGN-SPEC-001`) is
   authored first** because Tier 0 token modules (ranks 2, 3, 5) need
   its numeric values as input (PROMPT 802 §9 producer-decision-2).
   For *z-layers specifically* (rank 1), the layer ordering is
   structural rather than numeric, so rank 1 can land slightly before
   the spec is finalized; the spec authoring then ratifies the chosen
   integer values.
3. **Within Tier 0, ranks 1-5 are mostly serial.** Multiple stories
   touch a shared `client/src/ui/design_tokens/` host module; per
   PROMPT 802 §8 they run **mostly serial**. Rank 4 (viewport-invariant
   tests; new test bin) is parallel-safe with ranks 1-3.
4. **Tier 1 (ranks 7-12) waits for Tier 0.** Once Tier 0 lands, ranks
   7-12 each touch a different surface module (hud / shop_auction /
   lobby) so they are parallel-safe with each other within Tier 1.
5. **The Sprint 13 carry (`S11-HUD-TIMER-EYEBALL-VISUAL-001`) is
   parallel-safe with all other rows** (cosmetic visual check only;
   no shared host module; no code change unless a regression is found).
6. **Nice to Have Tier 1 Should-priority adjacent rows must NOT be
   pulled ahead of their paired Tier 1 Must row** on the same surface.
   (HUD opp figurine ↔ HUD top / bottom strip; auction free-gold
   counters + lead-loss state ↔ auction featured card; lobby button
   hit-targets ↔ lobby class-picker.)
7. **PROMPT 802 §9 producer-decisions 1-6 must be resolved or queued
   for resolution before `/dev-story`** on any row that names them as a
   blocker:
   - Decision 2 (numeric values for Tier 0 token modules): rows
     S11-TD-UI-FONT-CONSTANTS (rank 2), S11-TD-UI-FLEX-STRIPS (rank 3),
     S12-TD-UI-OVERLAY-ALPHA-TOKEN-001 (rank 5).
   - Decision 3 (lobby modal-panel vs full-viewport hero): row
     S12-UX-LOBBY-LAYOUT-MODAL-001 (rank 12).
   - Decision 4 (auction lead-loss visual language): Nice to Have row
     S12-UX-AUCTION-LEAD-LOSS-STATE-001 -- resolved by PROMPT 967 as a
     static token-colored border-frame; re-run `/story-readiness` next.
8. **PROMPT 802 §3.9 G8 lobby file collision risk**: the lobby class-
   picker (Sprint 14 Should Have, story 025) and the lobby layout modal
   (Sprint 14 Must Have, story 024) both touch `client/src/ui/lobby.rs`.
   Sequence story 024 first; story 025 rebases / re-checks on Sprint 14
   activation HEAD after story 024 lands. Historical collision on
   `client/src/ui/lobby.rs` with Sprint 13 stories 020 + 023 (both
   closed by PROMPT 884 / PROMPT 854 respectively).
9. **The Sprint 14 Must Have row `S11-UX-HUD-TOP-STRIP-LAYOUT` (rank 7,
   story 015) and Should Have row `S11-UX-HUD-BOTTOM-STRIP-LAYOUT`
   (rank 8, story 016) both touch `client/src/ui/hud/mod.rs`.** Sequence
   rank 7 first; rank 8 rebases on activation HEAD after rank 7 lands.

## Cargo Resource Policy (this draft)

**Not applied** -- PROMPT 896 is a paperwork-only draft. No `cargo`
command was invoked. `$env:CARGO_TARGET_DIR`,
`$env:CARGO_PROFILE_DEV_DEBUG`, `$env:CARGO_PROFILE_TEST_DEBUG`,
`$env:CARGO_INCREMENTAL`, `$env:RUSTFLAGS` were not set. Cargo resource
policy was not applied because no Cargo command was needed.

Sprint 14 implementation prompts (post-activation `/dev-story` workers and
integration merges) MUST apply the binding Windows / MSVC Cargo resource
policy:

- `$env:CARGO_TARGET_DIR = "D:\_DEV\cargo-target\ccgs-msvc"`
- `$env:CARGO_PROFILE_DEV_DEBUG = "0"`
- `$env:CARGO_PROFILE_TEST_DEBUG = "0"`
- `$env:CARGO_INCREMENTAL = "0"`
- `$env:RUSTFLAGS = "-C debuginfo=0 -C link-arg=/DEBUG:NONE"`

per the binding Sprint 13 precedent (PROMPT 829 / 833 worker + integration
applications carried this policy verbatim).
