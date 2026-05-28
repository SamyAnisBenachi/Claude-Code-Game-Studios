# PROMPT 1868 — Krosmaga Card Inspect Hover Glossary Integration Refresh

**Date:** 2026-05-28  
**Branch:** `prompt-1868-card-inspect-glossary-refresh`  
**Status:** SHIPPED

---

## Context

PROMPT 1852 shipped `origin/prompt-1852-card-inspect-glossary` with the keyword
glossary panel feature, but that branch was **not FF-ready** over `origin/main`:
- Merge base: `b856eef4` (PROMPT 1833 — analyze_evidence_run.py)
- `origin/main` advanced to `bb90d7c2` (PROMPT 1844 — autoplay vs-bot evidence audit report)
- Direct land would have deleted the PROMPT 1844 report artifact

This prompt reapplies the PROMPT 1852 payload cleanly onto latest `origin/main`
without touching any PROMPT 1833/1844 artifacts.

---

## Approach

Cherry-picked PROMPT 1852 commit `fa7be22f` onto a fresh branch from `origin/main`:

```
git worktree add D:/tmp/wt-1868-card-inspect-glossary -b prompt-1868-card-inspect-glossary-refresh origin/main
git cherry-pick fa7be22f
```

Cherry-pick succeeded with zero conflicts (touched files are entirely disjoint from
the PROMPT 1844 report commit).

---

## Changes Applied

**`client/src/ui/card_inspect.rs`** (+64 lines):
- Added `CARD_INSPECT_GLOSSARY_FONT_PX: f32 = 11.0` constant
- Added `keyword_glossary: Vec<(String, String)>` field to `CardInspectView`
- Added `glossary_panel: Entity` field to `CardInspectEntities`
- Added `CardInspectGlossaryPanel` and `CardInspectGlossaryRow` marker components
- Added `card_inspect_glossary_panel_node()` and `card_inspect_glossary_row_node()` layout helpers
- In `spawn_card_inspect`: conditionally spawns the glossary panel with one row per
  keyword entry, using muted text + top border separator

**`client/src/ui/hand/inspect.rs`** (+86 lines):
- Added `keyword_glossary_definition(keyword: &Keyword) -> String` — maps each
  keyword variant to a short player-readable definition string
- Added `simple_keyword_definition(keyword: SimpleKeyword) -> &'static str` — 20
  simple keyword definitions
- `build_card_inspect_view_from_card` now populates `keyword_glossary` by iterating
  `data.keywords`, pairing `format_keyword(kw)` labels with definitions
- Added two unit tests:
  - `glossary_entries_non_empty_for_keyworded_minion` — verifies 2 entries for a
    minion with Haste + ResistanceX{2}, checks labels and non-empty definitions
  - `glossary_empty_for_keyword_free_card` — verifies empty glossary for a spell

---

## Validation

```
git diff --check          → clean (no whitespace errors)
git diff --stat origin/main..HEAD → 2 files changed, 150 insertions(+)
```

No deletions. PROMPT 1833 (`tools/autoplay/analyze_evidence_run.py`) and PROMPT
1844 (`reports/PROMPT-1844-*.md`) artifacts are untouched.

---

## Original PROMPT 1852 Report

No `reports/PROMPT-1852-*.md` file existed in the stale branch — the 1852 commit
contained only the two source files. The payload is fully documented in this report.

---

## Commit

```
a55181d0  feat(ui/card-inspect): PROMPT 1852 — add keyword glossary definitions panel
```

1868: KROSMAGA-CARD-INSPECT-HOVER-GLOSSARY-INTEGRATION-REFRESH: SHIPPED
