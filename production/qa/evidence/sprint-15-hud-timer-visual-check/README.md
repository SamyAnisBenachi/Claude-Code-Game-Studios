# Sprint 15 HUD Timer Eyeball Visual Check -- Evidence Slot (Human-Operator Capture Prep)

> **Story**: `S11-HUD-TIMER-EYEBALL-VISUAL-001`
> **Story file**: `production/epics/hud/story-014-hud-timer-eyeball-visual-check.md`
> **Sprint**: Sprint 15 (Must Have; promoted Should -> Must per PROMPT 997 activation)
> **Sprint disposition**: ACTIVE per PROMPT 997 (activation merge `7a5965e`); Sprint 14 closed-with-conditions per PROMPT 987 preserved unchanged.
> **Stage**: `Polish` (unchanged). **NOT a Release activation.**
> **Story status at this commit**: `ready` -- human-operator capture **NOT YET PERFORMED**. Closure remains gated on a real two-client run + screenshot capture; no LLM `/story-done` authorised.
> **Authoring prompt**: PROMPT 1011 (S15 HUD Timer Eyeball Human Capture Prep -- paperwork-only evidence template)
> **Authoring date**: 2026-05-17
> **Source-of-truth at authoring base**: `origin/main@f3e635d657589ce41b7b1e9667207e0830bfedb0` (PROMPT 1010 commit `story-done(s15): close S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001-ROWFLIP (PROMPT 1010)`)
> **Worktree**: `D:/tmp/ccgs-prompt-1011`
> **Branch**: `prompt-1011-hud-timer-human-capture-prep`

---

## Purpose of This Document

This README is an **evidence-slot reservation and human-operator runbook** authored by PROMPT 1011 to prepare for the Sprint 15 manual two-client HUD timer eyeball visual capture session. It does **not** record a verdict, does **not** claim closure, and does **not** modify any production source. It exists so the human operator can run the capture session without ambiguity.

The companion file [`command-summary.md`](./command-summary.md) holds the exact PowerShell preamble + server / client launch commands the operator pastes into terminals.

When the operator completes the capture session, they:

1. Drop the three screenshot PNGs into this directory next to this README.
2. Append a "Run Results" section to this README (template provided below) recording the build commit, the eyeball verdict, and per-phase observations.
3. On `FAIL` or `NEEDS-FOLLOW-ON`, author a follow-on HUD story file under `production/epics/hud/`. On `PASS`, no follow-on is authored.
4. After the evidence README is updated, a **separate** `/story-done` paperwork prompt (NOT this prompt, NOT a worker) flips the Sprint 15 row `S11-HUD-TIMER-EYEBALL-VISUAL-001` from `ready` to `done` and updates `production/sprint-status.yaml`.

---

## Evidence Path Redirect Note (Sprint 13 -> Sprint 14 -> Sprint 15 Carry)

Per the PROMPT 997 Sprint 15 activation row notes and the PROMPT 1002 QA plan (`production/qa/qa-plan-sprint-15.md` row 143), the canonical evidence path for this capture session has been **redirected from the Sprint 13 path named in the story file** to this Sprint 15 path:

- Story file `AC2` / `AC8` / Likely-Files table: `production/qa/evidence/sprint-13-hud-timer-visual-check/` (the original Sprint 13 path; PROMPT 828 reserved the slot there).
- Activation row notes + QA plan + this README: `production/qa/evidence/sprint-15-hud-timer-visual-check/` (the Sprint 15 path; NEW).

The story file paths are **preserved verbatim** to maintain the multi-sprint carry trail (Sprint 13 -> Sprint 14 -> Sprint 15). The Sprint 14 path was never materialised (Sprint 14 closed-with-conditions per PROMPT 987 with the row carried forward unchanged). PROMPT 1011 does **not** modify the story file.

