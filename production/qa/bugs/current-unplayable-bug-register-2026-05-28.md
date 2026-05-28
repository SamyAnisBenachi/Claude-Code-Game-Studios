# Current Unplayable Bug Register

Date: 2026-05-28
Source of truth when updated: `origin/main@e1a61376`

This register consolidates bugs found by the current forensic audit wave so they
are not lost in worker reports or chat history.

Primary sources:

- `reports/PROMPT-2024-forensic-evidence-inventory-and-run-selection.md`
- `reports/PROMPT-2025-snapshot-log-gamestate-correlation-audit.md`
- `reports/PROMPT-2026-visible-screen-screenshot-visual-bug-audit.md`
- `reports/PROMPT-2027-autoplay-input-click-target-forensic-audit.md`
- `reports/PROMPT-2028-player-flow-unplayable-bug-classification.md`
- `reports/PROMPT-2029-qa-evidence-tools-truthfulness-audit.md`
- User live-play report in orchestrator chat on 2026-05-28
- `reports/PROMPT-2030-client-phase-sync-p0-repair.md`
- `reports/PROMPT-2031-server-draft-hand-awarding-p0-repair.md`
- `reports/PROMPT-2032-bot-placement-failsafe-spinloop-p0-repair.md`
- `reports/PROMPT-2033-server-board-gameover-vacuous-flow-p0-repair.md`
- `reports/PROMPT-2034-user-live-ui-ux-bug-ledger-backfill.md`
- `reports/PROMPT-2035-live-ui-visual-audit-redo.md`
- `reports/PROMPT-2036-placement-dragdrop-legal-cell-feedback-repair.md`
- `reports/PROMPT-2038-card-asset-shop-placeholder-binding-repair.md`
- `reports/PROMPT-2039-board-unit-combat-presentation-audit-repair-map.md`

## Executive State

The game is currently unplayable end to end.

The most important confirmed failure is that the client never transitions from
Lobby to InSession in the audited autoplay runs. The server advances through a
vacuous two-round match, but the visible client stays in Lobby for every
checkpoint. After PROMPT 2031 and PROMPT 2032, two server/bot-side blockers have
landed: bot draft hand awarding no longer permanently debounces before
`PlayerEconomy` exists, and placement failsafe timing is armed on Placement
entry. These are real fixes, but they do not close the game. PROMPT 2030 was
diagnostic/partial only, and the visible UI remains unplayable until client
phase sync, drag/drop, board presentation, card assets, and fresh evidence all
agree.

## P0 Blockers

