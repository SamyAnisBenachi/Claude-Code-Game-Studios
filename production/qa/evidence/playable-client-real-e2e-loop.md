# PLAYABLE-003 Real End-to-End Loop Evidence

This is internal friend-game evidence only. It is not public release readiness,
not broad accessibility completion, not playtest validation, not fun-hypothesis
validation, not QA sign-off, and not full playable-client manual QA.

## Version Control

- Worker branch: `work/playable-003-auction-placement-resolution-evidence`
- Base commit for prompt 298: `4f72a37f344ee217b46fac777d00f3085e8894af`
- Scope: continue PLAYABLE-003 only; do not create PLAYABLE-004; do not run `/story-done`.
- Prior prompt 296 branch: `work/playable-003-draftinitial-draftshop-evidence`
- Prior prompt 290 branch: `work/playable-003-room-session-evidence-repair`

## Runtime

- Environment type: controlled in-process real Lightyear server plus two primary client apps
- Local hardware, usernames, machine identifiers, process ids, raw timestamps,
  local filesystem paths, raw room codes, transient port values, and unsafe
  branch-local metadata are redacted from committed evidence
- Rust: `rustc 1.95.0`
- Cargo: `cargo 1.95.0`
- Browser target: not exercised in this repair pass

## Root Cause And Repair

Prompt 296 reached `DRAFT_SHOP` through the real server route:

`DRAFT_INITIAL -> PLACEMENT -> RESOLUTION -> DRAFT_SHOP`.

Prompt 298 extended that controlled real-Lightyear path and repaired the first
auction evidence blockers:

- The controlled room-flow server app did not include `AuctionPlugin`, so it
  could route into `DraftAuction` without running the auction draw/bid/settle
  systems.
- The controlled catalog fixture had no neutral Rare or Legendary cards, while
  `server/src/feature/auction/system.rs::auction_card_is_round_eligible` only
  permits neutral Rare cards, plus neutral Legendary cards at the configured
  entry round.
- Production `server/src/feature/auction/system.rs::auction_tick_system` wrote
  the internal auction-card message but did not dispatch the shared
  `S2CAuctionCard` payload over Lightyear. Prompt 298 now sends the real S2C
  auction card from the authoritative auction draw.

Open replay-content blocker:

- `S2CResolutionEvent` is emitted and received, but accepted placements are not
  proven as `UnitPlaced` replay entries. `server/src/feature/board/placement.rs::close_placement_phase`
  consumes and clears `PendingPlacements` after reveal/spawn, while
  `server/src/feature/combat/mod.rs::apply_placements` builds `UnitPlaced`
  trace from `PendingPlacements` after `BoardSystemSet::PlacementClose`.

## Capture Manifest

Capture directory: `production/qa/evidence/captures/playable-client-real-e2e-loop/`

- `prompt-290-room-session-trace.json`: sanitized controlled Lightyear trace for
  host create-room, joiner join-room, class select/confirm, class reveal, and
  server-confirmed `DraftInitial` session entry.
- `prompt-296-draft-shop-trace.json`: sanitized controlled Lightyear trace for
  DraftInitial offering, purchase/acquisition/economy, ready/retract,
  placement submit, resolution, and server-confirmed `DraftShop`.
- `prompt-298-auction-placement-resolution-trace.json`: sanitized controlled
  Lightyear trace from `DraftShop` through non-empty placement, resolution,
  auction card/bid/settlement, post-auction non-empty placement, resolution,
  and next-loop `DraftShop`.
- `phase-captures.md`: reached and unreached phase notes.
- `attempt-*`, `server.*`, `client-*`, and `live-process-summary*.json`:
  sanitized earlier native-process capture summaries retained for launch and
  protocol-mismatch history.

## Reached Endpoint

Prompt 298 controlled endpoint:

1. Both clients reached the real server-authored `DRAFT_SHOP` endpoint from
   prompt 296.
2. Both clients sent real `C2SSignalReady` from `DRAFT_SHOP`.
3. Server advanced to `PLACEMENT`; both clients sent real non-empty
   `C2SSubmitPlacement` payloads using card IDs acquired from server-authored
   hand state.
4. Server accepted both non-empty placement batches and emitted non-empty
   `S2CPlacementReveal`.
5. Server emitted `S2CResolutionEvent` and advanced into `DRAFT_AUCTION`.
6. Both clients received server-authored `S2CAuctionCard`.
7. Host sent real `C2SPlaceBid`; both clients received
   `S2CAuctionBidAccepted` and `S2CAuctionSettled`.
8. Host received `S2CCardAcquired { source: AuctionWon }`.
9. Both clients reached post-auction `DRAFT_SHOP`, readied again, submitted
   another non-empty placement, received another non-empty reveal and
   `S2CResolutionEvent`, then reached the next-loop `DRAFT_SHOP`.

Exact reached route:

`DRAFT_INITIAL -> PLACEMENT(empty) -> RESOLUTION -> DRAFT_SHOP -> PLACEMENT(non_empty) -> RESOLUTION -> DRAFT_AUCTION -> DRAFT_SHOP -> PLACEMENT(non_empty) -> RESOLUTION -> DRAFT_SHOP`.

Game-over was not reached and is not claimed.

## Phase Checklist

