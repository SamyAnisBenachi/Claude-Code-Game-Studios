# Sprint 14 Close-Out Disposition Evidence

**Prompt**: 987
**Date**: 2026-05-16
**Source-of-truth**: `origin/main@f6906020074f3d31e6594fb78788596bbac99477` (PROMPT 986 QA evidence integration tip)
**Verdict**: `closed-with-conditions` (NOT release-ready, NOT `closed`)
**Worktree**: `D:/_DEV/claude-code-game-studios-worktrees/s14-closeout-987`
**Branch**: `paperwork/sprint-14-closeout-987`
**Stage at close-out**: `Polish` (UNCHANGED; `production/stage.txt` NOT modified)
**PROMPT 761 Polish→Release gate-check**: `FAIL` (preserved at `production/gate-checks/gate-polish-release-2026-05-12.md`; NO retry)

## Sprint 14 row-by-row disposition

### Must Have (9 / 9 done)

| ID | Story file | Status | Worker / Integration / /story-done |
|---|---|---|---|
| `S11-TD-UI-ZINDEX-LAYERS` | `production/epics/ui-clean-pass/story-002-ui-zindex-layers.md` | done | PROMPT 899 `8669982` / PROMPT 902 `36c0b4b` / PROMPT 903 |
| `S11-TD-UI-FONT-CONSTANTS` | `production/epics/ui-clean-pass/story-003-ui-font-constants.md` | done | PROMPT 904 `aa1672b` / PROMPT 906 `eb1c128` / PROMPT 908 |
| `S11-TD-UI-FLEX-STRIPS` | `production/epics/ui-clean-pass/story-004-ui-flex-strips.md` | done | PROMPT 915 `cae2f75` / PROMPT 918 `6ab4a27` / PROMPT 919 |
| `S11-TD-UI-VIEWPORT-INVARIANT-TESTS` | `production/epics/ui-clean-pass/story-005-ui-viewport-invariant-tests.md` | done | PROMPT 905 `9234700` / PROMPT 907 `42eae31` / PROMPT 909 |
| `S12-TD-UI-OVERLAY-ALPHA-TOKEN-001` | `production/epics/ui-clean-pass/story-006-ui-overlay-alpha-token.md` | done | PROMPT 916 `837a611` / PROMPT 917 `c4e1936` / PROMPT 921 |
| `S12-UX-GLOBAL-UI-DESIGN-SPEC-001` | `production/epics/ui-clean-pass/story-007-global-ui-design-spec.md` | done | PROMPT 911 `f4ef52a` / PROMPT 912 `3d99a04` / PROMPT 922 |
| `S11-UX-HUD-TOP-STRIP-LAYOUT` | `production/epics/hud/story-015-hud-top-strip-layout.md` | done | PROMPT 940 `ea92597` / PROMPT 941 `4b9a23b` / PROMPT 942 |
| `S11-UX-AUCTION-FEATURED-CARD` | `production/epics/shop-auction-ui/story-016-auction-featured-card.md` | done | PROMPT 928 `1ddc372` / PROMPT 930 `b828587` / PROMPT 931 |
| `S12-UX-LOBBY-LAYOUT-MODAL-001` | `production/epics/playable-client/story-024-lobby-layout-modal.md` | done | PROMPT 937 `2ad29c9` / PROMPT 938 `c25aba7` / PROMPT 939 (PROMPT 933/935 producer-decision-3 Option A capture + PROMPT 936 readiness rerun preceded /dev-story) |

### Should Have (3 / 4 done — 1 human-operator-blocked carry)

