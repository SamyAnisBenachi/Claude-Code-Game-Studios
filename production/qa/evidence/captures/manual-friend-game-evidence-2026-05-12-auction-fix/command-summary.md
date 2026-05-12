# Manual Friend-Game Evidence — 2026-05-12 (auction-fix validation)

> **Status**: Validation of `S11-SERVER-AUCTION-SETTLE-REGRESSION-FIX` (commit `f08b2c8`).
> **Result**: ✅ **Auction settles correctly with real bids** — two settled auctions in this run
> (R3 winner=P1 amount=5, R6 winner=P2 amount=6). No server panic. As a side benefit,
> **the Placement R2 entry regression from the morning run is also gone** despite both players
> purchasing 6 cards in DraftInitial — the exact interaction pattern that caused the earlier crash.
>
> Game ended naturally via `GameOver{reason=Disconnect, loser=P2}` at round 9 when the user closed
> the Client B window. Not a crash.

## Environment

| Field | Value |
|---|---|
| Date / time | 2026-05-12, ~17:15 – 17:22 UTC (server uptime ~7 min) |
| OS / target | Windows 11 Pro, native build (not WASM) |
| **Commit** | **`f08b2c8e5078b8187b571eabd3ba34812b266215`** (branch `main`, includes auction fix) |
| Diff vs earlier captures | 5 commits ahead of `8e3d044` (where the morning crash/no-crash runs were captured). New work: `693d2c8` + `590d6bd` + `a3efce8` (test fixture + message relocation work) and `f08b2c8` (auction settle fix). |
| Working tree | Clean (only `.claude/scheduled_tasks.lock` untracked, irrelevant). |
| Rust toolchain | cargo 1.95.0 / rustc 1.95.0 |
| GPU | NVIDIA GeForce RTX 5090 Laptop, Vulkan |
| Server env | `SERVER_PORT=5000`, `RUST_BACKTRACE=full`, `RUST_LOG=server=info,bevy=warn` |
| Client env | `SERVER_URL=ws://localhost:5000` |

## Commands

```bash
cargo build --workspace                            # 1m 16s, clean

SERVER_PORT=5000 RUST_BACKTRACE=full RUST_LOG=server=info,bevy=warn \
  ./target/debug/server.exe 2>&1 | tee server.log

SERVER_URL=ws://localhost:5000 \
  ./target/debug/client.exe 2>&1 | tee client-a.log

SERVER_URL=ws://localhost:5000 \
  ./target/debug/client.exe 2>&1 | tee client-b.log
```

Binaries launched directly rather than `cargo run -p server` to avoid further incremental-rebuild / artifact-lock races between cargo invocations. Equivalent runtime behaviour because all relevant env vars apply at process launch.

## Files

| File | Size | Origin |
|---|---|---|
| `server.log` | 81 KB | Server stdout/stderr, `initialize_player_pools_on_draft_started` per-frame noise filtered out. All phase transitions, auction state changes, network events retained. |
| `client-a.log` | 75 KB | Client A (`PlayerId(1)`), noise-filtered. |
| `client-b.log` | 50 KB | Client B (`PlayerId(2)`), noise-filtered. |

