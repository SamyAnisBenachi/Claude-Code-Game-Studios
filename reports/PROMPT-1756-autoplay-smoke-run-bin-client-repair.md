# PROMPT 1756 — AUTOPLAY-SMOKE-RUN-BIN-CLIENT-REPAIR

**Date**: 2026-05-28
**Branch**: fix/1756-autoplay-run-bin-client
**Worktree**: tmpwt-1756-bin-client-repair (based on origin/main @ d3d883e7)

## Problem

`tools/autoplay/Run-AutoplaySmoke.ps1` build step (added in PROMPT 1753) correctly scoped
the build to `--bin client`:
```
cargo build -p client --bin client --features autoplay-remote
```

But the client launch step on line 71 still used the old form without `--bin client`:
```
cargo run -p client --features autoplay-remote
```

This caused client startup failure in PROMPT 1755's live vs-bot smoke run — Cargo could
not resolve the binary unambiguously and exited instead of launching the GUI.

## Fix

Single-line change in `tools/autoplay/Run-AutoplaySmoke.ps1` line 72:

```diff
-    "run","-p","client","--features","autoplay-remote"
+    "run","-p","client","--bin","client","--features","autoplay-remote"
```

Both build and run now carry `--bin client`, matching each other.

## Validation

| Check | Result |
|---|---|
| PowerShell parser (`[Language.Parser]::ParseFile`) | PARSE OK |
| Static grep — `--bin` present in build ArgumentList | ✅ line 56 |
| Static grep — `--bin` present in run ArgumentList | ✅ line 72 |
| `git diff --check` (whitespace) | exit 0 — clean |

## Files changed

- `tools/autoplay/Run-AutoplaySmoke.ps1` — one line changed (launch ArgumentList)
- `reports/PROMPT-1756-autoplay-smoke-run-bin-client-repair.md` — this report

## Not done (out of scope)

- Live GUI smoke re-run — deferred to verify lane (PROMPT 1757 or equivalent)

---

1756: AUTOPLAY-SMOKE-RUN-BIN-CLIENT-REPAIR: SHIPPED
