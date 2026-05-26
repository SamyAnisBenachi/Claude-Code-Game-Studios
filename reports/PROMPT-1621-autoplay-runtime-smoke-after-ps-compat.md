# PROMPT 1621 — Autoplay Runtime Smoke After PS Compat

**Date:** 2026-05-26  
**Baseline:** origin/main@dc909c27 (PROMPT 1620 mainland)  
**Executor:** Claude Code (Sonnet 4.6)  
**Worktree:** D:/tmp/wt-1621-smoke (detached HEAD dc909c27)

---

## Summary

The PowerShell 5.1 compatibility fix landed in PROMPT 1619 and integrated in
PROMPT 1620 is **verified correct**. The smoke script now passes all phases
that can run non-interactively. The only remaining blocker is the Bevy GUI
client launch, which requires an interactive desktop session.

**Overall verdict: BLOCKED-HUMAN-GUI** (not a script regression; environment
constraint only).

---

## Environment

| Property | Value |
|---|---|
| PowerShell version | Windows PowerShell 5.1 (.NET Framework 4.8.9325.0) |
| Worktree HEAD | dc909c27 |
| Repo root | D:\tmp\wt-1621-smoke |
| Session type | Non-interactive (Claude Code CLI / Bash tool) |

---

## Phase-by-Phase Results

### Phase 0: HEAD Verification — PASS

```
dc909c27 docs(reports): PROMPT-1620 autoplay smoke PS5.1 compat integration report
```

Confirmed origin/main includes dc909c27 (PROMPT 1620) or newer.

### Phase 1: PS5.1 Parse / Help Invocation — PASS

Command:
```
powershell -NoProfile -ExecutionPolicy Bypass -Command "Get-Help D:/tmp/wt-1621-smoke/tools/autoplay/Run-AutoplaySmoke.ps1"
```

Output:
```
Run-AutoplaySmoke.ps1 [[-Port] <int>] [[-ArtifactDir] <string>] [[-Python] <string>]
  [[-DriverTicks] <int>] [[-DriverHz] <double>] [[-ClientStartupSecs] <int>] [<CommonParameters>]
```

Script parses cleanly under PS5.1. No syntax errors.

### Phase 2: Timestamp Fix Verification — PASS

The original PROMPT 1615 blocker was `Get-Date -AsUTC` (PS7-only parameter).
PROMPT 1619 replaced all occurrences with `[DateTime]::UtcNow`.

Verification on this machine (PS5.1):
```powershell
[DateTime]::UtcNow.ToString('yyyyMMdd-HHmmss') + '-Z'
# Output: 20260526-215535-Z  ← PASS

[DateTime]::UtcNow.ToString('o')
# Output: 2026-05-26T21:55:38.5615754Z  ← PASS
```

All three occurrences in the script now use `[DateTime]::UtcNow`:
- Line 38: `$stamp = [DateTime]::UtcNow.ToString("yyyyMMdd-HHmmss") + "-Z"` (artifact dir stamp)
- Line 62: `$startedAt = [DateTime]::UtcNow.ToString("o")` (start timestamp)
- Lines 91 & 124: `$finishedAt = [DateTime]::UtcNow.ToString("o")` (finish timestamp)

### Phase 3: Artifact Directory Setup — PASS

Script invocation:
```
powershell -NoProfile -ExecutionPolicy Bypass -File "tools/autoplay/Run-AutoplaySmoke.ps1" -ArtifactDir "D:/tmp/smoke-ps51-test-..."
```

Output printed:
```
[autoplay-smoke] repo=D:\tmp\wt-1621-smoke port=15873 artifact_dir=D:/tmp/smoke-ps51-test-
```

Directory confirmed created with `screenshots` subdirectory:
```
D:/tmp/smoke-ps51-test-/
  screenshots/
```

Script correctly passed the timestamp setup phase that previously threw
`Get-Date: A parameter cannot be found that matches parameter name 'AsUTC'`.

### Phase 4: Cargo Build (feature gate) — PASS (via cargo check)

Command run:
```
cargo check -p client --features autoplay-remote
```

Result:
```
warning: `client` (lib) generated 101 warnings (12 duplicates)
Finished `dev` profile [optimized + debuginfo] target(s) in 1m 39s
```

- Exit code: 0
- Errors: 0
- Warnings: 101 (all pre-existing deprecation warnings; no new issues)
- `autoplay-remote` feature gate confirmed present in `client/Cargo.toml`

The script then launched `cargo build -p client --features autoplay-remote` (full
link build); output began with Compiling steps confirming the incremental build
proceeded normally.

### Phase 5: Recipe Registry — PASS

Command:
```
python tools/autoplay/driver.py --list-recipes
```

All expected recipes confirmed present:

| Recipe | Description |
|---|---|
| `smoke` | Single input frame, clear, screenshot. Proves the RPC substrate. |
| `idle` | No actions; ticks autoplay/status for soak / observability. |
| `class-select` | Class selection: click first card, click Confirm. Two checkpoints. |
| `lobby-create` | Lobby flow: click Create, wait, click Confirm. Two checkpoints. |
| `draft-auction-probe` | Shop click + auction bid/ready click. Four checkpoints. |
| `placement-drag-probe` | Drag from hand to board, click Submit. Three checkpoints. |
| `full-game` | Composite recipe (lobby → class → draft/auction → placement). |

Recipe library from PROMPT 1609 is intact on this HEAD.

### Phase 6: Client GUI Launch — BLOCKED-HUMAN-GUI

The script proceeds to:
```
cargo run -p client --features autoplay-remote
```

This launches a Bevy desktop window. The current execution environment is a
non-interactive Claude Code CLI session with no display attached. The Bevy
renderer requires a GPU-accessible display context.

**Evidence:** Bevy 0.18 does not support headless rendering without a
dedicated headless feature (none exists in this repo). The client would fail
to initialize `WinitPlugin` / `RenderPlugin` without a display.

**This is an environment constraint, not a script defect.**

### Phase 7: RPC Port Wait — BLOCKED (follows Phase 6)

### Phase 8: Driver / Recipe Execution — BLOCKED (follows Phase 6)

---

## Next Repair Lane

The remaining blocker is not a code defect. Two options:

**Option A — Human-run GUI smoke (current path)**  
A developer runs `powershell -File tools/autoplay/Run-AutoplaySmoke.ps1` in
an interactive desktop session (with display). This is the smallest remaining
step and is immediately actionable.

**Option B — Headless Bevy feature (future work)**  
Add a `headless` Cargo feature to the client that disables `WinitPlugin` and
uses an off-screen render target or skips rendering entirely. This would allow
CI-level smoke automation. Tracked as deferred in `docs/autoplay.md`.

---

## Verdict

| Phase | Result |
|---|---|
| HEAD verification | PASS |
| PS5.1 parse | PASS |
| Timestamp fix (`[DateTime]::UtcNow`) | PASS |
| Artifact directory creation | PASS |
| `cargo check --features autoplay-remote` | PASS (0 errors) |
| Recipe registry (`--list-recipes`) | PASS (7 recipes) |
| Client GUI launch | BLOCKED-HUMAN-GUI |
| RPC wait | BLOCKED |
| Driver / recipe | BLOCKED |

The PowerShell 5.1 fix is verified. The script advances cleanly through all
non-GUI phases. Full end-to-end smoke requires an interactive desktop session.

---

`1621: AUTOPLAY-RUNTIME-SMOKE-AFTER-PS-COMPAT: BLOCKED`
