---
name: Lanes and Lies — Game Session System Epic
description: 7 stories authored for the game-session-system epic; key blocking conditions, ADR dependencies, and CI gate patterns
type: project
---

Game Session System epic has 7 stories at `production/epics/game-session-system/`. Story 004 (F4 + SessionReady) is **Blocked** pending ADR-012 verification (Commands::trigger ordering invariant in Bevy 0.18 — 4 checklist items). All other 6 stories are Ready.

**Why:** The Observer-trigger path (insert_resource → insert_resource → trigger in same Commands flush) is the highest-risk Bevy 0.18 post-cutoff API in the project. Bevy 0.17 formalized the Event/Observer split; 0.18 is post-LLM-cutoff.

**How to apply:** Any future story touching `SessionReady`, `evaluate_session_ready`, or `on_session_ready` must reference ADR-012 and confirm the verification checklist items 1–4 are documented before implementation begins.

Key CI gate pattern used across this epic:
- `grep -r "EventReader<SessionReady>" server/src/` = 0 (Observer, not buffered)
- `grep -r "app.add_event::<SessionReady>" server/src/` = 0
- `grep -r "app.observe(on_session_ready" server/src/` = exactly 1

Story 007 (Reconnect Snapshot) has the widest dependency set (6 stories + 3 epics + all 14 ADR-011 checklist items). Sequence last in sprint planning.

Manifest Version used: 2026-04-29.
