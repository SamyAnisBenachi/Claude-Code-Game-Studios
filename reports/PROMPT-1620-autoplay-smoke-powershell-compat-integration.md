# PROMPT-1620 — Autoplay Smoke PowerShell Compat Integration Report

**Date:** 2026-05-26  
**Integrator:** Claude Code (claude-sonnet-4-6)  
**Source branch:** `origin/work/autoplay-smoke-powershell-compat-1619`  
**Integration branch:** `integrate/autoplay-smoke-powershell-compat-1620`  
**Integration tip:** `42e24411`  
**Base (origin/main):** `37306162`

---

## Summary

PROMPT 1619 fixed a Windows PowerShell 5.1 incompatibility in
`tools/autoplay/Run-AutoplaySmoke.ps1` by replacing `Get-Date -AsUTC`
(PowerShell 7+ only) with `[DateTime]::UtcNow` (compatible with PS 5.1 and
above). This integration cherry-picks both PROMPT 1619 commits cleanly onto
`origin/main`.

---

## Integration Steps

1. **Fetched** `origin` to refresh all remote refs.
2. **Verified** source branch `origin/work/autoplay-smoke-powershell-compat-1619`
   tip `cacb3fb0` is 2 commits ahead of `origin/main@37306162` — no rebase needed.
3. **Created** worktree `D:/Tmp/wt-1620` on new branch
   `integrate/autoplay-smoke-powershell-compat-1620` from `origin/main`.
4. **Cherry-picked** commits in order:
   - `434d93a0` → `a5502353` — script fix (0 conflicts)
   - `cacb3fb0` → `42e24411` — PROMPT 1619 report artifact (0 conflicts)
5. **Pushed** branch to `origin`.

---

## Validation Results

| Check | Result |
|-------|--------|
| `git diff --check` (whitespace) | PASS |
| `Get-Date -AsUTC` scan in `tools/autoplay/Run-AutoplaySmoke.ps1` | NOT FOUND (PASS) |
| `[DateTime]::UtcNow` replacement present (4 occurrences) | CONFIRMED |
| PowerShell 5.1 AST parse of `Run-AutoplaySmoke.ps1` | PASS |
| Path allowlist — only owned files changed | PASS (`tools/autoplay/Run-AutoplaySmoke.ps1`, `reports/PROMPT-1619-*.md`) |
| `git merge-base --is-ancestor origin/main HEAD` | PASS — fast-forward ready |

---

## Changed Files

```
reports/PROMPT-1619-autoplay-smoke-powershell-compat-repair.md  (new, 83 lines)
tools/autoplay/Run-AutoplaySmoke.ps1                             (7 insertions, 6 deletions)
```

---

## FF-Readiness

Integration branch `integrate/autoplay-smoke-powershell-compat-1620@42e24411`
is a **clean fast-forward** onto `origin/main@37306162`. No conflicts, no
out-of-scope edits, no Bevy/Cargo/shared-state touches.

---

## Deferred Steps

- **Live autoplay smoke run** not executed (per task scope: tooling-only
  integration, no GUI client launch). Mainland enqueue owner may optionally
  run `pwsh tools/autoplay/Run-AutoplaySmoke.ps1` or
  `powershell tools/autoplay/Run-AutoplaySmoke.ps1` to confirm runtime
  behavior under both shells before or after landing.

---

## Verdict

`READY_FOR_MAINLAND_ENQUEUE`

---

1620: AUTOPLAY-SMOKE-POWERSHELL-COMPAT-INTEGRATION: READY_FOR_MAINLAND_ENQUEUE
