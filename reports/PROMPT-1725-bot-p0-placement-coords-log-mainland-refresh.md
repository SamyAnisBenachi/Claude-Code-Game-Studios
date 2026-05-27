# PROMPT 1725 — BOT-P0-A-PLACEMENT-COORDS-LOG-MAINLAND-REFRESH

**Date:** 2026-05-27  
**Author:** orchestrator worker

## Summary

Refreshed the PROMPT 1719 placement-coordinate evidence logging changes onto
the latest `origin/main`. The previous integration branch
`integrate/bot-placement-coords-log-1724` (tip `a80c0b20`) was no longer
fast-forwardable from `origin/main` due to two docs commits that landed after
it diverged.

## SHAs

| Field | SHA |
|---|---|
| `origin/main` at time of refresh | `4685e508dc7fc4c822bb57a480a6ce755235b152` |
| Old rejected branch tip | `a80c0b20a9a07f25c9ea6455015a688535c145a6` |
| Cherry-pick source commit | `a80c0b20` (feat(bot): PROMPT 1719 — add placement coord logging to BotDecisionKind) |
| **New branch** | `integrate/bot-placement-coords-log-1725` |
| **New tip SHA** | `f71761d9b567ebbe7b94f2fe6598c3f3362abc23` |

## Method

- Created worktree `.claude/worktrees/prompt-1725-placement-coords-refresh`
  branching from `origin/main` (`4685e508`)
- Cherry-picked `a80c0b20` onto the new branch — applied cleanly with no conflicts
- Pushed to `origin/integrate/bot-placement-coords-log-1725`

## Validation

### Path allowlist review

All 7 files in the cherry-picked commit are within owned scope:

```
server/src/feature/bot/action_loop.rs   ✓
server/src/feature/bot/debug_push.rs    ✓
server/src/feature/bot/mod.rs           ✓
server/src/feature/bot/qa_snapshot.rs   ✓
server/src/feature/bot/state.rs         ✓
tests/unit/bot/bot_debug_push_test.rs   ✓
tests/unit/bot/bot_placement_wave_3_test.rs  ✓
```

No out-of-scope files touched.

### `git diff --check`

```
DIFF_CHECK_CLEAN
```

No whitespace errors.

### Ancestor check

```
origin/main IS ancestor of branch tip
```

`origin/main` (`4685e508`) is confirmed as ancestor of new branch tip (`f71761d9`).
The new branch is a strict fast-forward from `origin/main` + 1 cherry-picked commit.

### Cargo validation

Deferred to VERIFY lane per task instructions (focused validation only for
mainland refresh lanes).

## Result

**READY_FOR_MAINLAND_ENQUEUE**

- Branch: `integrate/bot-placement-coords-log-1725`
- Tip: `f71761d9b567ebbe7b94f2fe6598c3f3362abc23`
- Parent: `4685e508dc7fc4c822bb57a480a6ce755235b152` (current `origin/main`)
- Commits above main: 1
- Files changed: 7 (all in owned bot scope)

---

1725: BOT-P0-A-PLACEMENT-COORDS-LOG-MAINLAND-REFRESH: SHIPPED
