# PROMPT 1839 — QA-SNAPSHOT-OBSERVABILITY-GAP-REFRESH

- Source-of-truth base: `origin/main@71484998`
- Worktree: `D:\tmp\wt-1839-qa-obs-gap`
- Audit scope: read-only, no source file modifications.

---

## 1. Snapshot System Inventory

### 1.1 Client-side snapshot (`client/src/presentation/qa_snapshot.rs`)

Top-level `QASnapshotData` struct (line 391) — written on F9 or button click:

| Struct / Field | File:Line | Description |
|---|---|---|
| `QASnapshotData` | qa_snapshot.rs:391 | Root document |
| `snapshot_id`, `counter`, `unix_millis`, `snapshot_utc_iso` | :394-397 | Identity / timestamp |
| `evidence_layers` | :401 | Trust-boundary labels |
| `screenshot` (`ScreenshotInfo`) | :403 | PNG metadata: path, status, timestamps |
| `client_state` | :404 | `ClientState` enum as string |
| `current_phase` (`PhaseInfo`) | :405 | Phase, round, `timer_remaining_ms` |
| `phase_view` (`PhaseViewInfo`) | :406 | Phase, round, `timer_duration_ms` |
| `session_identity` (`SessionIdentityInfo`) | :407 | player_id, session_id, token flag |
| `window` (`WindowInfo`) | :408 | width, height, scale_factor |
| `ui_counts` (`UiCounts`) | :409 | Per-surface visible/spawned entity counts |
| `extras` (`ExtrasSnapshot`) | :415 | Large observability bag (see 1.2) |
| `layout` (`LayoutSnapshot`) | :424 | Viewport, surface bounds, button affordances, collisions |
| `ui_text_markers` (`Vec<UiTextMarkerSnapshot>`) | :429 | Rendered text with bounds, role token |
| `placement_state` (`PlacementStateSnapshot`) | :444 | Drag target kind, staged count, can_submit, disclosure_step, rejection |
| `auction_state` (`AuctionStateSnapshot`) | :447 | Panel state, card_id, prices, leader, timer, in-flight bid, local gold |
| `auction_won_pending` (`Option<AuctionWonPendingSnapshot>`) | :459 | Winner's pending placement block (omitted when absent) |
| `board_targeting` (`BoardTargetingSnapshot`) | :469 | Envelope, active targeting, overlay entity counts, overlap flags |
| `warnings` | :471 | Missing-resource diagnostics |

### 1.2 `ExtrasSnapshot` sub-fields (`ExtrasSnapshot`, line 1127)

| Field | Struct | Key contents |
|---|---|---|
| `frame_count` | `Option<u64>` | Frames since app start |
| `players` | `PlayerIdsSnapshot` | local_player_id, opponent_player_id, source |
| `timers` | `TimersSnapshot` | phase_timer, placement_timer, auction_timer, shop_timer (each with remaining/duration ms) |
| `resources` | `Option<PlayerResourcesSnapshot>` | gold, current_mana, reserve_mana, mana_cap, local_gold_view |
| `hand` | `Option<HandSnapshot>` | mode, disclosure_step, hand_count, cards, pending_placements, staged_count |
| `drag` | `DragSnapshot` | placement_drag (active, card_id, owner, target_kind, cursor_world), ghost_unstage |
| `shop_auction` | `Option<ShopAuctionExtrasSnapshot>` | ui_mode, draft_initial, shop (slots), auction, settlement, toast, auction_won_pending_state |
| `hud` | `Option<HudExtrasSnapshot>` | mode, local_class, opponent_class, player_ids |
| `board` | `BoardSnapshot` | local_player_id, units (per-entity: unit_id, lane, cell, card_id, stats, visibility, world_pos), objectives (per-entity: owner, lane, hp, art_kind) |
| `board_render_state` | `Option<String>` | `BoardRenderState` enum token |
| `session_settings` | `Option<SessionSettingsSnapshot>` | placement_timer_multiplier |
| `objective_identities` | `Vec<ObjectiveIdentitySnapshot>` | lane, is_fake |
| `opponent_connection` | `Option<OpponentConnectionSnapshot>` | disconnected_player_id, grace_remaining_ms |
| `session_lifecycle` | `Option<SessionLifecycleSnapshot>` | cancellation_reason |
| `outbound_intents` | `OutboundIntentsSnapshot` | per-message-type pending buffer counts |
| `input` | `InputDiagnosticsSnapshot` | pointer_screen/world, hovered_entity, target_cell, reject_reason, active_drag_state |
| `debug_grid` | `DebugGridSnapshot` | enabled, line_count, z_layer, blocks_input |
| `placement_lifecycle` | `PlacementLifecycleSnapshot` | submitted, accepted, rejected, awaiting_ack, accepted_source, last_rejection_reason, state |
| `connection_lost` | `ConnectionLostDiagnosticsSnapshot` | visible, cause, grace_remaining_ms, blocking_input |
| `resolution_phase` | `ResolutionPhaseSnapshot` | active, event_count, event_summary, per_lane_objective, gold_awards, unit_deaths, unit_removals, game_over, game_over_reason |

