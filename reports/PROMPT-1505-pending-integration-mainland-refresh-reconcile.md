# PROMPT 1505 — PENDING-INTEGRATION-MAINLAND-REFRESH-RECONCILE

## Summary

Reconciled 11 pending integration branches onto a fresh worktree rooted at the
current `origin/main`, producing one strict-fast-forwardable integration branch
ready for future `MAINLAND_ENQUEUE`.

## Identity

| Field | Value |
|---|---|
| Base SHA | `1a866f41bcd260cd39bccbe3b329208175aa2a27` (`origin/main`) |
| Final branch | `integrate/pending-krosmaga-ui-tooling-refresh-1505` |
| Final commit | `f3d41e68dc256014326e4df5a1c647ea6af7eb3e` |
| Worktree path | `D:/Tmp/ccgs-pending-refresh-1505` |
| Strict fast-forwardable from `origin/main` | YES — `git merge-base --is-ancestor origin/main HEAD` returned true |
| Ready for future `MAINLAND_ENQUEUE` | YES |
| Pushed to remote | YES (`origin/integrate/pending-krosmaga-ui-tooling-refresh-1505`) |

## Source branches consumed

| PROMPT | Source branch (input) | Source HEAD | Cherry-picked commits |
|---|---|---|---|
| 1481 | `origin/integrate/result-screen-hero-accounting-1481` | `ab3f8171` | `4d0f7443` (ab3f8171 became empty after rebase onto current main; skipped) |
| 1484 | `origin/integrate/dev-proxy-pack-validator-tooling-1484` | `87d950ce` | `ba715e6b`, `87d950ce` |
| 1493 | `origin/integrate/resolution-event-visual-replay-story-1493` | `cc093550` | `cc093550` |
| 1494 | `origin/integ/krosmaga-proxy-logical-id-map-stage1-1494` | `3fcb1b8b` | `3fcb1b8b` |
| 1495 | `origin/integrate/lobby-class-identity-confirm-cta-1495` | `691215d0` | `69e403b2`, `691215d0` |
| 1496 | `origin/integration/PROMPT-1496-shop-auction-polish` | `4395e98e` | `4395e98e` |
| 1497 | `origin/integrate/hud-edge-chrome-phase-timer-1497` | `a961926f` | `a961926f` |
| 1499 | `origin/integration/PROMPT-1499-board-play-area-physicality` | `ad216f3e` | `ad216f3e` |
| 1500 | `origin/integration/PROMPT-1500-qa-snapshot-1486` | `987f5cb0` | `7fb6dca7`, `987f5cb0` (allowlist matches task spec exactly) |
| 1502 | `origin/integration/PROMPT-1502-hand-fan-readability-1490` | `6307984f` | `6307984f` |
| 1503 | `origin/integrate/shared-card-inspect-zoom-primitive-1503` | `1d78d90d` | `1d78d90d` |

## Final commits on branch (oldest → newest)

```
a93d913d PROMPT-1481 result screen hero/accounting Krosmaga polish
380d15b3 PROMPT-1484 add dev proxy pack validator
f22b4c56 PROMPT-1504 integration report for dev proxy pack validator tooling
e6221ba8 PROMPT-1485 author resolution replay mutation story
3854991e docs: add Krosmaga proxy logical ID map stage 1
b12bdf82 PROMPT-1487 lobby class identity panel + Confirm CTA Krosmaga polish
545c57c8 PROMPT-1495 lobby class identity + confirm CTA Krosmaga polish integration report
98a97a41 PROMPT-1491 shop/auction/draft card product polish
f958f2b3 PROMPT-1488 HUD edge chrome + phase timer Krosmaga polish
391caf92 PROMPT-1489 board play-area physicality (Krosmaga polish)
9f62e9f3 PROMPT-1486 qa_snapshot: debug_grid + placement_lifecycle + pointer fields (recovery)
b604911d PROMPT-1500 integration report for qa_snapshot 1486 cherry-pick
8ee4f186 PROMPT-1490 hand fan readability + playable-affordance Krosmaga polish
f3d41e68 PROMPT-1482 shared card inspect primitive
```

## Files changed vs `origin/main` (grouped by source PROMPT)

### PROMPT 1481 — result-screen hero/accounting Krosmaga polish
- `client/Cargo.toml` (added `[[test]] result_screen_hero_accounting_polish_test`)
- `client/src/presentation/result_screen.rs`
- `tests/integration/presentation/result_screen_hero_accounting_polish_test.rs`
- `reports/PROMPT-1481-result-screen-hero-accounting-krosmaga-polish.md`

