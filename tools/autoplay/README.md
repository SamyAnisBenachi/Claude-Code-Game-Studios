# tools/autoplay

PROMPT 1595 -- dev-only automation harness for the CCGS Bevy client.
PROMPT 1609 -- recipe library on top of the substrate. See
[`docs/autoplay.md`](../../docs/autoplay.md) for architecture, scope
ladder, and the hard invariants (low-level input only; no gameplay
mutation). For the project-local runbook future agents should follow,
see [`skills/ccgs-autoplay/SKILL.md`](../../skills/ccgs-autoplay/SKILL.md).

## Files

| Path | Purpose |
| --- | --- |
| `driver.py` | Long-running Python driver. Ticks at a fixed Hz; runs a registered recipe. The autoplay loop. |
| `rpc.py` | One-shot RPC helper for ad-hoc pokes (status, screenshot, single input). Do not loop this. |
| `Run-AutoplaySmoke.ps1` | Windows-native launcher. Builds the client, starts it with `CCGS_AUTOPLAY=1`, waits for the RPC port, runs the smoke recipe, captures `process.log` + `launcher-status.json`. |
| `recipes/` | Recipe library (Python). One module per recipe; `__init__.py` registers them. |
| `recipes/_builder.py` | Reusable primitives (`click`, `drag`, `press`, `checkpoint`, `block`, …). |
| `recipes/_coords.py` | Fractional-coordinate defaults + env-var overrides for UI buttons. |

## Quick start

```powershell
# From the repo root, smallest possible smoke (auto-creates a timestamped artifact dir):
pwsh -File tools/autoplay/Run-AutoplaySmoke.ps1
```

```sh
# List available recipes:
python tools/autoplay/driver.py --list-recipes
```

```sh
# Run a custom driver directly against an already-running client:
python tools/autoplay/driver.py --port 15873 --recipe smoke --hz 5
python tools/autoplay/driver.py --recipe lobby-create
python tools/autoplay/driver.py --recipe full-game --timeout 300
```

```sh
# After the launcher has the client running, poke the surface manually:
python tools/autoplay/rpc.py capabilities
python tools/autoplay/rpc.py status
python tools/autoplay/rpc.py screenshot --reason debug-poke
python tools/autoplay/rpc.py input --keys-down KeyA --cursor 400 300
python tools/autoplay/rpc.py clear
```

## Recipes (PROMPT 1609 + 1634 + 1636 + 1639)

| Name | Purpose | Checkpoints |
| --- | --- | --- |
| `smoke` | Substrate probe: one input frame, clear, screenshot. | — |
| `idle` | Status-only ticks for soak / observability. | — |
| `lobby-create` | Click Create, wait, click Confirm. | `lobby-loaded`, `lobby-confirmed` |
| `add-bot-lobby` | Lobby flow: click Create, Add Bot, Confirm. Requires `CCGS_DEBUG_UI=1`. | `lobby-loaded`, `bot-added`, `lobby-confirmed` |
| `class-select` | Click first class card, click Confirm. | `class-select-loaded`, `class-confirmed` |
| `draft-auction-probe` | Click shop slot, confirm, bid on auction, ready. | `shop-loaded`, `shop-slot-clicked`, `auction-loaded`, `auction-ready` |
| `placement-drag-probe` | Drag from hand to board, click Submit. | `placement-loaded`, `placement-dragged`, `placement-submitted` |
| `resolution-observe` | Passive resolution-phase observation: soak + screenshots. No input. Override soak via `CCGS_AUTOPLAY_RESOLUTION_SOAK_TICKS` (default 60). | `resolution-started`, `resolution-complete` |
| `game-over-observe` | Passive game-over screen observation: soak + screenshots. No input. Override via `CCGS_AUTOPLAY_GAMEOVER_SOAK_TICKS` (default 120). | `game-over-screen`, `winner-confirmed` |
| `round-loop` | Multi-round composite: full-game → resolution-observe × N → game-over-observe. Requires `CCGS_AUTOPLAY_BOT_ROOM_READY=1`. Configure via `CCGS_AUTOPLAY_ROUND_LOOP_COUNT` (default 2). | All composite checkpoints |
| `full-game` | Composite (lobby → class → draft/auction → placement → resolution soak). Requires `CCGS_AUTOPLAY_BOT_ROOM_READY=1`. GameOver opt-in via `CCGS_AUTOPLAY_FULL_GAME_GAMEOVER=1`. | All of the above + `full-game-post-placement`, `full-game-post-resolution`, `full-game-complete` |

