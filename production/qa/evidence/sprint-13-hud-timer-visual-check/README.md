# Sprint 13 HUD Timer Eyeball Visual Check -- Evidence Slot

> **Story**: `S11-HUD-TIMER-EYEBALL-VISUAL-001`
> **Story file**: `production/epics/hud/story-014-hud-timer-eyeball-visual-check.md`
> **Sprint**: Sprint 13 (Should Have)
> **Sprint disposition**: ACTIVE per PROMPT 826
> **Stage**: `Polish` (unchanged)
> **Story status at this commit**: Draft -- evidence slot reserved; manual two-client visual run NOT yet performed
> **Authoring prompt**: PROMPT 828 (S13 HUD Timer Eyeball Visual Evidence)
> **Authoring date**: 2026-05-14
> **Source-of-truth at authoring base**: `origin/main@4bf95fa` (PROMPT 827 commit `qa-plan(s13): author Sprint 13 QA plan (PROMPT 827)`)
> **Worktree**: `D:\_DEV\claude-code-game-studios-worktrees\s13-hud-timer-eyeball`
> **Branch**: `work/s13-hud-timer-eyeball-visual`

---

## Status / No-Claim Banner (verbatim from story file)

This story is authored as a Sprint 13 candidate. Sprint 13 is **NOT**
activated by PROMPT 819. Sprint 12 is closed-with-conditions per PROMPT
817 and is not changed by this authoring run.

> **PROMPT 828 paperwork-only note**: Sprint 13 was activated by PROMPT 826
> on 2026-05-14 (after the story file was authored by PROMPT 819).
> Sprint 13 is now ACTIVE. PROMPT 828 (this evidence-slot reservation)
> does NOT change Sprint 13's `active` disposition, the stage (`Polish`),
> Sprint 12 close-out (`closed-with-conditions`), Sprint 11 close-out, or
> Sprint 10 close-out. The story-file no-claim text below is preserved
> verbatim from PROMPT 819 authoring.

PROMPT 819 (story-authoring run) does NOT:

- Activate Sprint 13.
- Modify `production/sprint-status.yaml`.
- Modify `production/sprints/sprint-13.md` or any other sprint plan file.
- Modify `production/stage.txt` (remains `Polish`).
- Modify any `production/session-state/*` file.
- Modify any QA-plan / smoke / Team-QA / gate-check / release-check
  artifact.
- Run `/story-readiness`, `/dev-story`, `/story-done`, `/smoke-check`,
  `/team-qa`, `/gate-check`, `/release-check`, or `/qa-plan` on this
  story.
- Modify any code under `client/`, `server/`, `shared/`, or `tests/`.
- Retry the PROMPT 761 Polish->Release gate-check.

This story does **not** claim: public release readiness,
release-candidate readiness, full game completion, broad / Standard-tier
accessibility completion (`QA-COND-0005`), playtest / fun-hypothesis
validation (`QA-COND-0006`), full playable-client manual QA, two-client
GAME_OVER closure (`S8-QA-001-W1`), or final-art / asset-production
completion.

Sprint 10 / Sprint 11 / Sprint 12 dispositions unchanged. PROMPT 761
Polish->Release gate-check FAIL evidence preserved at
`production/gate-checks/gate-polish-release-2026-05-12.md`.

**No optimistic client-side authority is introduced or proposed by this
story.** The timer is read-only over server-authoritative phase state
via `Res<CurrentClientPhase>` (ADR-021 binding); this story verifies its
visual rendering only and does not modify it.

---

## PROMPT 828 Evidence-Run Status

**Verdict at this commit**: `BLOCKED-HUMAN-OPERATOR`.

**Why blocked**: AC1-AC3 + AC5 + AC8 require a manual two-client run
(browser/WASM or native) with screenshot capture. PROMPT 828 was
executed by a CLI agent without an interactive rendering surface, two
browser/native client processes, or eyeball verification capability.
The agent cannot:

1. Launch a Lightyear server + two client processes locally with
   rendering.
2. Drive the friend-game route through `DraftInitial`, `DraftShop`, and
   `Placement` phases.
3. Capture screenshots of the timer rendering at each target phase.
4. Render a visual PASS / FAIL / NEEDS-FOLLOW-ON verdict by eyeball.

Per the PROMPT 828 brief: *"If the story requires live/native client
execution and the agent cannot visually verify it, record that honestly
as BLOCKED-HUMAN-OPERATOR or PARTIAL with exact last reached step."*

