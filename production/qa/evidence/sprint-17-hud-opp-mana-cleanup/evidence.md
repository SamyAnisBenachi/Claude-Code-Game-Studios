# S17-UI-HUD-OPP-MANA-CLEANUP-001 — PROMPT 1105 Implementation Evidence

> **Sprint**: Sprint 17 (Should Have row)
> **Story**: `production/epics/hud/story-018-opp-figurine-mana-cleanup.md`
> **Prompt**: PROMPT 1105 — `/dev-story` worker
> **Worker branch**: `work/s17-hud-opp-mana-cleanup`
> **Worktree**: `D:\_DEV\claude-code-game-studios-worktrees\s17-hud-opp-mana-cleanup`
> **Activation HEAD**: `origin/main@ff47075` (Sprint 17 QA plan tip / PROMPT 1100)
> **Author**: PROMPT 1105
> **Date**: 2026-05-18

---

## Implementation Summary

Sprint 17 row `S17-UI-HUD-OPP-MANA-CLEANUP-001` bundled three PROMPT 1076
audit findings (AUDIT-1076-10 / -16 / -17) under a single HUD owner. The
worker contract for this story explicitly contemplated the possibility
that one of the bundled findings (AUDIT-1076-17, the floating mana
microbadge) would need to be escalated out of scope:

> **Story §"Worker Contract" step 5**: "Locate the floating mana
> microbadge spawn site. If not under `client/src/ui/hud/`, pause and
> escalate (the audit suggests it is HUD-owned; if it is elsewhere — e.g.
> in an overlay file — file ownership for AC3 needs clarification)."

PROMPT 1105 implemented AUDIT-1076-10 + AUDIT-1076-16 HUD-locally and
escalates AUDIT-1076-17 per the contract above. Detail per AC below.

### AC1 — Opponent figurine re-skins on `S2CClassesRevealed` (AUDIT-1076-10)

**Implemented.** Added a new HUD-local resource `HudClassReveal { local,
opponent }` plus two MessageDrain projection systems:

- `sync_class_reveal_from_lobby_view_system` reads
  `Res<LobbyViewState>` (the canonical lobby reducer's `revealed_classes`
  mirror) and writes `HudClassReveal`. Skips while `HudMode::Frozen` so
  the FROZEN-on-GAME_OVER contract holds for incremental reveals.
- `sync_class_reveal_from_snapshot_system` reads
  `MessageReader<PresentationGameSnapshotMessage>` and writes
  `HudClassReveal` regardless of `HudMode`, so `S2CGameSnapshot`
  reconnect rebuilds (ADR-011) remain authoritative even at GAME_OVER.

In `HudSystemSet::StateSync` (after `sync_gold_text_system` and
`sync_figurine_image_system`), `sync_class_reveal_hud_system` applies
the resolved class identity to the opponent figurine `ImageNode` via
`hud_figurine_asset(opponent_class)`. A `Local<Option<ClassId>>` cache
short-circuits redundant writes.

Test: `tests/integration/hud/opp_figurine_label_mana_repaint_test.rs::
ac1_opponent_figurine_reskins_on_classes_revealed`.

### AC2 — OPP text label re-skins on `S2CClassesRevealed` (AUDIT-1076-16)

**Implemented.** The OPP pill is structurally `prefix_label + value_text`
(see `client/src/ui/hud/mod.rs` `spawn_pill_container` /
`spawn_pill_prefix` / `spawn_gold_label`). The value entity
(`opponent_gold_parent`) is the canonical opponent-gold readout — guarded
by `tests/integration/hud/reconnect_snapshot_rebuild_test.rs` which
asserts `"15g"` / `"8g"` / `" (2r)"` formats post-snapshot. Repurposing
the value as a class display would have broken that contract.

The worker therefore writes the per-class display string to the
`opponent_gold_prefix` Text entity (the static label that previously
read `"OPP"`). Combined visible glyph stream after reveal is e.g.
`"OPP Ecaflip 8g"`. The format helper `format_opp_class_display(class_id)`
returns `"OPP {ClassId:?}"` mirroring the Sprint 14 lobby class-picker
display strings (`format!("{:?}", class_id)`).

