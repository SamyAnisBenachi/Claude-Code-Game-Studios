# Story 020: S18-AUCTION-WON-CARD-DISPOSITION-001 -- Auction-Won Card Disposition Contract + Winner Discoverability

> **Epic**: Shop / Auction UI
> **Story ID**: S18-AUCTION-WON-CARD-DISPOSITION-001
> **Status**: Draft -- Sprint 18 candidate (Must Have), NOT activated. No sprint plan currently activates this row. `production/sprint-status.yaml`, `production/sprints/sprint-17.md`, `production/sprints/sprint-18.md` (does not exist at authoring), `production/stage.txt`, and every `production/session-state/*` file are NOT modified by this authoring run.
> **Layer**: Shop / Auction UI -- auction-won card disposition contract + winner UI affordance + observability
> **Type**: Integration -- spans server settle path, client shop-auction settlement display, hand UI newly-acquired affordance, QA snapshot enrichment
> **Sprint**: Sprint 18 candidate (Must Have). Authoring does NOT activate Sprint 18 (which does not yet have a plan file). Activation is a separate explicit prompt.
> **Authored**: 2026-05-18 by PROMPT 1137
> **Authoring source-of-truth**: `origin/main@05192b5f830c5d5b17ed7af07df37f56187130fc` (PROMPT 1125 `story-done(s17): close S17-OPS-VULKAN-VALIDATION-GATING-001 (PROMPT 1125)`)
> **Source audit**: PROMPT 1131 `reports/PROMPT-1131-game-state-to-visual-contract-deep-audit.md` §3 AUDIT-1131-02 (P0); §5 New Findings; §6 Observability gap (3); §7 Lane B1 + B2 + Lane D3.

---

## Target Epic Justification

This story is filed under `production/epics/shop-auction-ui/` rather than `production/epics/auction-system/`, `production/epics/card-acquisition/`, or `production/epics/hand-ui/`. Justification:

- The **server-side disposition contract is already implemented** at `server/src/feature/auction/system.rs:805-822` (`award_auction_card` calls `hand_push(hands, winner, card_id)` and pushes `S2CCardAcquired { card_id, source: CardSource::AuctionWon }` to the winner). It matches `design/gdd/auction-system.md` §"Case A" rule 2 verbatim. The protocol-level disposition does NOT change. This story is **NOT** a server protocol or auction-system state-machine change.
- The **visible gap** observed by AUDIT-1131-02 is **client-side discoverability + observability**: the winner is not visibly told they just acquired a card, the card is not visually salient during the 12s post-auction Placement window, and no server / client log or snapshot field captures the disposition for future audits. These are presentation-layer + observability concerns.
- The Shop / Auction UI epic already owns the auction settlement display surface (existing `story-007-auction-settlement-and-shop-transition.md` Ready, owning `S2CAuctionSettled` toast / transition flow). The natural follow-up to settlement is the post-settle winner affordance — same surface, same panel tree, same `ShopAuctionUiPlugin`. The hand UI affordance (newly-acquired pulse) is a small additive widget on hand cards that the Shop / Auction UI epic cross-cites; per the dependency map at the head of the epic, Hand UI is a Downstream (soft) consumer.
- The Auction System epic (`production/epics/auction-system/`) is closed-scope server-side; its EPIC.md DoD reads "`ResMut<AuctionState>` appears in exactly one system (`auction_tick_system`)". This row does not add a writer or reorder the system — it adds a single tracing-only log line at the settle path (Lane D3 of the audit) and does not change the state machine.
- The Card Acquisition epic (`production/epics/card-acquisition/`) owns `S2CCardAcquired` definition and the generic hand-grant pipeline; this row consumes the existing pipeline without redefining it.
- The Hand UI epic (`production/epics/hand-ui/`) is cross-cited for the newly-acquired pulse; per existing GDD §"Interactions" the hand UI already receives `S2CCardAcquired` and adds the card to the fan. This row adds a brief visual marker on the card entity, consistent with the existing Hand UI affordance surface.

The Shop / Auction UI epic is therefore the correct primary owner for **contract documentation + winner-side settlement display follow-through + post-auction Placement winner affordance**. Cross-surface AC references to Hand UI and to a server tracing-only log line are documented explicitly in the AC list and reflect the audit's Lane B + Lane D3 grouping.

---

## Status / No-Claim Banner

This story is a Sprint 18 Must Have **candidate** authored by PROMPT 1137. **No sprint is activated by this authoring run.** PROMPT 1137 does NOT modify `production/sprint-status.yaml`, `production/sprints/sprint-17.md`, `production/sprints/sprint-18.md` (does not exist), `production/sprints/sprint-16.md` or any earlier sprint file, `production/stage.txt`, any `production/session-state/*` file, any QA-plan / smoke / Team-QA / gate-check / release-check artifact under `production/qa/`, any code under `client/`, `server/`, `shared/`, `tests/`, `Cargo.toml`, `Cargo.lock`, `.cargo/`, `.github/`, or `Trunk.toml`. PROMPT 1137 does NOT run `/story-readiness`, `/dev-story`, `/story-done`, `/smoke-check`, `/team-qa`, `/gate-check`, `/release-check`, `/qa-plan`, `cargo`, `trunk`, or any CI command.

This story does **not** claim: public release readiness, release-candidate readiness, full game completion, broad / Standard-tier accessibility completion (`QA-COND-0005`), Standard-tier hit-target conformance (>=44 px), playtest / fun-hypothesis validation (`QA-COND-0006`), full playable-client manual QA, two-client GAME_OVER closure (`S8-QA-001-W1`), final-art / asset-production completion (`PAW-TD-*-a`), `Polish->Release` gate-check retry, stage advance, closure of `S11-HUD-TIMER-EYEBALL-VISUAL-001`, closure of any of the 24 PROMPT 1022 audit findings, closure of any AUDIT-1131-* finding outside AUDIT-1131-02 (and the Lane D3 observability-only sub-task that closes the AUDIT-1131-02 observability gap), closure of AUDIT-1131-01 (placement cell-index translation — distinct surface, distinct Lane A in PROMPT 1131 §7), closure of any other AUDIT-1076-* / SOURCE-1077-* finding, or rewrite of the placement drag pipeline (TQ-S12-C2 binding preserved).