| ID | Title | Evidence | Current status |
|---|---|---|---|
| P0-001 | Client never transitions to InSession | All three autoplay driver timelines: `client_state_label: "Lobby"` and `phase_label: "Lobby"` for all ticks | Confirmed |
| P0-002 | No in-game screen ever renders | PROMPT 2026 screenshots: shop, auction, placement, resolution, and post-resolution checkpoints all show Lobby | Confirmed |
| P0-003 | Server advances a match the client does not visually join | PROMPT 2025: server snapshots progress DraftInitial -> Placement -> Resolution -> DraftShop -> Placement -> Resolution -> GameOver while client remains Lobby | Confirmed |
| P0-004 | Bot player has no hand | PROMPT 2025 snapshots from first Placement onward contain hand for player 1 only; bot hand missing | Server-side debounce race fixed by PROMPT 2031; needs fresh post-2031/2032 evidence |
| P0-005 | Draft/card awarding broken | Player 1 keeps one card `[5]`; bot has no hand; DraftInitial and DraftShop do not award usable cards | Bot auto-pick retry fixed by PROMPT 2031; broader visible draft/shop UX still open |
| P0-006 | Bot placement failsafe spin-loop | `empty_placement_failsafe` logged about 16,229 times in a two-round game, roughly 1/ms | Placement phase timing fixed by PROMPT 2032; needs fresh post-2032 evidence |
| P0-007 | No units ever reach board | Board counts stay zero across minions/traps/structures/fields; `per_player_minions` empty | Root-caused by PROMPT 2033 as upstream client phase sync + bot hand/placement cascade; still open until fresh board/unit evidence |
| P0-008 | No combat occurs | Resolution phases run with empty board and all objectives unchanged | Root-caused by PROMPT 2033 as cascade of P0-007; pending post-repair verification |
| P0-009 | GameOver fires vacuously | GameOver after two empty rounds with all 10 objectives at 5/5 HP, none destroyed | Reclassified by PROMPT 2033: normal win-condition path is guarded; observed GameOver is soak max-round cap plus no-board cascade |
| P0-010 | Phase timers are bypassed | DraftInitial, DraftShop, and Resolution transition in milliseconds despite 30-60s configured timers | Resolution portion root-caused by PROMPT 2033 as no-units cascade; broader draft/shop timer behavior still open for post-repair verification |
| P0-011 | No successful human GUI end-to-end flow on record | PROMPT 2028/1883: no verified human flow through room, session, draft/shop, placement, resolution, GameOver | Confirmed coverage blocker |
| P0-012 | Human two-client stale binary protocol panic | PROMPT 1883: stale `client.exe` vs fresh `server.exe` caused Lightyear protocol mismatch before UI | Confirmed operational blocker; rebuild mitigates stale-binary case |
| P0-013 | Client phase sync repair is not complete | PROMPT 2030 shipped diagnostics/regressions only; it identified silent RSM sender drop and missing `C2SCreateBotRoom` in autoplay, but did not close P0-001/P0-003 | Confirmed P0 follow-up |
| P0-014 | Integrated client remains visually untrusted after server fixes | User live report plus PROMPT 2035/2036/2039: drag/drop, card assets, board presentation, and UI anchoring still fail even if server state advances | Confirmed P0 UI/playability follow-up |
| P0-015 | Placement drag preview does not follow the cursor | PROMPT 2034 user live observation 2026-05-28: when picking up a hand card and moving across the board, the visual drag preview does not track the mouse position; player cannot see what they are about to place where | Confirmed by user live play |
| P0-016 | Placed cards never become visible units on the board | PROMPT 2034 user live observation 2026-05-28: even when a card is dropped on a cell and confirm does not reject it, no unit sprite/HUD entity appears on that cell during Placement or Resolution | Confirmed by user live play; consistent with P0-007 |

## P1 Major Gameplay And State Bugs

