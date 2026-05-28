# PROMPT 1889 — AUTOPLAY-VSBOT-1841-SIGNOFF-PACK-REFRESH-AFTER-1872

**Date:** 2026-05-28  
**Branch:** wt-1889-signoff-refresh (from origin/main @ 2ce3dc6b)  
**Refreshes:** PROMPT 1841 (origin/wt-1841-signoff-pack @ 71484998)

> **Scope:** Report-only refresh. No source edits. No QA evidence mutation.  
> No Cargo build. Path allowlist: `reports/` only.

---

## Why This Refresh Exists

PROMPT 1841 was originally shipped on branch `origin/wt-1841-signoff-pack`, which
branched from `origin/main @ 71484998`. Since then, main advanced through five
additional report/tooling commits:

| Commit | PROMPT | Summary |
|--------|--------|---------|
| b856eef4 | 1833 | Add analyze_evidence_run.py evidence distinctness analyzer |
| bb90d7c2 | 1844 | Autoplay vs-bot viewport/click-target evidence audit |
| 5c91918d | 1858 | Backfill PROMPT 1845 evidence analyzer verify report |
| (part of 1858) | 1859 | (backfill, co-landed) |
| 2ce3dc6b | 1872 | Reapply PROMPT 1846/1859 analyzer reports over latest main |

Merging `wt-1841-signoff-pack` into current main is **not fast-forwardable** and
would silently delete or overwrite the five above commits' report/tooling files.
This refresh branch solves that by re-applying only the PROMPT 1841 report file on
top of current main, with no destructive side effects.

---

## What Was Added

| File | Action | Source |
|------|--------|--------|
| `reports/PROMPT-1841-autoplay-vsbot-1831-evidence-signoff-pack.md` | Added (new to main) | Verbatim from `origin/wt-1841-signoff-pack` |

No files were modified, deleted, or overwritten. All commits from PROMPT 1833,
1844, 1845, 1846, 1858, 1859, and 1872 are preserved intact.

---

## Path Allowlist Verification

Files touched by this branch (diff vs origin/main):
- `reports/PROMPT-1841-autoplay-vsbot-1831-evidence-signoff-pack.md` ✅ (allowed)
- `reports/PROMPT-1889-autoplay-vsbot-1841-signoff-pack-refresh-after-1872.md` ✅ (allowed)

No source files, no tests, no production/sprint-status.yaml, no session-state,
no stage.txt, no evidence files touched.

---

## Key Finding Preserved from PROMPT 1841

The 1841 report documents **Caveat C0 (BLOCKING)**: a human observation that the
game window opened too small during the autoplay vs-bot run, causing the bot to
click in empty/offscreen space. This is preserved verbatim and remains the
authoritative record that:

- Machine PASS (all 15 checkpoints reached, driver exit 0, capture chain OK)
  **cannot** substitute for human viewport validation.
- AUTOPLAY-VS-BOT-QA-001 requires the operator to complete checklist items 0a–0c
  (viewport size check) before any sign-off can proceed.
- If viewport clipping is confirmed, a viewport-repair story must be opened and
  a new run performed before QA-001 can be advanced.

The evidence analyzer work in PROMPT 1844/1872 further corroborates this: click
coordinate analysis flagged that some clicks landed outside the expected visible
UI region in the early-phase screenshots.

---

## Ancestor Check

Branch `wt-1889-signoff-refresh` is based on `origin/main @ 2ce3dc6b`. Run
`git merge-base --is-ancestor origin/main HEAD` on the branch to confirm.

---

1889: AUTOPLAY-VSBOT-1841-SIGNOFF-PACK-REFRESH-AFTER-1872: SHIPPED
