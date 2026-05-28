# PROMPT 1827 — AUTOPLAY-VSBOT-POST-HUMAN-RUN-VERIFY-TEMPLATE

**Status:** SHIPPED
**Date:** 2026-05-28
**Worktree:** `tmpwt-1827-post-human-run-verify-template`
**Branch:** `prompt/1827-post-human-run-verify-template`
**Base (origin/main):** `a0a96360`

---

## 1. Existing Run Scan — Pre-Human-GUI State

Evidence dir: `production/qa/evidence/autoplay-runs/`

Two runs found at time of scan:

| Dir | Timestamp | Has Screenshots | Notes |
|---|---|---|---|
| `20260528-051148-Z` | 2026-05-28T05:11:48Z | None (no screenshots/ contents) | Pre-1824; no win32 captures |
| `20260528-063609-Z` | 2026-05-28T06:36:09Z | 15 win32 + 15 Bevy | **Most recent — post-1824 run exists** |

### Summary of `20260528-063609-Z` (most recent, post-1824)

| Checklist Item | Result | Detail |
|---|---|---|
| `launcher-status.json` outcome | ✅ PASS | `"outcome": "ok"`, `driver_exit_code: 0` |
| Stale-pyc guard present | ⚠️ UNKNOWN | No stale-pyc guard log line found in `driver.log` or `process.log` |
| No `local.block` at vs-bot-precheck | ✅ PASS | vs-bot-precheck checkpoint reached and passed; no `local.block` emitted |
| Post-1818 driver labels (`win32_printwindow=`) | ❌ FAIL | All 15 captures logged as `win32_capture=OK` — pre-1818 label format |
| Screenshot captures exist | ✅ PASS | 15 win32 captures in artifact root + 15 Bevy screenshots in `screenshots/` |
| ≥3 distinct pixel hashes | ❌ FAIL | All 15 win32 captures share `pixel_hash=0x26207c4c` → **1 distinct hash** |
| Frozen fallback logs present | ❌ FAIL | No `win32_printwindow=FROZEN` or `desktop_bitblt ... reason=frozen_printwindow` |

**Diagnosis — frozen pixel hash:** The win32 capture path returns the same pixels for all 15
captures across the full recipe (lobby → bot-added → class-select → shop → auction →
placement → resolution). This is the frozen PrintWindow symptom that PROMPT 1818 addressed.
However, the driver does **not** detect the freeze under the `win32_capture=OK` label
(pre-1818 format), so the 1818 BitBlt fallback never triggers. The Bevy-internal
`screenshots/` path works correctly (15 distinct saves confirmed by bevy_render log).

**Conclusion:** This run shows the driver is still loaded with pre-1818 labels. The
`Start-AutoplayVsBot.ps1` composite entrypoint was used for this run (env vars were set:
`CCGS_AUTOPLAY_BOT_ROOM_READY` implicit from bot room running, `CCGS_DEBUG_UI` auto-set).
The env gate from PROMPT 1824 was not the bottleneck — the run reached `vs-bot-post-resolution`
and exited 0.

A fresh human GUI run using the **canonical** `Start-AutoplayVsBot.ps1` path and an updated
driver build should be performed to verify the post-1818 label and frozen fallback behaviors.

---

## 2. Human GUI Run Command

```powershell
# Terminal A — start bot soak room (leave running)
powershell -ExecutionPolicy Bypass -File tools\dev-launcher\Start-BotVsBotSoak.ps1

# Wait for: "Server listening on port ..." message in Terminal A

# Terminal B — run vs-bot smoke via canonical composite entrypoint
powershell -ExecutionPolicy Bypass -File tools\dev-launcher\Start-AutoplayVsBot.ps1 -Recipe vs-bot
```

**Alternative (direct smoke script, requires env gate to pass):**

```powershell
# Terminal A must already be running Start-BotVsBotSoak.ps1
$env:CCGS_AUTOPLAY_BOT_ROOM_READY = "1"
powershell -ExecutionPolicy Bypass -File tools\autoplay\Run-AutoplaySmoke.ps1 -Recipe vs-bot
# CCGS_DEBUG_UI will be auto-set to "1" by the PROMPT 1824 env gate
```

---

## 3. Artifact Directory Pattern

After the run, a new timestamped directory is created:

```
production\qa\evidence\autoplay-runs\YYYYMMDD-HHMMSS-Z\
  launcher-status.json          ← outcome, exit codes, port, timestamps
  status.json                   ← final client status snapshot
  driver.log                    ← full driver tick + capture log (primary evidence)
  driver-timeline.jsonl         ← per-tick timeline
  checkpoints.jsonl             ← named phase checkpoints with elapsed times
  capabilities.json             ← client capabilities negotiation
  process.log                   ← client stdout (Bevy INFO log)
  process.log.err               ← client stderr
  screenshots\                  ← Bevy-internal screenshot files (*.png + *.json)
  win32_tick_NNNNNN.png         ← win32 capture frames (in artifact root)
```

The new dir will sort **after** `20260528-063609-Z` — look for the latest directory.

---

## 4. Post-Run Verification Checklist

Fill in each item after inspecting the new artifact directory `<artifact-dir>`.

### 4.1 Launcher Status

```bash
# Inspect:
cat <artifact-dir>/launcher-status.json
```

| Item | Expected | Actual | Pass? |
|---|---|---|---|
| `"outcome"` field | `"ok"` | `<fill>` | `<fill>` |
| `driver_exit_code` | `0` | `<fill>` | `<fill>` |
| `client_exit_code` | `null` or `0` | `<fill>` | `<fill>` |

### 4.2 Stale-Pyc Guard

```bash
grep -i "stale\|pyc\|guard" <artifact-dir>/driver.log
```

| Item | Expected | Actual | Pass? |
|---|---|---|---|
| Stale-pyc guard log line present | Any `stale` / `pyc` / `guard` line | `<fill>` | `<fill>` |

> **Note:** This guard may not emit a line if no stale `.pyc` files were found.
> Absence of the line is acceptable if the guard ran silently. FAIL only if
> `driver.py` crashes at import with a `__pycache__` / `pyc` error.

### 4.3 vs-bot-precheck — No `local.block`

```bash
grep "local.block\|precheck\|BLOCKED" <artifact-dir>/driver.log
```

| Item | Expected | Actual | Pass? |
|---|---|---|---|
| `local.block` emitted by vs-bot-precheck | Absent | `<fill>` | `<fill>` |
| `vs-bot-precheck` exits cleanly | No BLOCKED exit before checkpoint reached | `<fill>` | `<fill>` |

### 4.4 Post-1818 Driver Labels

```bash
grep "win32_printwindow\|win32_capture" <artifact-dir>/driver.log | head -5
```

| Item | Expected | Actual | Pass? |
|---|---|---|---|
| Capture label format | `win32_printwindow=OK` or `win32_printwindow=FROZEN` | `<fill>` | `<fill>` |
| Old pre-1818 label absent | `win32_capture=OK` must NOT appear | `<fill>` | `<fill>` |

> **Context:** PROMPT 1818 renamed the win32 capture label from `win32_capture=OK` to
> `win32_printwindow=OK`/`FROZEN` to distinguish PrintWindow result from the
> fallback channel. If `win32_capture=OK` still appears, the driver was not
> rebuilt from the post-1818 source.

### 4.5 Screenshot Captures Exist

```bash
ls <artifact-dir>/win32_tick_*.png | wc -l
ls <artifact-dir>/screenshots/*.png | wc -l
```

| Item | Expected | Actual | Pass? |
|---|---|---|---|
| win32 capture PNGs | ≥ 10 | `<fill>` | `<fill>` |
| Bevy screenshots | ≥ 10 | `<fill>` | `<fill>` |

### 4.6 Distinct Screenshot Hashes (≥3)

```bash
grep "pixel_hash" <artifact-dir>/driver.log | awk '{print $NF}' | sort -u
```

| Item | Expected | Actual | Pass? |
|---|---|---|---|
| Distinct `pixel_hash` values | ≥ 3 | `<fill>` | `<fill>` |
| All hashes identical (frozen) | Must NOT all be the same | `<fill>` | `<fill>` |

> **Diagnosis if 1 distinct hash:** All captures are frozen to the same frame.
> Check whether `win32_printwindow=FROZEN` was logged and whether the
> `desktop_bitblt` fallback fired. If neither: driver sees frozen frames but
> freeze detection threshold not reached, or driver is pre-1818.
>
> **Diagnosis if ≥3 distinct:** PrintWindow is working correctly across game phases.

### 4.7 Frozen Fallback Logs (if applicable)

```bash
grep "FROZEN\|desktop_bitblt\|frozen_printwindow" <artifact-dir>/driver.log
```

