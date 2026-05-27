# PROMPT 1679 — BOT-SOAK-LAUNCHER-STALE-BINARY-REBUILD-GUARD

**Date:** 2026-05-27
**Branch:** main
**File changed:** tools/dev-launcher/Start-BotVsBotSoak.ps1

## Problem

Start-BotVsBotSoak.ps1 only rebuilt server.exe and bot-soak-trigger.exe
when the binary was **missing**. If a binary existed from a previous build
pre-dating a source fix, the launcher silently reused it, producing false
failure evidence.

## Solution

Two complementary guards added:

### 1. Freshness guard (automatic)

New helpers Get-NewestSourceTime / Get-BinaryBuildReason compare binary
LastWriteTimeUtc against tracked sources:

| Binary | Source dirs | Source files |
|---|---|---|
| server.exe | server/src/, shared/src/ | Cargo.toml, Cargo.lock |
| bot-soak-trigger.exe | tools/two-client-runtime/src/, shared/src/ | Cargo.toml, Cargo.lock |

If any tracked file is newer than the binary, reason is source-newer and rebuild runs.

### 2. -Rebuild switch (unconditional)

Forces rebuild of both binaries regardless of mtime (reason: forced).
Use after git pull for certainty.

### Build decision surfaced in output and summary

Console now logs Server build reason and Trigger build reason.
soak-summary.json gains: server_build_reason, trigger_build_reason, rebuild_flag.

### Pre-existing bug fixed

 was uninitialised under -DryRun causing Set-StrictMode crash. Fixed.

## Validation

- Static PS parse: PARSE OK 0 errors
- git diff --check: clean
- -Help documents -Rebuild: yes
- -DryRun -Rebuild shows forced for both binaries: yes
- -DryRun (no -Rebuild) shows up-to-date for fresh binaries: yes
- -DryRun exits 0 cleanly: yes

## Usage

    # Default: auto-rebuilds when source newer than binary
    powershell -ExecutionPolicy Bypass -File tools\dev-launcher\Start-BotVsBotSoak.ps1

    # Force rebuild after git pull
    powershell -ExecutionPolicy Bypass -File tools\dev-launcher\Start-BotVsBotSoak.ps1 -Rebuild

    # Dry-run preview
    powershell -ExecutionPolicy Bypass -File tools\dev-launcher\Start-BotVsBotSoak.ps1 -DryRun -Rebuild

---
1679: BOT-SOAK-LAUNCHER-STALE-BINARY-REBUILD-GUARD: SHIPPED
