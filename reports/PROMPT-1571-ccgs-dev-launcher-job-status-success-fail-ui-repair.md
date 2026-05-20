# PROMPT 1571 -- CCGS Dev Launcher Job Status Success/Fail UI Repair

- Branch: `work/ccgs-dev-launcher-job-status-1571`
- Worktree: `D:\_DEV\Work\ccgs-work-launcher-status-1571`
- Base: `origin/main@5be95a9b`
- Status: SHIPPED

## Problem

The launcher exposed exit codes but the visible "Status:" row was easy to miss:
- Success and failure used a pale-green / pale-yellow palette that did not read
  as a clear pass/fail at a glance.
- Nonzero exit was tagged "DONE WITH ERRORS" with the `Warning` tone (yellow),
  not the `Error` tone (red) -- so a real build failure looked like a soft warn.
- Initial state (no job has ever run, play/build root missing) and a successful
  job both showed muted backgrounds, easy to confuse visually.
- Status strings started with "Status: ..." which pushed the actual state word
  out of the first glance.

## Change

`tools/dev-launcher-app/src/main.rs`:

1. New `JobOutcome` enum (`Ready` / `Running(JobKind)` / `Success(JobKind)` /
   `Fail { job, code }` / `Error { job, reason }` / `ConfigError(String)`).
2. New pure helper `compose_status_line(&JobOutcome) -> (String, StatusTone)`
   that all UI callsites now use. Strings now lead with the state word:
   - `READY - idle. Click a button to start a job.`
   - `RUNNING - <job> in progress...`
   - `SUCCESS - <job> exited 0.`
   - `FAIL - <job> exited <code> (nonzero).`
   - `FAIL - <job> aborted: <reason>` (worker-side spawn/channel errors)
   - `FAIL - <message>` (config / missing-script / unresolved-repo errors)
3. Status palette strengthened so each final state is unambiguous:
   - `Success` -> solid green `[34, 139, 70]` background + white text
   - `Error`   -> solid red   `[192, 32, 32]` background + white text
   - `Running` -> solid blue  `[38, 89, 158]` background + white text
   - `Idle`    -> unchanged pale-blue background with panel text (no risk of
     being confused with the green Success badge)
   - `Warning` kept for symmetry but no longer fires on a nonzero exit.
4. Nonzero exit now resolves to `JobOutcome::Fail` -> `StatusTone::Error` (red),
   not the previous Warning tone (yellow). A real failed rebuild is now red.
5. Worker errors (`spawn failed`, `channel disconnected`) and worker
   `WorkerMessage::Error` paths also resolve through `JobOutcome::Error` ->
   `StatusTone::Error`, with the failure reason surfaced in the status row in
   addition to the existing log banner.
6. `JobKind` now derives `Debug` so `JobOutcome` can derive `Debug` for tests.
7. The launcher script path is unchanged; sidecar / play-root / build.json /
   evidence-dir behavior is preserved (no edits outside the status surface).

## UX behavior (now)

- Boot, play/build root missing: status row shows
  `READY - idle. Click a button to start a job.` on a pale-blue (Idle) panel.
  This is visually distinct from a green Success badge -- the previous palette
  used the same pale green for both `Success` and a quiet success-leaning idle.
- During a rebuild or two-client launch: status row shows
  `RUNNING - Rebuild Latest Main in progress...` on solid blue with white text.
- Exit 0: status row shows `SUCCESS - Rebuild Latest Main exited 0.` on solid
  green with white text. Last exit code is also in the diagnostics block.
- Exit nonzero: status row shows `FAIL - Rebuild Latest Main exited 1
  (nonzero).` on solid red with white text. Diagnostics + log retain the exit
  code and the run banner.
- Spawn or channel failure: `FAIL - Rebuild Latest Main aborted: <reason>` on
  solid red with white text.
- Repo root unresolved or launcher script missing: `FAIL - <reason>` on solid
  red with white text. Buttons stay disabled when repo root is unresolved.

## Validation

Cargo policy env vars applied per the project policy.

`cargo test -p dev-launcher-app --bin ccgs-dev-launcher`:
- 67 passed, 0 failed, 0 ignored.
- New tests for this PROMPT (all green):
  - `compose_status_line_ready_is_idle_tone`
  - `compose_status_line_running_is_running_tone_and_mentions_job`
  - `compose_status_line_success_exit_zero_is_success_tone`
  - `compose_status_line_fail_nonzero_is_error_tone` (exits 1, 2, -1, 255)
  - `compose_status_line_worker_error_is_fail_tone_and_quotes_reason`
  - `compose_status_line_config_error_is_fail_tone`
  - `status_tone_colors_success_uses_vivid_green_with_white_text`
  - `status_tone_colors_error_uses_vivid_red_with_white_text`
  - `status_tone_colors_running_is_distinct_from_success_and_error`
  - `status_tone_colors_idle_is_distinct_from_success`

`git diff --check`: clean.

Allowlist review: only `tools/dev-launcher-app/src/main.rs` modified. No
production/, no sprint state, no shared gameplay code, no Cargo manifest edits.

No broad workspace Cargo suites were run. The change is scoped to the
launcher crate; broader compile/test verification is left to VERIFY lanes per
PROMPT instructions.

## Files changed

- `tools/dev-launcher-app/src/main.rs` (+187 / -41)
- `reports/PROMPT-1571-ccgs-dev-launcher-job-status-success-fail-ui-repair.md`

## Blockers

None.

1571: CCGS-DEV-LAUNCHER-JOB-STATUS-SUCCESS-FAIL-UI-REPAIR: SHIPPED
