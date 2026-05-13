# Octogent Integration — GCS Orchestrator Pipeline

> Sidecar layer that wires the Codex orchestrator's textual dispositions
> (`NEW / CLEAR / REPONDRE / RELANCER -- PROMPT N` blocks and `PROMPT N --
> Title` launch headers) to the [Octogent](https://github.com/hesamsheikh/octogent)
> dashboard, so worker spawns/kills become visible and inspectable in real
> time without modifying the orchestrator or its output format.

**Status**: operational since 2026-05-13. Setup is per-developer-machine; no
runtime dependency on Octogent is required for the orchestrator to function.

---

## 1. What this is (and is not)

| It is | It is not |
|---|---|
| A passive **observer + dispatcher** that reads what the orchestrator just said and translates `NEW/CLEAR/REPONDRE/RELANCER/PROMPT` blocks into Octogent API calls | A modification of the orchestrator or its contract |
| A **visual cockpit** for multi-worker GCS waves | A scheduler/planner — the orchestrator still decides everything |
| A **convenience layer** — fully optional and rollback-able in <30s | A required service — orchestrator works unchanged if Octogent is off |
| Local-only (per-developer) | A team-shared service |

## 2. Architecture — sidecar / observer pattern

```
┌──────────────────────────────────────┐
│  Codex Orchestrator (UNCHANGED)      │   ← keeps its existing contract
│  (Current Operating Rules            │      (production/session-state/
│   2026-05-13 override)               │       codex-orchestrator-state.md)
│                                      │
│  emits text blocks in conversation:  │
│    NEW -- PROMPT N                   │
│    PROMPT N -- Short Task Name       │
│    body...                           │
│    CLEAR -- PROMPT M                 │
│    REPONDRE -- PROMPT P              │
│    RELANCER -- PROMPT Q              │
└──────────────┬───────────────────────┘
               │
               │ Codex writes to ~/.codex/sessions/.../rollout-<uuid>.jsonl
               │
               ▼ (Codex Stop hook fires at end of every assistant turn)
┌──────────────────────────────────────┐
│  ~/.codex/gcs-codex-stop-hook.py     │
│   • opens latest rollout (mtime)     │
│   • extracts last assistant          │
│     output_text                      │
│   • copies codex-orchestrator-       │
│     state.md → CONTEXT.md            │
│   • pipes text to dispatcher         │
└──────────────┬───────────────────────┘
               │ stdin
               ▼
┌──────────────────────────────────────┐
│  ~/.codex/gcs-octogent-dispatch.py   │
│   • regex-splits text into blocks    │
│   • per block: HTTP POST + sound     │
│   • writes inbox/<ts>.md backup      │
│   • logs every action                │
└──────────────┬───────────────────────┘
               │ HTTP (loopback only)
               ▼
        http://127.0.0.1:8787
   (Octogent API + Web UI on 5173)
               │
               ▼
   Workers appear/disappear under the
   `gcs-orchestrator` tentacle in the UI
```

**Key invariant**: the orchestrator does not know any of this exists. Stop the
dispatcher, kill Octogent, delete the hook — orchestrator behavior is
identical. The integration is a pure observer.

## 3. The five header patterns

The dispatcher recognises exactly these line-starts (case-sensitive, anchored
to start of line, `\d+` is the prompt number):

| Pattern | Action | Octogent API call | Sound |
|---|---|---|---|
| `PROMPT N -- Title` | **SPAWN** worker `PROMPT-N` under `gcs-orchestrator`; body (until next header) becomes the `initialPrompt` | `POST /api/terminals` | `oui-messire.mp3` |
| `NEW -- PROMPT N` | **NO-OP** (disposition label only). Recognised so it cleanly delimits preceding/following blocks; the real SPAWN trigger is the `PROMPT N -- Title` line that follows. Fallback: if no matching `PROMPT N -- Title` appears in the same response, an empty placeholder terminal is spawned and a WARN is logged. | (no call unless fallback) | (silent) |
| `CLEAR -- PROMPT N` | **KILL** worker `PROMPT-N` | `POST /api/terminals/PROMPT-N/kill` | `travail-termine.mp3` |
| `REPONDRE -- PROMPT N` | **Channel-message** the existing worker `PROMPT-N` with the body | `POST /api/channels/PROMPT-N/messages` | `pret-a-travailler.mp3` |
| `RELANCER -- PROMPT N` | **Stop + recreate** worker `PROMPT-N` with new body | `POST .../stop` then `POST /api/terminals` | `encore-du-travail.mp3` |

### Worked example

Orchestrator emits, in a single turn:

```text
CLEAR -- PROMPT 762
Worker terminé, push OK sur work/HAND-UI-016.

NEW -- PROMPT 763

PROMPT 763 -- Sprint 10 Polish Close-Out Disposition

Agent/skills:
- producer / qa-lead
Repo/mode:
- Branch: main
- Worktree: D:\_DEV\claude-code-game-studios-worktrees\sprint-10-closeout
```

Dispatcher does, in order:

1. CLEAR PROMPT-762 → `POST /api/terminals/PROMPT-762/kill` → `travail-termine.mp3`
2. NEW marker for PROMPT-763 noted, no-op
3. SPAWN PROMPT-763 with title "Sprint 10 Polish Close-Out Disposition" and body = the Agent/skills + Repo/mode block → `POST /api/terminals` → `oui-messire.mp3`

Result in UI: terminal `PROMPT-762` disappears, terminal `PROMPT-763` appears
under tentacle "Codex Orchestrator State".

### Things that do NOT match

| Line | Why ignored |
|---|---|
| `Prompt 813:` | Lowercase `p` |
| `Nouveau prompt #813:` | Wrong wording |
| `PROMPT 813 - Title` | Single dash (regex requires `--`) |
| `   PROMPT 813 -- Title` | Indented (regex anchors to line start) |
| `Voici le prompt à lancer:` | No header at all |
| `NEW: PROMPT 813` | Wrong separator (colon, not `--`) |

If the orchestrator's wording drifts, update `_HEADER_RX` in
`~/.codex/gcs-octogent-dispatch.py`. Five lines, no other code change needed.

## 4. Components — what lives where

### Inside this repo (committed)
- `docs/octogent-integration.md` — this file
- `.gitignore` — excludes `.octogent/`

### Inside this repo (gitignored, per-developer)
- `.octogent/tentacles/gcs-orchestrator/` — Octogent's view of the orchestrator
  - `CONTEXT.md` — auto-synced mirror of `production/session-state/codex-orchestrator-state.md` (updated on every Codex Stop hook)
  - `todo.md` — local checkbox file (informational, not auto-spawned from)
  - `prompt-log.md` — append-only log (manual or future-automatic)
  - `inbox/<ts>.md` — backup of every dispatched orchestrator response (anti-crash recovery)
- `.octogent/worktrees/` — git worktrees Octogent creates per worker (we currently use `workspaceMode=shared`, so this stays empty)

### Outside this repo (user-global config)

#### Codex side
- `~/.codex/gcs-octogent-dispatch.py` — the dispatcher (regex + HTTP + sounds)
- `~/.codex/gcs-codex-stop-hook.py` — the Codex Stop hook entry point
- `~/.codex/hooks.json` — Codex hook config (2 Stop hooks: sound + dispatcher)
- `~/.codex/gcs-dispatch.log` — append-only action log
- `~/.codex/gcs-stop-hook.log` — append-only hook-fire log
- `~/.codex/sessions/.../rollout-*.jsonl` — Codex's own rollout (read-only source for the hook)

#### Claude Code side
- `~/.claude/settings.json` — adds Octogent `/api/hooks/stop` and `/api/hooks/notification` callbacks (so Claude Code workers spawned inside Octogent terminals get idle-gated)

#### Octogent install + binding
- `D:\_APPS\Tools\octogent\` — the cloned Octogent source (Node 22+ / pnpm-managed)
- `D:\_APPS\Tools\octogent\scripts\dev.mjs` — patched locally to use `pnpm.cmd` (not `pnpm.exe`, which doesn't exist with `npm install -g pnpm` on Windows) and `shell: true` (Node 24 requirement). **Lost on `git pull` of Octogent — re-apply if you upgrade.**
- `D:\_APPS\Tools\octogent\.octogent\tentacles\gcs-orchestrator` — directory **junction** pointing at this repo's `.octogent/tentacles/gcs-orchestrator/`. Lets the install dir see GCS tentacle without needing `OCTOGENT_WORKSPACE_CWD` (which crashes Vite with `0xC0000409`).
- `D:\_APPS\Tools\octogent\launch-here.bat` — launcher invoked by the right-click context menu; spawns a persistent cmd via `start cmd /k`.
- `D:\_APPS\Tools\octogent\install-context-menu.reg` + `uninstall-context-menu.reg` — UTF-16 LE BOM `.reg` files for the Explorer right-click. Currently installed under `HKCU\Software\Classes\Directory\{Background\,}shell\Octogent`.

#### Sound assets
- `~/.claude/sounds/play-sound.ps1` — PowerShell MediaPlayer wrapper (pre-existing)
- `~/.claude/sounds/oui-messire.mp3` — PROMPT spawn
- `~/.claude/sounds/pret-a-travailler.mp3` — REPONDRE
- `~/.claude/sounds/travail-termine.mp3` — CLEAR (also bound to base Codex/Claude Stop)
- `~/.claude/sounds/encore-du-travail.mp3` — RELANCER (also Notification permission)

## 5. First-time setup for a new dev machine

Prerequisites: Node 22+, git, gh, curl. PowerShell + cmd. Codex CLI ≥0.130, Claude Code ≥2.1.

1. Clone Octogent: `git clone https://github.com/hesamsheikh/octogent D:\_APPS\Tools\octogent`
2. Install pnpm (user-prefix, no admin): `npm install -g pnpm`
3. Patch `D:\_APPS\Tools\octogent\scripts\dev.mjs`:
   - Line ~74: change `"pnpm.exe"` to `"pnpm.cmd"`
   - In the `spawn()` call (~line 125), add `shell: process.platform === "win32",`
4. Copy these files from a teammate or from a backup:
   - `~/.codex/gcs-octogent-dispatch.py`
   - `~/.codex/gcs-codex-stop-hook.py`
5. Add the Codex Stop hook to `~/.codex/hooks.json` (under `hooks.Stop[].hooks`):
   ```json
   { "type": "command", "command": "python C:/Users/Sam/.codex/gcs-codex-stop-hook.py" }
   ```
6. Trust the new hook on next Codex launch (press `t` in the hook review screen).
7. From this repo's root, right-click context menu install:
   - Copy `install-context-menu.reg`, `uninstall-context-menu.reg`, `launch-here.bat` from `D:\_APPS\Tools\octogent\` (already there if step 1 done)
   - Run `reg import D:\_APPS\Tools\octogent\install-context-menu.reg` (or double-click; UTF-16 BOM required)
8. In this repo, scaffold the tentacle:
   ```
   mkdir .octogent\tentacles\gcs-orchestrator\inbox
   copy production\session-state\codex-orchestrator-state.md .octogent\tentacles\gcs-orchestrator\CONTEXT.md
   echo "# GCS Orchestrator — Todo" > .octogent\tentacles\gcs-orchestrator\todo.md
   echo "# Prompt Log" > .octogent\tentacles\gcs-orchestrator\prompt-log.md
   ```
9. Create the install-dir junction so Octogent sees the tentacle:
   ```
   mklink /J "D:\_APPS\Tools\octogent\.octogent\tentacles\gcs-orchestrator" "D:\_DEV\Work\Claude-Code-Game-Studios\.octogent\tentacles\gcs-orchestrator"
   ```
10. Launch: right-click on the repo folder in Explorer → "Show more options" → "Open Octogent here". UI opens on `http://localhost:5173`.

## 6. Daily use

1. Make sure Octogent is running: right-click on the repo folder → "Open Octogent here". A persistent cmd window logs `Octogent API listening on http://127.0.0.1:8787` and `Local: http://localhost:5173/`.
2. Open `http://localhost:5173/` in a browser. The `Codex Orchestrator State` tentacle should appear.
3. Work with the Codex orchestrator normally. On each turn it ends with proper blocks, workers appear/disappear under the tentacle automatically.
4. To inspect: click any worker in the UI to see its transcript and channel messages.
5. To force a manual action without going through the orchestrator (debug):
   ```bash
   echo "CLEAR -- PROMPT 999" | python ~/.codex/gcs-octogent-dispatch.py
   ```
6. To stop Octogent cleanly: Ctrl+C in the cmd window. Don't click the X — leaves orphan node processes.

## 7. Logs and troubleshooting

| Symptom | First check |
|---|---|
| Block emitted but no terminal appears | `tail -30 ~/.codex/gcs-dispatch.log` — is the block detected? Is the POST returning 201? |
| Hook not firing | `tail -10 ~/.codex/gcs-stop-hook.log` — should show `=== stop hook fired ===` on every Codex turn |
| Hook fires but dispatcher silent | The hook found no rollout or the last assistant message is empty — check `~/.codex/sessions/.../rollout-*.jsonl` exists |
| Wrong terminal got killed | `terminalId` collision — the dispatcher uses `PROMPT-<N>` so two waves reusing the same N collide. Always use monotonically-increasing N. |
| Vite crashes with `0xC0000409` | You set `OCTOGENT_WORKSPACE_CWD`; remove it from the launcher. Use the junction approach instead. |
| `pnpm.exe ENOENT` | `dev.mjs` not patched (step 3 above). Change `pnpm.exe` → `pnpm.cmd`, add `shell: true`. |
| `git worktree remove` fails on DELETE | Use `POST /api/terminals/prune` instead of the UI delete button. |
| API on 8787 unreachable | `netstat -ano \| grep ":8787"` — is anything listening? If not, Octogent is dead — relaunch from Explorer right-click. |
| 2 mystery `node.exe` running, no port bound | `pnpm dev` parent + child orphan from a crashed launch; `taskkill /F /PID <pid>` is safe. |
| Sound silent | The Claude/Codex sound hooks rely on `~/.claude/sounds/play-sound.ps1` and the 4 mp3 files being present. |

## 8. Caveats and known issues

1. **PTY non-persistent**: if Octogent crashes mid-wave, every live worker dies. The `inbox/<ts>.md` backup of each orchestrator response gives a manual recovery path.
2. **Channels are in-memory**: messages queued via `octogent channel send` are lost on Octogent restart. The dispatcher only does spawn/kill/replace and `POST /api/channels/.../messages`, so a restart wipes pending REPONDRE deliveries that hadn't been injected yet.
3. **9-children-per-parent limit**: hard cap inside Octogent. Current waves of 4–7 are well under. For 12+ lanes, chain a sub-parent.
4. **Stop hook fires for every Codex session**: non-orchestrator sessions (research, dev, etc.) trigger the dispatcher too. Their responses contain no actionable headers, so the dispatcher logs `no actionable blocks found` and exits silently. Safe but watch the log noise.
5. **Per-developer setup**: nothing is shared. Each dev runs their own Octogent instance on their own machine.
6. **Octogent's own dev tentacles** (`api-runtime`, `web-ui`, etc.) are visible because the junction lives under the install dir. They're inert noise — click **HIDE IDLE** in the toolbar to mute them.
7. **Patched `dev.mjs` lost on Octogent upgrade**: re-apply the two-line patch after `git pull` on the Octogent clone.

## 9. Full rollback

```bash
# 1. Codex side — remove the new hook
#    Edit ~/.codex/hooks.json, drop the python command from the Stop array
#    (or untrust + delete via the Codex hook picker)
# 2. Claude side — remove the curl POSTs
#    Edit ~/.claude/settings.json, remove the /api/hooks/* commands from
#    the Stop and Notification arrays (keep the sound hooks).
# 3. Scripts
rm ~/.codex/gcs-octogent-dispatch.py ~/.codex/gcs-codex-stop-hook.py
# 4. Tentacle scaffolding (per repo)
rm -rf .octogent/
# 5. Octogent itself
taskkill /F /PID <pid>     # if running
# 6. Right-click context menu
reg import D:\_APPS\Tools\octogent\uninstall-context-menu.reg
# 7. Optionally remove the install
rm -rf D:\_APPS\Tools\octogent
```

After step 1+2 the orchestrator continues to work exactly as before with zero
behavioural change. The remaining steps just reclaim disk and tidy the
context menu.

## 10. Related

- [Octogent — GitHub](https://github.com/hesamsheikh/octogent)
- [Octogent API reference](https://github.com/hesamsheikh/octogent/blob/main/docs/reference/api.md)
- `production/session-state/codex-orchestrator-state.md` — orchestrator contract (source of truth)
- `.claude/docs/coordination-rules.md` — disposition label spec (`NEW / CLEAR / REPONDRE / RELANCER`)
- `.claude/docs/orchestrator-paralelisme-optimisation.md` — wave parallelism context