### 1.3 `ShopSlotDiagnosticSnapshot` (shop slots, line 1358)

Per shop slot: `slot_index`, `entity`, `card_id`, `name`, `cost`, `atk`, `hp`, `state`, `button_state`, `placeholder_visible`, `visible`.

### 1.4 Server-side bot snapshot (`server/src/feature/bot/qa_snapshot.rs`)

`BotQaSnapshot` (line 236) — written on phase transitions and periodic 10s ticks:

| Field | Struct | Description |
|---|---|---|
| `schema_version`, `trigger`, `timestamp_ms`, `sequence` | primitives | Metadata |
| `round` | `Option<RoundSnapshot>` | phase, round_number, draft_ready_players, submissions_received, disconnect_trackers, timers_ms (all 6 server timers) |
| `session` | `Option<SessionSnapshot>` | mode, player_count, players (player_id, team, class, is_bot), placement_timer_multiplier |
| `auction` | `Option<AuctionSnapshot>` | phase label, card_id, starting/current price, current_leader, timer_remaining_ms, live_bidding_deadline_elapsed_ms |
| `economies` | `Vec<EconomySnapshot>` | per player: gold, current_mana, reserve_mana, mana_cap, reserved_gold |
| `hands` | `Vec<HandSnapshot>` | per player: size, card_id list |
| `board` | `Option<BoardSnapshot>` | minion/trap/structure/field counts, per-player occupied lanes (lane list only) |
| `objectives` | `Vec<ObjectiveSnapshot>` | per (player, lane): hp, destroyed |
| `bots` | `Vec<BotStateSnapshot>` | per bot: difficulty, rng_seed, rng_word_counter, last_decision_at_ms, class_choice, next_decision_at_ms, failsafe_deadline_ms |
| `decision_log_tail` | `Vec<DecisionEntrySnapshot>` | Tail of decision log (cap=64): round, phase, bot_player_id, decision kind + details |
| `decision_log_total` | `usize` | Full count |

### 1.5 Bot debug overlay state (`client/src/presentation/debug_bot_overlay.rs`)

`DebugBotOverlayState` (line 100): `visible: bool`, `latest: Option<S2CDebugBotStatePush>`, `receive_count: u64`.

The overlay renders `S2CDebugBotStatePush` contents (per-bot: player_id, class_id, gold, mana, submitted, hand len, last_bid_valuation, decision tail). This resource is NOT read by `write_qa_snapshot_system`.

---

## 2. Coverage Table

### Area 1: Phase / Timer

| Sub-field | Fields present | Fields missing | Verdict |
|---|---|---|---|
| Current game phase string | `current_phase.phase`, `phase_view.phase`, `extras.timers.phase_timer.panel_state` | — | COVERED |
| Countdown timer value | `current_phase.timer_remaining_ms`, `extras.timers.phase_timer.remaining_ms` / `elapsed_ms` / `computed_remaining_ms` / `display_text` | — | COVERED |
| Timer active flag | `extras.timers.phase_timer.active` | — | COVERED |
| Phase duration | `extras.timers.phase_timer.duration_ms` | — | COVERED |