| ID | Story file | Status | Worker / Integration / /story-done |
|---|---|---|---|
| `S11-HUD-TIMER-EYEBALL-VISUAL-001` | `production/epics/hud/story-014-hud-timer-eyeball-visual-check.md` | **ready (human-operator-blocked)** | Cosmetic visual check requires human-operator screenshot capture across `DraftInitial 45s` / `DraftShop 30s` / `Placement 10-12s` phases per story file ACs. Cannot be auto-closed by an LLM `/story-done`. **Carried forward into Sprint 15 planning as Sprint 13 → Sprint 14 → Sprint 15 carry; originally Sprint 10 smoke retry-7 W2.** Story authored by PROMPT 822; `/story-readiness` READY per PROMPT 823 batch and unchanged. Evidence target path `production/qa/evidence/sprint-14-hud-timer-visual-check/` (NEW) remains unpopulated. |
| `S11-UX-HUD-BOTTOM-STRIP-LAYOUT` | `production/epics/hud/story-016-hud-bottom-strip-layout.md` | done | PROMPT 954 `acfc438` / PROMPT 955 `45c2d03` / PROMPT 956 |
| `S11-UX-DRAFT-GRID-CENTERED-MODAL` | `production/epics/shop-auction-ui/story-015-draft-grid-centered-modal.md` | done | PROMPT 946 `0b3ef30` / PROMPT 951 `a9721bc` / PROMPT 953 |
| `S11-UX-LOBBY-CLASS-PICKER` | `production/epics/playable-client/story-025-lobby-class-picker-layout.md` | done | PROMPT 957 `3344348` / PROMPT 961 `fed5fb9` / PROMPT 962 |

### Nice to Have (4 / 4 done)

| ID | Story file | Status | Worker / Integration / /story-done |
|---|---|---|---|
| `S11-UX-HUD-OPP-FIGURINE` | `production/epics/hud/story-017-hud-opponent-figurine.md` | done | PROMPT 968 `69f8136` / PROMPT 975 `a3bc885` / PROMPT 976 |
| `S11-UX-AUCTION-FREE-GOLD-COUNTERS` | `production/epics/shop-auction-ui/story-017-auction-free-gold-counters.md` | done | PROMPT 958 `8a91b18` / PROMPT 959 `5f5e72f` / PROMPT 960 |
| `S11-UX-LOBBY-BUTTON-HITTARGETS` | `production/epics/playable-client/story-026-lobby-button-hittargets.md` | done | PROMPT 966 `fd0ec22` / PROMPT 970 `2e0715f` / PROMPT 972 |
| `S12-UX-AUCTION-LEAD-LOSS-STATE-001` | `production/epics/shop-auction-ui/story-018-auction-lead-loss-state.md` | done | PROMPT 971 `bef153a` / PROMPT 973 `e3ca5d6` / PROMPT 974 (PROMPT 967 producer-decision-4 Option A capture preceded readiness rerun + `/dev-story`) |

### Totals

- **Must Have**: 9 / 9 done
- **Should Have**: 3 / 4 done (1 open: `S11-HUD-TIMER-EYEBALL-VISUAL-001`, human-operator-blocked)
- **Nice to Have**: 4 / 4 done
- **Total**: **16 / 17 rows closed**

## Smoke summary (PROMPT 983 rerun)

| Field | Value |
|---|---|
| File | `production/qa/smoke-sprint-14-2026-05-16-rerun.md` (integrated on `origin/main` per PROMPT 986 at `f690602`) |
| Verdict | **PASS-WITH-WARNINGS** |
| HEAD at smoke entry | `f94f4893cae3690372c5a12f81145de42bb4d94e` (PROMPT 982 UI drift repair integration tip) |
| Cargo aggregate | **213 binaries / 1350 passed / 0 failed / 0 ignored / 0 measured / 0 filtered** |
| Functional total (with renamed-binary direct run for AppCompat-blocked test) | **1355 passed / 0 failed / 0 ignored across 214 effective binaries** |
| PROMPT 978/979 targeted reruns | `shop_auction_ui_plugin_scaffold_formulas_test` **8/8 PASS**, `ui_clean_pass_z_layers_test` **6/6 PASS** |
| Warning | Single environment/tool warning: Windows AppCompat heuristic blocks spawn of `spawn_range_live_update_contract-*.exe` (OS error 740 — installer-detection on `update` substring). Workaround verified across 5 consecutive runs with renamed binary: **5/5 PASS each**. Identical classification to PROMPT 815 / 790 / 979 / 982. No code regression. |
| Pre-existing dead-code warning | `count_with_image_node` at `tests/integration/presentation/hand_ui_asset_wiring_test.rs:43` — not introduced by Sprint 14; out of Sprint 14 scope. |

## Team-QA summary (PROMPT 984)