**Operator action**: cite this Sprint 15 path in the "Run Results" section. The Sprint 13 README at `production/qa/evidence/sprint-13-hud-timer-visual-check/README.md` is the prior-sprint paperwork-only reservation (PROMPT 828) and remains unchanged on `origin/main`.

---

## Status / No-Claim Banner (verbatim from story file)

PROMPT 1011 (this evidence-prep prompt) does **not**:

- Run `/story-readiness`, `/dev-story`, `/story-done`, `/smoke-check`, `/team-qa`, `/gate-check`, `/release-check`, or `/qa-plan` on this story.
- Modify `production/sprint-status.yaml`, `production/sprints/sprint-15.md`, `production/sprints/sprint-14.md`, `production/sprints/sprint-13.md`, `production/sprints/sprint-12.md`, `production/sprints/sprint-11.md`, `production/sprints/sprint-10.md`, or any other sprint plan file.
- Modify `production/stage.txt` (remains `Polish`).
- Modify `production/qa/qa-plan-sprint-15.md` or any other QA-plan / smoke / Team-QA / gate-check / release-check artifact.
- Modify the story file `production/epics/hud/story-014-hud-timer-eyeball-visual-check.md`.
- Modify any code under `client/`, `server/`, `shared/`, or `tests/`.
- Modify `Cargo.toml`, `Cargo.lock`, `.cargo/`, or `.github/`.
- Retry the PROMPT 761 Polish->Release gate-check.

This evidence slot does **not** claim: public release readiness, release-candidate readiness, full game completion, broad / Standard-tier accessibility completion (`QA-COND-0005`), playtest / fun-hypothesis validation (`QA-COND-0006`), full playable-client manual QA, two-client GAME_OVER closure (`S8-QA-001-W1`), or final-art / asset-production completion (`PAW-TD-*-a`).

**No optimistic client-side authority is introduced or proposed.** The HUD timer is read-only over server-authoritative phase state via `Res<CurrentClientPhase>` (ADR-021 binding); this capture session verifies its visual rendering only and does not modify it.

Sprint 10 / 11 / 12 / 13 / 14 dispositions preserved unchanged. PROMPT 761 Polish->Release gate-check FAIL evidence preserved at `production/gate-checks/gate-polish-release-2026-05-12.md`. `TQ-S12-C1..C7` preserved verbatim. `TQ-S12-C2` binding: no third same-scope retest of Sprint 12 `hand-ui/story-019-drag-runtime-retest-tighter-capture.md` is authorised.

---

## Build / Source Metadata (to be filled by human operator)

The operator records the values from the real capture session in the table below. Until then, the placeholders below show the PROMPT 1011 authoring base; the operator MUST overwrite them with the live capture values:

| Field | Value at PROMPT 1011 authoring (placeholder) | Value at human-operator capture (TO BE FILLED) |
|---|---|---|
| Build commit (server) | `origin/main@f3e635d` (PROMPT 1010 tip) | `<paste commit hash here>` |
| Build commit (clients) | `origin/main@f3e635d` (PROMPT 1010 tip) | `<paste commit hash here>` |
| Repo checkout path | `D:/_DEV/Work/Claude-Code-Game-Studios` (operator-chosen) | `<paste checkout path here>` |
| Worktree path | `D:/tmp/ccgs-prompt-1011` (PROMPT 1011 paperwork worktree) | `<operator's capture worktree or checkout>` |
| Branch | `prompt-1011-hud-timer-human-capture-prep` (PROMPT 1011 paperwork branch) | `<operator's capture branch>` |
| Capture date | n/a | `<YYYY-MM-DD>` |
| Operator | CLI agent (paperwork-only; no rendering surface) | `<human operator name>` |
| Server build mode | n/a | `cargo run -p server --bin server` (release or dev; record which) |
| Client build mode | n/a | `cargo run -p client --bin client` (record dev / release; if browser/WASM via Trunk, record `trunk serve --release` instead) |
| Cargo Resource Policy applied | n/a -- no cargo invoked by PROMPT 1011 | `<yes / no>` (see [`command-summary.md`](./command-summary.md)) |
| Disk-cleanup authorization used | n/a | `<yes / no -- if yes, free space before/after>` |
| Eyeball verdict | n/a | `PASS` / `FAIL` / `NEEDS-FOLLOW-ON` (one of three; record in "Run Results" below) |

