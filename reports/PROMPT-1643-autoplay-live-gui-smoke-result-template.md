# PROMPT 1643 — Autoplay Live GUI Smoke Result Template

**Date:** 2026-05-27
**Baseline:** origin/main@e67a3488 (PROMPT 1636)
**Executor:** Claude Code (Sonnet 4.6)
**Worktree:** D:/tmp/wt-1643-smoke-template

---

## Summary

Created a durable fillable template for recording live GUI autoplay smoke
results. The template lives alongside the existing evidence-operator-guide in
`docs/autoplay/` and the operator guide now links to it.

---

## Files Changed

| File | Action |
|---|---|
| `docs/autoplay/live-gui-smoke-result-template.md` | Created — fillable Markdown template |
| `docs/autoplay/evidence-operator-guide.md` | Updated — added §9 with link to template |
| `reports/PROMPT-1643-autoplay-live-gui-smoke-result-template.md` | Created — this report |

---

## Template Coverage

The template (`docs/autoplay/live-gui-smoke-result-template.md`) includes the
following sections, matching every field specified in the PROMPT 1643 task:

| Section | Required? | Covers |
|---|---|---|
| Run Metadata | required | Date, operator, artifact path, commit, branch |
| Command | required | Exact `pwsh -File Run-AutoplaySmoke.ps1 ...` invocation |
| Environment Variables | required | All `CCGS_*` env vars set for the run |
| launcher-status.json Summary | required | Top-level verdict + exit codes |
| Recipe Checkpoint Log | required | `checkpoints.jsonl` paste + per-checkpoint ✓/✗ table |
| Screenshots | required | File list with checkpoint, description, anomalies |
| Bot Decision Log | optional | JSONL path, entry count, decision sanity |
| QA Snapshot | optional | Snapshot file, phase at capture, ECS state assessment |
| Bot Debug Overlay (F8) | optional | Overlay visibility, hand/bid valuation display |
| Verdict | required | PASS / FAIL / BLOCKED / PARTIAL + exit code table + one-liner |
| Failure Analysis | required if FAIL | First missing checkpoint, log excerpt, root cause |
| Follow-Up Prompts | required | Numbered action table linked to ORCHESTRATOR-QUEUE |

---

## Operator Guide Link

Added Section 9 to `docs/autoplay/evidence-operator-guide.md`:

> **9. Recording a Run Result**
> → live-gui-smoke-result-template.md
> Copy it to `production/qa/evidence/autoplay-runs/<run-stamp>/RESULT.md` and
> complete every `<!-- fill -->` field.

---

## Validation

- `git diff --check`: exit 0 (no trailing-whitespace or mixed-indent issues)
- Static link: `evidence-operator-guide.md` links to `live-gui-smoke-result-template.md`
  (same directory — relative link correct)
- Template links back to `evidence-operator-guide.md` for cross-reference

---

## Notes

- No source code, Cargo files, or tools/autoplay scripts were modified.
- `production/session-state/**` not touched.
- Sprint status not touched.
- `docs/autoplay/evidence-operator-guide.md` was present at HEAD e67a3488
  (materialized by PROMPT 1637); the §9 addition is additive only.

---

`1643: AUTOPLAY-LIVE-GUI-SMOKE-RESULT-TEMPLATE: SHIPPED`
