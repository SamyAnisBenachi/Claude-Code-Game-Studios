# PROMPT 1552 — HAND-INSPECT-INPUT-RES-OPTIONALIZE-INTEGRATION-REFRESH

## Summary

Integration-refresh of PROMPT 1548 payload onto current `origin/main`.
Single cherry-pick of `78ac41db` (fix(hand-inspect): optionalize
`ButtonInput<KeyCode>` resource) applied cleanly with no conflicts.

## Source

- Source worker: PROMPT 1548
- Source branch: `origin/work/hand-inspect-input-res-optionalize-1548`
- Source commit: `78ac41db`
- Source report: `reports/PROMPT-1548-hand-inspect-input-res-optionalize.md`

## Refresh

- Base: `origin/main@b09fb48a` (includes PROMPT 1550 landed during refresh).
- Branch: `integrate/hand-inspect-input-res-optionalize-1552`
- Refresh commit: `9735f04b` (clean cherry-pick of `78ac41db`)
- Worktree: `D:/tmp/wt-1552`

## Files Changed

- `client/src/ui/hand/inspect.rs` (+25/-3)
  - `apply_hand_card_inspect_target_system` takes
    `Option<Res<ButtonInput<KeyCode>>>` instead of `Res<…>`; Escape branch
    runs only when resource is present.
  - Added focused test
    `apply_target_system_runs_without_button_input_resource`.
- `reports/PROMPT-1548-hand-inspect-input-res-optionalize.md` (new, +58)

Both paths are within the owned allowlist (source-payload files plus this
report). No production/, sprint, QA, Cargo, or CI files touched.

## Checks

- `git diff --check HEAD~1 HEAD` — clean (no whitespace errors).
- Path allowlist review — PASS (only PROMPT 1548 payload + this report).
- FF-readiness: `git merge-base --is-ancestor origin/main HEAD` — PASS.
- Broad Cargo verification deferred to VERIFY lanes per policy.
- `liv-bevy-018`: change is API-correct for Bevy 0.18 — `Option<Res<T>>`
  is the supported pattern for systems that may run without a resource;
  `MessageReader`/`Messages::write` and `app.add_message::<…>()` already
  match the 0.18 message API used elsewhere in the file.

## Status

READY_FOR_MAINLAND_ENQUEUE — FF-ready on top of current origin/main, no
conflicts, owned-scope only.

1552: HAND-INSPECT-INPUT-RES-OPTIONALIZE-INTEGRATION-REFRESH: READY_FOR_MAINLAND_ENQUEUE
