# Coordination Rules

## Worktree Isolation

Parallel implementation uses one Git worktree and one branch per story.

- Root checkout: `D:\_DEV\claude-code-game-studios`
  - Reserved for orchestrator, integration merges, story-done, CI triage.
- Worker checkout: `D:\_DEV\claude-code-game-studios-worktrees\<story-id>`
  - Reserved for exactly one story implementation.
- Worker branch: `work/<story-id>-<short-slug>`

Workers must:

- Create or use their assigned worktree before editing implementation files.
- Commit only their story changes.
- Push their story branch with `git push -u origin work/<story-id>-<short-slug>`.
- Report branch name, commit hash, files changed, local checks, and CI run id if
  available.
- Never merge into `main`.
- Never run `/story-done` unless explicitly assigned by the orchestrator.

The orchestrator must:

- Assign story ids, worktree paths, and branch names.
- Merge worker branches into `main`.
- Push `main`.
- Serialize shared tracking updates:
  - `production/sprint-status.yaml`
  - `production/session-state/active.md`
  - story completion notes

Existing workers already launched in the shared checkout may finish normally.
All new implementation workers should use this worktree flow.
