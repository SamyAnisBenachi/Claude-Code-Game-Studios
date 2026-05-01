---
name: prism-system QL-STORY-READY review
description: QL-STORY-READY verdicts for Prism System Stories 001-010 (2026-05-01)
type: project
---

QL-STORY-READY review for Prism System epic (Feature M3), all 10 proposed stories.
Conducted 2026-05-01 against GDD design/gdd/prism-system.md (Pass 2, 2026-04-30).

| Story | Type | Verdict |
|-------|------|---------|
| 001 | Logic | GAPS — PS-15 untestable in Story 001 scope (reward path not yet implemented) |
| 002 | Logic | ADEQUATE — coordination note: PS-12 missing from submission; NP OQ1 ADVISORY on PS-20 |
| 003 | Logic | ADEQUATE |
| 004 | Logic | ADEQUATE — NP OQ1 ADVISORY on staging buffer assertions (PS-09, PS-23) |
| 005 | Logic | ADEQUATE — NP OQ1 ADVISORY on PS-24 staging portion |
| 006 | Logic | ADEQUATE — coordination: PS-12 duplicated with on-disk story-002; TR-ID gap for PS-12 |
| 007 | Logic | GAPS — PS-13 tests Board/Lane System behavior, not Prism System; no automatable unit test target |
| 008 | Config/Data | ADEQUATE — ADVISORY gate; panic! vs AppExit::Error pre-implementation question |
| 009 | Integration | GAPS — incorrect PS-24 cross-reference in PS-04; App::new() harness not specified |
| 010 | Integration | GAPS — App::new() harness not specified; PS-06 self-targeting sub-test missing |

**Blocking actions before story files are written:**
- Story 001: Move PS-15 to Story 002 (or add reward stub to Story 001 scope)
- Story 007: Rewrite PS-13 — test the Prism-System-local invariant (6th stale message goes to DiscardLog after pending_respawn set; respawn fires end-of-function after all buffer messages processed)
- Story 009: Remove incorrect "(see PS-24)" cross-reference; specify App::new() plugin list
- Story 010: Specify App::new() plugin list; require self-targeting sub-test in PS-06

**Coordination notes:**
- PS-12 is in the on-disk story-002 file AND proposed Story 006 — duplication must resolve before both are written
- NP OQ1 (Lightyear 0.26 unicast buffer inspectability) is still open; affects PS-20, PS-09, PS-23, PS-24 staging assertions — all marked ADVISORY until OQ1 closes

**Why:** Shift-left gate; Logic stories need automatable unit tests; Integration stories need App::new() harness spec.
**How to apply:** Do not write Story 001, 007, 009, 010 files until blocking gaps above are resolved.
Stories 002-006, 008 may be written to files with the coordination notes addressed in parallel.