Test: `tests/integration/hud/opp_figurine_label_mana_repaint_test.rs::
ac2_opp_text_label_reskins_on_classes_revealed`.

### AC3 — Mana microbadge removal (AUDIT-1076-17) — **ESCALATED**

**Out of scope for PROMPT 1105.** The microbadge spawn site is located
at:

- File: `client/src/ui/hand/mod.rs` (NOT under `client/src/ui/hud/`)
- Spawn function: `spawn_reserve_strip` (around line 3505)
- Per-card readout: `Text::new("Reserve 0 Current 0")` (line 3530)
- Per-card updater: `text.0 = format!("Reserve {reserve_amount} Current
  {current_amount}");` (around line 4108-4110)

Per PROMPT 1105's explicit "Do not edit hand" rule (prompt body, "Do not
edit hand, shop_auction, server, shared, protocol, sprint-status,
session-state, sprint plans, QA plan, or stage.txt"), the worker cannot
touch this spawn site.

The story's worker contract (step 5) anticipated this outcome and asks
the worker to pause-and-escalate: "If not under
`client/src/ui/hud/`, pause and escalate ... file ownership for AC3
needs clarification."

Additionally, the audit's claim that the microbadge "duplicates" the
HUD canonical `MANA current / cap` strip is **not exact**: the hand-UI
Reserve Strip displays per-card reserve commitment state (the player's
local pre-submit reserve toggle for a specific hand card), which is
semantically distinct from the HUD strip's authoritative
`current_mana / mana_cap` readout. The audit's
"if the microbadge currently carries any unique payload not represented
in the canonical strip ... the worker pauses and escalates; otherwise
the microbadge is removed" branch fires the pause-and-escalate path.

**Recommendation**: orchestrator should author a follow-up row
(suggested slug `S17-UI-HAND-RESERVE-STRIP-CLEANUP-001` or
`S18-UI-HAND-RESERVE-STRIP-CLEANUP-001`) owned by `client/src/ui/hand/`
to decide:

1. Should the hand-UI Reserve Strip be removed entirely (if the per-card
   reserve toggle UI is to be retired)?
2. Should the Reserve Strip be repositioned so it no longer "floats
   above the modal" (the audit's actual layout complaint)?
3. Should the Reserve Strip remain but adopt a less ambiguous label
   ("Card Reserve" / "Per-Card Reserve" / etc.) so it does not visually
   read as a duplicate of the HUD MANA strip?

PROMPT 1105 makes no changes to `client/src/ui/hand/mod.rs`.

### AC4 — Re-skin happens in StateSync (instantaneous)

**Implemented.** `sync_class_reveal_hud_system` is registered in
`HudSystemSet::StateSync`. No `Animator` / `TweenAnim` / `Tween::new`
appears in its body — matches TR-HUD-008 + Sprint 14 story 017 AC4
precedent. Guarded by
`opp_figurine_label_mana_repaint_test.rs::ac4_reskin_runs_in_state_sync_set`.

### AC5 — Reconnect rebuild covers OPP figurine + label

**Implemented.** `sync_class_reveal_from_snapshot_system` writes
`HudClassReveal` from `PresentationGameSnapshotMessage`. The downstream
`sync_class_reveal_hud_system` then re-skins both the opponent figurine
and the OPP prefix label. Guarded by
`opp_figurine_label_mana_repaint_test.rs::ac5_reconnect_snapshot_rebuilds_opp_figurine_and_label`.

### AC6 — FROZEN-on-GAME_OVER preserved

**Implemented.** `sync_class_reveal_from_lobby_view_system` early-returns
when `*mode == HudMode::Frozen`, blocking the incremental lobby reveal
path. `sync_class_reveal_from_snapshot_system` is intentionally NOT
gated, so `S2CGameSnapshot` can still overwrite during GAME_OVER.
Guarded by
`opp_figurine_label_mana_repaint_test.rs::ac6_frozen_blocks_lobby_reveal_but_snapshot_can_overwrite`.

