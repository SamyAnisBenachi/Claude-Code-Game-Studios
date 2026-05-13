# Agent Coordination Rules

## Current GCS Orchestrator Contract

The live orchestration contract is the top override block in
`production/session-state/codex-orchestrator-state.md`:
`Current Operating Rules (2026-05-13 override)`.

When this document conflicts with that override, use the override.

Current rules for orchestrator-style agent work:

- Put a state label directly above every agent-window disposition or launch
  prompt.
- `CLEAR` means the user can close the agent window.
- `REPONDRE` means paste the response into that same window.
- `RELANCER` means rerun the same work with a corrected prompt.
- `NEW` means launch the prompt in a new agent window.
- Use these exact labels: `CLEAR -- PROMPT N`, `REPONDRE -- PROMPT N`,
  `RELANCER -- PROMPT N`, `NEW -- PROMPT N`.
- New launch prompts use `PROMPT N -- Short Task Name`.
- Launch only actually ready work; do not invent parallelism to satisfy a quota.
- Keep only one shared-status writer active at a time.
- Implementation workers use one worktree and one branch, push the worker branch
  only, and never push `main`.
- Workers do not run `/story-done`, smoke, gate-check, QA sign-off, or edit
  shared tracker files unless explicitly scoped.
- Final status line is one line only: `N: TICKET-ID: STATUS`; no delimiter line.
- STATUS is a real outcome word, not a color name.
- Use `liv-bevy-018` for Bevy reads/reviews/edits and `liv-bevy-lightyear` for
  Lightyear/networking reads/reviews/edits.

1. **Vertical Delegation**: Leadership agents delegate to department leads, who
   delegate to specialists. Never skip a tier for complex decisions.
2. **Horizontal Consultation**: Agents at the same tier may consult each other
   but must not make binding decisions outside their domain.
3. **Conflict Resolution**: When two agents disagree, escalate to the shared
   parent. If no shared parent, escalate to `creative-director` for design
   conflicts or `technical-director` for technical conflicts.
4. **Change Propagation**: When a design change affects multiple domains, the
   `producer` agent coordinates the propagation.
5. **No Unilateral Cross-Domain Changes**: An agent must never modify files
   outside its designated directories without explicit delegation.

## Model Tier Assignment

Skills and agents are assigned to model tiers based on task complexity:

| Tier | Model | When to use |
|------|-------|-------------|
| **Haiku** | `claude-haiku-4-5-20251001` | Read-only status checks, formatting, simple lookups — no creative judgment needed |
| **Sonnet** | `claude-sonnet-4-6` | Implementation, design authoring, analysis of individual systems — default for most work |
| **Opus** | `claude-opus-4-6` | Multi-document synthesis, high-stakes phase gate verdicts, cross-system holistic review |

Skills with `model: haiku`: `/help`, `/sprint-status`, `/story-readiness`, `/scope-check`,
`/project-stage-detect`, `/changelog`, `/patch-notes`, `/onboard`

Skills with `model: opus`: `/review-all-gdds`, `/architecture-review`, `/gate-check`

All other skills default to Sonnet. When creating new skills, assign Haiku if the
skill only reads and formats; assign Opus if it must synthesize 5+ documents with
high-stakes output; otherwise leave unset (Sonnet).

## Subagents vs Agent Teams

This project uses two distinct multi-agent patterns:

### Subagents (current, always active)
Spawned via `Task` within a single Claude Code session. Used by all `team-*` skills
and orchestration skills. Subagents share the session's permission context, run
sequentially or in parallel within the session, and return results to the parent.

**When to spawn in parallel**: If two subagents' inputs are independent (neither
needs the other's output to begin), spawn both Task calls simultaneously rather
than waiting. Example: `/review-all-gdds` Phase 1 (consistency) and Phase 2
(design theory) are independent — spawn both at the same time.

### Agent Teams (experimental — opt-in)
Multiple independent Claude Code *sessions* running simultaneously, coordinated
via a shared task list. Each session has its own context window and token budget.
Requires `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1` environment variable.

**Use agent teams when**:
- Work spans multiple subsystems that will not touch the same files
- Each workstream would take >30 minutes and benefits from true parallelism
- A senior agent (technical-director, producer) needs to coordinate 3+ specialist
  sessions working on different epics simultaneously

**Do not use agent teams when**:
- One session's output is required as input for another (use sequential subagents)
- The task fits in a single session's context (use subagents instead)
- Cost is a concern — each team member burns tokens independently

**Current status**: Not yet used in this project. Document usage here when first adopted.

## Parallel Task Protocol

When an orchestration skill spawns multiple independent agents:

1. Issue all independent Task calls before waiting for any result
2. Collect all results before proceeding to dependent phases
3. If any agent is BLOCKED, surface it immediately — do not silently skip
4. Always produce a partial report if some agents complete and others block