**The operator MUST overwrite the right-hand column** with real values at capture time; the placeholders are not evidence on their own.

---

## Pre-Capture Checklist (Human Operator)

Walk through this list **before** launching server/client. Each box is a precondition for a clean run.

- [ ] **Latest `origin/main` checked out**. Pull `origin/main` to the latest tip (PROMPT 1011 authoring base is `f3e635d`; any later tip that does not regress the HUD timer surface is acceptable). Record the actual commit hash in the table above.
- [ ] **Sprint 15 still active**. `git show origin/main:production/sprint-status.yaml | grep -E '^sprint:|^status:'` reports `sprint: 15` / `status: active`. If Sprint 15 has been closed since PROMPT 1011 authoring, escalate to producer before capturing -- the row may have been carried forward.
- [ ] **`S11-HUD-TIMER-EYEBALL-VISUAL-001` row still `ready`**. `git show origin/main:production/sprint-status.yaml | grep -A1 S11-HUD-TIMER-EYEBALL-VISUAL-001` reports `status: ready`. If it is already `done`, do NOT capture -- the closure was made by a paperwork prompt that bypassed evidence; escalate.
- [ ] **No human-operator capture has landed since PROMPT 1011**. `ls production/qa/evidence/sprint-15-hud-timer-visual-check/` shows only `README.md` and `command-summary.md` (no screenshots). If screenshots are already present, the capture has been run; do not re-run without producer authorisation.
- [ ] **Disk space**. `D:` drive free space >= 40 GB. If below 40 GB, see "Stale Target Cleanup Authorisation" in [`command-summary.md`](./command-summary.md) before running `cargo`.
- [ ] **Rust toolchain**. `rustc --version` reports a stable channel toolchain matching `rust-toolchain.toml`. (Sprint 15 base does not require a specific minor version; any stable that compiles the workspace at `origin/main@f3e635d` is fine.)
- [ ] **Cargo Resource Policy understood**. The operator has read the PowerShell preamble in [`command-summary.md`](./command-summary.md) and will paste it into the terminal session **before** any `cargo` command.
- [ ] **Friend-game route accessible**. The operator knows how to drive the lobby from a fresh client into class confirmation, room creation, room-code join, and `DraftInitial`.

---

## Capture Plan

The operator captures **three screenshots**, one per timed phase, **mid-countdown**. The Sprint 15 activation row notes + QA plan name the phases and target capture windows:

| Slot | Phase | Server-authoritative countdown | Recommended capture window | Suggested filename (worker decides exact name) |
|------|-------|--------------------------------|-----------------------------|------------------------------------------------|
| 1 | `DraftInitial` | 45 s | ~20-30 s into the phase (i.e., 15-25 s remaining) | `hud-timer-draft-initial-mid.png` **OR** the story-file-named `draft-initial-timer.png` |
| 2 | `DraftShop` | 30 s | ~12-18 s into the phase (i.e., 12-18 s remaining) | `hud-timer-draft-shop-mid.png` **OR** `draft-shop-timer.png` |
| 3 | `Placement` | 10-12 s (tuning-dependent; see `assets/config/game_config.ron`) | ~4-6 s into the phase (i.e., 4-8 s remaining) | `hud-timer-placement-mid.png` **OR** `placement-timer.png` |

**Two filename conventions exist** because the story file (PROMPT 819) names `draft-initial-timer.png` / `draft-shop-timer.png` / `placement-timer.png` whereas the Sprint 15 QA plan (PROMPT 1002) names the `hud-timer-<phase>-mid.png` variant. Either is acceptable -- the operator picks one set and uses it consistently for all three slots. Record the chosen set in the "Run Results" section.

