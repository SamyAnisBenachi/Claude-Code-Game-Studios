# PROMPT 1584 — Dev Launcher Job Log Tail Panel

**Date**: 2026-05-21
**Source-of-truth**: `origin/main@9be8827fbd22b2a49d973ba585b5d210fdc8a903`
**Worker branch / cwd**: `work/dev-launcher-job-log-tail-panel-1584` @ `D:\Tmp\wt-1584`
**Scope**: implementation (Rust UI + pure-function tests in launcher crate)
**Status**: SHIPPED

---

## 1. Task recap

Add a visible "Last Job Tail" log panel to the CCGS dev launcher (`ccgs-dev-launcher.exe`)
so a tester can see WHY the most-recent job ended in FAIL / SUCCESS / RUNNING without
having to scroll the full Script Output log. Must preserve the PROMPT 1571 / 1577 SUCCESS
/ FAIL status badge contract (red/green/blue/idle palette + `compose_status_line` outcomes)
and the PROMPT 1579 PASS expectations.

Owned scope (per spawn prompt):

- `tools/dev-launcher-app/**`
- `tools/dev-launcher/**`
- launcher-specific tests under `tools/dev-launcher*/tests/**`

Forbidden: `client/**`, `server/**`, `shared/**`, sprint/QA/state files, unrelated
Cargo/CI files. None of these were touched.

---

## 2. Change summary

Single file modified: `tools/dev-launcher-app/src/main.rs` (+203 / -9).

### 2.1 New constants

- `TAIL_LINES: usize = 20` — number of lines surfaced in the tail panel.
- `TAIL_EMPTY_PLACEHOLDER: &str` — self-documenting placeholder shown before the
  first job has emitted any output (so the empty state explains itself).
- `COLOR_TAIL_BG: [u8; 3] = [29, 22, 40]` (plum/charcoal) and
  `COLOR_TAIL_TEXT: [u8; 3] = [253, 220, 156]` (warm amber).
  Distinct from the deep-navy full-log panel (`COLOR_LOG_BG = [18, 25, 34]`) and
  the muted-blue diagnostics panel (`COLOR_PANEL_BG = [246, 248, 251]`) so testers
  cannot confuse the always-visible 20-line tail with the scrollable full log
  directly below it.
- `WINDOW_SIZE` grew from `(980, 720)` to `(980, 820)` and `MIN_WINDOW_SIZE` from
  `(760, 560)` to `(760, 620)` to fit the new panel without crowding the existing
  diagnostics or log panels.

### 2.2 Pure tail helpers (unit-testable)

```rust
fn tail_log_lines<'a>(log: &'a [String], n: usize) -> &'a [String]
fn render_tail_text(log: &[String], n: usize) -> String
```

`tail_log_lines` returns the last `n` lines of `log` (or all of them if
`log.len() < n`); `n = 0` returns an empty slice. `render_tail_text` joins the
slice with `\n`, emitting `TAIL_EMPTY_PLACEHOLDER` when the log is empty.

These are kept pure (no `&self`, no Win32, no I/O) so the unit tests can
exercise them without spawning a window.

### 2.3 UI struct + GridLayout

Two new controls added to `LauncherUi`:

```rust
tail_heading_label: nwg::RichLabel    // "Last Job Tail (last 20 lines)"
tail_box:           nwg::RichTextBox  // readonly, monospace, dark theme
```

Grid rows after the change (col_span: 8 unless noted):

| Row(s)  | Control                            | Notes                                    |
|---------|------------------------------------|------------------------------------------|
| 0–1     | icon + brand                       | unchanged                                |
| 2–3     | rebuild_btn / launch_btn           | unchanged                                |
| 4       | state_label                        | unchanged                                |
| **5**   | **tail_heading_label**             | **new**                                  |
| **6–9** | **tail_box (row_span 4)**          | **new — pinned just below status badge** |
| 10      | diagnostics_heading_label          | shifted from row 5                       |
| 11–14   | diagnostics_box (row_span 4)       | shifted from rows 6–9, same span         |
| 15      | log_heading_label                  | shifted from row 10                      |
| 16–22   | log_box (row_span 7)               | shifted from rows 11–17, same span       |

The tail panel sits between the SUCCESS/FAIL/RUNNING badge and the diagnostics
panel — exactly where a tester's eye lands after the badge tells them "FAIL exit 1"
and they want to know why.

### 2.4 Render path

`refresh_log` was updated to repaint both panels in a single pass under one drain
of the `log_dirty` flag:

```rust
fn refresh_log(&self) {
    let (full, tail) = {
        // ... lock state, check log_dirty, set false, then ...
        let tail = render_tail_text(&state.log_lines, TAIL_LINES);
        let full = state.log_lines.join("\n");
        (full, tail)
    };
    set_rich_text_box(&self.log_box, &full, COLOR_LOG_TEXT, COLOR_LOG_BG, true);
    set_rich_text_box(&self.tail_box, &tail, COLOR_TAIL_TEXT, COLOR_TAIL_BG, true);
}
```

Doing both in one method prevents the race where two separate refreshes would
each consume the dirty flag and the second would skip a paint.

`on_init` now also applies the heading font + mono font to the new controls and
sets the tail panel background; the rendering itself is driven by the existing
`refresh_log()` call at end-of-init (init_lines push the launcher repo root /
branch / play-root status lines, which sets `log_dirty = true`).