| ID | Title | Evidence | Current status |
|---|---|---|---|
| P1-001 | Disconnect trackers initialized as disconnected for both players | PROMPT 2024/2025 snapshots show `seconds_since_disconnect` around 30000s from early game | Confirmed |
| P1-002 | Lobby room code never populates | PROMPT 2026 screenshots show `Room: ----` after bot add and lobby confirm checkpoints | Confirmed |
| P1-003 | Lobby player count never updates | PROMPT 2026 screenshots show `Players: 0/1` throughout | Confirmed |
| P1-004 | Class confirmation status never clears | PROMPT 2026 screenshots show `not confirmed` after class-confirmed checkpoint | Confirmed |
| P1-005 | Placement ACK missing; client relies on heuristic | PROMPT 2028 references PROMPT 1937 GAP-7: `S2CPlacementAck` still not shipped | Confirmed design gap |
| P1-006 | Result screen does not project win/loss/draw outcome | PROMPT 2028 references PROMPT 1937 GAP-1/GAP-2 and result-screen gap | Confirmed design/evidence gap |
| P1-007 | Player gold drops without recorded purchase | PROMPT 2025: player 1 gold 5 -> 3 at DraftInitial -> Placement without charge evidence | Open; re-check after PROMPT 2031 fresh run |
| P1-008 | `draft_ready` logs `legal_action_count: null` | PROMPT 2025: DraftInitial and DraftShop bot decisions use null legal count | Open; re-check after PROMPT 2031 fresh run |
| P1-009 | `draft_ready_players` never records ready players | PROMPT 2025 snapshots show empty list despite `draft_ready` decisions | Open; re-check after PROMPT 2031 fresh run |
| P1-010 | `submissions_received` leaks into next round | PROMPT 2025: `[1]` persists into round 2 DraftShop | Fixed by PROMPT 2033; `Resolution -> DraftShop/DraftAuction` now clears stale submissions |
| P1-011 | Bot RNG path not consumed | PROMPT 2025: `rng_word_counter` remains 0 despite thousands of bot decisions | Confirmed |
| P1-012 | Bot decision timestamps/deadlines stale or null | PROMPT 2025: `last_decision_at_ms` stuck; `next_decision_at_ms` and `failsafe_deadline_ms` null during spin-loop | Placement failsafe deadline fixed by PROMPT 2032; remaining timestamp behavior needs fresh evidence |
| P1-013 | Final GameOver snapshot loses session | PROMPT 2025: final GameOver snapshot has `session: null` | Root-caused by PROMPT 2033 as GameOver teardown/snapshot ordering; follow-up observability repair needed |
| P1-014 | `client_exit_code` never observed | PROMPT 2025: all launcher statuses have `client_exit_code: null` while outcome is `ok` | Confirmed |
| P1-015 | Autoplay `outcome: ok` is misleading | Checkpoints can pass while client stays Lobby and no real visible game occurs | Confirmed |
| P1-016 | Placement recipe coordinates are fragile near bottom edge | PROMPT 2028 references FRAG-01: hand/submit coords at `fy=0.92`, about 58px from 720p bottom | Confirmed fragility |
| P1-017 | Old autoplay run clicked stale 720p coordinates after mid-run resize | PROMPT 2027 run `20260528-090613-Z`: window grew from 720px to 1076px high; auction/drag/submit clicks landed 302-328px above intended targets | Confirmed in old evidence; mitigated by PROMPT 1880 drift guard, needs fresh guarded run |
| P1-018 | Client-side placement hard-gating missing | User live report and PROMPT 2036 partial: invalid card drops can be staged visually and only fail at confirm, instead of being blocked at legal cells | Confirmed UX/gameplay blocker |
| P1-019 | Placement rejection feedback is missing or too weak | User live report and PROMPT 2036 partial: the player cannot tell why placement is invalid or what changed after rejection | Confirmed UX/gameplay blocker |
| P1-020 | Drag/drop can get stuck or lose focus | PROMPT 2036 partial lists stuck-drop focus edge as remaining gap after live ghost overlay work | Confirmed follow-up |
| P1-021 | Combat damage does not mutate visible board HP | PROMPT 2039: `CombatDamage` replay never mutates `BoardUnitStats` at `board_rendering.rs:1728-1754`; HP bars are dormant during resolution | Confirmed presentation bug |
| P1-022 | Placement reveal can silently abort | PROMPT 2039: reveal presentation aborts on missing `BoardLayout` or `CardAtlas` instead of surfacing a visible/error state | Confirmed silent-failure bug |
| P1-023 | Out-of-bounds board units are silently dropped | PROMPT 2039: `visible_unit_cell` drops OOB units without a visible/debuggable error path | Confirmed silent-failure bug |

## P1 Visual Bugs

| ID | Title | Evidence | Current status |
|---|---|---|---|
| V1-001 | All class card art is black | PROMPT 2026: class picker and selected-class preview render black card faces | Confirmed |
| V1-002 | Neutral class card is clipped | PROMPT 2026: 7th class card is partly outside picker row | Confirmed |
| V1-003 | Lobby header separator glyphs render as broken boxes | PROMPT 2026 screenshots show tofu/broken separator characters | Confirmed |
| V1-004 | Game content does not fill larger window | PROMPT 2026: large dark margins at 1280x1076 | Confirmed |
| V1-005 | Board baseline renders as tiny island | PROMPT 2026 older baseline: board about 400x310 on 1920x1080, no HUD chrome | Confirmed baseline issue |
| V1-006 | Auction baseline lacks card art and strong visual context | PROMPT 2026 older auction capture: text-focused card, sparse locked slots | Confirmed baseline issue |
| V1-007 | Room code input looks like debug text | PROMPT 2026: `Type room code: -------- - idle` without input styling | Confirmed |
| V1-008 | Snapshot QA button visible in play view | PROMPT 2026: top-right Snapshot button visible; acceptable only if QA flag is intentionally enabled | Advisory |
| V1-009 | Card art falls back to placeholders/question marks/empty labels | User live report and PROMPT 2038: many cards/shop entries show default placeholders, `?`, or `[]` instead of intended card art | Confirmed |
| V1-010 | Drag preview does not stick to cursor | User live report: dragged card appears offset/approximate, can freeze in one place, and does not behave like a cursor-attached card | Confirmed |
| V1-011 | Legal placement cells are not clearly highlighted | User live report and PROMPT 2036 partial: valid cells/invalid target feedback is incomplete; cyan/red overlay work is partial only | Confirmed |
| V1-012 | Hand fan spreads cards across the entire hand rectangle | User live report: cards distribute to the sides instead of clustering as a readable fan around a central hand area | Confirmed |
| V1-013 | Cards lack readable combat/stat/cost information | User live report: attack, HP, cost, class/type, and important labels are not visible enough on cards/units | Confirmed |
| V1-014 | Board units and combat have no convincing visible action | User live report plus PROMPT 2039: placed cards do not clearly become units, units do not visibly advance/fight/attack, and resolution looks inert | Confirmed |
| V1-015 | Global Bevy UI anchoring/layout is unreliable | User live report: shop, hand, board, confirm buttons, and multiple screens appear offset, clipped, or positioned from unstable boxes | Confirmed |
| V1-016 | Krosmaga asset and audio library is not fully wired into gameplay | User live report: imported Krosmaga-style card art, sprites, and sounds are not consistently used by the live game surfaces | Confirmed |

