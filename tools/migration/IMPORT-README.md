# Migration Import — Target PC

You have `ccgs-migration-YYYY-MM-DD.zip` produced by `export-migration.ps1` on the source PC. This guide gets you back to "exactly where I was" on the target PC, **without clobbering the existing Claude setup** that already lives there.

## TL;DR

1. Install Rust + tooling.
2. `git clone` the repo (same path as source if possible — see "Path gotcha").
3. Unzip the migration archive somewhere temporary.
4. Restore project gitignored files (safe — those slots are empty on a fresh clone).
5. Restore Codex (target is fresh — safe to overwrite).
6. **Do NOT blind-copy Claude state.** Hand the unzipped archive's `claude/` folder to a Claude Code agent on the target PC and tell it to read `production/session-state/MIGRATION-HANDOVER.md`. The agent will merge.
7. Re-login: `claude login`, `codex login` (per-device).
8. Regenerate `.cargo/config.toml` for the target machine's toolchain.

## Path gotcha

Claude's per-project memory directory is keyed by the project path:
`~/.claude/projects/D---DEV-claude-code-game-studios/`.

If you clone to a different path (e.g. `C:\dev\ccgs`), Claude will start with empty per-project memory on the target. **Cloning to the same path `D:\_DEV\claude-code-game-studios` is strongly recommended** — the handover agent can then merge memories into the matching directory automatically.

## Step-by-step

### 1. Toolchain

- Install Rust via `rustup` (stable).
- Install Trunk: `cargo install trunk`.
- Install Visual Studio 2022/2026 Build Tools (or full VS) — required for the MSVC linker on Windows.
- Install `gh` (GitHub CLI) if you use it.

### 2. Clone the repo

```powershell
git clone <your-remote-url> D:\_DEV\claude-code-game-studios
cd D:\_DEV\claude-code-game-studios
```

### 3. Unzip the migration archive

```powershell
Expand-Archive C:\path\to\ccgs-migration-YYYY-MM-DD.zip -DestinationPath C:\ccgs-mig
```

You should see: `C:\ccgs-mig\project\`, `C:\ccgs-mig\claude\`, `C:\ccgs-mig\codex\`.

### 4. Restore project gitignored state (safe overwrite — fresh clone has none)

```powershell
Copy-Item C:\ccgs-mig\project\* D:\_DEV\claude-code-game-studios\ -Recurse -Force
```

This restores `production/session-state/`, `production/session-logs/`, `.claude/settings.local.json`, `CLAUDE.local.md`, `.agents/`, `.codex-tmp/`, `expansions/`.

### 5. Restore Codex (fresh target — safe overwrite)

```powershell
Copy-Item C:\ccgs-mig\codex\* $env:USERPROFILE\.codex\ -Recurse -Force
```

If `~/.codex/` doesn't exist yet, create it first.

**What the archive excludes (intentionally):**
- `~/.codex/memories/<project>-target/` — Rust build cache, NOT conversations (the name is misleading). Multi-GB. Regenerates on next `cargo build`.
- `~/.codex/.tmp/` — scratch.
- `~/.codex/logs_2.sqlite*` — 3.3 GB telemetry DB, regenerates locally.

**What IS included (your conversations):**
- `~/.codex/sessions/YYYY/MM/DD/rollout-<timestamp>-<session-id>.jsonl` — every Codex conversation you've had, organized by date. Verified: confirmed working with `codex resume <session-id>` on the source PC.

### 5b. Resuming a specific Codex conversation

Once the `sessions/` tree is restored, you can resume any past conversation by its session ID:

```powershell
codex resume <session-id>
```

The session ID is the UUID at the end of the rollout filename — e.g. for `rollout-2026-04-30T10-24-52-019dddb4-95f7-79e1-b48c-fdfc34fa3cd8.jsonl`, the ID is `019dddb4-95f7-79e1-b48c-fdfc34fa3cd8`.

**Finding a session by content** (when you remember what it was about but not the ID):

```powershell
# Grep across all session files for a distinctive string
Get-ChildItem "$env:USERPROFILE\.codex\sessions" -Recurse -Filter "*.jsonl" `
  | Select-String -Pattern "REVIEW SPRINT STATUS" -CaseSensitive:$false `
  | Select-Object -ExpandProperty Path -Unique
```

The filename of any match contains the session ID. Replace the pattern with whatever distinctive phrase you recall from the conversation (a label you used, a unique command name, a specific error).

### 6. Claude — merge via agent (DO NOT BLIND-COPY)

Open Claude Code in the cloned repo:

```powershell
cd D:\_DEV\claude-code-game-studios
claude
```

**Then paste this prompt verbatim** (copy the entire block between the `---` markers, including the absolute-rule paragraph):

---

```
Lis `tools/migration/MIGRATION-HANDOVER.md` end-to-end puis exécute le merge plan complet, dans l'ordre des étapes. Tu as carte blanche pour tout le travail technique (copies, restaurations, installations de skills, rewrites de paths) — je ne veux rien avoir à faire moi-même côté technique.

L'archive de migration est dépaquetée à `C:\ccgs-mig\` (sous-dossiers : `project/`, `claude/`, `codex/`).

