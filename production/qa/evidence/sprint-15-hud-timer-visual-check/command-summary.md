# Sprint 15 HUD Timer Eyeball -- Command Summary (Human-Operator Capture Launch Pattern)

> **Companion to**: [`README.md`](./README.md) in this directory.
> **Authoring prompt**: PROMPT 1011 (paperwork-only; no cargo invoked by this prompt).
> **Authoring base**: `origin/main@f3e635d657589ce41b7b1e9667207e0830bfedb0` (PROMPT 1010 tip).
> **Purpose**: ready-to-paste PowerShell commands for the operator's two-client capture session. The README owns the procedure; this file owns the exact commands.

---

## A. Cargo Resource Policy Preamble (Windows / MSVC -- PASTE FIRST)

Paste these five lines into **every** PowerShell terminal session **before** any `cargo` or `trunk` invocation. They redirect the cargo target dir to a dedicated SSD path, suppress debug-info bloat, and force a deterministic build profile that keeps disk pressure off the worktree drive.

```powershell
$env:CARGO_TARGET_DIR='D:\_DEV\cargo-target\ccgs-msvc'
$env:CARGO_PROFILE_DEV_DEBUG='0'
$env:CARGO_PROFILE_TEST_DEBUG='0'
$env:CARGO_INCREMENTAL='0'
$env:RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'
```

**Why this matters**: without the policy preamble, parallel `cargo` runs can balloon to 60-80 GB of incremental + debuginfo output and trigger `D:` disk-pressure stalls. The policy is canonical across Sprint 13 / 14 / 15 worker invocations -- see `production/qa/qa-plan-sprint-15.md` line 121-125 and the equivalent qa-plan-sprint-14.md / qa-plan-sprint-13.md preambles.

The preamble is **per-shell**, not global. New PowerShell windows MUST re-paste it.

---

## B. Repo Checkout Path Setup

Operator picks the repo checkout used for the capture (typical default below). Both server and client launch commands MUST run from this directory or from a child of it; `BEVY_ASSET_ROOT` MUST point at it absolutely.

```powershell
# Adjust to the operator's actual checkout path if different.
cd D:\_DEV\Work\Claude-Code-Game-Studios

# BEVY_ASSET_ROOT MUST match the repo path used by the operator; absolute, not relative.
$env:BEVY_ASSET_ROOT='D:\_DEV\Work\Claude-Code-Game-Studios'
```

If the operator runs from a different checkout (e.g., a fresh clone or a separate worktree dedicated to capture), substitute that path for **both** `cd` and `BEVY_ASSET_ROOT`. The two MUST match.

---

## C. Stale Target Cleanup Authorisation (only if disk pressure forces it)

The Sprint 15 QA plan (`production/qa/qa-plan-sprint-15.md` line 125) authorises **conditional** stale-target cleanup if `D:` free space drops below **40 GB**. The cleanup is implicit in the QA plan; the operator does NOT need a separate producer sign-off, but MUST verify the path before wiping.

```powershell
# 1. Check D: free space first.
Get-PSDrive D | Select-Object Used, Free

# 2. If Free < 40 GB, verify the canonical target dir.
Test-Path 'D:\_DEV\cargo-target\ccgs-msvc'   # MUST return True before next step.

# 3. Only after Test-Path = True, clear the CONTENTS (not the dir itself).
Remove-Item -Recurse -Force 'D:\_DEV\cargo-target\ccgs-msvc\*'
```

**Forbidden cleanup actions**:

- Do NOT clear directories outside `D:\_DEV\cargo-target\ccgs-msvc\`.
- Do NOT remove `D:\_DEV\cargo-target\ccgs-msvc` itself -- only its contents.
- Do NOT `Remove-Item` without first running `Test-Path` on the canonical dir.

---

## D. Server Launch (Terminal 1)

After Section A (Cargo Resource Policy) and Section B (Repo Checkout) are pasted:

```powershell
$env:SERVER_PORT='5000'
cargo run -p server --bin server
```

**Expected behaviour**:

- First build at the PROMPT 1010 tip will take several minutes on a cold target dir. Subsequent runs at the same commit are incremental and faster.
- The server prints a "listening on" line referencing `127.0.0.1:5000` (or whichever `SERVER_PORT` the operator chose). The operator pauses here until that line appears.
- If the server panics on startup, capture the full panic trace into the README "Run Results" -> Capture Session Metadata -> "Notes" subsection and abort the capture (do NOT proceed with clients against a panicking server).

The server MUST remain running for the full capture session. Stop it with `Ctrl+C` only after the third screenshot lands.

---

## E. Client A Launch (Terminal 2)

After Section A + Section B preambles in the new terminal:

```powershell
$env:SERVER_URL='ws://127.0.0.1:5000'
cargo run -p client --bin client
```

**Expected behaviour**:

- A Bevy window opens with the lobby UI.
- Connection status indicator shows "connected" within ~2-5 seconds.
- If the connection-lost overlay (per PROMPT 889 / `S13-CONN-LOST-UX-001`) appears at startup, the server is not reachable -- check Terminal 1 and re-confirm `SERVER_PORT` matches `SERVER_URL`.

The operator drives Client A through: lobby -> class select -> confirm class -> create friend-game room -> note room code.

---

## F. Client B Launch (Terminal 3)

Same as Section E (paste preambles, then run client). Client B uses the room code from Client A to join. Both clients confirm class.

```powershell
$env:SERVER_URL='ws://127.0.0.1:5000'
cargo run -p client --bin client
```

Once both clients have confirmed class, the server starts the round and `DraftInitial` (45 s countdown) begins. **First screenshot capture window**.

---

## G. WASM / Browser Variant (optional alternative to native clients)

If the operator wants browser/WASM clients instead of native Bevy windows, use `trunk` instead of `cargo run` for the clients. Server stays native.

```powershell
# Terminal 2 and 3 (one per browser tab):
trunk serve --release
```

Then open `http://127.0.0.1:8080` (or whichever port trunk picks) in two separate browser tabs. The same `SERVER_URL` env var is consumed by the client at build time; if the WASM client targets a different server address, the operator MUST rebuild with the correct `SERVER_URL` set before `trunk serve`.

