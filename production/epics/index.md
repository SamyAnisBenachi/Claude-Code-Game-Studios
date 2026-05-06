# Epics Index

**Last Updated:** 2026-05-06
**Engine:** Bevy 0.18 + Lightyear 0.26
**Layer in progress:** Feature (M1 + M2 + M3) + Presentation + Polish

| Epic | Layer | Architecture Module | GDD(s) | Stories | Status |
|------|-------|---------------------|--------|---------|--------|
| [workspace-and-shared-types](workspace-and-shared-types/EPIC.md) | Foundation | `shared/` crate | game-config, card-data-pool, network-protocol | 4 stories | Ready |
| [game-config-pipeline](game-config-pipeline/EPIC.md) | Foundation | `server/foundation/config.rs` | game-config, card-data-pool | 4 stories | Ready |
| [server-rng](server-rng/EPIC.md) | Foundation | `server/foundation/rng.rs` | server-rng | 3 stories | Ready |
| [lightyear-protocol-verification](lightyear-protocol-verification/EPIC.md) ⭐ | Foundation | `shared/protocol.rs` + `server/network/` + `client/network/` | network-protocol | 4 stories | Ready |
| [round-state-machine](round-state-machine/EPIC.md) | Core | `server/core/rsm/` | round-state-machine | TBD — see hint | Ready |
| [game-session-system](game-session-system/EPIC.md) | Core | `server/core/session/` (+ `on_session_ready` in `server/core/rsm/`) | game-session-system | TBD — see hint | Ready |
| [economy-system](economy-system/EPIC.md) | Core | `server/core/economy/` | economy-system | TBD — see hint | Ready |
| [card-data-pool](card-data-pool/EPIC.md) | Core | `server/core/pool/` | card-data-pool | 6 stories | Complete |
| [board-lane-system](board-lane-system/EPIC.md) | Feature (M1) | `server/feature/board/` | board-lane-system | TBD | Ready |
| [objective-system](objective-system/EPIC.md) | Feature (M1) | `server/feature/objective/` | objective-system | TBD | Ready |
| [auction-system](auction-system/EPIC.md) | Feature (M2) | `server/feature/auction/` | auction-system | Not yet created | Ready |
| [card-acquisition](card-acquisition/EPIC.md) | Feature (M2) | `server/feature/acquisition/` | card-acquisition | Not yet created | Ready |
| [combat-resolution](combat-resolution/EPIC.md) | Feature (M2) | `server/feature/combat/` | combat-resolution | Not yet created | Ready |
| [class-system](class-system/EPIC.md) | Feature (M3) | `server/feature/class/` | class-system | 10 stories | Ready |
| [prism-system](prism-system/EPIC.md) | Feature (M3) | `server/feature/prism/` | prism-system | Not yet created | Ready |
| [keyword-system](keyword-system/EPIC.md) | Feature (M3) | `server/feature/keyword/` + `protocol/src/keyword.rs` | keyword-system | Not yet created | Ready — 7 pre-impl gates to clear first |
| [presentation-layer](presentation-layer/EPIC.md) | Presentation | `client/src/presentation/` | ADR-021 cross-epic infrastructure | 1 story | Ready-for-Readiness |
| [hud](hud/EPIC.md) | Presentation | `client/src/ui/hud/` | hud | 10 stories | Ready |
| [card-animations](card-animations/EPIC.md) | Presentation | `client/src/card_animations/` | card-animations | 9 stories | Ready |
| [hand-ui](hand-ui/EPIC.md) | Presentation | `client/src/ui/hand/` | hand-ui | 13 stories | Ready - OQ8 gates activation-lock story |
| [board-rendering](board-rendering/EPIC.md) | Presentation | `client/src/ui/board/` | board-rendering | 10 stories | Ready - protocol/objective/combat gates on later stories |
| [shop-auction-ui](shop-auction-ui/EPIC.md) | Presentation | `client/src/ui/shop_auction/` | shop-auction-ui | 9 stories | Ready - UX spec gates final visual evidence |
| [playable-client](playable-client/EPIC.md) | Polish | `client/src/main.rs` + `client/src/network/` + client UI/presentation + server message bridges | Sprint 7 real primary client path | 3 stories | Ready |

> ⭐ Sprint 1 Story 1.0 = `lightyear-protocol-verification` Story 001 (Lightyear 0.26 verification spike)

## Story Count Summary

