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

## 2. Architecture — closed loop, observer + reverse channel

```
┌──────────────────────────────────────┐
│  Codex Orchestrator (UNCHANGED)      │   ← keeps its existing contract
│  hosted in an Octogent terminal      │     (production/session-state/
│  named `codex-orchestrator-main`     │      codex-orchestrator-state.md)
│                                      │
│  emits text blocks in conversation:  │
│    NEW -- PROMPT N                   │
│    PROMPT N -- Short Task Name       │
│    body...                           │
│    CLEAR / REPONDRE / RELANCER       │
└──────────────┬───────────────────────┘
               │ Codex writes assistant turn to
               │ ~/.codex/sessions/.../rollout-<uuid>.jsonl
               │
               ▼ Codex Stop hook fires at end of every assistant turn
┌──────────────────────────────────────┐
│  ~/.codex/gcs-codex-stop-hook.py     │
│   • opens latest rollout (mtime)     │
│   • extracts last assistant text     │
│   • mirrors codex-orchestrator-      │
│     state.md → CONTEXT.md            │
│   • pipes text to dispatcher         │
└──────────────┬───────────────────────┘
               │ stdin
               ▼
┌──────────────────────────────────────┐
│  ~/.codex/gcs-octogent-dispatch.py   │
│   • regex-splits text into blocks    │
│   • per block: HTTP POST + sound     │
│   • appends report-back curl         │
│     instruction to every spawned     │
│     worker's initialPrompt           │
│   • writes inbox/<ts>.md backup      │
└──────────────┬───────────────────────┘
               │ HTTP (loopback only)
               ▼
        http://127.0.0.1:8787
   (Octogent API + Web UI on 5173)
               │
               ▼ spawn / kill / channel-send
   Workers `PROMPT-N` appear under the
   `gcs-orchestrator` tentacle in the UI
               │
               ▼ each worker's last action before ending its turn
   `curl POST /api/channels/codex-orchestrator-main/messages`
   with content "DONE PROMPT-N: <status line>"
               │
               ▼ Octogent queues the message in-memory until target is idle,
               │ then delivers it as a bracketed-paste block + `\r`
               │ (auto-submit). Patched into channelMessaging.ts — without
               │ this, the message would land in the orchestrator's input
               │ buffer unsubmitted and require a manual Enter.
               ▼
   Orchestrator's PTY receives `[Channel message from PROMPT-N]: DONE...`
   as user-typed input → Codex processes it → orchestrator decides
   CLEAR / REPONDRE / RELANCER → dispatcher executes → loop continues
```

