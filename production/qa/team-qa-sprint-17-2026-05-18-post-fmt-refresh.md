# PROMPT 1288 — Sprint 17 Team-QA Refresh (post-fmt)

## Status line

1288: SPRINT-17-POST-FMT-TEAM-QA-REFRESH: APPROVED-WITH-CONDITIONS

## Verdict

APPROVED-WITH-CONDITIONS. This refresh supersedes the **smoke premise** of the
PROMPT 1278 Team-QA review of record without reopening Sprint 17 implementation,
without re-scoring story rows, and without changing Sprint 17 sprint state.
The carried conditions enumerated below are preserved verbatim.

This refresh is **not** Sprint 17 reopening, **not** Sprint 18 activation, **not**
a release-readiness or release-candidate claim, **not** a Polish → Release gate
retry, and **not** a stage advance.

## Why this refresh exists

| # | Event | Outcome |
|---|---|---|
| 1 | PROMPT 1278 Team-QA | `APPROVED-WITH-CONDITIONS` against `origin/main@946ca39`; accepted prompt-provided PROMPT 1277 disposition because no durable tracked smoke report existed. Recorded the missing PROMPT 1277 artifact as a documentation/evidence condition. |
| 2 | PROMPT 1279 closeout | Closed Sprint 17 `closed-with-conditions` on top of (1)'s smoke basis at `origin/main@946ca39`. |
| 3 | PROMPT 1277 durable report (post-hoc) | Showed FAIL on `cargo fmt --check`; other smoke segments PASS/PASS-WITH-WARNINGS. |
| 4 | PROMPT 1281 | Repaired the rustfmt drift surfaced by (3). |
| 5 | PROMPT 1282 | Verified the fmt repair locally on top of the PROMPT 1279 closeout. |
| 6 | PROMPT 1283 | Landed the PROMPT 1281 fmt repair on `origin/main` as `d73e25e49519a214f8fb0fefa1e78351ccd74795`. |
| 7 | PROMPT 1284 | Reran Sprint 17 smoke from `origin/main@d73e25e` in a clean worktree under the Windows/MSVC Cargo resource policy. Result: PASS across the board. |

PROMPT 1278's smoke basis was therefore **stale** (it predated PROMPT 1281's
fmt repair and lacked a durable tracked PROMPT 1277 artifact). PROMPT 1284 is
now the durable post-fmt smoke of record for Sprint 17.

## Source of truth

| Field | Value |
|---|---|
| Refresh date | 2026-05-18 |
| Refresh prompt | 1288 |
| `origin/main` HEAD at refresh | `d73e25e49519a214f8fb0fefa1e78351ccd74795` |
| `origin/main` HEAD subject | `PROMPT-1281: repair cargo fmt drift` |
| Refreshed Team-QA artifact | `production/qa/team-qa-sprint-17-2026-05-18-post-fmt-refresh.md` (this file) |
| Original Team-QA artifact (review of record) | `production/qa/team-qa-sprint-17-2026-05-18.md` (unmodified) |
| Original Team-QA report | `reports/PROMPT-1278-sprint-17-team-qa.md` (unmodified) |
| Closeout artifact | `production/sprint-status.yaml#sprint_17_closeout[date=2026-05-18, prompt=1279]` (unmodified) |
| Sprint 17 disposition | `closed-with-conditions` (UNCHANGED by this refresh) |
| Stage | `Polish` (UNCHANGED) |
| Cargo by PROMPT 1288 | **None.** PROMPT 1284 is the smoke evidence. This refresh is paperwork/review-of-record only. |

PROMPT 1288 did **not** invoke `cargo` or `trunk`, did **not** modify
`production/sprint-status.yaml`, did **not** modify any closeout banner,
did **not** modify any sprint plan or stage file, and did **not** modify any
production code, tests, Cargo files, CI, or assets.

## Refreshed smoke evidence (PROMPT 1284 — durable, post-fmt)

`reports/PROMPT-1284-sprint-17-post-fmt-smoke-rerun.md` is the durable
post-fmt smoke evidence of record for Sprint 17 Team-QA. Summary:

| Command | Result | Notes |
|---|---|---|
| `cargo fmt --check` | **PASS** (exit 0) | Confirms PROMPT 1281 fmt repair is on `origin/main@d73e25e`. |
| `cargo test -p client --test board_rendering_resolution_combat_feedback_test` | **PASS** | 7/7 tests passed. |
| `cargo test -p client --test shop_auction_ui_plugin_scaffold_formulas_test` | **PASS** | 8/8 tests passed. |
| `cargo test -p client --test ui_clean_pass_button_vs_chip_lint_test` | **PASS** | 5/5 tests passed. |
| `cargo test -p dev-launcher-app` | **PASS** | 31/31 tests passed; zero `#[ignore]`. |
| `cargo check --workspace --all-targets` | **PASS** (exit 0) | Compiles cleanly; deprecation warnings only (see classification). |
| `cargo test --workspace --tests --no-fail-fast` | **PASS** (exit 0) | **263 suites · 1861 tests passed · 0 failed · 0 ignored.** Full log: `reports/PROMPT-1284-workspace-tests.log` (in PROMPT 1284 worktree). |
| `rg -n '^\s*#\s*\[\s*ignore' --glob '*.rs' .` | **PASS** | Zero active `#[ignore]` markers across the worktree. |

Windows/MSVC Cargo resource policy applied by PROMPT 1284:

```
CARGO_TARGET_DIR=D:\_DEV\cargo-target\ccgs-msvc
CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0
CARGO_INCREMENTAL=0
RUSTFLAGS=-C debuginfo=0 -C link-arg=/DEBUG:NONE
```

D: free-space preflight: 494 GiB free (threshold 40 GiB cleared by >12x;
no target cleanup performed).

### Warning classifications (advisory; not regressions)

- **Deprecation warnings**: `client` lib emits ~84 deprecation warnings on
  `ShopAuctionUiEntity` (and a handful on `HudEntity`). These are
  authored, documented, time-boxed deprecations retained for one Sprint
  cycle so historical PROMPT 1022 / 1034 / 1036 QA-snapshot comparisons
  still resolve (SOURCE-1077-08). New code is steered toward per-sub-surface
  root markers (`ShopAuctionPanelRoot::{DraftOffering, Shop, Auction,
  ShopFooter, Toast, SettlementOverlay}` and `Hud{TopStrip, BottomStrip,
  ScoreboardDot, DimOverlay}Root`). **Not smoke regressions.**
- **Workspace test harness artifact**: PROMPT 1284's first full-workspace
  invocation through a `tee | tail` pipeline was OOM-killed at the wrapping
  shell (exit 137) because buffered stdout grew large; the same Cargo
  command re-run writing stdout/stderr directly to a file completed cleanly
  with exit 0 and 1861/1861 passing. **Harness/pipe artifact, not a smoke
  failure.**

## Sprint 17 row state (preserved from PROMPT 1278 / PROMPT 1279)

At `origin/main@d73e25e`, the `sprint_17_closeout[date=2026-05-18,
prompt=1279]` block in `production/sprint-status.yaml` records:

| Row | Priority | Status |
|---|---|---|
| `S11-HUD-TIMER-EYEBALL-VISUAL-001` | Must | `ready` — human-operator-blocked carry |
| `S17-UI-CARD-DISPLAY-ART-HELPER-001` | Must | `done` |
| `S17-UI-HUD-OPP-MANA-CLEANUP-001` | Should | `in_progress` — parent-row paperwork gap carried |
| `S17-UI-CARD-SLOT-INSET-WIRING-001` | Should | `done` |
| `S17-UI-QA-SNAPSHOT-MARKER-SPLIT-001` | Should | `done` |
| `S17-UI-BID-BUTTON-PHASE-RACE-001` | Should | `done` |
| `S17-OPS-VULKAN-VALIDATION-GATING-001` | Nice | `done` |
| `S17-SERVER-START-OF-TURN-DEBUG-001` | Nice | `done` |
| `S17-UI-HAND-B0004-CLEANUP-001` | Nice | `done` |

