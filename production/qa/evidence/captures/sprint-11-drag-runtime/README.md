# Sprint 11 Drag Runtime Retest — Capture Directory

> **Story**: S11-DRAG-RUNTIME-RETEST-001 / `production/epics/hand-ui/story-018-drag-runtime-retest.md`
> **Evidence file**: `../../sprint-11-drag-runtime-evidence.md`
> **Disposition**: `cannot-reproduce` (see evidence file for time-box reasoning)
> **Status**: NO RAW CAPTURES — the 1.0-day retest time-box could not be exercised in the
> automated worker that ran PROMPT 778. The friend-game route is operator-driven (two
> browser tabs, manual drag-and-drop, screenshots, raw `RUST_LOG` capture). No raw
> client / server logs and no screenshots are present in this directory.

## How to populate this directory in the follow-on operator-driven run

When the follow-on retest is dispatched (see evidence file §"Follow-on artefact"), the
operator should place artefacts here following the existing
`production/qa/evidence/captures/manual-friend-game-evidence-YYYY-MM-DD/` precedent:

| Artefact | Filename | Source |
|---|---|---|
| Server log | `server.log` | `cargo run -p server --release` redirected stderr/stdout |
| Client A log | `client-a.log` | browser devtools console export, or trunk-served client redirected stderr |
| Client B log | `client-b.log` | same as above for the second browser tab |
| Drag A screenshot | `client-a-drag-a-placement.png` | standard unit dropped on BoardCell (release frame) |
| Drag B screenshot | `client-a-drag-b-placement.png` | Instant card dropped on fan plate (release frame) |
| Drag C screenshot | `client-a-drag-c-placement.png` | drag cancelled mid-air, release over empty space |
| Drag D screenshot | `client-a-drag-d-placement.png` | drag released over invalid lane / out-of-range cell |
| Command summary | `command-summary.md` | exact commit, branch, OS/target, commands, ports, env vars, tool versions, start/stop times |

The exact `RUST_LOG` invocation and the per-drag flow are in
`production/epics/hand-ui/story-018-drag-runtime-retest.md` §"Reproduction Recipe".

## Redaction

Per the existing manual-friend-game runbook (see
`production/qa/evidence/manual-friend-game-evidence-runbook.md` §"Evidence Output
Package"), redact local usernames, raw room codes, machine identifiers, and transient
secrets before committing artefacts.