| Field | Value |
|---|---|
| File | `production/qa/team-qa-sprint-14-2026-05-16.md` (integrated on `origin/main` per PROMPT 986 at `f690602`) |
| Verdict | **APPROVED-WITH-CONDITIONS** |
| HEAD at Team-QA entry | `f94f4893cae3690372c5a12f81145de42bb4d94e` (same HEAD as smoke rerun) |
| Approval conditions | All carry conditions preserved verbatim — none closed. See "Carried conditions preserved" below. |

## Verification checks run by PROMPT 987

| # | Check | Result |
|---|---|---|
| 1 | `git fetch origin` | OK |
| 2 | `git rev-parse origin/main` | `f6906020074f3d31e6594fb78788596bbac99477` |
| 3 | `git rev-parse HEAD` (worktree base) | `f690602` (matches `origin/main`) |
| 4 | `git worktree add -b paperwork/sprint-14-closeout-987 D:/_DEV/claude-code-game-studios-worktrees/s14-closeout-987 origin/main` | OK |
| 5 | `production/stage.txt` | `Polish` (UNCHANGED; NOT modified by PROMPT 987) |
| 6 | `production/sprint-status.yaml` top-level | `sprint: 14`, `status: active` → flipped to `closed-with-conditions`, `stage: Polish` preserved |
| 7 | Sprint 14 row reconciliation | Must 9/9 done + Should 3/4 done (only `S11-HUD-TIMER-EYEBALL-VISUAL-001` ready) + Nice 4/4 done = 16/17 closed |
| 8 | PROMPT 761 Polish→Release gate-check FAIL | preserved at `production/gate-checks/gate-polish-release-2026-05-12.md` (NOT modified by PROMPT 987; NO retry) |
| 9 | Smoke evidence on main | `production/qa/smoke-sprint-14-2026-05-16-rerun.md` present at `f690602` (integrated by PROMPT 986) |
| 10 | Team-QA evidence on main | `production/qa/team-qa-sprint-14-2026-05-16.md` present at `f690602` (integrated by PROMPT 986) |
| 11 | 5-file paperwork edits applied | OK (sprint-status.yaml, sprint-14.md, active.md, codex-orchestrator-state.md, this NEW evidence file) |
| 12 | Forbidden-path check | no edit under `client/` / `server/` / `shared/` / `tests/`, `Cargo.*`, `.cargo/`, `.github/`, `production/stage.txt`, `production/qa/qa-plan-sprint-14.md`, `production/gate-checks/*` |
| 13 | `git diff --check` | will run before commit |
| 14 | `git diff --cached --check` | will run before commit |
| 15 | No release/RC/full-game claim text introduced | verified — every banner explicitly preserves the non-claims list |
| 16 | `git status --short --branch` (pre-commit) | will run before commit |
| 17 | `git commit` | will produce close-out commit on `paperwork/sprint-14-closeout-987` |
| 18 | `git push origin paperwork/sprint-14-closeout-987` | branch push; orchestrator integrates to `main` separately |

## Carried conditions preserved (verbatim — NONE closed by PROMPT 987)

1. **`S11-HUD-TIMER-EYEBALL-VISUAL-001`** — Sprint 14 Should Have row remains `ready` and human-operator-blocked. Sprint 13 → Sprint 14 → Sprint 15 carry. Closure requires real two-client run with screenshot capture across `DraftInitial 45s` / `DraftShop 30s` / `Placement 10-12s` phases; no LLM `/story-done` is authorised. Carried forward into Sprint 15 planning.
2. **`S8-QA-001-W1`** — manual / browser two-client GAME_OVER gap remains **OPEN**. Story 017 (two-client runtime harness; Sprint 13) AC12 forbid-auto-closure preserved through Sprint 14. No Sprint 14 row touched this surface.
3. **`QA-COND-0005`** — Standard-tier accessibility remains **accepted-risk** (friend-game scope). Sprint 14 UI clean-pass was friend-game visual polish only. The L5 `LOBBY_BUTTON_HEIGHT = 30.0` defect remains accepted-risk; story 026 was layout-stability work, not ≥44 px hit-target conformance.
4. **`QA-COND-0006`** — playtest / fun-hypothesis validation remains **accepted-risk / deferred**. No playtest sessions were required or run by any Sprint 14 row.
5. **`PAW-TD-*-a`** — placeholder-art accept-risk preserved across PAW-002..PAW-006. Story 016 auction featured card, story 017 HUD opponent figurine, and story 018 auction lead-loss differentiation were achieved by layout / composition / scale / typography / token color, NOT by final-art replacement.
6. **`TQ-S12-C1..C7`** — preserved verbatim. TQ-S12-C2 binding: no third same-scope retest of Sprint 12 story 019 is authorised by Sprint 14.
7. **PROMPT 683-era runtime divergence question** — preserved as folded into Sprint 12 story 019 `closed-with-conditions / cannot-reproduce`; cannot-reproduce disposition preserved.
8. **PROMPT 761 `Polish→Release` gate-check `FAIL`** — preserved at `production/gate-checks/gate-polish-release-2026-05-12.md`; **NO retry** attempted by any Sprint 14 row, and **NO retry** attempted by PROMPT 987.
9. **Sprint 12 story 019 underlying drag-runtime bug** — NOT claimed fixed.
10. **Sprint 13 / Sprint 12 / Sprint 11 / Sprint 10 closeouts** — preserved unchanged.
11. **`S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001`** backlog row — remains as-is (cited in S13-CONN-LOST-UX-001 evidence per AC8 design; row-status flip remains a separate paperwork prompt).
12. All 16 prior Sprint 14 `/story-done` closures preserved unchanged on `origin/main` (PROMPT 903 / 908 / 909 / 919 / 921 / 922 / 931 / 939 / 942 / 953 / 956 / 960 / 962 / 972 / 974 / 976).

