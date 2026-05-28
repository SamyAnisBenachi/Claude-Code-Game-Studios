# PROMPT 1787 — Autoplay Foreground Window Title Mainland Refresh After 1785

**Date:** 2026-05-28  
**Branch:** `integrate/autoplay-foreground-window-title-1787`  
**Worktree:** `tmpwt-1787-foreground-window-title-mainland`  
**Integration HEAD:** `40e6c48b`  
**Source commit cherry-picked:** `cd505f14` (PROMPT 1786, `fix/1786-foreground-window-title`)  
**Base:** `origin/main@4ffec0dfcba9a98a0f8c26a7d1ebf515c51ff02a` (PROMPT 1785 — BOM validator repair)

---

## What This Refresh Does

PROMPT 1786 shipped `cd505f14` on branch `fix/1786-foreground-window-title` based
on `4ab3e71e`, which predates PROMPT 1785 (`4ffec0df`). This PROMPT 1787 integration
refresh cherry-picks `cd505f14` cleanly onto `origin/main` so the foreground-title
fix is FF-mergeable without rebasing PROMPT 1785.

---

## Cherry-Pick Result

```
[integrate/autoplay-foreground-window-title-1787 40e6c48b]
  fix(autoplay): PROMPT 1786 — repair foreground window title discovery for Lanes and Lies
  3 files changed, 289 insertions(+), 5 deletions(-)
  create mode 100644 reports/PROMPT-1786-autoplay-foreground-window-title-diagnostic-repair.md
```

No conflicts. Cherry-pick applied cleanly.

---

## Files Changed

| File | Change |
|------|--------|
| `tools/autoplay/win_foreground.py` | +45 lines — `"lanes and lies"` / `"lanes"` hints, `_format_diag_titles()`, improved no-match log |
| `tests/tools/autoplay/test_win_foreground.py` | +100 lines — 13 new tests covering lanes hints, diag formatter, regression guard |
| `reports/PROMPT-1786-autoplay-foreground-window-title-diagnostic-repair.md` | New — PROMPT 1786 report carried over |

---

## Validation

### git diff --check

```
diff-check-clean
```

No trailing whitespace or merge conflict markers.

### FF-readiness

```bash
git merge-base --is-ancestor origin/main HEAD
# exit 0 → FF-READY: origin/main is ancestor of HEAD
```

Branch is a fast-forward from `origin/main@4ffec0df`.

### Python test suite

```
python -m pytest tests/tools/autoplay/test_win_foreground.py tests/tools/autoplay/test_validate_composite_run.py -v
```

```
============================= test session starts =============================
platform win32 -- Python 3.12.10, pytest-9.0.3, pluggy-1.6.0
collected 79 items

tests/tools/autoplay/test_win_foreground.py  ......................  38 passed
tests/tools/autoplay/test_validate_composite_run.py  .................  41 passed

============================= 79 passed in 0.75s ==============================
```

- **38 `test_win_foreground.py`** — all pass, including the 13 PROMPT 1786 additions
  (lanes-and-lies regression guard, `_format_diag_titles` suite, ensure_foreground integration)
- **41 `test_validate_composite_run.py`** — all pass, confirming PROMPT 1785 BOM repair
  coexists cleanly with the foreground title fix

---

## Coexistence with PROMPT 1785

PROMPT 1785 (`4ffec0df`) modifies `tools/autoplay/validate_composite_run.py` (BOM-tolerant
`autoplay-run-path.txt` reading). PROMPT 1786 modifies `tools/autoplay/win_foreground.py`
and its test file. No file overlap. Zero conflicts. All 79 tests green.

---

## Commits on Integration Branch

```
40e6c48b fix(autoplay): PROMPT 1786 — repair foreground window title discovery for Lanes and Lies
4ffec0df fix(autoplay): PROMPT 1785 — BOM-tolerant read of autoplay-run-path.txt in validator  ← origin/main
```

---

1787: AUTOPLAY-FOREGROUND-WINDOW-TITLE-MAINLAND-REFRESH-AFTER-1785: READY_FOR_MAINLAND_ENQUEUE
