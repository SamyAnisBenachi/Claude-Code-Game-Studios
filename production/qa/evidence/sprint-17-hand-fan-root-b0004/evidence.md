# S17-UI-HAND-B0004-CLEANUP-001 — Worker Evidence (PROMPT 1115)

> Story: `production/epics/hand-ui/story-021-hand-fan-root-b0004-hierarchy.md`
> Worker branch: `work/s17-hand-fan-root-b0004-cleanup`
> Base: `origin/main@30c9e0f`
> Prompt: PROMPT 1115 (`/dev-story`)
> Date: 2026-05-18

## Summary

Strategy A applied. Inserted `Transform::default()` on the `HandBar`
strip entity spawned by `client::ui::hand::spawn_hand_ui`. The Bevy
0.18 Required Components API auto-derives `GlobalTransform` from
`Transform`, which silences the `B0004` hierarchy warning emitted on
every `InSession` entry per PROMPT 1076 `AUDIT-1076-14`.

Verified at the registry layer that `bevy_ui-0.18.1` `Node`'s
`#[require(...)]` set is `(ComputedNode, ComputedUiTargetCamera,
ComputedUiRenderTargetInfo, UiTransform, BackgroundColor, BorderColor,
FocusPolicy, ScrollPosition, Visibility, ZIndex)` — `Node` does NOT
require `Transform`/`GlobalTransform`. `bevy_transform-0.18.1`
`Transform` `#[require(GlobalTransform, TransformTreeChanged)]`
confirms the single-component-insert strategy is sufficient.

Fan layout, drag-state visuals, placement staging, the Sprint 12
story 019 `closed-with-conditions / cannot-reproduce` disposition,
and the PROMPT 1112 AC3 hand reserve-strip carry are all preserved
unchanged. No fan-layout numbers, drag-state behavior, or schedule
wiring were touched.

## Files changed

| Path | Reason |
|---|---|
| `client/src/ui/hand/mod.rs` | Strategy A: insert `Transform::default()` on `HandBar` spawn so Required Components derives `GlobalTransform`. |
| `client/Cargo.toml` | Register `hand_fan_root_b0004_hierarchy_test` `[[test]]`. |
| `tests/integration/hand-ui/hand_fan_root_b0004_hierarchy_test.rs` | AC4 integration test. |
| `production/qa/evidence/sprint-17-hand-fan-root-b0004/evidence.md` | This document. |

No files under `server/`, `shared/`, `protocol/`, `tests/integration/server/`,
`tests/unit/server/`, `production/sprint-status.yaml`,
`production/sprints/`, `production/stage.txt`,
`production/session-state/`, `production/qa/qa-plan-*.md`,
`production/qa/smoke-*.md`, `production/qa/team-qa-*.md`,
`production/gate-checks/`, or `docs/architecture/adr-*.md` were
touched (per AC6, AC7, AC9, AC10 and the Forbidden files list).

## AC checklist

- **AC1 — B0004 warning is gone (default build)**: covered by AC4
  integration test below; cross-machine smoke evidence is a later
  prompt's scope (this row does NOT claim smoke-pass closure).
- **AC2 — Hierarchy invariant holds (Strategy A)**: `HandBar` now
  carries `Transform::default()` → Required Components inserts
  `GlobalTransform` → AC2(a) holds. Asserted by the new test.
- **AC3 — Existing hand UI tests continue to PASS**: the adjacent
  hand UI integration / unit suite passes unchanged (run log below).