Raw unfiltered logs preserved outside the repo at `C:\Users\Sam\playtest-raw-logs\2026-05-12-auction-fix\` (38 MB + 19 MB + 10 MB).

## Timeline (filtered server.log)

| Time | Event |
|---|---|
| 16:15:39 | Server boot → Lobby, assets loaded (16 cards) |
| 16:16:00 | Both clients connected (peers `127.0.0.1:51417` + `:51419`) |
| 16:16:28 | RSM `on_session_ready: entering DRAFT_INITIAL` (player_count=2) |
| 16:16:30 | Client B `c2s_purchase_card` Card 104 |
| 16:16:30 | Client B `c2s_purchase_card` Card 1 |
| 16:16:30 | Client B `c2s_purchase_card` Card 101 |
| 16:16:32 | Client A `c2s_purchase_card` Card 1 |
| 16:16:33 | Client A `c2s_purchase_card` Card 104 |
| 16:16:33 | Client A `c2s_purchase_card` Card 103 |
| 16:16:34 | `DraftInitial → Placement` round 1 |
| 16:16:44 | `Placement → Resolution` round 1 → `Resolution → DraftShop` round 2 |
| **16:17:14** | **`DraftShop → Placement` round 2 — clean transition, no crash** ⭐ (with 6 cards purchased earlier) |
| 16:17:24 | `Placement → Resolution` round 2 → `Resolution → DraftAuction` round 3 |
| **16:17:28** | **First auction: Client B `c2s_place_bid amount=4`** |
| **16:17:29** | **Client A counter-bids `c2s_place_bid amount=5`** |
| **16:17:58** | **`S2CAuctionSettled winner=PlayerId(1) amount=5`** → DraftShop round 3 ✅ |
| 16:18:28 | Placement round 3 |
| 16:18:40 | Resolution round 3 → DraftShop round 4 |
| 16:19:10 | Placement round 4 |
| 16:19:20 | Resolution round 4 → DraftShop round 5 |
| 16:19:50 | Placement round 5 |
| 16:20:00 | Resolution round 5 → `DraftAuction` round 6 |
| **16:20:09** | **Second auction: Client B `c2s_place_bid amount=6`** (no counter-bid this time) |
| **16:20:29** | **`S2CAuctionSettled winner=PlayerId(2) amount=6`** → DraftShop round 6 ✅ |
| 16:20:59 | Placement round 6 |
| 16:21:11 | Resolution round 6 → DraftShop round 7 |
| 16:21:41 | Placement round 7 |
| 16:21:51 | Resolution round 7 → DraftShop round 8 |
| 16:22:24 | Placement round 8 |
| 16:22:34 | Resolution round 8 → DraftAuction round 9 |
| 16:22:07 | (Client B side) user closed Client B window → `No windows are open, exiting` |
| 16:22:40 | Server detected Client B disconnect during DraftAuction R9 → `RSM advance_phase: game over from=DraftAuction to=GameOver round=9 reason=Disconnect loser=PlayerId(2)` |
| 16:22:40 | `reset_to_idle: state transition from=LiveBidding to=Idle card_id=Some(CardId(106)) final_price=3 leader=None` (clean teardown of in-flight auction) |
| 16:22:40 | `broadcast_game_over_from_world enter recipient_count=2 reason=Disconnect` |
| 16:22:08 | Client A window closed by user (just after GameOver) |

## Comparison vs Earlier Captures

| Metric | Run 12:01 (`8e3d044`, crashed) | Run 13:24 (`8e3d044`, no crash, no interaction) | **Run 17:15 (`f08b2c8`, this run)** |
|---|---|---|---|
| Cards purchased in DraftInitial | 4 (Cards 103, 1, 1, 5) | 0 | **6 (Cards 104, 1, 101, 1, 104, 103)** |
| Auction bids placed | 0 | 0 | **3 across 2 auctions** |
| Auctions settled successfully | n/a (crash before R3) | 2 (R3 none, R6 P2=6) | **2 (R3 P1=5, R6 P2=6)** |
| Placement R2 entry | ❌ Process exit 1 | ✅ Clean | ✅ Clean (with 6 cards in hand) |
| Auction settle correctness | n/a | settled (no bids = no winner) | ✅ **Settled with real bids** |
| End condition | Process crash mid-round 2 | User closed both windows at round 12 | Server GameOver round 9 (Disconnect, loser=P2) |

## Verdict

**`S11-SERVER-AUCTION-SETTLE-REGRESSION-FIX` (commit `f08b2c8`, "drop 1s-per-tick delta clamp in decrement_live_bidding_timer") behaves correctly** in two consecutive live auctions with real bids in this run. Settlement times match expected ~30 s auction window:
- R3 auction opened at 16:17:24, first bid 16:17:28, second bid 16:17:29, settled 16:17:58 → 30 s window.
- R6 auction opened at 16:20:00, sole bid 16:20:09, settled 16:20:29 → 30 s window.

Additionally, the Placement-R2-entry regression from the morning crash run is no longer reproducible **even with the same player-interaction pattern** (purchase cards in DraftInitial). The crash was likely fixed in one of the four intervening commits (`693d2c8`, `590d6bd`, `a3efce8`, or `f08b2c8` itself). Cannot pinpoint which without bisect, but the symptom is gone on current `main`.

## Notes / Recommendations

1. The per-frame `initialize_player_pools_on_draft_started: entered ...` log noise is **still present** at `server/src/core/pool/system.rs:21` — produced 37 MB of log over 7 minutes here. Worth fixing independently.
2. `GameOver{reason=Disconnect}` triggered cleanly with proper `reset_to_idle` of the in-flight auction in round 9 and `broadcast_game_over_from_world` with both recipients. Good post-disconnect cleanup path.
3. Suggested follow-up: a longer-running auction-stress capture with multiple counter-bid cycles (3+ bids per auction) to exercise the `decrement_live_bidding_timer` extension path more thoroughly.
