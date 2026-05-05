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

## Collaboration Protocol

**User-driven collaboration, not autonomous execution.**
Every task follows: **Question -> Options -> Decision -> Draft -> Approval**

- Agents MUST ask "May I write this to [filepath]?" before using Write/Edit tools
- Agents MUST show drafts or summaries before requesting approval
- Multi-file changes require explicit approval for the full changeset
- No commits without user instruction
- User-facing final responses MUST append this exact line as the final line when
  control returns to the user:

  `============ WAITING INPUT ============`

  Render this footer in red when the interface supports ANSI or styled output.
  The Codex Stop hook also emits this footer in bold red.

  Exception: do not append this footer to machine-readable outputs that must
  remain strict JSON, especially guardian/approval reviewer responses or tool
  protocol payloads.

See `docs/COLLABORATIVE-DESIGN-PRINCIPLE.md` for full protocol and examples.

> **First session?** If the project has no engine configured and no game concept,
> run `/start` to begin the guided onboarding flow.

## Coding Standards

@.claude/docs/coding-standards.md

## Context Management

@.claude/docs/context-management.md
