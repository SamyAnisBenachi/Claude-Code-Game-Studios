# Epic: Class System

> **Layer**: Feature (M3)
> **GDD**: design/gdd/class-system.md
> **Architecture Module**: `server/feature/class/` (`state.rs`, `lobby/handler.rs`, `resolution/effects.rs`, `board/spawn.rs`, `plugin.rs`)
> **Status**: Ready
> **Stories**: Not yet created — run `/create-stories class-system`

## Overview

Implements the six playable classes (Iop, Cra, Sacrier, Xelor, Ecaflip, Sadida) as a session-scoped player property. The epic owns `PlayerSessions` — a server-only `Resource` that stores `class: ClassId` and `class_locked: bool` per player — and the `SourceClass(ClassId)` ECS component applied to all 7 class-specific token entities. Class selection happens in LOBBY via `C2SClassChoice` (Lightyear message); locking is atomic at the LOBBY→DRAFT_INITIAL gate via `all_classes_chosen()` + `lock_all_classes()`. All 11 cross-system class-effect formulas (Gelure, Xelorium, Rollback, Garde-Temps, Miss Nuit, Sang Méprise, Punition, Sadida Seeds, Graines de Folie, Ecaflip dice, Coin flip) execute as plain Rust helper functions called from within the RESOLUTION system body — never as standalone Bevy systems, never via buffered Messages within a RESOLUTION tick. This preserves RESOLUTION sub-step ordering without frame-delay risk (ADR-014 §4 Decision).

## Governing ADRs

| ADR | Decision Summary | Engine Risk |
|-----|-----------------|-------------|
| ADR-014: Class System Architecture — PlayerSessionState, SourceClass Component, and Direct Effect Dispatch | `PlayerSessions` Resource (server-only HashMap); `SourceClass(ClassId)` component on tokens; `C2SClassChoice` LOBBY handler; class effects as plain Rust fns in RESOLUTION body; `PlayerSnapshot.class_id` + `UnitBoardState.source_class` snapshot fields | MEDIUM |

## Engine Notes

- **`C2SClassChoice` uses `lightyear::prelude::Message`** — NOT `bevy::prelude::Message`. Both traits exist simultaneously in this project. Use `lightyear::prelude::Message` explicitly.
- **`MessageReceiver<C2SClassChoice>.receive_messages()`** — Lightyear 0.26 server receive API. Pin exact Lightyear patch version in `Cargo.toml` and wrap in `server/src/lobby/handler.rs` as the sole drain point.
- **`Query::single()` returns `Result` in Bevy 0.16+.** Use `query.single()?` or `let Ok(x) = query.single()`.
- **`EventWriter`/`EventReader` do not exist in Bevy 0.17+.** Use `MessageWriter::write()` / `MessageReader::read()` with `app.add_message::<T>()`.
- **`ResMut<PlayerSessions>` in multiple RESOLUTION systems** — all must be in an explicit `.before()`/`.after()` ordering chain within the RESOLUTION `SystemSet`. Bevy's multi-threaded executor panics in debug builds on shared `ResMut<T>` without ordering.
- **`liv-bevy-018` + `liv-bevy-lightyear`** skills mandatory on every `.rs` file in this epic.

## Pre-Implementation Gates

Confirm all before any story begins:

1. **PIERCE keyword** — Verify PIERCE is defined in `keyword-system.md` and available in relevant card pools before Seed/Sadida stories (TR-CS-006) open. CS-7 designates PIERCE as the design counter to high-AR units; if undefined, flag as pre-implementation blocker.
2. **`garde_temps_used_this_game` counter** — Wired in Game Session System (owned by GSS, initialized at LOBBY→DRAFT_INITIAL, persists for session). Must exist before Garde-Temps story (TR-CS-003) begins.
3. **NP-1 through NP-8** — Open questions in `network-protocol.md`. Block integration-level assertion stories only; unit tests on pure formulas (Gelure, Xelorium, Rollback, etc.) can proceed independently. Resolve before any story requiring Lightyear message delivery is opened.
4. **`MessageReceiver<C2SClassChoice>` single-drain rule** — Only ONE system may drain this receiver (the LOBBY handler). Register as forbidden pattern per ADR-014 §5 (same pattern as ADR-013 for `MessageReceiver<C2SAuctionBid>`).

## GDD Requirements