**No optimistic client-side authority is introduced or proposed.** No protocol shape change. No new server-authoritative state. No new C2S / S2C message. The auction system continues to send `S2CAuctionCard` / `S2CAuctionBidAccepted` / `S2CAuctionSettled` / `S2CCardAcquired { source: CardSource::AuctionWon }` unchanged; this row formalises the disposition contract (which is **already** in the source and in the GDD) and adds **client-side discoverability + observability** on top of the existing pipeline.

Sprint 17 disposition `active` preserved unchanged. Sprint 16 / 15 / 14 / 13 / 12 / 11 / 10 dispositions preserved unchanged. PROMPT 761 `Polish->Release` gate-check `FAIL` preserved. `PAW-TD-*-a`, `QA-COND-0005`, `QA-COND-0006`, `TQ-S12-C1..C7` preserved verbatim. `S8-QA-001-W1` OPEN preserved.

---

## Problem

In the 2026-05-18 user-test bundle captured at `origin/main@c300b141` (snapshot run `manual-friend-game-evidence-2026-05-18/run-1/`), 2 of 3 auction wins did not produce a board unit. PROMPT 1131 audit §3 row AUDIT-1131-02 (P0) names the cause as a **product contract gap**:

- **R3** winner Player 2 wins `CardId 107` at 4g (`server.log:157-160`). Server settles correctly: `S2CAuctionSettled { winner: Some(PlayerId(2)), amount: 4 }` is dispatched, `S2CCardAcquired { card_id: 107, source: CardSource::AuctionWon }` is unicast to P2, and the card is appended to P2's `PlayerHands`. The 12s auction-followup PLACEMENT window then opens (`auction_followup_placement_timer_seconds = 12` per `design/gdd/round-state-machine.md` §"Rule 9"). **Player 2 does not drop the card.** Round closes with `committed_players=0, spawned_units=0` (`server.log:192-193`).
- **R6** winner Player 1 wins `CardId 107` at 11g: manual-drop succeeded (`server.log:357-359`). This is the only auction win in the run that produced a board unit.
- **R9** no winner — out-of-scope for this story.

The audit's verdict: "auction-won card must either: (a) auto-place after settle, or (b) UI must explicitly prompt user to 'drop your won card', or (c) be reservable for next round. **No story currently defines this contract.**"

This story **defines the contract** and adds the **client-side affordance** needed to close AUDIT-1131-02.

---

## Contract Decision

### Chosen disposition: **A — Auction-won card goes to hand; manual placement during next Placement phase with visual prompt; persists across rounds if not placed.**

PROMPT 1137 evaluates the four options listed in the audit and the prompt brief:

- **A — Hand + visual prompt + manual placement (recommended default)** -- **CHOSEN**.
- **B — Auto-place into a legal default cell at next Placement phase if possible** -- rejected.
- **C — Reserve / bench until explicitly placed later** -- rejected.
- **D — Other minimal contract if source / GDD already implies one** -- collapses into A (see below).

### Rationale for A (chosen)

1. **Server already implements A.** `server/src/feature/auction/system.rs:805-822` (`award_auction_card`) calls `hand_push(hands, winner, card_id)` and pushes `S2CCardAcquired { card_id, source: CardSource::AuctionWon }` to the winner. **No protocol change required.**
2. **GDD already specifies A.** `design/gdd/auction-system.md` §"Case A — Current leader exists" rule 2 reads: "If `leader.hand_size < 10`: add `card_id` to leader's hand. Unicast `S2CCardAcquired { card_id, source: AcquisitionSource::AuctionWon }` to the winner." The disposition is normative in the GDD.
3. **Player agency preserved.** The player chooses where the card goes (which lane, which row). No server-side guessing.
4. **Consistent with all other acquisition sources.** `CardSource` enum at `shared/src/protocol.rs:265-277` has 10 variants (`ShopPurchase | DraftInitial | AuctionWon | FreeCardPick | PrismLane1..5 | KeywordEffect`); every other source funnels to the same hand-push + manual-place flow. Auction is not a special case; treating it as one would create an inconsistency.
5. **Least invasive.** No new server state, no new protocol message, no new client-side placement pipeline. Source-side changes are scoped to: (a) a settlement-display affordance on the winner client during the post-auction Placement window, (b) a hand-card newly-acquired pulse, (c) a tracing-only server log line for future observability, (d) snapshot enrichment.
6. **Matches the existing 12s post-auction Placement timer** (`auction_followup_placement_timer_seconds = 12`, GDD §"Rule 9", AC RSM-29c). The 12s value was already chosen specifically because "DRAFT_AUCTION imposes peak cognitive load (20-50s of sustained bid attention). A 10s PLACEMENT immediately after auction is panic input, not deliberate commitment." The 12s gives the player time to deliberate; this story adds the affordance that makes the deliberation discoverable.

### Why B (auto-place) is rejected

- B requires the server to choose a lane + row on behalf of the winner. There is no GDD-defined "default legal cell" formula. Card types include `BoardCell { lane, cell }`, `TargetUnit { lane, unit_id }`, `TargetObj { player_id, lane }`, `LaneWide { lane }`, `Instant` — picking a default for non-BoardCell types is undefined.
- B removes player agency (the auction-card placement is often a strategic choice; auto-place would frequently put it in the wrong lane).
- B requires a new protocol message or a new authoritative server state field. The auction system epic DoD enforces "`ResMut<AuctionState>` appears in exactly one system (`auction_tick_system`)"; adding an auto-place writer would violate the DoD.
- B contradicts ADR-002 (Client-Server Authority): UI sends intent only; if the server places the card without a client `C2SSubmitPlacement` for it, the existing placement flow assumes intent came from the client.

### Why C (reserve / bench) is rejected

