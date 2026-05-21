#!/usr/bin/env python3
"""Continuous autoplay driver for the CCGS Bevy client.

PROMPT 1595 -- see `docs/autoplay.md` for the hard invariants:
    - Drives the client only through low-level input (keys, mouse buttons,
      cursor position, scroll). No semantic gameplay verbs. No ECS writes.
    - Talks to the `autoplay-remote` JSON-RPC server bound on
      127.0.0.1:<port> when the client is launched with the
      `autoplay-remote` Cargo feature and the `CCGS_AUTOPLAY=1` env var.

This script is intentionally dependency-free (Python 3.10+ standard library
only). It is the persistent driver process the bootstrap skill demands:
a single long-running process that ticks at a fixed rate, not one shell
RPC helper per frame.

Default recipe (`smoke`) confirms the harness substrate by:
    1. polling `autoplay/capabilities` once and writing it to
       `<artifact-dir>/capabilities.json`,
    2. ticking `autoplay/status` at `--hz`,
    3. on tick 1 sending one input frame
       (`KeyA` + Left mouse + cursor to centre of window),
    4. on tick 2 clearing input via `autoplay/clear_input`,
    5. on tick 3 requesting a screenshot,
    6. exiting after `--ticks` ticks (default 10) or `--timeout` seconds.

Add recipes by extending `RECIPES` below. Recipes return a list of action
dicts; each dict is consumed once on the matching tick.

Artifacts written to `--artifact-dir`:
    capabilities.json    -- one-shot capability probe
    driver-timeline.jsonl -- one row per tick (status snapshot)
    driver.log           -- human-readable progress log

Exit codes:
    0  -- recipe completed cleanly
    1  -- RPC error during the run
    2  -- unable to reach the RPC server at startup
"""

from __future__ import annotations

import argparse
import json
import os
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path
from typing import Any, Callable


def rpc(url: str, method: str, params: dict[str, Any] | None = None, timeout: float = 5.0) -> Any:
    payload = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params or {},
    }
    request = urllib.request.Request(
        url,
        data=json.dumps(payload).encode("utf-8"),
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    with urllib.request.urlopen(request, timeout=timeout) as response:
        body = json.loads(response.read().decode("utf-8"))
    if "error" in body:
        raise RuntimeError(body["error"].get("message", str(body["error"])))
    return body.get("result")


def append_jsonl(path: Path, row: dict[str, Any]) -> None:
    with path.open("a", encoding="utf-8") as fh:
        fh.write(json.dumps(row, separators=(",", ":")) + "\n")


def smoke_recipe(window_size: tuple[float, float]) -> list[dict[str, Any]]:
    """Three-action smoke recipe: input frame, clear, screenshot.

    Each list element is consumed on the matching tick number (1-indexed).
    """
    cx, cy = window_size[0] / 2.0, window_size[1] / 2.0
    return [
        {
            "tick": 1,
            "method": "autoplay/input",
            "params": {
                "keys_down": ["KeyA"],
                "mouse_down": ["Left"],
                "cursor": {"screen": [cx, cy]},
            },
        },
        {"tick": 2, "method": "autoplay/clear_input", "params": {}},
        {"tick": 3, "method": "autoplay/screenshot", "params": {"reason": "smoke-driver"}},
    ]


def idle_recipe(_size: tuple[float, float]) -> list[dict[str, Any]]:
    """Status-only recipe: useful for soak / observability runs."""
    return []


RECIPES: dict[str, Callable[[tuple[float, float]], list[dict[str, Any]]]] = {
    "smoke": smoke_recipe,
    "idle": idle_recipe,
}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--port", type=int, default=int(os.environ.get("CCGS_AUTOPLAY_PORT", "15873")))
    parser.add_argument(
        "--artifact-dir",
        default=os.environ.get("CCGS_AUTOPLAY_DRIVER_ARTIFACT_DIR", "production/qa/evidence/autoplay-runs/driver"),
    )
    parser.add_argument("--recipe", default="smoke", choices=sorted(RECIPES.keys()))
    parser.add_argument("--hz", type=float, default=10.0)
    parser.add_argument("--ticks", type=int, default=10, help="exit after this many ticks (0 = unbounded)")
    parser.add_argument("--timeout", type=float, default=30.0)
    parser.add_argument("--startup-grace", type=float, default=20.0)
    args = parser.parse_args()

    url = f"http://127.0.0.1:{args.port}/"
    artifact_dir = Path(args.artifact_dir)
    artifact_dir.mkdir(parents=True, exist_ok=True)
    timeline_path = artifact_dir / "driver-timeline.jsonl"
    log_path = artifact_dir / "driver.log"
    caps_path = artifact_dir / "capabilities.json"

    def log(line: str) -> None:
        stamp = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
        with log_path.open("a", encoding="utf-8") as fh:
            fh.write(f"{stamp} {line}\n")
        print(line, flush=True)

    # --- startup: probe capabilities (with retry / grace) ---
    started = time.monotonic()
    while True:
        try:
            capabilities = rpc(url, "autoplay/capabilities")
            caps_path.write_text(json.dumps(capabilities, indent=2), encoding="utf-8")
            log(f"capabilities OK; version={capabilities.get('version')}")
            break
        except (urllib.error.URLError, ConnectionError, TimeoutError, RuntimeError) as err:
            if time.monotonic() - started > args.startup_grace:
                log(f"startup failed after {args.startup_grace}s: {err}")
                return 2
            time.sleep(0.5)

    tick_secs = 1.0 / max(args.hz, 0.1)
    recipe_actions: list[dict[str, Any]] | None = None
    tick = 0
    rc = 0
    deadline = time.monotonic() + args.timeout

    try:
        while True:
            tick += 1
            now = time.monotonic()
            if now > deadline:
                log(f"timeout after {args.timeout}s ({tick - 1} ticks completed)")
                rc = 0  # timeout is a normal exit condition for soak recipes
                break

            try:
                status = rpc(url, "autoplay/status")
            except (urllib.error.URLError, RuntimeError, ConnectionError, TimeoutError) as err:
                log(f"status RPC failed on tick {tick}: {err}")
                rc = 1
                break

            if recipe_actions is None:
                size = status.get("window_logical_size") or [1280.0, 720.0]
                if not isinstance(size, list) or len(size) != 2:
                    size = [1280.0, 720.0]
                recipe_actions = RECIPES[args.recipe](tuple(size))
                log(f"recipe={args.recipe} actions={len(recipe_actions)}")

            action_result: Any = None
            for action in recipe_actions:
                if action["tick"] == tick:
                    try:
                        action_result = rpc(url, action["method"], action["params"])
                        log(
                            f"tick={tick} action method={action['method']} "
                            f"params_keys={list(action['params'].keys())}"
                        )
                    except (urllib.error.URLError, RuntimeError, ConnectionError, TimeoutError) as err:
                        log(f"action RPC failed on tick {tick}: {err}")
                        rc = 1

            append_jsonl(
                timeline_path,
                {
                    "tick": tick,
                    "recipe": args.recipe,
                    "elapsed_secs": round(now - started, 3),
                    "status": status,
                    "action_result": action_result,
                },
            )

            if args.ticks > 0 and tick >= args.ticks:
                log(f"reached --ticks={args.ticks}; exiting")
                break

            sleep_for = tick_secs - (time.monotonic() - now)
            if sleep_for > 0:
                time.sleep(sleep_for)
    except KeyboardInterrupt:
        log("interrupted by user")
        rc = 1

    log(f"exit rc={rc}")
    return rc


if __name__ == "__main__":
    raise SystemExit(main())