**Exact last reached step**: This evidence-slot README file was created
at `production/qa/evidence/sprint-13-hud-timer-visual-check/README.md`
on branch `work/s13-hud-timer-eyeball-visual` at worktree
`D:\_DEV\claude-code-game-studios-worktrees\s13-hud-timer-eyeball`,
parented on `origin/main@4bf95fa`. No client/server/shared/tests code
was touched. No screenshots were captured. No `cargo`, `trunk`, or
client/server process was launched. The three screenshot slots named in
the story file (`draft-initial-timer.png`, `draft-shop-timer.png`,
`placement-timer.png`) are **NOT** present in this directory at this
commit -- they require a human operator's two-client run to populate.

**What is satisfied by this commit**:

- **AC4** (no production-source change lands): satisfied at this
  commit. `git diff origin/main..HEAD --name-only` is constrained to
  this single evidence README file under
  `production/qa/evidence/sprint-13-hud-timer-visual-check/`.
- **AC6** (Sprint 13 disposition preserved): satisfied. PROMPT 828 does
  NOT modify `production/sprint-status.yaml`,
  `production/sprints/sprint-13.md`, `production/stage.txt`,
  `production/qa/qa-plan-sprint-13.md`, or
  `production/gate-checks/gate-polish-release-2026-05-12.md`.
- **AC7** (no condition closure claimed): satisfied. This README
  explicitly does not claim closure of `S8-QA-001-W1`, `QA-COND-0005`,
  `QA-COND-0006`, or `PAW-TD-*-a`. Standard-tier accessibility is not
  pursued by this evidence slot.
- **AC8** (evidence document slot reserved): satisfied. This README
  records the build commit, the planned three-screenshot slots, the
  current BLOCKED-HUMAN-OPERATOR verdict, no-claim restatement, and
  cross-links to the Sprint 10 smoke retry-7 W2 origin and
  `design/gdd/hud.md`.

**What is NOT satisfied by this commit**:

- **AC1** (manual two-client run executed): NOT performed.
- **AC2** (screenshot per phase captured): NOT captured.
- **AC3** (eyeball verdict recorded): NOT achievable without screenshots.
- **AC5** (follow-on story authored only on FAIL): conditional on AC3;
  defer until human operator records a real verdict.

---

## Build / Source Metadata

| Field | Value |
|---|---|
| Source-of-truth commit | `origin/main@4bf95fa` (PROMPT 827 QA plan commit) |
| Worktree | `D:\_DEV\claude-code-game-studios-worktrees\s13-hud-timer-eyeball` |
| Branch | `work/s13-hud-timer-eyeball-visual` |
| Authoring date | 2026-05-14 |
| Operator | CLI agent (no rendering surface) |
| Cargo resource policy applied | N/A -- no `cargo` command was invoked |
| Disk-cleanup authorization used | N/A -- no disk-pressure threshold hit |

---

## Screenshot Slots (NOT YET CAPTURED -- HUMAN OPERATOR REQUIRED)

When a human operator performs the manual two-client run, three
screenshot files are expected under this directory. The exact filenames
named in the story file (Likely Files table) are:

| Slot | Filename | Phase | Server-authoritative countdown | Capture point |
|------|----------|-------|--------------------------------|---------------|
| 1 | `draft-initial-timer.png` (or `.jpg`) | `DraftInitial` | 45 s | Mid-countdown (e.g., ~30 s remaining) |
| 2 | `draft-shop-timer.png` | `DraftShop` | 30 s | Mid-countdown (e.g., ~15 s remaining) |
| 3 | `placement-timer.png` | `Placement` | 10-12 s (tuning-dependent) | Mid-countdown (e.g., ~6 s remaining) |

**None of the three files are present in this directory at PROMPT 828
commit time.** They are reserved for the human operator's run.

The QA plan (`production/qa/qa-plan-sprint-13.md` line 545) clarifies
the capture mode:

> "Capture mode: manual two-client friend-game session reaching each
> timed phase. At minimum one screenshot per phase capturing the
> visible countdown at start; ideally screenshots at start + midpoint +
> end per phase."

---

## Cross-Links

- **Story file**: `production/epics/hud/story-014-hud-timer-eyeball-visual-check.md`
- **Sprint plan**: `production/sprints/sprint-13.md` Should Have row
  `S11-HUD-TIMER-EYEBALL-VISUAL-001`
- **Sprint 13 QA plan**: `production/qa/qa-plan-sprint-13.md`
  (see "HUD Timer Eyeball Visual" section at line 542+)
- **Sprint 10 W2 origin**: `production/qa/smoke-sprint-10-*.md`
  retry-7 W2 (the original deferral that this story drains)