- C requires a new server-side container (e.g. `PlayerReserve` resource) parallel to `PlayerHands`, with new persistence semantics across rounds.
- C requires a new protocol message (e.g. `S2CReserveUpdated`) or a new field on `S2CGameSnapshot`. Both are protocol-shape changes; the audit's Lane B brief explicitly avoids protocol changes.
- C requires new UI (the bench surface, the move-from-bench-to-hand interaction). Every interaction multiplies QA scope.
- C diverges from the GDD ("add `card_id` to leader's hand"). Re-opening the GDD here would require a `/design-system` + `/design-review` + cross-GDD propagation pass, all of which are out of scope for AUDIT-1131-02.
- C would make the auction acquisition gameplay-distinct from every other acquisition source. The audit signal (player did not place the card) is plausibly **discoverability**, not "the player wanted to reserve it". Without playtest evidence that players actively want a reserve mechanic, C is over-engineering.

### Why D collapses into A

The auction-system GDD already implies A (card → hand). The "minimal contract if source / GDD already implies one" reading of D is therefore exactly A. PROMPT 1137 names A explicitly to make the contract un-ambiguous in the story file.

### Do-nothing path (the "what happens if the user does nothing" question)

If the winner does not stage / submit the auction-won card during the 12s post-auction PLACEMENT window:

1. The PLACEMENT phase ends normally (timer expiry OR all-submit).
2. The RSM advances to RESOLUTION.
3. The card **remains in the winner's `PlayerHands`** for the next PLACEMENT phase.
4. The card is **not auto-placed**, **not discarded**, **not auto-submitted**.
5. The card may be placed in any subsequent PLACEMENT phase (including the regular 10s PLACEMENT two rounds later, after the next DRAFT_SHOP).
6. The hand-size cap (`max_hand_size = 10` per existing economy / hand-ui semantics) applies: if at the time of a future acquisition the hand is full, the new card is discarded per existing `S2CCardAcquired` hand-full handling (`hand-ui.md` Interactions table). This story does NOT change the hand-cap semantics; the auction-won card carries no special priority over other hand cards.

This do-nothing path is **already** the implemented behaviour on `origin/main@05192b5` — the server appends to `PlayerHands` and does not later prune un-placed cards. This story documents it explicitly so that future audits can verify it from the story file alone, and adds a tracing log line (Lane D3) so that future audits can also verify it from logs without source dives.

### Server / client state-source ownership

| Concern | Owner | Rationale |
|---|---|---|
| `PlayerHands` (authoritative card list, per player) | Server | Pre-existing; per `server/src/core/hand_state.rs` (or equivalent) + `award_auction_card`. Unchanged. |
| `S2CCardAcquired { source: CardSource::AuctionWon }` dispatch | Server | Pre-existing; `server/src/feature/auction/system.rs:815-822`. Unchanged. |
| `S2CAuctionSettled` dispatch | Server | Pre-existing; `server/src/feature/auction/system.rs:714-733`. Unchanged. |
| Winner client "newly-won card" highlight state | Client (presentation-only) | Derived from the most recent `S2CCardAcquired` with `source == AuctionWon`; cleared on the next `S2CPhaseChanged` whose target phase is `Resolution` or later, OR on a successful `C2SSubmitPlacement` that includes the won card, whichever comes first. Pure UI; not authoritative. |
| Loser client "opponent won X for Yg" toast state | Client (presentation-only) | Derived from `S2CAuctionSettled { winner: Some(p), amount: y }` where `p != local_player_id`. Pure UI; not authoritative. |
| Auction-followup PLACEMENT timer (12s) | RSM | Pre-existing; `placement_timer_used_ms = auction_followup_placement_timer_seconds * 1000 * placement_timer_multiplier_effective` (RSM-29c). Unchanged. |
| Snapshot `auction_won_pending` block | Client (qa-snapshot only) | New observability scaffolding; populated by client from `S2CCardAcquired { source: AuctionWon }` + last `S2CAuctionSettled`. Not authoritative; **diagnostic-only**. |

This row introduces **zero new authoritative state**. All new state is presentation-layer or QA-snapshot-only.

---

## Acceptance Criteria

All criteria are independently checkable.

### Disposition contract

- [ ] **AC1 -- Server hand-grant on `S2CAuctionSettled { winner: Some(p) }` unchanged**: GIVEN an auction settles with `current_leader = Some(PlayerId(p))` and `current_price = price`, WHEN `try_settle_auction` runs (`server/src/feature/auction/system.rs:670-745`), THEN `award_auction_card(p, card_id, hands, connections, outbox)` is called (line 713), `hand_push(hands, p, card_id)` returns `Ok(())` (unless hand is full, which is unreachable under correct RSM enforcement per GDD §"Case A" rule 3), and `S2CCardAcquired { card_id, source: CardSource::AuctionWon }` is dispatched to peer `peer_for_player(connections, p)`. The integration test re-asserts the existing pre-Sprint-18 server behaviour against the AUDIT-1131-02 disposition expectation. No source code under `server/src/feature/auction/` is modified by this AC.

- [ ] **AC2 -- Loser receives no card on `S2CAuctionSettled { winner: Some(p) }` where `p != local_player_id`**: GIVEN the same settle, WHEN the loser client drains `S2CAuctionSettled`, THEN no `S2CCardAcquired` for `card_id` arrives at the loser (already true per existing server behaviour: `S2CCardAcquired` is unicast to winner only). The integration test asserts unicast. No source code modification by this AC.

- [ ] **AC3 -- No-bid settlement (`S2CAuctionSettled { winner: None, amount: 0 }`) does not grant any card to any player**: GIVEN an auction settles with no leader (Case B), WHEN `try_settle_auction` runs, THEN no `S2CCardAcquired` is dispatched and the featured card is discarded back to the pool per existing `distribute()` semantics. No source modification by this AC.

### Winner-side discoverability (post-auction PLACEMENT window)

- [ ] **AC4 -- Winner sees an "Auction won" affordance during the auction-followup PLACEMENT window**: GIVEN the local player is the auction winner and the next phase is `Placement` (auction-followup; `placement_timer_used_ms = 12000 * placement_timer_multiplier_effective` per RSM-29c), WHEN the PLACEMENT phase begins, THEN the Shop / Auction UI surfaces a one-shot textual or chrome affordance naming the won-card disposition. Concrete wording chosen by the implementing worker (e.g. `"Auction won: <card-name>"` or equivalent localizable token); consistent with existing `ShopAuctionSettledReceived` toast styling (story-007 settlement-and-shop-transition lineage). The affordance MUST be visible from PLACEMENT entry through either: (a) the won-card being staged via drag-drop, OR (b) the PLACEMENT phase ending. The affordance MUST be reachable by sighted players at 1366x768 (minimum supported resolution per existing UX spec).

