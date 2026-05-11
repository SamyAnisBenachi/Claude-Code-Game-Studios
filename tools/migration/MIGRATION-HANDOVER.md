# MIGRATION HANDOVER — for the Claude Code agent on the TARGET PC

> This file is committed to git at `tools/migration/MIGRATION-HANDOVER.md`, so it transfers via `git pull` — the agent does not need the migration zip to read it.
>
> Read this file completely before doing anything. The user is migrating their CCGS work from one Windows PC to another. The target PC **already has an existing Claude Code installation with its own global state** (other projects, other memories, other skills, other hooks). Your job is to **merge** project-specific state into the existing setup, not to clobber it.

## ⚠️ ABSOLUTE RULE — MERGE-FIRST, NEVER SILENT OVERRIDE

The user has stated this as a non-negotiable constraint of the migration. Internalize it before every write operation:

- **You MUST NEVER overwrite a file that already exists target-side with different content** — under any circumstances, without an explicit user decision.
- For EVERY conflict (same path, different content), you MUST:
  1. STOP before writing.
  2. Show a clear diff (or a structured summary of divergent sections if the file is too large for an inline diff — e.g. >500 lines).
  3. Ask the user: keep target / replace with source / merge sections / skip. Wait for the answer.
- This rule applies to ALL destinations covered below: memory files, MEMORY.md, settings.json, .claude.json, hook scripts (`.ps1`), liv-skills, liv-skills.json, agent-memory, plugins, tasks, scheduled_tasks.lock, projects-recent restoration, and any other file you touch.
- **Adds (file does not exist target-side) do NOT require confirmation** — go ahead and create them.
- **Rewrites of Sam-hardcoded paths** (`C:/Users/Sam/...`, `/c/Users/Sam/...`, etc.) inside files you're writing: ALWAYS show the user the rewritten content before writing, even when the file is a pure add. The user must see the final paths.

If you violate this rule, you lose the user's trust and they cannot tell which of their files now contains the wrong content. Treat every existing target-side file as load-bearing until the user says otherwise.

## Inputs available to you

- The repo, already cloned at the canonical path `D:\_DEV\claude-code-game-studios` (or wherever the user actually put it — note the actual path in your first response).
- A folder of files extracted from the source PC's migration archive, at `C:\ccgs-mig\` (or wherever the user unzipped it). Inside you will find:
  - `project/` — gitignored project files (already restored to the repo by the user in step 4 of IMPORT-README).
  - `claude/.claude.json` — **CRITICAL** master config at user-home root (MCP servers, project trust list, OAuth state, session counters). See Step 9.
  - `claude/memory/` — per-project memory `.md` files + `MEMORY.md` index. Run `Get-ChildItem` to enumerate — counts and names drift; do not trust hardcoded lists.
  - `claude/agent-memory/` — agent-level memory (creative-director, producer, technical-director). Distinct from per-project memory.
  - `claude/skills/` — the source PC's subscribed `liv-*` skills directory.
  - `claude/liv-skills.json` — liv-skills subscription manifest. Without it `/liv-sync` misbehaves.
  - `claude/plugins/` — plugin marketplaces state (anything installed via `/plugin`).
  - `claude/tasks/`, `claude/scheduled_tasks.lock` — `/schedule` cron state.
  - `claude/settings.json` — the source PC's GLOBAL Claude settings (reference only — do not overwrite the target's).
  - `claude/stop-sound.ps1`, `ask-sound.ps1`, `notify-sound.ps1` — sound-notification hook scripts referenced by `settings.json`.
  - `claude/CLAUDE.md` — may or may not exist.
  - `claude/projects-recent/` — `.jsonl` transcripts from the last 7 days + the matching session subagent dirs (one per .jsonl). Enables `claude --resume <session-id>` on the target. **See Step 10 for the restoration procedure** (these need to be MOVED into the canonical projects directory, not left in `projects-recent/`).
- The currently-active session state at `production/session-state/active.md` — one source of truth for the resumed task. **See warning in Step 7 about a known-stale header.**

## Reference conversation to preserve

The user has flagged ONE specific Claude conversation as the highest-priority reference to preserve:

