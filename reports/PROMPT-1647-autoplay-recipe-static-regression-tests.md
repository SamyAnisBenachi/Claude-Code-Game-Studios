# PROMPT 1647 — AUTOPLAY-RECIPE-STATIC-REGRESSION-TESTS

**Status:** SHIPPED  
**Commit:** ce836b2f  
**Branch:** work/autoplay-docs-recipe-registry-1646  
**Date:** 2026-05-27

## Summary

Added 64 pytest static regression tests for the `tools/autoplay` recipe library.
No GUI, no Bevy launch, no Cargo required.

## Test file

`tests/tools/autoplay/test_recipe_static.py`

## Coverage

| Area | Tests | Result |
|------|-------|--------|
| Registry completeness (11 expected recipes) | 4 | PASS |
| Method-allowlist contract (all 11 recipes × 3 checks) | 33 | PASS |
| Env-gate blocking (full-game, round-loop, add-bot-lobby) | 7 | PASS |
| Checkpoint label contracts (all phase recipes) | 20 | PASS |
| **Total** | **64** | **64 PASS / 0 FAIL** |

## Test categories

### TestRegistry
- `test_expected_recipe_count` — registry has exactly 11 entries
- `test_expected_recipe_names_present` — all 11 expected names present, none extra
- `test_names_returns_sorted_list` — `names()` returns sorted list
- `test_each_registry_entry_has_description_and_builder` — every entry has non-empty description and callable builder

### TestMethodAllowlist (parametrized over all 11 recipes)
- `test_recipe_only_emits_allowed_methods` — no method outside `{autoplay/*, local.*}` appears
- `test_recipe_actions_have_required_keys` — every action has `tick` (int) and `method`
- `test_recipe_ticks_are_positive` — all tick values ≥ 1

### TestEnvGateBlocking
- `full-game` blocks without `CCGS_AUTOPLAY_BOT_ROOM_READY`, does not block with it
- `round-loop` blocks without `CCGS_AUTOPLAY_BOT_ROOM_READY`, does not block with it
- `add-bot-lobby` blocks without `CCGS_DEBUG_UI=1`, does not block with it
- `test_full_game_block_row_is_first_meaningful_action` — no RPC calls emitted before `local.block`

### TestCheckpointContracts
- `resolution-observe`: `resolution-started` → `resolution-complete` (order verified)
- `game-over-observe`: `game-over-wait-start` → `game-over-screen` → `winner-confirmed` (order verified)
- `lobby-create`: `lobby-loaded`, `lobby-confirmed`
- `class-select`: `class-select-loaded`, `class-confirmed`
- `draft-auction-probe`: `shop-loaded`, `auction-loaded`
- `placement-drag-probe`: `placement-loaded`, `placement-submitted`
- `add-bot-lobby` (unblocked): `lobby-loaded`, `bot-added`, `lobby-confirmed`
- `full-game` default (resolution on): tail = `full-game-post-resolution`
- `full-game` resolution off: tail = `full-game-post-placement`
- `full-game` gameover on: tail = `full-game-complete`
- `full-game` includes all sub-recipe checkpoints: `lobby-loaded`, `class-select-loaded`, `shop-loaded`, `placement-loaded`, `resolution-started`
- `round-loop`: `round-loop-complete`, `game-over-screen`, `winner-confirmed`
- `round-loop` default count=2: `round-2-start` present, `round-3-start` absent
- `round-loop` count=3: both `round-2-start` and `round-3-start` present
- `smoke`: non-empty action list
- `idle`: empty action list

## Validation runs

```
pytest tests/tools/autoplay/test_recipe_static.py -v
64 passed in 0.06s

python tools/autoplay/driver.py --list-recipes
11 recipes listed (add-bot-lobby, class-select, draft-auction-probe,
full-game, game-over-observe, idle, lobby-create, placement-drag-probe,
resolution-observe, round-loop, smoke)

git diff --check (staged only)
CLEAN
```

## No recipe behavior changes

No recipe file was modified. All defects found were none — the existing contracts
already held. The tests lock them in as a regression gate.

---

1647: AUTOPLAY-RECIPE-STATIC-REGRESSION-TESTS: SHIPPED