**Key invariant**: the orchestrator's textual contract is unchanged. Stop the
dispatcher, kill Octogent, delete the hook — orchestrator behavior is
identical, the integration is a pure sidecar. The reverse channel (worker →
orchestrator) is opt-in per worker via the report-back instruction the
dispatcher appends to every spawned `initialPrompt`. Workers from before that
instruction was added must report manually (paste a curl line per #6).

## 3. The five header patterns

The dispatcher recognises exactly these line-starts (case-sensitive, anchored
to start of line, `\d+` is the prompt number):

| Pattern | Action | Octogent API call | Sound |
|---|---|---|---|
| `PROMPT N -- Title` | **SPAWN** worker `PROMPT-N` under `gcs-orchestrator`; the worker's `initialPrompt` is `PROMPT N -- Title\n\n<body>` (the original header is re-prepended so the worker still sees its own opening line; the parser strips it only for routing). **Dedup**: skipped with `DEDUP_SKIP` if `PROMPT-N` already exists in a `running/live/starting` state. No `parentTerminalId` is set — workers appear top-level under the tentacle in the UI graph, which is consistent with "external orchestrator drives". | `POST /api/terminals` | `oui-messire.mp3` |
| `NEW -- PROMPT N` | **NO-OP** (disposition label only). Recognised so it cleanly delimits preceding/following blocks; the real SPAWN trigger is the `PROMPT N -- Title` line that follows. Fallback: if no matching `PROMPT N -- Title` appears in the same response, an empty placeholder terminal is spawned **unless** `PROMPT-N` is already alive (then `DEDUP_SKIP`). | (no call unless fallback) | (silent) |
| `CLEAR -- PROMPT N` | **KILL + DELETE** worker `PROMPT-N` so it disappears from the registry and the UI (matches the orchestrator contract's "close the agent window"). Earlier behaviour was kill-only, which left a `stopped` record visible — now fully removed. Logs `NOOP CLEAR` if the terminal does not exist. | `POST /api/terminals/PROMPT-N/kill` then `DELETE /api/terminals/PROMPT-N` | `travail-termine.mp3` |
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

### Decorative prefixes are tolerated

The regex accepts up to **10 non-word, non-newline characters** before the
disposition keyword, so the orchestrator can prefix each label with
coloured emoji markers or list bullets without breaking the match. All of
these are recognised:

| Line | Match |
|---|---|
| `CLEAR -- PROMPT 771` | ✅ plain |
| `🟢 CLEAR -- PROMPT 771` | ✅ green-dot prefix |
| `🔺🔺🔺 PROMPT 763 -- Title` | ✅ multi-emoji prefix |
| `- CLEAR -- PROMPT 100` | ✅ list bullet |
| `> NEW -- PROMPT 200` | ✅ blockquote |
| `   CLEAR -- PROMPT 999` | ✅ leading whitespace |

### Things that still do NOT match

| Line | Why ignored |
|---|---|
| `Prompt 813:` | Lowercase `p` |
| `Nouveau prompt #813:` | Wrong wording |
| `PROMPT 813 - Title` | Single dash (regex requires `--`) |
| `Voici le prompt à lancer:` | No header at all |
| `NEW: PROMPT 813` | Wrong separator (colon, not `--`) |
| `Just CLEAR -- PROMPT 813` | Word chars (`Just`) precede the keyword |
| `Some text CLEAR -- PROMPT 813` | Word chars precede the keyword |

The "word chars before keyword" anti-match is intentional: it prevents
the regex from grabbing dispositions mentioned inside prose paragraphs
(e.g. `the worker emitted CLEAR -- PROMPT 813 yesterday`).

If the orchestrator's wording drifts further (e.g. lowercase keywords,
different separator, etc.), update `_HEADER_RX` in
`~/.codex/gcs-octogent-dispatch.py`.

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

#### Octogent install + 3 source patches

- `D:\_APPS\Tools\octogent\` — the cloned Octogent source (Node 22+ / pnpm-managed)

Three local patches to Octogent's source — **all lost on `git pull` of Octogent**, re-apply if you upgrade:

1. `D:\_APPS\Tools\octogent\scripts\dev.mjs` — use `pnpm.cmd` (not `pnpm.exe`, which doesn't exist with `npm install -g pnpm` on Windows) + add `shell: process.platform === "win32"` to the spawn call (Node 24 requirement). Adds three `console.log` debug lines for `OCTOGENT_WORKSPACE_CWD` / `workspaceCwd` / `projectStateDir` so launches are self-diagnosing.
2. `D:\_APPS\Tools\octogent\apps\api\src\terminalRuntime\channelMessaging.ts` — three changes in `deliverChannelMessages`:
   - Wraps injected messages in **bracketed-paste markers** (`\x1b[200~` … `\x1b[201~`) and writes `\r` via a 150 ms `setTimeout` to trigger auto-submit; without this, channel-send to a Codex agent lands the text in the input buffer unsubmitted.
   - Delivers **one message at a time** (`undelivered[0]`) instead of batching all queued messages into a single combined injection — keeps per-worker reports as separate orchestrator turns.
   - Per-session `inFlightDeliveries` lock covering the 150 ms paste→`\r` window — without this lock, a second message arriving during the window short-circuits the idle check and stacks on top of the first.
3. `D:\_APPS\Tools\octogent\apps\api\src\terminalRuntime\sessionRuntime.ts` — `INITIAL_PROMPT_SUBMIT_DELAY_MS` bumped from `150` to `500` ms. The original 150 ms was reliable for a single spawn but flaky when two Codex PTYs bootstrapped close together (initial-prompt `\r` arrived before Codex finished consuming the paste, leaving the worker stuck on `[Pasted Content X chars]` waiting for a manual Enter). 500 ms gives the runtime breathing room under contention; the +350 ms latency cost is once per spawn.

Launcher chain (not patches, project files maintained alongside the install dir):

- `D:\_APPS\Tools\octogent\launch-here.bat` — invoked by the right-click context menu. Reads `%~1` as project dir, then `start "Octogent - X" cmd /k call launch-inner.bat "<dir>"` to spawn a detached persistent cmd window.
- `D:\_APPS\Tools\octogent\launch-inner.bat` — runs in the spawned cmd. Clears `PWD/OLDPWD/INIT_CWD` (bash artifacts that leak when launched from MSYS), sets `OCTOGENT_WORKSPACE_CWD=<projectDir>`, then `cd /d D:\_APPS\Tools\octogent && pnpm dev`. **NO `setlocal`** (interferes with env propagation through pnpm) and **NO `pnpm --dir`** (pnpm strips `OCTOGENT_*` env vars when invoked with `--dir`).
- `D:\_APPS\Tools\octogent\install-context-menu.reg` + `uninstall-context-menu.reg` — UTF-16 LE BOM `.reg` files for the Explorer right-click. Currently installed under `HKCU\Software\Classes\Directory\{Background\,}shell\Octogent`.

The launch chain sets `OCTOGENT_WORKSPACE_CWD` correctly, so Octogent natively binds the per-folder workspace: state lives in `<projectDir>/.octogent/state/`, tentacles are read from `<projectDir>/.octogent/tentacles/`, and a different "Open Octogent here" from a different folder picks up that project's own state. The legacy install-dir junction approach is **no longer needed** and can be removed.

#### Sound assets
- `~/.claude/sounds/play-sound.ps1` — PowerShell MediaPlayer wrapper (pre-existing)
- `~/.claude/sounds/oui-messire.mp3` — PROMPT spawn
- `~/.claude/sounds/pret-a-travailler.mp3` — REPONDRE
- `~/.claude/sounds/travail-termine.mp3` — CLEAR (also bound to base Codex/Claude Stop)
- `~/.claude/sounds/encore-du-travail.mp3` — RELANCER (also Notification permission)

## 5. First-time setup for a new dev machine

Prerequisites: Node 22+, git, gh, curl, Python 3.10+. PowerShell + cmd. Codex CLI ≥0.130, Claude Code ≥2.1.

1. Clone Octogent: `git clone https://github.com/hesamsheikh/octogent D:\_APPS\Tools\octogent`
2. Install pnpm (user-prefix, no admin): `npm install -g pnpm`
3. Apply the three Octogent source patches (lost on `git pull`; keep a copy):
   - `scripts/dev.mjs`:
     - change `"pnpm.exe"` to `"pnpm.cmd"` (~line 74)
     - add `shell: process.platform === "win32",` inside the spawn options object (~line 128)
   - `apps/api/src/terminalRuntime/channelMessaging.ts` `deliverChannelMessages`:
     - take `undelivered[0]` instead of all messages (one-at-a-time delivery)
     - wrap the chosen message in `BRACKETED_PASTE_START="\x1b[200~"` / `BRACKETED_PASTE_END="\x1b[201~"`, write it, then `setTimeout(() => writeInput(terminalId, "\r"), 150)`
     - declare a `const inFlightDeliveries = new Set<string>()` at the closure level, set it at the start of `deliverChannelMessages`, clear it inside the `\r` setTimeout — this is the per-session lock
   - `apps/api/src/terminalRuntime/sessionRuntime.ts`:
     - change `const INITIAL_PROMPT_SUBMIT_DELAY_MS = 150;` to `500` (~line 440)
4. Copy the dispatcher + hook scripts (or grab them from a teammate's backup):
   - `~/.codex/gcs-octogent-dispatch.py`
   - `~/.codex/gcs-codex-stop-hook.py`
5. Add the Codex Stop hook to `~/.codex/hooks.json` (under `hooks.Stop[0].hooks`):
   ```json
   { "type": "command", "command": "python C:/Users/Sam/.codex/gcs-codex-stop-hook.py" }
   ```
6. Trust the new hook on next Codex launch (press `t` in the hook review screen).
7. Install the right-click context menu:
   - Place `launch-here.bat`, `launch-inner.bat`, `install-context-menu.reg`, `uninstall-context-menu.reg` in `D:\_APPS\Tools\octogent\` (already there after step 1).
   - Run `reg import D:\_APPS\Tools\octogent\install-context-menu.reg` (or double-click; the `.reg` must be UTF-16 LE with BOM).
8. In this repo, scaffold the tentacle (paths use forward slashes; `cmd` accepts them):
   ```
   mkdir .octogent\tentacles\gcs-orchestrator\inbox
   copy production\session-state\codex-orchestrator-state.md .octogent\tentacles\gcs-orchestrator\CONTEXT.md
   echo # GCS Orchestrator Todo > .octogent\tentacles\gcs-orchestrator\todo.md
   echo # Prompt Log > .octogent\tentacles\gcs-orchestrator\prompt-log.md
   ```
9. Launch: right-click on the repo folder in Explorer → "Show more options" → "Open Octogent here". The cmd banner must show `OCTOGENT_WORKSPACE_CWD=<your project path>`. UI opens on `http://localhost:5173`, only the project's own tentacles are visible.
10. Spawn the orchestrator host terminal once, then resume the Codex session inside it:
    ```bash
    # Replace <project> with your project root path.
    curl -X POST http://127.0.0.1:8787/api/terminals \
      -H 'Content-Type: application/json' \
      -d '{"terminalId":"codex-orchestrator-main","name":"Codex Orchestrator Terminal","tentacleId":"gcs-orchestrator","workspaceMode":"shared"}'
    # Then in the Octogent UI, click on "Codex Orchestrator Terminal" and run:
    # codex resume <your-session-uuid>
    ```

## 6. Daily use

1. Make sure Octogent is running: right-click on the repo folder → "Open Octogent here". The cmd banner shows `OCTOGENT_WORKSPACE_CWD=<your project>` then `Octogent API listening on http://127.0.0.1:8787` and `Local: http://localhost:5173/`.
2. Open `http://localhost:5173/` in a browser. The `Codex Orchestrator State` tentacle and the `Codex Orchestrator Terminal` host terminal should appear.
3. Attach to the host terminal in the UI, run `codex resume <session-uuid>` once if you haven't already. **For Claude Code launches** anywhere on this machine, prefer `claude-safe` over `claude` — it auto-fixes `~/.claude.json` if a concurrent worker write left it corrupted, then launches `claude` normally. See `~/.codex/claude-safe.py` for the fixer and `~/AppData/Roaming/npm/claude-safe.cmd` for the wrapper on PATH.
4. Work with the Codex orchestrator normally. On each turn the dispatcher detects its disposition blocks, spawns/kills/messages workers in Octogent, and workers report back to the orchestrator via channel-send (auto-submitted thanks to the bracketed-paste patch).
5. To inspect any worker: click it in the UI to see its transcript and channel messages.
6. **Manual report-back fallback** (for workers spawned before the dispatcher's auto-instruction, or if a worker forgets). Three steps — the worker (or you posing as it) must do all three:
   1. Worker writes its full report as its normal assistant message in chat (readable Markdown — for the human watching the worker UI).
   2. Worker mirrors the same content to `reports/PROMPT-N.md` in the project root so the orchestrator can read it.
   3. Worker sends a single-line channel message announcing completion:
      ```bash
      curl -s -X POST http://127.0.0.1:8787/api/channels/codex-orchestrator-main/messages \
        -H 'Content-Type: application/json' \
        --data-raw '{"fromTerminalId":"PROMPT-N","content":"DONE PROMPT-N: N: TICKET-ID: STATUS // full report at reports/PROMPT-N.md"}'
      ```
   The single-line constraint is critical: multi-line channel content triggers Codex's paste-mode, the message lands in the orchestrator's input as `[Pasted Content N chars]`, and the `\r` auto-submit fails. Single-line short content reliably auto-submits.
7. To force a manual dispatch action without going through the orchestrator (debug):
   ```bash
   echo "CLEAR -- PROMPT 999" | python ~/.codex/gcs-octogent-dispatch.py
   ```
8. To stop Octogent cleanly: Ctrl+C in the cmd window. Don't click the X — leaves orphan node processes.

### 6.1 Multi-project workflow

Because `OCTOGENT_WORKSPACE_CWD` is bound per launch, "Open Octogent here" from a *different* project folder spawns Octogent against *that* project — its state, its tentacles, its terminals are isolated under `<thatProject>/.octogent/state/`. Switching projects = Ctrl+C the running Octogent, right-click "Open Octogent here" from the new folder. **One Octogent instance at a time on port 8787**; for two projects simultaneously, set `OCTOGENT_DEV_START_PORT=<other>` in the env before launch.

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
| Vite crashes with `0xC0000409` | Almost certainly a quoting bug in your `set "OCTOGENT_WORKSPACE_CWD=..."` line, not Vite itself. Make sure `launch-inner.bat` has `set "VAR=VAL"` (mandatory quotes around the whole assignment) and **no** `setlocal`. |
| Channel-send message stuck in orchestrator's input bar (not submitted) | Two possible causes: (a) `channelMessaging.ts` not patched — re-apply the bracketed-paste + delayed `\r` patch (see §4 / §5 step 3); (b) the channel message content has internal newlines (multi-line), which trips Codex paste-mode — keep the content single-line and put the multi-line body in a `reports/<id>.md` file instead (see §6 step 6, the 3-step protocol). |
| Worker spawns but never reports back | Worker's `initialPrompt` is missing the 3-step report-back instruction — either spawned before the dispatcher started appending it, or the worker ignored a step. Paste the manual 3-step sequence (§6 step 6). |
| Channel-send arrives as `[Pasted Content X chars]` and never executes | The content was multi-line — Codex treats it as paste. The Step 3 channel message must be ONE line. The full multi-line body lives in the file from Step 2. |
| `pnpm.exe ENOENT` | `dev.mjs` not patched (step 3 above). Change `pnpm.exe` → `pnpm.cmd`, add `shell: true`. |
| `OCTOGENT_WORKSPACE_CWD=(unset)` in dev.mjs debug log despite the env var being set | The cmd shell that runs `pnpm dev` was using `pnpm --dir X` instead of `cd X && pnpm dev`. pnpm strips `OCTOGENT_*` env vars when invoked with `--dir`. `launch-inner.bat` does it the right way. |
| `git worktree remove` fails on DELETE | Use `POST /api/terminals/prune` instead of the UI delete button. |
| API on 8787 unreachable | `netstat -ano \| grep ":8787"` — is anything listening? If not, Octogent is dead — relaunch from Explorer right-click. |
| 2 mystery `node.exe` running, no port bound | `pnpm dev` parent + child orphan from a crashed launch; `taskkill /F /PID <pid>` is safe. |
| Sound silent | The Claude/Codex sound hooks rely on `~/.claude/sounds/play-sound.ps1` and the 4 mp3 files being present. |
| `UserPromptSubmit hook (failed)` in Codex | Pre-existing user sound hook with a trailing `&`; harmless noise, the sound still plays. Not related to the dispatcher. |
| Toast not appearing | Check `GCS_DISPATCH_TOAST=0` is not set in the Codex environment. Also Windows Focus Assist (DND mode) suppresses toasts; check Settings → System → Notifications. To force-test: `printf 'PROMPT 9999 -- Test\nbody\n' \| python ~/.codex/gcs-octogent-dispatch.py`. |

## 8. Caveats and known issues

1. **PTY non-persistent**: if Octogent crashes mid-wave, every live worker dies. The `inbox/<ts>.md` backup of each orchestrator response gives a manual recovery path.
2. **Channels are in-memory**: messages queued via `POST /api/channels/.../messages` are lost on Octogent restart. Worker reports queued mid-flight at restart time will not be delivered.
3. **9-children-per-parent limit**: hard cap inside Octogent. Current waves of 4–7 are well under. For 12+ lanes, chain a sub-parent.
4. **Stop hook fires for every Codex session**: non-orchestrator sessions (research, dev, etc.) trigger the dispatcher too. Their responses contain no actionable headers, so the dispatcher logs `no actionable blocks found` and exits silently. Safe but watch the log noise.
5. **Per-developer setup**: nothing is shared. Each dev runs their own Octogent instance on their own machine.
6. **Octogent's own dev tentacles** (`api-runtime`, `web-ui`, etc.) are visible only when `OCTOGENT_WORKSPACE_CWD` is *not* set (Octogent falls back to its install dir and reads its own tentacles). With our launcher correctly setting `OCTOGENT_WORKSPACE_CWD` per project, only the project's tentacles appear — those install-dir ones are hidden.
7. **Two Octogent source patches** (`scripts/dev.mjs`, `apps/api/src/terminalRuntime/channelMessaging.ts`) are lost on `git pull` of the Octogent clone. Keep a copy of the patched files alongside this repo or re-apply manually.
8. **Codex loads hooks at session start**: if you add or modify hooks while a Codex orchestrator session is already running, the new hooks are NOT attached to that session. Restart Codex (`codex` for a fresh session, or `codex resume <id>` to pick up the existing rollout with the new hooks bound).
9. **`POST /api/terminals` with an existing terminalId**: Octogent silently auto-assigns a new id (e.g. `terminal-3`) rather than reusing or rejecting. The dispatcher works around this for `RELANCER` by DELETing the registry record first. For SPAWN it would have produced a duplicate worker with a different id, which is why dedup is enforced.
10. **Workers must follow the 3-step report-back instruction**: the dispatcher appends a 3-step protocol to every spawned worker's `initialPrompt` — (1) display full report in chat, (2) mirror to `reports/<terminalId>.md`, (3) send short single-line channel notification. A worker can in theory skip a step (forget to write the file, send multi-line content, etc.). If a worker doesn't notify, the orchestrator stays blocked on it — paste the manual sequence (§6 step 6) to recover.
11. **Codex paste-mode threshold**: Codex's TUI input treats bracketed-paste content with internal newlines as a pasted block, displays it as `[Pasted Content N chars]`, and **does not auto-submit on the trailing `\r`**. Single-line content auto-submits cleanly even at 2 KB+. This is why Step 3 of the worker protocol is a short single-line channel message — the full multi-line report goes via the file (Step 2) instead.
12. **Channel deliveries are serialised per target**: Octogent's `channelMessaging.deliverChannelMessages` previously batched all undelivered messages for an idle target into a single combined injection — two worker reports arriving in quick succession ended up in one orchestrator turn, blurring per-worker decisions. Patched: one message at a time, locked by a per-session in-flight flag covering the 150 ms between bracketed-paste write and the trailing `\r`. Subsequent messages stay queued until the orchestrator processes the current one and the next idle hook fires.
11. **`pnpm --dir` strips env vars**: a confirmed Windows/pnpm quirk — `pnpm --dir X dev` clears `OCTOGENT_*` env vars in the spawned dev process. The launcher uses `cd /d X && pnpm dev` instead. Don't switch back to `--dir`.

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
