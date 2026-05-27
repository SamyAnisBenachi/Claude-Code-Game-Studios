# PROMPT 1646 — AUTOPLAY-DOCS-RECIPE-REGISTRY-REFRESH

**Date:** 2026-05-27
**Branch:** `work/autoplay-docs-recipe-registry-1646`
**Scope:** `docs/autoplay.md`, `docs/autoplay/evidence-operator-guide.md`

---

## Summary

Refreshed both autoplay documentation files to match the current recipe
registry (11 registered recipes as of PROMPTs 1634/1636/1639/1641/1644).
Removed stale blocker notes about resolution/round-end/game-over/add-bot
being missing — all four are now shipped.

---

## Registry Verification

`python tools/autoplay/driver.py --list-recipes` returned 11 recipes:

```
add-bot-lobby, class-select, draft-auction-probe, full-game,
game-over-observe, idle, lobby-create, placement-drag-probe,
resolution-observe, round-loop, smoke
```

All 11 are now documented in both files.

---

## Changes: docs/autoplay.md

### Recipe library table

- Section header updated from "PROMPT 1609" to "PROMPTs 1609 / 1634 / 1636 / 1639 / 1641 / 1644".
- Added `add-bot-lobby` row (PROMPT 1634): checkpoints `lobby-loaded`, `bot-added`, `lobby-confirmed`; requires `CCGS_DEBUG_UI=1`.
- Added `resolution-observe` row (PROMPT 1636): checkpoints `resolution-started`, `resolution-complete`; passive, no input.
- Added `game-over-observe` row (PROMPT 1636): checkpoints `game-over-screen`, `winner-confirmed`; passive, no input.
- Added `round-loop` row (PROMPT 1639): multi-round composite; requires `CCGS_AUTOPLAY_BOT_ROOM_READY=1`.
- Updated `full-game` row to reflect post-placement resolution soak (on by default, PROMPT 1641) and optional game-over chain.

### Blocked steps section

- Replaced "Blocked steps as of PROMPT 1609" (stale) with "Known detection limits (as of PROMPT 1644)".
- Removed stale entry: "Resolution / round-end — No autoplay observability for round transitions" — resolved by `resolution-observe` and `game-over-observe`.
- Retained accurate limits: bot-room gate, `add-bot-lobby` debug-UI gate, auction bid acknowledgement, placement accept/reject, phase-name opacity.

### Env vars table

Added 8 new variables:

| Var | Default |
|-----|---------|
| `CCGS_AUTOPLAY_FULL_GAME_RESOLUTION` | `1` (on) |
| `CCGS_AUTOPLAY_FULL_GAME_GAMEOVER` | unset (off) |
| `CCGS_AUTOPLAY_RESOLUTION_SOAK_TICKS` | `60` |
| `CCGS_AUTOPLAY_GAMEOVER_SOAK_TICKS` | `120` |
| `CCGS_AUTOPLAY_GAMEOVER_RESULT_SOAK_TICKS` | `30` |
| `CCGS_AUTOPLAY_ROUND_LOOP_COUNT` | `2` |
| `CCGS_AUTOPLAY_ROUND_SETTLE_TICKS` | `4` |

Also updated `CCGS_AUTOPLAY_BOT_ROOM_READY` description to mention both `full-game` and `round-loop`.

---

## Changes: docs/autoplay/evidence-operator-guide.md

### Section 4 — Phase checkpoints table

Added 10 new checkpoint rows:

- `bot-added` → `add-bot-lobby`
- `lobby-loaded` / `lobby-confirmed` → now cross-referenced to both `lobby-create` and `add-bot-lobby`
- `resolution-started` / `resolution-complete` → `resolution-observe`
- `game-over-screen` / `winner-confirmed` → `game-over-observe`
- `full-game-post-resolution` / `full-game-post-placement` / `full-game-complete` → `full-game` (the three terminal checkpoints depending on env flags)
- `round-{k}-start` / `round-loop-complete` → `round-loop`

### Section 7 — Recipe Quick-Reference

Added 4 new recipe rows: `add-bot-lobby`, `resolution-observe`, `game-over-observe`, `round-loop`.
Updated `full-game` row to show current terminal checkpoint and env gates.

### Footer

Updated last-updated stamp from PROMPT 1643 to PROMPT 1646.

---

## Validation

- `git diff --check -- docs/` → **clean** (exit 0; no trailing whitespace in docs files).
- Pre-existing trailing whitespace in `.claude/settings.json` is unrelated to this task.
- `python tools/autoplay/driver.py --list-recipes` → 11 recipes listed; all match the updated tables.

---

## Human-GUI gate preserved

No claim of live PASS or headless CI support added. The "BLOCKED-HUMAN-GUI" gate in the scope ladder and the "Headless CI smoke" deferred row are unchanged.

---

1646: AUTOPLAY-DOCS-RECIPE-REGISTRY-REFRESH: SHIPPED