RÈGLE ABSOLUE — MERGE-FIRST, JAMAIS D'ÉCRASEMENT SILENCIEUX :
- Tu ne dois JAMAIS écraser silencieusement un fichier qui existe déjà sur ce PC avec un contenu différent du source.
- Pour CHAQUE conflit (même chemin, contenu différent), tu STOP, tu présentes un diff clair (ou un résumé des sections divergentes si le fichier est trop gros), et tu me demandes : keep target / replace with source / merge sections / skip. Aucune exception.
- Cette règle s'applique à TOUT : memory files, MEMORY.md, settings.json, .claude.json, hooks (`.ps1`), liv-skills, liv-skills.json, agent-memory, plugins, tasks, scheduled_tasks.lock, et tout autre destination side.
- Si un fichier n'existe PAS encore target-side : tu peux le créer directement sans me demander (c'est un add, pas un override).
- Pour les inserts/rewrites de paths Sam-hardcoded dans settings.json (forme `C:/Users/Sam/...`, `/c/Users/Sam/...`, etc.) : tu rewrites en utilisant `$env:USERPROFILE` ou le username target, et tu me montres la version finale avant write — même si la version actuelle target n'a pas ce chemin.

PRIORITÉS DE PRÉSERVATION :
1. Conversation Claude de référence à préserver absolument : session ID `15301581-d8cd-46e9-a61d-f9a97f15aef5` ("REVIEW SESSION STATE AND SPRINT STATUS"). Son `.jsonl` (45 MB) et son subagent dir sont dans l'archive sous `C:\ccgs-mig\claude\projects-recent\`. À la fin, vérifie que `claude --resume 15301581-d8cd-46e9-a61d-f9a97f15aef5` fonctionnerait (le fichier doit être à `$env:USERPROFILE\.claude\projects\D---DEV-claude-code-game-studios\` après ton merge).
2. État de session courant : `production/session-state/active.md` (déjà restauré à l'étape 4 — vérifie qu'il est bien là).
3. Toutes les mémoires per-project et agent-memory : à merger une par une avec la règle ci-dessus.

À LA FIN :
- Rapport structuré : (a) fichiers créés, (b) fichiers mergés avec mes décisions, (c) fichiers skipped à ma demande, (d) erreurs ou cas que tu n'as pas pu résoudre.
- État réel actuel via `git log --oneline -10` (PAS via le header de `active.md` qui peut être stale).
- Confirme que `claude --resume 15301581-d8cd-46e9-a61d-f9a97f15aef5` est prêt à fonctionner.
```

---

The handover document (`tools/migration/MIGRATION-HANDOVER.md`) tells the agent exactly:
- Which memory files to merge (per-project memory only — won't touch your other projects)
- Which `liv-*` skills to install via `/liv-subscribe` (and how to bootstrap when the target is empty)
- Which `.claude/settings.json` snippets to port and which to skip
- Which `permissions.allow` entries to keep (`Skill(*)`, `WebFetch(domain:*)`, bare `Bash(cargo/git/gh *)`) and which to drop
- How to merge `~/.claude.json` (MCP, project trust) WITHOUT clobbering the target's OAuth
- Whether to install the sound-notification `.ps1` hook scripts
- How to restore `projects-recent/` so `claude --resume` works
- How to reconcile `active.md` header against `git log` (header may be stale)

### 7. Authenticate

- `claude login` — per-device, can't be migrated.
- `codex login` — same.
- `gh auth login` if you use GitHub CLI.

### 8. Local Rust config

The source PC has `.cargo/config.toml` pointing at a Windows-specific linker (Developer PowerShell for VS 2026, `target-dir=target/msvc-local`). **Do not copy this file from the source.** Regenerate it on the target with paths that match the target's VS installation, or omit it (cargo will fall back to defaults).

### 9. Verify

```powershell
cargo check --workspace        # should compile clean from the cloned source
git status                     # should be clean (gitignored files restored, but git won't see them)
```

Then in Claude: `/help` or open the session — the `session-start` hook should detect `production/session-state/active.md` and brief you on the resumed task.

## What's intentionally NOT migrated

- **Git stashes** — the user will commit these manually before exporting.
- **Claude `.jsonl` transcripts older than 7 days** (`~/.claude/projects/*/*.jsonl`) — recent 7 days are kept under `claude/projects-recent/` for `/resume`; older ones are dropped.
- **`~/.codex/memories/<project>-target/`** — Rust build cache (NOT conversations). Regenerates on next `cargo build`.
- **`~/.codex/logs_2.sqlite*`** — 3.3 GB telemetry, regenerates locally.
- **`~/.claude/file-history/`, `shell-snapshots/`, `paste-cache/`** — ephemeral.
- **`target/`** in the repo — Cargo build output.
- **Auth tokens** — per-device by design.
- **`.cargo/config.toml`** — Windows-specific linker config, regenerate per machine.
- **External worktrees** at `<repo>-worktrees/` — not migrated. Commit/push WIP from any active worktree before export.

## What IS migrated for Codex (so you don't lose past conversations)

All Codex conversations live under `~/.codex/sessions/YYYY/MM/DD/rollout-*.jsonl` and are included in the archive in full. Confirmed working: `codex resume <session-id>` on a session migrated to the target PC. See section 5b above for finding a session by content.