## P1 User-Observed Interaction Bugs (UX-*)

These IDs capture the 2026-05-28 user-live UI/UX failures so each behavior has a
stable handle even where it overlaps a P0/V1 entry above. Repair workers should
treat the UX-* row as the user-facing acceptance criterion and the linked
P0/V1/T row as the technical cause.

| ID | Title | Evidence | Linked IDs | Current status |
|---|---|---|---|---|
| UX-001 | Drag preview does not track the cursor during placement | User live 2026-05-28: hand card lifted, mouse moved across board, preview lags or stays anchored | P0-013 | Confirmed by user live play |
| UX-002 | Invalid placement cells appear acceptable, then confirm rejects | User live 2026-05-28: drop onto a cell that looks like a valid target, server/client then rejects at confirm; no pre-confirm visual cue distinguishes legal vs illegal cells | P1-005, V1-015 | Confirmed by user live play |
| UX-003 | No hover/highlight feedback on legal placement cells | User live 2026-05-28: dragging gives no visible signal which cells will accept the card | V1-015 | Confirmed by user live play |
| UX-004 | No drop feedback when a card is placed | User live 2026-05-28: releasing the mouse over a target produces no animation, no sfx cue, no state cue that the card landed | P0-014, P1-005 | Confirmed by user live play |
| UX-005 | Hand cards spread across the full hand rectangle instead of fanning | User live 2026-05-28: 1-7 cards spread evenly across the bottom hand strip; no fan curve, no focal overlap | V1-009 | Confirmed by user live play |
| UX-006 | Card faces show `?`/`[]` placeholders instead of art | User live 2026-05-28: draft, shop, hand and auction slots commonly show placeholder glyphs | V1-001, V1-010 | Confirmed by user live play |
| UX-007 | Card stats and labels are unreadable or missing | User live 2026-05-28: power/HP/cost/class/effect lines absent, cropped, or illegible at the actual rendered size | V1-011 | Confirmed by user live play |
| UX-008 | First-round shop is visually broken | User live 2026-05-28: initial DraftShop panel shows missing art, missing prices, empty-but-clickable slots | V1-012 | Confirmed by user live play |
| UX-009 | Placed cards do not appear as visible units on the board | User live 2026-05-28: even when placement is not rejected, the board remains empty visually through Placement and into Resolution | P0-014, P0-007 | Confirmed by user live play |
| UX-010 | Combat/Resolution is visually absent | User live 2026-05-28: Resolution advances state with no visible attacks, no damage flashes, no death animations, no objective HP feedback | V1-013, P0-008 | Confirmed by user live play |
| UX-011 | UI anchors inconsistently across window sizes | User live 2026-05-28: panels and chrome do not hold their intended relative anchors when the window resizes or starts at non-720p sizes | V1-014, V1-004, T-005, T-006 | Confirmed by user live play |
| UX-012 | Prior audit/test PASS labels overclaimed correctness vs live UX | PROMPT 2029 truthfulness audit + PROMPT 2034 user live observation: many prior PASS/SHIPPED markers (S8/S9, harness captures, recipe checkpoints) do not match what the user actually sees in a live two-client run | T-020..T-033 | Confirmed by user live play and prior audit |
| UX-013 | Placement phase is too short for a human to understand and act | User live report: placement expires quickly while the UI gives too little feedback | P0-010 | Confirmed by user live play |
| UX-014 | Confirm/place buttons do not communicate state | User live report: confirm behavior gives unclear success/failure and does not show why the chosen placement is legal or illegal | P1-019 | Confirmed by user live play |

## P1/P2 Tooling And Evidence Bugs