### AC7 — No client-side opponent-class inference added

**Implemented.** The new systems read from:

1. `Res<LobbyViewState>.revealed_classes` (canonical lobby reducer mirror,
   server-authoritative)
2. `MessageReader<PresentationGameSnapshotMessage>` (server-authoritative
   snapshot rebuild)
3. `Res<ClientSessionIdentity>.player_id` (handshake-authoritative local
   identity)

No code paths derive opponent class from spawned units, lane state,
board state, or any other client-side observation. Guarded by
`opp_figurine_label_mana_repaint_test.rs::ac7_no_client_side_class_inference_introduced`,
which greps for `MessageReceiver<S2CClassesRevealed>` (forbidden) and for
`Unit` / `lane` / `board` / `Objective` / `was_fake` / `BoardSnapshot`
in the lobby projection body (also forbidden).

### AC8 — ADR-001 invariant preserved

**Implemented.** The OPP figurine + OPP prefix re-skin path carries
only `ClassId` — no `was_fake`, `ObjectiveSnapshot`,
`OpponentObjectiveSnapshot`, `is_real`, or `ObjectiveDotState` flows
through. Guarded by
`opp_figurine_label_mana_repaint_test.rs::ac8_adr_001_invariant_preserved_opp_carriers_carry_only_class`.

### AC9 — Integration test bin authored

**Implemented.** `tests/integration/hud/opp_figurine_label_mana_repaint_test.rs`
(NEW, 8 tests covering AC1, AC2, AC4, AC5, AC6, AC7, AC8 + opponent
figurine marker singleton guard). AC3 NOT covered per escalation above.
Registered in `client/Cargo.toml` as
`[[test]] name = "hud_opp_figurine_label_mana_repaint_test"`.

### AC10 — No protocol or server change

**Implemented.** `git diff <activation HEAD>..HEAD` shows zero changes
under `server/`, `shared/`, or `tests/integration/server/`. The
implementation is client-side only.

### AC11 — ADR-021 schedule preserved

**Implemented.** No new `SystemSet` variant introduced. New systems
slot into existing `HudSystemSet::MessageDrain` (lobby + snapshot
projections) and `HudSystemSet::StateSync` (entity reskin).

### AC12 — No accept-risk closure claimed

**Confirmed.** This evidence document and the commit message do NOT
claim closure of `S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006`,
`PAW-TD-*-a` (specifically `PAW-TD-004-a` for the figurine placeholder),
or any other accept-risk disposition. Final-art replacement of the
opponent figurine is explicitly out of scope. Standard-tier
hit-target conformance on the OPP label is NOT pursued. Playtest
validation is NOT pursued. AUDIT-1076-17 is escalated, not closed.

### AC13 — Sprint 17 disposition preserved

**Confirmed.** No edits under `production/sprint-status.yaml`,
`production/sprints/*`, `production/stage.txt`,
`production/session-state/*`, `production/qa/qa-plan-*.md`,
`production/gate-checks/*`, or `docs/architecture/adr-*.md`.

### AC14 — Worker branch scope contained

**Confirmed.** Branch `work/s17-hud-opp-mana-cleanup` from
`origin/main@ff47075`. Files changed:

- `client/src/ui/hud/mod.rs` — new imports, new resource
  `HudClassReveal`, three new systems, register/schedule wiring.
- `client/Cargo.toml` — new `[[test]]` registration for the new
  integration test bin.
- `tests/integration/hud/opp_figurine_label_mana_repaint_test.rs` —
  NEW.
- `production/qa/evidence/sprint-17-hud-opp-mana-cleanup/evidence.md` —
  NEW (this document).