| TR-ID | Requirement | ADR Coverage |
|-------|-------------|--------------|
| TR-CS-001 | Class lifecycle: LOBBY selection via `C2SClassChoice`; `PlayerSessions` Resource; `all_classes_chosen()` gate; `lock_all_classes()` at LOBBY→DRAFT_INITIAL; `class_locked` prevents re-selection; `PlayerSnapshot.class_id` in S2CGameSnapshot (resolves NP-1) | ADR-014 ✅ |
| TR-CS-002 | Xelor reserve formulas — Gelure (CS-1): `reserve += current_mana; current_mana = 0`; Xelorium (CS-2): `self.reserve += opponent.current_mana; opponent.current_mana = 0` at sub-step 1 post-cost-deduction; Rollback (CS-3): `n = self.reserve; reserve = 0; friendly Minions charge n cells, STUN-blocked excluded` | ADR-014 ✅ |
| TR-CS-003 | Garde-Temps reserve gate (CS-4): `reserve >= garde_temps_cost` AND `garde_temps_used_this_game < garde_temps_per_game_cap` AND target alive; on accept: `reserve -= cost`, counter +1, lethal `take_damage` on chosen enemy objective; on reject: no state change | ADR-014 ✅ |
| TR-CS-004 | Miss Nuit per-round cap (CS-11): +1 reserve per opponent Spell/Minion card played from hand while Miss Nuit alive and unsilenced; capped at `miss_nuit_cap` (default 2) per round; token spawns/prism-draws/own-card-plays excluded | ADR-014 ✅ |
| TR-CS-005 | Sacrier effects — Sang Méprise (CS-5): reliable unicast `S2CSangMepriseReveal` to both players containing all alive objective `is_fake` status; cleared at RESOLUTION end; Punition (CS-6): sacrifice chosen alive real objective (lethal `take_damage`) + 3 damage each alive opponent objective; self-elimination if real_destroyed ≥ 3 → GAME_OVER | ADR-014 ✅ |
| TR-CS-006 | Sadida Seed cell-hazard (CS-7): friendly walk-over → unit.AR +1 permanent; enemy walk-over → 1 damage pre-AR; seed persists; max 1 seed/cell (new placement discarded if occupied); Graines de Folie (CS-8): each board Seed removed → Madoll (HP=3/ATK=1/MP=3) spawned at seed cell; over-capacity spawn silently skipped, seed still consumed | ADR-014 ✅ |
| TR-CS-007 | Ecaflip RNG effects — Dé du Chateux (CS-9): RESOLUTION RNG roll ∈ [1,6], deal roll damage to target, reveal enemy objective in lane iff roll ≤ `dé_chateux_reveal_threshold` (Ecaflip-only unicast); Coin flip (CS-10): Chatar/Shava Shavien/Craps binary outcomes from RESOLUTION RNG; Craps: `share = floor(total/alive)` + ascending-lane remainder; `alive == 0` guard | ADR-014 ✅ |
| TR-CS-008 | Class card shop filtering: class slot samples `draw_class_card(player_class, ...)` exclusively (no cross-class cards in class slot); cross-class draw legality — `draw_random` uniform draw bypasses class filter; no runtime play-time gate on `card_class` field (cross-class cards in hand are always playable) | ADR-014 ✅ |
| TR-CS-009 | Token spawn: `SourceClass(ClassId)` component on all 7 token types at spawn time, never mutated; `TokenUnit` marker component on all tokens; `UnitBoardState.source_class: Option<ClassId>` derived from component at snapshot build (resolves NP-2); Sinistro 1 dmg/RESOLUTION to opposing-lane objective; La Gonflable END-OF-MOVEMENT +2 HP heal to other friendly lane units; La Sacrifiée DEATH → 1 dmg each enemy unit in lane | ADR-014 ✅ |

## Epic Dependencies

This epic requires the following to be **DONE** before implementation begins:

| Dependency | Why |
|------------|-----|
| `game-session-system` story (class selection + `PlayerSessions` scaffold) | Provides `PlayerSessions` Resource inserted before `SessionReady`; `garde_temps_used_this_game` counter initialized at LOBBY→DRAFT_INITIAL |
| `round-state-machine` story-001 (State & Events Scaffold) | Provides `LobbyComplete` Message on ADR-010 event bus; LOBBY phase gate for `C2SClassChoice` |
| `workspace-and-shared-types` story (Shared Card Types) | Provides `ClassId` enum in `shared/src/card.rs`; `C2SClassChoice` in `shared/src/protocol.rs` |
| `keyword-system` epic (PIERCE definition) | Required before TR-CS-006 Seed stories — PIERCE is the design counter to high-AR Sadida units (CS-7 pre-impl gate) |

## Scope

### Deliverables

**`server/src/core/session/state.rs`** (extends existing `PlayerSessions`)
- `PlayerSessionData` fields: `class: ClassId`, `class_locked: bool`
- `impl PlayerSessions`: `class_of()`, `is_locked()`, `all_classes_chosen()`, `lock_all_classes()`

**`server/src/lobby/handler.rs`**
- `handle_class_choice`: sole drainer of `MessageReceiver<C2SClassChoice>`; updates `player.class` if `!class_locked && class != ClassId::Neutral`

**`server/src/core/board/components.rs`**
- `SourceClass(ClassId)` component — derives `Component, Clone, Copy, Debug, PartialEq, Eq`; NO `Reflect` (server-only, headless build)
- `TokenUnit` marker component — derives `Component, Default`

**`server/src/core/board/spawn.rs`**
- One `spawn_*` function per token type (7 total): `spawn_mummy`, `spawn_chacha_noir`, `spawn_seed`, `spawn_madoll`, `spawn_la_gonflable`, `spawn_la_sacrifiee`, `spawn_sinistro`
- Each hard-codes its `SourceClass(ClassId::*)` variant and adds `TokenUnit`