| ID | Title | Evidence | Current status |
|---|---|---|---|
| T-001 | Bevy screenshots freeze on Lobby | PROMPT 2024/2026: many checkpoint PNGs identical, split only by resize | Confirmed |
| T-002 | Win32 PrintWindow freezes | PROMPT 2024/2026: repeated frozen hashes; BitBlt fallback required | Confirmed |
| T-003 | BitBlt captures desktop background around window | PROMPT 2026: background windows visible around game window | Capture artifact |
| T-004 | Foreground control unreliable | PROMPT 2025: `SetForegroundWindow returned 0` repeatedly in earlier runs | Confirmed |
| T-005 | Window height changes mid-run | PROMPT 2024/2025/2026: 1280x720 logical becomes about 1280x1076; native height 759 -> 1115 | Confirmed |
| T-006 | Window resize not fully tracked in snapshots | PROMPT 2028 references PROMPT 1937 GAP-12; startup guard exists but mid-run resize still needs tracking/abort clarity | Confirmed gap |
| T-007 | Frozen renderer not detectable from snapshot JSON alone | PROMPT 2028 references PROMPT 1937 GAP-13 | Confirmed gap |
| T-008 | QA snapshot missing result outcome | PROMPT 2028 references PROMPT 1937 GAP-1/GAP-2 | Confirmed gap |
| T-009 | QA snapshot missing bot debug overlay | PROMPT 2028 references PROMPT 1937 GAP-3 | Confirmed gap |
| T-010 | QA snapshot missing card class/rarity in draft/shop slots | PROMPT 2028 references PROMPT 1937 GAP-4 | Confirmed gap |
| T-011 | QA snapshot missing auctioned card class/rarity | PROMPT 2028 references PROMPT 1937 GAP-5 | Confirmed gap |
| T-012 | QA snapshot missing board unit class ID | PROMPT 2028 references PROMPT 1937 GAP-6 | Confirmed gap |
| T-013 | Deprecated UI marker warnings | PROMPT 2025/2028: about 101 warnings around `HudEntity`, `HandUiEntity`, `ShopAuctionUiEntity` | Confirmed tech debt |
| T-014 | Duplicate bot decision timestamps | PROMPT 2025: repeated adjacent log entries with identical ms timestamps | Confirmed low severity |
| T-015 | `viewport_shrink_guard.py` not imported directly by driver | PROMPT 2028 references PROMPT 2018; equivalent guards are inlined, but module integration is unclear | Confirmed tooling cleanup |
| T-016 | Old run lacked mid-run viewport drift guard | PROMPT 2027: all three audited runs occurred before PROMPT 1880; run 3 drifted at tick 115 and continued dispatching stale clicks | Fixed/mitigated by PROMPT 1880, verify via fresh run |
| T-017 | Future autoplay evidence must prove current driver is active | PROMPT 2027: trusted runs should log `build_win=(1280x720)` and abort on `viewport_drift`; old runs did not have this field | Required verification gate |
| T-018 | Click OOB guard alone is insufficient | PROMPT 2027: stale 720p clicks remained technically in-bounds at 1076px height, but hit wrong UI locations | Covered by drift guard, not by OOB guard alone |
| T-019 | Intra-tick resize race remains theoretical | PROMPT 2027: resize after status poll but before action dispatch could evade a 10Hz drift check for about 100ms | Low severity, not actionable unless reproduced |
| T-020 | Screenshot validators lack semantic content checks | PROMPT 2029: analyzer requires exit 0, screenshot existence, and non-frozen/non-black pixels, but no expected UI text/regions/cards/HUD/phase content | Confirmed QA false-positive risk |
| T-021 | Recipe checkpoints are time-based, not state-based | PROMPT 2029: labels like `placement-submitted` mean ticks elapsed, not that placement UI was visible or server accepted action | Confirmed QA false-positive risk |
| T-022 | Static fractional click targets can miss silently | PROMPT 2029: recipes use fixed fractions and status API exposes no element geometry; missed clicks are not detected | Confirmed QA false-positive risk |
| T-023 | Two-client-runtime PASS does not validate visuals | PROMPT 2029: harness uses `MinimalPlugins`; no rendering, windowing, bevy_ui, sprites, HUD, or hand fan are exercised | Confirmed evidence taxonomy bug |
| T-024 | Harness empty-placement path bypasses placement UI | PROMPT 2029: friend-game harness accepts empty placements; bot-soak only attempts limited real placement, so drag/drop UI is mostly untested | Confirmed coverage gap |
| T-025 | `NEEDS_HUMAN_GUI` is not a blocking gate | PROMPT 2029: exit code 3 and `live_pass_status: NOT-CLAIMED` allow reports to continue without clean visual proof | Confirmed process bug |
| T-026 | Frozen-frame detection downgrades instead of failing | PROMPT 2029: frozen PrintWindow/BitBlt evidence leads to fallback or NEEDS_HUMAN_GUI, not hard FAIL | Confirmed QA false-positive risk |
| T-027 | Smoke recipe proves RPC substrate only | PROMPT 2029: smoke sends one key/click and checks screenshot creation, not game UI correctness | Confirmed evidence taxonomy bug |
| T-028 | Foreground failure can still capture stale frames | PROMPT 2029: `ensure_foreground()` failure falls through with warning, so DWM stale content may be captured | Confirmed QA false-positive risk |
| T-029 | Report-chain churn creates false progress signal | PROMPT 2029: many PROMPT-19xx/20xx commits are report reapplications, not new gameplay implementation or visual runs | Confirmed orchestration risk |
| T-030 | Harness-only Chrome screenshots are treated as integrated game evidence | PROMPT 2029: `production/qa/evidence/captures/**` visual artifacts are Chrome DevTools harness captures, not live Bevy game client sessions | Confirmed evidence taxonomy bug |
| T-031 | S8/S9 QA labels are misleading | PROMPT 2029: S8 `PASS WITH WARNINGS` was headless protocol trace only; S9 `No product defects found` meant no client GUI was reached | Confirmed misleading label |
| T-032 | `sau-011` focus screenshots are byte-identical across distinct scenarios | PROMPT 2029: four focus-state PNGs have identical 19390-byte size despite expected visual differences; harness self-report is not pixel proof | Confirmed harness evidence bug |
| T-033 | No recent real two-client visual baseline exists | PROMPT 2029: only real native two-client evidence is from 2026-05-12 around `f08b2c8`, hundreds of commits behind current main | Confirmed coverage blocker |