Row counts: 7 done / 1 in_progress carried / 1 ready human-operator-blocked
carried (total 9). This refresh does NOT flip, close, or otherwise modify
any row status. The two carried rows remain carried.

## What this refresh changes vs. PROMPT 1278

| Aspect | PROMPT 1278 Team-QA | PROMPT 1288 refresh |
|---|---|---|
| Source under review | `origin/main@946ca39` | `origin/main@d73e25e` |
| Smoke basis | Prompt-provided PROMPT 1277 disposition + local rerun artifacts (no durable tracked report) | **Durable** PROMPT 1284 post-fmt smoke rerun (`reports/PROMPT-1284-sprint-17-post-fmt-smoke-rerun.md`) |
| `cargo fmt --check` | Not separately evidenced; latent FAIL revealed later by durable PROMPT 1277 report | **PASS** (verified post-fmt at `d73e25e`) |
| Workspace tests | Not run by PROMPT 1278 | **PASS** 1861/1861, 0 failed, 0 ignored |
| Targeted tests | Not run by PROMPT 1278 | **PASS** on board-rendering, shop-auction prepool, ui-clean-pass lint, dev-launcher |
| Verdict | `APPROVED-WITH-CONDITIONS` | `APPROVED-WITH-CONDITIONS` (unchanged; carried conditions remain) |
| Documentation-gap condition (missing PROMPT 1277 artifact) | Recorded as open | **Discharged** — PROMPT 1284 is now the durable post-fmt smoke artifact on disk; durable PROMPT 1277 report is no longer the gating evidence. |
| Sprint state | Sprint 17 then `active` | Sprint 17 `closed-with-conditions` (PROMPT 1279, **not touched by this refresh**) |

## Conditions (carried forward; not closed by this refresh)

All conditions from the PROMPT 1278 Team-QA review and the PROMPT 1279
closeout are preserved verbatim. This refresh re-states them so no carried
no-claim is silently dropped:

1. **Sprint 17 remains `closed-with-conditions`** per PROMPT 1279. This
   refresh does **not** reopen Sprint 17, **not** alter the closeout
   block, **not** modify `production/sprint-status.yaml`, and **not**
   change any row status.
2. **`S11-HUD-TIMER-EYEBALL-VISUAL-001`** remains **`ready` and
   human-operator-blocked**; no LLM `/dev-story` and no LLM `/story-done`
   is authorized. Closure requires real human screenshot evidence across
   the `DraftInitial` 45 s, `DraftShop` 30 s, and `Placement` 10–12 s
   timer phases. Allowed to carry to Sprint 18 if no human-operator slot
   opens in Sprint 17 timebox. **HUD timer human evidence remains
   required and unclosed.**
3. **`S17-UI-HUD-OPP-MANA-CLEANUP-001`** remains **`in_progress`**. The
   AC3 hand-reserve microbadge source repair is on `origin/main` via
   `c842668`, but no final `/story-done` paperwork closed the Sprint 17
   parent row. PROMPT 1279 explicitly did not silently mark it `done`;
   PROMPT 1288 preserves that explicit non-closure.
4. **PROMPT 761 Polish → Release gate-check FAIL** preserved verbatim at
   `production/gate-checks/gate-polish-release-2026-05-12.md`. No retry
   attempted by this refresh; no retry in scope for Sprint 17.
5. **Stage remains `Polish`.** No Sprint 18 activation. No release-readiness
   claim. No release-candidate claim. No full-game claim.
6. **`S8-QA-001-W1`** remains OPEN (manual / browser two-client GAME_OVER
   gap).
7. **`QA-COND-0005`** remains accepted-risk / friend-game scope only
   (Standard-tier accessibility not pursued).
8. **`QA-COND-0006`** remains accepted-risk / deferred (playtest /
   fun-hypothesis validation not advanced).
9. **`PAW-TD-*-a`** placeholder-art accept-risk across PAW-002..PAW-006
   preserved verbatim (placeholder PNGs only; real-art deferred to
   Sprint 18+).
