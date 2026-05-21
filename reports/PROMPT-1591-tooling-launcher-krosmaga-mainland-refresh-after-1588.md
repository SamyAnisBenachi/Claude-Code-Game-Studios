# PROMPT 1591 — Tooling Launcher + Krosmaga Validator Mainland Refresh After 1588

## Context

- Source-of-truth `origin/main` advanced from `9be8827f` to `945cbd71` when
  PROMPT 1588 (bot Wave 2 integration refresh) landed.
- PROMPT 1589 integration branch
  `origin/integrate/tooling-launcher-krosmaga-1589 @ aa2d9187` was strict-FF
  ready against the previous `origin/main@9be8827f` but lost FF-eligibility once
  1588 landed (it does not contain Wave 2 bot commits 1582/1583/1588).
- This refresh re-layers the same PROMPT 1584 launcher log-tail panel and
  PROMPT 1585 Krosmaga dev-proxy validator coverage payloads on top of the new
  mainland tip without re-merging the stale 1589 integration history.

## Source-of-truth and worker

- Base `origin/main` tip: `945cbd71` — `PROMPT-1588 bot Wave 2 integration
  refresh report (1582+1583 layered on 9be8827f)`.
- Refresh branch: `integrate/tooling-launcher-krosmaga-1591`.
- Worktree: `D:/Tmp/wt-1591`.
- Previous (stale) integration branch left intact for traceability:
  `origin/integrate/tooling-launcher-krosmaga-1589 @ aa2d9187`.

## Method

1. New branch `integrate/tooling-launcher-krosmaga-1591` created from current
   `origin/main` in an isolated worktree.
2. Cherry-picked the three non-merge tooling commits from the 1589 chain
   (skipping the two merge commits and the 1589-specific integration report,
   since this refresh authors its own):
   - `be42ec49` → `1a2413cf` — PROMPT 1584 launcher Last Job Tail panel.
   - `dd55ac3b` → `0eba48d9` — PROMPT 1585 dev-proxy validator coverage
     extension (Stage 3 readiness).
   - `483ff31d` → `be27b87c` — PROMPT 1585 report push-outcome amendment.
3. Authored this PROMPT 1591 report (this file) on top.

No conflicts during cherry-pick: the 1584/1585 changes touch only
`tools/dev-launcher-app/`, `tools/asset-provenance/`, and `reports/`, which do
not overlap with the bot Wave 2 deltas (`server/`, `tests/unit/bot/`) that
landed via PROMPT 1588.

## Final branch shape

```
integrate/tooling-launcher-krosmaga-1591 = origin/main (945cbd71)
  + 1a2413cf  PROMPT-1584 dev launcher: add Last Job Tail panel
  + 0eba48d9  PROMPT-1585 extend dev-proxy validator coverage for Stage 3
  + be27b87c  PROMPT-1585 report: record push outcome
  + <this commit>  PROMPT-1591 mainland refresh report
```

`git merge-base --is-ancestor origin/main HEAD` → exit 0 (strict-FF confirmed).

## Owned-scope path allowlist review

`git diff --name-status origin/main..HEAD`:

```
A  reports/PROMPT-1584-dev-launcher-job-log-tail-panel.md
A  reports/PROMPT-1585-krosmaga-dev-proxy-validator-coverage-extension.md
A  reports/PROMPT-1591-tooling-launcher-krosmaga-mainland-refresh-after-1588.md
M  tools/asset-provenance/README.md
A  tools/asset-provenance/fixtures/dev-proxy-pack-atlas-binding-bad.json
A  tools/asset-provenance/fixtures/dev-proxy-pack-stage3-candidate.json
M  tools/asset-provenance/test_validate_dev_proxy_pack.py
M  tools/asset-provenance/validate_dev_proxy_pack.py
M  tools/dev-launcher-app/src/main.rs
```

All paths fall inside the owned scope (`tools/dev-launcher-app/**`,
`tools/asset-provenance/**`, `reports/`). Nothing touched in `client/`,
`server/`, `shared/`, `production/`, `Cargo.toml`, or CI config.

## Validation

- `git diff --check origin/main..HEAD` → exit 0 (no whitespace errors).
- `git merge-base --is-ancestor origin/main HEAD` → exit 0 (strict-FF).
- Focused tooling test: `python -m unittest test_validate_dev_proxy_pack`
  in `tools/asset-provenance/` → `Ran 62 tests in 0.346s — OK`.
- Broad Cargo runs deferred to a VERIFY lane (out of scope per PROMPT).

## Carried reports

- `reports/PROMPT-1584-dev-launcher-job-log-tail-panel.md` (carried from 1584).
- `reports/PROMPT-1585-krosmaga-dev-proxy-validator-coverage-extension.md`
  (carried from 1585; includes push-outcome amendment).
- `reports/PROMPT-1591-tooling-launcher-krosmaga-mainland-refresh-after-1588.md`
  (this file).

The PROMPT 1589 integration report itself is intentionally NOT carried — its
narrative is now stale because the 1589 branch never landed and the new
ancestry is recorded here instead. The 1589 branch still exists on the remote
for audit reference.

## Push outcome

- `git push -u origin integrate/tooling-launcher-krosmaga-1591` succeeded.
- Remote ref now: `origin/integrate/tooling-launcher-krosmaga-1591 @ a7aa0008`.
- `main` was NOT pushed; only the integration branch was pushed.
- Branch is strict-FF-ready vs `origin/main @ 945cbd71` and ready for
  MAINLAND_ENQUEUE.

## Krosmaga disclaimer

Krosmaga assets remain dev-proxy only. No release, distribution, or legal claim
is implied by these tooling validator changes; they continue to gate fixtures
through provenance metadata only.

1591: TOOLING-LAUNCHER-KROSMAGA-MAINLAND-REFRESH-AFTER-1588: SHIPPED