**Capture mode** (verbatim from PROMPT 828 / QA plan): manual two-client friend-game session reaching each timed phase. At minimum one screenshot per phase capturing the visible countdown mid-phase; ideally screenshots at start + midpoint + end per phase if the operator has bandwidth.

**Capture target surface**: per `design/gdd/hud.md` Rule 12, the HUD itself does not own the timer rendering -- `DraftInitial` / `DraftShop` / `DraftAuction` timers are owned by Shop/Auction UI; the `Placement` timer is owned by Hand UI. The story title says "HUD timer" because the deferred Sprint 10 W2 finding referenced the visible countdown regardless of which client subsystem owns it; the **manual evidence is valid for AC1-AC3 whichever subsystem actually owns the rendering**. If the eyeball verdict surfaces a regression and a follow-on story is authored, the follow-on can scope the fix to the correct subsystem at that point.

---

## During-Capture Steps (Human Operator)

1. **Launch the server.** Use the launch pattern in [`command-summary.md`](./command-summary.md). Confirm the server reports listening on `127.0.0.1:5000` (or whichever `SERVER_PORT` the operator chose).
2. **Launch client A.** Use the same launch pattern. Confirm client A connects, reaches the lobby, picks a class, confirms class.
3. **Launch client B.** Same pattern. Confirm client B connects and reaches the lobby.
4. **Create friend-game room** from client A; **join via room code** from client B. Both confirm class.
5. **Advance to `DraftInitial`** (45 s countdown begins). Wait until ~20-30 s have elapsed (timer reads ~15-25 s). Capture screenshot 1. Save as PNG directly under this directory.
6. **Both clients place their first units / submit draft picks** as the round design requires to advance the phase. Reach `DraftShop` (30 s countdown). Wait until ~12-18 s have elapsed (timer reads ~12-18 s). Capture screenshot 2.
7. **Both clients complete shop interactions** as needed to advance the phase. Reach `Placement` (10-12 s countdown). Wait until ~4-6 s have elapsed (timer reads ~4-8 s). Capture screenshot 3.
8. **Stop the server / clients gracefully.** Apply any stale-target cleanup per [`command-summary.md`](./command-summary.md) if disk pressure was hit during the session.
9. **Fill in the "Run Results" section** below.
10. **On `PASS`**, commit only the three screenshots + the updated README. **On `FAIL` / `NEEDS-FOLLOW-ON`**, author the follow-on story file under `production/epics/hud/` and commit it alongside the evidence in the same paperwork commit (or a separate one -- producer decides).

---

## Run Results (TO BE FILLED BY HUMAN OPERATOR)

> **Operator**: replace this block (everything between the `<<RUN RESULTS START>>` and `<<RUN RESULTS END>>` markers) with the real capture results. Do not delete the markers; leave them in place so a future re-run can find the slot.

<<RUN RESULTS START>>

### Capture Session Metadata

(Fill the table from "Build / Source Metadata" above.)

### Phase 1 -- `DraftInitial` (45 s)

- Screenshot file: `<filename>.png`
- Capture point (seconds into phase): `<N>` s (i.e., timer read `<45 - N>` s)
- Observations (4-8 sentences): timer position on screen, font size, contrast against background, animation behaviour (tick / smooth countdown), any clipping / off-screen / wrong-colour / missing-tick / wrong-phase-label / unexpected hidden state. Note the responsible subsystem if known (HUD, Shop/Auction UI per GDD Rule 12, or Hand UI).
- Per-phase verdict: `PASS` / `FAIL` / `NEEDS-FOLLOW-ON`

### Phase 2 -- `DraftShop` (30 s)

- Screenshot file: `<filename>.png`
- Capture point (seconds into phase): `<N>` s
- Observations: as above
- Per-phase verdict: `PASS` / `FAIL` / `NEEDS-FOLLOW-ON`

