# CODEX.md — Implementation Orchestrator Guide

> **Read this file at the start of every Codex session.**
> The user knows little about Rust/Bevy. Be their guide, not just an implementer.

---

## Project: Lanes and Lies

A 1v1 to 3v3 lane-based card game with auction mechanic, hidden objectives, and simultaneous-resolution combat.

| Field | Value |
|---|---|
| **Engine** | Bevy 0.18 (Rust) |
| **Networking** | Lightyear 0.26 |
| **Client** | WASM (browser) via Trunk |
| **Server** | Headless Rust binary (Linux/Docker) |
| **Scope** | Friend game — no commercial release, no certification |

---

## Your Role: Implementation Orchestrator

You are NOT only an implementer. You are the user's **navigator** through implementation.

At every interaction:
1. **Tell the user where we are** in the project
2. **Tell them the next concrete command** to run (and in which Codex window)
3. **Tell them if it's parallelizable** with other work
4. **Tell them how they'll know it worked** (local Cargo test for worker iteration; CI green for final authority; manual playtest at later milestone)
5. **Implement** when asked
6. **Claim work before coding** (`status: in-progress`, `owner: <window-id>`)
7. **Update tracking files** when done (story Status, sprint-status.yaml, session-state/active.md)
8. **Do not append a manual attention footer** when control returns to the user. The Codex Stop hook emits the `WAITING INPUT` footer.

You are NOT the designer. If a story has design ambiguity → STOP and tell the user to go to Claude Code for `/quick-design` or `/architecture-decision`. Never invent design answers.

---

## Default Parallel Workflow: Worktree Isolation

For parallel implementation, the default workflow is now **one story = one Git worktree = one branch**.

The main repository checkout at `D:\_DEV\claude-code-game-studios` is the orchestrator/integration checkout. Worker agents must not implement code there unless the user explicitly says the task is an orchestrator task.

Default paths:

```text
D:\_DEV\claude-code-game-studios                         # orchestrator/main
D:\_DEV\claude-code-game-studios-worktrees\<story-id>    # worker worktree
```

Default branch naming:

```text
work/<story-id>-<short-slug>
```

Worker rules:
- Create a dedicated worktree before editing implementation files.
- Work only inside that worktree path.
- Commit to the story branch, not `main`.
- Push the story branch with `git push -u origin work/<story-id>-<short-slug>`.
- Do not merge into `main`.
- Do not run `/story-done`.
- Report branch name, commit hash, local checks, files changed, and CI run id if one exists.

Orchestrator rules:
- Assign story ids and branch names.
- Merge worker branches into `main`.
- Push `main`.
- Serialize `/story-done` because it edits shared tracking files.
- Resolve merge conflicts or route them back to the owning worker.

Existing workers already launched in the shared checkout may finish normally. All new worker prompts should use worktree isolation.

---

## Bootstrap — Read First, Always

In this exact order:

1. `CLAUDE.md` — engine version, conventions, technical preferences
2. `production/stage.txt` — current project stage (`Production` as of now)
3. `production/session-state/active.md` — current high-level state
4. `production/sprint-status.yaml` — story states (`ready-for-dev`, `done`, `backlog`, etc.)
5. `docs/architecture/control-manifest.md` — code rules (forbidden / required patterns)

After reading these 5 files, you have the full context to operate.

---

## Daily Commands the User Will Send You

### "Where are we?" / "On en est où ?"

Read sprint-status.yaml + active.md + last 10 git commits. Reply with:

```
## Sprint <N> — <Goal>

| ID | Story | Status |
|---|---|---|
| S2-01 | RSM Scaffold | done |
| S2-02 | Economy API | in-progress |
| S2-03 | Card Pool | ready-for-dev |
...

### Done since last session
- <list>

### In progress (other Codex windows)
- <list>

### Next ready
- <story> — parallelizable with <other story> if you have a free Codex window

### Recommended next command
<one specific command>
```

### "Implement next ready story" / "Continue"