**Verdict: COVERED**

---

### Area 2: Shop Offers

| Sub-field | Fields present | Fields missing | Verdict |
|---|---|---|---|
| Slot index | `extras.shop_auction.shop.slots[].slot_index` | — | COVERED |
| Card visible per slot | `extras.shop_auction.shop.slots[].card_id`, `name`, `visible` | — | COVERED |
| Slot state (Available/Empty/Pending…) | `extras.shop_auction.shop.slots[].state` | — | COVERED |
| Card cost/atk/hp | `extras.shop_auction.shop.slots[].cost`, `atk`, `hp` | — | COVERED |
| Footer slot cards | `extras.shop_auction.shop.footer_slots` | — | COVERED |
| Card **class/rarity** in slot | — | No `class_id`, `rarity` per slot snapshot | PARTIAL |
| Shop refresh count | `extras.shop_auction.shop.refresh_count_this_draft`, `refresh_in_flight` | — | COVERED |
| DraftInitial offering | `extras.shop_auction.draft_initial.*` | — | COVERED |

**Verdict: PARTIAL** — card class and rarity are not included in `ShopSlotDiagnosticSnapshot` (qa_snapshot.rs:1358). These are visible to the player (card colour-coded by rarity in the Krosmaga skin) but not currently accessible from the snapshot without querying `ShopAuctionCardCatalog` (which is already available in `ExtrasShopAuctionInputs.catalog` at line 1788).

---

### Area 3: Auction Leader / Status

| Sub-field | Fields present | Fields missing | Verdict |
|---|---|---|---|
| Panel state token | `auction_state.panel_state`, `extras.shop_auction.auction.panel_state` | — | COVERED |
| Card on auction | `auction_state.card_id`, `extras.shop_auction.auction.card_id` | — | COVERED |
| Current leader | `auction_state.current_leader`, `extras.shop_auction.auction.current_leader` | — | COVERED |
| Leader is local player | `auction_state.leader_is_local` | — | COVERED |
| Current/starting bid | `auction_state.current_price`, `starting_price` | — | COVERED |
| Auction timer remaining | `auction_state.timer_remaining_ms` | — | COVERED |
| In-flight bid | `auction_state.local_in_flight_bid_amount` | — | COVERED |
| Local gold / free gold | `auction_state.local_gold.*` | — | COVERED |
| Settlement outcome | `extras.shop_auction.settlement.*` | — | COVERED |
| Auction resolution state | `extras.shop_auction.auction.pending_bid_accepted`, `opponent_bid_gate_satisfied`, `waiting_for_local_gold_after_opponent_bid` | — | COVERED |
| Auctioned card **class/rarity** | — | Not in `auction_state` nor `extras.shop_auction.auction`; requires catalog lookup | PARTIAL |

**Verdict: PARTIAL** — the auctioned card's class and rarity (visible on the card art) are not included. The catalog resource (`ShopAuctionCardCatalog`) is already in scope at `ExtrasShopAuctionInputs.catalog` (qa_snapshot.rs:1788).

---

### Area 4: Placement Drag Target / Accepted Ack / Rejected Recovery

| Sub-field | Fields present | Fields missing | Verdict |
|---|---|---|---|
| Drag active flag | `placement_state.drag_active`, `extras.drag.placement_drag_active` | — | COVERED |
| Drag target kind (BoardCell/LaneWide/…) | `placement_state.drag_target_kind`, `extras.drag.placement_drag_target_kind` | — | COVERED |
| Drag cursor world coords | `extras.drag.placement_drag_cursor_world` | — | COVERED |
| Active target cell (lane, cell) | `extras.input.target_cell`, `board_targeting.active_targeting.endpoint_cell` | — | COVERED |
| Target invalid flag | `board_targeting.active_targeting.endpoint_invalid` | — | COVERED |
| Accepted ACK | `placement_lifecycle.accepted`, `accepted_source`, `awaiting_ack` | Heuristic only; no authoritative server ACK (gap documented PROMPT 1533) | PARTIAL |
| Rejection state | `placement_lifecycle.rejected`, `last_rejection_reason`, `placement_state.last_rejection_state` | — | COVERED |
| Disclosure step | `placement_state.disclosure_step` | — | COVERED |