### 2.5 PROMPT 1571 / 1577 / 1579 contracts preserved

No `JobOutcome`, `StatusTone`, `compose_status_line`, `COLOR_STATUS_*`, badge
routing, or play-root resolution touched. All 67 pre-existing tests still pass
unchanged.

---

## 3. Tests

12 new unit tests added inside the existing `#[cfg(test)] mod tests` block in
`tools/dev-launcher-app/src/main.rs`:

| Test                                                             | What it covers                                            |
|------------------------------------------------------------------|-----------------------------------------------------------|
| `tail_log_lines_returns_last_n_when_more_lines_exist`            | 50-line log, n=5 → returns lines 45..49                   |
| `tail_log_lines_returns_all_lines_when_fewer_than_n`             | 3-line log, n=20 → returns all 3                          |
| `tail_log_lines_empty_log_yields_empty_slice`                    | empty log → empty slice                                   |
| `tail_log_lines_n_zero_yields_empty_slice`                       | n=0 edge case                                             |
| `tail_log_lines_n_equal_to_len_returns_full_log`                 | n == len boundary                                         |
| `render_tail_text_shows_placeholder_when_log_empty`              | placeholder is shown + self-documenting                   |
| `render_tail_text_joins_lines_with_newline`                      | 3-line log → "alpha\nbeta\ngamma"                         |
| `render_tail_text_truncates_to_last_n_only`                      | 100-line log → only last TAIL_LINES rendered              |
| `tail_lines_constant_is_a_reasonable_default`                    | 10 <= TAIL_LINES <= 50                                    |
| `tail_surfaces_finished_banner_after_job_finishes`               | 150 noisy lines + `add_banner("FINISHED:...exit 1")` → banner appears in tail (regression for the user's actual pain) |
| `tail_color_palette_is_distinct_from_log_and_status_panels`      | TAIL_BG distinct from LOG_BG, PANEL_BG, STATUS_SUCCESS, STATUS_ERROR |
| `tail_panel_capacity_fits_within_log_cap`                        | TAIL_LINES <= MAX_LOG_LINES                               |

### 3.1 Focused test run

```
cargo test -p dev-launcher-app
```

Result:

```
test result: ok. 79 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
finished in 0.07s
```

Up from 67 to 79 tests; +12 new tests, all green. All 10 PROMPT 1571 status-UI
contract tests + 1 pre-existing `status_tone_colors_are_not_flat_defaults`
invariant + every PROMPT 1290 / 1309 / 1575 test continue to pass.

### 3.2 Path allowlist review

`git status --short tools/` shows only:

```
M tools/dev-launcher-app/src/main.rs
```

`git diff --check` reports no whitespace errors.

No edits to `client/**`, `server/**`, `shared/**`, sprint/QA/state files, or
unrelated Cargo / CI files. Compliant with the spawn-prompt allowlist.

### 3.3 Pre-existing warnings (not regressions)

The launcher build emits one pre-existing warning unchanged from PROMPT 1579:

```
warning: variant `Warning` is never constructed
   --> tools\dev-launcher-app\src\main.rs:296:5
```

`StatusTone::Warning` is reserved but currently unused (PROMPT 1571 routed
previously-yellow nonzero exits onto `StatusTone::Error`). Same warning the 1579
verify report flagged as "not a regression; cosmetic only" — left unchanged.

---

## 4. What the user will see at runtime

After a `Rebuild Latest Main` click that fails with exit 1:

1. Status badge turns solid red: `FAIL - Rebuild Latest Main exited 1 (nonzero).`
   (PROMPT 1571 behaviour, unchanged).
2. **NEW**: directly below the badge, the "Last Job Tail (last 20 lines)" panel
   shows on an amber-on-plum background the last 20 lines of script output,
   ending with the `==== FINISHED: Rebuild Latest Main (exit 1) ====` banner and
   the immediately preceding cargo / PowerShell error context.
3. Diagnostics panel below (unchanged) still surfaces the repo/play-root status.
4. Script Output panel at the bottom (unchanged) still shows the full
   `MAX_LOG_LINES = 2000` rolling log with scroll bars.

A tester now never needs to scroll the bottom Script Output panel to understand
a SUCCESS / FAIL outcome — the last 20 lines are always pinned in a dedicated,
high-contrast panel right under the status badge.

For an interactive verify (out of scope for this worker; report-only ran
`cargo test`):

1. Build: `cargo build -p dev-launcher-app --release` (or
   `tools/dev-launcher/build-launcher-exe.ps1` to stamp the sidecar).
2. Launch the EXE.
3. Click `Rebuild`. Confirm the tail panel populates as lines stream in, then
   shows the FINISHED banner once the worker finishes.
4. Force a fail-fast path: rename `tools/dev-launcher/Update-LatestMain.ps1`
   first; the launcher's pre-spawn missing-script branch fires the
   `ConfigError("launcher script missing on disk.")` outcome — the tail still
   shows the prior run's tail (the script-missing path doesn't append to the
   log mid-spawn; the diagnostics + badge surface the config error).

---

## 5. Blockers

None.

---

## 6. Commands run

```
git worktree add -b work/dev-launcher-job-log-tail-panel-1584 D:/Tmp/wt-1584 origin/main
cargo test -p dev-launcher-app
git diff --check
git diff --stat tools/
git status --short tools/
```

All commands succeeded.

---

1584: DEV-LAUNCHER-JOB-LOG-TAIL-PANEL: SHIPPED
