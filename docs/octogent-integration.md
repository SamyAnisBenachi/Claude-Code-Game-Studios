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
| `RELANCER -- PROMPT N` | **Bypasses dedup.** DELETEs the existing `PROMPT-N` record (regardless of state), then recreates with new body, preserving the same terminalId. Without the DELETE step Octogent silently auto-assigns a new id like `terminal-3`, which would break subsequent `CLEAR/REPONDRE -- PROMPT N` targeting. **Pairing**: if a `PROMPT N -- Title` block follows in the same response (orchestrator pattern: `🔴 RELANCER -- PROMPT N` header + `PROMPT N -- Title` + full body), the RELANCER line is treated as a no-op marker and the PROMPT block does the kill+DELETE+spawn with the real body — avoids a wasteful intermediate spawn-with-empty-body. Pre-scan computes `prompt_ns ∩ relancer_ns` once per dispatch. | `POST .../kill` → `DELETE /api/terminals/PROMPT-N` → `POST /api/terminals` | `encore-du-travail.mp3` |

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
- `~/.codex/gcs-octogent-dispatch.py` — the dispatcher (regex + HTTP + sounds + dedup); also launches the spawn-watchdog per SPAWN/RELANCER
- `~/.codex/gcs-spawn-watchdog.py` — per-spawn watchdog (the orchestrator → worker symmetric of the worker → orchestrator watchdog). Polls `/api/terminal-snapshots` every 10 s for the spawned terminal; on `initialPromptSubmitFailed: true` it kill+DELETE+POSTs the spawn again with the original initialPrompt (read from `reports/.watchdog/spawn/<terminalId>.json`). Exits cleanly on `initialPromptSubmittedAt` set or terminal-disappeared. Cap `MAX_RESPAWNS=5`, `MAX_OCTOGENT_DOWN_TICKS=360` (≈1 h tolerance for Octogent restarts). Detached/no-window on Windows so the dispatcher returns immediately after launching it. Logs to `~/.codex/gcs-spawn-watchdog.log`
- `~/.codex/gcs-codex-stop-hook.py` — the Codex Stop hook entry point
- `~/.codex/hooks.json` — Codex hook config (2 Stop hooks: sound + dispatcher)
- `~/.codex/gcs-dispatch.log` — verbose append-only action log (every HTTP call, every parsed block, full tracebacks on crash). Use for debug.
- `~/.codex/gcs-dispatch-summary.log` — condensed one-line-per-dispatch counter log. Use for at-a-glance flow monitoring (`tail -F`).
- `~/.codex/gcs-stop-hook.log` — append-only hook-fire log (every Codex Stop event with dispatcher exit code + truncated stderr).
- `~/.codex/sessions/.../rollout-*.jsonl` — Codex's own rollout (read-only source for the hook)

#### Claude Code side
- `~/.claude/settings.json` — adds Octogent `/api/hooks/stop` and `/api/hooks/notification` callbacks (so Claude Code workers spawned inside Octogent terminals get idle-gated)

#### Octogent install + 4 source patches