- [ ] **AC5 -- Newly-acquired card visual marker on hand fan during the auction-followup PLACEMENT window**: GIVEN AC4 conditions, WHEN the won-card entity appears in the hand fan (per existing Hand UI `S2CCardAcquired` consumption path), THEN the entity carries a visible "newly-acquired" marker (e.g. a glow, pulse, tinted border, or chevron — concrete strategy chosen by the implementing worker and justified in the commit message). The marker MUST clear on either: (a) the won-card being staged via drag-drop, OR (b) the PLACEMENT phase ending. The marker MUST NOT persist across PLACEMENT phases.

- [ ] **AC6 -- Won-card identity is unambiguous on the winner client**: GIVEN the winner client receives `S2CAuctionSettled { winner: Some(local_player_id), amount }` and `S2CCardAcquired { card_id, source: CardSource::AuctionWon }` in the same frame burst (server-side ordering: `award_auction_card` is called before `push_settled` per `system.rs:713-714`; client-side ordering is reliable-channel-preserved), WHEN the winner client renders the AC4 affordance and the AC5 hand marker, THEN both reference the same `card_id` (the affordance references the card-pool entry, the marker is attached to the hand entity). The integration test asserts the two UI states agree.

### Loser-side feedback

- [ ] **AC7 -- Loser sees an "opponent won" settlement toast**: GIVEN the local player is NOT the auction winner (`S2CAuctionSettled { winner: Some(other), amount: y }`), WHEN settlement is received, THEN the existing settlement display (story-007 lineage) renders a toast naming the opponent and the price (e.g. `"Opponent won <card-name> for <y>g"`). The toast surface is the existing `ShopAuctionSettledReceived` overlay; this AC re-asserts the loser-side path without adding new surfaces. The toast clears per the existing settlement-transition timer.

### Do-nothing / no-op path

- [ ] **AC8 -- Un-staged auction-won card persists in the winner's hand if the auction-followup PLACEMENT phase ends without a submission for it**: GIVEN the winner receives `S2CCardAcquired { source: AuctionWon }` at PLACEMENT entry and does NOT include the won-card in their `C2SSubmitPlacement` (or does not submit at all and the 12s timer expires), WHEN the PLACEMENT phase ends and RESOLUTION begins, THEN `PlayerHands[winner]` continues to contain the won-card on the server, AND the next `S2CGameSnapshot` (if any) lists the won-card in the winner's hand. The integration test simulates the no-submit path and asserts hand persistence. No source change to `server/src/feature/auction/` or `server/src/feature/board/` is required by this AC; the test re-asserts existing behaviour.

- [ ] **AC9 -- AC4 affordance and AC5 marker both clear at PLACEMENT-phase exit even on the no-op path**: GIVEN the no-op path of AC8, WHEN the auction-followup PLACEMENT phase ends, THEN the AC4 affordance is dismissed AND the AC5 hand-fan marker is removed. The won-card entity remains in the hand fan (per AC8), but it no longer carries the "newly-acquired" visual. If the same card is re-staged in a later PLACEMENT phase, the AC5 marker MUST NOT re-appear (the marker is one-shot per auction settle, not per-phase).

### Observability (Lane D3 from PROMPT 1131 §6 and §7)

- [ ] **AC10 -- Server emits a tracing log at settle naming the disposition**: GIVEN `try_settle_auction` runs Case A or Case B, WHEN `outbox.push_settled` is called, THEN a server-side tracing log line is emitted with fields (`target = "server::game"`, level `info` or higher): `{ event: "auction_settled", winner: <Option<u8>>, card_id: <u32>, current_price: <u32>, hand_size_before: <u8>, hand_size_after: <u8> }`. On Case B, `winner = None` and `hand_size_*` fields are omitted or `0`. The implementing worker chooses the exact tracing macro shape (single multi-field call preferred). This log line is the **AUDIT-1131-02 observability hook** required by PROMPT 1131 §6 row 3 and Lane D3. The log is **trace-only**; no behaviour change.

- [ ] **AC11 -- Client QA snapshot exposes a one-shot `auction_won_pending` block during the auction-followup PLACEMENT window**: GIVEN the winner client is in the auction-followup PLACEMENT phase and the local player won the most recent auction, WHEN the QA snapshot is generated, THEN the snapshot JSON includes a block (preferred shape):
  ```json
  "auction_won_pending": {
    "card_id": <u32>,
    "acquired_phase": "Placement",
    "settle_round": <u32>,
    "staged_yet": <bool>,
    "submitted_yet": <bool>
  }
  ```
  On non-winner clients, on Case B (no-winner) auction-followup PLACEMENT, and on non-auction-followup PLACEMENT phases, the block is **absent** (NOT `null` — absent). The block clears (becomes absent) at the next `S2CPhaseChanged` whose target phase is `Resolution` or later, OR on a successful `C2SSubmitPlacement` that includes the won-card (whichever comes first). The exact JSON key naming is chosen by the implementing worker; the contract is "one block per auction win on the winner client, scoped to the auction-followup PLACEMENT window". This block satisfies PROMPT 1131 §6 row 9 (placement_state observability gap) for the auction-won surface specifically.

### Tests required (before implementation closes)

- [ ] **AC12 -- Server integration test: auction settle disposition + hand grant + tracing log**: A new test under `tests/integration/auction/` (or extension of existing `tests/integration/auction/*_test.rs` per local pattern; exact path chosen by the implementing worker) constructs a real Bevy 0.18 `App`, runs an auction to settlement (Case A: accepted bid + timer expiry), and asserts: (a) `S2CCardAcquired { source: AuctionWon }` is in the outbox unicast to the winner, (b) `S2CAuctionSettled { winner: Some(_), amount: _ }` is in the outbox broadcast, (c) `PlayerHands[winner]` contains `card_id` after settle, (d) the AC10 tracing log line was emitted (via `tracing-test` or equivalent harness). A second test asserts the Case B (no-winner) path emits no `S2CCardAcquired`, no hand mutation, and the AC10 log line with `winner = None`.