- **Name**: "REVIEW SESSION STATE AND SPRINT STATUS"
- **Session ID**: `15301581-d8cd-46e9-a61d-f9a97f15aef5`
- **Files in the archive**:
  - `C:\ccgs-mig\claude\projects-recent\15301581-d8cd-46e9-a61d-f9a97f15aef5.jsonl` (~45 MB)
  - `C:\ccgs-mig\claude\projects-recent\15301581-d8cd-46e9-a61d-f9a97f15aef5\` (subagent dir, ~12 subagents)
- **Verification at end of merge**: confirm that the user can run `claude --resume 15301581-d8cd-46e9-a61d-f9a97f15aef5` on the target — i.e. both the `.jsonl` and the matching session dir are present at `$env:USERPROFILE\.claude\projects\D---DEV-claude-code-game-studios\`.

The user expects you to **report what you merged**, **stop and ask on every existing-file conflict** (per the absolute rule above), and finish by handing them a current-state summary derived from `git log -10`, NOT from `active.md`'s header.

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
  - If target has one → **merge** the bullet lists. Each line is `- [Title](file.md) — one-line hook`.
    - **Order**: keep target's pre-existing entries at the top; append source-from-migration entries below.
    - **Dedupe key**: filename in the link. If a filename appears on both sides, keep ONE bullet — prefer the source-PC description if the underlying `.md` is being copied across.
    - **Conflict on underlying `.md`**: if same filename has different content on both sides, present a unified diff and ask the user (keep target / replace with source / merge sections manually).
  - Show the merged result before writing.

Run `Get-ChildItem C:\ccgs-mig\claude\memory\*.md` to enumerate the actual files. Do NOT trust any hardcoded count or list — memory files drift over time.

### Step 3. Skills merge (`liv-*`)

The source PC has these `liv-*` skills under `~/.claude/skills/`: `liv-bevy-018`, `liv-bevy-lightyear`, `liv-grill-me`, `liv-info`, `liv-list`, `liv-subscribe`, `liv-sync`, `liv-unsubscribe`. Confirm by `Get-ChildItem C:\ccgs-mig\claude\skills\`.

**Bootstrap (chicken-and-egg)**: `/liv-subscribe` and `/liv-sync` are themselves liv-skills. If the target has ZERO liv-* skills installed, neither slash command exists yet on the target — you cannot use them. In that case, manually copy `liv-subscribe`, `liv-sync`, and `liv-list` folders from `C:\ccgs-mig\claude\skills\` to `$env:USERPROFILE\.claude\skills\` first. Once those three exist, the user can run `/liv-list` to verify, then `/liv-subscribe <name>` to install the rest properly.

**Also restore the subscription manifest**: copy `C:\ccgs-mig\claude\liv-skills.json` to `$env:USERPROFILE\.claude\liv-skills.json` if the target has none. If the target has one, present a diff and ask before merging.

**Action (post-bootstrap):**

- For each skill folder in `C:\ccgs-mig\claude\skills\liv-*`:
  - If the target already has the same skill folder → run `/liv-sync` (canonical update flow).
  - If absent → run `/liv-subscribe <skill-name>`. Do not copy folders manually unless the catalog is unreachable.
- After: run `/liv-list` and report subscribed skills.

**Critical**: `liv-bevy-018` and `liv-bevy-lightyear` are **mandatory** for this project (per `CLAUDE.md` and the user's memory `feedback_bevy_skill_mandatory`). If either is missing on the target after step 3, flag loudly.

### Step 4. Hooks / sound notifications (OPTIONAL — ask first)

The source PC's `~/.claude/settings.json` has hooks that play sounds via PowerShell scripts:

- `Stop` → `stop-sound.ps1` (Claude finished)
- `Notification` → `notify-sound.ps1` (system notification)
- `PermissionRequest` / `PreToolUse:AskUserQuestion` → `ask-sound.ps1` (waiting for input)
- Shell-command logging to `/c/Users/Sam/claude-bash-log.txt` (POSIX form, bash hook)
- Prompt-submit logging to `/c/Users/Sam/claude-session-log.txt` (POSIX form, bash hook)

**Action:**

- Ask the user: "Do you want the sound notifications on the target PC? (Yes / No / Choose subset)"
- If yes:
  - Copy the three `.ps1` scripts from `C:\ccgs-mig\claude\` to `$env:USERPROFILE\.claude\` on the target.
  - Show the user the relevant `hooks` block from the source `settings.json` and propose merging into the target's `settings.json`.
  - **Rewrite ALL path forms**, not just the obvious ones. The source `settings.json` mixes four prefixes:
    | Source form | Where it appears | Replace with (PowerShell) |
    |---|---|---|
    | `C:/Users/Sam/.claude/...` | hook `.ps1` paths in `Stop`/`Notification`/`PermissionRequest`/`PreToolUse:AskUserQuestion` | `$env:USERPROFILE/.claude/...` (or expand at write time) |
    | `C:\Users\Sam\.claude\...` (backslashes) | none currently in source but check before writing | `$env:USERPROFILE\.claude\...` |
    | `/c/Users/Sam/...` (POSIX/bash form) | bash hooks under `PreToolUse:Bash` and `UserPromptSubmit` — log file paths | `/c/Users/<TARGET_USER>/...` (bash sees `$HOME` as `/c/Users/<TARGET_USER>` on Git-Bash). DO NOT use `$env:USERPROFILE` here — bash won't expand it. Get the target username via `[Environment]::UserName` and substitute. |
    | `C:/Users/Sam/...` (bare home, not under `.claude/`) | the `claude-bash-log.txt` / `claude-session-log.txt` paths if rewritten back to Windows form | `$env:USERPROFILE/...` |
  - Verify BOTH log paths are rewritten: `claude-bash-log.txt` AND `claude-session-log.txt`. Both POSIX form.

### Step 5. Settings — global vs project (SELECTIVE permissions.allow merge)

The source PC's `~/.claude/settings.json` has 220+ entries in `permissions.allow`. Most are one-shot patterns (`gh run watch <specific-id>`, full `powershell.exe -Command "..."` invocations) that are not worth porting. But a meaningful subset SHOULD be ported — otherwise the user gets dozens of re-approval prompts on day one.

**Action — top-level settings (high-value, port verbatim):**

- `model`, `effortLevel`, `autoUpdatesChannel`, `skipDangerousModePermissionPrompt`, `skipAutoPermissionPrompt`
- `permissions.defaultMode`
- `permissions.additionalDirectories` — see Step 6 for path rewrites first
- `hooks` — see Step 4 for path rewrites first

Propose merging only those into the target's `settings.json`. Show a diff before writing.

**Action — `permissions.allow` selective port:**

Extract from the source array ONLY the broad-and-stable patterns:
- All `Skill(...)` entries (pre-authorized skills the user trusts).
- All `WebFetch(domain:*)` entries (e.g. `docs.rs`, `github.com`, project wikis).
- Bare-glob bash patterns: `Bash(cargo *)`, `Bash(cargo check *)`, `Bash(cargo test *)`, `Bash(git *)`, `Bash(git push *)`, `Bash(git commit -m ' *)`, `Bash(git add *)`, `Bash(git reset *)`, `Bash(git fetch *)`, `Bash(git pull *)`, `Bash(git stash *)`, `Bash(git restore *)`, `Bash(git rm *)`, `Bash(git check-ignore *)`, `Bash(git show *)`, `Bash(gh run *)`, `Bash(gh release *)`, `Bash(gh api *)`, `Bash(where.exe cargo *)`.
- Bare wildcards under `Edit(/...)` and `Read(/...)` that target broad project areas.

**Skip**: every entry containing `powershell.exe -Command "..."` (long one-shots) and every entry with a specific run-id or session-id baked in.

Show the user the filtered list before writing. Roughly: ~30-40 entries out of 220 should survive the filter.

The project-local `.claude/settings.local.json` was restored in step 4 of IMPORT-README — verify it's at `D:\_DEV\claude-code-game-studios\.claude\settings.local.json`. Keep this file verbatim.

### Step 6. `additionalDirectories` paths

Source `settings.json` has `additionalDirectories` with mixed-quality entries. Process them:

| Entry | Action |
|---|---|
| `"C:/Users/Sam/.claude"` | **Rewrite** to `$env:USERPROFILE/.claude` (or the actual target user path). The username "Sam" is hardcoded — this fails on any target unless the user is also "Sam". |
| `"\\tmp"` | **Drop**. Malformed path (Windows interprets as bare-root `\tmp` which resolves nowhere). Pre-existing bug, not a migration regression. |
| Anything under `d:\_DEV\claude-code-game-studios\...` | Keep as-is IF target cloned to the same path. Otherwise rewrite the prefix to the new clone path. |
| `"D:\_DEV\claude-code-game-studios\docs\architecture"` (duplicated with different casing) | Dedupe — keep one. |

Show the rewritten list to the user before writing.

### Step 7. Resume the task — RECONCILE WITH GIT LOG, DON'T TRUST THE HEADER

Read `production/session-state/active.md` end-to-end. It contains the rolling state of the project — Session Extracts back to 2026-04-29.

**⚠️ Header staleness warning**: the banner at the TOP of `active.md` can lag the actual current state by many PROMPTs. The Session Extract sections in the body are reliable; the header is hand-maintained and drifts.

**Procedure to determine "where the user actually is":**

1. Read `active.md` body for context (Session Extracts, carried QA conditions, structural decisions, blockers).
2. Run `git log --oneline -30` and look at the most recent N commits. The commit subjects encode the actual current PROMPT and Sprint (e.g. `PROMPT 705 / S11-TEST-LOBBY-ENTRY-IDEMPOTENT-ALIGNMENT-001`).
3. Cross-reference with `production/sprint-status.yaml` for the canonical sprint number.
4. If `active.md` header disagrees with git log (it probably will), **trust git log**.

Hand the user a one-line summary referencing the actual most-recent commit, not the header. Example: "Last committed work: PROMPT N on S11-... (commit abc1234). Active blockers from `active.md`: X, Y. Resume?"

Do NOT immediately implement the next task. Wait for the user's go.

### Step 8. `.cargo/config.toml` (cargo build linker config)

The archive does NOT include `.cargo/config.toml` from the source repo (it's machine-specific to the source PC's MSVC linker location). On the target:

1. Run `cargo check --workspace` from the cloned repo.
2. If it succeeds → done, no action needed.
3. If it fails with a linker error (`link.exe not found`, etc.) → the user needs to install or open Developer PowerShell for VS 2022/2026 to make MSVC tools visible, and may want to recreate `.cargo/config.toml` with a portable `target-dir = "target/msvc-local"` override (carried over from the user's memory `project_tech_stack`). Ask the user before writing this file — it's gitignored.

### Step 9. `~/.claude.json` master config merge

`C:\ccgs-mig\claude\.claude.json` is the source PC's master config. It typically contains: MCP server definitions, project trust list, OAuth/account state, session counters.

**⚠️ Do NOT blind-overwrite the target's `$env:USERPROFILE\.claude.json`** — that would clobber the target's OAuth and other-project trust list. Apply the ABSOLUTE RULE (top of doc).

**Action — surgical merge:**

1. Read both source and target `.claude.json`.
2. If target has no `.claude.json` → copy source verbatim, then warn the user that `claude login` may still be required (OAuth tokens may not be portable per Anthropic's design).
3. If target has one → merge only the project-scoped sections:
   - The `projects` (or equivalent) sub-object's entry for the repo path — copy from source.
   - Any MCP server definitions used by this project — confirm with the user before adding.
   - Leave the target's OAuth and other-project entries untouched.
4. Show a JSON diff before writing.

### Step 10. Restore `projects-recent/` so `claude --resume` works

The archive's `claude/projects-recent/` contains `.jsonl` transcripts (last 7 days) and matching session subagent dirs. Claude Code's `--resume` feature reads them from the canonical location `$env:USERPROFILE\.claude\projects\D---DEV-claude-code-game-studios\`, NOT from `projects-recent/`. You must MOVE them.

**Action:**

1. Ensure the target directory exists:
   ```powershell
   New-Item -ItemType Directory -Force `
     -Path "$env:USERPROFILE\.claude\projects\D---DEV-claude-code-game-studios" | Out-Null
   ```
2. For each `.jsonl` file in `C:\ccgs-mig\claude\projects-recent\*.jsonl`:
   - Check if `$env:USERPROFILE\.claude\projects\D---DEV-claude-code-game-studios\<same-name>.jsonl` exists.
   - If absent → copy it across. (Pure add — no confirmation needed per the absolute rule.)
   - If present → apply the ABSOLUTE RULE: show a comparison (file size + last 50 lines diff is enough for these — full diff is impractical for 45 MB files), ask the user. Most likely "keep target" since `.jsonl` files are append-only logs.
3. For each session subdirectory in `C:\ccgs-mig\claude\projects-recent\` (UUID-named folders containing `subagents/`):
   - Check if `$env:USERPROFILE\.claude\projects\D---DEV-claude-code-game-studios\<session-id>\` exists.
   - If absent → copy the whole subdir across.
   - If present → apply the ABSOLUTE RULE.

**Verification — the reference conversation**:

After step 10, confirm that both of these exist target-side:
- `$env:USERPROFILE\.claude\projects\D---DEV-claude-code-game-studios\15301581-d8cd-46e9-a61d-f9a97f15aef5.jsonl`
- `$env:USERPROFILE\.claude\projects\D---DEV-claude-code-game-studios\15301581-d8cd-46e9-a61d-f9a97f15aef5\subagents\` (12+ agent jsonls)

If either is missing, surface it loudly — `claude --resume 15301581-d8cd-46e9-a61d-f9a97f15aef5` will fail without them. This is the user's reference conversation "REVIEW SESSION STATE AND SPRINT STATUS".

If both are present, tell the user explicitly in your final report:
> Reference conversation restored. You can resume with:
> `claude --resume 15301581-d8cd-46e9-a61d-f9a97f15aef5`

## Things to flag to the user proactively

1. **`.cargo/config.toml`** — see Step 8. May or may not need recreation.
2. **Auth**: the user must `claude login`, `codex login`, `gh auth login` themselves.
3. **Git stashes** were intentionally not migrated (the user commits manually before export). Confirm no expected stashes are missing.
4. **`.jsonl` transcripts**: only the last 7 days are in `claude/projects-recent/`. `/resume` for older sessions won't work.
5. **External worktrees** at `D:\_DEV\claude-code-game-studios-worktrees\` (236 branches on source) were NOT migrated. If the user had WIP in any, it's lost unless they pushed branches.

## Style and constraints

- Match the user's collaboration protocol: ask before writing files, show drafts, never auto-commit (per `CLAUDE.md` "Collaboration Protocol").
- This user is `benachi.samy@gmail.com`, working solo on a Bevy 0.18 + Lightyear card game (Lanes and Lies). French is fine; English is fine.
- See the project's per-project memory (after Step 2) for stronger context on their preferences (parallelism-first workflow, orchestrator-skills-flow, paw-review-flow, etc.).

## Done condition

You are done when ALL of the following are verifiable, not subjective:

- `Get-ChildItem $env:USERPROFILE\.claude\projects\D---DEV-claude-code-game-studios\memory\` returns the merged set of `.md` files including `MEMORY.md`.
- `$env:USERPROFILE\.claude\liv-skills.json` exists.
- `/liv-list` output contains at minimum `liv-bevy-018` and `liv-bevy-lightyear` (or the user explicitly waived them in chat).
- If sound hooks were opted-in: the three `.ps1` scripts exist under `$env:USERPROFILE\.claude\`, and the target's `settings.json` references them via portable paths (no hardcoded "Sam").
- **Reference conversation restored**: `$env:USERPROFILE\.claude\projects\D---DEV-claude-code-game-studios\15301581-d8cd-46e9-a61d-f9a97f15aef5.jsonl` exists AND `...\15301581-d8cd-46e9-a61d-f9a97f15aef5\subagents\` dir exists with its subagent transcripts. Test: `claude --resume 15301581-d8cd-46e9-a61d-f9a97f15aef5` would not error out due to missing files.
- **No silent overrides**: you can produce a list of every existing-file conflict you encountered, with the user's decision recorded for each.
- `cargo check --workspace` succeeded once OR the user acknowledged a known linker issue.
- `$env:USERPROFILE\.claude.json` was merged or copied per Step 9, AND the user confirmed in chat.
- The user has been briefed using **commit-derived** current state (not active.md header).
- A short migration report has been delivered listing: files merged, files skipped, decisions deferred to the user, and any errors.
