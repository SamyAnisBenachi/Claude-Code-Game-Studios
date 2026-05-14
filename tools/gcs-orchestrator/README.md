# gcs-orchestrator

Codex orchestrator stack for GCS — runs a long-running `codex app-server`
as the orchestrator's runtime, with a Textual-based viewer, a single-shot
worker DONE relay, and Octogent dispatcher integration.

## Modules

- `gcs_orchestrator.transport` — protocol-agnostic Codex transport adapter
  (current impl: `AppServerTransport` over WebSocket; pluggable for future)
- `gcs_orchestrator.config` — unified `gcs.toml` config with pydantic validation
- `gcs_orchestrator.viewer` — interactive viewer/typer
- `gcs_orchestrator.relay` — single-shot worker DONE relay (file-lock + idempotency)
- `gcs_orchestrator.dispatch` — Octogent dispatcher with mode-toggle
- `gcs_orchestrator.spawn_watchdog` — per-spawn watchdog for worker initialPrompt delivery
- `gcs_orchestrator.history` — relay-receipts timeline CLI

## Installation

```cmd
pip install -e .[tui,dev]
```

Then `~/.codex/gcs-*.py` shims become one-liners that invoke
`python -m gcs_orchestrator.<entry>`.

## Configuration

Single source of truth: `~/.codex/gcs.toml`. See `config.py` for the schema.
Example in `templates/gcs.toml.example`.

## Tests

```cmd
pytest tests/
```

Tests cover:
- Dispatcher regex parsing (golden fixtures from real orchestrator outputs)
- Relay state machine (mock WS server)
- Config loader (pydantic validation matrix)
- Transport adapter contract (recorded JSON-RPC fixtures)

## Architecture

See `docs/octogent-integration.md` Section 9-bis in the parent repo.
