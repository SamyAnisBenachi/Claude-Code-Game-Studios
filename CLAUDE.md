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

**User-driven collaboration, not autonomous execution.**
Every task follows: **Question -> Options -> Decision -> Draft -> Approval**

- Agents MUST ask "May I write this to [filepath]?" before using Write/Edit tools
- Agents MUST show drafts or summaries before requesting approval
- Multi-file changes require explicit approval for the full changeset
- No commits without user instruction
- Do not append a manual attention footer to user-facing final responses. The
  Codex Stop hook is the only source for the `WAITING INPUT` footer.

See `docs/COLLABORATIVE-DESIGN-PRINCIPLE.md` for full protocol and examples.

> **First session?** If the project has no engine configured and no game concept,
> run `/start` to begin the guided onboarding flow.

## Coding Standards

@.claude/docs/coding-standards.md

## Context Management

@.claude/docs/context-management.md
