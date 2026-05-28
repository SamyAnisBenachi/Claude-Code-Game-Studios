# PROMPT 1920 — Krosmaga Card Inspect Hover Glossary Refresh After 1912

**Date:** 2026-05-28
**Branch:** `prompt-1920-card-inspect-glossary-refresh-after-1912`
**Worktree:** `D:/tmp/wt-1920-card-inspect-glossary`
**Based on:** `origin/main` @ `1c945fd2` (PROMPT 1912 whitespace cleanup)
**Status:** SHIPPED

---

## Context

`origin/prompt-1868-card-inspect-glossary-refresh` shipped the Krosmaga keyword
glossary panel feature (PROMPT 1852/1868), but became **NOT_FF** over
`origin/main` after PROMPT 1894 and 1912 landed (autoplay viewport guard work).
Direct fast-forward merge would have deleted recent autoplay report/tooling
artifacts.

This prompt reapplies the same payload cleanly onto `origin/main` post-1912
without touching any PROMPT 1894/1912 artifacts.

---

## Approach

1. Created a dedicated worktree from `origin/main`:
   ```
   git worktree add D:/tmp/wt-1920-card-inspect-glossary \
     -b prompt-1920-card-inspect-glossary-refresh-after-1912 origin/main
   ```
2. Identified the two owned commits on `origin/prompt-1868-card-inspect-glossary-refresh`
   not yet on main:
   - `a55181d0` feat(ui/card-inspect): PROMPT 1852 — add keyword glossary definitions panel
   - `3be567ac` docs(reports): PROMPT 1868 — card inspect hover glossary integration refresh report
3. Cherry-picked both commits — zero conflicts (all touched files are disjoint from
   PROMPT 1894/1912 autoplay changes).
4. Fixed trailing whitespace in the PROMPT 1868 report (Markdown `  ` line-break
   sequences on the date/branch header lines) and amended the report commit.

---

## Changes Applied

**`client/src/ui/card_inspect.rs`** (+64 lines):
- `CARD_INSPECT_GLOSSARY_FONT_PX: f32 = 11.0` constant
- `keyword_glossary: Vec<(String, String)>` field on `CardInspectView`
- `glossary_panel: Entity` field on `CardInspectEntities`
- `CardInspectGlossaryPanel` / `CardInspectGlossaryRow` marker components
- `card_inspect_glossary_panel_node()` / `card_inspect_glossary_row_node()` helpers
- `spawn_card_inspect` conditionally spawns the glossary panel (one row per keyword,
  muted text, top-border separator)

**`client/src/ui/hand/inspect.rs`** (+86 lines):
- `keyword_glossary_definition(keyword: &Keyword) -> String` — maps every Keyword
  variant to a short player-readable definition
- `simple_keyword_definition(keyword: SimpleKeyword) -> &'static str` — 20 simple
  keyword definitions
- `build_card_inspect_view_from_card` populates `keyword_glossary` from `data.keywords`
- Two unit tests:
  - `glossary_entries_non_empty_for_keyworded_minion` — 2-keyword minion → 2 entries,
    non-empty labels and definitions
  - `glossary_empty_for_keyword_free_card` — keyword-free spell → empty glossary

**`reports/PROMPT-1868-krosmaga-card-inspect-hover-glossary-integration-refresh.md`**
- Carried forward from the 1868 branch; trailing whitespace on header lines fixed
  (cosmetic only, no content change).

---

## Validation

```
git merge-base --is-ancestor origin/main HEAD  → PASS (branch is FF-ready)
git diff --check origin/main..HEAD             → PASS (no whitespace errors)
git diff --name-status origin/main..HEAD:
  M  client/src/ui/card_inspect.rs
  M  client/src/ui/hand/inspect.rs
  A  reports/PROMPT-1868-krosmaga-card-inspect-hover-glossary-integration-refresh.md
```

No deletions. PROMPT 1894 and 1912 artifacts (`tools/autoplay/**`,
`production/qa/evidence/**`, `reports/PROMPT-1894-*.md`, `reports/PROMPT-1912-*.md`)
are untouched.

### Unit Tests

The two glossary unit tests in `client/src/ui/hand/inspect.rs` were introduced in
the cherry-picked commit and are present on this branch. Full `cargo test` not run
(no broad Cargo per task rules); tests are documented in the commit message and
verified present via source inspection.

---

## Commits

```
51d7ea56  feat(ui/card-inspect): PROMPT 1852 — add keyword glossary definitions panel
714ae864  docs(reports): PROMPT 1868 — card inspect hover glossary integration refresh report
```

---

## Push

Branch pushed to `origin/prompt-1920-card-inspect-glossary-refresh-after-1912`.
FF-merge to main is unblocked.

---

1920: KROSMAGA-CARD-INSPECT-HOVER-GLOSSARY-REFRESH-AFTER-1912: SHIPPED
