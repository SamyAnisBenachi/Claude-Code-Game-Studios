# PROMPT-1747: AUTOPLAY-VS-BOT-QA-STORY004-RECIPE-COMPOSITION

**Date**: 2026-05-28
**Branch**: `feature/1747-autoplay-vs-bot-recipe-composition`

---

## What was found in the existing autoplay layer

### RECIPE_GAP identified by PROMPT-1740 smoke

- `full-game` recipe uses `lobby_create` as its entry point — this waits for
  a second human client, which never arrives in a bot-vs-human scenario.
- `add-bot-lobby` recipe covers only the lobby phase (Create + Add Bot + Confirm);
  it has no downstream phases.
- No composite recipe combined `add-bot-lobby` with the class-select, draft/auction,
  placement, and resolution phases into a single selectable Story 004 run.

### Existing recipe registry (pre-1747)

11 recipes: `smoke`, `idle`, `add-bot-lobby`, `lobby-create`, `class-select`,
`draft-auction-probe`, `placement-drag-probe`, `resolution-observe`,
`game-over-observe`, `round-loop`, `full-game`.

### `full-game` env gate

Requires only `CCGS_AUTOPLAY_BOT_ROOM_READY=1`. The Add Bot path additionally
requires `CCGS_DEBUG_UI=1` because the Add Bot button is a debug-only affordance.

---

## What was added / modified

### New file: `tools/autoplay/recipes/vs_bot.py`

New `vs-bot` recipe that:

- Requires both `CCGS_DEBUG_UI=1` (Add Bot button) and
  `CCGS_AUTOPLAY_BOT_ROOM_READY=1` (bot soak room); emits `local.block` early
  for each missing gate.
- Phase sequence:
  `add-bot-lobby` → `class-select` → `draft-auction-probe`
  → `placement-drag-probe` → `resolution-observe`
  → (opt-in) `game-over-observe`
- Tail checkpoints: `vs-bot-post-resolution` (default), `vs-bot-post-placement`
  (when `CCGS_AUTOPLAY_VS_BOT_RESOLUTION=0`), `vs-bot-complete` (when
  `CCGS_AUTOPLAY_VS_BOT_GAMEOVER=1`).
- All coordinate overrides used by child recipes are honoured unchanged.
- 4-tick settling gap between phases (same convention as `full-game`).

### Modified: `tools/autoplay/recipes/__init__.py`

- Added `vs_bot` to the import block.
- Added `vs_bot.NAME: (vs_bot.DESCRIPTION, vs_bot.build)` to `REGISTRY`.
- Registry count goes from 11 → 12.

### Modified: `tests/tools/autoplay/test_recipe_static.py`

- Updated docstring count comment: 11 → 12.
- Added `"vs-bot"` to `EXPECTED_RECIPES`.
- Added 4 env-gate blocking tests for `vs-bot` in `TestEnvGateBlocking`.
- Added 6 checkpoint contract tests for `vs-bot` in `TestCheckpointContracts`.

### Not modified: `tools/dev-launcher/Start-AutoplayVsBot.ps1`

The launcher already accepts `-Recipe NAME` and passes it through to
`Run-AutoplaySmoke.ps1` without any recipe-name hardcoding in validation
logic. Running `Start-AutoplayVsBot.ps1 -Recipe vs-bot` works as-is.
No edits needed.

### Not modified: `full-game` recipe

`full-game` behaviour is preserved exactly. It continues to use `lobby-create`
and requires only `CCGS_AUTOPLAY_BOT_ROOM_READY=1`.

---

## Static test results

```
pytest tests/tools/autoplay/test_recipe_static.py -v
77 passed in 0.08s
```

All 77 tests pass including 10 new `vs-bot`-specific tests covering:
- Registry presence and count (12 recipes)
- Method allowlist compliance
- Env-gate blocking (DEBUG_UI missing, BOT_ROOM missing, both set)
- Block-row-before-RPC contract
- Checkpoint labels: `lobby-loaded`, `bot-added`, `lobby-confirmed`,
  `class-select-loaded`, `shop-loaded`, `placement-loaded`, `resolution-started`
- Tail checkpoint variants (post-resolution / post-placement / complete)

`git diff --check` — no trailing whitespace.

---

## Blockers

None. Live GUI verification is deferred to a VERIFY prompt (out of scope for
this recipe-composition story).

---

1747: AUTOPLAY-VS-BOT-QA-STORY004-RECIPE-COMPOSITION: SHIPPED
