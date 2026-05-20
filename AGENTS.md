# Codex Game Studios -- Game Studio Agent Architecture

Indie game development managed through 48 coordinated Codex subagents.
Each agent owns a specific domain, enforcing separation of concerns and quality.

## Technology Stack

- **Engine**: Bevy 0.18 (Rust)
- **Language**: Rust (stable toolchain)
- **Version Control**: Git with trunk-based development
- **Build System**: Cargo + Trunk (WASM client) / Cargo (headless server)
- **Asset Pipeline**: bevy_asset_loader + TextureAtlas (sprite sheets)
- **Networking**: Lightyear (bevy_lightyear) — client/server over WebSocket/WebTransport
- **UI**: bevy_ui (0.18 Required Components API) + bevy_tweening
- **Deployment**: WASM client → Vercel / Rust server → Railway (Docker)

> **Note**: Use `liv-bevy-018` skill for ALL Bevy code. Use `liv-bevy-lightyear`
> skill for ALL networking code. These skills enforce correct 0.18 API patterns.

## Project Structure

@.Codex/docs/directory-structure.md

## Engine Version Reference

@docs/engine-reference/bevy/VERSION.md

## Technical Preferences

@.Codex/docs/technical-preferences.md

## Coordination Rules

@.Codex/docs/coordination-rules.md

## Current Orchestrator Contract

The current GCS orchestrator rules live in
`production/session-state/codex-orchestrator-state.md` under
`Current Operating Rules (2026-05-13 override)`. That section supersedes older
prompt-formatting, delimiter, close-out, and parallelism notes elsewhere in the
repo.

Key current rules:

- Structured `gcs.dispatch` actions are the preferred source of truth for
  orchestration side effects. Use `SPAWN`, `CLEAR`, `REPONDRE`, `RELANCER`,
  and `NEW` for worker lifecycle changes when the tool is available. Use
  `MAINLAND_LIST` before main-land work, then `MAINLAND_ENQUEUE` with
  `project_id`, `source_branch`, and `intent_id` (`mainland-<slug>-<prompt_n>`)
  for serialized fast-forward-only main landings. Use `MAINLAND_CANCEL` only for
  pending queue entries.
- For implementation, integration, verify, audit, and report workers, pass
  `workspace_mode: "worktree"` in `gcs.dispatch` `SPAWN`/`RELANCER` unless a
  specific task explicitly requires the shared root checkout. The root checkout
  must stay on `main` as the orchestrator source-of-truth workspace.
- If structured `gcs.dispatch` is unavailable, fallback emoji labels may be used
  for worker lifecycle actions only. Do not claim a main-land queue action was
  submitted unless `MAINLAND_ENQUEUE` returned a queue id, or a direct Git action
  was explicitly performed and reported.

- Put a plain emoji disposition label directly above every agent-window action:
  - `🟢 CLEAR -- PROMPT N` for a window the user can close.
  - `🟡 REPONDRE -- PROMPT N` for text to paste back into the same window.
  - `🔴 RELANCER -- PROMPT N` for a corrected rerun/repair in the same workstream.
  - `🟣 NEW -- PROMPT N` for a new prompt to launch in a new agent window.
- Do not wrap launch prompts in fences, triangle headers/closers, hash
  delimiters, or copied template wrappers. Put the body directly under the
  disposition label; for NEW, first body line is `PROMPT N -- Task Title`.
- Launch only work that is actually ready and file-disjoint; do not invent
  parallelism to fill a quota.
- Keep one shared-status writer active at a time (`/story-done`,
  `production/sprint-status.yaml`, `production/session-state/active.md`, story
  completion notes).
- Future-sprint work may be prepared only when it is Ready, disjoint, and does
  not activate that sprint.
- When many implementation agents run in parallel, do not make every worker run
  Cargo checks/tests. Implementation workers should commit and report quickly
  after focused local validation; Cargo verification should move into separate
  `VERIFY` prompts or serialized checkpoint/smoke lanes. If a verify lane fails,
  launch a follow-up repair worker from the failing output.
