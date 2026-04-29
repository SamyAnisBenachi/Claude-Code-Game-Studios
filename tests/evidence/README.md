# Test Evidence

Manual test sign-off records for Visual/Feel and UI story types.

## When to add evidence here

- A **Visual/Feel** story is Done: add a screenshot + note confirming the
  visual result matches the art bible / UX spec
- A **UI** story is Done without an automated test: add a manual walkthrough
  document confirming the interaction works as specified

## Naming

- Screenshots: `[date]-[system]-[feature].png`
  Example: `2026-05-01-board-rendering-unit-sprites.png`
- Walkthrough docs: `[date]-[system]-[feature]-walkthrough.md`
  Example: `2026-05-01-hand-ui-card-selection-walkthrough.md`

## Format for walkthrough docs

```markdown
# [System] — [Feature] Manual Test

**Date**: YYYY-MM-DD
**Story**: [story file path]
**Tester**: [name]
**Build**: [git commit hash]

## Steps Taken
1. [what was done]
2. [what was done]

## Expected Result
[what should have happened]

## Actual Result
[what actually happened]

## Screenshots
[attach or reference images]

## Verdict: PASS / FAIL
```
