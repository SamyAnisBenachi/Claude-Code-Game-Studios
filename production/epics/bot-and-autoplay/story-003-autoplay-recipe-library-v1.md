# Story 003: AUTOPLAY-RECIPE-LIBRARY-001 -- Autoplay Full-Game Recipe Library v1 (Real UI Input)

> **Epic**: Bot & Autoplay
> **Story ID**: AUTOPLAY-RECIPE-LIBRARY-001
> **Status**: Draft -- Sprint 19 candidate; bootstrap landed via PROMPT 1601; recipe library extension is the next slice
> **Layer**: Integration -- client-side real UI input automation
> **Type**: Integration -- recipe library under `client/src/autoplay.rs` / `tools/autoplay/**`
> **Sprint**: Sprint 19 candidate (NOT activated)
> **Authored**: 2026-05-21 by PROMPT 1608
> **Authoring source-of-truth**: `origin/main@576fbe8ce901a8b919a4c2db58847f2d497d3d15`

---

## Status / No-Claim Banner

This story is authored as a **Sprint 19 candidate**. The autoplay
bootstrap (substrate + first slice) is landed on `origin/main` via
PROMPT 1595 and PROMPT 1601, with the docs refresh via PROMPT 1606.
The remaining work is to build a **recipe library v1** that exercises
the full friend-game loop through real UI input.

PROMPT 1608 does NOT:

- Activate Sprint 19 or any row in this story.
- Modify any code under `client/`, `server/`, `shared/`, `tests/`,
  `tools/`, `docs/`, or `.claude/skills/`.
- Run `/dev-story`, `/story-readiness`, `/story-done`, `/smoke-check`,
  `/team-qa`, `/gate-check`, or `/qa-plan`.

**Critical constraint preserved verbatim**: Autoplay is **real UI input
automation, not semantic game-state mutation**. Recipes MUST drive the
client through the same pointer / keyboard / widget interactions a
human operator would use. Recipes MUST NOT short-circuit the
client-server message flow by directly mutating client or server state.

Non-claims preserved verbatim: NO public release readiness, NO RC
readiness, NO full game completion, NO Standard-tier accessibility
advancement, NO playtest validation, NO `S8-QA-001-W1` closure, NO
`PAW-TD-*-a` completion, NO stage advance.

---

## Problem Class / Prevention Target

**Defect class**: Autoplay bootstrap (PROMPT 1601) provides the
substrate (driver, RPC schema, skill scaffold) but covers only the
opening slice. QA cannot currently script a full friend-game pass
without a human at the controls.

**Prevention target**: A recipe library that covers the full
friend-game loop via real UI input:

- Lobby create / join / class confirm.
- DRAFT_INITIAL handling.
- Shop / Auction interaction (bid, accept).
- Placement drag/drop and accepted-ACK.
- Rejection recovery flow.
- Resolution viewing.
- Result screen acknowledgement and next-loop entry.