- **AC4 — Integration test asserts hierarchy invariant**: see
  `tests/integration/hand-ui/hand_fan_root_b0004_hierarchy_test.rs`.
  Asserts `HandFanRoot` has `GlobalTransform`, its `ChildOf` parent
  is the `HandBar` strip (matching the audit's exact edge), and
  the parent carries both `Transform` and `GlobalTransform`.
- **AC5 — Sprint 12 story 019 disposition preserved**: this commit
  explicitly does NOT claim closure of the drag-runtime question.
  The audit notes "no functional bug evident yet"; this row is ECS
  hygiene only. The `closed-with-conditions / cannot-reproduce`
  disposition per `TQ-S12-C2` is preserved verbatim.
- **AC6 — No protocol or server change**: `git diff` shows zero
  changes under `server/`, `shared/`, or `tests/integration/server/`.
  Implementation is client-side only.
- **AC7 — ADR-021 schedule preserved**: no new system-set, no
  schedule wiring change. The hand UI systems remain in their
  existing `PresentationSet` slots. `cargo check -p client` passes.
- **AC8 — No accept-risk closure claimed**: this commit makes NO
  claim on `S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006`,
  `PAW-TD-*-a`, the PROMPT 761 Polish->Release gate-check `FAIL`,
  the Sprint 12 story 019 / Sprint 11 story 018 retest questions,
  release-readiness, accessibility completion, playtest validation,
  final-art completion, two-client GAME_OVER closure, or stage
  advance.
- **AC9 — Sprint 17 disposition preserved**: `production/sprint-status.yaml`,
  `production/sprints/sprint-17.md`, `production/stage.txt`,
  `production/session-state/*`, `production/qa/qa-plan-*.md`,
  `production/qa/smoke-*.md`, `production/qa/team-qa-*.md`,
  `production/gate-checks/*`, and `docs/architecture/adr-*.md` are
  untouched by this worker.
- **AC10 — Worker branch scope contained**: worker branch
  `work/s17-hand-fan-root-b0004-cleanup` only. Files changed are
  scoped to `client/src/ui/hand/mod.rs`,
  `client/Cargo.toml` (test registration only — single `[[test]]`
  block, no profile / feature / dep changes), the new test bin
  under `tests/integration/hand-ui/`, and this evidence document.
  `main` is never pushed.
- **AC11 — Cargo resource policy applied**: every `cargo check` /
  `cargo test` invocation was run through a PowerShell script that
  sets `CARGO_TARGET_DIR=D:\_DEV\cargo-target\ccgs-msvc`,
  `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0`,
  `CARGO_INCREMENTAL=0`,
  `RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'` before
  invoking cargo. Build output line `Finished \`dev\` profile
  [optimized]` (no `+ debuginfo`) confirms the policy was applied.

## Blocking checks (run logs, abridged)

### `cargo check -p client`

```
    Checking client v0.1.0 (...)
    Finished `dev` profile [optimized] target(s) in 9.32s
```

PASS, zero warnings on touched files.

### `cargo test -p client --test hand_fan_root_b0004_hierarchy_test`

```
running 1 test
test hand_fan_root_parent_carries_global_transform ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured;
0 filtered out; finished in 0.02s
```

### Adjacent hand UI regression bins (all PASS)

| Bin | Result |
|---|---|
| `hand_ui_chrome_composition_test` | ok |
| `hand_ui_slot_onscreen_test` | ok |
| `hand_ui_viewport_sync_test` | ok |
| `hand_ui_drag_state_visuals_test` | ok |
| `hand_ui_drag_to_board_cell_test` | ok |
| `hand_ui_drag_end_non_instant_test` | ok |
| `hand_card_stat_label_rendering_test` | ok |
| `hand_ui_draft_initial_grid_test` | ok |
| `placement_entry_post_acquisition_test` | ok |
| `hand_ui_placement_perspective_snapshot_test` | ok |
| `hand_ui_placement_action_panel_test` | ok |
| `hand_ui_placement_unstaging_test` | ok |
| `hand_ui_placement_timer_test` | ok |
| `hand_ui_placement_drag_highlights_test` | ok |
| `hand_ui_placement_instant_staging_test` | ok |
| `hand_ui_placement_staged_disclosure_accessibility_test` | ok |
| `hand_ui_fan_layout_formula_test` | ok |
| `hand_ui_phase_state_machine_test` | ok |
| `hand_ui_plugin_scaffold_test` | ok |
| `hand_ui_reserve_mana_strip_test` | ok |
| `hand_ui_submit_prevalidation_test` | ok |
| `hand_ui_placement_submit_core_test` | ok |
| `card_display_art_chrome_preservation_test` | ok |
| `card_display_art_helper_test` | ok |

No new warnings on touched files. No fan layout / drag-state /
placement assertion was regressed (AC3).

### `git diff --check`

Clean (no whitespace / conflict-marker errors).

## Non-claims

This row does NOT claim closure of: `S8-QA-001-W1`, `QA-COND-0005`,
`QA-COND-0006`, `PAW-TD-*-a`, the PROMPT 761 Polish->Release
gate-check `FAIL`, the Sprint 12 story 019 drag-runtime
`closed-with-conditions / cannot-reproduce` disposition (per
`TQ-S12-C2`), the Sprint 11 story 018 retest, any AUDIT-1076-*
finding outside AUDIT-1076-14, any SOURCE-1077-* finding, any of
the 24 PROMPT 1022 audit findings, the PROMPT 1112 AC3 hand
reserve-strip carry (preserved), Sprint 17 close-out, release
readiness, accessibility completion, playtest validation, full
playable-client manual QA, two-client GAME_OVER closure, final-art
completion, stage advance, or any other accept-risk disposition.

Smoke validation, team-QA sign-off, story-done paperwork, and
sprint-state edits are explicitly out of scope for this `/dev-story`
worker per the AC9 / Worker Contract / Non-claims envelope.

## Worker reporting

```
1115: S17-UI-HAND-B0004-CLEANUP-001: PARTIAL
```

`PARTIAL` reflects that the implementation + integration test +
adjacent regression suite all pass at the owned-file scope (the
hierarchy invariant is verified in-test), but the cross-machine
smoke evidence that satisfies AC1's "Sprint 17 smoke" leg, the
story-done paperwork, and the sprint-state closure are all
explicitly carried for later prompts. The owned-file build gate
(per AC10 + story Build gate scope) is GREEN.