### PROMPT 1484 — dev proxy pack validator tooling
- `tools/asset-provenance/README.md`
- `tools/asset-provenance/fixtures/dev-proxy-pack-clean.json`
- `tools/asset-provenance/fixtures/dev-proxy-pack-release-claim.json`
- `tools/asset-provenance/fixtures/dev-proxy-pack-repo-assets-source.json`
- `tools/asset-provenance/test_validate_dev_proxy_pack.py`
- `tools/asset-provenance/validate_dev_proxy_pack.py`
- `reports/PROMPT-1484-dev-proxy-pack-validator-tooling.md`
- `reports/PROMPT-1504-dev-proxy-pack-validator-tooling-integration.md`

### PROMPT 1493 — resolution event visual replay story
- `production/epics/board-rendering/EPIC.md`
- `production/epics/board-rendering/story-015-resolution-event-visual-replay-mutation.md`
- `reports/PROMPT-1485-resolution-event-visual-replay-mutation-story.md`

### PROMPT 1494 — krosmaga proxy logical ID map stage 1
- `design/assets/provenance/krosmaga-proxy-logical-id-map-stage1.md`
- `reports/PROMPT-1483-krosmaga-proxy-logical-id-map-stage1.md`

### PROMPT 1495 — lobby class identity panel + confirm CTA
- `client/src/ui/lobby.rs`
- `tests/integration/playable_client/lobby_class_picker_layout_test.rs`
- `tests/integration/playable_client/lobby_room_browser_test.rs`
- `reports/PROMPT-1495-lobby-class-identity-confirm-cta-krosmaga-polish-integration.md`

### PROMPT 1496 — shop/auction/draft card product polish
- `client/src/ui/shop_auction/mod.rs`

### PROMPT 1497 — HUD edge chrome + phase timer Krosmaga polish
- `client/src/ui/hud/mod.rs`
- `client/src/ui/mod.rs` (phase_banner re-export block — preserved alongside 1503)
- `client/src/ui/phase_banner.rs`
- `tests/integration/hud/hud_phase_timer_countdown_test.rs`
- `tests/integration/ui_clean_pass/phase_banner_test.rs`

### PROMPT 1499 — board play-area physicality
- `client/src/presentation/board_rendering.rs`
- `client/src/presentation/board_rendering/rendering_constants.rs`
- `tests/unit/board_rendering/board_grid_camera_test.rs`
- `tests/unit/board_rendering/status_icons_test.rs`

### PROMPT 1500 — qa_snapshot debug_grid + placement_lifecycle + pointer fields
- `client/src/presentation/qa_snapshot.rs`
- `tests/integration/qa_snapshot/placement_auction_state_field_coverage_test.rs`
- `tests/unit/qa_snapshot/auction_won_pending_test.rs`
- `reports/PROMPT-1500-qa-snapshot-debug-grid-pointer-lifecycle-fields-integration.md`

Exact match to the task allowlist for PROMPT 1500.

### PROMPT 1502 — hand fan readability + playable-affordance
- `client/src/ui/hand/mod.rs`
- `tests/integration/hand-ui/hand_ui_chrome_composition_test.rs`
- `tests/unit/hand-ui/fan_layout_formula_test.rs`

### PROMPT 1503 — shared card inspect zoom primitive
- `client/src/ui/card_inspect.rs` (new)
- `client/src/ui/mod.rs` (`pub mod card_inspect;` — preserved alongside 1497)
- `reports/PROMPT-1482-shared-card-inspect-zoom-primitive.md`

## Conflicts and resolutions

### `client/src/ui/mod.rs` — PROMPT 1497 ↕ PROMPT 1503
Both touched this file but on **disjoint regions**:
- 1497 expanded the `pub use phase_banner::{ ... };` re-export block to include
  the new color/min-height constants (`PHASE_BANNER_BACKGROUND_COLOR`,
  `PHASE_BANNER_BORDER_COLOR`, `PHASE_BANNER_MIN_HEIGHT_PX`,
  `PHASE_BANNER_TEXT_COLOR`).
- 1503 added `pub mod card_inspect;` at the top of the module list.

`git cherry-pick` auto-merged the 1503 commit cleanly on top of 1497
(`Auto-merging client/src/ui/mod.rs`, no conflict markers). Both contributions
preserved — verified by reading the merged file: lines 2 (`pub mod card_inspect;`)
and 29–34 (full extended phase_banner re-export) both present.

