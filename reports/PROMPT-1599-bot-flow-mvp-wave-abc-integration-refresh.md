# PROMPT-1599 — BOT-FLOW-MVP-WAVE-ABC-INTEGRATION-REFRESH

**Status:** SHIPPED
**Branch:** `integrate/bot-flow-wave-abc-1599`
**Tip commit:** `219d13772cc8612ddac704b410c908bd72c627b5`
**Base:** `origin/main@3a4603af3227827b2fdf1e060354a14bf4389208`
**FF-ready vs origin/main:** YES (origin/main is an ancestor; 3 commits ahead, 0 behind)
**Worktree:** `D:\Tmp\wt-1599`
**Pushed:** YES (`origin/integrate/bot-flow-wave-abc-1599`)

## Payload combined

Three single-commit worker branches, cherry-picked in landing order (1596 → 1597 → 1598):

| Order | PROMPT | Source tip | Cherry-pick result | Owned scope |
|---|---|---|---|---|
| 1 | 1596 — BOT-FLOW-LOBBY-ADD-REMOVE-BOT-UX | `origin/work/bot-flow-lobby-add-remove-bot-ux-1596@4dc7841c` | `009f3fbd` | client lobby UI Add/Remove Bot |
| 2 | 1597 — BOT-FLOW-SERVER-QA-SNAPSHOT | `origin/work/bot-flow-server-qa-snapshot-1597@7632ae60` | `d4f58392` | server QA snapshot + decision-log JSONL |
| 3 | 1598 — BOT-FLOW-AUCTION-BID-FUNNEL-WAVE-2-5 | `origin/work/bot-flow-auction-bid-funnel-wave-2-5-1598@c7d53062` | `219d1377` | bot auction bid funnel into `process_bid_batch` |

All three cherry-picks landed cleanly. No conflict markers, no manual edits. Each
source branch's payload (code + tests + report) is preserved 1:1 — `git diff
--check origin/main..HEAD` produced no output (no whitespace/conflict artefacts).

## Combined diff stat

```
 client/Cargo.toml                                  |    8 +
 client/src/ui/lobby.rs                             |  293 +++-
 reports/PROMPT-1597-bot-flow-server-qa-snapshot-and-decision-log.md |  256 ++++
 reports/PROMPT-1598-bot-flow-auction-bid-funnel-wave-2-5.md         |  118 ++
 server/Cargo.toml                                  |    5 +
 server/src/feature/auction/mod.rs                  |    2 +-
 server/src/feature/auction/plugin.rs               |    6 +-
 server/src/feature/auction/system.rs               |   80 +-
 server/src/feature/bot/action_loop.rs              |   61 +-
 server/src/feature/bot/mod.rs                      |    7 +
 server/src/feature/bot/qa_snapshot.rs              | 1395 ++++++++++++++++++++
 server/src/main.rs                                 |    7 +
 tests/integration/playable_client/lobby_bot_controls_test.rs |  364 +++++
 tests/unit/bot/bot_auction_bid_funnel_wave_2_5_test.rs       |  276 ++++
 14 files changed, 2854 insertions(+), 24 deletions(-)
```

## Conflict analysis (pre-merge)

A pre-merge file-level overlap check confirmed the three branches operate on
disjoint surfaces with one trivial co-edited file:

- **1596** — client-only: `client/Cargo.toml`, `client/src/ui/lobby.rs`, new
  client test. **Zero overlap** with 1597/1598.
- **1597** — server: new `server/src/feature/bot/qa_snapshot.rs`, edits to
  `server/src/feature/bot/mod.rs` (adds `pub mod qa_snapshot;` + re-exports),
  edits to `server/src/main.rs` (registers `BotQaSnapshotPlugin`). Does **not**
  touch `server/Cargo.toml` or any auction/action_loop file.
- **1598** — server: edits `server/Cargo.toml` (new `[[test]]` entry only),
  `server/src/feature/auction/{mod,plugin,system}.rs`,
  `server/src/feature/bot/action_loop.rs`, and a new bot unit test. Does **not**
  touch `server/src/feature/bot/mod.rs`, `qa_snapshot.rs`, or `server/src/main.rs`.

Therefore 1596/1597/1598 are mutually orthogonal — there was nothing for a
conservative conflict-resolution policy to resolve. No payload had to be split
out.

## Path allowlist review

All landed paths fall inside the prompt's owned scope (client lobby UI from
1596, server bot/auction/snapshot files and tests from 1597/1598, plus the
1597/1598 reports carried forward — none were already on `main`). The
following forbidden paths were **not** touched on the integration branch:

- `client/src/ui/hand/mod.rs` — untouched
- `client/src/presentation/board_rendering.rs` — untouched
- `client/src/ui/shop_auction/mod.rs` — untouched
- `client/src/presentation/qa_snapshot.rs` — untouched
- `production/sprint-status.yaml` — untouched
- `production/session-state/**` — untouched
- `production/stage.txt` — untouched
- Sprint activation/close-out files — untouched
- Unrelated Cargo/CI files — only the in-scope `client/Cargo.toml` (1596 test
  registration) and `server/Cargo.toml` (1598 test registration) changed.

## Validation

- `git diff --check origin/main..HEAD` — clean (no output).
- Path allowlist review — PASS (above).
- Cherry-pick conflict counter — 0/3 (clean apply for every payload).
- FF-readiness vs `origin/main` — confirmed via `git merge-base --is-ancestor
  origin/main HEAD` returning 0 (ancestor).
- Focused test run — **deferred to VERIFY lane** per implementation rules
  ("Do not run broad Cargo. Run focused tests only if cheap…"). Bevy + lightyear
  compile cost is high; each worker branch already reports its own test pass in
  its report, and the integration branch introduces no new code beyond their
  exact cherry-picks.

## What this enables for downstream lanes

1. Lobby Add/Remove Bot CTAs (1596) — pure client UX; no server contract change.
2. Server-authoritative QA evidence for bot-driven flows (1597) — JSON snapshots
   + JSONL decision-log streamer gated by `CCGS_BOT_QA_SNAPSHOT=1`.
3. Bot auction bid funnel (1598) — bots now produce real `AuctionBid` items
   drained by `auction_tick_system` alongside network bids, validated by
   `process_bid_batch` under the same price-floor / leader / gold / hand-full /
   expiry rules as humans.

The combined chain unblocks bot-flow MVP smoke runs end-to-end (lobby Add Bot →
bot lobby-ready → bot draft-ready → bot auction bid → placement no-op →
resolution) with QA evidence captured at every phase transition.

## Notes for the orchestrator

- Branch is FF-ready against current `origin/main@3a4603af`. If another lane
  advances `main` between this report and a fast-forward attempt, a rebase
  refresh (PROMPT 1600-series) will be needed; the three payloads remain
  conflict-clean against any unrelated `main` advance because they touch a
  narrow file set.
- No `main` push attempted (per prompt rules — "Do not push main").
- Worktree retained at `D:\Tmp\wt-1599` for any follow-up verification.

---

1599: BOT-FLOW-MVP-WAVE-ABC-INTEGRATION-REFRESH: SHIPPED
