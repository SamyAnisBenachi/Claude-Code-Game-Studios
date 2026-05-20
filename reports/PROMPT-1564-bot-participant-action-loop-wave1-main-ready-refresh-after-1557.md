# PROMPT 1564 — Bot Participant Action Loop Wave 1 Main-Ready Refresh After 1557

## Summary

Refreshed PROMPT 1560 payload (which carries the PROMPT 1531 bot participant
action-loop Wave 1 work, plus the 1541/1549/1560 evidence reports) onto current
`origin/main@d09d02143c32699caadf858f8d90eb835b11097d` so the result is strict
fast-forward eligible for MAINLAND_ENQUEUE after PROMPT 1557 landed.

## Branches and commits

- Source-of-truth main at refresh time: `origin/main@f19ab3ea2173e6d88658ea22f86563863e189741`
  (main advanced from d09d0214 → f19ab3ea via PROMPT 1547/1561 while this refresh
  was in flight; branch rebased onto the new tip and remains FF-eligible.)
- Previous integration branch: `origin/integrate/bot-participant-action-loop-1560@08b3fad0858ec2e638ee6bed936f1478da7517db` (NOT FF after 1557)
- Refreshed branch: `integrate/bot-participant-action-loop-1564`
- Refreshed HEAD: `855c6e791fc88ccf3cc84648666c2b8aa52844c3` (pushed to `origin/integrate/bot-participant-action-loop-1564`)

## Cherry-pick order (preserves payload exactly)

1. `6f1d7d21` → `d9108808` PROMPT-1531 bot participant action loop wave1
2. `ade6d622` → `363b893a` PROMPT-1541 integration refresh report
3. `66eb6a1c` → `c28b9f82` PROMPT-1549 main-ready refresh report
4. `08b3fad0` → `8479e93e` PROMPT-1560 main-ready refresh report (after 1554)

Plus this 1564 refresh report commit on top.

## Path allowlist review (vs origin/main)

```
reports/PROMPT-1531-bot-participant-action-loop-wave1.md                                | 110 +
reports/PROMPT-1541-bot-participant-action-loop-wave1-integration-refresh.md            |  34 +
reports/PROMPT-1549-bot-participant-action-loop-wave1-main-ready-refresh.md             |  57 +
reports/PROMPT-1560-bot-participant-action-loop-wave1-main-ready-refresh-after-1554.md  |  44 +
server/src/feature/bot/action_loop.rs                                                   | 489 +
server/src/feature/bot/mod.rs                                                           |   2 +
server/src/main.rs                                                                      |   4 +
```

All paths are inside the owned scope declared by the task (PROMPT 1531/1541/1549/1560
payload plus the 1564 refresh report). No forbidden paths
(`production/sprint-status.yaml`, `production/session-state/**`,
`production/sprints/**`, `production/qa/**`, `production/stage.txt`, unrelated
Cargo/CI/source files) are modified.

## Checks

- `git diff --check origin/main HEAD` — clean (no whitespace errors).
- `git merge-base --is-ancestor origin/main HEAD` — TRUE (strict FF eligible).
- Path allowlist — passes.
- Broad Cargo verification deferred to VERIFY lanes per task policy.

## Verdict

`READY_FOR_MAINLAND_ENQUEUE`

1564: BOT-PARTICIPANT-ACTION-LOOP-WAVE1-MAIN-READY-REFRESH-AFTER-1557: READY_FOR_MAINLAND_ENQUEUE