| Epic | Stories | Logic | Integration | Config/Data |
|------|---------|-------|-------------|-------------|
| workspace-and-shared-types | 4 | 0 | 2 | 2 |
| game-config-pipeline | 4 | 1 | 2 | 1 |
| server-rng | 3 | 3 | 0 | 0 |
| lightyear-protocol-verification | 4 | 0 | 3 | 1 |
| round-state-machine | TBD | — | — | — |
| game-session-system | TBD | — | — | — |
| economy-system | TBD | — | — | — |
| card-data-pool | 6 | 3 | 3 | 0 |
| playable-client | 3 | 0 | 3 | 0 |
| **Foundation total** | **15** | **4** | **7** | **4** |

> Core layer story counts will populate after `/create-stories` is run on each Core epic.

## Layer Roadmap

| Layer | Status | Gate |
|-------|--------|------|
| Foundation | **Stories complete — ready for sprint planning** | Pre-Production → Production gate requires Foundation + Core epics |
| Core | **EPIC files written 2026-04-29 — run `/create-stories` per epic next** | Story authoring next; sprint planning after |
| Feature (M1) | **EPIC files written 2026-04-30 — run `/create-stories` per epic next** | Board/Lane + Objective epics ready; M2/M3 epics paused pending ADRs |
| Feature (M2) | **auction-system + card-acquisition + combat-resolution EPICs written 2026-04-30** — run `/create-stories` per epic next; Presentation epics pending ADRs | ADR-013 ✅ ADR-015 ✅ ADR-017 ✅ |
| Feature (M3) | **class-system + prism-system + keyword-system EPICs written 2026-04-30** — run `/create-stories class-system` next; prism-system + keyword-system have pre-impl gates to clear first (keyword: ADR-018/022 must be Accepted; prism: 3 gates) | ADR-014 ✅ (class-system); ADR-016 ✅ (prism-system); ADR-018/022 ⚠️ Proposed (keyword) |
| Presentation | **card-animations + hand-ui + hud EPICs written 2026-05-01; board-rendering + shop-auction-ui epics/stories written 2026-05-04 for S5-21; presentation-layer shared ADR-021 foundation story proposed 2026-05-04**. Next step: run `/story-readiness` for `presentation-layer` Story 001 before Board Rendering 001 or Shop/Auction UI 001. | ADR-021 ✅ (Presentation Layer Architecture) |
| Polish | **playable-client epic and Sprint 7 PLAYABLE story docs written 2026-05-06**. Next implementation step is PLAYABLE-001, then PLAYABLE-002, then PLAYABLE-003 evidence. | Friend-game playable path, not public release readiness |

## Core Layer Coordination Notes

The four Core epics have coordinated dependencies that should drive sprint sequencing:

- **Round State Machine (Epic 1)** defines `RoundState`, `advance_phase`, and the full ADR-010 event catalog (`DraftStarted`, `ShopRefreshNeeded`, `PlacementPhaseEntered`, `ResolutionPhaseEntered`, `GameOverEmitted`, `BroadcastPhaseChanged`). Every other Core epic subscribes to one or more of these events. **Implement Epic 1's Story 1 (State + Events scaffold) first** — once events.rs exists, the other epics' subscriber stories can begin in parallel.
- **Game Session System (Epic 2)** owns `SessionReady` (Observer trigger per ADR-012) and contributes `on_session_ready` to `server/core/rsm/system.rs`. Epic 2's Story 4 (F4 + SessionReady trigger) is the highest single-story risk in the Core layer — it is gated on completing the four ADR-012 verification checks against Bevy 0.18 source.
- **Economy System (Epic 3)** subscribes to `DraftStarted` and `ResolutionPhaseEntered`. Epic 3's Story 1 (State + API) is pure-function and can begin in parallel with Epic 1 Story 1.
- **Card Data & Pool (Epic 4)** subscribes to `ShopRefreshNeeded` and the `SessionReady` Observer. Epic 4's Stories 1–3 (data model, draw, refresh_shop) are pure-function and can begin in parallel with Epic 1 Story 1.

Recommended sprint sequence (subject to producer feasibility review):
- Sprint A: Epic 1 Story 1 (RSM scaffold) **+** Epic 3 Story 1 (Economy API) **+** Epic 4 Stories 1–3 (Pool API, draw, refresh) in parallel.
- Sprint B: Epic 1 Stories 2–4 (advance_phase, timers, win condition) **+** Epic 2 Stories 1–3 (lobby + class reveal) **+** Epic 3 Story 2 (DraftStarted subscriber) **+** Epic 4 Story 4 (ShopRefreshNeeded subscriber).
- Sprint C: Epic 2 Story 4 (SessionReady — high-risk, gated on ADR-012 verification) **+** Epic 1 Stories 5–6 (disconnect, network dispatch) **+** Epic 3 Stories 3–6 (snapshot, awards, reservation, dispatch) **+** Epic 4 Stories 5–6 (manual refresh, dispatch) **+** Epic 2 Stories 5–7 (disconnect, teardown, reconnect).
