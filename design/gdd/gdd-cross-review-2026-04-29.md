# Cross-GDD Review Report

**Date:** 2026-04-29 (Revision 5 — 9 GDDs, post-network-protocol R2 propagation fixes)
**GDDs Reviewed:** 9 (6 approved foundation + 3 in-review)
**Systems Covered:** Card Data & Pool, Game Config, Server-side RNG, Economy System, Board/Lane System, Round State Machine, Objective System, Network Protocol, Game Session System
**Registry baseline:** entities.yaml v1

---

## Progress History

| Review | Verdict | Blockers |
|---|---|---|
| R1 2026-04-29 | FAIL | 10 |
| R2 2026-04-29 | FAIL | 7 |
| R3 2026-04-29 | **PASS** | 0 (6 GDDs) |
| R4 2026-04-29 | FAIL→PASS | 2 (objective-system.md — both fixed inline) |
| R5 2026-04-29 | **FAIL→PASS** | 2 (S2CGameOver schema propagation — both fixed inline) |

All 15 cumulative blocking issues resolved. All 9 GDDs consistent.

---

## R5 Blocking Issues (resolved inline)

✅ **C-B1 — `S2CGameOver.loser` field type mismatch** *(FIXED)*
network-protocol.md R2 changed `loser` to `Option<PlayerId>` (required by Draw case in RSM-22/RSM-37) but round-state-machine.md Rule 14, the UI Requirements table, and entities.yaml still declared `PlayerId`. Fixed: RSM Rule 14 + UI table + entities.yaml all updated to `Option<PlayerId>` with `None = Draw` note.

✅ **C-B2 — `GameOverReason` enum variant set inconsistent** *(FIXED)*
network-protocol.md edge cases and NP-25 referenced non-canonical variants (`MutualDisconnect`, `ResolutionTimeout`, `Disconnect`). Canonical 3-variant enum `{ObjectivesDestroyed, Disconnection, Draw}` is defined in RSM Rule 14 and the registry. Fixed: NP dual-disconnect edge case updated to `GameOverReason::Draw`; NP-25 updated to `GameOverReason::Disconnection`; S2CGameOver doc comment updated.

---

## Consistency Issues

### Blocking (all resolved)
*None outstanding.*

### Warnings

⚠️ **C-W1 — BLS F2 missing `fake_objective_spawn_advance` multiplier**
board-lane-system.md F2 uses bare `fakes_A` count; card-data-pool.md F3 multiplies by `fake_objective_spawn_advance`. At default value (1) both produce identical results. At advance=2 they diverge. Reconcile in next BLS revision.

⚠️ **C-W2 — 2v2 spawn-range counter ownership unresolved (per-player vs per-team)**
card-data-pool.md Formula 3 + AC CP16 assume team-shared counter; board-lane-system.md OQ5 recommends per-player and flags as open. 2v2 and 3v3 are in full scope — this must be resolved before `TwoVTwo` implementation begins. Design decision needed in a GDD revision or quick-design spec.

⚠️ **C-W3 — `lobby_heartbeat_timeout_seconds` not yet in `game-config.md` or `entities.yaml`**
game-session-system.md Rule 9 + Tuning Knobs require this field (default 15s). GSS Open Question 8 explicitly tracks it. Add before GSS implementation.

⚠️ **C-W4 — `economy-system.md` EC19/EC20 test removed feature (`reserve_mana_cap`)**
Two ACs reference a cap that was removed by design decision (Economy OQ2). Delete EC19 and EC20 on next economy-system.md revision. (Carried from R3/R4.)

⚠️ **C-W5 — `round-state-machine.md` still labels authored GDDs as "GDD not yet written"**
Objective System, Network Protocol, Board/Lane System, and Game Session System are all authored. Update RSM Interactions/Dependencies tables on next RSM revision.

⚠️ **C-W6 — `round-state-machine.md` Tuning Knobs note: stale "must be added to game-config.md"**
`auction_max_duration_seconds` and `resolution_max_duration_seconds` are already in game-config.md. Delete the note.

⚠️ **C-W7 — `economy-system.md` Interactions table omits `interest_threshold_gold` and `refresh_base_cost`**
Dependencies table further down has both. Trivial cleanup to align tables.

⚠️ **C-W8 — `game-config.md` Objective System interactions row incomplete**
Lists only `objective_hp`, `fake_count`; objective-system.md also reads `objective_gold_reward` and `fake_objective_spawn_advance`. (Carried from R4 as C-W3.)