10. **`TQ-S12-C1..C7`** preserved verbatim. `TQ-S12-C2` binding (no third
    same-scope retest of Sprint 12 `hand-ui/story-019-drag-runtime-
    retest-tighter-capture.md`); `TQ-S12-C7` AppCompat informational
    condition not closed.
11. **PROMPT 683-era runtime divergence question** preserved (folded into
    Sprint 12 story 019 `closed-with-conditions / cannot-reproduce`).
    Underlying drag-runtime bug not claimed fixed.
12. **PROMPT 1054 P1 UI snapshot visual retest** remains
    `BLOCKED-HUMAN-OPERATOR`.
13. **All prior Sprint 10 / 11 / 12 / 13 / 14 / 15 / 16 dispositions**
    (`closed-with-conditions` per PROMPT 763 / 792 / 817 / 894 / 987 /
    1056 / 1082+1088) preserved unchanged.
14. **24 PROMPT 1022 QA snapshot audit findings** preserved as report-only
    inputs to future story authoring; none claimed closed by Sprint 17 or
    by this refresh.
15. **Long-tail PROMPT 1076 findings** (AUDIT-1076-05 / 08 / 11) remain
    Sprint 18+ candidates.
16. **Long-tail PROMPT 1077 findings** (SOURCE-1077-05 / 07 / 11 / 12 / 13
    / 14 / 15) deferred to Sprint 18+.
17. **PROMPT 1284 smoke-warning carry from PROMPT 1278** preserved as
    historical context: the local PROMPT 1277-era reruns logged
    `hand_ui_phase_transition_auto_submit_short_circuit` /
    `invalid_submit_state` at one round-1 `Placement → Resolution`
    transition and `RSM disconnect timer breach: grace window exceeded`
    after a later client disconnect in `DraftShop`. These warnings did
    not recur in the PROMPT 1284 automated smoke (which exercises tests
    + `cargo check`, not a multi-client browser session); they remain
    Sprint 18+ candidates for runtime-behavior follow-up.
18. **Sprint 17 carried/partial rows** (rows 1 and 3 above) remain
    carried/partial in their original carry posture.

## Non-claims (explicit; preserved verbatim from PROMPT 1278 and PROMPT 1279)

PROMPT 1288 does NOT claim:

- Sprint 17 reopening.
- Sprint 17 status flip (no change to `closed-with-conditions`).
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
- closure of any remaining PROMPT 1022, PROMPT 1076, or PROMPT 1077
  finding outside concrete repairs already on `origin/main`.

## Recommendation

The PROMPT 1278 Team-QA verdict (`APPROVED-WITH-CONDITIONS`) and the
PROMPT 1279 closeout disposition (`closed-with-conditions`) for Sprint 17
**stand**. The single documentation/evidence condition recorded by
PROMPT 1278 (missing durable PROMPT 1277 smoke artifact) is **discharged**
by PROMPT 1284 — the durable post-fmt smoke of record at
`reports/PROMPT-1284-sprint-17-post-fmt-smoke-rerun.md` against
`origin/main@d73e25e` shows PASS on `cargo fmt --check`, targeted tests,
workspace check, full workspace tests (1861/1861), and the ignored-marker
scan.

All other carried conditions remain open and must continue to be honored
by downstream prompts. **No Sprint 18 activation, no Polish → Release
retry, no release claim is unlocked by this refresh.**

## Files written by PROMPT 1288

- `production/qa/team-qa-sprint-17-2026-05-18-post-fmt-refresh.md` (this file).
- `reports/PROMPT-1288-sprint-17-post-fmt-team-qa-refresh.md` (final report copy).
- `reports/PROMPT-1288-sprint-17-post-fmt-team-qa-refresh.summary.txt` (relay summary).

No production code, tests, sprint status, sprint plan, stage file,
gate-check, release artifact, Sprint 18 activation file, session-state
file, or Cargo/Trunk/CI file was modified by PROMPT 1288.

1288: SPRINT-17-POST-FMT-TEAM-QA-REFRESH: APPROVED-WITH-CONDITIONS
