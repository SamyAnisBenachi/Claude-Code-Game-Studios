#!/usr/bin/env python3
"""Continuous autoplay driver for the CCGS Bevy client.

PROMPT 1595 introduced the substrate (status/input/clear/screenshot).
PROMPT 1609 layered the recipe library on top — see
``tools/autoplay/recipes/__init__.py`` for the registry. This module
remains the long-running driver process the bootstrap skill requires:
one process, ticking at a fixed rate, consuming a recipe's action
stream.

Hard invariants (enforced here, see ``ALLOWED_METHODS``):
  * Only the low-level autoplay RPC methods may reach the wire.
  * ``local.checkpoint`` / ``local.note`` / ``local.block`` are
    driver-side pseudo-methods that never leave this process — they
    write to ``checkpoints.jsonl`` (and ``local.block`` flips the
    exit code to 4 so a recipe that detects an upstream blocker
    fails loudly rather than silently passing).

Artifacts written to ``--artifact-dir``:
    capabilities.json     -- one-shot capability probe
    driver-timeline.jsonl -- one row per tick (status snapshot)
    driver.log            -- human-readable progress log
    checkpoints.jsonl     -- one row per local.* pseudo-action
                             (created lazily; absent if recipe never
                             uses checkpoint/note/block)

Exit codes:
    0  -- recipe completed cleanly
    1  -- RPC error during the run
    2  -- unable to reach the RPC server at startup
    4  -- recipe emitted ``local.block`` (upstream prerequisite missing)
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
from typing import Any

# Add the parent of this script to sys.path so ``recipes`` imports work
# whether the driver is invoked as ``python tools/autoplay/driver.py``
# or via ``python -m tools.autoplay.driver``.
_HERE = Path(__file__).resolve().parent
if str(_HERE) not in sys.path:
    sys.path.insert(0, str(_HERE))

from recipes import RecipeContext, REGISTRY, get as get_recipe, names as recipe_names  # noqa: E402
from win_foreground import ensure_foreground  # noqa: E402


ALLOWED_RPC_METHODS = {
    "autoplay/capabilities",
    "autoplay/status",
    "autoplay/input",
    "autoplay/clear_input",
    "autoplay/screenshot",
}

LOCAL_METHODS = {
    "local.checkpoint",
    "local.note",
    "local.block",
}

EXIT_OK = 0
EXIT_RPC_ERROR = 1
EXIT_NO_SERVER = 2
EXIT_BLOCKED = 4


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


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--port", type=int, default=int(os.environ.get("CCGS_AUTOPLAY_PORT", "15873")))
    parser.add_argument(
        "--artifact-dir",
        default=os.environ.get("CCGS_AUTOPLAY_DRIVER_ARTIFACT_DIR", "production/qa/evidence/autoplay-runs/driver"),
    )
    parser.add_argument("--recipe", default="smoke")
    parser.add_argument("--hz", type=float, default=10.0)
    parser.add_argument("--ticks", type=int, default=0, help="exit after this many ticks (0 = follow recipe length or --timeout)")
    parser.add_argument("--timeout", type=float, default=120.0)
    parser.add_argument("--startup-grace", type=float, default=20.0)
    parser.add_argument("--list-recipes", action="store_true", help="print the recipe registry and exit")
    args = parser.parse_args()

    if args.list_recipes:
        for name in recipe_names():
            desc, _ = REGISTRY[name]
            print(f"{name}\t{desc}")
        return EXIT_OK

    if args.recipe not in REGISTRY:
        parser.error(f"unknown recipe '{args.recipe}'. Known: {', '.join(recipe_names())}")
        return 2  # not reached

    url = f"http://127.0.0.1:{args.port}/"
    artifact_dir = Path(args.artifact_dir)
    artifact_dir.mkdir(parents=True, exist_ok=True)
    timeline_path = artifact_dir / "driver-timeline.jsonl"
    log_path = artifact_dir / "driver.log"
    caps_path = artifact_dir / "capabilities.json"
    checkpoints_path = artifact_dir / "checkpoints.jsonl"

    def log(line: str) -> None:
        stamp = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
        with log_path.open("a", encoding="utf-8") as fh:
            fh.write(f"{stamp} {line}\n")
        print(line, flush=True)

    def emit_checkpoint(row: dict[str, Any]) -> None:
        append_jsonl(checkpoints_path, row)

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
                return EXIT_NO_SERVER
            time.sleep(0.5)

    tick_secs = 1.0 / max(args.hz, 0.1)
    recipe_actions: list[dict[str, Any]] | None = None
    recipe_actions_by_tick: dict[int, list[dict[str, Any]]] = {}
    last_recipe_tick: int = 0
    tick = 0
    rc = EXIT_OK
    deadline = time.monotonic() + args.timeout
    blocked_reason: str | None = None
    # Frame-advance barrier (GAP-SCR-01 / PROMPT 1766): track the Bevy
    # ``status.frame`` value at the time of the last screenshot dispatch so we
    # can verify the renderer produced at least one new frame before the next
    # capture.  Initialised to -1 so the first screenshot always passes.
    last_screenshot_frame: int = -1

    _desc, build_fn = get_recipe(args.recipe)

    try:
        while True:
            tick += 1
            now = time.monotonic()
            if now > deadline:
                log(f"timeout after {args.timeout}s ({tick - 1} ticks completed)")
                break

            try:
                status = rpc(url, "autoplay/status")
            except (urllib.error.URLError, RuntimeError, ConnectionError, TimeoutError) as err:
                log(f"status RPC failed on tick {tick}: {err}")
                rc = EXIT_RPC_ERROR
                break

            if recipe_actions is None:
                size = status.get("window_logical_size") or [1280.0, 720.0]
                if not isinstance(size, list) or len(size) != 2:
                    size = [1280.0, 720.0]
                ctx = RecipeContext(window_size=(float(size[0]), float(size[1])), env=dict(os.environ))
                recipe_actions = build_fn(ctx)
                # Validate methods up-front so a malformed recipe fails before any RPC.
                for action in recipe_actions:
                    method = action.get("method", "")
                    if method not in ALLOWED_RPC_METHODS and method not in LOCAL_METHODS:
                        log(f"recipe {args.recipe} emits forbidden method {method!r}; refusing to run")
                        return EXIT_RPC_ERROR
                # Group actions by tick so a recipe can fire several on the same tick.
                for action in recipe_actions:
                    recipe_actions_by_tick.setdefault(int(action["tick"]), []).append(action)
                last_recipe_tick = max(recipe_actions_by_tick) if recipe_actions_by_tick else 0
                log(
                    f"recipe={args.recipe} actions={len(recipe_actions)} "
                    f"last_recipe_tick={last_recipe_tick}"
                )

            action_results: list[Any] = []
            for action in recipe_actions_by_tick.get(tick, []):
                method = action["method"]
                params = action.get("params", {}) or {}
                if method in LOCAL_METHODS:
                    if method == "local.checkpoint":
                        label = str(params.get("label", "unlabelled"))
                        emit_checkpoint({
                            "tick": tick,
                            "kind": "checkpoint",
                            "label": label,
                            "elapsed_secs": round(now - started, 3),
                            "screenshot": bool(params.get("screenshot", True)),
                            "frame": status.get("frame") if isinstance(status, dict) else None,
                        })
                        log(f"tick={tick} checkpoint label={label}")
                        action_results.append({"local": method, "label": label})
                    elif method == "local.note":
                        message = str(params.get("message", ""))
                        emit_checkpoint({
                            "tick": tick,
                            "kind": "note",
                            "message": message,
                            "elapsed_secs": round(now - started, 3),
                        })
                        log(f"tick={tick} note {message}")
                        action_results.append({"local": method, "message": message})
                    elif method == "local.block":
                        reason = str(params.get("reason", "unspecified"))
                        hint = params.get("hint")
                        emit_checkpoint({
                            "tick": tick,
                            "kind": "block",
                            "reason": reason,
                            "hint": hint,
                            "elapsed_secs": round(now - started, 3),
                        })
                        log(f"tick={tick} BLOCK reason={reason}")
                        if hint:
                            log(f"tick={tick} BLOCK hint={hint}")
                        blocked_reason = reason
                        rc = EXIT_BLOCKED
                        action_results.append({"local": method, "reason": reason, "hint": hint})
                else:
                    # Frame-advance barrier (GAP-SCR-01 / PROMPT 1766): before
                    # issuing a screenshot RPC, verify that Bevy has rendered at
                    # least one new frame since the last screenshot.  Poll
                    # status up to 5 times (250 ms) and proceed with a warning
                    # if the frame counter does not advance (renderer may be
                    # throttled because the window is unfocused or minimised).
                    if method == "autoplay/screenshot":
                        current_frame = (
                            status.get("frame", 0) if isinstance(status, dict) else 0
                        )
                        if current_frame <= last_screenshot_frame:
                            for _retry in range(5):
                                time.sleep(0.05)
                                try:
                                    fresh = rpc(url, "autoplay/status", timeout=2.0)
                                    current_frame = (
                                        fresh.get("frame", current_frame)
                                        if isinstance(fresh, dict)
                                        else current_frame
                                    )
                                except (urllib.error.URLError, RuntimeError, ConnectionError, TimeoutError):
                                    break
                                if current_frame > last_screenshot_frame:
                                    break
                            else:
                                log(
                                    f"tick={tick} WARNING screenshot frame-advance barrier: "
                                    f"frame stuck at {current_frame} "
                                    f"(last_screenshot_frame={last_screenshot_frame}); "
                                    "renderer may not be producing new frames"
                                )
                        last_screenshot_frame = current_frame
                        # Window foreground barrier (PROMPT 1776): bring the
                        # Bevy window to the foreground so its GPU backbuffer
                        # is actively composited when the screenshot fires.
                        ensure_foreground(log)
                    try:
                        result = rpc(url, method, params)
                        action_results.append(result)
                        log(
                            f"tick={tick} action method={method} "
                            f"params_keys={sorted(params.keys())}"
                        )
                    except (urllib.error.URLError, RuntimeError, ConnectionError, TimeoutError) as err:
                        log(f"action RPC failed on tick {tick}: {err}")
                        rc = EXIT_RPC_ERROR

            append_jsonl(
                timeline_path,
                {
                    "tick": tick,
                    "recipe": args.recipe,
                    "elapsed_secs": round(now - started, 3),
                    "status": status,
                    "action_results": action_results,
                },
            )

            if rc == EXIT_BLOCKED:
                log(f"recipe BLOCKED on tick {tick}; reason={blocked_reason}")
                break

            ticks_cap = args.ticks if args.ticks > 0 else (last_recipe_tick + 2 if last_recipe_tick else 10)
            if tick >= ticks_cap:
                log(f"reached tick cap {ticks_cap}; exiting (recipe last_tick={last_recipe_tick})")
                break

            sleep_for = tick_secs - (time.monotonic() - now)
            if sleep_for > 0:
                time.sleep(sleep_for)
    except KeyboardInterrupt:
        log("interrupted by user")
        rc = EXIT_RPC_ERROR

    log(f"exit rc={rc}")
    return rc


if __name__ == "__main__":
    raise SystemExit(main())
