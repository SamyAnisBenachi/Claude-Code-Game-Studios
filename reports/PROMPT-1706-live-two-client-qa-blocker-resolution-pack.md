# PROMPT 1706 — LIVE-TWO-CLIENT-QA-BLOCKER-RESOLUTION-PACK

**Date**: 2026-05-27  
**Source-of-truth**: origin/main@f9324431 (PROMPT 1678) + 2 local unpushed commits (a9731dce, a16d0229)  
**Supersedes**: PROMPT 1699 PARTIAL  
**Branch with fixes**: `prompt-1706-launcher-stub-fix` @ e8cf8101

---

## Blocker Inventory (from PROMPT 1699 PARTIAL)

| # | Blocker | Class | Resolution |
|---|---------|-------|------------|
| B-1 | `client.exe` is 6 days stale (predates PROMPT 1678) | BUILD | Operator must rebuild — see Step 1 below |
| B-2 | `start-two-clients.bat` default path hits `ccgs-play-main` stub (no Cargo.toml → exit 1) | LAUNCHER BUG | **Fixed in this PROMPT** — see Fix A |
| B-3 | `update-latest-main.bat` default path hits same stub (no .git → opaque error) | LAUNCHER BUG | **Fixed in this PROMPT** — see Fix B |
| B-4 | Live two-client GUI test requires native GUI interaction | HUMAN_SIGNOFF | Inherently manual — operator checklist below |

---

## Automation Boundary

| What | Can automate? | Why |
|------|--------------|-----|
| Binary staleness check | Yes — already done by DryRun and soak pre-checks | `BuildProvenance.psm1` tracks mtime |
| Launcher dry-run validation | Yes — `Start-TwoClients.ps1 -DryRun` | Verified in this PROMPT |
| Server state machine (lobby→auction→placement→resolution) | Yes — bot-vs-bot soak covers it | PROMPT 1699 soak: PASS |
| Client build freshness | Semi — operator must trigger rebuild; script automates the build itself | Requires network + Cargo |
| Lobby phase (class select + confirm) | **No** | GUI click required |
| Auction phase (card offer, bid, timer) | **No** | GUI interaction + visual confirmation |
| Placement drag-drop | **No** | Bevy drag-drop; no scriptable headless path |
| Resolution visual | **No** | Animation + HP readout visual check |

---

## Fixes Implemented

### Fix A — `Start-TwoClients.ps1`: stub-aware default-path guard

**File**: `tools/dev-launcher/Start-TwoClients.ps1`  
**Commit**: e8cf8101 on `prompt-1706-launcher-stub-fix`

**Before**: `elseif (Test-Path $DefaultPlayRoot)` accepted the stub directory (exists, no
Cargo.toml) → fell into the Cargo.toml validation → hard exit 1 with generic error.

**After**: The condition now also requires `Cargo.toml` at the default path. If the stub
exists but has no `Cargo.toml`, a clear warning is emitted and the script falls through to
the launcher-root fallback — the same path used when no dedicated checkout is configured.

**Dry-run result (no args — previously broken)**:
```
WARNING: 'D:\_DEV\ccgs-play-main' exists but has no Cargo.toml (leftover stub).
         Falling back to launcher root. Run Update-LatestMain.ps1 or set
         CCGS_PLAY_REPO_ROOT to fix.
...
Server binary: D:\_DEV\cargo-target\ccgs-msvc\debug\server.exe (exists=True)
Client binary: D:\_DEV\cargo-target\ccgs-msvc\debug\client.exe (exists=True)
[dry-run completes, exit 0]
```

**Effect**: `start-two-clients.bat` (double-click, no args) now works correctly even with
the stub in place. Client and server binaries are resolved from the shared Cargo target dir.

---

### Fix B — `Update-LatestMain.ps1`: stub detection with actionable instructions

**File**: `tools/dev-launcher/Update-LatestMain.ps1`  
**Commit**: e8cf8101 on `prompt-1706-launcher-stub-fix`

**Before**: Stub path existed → worktree-creation block was skipped → hit `.git` check →
exit 1 with "Play/build root has no .git after creation" (no hint about what to do).

**After**: A new block before the worktree-creation check detects the stub condition
(`Test-Path` true but no `.git`) and emits three concrete options:

```
ERROR: 'D:\_DEV\ccgs-play-main' exists but is not a git checkout (no .git directory).
This is a leftover stub directory. Choose one of:

  Option A — use the launcher checkout directly (fastest):
    powershell -ExecutionPolicy Bypass -File tools\dev-launcher\Update-LatestMain.ps1 `
        -PlayRepoRoot 'D:\_DEV\Work\Claude-Code-Game-Studios'

  Option B — delete the stub so this script auto-creates a linked worktree:
    Remove-Item -Recurse -Force 'D:\_DEV\ccgs-play-main'
    (then re-run Update-LatestMain.ps1 without -PlayRepoRoot)

  Option C — set the env var permanently:
    $env:CCGS_PLAY_REPO_ROOT = 'D:\_DEV\Work\Claude-Code-Game-Studios'
