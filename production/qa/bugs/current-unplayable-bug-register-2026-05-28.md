# Current Unplayable Bug Register

Date: 2026-05-28
Source of truth when written: `origin/main@41a7bbc9`

This register consolidates bugs found by the current forensic audit wave so they
are not lost in worker reports or chat history.

Primary sources:

- `reports/PROMPT-2024-forensic-evidence-inventory-and-run-selection.md`
- `reports/PROMPT-2025-snapshot-log-gamestate-correlation-audit.md`
- `reports/PROMPT-2026-visible-screen-screenshot-visual-bug-audit.md`
- `reports/PROMPT-2027-autoplay-input-click-target-forensic-audit.md`
- `reports/PROMPT-2028-player-flow-unplayable-bug-classification.md`

Pending sources to merge later:

- `PROMPT 2029` QA evidence/tooling truthfulness audit
- Repair outcomes from `PROMPT 2030-2033`

## Executive State

The game is currently unplayable end to end.

The most important confirmed failure is that the client never transitions from
Lobby to InSession in the audited autoplay runs. The server advances through a
vacuous two-round match, but the visible client stays in Lobby for every
checkpoint. In parallel, the server/bot path shows missing bot hand state, empty
placement failsafes, no board units, no combat, and GameOver with all objectives
still full HP.

## P0 Blockers

| ID | Title | Evidence | Current status |
|---|---|---|---|
| P0-001 | Client never transitions to InSession | All three autoplay driver timelines: `client_state_label: "Lobby"` and `phase_label: "Lobby"` for all ticks | Confirmed |
| P0-002 | No in-game screen ever renders | PROMPT 2026 screenshots: shop, auction, placement, resolution, and post-resolution checkpoints all show Lobby | Confirmed |
| P0-003 | Server advances a match the client does not visually join | PROMPT 2025: server snapshots progress DraftInitial -> Placement -> Resolution -> DraftShop -> Placement -> Resolution -> GameOver while client remains Lobby | Confirmed |
| P0-004 | Bot player has no hand | PROMPT 2025 snapshots from first Placement onward contain hand for player 1 only; bot hand missing | Confirmed |
| P0-005 | Draft/card awarding broken | Player 1 keeps one card `[5]`; bot has no hand; DraftInitial and DraftShop do not award usable cards | Confirmed |
| P0-006 | Bot placement failsafe spin-loop | `empty_placement_failsafe` logged about 16,229 times in a two-round game, roughly 1/ms | Confirmed |
| P0-007 | No units ever reach board | Board counts stay zero across minions/traps/structures/fields; `per_player_minions` empty | Confirmed |
| P0-008 | No combat occurs | Resolution phases run with empty board and all objectives unchanged | Confirmed |
| P0-009 | GameOver fires vacuously | GameOver after two empty rounds with all 10 objectives at 5/5 HP, none destroyed | Confirmed |
| P0-010 | Phase timers are bypassed | DraftInitial, DraftShop, and Resolution transition in milliseconds despite 30-60s configured timers | Confirmed |
| P0-011 | No successful human GUI end-to-end flow on record | PROMPT 2028/1883: no verified human flow through room, session, draft/shop, placement, resolution, GameOver | Confirmed coverage blocker |
| P0-012 | Human two-client stale binary protocol panic | PROMPT 1883: stale `client.exe` vs fresh `server.exe` caused Lightyear protocol mismatch before UI | Confirmed operational blocker; rebuild mitigates stale-binary case |

## P1 Major Gameplay And State Bugs

