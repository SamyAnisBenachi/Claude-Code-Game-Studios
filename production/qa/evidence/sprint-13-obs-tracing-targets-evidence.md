# Sprint 13 -- S13-OBS-TRACING-TARGETS-001 Evidence

> **Story**: `production/epics/playable-client/story-018-obs-tracing-targets.md`
> **Worker prompt**: PROMPT 847
> **Worker branch**: `work/s13-obs-tracing-targets`
> **Worktree**: `D:\_DEV\claude-code-game-studios-worktrees\s13-obs-tracing-targets`
> **Base**: `origin/main@fe74fb0` (PROMPT 844 closure commit)
> **Skills active**: `liv-bevy-018` (.rs edits), `liv-bevy-lightyear` (server
> network dispatch sites under `server/src/network/`)
> **Date**: 2026-05-14

---

## No-Claim Restatement

Verbatim from `production/epics/playable-client/story-018-obs-tracing-targets.md`
"Status / No-Claim Banner":

> This story is authored as a Sprint 13 candidate. ... PROMPT 804 (this
> authoring run) does NOT activate Sprint 13. ... This story does not
> claim: public release readiness, release-candidate readiness, full
> game completion, broad / Standard-tier accessibility completion
> (`QA-COND-0005`), playtest / fun-hypothesis validation
> (`QA-COND-0006`), full playable-client manual QA, two-client GAME_OVER
> closure (`S8-QA-001-W1`), or final-art / asset-production completion.
>
> Sprint 10 / Sprint 11 dispositions unchanged. PROMPT 761 Polish->Release
> gate-check FAIL evidence preserved.
>
> **No optimistic client-side authority is introduced or proposed by this
> story.** The change is purely additive on existing `tracing::*!()`
> emission sites -- each site adds `target: "module::path"` arguments;
> no behaviour or authoritative state is touched. ADR-002 binding.

PROMPT 847 (this worker run) does not retry the PROMPT 761
Polish->Release gate-check, does not run `/story-done`, `/smoke-check`,
`/team-qa`, `/gate-check`, `/release-check`, or `/qa-plan`, and does not
modify `production/sprint-status.yaml`, `production/sprints/sprint-12.md`,
`production/sprints/sprint-13.md`, `production/stage.txt`, or
`production/qa/qa-plan-sprint-13.md`. Sprint 12 disposition
(`closed-with-conditions` per PROMPT 817) is preserved; Sprint 13
disposition (`active` per PROMPT 833) is preserved.

**No optimistic client-side authority is introduced.** ADR-002 binding.
The diff is purely additive on `tracing::*!()` macro sites: a `target:
"..."` first argument is added (or, on 8 pre-existing sites that already
carried a non-module-path `target:`, the value is qualified with the
module path as a prefix to preserve subtree-matching under the Story 019
invocation while keeping the original narrow-capture tag as a suffix).

---

## Cross-Link to Source Finding

- PROMPT 803 §3 DC-11 (`reports/PROMPT-803-MULTIPLAYER-RUNTIME-HARDENING-AUDIT-ROADMAP.md`):
  "Tracing target hierarchy unscoped (HIGH for diagnostic): Story 019
  invocation `RUST_LOG=client::ui::hand=trace,...` would capture only
  crate-level emissions; no `tracing::*!(target: "client::ui::hand",
  ...)` calls exist."
- PROMPT 803 §5 Must row 7: "Add `target: "client::ui::hand"`, `target:
  "client::presentation::board_rendering"`, `target:
  "client::card_animations::input_gating"`, `target: "server::game"` to
  all relevant emission sites so the Story 019 `RUST_LOG` invocation
  actually captures something."
- PROMPT 803 §4 Lane A: server `rsm_dispatch.rs` S2C broadcast emission
  sites are tagged `server::game`.

---

## Pre/Post Grep Counts

**Pre-impl baseline** (`origin/main@fe74fb0` -- PROMPT 844 closure):

| Target string | Pre count |
|---|---|
| `target: "client::ui::hand"` (exact) | 0 |
| `target: "client::ui::hand` (prefix incl. narrower forms) | 3 (`fan_active_default_drop`, `drag_sprite_visible_flip`, `placement_cursor_move`) |
| `target: "client::presentation::board_rendering"` (exact) | 0 |
| `target: "client::presentation::board_rendering` (prefix) | 4 (4x `spawn_highlight_caller` / `spawn_highlight_state_change`) |
| `target: "client::card_animations::input_gating` (prefix) | 0 |
| `target: "server::game"` (exact) | 0 |
| `tracing::*!()` total sites in `client/src/ui/hand/mod.rs` | 18 |
| `tracing::*!()` total sites in `client/src/presentation/board_rendering.rs` | 8 |
| `tracing::*!()` total sites in `client/src/card_animations/input_gating.rs` | 1 |
| `tracing::*!()` total sites in `server/src/feature/*` | 104 |
| `tracing::*!()` total sites in `server/src/network/*` | 7 |

