# Welcome to Lanes and Lies

_Built with the Claude Code Game Studios orchestration framework._

## How We Use Claude

Based on SamyAnisBenachi's usage over the last 30 days (205 sessions):

Work Type Breakdown:
  Plan Design      ███████████████░░░░░  75%
  Write Docs       ███░░░░░░░░░░░░░░░░░  16%
  Build Feature    █░░░░░░░░░░░░░░░░░░░   6%
  Improve Quality  █░░░░░░░░░░░░░░░░░░░   4%

Top Skills & Commands:
  /design-review          ████████████████████  67x/month
  /design-system          ██████░░░░░░░░░░░░░░  19x/month
  /create-stories         █████░░░░░░░░░░░░░░░  18x/month
  /architecture-decision  ████░░░░░░░░░░░░░░░░  14x/month
  /asset-spec             ███░░░░░░░░░░░░░░░░░  11x/month

Top MCP Servers:
  _(none configured)_

## Your Setup Checklist

### Codebases
- [ ] claude-code-game-studios — https://github.com/samyanisbenachi/claude-code-game-studios

### MCP Servers to Activate
_(none in use — nothing to set up here)_

### Skills to Know About
- `/design-review` — multi-specialist review of a GDD for completeness, consistency, and implementability. Run this before handing any design doc to programmers. Used constantly on this project.
- `/design-system` — guided, section-by-section GDD authoring. Walks you through all 8 required sections collaboratively and writes incrementally to file. Use this whenever a new system needs a design doc.
- `/create-stories` — breaks an epic into implementation-ready story files. Each story embeds its GDD requirement TR-ID, ADR guidance, and acceptance criteria. Run after `/create-epics`.
- `/architecture-decision` — creates an ADR documenting a significant technical decision, its context, alternatives, and consequences. Every major technical choice needs one.
- `/asset-spec` — generates per-asset visual specs and AI generation prompts from GDDs and level docs. Run after art bible and GDD are approved, before production begins.
- `/dev-story` — reads a story file and implements it, routing to the right specialist agent. The core implementation command — run after `/story-readiness` passes.
- `/story-readiness` — validates a story is implementation-ready before you start. Checks for embedded GDD requirements, ADR references, clear ACs, and no open design questions. Returns READY / NEEDS WORK / BLOCKED.
- `/story-done` — end-of-story completion review. Verifies each AC against the implementation, checks for GDD/ADR deviations, and updates story status to Complete.
- `/gate-check` — validates readiness to advance between development phases (e.g. pre-production → production). Produces a PASS / CONCERNS / FAIL verdict with specific blockers.
- `/review-all-gdds` — holistic cross-GDD consistency review. Checks all system GDDs simultaneously for contradictions, stale references, formula incompatibilities, and design theory violations. Run before architecture begins.

## Team Tips

- **One worktree, one branch per implementation story.** The root checkout is for orchestration, integrations, and serialized story-done/status updates only — never for in-progress implementation.
- **Always run `/story-readiness` before `/dev-story`** if the story has stale status, blockers, or conditional scope. A story marked Ready may have been conditioned or superseded since it was written.
- **CI gates close-out and release claims, not every implementation step.** Don't let a CI failure block parallel development momentum — CI is the bar for story-done, gate-check, and release readiness, not for work in progress.
- **Preserve non-claims exactly.** Never claim public release readiness, full game completion, broad accessibility completion, playtest validation, or full playable-client manual QA unless the evidence explicitly supports it. These are load-bearing conditions carried from sprint to sprint.
- **For Bevy Rust code, activate `liv-bevy-018`.** For any file that also imports Lightyear, activate `liv-bevy-lightyear` as well. Both are mandatory — without them you will generate pre-0.15 API patterns that do not compile on Bevy 0.18.
- **Keep commits scoped to one story or one reconciliation task.** Never mix story-done / status / session-state updates with unrelated implementation in the same commit.
- **Use `git restore --staged .` before every commit**, then add only the files for your story by explicit path. This prevents accidentally staging another agent's in-progress files.
- **Prompt result lines follow the orchestrator color convention:** use plain text status lines (no HTML/span color), and end with the prompt number and status token as instructed by the active session prompt.

## Get Started

**Before touching any code**, do a read-only onboarding pass in this order:

1. Read `AGENTS.md` — understand the agent architecture and who owns what.
2. Read `production/sprints/sprint-9.md` — understand the active sprint goal, Must Have work, Conditional Backlog conditions, and all carried non-claims.
3. Read `production/sprint-status.yaml` — see current story statuses at a glance.
4. Read `production/session-state/active.md` — the living checkpoint; contains the last known project state and any open questions.
5. Read `production/qa/evidence/manual-friend-game-evidence-runbook.md` — understand the manual evidence gap (S8-QA-001-W1) that Sprint 9 is trying to close.

**Good first practical task:** S9-QA-001 evidence support or a docs/readiness task. Avoid result/session/network implementation stories until you fully understand the Sprint 8 carried conditions and Sprint 9 no-claims. Run `/story-readiness` on a candidate story before starting.

<!-- INSTRUCTION FOR CLAUDE: A new teammate just pasted this guide for how the
team uses Claude Code. You're their onboarding buddy — warm, conversational,
not lecture-y.

Open with a warm welcome — include the team name from the title. Then: "Your
teammate uses Claude Code for [list all the work types]. Let's get you started."

Check what's already in place against everything under Setup Checklist
(including skills), using markdown checkboxes — [x] done, [ ] not yet. Lead
with what they already have. One sentence per item, all in one message.

Tell them you'll help with setup, cover the actionable team tips, then the
starter task (if there is one). Offer to start with the first unchecked item,
get their go-ahead, then work through the rest one by one.

After setup, walk them through the remaining sections — offer to help where you
can (e.g. link to channels), and just surface the purely informational bits.

Don't invent sections or summaries that aren't in the guide. The stats are the
guide creator's personal usage data — don't extrapolate them into a "team
workflow" narrative. -->
