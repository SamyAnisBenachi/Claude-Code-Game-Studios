# PROMPT-1613 — Autoplay Recipe Library Main-Ready Refresh after PROMPT 1611

## Summary

Refreshed the PROMPT 1609 / PROMPT 1612 autoplay recipe library work onto the
current source-of-truth `origin/main@271f21bc` (PROMPT 1611 bot/autoplay
sprint-story ledger). The new integration branch is strict fast-forward ready
for mainland enqueue and preserves all PROMPT 1611 bot/autoplay ledger files
and notes.

Status: **SHIPPED** — branch built, validated, ready for the next mainland
worker.

## Sources

| Item | Value |
|---|---|
| Source PROMPT 1609 branch | `origin/prompt-1609-autoplay-recipe-library` |
| Source PROMPT 1609 commit | `87763de24ac77b7cb4dbc6c1c8aa7968054a8c9e` |
| Stale PROMPT 1612 branch  | `origin/integrate/autoplay-recipe-library-1612 @ 3b65f7de` (NOT mainlanded; based on `origin/main@ee9109c8` — predates PROMPT 1611) |
| Current source-of-truth   | `origin/main @ 271f21bc` (PROMPT 1611 integration refresh report) |

## New integration branch

| Field | Value |
|---|---|
| Worktree | `D:/Tmp/wt-1613` |
| Branch | `integrate/autoplay-recipe-library-1613` |
| Base    | `271f21bc86c38cea1920ec821036615220242220` (`origin/main`) |
| HEAD    | `fb21b0458f372cfc4da6ffbf1bc24e757661dbd4` |
| Strict-FF onto `origin/main@271f21bc` | YES |

The branch contains a single cherry-picked commit:

```
fb21b045 feat(autoplay): recipe library v1 (PROMPT 1609)
271f21bc docs(reports): PROMPT-1611 integration refresh report (strict-FF onto origin/main@ee9109c8)   <-- base
```

(The PROMPT-1613 report itself will be added as a follow-up commit on this
same branch before push, see "Report commit" below.)

## Why not reuse the PROMPT 1612 branch

`origin/integrate/autoplay-recipe-library-1612 @ 3b65f7de` was integrated on
top of `origin/main@ee9109c8`, before PROMPT 1611 (`ae9bdde3` and `271f21bc`)
landed. Pushing it as-is would either fail strict-FF or, if force-merged,
appear to revert PROMPT 1611 bot/autoplay ledger work. PROMPT 1613 therefore
rebuilds the integration on top of `271f21bc` instead.

Verified that within the owned scope (`docs/autoplay.md`,
`skills/ccgs-autoplay`, `tools/autoplay`) the PROMPT 1612 tree (`e6e2d7d2`)
and the PROMPT 1609 commit (`87763de2`) are identical
(`git diff 87763de2 e6e2d7d2 -- docs/autoplay.md skills/ccgs-autoplay tools/autoplay`
returns empty), so cherry-picking the PROMPT 1609 commit gives the exact same
recipe-library payload PROMPT 1612 was carrying.

Verified that no commit in the `576fbe8c..271f21bc` range touched the owned
scope (`git log 576fbe8c..271f21bc -- docs/autoplay.md skills/ccgs-autoplay
tools/autoplay` returns empty), so the cherry-pick applies cleanly with zero
conflicts and no preservation work is needed for PROMPT 1611 files (those
live outside the owned scope).

## Changed files (vs `origin/main@271f21bc`)

```
docs/autoplay.md                               |  99 ++++++++++-
skills/ccgs-autoplay/SKILL.md                  |  52 +++++-
tools/autoplay/README.md                       | 120 ++++++++++---
tools/autoplay/driver.py                       | 236 ++++++++++++++++---------
tools/autoplay/recipes/__init__.py             |  82 +++++++++
tools/autoplay/recipes/_builder.py             | 185 +++++++++++++++++++
tools/autoplay/recipes/_coords.py              |  71 ++++++++
tools/autoplay/recipes/class_select.py         |  40 +++++
tools/autoplay/recipes/draft_auction_probe.py  |  79 +++++++++
tools/autoplay/recipes/full_game.py            |  78 ++++++++
tools/autoplay/recipes/idle.py                 |  10 ++
tools/autoplay/recipes/lobby_create.py         |  44 +++++
tools/autoplay/recipes/placement_drag_probe.py |  53 ++++++
tools/autoplay/recipes/smoke.py                |  25 +++
14 files changed, 1050 insertions(+), 124 deletions(-)
```

All paths are inside the PROMPT 1613 owned scope
(`docs/autoplay.md`, `skills/ccgs-autoplay/**`, `tools/autoplay/**`).
No `production/**`, `client/**`, `server/**`, `shared/**`, or `Cargo*`
file was touched.

