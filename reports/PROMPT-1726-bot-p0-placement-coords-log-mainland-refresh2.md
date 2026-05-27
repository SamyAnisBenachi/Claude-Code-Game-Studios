# PROMPT 1726 — BOT-P0-A-PLACEMENT-COORDS-LOG-MAINLAND-REFRESH2

**Date**: 2026-05-27  
**Operator**: Claude Sonnet 4.6

---

## Why 1724 and 1725 Failed

### PROMPT 1724 failure
The `integrate/bot-placement-coords-log-1724` branch was based on an older
`origin/main` SHA. By the time MAINLAND_ENQUEUE ran, `origin/main` had advanced
(PROMPT 1713 doc commit `4685e508` landed on main), making the branch non-FF.

### PROMPT 1725 failure
The `integrate/bot-placement-coords-log-1725` branch was rebased onto
`4685e508`, but `origin/main` advanced again before the queue executed —
PROMPT 1725 report commit `3a4f7721` landed on main first. Non-FF again.

---

## This Refresh (PROMPT 1726)

### Base SHA (fetched `origin/main`)
```
3a4f772113c0a49fbdf37df7bbac51bfd99359ac
  docs(integration): PROMPT 1725 — bot-placement-coords-log mainland refresh report
```

### Branch
`integrate/bot-placement-coords-log-1726`

### Branch tip SHA
```
a0719f5c5a825bb419bf28f60d5f2e39369ce3c0
  feat(bot): PROMPT 1719 — add placement coord logging to BotDecisionKind
```

### Payload source
Cherry-picked from `f71761d9` (the PROMPT 1719 implementation commit from
`origin/wt-1719-placement-coords-log`).

---

## Validation

### Path allowlist review
All 7 changed files are within the declared owned scope:
```
server/src/feature/bot/action_loop.rs
server/src/feature/bot/debug_push.rs
server/src/feature/bot/mod.rs
server/src/feature/bot/qa_snapshot.rs
server/src/feature/bot/state.rs
tests/unit/bot/bot_debug_push_test.rs
tests/unit/bot/bot_placement_wave_3_test.rs
```
No forbidden files (production/, Cargo.toml, CI, unrelated modules) touched.

### `git diff --check`
PASS — no whitespace errors.

### Merge-base check
```
git merge-base --is-ancestor origin/main HEAD → PASS
```
`origin/main` (`3a4f772113c0...`) IS ancestor of branch tip (`a0719f5c5a...`).
Branch is strictly FF-mergeable into `origin/main`.

### Push
Branch pushed to `origin/integrate/bot-placement-coords-log-1726` — confirmed
new branch created on remote.

---

## Payload Summary

The cherry-picked commit extends `BotDecisionKind::PlacementSubmitted` with
`coords: Vec<PlacementCoord>` (lane, cell, card_id, mana fields), propagates
coordinates through `qa_snapshot.rs` → `DecisionKindSnapshot`, and formats
them in `debug_push.rs`. Tests in `bot_placement_wave_3_test.rs` and
`bot_debug_push_test.rs` assert the coord fields are present.

---

## Status

**READY_FOR_MAINLAND_ENQUEUE**

Enqueue command:
```
git checkout main && git merge --ff-only integrate/bot-placement-coords-log-1726
```

---

1726: BOT-P0-A-PLACEMENT-COORDS-LOG-MAINLAND-REFRESH2: SHIPPED
