# PROMPT-1641 — Autoplay Full Game Recipe Extend Post Placement

**Date:** 2026-05-27
**Branch:** `work/autoplay-full-game-post-placement-1641`
**Base:** `origin/integrate/autoplay-resolution-gameover-observe-1636-refresh@e67a3488`

---

## Summary

Extended the `full-game` recipe to optionally chain post-placement observation
checkpoints using the now-landed `resolution-observe` and `game-over-observe`
sub-recipes (PROMPT 1636). The extension produces better evidence of the
post-placement game state without requiring a complete game run or introducing
false failures.

---

## Changed Files

| File | Change |
|---|---|
| `tools/autoplay/recipes/full_game.py` | Extended — post-placement resolution/game-over phases added |
| `reports/PROMPT-1641-autoplay-full-game-recipe-extend-post-placement.md` | **New** — this report |

---

## Design Decisions

### Resolution observation: ON by default

After `placement-drag-probe` completes, the recipe now chains
`resolution-observe` by default (soak + 2 screenshots). This directly
improves evidence: operators get `resolution-started` and
`resolution-complete` screenshots without any extra configuration, and the
soak is passive — it cannot cause false failures (no assertion on phase name).

Disable with `CCGS_AUTOPLAY_FULL_GAME_RESOLUTION=0` if the operator
intentionally wants to stop at placement.

### GameOver observation: OFF by default

`game-over-observe` is opt-in (`CCGS_AUTOPLAY_FULL_GAME_GAMEOVER=1`).
GameOver requires natural HP drain which is not yet bounded by
`--bot-vs-bot-max-rounds` (Gap F, deferred). Enabling it without that
bound risks very long or indefinite soak windows. When enabled, the chain
extends to three additional checkpoints: `game-over-wait-start`,
`game-over-screen`, `winner-confirmed`.

### Checkpoint naming follows phase selected

The final composite checkpoint name reflects which phases ran:
- `full-game-post-placement` — resolution and game-over both skipped
- `full-game-post-resolution` — resolution observed, no game-over
- `full-game-complete` — full chain including game-over

### Soak length env vars are inherited from sub-recipes

`CCGS_AUTOPLAY_RESOLUTION_SOAK_TICKS`, `CCGS_AUTOPLAY_GAMEOVER_SOAK_TICKS`,
and `CCGS_AUTOPLAY_GAMEOVER_RESULT_SOAK_TICKS` work exactly as documented
in the standalone sub-recipes — no new env surface needed.

---

## Env Gates Reference

| Variable | Default | Effect |
|---|---|---|
| `CCGS_AUTOPLAY_BOT_ROOM_READY` | unset | Must be `"1"` — blocks recipe if missing |
| `CCGS_AUTOPLAY_FULL_GAME_RESOLUTION` | `"1"` (on) | Set `"0"` to skip resolution observe phase |
| `CCGS_AUTOPLAY_FULL_GAME_GAMEOVER` | `"0"` (off) | Set `"1"` to chain game-over observe phase |
| `CCGS_AUTOPLAY_RESOLUTION_SOAK_TICKS` | `60` | Resolution soak length in driver ticks |
| `CCGS_AUTOPLAY_GAMEOVER_SOAK_TICKS` | `120` | Ticks before game-over-screen checkpoint |
| `CCGS_AUTOPLAY_GAMEOVER_RESULT_SOAK_TICKS` | `30` | Ticks before winner-confirmed checkpoint |

---

## Validation

```
python -m py_compile tools/autoplay/recipes/full_game.py  → SYNTAX OK

python tools/autoplay/driver.py --list-recipes:
  full-game   Composite recipe (lobby -> class -> draft/auction -> placement -> resolution soak).
              Requires PROMPT 1607 bot-vs-bot soak room; emits BLOCKED otherwise.
              Resolution observation on by default; GameOver opt-in via
              CCGS_AUTOPLAY_FULL_GAME_GAMEOVER=1.
  (10 recipes total — unchanged count vs pre-1641)

git diff --check  → clean (LF→CRLF warning only, not a content error)
```

---

## Known Limitations

- **BLOCKED-HUMAN-GUI**: like all existing recipes, full-game requires an
  interactive desktop session with the Bevy client running.
- **No phase-name RPC**: the resolution and game-over soaks are passive.
  Human reviewer must inspect bracketing screenshots.
- **GameOver opt-in risk**: without `--bot-vs-bot-max-rounds`, enabling
  `CCGS_AUTOPLAY_FULL_GAME_GAMEOVER=1` may produce an indefinitely long run.
- **Dependencies on 1636**: `resolution_observe.py` and `game_over_observe.py`
  must be present in the recipes package (landed in PROMPT 1636 via the
  `integrate/autoplay-resolution-gameover-observe-1636-refresh` branch).

---

1641: AUTOPLAY-FULL-GAME-POST-PLACEMENT-EXTENSION: SHIPPED