No edits under `server/`, `shared/`, `tests/integration/server/`,
`tests/unit/server/`, `client/src/ui/hand/`, `client/src/ui/shop_auction/`,
`client/src/ui/lobby.rs` (the `apply_classes_revealed` reducer is read
via `Res<LobbyViewState>` but NOT modified), `Cargo.toml` (workspace),
`Cargo.lock`, `.cargo/`, `.github/`, `Trunk.toml`,
`production/sprint-status.yaml`, `production/sprints/*`,
`production/stage.txt`, `production/session-state/*`,
`production/qa/qa-plan-*.md`, `production/qa/smoke-*.md`,
`production/qa/team-qa-*.md`, `production/gate-checks/*`,
`docs/architecture/adr-*.md`, `assets/`, `.claude/`, `AGENTS.md`,
`CLAUDE.md`, `CODEX.md`.

### AC15 — Cargo resource policy applied

**Confirmed.** Every `cargo check` / `cargo test` invocation in this
session exported:

```
CARGO_TARGET_DIR=D:/_DEV/cargo-target/ccgs-msvc
CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0
CARGO_INCREMENTAL=0
RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'
```

D: drive free space at session start: ~810 GB (well above the 40 GB
threshold; no target-dir cleaning performed).

### AC16 — HUD epic story count refreshed

**Out of scope for PROMPT 1105.** The story file specifies the refresh
to `production/epics/hud/EPIC.md` is performed by the `/story-done`
paperwork prompt, not by `/dev-story`. PROMPT 1105 does not touch
`EPIC.md`.

---

## Blocking Test Gate Results

```
cargo test -p client --test hud_opp_figurine_label_mana_repaint_test
  → 8 passed; 0 failed; 0 ignored

cargo test -p client --test hud_opp_figurine_test
  → 5 passed; 0 failed; 0 ignored

cargo test -p client --test reconnect_snapshot_rebuild_test
  → 3 passed; 0 failed; 0 ignored

cargo test -p client --test hud_game_over_freeze_test
  → 2 passed; 0 failed; 0 ignored

cargo test -p client --test hud_plugin_scaffold_test
  → 4 passed; 0 failed; 0 ignored

cargo test -p client --test hud_phase_transitions_test
  → 5 passed; 0 failed; 0 ignored

cargo test -p client --test hud_top_strip_layout_test
  → 8 passed; 0 failed; 0 ignored

cargo check -p client
  → Finished `dev` profile

git diff --check
  → clean (no whitespace errors)
```

---

## Out-of-Scope Items Carried Forward Unchanged

- Sprint 16 disposition `closed-with-conditions` (UNCHANGED).
- Sprint 17 stage `Polish` (UNCHANGED).
- PROMPT 761 Polish→Release gate-check `FAIL` preserved.
- `S8-QA-001-W1` OPEN preserved.
- `QA-COND-0005` + `QA-COND-0006` accepted-risk preserved.
- `PAW-TD-*-a` placeholder-art accept-risk preserved (specifically
  `PAW-TD-004-a` for the figurine placeholder).
- `TQ-S12-C1..C7` preserved verbatim.
- Sprint 14 story 017 `S11-UX-HUD-OPP-FIGURINE` Done (closed PROMPT 976;
  UNCHANGED).
- HUD timer row `S11-HUD-TIMER-EYEBALL-VISUAL-001` human-operator-blocked
  carry preserved; NOT closed.
- 24 PROMPT 1022 audit findings preserved as report-only; NOT closed.
- AUDIT-1076-17 (floating mana microbadge) **escalated**, NOT closed by
  this PROMPT; recommended follow-up row owns `client/src/ui/hand/`.
- All other AUDIT-1076-* findings outside the bundled -10 / -16 preserved
  unchanged.
- All SOURCE-1077-* findings preserved unchanged.

---

## Status Line

```
1105: S17-UI-HUD-OPP-MANA-CLEANUP-001: PARTIAL
```

`PARTIAL` reflects that AC1, AC2, AC4–AC15 are implemented and tested,
and AC3 is **escalated per the story's worker-contract pause-and-
escalate branch** (the floating mana microbadge spawn site is owned by
`client/src/ui/hand/`, forbidden by the PROMPT 1105 scope guard). The
core HUD-local concerns of the row — opponent figurine + OPP label
re-skin on class reveal + reconnect rebuild + FROZEN/ADR-001/ADR-002
invariants — are fully delivered.
