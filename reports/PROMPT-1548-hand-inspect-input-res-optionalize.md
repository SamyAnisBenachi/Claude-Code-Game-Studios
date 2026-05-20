# PROMPT 1548 — Hand Inspect Input Resource Optionalize

## Root cause

`apply_hand_card_inspect_target_system` declared `keys: Res<ButtonInput<KeyCode>>` as a required system param. In test/headless app states that do not register `bevy::input::InputPlugin` (or otherwise omit `ButtonInput<KeyCode>`), Bevy panics on schedule run with a missing-resource error, blocking focused hand-inspect tests and any non-input scaffold that exercises the inspect logic.

PROMPT 1536 verify lane (PARTIAL) recommended optionalizing the resource so the inspect path tolerates its absence.

## Fix

`client/src/ui/hand/inspect.rs`:

- Change `keys: Res<ButtonInput<KeyCode>>` → `keys: Option<Res<ButtonInput<KeyCode>>>` on `apply_hand_card_inspect_target_system`.
- Guard the Escape check with `if let Some(keys) = keys.as_deref()`. When the resource exists, behavior is byte-identical (still sets `dismiss = true` on Escape just_pressed). When absent, the system runs cleanly and only consumes the message readers.

Runtime impact: none — the live client always registers `InputPlugin`, so `Some(keys)` is always taken in production.

## Focused test added

`apply_target_system_runs_without_button_input_resource` — builds an `App` that initializes `HandCardInspectTarget` and registers the two messages + the system, but does NOT init `ButtonInput<KeyCode>`. Writes a `HandCardInspectRequested` and asserts the target opens. Before the fix this panicked at schedule-run time on the missing resource.

## Files changed

- `client/src/ui/hand/inspect.rs` (system signature + Escape guard + new test)

## Focused validation

```
cargo test -p client --lib ui::hand::inspect
```

Result:

```
running 6 tests
test ui::hand::inspect::tests::build_view_spell_omits_attack_health_and_fills_fallback_rules_text ... ok
test ui::hand::inspect::tests::build_view_minion_includes_attack_health_keywords ... ok
test ui::hand::inspect::tests::request_opens_then_repeat_request_closes ... ok
test ui::hand::inspect::tests::dismiss_message_closes_overlay ... ok
test ui::hand::inspect::tests::request_switches_to_different_card_without_dismiss ... ok
test ui::hand::inspect::tests::apply_target_system_runs_without_button_input_resource ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 119 filtered out; finished in 0.01s
```

`git diff --check` clean.

Path allowlist: only `client/src/ui/hand/inspect.rs` modified (plus this report). No production/**, no unrelated hand modules, no Cargo/CI changes.

## Branch / commit

- Branch: `work/hand-inspect-input-res-optionalize-1548`
- Base: `origin/main@f341d6c5156eb22544a05c1834d7179f560bf317`
- Commit: see git log of branch (push state below)

## Status

1548: HAND-INSPECT-INPUT-RES-OPTIONALIZE: SHIPPED