Pre-impl pattern note: 8 sites already carried non-module-path `target:
"<event-name>"` values (`fan_active_default_drop`,
`drag_sprite_visible_flip`, `placement_cursor_move`,
`spawn_highlight_caller` x3, `spawn_highlight_state_change`,
`drag_lift_tween_install`). These were added by prior instrumentation
prompts for narrow diagnostic capture. PROMPT 803's claim of "zero
`target: "..."` arguments" was stale by the PROMPT 844 base.

**Post-impl** (`work/s13-obs-tracing-targets@<commit>`):

| Target string | Post count | Site files |
|---|---|---|
| `target: "client::ui::hand"` (exact) | 15 | `client/src/ui/hand/mod.rs` |
| `target: "client::ui::hand` (prefix incl. narrower forms) | 18 | `client/src/ui/hand/mod.rs` (15 exact + 3 narrower: `client::ui::hand::fan_active_default_drop`, `client::ui::hand::drag_sprite_visible_flip`, `client::ui::hand::placement_cursor_move`) |
| `target: "client::presentation::board_rendering"` (exact) | 4 | `client/src/presentation/board_rendering.rs` |
| `target: "client::presentation::board_rendering` (prefix) | 8 | `client/src/presentation/board_rendering.rs` (4 exact + 4 narrower: 3x `client::presentation::board_rendering::spawn_highlight_caller`, 1x `client::presentation::board_rendering::spawn_highlight_state_change`) |
| `target: "client::card_animations::input_gating` (prefix) | 1 | `client/src/card_animations/input_gating.rs` (narrower: `client::card_animations::input_gating::drag_lift_tween_install`) |
| `target: "server::game"` (exact) | 112 | `server/src/feature/{acquisition,auction,board,combat,keyword,objective,prism}/...` + `server/src/network/{economy_dispatch,mod,rsm_dispatch}.rs` |

Per-file post-impl `target: "server::game"` counts (from `git grep -c`):

| File | Sites tagged |
|---|---|
| `server/src/feature/acquisition/system.rs` | 34 |
| `server/src/feature/auction/system.rs` | 37 |
| `server/src/feature/board/movement.rs` | 3 |
| `server/src/feature/board/placement.rs` | 3 |
| `server/src/feature/combat/mod.rs` | 8 |
| `server/src/feature/keyword/observers.rs` | 5 |
| `server/src/feature/objective/system.rs` | 6 |
| `server/src/feature/prism/system.rs` | 8 |
| `server/src/network/economy_dispatch.rs` | 2 |
| `server/src/network/mod.rs` | 4 |
| `server/src/network/rsm_dispatch.rs` | 1 |

Total `tracing::*!()` site counts pre/post by file -- unchanged:

| File | Pre | Post |
|---|---|---|
| `client/src/ui/hand/mod.rs` | 18 | 18 |
| `client/src/presentation/board_rendering.rs` | 8 | 8 |
| `client/src/card_animations/input_gating.rs` | 1 | 1 |
| `server/src/feature/*` | 104 | 104 |
| `server/src/network/*` | 7 | 7 |

AC7 satisfied: zero new emission sites introduced; the diff is purely
"add `target:` arg to existing sites" (plus 8 narrower-form rewrites of
prior non-module-path targets).

---

## Acceptance Criteria

### AC1 -- `client::ui::hand` target landed

PASS. `git grep 'target: "client::ui::hand' -- client/src/ui/hand/mod.rs`
returns 18 matches. Of these:

- 15 carry exactly `target: "client::ui::hand"` (covering all
  `hand_ui_apply_fan_layout_slot`, `hand_ui_phase_transition`,
  `hand_ui_hand_count_set`, `hand_ui_pending_placements_cleared`,
  `hand_ui_placement_drop_resolved`, `hand_ui_round_started_consumed`,
  `hand_ui_placement_submit`, `hand_ui_class_card_dropped`, `HandUiPlugin
  loaded`, and the broader S1-S5 instrumentation sites).
- 3 carry narrower module-path-shaped forms (rationale: preserve
  pre-existing narrow-capture diagnostic intent under
  `client::ui::hand::*` subtree):
  - `client::ui::hand::fan_active_default_drop` (line 1944)
  - `client::ui::hand::drag_sprite_visible_flip` (line 2064)
  - `client::ui::hand::placement_cursor_move` (line 2093)