- `D:\_APPS\Tools\octogent\` — the cloned Octogent source (Node 22+ / pnpm-managed)

Four local patches to Octogent's source — **all lost on `git pull` of Octogent**, re-apply if you upgrade:

1. `D:\_APPS\Tools\octogent\scripts\dev.mjs` — use `pnpm.cmd` (not `pnpm.exe`, which doesn't exist with `npm install -g pnpm` on Windows) + add `shell: process.platform === "win32"` to the spawn call (Node 24 requirement). Adds three `console.log` debug lines for `OCTOGENT_WORKSPACE_CWD` / `workspaceCwd` / `projectStateDir` so launches are self-diagnosing.
2. `D:\_APPS\Tools\octogent\apps\api\src\terminalRuntime\channelMessaging.ts` + `D:\_APPS\Tools\octogent\packages\core\src\domain\channel.ts` — four changes in `deliverChannelMessages` plus two new optional fields on the `ChannelMessage` type:
   - Wraps injected messages in **bracketed-paste markers** (`\x1b[200~` … `\x1b[201~`) and writes `\r` via a 200 ms `setTimeout` to trigger auto-submit; without this, channel-send to a Codex agent lands the text in the input buffer unsubmitted.
   - Delivers **one message at a time** (`undelivered[0]`) instead of batching all queued messages into a single combined injection — keeps per-worker reports as separate orchestrator turns.
   - Per-session `inFlightDeliveries` lock covering the paste→`\r` window — without this lock, a second message arriving during the window short-circuits the idle check and stacks on top of the first.
   - **Submit-with-retry + outcome flag**: 1500 ms after each `\r`, re-checks `agentState`. If still `idle`, the `\r` didn't trigger Codex to submit (paste stuck in the input bar under PC/TTY lag — the keystroke was discarded before the END marker was consumed). Re-writes `\r`, up to 3 retries. Worst-case end-to-end ~4.7 s; success path (no lag) sees `agentState` go to `processing` after the first `\r` and the retries are no-ops. Uses Codex's `esc to interrupt` PTY marker (see `agentStateDetection.ts`) as the implicit "submit landed" signal. When the retry chain ends, sets one of two new outcome fields on the message itself: `submittedAt: string` (ISO timestamp, set when `agentState` had moved off `idle`) or `submitFailed: true` (set when all 3 retries finished with `agentState` still `idle`). Both fields are exposed through the existing `GET /api/channels/<id>/messages` endpoint and are what the worker-side L2 watchdog (§6 step 6.4) polls for self-healing resend decisions.
3. `D:\_APPS\Tools\octogent\apps\api\src\terminalRuntime\sessionRuntime.ts` + `D:\_APPS\Tools\octogent\apps\api\src\terminalRuntime\types.ts` + `D:\_APPS\Tools\octogent\packages\core\src\domain\terminal.ts` + `D:\_APPS\Tools\octogent\apps\api\src\terminalRuntime.ts` — initial-prompt submit-retry plus outcome flags on the persisted terminal:
   - `INITIAL_PROMPT_SUBMIT_DELAY_MS` bumped from `150` to `500` ms. The original 150 ms was reliable for a single spawn but flaky when two Codex PTYs bootstrapped close together (initial-prompt `\r` arrived before Codex finished consuming the paste, leaving the worker stuck on `[Pasted Content X chars]` waiting for a manual Enter). 500 ms gives the runtime breathing room under contention; the +350 ms latency cost is once per spawn.
   - **Submit-with-retry** on the initial prompt: same mechanism as channelMessaging — if `agentState` is still `idle` 1500 ms after each `\r`, retry up to 3 times. Catches the "spawn arrived but stuck in input bar" failure that the user reported on multiple Sprint 12 worker launches.
   - **Outcome flags on `PersistedTerminal`** (mirrored to `TerminalSnapshot`): at end of the retry chain, the runtime sets either `initialPromptSubmittedAt: string` (ISO timestamp — `agentState` had moved off `idle`) or `initialPromptSubmitFailed: true` (all 3 retries finished with `agentState` still `idle`). Both fields surface through `GET /api/terminal-snapshots` so the dispatcher-side per-spawn watchdog can decide whether to kill+respawn the worker. Symmetric to the channel-message `submittedAt` / `submitFailed` flags described in patch 2.
4. `D:\_APPS\Tools\octogent\apps\api\src\terminalRuntime.ts` — `onStateChange` callback triggers `deliverChannelMessages(sessionId)` whenever a session transitions to `idle`. Without this, Octogent only delivers queued channel messages on the Claude-Code-style `idle_prompt` notification, which Codex agents don't emit — so messages from workers reporting back to the orchestrator pile up `delivered:false` indefinitely (observed up to 16 stuck on one orchestrator before the patch). A `deliverChannelMessagesRef` forward-reference is declared above `createSessionRuntime(...)` and wired to `channelMessaging.deliverChannelMessages` after creation, so the runtime's idle transition can call into the messaging module's queue drainer.

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
3. Apply the four Octogent source patches (lost on `git pull`; keep a copy):
   - `scripts/dev.mjs`:
     - change `"pnpm.exe"` to `"pnpm.cmd"` (~line 74)
     - add `shell: process.platform === "win32",` inside the spawn options object (~line 128)
   - `apps/api/src/terminalRuntime/channelMessaging.ts` `deliverChannelMessages`:
     - take `undelivered[0]` instead of all messages (one-at-a-time delivery)
     - wrap the chosen message in `BRACKETED_PASTE_START="\x1b[200~"` / `BRACKETED_PASTE_END="\x1b[201~"`
     - declare a `const inFlightDeliveries = new Set<string>()` at the closure level, set it at the start of `deliverChannelMessages` (after the early-return guards), clear it when the submit-retry chain ends
     - replace the simple 150 ms `setTimeout(writeInput(\r))` with the submit-retry loop: write `\r`, then 1500 ms later check `sessions.get(terminalId)?.agentState === "idle"`; if so re-write `\r` up to 3 retries; otherwise clear `inFlightDeliveries`
   - `apps/api/src/terminalRuntime/sessionRuntime.ts`:
     - change `const INITIAL_PROMPT_SUBMIT_DELAY_MS = 150;` to `500` (~line 440)
     - wrap the initial-prompt `\r` write in the same submit-retry loop: `writeInitialSubmit` closure that writes `\r`, then 1500 ms later checks `session.agentState === "idle"`; if so re-fires itself up to 3 times
   - `apps/api/src/terminalRuntime.ts` (the outer one, not the folder):
     - declare `let deliverChannelMessagesRef: ((terminalId: string) => number) | undefined;` above `const sessionRuntime = createSessionRuntime({ ... })`
     - in `onStateChange`, add `if (state === "idle" && deliverChannelMessagesRef) deliverChannelMessagesRef(sessionId);` right after the existing `broadcastTerminalStateChanged(...)` call
     - after the `channelMessaging` object is created further down, add `deliverChannelMessagesRef = channelMessaging.deliverChannelMessages;` to wire the forward-reference
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
3. Attach to the host terminal in the UI, run `codex resume <session-uuid>` once if you haven't already.
4. Work with the Codex orchestrator normally. On each turn the dispatcher detects its disposition blocks, spawns/kills/messages workers in Octogent, and workers report back to the orchestrator via channel-send (auto-submitted thanks to the bracketed-paste patch).
5. To inspect any worker: click it in the UI to see its transcript and channel messages.
6. **Manual report-back fallback** (for workers spawned before the dispatcher's auto-instruction, or if a worker forgets). Four steps — the worker (or you posing as it) must do all four:
   1. Worker writes its full report as its normal assistant message in chat (readable Markdown — for the human watching the worker UI).
   2. Worker mirrors the same content to `reports/PROMPT-N-<task-slug>.md` in the project root so the orchestrator can read it. `<task-slug>` is the `Title` part of the worker's opening `PROMPT N -- Title` header, converted to a filesystem-safe slug (spaces/`/` → `-`, strip anything that's not alphanumeric/dash). Example: `reports/PROMPT-794-S11-DRAG-RUNTIME-RETEST-Story-Readiness.md`. The slug-in-filename makes reports greppable by ticket without opening each file. If a worker truly can't derive a slug, falling back to plain `reports/PROMPT-N.md` is acceptable — the orchestrator can still find it.
   3. Worker sends a single-line channel message announcing completion (path **must** match the file written in step 2):
      ```bash
      curl -s -X POST http://127.0.0.1:8787/api/channels/codex-orchestrator-main/messages \
        -H 'Content-Type: application/json' \
        --data-raw '{"fromTerminalId":"PROMPT-N","content":"DONE PROMPT-N: N: TICKET-ID: STATUS // full report at reports/PROMPT-N-<task-slug>.md"}'
      ```
   The single-line constraint is critical: multi-line channel content triggers Codex's paste-mode, the message lands in the orchestrator's input as `[Pasted Content N chars]`, and the `\r` auto-submit fails. Single-line short content reliably auto-submits.
   4. Worker spawns a **background watchdog** that polls Octogent every 10 s for the status of its own sent message and self-heals indefinitely. The watchdog inspects two new `ChannelMessage` fields exposed by Octogent (`submittedAt`, `submitFailed` — see §4 patch 2): if `submittedAt` is set the watchdog exits cleanly (orchestrator received and submitted); if `submitFailed` is set or the message has disappeared from the queue (Octogent crash/restart wiped it), the watchdog re-POSTs the Step 3 curl with a fresh `messageId`; otherwise it keeps polling. The watchdog process dies with the worker's PTY when the orchestrator CLEARs it, so there is no leak. Caps at `MAX_RESENDS=50` to avoid burning forever on a permanently-broken setup. The watchdog snippet is auto-injected into every dispatcher-spawned worker's `initialPrompt`; see `_report_back_instructions` in `~/.codex/gcs-octogent-dispatch.py` for the canonical Python loop, written to `reports/.watchdog/<terminalId>.py` and launched via `nohup … &`. This is the second layer of the L1 (Octogent paste-retry) + L2 (worker poll-and-resend) self-healing design.
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
| Emoji-prefixed dispositions (`🟢 CLEAR -- PROMPT N`) silently ignored by the dispatcher | The dispatcher's `sys.stdin` defaults to the Windows code page (cp1252), which mangles 4-byte UTF-8 emoji into double-encoded Latin letters like `ðŸŸ¢`. Those letters look like word chars to Python's `\w`, so the regex's "non-word prefix" rule fails. Fix: the dispatcher now calls `sys.stdin.reconfigure(encoding="utf-8")` at the top of `main()`. If you see garbage bytes (`ðŸŸ¢` etc.) in `inbox/<ts>.md` instead of the proper emoji, the patch is missing. |
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

## 9-bis. Codex orchestrator via `codex app-server` (2026-05-14)

The classic flow (Sections 2–6) routes worker → orchestrator DONE reports through Octogent's channel-send (HTTP POST → PTY input → Codex auto-submit on `\r`). This is unreliable for a **Codex CLI orchestrator**: Codex's TUI raw-input mode does not reliably auto-submit on `\r`/`\n`, so worker reports occasionally stack as `[Pasted Content N chars]` in the input bar requiring a manual Enter. (Claude Code workers do not have this issue — they have `idle_prompt` hooks Octogent uses natively.)

The 2026-05-14 migration replaces the orchestrator's runtime from `codex resume` (interactive TUI inside an Octogent terminal) to `codex app-server` (long-running JSON-RPC server), with two thin clients on top.

### Architecture

```
                ┌─────────────────────────────────────────┐
                │ codex app-server (long-running)         │
                │   --listen ws://127.0.0.1:9787          │
                │   thread 019dddb4-... (loaded once)     │
                └─────────────────────────────────────────┘
                          ▲           ▲           ▲
                          │JSON-RPC   │JSON-RPC   │JSON-RPC
        ┌─────────────────┘           │           └──────────────┐
        │                             │                          │
  ┌────────────────┐         ┌────────────────┐         ┌────────────────┐
  │ gcs-app-       │         │ User           │         │ Worker relay   │
  │ viewer.py      │         │ (you type into │         │ (via worker's  │
  │ (streams       │         │  the viewer)   │         │  Step 3 — see  │
  │  deltas)       │         │                │         │  gcs-mode=     │
  │                │         │ same client    │         │  relay branch  │
  │                │         │ as viewer      │         │  in dispatcher)│
  └────────────────┘         └────────────────┘         └────────────────┘
```

All three clients connect via the WebSocket transport on loopback (no auth needed for loopback per `codex app-server` defaults). The user's stdin to the viewer becomes a `turn/start` JSON-RPC call; workers do the same via `gcs-app-relay.py`. The app-server keeps the (89 MB) rollout loaded in memory after the first `thread/resume`, so subsequent client invocations are near-instant.

### Components

| File | Role |
|---|---|
| `C:/Users/Sam/.codex/gcs-app-viewer.py` | Interactive viewer + typer. Connects via WS, streams agent message deltas, sends user input as `turn/start`. Long-running, lives as long as the user wants to watch. |
| `C:/Users/Sam/.codex/gcs-app-relay.py` | Single-shot worker DONE relay. `python gcs-app-relay.py <session-id> -` reads stdin content and injects it as a user turn. File lock (`%LOCALAPPDATA%/gcs-app-relay/turn.lock`) + sha256 idempotency receipts + TCP keepalive. Auto-prunes receipts at >14 days or >500 files. |
| `C:/Users/Sam/.codex/gcs-octogent-dispatch.py` | Dispatcher with `_load_report_mode()` that reads `gcs-mode` and `gcs-orch-session-id` config files at every dispatch. Worker initialPrompt's Step 3 branches on the mode (curl channel-send vs python relay). |
| `C:/Users/Sam/.codex/gcs-mode` | Single-line config file. Contents `relay` activates app-server mode; missing / `channel-send` keeps legacy. **Atomic toggle**: `echo relay > C:/Users/Sam/.codex/gcs-mode` then next worker spawn uses relay. |
| `C:/Users/Sam/.codex/gcs-orch-session-id` | Single-line config file. Contents = the Codex thread/session UUID the relay will inject into. Must match the thread loaded in app-server. |

### Boot procedure (after PC reboot)

```cmd
:: 1. Start the long-running app-server in a dedicated cmd window
start "GCS app-server" cmd /k codex app-server --listen ws://127.0.0.1:9787

:: 2. Wait ~3 seconds, then launch the viewer in another window
start "GCS Viewer" cmd /k python C:/Users/Sam/.codex/gcs-app-viewer.py 019dddb4-95f7-79e1-b48c-fdfc34fa3cd8

:: 3. Verify Octogent is running for workers (separate, port 8787)
::    -- Right-click "Open Octogent here" on the project folder if needed
```

First `thread/resume` on the 89 MB rollout takes ~2–3 s; subsequent connects are sub-second (already loaded in app-server memory).

### Troubleshooting

| Symptom | Check | Recovery |
|---|---|---|
| Viewer prints `[DISCONNECTED from app-server: …]` | `curl http://127.0.0.1:9787/readyz` — if non-200, app-server is dead | Re-run the app-server cmd, then re-run the viewer (it auto-resumes the loaded session) |
| Worker exits with code 4 (`EXIT_APP_SERVER_ERROR`) | App-server unreachable | Same as above — restart the app-server cmd window |
| Worker exits with code 3 (`EXIT_LOCK_TIMEOUT`) | A previous relay holds the file lock | Check `%LOCALAPPDATA%/gcs-app-relay/turn.lock` content (PID + timestamp) — if PID is dead, delete the lock file |
| Worker exits with code 5 / 6 | Turn aborted / timeout | Check `%LOCALAPPDATA%/gcs-app-relay/relay.log`; investigate why orchestrator interrupted or stalled |
| App-server seems alive but worker relays still timeout | Codex CLI version mismatch | `codex --version` should report `0.130.x`+; the JSON-RPC method names are tied to the version (see "Verified Codex CLI version" below) |
| Viewer connects but `thread/resume` returns `no rollout found` | `~/.codex/gcs-orch-session-id` is wrong | Verify the UUID in the file matches the actual rollout filename in `~/.codex/sessions/YYYY/MM/DD/rollout-…-<UUID>.jsonl` |
| Orchestrator's shell exec calls report `invalid directory` | `session_meta.cwd` in the rollout is stale | One-time fix: kill the app-server, edit the JSON of the first line of the rollout to set `payload.cwd` to the correct path, restart the app-server (the 2026-05-14 migration backup includes a verified procedure) |

### Rollback to channel-send mode (legacy)

If the relay system breaks and you need the old behaviour back:

```cmd
:: 1. Disable relay mode
del C:/Users/Sam/.codex/gcs-mode
::    Future worker spawns now use Octogent channel-send + watchdog protocol

:: 2. Kill app-server + viewer cmd windows (they're no longer needed)

:: 3. Inside the Octogent terminal "codex-orchestrator-main",
::    re-launch the interactive Codex against the session:
codex resume 019dddb4-95f7-79e1-b48c-fdfc34fa3cd8
::    (close the terminal entirely first if a dead process is still listed)

:: 4. The dispatcher will now embed the legacy 4-step protocol in new
::    workers' initialPrompts. Already-spawned workers keep whatever
::    protocol was baked at their spawn time.
```

The migration backup is at `D:/_DEV/Work/Claude-Code-Game-Studios/reports/.backups/app-server-migration/rollout-019dddb4.pre-migration-2026-05-14T11-14-42.jsonl` (89 MB verbatim, sha256 verified) in case of catastrophic rollout corruption.

### Changing the orchestrator session-id

Forks, recovery resumes, or starting a new conversation can change the UUID. The relay's path depends on `gcs-orch-session-id` matching the rollout filename.

```cmd
:: 1. Update the config file
echo NEW-UUID-HERE > C:/Users/Sam/.codex/gcs-orch-session-id

:: 2. Verify the rollout exists at:
ls C:/Users/Sam/.codex/sessions/*/*/*/rollout-*NEW-UUID-HERE.jsonl

:: 3. Reconnect the viewer to the new id:
python C:/Users/Sam/.codex/gcs-app-viewer.py NEW-UUID-HERE
```

The app-server keeps prior threads loaded — if you resume a different thread, the previous one stays in memory but channel/turn ops are routed by `threadId` so no conflict.

### Verified Codex CLI version

Tested with `codex-cli 0.130.0`. The JSON-RPC schema is officially marked **experimental** in `codex app-server --help`. Method names like `thread/resume`, `turn/start`, `item/agentMessage/delta` can change in future versions. Pin the binary if drift is a concern:

```cmd
codex --version       :: should print 0.130.x or compatible
```

Generated JSON schemas live at `D:/tmp/codex-app-schema/` after running `codex app-server generate-json-schema --out <dir>` — useful diff against the running CLI's surface.

### What was reverted from the pre-app-server experiments

During the morning of 2026-05-14, several Octogent source patches were added in `D:/_APPS/Tools/octogent/` attempting to fix the channel-send-to-Codex-orchestrator issue at the Octogent layer (paste-mode hacks, retry chains, status flags, raw-keystroke branch). After the app-server migration, the raw-keystroke branch and the `submittedAt`/`submitFailed` flag-setting logic in `channelMessaging.ts` + the `ChannelMessage` core type were reverted (the orchestrator no longer receives reports via channel-send, so those code paths were dead-letter and risked false-positive submit-failed marks). Kept: `inFlightDeliveries` lock, one-at-a-time delivery, bracketed-paste + 200 ms `\r` (still used for orchestrator → worker REPONDRE), and the `initialPromptSubmittedAt`/`initialPromptSubmitFailed` mirror on PersistedTerminal/TerminalSnapshot (consumed by `gcs-spawn-watchdog.py`).

## 10. Related

- [Octogent — GitHub](https://github.com/hesamsheikh/octogent)
- [Octogent API reference](https://github.com/hesamsheikh/octogent/blob/main/docs/reference/api.md)
- `production/session-state/codex-orchestrator-state.md` — orchestrator contract (source of truth)
- `.claude/docs/coordination-rules.md` — disposition label spec (`NEW / CLEAR / REPONDRE / RELANCER`)
- `.claude/docs/orchestrator-paralelisme-optimisation.md` — wave parallelism context
