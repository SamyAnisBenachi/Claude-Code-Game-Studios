# Smoke Test: Critical Paths

**Purpose**: Run these checks before every QA hand-off and before merging to main.
**Run via**: `/smoke-check` (reads this file)
**Target**: < 15 minutes total
**Update**: Add new entries when each new core system is implemented.

---

## Core Stability (run always)

1. **Server starts without panic** — `cargo run -p server` exits cleanly or runs to ready state
2. **Workspace compiles** — `cargo build --workspace` exits 0
3. **All tests pass** — `cargo test --workspace` exits 0

## M1 — Core Loop (add as systems are implemented)

4. **GameConfig loads** — server starts with valid `game_config.ron`, no fatal errors
5. **CardCatalog loads** — `assets/data/cards.json` parses, no duplicate CardIds
6. **Two clients can connect** — WebSocket handshake completes, `S2CHandshake` received
7. **Lobby completes** — both players confirm class, `S2CClassesRevealed` fires
8. **DRAFT_INITIAL begins** — `S2CPhaseChanged(DraftInitial)` received by both clients
9. **PLACEMENT opens** — `S2CPhaseChanged(Placement)` received, timer starts
10. **PLACEMENT closes** — `S2CPlacementReveal` received by both clients simultaneously
11. **RESOLUTION runs** — `S2CResolutionEvent` stream fires, phase transitions to DRAFT_SHOP
12. **GAME_OVER fires** — destroying 2 real objectives sends `S2CGameOver` with correct loser

## M2 — Playable Game (add when M2 systems implemented)

13. **Auction runs** — `S2CAuctionUpdate` broadcasts, timer resets on accepted bid
14. **Shop refresh works** — `S2CShopSlots` received after DRAFT entry
15. **Combat resolves** — unit advancement + collision + objective damage

## Performance Checks (add when vertical slice exists)

16. **Server tick ≤ 5ms** — steady-state profiling via `/perf-profile`
17. **WASM bundle ≤ 50 MB** — `trunk build --release` output
18. **60 FPS client** — no frame drops on target browser

---

*Entries marked with a system name should be added when that system's
first story is marked Done. The smoke test grows with the game.*
