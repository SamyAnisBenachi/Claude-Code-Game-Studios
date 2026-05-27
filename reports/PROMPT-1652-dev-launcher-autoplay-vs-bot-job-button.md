# PROMPT-1652 Report: DEV-LAUNCHER-AUTOPLAY-VS-BOT-JOB-BUTTON

**Branch:** `work/1652-launcher-autoplay-btn`
**Commit:** `cecc192b`
**File changed:** `tools/dev-launcher-app/src/main.rs`

## What was done

Added a third "Autoplay vs Bot QA" button to the CCGS Dev Launcher app, wiring
it to `tools/dev-launcher/Start-AutoplayVsBot.ps1`. The button is clearly
visually separated from the two-client play session button and surfaces BLOCKED
state distinctly from FAIL.

### UI layout change

| Button | Old col / span | New col / span |
|---|---|---|
| Rebuild Latest Main | col 0, span 4 | col 0, span 3 |
| Start Two-Client Play Session | col 4, span 4 | col 3, span 2 |
| **Autoplay vs Bot QA** (new) | â€” | col 5, span 3 |

All three buttons remain in the same row (row 2, row_span 2). Total span = 3+2+3 = 8.

### BLOCKED exit code handling

`Start-AutoplayVsBot.ps1` defines four expected BLOCKED-* exit codes:

| Code | Meaning |
|------|---------|
| 4 | BLOCKED-RECIPE-GUARD (local.block fired) |
| 10 | BLOCKED-HUMAN-GUI (non-interactive; Bevy needs visible desktop) |
| 11 | BLOCKED-PRECONDITION (soak server absent or smoke script missing) |
| 12 | BLOCKED-PRECONDITION (soak server did not bind in time) |

These are classified as `JobOutcome::Blocked` (yellow/Warning tone) rather than
`JobOutcome::Fail` (red/Error tone). All other nonzero exits remain FAIL. This is
enforced by `classify_exit()`, a pure helper that is tested independently.

A `BLOCKED` badge status line reads:
> `BLOCKED - Autoplay vs Bot QA exited 10 (BLOCKED-HUMAN-GUI / BLOCKED-PRECONDITION / BLOCKED-RECIPE-GUARD). Check script output for details.`

### Diagnostics panel

The "Diagnostics" panel now lists the autoplay script path alongside Rebuild and
Two-Client script paths.

### Tests

87 tests pass (up from 77). 10 new tests added:

- `classify_exit_zero_is_success_for_all_job_kinds`
- `classify_exit_nonzero_is_fail_for_rebuild_and_launch`
- `classify_exit_blocked_codes_are_blocked_for_autoplay`
- `classify_exit_generic_fail_is_fail_for_autoplay`
- `compose_status_line_blocked_uses_warning_tone_and_says_blocked`
- `compose_status_line_blocked_mentions_job_name`
- `blocked_tone_is_visually_distinct_from_fail_and_success`
- `autoplay_script_constant_is_correct_ps1_name`
- Updated `job_kind_human_labels_match_button_text` (added Autoplay assertion)
- Updated `job_kind_script_paths_use_dev_launcher_dir` (added Autoplay assertions)
- Updated `diagnostics_text_surfaces_scrollworthy_paths` (AUTOPLAY_SCRIPT check)
- Updated `app_identity_strings_are_distinct_nonempty` (subtitle now mentions autoplay)

## What was NOT changed

- `Start-AutoplayVsBot.ps1` â€” untouched (owned by PROMPT 1648)
- All game client/server/shared Rust â€” untouched
- Sprint/session-state/story files â€” untouched
- Rebuild and Start Two-Client logic â€” unchanged, only layout spans adjusted

## Validation

- `git diff --check` â€” clean (no trailing whitespace)
- `cargo test` in `tools/dev-launcher-app` â€” 87 passed, 0 failed
- Branch pushed: `work/1652-launcher-autoplay-btn` @ `cecc192b`

---

## Manual test steps

After rebuilding the launcher EXE from this branch:

1. Open the launcher; confirm three buttons are visible in a single row: "Rebuild Latest Main", "Start Two-Client Play Session", "Autoplay vs Bot QA"
2. Click "Autoplay vs Bot QA" â€” status should change to RUNNING (blue)
3. If run non-interactively, script exits 10 â†’ badge should turn **yellow** and read `BLOCKED - Autoplay vs Bot QA exited 10 (...)`
4. If script is missing, script exits 11 â†’ badge turns **yellow** (`BLOCKED`)
5. Confirm Diagnostics panel lists `Autoplay-vs-Bot script: <repo>\tools\dev-launcher\Start-AutoplayVsBot.ps1`
6. Confirm Rebuild and Two-Client buttons still work as before

---

1652: DEV-LAUNCHER-AUTOPLAY-VS-BOT-JOB-BUTTON: SHIPPED
