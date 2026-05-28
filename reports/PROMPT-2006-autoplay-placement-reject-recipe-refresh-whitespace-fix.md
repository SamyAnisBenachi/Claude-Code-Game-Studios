# PROMPT 2006 — Autoplay Placement-Reject Recipe Refresh Whitespace Fix

**Date:** 2026-05-28
**Branch:** work/PROMPT-2006-autoplay-placement-reject-whitespace-fix
**Commit:** b196e72a
**Base origin/main SHA:** f16d60416651cbbaa9443ec76da25fae2f552af9

## Summary

Fixed trailing whitespace on lines 3, 4, and 5 of
`reports/PROMPT-2004-autoplay-placement-reject-recipe-refresh-after-1980.md`
that blocked `git diff --check` on the PROMPT-2004 mainland enqueue.
The whitespace was Markdown line-break `  ` (two trailing spaces) on the
`**Date:**`, `**Branch:**`, and `**Commit:**` header lines.

## Fix Applied

Stripped trailing spaces from three lines in the PROMPT-2004 report file.
No other files were modified. All PROMPT-2004 payload files carried unchanged.

## Validation

### git diff --check
```
(no output — exit 0)
PASS: git diff --check clean
```

### Strict-FF over origin/main
```
PASS: strict-FF over origin/main (git merge-base --is-ancestor exit 0)
```

### File set vs origin/main (no deletions)
```
A  reports/PROMPT-1928-autoplay-placement-reject-recipe-refresh-after-1912.md
A  reports/PROMPT-1952-autoplay-placement-reject-recipe-refresh-after-1937.md
A  reports/PROMPT-1960-autoplay-placement-reject-recipe-refresh-after-1920.md
A  reports/PROMPT-1977-autoplay-placement-reject-recipe-refresh-after-1972.md
A  reports/PROMPT-1990-autoplay-placement-reject-recipe-refresh-after-1988.md
A  reports/PROMPT-1999-autoplay-placement-reject-recipe-refresh-after-1994.md
A  reports/PROMPT-2004-autoplay-placement-reject-recipe-refresh-after-1980.md
M  tests/tools/autoplay/test_recipe_static.py
M  tools/autoplay/recipes/__init__.py
M  tools/autoplay/recipes/_coords.py
A  tools/autoplay/recipes/placement_reject_probe.py
```

All entries are from the PROMPT-2004 allowlist. Zero deletions.

### pytest test_recipe_static.py
```
83 passed in 0.11s
```

## PROMPT 1980 Reports Preserved

The PROMPT-2004 payload explicitly does NOT delete any PROMPT-1980-era
viewport/window-guard reports. The diff confirms zero `D` (deletion) entries;
all entries are additions or modifications from the PROMPT-2004 allowlist.

2006: AUTOPLAY-PLACEMENT-REJECT-RECIPE-REFRESH-WHITESPACE-FIX: READY_FOR_MAINLAND_ENQUEUE