`RUST_LOG=client::ui::hand=trace` will capture all 18 sites via subtree
match; `RUST_LOG=client::ui::hand::placement_cursor_move=trace` (etc.)
still narrows as before.

### AC2 -- `client::presentation::board_rendering` target landed

PASS. 8 matches (4 exact + 4 narrower). The 4 narrower targets
(`spawn_highlight_caller` x3, `spawn_highlight_state_change` x1) are
preserved as `client::presentation::board_rendering::spawn_highlight_*`
sub-paths for subtree match by Story 019 invocation.

### AC3 -- `client::card_animations::input_gating` target landed

PASS (narrower form, rationale recorded). 1 match:
`target: "client::card_animations::input_gating::drag_lift_tween_install"`
(line 163). This is the single pre-existing site in the file; its prior
target `"drag_lift_tween_install"` was rewritten to the module-path-shaped
narrower form. Story 019 invocation
`RUST_LOG=client::card_animations::input_gating=info` captures it via
subtree match.

### AC4 -- `server::game` target landed at server feature/network emission sites

PASS. 112 matches across `server/src/feature/*` and
`server/src/network/*`. The `server/src/network/rsm_dispatch.rs` S2C
broadcast emission site (per PROMPT 803 §4 Lane A) is tagged
`server::game` (line 34). All `tracing::*!()` sites in the listed
server files now carry `target: "server::game"`. No narrower forms
were introduced for the server modules.

### AC5 -- Verification harness pass

DEFERRED with rationale. PROMPT 847 scope explicitly forbids re-attempting
drag-runtime capture ("Do not re-attempt drag-runtime capture. Do not
broaden into logging redesign."). AC5 requires the Story 019 RUST_LOG
invocation against either a drag-runtime smoke test or
`S13-TWO-CLIENT-RUNTIME-HARNESS-001`; the former is excluded by prompt
scope, and the latter (`S13-TWO-CLIENT-RUNTIME-HARNESS-001`) has not
been observed landed at this base. AC5 verification is deferred to the
next Story 019 retest prompt or to a Sprint 13 harness-up prompt that
can drive the runtime path with the new targets in place.

Static evidence that the targets exist and are correctly shaped is
recorded in the grep tables above. The literal target strings that
Story 019's invocation matches (`client::ui::hand`,
`client::presentation::board_rendering`,
`client::card_animations::input_gating`, `server::game`) are all present
post-impl with non-zero counts (15, 4, 1 [narrower], 112 respectively),
so the Story 019 invocation -- whenever next run -- will capture
non-empty output per target via direct match or subtree match.

### AC6 -- Behaviour unchanged

