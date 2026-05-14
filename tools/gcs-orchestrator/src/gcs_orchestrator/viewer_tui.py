"""Textual TUI viewer for codex app-server orchestrator session.

Replaces the bare cmd-window streaming viewer with a proper TUI:
- Always-visible status footer (connection LED, token gauge, turn elapsed)
- Filtered transcript: only agent deltas + turn lifecycle + user input
  (hook/started, hook/completed, item/completed shown compactly
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
    from rich.markup import escape as _rich_escape
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
            # In-flight line buffer: Codex streams deltas of 2-8 chars; each
            # RichLog.write() forces a new line. We accumulate until we see
            # a '\n', then write the complete line — preserving paragraph
            # shape and letting RichLog's wrap handle long lines.
            self._delta_line_buf = ""
            # Speaker tracking: when a userMessage item arrives, we need to
            # know whether WE typed it locally (already echoed as "user>")
            # or it was injected externally via the relay (worker DONE).
            # Set True when local Input is submitted; cleared on first
            # item/started userMessage that arrives after.
            self._local_turn_pending = False
            # True once we've emitted the orchestrator> header for the
            # current turn (cleared on turn/started).
            self._orch_header_emitted = False
            # Tracks whether the last line written was blank — used to
            # collapse runs of empty lines (Codex often streams "\n\n\n…"
            # as paragraph separators which render as huge gaps).
            self._last_line_was_blank = False
            self.bus: Optional[CodexEventBus] = None
            # Token budget — model dependent; default 200k
            self.token_budget = 200_000
            # Rolling p50/p95 latency summary, refreshed every 5s
            self._metrics_summary = ""

        def compose(self) -> ComposeResult:
            yield Static(id="status_bar")
            yield RichLog(id="transcript", wrap=True, highlight=True, markup=True)
            yield Input(id="input", placeholder="Type a turn and press Enter…")
            yield Footer()

        def on_mount(self) -> None:
            self._refresh_status_bar()
            self._refresh_metrics_summary()
            self.set_interval(1.0, self._refresh_status_bar)
            self.set_interval(5.0, self._refresh_metrics_summary)
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
            # Context budget gauge: uses LAST turn's input+output as proxy
            # for "current context window usage" (the running session's
            # context is approximately the last turn's input prefix, since
            # Codex feeds the rolling history each turn).
            # `total.totalTokens` is cumulative-since-session-start and is
            # NOT comparable to the 200k context window.
            ctx_used = self.turn.input_tokens + self.turn.output_tokens
            budget = self.token_budget
            pct = (ctx_used / budget * 100) if budget else 0
            bar_color = "green" if pct < 60 else ("yellow" if pct < 85 else "red")
            cache_pct = (
                self.turn.cached_input_tokens / self.turn.input_tokens * 100
                if self.turn.input_tokens else 0
            )

            # Compact format that survives 80-column terminals
            parts = [led]
            if self.turn.in_progress and self.turn.started_at:
                elapsed = int(time.time() - self.turn.started_at)
                parts.append(f"[cyan]●{elapsed // 60:02d}:{elapsed % 60:02d}[/cyan]")
            # Token counts in k notation when large
            def _k(n: int) -> str:
                return f"{n / 1000:.1f}k" if n >= 1000 else str(n)
            parts.append(
                f"last in={_k(self.turn.input_tokens)} "
                f"(c{cache_pct:.0f}%) out={_k(self.turn.output_tokens)}"
            )
            parts.append(
                f"[{bar_color}]ctx {_k(ctx_used)}/{_k(budget)} ({pct:.0f}%)[/{bar_color}]"
            )
            parts.append(f"[dim]Σ{self.turn.total_thread_tokens / 1_000_000:.1f}M[/dim]")
            # p50/p95 from metrics.jsonl (refreshed via _refresh_metrics_summary every 5s)
            if self._metrics_summary:
                parts.append(f"[dim]{self._metrics_summary}[/dim]")
            tid_short = self.thread_id[:8] if self.thread_id else "?"
            parts.append(f"[dim]{tid_short}[/dim]")
            bar.update(" │ ".join(parts))

        def _refresh_metrics_summary(self) -> None:
            """Read p50/p95 from metrics.jsonl. Called every 5s by timer."""
            try:
                from . import metrics as _metrics
                self._metrics_summary = _metrics.summary_line(n=20)
            except Exception:
                self._metrics_summary = ""

        def _handle_status(self, status: str) -> None:
            self.connection_status = status
            log = self.query_one("#transcript", RichLog)
            color = {"connected": "green", "disconnected": "red"}.get(status, "yellow")
            log.write(f"[dim][[{color}]{status}[/{color}]][/dim]")
            self._refresh_status_bar()

        # ---- Notification dispatch ----

        def _write_line(self, log: "RichLog", content: str, *, is_blank: bool = False) -> None:
            """log.write wrapper that collapses runs of blank lines to max 1."""
            if is_blank:
                if self._last_line_was_blank:
                    return
                self._last_line_was_blank = True
                log.write("")
            else:
                self._last_line_was_blank = False
                log.write(content, scroll_end=True)

        def _handle_event(self, d: dict) -> None:
            log = self.query_one("#transcript", RichLog)
            method = d.get("method", "")
            params = d.get("params") or {}

            if method == "item/agentMessage/delta":
                delta = params.get("delta", "")
                if isinstance(delta, str):
                    # Emit orchestrator> header once per turn, on first agent delta
                    if not self._orch_header_emitted:
                        self._write_line(log, "[bold green]orchestrator>[/bold green]")
                        self._orch_header_emitted = True
                    self._current_agent_buffer += delta
                    self._delta_line_buf += delta
                    # Emit only complete lines; trailing partial stays buffered.
                    # Blank lines are collapsed by _write_line to avoid huge
                    # vertical gaps when Codex streams paragraph separators.
                    while "\n" in self._delta_line_buf:
                        line, self._delta_line_buf = self._delta_line_buf.split("\n", 1)
                        if line.strip():
                            self._write_line(log, f"[white]{_rich_escape(line)}[/white]")
                        else:
                            self._write_line(log, "", is_blank=True)

            elif method == "turn/started":
                self.turn = TurnState(
                    turn_id=params.get("turn", {}).get("id", ""),
                    started_at=time.time(),
                    in_progress=True,
                )
                self._current_agent_buffer = ""
                self._delta_line_buf = ""
                self._orch_header_emitted = False

            elif method == "turn/completed":
                # Flush any trailing partial line before the turn-done marker
                if self._delta_line_buf:
                    if self._delta_line_buf.strip():
                        self._write_line(log, f"[white]{_rich_escape(self._delta_line_buf)}[/white]")
                    self._delta_line_buf = ""
                turn = params.get("turn", {}) or {}
                status = turn.get("status") or "completed"
                elapsed = ""
                if self.turn.started_at:
                    e = time.time() - self.turn.started_at
                    elapsed = f" · {e:.1f}s"
                tok = f" · in={self.turn.input_tokens} out={self.turn.output_tokens}"
                if status == "completed":
                    self._write_line(log, f"[dim]── done{elapsed}{tok} ──[/dim]")
                else:
                    err = turn.get("error") or {}
                    msg = err.get("message", "") if isinstance(err, dict) else str(err)
                    self._write_line(log, f"[red]── {status}{elapsed}: {_rich_escape(msg[:200])} ──[/red]")
                self._write_line(log, "", is_blank=True)
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
                    if self._local_turn_pending:
                        # We already echoed it locally as "user>" — don't double
                        self._local_turn_pending = False
                    else:
                        # External injection (relay → worker DONE, or another client)
                        # Extract text from content array
                        text = ""
                        for c in item.get("content", []) or []:
                            if isinstance(c, dict):
                                text += (c.get("text") or c.get("input_text") or "")
                            elif isinstance(c, str):
                                text += c
                        # Detect worker prompt ID for a clearer label
                        import re
                        m = re.search(r"PROMPT[-\s]?(\d+)", text[:300])
                        label = f"worker PROMPT-{m.group(1)}" if m else "external"
                        preview = text.strip()[:200].replace("\n", " ⏎ ")
                        if len(text.strip()) > 200:
                            preview += "…"
                        self._write_line(log, f"[bold yellow]{label}>[/bold yellow] [dim]{_rich_escape(preview)}[/dim]")
                elif it == "commandExecution":
                    cmd = item.get("command", "")[:150]
                    self._write_line(log, f"[magenta][exec][/magenta] [dim]{_rich_escape(cmd)}[/dim]")
                elif it == "reasoning":
                    self._write_line(log, "[dim][thinking…][/dim]")

            elif method == "item/commandExecution/outputDelta":
                chunk = params.get("chunk", "") or params.get("delta", "")
                if chunk and chunk.strip():
                    self._write_line(log, f"[dim]{_rich_escape(chunk.rstrip())}[/dim]")

            elif method == "hook/started":
                hook = params.get("hook", {}) or {}
                name = hook.get("name") or hook.get("id") or "?"
                self._write_line(log, f"[dim cyan][hook ▸ {_rich_escape(str(name))}][/dim cyan]")

            elif method == "hook/completed":
                hook = params.get("hook", {}) or {}
                name = hook.get("name") or hook.get("id") or "?"
                exit_code = hook.get("exitCode")
                dur = hook.get("durationMs")
                bits = [f"hook ◂ {_rich_escape(str(name))}"]
                if exit_code is not None:
                    bits.append(f"exit={exit_code}")
                if dur is not None:
                    bits.append(f"{dur}ms")
                color = "dim cyan" if (exit_code in (0, None)) else "red"
                self._write_line(log, f"[{color}][{' '.join(bits)}][/{color}]")

            elif method == "item/completed":
                item = params.get("item", {}) or {}
                it = item.get("type", "?")
                # commandExecution completion: show exit status compactly
                if it == "commandExecution":
                    exit_code = item.get("exitCode")
                    if exit_code is not None:
                        color = "dim" if exit_code == 0 else "red"
                        self._write_line(log, f"[{color}][exec ◂ exit={exit_code}][/{color}]")

            elif method == "thread/status/changed":
                status = params.get("status") or "?"
                # status may be a dict like {"type": "active", "activeFlags": []}
                if isinstance(status, dict):
                    label = status.get("type") or "?"
                    flags = status.get("activeFlags") or []
                    if flags:
                        label += f"({','.join(map(str, flags))})"
                else:
                    label = str(status)
                self._write_line(log, f"[dim][thread ▸ {_rich_escape(label)}][/dim]")

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
            # Mark so the upcoming item/started userMessage echo from server
            # is suppressed (we already rendered it locally below)
            self._local_turn_pending = True
            log = self.query_one("#transcript", RichLog)
            self._write_line(log, f"[bold blue]user>[/bold blue] {_rich_escape(line)}")

        def action_interrupt_turn(self) -> None:
            if self.bus is None or not self.turn.in_progress or not self.turn.turn_id:
                return
            self.bus.send("turn/interrupt", {"threadId": self.thread_id, "turnId": self.turn.turn_id})
            log = self.query_one("#transcript", RichLog)
            log.write("[yellow][interrupting…][/yellow]")

        def action_copy_last(self) -> None:
            log = self.query_one("#transcript", RichLog)
            if not self.last_agent_message:
                log.write("[yellow][copy] no agent message buffered yet — wait for first [turn done][/yellow]")
                return
            try:
                import pyperclip
                pyperclip.copy(self.last_agent_message)
                preview = self.last_agent_message[:40].replace("\n", " ")
                log.write(f"[green][copy] {len(self.last_agent_message)} chars → clipboard ({_rich_escape(repr(preview))}…)[/green]")
            except Exception as exc:
                # Fallback: dump to log so user can manually shift+drag-select it
                log.write(f"[red][copy] pyperclip failed: {_rich_escape(str(exc))}[/red]")
                log.write(f"[dim]--- copy below this line ---[/dim]")
                log.write(_rich_escape(self.last_agent_message))
                log.write(f"[dim]--- end copy ---[/dim]")


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