```

---

## Binary Status (as of 2026-05-27)

| Binary | Path | Last modified (UTC) | Status |
|--------|------|---------------------|--------|
| `server.exe` | `D:\_DEV\cargo-target\ccgs-msvc\debug\server.exe` | 2026-05-27 12:53:27 | FRESH — rebuilt by PROMPT 1699 soak |
| `client.exe` | `D:\_DEV\cargo-target\ccgs-msvc\debug\client.exe` | 2026-05-21 13:06:08 | **STALE** — must rebuild before live test |

`client.exe` predates PROMPT 1678 (bot draft auto-pick fix) and must be rebuilt to get
an accurate two-client test result.

---

## Operator Checklist — Two-Client Live Test

> **Prerequisite**: merge or cherry-pick `prompt-1706-launcher-stub-fix` first, or run
> the scripts from that worktree at `.claude/worktrees/1706-launcher-stub-fix/`.

### Step 1 — Rebuild client (required)

```powershell
# Option A: use the worktree (fixes already applied)
cd D:\_DEV\Work\Claude-Code-Game-Studios\.claude\worktrees\1706-launcher-stub-fix
powershell -ExecutionPolicy Bypass -File tools\dev-launcher\Update-LatestMain.ps1 `
    -PlayRepoRoot "D:\_DEV\Work\Claude-Code-Game-Studios"
```

```powershell
# Option B: use main checkout after merge
cd D:\_DEV\Work\Claude-Code-Game-Studios
powershell -ExecutionPolicy Bypass -File tools\dev-launcher\Update-LatestMain.ps1 `
    -PlayRepoRoot "D:\_DEV\Work\Claude-Code-Game-Studios"
```

**Pass condition**: script exits 0; final output shows both binaries exist with today's mtime.

---

### Step 2 — Launch one server + two client windows

```powershell
# Double-click start-two-clients.bat — now works without args after Fix A
# Or explicitly:
powershell -ExecutionPolicy Bypass -File tools\dev-launcher\Start-TwoClients.ps1
```

**Pass condition**: two Bevy windows open; server log line confirms port bind.

---

### Phase 1 — Lobby

| Step | Client | Action | Pass condition |
|------|--------|--------|----------------|
| L-01 | A | Window opens | Title or connecting screen visible |
| L-02 | A | Wait for lobby | Class-selection panel + "Confirm" CTA visible |
| L-03 | A | Click a class | Class highlighted; stats preview updates |
| L-04 | A | Click Confirm | CTA → "Waiting for opponent…" |
| L-05 | B | Select class + confirm | Both confirmed → phase transitions |

---

### Phase 2 — Shop / Auction

| Step | Client | Action | Pass condition |
|------|--------|--------|----------------|
| A-01 | Both | Phase begins | Auction overlay visible; 3–5 card offers shown |
| A-02 | Both | Countdown timer | Ticks visibly; readable |
| A-03 | A | Leader label | "Your auction" OR opponent name from A's perspective |
| A-04 | B | Leader label | Reflects B's perspective (not A's label mirrored) |
| A-05 | A | Click a card | Bid highlighted/confirmed |
| A-06 | Both | Timer expires | Winner sees won card; loser sees opponent won |
| A-07 | Both | Transition | Phase advances to placement cleanly |

---

### Phase 3 — Placement

| Step | Client | Action | Pass condition |
|------|--------|--------|----------------|
| P-01 | Both | Phase begins | Board grid + unit card(s) in hand visible |
| P-02 | A | Hover card | Cell/lane highlight activates |
| P-03 | A | Drag card | Ghost follows cursor; target cell highlighted |
| P-04 | A | Drop on valid cell | Card snaps to cell; preview shown |
| P-05 | A | Submit | CTA clickable; server ACK → unit visible |
| P-06 | B | Place + submit | Mirror of above |

---

### Phase 4 — Resolution

| Step | Client | Action | Pass condition |
|------|--------|--------|----------------|
| R-01 | Both | Resolution starts | Visual transition / banner plays |
| R-02 | Both | Combat | Units visible in lanes; HP/damage indication |
| R-03 | Both | Round ends | Score / health update visible |
| R-04 | Both | Next phase | Returns to shop phase (or game-over cleanly) |

---

## Evidence Capture

After the run, collect from the auto-stamped evidence dir:

```
production/qa/evidence/dev-runs/<UTC-YYYY-MM-DD-HHMMSS>/
  launch-summary.json    ← written by launcher automatically
  build.json             ← binary provenance (commit SHA, mtime, profile)
  server.log             ← check for PANIC / ERROR lines
  client_a.log           ← check for ERROR / phase-transition lines
  client_b.log           ← same for Player B
```

Commit evidence with:
```
qa: PROMPT 1706 two-client live test — PASS/FAIL — <evidence-stamp>
```

---

## Summary

| Gate | Status |
|------|--------|
| B-2: `start-two-clients.bat` stub exit-1 | **FIXED** — stub now detected; falls to launcher-root fallback |
| B-3: `update-latest-main.bat` stub opaque error | **FIXED** — stub detected; three actionable options emitted |
| B-1: `client.exe` staleness | **ACTIONABLE** — Step 1 above rebuilds it |
| B-4: Live two-client GUI test | **HUMAN_SIGNOFF_REQUIRED** — operator checklist above; no automation path |

**What changed**: two launcher scripts patched to handle the `D:\_DEV\ccgs-play-main` stub.
Double-click `start-two-clients.bat` no longer exits 1. `update-latest-main.bat` gives
the operator three options instead of a cryptic error.

**What remains**: the operator must rebuild `client.exe` (Step 1) and run the GUI test
(Steps 2–4) — these are inherently human-gated steps.

---

1706: LIVE-TWO-CLIENT-QA-BLOCKER-RESOLUTION-PACK: SHIPPED