## Autoplay Click/Window Findings From PROMPT 2027

PROMPT 2027 directly explains the user-observed symptom: autoplay moved the
mouse and clicked into empty areas after the game window became the wrong size.

Key facts:

- Runs `20260528-051148-Z` and `20260528-063609-Z` had stable 1280x720 windows;
  all click coordinates matched their intended fractional targets.
- Run `20260528-090613-Z` resized mid-run from 1280x720 logical to about
  1280x1076 logical.
- After the resize, stale 720p coordinates were still used: auction click at
  y=612 became actual fraction 0.57 instead of intended 0.85; hand drag source
  at y=662 became 0.62 instead of intended 0.92; submit at y=662 missed by
  about 328px.
- The run occurred before PROMPT 1880, so no drift guard existed yet.
- Current code with PROMPT 1880 should abort at the first large drift
  (`|505 - 720| = 215 > 10`) before dispatching these stale actions.

Required future evidence before trusting autoplay as QA:

- `driver.log` includes `build_win=(1280x720)`.
- No `viewport_drift` aborts appear in `checkpoints.jsonl`.
- `phase_label` progresses through real phases instead of staying `Lobby`.
- BitBlt or equivalent live capture exists per checkpoint.

## QA Evidence Truthfulness Findings From PROMPT 2029

PROMPT 2029 explains why many previous PASS/SHIPPED labels did not match the
visible state of the game. The audit separates tooling health from playable
game evidence.

Key facts:

- No clean automated gameplay PASS exists for the audited autoplay game runs.
  The available 2026-05-28 bot-game runs are PARTIAL/NEEDS_HUMAN_GUI, not
  proof that gameplay works.
- The current validators check artifact existence, exit codes, rough brightness,
  and frozen-frame patterns. They do not assert that the right screen, cards,
  hand, HUD, board, or phase-specific UI is visible.
- Recipe checkpoints are mostly time-based. A checkpoint label can be emitted
  after a wait even if the click missed, the phase never changed, or the client
  stayed in Lobby.
- `two-client-runtime` uses Bevy `MinimalPlugins`, so a PASS there proves
  protocol/server flow only. It does not exercise rendering or UI.