**Note**: browser-WASM captures are valid for AC1-AC3; native captures are equally valid. Record the chosen mode in the README "Build / Source Metadata" table.

---

## H. Screenshot Capture Tooling

The operator picks any reliable Windows screenshot tool:

- **Snipping Tool** (Win + Shift + S) -- captures region; paste into image editor; save as PNG. Recommended for browser clients.
- **PrtScn key** (full screen) -- captures full screen to clipboard; paste; save. Useful for native client at known window position.
- **Windows Game Bar** (Win + G) -- can record full client window as PNG via the camera icon.
- **Bevy's screenshot system** -- the project does NOT ship a runtime screenshot hotkey for this capture session; do not enable one without authoring a follow-on story (AC4 forbids code change).

Save each PNG directly under `production/qa/evidence/sprint-15-hud-timer-visual-check/` with a name from one of the two filename conventions listed in the README "Capture Plan" table.

---

## I. Shutdown / Cleanup

After the third screenshot lands:

```powershell
# In Terminal 2 and 3 (clients):
# Close each client window normally (Bevy window X button, or Ctrl+C in the terminal).

# In Terminal 1 (server):
Ctrl+C

# Optional: clean up incremental build artefacts if disk pressure was hit during the run.
# Re-read Section C before doing this; the same Test-Path guard applies.
```

---

## J. Forbidden Commands (for the capture session only)

The following commands MUST NOT be run during the capture session (they would breach AC4 or close conditions that PROMPT 1011 and the capture session itself must preserve):

- `/dev-story`, `/story-done`, `/story-readiness`, `/smoke-check`, `/team-qa`, `/gate-check`, `/release-check`, `/qa-plan`. None of these is authorised by PROMPT 1011 nor by the capture session.
- Any `cargo` command that writes to `client/`, `server/`, `shared/`, or `tests/` source files (e.g., `cargo fmt --all` is forbidden mid-capture if it would touch files outside `production/qa/evidence/sprint-15-hud-timer-visual-check/`).
- Any `git` command that modifies `production/sprint-status.yaml`, `production/sprints/sprint-15.md`, `production/qa/qa-plan-sprint-15.md`, `production/stage.txt`, `production/gate-checks/*`, or the story file `production/epics/hud/story-014-hud-timer-eyeball-visual-check.md`.
- `git push --force` to `origin/main` (force-push to shared branches is never authorised).
- Any change to `Cargo.toml`, `Cargo.lock`, `.cargo/`, or `.github/`.

---

## K. Post-Capture Commit Recipe (after Run Results filled)

The operator stages ONLY the evidence README + the three screenshot PNGs (plus, on `FAIL`/`NEEDS-FOLLOW-ON`, the follow-on story file). Example:

```powershell
cd D:\_DEV\Work\Claude-Code-Game-Studios   # or the operator's checkout path

# Verify the only modified / untracked paths are the evidence dir (and optionally the follow-on story).
git status --short

# Stage explicitly (do NOT use `git add -A` or `git add .` -- avoid sweeping in unrelated work):
git add production/qa/evidence/sprint-15-hud-timer-visual-check/README.md
git add production/qa/evidence/sprint-15-hud-timer-visual-check/hud-timer-draft-initial-mid.png
git add production/qa/evidence/sprint-15-hud-timer-visual-check/hud-timer-draft-shop-mid.png
git add production/qa/evidence/sprint-15-hud-timer-visual-check/hud-timer-placement-mid.png
# On FAIL / NEEDS-FOLLOW-ON only:
# git add production/epics/hud/story-<NNN>-<slug>.md

# Verify staged changes match expectations.
git diff --cached --stat
git diff --cached --check     # no whitespace warnings expected

# Commit (use the operator's preferred sign-off footer; do NOT add a hidden agent footer).
git commit -m "evidence(s15): HUD timer eyeball human-operator capture (<verdict>)"
```

The commit lands on a branch (operator-named) and is pushed for review. The Sprint 15 row `S11-HUD-TIMER-EYEBALL-VISUAL-001` `status: ready -> done` flip happens in a **separate** `/story-done` paperwork prompt that runs AFTER the evidence commit lands on `origin/main`.

---

## L. Quick-Reference Capture Windows

| Phase | Countdown | Capture at ~ (elapsed) | Timer should read ~ (remaining) |
|-------|-----------|------------------------|---------------------------------|
| `DraftInitial` | 45 s | 20-30 s | 15-25 s |
| `DraftShop` | 30 s | 12-18 s | 12-18 s |
| `Placement` | 10-12 s | 4-6 s | 4-8 s |

Mid-countdown is the canonical capture point: it gives the visual evidence both a clearly running countdown (not phase-edge artefacts) and a clearly nonzero timer value (not the 0-second transition). End-of-phase or start-of-phase captures are acceptable as supplementary evidence but DO NOT substitute for the mid-countdown screenshot per phase.