Each recipe is composable so a higher-level recipe (e.g. "full game to
GAME_OVER") chains the smaller ones.

---

## Acceptance Criteria (ledger; not gated by PROMPT 1608)

- AC1: Recipe library exposes named recipes covering the full game flow.
  Inventory reconciled by PROMPT 1667 (2026-05-27) against
  `tools/autoplay/recipes/__init__.py` as of PROMPT 1655:

  | Registry name          | Conceptual AC1 origin                               | Landed   |
  |------------------------|-----------------------------------------------------|----------|
  | `smoke`                | utility / QA substrate (not a game-flow recipe)     | PROMPT 1609 |
  | `idle`                 | utility / status-soak (not a game-flow recipe)      | PROMPT 1609 |
  | `add-bot-lobby`        | `lobby_join` (superseded — see note below)          | PROMPT 1634 |
  | `lobby-create`         | `lobby_create` (renamed)                            | PROMPT 1636 |
  | `class-select`         | `class_confirm` (renamed)                           | PROMPT 1636 |
  | `draft-auction-probe`  | `draft_initial` + `shop_buy` + `auction_bid` + `auction_accept` (merged — see note below) | PROMPT 1639 |
  | `placement-drag-probe` | `placement_drag_drop` (renamed)                     | PROMPT 1639 |
  | `resolution-observe`   | `resolution_view` (renamed)                         | PROMPT 1636 |
  | `game-over-observe`    | `result_ack` (renamed)                              | PROMPT 1636 |
  | `round-loop`           | `next_loop_entry` (renamed / expanded to multi-round composite) | PROMPT 1655 |
  | `full-game`            | full-loop composite                                 | PROMPT 1655 |

  **Mapping notes:**
  - `lobby_join` is **superseded by `add-bot-lobby`** in bot-game mode.
    The original AC1 assumed a second human client joining; the
    implementation uses a bot opponent instead. `add-bot-lobby` is the
    functional equivalent for automated play.
  - `draft_initial`, `shop_buy`, `auction_bid`, and `auction_accept` are
    **merged into `draft-auction-probe`**. These steps are always
    sequential; no standalone use-case exists for any sub-step in v1.
  - `placement_reject_recovery` is **descoped from v1**: no standalone
    recipe exists in the registry. The rejection-recovery UX (unit bounces
    back to hand after a server rejection) is partially exercised inside
    `full-game` but is not addressable by name. This is **not an AC1
    blocker for v1**; it is a candidate for `placement-reject-probe` in a
    future v1.1 story or separate story ticket.
- AC2: A `full_friend_game` recipe chains the above into a complete
  loop and reaches at least one full RESOLUTION cycle.
- AC3: Recipes drive the real Bevy UI through pointer / keyboard /
  widget events; NO direct client-state mutation, NO direct C2S
  message emission outside what the UI itself emits.
- AC4: Recipes are deterministic given a fixed RNG seed and a fixed
  bot opponent / second-autoplay seed.
- AC5: A recipe can be invoked from `tools/autoplay/**` (CLI) and
  produces a structured pass/fail report.
- AC6: Failures surface the exact step that failed and capture the
  current QA snapshot for triage.
- AC7: Documentation under `docs/autoplay/` lists every recipe, its
  inputs, and its acceptance contract.

---

## Implementation Lineage on `origin/main`

| PROMPT | Status | Slice |
|---|---|---|
| 1595 | Shipped | Bevy autoplay bootstrap first slice (substrate) |
| 1600 | Superseded by 1601 | Integration refresh (FF-ready pre-state-commit) |
| 1601 | Main-landed | `client/src/autoplay.rs`, `tools/autoplay/**`, `docs/autoplay/**`, `skills/ccgs-autoplay` |
| 1605 | Shipped | Focused verify (docs field-name concerns) |
| 1606 | Shipped | Autoplay RPC/schema docs alignment |
| 1609 | Main-landed | `smoke` + `idle` utility recipes; autoplay live-GUI smoke template |
| 1634 | Main-landed | `add-bot-lobby` recipe; bot-game lobby setup |
| 1636 | Main-landed | `lobby-create`, `class-select`, `resolution-observe`, `game-over-observe` recipes |
| 1639 | Main-landed | `draft-auction-probe`, `placement-drag-probe` recipes |
| 1655 | Main-landed | `round-loop`, `full-game` recipes; README recipe table refresh |

Recipe library v1 is **substantially landed** on `origin/main` as of
PROMPT 1655. AC1 reconciliation completed by PROMPT 1667.

---

## Recommended Sprint 19 Follow-Up Prompts

1. `/story-readiness AUTOPLAY-RECIPE-LIBRARY-001` against the Sprint 19
   activation tip.
2. `/dev-story AUTOPLAY-RECIPE-LIBRARY-001` to implement the recipe
   library v1.
3. `/story-done` once acceptance criteria pass.
4. (Optional) A follow-up story that adds reconnect-path recipes; out
   of scope here.

---

## Test Evidence (target)

- Logic: recipe step unit tests under `tests/unit/autoplay/`.
- Integration: end-to-end recipe runs under
  `tests/integration/autoplay/` (single-client autoplay against
  scripted-bot opponent, deterministic seed).
- Manual: orchestrator-driven `full_friend_game` recipe invocation
  with captured artifacts under `production/qa/evidence/`.

---

## Dependencies

- BOT-ROOM-PARTICIPANT-001 (story 001) -- the bot opponent that recipes
  play against.
- BOT-SOAK-ENTRYPOINT-001 (story 002) -- bounded-round flag is reused
  for autoplay's "run-to-round-N" recipe variant.