### `client/Cargo.toml` — PROMPT 1481
The PROMPT 1481 cherry-pick added one `[[test]]` block
(`result_screen_hero_accounting_polish_test`) after the existing
`result_screen_return_to_lobby_test` entry. All existing test target additions
already on current `origin/main` preserved (lines 432–437 unchanged); the new
block sits at lines 441–442.

### Skipped empty commit
The PROMPT 1481 source branch had a duplicate commit (`ab3f8171`) which was a
re-cherry-pick of `4d0f7443` onto a different base. After applying `4d0f7443`
onto current `origin/main`, `ab3f8171` was reported as empty by cherry-pick and
skipped via `git cherry-pick --skip`. Net content is identical.

### No other conflicts
All other branches touched disjoint files. No textual conflicts during the
11-branch cherry-pick chain.

## Stale state-file scrub

Task spec required not carrying stale
`production/session-state/codex-orchestrator-state.md`, `AGENTS.md`, or
`CODEX.md` diffs from old-base branches. **Verified clean**: none of the 11
source branches' diffs against their respective merge-bases contained any of
those three files. Final `git diff --name-only origin/main..HEAD` confirms only
intended scope.

## Validation

| Check | Result |
|---|---|
| `git diff --check origin/main..HEAD` | PASS (no whitespace / conflict-marker warnings) |
| Path allowlist review per source branch | PASS (every file traces to a single intended PROMPT scope; matrix above) |
| `client/src/ui/mod.rs` merge preserves 1497 + 1503 | PASS (verified by file read) |
| `client/Cargo.toml` preserves existing main test targets + 1481 addition | PASS (lines 432–442) |
| PROMPT 1500 allowlist exact match | PASS (4 files, 0 extras) |
| Strict fast-forwardable from `origin/main` | PASS |
| Branch pushed to remote | PASS (`origin/integrate/pending-krosmaga-ui-tooling-refresh-1505`) |
| Optional cheap non-Cargo validation: `python -m pytest tools/asset-provenance/test_validate_dev_proxy_pack.py -q` | PASS (14/14) |
| Broad Cargo / workspace tests | NOT RUN per task spec (no Cargo bottleneck) — VERIFY lane deferred |

## Cargo validation status

Per task spec: "No broad Cargo or workspace tests. Do not bottleneck on Cargo."
Cargo validation **deferred to VERIFY lane** — source branches each carried
their own focused Cargo evidence; this refresh is path-allowlist + diff-check
gated only.

## Future `MAINLAND_ENQUEUE` readiness

| Criterion | Status |
|---|---|
| Strict fast-forwardable from current `origin/main` | YES |
| Single contiguous commit chain | YES (14 commits) |
| All commits author-attributed to the original source-branch author | YES (cherry-pick preserved authors / dates) |
| No off-scope diffs (orchestrator-state, AGENTS, CODEX, build configs) | YES |
| `git diff --check` clean | YES |
| Ready to enqueue once `gcs.dispatch` / `MAINLAND_ENQUEUE` is exposed | YES |

## Reproduction

```bash
git fetch origin
git worktree add -b integrate/pending-krosmaga-ui-tooling-refresh-1505 \
  D:/Tmp/ccgs-pending-refresh-1505 origin/main
cd D:/Tmp/ccgs-pending-refresh-1505
git cherry-pick 4d0f7443                # PROMPT 1481 (ab3f8171 skipped empty)
git cherry-pick ba715e6b 87d950ce       # PROMPT 1484 + 1504 report
git cherry-pick cc093550                # PROMPT 1493
git cherry-pick 3fcb1b8b                # PROMPT 1494
git cherry-pick 69e403b2 691215d0       # PROMPT 1495 (1487 work + 1495 report)
git cherry-pick 4395e98e                # PROMPT 1496
git cherry-pick a961926f                # PROMPT 1497
git cherry-pick ad216f3e                # PROMPT 1499
git cherry-pick 7fb6dca7 987f5cb0       # PROMPT 1500 (1486 work + 1500 report)
git cherry-pick 6307984f                # PROMPT 1502
git cherry-pick 1d78d90d                # PROMPT 1503
git diff --check origin/main..HEAD
git merge-base --is-ancestor origin/main HEAD && echo FF-OK
git push -u origin integrate/pending-krosmaga-ui-tooling-refresh-1505
```

## Final status

1505: PENDING-INTEGRATION-MAINLAND-REFRESH-RECONCILE: PASS
