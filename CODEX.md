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
4. **Tell them how they'll know it worked** (CI green, test passing, or manual playtest at later milestone)
5. **Implement** when asked
6. **Update tracking files** when done (story Status, sprint-status.yaml, session-state/active.md)

You are NOT the designer. If a story has design ambiguity → STOP and tell the user to go to Claude Code for `/quick-design` or `/architecture-decision`. Never invent design answers.

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
3. Read story file fully
4. Read every ADR referenced
5. Read the GDD section the story points to
6. Read `docs/architecture/control-manifest.md` Foundation/Core/Feature rules
7. Implement following Bevy 0.18 + Lightyear 0.26 constraints (see below)
8. Write tests prescribed in story's `## Test Evidence` section
9. Commit: `<story-id> impl: <short title>`
10. Push: `git push origin main`
11. Watch CI: `gh run watch <id>`
12. If CI fails → read failure log, fix, repeat
13. If CI green → update story Status to Complete + add Completion Notes
14. Update `sprint-status.yaml`: `status: done`, `completed: <YYYY-MM-DD>`
15. Append session extract to `production/session-state/active.md`
16. Tell user what unblocked + next recommended command

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
| **Compilation** | Code parses, types check | Always | `cargo check` (CI runs) |
| **Unit test** | One function's logic works | Sprint 2+ | `cargo test -p server <test_name>` |
| **Integration test** | Multi-system interaction works | Sprint 2+ | `cargo test -p server --test <name>` |
| **CI green** | All above pass on clean Linux | Always | `gh run watch <id>` |
| **Server smoke run** | Server boots without crashing | Sprint 5+ | `cargo run -p server` (CI or WSL2) |
| **Multiplayer connect** | Client connects to server | Sprint 6+ | server up + WASM client in browser |
| **Manual play** | A round can be played | **Sprint 7+** (Presentation Layer) | Trunk WASM build + 2 browsers |

**⚠️ Local builds blocked by Windows Smart App Control. Trust CI as source of truth.**

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

Multiple Codex sessions can implement multiple stories simultaneously IF they don't touch the same files.

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

→ Solution: each Codex session commits separately; git handles merge.

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

### CI green = your implementation is correct per the story contract.

If green → mark Done. If red → fix and re-push (don't skip / don't suppress).

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

```
<story-id> impl: <short imperative title>

<optional body — only for non-trivial commits>

Co-Authored-By: Codex <noreply@openai.com>
```

Examples:
- `S2-01 impl: RSM state and events scaffold`
- `S2-02 impl: Economy state + pure API`
- `Fix CI: rand 0.8 for rand_core 0.6 compat`

For story-done updates, use a separate commit:
- `story-done S2-01: RSM Scaffold COMPLETE`

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

# Cargo (CI runs these — but you can verify in WSL2 if needed)
cargo check --workspace
cargo test -p server --verbose
cargo tree -p server --prefix none

# Find ready stories
grep -A1 "ready-for-dev" production/sprint-status.yaml
```

---

## Right Now (as of 2026-04-30)

**Stage**: Production
**Sprint 1 Foundation**: 85% done (CardPool, GameConfig, RNG implemented)
**Sprint 2 Core**: 3 must-have stories `ready-for-dev`:

1. **S2-01** RSM State + Events Scaffold (`production/epics/round-state-machine/story-001-state-and-events-scaffold.md`)
2. **S2-02** Economy State + Pure API Scaffold (`production/epics/economy-system/story-001-state-and-pure-api-scaffold.md`)
3. **S2-03** Card Pool Weighted Draw (`production/epics/card-data-pool/story-002-weighted-draw-functions.md`)

These 3 are parallel-safe (different directories). User can launch 3 Codex windows and assign one each.

**First-time recommendation**: Start with S2-01 alone first to validate the workflow, then parallelize S2-02 and S2-03 once S2-01 is in CI.

---

## Memory & Continuity

You don't have persistent memory across sessions. Every Codex session reads:

1. **Files** — sprint-status.yaml, active.md, recent commits → recovers state
2. **CODEX.md** — your operating manual

After completing work, ALWAYS update:
- The story file (Status: Complete + Completion Notes)
- `sprint-status.yaml` (`status: done`, `completed: <date>`)
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
4. If I say "implement next" — pick the next ready-for-dev story, read its full context (story + ADRs + GDD + control-manifest), implement it, write tests, commit, push, watch CI, mark Done.
5. After every action, tell me: next concrete command, which window, parallelizable or not, how to know it worked.
```