**Verdict: PARTIAL** — placement acceptance is locally-inferred heuristic (`accepted_source: "local_clearance_heuristic"`). An authoritative `S2CPlacementAck` protocol message was proposed in PROMPT 1533 but not yet implemented.

---

### Area 5: Board Units

| Sub-field | Fields present | Fields missing | Verdict |
|---|---|---|---|
| Units on board (list) | `extras.board.units[]` (up to `MAX_BOARD_ENTITIES_PER_KIND=60`) | — | COVERED |
| Per-unit position (lane, cell) | `extras.board.units[].lane`, `.cell` | — | COVERED |
| Per-unit stats (hp_current, hp_max, atk, mp, ar) | `extras.board.units[].hp_current`, `hp_max`, `atk`, `mp`, `ar` | — | COVERED |
| Per-unit card_id | `extras.board.units[].card_id` | — | COVERED |
| Per-unit visibility | `extras.board.units[].visible` | — | COVERED |
| Per-unit owner | `extras.board.units[].owner_id` | — | COVERED |
| Per-unit render source | `extras.board.units[].render_source` | — | COVERED |
| Per-unit world position | `extras.board.units[].world_position` | — | COVERED |
| Per-unit **class** | — | No class_id per unit in `BoardUnitSnapshot`; `BoardUnitCard` only gives `card_id` | PARTIAL |
| Standing objectives HP | `extras.board.objectives[].hp_current`, `hp_max` | — | COVERED |
| Objective art kind | `extras.board.objectives[].art_kind` | — | COVERED |
| Objective fake flag | `extras.extras.objective_identities[].is_fake` | — | COVERED |

**Verdict: PARTIAL** — board unit class is not included. The class can only be derived by cross-referencing `card_id` with the card catalog, which is not done in the snapshot. For a visual UI audit this gap is notable because class determines the unit sprite column.

---

### Area 6: Resolution / Gameover