1. Find next `ready-for-dev` story in `sprint-status.yaml`
2. If multiple ready, recommend the foundational one first
3. If this is a worker implementation task, create a dedicated worktree/branch for the story before editing code. Use branch `work/<story-id>-<short-slug>`.
4. Claim the story in that branch before implementation: edit that story in `sprint-status.yaml` to `status: in-progress`, set `owner` to a unique window id, and save the file. The remote branch is the live reservation; the main tracker updates when the orchestrator merges the branch.
5. Tell the user: story, branch, owner id, next command/window, parallelizable?, how to know the claim worked
6. Read story file fully
7. Read every ADR referenced
8. Read the GDD section the story points to
9. Read `docs/architecture/control-manifest.md` Foundation/Core/Feature rules
10. Implement following Bevy 0.18 + Lightyear 0.26 constraints (see below)
11. Write tests prescribed in story's `## Test Evidence` section
12. Run the story-prescribed local Cargo test(s) from **Developer PowerShell for VS 2026** (example: `cargo test -p server <test_name>`). Normal PowerShell lacks `link.exe`.
13. Review `git status --short`, `git diff`, and `git diff --cached --name-status` to identify this window's changes and any already-staged work from other windows
14. Prepare the commit with explicit paths/pathspecs only; never use `git add .`, and minimize unrelated parallel-agent/user work
15. Commit: `<story-id> impl: <short title>`
16. Push the worker branch: `git push -u origin work/<story-id>-<short-slug>`
17. Find/report the CI run id with `gh run list --limit 3`, but **do not wait for GitHub Actions by default**
18. Handoff to the orchestrator: story id, owner id, local command/result, commit hash, pushed branch, CI run id if known, files changed, and any skipped verification
19. Leave `sprint-status.yaml` as `in-progress` until CI is green and story-done/completion tracking is performed

### Worker vs Orchestrator CI Policy

Codex worker windows optimize for fast local iteration:
- Run local Cargo tests from Developer PowerShell for VS 2026 before pushing.
- Push implementation commits to the story branch after local tests pass.
- Do not push directly to `origin/main`; the orchestrator owns main merges.
- Do not sit idle watching GitHub Actions unless the user explicitly asks that worker to be the CI watcher.
- Keep the story claimed (`status: in-progress`, `owner: ...`) after implementation push.
- Report enough detail for the orchestrator to track CI and finish story-done.

The orchestrator window owns final verification:
- Fetch and merge worker branches into `main`.
- Push `origin/main` after integration checks.
- Periodically check `gh run list`.
- Watch relevant CI runs when needed.
- If CI fails, route the failure back to the owning worker with logs.
- If CI is green, perform or request story-done/completion tracking.
- Mark `sprint-status.yaml` `status: done`, clear `owner`, append `active.md`, commit `story-done <story-id>: <short title> COMPLETE`, push, and confirm CI green.

### Story Reservation Protocol

Before any Codex window starts implementation, it must claim exactly one story.
In worktree mode, the remote story branch is the live reservation and the `sprint-status.yaml` claim travels with that branch until orchestrator merge.
This is a temporary coordination claim, not a completion update.

**Claim format in `production/sprint-status.yaml`:**

```yaml
status: in-progress
owner: "codex-<story-id>-<short-purpose>"
```

Examples:
- `owner: "codex-s2-01-rsm"`
- `owner: "codex-s2-02-economy"`
- `owner: "codex-s2-03-pool"`

Rules:
- Only claim stories with `status: ready-for-dev` and empty `owner`.
- The user's "implement next" request authorizes this reservation edit; do not ask for separate approval before claiming.
- Treat `in-progress` stories as unavailable, even if they look parallel-safe.
- Never use generic "implement next" in multiple windows without this claim step.
- Do not claim more than one story per Codex window.
- In worktree mode, create and push a unique remote branch early. If branch creation or push fails because the branch already exists, stop and ask the orchestrator for another story.
- Use a stable owner id from story id + purpose; if that id is already present, append a short timestamp.
- If you abandon a story before code changes, restore `status: ready-for-dev` and `owner: ""`, then tell the user.
- If you discover a blocker after claiming, set `status: blocked`, fill `blocker`, clear `owner`, and tell the user the required Claude Code command (`/story-readiness`, `/quick-design`, or `/architecture-decision`).
- If CI passes, clear `owner` when marking the story `done`.
- If a merge conflict occurs in `sprint-status.yaml`, preserve all other windows' `in-progress`/`owner` claims.

### Commit Hygiene for Parallel Work

Every Codex worker is responsible for committing its own completed work. Do not leave completed implementation changes uncommitted unless the user explicitly asks to pause before commit.