## Validation

| Check | Result |
|---|---|
| Path allowlist (owned scope only) | PASS — only `docs/autoplay.md`, `skills/ccgs-autoplay/SKILL.md`, `tools/autoplay/**` modified |
| `git diff --check 271f21bc..HEAD` | PASS — no whitespace errors |
| Cherry-pick conflicts | NONE |
| `git merge-base --is-ancestor 271f21bc HEAD` | PASS — strict-FF ready |
| Python `py_compile` on all changed `.py` | PASS (driver.py + 10 recipe modules) |
| `python tools/autoplay/driver.py --list-recipes` | PASS — 7 recipes enumerated (class-select, draft-auction-probe, full-game, idle, lobby-create, placement-drag-probe, smoke) |
| Cargo build | NOT RUN — task scope excludes broad Cargo work; no Rust files touched |
| Runtime autoplay GUI smoke | DEFERRED to next VERIFY lane per task scope |

`--list-recipes` output:

```
class-select        Class selection: click first card, click Confirm. Two checkpoints.
draft-auction-probe Shop click + auction bid/ready click. Four checkpoints (shop-loaded, shop-slot-clicked, auction-loaded, auction-ready).
full-game           Composite recipe (lobby -> class -> draft/auction -> placement). Requires PROMPT 1607 bot-vs-bot soak room; emits BLOCKED otherwise.
idle                No actions; ticks autoplay/status for soak / observability.
lobby-create        Lobby flow: click Create, wait, click Confirm. Two checkpoints (loaded, confirmed).
placement-drag-probe Drag from hand to board, click Submit. Three checkpoints (loaded, dragged, submitted).
smoke               Single input frame, clear, screenshot. Proves the RPC substrate.
```

## Autoplay semantics — preserved

No semantic gameplay verbs were added. Recipes drive the live Bevy client
exclusively through low-level keyboard / mouse / cursor RPCs plus the
`local.*` checkpoint/note/block pseudo-actions for run-log structure. This
matches the PROMPT 1595 substrate contract and the PROMPT 1609 acceptance
criteria. Behavior is unchanged versus the PROMPT 1609 source commit.

## Report commit

This report file is added in a follow-up commit on the same branch:

```
docs(reports): PROMPT-1613 autoplay recipe library main-ready refresh after 1611
```

Final branch tip after the report commit will be the actual mainland HEAD;
the SHA is recorded in the relay DONE summary.

## Next verify lane — exact commands

The next worker should run a runtime autoplay smoke against this branch:

```
# 1. Check out the branch
git fetch origin
git worktree add D:/Tmp/wt-1613-verify integrate/autoplay-recipe-library-1613

# 2. Launch a client (any standard CCGS dev launcher with autoplay RPC enabled).
#    PROMPT 1607 bot-vs-bot soak room is NOT on main yet, so do NOT set
#    CCGS_AUTOPLAY_BOT_ROOM_READY=1; the full-game recipe is expected to emit
#    BLOCKED (exit code 4) until that lands.

# 3. From the verify worktree, smoke each non-composite recipe individually:
cd D:/Tmp/wt-1613-verify
"D:/_APPS/Python312/python.exe" tools/autoplay/driver.py --list-recipes
"D:/_APPS/Python312/python.exe" tools/autoplay/driver.py --recipe smoke
"D:/_APPS/Python312/python.exe" tools/autoplay/driver.py --recipe idle --max-ticks 5
"D:/_APPS/Python312/python.exe" tools/autoplay/driver.py --recipe lobby-create
"D:/_APPS/Python312/python.exe" tools/autoplay/driver.py --recipe class-select
"D:/_APPS/Python312/python.exe" tools/autoplay/driver.py --recipe draft-auction-probe
"D:/_APPS/Python312/python.exe" tools/autoplay/driver.py --recipe placement-drag-probe
"D:/_APPS/Python312/python.exe" tools/autoplay/driver.py --recipe full-game   # expected exit 4 BLOCKED until PROMPT 1607 lands

# 4. Confirm exit codes (0 happy / 4 BLOCKED for full-game) and inspect
#    tools/autoplay/_runs/<timestamp>/checkpoints.jsonl per recipe.
```

## Mainland enqueue — exact next command

When the mainland worker is ready (orchestrator-supervised, single
shared-status writer policy honored):

```
git fetch origin
git checkout main
git merge --ff-only integrate/autoplay-recipe-library-1613
git push origin main
```

The branch is `integrate/autoplay-recipe-library-1613`; the local HEAD
prior to the report commit is `fb21b045`; the final tip including this
report will be relayed in the DONE summary.

1613: AUTOPLAY-RECIPE-LIBRARY-MAIN-READY-REFRESH-AFTER-1611: SHIPPED
