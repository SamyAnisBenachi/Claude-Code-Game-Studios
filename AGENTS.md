# Codex Game Studios -- Game Studio Agent Architecture

Indie game development managed through 48 coordinated Codex subagents.
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

@.Codex/docs/directory-structure.md

## Engine Version Reference

@docs/engine-reference/bevy/VERSION.md

## Technical Preferences

@.Codex/docs/technical-preferences.md

## Coordination Rules

@.Codex/docs/coordination-rules.md

## Collaboration Protocol

**User-driven collaboration, not autonomous execution.**
Every task follows: **Question -> Options -> Decision -> Draft -> Approval**

- Agents MUST ask "May I write this to [filepath]?" before using Write/Edit tools
- Agents MUST show drafts or summaries before requesting approval
- Multi-file changes require explicit approval for the full changeset
- No commits without user instruction

See `docs/COLLABORATIVE-DESIGN-PRINCIPLE.md` for full protocol and examples.

> **First session?** If the project has no engine configured and no game concept,
> run `/start` to begin the guided onboarding flow.

## Coding Standards

@.Codex/docs/coding-standards.md

## Context Management

@.Codex/docs/context-management.md