| ID | Title | Evidence | Current status |
|---|---|---|---|
| P1-001 | Disconnect trackers initialized as disconnected for both players | PROMPT 2024/2025 snapshots show `seconds_since_disconnect` around 30000s from early game | Confirmed |
| P1-002 | Lobby room code never populates | PROMPT 2026 screenshots show `Room: ----` after bot add and lobby confirm checkpoints | Confirmed |
| P1-003 | Lobby player count never updates | PROMPT 2026 screenshots show `Players: 0/1` throughout | Confirmed |
| P1-004 | Class confirmation status never clears | PROMPT 2026 screenshots show `not confirmed` after class-confirmed checkpoint | Confirmed |
| P1-005 | Placement ACK missing; client relies on heuristic | PROMPT 2028 references PROMPT 1937 GAP-7: `S2CPlacementAck` still not shipped | Confirmed design gap |
| P1-006 | Result screen does not project win/loss/draw outcome | PROMPT 2028 references PROMPT 1937 GAP-1/GAP-2 and result-screen gap | Confirmed design/evidence gap |
| P1-007 | Player gold drops without recorded purchase | PROMPT 2025: player 1 gold 5 -> 3 at DraftInitial -> Placement without charge evidence | Confirmed |
| P1-008 | `draft_ready` logs `legal_action_count: null` | PROMPT 2025: DraftInitial and DraftShop bot decisions use null legal count | Confirmed |
| P1-009 | `draft_ready_players` never records ready players | PROMPT 2025 snapshots show empty list despite `draft_ready` decisions | Confirmed |
| P1-010 | `submissions_received` leaks into next round | PROMPT 2025: `[1]` persists into round 2 DraftShop | Confirmed |
| P1-011 | Bot RNG path not consumed | PROMPT 2025: `rng_word_counter` remains 0 despite thousands of bot decisions | Confirmed |
| P1-012 | Bot decision timestamps/deadlines stale or null | PROMPT 2025: `last_decision_at_ms` stuck; `next_decision_at_ms` and `failsafe_deadline_ms` null during spin-loop | Confirmed |
| P1-013 | Final GameOver snapshot loses session | PROMPT 2025: final GameOver snapshot has `session: null` | Confirmed |
| P1-014 | `client_exit_code` never observed | PROMPT 2025: all launcher statuses have `client_exit_code: null` while outcome is `ok` | Confirmed |
| P1-015 | Autoplay `outcome: ok` is misleading | Checkpoints can pass while client stays Lobby and no real visible game occurs | Confirmed |
| P1-016 | Placement recipe coordinates are fragile near bottom edge | PROMPT 2028 references FRAG-01: hand/submit coords at `fy=0.92`, about 58px from 720p bottom | Confirmed fragility |
| P1-017 | Old autoplay run clicked stale 720p coordinates after mid-run resize | PROMPT 2027 run `20260528-090613-Z`: window grew from 720px to 1076px high; auction/drag/submit clicks landed 302-328px above intended targets | Confirmed in old evidence; mitigated by PROMPT 1880 drift guard, needs fresh guarded run |

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
| Placement | Server accepts/submits without board units; bot failsafe spin-loop | Client remains Lobby forever | Blocked |
| Resolution | Empty board, no combat | Client remains Lobby forever | Blocked |
| GameOver | Fires with objectives full HP; outcome projection untrusted | Client remains Lobby forever | Blocked |
| Return to Lobby | No reliable evidence | Unknown | Unknown |

## Repair Wave Mapping

Active repair workers at the time this register was written:

- `PROMPT 2030`: client phase sync P0 repair. Targets P0-001, P0-002, P0-003.
- `PROMPT 2031`: server draft/hand awarding P0 repair. Targets P0-004, P0-005, P1-007, P1-008, P1-009.
- `PROMPT 2032`: bot placement failsafe spin-loop P0 repair. Targets P0-006, P1-011, P1-012, T-014.
- `PROMPT 2033`: server board/GameOver vacuous-flow P0 repair. Targets P0-007, P0-008, P0-009, P0-010, P1-010, P1-013.

Repair workers not yet launched from this register:

- Disconnect tracker initialization repair for P1-001.
- Placement ACK protocol repair for P1-005.
- Result outcome projection/snapshot repair for P1-006 and T-008.
- Lobby visible state repair for P1-002, P1-003, P1-004.
- Class art/Neutral clipping/lobby polish repair for V1-001 through V1-004.
- Capture/window evidence hardening for T-001 through T-007 and verification
  gates for T-016 through T-019.

## Rules For Future Updates

- Add new bug IDs instead of renumbering existing IDs.
- If a bug is fixed, change `Current status` to `Fixed by PROMPT N` and add the
  verifying report path.
- Merge `PROMPT 2029` findings into this file when they land.
- Do not treat driver checkpoint success as proof of visible game progress unless
  screenshots, driver timeline, server snapshots, and logs agree.
