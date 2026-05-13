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
| `PROMPT N -- Title` | **SPAWN** worker `PROMPT-N` under `gcs-orchestrator`; the worker's `initialPrompt` is `PROMPT N -- Title\n\n<body>` (the original header is re-prepended so the worker still sees its own opening line; the parser strips it only for routing). **Dedup**: skipped with `DEDUP_SKIP` if `PROMPT-N` already exists in a `running/live/starting` state. No `parentTerminalId` is set — workers appear top-level under the tentacle in the UI graph, which is consistent with "external orchestrator drives". | `POST /api/terminals` | `oui-messire.mp3` |
| `NEW -- PROMPT N` | **NO-OP** (disposition label only). Recognised so it cleanly delimits preceding/following blocks; the real SPAWN trigger is the `PROMPT N -- Title` line that follows. Fallback: if no matching `PROMPT N -- Title` appears in the same response, an empty placeholder terminal is spawned **unless** `PROMPT-N` is already alive (then `DEDUP_SKIP`). | (no call unless fallback) | (silent) |
| `CLEAR -- PROMPT N` | **KILL** worker `PROMPT-N`. Logs `NOOP CLEAR` if the terminal does not exist in Octogent. | `POST /api/terminals/PROMPT-N/kill` | `travail-termine.mp3` |
| `REPONDRE -- PROMPT N` | **Channel-message** the existing worker `PROMPT-N` with the body. Logs `WARN REPONDRE` if no such terminal — message still POSTed (Octogent will reply 404, audible only in dispatch log). | `POST /api/channels/PROMPT-N/messages` | `pret-a-travailler.mp3` |
| `RELANCER -- PROMPT N` | **Bypasses dedup.** DELETEs the existing `PROMPT-N` record (regardless of state), then recreates with new body, preserving the same terminalId. Without the DELETE step Octogent silently auto-assigns a new id like `terminal-3`, which would break subsequent `CLEAR/REPONDRE -- PROMPT N` targeting. | `POST .../kill` → `DELETE /api/terminals/PROMPT-N` → `POST /api/terminals` | `encore-du-travail.mp3` |

### Dedup semantics

Before every SPAWN the dispatcher takes a snapshot of Octogent's terminal list.
For each spawn block it then checks whether `PROMPT-N` is in `{running, live,
starting}`:

- **Yes** → `DEDUP_SKIP`, no API call, no sound. The orchestrator probably
  re-emitted a prompt it already launched (e.g. the same PROMPT-N appearing
  in two consecutive turns). The log message tells the user to switch to
  `RELANCER -- PROMPT N` if they really want to overwrite.
- **No** → spawn proceeds. The local snapshot is updated so that later blocks
  in the same response see the freshly-spawned ID and dedup correctly too.

`RELANCER -- PROMPT N` is the deliberate escape hatch when you do want to
replace an alive worker — it skips the dedup check entirely.

The snapshot is loaded once per dispatch run (one HTTP GET). Cost: <50 ms.

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
- `~/.codex/gcs-octogent-dispatch.py` — the dispatcher (regex + HTTP + sounds + dedup)
- `~/.codex/gcs-codex-stop-hook.py` — the Codex Stop hook entry point
- `~/.codex/hooks.json` — Codex hook config (2 Stop hooks: sound + dispatcher)
- `~/.codex/gcs-dispatch.log` — verbose append-only action log (every HTTP call, every parsed block, full tracebacks on crash). Use for debug.
- `~/.codex/gcs-dispatch-summary.log` — condensed one-line-per-dispatch counter log. Use for at-a-glance flow monitoring (`tail -F`).
- `~/.codex/gcs-stop-hook.log` — append-only hook-fire log (every Codex Stop event with dispatcher exit code + truncated stderr).
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

### At-a-glance: the summary log

`tail -F ~/.codex/gcs-dispatch-summary.log` gives one line per dispatch run.
Format:

```
[YYYY-MM-DD HH:MM:SS] in=<chars>c blocks=<n> SPAWN=X DEDUP_SKIP=Y CLEAR=Z REPONDRE=W RELANCER=V NEW=U FALLBACK_SPAWN=F ERROR=E
```

Only non-zero counters appear. Empty dispatch = `no-ops`. Example session:

```
[2026-05-13 13:44:58] in=40c   blocks=1 SPAWN=1
[2026-05-13 13:44:59] in=62c   blocks=1 DEDUP_SKIP=1
[2026-05-13 13:45:01] in=64c   blocks=1 RELANCER=1
[2026-05-13 13:45:02] in=20c   blocks=1 CLEAR=1
[2026-05-13 13:46:30] in=11933c blocks=3 SPAWN=1 CLEAR=1 REPONDRE=1
```

For full HTTP detail of a specific dispatch, cross-reference the timestamp
with `~/.codex/gcs-dispatch.log`.

### Windows toast notifications

Each dispatch that produced at least one meaningful counter (anything other
than `NEW` alone or pure `no-ops`) also fires a non-blocking Windows toast
in the top-right notification area. Example body:

```
GCS Octogent · dispatch
SPAWN=1 · CLEAR=1  (PROMPT-813, PROMPT-812)
```

The toast lists up to 3 affected PROMPT-N ids, prioritised by importance
(spawn/relance > clear > respond > skip > new). Non-blocking — PowerShell is
spawned in a hidden window and the dispatcher continues immediately.

