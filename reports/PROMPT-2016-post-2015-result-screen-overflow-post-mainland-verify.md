# PROMPT-2016 — POST-2015-RESULT-SCREEN-OVERFLOW-POST-MAINLAND-VERIFY

**Branch**: `work/PROMPT-2016`
**Base verified**: `origin/main @ a2f5b3ef`
**Date**: 2026-05-28

---

## 1. origin/main tip check

**PASS.** `origin/main` is at `a2f5b3ef`:

```
a2f5b3ef feat(ui/result-screen): PROMPT 2015 — reapply result-screen 720px overflow scroll guard onto main after 2013
```

This is the expected PROMPT 2015 tip commit.

---

## 2. ResultScreenScrollPane presence

**PASS.** `git show origin/main:client/src/presentation/result_screen.rs` confirms:

- Line 254–258: `ResultScreenScrollPane` doc comment and struct definition present
- `overflow_y: scroll` node configured in the scroll pane builder (lines 841, 867, 897, 1004)

```
/// top and the pinned actions row; enables `overflow_y: scroll` so content
...
pub struct ResultScreenScrollPane;
```

---

## 3. Scroll pane test presence

**PASS.** `git show origin/main:tests/integration/presentation/result_screen_chrome_polish_test.rs` confirms:

- Line 174: `fn scroll_pane_enables_overflow_scroll_so_content_reachable_on_720p()` present

---

## 4. PROMPT 2013 artifact integrity

All three PROMPT 2013 artifacts confirmed present on `origin/main`:

| Artifact | Status |
|---|---|
| `tools/autoplay/recipes/placement_reject_probe.py` | ✅ PRESENT |
| `tests/tools/autoplay/test_recipe_static.py` | ✅ PRESENT |
| `reports/PROMPT-2013-autoplay-placement-reject-recipe-refresh-after-2009.md` | ✅ PRESENT |

No regression of PROMPT 2013 artifacts.

---

## 5. Bevy 0.18 static check

**PASS.** `git show origin/main -- client/src/presentation/result_screen.rs tests/integration/presentation/result_screen_chrome_polish_test.rs` diff lines checked for:

- `EventReader` — not found
- `EventWriter` — not found
- `Events<` — not found
- `add_event` — not found

PROMPT 2015 diff is clean of legacy pre-0.16 event API patterns.

---

## 6. cargo test — focused test run

**PARTIAL.** `cargo test -p client --test result_screen_chrome_polish_test scroll_pane_enables_overflow_scroll_so_content_reachable_on_720p`

- First attempt: ran against pre-rebase worktree (PROMPT 2007 branch — missing PROMPT 2015 test); compiled in 4m 30s; showed 5 tests, scroll_pane test filtered (not yet in that binary).
- Worktree rebased to `origin/main @ a2f5b3ef` (patch already upstream, commit dropped cleanly).
- Second compilation attempt: **killed with exit code 137 (OOM)** — system memory exhausted during recompile.
- Direct binary execution: binary locked from killed process (`Device or resource busy`).

**Blocker**: Cargo compilation OOM on this machine during recompile after rebase. Test function confirmed present in source at line 174 via `git show`; cannot execute binary to confirm runtime PASS.

---

## 7. Summary verdict

| Check | Result |
|---|---|
| origin/main at PROMPT 2015 tip | ✅ PASS |
| ResultScreenScrollPane in source | ✅ PASS |
| scroll_pane test function in source | ✅ PASS |
| PROMPT 2013 artifacts intact | ✅ PASS |
| Bevy 0.18 static clean | ✅ PASS |
| cargo test binary execution | ⚠️ PARTIAL — OOM/binary-locked; source confirmed |

All source and git-level checks pass. Cargo test execution blocked by OOM; test function presence confirmed via `git show`.

---

2016: POST-2015-RESULT-SCREEN-OVERFLOW-POST-MAINLAND-VERIFY: PARTIAL
