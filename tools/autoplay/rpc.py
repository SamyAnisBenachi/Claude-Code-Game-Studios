#!/usr/bin/env python3
"""One-shot RPC helper for the CCGS autoplay surface.

PROMPT 1595 -- the *interactive* counterpart to `driver.py`. Use this for
ad-hoc pokes (status check, single input, screenshot, capabilities probe).
Drive gameplay through `driver.py` — do NOT loop this helper in a shell.

Examples:
    python tools/autoplay/rpc.py capabilities
    python tools/autoplay/rpc.py status
    python tools/autoplay/rpc.py screenshot --reason debug
    python tools/autoplay/rpc.py input --keys-down KeyA --cursor 100 200
    python tools/autoplay/rpc.py input --mouse-down Left
    python tools/autoplay/rpc.py clear
"""

from __future__ import annotations

import argparse
import json
import os
import sys
import urllib.error
import urllib.request
from typing import Any


def rpc(url: str, method: str, params: dict[str, Any] | None = None) -> Any:
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
    with urllib.request.urlopen(request, timeout=5) as response:
        return json.loads(response.read().decode("utf-8"))


def main() -> int:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument(
        "--port",
        type=int,
        default=int(os.environ.get("CCGS_AUTOPLAY_PORT", "15873")),
    )
    sub = p.add_subparsers(dest="cmd", required=True)
    sub.add_parser("capabilities")
    sub.add_parser("status")
    sub.add_parser("clear")
    screenshot = sub.add_parser("screenshot")
    screenshot.add_argument("--reason", default="rpc")
    inp = sub.add_parser("input")
    inp.add_argument("--keys-down", nargs="*", default=[])
    inp.add_argument("--keys-up", nargs="*", default=[])
    inp.add_argument("--mouse-down", nargs="*", default=[])
    inp.add_argument("--mouse-up", nargs="*", default=[])
    inp.add_argument("--cursor", nargs=2, type=float, metavar=("X", "Y"))
    inp.add_argument("--scroll", nargs=2, type=float, metavar=("DX", "DY"))

    args = p.parse_args()
    url = f"http://127.0.0.1:{args.port}/"

    method: str
    params: dict[str, Any] = {}
    if args.cmd == "capabilities":
        method = "autoplay/capabilities"
    elif args.cmd == "status":
        method = "autoplay/status"
    elif args.cmd == "clear":
        method = "autoplay/clear_input"
    elif args.cmd == "screenshot":
        method = "autoplay/screenshot"
        params = {"reason": args.reason}
    elif args.cmd == "input":
        method = "autoplay/input"
        if args.keys_down:
            params["keys_down"] = args.keys_down
        if args.keys_up:
            params["keys_up"] = args.keys_up
        if args.mouse_down:
            params["mouse_down"] = args.mouse_down
        if args.mouse_up:
            params["mouse_up"] = args.mouse_up
        if args.cursor is not None:
            params["cursor"] = {"screen": [args.cursor[0], args.cursor[1]]}
        if args.scroll is not None:
            params["scroll"] = [args.scroll[0], args.scroll[1]]
        if not params:
            p.error("input requires at least one of --keys-down/--keys-up/--mouse-down/--mouse-up/--cursor/--scroll")
    else:
        p.error(f"unknown cmd: {args.cmd}")
        return 2

    try:
        body = rpc(url, method, params)
    except (urllib.error.URLError, ConnectionError, TimeoutError) as err:
        print(json.dumps({"transport_error": str(err)}, indent=2), file=sys.stderr)
        return 1
    print(json.dumps(body, indent=2))
    if isinstance(body, dict) and "error" in body:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
