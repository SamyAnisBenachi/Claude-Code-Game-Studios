# Story 018: S11-DRAG-RUNTIME-RETEST-001 — Drag-and-Drop Runtime Divergence Retest + S1-S5 Truth-Table Lock

> **Epic**: Hand UI
> **Sprint**: Sprint 11 (DRAFT — not activated; this story is authoring-only)
> **Status**: Draft (Sprint 11 not activated; pending `/story-readiness` after `/sprint-plan sprint-11`)
> **Layer**: Presentation
> **Type**: Integration — runtime evidence + disposition (NOT a code-change story unless evidence proves a production bug)
> **Authored**: 2026-05-13 (PROMPT 766)
> **Authoring source-of-truth**: `origin/main@2f9abfb` (Sprint 11 draft state)

---

## Status / No-Claim Banner

- Sprint 11 is `draft` per `production/sprints/sprint-11.md` and the `next_sprint:` block in `production/sprint-status.yaml`. This story is **not activated** and does **not** appear as an active row in `production/sprint-status.yaml`. Activation happens via `/sprint-plan sprint-11` in a separate prompt.
- This story authoring does **not** claim: public release readiness, release-candidate readiness, full game completion, broad / Standard-tier accessibility completion (`QA-COND-0005`), playtest / fun-hypothesis validation (`QA-COND-0006`), full playable-client manual QA, two-client GAME_OVER closure (`S8-QA-001-W1`), or final-art / asset-production completion.
- `production/stage.txt` remains `Polish`. No `/dev-story`, `/smoke-check`, `/team-qa`, `/gate-check`, or `/story-done` is run by the authoring of this story file.

---

## Context

**GDD**:
- `design/gdd/hand-ui.md` — placement drag-to-stage state machine
- `design/ux/hand-ui.md` — State machine (Dragging card / Valid board target hover / Staged board card / Un-staging, L210–230)

**ADR Governing Implementation**:
- [ADR-021: Presentation Layer Architecture](../../docs/architecture/adr-021-presentation-layer-architecture.md) — Hand UI owns the placement drag lifecycle from input to ghost; drag sprite is a bevy_ui `Node`; pre-pooled entities.
- [ADR-002: Client-Server Authority](../../docs/architecture/adr-002-client-server-authority.md) — Client is a view; no optimistic client-side authority is allowed to be introduced by this retest.
- [ADR-009: RSM Phase State](../../docs/architecture/adr-009-rsm-phase-state.md) — Phase state is read from `Res<CurrentClientPhase>`; the retest must verify that drag enable/disable correctly reflects phase, not buffer.

**Engine**: Bevy 0.18 | Lightyear 0.26 | **Risk**: HIGH (gameplay-blocking divergence in friend-game runtime; tests-green but runtime-broken).

**Mandatory skills**:
- `liv-bevy-018` — for any read/review/edit of Bevy `.rs` code touched by this retest.
- `liv-bevy-lightyear` — only if the retest reads or reviews protocol / `C2SActivateCard` / `S2C*` networking code while diagnosing.

**Why this story exists (the divergence we are retesting)**:

| # | Landed work | Commit / Prompt |
|---|-------------|-----------------|
| 1 | HU-card-drag MVP (Press / Move / Release producers + sprite follow) | `00ffe89` / PROMPT 696 (story-017) |
| 2 | `handle_placement_drag_ended` gate widened for all `PlacementTargetKind` variants | `cbb2565` / PROMPT 697 |
| 3 | S1-S5 tracing instrumentation (5 sites) per PROMPT 698 diagnostic spec | `7e0c663` / PROMPT 706 / 709 |

Despite (1)–(3) landing, **the runtime trace was never captured end-to-end on a real friend-game session**, and the **S1-S5 grey-square attribution truth-table was never locked**. PROMPT 762 candidate #1 surfaced this as a HIGH gameplay-blocking gap; PROMPT 764 folded it into the Sprint 11 draft Must Have as `S11-DRAG-RUNTIME-RETEST-001`. The 683 diagnostic previously reported 8 `C2SActivateCard` sends with zero `stage_or_update` events — that runtime evidence is the open question this story closes.

