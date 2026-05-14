# Sprint 11 Drag Runtime — Tighter-Capture Attempt (2026-05-14 CLI dispatch, second time-box)

> **Story**: S11-DRAG-RUNTIME-RETEST-TIGHTER-CAPTURE-001 / `production/epics/hand-ui/story-019-drag-runtime-retest-tighter-capture.md`
> **Evidence file**: `../../sprint-11-drag-runtime-evidence-tighter.md`
> **Disposition**: `cannot-reproduce` (**second time-box exhaustion** — first was PROMPT 778 / story 018)
> **PROMPT**: 807
> **Date**: 2026-05-14
> **Status**: NO RAW CAPTURES — the 1.5-day tighter-capture time-box could not be
> exercised in the automated worker that ran PROMPT 807, identically to PROMPT 778
> for story 018. The friend-game route is operator-driven (two browser tabs,
> manual drag-and-drop, **frame-level video**, synchronised UTC-millisecond
> log prefixing, raw `RUST_LOG=…=trace + lightyear=debug + server::game=debug`
> capture). No raw client / server logs, no screenshots, and no video are present
> in this directory.

## Why this directory exists empty (per the second-time `cannot-reproduce` rule)

Story 019 §"Time-box" and `HU-DRAG-RT-19-04` `cannot-reproduce` rule both
require that the second-time disposition NOT silently close the underlying
concern. This empty directory is the audit-trail artefact:

1. The tighter-capture protocol was attempted in CLI dispatch on 2026-05-14
   (PROMPT 807) on `work/s11-drag-runtime-retest-tighter-capture` from
   `origin/main@d8d0196`.
2. The structural unavailability of an interactive operator session inside a
   non-interactive CLI dispatch (no browser pointer, no real wall-clock
   shells, no screen recorder) means no S1-S5 row could be filled with PASS /
   FAIL on any of A / B / C / D.
3. Per the `cannot-reproduce` (second time) rule, no third same-scope retest
   is authored. The escalation is to **Sprint 13 expanded-tracing scope**
   (see evidence file §"Follow-on artefact").

## How to populate this directory in the Sprint 13 expanded-tracing run

When the Sprint 13 expanded-tracing follow-on is dispatched (see evidence
file §"Follow-on artefact"), the operator should place artefacts here (or
in a sibling dated subdir) following the existing
`production/qa/evidence/captures/manual-friend-game-evidence-YYYY-MM-DD/`
precedent:

| Artefact | Filename | Source |
|---|---|---|
| Server log | `server.log` | `cargo run -p server --release` with UTC-millisecond prefix wrapper |
| Client A log | `client-a.log` | trunk-served client A stderr with UTC-millisecond prefix wrapper |
| Client B log | `client-b.log` | trunk-served client B stderr with UTC-millisecond prefix wrapper |
| Drag A video | `client-a-drag-a-placement.mp4` | standard unit dropped on BoardCell — 60 fps preferred |
| Drag B video | `client-a-drag-b-placement.mp4` | Instant card dropped on fan plate |
| Drag C video | `client-a-drag-c-placement.mp4` | drag cancelled mid-air, release over empty space |
| Drag D video | `client-a-drag-d-placement.mp4` | drag released over invalid lane / out-of-range cell |
| Truth-table notes | `truth-table.md` | row-by-row PASS / FAIL / NOT-OBSERVED per stage × drag-attempt with log-line and video-timestamp pointers |
| Command summary | `command-summary.md` | exact commit, branch, OS/target, commands, ports, env vars, tool versions, start/stop times, OBS / ShareX config (FPS, encoder) |

The exact upgraded `RUST_LOG` invocation is in
`production/epics/hand-ui/story-019-drag-runtime-retest-tighter-capture.md`
§"Reproduction Recipe (tighter capture)".

## Redaction

Per the existing manual-friend-game runbook (see
`production/qa/evidence/manual-friend-game-evidence-runbook.md` §"Evidence Output
Package"), redact local usernames, raw room codes, machine identifiers, and
transient secrets before committing artefacts.