- [ ] **AC13 -- Server integration test: hand persistence across PLACEMENT-end on no-submit path**: A new test (or extension) drives the RSM through `DraftAuction -> Placement -> Resolution` with the winner client NOT submitting the won-card in `C2SSubmitPlacement`, then asserts `PlayerHands[winner]` still contains the won-card at `Resolution` entry. This is the AC8 BLOCKING test.

- [ ] **AC14 -- Client integration test: AC4 affordance + AC5 marker lifecycle**: A new test under `tests/integration/shop_auction_ui/` (e.g. `auction_won_card_disposition_test.rs`) constructs a real Bevy 0.18 client `App` (per existing `tests/integration/shop_auction_ui/` pattern), simulates the winner-side message sequence (`S2CAuctionSettled { winner: Some(local), amount: 4 } + S2CCardAcquired { card_id: 107, source: AuctionWon } + S2CPhaseChanged { target: Placement, timer_duration_ms: 12000 }`), and asserts: (a) the AC4 affordance entity is present and visible while PLACEMENT is active, (b) the AC5 marker is attached to the hand-fan entity for `card_id 107`, (c) staging the card via simulated drag-drop clears both AC4 and AC5, (d) on the no-op path (no drag-drop + simulated `S2CPhaseChanged { target: Resolution }`), both AC4 and AC5 clear at the phase change.

- [ ] **AC15 -- Client integration test: AC7 loser-side toast lifecycle**: A new test (or extension of `tests/integration/shop_auction_ui/auction_settlement_test.rs` per local pattern) asserts the loser client renders the AC7 settlement toast on `S2CAuctionSettled { winner: Some(other), amount: y }` and clears per the existing settlement-transition timer.

- [ ] **AC16 -- Client unit test: QA snapshot `auction_won_pending` block presence + clearing**: A new test under `tests/unit/qa_snapshot/` (or equivalent per local pattern) asserts: (a) the block is present on the winner client during auction-followup PLACEMENT, (b) the block is absent on non-winner clients, (c) the block is absent on non-auction-followup PLACEMENT, (d) the block becomes absent on phase-change to `Resolution`, (e) the block becomes absent on `C2SSubmitPlacement` including the won-card.

### Scope guards

- [ ] **AC17 -- No protocol shape change**: GIVEN `git diff <activation HEAD>..HEAD`, WHEN inspected, THEN `shared/src/protocol.rs` is **NOT** modified. No new `S2C*` or `C2S*` message variants. No new `CardSource` variants. No new fields on `S2CAuctionSettled`, `S2CCardAcquired`, `S2CGameSnapshot`, or `S2CPhaseChanged`. The disposition contract relies on the **existing** `S2CCardAcquired { source: CardSource::AuctionWon }` + `S2CAuctionSettled { winner, amount }` pair already on `origin/main@05192b5`.

- [ ] **AC18 -- No new authoritative server state**: GIVEN the implementation commit(s), WHEN inspected, THEN no new `Resource` is registered under `server/src/feature/auction/plugin.rs` or `server/src/feature/board/plugin.rs`. No new `AuctionPhase` variant. The only new server-side code is the AC10 tracing log line at the settle path (and any test-bin support code).

- [ ] **AC19 -- No new placement / drag pipeline**: GIVEN the implementation commit(s), WHEN inspected, THEN no source under `client/src/ui/hand/` placement drag-drop pipeline is rewritten, refactored, or moved. The won-card placement flow uses the **existing** drag-drop pipeline (`hand-ui.md` Rules 6-7) and the **existing** `C2SSubmitPlacement` message. **TQ-S12-C2 binding preserved** (no third same-scope drag-runtime retest). **AUDIT-1131-01 (placement cell-index translation) NOT closed by this row** — that is a distinct Lane A surface in PROMPT 1131 §7 and requires a separate story.

- [ ] **AC20 -- ADR-021 schedule preserved**: GIVEN `cargo check -p client --workspace`, WHEN run, THEN no new `SystemSet`, no new `PresentationSet` slot, no new schedule wiring is introduced. The new winner-affordance system + the new hand-marker system live in the existing `PresentationSet` slots owned by `ShopAuctionUiPlugin` and Hand UI plugin respectively.

- [ ] **AC21 -- ADR-002 + ADR-013 preserved**: GIVEN the implementation commit(s), WHEN inspected, THEN no client-side optimistic auction state is introduced. The client renders only what server-authoritative state allows. ADR-013 (auction state machine) and ADR-019 (economy resource architecture) are unchanged.

- [ ] **AC22 -- Authoring-only scope contained for PROMPT 1137**: GIVEN PROMPT 1137 worker branch diff, WHEN inspected, THEN the only files modified by PROMPT 1137 are:
  - `production/epics/shop-auction-ui/story-020-auction-won-card-disposition.md` (NEW; this file)
  - `production/epics/shop-auction-ui/EPIC.md` (index update only — appending story 020 row)
  - `reports/PROMPT-1137-auction-won-card-disposition-contract-story.md` (NEW; the report)
  No code under `client/`, `server/`, `shared/`, `tests/`. No GDD edit. No ADR edit. No sprint plan edit. No QA artifact edit. No production session-state edit. No `production/sprint-status.yaml` edit. No `production/stage.txt` edit. No Cargo / Trunk / CI edit. No skill / agent edit.

