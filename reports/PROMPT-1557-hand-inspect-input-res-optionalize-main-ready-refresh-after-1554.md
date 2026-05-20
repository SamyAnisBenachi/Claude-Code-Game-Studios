# PROMPT 1557 — Hand-Inspect Input-Res Optionalize Main-Ready Refresh After 1554

## Status
READY_FOR_MAINLAND_ENQUEUE

## Refresh basis
- Source-of-truth main: `origin/main@51b3a718b009a36ec588cccdca10557155754a9c`
  (PROMPT-1554 main-ready refresh report for result/mulligan Krosmaga chrome polish).
- Previous refresh: `origin/integrate/hand-inspect-input-res-optionalize-1555@c07980a37c0379322932d9988808c15a628e3ba3`
  was based on `origin/main@68a876cc` and is no longer FF-eligible against current main
  after PROMPT 1554 landed.
- Source payload: PROMPT 1548
  `origin/work/hand-inspect-input-res-optionalize-1548@78ac41dba22be962dc33fd7082659006ddb8193d`
  — `fix(hand-inspect): optionalize ButtonInput<KeyCode> resource`.

## Refreshed branch
- Branch: `integrate/hand-inspect-input-res-optionalize-1557`
- Base: `origin/main@51b3a718`
- HEAD: cherry-pick of `d9d32d5` (PROMPT 1548 payload commit from the 1555
  integration branch) onto current `origin/main`, plus this report.

## Payload preserved (no edits to source content)
- `client/src/ui/hand/inspect.rs` — `apply_hand_card_inspect_target_system`
  takes `Option<Res<ButtonInput<KeyCode>>>` so headless/test apps without
  `InputPlugin` no longer panic. Escape `just_pressed` dismiss behavior
  unchanged when resource present.
- `client/src/ui/hand/inspect.rs` test
  `apply_target_system_runs_without_button_input_resource` preserved.
- `reports/PROMPT-1548-hand-inspect-input-res-optionalize.md` preserved verbatim.

## Path allowlist review
Only owned-scope paths touched:
- `client/src/ui/hand/inspect.rs`
- `reports/PROMPT-1548-hand-inspect-input-res-optionalize.md` (from cherry-pick)
- `reports/PROMPT-1557-hand-inspect-input-res-optionalize-main-ready-refresh-after-1554.md`
  (this report)

No edits to `production/sprint-status.yaml`, `production/session-state/**`,
`production/sprints/**`, `production/qa/**`, `production/stage.txt`, or any
unrelated Cargo/CI/source files.

## Checks
- `git diff --check origin/main HEAD` → clean (no whitespace/conflict markers).
- `git merge-base --is-ancestor origin/main HEAD` → true (strict-FF eligible).
- Cherry-pick applied cleanly with no conflict resolution required.
- Broad Cargo verification deferred to VERIFY lanes per policy.

## Source-payload integrity
Cherry-picked commit content identical to PROMPT-1548 / PROMPT-1552 / PROMPT-1555
payload (8-section diff: 2 files changed, 83 insertions, 3 deletions). No
behavioral changes from prior refresh — only the base commit moved from
`68a876cc` to `51b3a718`.

## Final line
1557: HAND-INSPECT-INPUT-RES-OPTIONALIZE-MAIN-READY-REFRESH-AFTER-1554: READY_FOR_MAINLAND_ENQUEUE