- Worker Git permissions must not stall the flow. If push, protected-branch,
  GitHub export, rebase, or similar policy blocks a worker, the worker should
  keep its local commit/branch, push any non-protected worker/integration branch
  if allowed, and relay the exact branch, commit, command, and blocker to the
  orchestrator. Do not ask the human for special push/rebase permission from the
  worker window; the orchestrator handles main-land and permission-sensitive Git
  actions.
- The active dispatcher is `gcs-app` / `gcs.dispatch`. Legacy dispatcher docs are
  historical unless the current orchestrator state explicitly re-enables them.
- Final prompt/status line is one line only: `N: TICKET-ID: STATUS`. No delimiter
  line, no HTML/span/CSS/ANSI markup, and STATUS must be a real outcome word.

## NEW PROMPT TEMPLATE

Use this template for new worker launches. Keep older prompt templates intact;
this is the current default for new prompts.

```text
PROMPT N -- Short Task Title

Context:
- Current source-of-truth: origin/main@<sha if known>.
- Related prompts/reports: <ids and paths>.
- This worker is not alone in the repo; do not revert edits from other workers.

Task:
- <concrete task outcome>.

Owned scope:
- <files/modules this worker may edit>.
- Forbidden unless explicitly instructed: production/sprint-status.yaml,
  production/session-state/**, production/sprints/**, production/qa/**,
  production/stage.txt, unrelated Cargo/CI files, unrelated source modules.

Implementation rules:
- Use a dedicated worktree and branch. Orchestrator should launch this with
  `workspace_mode: "worktree"`; if you start in the shared/root checkout, create
  a dedicated worktree before editing.
- Use repo patterns and required skills (`liv-bevy-018` for Bevy code,
  `liv-bevy-lightyear` for networking/protocol work).
- Do not run broad Cargo suites by default. Run only focused local validation
  that is cheap and relevant; defer broad verification to a separate VERIFY
  prompt/checkpoint lane.
- If push/rebase/protected-branch/GitHub export blocks, keep the local commit or
  branch and relay exact branch, commit, command, and blocker. Do not ask the
  human for special Git permission from the worker.

Validation:
- Minimum: path allowlist review + `git diff --check`.
- Focused tests/checks: <specific tests if required>.
- Cargo-heavy verification: deferred to VERIFY lane unless this prompt is that
  lane.

Report:
- Write `reports/PROMPT-N-<slug>.md`.
- Final line exactly: `N: TICKET-ID: STATUS`.
```

## Parallel Worker Isolation

Implementation workers must use one Git worktree and one branch per story. The
root checkout is reserved for orchestration, integration merges, and serialized
story-done updates.

- Worker branch format: `work/<story-id>-<short-slug>`
- Worker path format: `D:\_DEV\claude-code-game-studios-worktrees\<story-id>`
- Workers push their branch, not `main`
- The orchestrator merges worker branches into `main`
- `/story-done`, `production/sprint-status.yaml`, and
  `production/session-state/active.md` updates are serialized by the
  orchestrator

## Collaboration Protocol

**User-driven collaboration, not autonomous execution.**
Every task follows: **Question -> Options -> Decision -> Draft -> Approval**

- Agents MUST ask "May I write this to [filepath]?" before using Write/Edit tools
- Agents MUST show drafts or summaries before requesting approval
- Multi-file changes require explicit approval for the full changeset
- No commits without user instruction
- Do not append a manual attention footer to user-facing final responses. The
  Codex Stop hook is the only source for the `WAITING INPUT` footer.

See `docs/COLLABORATIVE-DESIGN-PRINCIPLE.md` for full protocol and examples.

> **First session?** If the project has no engine configured and no game concept,
> run `/start` to begin the guided onboarding flow.

## Coding Standards

@.Codex/docs/coding-standards.md

## Context Management

@.Codex/docs/context-management.md
