# Coordination Rules

## Current GCS Orchestrator Contract

The canonical live orchestration contract is the top override block in
`production/session-state/codex-orchestrator-state.md`:
`Current Operating Rules (2026-05-13 override)`.

Use that override when older docs disagree.

Operational rules:

- Put a state label directly above every agent-window disposition or launch
  prompt.
- `CLEAR` means the user can close that agent window; no reply is needed.
- `REPONDRE` means the user should paste the response into that same existing
  agent window.
- `RELANCER` means rerun the same work with a corrected prompt.
- `NEW` means launch the prompt in a new agent window.
- Use these exact labels: `CLEAR -- PROMPT N`, `REPONDRE -- PROMPT N`,
  `RELANCER -- PROMPT N`, `NEW -- PROMPT N`.
- New prompts start with `PROMPT N -- Short Task Name`.
- Prompt final line is one line only: `N: TICKET-ID: STATUS`. No delimiter line.
- STATUS must be a real outcome word, not a color name and not the literal word
  `STATUS`.
- Use Game Studio roles in prompts (`ui-programmer`, `gameplay-programmer`,
  `network-programmer`, `qa-lead`, `producer`, `ux-designer`, `art-director`,
  `technical-artist`, `audio-director`, `sound-designer`).
- Use `liv-bevy-018` for Bevy reads/reviews/edits and `liv-bevy-lightyear` for
  Lightyear/networking reads/reviews/edits.

## Worktree Isolation

Parallel implementation uses one Git worktree and one branch per story.

- Root checkout: `D:\_DEV\Work\Claude-Code-Game-Studios` in the current local
  environment
  - Reserved for orchestrator, integration merges, story-done, CI triage.
- Worker checkout: `D:\_DEV\claude-code-game-studios-worktrees\<story-id>`
  - Reserved for exactly one story implementation.
- Worker branch: `work/<story-id>-<short-slug>`

Workers must:

- Create or use their assigned worktree before editing implementation files.
- Commit only their story changes.
- Use detailed commit messages. The subject stays short, but the body must
  include Summary, Files, Verification, and Notes so another agent can
  understand the commit from git history without reopening the full context.
- Push their story branch with `git push -u origin work/<story-id>-<short-slug>`.
- Report branch name, commit hash, files changed, local checks, and CI run id if
  available.
- Never merge into `main`.
- Never push `main`.
- Never run `/story-done` unless explicitly assigned by the orchestrator.
- Never edit `production/sprint-status.yaml`,
  `production/session-state/active.md`, or
  `production/session-state/codex-orchestrator-state.md` unless the prompt is
  explicitly a serialized tracking/closure task.

The orchestrator must:

- Assign story ids, worktree paths, and branch names.
- Merge worker branches into `main`.
- Push `main`.
- Serialize shared tracking updates:
  - `production/sprint-status.yaml`
  - `production/session-state/active.md`
  - story completion notes
- Write integration, story-done, and tracking commits with detailed bodies:
  original worker commit or story, files changed, local checks, CI status if
  known, blockers/advisories, and any skipped verification.
- Keep at most one shared-status writer active at a time.
- Maximize safe parallelism only when file ownership and architecture ownership
  are disjoint; do not invent work to fill a quota.

Existing workers already launched in the shared checkout may finish normally.
All new implementation workers should use this worktree flow.
