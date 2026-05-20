# PROMPT 1555 — Hand-Inspect Input Resource Optionalize Main-Ready Refresh

## Status

READY_FOR_MAINLAND_ENQUEUE

## Source-of-Truth

- origin/main @ `68a876cce6811e12228e7235f76970b8a662b828` (PROMPT 1551)

## Source Payload

- Source worker branch: `origin/work/hand-inspect-input-res-optionalize-1548` @ `78ac41db`
- Prior integration branch (NOT FF after PROMPT 1551): `origin/integrate/hand-inspect-input-res-optionalize-1552` @ `6e3d80d4`
- Reapplied commit:
  - `9735f04b` fix(hand-inspect): optionalize ButtonInput<KeyCode> resource (PROMPT 1548)

## Refreshed Branch

- Branch: `integrate/hand-inspect-input-res-optionalize-1555`
- New head: see commit appending this report.
- Base: `origin/main` (`68a876cc`).

## Changes (vs origin/main)

```
 client/src/ui/hand/inspect.rs                                  | 28 +++++++++--
 reports/PROMPT-1548-hand-inspect-input-res-optionalize.md      | 58 ++++++++++++++++++++++
 reports/PROMPT-1555-hand-inspect-input-res-optionalize-main-ready-refresh.md (this report)
```

The functional payload is identical to the PROMPT 1548 worker: `apply_hand_card_inspect_target_system` now takes
`Option<Res<ButtonInput<KeyCode>>>` so headless/test apps without `InputPlugin` no longer panic on the missing
resource. Escape `just_pressed` dismiss behavior is unchanged when the resource is present. Focused test
`apply_target_system_runs_without_button_input_resource` carried over.

The PROMPT 1552 integration report file (`reports/PROMPT-1552-hand-inspect-input-res-optionalize-integration-refresh.md`)
is intentionally NOT carried forward — it referenced the prior origin/main base (`b09fb48a`) and is superseded by this
PROMPT 1555 refresh report.

## Validation

- Path allowlist: only `client/src/ui/hand/inspect.rs`, `reports/PROMPT-1548-*.md`, and `reports/PROMPT-1555-*.md`
  (this report) are touched — no forbidden production/sprint/qa/state files.
- `git diff --check`: clean.
- `git merge-base --is-ancestor origin/main HEAD`: PASS (strict-FF eligible).
- Broad Cargo verification deferred per policy.

## Final Line

1555: HAND-INSPECT-INPUT-RES-OPTIONALIZE-MAIN-READY-REFRESH: READY_FOR_MAINLAND_ENQUEUE