- [ ] **AC23 -- Worker branch scope contained for the future `/dev-story` worker**: GIVEN the future implementation worker branch (slug recommendation: `work/s18-auction-won-card-disposition`), WHEN inspected, THEN it pushes only the worker branch — never `main`. Files changed at worker time are scoped to:
  - `server/src/feature/auction/system.rs` (AC10 tracing log line — single tracing call added at the settle path; no behaviour change)
  - `client/src/ui/shop_auction/` (AC4 affordance + AC7 toast surface; one or more `.rs` files)
  - `client/src/ui/hand/` (AC5 newly-acquired marker; one or more `.rs` files)
  - `client/src/presentation/qa_snapshot*.rs` (AC11 snapshot block — single new field in the snapshot serialiser + collector system)
  - `client/Cargo.toml` (test bin registration; additive `[[test]]` block(s) only)
  - `tests/integration/auction/` (AC12 + AC13 test bins; new or extended)
  - `tests/integration/shop_auction_ui/` (AC14 + AC15 test bins; new or extended)
  - `tests/unit/qa_snapshot/` or equivalent (AC16 test)
  - `production/qa/evidence/sprint-<active>-auction-won-card-disposition/evidence.md` (NEW evidence doc, authored by `/dev-story` worker)
  Optionally: a small `assets/` chrome glyph for the AC5 marker if the implementing worker chooses a non-source-only strategy (placeholder-class asset only; `PAW-TD-*-a` accept-risk preserved).

- [ ] **AC24 -- No accept-risk closure claimed**: GIVEN the commit message and any evidence document, WHEN inspected, THEN they explicitly do NOT claim closure of `S8-QA-001-W1`, `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a`, `TQ-S12-C1..C7`, `S11-HUD-TIMER-EYEBALL-VISUAL-001`, any AUDIT-1076-* finding, any SOURCE-1077-* finding, AUDIT-1131-01 (placement cell-index translation; separate Lane A), any 24 PROMPT 1022 audit finding, the PROMPT 761 `Polish->Release` `FAIL`, the Sprint 12 story 019 underlying drag-runtime bug, or any other accept-risk disposition outside AUDIT-1131-02.

- [ ] **AC25 -- Cargo resource policy applied for every Cargo command (future worker)**: future implementation MUST set the Cargo resource policy env vars (`CARGO_TARGET_DIR=D:\_DEV\cargo-target\ccgs-msvc`, `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_PROFILE_TEST_DEBUG=0`, `CARGO_INCREMENTAL=0`, `RUSTFLAGS='-C debuginfo=0 -C link-arg=/DEBUG:NONE'`) before every `cargo check` / `cargo test` invocation on Windows / MSVC. Story authoring (PROMPT 1137) does NOT invoke Cargo.

---

## Implementation Notes

### Owned files (likely change set for future `/dev-story` worker)

| Path | Expected change |
|------|-----------------|
| `server/src/feature/auction/system.rs` (settle path, `try_settle_auction` after `outbox.push_settled` at ~line 714 / 728) | Add the AC10 tracing log line. No behaviour change; trace-only. |
| `client/src/ui/shop_auction/mod.rs` or post-modsplit equivalent (`shop_auction/auction_settlement*.rs`) | Add the AC4 "Auction won" affordance entity + lifecycle systems. Reuse existing settlement-transition timer / surface from story-007 lineage. |
| `client/src/ui/shop_auction/mod.rs` (loser-side path) | Verify / extend AC7 loser settlement toast. May be no-op if story-007 already covers it. |
| `client/src/ui/hand/mod.rs` or post-modsplit equivalent | Add the AC5 newly-acquired card marker (glow / pulse / chevron). |
| `client/src/presentation/qa_snapshot*.rs` | Add AC11 `auction_won_pending` block to snapshot serialiser; add collector system. |
| `client/src/asset_wiring.rs` (optional) | If a placeholder PNG / asset is added for the AC5 marker, register it here. |
| `assets/ui/` or `assets/art/ui/auction/` (optional NEW; placeholder only) | Placeholder PNG for AC5 if a chrome-image marker strategy is chosen. Document as placeholder in commit message; `PAW-TD-*-a` preserved. |
| `tests/integration/auction/auction_won_card_disposition_test.rs` (NEW) | AC12 + AC13 server-side integration coverage. |
| `tests/integration/shop_auction_ui/auction_won_card_disposition_test.rs` (NEW) or extension of `auction_settlement_test.rs` | AC14 + AC15 client-side integration coverage. |
| `tests/unit/qa_snapshot/auction_won_pending_test.rs` (NEW) | AC16 snapshot coverage. |
| `client/Cargo.toml` and / or `server/Cargo.toml` | Additive `[[test]]` block(s) for the new test bins. No profile / feature / dependency edits. |
| `production/qa/evidence/sprint-<active>-auction-won-card-disposition/evidence.md` (NEW; by `/dev-story` worker) | Evidence document; NOT authored by PROMPT 1137. |

### Forbidden files (for future `/dev-story` worker)

- `shared/src/protocol.rs` — no protocol shape change.
- `shared/src/*` outside protocol.rs — no shared-type change.
- `server/src/feature/auction/state.rs`, `snapshot.rs`, `plugin.rs` — no auction state-machine or schedule change.
- `server/src/feature/board/placement.rs` — placement cell-index translation is **AUDIT-1131-01** (distinct surface, Lane A) and is NOT closed by this row.
- `server/src/core/hand_state.rs` or equivalent — no hand-cap or hand-push behaviour change.
- `server/src/feature/round_state_machine/` — no RSM change; `auction_followup_placement_timer_seconds = 12` preserved verbatim.
- `client/src/ui/hand/drag*.rs` (drag pipeline) — TQ-S12-C2 binding preserved.
- `design/gdd/auction-system.md`, `design/gdd/round-state-machine.md`, `design/gdd/hand-ui.md`, `design/gdd/shop-auction-ui.md` — no GDD edit by `/dev-story`; the disposition contract is already present in `design/gdd/auction-system.md` §"Case A" rule 2 and in `design/gdd/round-state-machine.md` §"Rule 9" + RSM-29c.
- `docs/architecture/adr-*.md` — no ADR amendment.
- `assets/ui/ui_bid_button_disabled.png` and any `assets/art/cards/display/*` — `PAW-TD-*-a` preserved verbatim.
- `production/sprint-status.yaml`, `production/sprints/*`, `production/stage.txt`, `production/session-state/*`, `production/qa/qa-plan-*.md`, `production/qa/smoke-*.md`, `production/qa/team-qa-*.md`, `production/gate-checks/*`.
- All other `production/epics/*` story files (no cross-epic edit). The EPIC.md index update at story-authoring time (PROMPT 1137) is **authoring-only**; the `/dev-story` worker does NOT re-edit EPIC.md.
- `.claude/`, `AGENTS.md`, `CLAUDE.md`, `CODEX.md`.
- `Cargo.toml`, `Cargo.lock` (workspace root), `.cargo/`, `.github/`, `Trunk.toml`.

