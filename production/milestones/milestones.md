# Milestones — Lanes and Lies

**Locked:** 2026-04-29

## Full scope

All 20 systems ship. All game modes (1v1, 2v2, 3v3, 1v1v1, 2v2v2) are in scope.
The milestone structure below defines implementation order only — nothing is cut.

## M1 — Core Loop
*Gate: Two players connect, play one full round, round resolves correctly.*

Systems: Card Data & Pool, Game Config, Server-side RNG, Economy System,
Board/Lane System, Round State Machine, Network Protocol, Game Session System,
Objective System.

Deployment target: WASM client → Vercel; headless Rust server → Railway (Docker).

## M2 — Playable Game
*Gate: Complete 1v1 game — auction, card acquisition, combat, shop, win condition, visual board.*

Systems: Card Acquisition, Auction System, Combat Resolution, Board Rendering,
Hand UI, Shop/Auction UI, HUD.

Note: 2v2 and 3v3 implementation begins once OneVOne is stable. Resolve C-W2
(spawn-range counter per-player vs per-team) before TwoVTwo implementation.

## M3 — Full Feature
*Gate: All GDD mechanics working — keywords, prisms, class rules, animations.*

Systems: Keyword System, Prism System, Class System, Card Animations.