**Control Manifest Rules (Presentation Layer, retest-scope)**:
- Required: All retest reasoning must be gated on `HandUiMode::Staging` (or `active_drag.is_active()` for follow/move/end paths), matching story-017 producer scope.
- Required: Pointer button checks must require `PointerButton::Primary`.
- Required: Phase state read via `Res<CurrentClientPhase>`; `MessageReceiver<S2CPhaseChanged>` is never drained directly by Hand UI.
- Required: `liv-bevy-018` skill applies to all `On<Pointer<...>>` observer signatures touched.
- Forbidden: Introducing client-side optimistic authority for stage / activate / submit (ADR-002 / ADR-009 line still binding).
- Forbidden: Synthesising bespoke `WindowEvent` / cursor-position polling; rely on `bevy_picking` events for input.
- Forbidden: Modifying `S2C*` / `C2S*` protocol shapes inside this retest. Any protocol-shape divergence discovered must be authored as a separate follow-on story.

---

## Goal

Capture an authoritative runtime trace of the PLACEMENT drag-to-stage flow on a real two-client friend-game session, **lock the S1-S5 grey-square attribution truth-table from PROMPT 698 / 706 / 709**, and **disposition** the test-green/runtime-broken divergence into exactly one of:

1. **Bug reproduced** — divergence reproduced, root cause identified, repair commit lands on `main` via a follow-on `/dev-story` (this story authors the repro story; it does **not** itself land the repair).
2. **Bug fixed** — divergence cannot be reproduced because already-landed work (PROMPT 696 / 697 / 706 / 709 cumulative) has silently resolved it; the truth-table is locked as PASS and an evidence note records the disposition.
3. **Cannot reproduce with evidence** — divergence is not reproducible under the documented `RUST_LOG` capture and friend-game route after a time-boxed run; evidence is recorded as such; a precise follow-on repro story is authored if any S1-S5 row remains ambiguous.
4. **Third-party / platform limitation** — divergence reproduces only under a specific browser, OS, GPU, or input-device profile (e.g., trackpad palm-rejection, touch capture, Chromium pointer-events quirk) and is **not** a bug in Hand UI / Board Rendering / Lightyear protocol code; evidence records the platform profile and the workaround (if any) plus a no-claim note.

**No optimistic client-side authority is introduced by any of (1)–(4).**

---

## Acceptance Criteria

All criteria below are independently checkable. Each criterion either passes (✅) or links to a written disposition (❌ + disposition path under `production/qa/evidence/`).

- [ ] **HU-DRAG-RT-01 — Runtime trace captured.** A real friend-game session is run with the exact `RUST_LOG` invocation in §"Reproduction Recipe" below; both clients connect; PLACEMENT phase is reached; at least one card from each player's hand is drag-attempted onto a valid `BoardCell` target and at least one drag-attempt is made on an `Instant` card (fan-plate target). Raw log capture is preserved at `production/qa/evidence/sprint-11-drag-runtime-evidence.md` and `production/qa/evidence/captures/sprint-11-drag-runtime/` (or equivalent capture directory).

- [ ] **HU-DRAG-RT-02 — S1-S5 truth-table locked.** Every row of the S1-S5 grey-square attribution truth-table in §"S1-S5 Truth Table" is filled with one of {PASS, FAIL, NOT-OBSERVED} and a one-line evidence pointer (log file + line range OR screenshot path + frame stamp). The locked truth-table is committed to the evidence file. If any row is `NOT-OBSERVED`, the disposition (`HU-DRAG-RT-04`) explicitly names which follow-on story will close it.

- [ ] **HU-DRAG-RT-03 — Test-vs-runtime divergence dispositioned.** The 683-era discrepancy (8 `C2SActivateCard` sends, zero `stage_or_update` events; tests green, runtime broken) is dispositioned as exactly one of {bug-reproduced, bug-fixed, cannot-reproduce, third-party-limitation}. Disposition is justified by S1-S5 evidence (`HU-DRAG-RT-02`) and recorded in the evidence file. If the disposition is `bug-reproduced`, the offending stage (S1 / S2 / S3 / S4 / S5) is named in the evidence file with the failing trace line range.

