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

If `~/.codex/` doesn't exist yet, create it first. The archive excludes the huge `memories/` build cache; Codex will rebuild it on first compile.

### 6. Claude — merge via agent (DO NOT BLIND-COPY)

Open Claude Code in the cloned repo:

```powershell
cd D:\_DEV\claude-code-game-studios
claude
```

In the chat, paste:

> Read `tools/migration/MIGRATION-HANDOVER.md` and execute the merge plan it describes. The migration archive is unzipped at `C:\ccgs-mig\claude\`. Do not overwrite my existing global Claude state — merge memories and skills selectively as the handover document instructs.

The handover document tells the agent exactly:
- Which memory files to copy across (per-project memory only — won't touch your other projects)
- Which `liv-*` skills to install via `/liv-subscribe`
- Which `.claude/settings.json` snippets to merge (hooks, permissions) — and which to skip
- Whether to install the sound-notification `.ps1` hook scripts
- What's in `production/session-state/active.md` (the current task) so the agent can resume

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
- **`.jsonl` conversation transcripts** (`~/.claude/projects/*/*.jsonl`) — big, only used by `/resume` history view, not by memory/skills. Lost.
- **`~/.codex/memories/<project>-target/`** — Rust build cache, regenerates on next `cargo build`.
- **`~/.claude/file-history/`, `shell-snapshots/`, `paste-cache/`** — ephemeral.
- **`target/`** in the repo — Cargo build output.
- **Auth tokens** — per-device by design.
- **`.cargo/config.toml`** — Windows-specific linker config, regenerate per machine.