### Phase 3 -- `Placement` (10-12 s)

- Screenshot file: `<filename>.png`
- Capture point (seconds into phase): `<N>` s
- Actual countdown observed (`10` / `11` / `12` s -- depends on `game_config.ron` placement_timer_ms): `<N>` s
- Observations: as above
- Per-phase verdict: `PASS` / `FAIL` / `NEEDS-FOLLOW-ON`

### Overall Eyeball Verdict

- **Verdict**: `PASS` / `FAIL` / `NEEDS-FOLLOW-ON` (one of three)
- **Rationale** (4-8 sentences): summarise the per-phase observations, name the worst phase if applicable, name the responsible subsystem if a regression was observed.
- **Follow-on story authored**: `yes (path: production/epics/hud/story-<NNN>-<slug>.md)` / `no -- verdict was PASS`

### Forbidden-Change Self-Check (Operator)

Confirm by inspection:

- [ ] No file under `client/`, `server/`, `shared/`, `tests/` modified by the capture commit.
- [ ] No `Cargo.toml`, `Cargo.lock`, `.cargo/`, `.github/` modified.
- [ ] No `production/sprint-status.yaml`, `production/sprints/sprint-15.md`, `production/qa/qa-plan-sprint-15.md`, `production/stage.txt`, `production/gate-checks/*` modified.
- [ ] Story file `production/epics/hud/story-014-hud-timer-eyeball-visual-check.md` NOT modified by the capture commit (separate `/story-done` paperwork prompt owns the status flip).
- [ ] If `FAIL` / `NEEDS-FOLLOW-ON`: a new follow-on story file under `production/epics/hud/` is the only new story file; no other epic touched.

<<RUN RESULTS END>>

---

## Explicit No-Claim Restatement (after capture)

Even after a `PASS` verdict lands, this evidence run does **NOT**:

- Claim release readiness or release-candidate readiness.
- Claim full game completion.
- Claim broad / Standard-tier accessibility completion (`QA-COND-0005` remains accepted-risk / friend-game scope).
- Claim playtest / fun-hypothesis validation (`QA-COND-0006` remains accepted-risk / deferred).
- Claim closure of `S8-QA-001-W1` (manual / browser two-client GAME_OVER gap; remains OPEN).
- Claim closure of any `PAW-TD-*-a` placeholder-art accept-risk across PAW-002..PAW-006.
- Claim closure of `TQ-S12-C1..C7` (preserved verbatim).
- Claim final-art / asset-production completion.
- Retry the PROMPT 761 `Polish->Release` gate-check (FAIL preserved at `production/gate-checks/gate-polish-release-2026-05-12.md`).
- Advance `production/stage.txt` from `Polish` to `Release`.
- Claim closure of the Sprint 12 story 019 drag-runtime "cannot-reproduce" disposition; no underlying drag-runtime bug fix is claimed. `TQ-S12-C2` binding preserved: no third same-scope retest authorised.
- Modify any of Sprint 10 / 11 / 12 / 13 / 14 closeouts; all `closed-with-conditions` dispositions preserved unchanged.

The Sprint 15 row `S11-HUD-TIMER-EYEBALL-VISUAL-001` closure happens only via a **separate** `/story-done` paperwork prompt that runs AFTER this evidence README is filled in. PROMPT 1011 does not author or trigger that follow-on.

---

## Cross-Links