## Explicitly NOT claimed by PROMPT 987 (non-claims)

- public release readiness
- release-candidate readiness
- full game completion
- broad / Standard-tier accessibility completion (`QA-COND-0005` unchanged)
- playtest / fun-hypothesis validation (`QA-COND-0006` unchanged)
- full playable-client manual QA (`S8-QA-001-W1` unchanged)
- two-client GAME_OVER closure (`S8-QA-001-W1` remains OPEN)
- final-art / asset-production completion (`PAW-TD-*-a` accept-risk preserved)
- Polish→Release gate-check retry (PROMPT 761 `FAIL` preserved)
- stage advance from Polish to Release (`production/stage.txt` NOT modified)
- closure of `S11-HUD-TIMER-EYEBALL-VISUAL-001` (human-operator-blocked carry; carried forward into Sprint 15 planning)
- closure of `S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001` backlog row
- TQ-S12-C7 closure
- underlying drag-runtime bug fix (Sprint 12 story 019 closed cannot-reproduce, NOT bug-fixed)
- full UI clean-pass repair beyond the 17 Sprint 14 candidate rows (Tier 2 / Tier 3 ranks remain out of scope)
- Sprint 15 activation (no `production/sprints/sprint-15.md` authored; no `sprint_15_*` block touched)
- any code change under `client/` / `server/` / `shared/` / `tests/` by PROMPT 987 (paperwork-only close-out)
- any change to `production/stage.txt`, `production/qa/qa-plan-sprint-14.md`, `production/gate-checks/*`, release artifacts, release-checklist, launch-checklist, changelog, or patch notes by PROMPT 987

## Cargo policy

N/A — paperwork-only close-out. No `cargo` / `trunk` command invoked by PROMPT 987.

## Files changed by PROMPT 987 (5 files; all paperwork)

- `production/sprint-status.yaml` — top-level `status:` flipped `active -> closed-with-conditions` + `updated:` annotation refreshed with PROMPT 987 prefix preserving PROMPT 976 narrative as `# Previous:` comment chain + `sprint_14_closeout:` block appended at end of file following `sprint_13_closeout:` / `sprint_12_closeout:` pattern.
- `production/sprints/sprint-14.md` — CLOSED-WITH-CONDITIONS banner prepended above prior PROMPT 897 ACTIVATED banner. Plan body NOT rewritten.
- `production/session-state/active.md` — PROMPT 987 close-out banner prepended above PROMPT 976 banner.
- `production/session-state/codex-orchestrator-state.md` — PROMPT 987 section prepended above PROMPT 976 section.
- `production/qa/evidence/sprint-14-close-out-disposition.md` — this file (NEW).

`reports/PROMPT-987-Sprint-14-Close-Out-Disposition.md` — mandatory final report file; NOT staged or committed; `reports/` is gitignored.

---

**`987: SPRINT-14-CLOSE-OUT-DISPOSITION: closed-with-conditions`**
