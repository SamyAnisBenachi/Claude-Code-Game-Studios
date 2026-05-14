"""Textual TUI viewer for codex app-server orchestrator session.

Replaces the bare cmd-window streaming viewer with a proper TUI:
- Always-visible status footer (connection LED, token gauge, turn elapsed)
- Filtered transcript: only agent deltas + turn lifecycle + user input
  (noise like hook/started, hook/completed, item/completed suppressed
   from main stream)
- Esc → turn/interrupt for the active turn
- Ctrl+Y → copy last agent message to clipboard (pyperclip)
- Ctrl+Q → quit
- Auto-reconnect on WS disconnect with backoff
- Visible status when the connection dies (red banner, not silent no-op)

Run:
    python -m gcs_orchestrator.viewer_tui
    # uses orchestrator.session_id from gcs.toml

Or override:
    python -m gcs_orchestrator.viewer_tui <thread-id>
"""
from __future__ import annotations

import asyncio
import json
import sys
import threading
import time
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional

import websocket  # websocket-client (sync)

from .config import load as load_config

try:
    from textual import on, work
    from textual.app import App, ComposeResult
    from textual.binding import Binding
    from textual.containers import Vertical
    from textual.reactive import reactive
    from textual.widgets import Footer, Header, Input, RichLog, Static
    HAS_TEXTUAL = True
except ImportError:  # pragma: no cover
    HAS_TEXTUAL = False


@dataclass
class TurnState:
    turn_id: str = ""
    started_at: float = 0.0
    in_progress: bool = False
    input_tokens: int = 0
    cached_input_tokens: int = 0
    output_tokens: int = 0
    total_thread_tokens: int = 0


class CodexEventBus(threading.Thread):
    """Background thread: WS connect/reconnect + dispatch notifications
    to the Textual app's call_from_thread.
    """

    def __init__(self, ws_url: str, thread_id: str, cwd_override: Optional[str],
                 on_event, on_connection_status) -> None:
        super().__init__(daemon=True)
        self.ws_url = ws_url
        self.thread_id = thread_id
        self.cwd_override = cwd_override
        self.on_event = on_event  # callable: (dict event) -> None (Textual-thread-safe)
        self.on_status = on_connection_status  # callable: (str status) -> None
        self.ws: Optional[websocket.WebSocket] = None
        self._send_lock = threading.Lock()
        self._next_id = 0
        self._stop_evt = threading.Event()
        self._reconnect_evt = threading.Event()
        # NOTE: do NOT name this `_initialized` — that's a private flag on
        # threading.Thread itself and clobbering it makes start() raise
        # RuntimeError("thread.__init__() not called").
        self._ws_initialized = False

    def stop(self) -> None:
        self._stop_evt.set()
        if self.ws is not None:
            try:
                self.ws.close()
            except Exception:
                pass

    def _next(self) -> int:
        self._next_id += 1
        return self._next_id

    def send(self, method: str, params: Optional[dict] = None, rid: Optional[int] = None) -> int:
        """Send a JSON-RPC request from the main thread."""
        if self.ws is None:
            return -1
        if rid is None:
            rid = self._next()
        msg: dict = {"jsonrpc": "2.0", "id": rid, "method": method}
        if params is not None:
            msg["params"] = params
        try:
            with self._send_lock:
                self.ws.send(json.dumps(msg))
        except Exception as exc:
            self.on_status(f"send error: {exc}")
            self._reconnect_evt.set()
        return rid

    def run(self) -> None:
        backoff = 1.0
        while not self._stop_evt.is_set():
            try:
                self.on_status("connecting…")
                self.ws = websocket.WebSocket()
                self.ws.settimeout(30.0)
                self.ws.connect(self.ws_url, suppress_origin=True)
                try:
                    import socket
                    self.ws.sock.setsockopt(socket.SOL_SOCKET, socket.SO_KEEPALIVE, 1)
                except Exception:
                    pass
                self.on_status("connected")
                backoff = 1.0

                # Initialize
                rid = self.send("initialize", {
                    "clientInfo": {"name": "gcs-viewer-tui", "title": "GCS TUI", "version": "0.1"}
                })
                # Resume thread
                params: dict = {"threadId": self.thread_id}
                if self.cwd_override:
                    params["cwd"] = self.cwd_override
                self.send("thread/resume", params)
                self._ws_initialized = True

                # Read loop
                while not self._stop_evt.is_set() and not self._reconnect_evt.is_set():
                    try:
                        self.ws.settimeout(2.0)
                        raw = self.ws.recv()
                    except websocket.WebSocketTimeoutException:
                        continue
                    except (websocket.WebSocketConnectionClosedException, ConnectionResetError, OSError) as exc:
                        self.on_status(f"disconnected: {type(exc).__name__}")
                        break
                    if not raw:
                        continue
                    try:
                        d = json.loads(raw)
                    except json.JSONDecodeError:
                        continue
                    self.on_event(d)
            except Exception as exc:
                self.on_status(f"connect failed: {exc}")

            if self._stop_evt.is_set():
                return
            self._reconnect_evt.clear()
            self._ws_initialized = False
            self.on_status(f"reconnecting in {backoff:.0f}s…")
            time.sleep(backoff)
            backoff = min(backoff * 2, 30.0)


