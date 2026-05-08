# S9-QA-001 Defects

## Defect Table

| ID | Severity | Owner / System | Status | Reproduction | Friend-game impact | Workaround | Evidence |
|---|---|---|---|---|---|---|---|
| MANUAL-FG-001 | S2 | QA / Evidence infrastructure | Open | Run S9-QA-001 evidence capture via non-interactive AI agent; agent cannot click or navigate Bevy windowed client applications | Full manually driven two-client GUI route through GAME_OVER cannot be captured by an AI-only run; S8-QA-001-W1 cannot be closed | Execute the run with a human operator who can launch two native client windows or two browser contexts and manually drive the friend-game route per `manual-friend-game-evidence-runbook.md` | `command-summary.md` — Manual Route Execution section |

## Severity Note

MANUAL-FG-001 is classified S2 (blocks the manual route) because it prevents
capturing the full GUI evidence required by S9-QA-001's acceptance criteria.
It is not an S1 (no data safety or launch block). It is a prerequisite
execution limitation, not a product defect.

## Product Defects Found

None. No product defects were encountered during this run because the client
GUI was not reached. All automated regressions and cargo checks passed cleanly.