- [ ] **HU-DRAG-RT-04 — Repair or follow-on authored.** Depending on `HU-DRAG-RT-03`'s disposition:
  - `bug-reproduced` → a concrete follow-on story is authored under `production/epics/hand-ui/` (or `production/epics/playable-client/` if cross-epic) with the precise repro from the runtime trace, the failing S1-S5 stage(s), and a proposed scope. **No repair commit lands inside this story.**
  - `bug-fixed` → the evidence file records the cumulative work that resolved the bug (PROMPT 696 / 697 / 706 / 709 cite) and the truth-table-PASS rows that prove it. No follow-on story is authored.
  - `cannot-reproduce` → the evidence file records the time-box exhausted, the route attempted, the browser/OS profile, and any ambiguous S1-S5 rows. A follow-on diagnostic-only story is authored with a tighter capture spec (e.g., additional tracing sites, MSAA capture, slow-motion playback).
  - `third-party-limitation` → the evidence file records the platform profile, the workaround (if any), the no-claim note that this is **not** a Hand UI / Board Rendering / protocol bug, and any documentation update needed in `docs/setup/` or `docs/architecture/` (authored as a separate small follow-on story if scope > a paragraph).

- [ ] **HU-DRAG-RT-05 — No production code changes in this story.** This story is verification + paperwork. No edits land under `client/`, `server/`, `shared/`, or `tests/` as part of this story's `/dev-story`. The repair / repro path (if any) is delegated to the follow-on story authored in `HU-DRAG-RT-04`. Verified by `git diff --stat origin/main..work/<story-id>-drag-runtime-retest -- client/ server/ shared/ tests/` returning empty.

- [ ] **HU-DRAG-RT-06 — No optimistic client-side authority introduced.** The retest reasoning and any follow-on story authored under `HU-DRAG-RT-04` explicitly state that client-side authority for stage / activate / submit is forbidden (ADR-002 + ADR-009). Verified by text search in the evidence file and any authored follow-on story for the phrase "no optimistic client-side authority" or equivalent.

- [ ] **HU-DRAG-RT-07 — Non-claims preserved.** The evidence file restates the no-claim banner at the top of this story verbatim (public release, full manual QA, Standard-tier accessibility, playtest, full game completion, S8-QA-001-W1, QA-COND-0005, QA-COND-0006) and confirms none of them are claimed closed by this retest.

- [ ] **HU-DRAG-RT-08 — Sprint 11 draft status preserved.** This story does not touch `production/sprint-status.yaml`, `production/stage.txt`, or `production/sprints/sprint-11.md`. Verified by `git diff --stat origin/main..work/<story-id>-drag-runtime-retest -- production/sprint-status.yaml production/stage.txt production/sprints/sprint-11.md` returning empty.

---

## Reproduction Recipe

### RUST_LOG invocation (server + each client)

Run the local server and **both** clients with the same `RUST_LOG` value (so that S1-S5 emit on the client side and any contributing server-side state changes are also captured):

```text
RUST_LOG=client::ui::hand=trace,client::presentation::board_rendering=trace,client::card_animations::input_gating=info,lightyear=info,server::game=info
```

Notes:
- The three `=trace` targets cover the producer surface (hand observers + sprite follow), the consumer/ghost surface (board rendering), and the input-gating state transitions. These are the three modules where S1-S5 emit per PROMPT 706 / 709 (commit `7e0c663`).
- `lightyear=info` is sufficient — replication-channel `trace` produces excessive noise that buries S1-S5 evidence. If protocol-shape ambiguity surfaces during retest, upgrade to `lightyear=debug` in a follow-on run.
- `server::game=info` captures `C2SActivateCard` reception, `stage_or_update` outcomes, and any server-side rejection (which is the 683-era discrepancy this retest must close).
- Adjust target paths to match your local crate / module layout if the canonical names above have moved (verify via `git grep -n "info!\\|trace!\\|debug!" -- client/src/ui/hand/ client/src/presentation/board_rendering/ client/src/card_animations/input_gating/` before the run).

### Friend-game route (manual; mirrors `production/qa/evidence/manual-friend-game-evidence-runbook.md`)