| Item | Expected (if freezing) | Actual | Pass? |
|---|---|---|---|
| `win32_printwindow=FROZEN` present | If PrintWindow froze | `<fill>` | `<fill>` |
| `desktop_bitblt ... reason=frozen_printwindow` | BitBlt fallback triggered | `<fill>` | `<fill>` |
| Post-fallback hashes distinct | After fallback, new distinct hashes appear | `<fill>` | `<fill>` |

> **If no frozen entries and ≥3 distinct hashes:** PrintWindow worked without
> needing fallback — this is the ideal outcome. Mark 4.7 as N/A.

---

## 5. Overall Pass Criteria

| # | Item | Weight |
|---|---|---|
| 4.1 | Launcher outcome `ok`, exit 0 | BLOCKING |
| 4.3 | No `local.block` at precheck | BLOCKING |
| 4.5 | Screenshots exist | BLOCKING |
| 4.6 | ≥3 distinct pixel hashes OR clear frozen+fallback diagnosis | BLOCKING |
| 4.2 | Stale-pyc guard silent-pass | ADVISORY |
| 4.4 | Post-1818 labels (`win32_printwindow=`) | ADVISORY |
| 4.7 | Frozen fallback path exercised (if applicable) | ADVISORY |

**PASS** = all BLOCKING items green.
**PARTIAL** = BLOCKING items green but ADVISORY items red (log for follow-up).
**FAIL** = any BLOCKING item red (file a new PROMPT for diagnosis).

---

## 6. Existing Run Verdict Against Checklist

Run `20260528-063609-Z` scored against the checklist above:

| # | Item | Result |
|---|---|---|
| 4.1 | Launcher outcome | ✅ PASS (`ok`, exit 0) |
| 4.3 | No `local.block` | ✅ PASS |
| 4.5 | Screenshots exist | ✅ PASS (15 win32 + 15 Bevy) |
| 4.6 | ≥3 distinct hashes | ❌ FAIL (1 hash `0x26207c4c` across all 15 captures) |
| 4.2 | Stale-pyc guard | ⚠️ UNKNOWN (no log line) |
| 4.4 | Post-1818 labels | ❌ FAIL (`win32_capture=OK` — pre-1818 format) |
| 4.7 | Frozen fallback | ❌ FAIL (no frozen detection, no BitBlt fallback) |

**Verdict: PARTIAL** — run completed exit 0 (BLOCKING items 4.1 + 4.3 PASS,
4.5 PASS), but 4.6 BLOCKING fails (1 distinct hash). The Bevy-internal
`screenshots/` path produces valid captures; the win32 capture path is frozen.
A fresh human GUI run with the updated driver is needed to close BLOCKING 4.6.

---

## 7. Draft Next Worker Prompt

After the human GUI run completes, launch this prompt with `<artifact-dir>`
replaced by the actual directory name:

```
PROMPT 1828 -- AUTOPLAY-VSBOT-POST-1827-RUN-EVIDENCE-VERIFY

Context:
- PROMPT 1827 prepared this verification template.
- A human GUI run was completed via Start-AutoplayVsBot.ps1 -Recipe vs-bot.
- Artifact dir: production/qa/evidence/autoplay-runs/<artifact-dir>

Task:
- Fill the PROMPT 1827 checklist against the new artifact dir.
- Report each checklist item (4.1 through 4.7) with actual values.
- If 4.6 fails (1 distinct hash), check 4.4 and 4.7 for root cause.
- If 4.4 shows pre-1818 labels, diagnose whether driver was rebuilt post-1818.
- Write filled report to reports/PROMPT-1828-autoplay-vsbot-post-1827-run-evidence-verify.md.
- Do not modify source code.

Owned scope:
- Read-only: production/qa/evidence/autoplay-runs/<artifact-dir>/**, driver.py, reports/PROMPT-1827-*.md
- Write only: reports/PROMPT-1828-autoplay-vsbot-post-1827-run-evidence-verify.md

Final line exactly: 1828: AUTOPLAY-VSBOT-POST-1827-RUN-EVIDENCE-VERIFY: STATUS
```

---

## 8. Files Written

| File | Action |
|---|---|
| `reports/PROMPT-1827-autoplay-vsbot-post-human-run-verify-template.md` | Created — this report |

---

1827: AUTOPLAY-VSBOT-POST-HUMAN-RUN-VERIFY-TEMPLATE: SHIPPED
