# PROMPT 1612 -- AUTOPLAY-FULL-GAME-RECIPE-LIBRARY-INTEGRATION-REFRESH

Status: SHIPPED (integration branch ready for mainland enqueue; runtime verify deferred to PROMPT 1613).

## Inputs

- Source branch: `origin/prompt-1609-autoplay-recipe-library`
- Source commit: `87763de24ac77b7cb4dbc6c1c8aa7968054a8c9e`
- Source-commit base: `576fbe8c` (PROMPT 1602 Wave 3 placement heuristic)
- Mainland source-of-truth: `origin/main @ ee9109c8` (PROMPT 1607 integration report)

## Integration branch

- Worktree: `D:/Tmp/wt-1612`
- Branch: `integrate/autoplay-recipe-library-1612`
- Base: `origin/main @ ee9109c8`
- Head: `e6e2d7d2e23b5fc0be29826bee417a44cff3d462`
- Method: cherry-pick `87763de2` onto `origin/main` (clean apply, no conflicts)
- Strict fast-forward readiness: YES (`git merge-base --is-ancestor origin/main HEAD` returns 0; `ee9109c8` is parent of `e6e2d7d2`)

## Why a refresh was needed

PROMPT 1609 was forked from `576fbe8c` (PROMPT 1602) before the following mainlands:
- `51dbb9c0` PROMPT 1603 bot-vs-bot soak entrypoint
- `0cb29c23` autoplay RPC schema alignment
- `237572af` post-1601 autoplay state record
- `ee9109c8` PROMPT 1607 integration refresh report

None of those intervening commits touched the PROMPT 1609 owned scope
(`tools/autoplay/`, `docs/autoplay.md`, `skills/ccgs-autoplay/`), so a
cherry-pick onto current `origin/main` applies cleanly with no semantic
adjustments required. The branch is now a fast-forward over `origin/main`.

## Files in changeset (allowlist review)

All 14 files are within the PROMPT 1609 owned scope:

```
docs/autoplay.md
skills/ccgs-autoplay/SKILL.md
tools/autoplay/README.md
tools/autoplay/driver.py
tools/autoplay/recipes/__init__.py
tools/autoplay/recipes/_builder.py
tools/autoplay/recipes/_coords.py
tools/autoplay/recipes/class_select.py
tools/autoplay/recipes/draft_auction_probe.py
tools/autoplay/recipes/full_game.py
tools/autoplay/recipes/idle.py
tools/autoplay/recipes/lobby_create.py
tools/autoplay/recipes/placement_drag_probe.py
tools/autoplay/recipes/smoke.py
```

Forbidden paths NOT touched: `client/**`, `server/**`, `shared/**`,
`Cargo.toml`, `Cargo.lock`, `production/sprint-status.yaml`,
`production/session-state/**`, `production/sprints/**`, `production/qa/**`.

Semantics preserved: autoplay still drives only low-level keyboard/mouse/cursor
RPCs and screenshot/status. No semantic gameplay-state mutation endpoints
introduced. `full-game` composite recipe still gates on
`CCGS_AUTOPLAY_BOT_ROOM_READY=1` since the PROMPT 1607 bot-vs-bot room flow is
not yet wired through autoplay.

## Validation

| Check | Result |
|---|---|
| Path allowlist review | PASS — every changed path inside `tools/autoplay/**`, `skills/ccgs-autoplay/**`, or `docs/autoplay.md` |
| `git diff --check origin/main..HEAD` | PASS (no whitespace/conflict markers) |
| Strict-FF check (`merge-base --is-ancestor origin/main HEAD`) | PASS |
| `python -m py_compile` (all 11 Python files) | PASS |
| `python tools/autoplay/driver.py --list-recipes` | PASS — 7 recipes registered (class-select, draft-auction-probe, full-game, idle, lobby-create, placement-drag-probe, smoke) |
| Runtime GUI/autoplay smoke | DEFERRED to PROMPT 1613 verify lane per task scope |
| `cargo` build/test | NOT RUN (no Rust files touched; out of refresh scope) |

## Recommended runtime verify command for PROMPT 1613

In a fresh worktree on `integrate/autoplay-recipe-library-1612` (or after
mainland), with a Bevy client launched with autoplay enabled
(`CCGS_AUTOPLAY=1`):

```
python tools/autoplay/driver.py --recipe smoke
```

Then sweep the no-prereq recipes:

```
python tools/autoplay/driver.py --recipe idle
python tools/autoplay/driver.py --recipe lobby-create
python tools/autoplay/driver.py --recipe class-select
python tools/autoplay/driver.py --recipe draft-auction-probe
python tools/autoplay/driver.py --recipe placement-drag-probe
```

`full-game` is expected to emit exit code 4 (BLOCKED) until
`CCGS_AUTOPLAY_BOT_ROOM_READY=1` is honored by the bot-vs-bot room flow; set
the env var only when the upstream wiring lands. Verify
`tools/autoplay/checkpoints.jsonl` is appended per recipe and per checkpoint.

## Mainland enqueue

- Push (when network available):
  ```
  git push -u origin integrate/autoplay-recipe-library-1612
  ```
- Mainland merge: strict fast-forward of `integrate/autoplay-recipe-library-1612` onto `origin/main`.

## Final line

1612: AUTOPLAY-FULL-GAME-RECIPE-LIBRARY-INTEGRATION-REFRESH: SHIPPED