### Cargo resource policy

Per the binding Sprint 15+ QA plan precedent, every `cargo` invocation on Windows / MSVC MUST set the five env vars under AC25.

### Target citations

- Source audit: `reports/PROMPT-1131-game-state-to-visual-contract-deep-audit.md` §3 AUDIT-1131-02 (P0), §5 New Findings (AUDIT-1131-02 line), §6 Observability gap (3), §7 Lane B1 + B2 + Lane D3, §9 Sprint 18 candidate Must Have row 2.
- Companion audit: `reports/PROMPT-1126-current-user-test-snapshot-log-master-qa-audit.md` (re-snapshot of the same 2026-05-18 batch; informational).
- Predecessors (existing closed shop-auction-ui stories on this surface): story 005 `auction-bid-buttons-affordability-and-inflight`, story 006 `auction-accepted-rejected-feedback`, story 007 `auction-settlement-and-shop-transition` (Ready). Story 020 is the natural follow-up to story 007.
- Server settle path: `server/src/feature/auction/system.rs` ~`:670-823` (`try_settle_auction`, `settle_winner_economy`, `award_auction_card`).
- Protocol shapes: `shared/src/protocol.rs:265-277` (`CardSource`), `:525-529` (`S2CCardAcquired`), `:581-585` (`S2CAuctionSettled`).
- RSM timer: `design/gdd/round-state-machine.md` §"Rule 9" + RSM-29c (`auction_followup_placement_timer_seconds = 12`).
- GDD disposition: `design/gdd/auction-system.md` §"Case A — Current leader exists" rule 2 (hand-grant + unicast `S2CCardAcquired`).
- Cross-GDD interactions: `design/gdd/hand-ui.md` §"Interactions" (Hand UI receives `S2CCardAcquired` and adds card to fan).

---

## Worker Contract (for future `/dev-story`)

The future `/dev-story` worker MUST:

1. Run `git checkout` against Sprint 18 activation HEAD (if Sprint 18 is activated by then) on a fresh worktree (suggested slug `work/s18-auction-won-card-disposition`). If Sprint 18 is not activated, the worker MUST NOT proceed; this row is a Sprint 18 Must Have candidate and depends on activation.
2. Read this story file end-to-end before any code change.
3. Re-verify the audit-time module shape by reading the current `client/src/ui/shop_auction/`, `client/src/ui/hand/`, and `client/src/presentation/` directories. Post-modsplit layouts may differ from PROMPT 1137 authoring; the AC list does not bind to specific file paths.
4. Activate `liv-bevy-018` skill before any `.rs` edit. `liv-bevy-lightyear` activation is NOT required by this row (no Lightyear protocol or channel change).
5. Pick the AC4 affordance copy + chrome strategy (e.g. settlement-transition toast extension vs new banner entity) and justify in the commit message.
6. Pick the AC5 marker strategy (glow / pulse / chevron / placeholder asset) and justify.
7. Pick the AC11 snapshot block exact JSON key naming and justify (consistent with the existing snapshot field conventions on `origin/main` at activation).
8. Set the Cargo resource policy env vars per AC25 before every `cargo check` / `cargo test` invocation.
9. Run **narrowest BLOCKING command set**:
   - `cargo check -p server` (AC10 trace-only addition; AC1-3 + AC8 + AC13 server-side regression)
   - `cargo check -p client` (AC4 + AC5 + AC7 + AC11 + AC20)
   - `cargo test --test <auction_won_card_disposition_test>` (or equivalent — AC12 + AC13)
   - `cargo test --test <shop_auction_ui_auction_won_card_disposition_test>` (or extension — AC14 + AC15)
   - `cargo test --test <qa_snapshot_auction_won_pending_test>` (or equivalent — AC16)
   - Adjacent regression sweep: `cargo test -p server --test auction_*` + `cargo test -p client --test shop_auction_*` + `cargo test -p client --test hand_ui_*` (transitive impact)
   - Full workspace test is **NOT** run inside this `/dev-story`.
10. Push the worker branch (never `main`).
11. Stop. Closure paperwork is a later prompt's scope.

The worker MUST NOT:

- Modify `shared/src/protocol.rs` or any `shared/` source.
- Modify `server/src/feature/auction/state.rs`, `snapshot.rs`, `plugin.rs`.
- Modify `server/src/feature/round_state_machine/`.
- Modify `server/src/feature/board/placement.rs` (AUDIT-1131-01 surface — distinct row).
- Modify any GDD or ADR.
- Modify any `production/sprint-status.yaml`, `production/sprints/`, `production/stage.txt`, `production/session-state/`, `production/qa/` artifact.
- Run `/story-done`, `/smoke-check`, `/team-qa`, `/gate-check`, `/release-check`, or `/qa-plan` on this story.
- Run the full workspace `cargo test --workspace` invocation (targeted bins only).
- Run `trunk` or any CI command.
- Push to `main`.
- Claim closure of `A11Y-ST-12`, `QA-COND-0005`, `QA-COND-0006`, `PAW-TD-*-a`, `TQ-S12-C1..C7`, `S8-QA-001-W1`, `S11-HUD-TIMER-EYEBALL-VISUAL-001`, any AUDIT-1076-* finding, any SOURCE-1077-* finding outside the implicit unblock of AUDIT-1131-02, AUDIT-1131-01 (distinct surface), any 24 PROMPT 1022 finding, or any other accept-risk disposition.
- Claim release-readiness, accessibility-completion, playtest-validation, two-client GAME_OVER closure, final-art completion, or stage advance.

### Build gate scope (parallel-agent isolation)

The build gate for this story MUST be scoped to files this worker owns under `server/src/feature/auction/system.rs` (trace-only line) + `client/src/ui/shop_auction/` + `client/src/ui/hand/` + `client/src/presentation/qa_snapshot*.rs` + the new test bins. The worker MUST NOT block on workspace-wide compilation errors introduced by other in-flight Sprint 18 workers' branches.