- **Sprint 11 close-out carry**: HUD timer eyeball deferred forward as
  `S11-HUD-TIMER-EYEBALL-VISUAL-001` Should Have row
- **Sprint 12 close-out deferral**: see
  `sprint_12_closeout.deferred_into_sprint_13_planning.should_have` in
  `production/sprint-status.yaml`
- **GDD reference**: `design/gdd/hud.md`
- **ADR-002**: Client-Server Authority (timer values are
  server-authoritative; client only renders)
- **ADR-021**: Presentation Layer Architecture (timer reads from
  `Res<CurrentClientPhase>`; no direct `MessageReceiver<...>` in the
  rendering system)
- **PROMPT 761 Polish->Release gate-check FAIL**:
  `production/gate-checks/gate-polish-release-2026-05-12.md` (preserved
  unchanged; no retry attempted)

---

## Observation Noted (NOT a verdict; for human operator follow-up)

While reading `design/gdd/hud.md` to confirm the timer rendering
boundary, PROMPT 828 noted that the GDD's **Rule 12 (line 142)**
states explicitly:

> "Rule 12 -- HUD never displays a timer. The PLACEMENT timer is owned
> by Hand UI (hand-ui.md Rule 11). The DRAFT_INITIAL, DRAFT_SHOP, and
> DRAFT_AUCTION timers are owned by Shop/Auction UI. The
> `timer_duration_ms` field of `S2CPhaseChanged` is read by those
> systems, not by HUD."

This contrasts with the story file (`story-014-hud-timer-eyeball-visual-check.md`
line 102), which says:

> "client/src/ui/hud/ (per ADR-021): HUD timer text entity is
> pre-pooled at session start and updated each frame from
> Res<CurrentClientPhase> and the phase-timer field of the latest
> S2CPhaseChanged."

**This is logged as an observation only; PROMPT 828 does NOT resolve
the boundary question.** The human operator's manual run is the
authority on what is actually rendered on screen. If the human run
shows the countdown rendered by `client/src/ui/hand/*` or
`client/src/ui/shop_auction/*` instead of `client/src/ui/hud/*`, the
manual evidence is still valid for AC1-AC3 -- it captures the
**phase-timer rendering** regardless of which client subsystem owns
the rendering. A follow-on paperwork prompt (NOT this evidence slot)
may refine the story title and the `client/src/ui/hud/` path
reference if needed. This is **out of scope** for PROMPT 828.

---

## Handover Notes for a Human Operator

When you (the human operator) come to fill in this evidence slot, the
recommended steps are (no warranty -- adapt to current build):

1. Build a friend-game-capable client + server from `origin/main` HEAD
   (or a Sprint 13 wave HEAD). PowerShell / Windows / MSVC notes for
   Cargo resource policy live in `production/qa/qa-plan-sprint-13.md`
   ("Cargo Resource Policy" section).
2. Launch the server (native binary) and two clients (browser via
   `trunk serve --release` or native debug) on the same machine or
   LAN.
3. From client A, create a friend-game room; from client B, join it
   via the room code. Both confirm classes.
4. Reach `DraftInitial` (45 s countdown). Capture screenshot
   `draft-initial-timer.png` mid-countdown showing the visible timer
   readout. Note the visible value, position, contrast, and any
   rendering anomalies (clipping, off-screen, wrong font, wrong
   colour, missing tick animation).
5. Proceed to placement to advance the round; reach `DraftShop` (30 s
   countdown). Capture `draft-shop-timer.png` mid-countdown.
6. Submit placement; once `Placement` phase fires (10-12 s
   countdown), capture `placement-timer.png` mid-countdown.
7. Write the eyeball verdict (PASS / FAIL / NEEDS-FOLLOW-ON) into a
   new section appended to this README -- include brief observations
   (4-8 sentences) per phase.
8. If the verdict is FAIL or NEEDS-FOLLOW-ON, author a follow-on
   story file under the same HUD epic (or under hand-ui / shop-auction
   if the GDD Rule 12 ownership observation above resolves toward
   those subsystems) with the precise visual regression and a
   recommended remediation scope. **No production-source code change
   lands under this story.**

If the build / friend-game flow surfaces a blocker that prevents one
or more phases from being reached, document the blocker in this README
(append a "Partial Run" section) and proceed to author a follow-on
story scoped to the blocker, not to a visual fix.

---

## Status Line for PROMPT 828

`828: S13-HUD-TIMER-EYEBALL-VISUAL: BLOCKED-HUMAN-OPERATOR`

Visual confirmation was **not achieved** by PROMPT 828. The evidence
slot is reserved; the manual two-client run remains pending a human
operator with a rendering surface.
