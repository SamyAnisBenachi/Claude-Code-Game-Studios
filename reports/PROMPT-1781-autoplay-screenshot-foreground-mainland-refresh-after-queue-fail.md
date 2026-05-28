# PROMPT 1781 — Autoplay Screenshot Foreground Mainland Refresh After Queue Fail

**Date**: 2026-05-28
**Branch**: work/1781-autoplay-screenshot-foreground-mainland
**Status**: SHIPPED

---

## Background

PROMPT 1779 reported SHIPPED on `origin/work/1779-screenshot-foreground-refresh`
but MAINLAND_ENQUEUE queue **mlq_3b1acb58adca46a0** failed because the 1779 source
branch was not fast-forwardable from `origin/main` at the time of queue submission
(intervening commit from PROMPT 1778 BOM repair had landed on main).

This task re-applies the PROMPT 1776 foreground payload cleanly from a fresh
worktree branched off the latest `origin/main`.

---

## Base

| Field | Value |
|---|---|
| Tested base SHA | `7d4a4872f19fc0fdeceda4a8f6a86596eb8564e5` |
| Base commit message | `fix(autoplay): PROMPT 1778 — tolerate UTF-8 BOM in composite-summary.json` |

---

## Cherry-pick source

| Field | Value |
|---|---|
| Source branch | `origin/wt/1776-autoplay-screenshot-foreground` |
| Source commit | `ec7e8f1b0858e26020e3c3cffeb583d8df31afa3` |
| Source commit message | `fix(autoplay): PROMPT 1776 — foreground Bevy window before screenshot RPC` |

Cherry-pick applied cleanly with no conflicts.

---

## Final branch

| Field | Value |
|---|---|
| Branch | `work/1781-autoplay-screenshot-foreground-mainland` |
| Final commit | `72db59653212537733836dc28f639dfa11ea6c90` |
| Pushed to origin | Yes |

---

## FF-readiness proof

```
$ git merge-base --is-ancestor origin/main HEAD && echo "FF_READY"
FF_READY
```

`origin/main` (`7d4a4872`) is a direct ancestor of HEAD (`72db5965`).
A `git merge --ff-only` from main will succeed with no merge commit.

---

## Files in diff

```
reports/PROMPT-1776-autoplay-screenshot-window-foreground-repair.md  (report from 1776)
tests/tools/autoplay/test_win_foreground.py                           (new, 252 lines)
tools/autoplay/driver.py                                              (modified, +5 lines)
tools/autoplay/win_foreground.py                                      (new, 142 lines)
```

Forbidden scope confirmed absent: `client/src/autoplay.rs` not touched.

---

## Whitespace check

```
git diff --check HEAD~1 HEAD -- tools/autoplay/ tests/tools/autoplay/
→ CLEAN (no trailing-whitespace issues in Python files)
```

(Report markdown uses Markdown trailing-space line-breaks; not a code issue.)

---

## Validation

```
pytest tests/tools/autoplay/test_win_foreground.py -v
→ 25 passed in 0.09s
```

All 25 mocked-ctypes tests pass headlessly without a GUI or live Bevy client.

---

## Original failed queue

| Field | Value |
|---|---|
| Failed queue ID | `mlq_3b1acb58adca46a0` |
| Failure reason | Source branch not FF-able from origin/main (1778 had landed) |
| Resolution | Fresh worktree + cherry-pick from clean base |

---

## Next action

Mainland enqueue `work/1781-autoplay-screenshot-foreground-mainland` →
`origin/main` via `MAINLAND_ENQUEUE` (FF-merge only).

---

1781: AUTOPLAY-SCREENSHOT-FOREGROUND-MAINLAND-REFRESH-AFTER-QUEUE-FAIL: SHIPPED