- Chrome harness captures validate isolated widget layouts, not the integrated
  Bevy game client connected to a live server.
- The only real native two-client visual evidence found is from 2026-05-12,
  around commit `f08b2c8`, far behind the current main lineage.

Six misleading labels/artifact classes specifically flagged by PROMPT 2029:

- S8 friend-game smoke `PASS WITH WARNINGS`: no native/browser two-client GUI
  run was attempted; only controlled protocol traces ran.
- S9 manual game-over `No product defects found`: the client GUI was never
  launched, so no product path was exercised.
- UI screenshots under `production/qa/evidence/captures/**`: Chrome harness
  captures, not live integrated game-client captures.
- `sau-011` focus screenshots: distinct focus scenarios produced identical PNG
  byte counts, so harness self-report outran pixel evidence.
- Real two-client evidence: last genuine native two-client session is stale
  evidence from 2026-05-12, not current main.
- PROMPT-19xx/20xx report chains: many are administrative report refreshes
  after NOT_FF rejections, not new gameplay evidence.

Required evidence taxonomy before accepting future Done/PASS claims:

- For interactive gameplay, require at least one artifact from a real running
  Bevy client, not only headless protocol tests or HTML harness captures.
- Treat `NEEDS_HUMAN_GUI` as blocking until a human/operator review is actually
  attached.
- Add semantic visual validators: phase-gated recipes, checkpoint phase labels,
  region/pixel assertions for HUD/hand/board, phase-to-phase screenshot
  distinctness, and click/placement acceptance checks.

## Server Board/GameOver Findings From PROMPT 2033

PROMPT 2033 fixed one concrete server RSM bug and narrowed the remaining
board/GameOver failures to upstream causes.

Fixed:

- `submissions_received` now clears when Resolution advances into the next
  round's DraftShop or DraftAuction phase. This prevents stale round-N
  placement submissions from making round N+1 Placement close immediately.

Root-caused but not fixed in 2033:

- The observed GameOver with all objectives intact is not a broken objective
  win-condition. In the audited bot-soak run it came from the intentional
  `CCGS_BOT_MAX_ROUNDS=3` cap after an empty/no-combat match.
- No units reach the board because both submitting paths are empty: the client
  remains stuck in Lobby and the bot path lacks a usable hand/placement.
- Instant Resolution is a cascade of the empty board: no units means no combat
  sequence to simulate.
- `session: null` in the final GameOver snapshot is a teardown/snapshot ordering
  issue, not proof that the GameOver broadcast itself failed.

Required follow-up:

- Verify the 2033 RSM fix on current main after 2030-2032 land.
- Keep P0-007/P0-008 open until the client phase sync, bot hand awarding, and
  bot placement repairs produce units on the board and a non-empty combat
  resolution in fresh evidence.
- Launch a focused snapshot ordering repair for P1-013 if GameOver evidence
  still needs `session` populated after core gameplay repairs.

## Flow Matrix

| Stage | Human flow | Autoplay flow | State |
|---|---|---|---|
| Server startup | Works in evidence | Works in evidence | Not main blocker |
| Client launch/connect | Previously blocked by stale binary protocol mismatch; fresh rebuild mitigates that case | Driver connects | Requires fresh-binary guard |
| Lobby/create room | Server path partly verified; UI not fully human-tested | Client remains Lobby forever | Blocked/degraded |
| Class select/confirm | Server path partly verified; UI not fully human-tested | Client remains Lobby forever | Blocked/degraded |
| DraftInitial | Server path exists but card awarding broken in snapshots | Client remains Lobby forever | Blocked |
| DraftShop | Server advances too fast and no useful awarding observed | Client remains Lobby forever | Blocked |
| Auction | Server path exists; visible UI not reached in current run | Client remains Lobby forever | Blocked |
| Placement | Server accepts/submits without board units; bot failsafe spin-loop | Client remains Lobby forever | Blocked; stale submission leak fixed by 2033 |
| Resolution | Empty board, no combat | Client remains Lobby forever | Blocked; root-caused by 2033 as no-units cascade |
| GameOver | Soak max-round cap can end an empty match; normal win-condition path is guarded | Client remains Lobby forever | Blocked until non-empty board/combat evidence exists |
| Return to Lobby | No reliable evidence | Unknown | Unknown |

## Repair Wave Mapping

Repair workers after PROMPT 2033:

- `PROMPT 2030`: client phase sync P0 repair. Outcome PARTIAL/diagnostic; still targets P0-001, P0-002, P0-003, and P0-013.
- `PROMPT 2031`: server draft/hand awarding P0 repair. Landed on main at `28482bd5`; targets P0-004/P0-005 server-side bot hand awarding race.
- `PROMPT 2032`: bot placement failsafe spin-loop P0 repair. Landed on main at `e1a61376`; targets P0-006 and placement failsafe timing.
- `PROMPT 2036`: placement drag/drop legal-cell feedback repair. Outcome PARTIAL; added live ghost/legal-cell overlay but left hard-gating, rejection banner, scaling drift, and stuck-drop focus gaps.
- `PROMPT 2038`: card asset/shop placeholder binding repair. Outcome SHIPPED; still needs fresh visual verification because user reports broad placeholder/art failures.
- `PROMPT 2039`: board unit/combat presentation audit map. Outcome SHIPPED audit only; no code repair yet for combat HP mutation/reveal silent failures.

Completed repair outcomes:

- `PROMPT 2033`: server board/GameOver vacuous-flow P0 repair. Fixed P1-010,
  root-caused P0-007, P0-008, P0-009, P0-010, and P1-013.
- `PROMPT 2031`: fixed bot draft auto-pick debounce before `PlayerEconomy`.
- `PROMPT 2032`: fixed placement failsafe phase timing arming on Placement entry.

Repair workers not yet launched from this register:

- Client phase sync repair follow-up from PROMPT 2030 diagnostics for P0-001 through P0-003 and P0-013.
- Placement hard-gating/rejection UX/focus repair for P1-018 through P1-020 and UX-001 through UX-005.
- Board/combat presentation repair for P1-021 through P1-023 and V1-014.
- Card art/shop placeholder verification and follow-up repair for V1-009 and UX-006 after PROMPT 2038.
- Hand fan/card readability/layout repair for V1-012/V1-013.
- Global UI anchoring/layout audit and repair for V1-015.
- Asset/audio wiring audit and repair for V1-016.
- Disconnect tracker initialization repair for P1-001.
- Placement ACK protocol repair for P1-005.
- Result outcome projection/snapshot repair for P1-006 and T-008.
- Lobby visible state repair for P1-002, P1-003, P1-004.
- Class art/Neutral clipping/lobby polish repair for V1-001 through V1-004.
- Capture/window evidence hardening for T-001 through T-007 and verification
  gates for T-016 through T-019.
- QA evidence truthfulness hardening for T-020 through T-033, especially
  semantic screenshot validation, phase-gated checkpoints, real-client evidence
  taxonomy, and blocking treatment for `NEEDS_HUMAN_GUI`.
- GameOver snapshot ordering repair for P1-013 if fresh post-repair evidence
  still requires non-null session data in final snapshots.
- Placement drag/preview repair for P0-013, UX-001, UX-003, UX-004, V1-015.
- Placement legality preview / pre-confirm rejection feedback repair for
  P0-013, P1-005, V1-015, UX-002, UX-003.
- Board unit visibility repair for P0-014, UX-009 (downstream of P0-007 board
  cascade but must own the visible-unit acceptance criterion).
- Hand fan layout repair for V1-009, UX-005.
- Card art and stats rendering repair for V1-001, V1-010, V1-011, UX-006,
  UX-007 (covers placeholder glyphs and missing/illegible stats across draft,
  hand, shop, and auction).
- First-round shop initialization/visual repair for V1-012, UX-008.
- Resolution/combat visualization repair for V1-013, UX-010 (visible attacks,
  damage feedback, objective HP cues).
- Global bevy_ui anchoring/responsive layout repair for V1-014, V1-004, UX-011,
  with cross-checks against T-005/T-006 window-size tracking.
- Audit/test label truthfulness backstop repair for UX-012, reinforcing the
  T-020..T-033 evidence-taxonomy work with explicit user-facing acceptance
  criteria before any future PASS/SHIPPED label is honored.

## Rules For Future Updates

- Add new bug IDs instead of renumbering existing IDs.
- If a bug is fixed, change `Current status` to `Fixed by PROMPT N` and add the
  verifying report path.
- Merge repair outcomes from `PROMPT 2030+` into this file when they land.
- Do not treat driver checkpoint success as proof of visible game progress unless
  screenshots, driver timeline, server snapshots, and logs agree.