| Sub-field | Fields present | Fields missing | Verdict |
|---|---|---|---|
| Resolution active | `extras.resolution_phase.active` | — | COVERED |
| Event summary (per-variant counts) | `extras.resolution_phase.event_summary.*` | — | COVERED |
| Per-lane objective damage / hp_after / destroyed | `extras.resolution_phase.per_lane_objective[]` | — | COVERED |
| Gold awards | `extras.resolution_phase.gold_awards[]` | — | COVERED |
| Unit deaths / removals | `extras.resolution_phase.unit_deaths[]`, `unit_removals[]` | — | COVERED |
| Game-over flag | `extras.resolution_phase.game_over`, `game_over_reason` | — | COVERED |
| AnimQueue playback index | `extras.resolution_phase.anim_queue_current_index` | — | COVERED |
| **Result screen visible** | `ui_counts.result_screen_visible` | — | COVERED |
| **S2CGameOver payload** (loser, round, reason) | — | `ResultScreenViewState.cached_result` (S2CGameOver) is NOT read by the snapshot system | MISSING |
| **S2CGamSnapshot** (winner/loser ids as known to result screen) | — | `ResultScreenViewState.cached_snapshot` is NOT read | MISSING |
| **Local outcome** (win/loss/draw from local player's perspective) | — | The local player's final outcome is not projected; `S2CGameOver.loser` is only accessible via the unread `ResultScreenViewState` resource | MISSING |
| AnimQueue round (mid-playback) | — | Round number is lost after `consume_pending_resolution_script_system` flattens the script into `AnimGroup`s (documented PROMPT 1586 out-of-scope) | PARTIAL |
| Resolution recovery signal | — | `PendingResolutionScript::recovery_requested` is private (documented PROMPT 1586 out-of-scope) | PARTIAL |

**Verdict: PARTIAL** — the result screen's *content* (who won, `S2CGameOver.loser`, `GameOverReason`, `S2CGameSnapshot` final-score snapshot) is not captured in the QA snapshot. `ResultScreenViewState` (result_screen.rs:85) is a `Resource` with fields `cached_result: Option<S2CGameOver>`, `cached_snapshot: Option<S2CGameSnapshot>`, `visible: bool`. None of these are read in `write_qa_snapshot_system` or any `ExtrasInputs` `SystemParam`.

---

### Area 7: Bot / Autoplay Debug Overlay

| Sub-field | Fields present | Fields missing | Verdict |
|---|---|---|---|
| Overlay visibility flag | — | `DebugBotOverlayState.visible` is NOT read by QA snapshot | MISSING |
| Latest S2CDebugBotStatePush | — | `DebugBotOverlayState.latest` (bot states, decision tails) is NOT read | MISSING |
| Receive count | — | `DebugBotOverlayState.receive_count` is NOT read | MISSING |
| Bot decision state (via server snapshot) | `server: bots[].last_decision_at_ms`, `next_decision_at_ms`, `rng_seed`, `class_choice` | Client-side overlay state is separate from server snapshot | PARTIAL |
| Bot autoplay step info | — | Client-side `AutoplayState` / recipe step tracking not in snapshot | MISSING |
| Debug overlay config (enabled/disabled) | — | `DebugBotOverlayConfig.enabled` not captured | MISSING |

**Verdict: MISSING** — the client-side bot debug overlay state (`DebugBotOverlayState`, debug_bot_overlay.rs:100) is entirely absent from the QA snapshot. A screenshot shows the overlay if it is visible, but `snapshot.json` carries no corroborating fields. The server-side snapshot has good bot decision data, but the client-side overlay state (which `F8` toggles) is not bridged to `snapshot.json`.

---

## 3. Gap List

1. **GAP-1 — Result screen outcome fields missing**: `ResultScreenViewState.cached_result` (`S2CGameOver` with `loser`, `round`, `reason`) and `cached_snapshot` (`S2CGameSnapshot` with per-objective final states) are not read by the snapshot system. A screenshot of the result screen cannot be verified against structured data.
   - Source: `client/src/presentation/result_screen.rs:85-90`
   - Missing struct fields: `loser: Option<PlayerId>`, `round: u32`, `reason: GameOverReason`, `visible: bool`, `snapshot_game_over_seen: bool`

2. **GAP-2 — Local win/loss/draw outcome not projected**: The local player's outcome (win, loss, draw) cannot be derived from existing snapshot fields without combining `session_identity.player_id` with the missing `S2CGameOver.loser`. No `local_outcome` field exists.

3. **GAP-3 — Bot debug overlay state absent from snapshot**: `DebugBotOverlayState` (`visible`, `latest: Option<S2CDebugBotStatePush>`, `receive_count`) is never read. Overlay visibility and the last received bot state push are not in `snapshot.json`.
   - Source: `client/src/presentation/debug_bot_overlay.rs:100-108`

4. **GAP-4 — Card class/rarity missing from shop slot diagnostics**: `ShopSlotDiagnosticSnapshot` (qa_snapshot.rs:1358) lacks `class_id` and `rarity`. The `ShopAuctionCardCatalog` resource is already in scope at `ExtrasShopAuctionInputs.catalog` (qa_snapshot.rs:1788) and could supply these fields by lookup.

5. **GAP-5 — Auctioned card class/rarity not in auction state snapshot**: `AuctionStateSnapshot` (qa_snapshot.rs:634) and `AuctionPanelSnapshot` (qa_snapshot.rs:1373) do not include the card's class or rarity, only `card_id`. Same catalog lookup applies.

6. **GAP-6 — Board unit class not in unit snapshot**: `BoardUnitSnapshot` (qa_snapshot.rs:1434) lacks a `class_id` field. The unit class determines the sprite and is visible on-screen, but is not derivable from the snapshot without catalog cross-reference.

7. **GAP-7 — Placement ACK is heuristic-only (no authoritative server signal)**: `placement_lifecycle.accepted_source` is always `"local_clearance_heuristic"` because no `S2CPlacementAck` exists in the protocol. Proposed in PROMPT 1533, not yet shipped.

8. **GAP-8 — AnimQueue round number lost mid-playback**: `extras.resolution_phase.round` becomes `None` once `consume_pending_resolution_script_system` flattens the script. The round number is not propagated into `AnimGroup`. Documented in PROMPT 1586 out-of-scope.

9. **GAP-9 — Resolution recovery signal private**: `PendingResolutionScript::recovery_requested` and `ResolutionRevealWait::recovery_requested` have no public accessor. Documented in PROMPT 1586 out-of-scope.

10. **GAP-10 — Per-card-art aspect-ratio diagnostics absent (Q-06)**: Image nodes for card art carry no `CardArtDiagnostic` marker. The `aspect_ratio_src` / `aspect_ratio_rendered` fields proposed in PROMPT 1533 are still unimplemented.

11. **GAP-11 — Autoplay recipe step state not in snapshot**: The client-side autoplay step tracking (`client/src/autoplay.rs`) is not read by the snapshot system. When an autoplay run is in progress, the snapshot has no field describing which recipe step is active or what the last autoplay action was.
    - Source: `client/src/autoplay.rs`

---

## 4. Proposed Implementation Lanes

### Lane A — Result Screen Outcome Fields (GAP-1, GAP-2)

**Owner**: `client/src/presentation/qa_snapshot.rs` only.

**What to add**:
- New `ResultScreenSnapshot` struct: `visible: bool`, `loser_id: Option<String>`, `loser_is_local: Option<bool>`, `round: Option<u32>`, `reason: Option<String>`, `snapshot_game_over_seen: bool`.
- New `ExtrasResultScreenInputs` SystemParam (or add `Option<Res<ResultScreenViewState>>` to `ExtrasSessionInputs` — currently at 3 fields, has room):
  ```rust
  pub result_screen_view: Option<Res<'w, crate::presentation::result_screen::ResultScreenViewState>>,
  ```
- Projection in `ExtrasInputs::snapshot_with_warnings`: read `ResultScreenViewState`, derive `loser_is_local` by comparing `loser_id` with `session_identity.player_id`.
- Add `result_screen: Option<ResultScreenSnapshot>` to `ExtrasSnapshot`.
- No writes to `result_screen.rs` or `state/mod.rs` required — pure read.

**No conflicts**: `ExtrasSessionInputs` owns its own `SystemParam` group; adding a field there does not touch any UI module.

---

### Lane B — Bot Debug Overlay State (GAP-3, GAP-11 partial)

**Owner**: `client/src/presentation/qa_snapshot.rs` only.

**What to add**:
- New `BotDebugOverlaySnapshot` struct: `enabled: bool`, `visible: bool`, `receive_count: u64`, `has_latest_payload: bool`, plus a summary sub-struct `BotDebugPayloadSummary { bot_count: usize, decision_log_total: u64, assembled_at_ms: u64 }` (not the full payload text, to keep JSON size bounded).
- Add `Option<Res<'w, crate::presentation::debug_bot_overlay::DebugBotOverlayState>>` and `Option<Res<'w, crate::presentation::debug_bot_overlay::DebugBotOverlayConfig>>` to one of the existing nested `SystemParam` groups (e.g., `ExtrasSessionInputs`, currently 3 fields → 5 fields — well within limit).
- Add `bot_debug_overlay: Option<BotDebugOverlaySnapshot>` to `ExtrasSnapshot`.

**No conflicts**: debug_bot_overlay resources are public and read-only here; no UI writes.

---

### Lane C — Card Class/Rarity in Shop/Auction/Board Snapshots (GAP-4, GAP-5, GAP-6)

**Owner**: `client/src/presentation/qa_snapshot.rs` only.

**What to add**:
- `ShopAuctionCardCatalog` is already in `ExtrasShopAuctionInputs.catalog` (qa_snapshot.rs:1788). Add a catalog lookup helper:
  ```rust
  fn catalog_class(catalog: &ShopAuctionCardCatalog, card_id: u32) -> Option<String>
  fn catalog_rarity(catalog: &ShopAuctionCardCatalog, card_id: u32) -> Option<String>
  ```
- Add `class_id: Option<String>`, `rarity: Option<String>` to `ShopSlotDiagnosticSnapshot` (qa_snapshot.rs:1358), `AuctionStateSnapshot` (qa_snapshot.rs:634), and `AuctionPanelSnapshot` (qa_snapshot.rs:1373).
- For `BoardUnitSnapshot` (qa_snapshot.rs:1434): add `class_id: Option<String>`. Board unit has a `BoardUnitCard` component that carries `card_id`; the catalog lookup can happen inside `build_board_snapshot`.
- The catalog resource may be absent outside shop/draft phases — all new fields should be `Option<String>` and resolve to `None` when the catalog is absent.

**No conflicts**: Catalog is owned by the shop_auction plugin and is read-only here.

---

### Lane D — Authoritative Placement ACK (GAP-7)

**Owner**: Server protocol + client state resources (multi-file, larger scope).

**What to add** (out of scope for a pure qa_snapshot prompt):
- Server: `S2CPlacementAck { round: u32, accepted_ids: Vec<u32>, rejected: Vec<(u32, RejectReason)> }` in `shared/src/protocol.rs`.
- Client: resource `PlacementLifecycle { accepted_ids: Vec<u32>, rejected_pairs: Vec<(u32, RejectReason)> }` inserted by the receiver.
- `placement_lifecycle.accepted_source` can then advance from `local_clearance_heuristic` to `server_ack`.
- The qa_snapshot side requires only reading the new resource — one extra field on `ExtrasHandInputs`.

**No conflicts per lane** if this is a separate prompt owning `shared/`, server placement handler, and `client/src/ui/hand/`.

---

### Lane E — AnimQueue Round Propagation (GAP-8)

**Owner**: `client/src/card_animations/queue.rs` + `client/src/presentation/board_rendering.rs` (multi-file).

**What to add**:
- Add `round: Option<u32>` to `AnimGroup` (or a companion `AnimQueueMeta` resource set at consume time).
- `qa_snapshot.rs` already reads `AnimQueue`; the field would auto-populate `extras.resolution_phase.round` during mid-playback.

**No conflicts** if scoped to `card_animations/` and board_rendering consume path.

---

### Lane F — Autoplay Step State (GAP-11)

**Owner**: `client/src/presentation/qa_snapshot.rs` + `client/src/autoplay.rs` (read only from qa_snapshot).

**What to add**:
- Audit `client/src/autoplay.rs` for a public resource tracking current recipe step and last autoplay action.
- Add `autoplay: Option<AutoplayStepSnapshot>` to `ExtrasSnapshot` with fields like `active: bool`, `current_step: Option<String>`, `recipe_id: Option<String>`, `last_action: Option<String>`.
- Read-only access from `ExtrasInputs` — no writes to autoplay code.

---

## 5. Summary Coverage Table

| UI / Game Area | Verdict | Key gaps |
|---|---|---|
| Phase / timer | COVERED | — |
| Shop offers | PARTIAL | Card class/rarity per slot (GAP-4) |
| Auction leader/status | PARTIAL | Auctioned card class/rarity (GAP-5) |
| Placement drag target/accepted ack/rejected recovery | PARTIAL | ACK is heuristic-only (GAP-7) |
| Board units | PARTIAL | Unit class_id missing (GAP-6) |
| Resolution / gameover | PARTIAL | Result screen outcome fields absent (GAP-1/2); round lost mid-playback (GAP-8); recovery signal private (GAP-9) |
| Bot/autoplay debug overlay | MISSING | Overlay state entirely absent (GAP-3); autoplay step not captured (GAP-11) |

---

1839: QA-SNAPSHOT-OBSERVABILITY-GAP-REFRESH: SHIPPED