1. Build both client and server in `--release` (`cargo run -p server --release` + `trunk serve --release` for client) — debug builds add timing jitter that can mask the divergence.
2. Open two browser tabs (Chrome stable + a second browser of choice — Firefox stable or Chromium-derivative) pointed at the local client URL. Use distinct profiles to prevent picking session-cookie collisions.
3. Both clients enter the lobby; one player picks a class (Iop or Ecaflip is preferred — broadest known card set); the other picks the same or different class; both confirm class.
4. Wait for `DRAFT_INITIAL` to enter; do **not** purchase cards yet — confirm the 9-card grid renders correctly on both clients (this exercises S2 / S3 indirectly via grid hover paths).
5. Purchase one card per client from the grid; confirm hand fan renders with the purchased card; observe **S1 / S2 / S3** for the grid → hand transition.
6. Phase advances through `DRAFT_SHOP` to `PLACEMENT` (round 1). On `PLACEMENT` entry:
   a. **Drag attempt A (standard unit onto BoardCell)**: Press primary button on a non-Instant card in fan slot 0 → drag pointer to a valid spawn-range board cell → release. Observe S1 → S2 → S3 → S4 → S5 in the trace.
   b. **Drag attempt B (Instant card onto fan plate)**: Press primary on an Instant card → drag pointer over the fan plate (not the board) → release. Observe S1 → S2 → S3 → S4 → S5 with the Instant fan-plate target branch.
   c. **Drag attempt C (cancel mid-drag)**: Press primary on a non-Instant card → drag pointer **off** any valid target → release over empty space. Observe S1 → S2 → S3 → S4 (cursor outside) → S5 (no-op cleanup).
   d. **Drag attempt D (invalid drop on opponent's lane / out-of-spawn-range cell)**: Press primary on a non-Instant card → drag pointer to an invalid board cell → release. Observe S1 → S2 → S3 → S4 (invalid target) → S5 (snap-back path).
7. Submit placement on both clients; let `RESOLUTION` play out; verify HUD timer and board rendering do not drop frames or freeze (this is **not** the focus of the retest but rules out an interaction with the resolution animation queue).
8. Capture raw log files from both clients + server. Capture at least one screenshot per drag-attempt (A/B/C/D) at the moment of release. Save under `production/qa/evidence/captures/sprint-11-drag-runtime/<client>-<attempt>-<phase>.png` (and `.log`).

### Time-box

Retest is time-boxed to **1.0 day** per the Sprint 11 draft acceptance criteria for `S11-DRAG-RUNTIME-RETEST-001`. If S1-S5 cannot be filled in 1.0 day:
- Lock the truth-table as **best-effort** with `NOT-OBSERVED` rows explicitly named.
- Disposition `HU-DRAG-RT-03` as `cannot-reproduce` and author the follow-on diagnostic-only story per `HU-DRAG-RT-04`.

---

## S1-S5 Truth Table (to be filled by retest evidence)

The 5 tracing sites correspond to the PROMPT 706 / 709 instrumentation landed at `7e0c663`. Stage names below are descriptive of the producer-consumer chain documented in story-017 §"Implementation Notes" and §"QA Test Cases"; the precise emit-site identifier on each row (`hand_ui::on_fan_slot_press`, etc.) must match what is actually present in the source at retest time (verify via `git grep -n "S1\\|S2\\|S3\\|S4\\|S5" -- client/src/ui/hand/ client/src/presentation/board_rendering/` immediately before the run).

| Stage | Description | Trace event (expected) | Drag A (BoardCell) | Drag B (Instant fan-plate) | Drag C (cancel empty) | Drag D (invalid cell) |
|-------|-------------|------------------------|--------------------|-----------------------------|----------------------|------------------------|
| **S1** | Pointer Press observer fires on a `FanSlotIndex` entity with `PointerButton::Primary` while `HandUiMode == Staging` | `client::ui::hand::on_fan_slot_press` trace line including `slot_index`, `card_id`, `phase`, `mode` | _PASS / FAIL / NOT-OBSERVED_ | _ditto_ | _ditto_ | _ditto_ |
| **S2** | `HandUiPlacementDragStarted { card, owner_id }` emitted exactly once that tick | `client::ui::hand` trace line on message write; cross-checked by drag-start consumer trace | _PASS / FAIL / NOT-OBSERVED_ | _ditto_ | _ditto_ | _ditto_ |
| **S3** | Drag-start consumer flipped `HandDragSprite` `Visibility` to `Visible` AND board rendering received `GhostPlacementChanged` (BoardCell target) OR fan-plate ghost engaged (Instant target) | `client::presentation::board_rendering` trace on ghost insertion / fan-plate ghost engaged | _PASS / FAIL / NOT-OBSERVED_ | _ditto_ | _ditto_ (no ghost expected; record NOT-OBSERVED) | _ditto_ (invalid target — expect ghost cleared) |
| **S4** | `Pointer<Move>` during active drag emits `HandUiPlacementCursorMoved { world_position }`; `HandDragSprite`'s `Node.left` / `Node.top` track that position the same tick; `card_animations::input_gating` confirms gating state stayed in Staging | `client::ui::hand` cursor-moved trace + `client::card_animations::input_gating=info` confirms gating | _PASS / FAIL / NOT-OBSERVED_ | _ditto_ | _ditto_ | _ditto_ |
| **S5** | `Pointer<Release>` with `PointerButton::Primary` emits `HandUiPlacementDragEnded`; existing drag-ended consumer routes to the correct `PlacementTargetKind` branch (BoardCell drop / Instant fan-plate drop / snap-back / invalid-cell snap-back); `active_drag.clear()` runs; `C2SActivateCard` (or `C2SSubmitPlacement` later) reflects the resolved target authoritatively (no client-side optimism) | `client::ui::hand` drag-ended trace + `server::game=info` confirms `stage_or_update` outcome (or rejection with reason) | _PASS / FAIL / NOT-OBSERVED_ | _ditto_ | _ditto_ (no `C2SActivateCard` send expected — record NOT-OBSERVED) | _ditto_ (expect snap-back; **no** `C2SActivateCard` send) |

**Grey-square attribution legend**:
- A "grey square" event is the visual symptom from PROMPT 698 — a momentary grey square or 1-frame flash that suggests a card was "almost staged" but rolled back, with no corresponding `stage_or_update` server event.
- The truth-table locks the stage at which the grey-square symptom **originates** (which row first transitions to FAIL across A/B/C/D).
- If the symptom is **not observed** in any of A/B/C/D under the documented capture, that itself is evidence — record the disposition as `bug-fixed` or `cannot-reproduce` per `HU-DRAG-RT-03`.
- Special call-out: the 683-era runtime evidence ("8 `C2SActivateCard` sends, zero `stage_or_update` events") implies **S5 (release branch) → server** is the failing edge. The retest must confirm or refute this specifically.

---

## QA Test Cases (manual; no automated test is added by this story)

**Story type** is Integration but the test evidence is a manual runtime trace, not an automated `cargo test`. This is consistent with the test-evidence rule in `.claude/docs/coding-standards.md` for Integration stories that surface "test-green/runtime-broken" divergence — automated tests have already been written (`hand_ui_drag_to_board_cell_test`, etc.) and they pass; the gap is **runtime-only**. The evidence path is `production/qa/evidence/sprint-11-drag-runtime-evidence.md`, not `tests/integration/hand-ui/`.

- **HU-DRAG-RT-T1** — Two-client friend-game session reaches PLACEMENT with both players holding ≥1 card; both clients running the documented `RUST_LOG`; raw logs and screenshots captured for drag-attempts A / B / C / D per §"Friend-game route".

- **HU-DRAG-RT-T2** — For each of drag-attempts A / B / C / D, fill the S1-S5 row in the truth-table with PASS / FAIL / NOT-OBSERVED and an evidence pointer. No row left empty.

- **HU-DRAG-RT-T3** — Disposition `HU-DRAG-RT-03` decided based on T2 rows; matches exactly one of the four outcome branches.

- **HU-DRAG-RT-T4** — Follow-on artifact authored per the chosen branch in `HU-DRAG-RT-04`; the follow-on either lives at `production/epics/hand-ui/story-019-<slug>.md` (repair / repro / tighter-capture) or is explicitly stated as "no follow-on needed" in the evidence file (only for the `bug-fixed` branch).

---

## Test Evidence

**Story Type**: Integration — manual runtime evidence (no new automated tests added by this story).

**Required evidence**:
- `production/qa/evidence/sprint-11-drag-runtime-evidence.md` (NEW; the locked truth-table + disposition + no-claim restatement + follow-on pointer).
- `production/qa/evidence/captures/sprint-11-drag-runtime/` (NEW directory; per-attempt log files + screenshots).

**Status**: [ ] Captured and locked

---

## Out of Scope

- Landing any repair commit under `client/`, `server/`, `shared/`, or `tests/` as part of this story. Repair (if needed) is delegated to a follow-on story authored under `HU-DRAG-RT-04`.
- Modifying any `S2C*` / `C2S*` protocol shape inside this retest. Protocol shape changes (if needed) are a separate story tracked under `S11-TD-NET-*` or a new ticket.
- Closing `S8-QA-001-W1` (two-client GAME_OVER manual gap). This retest covers PLACEMENT drag only.
- Standard-tier accessibility remediation (`QA-COND-0005`). The retest exercises mouse + pointer flow only; keyboard / focus equivalent of drag is owned by stories 014 / 015 and is **not** in scope here.
- Playtest / fun-hypothesis validation (`QA-COND-0006`).
- Activating Sprint 11. Activation is a separate prompt (`/sprint-plan sprint-11`).
- Sprint 11 QA plan authoring (`/qa-plan sprint`); that is a separate prerequisite for `/dev-story` to run on this story.
- Modifying `production/sprint-status.yaml`, `production/stage.txt`, or `production/sprints/sprint-11.md`.

---

## Dependencies

- **Depends on** (already on `main`, no work needed):
  - Story 017 (`HU-card-drag MVP — Press / Move / Release producers + sprite follow`, commit `00ffe89`, PROMPT 696).
  - `S11-HU-DRAG-DROP-NON-INSTANT-001` (`handle_placement_drag_ended` gate widened for all `PlacementTargetKind`, commit `cbb2565`, PROMPT 697 / 710).
  - `S11-OBS-GREY-SQUARE-ATTRIBUTION-001` (S1-S5 instrumentation at 5 sites, commit `7e0c663`, PROMPT 706 / 709).
- **Depends on** (sprint-level prerequisites that gate `/dev-story` for this story):
  - Sprint 11 activation via `/sprint-plan sprint-11`.
  - Sprint 11 QA plan authoring via `/qa-plan sprint`.
  - `/story-readiness` pass on this story file.
- **Unlocks** (depending on disposition):
  - If `bug-reproduced`: follow-on repair story authored at `production/epics/hand-ui/story-019-<slug>.md`.
  - If `cannot-reproduce`: follow-on diagnostic-only story with tighter capture spec.
  - If `bug-fixed` or `third-party-limitation`: closes the open question from PROMPT 762 candidate #1 (gameplay-blocking).

---

## Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Retest is non-reproducible within the 1.0-day time-box; truth-table locked as best-effort with NOT-OBSERVED rows | Medium | High | Time-box enforced. `cannot-reproduce` disposition is a valid outcome of this story; follow-on diagnostic-only story is authored. |
| Retest reveals an additional, unrelated bug (e.g., AUCTION-asymmetric, HUD timer drift) outside the S1-S5 surface | Medium | Medium | Out-of-scope findings are noted in the evidence file but **not** repaired in this story. A separate Sprint 12 candidate is authored if severity warrants. |
| Retest tempts a reviewer to fold a repair commit into this story | Medium | Medium | Acceptance criterion `HU-DRAG-RT-05` requires zero edits under `client/` / `server/` / `shared/` / `tests/`. Verified by `git diff --stat` on the worker branch. |
| Retest is conducted only on Chrome stable and a Firefox-side divergence is missed | Low | Medium | §"Friend-game route" step 2 requires two distinct browsers. If only one browser is feasible during retest, this is noted as a follow-on risk and a `third-party-limitation` follow-up is scoped. |
| Retest interpretation conflates symptom (grey square) with cause (which S1-S5 row first FAILs) | Medium | High | Truth-table requires the **originating** FAIL row to be named, not just the visible symptom; `HU-DRAG-RT-03` requires the disposition to name the offending stage explicitly. |
| Concurrent root-checkout race during retest evidence write damages `production/qa/evidence/sprint-11-drag-runtime-evidence.md` | Low | Medium | Per 2026-05-13 orchestrator override, only one shared-status writer runs at a time. This story does not touch shared status files; evidence file is a single new write. |

---

## Verification (orchestrator-side, before worker dispatch)

These are sanity checks for the orchestrator emitting the retest prompt, not for the worker:

- `production/sprint-status.yaml` `sprint:` field is **not** bumped by this story authoring.
- `production/stage.txt` reads `Polish` and is unchanged.
- `production/sprints/sprint-11.md` status block reads `draft` and is unchanged.
- The PROMPT 761 Polish->Release gate-check FAIL evidence at `production/gate-checks/gate-polish-release-2026-05-12.md` is preserved.
- `git diff --check` and `git diff --cached --check` pass before commit.

---

## Authoring Trail

- 2026-05-13 — PROMPT 766 — Story file authored as Sprint 11 draft Must Have. Sprint 11 not activated. No code changes, no smoke / gate / QA / `/dev-story` / `/story-done` run.