PASS within prompt scope. PROMPT 847 scope explicitly forbids running
full workspace tests ("Run only story-prescribed targeted checks. Do
not run full workspace tests."). Static evidence:

- `cargo fmt --all -- --check`: PASS (exit 0).
- `cargo check -p client`: PASS (exit 0; no warnings introduced).
- `cargo check -p server`: PASS (exit 0; no warnings introduced).
- Diff inspection: the only `+`/`-` lines in the entire diff are
  `target: "..."` additions/rewrites. No control-flow, no field-name,
  no message-string changes. `git diff origin/main...HEAD --unified=0
  | grep '^[+-]' | grep -v '^[+-]\{3\}' | grep -v 'target: "'`
  returns zero lines outside the `target:` edits.

The `tracing` crate's macro expansion for `target: "..."` is well-known
to be behaviourally inert (the `target:` argument sets the
`tracing::Metadata::target` field; it does not alter call-site control
flow, allocation, ordering, or message formatting). No runtime semantics
modified.

### AC7 -- No new emission sites added

PASS. Per-file `tracing::*!()` site counts unchanged pre/post (see table
above). All `+` lines outside `target:` additions are zero per the diff
inspection in AC6.

### AC8 -- No optimistic client-side authority introduced

PASS. The diff touches only the first argument slot of `tracing::*!()`
macro invocations. No mutation of authoritative state, no new
client-side `ResMut<_>` on `CurrentClientPhase` / `ClientState` /
`PendingPlacements` / `S2C*` consumer resources. Phase sink
(`client/src/presentation/mod.rs::phase_sink_system`) is not in the
diff. The phrase "no optimistic client-side authority introduced" is
preserved here verbatim per the AC8 search-string requirement. ADR-002
binding maintained.

### AC9 -- Sprint 12 disposition preserved

PASS. `production/sprint-status.yaml`, `production/sprints/sprint-12.md`,
`production/stage.txt`, and `production/qa/qa-plan-sprint-12.md` are
NOT in the worker diff. `git diff --name-only origin/main...HEAD` lists
exactly the 14 source files modified plus this evidence document.

### AC10 -- Evidence document slot reserved

PASS. This file at
`production/qa/evidence/sprint-13-obs-tracing-targets-evidence.md`.

---

## Verification Commands Run

```
cargo fmt --all -- --check                            # exit 0
cargo check -p client                                 # exit 0
cargo check -p server                                 # exit 0
git grep -c 'target: "client::ui::hand"' -- client/src/
git grep -c 'target: "client::ui::hand' -- client/src/
git grep -c 'target: "client::presentation::board_rendering"' -- client/src/
git grep -c 'target: "client::presentation::board_rendering' -- client/src/
git grep -c 'target: "client::card_animations::input_gating' -- client/src/
git grep -c 'target: "server::game"' -- server/src/
git diff --check origin/main...HEAD                   # exit 0
git diff --check                                       # exit 0 (no staged-vs-worktree whitespace)
```

Cargo policy (`CARGO_TARGET_DIR=D:/_DEV/cargo-target/ccgs-msvc`,
`CARGO_PROFILE_*_DEBUG=0`, `CARGO_INCREMENTAL=0`,
`RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'`) was applied for
both `cargo fmt` and `cargo check` invocations. No stale-target
cleanup was needed.

---

## Files Changed

| File | Lines +/- |
|---|---|
| `client/src/card_animations/input_gating.rs` | +1 / -1 (target rewrite) |
| `client/src/presentation/board_rendering.rs` | +8 / -4 (4 added + 4 rewritten) |
| `client/src/ui/hand/mod.rs` | +18 / -3 (15 added + 3 rewritten; the 15th addition is the inline single-line on line 802) |
| `server/src/feature/acquisition/system.rs` | +34 / 0 |
| `server/src/feature/auction/system.rs` | +37 / 0 |
| `server/src/feature/board/movement.rs` | +3 / 0 |
| `server/src/feature/board/placement.rs` | +3 / 0 |
| `server/src/feature/combat/mod.rs` | +8 / 0 |
| `server/src/feature/keyword/observers.rs` | +5 / 0 |
| `server/src/feature/objective/system.rs` | +6 / 0 |
| `server/src/feature/prism/system.rs` | +8 / 0 |
| `server/src/network/economy_dispatch.rs` | +2 / 0 |
| `server/src/network/mod.rs` | +4 / 0 |
| `server/src/network/rsm_dispatch.rs` | +1 / 0 |
| `production/qa/evidence/sprint-13-obs-tracing-targets-evidence.md` | NEW |

Total: 14 source files modified + 1 NEW evidence doc.

The numbers in the table above reflect per-site additions before
rustfmt. `cargo fmt --all -- --check` passed against the post-impl
content, so no rustfmt reflow was needed.

---

## Out-of-Scope Reiteration

This implementation:

- Does NOT add new `tracing::*!()` emission sites.
- Does NOT modify the tracing subscriber configuration (UTC / wall-clock
  timestamps are scoped to `S13-OBS-WALLCLOCK-TIMESTAMPS-001`, which
  closed via PROMPT 843).
- Does NOT remove or replace existing log message content (only the
  `target:` metadata argument is added or rewritten).
- Does NOT modify any `client/src/ui/hand/` field names, system bodies,
  message contents, or function signatures.
- Does NOT modify any server feature / network module behaviour.
- Does NOT touch `client/src/presentation/mod.rs::phase_sink_system`
  (Sprint 13 Story 022 `S11-HU-PHASE-IDEMPOTENCY-001` consumer-private
  state remains unchanged).
- Does NOT modify shared/, protocol shapes, or any test file.
- Does NOT run `/story-done`, `/smoke-check`, `/team-qa`,
  `/gate-check`, `/release-check`, or `/qa-plan`.
- Does NOT modify `production/sprint-status.yaml`,
  `production/sprints/sprint-12.md`, `production/sprints/sprint-13.md`,
  `production/stage.txt`, `production/qa/qa-plan-sprint-12.md`, or
  `production/qa/qa-plan-sprint-13.md`.
- Does NOT close `S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006`, or any
  other carried Sprint condition.
- Does NOT claim public release readiness, release-candidate readiness,
  full playable-client manual QA, broad Standard-tier accessibility
  completion, playtest / fun-hypothesis validation, full-game
  completion, or final-art / asset-production completion.

Sprint 12 disposition `closed-with-conditions` (PROMPT 817), Sprint 13
disposition `active` (PROMPT 833 onwards), PROMPT 761 Polish->Release
gate-check FAIL evidence, Sprint 10 / Sprint 11 closeouts, and all
carried QA conditions are preserved unchanged by PROMPT 847.