| Phase | Result | Evidence |
|---|---|---|
| Fresh hello | Reached | `playable_client_real_e2e_loop_test`, `prompt-290-room-session-trace.json` |
| Create/join room | Reached | `prompt-290-room-session-trace.json` |
| Class select/confirm/reveal | Reached | `prompt-290-room-session-trace.json` |
| Server-confirmed session entry | Reached | Server room `GameActive`, `SessionConfig`, `RoundState::DraftInitial`, client `S2CPhaseChanged(DraftInitial)` |
| DRAFT_INITIAL offering | Reached | `prompt-296-draft-shop-trace.json`, `prompt-298-auction-placement-resolution-trace.json` |
| Purchase/acquisition/economy | Reached | Real `C2SPurchaseCard`, `S2CCardAcquired`, `S2CGoldUpdate` |
| Ready/retract | Reached | Real `C2SSignalReady` ready/retract/ready observed by server RSM |
| DRAFT_SHOP | Reached | Prompt 298 observes three `DRAFT_SHOP` S2C phase changes per client |
| Placement submit | Reached | Empty submit plus two non-empty submit rounds through real `C2SSubmitPlacement` |
| Placement reveal | Reached | Empty reveal plus two non-empty `S2CPlacementReveal` rounds |
| Resolution event | Reached with content limit | Three `S2CResolutionEvent` messages per client; `UnitPlaced` replay content remains open |
| Auction | Reached | `S2CAuctionCard`, real `C2SPlaceBid`, `S2CAuctionBidAccepted`, `S2CAuctionSettled`, `S2CCardAcquired(source=AuctionWon)` |
| Next loop | Reached | Next `DRAFT_SHOP` after post-auction placement/resolution |
| Game-over | Not reached | Not claimed |

## Defects And Gaps

| ID | Severity | Owner/System | Status | Friend-game Impact | Evidence |
|---|---|---|---|---|---|
| PLAYABLE-003-D7 | Major evidence gap | Full friend-game loop capture | Partially repaired | Controlled real-Lightyear evidence now covers auction, non-empty placement, resolution events, and next-loop `DRAFT_SHOP`; live native manual completion remains unproven | `prompt-298-auction-placement-resolution-trace.json` |
| PLAYABLE-003-D9 | Major | Auction network dispatch | Repaired in prompt 298 | Clients could not receive the server-authored auction card needed for real auction entry | `server/src/feature/auction/system.rs::auction_tick_system` |
| PLAYABLE-003-D10 | Major | Resolution replay content | Open | `S2CResolutionEvent` is received, but accepted placements are not proven as `UnitPlaced` replay entries | `server/src/feature/board/placement.rs::close_placement_phase`, `server/src/feature/combat/mod.rs::apply_placements` |

## Verification Results

Prompt 298 verification:

- `cargo fmt -p client -p server -- --check`: PASS.
- `cargo check --workspace`: PASS.
- `cargo test -p server --test playable_client_real_e2e_loop_test`: PASS, 4 passed.
- PLAYABLE-001 focused tests:
  - `cargo test -p client --test playable_client_lobby_entry_test`: PASS, 5 passed.
  - `cargo test -p server --test playable_client_lobby_entry_server_test`: PASS, 3 passed.
- PLAYABLE-002 focused tests:
  - `cargo test -p client --test playable_client_draft_shop_hand_bridge_test`: PASS, 4 passed.
  - `cargo test -p server --test playable_client_draft_ready_bridge_test`: PASS, 3 passed.
- Client shop/auction/placement/resolution regressions:
  - `shop_auction_ui_auction_activation_test`: PASS, 6 passed.
  - `shop_auction_ui_auction_bid_buttons_test`: PASS, 5 passed.
  - `shop_auction_ui_auction_bid_target_focus_test`: PASS, 4 passed.
  - `shop_auction_ui_auction_feedback_test`: PASS, 6 passed.
  - `shop_auction_ui_shop_panel_test`: PASS, 8 passed.
  - `hand_ui_placement_submit_core_test`: PASS, 5 passed.
  - `card_animations_placement_reveal_test`: PASS, 9 passed.
  - `board_rendering_resolution_anim_queue_test`: PASS, 5 passed.
- Server auction/acquisition/session/RSM/pool/economy/network/placement/resolution regressions:
  - `auction_fifo_ordering_test`, `auction_phase_entry_test`,
    `auction_bid_validation_gate_test`, `auction_resolution_settlement_test`,
    `auction_resolution_settlement_integration_test`, `auction_pool_integration_test`,
    and `accepted_bid_reservation_test`: PASS.
  - `card_acquisition_draft_initial_test`,
    `card_acquisition_purchase_atomicity_test`,
    `card_acquisition_state_scaffold_test`,
    `card_acquisition_refresh_cost_test`,
    and `card_acquisition_draw_pipeline_test`: PASS.
  - `room_create_join_test`, `class_reveal_test`, `class_lifecycle_test`,
    `session_ready_test`, `session_scaffold_test`, `lobby_to_draft_initial_test`,
    and `reconnect_snapshot_test`: PASS.
  - `rsm_timers_test`, `rsm_transitions_test`, `rsm_network_dispatch_test`,
    and `rsm_scaffold_test`: PASS.
  - `pool_session_ready_test`, `pool_manual_refresh_test`, `shop_dispatch_test`,
    `economy_draft_subscriber_test`, `economy_network_dispatch_test`,
    `economy_interest_snapshot_test`, and `economy_round_trace_test`: PASS.
  - `placement_submit_authority_validation_test`, `placement_buffer_test`,
    `resolution_event_log_test`, and `substep1_placement_test`: PASS.
- `git diff --check origin/main...HEAD`: PASS.

## Non-Claims

- No public release readiness.
- No broad accessibility completion.
- No playtest validation.
- No full playable-client manual QA.
- No live native manual two-client completion.
- No game-over coverage.
- No full game completion.
