# PROMPT 1570 — KROSMAGA-DEV-PROXY-STAGE2-MAIN-READY-REFRESH-AFTER-1564

**Status:** READY_FOR_MAINLAND_ENQUEUE
**Base (current main):** `origin/main@5be95a9b8c2375c8e704efe2460501d20110c82d`
(PROMPT 1564 bot participant action-loop main-ready refresh)
**Source branch:** `origin/integrate/krosmaga-dev-proxy-stage2-1567 @ ab025b08715e52c81d393d6f238b388e48e8a29e`
**Refreshed branch:** `integrate/krosmaga-dev-proxy-stage2-1570`
**Worktree:** `D:/tmp/wt-1570`

## Why the refresh

PROMPT 1567 declared `READY_FOR_MAINLAND_ENQUEUE` over
`origin/main@f19ab3ea`, but PROMPT 1564 landed the bot participant
action-loop wave-1 mainland chain (1541/1549/1560/1564) onto current main at
`5be95a9b`. The source branch
`origin/integrate/krosmaga-dev-proxy-stage2-1567` still carried the pre-bot
state of `server/src/feature/bot/action_loop.rs`, `server/src/feature/bot/mod.rs`,
`server/src/main.rs`, and the 1541/1549/1560/1564 bot reports — so a direct
FF was not possible:

```
git merge-base --is-ancestor origin/main origin/integrate/krosmaga-dev-proxy-stage2-1567
=> NOT_FF_READY
```

This refresh rebases the **Krosmaga dev-proxy Stage-2 payload only** onto
current `origin/main@5be95a9b`. The bot action-loop server files and bot
reports already on main are left untouched. No mainland push performed.

## Payload reapplied onto current main

Owned scope from PROMPT 1534/1539/1545/1559/1563/1567 reapplied verbatim:

```
M	tools/asset-provenance/README.md
M	tools/asset-provenance/fixtures/dev-proxy-pack-clean.json
A	tools/asset-provenance/fixtures/dev-proxy-pack-bad-logical-id.json
A	tools/asset-provenance/fixtures/dev-proxy-pack-stage2-candidate.json
M	tools/asset-provenance/test_validate_dev_proxy_pack.py
M	tools/asset-provenance/validate_dev_proxy_pack.py
A	reports/PROMPT-1534-krosmaga-dev-proxy-pack-materialization-stage2.md
A	reports/PROMPT-1539-krosmaga-dev-proxy-stage2-integration-refresh.md
A	reports/PROMPT-1545-krosmaga-dev-proxy-stage2-integration-refresh-after-state-commits.md
A	reports/PROMPT-1559-krosmaga-dev-proxy-stage2-main-ready-refresh.md
A	reports/PROMPT-1563-krosmaga-dev-proxy-stage2-main-ready-refresh-after-1557.md
A	reports/PROMPT-1567-krosmaga-dev-proxy-stage2-main-ready-refresh-after-1561.md
A	reports/PROMPT-1570-krosmaga-dev-proxy-stage2-main-ready-refresh-after-1564.md
```

The pre-bot deletions of `server/src/feature/bot/action_loop.rs`,
`server/src/feature/bot/mod.rs`, `server/src/main.rs`, and the deletions of
`reports/PROMPT-1531/1541/1549/1560/1564-bot-*.md` and
`reports/PROMPT-1554-result-mulligan-*.md` that the source branch carried
were **not** reapplied — those represent stale pre-PROMPT-1531 state.

## Allowlist compliance

All staged paths fall under the PROMPT 1570 owned-scope allowlist:

- `tools/asset-provenance/**`
- `reports/PROMPT-153{4,9}-*.md`, `reports/PROMPT-15{45,59,63,67,70}-*.md`

No edits to: `production/sprint-status.yaml`, `production/session-state/**`,
`production/sprints/**`, `production/qa/**`, `production/stage.txt`,
`Cargo.toml`, CI configs, or any source/server files.

## Validation

- `git diff --check` (cached): clean.
- Focused Python tests:
  ```
  D:/_APPS/Python312/python.exe -m pytest tools/asset-provenance/test_validate_dev_proxy_pack.py -q
  ......................... 25 passed in 0.34s
  ```
- After commit:
  ```
  git merge-base --is-ancestor origin/main HEAD => IS_FF_READY
  ```
- Broad Cargo verification intentionally deferred to VERIFY lanes per user
  policy (Krosmaga payload is Python tooling + reports only; touches no Rust
  source).

## Krosmaga claims unchanged

Krosmaga assets remain dev-proxy only. No release/legal claims introduced;
all fixtures stay `dev_only: true` with `release_class: "dev_only"` and the
`Dev-only Krosmaga proxy; not release-approved.` license/provenance warning
preserved from the source branch.

## Result

`integrate/krosmaga-dev-proxy-stage2-1570` is strict-FF eligible over
current `origin/main@5be95a9b` and ready for MAINLAND_ENQUEUE.

1570: KROSMAGA-DEV-PROXY-STAGE2-MAIN-READY-REFRESH-AFTER-1564: READY_FOR_MAINLAND_ENQUEUE