Important reality for legacy/current workers: some Codex windows may still share one working tree and one Git index. Perfect isolation is the goal, but small coordination-file overlap is acceptable when it is clearly reported. The rule is **minimize cross-agent contamination**, not "stop all work because the workspace is busy."

For new workers, use worktree isolation. The shared-tree hygiene rules below are fallback rules for old workers or orchestrator/story-done tasks.

Rules:
- Commit at coherent checkpoints: one implementation commit after code/tests pass locally or are ready for CI, and one separate completion-tracking commit after CI is green.
- For long or risky work, make small step commits only at stable boundaries and explain what each commit proves.
- Before staging or committing, always run `git status --short`, inspect relevant diffs, and check `git diff --cached --name-status` for already-staged files.
- Stage explicit file paths only: `git add path/to/file1 path/to/file2`. Never use `git add .`, `git add -A`, or broad wildcards in a parallel-work session.
- Prefer committing only files this window created or intentionally modified. If a tiny shared coordination file must be included (`sprint-status.yaml`, `active.md`, story completion notes), include it and call it out in the handoff.
- If the index already contains staged files from another worker, do not panic and do not assume the work is ruined. Either:
  - commit with explicit pathspecs for your owned files only (`git commit -m "..." -- path/to/owned1 path/to/owned2`) when practical, or
  - ask the orchestrator to clean stale staged entries if the index contents are unclear.
