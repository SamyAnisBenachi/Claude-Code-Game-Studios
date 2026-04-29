---
name: Test evidence type rules for this project
description: What test evidence is required per story type, and project-specific no-mocks rule
type: project
---

Test evidence requirements enforced in this project (from CLAUDE.md coding standards):

| Story Type | Required Evidence | Gate Level |
|---|---|---|
| Logic (formulas, state machines) | Automated unit test in tests/unit/[system]/ | BLOCKING |
| Integration (multi-system) | Integration test OR documented playtest | BLOCKING |
| Visual/Feel | Screenshot + lead sign-off in production/qa/evidence/ | ADVISORY |
| UI | Manual walkthrough doc OR interaction test | ADVISORY |
| Config/Data | Smoke check pass | ADVISORY |

**No mocks rule**: Tests must use real ECS World state and real structs/resources. "Mock" resources are not acceptable for Logic or Integration stories. Use fixture-builder helper functions (e.g., fn make_catalog(...) -> CardCatalog) instead.

**Bevy test patterns**: Logic stories use pure Rust #[test] functions or World::new(). Integration stories use App::new() with relevant plugins registered. No bevy_egui, no full client/server stack.

**Why:** The project was explicitly designed with "no mocks" — test against real ECS World state per technical-preferences.md. This was validated and is a hard project constraint, not a preference.

**How to apply:** When reviewing any Logic or Integration story AC that says "mock X resource", flag it and require a fixture-built real resource instead.