⚠️ **C-W9 — `server-rng.md` does not list `game-session-system.md` as session lifecycle owner**
GSS Rule 11 initializes and destroys ServerRng. Add GSS as a downstream dependency row in server-rng.md.

⚠️ **C-W10 — Master GDD §3.3 + AC C9 say "in that lane"; §3.2 says "global"**
§3.2 is authoritative ("global — all lanes simultaneously"). §3.3 and AC C9 retain stale "in that lane" wording. Update master GDD on next master revision.

⚠️ **C-W11 — `C2SAcknowledgeResult` valid phase contradicts RSM Rule 15 "None"**
RSM Rule 15 lists GAME_OVER accepted actions as "None"; NP C2S table allows `C2SAcknowledgeResult` in GAME_OVER (it is a UI handshake, not a game-state change). Clarify RSM Rule 15 wording.

---

## Game Design Issues

### Blocking
*None.*

### Warnings

⚠️ **D-W1 (carried) — Double-fake-by-round-6 economy snowball**
+6g + up to +2 mana_cap from both fake destructions, potentially crossing interest brackets. Monitor in first playtests.

⚠️ **D-W2 (carried) — RESOLUTION zero player decisions**
6 server-executed sub-steps; player is passive. Mitigated by reveal animations and live information watching. Monitor vs "no idle spectating" pillar.

⚠️ **D-W3 (carried) — Manual shop refresh is the only Rare-without-auction path**
Escalating refresh cost (1g/2g/3g…) can conflict with auction participation budget. Validate in playtests.

⚠️ **D-W4 (NEW) — Xelor Garde-Temps + uncapped reserve = single-spell objective deletion**
No reserve cap (Economy OQ2 design decision). Garde-Temps costs 20 reserve and destroys any objective with no positional requirement — bypassing the lane/bluff game entirely. Xelor can realistically accumulate 20+ reserve by round ~8–10 via Gelure, Miss Nuit, and Lane 2/4 prisms. Recommend monitoring in playtests; consider per-game Garde-Temps use limit if Xelor win-rate trends high.

⚠️ **D-W5 (NEW) — PLACEMENT cognitive load (~5 active systems in 10 seconds)**
Players simultaneously manage card selection, lane choice, spawn-range constraint, hidden-info deduction, and mana split. 10s is intentionally tight ("no idle spectating"). Watch placement-timeout rate; consider 15s default if >20% of PLACEMENT phases time out.

*Previously D-W3 (class confirmation order delta) — RESOLVED: GSS Rule 7 deferred simultaneous reveal means neither player sees the opponent's class until both lock. ✓*

---

## Cross-System Scenario Issues

**Scenarios walked: 5**

### Blockers
*None (C-B1 resolution also resolves S1 scenario).*

### Warnings
⚠️ **S2 — Auction AbortAuction gold reservation behavior undefined**
When RSM sends `AbortAuction` during a disconnect GAME_OVER mid-auction: it is unspecified whether the leading bidder's gold reservation releases or whether the auctioned card returns to the shared pool. Auction System GDD must specify.

### Info
ℹ️ **S3 — Reconnect during PLACEMENT reveal**: snapshot-only recovery is correct and intentional.
ℹ️ **S5 — DRAFT_INITIAL gold forfeiture**: RSM-30 says "zeroed"; Economy GDD doesn't confirm. Trivial alignment.

---

## GDDs Flagged for Revision

| GDD | Warning | Priority |
|---|---|---|
| `round-state-machine.md` | C-W5 (stale "not yet written" labels), C-W6 (stale config note), C-W11 (Rule 15 wording) | Warning |
| `economy-system.md` | C-W4 (EC19/EC20 stale ACs), C-W7 (Interactions table) | Warning |
| `game-config.md` | C-W3 (lobby_heartbeat_timeout_seconds), C-W8 (Objective System row) | Warning |
| `board-lane-system.md` | C-W1 (F2 multiplier), C-W2 (2v2 OQ5 unresolved) | Warning |
| `card-data-pool.md` | C-W2 (2v2 OQ5 — opposite side) | Warning |
| `server-rng.md` | C-W9 (GSS lifecycle ownership) | Warning |
| `lanes-and-lies-gdd.md` | C-W10 (§3.3 + C9 "in that lane" stale) | Warning |

---

## Verdict: PASS (post-inline fixes)

Both blocking issues were fixed inline (RSM Rule 14, NP edge cases, NP-25, entities.yaml). All 9 GDDs are now consistent. Architecture can begin.