if HAS_TEXTUAL:

    class GcsViewerApp(App):
        """Textual TUI for the Codex orchestrator session."""

        CSS = """
        Screen { layout: vertical; }

        #status_bar {
            dock: top;
            height: 1;
            background: $panel;
            color: $text;
            padding: 0 1;
        }

        #transcript {
            height: 1fr;
            background: $surface;
        }

        #input {
            dock: bottom;
            height: 3;
            border: solid $accent;
        }
        """

        BINDINGS = [
            Binding("escape", "interrupt_turn", "Interrupt", show=True),
            Binding("ctrl+y", "copy_last", "Copy last", show=True),
            Binding("ctrl+q", "quit", "Quit", show=True),
        ]

        connection_status = reactive("disconnected")

        def __init__(self, ws_url: str, thread_id: str, cwd_override: Optional[str] = None) -> None:
            super().__init__()
            self.ws_url = ws_url
            self.thread_id = thread_id
            self.cwd_override = cwd_override
            self.turn = TurnState()
            self.last_agent_message = ""
            self._current_agent_buffer = ""
            self.bus: Optional[CodexEventBus] = None
            # Token budget — model dependent; default 200k
            self.token_budget = 200_000

        def compose(self) -> ComposeResult:
            yield Static(id="status_bar")
            yield RichLog(id="transcript", wrap=True, highlight=True, markup=True)
            yield Input(id="input", placeholder="Type a turn and press Enter…")
            yield Footer()

        def on_mount(self) -> None:
            self._refresh_status_bar()
            self.set_interval(1.0, self._refresh_status_bar)
            self.bus = CodexEventBus(
                ws_url=self.ws_url,
                thread_id=self.thread_id,
                cwd_override=self.cwd_override,
                on_event=lambda d: self.call_from_thread(self._handle_event, d),
                on_connection_status=lambda s: self.call_from_thread(self._handle_status, s),
            )
            self.bus.start()
            self.query_one("#input", Input).focus()

        def on_unmount(self) -> None:
            if self.bus is not None:
                self.bus.stop()

        # ---- Status bar ----

        def _refresh_status_bar(self) -> None:
            bar = self.query_one("#status_bar", Static)
            led = {"connected": "[green]●[/green]",
                   "connecting…": "[yellow]●[/yellow]",
                   "disconnected": "[red]●[/red]"}.get(self.connection_status,
                                                       "[yellow]●[/yellow]")
            turn_str = ""
            if self.turn.in_progress and self.turn.started_at:
                elapsed = int(time.time() - self.turn.started_at)
                turn_str = f"[cyan]turn {elapsed // 60:02d}:{elapsed % 60:02d}[/cyan]"
            tokens = self.turn.total_thread_tokens
            budget = self.token_budget
            pct = (tokens / budget * 100) if budget else 0
            bar_color = "green" if pct < 60 else ("yellow" if pct < 85 else "red")
            tok_str = (f"tokens in={self.turn.input_tokens:,} cached={self.turn.cached_input_tokens:,} "
                       f"out={self.turn.output_tokens:,} | "
                       f"[{bar_color}]thread {tokens:,}/{budget:,} ({pct:.0f}%)[/{bar_color}]")
            tid_short = self.thread_id[:8] if self.thread_id else "?"
            bar.update(f"{led} thread {tid_short} | {turn_str} | {tok_str}")

        def _handle_status(self, status: str) -> None:
            self.connection_status = status
            log = self.query_one("#transcript", RichLog)
            color = {"connected": "green", "disconnected": "red"}.get(status, "yellow")
            log.write(f"[dim][[{color}]{status}[/{color}]][/dim]")
            self._refresh_status_bar()

        # ---- Notification dispatch ----

        def _handle_event(self, d: dict) -> None:
            log = self.query_one("#transcript", RichLog)
            method = d.get("method", "")
            params = d.get("params") or {}

            if method == "item/agentMessage/delta":
                delta = params.get("delta", "")
                if isinstance(delta, str):
                    self._current_agent_buffer += delta
                    log.write(f"[white]{delta}[/white]", scroll_end=True)

            elif method == "turn/started":
                self.turn = TurnState(
                    turn_id=params.get("turn", {}).get("id", ""),
                    started_at=time.time(),
                    in_progress=True,
                )
                self._current_agent_buffer = ""
                log.write("\n[cyan][turn started][/cyan]")

            elif method == "turn/completed":
                turn = params.get("turn", {}) or {}
                status = turn.get("status") or "completed"
                if status == "completed":
                    log.write("[cyan][turn done][/cyan]\n")
                else:
                    err = turn.get("error") or {}
                    msg = err.get("message", "") if isinstance(err, dict) else str(err)
                    log.write(f"[red][turn {status}: {msg[:200]}][/red]\n")
                self.last_agent_message = self._current_agent_buffer
                self.turn.in_progress = False

            elif method == "thread/tokenUsage/updated":
                tu = (params.get("tokenUsage") or {}).get("last", {}) or {}
                self.turn.input_tokens = tu.get("inputTokens", self.turn.input_tokens)
                self.turn.cached_input_tokens = tu.get("cachedInputTokens", self.turn.cached_input_tokens)
                self.turn.output_tokens = tu.get("outputTokens", self.turn.output_tokens)
                total = (params.get("tokenUsage") or {}).get("total", {}) or {}
                self.turn.total_thread_tokens = total.get("totalTokens", self.turn.total_thread_tokens)

            elif method == "item/started":
                item = params.get("item", {})
                it = item.get("type", "?")
                if it == "userMessage":
                    content = item.get("content") or []
                    text = ""
                    for c in content:
                        if isinstance(c, dict):
                            text += c.get("text", "") or c.get("input_text", "")
                    if text:
                        log.write(f"[blue]user>[/blue] {text[:500]}")
                elif it == "commandExecution":
                    cmd = item.get("command", "")[:150]
                    log.write(f"[magenta][exec][/magenta] [dim]{cmd}[/dim]")
                elif it == "reasoning":
                    log.write("[dim][thinking…][/dim]")
                # Suppress agentMessage start (we render via delta)

            elif method == "item/commandExecution/outputDelta":
                chunk = params.get("chunk", "") or params.get("delta", "")
                if chunk:
                    log.write(f"[dim]{chunk.rstrip()}[/dim]")

            # Drop noise: hook/started, hook/completed, item/completed,
            # thread/status/changed, account/rateLimits/updated

            self._refresh_status_bar()

        # ---- Input + actions ----

        @on(Input.Submitted, "#input")
        def _on_submit(self, event: Input.Submitted) -> None:
            line = event.value.strip()
            self.query_one("#input", Input).value = ""
            if not line:
                return
            if self.bus is None:
                return
            params: dict = {
                "threadId": self.thread_id,
                "input": [{"type": "text", "text": line}],
            }
            if self.cwd_override:
                params["cwd"] = self.cwd_override
            self.bus.send("turn/start", params)
            log = self.query_one("#transcript", RichLog)
            log.write(f"[blue]user>[/blue] {line}")

        def action_interrupt_turn(self) -> None:
            if self.bus is None or not self.turn.in_progress or not self.turn.turn_id:
                return
            self.bus.send("turn/interrupt", {"threadId": self.thread_id, "turnId": self.turn.turn_id})
            log = self.query_one("#transcript", RichLog)
            log.write("[yellow][interrupting…][/yellow]")

        def action_copy_last(self) -> None:
            if not self.last_agent_message:
                return
            try:
                import pyperclip
                pyperclip.copy(self.last_agent_message)
                log = self.query_one("#transcript", RichLog)
                log.write(f"[dim][copied {len(self.last_agent_message)} chars to clipboard][/dim]")
            except Exception as exc:
                log = self.query_one("#transcript", RichLog)
                log.write(f"[red][copy failed: {exc}][/red]")


def main(argv: Optional[list[str]] = None) -> int:
    if not HAS_TEXTUAL:
        sys.stderr.write("textual not installed. Run: pip install -e .[tui]\n")
        return 2

    cfg = load_config()
    argv = argv if argv is not None else sys.argv[1:]
    thread_id = argv[0] if argv else cfg.orchestrator.session_id
    if not thread_id:
        sys.stderr.write("no thread id — pass as argv or set orchestrator.session_id in gcs.toml\n")
        return 2

    cwd = cfg.orchestrator.cwd_override or None
    app = GcsViewerApp(cfg.transport.ws_url, thread_id, cwd_override=cwd)
    app.run()
    return 0


if __name__ == "__main__":
    sys.exit(main())