### Relay / reporting expectation for future workers

Final status line:

```
N: S18-AUCTION-WON-CARD-DISPOSITION-001: STATUS
```

where `N` is the prompt number that ran `/dev-story`.

---

## Dependencies and Parallelism

### Prerequisites

- Sprint 18 activation (does not exist at PROMPT 1137 authoring time; activation is a separate prompt).
- `origin/main` baseline at activation HEAD — this row is independent of the other Sprint 18 candidate rows named in PROMPT 1131 §7.

### Parallelism summary

| Sibling Sprint 18 candidate row (per PROMPT 1131 §7) | Parallel-safe? | Notes |
|---|---|---|
| Lane A2 `S18-CLIENT-PLACEMENT-PERSPECTIVE-COORD-FIX` (AUDIT-1131-01, P0) | **PARTIAL** | both touch the post-PLACEMENT flow but on disjoint files: A2 edits the client `C2SSubmitPlacement` serialisation under `client/src/ui/hand/*`, this row edits `client/src/ui/shop_auction/` + `client/src/ui/hand/` newly-acquired marker. File overlap is on `client/src/ui/hand/` — serialise on `hand/mod.rs` if both touch the same module. AC overlap: AC14 client integration test simulates a successful drag-drop; A2 fixes the drag-drop coord; landing A2 first removes the AUDIT-1131-01 noise from AC14 test setup. **Recommend landing A2 first, then this row.** |
| Lane C1 `S18-UI-HUD-OPP-MANA-CLEANUP-PHASE-2` | **YES** | disjoint (HUD only). |
| Lane C2 `S18-UI-HUD-OPP-CLASS-LABEL` | **YES** | disjoint (HUD + asset wiring). |
| Lane C3 `S18-UI-PHASE-CHIP-DISAMBIGUATE` | **YES** | disjoint (HUD). |
| Lane C4 `S18-UI-AUCTION-FEATURED-CARD-ART-PLACEHOLDER-STRIP` | **PARTIAL** | both edit `client/src/ui/shop_auction/`. Serialise on `shop_auction/mod.rs` if both touch it; the featured-card art surface is distinct from the settlement / won-card affordance surface, so AC overlap is minimal. |
| Lane D1 `S18-OBS-SNAPSHOT-EXTRA-FIELDS` | **PARTIAL** | both edit `client/src/presentation/qa_snapshot*.rs`. Serialise on the snapshot serialiser. AC11 here adds `auction_won_pending`; D1 adds `auction_state`, `placement_state`, `current_phase.timer_remaining_ms`, `opp_class`. The two sets are disjoint; serialise the merge but not the design. |
| Lane D2 `S18-OBS-PLACEMENT-DRAG-EVENT-LOG` | **PARTIAL** | both edit `client/src/ui/hand/*`. Serialise on `hand/mod.rs`. |
| Lane E1 `S18-UI-HAND-INSPECT-FEEDBACK` | **PARTIAL** | both edit `client/src/ui/hand/*`. Serialise on `hand/mod.rs`. |

Recommended Sprint 18 sequencing (if all rows are activated together): **A2 (placement coord fix) → this row 020 → D1 (snapshot extra fields)**, with C1 / C2 / C3 / C4 running in parallel on disjoint surfaces.

---

## Conditions Carried Forward Unchanged

- Sprint 17 disposition `active` (UNCHANGED).
- Sprint 17 stage `Polish` (UNCHANGED; `production/stage.txt` NOT modified).
- PROMPT 761 Polish->Release gate-check `FAIL` preserved; **NO retry** in scope for this row.
- `S8-QA-001-W1` OPEN preserved.
- `QA-COND-0005` + `QA-COND-0006` accepted-risk preserved.
- `PAW-TD-*-a` placeholder-art accept-risk preserved.
- `TQ-S12-C1..C7` preserved verbatim, including `TQ-S12-C7` AppCompat informational condition and the `TQ-S12-C2` binding on drag-runtime retest.
- `S11-HUD-TIMER-EYEBALL-VISUAL-001` human-operator-blocked carry preserved; NOT closed by this row.
- 24 PROMPT 1022 audit findings preserved as report-only; NOT closed by this row.
- All AUDIT-1076-* findings preserved as open / report-only.
- All SOURCE-1077-* findings preserved (closed set: 01/02/03/04 by PROMPT 1113-1117; 06 by PROMPT 1102-1110; 08/09/16 by PROMPT 1122-1124; 10 by PROMPT 1116-1121; deferred: 05 / 07 / 11 / 12 / 13 / 14 / 15).
- AUDIT-1131-01 (placement cell-index translation, P0) NOT closed by this row; remains the Sprint 18 Lane A target.
- AUDIT-1131-03 / 04 / 05 / 06 / 07 / 11-17 NOT closed by this row.

---

## Explicitly NOT Claimed by this Story or its `/dev-story` Worker

- Closure of any AUDIT-1131-* finding outside AUDIT-1131-02 (the AC10 + AC11 observability sub-tasks close the Lane D3 / §6 row 3 observability gap that is part of the AUDIT-1131-02 scope; no other audit row is closed).
- Closure of AUDIT-1131-01 (placement cell-index translation).
- Closure of any AUDIT-1076-* finding.
- Closure of any SOURCE-1077-* finding.
- Closure of any of the 24 PROMPT 1022 audit findings.
- Sprint 17 close-out.
- Sprint 18 activation.
- Real-art replacement of any auction or hand asset.
- Public release readiness; release-candidate readiness; full game completion.
- Broad / Standard-tier accessibility completion; playtest / fun-hypothesis validation; full playable-client manual QA; two-client GAME_OVER closure; final-art completion; Polish->Release gate-check retry; stage advance.
- Discharge of PROMPT 1112 AC3 hand reserve-strip carry / AUDIT-1076-17 (semantically distinct surface).
- Sprint 17 smoke / team-QA / gate-check / release-check execution.
- Rewrite of the placement drag-runtime pipeline (TQ-S12-C2 binding preserved).

---

`020: S18-AUCTION-WON-CARD-DISPOSITION-001: DRAFT`
