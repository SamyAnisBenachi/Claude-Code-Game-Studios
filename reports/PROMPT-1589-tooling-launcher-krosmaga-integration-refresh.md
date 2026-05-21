# PROMPT-1589 — TOOLING-LAUNCHER-KROSMAGA-INTEGRATION-REFRESH

## Scope

Combine two tooling-only worker payloads onto latest `origin/main`:

- PROMPT 1584 — `origin/work/dev-launcher-job-log-tail-panel-1584` @
  `be42ec4962dc3d09803826a1c24d6d6c0f89a2e7` (Last Job Tail panel in
  `tools/dev-launcher-app`).
- PROMPT 1585 — `origin/work/krosmaga-dev-proxy-validator-coverage-1585` @
  `483ff31d` (extends `tools/asset-provenance/validate_dev_proxy_pack.py`
  coverage; adds fixtures and pytest cases).

Base: `origin/main` @ `9be8827fbd22b2a49d973ba585b5d210fdc8a903` (verified
after `git fetch --all --prune`; no advance since prompt issued).

Krosmaga assets remain dev-proxy only — no release/legal claim introduced.

## Worktree / Branch

- Worktree: `D:/Tmp/wt-1589`
- Branch: `integrate/tooling-launcher-krosmaga-1589`
- Created from: `origin/main` (`9be8827f`)
- Tip after both merges: `1a3f3bf4`

Integration commits on top of main:

```
1a3f3bf4 PROMPT-1589 integrate: merge 1585 krosmaga dev-proxy validator coverage extension
ed054932 PROMPT-1589 integrate: merge 1584 dev-launcher job log tail panel
483ff31d PROMPT-1585 report: record push outcome
dd55ac3b PROMPT-1585 extend dev-proxy validator coverage for Stage 3 readiness
be42ec49 PROMPT-1584 dev launcher: add Last Job Tail panel surfacing last 20 lines of script output
```

Both merges performed with `--no-ff` to keep payload provenance traceable.
No conflicts (1584 touched `tools/dev-launcher-app/**`, 1585 touched
`tools/asset-provenance/**`; disjoint trees).

## Combined Diff vs origin/main

```
 reports/PROMPT-1584-dev-launcher-job-log-tail-panel.md                          | 243 +++++++++++
 reports/PROMPT-1585-krosmaga-dev-proxy-validator-coverage-extension.md          | 150 +++++++
 tools/asset-provenance/README.md                                                |  32 ++
 tools/asset-provenance/fixtures/dev-proxy-pack-atlas-binding-bad.json           |  47 +++
 tools/asset-provenance/fixtures/dev-proxy-pack-stage3-candidate.json            | 109 +++++
 tools/asset-provenance/test_validate_dev_proxy_pack.py                          | 469 +++++++++++++++++++++
 tools/asset-provenance/validate_dev_proxy_pack.py                               | 372 +++++++++++++++-
 tools/dev-launcher-app/src/main.rs                                              | 212 +++++++++-
 8 files changed, 1623 insertions(+), 11 deletions(-)
```

## Scope Allowlist Review (PASS)

Every changed path is in the owned scope:

| Path | Allowed by 1589 scope? | Owner payload |
|---|---|---|
| `tools/dev-launcher-app/src/main.rs` | yes (`tools/dev-launcher-app/**`) | 1584 |
| `reports/PROMPT-1584-…md` | yes (carry report) | 1584 |
| `tools/asset-provenance/validate_dev_proxy_pack.py` | yes (`tools/asset-provenance/**`) | 1585 |
| `tools/asset-provenance/test_validate_dev_proxy_pack.py` | yes | 1585 |
| `tools/asset-provenance/fixtures/dev-proxy-pack-*.json` | yes | 1585 |
| `tools/asset-provenance/README.md` | yes | 1585 |
| `reports/PROMPT-1585-…md` | yes (carry report) | 1585 |

No edits under any forbidden tree (`client/**`, `server/**`, `shared/**`,
`production/**`, Cargo workspace files, CI files, `assets/dev-proxy/manifest*.json`).
The 1585 payload did not touch `assets/dev-proxy/manifest*.json`, so the
conditional allowance for that path was not exercised.

## Validation Performed

- `git fetch --all --prune` → confirmed base `origin/main@9be8827f` current.
- `git diff --check origin/main..HEAD` → clean (no whitespace/conflict markers).
- `git merge-base --is-ancestor origin/main HEAD` → **HEAD descends from
  origin/main**; branch is FF-ready against current `origin/main`.
- Path allowlist review → PASS (table above).
- Focused pytest (cheap):
  `python -m pytest tools/asset-provenance/test_validate_dev_proxy_pack.py -q`
  → `62 passed in 0.41s`.
- Launcher-app Rust build/test deliberately deferred to VERIFY lane (the
  prompt restricts broad Cargo and asks for only cheap focused checks; a
  full `cargo test -p` against `tools/dev-launcher-app` would compile the
  whole launcher tree and the rest of the local workspace and is not
  considered cheap on this machine).

## Push Outcome

Push attempt to `origin` for branch
`integrate/tooling-launcher-krosmaga-1589` is left to relay (the orchestrator
typically performs the push). The branch and tip commit are:

- Branch: `integrate/tooling-launcher-krosmaga-1589`
- Tip: `1a3f3bf4`

Main was **not** touched; only the integration branch was created locally.

## FF-Readiness Statement

Yes — `integrate/tooling-launcher-krosmaga-1589` (`1a3f3bf4`) is
fast-forward-ready against `origin/main` (`9be8827f`) at report time. No
rebase required.

## Final line

1589: TOOLING-LAUNCHER-KROSMAGA-INTEGRATION-REFRESH: SHIPPED