### Driver exit codes

| Code | Meaning |
| --- | --- |
| 0 | recipe completed cleanly (or hit `--timeout`) |
| 1 | RPC error during the run |
| 2 | RPC server never bound during `--startup-grace` |
| 4 | recipe emitted `local.block` — upstream prerequisite missing |

### Coordinate overrides

Phase recipes target UI buttons at default fractional positions
(centre-column / lower-third). When the live layout drifts, override
with env vars:

```sh
# Format: CCGS_AUTOPLAY_<KEY>=<fx>,<fy>   (both fractions in [0,1])
$env:CCGS_AUTOPLAY_LOBBY_CREATE_BTN  = "0.50,0.60"
$env:CCGS_AUTOPLAY_LOBBY_CONFIRM_BTN = "0.50,0.88"
$env:CCGS_AUTOPLAY_CLASS_FIRST_CARD  = "0.22,0.42"
$env:CCGS_AUTOPLAY_CLASS_CONFIRM_BTN = "0.50,0.88"
$env:CCGS_AUTOPLAY_SHOP_FIRST_SLOT   = "0.30,0.45"
$env:CCGS_AUTOPLAY_SHOP_CONFIRM_BTN  = "0.50,0.85"
$env:CCGS_AUTOPLAY_AUCTION_BID_BTN   = "0.50,0.55"
$env:CCGS_AUTOPLAY_AUCTION_READY_BTN = "0.50,0.85"
$env:CCGS_AUTOPLAY_HAND_FIRST_CARD   = "0.35,0.92"
$env:CCGS_AUTOPLAY_BOARD_FIRST_CELL  = "0.50,0.55"
$env:CCGS_AUTOPLAY_SUBMIT_BTN        = "0.85,0.92"
```

All keys default to sensible centre/lower-third positions; unset env
vars use the defaults. Malformed values are logged as a `local.note`
row and fall back to the default.

## Adding a recipe

1. Create `tools/autoplay/recipes/<slug>.py` exposing `NAME`,
   `DESCRIPTION`, and `build(ctx) -> list[dict]`.
2. Use `RecipeBuilder` from `recipes._builder` for the primitives —
   never hand-roll action dicts that bypass the allowlist.
3. Register the module in `tools/autoplay/recipes/__init__.py`.
4. Confirm it appears in `python tools/autoplay/driver.py --list-recipes`.

Recipes MAY emit the low-level autoplay RPC methods
(`autoplay/input`, `autoplay/clear_input`, `autoplay/screenshot`,
`autoplay/status`, `autoplay/capabilities`) and the driver-local
pseudo-methods (`local.checkpoint`, `local.note`, `local.block`).
Anything else is rejected before the first tick — the driver refuses
to start a recipe that names a method outside the allowlist.

## Rules

- **Low-level input only.** Never add a recipe that synthesizes
  gameplay state. The Rust harness's RPC surface enforces this —
  there is no `autoplay/select_card` method and there must never be
  one.
- **One driver process per session.** `driver.py` is the autoplay
  engine; do not loop `rpc.py` to fake one. Multiple drivers against
  the same client are allowed for soak tests but each must use its
  own `--artifact-dir`.
- **Always pass through the launcher for evidence.**
  `Run-AutoplaySmoke.ps1` emits `process.log` +
  `launcher-status.json` next to the driver's `driver.log` +
  `driver-timeline.jsonl` + `checkpoints.jsonl` so a reviewer can
  reconstruct the full run.
- **Block, do not silently pass.** If a recipe depends on an
  upstream prompt (e.g. PROMPT 1607 bot-vs-bot soak room), emit
  `RecipeBuilder.block(...)`; the driver exits with code 4 and the
  blocker row lands in `checkpoints.jsonl`.