**`server/src/core/resolution/effects.rs`**
- Pure Rust helper functions for all 11 formulas: `apply_gelure`, `apply_xelorium`, `apply_rollback`, `apply_garde_temps`, `apply_miss_nuit_trigger`, `apply_sang_meprise`, `apply_punition`, `apply_seed_walkover`, `apply_graines_de_folie`, `apply_de_du_chateux`, `apply_coin_flip`
- Called from within `resolve_resolution` system body — NOT standalone Bevy systems

**Protocol additions** (in `shared/src/protocol.rs` and snapshot builder)
- `C2SClassChoice { class: ClassId }` — derives `lightyear::prelude::Message`
- `PlayerSnapshot.class_id: ClassId` (resolves NP-1)
- `UnitBoardState.source_class: Option<ClassId>` (resolves NP-2)

### Out of Scope

- Economy `current_mana`/`reserve` mutation implementation — owned by `economy-system` epic; class formulas call its API
- Objective `take_damage` implementation — owned by `objective-system` epic
- Combat Resolution sub-step execution framework — owned by `combat-resolution` epic
- Server-side RNG RESOLUTION chain — owned by `server-rng` epic; Ecaflip formulas consume from it
- Class-related UI (class picker, Garde-Temps gate feedback, Rollback n=0 warning, Xelorium drain animation) — owned by Presentation layer epics
- Sang Méprise reconnect snapshot field (`active_sang_meprise_reveals`) — owned by `game-session-system` epic (snapshot builder)
- Miranda control transfer protocol messages (NP-7) — owned by Network Protocol epic
- Chacha Noir `SpawnSource::Replacement` variant (NP-8) — owned by Network Protocol epic

## Definition of Done

This epic is complete when:
- All stories are implemented, reviewed, and closed via `/story-done`
- All 27 BLOCKING acceptance criteria from `design/gdd/class-system.md` (CS-AC-01 through CS-AC-30, excluding CS-AC-03b and CS-AC-14b which are ADVISORY UI) verified by passing tests
- All Logic stories have passing test files in `tests/unit/class/`
- All Integration stories have passing test files in `tests/integration/class/`
- `cargo check --workspace` green; zero warnings on `server/src/feature/class/**`
- Code review gate: `MessageReceiver<C2SClassChoice>` appears in exactly one system (`handle_class_choice`)
- Code review gate: `ResMut<PlayerSessions>` in RESOLUTION system has explicit `.before()`/`.after()` ordering
- CI grep gate: `grep -rE "EventWriter|EventReader|Events<|add_event" server/src/feature/class/` returns zero matches
- PIERCE keyword confirmed defined in `keyword-system.md` before Seed stories close

## Stories

| # | Story | Type | Status | ADR |
|---|-------|------|--------|-----|
| 001 | [Class Lifecycle — PlayerSessions Scaffold](story-001-class-lifecycle.md) | Logic | Ready | ADR-014 |
| 002 | [Token Spawn Scaffold — SourceClass Component](story-002-token-spawn-scaffold.md) | Logic | Ready | ADR-014 |
| 003 | [Xelor Reserve Formulas — Gelure, Xelorium, Rollback](story-003-xelor-reserve-formulas.md) | Logic | Ready | ADR-014 |
| 004 | [Garde-Temps Reserve Gate](story-004-garde-temps-gate.md) | Logic | Ready | ADR-014 |
| 005 | [Miss Nuit Per-Round Cap](story-005-miss-nuit-cap.md) | Logic | Ready | ADR-014 |
| 006 | [Sacrier Effects — Sang Méprise and Punition](story-006-sacrier-effects.md) | Integration | Ready | ADR-014 |
| 007 | [Sadida Seeds and Graines de Folie](story-007-sadida-seeds.md) | Integration | Ready | ADR-014 |
| 008 | [Ecaflip RNG Effects — Dé du Chateux and Coin Flip](story-008-ecaflip-rng.md) | Logic | Ready | ADR-014 |
| 009 | [Class Card Shop Filtering](story-009-shop-filtering.md) | Logic | Ready | ADR-014 |
| 010 | [Token Passive Behaviors — Sinistro, La Gonflable, La Sacrifiée](story-010-token-passives.md) | Integration | Ready | ADR-014 |
| 011 | [C2SClassChoice Protocol Path Drop](story-011-classchoice-drop.md) | Decision-first + Config/Data + docs sync | Draft — Sprint 18 candidate (S18-PROTO-CLASSCHOICE-DROP-001 per PROMPT 1298 / authored by PROMPT 1305, integrated by PROMPT 1313), NOT activated. Supersedes PROMPT 1202 placeholder `S14-PROTO-CLASSCHOICE-DISPOSITION-001` and closes the `lightyear-protocol-verification/story-008` allowlist row for `C2SClassChoice` | ADR-014 |