- **Story file**: `production/epics/hud/story-014-hud-timer-eyeball-visual-check.md` (PROMPT 819 authored 2026-05-14; preserved verbatim through Sprint 13 / 14 / 15 carries; story-file `AC2/AC8/Likely-Files` paths name the Sprint 13 evidence path and remain unchanged).
- **Sprint 13 evidence slot (prior carry, paperwork-only reservation)**: `production/qa/evidence/sprint-13-hud-timer-visual-check/README.md` (PROMPT 828 authored 2026-05-14; verdict `BLOCKED-HUMAN-OPERATOR`; preserved unchanged).
- **Sprint 14 evidence slot**: never materialised (Sprint 14 closed-with-conditions per PROMPT 987 carried the row forward).
- **Sprint plan**: `production/sprints/sprint-15.md` Must Have row `S11-HUD-TIMER-EYEBALL-VISUAL-001` (PROMPT 988 draft + PROMPT 997 activation; row promoted Should -> Must to surface the Sprint 13 -> 14 -> 15 carry as a sprint-level blocker).
- **Sprint 15 QA plan**: `production/qa/qa-plan-sprint-15.md` Row 1 (lines 131-158) -- canonical human-operator path sequence and AC mapping.
- **Sprint 15 activation**: PROMPT 997 (merge `7a5965e`); paperwork-only; preserves carried conditions verbatim.
- **PROMPT 1009 integrated `/story-done` batch**: closed three Sprint 15 integrated implementation/spec rows; preserved this human-operator-blocked row unchanged at `status: ready`.
- **PROMPT 1010 row-flip**: closed `S11-CLIENT-CONNECTION-LOST-OBSERVABILITY-001-ROWFLIP` paperwork-only Must Have; preserved this human-operator-blocked row unchanged at `status: ready`.
- **Sprint 10 W2 origin**: `production/qa/smoke-sprint-10-*.md` retry-7 W2 -- the original deferral that this story drains.
- **Sprint 11 close-out carry**: HUD timer eyeball deferred forward as `S11-HUD-TIMER-EYEBALL-VISUAL-001` Should Have row.
- **Sprint 12 close-out deferral**: see `sprint_12_closeout.deferred_into_sprint_13_planning.should_have` in `production/sprint-status.yaml`.
- **Sprint 13 close-out carry**: PROMPT 894 carried the row forward into Sprint 14.
- **Sprint 14 close-out carry**: PROMPT 987 carried the row forward into Sprint 15 with note "human-operator-blocked".
- **GDD reference**: `design/gdd/hud.md` -- TR-HUD-003 (phase label + timer presentation) AND Rule 12 (HUD never displays a timer; phase timers are owned by Shop/Auction UI and Hand UI).
- **ADR-002**: Client-Server Authority -- timer values are server-authoritative; client only renders.
- **ADR-021**: Presentation Layer Architecture -- HUD / Hand UI / Shop-Auction UI timer rendering reads from `Res<CurrentClientPhase>`; no direct `MessageReceiver<...>` in the rendering systems.
- **PROMPT 761 Polish->Release gate-check FAIL**: `production/gate-checks/gate-polish-release-2026-05-12.md` (preserved unchanged; no retry attempted; no retry in scope for Sprint 15).

---

## Closure Path (informational; NOT authorised by PROMPT 1011)

After the operator fills in the "Run Results" section and lands the evidence commit, the recommended closure sequence is:

1. **Producer review** of the verdict + observations + forbidden-change self-check.
2. **On `PASS`**: a dedicated `/story-done` paperwork prompt (single shared-status writer, serialized through the closeout queue) flips the Sprint 15 row from `ready` to `done`, records the evidence path, and preserves all carried conditions verbatim. No code change. No test change.
3. **On `FAIL` / `NEEDS-FOLLOW-ON`**: a follow-on story is authored first (no code change in that authoring prompt); then a `/story-done` paperwork prompt flips the Sprint 15 carry row to `done` with the follow-on story file path recorded as the closure evidence; the follow-on story file becomes the new open row (Sprint 16+ candidate).
4. After the row closes, Sprint 15 close-out follows the PROMPT 894 / PROMPT 987 paperwork-only close-out precedent: `/smoke-check` -> `/team-qa sprint` -> Sprint 15 close-out `/story-done` paperwork closing the sprint at `closed-with-conditions` or `closed` per producer call.

**No LLM is authorised to run any of steps 2-4 before this README's "Run Results" section is filled with a real verdict from a real human operator with a rendering surface.**
