# MIGRATION HANDOVER — for the Claude Code agent on the TARGET PC

> This file is committed to git at `tools/migration/MIGRATION-HANDOVER.md`, so it transfers via `git pull` — the agent does not need the migration zip to read it.
>
> Read this file completely before doing anything. The user is migrating their CCGS work from one Windows PC to another. The target PC **already has an existing Claude Code installation with its own global state** (other projects, other memories, other skills, other hooks). Your job is to **merge** project-specific state into the existing setup, not to clobber it.

## Inputs available to you

- The repo, already cloned at the canonical path `D:\_DEV\claude-code-game-studios` (or wherever the user actually put it — note the actual path in your first response).
- A folder of files extracted from the source PC's migration archive, at `C:\ccgs-mig\` (or wherever the user unzipped it). Inside you will find:
  - `project/` — gitignored project files (already restored to the repo by the user in step 4 of IMPORT-README).
  - `claude/` — the relevant slices of the source PC's `~/.claude/`:
    - `claude/memory/` — 12 memory `.md` files + `MEMORY.md` index from the source PC's per-project memory directory (`~/.claude/projects/D---DEV-claude-code-game-studios/memory/`)
    - `claude/skills/` — the source PC's subscribed `liv-*` skills directory
    - `claude/settings.json` — the source PC's GLOBAL Claude settings (reference only — do not overwrite the target's)
    - `claude/stop-sound.ps1`, `ask-sound.ps1`, `notify-sound.ps1` — sound-notification hook scripts referenced by `settings.json`
    - `claude/CLAUDE.md` — may or may not exist; source PC had none at export time
- The currently-active session state at `production/session-state/active.md` — this is the source of truth for the resumed task.

The user expects you to **report what you merged**, **ask before doing anything destructive**, and finish by handing them a one-liner about the current task they were on (read it from `active.md`).

## Merge plan — execute in order

### Step 1. Inspect the target's existing state (read-only)

Before touching anything, gather facts:

1. List files at `$env:USERPROFILE\.claude\` (top-level only). Note whether the target already has `settings.json`, `CLAUDE.md`, any `*.ps1` hook scripts, and a `skills/` directory.
2. List `$env:USERPROFILE\.claude\skills\` if it exists — note which `liv-*` skills are already installed.
3. List `$env:USERPROFILE\.claude\projects\D---DEV-claude-code-game-studios\memory\` if it exists — if the user cloned to the same path, this directory may already exist (created by Claude on first session in the cloned repo) and may have its own MEMORY.md.

Report findings to the user before proceeding. Ask: "Confirm you want me to proceed with the merge below?"

### Step 2. Memory merge (project-scoped — SAFE)

Per-project memory lives at `~/.claude/projects/<project-slug>/memory/`. Other projects' memory is untouched.

**Action:**

- Ensure `$env:USERPROFILE\.claude\projects\D---DEV-claude-code-game-studios\memory\` exists.
- For each `.md` file in `C:\ccgs-mig\claude\memory\` **except `MEMORY.md`**:
  - If a same-named file exists on the target with different content → show a diff, ask the user to pick (keep, replace, or merge sections).
  - Otherwise → copy it across.
- For `MEMORY.md`:
  - If target has none → copy directly.
  - If target has one → **merge** the bullet lists. Each line is `- [Title](file.md) — one-line hook`. Deduplicate by filename. Show the user the merged result before writing.

Note: the user's source-PC memory contains 12 files (project_tech_stack, feedback_bevy_skill_mandatory, project_bevy_018_violations, feedback_parallelism_first, project_scope, project_codex_split, feedback_commit_hygiene, feedback_disk_space, feedback_paw_review_flow, feedback_orchestrator_skills_flow, feedback_orchestrator_prompt_quality, project_tech_stack). The full live index is in MEMORY.md at the top of this conversation's claudeMd context if the user already loaded this repo — otherwise read `C:\ccgs-mig\claude\memory\MEMORY.md` directly.

### Step 3. Skills merge (`liv-*`)

The source PC has 8 `liv-*` skills under `~/.claude/skills/`: `liv-bevy-018`, `liv-bevy-lightyear`, `liv-grill-me`, `liv-info`, `liv-list`, `liv-subscribe`, `liv-sync`, `liv-unsubscribe`.

**Action:**

- For each skill folder in `C:\ccgs-mig\claude\skills\liv-*`:
  - If the target already has the same skill folder → run `/liv-sync` (the canonical update flow) instead of copying files. This keeps the catalog state coherent.
  - If absent → suggest the user run `/liv-subscribe <skill-name>` for each missing one. This is the proper install path; do not copy folders manually unless the catalog is unreachable.
- After: run `/liv-list` and report which skills are now subscribed.

**Critical**: `liv-bevy-018` and `liv-bevy-lightyear` are **mandatory** for this project (per `CLAUDE.md` and the user's memory `feedback_bevy_skill_mandatory`). If either is missing on the target after step 3, flag it loudly.

### Step 4. Hooks / sound notifications (OPTIONAL — ask first)

The source PC's `~/.claude/settings.json` has hooks that play sounds via PowerShell scripts:

- `Stop` → `stop-sound.ps1` (Claude finished)
- `Notification` → `notify-sound.ps1` (system notification)
- `PermissionRequest` / `PreToolUse:AskUserQuestion` → `ask-sound.ps1` (waiting for input)
- Plus shell logging to `C:\Users\Sam\claude-bash-log.txt` and prompt logging.

**Action:**

- Ask the user: "Do you want the sound notifications on the target PC? (Yes / No / Choose subset)"
- If yes:
  - Copy the three `.ps1` scripts from `C:\ccgs-mig\claude\` to `$env:USERPROFILE\.claude\` on the target.
  - Show the user the relevant `hooks` block from the source `settings.json` and propose merging it into the target's `settings.json` (use `/update-config` skill if available, otherwise manual edit).
  - **Rewrite paths**: replace `C:/Users/Sam/.claude/` with `$env:USERPROFILE/.claude/` or the target user's actual path. The source paths are hardcoded.
  - The bash log path `C:/Users/Sam/claude-bash-log.txt` must also be rewritten to the target user's home.

### Step 5. Settings — global vs project (DO NOT BLIND-MERGE PERMISSIONS)

The source PC's `~/.claude/settings.json` has 220+ entries in `permissions.allow`. Most of those are one-off command patterns accumulated during specific past tasks (e.g. specific `gh run watch <run-id>` invocations). They are **not worth porting** — they'll regenerate as the user works.

**Action:**

- Read source `settings.json` keys: `model`, `effortLevel`, `autoUpdatesChannel`, `skipDangerousModePermissionPrompt`, `skipAutoPermissionPrompt`, `permissions.defaultMode`, `permissions.additionalDirectories`.
- Propose merging **only** those high-value top-level settings into the target's `settings.json`. Show a diff before writing.
- **Skip** the entire `permissions.allow` array. The user will re-approve commands as they come up. (If the user explicitly asks to port it, do so — but warn that most entries are stale one-shots.)

The project-local `.claude/settings.local.json` was already restored by the user in step 4 of IMPORT-README — verify it's at `D:\_DEV\claude-code-game-studios\.claude\settings.local.json`. This file IS project-specific and SHOULD be kept verbatim.

### Step 6. `additionalDirectories` paths

Source `settings.json` has `additionalDirectories` with Windows source paths. If the user cloned to the same path (`D:\_DEV\claude-code-game-studios`), these mostly work. Otherwise, rewrite each entry to the new clone path. Show the list to the user and confirm.

### Step 7. Resume the task

Read `production/session-state/active.md` end-to-end. It contains the full state of the project at the source PC at export time, including:

- Active sprint and milestone (from the session-start hook preview)
- Last completed prompt and what's next
- Open blockers
- Files in-flight

Hand the user a one-line summary: "You were on `<task>` — last completed `<prompt N>`, next is `<prompt N+1>`. Resume?"

Do NOT immediately start implementing the next task. Wait for the user's go.

## Things to flag to the user proactively

1. **`.cargo/config.toml`** is not in the archive. Cargo will fall back to defaults and `cargo check` should still work. If it fails to find a linker, the user needs Developer PowerShell for VS 2022/2026 and a regenerated config.
2. **Auth**: the user must `claude login`, `codex login`, `gh auth login` themselves.
3. **Git stashes** from the source PC were intentionally not migrated (the user said they'd commit them manually before export). Confirm there are no missing stashes the user expected.
4. **Conversation transcripts** (`.jsonl` history) were NOT migrated. `/resume` history will only show sessions started on the target PC.

## Style and constraints

- Match the user's collaboration protocol: ask before writing files, show drafts, never auto-commit (per `CLAUDE.md` "Collaboration Protocol").
- This user is `benachi.samy@gmail.com`, working solo on a Bevy 0.18 + Lightyear card game (Lanes and Lies). French is fine; English is fine.
- See the project's per-project memory (after Step 2) for stronger context on their preferences (parallelism-first workflow, orchestrator-skills-flow, paw-review-flow, etc.).

## Done condition

You are done when:
- Memory directory at the target contains the merged set of memory files (with `MEMORY.md` index updated)
- All mandatory `liv-*` skills are subscribed (or the user explicitly waived them)
- Sound hooks are either installed-with-rewritten-paths or explicitly skipped by the user
- The user has been briefed on the current task from `active.md`
- A short "migration report" message has been delivered to the user listing: files merged, files skipped, decisions deferred to them, and any errors.
