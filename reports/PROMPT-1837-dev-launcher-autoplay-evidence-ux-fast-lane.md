# PROMPT 1837 — DEV-LAUNCHER-AUTOPLAY-EVIDENCE-UX-FAST-LANE

_Generated: 2026-05-28 | Branch: `wt-1837-dev-launcher-evidence-ux` | Final commit: f256fb85_

---

## Problem

After `Start-AutoplayVsBot.ps1` completes a run (success, failure, or blocked),
the operator's terminal shows an outcome line but **no evidence paths**. The paths
are printed mid-run (section 4 "Evidence dir" and section 7 "Autoplay artifact
dir:") buried above minutes of build and soak output. Finding them requires
scrolling back through the full terminal log.

---

## Inspection Summary

| File | Finding |
|---|---|
| `tools/dev-launcher/Start-AutoplayVsBot.ps1` | Sections 1–9 complete. Final block (human-readable outcome, lines 415–428) prints PASS/FAIL text but zero evidence paths. |
| `tools/autoplay/Run-AutoplaySmoke.ps1` | Prints `[autoplay-smoke] artifacts under $ArtifactDir` at end — analogous pattern, but Run-AutoplaySmoke is called as a child process; its terminal output doesn't persist in the parent window on all configurations. |
| `docs/autoplay/evidence-operator-guide.md` | §3 documents the layout; §10 documents `validate_composite_run.py`. Both are well-maintained. No doc update needed. |
| PROMPT 1832/1833 conflict check | Neither touches `tools/dev-launcher/Start-AutoplayVsBot.ps1`. Conflict: NONE. |

---

## Change Implemented

**File:** `tools/dev-launcher/Start-AutoplayVsBot.ps1`  
**Scope:** Section 10 appended before `exit $smokeExit` (lines 430–447 post-edit)

The new section always emits (regardless of exit code):

```
---- Evidence paths ----
Composite dir:  <evidenceDir>
Autoplay run:   <autoplayArtifactDir>
Summary JSON:   <compositeSummaryPath>

  Validate: python "<validate_composite_run.py path>" "<evidenceDir>"
------------------------
```

The validate hint is only printed on `exit 0` + non-dry-run and only when
`tools/autoplay/validate_composite_run.py` exists — so it degrades gracefully
on setups where the validator is missing.

---

## Addendum — Window-Size Visibility (human observation post-ship)

**Observation:** autoplay opened the game window too small; UI was clipped and
the bot clicked blank/offscreen space.

**Division with PROMPT 1842:** 1842 owns the actual window-size **repair**
(setting `CCGS_WINDOW_WIDTH`/`CCGS_WINDOW_HEIGHT` or equivalent in the launch
path). 1837 owns **diagnostic visibility** — surfacing whatever size is active
so operators can confirm the viewport at a glance.

**Addendum change (commit f256fb85):** Section 10 extended with a
`---- Window config ----` block that always prints:

```
---- Window config ----
CCGS_WINDOW_WIDTH         = (not set)   ← or value once PROMPT 1842 lands
CCGS_WINDOW_HEIGHT        = (not set)
CCGS_WINDOW_POSITION      = (not set)
WINIT_X11_SCALE_FACTOR    = (not set)
  WARNING: window size not set by launcher; game opens at OS default.
           Bot click targets may be offscreen if the window is too small (< 1280x720).
           See PROMPT 1842 default-size repair.
-----------------------
```

When 1842 sets the env vars in the earlier launch section, this block will
automatically switch to the green confirmation line instead of the warning.
No edits needed in the same launch-section lines that 1842 will touch — zero
merge conflict risk.

**Conflict check:** 1842's edits are in sections 1–9 (launch args / env injection).
Section 10 is append-only. No overlapping lines.

---

## Validation

- `git diff --check` — clean (no trailing-whitespace issues)
- PowerShell `Parser::ParseFile` — PARSE OK
- Path allowlist: only `tools/dev-launcher/Start-AutoplayVsBot.ps1` and
  `reports/PROMPT-1837-*.md` touched — within owned scope.
- Conflict check: no edits to `tools/autoplay/` recipe/analyzer files.

---

## Files Changed

| File | Action |
|---|---|
| `tools/dev-launcher/Start-AutoplayVsBot.ps1` | +17 lines section 10 evidence fast-lane (commit d0c28cd7); +21 lines window-config block (commit f256fb85) |
| `reports/PROMPT-1837-dev-launcher-autoplay-evidence-ux-fast-lane.md` | new + addendum |

---

1837: DEV-LAUNCHER-AUTOPLAY-EVIDENCE-UX-FAST-LANE: SHIPPED