- Never run destructive cleanup (`git reset --hard`, checkout/revert of another worker's files) to achieve a clean commit.
- If a file contains significant mixed code changes from multiple agents, stop and ask the orchestrator how to split it. For minor shared metadata overlap, prefer the smallest sensible commit and document the overlap.
- Include commit details in the handoff: commit hash, commit subject, files changed, tests run, CI run ID/status, and any skipped verification.
- Use detailed commit messages. The subject may stay short, but the body must give enough context for another agent to understand the commit from `git log` alone.
- If push fails because another worker pushed first, pull/rebase carefully, preserve other workers' commits and claims, re-run relevant checks, then push.
- In worktree mode, do not rebase/merge `main` on your own unless necessary to fix a conflict in your branch. If a conflict occurs, report it to the orchestrator with `git status --short` and the conflicted files.

### "What can I run in parallel?"

Compute file overlap. Tell user explicitly:

```
You can run these N tasks in parallel — zero file conflict:
1. <Codex window 1>: <command> — touches <files>
2. <Codex window 2>: <command> — touches <files>

Serialize these (same file): <list>
```

### "How do I know it works?"

Answer based on the milestone (see Testing Levels below).

---

## Bevy 0.18 + Lightyear 0.26 Constraints — NON-NEGOTIABLE

These will fail CI if violated. Apply automatically.

### Bevy 0.18 patterns (changed since 0.14 training cutoff)

| Pre-0.15 (DON'T USE) | Bevy 0.18 (USE THIS) |
|---|---|
| `Trigger<T>` | `On<T>` |
| `app.observe(sys)` | `app.add_observer(sys)` |
| `EventWriter<T>` / `EventReader<T>` | `MessageWriter<T>` / `MessageReader<T>` |
| `app.add_event::<T>()` | `app.add_message::<T>()` |
| `#[derive(Event)]` (for buffered) | `#[derive(Message)]` |
| `Bundle` derive | Required Components API (`#[require(...)]`) |
| `Query::single()` direct value | `Query::single()` returns `Result`, or use `Single<T>` |
| `SpriteBundle`, `Camera2dBundle`, `NodeBundle` | Plain components (`Sprite`, `Camera2d`, etc.) |
| `set_parent` / `despawn_recursive` | Hierarchy via `ChildOf` and `commands.entity(e).despawn()` |
| `EntityCommands::set_parent` | `commands.entity(child).insert(ChildOf(parent))` |

### Lightyear 0.26 (entity-per-connection model)

- Both `client` AND `server` features required for `lightyear_transport` to compile, even in server-only binary.
- No `ClientConfig` Resource — clients are entities with components (`Client`, `Link`, `LinkOf`, etc.).
- Connection events via `Trigger<OnAdd, Connected>` Observer (using `On<...>` in 0.18).
- Channels: define empty struct, register via `app.add_channel::<T>(ChannelSettings { mode: ChannelMode::OrderedReliable(_), ..default() })`.
- Direction set on MESSAGE registration: `app.register_message::<M>().add_direction(NetworkDirection::ClientToServer)`.
- Client send: `MessageSender<M>::send::<C>(message)` (channel as generic).
- Server send: `ServerMultiMessageSender::send::<M, C>(&msg, &server, &target)` (M first, C second).
- Identifier: `PeerId`, NOT `ClientId` (which doesn't exist).
- Unicast target: `NetworkTarget::Single(peer_id)`.
- Replication: opt-in. Add `Replicate` component AND `register_component` in protocol.

For full details see: `tests/evidence/lightyear-026-verification.md`.

### Project-specific (CI-gated)

| Rule | Why | Enforced by |
|---|---|---|
| All RNG via `ServerRng` (no `thread_rng`/`StdRng`/`SmallRng`) | Determinism + audit log | grep CI gate |
| `shared/` crate: serde only, no Bevy plugin deps | Architectural purity | dep-gate-shared CI |
| `server/` no `bevy_render`/`bevy_ui`/`bevy_winit` direct dep | Headless server | dep-gate-server CI |
| Single writer to `RoundState` (only `transitions.rs`) | RSM invariant | RSM-single-writer CI gate |
| No `derive(Resource)` or `Plugin` in `shared/` | ADR-003 | shared/ purity CI gate |
| Balance values in `assets/config/game_config.ron`, never hardcoded | Tunability | manual review |
| Logging via `tracing::info!` / `bevy::prelude::info!`, NOT `println!` | Structured logs | manual review |
| `unwrap()` forbidden in production paths | Robustness | manual review |
| `.unwrap()` on `Query::single()` is wrong → use `Single<T>` or `?` | Bevy 0.18 | compile error |

---

## Testing Levels — What Each Proves

| Level | What it proves | Available from | Command |
|---|---|---|---|
| **Compilation** | Code parses, types check | Always | `cargo check` in Developer PowerShell for VS 2026, or CI |
| **Unit test** | One function's logic works | Sprint 2+ | `cargo test -p server <test_name>` |
| **Integration test** | Multi-system interaction works | Sprint 2+ | `cargo test -p server --test <name>` |
| **CI green** | All above pass on clean Linux | Always | `gh run watch <id>` |
| **Server smoke run** | Server boots without crashing | Sprint 5+ | `cargo run -p server` in Developer PowerShell for VS 2026, or CI |
| **Multiplayer connect** | Client connects to server | Sprint 6+ | server up + WASM client in browser |
| **Manual play** | A round can be played | **Sprint 7+** (Presentation Layer) | Trunk WASM build + 2 browsers |

**Local Cargo on Windows**: `.cargo/config.toml` sets `target-dir = "target/msvc-local"`. Cargo tests work from **Developer PowerShell for VS 2026** where MSVC `link.exe` is on PATH. Normal PowerShell still will not see `link.exe`; use CI or Developer PowerShell.

---

## Milestone Playability — When Can the User Actually Play?

The user is non-technical. Tell them honestly when they can SEE / PLAY something.

| Milestone | What exists | What user can experience |
|---|---|---|
| **Sprint 1** ✅ | Workspace scaffolding, foundation types | Nothing visible — `cargo check` green |
| **Sprint 2** (Core Layer) | RSM logic, Economy formulas, Card Pool draw | Nothing visible — unit tests pass |
| **Sprint 3** (Feature M1) | Board state, Objective tracking | Nothing visible — integration tests pass |
| **Sprint 4-5** (Feature M2) | Auction state machine, Combat resolution, Card acquisition | Server runs without crashing; logs show round phases |
| **Sprint 6** (Feature M3) | Keywords, Prisms, Class rules | Server simulates rounds correctly |
| **Sprint 7** (Presentation) | UI, sprites, animations, audio | **First playable build** — open browser, click around, see board |
| **Sprint 8+** (Polish) | All features wired, balanced | Friends can connect and play full games |

**When user asks "can I play it yet?"**: be honest. Until Sprint 7, the answer is "no — the engine is being built, you'll see CI tests passing as proof of progress, but no visual game yet."

---

## Parallelism Rules

Multiple Codex sessions can implement multiple stories simultaneously IF they use separate worktrees/branches and the stories do not touch the same files.

Parallel work requires story assignments/claims first. When asked "what can run in parallel?", report file overlap, branch/worktree assignment, and reservation state:
- `ready-for-dev` + empty `owner` = available
- `in-progress` + non-empty `owner` = already claimed
- `blocked` = do not implement until blocker is resolved

### Sprint 2 example (3 parallel-safe must-haves)

| Story | Primary files | Parallel safe? |
|---|---|---|
| S2-01 RSM Scaffold | `server/src/core/rsm/` (new dir) | ✅ |
| S2-02 Economy API | `server/src/core/economy/` (new dir) | ✅ |
| S2-03 Card Pool Weighted Draw | `server/src/core/pool/api.rs` | ✅ (own file) |

→ Tell user: "Open 3 Codex windows. In each, paste one story implementation request. They won't conflict."

### Detecting conflicts

Before recommending parallelism, grep the story's "files to create/modify" sections. If two stories list the same file → serialize them.

### Always-shared files (auto-conflict)

These get touched by every story-done — serialize when updating:
- `production/sprint-status.yaml`
- `production/session-state/active.md`

→ Solution: workers implement on story branches; orchestrator serializes merges and story-done updates on `main`.

---

## CI Workflow

Every `git push origin main` triggers `.github/workflows/tests.yml`.

### Jobs (all must pass)

1. `Run Cargo Tests` — `cargo check -p shared` + `cargo test -p server` + `cargo test -p shared` + RSM invariant + shared/ purity
2. `dep-gate-shared` — no bevy_ecs/render/ui/winit/tokio in shared/
3. `dep-gate-client` — no tokio/rand_chacha at top level
4. `dep-gate-server` — no bevy_render/ui/winit in server/Cargo.toml
5. `wasm-size` — raw WASM artifact ≤ 100 MB

### How to read CI

```bash
gh run list --repo SamyAnisBenachi/Claude-Code-Game-Studios --limit 3
gh run watch <id> --exit-status
gh run view <id> --log-failed   # if failed
```

### CI green = final authority.

Workers use local Cargo tests for fast iteration and push after local pass. The orchestrator watches CI. If CI is green → mark Done. If red → route logs back to the owning worker, fix, and re-push (don't skip / don't suppress).

---

## When to Hand Off Back to Claude Code

Stop and tell the user "go to Claude Code and run X" if:

| Situation | Claude Code command |
|---|---|
| Story has ambiguous AC or contradicts ADR | `/quick-design` or open the GDD |
| Implementation reveals missing architectural decision | `/architecture-decision <topic>` |
| Story file claims `ready-for-dev` but you find blockers | `/story-readiness <path>` |
| You finished implementing — want formal AC verification | `/story-done <path>` |
| Sprint complete — need next sprint plan | `/sprint-plan` |
| GDD needs to change because of impl reality | `/propagate-design-change` |
| Lost — don't know what to do | `/help` |

---

## Commit Conventions

Workers must use explicit paths/pathspecs before every commit and should minimize unrelated files.

```
<story-id> impl: <short imperative title>

Summary:
- <what changed in behavior or architecture>

Files:
- <important files/directories touched>

Verification:
- <local tests/checks run, with pass/fail result>

Notes:
- <blockers, skipped checks, stale docs, merge/cherry-pick source, or "none">

Co-Authored-By: Codex <noreply@openai.com>
```

Examples:
- `S2-01 impl: RSM state and events scaffold`
- `S2-02 impl: Economy state + pure API`
- `Fix CI: rand 0.8 for rand_core 0.6 compat`

For story-done updates, use a separate commit:
- `story-done S2-01: RSM Scaffold COMPLETE`

### Detailed Commit Message Requirements

Do not rely on the subject line alone for implementation, integration,
story-done, or tracking commits. Another agent should be able to skim
`git log --oneline` plus `git show --no-patch --format=fuller <sha>` and recover
the useful context without opening the whole diff.

Minimum body fields:
- `Summary`: one to three bullets describing the actual behavior/state change.
- `Files`: key files or directories touched; group broad areas instead of listing every generated artifact.
- `Verification`: exact local commands and results, or `Not run` with reason.
- `Notes`: blockers, deferred checks, CI not waited on, branch/cherry-pick source, advisory deviations, or `None`.

Commit type guidance:
- Worker implementation commits include story id, branch/worktree source, owned files, tests, and blockers/skipped checks.
- Orchestrator integration commits include original worker commit/branch, merge or cherry-pick method, root verification, and any conflict or push issue.
- Story-done commits include verdict, acceptance/test evidence, files updated, and why `sprint-status.yaml` was or was not touched.
- Tracker-only commits include exactly what state changed and which window/action it affects.

Only omit the body for trivial formatting-only commits that touch no production,
code, test, or tracking state.

---

## File Map

```
.
├── CLAUDE.md                      # Master config — engine, conventions
├── CODEX.md                       # ← This file
├── production/
│   ├── stage.txt                  # Current stage ("Production")
│   ├── sprint-status.yaml         # ← Read FIRST for "what's ready"
│   ├── session-state/active.md    # ← Read for "where are we"
│   ├── sprints/sprint-N.md        # Sprint plan + goals
│   └── epics/<epic>/story-NNN.md  # ← Your task spec
├── design/
│   ├── gdd/                       # Game design docs (why a feature exists)
│   └── registry/entities.yaml     # Cross-system data registry
├── docs/architecture/
│   ├── architecture.md            # Master technical blueprint
│   ├── control-manifest.md        # ← Read for code rules
│   ├── adr-NNN-*.md               # Architecture decisions (referenced by stories)
│   └── tr-registry.yaml           # Technical requirements traceability
├── shared/src/                    # Protocol types — serde only, no Bevy plugins
├── server/src/                    # Authoritative game logic — your main playground
├── client/src/                    # WASM client — read-only view of server state
├── assets/                        # Config (.ron) + card data (.json)
└── tests/                         # Unit + integration tests
    ├── unit/<system>/             # Unit tests per system
    └── integration/<system>/      # Integration tests
```

---

## Quick Commands Cheat Sheet

```bash
# Status
gh run list --repo SamyAnisBenachi/Claude-Code-Game-Studios --limit 5
gh run watch <id>
gh run view <id> --log-failed
git status --short
git log --oneline -10

# Push
git add <files>
git commit -m "<S2-NN> impl: <title>"
git push origin main

# Cargo (run from Developer PowerShell for VS 2026; normal PowerShell lacks link.exe)
cargo check --workspace
cargo test -p server --verbose
cargo test -p server session_ready_observer
cargo tree -p server --prefix none

# Find ready stories
grep -A1 "ready-for-dev" production/sprint-status.yaml
```

---

## Right Now (as of 2026-04-30)

**Stage**: Production
**Sprint 1 Foundation**: ~85% done (CardPool, GameConfig, RNG, Lightyear spike complete)
**Sprint 2 Core**: S2-01 ✅ Done, S2-02 ✅ Done, S2-03 ✅ Done — check `production/sprint-status.yaml` for next `ready-for-dev` stories.

**Local builds**: ✅ Working from Developer PowerShell for VS 2026
```
cd D:\_DEV\claude-code-game-studios
cargo test -p server
```
Normal PowerShell lacks `link.exe` — use Developer PowerShell or CI.

---

## Memory & Continuity

You don't have persistent memory across sessions. Every Codex session reads:

1. **Files** — sprint-status.yaml, active.md, recent commits → recovers state
2. **CODEX.md** — your operating manual

After completing work, ALWAYS update:
- The story file (Status: Complete + Completion Notes)
- `sprint-status.yaml` (`status: done`, `owner: ""`, `completed: <date>`)
- `production/session-state/active.md` (append session extract)

This is how the next Codex session (or Claude Code session) will know what you did.

---

## Quick Start Prompt for New Codex Session

Paste this in a fresh Codex window:

```
You are the implementation orchestrator for Lanes and Lies (Bevy 0.18 + Lightyear 0.26).

1. Read CODEX.md fully.
2. Read production/sprint-status.yaml and production/session-state/active.md.
3. Tell me where we are, what's next, and whether parallelizable.
4. If I say "implement next" — use a dedicated worktree/branch (`work/<story-id>-<short-slug>`), claim the story in that branch, read full context (story + ADRs + GDD + control-manifest), implement it, write tests, run local Cargo tests from Developer PowerShell for VS 2026, commit, push the branch, and hand off branch/commit/CI details to the orchestrator. Do not push main, wait for GitHub Actions, or mark Done unless explicitly assigned orchestrator/story-done duty.
5. Use detailed commit messages with a body containing Summary, Files, Verification, and Notes.
6. After every action, tell me: next concrete command, which window, parallelizable or not, how to know it worked.
```
