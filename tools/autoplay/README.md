# tools/autoplay

PROMPT 1595 -- dev-only automation harness for the CCGS Bevy client. See
[`docs/autoplay.md`](../../docs/autoplay.md) for architecture, scope ladder,
and the hard invariants (low-level input only; no gameplay mutation). For
the project-local runbook future agents should follow, see
[`skills/ccgs-autoplay/SKILL.md`](../../skills/ccgs-autoplay/SKILL.md).

## Files

| Path | Purpose |
| --- | --- |
| `driver.py` | Long-running Python driver. Ticks at a fixed Hz; runs recipes (`smoke`, `idle`). The autoplay loop. |
| `rpc.py` | One-shot RPC helper for ad-hoc pokes (status, screenshot, single input). Do not loop this. |
| `Run-AutoplaySmoke.ps1` | Windows-native launcher. Builds the client, starts it with `CCGS_AUTOPLAY=1`, waits for the RPC port, runs the smoke recipe, captures `process.log` + `launcher-status.json`. |

## Quick start

```powershell
# From the repo root, smallest possible smoke (auto-creates a timestamped artifact dir):
pwsh -File tools/autoplay/Run-AutoplaySmoke.ps1
```

```powershell
# Custom port + artifact dir:
pwsh -File tools/autoplay/Run-AutoplaySmoke.ps1 -Port 15874 -ArtifactDir D:/Tmp/autoplay-test
```

```sh
# After the launcher has the client running, poke the surface manually:
python tools/autoplay/rpc.py capabilities
python tools/autoplay/rpc.py status
python tools/autoplay/rpc.py screenshot --reason debug-poke
python tools/autoplay/rpc.py input --keys-down KeyA --cursor 400 300
python tools/autoplay/rpc.py clear
```

```sh
# Run a custom driver directly against an already-running client:
python tools/autoplay/driver.py --port 15873 --recipe smoke --ticks 20 --hz 5
```

## Adding a recipe

Edit `tools/autoplay/driver.py`. Recipes are pure-Python functions registered
in the `RECIPES` dict. Each function returns a list of action dicts; each
action runs on its matching `tick` (1-indexed). Recipes MUST only use the
low-level input methods (`autoplay/input`, `autoplay/clear_input`,
`autoplay/screenshot`, `autoplay/status`, `autoplay/capabilities`).

## Rules

- **Low-level input only.** Never add a recipe that synthesizes gameplay
  state. The Rust harness's RPC surface enforces this — there is no
  `autoplay/select_card` method and there must never be one.
- **One driver process per session.** `driver.py` is the autoplay engine;
  do not loop `rpc.py` to fake one. Multiple drivers against the same
  client are allowed for soak tests but each must use its own
  `--artifact-dir`.
- **Always pass through the launcher for evidence.** `Run-AutoplaySmoke.ps1`
  emits `process.log` + `launcher-status.json` next to the driver's
  `driver.log` + `driver-timeline.jsonl` so a reviewer can reconstruct the
  full run.
