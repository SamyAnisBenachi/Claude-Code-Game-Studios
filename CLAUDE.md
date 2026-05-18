# Claude Code Game Studios -- Game Studio Agent Architecture

Indie game development managed through 48 coordinated Claude Code subagents.
Each agent owns a specific domain, enforcing separation of concerns and quality.

## Technology Stack

- **Engine**: Bevy 0.18 (Rust)
- **Language**: Rust (stable toolchain)
- **Version Control**: Git with trunk-based development
- **Build System**: Cargo + Trunk (WASM client) / Cargo (headless server)
- **Asset Pipeline**: bevy_asset_loader + TextureAtlas (sprite sheets)
- **Networking**: Lightyear (bevy_lightyear) — client/server over WebSocket/WebTransport
- **UI**: bevy_ui (0.18 Required Components API) + bevy_tweening
- **Deployment**: WASM client → Vercel / Rust server → Railway (Docker)

> **Note**: Use `liv-bevy-018` skill for ALL Bevy code. Use `liv-bevy-lightyear`
> skill for ALL networking code. These skills enforce correct 0.18 API patterns.

## Project Structure

@.claude/docs/directory-structure.md

## Engine Version Reference

@docs/engine-reference/bevy/VERSION.md

## Technical Preferences

@.claude/docs/technical-preferences.md

## Coordination Rules

@.claude/docs/coordination-rules.md

## Orchestrator Runtime

The Codex orchestrator runs in `codex app-server` mode (long-running JSON-RPC
on `ws://127.0.0.1:9787`) since 2026-05-14, not the legacy `codex resume`
TUI. User interaction goes through `C:/Users/Sam/.codex/gcs-app-viewer.py`
and worker DONE reports through `C:/Users/Sam/.codex/gcs-app-relay.py`. The
toggle file `C:/Users/Sam/.codex/gcs-mode` (`relay` vs missing) and
`C:/Users/Sam/.codex/gcs-orch-session-id` (current thread UUID) drive the
dispatcher (`C:/Users/Sam/.codex/gcs-octogent-dispatch.py`).

See `docs/octogent-integration.md` Section 9-bis for full architecture,
boot procedure, troubleshooting, rollback, and verified Codex CLI version.

## Collaboration Protocol

**Default-to-acting mode (2026-05-18 user-confirmed override).** The user
runs many parallel sessions and has explicitly asked to remove
end-of-turn permission friction. Apply these rules:

- **Don't end turns with "should I commit?" / "want me to push?" / "OK?"**
  If the next step is obvious from prior context, just do it and report
  what you did. Surface alternatives in the turn output ("I went with
  X; alternatives were Y, Z — say so if you want a different path")
  rather than blocking on a choice.
- **Don't call `AskUserQuestion` for low-stakes choices.** Pick the
  most defensible option based on project rules + recent context and
  continue. Reserve real `AskUserQuestion` calls (or a pause-and-ask
  in turn output) for the destructive list below.
- **`permissions.defaultMode = "bypassPermissions"`** is set in
  `.claude/settings.json`. Tool-permission popups are silent — you
  still respect the `deny` list, but allow/ask prompts skip the
  human.
- **Codex CLI** runs with `approval_policy = "on-failure"` and
  `sandbox = "danger-full-access"` (in `~/.codex/config.toml`). Same
  posture: no friction unless something actually breaks.

**Still pause-and-ask before:**

- `rm -rf` of anything non-trivial
- `git push --force` (especially to `main` / integration branches)
- `git reset --hard`, `git clean -f`
- Dropping data, truncating tables, deleting prod assets
- Branching off `main` without explicit intent
- Mutating user-environment files outside the project
  (e.g. `~/.codex/config.toml`, system-wide settings)

These three guardrails compose: bypassPermissions removes Type-A
runtime friction; the "don't ask" rule above removes Type-B
conversational friction; the destructive list keeps the genuine
guardrails in place. See also the auto-memory entry
`feedback_default_to_acting.md` for the rationale.

**Historical fallback (pre-override, for reference):**
The original protocol was "Question → Options → Decision → Draft →
Approval" with mandatory `May I write this to [filepath]?` before
Edit/Write. That is now superseded for non-destructive work; agents
who default to acting and report cleanly are preferred over agents
who ask permission before every save. The Codex Stop hook remains
the only source for the `WAITING INPUT` footer; do not append a
manual attention footer to final responses.

See `docs/COLLABORATIVE-DESIGN-PRINCIPLE.md` for full protocol and examples.

> **First session?** If the project has no engine configured and no game concept,
> run `/start` to begin the guided onboarding flow.

## Coding Standards

@.claude/docs/coding-standards.md

## Context Management

@.claude/docs/context-management.md