**Disable toasts**: set `GCS_DISPATCH_TOAST=0` in the env where Codex runs.
Equivalent values: `0`, `false`, `False`, empty string. Anything else keeps
toasts on.

**No toast on**:
- A dispatch with no recognised blocks (no-op).
- A dispatch with only `NEW -- PROMPT N` markers and nothing else (label-only
  output; the real spawn would be in a later dispatch).

### Symptom table

| Symptom | First check |
|---|---|
| Block emitted but no terminal appears | `tail -30 ~/.codex/gcs-dispatch.log` — is the block detected? Is the POST returning 201? Look for `DEDUP_SKIP` (block was a duplicate). |
| Same `PROMPT N` repeatedly skipped | This is correct dedup. If you really want to overwrite, the orchestrator must emit `RELANCER -- PROMPT N`. |
| `RELANCER` produced a `terminal-3` instead of reusing `PROMPT-N` | You're on an older dispatcher. Make sure RELANCER does `kill → DELETE → POST`, not just `stop → POST`. |
| Hook not firing | `tail -10 ~/.codex/gcs-stop-hook.log` — should show `=== stop hook fired ===` on every Codex turn. If empty, the Codex session predates the hook trust — restart Codex (see §6). |
| Hook fires but dispatcher silent | The hook found no rollout or the last assistant message is empty — check `~/.codex/sessions/.../rollout-*.jsonl` exists and the latest assistant `output_text` isn't an empty string. |
| Dispatcher crashes with traceback | Full traceback is captured in `~/.codex/gcs-stop-hook.log` (up to 4 KiB). The dispatcher itself never propagates exceptions, but the inbox backup or summary write can fail on weird inputs — those errors are logged and dispatch continues. |
| Wrong terminal got killed | `terminalId` collision — the dispatcher uses `PROMPT-<N>` so two waves reusing the same N collide. Always use monotonically-increasing N (the orchestrator contract guarantees this). |
| Vite crashes with `0xC0000409` | You set `OCTOGENT_WORKSPACE_CWD`; remove it from the launcher. Use the junction approach instead. |
| `pnpm.exe ENOENT` | `dev.mjs` not patched (step 3 above). Change `pnpm.exe` → `pnpm.cmd`, add `shell: true`. |
| `git worktree remove` fails on DELETE | Use `POST /api/terminals/prune` instead of the UI delete button. |
| API on 8787 unreachable | `netstat -ano \| grep ":8787"` — is anything listening? If not, Octogent is dead — relaunch from Explorer right-click. |
| 2 mystery `node.exe` running, no port bound | `pnpm dev` parent + child orphan from a crashed launch; `taskkill /F /PID <pid>` is safe. |
| Sound silent | The Claude/Codex sound hooks rely on `~/.claude/sounds/play-sound.ps1` and the 4 mp3 files being present. |
| `UserPromptSubmit hook (failed)` in Codex | Pre-existing user sound hook with a trailing `&`; harmless noise, the sound still plays. Not related to the dispatcher. |
| Toast not appearing | Check `GCS_DISPATCH_TOAST=0` is not set in the Codex environment. Also Windows Focus Assist (DND mode) suppresses toasts; check Settings → System → Notifications. To force-test: `printf 'PROMPT 9999 -- Test\nbody\n' \| python ~/.codex/gcs-octogent-dispatch.py`. |

## 8. Caveats and known issues

1. **PTY non-persistent**: if Octogent crashes mid-wave, every live worker dies. The `inbox/<ts>.md` backup of each orchestrator response gives a manual recovery path.
2. **Channels are in-memory**: messages queued via `octogent channel send` are lost on Octogent restart. The dispatcher only does spawn/kill/replace and `POST /api/channels/.../messages`, so a restart wipes pending REPONDRE deliveries that hadn't been injected yet.
3. **9-children-per-parent limit**: hard cap inside Octogent. Current waves of 4–7 are well under. For 12+ lanes, chain a sub-parent.
4. **Stop hook fires for every Codex session**: non-orchestrator sessions (research, dev, etc.) trigger the dispatcher too. Their responses contain no actionable headers, so the dispatcher logs `no actionable blocks found` and exits silently. Safe but watch the log noise.
5. **Per-developer setup**: nothing is shared. Each dev runs their own Octogent instance on their own machine.
6. **Octogent's own dev tentacles** (`api-runtime`, `web-ui`, etc.) are visible because the junction lives under the install dir. They're inert noise — click **HIDE IDLE** in the toolbar to mute them.
7. **Patched `dev.mjs` lost on Octogent upgrade**: re-apply the two-line patch after `git pull` on the Octogent clone.
8. **Codex loads hooks at session start**: if you add or modify hooks while a Codex orchestrator session is already running, the new hooks are NOT attached to that session. Restart Codex (`codex` for a fresh session, or `codex resume <id>` to pick up the existing rollout with the new hooks bound).
9. **`POST /api/terminals` with an existing terminalId**: Octogent silently auto-assigns a new id (e.g. `terminal-3`) rather than reusing or rejecting. The dispatcher works around this for `RELANCER` by DELETing the registry record first. For SPAWN it would have produced a duplicate worker with a different id, which is why dedup is enforced.

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
